//! The AI plan's quotas (TS §52, v6.2implementation D-461, D-494): signed-in
//! people only, counted in `ai_usage` per UTC month and only on success.
//! There is no anonymous quota: the route needs a bearer.
//!
//! Two requests in flight at the last plan can both pass the check, so a
//! month can end one or two over: accepted, since `PlanUser` allows ten a
//! minute and each counted plan was a real answer.

use offcut_types::Plan;

/// Free: plans a month.
pub const FREE_PLANS_PER_MONTH: i32 = 40;
/// Creator: above this many, each success logs one line (soft).
pub const CREATOR_PLANS_SOFT: i32 = 1_000;
/// Creator: at this many, the month's plans are used up.
pub const CREATOR_PLANS_HARD: i32 = 3_000;

/// The count at which `plan` is refused.
fn limit(plan: Plan) -> i32 {
    match plan {
        Plan::Free => FREE_PLANS_PER_MONTH,
        Plan::Creator => CREATOR_PLANS_HARD,
    }
}

/// Whether one more plan is allowed after `used` this month.
pub fn allows(plan: Plan, used: i32) -> bool {
    used < limit(plan)
}

/// What the response tells the person after their `used`-th plan: the plans
/// left on Free, nothing on Creator (its quota is not one to count down).
pub fn remaining(plan: Plan, used: i32) -> Option<u32> {
    match plan {
        Plan::Free => Some(u32::try_from(FREE_PLANS_PER_MONTH.saturating_sub(used)).unwrap_or(0)),
        Plan::Creator => None,
    }
}

/// Whether the `used`-th plan of a Creator is above the soft line.
pub fn above_soft_line(plan: Plan, used: i32) -> bool {
    plan == Plan::Creator && used > CREATOR_PLANS_SOFT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_has_40_a_month_and_creator_3000_with_a_soft_line_at_1000() {
        assert!(allows(Plan::Free, 39));
        assert!(!allows(Plan::Free, 40), "the 41st");
        assert!(allows(Plan::Creator, 2_999));
        assert!(!allows(Plan::Creator, 3_000), "the 3,001st");
        assert!(!above_soft_line(Plan::Creator, 1_000));
        assert!(above_soft_line(Plan::Creator, 1_001));
        assert!(!above_soft_line(Plan::Free, 1_001));
    }

    #[test]
    fn free_counts_down_and_creator_does_not() {
        assert_eq!(remaining(Plan::Free, 1), Some(39));
        assert_eq!(remaining(Plan::Free, 40), Some(0));
        assert_eq!(remaining(Plan::Free, 42), Some(0), "never below zero");
        assert_eq!(remaining(Plan::Creator, 5), None);
    }
}
