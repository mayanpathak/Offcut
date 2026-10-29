//! Which part of the source frame is shown. This is the only place the crop
//! rectangle is worked out (TS §19.3).

use offcut_types::{ClipInfo, CropOffset};

use crate::CropRect;

/// The 9:16 part of the clip's frame, in source display pixels. A frame
/// wider than 9:16 loses its sides, moved by `offset` from the far left
/// (-1) through the centre (0) to the far right (1) (D-131); a narrower one
/// loses top and bottom, and its offset is ignored (TS §19.7).
pub fn crop_rect(clip: &ClipInfo, offset: CropOffset) -> CropRect {
    let offset = offset.get();
    let display_w = clip.display_width.get() as f32;
    let display_h = clip.display_height.get() as f32;
    // `display_w / display_h > 9 / 16`, in whole numbers.
    let wider = u64::from(clip.display_width.get()) * 16 > u64::from(clip.display_height.get()) * 9;
    if wider {
        let w = display_h * 9.0 / 16.0;
        CropRect {
            x: (display_w - w) / 2.0 * (1.0 + offset),
            y: 0.0,
            w,
            h: display_h,
        }
    } else {
        let h = display_w * 16.0 / 9.0;
        CropRect {
            x: 0.0,
            y: (display_h - h) / 2.0,
            w: display_w,
            h,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::clip;

    #[test]
    fn a_1920x1080_clip_is_cropped_to_the_strip_its_offset_names() {
        // D-160: V2's stand-in centred every offset; the offset now moves
        // the strip, `x = (1920 - 607.5) / 2 x (1 + offset)`.
        let strips = [
            (-1.0, 0.0),
            (-0.25, 492.1875),
            (0.0, 656.25),
            (0.5, 984.375),
            (1.0, 1_312.5),
        ];
        for (offset, x) in strips {
            let crop = crop_rect(&clip(1920, 1080, 60_000), CropOffset::new(offset).unwrap());
            let expected = CropRect {
                x,
                y: 0.0,
                w: 607.5,
                h: 1080.0,
            };
            assert_eq!(crop, expected, "offset {offset}");
        }
        // A square is wider than 9:16 too.
        let crop = crop_rect(&clip(1080, 1080, 60_000), CropOffset::new(0.0).unwrap());
        assert_eq!(
            (crop.x, crop.y, crop.w, crop.h),
            (236.25, 0.0, 607.5, 1080.0)
        );
    }

    #[test]
    fn a_1080x1920_clip_keeps_the_whole_frame() {
        let whole = CropRect {
            x: 0.0,
            y: 0.0,
            w: 1080.0,
            h: 1920.0,
        };
        assert_eq!(
            crop_rect(&clip(1080, 1920, 60_000), CropOffset::new(0.0).unwrap()),
            whole
        );
        assert_eq!(
            crop_rect(&clip(1080, 1920, 60_000), CropOffset::new(1.0).unwrap()),
            whole
        );

        // Narrower than 9:16: the top and the bottom go.
        let crop = crop_rect(&clip(900, 1920, 60_000), CropOffset::new(0.0).unwrap());
        assert_eq!(
            (crop.x, crop.y, crop.w, crop.h),
            (0.0, 160.0, 900.0, 1600.0)
        );
    }
}
