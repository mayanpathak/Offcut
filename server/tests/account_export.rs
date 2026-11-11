#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! `GET /account/export` (TS §22.7, v6implementation D-270): the server's records
//! of a person, with no secret among them.

mod common;

use common::TestApp;
use offcut_api::db::USER_TABLES;
use offcut_api::db::ai_usage::{self, AiKind};
use offcut_api_types::{AccountExport, BillingInterval, SubscriptionStatus};
use offcut_types::{Plan, UserId};
use reqwest::{Response, StatusCode};
use serde_json::{Value, json};
use time::OffsetDateTime;
use uuid::Uuid;

const EMAIL: &str = "someone@example.com";

async fn signed_in(app: &TestApp, email: &str) -> (String, String) {
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
    let cookie = response.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .to_owned();
    let refresh = cookie
        .strip_prefix("offcut_rt=")
        .unwrap()
        .split_once(';')
        .unwrap()
        .0
        .to_owned();
    let body: Value = response.json().await.unwrap();
    (body["access_token"].as_str().unwrap().to_owned(), refresh)
}

async fn get_export(app: &TestApp, access: &str) -> Response {
    let url = format!("{}/account/export", app.base_url);
    app.client
        .get(url)
        .bearer_auth(access)
        .send()
        .await
        .unwrap()
}

async fn subscribe(app: &TestApp, email: &str) {
    sqlx::query(
        "INSERT INTO subscriptions (user_id, provider_customer_id, provider_subscription_id, plan,
           billing_interval, status, current_period_end, last_event_at)
         SELECT id, 'cus_secret_id', 'sub_' || email_normalized, 'creator', 'annual', 'active',
                now() + interval '10 days', now()
         FROM users WHERE email_normalized = $1",
    )
    .bind(email)
    .execute(&app.pool)
    .await
    .unwrap();
}

