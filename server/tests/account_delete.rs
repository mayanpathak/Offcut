#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! `POST /account/delete` (TS §22.7, v6implementation D-270, D-271).

mod common;

use std::sync::atomic::Ordering;

use common::TestApp;
use offcut_api::db::USER_TABLES;
use offcut_api_types::{ApiError, ApiErrorCode};
use reqwest::{Response, StatusCode};
use serde_json::json;
use uuid::Uuid;

const EMAIL: &str = "someone@example.com";
const OTHER: &str = "other@example.com";
const CLEARED: &str = "offcut_rt=; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=0";

/// A session through the real routes: the access token and the refresh token.
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
    let body: serde_json::Value = response.json().await.unwrap();
    (body["access_token"].as_str().unwrap().to_owned(), refresh)
}

async fn delete_account(app: &TestApp, access: &str) -> Response {
    let url = format!("{}/account/delete", app.base_url);
    app.client
        .post(url)
        .bearer_auth(access)
        .send()
        .await
        .unwrap()
}

async fn count(app: &TestApp, sql: &str) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql.to_owned()))
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

/// A subscription, receipts and a waitlist row for this address.
async fn rows_for(app: &TestApp, email: &str, status: &str) {
    sqlx::query(
        "INSERT INTO subscriptions (user_id, provider_customer_id, provider_subscription_id, plan,
           billing_interval, status, current_period_end, last_event_at)
         SELECT id, 'cus_' || id, 'sub_' || email_normalized, 'creator', 'monthly', $2,
                now() + interval '10 days', now()
         FROM users WHERE email_normalized = $1",
    )
    .bind(email)
    .bind(status)
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO usage_receipts (export_id, user_id, plan_at_export)
         SELECT $2, id, 'creator' FROM users WHERE email_normalized = $1",
    )
    .bind(email)
    .bind(Uuid::now_v7())
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO platform_waitlist (email_normalized, wanted) VALUES ($1, 'launch')")
        .bind(email)
        .execute(&app.pool)
        .await
        .unwrap();
}

async fn rows_of_user(app: &TestApp, table: &str, email: &str) -> i64 {
    let sql = format!(
        "SELECT count(*) FROM {table} WHERE user_id IN
           (SELECT id FROM users WHERE email_normalized = '{email}')"
    );
    count(app, &sql).await
}

#[tokio::test]
async fn deleting_removes_every_row_of_the_user_and_clears_the_cookie() {
    let app = TestApp::spawn().await;
    let (access, _) = signed_in(&app, EMAIL).await;
    rows_for(&app, EMAIL, "active").await;
    for table in USER_TABLES {
        assert!(rows_of_user(&app, table, EMAIL).await > 0, "{table}");
    }

    let response = delete_account(&app, &access).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(response.headers()["set-cookie"], CLEARED);

    assert_eq!(count(&app, "SELECT count(*) FROM users").await, 0);
    for table in USER_TABLES {
        let sql = format!("SELECT count(*) FROM {table}");
        assert_eq!(count(&app, &sql).await, 0, "{table}");
    }
    assert_eq!(count(&app, "SELECT count(*) FROM magic_links").await, 0);
    assert_eq!(
        count(&app, "SELECT count(*) FROM platform_waitlist").await,
        0
    );
}

#[tokio::test]
async fn another_accounts_rows_stay() {
    let app = TestApp::spawn().await;
    let (access, _) = signed_in(&app, EMAIL).await;
    let (_, other_refresh) = signed_in(&app, OTHER).await;
    rows_for(&app, EMAIL, "active").await;
    rows_for(&app, OTHER, "active").await;

    assert_eq!(
        delete_account(&app, &access).await.status(),
        StatusCode::NO_CONTENT
    );

    assert_eq!(count(&app, "SELECT count(*) FROM users").await, 1);
    for table in USER_TABLES {
        assert_eq!(rows_of_user(&app, table, OTHER).await, 1, "{table}");
    }
    assert_eq!(count(&app, "SELECT count(*) FROM magic_links").await, 1);
    assert_eq!(
        count(&app, "SELECT count(*) FROM platform_waitlist").await,
        1
    );
    assert!(!other_refresh.is_empty());
}

