#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The real provider's adapter, with no network. Its fixtures are real test-mode
//! deliveries with every person's details and id replaced (V6 CL Entry 13); this
//! test signs them with a test secret.

use std::sync::Arc;

use axum::http::{HeaderMap, HeaderValue};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use ed25519_dalek::SigningKey;
use hmac::{Hmac, KeyInit as _, Mac as _};
use offcut_api::billing::adapter::DodoBilling;
use offcut_api::billing::provider::{
    BillingError, BillingEvent, BillingEventKind, BillingProvider, EventOwner,
};
use offcut_api::config::Config;
use offcut_api_types::BillingInterval;
use offcut_types::UnixSecs;
use sha2::Sha256;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

const KEY: [u8; 3] = [1, 2, 3];
const SECRET: &str = "whsec_AQID";
const NOW: i64 = 1_793_660_000;
const USER: &str = "0190f3a2-7b1c-7def-8a55-0123456789ab";
const CUSTOMER: &str = "cus_test";
const SUBSCRIPTION: &str = "sub_test";

fn config(secret: &str) -> Arc<Config> {
    // As `main.rs` does. Only the first call in a process installs it.
    let _ = rustls::crypto::ring::default_provider().install_default();
    let placeholder = || "unset-in-tests".to_owned();
    Arc::new(Config {
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
        billing_api_base_url: "https://billing.invalid".to_owned(),
        billing_api_key: placeholder(),
        billing_webhook_secret: secret.to_owned(),
        billing_price_creator_monthly: "prod_monthly".to_owned(),
        billing_price_creator_annual: "prod_annual".to_owned(),
        billing_price_creator_annual_founding: "prod_founding".to_owned(),
        founding_offer_enabled: false,
        transcribe_enabled: false,
        transcribe_api_key: "unset-in-tests".to_owned(),
        transcribe_api_base_url: "unset-in-tests".to_owned(),
        trusted_proxy_hops: 1,
        git_sha: "test-sha".to_owned(),
    })
}

fn adapter() -> DodoBilling {
    DodoBilling::new(config(SECRET)).unwrap()
}

fn signed_headers(id: &str, timestamp: i64, body: &[u8], key: &[u8]) -> HeaderMap {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).unwrap();
    mac.update(format!("{id}.{timestamp}.").as_bytes());
    mac.update(body);
    let signature = STANDARD.encode(mac.finalize().into_bytes());
    let mut headers = HeaderMap::new();
    headers.insert("webhook-id", HeaderValue::from_str(id).unwrap());
    headers.insert(
        "webhook-timestamp",
        HeaderValue::from_str(&timestamp.to_string()).unwrap(),
    );
    headers.insert(
        "webhook-signature",
        HeaderValue::from_str(&format!("v1,{signature}")).unwrap(),
    );
    headers
}

fn fixture(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/tests/fixtures/dodo/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(&path).unwrap_or_else(|_| panic!("{path}"))
}

/// A fixture as the adapter reads it, once its signature is believed.
fn parsed(name: &str) -> Option<BillingEvent> {
    let body = fixture(name);
    let headers = signed_headers("msg_test", NOW, &body, &KEY);
    let adapter = adapter();
    adapter
        .verify_signature(&headers, &body, UnixSecs::new(NOW))
        .unwrap();
    adapter.parse_event(&headers, &body).unwrap()
}

fn seconds(text: &str) -> i64 {
    OffsetDateTime::parse(text, &Rfc3339)
        .unwrap()
        .unix_timestamp()
}

fn owner_user() -> EventOwner {
    EventOwner::User(serde_json::from_value(serde_json::json!(USER)).unwrap())
}

fn field(name: &str, field: &str) -> String {
    let value: serde_json::Value = serde_json::from_slice(&fixture(name)).unwrap();
    value["data"][field].as_str().unwrap().to_owned()
}

// --- The mapping --------------------------------------------------------------

#[test]
fn subscription_active_is_activated_with_the_user_the_period_and_the_interval() {
    let event = parsed("subscription_active").unwrap();
    assert_eq!(event.kind, BillingEventKind::Activated);
    assert_eq!(event.provider_event_id, "msg_test");
    assert_eq!(event.owner, owner_user());
    assert_eq!(event.customer_id, CUSTOMER);
    assert_eq!(event.subscription_id.as_deref(), Some(SUBSCRIPTION));
    assert_eq!(event.interval, Some(BillingInterval::Monthly));
    let next = seconds(&field("subscription_active", "next_billing_date"));
    assert_eq!(event.current_period_end.unwrap().get(), next);
}

