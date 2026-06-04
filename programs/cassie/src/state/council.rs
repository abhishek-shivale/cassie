use anchor_lang::prelude::*;


#[account]
#[derive(InitSpace)]
pub struct CouncilTotal {
    // total yes count of council member
    pub yes_count: u8,
    // total no count of council member
    pub no_count: u8,
    // council voting open at
    pub opened_at: i64,
    // finalized at
    pub finalized_at: Option<bool>,
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
    // when member voted
    pub voted_at: i64,
    // bump
    pub bump: u8,
}