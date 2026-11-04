//! The `subscriptions` table. Only the merchant's webhooks write it (C-19).

use offcut_api_types::{BillingInterval, SubscriptionStatus, SubscriptionSummary};
use offcut_types::{UnixSecs, UserId};
use sqlx::{PgConnection, PgPool};
use time::OffsetDateTime;

/// A row as `billing/webhook.rs` reads and writes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubscriptionRow {
    pub customer_id: String,
    pub subscription_id: String,
    pub interval: BillingInterval,
    pub status: SubscriptionStatus,
    pub cancel_at_period_end: bool,
    pub current_period_end: OffsetDateTime,
    pub last_event_at: OffsetDateTime,
}

/// The user's row, locked until the transaction ends.
pub async fn for_update(
    conn: &mut PgConnection,
    user: UserId,
) -> Result<Option<SubscriptionRow>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT provider_customer_id, provider_subscription_id, billing_interval, status,
                cancel_at_period_end, current_period_end, last_event_at
         FROM subscriptions WHERE user_id = $1 FOR UPDATE",
        user.get(),
    )
    .fetch_optional(conn)
    .await?;
    row.map(|row| {
        Ok(SubscriptionRow {
            customer_id: row.provider_customer_id,
            subscription_id: row.provider_subscription_id,
            interval: interval_of(&row.billing_interval).ok_or(sqlx::Error::RowNotFound)?,
            status: status_of(&row.status).ok_or(sqlx::Error::RowNotFound)?,
            cancel_at_period_end: row.cancel_at_period_end,
            current_period_end: row.current_period_end,
            last_event_at: row.last_event_at,
        })
    })
    .transpose()
}

/// Writes the user's row, new or not.
pub async fn upsert(
    conn: &mut PgConnection,
    user: UserId,
    row: &SubscriptionRow,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO subscriptions (user_id, provider_customer_id, provider_subscription_id, plan,
           billing_interval, status, cancel_at_period_end, current_period_end, last_event_at)
         VALUES ($1, $2, $3, 'creator', $4, $5, $6, $7, $8)
         ON CONFLICT (user_id) DO UPDATE SET
           provider_customer_id = EXCLUDED.provider_customer_id,
           provider_subscription_id = EXCLUDED.provider_subscription_id,
           billing_interval = EXCLUDED.billing_interval,
           status = EXCLUDED.status,
           cancel_at_period_end = EXCLUDED.cancel_at_period_end,
           current_period_end = EXCLUDED.current_period_end,
           last_event_at = EXCLUDED.last_event_at,
           updated_at = now()",
        user.get(),
        row.customer_id,
        row.subscription_id,
        interval_text(row.interval),
        status_text(row.status),
        row.cancel_at_period_end,
        row.current_period_end,
        row.last_event_at,
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// The user who holds this merchant customer, locked until the transaction ends.
pub async fn user_of_customer(
    conn: &mut PgConnection,
    customer_id: &str,
) -> Result<Option<UserId>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT user_id FROM subscriptions WHERE provider_customer_id = $1
         ORDER BY updated_at DESC LIMIT 1 FOR UPDATE",
        customer_id,
    )
    .fetch_optional(conn)
    .await?;
    Ok(row.map(|row| UserId::new(row.user_id)))
}

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

/// The merchant's id of the user's customer, if they have ever subscribed.
pub async fn customer_id(pool: &PgPool, user: UserId) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT provider_customer_id FROM subscriptions WHERE user_id = $1",
        user.get(),
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| row.provider_customer_id))
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

fn status_text(status: SubscriptionStatus) -> &'static str {
    match status {
        SubscriptionStatus::Active => "active",
        SubscriptionStatus::PastDue => "past_due",
        SubscriptionStatus::Canceled => "canceled",
    }
}

fn interval_text(interval: BillingInterval) -> &'static str {
    match interval {
        BillingInterval::Monthly => "monthly",
        BillingInterval::Annual => "annual",
    }
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
        for (_, status) in statuses {
            assert_eq!(status_of(status_text(status)), Some(status));
        }
        for interval in [BillingInterval::Monthly, BillingInterval::Annual] {
            assert_eq!(interval_of(interval_text(interval)), Some(interval));
        }
        assert_eq!(interval_of("monthly"), Some(BillingInterval::Monthly));
        assert_eq!(interval_of("annual"), Some(BillingInterval::Annual));
        assert_eq!(interval_of("weekly"), None);
    }
}