#[test]
fn subscription_renewed_is_renewed_with_its_new_period() {
    let event = parsed("subscription_renewed").unwrap();
    assert_eq!(event.kind, BillingEventKind::Renewed);
    let next = seconds(&field("subscription_renewed", "next_billing_date"));
    assert_eq!(event.current_period_end.unwrap().get(), next);
}

#[test]
fn subscription_on_hold_is_past_due_and_does_not_move_the_paid_period() {
    // Dodo has already moved `next_billing_date` here, with the charge declined.
    let event = parsed("subscription_on_hold").unwrap();
    assert_eq!(event.kind, BillingEventKind::PastDue);
    assert_eq!(event.current_period_end, None);
}

#[test]
fn subscription_updated_says_a_cancel_scheduled_or_taken_back_by_its_flag() {
    let scheduled = parsed("subscription_updated_cancel_scheduled").unwrap();
    assert_eq!(scheduled.kind, BillingEventKind::CancelScheduled);
    assert_eq!(scheduled.current_period_end, None);
    let resumed = parsed("subscription_updated").unwrap();
    assert_eq!(resumed.kind, BillingEventKind::Resumed);
    assert_eq!(resumed.current_period_end, None);
}

#[test]
fn subscription_cancelled_is_canceled_keeping_the_time_paid_for() {
    let event = parsed("subscription_cancelled").unwrap();
    assert_eq!(event.kind, BillingEventKind::Canceled);
    let next = seconds(&field("subscription_cancelled", "next_billing_date"));
    assert_eq!(event.current_period_end.unwrap().get(), next);
}

#[test]
fn a_whole_refund_is_refunded_and_names_only_the_customer() {
    let event = parsed("refund_succeeded").unwrap();
    assert_eq!(event.kind, BillingEventKind::Refunded);
    assert_eq!(event.owner, EventOwner::Customer(CUSTOMER.to_owned()));
    assert_eq!(event.subscription_id, None);
    assert_eq!(event.current_period_end, None);
}

#[test]
fn a_partial_refund_ends_nothing() {
    let mut value: serde_json::Value =
        serde_json::from_slice(&fixture("refund_succeeded")).unwrap();
    value["data"]["is_partial"] = serde_json::json!(true);
    let body = serde_json::to_vec(&value).unwrap();
    let headers = signed_headers("msg_test", NOW, &body, &KEY);
    assert_eq!(adapter().parse_event(&headers, &body).unwrap(), None);
}

#[test]
fn events_of_no_interest_parse_to_nothing() {
    for name in [
        "payment_succeeded",
        "payment_failed",
        "subscription_failed",
        "subscription_update_payment_method",
    ] {
        assert_eq!(parsed(name), None, "{name}");
    }
}

#[test]
fn a_product_no_offer_names_is_ignored() {
    let mut value: serde_json::Value =
        serde_json::from_slice(&fixture("subscription_active")).unwrap();
    value["data"]["product_id"] = serde_json::json!("prod_someone_elses");
    let body = serde_json::to_vec(&value).unwrap();
    let headers = signed_headers("msg_test", NOW, &body, &KEY);
    assert_eq!(adapter().parse_event(&headers, &body).unwrap(), None);
}

#[test]
fn a_subscription_with_no_user_id_falls_back_to_its_customer() {
    let mut value: serde_json::Value =
        serde_json::from_slice(&fixture("subscription_renewed")).unwrap();
    value["data"]["metadata"] = serde_json::json!({});
    let body = serde_json::to_vec(&value).unwrap();
    let headers = signed_headers("msg_test", NOW, &body, &KEY);
    let event = adapter().parse_event(&headers, &body).unwrap().unwrap();
    assert_eq!(event.owner, EventOwner::Customer(CUSTOMER.to_owned()));
}

#[test]
fn a_body_that_cannot_be_read_is_bad() {
    let headers = signed_headers("msg_test", NOW, b"x", &KEY);
    assert_eq!(
        adapter().parse_event(&headers, b"not json"),
        Err(BillingError::BadEvent)
    );

    let missing = serde_json::to_vec(&serde_json::json!({
        "type": "subscription.active",
        "timestamp": "2026-11-02T22:05:55Z",
        "data": {},
    }))
    .unwrap();
    assert_eq!(
        adapter().parse_event(&headers, &missing),
        Err(BillingError::BadEvent)
    );

    let bad_time = serde_json::to_vec(&serde_json::json!({
        "type": "subscription.active",
        "timestamp": "yesterday",
        "data": {},
    }))
    .unwrap();
    assert_eq!(
        adapter().parse_event(&headers, &bad_time),
        Err(BillingError::BadEvent)
    );

    let mut no_id = headers.clone();
    no_id.remove("webhook-id");
    assert_eq!(
        adapter().parse_event(&no_id, &fixture("subscription_active")),
        Err(BillingError::BadEvent)
    );
}

