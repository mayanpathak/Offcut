//! `POST /account/delete` (TS §22.7, PS §17): the subscription first, at the
//! merchant, then every row of the person in one transaction.

use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};

use crate::auth::extract::AuthUser;
use crate::auth::session::cleared_cookie;
use crate::db::{magic_links, platform_waitlist, subscriptions, users};
use crate::error::AppError;
use crate::rate_limit::{RateKey, RouteGroup, limit};
use crate::state::AppState;

/// 204 with the cookie cleared, also for an account already gone
/// (v6implementation D-271). A merchant that refuses is 502, with nothing
/// deleted, so a person is not left paying with no account to stop it from.
pub async fn delete_account(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<(StatusCode, HeaderMap), AppError> {
    limit(&state, RouteGroup::AccountDelete, RateKey::User(auth.user))?;

    if let Some(email) = users::email(&state.db, auth.user).await? {
        if let Some(subscription) = subscriptions::live_id(&state.db, auth.user).await? {
            state
                .billing
                .cancel_subscription(&subscription)
                .await
                .map_err(|_| AppError::BillingUnavailable)?;
        }
        let mut tx = state.db.begin().await?;
        // The tables of `USER_TABLES` go with the user, by their foreign keys.
        users::delete(&mut tx, auth.user).await?;
        magic_links::delete_for_email(&mut tx, &email).await?;
        platform_waitlist::delete_for_email(&mut tx, &email).await?;
        tx.commit().await?;
    }

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cleared_cookie());
    Ok((StatusCode::NO_CONTENT, headers))
}
