mod helper;

use anchor_lang::AccountDeserialize;
use cassie::state::admin::OracleConfig;
use helper::initialize::{config_pda, init_config, InitParams, COUNCIL_SIZE};
use helper::utils::setup_svm;
use solana_signer::Signer;

// happy path: valid params create the config PDA and persist every field.
#[test]
fn initialize_config_ok() {
    let (mut svm, admin) = setup_svm();
    let params = InitParams::default();

    let res = init_config(&mut svm, &admin, &params);
    assert!(res.is_ok(), "init should succeed: {:?}", res.err());

    // read back the PDA and decode the OracleConfig account.
    let raw = svm.get_account(&config_pda()).expect("config not created");
    let config = OracleConfig::try_deserialize(&mut raw.data.as_slice()).unwrap();

    assert_eq!(config.admin, admin.pubkey());
    assert_eq!(config.council_size, COUNCIL_SIZE);
    // quorum is derived as floor(size * 2 / 3).
    assert_eq!(config.quorum, (COUNCIL_SIZE * 2) / 3);
    assert!(!config.freeze);
}

// divergence_bps above 100% (10_000) must trip MaxBpsReached.
#[test]
fn initialize_config_divergence_bps_too_high() {
    let (mut svm, admin) = setup_svm();
    let params = InitParams {
        divergence_bps: 10_001,
        ..Default::default()
    };

    assert!(init_config(&mut svm, &admin, &params).is_err());
}

// dispute window below the 7200s floor must trip InvalidWindow.
#[test]
fn initialize_config_dispute_window_too_small() {
    let (mut svm, admin) = setup_svm();
    let params = InitParams {
        default_dispute_window: 7199,
        ..Default::default()
    };

    assert!(init_config(&mut svm, &admin, &params).is_err());
}

// council_size of zero must trip CouncilMemberShouldNotBeZero.
#[test]
fn initialize_config_council_size_zero() {
    let (mut svm, admin) = setup_svm();
    let params = InitParams {
        council_size: 0,
        ..Default::default()
    };

    assert!(init_config(&mut svm, &admin, &params).is_err());
}

// min_bounty below 10 must trip BountySizeCanNotBeLower.
#[test]
fn initialize_config_bounty_too_low() {
    let (mut svm, admin) = setup_svm();
    let params = InitParams {
        min_bounty: 9,
        ..Default::default()
    };

    assert!(init_config(&mut svm, &admin, &params).is_err());
}

// initializing twice on the same PDA must fail (account already exists).
#[test]
fn initialize_config_twice_fails() {
    let (mut svm, admin) = setup_svm();
    let params = InitParams::default();

    assert!(init_config(&mut svm, &admin, &params).is_ok());
    assert!(init_config(&mut svm, &admin, &params).is_err());
}
