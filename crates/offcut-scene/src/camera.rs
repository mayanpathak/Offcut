//! The camera drift (v6.2implementation D-476, §10.1; TS §47.2, "motion by
//! rules"): every shot moves slowly, so that no frame of a short stands
//! still. Pure: from the times of the clip's visual changes, its sentences
//! and its length to a scale at each program time, which `video.rs` turns
//! into the crop. `motion.rs` is the captions' motion (D-137), not this.
//!
//! Segments run from change to change. One longer than `MAX_SEG_MS` is split
//! at the sentence start nearest its middle, again until each is short
//! enough or has no sentence start inside. Each segment drifts by `DRIFT`,
//! in then out then in, on the `STANDARD` curve, so the scale is continuous
//! and still at every boundary; one shorter than `MIN_SEG_MS` holds where the
//! last one ended. V6.2's only change is the clip's start (the hook's end
//! joins when P205 draws the hook); there are no shots before V6.5, which is
//! why production keeps `motion` off until then (D-497).

use offcut_types::{OutMs, OutSpan, TimeMap, Transcript};

use crate::easing::{STANDARD, bezier};

/// A segment longer than this is split (assumption, TS §47.2).
pub const MAX_SEG_MS: u32 = 4_000;
/// A segment shorter than this holds still (assumption).
pub const MIN_SEG_MS: u32 = 800;
/// How far a segment drifts: from 1.00 to 1.05, or back (assumption).
pub const DRIFT: f32 = 0.05;

/// One stretch of program time `[start, end)`, its scale moving from `from`
/// to `to` on `STANDARD`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraSeg {
    pub start: OutMs,
    pub end: OutMs,
    pub from: f32,
    pub to: f32,
}

/// The scale of the frame over the program: 1.0 where no segment is.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CameraTrack {
    segs: Vec<CameraSeg>,
}

impl CameraTrack {
    /// No drift: 1.0 at every time, today's frames (`Enhance.motion` off).
    pub fn none() -> CameraTrack {
        CameraTrack { segs: Vec::new() }
    }

    /// A track of these segments, as given: for tests of the zoom's cap.
    #[cfg(test)]
    pub(crate) fn from_segs(segs: Vec<CameraSeg>) -> CameraTrack {
        CameraTrack { segs }
    }

    /// The segments, in time order, end to end.
    pub fn segs(&self) -> &[CameraSeg] {
        &self.segs
    }

    /// The scale at `t`: 1.0 outside every segment.
    pub fn scale_at(&self, t: OutMs) -> f32 {
        let Some(seg) = self.segs.iter().find(|s| s.start <= t && t < s.end) else {
            return 1.0;
        };
        let length = seg.end.get() - seg.start.get();
        let share = (t.get() - seg.start.get()) as f32 / length as f32;
        seg.from + (seg.to - seg.from) * bezier(STANDARD, share)
    }
}

/// The drift of a program `duration` long whose visual changes are at
/// `changes` (0 and the end are always ones) and whose sentences are
/// `sentences`. A change outside the program is ignored.
pub fn build(changes: &[OutMs], sentences: &[OutSpan], duration: OutMs) -> CameraTrack {
    let end = duration.get();
    let mut bounds: Vec<u32> = changes
        .iter()
        .map(|c| c.get())
        .filter(|c| *c > 0 && *c < end)
        .collect();
    bounds.extend([0, end]);
    bounds.sort_unstable();
    bounds.dedup();
    let mut starts: Vec<u32> = sentences.iter().map(|s| s.start.get()).collect();
    starts.sort_unstable();
    starts.dedup();

    let mut pieces = Vec::new();
    for pair in bounds.windows(2) {
        if let [from, to] = pair {
            split(*from, *to, &starts, &mut pieces);
        }
    }

    let (low, high) = (1.0, 1.0 + DRIFT);
    let mut at_high = false;
    let segs = pieces
        .into_iter()
        .filter(|(from, to)| to > from)
        .map(|(from, to)| {
            let level = if at_high { high } else { low };
            let next = if to - from < MIN_SEG_MS {
                level
            } else {
                at_high = !at_high;
                if at_high { high } else { low }
            };
            CameraSeg {
                start: OutMs::new(from),
                end: OutMs::new(to),
                from: level,
                to: next,
            }
        })
        .collect();
    CameraTrack { segs }
}

/// `[start, end)` as pieces of at most `MAX_SEG_MS`, each cut at the sentence
/// start nearest the middle of what it cuts (the earlier of two as near).
/// A stretch with no sentence start inside stays whole.
fn split(start: u32, end: u32, starts: &[u32], out: &mut Vec<(u32, u32)>) {
    if end - start <= MAX_SEG_MS {
        out.push((start, end));
        return;
    }
    let middle = start + (end - start) / 2;
    let cut = starts
        .iter()
        .copied()
        .filter(|s| *s > start && *s < end)
        .min_by_key(|s| s.abs_diff(middle));
    match cut {
        Some(at) => {
            split(start, at, starts, out);
            split(at, end, starts, out);
        }
        None => out.push((start, end)),
    }
}

/// The drift of a clip with `Enhance.motion` on, as `build_scene` makes it:
/// from the clip's start, over its sentences in program time.
pub fn for_clip(transcript: &Transcript, map: &TimeMap) -> CameraTrack {
    let sentences: Vec<OutSpan> = transcript
        .sentences
        .iter()
        .filter_map(|s| OutSpan::new(map.to_out(s.start_ms)?, map.to_out(s.end_ms)?))
        .collect();
    let duration = OutMs::new(map.out_duration().get());
    build(&[OutMs::new(0)], &sentences, duration)
}
