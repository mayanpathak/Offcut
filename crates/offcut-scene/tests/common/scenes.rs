//! Scene inputs as the pipeline makes them: written sentences timed and
//! stressed as `precision_recall.rs` times the labeled set (v3implementation
//! §17.5), or recorded words, detected and validated, with the user's
//! choices; and their scenes, with the watermark or without. Used by every
//! scene test that builds from a transcript; `labeled.rs` reads the files of
//! `fixtures/labeled/` through it.

use std::collections::BTreeMap;

use offcut_detect::{DetectorConfig, detect};
use offcut_plan::{PlanConfig, ValidPlan, from_rules, validate};
use offcut_scene::{FontSet, Scene, SceneInput, Theme, build_scene};
use offcut_text::{RawWord, normalize_transcript};
use offcut_types::{
    BitsPerSec, ClipInfo, DurMs, EditState, ExportProfile, Hz, Orientation, ProfileKind, Prosody,
    Px, Rotation, TimeMap, Transcript, WordIdx, WordProsody,
};

/// A scene's input, but for the output width and the watermark.
pub struct Case {
    pub name: String,
    pub transcript: Transcript,
    pub plan: ValidPlan,
    pub edit: EditState,
    pub length: u32,
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

/// Written `sentences` (a word in `*…*` is stressed), timed as the labeled
/// files are, with the word `edits`, detected and validated, with `choose`
/// applied to the user's choices.
pub fn written(
    name: &str,
    sentences: &[&str],
    edits: BTreeMap<WordIdx, String>,
    choose: impl FnOnce(&mut EditState, &ValidPlan),
) -> Case {
    let sentences: Vec<String> = sentences.iter().map(|s| (*s).to_owned()).collect();
    detected(name, synthesize(&sentences), edits, choose)
}

pub fn detected(
    name: &str,
    (raw, per_word, length): (Vec<RawWord>, Vec<WordProsody>, u32),
    edits: BTreeMap<WordIdx, String>,
    choose: impl FnOnce(&mut EditState, &ValidPlan),
) -> Case {
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
        name: name.to_owned(),
        transcript,
        plan,
        edit,
        length,
    }
}

/// The scene of `case` for an output `width` wide, 9:16, with no watermark.
pub fn build(case: &Case, width: u32) -> Scene {
    build_with(case, width, false)
}

/// The scene of `case` for an output `width` wide, 9:16, with the
/// watermark when `watermark`.
pub fn build_with(case: &Case, width: u32, watermark: bool) -> Scene {
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
            watermark,
            video_bitrate: BitsPerSec::new(8_000_000),
            audio_bitrate: BitsPerSec::new(160_000),
        },
        time_map: &TimeMap::identity(DurMs::new(case.length)),
        theme: &Theme::default_for(case.edit.style),
        fonts: &FontSet::embedded(),
    })
    .unwrap()
}
