mod helper;

use anchor_lang::AccountDeserialize;
use cassie::constants::USDC_PUBKEY;
use cassie::DisputeConfig;
use cassie::state::question::{Question, QuestionState};
use helper::ask::{bounty_ata, question_pda};
use helper::dispute::{
    dispute, dispute_params, dispute_pda, setup_escalated, setup_resolved,
    warp_past_dispute_window, BOND,
};
use helper::update_config::{update_config, UpdateConfigParams};
use helper::utils::{ata, token_balance};
use solana_signer::Signer;

fn read_question(svm: &litesvm::LiteSVM, hash: &[u8; 32]) -> Question {
    let raw = svm.get_account(&question_pda(hash)).unwrap();
    Question::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

fn read_dispute(svm: &litesvm::LiteSVM, hash: &[u8; 32]) -> DisputeConfig {
    let raw = svm.get_account(&dispute_pda(hash)).unwrap();
    DisputeConfig::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

// happy path: a counter-bond escalates the resolved question and funds the pool.
#[test]
fn dispute_ok() {
    let (mut svm, _admin, disputer, hash) = setup_resolved();
    let params = dispute_params(hash);
    let pool_before = token_balance(&svm, bounty_ata(&hash));
    let funded = token_balance(&svm, ata(disputer.pubkey(), USDC_PUBKEY));

    assert!(dispute(&mut svm, &disputer, &params).is_ok());

    let d = read_dispute(&svm, &hash);
    assert_eq!(d.disputer, disputer.pubkey());
    assert_eq!(d.bond_amount, BOND);
    assert!(!d.claimed_outcome);
    assert!(!d.resolved);
    assert!(!d.claimed);

    let q = read_question(&svm, &hash);
    assert!(matches!(q.state, QuestionState::Escalated));
    assert!(q.escalated);

    // bond moved from disputer into the question pool.
    assert_eq!(token_balance(&svm, bounty_ata(&hash)), pool_before + BOND);
    assert_eq!(
        token_balance(&svm, ata(disputer.pubkey(), USDC_PUBKEY)),
        funded - BOND
    );
}

// bond not equal to MIN_DISPUTE_BOND trips InsufficientStake.
#[test]
fn dispute_wrong_bond() {
    let (mut svm, _admin, disputer, hash) = setup_resolved();
    let mut params = dispute_params(hash);
    params.bond = BOND - 1;
    assert!(dispute(&mut svm, &disputer, &params).is_err());
}

// claiming the same outcome the question already resolved to trips InvalidDisputeOutcome.
#[test]
fn dispute_same_outcome() {
    let (mut svm, _admin, disputer, hash) = setup_resolved();
    let mut params = dispute_params(hash);
    // resolved result is true; claiming true is not a real dispute.
    params.claimed_outcome = true;
    assert!(dispute(&mut svm, &disputer, &params).is_err());
}

// disputing a question that escalated at close (not Resolved) trips InvalidState.
#[test]
fn dispute_not_resolved() {
    let (mut svm, _admin, disputer, hash) = setup_escalated();
    let params = dispute_params(hash);
    assert!(dispute(&mut svm, &disputer, &params).is_err());
}

// disputing after the dispute window closed trips DisputeWindowClosed.
#[test]
fn dispute_after_window() {
    let (mut svm, _admin, disputer, hash) = setup_resolved();
    warp_past_dispute_window(&mut svm, &hash);
    let params = dispute_params(hash);
    assert!(dispute(&mut svm, &disputer, &params).is_err());
}

// a second dispute on the same question collides with the dispute PDA and fails.
#[test]
fn dispute_twice() {
    let (mut svm, _admin, disputer, hash) = setup_resolved();
    let params = dispute_params(hash);

    assert!(dispute(&mut svm, &disputer, &params).is_ok());
    assert!(dispute(&mut svm, &disputer, &params).is_err());
}

// disputing on a frozen program trips ProgramFrozen.
#[test]
fn dispute_when_frozen() {
    let (mut svm, admin, disputer, hash) = setup_resolved();
    update_config(
        &mut svm,
        &admin,
        &UpdateConfigParams {
            freeze: Some(true),
            ..Default::default()
        },
    )
    .unwrap();

    let params = dispute_params(hash);
    assert!(dispute(&mut svm, &disputer, &params).is_err());
}
