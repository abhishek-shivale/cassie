mod helper;

use anchor_lang::prelude::Pubkey;
use anchor_lang::AccountDeserialize;
use cassie::state::question::{Question, QuestionState};
use helper::ask::question_pda;
use helper::dispute::warp_past_dispute_window;
use helper::settle::{
    setup_resolved_with_callback, settle, settle_with_callback, MEMO_PROGRAM_ID,
};

fn read_question(svm: &litesvm::LiteSVM, hash: &[u8; 32]) -> Question {
    let raw = svm.get_account(&question_pda(hash)).unwrap();
    Question::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

// success: payload (discriminator ++ hash ++ result) is valid UTF-8, so the memo
// "consumer" accepts the CPI and the question settles.
#[test]
fn callback_success() {
    // all bytes <= 0x7F -> valid UTF-8 (default hash is [7u8; 32], result is 0/1).
    let (mut svm, admin, treasury, hash) = setup_resolved_with_callback([1, 2, 3, 4, 5, 6, 7, 8]);
    warp_past_dispute_window(&mut svm, &hash);

    assert!(settle_with_callback(&mut svm, &admin, treasury, &hash, MEMO_PROGRAM_ID).is_ok());
    assert!(matches!(
        read_question(&svm, &hash).state,
        QuestionState::Settled
    ));
}

// failure: an invalid-UTF-8 discriminator makes the memo consumer revert; that
// bubbles up as CallbackInvocationFailed and the whole settle tx rolls back.
#[test]
fn callback_failure_reverts_settle() {
    // 0xFF is an invalid UTF-8 byte -> memo rejects the payload.
    let (mut svm, admin, treasury, hash) =
        setup_resolved_with_callback([0xFF, 1, 2, 3, 4, 5, 6, 7]);
    warp_past_dispute_window(&mut svm, &hash);

    assert!(settle_with_callback(&mut svm, &admin, treasury, &hash, MEMO_PROGRAM_ID).is_err());
    // atomic: settle reverted, question is still Resolved (not Settled).
    assert!(matches!(
        read_question(&svm, &hash).state,
        QuestionState::Resolved
    ));
}

// passing a callback_program account that doesn't match the one stored on the
// question trips CallbackInvocationFailed (require_keys_eq).
#[test]
fn callback_program_mismatch() {
    let (mut svm, admin, treasury, hash) = setup_resolved_with_callback([1, 2, 3, 4, 5, 6, 7, 8]);
    warp_past_dispute_window(&mut svm, &hash);

    let wrong = Pubkey::new_unique();
    assert!(settle_with_callback(&mut svm, &admin, treasury, &hash, wrong).is_err());
}

// a question that registered a callback but is settled WITHOUT the callback
// account also trips CallbackInvocationFailed.
#[test]
fn callback_missing_account() {
    let (mut svm, admin, treasury, hash) = setup_resolved_with_callback([1, 2, 3, 4, 5, 6, 7, 8]);
    warp_past_dispute_window(&mut svm, &hash);

    // plain settle() passes no callback_program account.
    assert!(settle(&mut svm, &admin, treasury, &hash).is_err());
}
