//! How captions move, as data (TS §19.4, D-137). A style's motion is one
//! `CaptionMotion`; V6.5's kinetic presets append members to the enums here.

use offcut_types::OutMs;

use crate::anim::{Track, eval};
use crate::display_list::Rect;
use crate::easing::{Easing, ease};
use crate::styles::{PILL_PADDING, PILL_RADIUS};
use crate::theme::Role;

/// What one step of a motion moves. No V4 style moves glyph by glyph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionUnit {
    Word,
    Glyph,
}

/// How a caption comes in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionPreset {
    /// Shown at once.
    None,
    /// The chunk's opacity goes from 0 to 1 from its show start.
    Fade,
    /// Each word scales 1.0 → `POP_SCALE` → 1.0 from its own start.
    Pop,
}

/// How the word that is being spoken stands out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveTreatment {
    /// Its fill is the style's `active` colour.
    Highlight,
    /// Its fill is the style's `active` colour, and a block in that colour
    /// follows it (D-139).
    Cursor,
    /// It sits on a rounded pill in the `active` colour, drawn in
    /// `OnAccent`; the pill slides from the word before (D-170).
    Pill,
}

/// A hard drop shadow: each glyph run drawn again under itself, moved by
/// `offset` (logical pixels), in `role`'s colour at `opacity` (D-170).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowSpec {
    pub offset: (f32, f32),
    pub role: Role,
    pub opacity: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaptionMotion {
    pub unit: MotionUnit,
    pub enter: MotionPreset,
    pub active: ActiveTreatment,
    pub easing: Easing,
    pub duration_ms: u32,
}

/// The middle of a `Pop` (TS §19.4).
pub const POP_SCALE: f32 = 1.12;

/// A KeywordPop word's scale from its start: up to the peak at 90 ms, back
/// to the hold at 180 ms, held until the word ends (D-139, TS §19.4).
pub const KEYWORD_POP_PEAK: (u32, f32) = (90, 1.25);
pub const KEYWORD_POP_HOLD: (u32, f32) = (180, 1.10);
const KEYWORD_POP_EASING: Easing = Easing::EaseOutCubic;

/// A KeywordPop word's scale `since_ms` after its start, before it ends.
pub(crate) fn keyword_scale(since_ms: u32) -> f32 {
    let at = OutMs::new;
    let track = Track {
        keys: vec![
            (at(0), 1.0),
            (at(KEYWORD_POP_PEAK.0), KEYWORD_POP_PEAK.1),
            (at(KEYWORD_POP_HOLD.0), KEYWORD_POP_HOLD.1),
        ],
        easing: KEYWORD_POP_EASING,
    };
    eval(&track, at(since_ms))
}

/// Opacity and scale `since_ms` after the motion's start: for `Fade` the
/// chunk's show start, for `Pop` the word's own start. Pure: a function of
/// its two arguments only.
pub(crate) fn enter_at(m: &CaptionMotion, since_ms: u32) -> (f32, f32) {
    let x = if m.duration_ms == 0 {
        1.0
    } else {
        since_ms as f32 / m.duration_ms as f32
    };
    match m.enter {
        MotionPreset::None => (1.0, 1.0),
        MotionPreset::Fade => (ease(m.easing, x), 1.0),
        MotionPreset::Pop => {
            // Up in the first half, back in the second.
            let swing = if x < 0.5 {
                ease(m.easing, x * 2.0)
            } else {
                1.0 - ease(m.easing, (x - 0.5) * 2.0)
            };
            (1.0, 1.0 + (POP_SCALE - 1.0) * swing)
        }
    }
}

// The geometry of what moves in a caption (V4 Prompt 107 moved it here
// from `captions_draw.rs`): the pill's slide, a word's growth, the room a
// growing word's line makes.

/// The pill's move from the word before to the spoken word, from the spoken
/// word's start, and the scale it grows from on the way (D-170).
pub const PILL_SLIDE_MS: u32 = 90;
pub const PILL_GROW_FROM: f32 = 0.9;
pub(crate) const PILL_EASING: Easing = Easing::EaseOutCubic;

/// `r` scaled by `k` about the point `about`.
pub(crate) fn scaled_about(r: Rect, (x, y): (f32, f32), k: f32) -> Rect {
    Rect {
        x: x + (r.x - x) * k,
        y: y + (r.y - y) * k,
        w: r.w * k,
        h: r.h * k,
    }
}

