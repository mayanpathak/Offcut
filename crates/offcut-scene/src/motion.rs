//! How captions move, as data (TS §19.4, D-137). A style's motion is one
//! `CaptionMotion`; V6.5's kinetic presets append members to the enums here.

use offcut_types::OutMs;

use crate::anim::{Track, eval};
use crate::easing::{Easing, ease};
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
