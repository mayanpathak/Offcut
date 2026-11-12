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
use offcut_types::{EmphasisSize, Label, TextExtras};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::catalog::{CAPTION_ZONE, CATALOG_VERSION, spec};
use crate::config::PlanConfig;
use crate::resolve::effective_tokens;
use crate::validate_params::check_item;

/// How long after its last word a visual of the event zone still holds it
/// (the visual's own hold and fade, TS §19.5): NumberReveal, then the others.
const NUMBER_HOLD_MS: u32 = 1_200;
pub(crate) const EVENT_HOLD_MS: u32 = 1_500;

/// A plan the scene may draw. It has no public constructor: `validate` is the
/// only way to make one (INV-21).
#[derive(Clone, Debug, PartialEq)]
pub struct ValidPlan {
    source: PlanSource,
    items: Vec<ValidItem>,
    extras: ValidExtras,
}

/// The text extras that passed D-454, for the panel and the post (V6.2).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ValidExtras {
    pub hook: Option<String>,
    pub post_caption: Option<String>,
    pub hashtags: Vec<String>,
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

    /// The extras that passed (D-454); empty for a plan of the rules.
    pub fn extras(&self) -> &ValidExtras {
        &self.extras
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
    /// Newer than this build's `CATALOG_VERSION` (DC-7, D-449: an older one is read).
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

    // 1. V-1: the format is this build's, and the catalog no newer (DC-7, D-449).
    if format != PLAN_FORMAT {
        return Err(PlanRejected {
            reason: RejectReason::Format,
        });
    }
    if catalog > CATALOG_VERSION {
        return Err(PlanRejected {
            reason: RejectReason::Catalog,
        });
    }

    // 2. No cut, chapter or music in this build (D-121, D-164, D-455); the
    // text extras are checked in 3, after the words.
    let not_here = cuts.len() + chapters.len() + usize::from(music.is_some());
    let mut dropped = vec![
        Dropped {
            kind: None,
            reason: DropReason::NotInThisBuild,
        };
        not_here
    ];

    // 3. The words as the user sees them, once.
    let tokens = effective_tokens(t, edits);
    let extras = check_extras(extras, &tokens, &mut dropped);

    // 4. V-2 to V-5, each item on its own, in the plan's order.
    let mut checked = Vec::with_capacity(items.len());
    for item in items {
        let kind = item.kind;
        match check_item(item, &source, &tokens, t, cfg, extras.hook.as_deref()) {
            Ok(valid) => checked.push(valid),
            Err(reason) => dropped.push(Dropped {
                kind: Some(kind),
                reason,
            }),
        }
    }

    // 5. V-6: the limits of each kind and of the screen.
    let mut kept = limit(checked, cfg, &mut dropped);
    // V6.2: the zoom rules (D-450), the punch words (D-452), then the density
    // of a model's visuals (D-451); a plan of the rules keeps its frames.
    kept = zoom_rule(kept, t, &mut dropped);
    kept = punch_rule(kept, &mut dropped);
    if matches!(source, PlanSource::Model { .. }) {
        kept = density_rule(kept, &mut dropped);
    }

    // 6. The order the scene draws in.
    kept.sort_by_key(|item| (item.span.start, item.kind, item.anchors.start));
    Ok((
        ValidPlan {
            source,
            items: kept,
            extras,
        },
        dropped,
    ))
}

