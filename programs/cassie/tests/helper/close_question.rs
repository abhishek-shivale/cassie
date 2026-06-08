#![allow(dead_code)]
use crate::helper::ask::{bounty_ata, question_pda};
use crate::helper::close::outcome_pda;
use crate::helper::council_vote::council_total_pda;
use crate::helper::initialize::config_pda;
use crate::helper::settle::treasury_ata;
use crate::helper::utils::{send_ix, warp_unix};
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use cassie::constants::USDC_PUBKEY;
use cassie::state::outcome::Outcome;
use litesvm::types::TransactionResult;
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

// move the clock one second past the settlement close-grace window (24h).
pub fn warp_past_close_grace(svm: &mut LiteSVM, hash: &[u8; 32]) {
    let raw = svm.get_account(&outcome_pda(hash)).unwrap();
    let o = Outcome::try_deserialize(&mut raw.data.as_slice()).unwrap();
    warp_unix(svm, o.settled_at + 86_400 + 1);
}

fn close_question_ix(
    cranker: Pubkey,
    creator: Pubkey,
    treasury: Pubkey,
    hash: &[u8; 32],
    council: bool,
) -> Instruction {
    let data = cassie::instruction::CloseQuestion { hash: *hash }.data();

    let accounts = cassie::accounts::CloseQuestion {
        cranker,
        creator,
        question: question_pda(hash),
        config: config_pda(),
        outcome: outcome_pda(hash),
        council_total: council.then(|| council_total_pda(hash)),
        usdc_mint: USDC_PUBKEY,
        pool_ata: bounty_ata(hash),
        treasury_ata: treasury_ata(treasury),
        token_program: TOKEN_PROGRAM_ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: cassie::id(),
        accounts,
        data,
    }
}

pub fn close_question(
    svm: &mut LiteSVM,
    cranker: &Keypair,
    creator: Pubkey,
    treasury: Pubkey,
    hash: &[u8; 32],
) -> TransactionResult {
    let ix = close_question_ix(cranker.pubkey(), creator, treasury, hash, false);
    send_ix(svm, ix, cranker, &[cranker])
}

pub fn close_question_council(
    svm: &mut LiteSVM,
    cranker: &Keypair,
    creator: Pubkey,
    treasury: Pubkey,
    hash: &[u8; 32],
) -> TransactionResult {
    let ix = close_question_ix(cranker.pubkey(), creator, treasury, hash, true);
    send_ix(svm, ix, cranker, &[cranker])
}
