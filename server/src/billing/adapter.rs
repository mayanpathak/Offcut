//! Dodo Payments (v6implementation §10.2). The mapping below was read from real
//! test-mode deliveries (V6 CL Entry 13), not from its documents alone. This is
//! the only file that knows the provider.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::http::HeaderMap;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use hmac::{Hmac, KeyInit as _, Mac as _};
use offcut_api_types::Offer;
use offcut_types::{UnixSecs, UserId};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::Sha256;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

use crate::billing::offers::Offers;
use crate::billing::provider::{
    BillingError, BillingEvent, BillingEventKind, BillingProvider, EventOwner,
};
use crate::config::Config;

/// The client gives a request 10 s (v6implementation §11.2).
const API_TIMEOUT: Duration = Duration::from_secs(8);
/// Standard Webhooks' tolerance for a delivery's clock.
const SIGNATURE_TOLERANCE_SECS: i64 = 5 * 60;
const SECRET_PREFIX: &str = "whsec_";

pub struct DodoBilling {
    client: reqwest::Client,
    webhook_key: Vec<u8>,
    config: Arc<Config>,
}

impl DodoBilling {
    /// A secret that is not one stops the boot: no webhook could ever be believed.
    pub fn new(config: Arc<Config>) -> Result<DodoBilling, BillingError> {
        let client = reqwest::Client::builder()
            .timeout(API_TIMEOUT)
            .build()
            .map_err(|_| BillingError::Unavailable)?;
        let webhook_key =
            webhook_key(&config.billing_webhook_secret).ok_or(BillingError::Unavailable)?;
        Ok(DodoBilling {
            client,
            webhook_key,
            config,
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.config.billing_api_base_url)
    }

    /// Sends a request and reads the JSON of a success. Neither a refusal's text
    /// nor the error's reaches a log: they can hold a customer's details.
    async fn answer(&self, request: reqwest::RequestBuilder) -> Result<Value, BillingError> {
        let response = request
            .bearer_auth(&self.config.billing_api_key)
            .send()
            .await
            .map_err(|_| unavailable("network"))?;
        if !response.status().is_success() {
            return Err(unavailable("refused"));
        }
        response.json().await.map_err(|_| unavailable("body"))
    }
}

fn unavailable(kind: &'static str) -> BillingError {
    tracing::warn!(kind, "billing request failed");
    BillingError::Unavailable
}

fn webhook_key(secret: &str) -> Option<Vec<u8>> {
    let encoded = secret.strip_prefix(SECRET_PREFIX)?;
    STANDARD.decode(encoded).ok().filter(|key| !key.is_empty())
}

/// An id from the merchant goes into a URL's path: only its own characters.
fn path_id(id: &str) -> Option<&str> {
    let plain = !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    plain.then_some(id)
}

#[async_trait]
impl BillingProvider for DodoBilling {
    async fn create_checkout_url(
        &self,
        user: UserId,
        email: &str,
        offer: Offer,
        return_url: &str,
        cancel_url: &str,
    ) -> Result<String, BillingError> {
        let product = Offers::of(&self.config)
            .product(offer)
            .ok_or(BillingError::Unavailable)?;
        // `metadata` comes back on every subscription event (read in test mode).
        let body = json!({
            "product_cart": [{ "product_id": product, "quantity": 1 }],
            "customer": { "email": email },
            "return_url": return_url,
            "cancel_url": cancel_url,
            "metadata": { "user_id": user.get() },
        });
        let request = self.client.post(self.url("/checkouts")).json(&body);
        let answer = self.answer(request).await?;
        let url = answer.get("checkout_url").and_then(Value::as_str);
        url.map(str::to_owned).ok_or_else(|| unavailable("body"))
    }

