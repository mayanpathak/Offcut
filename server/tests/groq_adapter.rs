#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The Groq adapter (cloudimplementation §7.4, §11.1) with no network: real
//! answers recorded in P161 (`fixtures/groq/`), and a local mock server for
//! the request it sends and the failures it maps.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::post;
use ed25519_dalek::SigningKey;
use offcut_api::config::Config;
use offcut_api::transcribe::groq::{GroqTranscriber, MODEL_VERSION, parse};
use offcut_api::transcribe::provider::{TranscribeError, Transcriber};

const OPUS32: &[u8] = include_bytes!("fixtures/groq/speech_60s_portrait.opus32.json");
const OPUS32_SEGMENTS: &[u8] =
    include_bytes!("fixtures/groq/speech_60s_portrait.opus32.segments.json");
const FLAC: &[u8] = include_bytes!("fixtures/groq/speech_60s_portrait.flac.json");
const TONE_ANSWER: &[u8] = include_bytes!("fixtures/groq/tone_3s.json");
const TONE: &[u8] = include_bytes!("fixtures/ogg/ffmpeg_tone_3s.ogg");

#[test]
fn the_reference_clips_answer_parses_to_its_words_with_their_punctuation() {
    let t = parse(OPUS32, 60_000).unwrap();
    assert_eq!(t.words.len(), 129);
    assert_eq!(t.model, MODEL_VERSION);
    assert_eq!(t.duration_ms, 60_000);
    let first: Vec<&str> = t.words.iter().take(7).map(|w| w.text.as_str()).collect();
    assert_eq!(
        first,
        ["Last", "year,", "I", "almost", "quit", "making", "videos."]
    );
    assert_eq!((t.words[0].start_ms, t.words[0].end_ms), (380, 600));
    // Groq's words already carry their marks: `align` changed none of them.
    let raw: serde_json::Value = serde_json::from_slice(OPUS32).unwrap();
    let raw_words: Vec<&str> = raw["words"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["word"].as_str().unwrap())
        .collect();
    let ours: Vec<&str> = t.words.iter().map(|w| w.text.as_str()).collect();
    assert_eq!(ours, raw_words);
    // Times rise and stay inside a little over the audio (the invented
    // "Thank you." ends at 60.48 s; the client's silence guard drops it, D-382).
    assert!(t.words.windows(2).all(|p| p[0].start_ms <= p[1].start_ms));
    assert_eq!(t.words.last().unwrap().end_ms, 60_480);
}

#[test]
fn every_recorded_answer_parses() {
    for (name, body, words) in [
        ("opus32 with segments", OPUS32_SEGMENTS, 129),
        ("flac", FLAC, 128),
        ("tone", TONE_ANSWER, 1),
    ] {
        let t = parse(body, 3_000).unwrap();
        assert_eq!(t.words.len(), words, "{name}");
    }
}

/// Fixed values pointing the adapter at `base_url`. Nothing here is a real key.
fn config(base_url: String) -> Config {
    // As `main.rs` does. Only the first call in a process installs it.
    let _ = rustls::crypto::ring::default_provider().install_default();
    let placeholder = || "unset-in-tests".to_owned();
    Config {
        database_url: placeholder(),
        app_origin: "http://localhost:5173".to_owned(),
        port: 0,
        log_level: tracing::Level::INFO,
        entitlement_signing_key: SigningKey::from_bytes(&[1; 32]),
        access_token_signing_key: SigningKey::from_bytes(&[2; 32]),
        smtp_host: placeholder(),
        smtp_port: 2525,
        smtp_username: placeholder(),
        smtp_password: placeholder(),
        mail_from: placeholder(),
        billing_api_base_url: placeholder(),
        billing_api_key: placeholder(),
        billing_webhook_secret: placeholder(),
        billing_price_creator_monthly: placeholder(),
        billing_price_creator_annual: placeholder(),
        billing_price_creator_annual_founding: placeholder(),
        founding_offer_enabled: false,
        transcribe_enabled: true,
        transcribe_api_key: "test-transcribe-key".to_owned(),
        transcribe_api_base_url: base_url,
        trusted_proxy_hops: 1,
        git_sha: "test-sha".to_owned(),
    }
}

