//! The checks of one plan item, V-2 to V-5 of TS §40.4, with its span and
//! marks: the per-kind half of `validate.rs` (v4implementation C.1). Private
//! to the crate: `validate` is the only caller.

use offcut_text::{Token, parse_quantity};
use offcut_types::{
    EventKind, EventParams, ItemParams, Label, ListItem, PlanItem, PlanSource, QuantityRef, Span,
    TimeMs, Transcript, WordIdx, WordRange, ZoomMotion,
};

use crate::catalog::{AnchorRule, ItemSpec, ParamTy, Producers, spec};
use crate::config::{KindLimits, PlanConfig};
use crate::resolve::{inherit, label_digits_ok, quantity_at};
use crate::validate::{DropReason, ValidItem};

/// The kind an item's params are for.
fn params_kind(params: &ItemParams) -> EventKind {
    match params {
        ItemParams::NumberReveal { .. } => EventKind::NumberReveal,
        ItemParams::ListReveal { .. } => EventKind::ListReveal,
        ItemParams::FromTo { .. } => EventKind::FromTo,
        ItemParams::KeywordPop { .. } => EventKind::KeywordPop,
        ItemParams::Detected(detected) => match detected {
            EventParams::NumberReveal { .. } => EventKind::NumberReveal,
            EventParams::ListReveal { .. } => EventKind::ListReveal,
            EventParams::FromTo { .. } => EventKind::FromTo,
            EventParams::KeywordPop { .. } => EventKind::KeywordPop,
            EventParams::Emphasis { .. } => EventKind::Emphasis,
            EventParams::IconPop { .. } => EventKind::IconPop,
            EventParams::PunchIn { .. } => EventKind::PunchIn,
            EventParams::Emoji { .. } => EventKind::Emoji,
            EventParams::HookTitle { .. } => EventKind::HookTitle,
        },
        ItemParams::Emphasis { .. } => EventKind::Emphasis,
        ItemParams::IconPop { .. } => EventKind::IconPop,
        ItemParams::PunchIn { .. } => EventKind::PunchIn,
        ItemParams::Emoji { .. } => EventKind::Emoji,
        ItemParams::HookTitle { .. } => EventKind::HookTitle,
    }
}

/// The most list points a kind takes, from its catalog row.
fn max_points(row: &ItemSpec) -> u32 {
    row.params
        .iter()
        .find_map(|param| match param.ty {
            ParamTy::ListPoints { max, .. } => Some(max),
            ParamTy::Label { .. }
            | ParamTy::QuantityRef
            | ParamTy::WordIdx
            | ParamTy::Enum(_)
            | ParamTy::Bool
            | ParamTy::Index { .. } => None,
        })
        .unwrap_or(0)
}

/// V-2: the kind is known, the producer may write it, the params are its own
/// and hold what the kind requires.
fn v2_kind_and_params(
    item: &PlanItem,
    source: &PlanSource,
    cfg: &PlanConfig,
) -> Result<(&'static ItemSpec, KindLimits), DropReason> {
    let row = spec(item.kind).ok_or(DropReason::UnknownKind)?;
    let limits = *cfg
        .per_kind
        .get(&item.kind)
        .ok_or(DropReason::UnknownKind)?;
    let from_rules = matches!(source, PlanSource::Rules);
    let detected = matches!(item.params, ItemParams::Detected(_));
    let allowed = match row.producers {
        Producers::Rules => from_rules,
        Producers::Model => !from_rules,
        Producers::Both => true,
    };
    // The detector's own reading comes from the rules only; a word pointer from a model only.
    if detected != from_rules || !allowed {
        return Err(DropReason::WrongProducer);
    }
    if params_kind(&item.params) != item.kind {
        return Err(DropReason::ParamsMismatch);
    }
    let points = match &item.params {
        ItemParams::ListReveal { items, .. } => Some(items.len()),
        ItemParams::Detected(EventParams::ListReveal { items, .. }) => Some(items.len()),
        ItemParams::NumberReveal { .. }
        | ItemParams::FromTo { .. }
        | ItemParams::KeywordPop { .. }
        | ItemParams::Detected(_) => None,
        ItemParams::Emphasis { .. }
        | ItemParams::IconPop { .. }
        | ItemParams::PunchIn { .. }
        | ItemParams::Emoji { .. }
        | ItemParams::HookTitle { .. } => None,
    };
    if let Some(points) = points {
        let max = usize::try_from(max_points(row)).unwrap_or(usize::MAX);
        let over = !detected && points > max;
        if points == 0 || over {
            return Err(DropReason::ParamsMismatch);
        }
    }
    Ok((row, limits))
}

