//! [ONLY colour values] The colours and fonts a style is drawn in, by role
//! (TS §40.5, D-136). A style names roles; a theme gives each role its value,
//! so that a brand kit (V6.7) is one `Theme`. The values are the table of
//! v4implementation §8.4 (D-139, D-170), all (assumption), reviewed in E-2.

use offcut_types::StyleId;

use crate::display_list::{FontKey, Rgba};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Text,
    Stroke,
    Accent,
    AccentAlt,
    Panel,
    Backdrop,
    /// A word drawn on the accent: the spoken word on Clean's pill (D-170).
    OnAccent,
    /// A drop shadow, before its opacity (D-170).
    Shadow,
}

/// Which font of the theme a text is set in. Mono is the code font, which a
/// brand kit does not replace (D-136).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontRole {
    Display,
    Body,
    Mono,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub text: Rgba,
    pub stroke: Rgba,
    /// The word that is being spoken (Clean: its pill).
    pub accent: Rgba,
    /// A KeywordPop word, FromTo's "to" value.
    pub accent_alt: Rgba,
    pub panel: Rgba,
    pub backdrop: Rgba,
    pub on_accent: Rgba,
    pub shadow: Rgba,
    pub display_font: FontKey,
    pub body_font: FontKey,
    pub mono_font: FontKey,
}

const fn opaque(r: u8, g: u8, b: u8) -> Rgba {
    Rgba { r, g, b, a: 255 }
}

/// 85% of 255, the panels' opacity (D-139).
const PANEL_ALPHA: u8 = 217;

const WHITE: Rgba = opaque(255, 255, 255);
/// `#101014`.
const DARK: Rgba = opaque(16, 16, 20);
/// `#FFD60A`.
const YELLOW: Rgba = opaque(255, 214, 10);
/// `#FF5E3A`, Bold's KeywordPop.
const ORANGE: Rgba = opaque(255, 94, 58);
/// `#3DDC84`, Tech's spoken word.
const GREEN: Rgba = opaque(61, 220, 132);
/// `#E8F0EA`, Tech's text.
const PALE_GREEN: Rgba = opaque(232, 240, 234);
/// `#111114`, the word on Clean's pill.
const INK: Rgba = opaque(17, 17, 20);
/// `#101014` at 85%: Clean's and Bold's panel.
const PANEL: Rgba = Rgba {
    a: PANEL_ALPHA,
    ..DARK
};
/// `#0B0F0C` at 85%: Tech's panel.
const TECH_PANEL: Rgba = Rgba {
    a: PANEL_ALPHA,
    ..opaque(11, 15, 12)
};
/// What the canvas shows where no video is (D-133), and a shadow's colour.
const BLACK: Rgba = opaque(0, 0, 0);

/// Keys of `FontSet::embedded()`.
const INTER_700: FontKey = FontKey(0);
const INTER_900: FontKey = FontKey(1);
const JETBRAINS_MONO_700: FontKey = FontKey(2);
const INTER_800: FontKey = FontKey(4);

/// Clean, the default (D-170): white Inter 800 with a dark stroke, the
/// spoken word in ink on a yellow pill.
fn clean() -> Theme {
    Theme {
        text: WHITE,
        stroke: DARK,
        accent: YELLOW,
        accent_alt: YELLOW,
        panel: PANEL,
        backdrop: BLACK,
        on_accent: INK,
        shadow: BLACK,
        display_font: INTER_800,
        body_font: INTER_700,
        mono_font: JETBRAINS_MONO_700,
    }
}

/// Bold (D-170): Clean's colours in Inter 900, a KeywordPop in orange.
fn bold() -> Theme {
    Theme {
        accent_alt: ORANGE,
        display_font: INTER_900,
        ..clean()
    }
}

/// Tech, "Mono" in the interface (D-139): pale green JetBrains Mono on a
/// dark green panel, the spoken word in green.
fn tech() -> Theme {
    Theme {
        text: PALE_GREEN,
        accent: GREEN,
        accent_alt: GREEN,
        panel: TECH_PANEL,
        display_font: JETBRAINS_MONO_700,
        ..clean()
    }
}

impl Theme {
    pub fn default_for(style: StyleId) -> Theme {
        match style {
            StyleId::Clean => clean(),
            StyleId::Bold => bold(),
            StyleId::Tech => tech(),
        }
    }

    pub fn color(&self, r: Role) -> Rgba {
        match r {
            Role::Text => self.text,
            Role::Stroke => self.stroke,
            Role::Accent => self.accent,
            Role::AccentAlt => self.accent_alt,
            Role::Panel => self.panel,
            Role::Backdrop => self.backdrop,
            Role::OnAccent => self.on_accent,
            Role::Shadow => self.shadow,
        }
    }

    pub fn font(&self, r: FontRole) -> FontKey {
        match r {
            FontRole::Display => self.display_font,
            FontRole::Body => self.body_font,
            FontRole::Mono => self.mono_font,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::styles::spec;

    fn rgba(r: u8, g: u8, b: u8, a: u8) -> Rgba {
        Rgba { r, g, b, a }
    }

    #[test]
    fn each_style_is_drawn_in_the_colours_and_fonts_of_its_row() {
        // v4implementation §8.4: text, stroke, accent, accent_alt, panel.
        let rows = [
            (
                StyleId::Clean,
                (255, 255, 255),
                (255, 214, 10),
                (16, 16, 20),
            ),
            (StyleId::Bold, (255, 255, 255), (255, 94, 58), (16, 16, 20)),
            (StyleId::Tech, (232, 240, 234), (61, 220, 132), (11, 15, 12)),
        ];
        for (style, text, alt, panel) in rows {
            let theme = Theme::default_for(style);
            assert_eq!(theme.text, rgba(text.0, text.1, text.2, 255), "{style:?}");
            assert_eq!(theme.stroke, rgba(16, 16, 20, 255), "{style:?}");
            assert_eq!(theme.accent_alt, rgba(alt.0, alt.1, alt.2, 255));
            assert_eq!(theme.panel, rgba(panel.0, panel.1, panel.2, 217));
            assert_eq!(theme.backdrop, rgba(0, 0, 0, 255), "{style:?}");
            // The pill's word #111114 and a shadow's black (D-170).
            assert_eq!(theme.on_accent, rgba(17, 17, 20, 255), "{style:?}");
            assert_eq!(theme.shadow, rgba(0, 0, 0, 255), "{style:?}");
            assert_eq!(theme.body_font, INTER_700, "{style:?}");
            assert_eq!(theme.mono_font, JETBRAINS_MONO_700, "{style:?}");
        }
        let accent = |style| Theme::default_for(style).accent;
        assert_eq!(accent(StyleId::Clean), rgba(255, 214, 10, 255));
        assert_eq!(accent(StyleId::Bold), rgba(255, 214, 10, 255));
        assert_eq!(accent(StyleId::Tech), rgba(61, 220, 132, 255));
        // Clean's KeywordPop is its accent.
        assert_eq!(
            Theme::default_for(StyleId::Clean).accent_alt,
            accent(StyleId::Clean)
        );
    }

    #[test]
    fn each_style_sets_its_captions_in_its_rows_font() {
        // Inter 800 (key 4, D-170), Inter 900, JetBrains Mono 700.
        let fonts = [
            (StyleId::Clean, FontKey(4)),
            (StyleId::Bold, FontKey(1)),
            (StyleId::Tech, FontKey(2)),
        ];
        for (style, key) in fonts {
            let theme = Theme::default_for(style);
            assert_eq!(theme.font(spec(style).font), key, "{style:?}");
        }
    }
}
