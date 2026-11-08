#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! V6.1 (v6.1implementation D-411): the log line of `POST /transcribe` says
//! whether a request was a window of a long source, and never its index. A
//! binary of its own, with one test, as `log_redaction.rs` is: a log
//! subscriber is process-wide, and tests in parallel would race it.

mod common;

use std::io::Write;
use std::sync::{Arc, Mutex};

use common::TestApp;
use reqwest::StatusCode;
use serde_json::{Value, json};
use tracing_subscriber::fmt::MakeWriter;

/// 3 s of a 440 Hz tone, written by `ffmpeg` with `libopus` (P161).
const TONE: &[u8] = include_bytes!("fixtures/ogg/ffmpeg_tone_3s.ogg");

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

async fn signed_in(app: &TestApp, email: &str) -> String {
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
    let body: Value = response.json().await.unwrap();
    body["access_token"].as_str().unwrap().to_owned()
}

async fn post(app: &TestApp, token: &str, window: Option<&str>) -> reqwest::Response {
    let mut request = app
        .client
        .post(format!("{}/transcribe", app.base_url))
        .bearer_auth(token)
        .header("content-type", "audio/ogg")
        .header("x-forwarded-for", "192.0.2.15")
        .body(TONE.to_vec());
    if let Some(window) = window {
        request = request.header("x-offcut-window", window);
    }
    request.send().await.unwrap()
}

#[tokio::test]
async fn the_log_line_says_a_window_was_sent_and_never_its_index() {
    let logs = Captured::default();
    tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::INFO)
        .with_writer(logs.clone())
        .init();

    let app = TestApp::spawn().await;
    let token = signed_in(&app, "w4@example.com").await;
    assert_eq!(
        post(&app, &token, Some("27/34")).await.status(),
        StatusCode::OK
    );
    assert_eq!(post(&app, &token, None).await.status(), StatusCode::OK);
    assert_eq!(
        post(&app, &token, Some("99/34")).await.status(),
        StatusCode::BAD_REQUEST
    );

    let text = String::from_utf8(logs.0.lock().unwrap().clone()).unwrap();
    let lines: Vec<&str> = text
        .lines()
        .filter(|line| line.contains("\"message\":\"transcribe\""))
        .collect();
    assert_eq!(lines.len(), 3, "{text}");
    assert!(
        lines
            .iter()
            .any(|l| l.contains("\"kind\":\"ok\"") && l.contains("\"window\":true"))
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains("\"kind\":\"ok\"") && l.contains("\"window\":false"))
    );
    assert!(lines.iter().any(|l| l.contains("\"kind\":\"bad_window\"")));
    assert!(!text.contains("27/34") && !text.contains("99/34"), "{text}");
}
