//! The video of a frame, as a draw command (D-130, D-131). The crop is per
//! frame: V6.2's punch-in, V6.3's jump-cut zoom and V6.6's shapes are keys of
//! this track, all in this file.

use offcut_types::OutMs;

use crate::camera::CameraTrack;
use crate::display_list::{CropRect, DrawCmd, Rect, VideoSlot};

/// The most the frame is ever zoomed: the camera's drift times the plan's
/// punch-in (P200), and later the cut's jump zoom (v6.2implementation D-477,
/// TS §47.2; assumption).
pub const MAX_TOTAL_ZOOM: f32 = 1.35;

/// The video of a scene: the crop, zoomed about its centre by the scale of
/// the camera at each time (V6.2, D-477). A scale of 1.0 is the crop itself.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Track {
    crop: CropRect,
    camera: CameraTrack,
    /// The source frame's display size: a zoomed crop never leaves it.
    frame: (f32, f32),
}

impl Track {
    /// One crop for the whole clip: V4's track, and V6.2's with motion off.
    pub fn fixed(crop: CropRect) -> Track {
        Track {
            crop,
            camera: CameraTrack::none(),
            frame: (crop.x + crop.w, crop.y + crop.h),
        }
    }

    /// The crop zoomed by `camera`, inside a source frame of `frame` display pixels.
    pub fn with_camera(crop: CropRect, camera: CameraTrack, frame: (f32, f32)) -> Track {
        Track {
            crop,
            camera,
            frame,
        }
    }

    /// The part of the source shown at `t`: the crop itself at a scale of
    /// exactly 1.0 (bit for bit), else the crop shrunk by the scale about
    /// its centre and moved back inside the source frame.
    pub fn src_at(&self, t: OutMs) -> CropRect {
        let scale = self.camera.scale_at(t).clamp(1.0, MAX_TOTAL_ZOOM);
        if scale == 1.0 {
            return self.crop;
        }
        zoomed(self.crop, scale, self.frame)
    }

    /// The `Video` command of the frame at `t`: `src_at(t)`, on the whole
    /// canvas of `canvas` output pixels, opaque, with no mask.
    pub fn cmd(&self, t: OutMs, canvas: (f32, f32)) -> DrawCmd {
        DrawCmd::Video {
            slot: VideoSlot::Main,
            src: self.src_at(t),
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

/// `crop` shrunk by `scale` (above 1.0) about its centre, then moved, if it
/// must, to lie inside a frame of `frame` pixels.
fn zoomed(crop: CropRect, scale: f32, frame: (f32, f32)) -> CropRect {
    let (w, h) = (crop.w / scale, crop.h / scale);
    let inside = |at: f32, size: f32, room: f32| at.min(room - size).max(0.0);
    CropRect {
        x: inside(crop.x + (crop.w - w) / 2.0, w, frame.0),
        y: inside(crop.y + (crop.h - h) / 2.0, h, frame.1),
        w,
        h,
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

    const CROP: CropRect = CropRect {
        x: 656.25,
        y: 0.0,
        w: 607.5,
        h: 1080.0,
    };
    const FRAME: (f32, f32) = (1920.0, 1080.0);

    /// Every frame time of a program `ms` long at 30 fps.
    fn frames(ms: u32) -> impl Iterator<Item = OutMs> {
        (0..ms * 30 / 1000).map(|n| OutMs::new(n * 1000 / 30))
    }

    fn bits(r: CropRect) -> [u32; 4] {
        [r.x.to_bits(), r.y.to_bits(), r.w.to_bits(), r.h.to_bits()]
    }

    #[test]
    fn no_drift_is_todays_track_at_every_frame_of_a_60_s_clip() {
        let today = Track::fixed(CROP);
        let none = Track::with_camera(CROP, CameraTrack::none(), FRAME);
        for t in frames(60_000) {
            assert_eq!(
                none.cmd(t, (540.0, 960.0)),
                today.cmd(t, (540.0, 960.0)),
                "{t:?}"
            );
            assert_eq!(bits(none.src_at(t)), bits(CROP), "{t:?}");
        }
    }

    #[test]
    fn a_scale_of_exactly_one_gives_the_crop_bit_for_bit() {
        // A still segment at 1.0, and the first instant of a moving one.
        let still = crate::camera::build(&[], &[], OutMs::new(500));
        assert_eq!(still.scale_at(OutMs::new(250)), 1.0);
        let moving = crate::camera::build(&[], &[], OutMs::new(3_000));
        assert_eq!(moving.scale_at(OutMs::new(0)), 1.0);
        for (camera, t) in [(still, 250), (moving, 0)] {
            let track = Track::with_camera(CROP, camera, FRAME);
            assert_eq!(bits(track.src_at(OutMs::new(t))), bits(CROP));
        }
    }

    #[test]
    fn a_zoom_shrinks_the_crop_about_its_centre() {
        let camera = crate::camera::build(&[], &[], OutMs::new(3_000));
        let track = Track::with_camera(CROP, camera.clone(), FRAME);
        for t in frames(3_000) {
            let (scale, src) = (camera.scale_at(t), track.src_at(t));
            assert!((src.w - CROP.w / scale).abs() < 1e-3, "{t:?}");
            assert!((src.h - CROP.h / scale).abs() < 1e-3, "{t:?}");
            let centre = (src.x + src.w / 2.0, src.y + src.h / 2.0);
            assert!((centre.0 - (CROP.x + CROP.w / 2.0)).abs() < 1e-3, "{t:?}");
            assert!((centre.1 - (CROP.y + CROP.h / 2.0)).abs() < 1e-3, "{t:?}");
        }
        // Halfway the drift is under way, and inside the crop.
        let mid = track.src_at(OutMs::new(1_500));
        assert!(mid.w < CROP.w && mid.x > CROP.x, "{mid:?}");
    }

    #[test]
    fn a_zoomed_crop_never_leaves_the_source_frame() {
        // A crop at the frame's right edge stays inside when zoomed.
        let edge = CropRect { x: 1_312.5, ..CROP };
        let z = zoomed(edge, 1.35, FRAME);
        assert!(z.x >= 0.0 && z.x + z.w <= FRAME.0 && z.y >= 0.0 && z.y + z.h <= FRAME.1);
        // A crop that reaches past the frame (as a face-centred crop of V6.5
        // could) is moved back inside, its size kept.
        let past = CropRect {
            x: 1_500.0,
            y: -40.0,
            ..CROP
        };
        let z = zoomed(past, 1.05, FRAME);
        assert_eq!(z.x, FRAME.0 - z.w);
        assert_eq!(z.y, 0.0);
        assert!((z.w - CROP.w / 1.05).abs() < 1e-3);
    }

    #[test]
    fn the_scale_is_capped_at_the_total_zoom() {
        let camera = CameraTrack::from_segs(vec![crate::camera::CameraSeg {
            start: OutMs::new(0),
            end: OutMs::new(1_000),
            from: 2.0,
            to: 2.0,
        }]);
        let src = Track::with_camera(CROP, camera, FRAME).src_at(OutMs::new(500));
        assert!((src.w - CROP.w / MAX_TOTAL_ZOOM).abs() < 1e-3, "{src:?}");
    }
}
