//! `validate`, rule by rule (TS §40.4, v4implementation §7.6, §7.8): one case
//! per V-rule and drop reason with the smallest plan that triggers it (21), a
//! model plan of the four word-pointing kinds that validates whole, the
//! edited number of INV-22, and the non-item fields V4 drops.
//!
//! The transcript is `fixtures/labeled/a01-scriptA.json` (the founder's
//! script), timed as the labeled tests time it. Words used: 23-24 "3000
//! hours", 31 "brutal", 54-59 "from 90 minutes to 10 minutes", 75-94 the
//! three reasons (points at 80, 85, 90), 132-133 "$12 ,000".

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use offcut_plan::{
    CATALOG_VERSION, DropReason, Dropped, PlanConfig, RejectReason, ValidPlan, validate,
};
use offcut_text::{RawWord, normalize_transcript};
use offcut_types::{
    Chapter, Cut, CutWhy, DurMs, EditPlan, EventId, EventKind, EventParams, ItemParams, Label,
    ListPoint, MusicCue, MusicEnergy, PlanItem, PlanSource, Quantity, QuantityRef, Transcript,
    Unit, WordIdx, WordRange,
};

// --- The transcript ------------------------------------------------------------

/// `synthesize` of `offcut-detect/tests/precision_recall.rs`, times only.
fn a01() -> Transcript {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/labeled/a01-scriptA.json");
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("a01")).expect("a01 parses");
    let sentences: Vec<String> = serde_json::from_value(json["sentences"].clone()).unwrap();
    let mut raw = Vec::new();
    let mut at = 500u32;
    for text in &sentences {
        for piece in text.split_whitespace() {
            let word = piece.replace('*', "");
            let letters = word.chars().filter(|c| c.is_alphanumeric()).count() as u32;
            let length = (120 + 45 * letters).clamp(150, 700);
            raw.push(RawWord {
                text: word.clone(),
                start_ms: at,
                end_ms: at + length,
                confidence: 1.0,
            });
            at += length
                + if word.ends_with(['.', '?', '!']) {
                    400
                } else {
                    80
                };
        }
    }
    normalize_transcript(raw, "labeled")
}

// --- Plans -----------------------------------------------------------------------

fn range(start: u32, end: u32) -> WordRange {
    WordRange {
        start: WordIdx::new(start),
        end: WordIdx::new(end),
    }
}

fn label(text: &str) -> Label {
    serde_json::from_value(serde_json::json!(text)).unwrap()
}

fn words(start: u32, end: u32) -> QuantityRef {
    QuantityRef {
        words: range(start, end),
    }
}

fn item(id: u64, kind: EventKind, anchors: (u32, u32), params: ItemParams) -> PlanItem {
    PlanItem {
        id: EventId::new(id),
        kind,
        anchors: range(anchors.0, anchors.1),
        params,
        confidence: None,
    }
}

/// "3000 hours", or another number at `at`.
fn number_at(id: u64, at: (u32, u32), label_text: Option<&str>) -> PlanItem {
    let params = ItemParams::NumberReveal {
        value: words(at.0, at.1),
        label: label_text.map(label),
    };
    item(id, EventKind::NumberReveal, at, params)
}

fn number(label_text: Option<&str>) -> PlanItem {
    number_at(1, (23, 25), label_text)
}

fn from_to() -> PlanItem {
    let params = ItemParams::FromTo {
        from: words(55, 57),
        to: words(58, 60),
        label: None,
    };
    item(2, EventKind::FromTo, (54, 60), params)
}

fn list(header: Option<&str>, points: &[(u32, &str)]) -> PlanItem {
    let items = points
        .iter()
        .map(|&(at, text)| ListPoint {
            at: WordIdx::new(at),
            label: label(text),
        })
        .collect();
    let params = ItemParams::ListReveal {
        header: header.map(label),
        items,
    };
    item(3, EventKind::ListReveal, (75, 95), params)
}

fn reasons_list() -> PlanItem {
    list(
        Some("Reasons it worked"),
        &[
            (80, "the captions write themselves"),
            (85, "numbers animate automatically"),
            (90, "everything runs on my laptop"),
        ],
    )
}

fn keyword(id: u64, at: u32) -> PlanItem {
    let params = ItemParams::KeywordPop {
        word: WordIdx::new(at),
    };
    item(id, EventKind::KeywordPop, (at, at + 1), params)
}

fn model(items: Vec<PlanItem>) -> EditPlan {
    EditPlan {
        source: PlanSource::Model {
            provider: "test".to_owned(),
            model: "test".to_owned(),
        },
        items,
        ..EditPlan::empty_rules(CATALOG_VERSION)
    }
}

