#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! `GET /healthz`, `POST /notify-me` and the 404 fallback, against a real
//! Postgres (v1implementation §13.3, D-12).

mod common;

use common::TestApp;
use reqwest::{Response, StatusCode};
use serde_json::{Value, json};

async fn notify(app: &TestApp, body: &Value) -> Response {
    let request = app.client.post(format!("{}/notify-me", app.base_url));
    let request = request.header("content-type", "application/json");
    request.body(body.to_string()).send().await.unwrap()
}

/// The waitlist, as `(email_normalized, wanted)` pairs in a fixed order.
async fn waitlist(app: &TestApp) -> Vec<(String, String)> {
    let rows = "SELECT email_normalized, wanted FROM platform_waitlist ORDER BY 1, 2";
    sqlx::query_as(rows).fetch_all(&app.pool).await.unwrap()
}

fn pair(email: &str, wanted: &str) -> (String, String) {
    (email.to_owned(), wanted.to_owned())
}

async fn error_body(response: Response) -> Value {
    serde_json::from_str(&response.text().await.unwrap()).unwrap()
}

#[tokio::test]
async fn healthz_answers_with_the_version_and_the_api_headers() {
    let app = TestApp::spawn().await;
    let response = app.client.get(format!("{}/healthz", app.base_url));
    let response = response.send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers().clone();
    assert_eq!(headers["cache-control"], "no-store");
    assert_eq!(headers["x-content-type-options"], "nosniff");
    assert!(headers.contains_key("x-request-id"));
    let cors: Vec<_> = headers
        .keys()
        .filter(|name| name.as_str().starts_with("access-control-"))
        .collect();
    assert!(cors.is_empty(), "{cors:?}");

    let body: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(body, json!({"ok": true, "version": "test-sha"}));
}

#[tokio::test]
async fn a_valid_email_is_stored_once_per_wanted_value() {
    let app = TestApp::spawn().await;
    let launch = json!({"email": "someone@example.com", "wanted": "launch"});

    assert_eq!(notify(&app, &launch).await.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        waitlist(&app).await,
        [pair("someone@example.com", "launch")]
    );

    // The same request again: the same answer, and still one row.
    let again = notify(&app, &launch).await;
    assert_eq!(again.status(), StatusCode::NO_CONTENT);
    assert_eq!(again.text().await.unwrap(), "");
    assert_eq!(
        waitlist(&app).await,
        [pair("someone@example.com", "launch")]
    );

    // The same email waiting for something else: a second row.
    let safari = json!({"email": "someone@example.com", "wanted": "safari"});
    assert_eq!(notify(&app, &safari).await.status(), StatusCode::NO_CONTENT);
    let expected = [
        pair("someone@example.com", "launch"),
        pair("someone@example.com", "safari"),
    ];
    assert_eq!(waitlist(&app).await, expected);
}

#[tokio::test]
async fn the_email_is_stored_normalized() {
    let app = TestApp::spawn().await;
    let body = json!({"email": "  Someone@Example.COM ", "wanted": "launch"});
    assert_eq!(notify(&app, &body).await.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        waitlist(&app).await,
        [pair("someone@example.com", "launch")]
    );

    // A different spelling of the same address is the same row.
    let body = json!({"email": "SOMEONE@example.com", "wanted": "launch"});
    assert_eq!(notify(&app, &body).await.status(), StatusCode::NO_CONTENT);
    assert_eq!(waitlist(&app).await.len(), 1);
}

#[tokio::test]
async fn every_wanted_value_fits_the_column_check() {
    let app = TestApp::spawn().await;
    let all = ["launch", "safari", "firefox", "mobile", "linux"];
    for wanted in all {
        let body = json!({"email": "someone@example.com", "wanted": wanted});
        assert_eq!(
            notify(&app, &body).await.status(),
            StatusCode::NO_CONTENT,
            "{wanted}"
        );
    }
    let stored: Vec<String> = waitlist(&app).await.into_iter().map(|row| row.1).collect();
    assert_eq!(stored, ["firefox", "launch", "linux", "mobile", "safari"]);
}

#[tokio::test]
async fn an_email_that_does_not_normalize_is_400_and_stores_nothing() {
    let app = TestApp::spawn().await;
    let over_length = format!("{}@example.com", "a".repeat(250));
    let refused = [
        "someone.example.com",
        over_length.as_str(),
        "someone@example.com\nbcc: other@example.com",
    ];
    for email in refused {
        let response = notify(&app, &json!({"email": email, "wanted": "launch"})).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{email:?}");
        let body = error_body(response).await;
        // The body names the error and echoes nothing of the request.
        assert_eq!(
            body,
            json!({"code": "bad_request", "retry_after_secs": null})
        );
    }
    assert!(waitlist(&app).await.is_empty());
}

#[tokio::test]
async fn an_unknown_wanted_value_or_an_extra_field_is_400() {
    let app = TestApp::spawn().await;
    let refused = [
        json!({"email": "someone@example.com", "wanted": "edge"}),
        json!({"email": "someone@example.com", "wanted": "launch", "note": "hello"}),
        json!({"email": "someone@example.com"}),
        json!({"wanted": "launch"}),
    ];
    for body in refused {
        let response = notify(&app, &body).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(error_body(response).await["code"], "bad_request");
    }
    assert!(waitlist(&app).await.is_empty());
}

#[tokio::test]
async fn a_body_over_1_kb_is_413() {
    let app = TestApp::spawn().await;
    // A request that would be valid but for its size: JSON allows whitespace.
    let padded = format!(
        r#"{{"email":"someone@example.com",{}"wanted":"launch"}}"#,
        " ".repeat(1024)
    );
    let request = app.client.post(format!("{}/notify-me", app.base_url));
    let request = request.header("content-type", "application/json");
    let response = request.body(padded).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(error_body(response).await["code"], "payload_too_large");
    assert!(waitlist(&app).await.is_empty());
}

#[tokio::test]
async fn the_sixth_request_from_one_ip_within_the_hour_is_429() {
    let app = TestApp::spawn().await;
    let body = json!({"email": "someone@example.com", "wanted": "launch"});
    for n in 1..=5 {
        assert_eq!(
            notify(&app, &body).await.status(),
            StatusCode::NO_CONTENT,
            "request {n}"
        );
    }

    let sixth = notify(&app, &body).await;
    assert_eq!(sixth.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry_after: u32 = sixth.headers()["retry-after"]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    // Five per hour: the next token is at most 720 s away.
    assert!((1..=720).contains(&retry_after), "{retry_after}");
    let body = error_body(sixth).await;
    assert_eq!(
        body,
        json!({"code": "rate_limited", "retry_after_secs": retry_after})
    );

    // The limit is per route group: the same IP can still reach /healthz.
    let healthz = app.client.get(format!("{}/healthz", app.base_url));
    assert_eq!(healthz.send().await.unwrap().status(), StatusCode::OK);
}

#[tokio::test]
async fn an_unknown_path_under_api_v1_is_404_not_found() {
    let app = TestApp::spawn().await;
    for path in ["/nope", "/notify-me/extra", "/auth/magic-link"] {
        let response = app.client.get(format!("{}{path}", app.base_url));
        let response = response.send().await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
        assert_eq!(response.headers()["cache-control"], "no-store");
        let body = error_body(response).await;
        assert_eq!(body, json!({"code": "not_found", "retry_after_secs": null}));
    }
}
