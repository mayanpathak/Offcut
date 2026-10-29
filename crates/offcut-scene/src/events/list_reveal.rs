//! ListReveal: a spoken list, its items stacked as they are said (TS §19.5,
//! D-140). Every text is shaped here, at layout; `draw` only places it.

use offcut_types::{ListItem, OutMs, OutSpan, TimeMap};

use crate::SceneError;
use crate::anim::{self, eval};
use crate::display_list::{Affine, DrawCmd, Rect, Rgba, Stroke};
use crate::easing::{Easing, ease};
use crate::events::{Text, after, text};
use crate::fonts::FontSet;
use crate::styles::StyleSpec;
use crate::theme::{FontRole, Theme};

/// The header's size and the items', in logical pixels, when they fit.
pub const HEADER_SIZE: f32 = 56.0;
pub const ITEM_SIZE: f32 = 48.0;
/// The smallest an item is drawn: below it, the items that do not fit are
/// not shown.
pub const MIN_ITEM_SIZE: f32 = 32.0;
/// The room between two lines, and the lines' indent from the zone's left.
pub const LINE_GAP: f32 = 16.0;
pub const INDENT: f32 = 40.0;
/// Each line fades in and rises this far into place, from its own time,
pub const ENTER_MS: u32 = 200;
pub const RISE: f32 = 24.0;
const ENTER_EASING: Easing = Easing::EaseOutCubic;
/// and the list stays this long after its last item, then fades.
pub const HOLD_MS: u32 = 1_500;
pub const FADE_MS: u32 = 200;

/// A line of the list: its text, when it comes in, where it rests, and the
/// scale it is drawn at.
#[derive(Clone, Debug, PartialEq)]
struct Line {
    text: Text,
    at: OutMs,
    baseline: f32,
    scale: f32,
}

/// A ListReveal laid out: the header, when there is one, then the items.
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    span: OutSpan,
    lines: Vec<Line>,
    left: f32,
    opacity: anim::Track,
    fill: Rgba,
    stroke: Option<Stroke>,
}

/// The header (at the item's start, unless empty) and each item said where
/// the program plays (at its own time, `"{ordinal}. {text}"`), in the event
/// zone. The texts share one scale: 1 when every line fits the zone, else
/// the largest that fits, but never smaller than `MIN_ITEM_SIZE` for the
/// items; at that size the last items that do not fit are left out. `None`
/// when no item is shown.
pub fn layout(
    header: &str,
    items: &[ListItem],
    (span, zone): (OutSpan, Rect),
    map: &TimeMap,
    (style, theme, fonts): (&StyleSpec, &Theme, &FontSet),
) -> Result<Option<Track>, SceneError> {
    let mut lines: Vec<(Text, OutMs)> = Vec::new();
    let header = header.trim();
    if !header.is_empty() {
        let display = (theme.font(FontRole::Display), fonts);
        lines.push((text(header, display, HEADER_SIZE)?, span.start));
    }
    let first_item = lines.len();
    let font = (theme.font(style.font), fonts);
    for item in items {
        // An item said where the program does not play is not shown.
        let Some(at) = map.to_out(item.at) else {
            continue;
        };
        let line = format!("{}. {}", item.ordinal, item.text.trim());
        lines.push((text(&line, font, ITEM_SIZE)?, at.max(span.start)));
    }
    if lines.len() == first_item {
        return Ok(None);
    }

    let room = zone.w - INDENT - style.stroke_width.max(0.0);
    let stack = |lines: &[(Text, OutMs)], k: f32| {
        let texts: f32 = lines.iter().map(|(text, _)| text.height() * k).sum();
        texts + LINE_GAP * lines.len().saturating_sub(1) as f32
    };
    let widest = lines.iter().map(|(text, _)| text.width).fold(0.0, f32::max);
    let high = stack(&lines, 1.0) - stack(&lines, 0.0);
    let mut k = 1.0f32.min((zone.h - stack(&lines, 0.0)) / high);
    if widest > 0.0 {
        k = k.min(room / widest);
    }
    let least = MIN_ITEM_SIZE / ITEM_SIZE;
    if k < least {
        k = least;
        while lines.len() > first_item + 1 && stack(&lines, k) > zone.h {
            lines.pop();
        }
    }

    // Centred down the zone; a line still too wide at `k` is drawn smaller.
    let mut top = zone.y + (zone.h - stack(&lines, k)) / 2.0;
    let mut placed = Vec::with_capacity(lines.len());
    for (text, at) in lines {
        let scale = if text.width * k > room {
            room / text.width
        } else {
            k
        };
        let baseline = top + text.ascent * k;
        top += text.height() * k + LINE_GAP;
        placed.push(Line {
            text,
            at,
            baseline,
            scale,
        });
    }
    let hold_end = after(span.end, HOLD_MS);
    Ok(Some(Track {
        span,
        lines: placed,
        left: zone.x + INDENT,
        opacity: anim::Track {
            keys: vec![(hold_end, 1.0), (after(hold_end, FADE_MS), 0.0)],
            easing: Easing::Linear,
        },
        fill: theme.color(style.fill),
        stroke: (style.stroke_width > 0.0).then_some(Stroke {
            width: style.stroke_width,
            color: theme.color(style.stroke),
        }),
    }))
}