    async fn create_portal_url(
        &self,
        customer_id: &str,
        return_url: &str,
    ) -> Result<String, BillingError> {
        let customer = path_id(customer_id).ok_or(BillingError::Unavailable)?;
        let path = format!("/customers/{customer}/customer-portal/session");
        let request = self
            .client
            .post(self.url(&path))
            .query(&[("return_url", return_url)]);
        let answer = self.answer(request).await?;
        let link = answer.get("link").and_then(Value::as_str);
        link.map(str::to_owned).ok_or_else(|| unavailable("body"))
    }

    /// A subscription already cancelled answers 200 as well (read in test mode),
    /// so a retry of a deletion cannot stick on it.
    async fn cancel_subscription(&self, subscription_id: &str) -> Result<(), BillingError> {
        let subscription = path_id(subscription_id).ok_or(BillingError::Unavailable)?;
        let path = format!("/subscriptions/{subscription}");
        let request = self
            .client
            .patch(self.url(&path))
            .json(&json!({ "status": "cancelled" }));
        self.answer(request).await.map(|_| ())
    }

    fn verify_signature(
        &self,
        headers: &HeaderMap,
        raw_body: &[u8],
        now: UnixSecs,
    ) -> Result<(), BillingError> {
        verify(&self.webhook_key, headers, raw_body, now)
    }

    fn parse_event(
        &self,
        headers: &HeaderMap,
        raw_body: &[u8],
    ) -> Result<Option<BillingEvent>, BillingError> {
        let id = header(headers, "webhook-id").ok_or(BillingError::BadEvent)?;
        let envelope: Envelope =
            serde_json::from_slice(raw_body).map_err(|_| BillingError::BadEvent)?;
        parse(id, &envelope, &Offers::of(&self.config))
    }
}

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

/// Standard Webhooks: HMAC-SHA256 over `id.timestamp.body`, against any `v1,`
/// entry of the header, compared in constant time.
fn verify(
    key: &[u8],
    headers: &HeaderMap,
    raw_body: &[u8],
    now: UnixSecs,
) -> Result<(), BillingError> {
    let bad = BillingError::BadSignature;
    let id = header(headers, "webhook-id").ok_or(bad)?;
    let timestamp = header(headers, "webhook-timestamp").ok_or(bad)?;
    let signatures = header(headers, "webhook-signature").ok_or(bad)?;
    let sent: i64 = timestamp.parse().map_err(|_| bad)?;
    if now.get().abs_diff(sent) > SIGNATURE_TOLERANCE_SECS.unsigned_abs() {
        return Err(bad);
    }
    let mac = |signature: &[u8]| {
        let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(key) else {
            return false;
        };
        mac.update(id.as_bytes());
        mac.update(b".");
        mac.update(timestamp.as_bytes());
        mac.update(b".");
        mac.update(raw_body);
        mac.verify_slice(signature).is_ok()
    };
    let good = signatures
        .split_ascii_whitespace()
        .filter_map(|entry| entry.strip_prefix("v1,"))
        .filter_map(|encoded| STANDARD.decode(encoded).ok())
        .any(|signature| mac(&signature));
    if good { Ok(()) } else { Err(bad) }
}

/// A delivery's body. Fields not read here are not looked at.
#[derive(Deserialize)]
struct Envelope {
    #[serde(rename = "type")]
    kind: String,
    timestamp: String,
    data: Value,
}

fn time_of(text: &str) -> Option<UnixSecs> {
    let at = OffsetDateTime::parse(text, &Rfc3339).ok()?;
    Some(UnixSecs::new(at.unix_timestamp()))
}

fn text<'a>(data: &'a Value, field: &str) -> Option<&'a str> {
    data.get(field).and_then(Value::as_str)
}

