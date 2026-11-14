//! Emoji: a word an emoji says better, with the emoji under it or beside it
//! (v6.2implementation D-483, TS §47.3). Like a KeywordPop it belongs to the
//! captions (the catalog's zone is the caption zone): its track draws
//! nothing in the event zone, and `build_scene` places it once by its
//! chunk's layout boxes, the boxes `hit` uses, into a `Placed` the scene
//! draws after the captions.
//!
//! The emoji is 1.6 times the capitals' height of its line, never larger
//! than its image (160 px). `Beside` sits at the end of the word's line,
//! centred on the capitals, when the line leaves room inside the safe area
//! and clear of the chunk's other lines; else it is placed as `Under`: centred on the word, below the chunk's
//! lines. When that would reach the watermark's band or leave the safe area
//! (Bottom, the default placement, leaves no room under the lines) it sits
//! over them instead. It pops in on `OUT` at its word's program start and
//! leaves with its chunk.

use offcut_types::{EmojiId, EmojiPlace, OutMs, OutSpan, WordIdx};

use crate::captions::Chunk;
use crate::display_list::{DrawCmd, ImageKey, Rect};
use crate::events::{about_centre, image_pop};
use crate::hit::word_box;
use crate::images::ImageSet;
use crate::motion::KEYWORD_POP_PEAK;
use crate::placement::WATERMARK_BAND;
use crate::safe_area::SAFE_AREA;
use crate::styles::StyleSpec;

/// An emoji's size in capitals' heights (D-483).
pub const EMOJI_CAP_HEIGHTS: f32 = 1.6;
/// The room between the captions, with what is drawn around them, and the
/// emoji (assumption).
pub const EMOJI_GAP: f32 = 16.0;

/// The word, its emoji's image, where it goes, and from when.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Track {
    pub word: WordIdx,
    pub image: ImageKey,
    pub place: EmojiPlace,
    pub start: OutMs,
}

/// The item's word and emoji, from the start of its program span.
pub fn layout(word: WordIdx, emoji: EmojiId, place: EmojiPlace, span: OutSpan) -> Track {
    Track {
        word,
        image: ImageKey::of_emoji(emoji),
        place,
        start: span.start,
    }
}

/// Nothing: the scene draws the emoji as `place` placed it (`draw_placed`).
pub fn draw(track: &Track, t: OutMs, out: &mut Vec<DrawCmd>) {
    let _ = (track, t, out);
}

/// Where an emoji ended up against its word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Beside,
    Under,
    /// Over the chunk's lines: `Under` with no room under them.
    Over,
}

/// An emoji placed by its chunk: its image at rest, and when it is shown.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub image: ImageKey,
    pub rect: Rect,
    pub side: Side,
    pub start: OutMs,
    pub until: OutMs,
}

/// The emoji of `track` placed by `chunk`, the chunk that holds its word,
/// in `style`, whose capitals are `cap` high at its size; until the chunk
/// ends. `None` when the chunk does not hold the word, when it ends before
/// the word starts, or when no place is inside the safe area.
pub fn place(track: &Track, chunk: &Chunk, style: &StyleSpec, cap: f32) -> Option<Placed> {
    let (line, word) = chunk.lines.iter().find_map(|line| {
        let placed = line.words.iter().find(|p| {
            chunk
                .words
                .get(p.word)
                .is_some_and(|w| w.word == track.word)
        })?;
        Some((line, word_box(line, placed)))
    })?;
    let (start, until) = (track.start, chunk.show.end);
    if start >= until {
        return None;
    }
    let size = (EMOJI_CAP_HEIGHTS * cap * line.scale).min(ImageSet::SIZE as f32);
    let shown = |rect: Rect, side: Side| Placed {
        image: track.image,
        rect,
        side,
        start,
        until,
    };
    let area = SAFE_AREA;
    let fits = |r: Rect| {
        r.x >= area.x
            && r.y >= area.y
            && r.x + r.w <= area.x + area.w
            && r.y + r.h <= area.y + area.h - WATERMARK_BAND
    };
    // A word may grow while it is shown (a KeywordPop's peak, more than any
    // style's pop): what it can reach is kept clear of the emoji.
    let grow = KEYWORD_POP_PEAK.1 - 1.0;
    let scale = chunk.lines.iter().map(|l| l.scale).fold(1.0, f32::max);
    let boxes: Vec<Rect> = chunk
        .lines
        .iter()
        .flat_map(|line| line.words.iter().map(move |p| word_box(line, p)))
        .collect();
    let deco = style.decoration_height() * scale;

    if matches!(track.place, EmojiPlace::Beside) {
        let own: Vec<Rect> = line.words.iter().map(|p| word_box(line, p)).collect();
        let right = own.iter().map(|b| b.x + b.w).fold(f32::MIN, f32::max);
        let widest = own.iter().map(|b| b.w).fold(0.0, f32::max);
        let side = style.decoration_width() / 2.0 * scale + widest * grow / 2.0 + EMOJI_GAP;
        let x = right + side;
        let middle = line.baseline - cap * line.scale / 2.0;
        let rect = Rect {
            x,
            y: middle - size / 2.0,
            w: size,
            h: size,
        };
        // Clear of every word of the chunk, a wider line's too, with what is
        // drawn around them and their growth.
        let clear = boxes.iter().all(|b| {
            let around = Rect {
                x: b.x - side,
                y: b.y - b.h * grow / 2.0 - deco - EMOJI_GAP,
                w: b.w + 2.0 * side,
                h: b.h * (1.0 + grow) + 2.0 * (deco + EMOJI_GAP),
            };
            !overlap(around, rect)
        });
        if fits(rect) && clear {
            return Some(shown(rect, Side::Beside));
        }
    }

    let top = boxes.iter().map(|b| b.y).fold(f32::MAX, f32::min);
    let low = boxes.iter().map(|b| b.y + b.h).fold(f32::MIN, f32::max);
    let (first, last) = (chunk.lines.first()?, chunk.lines.last()?);
    let x = (word.x + word.w / 2.0 - size / 2.0).clamp(area.x, area.x + area.w - size);
    let under = Rect {
        x,
        y: low + (low - last.baseline) * grow + deco + EMOJI_GAP,
        w: size,
        h: size,
    };
    if fits(under) {
        return Some(shown(under, Side::Under));
    }
    let over = Rect {
        y: top - (first.baseline - top) * grow - deco - EMOJI_GAP - size,
        ..under
    };
    fits(over).then(|| shown(over, Side::Over))
}

