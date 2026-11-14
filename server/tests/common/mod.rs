//! The test application: the real router on a local port, over a database of
//! its own. Each test gets a fresh database, so the tests run in parallel.

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use axum::body::Bytes;
use axum::http::HeaderMap;
use ed25519_dalek::SigningKey;
use offcut_api::billing::provider::{
    BillingError, BillingEvent, BillingEventKind, BillingProvider, EventOwner,
};
use offcut_api::config::{Config, PlanProvider as PlanProviderName, PlanSettings};
use offcut_api::mail::{MailError, Mailer};
use offcut_api::plan::PlanState;
use offcut_api::plan::provider::{
    PlanProvider, PlanProviderError, ProviderOutput, ProviderRequest, Usage,
};
use offcut_api::plan::spend::SpendCap;
use offcut_api::rate_limit::RateLimiter;
use offcut_api::state::AppState;
use offcut_api::transcribe::budget::Budget;
use offcut_api::transcribe::provider::{CloudTranscript, TranscribeError, Transcriber};
use offcut_api::{db, router};
use offcut_api_types::{BillingInterval, CloudWord, Offer};
use offcut_types::{UnixSecs, UserId};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{AssertSqlSafe, ConnectOptions, Connection, PgPool};
use uuid::Uuid;

pub mod plan;

/// Every mail a test's sign-in would have sent, as (to, link). With `fail` set
/// it refuses, as a provider that is down would.
#[derive(Default)]
pub struct FakeMailer {
    pub sent: Mutex<Vec<(String, String)>>,
    pub fail: AtomicBool,
}

#[async_trait]
impl Mailer for FakeMailer {
    async fn send_magic_link(&self, to: &str, link: &str) -> Result<(), MailError> {
        if self.fail.load(Ordering::SeqCst) {
            return Err(MailError::Unavailable);
        }
        self.sent
            .lock()
            .unwrap()
            .push((to.to_owned(), link.to_owned()));
        Ok(())
    }
}

/// A merchant that records its cancels. With `fail` set its API calls are
/// refused. Its webhook is believed with the header `FAKE_SIGNATURE: good`, and
/// its body is `FakeEvent` as JSON, or `{"ignore": true}` for an event of no interest.
#[derive(Default)]
pub struct FakeBilling {
    pub cancelled: Mutex<Vec<String>>,
    pub fail: AtomicBool,
}

impl FakeBilling {
    fn answer<T>(&self, value: T) -> Result<T, BillingError> {
        if self.fail.load(Ordering::SeqCst) {
            Err(BillingError::Unavailable)
        } else {
            Ok(value)
        }
    }
}

#[async_trait]
impl BillingProvider for FakeBilling {
    async fn create_checkout_url(
        &self,
        _user: UserId,
        _email: &str,
        _offer: Offer,
        _return_url: &str,
        _cancel_url: &str,
    ) -> Result<String, BillingError> {
        self.answer("https://merchant.example.com/checkout".to_owned())
    }

    async fn create_portal_url(
        &self,
        _customer_id: &str,
        _return_url: &str,
    ) -> Result<String, BillingError> {
        self.answer("https://merchant.example.com/portal".to_owned())
    }

    async fn cancel_subscription(&self, subscription_id: &str) -> Result<(), BillingError> {
        self.answer(())?;
        self.cancelled
            .lock()
            .unwrap()
            .push(subscription_id.to_owned());
        Ok(())
    }

    fn verify_signature(
        &self,
        headers: &HeaderMap,
        _raw_body: &[u8],
        _now: UnixSecs,
    ) -> Result<(), BillingError> {
        match headers.get(FAKE_SIGNATURE) {
            Some(value) if value == "good" => Ok(()),
            Some(_) | None => Err(BillingError::BadSignature),
        }
    }

