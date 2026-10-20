//! `redetect_sentence` on Script A as recognized (v3implementation §19.13):
//! the sentences of `fixtures/labeled/a01-scriptA.json`, timed as
//! `rules_table.rs` times them (§17.5). Word and sentence numbers count from
//! 0; `fixtures/speech/README.md` counts its sentences from 1, so its sentence
//! 3 is sentence 2 here, its 8 is 7 and its 9-11 are 8-10.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::collections::BTreeMap;
use std::time::Instant;

use offcut_detect::{DetectorConfig, detect, event_id, redetect_sentence};
use offcut_text::{RawWord, normalize_transcript};
use offcut_types::{
    DetectedEvent, EventKind, EventParams, Prosody, SentenceIdx, Transcript, WordIdx, WordProsody,
    WordRange,
};

// --- Timing synthesis (§17.5), the same rule as rules_table.rs. ------------

/// A sentence, and the earliest time its first word may start.
type Line<'a> = (&'a str, u32);

/// A word lasts `clamp(120 + 45 x letters, 150, 700)` ms; 80 ms between the
/// words of a sentence, 400 ms after a word ending in `.` `?` `!`; the first
/// word starts at 500 ms. `*word*` is emphasized: energy_z and pitch_z 2.0
/// (§23 item 27), every other word 0.0. Confidence 1.0 (TE-2).
fn synthesize(lines: &[Line<'_>]) -> (Transcript, Prosody) {
    let mut raw = Vec::new();
    let mut per_word = Vec::new();
    let mut at = 500u32;
    for (text, earliest) in lines {
        at = at.max(*earliest);
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
    (normalize_transcript(raw, "labeled"), Prosody { per_word })
}

// --- Script A. --------------------------------------------------------------

/// Script A as recognized, its events, and the detector's defaults.
struct ScriptA {
    t: Transcript,
    p: Prosody,
    prev: Vec<DetectedEvent>,
    cfg: DetectorConfig,
}

fn script_a() -> ScriptA {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/labeled/a01-scriptA.json"
    );
    let file: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let sentences: Vec<&str> = file["sentences"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap())
        .collect();
    let lines: Vec<Line<'_>> = sentences.iter().map(|s| (*s, 0)).collect();
    let (t, p) = synthesize(&lines);
    // The README's 157 words in 17 sentences, numbered as it numbers them.
    assert_eq!((t.words.len(), t.sentences.len()), (157, 17));
    assert_eq!(t.words[23].text, "3000");
    assert_eq!(t.words[79].text, "First");
    let cfg = DetectorConfig::default();
    let prev = detect(&t, &p, &BTreeMap::new(), &cfg);
    // The README's events of V3 but the KeywordPop, which needs measured prosody.
    let kinds: Vec<(EventKind, u32)> = prev
        .iter()
        .map(|e| (e.kind, e.anchors.start.get()))
        .collect();
    assert_eq!(
        kinds,
        [
            (EventKind::NumberReveal, 23),
            (EventKind::FromTo, 55),
            (EventKind::ListReveal, 75),
            (EventKind::NumberReveal, 132),
        ]
    );
    ScriptA { t, p, prev, cfg }
}

fn edits(pairs: &[(u32, &str)]) -> BTreeMap<WordIdx, String> {
    pairs
        .iter()
        .map(|(word, text)| (WordIdx::new(*word), (*text).to_owned()))
        .collect()
}

fn sentence(s: u32) -> SentenceIdx {
    SentenceIdx::new(s)
}

/// `events` without the one whose first word is `word`.
fn without(events: &[DetectedEvent], word: u32) -> Vec<DetectedEvent> {
    events
        .iter()
        .filter(|e| e.anchors.start.get() != word)
        .cloned()
        .collect()
}

// --- The seven cases. -------------------------------------------------------

#[test]
fn without_an_edit_every_sentence_gives_prev_again() {
    let a = script_a();
    for s in 0..17 {
        let again = redetect_sentence(&a.t, &a.p, &BTreeMap::new(), sentence(s), &a.prev, &a.cfg);
        // Ids, order, everything.
        assert_eq!(again, a.prev, "sentence {s}");
    }
}

#[test]
fn word_23_edited_to_4000_keeps_its_id_and_nothing_else_moves() {
    let a = script_a();
    let edited = edits(&[(23, "4000")]);
    let after = redetect_sentence(&a.t, &a.p, &edited, sentence(2), &a.prev, &a.cfg);
    assert_eq!(after.len(), a.prev.len());
    let number = after[0].clone();
    assert_eq!(number.kind, EventKind::NumberReveal);
    match &number.params {
        EventParams::NumberReveal { value, .. } => {
            assert_eq!(
                (value.value, value.display.as_str()),
                (4_000.0, "4,000 hours")
            );
        }
        other @ (EventParams::ListReveal { .. }
        | EventParams::FromTo { .. }
        | EventParams::KeywordPop { .. }) => panic!("{other:?}"),
    }
    // The same kind and anchors, the same id (D-47; open item 14, decided
    // 2026-10-18): the user's toggle carries over.
    assert_eq!(number.anchors, a.prev[0].anchors);
    assert_eq!(number.id, a.prev[0].id);
    assert_eq!(after[1..], a.prev[1..]);
    // What `detect` finds on the whole edited text.
    assert_eq!(after, detect(&a.t, &a.p, &edited, &a.cfg));
}

#[test]
fn word_79_hidden_takes_the_list_away_and_nothing_else() {
    let a = script_a();
    let hidden = edits(&[(79, "")]);
    let after = redetect_sentence(&a.t, &a.p, &hidden, sentence(8), &a.prev, &a.cfg);
    // "three reasons" with two markers is a partial list (TS §17.6).
    assert!(after.iter().all(|e| e.kind != EventKind::ListReveal));
    assert_eq!(after, without(&a.prev, 75));
    assert_eq!(after, detect(&a.t, &a.p, &hidden, &a.cfg));
}

#[test]
fn an_edit_of_the_announcer_reaches_the_markers_of_sentences_8_to_10() {
    let a = script_a();
    let list = a.prev[2].clone();
    assert_eq!(list.anchors.start.get(), 75);
    // "There are four reasons" with three markers: no list.
    let four = edits(&[(75, "four")]);
    let no_list = redetect_sentence(&a.t, &a.p, &four, sentence(7), &a.prev, &a.cfg);
    assert_eq!(no_list, without(&a.prev, 75));

    // Back to "three": no event of `prev` touches sentence 7 now, so only the
    // 45 s of D-86 bring sentences 8-10, and their markers, into the scope.
    let back = redetect_sentence(&a.t, &a.p, &BTreeMap::new(), sentence(7), &no_list, &a.cfg);
    assert_eq!(back, a.prev);
    assert_eq!(back[2].id, event_id(EventKind::ListReveal, list.anchors));
    match &back[2].params {
        EventParams::ListReveal {
            count,
            header,
            items,
        } => {
            assert_eq!((*count, header.as_str(), items.len()), (3, "3 REASONS", 3));
        }
        other @ (EventParams::NumberReveal { .. }
        | EventParams::FromTo { .. }
        | EventParams::KeywordPop { .. }) => panic!("{other:?}"),
    }
}

/// §19.13 puts the new amount in sentence 1, beside `3,000 hours`. A sentence
/// before an event brings it into the scope (it starts within 45 s after,
/// D-86), so that event is detected again, not fixed; and the first sentence
/// after it starts 35 ms too late for the two windows to meet. The case is
/// played on the FromTo of sentence 5 and an amount in sentence 6 instead
/// (V3 changelog, open item 18).
#[test]
fn a_new_amount_meeting_the_window_of_a_fixed_event_is_dropped() {
    let a = script_a();
    let from_to = a.prev[1].clone();
    assert_eq!(from_to.kind, EventKind::FromTo);
    let shown_until = from_to.span.end.get() + a.cfg.hold_ms(EventKind::FromTo);
    let s6 = a.t.sentences[6].clone();
    assert_eq!(s6.words.start.get(), 60);
    // "That" becomes "$5,000": its window opens 150 ms before the word, inside
    // the FromTo's, which is outside the scope of sentence 6 and so fixed.
    assert!(s6.start_ms.get() - a.cfg.overlay_lead_ms <= shown_until);
    let amount = edits(&[(60, "$5,000")]);
    let after = redetect_sentence(&a.t, &a.p, &amount, sentence(6), &a.prev, &a.cfg);
    assert_eq!(after, a.prev);

    // Without the FromTo in `prev` nothing is fixed there, and the amount is an
    // event: it was the fixed event that took its place.
    let alone = redetect_sentence(
        &a.t,
        &a.p,
        &amount,
        sentence(6),
        &without(&a.prev, 55),
        &a.cfg,
    );
    let new = WordRange {
        start: WordIdx::new(60),
        end: WordIdx::new(61),
    };
    assert!(
        alone
            .iter()
            .any(|e| e.kind == EventKind::NumberReveal && e.anchors == new),
        "{alone:?}"
    );
}

#[test]
fn a_sentence_out_of_range_returns_prev_unchanged() {
    let a = script_a();
    let edited = edits(&[(23, "4000"), (79, "")]);
    // Even a `prev` that `detect` would not give is returned as it is.
    let partial = without(&a.prev, 23);
    for s in [17, 18, u32::MAX] {
        assert_eq!(
            redetect_sentence(&a.t, &a.p, &edited, sentence(s), &partial, &a.cfg),
            partial
        );
    }
    let empty = Transcript {
        words: Vec::new(),
        sentences: Vec::new(),
        numbers: Vec::new(),
        model_version: "labeled".to_owned(),
    };
    let none = Prosody {
        per_word: Vec::new(),
    };
    assert_eq!(
        redetect_sentence(&empty, &none, &edited, sentence(0), &a.prev, &a.cfg),
        a.prev
    );
}

/// The one clock of this crate's tests: budget 50 ms per call for 150 words
/// on R1 (assumption M1.5), printed, not asserted. Read it from a release
/// build. The crate itself reads no clock (TS §7).
#[test]
#[allow(clippy::disallowed_methods)]
fn the_time_of_one_call_is_printed() {
    const CALLS: u32 = 100;
    let a = script_a();
    let edited = edits(&[(23, "4000")]);
    let started = Instant::now();
    for call in 0..CALLS {
        let after = redetect_sentence(&a.t, &a.p, &edited, sentence(call % 17), &a.prev, &a.cfg);
        assert_eq!(after.len(), a.prev.len());
    }
    let per_call = started.elapsed().as_micros() / u128::from(CALLS);
    let build = if cfg!(debug_assertions) {
        "debug build, not the measurement"
    } else {
        "release"
    };
    println!(
        "redetect_sentence: {per_call} us per call over {CALLS} calls, Script A ({} words), {build}",
        a.t.words.len()
    );
}
