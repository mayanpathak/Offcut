//! Sign-in end to end, against a real Postgres with a fake mailer (TS §22.1,
//! §22.2; v6implementation §7, §15.1). Refresh and logout arrive with Prompt 140.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod common;

use std::sync::atomic::Ordering;

use common::TestApp;
use ed25519_dalek::SigningKey;
use offcut_api::auth::access_token;
use offcut_api::auth::session::token_hash;
use offcut_api_types::{ApiError, ApiErrorCode, SessionResponse};
use offcut_entitlement::verify_token;
use offcut_types::{Plan, UnixSecs};
use reqwest::{Response, StatusCode};
use serde_json::json;
use time::OffsetDateTime;

const EMAIL: &str = "Someone@Example.com ";
const NORMALIZED: &str = "someone@example.com";

/// The test app's keys (`test_config`).
fn access_key() -> SigningKey {
    SigningKey::from_bytes(&[2; 32])
}

fn entitlement_key() -> SigningKey {
    SigningKey::from_bytes(&[1; 32])
}

fn now() -> UnixSecs {
    UnixSecs::new(OffsetDateTime::now_utc().unix_timestamp())
}

async fn request_link(app: &TestApp, email: &str) -> Response {
    let request = app.client.post(format!("{}/auth/magic-link", app.base_url));
    request
        .json(&json!({ "email": email }))
        .send()
        .await
        .unwrap()
}

async fn post_verify(app: &TestApp, token: &str) -> Response {
    let request = app.client.post(format!("{}/auth/verify", app.base_url));
    request
        .json(&json!({ "token": token }))
        .send()
        .await
        .unwrap()
}

/// The token of the last link the fake mailer was given.
fn last_token(app: &TestApp) -> String {
    let sent = app.mailer.sent.lock().unwrap();
    let (_, link) = sent.last().expect("a mail was sent");
    link.rsplit_once("token=").unwrap().1.to_owned()
}

async fn signed_up(app: &TestApp) -> String {
    let response = request_link(app, EMAIL).await;
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    last_token(app)
}

async fn error_code(response: Response) -> ApiErrorCode {
    response.json::<ApiError>().await.unwrap().code
}

async fn count(app: &TestApp, table: &str) -> i64 {
    let sql = format!("SELECT count(*) FROM {table}");
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn a_link_is_mailed_to_the_normalized_address_and_only_its_hash_is_stored() {
    let app = TestApp::spawn().await;
    let token = signed_up(&app).await;

    let sent = app.mailer.sent.lock().unwrap().clone();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].0, NORMALIZED);
    assert_eq!(
        sent[0].1,
        format!("http://localhost:5173/auth/callback?token={token}")
    );
    assert_eq!(token.len(), 43);

    let (hash, email): (Vec<u8>, String) =
        sqlx::query_as("SELECT token_hash, email_normalized FROM magic_links")
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(hash, token_hash(&token));
    assert_ne!(hash, token.as_bytes());
    assert_eq!(email, NORMALIZED);
}

#[tokio::test]
async fn a_request_answers_alike_for_a_known_and_an_unknown_address() {
    let app = TestApp::spawn().await;
    let token = signed_up(&app).await;
    assert_eq!(post_verify(&app, &token).await.status(), StatusCode::OK);

    let known = request_link(&app, NORMALIZED).await;
    let unknown = request_link(&app, "nobody@example.com").await;
    assert_eq!(known.status(), StatusCode::ACCEPTED);
    assert_eq!(unknown.status(), StatusCode::ACCEPTED);
    assert_eq!(known.text().await.unwrap(), "");
    assert_eq!(unknown.text().await.unwrap(), "");
    assert_eq!(count(&app, "users").await, 1);
}

#[tokio::test]
async fn what_is_not_an_address_is_a_bad_request_and_sends_nothing() {
    let app = TestApp::spawn().await;
    for email in ["", "no-at-sign", "two@@example.com"] {
        let response = request_link(&app, email).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{email}");
    }
    assert!(app.mailer.sent.lock().unwrap().is_empty());
    assert_eq!(count(&app, "magic_links").await, 0);
}

#[tokio::test]
async fn a_second_request_spends_the_first_link() {
    let app = TestApp::spawn().await;
    let first = signed_up(&app).await;
    let second = signed_up(&app).await;
    assert_ne!(first, second);

    let old = post_verify(&app, &first).await;
    assert_eq!(old.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(old).await, ApiErrorCode::LinkInvalid);
    assert_eq!(post_verify(&app, &second).await.status(), StatusCode::OK);
}