fn inside(anchors: WordRange, word: WordIdx) -> bool {
    anchors.start <= word && word < anchors.end
}

/// A range of words inside the anchors, and not empty.
fn range_inside(anchors: WordRange, words: WordRange) -> bool {
    words.start < words.end && anchors.start <= words.start && words.end <= anchors.end
}

/// V-3: the anchors are words of the transcript (`AnchorsOutOfRange`) and as
/// many as the kind takes; every word an item points at is one of its anchors
/// (`AnchorRule`); a detector event's own times lie in the transcript
/// (`AnchorsOutOfRange`).
fn v3_anchors(item: &PlanItem, row: &ItemSpec, t: &Transcript) -> Result<(), DropReason> {
    let anchors = item.anchors;
    let words = u32::try_from(t.words.len()).unwrap_or(u32::MAX);
    if anchors.start >= anchors.end || anchors.end.get() > words {
        return Err(DropReason::AnchorsOutOfRange);
    }
    let count = anchors.end.get() - anchors.start.get();
    let fits = match row.anchor {
        AnchorRule::Words { min, max } => (min..=max).contains(&count),
        AnchorRule::Point => count == 1,
        // V6.2 (D-453): every word of the clip, from the first.
        AnchorRule::ClipWide => anchors.start.get() == 0 && anchors.end.get() == words,
    };
    if !fits {
        return Err(DropReason::AnchorRule);
    }
    let pointed = match &item.params {
        ItemParams::NumberReveal { value, .. } => range_inside(anchors, value.words),
        ItemParams::ListReveal { items, .. } => items.iter().all(|point| inside(anchors, point.at)),
        ItemParams::FromTo { from, to, .. } => {
            range_inside(anchors, from.words) && range_inside(anchors, to.words)
        }
        ItemParams::KeywordPop { word }
        | ItemParams::Detected(EventParams::KeywordPop { word }) => inside(anchors, *word),
        ItemParams::Detected(_) => true,
        // V6.2 (D-446): the word an item marks is one of its anchors.
        ItemParams::Emphasis { word, .. }
        | ItemParams::IconPop { word, .. }
        | ItemParams::Emoji { word, .. } => inside(anchors, *word),
        ItemParams::PunchIn { motion, .. } => zoom_fits(*motion, anchors, t),
        ItemParams::HookTitle { .. } => true,
    };
    if !pointed {
        return Err(DropReason::AnchorRule);
    }
    if let ItemParams::Detected(detected) = &item.params
        && !detected_times_inside(detected, t)
    {
        return Err(DropReason::AnchorsOutOfRange);
    }
    Ok(())
}

/// V-3's zoom rule (D-450): a `Snap` spans 1 to 3 words, a `Push` 2 to 40,
/// both inside one sentence.
fn zoom_fits(motion: ZoomMotion, anchors: WordRange, t: &Transcript) -> bool {
    let count = anchors.end.get().saturating_sub(anchors.start.get());
    let fits = match motion {
        ZoomMotion::Snap => (1..=3).contains(&count),
        ZoomMotion::Push => (2..=40).contains(&count),
    };
    fits && t
        .sentences
        .iter()
        .any(|s| s.words.start <= anchors.start && anchors.end <= s.words.end)
}

