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

// dispute created
#[event]
pub struct Dispute {
    pub hash: [u8; 32],
    pub disputer: Pubkey,
    pub bond_amount: u64,
    pub claimed_outcome: bool,
    pub reason_hash: [u8; 128],
}