// --- The signature ------------------------------------------------------------

/// The example the Standard Webhooks specification publishes.
#[test]
fn the_published_vector_of_standard_webhooks_verifies() {
    let adapter = DodoBilling::new(config("whsec_MfKQ9r8GKYqrTwjUPD8ILPZIo2LaLaSw")).unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(
        "webhook-id",
        HeaderValue::from_static("msg_p5jXN8AQM9LWM0D4loKWxJek"),
    );
    headers.insert("webhook-timestamp", HeaderValue::from_static("1614265330"));
    headers.insert(
        "webhook-signature",
        HeaderValue::from_static("v1,g0hM9SsE+OTPJTGt/tmIKtSyZlE3uFJELVlNIOLJ1OE="),
    );
    let body = br#"{"test": 2432232314}"#;
    assert_eq!(
        adapter.verify_signature(&headers, body, UnixSecs::new(1_614_265_330)),
        Ok(())
    );
    assert_eq!(
        adapter.verify_signature(&headers, b"{\"test\": 1}", UnixSecs::new(1_614_265_330)),
        Err(BillingError::BadSignature)
    );
}

#[test]
fn a_body_changed_by_one_byte_is_refused() {
    let body = fixture("subscription_active");
    let headers = signed_headers("msg_test", NOW, &body, &KEY);
    let mut changed = body.clone();
    changed[10] ^= 1;
    assert_eq!(
        adapter().verify_signature(&headers, &changed, UnixSecs::new(NOW)),
        Err(BillingError::BadSignature)
    );
}

#[test]
fn a_wrong_secret_or_a_changed_id_is_refused() {
    let body = fixture("subscription_active");
    let other = signed_headers("msg_test", NOW, &body, &[9, 9, 9]);
    assert_eq!(
        adapter().verify_signature(&other, &body, UnixSecs::new(NOW)),
        Err(BillingError::BadSignature)
    );
    let mut moved = signed_headers("msg_test", NOW, &body, &KEY);
    moved.insert("webhook-id", HeaderValue::from_static("msg_other"));
    assert_eq!(
        adapter().verify_signature(&moved, &body, UnixSecs::new(NOW)),
        Err(BillingError::BadSignature)
    );
}

#[test]
fn a_delivery_more_than_five_minutes_from_now_is_refused() {
    let body = fixture("subscription_active");
    let check = |sent: i64| {
        let headers = signed_headers("msg_test", sent, &body, &KEY);
        adapter().verify_signature(&headers, &body, UnixSecs::new(NOW))
    };
    assert_eq!(check(NOW - 300), Ok(()));
    assert_eq!(check(NOW + 300), Ok(()));
    assert_eq!(check(NOW - 301), Err(BillingError::BadSignature));
    assert_eq!(check(NOW + 301), Err(BillingError::BadSignature));
}

#[test]
fn any_v1_entry_of_the_header_can_verify_during_a_secret_rotation() {
    let body = fixture("subscription_active");
    let good = signed_headers("msg_test", NOW, &body, &KEY);
    let entry = good["webhook-signature"].to_str().unwrap().to_owned();
    let mut headers = good.clone();
    let both = format!("v1,AAAA v2,BBBB {entry}");
    headers.insert("webhook-signature", HeaderValue::from_str(&both).unwrap());
    assert_eq!(
        adapter().verify_signature(&headers, &body, UnixSecs::new(NOW)),
        Ok(())
    );
}

#[test]
fn missing_or_malformed_headers_are_refused() {
    let body = fixture("subscription_active");
    let good = signed_headers("msg_test", NOW, &body, &KEY);
    for name in ["webhook-id", "webhook-timestamp", "webhook-signature"] {
        let mut headers = good.clone();
        headers.remove(name);
        assert_eq!(
            adapter().verify_signature(&headers, &body, UnixSecs::new(NOW)),
            Err(BillingError::BadSignature),
            "{name}"
        );
    }
    let mut late = good.clone();
    late.insert("webhook-timestamp", HeaderValue::from_static("soon"));
    assert_eq!(
        adapter().verify_signature(&late, &body, UnixSecs::new(NOW)),
        Err(BillingError::BadSignature)
    );
    let mut garbled = good;
    garbled.insert("webhook-signature", HeaderValue::from_static("v1,!!!"));
    assert_eq!(
        adapter().verify_signature(&garbled, &body, UnixSecs::new(NOW)),
        Err(BillingError::BadSignature)
    );
}

#[test]
fn a_secret_that_is_not_one_stops_the_boot() {
    for secret in ["webhook-secret", "whsec_", "whsec_!!!"] {
        assert!(DodoBilling::new(config(secret)).is_err(), "{secret}");
    }
}