fn rules(items: Vec<PlanItem>) -> EditPlan {
    EditPlan {
        items,
        ..EditPlan::empty_rules(CATALOG_VERSION)
    }
}

fn run_with(plan: EditPlan, cfg: &PlanConfig) -> (ValidPlan, Vec<Dropped>) {
    validate(plan, &a01(), &BTreeMap::new(), cfg).expect("V-1 passes")
}

fn run(plan: EditPlan) -> (ValidPlan, Vec<Dropped>) {
    run_with(plan, &PlanConfig::default())
}

/// The one item of `plan` is dropped, for `reason`.
fn drops_for(plan: EditPlan, reason: DropReason) {
    let kind = plan.items.first().map(|i| i.kind);
    let (valid, dropped) = run(plan);
    assert_eq!(dropped, [Dropped { kind, reason }]);
    assert!(valid.is_empty());
}

fn kept(plan: EditPlan) -> ValidPlan {
    let (valid, dropped) = run(plan);
    assert_eq!(dropped, []);
    valid
}

fn quantity(value: f64, unit: Unit, display: &str) -> Quantity {
    Quantity {
        value,
        unit,
        display: display.to_owned(),
    }
}

// --- The 21 cases ----------------------------------------------------------------

#[test]
fn case_01_v1_another_format_refuses_the_plan() {
    let mut plan = model(vec![number(None)]);
    plan.format = 2;
    let refused = validate(plan, &a01(), &BTreeMap::new(), &PlanConfig::default()).unwrap_err();
    assert_eq!(refused.reason, RejectReason::Format);
}

#[test]
fn case_02_v1_another_catalog_refuses_the_plan() {
    let mut plan = model(vec![number(None)]);
    plan.catalog = CATALOG_VERSION + 1;
    let refused = validate(plan, &a01(), &BTreeMap::new(), &PlanConfig::default()).unwrap_err();
    assert_eq!(refused.reason, RejectReason::Catalog);
}

#[test]
fn case_03_unknown_kind() {
    // Every V4 kind is in the catalog; a build whose limits know no
    // KeywordPop does not draw one.
    let mut cfg = PlanConfig::default();
    cfg.per_kind.remove(&EventKind::KeywordPop);
    let (_, dropped) = run_with(model(vec![keyword(4, 31)]), &cfg);
    assert_eq!(dropped[0].reason, DropReason::UnknownKind);
}

#[test]
fn case_04_wrong_producer_detected_in_a_model_plan() {
    let detected = ItemParams::Detected(EventParams::KeywordPop {
        word: WordIdx::new(31),
    });
    drops_for(
        model(vec![item(4, EventKind::KeywordPop, (31, 32), detected)]),
        DropReason::WrongProducer,
    );
}

#[test]
fn case_05_wrong_producer_word_pointer_in_a_rules_plan() {
    drops_for(rules(vec![keyword(4, 31)]), DropReason::WrongProducer);
}

#[test]
fn case_06_params_mismatch_arm_against_kind() {
    let wrong = PlanItem {
        kind: EventKind::KeywordPop,
        ..number(None)
    };
    drops_for(model(vec![wrong]), DropReason::ParamsMismatch);
}

#[test]
fn case_07_params_mismatch_required_param_missing() {
    // A list's points are required: none is no list.
    drops_for(
        model(vec![list(Some("Reasons"), &[])]),
        DropReason::ParamsMismatch,
    );
}

#[test]
fn case_08_anchors_out_of_range_empty() {
    drops_for(
        model(vec![number_at(1, (23, 23), None)]),
        DropReason::AnchorsOutOfRange,
    );
}

#[test]
fn case_09_anchors_out_of_range_past_the_end() {
    drops_for(model(vec![keyword(4, 157)]), DropReason::AnchorsOutOfRange);
}

#[test]
fn case_10_anchor_rule_word_count() {
    // A NumberReveal anchors 1-6 words; seven is too many.
    let params = ItemParams::NumberReveal {
        value: words(23, 25),
        label: None,
    };
    drops_for(
        model(vec![item(1, EventKind::NumberReveal, (20, 27), params)]),
        DropReason::AnchorRule,
    );
}

#[test]
fn case_11_anchor_rule_point_outside_the_anchors() {
    drops_for(
        model(vec![list(None, &[(80, "captions"), (96, "outside")])]),
        DropReason::AnchorRule,
    );
}

#[test]
fn case_12_label_too_long_in_characters() {
    drops_for(
        model(vec![number(Some(&"h".repeat(33)))]),
        DropReason::LabelTooLong,
    );
    // 32 characters of two bytes each: at the limit, kept.
    kept(model(vec![number(Some(&"é".repeat(32)))]));
}

