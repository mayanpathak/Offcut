//! Nothing leaves the safe area, and nothing meets the watermark (TS §19.6,
//! D-143, D-158): the scenes of the 31 labeled transcripts, in the three
//! styles, at the three snap points of the captions (D-171), with the
//! watermark on, at every 33 ms. Each glyph run, rectangle and path is
//! measured by its box: a run from its font's ascent to its descent, its
//! outline included.

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

use std::path::PathBuf;

use offcut_scene::layout::shape;
use offcut_scene::placement::{SNAP_BOTTOM, SNAP_MIDDLE, SNAP_TOP};
use offcut_scene::safe_area::SAFE_AREA;
use offcut_scene::watermark::{self, WATERMARK_TEXT};
use offcut_scene::{DrawCmd, FontSet, PathEl, Rect, Theme};
use offcut_types::{CaptionPlacement, EventKind, OutMs, StyleId};

const STYLES: [StyleId; 3] = [StyleId::Clean, StyleId::Bold, StyleId::Tech];
const PLACES: [f32; 3] = [SNAP_TOP, SNAP_MIDDLE, SNAP_BOTTOM];
/// The step between two instants checked.
const STEP_MS: u32 = 33;

/// The labeled files, by name, in order.
fn names() -> Vec<String> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/labeled");
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .map(|path| path.file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn rect_of(points: &[(f32, f32)]) -> Option<Rect> {
    let (mut left, mut top) = (f32::MAX, f32::MAX);
    let (mut right, mut bottom) = (f32::MIN, f32::MIN);
    for &(x, y) in points {
        (left, top) = (left.min(x), top.min(y));
        (right, bottom) = (right.max(x), bottom.max(y));
    }
    (left <= right).then_some(Rect {
        x: left,
        y: top,
        w: right - left,
        h: bottom - top,
    })
}

/// The box a command draws in, at 1080 wide (a logical pixel is an output
/// pixel), and whether it is the watermark's. `None` for a layer or the
/// video.
fn bounds(cmd: &DrawCmd) -> Option<(Rect, bool)> {
    match cmd {
        DrawCmd::GlyphRun {
            font,
            size,
            stroke,
            transform,
            text,
            ..
        } => {
            let runs = shape(text, *font, *size, &FontSet::embedded()).unwrap();
            let width: f32 = runs.iter().map(|run| run.width).sum();
            let ascent = runs.iter().map(|run| run.ascent).fold(0.0, f32::max);
            let descent = runs.iter().map(|run| run.descent).fold(0.0, f32::max);
            let pad = stroke.map_or(0.0, |stroke| stroke.width / 2.0);
            let corners = [(-pad, -ascent - pad), (width + pad, descent + pad)];
            let box_ = rect_of(&corners.map(|(x, y)| transform.apply(x, y)))?;
            Some((box_, text == WATERMARK_TEXT))
        }
        DrawCmd::FillRect { rect, .. } => Some((*rect, false)),
        DrawCmd::FillPath { path, .. } => {
            let points: Vec<(f32, f32)> = path
                .iter()
                .flat_map(|el| match *el {
                    PathEl::MoveTo(x, y) | PathEl::LineTo(x, y) => vec![(x, y)],
                    PathEl::QuadTo(a, b, x, y) => vec![(a, b), (x, y)],
                    PathEl::CurveTo(a, b, c, d, x, y) => vec![(a, b), (c, d), (x, y)],
                    PathEl::Close => Vec::new(),
                })
                .collect();
            rect_of(&points).map(|r| (r, false))
        }
        DrawCmd::PushLayer { .. }
        | DrawCmd::PopLayer
        | DrawCmd::Video { .. }
        | DrawCmd::Image { .. } => None,
    }
}

/// What a command is, in a few words: its text, or its kind.
fn kind(cmd: &DrawCmd) -> String {
    match cmd {
        DrawCmd::GlyphRun { text, size, .. } => format!("{text:?} at {size}"),
        DrawCmd::FillRect { radius, .. } => format!("a rectangle (radius {radius})"),
        DrawCmd::FillPath { .. } => "a path".to_owned(),
        DrawCmd::PushLayer { .. } | DrawCmd::PopLayer => "a layer".to_owned(),
        DrawCmd::Video { .. } | DrawCmd::Image { .. } => "a picture".to_owned(),
    }
}

fn inside(r: Rect, area: Rect) -> bool {
    r.x >= area.x - 0.01
        && r.y >= area.y - 0.01
        && r.x + r.w <= area.x + area.w + 0.01
        && r.y + r.h <= area.y + area.h + 0.01
}

fn overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

#[test]
fn no_box_of_any_labeled_scene_leaves_the_safe_area_or_meets_the_watermark() {
    let names = names();
    assert_eq!(names.len(), 31, "{names:?}");
    let (mut scenes_built, mut frames, mut boxes) = (0u32, 0u64, 0u64);
    let mut problems: Vec<String> = Vec::new();
    for style in STYLES {
        let theme = Theme::default_for(style);
        let mark = watermark::bounds(&watermark::layout(&theme, &FontSet::embedded()).unwrap());
        for name in &names {
            for p in PLACES {
                let case = labeled::case(name, |edit, _| {
                    edit.style = style;
                    edit.caption_position = CaptionPlacement::new(p).unwrap();
                });
                let scene = scenes::build_with(&case, 1080, true);
                scenes_built += 1;
                let mut t = 0;
                while t < scene.duration().get() {
                    frames += 1;
                    let list = scene.frame_at(OutMs::new(t));
                    let mut marked = false;
                    for cmd in &list.cmds {
                        let Some((r, is_mark)) = bounds(cmd) else {
                            continue;
                        };
                        boxes += 1;
                        marked |= is_mark;
                        let what =
                            || format!("{name} {style:?} at {p}, {t} ms: {r:?} of {}", kind(cmd));
                        if !inside(r, SAFE_AREA) {
                            problems.push(format!("outside the safe area: {}", what()));
                        }
                        if !is_mark && overlap(r, mark) {
                            problems.push(format!("on the watermark: {}", what()));
                        }
                    }
                    if !marked {
                        problems.push(format!("{name} {style:?} at {p}, {t} ms: no watermark"));
                    }
                    t += STEP_MS;
                }
            }
        }
    }
    println!(
        "{scenes_built} scenes ({} files x {} styles x {} placements), {frames} frames, {boxes} boxes, {} problems",
        names.len(),
        STYLES.len(),
        PLACES.len(),
        problems.len()
    );
    let first: Vec<&String> = problems.iter().take(20).collect();
    assert!(problems.is_empty(), "{first:#?}");
}

#[test]
fn every_from_to_of_the_labeled_set_steps_at_one_mark_after_its_first_word() {
    // §8.7: `validate` stores the second value's time; the scene reads it.
    let mut seen = 0;
    for name in names() {
        let case = labeled::case(&name, |_, _| {});
        for item in case.plan.items() {
            if item.kind != EventKind::FromTo {
                continue;
            }
            seen += 1;
            let first = &case.transcript.words[item.anchors.start.get() as usize];
            assert_eq!(item.marks.len(), 1, "{name}: {item:?}");
            assert!(item.marks[0] > first.start_ms, "{name}: {item:?}");
            assert!(item.marks[0] < item.span.end, "{name}: {item:?}");
        }
    }
    // The detector finds 11 of the 12 the labels hold.
    println!("{seen} FromTo items, each with one mark");
    assert!(seen >= 10, "{seen}");
}

#[test]
fn the_six_shared_scenes_stay_in_the_safe_area_without_the_watermark() {
    // Their own choices: placement 0.0, numbers not highlighted, a word
    // edited to "$500", a visual turned off.
    for case in labeled::six() {
        let scene = scenes::build(&case, 1080);
        let mut t = 0;
        while t < scene.duration().get() {
            for cmd in &scene.frame_at(OutMs::new(t)).cmds {
                if let Some((r, is_mark)) = bounds(cmd) {
                    assert!(!is_mark, "{}: a watermark nobody asked for", case.name);
                    assert!(
                        inside(r, SAFE_AREA),
                        "{} at {t}: {r:?} of {}",
                        case.name,
                        kind(cmd)
                    );
                }
            }
            t += STEP_MS;
        }
    }
}
