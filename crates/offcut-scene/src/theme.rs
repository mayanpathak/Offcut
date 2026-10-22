//! [ONLY colour values] The colours and fonts a style is drawn in, by role
//! (TS §40.5, D-136). A style names roles; a theme gives each role its value,
//! so that a brand kit (V6.7) is one `Theme`. All values are (assumption),
//! reviewed in E-2.

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
    /// The word that is being spoken.
    pub accent: Rgba,
    /// A KeywordPop word, FromTo's "to" value.
    pub accent_alt: Rgba,
    pub panel: Rgba,
    pub backdrop: Rgba,
    pub display_font: FontKey,
    pub body_font: FontKey,
    pub mono_font: FontKey,
}

const WHITE: Rgba = Rgba {
    r: 255,
    g: 255,
    b: 255,
    a: 255,
};
const DARK: Rgba = Rgba {
    r: 16,
    g: 16,
    b: 20,
    a: 255,
};
const ACCENT: Rgba = Rgba {
    r: 255,
    g: 214,
    b: 10,
    a: 255,
};
/// `#101014` at 85%. No V3 style draws a panel.
const PANEL: Rgba = Rgba {
    r: 16,
    g: 16,
    b: 20,
    a: 217,
};
/// What the canvas shows where no video is (D-133).
const BLACK: Rgba = Rgba {
    r: 0,
    g: 0,
    b: 0,
    a: 255,
};

/// Keys of `FontSet::embedded()`.
const INTER_700: FontKey = FontKey(0);
const JETBRAINS_MONO_700: FontKey = FontKey(2);

/// V3's look: white text, a dark stroke, the spoken word in yellow, Inter
/// 700. V3 drew every style as Clean (V4 Prompt 105 gives each its own).
fn v3() -> Theme {
    Theme {
        text: WHITE,
        stroke: DARK,
        accent: ACCENT,
        accent_alt: ACCENT,
        panel: PANEL,
        backdrop: BLACK,
        display_font: INTER_700,
        body_font: INTER_700,
        mono_font: JETBRAINS_MONO_700,
    }
}

impl Theme {
    pub fn default_for(style: StyleId) -> Theme {
        match style {
            StyleId::Clean | StyleId::Bold | StyleId::Tech => v3(),
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

    /// V3's constants, as `styles.rs` held them before V4.
    const fn rgba(r: u8, g: u8, b: u8) -> Rgba {
        Rgba { r, g, b, a: 255 }
    }

    #[test]
    fn every_style_is_drawn_in_v3s_colours_and_font() {
        for style in [StyleId::Clean, StyleId::Bold, StyleId::Tech] {
            let theme = Theme::default_for(style);
            let row = spec(style);
            // Fill white, stroke #101014, the spoken word and a pop #FFD60A.
            assert_eq!(theme.color(row.fill), rgba(255, 255, 255), "{style:?}");
            assert_eq!(theme.color(row.stroke), rgba(16, 16, 20), "{style:?}");
            assert_eq!(theme.color(row.active), rgba(255, 214, 10), "{style:?}");
            assert_eq!(theme.color(row.pop), rgba(255, 214, 10), "{style:?}");
            // Inter 700, V2's FontId::Inter700.
            assert_eq!(theme.font(row.font), FontKey(0), "{style:?}");
            assert_eq!(theme, Theme::default_for(StyleId::Clean));
        }
    }
}
