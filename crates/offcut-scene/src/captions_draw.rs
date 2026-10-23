//! Drawing a chunk (TS §5, v4implementation §8.6, §8.9): its glyph runs at a
//! time, in the theme's colours, moving as the style's motion says, each
//! word in the colour of its first layer of emphasis (D-169), with the
//! style's panel, shadow, pill and cursor (D-139, D-170). Where each word is,
//! is `captions`'; nothing drawn here changes a layout box.

use offcut_types::{OutMs, WordIdx};

use crate::captions::{CaptionWord, Chunk};
use crate::display_list::{Affine, DrawCmd, Rect, Rgba, Stroke};
use crate::easing::{Easing, ease};
use crate::hit::word_box;
use crate::layout::ShapedRun;
use crate::motion::{ActiveTreatment, ShadowSpec, enter_at, keyword_scale};
use crate::safe_area::SAFE_AREA;
use crate::styles::{CURSOR_WIDTH_EM, PILL_PADDING, PILL_RADIUS, PanelSpec, StyleSpec};
use crate::theme::{Role, Theme};

/// The pill's move from the word before to the spoken word, from the spoken
/// word's start, and the scale it grows from on the way (D-170).
pub const PILL_SLIDE_MS: u32 = 90;
pub const PILL_GROW_FROM: f32 = 0.9;
const PILL_EASING: Easing = Easing::EaseOutCubic;

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
/// when numbers are highlighted, and the fill. The spoken word is in the
/// style's `active` (Tech's beside its cursor, D-139), or in `OnAccent` on
/// Clean's pill, which is in `active` (D-170).
fn fill_role(style: &StyleSpec, active: bool, popped: bool, number: bool, numbers: bool) -> Role {
    if active {
        return match style.motion.active {
            ActiveTreatment::Highlight | ActiveTreatment::Cursor => style.active,
            ActiveTreatment::Pill => Role::OnAccent,
        };
    }
    if popped {
        style.pop
    } else if number && numbers {
        Role::Accent
    } else {
        style.fill
    }
}

/// How much bigger than laid out `word` is drawn at `t`: a KeywordPop's
/// scale from its start until the word ends, else the style's pop.
fn word_scale(word: &CaptionWord, t: OutMs, style: &StyleSpec, emphasis: &Emphasis) -> f32 {
    let popped_from = emphasis
        .pops
        .iter()
        .filter(|&&(at, _)| at == word.word)
        .map(|&(_, start)| start)
        .min();
    if let Some(start) = popped_from
        && start <= t
        && t < word.end
    {
        return keyword_scale(t.get() - start.get());
    }
    enter_at(&style.motion, t.get().saturating_sub(word.start.get())).1
}

/// `r` scaled by `k` about the point `about`.
fn scaled_about(r: Rect, (x, y): (f32, f32), k: f32) -> Rect {
    Rect {
        x: x + (r.x - x) * k,
        y: y + (r.y - y) * k,
        w: r.w * k,
        h: r.h * k,
    }
}

/// The largest scale a word with the box `r`, on a line drawn at
/// `line_scale`, can be drawn at about `(x, y)` with what is drawn around it
/// inside the safe area, and never less than 1: a word that fills its line
/// pops less rather than leave the area (TS §19.6). Its outline, half the
/// stroke, grows with it; the pill's padding, the panel's, the cursor's cell
/// and the shadow's move do not.
fn room_to_grow(r: Rect, (x, y): (f32, f32), style: &StyleSpec, line_scale: f32) -> f32 {
    let stroke = style.stroke_width.max(0.0) / 2.0;
    let outline = stroke * line_scale;
    let pill = match style.motion.active {
        ActiveTreatment::Pill => PILL_PADDING.0,
        ActiveTreatment::Highlight | ActiveTreatment::Cursor => 0.0,
    };
    let fixed = (style.decoration_width() / 2.0 - stroke).max(pill);
    let drop = style.shadow.map_or(0.0, |s| s.offset.1.abs());
    let area = SAFE_AREA;
    // How far the outline may reach from the point, over how far it reaches.
    let limit = |room: f32, reach: f32| room / (reach + outline);
    let across =
        limit(x - area.x - fixed, x - r.x).min(limit(area.x + area.w - fixed - x, r.x + r.w - x));
    let down = limit(y - area.y - fixed, y - r.y)
        .min(limit(area.y + area.h - fixed - drop - y, r.y + r.h - y));
    across.min(down).max(1.0)
}

