use anchor_lang::prelude::*;

declare_id!("DANGHof54KqrvGnipP3Hm8whXXifmaWKQwYYH533jVaq");

#[program]
pub mod callback_example {
    use super::*;

    pub fn handle_cassie_result(ctx: Context<Callback>, hash: [u8; 32], result: bool) -> Result<()> {
        msg!(
            "Cassie callback received — question hash: {:?}, result: {}",
            hash,
            result
        );

        if result {
            msg!("The affirmative side (YES) won the question.");
        } else {
            msg!("The negative side (NO) won the question.");
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Callback<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
}
