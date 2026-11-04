//! Signing a token: the server's half (feature `sign`). The browser's bundle
//! is built without it.

use ed25519_dalek::{Signer as _, SigningKey};

use crate::claims::EntitlementClaims;
use crate::token::{encode, signing_input};

/// The token of `claims`, signed with `key`. `verify_token` reads it with the
/// matching public key.
pub fn sign_token(claims: &EntitlementClaims, key: &SigningKey) -> String {
    let signature = key.sign(signing_input(claims).as_bytes());
    encode(claims, &signature.to_bytes())
}
