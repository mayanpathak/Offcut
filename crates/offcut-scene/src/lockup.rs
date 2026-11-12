//! The hook's lockup (v6.2implementation D-484, §10.4): the hook's words in
//! Inter 800 capitals, broken into 2 or 3 lines of their own sizes that
//! best fill a width, the accent word twice its line's size. Pure: the same
//! text, accent, room and fonts always give the same lockup.
//!
//! Every split of the words into 2, then 3, lines is tried. Each line takes
//! the largest size from `LOCKUP_MAX` down to `LOCKUP_MIN` in steps of
//! `LOCKUP_STEP` at which it fits the width; a line that does not fit at
//! `LOCKUP_MIN` rules its split out, as does a split taller than the room.
//! A split scores the worst of its lines' distances from the width; the
//! least wins, and of equal scores the first tried (so fewer lines). The
//! words are measured once at `REF` and scaled: a shaped width grows with
//! the size exactly (`tests/lockup.rs`), so a scene build shapes each hook
//! word only twice.

use crate::SceneError;
use crate::display_list::FontKey;
use crate::fonts::FontSet;
use crate::layout::{ShapedRun, cap_height, shape};

/// Inter 800, the lockup's face (D-484), in `FontSet::embedded()`.
pub const LOCKUP_FONT: FontKey = FontKey(4);
/// The sizes a line may take (assumptions, §10.4).
pub const LOCKUP_MAX: f32 = 140.0;
pub const LOCKUP_MIN: f32 = 48.0;
pub const LOCKUP_STEP: f32 = 4.0;
/// The accent word's size, of its line's (D-484).
pub const ACCENT_SCALE: f32 = 2.0;
/// The room between two lines' capitals, of the smaller line's size
/// (assumption).
pub const LINE_GAP: f32 = 0.25;
/// The size the words are measured at.
const REF: f32 = 100.0;

/// A word of the lockup: its runs at its size, from `x` along its line.
#[derive(Clone, Debug, PartialEq)]
pub struct LockupWord {
    pub text: String,
    pub runs: Vec<ShapedRun>,
    pub x: f32,
    pub size: f32,
    pub accent: bool,
}

/// A line: its words, its size (the accent word's is twice it), its width,
/// and its baseline, measured down from the lockup's top.
#[derive(Clone, Debug, PartialEq)]
pub struct LockupLine {
    pub words: Vec<LockupWord>,
    pub size: f32,
    pub width: f32,
    pub baseline: f32,
}

/// The lines, and the height from the first line's ascent to the last
/// line's descent.
#[derive(Clone, Debug, PartialEq)]
pub struct Lockup {
    pub lines: Vec<LockupLine>,
    pub height: f32,
}

/// A line of a split: its words' range and its size.
type LineFit = ((usize, usize), f32);

/// A word measured at `REF`.
struct Measure {
    width: f32,
    ascent: f32,
    descent: f32,
}

fn measure(text: &str, fonts: &FontSet) -> Result<Measure, SceneError> {
    let runs = shape(text, LOCKUP_FONT, REF, fonts)?;
    Ok(Measure {
        width: runs.iter().map(|run| run.width).sum(),
        ascent: runs.iter().map(|run| run.ascent).fold(0.0, f32::max),
        descent: runs.iter().map(|run| run.descent).fold(0.0, f32::max),
    })
}

/// The sizes a line may take, the largest first.
pub fn sizes() -> impl Iterator<Item = f32> {
    let count = ((LOCKUP_MAX - LOCKUP_MIN) / LOCKUP_STEP).round() as u32;
    (0..=count).map(|i| LOCKUP_MAX - LOCKUP_STEP * i as f32)
}

/// Every split of `n` words into `lines` lines of one word or more, as
/// `(first, end)` ranges, in order.
fn splits(n: usize, lines: usize) -> Vec<Vec<(usize, usize)>> {
    match lines {
        2 => (1..n).map(|c| vec![(0, c), (c, n)]).collect(),
        3 => (1..n)
            .flat_map(|a| (a + 1..n).map(move |b| vec![(0, a), (a, b), (b, n)]))
            .collect(),
        _ => Vec::new(),
    }
}