/// The mapping of v6implementation §10.2, as read in test mode. `Ok(None)` is an
/// event of no interest, including every `payment.*`: the subscription's own
/// events say what matters.
fn parse(
    id: &str,
    envelope: &Envelope,
    offers: &Offers<'_>,
) -> Result<Option<BillingEvent>, BillingError> {
    let bad = BillingError::BadEvent;
    let data = &envelope.data;
    let occurred_at = time_of(&envelope.timestamp).ok_or(bad)?;

    if envelope.kind == "refund.succeeded" {
        // Only a whole refund ends a plan. A refund holds no metadata and names
        // no subscription: only the customer.
        if data.get("is_partial").and_then(Value::as_bool).ok_or(bad)? {
            return Ok(None);
        }
        let customer = customer_of(data).ok_or(bad)?;
        return Ok(Some(BillingEvent {
            provider_event_id: id.to_owned(),
            owner: EventOwner::Customer(customer.clone()),
            customer_id: customer,
            subscription_id: None,
            kind: BillingEventKind::Refunded,
            interval: None,
            current_period_end: None,
            occurred_at,
        }));
    }

    let cancel_scheduled = data
        .get("cancel_at_next_billing_date")
        .and_then(Value::as_bool);
    let kind = match envelope.kind.as_str() {
        "subscription.active" => BillingEventKind::Activated,
        "subscription.renewed" => BillingEventKind::Renewed,
        "subscription.on_hold" | "subscription.past_due" => BillingEventKind::PastDue,
        "subscription.cancelled" | "subscription.expired" => BillingEventKind::Canceled,
        "subscription.plan_changed" => BillingEventKind::PlanChanged,
        // Fires on any change, so its flag is what is read (read in test mode).
        "subscription.updated" => {
            if cancel_scheduled.ok_or(bad)? {
                BillingEventKind::CancelScheduled
            } else {
                BillingEventKind::Resumed
            }
        }
        _ => return Ok(None),
    };

    let product = text(data, "product_id").ok_or(bad)?;
    let Some(interval) = offers.interval(product) else {
        tracing::info!("billing_event_ignored");
        return Ok(None);
    };
    let customer = customer_of(data).ok_or(bad)?;
    let owner = data
        .get("metadata")
        .and_then(|metadata| metadata.get("user_id"))
        .and_then(Value::as_str)
        .and_then(|user| Uuid::parse_str(user).ok())
        .map_or_else(
            || EventOwner::Customer(customer.clone()),
            |user| EventOwner::User(UserId::new(user)),
        );
    // Dodo moves `next_billing_date` before a renewal's charge has gone through
    // (read in test mode). Only the kinds that say a period was paid read it.
    let current_period_end = match kind {
        BillingEventKind::Activated
        | BillingEventKind::Renewed
        | BillingEventKind::Canceled
        | BillingEventKind::PlanChanged => {
            Some(time_of(text(data, "next_billing_date").ok_or(bad)?).ok_or(bad)?)
        }
        BillingEventKind::PastDue
        | BillingEventKind::CancelScheduled
        | BillingEventKind::Resumed
        | BillingEventKind::Refunded => None,
    };
    Ok(Some(BillingEvent {
        provider_event_id: id.to_owned(),
        owner,
        customer_id: customer,
        subscription_id: Some(text(data, "subscription_id").ok_or(bad)?.to_owned()),
        kind,
        interval: Some(interval),
        current_period_end,
        occurred_at,
    }))
}

fn customer_of(data: &Value) -> Option<String> {
    let customer = data.get("customer")?.get("customer_id")?.as_str()?;
    Some(customer.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_secret_is_whsec_and_base64() {
        assert_eq!(webhook_key("whsec_AQID"), Some(vec![1, 2, 3]));
        for bad in ["AQID", "whsec_", "whsec_!!!", "webhook-secret", ""] {
            assert_eq!(webhook_key(bad), None, "{bad}");
        }
    }

    #[test]
    fn an_id_for_a_path_holds_only_its_own_characters() {
        assert_eq!(path_id("sub_0Nr6QLNN1w"), Some("sub_0Nr6QLNN1w"));
        for bad in ["", "../admin", "a/b", "a?b=c", "a b", "a%2Fb"] {
            assert_eq!(path_id(bad), None, "{bad}");
        }
    }
}
