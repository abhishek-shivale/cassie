use crate::constants::{ADMIN_CONFIG_SEED, USDC_PUBKEY};
use crate::state::admin::AdminConfig;
use anchor_lang::prelude::*;
use anchor_spl::{token::Mint, token_interface::TokenInterface};

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init_if_needed,
        payer = authority,
        seeds = [ADMIN_CONFIG_SEED.as_ref()],
        space= 8 + AdminConfig::INIT_SPACE,
        bump,
    )]
    pub config: Account<'info, AdminConfig>,

    #[account(
        address = USDC_PUBKEY
    )]
    pub usdc_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> InitializeConfig<'info> {
    pub fn init_config(
        &mut self,
        bump: u8,
        default_answer_period: u16,
        default_council_resolve_period: u16,
        default_dispute_period: u16,
        max_council_members: u8,
        slash_rate: u16,
    ) -> Result<()> {
        self.config.set_inner(AdminConfig {
            authority: self.authority.key(),
            usdc_mint: self.usdc_mint.key(),
            bump,
            default_answer_period,
            default_council_resolve_period,
            slash_rate,
            max_council_members,
            default_dispute_period,
        });
        Ok(())
    }
}
