//! Which candidates become events when they compete for the same words or
//! the same screen (TS §17.5, section 9.8). Nothing here scores or moves a
//! candidate: it only keeps or drops.

use std::cmp::Ordering;

use offcut_types::{DetectedEvent, EventKind, Span, WordRange};

use crate::config::DetectorConfig;
use crate::number::Candidate;

/// The candidates to keep, in no particular order (`detect` sorts them). The
/// `fixed` events stay whatever happens and are not returned; a candidate that
/// conflicts with one is dropped (`redetect_sentence`, D-86).
pub fn resolve(
    mut cands: Vec<Candidate>,
    fixed: &[DetectedEvent],
    cfg: &DetectorConfig,
) -> Vec<Candidate> {
    // Step 0: priority, then score, then the earlier start, then the lower
    // anchors. The sort is stable, so equal candidates keep the finders' order.
    cands.sort_by(order);

    // Steps 1 and 2: no shared word, no two overlays on the screen at once.
    let mut kept: Vec<Candidate> = Vec::new();
    for cand in cands {
        let mine = Shown::of(cand.kind, cand.span, cand.anchors, cfg);
        let others = fixed
            .iter()
            .map(|e| Shown::of(e.kind, e.span, e.anchors, cfg))
            .chain(
                kept.iter()
                    .map(|k| Shown::of(k.kind, k.span, k.anchors, cfg)),
            );
        let clashes = others.into_iter().any(|other| mine.clashes(&other));
        if !clashes {
            kept.push(cand);
        }
    }

    cap_overlays(&mut kept, fixed, cfg);
    space_keywords(&mut kept, fixed, cfg);
    kept
}

fn priority(kind: EventKind) -> u8 {
    match kind {
        EventKind::FromTo => 0,
        EventKind::ListReveal => 1,
        EventKind::NumberReveal => 2,
        EventKind::KeywordPop => 3,
    }
}

fn order(a: &Candidate, b: &Candidate) -> Ordering {
    priority(a.kind)
        .cmp(&priority(b.kind))
        .then(b.score.total_cmp(&a.score))
        .then(a.span.start.cmp(&b.span.start))
        .then(a.anchors.start.cmp(&b.anchors.start))
        .then(a.anchors.end.cmp(&b.anchors.end))
}

/// What an event takes: its words, and for an overlay its display window,
/// from `overlay_lead_ms` before its first word to its hold after its last.
struct Shown {
    anchors: WordRange,
    window: Option<(u32, u32)>,
}

impl Shown {
    fn of(kind: EventKind, span: Span, anchors: WordRange, cfg: &DetectorConfig) -> Self {
        Self {
            anchors,
            window: window(kind, span, cfg),
        }
    }

    /// Shared words, or two windows that overlap or touch (V2 as built).
    fn clashes(&self, other: &Self) -> bool {
        let words =
            self.anchors.start < other.anchors.end && other.anchors.start < self.anchors.end;
        let screen = self
            .window
            .zip(other.window)
            .is_some_and(|((a0, a1), (b0, b1))| a0 <= b1 && b0 <= a1);
        words || screen
    }
}

fn window(kind: EventKind, span: Span, cfg: &DetectorConfig) -> Option<(u32, u32)> {
    let overlay = match kind {
        EventKind::NumberReveal | EventKind::ListReveal | EventKind::FromTo => true,
        EventKind::KeywordPop => false,
    };
    overlay.then(|| {
        (
            span.start.get().saturating_sub(cfg.overlay_lead_ms),
            span.end.get().saturating_add(cfg.hold_ms(kind)),
        )
    })
}

/// One overlay's window start and score; `kept` is its index in the kept
/// candidates, `None` for a fixed event, which is never dropped.
struct Start {
    at: u32,
    score: f32,
    anchors: WordRange,
    kept: Option<usize>,
}

