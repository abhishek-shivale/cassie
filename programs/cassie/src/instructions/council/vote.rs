use crate::constants::*;
use crate::error::CassieError;
use crate::{
    CouncilTotal, CouncilVote, CouncilVoted, OracleConfig, Question, QuestionState, Reputation,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(hash: [u8; 32])]
pub struct Vote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account(
        mut,
        seeds = [QUESTION_CONFIG_SEED.as_bytes(), hash.as_ref()],
        bump = question.bump,
    )]
    pub question: Box<Account<'info, Question>>,

    #[account(
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, OracleConfig>>,

    #[account(
        init_if_needed,
        payer = voter,
        space = CouncilTotal::DISCRIMINATOR.len() + CouncilTotal::INIT_SPACE,
        seeds = [COUNCIL_TOTAL_SEED.as_bytes(), hash.as_ref()],
        bump
    )]
    pub council_total: Box<Account<'info, CouncilTotal>>,

    #[account(
        init,
        payer = voter,
        space = CouncilVote::DISCRIMINATOR.len() + CouncilVote::INIT_SPACE,
        seeds = [COUNCIL_VOTE_SEED.as_bytes(), hash.as_ref(), voter.key().as_ref()],
        bump
    )]
    pub council_vote: Box<Account<'info, CouncilVote>>,

    #[account(
        init_if_needed,
        payer = voter,
        space = Reputation::DISCRIMINATOR.len() + Reputation::INIT_SPACE,
        seeds = [REPUTATION_SEED.as_bytes(), voter.key().as_ref()],
        bump
    )]
    pub reputation: Box<Account<'info, Reputation>>,

    pub system_program: Program<'info, System>,
}

impl<'info> Vote<'info> {
    pub fn vote(&mut self, vote: bool, bumps: &VoteBumps) -> Result<()> {
        require!(!self.config.freeze, CassieError::ProgramFrozen);

        // caller must be a council member
        require!(
            self.config.council.contains(&self.voter.key()),
            CassieError::NotCouncilMember
        );

        // only an escalated question (or one already in council) can be voted on
        require!(
            matches!(
                self.question.state,
                QuestionState::Escalated | QuestionState::Council
            ),
            CassieError::InvalidState
        );

        let now = Clock::get()?.unix_timestamp;

        // first vote opens the council round + transitions state
        if self.council_total.opened_at == 0 {
            self.council_total.opened_at = now;
            self.council_total.bump = bumps.council_total;
            self.question.state = QuestionState::Council;
        }

        // council window must still be open
        require!(
            now <= self.council_total.opened_at + self.config.default_council_window,
            CassieError::CouncilWindowClosed
        );

        // tally
        if vote {
            self.council_total.yes_count = self.council_total.yes_count.checked_add(1).unwrap();
        } else {
            self.council_total.no_count = self.council_total.no_count.checked_add(1).unwrap();
        }

        self.council_vote.set_inner(CouncilVote {
            member: self.voter.key(),
            vote,
            voted_at: now,
            bump: bumps.council_vote,
        });

        // ensure the voter has a reputation account for the eventual claim.
        if self.reputation.voter == Pubkey::default() {
            self.reputation.voter = self.voter.key();
            self.reputation.bump = bumps.reputation;
        }
        self.reputation.is_council = true;

        emit!(CouncilVoted {
            hash: self.question.hash,
            member: self.voter.key(),
            vote,
            yes_count: self.council_total.yes_count,
            no_count: self.council_total.no_count,
        });

        Ok(())
    }
}
