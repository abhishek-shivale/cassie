use crate::dep::{MEMO_PROGRAM_ID, ONE_SOL};
use crate::{account_data, add_ata, ata, get_pda, send_ix, SLASH_BPS, TREASURY_BPS};
use anchor_lang::prelude::{system_program, AccountMeta, Pubkey};
use anchor_lang::{InstructionData, ToAccountMetas};
use cassie::{
    CouncilTotal, OracleConfig, ADMIN_CONFIG_SEED, ANSWER_SEED, COUNCIL_TOTAL_SEED,
    COUNCIL_VOTE_SEED, DISPUTE_SEED, MIN_DISPUTE_BOND, MIN_STAKE, OUTCOME_SEED,
    QUESTION_CONFIG_SEED, REPUTATION_SEED, USDC_PUBKEY,
};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_transaction::Instruction;
use spl_associated_token_account_interface::program::ID as ATA_PROGRAM_ID;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

pub fn init_ix(accounts: Vec<AccountMeta>, data: Vec<u8>) -> Instruction {
    Instruction {
        program_id: cassie::id(),
        accounts,
        data,
    }
}

pub fn initialize_config(
    svm: &mut LiteSVM,
    council: [Pubkey; 9],
    treasury_pubkey: Pubkey,
    owner: &Keypair,
) {
    let init_data = cassie::instruction::InitializeConfig {
        council,
        council_size: 9,
        default_council_window: cassie::constants::SECONDS_PER_DAY,
        slash_bps: SLASH_BPS,
        treasury_bps: TREASURY_BPS,
        treasury: treasury_pubkey,
        default_answer_window: 3600,
        default_dispute_window: 7200,
        min_bounty: 10,
        divergence_bps: 3500,
        council_bps: 1500,
    }
    .data();

    let init_accounts = cassie::accounts::InitializeConfig {
        usdc_mint: USDC_PUBKEY,
        token_program: TOKEN_PROGRAM_ID,
        config: get_pda(&[ADMIN_CONFIG_SEED.as_ref()]),
        system_program: system_program::ID,
        admin: owner.pubkey(),
    }
    .to_account_metas(None);

    let initialize_config = init_ix(init_accounts, init_data);

    let res = send_ix(svm, initialize_config, &owner, &[&owner]);

    assert!(
        res.is_ok(),
        "InitializeConfig should be ok {:?}.",
        res.err()
    );
}

pub fn update_config(svm: &mut LiteSVM, owner: &Keypair) {
    let data = cassie::instruction::UpdateConfig {
        default_dispute_window: Some(7250i64),
        default_answer_window: None,
        default_council_window: None,
        freeze: None,
    }
    .data();

    let accounts = cassie::accounts::UpdateConfig {
        admin: owner.pubkey(),
        config: get_pda(&[ADMIN_CONFIG_SEED.as_ref()]),
    }
    .to_account_metas(None);

    let ix = init_ix(accounts, data);
    let res = send_ix(svm, ix, &owner, &[&owner]);
    assert!(res.is_ok(), "UpdateConfig should be ok {:?}.", res.err());
}

pub fn update_council(svm: &mut LiteSVM, owner: &Keypair, new_member: Pubkey, old_member: Pubkey) {
    let data = cassie::instruction::UpdateCouncil {
        new: new_member,
        old: old_member,
    }
    .data();

    let accounts = cassie::accounts::UpdateCouncil {
        config: get_pda(&[ADMIN_CONFIG_SEED.as_ref()]),
        admin: owner.pubkey(),
    }
    .to_account_metas(None);

    let ix = init_ix(accounts, data);
    let res = send_ix(svm, ix, &owner, &[&owner]);
    assert!(res.is_ok(), "UpdateCouncil should be ok {:?}.", res.err());
}

