//! The same inputs give the same events (INV-7 for detection,
//! v3implementation §19.14): over every labeled file, over random
//! transcripts, and over a transcript built twice from the same words.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use offcut_detect::{DetectorConfig, detect, redetect_sentence};
use offcut_text::{RawWord, normalize_transcript};
use offcut_types::{
    DetectedEvent, EventKind, Prosody, SentenceIdx, Transcript, WordIdx, WordProsody,
};
use proptest::prelude::*;

// --- Timing synthesis (§17.5), the same rule as rules_table.rs. ------------

/// A word lasts `clamp(120 + 45 x letters, 150, 700)` ms; 80 ms between the
/// words of a sentence, 400 ms after a word ending in `.` `?` `!`; the first
/// word starts at 500 ms. `*word*` is emphasized: energy_z and pitch_z 2.0
/// (§23 item 27), every other word 0.0. Confidence 1.0 (TE-2).
fn synthesize(sentences: &[&str]) -> (Vec<RawWord>, Prosody) {
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

// --- The labeled files (§17.5). ---------------------------------------------

/// A labeled file's words as the recognizer would give them, its prosody and
/// its edits.
struct Labeled {
    id: String,
    raw: Vec<RawWord>,
    prosody: Prosody,
    edits: BTreeMap<WordIdx, String>,
}

/// Every file of `fixtures/labeled` with data, by name; a file whose data is
/// still null (`a02`) is skipped.
fn labeled() -> Vec<Labeled> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/labeled");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    let mut files = Vec::new();
    for path in paths {
        let file: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let (raw, prosody) = match (file["sentences"].as_array(), file["words"].as_array()) {
            (Some(sentences), _) => {
                let texts: Vec<&str> = sentences.iter().map(|s| s.as_str().unwrap()).collect();
                synthesize(&texts)
            }
            (None, Some(words)) => (
                words
                    .iter()
                    .map(|w| RawWord {
                        text: w["text"].as_str().unwrap().to_owned(),
                        start_ms: u32::try_from(w["startMs"].as_u64().unwrap()).unwrap(),
                        end_ms: u32::try_from(w["endMs"].as_u64().unwrap()).unwrap(),
                        confidence: w["confidence"].as_f64().unwrap() as f32,
                    })
                    .collect(),
                Prosody {
                    per_word: file["prosody"]["per_word"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|z| WordProsody {
                            energy_z: z["energy_z"].as_f64().unwrap() as f32,
                            pitch_z: z["pitch_z"].as_f64().unwrap() as f32,
                        })
                        .collect(),
                },
            ),
            (None, None) => continue,
        };
        let edits = file["edits"]
            .as_object()
            .map(|edits| {
                edits
                    .iter()
                    .map(|(word, text)| {
                        (
                            WordIdx::new(word.parse().unwrap()),
                            text.as_str().unwrap().to_owned(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        files.push(Labeled {
            id: file["id"].as_str().unwrap().to_owned(),
            raw,
            prosody,
            edits,
        });
    }
    files
}

#[test]
fn every_labeled_file_detected_twice_gives_the_same_events() {
    let cfg = DetectorConfig::default();
    let files = labeled();
    // The 31 files of the set, `a02` skipped until its data is filled.
    assert!(files.len() >= 30, "{} files read", files.len());
    let mut events = 0;
    for file in &files {
        let t = normalize_transcript(file.raw.clone(), "labeled");
        let first = detect(&t, &file.prosody, &file.edits, &cfg);
        let second = detect(&t, &file.prosody, &file.edits, &cfg);
        // Including the order and the ids.
        assert_eq!(first, second, "{}", file.id);
        events += first.len();
    }
    assert!(events > 0);
}

#[test]
fn a_transcript_built_twice_from_the_same_words_gives_the_same_events() {
    let cfg = DetectorConfig::default();
    for file in labeled() {
        let once = normalize_transcript(file.raw.clone(), "labeled");
        let twice = normalize_transcript(file.raw, "labeled");
        assert_eq!(once, twice, "{}", file.id);
        assert_eq!(
            detect(&once, &file.prosody, &file.edits, &cfg),
            detect(&twice, &file.prosody, &file.edits, &cfg),
            "{}",
            file.id
        );
    }
}

// --- Random transcripts. ----------------------------------------------------

/// Number words, list nouns, ordinals, fillers, stop words, emphasis words
/// and digits (G 8.2), with the units, traps and announcers that make them
/// meet.
const VOCABULARY: &[&str] = &[
    // Number words.
    "one",
    "two",
    "three",
    "four",
    "ten",
    "twelve",
    "ninety",
    "hundred",
    "thousand",
    "million",
    // List nouns.
    "reasons",
    "tips",
    "lessons",
    "steps",
    "ways",
    "mistakes",
    // Ordinals and markers.
    "First",
    "first",
    "Second",
    "second",
    "Third",
    "third",
    "Fourth",
    "Finally",
    "1,",
    "2,",
    "3,",
    // Fillers.
    "um",
    "uh",
    "so",
    "and",
    "okay",
    "like",
    "well",
    // Stop words and frames.
    "There",
    "are",
    "Here",
    "the",
    "from",
    "to",
    "went",
    "about",
    "that",
    "is",
    "it",
    "my",
    "dropped",
    "raised",
    // Emphasis words.
    "brutal",
    "insane",
    "literally",
    "crazy",
    "huge",
    "workflow",
    "perfect",
    // Digits, amounts and their units.
    "3000",
    "$12",
    ",000",
    "$5,000",
    "40%",
    "25%",
    "90",
    "10",
    "3",
    "$2k",
    "hours",
    "minutes",
    "dollars",
    "users",
    "people",
    // Traps: years, versions, clock times, phone-like runs.
    "2019",
    "version",
    "3.12",
    "pm",
    "5551234",
];

/// Runs of words single words would almost never line up into: without them
/// no random transcript holds a list and few a FromTo. One that starts with a
/// capital starts a sentence.
const PHRASES: &[&str] = &[
    "There are three reasons it worked. First, the captions write themselves. \
     Second, the numbers animate automatically. Third, everything runs on my laptop.",
    "Here are 4 tips. 1, sleep early. 2, eat well. 3, ship daily. 4, rest often.",
    "First, open the app. Second, drop a clip. Third, read the result.",
    "There are three reasons it worked.",
    "Here are 4 tips.",
    "Here are two lessons.",
    "First, the captions write themselves.",
    "Second, the numbers animate automatically.",
    "Third, everything runs on my laptop.",
    "1, sleep early.",
    "2, eat well.",
    "3, ship daily.",
    "went from 90 minutes to 10 minutes",
    "from $2k to $20k",
    "dropped 40% to 25%",
    "That is literally brutal.",
    "Meet me at 5 pm.",
    "Upgrade to Python 3.12 now.",
];

/// The text of a piece: a word of the vocabulary, or a phrase after them.
fn piece(index: usize) -> &'static str {
    VOCABULARY
        .get(index)
        .or_else(|| PHRASES.get(index - VOCABULARY.len()))
        .unwrap()
}

/// A piece: its text, what follows it, how long each of its words lasts, the
/// pause after it, how sure the recognizer was, and how it was said.
type Spoken = (usize, u8, u32, u32, f32, f32, f32);

fn spoken() -> impl Strategy<Value = Spoken> {
    (
        prop_oneof![
            3 => 0..VOCABULARY.len(),
            1 => VOCABULARY.len()..VOCABULARY.len() + PHRASES.len(),
        ],
        0u8..8,
        150u32..=700,
        // Mostly the 80 ms of a sentence; then the pauses that end sentences,
        // and gaps that reach past a list's 45 s.
        prop_oneof![
            6 => Just(80u32),
            2 => 300u32..3_000,
            1 => 3_000u32..30_000,
        ],
        prop_oneof![Just(1.0f32), Just(0.95), Just(0.85), Just(0.6)],
        -3.0f32..=3.0,
        -3.0f32..=3.0,
    )
}

/// The words of the pieces, at most `MAX_WORDS`: 80 ms between the words of
/// a piece, the piece's pause after its last.
fn transcript(pieces: &[Spoken]) -> (Vec<RawWord>, Prosody) {
    const MAX_WORDS: usize = 200;
    let mut at = 500u32;
    let mut raw: Vec<RawWord> = Vec::new();
    let mut per_word = Vec::new();
    for (index, after, length, gap, confidence, energy_z, pitch_z) in pieces {
        let text = piece(*index);
        let opens = text.starts_with(char::is_uppercase);
        if let Some(before) = raw.last_mut().filter(|w| opens && !w.text.ends_with('.')) {
            before.text.push('.');
        }
        let stop = match after {
            _ if text.ends_with(['.', ',']) => "",
            0 => ".",
            1 => ",",
            _ => "",
        };
        let words: Vec<&str> = text.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            let last = i + 1 == words.len();
            raw.push(RawWord {
                text: if last {
                    format!("{word}{stop}")
                } else {
                    (*word).to_owned()
                },
                start_ms: at,
                end_ms: at + length,
                confidence: *confidence,
            });
            per_word.push(WordProsody {
                energy_z: *energy_z,
                pitch_z: *pitch_z,
            });
            at += length + if last { *gap } else { 80 };
        }
    }
    raw.truncate(MAX_WORDS);
    per_word.truncate(MAX_WORDS);
    (raw, Prosody { per_word })
}

/// The kind order of D-47, which `detect` sorts by after the start.
fn rank(kind: EventKind) -> u8 {
    match kind {
        EventKind::NumberReveal => 0,
        EventKind::ListReveal => 1,
        EventKind::FromTo => 2,
        EventKind::KeywordPop => 3,
    }
}

/// An overlay's display window, from `overlay_lead_ms` before its first word
/// to its hold after its last; none for a KeywordPop.
fn window(e: &DetectedEvent, cfg: &DetectorConfig) -> Option<(u32, u32)> {
    (e.kind != EventKind::KeywordPop).then(|| {
        (
            e.span.start.get().saturating_sub(cfg.overlay_lead_ms),
            e.span.end.get().saturating_add(cfg.hold_ms(e.kind)),
        )
    })
}

/// The rules every output holds (§19.14).
fn check(t: &Transcript, events: &[DetectedEvent], cfg: &DetectorConfig) {
    let words = u32::try_from(t.words.len()).unwrap();
    for e in events {
        assert!(
            e.anchors.start < e.anchors.end && e.anchors.end.get() <= words,
            "anchors out of 0..{words}: {e:?}"
        );
        assert!(e.span.start <= e.span.end, "{e:?}");
        // INV-6.
        assert!(e.confidence.get() >= cfg.threshold(e.kind), "{e:?}");
    }
    for pair in events.windows(2) {
        let key = |e: &DetectedEvent| (e.span.start, rank(e.kind), e.anchors.start);
        assert!(key(&pair[0]) <= key(&pair[1]), "unsorted: {pair:?}");
    }
    for (i, a) in events.iter().enumerate() {
        for b in &events[i + 1..] {
            let shared = a.anchors.start < b.anchors.end && b.anchors.start < a.anchors.end;
            assert!(!shared, "shared words: {a:?} {b:?}");
            if let (Some((a0, a1)), Some((b0, b1))) = (window(a, cfg), window(b, cfg)) {
                assert!(!(a0 <= b1 && b0 <= a1), "windows meet: {a:?} {b:?}");
            }
        }
    }
    let mut pops = BTreeMap::new();
    for e in events.iter().filter(|e| e.kind == EventKind::KeywordPop) {
        let sentence = t
            .sentences
            .iter()
            .position(|s| s.words.start <= e.anchors.start && e.anchors.start < s.words.end);
        *pops.entry(sentence).or_insert(0u32) += 1;
    }
    assert!(
        pops.values().all(|n| *n <= 1),
        "pops per sentence: {pops:?}"
    );
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    #[test]
    fn random_transcripts_give_the_same_events_twice_and_keep_the_rules(
        words in prop::collection::vec(spoken(), 0..=200),
        edits in prop::collection::vec((0u32..220, prop::option::of(0..VOCABULARY.len())), 0..=3),
        sentence in 0u32..64,
    ) {
        let cfg = DetectorConfig::default();
        let (raw, p) = transcript(&words);
        let t = normalize_transcript(raw, "prop");
        // An edit replaces a word by another of the vocabulary, or hides it;
        // some fall past the last word.
        let edits: BTreeMap<WordIdx, String> = edits
            .into_iter()
            .map(|(word, text)| {
                (WordIdx::new(word), text.map_or_else(String::new, |w| VOCABULARY[w].to_owned()))
            })
            .collect();
        let first = detect(&t, &p, &edits, &cfg);
        let second = detect(&t, &p, &edits, &cfg);
        prop_assert_eq!(&first, &second);
        check(&t, &first, &cfg);
        // Re-detection with nothing changed since `prev` finds `prev` again,
        // ids included (D-86, D-47); an index past the last sentence returns it.
        let again = redetect_sentence(&t, &p, &edits, SentenceIdx::new(sentence), &first, &cfg);
        prop_assert_eq!(&again, &first);
    }
}
