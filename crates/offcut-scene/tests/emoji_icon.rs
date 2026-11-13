//! Emoji and icons in a scene (v6.2implementation D-483, TS §47.3): a
//! model's plan over written sentences, read at every frame in the four
//! styles and at the three snap points. An emoji is 1.6 capitals high,
//! under its word below the lines, over them where there is no room under
//! them (Bottom), beside its line when the line leaves room; an icon is on
//! an accent disc in the event zone, centred on its word. Both pop in at
//! their word and leave with its chunk; only images (and the disc) are
//! added to a frame; hidden or switched off, nothing is drawn or counted.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

#[path = "common/model.rs"]
mod model;
#[path = "common/scenes.rs"]
mod scenes;

use std::collections::BTreeMap;

use offcut_scene::events::{IMAGE_POP_FROM, IMAGE_POP_MS};
use offcut_scene::layout::cap_height;
use offcut_scene::placement::{SNAP_BOTTOM, SNAP_MIDDLE, SNAP_TOP, WATERMARK_BAND, event_zone};
use offcut_scene::safe_area::SAFE_AREA;
use offcut_scene::styles::spec;
use offcut_scene::{DisplayList, DrawCmd, FontSet, ImageKey, Rect, Rgba, Scene, Theme};
use offcut_types::{
    CaptionPlacement, EmojiId, EmojiPlace, EmphasisSize, EventId, EventKind, IconId, ItemParams,
    OutMs, PlanItem, StyleId, Treatment, WordIdx, WordRange,
};

const STYLES: [StyleId; 4] = [StyleId::Clean, StyleId::Bold, StyleId::Tech, StyleId::Glow];
const PLACES: [f32; 3] = [SNAP_TOP, SNAP_MIDDLE, SNAP_BOTTOM];
const SENTENCES: [&str; 2] = [
    "The first week felt brutal for everyone.",
    "Thanks for watching, and see you next week.",
];
/// "brutal".
const WORD: u32 = 4;
/// "see", 3.4 s after "brutal".
const SEE: u32 = 11;

fn anchors(word: u32) -> WordRange {
    WordRange {
        start: WordIdx::new(word),
        end: WordIdx::new(word + 1),
    }
}

fn emoji(word: u32, place: EmojiPlace) -> PlanItem {
    PlanItem {
        id: EventId::new(1),
        kind: EventKind::Emoji,
        anchors: anchors(word),
        params: ItemParams::Emoji {
            word: WordIdx::new(word),
            emoji: EmojiId::Fire,
            place,
        },
        confidence: None,
    }
}

fn icon(word: u32) -> PlanItem {
    PlanItem {
        id: EventId::new(2),
        kind: EventKind::IconPop,
        anchors: anchors(word),
        params: ItemParams::IconPop {
            word: WordIdx::new(word),
            icon: IconId::Lightbulb,
        },
        confidence: None,
    }
}

/// `sentences` in `style` at `place`, with a model's plan of `items`,
/// every item kept by the validation.
fn case(style: StyleId, place: f32, sentences: &[&str], items: Vec<PlanItem>) -> scenes::Case {
    let case = scenes::written("emoji", sentences, BTreeMap::new(), |edit, _| {
        edit.style = style;
        edit.caption_position = CaptionPlacement::new(place).unwrap();
    });
    let count = items.len();
    let case = model::with_model_items(case, items);
    assert_eq!(case.plan.items().len(), count, "{}: all kept", case.name);
    case
}

fn images(list: &DisplayList) -> Vec<(ImageKey, Rect)> {
    let pick = |cmd: &DrawCmd| {
        if let DrawCmd::Image { image, rect, .. } = cmd {
            Some((*image, *rect))
        } else {
            None
        }
    };
    list.cmds.iter().filter_map(pick).collect()
}

/// The glyph boxes of the words of the chunk shown at `t`.
fn shown_boxes(scene: &Scene, t: OutMs) -> Vec<Rect> {
    let ranges = scene.chunk_ranges();
    let words = ranges.iter().flat_map(|(r, _)| r.start.get()..r.end.get());
    words
        .filter_map(|w| scene.word_box(t, WordIdx::new(w)))
        .collect()
}

