use crate::aggregation::{
    apply_answer_reputation, apply_council_reputation, apply_dispute_reputation, compute_payout,
    RepUpdate,
};
use crate::constants::*;
use crate::error::CassieError;
use crate::{Answer, DisputeConfig, OracleConfig, Outcome, Question, QuestionState, Reputation};
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
        seeds = [QUESTION_CONFIG_SEED.as_ref(), hash.as_ref()],
        bump = question.bump,
    )]
    pub question: Account<'info, Question>,

    #[account(
        seeds = [ADMIN_CONFIG_SEED.as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, OracleConfig>,

    #[account(
        seeds = [OUTCOME_SEED.as_ref(), hash.as_ref()],
        bump = outcome.bump,
    )]
    pub outcome: Account<'info, Outcome>,

    #[account(
        mut,
        seeds = [ANSWER_SEED.as_ref(), hash.as_ref(), claimer.key().as_ref()],
        bump = answer.bump,
    )]
    pub answer: Option<Account<'info, Answer>>,
    
    #[account(
        mut,
        seeds = [DISPUTE_SEED.as_ref(), hash.as_ref()],
        bump = dispute.bump,
        constraint = dispute.disputer == claimer.key() @ CassieError::UnauthorizedAdmin,
    )]
    pub dispute: Option<Account<'info, DisputeConfig>>,

    #[account(
        mut,
        seeds = [REPUTATION_SEED.as_ref(), claimer.key().as_ref()],
        bump = reputation.bump,
    )]
    pub reputation: Account<'info, Reputation>,

    #[account(
        address = USDC_PUBKEY
    )]
    pub usdc_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = question,
    )]
    pub pool_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = claimer,
    )]
    pub claimer_ata: InterfaceAccount<'info, TokenAccount>,

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
        let mut acted = false;
        let mut dispute_won: Option<bool> = None;

        if let Some(payout) = self.settle_answer(&mut ru, result, now) {
            total_payout = total_payout.saturating_add(payout);
            acted = true;
        }
        if let Some((payout, won)) = self.settle_dispute(&mut ru, now) {
            total_payout = total_payout.saturating_add(payout);
            dispute_won = Some(won);
            acted = true;
        }

        require!(acted, CassieError::AlreadyClaimed);

        if total_payout > 0 {
            self.transfer_payout(hash, total_payout)?;
        }
        self.commit_reputation(ru, now, dispute_won);

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

    fn settle_answer(&mut self, ru: &mut RepUpdate, result: bool, now: i64) -> Option<u64> {
        let is_council = self.config.council.contains(&self.claimer.key());
        let per_answer_reward = self.question.per_answer_reward;
        let slash_bps = self.config.slash_bps as u16;

        let answer = self.answer.as_mut()?;
        if answer.claimed {
            return None;
        }
        let correct = answer.side == result;
        let payout = compute_payout(
            answer.side,
            answer.stake,
            result,
            per_answer_reward,
            slash_bps,
        );
        let slashed = answer.stake.saturating_sub(payout);
        answer.claimed = true;

        if is_council {
            apply_council_reputation(ru, correct, now);
        } else {
            apply_answer_reputation(ru, correct, slashed, now);
        }
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

    fn transfer_payout(&self, hash: [u8; 32], amount: u64) -> Result<()> {
        let bump = [self.question.bump];
        let seeds: &[&[u8]] = &[QUESTION_CONFIG_SEED.as_ref(), hash.as_ref(), &bump];
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

    fn commit_reputation(&mut self, ru: RepUpdate, now: i64, dispute_won: Option<bool>) {
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
        rep.last_updated = now;
    }
}
