//! Which moments of a transcript become visual events: NumberReveal,
//! ListReveal, FromTo and KeywordPop (TS §17). An event below its kind's
//! threshold is never made (INV-6), the same inputs give the same events
//! (INV-7), and nothing here lays out or draws.

pub mod config;
pub mod event_id;
mod exclusions;
mod from_to;
mod keyword;
mod list;
pub mod number;
mod resolve;

use std::cmp::Ordering;
use std::collections::BTreeMap;

use offcut_text::tokenize;
use offcut_types::{
    Confidence, DetectedEvent, EventKind, Prosody, Sentence, SentenceIdx, Transcript, WordIdx,
    WordRange,
};

pub use crate::config::DetectorConfig;
use crate::config::SCORE_NOISE;
pub use crate::event_id::event_id;
use crate::number::Candidate;

/// The events of a transcript, in the order they are spoken. `edits` are the
/// user's word edits: detection reads the text as edited (TS §17.2). The
/// same inputs give the same events, with the same ids. An empty transcript
/// gives none; nothing here fails.
pub fn detect(
    t: &Transcript,
    p: &Prosody,
    edits: &BTreeMap<WordIdx, String>,
    cfg: &DetectorConfig,
) -> Vec<DetectedEvent> {
    let cands = candidates(t, p, edits, cfg);
    // 6-7: what competes is resolved, and each kept candidate is an event.
    into_events(resolve::resolve(cands, &[], cfg), Vec::new())
}

/// The events after an edit in sentence `s` (section 9.2, D-86): the events
/// of `prev` outside the scope stay as they are, and the scope is detected
/// again with them fixed, so the rules of TS §17.5 hold for the whole clip.
/// An event whose kind and anchors did not change keeps its id (D-47). `s`
/// out of range returns `prev` unchanged.
pub fn redetect_sentence(
    t: &Transcript,
    p: &Prosody,
    edits: &BTreeMap<WordIdx, String>,
    s: SentenceIdx,
    prev: &[DetectedEvent],
    cfg: &DetectorConfig,
) -> Vec<DetectedEvent> {
    let Some(sentence) = t.sentences.get(s.get() as usize) else {
        return prev.to_vec();
    };
    let scope = scope(t, sentence, prev, cfg);
    let outside: Vec<DetectedEvent> = prev
        .iter()
        .filter(|e| !meets(e.anchors, scope))
        .cloned()
        .collect();
    let mut cands = candidates(t, p, edits, cfg);
    cands.retain(|c| scope.start <= c.anchors.start && c.anchors.end <= scope.end);
    let kept = resolve::resolve(cands, &outside, cfg);
    into_events(kept, outside)
}

/// Steps 1-5 of `detect`: the candidates of the text as edited that clear
/// the threshold of their kind.
fn candidates(
    t: &Transcript,
    p: &Prosody,
    edits: &BTreeMap<WordIdx, String>,
    cfg: &DetectorConfig,
) -> Vec<Candidate> {
    // 1-3: the text as edited, and the candidates of the four finders, each
    // reading the quantities of each sentence again (D-104, D-109).
    let toks = tokenize(&t.words, edits);
    let mut cands = from_to::find(&toks, t, p, cfg);
    cands.extend(list::find(&toks, t, p, cfg));
    cands.extend(number::find(&toks, t, p, cfg));
    cands.extend(keyword::find(&toks, t, p, cfg));
    // 4: years, versions, clock times, phone-like runs, bare small numbers,
    // markers, hidden words.
    exclusions::apply(&mut cands, &toks, t, edits);
    // 5: the threshold of each kind, on the score without f32 noise.
    for cand in &mut cands {
        cand.score = settled(cand.score);
    }
    cands.retain(|c| c.score >= cfg.threshold(c.kind));
    cands
}

