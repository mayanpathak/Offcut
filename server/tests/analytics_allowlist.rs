#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! `POST /events` against a real Postgres: only allowlisted events are
//! stored, a bad batch is rejected whole, and a row holds nothing that
//! identifies a person (v1implementation §13.2).

mod common;

use common::TestApp;
use offcut_api::analytics::retention::purge_once;
use reqwest::{Response, StatusCode};
use serde_json::{Value, json};

const ANON_ID: &str = "11111111-1111-4111-8111-111111111111";

/// One valid event of each of the 19 kinds, as the client sends it.
fn one_of_each() -> Vec<Value> {
    vec![
        json!({"name": "landing_view", "props": {"hero_variant": "outcome"}}),
        json!({"name": "capability_check", "props": {
            "result": "fail", "unsupported_reason": "UNSUPPORTED_WEBGPU",
            "gpu_vendor": "intel", "memory_bucket": "gb8plus", "platform": "windows"}}),
        json!({"name": "clip_accepted", "props": {
            "duration_bucket": "lt60", "orientation": "portrait", "source": "user"}}),
        json!({"name": "clip_rejected", "props": {"reject_reason": "REJECT_HEVC"}}),
        json!({"name": "model_download", "props": {
            "outcome": "hash_mismatch", "duration_ms": 41000, "resumed": true}}),
        json!({"name": "stage_timing", "props": {
            "stage": "asr", "duration_ms": 18000, "asr_backend": "webgpu"}}),
        json!({"name": "pipeline_done", "props": {
            "total_ms": 52000, "n_number": 2, "n_list": 1, "n_from_to": 1, "n_keyword": 3}}),
        json!({"name": "preview_played"}),
        json!({"name": "review_action", "props": {
            "action": "event_toggle", "event_kind": "from_to", "enabled": false}}),
        json!({"name": "export_started", "props": {"profile": "free"}}),
        json!({"name": "export_done", "props": {
            "total_ms": 88000, "profile": "creator", "word_edits": 2, "events_kept": 4,
            "events_disabled": 1, "style": "bold", "crop_adjusted": true, "from_cache": false}}),
        json!({"name": "export_failed", "props": {
            "error_code": "E_ENCODE_VIDEO", "stage": "render_encode"}}),
        json!({"name": "job_cancelled", "props": {"stage": "model"}}),
        json!({"name": "post_export_answer", "props": {"answer": "small_edits"}}),
        json!({"name": "upgrade_prompt", "props": {
            "placement": "limit_reached", "action": "clicked"}}),
        json!({"name": "signin_step", "props": {"step": "link_requested"}}),
        json!({"name": "checkout_step", "props": {"step": "returned_success"}}),
        json!({"name": "client_error", "props": {
            "error_code": "E_WORKER_CRASH", "stage": "import"}}),
        json!({"name": "local_data_cleared"}),
    ]
}

fn landing_view() -> Value {
    json!({"name": "landing_view", "props": {"hero_variant": "outcome"}})
}

fn batch(events: &[Value]) -> Value {
    json!({"anon_id": ANON_ID, "events": events})
}

async fn post_as(app: &TestApp, content_type: &str, body: String) -> Response {
    let request = app.client.post(format!("{}/events", app.base_url));
    let request = request.header("content-type", content_type);
    request.body(body).send().await.unwrap()
}

async fn post(app: &TestApp, body: &Value) -> Response {
    post_as(app, "application/json", body.to_string()).await
}

/// The stored rows as `(name, props)`, in the order they were inserted.
async fn rows(app: &TestApp) -> Vec<(String, Value)> {
    let rows = "SELECT name, props FROM analytics_events ORDER BY id";
    sqlx::query_as(rows).fetch_all(&app.pool).await.unwrap()
}

/// Asserts that each body is answered with 400 `bad_request` and stores nothing.
async fn assert_all_rejected(app: &TestApp, bodies: &[Value]) {
    for body in bodies {
        let response = post(app, body).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{body}");
        let error: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        // The body names the error and echoes nothing of the request.
        assert_eq!(
            error,
            json!({"code": "bad_request", "retry_after_secs": null}),
            "{body}"
        );
    }
    assert!(rows(app).await.is_empty());
}

#[tokio::test]
async fn one_valid_event_of_each_kind_is_stored_as_sent() {
    let app = TestApp::spawn().await;
    let events = one_of_each();
    assert_eq!(events.len(), offcut_api_types::ANALYTICS_EVENT_DOCS.len());

    let response = post(&app, &batch(&events)).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(response.text().await.unwrap(), "");

    let stored = rows(&app).await;
    assert_eq!(stored.len(), 19);
    let docs = offcut_api_types::ANALYTICS_EVENT_DOCS;
    for ((sent, (name, props)), doc) in events.iter().zip(&stored).zip(docs) {
        assert_eq!(
            *name, doc.name,
            "the samples follow the order of the allowlist"
        );
        assert_eq!(sent["name"], *name);
        // An event without props is stored with an empty object.
        let sent_props = sent.get("props").cloned().unwrap_or_else(|| json!({}));
        assert_eq!(*props, sent_props, "{name}");
    }

    let anon_ids = "SELECT DISTINCT anon_id::text FROM analytics_events";
    let anon_ids: Vec<String> = sqlx::query_scalar(anon_ids)
        .fetch_all(&app.pool)
        .await
        .unwrap();
    assert_eq!(anon_ids, [ANON_ID]);
}

