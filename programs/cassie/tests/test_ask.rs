mod helper;

use anchor_lang::AccountDeserialize;
use cassie::constants::USDC_PUBKEY;
use cassie::state::question::{Question, QuestionState};
use helper::ask::{ask, bounty_ata, question_pda, setup_with_questioner, AskParams, BOUNTY};
use helper::update_config::{update_config, UpdateConfigParams};
use helper::utils::{ata, token_balance};
use solana_signer::Signer;

// decode a question PDA.
fn read_question(svm: &litesvm::LiteSVM, hash: &[u8; 32]) -> Question {
    let raw = svm.get_account(&question_pda(hash)).unwrap();
    Question::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

// happy path: question PDA is created, fields set, bounty moved into the pool.
#[test]
fn ask_ok() {
    let (mut svm, questioner) = setup_with_questioner();
    let params = AskParams::default();
    let funded = token_balance(&svm, ata(questioner.pubkey(), USDC_PUBKEY));

    assert!(ask(&mut svm, &questioner, &params).is_ok());

    let q = read_question(&svm, &params.hash);
    assert_eq!(q.creator, questioner.pubkey());
    assert_eq!(q.bounty, BOUNTY);
    assert_eq!(q.hash, params.hash);
    assert!(matches!(q.state, QuestionState::Asked));

    // bounty pool holds the bounty, questioner debited by the same.
    assert_eq!(token_balance(&svm, bounty_ata(&params.hash)), BOUNTY);
    assert_eq!(
        token_balance(&svm, ata(questioner.pubkey(), USDC_PUBKEY)),
        funded - BOUNTY
    );
}

// bounty below config.min_bounty must trip InsufficientBounty.
#[test]
fn ask_bounty_below_min() {
    let (mut svm, questioner) = setup_with_questioner();
    let params = AskParams {
        bounty: 9,
        ..Default::default()
    };
    assert!(ask(&mut svm, &questioner, &params).is_err());
}

// asking on a frozen program must trip ProgramFrozen.
#[test]
fn ask_when_frozen() {
    let (mut svm, questioner) = setup_with_questioner();
    // admin == questioner here, so it can flip the freeze flag.
    update_config(
        &mut svm,
        &questioner,
        &UpdateConfigParams {
            freeze: Some(true),
            ..Default::default()
        },
    )
    .unwrap();

    assert!(ask(&mut svm, &questioner, &AskParams::default()).is_err());
}

// reusing a hash collides with the existing question PDA (init) and fails.
#[test]
fn ask_duplicate_hash() {
    let (mut svm, questioner) = setup_with_questioner();
    let params = AskParams::default();

    assert!(ask(&mut svm, &questioner, &params).is_ok());
    assert!(ask(&mut svm, &questioner, &params).is_err());
}

// bounty exceeding the questioner's balance fails on the token transfer.
#[test]
fn ask_insufficient_funds() {
    let (mut svm, questioner) = setup_with_questioner();
    let params = AskParams {
        bounty: 2_000_000,
        ..Default::default()
    };
    assert!(ask(&mut svm, &questioner, &params).is_err());
}
