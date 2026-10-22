//! [ONLY] The caption styles (TS §19.4, D-136, D-137). This is the only file
//! that holds their values; all of them are (assumption), reviewed in E-2. A
//! style names theme roles and a font role, never a colour or a font: the
//! theme gives them their values.

use offcut_types::StyleId;

use crate::easing::Easing;
use crate::motion::{ActiveTreatment, CaptionMotion, MotionPreset, MotionUnit};
use crate::theme::{FontRole, Role};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    AsSpoken,
    Upper,
}

/// A rounded box behind a chunk's lines (Tech, V4 Prompt 105).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanelSpec {
    pub role: Role,
    /// In logical pixels.
    pub radius: f32,
    pub padding: f32,
}

/// One row of the table of TS §19.4.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleSpec {
    pub font: FontRole,
    /// In logical pixels.
    pub size: f32,
    pub case: Case,
    pub max_chars_per_line: u32,
    pub max_lines: u32,
    pub max_words: u32,
    pub fill: Role,
    /// In logical pixels.
    pub stroke_width: f32,
    pub stroke: Role,
    /// The word that is being spoken.
    pub active: Role,
    /// A KeywordPop word.
    pub pop: Role,
    pub panel: Option<PanelSpec>,
    pub motion: CaptionMotion,
}

fn clean() -> StyleSpec {
    StyleSpec {
        font: FontRole::Display,
        size: 64.0,
        case: Case::AsSpoken,
        max_chars_per_line: 18,
        max_lines: 2,
        max_words: 5,
        fill: Role::Text,
        stroke_width: 6.0,
        stroke: Role::Stroke,
        active: Role::Accent,
        pop: Role::Accent,
        panel: None,
        // V3's one motion: captions shown at once, the spoken word in the
        // active colour, a number moving ease-out cubic.
        motion: CaptionMotion {
            unit: MotionUnit::Word,
            enter: MotionPreset::None,
            active: ActiveTreatment::Highlight,
            easing: Easing::EaseOutCubic,
            duration_ms: 120,
        },
    }
}

pub fn spec(id: StyleId) -> StyleSpec {
    match id {
        StyleId::Clean => clean(),
        // V4: the Bold row of TS §19.4.
        StyleId::Bold => clean(),
        // V4: the Tech row of TS §19.4.
        StyleId::Tech => clean(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_style_moves_glyph_by_glyph() {
        // Per-glyph timing is not drawn in V4 (v4implementation §8.5).
        for style in [StyleId::Clean, StyleId::Bold, StyleId::Tech] {
            assert_eq!(spec(style).motion.unit, MotionUnit::Word, "{style:?}");
        }
    }
}
