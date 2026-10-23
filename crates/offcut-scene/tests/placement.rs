//! Caption placement (D-171, v4implementation §8.10): at the two ends of the
//! band and the three snap points, the caption block and the event zone are
//! inside the safe area and apart, the watermark's band is clear of what it
//! must not meet, the captions and the visuals are drawn where the placement
//! puts them, and the watermark flag changes no placement. At 1.0 the block
//! ends where the watermark's band begins (1164 to 1444, the event zone 804
//! to 1124; V3's were 56 lp lower).
//!
//! The scene is the reference clip's transcript (`fixtures/labeled/a01-
//! scriptA.json`, timed as the labeled tests time it), whose two numbers are
//! drawn in the event zone.

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
use offcut_scene::placement::{
    CAPTION_BOTTOM, SNAP_BOTTOM, SNAP_MIDDLE, SNAP_TOP, WATERMARK_BAND, caption_block, event_zone,
};
use offcut_scene::safe_area::{SAFE_AREA, ZoneId, zone};
use offcut_scene::styles::spec;
use offcut_scene::watermark;
use offcut_scene::{DrawCmd, FontSet, Rect, Scene, SceneInput, Theme, build_scene};
use offcut_text::{RawWord, normalize_transcript};
use offcut_types::{
    BitsPerSec, CaptionPlacement, ClipInfo, DurMs, EditState, ExportProfile, FrameIdx, Hz,
    Orientation, ProfileKind, Prosody, Px, Rotation, StyleId, TimeMap, WordProsody,
};

/// The two ends of the band and its three snap points.
const PLACES: [f32; 4] = [0.0, SNAP_TOP, SNAP_MIDDLE, SNAP_BOTTOM];

/// The rows of D-143's watermark box, as `watermark.rs` lays it out.
fn watermark_band() -> (f32, f32) {
    let theme = Theme::default_for(StyleId::Clean);
    let mark = watermark::bounds(&watermark::layout(&theme, &FontSet::embedded()).unwrap());
    (mark.y, mark.y + mark.h)
}

fn placement(p: f32) -> CaptionPlacement {
    CaptionPlacement::new(p).unwrap()
}

fn inside(inner: Rect, outer: Rect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.w <= outer.x + outer.w
        && inner.y + inner.h <= outer.y + outer.h
}

/// Whether the rows `top..bottom` of two boxes share any height.
fn overlap((top, bottom): (f32, f32), (other_top, other_bottom): (f32, f32)) -> bool {
    top < other_bottom && other_top < bottom
}

fn rows(r: Rect) -> (f32, f32) {
    (r.y, r.y + r.h)
}

/// The reference transcript as `timemap_identity.rs` times it, without the
/// stress (no item here needs it), and the clip's length.
fn reference() -> (offcut_types::Transcript, offcut_plan::ValidPlan, u32) {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/labeled/a01-scriptA.json");
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("a01")).expect("a01 parses");
    let sentences: Vec<String> = serde_json::from_value(json["sentences"].clone()).unwrap();
    let (mut raw, mut at) = (Vec::new(), 500u32);
    for piece in sentences.iter().flat_map(|s| s.split_whitespace()) {
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
    let prosody = Prosody {
        per_word: vec![
            WordProsody {
                energy_z: 0.0,
                pitch_z: 0.0
            };
            raw.len()
        ],
    };
    let t = normalize_transcript(raw, "labeled");
    let events = detect(&t, &prosody, &BTreeMap::new(), &DetectorConfig::default());
    let (plan, dropped) = validate(
        from_rules(&events, &t),
        &t,
        &BTreeMap::new(),
        &PlanConfig::default(),
    )
    .unwrap();
    assert_eq!(dropped, []);
    (t, plan, at)
}

fn scene(p: f32, watermark: bool) -> Scene {
    let (t, plan, length) = reference();
    let edit = EditState {
        caption_position: placement(p),
        ..EditState::default()
    };
    build_scene(SceneInput {
        transcript: &t,
        plan: &plan,
        edit: &edit,
        clip: &ClipInfo {
            duration: DurMs::new(length),
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
            watermark,
            video_bitrate: BitsPerSec::new(8_000_000),
            audio_bitrate: BitsPerSec::new(160_000),
        },
        time_map: &TimeMap::identity(DurMs::new(length)),
        theme: &Theme::default_for(StyleId::Clean),
        fonts: &FontSet::embedded(),
    })
    .unwrap()
}

