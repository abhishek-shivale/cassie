mod helper;

use anchor_lang::AccountDeserialize;
use cassie::state::question::{Question, QuestionState};
use cassie::{CouncilTotal, CouncilVote};
use helper::ask::question_pda;
use helper::council_vote::{
    council_total_pda, council_vote_pda, setup_asked, setup_escalated, vote,
    warp_past_council_window,
};
use helper::update_config::{update_config, UpdateConfigParams};
use helper::utils::ONE_SOL;
use solana_keypair::Keypair;
use solana_signer::Signer;

fn read_question(svm: &litesvm::LiteSVM, hash: &[u8; 32]) -> Question {
    let raw = svm.get_account(&question_pda(hash)).unwrap();
    Question::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

fn read_total(svm: &litesvm::LiteSVM, hash: &[u8; 32]) -> CouncilTotal {
    let raw = svm.get_account(&council_total_pda(hash)).unwrap();
    CouncilTotal::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

fn read_vote(svm: &litesvm::LiteSVM, hash: &[u8; 32], voter: &Keypair) -> CouncilVote {
    let raw = svm.get_account(&council_vote_pda(hash, voter.pubkey())).unwrap();
    CouncilVote::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

// first council vote opens the round, transitions to Council, and tallies a yes.
#[test]
fn vote_ok() {
    let (mut svm, _admin, members, hash) = setup_escalated();

    assert!(vote(&mut svm, &members[0], &hash, true).is_ok());

    let total = read_total(&svm, &hash);
    assert_eq!(total.yes_count, 1);
    assert_eq!(total.no_count, 0);
    assert!(total.opened_at > 0);

    let q = read_question(&svm, &hash);
    assert!(matches!(q.state, QuestionState::Council));

    let v = read_vote(&svm, &hash, &members[0]);
    assert_eq!(v.member, members[0].pubkey());
    assert!(v.vote);
}

// a "no" vote tallies the no side.
#[test]
fn vote_no() {
    let (mut svm, _admin, members, hash) = setup_escalated();
    assert!(vote(&mut svm, &members[0], &hash, false).is_ok());

    let total = read_total(&svm, &hash);
    assert_eq!(total.no_count, 1);
    assert_eq!(total.yes_count, 0);
}

// several distinct members tally independently.
#[test]
fn vote_multiple_members() {
    let (mut svm, _admin, members, hash) = setup_escalated();
    assert!(vote(&mut svm, &members[0], &hash, true).is_ok());
    assert!(vote(&mut svm, &members[1], &hash, true).is_ok());
    assert!(vote(&mut svm, &members[2], &hash, false).is_ok());

    let total = read_total(&svm, &hash);
    assert_eq!(total.yes_count, 2);
    assert_eq!(total.no_count, 1);
}

// a non-council signer trips NotCouncilMember.
#[test]
fn vote_non_member() {
    let (mut svm, _admin, _members, hash) = setup_escalated();
    let intruder = Keypair::new();
    svm.airdrop(&intruder.pubkey(), ONE_SOL).unwrap();

    assert!(vote(&mut svm, &intruder, &hash, true).is_err());
}

// the same member voting twice collides with the vote PDA and fails.
#[test]
fn vote_double() {
    let (mut svm, _admin, members, hash) = setup_escalated();
    assert!(vote(&mut svm, &members[0], &hash, true).is_ok());
    assert!(vote(&mut svm, &members[0], &hash, false).is_err());
}

// voting on a non-escalated question (still Asked) trips InvalidState.
#[test]
fn vote_wrong_state() {
    let (mut svm, _admin, members, hash) = setup_asked();
    assert!(vote(&mut svm, &members[0], &hash, true).is_err());
}

// voting after the council window closed trips CouncilWindowClosed.
#[test]
fn vote_after_window() {
    let (mut svm, _admin, members, hash) = setup_escalated();
    // first vote opens the round.
    assert!(vote(&mut svm, &members[0], &hash, true).is_ok());
    warp_past_council_window(&mut svm, &hash);

    assert!(vote(&mut svm, &members[1], &hash, true).is_err());
}

// voting on a frozen program trips ProgramFrozen.
#[test]
fn vote_when_frozen() {
    let (mut svm, admin, members, hash) = setup_escalated();
    update_config(
        &mut svm,
        &admin,
        &UpdateConfigParams {
            freeze: Some(true),
            ..Default::default()
        },
    )
    .unwrap();

    assert!(vote(&mut svm, &members[0], &hash, true).is_err());
}
