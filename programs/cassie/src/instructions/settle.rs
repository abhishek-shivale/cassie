use crate::aggregation::{compute_reward_split, RewardSplit};
use crate::constants::*;
use crate::error::CassieError;
use crate::{DisputeConfig, OracleConfig, Outcome, Question, QuestionSettled, QuestionState};
use anchor_lang::prelude::*;
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
        bump = question.bump,
    )]
    pub question: Box<Account<'info, Question>>,

    #[account(
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, OracleConfig>>,

    #[account(
        seeds = [OUTCOME_SEED.as_bytes(), hash.as_ref()],
        bump = outcome.bump,
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
        bump = dispute.bump,
    )]
    pub dispute: Option<Box<Account<'info, DisputeConfig>>>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Settle<'info> {
    pub fn settle(&mut self, hash: [u8; 32]) -> Result<()> {
        require!(!self.config.freeze, CassieError::ProgramFrozen);
        require!(
            matches!(self.question.state, QuestionState::Resolved),
            CassieError::InvalidState
        );
        require!(
            Clock::get()?.unix_timestamp > self.question.dispute_deadline,
            CassieError::DisputeWindowActive
        );

        let result = self.outcome.result;
        let (correct_count, loser_stake) = if result {
            (self.question.yes_count, self.question.total_no_stake)
        } else {
            (self.question.no_count, self.question.total_yes_stake)
        };

        let split = self.deduct_treasury(loser_stake as u64, hash)?;

        let mut answer_pool = split.total;
        if let Some(dispute) = &mut self.dispute {
            if dispute.claimed_outcome == result {
                let slash_share = ((split.slash_amount as u128) * (DISPUTE_REWARD_BPS as u128)
                    / BPS_DENOMINATOR) as u64;
                dispute.resolved = true;
                dispute.reward = dispute.bond_amount.saturating_add(slash_share);
                answer_pool = answer_pool.saturating_sub(slash_share);
            } else {
                dispute.resolved = false;
                dispute.reward = 0;
            }
        }

        let per_answer_reward = if correct_count == 0 {
            0
        } else {
            answer_pool / (correct_count as u64)
        };
        self.question.per_answer_reward = per_answer_reward;
        self.question.state = QuestionState::Settled;

        emit!(QuestionSettled {
            hash,
            result,
            treasury_cut: split.treasury_cut,
            per_answer_reward,
            slash_amount: split.slash_amount,
        });

        Ok(())
    }

    fn deduct_treasury(&self, loser_stake: u64, hash: [u8; 32]) -> Result<RewardSplit> {
        let split = compute_reward_split(
            self.question.bounty,
            loser_stake,
            self.config.slash_bps as u16,
            self.config.treasury_bps as u16,
        );

        if split.treasury_cut > 0 {
            let bump = [self.question.bump];
            let seeds: &[&[u8]] = &[QUESTION_CONFIG_SEED.as_bytes(), hash.as_ref(), &bump];
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
                split.treasury_cut,
                self.usdc_mint.decimals,
            )?;
        }
        Ok(split)
    }
}
