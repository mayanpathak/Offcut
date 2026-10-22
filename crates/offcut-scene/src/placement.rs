//! [ONLY] Where the captions sit and where the visuals go (D-171): the caption
//! block from the clip's `CaptionPlacement`, and the event zone that follows
//! it. Nothing here reads the watermark flag: the preview and the export
//! place captions alike (Appendix A). All values are (assumption, E-2).

use offcut_types::CaptionPlacement;

use crate::display_list::Rect;
use crate::safe_area::{CANVAS_HEIGHT, SAFE_AREA};
use crate::styles::StyleSpec;

/// The caption block's centre at placement 0.0,
pub const CAPTION_TOP: f32 = SAFE_AREA.y + 140.0;
/// and at 1.0: V3's caption zone, 1,220 to 1,500.
pub const CAPTION_BOTTOM: f32 = SAFE_AREA.y + SAFE_AREA.h - 140.0;
/// The three snap points of the drag (D-171).
pub const SNAP_TOP: f32 = 0.15;
pub const SNAP_MIDDLE: f32 = 0.55;
pub const SNAP_BOTTOM: f32 = 1.0;
/// The caption block's height, in logical pixels.
pub const CAPTION_BLOCK_HEIGHT: f32 = 280.0;
/// The event zone's height,
pub const EVENT_ZONE_HEIGHT: f32 = 320.0;
/// and the room between it and the caption block.
pub const EVENT_ZONE_GAP: f32 = 40.0;

/// The box the caption lines are centred in, as wide as the safe area, its
/// centre `CAPTION_TOP + p x (CAPTION_BOTTOM - CAPTION_TOP)`. Every V4 style
/// has the same block: `style` is the seam for one that will not.
pub fn caption_block(p: CaptionPlacement, _style: &StyleSpec) -> Rect {
    block_at(p)
}

/// `caption_block` for any style: what `safe_area::zone` reads.
pub(crate) fn block_at(p: CaptionPlacement) -> Rect {
    let centre = CAPTION_TOP + p.get() * (CAPTION_BOTTOM - CAPTION_TOP);
    Rect {
        x: SAFE_AREA.x,
        y: centre - CAPTION_BLOCK_HEIGHT / 2.0,
        w: SAFE_AREA.w,
        h: CAPTION_BLOCK_HEIGHT,
    }
}

/// Where the visuals of plan items go: the free side of the captions. When
/// the block's centre is at or below the canvas middle, the zone ends
/// `EVENT_ZONE_GAP` above the block; above the middle, it starts that far
/// below it.
pub fn event_zone(caption_block: Rect) -> Rect {
    let centre = caption_block.y + caption_block.h / 2.0;
    let y = if centre >= CANVAS_HEIGHT / 2.0 {
        caption_block.y - EVENT_ZONE_GAP - EVENT_ZONE_HEIGHT
    } else {
        caption_block.y + caption_block.h + EVENT_ZONE_GAP
    };
    Rect {
        x: caption_block.x,
        y,
        w: caption_block.w,
        h: EVENT_ZONE_HEIGHT,
    }
}

#[cfg(test)]
mod tests {
    use offcut_types::StyleId;

    use super::*;
    use crate::styles::spec;

    fn at(p: f32) -> (Rect, Rect) {
        let block = caption_block(CaptionPlacement::new(p).unwrap(), &spec(StyleId::Clean));
        (block, event_zone(block))
    }

    #[test]
    fn the_formula_at_the_two_ends_of_the_band() {
        assert_eq!((CAPTION_TOP, CAPTION_BOTTOM), (390.0, 1_360.0));
        // 1.0: V3's caption zone, and V3's event zone above it.
        let (block, events) = at(1.0);
        assert_eq!(
            (block.x, block.y, block.w, block.h),
            (60.0, 1_220.0, 900.0, 280.0)
        );
        assert_eq!(
            (events.x, events.y, events.w, events.h),
            (60.0, 860.0, 900.0, 320.0)
        );
        // 0.0: the block at the top of the safe area, the visuals below it.
        let (block, events) = at(0.0);
        assert_eq!((block.y, block.y + block.h), (250.0, 530.0));
        assert_eq!((events.y, events.y + events.h), (570.0, 890.0));
    }

    #[test]
    fn the_event_zone_changes_side_at_the_canvas_middle() {
        // A centre at exactly 960 counts as below the middle: the visuals
        // above, ending 40 lp over the block; 1 lp higher, they go below.
        let middle = Rect {
            x: 60.0,
            y: 820.0,
            w: 900.0,
            h: 280.0,
        };
        assert_eq!(event_zone(middle).y, 460.0);
        let higher = Rect { y: 819.0, ..middle };
        assert_eq!(event_zone(higher).y, 1_139.0);
        // Every style has the same block.
        let p = CaptionPlacement::new(SNAP_MIDDLE).unwrap();
        for style in [StyleId::Clean, StyleId::Bold, StyleId::Tech] {
            assert_eq!(caption_block(p, &spec(style)), block_at(p));
        }
    }
}
