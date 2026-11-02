//! Retention (TS §23.3, PS §16): analytics rows, sign-in links, webhook events
//! and sessions that are over. The purge runs inside the server process; there
//! is no cron job.

use std::time::Duration;

use offcut_types::ANALYTICS_RETENTION_DAYS;
use sqlx::PgPool;

use crate::db::{analytics_events, magic_links, sessions, webhook_events};
use crate::error::db_error_kind;

const PURGE_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);
const MAGIC_LINK_RETENTION_DAYS: u32 = 1;
/// Longer than the merchant's retries, about 27 hours (assumption, TE-9).
const WEBHOOK_EVENT_RETENTION_DAYS: u32 = 90;

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

/// Deletes the rows past their retention and returns how many, in all.
pub async fn purge_once(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let analytics = analytics_events::purge_older_than(pool, ANALYTICS_RETENTION_DAYS).await?;
    let links = magic_links::purge_older_than(pool, MAGIC_LINK_RETENTION_DAYS).await?;
    let events = webhook_events::purge_older_than(pool, WEBHOOK_EVENT_RETENTION_DAYS).await?;
    let sessions = sessions::purge_finished(pool).await?;
    Ok(analytics + links + events + sessions)
}
