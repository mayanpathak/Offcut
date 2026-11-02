//! Sessions: the refresh token and its cookie (TS §22.2). Rotation arrives with
//! `POST /auth/refresh` (Prompt 140).

use axum::http::HeaderValue;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest as _, Sha256};

use crate::error::AppError;

/// How long a session lasts with no refresh (TS §22.2, an assumption of M1.7).
pub const REFRESH_TOKEN_TTL_SECS: i64 = 60 * 24 * 3600;

const TOKEN_BYTES: usize = 32;

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

#[cfg(test)]
mod tests {
    use super::*;

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
