//! [ONLY constructor of `ValidPlan`] Any plan in, a plan the scene can draw
//! without a further check out, with what was dropped and why (TS §40.4,
//! v4implementation §7.6, C.1, D-121, D-122). Only V-1 refuses a whole plan;
//! every other rule drops what breaks it. Nothing here changes the input: the
//! output is built.

use std::collections::BTreeMap;

// What `validate` leaves out, and why: in `offcut-types` so that the page's
// TypeScript is generated from them (v4changelog known issue 18).
use offcut_types::{
    Confidence, EditPlan, EventId, EventKind, EventParams, PLAN_FORMAT, PlanSource, Span, TimeMs,
    Transcript, WordIdx, WordRange,
};
pub use offcut_types::{DropReason, Dropped};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::catalog::{CAPTION_ZONE, CATALOG_VERSION, spec};
use crate::config::PlanConfig;
use crate::resolve::effective_tokens;
use crate::validate_params::check_item;

/// How long after its last word a visual of the event zone still holds it
/// (the visual's own hold and fade, TS §19.5): NumberReveal, then the others.
const NUMBER_HOLD_MS: u32 = 1_200;
const EVENT_HOLD_MS: u32 = 1_500;

/// A plan the scene may draw. It has no public constructor: `validate` is the
/// only way to make one (INV-21).
#[derive(Clone, Debug, PartialEq)]
pub struct ValidPlan {
    source: PlanSource,
    items: Vec<ValidItem>,
}

impl ValidPlan {
    pub fn source(&self) -> &PlanSource {
        &self.source
    }