/// How long the zone of `kind` stays taken after the item's last word.
fn hold_ms(kind: EventKind) -> u32 {
    match kind {
        EventKind::NumberReveal => NUMBER_HOLD_MS,
        EventKind::ListReveal | EventKind::FromTo | EventKind::KeywordPop => EVENT_HOLD_MS,
        // V6.2: an icon holds the event zone as the others do; the caption marks,
        // the camera and the hook take a zone nothing else shares, or none.
        EventKind::IconPop => EVENT_HOLD_MS,
        EventKind::Emphasis | EventKind::PunchIn | EventKind::Emoji | EventKind::HookTitle => 0,
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

// --- V6.2: the text extras and the rules after V-6 (all numbers assumptions, E-11).

/// The hook: at most this many characters, and 2 to 9 words (D-454).
pub(crate) const HOOK_MAX_CHARS: usize = 48;
pub(crate) const HOOK_WORDS: std::ops::RangeInclusive<usize> = 2..=9;
/// The shortest platform's limit for a post's caption (assumption, D-454).
pub(crate) const POST_CAPTION_MAX_CHARS: usize = 2_200;
/// At most this many hashtags, each `#` and 1 to 30 letters, digits or `_`.
pub(crate) const MAX_HASHTAGS: usize = 8;
pub(crate) const HASHTAG_MAX_CHARS: usize = 30;
/// D-452: one punch word per 6 s at most (beside the kind's own gap), 4 a clip.
pub(crate) const PUNCH_GAP_MS: u32 = 6_000;
pub(crate) const MAX_PUNCH: usize = 4;
/// D-451: at most one visual starts in any window this long (model plans).
pub(crate) const DENSITY_WINDOW_MS: u32 = 2_500;

/// D-454: each extra is checked alone, and a bad one is dropped alone, with
/// `kind: None`. The hook and the hashtags say no digit the speaker did not
/// (V-4); the post caption is the person's to edit, so it is not checked.
fn check_extras(
    extras: TextExtras,
    tokens: &[offcut_text::Token],
    dropped: &mut Vec<Dropped>,
) -> ValidExtras {
    let mut drop = |reason| dropped.push(Dropped { kind: None, reason });
    let hook = extras
        .hook
        .and_then(|hook| match hook_reason(&hook, tokens) {
            None => Some(hook.as_str().to_owned()),
            Some(reason) => {
                drop(reason);
                None
            }
        });
    let post_caption = extras.post_caption.and_then(|text| {
        if text.chars().count() > POST_CAPTION_MAX_CHARS {
            drop(DropReason::LabelTooLong);
            None
        } else {
            Some(text)
        }
    });
    let mut hashtags: Vec<String> = Vec::new();
    for tag in extras.hashtags {
        let reason = if hashtags.len() >= MAX_HASHTAGS {
            Some(DropReason::OverLimit)
        } else {
            hashtag_reason(&tag, tokens)
        };
        match reason {
            None => hashtags.push(tag.as_str().to_owned()),
            Some(reason) => drop(reason),
        }
    }
    ValidExtras {
        hook,
        post_caption,
        hashtags,
    }
}

fn hook_reason(hook: &Label, tokens: &[offcut_text::Token]) -> Option<DropReason> {
    let text = hook.as_str();
    let words = text.split_whitespace().count();
    if text.chars().count() > HOOK_MAX_CHARS || !HOOK_WORDS.contains(&words) {
        Some(DropReason::LabelTooLong)
    } else if !crate::resolve::label_digits_ok(text, tokens) {
        Some(DropReason::LabelDigits)
    } else {
        None
    }
}

/// `#` then 1 to 30 letters, digits or `_`: not that shape is
/// `ParamsMismatch`, too long `LabelTooLong`, a new digit `LabelDigits`.
fn hashtag_reason(tag: &Label, tokens: &[offcut_text::Token]) -> Option<DropReason> {
    let Some(body) = tag.as_str().strip_prefix('#') else {
        return Some(DropReason::ParamsMismatch);
    };
    let shaped = !body.is_empty() && body.chars().all(|c| c.is_alphanumeric() || c == '_');
    if !shaped {
        Some(DropReason::ParamsMismatch)
    } else if body.chars().count() > HASHTAG_MAX_CHARS {
        Some(DropReason::LabelTooLong)
    } else if !crate::resolve::label_digits_ok(body, tokens) {
        Some(DropReason::LabelDigits)
    } else {
        None
    }
}

/// Drops each item `rule` refuses, in the order given (anchor order), into
/// `dropped`; `rule` sees the items kept so far.
fn keep_where(
    items: Vec<ValidItem>,
    dropped: &mut Vec<Dropped>,
    mut rule: impl FnMut(&ValidItem, &[ValidItem]) -> Option<DropReason>,
) -> Vec<ValidItem> {
    let mut kept: Vec<ValidItem> = Vec::with_capacity(items.len());
    for item in items {
        match rule(&item, &kept) {
            Some(reason) => dropped.push(Dropped {
                kind: Some(item.kind),
                reason,
            }),
            None => kept.push(item),
        }
    }
    kept
}

/// The sentence a word is in, by index.
fn sentence_of(t: &Transcript, word: WordIdx) -> Option<usize> {
    t.sentences
        .iter()
        .position(|s| s.words.start <= word && word < s.words.end)
}

/// V-3's zoom rule (D-450): two `PunchIn`s never overlap, and a sentence has
/// at most one (`TooClose`). The span rule of each is checked per item.
fn zoom_rule(items: Vec<ValidItem>, t: &Transcript, dropped: &mut Vec<Dropped>) -> Vec<ValidItem> {
    keep_where(items, dropped, |item, kept| {
        if item.kind != EventKind::PunchIn {
            return None;
        }
        let sentence = sentence_of(t, item.anchors.start);
        let clash = kept
            .iter()
            .filter(|k| k.kind == EventKind::PunchIn)
            .any(|k| {
                let overlap = item.span.start < k.span.end && k.span.start < item.span.end;
                overlap || sentence_of(t, k.anchors.start) == sentence
            });
        clash.then_some(DropReason::TooClose)
    })
}

fn is_punch(item: &ValidItem) -> bool {
    matches!(
        item.params,
        EventParams::Emphasis {
            size: EmphasisSize::Punch,
            ..
        }
    )
}

/// D-452: a punch word (`Emphasis` of size `Punch`) starts at least 6 s after
/// the last one ends (`TooClose`), and a clip has at most 4 (`OverLimit`).
fn punch_rule(items: Vec<ValidItem>, dropped: &mut Vec<Dropped>) -> Vec<ValidItem> {
    keep_where(items, dropped, |item, kept| {
        if !is_punch(item) {
            return None;
        }
        let punches: Vec<&ValidItem> = kept.iter().filter(|k| is_punch(k)).collect();
        let near = punches.last().is_some_and(|last| {
            item.span.start.get() < last.span.end.get().saturating_add(PUNCH_GAP_MS)
        });
        if punches.len() >= MAX_PUNCH {
            Some(DropReason::OverLimit)
        } else if near {
            Some(DropReason::TooClose)
        } else {
            None
        }
    })
}

/// The kinds V-9 counts: a visual over the video, not a caption's own mark
/// or the camera (D-451).
fn is_visual(kind: EventKind) -> bool {
    match kind {
        EventKind::NumberReveal
        | EventKind::ListReveal
        | EventKind::FromTo
        | EventKind::IconPop
        | EventKind::Emoji
        | EventKind::HookTitle => true,
        EventKind::KeywordPop | EventKind::Emphasis | EventKind::PunchIn => false,
    }
}

/// V-9 (D-451), for a model's plan: at most one visual starts in any 2.5 s;
/// a later one is dropped (`Density`), the earlier anchor winning.
fn density_rule(items: Vec<ValidItem>, dropped: &mut Vec<Dropped>) -> Vec<ValidItem> {
    keep_where(items, dropped, |item, kept| {
        if !is_visual(item.kind) {
            return None;
        }
        let start = item.span.start.get();
        let crowded = kept.iter().filter(|k| is_visual(k.kind)).any(|k| {
            let other = k.span.start.get();
            start.abs_diff(other) < DENSITY_WINDOW_MS
        });
        crowded.then_some(DropReason::Density)
    })
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
        let newer = EditPlan::empty_rules(CATALOG_VERSION + 1);
        let refused = validate(newer, &t, &BTreeMap::new(), &cfg).unwrap_err();
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
        // D-440 (2): from V6.2 the extras are kept (D-454); cuts, chapters and music are not.
        plan.extras.hashtags = vec![label("#editing"), label("#video")];
        plan.music = Some(MusicCue {
            fits: true,
            energy: MusicEnergy::Calm,
        });
        let (valid, dropped) = run(plan);
        assert_eq!(valid.items().len(), 1);
        assert_eq!(dropped.len(), 4);
        let extras = valid.extras();
        assert_eq!(extras.hook.as_deref(), Some("I almost quit"));
        assert_eq!(extras.post_caption.as_deref(), Some("caption"));
        assert_eq!(extras.hashtags, ["#editing", "#video"]);
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

    // --- V6.2 (P185): the passes after V-6, tested on `ValidItem`s directly.

    fn valid(kind: EventKind, words: (u32, u32), ms: (u32, u32), params: EventParams) -> ValidItem {
        ValidItem {
            id: EventId::new(u64::from(words.0) * 100 + u64::from(ms.0)),
            kind,
            anchors: range(words.0, words.1),
            span: Span::new(TimeMs::new(ms.0), TimeMs::new(ms.1)).unwrap(),
            params,
            marks: Vec::new(),
            confidence: None,
        }
    }

    /// A one-word zoom on word `w`, at its time in `transcript()`.
    fn zoom(w: u32) -> ValidItem {
        let motion = offcut_types::ZoomMotion::Snap;
        let scale = offcut_types::ZoomScale::Mild;
        valid(
            EventKind::PunchIn,
            (w, w + 1),
            (500 + w * 700, 800 + w * 700),
            EventParams::PunchIn { scale, motion },
        )
    }

    fn emphasis(at: u32, size: EmphasisSize) -> ValidItem {
        let treatment = offcut_types::Treatment::Colour;
        let word = WordIdx::new(at / 1_000);
        valid(
            EventKind::Emphasis,
            (at / 1_000, at / 1_000 + 1),
            (at, at + 300),
            EventParams::Emphasis {
                word,
                treatment,
                size,
            },
        )
    }

    fn icon(at: u32) -> ValidItem {
        let icon = offcut_types::IconId::Clock;
        let word = WordIdx::new(at / 1_000);
        valid(
            EventKind::IconPop,
            (at / 1_000, at / 1_000 + 1),
            (at, at + 300),
            EventParams::IconPop { word, icon },
        )
    }

    fn starts(items: &[ValidItem]) -> Vec<u32> {
        items.iter().map(|i| i.span.start.get()).collect()
    }

    #[test]
    fn v3_zooms_never_overlap_and_a_sentence_has_one() {
        let t = transcript();
        let first = t.sentences.first().unwrap().words;
        let second = t.sentences.get(1).unwrap().words;
        // Two words of the first sentence, then the first word of the second.
        let (a, b, c) = (first.start.get(), first.start.get() + 2, second.start.get());
        let mut dropped = Vec::new();
        let kept = zoom_rule(vec![zoom(a), zoom(b), zoom(c)], &t, &mut dropped);
        assert_eq!(starts(&kept), [500 + a * 700, 500 + c * 700]);
        assert_eq!(
            dropped,
            [Dropped {
                kind: Some(EventKind::PunchIn),
                reason: DropReason::TooClose
            }]
        );
        // Overlapping spans, whatever the sentences.
        let mut long = zoom(c);
        long.span = Span::new(TimeMs::new(500), TimeMs::new(500 + c * 700 + 100)).unwrap();
        let mut dropped = Vec::new();
        let kept = zoom_rule(vec![long, zoom(c)], &t, &mut dropped);
        assert_eq!(kept.len(), 1);
        assert_eq!(dropped.len(), 1);
        // Other kinds pass untouched.
        let mut dropped = Vec::new();
        let kept = zoom_rule(vec![icon(1_000), icon(1_200)], &t, &mut dropped);
        assert_eq!((kept.len(), dropped.len()), (2, 0));
    }

    #[test]
    fn d452_a_punch_word_every_6_s_and_4_a_clip() {
        let punch = |at| emphasis(at, EmphasisSize::Punch);
        let items = vec![
            punch(0),
            punch(3_000),
            emphasis(3_500, EmphasisSize::Normal),
            punch(6_300),
            punch(13_000),
            punch(20_000),
            punch(27_000),
        ];
        let mut dropped = Vec::new();
        let kept = punch_rule(items, &mut dropped);
        assert_eq!(starts(&kept), [0, 3_500, 6_300, 13_000, 20_000]);
        let reasons: Vec<DropReason> = dropped.iter().map(|d| d.reason).collect();
        assert_eq!(reasons, [DropReason::TooClose, DropReason::OverLimit]);
        // 6,300 is 6,000 after the first punch's end (300): the gap is met exactly.
    }

    #[test]
    fn v9_one_visual_starts_in_any_2_5_s() {
        let items = vec![
            icon(1_000),
            emphasis(1_100, EmphasisSize::Normal),
            icon(2_000),
            zoom(2),
            icon(3_400),
            icon(3_500),
        ];
        let mut dropped = Vec::new();
        let kept = density_rule(items, &mut dropped);
        // 2,000 and 3,400 start within 2.5 s of 1,000; 3,500 does not.
        assert_eq!(starts(&kept), [1_000, 1_100, 1_900, 3_500]);
        assert_eq!(
            dropped,
            [Dropped {
                kind: Some(EventKind::IconPop),
                reason: DropReason::Density
            }; 2]
        );
        assert!(is_visual(EventKind::HookTitle) && is_visual(EventKind::NumberReveal));
        assert!(!is_visual(EventKind::KeywordPop) && !is_visual(EventKind::PunchIn));
    }

    #[test]
    fn dc7_an_older_catalog_is_read_a_newer_refused() {
        let t = transcript();
        let cfg = PlanConfig::default();
        for older in 0..=CATALOG_VERSION {
            assert!(validate(EditPlan::empty_rules(older), &t, &BTreeMap::new(), &cfg).is_ok());
        }
        let refused = validate(
            EditPlan::empty_rules(CATALOG_VERSION + 1),
            &t,
            &BTreeMap::new(),
            &cfg,
        )
        .unwrap_err();
        assert_eq!(refused.reason, RejectReason::Catalog);
    }
}
