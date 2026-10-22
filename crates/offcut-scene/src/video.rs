//! The video of a frame, as a draw command (D-130, D-131). The crop is per
//! frame: V6.1's punch-in, V6.2's jump-cut zoom and V6.5's shapes are keys of
//! this track, all in this file.

use offcut_types::OutMs;

use crate::display_list::{CropRect, DrawCmd, Rect, VideoSlot};

/// The video of a scene. V4's never moves: one crop for the whole clip.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Track {
    crop: CropRect,
}

impl Track {
    pub fn fixed(crop: CropRect) -> Track {
        Track { crop }
    }

    /// The `Video` command of the frame at `t`: the crop, on the whole
    /// canvas of `canvas` output pixels, opaque, with no mask.
    pub fn cmd(&self, t: OutMs, canvas: (f32, f32)) -> DrawCmd {
        // A fixed track is the same at every time.
        let _ = t;
        DrawCmd::Video {
            slot: VideoSlot::Main,
            src: self.crop,
            dst: Rect {
                x: 0.0,
                y: 0.0,
                w: canvas.0,
                h: canvas.1,
            },
            opacity: 1.0,
            mask: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fixed_track_draws_its_crop_on_the_whole_canvas_at_every_time() {
        let crop = CropRect {
            x: 656.25,
            y: 0.0,
            w: 607.5,
            h: 1080.0,
        };
        let track = Track::fixed(crop);
        let expected = DrawCmd::Video {
            slot: VideoSlot::Main,
            src: crop,
            dst: Rect {
                x: 0.0,
                y: 0.0,
                w: 540.0,
                h: 960.0,
            },
            opacity: 1.0,
            mask: None,
        };
        for t in [0, 1, 33, 59_999] {
            assert_eq!(track.cmd(OutMs::new(t), (540.0, 960.0)), expected);
        }
    }
}