/// Shown from the item's start until the fade has ended.
pub fn visible_at(track: &Track, t: OutMs) -> bool {
    track.span.start <= t && t < after(after(track.span.end, HOLD_MS), FADE_MS)
}

/// Adds the list as it looks at `t`: each line from its own time, fading in
/// and rising into place; nothing outside the list's time.
pub fn draw(track: &Track, t: OutMs, out: &mut Vec<DrawCmd>) {
    if !visible_at(track, t) {
        return;
    }
    let opacity = eval(&track.opacity, t);
    let fading = opacity < 1.0;
    if fading {
        out.push(DrawCmd::PushLayer {
            opacity,
            clip: None,
        });
    }
    for line in &track.lines {
        if t < line.at {
            continue;
        }
        let since = t.get().saturating_sub(line.at.get());
        let e = ease(ENTER_EASING, since as f32 / ENTER_MS as f32);
        let entering = e < 1.0;
        if entering {
            out.push(DrawCmd::PushLayer {
                opacity: e,
                clip: None,
            });
        }
        let baseline = line.baseline + RISE * (1.0 - e);
        let mut x = track.left;
        for run in &line.text.runs {
            out.push(DrawCmd::GlyphRun {
                font: run.font,
                size: run.size,
                glyphs: run.glyphs.clone(),
                fill: track.fill,
                stroke: track.stroke,
                transform: Affine::scale_then_translate(line.scale, x, baseline),
                text: run.text.clone(),
            });
            x += run.width * line.scale;
        }
        if entering {
            out.push(DrawCmd::PopLayer);
        }
    }
    if fading {
        out.push(DrawCmd::PopLayer);
    }
}

#[cfg(test)]
mod tests {
    use offcut_types::{CaptionPlacement, DurMs, StyleId, TimeMs};

    use super::*;
    use crate::safe_area::{ZoneId, zone};
    use crate::styles::spec;

    fn items(texts: &[&str], from_ms: u32) -> Vec<ListItem> {
        (1u8..)
            .zip(texts)
            .map(|(ordinal, text)| ListItem {
                ordinal,
                text: (*text).to_owned(),
                at: TimeMs::new(from_ms + u32::from(ordinal) * 1_000),
            })
            .collect()
    }

    fn span(start: u32, end: u32) -> OutSpan {
        OutSpan::new(OutMs::new(start), OutMs::new(end)).unwrap()
    }

    fn lay(header: &str, list: &[ListItem], map: &TimeMap) -> Option<Track> {
        let (style, theme) = (spec(StyleId::Clean), Theme::default_for(StyleId::Clean));
        let event = zone(ZoneId::Event, CaptionPlacement::bottom());
        let look = (&style, &theme, &FontSet::embedded());
        layout(header, list, (span(1_000, 9_000), event), map, look).unwrap()
    }

    fn identity() -> TimeMap {
        TimeMap::identity(DurMs::new(20_000))
    }

    /// The lines' sizes (text size times scale) and the stack's height.
    fn measure(track: &Track) -> (Vec<f32>, f32) {
        let sizes = track
            .lines
            .iter()
            .map(|l| l.text.runs[0].size * l.scale)
            .collect();
        let first = &track.lines[0];
        let last = &track.lines[track.lines.len() - 1];
        let k = last.scale;
        let height =
            last.baseline + last.text.descent * k - (first.baseline - first.text.ascent * k);
        (sizes, height)
    }

    #[test]
    fn three_items_and_a_header_fit_at_their_sizes() {
        let list = items(&["speed", "price", "support"], 1_000);
        let track = lay("3 REASONS", &list, &identity()).unwrap();
        let (sizes, height) = measure(&track);
        assert_eq!(sizes, [56.0, 48.0, 48.0, 48.0]);
        assert!(height <= 320.0, "{height}");
        // Left at the zone's left plus 40, numbered.
        assert_eq!(track.left, 60.0 + 40.0);
        let texts: Vec<&str> = track
            .lines
            .iter()
            .map(|l| l.text.runs[0].text.as_str())
            .collect();
        assert_eq!(texts, ["3 REASONS", "1. speed", "2. price", "3. support"]);
        // No header: the items alone, from the first.
        let bare = lay("  ", &list, &identity()).unwrap();
        assert_eq!(bare.lines.len(), 3);
    }

