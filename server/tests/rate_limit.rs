#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! TS §22.1's limits, end to end through the real routes. `rate_limit.rs`'s own
//! tests hold the table and the bucket. The three billing routes join in
//! Prompts 143 and 144.

mod common;

use std::future::Future;

use common::TestApp;
use offcut_api_types::{ApiError, ApiErrorCode};
use reqwest::{RequestBuilder, Response, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;

const ORIGIN: &str = "http://localhost:5173";

/// A bucket refills as a test runs, so a route may give a few more than its
/// capacity on a slow machine, and no fewer.
const REFILL_ALLOWANCE: usize = 5;

/// `capacity` requests pass, and a 429 with its wait comes soon after.
async fn holds_to<F, Fut>(route: &str, capacity: usize, mut send: F)
where
    F: FnMut(usize) -> Fut,
    Fut: Future<Output = Response>,
{
    for n in 0..capacity {
        let response = send(n).await;
        assert_ne!(
            response.status(),
            StatusCode::TOO_MANY_REQUESTS,
            "{route}: request {n} of {capacity}"
        );
    }
    for n in capacity..capacity + REFILL_ALLOWANCE {
        let response = send(n).await;
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            let header: u32 = response.headers()["retry-after"]
                .to_str()
                .unwrap()
                .parse()
                .unwrap();
            let body: ApiError = response.json().await.unwrap();
            assert_eq!(body.code, ApiErrorCode::RateLimited, "{route}");
            assert_eq!(body.retry_after_secs, Some(header), "{route}");
            assert!(header > 0, "{route}");
            return;
        }
    }
    panic!("{route}: no 429 after {capacity} requests");
}

/// From a client address of our own, so that two cases never share a bucket.
fn from(request: RequestBuilder, ip: &str) -> RequestBuilder {
    request.header("x-forwarded-for", ip)
}

async fn signed_in(app: &TestApp, email: &str) -> String {
    let request = from(
        app.client.post(format!("{}/auth/magic-link", app.base_url)),
        "198.51.100.1",
    );
    let response = request
        .json(&json!({ "email": email }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let link = app.mailer.sent.lock().unwrap().last().unwrap().1.clone();
    let token = link.rsplit_once("token=").unwrap().1.to_owned();
    let request = from(
        app.client.post(format!("{}/auth/verify", app.base_url)),
        "198.51.100.1",
    );
    let response = request
        .json(&json!({ "token": token }))
        .send()
        .await
        .unwrap();
    let body: Value = response.json().await.unwrap();
    body["access_token"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn the_routes_limited_by_address_hold_to_their_limits() {
    let app = TestApp::spawn().await;
    let url = |path: &str| format!("{}{path}", app.base_url);

    holds_to("/healthz", 60, |_| {
        from(app.client.get(url("/healthz")), "203.0.113.1")
            .send()
            .map_ok()
    })
    .await;
    holds_to("/events", 60, |_| {
        let request = from(app.client.post(url("/events")), "203.0.113.2");
        request.json(&json!({})).send().map_ok()
    })
    .await;
    holds_to("/notify-me", 5, |n| {
        let request = from(app.client.post(url("/notify-me")), "203.0.113.3");
        let body = json!({ "email": format!("n{n}@example.com"), "wanted": "launch" });
        request.json(&body).send().map_ok()
    })
    .await;
    holds_to("/auth/magic-link by address", 10, |n| {
        let request = from(app.client.post(url("/auth/magic-link")), "203.0.113.4");
        let body = json!({ "email": format!("n{n}@example.com") });
        request.json(&body).send().map_ok()
    })
    .await;
    holds_to("/auth/verify", 10, |_| {
        let request = from(app.client.post(url("/auth/verify")), "203.0.113.5");
        request
            .json(&json!({ "token": "never-issued" }))
            .send()
            .map_ok()
    })
    .await;
    for route in ["/auth/refresh", "/auth/logout"] {
        let ip = if route == "/auth/refresh" {
            "203.0.113.6"
        } else {
            "203.0.113.7"
        };
        holds_to(route, 30, |_| {
            from(app.client.post(url(route)), ip)
                .header("x-offcut-csrf", "1")
                .header("origin", ORIGIN)
                .send()
                .map_ok()
        })
        .await;
    }
}

#[tokio::test]
async fn a_sign_in_link_is_limited_by_its_address_whatever_the_client_address() {
    let app = TestApp::spawn().await;
    let url = format!("{}/auth/magic-link", app.base_url);
    let send = |email: &'static str, ip: String| {
        let request = from(app.client.post(&url), &ip);
        request.json(&json!({ "email": email })).send().map_ok()
    };

    holds_to("/auth/magic-link by email", 3, |n| {
        send("someone@example.com", format!("203.0.113.{}", n + 10))
    })
    .await;

    // Another address is not held back, from a client address already used.
    let other = send("other@example.com", "203.0.113.10".to_owned()).await;
    assert_eq!(other.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn the_routes_limited_by_user_hold_to_their_limits_for_each_user() {
    let app = TestApp::spawn().await;
    let access = signed_in(&app, "one@example.com").await;
    let other = signed_in(&app, "two@example.com").await;
    let url = |path: &str| format!("{}{path}", app.base_url);

    holds_to("/me", 60, |_| {
        app.client
            .get(url("/me"))
            .bearer_auth(&access)
            .send()
            .map_ok()
    })
    .await;
    holds_to("/entitlement", 60, |_| {
        app.client
            .get(url("/entitlement"))
            .bearer_auth(&access)
            .send()
            .map_ok()
    })
    .await;
    holds_to("/usage/receipts", 30, |_| {
        let request = app.client.post(url("/usage/receipts")).bearer_auth(&access);
        request
            .json(&json!({ "export_id": Uuid::now_v7() }))
            .send()
            .map_ok()
    })
    .await;
    holds_to("/account/export", 6, |_| {
        app.client
            .get(url("/account/export"))
            .bearer_auth(&access)
            .send()
            .map_ok()
    })
    .await;

    // Another user's buckets are their own.
    let theirs = app
        .client
        .get(url("/me"))
        .bearer_auth(&other)
        .send()
        .await
        .unwrap();
    assert_eq!(theirs.status(), StatusCode::OK);

    // Last, since it ends the account; a repeat answers the same.
    holds_to("/account/delete", 3, |_| {
        app.client
            .post(url("/account/delete"))
            .bearer_auth(&access)
            .send()
            .map_ok()
    })
    .await;
}

/// A sent request as a plain response: every case here expects an answer.
trait AnswerOf {
    fn map_ok(self) -> impl Future<Output = Response>;
}

impl<F: Future<Output = Result<Response, reqwest::Error>>> AnswerOf for F {
    async fn map_ok(self) -> Response {
        self.await.unwrap()
    }
}
