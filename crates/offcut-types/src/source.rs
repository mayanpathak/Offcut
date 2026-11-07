//! The long path's types (TS §43.1-§43.2, v6.1implementation §6.1): which page
//! a file was dropped on, which path it takes, and what a long source is.
//!
//! As read (V6.1, P172): the existing code enums and `VideoTrackInfo` are each
//! pinned by tests that list every member and build every literal, so V6.1 adds
//! beside them instead of inside them. `REJECT_HDR` is a `SourceReject`, carried
//! with the V5 reasons by `RouteReject`; `E_SOURCE_READ` and `E_SOURCE_AUDIO`
//! are a `SourceErrorCode`; `is_hdr` and the HEVC codec string arrive in a
//! `VideoDetail` beside the `ProbeInfo`. The serialized codes are the plan's.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::media::{ClipInfo, Orientation, RejectReason, Rotation, VideoCodec};
use crate::units::{Bytes, DurMs, FpsMilli, Hz, Px};

/// The longest long source: 45 minutes (founder, 2026-11-08; TS §43.2).
pub const MAX_SOURCE_DURATION: DurMs = DurMs::new(2_700_000);
/// The longest side of an input with `high_res` on, or on the long page (TS §43.2).
pub const MAX_INPUT_LONG_SIDE: Px = Px::new(3840);
/// The clip page's file limit with `high_res` on (DC-3), MB read as 10^6 B as `MAX_FILE_SIZE` is.
pub const MAX_FILE_SIZE_HIGH_RES: Bytes = Bytes::new(16_000_000_000);
/// One cloud window's length, in seconds (D-405).
pub const CLOUD_WINDOW_S: u32 = 90;
/// What two windows share, in seconds. 10, not the plan's 5: TE-18's first
/// read lost or doubled a word at 3 of 31 seams with 5 (P171, ON §6).
pub const CLOUD_OVERLAP_S: u32 = 10;
/// The most windows `X-Offcut-Window` may name: 45 minutes' count, plus one (D-411).
pub const MAX_SOURCE_WINDOWS: u32 = 35;

/// How many windows a source of `duration` takes: `max(1, ⌈(D − overlap) ÷ step⌉)` (D-405).
pub fn window_count(duration: DurMs) -> u32 {
    let step_ms = u64::from(CLOUD_WINDOW_S - CLOUD_OVERLAP_S) * 1_000;
    let overlap_ms = u64::from(CLOUD_OVERLAP_S) * 1_000;
    let beyond = u64::from(duration.get()).saturating_sub(overlap_ms);
    u32::try_from(beyond.div_ceil(step_ms))
        .unwrap_or(u32::MAX)
        .max(1)
}

/// The page a file was dropped on (v1.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Entry {
    Clip,
    Long,
}

/// The build's flags that change routing (`features.ts`); `hdr_ok` stays false in V6.1.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct RouteFlags {
    pub long_video: bool,
    pub high_res: bool,
    pub hdr_ok: bool,
}

/// What the video's sample entry says beyond `ProbeInfo` (D-396, D-397).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct VideoDetail {
    /// From `colr`: `nclx` with transfer characteristics 16 (PQ) or 18 (HLG).
    pub is_hdr: bool,
    /// The WebCodecs string built from `hvcC` (e.g. "hvc1.1.6.L150.90"); none for other codecs.
    pub hevc_codec_string: Option<String>,
}

/// A long source; constructed only by `route_probe`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
pub struct SourceInfo {
    pub duration: DurMs,
    /// After rotation.
    pub display_width: Px,
    /// After rotation.
    pub display_height: Px,
    pub rotation: Rotation,
    pub orientation: Orientation,
    pub is_vfr: bool,
    pub is_hdr: bool,
    pub avg_fps: FpsMilli,
    pub video_codec: VideoCodec,
    pub video_codec_string: String,
    pub audio_codec_string: String,
    pub audio_sample_rate: Hz,
    pub audio_channels: u32,
    /// The track the long path reads: the first audio track in file order (P171).
    pub audio_track: u32,
    /// Every audio track; the long path accepts more than one.
    pub audio_tracks: u32,
}

/// Which path a file takes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Route {
    Clip(ClipInfo),
    Long(SourceInfo),
}

/// The reasons V6.1 adds to the V5 list, serialized as their `REJECT_*` codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub enum SourceReject {
    #[serde(rename = "REJECT_HDR")]
    Hdr,
}

/// Why `route_probe` refuses a file: one V5 reason, or one V6.1 adds. It
/// serializes as the bare code, so the two sets read as one list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(untagged)]
pub enum RouteReject {
    Clip(RejectReason),
    Source(SourceReject),
}

