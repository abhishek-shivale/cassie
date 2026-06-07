#![allow(dead_code)]
use crate::helper::initialize::{config_pda, init_config, InitParams};
use crate::helper::utils::{send_ix, setup_svm};
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::types::TransactionResult;
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;

#[derive(Default)]
pub struct UpdateConfigParams {
    pub default_dispute_window: Option<i64>,
    pub default_council_window: Option<i64>,
    pub default_answer_window: Option<i64>,
    pub freeze: Option<bool>,
}

pub fn setup_initialized() -> (LiteSVM, Keypair) {
    let (mut svm, admin) = setup_svm();
    init_config(&mut svm, &admin, &InitParams::default()).unwrap();
    (svm, admin)
}

pub fn update_config_ix(admin: Pubkey, params: &UpdateConfigParams) -> Instruction {
    let data = cassie::instruction::UpdateConfig {
        default_dispute_window: params.default_dispute_window,
        default_council_window: params.default_council_window,
        default_answer_window: params.default_answer_window,
        freeze: params.freeze,
    }
    .data();

    let accounts = cassie::accounts::UpdateConfig {
        admin,
        config: config_pda(),
    }
    .to_account_metas(None);

    Instruction {
        program_id: cassie::id(),
        accounts,
        data,
    }
}

pub fn update_config(
    svm: &mut LiteSVM,
    admin: &Keypair,
    params: &UpdateConfigParams,
) -> TransactionResult {
    let ix = update_config_ix(admin.pubkey(), params);
    send_ix(svm, ix, admin, &[admin])
}
