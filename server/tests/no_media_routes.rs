#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The API takes no media and no upload on any route (INV-4, TS §22.1). The
//! routes are read from `router.rs` itself, so a new one is walked without
//! anyone remembering to add it here.

mod common;

use common::TestApp;
use offcut_api_types::{ApiError, ApiErrorCode};
use reqwest::{Method, StatusCode};
use serde_json::{Value, json};

const ROUTER: &str = include_str!("../src/router.rs");
const MAX_BODY: usize = 64 * 1024;
const MEDIA_TYPES: [&str; 5] = [
    "multipart/form-data; boundary=x",
    "application/octet-stream",
    "video/mp4",
    "audio/wav",
    "AUDIO/OGG; codecs=opus",
];

/// The POST routes that read a body. Each one must refuse more than 64 kB. A new
/// POST route fails `every_post_route_refuses_a_body_over_the_limit` until it
/// is named here or in `IGNORE_BODY`.
const READ_BODY: [&str; 6] = [
    "/billing/webhook",
    "/auth/magic-link",
    "/auth/verify",
    "/usage/receipts",
    "/events",
    "/notify-me",
];
/// The POST routes that read no body at all.
const IGNORE_BODY: [&str; 3] = ["/auth/refresh", "/auth/logout", "/account/delete"];

/// (path, method) of each `.route(...)` of the table, read from the source.
fn routes() -> Vec<(String, Method)> {
    let table = ROUTER.split("#[cfg(test)]").next().unwrap();
    let mut routes = Vec::new();
    for part in table.split(".route(").skip(1) {
        let path = part.split('"').nth(1).unwrap().to_owned();
        let after = part
            .split('"')
            .nth(2)
            .unwrap()
            .trim_start_matches([',', ' ', '\r', '\n']);
        let method = if after.starts_with("get(") {
            Method::GET
        } else if after.starts_with("post(") {
            Method::POST
        } else {
            panic!("the route {path} has a method this test does not read: {after}");
        };
        routes.push((path, method));
    }
    routes
}

#[test]
fn the_table_is_read() {
    let routes = routes();
    assert!(routes.len() >= 12, "{routes:?}");
    assert!(routes.contains(&("/healthz".to_owned(), Method::GET)));
    assert!(routes.contains(&("/auth/verify".to_owned(), Method::POST)));
    assert!(routes.contains(&("/me".to_owned(), Method::GET)));
}

#[tokio::test]
async fn every_route_refuses_media_and_uploads() {
    let app = TestApp::spawn().await;
    for (path, method) in routes() {
        for media in MEDIA_TYPES {
            let url = format!("{}{path}", app.base_url);
            let request = app.client.request(method.clone(), url);
            let response = request
                .header("content-type", media)
                .body("x")
                .send()
                .await
                .unwrap();
            let context = format!("{method} {path} with {media}");
            assert_eq!(
                response.status(),
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "{context}"
            );
            let body: ApiError = response.json().await.unwrap();
            assert_eq!(body.code, ApiErrorCode::UnsupportedMediaType, "{context}");
        }
    }
    // The refused requests reached no handler.
    assert!(app.mailer.sent.lock().unwrap().is_empty());
}

#[tokio::test]
async fn every_post_route_refuses_a_body_over_the_limit() {
    let app = TestApp::spawn().await;
    let access = signed_in(&app).await;
    let over = format!("{{\"x\":\"{}\"}}", "a".repeat(MAX_BODY));
    assert!(over.len() > MAX_BODY);

    for (path, method) in routes() {
        if method != Method::POST {
            continue;
        }
        let url = format!("{}{path}", app.base_url);
        if READ_BODY.contains(&path.as_str()) {
            let response = app
                .client
                .post(url)
                .bearer_auth(&access)
                .header("content-type", "application/json")
                .body(over.clone())
                .send()
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE, "{path}");
            let body: ApiError = response.json().await.unwrap();
            assert_eq!(body.code, ApiErrorCode::PayloadTooLarge, "{path}");
        } else if IGNORE_BODY.contains(&path.as_str()) {
            // No credential: it is refused, and it never looks at what was sent.
            let response = app
                .client
                .post(url)
                .header("content-type", "application/json")
                .body(over.clone())
                .send()
                .await
                .unwrap();
            assert!(
                response.status().is_client_error(),
                "{path}: {}",
                response.status()
            );
        } else {
            panic!("the POST route {path} must be named as reading a body or not");
        }
    }
}

async fn signed_in(app: &TestApp) -> String {
    let request = app.client.post(format!("{}/auth/magic-link", app.base_url));
    let response = request
        .json(&json!({ "email": "a@example.com" }))
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
