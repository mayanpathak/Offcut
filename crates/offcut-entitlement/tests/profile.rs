//! Which file a token gives (INV-9): the size and the watermark come from the
//! verified claims, and from nothing else.

#![allow(clippy::unwrap_used)]

use offcut_entitlement::claims::EntitlementClaims;
use offcut_entitlement::export_profile;
use offcut_types::limits::{CREATOR_HEIGHT, CREATOR_WIDTH, FREE_HEIGHT, FREE_WIDTH};
use offcut_types::{ExportCount, Plan, ProfileKind, UnixSecs};

const NOW: i64 = 1_760_100_000;

fn claims(plan: Plan, period_end: i64, exp: i64) -> EntitlementClaims {
    EntitlementClaims {
        v: 1,
        sub: serde_json::from_str("\"0190f3a2-7b1c-7def-8a55-0123456789ab\"").unwrap(),
        plan,
        free_exports_remaining: ExportCount::new(3),
        period_end: UnixSecs::new(period_end),
        iat: UnixSecs::new(NOW - 10),
        exp: UnixSecs::new(exp),
    }
}

fn profile(claims: Option<&EntitlementClaims>) -> offcut_types::ExportProfile {
    export_profile(claims, UnixSecs::new(NOW))
}

fn assert_free(claims: &EntitlementClaims) {
    let free = profile(Some(claims));
    assert_eq!(free.kind, ProfileKind::Free);
    assert_eq!((free.width, free.height), (FREE_WIDTH, FREE_HEIGHT));
    assert!(free.watermark);
}

#[test]
fn no_claims_is_the_preview() {
    assert_eq!(profile(None).kind, ProfileKind::Preview);
}

#[test]
fn creator_in_its_period_is_1080p_with_no_watermark() {
    let creator = profile(Some(&claims(Plan::Creator, NOW + 100, NOW + 100)));
    assert_eq!(creator.kind, ProfileKind::Creator);
    assert_eq!(
        (creator.width, creator.height),
        (CREATOR_WIDTH, CREATOR_HEIGHT)
    );
    assert!(!creator.watermark);
}

#[test]
fn free_is_720p_with_a_watermark() {
    assert_free(&claims(Plan::Free, NOW + 100, NOW + 100));
}

#[test]
fn creator_past_its_period_or_its_token_is_free() {
    assert_free(&claims(Plan::Creator, NOW - 1, NOW + 100));
    assert_free(&claims(Plan::Creator, NOW + 100, NOW - 1));
}
