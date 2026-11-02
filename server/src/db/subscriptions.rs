//! The `subscriptions` table. Webhooks write it (Prompt 143).

use offcut_types::UserId;
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
