use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct OracleConfig  {
    pub bump: u8,
    pub authority: Pubkey,
    pub usdc_mint: Pubkey,
    pub slash_rate: u16, // 50 percent
    pub default_dispute_period: u16, // in sec
    pub default_answer_period: u16,
    pub default_council_resolve_period: u16,
    pub council: [Pubkey; 3],  
    pub quorum: u8,          
}