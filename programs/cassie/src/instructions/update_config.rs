use crate::constants::ADMIN_CONFIG_SEED;
use crate::state::admin::OracleConfig;
use crate::error::CassieError;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [ADMIN_CONFIG_SEED.as_ref()],
        bump = config.bump,
        has_one = admin @ CassieError::UnauthorizedAdmin,
    )]
    pub config: Account<'info, OracleConfig>,
}

impl<'info> UpdateConfig<'info> {
    pub fn update_config(
        &mut self,
        default_dispute_window: i64,
        default_council_window: i64,
        default_dispute_period: i64,
    ) -> Result<()> {
       let cfg = &mut self.config;
        cfg.default_dispute_window = default_dispute_window;
        cfg.default_council_window = default_council_window;
        cfg.default_answer_window = default_dispute_period;
        Ok(())
    }
}
