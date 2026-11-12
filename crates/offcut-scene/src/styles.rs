//! [ONLY] The caption styles (TS §19.4, D-136, D-137, D-139, D-170). This is
//! the only file that holds their values; all of them are (assumption),
//! reviewed in E-2. A style names theme roles and a font role, never a
//! colour or a font: the theme gives them their values.

use offcut_types::StyleId;

use crate::easing::Easing;
use crate::motion::{ActiveTreatment, CaptionMotion, MotionPreset, MotionUnit, ShadowSpec};
use crate::theme::{FontRole, Role};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    AsSpoken,
    Upper,
}

/// A rounded box behind a chunk's lines (Tech).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanelSpec {
    pub role: Role,
    /// In logical pixels.
    pub radius: f32,
    pub padding: f32,
}

/// A soft glow behind each word (V6.2, v6.2implementation D-479): `layers`
/// strokes of the word's own colour, the widest drawn first, each `spread`
/// wider than the one before and `alpha` opaque, so that where they overlap
/// near the letters the glow is strongest. No blur: strokes in Vello.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glow {
    pub layers: u8,
    /// In logical pixels.
    pub spread: f32,
    pub alpha: f32,
}

impl Glow {
    /// How far past a glyph's outline the widest stroke reaches, in logical
    /// pixels, on one side, over a run stroked `stroke_width` wide.
    pub fn reach(&self, stroke_width: f32) -> f32 {
        (stroke_width.max(0.0) + self.spread * f32::from(self.layers)) / 2.0
    }
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
    /// The word that is being spoken, or its pill.
    pub active: Role,
    /// A KeywordPop word.
    pub pop: Role,
    pub panel: Option<PanelSpec>,
    pub shadow: Option<ShadowSpec>,
    /// V6.2 (D-479): Glow's only; the other styles have none.
    pub glow: Option<Glow>,
    pub motion: CaptionMotion,
}

/// The pill's corner radius, and its padding around the word's box,
/// across and down, in logical pixels (D-170).
pub const PILL_RADIUS: f32 = 14.0;
pub const PILL_PADDING: (f32, f32) = (12.0, 4.0);
/// The cursor's width, in ems of the caption size (D-139).
pub const CURSOR_WIDTH_EM: f32 = 0.6;

/// Clean and Bold's shadow: down 4 lp, black at 45% (D-170).
const DROP_SHADOW: ShadowSpec = ShadowSpec {
    offset: (0.0, 4.0),
    role: Role::Shadow,
    opacity: 0.45,
};

impl StyleSpec {
    /// The width beside a line, both sides together, that what is drawn
    /// around its glyphs takes: half the stroke and the shadow's move, the
    /// pill's padding, the panel's padding and the cursor's cell. Lines are
    /// fitted in the caption block less this, so that none of it leaves the
    /// block (§8.6, §8.9). It moves no line that fits.
    pub fn decoration_width(&self) -> f32 {
        let shadow = self.shadow.map_or(0.0, |s| s.offset.0.abs());
        let stroke = (self.stroke_width.max(0.0) / 2.0).max(self.glow_reach()) + shadow;
        let cursor = match self.motion.active {
            ActiveTreatment::Cursor => CURSOR_WIDTH_EM * self.size,
            ActiveTreatment::Highlight | ActiveTreatment::Pill => 0.0,
        };
        let pill = match self.motion.active {
            ActiveTreatment::Pill => PILL_PADDING.0,
            ActiveTreatment::Highlight | ActiveTreatment::Cursor => 0.0,
        };
        let panel = self.panel.map_or(0.0, |panel| panel.padding);
        2.0 * stroke.max(pill).max(panel + cursor)
    }

    /// The height above and below a line's glyphs that what is drawn around
    /// them may take, on one side: half the stroke with the drop shadow's
    /// 4 lp, the pill's padding down, and the panel's padding (DC-13). Lines
    /// aligned to the caption block's top or bottom keep this inside it. The
    /// shadow and the pill count whether the style draws them or not: they
    /// change no layout box (V4), so a line sits where it would without them.
    pub fn decoration_height(&self) -> f32 {
        let stroke =
            (self.stroke_width.max(0.0) / 2.0).max(self.glow_reach()) + DROP_SHADOW.offset.1.abs();
        let panel = self.panel.map_or(0.0, |panel| panel.padding);
        stroke.max(PILL_PADDING.1).max(panel)
    }

