//! Emphasis in a scene (v6.2implementation D-481, TS §47.3): a model's plan
//! with one emphasis, over written sentences, read at every frame in the
//! four styles. The captions mark the word from its program start until its
//! chunk ends, and only in its own chunk; a word an edit hid is not marked;
//! switched off, it is gone; only the captions change; it counts as
//! "captions emphasized"; a punch word is marked at the captions' size until
//! P203 gives it its own chunk.

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

use offcut_scene::safe_area::SAFE_AREA;
use offcut_scene::{DisplayList, DrawCmd, Rect, Scene};
use offcut_types::{
    EmphasisSize, EventId, EventKind, ItemParams, OutMs, PlanItem, StyleId, Treatment, WordIdx,
    WordRange,
};

const ID: u64 = 9_200;
const STYLES: [StyleId; 4] = [StyleId::Clean, StyleId::Bold, StyleId::Tech, StyleId::Glow];
const TREATMENTS: [Treatment; 3] = [
    Treatment::Colour,
    Treatment::Underline,
    Treatment::Highlight,
];

/// "week" is said twice: word 2 of the first sentence, word 14 of the last,
/// in two chunks in every style.
const SENTENCES: [&str; 2] = [
    "The first week felt brutal for everyone.",
    "Thanks for watching, and see you next week.",
];
const WEEK: u32 = 2;
const BRUTAL: u32 = 4;
const LAST_WEEK: u32 = 14;

fn emphasis(word: u32, treatment: Treatment, size: EmphasisSize) -> PlanItem {
    PlanItem {
        id: EventId::new(ID),
        kind: EventKind::Emphasis,
        anchors: WordRange {
            start: WordIdx::new(word),
            end: WordIdx::new(word + 1),
        },
        params: ItemParams::Emphasis {
            word: WordIdx::new(word),
            treatment,
            size,
        },
        confidence: None,
    }
}

/// The sentences in `style` with a model's plan of `items`, validated.
fn case(style: StyleId, items: Vec<PlanItem>) -> scenes::Case {
    let case = scenes::written("emphasis", &SENTENCES, BTreeMap::new(), |edit, _| {
        edit.style = style;
    });
    let count = items.len();
    let case = model::with_model_items(case, items);
    assert_eq!(
        case.plan.items().len(),
        count,
        "{}: every item passed validation",
        case.name
    );
    case
}

/// The case with one emphasis, and the same case with a model's plan of none.
fn pair(style: StyleId, word: u32, treatment: Treatment) -> (scenes::Case, scenes::Case) {
    let with = case(style, vec![emphasis(word, treatment, EmphasisSize::Normal)]);
    (with, case(style, Vec::new()))
}

/// Every frame time of the case at 30 fps.
fn frames(case: &scenes::Case) -> impl Iterator<Item = u32> {
    (0..case.length * 30 / 1000).map(|n| n * 1000 / 30)
}

fn at(scene: &Scene, t: u32) -> DisplayList {
    scene.frame_at(OutMs::new(t))
}

/// The emphasis' program start.
fn start(case: &scenes::Case) -> u32 {
    case.plan.items()[0].span.start.get()
}

#[test]
fn the_word_is_marked_from_its_start_until_its_chunk_ends_and_only_in_its_own_chunk() {
    for style in STYLES {
        for treatment in [Treatment::Underline, Treatment::Highlight] {
            let (with, without) = pair(style, WEEK, treatment);
            let from = start(&with);
            assert_eq!(from, with.transcript.words[WEEK as usize].start_ms.get());
            let (a, b) = (scenes::build(&with, 1080), scenes::build(&without, 1080));
            let (mut marked, mut other_week) = (0, 0);
            for t in frames(&with) {
                // Its chunk is the one shown that holds word 2.
                let shown = a.word_box(OutMs::new(t), WordIdx::new(WEEK)).is_some();
                let differs = at(&a, t) != at(&b, t);
                assert_eq!(
                    differs,
                    shown && t >= from,
                    "{style:?} {treatment:?} at {t}"
                );
                marked += u32::from(differs);
                // The second "week", in another chunk, is never marked.
                if a.word_box(OutMs::new(t), WordIdx::new(LAST_WEEK)).is_some() {
                    other_week += 1;
                    assert!(!differs, "{style:?} {treatment:?} at {t}");
                }
            }
            assert!(
                marked > 5 && other_week > 5,
                "{style:?}: {marked} {other_week}"
            );
        }
    }
}

#[test]
fn a_coloured_word_is_in_the_accent_and_the_same_word_in_another_chunk_is_not() {
    for style in STYLES {
        let (with, without) = pair(style, WEEK, Treatment::Colour);
        let (a, b) = (scenes::build(&with, 1080), scenes::build(&without, 1080));
        let theme = offcut_scene::Theme::default_for(style);
        let week = &with.transcript.words[WEEK as usize];
        // From its start, while its chunk shows: in the accent; while it is
        // said in Clean, on the pill, in ink.
        let mut seen = 0;
        for t in frames(&with) {
            let t = OutMs::new(t);
            let words = a.caption_words(t);
            if a.word_box(t, WordIdx::new(WEEK)).is_some() && t.get() >= week.start_ms.get() {
                let found = words
                    .iter()
                    .find(|(text, _)| text.eq_ignore_ascii_case("week"))
                    .unwrap();
                let spoken = t.get() < week.end_ms.get();
                let want = if spoken && style == StyleId::Clean {
                    theme.on_accent
                } else {
                    theme.accent
                };
                assert_eq!(found.1, want, "{style:?} at {t:?}");
                seen += 1;
            }
            // In the other chunk, "week." as without the emphasis.
            if a.word_box(t, WordIdx::new(LAST_WEEK)).is_some() {
                assert_eq!(words, b.caption_words(t), "{style:?} at {t:?}");
            }
        }
        let times = (week.start_ms, week.end_ms);
        assert!(seen > 0, "{style:?}: {:?} {times:?}", a.chunk_ranges());
    }
}

