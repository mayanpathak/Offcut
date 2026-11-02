#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! `POST /usage/receipts` (TS §22.4): one row per export, never refused for
//! being over a limit, and repeats change nothing.

mod common;

use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};

use common::TestApp;
use ed25519_dalek::SigningKey;
use offcut_api::auth::access_token;
use offcut_api::log::user_tag;
use offcut_api_types::{ApiError, ApiErrorCode, EntitlementResponse};
use offcut_entitlement::claims::EntitlementClaims;
use offcut_entitlement::verify_token;
use offcut_types::{Plan, UnixSecs, UserId};
use reqwest::{Response, StatusCode};
use serde_json::json;
use time::OffsetDateTime;
use tracing_subscriber::fmt::MakeWriter;
use uuid::Uuid;

/// Everything this test binary logs, for the fair-use case.
#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

impl Write for Captured {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for Captured {
    type Writer = Captured;

    fn make_writer(&'a self) -> Captured {
        self.clone()
    }
}

fn captured_logs() -> Captured {
    static LOGS: OnceLock<Captured> = OnceLock::new();
    LOGS.get_or_init(|| {
        let logs = Captured::default();
        tracing_subscriber::fmt()
            .json()
            .with_max_level(tracing::Level::INFO)
            .with_writer(logs.clone())
            .init();
        logs
    })
    .clone()
}

/// A session of a new address: the user and an access token.
async fn signed_in(app: &TestApp, email: &str) -> (UserId, String) {
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
    let access = body["access_token"].as_str().unwrap().to_owned();
    let key = SigningKey::from_bytes(&[2; 32]).verifying_key();
    let now = UnixSecs::new(OffsetDateTime::now_utc().unix_timestamp());
    let claims = access_token::verify(&access, &key, now).unwrap();
    (claims.sub, access)
}

async fn post_receipt(app: &TestApp, access: &str, export_id: Uuid) -> Response {
    let request = app.client.post(format!("{}/usage/receipts", app.base_url));
    request
        .bearer_auth(access)
        .json(&json!({ "export_id": export_id }))
        .send()
        .await
        .unwrap()
}

async fn token_claims(response: Response) -> EntitlementClaims {
    assert_eq!(response.status(), StatusCode::OK);
    let body: EntitlementResponse = response.json().await.unwrap();
    let public = SigningKey::from_bytes(&[1; 32]).verifying_key().to_bytes();
    verify_token(&body.entitlement_token, &[public]).unwrap()
}

async fn rows(app: &TestApp) -> Vec<(Uuid, Uuid, String)> {
    sqlx::query_as(
        "SELECT export_id, user_id, plan_at_export FROM usage_receipts ORDER BY created_at",
    )
    .fetch_all(&app.pool)
    .await
    .unwrap()
}

async fn subscribe(app: &TestApp, user: UserId) {
    sqlx::query(
        "INSERT INTO subscriptions (user_id, provider_customer_id, provider_subscription_id, plan,
           billing_interval, status, current_period_end, last_event_at)
         VALUES ($1, 'cus', 'sub', 'creator', 'monthly', 'active', now() + interval '10 days', now())",
    )
    .bind(user.get())
    .execute(&app.pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn a_receipt_is_a_row_under_the_plan_of_now_and_answers_a_fresh_token() {
    let app = TestApp::spawn().await;
    let (user, access) = signed_in(&app, "someone@example.com").await;
    let export = Uuid::now_v7();

    let response = post_receipt(&app, &access, export).await;
    assert_eq!(response.headers()["cache-control"], "no-store");
    let claims = token_claims(response).await;
    assert_eq!(claims.sub, user);
    assert_eq!(claims.plan, Plan::Free);
    assert_eq!(claims.free_exports_remaining.get(), 2);
    assert_eq!(rows(&app).await, [(export, user.get(), "free".to_owned())]);
}

#[tokio::test]
async fn a_creators_receipt_records_creator_and_spends_no_free_count() {
    let app = TestApp::spawn().await;
    let (user, access) = signed_in(&app, "someone@example.com").await;
    subscribe(&app, user).await;
    let export = Uuid::now_v7();

    let claims = token_claims(post_receipt(&app, &access, export).await).await;
    assert_eq!(claims.plan, Plan::Creator);
    assert_eq!(
        rows(&app).await,
        [(export, user.get(), "creator".to_owned())]
    );
}

#[tokio::test]
async fn a_repeat_changes_nothing_and_answers_the_same_shape() {
    let app = TestApp::spawn().await;
    let (_, access) = signed_in(&app, "someone@example.com").await;
    let export = Uuid::now_v7();

    let first = token_claims(post_receipt(&app, &access, export).await).await;
    let before = rows(&app).await;
    let second = token_claims(post_receipt(&app, &access, export).await).await;
    assert_eq!(second.free_exports_remaining, first.free_exports_remaining);
    assert_eq!(rows(&app).await, before);
    assert_eq!(before.len(), 1);
}

#[tokio::test]
async fn another_users_export_id_changes_nothing_of_theirs_or_mine() {
    let app = TestApp::spawn().await;
    let (first, first_access) = signed_in(&app, "first@example.com").await;
    let (_, second_access) = signed_in(&app, "second@example.com").await;
    let export = Uuid::now_v7();
    token_claims(post_receipt(&app, &first_access, export).await).await;

    let second = token_claims(post_receipt(&app, &second_access, export).await).await;
    assert_eq!(second.free_exports_remaining.get(), 3);
    assert_eq!(rows(&app).await, [(export, first.get(), "free".to_owned())]);
}

#[tokio::test]
async fn a_receipt_over_the_limit_is_accepted_and_answers_a_token() {
    let app = TestApp::spawn().await;
    let (_, access) = signed_in(&app, "someone@example.com").await;
    for left in [2, 1, 0, 0, 0] {
        let claims = token_claims(post_receipt(&app, &access, Uuid::now_v7()).await).await;
        assert_eq!(claims.free_exports_remaining.get(), left);
    }
    assert_eq!(rows(&app).await.len(), 5);
}

#[tokio::test]
async fn a_receipt_needs_a_session_and_a_user() {
    let app = TestApp::spawn().await;
    let url = format!("{}/usage/receipts", app.base_url);
    let body = json!({ "export_id": Uuid::now_v7() });
    let none = app.client.post(&url).json(&body).send().await.unwrap();
    assert_eq!(none.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(rows(&app).await.len(), 0);

    let (_, access) = signed_in(&app, "someone@example.com").await;
    sqlx::query("DELETE FROM users")
        .execute(&app.pool)
        .await
        .unwrap();
    let gone = post_receipt(&app, &access, Uuid::now_v7()).await;
    assert_eq!(gone.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        gone.json::<ApiError>().await.unwrap().code,
        ApiErrorCode::SessionExpired
    );
}

#[tokio::test]
async fn what_is_not_an_export_id_is_a_bad_request() {
    let app = TestApp::spawn().await;
    let (_, access) = signed_in(&app, "someone@example.com").await;
    let url = format!("{}/usage/receipts", app.base_url);
    for body in [
        json!({ "export_id": "not-a-uuid" }),
        json!({}),
        json!({ "export_id": Uuid::now_v7(), "file_name": "a.mp4" }),
    ] {
        let request = app.client.post(&url).bearer_auth(&access).json(&body);
        let response = request.send().await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{body}");
    }
    assert_eq!(rows(&app).await.len(), 0);
}

#[tokio::test]
async fn a_creator_above_fair_use_is_logged_once_a_day_and_nothing_else_changes() {
    let logs = captured_logs();
    let app = TestApp::spawn().await;
    let (user, access) = signed_in(&app, "heavy@example.com").await;
    subscribe(&app, user).await;
    let tag = user_tag(user);
    sqlx::query(
        "INSERT INTO usage_receipts (export_id, user_id, plan_at_export)
         SELECT gen_random_uuid(), $1, 'creator' FROM generate_series(1, 99)",
    )
    .bind(user.get())
    .execute(&app.pool)
    .await
    .unwrap();

    let warnings = || {
        let text = String::from_utf8(logs.0.lock().unwrap().clone()).unwrap();
        text.lines()
            .filter(|line| line.contains("fair_use_exceeded") && line.contains(&tag))
            .count()
    };

    // The 100th is at the ceiling, not above it.
    token_claims(post_receipt(&app, &access, Uuid::now_v7()).await).await;
    assert_eq!(warnings(), 0);

    for _ in 0..3 {
        let claims = token_claims(post_receipt(&app, &access, Uuid::now_v7()).await).await;
        assert_eq!(claims.plan, Plan::Creator);
    }
    assert_eq!(warnings(), 1);
    assert_eq!(rows(&app).await.len(), 103);
}
