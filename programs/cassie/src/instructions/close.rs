use crate::constants::*;
use crate::error::CassieError;
use crate::{CouncilTotal, OracleConfig, Outcome, Question, QuestionState};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
    TransferChecked,
};

#[derive(Accounts)]
#[instruction(hash: [u8; 32])]
pub struct CloseQuestion<'info> {
    #[account(mut)]
    pub cranker: Signer<'info>,

    // SOL-011: (1) token account drained before close (see handler drain + close_account below).
    //          (2) lamports from question + outcome go to creator (verified address).
    //          (3) pool_ata (token account) emptied via transfer_checked then close_account.
    /// CHECK:
    #[account(mut, address = question.creator)]
    pub creator: UncheckedAccount<'info>,

    #[account(
        mut,
        close = creator,
        seeds = [QUESTION_CONFIG_SEED.as_bytes(), hash.as_ref()],
        bump,
    )]
    pub question: Box<Account<'info, Question>>,

    #[account(
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump,
    )]
    pub config: Box<Account<'info, OracleConfig>>,

    #[account(
        mut,
        close = creator,
        seeds = [OUTCOME_SEED.as_bytes(), hash.as_ref()],
        bump,
    )]
    pub outcome: Box<Account<'info, Outcome>>,

    #[account(
        mut,
        seeds = [COUNCIL_TOTAL_SEED.as_bytes(), hash.as_ref()],
        bump,
    )]
    pub council_total: Option<Box<Account<'info, CouncilTotal>>>,

    #[account(address = USDC_PUBKEY)]
    pub usdc_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = question,
    )]
    pub pool_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = config.treasury,
    )]
    pub treasury_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> CloseQuestion<'info> {
    pub fn close(&mut self, hash: [u8; 32]) -> Result<()> {
        require!(
            matches!(self.question.state, QuestionState::Settled),
            CassieError::InvalidState
        );
        let now = Clock::get()?.unix_timestamp;
        require!(
            now > self.outcome.settled_at + CLOSE_GRACE,
            CassieError::CloseGraceActive
        );

        let bump = [self.question.bump];
        let seeds: &[&[u8]] = &[QUESTION_CONFIG_SEED.as_bytes(), hash.as_ref(), &bump];

        let remaining = self.pool_ata.amount;
        if remaining > 0 {
            transfer_checked(
                CpiContext::new_with_signer(
                    self.token_program.key(),
                    TransferChecked {
                        from: self.pool_ata.to_account_info(),
                        to: self.treasury_ata.to_account_info(),
                        mint: self.usdc_mint.to_account_info(),
                        authority: self.question.to_account_info(),
                    },
                    &[seeds],
                ),
                remaining,
                self.usdc_mint.decimals,
            )?;
        }

        close_account(CpiContext::new_with_signer(
            self.token_program.key(),
            CloseAccount {
                account: self.pool_ata.to_account_info(),
                destination: self.creator.to_account_info(),
                authority: self.question.to_account_info(),
            },
            &[seeds],
        ))?;

        Ok(())
    }
}
