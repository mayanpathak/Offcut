//! FromTo: a change between two spoken values, as two columns with bars
//! (TS §19.5, D-141, §8.7). The second value's time is the item's mark,
//! through the time map: nothing here tokenizes or parses (D-121).

use offcut_types::{OutMs, OutSpan, Quantity};

use crate::SceneError;
use crate::anim::{self, eval};
use crate::display_list::{Affine, DrawCmd, PathEl, Rect, Rgba, Stroke};
use crate::easing::{Easing, ease};
use crate::events::{Text, after, text};
use crate::fonts::FontSet;
use crate::styles::StyleSpec;
use crate::theme::{Role, Theme};

/// The values' size when it fits, and the smallest drawn (TS §19.7's rule).
pub const VALUE_SIZE: f32 = 96.0;
pub const MIN_VALUE_SIZE: f32 = 56.0;
/// The "to" value is drawn this much larger.
pub const TO_SCALE: f32 = 1.15;
/// The tallest bar, the share the smallest is drawn at, the room above the
/// bars, and the bars' width and corner radius (logical pixels).
pub const BAR_HEIGHT: f32 = 180.0;
pub const MIN_BAR_SHARE: f32 = 0.08;
pub const VALUE_GAP: f32 = 16.0;
pub const BAR_WIDTH: f32 = 96.0;
pub const BAR_RADIUS: f32 = 12.0;
/// The arrow between the bars: its length, its head's length and height,
/// and its shaft's thickness.
pub const ARROW_LENGTH: f32 = 96.0;
pub const ARROW_HEAD: (f32, f32) = (40.0, 56.0);
pub const ARROW_SHAFT: f32 = 20.0;
/// Each column fades in and its bar grows over this, from its own time,
pub const ENTER_MS: u32 = 200;
const ENTER_EASING: Easing = Easing::EaseOutCubic;
/// and both stay this long after the item's span, then fade.
pub const HOLD_MS: u32 = 1_500;
pub const FADE_MS: u32 = 200;

/// One side: its value, where it is drawn, and its bar at full height.
#[derive(Clone, Debug, PartialEq)]
struct Column {
    value: Text,
    scale: f32,
    centre_x: f32,
    fill: Rgba,
    bar: Rect,
    at: OutMs,
}

/// A FromTo laid out: `draw` shapes nothing.
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    span: OutSpan,
    from: Column,
    to: Column,
    baseline: f32,
    arrow: Vec<PathEl>,
    arrow_fill: Rgba,
    stroke: Option<Stroke>,
    opacity: anim::Track,
}

/// A bar's share of `BAR_HEIGHT`: its value over the larger of the two, at
/// least `MIN_BAR_SHARE`; a value of zero or less, or two values none of
/// which is above zero, draw the least.
fn share(value: f64, max: f64) -> f32 {
    let ratio = value / max;
    if value > 0.0 && max > 0.0 && ratio.is_finite() {
        (ratio as f32).clamp(MIN_BAR_SHARE, 1.0)
    } else {
        MIN_BAR_SHARE
    }
}

/// A right-pointing arrow centred on `(x, y)`, as a filled path.
fn arrow(x: f32, y: f32) -> Vec<PathEl> {
    let (left, right) = (x - ARROW_LENGTH / 2.0, x + ARROW_LENGTH / 2.0);
    let neck = right - ARROW_HEAD.0;
    let (shaft, head) = (ARROW_SHAFT / 2.0, ARROW_HEAD.1 / 2.0);
    vec![
        PathEl::MoveTo(left, y - shaft),
        PathEl::LineTo(neck, y - shaft),
        PathEl::LineTo(neck, y - head),
        PathEl::LineTo(right, y),
        PathEl::LineTo(neck, y + head),
        PathEl::LineTo(neck, y + shaft),
        PathEl::LineTo(left, y + shaft),
        PathEl::Close,
    ]
}

