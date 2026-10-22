//! The logical canvas and where on it things may be drawn (TS §19.1). This is
//! the only file that holds these numbers. All layout is in logical pixels
//! (lp); the insets are (assumption, E-2).

use offcut_types::CaptionPlacement;

use crate::display_list::Rect;
use crate::placement;

pub const CANVAS_WIDTH: f32 = 1080.0;
pub const CANVAS_HEIGHT: f32 = 1920.0;

pub const INSET_TOP: f32 = 250.0;
pub const INSET_BOTTOM: f32 = 420.0;
pub const INSET_LEFT: f32 = 60.0;
pub const INSET_RIGHT: f32 = 120.0;

/// No glyph run and no rectangle leaves this (TS §19.6).
pub const SAFE_AREA: Rect = Rect {
    x: INSET_LEFT,
    y: INSET_TOP,
    w: CANVAS_WIDTH - INSET_LEFT - INSET_RIGHT,
    h: CANVAS_HEIGHT - INSET_TOP - INSET_BOTTOM,
};

/// A part of the safe area a visual is placed in (D-138). Later releases
/// append rows (`Top`, `LowerThird`, …).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneId {
    /// 320 lp tall, on the free side of the captions (D-171): y 860 to 1180
    /// at the default placement.
    Event,
    /// The caption block, 280 lp tall: y 1220 to 1500 at the default
    /// placement.
    Caption,
    /// The safe area.
    Full,
}

/// The zone `id` for a clip whose captions are at `placement`, as wide as
/// the safe area (`placement.rs`, D-171).
pub fn zone(id: ZoneId, placement: CaptionPlacement) -> Rect {
    match id {
        ZoneId::Event => placement::event_zone(placement::block_at(placement)),
        ZoneId::Caption => placement::block_at(placement),
        ZoneId::Full => SAFE_AREA,
    }
}

/// The bottom-left corner of the safe area, as `(x, y)`.
pub const WATERMARK_ANCHOR: (f32, f32) = (SAFE_AREA.x, SAFE_AREA.y + SAFE_AREA.h);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_zones_are_v3s_and_inside_the_safe_area() {
        let rows = |id| {
            let r = zone(id, CaptionPlacement::bottom());
            (r.x, r.y, r.w, r.h)
        };
        assert_eq!(rows(ZoneId::Event), (60.0, 860.0, 900.0, 320.0));
        assert_eq!(rows(ZoneId::Caption), (60.0, 1220.0, 900.0, 280.0));
        assert_eq!(rows(ZoneId::Full), (60.0, 250.0, 900.0, 1250.0));
    }
}
