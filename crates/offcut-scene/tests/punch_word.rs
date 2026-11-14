//! Punch words in a scene (v6.2implementation D-482, D-452; TS §47.3): a
//! model's plan with an `Emphasis` of size `Punch`, over written sentences.
//! The word is shown alone, in capitals, at 1.9 times the style's size; the
//! reading floor holds it as any chunk, never past the next chunk or the
//! clip's end; two 6 s apart are both alone, a third closer is dropped; only
//! its own sentence is chunked anew; switched off or hidden, nothing changes.

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

use offcut_scene::Scene;
use offcut_types::{
    EmphasisSize, EventId, EventKind, ItemParams, OutMs, PlanItem, StyleId, Treatment, WordIdx,
    WordRange,
};

const STYLES: [StyleId; 4] = [StyleId::Clean, StyleId::Bold, StyleId::Tech, StyleId::Glow];

fn emphasis(id: u64, word: u32, size: EmphasisSize) -> PlanItem {
    PlanItem {
        id: EventId::new(id),
        kind: EventKind::Emphasis,
        anchors: WordRange {
            start: WordIdx::new(word),
            end: WordIdx::new(word + 1),
        },
        params: ItemParams::Emphasis {
            word: WordIdx::new(word),
            treatment: Treatment::Colour,
            size,
        },
        confidence: None,
    }
}

/// `sentences` in `style` with a model's plan of `items`, validated.
fn case(style: StyleId, sentences: &[&str], items: Vec<PlanItem>) -> scenes::Case {
    let case = scenes::written("punch", sentences, BTreeMap::new(), |edit, _| {
        edit.style = style;
    });
    model::with_model_items(case, items)
}

/// The index of the word written `text`.
fn index(case: &scenes::Case, text: &str) -> u32 {
    let words = &case.transcript.words;
    let found = words.iter().position(|w| w.text == text);
    found.unwrap_or_else(|| panic!("{}: no {text:?}", case.name)) as u32
}

/// Each chunk's transcript words.
fn ranges(scene: &Scene) -> Vec<(u32, u32)> {
    let ranges = scene.chunk_ranges();
    ranges
        .iter()
        .map(|(r, _)| (r.start.get(), r.end.get()))
        .collect()
}

/// The program times, to the millisecond, at which `word`'s chunk is the one
/// shown and holds it alone: its first and its last.
fn alone(scene: &Scene, word: u32) -> (u32, u32) {
    let shown: Vec<u32> = (0..scene.duration().get())
        .filter(|&t| {
            let at = OutMs::new(t);
            scene.caption_words(at).len() == 1 && scene.word_box(at, WordIdx::new(word)).is_some()
        })
        .collect();
    let (first, last) = (shown[0], *shown.last().unwrap());
    assert_eq!(shown.len() as u32, last - first + 1, "one span");
    (first, last)
}

/// "It felt *brutal*." then a sentence: "brutal." is followed by the
/// sentence end's 400 ms of silence.
const AT_THE_END: [&str; 2] = ["It felt brutal.", "Then it got better every week."];
/// "brutal" followed at once (80 ms) by "for".
const IN_THE_MIDDLE: [&str; 2] = [
    "It felt brutal for everyone.",
    "Then it got better every week.",
];

