#![allow(dead_code)]
use crate::helper::ask::question_pda;
use crate::helper::initialize::config_pda;
use crate::helper::utils::{pda, send_ix, warp_unix};
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{system_program, AccountDeserialize, InstructionData, ToAccountMetas};
use cassie::constants::OUTCOME_SEED;
use cassie::state::question::Question;
use litesvm::types::TransactionResult;
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;

pub fn outcome_pda(hash: &[u8; 32]) -> Pubkey {
    pda(&[OUTCOME_SEED.as_ref(), hash.as_ref()]).0
}

// move the clock one second past the question's answer deadline.
pub fn warp_past_answer_window(svm: &mut LiteSVM, hash: &[u8; 32]) {
    let raw = svm.get_account(&question_pda(hash)).unwrap();
    let q = Question::try_deserialize(&mut raw.data.as_slice()).unwrap();
    warp_unix(svm, q.answer_deadline + 1);
}

pub fn close_ix(cranker: Pubkey, hash: &[u8; 32]) -> Instruction {
    let data = cassie::instruction::CloseProposers { _hash: *hash }.data();

    let accounts = cassie::accounts::CloseProposer {
        cranker,
        question: question_pda(hash),
        config: config_pda(),
        outcome: outcome_pda(hash),
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: cassie::id(),
        accounts,
        data,
    }
}

pub fn close(svm: &mut LiteSVM, cranker: &Keypair, hash: &[u8; 32]) -> TransactionResult {
    let ix = close_ix(cranker.pubkey(), hash);
    send_ix(svm, ix, cranker, &[cranker])
}
