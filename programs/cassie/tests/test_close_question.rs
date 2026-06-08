mod helper;

use helper::ask::{bounty_ata, question_pda};
use helper::close::outcome_pda;
use helper::close_question::{close_question, warp_past_close_grace};
use helper::dispute::warp_past_dispute_window;
use helper::settle::{fund_cranker, setup_resolved, settle, treasury_ata};
use helper::utils::token_balance;
use solana_signer::Signer;

// closing before the grace window elapses trips CloseGraceActive.
#[test]
fn close_before_grace_fails() {
    let (mut svm, admin, treasury, hash) = setup_resolved();
    warp_past_dispute_window(&mut svm, &hash);
    settle(&mut svm, &admin, treasury, &hash).unwrap();

    let cranker = fund_cranker(&mut svm);
    // no grace warp.
    assert!(close_question(&mut svm, &cranker, admin.pubkey(), treasury, &hash).is_err());
}

// closing a not-yet-settled question trips InvalidState.
#[test]
fn close_unsettled_fails() {
    let (mut svm, admin, treasury, hash) = setup_resolved();
    // still Resolved, never settled.
    let cranker = fund_cranker(&mut svm);
    assert!(close_question(&mut svm, &cranker, admin.pubkey(), treasury, &hash).is_err());
}

// after the grace window, a permissionless crank sweeps unclaimed funds to the
// treasury and tears down the question/outcome/pool accounts.
#[test]
fn close_after_grace_sweeps_and_closes() {
    let (mut svm, admin, treasury, hash) = setup_resolved();
    warp_past_dispute_window(&mut svm, &hash);
    settle(&mut svm, &admin, treasury, &hash).unwrap();
    warp_past_close_grace(&mut svm, &hash);

    // nobody claimed: the whole post-treasury pool is still sitting there.
    let pool_remaining = token_balance(&svm, bounty_ata(&hash));
    let treasury_before = token_balance(&svm, treasury_ata(treasury));
    assert!(pool_remaining > 0);

    let cranker = fund_cranker(&mut svm);
    assert!(close_question(&mut svm, &cranker, admin.pubkey(), treasury, &hash).is_ok());

    // unclaimed funds swept to treasury.
    assert_eq!(
        token_balance(&svm, treasury_ata(treasury)),
        treasury_before + pool_remaining
    );
    // accounts torn down.
    assert!(svm.get_account(&question_pda(&hash)).is_none());
    assert!(svm.get_account(&outcome_pda(&hash)).is_none());
    assert!(svm.get_account(&bounty_ata(&hash)).is_none());
}
