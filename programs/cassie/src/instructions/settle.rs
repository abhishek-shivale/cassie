use crate::constants::*;
use crate::{OracleConfig, Question};
use anchor_lang::prelude::*;
#[derive(Accounts)]
#[instruction(hash: [u8; 32])]
pub struct Settle<'info> {
    #[account(mut)]
    pub cranker: Signer<'info>,

    #[account(
        mut,
        seeds = [QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()],
        bump = question.bump,
    )]
    pub question: Account<'info, Question>,

    #[account(
        seeds = [ADMIN_CONFIG_SEED.as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, OracleConfig>,
}
