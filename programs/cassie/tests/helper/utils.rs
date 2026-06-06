#![allow(dead_code)]
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
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

pub const ONE_SOL: u64 = 1_000_000_000;

pub fn program_id() -> Pubkey {
    cassie::id()
}

pub fn setup_svm() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();
    let bytes = include_bytes!("../../../../target/deploy/cassie.so");
    svm.add_program(cassie::id(), bytes).unwrap();
    svm.airdrop(&payer.pubkey(), ONE_SOL).unwrap();
    (svm, payer)
}

pub fn pda(seeds: &[&[u8]]) -> (Pubkey, u8) {
    Pubkey::find_program_address(seeds, &cassie::id())
}

pub fn set_mint(svm: &mut LiteSVM, address: Pubkey, authority: Pubkey, decimals: u8) {
    let mint = Mint {
        mint_authority: COption::Some(authority),
        supply: 0,
        decimals,
        is_initialized: true,
        freeze_authority: COption::None,
    };
    let mut data = vec![0u8; Mint::LEN];
    Mint::pack(mint, &mut data).unwrap();
    svm.set_account(
        address,
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

pub fn set_token_account(svm: &mut LiteSVM, owner: Pubkey, mint: Pubkey, amount: u64) -> Pubkey {
    let address = ata(owner, mint);
    let token = SplTokenAccount {
        mint,
        owner,
        amount,
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
            data,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
    address
}

pub fn token_balance(svm: &LiteSVM, address: Pubkey) -> u64 {
    let raw = svm.get_account(&address).unwrap();
    SplTokenAccount::unpack(&raw.data).unwrap().amount
}

pub fn send_ix(
    svm: &mut LiteSVM,
    ix: Instruction,
    payer: &Keypair,
    signers: &[&Keypair],
) -> litesvm::types::TransactionResult {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}
