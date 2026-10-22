//! Drawing a chunk (TS §5): its glyph runs at a time, in the theme's colours,
//! moving as the style's motion says, each word in the colour of its first
//! layer of emphasis (D-169). Where each word is, is `captions`'.

use offcut_types::{OutMs, WordIdx};

use crate::captions::Chunk;
use crate::display_list::{Affine, DrawCmd, Stroke};
use crate::motion::{ActiveTreatment, enter_at};
use crate::styles::StyleSpec;
use crate::theme::{Role, Theme};

/// What a chunk's words are emphasized by besides the spoken word (D-169).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Emphasis {
    /// The word of each KeywordPop item and its program start: from then
    /// on, while its chunk is shown, the word is in the style's `pop`.
    pub pops: Vec<(WordIdx, OutMs)>,
    /// `EditState.highlight_numbers`: a caption word that shows a number is
    /// in `accent`.
    pub numbers: bool,
}

/// The role a word is filled with: the first that applies of the spoken
/// word (through the style's active treatment), a KeywordPop word, a number
/// when numbers are highlighted, and the fill. A style whose active word is
/// not a colour (Tech's cursor) leaves the spoken word to the next layers.
fn fill_role(style: &StyleSpec, active: bool, popped: bool, number: bool, numbers: bool) -> Role {
    let highlights = matches!(style.motion.active, ActiveTreatment::Highlight);
    if active && highlights {
        style.active
    } else if popped {
        style.pop
    } else if number && numbers {
        Role::Accent
    } else {
        style.fill
    }
}

/// `draw_with` and no emphasis besides the spoken word: V3's look.
pub fn draw(chunk: &Chunk, t: OutMs, style: &StyleSpec, theme: &Theme, out: &mut Vec<DrawCmd>) {
    draw_with(chunk, t, style, theme, &Emphasis::default(), out);
}

/// Adds the chunk as it looks at `t`: each word in the colour of its first
/// layer of emphasis, each moving as the style's motion says.
pub fn draw_with(
    chunk: &Chunk,
    t: OutMs,
    style: &StyleSpec,
    theme: &Theme,
    emphasis: &Emphasis,
    out: &mut Vec<DrawCmd>,
) {
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
            let popped = word.is_some_and(|word| {
                let pop = |&(at, start): &(WordIdx, OutMs)| at == word.word && start <= t;
                emphasis.pops.iter().any(pop)
            });
            let number = word.is_some_and(|word| word.number);
            let fill = theme.color(fill_role(style, active, popped, number, emphasis.numbers));
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

#[cfg(test)]
mod tests {
    use offcut_types::StyleId;

    use super::*;
    use crate::motion::ActiveTreatment;
    use crate::styles::spec;

    #[test]
    fn the_spoken_word_then_a_keyword_pop_then_a_number_then_the_fill() {
        // Four roles that differ, so that each layer shows which won.
        let style = StyleSpec {
            active: Role::Panel,
            pop: Role::AccentAlt,
            fill: Role::Text,
            ..spec(StyleId::Clean)
        };
        for popped in [false, true] {
            for number in [false, true] {
                for numbers in [false, true] {
                    // The spoken word wins over every other layer.
                    assert_eq!(
                        fill_role(&style, true, popped, number, numbers),
                        Role::Panel
                    );
                    let quiet = fill_role(&style, false, popped, number, numbers);
                    let expected = if popped {
                        Role::AccentAlt
                    } else if number && numbers {
                        Role::Accent
                    } else {
                        Role::Text
                    };
                    assert_eq!(quiet, expected, "{popped} {number} {numbers}");
                }
            }
        }
        // Numbers are coloured only when highlighted.
        assert_eq!(fill_role(&style, false, false, true, false), Role::Text);
        // A cursor is not a colour: the spoken word takes the next layer.
        let mut cursor = style.clone();
        cursor.motion.active = ActiveTreatment::Cursor;
        assert_eq!(fill_role(&cursor, true, false, true, true), Role::Accent);
        assert_eq!(fill_role(&cursor, true, true, false, true), Role::AccentAlt);
        assert_eq!(fill_role(&cursor, true, false, false, true), Role::Text);
    }
}