/// Step 3: at most `overlay_max_per_window` overlay windows start inside any
/// `overlay_window_ms`. While an interval holds more, the lowest score among
/// its kept candidates goes, the later of equal scores.
fn cap_overlays(kept: &mut Vec<Candidate>, fixed: &[DetectedEvent], cfg: &DetectorConfig) {
    loop {
        let mut starts: Vec<Start> = fixed
            .iter()
            .filter_map(|e| {
                let (at, _) = window(e.kind, e.span, cfg)?;
                Some(Start {
                    at,
                    score: e.confidence.get(),
                    anchors: e.anchors,
                    kept: None,
                })
            })
            .chain(kept.iter().enumerate().filter_map(|(index, c)| {
                let (at, _) = window(c.kind, c.span, cfg)?;
                Some(Start {
                    at,
                    score: c.score,
                    anchors: c.anchors,
                    kept: Some(index),
                })
            }))
            .collect();
        starts.sort_by(|a, b| a.at.cmp(&b.at).then(a.anchors.start.cmp(&b.anchors.start)));

        let mut drop = None;
        for (from, first) in starts.iter().enumerate() {
            let until = first.at.saturating_add(cfg.overlay_window_ms);
            let inside = starts
                .get(from..)
                .unwrap_or(&[])
                .iter()
                .take_while(|s| s.at < until);
            if inside.clone().count() <= cfg.overlay_max_per_window {
                continue;
            }
            // The lowest score; of equal scores the later, which sorts last.
            drop = inside
                .filter(|s| s.kept.is_some())
                .min_by(|a, b| {
                    a.score
                        .total_cmp(&b.score)
                        .then(b.at.cmp(&a.at))
                        .then(b.anchors.start.cmp(&a.anchors.start))
                })
                .and_then(|s| s.kept);
            if drop.is_some() {
                break;
            }
        }
        match drop {
            Some(index) if index < kept.len() => {
                kept.remove(index);
            }
            Some(_) | None => return,
        }
    }
}

/// One KeywordPop, kept or fixed, in time order.
struct Pop {
    at: u32,
    score: f32,
    anchors: WordRange,
    kept: Option<usize>,
}

/// Step 4: one KeywordPop per sentence (the highest score, then the
/// earliest), then at least `keyword_min_gap_ms` between two, the higher
/// score staying. Fixed KeywordPops count and stay; they are never in a
/// sentence of the candidates, because `redetect_sentence` fixes only events
/// wholly outside its sentences.
fn space_keywords(kept: &mut Vec<Candidate>, fixed: &[DetectedEvent], cfg: &DetectorConfig) {
    let is_pop = |c: &Candidate| c.kind == EventKind::KeywordPop;
    let beats = |a: &Candidate, b: &Candidate| {
        a.score > b.score
            || (a.score == b.score
                && (a.span.start, a.anchors.start) < (b.span.start, b.anchors.start))
    };
    let mut gone: Vec<bool> = kept
        .iter()
        .map(|c| {
            is_pop(c)
                && kept
                    .iter()
                    .any(|other| is_pop(other) && other.sentence == c.sentence && beats(other, c))
        })
        .collect();

    let mut pops: Vec<Pop> = fixed
        .iter()
        .filter(|e| e.kind == EventKind::KeywordPop)
        .map(|e| Pop {
            at: e.span.start.get(),
            score: e.confidence.get(),
            anchors: e.anchors,
            kept: None,
        })
        .chain(
            kept.iter()
                .enumerate()
                .filter(|(index, c)| is_pop(c) && !gone.get(*index).is_some_and(|g| *g))
                .map(|(index, c)| Pop {
                    at: c.span.start.get(),
                    score: c.score,
                    anchors: c.anchors,
                    kept: Some(index),
                }),
        )
        .collect();
    pops.sort_by(|a, b| a.at.cmp(&b.at).then(a.anchors.start.cmp(&b.anchors.start)));

    let mut previous: Option<&Pop> = None;
    for pop in &pops {
        let Some(before) =
            previous.filter(|before| pop.at.saturating_sub(before.at) < cfg.keyword_min_gap_ms)
        else {
            previous = Some(pop);
            continue;
        };
        // Too close: a fixed one stays; else the higher score, the earlier on a tie.
        let (loser, winner_is_pop) = match (before.kept, pop.kept) {
            (None, None) => (None, true),
            (None, Some(_)) => (pop.kept, false),
            (Some(_), None) => (before.kept, true),
            (Some(_), Some(_)) if pop.score > before.score => (before.kept, true),
            (Some(_), Some(_)) => (pop.kept, false),
        };
        if let Some(slot) = loser.and_then(|index| gone.get_mut(index)) {
            *slot = true;
        }
        if winner_is_pop {
            previous = Some(pop);
        }
    }

    let mut index = 0;
    kept.retain(|_| {
        let keep = !gone.get(index).is_some_and(|g| *g);
        index += 1;
        keep
    });
}

