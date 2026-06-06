use anchor_lang::prelude::*;

pub const MAX_COUNCIL_MEMBER: usize = 9;

// v1: dispute bond + min stake fixed at 750 USDC
pub const MIN_DISPUTE_BOND: u64 = 750;
pub const MIN_STAKE: u64 = 750;

#[constant]
pub const ADMIN_CONFIG_SEED: &str = "admin_config";

#[constant]
pub const QUESTION_CONFIG_SEED: &str = "question_config";

// pub const USDC_PUBKEY: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"); // mainnet
pub const USDC_PUBKEY: Pubkey = pubkey!("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"); // devnet
