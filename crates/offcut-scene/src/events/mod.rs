//! The visuals of plan items (TS §19.5, §40.5): one file per kind of the
//! catalog (INV-23), each laid out once and drawn per frame.

pub mod from_to;
pub mod keyword_pop;
pub mod list_reveal;
pub mod number_reveal;

use offcut_plan::ValidItem;
use offcut_types::{EventKind, EventParams, OutMs, OutSpan, TimeMap, Transcript};

use crate::SceneError;
use crate::display_list::{DrawCmd, FontKey, Rect};
use crate::fonts::FontSet;
use crate::layout::{ShapedRun, shape};
use crate::styles::StyleSpec;
use crate::theme::Theme;

/// An item laid out: everything `draw` needs, so that drawing a frame
/// shapes no text.
#[derive(Clone, Debug, PartialEq)]
pub enum EventTrack {
    NumberReveal(number_reveal::Track),
    ListReveal(list_reveal::Track),
    FromTo(from_to::Track),
    KeywordPop(keyword_pop::Track),
}

impl EventTrack {
    /// The catalog kind the track draws (INV-23).
    pub fn kind(&self) -> EventKind {
        match self {
            EventTrack::NumberReveal(_) => EventKind::NumberReveal,
            EventTrack::ListReveal(_) => EventKind::ListReveal,
            EventTrack::FromTo(_) => EventKind::FromTo,
            EventTrack::KeywordPop(_) => EventKind::KeywordPop,
        }
    }
}

/// `None` only for an item that cannot be drawn: a number that cannot fit
/// (TS §19.7), a from-to whose values cannot fit or whose second value has
/// no program time, a list with nothing to show. `span` is the item's
/// program time and `zone` the clip's event zone (D-171): when and where it
/// is shown.
pub fn layout(
    item: &ValidItem,
    (span, zone): (OutSpan, Rect),
    t: &Transcript,
    map: &TimeMap,
    style: &StyleSpec,
    theme: &Theme,
    fonts: &FontSet,
) -> Result<Option<EventTrack>, SceneError> {
    // The visuals are placed by the item's resolved values and its times:
    // nothing reads the transcript's words (D-121).
    let _ = t;
    let look = (style, theme, fonts);
    match &item.params {
        EventParams::NumberReveal { value, label } => {
            let label = label.as_deref();
            let track = number_reveal::layout(value, label, span, zone, style, theme, fonts)?;
            Ok(track.map(EventTrack::NumberReveal))
        }
        EventParams::ListReveal { header, items, .. } => {
            let track = list_reveal::layout(header, items, (span, zone), map, look)?;
            Ok(track.map(EventTrack::ListReveal))
        }
        EventParams::FromTo { from, to, .. } => {
            // Its second value steps in at the item's mark (§8.7).
            let at = item.marks.first().and_then(|mark| map.to_out(*mark));
            let track = from_to::layout((from, to), (span, at), zone, look)?;
            Ok(track.map(EventTrack::FromTo))
        }
        EventParams::KeywordPop { word } => Ok(Some(EventTrack::KeywordPop(keyword_pop::layout(
            *word, span,
        )))),
    }
}

/// Adds what the item shows at `t`; nothing outside its time.
pub fn draw(track: &EventTrack, t: OutMs, out: &mut Vec<DrawCmd>) {
    match track {
        EventTrack::NumberReveal(track) => number_reveal::draw(track, t, out),
        EventTrack::ListReveal(track) => list_reveal::draw(track, t, out),
        EventTrack::FromTo(track) => from_to::draw(track, t, out),
        EventTrack::KeywordPop(track) => keyword_pop::draw(track, t, out),
    }
}

/// A text shaped once: its runs and what they measure together.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Text {
    pub runs: Vec<ShapedRun>,
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
}

impl Text {
    pub fn height(&self) -> f32 {
        self.ascent + self.descent
    }
}

pub(crate) fn text(
    string: &str,
    (font, fonts): (FontKey, &FontSet),
    size: f32,
) -> Result<Text, SceneError> {
    let runs = shape(string, font, size, fonts)?;
    Ok(Text {
        width: runs.iter().map(|run| run.width).sum(),
        ascent: runs.iter().map(|run| run.ascent).fold(0.0, f32::max),
        descent: runs.iter().map(|run| run.descent).fold(0.0, f32::max),
        runs,
    })
}

/// `ms` after `t`.
pub(crate) fn after(t: OutMs, ms: u32) -> OutMs {
    OutMs::new(t.get().saturating_add(ms))
}