#[cfg(test)]
mod tests {
    use offcut_types::{Confidence, EventParams, Quantity, SentenceIdx, TimeMs, Unit, WordIdx};

    use super::*;
    use crate::event_id;

    fn number(at: u32, word: u32, score: f32) -> Candidate {
        Candidate {
            kind: EventKind::NumberReveal,
            anchors: WordRange {
                start: WordIdx::new(word),
                end: WordIdx::new(word + 1),
            },
            span: Span::new(TimeMs::new(at), TimeMs::new(at + 300)).unwrap(),
            score,
            params: EventParams::NumberReveal {
                value: Quantity {
                    value: 1_000.0,
                    unit: Unit::Usd,
                    display: "$1,000".to_owned(),
                },
                label: None,
            },
            parts: Vec::new(),
            sentence: SentenceIdx::new(word),
        }
    }

    fn pop(at: u32, word: u32, sentence: u32, score: f32) -> Candidate {
        Candidate {
            kind: EventKind::KeywordPop,
            anchors: WordRange {
                start: WordIdx::new(word),
                end: WordIdx::new(word + 1),
            },
            span: Span::new(TimeMs::new(at), TimeMs::new(at + 300)).unwrap(),
            score,
            params: EventParams::KeywordPop {
                word: WordIdx::new(word),
            },
            parts: Vec::new(),
            sentence: SentenceIdx::new(sentence),
        }
    }

    fn event(c: &Candidate) -> DetectedEvent {
        DetectedEvent {
            id: event_id(c.kind, c.anchors),
            kind: c.kind,
            span: c.span,
            anchors: c.anchors,
            params: c.params.clone(),
            confidence: Confidence::new(c.score).unwrap(),
            enabled: true,
        }
    }

    fn starts(kept: &[Candidate]) -> Vec<u32> {
        let mut at: Vec<u32> = kept.iter().map(|c| c.span.start.get()).collect();
        at.sort_unstable();
        at
    }

    #[test]
    fn a_fixed_number_at_10_s_drops_a_new_one_at_10_5_s() {
        let cfg = DetectorConfig::default();
        let fixed = [event(&number(10_000, 20, 1.0))];
        assert_eq!(resolve(vec![number(10_500, 22, 1.0)], &fixed, &cfg), []);
        // Far enough: kept, and the fixed event is not returned.
        assert_eq!(
            starts(&resolve(vec![number(12_000, 22, 1.0)], &fixed, &cfg)),
            [12_000]
        );
    }

    #[test]
    fn shared_words_go_to_the_higher_priority_then_the_higher_score() {
        let cfg = DetectorConfig::default();
        let mut list = number(1_000, 3, 0.9);
        list.kind = EventKind::ListReveal;
        list.anchors.end = WordIdx::new(9);
        let kept = resolve(
            vec![number(30_000, 5, 1.0), list.clone(), pop(40_000, 6, 1, 1.0)],
            &[],
            &cfg,
        );
        // The list's words hold word 5 and word 6; the number and the pop go,
        // whatever their times.
        assert_eq!(kept, [list]);
    }