/// Step 7: each kept candidate as an event, beside the `fixed` ones, all in
/// spoken order.
fn into_events(kept: Vec<Candidate>, fixed: Vec<DetectedEvent>) -> Vec<DetectedEvent> {
    let mut events = fixed;
    events.extend(kept.into_iter().filter_map(|c| {
        Some(DetectedEvent {
            id: event_id(c.kind, c.anchors),
            kind: c.kind,
            span: c.span,
            anchors: c.anchors,
            params: c.params,
            // A score is within 0 to 1; one that is not makes no event.
            confidence: Confidence::new(c.score)?,
            enabled: true,
        })
    }));
    events.sort_by(spoken_order);
    events
}

/// The words `redetect_sentence` reads again (D-86): from sentence `s` to
/// the last sentence that starts within `list_window_ms` after it, a list
/// announced in `s` finding its markers there; then the sentences of every
/// event of `prev` that reaches into the scope, until none crosses its edge.
/// An event of `prev` is so either wholly inside, detected again, or wholly
/// outside, kept: none is lost at the edge.
fn scope(t: &Transcript, s: &Sentence, prev: &[DetectedEvent], cfg: &DetectorConfig) -> WordRange {
    let from = s.start_ms.get();
    let until = from.saturating_add(cfg.list_window_ms);
    let mut scope = s.words;
    for later in &t.sentences {
        let at = later.start_ms.get();
        if from < at && at < until {
            scope = hull(scope, later.words);
        }
    }
    loop {
        let mut grown = scope;
        for e in prev.iter().filter(|e| meets(e.anchors, scope)) {
            for touched in t.sentences.iter().filter(|x| meets(x.words, e.anchors)) {
                grown = hull(grown, touched.words);
            }
        }
        if grown == scope {
            return scope;
        }
        scope = grown;
    }
}

/// Two word ranges that share a word.
fn meets(a: WordRange, b: WordRange) -> bool {
    a.start < b.end && b.start < a.end
}

fn hull(a: WordRange, b: WordRange) -> WordRange {
    WordRange {
        start: a.start.min(b.start),
        end: a.end.max(b.end),
    }
}

/// A score within `SCORE_NOISE` of a whole hundredth, as that hundredth: the
/// sums of `config.rs`'s two-decimal values, free of `f32` rounding.
fn settled(score: f32) -> f32 {
    let hundredths = (score * 100.0).round();
    if (score * 100.0 - hundredths).abs() < SCORE_NOISE * 100.0 {
        hundredths / 100.0
    } else {
        score
    }
}

/// By start, then the kind order of D-47, then the first word.
fn spoken_order(a: &DetectedEvent, b: &DetectedEvent) -> Ordering {
    let rank = |kind: EventKind| match kind {
        EventKind::NumberReveal => 0u8,
        EventKind::ListReveal => 1,
        EventKind::FromTo => 2,
        EventKind::KeywordPop => 3,
    };
    a.span
        .start
        .cmp(&b.span.start)
        .then(rank(a.kind).cmp(&rank(b.kind)))
        .then(a.anchors.start.cmp(&b.anchors.start))
}

#[cfg(test)]
mod tests {
    use offcut_text::{RawWord, normalize_transcript};
    use offcut_types::{EventKind, EventParams, Unit, WordProsody};

    use super::*;

