//! The `webhook_events` table: which merchant events have been applied.

use sqlx::{PgConnection, PgPool};

/// Records an event. `false` means it has been recorded before: a replay.
pub async fn record(
    conn: &mut PgConnection,
    provider_event_id: &str,
    event_type: &str,
) -> Result<bool, sqlx::Error> {
    let inserted = sqlx::query!(
        "INSERT INTO webhook_events (provider_event_id, event_type) VALUES ($1, $2)
         ON CONFLICT (provider_event_id) DO NOTHING",
        provider_event_id,
        event_type,
    )
    .execute(conn)
    .await?;
    Ok(inserted.rows_affected() == 1)
}

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
