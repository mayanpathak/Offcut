//! The entry point: read the configuration, connect, migrate, serve.

use std::net::{Ipv4Addr, SocketAddr};
use std::process::ExitCode;
use std::sync::Arc;

use offcut_api::config::Config;
use offcut_api::rate_limit::RateLimiter;
use offcut_api::state::AppState;
use offcut_api::{analytics, db, log, router};
use tokio::net::TcpListener;

/// The most buckets the rate limiter holds (TS §24.5).
const RATE_LIMIT_MAX_KEYS: usize = 50_000;

#[tokio::main]
async fn main() -> ExitCode {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(error) => {
            // Logging is not set up yet. The error names the variable, never its value.
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    log::init(config.log_level);

    match serve(config).await {
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
    #[error("cannot listen or serve: {0}")]
    Io(#[from] std::io::Error),
}

async fn serve(config: Config) -> Result<(), ServeError> {
    let pool = db::connect(&config.database_url)
        .await
        .map_err(|error| ServeError::Connect(offcut_api::error::db_error_kind(&error)))?;
    db::migrate(&pool).await?;
    analytics::retention::spawn_purge_task(pool.clone());

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port));
    let state = AppState {
        db: pool,
        config: Arc::new(config),
        limiter: Arc::new(RateLimiter::new(RATE_LIMIT_MAX_KEYS)),
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
