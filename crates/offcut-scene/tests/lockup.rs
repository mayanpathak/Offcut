//! The hook's lockup (v6.2implementation D-484, §10.4), pure: short, long,
//! one-word and impossible hooks; each line as large as fits; the accent at
//! twice its line; the least worst distance from the width, ties to fewer
//! lines, checked against a brute force that shapes every line at every
//! size; widths that grow with the size exactly; the same answer twice.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use offcut_scene::FontSet;
use offcut_scene::events::hook_title::room;
use offcut_scene::layout::shape;
use offcut_scene::lockup::{
    ACCENT_SCALE, LOCKUP_FONT, LOCKUP_MAX, LOCKUP_MIN, Lockup, lay_out, sizes,
};

/// The room of every case: the safe area above the watermark's band.
fn room_size() -> (f32, f32) {
    let r = room();
    (r.w, r.h)
}

fn lockup(text: &str, accent: usize) -> Option<Lockup> {
    lay_out(text, accent, room_size(), &FontSet::embedded()).unwrap()
}

/// The width of `words` set in the lockup's face, each at its size, with a
/// space at `size` between them, shaped for real.
fn real_width(words: &[(String, f32)], size: f32) -> f32 {
    let fonts = FontSet::embedded();
    let width = |text: &str, s: f32| -> f32 {
        shape(text, LOCKUP_FONT, s, &fonts)
            .unwrap()
            .iter()
            .map(|r| r.width)
            .sum()
    };
    let spaces = width(" ", size) * words.len().saturating_sub(1) as f32;
    words.iter().map(|(w, s)| width(w, *s)).sum::<f32>() + spaces
}

/// Every hook of the cases: from 2 words to 9 and 48 characters.
const HOOKS: [(&str, usize); 8] = [
    ("Stop scrolling", 1),
    ("The one thing nobody tells you", 3),
    ("This changed everything for me", 2),
    ("Why most startups fail in year one", 4),
    ("I quit my job to do this", 2),
    ("The one thing nobody tells you about saving", 3),
    ("Three habits that made me a better writer", 0),
    ("You are doing it wrong", 4),
];

#[test]
fn a_shaped_width_grows_with_the_size_exactly() {
    let fonts = FontSet::embedded();
    for word in ["NOBODY", "STOP", "SCROLLING", "W", " "] {
        let at = |s: f32| -> f32 {
            shape(word, LOCKUP_FONT, s, &fonts)
                .unwrap()
                .iter()
                .map(|r| r.width)
                .sum()
        };
        for s in [48.0, 92.0, 137.0, 280.0] {
            let scaled = at(100.0) * s / 100.0;
            assert!(
                (at(s) - scaled).abs() < 1e-3,
                "{word:?} at {s}: {} {scaled}",
                at(s)
            );
        }
    }
}

#[test]
fn the_face_is_inter_800() {
    let face = FontSet::embedded().get(LOCKUP_FONT).unwrap().clone();
    // The embedded variable font, at weight 800.
    assert_eq!(
        (face.family.as_str(), face.weight),
        ("Inter Variable", 800.0)
    );
}

#[test]
fn every_line_is_as_large_as_fits_its_accent_twice_its_size_and_in_capitals() {
    let (width, height) = room_size();
    for (hook, accent) in HOOKS {
        let found = lockup(hook, accent).unwrap_or_else(|| panic!("{hook:?}"));
        assert!((2..=3).contains(&found.lines.len()), "{hook:?}");
        assert!(found.height <= height, "{hook:?}");
        let mut words = 0;
        let mut baseline = f32::MIN;
        for line in &found.lines {
            assert!(sizes().any(|s| s == line.size), "{hook:?}: {}", line.size);
            assert!(line.size >= LOCKUP_MIN && line.size <= LOCKUP_MAX);
            assert!(line.width <= width + 1e-2, "{hook:?}: {}", line.width);
            assert!(line.baseline > baseline, "{hook:?}: lines go down");
            baseline = line.baseline;
            let set: Vec<(String, f32)> = line
                .words
                .iter()
                .map(|w| (w.text.clone(), w.size))
                .collect();
            assert!(
                (real_width(&set, line.size) - line.width).abs() < 1e-2,
                "{hook:?}"
            );
            // One size up would not fit (or this is the largest).
            if line.size < LOCKUP_MAX {
                let up = line.size + 4.0;
                let bigger: Vec<(String, f32)> = line
                    .words
                    .iter()
                    .map(|w| (w.text.clone(), w.size / line.size * up))
                    .collect();
                assert!(real_width(&bigger, up) > width, "{hook:?}: {up} would fit");
            }
            for word in &line.words {
                let k = if word.accent { ACCENT_SCALE } else { 1.0 };
                assert_eq!(word.size, line.size * k, "{hook:?}");
                assert_eq!(word.accent, words == accent, "{hook:?}");
                assert_eq!(word.text, word.text.to_uppercase());
                words += 1;
            }
        }
        assert_eq!(
            words,
            hook.split_whitespace().count(),
            "{hook:?}: every word once"
        );
    }
}

