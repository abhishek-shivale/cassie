#![allow(dead_code)]

use anchor_lang::prelude::{AccountInfo, Clock, Pubkey};
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::Accounts;
use cassie::{CouncilTotal, Question, USDC_PUBKEY};
use litesvm::LiteSVM;
use solana_account::Account;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_program_option::COption;
use solana_program_pack::Pack;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use spl_associated_token_account_interface::address::get_associated_token_address;
use spl_token_interface::{
    state::{Account as SplTokenAccount, AccountState, Mint},
    ID as TOKEN_PROGRAM_ID,
};
use std::fmt::Debug;

pub fn program_id() -> Pubkey {
    cassie::ID
}

pub const ONE_SOL: u64 = 1_000_000_000;
pub const SLASH_BPS: u64 = 5_000;

pub const TREASURY_BPS: u64 = 1_000;

pub fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();
    let bytes = include_bytes!("../../../../target/deploy/cassie.so");
    svm.add_program(cassie::id(), bytes).unwrap();
    svm.airdrop(&payer.pubkey(), ONE_SOL).unwrap();
    (svm, payer)
}

pub fn council_members(svm: &mut LiteSVM) -> [Keypair; 9] {
    std::array::from_fn(|_| {
        let keypair = Keypair::new();
        svm.airdrop(&keypair.pubkey(), ONE_SOL).unwrap();
        keypair
    })
}

pub fn mint_token(svm: &mut LiteSVM, pubkey: Pubkey, authority: Pubkey) {
    let mint = Mint {
        mint_authority: COption::Some(authority),
        supply: 0,
        decimals: 2,
        is_initialized: true,
        freeze_authority: COption::None,
    };
    let mut data = vec![0u8; Mint::LEN];
    Mint::pack(mint, &mut data).unwrap();
    svm.set_account(
        pubkey,
        Account {
            lamports: ONE_SOL,
            data,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
}

pub fn ata(owner: Pubkey, mint: Pubkey) -> Pubkey {
    get_associated_token_address(&owner, &mint)
}

pub fn add_ata(svm: &mut LiteSVM, owner: Pubkey, amount: u64) -> Pubkey {
    let address = get_associated_token_address(&owner, &USDC_PUBKEY);

    let token = SplTokenAccount {
        mint: USDC_PUBKEY,
        amount,
        owner,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };

    let mut data = vec![0u8; SplTokenAccount::LEN];
    SplTokenAccount::pack(token, &mut data).unwrap();
    svm.set_account(
        address,
        Account {
            lamports: ONE_SOL,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
            data,
        },
    )
    .unwrap();

    address
}

pub fn send_ix(
    svm: &mut LiteSVM,
    ix: Instruction,
    payer: &Keypair,
    signers: &[&Keypair],
) -> litesvm::types::TransactionResult {
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        signers,
        svm.latest_blockhash(),
    );
    let result = svm.send_transaction(tx);
    let logs  = match &result {
        Ok(x) => {&x.logs},
        Err(e) => {
            &e.meta.logs
        }
    };

    for log in logs {
        println!("Logs: -> {}", log)
    }

    result
}

pub fn wrap_unix(svm: &mut LiteSVM, clock: &mut Clock, unix_timestamp: i64) {
    clock.unix_timestamp = clock.unix_timestamp + unix_timestamp;
    svm.set_sysvar(clock)
}

pub fn token_balance(svm: &LiteSVM, address: &Pubkey) -> u64 {
    let raw = svm.get_account(address).unwrap();
    SplTokenAccount::unpack(&raw.data).unwrap().amount
}

pub fn get_new_account(svm: &mut LiteSVM) -> Keypair {
    let key_pair = Keypair::new();
    svm.airdrop(&key_pair.pubkey(), ONE_SOL).unwrap();
    key_pair
}

pub fn get_pda(seed: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seed, &cassie::id()).0
}

use anchor_lang::AccountDeserialize;
use solana_transaction::Transaction;

pub fn account_data<T>(svm: &mut LiteSVM, acc: Pubkey)
where
    T: AccountDeserialize + Debug,
{
    let account = svm.get_account(&acc).unwrap();

    let data = T::try_deserialize(&mut account.data.as_slice()).unwrap();

    println!("account data: {:?}", data);
}
