use crate::constants::*;
use crate::error::CassieError;
use crate::{OracleConfig, Outcome, ProposersClosed, Question, QuestionState, Resolver};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(hash: [u8; 32])]
pub struct CloseProposer<'info> {
    #[account(mut)]
    pub cranker: Signer<'info>,

    #[account(
        mut,
        seeds = [QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()],
        bump = question.bump,
    )]
    pub question: Account<'info, Question>,

    #[account(
        seeds = [ADMIN_CONFIG_SEED.as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, OracleConfig>,

    // permanent on-chain record of the outcome. cranker fronts rent.
    #[account(
        init,
        payer = cranker,
        space = Outcome::DISCRIMINATOR.len() + Outcome::INIT_SPACE,
        seeds = [OUTCOME_SEED.as_ref(), hash.as_ref()],
        bump
    )]
    pub outcome: Account<'info, Outcome>,

    pub system_program: Program<'info, System>,
}

impl<'info> CloseProposer<'info> {
    pub fn close(&mut self, bumps: &CloseProposerBumps) -> Result<()> {
        require!(!self.config.freeze, CassieError::ProgramFrozen);

        let now = Clock::get()?.unix_timestamp;
        // answer window must be closed before aggregating
        require!(
            now >= self.question.answer_deadline,
            CassieError::AnswerWindowActive
        );
        // only aggregate a question still in the answering phase
        require!(
            matches!(
                self.question.state,
                QuestionState::Asked | QuestionState::Answering
            ),
            CassieError::InvalidState
        );

        let yes_w = self.question.total_yes_weight;
        let no_w = self.question.total_no_weight;
        let answer_count = self.question.yes_count.checked_add(self.question.no_count).unwrap();

        let result = crate::aggregation::resolve_or_escalate(
            yes_w,
            no_w,
            answer_count,
            self.config.divergence_bps as u16,
        );

        let (final_result, resolver) = match result {
            crate::aggregation::AggregationResult::Resolved { result, resolver } => {
                self.question.state = QuestionState::Resolved;
                // open the dispute window
                self.question.dispute_deadline = now + self.config.default_dispute_window;
                (result, resolver)
            }
            crate::aggregation::AggregationResult::Escalate { .. } => {
                self.question.state = QuestionState::Escalated;
                self.question.escalated = true;
                // tentative result; council overwrites at finalize
                (yes_w > no_w, Resolver::Council)
            }
        };

        self.outcome.set_inner(Outcome {
            hash: self.question.hash,
            result: final_result,
            resolver,
            total_yes_weight: yes_w,
            total_no_weight: no_w,
            answer_count,
            council_yes: 0,
            council_no: 0,
            settled_at: now,
            bump: bumps.outcome,
        });

        emit!(ProposersClosed {
            hash: self.question.hash,
            result: final_result,
            resolver,
            escalated: self.question.escalated,
            total_yes_weight: yes_w,
            total_no_weight: no_w,
            answer_count,
            settled_at: now,
        });

        Ok(())
    }
}
