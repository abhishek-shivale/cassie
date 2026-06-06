use crate::{ACCURACY_MAX_MULT, LOYALTY_MAX_MULT, MAX_DAYS, MAX_SCORE, SCALE};

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
}