/// The brute force (§10.4): every split into 2, then 3, lines, each line at
/// the largest size its real shaping fits; the least worst distance, and
/// the fewest lines of those.
fn brute(hook: &str, accent: usize) -> Option<(f32, usize)> {
    let (width, _) = room_size();
    let words: Vec<String> = hook.split_whitespace().map(str::to_uppercase).collect();
    let n = words.len();
    let mut best: Option<(f32, usize)> = None;
    for lines in [2usize, 3] {
        let mut splits: Vec<Vec<(usize, usize)>> = Vec::new();
        if lines == 2 {
            splits.extend((1..n).map(|c| vec![(0, c), (c, n)]));
        } else {
            for a in 1..n {
                for b in a + 1..n {
                    splits.push(vec![(0, a), (a, b), (b, n)]);
                }
            }
        }
        for split in splits {
            let mut worst = 0.0f32;
            let mut ok = true;
            for (a, b) in split {
                let fit = sizes().find(|&s| {
                    let set: Vec<(String, f32)> = (a..b)
                        .map(|i| {
                            let k = if i == accent { ACCENT_SCALE } else { 1.0 };
                            (words[i].clone(), s * k)
                        })
                        .collect();
                    real_width(&set, s) <= width
                });
                let Some(s) = fit else {
                    ok = false;
                    break;
                };
                let set: Vec<(String, f32)> = (a..b)
                    .map(|i| (words[i].clone(), if i == accent { 2.0 * s } else { s }))
                    .collect();
                worst = worst.max(width - real_width(&set, s));
            }
            if ok && best.is_none_or(|(least, _)| worst < least - 1e-3) {
                best = Some((worst, lines));
            }
        }
    }
    best
}

#[test]
fn the_lockup_is_the_least_worst_split_and_ties_go_to_fewer_lines() {
    let (width, _) = room_size();
    for (hook, accent) in HOOKS {
        let found = lockup(hook, accent).unwrap();
        let worst = found
            .lines
            .iter()
            .map(|line| width - line.width)
            .fold(0.0, f32::max);
        let (least, lines) = brute(hook, accent).unwrap();
        assert!((worst - least).abs() < 1e-2, "{hook:?}: {worst} vs {least}");
        assert_eq!(found.lines.len(), lines, "{hook:?}");
        println!(
            "{hook:?}: {} lines, sizes {:?}, worst {:.1} lp ({:.1}% of the width)",
            found.lines.len(),
            found.lines.iter().map(|l| l.size).collect::<Vec<_>>(),
            worst,
            100.0 * worst / width
        );
    }
}

#[test]
fn the_long_hooks_fill_the_width_within_10_percent() {
    // D-484's "within 10%": hooks of 5 words or more, whose lines can each
    // be filled; two short words ("Stop scrolling") cannot reach it at 140.
    let (width, _) = room_size();
    for (hook, accent) in HOOKS {
        if hook.split_whitespace().count() < 5 {
            continue;
        }
        let found = lockup(hook, accent).unwrap();
        for line in &found.lines {
            assert!(
                line.width >= 0.9 * width,
                "{hook:?}: {} of {width}",
                line.width
            );
        }
    }
}

#[test]
fn one_word_a_word_too_wide_a_bad_accent_or_no_room_is_no_lockup() {
    assert!(lockup("Hello", 0).is_none(), "one word");
    assert!(lockup("", 0).is_none());
    assert!(
        lockup("Stop scrolling", 2).is_none(),
        "the accent is not a word"
    );
    assert!(
        lockup("Supercalifragilisticexpialidocious, wow", 1).is_none(),
        "too wide at 48 lp"
    );
    // The accent at twice 48 lp too wide for the width.
    assert!(lockup("Extraordinarily good", 0).is_none());
    // A room too low for two lines at 48 lp.
    let fonts = FontSet::embedded();
    assert!(
        lay_out("Stop scrolling", 1, (960.0, 60.0), &fonts)
            .unwrap()
            .is_none()
    );
}

#[test]
fn the_same_hook_gives_the_same_lockup() {
    for (hook, accent) in HOOKS {
        assert_eq!(lockup(hook, accent), lockup(hook, accent), "{hook:?}");
    }
}
