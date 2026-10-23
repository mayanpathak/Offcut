//! The labeled set as scenes (D-158): any file of `fixtures/labeled/`, and
//! six of them with other user choices, for `parity.rs`, `scene_cache.rs`,
//! `determinism.rs` and `layout_safe_area.rs` (D-172, D-173, INV-7). Each
//! takes other choices: the three styles, the two ends of the band and its
//! snap points, numbers highlighted or not, a word edit and a visual turned
//! off. Needs `scenes.rs` as the module `scenes`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use offcut_plan::ValidPlan;
use offcut_text::RawWord;
use offcut_types::{CaptionPlacement, EditState, Prosody, StyleId, WordIdx};
use serde_json::Value;

use crate::scenes::{Case, detected, written};

/// The labeled file `name`, detected and validated, with `choose` applied to
/// the user's choices.
pub fn case(name: &str, choose: impl FnOnce(&mut EditState, &ValidPlan)) -> Case {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/labeled")
        .join(format!("{name}.json"));
    let json: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let edits: BTreeMap<WordIdx, String> = json["edits"]
        .as_object()
        .map(|map| {
            map.iter()
                .map(|(word, text)| {
                    let word = WordIdx::new(word.parse().unwrap());
                    (word, text.as_str().unwrap().to_owned())
                })
                .collect()
        })
        .unwrap_or_default();
    match json["source"].as_str() {
        Some("written") => {
            let sentences: Vec<String> = serde_json::from_value(json["sentences"].clone()).unwrap();
            let sentences: Vec<&str> = sentences.iter().map(String::as_str).collect();
            written(name, &sentences, edits, choose)
        }
        Some("asr") => {
            let raw: Vec<RawWord> = json["words"]
                .as_array()
                .unwrap()
                .iter()
                .map(|w| RawWord {
                    text: w["text"].as_str().unwrap().to_owned(),
                    start_ms: u32::try_from(w["startMs"].as_u64().unwrap()).unwrap(),
                    end_ms: u32::try_from(w["endMs"].as_u64().unwrap()).unwrap(),
                    confidence: w["confidence"].as_f64().unwrap() as f32,
                })
                .collect();
            let prosody: Prosody = serde_json::from_value(json["prosody"].clone()).unwrap();
            let length = raw.last().map_or(0, |w| w.end_ms) + 1_000;
            detected(name, (raw, prosody.per_word, length), edits, choose)
        }
        other => panic!("{name}: source {other:?}"),
    }
}

fn at(p: f32) -> CaptionPlacement {
    CaptionPlacement::new(p).unwrap()
}

/// The six cases.
pub fn six() -> Vec<Case> {
    vec![
        case("a01-scriptA", |_, _| {}),
        case("a02-v3testclip", |edit, _| {
            edit.style = StyleId::Bold;
            edit.caption_position = at(0.15);
        }),
        case("w01-saas-pricing", |edit, _| {
            edit.style = StyleId::Tech;
            edit.caption_position = at(0.55);
        }),
        case("w16-open-source", |edit, _| {
            edit.caption_position = at(0.0);
            edit.highlight_numbers = false;
        }),
        case("w20-battery-life", |edit, _| {
            edit.style = StyleId::Bold;
            edit.word_edits.insert(WordIdx::new(3), "$500".to_owned());
        }),
        case("w28-mistakes", |edit, plan| {
            edit.caption_position = at(0.55);
            if let Some(item) = plan.items().first() {
                edit.event_overrides.insert(item.id, false);
            }
        }),
    ]
}
