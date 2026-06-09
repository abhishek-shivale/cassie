use crate::constants::*;
use crate::error::CassieError;
use crate::{
    CouncilFinalized, CouncilTotal, OracleConfig, Outcome, Question, QuestionState, Resolver,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(hash: [u8; 32])]
pub struct Finalize<'info> {
    // permissionless crank. anyone can finalize once quorum is reached.
    #[account(mut)]
    pub cranker: Signer<'info>,

    #[account(
        mut,
        seeds = [QUESTION_CONFIG_SEED.as_bytes(), hash.as_ref()],
        bump = question.bump,
    )]
    pub question: Account<'info, Question>,

    #[account(
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump = config.bump,
    )]
    pub config: Account<'info, OracleConfig>,

    #[account(
        mut,
        seeds = [COUNCIL_TOTAL_SEED.as_bytes(), hash.as_ref()],
        bump = council_total.bump,
    )]
    pub council_total: Account<'info, CouncilTotal>,

    #[account(
        mut,
        seeds = [OUTCOME_SEED.as_bytes(), hash.as_ref()],
        bump = outcome.bump,
    )]
    pub outcome: Account<'info, Outcome>,
}

impl<'info> Finalize<'info> {
    pub fn finalize(&mut self) -> Result<()> {
        require!(!self.config.freeze, CassieError::ProgramFrozen);

        // must be mid-council
        require!(
            matches!(self.question.state, QuestionState::Council),
            CassieError::InvalidState
        );

        // not already finalized
        require!(
            self.council_total.finalized_at.is_none(),
            CassieError::InvalidState
        );

        let now = Clock::get()?.unix_timestamp;

        let deadline =
            self.council_total.opened_at + self.config.default_council_window;

        msg!("now: {}", now);
        msg!("opened_at: {}", self.council_total.opened_at);
        msg!("window: {}", self.config.default_council_window);
        msg!("deadline: {}", deadline);

        require!(now >= deadline, CassieError::CouncilWindowActive);

        // quorum: enough members must have voted
        let total_votes = self
            .council_total
            .yes_count
            .checked_add(self.council_total.no_count)
            .unwrap();
        require!(
            total_votes >= self.config.quorum,
            CassieError::QuorumNotReached
        );

        // majority verdict (tie -> no)
        let verdict = self.council_total.yes_count > self.council_total.no_count;
        let now = Clock::get()?.unix_timestamp;

        // council is final
        self.council_total.finalized_at = Some(verdict);
        self.question.state = QuestionState::Resolved;

        // overwrite outcome with the council verdict
        self.outcome.result = verdict;
        self.outcome.resolver = Resolver::Council;
        self.outcome.council_yes = self.council_total.yes_count;
        self.outcome.council_no = self.council_total.no_count;
        self.outcome.settled_at = now;

        emit!(CouncilFinalized {
            hash: self.question.hash,
            result: verdict,
            council_yes: self.council_total.yes_count,
            council_no: self.council_total.no_count,
        });

        Ok(())
    }
}
