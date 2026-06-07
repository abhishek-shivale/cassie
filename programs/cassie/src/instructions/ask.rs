use crate::constants::{ADMIN_CONFIG_SEED, QUESTION_CONFIG_SEED, USDC_PUBKEY};
use crate::state::admin::OracleConfig;
use crate::state::question::Question;
use crate::error::CassieError;
use crate::{CreateQuestion, QuestionState};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
#[instruction(hash: [u8; 32])]
pub struct Ask<'info> {
    #[account(mut)]
    pub questioner: Signer<'info>,

    #[account(
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, OracleConfig>>,

    #[account(
        init,
        payer = questioner,
        space = Question::DISCRIMINATOR.len() + Question::INIT_SPACE,
        seeds = [QUESTION_CONFIG_SEED.as_bytes(), hash.as_ref()],
        bump
    )]
    pub question: Box<Account<'info, Question>>,

    #[account(
        address = USDC_PUBKEY
    )]
    pub usdc_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = questioner,
    )]
    pub questioner_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer = questioner,
        associated_token::mint = usdc_mint,
        associated_token::authority = question,
    )]
    pub bounty_ata: Box<InterfaceAccount<'info, TokenAccount>>, // reward pool

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Ask<'info> {
    pub fn deposit_bounty(&mut self, bounty: u64) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                self.token_program.key(),
                TransferChecked {
                    from: self.questioner_ata.to_account_info(),
                    to: self.bounty_ata.to_account_info(),
                    mint: self.usdc_mint.to_account_info(),
                    authority: self.questioner.to_account_info(),
                },
            ),
            bounty,
            self.usdc_mint.decimals,
        )?;
        Ok(())
    }

    pub fn ask_question(
        &mut self,
        hash: [u8; 32],
        bump: u8,
        bounty: u64,
        category: u8,
        metadata_uri: [u8; 128],
        callback_program: Pubkey,
        callback_discriminator: [u8; 8],
    ) -> Result<()> {
        require!(!self.config.freeze, CassieError::ProgramFrozen);
        require_gte!(bounty, self.config.min_bounty, CassieError::InsufficientBounty);

        let created_at = Clock::get()?.unix_timestamp;
        let answer_deadline = self.config.get_question_deadline(created_at);
        self.question.set_inner(Question {
            creator: self.questioner.key(),
            total_yes_weight: 0,
            total_no_weight: 0,
            total_yes_stake: 0,
            total_no_stake: 0,
            has_dispute: false,
            escalated: false,
            dispute_deadline: 0,
            bump,
            bounty,
            answer_deadline,
            category,
            created_at,
            hash,
            yes_count: 0,
            no_count: 0,
            per_answer_reward: 0,
            metadata_uri,
            callback_program,
            callback_discriminator,
            state: QuestionState::Asked,
        });

        self.deposit_bounty(bounty)?;

        emit!(CreateQuestion {
            creator: self.questioner.key(),
            hash,
            metadata_uri,
            bounty,
        });

        Ok(())
    }
}
