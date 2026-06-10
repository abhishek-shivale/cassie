use crate::aggregation::{compute_reward_split, RewardSplit};
use crate::constants::*;
use crate::error::CassieError;
use crate::{
    CouncilTotal, DisputeConfig, OracleConfig, Outcome, Question, QuestionSettled, QuestionState,
    Resolver,
};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
#[instruction(hash: [u8; 32])]
pub struct Settle<'info> {
    #[account(mut)]
    pub cranker: Signer<'info>,

    #[account(
        mut,
        seeds = [QUESTION_CONFIG_SEED.as_bytes(), hash.as_ref()],
        bump,
    )]
    pub question: Box<Account<'info, Question>>,

    #[account(
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump,
    )]
    pub config: Box<Account<'info, OracleConfig>>,

    #[account(
        seeds = [OUTCOME_SEED.as_bytes(), hash.as_ref()],
        bump,
    )]
    pub outcome: Box<Account<'info, Outcome>>,

    #[account(
        address = USDC_PUBKEY
    )]
    pub usdc_mint: Box<InterfaceAccount<'info, Mint>>,

    // per-question reward pool (authority = question PDA)
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = question,
    )]
    pub pool_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = config.treasury,
    )]
    pub treasury_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    // present only if the question was disputed. settle marks won/lost here.
    #[account(
        mut,
        seeds = [DISPUTE_SEED.as_bytes(), hash.as_ref()],
        bump,
    )]
    pub dispute: Option<Box<Account<'info, DisputeConfig>>>,

    // present only when the question was resolved by council. settle reads the
    #[account(
        seeds = [COUNCIL_TOTAL_SEED.as_bytes(), hash.as_ref()],
        bump,
    )]
    pub council_total: Option<Box<Account<'info, CouncilTotal>>>,

    pub callback_program: Option<UncheckedAccount<'info>>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Settle<'info> {
    pub fn settle(&mut self, hash: [u8; 32], remaining: &[AccountInfo<'info>]) -> Result<()> {
        require!(!self.config.freeze, CassieError::ProgramFrozen);
        require!(
            matches!(self.question.state, QuestionState::Resolved),
            CassieError::InvalidState
        );

        // SOL-006: remaining forwarded to callback CPI only; not mutated here.
        if self.question.callback_program == Pubkey::default() {
            require!(remaining.is_empty(), CassieError::CallbackInvocationFailed);
        }

        let now = Clock::get()?.unix_timestamp;
        require!(
            now > self.question.dispute_deadline,
            CassieError::DisputeWindowActive
        );
        require!(
            !self.question.has_dispute || self.dispute.is_some(),
            CassieError::MissingDisputeAccount
        );

        let result = self.outcome.result;
        let (correct_answer_count, loser_answer_stake) = if result {
            (self.question.yes_count as u64, self.question.total_no_stake)
        } else {
            (self.question.no_count as u64, self.question.total_yes_stake)
        };

        let (treasury_cut, total_slash) = match self.outcome.resolver {
            Resolver::Optimistic => {
                self.settle_optimistic(hash, result, correct_answer_count, loser_answer_stake)?
            }
            Resolver::Council => {
                let (correct_council_count, loser_council_stake) = {
                    let ct = self
                        .council_total
                        .as_ref()
                        .ok_or(CassieError::MissingCouncilAccount)?;
                    if result {
                        (ct.yes_count as u64, ct.total_no_stake)
                    } else {
                        (ct.no_count as u64, ct.total_yes_stake)
                    }
                };
                self.settle_council(
                    hash,
                    result,
                    correct_answer_count,
                    loser_answer_stake,
                    correct_council_count,
                    loser_council_stake,
                )?
            }
        };

        self.question.state = QuestionState::Settled;
        self.outcome.settled_at = now;

        if self.question.callback_program != Pubkey::default() {
            self.fire_callback(hash, remaining)?;
        }

        emit!(QuestionSettled {
            hash,
            result,
            treasury_cut,
            per_answer_reward: self.question.per_answer_reward,
            slash_amount: total_slash,
        });

        Ok(())
    }

    fn settle_optimistic(
        &mut self,
        hash: [u8; 32],
        result: bool,
        correct_count: u64,
        loser_stake: u128,
    ) -> Result<(u64, u64)> {
        let split = compute_reward_split(
            self.question.bounty,
            loser_stake,
            self.config.slash_bps as u16,
            self.config.treasury_bps as u16,
        );

        let mut answer_pool = split.total;
        if let Some(dispute) = &mut self.dispute {
            if dispute.claimed_outcome == result {
                let dispute_share = ((split.slash_amount as u128) * DISPUTE_REWARD_BPS as u128
                    / BPS_DENOMINATOR) as u64;
                dispute.resolved = true;
                dispute.reward = dispute.bond_amount.saturating_add(dispute_share);
                answer_pool = answer_pool.saturating_sub(dispute_share);
            } else {
                dispute.resolved = false;
                dispute.reward = 0;
            }
        }

        self.transfer_treasury(hash, split.treasury_cut)?;

        self.question.per_answer_reward = if correct_count == 0 {
            0
        } else {
            answer_pool / correct_count
        };

        Ok((split.treasury_cut, split.slash_amount))
    }

    fn settle_council(
        &mut self,
        hash: [u8; 32],
        result: bool,
        correct_answer_count: u64,
        loser_answer_stake: u128,
        correct_council_count: u64,
        loser_council_stake: u128,
    ) -> Result<(u64, u64)> {
        let slash_bps = self.config.slash_bps as u128;
        let treasury_bps = self.config.treasury_bps as u128;
        let council_bps = self.config.council_bps as u128;

        let council_slash_bps = (slash_bps * 2).min(BPS_DENOMINATOR);

        let answer_slash = (loser_answer_stake * slash_bps / BPS_DENOMINATOR) as u64;
        let council_slash = (loser_council_stake * council_slash_bps / BPS_DENOMINATOR) as u64;

        let gross = (self.question.bounty as u128)
            .saturating_add(answer_slash as u128)
            .saturating_add(council_slash as u128) as u64;

        let treasury_cut = ((gross as u128) * treasury_bps / BPS_DENOMINATOR) as u64;

        let council_pool = ((gross as u128) * council_bps / BPS_DENOMINATOR) as u64;

        let distributable = gross.saturating_sub(treasury_cut);
        let mut answer_pool = distributable.saturating_sub(council_pool);

        if let Some(dispute) = &mut self.dispute {
            if dispute.claimed_outcome == result {
                let dispute_share =
                    ((answer_slash as u128) * DISPUTE_REWARD_BPS as u128 / BPS_DENOMINATOR) as u64;
                dispute.resolved = true;
                dispute.reward = dispute.bond_amount.saturating_add(dispute_share);
                answer_pool = answer_pool.saturating_sub(dispute_share);
            } else {
                dispute.resolved = false;
                dispute.reward = 0;
            }
        }

        self.transfer_treasury(hash, treasury_cut)?;

        self.question.per_answer_reward = if correct_answer_count == 0 {
            0
        } else {
            answer_pool / correct_answer_count
        };
        self.question.council_reward_per_vote = if correct_council_count == 0 {
            0
        } else {
            council_pool / correct_council_count
        };

        Ok((treasury_cut, answer_slash + council_slash))
    }

    fn transfer_treasury(&self, hash: [u8; 32], amount: u64) -> Result<()> {
        if amount == 0 {
            return Ok(());
        }
        let (_, canonical_bump) = Pubkey::find_program_address(
            &[QUESTION_CONFIG_SEED.as_bytes(), hash.as_ref()],
            &crate::id(),
        );
        let seeds: &[&[u8]] = &[QUESTION_CONFIG_SEED.as_bytes(), hash.as_ref(), &[canonical_bump]];
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.key(),
                TransferChecked {
                    from: self.pool_ata.to_account_info(),
                    to: self.treasury_ata.to_account_info(),
                    mint: self.usdc_mint.to_account_info(),
                    authority: self.question.to_account_info(),
                },
                &[seeds],
            ),
            amount,
            self.usdc_mint.decimals,
        )?;
        Ok(())
    }

    fn fire_callback(&self, hash: [u8; 32], remaining: &[AccountInfo<'info>]) -> Result<()> {
        let target = self.question.callback_program;
        let program_ai = self
            .callback_program
            .as_ref()
            .ok_or(CassieError::CallbackInvocationFailed)?;
        require_keys_eq!(
            program_ai.key(),
            target,
            CassieError::CallbackInvocationFailed
        );
        require!(program_ai.executable, CassieError::CallbackInvocationFailed);

        let mut data = Vec::with_capacity(8 + 32 + 1);
        data.extend_from_slice(&self.question.callback_discriminator);
        data.extend_from_slice(&hash);
        data.push(self.outcome.result as u8);

        // SOL-006: remaining accounts forwarded to callback CPI; runtime enforces signer/writable.
        let metas: Vec<AccountMeta> = remaining
            .iter()
            .map(|a| AccountMeta {
                pubkey: a.key(),
                is_signer: a.is_signer,
                is_writable: a.is_writable,
            })
            .collect();

        let ix = Instruction {
            program_id: target,
            accounts: metas,
            data,
        };

        // Authority: question creator authorized this callback by setting
        // callback_program + callback_discriminator at ask_question time (signed).
        // The question PDA (verified by Anchor seeds constraint) signs the CPI.
        require!(
            target != Pubkey::default(),
            CassieError::CallbackInvocationFailed
        );

        let (_, canonical_bump) = Pubkey::find_program_address(
            &[QUESTION_CONFIG_SEED.as_bytes(), hash.as_ref()],
            &crate::id(),
        );
        let seeds: &[&[u8]] = &[QUESTION_CONFIG_SEED.as_bytes(), hash.as_ref(), &[canonical_bump]];

        let mut infos: Vec<AccountInfo> = Vec::with_capacity(1 + remaining.len());
        infos.push(program_ai.to_account_info());
        infos.extend_from_slice(remaining);

        invoke_signed(&ix, &infos, &[seeds])
            .map_err(|_| error!(CassieError::CallbackInvocationFailed))?;
        Ok(())
    }
}
