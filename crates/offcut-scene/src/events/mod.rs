//! The visuals of plan items (TS §19.5, §40.5): one file per kind of the
//! catalog (INV-23), each laid out once and drawn per frame.

pub mod emoji;
pub mod emphasis;
pub mod from_to;
pub mod icon_pop;
pub mod keyword_pop;
pub mod list_reveal;
pub mod number_reveal;
pub mod punch_in;

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
    /// V6.2 (D-478): the plan's zoom, drawn by the video track.
    PunchIn(punch_in::Track),
    /// V6.2 (D-481): a word marked in the captions, drawn by them.
    Emphasis(emphasis::Track),
    /// V6.2 (D-483): an emoji by its word, drawn with the captions.
    Emoji(emoji::Track),
    /// V6.2 (D-483): an icon on a disc in the event zone.
    IconPop(icon_pop::Track),
}

impl EventTrack {
    /// The catalog kind the track draws (INV-23).
    pub fn kind(&self) -> EventKind {
        match self {
            EventTrack::NumberReveal(_) => EventKind::NumberReveal,
            EventTrack::ListReveal(_) => EventKind::ListReveal,
            EventTrack::FromTo(_) => EventKind::FromTo,
            EventTrack::KeywordPop(_) => EventKind::KeywordPop,
            EventTrack::PunchIn(_) => EventKind::PunchIn,
            EventTrack::Emphasis(_) => EventKind::Emphasis,
            EventTrack::Emoji(_) => EventKind::Emoji,
            EventTrack::IconPop(_) => EventKind::IconPop,
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
        // V6.2 (D-478): a zoom over the item's program span.
        EventParams::PunchIn { scale, motion } => Ok(Some(EventTrack::PunchIn(punch_in::layout(
            *scale, *motion, span,
        )))),
        // V6.2 (D-481): a word the captions mark, from its program start.
        EventParams::Emphasis {
            word,
            treatment,
            size,
        } => Ok(Some(EventTrack::Emphasis(emphasis::layout(
            *word, *treatment, *size, span,
        )))),
        // V6.2 (D-483): an emoji by its word, placed by `build_scene` from
        // its chunk; an icon on a disc in the event zone.
        EventParams::Emoji { word, emoji, place } => Ok(Some(EventTrack::Emoji(emoji::layout(
            *word, *emoji, *place, span,
        )))),
        EventParams::IconPop { word, icon } => Ok(Some(EventTrack::IconPop(icon_pop::layout(
            *word, *icon, span, zone, theme,
        )))),
        // V6.2 (D-446): not drawn yet (P205 draws it); no track, no command, no count.
        EventParams::HookTitle { .. } => Ok(None),
    }
}

/// Adds what the item shows at `t`; nothing outside its time.
pub fn draw(track: &EventTrack, t: OutMs, out: &mut Vec<DrawCmd>) {
    match track {
        EventTrack::NumberReveal(track) => number_reveal::draw(track, t, out),
        EventTrack::ListReveal(track) => list_reveal::draw(track, t, out),
        EventTrack::FromTo(track) => from_to::draw(track, t, out),
        EventTrack::KeywordPop(track) => keyword_pop::draw(track, t, out),
        EventTrack::PunchIn(track) => punch_in::draw(track, t, out),
        EventTrack::Emphasis(track) => emphasis::draw(track, t, out),
        EventTrack::Emoji(track) => emoji::draw(track, t, out),
        EventTrack::IconPop(track) => icon_pop::draw(track, t, out),
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

/// An emoji's and an icon's entry (V6.2, D-483): from this share of its
/// size to all of it,
pub const IMAGE_POP_FROM: f32 = 0.6;
/// over this long, on `OUT`.
pub const IMAGE_POP_MS: u32 = 120;

/// The scale of an image `since_ms` after it enters: `IMAGE_POP_FROM` at 0,
/// 1.0 from `IMAGE_POP_MS` on.
pub fn image_pop(since_ms: u32) -> f32 {
    let share = (since_ms as f32 / IMAGE_POP_MS as f32).min(1.0);
    IMAGE_POP_FROM + (1.0 - IMAGE_POP_FROM) * crate::easing::bezier(crate::easing::OUT, share)
}

/// `r` scaled by `k` about its centre.
pub(crate) fn about_centre(r: Rect, k: f32) -> Rect {
    let (w, h) = (r.w * k, r.h * k);
    Rect {
        x: r.x + (r.w - w) / 2.0,
        y: r.y + (r.h - h) / 2.0,
        w,
        h,
    }
}
