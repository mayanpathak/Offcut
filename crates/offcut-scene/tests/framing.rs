//! The crop (D-131, TS §19.3, §19.7, §8.10): the 9:16 part of the frame each
//! clip shows, moved by the user's offset, and the `Video` command of every
//! frame carrying it.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use offcut_scene::framing::crop_rect;
use offcut_scene::{CropRect, DrawCmd};
use offcut_types::{ClipInfo, CropOffset, DurMs, Hz, Orientation, Px, Rotation};

fn clip(width: u32, height: u32, rotation: Rotation) -> ClipInfo {
    ClipInfo {
        duration: DurMs::new(10_000),
        display_width: Px::new(width),
        display_height: Px::new(height),
        rotation,
        orientation: if width > height {
            Orientation::Landscape
        } else {
            Orientation::Portrait
        },
        is_vfr: false,
        video_codec_string: "avc1.640028".to_owned(),
        audio_codec_string: "mp4a.40.2".to_owned(),
        audio_sample_rate: Hz::new(48_000),
        audio_channels: 2,
    }
}

fn crop(width: u32, height: u32, offset: f32) -> CropRect {
    crop_rect(
        &clip(width, height, Rotation::R0),
        CropOffset::new(offset).unwrap(),
    )
}

#[test]
fn a_1920x1080_clip_shows_the_strip_its_offset_names() {
    // x = (1920 - 607.5) / 2 x (1 + offset).
    let strips = [
        (-1.0, 0.0),
        (-0.5, 328.125),
        (0.0, 656.25),
        (0.5, 984.375),
        (1.0, 1_312.5),
    ];
    for (offset, x) in strips {
        let expected = CropRect {
            x,
            y: 0.0,
            w: 607.5,
            h: 1_080.0,
        };
        assert_eq!(crop(1920, 1080, offset), expected, "{offset}");
    }
}

#[test]
fn ok_portrait_3x4() {
    // A 3:4 frame is wider than 9:16: its sides go (810 of 1080 shown),
    // moved by the offset like a landscape frame's.
    let at = |offset| crop(1080, 1440, offset);
    assert_eq!(
        at(0.0),
        CropRect {
            x: 135.0,
            y: 0.0,
            w: 810.0,
            h: 1_440.0
        }
    );
    assert_eq!(at(-1.0).x, 0.0);
    assert_eq!(at(1.0).x, 270.0);
}

#[test]
fn a_1080x1920_clip_is_shown_whole_at_any_offset() {
    let whole = CropRect {
        x: 0.0,
        y: 0.0,
        w: 1_080.0,
        h: 1_920.0,
    };
    for offset in [-1.0, -0.5, 0.0, 0.5, 1.0] {
        assert_eq!(crop(1080, 1920, offset), whole, "{offset}");
    }
    // Narrower than 9:16: the top and the bottom go, and the offset is
    // ignored (TS §19.7).
    for offset in [-1.0, 0.0, 1.0] {
        let tall = crop(900, 1920, offset);
        assert_eq!(
            (tall.x, tall.y, tall.w, tall.h),
            (0.0, 160.0, 900.0, 1_600.0)
        );
    }
}

#[test]
fn a_clip_rotated_90_is_cropped_by_its_display_size() {
    // Recorded 1280 x 720 and turned: shown 720 x 1280, exactly 9:16.
    for rotation in [Rotation::R90, Rotation::R270] {
        let crop = crop_rect(&clip(720, 1280, rotation), CropOffset::new(0.5).unwrap());
        assert_eq!(
            crop,
            CropRect {
                x: 0.0,
                y: 0.0,
                w: 720.0,
                h: 1_280.0
            }
        );
    }
}

#[test]
fn every_frame_draws_the_crop_first() {
    use offcut_plan::{PlanConfig, from_rules, validate};
    use offcut_scene::{FontSet, SceneInput, Theme, build_scene};
    use offcut_types::{
        BitsPerSec, EditState, ExportProfile, OutMs, ProfileKind, StyleId, TimeMap, Transcript,
    };

    let transcript = Transcript {
        words: Vec::new(),
        sentences: Vec::new(),
        numbers: Vec::new(),
        model_version: "framing".to_owned(),
    };
    let edits = Default::default();
    let rules = from_rules(&[], &transcript);
    let (plan, _) = validate(rules, &transcript, &edits, &PlanConfig::default()).unwrap();
    let landscape = clip(1920, 1080, Rotation::R0);
    for offset in [-1.0, 0.25, 1.0] {
        let edit = EditState {
            crop_offset: CropOffset::new(offset).unwrap(),
            ..EditState::default()
        };
        let scene = build_scene(SceneInput {
            transcript: &transcript,
            plan: &plan,
            edit: &edit,
            clip: &landscape,
            profile: &ExportProfile {
                kind: ProfileKind::Creator,
                width: Px::new(540),
                height: Px::new(960),
                watermark: false,
                video_bitrate: BitsPerSec::new(8_000_000),
                audio_bitrate: BitsPerSec::new(160_000),
            },
            time_map: &TimeMap::identity(DurMs::new(10_000)),
            theme: &Theme::default_for(StyleId::Clean),
            fonts: &FontSet::embedded(),
        })
        .unwrap();
        let expected = crop_rect(&landscape, edit.crop_offset);
        for t in [0, 4_000, 9_999] {
            let list = scene.frame_at(OutMs::new(t));
            let DrawCmd::Video { src, dst, .. } = &list.cmds[0] else {
                panic!("the video first");
            };
            assert_eq!(*src, expected, "{offset} at {t}");
            // On the whole output canvas.
            assert_eq!((dst.x, dst.y, dst.w, dst.h), (0.0, 0.0, 540.0, 960.0));
        }
    }
}