/// The lockup of `text`, its word `accent` the accent, in a room `width`
/// wide and `height` high; `None` when it has one word or none, when
/// `accent` is not one of its words, or when no split fits.
pub fn lay_out(
    text: &str,
    accent: usize,
    (width, height): (f32, f32),
    fonts: &FontSet,
) -> Result<Option<Lockup>, SceneError> {
    let words: Vec<String> = text.split_whitespace().map(str::to_uppercase).collect();
    if words.len() < 2 || accent >= words.len() {
        return Ok(None);
    }
    let m = words
        .iter()
        .map(|word| measure(word, fonts))
        .collect::<Result<Vec<_>, _>>()?;
    let space = measure(" ", fonts)?.width;
    let cap = cap_height(LOCKUP_FONT, REF, fonts)? / REF;
    let k = |i: usize| if i == accent { ACCENT_SCALE } else { 1.0 };
    let get = |i: usize| {
        m.get(i)
            .map_or((0.0, 0.0, 0.0), |m| (m.width, m.ascent, m.descent))
    };
    let unit = |s: f32| s / REF;
    let line_width = |(a, b): (usize, usize), s: f32| {
        let words: f32 = (a..b).map(|i| get(i).0 * k(i)).sum();
        (words + space * b.saturating_sub(a + 1) as f32) * unit(s)
    };
    // The tallest scale of a line: 2 when it holds the accent word.
    let most = |(a, b): (usize, usize)| (a..b).map(k).fold(1.0, f32::max);
    let ascent = |(a, b): (usize, usize), s: f32| {
        let up = (a..b).map(|i| get(i).1 * k(i)).fold(0.0, f32::max);
        up * unit(s)
    };
    let descent = |(a, b): (usize, usize), s: f32| {
        let down = (a..b).map(|i| get(i).2 * k(i)).fold(0.0, f32::max);
        down * unit(s)
    };
    // Each line's baseline from the top, and the height.
    let stack = |sized: &[LineFit]| -> (Vec<f32>, f32) {
        let mut baselines: Vec<f32> = Vec::with_capacity(sized.len());
        let mut before: Option<f32> = None;
        for &(range, s) in sized {
            let baseline = match (baselines.last(), before) {
                (Some(&last), Some(prev)) => last + LINE_GAP * prev.min(s) + cap * s * most(range),
                _ => ascent(range, s),
            };
            baselines.push(baseline);
            before = Some(s);
        }
        let bottom = match (baselines.last(), sized.last()) {
            (Some(&last), Some(&(range, s))) => last + descent(range, s),
            _ => 0.0,
        };
        (baselines, bottom)
    };

    let mut best: Option<(f32, Vec<LineFit>)> = None;
    for lines in [2, 3] {
        for split in splits(words.len(), lines) {
            let sized: Option<Vec<LineFit>> = split
                .iter()
                .map(|&range| {
                    let fits = sizes().find(|&s| line_width(range, s) <= width);
                    fits.map(|s| (range, s))
                })
                .collect();
            let Some(sized) = sized else {
                continue;
            };
            if stack(&sized).1 > height {
                continue;
            }
            let score = sized
                .iter()
                .map(|&(range, s)| width - line_width(range, s))
                .fold(0.0, f32::max);
            if best.as_ref().is_none_or(|(least, _)| score < *least) {
                best = Some((score, sized));
            }
        }
    }
    let Some((_, sized)) = best else {
        return Ok(None);
    };

    let (baselines, height) = stack(&sized);
    let mut lines = Vec::with_capacity(sized.len());
    for (&((a, b), s), baseline) in sized.iter().zip(baselines) {
        let mut x = 0.0f32;
        let mut placed = Vec::with_capacity(b - a);
        for i in a..b {
            let size = s * k(i);
            let text = words.get(i).cloned().unwrap_or_default();
            let runs = shape(&text, LOCKUP_FONT, size, fonts)?;
            let advance: f32 = runs.iter().map(|run| run.width).sum();
            placed.push(LockupWord {
                text,
                runs,
                x,
                size,
                accent: i == accent,
            });
            x += advance + space * unit(s);
        }
        lines.push(LockupLine {
            words: placed,
            size: s,
            width: x - space * unit(s),
            baseline,
        });
    }
    Ok(Some(Lockup { lines, height }))
}
