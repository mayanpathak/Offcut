//! [ONLY] The map between the recording's time and the program's (TS §40.1,
//! v4implementation D-125). The transcript, the events and the plan keep
//! `TimeMs`; the scene, the clock, the frame index and the program audio run
//! on `OutMs`; this file is the only place one becomes the other.
//!
//! V4 builds one map only, the identity of a clip's duration. A map that is
//! not the identity is made from a plan's cuts in V6.2 (`from_segments`);
//! until then it can only arrive by deserialization, which checks it.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use ts_rs::TS;

use crate::units::{DurMs, OutMs, OutSpan, Span, TimeMs};

/// One stretch of the program, in program order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Segment {
    /// The recording from `src.start` to `src.end`, played as it is.
    Source { src: Span },
    /// The frame of `at` held for `dur` (an end card, V6.6).
    Hold { at: TimeMs, dur: DurMs },
}

/// Where a program time is in the recording. Not serialized: the binding
/// returns it as a plain object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SrcPos {
    /// This instant of the recording plays.
    At(TimeMs),
    /// This frame of the recording is held.
    Held(TimeMs),
}

/// The program as a list of segments. The field is private: `identity` is the
/// one constructor in V4, and deserialization checks what it reads.
#[derive(Clone, Debug, PartialEq, Serialize, TS)]
pub struct TimeMap {
    segments: Vec<Segment>,
}

/// The loudness of a recording, one value per 10 ms, in dB: what the time map
/// snaps its splices to from V6.2 (TS §40.6). Made by `offcut-dsp`; not
/// serialized.
#[derive(Clone, Debug, PartialEq)]
pub struct SilenceProfile {
    pub rms_db_10ms: Vec<f32>,
}

impl Segment {
    /// How long it plays.
    fn len(&self) -> u32 {
        match self {
            Segment::Source { src } => src.end.get().saturating_sub(src.start.get()),
            Segment::Hold { dur, .. } => dur.get(),
        }
    }
}

impl TimeMap {
    /// The recording as it is: one `Source` segment from 0 to `duration`.
    pub fn identity(duration: DurMs) -> TimeMap {
        let src = Span {
            start: TimeMs::new(0),
            end: TimeMs::new(duration.get()),
        };
        TimeMap {
            segments: vec![Segment::Source { src }],
        }
    }

    /// Exactly one `Source` segment, starting at 0.
    pub fn is_identity(&self) -> bool {
        matches!(self.segments.as_slice(), [Segment::Source { src }] if src.start.get() == 0)
    }

    /// The length of the program: the sum of the segments' lengths.
    pub fn out_duration(&self) -> DurMs {
        let total = self
            .segments
            .iter()
            .fold(0u32, |sum, segment| sum.saturating_add(segment.len()));
        DurMs::new(total)
    }

    /// The program time of `t`. `None` when `t` lies in a part of the
    /// recording that is not played, or past its end. The end of a `Source`
    /// segment maps to the end of its stretch of the program, so a word that
    /// ends where the clip ends maps.
    pub fn to_out(&self, t: TimeMs) -> Option<OutMs> {
        let mut out = 0u32;
        for segment in &self.segments {
            if let Segment::Source { src } = segment
                && src.start <= t
                && t <= src.end
            {
                let into = t.get().saturating_sub(src.start.get());
                return Some(OutMs::new(out.saturating_add(into)));
            }
            out = out.saturating_add(segment.len());
        }
        None
    }

    /// The recording at program time `t`. Past the end, the last instant of
    /// the last segment.
    pub fn to_src(&self, t: OutMs) -> SrcPos {
        let mut out = 0u32;
        let mut last = SrcPos::At(TimeMs::new(0));
        for segment in &self.segments {
            let into = t.get().saturating_sub(out);
            let len = segment.len();
            match segment {
                Segment::Source { src } => {
                    if t.get() >= out && into < len {
                        return SrcPos::At(TimeMs::new(src.start.get().saturating_add(into)));
                    }
                    last = SrcPos::At(src.end);
                }
                Segment::Hold { at, .. } => {
                    if t.get() >= out && into < len {
                        return SrcPos::Held(*at);
                    }
                    last = SrcPos::Held(*at);
                }
            }
            out = out.saturating_add(len);
        }
        last
    }

    /// Both ends of `s` in program time; `None` when either is not played.
    pub fn out_span(&self, s: Span) -> Option<OutSpan> {
        OutSpan::new(self.to_out(s.start)?, self.to_out(s.end)?)
    }

    /// The program time of every boundary between two segments, ascending.
    pub fn joins(&self) -> Vec<OutMs> {
        let mut out = 0u32;
        let mut joins = Vec::new();
        for (i, segment) in self.segments.iter().enumerate() {
            if i > 0 {
                joins.push(OutMs::new(out));
            }
            out = out.saturating_add(segment.len());
        }
        joins
    }
}

/// What deserialization reads before it checks.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UncheckedTimeMap {
    segments: Vec<Segment>,
}

impl<'de> Deserialize<'de> for TimeMap {
    /// Accepts a non-empty list in which every `Source` span is non-empty and
    /// starts at or after the end of the one before, and no `Hold` is empty.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let UncheckedTimeMap { segments } = UncheckedTimeMap::deserialize(deserializer)?;
        if segments.is_empty() {
            return Err(D::Error::custom(
                "a time map must have at least one segment",
            ));
        }
        let mut played_until: Option<TimeMs> = None;
        for segment in &segments {
            match segment {
                Segment::Source { src } => {
                    if src.start >= src.end {
                        return Err(D::Error::custom("a source segment must not be empty"));
                    }
                    if played_until.is_some_and(|until| src.start < until) {
                        return Err(D::Error::custom(
                            "source segments must follow each other in the recording",
                        ));
                    }
                    played_until = Some(src.end);
                }
                Segment::Hold { dur, .. } => {
                    if dur.get() == 0 {
                        return Err(D::Error::custom("a hold segment must not be empty"));
                    }
                }
            }
        }
        Ok(TimeMap { segments })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(v: u32) -> TimeMs {
        TimeMs::new(v)
    }

