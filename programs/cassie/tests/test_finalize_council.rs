mod helper;

use anchor_lang::AccountDeserialize;
use cassie::state::outcome::{Outcome, Resolver};
use cassie::state::question::{Question, QuestionState};
use cassie::CouncilTotal;
use helper::ask::question_pda;
use helper::close::outcome_pda;
use helper::council_vote::{council_total_pda, setup_escalated, vote};
use helper::finalize_council::finalize;
use helper::update_config::{update_config, UpdateConfigParams};

fn read_question(svm: &litesvm::LiteSVM, hash: &[u8; 32]) -> Question {
    let raw = svm.get_account(&question_pda(hash)).unwrap();
    Question::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

fn read_total(svm: &litesvm::LiteSVM, hash: &[u8; 32]) -> CouncilTotal {
    let raw = svm.get_account(&council_total_pda(hash)).unwrap();
    CouncilTotal::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

fn read_outcome(svm: &litesvm::LiteSVM, hash: &[u8; 32]) -> Outcome {
    let raw = svm.get_account(&outcome_pda(hash)).unwrap();
    Outcome::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

// quorum of yes votes finalizes to a true verdict and overwrites the outcome.
#[test]
fn finalize_ok_yes() {
    let (mut svm, admin, members, hash) = setup_escalated();
    vote(&mut svm, &members[0], &hash, true).unwrap();
    vote(&mut svm, &members[1], &hash, true).unwrap();

    assert!(finalize(&mut svm, &admin, &hash).is_ok());

    let o = read_outcome(&svm, &hash);
    assert!(o.result);
    assert_eq!(o.resolver, Resolver::Council);
    assert_eq!(o.council_yes, 2);
    assert_eq!(o.council_no, 0);

    let q = read_question(&svm, &hash);
    assert!(matches!(q.state, QuestionState::Resolved));

    let total = read_total(&svm, &hash);
    assert_eq!(total.finalized_at, Some(true));
}

// quorum of no votes finalizes to a false verdict.
#[test]
fn finalize_verdict_no() {
    let (mut svm, admin, members, hash) = setup_escalated();
    vote(&mut svm, &members[0], &hash, false).unwrap();
    vote(&mut svm, &members[1], &hash, false).unwrap();

    assert!(finalize(&mut svm, &admin, &hash).is_ok());

    let o = read_outcome(&svm, &hash);
    assert!(!o.result);
    assert_eq!(o.council_no, 2);
}

// a tie (1 yes, 1 no) breaks to no.
#[test]
fn finalize_tie_is_no() {
    let (mut svm, admin, members, hash) = setup_escalated();
    vote(&mut svm, &members[0], &hash, true).unwrap();
    vote(&mut svm, &members[1], &hash, false).unwrap();

    assert!(finalize(&mut svm, &admin, &hash).is_ok());

    let o = read_outcome(&svm, &hash);
    assert!(!o.result);
    assert_eq!(o.council_yes, 1);
    assert_eq!(o.council_no, 1);
}

// fewer votes than quorum trips QuorumNotReached.
#[test]
fn finalize_quorum_not_reached() {
    let (mut svm, admin, members, hash) = setup_escalated();
    // single vote opens the round but quorum is 2.
    vote(&mut svm, &members[0], &hash, true).unwrap();

    assert!(finalize(&mut svm, &admin, &hash).is_err());
}

// finalizing an already-finalized question trips InvalidState.
#[test]
fn finalize_twice() {
    let (mut svm, admin, members, hash) = setup_escalated();
    vote(&mut svm, &members[0], &hash, true).unwrap();
    vote(&mut svm, &members[1], &hash, true).unwrap();

    assert!(finalize(&mut svm, &admin, &hash).is_ok());
    assert!(finalize(&mut svm, &admin, &hash).is_err());
}

// finalizing on a frozen program trips ProgramFrozen.
#[test]
fn finalize_when_frozen() {
    let (mut svm, admin, members, hash) = setup_escalated();
    vote(&mut svm, &members[0], &hash, true).unwrap();
    vote(&mut svm, &members[1], &hash, true).unwrap();
    update_config(
        &mut svm,
        &admin,
        &UpdateConfigParams {
            freeze: Some(true),
            ..Default::default()
        },
    )
    .unwrap();

    assert!(finalize(&mut svm, &admin, &hash).is_err());
}
