mod aggregation;
pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
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
        slash_bps: u64,
        treasury: Pubkey,
        treasury_bps: u64,
        council_bps: u64,
        council: [Pubkey; 9],
        council_size: u8,
    ) -> Result<()> {
        ctx.accounts.init_config(
            ctx.bumps.config,
            default_answer_window,
            default_council_window,
            default_dispute_window,
            divergence_bps,
            min_bounty,
            slash_bps,
            treasury,
            treasury_bps,
            council_bps,
            council,
            council_size,
        )
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>,
        default_dispute_window: Option<i64>,
        default_council_window: Option<i64>,
        default_answer_window: Option<i64>,
        freeze: Option<bool>,
    ) -> Result<()> {
        ctx.accounts.update_config(
            default_dispute_window,
            default_council_window,
            default_answer_window,
            freeze,
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

    #[allow(unused_variables)]
    pub fn propose(ctx: Context<Propose>, hash: [u8; 32], stake: u64, side: bool) -> Result<()> {
        ctx.accounts.propose(stake, side, &ctx.bumps)
    }

    #[allow(unused_variables)]
    pub fn close_proposers(ctx: Context<CloseProposer>, hash: [u8; 32]) -> Result<()> {
        ctx.accounts.close(&ctx.bumps)
    }

    #[allow(unused_variables)]
    pub fn dispute(
        ctx: Context<Dispute>,
        hash: [u8; 32],
        bond: u64,
        claimed_outcome: bool,
        reason_hash: [u8; 128],
    ) -> Result<()> {
        ctx.accounts
            .dispute(bond, claimed_outcome, reason_hash, &ctx.bumps)
    }

    #[allow(unused_variables)]
    pub fn council_vote(ctx: Context<Vote>, hash: [u8; 32], vote: bool) -> Result<()> {
        ctx.accounts.vote(vote, &ctx.bumps)
    }

    #[allow(unused_variables)]
    pub fn finalize_council(ctx: Context<Finalize>, hash: [u8; 32]) -> Result<()> {
        ctx.accounts.finalize()
    }

    pub fn settle_question<'info>(
        ctx: Context<'info, Settle<'info>>,
        hash: [u8; 32],
    ) -> Result<()> {
        ctx.accounts.settle(hash)?;
        ctx.accounts.fire_callback(ctx.remaining_accounts)
    }

    pub fn claim_reward(ctx: Context<ClaimReward>, hash: [u8; 32]) -> Result<()> {
        ctx.accounts.claim(hash)
    }

    pub fn close_question(ctx: Context<CloseQuestion>, hash: [u8; 32]) -> Result<()> {
        ctx.accounts.close(hash)
    }
}
