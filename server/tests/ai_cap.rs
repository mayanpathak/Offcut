#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The AI spend cap (TS §52, v6.2implementation D-463). P189: the month's
//! spend in `ai_spend`; P191 adds the cap's refusal (its 80% line is read in
//! `log_redaction.rs`, whose subscriber is the process's).

mod common;

use common::plan::{last_month, person, post, request, spent, this_month};
use common::{FAKE_USAGE, TestApp, TestOptions, plan_settings};
use offcut_api::db::ai_spend;
use offcut_api::plan::spend::SpendCap;
use offcut_api_types::{ApiError, ApiErrorCode};
use reqwest::StatusCode;
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

// --- P191: the cap on the route ---------------------------------------------

/// A cap of three calls' cost at the tests' prices.
fn three_calls() -> i64 {
    3 * SpendCap::new(&plan_settings(1)).cost(FAKE_USAGE)
}

#[tokio::test]
async fn at_the_cap_every_plan_is_unavailable_until_the_next_month() {
    let app = TestApp::spawn_with(TestOptions {
        plan_cap_micro_usd: three_calls(),
        ..TestOptions::default()
    })
    .await;
    let (_, token) = person(&app).await;
    // Last month spent its whole cap: this month is open.
    ai_spend::add(&app.pool, last_month(), three_calls())
        .await
        .unwrap();
    for n in 0..3 {
        let response = post(&app, Some(&token), &request()).await;
        assert_eq!(response.status(), StatusCode::OK, "call {n}");
    }
    assert_eq!(spent(&app, this_month()).await, three_calls());
    let response = post(&app, Some(&token), &request()).await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let body: ApiError = response.json().await.unwrap();
    assert_eq!(body.code, ApiErrorCode::PlanUnavailable);
    assert_eq!(
        app.plan.received.lock().unwrap().len(),
        3,
        "the 4th was not sent"
    );
}