#[test]
fn the_reading_floor_holds_a_punch_word_when_silence_follows_and_no_word_moves() {
    for style in STYLES {
        // Silence after it: held to the 700 ms floor (D-168).
        let base = case(style, &AT_THE_END, Vec::new());
        let word = index(&base, "brutal.");
        let held = case(
            style,
            &AT_THE_END,
            vec![emphasis(1, word, EmphasisSize::Punch)],
        );
        let scene = scenes::build(&held, 1080);
        let (first, last) = alone(&scene, word);
        let spoken = &held.transcript.words[word as usize];
        assert!(
            first < spoken.start_ms.get(),
            "{style:?}: shown before it is said"
        );
        assert_eq!(last + 1 - first, 700, "{style:?}: the floor");
        assert!(
            last + 1 >= spoken.end_ms.get() + 120,
            "{style:?}: and its tail"
        );
        // In capitals, whatever the style's case.
        let at = OutMs::new(spoken.start_ms.get());
        assert_eq!(scene.caption_words(at)[0].0, "BRUTAL.", "{style:?}");

        // The next word at once: shown until the next chunk, under the floor;
        // every word still shown whole, from before it starts to its end.
        let close = case(style, &IN_THE_MIDDLE, Vec::new());
        let word = index(&close, "brutal");
        let close = case(
            style,
            &IN_THE_MIDDLE,
            vec![emphasis(1, word, EmphasisSize::Punch)],
        );
        let scene = scenes::build(&close, 1080);
        let (first, last) = alone(&scene, word);
        let (spoken, next) = (
            &close.transcript.words[word as usize],
            &close.transcript.words[word as usize + 1],
        );
        assert!(last + 1 - first < 700, "{style:?}");
        assert!(first < spoken.start_ms.get() && last + 1 >= spoken.end_ms.get());
        let next_at = OutMs::new(last + 1);
        assert!(scene.word_box(next_at, WordIdx::new(word + 1)).is_some());
        assert!(
            last < next.start_ms.get(),
            "{style:?}: the next before it is said"
        );
    }
}

/// Four sentences of about 3.7 s each: room for punch words 6 s apart.
const LONG: [&str; 4] = [
    "We started this company in a small apartment with two laptops.",
    "Nobody believed the product would work for real customers.",
    "Then one morning everything changed for the whole team.",
    "Today we serve thousands of people every single day.",
];

#[test]
fn two_punch_words_6_s_apart_are_each_alone_and_a_third_closer_is_dropped() {
    let base = case(StyleId::Clean, &LONG, Vec::new());
    let words = &base.transcript.words;
    let start = |i: u32| words[i as usize].start_ms.get();
    let end = |i: u32| words[i as usize].end_ms.get();
    // D-452 counts the 6 s from the end of the punch word before.
    let first = index(&base, "company");
    let second = (first..words.len() as u32)
        .find(|&i| start(i) >= end(first) + 6_000)
        .unwrap();
    let third = (second..words.len() as u32)
        .find(|&i| start(i) >= end(second) + 3_000)
        .unwrap();
    assert!(start(third) < end(second) + 6_000);
    let items = vec![
        emphasis(1, first, EmphasisSize::Punch),
        emphasis(2, second, EmphasisSize::Punch),
        emphasis(3, third, EmphasisSize::Punch),
    ];
    for style in STYLES {
        let punched = case(style, &LONG, items.clone());
        // D-452: the third is 3 s after the second, under its 6 s.
        let kept: Vec<u64> = punched.plan.items().iter().map(|i| i.id.get()).collect();
        assert_eq!(kept, [1, 2], "{style:?}");
        let scene = scenes::build(&punched, 1080);
        let all = ranges(&scene);
        for word in [first, second] {
            assert!(
                all.contains(&(word, word + 1)),
                "{style:?}: {word} alone in {all:?}"
            );
            alone(&scene, word);
        }
        assert!(
            !all.contains(&(third, third + 1)) || {
                // Unless the break search had already set it alone.
                let plain = scenes::build(&case(style, &LONG, Vec::new()), 1080);
                ranges(&plain).contains(&(third, third + 1))
            }
        );
        assert_eq!(scene.summary().captions_emphasized, 2, "{style:?}");
    }
}

