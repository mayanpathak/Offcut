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