fn overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

fn inside(r: Rect, area: Rect) -> bool {
    r.x >= area.x - 0.01
        && r.y >= area.y - 0.01
        && r.x + r.w <= area.x + area.w + 0.01
        && r.y + r.h <= area.y + area.h + 0.01
}

/// 300 ms after the word starts: the emoji at rest, its chunk shown.
fn rest(case: &scenes::Case, word: u32) -> OutMs {
    OutMs::new(case.transcript.words[word as usize].start_ms.get() + 300)
}

/// The emoji's size: 1.6 times the style's capitals.
fn emoji_size(style: StyleId) -> f32 {
    let (row, theme) = (spec(style), Theme::default_for(style));
    1.6 * cap_height(theme.font(row.font), row.size, &FontSet::embedded()).unwrap()
}

#[test]
fn an_emoji_under_its_word_is_centred_below_the_lines_or_over_them_at_bottom() {
    for style in STYLES {
        for place in PLACES {
            let case = case(
                style,
                place,
                &SENTENCES,
                vec![emoji(WORD, EmojiPlace::Under)],
            );
            let scene = scenes::build(&case, 1080);
            let t = rest(&case, WORD);
            let found = images(&scene.frame_at(t));
            assert_eq!(found.len(), 1, "{style:?} at {place}");
            let (key, r) = found[0];
            assert_eq!(key, ImageKey::of_emoji(EmojiId::Fire));
            let what = format!("{style:?} at {place}: {r:?}");
            assert!((r.w - emoji_size(style)).abs() < 1e-3, "{what}");
            let word = scene.word_box(t, WordIdx::new(WORD)).unwrap();
            assert!(
                (r.x + r.w / 2.0 - (word.x + word.w / 2.0)).abs() < 1e-3,
                "{what}"
            );
            let lines = shown_boxes(&scene, t);
            let top = lines.iter().map(|b| b.y).fold(f32::MAX, f32::min);
            let low = lines.iter().map(|b| b.y + b.h).fold(f32::MIN, f32::max);
            if place == SNAP_BOTTOM {
                // No room under the lines above the watermark's band: over them.
                assert!(r.y + r.h < top, "{what}");
            } else {
                assert!(r.y > low, "{what}");
            }
            assert!(!lines.iter().any(|b| overlap(*b, r)), "{what}");
            let band = SAFE_AREA.y + SAFE_AREA.h - WATERMARK_BAND;
            assert!(inside(r, SAFE_AREA) && r.y + r.h <= band + 0.01, "{what}");
        }
    }
}

#[test]
fn beside_is_at_the_end_of_its_line_when_the_line_leaves_room_else_as_under() {
    for style in STYLES {
        // "Wow." alone on its line: room beside it.
        let short = ["Wow.", "Thanks for watching, and see you next week."];
        let wow = case(
            style,
            SNAP_MIDDLE,
            &short,
            vec![emoji(0, EmojiPlace::Beside)],
        );
        let scene = scenes::build(&wow, 1080);
        let t = rest(&wow, 0);
        let (_, r) = images(&scene.frame_at(t))[0];
        let word = scene.word_box(t, WordIdx::new(0)).unwrap();
        assert!(r.x > word.x + word.w, "{style:?}: {r:?} {word:?}");
        // Centred on the capitals: inside the word's height.
        let middle = r.y + r.h / 2.0;
        assert!(middle > word.y && middle < word.y + word.h, "{style:?}");
        assert!(inside(r, SAFE_AREA), "{style:?}");
        // A line that fills the block: no room, so under (or over) it.
        let long = [
            "Internationalization internationalization.",
            "Thanks for watching, and see you next week.",
        ];
        let full = case(
            style,
            SNAP_MIDDLE,
            &long,
            vec![emoji(0, EmojiPlace::Beside)],
        );
        let scene = scenes::build(&full, 1080);
        let t = rest(&full, 0);
        let (_, r) = images(&scene.frame_at(t))[0];
        let lines = shown_boxes(&scene, t);
        assert!(!lines.iter().any(|b| overlap(*b, r)), "{style:?}: {r:?}");
        let low = lines.iter().map(|b| b.y + b.h).fold(f32::MIN, f32::max);
        assert!(r.y > low, "{style:?}: under it, at Middle");
    }
}

