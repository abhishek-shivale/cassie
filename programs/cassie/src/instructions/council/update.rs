use crate::constants::ADMIN_CONFIG_SEED;
use crate::error::CassieError;
use crate::state::admin::OracleConfig;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdateCouncil<'info> {
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

impl<'info> UpdateCouncil<'info> {
    pub fn update_council(&mut self, old: Pubkey, new: Pubkey) -> Result<()> {
        // neither slot may be the zero pubkey
        require_keys_neq!(new, Pubkey::default(), CassieError::CouncilMemberShouldNotBeZero);
        require_keys_neq!(old, Pubkey::default(), CassieError::CouncilMemberShouldNotBeZero);

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