/// A line whose words grow at `t` (a pop, a KeywordPop) widens instead of
/// letting them overlap: each word keeps its room to the next, the others
/// move aside, and the line stays centred. `boxes` are the words' glyph
/// boxes as laid out and `scales` the scales they would grow to; the line
/// widens by at most `room` on each side (the safe area's), the growth cut
/// in proportion beyond it. The scales drawn and how far each word's centre
/// moves.
pub(crate) fn reflow(boxes: &[Rect], scales: &[f32], room: f32) -> (Vec<f32>, Vec<f32>) {
    let growth: Vec<f32> = boxes
        .iter()
        .zip(scales)
        .map(|(r, k)| (k - 1.0) * r.w)
        .collect();
    let widening: f32 = growth.iter().filter(|g| **g > 0.0).sum();
    let cut = if widening > 2.0 * room {
        (2.0 * room / widening).max(0.0)
    } else {
        1.0
    };
    let growth: Vec<f32> = growth
        .into_iter()
        .map(|g| if g > 0.0 { g * cut } else { g })
        .collect();
    let total: f32 = growth.iter().sum();
    let mut before = 0.0;
    let mut kept = Vec::with_capacity(growth.len());
    let mut shifts = Vec::with_capacity(growth.len());
    for (g, r) in growth.iter().zip(boxes) {
        kept.push(if r.w > 0.0 { 1.0 + g / r.w } else { 1.0 });
        shifts.push(before + g / 2.0 - total / 2.0);
        before += g;
    }
    (kept, shifts)
}

/// The pill `since_ms` after the spoken word's start, and its corner
/// radius: on its way from the box `from` to the box `to`, padded, growing
/// from `PILL_GROW_FROM` to its size as it arrives (D-170).
pub(crate) fn pill(from: Rect, to: Rect, since_ms: u32) -> (Rect, f32) {
    let e = ease(PILL_EASING, since_ms as f32 / PILL_SLIDE_MS as f32);
    // Exactly `a` at the start and `b` at the end.
    let mix = |a: f32, b: f32| a * (1.0 - e) + b * e;
    let k = mix(PILL_GROW_FROM, 1.0);
    let (pad_x, pad_y) = PILL_PADDING;
    let w = (mix(from.w, to.w) + 2.0 * pad_x) * k;
    let h = (mix(from.h, to.h) + 2.0 * pad_y) * k;
    let x = mix(from.x + from.w / 2.0, to.x + to.w / 2.0);
    let y = mix(from.y + from.h / 2.0, to.y + to.h / 2.0);
    let rect = Rect {
        x: x - w / 2.0,
        y: y - h / 2.0,
        w,
        h,
    };
    (rect, PILL_RADIUS * k)
}

#[cfg(test)]
mod tests {
    use offcut_types::StyleId;

    use super::*;
    use crate::styles::spec;

    fn motion(enter: MotionPreset, easing: Easing) -> CaptionMotion {
        CaptionMotion {
            unit: MotionUnit::Word,
            enter,
            active: ActiveTreatment::Highlight,
            easing,
            duration_ms: 120,
        }
    }

    #[test]
    fn each_preset_starts_where_it_should_and_ends_at_rest() {
        let none = motion(MotionPreset::None, Easing::EaseOutCubic);
        for since in [0, 60, 120, 5_000] {
            assert_eq!(enter_at(&none, since), (1.0, 1.0));
        }
        let fade = motion(MotionPreset::Fade, Easing::Linear);
        assert_eq!(enter_at(&fade, 0), (0.0, 1.0));
        assert_eq!(enter_at(&fade, 60), (0.5, 1.0));
        assert_eq!(enter_at(&fade, 120), (1.0, 1.0));
        assert_eq!(enter_at(&fade, 900), (1.0, 1.0));
        let pop = motion(MotionPreset::Pop, Easing::Linear);
        assert_eq!(enter_at(&pop, 0), (1.0, 1.0));
        assert_eq!(enter_at(&pop, 60), (1.0, POP_SCALE));
        assert_eq!(enter_at(&pop, 120), (1.0, 1.0));
        assert_eq!(enter_at(&pop, 900), (1.0, 1.0));
        // No time to move in: at rest at once.
        let instant = CaptionMotion {
            duration_ms: 0,
            ..fade
        };
        assert_eq!(enter_at(&instant, 0), (1.0, 1.0));
    }

    #[test]
    fn cleans_fade_and_bolds_pop_at_their_start_half_and_end() {
        // Clean: a fade over 120 ms, ease-out cubic (D-170).
        let clean = spec(StyleId::Clean).motion;
        assert_eq!(enter_at(&clean, 0), (0.0, 1.0));
        assert_eq!(enter_at(&clean, 60), (0.875, 1.0));
        assert_eq!(enter_at(&clean, 120), (1.0, 1.0));
        // Bold: a pop over 140 ms, back-out: at its height half-way, past
        // it on the way, at rest at the end (D-170).
        let bold = spec(StyleId::Bold).motion;
        assert_eq!(enter_at(&bold, 0), (1.0, 1.0));
        assert_eq!(enter_at(&bold, 70), (1.0, POP_SCALE));
        let (_, rising) = enter_at(&bold, 49);
        assert!(rising > POP_SCALE, "{rising}");
        assert_eq!(enter_at(&bold, 140), (1.0, 1.0));
        // Tech: shown at once.
        let tech = spec(StyleId::Tech).motion;
        for since in [0, 40, 80] {
            assert_eq!(enter_at(&tech, since), (1.0, 1.0));
        }
    }
}
