use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Question {
    // creator = who creates the question
    pub creator: Pubkey,
    // hash = the is unique to the on-chain id
    pub hash: [u8; 32],
    // metadata_uri = arweave/ipfs
    pub metadata_uri: [u8; 128],
    // category = category of the question
    pub category: u8,

    // bounty
    pub bounty: u64,

    // deadline for answering this question - set by config if explicitly not provided
    pub answer_deadline: i64,
    // deadline for dispute - set by the config.
    pub dispute_deadline: i64,
    // Unix timestamp of question creation time
    pub created_at: i64,

    // for on chain calculation, total yes weight
    pub total_yes_weight: u128,
    // total no weight
    pub total_no_weight: u128,

    // total yes stake for slash
    pub total_yes_stake: u128,
    // total no stake for stake
    pub total_no_stake: u128,

    // did dispute happen ? default false
    pub has_dispute: bool,
    // did escalation happen ? default false
    pub escalated: bool,

    // consumer callback - set when ask question default None
    pub callback_program: Pubkey,
    // callback discriminator
    pub callback_discriminator: [u8; 8],

    // bump of the question
    pub bump: u8,
}

pub enum QuestionState {
    Asked,
    Answering,
    Aggregating,
    Resolved,
    Escalated,
    Council,
    Settled,
}
