mod helper;

use anchor_lang::AccountDeserialize;
use cassie::constants::USDC_PUBKEY;
use cassie::state::answer::Answer;
use cassie::state::question::{Question, QuestionState};
use helper::ask::{bounty_ata, question_pda, BOUNTY};
use helper::propose::{
    answer_pda, fund_proposer, propose, setup_with_question, ProposeParams, STAKE,
};
use helper::update_config::{update_config, UpdateConfigParams};
use helper::utils::{ata, token_balance, warp_unix};
use solana_signer::Signer;

fn read_question(svm: &litesvm::LiteSVM, hash: &[u8; 32]) -> Question {
    let raw = svm.get_account(&question_pda(hash)).unwrap();
    Question::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

fn read_answer(svm: &litesvm::LiteSVM, hash: &[u8; 32], proposer: &solana_keypair::Keypair) -> Answer {
    let raw = svm.get_account(&answer_pda(hash, proposer.pubkey())).unwrap();
    Answer::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

// happy path: first answer flips the question to Answering, records the answer,
// tallies the yes side, and moves the stake into the question's bond pool.
#[test]
fn propose_ok() {
    let (mut svm, _admin, proposer, hash) = setup_with_question();
    let params = ProposeParams::default();
    let funded = token_balance(&svm, ata(proposer.pubkey(), USDC_PUBKEY));

    assert!(propose(&mut svm, &proposer, &params).is_ok());

    let ans = read_answer(&svm, &hash, &proposer);
    assert_eq!(ans.answerer, proposer.pubkey());
    assert!(ans.side);
    assert_eq!(ans.stake, STAKE);

    let q = read_question(&svm, &hash);
    assert!(matches!(q.state, QuestionState::Answering));
    assert_eq!(q.yes_count, 1);
    assert_eq!(q.no_count, 0);
    assert_eq!(q.total_yes_stake, STAKE as u128);

    // pool now holds bounty + stake; proposer debited by the stake.
    assert_eq!(token_balance(&svm, bounty_ata(&hash)), BOUNTY + STAKE);
    assert_eq!(
        token_balance(&svm, ata(proposer.pubkey(), USDC_PUBKEY)),
        funded - STAKE
    );
}

// a "no" answer tallies the opposite side.
#[test]
fn propose_no_side() {
    let (mut svm, _admin, proposer, hash) = setup_with_question();
    let params = ProposeParams {
        side: false,
        ..Default::default()
    };

    assert!(propose(&mut svm, &proposer, &params).is_ok());

    let q = read_question(&svm, &hash);
    assert_eq!(q.no_count, 1);
    assert_eq!(q.yes_count, 0);
    assert_eq!(q.total_no_stake, STAKE as u128);
}

// two distinct proposers can both answer the same question.
#[test]
fn propose_two_proposers() {
    let (mut svm, _admin, first, hash) = setup_with_question();
    assert!(propose(&mut svm, &first, &ProposeParams::default()).is_ok());

    let second = fund_proposer(&mut svm);
    let params = ProposeParams {
        side: false,
        ..Default::default()
    };
    assert!(propose(&mut svm, &second, &params).is_ok());

    let q = read_question(&svm, &hash);
    assert_eq!(q.yes_count, 1);
    assert_eq!(q.no_count, 1);
}

// stake must equal MIN_STAKE exactly; anything else trips InsufficientStake.
#[test]
fn propose_wrong_stake() {
    let (mut svm, _admin, proposer, _hash) = setup_with_question();
    let params = ProposeParams {
        stake: STAKE - 1,
        ..Default::default()
    };
    assert!(propose(&mut svm, &proposer, &params).is_err());
}

// the same proposer answering twice collides with the answer PDA (init) and fails.
#[test]
fn propose_duplicate_answer() {
    let (mut svm, _admin, proposer, _hash) = setup_with_question();
    let params = ProposeParams::default();

    assert!(propose(&mut svm, &proposer, &params).is_ok());
    assert!(propose(&mut svm, &proposer, &params).is_err());
}

// answering past the answer_deadline trips AnswerWindowClosed.
#[test]
fn propose_after_window() {
    let (mut svm, _admin, proposer, hash) = setup_with_question();
    let deadline = read_question(&svm, &hash).answer_deadline;
    // jump one second past the deadline.
    warp_unix(&mut svm, deadline + 1);

    assert!(propose(&mut svm, &proposer, &ProposeParams::default()).is_err());
}

// proposing on a frozen program trips ProgramFrozen.
#[test]
fn propose_when_frozen() {
    let (mut svm, admin, proposer, _hash) = setup_with_question();
    update_config(
        &mut svm,
        &admin,
        &UpdateConfigParams {
            freeze: Some(true),
            ..Default::default()
        },
    )
    .unwrap();

    assert!(propose(&mut svm, &proposer, &ProposeParams::default()).is_err());
}
