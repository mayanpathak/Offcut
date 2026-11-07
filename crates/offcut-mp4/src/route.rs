//! Which path a file takes (TS §43.1, v6.1implementation §6.2, D-395). With
//! both flags off, `route_probe` is `validate_probe`: rule 1 calls it and
//! returns what it returns. Every later rule runs only with a flag on.
//!
//! As read (P172): the signature carries a `VideoDetail` beside the
//! `ProbeInfo` (HDR and the HEVC string, which `ProbeInfo` cannot hold without
//! changing V5's literals) and refuses with a `RouteReject` (`REJECT_HDR` is
//! not a `RejectReason`, whose list V5's tests pin). See `offcut-types/src/source.rs`.

use offcut_types::limits::{INPUT_FPS_TOLERANCE, MAX_CLIP_DURATION, MAX_INPUT_FPS, MAX_LONG_SIDE};
use offcut_types::{
    AudioCodec, ClipInfo, ContainerKind, Entry, MAX_INPUT_LONG_SIDE, MAX_SOURCE_DURATION,
    Orientation, ProbeInfo, Px, RejectReason, Rotation, Route, RouteFlags, RouteReject, SourceInfo,
    SourceReject, VideoCodec, VideoDetail, VideoTrackInfo,
};

use crate::validate::validate_probe;

fn reject(reason: RejectReason) -> Result<Route, RouteReject> {
    Err(RouteReject::Clip(reason))
}

/// The display size, after the rotation.
fn display_size(video: &VideoTrackInfo) -> (Px, Px) {
    match video.rotation {
        Rotation::R0 | Rotation::R180 => (video.coded_width, video.coded_height),
        Rotation::R90 | Rotation::R270 => (video.coded_height, video.coded_width),
    }
}

/// The codec string the browser is given: for HEVC the one built from `hvcC`
/// when there is one; otherwise the probe's, as V5 has it.
fn video_codec_string(video: &VideoTrackInfo, detail: &VideoDetail) -> String {
    match (video.codec, &detail.hevc_codec_string) {
        (VideoCodec::Hevc, Some(built)) => built.clone(),
        _ => video.codec_string.clone(),
    }
}

/// A `ClipInfo` built as `validate_probe` builds it (a property test over
/// every fixture proves the two equal); `validate_probe` keeps its own body.
fn clip_info(p: &ProbeInfo, detail: &VideoDetail) -> Option<ClipInfo> {
    let video = p.video.as_ref()?;
    let audio = p.audio.as_ref()?;
    let (display_width, display_height) = display_size(video);
    // A square clip is landscape (TS §10.3).
    let portrait = display_height > display_width;
    Some(ClipInfo {
        duration: p.duration,
        display_width,
        display_height,
        rotation: video.rotation,
        orientation: if portrait {
            Orientation::Portrait
        } else {
            Orientation::Landscape
        },
        is_vfr: video.is_vfr,
        video_codec_string: video_codec_string(video, detail),
        audio_codec_string: audio.codec_string.clone(),
        audio_sample_rate: audio.sample_rate,
        audio_channels: audio.channels,
    })
}

fn source_info(p: &ProbeInfo, detail: &VideoDetail) -> Option<SourceInfo> {
    let clip = clip_info(p, detail)?;
    let video = p.video.as_ref()?;
    Some(SourceInfo {
        duration: clip.duration,
        display_width: clip.display_width,
        display_height: clip.display_height,
        rotation: clip.rotation,
        orientation: clip.orientation,
        is_vfr: clip.is_vfr,
        is_hdr: detail.is_hdr,
        avg_fps: video.avg_fps,
        video_codec: video.codec,
        video_codec_string: clip.video_codec_string,
        audio_codec_string: clip.audio_codec_string,
        audio_sample_rate: clip.audio_sample_rate,
        audio_channels: clip.audio_channels,
        // The demuxer reads the first audio track in file order (P171).
        audio_track: 0,
        audio_tracks: p.audio_tracks,
    })
}

/// Routes a probed file. `decode_supported` is the browser's answer for the
/// video config (for HEVC, asked with the `hvcC` description, D-398). The
/// file-size rule is not here: the import checks it before any copy, as V5.
pub fn route_probe(
    p: &ProbeInfo,
    detail: &VideoDetail,
    decode_supported: bool,
    entry: Entry,
    flags: RouteFlags,
) -> Result<Route, RouteReject> {
    // The long page does not exist with its flag off; a call from it is a clip's.
    let entry = if flags.long_video { entry } else { Entry::Clip };
    // 1: exact V5.
    if entry == Entry::Clip && !flags.high_res {
        return validate_probe(p, decode_supported)
            .map(Route::Clip)
            .map_err(RouteReject::Clip);
    }
    // 2
    if p.container == ContainerKind::Other {
        return reject(RejectReason::Container);
    }
    // 3
    let Some(video) = p.video.as_ref().filter(|_| p.video_tracks > 0) else {
        return reject(RejectReason::NoVideo);
    };
    // 4: V5's rule while `high_res` is off.
    if video.codec == VideoCodec::Hevc && !flags.high_res {
        return reject(RejectReason::Hevc);
    }
    // 5
    if !matches!(video.codec, VideoCodec::H264 | VideoCodec::Hevc) {
        return reject(RejectReason::VideoCodec);
    }
    // 6: HEVC is refused only when the browser cannot decode it (DC-2).
    if video.codec == VideoCodec::Hevc && !decode_supported {
        return reject(RejectReason::Hevc);
    }
    // 7
    let Some(audio) = p.audio.as_ref().filter(|_| p.audio_tracks > 0) else {
        return reject(RejectReason::NoAudio);
    };
    // 8: the first audio track's codec.
    if audio.codec != AudioCodec::Aac {
        return reject(RejectReason::AudioCodec);
    }
    // 9: the size as shown, after the rotation.
    let (width, height) = display_size(video);
    let max_side = if flags.high_res {
        MAX_INPUT_LONG_SIDE
    } else {
        MAX_LONG_SIDE
    };
    if width.max(height) > max_side {
        return reject(RejectReason::Resolution);
    }
    // 10
    if video.avg_fps.get()
        > MAX_INPUT_FPS
            .get()
            .saturating_add(INPUT_FPS_TOLERANCE.get())
    {
        return reject(RejectReason::FrameRate);
    }
    // 11
    if detail.is_hdr && !flags.hdr_ok {
        return Err(RouteReject::Source(SourceReject::Hdr));
    }
    // 12
    if !decode_supported {
        return reject(RejectReason::DecodeUnsupported);
    }
    // 13: a short file is a clip, from either page.
    if p.duration <= MAX_CLIP_DURATION && p.audio_tracks == 1 {
        return clip_info(p, detail)
            .map(Route::Clip)
            .ok_or(RouteReject::Clip(RejectReason::NoAudio));
    }
    // 14: the clip path keeps its rule.
    if p.duration <= MAX_CLIP_DURATION {
        return reject(RejectReason::MultiAudioTrack);
    }
    // 15: the clip page keeps V5's limit.
    if entry == Entry::Clip {
        return reject(RejectReason::Duration);
    }
    // 16: the long page's limit.
    if p.duration > MAX_SOURCE_DURATION {
        return reject(RejectReason::Duration);
    }
    // 17
    source_info(p, detail)
        .map(Route::Long)
        .ok_or(RouteReject::Clip(RejectReason::NoAudio))
}
