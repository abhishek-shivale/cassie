#![allow(dead_code)]
use crate::helper::ask::{ask, bounty_ata, question_pda, setup_with_questioner, AskParams};
use crate::helper::initialize::config_pda;
use crate::helper::utils::{ata, pda, send_ix, set_token_account, ONE_SOL};
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use cassie::constants::{ANSWER_SEED, MIN_STAKE, REPUTATION_SEED, USDC_PUBKEY};
use litesvm::types::TransactionResult;
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;
use spl_associated_token_account_interface::program::ID as ATA_PROGRAM_ID;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

pub const STAKE: u64 = MIN_STAKE;
pub const PROPOSER_FUNDS: u64 = 1_000_000;

pub struct ProposeParams {
    pub hash: [u8; 32],
    pub stake: u64,
    pub side: bool,
}

impl Default for ProposeParams {
    fn default() -> Self {
        Self {
            hash: AskParams::default().hash,
            stake: STAKE,
            side: true,
        }
    }
}

// initialized config + an asked question + a funded proposer (distinct from the asker).
// returns the admin/asker too, so freeze-path tests can flip the config.
pub fn setup_with_question() -> (LiteSVM, Keypair, Keypair, [u8; 32]) {
    let (mut svm, asker) = setup_with_questioner();
    let ask_params = AskParams::default();
    ask(&mut svm, &asker, &ask_params).unwrap();

    let proposer = fund_proposer(&mut svm);
    (svm, asker, proposer, ask_params.hash)
}

// fresh keypair with SOL for rent/fees and a funded USDC ATA.
pub fn fund_proposer(svm: &mut LiteSVM) -> Keypair {
    let proposer = Keypair::new();
    svm.airdrop(&proposer.pubkey(), ONE_SOL).unwrap();
    set_token_account(svm, proposer.pubkey(), USDC_PUBKEY, PROPOSER_FUNDS);
    proposer
}

pub fn answer_pda(hash: &[u8; 32], proposer: Pubkey) -> Pubkey {
    pda(&[ANSWER_SEED.as_ref(), hash.as_ref(), proposer.as_ref()]).0
}

pub fn reputation_pda(proposer: Pubkey) -> Pubkey {
    pda(&[REPUTATION_SEED.as_ref(), proposer.as_ref()]).0
}

pub fn propose_ix(proposer: Pubkey, params: &ProposeParams) -> Instruction {
    let data = cassie::instruction::Propose {
        hash: params.hash,
        stake: params.stake,
        side: params.side,
    }
    .data();

    let accounts = cassie::accounts::Propose {
        proposer,
        question: question_pda(&params.hash),
        config: config_pda(),
        usdc_mint: USDC_PUBKEY,
        proposer_ata: ata(proposer, USDC_PUBKEY),
        bond_ata: bounty_ata(&params.hash),
        reputation: reputation_pda(proposer),
        answer: answer_pda(&params.hash, proposer),
        token_program: TOKEN_PROGRAM_ID,
        system_program: system_program::ID,
        associated_token_program: ATA_PROGRAM_ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: cassie::id(),
        accounts,
        data,
    }
}

pub fn propose(svm: &mut LiteSVM, proposer: &Keypair, params: &ProposeParams) -> TransactionResult {
    let ix = propose_ix(proposer.pubkey(), params);
    send_ix(svm, ix, proposer, &[proposer])
}
