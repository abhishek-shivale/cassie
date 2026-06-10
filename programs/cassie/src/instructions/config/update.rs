use crate::constants::ADMIN_CONFIG_SEED;
use crate::error::CassieError;
use crate::state::admin::OracleConfig;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump,
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
        freeze: Option<bool>,
    ) -> Result<()> {
        let cfg = &mut self.config;
        // its window check all in seconds
        if let Some(v) = default_dispute_window {
            require_gte!(v, 7200, CassieError::InvalidWindow);
            cfg.default_dispute_window = v;
        }
        if let Some(v) = default_council_window {
            require_gte!(v, 86400, CassieError::InvalidWindow);
            cfg.default_council_window = v;
        }
        if let Some(v) = default_answer_window {
            require_gte!(v, 3600, CassieError::InvalidWindow);
            cfg.default_answer_window = v;
        }
        if let Some(f) = freeze {
            cfg.freeze = f;
        }

        Ok(())
    }
}
