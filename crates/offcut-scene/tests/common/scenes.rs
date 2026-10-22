//! Six scenes of the labeled set, for `parity.rs` and `scene_cache.rs`
//! (D-172, D-173): six transcripts of `fixtures/labeled/`, written ones timed
//! and stressed as `precision_recall.rs` does (v3implementation §17.5), ASR
//! ones as recorded, each detected and validated as the pipeline does, with
//! the file's own edits. Each takes other user choices: the three styles, the
//! two ends of the band and its snap points, numbers highlighted or not, a
//! word edit and a visual turned off.

use std::collections::BTreeMap;
use std::path::PathBuf;

use offcut_detect::{DetectorConfig, detect};
use offcut_plan::{PlanConfig, ValidPlan, from_rules, validate};
use offcut_scene::{FontSet, Scene, SceneInput, Theme, build_scene};
use offcut_text::{RawWord, normalize_transcript};
use offcut_types::{
    BitsPerSec, CaptionPlacement, ClipInfo, DurMs, EditState, ExportProfile, Hz, Orientation,
    ProfileKind, Prosody, Px, Rotation, StyleId, TimeMap, Transcript, WordIdx, WordProsody,
};
use serde_json::Value;

/// A scene's input, but for the output width.
pub struct Case {
    pub name: &'static str,
    transcript: Transcript,
    plan: ValidPlan,
    edit: EditState,
    length: u32,
}

/// A written file's words, timed, and their stress: a word in `*…*` is said
/// with emphasis. Also where the next word would start: the clip's length.
fn synthesize(sentences: &[String]) -> (Vec<RawWord>, Vec<WordProsody>, u32) {
    let (mut raw, mut stress, mut at) = (Vec::new(), Vec::new(), 500u32);
    for piece in sentences.iter().flat_map(|s| s.split_whitespace()) {
        let stressed =
            piece.starts_with('*') && piece.trim_end_matches(['.', ',', '?', '!']).ends_with('*');
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
        stress.push(WordProsody {
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
    (raw, stress, at)
}

/// The labeled file `name`, detected and validated, with `choose` applied to
/// the user's choices.
fn case(name: &'static str, choose: impl FnOnce(&mut EditState, &ValidPlan)) -> Case {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/labeled")
        .join(format!("{name}.json"));
    let json: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let (raw, per_word, length) = match json["source"].as_str() {
        Some("written") => {
            synthesize(&serde_json::from_value::<Vec<String>>(json["sentences"].clone()).unwrap())
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
            (raw, prosody.per_word, length)
        }
        other => panic!("{name}: source {other:?}"),
    };
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
    let transcript = normalize_transcript(raw, "labeled");
    let prosody = Prosody { per_word };
    let events = detect(&transcript, &prosody, &edits, &DetectorConfig::default());
    let rules = from_rules(&events, &transcript);
    let (plan, _) = validate(rules, &transcript, &edits, &PlanConfig::default()).unwrap();
    let mut edit = EditState {
        word_edits: edits,
        ..EditState::default()
    };
    choose(&mut edit, &plan);
    Case {
        name,
        transcript,
        plan,
        edit,
        length,
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

/// The scene of `case` for an output `width` wide, 9:16.
pub fn build(case: &Case, width: u32) -> Scene {
    build_scene(SceneInput {
        transcript: &case.transcript,
        plan: &case.plan,
        edit: &case.edit,
        clip: &ClipInfo {
            duration: DurMs::new(case.length),
            display_width: Px::new(1080),
            display_height: Px::new(1920),
            rotation: Rotation::R0,
            orientation: Orientation::Portrait,
            is_vfr: false,
            video_codec_string: "avc1.640028".to_owned(),
            audio_codec_string: "mp4a.40.2".to_owned(),
            audio_sample_rate: Hz::new(48_000),
            audio_channels: 2,
        },
        profile: &ExportProfile {
            kind: ProfileKind::Creator,
            width: Px::new(width),
            height: Px::new(width * 16 / 9),
            watermark: false,
            video_bitrate: BitsPerSec::new(8_000_000),
            audio_bitrate: BitsPerSec::new(160_000),
        },
        time_map: &TimeMap::identity(DurMs::new(case.length)),
        theme: &Theme::default_for(case.edit.style),
        fonts: &FontSet::embedded(),
    })
    .unwrap()
}
