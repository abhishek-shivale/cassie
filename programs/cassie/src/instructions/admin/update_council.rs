use crate::constants::ADMIN_CONFIG_SEED;
use crate::error::CassieError;
use crate::state::admin::OracleConfig;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdateCouncil<'info> {
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

impl<'info> UpdateCouncil<'info> {
    pub fn update_council(&mut self, old: Pubkey, new: Pubkey) -> Result<()> {
        let cfg = &mut self.config;
        require!(
            !cfg.council.contains(&new),
            CassieError::DuplicateCouncilMember
        );
        let slot = cfg
            .council
            .iter_mut()
            .find(|k| **k == old)
            .ok_or(CassieError::NotCouncilMember)?;

        *slot = new;
        Ok(())
    }
}
