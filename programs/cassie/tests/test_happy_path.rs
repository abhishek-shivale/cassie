mod dep;
use anchor_lang::prelude::Clock;
use dep::*;

use anchor_lang::prelude::*;
use cassie::{
    Answer, CouncilTotal, DisputeConfig, OracleConfig, Outcome, Question, QuestionState,
    Reputation, Resolver, ADMIN_CONFIG_SEED, ANSWER_SEED, COUNCIL_TOTAL_SEED, DISPUTE_SEED,
    MIN_DISPUTE_BOND, MIN_STAKE, OUTCOME_SEED, QUESTION_CONFIG_SEED, REPUTATION_SEED,
    SECONDS_PER_DAY, USDC_PUBKEY,
};
use litesvm::LiteSVM;
use solana_signer::Signer;

fn account_exists(svm: &LiteSVM, addr: &Pubkey) -> bool {
    svm.get_account(addr)
        .map(|a| a.lamports > 0)
        .unwrap_or(false)
}

#[test]
fn test_happy_path() -> Result<()> {
    let (mut svm, owner) = setup();
    let owner_pubkey = owner.pubkey();
    let treasury_pubkey = get_new_account(&mut svm).pubkey();

    let mut members = council_members(&mut svm);
    let mut clock = svm.get_sysvar::<Clock>();
    let initial_time = clock.unix_timestamp;

    mint_token(&mut svm, USDC_PUBKEY, owner_pubkey);
    let council: [Pubkey; 9] = members.each_ref().map(|m| m.pubkey());
    let config_pda = get_pda(&[ADMIN_CONFIG_SEED.as_ref()]);

    // ===================== Initialize Config =====================
    initialize_config(&mut svm, council, treasury_pubkey, &owner);

    let cfg: OracleConfig = get_account_data(&svm, &config_pda);
    assert_eq!(cfg.admin, owner_pubkey, "admin should match owner");
    assert_eq!(cfg.treasury, treasury_pubkey, "treasury should match");
    assert_eq!(cfg.mint, USDC_PUBKEY, "mint should be USDC");
    assert_eq!(cfg.council_size, 9, "council size should be 9");
    assert_eq!(cfg.quorum, 6, "quorum should be 2/3 of 9 = 6");
    assert_eq!(
        cfg.default_answer_window, 3600,
        "answer window should be 3600"
    );
    assert_eq!(
        cfg.default_dispute_window, 7200,
        "initial dispute window should be 7200"
    );
    assert_eq!(
        cfg.default_council_window, SECONDS_PER_DAY,
        "council window should be 86400"
    );
    assert_eq!(cfg.min_bounty, 10, "min bounty should be 10");
    assert_eq!(cfg.min_stake, MIN_STAKE, "min stake should be 5");
    assert_eq!(
        cfg.min_dispute_bond, MIN_DISPUTE_BOND,
        "min dispute bond should be 5"
    );
    assert_eq!(cfg.slash_bps, SLASH_BPS, "slash bps should be 5000");
    assert_eq!(
        cfg.treasury_bps, TREASURY_BPS,
        "treasury bps should be 1000"
    );
    assert_eq!(cfg.council_bps, 1500, "council bps should be 1500");
    assert_eq!(cfg.divergence_bps, 3500, "divergence bps should be 3500");
    assert!(!cfg.freeze, "should not be frozen");
    assert_eq!(
        cfg.council[..9],
        council[..9],
        "council members should match"
    );

    // ===================== Update Config =====================
    update_config(&mut svm, &owner);

    let cfg: OracleConfig = get_account_data(&svm, &config_pda);
    assert_eq!(
        cfg.default_dispute_window, 7250,
        "dispute window should be updated to 7250"
    );
    assert_eq!(
        cfg.default_answer_window, 3600,
        "answer window should be unchanged"
    );
    assert_eq!(
        cfg.default_council_window, SECONDS_PER_DAY,
        "council window should be unchanged"
    );
    assert!(!cfg.freeze, "freeze should still be false");

    // ===================== Update Council =====================
    let new_council_member = get_new_account(&mut svm);
    let old_member = council[0];
    update_council(&mut svm, &owner, new_council_member.pubkey(), old_member);

    let cfg: OracleConfig = get_account_data(&svm, &config_pda);
    assert_eq!(
        cfg.council[0],
        new_council_member.pubkey(),
        "first council slot should be replaced"
    );
    assert_eq!(
        cfg.council[1], council[1],
        "second council slot should be unchanged"
    );
    assert!(
        !cfg.council.contains(&old_member),
        "old member should no longer be in council"
    );
    members[0] = new_council_member;

    // ===================== Ask Question =====================
    let hash = [0u8; 32];
    let asker = get_new_account(&mut svm);
    let question_pda = get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]);
    ask_ix(&mut svm, &asker, hash);

    let q: Question = get_account_data(&svm, &question_pda);
    assert_eq!(q.creator, asker.pubkey(), "creator should be asker");
    assert_eq!(q.hash, hash, "hash should match");
    assert_eq!(q.bounty, 70, "bounty should be 70");
    assert_eq!(q.state, QuestionState::Asked, "state should be Asked");
    assert_eq!(q.total_yes_weight, 0, "no yes weight yet");
    assert_eq!(q.total_no_weight, 0, "no no weight yet");
    assert_eq!(q.yes_count, 0, "yes count should be 0");
    assert_eq!(q.no_count, 0, "no count should be 0");
    assert!(!q.has_dispute, "no dispute yet");
    assert!(!q.escalated, "not escalated");
    assert_eq!(q.category, b'b', "category should be 'b' for boxing");
    assert_eq!(
        q.created_at, initial_time,
        "created_at should be initial time"
    );
    assert_eq!(
        q.answer_deadline,
        initial_time + 3600,
        "answer deadline should be created + 3600"
    );
    assert_eq!(q.per_answer_reward, 0, "no per answer reward yet");
    assert_eq!(q.council_reward_per_vote, 0, "no council reward yet");

    let asker_ata = ata(asker.pubkey(), USDC_PUBKEY);
    assert_eq!(
        token_balance(&svm, &asker_ata),
        70_000 - 70,
        "asker should have 69_930 after bounty"
    );

    // ===================== Propose Answer =====================
    let proposer = &get_new_account(&mut svm);
    let proposer_ata = ata(proposer.pubkey(), USDC_PUBKEY);
    let answer_pda = get_pda(&[
        ANSWER_SEED.as_ref(),
        hash.as_ref(),
        proposer.pubkey().as_ref(),
    ]);
    let proposer_rep_pda = get_pda(&[REPUTATION_SEED.as_ref(), proposer.pubkey().as_ref()]);
    propose_answer(&mut svm, proposer, hash, true);

    let q: Question = get_account_data(&svm, &question_pda);
    assert_eq!(
        q.state,
        QuestionState::Answering,
        "state should be Answering after proposal"
    );
    assert_eq!(q.total_yes_weight, MIN_STAKE as u128, "total yes weight should be 5");
    assert_eq!(q.total_no_weight, 0, "total no weight should be 0");
    assert_eq!(q.total_yes_stake, MIN_STAKE as u128, "total yes stake should be 5");
    assert_eq!(q.total_no_stake, 0, "total no stake should be 0");
    assert_eq!(q.yes_count, 1, "yes count should be 1");
    assert_eq!(q.no_count, 0, "no count should be 0");

    let a: Answer = get_account_data(&svm, &answer_pda);
    assert_eq!(a.answerer, proposer.pubkey(), "answerer should be proposer");
    assert!(a.side, "side should be true");
    assert_eq!(a.stake, MIN_STAKE, "stake should be 5");
    assert_eq!(a.weight, MIN_STAKE as u128, "weight should be 5 for new account");
    assert!(!a.claimed, "should not be claimed yet");

    let rep: Reputation = get_account_data(&svm, &proposer_rep_pda);
    assert_eq!(
        rep.voter,
        proposer.pubkey(),
        "reputation voter should be proposer"
    );
    assert_eq!(rep.score, 0, "initial score should be 0");
    assert_eq!(rep.answered, 0, "initial answered should be 0");

    assert_eq!(
        token_balance(&svm, &proposer_ata),
        10_000_000 - MIN_STAKE as u64,
        "proposer should have 5_000_000 after stake"
    );

    // ===================== Close Proposer =====================
    wrap_unix(&mut svm, &mut clock, 3610);
    let outcome_pda = get_pda(&[OUTCOME_SEED.as_ref(), hash.as_ref()]);
    let cranker1 = get_new_account(&mut svm);
    close_proposer(&mut svm, cranker1, hash);

    let q: Question = get_account_data(&svm, &question_pda);
    assert_eq!(
        q.state,
        QuestionState::Resolved,
        "state should be Resolved after close"
    );
    assert!(
        q.answer_deadline <= initial_time + 3610,
        "answer deadline should be in the past"
    );

    let o: Outcome = get_account_data(&svm, &outcome_pda);
    assert!(o.result, "result should be true (yes won)");
    assert_eq!(
        o.resolver,
        Resolver::Optimistic,
        "should be optimistic resolution"
    );
    assert_eq!(o.total_yes_weight, MIN_STAKE as u128, "yes weight should be 5");
    assert_eq!(o.total_no_weight, 0, "no weight should be 0");
    assert_eq!(o.answer_count, 1, "1 answer");
    assert_eq!(o.council_yes, 0, "no council votes yet");
    assert_eq!(o.council_no, 0, "no council votes yet");

    // ===================== Dispute =====================
    let disputer = get_new_account(&mut svm);
    let disputer_pk = disputer.pubkey();
    let dispute_pda = get_pda(&[DISPUTE_SEED.as_bytes(), hash.as_ref()]);
    let disputer_rep_pda = get_pda(&[REPUTATION_SEED.as_ref(), disputer_pk.as_ref()]);
    dispute(&mut svm, &disputer, hash);

    let q: Question = get_account_data(&svm, &question_pda);
    assert_eq!(
        q.state,
        QuestionState::Escalated,
        "state should be Escalated after dispute"
    );
    assert!(q.escalated, "should be escalated");
    assert!(q.has_dispute, "should have dispute flag");

    let dc: DisputeConfig = get_account_data(&svm, &dispute_pda);
    assert_eq!(dc.disputer, disputer_pk, "disputer should match");
    assert_eq!(dc.bond_amount, MIN_DISPUTE_BOND, "bond should be 5");
    assert!(
        !dc.claimed_outcome,
        "claimed_outcome should be false (opposite of outcome)"
    );
    assert!(!dc.resolved, "not resolved yet");
    assert!(!dc.claimed, "not claimed yet");

    // ===================== Council Vote =====================
    let council_total_pda = get_pda(&[COUNCIL_TOTAL_SEED.as_bytes(), hash.as_ref()]);
    council_vote(&mut svm, hash, members);

    let ct: CouncilTotal = get_account_data(&svm, &council_total_pda);
    assert_eq!(ct.yes_count, 8, "8 council members voted yes");
    assert_eq!(ct.no_count, 1, "1 council member voted no");
    assert_eq!(
        ct.opened_at,
        initial_time + 3610,
        "council should have been opened at close_proposer time"
    );
    assert_eq!(
        ct.total_yes_stake,
        (8 * MIN_STAKE) as u128,
        "total yes stake should be 8 * 5 = 40"
    );
    assert_eq!(ct.total_no_stake, MIN_STAKE as u128, "total no stake should be 5");
    assert!(ct.finalized.is_none(), "should not be finalized yet");

    let q: Question = get_account_data(&svm, &question_pda);
    assert_eq!(
        q.state,
        QuestionState::Council,
        "state should be Council after voting"
    );

    // ===================== Finalize Council =====================
    wrap_unix(&mut svm, &mut clock, SECONDS_PER_DAY + 1);
    let cranker2 = get_new_account(&mut svm);
    finalize_council(&mut svm, hash, cranker2);

    let ct: CouncilTotal = get_account_data(&svm, &council_total_pda);
    assert_eq!(
        ct.finalized,
        Some(true),
        "council should be finalized with verdict true"
    );
    assert_eq!(ct.yes_count, 8, "yes count unchanged after finalize");
    assert_eq!(ct.no_count, 1, "no count unchanged after finalize");

    let q: Question = get_account_data(&svm, &question_pda);
    assert_eq!(
        q.state,
        QuestionState::Resolved,
        "state should be Resolved after council finalize"
    );

    let o: Outcome = get_account_data(&svm, &outcome_pda);
    assert!(
        o.result,
        "outcome result should be true (council majority yes)"
    );
    assert_eq!(o.resolver, Resolver::Council, "resolver should be Council");
    assert_eq!(o.council_yes, 8, "council yes count should be 8");
    assert_eq!(o.council_no, 1, "council no count should be 1");

    // ===================== Settle Question =====================
    let pool_ata = ata(question_pda, USDC_PUBKEY);
    let treasury_ata = ata(treasury_pubkey, USDC_PUBKEY);
    let cranker3 = get_new_account(&mut svm);

    add_ata(&mut svm, treasury_pubkey, ONE_SOL);
    let pre_treasury = token_balance(&svm, &treasury_ata);
    let pre_pool = token_balance(&svm, &pool_ata);
    settle_question(&mut svm, hash, &cranker3, treasury_pubkey, true, true);

    let q: Question = get_account_data(&svm, &question_pda);
    assert_eq!(
        q.state,
        QuestionState::Settled,
        "state should be Settled after settle"
    );

    // gross = bounty(70) + council_slash(MIN_STAKE) ≈ 5_000_070
    // treasury 10% = 500_007, council 15% = 750_008 (/8 = 93_751), answer pool = rest
    let expected_treasury_cut: u64 = 500_007;
    let expected_per_answer_reward: u64 = 3_750_053;
    let expected_council_reward: u64 = 93_751;
    assert_eq!(
        q.per_answer_reward, expected_per_answer_reward,
        "per_answer_reward should be answer_pool / 1 answerer"
    );
    assert_eq!(
        q.council_reward_per_vote, expected_council_reward,
        "council_reward_per_vote should be council_pool / 8 correct voters"
    );

    assert_eq!(
        token_balance(&svm, &treasury_ata),
        pre_treasury + expected_treasury_cut,
        "treasury should have received treasury_cut"
    );
    assert_eq!(
        token_balance(&svm, &pool_ata),
        pre_pool - expected_treasury_cut,
        "pool should decrease by treasury_cut"
    );

    let dc: DisputeConfig = get_account_data(&svm, &dispute_pda);
    assert!(
        !dc.resolved,
        "dispute should be resolved=false (disputer was wrong)"
    );
    assert!(!dc.claimed, "dispute not yet claimed");

    // ===================== Claim Reward (Proposer) =====================
    let pre_proposer_balance = token_balance(&svm, &proposer_ata);
    let a_pre: Answer = get_account_data(&svm, &answer_pda);
    assert!(
        !a_pre.claimed,
        "answer should not be claimed yet before claim"
    );
    claim_question(&mut svm, hash, proposer, false);

    let expected_payout = MIN_STAKE + expected_per_answer_reward;
    assert_eq!(
        token_balance(&svm, &proposer_ata),
        pre_proposer_balance + expected_payout,
        "proposer should get stake + reward"
    );

    let rep: Reputation = get_account_data(&svm, &proposer_rep_pda);
    assert_eq!(rep.score, 10, "proposer rep score should be 10 (GAIN)");
    assert_eq!(rep.answered, 1, "proposer answered count should be 1");
    assert_eq!(rep.correct, 1, "proposer correct count should be 1");
    assert_eq!(rep.times_slashed, 0, "proposer should not be slashed");
    assert_eq!(rep.total_slashed, 0, "proposer total slashed should be 0");
    assert!(
        rep.active_days >= 1,
        "proposer should have at least 1 active day"
    );
    assert!(
        !account_exists(&svm, &answer_pda),
        "answer account should be closed after claim"
    );

    // ===================== Claim Dispute (Disputer lost) =====================
    let dc_pre: DisputeConfig = get_account_data(&svm, &dispute_pda);
    assert!(!dc_pre.claimed, "dispute should not be claimed yet");
    assert!(
        !dc_pre.resolved,
        "dispute should be unresolved (disputer was wrong)"
    );

    let disputer_ata = ata(disputer_pk, USDC_PUBKEY);
    let pre_disputer_balance = token_balance(&svm, &disputer_ata);
    claim_dispute(&mut svm, hash, &disputer);

    assert_eq!(
        token_balance(&svm, &disputer_ata),
        pre_disputer_balance,
        "disputer balance should be unchanged (lost, no payout)"
    );

    let rep: Reputation = get_account_data(&svm, &disputer_rep_pda);
    assert_eq!(rep.disputes_filed, 1, "disputes filed should be 1");
    assert_eq!(rep.disputes_lost, 1, "disputes lost should be 1");
    assert_eq!(rep.disputes_won, 0, "disputes won should be 0");

    assert!(
        !account_exists(&svm, &dispute_pda),
        "dispute account should be closed after claim"
    );

    // ===================== Close Question =====================
    wrap_unix(&mut svm, &mut clock, SECONDS_PER_DAY + 1);
    let remaining_pool = token_balance(&svm, &pool_ata);
    let pre_treasury_close = token_balance(&svm, &treasury_ata);

    close(&mut svm, hash, &cranker3, &asker, treasury_pubkey);

    assert_eq!(
        token_balance(&svm, &treasury_ata),
        pre_treasury_close + remaining_pool,
        "remaining pool should be swept to treasury"
    );
    assert!(
        !account_exists(&svm, &pool_ata),
        "pool ATA should be closed"
    );
    assert!(
        !account_exists(&svm, &question_pda),
        "question account should be closed"
    );
    assert!(
        !account_exists(&svm, &outcome_pda),
        "outcome account should be closed"
    );

    Ok(())
}
