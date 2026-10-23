//! The preview is the export (D-172): for six scenes, the display lists at
//! 540 (the preview), 720 and 1080 wide are the same list after scaling,
//! every coordinate within 0.01 logical pixel of the 1080 list's, every
//! colour, glyph, text, opacity and source rectangle equal.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

#[path = "common/labeled.rs"]
mod labeled;
#[path = "common/scenes.rs"]
mod scenes;

use offcut_scene::{DrawCmd, PathEl, Rect};
use offcut_types::FrameIdx;

/// How far two lengths in logical pixels may be apart.
const TOLERANCE_LP: f32 = 0.01;

fn near(a: f32, b: f32, what: &str) {
    assert!((a - b).abs() <= TOLERANCE_LP, "{what}: {a} against {b}");
}

/// `r`, from an output `k` times the logical canvas, in logical pixels.
fn rect_lp(r: Rect, k: f32) -> [f32; 4] {
    [r.x / k, r.y / k, r.w / k, r.h / k]
}

fn near_rects(a: Rect, k: f32, full: Rect, what: &str) {
    for (a, b) in rect_lp(a, k).into_iter().zip(rect_lp(full, 1.0)) {
        near(a, b, what);
    }
}

fn points(el: PathEl) -> Vec<f32> {
    match el {
        PathEl::MoveTo(x, y) | PathEl::LineTo(x, y) => vec![x, y],
        PathEl::QuadTo(a, b, x, y) => vec![a, b, x, y],
        PathEl::CurveTo(a, b, c, d, x, y) => vec![a, b, c, d, x, y],
        PathEl::Close => Vec::new(),
    }
}

fn near_paths(a: &[PathEl], k: f32, full: &[PathEl], what: &str) {
    assert_eq!(a.len(), full.len(), "{what}");
    for (a, b) in a.iter().zip(full) {
        assert_eq!(
            std::mem::discriminant(a),
            std::mem::discriminant(b),
            "{what}"
        );
        for (a, b) in points(*a).into_iter().zip(points(*b)) {
            near(a / k, b, what);
        }
    }
}

/// `cmd` of an output `k` times the logical canvas is `full`, the same
/// command at 1080 wide (`k` = 1).
fn same(cmd: &DrawCmd, k: f32, full: &DrawCmd, what: &str) {
    match (cmd, full) {
        (
            DrawCmd::GlyphRun {
                font,
                size,
                glyphs,
                fill,
                stroke,
                transform,
                text,
            },
            DrawCmd::GlyphRun {
                font: font_f,
                size: size_f,
                glyphs: glyphs_f,
                fill: fill_f,
                stroke: stroke_f,
                transform: transform_f,
                text: text_f,
            },
        ) => {
            assert_eq!((font, fill, text), (font_f, fill_f, text_f), "{what}");
            near(size / k, *size_f, what);
            assert_eq!(stroke.map(|s| s.color), stroke_f.map(|s| s.color), "{what}");
            if let (Some(s), Some(f)) = (stroke, stroke_f) {
                near(s.width / k, f.width, what);
            }
            assert_eq!(glyphs.len(), glyphs_f.len(), "{what}");
            for (g, f) in glyphs.iter().zip(glyphs_f) {
                assert_eq!(g.id, f.id, "{what}");
                // Where the glyph lands on the canvas.
                let (x, y) = transform.apply(g.x, g.y);
                let (x_f, y_f) = transform_f.apply(f.x, f.y);
                near(x / k, x_f, what);
                near(y / k, y_f, what);
            }
        }
        (
            DrawCmd::FillRect {
                rect,
                radius,
                color,
            },
            DrawCmd::FillRect {
                rect: rect_f,
                radius: radius_f,
                color: color_f,
            },
        ) => {
            assert_eq!(color, color_f, "{what}");
            near_rects(*rect, k, *rect_f, what);
            near(radius / k, *radius_f, what);
        }
        (
            DrawCmd::FillPath { path, color },
            DrawCmd::FillPath {
                path: path_f,
                color: color_f,
            },
        ) => {
            assert_eq!(color, color_f, "{what}");
            near_paths(path, k, path_f, what);
        }
        (
            DrawCmd::PushLayer { opacity, clip },
            DrawCmd::PushLayer {
                opacity: opacity_f,
                clip: clip_f,
            },
        ) => {
            assert_eq!(opacity, opacity_f, "{what}");
            assert_eq!(clip.is_some(), clip_f.is_some(), "{what}");
            if let (Some(c), Some(f)) = (clip, clip_f) {
                near_rects(*c, k, *f, what);
            }
        }
        (DrawCmd::PopLayer, DrawCmd::PopLayer) => {}
        // The video is placed in output pixels: the whole canvas at every
        // width, from the same part of the source.
        (
            DrawCmd::Video {
                slot,
                src,
                dst,
                opacity,
                mask,
            },
            DrawCmd::Video {
                slot: slot_f,
                src: src_f,
                dst: dst_f,
                opacity: opacity_f,
                mask: mask_f,
            },
        ) => {
            assert_eq!((slot, src, opacity), (slot_f, src_f, opacity_f), "{what}");
            near_rects(*dst, k, *dst_f, what);
            assert_eq!(mask.is_some(), mask_f.is_some(), "{what}");
            if let (Some(m), Some(f)) = (mask, mask_f) {
                near_paths(m, k, f, what);
            }
        }
        (
            DrawCmd::Image {
                image,
                rect,
                opacity,
            },
            DrawCmd::Image {
                image: image_f,
                rect: rect_f,
                opacity: opacity_f,
            },
        ) => {
            assert_eq!((image, opacity), (image_f, opacity_f), "{what}");
            near_rects(*rect, k, *rect_f, what);
        }
        (a, b) => panic!("{what}: {a:?} against {b:?}"),
    }
}

#[test]
fn six_scenes_draw_the_same_lists_at_540_720_and_1080_after_scaling() {
    for case in labeled::six() {
        let full = scenes::build(&case, 1080);
        let mut compared = 0usize;
        for width in [540u32, 720] {
            let k = width as f32 / 1080.0;
            let small = scenes::build(&case, width);
            assert_eq!(small.frame_count(), full.frame_count(), "{}", case.name);
            for frame in 0..full.frame_count() {
                let t = FrameIdx::new(frame).to_out_ms();
                let (list, list_f) = (small.frame_at(t), full.frame_at(t));
                let what = format!("{} at {width}, frame {frame}", case.name);
                assert_eq!(list.cmds.len(), list_f.cmds.len(), "{what}");
                for (cmd, cmd_f) in list.cmds.iter().zip(&list_f.cmds) {
                    same(cmd, k, cmd_f, &what);
                    compared += 1;
                }
            }
        }
        assert!(compared > 2_000, "{}: {compared}", case.name);
    }
}