/// What the mock received, and what it answers.
#[derive(Default)]
struct Mock {
    received: Mutex<Vec<(HeaderMap, Vec<u8>)>>,
    answer: Mutex<Option<(u16, Vec<u8>, Duration)>>,
}

async fn answer(
    State(mock): State<Arc<Mock>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    mock.received.lock().unwrap().push((headers, body.to_vec()));
    let (status, body, delay) =
        mock.answer
            .lock()
            .unwrap()
            .clone()
            .unwrap_or((200, OPUS32.to_vec(), Duration::ZERO));
    tokio::time::sleep(delay).await;
    (StatusCode::from_u16(status).unwrap(), body)
}

/// A mock of Groq on a local port, and an adapter pointed at it.
async fn mock(timeout: Duration) -> (Arc<Mock>, GroqTranscriber) {
    let state = Arc::new(Mock::default());
    let app = Router::new()
        .route("/openai/v1/audio/transcriptions", post(answer))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let config = config(format!("http://{address}/openai/v1"));
    (
        state,
        GroqTranscriber::with_timeout(&config, timeout).unwrap(),
    )
}

#[tokio::test]
async fn the_request_is_the_one_groq_documents() {
    let (state, groq) = mock(Duration::from_secs(5)).await;
    let t = groq
        .transcribe(Bytes::from_static(TONE), 3_000)
        .await
        .unwrap();
    assert_eq!(t.words.len(), 129);

    let received = state.received.lock().unwrap();
    let (headers, body) = &received[0];
    assert_eq!(headers["authorization"], "Bearer test-transcribe-key");
    assert!(
        headers["content-type"]
            .to_str()
            .unwrap()
            .starts_with("multipart/form-data; boundary=")
    );
    let text = String::from_utf8_lossy(body);
    for (name, value) in [
        ("model", "whisper-large-v3-turbo"),
        ("response_format", "verbose_json"),
        ("timestamp_granularities[]", "word"),
        ("language", "en"),
        ("temperature", "0"),
    ] {
        let field = format!("name=\"{name}\"\r\n\r\n{value}\r\n");
        assert!(text.contains(&field), "{name} = {value} is not in the form");
    }
    assert!(text.contains("name=\"file\"; filename=\"audio.ogg\"\r\nContent-Type: audio/ogg"));
    // The file's bytes, unchanged, inside the form.
    assert!(body.windows(TONE.len()).any(|w| w == TONE));
}

#[tokio::test]
async fn each_failure_maps_to_unavailable_or_bad_audio() {
    let (state, groq) = mock(Duration::from_millis(500)).await;
    for (status, expected) in [
        (429, TranscribeError::Unavailable),
        (500, TranscribeError::Unavailable),
        (503, TranscribeError::Unavailable),
        (401, TranscribeError::Unavailable),
        (400, TranscribeError::BadAudio),
        (413, TranscribeError::BadAudio),
    ] {
        *state.answer.lock().unwrap() = Some((
            status,
            b"{\"error\":{\"message\":\"markerprovidertext\"}}".to_vec(),
            Duration::ZERO,
        ));
        let error = groq
            .transcribe(Bytes::from_static(TONE), 3_000)
            .await
            .unwrap_err();
        assert_eq!(error, expected, "{status}");
    }
    *state.answer.lock().unwrap() = Some((200, b"not json".to_vec(), Duration::ZERO));
    assert_eq!(
        groq.transcribe(Bytes::from_static(TONE), 3_000)
            .await
            .unwrap_err(),
        TranscribeError::Unavailable
    );
    // A stall past the timeout (the production adapter's is 25 s; here 0.5 s).
    *state.answer.lock().unwrap() = Some((200, OPUS32.to_vec(), Duration::from_secs(2)));
    assert_eq!(
        groq.transcribe(Bytes::from_static(TONE), 3_000)
            .await
            .unwrap_err(),
        TranscribeError::Unavailable
    );
}

#[tokio::test]
async fn nothing_listening_is_unavailable() {
    let config = config("http://127.0.0.1:9/openai/v1".to_owned());
    let groq = GroqTranscriber::with_timeout(&config, Duration::from_secs(2)).unwrap();
    assert_eq!(
        groq.transcribe(Bytes::from_static(TONE), 3_000)
            .await
            .unwrap_err(),
        TranscribeError::Unavailable
    );
}
