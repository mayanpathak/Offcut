//! Text shaping through parley: from a string to glyphs with positions.

use parley::PositionedLayoutItem;

use crate::SceneError;
use crate::display_list::{FontKey, PositionedGlyph};
use crate::fonts::{self, FontSet, Shaper};

/// Glyphs of one face, one after the other on a baseline. A glyph's `x` is
/// measured from the start of the run; `width` is where the next run starts.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapedRun {
    pub font: FontKey,
    pub size: f32,
    pub glyphs: Vec<PositionedGlyph>,
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub text: String,
}

/// `text` set in the face `key` of `fonts` at `size`. A character the face
/// has no glyph for is set in `FontSet::EMOJI`, so the result may be several
/// runs; a character that face lacks too is left out and takes no room
/// (TS §19.6).
pub fn shape(
    text: &str,
    key: FontKey,
    size: f32,
    fonts: &FontSet,
) -> Result<Vec<ShapedRun>, SceneError> {
    if !(size.is_finite() && size > 0.0) {
        return Err(SceneError::Layout);
    }
    if text.is_empty() {
        return Ok(Vec::new());
    }
    fonts::with(|shaper| shape_with(shaper, text, key, size, fonts))
}

fn shape_with(
    shaper: &mut Shaper,
    text: &str,
    key: FontKey,
    size: f32,
    fonts: &FontSet,
) -> Result<Vec<ShapedRun>, SceneError> {
    let face = fonts.get(key).ok_or(SceneError::Font)?;
    let emoji = fonts.get(FontSet::EMOJI).ok_or(SceneError::Font)?;
    shaper.ensure(fonts)?;
    let layout = shaper.lay_out(text, (&face.family, face.weight), &emoji.family, size);

    let mut runs = Vec::new();
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let run = glyph_run.run();
            let Some(used) = shaper.key_of(run.font().data.id(), key, fonts) else {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_character_the_face_lacks_is_set_in_the_emoji_face_and_a_missing_key_fails() {
        let fonts = FontSet::embedded();
        let runs = shape("ok \u{1F600}", FontKey(0), 64.0, &fonts).unwrap();
        let keys: Vec<FontKey> = runs.iter().map(|run| run.font).collect();
        assert_eq!(keys, [FontKey(0), FontSet::EMOJI]);
        assert_eq!(shape("ok", FontKey(9), 64.0, &fonts), Err(SceneError::Font));
        assert_eq!(
            shape("ok", FontKey(0), 64.0, &FontSet::default()),
            Err(SceneError::Font)
        );
    }
}
