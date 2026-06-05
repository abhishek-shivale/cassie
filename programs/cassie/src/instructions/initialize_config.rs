use crate::constants::{ADMIN_CONFIG_SEED, USDC_PUBKEY};
use crate::state::admin::OracleConfig;
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
        space= OracleConfig::DISCRIMINATOR.len() + OracleConfig::INIT_SPACE,
        bump,
    )]
    pub config: Account<'info, OracleConfig>,

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
        default_answer_window: i64,
        default_council_window: i64,
        default_dispute_window: i64,
        divergence_bps: u64,
        min_bounty: u64,
        min_dispute_bond: u64,
        slash_bps: u64,
        treasury: Pubkey,
        treasury_bps: u64,
        min_stake: u64,
        council: [Pubkey; 9],
    ) -> Result<()> {
        let quorum = council.len().checked_mul(2).unwrap().checked_div(3).unwrap();
        self.config.set_inner(OracleConfig {
            admin: self.authority.key(),
            mint: self.usdc_mint.key(),
            council_size: council.len() as u8,
            quorum: quorum as u8,
            bump,
            council,
            default_answer_window,
            default_council_window,
            default_dispute_window,
            divergence_bps,
            min_bounty,
            min_dispute_bond,
            slash_bps,
            treasury,
            treasury_bps,
            min_stake,
        });
        Ok(())
    }
}