/// The two values in the halves of the event zone, from the item's start
/// ("from") and from `at`, its mark in program time ("to", in `accent`, at
/// `TO_SCALE`, with the arrow). One size for both values: `VALUE_SIZE` when
/// it fits the halves and the zone's height, else the largest that does.
/// `None` when that is below `MIN_VALUE_SIZE`, when a value is empty, or
/// when the mark has no program time.
pub fn layout(
    (from, to): (&Quantity, &Quantity),
    (span, at): (OutSpan, Option<OutMs>),
    zone: Rect,
    (style, theme, fonts): (&StyleSpec, &Theme, &FontSet),
) -> Result<Option<Track>, SceneError> {
    let Some(at) = at else {
        return Ok(None);
    };
    let at = at.max(span.start).min(span.end);
    let font = (theme.font(style.font), fonts);
    let (a, b) = (
        text(&from.display, font, VALUE_SIZE)?,
        text(&to.display, font, VALUE_SIZE)?,
    );
    if a.width <= 0.0 || b.width <= 0.0 {
        return Ok(None);
    }
    // Each value and its outline fit its half; the "to" value's outline
    // grows with it.
    let half = zone.w / 2.0;
    let stroke = style.stroke_width.max(0.0);
    let line = a.height().max(b.height());
    let fits = [
        1.0,
        (half - stroke) / a.width,
        (half / TO_SCALE - stroke) / b.width,
        (zone.h - VALUE_GAP - BAR_HEIGHT) / (line * TO_SCALE),
    ];
    let size = VALUE_SIZE * fits.into_iter().fold(f32::INFINITY, f32::min);
    if size < MIN_VALUE_SIZE {
        return Ok(None);
    }
    let (a, b) = (
        text(&from.display, font, size)?,
        text(&to.display, font, size)?,
    );

    // From the top: the values on one baseline, the gap, the bars standing
    // on one line; centred down the zone.
    let ascent = a.ascent.max(b.ascent);
    let stack = TO_SCALE * a.height().max(b.height()) + VALUE_GAP + BAR_HEIGHT;
    let top = zone.y + (zone.h - stack) / 2.0;
    let floor = top + stack;
    let max = from.value.max(to.value);
    let side = |value: Text, quantity: &Quantity, (centre_x, at): (f32, OutMs), role| {
        let height = BAR_HEIGHT * share(quantity.value, max);
        Column {
            value,
            scale: 1.0,
            centre_x,
            fill: theme.color(role),
            bar: Rect {
                x: centre_x - BAR_WIDTH / 2.0,
                y: floor - height,
                w: BAR_WIDTH,
                h: height,
            },
            at,
        }
    };
    let from = side(a, from, (zone.x + half / 2.0, span.start), style.fill);
    let to = Column {
        scale: TO_SCALE,
        ..side(b, to, (zone.x + half * 1.5, at), Role::Accent)
    };
    let hold_end = after(span.end, HOLD_MS);
    Ok(Some(Track {
        span,
        from,
        to,
        baseline: top + TO_SCALE * ascent,
        arrow: arrow(zone.x + half, floor - BAR_HEIGHT / 2.0),
        arrow_fill: theme.color(style.fill),
        stroke: (style.stroke_width > 0.0).then_some(Stroke {
            width: style.stroke_width,
            color: theme.color(style.stroke),
        }),
        opacity: anim::Track {
            keys: vec![(hold_end, 1.0), (after(hold_end, FADE_MS), 0.0)],
            easing: Easing::Linear,
        },
    }))
}

/// Shown from the item's start until the fade has ended.
pub fn visible_at(track: &Track, t: OutMs) -> bool {
    track.span.start <= t && t < after(after(track.span.end, HOLD_MS), FADE_MS)
}