#[tokio::test]
async fn a_mail_that_fails_is_503_and_leaves_no_link() {
    let app = TestApp::spawn().await;
    app.mailer.fail.store(true, Ordering::SeqCst);
    let response = request_link(&app, EMAIL).await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(error_code(response).await, ApiErrorCode::EmailUnavailable);
    assert_eq!(count(&app, "magic_links").await, 0);
}

#[tokio::test]
async fn an_expired_link_says_so_and_makes_nothing() {
    let app = TestApp::spawn().await;
    let token = signed_up(&app).await;
    sqlx::query("UPDATE magic_links SET expires_at = now() - interval '1 second'")
        .execute(&app.pool)
        .await
        .unwrap();

    let response = post_verify(&app, &token).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(response).await, ApiErrorCode::LinkExpired);
    assert_eq!(count(&app, "users").await, 0);
    assert_eq!(count(&app, "sessions").await, 0);
}

#[tokio::test]
async fn an_unknown_or_used_link_is_invalid() {
    let app = TestApp::spawn().await;
    let unknown = post_verify(&app, "never-issued").await;
    assert_eq!(unknown.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(unknown).await, ApiErrorCode::LinkInvalid);

    let token = signed_up(&app).await;
    assert_eq!(post_verify(&app, &token).await.status(), StatusCode::OK);
    let again = post_verify(&app, &token).await;
    assert_eq!(again.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(again).await, ApiErrorCode::LinkInvalid);
    assert_eq!(count(&app, "sessions").await, 1);
}

#[tokio::test]
async fn two_racing_verifies_of_one_link_give_one_success() {
    let app = TestApp::spawn().await;
    let token = signed_up(&app).await;
    let (one, two) = tokio::join!(post_verify(&app, &token), post_verify(&app, &token));
    let mut statuses = [one.status(), two.status()];
    statuses.sort();
    assert_eq!(statuses, [StatusCode::OK, StatusCode::BAD_REQUEST]);
    assert_eq!(count(&app, "users").await, 1);
    assert_eq!(count(&app, "sessions").await, 1);
}

#[tokio::test]
async fn verify_makes_the_user_once_and_finds_it_after_that() {
    let app = TestApp::spawn().await;
    let first = signed_up(&app).await;
    assert_eq!(post_verify(&app, &first).await.status(), StatusCode::OK);
    let second = signed_up(&app).await;
    assert_eq!(post_verify(&app, &second).await.status(), StatusCode::OK);

    assert_eq!(count(&app, "users").await, 1);
    assert_eq!(count(&app, "sessions").await, 2);
    // The address as normalized: the link keeps no other (D-260).
    let (email, normalized): (String, String) =
        sqlx::query_as("SELECT email, email_normalized FROM users")
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(
        (email.as_str(), normalized.as_str()),
        (NORMALIZED, NORMALIZED)
    );
}

#[tokio::test]
async fn verify_answers_a_cookie_an_access_token_and_a_free_token() {
    let app = TestApp::spawn().await;
    let token = signed_up(&app).await;
    let response = post_verify(&app, &token).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");

    let cookie = response.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .to_owned();
    let refresh = cookie
        .strip_prefix("offcut_rt=")
        .and_then(|rest| rest.split_once(';'))
        .unwrap()
        .0
        .to_owned();
    assert_eq!(
        cookie,
        format!(
            "offcut_rt={refresh}; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=5184000"
        )
    );

    // Only the refresh token's hash is stored.
    let (hash,): (Vec<u8>,) = sqlx::query_as("SELECT refresh_token_hash FROM sessions")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(hash, token_hash(&refresh));

    let body: SessionResponse = response.json().await.unwrap();
    let claims =
        access_token::verify(&body.access_token, &access_key().verifying_key(), now()).unwrap();
    assert_eq!(claims.exp, body.access_expires_at);
    let left = claims.exp.get() - now().get();
    assert!((890..=900).contains(&left), "{left}");

    let public = entitlement_key().verifying_key().to_bytes();
    let entitlement = verify_token(&body.entitlement_token, &[public]).unwrap();
    assert_eq!(entitlement.sub, claims.sub);
    assert_eq!(entitlement.plan, Plan::Free);
    assert_eq!(entitlement.free_exports_remaining.get(), 3);
    assert_eq!(entitlement.exp.get() - entitlement.iat.get(), 7 * 24 * 3600);
}

#[tokio::test]
async fn the_routes_take_small_json_bodies_only() {
    let app = TestApp::spawn().await;
    let long = "a".repeat(2_000);
    let response = post_verify(&app, &long).await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);

    let request = app.client.post(format!("{}/auth/verify", app.base_url));
    let unknown_field = json!({ "token": "x", "extra": 1 });
    let response = request.json(&unknown_field).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
