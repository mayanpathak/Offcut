//! The entitlement token. `issue.rs` is the only place a plan and a free count
//! are worked out (TS §22.3).

pub mod issue;

use axum::Json;
use axum::extract::State;
use offcut_api_types::EntitlementResponse;
use time::OffsetDateTime;

use crate::auth::extract::AuthUser;
use crate::db::users;
use crate::error::AppError;
use crate::rate_limit::{RateKey, RouteGroup, limit};
use crate::state::AppState;

/// `GET /entitlement`: a fresh token. An account that is gone is a session that
/// is over (v6implementation D-262).
pub async fn get_entitlement(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<EntitlementResponse>, AppError> {
    limit(&state, RouteGroup::Entitlement, RateKey::User(auth.user))?;
    users::email(&state.db, auth.user)
        .await?
        .ok_or(AppError::SessionExpired)?;
    let issued = issue::issue(
        &state.db,
        &state.config.entitlement_signing_key,
        auth.user,
        OffsetDateTime::now_utc(),
    )
    .await?;
    Ok(Json(EntitlementResponse {
        entitlement_token: issued.token,
    }))
}