async fn receipt(app: &TestApp, email: &str, export_id: Uuid, age: &str, plan: &str) {
    let sql = format!(
        "INSERT INTO usage_receipts (export_id, user_id, plan_at_export, created_at)
         SELECT $2, id, $3, now() - interval '{age}' FROM users WHERE email_normalized = $1"
    );
    sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(email)
        .bind(export_id)
        .bind(plan)
        .execute(&app.pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn the_export_holds_the_user_sessions_subscription_and_receipts() {
    let app = TestApp::spawn().await;
    let (access, _) = signed_in(&app, EMAIL).await;
    subscribe(&app, EMAIL).await;
    let (older, newer) = (Uuid::now_v7(), Uuid::now_v7());
    receipt(&app, EMAIL, newer, "1 hour", "creator").await;
    receipt(&app, EMAIL, older, "1 day", "free").await;

    let response = get_export(&app, &access).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let export: AccountExport = response.json().await.unwrap();

    assert_eq!(export.user.email, EMAIL);
    assert!(export.user.created_at.get() > 0);
    assert_eq!(export.sessions.len(), 1);
    let session = &export.sessions[0];
    assert!(session.expires_at.get() > session.created_at.get());

    let summary = export.subscription.expect("a subscription");
    assert_eq!(summary.status, SubscriptionStatus::Active);
    assert_eq!(summary.interval, BillingInterval::Annual);

    let receipts: Vec<(Uuid, Plan)> = export
        .usage_receipts
        .iter()
        .map(|receipt| (receipt.export_id.get(), receipt.plan_at_export))
        .collect();
    assert_eq!(receipts, [(older, Plan::Free), (newer, Plan::Creator)]);
}

#[tokio::test]
async fn the_export_holds_no_token_no_hash_and_no_merchant_id() {
    let app = TestApp::spawn().await;
    let (access, refresh) = signed_in(&app, EMAIL).await;
    subscribe(&app, EMAIL).await;

    let text = get_export(&app, &access).await.text().await.unwrap();
    assert!(!text.contains(&refresh));
    assert!(!text.contains(&access));
    assert!(!text.contains("hash"), "{text}");
    assert!(!text.contains("cus_secret_id"), "{text}");
    assert!(!text.contains("sub_"), "{text}");
}

/// S-10: each table of the list reaches the export. A table this match does not
/// name fails here, until the export is given its rows.
#[tokio::test]
async fn a_row_of_every_listed_table_appears_in_the_export() {
    let app = TestApp::spawn().await;
    let (access, _) = signed_in(&app, EMAIL).await;
    subscribe(&app, EMAIL).await;
    receipt(&app, EMAIL, Uuid::now_v7(), "1 hour", "free").await;
    // V6.2 (D-440 (6), S-10): an AI plan counted and a long-video grant.
    ai_rows(&app, EMAIL).await;

    let export: Value = get_export(&app, &access).await.json().await.unwrap();
    for table in USER_TABLES {
        let holds_a_row = match *table {
            "sessions" => export["sessions"]
                .as_array()
                .is_some_and(|rows| !rows.is_empty()),
            "subscriptions" => export["subscription"].is_object(),
            "usage_receipts" => export["usage_receipts"]
                .as_array()
                .is_some_and(|rows| !rows.is_empty()),
            "ai_usage" => export["ai_usage"]
                .as_array()
                .is_some_and(|rows| !rows.is_empty()),
            "ai_shorts_grants" => export["ai_shorts_grant_months"]
                .as_array()
                .is_some_and(|rows| !rows.is_empty()),
            other => panic!("the export does not know the table {other}"),
        };
        assert!(holds_a_row, "{table}");
    }
}

#[tokio::test]
async fn a_new_account_has_empty_lists_and_no_subscription() {
    let app = TestApp::spawn().await;
    let (access, _) = signed_in(&app, EMAIL).await;
    let export: AccountExport = get_export(&app, &access).await.json().await.unwrap();
    assert_eq!(export.subscription, None);
    assert!(export.usage_receipts.is_empty());
}

#[tokio::test]
async fn the_export_is_of_this_person_only() {
    let app = TestApp::spawn().await;
    let (access, _) = signed_in(&app, EMAIL).await;
    signed_in(&app, "other@example.com").await;
    receipt(&app, "other@example.com", Uuid::now_v7(), "1 hour", "free").await;
    let export: AccountExport = get_export(&app, &access).await.json().await.unwrap();
    assert_eq!(export.sessions.len(), 1);
    assert!(export.usage_receipts.is_empty());
}

#[tokio::test]
async fn the_export_needs_a_session_and_a_user() {
    let app = TestApp::spawn().await;
    let url = format!("{}/account/export", app.base_url);
    let none = app.client.get(url).send().await.unwrap();
    assert_eq!(none.status(), StatusCode::UNAUTHORIZED);

    let (access, _) = signed_in(&app, EMAIL).await;
    sqlx::query("DELETE FROM users")
        .execute(&app.pool)
        .await
        .unwrap();
    assert_eq!(
        get_export(&app, &access).await.status(),
        StatusCode::UNAUTHORIZED
    );
}

/// V6.2: two plans counted through `db::ai_usage`, and one grant.
async fn ai_rows(app: &TestApp, email: &str) {
    let id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email_normalized = $1")
        .bind(email)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    let user = UserId::new(id);
    let month = ai_usage::month_of(OffsetDateTime::now_utc());
    for expected in [1, 2] {
        let count = ai_usage::increment(&app.pool, user, month, AiKind::Plan).await;
        assert_eq!(count.unwrap(), expected);
    }
    assert_eq!(
        ai_usage::count(&app.pool, user, month, AiKind::Plan)
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        ai_usage::count(&app.pool, user, month, AiKind::Chat)
            .await
            .unwrap(),
        0
    );
    sqlx::query("INSERT INTO ai_shorts_grants (user_id, month, prefix_hash) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(month)
        .bind(vec![0xAB_u8, 0xCD])
        .execute(&app.pool)
        .await
        .unwrap();
}

/// V6.2 (TS §52, S-10): the AI rows reach the export by month, the grant
/// without its hash; an account that never used the AI plan exports
/// exactly V6.1's fields.
#[tokio::test]
async fn the_ai_rows_are_exported_by_month_and_never_the_grants_hash() {
    let app = TestApp::spawn().await;
    let (access, _) = signed_in(&app, EMAIL).await;
    let before: Value = get_export(&app, &access).await.json().await.unwrap();
    let keys: Vec<&String> = before.as_object().unwrap().keys().collect();
    assert_eq!(
        keys,
        ["sessions", "subscription", "usage_receipts", "user"],
        "{before}"
    );

    ai_rows(&app, EMAIL).await;
    let text = get_export(&app, &access).await.text().await.unwrap();
    assert!(!text.contains("hash") && !text.contains("abcd"), "{text}");
    let export: Value = serde_json::from_str(&text).unwrap();
    let month = OffsetDateTime::now_utc();
    let month = format!("{:04}-{:02}", month.year(), u8::from(month.month()));
    assert_eq!(
        export["ai_usage"],
        serde_json::json!([{ "month": month, "kind": "plan", "count": 2 }])
    );
    assert_eq!(export["ai_shorts_grant_months"], serde_json::json!([month]));
    // The V6 fields still read as `AccountExport`.
    let typed: AccountExport = serde_json::from_str(&text).unwrap();
    assert_eq!(typed.user.email, EMAIL);
}
