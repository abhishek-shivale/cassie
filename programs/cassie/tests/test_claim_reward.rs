mod helper;

use anchor_lang::AccountDeserialize;
use cassie::constants::USDC_PUBKEY;
use cassie::state::answer::Answer;
use cassie::Reputation;
use helper::ask::bounty_ata;
use helper::claim_reward::{
    answer_pda, claim, setup_resolved_winner, setup_settled_weighted, setup_settled_winner,
};
use helper::propose::reputation_pda;
use helper::update_config::{update_config, UpdateConfigParams};
use helper::utils::{ata, token_balance};
use solana_keypair::Keypair;
use solana_signer::Signer;

fn read_answer(svm: &litesvm::LiteSVM, hash: &[u8; 32], claimer: &Keypair) -> Answer {
    let raw = svm.get_account(&answer_pda(hash, claimer.pubkey())).unwrap();
    Answer::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

fn read_rep(svm: &litesvm::LiteSVM, claimer: &Keypair) -> Reputation {
    let raw = svm.get_account(&reputation_pda(claimer.pubkey())).unwrap();
    Reputation::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

// winner claims stake (750) + per-answer reward (990) = 1740, gets rep credit.
#[test]
fn claim_ok_winner() {
    let (mut svm, _admin, winner, hash) = setup_settled_winner();
    let before = token_balance(&svm, ata(winner.pubkey(), USDC_PUBKEY));
    let pool_before = token_balance(&svm, bounty_ata(&hash));

    assert!(claim(&mut svm, &winner, &hash).is_ok());

    assert_eq!(
        token_balance(&svm, ata(winner.pubkey(), USDC_PUBKEY)),
        before + 1740
    );
    assert_eq!(token_balance(&svm, bounty_ata(&hash)), pool_before - 1740);

    let ans = read_answer(&svm, &hash, &winner);
    assert!(ans.claimed);

    let rep = read_rep(&svm, &winner);
    assert_eq!(rep.correct, 1);
    assert_eq!(rep.answered, 1);
    assert_eq!(rep.score, 10); // GAIN
}

// loser on a slashed question recovers half the stake (750 * 50% = 375) and is slashed.
#[test]
fn claim_loser_slashed() {
    let (mut svm, _admin, _winner, loser, hash) = setup_settled_weighted();
    let before = token_balance(&svm, ata(loser.pubkey(), USDC_PUBKEY));

    assert!(claim(&mut svm, &loser, &hash).is_ok());

    assert_eq!(
        token_balance(&svm, ata(loser.pubkey(), USDC_PUBKEY)),
        before + 375
    );

    let ans = read_answer(&svm, &hash, &loser);
    assert!(ans.claimed);

    let rep = read_rep(&svm, &loser);
    assert_eq!(rep.correct, 0);
    assert_eq!(rep.times_slashed, 1);
    assert_eq!(rep.total_slashed, 375);
}

// claiming twice trips AlreadyClaimed (the answer is already marked claimed).
#[test]
fn claim_twice() {
    let (mut svm, _admin, winner, hash) = setup_settled_winner();
    assert!(claim(&mut svm, &winner, &hash).is_ok());
    assert!(claim(&mut svm, &winner, &hash).is_err());
}

// claiming before the question is settled trips InvalidState.
#[test]
fn claim_before_settled() {
    let (mut svm, _admin, winner, hash) = setup_resolved_winner();
    assert!(claim(&mut svm, &winner, &hash).is_err());
}

// claiming on a frozen program trips ProgramFrozen.
#[test]
fn claim_when_frozen() {
    let (mut svm, admin, winner, hash) = setup_settled_winner();
    update_config(
        &mut svm,
        &admin,
        &UpdateConfigParams {
            freeze: Some(true),
            ..Default::default()
        },
    )
    .unwrap();

    assert!(claim(&mut svm, &winner, &hash).is_err());
}
