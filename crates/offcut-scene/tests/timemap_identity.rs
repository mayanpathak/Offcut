//! Seam S-1 on the identity map (TS §40.1, v4implementation §8.10): for the
//! reference clip's plan, a scene on `TimeMap::identity(d)` lasts `d`, has the
//! frame count of TS §19.2, reads the source at the program time of every
//! frame, and shows every caption word and every item at the program time
//! that equals its recording time.
//!
//! The transcript is `fixtures/labeled/a01-scriptA.json`, the reference
//! clip's recognized text in written form, timed as the labeled tests time
//! it, with "brutal" (word 31) stressed as a measured recording stresses it
//! (as `offcut-plan/tests/plan_from_rules.rs` builds it, V4 Prompt 97).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use offcut_detect::{DetectorConfig, detect};
use offcut_plan::{PlanConfig, ValidPlan, from_rules, validate};
use offcut_scene::captions::words;
use offcut_scene::{DisplayList, DrawCmd, FontSet, Scene, SceneInput, Theme, build_scene};
use offcut_text::{RawWord, normalize_transcript};
use offcut_types::{
    BitsPerSec, ClipInfo, DurMs, EditState, EventKind, ExportProfile, FrameIdx, Hz, Orientation,
    OutMs, ProfileKind, Prosody, Px, Rotation, SrcPos, StyleId, TimeMap, TimeMs, Transcript,
    WordProsody,
};

/// `synthesize` of `offcut-detect/tests/precision_recall.rs` (v3implementation
/// §17.5). Also returns where the next word would start: the clip's length.
fn synthesize(sentences: &[String]) -> (Vec<RawWord>, Prosody, u32) {
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
    (raw, Prosody { per_word }, at)
}

/// The reference transcript, its valid rules plan, and the clip's length.
fn reference() -> (Transcript, ValidPlan, u32) {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/labeled/a01-scriptA.json");
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("a01")).expect("a01 parses");
    let sentences: Vec<String> = serde_json::from_value(json["sentences"].clone()).unwrap();
    let (raw, mut p, length) = synthesize(&sentences);
    assert_eq!(raw[31].text, "brutal", "the word README names");
    p.per_word[31] = WordProsody {
        energy_z: 2.0,
        pitch_z: 2.0,
    };
    let t = normalize_transcript(raw, "labeled");
    let events = detect(&t, &p, &BTreeMap::new(), &DetectorConfig::default());
    assert_eq!(events.len(), 5, "README's five events");
    let rules = from_rules(&events, &t);
    let (plan, dropped) = validate(rules, &t, &BTreeMap::new(), &PlanConfig::default()).unwrap();
    assert_eq!(dropped, []);
    (t, plan, length)
}

fn clip(duration_ms: u32) -> ClipInfo {
    ClipInfo {
        duration: DurMs::new(duration_ms),
        display_width: Px::new(1080),
        display_height: Px::new(1920),
        rotation: Rotation::R0,
        orientation: Orientation::Portrait,
        is_vfr: false,
        video_codec_string: "avc1.640028".to_owned(),
        audio_codec_string: "mp4a.40.2".to_owned(),
        audio_sample_rate: Hz::new(48_000),
        audio_channels: 2,
    }
}

fn profile() -> ExportProfile {
    ExportProfile {
        kind: ProfileKind::Creator,
        width: Px::new(1080),
        height: Px::new(1920),
        watermark: false,
        video_bitrate: BitsPerSec::new(8_000_000),
        audio_bitrate: BitsPerSec::new(160_000),
    }
}

/// The glyph runs of a frame: their text and fill, and whether each is in
/// the event zone (its baseline above the caption zone, at 1080 wide).
fn runs(list: &DisplayList) -> Vec<(String, [u8; 4], bool)> {
    list.cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DrawCmd::GlyphRun {
                text,
                fill,
                transform,
                ..
            } => Some((
                text.clone(),
                [fill.r, fill.g, fill.b, fill.a],
                transform.m[5] < 1_200.0,
            )),
            DrawCmd::FillRect { .. }
            | DrawCmd::FillPath { .. }
            | DrawCmd::PushLayer { .. }
            | DrawCmd::PopLayer
            | DrawCmd::Video { .. }
            | DrawCmd::Image { .. } => None,
        })
        .collect()
}

fn item_drawn(scene: &Scene, t: u32) -> bool {
    runs(&scene.frame_at(OutMs::new(t))).iter().any(|run| run.2)
}

#[test]
fn on_the_identity_map_every_program_time_is_its_recording_time() {
    let (t, plan, length) = reference();
    let map = TimeMap::identity(DurMs::new(length));
    let theme = Theme::default_for(StyleId::Clean);
    let edit = EditState::default();
    let scene = build_scene(SceneInput {
        transcript: &t,
        plan: &plan,
        edit: &edit,
        clip: &clip(length),
        profile: &profile(),
        time_map: &map,
        theme: &theme,
        fonts: &FontSet::embedded(),
    })
    .unwrap();

    // The length, and TS §19.2's frame count.
    assert_eq!(scene.duration(), DurMs::new(length));
    let frames = (u64::from(length) * 30).div_ceil(1_000);
    assert_eq!(u64::from(scene.frame_count()), frames);

    // The source is read at the frame's own time, at every frame.
    for n in 0..scene.frame_count() {
        let at = FrameIdx::new(n).to_out_ms();
        assert_eq!(
            scene.source_at(at),
            SrcPos::At(TimeMs::new(at.get())),
            "frame {n}"
        );
    }

    // Every caption word is spoken at its recording time: from its first
    // millisecond to its last it is the one run in the active colour.
    let accent = [
        theme.accent.r,
        theme.accent.g,
        theme.accent.b,
        theme.accent.a,
    ];
    let shown = words(&t, &edit.word_edits, &map);
    assert!(shown.len() > 100, "{}", shown.len());
    for word in &shown {
        let spoken = &t.words[word.word.get() as usize];
        assert_eq!(word.start.get(), spoken.start_ms.get(), "{word:?}");
        for at in [word.start.get(), word.end.get() - 1] {
            let active: Vec<String> = runs(&scene.frame_at(OutMs::new(at)))
                .into_iter()
                .filter(|run| run.1 == accent && !run.2)
                .map(|run| run.0)
                .collect();
            assert_eq!(active.concat(), word.text, "at {at} ms");
        }
    }

    // Every item the scene draws is there from its recording start until its
    // hold and fade end, and not a millisecond before or after (TS §19.5).
    let numbers: Vec<_> = plan
        .items()
        .iter()
        .filter(|item| item.kind == EventKind::NumberReveal)
        .collect();
    assert_eq!(numbers.len(), 2, "3,000 hours and $12k");
    assert_eq!(scene.summary().visual_moments, 2);
    for item in numbers {
        let (start, end) = (item.span.start.get(), item.span.end.get() + 1_200 + 200);
        assert!(!item_drawn(&scene, start - 1), "{item:?}");
        assert!(item_drawn(&scene, start), "{item:?}");
        assert!(item_drawn(&scene, end - 1), "{item:?}");
        assert!(!item_drawn(&scene, end), "{item:?}");
    }
}
