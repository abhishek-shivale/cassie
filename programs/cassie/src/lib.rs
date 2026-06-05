pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use events::*;
pub use instructions::*;
pub use state::*;

declare_id!("5sU2QBvow11aj1m6z6DdqpsaqVuh84e8RWpQD5njdgYM");

#[program]
pub mod cassie {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
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
        ctx.accounts.init_config(
            ctx.bumps.config,
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
            council,
        )
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>,
        default_dispute_window: i64,
        default_council_window: i64,
        default_dispute_period: i64,
    ) -> Result<()> {
        ctx.accounts.update_config(
            default_dispute_window,
            default_council_window,
            default_dispute_period,
        )
    }

    pub fn update_council(ctx: Context<UpdateCouncil>, old: Pubkey, new: Pubkey) -> Result<()> {
        ctx.accounts.update_council(old, new)
    }

    pub fn ask(
        ctx: Context<Ask>,
        hash: [u8; 32],
        bounty: u64,
        category: u8,
        metadata_uri: [u8; 128],
        callback_program: Pubkey,
        callback_discriminator: [u8; 8],
    ) -> Result<()> {
        ctx.accounts.ask_question(
            hash,
            ctx.bumps.question,
            bounty,
            category,
            metadata_uri,
            callback_program,
            callback_discriminator,
        )
    }

    // pub fn answer(ctx: Context<Answer>) -> Result<()> {
    //
    // }
}
