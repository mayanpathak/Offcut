#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The AI spend cap (TS §52, v6.2implementation D-463). P189: the month's
//! spend in `ai_spend`; P191 adds the cap's 80% line and its refusal.

mod common;

use common::TestApp;
use offcut_api::db::ai_spend;
use time::macros::date;

#[tokio::test]
async fn each_call_adds_to_its_month_and_returns_the_total() {
    let app = TestApp::spawn().await;
    let november = date!(2026 - 11 - 01);
    let december = date!(2026 - 12 - 01);
    assert_eq!(ai_spend::total(&app.pool, november).await.unwrap(), 0);
    assert_eq!(
        ai_spend::add(&app.pool, november, 1_100).await.unwrap(),
        1_100
    );
    assert_eq!(
        ai_spend::add(&app.pool, november, 4_900).await.unwrap(),
        6_000
    );
    assert_eq!(ai_spend::add(&app.pool, december, 700).await.unwrap(), 700);
    assert_eq!(ai_spend::total(&app.pool, november).await.unwrap(), 6_000);
    assert_eq!(ai_spend::total(&app.pool, december).await.unwrap(), 700);
}

/// Calls in flight at once each add exactly once (`ON CONFLICT ... RETURNING`).
#[tokio::test]
async fn concurrent_calls_lose_no_spend() {
    let app = TestApp::spawn().await;
    let month = date!(2026 - 11 - 01);
    let mut adds = tokio::task::JoinSet::new();
    for _ in 0..20 {
        let pool = app.pool.clone();
        adds.spawn(async move { ai_spend::add(&pool, month, 50).await });
    }
    while let Some(added) = adds.join_next().await {
        assert!(added.unwrap().is_ok());
    }
    assert_eq!(ai_spend::total(&app.pool, month).await.unwrap(), 1_000);
}