#[tokio::test]
async fn a_live_subscription_is_cancelled_at_the_merchant() {
    for status in ["active", "past_due"] {
        let app = TestApp::spawn().await;
        let (access, _) = signed_in(&app, EMAIL).await;
        rows_for(&app, EMAIL, status).await;

        assert_eq!(
            delete_account(&app, &access).await.status(),
            StatusCode::NO_CONTENT
        );
        let cancelled = app.billing.cancelled.lock().unwrap().clone();
        assert_eq!(cancelled, [format!("sub_{EMAIL}")], "{status}");
    }
}

#[tokio::test]
async fn a_merchant_that_refuses_the_cancel_is_502_and_nothing_is_deleted() {
    let app = TestApp::spawn().await;
    let (access, _) = signed_in(&app, EMAIL).await;
    rows_for(&app, EMAIL, "active").await;
    app.billing.fail.store(true, Ordering::SeqCst);

    let response = delete_account(&app, &access).await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert!(response.headers().get("set-cookie").is_none());
    assert_eq!(
        response.json::<ApiError>().await.unwrap().code,
        ApiErrorCode::BillingUnavailable
    );
    assert_eq!(count(&app, "SELECT count(*) FROM users").await, 1);
    for table in USER_TABLES {
        assert_eq!(rows_of_user(&app, table, EMAIL).await, 1, "{table}");
    }

    // The merchant is back: the person tries again.
    app.billing.fail.store(false, Ordering::SeqCst);
    assert_eq!(
        delete_account(&app, &access).await.status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(count(&app, "SELECT count(*) FROM users").await, 0);
}

#[tokio::test]
async fn no_live_subscription_means_no_call_to_the_merchant() {
    let app = TestApp::spawn().await;
    let (access, _) = signed_in(&app, EMAIL).await;
    rows_for(&app, EMAIL, "canceled").await;
    // It would fail if it were called.
    app.billing.fail.store(true, Ordering::SeqCst);

    assert_eq!(
        delete_account(&app, &access).await.status(),
        StatusCode::NO_CONTENT
    );
    assert!(app.billing.cancelled.lock().unwrap().is_empty());
    assert_eq!(count(&app, "SELECT count(*) FROM users").await, 0);
}

#[tokio::test]
async fn a_second_delete_is_the_same_answer_and_the_session_is_over() {
    let app = TestApp::spawn().await;
    let (access, refresh) = signed_in(&app, EMAIL).await;

    assert_eq!(
        delete_account(&app, &access).await.status(),
        StatusCode::NO_CONTENT
    );
    let again = delete_account(&app, &access).await;
    assert_eq!(again.status(), StatusCode::NO_CONTENT);
    assert_eq!(again.headers()["set-cookie"], CLEARED);

    let me = app
        .client
        .get(format!("{}/me", app.base_url))
        .bearer_auth(&access);
    assert_eq!(me.send().await.unwrap().status(), StatusCode::UNAUTHORIZED);
    let session = app
        .client
        .post(format!("{}/auth/refresh", app.base_url))
        .header("cookie", format!("offcut_rt={refresh}"))
        .header("x-offcut-csrf", "1")
        .header("origin", "http://localhost:5173");
    assert_eq!(
        session.send().await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn deleting_needs_a_session() {
    let app = TestApp::spawn().await;
    let url = format!("{}/account/delete", app.base_url);
    let response = app.client.post(url).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn the_address_can_sign_up_again_afterwards() {
    let app = TestApp::spawn().await;
    let (access, _) = signed_in(&app, EMAIL).await;
    assert_eq!(
        delete_account(&app, &access).await.status(),
        StatusCode::NO_CONTENT
    );
    let (again, _) = signed_in(&app, EMAIL).await;
    assert_ne!(again, access);
    assert_eq!(count(&app, "SELECT count(*) FROM users").await, 1);
}

/// S-10: a new table with a `user_id` must join the list, or this fails.
#[tokio::test]
async fn user_tables_names_every_table_with_a_user_id_column() {
    let app = TestApp::spawn().await;
    let mut tables: Vec<String> = sqlx::query_scalar(
        "SELECT table_name::text FROM information_schema.columns
         WHERE table_schema = 'public' AND column_name = 'user_id'",
    )
    .fetch_all(&app.pool)
    .await
    .unwrap();
    tables.sort();
    let mut listed: Vec<&str> = USER_TABLES.to_vec();
    listed.sort_unstable();
    assert_eq!(tables, listed);
}
