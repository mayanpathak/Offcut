//! The watermark of a Free export (D-143, TS §19.6): one line of text at the
//! bottom-left corner of the safe area. The scene draws it when, and only
//! when, the export profile asks (INV-9); nothing else draws or removes it.
//! All values are (assumption, alpha: PS §19 minor question).

use crate::SceneError;
use crate::display_list::{Affine, DrawCmd, Rect, Rgba, Stroke};
use crate::events::{Text, text};
use crate::fonts::FontSet;
use crate::safe_area::WATERMARK_ANCHOR;
use crate::theme::{FontRole, Role, Theme};

/// The working name (A-11).
pub const WATERMARK_TEXT: &str = "Made with Offcut";
/// Inter 700 (the theme's body font) at this size, in logical pixels,
pub const WATERMARK_SIZE: f32 = 36.0;
/// at this opacity, outlined this wide, its box's bottom on the safe area's
/// bottom: inside the band no caption reaches (`placement::WATERMARK_BAND`).
pub const WATERMARK_OPACITY: f32 = 0.6;
pub const WATERMARK_STROKE: f32 = 3.0;

/// The watermark laid out: shaped once.
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    text: Text,
    x: f32,
    baseline: f32,
    fill: Rgba,
    stroke: Stroke,
}

/// `WATERMARK_TEXT` in the theme's text colour with a dark outline, its
/// box's left and bottom (the font's descent and the outline) on the safe
/// area's: its baseline about 10 lp above the bottom. D-143 says 6, which
/// put the box 4 lp below the safe area (Entry 17).
pub fn layout(theme: &Theme, fonts: &FontSet) -> Result<Track, SceneError> {
    let font = (theme.font(FontRole::Body), fonts);
    let (left, bottom) = WATERMARK_ANCHOR;
    let pad = WATERMARK_STROKE / 2.0;
    let text = text(WATERMARK_TEXT, font, WATERMARK_SIZE)?;
    Ok(Track {
        x: left + pad,
        baseline: bottom - text.descent - pad,
        text,
        fill: theme.color(Role::Text),
        stroke: Stroke {
            width: WATERMARK_STROKE,
            color: theme.color(Role::Stroke),
        },
    })
}

/// Its box, from its outline's left to its right, from the font's ascent
/// above the baseline to its descent below, in logical pixels: what no
/// caption may meet (D-143).
pub fn bounds(track: &Track) -> Rect {
    let pad = track.stroke.width / 2.0;
    Rect {
        x: track.x - pad,
        y: track.baseline - track.text.ascent - pad,
        w: track.text.width + 2.0 * pad,
        h: track.text.height() + 2.0 * pad,
    }
}

/// Adds the watermark: the same at every frame.
pub fn draw(track: &Track, out: &mut Vec<DrawCmd>) {
    out.push(DrawCmd::PushLayer {
        opacity: WATERMARK_OPACITY,
        clip: None,
    });
    let mut x = track.x;
    for run in &track.text.runs {
        out.push(DrawCmd::GlyphRun {
            font: run.font,
            size: run.size,
            glyphs: run.glyphs.clone(),
            fill: track.fill,
            stroke: Some(track.stroke),
            transform: Affine::scale_then_translate(1.0, x, track.baseline),
            text: run.text.clone(),
        });
        x += run.width;
    }
    out.push(DrawCmd::PopLayer);
}

#[cfg(test)]
mod tests {
    use offcut_types::StyleId;

    use super::*;
    use crate::display_list::FontKey;
    use crate::placement::WATERMARK_BAND;
    use crate::safe_area::SAFE_AREA;

    #[test]
    fn made_with_offcut_in_inter_700_at_36_lp_at_60_percent_in_the_bottom_left_corner() {
        let theme = Theme::default_for(StyleId::Clean);
        let track = layout(&theme, &FontSet::embedded()).unwrap();
        let mut out = Vec::new();
        draw(&track, &mut out);
        assert_eq!(
            out[0],
            DrawCmd::PushLayer {
                opacity: 0.6,
                clip: None
            }
        );
        assert_eq!(out.last(), Some(&DrawCmd::PopLayer));
        let DrawCmd::GlyphRun {
            font,
            size,
            fill,
            stroke,
            transform,
            text,
            ..
        } = &out[1]
        else {
            panic!("the text");
        };
        assert_eq!(
            (text.as_str(), *font, *size),
            (WATERMARK_TEXT, FontKey(0), 36.0)
        );
        assert_eq!(
            (*fill, *stroke),
            (
                theme.text,
                Some(Stroke {
                    width: 3.0,
                    color: theme.stroke
                })
            )
        );
        // Its box's left and bottom on the safe area's; the baseline about
        // 10 lp above the bottom; the box inside the band kept for it.
        assert_eq!(transform.m[4] - 1.5, SAFE_AREA.x);
        let (bottom, b) = (SAFE_AREA.y + SAFE_AREA.h, bounds(&track));
        assert_eq!(b.x, SAFE_AREA.x);
        assert!((b.y + b.h - bottom).abs() < 1e-3, "{b:?}");
        let rise = bottom - transform.m[5];
        assert!((10.0..10.5).contains(&rise), "{rise}");
        assert!(b.h < WATERMARK_BAND, "{b:?}");
    }
}
