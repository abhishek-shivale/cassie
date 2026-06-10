mod dep;
use dep::*;
use anchor_lang::prelude::Clock;

use anchor_lang::InstructionData;
use anchor_lang::prelude::*;
use solana_keypair::Keypair;
use solana_signer::Signer;
use cassie::{QUESTION_CONFIG_SEED, SECONDS_PER_DAY, USDC_PUBKEY};
#[test]
fn test_happy_path() -> Result<()> {
    let (mut svm, owner) = setup();
    let owner_pubkey = owner.pubkey();
    let treasury_pubkey = get_new_account(&mut svm).pubkey();

    // council members
    let mut members = council_members(&mut svm);
    let mut clock = svm.get_sysvar::<Clock>();


    // mint usdc
    mint_token(&mut svm, USDC_PUBKEY, owner_pubkey);
    let council: [Pubkey; 9] = members.each_ref().map(|m| m.pubkey());

    // init config instruction
    initialize_config(&mut svm, council, treasury_pubkey, &owner);

    // update config
    update_config(&mut svm, &owner);

    let new_council_member = get_new_account(&mut svm);

    // update council
    update_council(&mut svm, &owner, new_council_member.pubkey(), council[0]);

    // ask question
    let asker = get_new_account(&mut svm);
    let hash = [0u8; 32];
    ask_ix(&mut svm, asker, hash);

    // propose answer
    let proposer = get_new_account(&mut svm);
    let proposer1 = get_new_account(&mut svm);
    propose_answer(&mut svm, proposer, hash, true);
    // propose_answer(&mut svm, proposer1, hash, false);


    // close proposer
    wrap_unix(&mut svm, &mut clock, 3600 + 10);
    let cranker = get_new_account(&mut svm);
    close_proposer(&mut svm, cranker, hash);
    // you can see data from here


    // dispute
    let disputer = get_new_account(&mut svm);
    dispute(&mut svm, disputer, hash);
    // account_data(&mut svm, get_pda(&[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()]));

    //council vote
    members[0] = new_council_member;
    council_vote(&mut svm, hash, members);

    // finalize council
    wrap_unix(&mut svm, &mut clock, SECONDS_PER_DAY + 1);
    let cranker = get_new_account(&mut svm);
    finalize_council(&mut svm, hash, cranker);

    Ok(())
}