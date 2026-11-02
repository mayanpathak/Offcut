//! What the server asks of a merchant of record (TS §22.5, as reopened by
//! v6implementation §10.1). No other file names the provider.

use async_trait::async_trait;
use axum::http::HeaderMap;
use offcut_api_types::{BillingInterval, Offer};
use offcut_types::{UnixSecs, UserId};

/// Holds no text from the provider: it can hold a customer's details.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum BillingError {
    #[error("the billing provider cannot be used")]
    Unavailable,
    #[error("the webhook's signature is not good")]
    BadSignature,
    #[error("the webhook's body cannot be read")]
    BadEvent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BillingEventKind {
    Activated,
    Renewed,
    PastDue,
    CancelScheduled,
    Canceled,
    Refunded,
    /// A scheduled cancel was taken back (D-301).
    Resumed,
    /// The billing interval changed (D-301).
    PlanChanged,
}

/// Who an event is about. A refund may name only the customer (D-275).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventOwner {
    User(UserId),
    Customer(String),
}

/// A field a kind does not carry is `None`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BillingEvent {
    pub provider_event_id: String,
    pub owner: EventOwner,
    pub customer_id: String,
    pub subscription_id: Option<String>,
    pub kind: BillingEventKind,
    pub interval: Option<BillingInterval>,
    pub current_period_end: Option<UnixSecs>,
    pub occurred_at: UnixSecs,
}

#[async_trait]
pub trait BillingProvider: Send + Sync {
    async fn create_checkout_url(
        &self,
        user: UserId,
        email: &str,
        offer: Offer,
        return_url: &str,
        cancel_url: &str,
    ) -> Result<String, BillingError>;
    async fn create_portal_url(
        &self,
        customer_id: &str,
        return_url: &str,
    ) -> Result<String, BillingError>;
    async fn cancel_subscription(&self, subscription_id: &str) -> Result<(), BillingError>;
    fn verify_signature(
        &self,
        headers: &HeaderMap,
        raw_body: &[u8],
        now: UnixSecs,
    ) -> Result<(), BillingError>;
    /// `Ok(None)` is an event of no interest.
    fn parse_event(
        &self,
        headers: &HeaderMap,
        raw_body: &[u8],
    ) -> Result<Option<BillingEvent>, BillingError>;
}

/// The provider until the real one exists (Prompt 144): every call answers
/// that billing cannot be used, and no webhook is believed.
pub struct UnconfiguredBilling;

#[async_trait]
impl BillingProvider for UnconfiguredBilling {
    async fn create_checkout_url(
        &self,
        _user: UserId,
        _email: &str,
        _offer: Offer,
        _return_url: &str,
        _cancel_url: &str,
    ) -> Result<String, BillingError> {
        Err(BillingError::Unavailable)
    }

    async fn create_portal_url(
        &self,
        _customer_id: &str,
        _return_url: &str,
    ) -> Result<String, BillingError> {
        Err(BillingError::Unavailable)
    }

    async fn cancel_subscription(&self, _subscription_id: &str) -> Result<(), BillingError> {
        Err(BillingError::Unavailable)
    }

    fn verify_signature(
        &self,
        _headers: &HeaderMap,
        _raw_body: &[u8],
        _now: UnixSecs,
    ) -> Result<(), BillingError> {
        Err(BillingError::BadSignature)
    }

    fn parse_event(
        &self,
        _headers: &HeaderMap,
        _raw_body: &[u8],
    ) -> Result<Option<BillingEvent>, BillingError> {
        Err(BillingError::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unconfigured_billing_does_nothing_and_believes_no_webhook() {
        let billing = UnconfiguredBilling;
        let user: UserId =
            serde_json::from_str("\"0190f3a2-7b1c-7def-8a55-0123456789ab\"").unwrap();
        let unavailable = Err(BillingError::Unavailable);

        let checkout =
            billing.create_checkout_url(user, "a@example.com", Offer::CreatorMonthly, "r", "c");
        assert_eq!(checkout.await, unavailable);
        assert_eq!(billing.create_portal_url("c", "r").await, unavailable);
        assert_eq!(
            billing.cancel_subscription("s").await,
            Err(BillingError::Unavailable)
        );

        let headers = HeaderMap::new();
        assert_eq!(
            billing.verify_signature(&headers, b"{}", UnixSecs::new(0)),
            Err(BillingError::BadSignature)
        );
        assert_eq!(
            billing.parse_event(&headers, b"{}"),
            Err(BillingError::Unavailable)
        );
    }
}
