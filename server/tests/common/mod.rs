//! The test application: the real router on a local port, over a database of
//! its own. Each test gets a fresh database, so the tests run in parallel.

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use ed25519_dalek::SigningKey;
use offcut_api::config::Config;
use offcut_api::rate_limit::RateLimiter;
use offcut_api::state::AppState;
use offcut_api::{db, router};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{AssertSqlSafe, ConnectOptions, Connection, PgPool};
use uuid::Uuid;

pub struct TestApp {
    /// `http://127.0.0.1:<port>/api/v1`
    pub base_url: String,
    pub pool: PgPool,
    pub client: reqwest::Client,
    database: String,
}

impl TestApp {
    /// Creates a database with a random name on the server that
    /// `DATABASE_URL` names, migrates it, and serves the router over it. The
    /// role in `DATABASE_URL` needs `CREATEDB`.
    pub async fn spawn() -> TestApp {
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

        let state = AppState {
            db: pool.clone(),
            config: Arc::new(test_config()),
            limiter: Arc::new(RateLimiter::new(50_000)),
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
            database,
        }
    }
}

/// Drops the test's database. `Drop` cannot await and the test's runtime may
/// be shutting down, so this runs on a thread with a runtime of its own.
impl Drop for TestApp {
    fn drop(&mut self) {
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
        mail_api_key: placeholder(),
        mail_from: placeholder(),
        billing_api_key: placeholder(),
        billing_webhook_secret: placeholder(),
        billing_price_creator_monthly: placeholder(),
        billing_price_creator_annual: placeholder(),
        billing_price_creator_annual_founding: placeholder(),
        founding_offer_enabled: false,
        // One proxy in front, as in production behind the Vercel rewrite: the
        // rate limiter reads `X-Forwarded-For` when a request carries it.
        trusted_proxy_hops: 1,
        git_sha: "test-sha".to_owned(),
    }
}
