//! The `webhook_events` table: which merchant events have been applied.

use sqlx::PgPool;

/// Deletes the rows older than `days` days and returns how many.
pub async fn purge_older_than(pool: &PgPool, days: u32) -> Result<u64, sqlx::Error> {
    let days = i32::try_from(days).unwrap_or(i32::MAX);
    let deleted = sqlx::query!(
        "DELETE FROM webhook_events WHERE processed_at < now() - make_interval(days => $1)",
        days,
    )
    .execute(pool)
    .await?;
    Ok(deleted.rows_affected())
}