#[test]
fn a_word_an_edit_hid_is_not_marked_nor_counted() {
    for style in STYLES {
        for treatment in TREATMENTS {
            // The plan was made before the edit (a model's plan is kept on
            // word edits, P193); the edit hides "brutal".
            let (mut with, mut without) = pair(style, BRUTAL, treatment);
            let hidden = BTreeMap::from([(WordIdx::new(BRUTAL), String::new())]);
            with.edit.word_edits = hidden.clone();
            without.edit.word_edits = hidden;
            let (a, b) = (scenes::build(&with, 1080), scenes::build(&without, 1080));
            for t in frames(&with) {
                assert_eq!(at(&a, t), at(&b, t), "{style:?} {treatment:?} at {t}");
            }
            let summary = a.summary();
            assert_eq!(
                (summary.captions_emphasized, summary.visual_moments),
                (0, 0)
            );
        }
    }
}

#[test]
fn switched_off_it_is_gone_and_on_it_counts_as_captions_emphasized() {
    for treatment in TREATMENTS {
        let (with, without) = pair(StyleId::Bold, BRUTAL, treatment);
        let mut off = case(
            StyleId::Bold,
            vec![emphasis(BRUTAL, treatment, EmphasisSize::Normal)],
        );
        off.edit.event_overrides.insert(EventId::new(ID), false);
        let on = scenes::build(&with, 1080);
        let (off, none) = (scenes::build(&off, 1080), scenes::build(&without, 1080));
        for t in frames(&with) {
            assert_eq!(at(&off, t), at(&none, t), "{treatment:?} at {t}");
        }
        let counted = on.summary();
        assert_eq!(
            (counted.captions_emphasized, counted.visual_moments),
            (1, 0)
        );
        assert_eq!(off.summary().captions_emphasized, 0);
    }
}

/// The glyph runs of a list: text and transform.
fn placed(list: &DisplayList) -> Vec<(String, [f32; 6])> {
    list.cmds
        .iter()
        .filter_map(|cmd| {
            if let DrawCmd::GlyphRun {
                text, transform, ..
            } = cmd
            {
                Some((text.clone(), transform.m))
            } else {
                None
            }
        })
        .collect()
}

fn rects(list: &DisplayList) -> Vec<Rect> {
    list.cmds
        .iter()
        .filter_map(|cmd| {
            if let DrawCmd::FillRect { rect, .. } = cmd {
                Some(*rect)
            } else {
                None
            }
        })
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

#[test]
fn only_the_captions_change_the_video_and_every_word_stay_where_they_were() {
    for style in STYLES {
        for treatment in TREATMENTS {
            let (with, without) = pair(style, BRUTAL, treatment);
            let (a, b) = (scenes::build(&with, 1080), scenes::build(&without, 1080));
            let brutal = &with.transcript.words[BRUTAL as usize].text;
            for t in frames(&with) {
                let (x, y) = (at(&a, t), at(&b, t));
                let what = format!("{style:?} {treatment:?} at {t}");
                assert_eq!(x.cmds[0], y.cmds[0], "{what}: the video");
                // Every run is where it was; only "brutal"'s shadow and glow
                // may be gone (a highlighted word has none).
                let (runs, before) = (placed(&x), placed(&y));
                for run in &runs {
                    assert!(before.contains(run), "{what}: {run:?} moved");
                }
                for run in before.iter().filter(|run| !runs.contains(run)) {
                    assert!(run.0.eq_ignore_ascii_case(brutal), "{what}: {run:?} gone");
                }
                // Every new rectangle is on "brutal", in the safe area.
                let old = rects(&y);
                let word = a.word_box(OutMs::new(t), WordIdx::new(BRUTAL));
                for r in rects(&x).into_iter().filter(|r| !old.contains(r)) {
                    let word = word.unwrap_or_else(|| panic!("{what}: {r:?} with no word"));
                    assert!(overlap(r, word), "{what}: {r:?} off {word:?}");
                    assert!(inside(r, SAFE_AREA), "{what}: {r:?}");
                }
            }
        }
    }
}

#[test]
fn a_punch_word_is_marked_at_the_captions_size_until_p203() {
    for treatment in TREATMENTS {
        let normal = case(
            StyleId::Bold,
            vec![emphasis(BRUTAL, treatment, EmphasisSize::Normal)],
        );
        let punch = case(
            StyleId::Bold,
            vec![emphasis(BRUTAL, treatment, EmphasisSize::Punch)],
        );
        let (a, b) = (scenes::build(&normal, 1080), scenes::build(&punch, 1080));
        for t in frames(&normal) {
            assert_eq!(at(&a, t), at(&b, t), "{treatment:?} at {t}");
        }
    }
}