/// A detector event's own times lie in the transcript.
fn detected_times_inside(detected: &EventParams, t: &Transcript) -> bool {
    let first = t.words.first().map(|w| w.start_ms);
    let last = t.words.last().map(|w| w.end_ms);
    let in_clip = |at: TimeMs| first.zip(last).is_some_and(|(a, b)| a <= at && at <= b);
    match detected {
        EventParams::ListReveal { items, .. } => items.iter().all(|item| in_clip(item.at)),
        EventParams::NumberReveal { .. }
        | EventParams::FromTo { .. }
        | EventParams::KeywordPop { .. } => true,
        // V6.2 (D-446): no times of their own.
        EventParams::Emphasis { .. }
        | EventParams::IconPop { .. }
        | EventParams::PunchIn { .. }
        | EventParams::Emoji { .. }
        | EventParams::HookTitle { .. } => true,
    }
}

/// V-4: every label of a word-pointing item fits its length, in characters,
/// and says no digit the transcript does not. The detector's texts are not
/// checked: it wrote them from the transcript.
fn v4_labels(item: &PlanItem, limits: KindLimits, tokens: &[Token]) -> Result<(), DropReason> {
    let mut labels: Vec<(&Label, u32)> = Vec::new();
    match &item.params {
        ItemParams::NumberReveal { label, .. } | ItemParams::FromTo { label, .. } => {
            labels.extend(label.iter().map(|l| (l, limits.label_chars)));
        }
        ItemParams::ListReveal { header, items } => {
            labels.extend(header.iter().map(|l| (l, limits.header_chars)));
            labels.extend(items.iter().map(|point| (&point.label, limits.label_chars)));
        }
        ItemParams::KeywordPop { .. } | ItemParams::Detected(_) => {}
        // V6.2 (D-446): no label; the hook is an extra, checked with the extras (D-454).
        ItemParams::Emphasis { .. }
        | ItemParams::IconPop { .. }
        | ItemParams::PunchIn { .. }
        | ItemParams::Emoji { .. }
        | ItemParams::HookTitle { .. } => {}
    }
    let too_long = |(label, max): &(&Label, u32)| {
        u32::try_from(label.as_str().chars().count()).unwrap_or(u32::MAX) > *max
    };
    if labels.iter().any(too_long) {
        return Err(DropReason::LabelTooLong);
    }
    if labels
        .iter()
        .any(|(label, _)| !label_digits_ok(label.as_str(), tokens))
    {
        return Err(DropReason::LabelDigits);
    }
    Ok(())
}

fn start_of(t: &Transcript, word: WordIdx) -> Option<TimeMs> {
    t.words
        .get(usize::try_from(word.get()).ok()?)
        .map(|w| w.start_ms)
}

/// Where the second quantity inside the anchors starts: a detector FromTo was
/// built from exactly two (D-106), and its params keep no word of either.
fn second_quantity_start(tokens: &[Token], anchors: WordRange, t: &Transcript) -> Option<TimeMs> {
    let first = tokens
        .iter()
        .position(|token| token.word >= anchors.start)?;
    let rest = tokens.get(first..)?;
    let len = rest
        .iter()
        .take_while(|token| token.word < anchors.end)
        .count();
    let inside = rest.get(..len)?;
    let (mut at, mut found) = (0, 0);
    while at < inside.len() {
        match inside.get(at..).and_then(parse_quantity) {
            Some((used, _)) => {
                found += 1;
                if found == 2 {
                    return start_of(t, inside.get(at)?.word);
                }
                at += used.max(1);
            }
            None => at += 1,
        }
    }
    None
}

fn resolved(tokens: &[Token], value: QuantityRef) -> Result<offcut_types::Quantity, DropReason> {
    quantity_at(tokens, value.words).ok_or(DropReason::NumberUnresolved)
}

