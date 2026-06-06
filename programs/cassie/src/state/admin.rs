use crate::MAX_COUNCIL_MEMBER;
use anchor_lang::prelude::clock::UnixTimestamp;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct OracleConfig {
    // admin authority
    pub admin: Pubkey,
    // SPL but usdc for now
    pub mint: Pubkey,
    // Treasury
    pub treasury: Pubkey,

    // Default seconds to answer the question e.g. store in second - 2h = 60*60*2
    pub default_answer_window: i64,
    // Default seconds to dispute the answer e.g. store in second - 2h = 60*60*2
    pub default_dispute_window: i64,
    // pub default council window e.g. store in second - 2h = 60*60*2
    pub default_council_window: i64,

    // minimum stack to answer the question
    pub min_stake: u64,
    // minimum bounty question must put
    pub min_bounty: u64,
    // minimum dispute bond
    pub min_dispute_bond: u64,
    // divergence bps = the percent if no comes it will auto-escalate to council. example - 3500  = 35 percent
    pub divergence_bps: u64,
    // slash bps = slash percentage e.g. 5000 = 50%
    pub slash_bps: u64,
    // treasury_bps = protocol fee e.g. 10 = 0.1% of reward pool
    pub treasury_bps: u64,

    // council
    pub council: [Pubkey; MAX_COUNCIL_MEMBER],
    // max council size.
    pub council_size: u8,
    // quorum = minimum vote require by council member to settle the question
    pub quorum: u8,

    pub bump: u8, // bump of the pda so we don't have to derive everytime
    
    // freeze this will freeze the program no action will be done from this moment
    pub freeze: bool
}

impl OracleConfig {
    pub fn get_question_deadline(&self, created_at: UnixTimestamp) -> i64 {
        created_at + self.default_answer_window
    }

    pub fn get_dispute_deadline(&self, disputed_at: UnixTimestamp) -> i64 {
        disputed_at + self.default_dispute_window
    }
    
    pub fn get_council_deadline(&self, council_at: UnixTimestamp) -> i64 {
        council_at + self.default_council_window
    }

}
