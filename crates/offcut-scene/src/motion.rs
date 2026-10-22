//! How captions move, as data (TS §19.4, D-137). A style's motion is one
//! `CaptionMotion`; V6.5's kinetic presets append members to the enums here.

use crate::easing::{Easing, ease};

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
    /// A block in the `active` colour after it.
    Cursor,
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
    use super::*;

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
}
