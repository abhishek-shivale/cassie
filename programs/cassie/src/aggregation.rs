use crate::{
    EscalationReason, Resolver, ACCURACY_MAX_MULT, BPS_DENOMINATOR, COUNCIL_GAIN, COUNCIL_LOSS,
    DISPUTE_LOSS, DISPUTE_WIN_GAIN, GAIN, LOSS, LOYALTY_MAX_MULT, MAX_DAYS, MAX_SCORE, SCALE,
    SECONDS_PER_DAY,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AggregationResult {
    /// Resolved cleanly. `result` is the winning side, `resolver` how it was decided.
    Resolved { result: bool, resolver: Resolver },
    /// Must escalate to the council. `reason` says why.
    Escalate { reason: EscalationReason },
}

// calculate weight for user for answering the question
pub fn compute_weight(stake: u64, score: u64, active_days: u32) -> u128 {
    let score = score.min(MAX_SCORE);
    let days = active_days.min(MAX_DAYS) as u64;

    let accuracy_mult = (score * ACCURACY_MAX_MULT) / MAX_SCORE;
    let loyalty_mult = (days * LOYALTY_MAX_MULT) / (MAX_DAYS as u64);
    (stake as u128)
        * (SCALE as u128 + accuracy_mult as u128)
        * (SCALE as u128 + loyalty_mult as u128)
        / ((SCALE as u128) * (SCALE as u128))
}

// resolve or escalate function helps to make sure there is no divergence from other side
// if yes then it would resolve or escalate based on result
pub fn resolve_or_escalate(
    yes_weight: u128,
    no_weight: u128,
    answer_count: u32,
    divergence_bps: u16,
) -> AggregationResult {
    if answer_count == 0 {
        return AggregationResult::Escalate {
            reason: EscalationReason::NoAnswer,
        };
    };

    if yes_weight == 0 || no_weight == 0 {
        return AggregationResult::Resolved {
            result: yes_weight > no_weight,
            resolver: Resolver::Optimistic,
        };
    };

    let total = yes_weight + no_weight;
    let minority = yes_weight.min(no_weight);

    let minority_bps = (minority * BPS_DENOMINATOR) / total;

    if minority_bps >= divergence_bps as u128 {
        // Too contested to call.
        AggregationResult::Escalate {
            reason: EscalationReason::Divergence,
        }
    } else {
        AggregationResult::Resolved {
            result: yes_weight > no_weight,
            resolver: Resolver::Weighted,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RewardSplit {
    pub total: u64,             // distributable pool (bounty + slash - treasury)
    pub treasury_cut: u64,      // protocol cut
    pub per_answer_reward: u64, // equal share for each correct answerer (v1)
    pub slash_amount: u64,      // total slashed from losers
}

pub fn compute_reward_split(
    bounty: u64,
    loser_total_stake: u64,
    correct_count: u32,
    slash_bps: u16,
    treasury_bps: u16,
) -> RewardSplit {
    let slash_amount = ((loser_total_stake as u128) * (slash_bps as u128) / BPS_DENOMINATOR) as u64;

    let gross = bounty.saturating_add(slash_amount);
    let treasury_cut = ((gross as u128) * (treasury_bps as u128) / BPS_DENOMINATOR) as u64;
    let distributable = gross.saturating_sub(treasury_cut);

    let per_answer_reward = if correct_count == 0 {
        0
    } else {
        distributable / (correct_count as u64)
    };

    RewardSplit {
        total: distributable,
        treasury_cut,
        per_answer_reward,
        slash_amount,
    }
}

pub fn compute_payout(
    answer_side: bool,
    answer_stake: u64,
    outcome: bool,
    per_answer_reward: u64,
    slash_bps: u16,
) -> u64 {
    if answer_side == outcome {
        answer_stake.saturating_add(per_answer_reward)
    } else {
        let returned_bps = BPS_DENOMINATOR - (slash_bps as u128);
        ((answer_stake as u128) * returned_bps / BPS_DENOMINATOR) as u64
    }
}

pub struct RepUpdate {
    pub score: u64,
    pub answered: u32,
    pub correct: u32,
    pub active_days: u32,
    pub last_answer_day: i64,
    pub times_slashed: u32,
    pub total_slashed: u64,
}

pub fn apply_answer_reputation(rep: &mut RepUpdate, correct: bool, slashed_amount: u64, now: i64) {
    if correct {
        rep.score = (rep.score + GAIN).min(MAX_SCORE);
        rep.correct = rep.correct.saturating_add(1);
    } else {
        rep.score = rep.score.saturating_sub(LOSS);
        if slashed_amount > 0 {
            rep.times_slashed = rep.times_slashed.saturating_add(1);
            rep.total_slashed = rep.total_slashed.saturating_add(slashed_amount);
        }
    }

    bump_loyalty(rep, now);
    rep.answered = rep.answered.saturating_add(1);
}

pub fn apply_council_reputation(rep: &mut RepUpdate, voted_with_verdict: bool, now: i64) {
    if voted_with_verdict {
        rep.score = (rep.score + COUNCIL_GAIN).min(MAX_SCORE);
    } else {
        rep.score = rep.score.saturating_sub(COUNCIL_LOSS);
    }
    bump_loyalty(rep, now);
}

fn bump_loyalty(rep: &mut RepUpdate, now: i64) {
    let today = now / SECONDS_PER_DAY;
    if today > rep.last_answer_day {
        rep.active_days = (rep.active_days + 1).min(MAX_DAYS);
        rep.last_answer_day = today;
    }
}

pub fn apply_dispute_reputation(rep: &mut RepUpdate, won: bool, now: i64) {
    if won {
        rep.score = (rep.score + DISPUTE_WIN_GAIN).min(MAX_SCORE);
    } else {
        rep.score = rep.score.saturating_sub(DISPUTE_LOSS);
    }
    bump_loyalty(rep, now);
}

#[cfg(test)]
mod test {
    use super::*;

    // normal test for the fn compute weight to
    // goal - correct the good path
    #[test]
    fn test_compute_weight() {
        let weight = compute_weight(750, 10, 10);
        assert_eq!(weight, 866);
    }

    // goal to test that veteran should win over fresh wallet
    // test is not valid for the v1, but we are ready for v2
    #[test]
    fn test_calculate_weight_compare_veteran_vs_fresh() {
        let whale = compute_weight(1000, 0, 1); // big stake, no rep
        let veteran = compute_weight(100, MAX_SCORE, MAX_DAYS);
        assert!(veteran > whale);
    }

    #[test]
    fn newcomer_weight_equals_stake() {
        assert_eq!(compute_weight(750, 0, 0), 750);
    }

    #[test]
    fn maxed_veteran_weight() {
        // (100 + 500) * (100 + 200) / (100*100) = 600*300/10000 = 18x
        assert_eq!(compute_weight(100, MAX_SCORE, MAX_DAYS), 1800);
    }

    #[test]
    fn unanimous_resolves_optimistic() {
        let r = resolve_or_escalate(500, 0, 3, 3500);
        assert_eq!(
            r,
            AggregationResult::Resolved {
                result: true,
                resolver: Resolver::Optimistic
            }
        );
    }

    #[test]
    fn lopsided_resolves_weighted() {
        // 92/8 split, minority 8% < 35%
        let r = resolve_or_escalate(920, 80, 10, 3500);
        assert_eq!(
            r,
            AggregationResult::Resolved {
                result: true,
                resolver: Resolver::Weighted
            }
        );
    }

    #[test]
    fn contested_escalates() {
        // 60/40 split, minority 40% >= 35%
        let r = resolve_or_escalate(600, 400, 10, 3500);
        assert_eq!(
            r,
            AggregationResult::Escalate {
                reason: EscalationReason::Divergence
            }
        );
    }

    #[test]
    fn no_answers_escalates() {
        let r = resolve_or_escalate(0, 0, 0, 3500);
        assert_eq!(
            r,
            AggregationResult::Escalate {
                reason: EscalationReason::NoAnswer
            }
        );
    }

    #[test]
    fn reward_split_equal() {
        // bounty 1000, losers staked 400 total, 50% slash, 5% treasury, 4 correct
        // slash = 200, gross = 1200, treasury = 60, distributable = 1140, per = 285
        let s = compute_reward_split(1000, 400, 4, 5000, 500);
        assert_eq!(s.slash_amount, 200);
        assert_eq!(s.treasury_cut, 60);
        assert_eq!(s.total, 1140);
        assert_eq!(s.per_answer_reward, 285);
    }

    #[test]
    fn payout_correct_vs_wrong() {
        // correct: stake 100 + reward 285 = 385
        assert_eq!(compute_payout(true, 100, true, 285, 5000), 385);
        // wrong: 50% of stake back = 50
        assert_eq!(compute_payout(false, 100, true, 285, 5000), 50);
    }

    #[test]
    fn loyalty_bumps_once_per_day() {
        let mut rep = RepUpdate {
            score: 0,
            answered: 0,
            correct: 0,
            active_days: 0,
            last_answer_day: 0,
            times_slashed: 0,
            total_slashed: 0,
        };
        let day1 = 5 * SECONDS_PER_DAY;
        apply_answer_reputation(&mut rep, true, 0, day1);
        apply_answer_reputation(&mut rep, true, 0, day1 + 100); // same day
        assert_eq!(rep.active_days, 1);
        assert_eq!(rep.score, GAIN * 2);

        let day2 = 6 * SECONDS_PER_DAY;
        apply_answer_reputation(&mut rep, true, 0, day2);
        assert_eq!(rep.active_days, 2);
    }
}