#[test]
fn a_punch_word_at_the_clips_last_word_is_the_last_chunk_and_ends_by_the_clips_end() {
    for style in STYLES {
        let base = case(style, &AT_THE_END, Vec::new());
        let last_word = base.transcript.words.len() as u32 - 1;
        let punched = case(
            style,
            &AT_THE_END,
            vec![emphasis(1, last_word, EmphasisSize::Punch)],
        );
        let scene = scenes::build(&punched, 1080);
        let all = ranges(&scene);
        assert_eq!(all.last(), Some(&(last_word, last_word + 1)), "{style:?}");
        let (first, last) = alone(&scene, last_word);
        let end = scene.duration().get();
        // Held to the floor, or to the clip's end when that comes first.
        assert_eq!(last + 1, (first + 700).min(end), "{style:?}");
        assert!(last < end);
        assert_eq!(scene.caption_words(OutMs::new(last))[0].0, "WEEK.");
    }
}

#[test]
fn only_the_punch_words_sentence_is_chunked_anew_and_a_normal_emphasis_changes_none() {
    for style in STYLES {
        let plain = case(style, &LONG, Vec::new());
        let word = index(&plain, "believed");
        let normal = case(style, &LONG, vec![emphasis(1, word, EmphasisSize::Normal)]);
        let punch = case(style, &LONG, vec![emphasis(1, word, EmphasisSize::Punch)]);
        let (plain, normal, punch) = (
            ranges(&scenes::build(&plain, 1080)),
            ranges(&scenes::build(&normal, 1080)),
            ranges(&scenes::build(&punch, 1080)),
        );
        assert_eq!(
            normal, plain,
            "{style:?}: a normal emphasis chunks as before"
        );
        // "believed" is in the second sentence, words 11 to 19.
        let sentence = 11..20;
        let outside = |all: &[(u32, u32)]| -> Vec<(u32, u32)> {
            let keep = |r: &&(u32, u32)| !sentence.contains(&r.0);
            all.iter().filter(keep).copied().collect()
        };
        assert_eq!(outside(&punch), outside(&plain), "{style:?}");
        assert_ne!(punch, plain, "{style:?}");
        assert!(punch.contains(&(word, word + 1)), "{style:?}");
    }
}

#[test]
fn switched_off_or_hidden_a_punch_word_changes_nothing() {
    for style in STYLES {
        let base = case(style, &IN_THE_MIDDLE, Vec::new());
        let word = index(&base, "brutal");
        let items = vec![emphasis(1, word, EmphasisSize::Punch)];
        let none = scenes::build(&case(style, &IN_THE_MIDDLE, vec![]), 1080);
        // Switched off.
        let mut off = case(style, &IN_THE_MIDDLE, items.clone());
        off.edit.event_overrides.insert(EventId::new(1), false);
        let off = scenes::build(&off, 1080);
        // Hidden by an edit made after the plan.
        let mut hidden = case(style, &IN_THE_MIDDLE, items.clone());
        hidden.edit.word_edits = BTreeMap::from([(WordIdx::new(word), String::new())]);
        let mut bare = case(style, &IN_THE_MIDDLE, vec![]);
        bare.edit.word_edits = hidden.edit.word_edits.clone();
        let (hidden, bare) = (scenes::build(&hidden, 1080), scenes::build(&bare, 1080));
        assert_eq!(ranges(&off), ranges(&none), "{style:?}");
        assert_eq!(ranges(&hidden), ranges(&bare), "{style:?}");
        for t in (0..none.duration().get()).step_by(33) {
            let t = OutMs::new(t);
            assert_eq!(off.frame_at(t), none.frame_at(t), "{style:?} {t:?}");
            assert_eq!(hidden.frame_at(t), bare.frame_at(t), "{style:?} {t:?}");
        }
        assert_eq!(off.summary().captions_emphasized, 0);
        assert_eq!(hidden.summary().captions_emphasized, 0);
        // On, it counts as one emphasis of the captions.
        let on = scenes::build(&case(style, &IN_THE_MIDDLE, items), 1080);
        let counted = on.summary();
        assert_eq!(
            (counted.captions_emphasized, counted.visual_moments),
            (1, 0)
        );
    }
}