/// One column at `t`: nothing before its time, then fading in with its bar
/// growing; the arrow with the "to" column.
fn column(track: &Track, column: &Column, arrow: bool, t: OutMs, out: &mut Vec<DrawCmd>) {
    if t < column.at {
        return;
    }
    let since = t.get().saturating_sub(column.at.get());
    let e = ease(ENTER_EASING, since as f32 / ENTER_MS as f32);
    let entering = e < 1.0;
    if entering {
        out.push(DrawCmd::PushLayer {
            opacity: e,
            clip: None,
        });
    }
    if arrow {
        out.push(DrawCmd::FillPath {
            path: track.arrow.clone(),
            color: track.arrow_fill,
        });
    }
    let bar = Rect {
        y: column.bar.y + column.bar.h * (1.0 - e),
        h: column.bar.h * e,
        ..column.bar
    };
    if bar.h > 0.0 {
        out.push(DrawCmd::FillRect {
            rect: bar,
            radius: BAR_RADIUS.min(bar.h / 2.0),
            color: column.fill,
        });
    }
    let value = &column.value;
    let mut x = column.centre_x - value.width * column.scale / 2.0;
    for run in &value.runs {
        out.push(DrawCmd::GlyphRun {
            font: run.font,
            size: run.size,
            glyphs: run.glyphs.clone(),
            fill: column.fill,
            stroke: track.stroke,
            transform: Affine::scale_then_translate(column.scale, x, track.baseline),
            text: run.text.clone(),
        });
        x += run.width * column.scale;
    }
    if entering {
        out.push(DrawCmd::PopLayer);
    }
}

/// Adds the from-to as it looks at `t`; nothing outside its time.
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
    column(track, &track.from, false, t, out);
    column(track, &track.to, true, t, out);
    if fading {
        out.push(DrawCmd::PopLayer);
    }
}

#[cfg(test)]
mod tests {
    use offcut_types::{CaptionPlacement, StyleId, Unit};

    use super::*;
    use crate::safe_area::{ZoneId, zone};
    use crate::styles::spec;

    fn usd(value: f64, display: &str) -> Quantity {
        Quantity {
            value,
            unit: Unit::Usd,
            display: display.to_owned(),
        }
    }

    fn lay(from: &Quantity, to: &Quantity, at: Option<u32>) -> Option<Track> {
        let (style, theme) = (spec(StyleId::Clean), Theme::default_for(StyleId::Clean));
        let event = zone(ZoneId::Event, CaptionPlacement::bottom());
        let span = OutSpan::new(OutMs::new(1_000), OutMs::new(3_000)).unwrap();
        let look = (&style, &theme, &FontSet::embedded());
        layout((from, to), (span, at.map(OutMs::new)), event, look).unwrap()
    }

    fn drawn(track: &Track, t: u32) -> Vec<DrawCmd> {
        let mut out = Vec::new();
        draw(track, OutMs::new(t), &mut out);
        out
    }

    #[test]
    fn two_short_values_share_the_size_that_fits_the_zone_and_the_bars_their_ratio() {
        let track = lay(&usd(2_800.0, "$2,800"), &usd(900.0, "$900"), Some(2_000)).unwrap();
        let (a, b) = (&track.from, &track.to);
        // 96 lp would not leave room for the bars in the 320 lp zone at
        // 1.15: the size is the zone's, the same for both, at least 56.
        let size = a.value.runs[0].size;
        assert_eq!(size, b.value.runs[0].size);
        assert!((MIN_VALUE_SIZE..VALUE_SIZE).contains(&size), "{size}");
        let stack = TO_SCALE * a.value.height().max(b.value.height()) + VALUE_GAP + BAR_HEIGHT;
        assert!((stack - 320.0).abs() < 0.01, "{stack}");
        // "to" larger and in the accent; the bars on one line, in proportion.
        let theme = Theme::default_for(StyleId::Clean);
        assert_eq!((a.scale, b.scale), (1.0, TO_SCALE));
        assert_eq!((a.fill, b.fill), (theme.text, theme.accent));
        assert_eq!(a.bar.h, 180.0);
        assert!((b.bar.h - 180.0 * 900.0 / 2_800.0).abs() < 1e-3);
        assert_eq!(a.bar.y + a.bar.h, b.bar.y + b.bar.h);
        // The columns are the zone's halves.
        assert_eq!((a.centre_x, b.centre_x), (60.0 + 225.0, 60.0 + 675.0));
    }