    /// `(text, start, end, confidence)`.
    type Spoken<'a> = (&'a str, u32, u32, f32);

    fn transcript(words: &[Spoken<'_>]) -> Transcript {
        let raw = words
            .iter()
            .map(|(text, start_ms, end_ms, confidence)| RawWord {
                text: (*text).to_owned(),
                start_ms: *start_ms,
                end_ms: *end_ms,
                confidence: *confidence,
            })
            .collect();
        normalize_transcript(raw, "test")
    }

    /// No word stands out: what V2 measures (D-22).
    fn flat(t: &Transcript) -> Prosody {
        let neutral = WordProsody {
            energy_z: 0.0,
            pitch_z: 0.0,
        };
        Prosody {
            per_word: vec![neutral; t.words.len()],
        }
    }

    fn events(words: &[Spoken<'_>]) -> Vec<DetectedEvent> {
        let t = transcript(words);
        detect(&t, &flat(&t), &BTreeMap::new(), &DetectorConfig::default())
    }

    #[test]
    fn ten_thousand_dollars_at_confidence_0_9_is_one_number_reveal() {
        let spoken = [
            ("we", 1_000, 1_200, 1.0),
            ("made", 1_250, 1_500, 1.0),
            ("$10,000", 1_600, 2_300, 0.9),
            ("there.", 2_400, 2_700, 1.0),
        ];
        let found = events(&spoken);
        assert_eq!(found.len(), 1);
        let event = &found[0];
        assert_eq!(event.kind, EventKind::NumberReveal);
        assert_eq!(event.confidence.get(), 0.9);
        assert!(event.enabled);
        assert_eq!(
            (event.span.start.get(), event.span.end.get()),
            (1_600, 2_300)
        );
        assert_eq!((event.anchors.start.get(), event.anchors.end.get()), (2, 3));
        assert_eq!(event.id, event_id(EventKind::NumberReveal, event.anchors));
        match &event.params {
            EventParams::NumberReveal { value, label } => {
                assert_eq!(value.display, "$10k");
                assert_eq!((value.value, &value.unit), (10_000.0, &Unit::Usd));
                // As of V3, the words after the number, up to the full stop (D-107).
                assert_eq!(label.as_deref(), Some("there"));
            }
            EventParams::ListReveal { .. }
            | EventParams::FromTo { .. }
            | EventParams::KeywordPop { .. } => panic!("{:?}", event.params),
        }

        // The amount as the recognizer writes it for the reference clip: two words.
        let split = events(&[
            ("about", 62_300, 62_600, 1.0),
            ("$12", 62_660, 63_100, 1.0),
            (",000", 63_100, 63_540, 1.0),
            ("in", 63_600, 63_700, 1.0),
        ]);
        assert_eq!(split.len(), 1);
        assert_eq!(
            (split[0].span.start.get(), split[0].span.end.get()),
            (62_660, 63_540)
        );
        assert_eq!(
            (split[0].anchors.start.get(), split[0].anchors.end.get()),
            (1, 3)
        );

        // As of V3, a number whose word was edited is read again from the new text
        // (D-109); a hidden one is not shown; an edit somewhere else changes nothing.
        let t = transcript(&spoken);
        let with = |word: u32, text: &str| {
            let edits = BTreeMap::from([(WordIdx::new(word), text.to_owned())]);
            detect(&t, &flat(&t), &edits, &DetectorConfig::default())
        };
        let edited = with(2, "$20,000");
        assert_eq!(edited.len(), 1);
        match &edited[0].params {
            EventParams::NumberReveal { value, .. } => {
                assert_eq!((value.value, value.display.as_str()), (20_000.0, "$20k"));
            }
            EventParams::ListReveal { .. }
            | EventParams::FromTo { .. }
            | EventParams::KeywordPop { .. } => panic!("{:?}", edited[0].params),
        }
        // The id is a hash of the kind and the anchors only (D-47), and an edit of
        // one word keeps the anchors: the id is the one the old event had. Not the
        // "new id" of v3implementation §9.9 (V3 changelog, open item 14).
        assert_eq!(edited[0].anchors, event.anchors);
        assert_eq!(edited[0].id, event.id);
        assert_eq!(with(2, ""), []);
        assert_eq!(with(1, "earned"), found);
    }

    #[test]
    fn ten_thousand_in_words_has_no_unit_and_is_no_event() {
        let found = events(&[
            ("about", 1_000, 1_300, 1.0),
            ("ten", 1_400, 1_600, 1.0),
            ("thousand", 1_650, 2_100, 1.0),
            // As of V3, "people" is a count noun, a unit (v3implementation D-101), so the
            // number is followed by words that are none.
            ("of", 2_200, 2_400, 1.0),
            ("them", 2_450, 2_700, 1.0),
        ]);
        assert_eq!(found, []);
        // A bare number in digits scores the same 0.70, and a small one 0.50.
        assert_eq!(events(&[("3000", 1_000, 1_500, 1.0)]), []);
        assert_eq!(events(&[("three", 1_000, 1_500, 1.0)]), []);
        // Nothing spoken, nothing found.
        assert_eq!(events(&[]), []);
    }

    #[test]
    fn forty_percent_is_an_event_at_confidence_1_and_none_at_0_99() {
        let sure = events(&[("40%", 1_000, 1_500, 1.0)]);
        assert_eq!(sure.len(), 1);
        assert_eq!(sure[0].confidence.get(), 0.80);
        assert_eq!(events(&[("40%", 1_000, 1_500, 0.99)]), []);
        // The least certain word of a number counts.
        let split = [("$12", 1_000, 1_300, 1.0), (",000", 1_300, 1_600, 0.7)];
        assert_eq!(events(&split), []);
    }

    #[test]
    fn of_two_amounts_600_ms_apart_only_the_earlier_one_is_kept() {
        let found = events(&[
            ("$5,000", 1_000, 1_300, 1.0),
            ("or", 1_400, 1_600, 1.0),
            ("$7,000", 1_900, 2_200, 1.0),
        ]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].span.start.get(), 1_000);

        // The first is shown until 1,300 + 1,400 = 2,700; the second from
        // 150 ms before its first word. They may not meet.
        let starts = |second: u32| -> Vec<u32> {
            events(&[
                ("$5,000", 1_000, 1_300, 1.0),
                ("$7,000", second, second + 300, 1.0),
            ])
            .iter()
            .map(|event| event.span.start.get())
            .collect()
        };
        assert_eq!(starts(2_850), [1_000]);
        assert_eq!(starts(2_851), [1_000, 2_851]);
    }

    #[test]
    fn two_amounts_5_s_apart_are_both_kept_in_time_order() {
        let found = events(&[
            ("$5,000", 1_000, 1_300, 1.0),
            ("and", 3_000, 3_200, 1.0),
            ("$7,000", 6_000, 6_300, 1.0),
        ]);
        let starts: Vec<u32> = found.iter().map(|event| event.span.start.get()).collect();
        assert_eq!(starts, [1_000, 6_000]);
        assert_ne!(found[0].id, found[1].id);
    }

    /// Numbers of every score, some certain and some not.
    fn mixed() -> Transcript {
        let texts = [
            "$5,000", "three", "40%", "$12", ",000", "900", "$8", "25%", "$90,000", "1,200",
        ];
        let words: Vec<Spoken<'_>> = (0u32..)
            .zip(texts)
            .map(|(i, text)| {
                let confidence = [1.0, 0.95, 0.85, 0.6][i as usize % 4];
                (text, i * 4_000, i * 4_000 + 500, confidence)
            })
            .collect();
        transcript(&words)
    }

    #[test]
    fn no_returned_event_is_below_the_threshold_of_its_kind() {
        let t = mixed();
        let cfg = DetectorConfig::default();
        let found = detect(&t, &flat(&t), &BTreeMap::new(), &cfg);
        assert!(!found.is_empty());
        assert!(found.len() < t.numbers.len());
        for event in &found {
            assert!(event.confidence.get() >= cfg.threshold_number, "{event:?}");
        }
        // A word said with emphasis lifts a number over the line, and only then.
        let said = [("70%", 1_000, 1_500, 0.9)];
        assert_eq!(events(&said), []);
        let t = transcript(&said);
        let loud = Prosody {
            per_word: vec![WordProsody {
                energy_z: 1.0,
                pitch_z: 0.0,
            }],
        };
        let found = detect(&t, &loud, &BTreeMap::new(), &cfg);
        assert_eq!(found.len(), 1);
        assert!(found[0].confidence.get() >= cfg.threshold_number);
    }

    #[test]
    fn the_same_inputs_twice_give_the_same_events_and_the_same_ids() {
        let t = mixed();
        let cfg = DetectorConfig::default();
        let first = detect(&t, &flat(&t), &BTreeMap::new(), &cfg);
        let second = detect(&t, &flat(&t), &BTreeMap::new(), &cfg);
        assert_eq!(first, second);
        let ids = |events: &[DetectedEvent]| events.iter().map(|e| e.id).collect::<Vec<_>>();
        assert_eq!(ids(&first), ids(&second));
        // A transcript built again from the same words gives them too.
        let again = mixed();
        assert_eq!(detect(&again, &flat(&again), &BTreeMap::new(), &cfg), first);
    }
}
