#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! `POST /billing/webhook` (C-19): believed only when signed, applied once, and
//! never a reason for the merchant to retry what can never apply.

mod common;

use common::{FAKE_SIGNATURE, TestApp, webhook};
use ed25519_dalek::SigningKey;
use offcut_api::entitlement::issue::issue;
use offcut_api_types::{ApiError, ApiErrorCode};
use offcut_types::{Plan, UserId};
use reqwest::StatusCode;
use serde_json::{Value, json};
use time::OffsetDateTime;
use uuid::Uuid;

const AT: i64 = 1_760_000_000;
const END: i64 = AT + 1_000_000;

async fn new_user(app: &TestApp) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO users (id, email, email_normalized) VALUES ($1, $2, $2)")
        .bind(id)
        .bind(format!("{id}@example.com"))
        .execute(&app.pool)
        .await
        .unwrap();
    id
}

fn activated(id: &str, user: Uuid) -> Value {
    json!({
        "id": id,
        "user": user,
        "customer": format!("cus_{user}"),
        "subscription": format!("sub_{user}"),
        "kind": "activated",
        "interval": "monthly",
        "period_end": END,
        "at": AT,
    })
}

async fn events(app: &TestApp) -> Vec<String> {
    sqlx::query_scalar("SELECT provider_event_id FROM webhook_events ORDER BY provider_event_id")
        .fetch_all(&app.pool)
        .await
        .unwrap()
}

async fn subscriptions(app: &TestApp) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM subscriptions")
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

async fn period_end(app: &TestApp, user: Uuid) -> i64 {
    let end: OffsetDateTime =
        sqlx::query_scalar("SELECT current_period_end FROM subscriptions WHERE user_id = $1")
            .bind(user)
            .fetch_one(&app.pool)
            .await
            .unwrap();
    end.unix_timestamp()
}

#[tokio::test]
async fn a_bad_signature_is_401_and_nothing_is_stored() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    let url = format!("{}/billing/webhook", app.base_url);
    let body = activated("evt_1", user);

    let none = app.client.post(&url).json(&body).send().await.unwrap();
    assert_eq!(none.status(), StatusCode::UNAUTHORIZED);
    let wrong = app
        .client
        .post(&url)
        .header(FAKE_SIGNATURE, "wrong")
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        wrong.json::<ApiError>().await.unwrap().code,
        ApiErrorCode::Unauthorized
    );

    assert!(events(&app).await.is_empty());
    assert_eq!(subscriptions(&app).await, 0);
}

#[tokio::test]
async fn an_event_applies_and_the_next_token_says_creator() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    let response = webhook(&app, &activated("evt_1", user)).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().await.unwrap(), "");
    assert_eq!(events(&app).await, ["evt_1"]);

    let key = SigningKey::from_bytes(&[1; 32]);
    let before_end = OffsetDateTime::from_unix_timestamp(END - 1).unwrap();
    let issued = issue(&app.pool, &key, UserId::new(user), before_end)
        .await
        .unwrap();
    assert_eq!(issued.plan, Plan::Creator);
}

#[tokio::test]
async fn a_replay_is_200_and_changes_nothing() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    let first = activated("evt_1", user);
    assert_eq!(webhook(&app, &first).await.status(), StatusCode::OK);

    // The same id, with a later date: if it applied, the date would move.
    let mut replay = first.clone();
    replay["period_end"] = json!(END + 5);
    replay["at"] = json!(AT + 5);
    assert_eq!(webhook(&app, &replay).await.status(), StatusCode::OK);

    assert_eq!(period_end(&app, user).await, END);
    assert_eq!(events(&app).await, ["evt_1"]);
}

#[tokio::test]
async fn two_deliveries_of_one_event_at_once_apply_it_once() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    let body = activated("evt_1", user);
    let (one, two) = tokio::join!(webhook(&app, &body), webhook(&app, &body));
    assert_eq!(one.status(), StatusCode::OK);
    assert_eq!(two.status(), StatusCode::OK);
    assert_eq!(events(&app).await, ["evt_1"]);
    assert_eq!(subscriptions(&app).await, 1);
}

