//! What every handler can reach. Nothing here has to survive a restart (TS §1).

use std::sync::Arc;

use crate::billing::provider::BillingProvider;
use crate::config::Config;
use crate::mail::Mailer;
use crate::rate_limit::RateLimiter;
use crate::usage::FairUseLog;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Arc<Config>,
    pub limiter: Arc<RateLimiter>,
    pub mailer: Arc<dyn Mailer>,
    pub billing: Arc<dyn BillingProvider>,
    pub fair_use: Arc<FairUseLog>,
}
