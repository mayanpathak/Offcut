//! `GET /account/export` (TS §22.7): the server's records of a person. No token
//! hash and no merchant id: only what is about the person.

use axum::Json;
use axum::extract::State;
use offcut_api_types::AccountExport;
use serde::Serialize;

use crate::auth::extract::AuthUser;
use crate::db::ai_usage::ExportedAiUsage;
use crate::db::{ai_usage, sessions, subscriptions, usage_receipts, users};
use crate::error::AppError;
use crate::rate_limit::{RateKey, RouteGroup, limit};
use crate::state::AppState;

/// V6.2 (TS §52, S-10): the export with the person's AI rows beside V6's
/// fields. Each list is left out while empty, so an account that never used
/// the AI plan exports exactly what V6.1 exported; `AccountExport` is
/// unchanged.
#[derive(Serialize)]
pub struct Export {
    #[serde(flatten)]
    base: AccountExport,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ai_usage: Vec<ExportedAiUsage>,
    /// The months of the long-video grants (V6.4), never their hashes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ai_shorts_grant_months: Vec<String>,
}

pub async fn export_account(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Export>, AppError> {
    limit(&state, RouteGroup::AccountExport, RateKey::User(auth.user))?;
    let user = users::exported(&state.db, auth.user)
        .await?
        .ok_or(AppError::SessionExpired)?;
    Ok(Json(Export {
        base: AccountExport {
            user,
            sessions: sessions::exported(&state.db, auth.user).await?,
            subscription: subscriptions::summary(&state.db, auth.user).await?,
            usage_receipts: usage_receipts::exported(&state.db, auth.user).await?,
        },
        ai_usage: ai_usage::exported(&state.db, auth.user).await?,
        ai_shorts_grant_months: ai_usage::exported_grant_months(&state.db, auth.user).await?,
    }))
}