/// The error codes the long path adds (TS §43.9), serialized as their `E_*` codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub enum SourceErrorCode {
    /// The file moved, was deleted or cannot be read any more.
    #[serde(rename = "E_SOURCE_READ")]
    SourceRead,
    /// The file's sound cannot be decoded.
    #[serde(rename = "E_SOURCE_AUDIO")]
    SourceAudio,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::limits::{MAX_CLIP_DURATION, MAX_FILE_SIZE, MAX_LONG_SIDE};

    #[test]
    fn the_window_count_follows_d405_with_a_10_s_overlap() {
        assert_eq!(window_count(DurMs::new(0)), 1);
        assert_eq!(window_count(DurMs::new(60_000)), 1);
        assert_eq!(window_count(DurMs::new(90_000)), 1);
        assert_eq!(window_count(DurMs::new(90_001)), 2);
        assert_eq!(window_count(DurMs::new(170_000)), 2);
        assert_eq!(window_count(DurMs::new(720_000)), 9);
        assert_eq!(window_count(MAX_SOURCE_DURATION), 34);
        // ⌈(2700 − 10) ÷ 80⌉ + 1: the 45-minute count with one to spare.
        assert_eq!(MAX_SOURCE_WINDOWS, window_count(MAX_SOURCE_DURATION) + 1);
    }

    #[test]
    fn every_window_but_the_last_is_whole_and_the_last_is_longer_than_the_overlap() {
        for seconds in [91u32, 100, 171, 600, 1_234, 2_699, 2_700] {
            let d = seconds * 1_000;
            let n = window_count(DurMs::new(d));
            let step = (CLOUD_WINDOW_S - CLOUD_OVERLAP_S) * 1_000;
            let last_start = (n - 1) * step;
            assert!(
                last_start + CLOUD_WINDOW_S * 1_000 >= d,
                "{seconds}: the windows end before the source"
            );
            assert!(
                d - last_start > CLOUD_OVERLAP_S * 1_000,
                "{seconds}: the last window is only overlap"
            );
        }
    }

    #[test]
    fn the_long_limits_sit_above_the_clip_limits() {
        assert!(MAX_SOURCE_DURATION > MAX_CLIP_DURATION);
        assert!(MAX_INPUT_LONG_SIDE > MAX_LONG_SIDE);
        assert!(MAX_FILE_SIZE_HIGH_RES > MAX_FILE_SIZE);
        // DC-3: the in-page fixture references 15,259 × 1 MiB, just above 16 GB.
        assert!(15_259 * 1_048_576 > MAX_FILE_SIZE_HIGH_RES.get());
        assert!(15_258 * 1_048_576 < MAX_FILE_SIZE_HIGH_RES.get());
    }

    #[test]
    fn route_reject_serializes_as_the_bare_code_of_either_list() {
        let hdr = RouteReject::Source(SourceReject::Hdr);
        assert_eq!(serde_json::to_string(&hdr).unwrap(), "\"REJECT_HDR\"");
        let duration = RouteReject::Clip(RejectReason::Duration);
        assert_eq!(
            serde_json::to_string(&duration).unwrap(),
            "\"REJECT_DURATION\""
        );
        assert_eq!(
            serde_json::from_str::<RouteReject>("\"REJECT_HDR\"").unwrap(),
            hdr
        );
        assert_eq!(
            serde_json::from_str::<RouteReject>("\"REJECT_DURATION\"").unwrap(),
            duration
        );
        assert!(serde_json::from_str::<RouteReject>("\"REJECT_UNKNOWN\"").is_err());
        assert_eq!(
            RouteReject::inline(&ts_rs::Config::default()),
            "RejectReason | SourceReject"
        );
    }

    #[test]
    fn source_error_codes_serialize_as_e_codes() {
        assert_eq!(
            serde_json::to_string(&SourceErrorCode::SourceRead).unwrap(),
            "\"E_SOURCE_READ\""
        );
        assert_eq!(
            serde_json::to_string(&SourceErrorCode::SourceAudio).unwrap(),
            "\"E_SOURCE_AUDIO\""
        );
        assert_eq!(
            SourceErrorCode::inline(&ts_rs::Config::default()),
            "\"E_SOURCE_READ\" | \"E_SOURCE_AUDIO\""
        );
    }

    #[test]
    fn route_and_entry_serialize_as_the_worker_protocol_reads_them() {
        assert_eq!(serde_json::to_string(&Entry::Long).unwrap(), "\"long\"");
        let flags = RouteFlags {
            long_video: true,
            high_res: false,
            hdr_ok: false,
        };
        assert_eq!(
            serde_json::to_string(&flags).unwrap(),
            r#"{"long_video":true,"high_res":false,"hdr_ok":false}"#
        );
        let info = SourceInfo {
            duration: DurMs::new(1_800_000),
            display_width: Px::new(1920),
            display_height: Px::new(1080),
            rotation: Rotation::R0,
            orientation: Orientation::Landscape,
            is_vfr: false,
            is_hdr: false,
            avg_fps: FpsMilli::new(30_000),
            video_codec: VideoCodec::H264,
            video_codec_string: "avc1.640028".to_owned(),
            audio_codec_string: "mp4a.40.2".to_owned(),
            audio_sample_rate: Hz::new(48_000),
            audio_channels: 2,
            audio_track: 0,
            audio_tracks: 1,
        };
        let json = serde_json::to_value(Route::Long(info.clone())).unwrap();
        assert_eq!(json["long"]["duration"], 1_800_000);
        assert_eq!(
            serde_json::from_value::<Route>(json).unwrap(),
            Route::Long(info)
        );
    }
}
