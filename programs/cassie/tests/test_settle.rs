mod helper;

use anchor_lang::AccountDeserialize;
use cassie::state::question::{Question, QuestionState};
use helper::ask::{bounty_ata, question_pda};
use helper::dispute::warp_past_dispute_window;
use helper::settle::{setup_resolved, setup_resolved_weighted, settle, treasury_ata};
use helper::update_config::{update_config, UpdateConfigParams};
use helper::utils::token_balance;

fn read_question(svm: &litesvm::LiteSVM, hash: &[u8; 32]) -> Question {
    let raw = svm.get_account(&question_pda(hash)).unwrap();
    Question::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

// optimistic resolve, no losers: bounty 1000, treasury 1% = 10, reward pool 990
// split across the single correct answer.
#[test]
fn settle_ok_optimistic() {
    let (mut svm, admin, treasury, hash) = setup_resolved();
    warp_past_dispute_window(&mut svm, &hash);
    let pool_before = token_balance(&svm, bounty_ata(&hash));

    assert!(settle(&mut svm, &admin, treasury, &hash).is_ok());

    let q = read_question(&svm, &hash);
    assert!(matches!(q.state, QuestionState::Settled));
    assert_eq!(q.per_answer_reward, 990);

    // only the treasury cut leaves the pool at settle; payouts happen at claim.
    assert_eq!(token_balance(&svm, treasury_ata(treasury)), 10);
    assert_eq!(token_balance(&svm, bounty_ata(&hash)), pool_before - 10);
}

// weighted resolve with a slashed loser: bounty 1000 + slash 375 = 1375 gross,
// treasury 1% = 13, distributable 1362 across 2 correct answers = 681 each.
#[test]
fn settle_weighted_slash() {
    let (mut svm, admin, treasury, hash) = setup_resolved_weighted();
    warp_past_dispute_window(&mut svm, &hash);

    assert!(settle(&mut svm, &admin, treasury, &hash).is_ok());

    let q = read_question(&svm, &hash);
    assert_eq!(q.per_answer_reward, 681);
    assert_eq!(token_balance(&svm, treasury_ata(treasury)), 13);
}

// settling before the dispute window closes trips DisputeWindowActive.
#[test]
fn settle_before_window() {
    let (mut svm, admin, treasury, hash) = setup_resolved();
    // no warp: dispute window still open.
    assert!(settle(&mut svm, &admin, treasury, &hash).is_err());
}

// settling an already-settled question trips InvalidState.
#[test]
fn settle_twice() {
    let (mut svm, admin, treasury, hash) = setup_resolved();
    warp_past_dispute_window(&mut svm, &hash);

    assert!(settle(&mut svm, &admin, treasury, &hash).is_ok());
    assert!(settle(&mut svm, &admin, treasury, &hash).is_err());
}

// settling on a frozen program trips ProgramFrozen.
#[test]
fn settle_when_frozen() {
    let (mut svm, admin, treasury, hash) = setup_resolved();
    warp_past_dispute_window(&mut svm, &hash);
    update_config(
        &mut svm,
        &admin,
        &UpdateConfigParams {
            freeze: Some(true),
            ..Default::default()
        },
    )
    .unwrap();

    assert!(settle(&mut svm, &admin, treasury, &hash).is_err());
}
