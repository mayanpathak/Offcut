//! What the server signs, the browser verifies (TS §10.8, §22.3). The test
//! needs the feature `sign`: `cargo test --workspace` turns it on through the server.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::SigningKey;
use offcut_entitlement::claims::EntitlementClaims;
use offcut_entitlement::sign::sign_token;
use offcut_entitlement::{TokenError, verify_token};
use offcut_types::{ExportCount, Plan, UnixSecs};

fn key() -> SigningKey {
    SigningKey::from_bytes(&[7; 32])
}

fn public() -> [u8; 32] {
    key().verifying_key().to_bytes()
}

fn claims(plan: Plan, remaining: u32) -> EntitlementClaims {
    EntitlementClaims {
        v: 1,
        sub: serde_json::from_str("\"0190f3a2-7b1c-7def-8a55-0123456789ab\"").unwrap(),
        plan,
        free_exports_remaining: ExportCount::new(remaining),
        period_end: UnixSecs::new(1_762_592_000),
        iat: UnixSecs::new(1_760_000_000),
        exp: UnixSecs::new(1_760_604_800),
    }
}

#[test]
fn a_signed_token_verifies_and_gives_back_its_claims() {
    for claims in [claims(Plan::Free, 2), claims(Plan::Creator, 0)] {
        let token = sign_token(&claims, &key());
        assert_eq!(verify_token(&token, &[public()]).unwrap(), claims);
    }
}

#[test]
fn another_key_does_not_verify_it() {
    let token = sign_token(&claims(Plan::Creator, 0), &key());
    let other = SigningKey::from_bytes(&[8; 32]).verifying_key().to_bytes();
    assert_eq!(verify_token(&token, &[other]), Err(TokenError::Signature));
    // A rotation ships the new key beside the old one (TS §24.2).
    assert!(verify_token(&token, &[other, public()]).is_ok());
}

#[test]
fn a_change_to_either_segment_is_refused() {
    let token = sign_token(&claims(Plan::Free, 3), &key());
    let (payload, signature) = token.split_once('.').unwrap();

    // The same claims but for the plan, under the old signature.
    let json = URL_SAFE_NO_PAD.decode(payload).unwrap();
    let raised = String::from_utf8(json)
        .unwrap()
        .replace("\"free\"", "\"creator\"");
    let forged = format!("{}.{signature}", URL_SAFE_NO_PAD.encode(raised));
    assert_eq!(
        verify_token(&forged, &[public()]),
        Err(TokenError::Signature)
    );

    let mut bytes = URL_SAFE_NO_PAD.decode(signature).unwrap();
    bytes[0] ^= 1;
    let forged = format!("{payload}.{}", URL_SAFE_NO_PAD.encode(bytes));
    assert_eq!(
        verify_token(&forged, &[public()]),
        Err(TokenError::Signature)
    );
}
