//! `GET /account/export` (TS §22.7): the server's records of a person. No token
//! hash and no merchant id: only what is about the person.

use axum::Json;
use axum::extract::State;
use offcut_api_types::AccountExport;

use crate::auth::extract::AuthUser;
use crate::db::{sessions, subscriptions, usage_receipts, users};
use crate::error::AppError;
use crate::rate_limit::{RateKey, RouteGroup, limit};
use crate::state::AppState;

pub async fn export_account(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<AccountExport>, AppError> {
    limit(&state, RouteGroup::AccountExport, RateKey::User(auth.user))?;
    let user = users::exported(&state.db, auth.user)
        .await?
        .ok_or(AppError::SessionExpired)?;
    Ok(Json(AccountExport {
        user,
        sessions: sessions::exported(&state.db, auth.user).await?,
        subscription: subscriptions::summary(&state.db, auth.user).await?,
        usage_receipts: usage_receipts::exported(&state.db, auth.user).await?,
    }))
}
