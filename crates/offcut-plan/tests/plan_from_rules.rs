//! The rules producer on the reference clip's five events
//! (`fixtures/speech/README.md`, "Expected events of V3"): five items with the
//! detector's ids, kinds, anchors and params, which validate with the
//! detector's spans; and no event gives a valid empty plan (section 7.8).
//!
//! The transcript is `fixtures/labeled/a01-scriptA.json`, the reference
//! clip's recognized text in written form, timed as the labeled tests time
//! it. A written file carries no prosody, so "brutal" (word 31) is given the
//! stress a measured recording gives it (README: the KeywordPop passes "only
//! with its loudness and pitch"), the labeled files' `*word*` value of 2.0.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use offcut_detect::{DetectorConfig, detect};
use offcut_plan::{CATALOG_VERSION, PlanConfig, from_rules, validate};
use offcut_text::{RawWord, normalize_transcript};
use offcut_types::{
    EditPlan, EventKind, EventParams, ItemParams, PlanSource, Prosody, Transcript, Unit,
    WordProsody,
};

/// `synthesize` of `offcut-detect/tests/precision_recall.rs` (v3implementation §17.5).
fn synthesize(sentences: &[String]) -> (Vec<RawWord>, Prosody) {
    let mut raw = Vec::new();
    let mut per_word = Vec::new();
    let mut at = 500u32;
    for text in sentences {
        for piece in text.split_whitespace() {
            let stressed = piece.starts_with('*')
                && piece.trim_end_matches(['.', ',', '?', '!']).ends_with('*');
            let word = piece.replace('*', "");
            let letters = word.chars().filter(|c| c.is_alphanumeric()).count() as u32;
            let length = (120 + 45 * letters).clamp(150, 700);
            raw.push(RawWord {
                text: word.clone(),
                start_ms: at,
                end_ms: at + length,
                confidence: 1.0,
            });
            let z = if stressed { 2.0 } else { 0.0 };
            per_word.push(WordProsody {
                energy_z: z,
                pitch_z: z,
            });
            at += length
                + if word.ends_with(['.', '?', '!']) {
                    400
                } else {
                    80
                };
        }
    }
    (raw, Prosody { per_word })
}

/// The reference transcript, with "brutal" stressed.
fn reference() -> (Transcript, Prosody) {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/labeled/a01-scriptA.json");
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("a01")).expect("a01 parses");
    let sentences: Vec<String> = serde_json::from_value(json["sentences"].clone()).unwrap();
    let (raw, mut p) = synthesize(&sentences);
    assert_eq!(raw[31].text, "brutal", "the word README names");
    p.per_word[31] = WordProsody {
        energy_z: 2.0,
        pitch_z: 2.0,
    };
    (normalize_transcript(raw, "labeled"), p)
}

#[test]
fn the_reference_clips_five_events_become_five_items() {
    let (t, p) = reference();
    let events = detect(&t, &p, &BTreeMap::new(), &DetectorConfig::default());

    // README's table, in time order: kind, first anchor word, and what it shows.
    let kinds: Vec<EventKind> = events.iter().map(|e| e.kind).collect();
    assert_eq!(
        kinds,
        [
            EventKind::NumberReveal,
            EventKind::KeywordPop,
            EventKind::FromTo,
            EventKind::ListReveal,
            EventKind::NumberReveal
        ]
    );
    let starts: Vec<u32> = events.iter().map(|e| e.anchors.start.get()).collect();
    assert_eq!(starts, [23, 31, 55, 75, 132]);
    match &events[0].params {
        EventParams::NumberReveal { value, label } => {
            assert_eq!((value.value, &value.unit), (3_000.0, &Unit::Hours));
            assert_eq!(label.as_deref(), Some("editing by hand"));
        }
        other @ (EventParams::ListReveal { .. }
        | EventParams::FromTo { .. }
        | EventParams::KeywordPop { .. }) => panic!("{other:?}"),
    }
    match &events[3].params {
        EventParams::ListReveal { count, header, .. } => {
            assert_eq!((*count, header.as_str()), (3, "3 REASONS"));
        }
        other @ (EventParams::NumberReveal { .. }
        | EventParams::FromTo { .. }
        | EventParams::KeywordPop { .. }) => panic!("{other:?}"),
    }
    match &events[4].params {
        EventParams::NumberReveal { value, label } => {
            assert_eq!((value.value, value.display.as_str()), (12_000.0, "$12k"));
            assert_eq!(label.as_deref(), Some("in sales"));
        }
        other @ (EventParams::ListReveal { .. }
        | EventParams::FromTo { .. }
        | EventParams::KeywordPop { .. }) => panic!("{other:?}"),
    }

    let plan = from_rules(&events, &t);
    assert_eq!(plan.source, PlanSource::Rules);
    assert_eq!(plan.catalog, CATALOG_VERSION);
    assert_eq!(plan.items.len(), 5);
    for (item, event) in plan.items.iter().zip(&events) {
        assert_eq!(
            (item.id, item.kind, item.anchors),
            (event.id, event.kind, event.anchors)
        );
        assert_eq!(item.params, ItemParams::Detected(event.params.clone()));
        assert_eq!(item.confidence, Some(event.confidence));
    }

    let (valid, dropped) =
        validate(plan, &t, &BTreeMap::new(), &PlanConfig::default()).expect("a rules plan");
    assert_eq!(dropped, []);
    assert_eq!(valid.items().len(), 5);
    for (item, event) in valid.items().iter().zip(&events) {
        assert_eq!((item.id, item.span), (event.id, event.span));
        assert_eq!(item.params, event.params);
    }
}

#[test]
fn no_event_gives_a_valid_empty_plan() {
    let (t, _) = reference();
    let plan = from_rules(&[], &t);
    assert_eq!(plan, EditPlan::empty_rules(CATALOG_VERSION));
    let (valid, dropped) = validate(plan, &t, &BTreeMap::new(), &PlanConfig::default()).unwrap();
    assert!(valid.is_empty());
    assert_eq!(dropped, []);
    assert_eq!(valid.source(), &PlanSource::Rules);
}
