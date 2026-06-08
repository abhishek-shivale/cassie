mod helper;

use anchor_lang::prelude::Pubkey;
use anchor_lang::AccountDeserialize;
use cassie::constants::USDC_PUBKEY;
use cassie::state::question::Question;
use cassie::Reputation;
use helper::ask::{ask, question_pda, AskParams};
use helper::claim_reward::claim_council;
use helper::close::{close, warp_past_answer_window};
use helper::council_vote::{council_vote_pda, vote};
use helper::dispute::warp_past_dispute_window;
use helper::finalize_council::finalize;
use helper::initialize::{init_config, InitParams};
use helper::propose::{fund_proposer, propose, reputation_pda, ProposeParams};
use helper::settle::settle_council;
use helper::utils::{ata, set_token_account, setup_svm, token_balance, ONE_SOL};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;

const COUNCIL_BPS: u64 = 1000; // 10%

fn read_question(svm: &LiteSVM, hash: &[u8; 32]) -> Question {
    let raw = svm.get_account(&question_pda(hash)).unwrap();
    Question::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

fn read_rep(svm: &LiteSVM, who: Pubkey) -> Reputation {
    let raw = svm.get_account(&reputation_pda(who)).unwrap();
    Reputation::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

fn usdc(svm: &LiteSVM, who: Pubkey) -> u64 {
    token_balance(svm, ata(who, USDC_PUBKEY))
}

// build a council-resolved (divergence) question with council_bps = 10%, settled.
// returns svm, council members, treasury, hash.
fn setup_council_settled() -> (LiteSVM, Vec<Keypair>, Pubkey, [u8; 32]) {
    let (mut svm, admin) = setup_svm();

    let members: Vec<Keypair> = (0..3).map(|_| Keypair::new()).collect();
    let mut council = [Pubkey::default(); 9];
    for (s, m) in council.iter_mut().zip(&members) {
        *s = m.pubkey();
    }
    let treasury = Pubkey::new_unique();
    let params = InitParams {
        council,
        council_size: 3,
        treasury,
        council_bps: COUNCIL_BPS,
        ..Default::default()
    };
    init_config(&mut svm, &admin, &params).unwrap();
    set_token_account(&mut svm, admin.pubkey(), USDC_PUBKEY, 1_000_000);
    set_token_account(&mut svm, treasury, USDC_PUBKEY, 0);
    for m in &members {
        svm.airdrop(&m.pubkey(), ONE_SOL).unwrap();
        // council members need a USDC ATA to receive their reward.
        set_token_account(&mut svm, m.pubkey(), USDC_PUBKEY, 0);
    }

    let hash = AskParams::default().hash;
    ask(&mut svm, &admin, &AskParams::default()).unwrap();

    // equal-weight yes/no -> escalate to council.
    let yes_voter = fund_proposer(&mut svm);
    let no_voter = fund_proposer(&mut svm);
    propose(
        &mut svm,
        &yes_voter,
        &ProposeParams { hash, side: true, ..Default::default() },
    )
    .unwrap();
    propose(
        &mut svm,
        &no_voter,
        &ProposeParams { hash, side: false, ..Default::default() },
    )
    .unwrap();
    warp_past_answer_window(&mut svm, &hash);
    close(&mut svm, &admin, &hash).unwrap();

    // council: 2 yes, 1 no -> verdict yes.
    vote(&mut svm, &members[0], &hash, true).unwrap();
    vote(&mut svm, &members[1], &hash, true).unwrap();
    vote(&mut svm, &members[2], &hash, false).unwrap();
    finalize(&mut svm, &admin, &hash).unwrap();

    warp_past_dispute_window(&mut svm, &hash);
    settle_council(&mut svm, &admin, treasury, &hash).unwrap();
    (svm, members, treasury, hash)
}

// settle carves council_bps of gross into a council pool, split across the
// voters on the winning side.
// gross = bounty 1000 + slash 375 = 1375; council pool = 10% = 137;
// 2 correct voters -> 68 each; answer pool = 1362 - 137 = 1225.
#[test]
fn council_reward_split_at_settle() {
    let (svm, _members, _treasury, hash) = setup_council_settled();
    let q = read_question(&svm, &hash);
    assert_eq!(q.council_reward_per_vote, 68);
    assert_eq!(q.per_answer_reward, 1225);
}

// a council member who voted with the verdict claims their share + council rep,
// and their vote PDA is closed.
#[test]
fn council_member_claims_reward() {
    let (mut svm, members, _treasury, hash) = setup_council_settled();
    let winner = &members[0]; // voted yes (verdict)
    let before = usdc(&svm, winner.pubkey());

    assert!(claim_council(&mut svm, winner, &hash).is_ok());

    assert_eq!(usdc(&svm, winner.pubkey()), before + 68);
    // vote PDA closed (rent reclaimed).
    assert!(svm
        .get_account(&council_vote_pda(&hash, winner.pubkey()))
        .is_none());

    let rep = read_rep(&svm, winner.pubkey());
    assert!(rep.is_council);
    assert_eq!(rep.score, 5); // COUNCIL_GAIN
}

// a council member who voted against the verdict gets 0 and a rep loss.
#[test]
fn council_wrong_voter_gets_nothing() {
    let (mut svm, members, _treasury, hash) = setup_council_settled();
    let loser = &members[2]; // voted no
    let before = usdc(&svm, loser.pubkey());

    assert!(claim_council(&mut svm, loser, &hash).is_ok());

    assert_eq!(usdc(&svm, loser.pubkey()), before); // no payout
    let rep = read_rep(&svm, loser.pubkey());
    assert_eq!(rep.score, 0); // COUNCIL_LOSS saturated at 0
}

// a council member can only claim once; the closed vote PDA blocks a re-claim.
#[test]
fn council_claim_twice_fails() {
    let (mut svm, members, _treasury, hash) = setup_council_settled();
    assert!(claim_council(&mut svm, &members[0], &hash).is_ok());
    assert!(claim_council(&mut svm, &members[0], &hash).is_err());
}
