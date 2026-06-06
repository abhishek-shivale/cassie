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

// returns the initialized svm, admin, and the council seeded into the config.
pub fn setup_initialized() -> (LiteSVM, Keypair, [Pubkey; 9]) {
    let (mut svm, admin) = setup_svm();
    let params = InitParams::default();
    let council = params.council;
    init_config(&mut svm, &admin, &params).unwrap();
    (svm, admin, council)
}

pub fn update_council_ix(admin: Pubkey, old: Pubkey, new: Pubkey) -> Instruction {
    let data = cassie::instruction::UpdateCouncil { old, new }.data();

    let accounts = cassie::accounts::UpdateCouncil {
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

pub fn update_council(
    svm: &mut LiteSVM,
    admin: &Keypair,
    old: Pubkey,
    new: Pubkey,
) -> TransactionResult {
    let ix = update_council_ix(admin.pubkey(), old, new);
    send_ix(svm, ix, admin, &[admin])
}
