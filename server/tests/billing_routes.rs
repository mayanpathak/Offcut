#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! `POST /billing/checkout` and `GET /billing/portal`, on the fake merchant.

mod common;

use std::sync::atomic::Ordering;

use common::TestApp;
use offcut_api_types::{ApiError, ApiErrorCode, UrlResponse};
use reqwest::{Response, StatusCode};
use serde_json::{Value, json};

const EMAIL: &str = "someone@example.com";

async fn signed_in(app: &TestApp) -> String {
    let request = app.client.post(format!("{}/auth/magic-link", app.base_url));
    let response = request
        .json(&json!({ "email": EMAIL }))
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
    let body: Value = response.json().await.unwrap();
    body["access_token"].as_str().unwrap().to_owned()
}

async fn checkout(app: &TestApp, access: &str, offer: &str) -> Response {
    let url = format!("{}/billing/checkout", app.base_url);
    let request = app.client.post(url).bearer_auth(access);
    request
        .json(&json!({ "offer": offer }))
        .send()
        .await
        .unwrap()
}

async fn portal(app: &TestApp, access: &str) -> Response {
    let url = format!("{}/billing/portal", app.base_url);
    app.client
        .get(url)
        .bearer_auth(access)
        .send()
        .await
        .unwrap()
}

async fn subscribe(app: &TestApp, status: &str) {
    sqlx::query(
        "INSERT INTO subscriptions (user_id, provider_customer_id, provider_subscription_id, plan,
           billing_interval, status, current_period_end, last_event_at)
         SELECT id, 'cus', 'sub', 'creator', 'monthly', $1, now() + interval '10 days', now()
         FROM users",
    )
    .bind(status)
    .execute(&app.pool)
    .await
    .unwrap();
}

async fn code(response: Response) -> ApiErrorCode {
    response.json::<ApiError>().await.unwrap().code
}

#[tokio::test]
async fn a_checkout_answers_where_to_go() {
    let app = TestApp::spawn().await;
    let access = signed_in(&app).await;
    for offer in ["creator_monthly", "creator_annual"] {
        let response = checkout(&app, &access, offer).await;
        assert_eq!(response.status(), StatusCode::OK, "{offer}");
        let body: UrlResponse = response.json().await.unwrap();
        assert_eq!(body.url, "https://merchant.example.com/checkout");
    }
}

#[tokio::test]
async fn the_founding_offer_is_refused_while_it_is_off() {
    let app = TestApp::spawn().await;
    let access = signed_in(&app).await;
    let response = checkout(&app, &access, "creator_annual_founding").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn a_person_already_paying_is_refused_a_second_subscription() {
    for status in ["active", "past_due"] {
        let app = TestApp::spawn().await;
        let access = signed_in(&app).await;
        subscribe(&app, status).await;
        let response = checkout(&app, &access, "creator_monthly").await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{status}");
        assert_eq!(code(response).await, ApiErrorCode::Forbidden);
    }
}

#[tokio::test]
async fn a_person_whose_subscription_ended_can_buy_again() {
    let app = TestApp::spawn().await;
    let access = signed_in(&app).await;
    subscribe(&app, "canceled").await;
    let response = checkout(&app, &access, "creator_monthly").await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn a_merchant_that_fails_is_502() {
    let app = TestApp::spawn().await;
    let access = signed_in(&app).await;
    subscribe(&app, "canceled").await;
    app.billing.fail.store(true, Ordering::SeqCst);

    let response = checkout(&app, &access, "creator_monthly").await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(code(response).await, ApiErrorCode::BillingUnavailable);

    let response = portal(&app, &access).await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(code(response).await, ApiErrorCode::BillingUnavailable);
}

#[tokio::test]
async fn the_portal_is_for_someone_who_has_subscribed() {
    let app = TestApp::spawn().await;
    let access = signed_in(&app).await;
    let none = portal(&app, &access).await;
    assert_eq!(none.status(), StatusCode::NOT_FOUND);

    subscribe(&app, "active").await;
    let response = portal(&app, &access).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body: UrlResponse = response.json().await.unwrap();
    assert_eq!(body.url, "https://merchant.example.com/portal");
}

#[tokio::test]
async fn both_routes_need_a_session_and_a_user() {
    let app = TestApp::spawn().await;
    let url = format!("{}/billing/checkout", app.base_url);
    let body = json!({ "offer": "creator_monthly" });
    let none = app.client.post(&url).json(&body).send().await.unwrap();
    assert_eq!(none.status(), StatusCode::UNAUTHORIZED);
    let url = format!("{}/billing/portal", app.base_url);
    assert_eq!(
        app.client.get(&url).send().await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );

    let access = signed_in(&app).await;
    sqlx::query("DELETE FROM users")
        .execute(&app.pool)
        .await
        .unwrap();
    let gone = checkout(&app, &access, "creator_monthly").await;
    assert_eq!(gone.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn an_unknown_offer_is_a_bad_request() {
    let app = TestApp::spawn().await;
    let access = signed_in(&app).await;
    let response = checkout(&app, &access, "creator_lifetime").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
