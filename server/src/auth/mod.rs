//! Sign-in by magic link (TS §22.1, §22.2; v6implementation §7).

pub mod access_token;
pub mod extract;
pub mod magic_link;
pub mod session;

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use offcut_api_types::{MagicLinkRequest, MeResponse, SessionResponse, VerifyRequest};
use offcut_types::{UnixSecs, UserId};
use sha2::{Digest as _, Sha256};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::auth::access_token::{ACCESS_TOKEN_TTL_SECS, AccessClaims};
use crate::auth::extract::AuthUser;
use crate::auth::magic_link::{MAGIC_LINK_TTL_MINUTES, normalize_email};
use crate::auth::session::{
    REFRESH_TOKEN_TTL_SECS, Refreshed, check_csrf, cleared_cookie, random_token, refresh_cookie,
    refresh_token_of, token_hash,
};
use crate::db::{magic_links, sessions, subscriptions, users};
use crate::entitlement::issue::issue;
use crate::error::{AppError, json_rejection};
use crate::rate_limit::{RateKey, RouteGroup, limit};
use crate::state::AppState;

/// `POST /auth/magic-link`. 202 whether or not the address has an account:
/// users are made only at verify (v6implementation §7.1).
pub async fn request_magic_link(
    State(state): State<AppState>,
    body: Result<Json<MagicLinkRequest>, JsonRejection>,
) -> Result<StatusCode, AppError> {
    let Json(request) = body.map_err(json_rejection)?;
    let email = normalize_email(&request.email).ok_or(AppError::BadRequest)?;
    let email_hash = Sha256::digest(email.as_bytes()).into();
    limit(
        &state,
        RouteGroup::MagicLinkEmail,
        RateKey::EmailHash(email_hash),
    )?;

    let token = random_token()?;
    let hash = token_hash(&token);
    let expires_at = OffsetDateTime::now_utc() + Duration::minutes(MAGIC_LINK_TTL_MINUTES);
    magic_links::replace(&state.db, &hash, &email, expires_at).await?;

    let link = format!("{}/auth/callback?token={token}", state.config.app_origin);
    if state.mailer.send_magic_link(&email, &link).await.is_err() {
        magic_links::delete(&state.db, &hash).await?;
        return Err(AppError::EmailUnavailable);
    }
    Ok(StatusCode::ACCEPTED)
}

/// `POST /auth/verify`: one link makes one session. The row stays locked until
/// the commit, so two requests with one link give one success
/// (v6implementation §7.2).
pub async fn verify(
    State(state): State<AppState>,
    body: Result<Json<VerifyRequest>, JsonRejection>,
) -> Result<(StatusCode, HeaderMap, Json<SessionResponse>), AppError> {
    let Json(request) = body.map_err(json_rejection)?;
    let hash = token_hash(&request.token);
    let now = OffsetDateTime::now_utc();

    let mut tx = state.db.begin().await?;
    let link = magic_links::for_update(&mut tx, &hash)
        .await?
        .filter(|link| link.consumed_at.is_none())
        .ok_or(AppError::LinkInvalid)?;
    if now > link.expires_at {
        return Err(AppError::LinkExpired);
    }
    magic_links::consume(&mut tx, &hash).await?;

    // The address as normalized: the link holds no other (v6implementation D-260).
    let user: UserId =
        users::find_or_create(&mut tx, Uuid::now_v7(), &link.email_normalized).await?;
    let session = Uuid::now_v7();
    let refresh = random_token()?;
    let session_end = now + Duration::seconds(REFRESH_TOKEN_TTL_SECS);
    sessions::create(&mut tx, session, user, &token_hash(&refresh), session_end).await?;
    tx.commit().await?;

    let response = session_response(&state, user, session, now).await?;
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, refresh_cookie(&refresh)?);
    Ok((StatusCode::OK, headers, Json(response)))
}

/// `POST /auth/refresh` (v6implementation §7.3).
pub async fn refresh(
    State(state): State<AppState>,
    request_headers: HeaderMap,
) -> Result<Response, AppError> {
    check_csrf(&request_headers, &state.config.app_origin)?;
    let presented = refresh_token_of(&request_headers).ok_or(AppError::SessionExpired)?;
    let now = OffsetDateTime::now_utc();
    match session::refresh(&state.db, presented, now).await? {
        Refreshed::Rotated {
            token,
            user,
            session,
        } => {
            let response = session_response(&state, user, session, now).await?;
            let mut headers = HeaderMap::new();
            headers.insert(SET_COOKIE, refresh_cookie(&token)?);
            Ok((StatusCode::OK, headers, Json(response)).into_response())
        }
        Refreshed::Reused => {
            let mut headers = HeaderMap::new();
            headers.insert(SET_COOKIE, cleared_cookie());
            Ok((headers, AppError::SessionExpired).into_response())
        }
        Refreshed::Refused => Err(AppError::SessionExpired),
    }
}

/// `POST /auth/logout`. 204 with the cookie cleared, whatever the token was.
pub async fn logout(
    State(state): State<AppState>,
    request_headers: HeaderMap,
) -> Result<(StatusCode, HeaderMap), AppError> {
    check_csrf(&request_headers, &state.config.app_origin)?;
    if let Some(presented) = refresh_token_of(&request_headers) {
        sessions::revoke_by_token(&state.db, &token_hash(presented)).await?;
    }
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cleared_cookie());
    Ok((StatusCode::NO_CONTENT, headers))
}

/// `GET /me`. An account that is gone is a session that is over (D-262).
pub async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<MeResponse>, AppError> {
    limit(&state, RouteGroup::Me, RateKey::User(auth.user))?;
    let email = users::email(&state.db, auth.user)
        .await?
        .ok_or(AppError::SessionExpired)?;
    let subscription = subscriptions::summary(&state.db, auth.user).await?;
    let issued = issue(
        &state.db,
        &state.config.entitlement_signing_key,
        auth.user,
        OffsetDateTime::now_utc(),
    )
    .await?;
    Ok(Json(MeResponse {
        email,
        plan: issued.plan,
        subscription,
        entitlement_token: issued.token,
    }))
}

/// What a sign-in and a refresh both answer.
async fn session_response(
    state: &AppState,
    user: UserId,
    session: Uuid,
    now: OffsetDateTime,
) -> Result<SessionResponse, AppError> {
    let exp = UnixSecs::new(now.unix_timestamp() + ACCESS_TOKEN_TTL_SECS);
    let claims = AccessClaims {
        sub: user,
        sid: session,
        exp,
    };
    let issued = issue(&state.db, &state.config.entitlement_signing_key, user, now).await?;
    Ok(SessionResponse {
        access_token: access_token::sign(&claims, &state.config.access_token_signing_key),
        access_expires_at: exp,
        entitlement_token: issued.token,
    })
}
