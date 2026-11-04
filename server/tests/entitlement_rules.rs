#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! What a token says (TS §22.3), read through a real database and a real
//! signature. `issue` takes its clock as an argument, so a month's edge is
//! exact here, with no waiting.

mod common;

use common::TestApp;
use ed25519_dalek::SigningKey;
use offcut_api::entitlement::issue::issue;
use offcut_api_types::{ApiError, ApiErrorCode, EntitlementResponse};
use offcut_entitlement::claims::EntitlementClaims;
use offcut_entitlement::verify_token;
use offcut_types::{Plan, UserId};
use reqwest::StatusCode;
use serde_json::json;
use time::OffsetDateTime;
use time::macros::datetime;
use uuid::Uuid;

/// The test app's entitlement key (`test_config`).
fn key() -> SigningKey {
    SigningKey::from_bytes(&[1; 32])
}

async fn new_user(app: &TestApp) -> UserId {
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO users (id, email, email_normalized) VALUES ($1, $2, $2)")
        .bind(id)
        .bind(format!("{id}@example.com"))
        .execute(&app.pool)
        .await
        .unwrap();
    UserId::new(id)
}

async fn receipt(app: &TestApp, user: UserId, plan: &str, at: OffsetDateTime) {
    sqlx::query(
        "INSERT INTO usage_receipts (export_id, user_id, plan_at_export, created_at)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(user.get())
    .bind(plan)
    .bind(at)
    .execute(&app.pool)
    .await
    .unwrap();
}

async fn subscription(app: &TestApp, user: UserId, status: &str, period_end: OffsetDateTime) {
    sqlx::query(
        "INSERT INTO subscriptions (user_id, provider_customer_id, provider_subscription_id, plan,
           billing_interval, status, current_period_end, last_event_at)
         VALUES ($1, 'cus', $2, 'creator', 'monthly', $3, $4, now())",
    )
    .bind(user.get())
    .bind(format!("sub_{}", user.get()))
    .bind(status)
    .bind(period_end)
    .execute(&app.pool)
    .await
    .unwrap();
}

async fn claims_at(app: &TestApp, user: UserId, now: OffsetDateTime) -> EntitlementClaims {
    let issued = issue(&app.pool, &key(), user, now).await.unwrap();
    let public = key().verifying_key().to_bytes();
    let claims = verify_token(&issued.token, &[public]).unwrap();
    assert_eq!(issued.plan, claims.plan);
    claims
}

const NOW: OffsetDateTime = datetime!(2026-11-15 12:00 UTC);

#[tokio::test]
async fn a_free_user_has_three_then_fewer_then_none_never_below() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    for (used, left) in [(0, 3), (1, 2), (2, 1), (3, 0), (4, 0)] {
        let claims = claims_at(&app, user, NOW).await;
        assert_eq!(claims.plan, Plan::Free);
        assert_eq!(claims.free_exports_remaining.get(), left, "{used} used");
        receipt(&app, user, "free", NOW).await;
    }
}

#[tokio::test]
async fn a_token_holds_its_seven_fields() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    let claims = claims_at(&app, user, NOW).await;
    assert_eq!(claims.v, 1);
    assert_eq!(claims.sub, user);
    assert_eq!(claims.plan, Plan::Free);
    assert_eq!(claims.free_exports_remaining.get(), 3);
    assert_eq!(
        claims.period_end.get(),
        datetime!(2026-11-30 23:59:59 UTC).unix_timestamp()
    );
    assert_eq!(claims.iat.get(), NOW.unix_timestamp());
    assert_eq!(claims.exp.get(), NOW.unix_timestamp() + 7 * 24 * 3600);
}

#[tokio::test]
async fn only_exports_made_on_the_free_plan_count() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    receipt(&app, user, "creator", NOW).await;
    receipt(&app, user, "creator", NOW).await;
    receipt(&app, user, "free", NOW).await;
    let claims = claims_at(&app, user, NOW).await;
    assert_eq!(claims.free_exports_remaining.get(), 2);
}

#[tokio::test]
async fn another_users_exports_do_not_count() {
    let app = TestApp::spawn().await;
    let (user, other) = (new_user(&app).await, new_user(&app).await);
    receipt(&app, other, "free", NOW).await;
    assert_eq!(
        claims_at(&app, user, NOW)
            .await
            .free_exports_remaining
            .get(),
        3
    );
}

#[tokio::test]
async fn the_count_starts_again_at_the_first_instant_of_a_utc_month() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    receipt(&app, user, "free", datetime!(2026-11-30 23:59:59 UTC)).await;

    let last_second = claims_at(&app, user, datetime!(2026-11-30 23:59:59 UTC)).await;
    assert_eq!(last_second.free_exports_remaining.get(), 2);
    assert_eq!(
        last_second.period_end.get(),
        datetime!(2026-11-30 23:59:59 UTC).unix_timestamp()
    );

    let next_month = claims_at(&app, user, datetime!(2026-12-01 00:00 UTC)).await;
    assert_eq!(next_month.free_exports_remaining.get(), 3);
    assert_eq!(
        next_month.period_end.get(),
        datetime!(2026-12-31 23:59:59 UTC).unix_timestamp()
    );

    // An export at that first instant counts in the new month.
    receipt(&app, user, "free", datetime!(2026-12-01 00:00 UTC)).await;
    let counted = claims_at(&app, user, datetime!(2026-12-01 00:00 UTC)).await;
    assert_eq!(counted.free_exports_remaining.get(), 2);
}

