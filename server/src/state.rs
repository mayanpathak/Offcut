//! What every handler can reach. Nothing here has to survive a restart (TS §1).

use std::sync::Arc;

use crate::config::Config;
use crate::rate_limit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Arc<Config>,
    pub limiter: Arc<RateLimiter>,
}
