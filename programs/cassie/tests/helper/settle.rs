#![allow(dead_code)]
use crate::helper::ask::{ask, bounty_ata, question_pda, AskParams};
use crate::helper::close::{close, outcome_pda, warp_past_answer_window};
use crate::helper::initialize::{config_pda, init_config, InitParams};
use crate::helper::propose::{fund_proposer, propose, ProposeParams};
use crate::helper::utils::{ata, send_ix, set_token_account, setup_svm, ONE_SOL};
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{InstructionData, ToAccountMetas};
use cassie::constants::USDC_PUBKEY;
use litesvm::types::TransactionResult;
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

// init config with a known treasury, fund the asker, and create the treasury's
// (empty) USDC ATA so settle can pay the protocol fee into it.
fn init_with_treasury() -> (LiteSVM, Keypair, Pubkey) {
    let (mut svm, admin) = setup_svm();
    let treasury = Pubkey::new_unique();
    let params = InitParams {
        treasury,
        ..Default::default()
    };
    init_config(&mut svm, &admin, &params).unwrap();
    set_token_account(&mut svm, admin.pubkey(), USDC_PUBKEY, 1_000_000);
    set_token_account(&mut svm, treasury, USDC_PUBKEY, 0);
    (svm, admin, treasury)
}

fn propose_side(svm: &mut LiteSVM, hash: &[u8; 32], side: bool) {
    let proposer = fund_proposer(svm);
    let params = ProposeParams {
        hash: *hash,
        side,
        ..Default::default()
    };
    propose(svm, &proposer, &params).unwrap();
}

// resolved optimistically (single yes answer, no losers). still inside the
// dispute window; callers warp past it before settling.
pub fn setup_resolved() -> (LiteSVM, Keypair, Pubkey, [u8; 32]) {
    let (mut svm, admin, treasury) = init_with_treasury();
    let hash = AskParams::default().hash;
    ask(&mut svm, &admin, &AskParams::default()).unwrap();
    propose_side(&mut svm, &hash, true);
    warp_past_answer_window(&mut svm, &hash);
    close(&mut svm, &admin, &hash).unwrap();
    (svm, admin, treasury, hash)
}

// resolved by weight (2 yes, 1 no): minority 33% < 35% divergence, so the no
// side loses and gets slashed at settle.
pub fn setup_resolved_weighted() -> (LiteSVM, Keypair, Pubkey, [u8; 32]) {
    let (mut svm, admin, treasury) = init_with_treasury();
    let hash = AskParams::default().hash;
    ask(&mut svm, &admin, &AskParams::default()).unwrap();
    propose_side(&mut svm, &hash, true);
    propose_side(&mut svm, &hash, true);
    propose_side(&mut svm, &hash, false);
    warp_past_answer_window(&mut svm, &hash);
    close(&mut svm, &admin, &hash).unwrap();
    (svm, admin, treasury, hash)
}

pub fn treasury_ata(treasury: Pubkey) -> Pubkey {
    ata(treasury, USDC_PUBKEY)
}

pub const MEMO_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");

pub fn setup_resolved_with_callback(discriminator: [u8; 8]) -> (LiteSVM, Keypair, Pubkey, [u8; 32]) {
    let (mut svm, admin, treasury) = init_with_treasury();
    let ask_params = AskParams {
        callback_program: MEMO_PROGRAM_ID,
        callback_discriminator: discriminator,
        ..Default::default()
    };
    let hash = ask_params.hash;
    ask(&mut svm, &admin, &ask_params).unwrap();
    propose_side(&mut svm, &hash, true);
    warp_past_answer_window(&mut svm, &hash);
    close(&mut svm, &admin, &hash).unwrap();
    (svm, admin, treasury, hash)
}

fn settle_ix_inner(
    cranker: Pubkey,
    treasury: Pubkey,
    hash: &[u8; 32],
    dispute: Option<Pubkey>,
    council_total: Option<Pubkey>,
    callback_program: Option<Pubkey>,
) -> Instruction {
    let data = cassie::instruction::SettleQuestion { hash: *hash }.data();

    let accounts = cassie::accounts::Settle {
        cranker,
        question: question_pda(hash),
        config: config_pda(),
        outcome: outcome_pda(hash),
        usdc_mint: USDC_PUBKEY,
        pool_ata: bounty_ata(hash),
        treasury_ata: treasury_ata(treasury),
        dispute,
        council_total,
        callback_program,
        token_program: TOKEN_PROGRAM_ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: cassie::id(),
        accounts,
        data,
    }
}

pub fn settle_ix(cranker: Pubkey, treasury: Pubkey, hash: &[u8; 32]) -> Instruction {
    settle_ix_inner(cranker, treasury, hash, None, None, None)
}

pub fn settle(
    svm: &mut LiteSVM,
    cranker: &Keypair,
    treasury: Pubkey,
    hash: &[u8; 32],
) -> TransactionResult {
    let ix = settle_ix(cranker.pubkey(), treasury, hash);
    send_ix(svm, ix, cranker, &[cranker])
}

fn dispute_pda(hash: &[u8; 32]) -> Pubkey {
    crate::helper::dispute::dispute_pda(hash)
}

fn council_total_pda(hash: &[u8; 32]) -> Pubkey {
    crate::helper::council_vote::council_total_pda(hash)
}

// settle a disputed (but not council-resolved) question.
pub fn settle_disputed(
    svm: &mut LiteSVM,
    cranker: &Keypair,
    treasury: Pubkey,
    hash: &[u8; 32],
) -> TransactionResult {
    let ix = settle_ix_inner(
        cranker.pubkey(),
        treasury,
        hash,
        Some(dispute_pda(hash)),
        None,
        None,
    );
    send_ix(svm, ix, cranker, &[cranker])
}

// settle a council-resolved question (passes the council tally).
pub fn settle_council(
    svm: &mut LiteSVM,
    cranker: &Keypair,
    treasury: Pubkey,
    hash: &[u8; 32],
) -> TransactionResult {
    let ix = settle_ix_inner(
        cranker.pubkey(),
        treasury,
        hash,
        None,
        Some(council_total_pda(hash)),
        None,
    );
    send_ix(svm, ix, cranker, &[cranker])
}

// settle a question that was disputed AND escalated to council.
pub fn settle_disputed_council(
    svm: &mut LiteSVM,
    cranker: &Keypair,
    treasury: Pubkey,
    hash: &[u8; 32],
) -> TransactionResult {
    let ix = settle_ix_inner(
        cranker.pubkey(),
        treasury,
        hash,
        Some(dispute_pda(hash)),
        Some(council_total_pda(hash)),
        None,
    );
    send_ix(svm, ix, cranker, &[cranker])
}

// settle and fire the consumer callback CPI into `callback_program`.
pub fn settle_with_callback(
    svm: &mut LiteSVM,
    cranker: &Keypair,
    treasury: Pubkey,
    hash: &[u8; 32],
    callback_program: Pubkey,
) -> TransactionResult {
    let ix = settle_ix_inner(
        cranker.pubkey(),
        treasury,
        hash,
        None,
        None,
        Some(callback_program),
    );
    send_ix(svm, ix, cranker, &[cranker])
}

// convenience: fresh funded cranker (settle is permissionless).
pub fn fund_cranker(svm: &mut LiteSVM) -> Keypair {
    let cranker = Keypair::new();
    svm.airdrop(&cranker.pubkey(), ONE_SOL).unwrap();
    cranker
}
