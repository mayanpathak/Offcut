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
// V6.2 (P202): a model's plan of emphases over the labeled files.
#[path = "common/model.rs"]
mod model;

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

/// V6.2 (P201, D-479, D-480): the Glow style over the same labeled scenes,
/// placements and instants, watermark on. Its glow reaches 12 lp past the
/// letters, and the box of each glow stroke counts its width: none leaves
/// the safe area or meets the watermark.
#[test]
fn no_box_of_any_labeled_scene_in_glow_leaves_the_safe_area_or_meets_the_watermark() {
    let style = StyleId::Glow;
    let names = names();
    let theme = Theme::default_for(style);
    let mark = watermark::bounds(&watermark::layout(&theme, &FontSet::embedded()).unwrap());
    let (mut frames, mut glows) = (0u64, 0u64);
    let mut problems: Vec<String> = Vec::new();
    for name in &names {
        for p in PLACES {
            let case = labeled::case(name, |edit, _| {
                edit.style = style;
                edit.caption_position = CaptionPlacement::new(p).unwrap();
            });
            let scene = scenes::build_with(&case, 1080, true);
            let mut t = 0;
            while t < scene.duration().get() {
                frames += 1;
                for cmd in &scene.frame_at(OutMs::new(t)).cmds {
                    let Some((r, is_mark)) = bounds(cmd) else {
                        continue;
                    };
                    if let DrawCmd::GlyphRun { fill, .. } = cmd
                        && fill.a == 0
                    {
                        glows += 1;
                    }
                    let what = || format!("{name} at {p}, {t} ms: {r:?} of {}", kind(cmd));
                    if !inside(r, SAFE_AREA) {
                        problems.push(format!("outside the safe area: {}", what()));
                    }
                    if !is_mark && overlap(r, mark) {
                        problems.push(format!("on the watermark: {}", what()));
                    }
                }
                t += STEP_MS;
            }
        }
    }
    println!(
        "Glow: {frames} frames, {glows} glow strokes, {} problems",
        problems.len()
    );
    assert!(glows > 1_000, "{glows} glow strokes");
    let first: Vec<&String> = problems.iter().take(20).collect();
    assert!(problems.is_empty(), "{first:#?}");
}

/// V6.2 (P202, D-481): the labeled scenes with a model's plan of emphases in
/// place of their rules plans: from the first word, the next word at least
/// 900 ms after the last marked one (the kind's gap is 800 ms), up to the
/// kind's 12 a clip, colour, underline and highlight in turn; in the four
/// styles, at the three placements, watermark on, at every 33 ms. No
/// underline or highlight leaves the safe area or meets the watermark.
#[test]
fn no_emphasis_of_any_labeled_scene_leaves_the_safe_area_or_meets_the_watermark() {
    use offcut_types::{
        EmphasisSize, EventId, ItemParams, PlanItem, Treatment, WordIdx, WordRange,
    };
    let treatments = [
        Treatment::Colour,
        Treatment::Underline,
        Treatment::Highlight,
    ];
    let styles = [StyleId::Clean, StyleId::Bold, StyleId::Tech, StyleId::Glow];
    let (mut marked, mut rects, mut frames) = (0u32, 0u64, 0u64);
    let mut problems: Vec<String> = Vec::new();
    for style in styles {
        let theme = Theme::default_for(style);
        let mark = watermark::bounds(&watermark::layout(&theme, &FontSet::embedded()).unwrap());
        for name in &names() {
            for p in PLACES {
                let case = labeled::case(name, |edit, _| {
                    edit.style = style;
                    edit.caption_position = CaptionPlacement::new(p).unwrap();
                });
                let mut items = Vec::new();
                let mut last: Option<u32> = None;
                for (i, word) in case.transcript.words.iter().enumerate() {
                    let start = word.start_ms.get();
                    if items.len() == 12 || last.is_some_and(|at| start < at + 900) {
                        continue;
                    }
                    let at = WordIdx::new(i as u32);
                    items.push(PlanItem {
                        id: EventId::new(9_300 + i as u64),
                        kind: EventKind::Emphasis,
                        anchors: WordRange {
                            start: at,
                            end: WordIdx::new(i as u32 + 1),
                        },
                        params: ItemParams::Emphasis {
                            word: at,
                            treatment: treatments[items.len() % 3],
                            size: EmphasisSize::Normal,
                        },
                        confidence: None,
                    });
                    last = Some(start);
                }
                let case = model::with_model_items(case, items);
                let scene = scenes::build_with(&case, 1080, true);
                marked += scene.summary().captions_emphasized;
                let mut t = 0;
                while t < scene.duration().get() {
                    frames += 1;
                    for cmd in &scene.frame_at(OutMs::new(t)).cmds {
                        let Some((r, is_mark)) = bounds(cmd) else {
                            continue;
                        };
                        if matches!(cmd, DrawCmd::FillRect { color, .. } if *color == theme.accent)
                        {
                            rects += 1;
                        }
                        let what =
                            || format!("{name} {style:?} at {p}, {t} ms: {r:?} of {}", kind(cmd));
                        if !inside(r, SAFE_AREA) {
                            problems.push(format!("outside the safe area: {}", what()));
                        }
                        if !is_mark && overlap(r, mark) {
                            problems.push(format!("on the watermark: {}", what()));
                        }
                    }
                    t += STEP_MS;
                }
            }
        }
    }
    println!(
        "Emphasis: {marked} marks drawn, {frames} frames, {rects} rectangles in the accent, {} problems",
        problems.len()
    );
    assert!(marked > 500 && rects > 1_000, "{marked} {rects}");
    let first: Vec<&String> = problems.iter().take(20).collect();
    assert!(problems.is_empty(), "{first:#?}");
}

