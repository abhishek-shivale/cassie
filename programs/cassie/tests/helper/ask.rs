#![allow(dead_code)]
use crate::helper::initialize::{init_config, InitParams};
use crate::helper::utils::{ata, pda, send_ix, set_token_account, setup_svm};
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use cassie::constants::{QUESTION_CONFIG_SEED, USDC_PUBKEY};
use litesvm::types::TransactionResult;
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;
use spl_associated_token_account_interface::program::ID as ATA_PROGRAM_ID;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

pub const BOUNTY: u64 = 1_000;
pub const QUESTIONER_FUNDS: u64 = 1_000_000;

pub struct AskParams {
    pub hash: [u8; 32],
    pub bounty: u64,
    pub category: u8,
    pub metadata_uri: [u8; 128],
    pub callback_program: Pubkey,
    pub callback_discriminator: [u8; 8],
}

impl Default for AskParams {
    fn default() -> Self {
        Self {
            hash: [7u8; 32],
            bounty: BOUNTY,
            category: 1,
            metadata_uri: [0u8; 128],
            callback_program: Pubkey::default(),
            callback_discriminator: [0u8; 8],
        }
    }
}

// initialized config + a questioner whose USDC ATA is funded.
pub fn setup_with_questioner() -> (LiteSVM, Keypair) {
    let (mut svm, admin) = setup_svm();
    init_config(&mut svm, &admin, &InitParams::default()).unwrap();
    // admin doubles as questioner; fund its USDC ATA.
    set_token_account(&mut svm, admin.pubkey(), USDC_PUBKEY, QUESTIONER_FUNDS);
    (svm, admin)
}

pub fn question_pda(hash: &[u8; 32]) -> Pubkey {
    pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]).0
}

pub fn bounty_ata(hash: &[u8; 32]) -> Pubkey {
    ata(question_pda(hash), USDC_PUBKEY)
}

pub fn ask_ix(questioner: Pubkey, params: &AskParams) -> Instruction {
    let data = cassie::instruction::Ask {
        hash: params.hash,
        bounty: params.bounty,
        category: params.category,
        metadata_uri: params.metadata_uri,
        callback_program: params.callback_program,
        callback_discriminator: params.callback_discriminator,
    }
    .data();

    let accounts = cassie::accounts::Ask {
        questioner,
        config: crate::helper::initialize::config_pda(),
        question: question_pda(&params.hash),
        usdc_mint: USDC_PUBKEY,
        questioner_ata: ata(questioner, USDC_PUBKEY),
        bounty_ata: bounty_ata(&params.hash),
        token_program: TOKEN_PROGRAM_ID,
        associated_token_program: ATA_PROGRAM_ID,
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: cassie::id(),
        accounts,
        data,
    }
}

pub fn ask(svm: &mut LiteSVM, questioner: &Keypair, params: &AskParams) -> TransactionResult {
    let ix = ask_ix(questioner.pubkey(), params);
    send_ix(svm, ix, questioner, &[questioner])
}