    /// In the order the scene draws them: by start, then kind, then first word.
    pub fn items(&self) -> &[ValidItem] {
        &self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// An item with every number read from the words (V-5). `marks` are the
/// recording times its visual steps at: FromTo's second value, else none.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidItem {
    pub id: EventId,
    pub kind: EventKind,
    pub anchors: WordRange,
    pub span: Span,
    pub params: EventParams,
    pub marks: Vec<TimeMs>,
    pub confidence: Option<Confidence>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectReason {
    /// Not `PLAN_FORMAT`.
    Format,
    /// Not this build's `CATALOG_VERSION`.
    Catalog,
}

/// A plan this build cannot read at all (V-1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
#[error("the plan cannot be read by this build: {reason:?}")]
pub struct PlanRejected {
    pub reason: RejectReason,
}

/// Checks `plan` against the transcript as edited, item by item (TS §40.4).
pub fn validate(
    plan: EditPlan,
    t: &Transcript,
    edits: &BTreeMap<WordIdx, String>,
    cfg: &PlanConfig,
) -> Result<(ValidPlan, Vec<Dropped>), PlanRejected> {
    let EditPlan {
        format,
        catalog,
        source,
        cuts,
        items,
        chapters,
        extras,
        music,
    } = plan;

    // 1. V-1: the format and the catalog are this build's.
    if format != PLAN_FORMAT {
        return Err(PlanRejected {
            reason: RejectReason::Format,
        });
    }
    if catalog != CATALOG_VERSION {
        return Err(PlanRejected {
            reason: RejectReason::Catalog,
        });
    }

    // 2. V4 applies no cut, chapter, text extra or music (D-121, D-164).
    let not_here = cuts.len()
        + chapters.len()
        + usize::from(extras.hook.is_some())
        + usize::from(extras.post_caption.is_some())
        + extras.hashtags.len()
        + usize::from(music.is_some());
    let mut dropped = vec![
        Dropped {
            kind: None,
            reason: DropReason::NotInThisBuild,
        };
        not_here
    ];

    // 3. The words as the user sees them, once.
    let tokens = effective_tokens(t, edits);

    // 4. V-2 to V-5, each item on its own, in the plan's order.
    let mut checked = Vec::with_capacity(items.len());
    for item in items {
        let kind = item.kind;
        match check_item(item, &source, &tokens, t, cfg) {
            Ok(valid) => checked.push(valid),
            Err(reason) => dropped.push(Dropped {
                kind: Some(kind),
                reason,
            }),
        }
    }

    // 5. V-6: the limits of each kind and of the screen.
    let mut kept = limit(checked, cfg, &mut dropped);

    // 6. The order the scene draws in.
    kept.sort_by_key(|item| (item.span.start, item.kind, item.anchors.start));
    Ok((
        ValidPlan {
            source,
            items: kept,
        },
        dropped,
    ))
}

/// How long the zone of `kind` stays taken after the item's last word.
fn hold_ms(kind: EventKind) -> u32 {
    match kind {
        EventKind::NumberReveal => NUMBER_HOLD_MS,
        EventKind::ListReveal | EventKind::FromTo | EventKind::KeywordPop => EVENT_HOLD_MS,
    }
}

/// V-6, in anchor order (the earlier anchor wins, then the catalog order): at
/// most `max_per_clip` of a kind, `min_gap` between two of a kind, and one
/// visual at a time in a zone. The caption zone is shared by every
/// KeywordPop and never taken.
fn limit(
    mut items: Vec<ValidItem>,
    cfg: &PlanConfig,
    dropped: &mut Vec<Dropped>,
) -> Vec<ValidItem> {
    items.sort_by_key(|item| (item.anchors.start, item.kind));
    let mut count: BTreeMap<EventKind, u8> = BTreeMap::new();
    let mut last_end: BTreeMap<EventKind, TimeMs> = BTreeMap::new();
    // Zone, first instant, and the instant it is free again.
    let mut held: Vec<(&'static str, u32, u32)> = Vec::new();
    let mut kept = Vec::with_capacity(items.len());
    for item in items {
        let reason = match (spec(item.kind), cfg.per_kind.get(&item.kind)) {
            (Some(row), Some(limits)) => {
                let (start, end) = (item.span.start.get(), item.span.end.get());
                let until = end.saturating_add(hold_ms(item.kind));
                let so_far = count.get(&item.kind).copied().unwrap_or(0);
                let too_close = last_end
                    .get(&item.kind)
                    .is_some_and(|prev| start < prev.get().saturating_add(limits.min_gap.get()));
                let taken = row.zone != CAPTION_ZONE
                    && held.iter().any(|&(zone, from, free)| {
                        zone == row.zone && start < free && from < until
                    });
                if so_far >= limits.max_per_clip {
                    Some(DropReason::OverLimit)
                } else if too_close {
                    Some(DropReason::TooClose)
                } else if taken {
                    Some(DropReason::ZoneTaken)
                } else {
                    count.insert(item.kind, so_far.saturating_add(1));
                    last_end.insert(item.kind, item.span.end);
                    held.push((row.zone, start, until));
                    None
                }
            }
            (None, _) | (_, None) => Some(DropReason::UnknownKind),
        };
        match reason {
            Some(reason) => dropped.push(Dropped {
                kind: Some(item.kind),
                reason,
            }),
            None => kept.push(item),
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    use offcut_text::{RawWord, normalize_transcript};
    use offcut_types::{
        Chapter, Cut, DurMs, ItemParams, Label, ListPoint, MusicCue, MusicEnergy, PlanItem,
        Quantity, QuantityRef, Unit,
    };

    use super::*;

    /// 26 words, one every 700 ms, each 300 ms long.
    const WORDS: [&str; 26] = [
        "Last", "year", "we", "made", "$12,000", "in", "sales.", "Editing", "went", "from", "90",
        "minutes", "to", "10", "minutes.", "There", "are", "three", "reasons:", "first", "speed,",
        "second", "focus.", "It", "is", "brutal.",
    ];

    fn transcript() -> Transcript {
        let words = (0u32..)
            .zip(WORDS)
            .map(|(i, text)| RawWord {
                text: text.to_owned(),
                start_ms: 500 + i * 700,
                end_ms: 800 + i * 700,
                confidence: 1.0,
            })
            .collect();
        normalize_transcript(words, "test")
    }

    fn start(i: u32) -> TimeMs {
        TimeMs::new(500 + i * 700)
    }

    fn range(start: u32, end: u32) -> WordRange {
        WordRange {
            start: WordIdx::new(start),
            end: WordIdx::new(end),
        }
    }

    fn label(text: &str) -> Label {
        serde_json::from_value(serde_json::json!(text)).unwrap()
    }

    fn quantity(words: (u32, u32)) -> QuantityRef {
        QuantityRef {
            words: range(words.0, words.1),
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

    fn number(id: u64, anchors: (u32, u32), label_text: Option<&str>) -> PlanItem {
        let params = ItemParams::NumberReveal {
            value: quantity(anchors),
            label: label_text.map(label),
        };
        item(id, EventKind::NumberReveal, anchors, params)
    }

    fn from_to() -> PlanItem {
        let params = ItemParams::FromTo {
            from: quantity((10, 12)),
            to: quantity((13, 15)),
            label: None,
        };
        item(2, EventKind::FromTo, (9, 15), params)
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
        item(3, EventKind::ListReveal, (17, 23), params)
    }

    fn keyword(anchors: (u32, u32), word: u32) -> PlanItem {
        let params = ItemParams::KeywordPop {
            word: WordIdx::new(word),
        };
        item(4, EventKind::KeywordPop, anchors, params)
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

    fn run(plan: EditPlan) -> (ValidPlan, Vec<Dropped>) {
        validate(
            plan,
            &transcript(),
            &BTreeMap::new(),
            &PlanConfig::default(),
        )
        .unwrap()
    }

    /// The reasons the items of `plan` were dropped, in order.
    fn reasons(plan: EditPlan) -> Vec<DropReason> {
        run(plan).1.into_iter().map(|d| d.reason).collect()
    }

    fn the_four() -> Vec<PlanItem> {
        vec![
            keyword((25, 26), 25),
            list(Some("Reasons"), &[(19, "speed"), (21, "focus")]),
            from_to(),
            number(1, (4, 5), Some("in sales")),
        ]
    }

    #[test]
    fn v1_another_format_or_catalog_refuses_the_plan() {
        let t = transcript();
        let cfg = PlanConfig::default();
        let mut plan = EditPlan::empty_rules(CATALOG_VERSION);
        plan.format = 2;
        let refused = validate(plan, &t, &BTreeMap::new(), &cfg).unwrap_err();
        assert_eq!(refused.reason, RejectReason::Format);
        let refused = validate(EditPlan::empty_rules(2), &t, &BTreeMap::new(), &cfg).unwrap_err();
        assert_eq!(refused.reason, RejectReason::Catalog);
        let (valid, dropped) = run(EditPlan::empty_rules(CATALOG_VERSION));
        assert!(valid.is_empty() && dropped.is_empty());
        assert_eq!(valid.source(), &PlanSource::Rules);
    }

    #[test]
    fn a_model_plan_of_the_four_kinds_validates_whole_and_in_order() {
        let (valid, dropped) = run(model(the_four()));
        assert_eq!(dropped, []);
        let kinds: Vec<EventKind> = valid.items().iter().map(|i| i.kind).collect();
        assert_eq!(
            kinds,
            [
                EventKind::NumberReveal,
                EventKind::FromTo,
                EventKind::ListReveal,
                EventKind::KeywordPop
            ]
        );
        let [n, f, l, k] = valid.items() else {
            panic!("four items")
        };
        assert_eq!(n.span, Span::new(start(4), TimeMs::new(3_600)).unwrap());
        let EventParams::NumberReveal { value, label } = &n.params else {
            panic!("{:?}", n.params)
        };
        assert_eq!((value.value, &value.unit), (12_000.0, &Unit::Usd));
        assert_eq!(label.as_deref(), Some("in sales"));
        let EventParams::FromTo { from, to, .. } = &f.params else {
            panic!("{:?}", f.params)
        };
        assert_eq!((from.value, &from.unit), (90.0, &Unit::Minutes));
        assert_eq!((to.value, &to.unit), (10.0, &Unit::Minutes));
        // The time FromTo's second value is said at (§8.7).
        assert_eq!(f.marks, [start(13)]);
        let EventParams::ListReveal {
            count,
            header,
            items,
        } = &l.params
        else {
            panic!("{:?}", l.params)
        };
        assert_eq!((*count, header.as_str()), (2, "Reasons"));
        let points: Vec<(u8, &str, TimeMs)> = items
            .iter()
            .map(|i| (i.ordinal, i.text.as_str(), i.at))
            .collect();
        assert_eq!(points, [(1, "speed", start(19)), (2, "focus", start(21))]);
        assert_eq!(
            k.params,
            EventParams::KeywordPop {
                word: WordIdx::new(25)
            }
        );
        assert!(n.marks.is_empty() && l.marks.is_empty() && k.marks.is_empty());
    }

    #[test]
    fn an_edited_number_is_drawn_as_edited() {
        let edits = BTreeMap::from([(WordIdx::new(4), "$15,000".to_owned())]);
        let plan = model(vec![number(1, (4, 5), None)]);
        let (valid, _) = validate(plan, &transcript(), &edits, &PlanConfig::default()).unwrap();
        let params = &valid.items().first().unwrap().params;
        let EventParams::NumberReveal { value, .. } = params else {
            panic!("{params:?}")
        };
        assert_eq!(value.value, 15_000.0);
        // A number edited away is no number.
        let edits = BTreeMap::from([(WordIdx::new(4), "lots".to_owned())]);
        let plan = model(vec![number(1, (4, 5), None)]);
        let (_, dropped) = validate(plan, &transcript(), &edits, &PlanConfig::default()).unwrap();
        assert_eq!(
            dropped.first().unwrap().reason,
            DropReason::NumberUnresolved
        );
    }

    #[test]
    fn every_non_item_field_is_dropped_in_v4() {
        let mut plan = model(vec![number(1, (4, 5), None)]);
        plan.cuts = vec![
            Cut::Lead {
                keep: DurMs::new(200),
            },
            Cut::KeepGap {
                after: WordIdx::new(3),
            },
        ];
        plan.chapters = vec![Chapter {
            starts_at: WordIdx::new(0),
            title: label("Money"),
        }];
        plan.extras.hook = Some(label("I almost quit"));
        plan.extras.post_caption = Some("caption".to_owned());
        plan.extras.hashtags = vec![label("editing"), label("video")];
        plan.music = Some(MusicCue {
            fits: true,
            energy: MusicEnergy::Calm,
        });
        let (valid, dropped) = run(plan);
        assert_eq!(valid.items().len(), 1);
        assert_eq!(dropped.len(), 8);
        assert!(
            dropped
                .iter()
                .all(|d| d.kind.is_none() && d.reason == DropReason::NotInThisBuild)
        );
    }

    #[test]
    fn v2_producer_kind_and_params() {
        // A model may not send the detector's reading; the rules may not point at words.
        let detected = ItemParams::Detected(EventParams::KeywordPop {
            word: WordIdx::new(25),
        });
        let item_detected = item(9, EventKind::KeywordPop, (25, 26), detected);
        assert_eq!(
            reasons(model(vec![item_detected.clone()])),
            [DropReason::WrongProducer]
        );
        let mut rules = EditPlan::empty_rules(CATALOG_VERSION);
        rules.items = vec![keyword((25, 26), 25)];
        assert_eq!(reasons(rules.clone()), [DropReason::WrongProducer]);
        rules.items = vec![item_detected];
        assert_eq!(reasons(rules), []);
        // Params of another kind; a list with no point, or more than seven.
        let mismatched = PlanItem {
            kind: EventKind::NumberReveal,
            ..keyword((25, 26), 25)
        };
        assert_eq!(
            reasons(model(vec![mismatched])),
            [DropReason::ParamsMismatch]
        );
        assert_eq!(
            reasons(model(vec![list(None, &[])])),
            [DropReason::ParamsMismatch]
        );
        let eight: Vec<(u32, &str)> = (17..23).chain(17..19).map(|at| (at, "point")).collect();
        assert_eq!(
            reasons(model(vec![list(None, &eight)])),
            [DropReason::ParamsMismatch]
        );
    }

    #[test]
    fn v3_anchors_inside_the_transcript_and_the_rule() {
        assert_eq!(
            reasons(model(vec![number(1, (26, 27), None)])),
            [DropReason::AnchorsOutOfRange]
        );
        assert_eq!(
            reasons(model(vec![number(1, (4, 4), None)])),
            [DropReason::AnchorsOutOfRange]
        );
        // Seven words for a NumberReveal (1-6); two for a KeywordPop (one).
        let long = item(
            1,
            EventKind::NumberReveal,
            (0, 7),
            ItemParams::NumberReveal {
                value: quantity((4, 5)),
                label: None,
            },
        );
        assert_eq!(reasons(model(vec![long])), [DropReason::AnchorRule]);
        assert_eq!(
            reasons(model(vec![keyword((24, 26), 25)])),
            [DropReason::AnchorRule]
        );
        // A word pointed at outside the anchors breaks the anchor rule.
        assert_eq!(
            reasons(model(vec![keyword((24, 25), 25)])),
            [DropReason::AnchorRule]
        );
        assert_eq!(
            reasons(model(vec![list(None, &[(19, "a"), (24, "b")])])),
            [DropReason::AnchorRule]
        );
    }

    #[test]
    fn v4_labels_fit_and_say_no_new_digit() {
        let long = "x".repeat(33);
        assert_eq!(
            reasons(model(vec![number(1, (4, 5), Some(&long))])),
            [DropReason::LabelTooLong]
        );
        // 32 characters, not bytes: "é" is two bytes.
        let accented = "é".repeat(32);
        assert_eq!(reasons(model(vec![number(1, (4, 5), Some(&accented))])), []);
        let header = "h".repeat(25);
        assert_eq!(
            reasons(model(vec![list(Some(&header), &[(19, "a")])])),
            [DropReason::LabelTooLong]
        );
        assert_eq!(
            reasons(model(vec![number(1, (4, 5), Some("3x faster"))])),
            [DropReason::LabelDigits]
        );
        assert_eq!(
            reasons(model(vec![number(1, (4, 5), Some("12 thousand"))])),
            []
        );
    }

    #[test]
    fn v5_a_range_that_is_not_exactly_a_number_is_dropped() {
        let wide = item(
            1,
            EventKind::NumberReveal,
            (3, 5),
            ItemParams::NumberReveal {
                value: quantity((3, 5)),
                label: None,
            },
        );
        assert_eq!(reasons(model(vec![wide])), [DropReason::NumberUnresolved]);
    }

    #[test]
    fn a_detected_from_to_is_marked_at_its_second_quantity() {
        let q = |value: f64, display: &str| Quantity {
            value,
            unit: Unit::Minutes,
            display: display.to_owned(),
        };
        let params = |anchors| {
            item(
                5,
                EventKind::FromTo,
                anchors,
                ItemParams::Detected(EventParams::FromTo {
                    from: q(90.0, "90 minutes"),
                    to: q(10.0, "10 minutes"),
                    label: None,
                }),
            )
        };
        let mut rules = EditPlan::empty_rules(CATALOG_VERSION);
        rules.items = vec![params((9, 15))];
        let (valid, dropped) = run(rules.clone());
        assert_eq!(dropped, []);
        let only = valid.items().first().unwrap();
        assert_eq!(only.marks, [start(13)]);
        assert_eq!(only.span, Span::new(start(9), TimeMs::new(10_600)).unwrap());
        // Anchors that hold one quantity only.
        rules.items = vec![params((9, 12))];
        assert_eq!(reasons(rules), [DropReason::NumberUnresolved]);
    }

    #[test]
    fn v6_the_limits_of_a_kind_and_of_the_screen() {
        let two = || model(vec![number(1, (4, 5), None), number(6, (13, 15), None)]);
        let with = |change: &dyn Fn(&mut crate::config::KindLimits)| {
            let mut cfg = PlanConfig::default();
            change(cfg.per_kind.get_mut(&EventKind::NumberReveal).unwrap());
            let (_, dropped) = validate(two(), &transcript(), &BTreeMap::new(), &cfg).unwrap();
            dropped.into_iter().map(|d| d.reason).collect::<Vec<_>>()
        };
        assert_eq!(with(&|_| {}), []);
        assert_eq!(with(&|l| l.max_per_clip = 1), [DropReason::OverLimit]);
        assert_eq!(
            with(&|l| l.min_gap = DurMs::new(10_000)),
            [DropReason::TooClose]
        );
        // A number inside a FromTo's time: the earlier anchor holds the zone.
        // A KeywordPop at the same time is in the caption zone, never taken.
        let (valid, dropped) = run(model(vec![
            number(1, (10, 12), None),
            from_to(),
            keyword((11, 12), 11),
        ]));
        assert_eq!(
            dropped,
            [Dropped {
                kind: Some(EventKind::NumberReveal),
                reason: DropReason::ZoneTaken
            }]
        );
        assert_eq!(valid.items().len(), 2);
    }

    #[test]
    fn drop_reasons_are_snake_case() {
        let json = serde_json::to_string(&DropReason::NotInThisBuild).unwrap();
        assert_eq!(json, "\"not_in_this_build\"");
        assert_eq!(
            serde_json::to_string(&RejectReason::Catalog).unwrap(),
            "\"catalog\""
        );
    }
}
