#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! `POST /transcribe` end to end, on the real router and a real Postgres for
//! the session, with `FakeTranscriber` as the provider (cloudimplementation
//! §11.1). Every check of D-352, in its order, and the switch of `/healthz`.

mod common;

use std::sync::atomic::Ordering;

use common::{TestApp, TestOptions};
use offcut_api::transcribe::provider::TranscribeError;
use offcut_api_types::{ApiError, ApiErrorCode, CloudWord, HealthResponse, TranscribeResponse};
use reqwest::StatusCode;
use serde_json::{Value, json};

const MIB: usize = 1024 * 1024;
/// 3 s of a 440 Hz tone, written by `ffmpeg` with `libopus` (P161).
const TONE: &[u8] = include_bytes!("fixtures/ogg/ffmpeg_tone_3s.ogg");

/// One Ogg page; each packet under 255 bytes.
fn page(header_type: u8, granule: i64, sequence: u32, packets: &[&[u8]]) -> Vec<u8> {
    let mut out = b"OggS\0".to_vec();
    out.push(header_type);
    out.extend_from_slice(&granule.to_le_bytes());
    out.extend_from_slice(&9u32.to_le_bytes());
    out.extend_from_slice(&sequence.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.push(packets.len() as u8);
    out.extend(packets.iter().map(|p| p.len() as u8));
    for p in packets {
        out.extend_from_slice(p);
    }
    out
}

/// A minimal Ogg Opus file of `channels` and `duration_ms` (pre-skip 104).
fn ogg(channels: u8, duration_ms: i64) -> Vec<u8> {
    let mut head = b"OpusHead\x01".to_vec();
    head.push(channels);
    head.extend_from_slice(&104u16.to_le_bytes());
    head.extend_from_slice(&16_000u32.to_le_bytes());
    head.extend_from_slice(&[0, 0, 0]);
    let mut out = page(0x02, 0, 0, &[&head]);
    out.extend(page(0, 0, 1, &[b"OpusTags\x06\0\0\0offcut\0\0\0\0"]));
    out.extend(page(0x04, 104 + 48 * duration_ms, 2, &[&[0x78, 0, 0]]));
    out
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

/// Sends `body` as `content_type`, with the bearer if given, from its own address.
async fn post(
    app: &TestApp,
    bearer: Option<&str>,
    content_type: &str,
    body: Vec<u8>,
    ip: &str,
) -> reqwest::Response {
    let mut request = app
        .client
        .post(format!("{}/transcribe", app.base_url))
        .header("content-type", content_type)
        .header("x-forwarded-for", ip)
        .body(body);
    if let Some(token) = bearer {
        request = request.bearer_auth(token);
    }
    request.send().await.unwrap()
}

async fn code(response: reqwest::Response) -> ApiErrorCode {
    response.json::<ApiError>().await.unwrap().code
}

async fn health(app: &TestApp) -> HealthResponse {
    let response = app.client.get(format!("{}/healthz", app.base_url));
    response.send().await.unwrap().json().await.unwrap()
}

#[tokio::test]
async fn a_good_file_gives_the_providers_words_and_the_provider_got_the_exact_bytes() {
    let app = TestApp::spawn().await;
    let token = signed_in(&app, "a@example.com").await;
    let response = post(
        &app,
        Some(&token),
        "audio/ogg; codecs=opus",
        TONE.to_vec(),
        "203.0.113.1",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let body: TranscribeResponse = response.json().await.unwrap();
    assert_eq!(body.words.len(), 2);
    assert_eq!(body.words[1].text, "there.");
    assert!(
        body.duration_ms.abs_diff(3_000) <= 20,
        "{}",
        body.duration_ms
    );
    assert_eq!(body.model, "fake:whisper");
    assert_eq!(
        app.transcriber.received.lock().unwrap().as_slice(),
        [TONE.to_vec()]
    );
    assert!(health(&app).await.cloud_transcribe);
}

#[tokio::test]
async fn without_a_bearer_it_is_401_and_the_body_is_never_read() {
    let app = TestApp::spawn().await;
    // The server answers before reading and closes the connection, so the
    // client sees the 401 or, if it was still sending, a reset. Either way
    // nothing reached the provider.
    let own = reqwest::Client::builder()
        .no_proxy()
        .pool_max_idle_per_host(0)
        .build()
        .unwrap();
    let sent = own
        .post(format!("{}/transcribe", app.base_url))
        .header("content-type", "audio/ogg")
        .header("x-forwarded-for", "203.0.113.2")
        .body(vec![0u8; 2 * MIB])
        .send()
        .await;
    match sent {
        Ok(response) => {
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            assert_eq!(code(response).await, ApiErrorCode::Unauthorized);
        }
        Err(error) => assert!(error.is_request() || error.is_body(), "{error}"),
    }
    let response = post(
        &app,
        Some("not-a-token"),
        "audio/ogg",
        TONE.to_vec(),
        "203.0.113.2",
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(app.transcriber.received.lock().unwrap().is_empty());
}

#[tokio::test]
async fn the_switch_off_is_503_and_healthz_says_false() {
    let app = TestApp::spawn_with(TestOptions {
        transcribe_enabled: false,
        ..TestOptions::default()
    })
    .await;
    assert!(!health(&app).await.cloud_transcribe);
    let token = signed_in(&app, "b@example.com").await;
    let response = post(
        &app,
        Some(&token),
        "audio/ogg",
        TONE.to_vec(),
        "203.0.113.3",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(code(response).await, ApiErrorCode::TranscribeUnavailable);
    assert!(app.transcriber.received.lock().unwrap().is_empty());
}

#[tokio::test]
async fn every_other_type_is_415() {
    let app = TestApp::spawn().await;
    let token = signed_in(&app, "c@example.com").await;
    for content_type in [
        "audio/wav",
        "audio/webm",
        "video/mp4",
        "application/octet-stream",
        "multipart/form-data; boundary=x",
        "application/json",
    ] {
        let response = post(
            &app,
            Some(&token),
            content_type,
            TONE.to_vec(),
            "203.0.113.4",
        )
        .await;
        assert_eq!(
            response.status(),
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "{content_type}"
        );
    }
    assert!(app.transcriber.received.lock().unwrap().is_empty());
}

#[tokio::test]
async fn more_than_one_mib_is_413() {
    let app = TestApp::spawn().await;
    let token = signed_in(&app, "d@example.com").await;
    let mut body = TONE.to_vec();
    body.resize(MIB + 1, 0);
    let response = post(&app, Some(&token), "audio/ogg", body, "203.0.113.5").await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(code(response).await, ApiErrorCode::PayloadTooLarge);
    assert!(app.transcriber.received.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_file_that_is_not_one_mono_opus_stream_of_an_allowed_length_is_400() {
    let app = TestApp::spawn().await;
    let token = signed_in(&app, "e@example.com").await;
    let mut no_bos = ogg(1, 3_000);
    no_bos[5] = 0;
    for (what, body) in [
        (
            "random bytes",
            b"definitely not an ogg file at all, just bytes".to_vec(),
        ),
        ("stereo", ogg(2, 3_000)),
        ("no BOS", no_bos),
        ("no audio", ogg(1, 0)),
        ("one ms over 92 s", ogg(1, 92_001)),
    ] {
        let response = post(&app, Some(&token), "audio/ogg", body, "203.0.113.6").await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{what}");
        assert_eq!(code(response).await, ApiErrorCode::BadRequest, "{what}");
    }
    let response = post(
        &app,
        Some(&token),
        "audio/ogg",
        ogg(1, 92_000),
        "203.0.113.6",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK, "exactly 92 s");
    assert_eq!(app.transcriber.received.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn the_providers_failures_are_503_and_400() {
    let app = TestApp::spawn().await;
    let token = signed_in(&app, "f@example.com").await;
    *app.transcriber.answer.lock().unwrap() = Err(TranscribeError::Unavailable);
    let response = post(
        &app,
        Some(&token),
        "audio/ogg",
        TONE.to_vec(),
        "203.0.113.7",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(code(response).await, ApiErrorCode::TranscribeUnavailable);
    *app.transcriber.answer.lock().unwrap() = Err(TranscribeError::BadAudio);
    let response = post(
        &app,
        Some(&token),
        "audio/ogg",
        TONE.to_vec(),
        "203.0.113.7",
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn a_spent_budget_is_503_and_the_provider_is_not_called() {
    // Room for one 3 s clip in the hour, not two.
    let app = TestApp::spawn_with(TestOptions {
        transcribe_budget: (5, 1_000),
        ..TestOptions::default()
    })
    .await;
    let token = signed_in(&app, "g@example.com").await;
    let first = post(
        &app,
        Some(&token),
        "audio/ogg",
        TONE.to_vec(),
        "203.0.113.8",
    )
    .await;
    assert_eq!(first.status(), StatusCode::OK);
    let second = post(
        &app,
        Some(&token),
        "audio/ogg",
        TONE.to_vec(),
        "203.0.113.8",
    )
    .await;
    assert_eq!(second.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(code(second).await, ApiErrorCode::TranscribeUnavailable);
    assert_eq!(app.transcriber.received.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn the_21st_clip_of_a_user_in_a_day_is_429_and_another_user_still_passes() {
    let app = TestApp::spawn().await;
    let first = signed_in(&app, "h@example.com").await;
    let other = signed_in(&app, "i@example.com").await;
    // Each from its own address, so that the IP's bucket is not what refuses.
    for n in 0..20 {
        let ip = format!("198.51.100.{n}");
        let response = post(&app, Some(&first), "audio/ogg", TONE.to_vec(), &ip).await;
        assert_eq!(response.status(), StatusCode::OK, "clip {n}");
    }
    let refused = post(
        &app,
        Some(&first),
        "audio/ogg",
        TONE.to_vec(),
        "198.51.100.200",
    )
    .await;
    assert_eq!(refused.status(), StatusCode::TOO_MANY_REQUESTS);
    let body: ApiError = refused.json().await.unwrap();
    assert_eq!(body.code, ApiErrorCode::RateLimited);
    assert!(body.retry_after_secs.is_some_and(|s| s > 0));
    let response = post(
        &app,
        Some(&other),
        "audio/ogg",
        TONE.to_vec(),
        "198.51.100.201",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn one_address_is_held_to_60_an_hour() {
    let app = TestApp::spawn().await;
    // Unauthenticated requests still take the address's tokens: the layer runs first.
    for n in 0..60 {
        let response = post(&app, None, "audio/ogg", TONE.to_vec(), "203.0.113.9").await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "request {n}");
    }
    let response = post(&app, None, "audio/ogg", TONE.to_vec(), "203.0.113.9").await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn the_switch_is_read_on_every_request() {
    let app = TestApp::spawn().await;
    let token = signed_in(&app, "j@example.com").await;
    app.transcriber.enabled.store(false, Ordering::SeqCst);
    let response = post(
        &app,
        Some(&token),
        "audio/ogg",
        TONE.to_vec(),
        "203.0.113.10",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    app.transcriber.enabled.store(true, Ordering::SeqCst);
    let response = post(
        &app,
        Some(&token),
        "audio/ogg",
        TONE.to_vec(),
        "203.0.113.10",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[test]
fn nothing_under_transcribe_writes_to_the_database() {
    // P-16: the route stores nothing. No query macro or call names the pool there.
    for (name, source) in [
        ("mod.rs", include_str!("../src/transcribe/mod.rs")),
        ("provider.rs", include_str!("../src/transcribe/provider.rs")),
        ("ogg.rs", include_str!("../src/transcribe/ogg.rs")),
        ("budget.rs", include_str!("../src/transcribe/budget.rs")),
        ("route.rs", include_str!("../src/transcribe/route.rs")),
    ] {
        assert!(
            !source.contains("sqlx") && !source.contains("state.db"),
            "{name}"
        );
    }
    let _ = CloudWord {
        text: String::new(),
        start_ms: 0,
        end_ms: 0,
    };
}

// --- V6.1: windows of a long source (v6.1implementation D-411, §11.2) --------

/// `post` with an `X-Offcut-Window` header.
async fn post_window(
    app: &TestApp,
    bearer: Option<&str>,
    window: &str,
    ip: &str,
) -> reqwest::Response {
    let mut request = app
        .client
        .post(format!("{}/transcribe", app.base_url))
        .header("content-type", "audio/ogg")
        .header("x-forwarded-for", ip)
        .header("x-offcut-window", window)
        .body(TONE.to_vec());
    if let Some(token) = bearer {
        request = request.bearer_auth(token);
    }
    request.send().await.unwrap()
}

#[tokio::test]
async fn a_window_of_a_long_source_is_answered_as_a_clip_is() {
    let app = TestApp::spawn().await;
    let token = signed_in(&app, "w1@example.com").await;
    for window in ["0/34", "33/34", "34/35"] {
        let response = post_window(&app, Some(&token), window, "192.0.2.10").await;
        assert_eq!(response.status(), StatusCode::OK, "{window}");
        let body: TranscribeResponse = response.json().await.unwrap();
        assert!(body.duration_ms > 0);
    }
    assert_eq!(app.transcriber.received.lock().unwrap().len(), 3);
}

#[tokio::test]
async fn a_malformed_window_header_is_400_before_the_bearer_and_the_body() {
    let app = TestApp::spawn().await;
    let token = signed_in(&app, "w2@example.com").await;
    for bad in ["34/34", "0/36", "a/b", "-1/3", "3", "1/"] {
        let response = post_window(&app, Some(&token), bad, "192.0.2.11").await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{bad}");
        assert_eq!(code(response).await, ApiErrorCode::BadRequest, "{bad}");
        // Without a bearer the header is still what refuses it.
        let response = post_window(&app, None, bad, "192.0.2.11").await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{bad}");
    }
    assert!(app.transcriber.received.lock().unwrap().is_empty());
}

#[tokio::test]
async fn the_121st_window_of_a_user_in_a_day_is_429_while_their_clips_still_pass() {
    let app = TestApp::spawn().await;
    let token = signed_in(&app, "w3@example.com").await;
    for n in 0..120 {
        let response = post_window(&app, Some(&token), "0/34", "192.0.2.12").await;
        assert_eq!(response.status(), StatusCode::OK, "window {n}");
    }
    let refused = post_window(&app, Some(&token), "1/34", "192.0.2.12").await;
    assert_eq!(refused.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(code(refused).await, ApiErrorCode::RateLimited);
    // The person's clips have their own bucket.
    let clip = post(&app, Some(&token), "audio/ogg", TONE.to_vec(), "192.0.2.13").await;
    assert_eq!(clip.status(), StatusCode::OK);
}

#[tokio::test]
async fn windows_and_clips_take_an_address_s_tokens_from_their_own_buckets() {
    let app = TestApp::spawn().await;
    // The clips' 60 an hour, spent by unauthenticated requests, as before V6.1.
    for n in 0..60 {
        let response = post(&app, None, "audio/ogg", TONE.to_vec(), "192.0.2.14").await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "clip {n}");
    }
    let clip = post(&app, None, "audio/ogg", TONE.to_vec(), "192.0.2.14").await;
    assert_eq!(clip.status(), StatusCode::TOO_MANY_REQUESTS);
    // A window from the same address is not refused by the clips' bucket: it reaches the bearer.
    let window = post_window(&app, None, "0/34", "192.0.2.14").await;
    assert_eq!(window.status(), StatusCode::UNAUTHORIZED);
    // The windows' own bucket: 200 an hour.
    for n in 1..200 {
        let response = post_window(&app, None, "0/34", "192.0.2.14").await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "window {n}");
    }
    let window = post_window(&app, None, "0/34", "192.0.2.14").await;
    assert_eq!(window.status(), StatusCode::TOO_MANY_REQUESTS);
}