    fn span(start: u32, end: u32) -> Span {
        Span::new(ms(start), ms(end)).unwrap()
    }

    fn refusal(json: &str) -> String {
        serde_json::from_str::<TimeMap>(json)
            .unwrap_err()
            .to_string()
    }

    #[test]
    fn the_identity_maps_every_time_to_its_own_number() {
        let map = TimeMap::identity(DurMs::new(60_000));
        assert!(map.is_identity());
        assert_eq!(map.out_duration(), DurMs::new(60_000));
        for t in [0, 1, 33, 59_999, 60_000] {
            let out = map.to_out(ms(t)).unwrap();
            assert_eq!(out, OutMs::new(t));
            assert_eq!(map.to_src(out), SrcPos::At(ms(t)));
        }
        // The end instant maps (a word that ends with the clip); after it nothing does.
        assert_eq!(map.to_out(ms(60_000)), Some(OutMs::new(60_000)));
        assert_eq!(map.to_out(ms(60_001)), None);
        // Past the end, the last instant.
        assert_eq!(map.to_src(OutMs::new(70_000)), SrcPos::At(ms(60_000)));
        assert_eq!(map.joins(), []);
    }

    #[test]
    fn spans_of_the_identity_keep_their_numbers() {
        let map = TimeMap::identity(DurMs::new(60_000));
        for (start, end) in [
            (0, 0),
            (0, 1),
            (1_560, 8_880),
            (59_999, 60_000),
            (0, 60_000),
        ] {
            let out = map.out_span(span(start, end)).unwrap();
            assert_eq!((out.start.get(), out.end.get()), (start, end));
        }
        assert_eq!(map.out_span(span(59_000, 61_000)), None);
    }

    #[test]
    fn a_checked_map_with_a_removed_span_and_a_hold() {
        // Recording 0-10 s and 12-20 s played, then the frame of 20 s held for 2.5 s.
        let json = r#"{"segments":[
            {"kind":"source","src":{"start":0,"end":10000}},
            {"kind":"source","src":{"start":12000,"end":20000}},
            {"kind":"hold","at":20000,"dur":2500}
        ]}"#;
        let map: TimeMap = serde_json::from_str(json).unwrap();
        assert!(!map.is_identity());
        assert_eq!(map.out_duration(), DurMs::new(20_500));
        assert_eq!(map.to_out(ms(11_000)), None);
        assert_eq!(map.to_out(ms(10_000)), Some(OutMs::new(10_000)));
        assert_eq!(map.to_out(ms(12_000)), Some(OutMs::new(10_000)));
        assert_eq!(map.to_out(ms(15_000)), Some(OutMs::new(13_000)));
        assert_eq!(map.to_src(OutMs::new(13_000)), SrcPos::At(ms(15_000)));
        assert_eq!(map.to_src(OutMs::new(19_000)), SrcPos::Held(ms(20_000)));
        assert_eq!(map.to_src(OutMs::new(25_000)), SrcPos::Held(ms(20_000)));
        assert_eq!(map.joins(), [OutMs::new(10_000), OutMs::new(18_000)]);
        assert_eq!(map.out_span(span(9_000, 11_000)), None);
        let out = map.out_span(span(13_000, 14_000)).unwrap();
        assert_eq!((out.start.get(), out.end.get()), (11_000, 12_000));
        // It serializes as it was read.
        let back: TimeMap = serde_json::from_str(&serde_json::to_string(&map).unwrap()).unwrap();
        assert_eq!(back, map);
    }

    #[test]
    fn the_identity_serializes_as_one_source_segment() {
        let map = TimeMap::identity(DurMs::new(74_705));
        let json = serde_json::to_string(&map).unwrap();
        assert_eq!(
            json,
            r#"{"segments":[{"kind":"source","src":{"start":0,"end":74705}}]}"#
        );
        assert_eq!(serde_json::from_str::<TimeMap>(&json).unwrap(), map);
    }

    #[test]
    fn deserialization_refuses_each_broken_rule() {
        assert!(refusal(r#"{"segments":[]}"#).contains("at least one segment"));
        assert!(
            refusal(r#"{"segments":[{"kind":"source","src":{"start":500,"end":500}}]}"#)
                .contains("must not be empty")
        );
        assert!(
            refusal(r#"{"segments":[{"kind":"source","src":{"start":900,"end":500}}]}"#)
                .contains("must not be empty")
        );
        let decreasing = r#"{"segments":[
            {"kind":"source","src":{"start":5000,"end":9000}},
            {"kind":"source","src":{"start":1000,"end":2000}}
        ]}"#;
        assert!(refusal(decreasing).contains("follow each other"));
        let zero_hold = r#"{"segments":[
            {"kind":"source","src":{"start":0,"end":9000}},
            {"kind":"hold","at":9000,"dur":0}
        ]}"#;
        assert!(refusal(zero_hold).contains("hold segment must not be empty"));
        assert!(refusal(r#"{"segments":[],"extra":1}"#).contains("unknown field"));
        assert!(
            refusal(r#"{"segments":[{"kind":"source","src":{"start":0,"end":1},"x":0}]}"#)
                .contains("unknown field")
        );
    }
}
