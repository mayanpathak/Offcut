#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The DeepSeek adapter (v6.2implementation D-500, §7.3) with no network:
//! real answers recorded in P188 and P188b (`fixtures/deepseek/`, ids
//! redacted), and a local mock for the request it sends and the failures it
//! maps. Never a real call.

use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::post;
use offcut_api::config::{PlanProvider as Provider, PlanSettings};
use offcut_api::plan::deepseek::{
    DeepseekProvider, PLAN_UPSTREAM_TIMEOUT, PROVIDER, SCHEMA_LEAD, parse, system_message,
};
use offcut_api::plan::parse::{ParseFailureKind, parse_plan};
use offcut_api::plan::provider::{PlanProvider, PlanProviderError, ProviderRequest, Usage};
use offcut_plan::prompt::{output_schema, system_prompt};
use offcut_types::PlanSource;
use serde_json::{Value, json};
use tracing_subscriber::fmt::MakeWriter;

const FLASH_A01: &[u8] = include_bytes!("fixtures/deepseek/flash_a01.response.json");
const PRO_A01: &[u8] = include_bytes!("fixtures/deepseek/pro_a01.response.json");
const REQUEST: &str = include_str!("fixtures/deepseek/request.recorded.json");
const REFUSED_400: &[u8] = include_bytes!("fixtures/deepseek/json_schema_refused.400.json");
const UNKNOWN_EMOJI: &str = include_str!("fixtures/deepseek/flash_w03_unknown_emoji.content.json");
const ICON_AS_EMOJI: &str = include_str!("fixtures/deepseek/flash_w10_icon_as_emoji.content.json");
const UNKNOWN_ICON: &str = include_str!("fixtures/deepseek/pro_w05_unknown_icon.content.json");

const MARKER_TEXT: &str = "markermodeltext";

fn source(model: &str) -> PlanSource {
    PlanSource::Model {
        provider: PROVIDER.to_owned(),
        model: model.to_owned(),
    }
}

// --- The recorded answers --------------------------------------------------

#[test]
fn every_recorded_answer_reads_with_its_tokens_and_parses_whole() {
    for (name, body, model, usage, items) in [
        (
            "flash a01 (P188b)",
            FLASH_A01,
            "deepseek-flash",
            (7_246, 640, 669),
            9,
        ),
        (
            "pro a01 (P188)",
            PRO_A01,
            "deepseek-v4-pro",
            (6_986, 0, 1_591),
            13,
        ),
    ] {
        let output = parse(body, "asked").unwrap();
        assert_eq!(output.model, model, "{name}");
        assert_eq!(
            output.usage,
            Usage {
                input_tokens: usage.0,
                cached_input_tokens: usage.1,
                output_tokens: usage.2,
            },
            "{name}"
        );
        let parsed = parse_plan(&output.text, source(&output.model)).unwrap();
        assert_eq!(parsed.items_skipped, 0, "{name}");
        assert_eq!(parsed.plan.items.len(), items, "{name}");
        assert_eq!(parsed.plan.source, source(model), "{name}");
    }
}

/// TE-15's three plans that failed whole on one name outside the 72: item by
/// item, each loses that one item and keeps the rest (checkpoint 1).
#[test]
fn a_recorded_answer_with_one_unknown_name_keeps_every_other_item() {
    for (name, text) in [
        ("w03, the emoji `cat`", UNKNOWN_EMOJI),
        ("w10, the icon `lock` as an emoji", ICON_AS_EMOJI),
        ("w05, the icon `cloud`", UNKNOWN_ICON),
    ] {
        let written = serde_json::from_str::<Value>(text).unwrap()["items"]
            .as_array()
            .unwrap()
            .len();
        // As the whole plan, it fails: what P188 saw.
        assert!(
            serde_json::from_str::<offcut_types::EditPlan>(text).is_err(),
            "{name}"
        );
        let parsed = parse_plan(text, source("m")).unwrap();
        assert_eq!(parsed.items_skipped, 1, "{name}");
        assert_eq!(parsed.plan.items.len(), written - 1, "{name}");
    }
}

// --- The mock --------------------------------------------------------------

