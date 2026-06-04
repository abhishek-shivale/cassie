use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Dispute {
    // person who dispute the question on answer
    pub disputer: Pubkey,
    // bond amount for dispute same as the default
    pub bond_amount: u64,
    // what disputer says is correct
    pub claimed_outcome: bool,
    // sha256 of off-chain reasoning
    pub reason_hash: [u8; 32],
    // disputed at like Unix
    pub disputed_at: i64,
    // dispute expire at
    pub expires_at: i64,
    // resolved as bool decided by council voting
    pub resolved: bool,
    // dispute reward
    pub reward: u64,

    // bump
    pub bump: u8,
}