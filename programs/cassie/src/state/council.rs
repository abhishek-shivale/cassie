use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct CouncilTotal {
    // total yes count of council member
    pub yes_count: u8,
    // total no count of council member
    pub no_count: u8,
    // council voting open at
    pub opened_at: i64,
    // yes stake
    pub total_yes_stake: u128,
    // no stake
    pub total_no_stake: u128,
    // finalized at
    pub finalized: Option<bool>,
    // dispute period
    pub dispute_time: i64,
    // bump of config
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct CouncilVote {
    // council member
    pub member: Pubkey,
    // answer for the question
    pub vote: bool,
    // stake added
    pub stake: u64,
    // when member voted
    pub voted_at: i64,
    // did the member claim their reputation update
    pub claimed: bool,
    // bump
    pub bump: u8,
}
