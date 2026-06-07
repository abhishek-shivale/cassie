use crate::constants::{ADMIN_CONFIG_SEED, USDC_PUBKEY};
use crate::error::CassieError;
use crate::state::admin::OracleConfig;
use crate::{BPS_DENOMINATOR, MAX_COUNCIL_MEMBER, MIN_DISPUTE_BOND, MIN_STAKE};
use anchor_lang::prelude::*;
use anchor_spl::{token::Mint, token_interface::TokenInterface};

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
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
        slash_bps: u64,
        treasury: Pubkey,
        treasury_bps: u64,
        council: [Pubkey; MAX_COUNCIL_MEMBER],
        council_size: u8,
    ) -> Result<()> {
        // bps can't exceed 100%
        require_gte!(BPS_DENOMINATOR as u64, divergence_bps, CassieError::MaxBpsReached);
        require_gte!(BPS_DENOMINATOR as u64, treasury_bps, CassieError::MaxBpsReached);
        require_gte!(BPS_DENOMINATOR as u64, slash_bps, CassieError::MaxBpsReached);

        // windows, all in seconds
        require_gte!(default_dispute_window, 7200, CassieError::InvalidWindow);
        require_gte!(default_answer_window, 3600, CassieError::InvalidWindow);
        require_gte!(default_council_window, 86400, CassieError::InvalidWindow);

        // council size in 1..=MAX
        require!(council_size > 0, CassieError::CouncilMemberShouldNotBeZero);
        require!(
            council_size as usize <= MAX_COUNCIL_MEMBER,
            CassieError::MaxCouncilSizeReached
        );

        // bounty floor
        require!(min_bounty >= 10, CassieError::BountySizeCanNotBeLower);

        // active council slots must be non-zero and unique.
        let active = council_size as usize;
        for i in 0..active {
            require_keys_neq!(
                council[i],
                Pubkey::default(),
                CassieError::CouncilMemberShouldNotBeZero
            );
            for j in (i + 1)..active {
                require_keys_neq!(council[i], council[j], CassieError::DuplicateCouncilMember);
            }
        }

        let quorum = council_size.checked_mul(2).unwrap().checked_div(3).unwrap();
        self.config.set_inner(OracleConfig {
            admin: self.admin.key(),
            mint: self.usdc_mint.key(),
            council_size,
            quorum,
            bump,
            council,
            default_answer_window,
            default_council_window,
            default_dispute_window,
            divergence_bps,
            min_bounty,
            min_dispute_bond: MIN_DISPUTE_BOND,
            slash_bps,
            treasury,
            treasury_bps,
            min_stake: MIN_STAKE,
            freeze: false,
        });
        Ok(())
    }
}