/// V-5: the params with every number read from the effective words, and the
/// marks a visual steps at (FromTo's second value, §8.7).
fn v5_params(
    item: &PlanItem,
    (row, hook): (&ItemSpec, Option<&str>),
    tokens: &[Token],
    t: &Transcript,
) -> Result<(EventParams, Vec<TimeMs>), DropReason> {
    let text = |label: &Option<Label>| label.as_ref().map(|l| l.as_str().to_owned());
    match &item.params {
        ItemParams::NumberReveal { value, label } => Ok((
            EventParams::NumberReveal {
                value: resolved(tokens, *value)?,
                label: text(label),
            },
            Vec::new(),
        )),
        ItemParams::FromTo { from, to, label } => {
            let (from_q, to_q) = inherit(resolved(tokens, *from)?, resolved(tokens, *to)?);
            let mark = start_of(t, to.words.start).ok_or(DropReason::AnchorsOutOfRange)?;
            let params = EventParams::FromTo {
                from: from_q,
                to: to_q,
                label: text(label),
            };
            Ok((params, vec![mark]))
        }
        ItemParams::ListReveal { header, items } => {
            let mut list = Vec::with_capacity(items.len());
            for (ordinal, point) in (1u8..).zip(items) {
                list.push(ListItem {
                    ordinal,
                    text: point.label.as_str().to_owned(),
                    at: start_of(t, point.at).ok_or(DropReason::AnchorsOutOfRange)?,
                });
            }
            let count = u8::try_from(list.len()).map_err(|_| DropReason::ParamsMismatch)?;
            let header = text(header).unwrap_or_default();
            Ok((
                EventParams::ListReveal {
                    count,
                    header,
                    items: list,
                },
                Vec::new(),
            ))
        }
        ItemParams::KeywordPop { word } => {
            Ok((EventParams::KeywordPop { word: *word }, Vec::new()))
        }
        ItemParams::Detected(detected) => {
            let marks = match detected {
                EventParams::FromTo { .. } => vec![
                    second_quantity_start(tokens, item.anchors, t)
                        .ok_or(DropReason::NumberUnresolved)?,
                ],
                EventParams::NumberReveal { .. }
                | EventParams::ListReveal { .. }
                | EventParams::KeywordPop { .. } => Vec::new(),
                EventParams::Emphasis { .. }
                | EventParams::IconPop { .. }
                | EventParams::PunchIn { .. }
                | EventParams::Emoji { .. }
                | EventParams::HookTitle { .. } => Vec::new(),
            };
            Ok((detected.clone(), marks))
        }
        // V6.2 (D-446): a model's pointers and choices are already resolved.
        ItemParams::Emphasis {
            word,
            treatment,
            size,
        } => Ok((
            EventParams::Emphasis {
                word: *word,
                treatment: *treatment,
                size: *size,
            },
            Vec::new(),
        )),
        ItemParams::IconPop { word, icon } => Ok((
            EventParams::IconPop {
                word: *word,
                icon: *icon,
            },
            Vec::new(),
        )),
        ItemParams::PunchIn { scale, motion } => Ok((
            EventParams::PunchIn {
                scale: *scale,
                motion: *motion,
            },
            Vec::new(),
        )),
        ItemParams::Emoji { word, emoji, place } => Ok((
            EventParams::Emoji {
                word: *word,
                emoji: *emoji,
                place: *place,
            },
            Vec::new(),
        )),
        ItemParams::HookTitle { accent } => Ok((hook_title(*accent, row, hook)?, Vec::new())),
    }
}

/// D-453: a `HookTitle` shows the plan's valid hook (`HookMissing` without
/// one); its accent is the index of a word of that hook, within the row's
/// `Index` (`ParamsMismatch`).
fn hook_title(accent: u8, row: &ItemSpec, hook: Option<&str>) -> Result<EventParams, DropReason> {
    let text = hook.ok_or(DropReason::HookMissing)?;
    let max = row.params.iter().find_map(|param| match param.ty {
        ParamTy::Index { max } => Some(max),
        ParamTy::Label { .. }
        | ParamTy::QuantityRef
        | ParamTy::WordIdx
        | ParamTy::ListPoints { .. }
        | ParamTy::Enum(_)
        | ParamTy::Bool => None,
    });
    let words = text.split_whitespace().count();
    let in_row = max.is_none_or(|max| u32::from(accent) <= max);
    if !in_row || usize::from(accent) >= words {
        return Err(DropReason::ParamsMismatch);
    }
    Ok(EventParams::HookTitle {
        text: text.to_owned(),
        accent,
    })
}

