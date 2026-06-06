use crate::aggregation::compute_weight;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Reputation {
    // reputation per wallet as proposer - council member + proposer
    pub voter: Pubkey,

    // score = max score of weight 0-500 it will be 6.0x multiplier
    pub score: u64,
    // number of the answered question
    pub answered: u32,
    // correct answered the question
    pub correct: u32,

    // pub active days on protocol, 0 - 365
    pub active_days: u32,
    // last answer day - Unix day of last answer
    pub last_answer_day: i64,

    // dispute created filed by the user
    pub disputes_filed: u32,
    // disputes won by the user
    pub disputes_won: u32,
    // disputes lost by the user
    pub disputes_lost: u32,

    // slash history how many times user got the slashed
    pub times_slashed: u32,
    // total time user got the slashed
    pub total_slashed: u64,

    // is council to check is this belong to the member or not,
    pub is_council: bool,
    // council votes how many vote he did give as member
    pub council_votes: u32,
    // council corrected votes
    pub council_correct: u32,

    // last updated reputation
    pub last_updated: i64,
    // bump for the config
    pub bump: u8,
}

impl Reputation {
    pub fn calculate_weight(&self, stake: u64) -> u128 {
        compute_weight(stake, self.score, self.active_days)
    }
}
