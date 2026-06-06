use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Answer {
    // answerer = who added the answer to the question
    pub answerer: Pubkey,
    // side = true or false answer to the question
    pub side: bool,
    // stake = same as the default
    pub stake: u64,
    // weight = stake * accuracy * loyalty, snapshot
    pub weight: u128,
    // score reputation of answerer on the time of answering
    pub rep_score_at_answer: u64,
    // loyalty reputation of answerer
    pub rep_days_at_answer: u32,
    // claimed = is user claim the bounty
    pub claimed: bool,
    // submitted at Unix timestamp for creating the answer
    pub submitted_at: i64,
    // bump = bump of the answer
    pub bump: u8,
}
