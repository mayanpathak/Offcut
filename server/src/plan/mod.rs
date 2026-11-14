//! The AI plan (v6.2implementation §7, D-458): `POST /plan`, its provider and
//! the one adapter that names DeepSeek (D-500), the reading of the model's
//! answer, the quotas and the spend cap. The only code that calls the AI
//! provider. Nothing here stores or logs a transcript or the model's text
//! (P-14).

use std::sync::Arc;

pub mod deepseek;
pub mod parse;
pub mod provider;
pub mod quota;
pub mod route;
pub mod spend;

/// What the route needs, built once at boot.
pub struct PlanState {
    /// `UnconfiguredPlanProvider` when `PLAN_ENABLED=false`.
    pub provider: Arc<dyn provider::PlanProvider>,
    /// The provider's id in a plan's `source` (`deepseek::PROVIDER`).
    pub provider_id: &'static str,
    pub cap: spend::SpendCap,
}