/// The pill `since_ms` after the spoken word's start, and its corner
/// radius: on its way from the box `from` to the box `to`, padded, growing
/// from `PILL_GROW_FROM` to its size as it arrives (D-170).
fn pill(from: Rect, to: Rect, since_ms: u32) -> (Rect, f32) {
    let e = ease(PILL_EASING, since_ms as f32 / PILL_SLIDE_MS as f32);
    // Exactly `a` at the start and `b` at the end.
    let mix = |a: f32, b: f32| a * (1.0 - e) + b * e;
    let k = mix(PILL_GROW_FROM, 1.0);
    let (pad_x, pad_y) = PILL_PADDING;
    let w = (mix(from.w, to.w) + 2.0 * pad_x) * k;
    let h = (mix(from.h, to.h) + 2.0 * pad_y) * k;
    let x = mix(from.x + from.w / 2.0, to.x + to.w / 2.0);
    let y = mix(from.y + from.h / 2.0, to.y + to.h / 2.0);
    let rect = Rect {
        x: x - w / 2.0,
        y: y - h / 2.0,
        w,
        h,
    };
    (rect, PILL_RADIUS * k)
}

/// The width of a cursor's cell on a line drawn at `scale`.
fn cursor_cell(style: &StyleSpec, scale: f32) -> f32 {
    match style.motion.active {
        ActiveTreatment::Cursor => CURSOR_WIDTH_EM * style.size * scale,
        ActiveTreatment::Highlight | ActiveTreatment::Pill => 0.0,
    }
}

/// Tech's panel: the box of the chunk's lines and of a cursor's cell on
/// either side of each, so that the cursor stays on it, plus the padding.
fn panel_rect(chunk: &Chunk, style: &StyleSpec, panel: &PanelSpec) -> Option<Rect> {
    let mut edges: Option<(f32, f32, f32, f32)> = None;
    for line in &chunk.lines {
        let cell = cursor_cell(style, line.scale);
        for placed in &line.words {
            let r = word_box(line, placed);
            let (left, top, right, bottom) = (r.x - cell, r.y, r.x + r.w + cell, r.y + r.h);
            edges = Some(edges.map_or((left, top, right, bottom), |(l, t, r, b)| {
                (l.min(left), t.min(top), r.max(right), b.max(bottom))
            }));
        }
    }
    let (left, top, right, bottom) = edges?;
    let pad = panel.padding;
    Some(Rect {
        x: left - pad,
        y: top - pad,
        w: right - left + 2.0 * pad,
        h: bottom - top + 2.0 * pad,
    })
}

/// A word of the chunk as it is drawn at a time.
struct Drawn<'a> {
    /// Its place in the chunk's `words`.
    index: usize,
    /// Its glyph box, at the size it is drawn.
    rect: Rect,
    /// The line's scale times the word's own.
    scale: f32,
    runs: Vec<(&'a ShapedRun, Affine)>,
    fill: Rgba,
    /// Its fill were it not spoken: the spoken word's look off the pill.
    quiet: Rgba,
    active: bool,
}

/// Each word of `chunk` as it is drawn at `t`, in the order of its words.
fn lay_out<'a>(
    chunk: &'a Chunk,
    t: OutMs,
    style: &StyleSpec,
    theme: &Theme,
    emphasis: &Emphasis,
) -> Vec<Drawn<'a>> {
    let mut drawn = Vec::new();
    for line in &chunk.lines {
        for placed in &line.words {
            let Some(word) = chunk.words.get(placed.word) else {
                continue;
            };
            let active = word.start <= t && t < word.end;
            let popped = emphasis
                .pops
                .iter()
                .any(|&(at, start)| at == word.word && start <= t);
            let role = fill_role(style, active, popped, word.number, emphasis.numbers);
            let quiet = fill_role(style, false, popped, word.number, emphasis.numbers);
            // A popping word scales about its centre on the baseline, as
            // far as the safe area lets it.
            let laid_out = word_box(line, placed);
            let centre = laid_out.x + laid_out.w / 2.0;
            let room = room_to_grow(laid_out, (centre, line.baseline), style, line.scale);
            let pop = word_scale(word, t, style, emphasis).min(room);
            let mut runs = Vec::with_capacity(placed.runs.len());
            let mut x = placed.x;
            for run in &placed.runs {
                let left = line.x + x * line.scale;
                let transform = if pop == 1.0 {
                    Affine::scale_then_translate(line.scale, left, line.baseline)
                } else {
                    let left = centre + (left - centre) * pop;
                    Affine::scale_then_translate(line.scale * pop, left, line.baseline)
                };
                runs.push((run, transform));
                x += run.width;
            }
            drawn.push(Drawn {
                index: placed.word,
                rect: scaled_about(laid_out, (centre, line.baseline), pop),
                scale: line.scale * pop,
                runs,
                fill: theme.color(role),
                quiet: theme.color(quiet),
                active,
            });
        }
    }
    drawn
}