    #[test]
    fn a_zero_or_negative_value_draws_the_smallest_bar() {
        for (from, to) in [(0.0, 10.0), (-5.0, 10.0), (0.0, 0.0), (1.0, 1_000.0)] {
            let track = lay(&usd(from, "$1"), &usd(to, "$2"), Some(2_000)).unwrap();
            assert!(
                (track.from.bar.h - 180.0 * MIN_BAR_SHARE).abs() < 1e-3,
                "{from} {to}"
            );
        }
        assert_eq!(share(5.0, -1.0), MIN_BAR_SHARE);
        assert_eq!(share(f64::NAN, 1.0), MIN_BAR_SHARE);
        assert_eq!(share(1.0, 1.0), 1.0);
    }

    #[test]
    fn values_too_wide_for_a_half_shrink_and_below_56_lp_are_not_drawn() {
        let wide = usd(1.0, "$1,000,000");
        let track = lay(&wide, &usd(2.0, "$2"), Some(2_000)).unwrap();
        let a = &track.from;
        assert!(a.value.width <= 450.0 - 5.0 + 1e-3, "{}", a.value.width);
        let too_wide = usd(1.0, "$1,000,000,000,000,000,000");
        assert!(lay(&too_wide, &usd(2.0, "$2"), Some(2_000)).is_none());
        // No mark in program time: not drawn.
        assert!(lay(&usd(1.0, "$1"), &usd(2.0, "$2"), None).is_none());
    }

    #[test]
    fn from_at_the_start_to_and_the_arrow_at_the_mark_then_hold_and_fade() {
        let track = lay(&usd(2_800.0, "$2,800"), &usd(900.0, "$900"), Some(2_000)).unwrap();
        let texts = |t: u32| -> Vec<String> {
            drawn(&track, t)
                .into_iter()
                .filter_map(|cmd| {
                    if let DrawCmd::GlyphRun { text, .. } = cmd {
                        Some(text)
                    } else {
                        None
                    }
                })
                .collect()
        };
        let paths = |t: u32| {
            drawn(&track, t)
                .iter()
                .filter(|cmd| matches!(cmd, DrawCmd::FillPath { .. }))
                .count()
        };
        assert!(drawn(&track, 999).is_empty());
        assert_eq!(texts(1_000), ["$2,800"]);
        assert_eq!(texts(1_999), ["$2,800"]);
        assert_eq!(paths(1_999), 0);
        assert_eq!(texts(2_000), ["$2,800", "$900"]);
        assert_eq!(paths(2_000), 1);
        // The bar grows in with its column: none at its first instant, all
        // of it from +200 ms.
        let bars = |t: u32| -> Vec<f32> {
            drawn(&track, t)
                .into_iter()
                .filter_map(|cmd| {
                    if let DrawCmd::FillRect { rect, .. } = cmd {
                        Some(rect.h)
                    } else {
                        None
                    }
                })
                .collect()
        };
        assert_eq!(bars(1_000), Vec::<f32>::new());
        assert_eq!(bars(1_200), [180.0]);
        assert_eq!(bars(2_200).len(), 2);
        // Held to the span's end + 1,500 ms, faded over 200 ms.
        assert_eq!(texts(4_499).len(), 2);
        assert!(drawn(&track, 4_700).is_empty());
        // A mark before the start is the start; after the end, the end.
        let early = lay(&usd(1.0, "$1"), &usd(2.0, "$2"), Some(500)).unwrap();
        assert_eq!(early.to.at, OutMs::new(1_000));
    }
}
