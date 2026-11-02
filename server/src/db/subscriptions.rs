//! The `subscriptions` table. Webhooks write it (Prompt 143).

use offcut_api_types::{BillingInterval, SubscriptionStatus, SubscriptionSummary};
use offcut_types::{UnixSecs, UserId};
use sqlx::PgPool;
use time::OffsetDateTime;

/// When the user's paid period ends, whatever the subscription's status:
/// a canceled or past-due one keeps its plan until then (TS §22.3).
pub async fn period_end(
    pool: &PgPool,
    user: UserId,
) -> Result<Option<OffsetDateTime>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT current_period_end FROM subscriptions WHERE user_id = $1",
        user.get(),
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| row.current_period_end))
}

/// The merchant's id of a subscription that would go on charging, if any.
pub async fn live_id(pool: &PgPool, user: UserId) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT provider_subscription_id FROM subscriptions
         WHERE user_id = $1 AND status IN ('active', 'past_due')",
        user.get(),
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| row.provider_subscription_id))
}

/// The user's subscription, as `GET /me` shows it. A column value outside its
/// `CHECK` is an error, since the database cannot hold one.
pub async fn summary(
    pool: &PgPool,
    user: UserId,
) -> Result<Option<SubscriptionSummary>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT status, billing_interval, current_period_end, cancel_at_period_end
         FROM subscriptions WHERE user_id = $1",
        user.get(),
    )
    .fetch_optional(pool)
    .await?;
    row.map(|row| {
        Ok(SubscriptionSummary {
            status: status_of(&row.status).ok_or(sqlx::Error::RowNotFound)?,
            interval: interval_of(&row.billing_interval).ok_or(sqlx::Error::RowNotFound)?,
            current_period_end: UnixSecs::new(row.current_period_end.unix_timestamp()),
            cancel_at_period_end: row.cancel_at_period_end,
        })
    })
    .transpose()
}

fn status_of(text: &str) -> Option<SubscriptionStatus> {
    match text {
        "active" => Some(SubscriptionStatus::Active),
        "past_due" => Some(SubscriptionStatus::PastDue),
        "canceled" => Some(SubscriptionStatus::Canceled),
        _ => None,
    }
}

fn interval_of(text: &str) -> Option<BillingInterval> {
    match text {
        "monthly" => Some(BillingInterval::Monthly),
        "annual" => Some(BillingInterval::Annual),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_column_value_of_the_check_has_its_member() {
        let statuses = [
            ("active", SubscriptionStatus::Active),
            ("past_due", SubscriptionStatus::PastDue),
            ("canceled", SubscriptionStatus::Canceled),
        ];
        for (text, status) in statuses {
            assert_eq!(status_of(text), Some(status));
        }
        assert_eq!(status_of("paused"), None);
        assert_eq!(interval_of("monthly"), Some(BillingInterval::Monthly));
        assert_eq!(interval_of("annual"), Some(BillingInterval::Annual));
        assert_eq!(interval_of("weekly"), None);
    }
}
