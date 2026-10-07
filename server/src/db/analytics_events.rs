//! The `analytics_events` table. A row holds an anonymous id, an event name,
//! its props and the time the server received it, and nothing else (TS §22.6).

use offcut_types::AnonId;
use sqlx::PgPool;

/// Inserts the events of one batch, each a `(name, props)` pair. It is one
/// statement, so either every row is stored or none is. `ts` is set by the
/// database.
pub async fn insert_batch(
    pool: &PgPool,
    anon_id: AnonId,
    rows: &[(&'static str, serde_json::Value)],
) -> Result<(), sqlx::Error> {
    let names: Vec<&str> = rows.iter().map(|(name, _)| *name).collect();
    let props: Vec<&serde_json::Value> = rows.iter().map(|(_, props)| props).collect();
    sqlx::query!(
        "INSERT INTO analytics_events (anon_id, name, props)
         SELECT $1, name, props FROM UNNEST($2::text[], $3::jsonb[]) AS batch (name, props)",
        anon_id.get(),
        &names as &[&str],
        &props as &[&serde_json::Value],
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Deletes the rows received more than `days` days ago and returns how many.
pub async fn purge_older_than(pool: &PgPool, days: u32) -> Result<u64, sqlx::Error> {
    let days = i32::try_from(days).unwrap_or(i32::MAX);
    let deleted = sqlx::query!(
        "DELETE FROM analytics_events WHERE ts < now() - make_interval(days => $1)",
        days,
    )
    .execute(pool)
    .await?;
    Ok(deleted.rows_affected())
}
