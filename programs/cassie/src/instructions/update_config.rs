use crate::constants::ADMIN_CONFIG_SEED;
use crate::state::admin::OracleConfig;
use crate::error::CassieError;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [ADMIN_CONFIG_SEED.as_ref()],
        bump = config.bump,
        has_one = authority @ CassieError::UnauthorizedAdmin,
    )]
    pub config: Account<'info, OracleConfig>,
}

impl<'info> UpdateConfig<'info> {
    pub fn update_config(
        &mut self,
        default_answer_period: u16,
        default_council_resolve_period: u16,
        default_dispute_period: u16,
        slash_rate: u16,
    ) -> Result<()> {
       let cfg = &mut self.config;
        cfg.default_answer_period = default_answer_period;
        cfg.default_council_resolve_period = default_council_resolve_period;
        cfg.default_dispute_period = default_dispute_period;
        cfg.slash_rate = slash_rate;
        Ok(())
    }
}