    #[test]
    fn windows_that_touch_clash_and_the_earlier_of_equals_stays() {
        let cfg = DetectorConfig::default();
        // Shown until 1,300 + 1,400 = 2,700; the next from 150 ms before it.
        assert_eq!(
            starts(&resolve(
                vec![number(2_850, 1, 1.0), number(1_000, 0, 1.0)],
                &[],
                &cfg
            )),
            [1_000]
        );
        assert_eq!(
            starts(&resolve(
                vec![number(2_851, 1, 1.0), number(1_000, 0, 1.0)],
                &[],
                &cfg
            )),
            [1_000, 2_851]
        );
        // The higher score stays even when later.
        assert_eq!(
            starts(&resolve(
                vec![number(1_000, 0, 0.9), number(2_000, 1, 1.0)],
                &[],
                &cfg
            )),
            [2_000]
        );
    }

    #[test]
    fn at_most_three_overlays_start_in_ten_seconds() {
        let cfg = DetectorConfig::default();
        let four = vec![
            number(500, 0, 1.0),
            number(3_000, 1, 1.0),
            number(5_500, 2, 0.9),
            number(8_000, 3, 1.0),
        ];
        // The lowest score goes.
        assert_eq!(
            starts(&resolve(four.clone(), &[], &cfg)),
            [500, 3_000, 8_000]
        );
        // Of equal scores, the later.
        let mut equal = four;
        equal[2].score = 1.0;
        assert_eq!(
            starts(&resolve(equal.clone(), &[], &cfg)),
            [500, 3_000, 5_500]
        );
        // A fixed event counts and stays.
        let fixed = [event(&number(500, 0, 0.8))];
        let rest: Vec<Candidate> = equal.into_iter().skip(1).collect();
        assert_eq!(starts(&resolve(rest, &fixed, &cfg)), [3_000, 5_500]);
    }

    #[test]
    fn one_keyword_pop_per_sentence_and_two_and_a_half_seconds_apart() {
        let cfg = DetectorConfig::default();
        // One sentence: the higher score, then the earlier.
        assert_eq!(
            starts(&resolve(
                vec![pop(1_000, 0, 0, 0.9), pop(5_000, 4, 0, 1.0)],
                &[],
                &cfg
            )),
            [5_000]
        );
        assert_eq!(
            starts(&resolve(
                vec![pop(5_000, 4, 0, 0.9), pop(1_000, 0, 0, 0.9)],
                &[],
                &cfg
            )),
            [1_000]
        );
        // Two sentences, 2,000 ms apart: the higher score, wherever it is.
        assert_eq!(
            starts(&resolve(
                vec![pop(1_000, 0, 0, 0.9), pop(3_000, 4, 1, 1.0)],
                &[],
                &cfg
            )),
            [3_000]
        );
        assert_eq!(
            starts(&resolve(
                vec![pop(1_000, 0, 0, 1.0), pop(3_000, 4, 1, 0.9)],
                &[],
                &cfg
            )),
            [1_000]
        );
        assert_eq!(
            starts(&resolve(
                vec![pop(1_000, 0, 0, 0.9), pop(3_500, 4, 1, 0.9)],
                &[],
                &cfg
            )),
            [1_000, 3_500]
        );
        // A fixed pop stays, and a close candidate goes even with a higher score.
        let fixed = [event(&pop(1_000, 0, 0, 0.8))];
        assert_eq!(resolve(vec![pop(3_000, 4, 1, 1.0)], &fixed, &cfg), []);
        // A pop is no overlay: a number beside it does not clash.
        assert_eq!(
            starts(&resolve(
                vec![pop(1_000, 0, 0, 0.9), number(1_200, 1, 1.0)],
                &[],
                &cfg
            )),
            [1_000, 1_200]
        );
    }
}
