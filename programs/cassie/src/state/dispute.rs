use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct DisputeConfig {
    // person who dispute the question on answer
    pub disputer: Pubkey,
    // bond amount for dispute same as the default
    pub bond_amount: u64,
    // what disputer says is correct
    pub claimed_outcome: bool,
    // sha256 of off-chain reasoning arweave/ipfs
    pub reason_hash: [u8; 128],
    // disputed at like Unix
    pub disputed_at: i64,
    // resolved as bool decided by council voting
    pub resolved: bool,
    // dispute reward
    pub reward: u64,
    // did the disputer claim their payout + rep update
    pub claimed: bool,

    // bump
    pub bump: u8,
}
