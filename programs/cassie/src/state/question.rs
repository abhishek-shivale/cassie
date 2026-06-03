use anchor_lang::prelude::*;


#[account]
#[derive(InitSpace)]
pub struct QuestionConfig {
    pub questioner: Pubkey,
    pub bounty: u64,
    #[max_len(32)]
    pub question: String,
    #[max_len(256)]
    pub description: String,
    #[max_len(256)]
    pub rules: String,
    #[max_len(16)]
    pub category: String,
    pub created_at: i64,
    pub bump: u8,
    pub nonce: u64,
}