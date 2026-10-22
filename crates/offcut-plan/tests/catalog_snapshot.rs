//! The three texts a model gets, pinned (v4implementation §7.3, D-123,
//! INV-23): the system prompt, the output schema and the user message on the
//! `a01` labeled transcript with zero prosody. Each snapshot's name carries
//! `CATALOG_VERSION` (`system_prompt@v1`): a change to either generated
//! output fails against the old file, and a new catalog version needs new
//! files. A changed snapshot is read before it is accepted, and named in the
//! changelog (§18).

#![cfg(feature = "prompt")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::path::PathBuf;

use offcut_plan::CATALOG_VERSION;
use offcut_plan::prompt::{PromptInput, output_schema, system_prompt, user_message};
use offcut_text::{RawWord, normalize_transcript};
use offcut_types::{DurMs, WordProsody};

fn name(output: &str) -> String {
    format!("{output}@v{CATALOG_VERSION}")
}

#[test]
fn the_system_prompt() {
    insta::assert_snapshot!(name("system_prompt"), system_prompt());
}

#[test]
fn the_output_schema() {
    insta::assert_json_snapshot!(name("output_schema"), output_schema());
}

#[test]
fn the_user_message_on_a01() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/labeled/a01-scriptA.json");
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("a01")).expect("a01 parses");
    let sentences: Vec<String> = serde_json::from_value(json["sentences"].clone()).unwrap();
    // Timed as `offcut-detect/tests/precision_recall.rs` times a written file.
    let mut raw = Vec::new();
    let mut at = 500u32;
    for text in &sentences {
        for piece in text.split_whitespace() {
            let word = piece.replace('*', "");
            let letters = word.chars().filter(|c| c.is_alphanumeric()).count() as u32;
            let length = (120 + 45 * letters).clamp(150, 700);
            raw.push(RawWord {
                text: word.clone(),
                start_ms: at,
                end_ms: at + length,
                confidence: 1.0,
            });
            at += length
                + if word.ends_with(['.', '?', '!']) {
                    400
                } else {
                    80
                };
        }
    }
    let t = normalize_transcript(raw, "labeled");
    let prosody = vec![
        WordProsody {
            energy_z: 0.0,
            pitch_z: 0.0
        };
        t.words.len()
    ];
    let input = PromptInput {
        duration: DurMs::new(at),
        words: &t.words,
        prosody: &prosody,
        sentences: &t.sentences,
    };
    insta::assert_snapshot!(name("user_message"), user_message(&input));
}
