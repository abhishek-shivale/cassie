#![allow(dead_code)]
use crate::helper::utils::{pda, send_ix, set_mint};
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use cassie::constants::{ADMIN_CONFIG_SEED, USDC_PUBKEY};
use litesvm::types::TransactionResult;
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

pub const ANSWER_WINDOW: i64 = 3600;
pub const COUNCIL_WINDOW: i64 = 86400;
pub const DISPUTE_WINDOW: i64 = 7200;
pub const DIVERGENCE_BPS: u64 = 3500;
pub const MIN_BOUNTY: u64 = 10;
pub const SLASH_BPS: u64 = 5000;
pub const TREASURY_BPS: u64 = 100;
pub const COUNCIL_SIZE: u8 = 3;

pub struct InitParams {
    pub default_answer_window: i64,
    pub default_council_window: i64,
    pub default_dispute_window: i64,
    pub divergence_bps: u64,
    pub min_bounty: u64,
    pub slash_bps: u64,
    pub treasury: Pubkey,
    pub treasury_bps: u64,
    pub council: [Pubkey; 9],
    pub council_size: u8,
}

impl Default for InitParams {
    fn default() -> Self {
        Self {
            default_answer_window: ANSWER_WINDOW,
            default_council_window: COUNCIL_WINDOW,
            default_dispute_window: DISPUTE_WINDOW,
            divergence_bps: DIVERGENCE_BPS,
            min_bounty: MIN_BOUNTY,
            slash_bps: SLASH_BPS,
            treasury: Pubkey::new_unique(),
            treasury_bps: TREASURY_BPS,
            council: council_members(COUNCIL_SIZE),
            council_size: COUNCIL_SIZE,
        }
    }
}

pub fn council_members(size: u8) -> [Pubkey; 9] {
    let mut council = [Pubkey::default(); 9];
    for slot in council.iter_mut().take(size as usize) {
        *slot = Pubkey::new_unique();
    }
    council
}

pub fn config_pda() -> Pubkey {
    pda(&[ADMIN_CONFIG_SEED.as_ref()]).0
}

pub fn init_config_ix(admin: Pubkey, params: &InitParams) -> Instruction {
    let data = cassie::instruction::InitializeConfig {
        default_answer_window: params.default_answer_window,
        default_council_window: params.default_council_window,
        default_dispute_window: params.default_dispute_window,
        divergence_bps: params.divergence_bps,
        min_bounty: params.min_bounty,
        slash_bps: params.slash_bps,
        treasury: params.treasury,
        treasury_bps: params.treasury_bps,
        council: params.council,
        council_size: params.council_size,
    }
    .data();

    let accounts = cassie::accounts::InitializeConfig {
        admin,
        config: config_pda(),
        usdc_mint: USDC_PUBKEY,
        system_program: system_program::ID,
        token_program: TOKEN_PROGRAM_ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: cassie::id(),
        accounts,
        data,
    }
}

pub fn init_config(svm: &mut LiteSVM, admin: &Keypair, params: &InitParams) -> TransactionResult {
    set_mint(svm, USDC_PUBKEY, admin.pubkey(), 6);
    let ix = init_config_ix(admin.pubkey(), params);
    send_ix(svm, ix, admin, &[admin])
}