#[tokio::test]
async fn an_unknown_event_name_is_rejected() {
    let app = TestApp::spawn().await;
    let bodies = [
        batch(&[json!({"name": "page_view"})]),
        batch(&[json!({"name": "Landing_View", "props": {"hero_variant": "outcome"}})]),
        batch(&[json!({"props": {"hero_variant": "outcome"}})]),
    ];
    assert_all_rejected(&app, &bodies).await;
}

#[tokio::test]
async fn a_valid_event_with_one_extra_prop_is_rejected() {
    let app = TestApp::spawn().await;
    let bodies = [
        batch(&[json!({"name": "landing_view", "props": {
            "hero_variant": "outcome", "referrer": "search"}})]),
        batch(&[json!({"name": "export_started", "props": {"profile": "free", "seconds": 60}})]),
        batch(&[json!({"name": "landing_view", "props": {"hero_variant": "outcome"}, "ts": 1})]),
    ];
    assert_all_rejected(&app, &bodies).await;
}

#[tokio::test]
async fn a_string_where_an_enum_is_expected_is_rejected() {
    let app = TestApp::spawn().await;
    let bodies = [
        batch(&[json!({"name": "landing_view", "props": {"hero_variant": "<free text>"}})]),
        batch(&[
            json!({"name": "clip_rejected", "props": {"reject_reason": "my file is secret.mp4"}}),
        ]),
        batch(&[json!({"name": "export_failed", "props": {
            "error_code": "E_ENCODE_VIDEO", "stage": "somewhere"}})]),
    ];
    assert_all_rejected(&app, &bodies).await;
}

#[tokio::test]
async fn a_free_text_prop_is_rejected_on_any_event() {
    let app = TestApp::spawn().await;
    let text = "the words I said in my clip";
    let mut bodies = Vec::new();
    for mut event in one_of_each() {
        // The event is valid until the text is added to it.
        match event.get_mut("props") {
            Some(props) => props["text"] = json!(text),
            None => event["props"] = json!({"text": text}),
        }
        bodies.push(batch(&[event]));
    }
    // Text in place of a number or a boolean is no better.
    bodies.push(batch(&[json!({"name": "model_download", "props": {
        "outcome": "ok", "duration_ms": text, "resumed": false}})]));
    bodies.push(batch(&[json!({"name": "model_download", "props": {
        "outcome": "ok", "duration_ms": 1, "resumed": text}})]));
    assert_eq!(bodies.len(), 21);
    assert_all_rejected(&app, &bodies).await;
}

#[tokio::test]
async fn one_invalid_event_among_valid_ones_rejects_the_whole_batch() {
    let app = TestApp::spawn().await;
    let invalid = json!({"name": "landing_view", "props": {"hero_variant": "hello"}});
    let out_of_bounds = json!({"name": "stage_timing", "props": {
        "stage": "asr", "duration_ms": 3_600_001}});
    let bodies = [
        batch(&[landing_view(), invalid.clone(), landing_view()]),
        batch(&[landing_view(), landing_view(), invalid]),
        // A batch that parses but breaks a cap is rejected whole as well.
        batch(&[landing_view(), out_of_bounds, landing_view()]),
    ];
    assert_all_rejected(&app, &bodies).await;
}

#[tokio::test]
async fn a_duration_or_a_count_above_its_cap_is_rejected() {
    let app = TestApp::spawn().await;
    let stage = |duration_ms: u64| json!({"name": "stage_timing", "props": {"stage": "asr", "duration_ms": duration_ms}});
    let pipeline = |n_list: u64| {
        json!({"name": "pipeline_done", "props": {
            "total_ms": 1, "n_number": 0, "n_list": n_list, "n_from_to": 0, "n_keyword": 0}})
    };
    let bodies = [
        batch(&[stage(3_600_001)]),
        batch(&[pipeline(10_001)]),
        // Past the range of the integer type: refused when the body is parsed.
        batch(&[stage(u64::from(u32::MAX) + 1)]),
        batch(&[json!({"name": "stage_timing", "props": {"stage": "asr", "duration_ms": -1}})]),
    ];
    assert_all_rejected(&app, &bodies).await;

    // Exactly at the caps is accepted.
    let at_caps = batch(&[stage(3_600_000), pipeline(10_000)]);
    assert_eq!(post(&app, &at_caps).await.status(), StatusCode::NO_CONTENT);
    assert_eq!(rows(&app).await.len(), 2);
}

