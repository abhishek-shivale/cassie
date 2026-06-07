#![allow(dead_code)]
use crate::helper::ask::{ask, question_pda, AskParams};
use crate::helper::close::{close, warp_past_answer_window};
use crate::helper::initialize::{config_pda, init_config, InitParams};
use crate::helper::utils::{pda, send_ix, set_token_account, setup_svm, warp_unix, ONE_SOL};
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{system_program, AccountDeserialize, InstructionData, ToAccountMetas};
use cassie::constants::{COUNCIL_TOTAL_SEED, COUNCIL_VOTE_SEED, USDC_PUBKEY};
use cassie::CouncilTotal;
use litesvm::types::TransactionResult;
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;

pub const COUNCIL_SIZE: u8 = 3;

// init config whose council is a set of keypairs we control (each funded with SOL).
fn init_with_council(size: u8) -> (LiteSVM, Keypair, Vec<Keypair>) {
    let (mut svm, admin) = setup_svm();

    let members: Vec<Keypair> = (0..size).map(|_| Keypair::new()).collect();
    let mut council = [Pubkey::default(); 9];
    for (slot, m) in council.iter_mut().zip(members.iter()) {
        *slot = m.pubkey();
    }
    let params = InitParams {
        council,
        council_size: size,
        ..Default::default()
    };
    init_config(&mut svm, &admin, &params).unwrap();

    for m in &members {
        svm.airdrop(&m.pubkey(), ONE_SOL).unwrap();
    }
    (svm, admin, members)
}

// custom council + an asked question (state Asked, not yet escalated).
pub fn setup_asked() -> (LiteSVM, Keypair, Vec<Keypair>, [u8; 32]) {
    let (mut svm, admin, members) = init_with_council(COUNCIL_SIZE);
    set_token_account(&mut svm, admin.pubkey(), USDC_PUBKEY, 1_000_000);
    let ask_params = AskParams::default();
    ask(&mut svm, &admin, &ask_params).unwrap();
    (svm, admin, members, ask_params.hash)
}

// custom council + a question escalated at close (NoAnswer path, zero proposers).
pub fn setup_escalated() -> (LiteSVM, Keypair, Vec<Keypair>, [u8; 32]) {
    let (mut svm, admin, members, hash) = setup_asked();
    warp_past_answer_window(&mut svm, &hash);
    close(&mut svm, &admin, &hash).unwrap();
    (svm, admin, members, hash)
}

pub fn council_total_pda(hash: &[u8; 32]) -> Pubkey {
    pda(&[COUNCIL_TOTAL_SEED.as_ref(), hash.as_ref()]).0
}

pub fn council_vote_pda(hash: &[u8; 32], voter: Pubkey) -> Pubkey {
    pda(&[COUNCIL_VOTE_SEED.as_ref(), hash.as_ref(), voter.as_ref()]).0
}

// move the clock one second past the council voting window.
pub fn warp_past_council_window(svm: &mut LiteSVM, hash: &[u8; 32]) {
    let raw = svm.get_account(&council_total_pda(hash)).unwrap();
    let total = CouncilTotal::try_deserialize(&mut raw.data.as_slice()).unwrap();
    // default_council_window is 86400 (see InitParams default).
    warp_unix(svm, total.opened_at + 86400 + 1);
}

pub fn vote_ix(voter: Pubkey, hash: &[u8; 32], vote: bool) -> Instruction {
    let data = cassie::instruction::CouncilVote { hash: *hash, vote }.data();

    let accounts = cassie::accounts::Vote {
        voter,
        question: question_pda(hash),
        config: config_pda(),
        council_total: council_total_pda(hash),
        council_vote: council_vote_pda(hash, voter),
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: cassie::id(),
        accounts,
        data,
    }
}

pub fn vote(svm: &mut LiteSVM, voter: &Keypair, hash: &[u8; 32], v: bool) -> TransactionResult {
    let ix = vote_ix(voter.pubkey(), hash, v);
    send_ix(svm, ix, voter, &[voter])
}
