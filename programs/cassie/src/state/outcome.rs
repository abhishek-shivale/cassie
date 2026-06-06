use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Outcome {
    // unique on-chain id of question
    pub hash: [u8; 32],

    // result in bool form
    pub result: bool,
    // resolver = who resolve the question
    pub resolver: Resolver,

    // aggregation result = snapshot on time of result
    pub total_yes_weight: u128,
    // for no aggregating
    pub total_no_weight: u128,
    // snapshot of total weight
    pub answer_count: u32,

    // council snapshot if escalated
    pub council_yes: u8,
    // snapshot of no
    pub council_no: u8,

    // settled at the time when this outcome released it might be updated after escalation
    pub settled_at: i64,

    //bump for outcome for pda
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub enum Resolver {
    Optimistic, // question resolve without any dispute and no opposition
    Weighted,   // question resolve as weighted there was the opposition
    Council,    // resolve by council
}