    fn parse_event(
        &self,
        _headers: &HeaderMap,
        raw_body: &[u8],
    ) -> Result<Option<BillingEvent>, BillingError> {
        let value: serde_json::Value =
            serde_json::from_slice(raw_body).map_err(|_| BillingError::BadEvent)?;
        if value.get("ignore").is_some() {
            return Ok(None);
        }
        let event: FakeEvent = serde_json::from_value(value).map_err(|_| BillingError::BadEvent)?;
        let owner = match (event.user, event.customer_owner) {
            (Some(user), _) => EventOwner::User(UserId::new(user)),
            (None, Some(customer)) => EventOwner::Customer(customer),
            (None, None) => return Err(BillingError::BadEvent),
        };
        Ok(Some(BillingEvent {
            provider_event_id: event.id,
            owner,
            customer_id: event.customer,
            subscription_id: event.subscription,
            kind: event.kind,
            interval: event.interval,
            current_period_end: event.period_end.map(UnixSecs::new),
            occurred_at: UnixSecs::new(event.at),
        }))
    }
}

/// A transcription provider that records the audio it is given and answers
/// what the test sets: by default the two words "Hello there." over the
/// audio's length. With `enabled` false it is the server's switch turned off.
pub struct FakeTranscriber {
    pub enabled: AtomicBool,
    pub received: Mutex<Vec<Vec<u8>>>,
    pub answer: Mutex<Result<Vec<CloudWord>, TranscribeError>>,
}

impl Default for FakeTranscriber {
    fn default() -> FakeTranscriber {
        let word = |text: &str, start_ms, end_ms| CloudWord {
            text: text.to_owned(),
            start_ms,
            end_ms,
        };
        FakeTranscriber {
            enabled: AtomicBool::new(true),
            received: Mutex::default(),
            answer: Mutex::new(Ok(vec![word("Hello", 100, 400), word("there.", 400, 800)])),
        }
    }
}

#[async_trait]
impl Transcriber for FakeTranscriber {
    fn enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    async fn transcribe(
        &self,
        audio: Bytes,
        duration_ms: u32,
    ) -> Result<CloudTranscript, TranscribeError> {
        self.received.lock().unwrap().push(audio.to_vec());
        let words = self.answer.lock().unwrap().clone()?;
        Ok(CloudTranscript {
            words,
            duration_ms,
            model: "fake:whisper".to_owned(),
        })
    }
}

/// V6.2: the fake's plan when a test sets none: one keyword pop on the
/// first word, a hook, and nothing the model should not write.
pub const FAKE_PLAN: &str = r#"{"format":1,"catalog":2,"cuts":[],"items":[{"id":"1","kind":"keyword_pop","anchors":{"start":0,"end":1},"params":{"kind":"keyword_pop","params":{"word":0}},"confidence":null}],"chapters":[],"extras":{"hook":"A short hook"},"music":null}"#;

/// The usage of each of the fake's answers: TE-15's median Flash plan.
pub const FAKE_USAGE: Usage = Usage {
    input_tokens: 5_465,
    cached_input_tokens: 4_224,
    output_tokens: 492,
};

/// An AI plan provider that records what it is sent (system, user) and
/// answers from `answers` in turn, then with `FAKE_PLAN`; `delay` holds each
/// answer back. With `enabled` false it is the server's switch turned off.
pub struct FakePlanProvider {
    pub enabled: AtomicBool,
    pub received: Mutex<Vec<(String, String)>>,
    pub answers: Mutex<VecDeque<Result<String, PlanProviderError>>>,
    pub delay: Mutex<Duration>,
}

impl Default for FakePlanProvider {
    fn default() -> FakePlanProvider {
        FakePlanProvider {
            enabled: AtomicBool::new(true),
            received: Mutex::default(),
            answers: Mutex::default(),
            delay: Mutex::new(Duration::ZERO),
        }
    }
}

#[async_trait]
impl PlanProvider for FakePlanProvider {
    fn enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    async fn complete(
        &self,
        request: ProviderRequest,
    ) -> Result<ProviderOutput, PlanProviderError> {
        self.received
            .lock()
            .unwrap()
            .push((request.system, request.user));
        let delay = *self.delay.lock().unwrap();
        tokio::time::sleep(delay).await;
        let answer = self.answers.lock().unwrap().pop_front();
        let text = answer.unwrap_or_else(|| Ok(FAKE_PLAN.to_owned()))?;
        Ok(ProviderOutput {
            text,
            usage: FAKE_USAGE,
            model: "fake-model".to_owned(),
        })
    }
}

