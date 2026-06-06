use crate::Resolver;
use anchor_lang::prelude::*;

//  ----------------------------------------------------
//  |                 Events                           |
//  ----------------------------------------------------
// it will appear in rpc so we can index data off chain

// creator created a question
#[event]
pub struct CreateQuestion {
    pub creator: Pubkey,
    pub hash: [u8; 32],
    pub metadata_uri: [u8; 128],
    pub bounty: u64,
}

// proposed answer - answer proposed by answerer
#[event]
pub struct ProposedAnswer {
    pub hash: [u8; 32],
    pub answerer: Pubkey,
    pub side: bool,
    pub stake: u64,
    pub weight: u128,
}

// answering phase closed - question resolved or escalated
#[event]
pub struct ProposersClosed {
    pub hash: [u8; 32],
    // winning side (tentative if escalated)
    pub result: bool,
    // how it was resolved
    pub resolver: Resolver,
    // true = sent to council, false = resolved
    pub escalated: bool,
    pub total_yes_weight: u128,
    pub total_no_weight: u128,
    pub answer_count: u32,
    pub settled_at: i64,
}

// dispute created
#[event]
pub struct DisputeCreated {
    pub hash: [u8; 32],
    pub disputer: Pubkey,
    pub bond_amount: u64,
    pub claimed_outcome: bool,
    pub reason_hash: [u8; 128],
}

// a council member cast a vote
#[event]
pub struct CouncilVoted {
    pub hash: [u8; 32],
    pub member: Pubkey,
    pub vote: bool,
    pub yes_count: u8,
    pub no_count: u8,
}

// council reached quorum, verdict set
#[event]
pub struct CouncilFinalized {
    pub hash: [u8; 32],
    pub result: bool,
    pub council_yes: u8,
    pub council_no: u8,
}

// question settled - pool split + payouts ready to claim
#[event]
pub struct QuestionSettled {
    pub hash: [u8; 32],
    pub result: bool,
    pub treasury_cut: u64,
    pub per_answer_reward: u64,
    pub slash_amount: u64,
}
