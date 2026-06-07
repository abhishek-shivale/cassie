mod helper;

use anchor_lang::prelude::Pubkey;
use anchor_lang::AccountDeserialize;
use cassie::state::admin::OracleConfig;
use helper::initialize::config_pda;
use helper::update_council::{setup_initialized, update_council};
use helper::utils::ONE_SOL;
use solana_keypair::Keypair;
use solana_signer::Signer;

// decode the on-chain config PDA.
fn read_config(svm: &litesvm::LiteSVM) -> OracleConfig {
    let raw = svm.get_account(&config_pda()).unwrap();
    OracleConfig::try_deserialize(&mut raw.data.as_slice()).unwrap()
}

// happy path: an existing member is swapped out for a fresh key.
#[test]
fn update_council_replace_ok() {
    let (mut svm, admin, council) = setup_initialized();
    let old = council[0];
    let new = Pubkey::new_unique();

    assert!(update_council(&mut svm, &admin, old, new).is_ok());

    let cfg = read_config(&svm);
    assert!(cfg.council.contains(&new));
    assert!(!cfg.council.contains(&old));
}

// `old` not present in the council must trip NotCouncilMember.
#[test]
fn update_council_old_not_member() {
    let (mut svm, admin, _council) = setup_initialized();
    let old = Pubkey::new_unique();
    let new = Pubkey::new_unique();

    assert!(update_council(&mut svm, &admin, old, new).is_err());
}

// `new` equal to the zero pubkey must trip CouncilMemberShouldNotBeZero.
#[test]
fn update_council_new_is_zero() {
    let (mut svm, admin, council) = setup_initialized();
    assert!(update_council(&mut svm, &admin, council[0], Pubkey::default()).is_err());
}

// `old` equal to the zero pubkey must trip CouncilMemberShouldNotBeZero.
#[test]
fn update_council_old_is_zero() {
    let (mut svm, admin, _council) = setup_initialized();
    let new = Pubkey::new_unique();
    assert!(update_council(&mut svm, &admin, Pubkey::default(), new).is_err());
}

// `new` already seated must trip DuplicateCouncilMember.
#[test]
fn update_council_new_already_member() {
    let (mut svm, admin, council) = setup_initialized();
    // council[1] is already a member, reusing it as `new` is a duplicate.
    assert!(update_council(&mut svm, &admin, council[0], council[1]).is_err());
}

// a signer that is not the stored admin must trip UnauthorizedAdmin (has_one).
#[test]
fn update_council_unauthorized_admin() {
    let (mut svm, _admin, council) = setup_initialized();
    let intruder = Keypair::new();
    svm.airdrop(&intruder.pubkey(), ONE_SOL).unwrap();

    let new = Pubkey::new_unique();
    assert!(update_council(&mut svm, &intruder, council[0], new).is_err());
}