/// From the first anchor word's start to the last one's end: the detector's
/// rule for an event's span too.
fn span_of(anchors: WordRange, t: &Transcript) -> Option<Span> {
    let last = anchors.end.get().checked_sub(1)?;
    let end = t.words.get(usize::try_from(last).ok()?)?.end_ms;
    Span::new(start_of(t, anchors.start)?, end)
}

/// One item, checked and resolved, or the reason it is dropped.
pub(crate) fn check_item(
    item: PlanItem,
    source: &PlanSource,
    tokens: &[Token],
    t: &Transcript,
    cfg: &PlanConfig,
    hook: Option<&str>,
) -> Result<ValidItem, DropReason> {
    let (row, limits) = v2_kind_and_params(&item, source, cfg)?;
    v3_anchors(&item, row, t)?;
    v4_labels(&item, limits, tokens)?;
    let (params, marks) = v5_params(&item, (row, hook), tokens, t)?;
    let span = span_of(item.anchors, t).ok_or(DropReason::AnchorsOutOfRange)?;
    Ok(ValidItem {
        id: item.id,
        kind: item.kind,
        anchors: item.anchors,
        span,
        params,
        marks,
        confidence: item.confidence,
    })
}

#[cfg(test)]
mod tests {
    use offcut_text::{RawWord, normalize_transcript};
    use offcut_types::EventKind;

    use super::*;
    use crate::catalog::{ParamSpec, Producers};

    /// A row for a hook: `Index { max: 8 }` (§6.2), nothing else read here.
    const HOOK_ROW: ItemSpec = ItemSpec {
        kind: EventKind::HookTitle,
        name: "hook_title",
        purpose: "",
        anchor: AnchorRule::ClipWide,
        params: &[ParamSpec {
            name: "accent",
            ty: ParamTy::Index { max: 8 },
            required: true,
        }],
        producers: Producers::Model,
        zone: "hook",
        max_per_clip: 1,
        min_gap_ms: 0,
    };

    #[test]
    fn d453_a_hook_title_needs_the_hook_and_an_accent_inside_it() {
        assert_eq!(hook_title(0, &HOOK_ROW, None), Err(DropReason::HookMissing));
        assert_eq!(
            hook_title(1, &HOOK_ROW, Some("I almost quit")),
            Ok(EventParams::HookTitle {
                text: "I almost quit".to_owned(),
                accent: 1
            })
        );
        // The accent indexes the hook's words, not the transcript's.
        assert_eq!(
            hook_title(3, &HOOK_ROW, Some("I almost quit")),
            Err(DropReason::ParamsMismatch)
        );
        let nine = "one two three four five six seven eight nine";
        assert!(hook_title(8, &HOOK_ROW, Some(nine)).is_ok());
        assert_eq!(
            hook_title(9, &HOOK_ROW, Some(&format!("{nine} ten"))),
            Err(DropReason::ParamsMismatch)
        );
    }

    fn transcript() -> Transcript {
        let words = [
            "We", "doubled", "it.", "Then", "we", "doubled", "it", "again.",
        ];
        let words = (0u32..)
            .zip(words)
            .map(|(i, text)| RawWord {
                text: text.to_owned(),
                start_ms: 500 + i * 700,
                end_ms: 800 + i * 700,
                confidence: 1.0,
            })
            .collect();
        normalize_transcript(words, "test")
    }

    fn words(start: u32, end: u32) -> WordRange {
        WordRange {
            start: WordIdx::new(start),
            end: WordIdx::new(end),
        }
    }

    #[test]
    fn d450_a_snap_spans_1_to_3_words_a_push_2_to_40_inside_one_sentence() {
        let t = transcript();
        assert_eq!(t.sentences.len(), 2, "{:?}", t.sentences);
        let snap = |w| zoom_fits(ZoomMotion::Snap, w, &t);
        let push = |w| zoom_fits(ZoomMotion::Push, w, &t);
        assert!(snap(words(1, 2)) && snap(words(0, 3)));
        assert!(!snap(words(3, 7)), "four words");
        assert!(!push(words(1, 2)), "one word");
        assert!(push(words(3, 8)));
        // Across the full stop of "it.".
        assert!(!snap(words(2, 4)) && !push(words(1, 5)));
    }
}