#[test]
fn the_block_and_the_event_zone_are_inside_the_safe_area_and_apart_at_every_snap_point() {
    let (band_top, band_bottom) = watermark_band();
    // The watermark's box lies in the band kept for it at the safe area's
    // bottom, below the lowest place a caption block's centre can be.
    let bottom = SAFE_AREA.y + SAFE_AREA.h;
    assert!(band_top > bottom - WATERMARK_BAND && band_bottom <= bottom + 0.001);
    assert!(band_top > CAPTION_BOTTOM, "{band_top}");
    for p in PLACES {
        let block = caption_block(placement(p), &spec(StyleId::Clean));
        let events = event_zone(block);
        // `safe_area::zone` gives the same two boxes for the placement.
        assert_eq!(zone(ZoneId::Caption, placement(p)), block, "{p}");
        assert_eq!(zone(ZoneId::Event, placement(p)), events, "{p}");
        assert!(inside(block, SAFE_AREA), "{p}: {block:?}");
        assert!(inside(events, SAFE_AREA), "{p}: {events:?}");
        assert!(!overlap(rows(block), rows(events)), "{p}");
        assert!(!overlap(rows(events), (band_top, band_bottom)), "{p}");
        // The block is clear of the watermark everywhere, the bottom
        // included (the founder's choice of Entry 17; `layout_safe_area.rs`
        // checks every caption box on the labeled set).
        assert!(!overlap(rows(block), (band_top, band_bottom)), "{p}");
    }
    // At 1.0, the block's bottom on the band: 56 lp above V3's zones.
    let block = caption_block(placement(1.0), &spec(StyleId::Clean));
    assert_eq!(rows(block), (1_164.0, 1_444.0));
    assert_eq!(rows(event_zone(block)), (804.0, 1_124.0));
}

#[test]
fn the_captions_and_the_numbers_are_drawn_where_the_placement_puts_them() {
    let caption_size = spec(StyleId::Clean).size;
    for p in PLACES {
        let scene = scene(p, false);
        let block = scene.caption_block();
        let events = event_zone(block);
        let (mut captions, mut numbers) = (0, 0);
        for frame in 0..scene.frame_count() {
            let list = scene.frame_at(FrameIdx::new(frame).to_out_ms());
            for cmd in &list.cmds {
                let DrawCmd::GlyphRun {
                    size, transform, ..
                } = cmd
                else {
                    continue;
                };
                // At 1080 wide a logical pixel is an output pixel.
                let baseline = transform.m[5];
                let zone = if *size == caption_size {
                    captions += 1;
                    block
                } else {
                    numbers += 1;
                    events
                };
                assert!(
                    baseline > zone.y && baseline < zone.y + zone.h,
                    "{p}: a run at {baseline} outside {zone:?}"
                );
            }
        }
        assert!(
            captions > 1_000 && numbers > 50,
            "{p}: {captions} {numbers}"
        );
    }
}

#[test]
fn the_watermark_flag_changes_no_placement() {
    // D-171 and Appendix A: `placement.rs` never reads `profile.watermark`,
    // so the preview (no watermark) and a Free export place captions alike.
    // The watermark, once drawn (V4 Prompt 106), is the last command of a
    // frame: without it, a frame is the same list.
    for p in PLACES {
        let (without, with) = (scene(p, false), scene(p, true));
        assert_eq!(without.caption_block(), with.caption_block(), "{p}");
        assert_eq!(without.frame_count(), with.frame_count());
        for frame in 0..without.frame_count() {
            let t = FrameIdx::new(frame).to_out_ms();
            let (a, b) = (without.frame_at(t), with.frame_at(t));
            assert!(b.cmds.len() >= a.cmds.len(), "{p}, frame {frame}");
            assert_eq!(a.cmds, b.cmds[..a.cmds.len()], "{p}, frame {frame}");
        }
    }
}
