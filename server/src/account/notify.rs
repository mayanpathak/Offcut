//! `POST /notify-me`: the waitlist and the unsupported-platform email capture.

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use offcut_api_types::NotifyMeRequest;

use crate::auth::magic_link::normalize_email;
use crate::db::platform_waitlist;
use crate::error::{AppError, json_rejection};
use crate::state::AppState;

/// 204 whether or not the address was already on the list, so the route
/// cannot be used to find out whether it is.
pub async fn notify_me(
    State(state): State<AppState>,
    body: Result<Json<NotifyMeRequest>, JsonRejection>,
) -> Result<StatusCode, AppError> {
    let Json(request) = body.map_err(json_rejection)?;
    let email = normalize_email(&request.email).ok_or(AppError::BadRequest)?;
    platform_waitlist::upsert(&state.db, &email, request.wanted).await?;
    Ok(StatusCode::NO_CONTENT)
}
