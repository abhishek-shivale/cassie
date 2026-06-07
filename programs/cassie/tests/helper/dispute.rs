#![allow(dead_code)]
use crate::helper::ask::{bounty_ata, question_pda};
use crate::helper::close::{close, warp_past_answer_window};
use crate::helper::initialize::config_pda;
use crate::helper::propose::{fund_proposer, propose, reputation_pda, setup_with_question, ProposeParams};
use crate::helper::utils::{ata, pda, send_ix, set_token_account, warp_unix, ONE_SOL};
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{system_program, AccountDeserialize, InstructionData, ToAccountMetas};
use cassie::constants::{DISPUTE_SEED, MIN_DISPUTE_BOND, USDC_PUBKEY};
use cassie::state::question::Question;
use litesvm::types::TransactionResult;
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;
use spl_associated_token_account_interface::program::ID as ATA_PROGRAM_ID;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

pub const BOND: u64 = MIN_DISPUTE_BOND;
pub const DISPUTER_FUNDS: u64 = 1_000_000;

pub struct DisputeParams {
    pub hash: [u8; 32],
    pub bond: u64,
    // disputer's claimed correct outcome; must differ from the resolved result.
    pub claimed_outcome: bool,
    pub reason_hash: [u8; 128],
}

pub fn dispute_params(hash: [u8; 32]) -> DisputeParams {
    DisputeParams {
        hash,
        bond: BOND,
        claimed_outcome: false,
        reason_hash: [0u8; 128],
    }
}

pub fn dispute_pda(hash: &[u8; 32]) -> Pubkey {
    pda(&[DISPUTE_SEED.as_ref(), hash.as_ref()]).0
}

pub fn fund_disputer(svm: &mut LiteSVM) -> Keypair {
    let disputer = Keypair::new();
    svm.airdrop(&disputer.pubkey(), ONE_SOL).unwrap();
    set_token_account(svm, disputer.pubkey(), USDC_PUBKEY, DISPUTER_FUNDS);
    disputer
}

// a question resolved optimistically to `true` (single yes answer), inside the
// dispute window, plus a funded disputer.
pub fn setup_resolved() -> (LiteSVM, Keypair, Keypair, [u8; 32]) {
    let (mut svm, admin, proposer, hash) = setup_with_question();
    propose(&mut svm, &proposer, &ProposeParams::default()).unwrap();
    warp_past_answer_window(&mut svm, &hash);
    close(&mut svm, &admin, &hash).unwrap();
    let disputer = fund_disputer(&mut svm);
    (svm, admin, disputer, hash)
}

// a question escalated at close time (equal-weight yes/no split), plus a funded disputer.
pub fn setup_escalated() -> (LiteSVM, Keypair, Keypair, [u8; 32]) {
    let (mut svm, admin, yes_voter, hash) = setup_with_question();
    propose(&mut svm, &yes_voter, &ProposeParams::default()).unwrap();
    let no_voter = fund_proposer(&mut svm);
    let no_params = ProposeParams {
        side: false,
        ..Default::default()
    };
    propose(&mut svm, &no_voter, &no_params).unwrap();
    warp_past_answer_window(&mut svm, &hash);
    close(&mut svm, &admin, &hash).unwrap();
    let disputer = fund_disputer(&mut svm);
    (svm, admin, disputer, hash)
}

// move the clock one second past the question's dispute deadline.
pub fn warp_past_dispute_window(svm: &mut LiteSVM, hash: &[u8; 32]) {
    let raw = svm.get_account(&question_pda(hash)).unwrap();
    let q = Question::try_deserialize(&mut raw.data.as_slice()).unwrap();
    warp_unix(svm, q.dispute_deadline + 1);
}

pub fn dispute_ix(disputer: Pubkey, params: &DisputeParams) -> Instruction {
    let data = cassie::instruction::Dispute {
        hash: params.hash,
        bond: params.bond,
        claimed_outcome: params.claimed_outcome,
        reason_hash: params.reason_hash,
    }
    .data();

    let accounts = cassie::accounts::Dispute {
        disputer,
        question: question_pda(&params.hash),
        usdc_mint: USDC_PUBKEY,
        disputer_ata: ata(disputer, USDC_PUBKEY),
        bond_ata: bounty_ata(&params.hash),
        config: config_pda(),
        outcome: crate::helper::close::outcome_pda(&params.hash),
        disputer_config: dispute_pda(&params.hash),
        reputation: reputation_pda(disputer),
        token_program: TOKEN_PROGRAM_ID,
        system_program: system_program::ID,
        associated_token_program: ATA_PROGRAM_ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: cassie::id(),
        accounts,
        data,
    }
}

pub fn dispute(svm: &mut LiteSVM, disputer: &Keypair, params: &DisputeParams) -> TransactionResult {
    let ix = dispute_ix(disputer.pubkey(), params);
    send_ix(svm, ix, disputer, &[disputer])
}
