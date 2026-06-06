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
        council: [Pubkey; 9],
        council_size: u8,
    ) -> Result<()> {
        // check bps because we don't want admin to slash more than 100%
        require_gte!(
            BPS_DENOMINATOR as u64,
            divergence_bps,
            CassieError::MaxBpsReached
        );
        require_gte!(
            BPS_DENOMINATOR as u64,
            treasury_bps,
            CassieError::MaxBpsReached
        );
        require_gte!(
            BPS_DENOMINATOR as u64,
            slash_bps,
            CassieError::MaxBpsReached
        );

        // its window check all in seconds
        require_gte!(default_dispute_window, 7200, CassieError::InvalidWindow);
        require_gte!(default_answer_window, 3600, CassieError::InvalidWindow);
        require_gte!(default_council_window, 86400, CassieError::InvalidWindow);

        // council
        require!(council_size <= 9, CassieError::MaxCouncilSizeReached);
        require!(council_size > 0, CassieError::CouncilMemberShouldNotBeZero);

        // bounty
        require!(min_bounty >= 10, CassieError::BountySizeCanNotBeLower);

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
        // this we compare and if new or old are zero pubkey it will throw error
        require_keys_neq!(
            new,
            Pubkey::default(),
            CassieError::CouncilMemberShouldNotBeZero
        );
        require_keys_neq!(
            old,
            Pubkey::default(),
            CassieError::CouncilMemberShouldNotBeZero
        );

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
        require_gte!(
            bounty,
            ctx.accounts.config.min_bounty,
            CassieError::InsufficientBounty
        );
        require!(!ctx.accounts.config.freeze, CassieError::ProgramFrozen);
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

    pub fn propose(ctx: Context<Propose>, _hash: [u8; 32], stake: u64, side: bool) -> Result<()> {
        require!(!ctx.accounts.config.freeze, CassieError::ProgramFrozen);
        require!(
            Clock::get()?.unix_timestamp < ctx.accounts.question.answer_deadline,
            CassieError::AnswerWindowClosed
        );
        require_eq!(stake, MIN_STAKE, CassieError::InsufficientStake);
        require!(
            matches!(
                ctx.accounts.question.state,
                QuestionState::Asked | QuestionState::Answering
            ),
            CassieError::InvalidState
        );

        ctx.accounts.propose(stake, side, &ctx.bumps)
    }

    pub fn close_proposers(ctx: Context<CloseProposer>, _hash: [u8; 32]) -> Result<()> {
        ctx.accounts.close(&ctx.bumps)
    }

    pub fn dispute(
        ctx: Context<Dispute>,
        _hash: [u8; 32],
        bond: u64,
        claimed_outcome: bool,
        reason_hash: [u8; 128],
    ) -> Result<()> {
        ctx.accounts.dispute(bond, claimed_outcome, reason_hash, &ctx.bumps)
    }

    pub fn council_vote(ctx: Context<Vote>, _hash: [u8; 32], vote: bool) -> Result<()> {
        ctx.accounts.vote(vote, &ctx.bumps)
    }

    pub fn finalize_council(ctx: Context<Finalize>, _hash: [u8; 32]) -> Result<()> {
        ctx.accounts.finalize()
    }

    pub fn settle_question(ctx: Context<Settle>, hash: [u8; 32]) -> Result<()> {
        ctx.accounts.settle(hash)
    }

    pub fn claim_reward(ctx: Context<ClaimReward>, hash: [u8; 32]) -> Result<()> {
        ctx.accounts.claim(hash)
    }
}