    /// How far the glow reaches past the outline: 0.0 without one, so a
    /// style with none is laid out as before.
    fn glow_reach(&self) -> f32 {
        self.glow.map_or(0.0, |glow| glow.reach(self.stroke_width))
    }
}

/// Glow's glow (D-479; assumption, tuned in the quality pass): four strokes
/// reaching 12 lp past the letters at 72 lp.
const GLOW: Glow = Glow {
    layers: 4,
    spread: 6.0,
    alpha: 0.16,
};

/// The premium default (D-170): Inter 800, a drop shadow, the spoken word on
/// a sliding pill, the chunk fading in.
fn clean() -> StyleSpec {
    StyleSpec {
        font: FontRole::Display,
        size: 72.0,
        case: Case::AsSpoken,
        max_chars_per_line: 16,
        max_lines: 2,
        max_words: 4,
        fill: Role::Text,
        stroke_width: 5.0,
        stroke: Role::Stroke,
        active: Role::Accent,
        pop: Role::AccentAlt,
        panel: None,
        shadow: Some(DROP_SHADOW),
        glow: None,
        motion: CaptionMotion {
            unit: MotionUnit::Word,
            enter: MotionPreset::Fade,
            active: ActiveTreatment::Pill,
            easing: Easing::EaseOutCubic,
            duration_ms: 120,
        },
    }
}

/// Inter 900 in upper case, three words, each popping as it is spoken, the
/// spoken word in the accent (D-170).
fn bold() -> StyleSpec {
    StyleSpec {
        font: FontRole::Display,
        size: 84.0,
        case: Case::Upper,
        max_chars_per_line: 12,
        max_lines: 2,
        max_words: 3,
        fill: Role::Text,
        stroke_width: 10.0,
        stroke: Role::Stroke,
        active: Role::Accent,
        pop: Role::AccentAlt,
        panel: None,
        shadow: Some(DROP_SHADOW),
        glow: None,
        motion: CaptionMotion {
            unit: MotionUnit::Word,
            enter: MotionPreset::Pop,
            active: ActiveTreatment::Highlight,
            easing: Easing::BackOut,
            duration_ms: 140,
        },
    }
}

/// "Mono" in the interface (D-139, D-170): JetBrains Mono on a dark panel,
/// the spoken word in green with a block cursor after it.
fn tech() -> StyleSpec {
    StyleSpec {
        font: FontRole::Mono,
        size: 56.0,
        case: Case::AsSpoken,
        max_chars_per_line: 22,
        max_lines: 2,
        max_words: 5,
        fill: Role::Text,
        stroke_width: 0.0,
        stroke: Role::Stroke,
        active: Role::Accent,
        pop: Role::AccentAlt,
        panel: Some(PanelSpec {
            role: Role::Panel,
            radius: 24.0,
            padding: 32.0,
        }),
        shadow: None,
        glow: None,
        motion: CaptionMotion {
            unit: MotionUnit::Word,
            enter: MotionPreset::None,
            active: ActiveTreatment::Cursor,
            easing: Easing::Linear,
            duration_ms: 80,
        },
    }
}

/// V6.2 (D-480, F-3): white Inter 800 with a soft glow and Clean's drop
/// shadow, two or three words, cut in hard, the spoken word and a keyword in
/// the accent, each glowing in its own colour.
fn glow() -> StyleSpec {
    StyleSpec {
        font: FontRole::Display,
        size: 76.0,
        case: Case::AsSpoken,
        max_chars_per_line: 14,
        max_lines: 2,
        max_words: 3,
        fill: Role::Text,
        stroke_width: 0.0,
        stroke: Role::Stroke,
        active: Role::Accent,
        pop: Role::AccentAlt,
        panel: None,
        shadow: Some(DROP_SHADOW),
        glow: Some(GLOW),
        motion: CaptionMotion {
            unit: MotionUnit::Word,
            enter: MotionPreset::None,
            active: ActiveTreatment::Highlight,
            easing: Easing::EaseOutCubic,
            duration_ms: 100,
        },
    }
}

