//! The AI spend cap (TS §52, v6.2implementation D-463): the only place it is
//! worked out. Each provider call's cost is added to `ai_spend` for its UTC
//! month; at 80% of `AI_MONTHLY_CAP_USD` one `warn` line a month; at 100%
//! every AI route answers `E_PLAN_UNAVAILABLE` until the next month.
//!
//! Every input token is priced at the cache-miss price (D-500): an upper
//! bound, so the cap is reached early rather than late.

use std::sync::Mutex;

use sqlx::PgPool;
use time::Date;

use crate::config::PlanSettings;
use crate::db::ai_spend;
use crate::plan::provider::Usage;

/// The cap and the month whose 80% line has been logged. The month is held
/// in memory: after a restart the line may be logged once more.
pub struct SpendCap {
    cap_micro_usd: i64,
    price_in: f64,
    price_out: f64,
    warned: Mutex<Option<Date>>,
}

impl SpendCap {
    pub fn new(settings: &PlanSettings) -> SpendCap {
        SpendCap {
            cap_micro_usd: settings.monthly_cap_micro_usd,
            price_in: settings.price_in_usd_per_mtok,
            price_out: settings.price_out_usd_per_mtok,
            warned: Mutex::new(None),
        }
    }

    /// A call's cost in micro-dollars: a token at `p` USD per 1M tokens costs
    /// `p` micro-dollars. Rounded up.
    pub fn cost(&self, usage: Usage) -> i64 {
        let cost = f64::from(usage.input_tokens) * self.price_in
            + f64::from(usage.output_tokens) * self.price_out;
        // At most a few dollars a call: in range.
        cost.ceil() as i64
    }

    /// Whether `month`'s spend so far leaves room for another call.
    pub async fn open(&self, pool: &PgPool, month: Date) -> Result<bool, sqlx::Error> {
        Ok(ai_spend::total(pool, month).await? < self.cap_micro_usd)
    }

    /// Adds a call's cost to `month`, and logs the 80% line the first time
    /// the month crosses it.
    pub async fn charge(
        &self,
        pool: &PgPool,
        month: Date,
        usage: Usage,
    ) -> Result<(), sqlx::Error> {
        let total = ai_spend::add(pool, month, self.cost(usage)).await?;
        if self.first_crossing(month, total) {
            tracing::warn!(
                percent = total.saturating_mul(100) / self.cap_micro_usd.max(1),
                "ai spend passed 80% of the monthly cap"
            );
        }
        Ok(())
    }

    /// Whether `total` is at or over 80% of the cap and `month` has not said so yet.
    fn first_crossing(&self, month: Date, total: i64) -> bool {
        if total.saturating_mul(5) < self.cap_micro_usd.saturating_mul(4) {
            return false;
        }
        let mut warned = self
            .warned
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *warned == Some(month) {
            return false;
        }
        *warned = Some(month);
        true
    }
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;
    use crate::config::PlanProvider;

    fn cap(dollars: i64) -> SpendCap {
        SpendCap::new(&PlanSettings {
            enabled: true,
            provider: PlanProvider::Deepseek,
            model: "m".to_owned(),
            api_key: String::new(),
            api_base_url: String::new(),
            monthly_cap_micro_usd: dollars * 1_000_000,
            price_in_usd_per_mtok: 0.30,
            price_out_usd_per_mtok: 1.20,
        })
    }

    #[test]
    fn a_call_costs_its_tokens_at_the_two_prices_rounded_up() {
        let usage = |input, output| Usage {
            input_tokens: input,
            cached_input_tokens: input / 2,
            output_tokens: output,
        };
        // TE-15's median Flash plan: 5,465 in, 492 out → 1,639.5 + 590.4 µ$.
        assert_eq!(cap(5).cost(usage(5_465, 492)), 2_230);
        assert_eq!(cap(5).cost(usage(0, 0)), 0);
        assert_eq!(
            cap(5).cost(usage(1, 0)),
            1,
            "a part of a micro-dollar rounds up"
        );
    }

    #[test]
    fn the_80_percent_line_is_said_once_a_month() {
        let cap = cap(5);
        let (november, december) = (date!(2026 - 11 - 01), date!(2026 - 12 - 01));
        assert!(!cap.first_crossing(november, 3_999_999));
        assert!(cap.first_crossing(november, 4_000_000));
        assert!(!cap.first_crossing(november, 4_500_000));
        assert!(!cap.first_crossing(november, 5_000_000));
        assert!(
            cap.first_crossing(december, 4_100_000),
            "a new month says it again"
        );
    }
}
