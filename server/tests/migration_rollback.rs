#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! A rollback across a migration (v6.2implementation §14.4, V6 CL open item
//! 38): V6.1's migrator refuses a database that V6.2 migrated until §14.4's
//! one step is taken; V6.2 then comes back without losing a row; and from
//! V6.2 on, a version the build does not know is ignored.

mod common;

use common::TestApp;
use offcut_api::db::{ai_spend, migrate};
use sqlx::migrate::{MigrateError, Migrator};
use time::macros::date;

/// V6.1's migrations as V6.1's binary embeds them: `0001_init.sql` alone.
async fn v61_migrator() -> Migrator {
    let dir = std::env::temp_dir().join(format!("offcut-v61-migrations-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("0001_init.sql"),
        include_str!("../migrations/0001_init.sql"),
    )
    .unwrap();
    let mut migrator = Migrator::new(dir.as_path()).await.unwrap();
    // sqlx 0.9 returns `VersionMissing` while still holding its advisory
    // lock, on a connection the pool keeps. A refused binary exits, which
    // frees it; this test goes on, so this migrator takes no lock.
    migrator.set_locking(false);
    let _ = std::fs::remove_dir_all(&dir);
    migrator
}

async fn versions(app: &TestApp) -> Vec<i64> {
    sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
        .fetch_all(&app.pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn v61_boots_after_the_rollback_step_and_v62_comes_back_with_its_rows() {
    let app = TestApp::spawn().await;
    let month = date!(2026 - 11 - 01);
    ai_spend::add(&app.pool, month, 1_234).await.unwrap();
    assert_eq!(versions(&app).await, [1, 2]);

    // V6.1's binary on V6.2's database, as it is: refused.
    let v61 = v61_migrator().await;
    assert!(matches!(
        v61.run(&app.pool).await,
        Err(MigrateError::VersionMissing(2))
    ));

    // §14.4's step: forget version 2, keep its tables. V6.1 then boots.
    sqlx::query("DELETE FROM _sqlx_migrations WHERE version = 2")
        .execute(&app.pool)
        .await
        .unwrap();
    v61.run(&app.pool).await.unwrap();

    // Forward again: 0002 runs a second time over its own tables.
    migrate(&app.pool).await.unwrap();
    assert_eq!(versions(&app).await, [1, 2]);
    assert_eq!(ai_spend::total(&app.pool, month).await.unwrap(), 1_234);
}

#[tokio::test]
async fn a_version_this_build_does_not_know_is_ignored() {
    let app = TestApp::spawn().await;
    // A later release's migration, as V6.3 would record it.
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) \
         VALUES (3, 'a later release', TRUE, '\\x00', 0)",
    )
    .execute(&app.pool)
    .await
    .unwrap();
    migrate(&app.pool).await.unwrap();
    assert_eq!(versions(&app).await, [1, 2, 3]);
}
