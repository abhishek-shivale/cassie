use crate::constants::*;
use crate::{
    CassieError, DisputeConfig, DisputeCreated, OracleConfig, Outcome, Question, QuestionState,
    Reputation,
};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
#[instruction(hash: [u8; 32])]
pub struct Dispute<'info> {
    #[account(mut)]
    pub disputer: Signer<'info>,

    #[account(
        mut,
        seeds = [QUESTION_CONFIG_SEED.as_bytes(), hash.as_ref()],
        bump = question.bump,
    )]
    pub question: Box<Account<'info, Question>>,

    #[account(
        address = USDC_PUBKEY
    )]
    pub usdc_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = disputer,
    )]
    pub disputer_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = question,
    )]
    pub bond_ata: Box<InterfaceAccount<'info, TokenAccount>>, // reward pool

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
        init,
        payer = disputer,
        space = DisputeConfig::DISCRIMINATOR.len() + DisputeConfig::INIT_SPACE,
        seeds = [DISPUTE_SEED.as_bytes(), hash.as_ref()],
        bump
    )]
    pub disputer_config: Box<Account<'info, DisputeConfig>>,

    #[account(
        init_if_needed,
        payer = disputer,
        space = Reputation::DISCRIMINATOR.len() + Reputation::INIT_SPACE,
        seeds = [REPUTATION_SEED.as_bytes(), disputer.key().as_ref()],
        bump
    )]
    pub reputation: Box<Account<'info, Reputation>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Dispute<'info> {
    pub fn dispute(
        &mut self,
        bond: u64,
        claimed_outcome: bool,
        reason_hash: [u8; 128],
        bumps: &DisputeBumps,
    ) -> Result<()> {
        require!(!self.config.freeze, CassieError::ProgramFrozen);
        require_eq!(bond, MIN_DISPUTE_BOND, CassieError::InsufficientStake);
        let now = Clock::get()?.unix_timestamp;
        require!(
            now >= self.question.answer_deadline,
            CassieError::AnswerWindowActive
        );
        require!(
            matches!(self.question.state, QuestionState::Resolved),
            CassieError::InvalidState
        );
        require!(
            now <= self.question.dispute_deadline,
            CassieError::DisputeWindowClosed
        );
        require!(
            claimed_outcome != self.outcome.result,
            CassieError::InvalidDisputeOutcome
        );

        let question = &mut self.question;
        question.state = QuestionState::Escalated;
        question.escalated = true;

        self.disputer_config.set_inner(DisputeConfig {
            disputer: self.disputer.key(),
            disputed_at: now,
            bond_amount: bond,
            resolved: false,
            reward: 0,
            claimed: false,
            claimed_outcome,
            reason_hash,
            bump: bumps.disputer_config,
        });

        if self.reputation.voter == Pubkey::default() {
            self.reputation.voter = self.disputer.key();
            self.reputation.bump = bumps.reputation;
        }

        emit!(DisputeCreated {
            hash: self.question.hash,
            disputer: self.disputer.key(),
            bond_amount: bond,
            claimed_outcome,
            reason_hash,
        });

        self.add_stake(bond)
    }

    fn add_stake(&mut self, stake: u64) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                self.token_program.key(),
                TransferChecked {
                    authority: self.disputer.to_account_info(),
                    from: self.disputer_ata.to_account_info(),
                    to: self.bond_ata.to_account_info(),
                    mint: self.usdc_mint.to_account_info(),
                },
            ),
            stake,
            self.usdc_mint.decimals,
        )?;

        Ok(())
    }
}