#[test]
fn an_emoji_pops_in_at_its_word_and_leaves_with_its_chunk() {
    for style in STYLES {
        let case = case(
            style,
            SNAP_MIDDLE,
            &SENTENCES,
            vec![emoji(WORD, EmojiPlace::Under)],
        );
        let scene = scenes::build(&case, 1080);
        let start = case.transcript.words[WORD as usize].start_ms.get();
        let size = emoji_size(style);
        let at = |t: u32| images(&scene.frame_at(OutMs::new(t)));
        assert!(at(start - 1).is_empty(), "{style:?}");
        assert!(
            (at(start)[0].1.w - size * IMAGE_POP_FROM).abs() < 1e-2,
            "{style:?}"
        );
        assert!(
            (at(start + IMAGE_POP_MS)[0].1.w - size).abs() < 1e-3,
            "{style:?}"
        );
        // Shown exactly while its word's chunk is.
        for t in (start..case.length).step_by(7) {
            let held = scene.word_box(OutMs::new(t), WordIdx::new(WORD)).is_some();
            assert_eq!(!at(t).is_empty(), held, "{style:?} at {t}");
        }
    }
}

#[test]
fn an_icon_is_on_an_accent_disc_in_the_event_zone_centred_on_its_word() {
    for style in STYLES {
        for place in PLACES {
            let case = case(style, place, &SENTENCES, vec![icon(WORD)]);
            let scene = scenes::build(&case, 1080);
            let zone = event_zone(scene.caption_block());
            let t = rest(&case, WORD);
            let list = scene.frame_at(t);
            let found = images(&list);
            assert_eq!(found.len(), 1, "{style:?} at {place}");
            let (key, icon) = found[0];
            assert_eq!(key, ImageKey::of_icon(IconId::Lightbulb));
            let disc = list
                .cmds
                .iter()
                .find_map(|cmd| match cmd {
                    DrawCmd::FillRect {
                        rect,
                        radius,
                        color,
                    } if *color == Theme::default_for(style).accent
                        && rect.w > 150.0
                        && (radius - rect.w / 2.0).abs() < 1e-3 =>
                    {
                        Some((*rect, *radius))
                    }
                    DrawCmd::FillRect { .. }
                    | DrawCmd::FillPath { .. }
                    | DrawCmd::GlyphRun { .. }
                    | DrawCmd::PushLayer { .. }
                    | DrawCmd::PopLayer
                    | DrawCmd::Video { .. }
                    | DrawCmd::Image { .. } => None,
                })
                .unwrap();
            let what = format!("{style:?} at {place}: {disc:?}");
            assert!((disc.1 - disc.0.w / 2.0).abs() < 1e-3, "{what}: a circle");
            assert!(inside(disc.0, zone), "{what} in {zone:?}");
            assert!(inside(icon, disc.0), "{what}: the icon on it");
            let word = scene.word_box(t, WordIdx::new(WORD)).unwrap();
            let centre = disc.0.x + disc.0.w / 2.0;
            assert!((centre - (word.x + word.w / 2.0)).abs() < 1e-3, "{what}");
            // On the zone's edge by the captions.
            let block = scene.caption_block();
            let edge = if zone.y < block.y {
                disc.0.y + disc.0.h - (zone.y + zone.h)
            } else {
                disc.0.y - zone.y
            };
            assert!(edge.abs() < 1e-3, "{what}");
            // It leaves with the word's chunk.
            let end = (t.get()..case.length)
                .find(|&at| scene.word_box(OutMs::new(at), WordIdx::new(WORD)).is_none())
                .unwrap();
            assert!(
                images(&scene.frame_at(OutMs::new(end))).is_empty(),
                "{what}"
            );
            assert!(!images(&scene.frame_at(OutMs::new(end - 1))).is_empty());
        }
    }
}

