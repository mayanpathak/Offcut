//! No rules plan of the labeled set loses an item (v4implementation D-165,
//! INV-24): for every file of `fixtures/labeled/`, the detector's events
//! become a plan (`from_rules`) and the plan validates whole, with the
//! detector's spans and, for every FromTo, one mark later than its first word
//! (section 8.7). Prints "31 files, 0 items dropped".

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use offcut_detect::{DetectorConfig, detect};
use offcut_plan::{PlanConfig, from_rules, validate};
use offcut_text::{RawWord, normalize_transcript};
use offcut_types::{EventKind, Prosody, Transcript, WordIdx, WordProsody};
use serde_json::Value;

/// A labeled file as `precision_recall.rs` of `offcut-detect` reads it: a
/// written file timed by its `synthesize`, an ASR file with its own words and
/// prosody, and the file's word edits.
struct Labeled {
    name: String,
    t: Transcript,
    p: Prosody,
    edits: BTreeMap<WordIdx, String>,
}

/// `synthesize` of `precision_recall.rs` (v3implementation §17.5).
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

fn labeled() -> Vec<Labeled> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/labeled");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("fixtures/labeled")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| {
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{name}: {e}"));
            let json: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
            let (raw, p) = match json["source"].as_str() {
                Some("written") => {
                    let sentences: Vec<String> = serde_json::from_value(json["sentences"].clone())
                        .unwrap_or_else(|e| panic!("{name}: sentences: {e}"));
                    synthesize(&sentences)
                }
                Some("asr") => {
                    let words = json["words"]
                        .as_array()
                        .unwrap_or_else(|| panic!("{name}: words"));
                    let raw = words
                        .iter()
                        .map(|w| RawWord {
                            text: w["text"].as_str().unwrap().to_owned(),
                            start_ms: u32::try_from(w["startMs"].as_u64().unwrap()).unwrap(),
                            end_ms: u32::try_from(w["endMs"].as_u64().unwrap()).unwrap(),
                            confidence: w["confidence"].as_f64().unwrap() as f32,
                        })
                        .collect();
                    let p: Prosody = serde_json::from_value(json["prosody"].clone())
                        .unwrap_or_else(|e| panic!("{name}: prosody: {e}"));
                    (raw, p)
                }
                other => panic!("{name}: source {other:?}"),
            };
            let edits = json["edits"]
                .as_object()
                .map(|map| {
                    map.iter()
                        .map(|(word, text)| {
                            let word: u32 =
                                word.parse().unwrap_or_else(|_| panic!("{name}: {word}"));
                            (WordIdx::new(word), text.as_str().unwrap().to_owned())
                        })
                        .collect()
                })
                .unwrap_or_default();
            Labeled {
                t: normalize_transcript(raw, "labeled"),
                name,
                p,
                edits,
            }
        })
        .collect()
}

#[test]
fn no_rules_plan_of_the_labeled_set_loses_an_item() {
    let files = labeled();
    assert_eq!(files.len(), 31, "the labeled set");
    let (mut items, mut dropped, mut from_tos) = (0, 0, 0);
    for file in &files {
        let events = detect(&file.t, &file.p, &file.edits, &DetectorConfig::default());
        let plan = from_rules(&events, &file.t);
        let (valid, drops) = validate(plan, &file.t, &file.edits, &PlanConfig::default())
            .unwrap_or_else(|e| panic!("{}: {e}", file.name));
        for drop in &drops {
            println!("{}: dropped {:?}", file.name, drop);
        }
        dropped += drops.len();
        items += valid.items().len();
        // The detector's spans, ids and params, in its time order.
        let mut by_id: BTreeMap<u64, _> = events.iter().map(|e| (e.id.get(), e)).collect();
        for item in valid.items() {
            let event = by_id
                .remove(&item.id.get())
                .unwrap_or_else(|| panic!("{}: an item no event made", file.name));
            assert_eq!(item.span, event.span, "{}: {:?}", file.name, item.kind);
            assert_eq!(item.params, event.params, "{}", file.name);
            assert_eq!(item.anchors, event.anchors, "{}", file.name);
            if item.kind == EventKind::FromTo {
                from_tos += 1;
                assert_eq!(item.marks.len(), 1, "{}: a FromTo has one mark", file.name);
                assert!(
                    item.marks[0] > item.span.start,
                    "{}: the mark is after the first word",
                    file.name
                );
                assert!(item.marks[0] <= item.span.end, "{}", file.name);
            } else {
                assert!(item.marks.is_empty(), "{}", file.name);
            }
        }
    }
    println!("{} files, {dropped} items dropped", files.len());
    println!("{items} items kept, {from_tos} FromTo marked");
    assert_eq!(dropped, 0, "D-165: a rules plan loses no item");
    assert!(items > 0 && from_tos > 0);
}