/// What a test may change about the app before it starts.
pub struct TestOptions {
    pub transcribe_enabled: bool,
    /// The audio budget in seconds: (hour, day).
    pub transcribe_budget: (u32, u32),
    /// V6.2: the AI plan's switch and its monthly cap, in micro-dollars.
    pub plan_enabled: bool,
    pub plan_cap_micro_usd: i64,
}

impl Default for TestOptions {
    fn default() -> TestOptions {
        TestOptions {
            transcribe_enabled: true,
            transcribe_budget: (7_000, 27_000),
            plan_enabled: true,
            plan_cap_micro_usd: 5_000_000,
        }
    }
}

/// The plan settings the tests run with: Flash's prices (CP 1), the cap given.
pub fn plan_settings(cap_micro_usd: i64) -> PlanSettings {
    PlanSettings {
        enabled: true,
        provider: PlanProviderName::Deepseek,
        model: "fake-model".to_owned(),
        api_key: "unset-in-tests".to_owned(),
        api_base_url: "https://plan.example.com".to_owned(),
        monthly_cap_micro_usd: cap_micro_usd,
        price_in_usd_per_mtok: 0.30,
        price_out_usd_per_mtok: 1.20,
    }
}

/// The header the fake's webhook is signed with.
pub const FAKE_SIGNATURE: &str = "x-fake-signature";

/// The fake merchant's webhook body: `BillingEvent`, as plain JSON.
#[derive(serde::Deserialize)]
struct FakeEvent {
    id: String,
    user: Option<Uuid>,
    customer_owner: Option<String>,
    customer: String,
    subscription: Option<String>,
    #[serde(deserialize_with = "kind_of")]
    kind: BillingEventKind,
    #[serde(default)]
    interval: Option<BillingInterval>,
    period_end: Option<i64>,
    at: i64,
}

fn kind_of<'de, D: serde::Deserializer<'de>>(reader: D) -> Result<BillingEventKind, D::Error> {
    let text: String = serde::Deserialize::deserialize(reader)?;
    match text.as_str() {
        "activated" => Ok(BillingEventKind::Activated),
        "renewed" => Ok(BillingEventKind::Renewed),
        "past_due" => Ok(BillingEventKind::PastDue),
        "cancel_scheduled" => Ok(BillingEventKind::CancelScheduled),
        "canceled" => Ok(BillingEventKind::Canceled),
        "refunded" => Ok(BillingEventKind::Refunded),
        "resumed" => Ok(BillingEventKind::Resumed),
        "plan_changed" => Ok(BillingEventKind::PlanChanged),
        other => Err(serde::de::Error::custom(other.to_owned())),
    }
}

/// A webhook from the fake merchant, signed.
pub async fn webhook(app: &TestApp, body: &serde_json::Value) -> reqwest::Response {
    app.client
        .post(format!("{}/billing/webhook", app.base_url))
        .header(FAKE_SIGNATURE, "good")
        .json(body)
        .send()
        .await
        .unwrap()
}

pub struct TestApp {
    /// `http://127.0.0.1:<port>/api/v1`
    pub base_url: String,
    pub pool: PgPool,
    pub client: reqwest::Client,
    pub mailer: Arc<FakeMailer>,
    pub billing: Arc<FakeBilling>,
    pub transcriber: Arc<FakeTranscriber>,
    pub plan: Arc<FakePlanProvider>,
    database: String,
}

impl TestApp {
    /// Creates a database with a random name on the server that
    /// `DATABASE_URL` names, migrates it, and serves the router over it. The
    /// role in `DATABASE_URL` needs `CREATEDB`.
    pub async fn spawn() -> TestApp {
        TestApp::spawn_with(TestOptions::default()).await
    }