#[tokio::test]
async fn the_count_starts_again_across_a_year() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    receipt(&app, user, "free", datetime!(2026-12-31 23:59:59 UTC)).await;

    let december = claims_at(&app, user, datetime!(2026-12-31 23:59:59 UTC)).await;
    assert_eq!(december.free_exports_remaining.get(), 2);
    assert_eq!(
        december.period_end.get(),
        datetime!(2026-12-31 23:59:59 UTC).unix_timestamp()
    );

    let january = claims_at(&app, user, datetime!(2027-01-01 00:00 UTC)).await;
    assert_eq!(january.free_exports_remaining.get(), 3);
    assert_eq!(
        january.period_end.get(),
        datetime!(2027-01-31 23:59:59 UTC).unix_timestamp()
    );
}

#[tokio::test]
async fn the_month_is_utc_whatever_the_clock_it_is_read_on() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    receipt(&app, user, "free", datetime!(2026-10-31 23:00 UTC)).await;
    // 02:00 on the 1st in India is still October in UTC.
    let india = claims_at(&app, user, datetime!(2026-11-01 02:00 +5:30)).await;
    assert_eq!(india.free_exports_remaining.get(), 2);
}

#[tokio::test]
async fn every_status_is_creator_until_the_period_ends_and_free_after() {
    let app = TestApp::spawn().await;
    let end = datetime!(2026-12-10 00:00 UTC);
    for status in ["active", "past_due", "canceled"] {
        let user = new_user(&app).await;
        subscription(&app, user, status, end).await;

        let paid = claims_at(&app, user, end).await;
        assert_eq!(paid.plan, Plan::Creator, "{status}");
        assert_eq!(paid.period_end.get(), end.unix_timestamp(), "{status}");
        assert_eq!(paid.free_exports_remaining.get(), 0, "{status}");

        let after = claims_at(&app, user, datetime!(2026-12-10 00:00:01 UTC)).await;
        assert_eq!(after.plan, Plan::Free, "{status}");
        assert_eq!(after.free_exports_remaining.get(), 3, "{status}");
        assert_eq!(
            after.period_end.get(),
            datetime!(2026-12-31 23:59:59 UTC).unix_timestamp(),
            "{status}"
        );
    }
}

#[tokio::test]
async fn a_refund_ends_the_plan_when_it_happens() {
    // The webhook sets the period's end to the refund's time (TS §22.5).
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    subscription(&app, user, "canceled", NOW).await;
    assert_eq!(claims_at(&app, user, NOW).await.plan, Plan::Creator);
    let later = datetime!(2026-11-15 12:00:01 UTC);
    assert_eq!(claims_at(&app, user, later).await.plan, Plan::Free);
}

#[tokio::test]
async fn exports_made_while_paying_do_not_spend_the_free_count_after() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    receipt(&app, user, "creator", datetime!(2026-11-02 12:00 UTC)).await;
    subscription(&app, user, "canceled", datetime!(2026-11-03 00:00 UTC)).await;
    let after = claims_at(&app, user, NOW).await;
    assert_eq!(after.plan, Plan::Free);
    assert_eq!(after.free_exports_remaining.get(), 3);
}

// --- GET /entitlement --------------------------------------------------------

async fn access_token_of(app: &TestApp) -> String {
    let email = "someone@example.com";
    let request = app.client.post(format!("{}/auth/magic-link", app.base_url));
    let response = request
        .json(&json!({ "email": email }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let link = app.mailer.sent.lock().unwrap().last().unwrap().1.clone();
    let token = link.rsplit_once("token=").unwrap().1.to_owned();
    let request = app.client.post(format!("{}/auth/verify", app.base_url));
    let response = request
        .json(&json!({ "token": token }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = response.json().await.unwrap();
    body["access_token"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn the_route_answers_a_fresh_token_for_a_session() {
    let app = TestApp::spawn().await;
    let access = access_token_of(&app).await;
    let url = format!("{}/entitlement", app.base_url);
    let response = app
        .client
        .get(&url)
        .bearer_auth(&access)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let body: EntitlementResponse = response.json().await.unwrap();
    let public = key().verifying_key().to_bytes();
    let claims = verify_token(&body.entitlement_token, &[public]).unwrap();
    assert_eq!(claims.plan, Plan::Free);
}

#[tokio::test]
async fn the_route_refuses_no_session_and_a_user_who_is_gone() {
    let app = TestApp::spawn().await;
    let url = format!("{}/entitlement", app.base_url);
    let none = app.client.get(&url).send().await.unwrap();
    assert_eq!(none.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        none.json::<ApiError>().await.unwrap().code,
        ApiErrorCode::Unauthorized
    );

    let access = access_token_of(&app).await;
    sqlx::query("DELETE FROM users")
        .execute(&app.pool)
        .await
        .unwrap();
    let gone = app
        .client
        .get(&url)
        .bearer_auth(&access)
        .send()
        .await
        .unwrap();
    assert_eq!(gone.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        gone.json::<ApiError>().await.unwrap().code,
        ApiErrorCode::SessionExpired
    );
}
