//! The visuals of plan items (TS §19.5, §40.5). V2 draws NumberReveal only.

pub mod number_reveal;

use offcut_plan::ValidItem;
use offcut_types::{EventParams, OutMs, OutSpan, TimeMap, Transcript};

use crate::SceneError;
use crate::display_list::{DrawCmd, Rect};
use crate::fonts::FontSet;
use crate::styles::StyleSpec;
use crate::theme::Theme;

/// An item laid out: everything `draw` needs, so that drawing a frame
/// shapes no text. V4 adds three variants.
#[derive(Clone, Debug, PartialEq)]
pub enum EventTrack {
    NumberReveal(number_reveal::Track),
}

/// `None` for an item that is not drawn: a kind V2 has no visual for, or a
/// number that cannot fit (TS §19.7). `span` is the item's program time and
/// `zone` the clip's event zone (D-171): when and where it is shown.
pub fn layout(
    item: &ValidItem,
    (span, zone): (OutSpan, Rect),
    t: &Transcript,
    map: &TimeMap,
    style: &StyleSpec,
    theme: &Theme,
    fonts: &FontSet,
) -> Result<Option<EventTrack>, SceneError> {
    // ListReveal and FromTo place their steps through these (V4 Prompt 106).
    let _ = (t, map);
    match &item.params {
        EventParams::NumberReveal { value, label } => {
            let label = label.as_deref();
            let track = number_reveal::layout(value, label, span, zone, style, theme, fonts)?;
            Ok(track.map(EventTrack::NumberReveal))
        }
        // V4: the visuals of TS §19.5 for these three.
        EventParams::ListReveal { .. } => Ok(None),
        EventParams::FromTo { .. } => Ok(None),
        EventParams::KeywordPop { .. } => Ok(None),
    }
}

/// Adds what the item shows at `t`; nothing outside its time.
pub fn draw(track: &EventTrack, t: OutMs, out: &mut Vec<DrawCmd>) {
    match track {
        EventTrack::NumberReveal(track) => number_reveal::draw(track, t, out),
    }
}
