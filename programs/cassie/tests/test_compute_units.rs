mod helper;

use anchor_lang::prelude::Pubkey;
use cassie::constants::USDC_PUBKEY;
use helper::ask::{ask, AskParams};
use helper::close::{close, warp_past_answer_window};
use helper::council_vote::vote;
use helper::dispute::{dispute, dispute_params, fund_disputer, warp_past_dispute_window};
use helper::finalize_council::finalize;
use helper::initialize::{init_config, InitParams};
use helper::propose::{fund_proposer, propose, ProposeParams};
use helper::settle::{settle, settle_disputed_council};
use helper::claim_reward::{claim, claim_dispute_only};
use helper::update_config::{update_config, UpdateConfigParams};
use helper::update_council::update_council;
use helper::utils::{set_token_account, setup_svm, ONE_SOL};
use litesvm::types::TransactionResult;
use solana_keypair::Keypair;
use solana_signer::Signer;

// print one row: instruction name + compute units consumed.
fn cu(label: &str, r: TransactionResult) {
    let meta = r.unwrap_or_else(|e| panic!("{label} failed: {:?}", e.err));
    println!("{:<26} {:>7} CU", label, meta.compute_units_consumed);
}

fn ask_params(hash: [u8; 32]) -> AskParams {
    AskParams {
        hash,
        ..Default::default()
    }
}

// run every instruction once and print its compute-unit cost.
// view with:  cargo test -p cassie --test test_compute_units -- --nocapture
#[test]
fn report_compute_units() {
    let (mut svm, admin) = setup_svm();

    // controlled council (so council votes can sign) + known treasury.
    let members: Vec<Keypair> = (0..3).map(|_| Keypair::new()).collect();
    let mut council = [Pubkey::default(); 9];
    for (s, m) in council.iter_mut().zip(&members) {
        *s = m.pubkey();
    }
    let treasury = Pubkey::new_unique();
    for m in &members {
        svm.airdrop(&m.pubkey(), ONE_SOL).unwrap();
    }

    println!("\n=== cassie compute units (per instruction) ===");

    cu(
        "initialize_config",
        {
            let p = InitParams {
                council,
                council_size: 3,
                treasury,
                ..Default::default()
            };
            init_config(&mut svm, &admin, &p)
        },
    );
    set_token_account(&mut svm, admin.pubkey(), USDC_PUBKEY, 10_000_000);
    set_token_account(&mut svm, treasury, USDC_PUBKEY, 0);

    cu(
        "update_config",
        update_config(
            &mut svm,
            &admin,
            &UpdateConfigParams {
                freeze: Some(false),
                ..Default::default()
            },
        ),
    );
    cu(
        "update_council",
        update_council(&mut svm, &admin, members[0].pubkey(), Pubkey::new_unique()),
    );

    // ---- optimistic lifecycle: ask -> propose -> close -> settle -> claim ----
    let h1 = [11u8; 32];
    cu("ask", ask(&mut svm, &admin, &ask_params(h1)));
    let p1 = fund_proposer(&mut svm);
    cu(
        "propose",
        propose(
            &mut svm,
            &p1,
            &ProposeParams {
                hash: h1,
                side: true,
                ..Default::default()
            },
        ),
    );
    warp_past_answer_window(&mut svm, &h1);
    cu("close_proposers", close(&mut svm, &admin, &h1));
    warp_past_dispute_window(&mut svm, &h1);
    cu("settle_question", settle(&mut svm, &admin, treasury, &h1));
    cu("claim_reward", claim(&mut svm, &p1, &h1));

    // ---- dispute + council lifecycle: dispute -> vote -> finalize -> settle ----
    let h2 = [22u8; 32];
    ask(&mut svm, &admin, &ask_params(h2)).unwrap();
    let p2 = fund_proposer(&mut svm);
    propose(
        &mut svm,
        &p2,
        &ProposeParams {
            hash: h2,
            side: true,
            ..Default::default()
        },
    )
    .unwrap();
    warp_past_answer_window(&mut svm, &h2);
    close(&mut svm, &admin, &h2).unwrap();

    let disputer = fund_disputer(&mut svm);
    cu(
        "dispute",
        dispute(&mut svm, &disputer, &dispute_params(h2)),
    );
    cu("council_vote", vote(&mut svm, &members[1], &h2, false));
    vote(&mut svm, &members[2], &h2, false).unwrap();
    cu("finalize_council", finalize(&mut svm, &admin, &h2));
    warp_past_dispute_window(&mut svm, &h2);
    cu(
        "settle_question (disputed)",
        settle_disputed_council(&mut svm, &admin, treasury, &h2),
    );
    cu(
        "claim_reward (dispute)",
        claim_dispute_only(&mut svm, &disputer, &h2),
    );

    println!("(limit is 200_000 CU per instruction)\n");
}
