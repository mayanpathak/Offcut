//! The global audio budget of cloud transcription, the only place it is
//! set (cloudimplementation D-346, D-362). It is held under the provider's
//! free tier: 7,200 audio seconds an hour (read from its response headers,
//! cloud CL Entry 2) and 28,800 a day (its documents). The counters live in
//! memory and are lost at a restart, which is accepted: the provider's own
//! limit is the last line.

use std::sync::{Mutex, MutexGuard, PoisonError};

/// Audio seconds accepted per UTC hour, in production.
pub const HOUR_SECONDS: u32 = 7_000;
/// Audio seconds accepted per UTC day, in production.
pub const DAY_SECONDS: u32 = 27_000;

// Under the provider's free tier, always.
const _: () = assert!(HOUR_SECONDS < 7_200 && DAY_SECONDS < 28_800);

const MS_PER_SECOND: u64 = 1_000;
const SECONDS_PER_HOUR: i64 = 3_600;
const SECONDS_PER_DAY: i64 = 86_400;

#[derive(Default)]
struct Counters {
    hour: i64,
    hour_used_ms: u64,
    day: i64,
    day_used_ms: u64,
}

pub struct Budget {
    hour_limit_ms: u64,
    day_limit_ms: u64,
    counters: Mutex<Counters>,
}

impl Budget {
    pub fn new(hour_seconds: u32, day_seconds: u32) -> Budget {
        Budget {
            hour_limit_ms: u64::from(hour_seconds) * MS_PER_SECOND,
            day_limit_ms: u64::from(day_seconds) * MS_PER_SECOND,
            counters: Mutex::default(),
        }
    }

    /// Charges `duration_ms` at `now` (Unix seconds) if both the hour's and the
    /// day's budget hold it, and says whether it did. A refused charge
    /// changes nothing.
    pub fn charge(&self, duration_ms: u32, now: i64) -> bool {
        let hour = now.div_euclid(SECONDS_PER_HOUR);
        let day = now.div_euclid(SECONDS_PER_DAY);
        let mut counters = self.lock();
        if counters.hour != hour {
            counters.hour = hour;
            counters.hour_used_ms = 0;
        }
        if counters.day != day {
            counters.day = day;
            counters.day_used_ms = 0;
        }
        let cost = u64::from(duration_ms);
        let hour_after = counters.hour_used_ms.saturating_add(cost);
        let day_after = counters.day_used_ms.saturating_add(cost);
        if hour_after > self.hour_limit_ms || day_after > self.day_limit_ms {
            return false;
        }
        counters.hour_used_ms = hour_after;
        counters.day_used_ms = day_after;
        true
    }

    fn lock(&self) -> MutexGuard<'_, Counters> {
        // A poisoned lock is still used: the worst a half-finished charge
        // leaves is a count a few seconds off.
        self.counters.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Default for Budget {
    fn default() -> Budget {
        Budget::new(HOUR_SECONDS, DAY_SECONDS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-11-06 10:00:00 UTC.
    const TEN_AM: i64 = 1_793_959_200;

    #[test]
    fn the_hour_refuses_what_would_pass_it_and_refuses_without_charging() {
        let budget = Budget::new(100, 1_000);
        assert!(budget.charge(90_000, TEN_AM));
        assert!(!budget.charge(10_001, TEN_AM + 60));
        // The refused charge took nothing: exactly what is left still fits.
        assert!(budget.charge(10_000, TEN_AM + 120));
        assert!(!budget.charge(1, TEN_AM + 180));
    }

    #[test]
    fn a_new_hour_starts_empty_and_the_day_keeps_counting() {
        let budget = Budget::new(100, 150);
        assert!(budget.charge(100_000, TEN_AM + 3_599));
        assert!(budget.charge(50_000, TEN_AM + 3_600));
        // The second hour has room, the day has none.
        assert!(!budget.charge(1, TEN_AM + 3_700));
    }

    #[test]
    fn a_new_utc_day_starts_empty() {
        let budget = Budget::new(100, 100);
        let last_second = TEN_AM - 10 * 3_600 + 86_399;
        assert!(budget.charge(100_000, last_second));
        assert!(!budget.charge(1, last_second));
        assert!(budget.charge(100_000, last_second + 1));
    }

    #[test]
    fn production_values_are_under_the_free_tier() {
        let budget = Budget::default();
        assert!(budget.charge(HOUR_SECONDS * 1_000, TEN_AM));
        assert!(!budget.charge(1, TEN_AM));
    }
}