fn overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// Adds the emoji while it is shown: popping in from its start (D-483).
pub fn draw_placed(placed: &Placed, t: OutMs, out: &mut Vec<DrawCmd>) {
    if t < placed.start || t >= placed.until {
        return;
    }
    let k = image_pop(t.get() - placed.start.get());
    out.push(DrawCmd::Image {
        image: placed.image,
        rect: about_centre(placed.rect, k),
        opacity: 1.0,
    });
}

#[cfg(test)]
mod tests {
    use offcut_types::EmojiId;

    use super::*;
    use crate::events::{IMAGE_POP_FROM, IMAGE_POP_MS};

    fn span() -> OutSpan {
        OutSpan::new(OutMs::new(1_250), OutMs::new(1_500)).unwrap()
    }

    #[test]
    fn the_track_is_the_word_its_image_and_its_start_and_draws_nothing() {
        let track = layout(WordIdx::new(1), EmojiId::Fire, EmojiPlace::Under, span());
        assert_eq!(track.image, ImageKey::of_emoji(EmojiId::Fire));
        assert_eq!(
            (track.word, track.start),
            (WordIdx::new(1), OutMs::new(1_250))
        );
        let mut out = Vec::new();
        for t in [0, 1_250, 1_400, 9_000] {
            draw(&track, OutMs::new(t), &mut out);
        }
        assert!(out.is_empty());
    }

    #[test]
    fn a_placed_emoji_pops_in_on_out_from_its_start_and_leaves_at_its_end() {
        let rect = Rect {
            x: 100.0,
            y: 200.0,
            w: 80.0,
            h: 80.0,
        };
        let placed = Placed {
            image: ImageKey::of_emoji(EmojiId::Fire),
            rect,
            side: Side::Under,
            start: OutMs::new(1_000),
            until: OutMs::new(2_000),
        };
        let at = |t: u32| {
            let mut out = Vec::new();
            draw_placed(&placed, OutMs::new(t), &mut out);
            out
        };
        assert!(at(999).is_empty() && at(2_000).is_empty());
        let size = |t: u32| match at(t).as_slice() {
            [
                DrawCmd::Image {
                    rect: r, opacity, ..
                },
            ] => {
                assert_eq!(*opacity, 1.0);
                // About its centre.
                assert!((r.x + r.w / 2.0 - 140.0).abs() < 1e-3);
                assert!((r.y + r.h / 2.0 - 240.0).abs() < 1e-3);
                r.w / 80.0
            }
            other => panic!("{other:?}"),
        };
        assert!((size(1_000) - IMAGE_POP_FROM).abs() < 1e-5);
        assert!((size(1_000 + IMAGE_POP_MS) - 1.0).abs() < 1e-5);
        assert_eq!(size(1_999), 1.0);
        // Rising all the way, fast first (`OUT`).
        let mut was = size(1_000);
        for t in 1_001..=1_120 {
            let now = size(t);
            assert!(now >= was, "{t}");
            was = now;
        }
        assert!(size(1_060) > 0.8, "{}", size(1_060));
    }
}