/// V6.2 (P203, D-482): the labeled scenes with a model's plan of punch
/// words: from the first word, the next starting at least 6.5 s after the
/// last one ends (D-452's gap is 6 s), up to D-452's 4 a clip, colour,
/// underline and highlight in turn; in the four styles, at the three
/// placements, watermark on, at every 33 ms. No punch word, at 1.9 times
/// the style's size, nor what is drawn with it, leaves the safe area or
/// meets the watermark.
#[test]
fn no_punch_word_of_any_labeled_scene_leaves_the_safe_area_or_meets_the_watermark() {
    use offcut_types::{
        EmphasisSize, EventId, ItemParams, PlanItem, Treatment, WordIdx, WordRange,
    };
    let treatments = [
        Treatment::Colour,
        Treatment::Underline,
        Treatment::Highlight,
    ];
    let styles = [StyleId::Clean, StyleId::Bold, StyleId::Tech, StyleId::Glow];
    let (mut punches, mut punched_frames, mut frames) = (0usize, 0u64, 0u64);
    let mut problems: Vec<String> = Vec::new();
    for style in styles {
        let theme = Theme::default_for(style);
        let mark = watermark::bounds(&watermark::layout(&theme, &FontSet::embedded()).unwrap());
        for name in &names() {
            for p in PLACES {
                let case = labeled::case(name, |edit, _| {
                    edit.style = style;
                    edit.caption_position = CaptionPlacement::new(p).unwrap();
                });
                let mut items = Vec::new();
                let mut last_end: Option<u32> = None;
                for (i, word) in case.transcript.words.iter().enumerate() {
                    let start = word.start_ms.get();
                    if items.len() == 4 || last_end.is_some_and(|end| start < end + 6_500) {
                        continue;
                    }
                    let at = WordIdx::new(i as u32);
                    items.push(PlanItem {
                        id: EventId::new(9_400 + i as u64),
                        kind: EventKind::Emphasis,
                        anchors: WordRange {
                            start: at,
                            end: WordIdx::new(i as u32 + 1),
                        },
                        params: ItemParams::Emphasis {
                            word: at,
                            treatment: treatments[items.len() % 3],
                            size: EmphasisSize::Punch,
                        },
                        confidence: None,
                    });
                    last_end = Some(word.end_ms.get());
                }
                let case = model::with_model_items(case, items);
                punches += case.plan.items().len();
                let scene = scenes::build_with(&case, 1080, true);
                let mut t = 0;
                while t < scene.duration().get() {
                    frames += 1;
                    let list = scene.frame_at(OutMs::new(t));
                    // A punch word's runs are drawn at 1.9 times their size
                    // or near it, which no other run of the captions is.
                    let big = list.cmds.iter().any(|cmd| {
                        matches!(cmd, DrawCmd::GlyphRun { transform, text, .. }
                            if transform.m[0] > 1.5 && text != WATERMARK_TEXT)
                    });
                    punched_frames += u64::from(big);
                    for cmd in &list.cmds {
                        let Some((r, is_mark)) = bounds(cmd) else {
                            continue;
                        };
                        let what =
                            || format!("{name} {style:?} at {p}, {t} ms: {r:?} of {}", kind(cmd));
                        if !inside(r, SAFE_AREA) {
                            problems.push(format!("outside the safe area: {}", what()));
                        }
                        if !is_mark && overlap(r, mark) {
                            problems.push(format!("on the watermark: {}", what()));
                        }
                    }
                    t += STEP_MS;
                }
            }
        }
    }
    println!(
        "Punch words: {punches} kept, {punched_frames} of {frames} frames with one, {} problems",
        problems.len()
    );
    assert!(
        punches > 400 && punched_frames > 5_000,
        "{punches} {punched_frames}"
    );
    let first: Vec<&String> = problems.iter().take(20).collect();
    assert!(problems.is_empty(), "{first:#?}");
}

