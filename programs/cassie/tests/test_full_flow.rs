mod helper;

use anchor_lang::prelude::Pubkey;
use anchor_lang::AccountDeserialize;
use cassie::constants::USDC_PUBKEY;
use cassie::state::answer::Answer;
use cassie::state::outcome::{Outcome, Resolver};
use cassie::state::question::{Question, QuestionState};
use cassie::{DisputeConfig, Reputation};

use helper::ask::{ask, question_pda, AskParams};
use helper::claim_reward::{answer_pda, claim, claim_dispute_only};
use helper::close::{close, outcome_pda, warp_past_answer_window};
use helper::council_vote::vote;
use helper::dispute::{dispute, dispute_params, dispute_pda, fund_disputer, warp_past_dispute_window};
use helper::finalize_council::finalize;
use helper::initialize::{init_config, InitParams};
use helper::propose::{fund_proposer, propose, reputation_pda, ProposeParams};
use helper::settle::{settle, settle_disputed};
use helper::utils::{ata, set_token_account, setup_svm, token_balance, ONE_SOL};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;

// ---- shared reads ------------------------------------------------------------

fn read_question(svm: &LiteSVM, hash: &[u8; 32]) -> Question {
    let raw = svm.get_account(&question_pda(hash)).unwrap();
    Question::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

fn read_outcome(svm: &LiteSVM, hash: &[u8; 32]) -> Outcome {
    let raw = svm.get_account(&outcome_pda(hash)).unwrap();
    Outcome::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

fn read_dispute(svm: &LiteSVM, hash: &[u8; 32]) -> DisputeConfig {
    let raw = svm.get_account(&dispute_pda(hash)).unwrap();
    DisputeConfig::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

fn read_answer(svm: &LiteSVM, hash: &[u8; 32], who: Pubkey) -> Answer {
    let raw = svm.get_account(&answer_pda(hash, who)).unwrap();
    Answer::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

fn read_rep(svm: &LiteSVM, who: Pubkey) -> Reputation {
    let raw = svm.get_account(&reputation_pda(who)).unwrap();
    Reputation::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

fn usdc(svm: &LiteSVM, who: Pubkey) -> u64 {
    token_balance(svm, ata(who, USDC_PUBKEY))
}

// init config with a controlled council + known treasury, fund the asker and the
// treasury ATA, and airdrop SOL to council members. returns (svm, admin, council, treasury).
fn init_full(council_size: u8) -> (LiteSVM, Keypair, Vec<Keypair>, Pubkey) {
    let (mut svm, admin) = setup_svm();

    let members: Vec<Keypair> = (0..council_size).map(|_| Keypair::new()).collect();
    let mut council = [Pubkey::default(); 9];
    for (slot, m) in council.iter_mut().zip(&members) {
        *slot = m.pubkey();
    }
    let treasury = Pubkey::new_unique();
    let params = InitParams {
        council,
        council_size,
        treasury,
        ..Default::default()
    };
    init_config(&mut svm, &admin, &params).unwrap();

    set_token_account(&mut svm, admin.pubkey(), USDC_PUBKEY, 1_000_000);
    set_token_account(&mut svm, treasury, USDC_PUBKEY, 0);
    for m in &members {
        svm.airdrop(&m.pubkey(), ONE_SOL).unwrap();
    }
    (svm, admin, members, treasury)
}

fn yes(hash: [u8; 32]) -> ProposeParams {
    ProposeParams {
        hash,
        side: true,
        ..Default::default()
    }
}

fn no(hash: [u8; 32]) -> ProposeParams {
    ProposeParams {
        hash,
        side: false,
        ..Default::default()
    }
}

// =============================================================================
// Flow A: optimistic happy path
// init -> ask -> propose(yes) -> close(Resolved) -> settle -> claim
// =============================================================================
#[test]
fn flow_optimistic_full_lifecycle() {
    let (mut svm, admin, _council, treasury) = init_full(3);
    let hash = AskParams::default().hash;

    ask(&mut svm, &admin, &AskParams::default()).unwrap();
    assert!(matches!(read_question(&svm, &hash).state, QuestionState::Asked));

    let winner = fund_proposer(&mut svm);
    propose(&mut svm, &winner, &yes(hash)).unwrap();
    assert!(matches!(
        read_question(&svm, &hash).state,
        QuestionState::Answering
    ));

    warp_past_answer_window(&mut svm, &hash);
    close(&mut svm, &admin, &hash).unwrap();
    assert!(matches!(
        read_question(&svm, &hash).state,
        QuestionState::Resolved
    ));
    assert_eq!(read_outcome(&svm, &hash).resolver, Resolver::Optimistic);

    warp_past_dispute_window(&mut svm, &hash);
    settle(&mut svm, &admin, treasury, &hash).unwrap();
    assert!(matches!(
        read_question(&svm, &hash).state,
        QuestionState::Settled
    ));
    // bounty 1000, treasury 1% = 10, reward pool 990 to the single winner.
    assert_eq!(usdc(&svm, treasury), 10);
    assert_eq!(read_question(&svm, &hash).per_answer_reward, 990);

    let before = usdc(&svm, winner.pubkey());
    claim(&mut svm, &winner, &hash).unwrap();
    // stake 750 + reward 990 = 1740.
    assert_eq!(usdc(&svm, winner.pubkey()), before + 1740);
    assert!(read_answer(&svm, &hash, winner.pubkey()).claimed);
    assert_eq!(read_rep(&svm, winner.pubkey()).correct, 1);
}

// =============================================================================
// Flow B: divergence -> council vote -> finalize -> settle -> claims
// =============================================================================
#[test]
fn flow_council_full_lifecycle() {
    let (mut svm, admin, council, treasury) = init_full(3);
    let hash = AskParams::default().hash;

    ask(&mut svm, &admin, &AskParams::default()).unwrap();
    let winner = fund_proposer(&mut svm);
    let loser = fund_proposer(&mut svm);
    propose(&mut svm, &winner, &yes(hash)).unwrap();
    propose(&mut svm, &loser, &no(hash)).unwrap();

    warp_past_answer_window(&mut svm, &hash);
    close(&mut svm, &admin, &hash).unwrap();
    // equal weight -> escalate to council.
    assert!(matches!(
        read_question(&svm, &hash).state,
        QuestionState::Escalated
    ));

    // council: 2 yes, 1 no -> verdict yes.
    vote(&mut svm, &council[0], &hash, true).unwrap();
    assert!(matches!(
        read_question(&svm, &hash).state,
        QuestionState::Council
    ));
    vote(&mut svm, &council[1], &hash, true).unwrap();
    vote(&mut svm, &council[2], &hash, false).unwrap();

    finalize(&mut svm, &admin, &hash).unwrap();
    let o = read_outcome(&svm, &hash);
    assert!(o.result);
    assert_eq!(o.resolver, Resolver::Council);
    assert_eq!(o.council_yes, 2);
    assert!(matches!(
        read_question(&svm, &hash).state,
        QuestionState::Resolved
    ));

    warp_past_dispute_window(&mut svm, &hash);
    settle(&mut svm, &admin, treasury, &hash).unwrap();
    // loser stake 750 slashed 50% = 375; gross 1375; treasury 13; pool 1362 / 1 correct.
    assert_eq!(usdc(&svm, treasury), 13);
    assert_eq!(read_question(&svm, &hash).per_answer_reward, 1362);

    let w_before = usdc(&svm, winner.pubkey());
    let l_before = usdc(&svm, loser.pubkey());
    claim(&mut svm, &winner, &hash).unwrap();
    claim(&mut svm, &loser, &hash).unwrap();
    // winner: 750 + 1362 = 2112; loser: 50% of 750 = 375.
    assert_eq!(usdc(&svm, winner.pubkey()), w_before + 2112);
    assert_eq!(usdc(&svm, loser.pubkey()), l_before + 375);
}

// =============================================================================
// Flow C: dispute WON -> council overturns -> settle(disputed) -> claim dispute
// init -> ask -> propose(yes) -> close(Resolved true) -> dispute(false)
//      -> council votes false -> finalize(false) -> settle -> disputer claims
// =============================================================================
#[test]
fn flow_dispute_won_full_lifecycle() {
    let (mut svm, admin, council, treasury) = init_full(3);
    let hash = AskParams::default().hash;

    ask(&mut svm, &admin, &AskParams::default()).unwrap();
    let answerer = fund_proposer(&mut svm);
    propose(&mut svm, &answerer, &yes(hash)).unwrap();

    warp_past_answer_window(&mut svm, &hash);
    close(&mut svm, &admin, &hash).unwrap();
    assert!(read_outcome(&svm, &hash).result); // resolved true

    // disputer counter-bonds claiming the opposite (false).
    let disputer = fund_disputer(&mut svm);
    dispute(&mut svm, &disputer, &dispute_params(hash)).unwrap();
    assert!(matches!(
        read_question(&svm, &hash).state,
        QuestionState::Escalated
    ));

    // council sides with the disputer: false wins (2 false, 1 true).
    vote(&mut svm, &council[0], &hash, false).unwrap();
    vote(&mut svm, &council[1], &hash, false).unwrap();
    vote(&mut svm, &council[2], &hash, true).unwrap();
    finalize(&mut svm, &admin, &hash).unwrap();
    assert!(!read_outcome(&svm, &hash).result); // overturned to false

    warp_past_dispute_window(&mut svm, &hash);
    settle_disputed(&mut svm, &admin, treasury, &hash).unwrap();

    let d = read_dispute(&svm, &hash);
    assert!(d.resolved);
    // loser (yes) stake 750 slashed 50% = 375; dispute reward = bond 750 + 25% of 375 (93) = 843.
    assert_eq!(d.reward, 843);

    let before = usdc(&svm, disputer.pubkey());
    claim_dispute_only(&mut svm, &disputer, &hash).unwrap();
    assert_eq!(usdc(&svm, disputer.pubkey()), before + 843);
    assert_eq!(read_rep(&svm, disputer.pubkey()).disputes_won, 1);

    // the original yes answer is now wrong: recovers half its stake.
    let a_before = usdc(&svm, answerer.pubkey());
    claim(&mut svm, &answerer, &hash).unwrap();
    assert_eq!(usdc(&svm, answerer.pubkey()), a_before + 375);
}

// =============================================================================
// Flow D: dispute LOST -> council upholds -> settle(disputed) -> claim (0 payout)
// =============================================================================
#[test]
fn flow_dispute_lost_full_lifecycle() {
    let (mut svm, admin, council, treasury) = init_full(3);
    let hash = AskParams::default().hash;

    ask(&mut svm, &admin, &AskParams::default()).unwrap();
    let answerer = fund_proposer(&mut svm);
    propose(&mut svm, &answerer, &yes(hash)).unwrap();

    warp_past_answer_window(&mut svm, &hash);
    close(&mut svm, &admin, &hash).unwrap();

    let disputer = fund_disputer(&mut svm);
    dispute(&mut svm, &disputer, &dispute_params(hash)).unwrap();

    // council upholds the original verdict: true wins (2 true, 1 false).
    vote(&mut svm, &council[0], &hash, true).unwrap();
    vote(&mut svm, &council[1], &hash, true).unwrap();
    vote(&mut svm, &council[2], &hash, false).unwrap();
    finalize(&mut svm, &admin, &hash).unwrap();
    assert!(read_outcome(&svm, &hash).result); // stays true

    warp_past_dispute_window(&mut svm, &hash);
    settle_disputed(&mut svm, &admin, treasury, &hash).unwrap();

    let d = read_dispute(&svm, &hash);
    assert!(!d.resolved);
    assert_eq!(d.reward, 0);

    // disputer lost: claim succeeds (rep update) but pays out nothing.
    let before = usdc(&svm, disputer.pubkey());
    claim_dispute_only(&mut svm, &disputer, &hash).unwrap();
    assert_eq!(usdc(&svm, disputer.pubkey()), before);
    assert_eq!(read_rep(&svm, disputer.pubkey()).disputes_lost, 1);

    // the original yes answer is correct: stake 750 + reward 990 = 1740.
    let a_before = usdc(&svm, answerer.pubkey());
    claim(&mut svm, &answerer, &hash).unwrap();
    assert_eq!(usdc(&svm, answerer.pubkey()), a_before + 1740);
}
