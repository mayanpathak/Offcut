//! What a user may export, signed (TS §22.3). The only place a plan and the
//! remaining exports are computed (v6implementation D-266). "Month" is the UTC
//! calendar month.

use ed25519_dalek::SigningKey;
use offcut_entitlement::claims::{CLAIMS_VERSION, EntitlementClaims};
use offcut_entitlement::sign::sign_token;
use offcut_types::limits::{ENTITLEMENT_OFFLINE_TTL_SECS, FREE_EXPORTS_PER_MONTH};
use offcut_types::{ExportCount, Plan, UnixSecs, UserId};
use sqlx::PgPool;
use time::{Date, Duration, OffsetDateTime, Time};

use crate::db::{subscriptions, usage_receipts};
use crate::error::AppError;

/// A signed token, and the plan it says.
pub struct Issued {
    pub token: String,
    pub plan: Plan,
}

/// The plan of a user at `now`: the only place it is worked out. A canceled or
/// past-due subscription keeps Creator until its period ends (TS §22.3).
pub async fn plan_now(pool: &PgPool, user: UserId, now: OffsetDateTime) -> Result<Plan, AppError> {
    let paid = paid_until(pool, user, now).await?;
    Ok(if paid.is_some() {
        Plan::Creator
    } else {
        Plan::Free
    })
}

async fn paid_until(
    pool: &PgPool,
    user: UserId,
    now: OffsetDateTime,
) -> Result<Option<OffsetDateTime>, AppError> {
    let period = subscriptions::period_end(pool, user).await?;
    Ok(period.filter(|end| now <= *end))
}

pub async fn issue(
    pool: &PgPool,
    key: &SigningKey,
    user: UserId,
    now: OffsetDateTime,
) -> Result<Issued, AppError> {
    let (plan, remaining, period_end) = match paid_until(pool, user, now).await? {
        Some(end) => (Plan::Creator, 0, end),
        None => {
            let (start, end) = utc_month(now).ok_or(AppError::Internal)?;
            let used = usage_receipts::free_used_since(pool, user, start).await?;
            let used = u32::try_from(used).unwrap_or(u32::MAX);
            let remaining = FREE_EXPORTS_PER_MONTH.get().saturating_sub(used);
            (Plan::Free, remaining, end)
        }
    };
    let claims = EntitlementClaims {
        v: CLAIMS_VERSION,
        sub: user,
        plan,
        free_exports_remaining: ExportCount::new(remaining),
        period_end: UnixSecs::new(period_end.unix_timestamp()),
        iat: UnixSecs::new(now.unix_timestamp()),
        exp: UnixSecs::new(now.unix_timestamp() + ENTITLEMENT_OFFLINE_TTL_SECS),
    };
    Ok(Issued {
        token: sign_token(&claims, key),
        plan,
    })
}

/// The first instant of the UTC month of `now`, and the last second of it.
pub fn utc_month(now: OffsetDateTime) -> Option<(OffsetDateTime, OffsetDateTime)> {
    let now = now.to_offset(time::UtcOffset::UTC);
    let start = now.date().replace_day(1).ok()?;
    // `next()` goes from December to January, so the year follows it.
    let month = now.month().next();
    let year = if month == time::Month::January {
        now.year().checked_add(1)?
    } else {
        now.year()
    };
    let next = Date::from_calendar_date(year, month, 1).ok()?;
    let at_midnight = |date: Date| date.with_time(Time::MIDNIGHT).assume_utc();
    Some((at_midnight(start), at_midnight(next) - Duration::SECOND))
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;

    #[test]
    fn a_month_runs_from_its_first_instant_to_its_last_second() {
        let cases = [
            (
                datetime!(2026-11-02 13:00 UTC),
                datetime!(2026-11-01 00:00 UTC),
                datetime!(2026-11-30 23:59:59 UTC),
            ),
            (
                datetime!(2026-12-31 23:59:59 UTC),
                datetime!(2026-12-01 00:00 UTC),
                datetime!(2026-12-31 23:59:59 UTC),
            ),
            (
                datetime!(2028-02-29 12:00 UTC),
                datetime!(2028-02-01 00:00 UTC),
                datetime!(2028-02-29 23:59:59 UTC),
            ),
            // Late on the 31st in India is still the 31st in UTC, and early
            // on the 1st there is the month before in UTC.
            (
                datetime!(2026-11-01 02:00 +5:30),
                datetime!(2026-10-01 00:00 UTC),
                datetime!(2026-10-31 23:59:59 UTC),
            ),
        ];
        for (now, start, end) in cases {
            assert_eq!(utc_month(now), Some((start, end)), "{now}");
        }
    }
}
