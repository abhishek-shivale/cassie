use crate::aggregation::{
    apply_answer_reputation, apply_council_reputation, apply_dispute_reputation, compute_payout,
    RepUpdate,
};
use crate::constants::*;
use crate::error::CassieError;
use crate::{
    Answer, CouncilVote, DisputeConfig, OracleConfig, Outcome, Question, QuestionState, Reputation,
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
#[instruction(hash: [u8; 32])]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub claimer: Signer<'info>,

    #[account(
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
        seeds = [OUTCOME_SEED.as_bytes(), hash.as_ref()],
        bump = outcome.bump,
    )]
    pub outcome: Box<Account<'info, Outcome>>,

    #[account(
        mut,
        seeds = [ANSWER_SEED.as_bytes(), hash.as_ref(), claimer.key().as_ref()],
        bump = answer.bump,
    )]
    pub answer: Option<Box<Account<'info, Answer>>>,

    #[account(
        mut,
        seeds = [DISPUTE_SEED.as_bytes(), hash.as_ref()],
        bump = dispute.bump,
        constraint = dispute.disputer == claimer.key() @ CassieError::UnauthorizedAdmin,
    )]
    pub dispute: Option<Box<Account<'info, DisputeConfig>>>,

    #[account(
        mut,
        seeds = [COUNCIL_VOTE_SEED.as_bytes(), hash.as_ref(), claimer.key().as_ref()],
        bump = council_vote.bump,
    )]
    pub council_vote: Option<Account<'info, CouncilVote>>,

    #[account(
        mut,
        seeds = [REPUTATION_SEED.as_bytes(), claimer.key().as_ref()],
        bump = reputation.bump,
    )]
    pub reputation: Box<Account<'info, Reputation>>,

    #[account(address = USDC_PUBKEY)]
    pub usdc_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint      = usdc_mint,
        associated_token::authority = question,
    )]
    pub pool_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint      = usdc_mint,
        associated_token::authority = claimer,
    )]
    pub claimer_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> ClaimReward<'info> {
    pub fn claim(&mut self, hash: [u8; 32]) -> Result<()> {
        require!(!self.config.freeze, CassieError::ProgramFrozen);
        require!(
            matches!(self.question.state, QuestionState::Settled),
            CassieError::InvalidState
        );

        let result = self.outcome.result;
        let now = Clock::get()?.unix_timestamp;
        let mut ru = self.snapshot_rep();

        let mut total_payout: u64 = 0;
        let mut acted: bool = false;
        let mut dispute_won: Option<bool> = None;
        let mut council_correct: Option<bool> = None;

        if let Some(payout) = self.settle_answer(&mut ru, result, now) {
            total_payout = total_payout.saturating_add(payout);
            acted = true;
        }
        if let Some((payout, won)) = self.settle_dispute(&mut ru, now) {
            total_payout = total_payout.saturating_add(payout);
            dispute_won = Some(won);
            acted = true;
        }
        if let Some((payout, correct)) = self.settle_council(&mut ru, result, now) {
            total_payout = total_payout.saturating_add(payout);
            council_correct = Some(correct);
            acted = true;
        }

        require!(acted, CassieError::AlreadyClaimed);

        if total_payout > 0 {
            self.transfer_payout(hash, total_payout)?;
        }
        self.commit_reputation(ru, now, dispute_won, council_correct);
        self.close_personal_accounts()?;

        Ok(())
    }

    fn settle_answer(&mut self, ru: &mut RepUpdate, result: bool, now: i64) -> Option<u64> {
        let per_answer_reward = self.question.per_answer_reward;
        let slash_bps = self.config.slash_bps as u16;

        let answer = self.answer.as_mut()?;
        if answer.claimed {
            return None;
        }
        let correct = answer.side == result;
        // compute_payout: correct → stake + reward, wrong → stake * (1 - slash_bps)
        let payout = compute_payout(
            answer.side,
            answer.stake,
            result,
            per_answer_reward,
            slash_bps,
        );
        let slashed = answer.stake.saturating_sub(payout);
        answer.claimed = true;

        apply_answer_reputation(ru, correct, slashed, now);
        Some(payout)
    }

    fn settle_dispute(&mut self, ru: &mut RepUpdate, now: i64) -> Option<(u64, bool)> {
        let dispute = self.dispute.as_mut()?;
        if dispute.claimed {
            return None;
        }
        let won = dispute.resolved;
        let payout = if won { dispute.reward } else { 0 };
        dispute.claimed = true;

        apply_dispute_reputation(ru, won, now);
        Some((payout, won))
    }

    fn settle_council(
        &mut self,
        ru: &mut RepUpdate,
        result: bool,
        now: i64,
    ) -> Option<(u64, bool)> {
        let per_vote = self.question.council_reward_per_vote;
        let slash_bps = self.config.slash_bps as u128;
        let council_slash_bps = (slash_bps * 2).min(BPS_DENOMINATOR);

        let cv = self.council_vote.as_mut()?;
        if cv.claimed {
            return None;
        }
        let correct = cv.vote == result;
        apply_council_reputation(ru, correct, now);

        let payout = if correct {
            cv.stake.saturating_add(per_vote)
        } else {
            let returned_bps = BPS_DENOMINATOR.saturating_sub(council_slash_bps);
            ((cv.stake as u128) * returned_bps / BPS_DENOMINATOR) as u64
        };

        cv.claimed = true;
        Some((payout, correct))
    }

    fn transfer_payout(&self, hash: [u8; 32], amount: u64) -> Result<()> {
        let bump = [self.question.bump];
        let seeds: &[&[u8]] = &[QUESTION_CONFIG_SEED.as_bytes(), hash.as_ref(), &bump];
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.key(),
                TransferChecked {
                    from: self.pool_ata.to_account_info(),
                    to: self.claimer_ata.to_account_info(),
                    mint: self.usdc_mint.to_account_info(),
                    authority: self.question.to_account_info(),
                },
                &[seeds],
            ),
            amount,
            self.usdc_mint.decimals,
        )?;
        Ok(())
    }

    fn snapshot_rep(&self) -> RepUpdate {
        RepUpdate {
            score: self.reputation.score,
            answered: self.reputation.answered,
            correct: self.reputation.correct,
            active_days: self.reputation.active_days,
            last_answer_day: self.reputation.last_answer_day,
            times_slashed: self.reputation.times_slashed,
            total_slashed: self.reputation.total_slashed,
        }
    }

    fn commit_reputation(
        &mut self,
        ru: RepUpdate,
        now: i64,
        dispute_won: Option<bool>,
        council_correct: Option<bool>,
    ) {
        let rep = &mut self.reputation;
        rep.score = ru.score;
        rep.answered = ru.answered;
        rep.correct = ru.correct;
        rep.active_days = ru.active_days;
        rep.last_answer_day = ru.last_answer_day;
        rep.times_slashed = ru.times_slashed;
        rep.total_slashed = ru.total_slashed;

        if let Some(won) = dispute_won {
            rep.disputes_filed = rep.disputes_filed.saturating_add(1);
            if won {
                rep.disputes_won = rep.disputes_won.saturating_add(1);
            } else {
                rep.disputes_lost = rep.disputes_lost.saturating_add(1);
            }
        }
        if let Some(correct) = council_correct {
            rep.council_votes = rep.council_votes.saturating_add(1);
            if correct {
                rep.council_correct = rep.council_correct.saturating_add(1);
            }
        }
        rep.last_updated = now;
    }

    fn close_personal_accounts(&self) -> Result<()> {
        let dest = self.claimer.to_account_info();
        if let Some(answer) = self.answer.as_ref() {
            answer.close(dest.clone())?;
        }
        if let Some(dispute) = self.dispute.as_ref() {
            dispute.close(dest.clone())?;
        }
        if let Some(council_vote) = self.council_vote.as_ref() {
            council_vote.close(dest)?;
        }
        Ok(())
    }
}
