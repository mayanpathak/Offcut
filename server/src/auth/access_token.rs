//! The access token: `{sub, sid, exp}` in the entitlement token's wire form,
//! signed with its own key (TS §22.2, v6implementation D-261). It lives in the
//! client's memory for 15 minutes and is never stored.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, Signer as _, SigningKey, VerifyingKey};
use offcut_types::{UnixSecs, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// How long an access token works (TS §22.2, an assumption of M1.7).
pub const ACCESS_TOKEN_TTL_SECS: i64 = 15 * 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessClaims {
    pub sub: UserId,
    pub sid: Uuid,
    pub exp: UnixSecs,
}

/// Why a token is refused. No value holds a part of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessTokenError {
    Malformed,
    Signature,
    Expired,
}

pub fn sign(claims: &AccessClaims, key: &SigningKey) -> String {
    // Two UUIDs and a number: writing them cannot fail.
    let json = serde_json::to_vec(claims).unwrap_or_default();
    let payload = URL_SAFE_NO_PAD.encode(json);
    let signature = key.sign(payload.as_bytes());
    format!("{payload}.{}", URL_SAFE_NO_PAD.encode(signature.to_bytes()))
}

/// The claims of a token signed with this key that has not run out at `now`.
pub fn verify(
    token: &str,
    key: &VerifyingKey,
    now: UnixSecs,
) -> Result<AccessClaims, AccessTokenError> {
    let (payload, signature) = token.split_once('.').ok_or(AccessTokenError::Malformed)?;
    let signature: [u8; 64] = URL_SAFE_NO_PAD
        .decode(signature)
        .map_err(|_| AccessTokenError::Malformed)?
        .try_into()
        .map_err(|_| AccessTokenError::Malformed)?;
    key.verify_strict(payload.as_bytes(), &Signature::from_bytes(&signature))
        .map_err(|_| AccessTokenError::Signature)?;
    let json = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| AccessTokenError::Malformed)?;
    let claims: AccessClaims =
        serde_json::from_slice(&json).map_err(|_| AccessTokenError::Malformed)?;
    if now > claims.exp {
        return Err(AccessTokenError::Expired);
    }
    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_760_000_000;

    fn key() -> SigningKey {
        SigningKey::from_bytes(&[3; 32])
    }

    fn claims(exp: i64) -> AccessClaims {
        AccessClaims {
            sub: serde_json::from_str("\"0190f3a2-7b1c-7def-8a55-0123456789ab\"").unwrap(),
            sid: Uuid::from_u128(5),
            exp: UnixSecs::new(exp),
        }
    }

    fn check(token: &str) -> Result<AccessClaims, AccessTokenError> {
        verify(token, &key().verifying_key(), UnixSecs::new(NOW))
    }

    #[test]
    fn a_token_verifies_until_it_runs_out() {
        let good = claims(NOW);
        assert_eq!(check(&sign(&good, &key())), Ok(good));
        let late = sign(&claims(NOW - 1), &key());
        assert_eq!(check(&late), Err(AccessTokenError::Expired));
    }

    #[test]
    fn another_key_or_a_changed_token_is_refused() {
        let other = SigningKey::from_bytes(&[4; 32]);
        let token = sign(&claims(NOW + 1), &other);
        assert_eq!(check(&token), Err(AccessTokenError::Signature));

        let (payload, signature) = sign(&claims(NOW + 1), &key())
            .split_once('.')
            .map(|(p, s)| (p.to_owned(), s.to_owned()))
            .unwrap();
        let later = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims(NOW + 9_999)).unwrap());
        assert_ne!(later, payload);
        let forged = format!("{later}.{signature}");
        assert_eq!(check(&forged), Err(AccessTokenError::Signature));
    }

    #[test]
    fn what_is_not_a_token_is_malformed() {
        for token in ["", "no-dot", "a.b", "a.!!!"] {
            assert_eq!(check(token), Err(AccessTokenError::Malformed), "{token}");
        }
    }
}