/// `list` without its images and an icon's discs (an accent or a panel
/// circle 100 lp across or more). D-440 (22), P206b: widened by the panel's
/// disc inside the accent ring (D-502); nothing else changed.
fn without_images(list: &DisplayList, (accent, panel): (Rgba, Rgba)) -> Vec<DrawCmd> {
    let added = |cmd: &&DrawCmd| match cmd {
        DrawCmd::Image { .. } => false,
        DrawCmd::FillRect {
            rect,
            radius,
            color,
        } => {
            !((*color == accent || *color == panel)
                && rect.w >= 100.0
                && (radius - rect.w / 2.0).abs() < 1e-3)
        }
        DrawCmd::FillPath { .. }
        | DrawCmd::GlyphRun { .. }
        | DrawCmd::PushLayer { .. }
        | DrawCmd::PopLayer
        | DrawCmd::Video { .. } => true,
    };
    list.cmds.iter().filter(added).cloned().collect()
}

#[test]
fn only_images_and_the_disc_are_added_and_hidden_or_switched_off_nothing_is() {
    for style in STYLES {
        let theme = Theme::default_for(style);
        let discs = (theme.accent, theme.panel);
        // The icon on "see", over 2.5 s after "brutal" (D-451's window
        // for the visual kinds, and each kind's gap).
        let items = vec![emoji(WORD, EmojiPlace::Under), icon(SEE)];
        let with = case(style, SNAP_BOTTOM, &SENTENCES, items.clone());
        let none = case(style, SNAP_BOTTOM, &SENTENCES, Vec::new());
        let mut off = case(style, SNAP_BOTTOM, &SENTENCES, items.clone());
        off.edit.event_overrides.insert(EventId::new(1), false);
        off.edit.event_overrides.insert(EventId::new(2), false);
        let mut hidden = case(style, SNAP_BOTTOM, &SENTENCES, items);
        let gone = BTreeMap::from([
            (WordIdx::new(WORD), String::new()),
            (WordIdx::new(SEE), String::new()),
        ]);
        hidden.edit.word_edits = gone.clone();
        let mut bare = case(style, SNAP_BOTTOM, &SENTENCES, Vec::new());
        bare.edit.word_edits = gone;
        let [with, none, off, hidden, bare] =
            [&with, &none, &off, &hidden, &bare].map(|c| scenes::build(c, 1080));
        let mut drawn = 0;
        for t in (0..with.duration().get()).step_by(33) {
            let t = OutMs::new(t);
            let (a, b) = (with.frame_at(t), none.frame_at(t));
            assert_eq!(without_images(&a, discs), b.cmds, "{style:?} {t:?}");
            drawn += images(&a).len();
            assert_eq!(off.frame_at(t), b, "{style:?} {t:?}: switched off");
            assert_eq!(
                hidden.frame_at(t),
                bare.frame_at(t),
                "{style:?} {t:?}: hidden"
            );
        }
        assert!(drawn > 20, "{style:?}: {drawn}");
        let counted = with.summary();
        assert_eq!(
            (counted.captions_emphasized, counted.visual_moments),
            (0, 2)
        );
        assert_eq!(off.summary().visual_moments, 0);
        assert_eq!(hidden.summary().visual_moments, 0);
    }
}

#[test]
fn an_emoji_on_a_punch_word_grows_with_it_but_never_past_its_image() {
    for style in STYLES {
        let punch = PlanItem {
            id: EventId::new(3),
            kind: EventKind::Emphasis,
            anchors: anchors(WORD),
            params: ItemParams::Emphasis {
                word: WordIdx::new(WORD),
                treatment: Treatment::Colour,
                size: EmphasisSize::Punch,
            },
            confidence: None,
        };
        for place in PLACES {
            let items = vec![punch.clone(), emoji(WORD, EmojiPlace::Under)];
            let case = case(style, place, &SENTENCES, items);
            let scene = scenes::build(&case, 1080);
            let t = rest(&case, WORD);
            let (_, r) = images(&scene.frame_at(t))[0];
            let want = (emoji_size(style) * 1.9).min(160.0);
            assert!((r.w - want).abs() < 1e-2, "{style:?} at {place}: {r:?}");
            assert!(inside(r, SAFE_AREA), "{style:?} at {place}: {r:?}");
            let word = scene.word_box(t, WordIdx::new(WORD)).unwrap();
            assert!(!overlap(r, word), "{style:?} at {place}");
        }
    }
}