/// Fixed values pointing the adapter at `base_url`. Nothing here is a real key.
fn settings(base_url: String) -> PlanSettings {
    // As `main.rs` does. Only the first call in a process installs it.
    let _ = rustls::crypto::ring::default_provider().install_default();
    PlanSettings {
        enabled: true,
        provider: Provider::Deepseek,
        model: "deepseek-flash".to_owned(),
        api_key: "test-plan-key".to_owned(),
        api_base_url: base_url,
        monthly_cap_micro_usd: 5_000_000,
        price_in_usd_per_mtok: 0.30,
        price_out_usd_per_mtok: 1.20,
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
            .unwrap_or((200, FLASH_A01.to_vec(), Duration::ZERO));
    tokio::time::sleep(delay).await;
    (StatusCode::from_u16(status).unwrap(), body)
}

/// A mock of DeepSeek on a local port, and an adapter pointed at it.
async fn mock(timeout: Duration) -> (Arc<Mock>, DeepseekProvider) {
    let state = Arc::new(Mock::default());
    let app = Router::new()
        .route("/chat/completions", post(answer))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let settings = settings(format!("http://{address}"));
    (
        state,
        DeepseekProvider::with_timeout(&settings, timeout).unwrap(),
    )
}

fn request(user: &str) -> ProviderRequest {
    ProviderRequest {
        system: system_prompt(),
        user: user.to_owned(),
        schema: output_schema(),
    }
}

#[tokio::test]
async fn the_request_is_the_one_recorded() {
    let (state, deepseek) = mock(Duration::from_secs(5)).await;
    let output = deepseek
        .complete(request("the user message"))
        .await
        .unwrap();
    assert_eq!(output.model, "deepseek-flash");
    assert_eq!(output.usage.output_tokens, 669);

    let received = state.received.lock().unwrap();
    let (headers, body) = &received[0];
    assert_eq!(headers["authorization"], "Bearer test-plan-key");
    assert_eq!(headers["content-type"], "application/json");
    let sent: Value = serde_json::from_slice(body).unwrap();
    // Every field as P188c and P206c sent it, the two texts aside.
    let mut recorded: Value = serde_json::from_str(REQUEST).unwrap();
    let mut sent_rest = sent.clone();
    for value in [&mut recorded, &mut sent_rest] {
        value.as_object_mut().unwrap().remove("messages");
    }
    assert_eq!(sent_rest, recorded);
    assert_eq!(sent["messages"][0]["role"], "system");
    assert_eq!(
        sent["messages"][1],
        json!({ "role": "user", "content": "the user message" })
    );
    // The system message is the prompt, the lead and the compact schema: the
    // same 16,261 characters (UTF-16, as the recording counted) P206c sent
    // (16,366 in P188c; D-503's hook line, declared in V6 CL Entry 71).
    let system = sent["messages"][0]["content"].as_str().unwrap();
    assert_eq!(system, system_message(&request("")));
    assert!(system.starts_with(&system_prompt()));
    assert!(system.ends_with(&format!("{SCHEMA_LEAD}\n{}", output_schema())));
    assert_eq!(
        recorded_length(&recorded_messages()[0]),
        system.encode_utf16().count()
    );
}

/// The recorded request's messages, whose texts were cut to their lengths.
fn recorded_messages() -> Vec<String> {
    let recorded: Value = serde_json::from_str(REQUEST).unwrap();
    recorded["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["content"].as_str().unwrap().to_owned())
        .collect()
}

/// `<16159 chars: …>` to 16159.
fn recorded_length(text: &str) -> usize {
    text.trim_start_matches('<')
        .split(' ')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

#[tokio::test]
async fn each_failure_maps_to_one_error() {
    let (state, deepseek) = mock(Duration::from_millis(500)).await;
    let error_body = format!("{{\"error\":{{\"message\":\"{MARKER_TEXT}\"}}}}").into_bytes();
    for (status, body) in [
        (400, REFUSED_400.to_vec()),
        (401, error_body.clone()),
        (402, error_body.clone()),
        (429, error_body.clone()),
        (500, error_body.clone()),
        (503, b"<html>busy</html>".to_vec()),
    ] {
        *state.answer.lock().unwrap() = Some((status, body, Duration::ZERO));
        assert_eq!(
            deepseek.complete(request("u")).await.unwrap_err(),
            PlanProviderError::Unavailable,
            "{status}"
        );
    }
    // A 429 is never retried (D-459): one request for one call.
    let before = state.received.lock().unwrap().len();
    *state.answer.lock().unwrap() = Some((429, error_body, Duration::ZERO));
    let _ = deepseek.complete(request("u")).await;
    assert_eq!(state.received.lock().unwrap().len(), before + 1);

    let usage = r#""usage":{"prompt_tokens":9,"completion_tokens":3,"prompt_cache_hit_tokens":2}"#;
    for (body, expected) in [
        // Malformed JSON.
        ("{\"choices\": [".to_owned(), PlanProviderError::Unavailable),
        // No choice.
        (
            format!("{{\"choices\":[],{usage}}}"),
            PlanProviderError::Unavailable,
        ),
        // A blocked answer: billed, so its usage is kept.
        (
            format!(
                r#"{{"choices":[{{"message":{{"content":""}},"finish_reason":"content_filter"}}],{usage}}}"#
            ),
            PlanProviderError::Blocked(Usage {
                input_tokens: 9,
                cached_input_tokens: 2,
                output_tokens: 3,
            }),
        ),
    ] {
        *state.answer.lock().unwrap() = Some((200, body.into_bytes(), Duration::ZERO));
        assert_eq!(deepseek.complete(request("u")).await.unwrap_err(), expected);
    }
    // A stall past the timeout (production's is 15 s; here 0.5 s).
    assert_eq!(PLAN_UPSTREAM_TIMEOUT, Duration::from_secs(15));
    *state.answer.lock().unwrap() = Some((200, FLASH_A01.to_vec(), Duration::from_secs(2)));
    assert_eq!(
        deepseek.complete(request("u")).await.unwrap_err(),
        PlanProviderError::Timeout
    );
}

/// DeepSeek documents a blank answer with `finish_reason: stop` (D-500): the
/// adapter returns it with its usage, and the parse refuses it, so the route
/// takes its one retry and the spend still counts the call.
#[tokio::test]
async fn a_blank_answer_is_returned_and_the_parse_refuses_it() {
    let (state, deepseek) = mock(Duration::from_secs(5)).await;
    let blank = br#"{"model":"deepseek-flash","choices":[{"message":{"role":"assistant","content":"\n\n  "},"finish_reason":"stop"}],
                     "usage":{"prompt_tokens":5000,"completion_tokens":2,"prompt_cache_hit_tokens":4096}}"#;
    *state.answer.lock().unwrap() = Some((200, blank.to_vec(), Duration::ZERO));
    let output = deepseek.complete(request("u")).await.unwrap();
    assert_eq!(output.usage.input_tokens, 5_000);
    assert_eq!(
        parse_plan(&output.text, source(&output.model))
            .unwrap_err()
            .kind,
        ParseFailureKind::Blank
    );
}

/// A refused connection is not a timeout. Windows takes about 2 s to report
/// one (it retries the connect), so the timeout here is well above that.
#[tokio::test]
async fn nothing_listening_is_unavailable() {
    let settings = settings("http://127.0.0.1:9".to_owned());
    let deepseek = DeepseekProvider::with_timeout(&settings, Duration::from_secs(10)).unwrap();
    assert_eq!(
        deepseek.complete(request("u")).await.unwrap_err(),
        PlanProviderError::Unavailable
    );
}

// --- The log ---------------------------------------------------------------

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

/// Nothing the model or the provider wrote, and nothing of the request,
/// reaches a log: not on success, not on any failure (P-14, D-464).
#[tokio::test]
async fn no_text_of_the_request_or_the_answer_is_logged() {
    let logs = Captured::default();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(logs.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let (state, deepseek) = mock(Duration::from_millis(500)).await;
    let user = "markerusertranscript";
    let content = format!(
        r#"{{"choices":[{{"message":{{"content":"{MARKER_TEXT}"}},"finish_reason":"stop"}}],
            "usage":{{"prompt_tokens":1,"completion_tokens":1}}}}"#
    );
    let error = format!("{{\"error\":{{\"message\":\"{MARKER_TEXT}\"}}}}");
    for (status, body) in [
        (200, content.clone()),
        (429, error.clone()),
        (500, error),
        (200, content.replace("\"stop\"", "\"content_filter\"")),
        (200, format!("{MARKER_TEXT} not json")),
    ] {
        *state.answer.lock().unwrap() = Some((status, body.into_bytes(), Duration::ZERO));
        let _ = deepseek.complete(request(user)).await;
    }
    let text = String::from_utf8(logs.0.lock().unwrap().clone()).unwrap();
    assert!(
        text.contains("plan request failed"),
        "the failures are logged: {text}"
    );
    for marker in [MARKER_TEXT, user, "test-plan-key", "You plan the edit"] {
        assert!(!text.contains(marker), "{marker} is in the log: {text}");
    }
}
