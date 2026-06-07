#![allow(dead_code)]
use crate::helper::ask::question_pda;
use crate::helper::close::outcome_pda;
use crate::helper::council_vote::council_total_pda;
use crate::helper::initialize::config_pda;
use crate::helper::utils::send_ix;
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::types::TransactionResult;
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;

pub fn finalize_ix(cranker: Pubkey, hash: &[u8; 32]) -> Instruction {
    let data = cassie::instruction::FinalizeCouncil { hash: *hash }.data();

    let accounts = cassie::accounts::Finalize {
        cranker,
        question: question_pda(hash),
        config: config_pda(),
        council_total: council_total_pda(hash),
        outcome: outcome_pda(hash),
    }
    .to_account_metas(None);

    Instruction {
        program_id: cassie::id(),
        accounts,
        data,
    }
}

pub fn finalize(svm: &mut LiteSVM, cranker: &Keypair, hash: &[u8; 32]) -> TransactionResult {
    let ix = finalize_ix(cranker.pubkey(), hash);
    send_ix(svm, ix, cranker, &[cranker])
}