pub fn ask_ix(svm: &mut LiteSVM, asker: &Keypair, hash: [u8; 32]) {
    let data = cassie::instruction::Ask {
        hash,
        bounty: 70,
        callback_discriminator: [0u8; 8],
        callback_program: MEMO_PROGRAM_ID,
        metadata_uri: [0u8; 128],
        category: "boxing".as_bytes()[0],
    }
    .data();

    let accounts = cassie::accounts::Ask {
        questioner: asker.pubkey(),
        config: get_pda(&[ADMIN_CONFIG_SEED.as_ref()]),
        question: get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
        questioner_ata: add_ata(svm, asker.pubkey(), 70000),
        usdc_mint: USDC_PUBKEY,
        token_program: TOKEN_PROGRAM_ID,
        associated_token_program: ATA_PROGRAM_ID,
        system_program: system_program::id(),
        bounty_ata: ata(
            get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
            USDC_PUBKEY,
        ),
    }
    .to_account_metas(None);
    let ix = init_ix(accounts, data);
    let res = send_ix(svm, ix, &asker, &[&asker]);
    assert!(res.is_ok(), "ask should be ok {:?}.", res.err());
}

pub fn propose_answer(svm: &mut LiteSVM, proposer: &Keypair, hash: [u8; 32], side: bool) {
    let data = cassie::instruction::Propose {
        hash,
        stake: MIN_STAKE,
        side,
    }
    .data();

    let accounts = cassie::accounts::Propose {
        proposer: proposer.pubkey(),
        proposer_ata: add_ata(svm, proposer.pubkey(), 10_000_000),
        answer: get_pda(&[
            ANSWER_SEED.as_ref(),
            hash.as_ref(),
            proposer.pubkey().as_ref(),
        ]),
        config: get_pda(&[ADMIN_CONFIG_SEED.as_ref()]),
        question: get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
        usdc_mint: USDC_PUBKEY,
        bond_ata: ata(
            get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
            USDC_PUBKEY,
        ),
        reputation: get_pda(&[REPUTATION_SEED.as_ref(), proposer.pubkey().as_ref()]),
        token_program: TOKEN_PROGRAM_ID,
        associated_token_program: ATA_PROGRAM_ID,
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    let ix = init_ix(accounts, data);
    let res = send_ix(svm, ix, &proposer, &[&proposer]);
    assert!(res.is_ok(), "propose should be ok {:?}.", res.err());
}

pub fn dispute(svm: &mut LiteSVM, dispute: &Keypair, hash: [u8; 32]) {
    let accounts = cassie::accounts::Dispute {
        disputer: dispute.pubkey(),
        reputation: get_pda(&[REPUTATION_SEED.as_ref(), dispute.pubkey().as_ref()]),
        disputer_config: get_pda(&[DISPUTE_SEED.as_bytes(), hash.as_ref()]),
        disputer_ata: add_ata(svm, dispute.pubkey(), 10_000_000),
        bond_ata: ata(
            get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
            USDC_PUBKEY,
        ),
        config: get_pda(&[ADMIN_CONFIG_SEED.as_ref()]),
        usdc_mint: USDC_PUBKEY,
        outcome: get_pda(&[OUTCOME_SEED.as_ref(), hash.as_ref()]),
        question: get_pda(&[QUESTION_CONFIG_SEED.as_bytes(), hash.as_ref()]),
        token_program: TOKEN_PROGRAM_ID,
        associated_token_program: ATA_PROGRAM_ID,
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    let data = cassie::instruction::Dispute {
        hash,
        bond: MIN_DISPUTE_BOND,
        claimed_outcome: false,
        reason_hash: [0u8; 128],
    }
    .data();

    let ix = init_ix(accounts, data);
    let res = send_ix(svm, ix, &dispute, &[&dispute]);
    assert!(res.is_ok(), "dispute should be ok {:?}.", res.err());
}

pub fn close_proposer(svm: &mut LiteSVM, cranker: Keypair, hash: [u8; 32]) {
    let data = cassie::instruction::CloseProposers { hash }.data();
    let accounts = cassie::accounts::CloseProposer {
        question: get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
        config: get_pda(&[ADMIN_CONFIG_SEED.as_ref()]),
        cranker: cranker.pubkey(),
        outcome: get_pda(&[OUTCOME_SEED.as_ref(), hash.as_ref()]),
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    let ix = init_ix(accounts, data);
    let res = send_ix(svm, ix, &cranker, &[&cranker]);
    assert!(res.is_ok(), "close proposer should be ok {:?}.", res.err());
}

pub fn council_vote(svm: &mut LiteSVM, hash: [u8; 32], council_mem: [Keypair; 9]) {
    let c = council_mem;
    let zero = &c[0];
    vote(svm, hash, zero, false);
    for n in 1..9 {
        vote(svm, hash, &c[n], true);
    }
}

fn vote(svm: &mut LiteSVM, hash: [u8; 32], members: &Keypair, vote: bool) {
    let accounts = cassie::accounts::Vote {
        question: get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
        config: get_pda(&[ADMIN_CONFIG_SEED.as_ref()]),
        reputation: get_pda(&[REPUTATION_SEED.as_ref(), members.pubkey().as_ref()]),
        council_vote: get_pda(&[
            COUNCIL_VOTE_SEED.as_ref(),
            hash.as_ref(),
            members.pubkey().as_ref(),
        ]),
        reward_pool: ata(
            get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
            USDC_PUBKEY,
        ),
        council_total: get_pda(&[COUNCIL_TOTAL_SEED.as_bytes(), hash.as_ref()]),
        usdc_mint: USDC_PUBKEY,
        voter: members.pubkey(),
        voter_ata: add_ata(svm, members.pubkey(), 10_000_000),
        system_program: system_program::id(),
        token_program: TOKEN_PROGRAM_ID,
        associated_token_program: ATA_PROGRAM_ID,
    }
    .to_account_metas(None);

    let data = cassie::instruction::CouncilVote {
        hash,
        bond: MIN_STAKE,
        vote,
    }
    .data();

    let ix = init_ix(accounts, data);
    let res = send_ix(svm, ix, &members, &[&members]);
    assert!(res.is_ok(), "council vote should be ok {:?}.", res.err());
}

pub fn finalize_council(svm: &mut LiteSVM, hash: [u8; 32], cranker: Keypair) {
    let data = cassie::instruction::FinalizeCouncil { hash }.data();

    account_data::<CouncilTotal>(
        svm,
        get_pda(&[COUNCIL_TOTAL_SEED.as_bytes(), hash.as_ref()]),
    );

    account_data::<OracleConfig>(svm, get_pda(&[ADMIN_CONFIG_SEED.as_ref()]));

    let account = cassie::accounts::Finalize {
        council_total: get_pda(&[COUNCIL_TOTAL_SEED.as_bytes(), hash.as_ref()]),
        question: get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
        config: get_pda(&[ADMIN_CONFIG_SEED.as_ref()]),
        cranker: cranker.pubkey(),
        outcome: get_pda(&[OUTCOME_SEED.as_ref(), hash.as_ref()]),
    }
    .to_account_metas(None);

    let ix = init_ix(account, data);
    let res = send_ix(svm, ix, &cranker, &[&cranker]);
    assert!(
        res.is_ok(),
        "finalize council should be ok {:?}.",
        res.err()
    );
}

pub fn settle_question(
    svm: &mut LiteSVM,
    hash: [u8; 32],
    cranker: &Keypair,
    treasury_pubkey: Pubkey,
    has_dispute: bool,
    is_council: bool,
) {
    let data = cassie::instruction::SettleQuestion { hash }.data();

    let dispute = if has_dispute {
        Some(get_pda(&[DISPUTE_SEED.as_bytes(), hash.as_ref()]))
    } else {
        None
    };

    let council_total = if is_council {
        Some(get_pda(&[COUNCIL_TOTAL_SEED.as_bytes(), hash.as_ref()]))
    } else {
        None
    };

    let account = cassie::accounts::Settle {
        question: get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
        config: get_pda(&[ADMIN_CONFIG_SEED.as_ref()]),
        cranker: cranker.pubkey(),
        outcome: get_pda(&[OUTCOME_SEED.as_ref(), hash.as_ref()]),
        usdc_mint: USDC_PUBKEY,
        pool_ata: ata(
            get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
            USDC_PUBKEY,
        ),
        treasury_ata: add_ata(svm, treasury_pubkey, ONE_SOL),
        council_total,
        dispute,
        token_program: TOKEN_PROGRAM_ID,
        callback_program: Some(MEMO_PROGRAM_ID),
    }
    .to_account_metas(None);

    let ix = init_ix(account, data);
    let res = send_ix(svm, ix, &cranker, &[&cranker]);
    assert!(res.is_ok(), "settle question should be ok {:?}.", res.err());
}

pub fn claim_question(svm: &mut LiteSVM, hash: [u8; 32], claimer: &Keypair, is_disputer: bool) {
    let data = cassie::instruction::ClaimReward { hash }.data();

    let dispute = if is_disputer {
        Some(get_pda(&[DISPUTE_SEED.as_bytes(), hash.as_ref()]))
    } else {
        None
    };

    let account = cassie::accounts::ClaimReward {
        question: get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
        config: get_pda(&[ADMIN_CONFIG_SEED.as_ref()]),
        claimer: claimer.pubkey(),
        outcome: get_pda(&[OUTCOME_SEED.as_ref(), hash.as_ref()]),
        usdc_mint: USDC_PUBKEY,
        pool_ata: ata(
            get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
            USDC_PUBKEY,
        ),
        claimer_ata: ata(claimer.pubkey(), USDC_PUBKEY),
        dispute,
        token_program: TOKEN_PROGRAM_ID,
        reputation: get_pda(&[REPUTATION_SEED.as_ref(), claimer.pubkey().as_ref()]),
        council_vote: Option::from(None),
        answer: Some(get_pda(&[
            ANSWER_SEED.as_ref(),
            hash.as_ref(),
            claimer.pubkey().as_ref(),
        ])),
    }
    .to_account_metas(None);

    let ix = init_ix(account, data);
    let res = send_ix(svm, ix, &claimer, &[&claimer]);
    assert!(res.is_ok(), "claim question should be ok {:?}.", res.err());
}

pub fn claim_dispute(svm: &mut LiteSVM, hash: [u8; 32], claimer: &Keypair) {
    let data = cassie::instruction::ClaimReward { hash }.data();

    let account = cassie::accounts::ClaimReward {
        question: get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
        config: get_pda(&[ADMIN_CONFIG_SEED.as_ref()]),
        claimer: claimer.pubkey(),
        outcome: get_pda(&[OUTCOME_SEED.as_ref(), hash.as_ref()]),
        usdc_mint: USDC_PUBKEY,
        pool_ata: ata(
            get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
            USDC_PUBKEY,
        ),
        claimer_ata: ata(claimer.pubkey(), USDC_PUBKEY),
        dispute: Some(get_pda(&[DISPUTE_SEED.as_bytes(), hash.as_ref()])),
        token_program: TOKEN_PROGRAM_ID,
        reputation: get_pda(&[REPUTATION_SEED.as_ref(), claimer.pubkey().as_ref()]),
        council_vote: Option::from(None),
        answer: None,
    }
    .to_account_metas(None);

    let ix = init_ix(account, data);
    let res = send_ix(svm, ix, &claimer, &[&claimer]);
    assert!(res.is_ok(), "claim dispute should be ok {:?}.", res.err());
}

pub fn close(
    svm: &mut LiteSVM,
    hash: [u8; 32],
    cranker: &Keypair,
    questioner: &Keypair,
    treasury_pubkey: Pubkey,
) {
    let data = cassie::instruction::CloseQuestion { hash }.data();
    let treasury_ata_addr = ata(treasury_pubkey, USDC_PUBKEY);
    if svm.get_account(&treasury_ata_addr).is_none() {
        add_ata(svm, treasury_pubkey, ONE_SOL);
    }
    let account = cassie::accounts::CloseQuestion {
        question: get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
        config: get_pda(&[ADMIN_CONFIG_SEED.as_ref()]),
        cranker: cranker.pubkey(),
        creator: questioner.pubkey(),
        council_total: Some(get_pda(&[COUNCIL_TOTAL_SEED.as_bytes(), hash.as_ref()])),
        outcome: get_pda(&[OUTCOME_SEED.as_ref(), hash.as_ref()]),
        treasury_ata: treasury_ata_addr,
        usdc_mint: USDC_PUBKEY,
        pool_ata: ata(
            get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]),
            USDC_PUBKEY,
        ),
        token_program: TOKEN_PROGRAM_ID,
    }
    .to_account_metas(None);

    let ix = init_ix(account, data);
    let res = send_ix(svm, ix, &cranker, &[&cranker]);
    assert!(res.is_ok(), "close question should be ok {:?}.", res.err());
}
