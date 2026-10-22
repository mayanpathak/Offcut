//! `Scene::hit` (D-174, v4implementation §8.10): a word's centre hits that
//! word; a point of the caption block away from the words hits `Captions`;
//! a point outside it hits `Video`; while no chunk is shown no word or block
//! is hit; a point outside the canvas hits nothing, and no point panics.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::collections::BTreeMap;

use offcut_plan::{PlanConfig, from_rules, validate};
use offcut_scene::captions::words;
use offcut_scene::hit::HIT_PADDING;
use offcut_scene::layout::shape;
use offcut_scene::{DrawCmd, FontSet, Hit, Scene, SceneInput, Theme, build_scene};
use offcut_text::{RawWord, normalize_transcript};
use offcut_types::{
    BitsPerSec, ClipInfo, DurMs, EditState, ExportProfile, Hz, Orientation, OutMs, ProfileKind, Px,
    Rotation, StyleId, TimeMap, Transcript, WordIdx,
};

/// Two sentences, a second of silence between them: two chunks of two
/// lines and one of one, then nothing shown.
fn transcript() -> Transcript {
    let texts = [
        "We", "finally", "shipped", "the", "new", "editor", "today.", "Everyone", "loved", "it.",
    ];
    let mut raw = Vec::new();
    let mut at = 500;
    for text in texts {
        raw.push(RawWord {
            text: text.to_owned(),
            start_ms: at,
            end_ms: at + 250,
            confidence: 1.0,
        });
        at += if text.ends_with('.') { 1_250 } else { 300 };
    }
    normalize_transcript(raw, "hit")
}

fn scene(t: &Transcript) -> Scene {
    let (plan, _) = validate(
        from_rules(&[], t),
        t,
        &BTreeMap::new(),
        &PlanConfig::default(),
    )
    .unwrap();
    build_scene(SceneInput {
        transcript: t,
        plan: &plan,
        edit: &EditState::default(),
        clip: &ClipInfo {
            duration: DurMs::new(10_000),
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
            width: Px::new(1080),
            height: Px::new(1920),
            watermark: false,
            video_bitrate: BitsPerSec::new(8_000_000),
            audio_bitrate: BitsPerSec::new(160_000),
        },
        time_map: &TimeMap::identity(DurMs::new(10_000)),
        theme: &Theme::default_for(StyleId::Clean),
        fonts: &FontSet::embedded(),
    })
    .unwrap()
}

/// The centre of each caption run drawn at `t`, with its text: at 1080 wide
/// a logical pixel is an output pixel.
fn centres(scene: &Scene, t: u32) -> Vec<(String, f32, f32)> {
    let list = scene.frame_at(OutMs::new(t));
    list.cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DrawCmd::GlyphRun {
                font,
                size,
                transform,
                text,
                ..
            } => {
                let run = &shape(text, *font, *size, &FontSet::embedded()).unwrap()[0];
                let k = transform.m[0];
                let x = transform.m[4] + run.width * k / 2.0;
                let y = transform.m[5] - (run.ascent - run.descent) * k / 2.0;
                Some((text.clone(), x, y))
            }
            DrawCmd::FillRect { .. }
            | DrawCmd::FillPath { .. }
            | DrawCmd::PushLayer { .. }
            | DrawCmd::PopLayer
            | DrawCmd::Video { .. }
            | DrawCmd::Image { .. } => None,
        })
        .collect()
}

#[test]
fn a_words_centre_hits_that_word_and_the_block_around_the_words_hits_captions() {
    let t = transcript();
    let scene = scene(&t);
    let index: BTreeMap<String, WordIdx> =
        words(&t, &BTreeMap::new(), &TimeMap::identity(DurMs::new(10_000)))
            .into_iter()
            .map(|w| (w.text, w.word))
            .collect();
    let mut hit_words = 0;
    // Inside each sentence's chunks, at the start of each word.
    for at in (500..9_000).step_by(100) {
        for (text, x, y) in centres(&scene, at) {
            assert_eq!(
                scene.hit(OutMs::new(at), x, y),
                Some(Hit::Word(index[&text])),
                "{text} at {at}"
            );
            hit_words += 1;
        }
    }
    assert!(hit_words > 50, "{hit_words}");

    // The block's corners are away from the centred lines: the captions,
    // while a chunk is shown; just outside the block, the video.
    let block = scene.caption_block();
    let shown = OutMs::new(600);
    assert!(!centres(&scene, 600).is_empty());
    let (left, top) = (block.x + 1.0, block.y + 1.0);
    let (right, bottom) = (block.x + block.w - 1.0, block.y + block.h - 1.0);
    for (x, y) in [(left, top), (right, top), (left, bottom), (right, bottom)] {
        assert_eq!(scene.hit(shown, x, y), Some(Hit::Captions), "{x} {y}");
    }
    assert_eq!(scene.hit(shown, left, block.y - 1.0), Some(Hit::Video));
    assert_eq!(scene.hit(shown, 540.0, 300.0), Some(Hit::Video));
}

#[test]
fn a_word_is_hit_within_its_padding_and_not_beyond() {
    let t = transcript();
    let scene = scene(&t);
    let at = OutMs::new(600);
    // "We" is the first word of the first line: left of it, within the
    // padding, it is hit; past the padding, the block is.
    let list = scene.frame_at(at);
    let left = list
        .cmds
        .iter()
        .find_map(|cmd| match cmd {
            DrawCmd::GlyphRun {
                text, transform, ..
            } if text == "We" => Some(transform.m[4]),
            DrawCmd::GlyphRun { .. }
            | DrawCmd::FillRect { .. }
            | DrawCmd::FillPath { .. }
            | DrawCmd::PushLayer { .. }
            | DrawCmd::PopLayer
            | DrawCmd::Video { .. }
            | DrawCmd::Image { .. } => None,
        })
        .unwrap();
    let (_, _, y) = centres(&scene, 600)
        .into_iter()
        .find(|c| c.0 == "We")
        .unwrap();
    let we = Some(Hit::Word(WordIdx::new(0)));
    assert_eq!(scene.hit(at, left - HIT_PADDING + 0.5, y), we);
    assert_eq!(
        scene.hit(at, left - HIT_PADDING - 0.5, y),
        Some(Hit::Captions)
    );
}

#[test]
fn while_no_chunk_is_shown_nothing_but_the_video_is_hit_and_outside_the_canvas_nothing() {
    let t = transcript();
    let scene = scene(&t);
    let block = scene.caption_block();
    let (x, y) = (block.x + block.w / 2.0, block.y + block.h / 2.0);
    // Before the first chunk and after the last.
    for at in [0, 9_500] {
        assert!(centres(&scene, at).is_empty(), "{at}");
        assert_eq!(scene.hit(OutMs::new(at), x, y), Some(Hit::Video), "{at}");
    }
    // Outside the canvas, at any time, and never a panic over a wide sweep.
    for at in [0, 600, 9_500] {
        let at = OutMs::new(at);
        assert_eq!(scene.hit(at, -0.5, y), None);
        assert_eq!(scene.hit(at, 1_080.5, y), None);
        assert_eq!(scene.hit(at, x, 1_920.5), None);
        assert_eq!(scene.hit(at, f32::NAN, y), None);
        assert_eq!(scene.hit(at, x, f32::INFINITY), None);
        for i in -10..=60 {
            for j in -10..=110 {
                let _ = scene.hit(at, i as f32 * 20.0, j as f32 * 20.0);
            }
        }
    }
}
