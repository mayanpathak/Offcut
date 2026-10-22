//! Drawing a chunk (TS §5): its glyph runs at a time, in the theme's colours,
//! moving as the style's motion says. Where each word is, is `captions`'.

use offcut_types::OutMs;

use crate::captions::Chunk;
use crate::display_list::{Affine, DrawCmd, Stroke};
use crate::motion::{ActiveTreatment, enter_at};
use crate::styles::StyleSpec;
use crate::theme::Theme;

/// Adds the chunk as it looks at `t`: the word that is being spoken in the
/// active colour, the others in the fill colour, each moving as the style's
/// motion says.
pub fn draw(chunk: &Chunk, t: OutMs, style: &StyleSpec, theme: &Theme, out: &mut Vec<DrawCmd>) {
    let motion = &style.motion;
    let stroke = (style.stroke_width > 0.0).then_some(Stroke {
        width: style.stroke_width,
        color: theme.color(style.stroke),
    });
    // A fade is the chunk's, from its show start; a pop is each word's.
    let (opacity, _) = enter_at(motion, t.get().saturating_sub(chunk.show.start.get()));
    let fading = opacity < 1.0;
    if fading {
        out.push(DrawCmd::PushLayer {
            opacity,
            clip: None,
        });
    }
    for line in &chunk.lines {
        for placed in &line.words {
            let word = chunk.words.get(placed.word);
            let active = word.is_some_and(|word| word.start <= t && t < word.end);
            let fill = match motion.active {
                ActiveTreatment::Highlight if active => theme.color(style.active),
                ActiveTreatment::Highlight => theme.color(style.fill),
                // V4: the block after the spoken word (TS §19.4, V4 Prompt 105).
                ActiveTreatment::Cursor => theme.color(style.fill),
            };
            let since = word.map_or(0, |word| t.get().saturating_sub(word.start.get()));
            let (_, pop) = enter_at(motion, since);
            // A popping word scales about its centre on the baseline.
            let width: f32 = placed.runs.iter().map(|run| run.width).sum();
            let centre = line.x + (placed.x + width / 2.0) * line.scale;
            let mut x = placed.x;
            for run in &placed.runs {
                let left = line.x + x * line.scale;
                let transform = if pop == 1.0 {
                    Affine::scale_then_translate(line.scale, left, line.baseline)
                } else {
                    let left = centre + (left - centre) * pop;
                    Affine::scale_then_translate(line.scale * pop, left, line.baseline)
                };
                out.push(DrawCmd::GlyphRun {
                    font: run.font,
                    size: run.size,
                    glyphs: run.glyphs.clone(),
                    fill,
                    stroke,
                    transform,
                    text: run.text.clone(),
                });
                x += run.width;
            }
        }
    }
    if fading {
        out.push(DrawCmd::PopLayer);
    }
}
