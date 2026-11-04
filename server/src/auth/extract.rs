//! `AuthUser`: a handler that takes one runs only for a good access token. It
//! reads no database (v6implementation D-261), so a route that must know the
//! user still exists reads the user's row (D-262).

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use offcut_types::{UnixSecs, UserId};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::auth::access_token;
use crate::error::AppError;
use crate::log::user_tag;
use crate::state::AppState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthUser {
    pub user: UserId,
    pub session: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<AuthUser, AppError> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;
        let key = state.config.access_token_signing_key.verifying_key();
        let now = UnixSecs::new(OffsetDateTime::now_utc().unix_timestamp());
        let claims = access_token::verify(token, &key, now).map_err(|_| AppError::Unauthorized)?;
        // The request's log line says whose it was, hashed (TS §24.4).
        tracing::Span::current().record("user", user_tag(claims.sub));
        Ok(AuthUser {
            user: claims.sub,
            session: claims.sid,
        })
    }
}
