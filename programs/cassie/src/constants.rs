use anchor_lang::prelude::*;

pub const MAX_COUNCIL_MEMBER: usize = 9;

// v1: dispute bond + min stake fixed at 750 USDC
pub const MIN_DISPUTE_BOND: u64 = 750;
pub const MIN_STAKE: u64 = 750;

#[constant]
pub const ADMIN_CONFIG_SEED: &str = "admin_config";

#[constant]
pub const QUESTION_CONFIG_SEED: &str = "question_config";

#[constant]
pub const ANSWER_SEED: &str = "answer";

#[constant]
pub const REPUTATION_SEED: &str = "reputation";

#[constant]
pub const OUTCOME_SEED: &str = "outcome";

pub const BPS_DENOMINATOR: u128 = 10_000;

// fixed-point scale. SCALE = 1.0x
pub const SCALE: u64 = 100;

// max reputation score, maps to ACCURACY_MAX_MULT
pub const MAX_SCORE: u64 = 500;

// max loyalty days, maps to LOYALTY_MAX_MULT
pub const MAX_DAYS: u32 = 365;

// accuracy adds up to +5.0x (so 1.0x..6.0x)
pub const ACCURACY_MAX_MULT: u64 = 500;

// loyalty adds up to +2.0x (so 1.0x..3.0x)
pub const LOYALTY_MAX_MULT: u64 = 200;

pub const GAIN: u64 = 10; // correct answer
pub const LOSS: u64 = 20; // wrong answer (loss > gain on purpose)
pub const DISPUTE_WIN_GAIN: u64 = 15;
pub const DISPUTE_LOSS: u64 = 25;
pub const COUNCIL_GAIN: u64 = 5;
pub const COUNCIL_LOSS: u64 = 10;

pub const SECONDS_PER_DAY: i64 = 86_400;

// pub const USDC_PUBKEY: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"); // mainnet
pub const USDC_PUBKEY: Pubkey = pubkey!("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"); // devnet
