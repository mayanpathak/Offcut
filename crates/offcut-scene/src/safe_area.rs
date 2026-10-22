//! The logical canvas and where on it things may be drawn (TS §19.1). This is
//! the only file that holds these numbers. All layout is in logical pixels
//! (lp); the insets are (assumption, E-2).

use crate::display_list::Rect;

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
    /// y 860 to 1180, as wide as the safe area.
    Event,
    /// y 1220 to 1500, as wide as the safe area.
    Caption,
    /// The safe area.
    Full,
}

pub const fn zone(id: ZoneId) -> Rect {
    match id {
        ZoneId::Event => Rect {
            x: SAFE_AREA.x,
            y: 860.0,
            w: SAFE_AREA.w,
            h: 320.0,
        },
        ZoneId::Caption => Rect {
            x: SAFE_AREA.x,
            y: 1220.0,
            w: SAFE_AREA.w,
            h: 280.0,
        },
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
            let r = zone(id);
            (r.x, r.y, r.w, r.h)
        };
        assert_eq!(rows(ZoneId::Event), (60.0, 860.0, 900.0, 320.0));
        assert_eq!(rows(ZoneId::Caption), (60.0, 1220.0, 900.0, 280.0));
        assert_eq!(rows(ZoneId::Full), (60.0, 250.0, 900.0, 1250.0));
    }
}
