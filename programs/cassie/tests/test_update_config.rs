mod helper;

use anchor_lang::AccountDeserialize;
use cassie::state::admin::OracleConfig;
use helper::initialize::config_pda;
use helper::update_config::{setup_initialized, update_config, UpdateConfigParams};
use helper::utils::ONE_SOL;
use solana_keypair::Keypair;
use solana_signer::Signer;

// decode the on-chain config PDA.
fn read_config(svm: &litesvm::LiteSVM) -> OracleConfig {
    let raw = svm.get_account(&config_pda()).unwrap();
    OracleConfig::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

// happy path: every field at/above its floor is persisted.
#[test]
fn update_config_all_fields_ok() {
    let (mut svm, admin) = setup_initialized();
    let params = UpdateConfigParams {
        default_dispute_window: Some(8000),
        default_council_window: Some(90000),
        default_answer_window: Some(4000),
        freeze: Some(true),
    };

    assert!(update_config(&mut svm, &admin, &params).is_ok());

    let cfg = read_config(&svm);
    assert_eq!(cfg.default_dispute_window, 8000);
    assert_eq!(cfg.default_council_window, 90000);
    assert_eq!(cfg.default_answer_window, 4000);
    assert!(cfg.freeze);
}

// None fields are left untouched; only freeze flips here.
#[test]
fn update_config_partial_only_freeze() {
    let (mut svm, admin) = setup_initialized();
    let before = read_config(&svm);

    let params = UpdateConfigParams {
        freeze: Some(true),
        ..Default::default()
    };
    assert!(update_config(&mut svm, &admin, &params).is_ok());

    let after = read_config(&svm);
    assert!(after.freeze);
    // untouched windows keep their initialized values.
    assert_eq!(after.default_dispute_window, before.default_dispute_window);
    assert_eq!(after.default_council_window, before.default_council_window);
    assert_eq!(after.default_answer_window, before.default_answer_window);
}

// dispute window below 7200s floor must trip InvalidWindow.
#[test]
fn update_config_dispute_window_too_small() {
    let (mut svm, admin) = setup_initialized();
    let params = UpdateConfigParams {
        default_dispute_window: Some(7199),
        ..Default::default()
    };
    assert!(update_config(&mut svm, &admin, &params).is_err());
}

// council window below 86400s floor must trip InvalidWindow.
#[test]
fn update_config_council_window_too_small() {
    let (mut svm, admin) = setup_initialized();
    let params = UpdateConfigParams {
        default_council_window: Some(86399),
        ..Default::default()
    };
    assert!(update_config(&mut svm, &admin, &params).is_err());
}

// answer window below 3600s floor must trip InvalidWindow.
#[test]
fn update_config_answer_window_too_small() {
    let (mut svm, admin) = setup_initialized();
    let params = UpdateConfigParams {
        default_answer_window: Some(3599),
        ..Default::default()
    };
    assert!(update_config(&mut svm, &admin, &params).is_err());
}

// a signer that is not the stored admin must trip UnauthorizedAdmin (has_one).
#[test]
fn update_config_unauthorized_admin() {
    let (mut svm, _admin) = setup_initialized();
    let intruder = Keypair::new();
    svm.airdrop(&intruder.pubkey(), ONE_SOL).unwrap();

    let params = UpdateConfigParams {
        freeze: Some(true),
        ..Default::default()
    };
    assert!(update_config(&mut svm, &intruder, &params).is_err());
}