#[tokio::test]
async fn an_event_of_no_interest_is_200_and_stores_nothing() {
    let app = TestApp::spawn().await;
    let response = webhook(&app, &json!({ "ignore": true })).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(events(&app).await.is_empty());
}

#[tokio::test]
async fn a_signed_body_that_cannot_be_read_is_400_so_the_merchant_retries() {
    let app = TestApp::spawn().await;
    let response = app
        .client
        .post(format!("{}/billing/webhook", app.base_url))
        .header(FAKE_SIGNATURE, "good")
        .header("content-type", "application/json")
        .body("not json")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(events(&app).await.is_empty());
}

/// D-272: deleting an account cancels its subscription, so this always comes.
#[tokio::test]
async fn a_canceled_for_a_user_who_is_gone_is_200_with_the_event_kept() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    assert_eq!(
        webhook(&app, &activated("evt_1", user)).await.status(),
        StatusCode::OK
    );
    sqlx::query("DELETE FROM users")
        .execute(&app.pool)
        .await
        .unwrap();

    let mut canceled = activated("evt_2", user);
    canceled["kind"] = json!("canceled");
    canceled["at"] = json!(AT + 1);
    let response = webhook(&app, &canceled).await;
    assert_eq!(response.status(), StatusCode::OK);

    assert_eq!(events(&app).await, ["evt_1", "evt_2"]);
    assert_eq!(subscriptions(&app).await, 0);

    // Its replay is the same answer.
    assert_eq!(webhook(&app, &canceled).await.status(), StatusCode::OK);
}

/// The dedupe insert and the apply are one transaction: a failed apply leaves no
/// event row, so the merchant's retry still applies it.
#[tokio::test]
async fn a_failed_apply_leaves_no_event_row_and_the_retry_applies_it() {
    let app = TestApp::spawn().await;
    let (user, other) = (new_user(&app).await, new_user(&app).await);

    // `provider_subscription_id` is unique, so this makes the apply fail.
    let taken = activated("evt_other", other);
    let mut body = activated("evt_1", user);
    body["subscription"] = taken["subscription"].clone();
    assert_eq!(webhook(&app, &taken).await.status(), StatusCode::OK);

    let failed = webhook(&app, &body).await;
    assert_eq!(failed.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(events(&app).await, ["evt_other"]);
    assert_eq!(subscriptions(&app).await, 1);

    // The obstacle goes; the same event is delivered again and applies.
    sqlx::query("DELETE FROM subscriptions")
        .execute(&app.pool)
        .await
        .unwrap();
    assert_eq!(webhook(&app, &body).await.status(), StatusCode::OK);
    assert_eq!(events(&app).await, ["evt_1", "evt_other"]);
    assert_eq!(subscriptions(&app).await, 1);
}

#[tokio::test]
async fn a_body_up_to_64_kb_is_read_and_one_over_is_refused() {
    let app = TestApp::spawn().await;
    let url = format!("{}/billing/webhook", app.base_url);
    let of_size = |bytes: usize| format!("\"{}\"", "a".repeat(bytes - 2));
    let send = |body: String| {
        app.client
            .post(&url)
            .header(FAKE_SIGNATURE, "good")
            .header("content-type", "application/json")
            .body(body)
            .send()
    };
    // Read, and not an event the fake can parse.
    let at_limit = send(of_size(64 * 1024)).await.unwrap();
    assert_eq!(at_limit.status(), StatusCode::BAD_REQUEST);
    let over = send(of_size(64 * 1024 + 1)).await.unwrap();
    assert_eq!(over.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(
        over.json::<ApiError>().await.unwrap().code,
        ApiErrorCode::PayloadTooLarge
    );
}

#[tokio::test]
async fn the_route_answers_post_only() {
    let app = TestApp::spawn().await;
    let url = format!("{}/billing/webhook", app.base_url);
    let response = app.client.get(url).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_routes_own_answers_never_echo_the_body() {
    let app = TestApp::spawn().await;
    let response = app
        .client
        .post(format!("{}/billing/webhook", app.base_url))
        .header("content-type", "application/json")
        .body("{\"marker\":\"echo-me\"}")
        .send()
        .await
        .unwrap();
    assert!(!response.text().await.unwrap().contains("echo-me"));
}
