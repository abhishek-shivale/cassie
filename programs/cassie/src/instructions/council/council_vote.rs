// use crate::constants::ADMIN_CONFIG_SEED;
// use crate::state::admin::OracleConfig;
// use crate::error::CassieError;
// use anchor_lang::prelude::*;
//
//
// #[derive(Accounts)]
// pub struct CouncilVote<'info> {
//     #[account(mut)]
//     pub member: Signer<'info>,
//
//     #[account(
//         mut,
//         seeds = [ADMIN_CONFIG_SEED.as_ref()],
//         bump = config.bump,
//     )]
//     pub config: Account<'info, OracleConfig>,
// }
