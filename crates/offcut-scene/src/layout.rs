//! Text shaping through parley: from a string to glyphs with positions.

use parley::{FontFamily, FontFamilyName, FontWeight, Layout, PositionedLayoutItem, StyleProperty};

use crate::SceneError;
use crate::display_list::{FontId, PositionedGlyph};
use crate::fonts::{self, Fonts};

/// Glyphs of one font, one after the other on a baseline. A glyph's `x` is
/// measured from the start of the run; `width` is where the next run starts.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapedRun {
    pub font: FontId,
    pub size: f32,
    pub glyphs: Vec<PositionedGlyph>,
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub text: String,
}

/// `text` set in `font` at `size`. A character the font has no glyph for is
/// set in Noto Emoji, so the result may be several runs; a character that
/// font lacks too is left out and takes no room (TS §19.6).
pub fn shape(text: &str, font: FontId, size: f32) -> Result<Vec<ShapedRun>, SceneError> {
    if !(size.is_finite() && size > 0.0) {
        return Err(SceneError::Layout);
    }
    if text.is_empty() {
        return Ok(Vec::new());
    }
    fonts::with(|fonts| shape_with(fonts, text, font, size))
}

fn shape_with(
    fonts: &mut Fonts,
    text: &str,
    font: FontId,
    size: f32,
) -> Result<Vec<ShapedRun>, SceneError> {
    let layout = {
        let stack = [
            FontFamilyName::named(&fonts.faces.face(font).family),
            FontFamilyName::named(&fonts.faces.emoji.family),
        ];
        // A scale of 1 and no rounding: a scene is laid out in logical pixels.
        let mut builder = fonts
            .layout_cx
            .ranged_builder(&mut fonts.font_cx, text, 1.0, false);
        builder.push_default(StyleProperty::FontFamily(FontFamily::from(&stack[..])));
        builder.push_default(StyleProperty::FontSize(size));
        builder.push_default(StyleProperty::FontWeight(FontWeight::new(fonts::weight(
            font,
        ))));
        let mut layout: Layout<()> = builder.build(text);
        layout.break_all_lines(None);
        layout
    };

    let mut runs = Vec::new();
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let run = glyph_run.run();
            let Some(used) = fonts.faces.font_of(run.font().data.id(), font) else {
                return Err(SceneError::Font);
            };
            // A cluster is one or more characters and the glyphs that draw
            // them. Glyph 0 is the font's "no such character" box: a cluster
            // that holds one is left out, its text with it.
            let mut width = 0.0f32;
            let mut glyphs = Vec::new();
            for cluster in run.visual_clusters() {
                if cluster.glyphs().any(|glyph| glyph.id == 0) {
                    continue;
                }
                for glyph in cluster.glyphs() {
                    glyphs.push(PositionedGlyph {
                        id: glyph.id,
                        x: width + glyph.x,
                        y: glyph.y,
                    });
                    width += glyph.advance;
                }
            }
            if glyphs.is_empty() {
                continue;
            }
            let shown: String = run
                .clusters()
                .filter(|cluster| cluster.glyphs().all(|glyph| glyph.id != 0))
                .filter_map(|cluster| text.get(cluster.text_range()))
                .collect();
            let metrics = run.metrics();
            runs.push(ShapedRun {
                font: used,
                size,
                glyphs,
                width,
                ascent: metrics.ascent,
                descent: metrics.descent,
                text: shown,
            });
        }
    }
    Ok(runs)
}
