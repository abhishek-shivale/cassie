use crate::constants::{ADMIN_CONFIG_SEED, QUESTION_CONFIG_SEED, USDC_PUBKEY};
use crate::{OracleConfig, Question};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
#[instruction(asker: Pubkey, nonce: u64)]
pub struct Propose<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(
        mut,
         seeds = [QUESTION_CONFIG_SEED.as_ref(), asker.key().as_ref(), nonce.to_le_bytes().as_ref()],
        bump = question.bump
    )]
    pub question: Account<'info, Question>,

    #[account(
        seeds = [ADMIN_CONFIG_SEED.as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, OracleConfig>,

    #[account(
        address = USDC_PUBKEY,
    )]
    pub usdc_mint: InterfaceAccount<'info, Mint>,

    #[account(
        associated_token::mint = usdc_mint,
        associated_token::authority = proposer,
    )]
    pub proposer_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        associated_token::mint = usdc_mint,
        associated_token::authority = config,
    )]
    pub bond_ata: InterfaceAccount<'info, TokenAccount>,

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


    pub fn propose(&mut self) -> Result<()> {
        let ans = &mut self.question;
        // ans.

        Ok(())
    }
}
