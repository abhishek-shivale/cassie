use crate::constants::*;
use crate::error::CassieError;
use crate::{Answer, OracleConfig, Question, QuestionState, Reputation};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
#[instruction(hash: [u8; 32])]
pub struct Propose<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(
        mut,
         seeds = [QUESTION_CONFIG_SEED.as_bytes(), hash.as_ref()],
        bump
    )]
    pub question: Box<Account<'info, Question>>,

    #[account(
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump,
    )]
    pub config: Box<Account<'info, OracleConfig>>,

    #[account(
        address = USDC_PUBKEY,
    )]
    pub usdc_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = proposer,
    )]
    pub proposer_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = question,
    )]
    pub bond_ata: Box<InterfaceAccount<'info, TokenAccount>>, // reward pool

    #[account(
        init_if_needed,
        payer = proposer,
        space = Reputation::DISCRIMINATOR.len() + Reputation::INIT_SPACE,
        seeds = [REPUTATION_SEED.as_bytes(), proposer.key().as_ref()],
        bump,
        constraint = reputation.voter == Pubkey::default() || reputation.voter == proposer.key() @ CassieError::UnauthorizedAdmin,
    )]
    pub reputation: Box<Account<'info, Reputation>>,

    #[account(
        init,
        payer = proposer,
        space = Answer::DISCRIMINATOR.len() + Answer::INIT_SPACE,
        seeds = [ANSWER_SEED.as_bytes(), hash.as_ref(), proposer.key().as_ref()],
        bump
    )]
    pub answer: Box<Account<'info, Answer>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Propose<'info> {
    pub fn add_bond(&mut self, bond: u64) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                self.token_program.key(),
                TransferChecked {
                    from: self.proposer_ata.to_account_info(),
                    to: self.bond_ata.to_account_info(),
                    mint: self.usdc_mint.to_account_info(),
                    authority: self.proposer.to_account_info(),
                },
            ),
            bond,
            self.usdc_mint.decimals,
        )?;
        Ok(())
    }

    pub fn propose(&mut self, stake: u64, side: bool, bump: &ProposeBumps) -> Result<()> {
        require!(!self.config.freeze, CassieError::ProgramFrozen);
        require!(
            Clock::get()?.unix_timestamp < self.question.answer_deadline,
            CassieError::AnswerWindowClosed
        );
        require_eq!(stake, MIN_STAKE, CassieError::InsufficientStake);
        require!(
            matches!(
                self.question.state,
                QuestionState::Asked | QuestionState::Answering
            ),
            CassieError::InvalidState
        );

        if self.question.state == QuestionState::Asked {
            self.question.state = QuestionState::Answering; // first answer
        }
        // check if voter is new init or old
        if self.reputation.voter == Pubkey::default() {
            self.reputation.voter = self.proposer.key();
            self.reputation.bump = bump.reputation;
        }
        // update question field
        let question = &mut self.question;
        let weight = self.reputation.calculate_weight(stake);
        if side {
            question.total_yes_stake = question.total_yes_stake.checked_add(stake as u128).unwrap();
            question.total_yes_weight = question.total_yes_weight.checked_add(weight).unwrap();
            question.yes_count = question.yes_count.checked_add(1).unwrap();
        } else {
            question.total_no_stake = question.total_no_stake.checked_add(stake as u128).unwrap();
            question.total_no_weight = question.total_no_weight.checked_add(weight).unwrap();
            question.no_count = question.no_count.checked_add(1).unwrap();
        }

        self.answer.set_inner(Answer {
            answerer: self.proposer.key(),
            side,
            stake,
            bump: bump.answer,
            weight,
            claimed: false,
            rep_score_at_answer: self.reputation.score,
            rep_days_at_answer: self.reputation.active_days,
            submitted_at: Clock::get()?.unix_timestamp,
        });

        self.add_bond(stake)?;

        Ok(())
    }
}