#[test]
fn case_13_label_digits() {
    // "3x faster": no "3" was said ("3000" is another digit run).
    drops_for(
        model(vec![number(Some("3x faster"))]),
        DropReason::LabelDigits,
    );
}

#[test]
fn case_14_number_unresolved_range_not_exactly_a_number() {
    // "spent 3000 hours": a word before the number.
    let params = ItemParams::NumberReveal {
        value: words(22, 25),
        label: None,
    };
    drops_for(
        model(vec![item(1, EventKind::NumberReveal, (22, 25), params)]),
        DropReason::NumberUnresolved,
    );
}

#[test]
fn case_15_number_unresolved_from_to_without_a_second_quantity() {
    // A detector FromTo whose anchors hold "from 90 minutes to" only.
    let detected = ItemParams::Detected(EventParams::FromTo {
        from: quantity(90.0, Unit::Minutes, "90 minutes"),
        to: quantity(10.0, Unit::Minutes, "10 minutes"),
        label: None,
    });
    drops_for(
        rules(vec![item(2, EventKind::FromTo, (54, 58), detected)]),
        DropReason::NumberUnresolved,
    );
}

#[test]
fn case_16_over_limit() {
    let mut cfg = PlanConfig::default();
    cfg.per_kind
        .get_mut(&EventKind::NumberReveal)
        .unwrap()
        .max_per_clip = 1;
    let plan = model(vec![number(None), number_at(5, (132, 134), None)]);
    let (valid, dropped) = run_with(plan, &cfg);
    assert_eq!(valid.items().len(), 1);
    assert_eq!(
        dropped,
        [Dropped {
            kind: Some(EventKind::NumberReveal),
            reason: DropReason::OverLimit
        }]
    );
}

#[test]
fn case_17_too_close() {
    // Two KeywordPops 80 ms apart; their kind keeps 800 ms between two.
    let (valid, dropped) = run(model(vec![keyword(4, 31), keyword(5, 32)]));
    assert_eq!(valid.items().len(), 1);
    assert_eq!(
        dropped,
        [Dropped {
            kind: Some(EventKind::KeywordPop),
            reason: DropReason::TooClose
        }]
    );
}

#[test]
fn case_18_zone_taken() {
    // "90 minutes" as a number inside the FromTo's time: the FromTo's
    // earlier anchor holds the event zone.
    let (valid, dropped) = run(model(vec![number_at(1, (55, 57), None), from_to()]));
    assert_eq!(valid.items()[0].kind, EventKind::FromTo);
    assert_eq!(
        dropped,
        [Dropped {
            kind: Some(EventKind::NumberReveal),
            reason: DropReason::ZoneTaken
        }]
    );
}

#[test]
fn case_19_a_keyword_pop_is_never_zone_taken() {
    let valid = kept(model(vec![from_to(), keyword(4, 56)]));
    assert_eq!(valid.items().len(), 2);
}

#[test]
fn case_20_not_in_this_build_each_non_item_field() {
    let cuts = [
        Cut::Lead {
            keep: DurMs::new(200),
        },
        Cut::Trail {
            keep: DurMs::new(400),
        },
        Cut::Gap {
            after: WordIdx::new(14),
            keep: DurMs::new(150),
        },
        Cut::KeepGap {
            after: WordIdx::new(19),
        },
        Cut::Words {
            range: range(28, 29),
            why: CutWhy::FalseStart,
        },
    ];
    let one = |change: &dyn Fn(&mut EditPlan)| {
        let mut plan = model(Vec::new());
        change(&mut plan);
        let (_, dropped) = run(plan);
        assert_eq!(
            dropped,
            [Dropped {
                kind: None,
                reason: DropReason::NotInThisBuild
            }]
        );
    };
    for cut in cuts {
        one(&|plan| plan.cuts = vec![cut.clone()]);
    }
    one(&|plan| {
        plan.chapters = vec![Chapter {
            starts_at: WordIdx::new(0),
            title: label("Quitting"),
        }]
    });
    one(&|plan| plan.extras.hook = Some(label("I almost quit")));
    one(&|plan| plan.extras.post_caption = Some("How I got my evenings back".to_owned()));
    one(&|plan| plan.extras.hashtags = vec![label("editing")]);
    one(&|plan| {
        plan.music = Some(MusicCue {
            fits: true,
            energy: MusicEnergy::Upbeat,
        })
    });
}

