//! Billing through the merchant of record: checkout, the portal, and its webhook.

pub mod adapter;
pub mod offers;
pub mod provider;
pub mod webhook;

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use offcut_api_types::{CheckoutRequest, UrlResponse};

use crate::auth::extract::AuthUser;
use crate::billing::offers::Offers;
use crate::db::{subscriptions, users};
use crate::error::{AppError, json_rejection};
use crate::rate_limit::{RateKey, RouteGroup, limit};
use crate::state::AppState;

/// `POST /billing/checkout`: where a person goes to pay. A person already paying
/// is refused, since a second subscription would go on billing with no row, and
/// their account's deletion could not stop it (v6implementation D-300).
pub async fn checkout(
    State(state): State<AppState>,
    auth: AuthUser,
    body: Result<Json<CheckoutRequest>, JsonRejection>,
) -> Result<Json<UrlResponse>, AppError> {
    limit(&state, RouteGroup::Checkout, RateKey::User(auth.user))?;
    let Json(request) = body.map_err(json_rejection)?;
    let email = users::email(&state.db, auth.user)
        .await?
        .ok_or(AppError::SessionExpired)?;
    if subscriptions::live_id(&state.db, auth.user)
        .await?
        .is_some()
    {
        return Err(AppError::Forbidden);
    }
    // A founding offer while it is off.
    Offers::of(&state.config)
        .product(request.offer)
        .ok_or(AppError::BadRequest)?;

    let origin = &state.config.app_origin;
    let url = state
        .billing
        .create_checkout_url(
            auth.user,
            &email,
            request.offer,
            &format!("{origin}/account?checkout=success"),
            &format!("{origin}/account?checkout=cancel"),
        )
        .await
        .map_err(|_| AppError::BillingUnavailable)?;
    Ok(Json(UrlResponse { url }))
}

/// `GET /billing/portal`: where a person manages what they pay for. 404 for one
/// who has never had a subscription.
pub async fn portal(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<UrlResponse>, AppError> {
    limit(&state, RouteGroup::Portal, RateKey::User(auth.user))?;
    let customer = subscriptions::customer_id(&state.db, auth.user)
        .await?
        .ok_or(AppError::NotFound)?;
    let url = state
        .billing
        .create_portal_url(&customer, &format!("{}/account", state.config.app_origin))
        .await
        .map_err(|_| AppError::BillingUnavailable)?;
    Ok(Json(UrlResponse { url }))
}