#[tokio::test]
async fn a_batch_of_51_events_or_of_none_is_rejected() {
    let app = TestApp::spawn().await;
    let bodies = [batch(&vec![landing_view(); 51]), batch(&[])];
    assert_all_rejected(&app, &bodies).await;

    // 50 is the most a batch may hold.
    let full = batch(&vec![landing_view(); 50]);
    assert_eq!(post(&app, &full).await.status(), StatusCode::NO_CONTENT);
    assert_eq!(rows(&app).await.len(), 50);
}

#[tokio::test]
async fn a_body_over_16_kb_is_413() {
    let app = TestApp::spawn().await;
    // A batch that would be valid but for its size: JSON allows whitespace.
    let body = batch(&[landing_view()]).to_string();
    let padded = format!("{}{body}", " ".repeat(16 * 1024));
    let response = post_as(&app, "application/json", padded).await;

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    let error: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(error["code"], "payload_too_large");
    assert!(rows(&app).await.is_empty());
}

#[tokio::test]
async fn a_media_or_upload_content_type_is_415() {
    let app = TestApp::spawn().await;
    // The body is a valid batch each time: only the content type is wrong.
    let body = batch(&[landing_view()]).to_string();
    let content_types = [
        "application/octet-stream",
        "multipart/form-data; boundary=x",
        "video/mp4",
        "audio/wav",
        "text/plain",
    ];
    for content_type in content_types {
        let response = post_as(&app, content_type, body.clone()).await;
        assert_eq!(
            response.status(),
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "{content_type}"
        );
        let error: Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
        assert_eq!(error["code"], "unsupported_media_type", "{content_type}");
    }
    assert!(rows(&app).await.is_empty());
}

#[tokio::test]
async fn an_extra_top_level_key_is_rejected() {
    let app = TestApp::spawn().await;
    let bodies = [
        json!({"anon_id": ANON_ID, "events": [landing_view()], "user_id": ANON_ID}),
        json!({"anon_id": ANON_ID, "events": [landing_view()], "ts": 1_700_000_000}),
        json!({"anon_id": "not-a-uuid", "events": [landing_view()]}),
        json!({"events": [landing_view()]}),
    ];
    assert_all_rejected(&app, &bodies).await;
}

#[tokio::test]
async fn the_table_has_exactly_five_columns() {
    let app = TestApp::spawn().await;
    let columns = "SELECT column_name::text FROM information_schema.columns
                   WHERE table_schema = 'public' AND table_name = 'analytics_events'
                   ORDER BY ordinal_position";
    let columns: Vec<String> = sqlx::query_scalar(columns)
        .fetch_all(&app.pool)
        .await
        .unwrap();
    // No user id, no IP address, no user agent (TS §25.1 P-10).
    assert_eq!(columns, ["id", "anon_id", "name", "props", "ts"]);
}

#[tokio::test]
async fn the_purge_deletes_rows_older_than_90_days_only() {
    let app = TestApp::spawn().await;
    let insert = "INSERT INTO analytics_events (anon_id, name, props, ts)
                  VALUES ($1::uuid, $2, '{}', now() - make_interval(days => $3))";
    for (name, days_ago) in [("old", 91), ("recent", 89)] {
        let insert = sqlx::query(insert).bind(ANON_ID).bind(name).bind(days_ago);
        insert.execute(&app.pool).await.unwrap();
    }

    assert_eq!(purge_once(&app.pool).await.unwrap(), 1);
    let left: Vec<String> = rows(&app).await.into_iter().map(|row| row.0).collect();
    assert_eq!(left, ["recent"]);
    // Nothing more to delete.
    assert_eq!(purge_once(&app.pool).await.unwrap(), 0);
}

#[tokio::test]
async fn a_stored_row_holds_neither_the_ip_address_nor_the_user_agent() {
    let app = TestApp::spawn().await;
    let forwarded_for = "198.51.100.77";
    let user_agent = "MarkerAgent/9.9";
    let request = app.client.post(format!("{}/events", app.base_url));
    let request = request
        .header("content-type", "application/json")
        .header("x-forwarded-for", forwarded_for)
        .header("user-agent", user_agent)
        .header("referer", "https://marker-referrer.example/");
    let body = batch(&[landing_view()]).to_string();
    assert_eq!(
        request.body(body).send().await.unwrap().status(),
        StatusCode::NO_CONTENT
    );

    // Every column of the row, as text.
    let row = "SELECT row_to_json(e)::text FROM analytics_events e";
    let stored: Vec<String> = sqlx::query_scalar(row).fetch_all(&app.pool).await.unwrap();
    assert_eq!(stored.len(), 1);
    for marker in [
        forwarded_for,
        user_agent,
        "MarkerAgent",
        "marker-referrer",
        "127.0.0.1",
    ] {
        assert!(!stored[0].contains(marker), "{marker} is in {}", stored[0]);
    }

    // The row has the server's time, taken when the request arrived.
    let fresh = "SELECT count(*) FROM analytics_events WHERE ts > now() - interval '1 minute'";
    let fresh: i64 = sqlx::query_scalar(fresh)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(fresh, 1);
}
