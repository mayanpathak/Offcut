//! The checks of one plan item, V-2 to V-5 of TS §40.4, with its span and
//! marks: the per-kind half of `validate.rs` (v4implementation C.1). Private
//! to the crate: `validate` is the only caller.

use offcut_text::{Token, parse_quantity};
use offcut_types::{
    EventKind, EventParams, ItemParams, Label, ListItem, PlanItem, PlanSource, QuantityRef, Span,
    TimeMs, Transcript, WordIdx, WordRange,
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
        },
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
            | ParamTy::Bool => None,
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
        AnchorRule::ClipWide => true,
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
            };
            Ok((detected.clone(), marks))
        }
    }
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
) -> Result<ValidItem, DropReason> {
    let (row, limits) = v2_kind_and_params(&item, source, cfg)?;
    v3_anchors(&item, row, t)?;
    v4_labels(&item, limits, tokens)?;
    let (params, marks) = v5_params(&item, tokens, t)?;
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