    #[test]
    fn more_lines_than_fit_shrink_together_and_at_the_minimum_the_last_are_left_out() {
        // A header and four items do not fit 320 lp at 48: one factor for all.
        let four = items(&["one", "two", "three", "four"], 1_000);
        let track = lay("4 TIPS", &four, &identity()).unwrap();
        let (sizes, height) = measure(&track);
        let k = sizes[1] / 48.0;
        assert!((32.0 / 48.0..1.0).contains(&k), "{k}");
        assert!((sizes[0] - 56.0 * k).abs() < 1e-3, "{sizes:?}");
        assert!((height - 320.0).abs() < 0.01, "{height}");
        // Twelve items: at 32 lp, the ones that do not fit are not shown.
        let names = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l"];
        let track = lay("12 THINGS", &items(&names, 0), &identity()).unwrap();
        let (sizes, height) = measure(&track);
        assert!((sizes[1] - 32.0).abs() < 1e-3, "{sizes:?}");
        assert!(track.lines.len() < 13 && height <= 320.0, "{height}");
        let next = height + 16.0 + track.lines[1].text.height() * (32.0 / 48.0);
        assert!(next > 320.0, "one more would have fitted: {height}");
    }

    #[test]
    fn a_line_wider_than_the_zone_shrinks_the_list() {
        let list = items(&["WWWWWWWWWWWWWWWWWW", "b"], 1_000);
        let track = lay("", &list, &identity()).unwrap();
        let widest = &track.lines[0];
        let right = track.left + widest.text.width * widest.scale;
        assert!((right - (960.0 - 5.0)).abs() < 0.01, "{right}");
        assert!(widest.scale > 32.0 / 48.0 && widest.scale < 1.0);
        assert_eq!(track.lines[0].scale, track.lines[1].scale);
        // Too wide even at 32 lp: the others stay at 32, it alone is smaller.
        let list = items(&["WWWWWWWWWWWWWWWWWWWWWWWWWWWW", "b"], 1_000);
        let track = lay("", &list, &identity()).unwrap();
        let (widest, other) = (&track.lines[0], &track.lines[1]);
        let right = track.left + widest.text.width * widest.scale;
        assert!((right - (960.0 - 5.0)).abs() < 0.01, "{right}");
        assert_eq!(other.scale, 32.0 / 48.0);
        assert!(widest.scale < other.scale);
    }

    #[test]
    fn each_item_comes_in_at_its_own_time_rising_and_the_list_holds_then_fades() {
        let list = items(&["speed", "price"], 1_000);
        let track = lay("", &list, &identity()).unwrap();
        let at = |t: u32| {
            let mut out = Vec::new();
            draw(&track, OutMs::new(t), &mut out);
            out
        };
        let shown = |t: u32| -> Vec<(String, f32)> {
            at(t)
                .into_iter()
                .filter_map(|cmd| {
                    if let DrawCmd::GlyphRun {
                        text, transform, ..
                    } = cmd
                    {
                        Some((text, transform.m[5]))
                    } else {
                        None
                    }
                })
                .collect()
        };
        // Item 1 at 2,000, item 2 at 3,000 (ms).
        assert!(at(999).is_empty());
        assert!(shown(1_999).is_empty());
        let (rest, second) = (track.lines[0].baseline, track.lines[1].baseline);
        let entering = shown(2_000);
        assert_eq!(entering, [("1. speed".to_owned(), rest + 24.0)]);
        assert_eq!(
            at(2_000)[0],
            DrawCmd::PushLayer {
                opacity: 0.0,
                clip: None
            }
        );
        // Half-way, ease-out cubic: 87.5% of the way up.
        assert!((shown(2_100)[0].1 - (rest + 3.0)).abs() < 1e-3);
        assert_eq!(shown(2_200), [("1. speed".to_owned(), rest)]);
        assert_eq!(shown(3_250).len(), 2);
        assert_eq!(shown(3_250)[1].1, second);
        // Held to the span's end + 1,500 ms, faded over 200 ms.
        assert_eq!(shown(10_499).len(), 2);
        assert!(
            matches!(at(10_600)[0], DrawCmd::PushLayer { opacity, .. } if (opacity - 0.5).abs() < 1e-3)
        );
        assert!(at(10_700).is_empty());
    }

    #[test]
    fn an_item_said_where_the_program_does_not_play_is_not_shown() {
        // The program plays 0-2,500 and 4,000-20,000 of the recording: the
        // second item (said at 3,000) is in the cut.
        let cut: TimeMap = serde_json::from_str(
            r#"{"segments":[{"kind":"source","src":{"start":0,"end":2500}},{"kind":"source","src":{"start":4000,"end":20000}}]}"#,
        )
        .unwrap();
        let list = items(&["speed", "price", "support"], 1_000);
        let track = lay("", &list, &cut).unwrap();
        let texts: Vec<&str> = track
            .lines
            .iter()
            .map(|l| l.text.runs[0].text.as_str())
            .collect();
        assert_eq!(texts, ["1. speed", "3. support"]);
        // The third at its program time: 4,000 recording ms is 2,500 program.
        assert_eq!(track.lines[1].at, OutMs::new(2_500));
        // A list none of whose items plays is not drawn.
        let none = items(&["price"], 2_000);
        assert!(lay("HEADER", &none, &cut).is_none());
    }
}
