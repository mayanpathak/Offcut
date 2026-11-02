//! Sessions: the refresh token, its cookie, and what a refresh does (TS §22.2,
//! v6implementation §7.3).

use axum::http::header::{COOKIE, ORIGIN};
use axum::http::{HeaderMap, HeaderValue};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use offcut_types::UserId;
use sha2::{Digest as _, Sha256};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::db::sessions::{self, SessionRow};
use crate::error::AppError;

/// How long a session lasts with no refresh (TS §22.2, an assumption of M1.7).
pub const REFRESH_TOKEN_TTL_SECS: i64 = 60 * 24 * 3600;

const TOKEN_BYTES: usize = 32;
const COOKIE_NAME: &str = "offcut_rt";
/// How long a rotated token is still good: a lost answer's retry, or another tab.
const PREVIOUS_TOKEN_GRACE: Duration = Duration::seconds(30);
const CSRF_HEADER: &str = "x-offcut-csrf";

/// 32 random bytes in base64url: a sign-in link's token or a refresh token.
/// Only its hash is ever stored.
pub fn random_token() -> Result<String, AppError> {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes).map_err(|_| AppError::Internal)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub fn token_hash(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

/// The cookie that holds a refresh token (TS §22.2). A token is base64url, so
/// it always makes a valid header.
pub fn refresh_cookie(token: &str) -> Result<HeaderValue, AppError> {
    HeaderValue::try_from(format!(
        "offcut_rt={token}; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age={REFRESH_TOKEN_TTL_SECS}"
    ))
    .map_err(|_| AppError::Internal)
}

/// The header that removes the cookie.
pub fn cleared_cookie() -> HeaderValue {
    HeaderValue::from_static(
        "offcut_rt=; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=0",
    )
}

/// The refresh token a request carries, if it has one.
pub fn refresh_token_of(headers: &HeaderMap) -> Option<&str> {
    let cookies = headers.get_all(COOKIE).into_iter();
    cookies
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find_map(|(name, value)| (name == COOKIE_NAME && !value.is_empty()).then_some(value))
}

/// The two cookie routes want the header and their own origin (TS §22.2). A
/// page on another site can set neither.
pub fn check_csrf(headers: &HeaderMap, app_origin: &str) -> Result<(), AppError> {
    let said = |name| {
        headers
            .get(name)
            .and_then(|value: &HeaderValue| value.to_str().ok())
    };
    if said(CSRF_HEADER) == Some("1") && said(ORIGIN.as_str()) == Some(app_origin) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

/// What a refresh token gets, by §7.3's table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Decision {
    /// The current token: it becomes the previous one.
    RotateCurrent,
    /// The previous token in its grace: a retry after a lost answer, or another
    /// tab. It still gets a new cookie, as only that saves the browser.
    RotateInGrace,
    /// The previous token after its grace: someone holds an old token.
    Reuse,
    /// Nothing says which session this is, or the session is over.
    Refuse,
}

fn decide(row: Option<&SessionRow>, hash: &[u8], now: OffsetDateTime) -> Decision {
    let Some(row) = row else {
        return Decision::Refuse;
    };
    if row.revoked_at.is_some() || now > row.expires_at {
        return Decision::Refuse;
    }
    if row.refresh_token_hash == hash {
        return Decision::RotateCurrent;
    }
    if row.prev_token_hash.as_deref() != Some(hash) {
        return Decision::Refuse;
    }
    match row.prev_valid_until {
        Some(until) if now <= until => Decision::RotateInGrace,
        Some(_) | None => Decision::Reuse,
    }
}

/// What came of a refresh.
pub enum Refreshed {
    Rotated {
        token: String,
        user: UserId,
        session: Uuid,
    },
    /// The session was revoked. The cookie is cleared.
    Reused,
    /// The cookie stays as it is: the browser may hold a good one by the time
    /// this answer arrives.
    Refused,
}

/// Rows lock in turn, so one cookie's requests are decided one after the other.
/// The transaction is committed before the answer leaves: an answer that is
/// lost after this is what the grace is for.
pub async fn refresh(
    pool: &PgPool,
    presented: &str,
    now: OffsetDateTime,
) -> Result<Refreshed, AppError> {
    let hash = token_hash(presented);
    let mut tx = pool.begin().await?;
    let row = sessions::for_update(&mut tx, &hash).await?;
    let decision = decide(row.as_ref(), &hash, now);
    let Some(row) = row else {
        return Ok(Refreshed::Refused);
    };
    let token = random_token()?;
    let new_hash = token_hash(&token);
    match decision {
        Decision::RotateCurrent => {
            let until = now + PREVIOUS_TOKEN_GRACE;
            sessions::rotate_current(&mut tx, row.id, &new_hash, until).await?;
        }
        Decision::RotateInGrace => {
            sessions::rotate_in_grace(&mut tx, row.id, &new_hash).await?;
        }
        Decision::Reuse => {
            sessions::revoke(&mut tx, row.id).await?;
            tx.commit().await?;
            return Ok(Refreshed::Reused);
        }
        Decision::Refuse => return Ok(Refreshed::Refused),
    }
    tx.commit().await?;
    Ok(Refreshed::Rotated {
        token,
        user: UserId::new(row.user_id),
        session: row.id,
    })
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;

    const NOW: OffsetDateTime = datetime!(2026-11-02 12:00 UTC);

    fn row() -> SessionRow {
        SessionRow {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            refresh_token_hash: vec![1],
            prev_token_hash: Some(vec![2]),
            prev_valid_until: Some(NOW),
            expires_at: NOW + Duration::days(1),
            revoked_at: None,
        }
    }

    #[test]
    fn the_table_of_section_7_3() {
        assert_eq!(decide(Some(&row()), &[1], NOW), Decision::RotateCurrent);
        assert_eq!(decide(Some(&row()), &[2], NOW), Decision::RotateInGrace);
        let after = NOW + Duration::SECOND;
        assert_eq!(decide(Some(&row()), &[2], after), Decision::Reuse);
        assert_eq!(decide(Some(&row()), &[3], NOW), Decision::Refuse);
        assert_eq!(decide(None, &[1], NOW), Decision::Refuse);

        let revoked = SessionRow {
            revoked_at: Some(NOW),
            ..row()
        };
        assert_eq!(decide(Some(&revoked), &[1], NOW), Decision::Refuse);
        assert_eq!(decide(Some(&revoked), &[2], after), Decision::Refuse);
        let expired = SessionRow {
            expires_at: NOW - Duration::SECOND,
            ..row()
        };
        assert_eq!(decide(Some(&expired), &[1], NOW), Decision::Refuse);
    }

    #[test]
    fn a_first_token_has_no_previous() {
        let first = SessionRow {
            prev_token_hash: None,
            prev_valid_until: None,
            ..row()
        };
        assert_eq!(decide(Some(&first), &[1], NOW), Decision::RotateCurrent);
        assert_eq!(decide(Some(&first), &[2], NOW), Decision::Refuse);
    }

    #[test]
    fn the_refresh_token_is_found_among_the_cookies() {
        let of = |text: &'static str| {
            let mut headers = HeaderMap::new();
            headers.insert(COOKIE, HeaderValue::from_static(text));
            refresh_token_of(&headers).map(str::to_owned)
        };
        assert_eq!(of("offcut_rt=abc").as_deref(), Some("abc"));
        assert_eq!(of("a=1; offcut_rt=abc; b=2").as_deref(), Some("abc"));
        assert_eq!(of("not_offcut_rt=abc"), None);
        assert_eq!(of("offcut_rt="), None);
        assert_eq!(of("a=1"), None);
        assert_eq!(refresh_token_of(&HeaderMap::new()), None);
    }

    #[test]
    fn a_random_token_is_32_bytes_of_base64url_and_new_each_time() {
        let one = random_token().unwrap();
        assert_eq!(one.len(), 43);
        assert!(
            one.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        assert_ne!(one, random_token().unwrap());
    }

    #[test]
    fn the_cookie_has_the_attributes_of_the_spec() {
        assert_eq!(
            refresh_cookie("abc").unwrap().to_str().unwrap(),
            "offcut_rt=abc; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=5184000"
        );
    }

    #[test]
    fn a_hash_is_the_sha256_of_the_token() {
        // printf '%s' abc | sha256sum
        assert_eq!(token_hash("abc")[..4], [0xba, 0x78, 0x16, 0xbf]);
    }
}
