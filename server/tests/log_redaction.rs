#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! What the logs never hold (TS §24.4). The flows run with marker values, at a
//! level louder than production's, and none may appear. This binary has one
//! test, as a log subscriber is process-wide.

mod common;

use std::io::Write;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use common::{TestApp, webhook};
use ed25519_dalek::SigningKey;
use offcut_api::auth::access_token;
use offcut_api::log::user_tag;
use offcut_types::UnixSecs;
use reqwest::StatusCode;
use serde_json::{Value, json};
use time::OffsetDateTime;
use tracing_subscriber::fmt::MakeWriter;
use uuid::Uuid;

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

const EMAIL: &str = "marker.person@marker-mail.example";
const WAITLIST_EMAIL: &str = "marker.waitlist@marker-mail.example";
const FAILED_MAIL_EMAIL: &str = "marker.failed@marker-mail.example";
const CLIENT_IP: &str = "203.0.113.77";
const QUERY_MARKER: &str = "querymarkervalue";
const PATH_MARKER: &str = "pathmarkervalue";
const ORIGIN: &str = "http://localhost:5173";

#[tokio::test]
async fn no_flow_writes_a_secret_or_a_person_to_the_log() {
    let logs = Captured::default();
    tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(logs.clone())
        .init();

    let app = TestApp::spawn().await;
    let url = |path: &str| format!("{}{path}", app.base_url);
    let mut markers: Vec<(&str, String)> = vec![
        ("the address", EMAIL.to_owned()),
        ("the address's domain", "marker-mail".to_owned()),
        ("the client address", CLIENT_IP.to_owned()),
        ("a query string", QUERY_MARKER.to_owned()),
        ("a raw path", PATH_MARKER.to_owned()),
    ];

    // Sign-in.
    let response = app
        .client
        .post(url("/auth/magic-link"))
        .header("x-forwarded-for", CLIENT_IP)
        .json(&json!({ "email": EMAIL }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let link = app.mailer.sent.lock().unwrap().last().unwrap().1.clone();
    let token = link.rsplit_once("token=").unwrap().1.to_owned();
    markers.push(("the link's token", token.clone()));

    let response = app
        .client
        .post(url("/auth/verify"))
        .header("x-forwarded-for", CLIENT_IP)
        .json(&json!({ "token": token }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
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
    let session: Value = response.json().await.unwrap();
    let access = session["access_token"].as_str().unwrap().to_owned();
    let entitlement = session["entitlement_token"].as_str().unwrap().to_owned();
    markers.push(("the refresh token", refresh.clone()));
    markers.push(("the access token", access.clone()));
    markers.push(("the entitlement token", entitlement));

    let key = SigningKey::from_bytes(&[2; 32]).verifying_key();
    let now = UnixSecs::new(OffsetDateTime::now_utc().unix_timestamp());
    let claims = access_token::verify(&access, &key, now).unwrap();
    markers.push(("the user's id", claims.sub.get().to_string()));
    markers.push(("the session's id", claims.sid.to_string()));

    // A refresh that rotates, and one with a wrong cookie.
    let response = app
        .client
        .post(url("/auth/refresh"))
        .header("cookie", format!("offcut_rt={refresh}"))
        .header("x-offcut-csrf", "1")
        .header("origin", ORIGIN)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let rotated = response.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .to_owned();
    let rotated = rotated
        .strip_prefix("offcut_rt=")
        .unwrap()
        .split_once(';')
        .unwrap()
        .0
        .to_owned();
    markers.push(("the rotated token", rotated.clone()));
    let wrong = "markerwrongrefreshtoken";
    markers.push(("a wrong refresh token", wrong.to_owned()));
    let response = app
        .client
        .post(url("/auth/refresh"))
        .header("cookie", format!("offcut_rt={wrong}"))
        .header("x-offcut-csrf", "1")
        .header("origin", ORIGIN)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // The account's own routes, and a receipt.
    let request = app.client.get(url("/me")).bearer_auth(&access);
    assert_eq!(request.send().await.unwrap().status(), StatusCode::OK);
    let request = app.client.get(url("/entitlement")).bearer_auth(&access);
    assert_eq!(request.send().await.unwrap().status(), StatusCode::OK);
    let export_id = Uuid::now_v7();
    markers.push(("an export id", export_id.to_string()));
    let request = app.client.post(url("/usage/receipts")).bearer_auth(&access);
    let response = request
        .json(&json!({ "export_id": export_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let request = app.client.get(url("/account/export")).bearer_auth(&access);
    assert_eq!(request.send().await.unwrap().status(), StatusCode::OK);

    // Analytics and the waitlist.
    let anon_id = Uuid::new_v4();
    markers.push(("an anonymous id", anon_id.to_string()));
    let batch = json!({
        "anon_id": anon_id,
        "events": [{ "name": "landing_view", "props": { "hero_variant": "outcome" } }],
    });
    let response = app
        .client
        .post(url("/events"))
        .json(&batch)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    markers.push(("a waitlist address", WAITLIST_EMAIL.to_owned()));
    let body = json!({ "email": WAITLIST_EMAIL, "wanted": "launch" });
    let response = app
        .client
        .post(url("/notify-me"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // A mail that fails, a bad body and paths that match nothing.
    app.mailer.fail.store(true, Ordering::SeqCst);
    markers.push(("an address whose mail failed", FAILED_MAIL_EMAIL.to_owned()));
    let body = json!({ "email": FAILED_MAIL_EMAIL });
    let response = app
        .client
        .post(url("/auth/magic-link"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let bad = "markerinabadbody";
    markers.push(("a bad body", bad.to_owned()));
    let response = app
        .client
        .post(url("/auth/verify"))
        .header("content-type", "application/json")
        .body(format!("{{\"token\":\"{bad}\",\"extra\":\"{bad}\"}}"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let nowhere = url(&format!("/{PATH_MARKER}?secret={QUERY_MARKER}"));
    assert_eq!(
        app.client.get(nowhere).send().await.unwrap().status(),
        StatusCode::NOT_FOUND
    );

    // A webhook applied, one ignored, and one with a bad signature.
    let customer = "cus_markercustomer";
    let subscription = "sub_markersubscription";
    let event_id = "evt_markerevent";
    markers.push(("a merchant's customer", customer.to_owned()));
    markers.push(("a merchant's subscription", subscription.to_owned()));
    markers.push(("a merchant's event", event_id.to_owned()));
    let event = json!({
        "id": event_id,
        "user": claims.sub.get(),
        "customer": customer,
        "subscription": subscription,
        "kind": "activated",
        "interval": "monthly",
        "period_end": now.get() + 1000,
        "at": now.get(),
    });
    assert_eq!(webhook(&app, &event).await.status(), StatusCode::OK);
    let mut ignored = event.clone();
    ignored["id"] = json!("evt_markerignored");
    markers.push(("an ignored event", "evt_markerignored".to_owned()));
    assert_eq!(webhook(&app, &ignored).await.status(), StatusCode::OK);
    let unsigned = app.client.post(url("/billing/webhook")).json(&event);
    assert_eq!(
        unsigned.send().await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );

    // Cloud transcription (P-16): the words returned and the audio sent.
    let spoken = "markerspokenword";
    markers.push(("a word of a transcript", spoken.to_owned()));
    *app.transcriber.answer.lock().unwrap() = Ok(vec![offcut_api_types::CloudWord {
        text: spoken.to_owned(),
        start_ms: 0,
        end_ms: 300,
    }]);
    let mut audio = include_bytes!("fixtures/ogg/ffmpeg_tone_3s.ogg").to_vec();
    // A marker inside the last page's data: the file still reads, and its bytes must not be logged.
    let marker_bytes = "markeraudiobytes";
    markers.push(("the audio's bytes", marker_bytes.to_owned()));
    let at = audio.len() - marker_bytes.len();
    audio[at..].copy_from_slice(marker_bytes.as_bytes());
    let request = app.client.post(url("/transcribe")).bearer_auth(&access);
    let response = request
        .header("content-type", "audio/ogg")
        .body(audio)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().await.unwrap().contains(spoken));

    // Deleting the account.
    app.billing.fail.store(false, Ordering::SeqCst);
    let request = app.client.post(url("/account/delete")).bearer_auth(&access);
    assert_eq!(
        request.send().await.unwrap().status(),
        StatusCode::NO_CONTENT
    );

    let text = String::from_utf8(logs.0.lock().unwrap().clone()).unwrap();
    for (what, marker) in &markers {
        assert!(!text.contains(marker.as_str()), "{what} is in the log");
    }

    // The test is worth nothing if the log is empty: the request lines are there,
    // with the route's template and the user's tag.
    assert!(text.contains("/api/v1/auth/verify"), "{text}");
    assert!(text.contains("/api/v1/me"), "{text}");
    assert!(text.contains(&user_tag(claims.sub)), "{text}");
    assert!(text.contains("billing_event_ignored"), "{text}");
    assert!(
        text.contains("/api/v1/transcribe") && text.contains("\"kind\":\"ok\""),
        "{text}"
    );
}
