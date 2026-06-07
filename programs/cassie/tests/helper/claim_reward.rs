#![allow(dead_code)]
use crate::helper::ask::{ask, bounty_ata, question_pda, AskParams};
use crate::helper::close::{close, outcome_pda, warp_past_answer_window};
use crate::helper::dispute::warp_past_dispute_window;
use crate::helper::initialize::{config_pda, init_config, InitParams};
use crate::helper::propose::{fund_proposer, propose, reputation_pda, ProposeParams};
use crate::helper::settle::settle;
use crate::helper::utils::{ata, send_ix, set_token_account, setup_svm};
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{InstructionData, ToAccountMetas};
use cassie::constants::{ANSWER_SEED, USDC_PUBKEY};
use crate::helper::utils::pda;
use litesvm::types::TransactionResult;
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

pub fn answer_pda(hash: &[u8; 32], claimer: Pubkey) -> Pubkey {
    pda(&[ANSWER_SEED.as_ref(), hash.as_ref(), claimer.as_ref()]).0
}

fn init_and_ask() -> (LiteSVM, Keypair, Pubkey, [u8; 32]) {
    let (mut svm, admin) = setup_svm();
    let treasury = Pubkey::new_unique();
    let params = InitParams {
        treasury,
        ..Default::default()
    };
    init_config(&mut svm, &admin, &params).unwrap();
    set_token_account(&mut svm, admin.pubkey(), USDC_PUBKEY, 1_000_000);
    set_token_account(&mut svm, treasury, USDC_PUBKEY, 0);
    let hash = AskParams::default().hash;
    ask(&mut svm, &admin, &AskParams::default()).unwrap();
    (svm, admin, treasury, hash)
}

fn add_proposer(svm: &mut LiteSVM, hash: &[u8; 32], side: bool) -> Keypair {
    let proposer = fund_proposer(svm);
    let params = ProposeParams {
        hash: *hash,
        side,
        ..Default::default()
    };
    propose(svm, &proposer, &params).unwrap();
    proposer
}

// settled question (optimistic, single yes answer) + the winning proposer.
pub fn setup_settled_winner() -> (LiteSVM, Keypair, Keypair, [u8; 32]) {
    let (mut svm, admin, treasury, hash) = init_and_ask();
    let winner = add_proposer(&mut svm, &hash, true);
    warp_past_answer_window(&mut svm, &hash);
    close(&mut svm, &admin, &hash).unwrap();
    warp_past_dispute_window(&mut svm, &hash);
    settle(&mut svm, &admin, treasury, &hash).unwrap();
    (svm, admin, winner, hash)
}

// settled-by-weight question (2 yes, 1 no) + a winning and a losing proposer.
pub fn setup_settled_weighted() -> (LiteSVM, Keypair, Keypair, Keypair, [u8; 32]) {
    let (mut svm, admin, treasury, hash) = init_and_ask();
    let winner = add_proposer(&mut svm, &hash, true);
    add_proposer(&mut svm, &hash, true);
    let loser = add_proposer(&mut svm, &hash, false);
    warp_past_answer_window(&mut svm, &hash);
    close(&mut svm, &admin, &hash).unwrap();
    warp_past_dispute_window(&mut svm, &hash);
    settle(&mut svm, &admin, treasury, &hash).unwrap();
    (svm, admin, winner, loser, hash)
}

// resolved but NOT settled (optimistic) + the winning proposer.
pub fn setup_resolved_winner() -> (LiteSVM, Keypair, Keypair, [u8; 32]) {
    let (mut svm, admin, _treasury, hash) = init_and_ask();
    let winner = add_proposer(&mut svm, &hash, true);
    warp_past_answer_window(&mut svm, &hash);
    close(&mut svm, &admin, &hash).unwrap();
    (svm, admin, winner, hash)
}

pub fn claim_ix(claimer: Pubkey, hash: &[u8; 32]) -> Instruction {
    let data = cassie::instruction::ClaimReward { hash: *hash }.data();

    let accounts = cassie::accounts::ClaimReward {
        claimer,
        question: question_pda(hash),
        config: config_pda(),
        outcome: outcome_pda(hash),
        answer: Some(answer_pda(hash, claimer)),
        dispute: None,
        reputation: reputation_pda(claimer),
        usdc_mint: USDC_PUBKEY,
        pool_ata: bounty_ata(hash),
        claimer_ata: ata(claimer, USDC_PUBKEY),
        token_program: TOKEN_PROGRAM_ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: cassie::id(),
        accounts,
        data,
    }
}

pub fn claim(svm: &mut LiteSVM, claimer: &Keypair, hash: &[u8; 32]) -> TransactionResult {
    let ix = claim_ix(claimer.pubkey(), hash);
    send_ix(svm, ix, claimer, &[claimer])
}
