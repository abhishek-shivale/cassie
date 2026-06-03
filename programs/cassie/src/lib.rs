pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("5sU2QBvow11aj1m6z6DdqpsaqVuh84e8RWpQD5njdgYM");

#[program]
pub mod cassie {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        default_answer_period: u16,
        default_council_resolve_period: u16,
        default_dispute_period: u16,
        slash_rate: u16,
        council: [Pubkey; 3],
    ) -> Result<()> {
        ctx.accounts.init_config(
            ctx.bumps.config,
            default_answer_period,
            default_council_resolve_period,
            default_dispute_period,
            slash_rate,
            council,
        )
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>,
        default_answer_period: u16,
        default_council_resolve_period: u16,
        default_dispute_period: u16,
        slash_rate: u16,
    ) -> Result<()> {
        ctx.accounts.update_config(
            default_answer_period,
            default_council_resolve_period,
            default_dispute_period,
            slash_rate,
        )
    }

    pub fn update_council(ctx: Context<UpdateCouncil>, old: Pubkey, new: Pubkey) -> Result<()> {
        ctx.accounts.update_council(old, new)
    }

    pub fn ask(
        ctx: Context<Ask>,
        nonce: u64,
        question: String,
        bounty: u64,
        category: String,
        description: String,
        rules: String,
    ) -> Result<()> {
        ctx.accounts.ask_question(
            question,
            nonce,
            ctx.bumps.question,
            bounty,
            category,
            description,
            rules,
        )
    }
    
    // pub fn answer(ctx: Context<Answer>) -> Result<()> {
    //     
    // }
}
