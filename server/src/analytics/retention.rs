//! Retention: analytics rows are deleted after `ANALYTICS_RETENTION_DAYS`
//! (PS §16). The purge runs inside the server process; there is no cron job.

use std::time::Duration;

use offcut_types::ANALYTICS_RETENTION_DAYS;
use sqlx::PgPool;

use crate::db::analytics_events;
use crate::error::db_error_kind;

const PURGE_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

/// Purges once now and then every 6 hours, for as long as the process lives.
/// Call once at startup, inside the runtime.
pub fn spawn_purge_task(pool: PgPool) {
    tokio::spawn(async move {
        // The first tick completes at once.
        let mut ticks = tokio::time::interval(PURGE_INTERVAL);
        loop {
            ticks.tick().await;
            match purge_once(&pool).await {
                Ok(deleted) => tracing::info!(deleted, "retention purge done"),
                Err(error) => {
                    tracing::error!(kind = db_error_kind(&error), "retention purge failed");
                }
            }
        }
    });
}

/// Deletes the analytics rows past their retention and returns how many.
pub async fn purge_once(pool: &PgPool) -> Result<u64, sqlx::Error> {
    analytics_events::purge_older_than(pool, ANALYTICS_RETENTION_DAYS).await
}
