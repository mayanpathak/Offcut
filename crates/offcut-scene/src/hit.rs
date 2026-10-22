//! Which part of a frame a point is on (D-174): a caption word, the caption
//! block, or the video. Pure, from the chunk layout the scene built: nothing
//! is shaped. Every coordinate is in logical pixels.

use offcut_types::WordIdx;

use crate::captions::Chunk;
use crate::display_list::Rect;
use crate::safe_area::{CANVAS_HEIGHT, CANVAS_WIDTH};

/// The room around a word's glyph box that still hits the word.
pub const HIT_PADDING: f32 = 8.0;

/// What a point of the canvas is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    /// A caption word drawn at that time: the first transcript word it
    /// stands for.
    Word(WordIdx),
    /// The caption block, between or around the words.
    Captions,
    /// Anything else on the canvas.
    Video,
}

/// The glyph box of each word of `chunk`, as it is laid out: from the left
/// of its first run to the right of its last, from its ascent above the
/// line's baseline to its descent below, at the line's scale.
pub fn word_boxes(chunk: &Chunk) -> Vec<(WordIdx, Rect)> {
    let mut boxes = Vec::new();
    for line in &chunk.lines {
        for placed in &line.words {
            let Some(word) = chunk.words.get(placed.word) else {
                continue;
            };
            let width: f32 = placed.runs.iter().map(|run| run.width).sum();
            let ascent = placed.runs.iter().map(|run| run.ascent).fold(0.0, f32::max);
            let descent = placed
                .runs
                .iter()
                .map(|run| run.descent)
                .fold(0.0, f32::max);
            boxes.push((
                word.word,
                Rect {
                    x: line.x + placed.x * line.scale,
                    y: line.baseline - ascent * line.scale,
                    w: width * line.scale,
                    h: (ascent + descent) * line.scale,
                },
            ));
        }
    }
    boxes
}

/// How far `(x, y)` is from `r`: 0 inside it.
fn distance(r: Rect, x: f32, y: f32) -> f32 {
    let dx = (r.x - x).max(x - (r.x + r.w)).max(0.0);
    let dy = (r.y - y).max(y - (r.y + r.h)).max(0.0);
    dx.hypot(dy)
}

/// What `(x, y)` is on, with `chunk` the chunk shown at the time and `block`
/// the caption block. A word is hit within `HIT_PADDING` of its box; where
/// two padded boxes meet, the nearer box wins, and of two as near, the first
/// drawn. `None` for a point outside the canvas, or not a number.
pub(crate) fn hit(chunk: Option<&Chunk>, block: Rect, x: f32, y: f32) -> Option<Hit> {
    let on_canvas = (0.0..=CANVAS_WIDTH).contains(&x) && (0.0..=CANVAS_HEIGHT).contains(&y);
    if !on_canvas {
        return None;
    }
    let Some(chunk) = chunk else {
        return Some(Hit::Video);
    };
    let mut nearest: Option<(f32, WordIdx)> = None;
    for (word, r) in word_boxes(chunk) {
        let d = distance(r, x, y);
        if d <= HIT_PADDING && nearest.is_none_or(|(least, _)| d < least) {
            nearest = Some((d, word));
        }
    }
    if let Some((_, word)) = nearest {
        return Some(Hit::Word(word));
    }
    Some(if distance(block, x, y) == 0.0 {
        Hit::Captions
    } else {
        Hit::Video
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_point_is_as_far_from_a_box_as_its_nearest_edge() {
        let r = Rect {
            x: 10.0,
            y: 20.0,
            w: 30.0,
            h: 40.0,
        };
        assert_eq!(distance(r, 20.0, 30.0), 0.0);
        assert_eq!(distance(r, 40.0, 60.0), 0.0);
        assert_eq!(distance(r, 2.0, 30.0), 8.0);
        assert_eq!(distance(r, 43.0, 64.0), 5.0);
        // Outside the canvas, or not a number: nothing.
        assert_eq!(hit(None, r, -1.0, 5.0), None);
        assert_eq!(hit(None, r, 5.0, 1_921.0), None);
        assert_eq!(hit(None, r, f32::NAN, 5.0), None);
        assert_eq!(hit(None, r, 20.0, 30.0), Some(Hit::Video));
    }
}
