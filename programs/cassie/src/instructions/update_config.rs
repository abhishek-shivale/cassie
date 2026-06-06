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
        default_dispute_window: Option<i64>,
        default_council_window: Option<i64>,
        default_answer_window: Option<i64>,
        freeze: Option<bool>
    ) -> Result<()> {
       let cfg = &mut self.config;
        if let Some(v) = default_dispute_window { cfg.default_dispute_window = v; }
        if let Some(v) = default_council_window { cfg.default_council_window = v; }
        if let Some(v) = default_answer_window { cfg.default_answer_window = v; }
        if let Some(f) = freeze { cfg.freeze = f; }
        
        Ok(())
    }
}
