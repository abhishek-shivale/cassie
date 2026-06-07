mod helper;

use anchor_lang::AccountDeserialize;
use cassie::state::outcome::{Outcome, Resolver};
use cassie::state::question::{Question, QuestionState};
use helper::ask::question_pda;
use helper::close::{close, outcome_pda, warp_past_answer_window};
use helper::propose::{fund_proposer, propose, setup_with_question, ProposeParams};
use helper::update_config::{update_config, UpdateConfigParams};

fn read_question(svm: &litesvm::LiteSVM, hash: &[u8; 32]) -> Question {
    let raw = svm.get_account(&question_pda(hash)).unwrap();
    Question::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

fn read_outcome(svm: &litesvm::LiteSVM, hash: &[u8; 32]) -> Outcome {
    let raw = svm.get_account(&outcome_pda(hash)).unwrap();
    Outcome::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

// single "yes" answer resolves optimistically and opens the dispute window.
#[test]
fn close_resolved_optimistic_yes() {
    let (mut svm, admin, proposer, hash) = setup_with_question();
    propose(&mut svm, &proposer, &ProposeParams::default()).unwrap();
    warp_past_answer_window(&mut svm, &hash);

    assert!(close(&mut svm, &admin, &hash).is_ok());

    let o = read_outcome(&svm, &hash);
    assert!(o.result);
    assert_eq!(o.resolver, Resolver::Optimistic);
    assert_eq!(o.answer_count, 1);

    let q = read_question(&svm, &hash);
    assert!(matches!(q.state, QuestionState::Resolved));
    assert!(q.dispute_deadline > 0);
}

// single "no" answer resolves optimistically to false.
#[test]
fn close_resolved_optimistic_no() {
    let (mut svm, admin, proposer, hash) = setup_with_question();
    let params = ProposeParams {
        side: false,
        ..Default::default()
    };
    propose(&mut svm, &proposer, &params).unwrap();
    warp_past_answer_window(&mut svm, &hash);

    assert!(close(&mut svm, &admin, &hash).is_ok());

    let o = read_outcome(&svm, &hash);
    assert!(!o.result);
    assert_eq!(o.resolver, Resolver::Optimistic);
}

// equal-weight yes/no split (50% minority >= 35% divergence) escalates to council.
#[test]
fn close_escalate_divergence() {
    let (mut svm, admin, yes_voter, hash) = setup_with_question();
    propose(&mut svm, &yes_voter, &ProposeParams::default()).unwrap();

    let no_voter = fund_proposer(&mut svm);
    let no_params = ProposeParams {
        side: false,
        ..Default::default()
    };
    propose(&mut svm, &no_voter, &no_params).unwrap();
    warp_past_answer_window(&mut svm, &hash);

    assert!(close(&mut svm, &admin, &hash).is_ok());

    let q = read_question(&svm, &hash);
    assert!(matches!(q.state, QuestionState::Escalated));
    assert!(q.escalated);

    let o = read_outcome(&svm, &hash);
    assert_eq!(o.resolver, Resolver::Council);
    assert_eq!(o.answer_count, 2);
}

// closing with zero answers escalates (NoAnswer) to council.
#[test]
fn close_escalate_no_answer() {
    let (mut svm, admin, _proposer, hash) = setup_with_question();
    warp_past_answer_window(&mut svm, &hash);

    assert!(close(&mut svm, &admin, &hash).is_ok());

    let q = read_question(&svm, &hash);
    assert!(matches!(q.state, QuestionState::Escalated));

    let o = read_outcome(&svm, &hash);
    assert_eq!(o.resolver, Resolver::Council);
    assert_eq!(o.answer_count, 0);
}

// closing before the answer window ends trips AnswerWindowActive.
#[test]
fn close_before_window() {
    let (mut svm, admin, proposer, hash) = setup_with_question();
    propose(&mut svm, &proposer, &ProposeParams::default()).unwrap();
    // no warp: window still open.
    assert!(close(&mut svm, &admin, &hash).is_err());
}

// closing an already-closed question collides with the outcome PDA and fails.
#[test]
fn close_twice() {
    let (mut svm, admin, proposer, hash) = setup_with_question();
    propose(&mut svm, &proposer, &ProposeParams::default()).unwrap();
    warp_past_answer_window(&mut svm, &hash);

    assert!(close(&mut svm, &admin, &hash).is_ok());
    assert!(close(&mut svm, &admin, &hash).is_err());
}

// closing a frozen program trips ProgramFrozen.
#[test]
fn close_when_frozen() {
    let (mut svm, admin, proposer, hash) = setup_with_question();
    propose(&mut svm, &proposer, &ProposeParams::default()).unwrap();
    warp_past_answer_window(&mut svm, &hash);
    update_config(
        &mut svm,
        &admin,
        &UpdateConfigParams {
            freeze: Some(true),
            ..Default::default()
        },
    )
    .unwrap();

    assert!(close(&mut svm, &admin, &hash).is_err());
}
