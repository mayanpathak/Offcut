//! The entry point: read the configuration, connect, migrate, serve.

use std::net::{Ipv4Addr, SocketAddr};
use std::process::ExitCode;
use std::sync::Arc;

use offcut_api::billing::adapter::DodoBilling;
use offcut_api::billing::provider::BillingError;
use offcut_api::config::Config;
use offcut_api::mail::MailError;
use offcut_api::mail::smtp_mailer::SmtpMailer;
use offcut_api::rate_limit::RateLimiter;
use offcut_api::state::AppState;
use offcut_api::transcribe::groq::GroqTranscriber;
use offcut_api::transcribe::provider::{TranscribeError, Transcriber, UnconfiguredTranscriber};
use offcut_api::{analytics, db, log, router};
use tokio::net::TcpListener;

/// The most buckets the rate limiter holds (TS §24.5).
const RATE_LIMIT_MAX_KEYS: usize = 50_000;

#[tokio::main]
async fn main() -> ExitCode {
    // One crypto provider for every TLS client here (v6implementation D-281).
    // An error means one is installed already, which is what was asked for.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(error) => {
            // Logging is not set up yet. The error names the variable, never its value.
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    // V6.1 (D-412): the global transcription budget, optional, from its two variables.
    let budget = match offcut_api::config::transcribe_budget(|name| std::env::var(name).ok()) {
        Ok(budget) => budget,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    // V6.2 (D-465): the AI plan's settings. Off by default; on without its key,
    // cap or prices, the boot stops naming the variable. P191 hands them to the route.
    let _plan = match offcut_api::config::plan_settings(|name| std::env::var(name).ok()) {
        Ok(plan) => plan,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    log::init(config.log_level);

    match serve(config, budget).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "the server stopped");
            ExitCode::FAILURE
        }
    }
}

/// What can stop the server. The text of each names the step and, for the
/// database, only the kind of failure: a database error's own text can hold
/// the connection string.
#[derive(Debug, thiserror::Error)]
enum ServeError {
    #[error("cannot connect to the database ({0})")]
    Connect(&'static str),
    #[error("cannot migrate the database: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("cannot set up mail: {0}")]
    Mail(#[from] MailError),
    #[error("cannot set up billing: {0}")]
    Billing(#[from] BillingError),
    #[error("cannot set up transcription: {0}")]
    Transcribe(#[from] TranscribeError),
    #[error("cannot listen or serve: {0}")]
    Io(#[from] std::io::Error),
}

async fn serve(config: Config, (hour_s, day_s): (u32, u32)) -> Result<(), ServeError> {
    let pool = db::connect(&config.database_url)
        .await
        .map_err(|error| ServeError::Connect(offcut_api::error::db_error_kind(&error)))?;
    db::migrate(&pool).await?;
    analytics::retention::spawn_purge_task(pool.clone());

    let mailer = Arc::new(SmtpMailer::new(&config)?);
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port));
    let config = Arc::new(config);
    let billing = Arc::new(DodoBilling::new(config.clone())?);
    // The provider only when the switch is on; off, the route refuses and
    // nothing is ever sent (cloudimplementation D-357, D-381).
    let transcriber: Arc<dyn Transcriber> = if config.transcribe_enabled {
        Arc::new(GroqTranscriber::new(&config)?)
    } else {
        Arc::new(UnconfiguredTranscriber)
    };
    let state = AppState {
        db: pool,
        config,
        limiter: Arc::new(RateLimiter::new(RATE_LIMIT_MAX_KEYS)),
        mailer,
        billing,
        fair_use: Arc::default(),
        transcriber,
        transcribe_budget: Arc::new(offcut_api::transcribe::budget::Budget::new(hour_s, day_s)),
    };
    // The rate limiter needs the peer address of each connection.
    let app = router::build(state).into_make_service_with_connect_info::<SocketAddr>();

    let listener = TcpListener::bind(address).await?;
    tracing::info!(port = address.port(), "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// Completes on SIGTERM, which Render sends before a restart, or on Ctrl-C.
/// Requests in flight are then allowed to finish.
async fn shutdown_signal() {
    tokio::select! {
        () = terminate() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

#[cfg(unix)]
async fn terminate() {
    use tokio::signal::unix::{SignalKind, signal};

    match signal(SignalKind::terminate()) {
        Ok(mut sigterm) => {
            sigterm.recv().await;
        }
        // Without a SIGTERM handler, Ctrl-C is the only way to shut down cleanly.
        Err(_) => std::future::pending().await,
    }
}

#[cfg(not(unix))]
async fn terminate() {
    std::future::pending().await
}
