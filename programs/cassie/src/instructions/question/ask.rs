use crate::constants::{ADMIN_CONFIG_SEED, QUESTION_CONFIG_SEED, USDC_PUBKEY};
use crate::state::admin::OracleConfig;
use crate::state::question::QuestionConfig;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct Ask<'info> {
    #[account(mut)]
    pub questioner: Signer<'info>,

    #[account(
        seeds = [ADMIN_CONFIG_SEED.as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, OracleConfig>,

    #[account(
        init,
        payer = questioner,
        space = QuestionConfig::DISCRIMINATOR.len() + QuestionConfig::INIT_SPACE,
        seeds = [QUESTION_CONFIG_SEED.as_ref(), questioner.key().as_ref(), nonce.to_le_bytes().as_ref()],
        bump
    )]
    pub question: Account<'info, QuestionConfig>,

    #[account(
        address = USDC_PUBKEY
    )]
    pub usdc_mint: InterfaceAccount<'info, Mint>,

    #[account(
        associated_token::mint = usdc_mint,
        associated_token::authority = questioner,
    )]
    pub questioner_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        associated_token::mint = usdc_mint,
        associated_token::authority = config,
    )]
    pub bounty_ata: InterfaceAccount<'info, TokenAccount>, // reward pool with Escrow

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Ask<'info> {
    pub fn deposit_bounty(&mut self, bounty: u64) -> Result<()> {
        // require_gt!(self.config.) !todo
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
        question: String,
        nonce: u64,
        bump: u8,
        bounty: u64,
        category: String,
        description: String,
        rules: String,
    ) -> Result<()> {
        let created_at = Clock::get()?.unix_timestamp;
        self.question.set_inner(QuestionConfig {
            questioner: self.questioner.key(),
            question,
            nonce,
            bump,
            bounty,
            category,
            description,
            rules,
            created_at,
        });
        // emit here !todo
        Ok(())
    }

    pub fn handler(
        &mut self,
        question: String,
        nonce: u64,
        bump: u8,
        bounty: u64,
        category: String,
        description: String,
        rules: String,
    ) -> Result<()> {
        self.ask_question(question, nonce, bump, bounty, category, description, rules)?;

        self.deposit_bounty(bounty)?;

        Ok(())
    }
}