#[test]
fn case_21_items_come_out_by_start_then_kind_then_word() {
    let valid = kept(model(vec![from_to(), keyword(4, 31), number(None)]));
    let order: Vec<(EventKind, u32)> = valid
        .items()
        .iter()
        .map(|i| (i.kind, i.anchors.start.get()))
        .collect();
    assert_eq!(
        order,
        [
            (EventKind::NumberReveal, 23),
            (EventKind::KeywordPop, 31),
            (EventKind::FromTo, 54)
        ]
    );
    let starts: Vec<u32> = valid.items().iter().map(|i| i.span.start.get()).collect();
    assert!(starts.windows(2).all(|w| w[0] <= w[1]));
}

// --- The whole plan, the edited number, the non-item fields ----------------------

fn the_four() -> EditPlan {
    model(vec![
        number(Some("editing by hand")),
        keyword(4, 31),
        from_to(),
        reasons_list(),
        number_at(5, (132, 134), Some("in sales")),
    ])
}

#[test]
fn a_model_plan_of_the_four_kinds_on_a01_validates_whole() {
    let t = a01();
    let valid = kept(the_four());
    let kinds: Vec<EventKind> = valid.items().iter().map(|i| i.kind).collect();
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
    let params: Vec<&EventParams> = valid.items().iter().map(|i| &i.params).collect();
    assert_eq!(
        params[0],
        &EventParams::NumberReveal {
            value: quantity(3_000.0, Unit::Hours, "3,000 hours"),
            label: Some("editing by hand".to_owned()),
        }
    );
    let EventParams::FromTo { from, to, .. } = params[2] else {
        panic!("{:?}", params[2])
    };
    assert_eq!(
        (from.value, &from.unit, to.value),
        (90.0, &Unit::Minutes, 10.0)
    );
    assert_eq!(valid.items()[2].marks, [t.words[58].start_ms]);
    let EventParams::ListReveal {
        count,
        header,
        items,
    } = params[3]
    else {
        panic!("{:?}", params[3])
    };
    assert_eq!((*count, header.as_str()), (3, "Reasons it worked"));
    let ats: Vec<_> = items.iter().map(|i| (i.ordinal, i.at)).collect();
    assert_eq!(
        ats,
        [
            (1, t.words[80].start_ms),
            (2, t.words[85].start_ms),
            (3, t.words[90].start_ms)
        ]
    );
    let EventParams::NumberReveal { value, .. } = params[4] else {
        panic!("{:?}", params[4])
    };
    assert_eq!((value.value, &value.unit), (12_000.0, &Unit::Usd));
    // Each item's span runs from its first anchor word to its last.
    for item in valid.items() {
        let first = &t.words[item.anchors.start.get() as usize];
        let last = &t.words[item.anchors.end.get() as usize - 1];
        assert_eq!(
            (item.span.start, item.span.end),
            (first.start_ms, last.end_ms)
        );
    }
}

#[test]
fn inv_22_an_edited_number_shows_the_edited_value() {
    let t = a01();
    let before = validate(the_four(), &t, &BTreeMap::new(), &PlanConfig::default()).unwrap();
    let edits = BTreeMap::from([(WordIdx::new(23), "4000".to_owned())]);
    let (after, dropped) = validate(the_four(), &t, &edits, &PlanConfig::default()).unwrap();
    assert_eq!(dropped, []);
    let first = |plan: &ValidPlan| plan.items()[0].params.clone();
    let EventParams::NumberReveal { value, .. } = first(&before.0) else {
        panic!("a number")
    };
    assert_eq!(value.value, 3_000.0);
    let EventParams::NumberReveal { value, .. } = first(&after) else {
        panic!("a number")
    };
    assert_eq!(
        (value.value, value.display.as_str()),
        (4_000.0, "4,000 hours")
    );
}

#[test]
fn the_non_item_fields_are_dropped_and_the_items_kept() {
    let mut plan = the_four();
    plan.cuts = vec![Cut::Lead {
        keep: DurMs::new(200),
    }];
    plan.chapters = vec![Chapter {
        starts_at: WordIdx::new(0),
        title: label("Quitting"),
    }];
    plan.extras.hook = Some(label("I almost quit"));
    plan.extras.post_caption = Some("caption".to_owned());
    plan.extras.hashtags = vec![label("editing")];
    plan.music = Some(MusicCue {
        fits: false,
        energy: MusicEnergy::Calm,
    });
    let (valid, dropped) = run(plan);
    assert_eq!(valid.items().len(), 5);
    assert_eq!(dropped.len(), 6);
    assert!(
        dropped
            .iter()
            .all(|d| d.kind.is_none() && d.reason == DropReason::NotInThisBuild)
    );
}