fn glyph_run(run: &ShapedRun, fill: Rgba, stroke: Option<Stroke>, transform: Affine) -> DrawCmd {
    DrawCmd::GlyphRun {
        font: run.font,
        size: run.size,
        glyphs: run.glyphs.clone(),
        fill,
        stroke,
        transform,
        text: run.text.clone(),
    }
}

/// Every run but the spoken word's when `spoken` is false, again, moved by
/// the offset, in the shadow's colour, inside one layer at its opacity:
/// overlapping parts of a shadow do not darken each other.
fn draw_shadow(
    words: &[Drawn<'_>],
    spoken: bool,
    style: &StyleSpec,
    theme: &Theme,
    out: &mut Vec<DrawCmd>,
) {
    let Some(ShadowSpec {
        offset: (dx, dy),
        role,
        opacity,
    }) = style.shadow
    else {
        return;
    };
    let cast: Vec<&Drawn<'_>> = words.iter().filter(|w| spoken || !w.active).collect();
    if cast.is_empty() {
        return;
    }
    let color = theme.color(role);
    let stroke = (style.stroke_width > 0.0).then_some(Stroke {
        width: style.stroke_width,
        color,
    });
    out.push(DrawCmd::PushLayer {
        opacity,
        clip: None,
    });
    for word in cast {
        for &(run, transform) in &word.runs {
            let [a, b, c, d, e, f] = transform.m;
            let moved = Affine {
                m: [a, b, c, d, e + dx, f + dy],
            };
            out.push(glyph_run(run, color, stroke, moved));
        }
    }
    out.push(DrawCmd::PopLayer);
}

/// `draw_with` and no emphasis besides the spoken word.
pub fn draw(chunk: &Chunk, t: OutMs, style: &StyleSpec, theme: &Theme, out: &mut Vec<DrawCmd>) {
    draw_with(chunk, t, style, theme, &Emphasis::default(), out);
}

/// Adds the chunk as it looks at `t`, in this order: Tech's panel, the
/// shadow, Clean's pill, the words (each in the colour of its first layer of
/// emphasis, each moving as the style's motion says), Tech's cursor.
pub fn draw_with(
    chunk: &Chunk,
    t: OutMs,
    style: &StyleSpec,
    theme: &Theme,
    emphasis: &Emphasis,
    out: &mut Vec<DrawCmd>,
) {
    let treatment = style.motion.active;
    let stroke = (style.stroke_width > 0.0).then_some(Stroke {
        width: style.stroke_width,
        color: theme.color(style.stroke),
    });
    // A fade is the chunk's, from its show start; a pop is each word's.
    let since_shown = t.get().saturating_sub(chunk.show.start.get());
    let (opacity, _) = enter_at(&style.motion, since_shown);
    let fading = opacity < 1.0;
    if fading {
        out.push(DrawCmd::PushLayer {
            opacity,
            clip: None,
        });
    }
    if let Some(panel) = &style.panel
        && let Some(rect) = panel_rect(chunk, style, panel)
    {
        out.push(DrawCmd::FillRect {
            rect,
            radius: panel.radius,
            color: theme.color(panel.role),
        });
    }

    let words = lay_out(chunk, t, style, theme, emphasis);
    let spoken = words.iter().find(|word| word.active);
    let on_pill = matches!(treatment, ActiveTreatment::Pill);
    // The pill exists only while a word is spoken. It comes from the word
    // before; the chunk's first word grows on its own box.
    let pill_now = spoken
        .filter(|_| on_pill)
        .and_then(|spoken| Some((spoken, chunk.words.get(spoken.index)?)))
        .map(|(spoken, word)| {
            let before = spoken.index.checked_sub(1);
            let from = words.iter().find(|w| Some(w.index) == before);
            let since = t.get().saturating_sub(word.start.get());
            let (rect, radius) = pill(from.map_or(spoken.rect, |w| w.rect), spoken.rect, since);
            (rect, radius, since < PILL_SLIDE_MS)
        });
    // While the pill slides in, the spoken word is drawn as it looks off the
    // pill, under it, and in ink only where the pill has reached it. At rest
    // the pill holds it all: ink, no outline, no shadow (D-170).
    let sliding = pill_now.is_some_and(|(_, _, sliding)| sliding);
    draw_shadow(&words, !on_pill || sliding, style, theme, out);
    if sliding && let Some(spoken) = spoken {
        for &(run, transform) in &spoken.runs {
            out.push(glyph_run(run, spoken.quiet, stroke, transform));
        }
    }
    if let Some((rect, radius, _)) = pill_now {
        out.push(DrawCmd::FillRect {
            rect,
            radius,
            color: theme.color(style.active),
        });
    }
    for word in &words {
        let inked = on_pill && word.active;
        // Only where the pill is, while it slides in.
        let clip = pill_now.filter(|_| inked && sliding).map(|(rect, ..)| rect);
        if clip.is_some() {
            out.push(DrawCmd::PushLayer { opacity: 1.0, clip });
        }
        // The word on the pill has no outline: the pill sets it off.
        let stroke = if inked { None } else { stroke };
        for &(run, transform) in &word.runs {
            out.push(glyph_run(run, word.fill, stroke, transform));
        }
        if clip.is_some() {
            out.push(DrawCmd::PopLayer);
        }
    }
    if matches!(treatment, ActiveTreatment::Cursor)
        && let Some(spoken) = spoken
    {
        let r = spoken.rect;
        out.push(DrawCmd::FillRect {
            rect: Rect {
                x: r.x + r.w,
                y: r.y,
                w: cursor_cell(style, spoken.scale),
                h: r.h,
            },
            radius: 0.0,
            color: theme.color(style.active),
        });
    }
    if fading {
        out.push(DrawCmd::PopLayer);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use offcut_types::{CaptionPlacement, DurMs, StyleId, TimeMap};

    use super::*;
    use crate::captions::{chunk, words};
    use crate::fonts::FontSet;
    use crate::hit::word_boxes;
    use crate::motion::POP_SCALE;
    use crate::safe_area::{ZoneId, zone};
    use crate::styles::spec;
    use crate::testing::{transcript, word};

    /// One chunk: "made" ends 100 ms before "it" starts.
    const WE_MADE_IT: [(&str, u32, u32); 3] = [
        ("we", 1_000, 1_200),
        ("made", 1_250, 1_500),
        ("it", 1_600, 1_800),
    ];

    fn chunk_in(style: &StyleSpec, theme: &Theme) -> Chunk {
        chunk_with(&WE_MADE_IT, style, theme)
    }

    fn chunk_with(timed: &[(&str, u32, u32)], style: &StyleSpec, theme: &Theme) -> Chunk {
        let spoken: Vec<_> = timed.iter().map(|w| word(w.0, w.1, w.2)).collect();
        let t = transcript(&spoken);
        let map = TimeMap::identity(DurMs::new(10_000));
        let shown = words(&t, &BTreeMap::new(), &map);
        let block = zone(ZoneId::Caption, CaptionPlacement::bottom());
        let fonts = FontSet::embedded();
        let mut chunks = chunk(&shown, &t.sentences, style, theme, &fonts, block).unwrap();
        assert_eq!(chunks.len(), 1);
        chunks.remove(0)
    }

    fn chunk_of(style: StyleId) -> (Chunk, StyleSpec, Theme) {
        let (row, theme) = (spec(style), Theme::default_for(style));
        (chunk_in(&row, &theme), row, theme)
    }

    fn drawn(
        chunk: &Chunk,
        t: u32,
        style: &StyleSpec,
        theme: &Theme,
        e: &Emphasis,
    ) -> Vec<DrawCmd> {
        let mut out = Vec::new();
        draw_with(chunk, OutMs::new(t), style, theme, e, &mut out);
        out
    }

    fn plain(chunk: &Chunk, t: u32, style: &StyleSpec, theme: &Theme) -> Vec<DrawCmd> {
        drawn(chunk, t, style, theme, &Emphasis::default())
    }

    /// The rectangles of a list: box, radius, colour.
    fn rects(cmds: &[DrawCmd]) -> Vec<(Rect, f32, Rgba)> {
        cmds.iter()
            .filter_map(|cmd| match cmd {
                DrawCmd::FillRect {
                    rect,
                    radius,
                    color,
                } => Some((*rect, *radius, *color)),
                DrawCmd::FillPath { .. }
                | DrawCmd::GlyphRun { .. }
                | DrawCmd::PushLayer { .. }
                | DrawCmd::PopLayer
                | DrawCmd::Video { .. }
                | DrawCmd::Image { .. } => None,
            })
            .collect()
    }

    /// The glyph runs of a list: text, fill, stroke width, transform.
    fn runs(cmds: &[DrawCmd]) -> Vec<(String, Rgba, Option<f32>, Affine)> {
        cmds.iter()
            .filter_map(|cmd| match cmd {
                DrawCmd::GlyphRun {
                    text,
                    fill,
                    stroke,
                    transform,
                    ..
                } => Some((text.clone(), *fill, stroke.map(|s| s.width), *transform)),
                DrawCmd::FillRect { .. }
                | DrawCmd::FillPath { .. }
                | DrawCmd::PushLayer { .. }
                | DrawCmd::PopLayer
                | DrawCmd::Video { .. }
                | DrawCmd::Image { .. } => None,
            })
            .collect()
    }

    /// The runs drawn over the shadow, by their fill.
    fn words_of(cmds: &[DrawCmd], theme: &Theme) -> Vec<(String, Rgba, Option<f32>, Affine)> {
        let mut words = runs(cmds);
        words.retain(|run| run.1 != theme.shadow);
        words
    }

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    fn same(a: Rect, b: Rect) -> bool {
        near(a.x, b.x) && near(a.y, b.y) && near(a.w, b.w) && near(a.h, b.h)
    }

    fn centre(r: Rect) -> (f32, f32) {
        (r.x + r.w / 2.0, r.y + r.h / 2.0)
    }

    fn padded(r: Rect) -> Rect {
        Rect {
            x: r.x - 12.0,
            y: r.y - 4.0,
            w: r.w + 24.0,
            h: r.h + 8.0,
        }
    }

    fn overlap(a: Rect, b: Rect) -> bool {
        a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
    }

    #[test]
    fn the_spoken_word_then_a_keyword_pop_then_a_number_then_the_fill() {
        // Four roles that differ, so that each layer shows which won.
        let mut style = StyleSpec {
            active: Role::Panel,
            pop: Role::AccentAlt,
            fill: Role::Text,
            ..spec(StyleId::Clean)
        };
        style.motion.active = ActiveTreatment::Highlight;
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
        // Tech's spoken word is in its `active` beside its cursor (D-139,
        // D-170); Clean's is on the pill, in the colour drawn on the accent.
        let mut cursor = style.clone();
        cursor.motion.active = ActiveTreatment::Cursor;
        let mut pill = style.clone();
        pill.motion.active = ActiveTreatment::Pill;
        for (popped, number) in [(false, false), (true, false), (false, true)] {
            assert_eq!(fill_role(&cursor, true, popped, number, true), Role::Panel);
            assert_eq!(fill_role(&pill, true, popped, number, true), Role::OnAccent);
        }
        assert_eq!(fill_role(&pill, false, false, true, true), Role::Accent);
        assert_eq!(
            fill_role(&cursor, false, true, false, true),
            Role::AccentAlt
        );
    }

    #[test]
    fn the_pill_slides_from_the_word_before_and_grows_as_it_arrives() {
        let (chunk, style, theme) = chunk_of(StyleId::Clean);
        let boxes: Vec<Rect> = word_boxes(&chunk).into_iter().map(|(_, r)| r).collect();
        let (we, made, it) = (boxes[0], boxes[1], boxes[2]);
        let pill_at = |t: u32| {
            let found = rects(&plain(&chunk, t, &style, &theme));
            assert_eq!(found.len(), 1, "one pill at {t}");
            assert_eq!(found[0].2, theme.accent);
            (found[0].0, found[0].1)
        };

        // At "made"'s start: on the word before, at 0.9 of its size.
        let (start, radius) = pill_at(1_250);
        assert!(near(centre(start).0, centre(we).0) && near(centre(start).1, centre(we).1));
        assert!(near(start.w, (we.w + 24.0) * 0.9) && near(start.h, (we.h + 8.0) * 0.9));
        assert!(near(radius, 14.0 * 0.9));
        // At +45 ms, ease-out cubic: 87.5% of the way, at 0.9875.
        let (half, _) = pill_at(1_295);
        let mix = |a: f32, b: f32| a + (b - a) * 0.875;
        let x = mix(centre(we).0, centre(made).0);
        assert!(near(centre(half).0, x), "{half:?}");
        assert!(
            near(half.w, (mix(we.w, made.w) + 24.0) * 0.9875),
            "{half:?}"
        );
        // At +90 ms and until the word ends: on "made", padded 12 x 4 lp.
        for t in [1_340, 1_420, 1_499] {
            let (rest, radius) = pill_at(t);
            assert!(same(rest, padded(made)), "{t}: {rest:?}");
            assert!(near(radius, 14.0));
            // Behind exactly one word.
            assert!(!overlap(rest, we) && !overlap(rest, it), "{t}");
        }
        // It never jumps: millisecond by millisecond its centre moves one way,
        // never further than ease-out cubic's steepest step (3 / 90 of the way).
        let way = centre(made).0 - centre(we).0;
        let mut was = centre(we).0;
        for t in 1_251..=1_340 {
            let now = centre(pill_at(t).0).0;
            assert!((now - was) * way.signum() >= -1e-3, "{t}");
            assert!((now - was).abs() <= way.abs() * 3.0 / 90.0 + 1e-3, "{t}");
            was = now;
        }

        // The chunk's first word grows on its own box.
        let (first, _) = pill_at(1_000);
        assert!(near(centre(first).0, centre(we).0) && near(first.w, (we.w + 24.0) * 0.9));
        assert!(same(pill_at(1_090).0, padded(we)));

        // No pill while no word is spoken: before the first, in the pause
        // between "made" and "it", after the last.
        for t in [920, 999, 1_200, 1_249, 1_500, 1_550, 1_599, 1_800, 1_900] {
            assert!(rects(&plain(&chunk, t, &style, &theme)).is_empty(), "{t}");
        }
    }

    #[test]
    fn the_word_on_the_pill_is_in_ink_with_no_stroke_and_the_others_are_outlined() {
        let (chunk, style, theme) = chunk_of(StyleId::Clean);
        let words = words_of(&plain(&chunk, 1_340, &style, &theme), &theme);
        let shown: Vec<(&str, Rgba, Option<f32>)> =
            words.iter().map(|w| (w.0.as_str(), w.1, w.2)).collect();
        assert_eq!(
            shown,
            [
                ("we", theme.text, Some(5.0)),
                ("made", theme.on_accent, None),
                ("it", theme.text, Some(5.0)),
            ]
        );
    }

    #[test]
    fn the_shadow_is_each_other_run_again_4_lp_lower_in_one_layer_at_45_percent() {
        let (chunk, style, theme) = chunk_of(StyleId::Clean);
        // After the fade: the shadow's layer, the pill, the words.
        let cmds = plain(&chunk, 1_340, &style, &theme);
        assert_eq!(
            cmds[0],
            DrawCmd::PushLayer {
                opacity: 0.45,
                clip: None
            }
        );
        let close = cmds.iter().position(|c| *c == DrawCmd::PopLayer).unwrap();
        let shadow = runs(&cmds[1..close]);
        assert!(matches!(cmds[close + 1], DrawCmd::FillRect { .. }));
        let words = runs(&cmds[close + 2..]);
        assert_eq!(close + 2 + words.len(), cmds.len());
        // Not the word on the pill.
        let cast: Vec<&str> = shadow.iter().map(|run| run.0.as_str()).collect();
        assert_eq!(cast, ["we", "it"]);
        for run in &shadow {
            assert_eq!((run.1, run.2), (theme.shadow, Some(5.0)));
            let word = words.iter().find(|w| w.0 == run.0).unwrap();
            let [a, b, c, d, e, f] = word.3.m;
            assert_eq!(run.3.m, [a, b, c, d, e, f + 4.0]);
        }
        // Bold casts one too; Tech none.
        let (chunk, style, theme) = chunk_of(StyleId::Bold);
        let cmds = plain(&chunk, 1_340, &style, &theme);
        assert_eq!(
            runs(&cmds).iter().filter(|r| r.1 == theme.shadow).count(),
            3
        );
        let (chunk, style, theme) = chunk_of(StyleId::Tech);
        let cmds = plain(&chunk, 1_340, &style, &theme);
        assert!(!cmds.iter().any(|c| matches!(c, DrawCmd::PushLayer { .. })));
    }

    #[test]
    fn the_shadow_and_the_pill_change_no_layout_box() {
        let (chunk, style, theme) = chunk_of(StyleId::Clean);
        let mut bare = style.clone();
        bare.shadow = None;
        bare.motion.active = ActiveTreatment::Highlight;
        // Lines that fit are where they were: the pill's padding is room
        // only a line wider than the block less it would need.
        assert_eq!(chunk_in(&bare, &theme), chunk);
        assert_eq!(chunk.lines[0].scale, 1.0);
        // Drawn, every word's run is where its layout puts it.
        let words = words_of(&plain(&chunk, 1_340, &style, &theme), &theme);
        let boxes = word_boxes(&chunk);
        for (run, (_, r)) in words.iter().zip(&boxes) {
            assert!(near(run.3.m[4], r.x), "{run:?} {r:?}");
        }
    }

    #[test]
    fn bolds_spoken_word_pops_in_the_accent() {
        let (chunk, style, theme) = chunk_of(StyleId::Bold);
        let scale = chunk.lines[0].scale;
        // 70 ms into "MADE": at the pop's height; "WE" at rest, "IT" not started.
        let words = words_of(&plain(&chunk, 1_320, &style, &theme), &theme);
        let at: Vec<(&str, Rgba, f32)> = words
            .iter()
            .map(|w| (w.0.as_str(), w.1, w.3.m[0]))
            .collect();
        assert_eq!(
            at,
            [
                ("WE", theme.text, scale),
                ("MADE", theme.accent, scale * POP_SCALE),
                ("IT", theme.text, scale),
            ]
        );
        // No pill and no panel.
        assert!(rects(&plain(&chunk, 1_320, &style, &theme)).is_empty());
    }

    #[test]
    fn techs_panel_holds_the_lines_and_the_cursor_follows_the_spoken_word() {
        let (chunk, style, theme) = chunk_of(StyleId::Tech);
        let boxes: Vec<Rect> = word_boxes(&chunk).into_iter().map(|(_, r)| r).collect();
        let cell = 0.6 * 56.0 * chunk.lines[0].scale;
        let (left, right) = (
            boxes[0].x - cell - 32.0,
            boxes[2].x + boxes[2].w + cell + 32.0,
        );
        let panel = Rect {
            x: left,
            y: boxes[0].y - 32.0,
            w: right - left,
            h: boxes[0].h + 64.0,
        };
        for (t, after) in [(1_300, Some(1)), (1_550, None), (1_700, Some(2))] {
            let cmds = plain(&chunk, t, &style, &theme);
            // The panel first, at 85%.
            let DrawCmd::FillRect {
                rect,
                radius,
                color,
            } = cmds[0]
            else {
                panic!("{t}: the panel is drawn first");
            };
            assert!(same(rect, panel), "{t}: {rect:?}");
            assert_eq!((radius, color), (24.0, theme.panel));
            // No outline.
            assert!(runs(&cmds).iter().all(|run| run.2.is_none()));
            let found = rects(&cmds);
            match after {
                // The cursor after the spoken word, its height, in green; the
                // word green too.
                Some(i) => {
                    assert_eq!(found.len(), 2, "{t}");
                    let (cursor, word) = (found[1].0, boxes[i]);
                    let expected = Rect {
                        x: word.x + word.w,
                        y: word.y,
                        w: cell,
                        h: word.h,
                    };
                    assert!(same(cursor, expected), "{t}: {cursor:?}");
                    assert_eq!((found[1].1, found[1].2), (0.0, theme.accent));
                    assert!(cursor.x + cursor.w <= panel.x + panel.w - 32.0 + 1e-3);
                    let spoken = &runs(&cmds)[i];
                    assert_eq!(spoken.1, theme.accent);
                    assert!(matches!(cmds.last(), Some(DrawCmd::FillRect { .. })));
                }
                // In the pause, the panel alone.
                None => assert_eq!(found.len(), 1, "{t}"),
            }
        }
    }

    #[test]
    fn a_keyword_pops_to_1_25_settles_at_1_10_and_rests_when_it_ends() {
        assert_eq!(keyword_scale(0), 1.0);
        assert_eq!(keyword_scale(90), 1.25);
        assert_eq!(keyword_scale(180), 1.10);
        assert_eq!(keyword_scale(2_000), 1.10);
        let rising = keyword_scale(45);
        assert!(rising > 1.0 && rising < 1.25, "{rising}");

        // "made" is the keyword from its start.
        let (chunk, style, theme) = chunk_of(StyleId::Clean);
        let pops = Emphasis {
            pops: vec![(WordIdx::new(1), OutMs::new(1_250))],
            numbers: true,
        };
        let scale = chunk.lines[0].scale;
        let made = |t: u32| {
            let words = words_of(&drawn(&chunk, t, &style, &theme, &pops), &theme);
            // The last: over the copy under the pill while it slides in.
            let run = words.into_iter().rfind(|w| w.0 == "made").unwrap();
            (run.1, run.3.m[0])
        };
        assert_eq!(made(1_250), (theme.on_accent, scale));
        assert_eq!(made(1_340), (theme.on_accent, scale * 1.25));
        assert_eq!(made(1_430), (theme.on_accent, scale * 1.10));
        // After it ends: its size, still in the pop colour.
        assert_eq!(made(1_600), (theme.accent_alt, scale));
        // The pill grows with the word it sits behind.
        let made_box = word_boxes(&chunk)[1].1;
        let baseline = chunk.lines[0].baseline;
        let grown = scaled_about(made_box, (centre(made_box).0, baseline), 1.25);
        let pill = rects(&drawn(&chunk, 1_340, &style, &theme, &pops))[0].0;
        assert!(same(pill, padded(grown)), "{pill:?}");
    }

    #[test]
    fn a_word_that_fills_its_line_pops_only_as_far_as_the_safe_area_lets_it() {
        // TS §19.6 over D-170's 1.12: one word wider than Bold's line,
        // shrunk to the block, would leave the safe area at its pop's height.
        let (style, theme) = (spec(StyleId::Bold), Theme::default_for(StyleId::Bold));
        let chunk = chunk_with(&[("internationalization", 1_000, 1_600)], &style, &theme);
        let line = &chunk.lines[0];
        assert!(line.scale < 1.0, "{}", line.scale);
        let r = word_boxes(&chunk)[0].1;
        let about = (r.x + r.w / 2.0, line.baseline);
        let room = room_to_grow(r, about, &style, line.scale);
        assert!((1.0..POP_SCALE).contains(&room), "{room}");
        // At the pop's height it is drawn at that scale, outline inside.
        let words = words_of(&plain(&chunk, 1_070, &style, &theme), &theme);
        assert!(near(words[0].3.m[0], line.scale * room), "{:?}", words[0].3);
        // The outline, half the 10 lp stroke, grows with the word.
        let grown = scaled_about(r, about, room);
        let outline = 5.0 * line.scale * room;
        assert!(grown.x - outline >= SAFE_AREA.x - 1e-3, "{grown:?}");
        assert!(grown.x + grown.w + outline <= SAFE_AREA.x + SAFE_AREA.w + 1e-3);
        // A word with room pops all the way (bolds_spoken_word_pops_in_the_accent).
        let short = chunk_of(StyleId::Bold).0;
        let made = word_boxes(&short)[1].1;
        let about = (made.x + made.w / 2.0, short.lines[0].baseline);
        assert!(room_to_grow(made, about, &style, short.lines[0].scale) > POP_SCALE);
    }

    #[test]
    fn while_the_pill_slides_in_the_word_turns_to_ink_where_the_pill_reaches_it() {
        let (chunk, style, theme) = chunk_of(StyleId::Clean);
        // 30 ms into "made": its shadow, its look off the pill under the
        // pill, then its ink clipped to the pill.
        let cmds = plain(&chunk, 1_280, &style, &theme);
        let close = cmds.iter().position(|c| *c == DrawCmd::PopLayer).unwrap();
        let cast: Vec<String> = runs(&cmds[1..close]).into_iter().map(|r| r.0).collect();
        assert_eq!(cast, ["we", "made", "it"]);
        let DrawCmd::GlyphRun {
            text, fill, stroke, ..
        } = &cmds[close + 1]
        else {
            panic!("the word under the pill");
        };
        assert_eq!((text.as_str(), *fill), ("made", theme.text));
        assert_eq!(stroke.map(|s| s.width), Some(5.0));
        let DrawCmd::FillRect { rect: pill, .. } = cmds[close + 2] else {
            panic!("the pill");
        };
        let clip = cmds
            .iter()
            .position(|c| {
                *c == DrawCmd::PushLayer {
                    opacity: 1.0,
                    clip: Some(pill),
                }
            })
            .unwrap();
        let DrawCmd::GlyphRun {
            text, fill, stroke, ..
        } = &cmds[clip + 1]
        else {
            panic!("the ink");
        };
        assert_eq!(
            (text.as_str(), *fill, *stroke),
            ("made", theme.on_accent, None)
        );
        assert_eq!(cmds[clip + 2], DrawCmd::PopLayer);
        // At rest, from +90 ms: the ink alone, unclipped, and no shadow.
        let rest = plain(&chunk, 1_340, &style, &theme);
        let made: Vec<_> = runs(&rest).into_iter().filter(|r| r.0 == "made").collect();
        assert_eq!(made.len(), 1);
        assert_eq!((made[0].1, made[0].2), (theme.on_accent, None));
        let layers = rest
            .iter()
            .filter(|c| matches!(c, DrawCmd::PushLayer { .. }));
        assert_eq!(layers.count(), 1, "the shadow's only");
    }
}
