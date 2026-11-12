#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The AI plan's quota (TS §52, v6.2implementation D-461): counted in
//! `ai_usage` per UTC month, on success only.

mod common;

use common::TestApp;
use common::plan::{last_month, person, plans_counted, post, request, seed_plans, this_month};
use offcut_api::plan::provider::PlanProviderError;
use offcut_api_types::PlanResponse;
use reqwest::StatusCode;

#[tokio::test]
async fn only_a_plan_that_came_back_is_counted() {
    let app = TestApp::spawn().await;
    let (user, token) = person(&app).await;
    app.plan.answers.lock().unwrap().extend([
        Err(PlanProviderError::Unavailable),
        Err(PlanProviderError::Timeout),
        Ok("not a plan".to_owned()),
        Ok("still not a plan".to_owned()),
    ]);
    for _ in 0..3 {
        assert_ne!(
            post(&app, Some(&token), &request()).await.status(),
            StatusCode::OK
        );
    }
    assert_eq!(plans_counted(&app, user, this_month()).await, 0);
    let body: PlanResponse = post(&app, Some(&token), &request())
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(body.plans_remaining, Some(39));
    assert_eq!(plans_counted(&app, user, this_month()).await, 1);
}

#[tokio::test]
async fn each_month_counts_on_its_own() {
    let app = TestApp::spawn().await;
    let (user, token) = person(&app).await;
    seed_plans(&app, user, last_month(), 25).await;
    let body: PlanResponse = post(&app, Some(&token), &request())
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(
        body.plans_remaining,
        Some(39),
        "last month's 25 do not count"
    );
    assert_eq!(plans_counted(&app, user, last_month()).await, 25);
    assert_eq!(plans_counted(&app, user, this_month()).await, 1);
    // Another person's count is their own.
    let (other, other_token) = person(&app).await;
    assert_eq!(
        post(&app, Some(&other_token), &request()).await.status(),
        StatusCode::OK
    );
    assert_eq!(plans_counted(&app, other, this_month()).await, 1);
    assert_eq!(plans_counted(&app, user, this_month()).await, 1);
}
