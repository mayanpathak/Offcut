//! The database: the pool, the migrations, and one module per table.

use sqlx::PgPool;
use sqlx::migrate::MigrateError;
use sqlx::postgres::PgPoolOptions;

pub mod ai_spend;
pub mod ai_usage;
pub mod analytics_events;
pub mod magic_links;
pub mod platform_waitlist;
pub mod sessions;
pub mod subscriptions;
pub mod usage_receipts;
pub mod users;
pub mod webhook_events;

/// Every table with a row per user (v6implementation D-270, S-10). Deletion and
/// export each have a test that fails on a table left out: a new one goes here.
pub const USER_TABLES: &[&str] = &[
    "sessions",
    "subscriptions",
    "usage_receipts",
    // V6.2 (TS §52, D-462): the AI plan's counts and the long path's grants.
    "ai_usage",
    "ai_shorts_grants",
];

/// The free database tier allows few connections (TS §22.9).
const MAX_CONNECTIONS: u32 = 5;

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .connect(database_url)
        .await
}

/// Applies the migrations that the database does not have yet. They are
/// forward-only and run at every boot (TS §23.3).
///
/// From V6.2 (v6.2implementation §14.4, V6 CL open item 38), a version the
/// database has and this build does not know is ignored: a rollback to the
/// previous release then boots on a database a later one migrated. sqlx
/// refuses it by default (`VersionMissing`), which is why V6.1 needs the
/// rollback step of §14.4.
pub async fn migrate(pool: &PgPool) -> Result<(), MigrateError> {
    let mut migrator = sqlx::migrate!("./migrations");
    migrator.set_ignore_missing(true);
    migrator.run(pool).await
}