/// `bounds`, and an image's rectangle (V6.2, P204: the scene draws images).
fn bounds_with_images(cmd: &DrawCmd) -> Option<(Rect, bool)> {
    if let DrawCmd::Image { rect, .. } = cmd {
        Some((*rect, false))
    } else {
        bounds(cmd)
    }
}

/// V6.2 (P204, D-483): the labeled scenes with a model's plan of emoji and
/// icons: from the first word, the next starting at least 2.7 s after the
/// last (D-451's window and each kind's gap are 2.5 s), an emoji under its
/// word, an emoji beside it and an icon in turn, through every emoji and
/// icon; in the four styles, at the three placements, watermark on, at every
/// 33 ms. **No image, nor an icon's disc, leaves the safe area, meets the
/// watermark, or covers a caption word.**
#[test]
fn no_emoji_or_icon_of_any_labeled_scene_leaves_the_safe_area_or_meets_the_watermark() {
    use offcut_types::{
        EmojiId, EmojiPlace, EventId, IconId, ItemParams, PlanItem, WordIdx, WordRange,
    };
    let styles = [StyleId::Clean, StyleId::Bold, StyleId::Tech, StyleId::Glow];
    let (mut kept, mut images, mut frames, mut n) = (0usize, 0u64, 0u64, 0usize);
    let mut problems: Vec<String> = Vec::new();
    for style in styles {
        let theme = Theme::default_for(style);
        let mark = watermark::bounds(&watermark::layout(&theme, &FontSet::embedded()).unwrap());
        for name in &names() {
            for p in PLACES {
                let case = labeled::case(name, |edit, _| {
                    edit.style = style;
                    edit.caption_position = CaptionPlacement::new(p).unwrap();
                });
                let mut items = Vec::new();
                let mut last: Option<u32> = None;
                for (i, word) in case.transcript.words.iter().enumerate() {
                    let start = word.start_ms.get();
                    if last.is_some_and(|at| start < at + 2_700) {
                        continue;
                    }
                    let at = WordIdx::new(i as u32);
                    let (kind, params) = match n % 3 {
                        0 | 1 => (
                            EventKind::Emoji,
                            ItemParams::Emoji {
                                word: at,
                                emoji: EmojiId::ALL[n % EmojiId::ALL.len()],
                                place: if n % 3 == 0 {
                                    EmojiPlace::Under
                                } else {
                                    EmojiPlace::Beside
                                },
                            },
                        ),
                        _ => (
                            EventKind::IconPop,
                            ItemParams::IconPop {
                                word: at,
                                icon: IconId::ALL[n % IconId::ALL.len()],
                            },
                        ),
                    };
                    n += 1;
                    items.push(PlanItem {
                        id: EventId::new(9_500 + i as u64),
                        kind,
                        anchors: WordRange {
                            start: at,
                            end: WordIdx::new(i as u32 + 1),
                        },
                        params,
                        confidence: None,
                    });
                    last = Some(start);
                }
                let case = model::with_model_items(case, items);
                kept += case.plan.items().len();
                let scene = scenes::build_with(&case, 1080, true);
                let mut t = 0;
                while t < scene.duration().get() {
                    frames += 1;
                    let list = scene.frame_at(OutMs::new(t));
                    // The caption words' runs (not a glow's clear strokes).
                    let words: Vec<Rect> = list
                        .cmds
                        .iter()
                        .filter(|cmd| {
                            matches!(cmd, DrawCmd::GlyphRun { fill, text, .. }
                                if fill.a > 0 && text != WATERMARK_TEXT)
                        })
                        .filter_map(|cmd| bounds(cmd).map(|(r, _)| r))
                        .collect();
                    for cmd in &list.cmds {
                        let Some((r, is_mark)) = bounds_with_images(cmd) else {
                            continue;
                        };
                        let what =
                            || format!("{name} {style:?} at {p}, {t} ms: {r:?} of {}", kind(cmd));
                        if !inside(r, SAFE_AREA) {
                            problems.push(format!("outside the safe area: {}", what()));
                        }
                        if !is_mark && overlap(r, mark) {
                            problems.push(format!("on the watermark: {}", what()));
                        }
                        if let DrawCmd::Image { .. } = cmd {
                            images += 1;
                            if words.iter().any(|w| overlap(*w, r)) {
                                problems.push(format!("on a caption word: {}", what()));
                            }
                        }
                    }
                    t += STEP_MS;
                }
            }
        }
    }
    println!(
        "Emoji and icons: {kept} kept, {images} images drawn in {frames} frames, {} problems",
        problems.len()
    );
    assert!(kept > 1_000 && images > 50_000, "{kept} {images}");
    let first: Vec<&String> = problems.iter().take(20).collect();
    assert!(problems.is_empty(), "{first:#?}");
}