    /// As `spawn`, with `options`.
    pub async fn spawn_with(options: TestOptions) -> TestApp {
        // As `main.rs` does. Only the first call in a process installs it.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let server = server_options();
        let database = format!("offcut_test_{}", Uuid::new_v4().simple());
        let mut admin = server.connect().await.expect("connect to DATABASE_URL");
        sqlx::query(AssertSqlSafe(format!(r#"CREATE DATABASE "{database}""#)))
            .execute(&mut admin)
            .await
            .expect("create the test database");
        admin.close().await.unwrap();

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(server.database(&database))
            .await
            .unwrap();
        db::migrate(&pool).await.unwrap();

        let mailer = Arc::new(FakeMailer::default());
        let billing = Arc::new(FakeBilling::default());
        let transcriber = Arc::new(FakeTranscriber::default());
        transcriber
            .enabled
            .store(options.transcribe_enabled, Ordering::SeqCst);
        let (hour, day) = options.transcribe_budget;
        let plan = Arc::new(FakePlanProvider::default());
        plan.enabled.store(options.plan_enabled, Ordering::SeqCst);
        let plan_state = Arc::new(PlanState {
            provider: plan.clone(),
            provider_id: "fake",
            cap: SpendCap::new(&plan_settings(options.plan_cap_micro_usd)),
        });
        let state = AppState {
            db: pool.clone(),
            config: Arc::new(test_config()),
            limiter: Arc::new(RateLimiter::new(50_000)),
            mailer: mailer.clone(),
            billing: billing.clone(),
            fair_use: Arc::default(),
            transcriber: transcriber.clone(),
            transcribe_budget: Arc::new(Budget::new(hour, day)),
            plan: plan_state,
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = router::build(state).into_make_service_with_connect_info::<SocketAddr>();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        TestApp {
            base_url: format!("http://{address}/api/v1"),
            pool,
            // A proxy configured on the machine must not see local requests.
            client: reqwest::Client::builder().no_proxy().build().unwrap(),
            mailer,
            billing,
            transcriber,
            plan,
            database,
        }
    }
}

/// Drops the test's database. `Drop` cannot await and the test's runtime may
/// be shutting down, so this runs on a thread with a runtime of its own.
impl Drop for TestApp {
    fn drop(&mut self) {
        // Each test binary reads what it needs of these. These reads keep the
        // ones a binary does not use from being called dead.
        let _ = (
            &self.pool,
            &self.mailer,
            &self.billing,
            &self.transcriber,
            &self.plan,
        );
        let _ = (
            FAKE_PLAN,
            FAKE_USAGE,
            plan_settings as fn(i64) -> PlanSettings,
        );
        let _: fn(TestOptions) -> _ = TestApp::spawn_with;
        let _: fn(&'static TestApp, &'static serde_json::Value) -> _ = webhook;
        let drop_database = format!(
            r#"DROP DATABASE IF EXISTS "{}" WITH (FORCE)"#,
            self.database
        );
        let cleanup = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            runtime.block_on(async {
                let mut admin = server_options().connect().await?;
                sqlx::query(AssertSqlSafe(drop_database))
                    .execute(&mut admin)
                    .await?;
                admin.close().await
            })?;
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        });
        // A database that could not be dropped is left behind; the test's own
        // result stands.
        let _ = cleanup.join();
    }
}

fn server_options() -> PgConnectOptions {
    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL is not set. Load it first: set -a; . ./.env; set +a");
    PgConnectOptions::from_str(&url).expect("DATABASE_URL is not a Postgres URL")
}

/// Fixed values. Nothing here is a real key or a real secret.
fn test_config() -> Config {
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
        transcribe_api_key: placeholder(),
        transcribe_api_base_url: "https://api.transcribe.example.com/openai/v1".to_owned(),
        // One proxy in front, as in production behind the Vercel rewrite: the
        // rate limiter reads `X-Forwarded-For` when a request carries it.
        trusted_proxy_hops: 1,
        git_sha: "test-sha".to_owned(),
    }
}
