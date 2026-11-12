//! PunchIn: the plan's zoom on a claim, a number or a punchline
//! (v6.2implementation D-478, TS §47.2). It draws nothing in any zone: its
//! track gives the scale of the frame at each time, which the video track
//! multiplies with the camera's drift (`video.rs`, D-477), as a KeywordPop
//! hands its word to the captions.
//!
//! `Snap` jumps to the scale at the first anchor word's program start and
//! holds it over the anchors; `Push` rises from 1.0 on `IN_OUT` across them.
//! Both then release to 1.0 over `ZOOM_RELEASE_MS` on `OUT`. The plan's zooms
//! are not the camera's: they stay with `Enhance.motion` off (§12's copy),
//! and an item switched off has no track.

use offcut_types::{OutMs, OutSpan, ZoomMotion, ZoomScale};

use crate::display_list::DrawCmd;
use crate::easing::{IN_OUT, OUT, bezier};

/// How long the zoom takes to let go after the last anchor word (assumption).
pub const ZOOM_RELEASE_MS: u32 = 300;
/// `Mild` and `Strong` as scales of the frame (assumptions, E-11).
pub const MILD: f32 = 1.12;
pub const STRONG: f32 = 1.25;

/// One zoom: over `span` (the anchors' program time) to `scale`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Track {
    pub span: OutSpan,
    pub scale: f32,
    pub motion: ZoomMotion,
}

/// The item's zoom over its program span.
pub fn layout(scale: ZoomScale, motion: ZoomMotion, span: OutSpan) -> Track {
    let scale = match scale {
        ZoomScale::Mild => MILD,
        ZoomScale::Strong => STRONG,
    };
    Track {
        span,
        scale,
        motion,
    }
}

impl Track {
    /// The scale this zoom gives the frame at `t`: 1.0 before it and after
    /// its release.
    pub fn scale_at(&self, t: OutMs) -> f32 {
        let (start, end, t) = (self.span.start.get(), self.span.end.get(), t.get());
        if t < start {
            return 1.0;
        }
        if t < end {
            return match self.motion {
                ZoomMotion::Snap => self.scale,
                ZoomMotion::Push => {
                    let share = (t - start) as f32 / (end - start) as f32;
                    1.0 + (self.scale - 1.0) * bezier(IN_OUT, share)
                }
            };
        }
        let since = t - end;
        if since >= ZOOM_RELEASE_MS {
            return 1.0;
        }
        let share = since as f32 / ZOOM_RELEASE_MS as f32;
        self.scale + (1.0 - self.scale) * bezier(OUT, share)
    }
}

/// Nothing: the video track draws the zoom.
pub fn draw(track: &Track, t: OutMs, out: &mut Vec<DrawCmd>) {
    let _ = (track, t, out);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(start: u32, end: u32) -> OutSpan {
        OutSpan::new(OutMs::new(start), OutMs::new(end)).unwrap()
    }

    fn at(track: &Track, t: u32) -> f32 {
        track.scale_at(OutMs::new(t))
    }

    #[test]
    fn mild_and_strong_are_their_scales() {
        let s = span(1_000, 2_000);
        assert_eq!(layout(ZoomScale::Mild, ZoomMotion::Snap, s).scale, 1.12);
        assert_eq!(layout(ZoomScale::Strong, ZoomMotion::Snap, s).scale, 1.25);
    }

    #[test]
    fn a_snap_is_in_at_its_first_word_held_then_released_over_300_ms() {
        let snap = layout(ZoomScale::Strong, ZoomMotion::Snap, span(1_000, 2_000));
        assert_eq!(at(&snap, 0), 1.0);
        assert_eq!(at(&snap, 999), 1.0);
        // In within the frame of its first word.
        assert_eq!(at(&snap, 1_000), 1.25);
        assert_eq!(at(&snap, 1_999), 1.25);
        // The release starts from the scale, on OUT, and ends at 1.0.
        assert_eq!(at(&snap, 2_000), 1.25);
        let mid = at(&snap, 2_150);
        let want = 1.25 + (1.0 - 1.25) * bezier(OUT, 0.5);
        assert!((mid - want).abs() < 1e-6, "{mid} {want}");
        assert!(mid < 1.25 && mid > 1.0);
        assert_eq!(at(&snap, 2_000 + ZOOM_RELEASE_MS), 1.0);
        assert_eq!(at(&snap, 9_000), 1.0);
        // Falling all the way through the release.
        let mut before = 1.25;
        for t in 2_000..2_300 {
            let now = at(&snap, t);
            assert!(now <= before, "{t}");
            before = now;
        }
    }

    #[test]
    fn a_push_rises_on_in_out_across_its_anchors() {
        let push = layout(ZoomScale::Mild, ZoomMotion::Push, span(1_000, 3_000));
        assert_eq!(at(&push, 1_000), 1.0);
        // IN_OUT is half way at half the time.
        assert!(
            (at(&push, 2_000) - 1.06).abs() < 1e-5,
            "{}",
            at(&push, 2_000)
        );
        let mut before = 1.0;
        for t in 1_000..3_000 {
            let now = at(&push, t);
            assert!(now >= before && now <= 1.12, "{t}: {now}");
            before = now;
        }
        // At the last word's end it has arrived, and lets go as a snap does.
        assert_eq!(at(&push, 3_000), 1.12);
        assert_eq!(at(&push, 3_300), 1.0);
    }

    #[test]
    fn an_empty_span_only_releases() {
        let push = layout(ZoomScale::Strong, ZoomMotion::Push, span(1_000, 1_000));
        assert_eq!(at(&push, 999), 1.0);
        assert_eq!(at(&push, 1_000), 1.25);
        assert_eq!(at(&push, 1_300), 1.0);
    }

    #[test]
    fn it_draws_nothing() {
        let snap = layout(ZoomScale::Strong, ZoomMotion::Snap, span(0, 500));
        let mut out = Vec::new();
        for t in [0, 250, 600] {
            draw(&snap, OutMs::new(t), &mut out);
        }
        assert!(out.is_empty());
    }
}