pub fn spec(id: StyleId) -> StyleSpec {
    match id {
        StyleId::Clean => clean(),
        StyleId::Bold => bold(),
        StyleId::Tech => tech(),
        StyleId::Glow => glow(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STYLES: [StyleId; 3] = [StyleId::Clean, StyleId::Bold, StyleId::Tech];

    #[test]
    fn no_style_moves_glyph_by_glyph() {
        // Per-glyph timing is not drawn in V4 (v4implementation §8.5).
        for style in STYLES {
            assert_eq!(spec(style).motion.unit, MotionUnit::Word, "{style:?}");
        }
    }

    #[test]
    fn each_style_has_the_values_of_its_decision() {
        // Size, case, characters per line x lines, words, stroke (D-170 for
        // Clean and Bold, D-139 for Tech).
        let rows = [
            (StyleId::Clean, 72.0, Case::AsSpoken, 16, 2, 4, 5.0),
            (StyleId::Bold, 84.0, Case::Upper, 12, 2, 3, 10.0),
            (StyleId::Tech, 56.0, Case::AsSpoken, 22, 2, 5, 0.0),
        ];
        for (style, size, case, chars, lines, words, stroke) in rows {
            let row = spec(style);
            let got = (row.size, row.case, row.max_chars_per_line, row.max_lines);
            assert_eq!(got, (size, case, chars, lines), "{style:?}");
            assert_eq!((row.max_words, row.stroke_width), (words, stroke));
            // The spoken word (or its pill) in the accent, a KeywordPop in
            // the second accent (D-139).
            assert_eq!(
                (row.fill, row.active, row.pop),
                (Role::Text, Role::Accent, Role::AccentAlt)
            );
        }
        let motion = |style| {
            let m = spec(style).motion;
            (m.enter, m.active, m.easing, m.duration_ms)
        };
        use ActiveTreatment::{Cursor, Highlight, Pill};
        let clean = (MotionPreset::Fade, Pill, Easing::EaseOutCubic, 120);
        assert_eq!(motion(StyleId::Clean), clean);
        let bold = (MotionPreset::Pop, Highlight, Easing::BackOut, 140);
        assert_eq!(motion(StyleId::Bold), bold);
        let tech = (MotionPreset::None, Cursor, Easing::Linear, 80);
        assert_eq!(motion(StyleId::Tech), tech);
        // Clean and Bold cast a shadow; only Tech has a panel.
        assert_eq!(spec(StyleId::Clean).shadow, Some(DROP_SHADOW));
        assert_eq!(spec(StyleId::Bold).shadow, Some(DROP_SHADOW));
        assert_eq!(spec(StyleId::Tech).shadow, None);
        let panel = spec(StyleId::Tech).panel.unwrap();
        assert_eq!(
            (panel.role, panel.radius, panel.padding),
            (Role::Panel, 24.0, 32.0)
        );
        assert!(spec(StyleId::Clean).panel.is_none() && spec(StyleId::Bold).panel.is_none());
        let fonts = STYLES.map(|style| spec(style).font);
        assert_eq!(
            fonts,
            [FontRole::Display, FontRole::Display, FontRole::Mono]
        );
    }

    #[test]
    fn the_room_beside_a_line_is_its_widest_decoration_on_both_sides() {
        // Clean: the pill's 12 lp over half its 5 lp stroke.
        assert_eq!(spec(StyleId::Clean).decoration_width(), 24.0);
        // Bold: half its 10 lp stroke.
        assert_eq!(spec(StyleId::Bold).decoration_width(), 10.0);
        // Tech: the panel's 32 lp and a cursor cell of 0.6 x 56 lp.
        let tech = spec(StyleId::Tech).decoration_width();
        assert!((tech - 2.0 * (32.0 + 33.6)).abs() < 1e-3, "{tech}");
    }

    #[test]
    fn only_glow_has_a_glow_and_the_others_lay_out_as_before() {
        for style in STYLES {
            assert_eq!(spec(style).glow, None, "{style:?}");
        }
        let glow = spec(StyleId::Glow);
        assert_eq!(
            glow.glow,
            Some(Glow {
                layers: 4,
                spread: 6.0,
                alpha: 0.16
            })
        );
        assert_eq!(
            (glow.size, glow.max_words, glow.stroke_width),
            (76.0, 3, 0.0)
        );
        assert_eq!(glow.motion.active, ActiveTreatment::Highlight);
        assert_eq!(glow.motion.enter, MotionPreset::None);
        assert_eq!(glow.shadow, Some(DROP_SHADOW));
        // The glow reaches 12 lp past the letters: the room kept beside a line.
        assert_eq!(GLOW.reach(0.0), 12.0);
        assert_eq!(glow.decoration_width(), 2.0 * (12.0 + 0.0));
        assert_eq!(glow.decoration_height(), 12.0 + 4.0);
    }
}
