#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! `route_probe` (v6.1implementation §6.2, §11.1): every rule from both pages;
//! with both flags off it is `validate_probe` on every generated fixture; the
//! earlier of two failing rules wins; the long page with its flag off is the
//! clip page; a long source reads its first audio track.

use std::path::PathBuf;

use offcut_mp4::demux::Demuxer;
use offcut_mp4::probe::{probe, video_detail};
use offcut_mp4::reader::MemReader;
use offcut_mp4::route::route_probe;
use offcut_mp4::validate::validate_probe;
use offcut_types::{
    AudioCodec, AudioTrackInfo, Bytes, ContainerKind, DurMs, Entry, FpsMilli, Hz, ProbeInfo, Px,
    RejectReason, Rotation, Route, RouteFlags, RouteReject, SourceReject, VideoCodec, VideoDetail,
    VideoTrackInfo,
};
use sha2::{Digest, Sha256};

const STALE: &str = "the generated fixtures are stale or missing: run `pnpm fixtures`";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A generated fixture's bytes, after checking `fixtures/.gen/` matches this checkout's manifest.
fn fixture(name: &str) -> Vec<u8> {
    let dir = std::env::var_os("OFFCUT_FIXTURES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join("fixtures/.gen"));
    let stamp = std::fs::read_to_string(dir.join("STAMP")).expect(STALE);
    let manifest = std::fs::read(root().join("fixtures/manifest.json")).expect(STALE);
    let sha: String = Sha256::digest(&manifest)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert!(
        stamp.contains(&format!("\"manifest_sha256\": \"{sha}\"")),
        "{STALE}"
    );
    std::fs::read(dir.join(name)).expect(STALE)
}

/// Every fixture the manifest names.
fn fixture_names() -> Vec<String> {
    let text = std::fs::read_to_string(root().join("fixtures/manifest.json")).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&text).unwrap();
    manifest["fixtures"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["name"].as_str().unwrap().to_owned())
        .collect()
}

/// The probe and the detail of a fixture, or `None` for a file the demuxer refuses.
fn probed(name: &str) -> Option<(ProbeInfo, VideoDetail)> {
    let bytes = fixture(name);
    let size = Bytes::new(bytes.len() as u64);
    let mut d = Demuxer::open(MemReader(bytes)).ok()?;
    let p = probe(&d, size);
    Some((p, video_detail(&mut d)))
}

const OFF: RouteFlags = RouteFlags {
    long_video: false,
    high_res: false,
    hdr_ok: false,
};
const LONG: RouteFlags = RouteFlags {
    long_video: true,
    high_res: false,
    hdr_ok: false,
};
const HIGH: RouteFlags = RouteFlags {
    long_video: false,
    high_res: true,
    hdr_ok: false,
};
const BOTH: RouteFlags = RouteFlags {
    long_video: true,
    high_res: true,
    hdr_ok: false,
};

/// 10 minutes of 1920x1080 H.264 at 30 fps with one AAC track: a long source every rule accepts.
fn long() -> ProbeInfo {
    ProbeInfo {
        container: ContainerKind::Mp4,
        file_size: Bytes::new(1_200_000_000),
        duration: DurMs::new(600_000),
        video_tracks: 1,
        audio_tracks: 1,
        video: Some(VideoTrackInfo {
            codec: VideoCodec::H264,
            codec_string: "avc1.640028".to_owned(),
            coded_width: Px::new(1920),
            coded_height: Px::new(1080),
            rotation: Rotation::R0,
            avg_fps: FpsMilli::new(30_000),
            max_fps: FpsMilli::new(30_000),
            is_vfr: false,
            frame_count: 18_000,
        }),
        audio: Some(AudioTrackInfo {
            codec: AudioCodec::Aac,
            codec_string: "mp4a.40.2".to_owned(),
            sample_rate: Hz::new(48_000),
            channels: 2,
        }),
    }
}

fn with(change: impl FnOnce(&mut ProbeInfo)) -> ProbeInfo {
    let mut p = long();
    change(&mut p);
    p
}

fn video(p: &mut ProbeInfo) -> &mut VideoTrackInfo {
    p.video.as_mut().unwrap()
}

fn sdr() -> VideoDetail {
    VideoDetail::default()
}

fn hdr() -> VideoDetail {
    VideoDetail {
        is_hdr: true,
        hevc_codec_string: None,
    }
}

fn rejected(r: RejectReason) -> Result<Route, RouteReject> {
    Err(RouteReject::Clip(r))
}

fn is_long(result: &Result<Route, RouteReject>) -> bool {
    matches!(result, Ok(Route::Long(_)))
}

#[test]
fn with_both_flags_off_route_probe_is_validate_probe_on_every_fixture() {
    let mut compared = 0;
    for name in fixture_names() {
        let Some((p, detail)) = probed(&name) else {
            continue;
        };
        for decode in [true, false] {
            let v5 = validate_probe(&p, decode)
                .map(Route::Clip)
                .map_err(RouteReject::Clip);
            // Rule 1 holds whenever high_res is off on the clip page, and the long
            // page with its flag off is the clip page.
            for (entry, flags) in [(Entry::Clip, OFF), (Entry::Clip, LONG), (Entry::Long, OFF)] {
                assert_eq!(route_probe(&p, &detail, decode, entry, flags), v5, "{name}");
            }
            compared += 1;
        }
    }
    // Every fixture the demuxer opens (all but the WebM one), twice.
    assert!(compared >= 2 * 24, "only {compared} probes compared");
}

#[test]
fn with_high_res_on_every_fixture_v5_accepts_is_the_same_clip() {
    let mut accepted = 0;
    for name in fixture_names() {
        let Some((p, detail)) = probed(&name) else {
            continue;
        };
        let Ok(clip) = validate_probe(&p, true) else {
            continue;
        };
        assert_eq!(
            route_probe(&p, &detail, true, Entry::Clip, HIGH),
            Ok(Route::Clip(clip.clone())),
            "{name}"
        );
        // A short file dropped on the long page opens as the same clip (D-421).
        assert_eq!(
            route_probe(&p, &detail, true, Entry::Long, BOTH),
            Ok(Route::Clip(clip)),
            "{name}"
        );
        accepted += 1;
    }
    assert!(accepted >= 8, "only {accepted} accepted fixtures");
}

#[test]
fn high_res_accepts_rej_4k_and_rej_hevc_where_the_browser_decodes_it() {
    let (p, detail) = probed("rej_4k.mp4").unwrap();
    assert_eq!(
        route_probe(&p, &detail, true, Entry::Clip, OFF),
        rejected(RejectReason::Resolution)
    );
    let Ok(Route::Clip(clip)) = route_probe(&p, &detail, true, Entry::Clip, HIGH) else {
        panic!("rej_4k.mp4 is not a clip with high_res on");
    };
    assert_eq!(
        clip.display_width.get().max(clip.display_height.get()),
        3840
    );

    let (p, detail) = probed("rej_hevc.mov").unwrap();
    assert_eq!(detail.hevc_codec_string.as_deref(), Some("hvc1.1.6.L93.90"));
    assert!(!detail.is_hdr);
    assert_eq!(
        route_probe(&p, &detail, true, Entry::Clip, OFF),
        rejected(RejectReason::Hevc)
    );
    let Ok(Route::Clip(clip)) = route_probe(&p, &detail, true, Entry::Clip, HIGH) else {
        panic!("rej_hevc.mov is not a clip with high_res on and a decoder");
    };
    assert_eq!(clip.video_codec_string, "hvc1.1.6.L93.90");
    assert_eq!(
        route_probe(&p, &detail, false, Entry::Clip, HIGH),
        rejected(RejectReason::Hevc)
    );
}

#[test]
fn each_rule_from_the_long_page_violated_alone_gives_its_reason() {
    assert!(is_long(&route_probe(
        &long(),
        &sdr(),
        true,
        Entry::Long,
        LONG
    )));
    let broken = [
        (
            with(|p| p.container = ContainerKind::Other),
            RejectReason::Container,
        ),
        (
            with(|p| (p.video_tracks, p.video) = (0, None)),
            RejectReason::NoVideo,
        ),
        (
            with(|p| video(p).codec = VideoCodec::Hevc),
            RejectReason::Hevc,
        ),
        (
            with(|p| video(p).codec = VideoCodec::Vp9),
            RejectReason::VideoCodec,
        ),
        (
            with(|p| (p.audio_tracks, p.audio) = (0, None)),
            RejectReason::NoAudio,
        ),
        (
            with(|p| p.audio.as_mut().unwrap().codec = AudioCodec::Mp3),
            RejectReason::AudioCodec,
        ),
        (
            with(|p| video(p).coded_width = Px::new(1_921)),
            RejectReason::Resolution,
        ),
        (
            with(|p| video(p).avg_fps = FpsMilli::new(60_501)),
            RejectReason::FrameRate,
        ),
        (
            with(|p| p.duration = DurMs::new(2_700_001)),
            RejectReason::Duration,
        ),
        (
            with(|p| (p.duration, p.audio_tracks) = (DurMs::new(60_000), 2)),
            RejectReason::MultiAudioTrack,
        ),
    ];
    for (p, reason) in broken {
        assert_eq!(
            route_probe(&p, &sdr(), true, Entry::Long, LONG),
            rejected(reason),
            "{reason:?}"
        );
    }
    assert_eq!(
        route_probe(&long(), &hdr(), true, Entry::Long, LONG),
        Err(RouteReject::Source(SourceReject::Hdr))
    );
    assert_eq!(
        route_probe(&long(), &sdr(), false, Entry::Long, LONG),
        rejected(RejectReason::DecodeUnsupported)
    );
}

#[test]
fn high_res_moves_the_resolution_limit_and_lets_hevc_through_on_both_pages() {
    let uhd =
        with(|p| (video(p).coded_width, video(p).coded_height) = (Px::new(3840), Px::new(2160)));
    assert_eq!(
        route_probe(&uhd, &sdr(), true, Entry::Long, LONG),
        rejected(RejectReason::Resolution)
    );
    assert!(is_long(&route_probe(&uhd, &sdr(), true, Entry::Long, BOTH)));
    let eight_k = with(|p| video(p).coded_width = Px::new(7680));
    assert_eq!(
        route_probe(&eight_k, &sdr(), true, Entry::Long, BOTH),
        rejected(RejectReason::Resolution)
    );
    // Rotated: the long side is measured as shown.
    let rotated = with(|p| {
        (video(p).coded_width, video(p).coded_height) = (Px::new(2160), Px::new(3840));
        video(p).rotation = Rotation::R90;
    });
    let Ok(Route::Long(info)) = route_probe(&rotated, &sdr(), true, Entry::Long, BOTH) else {
        panic!("a rotated 4K source is not long");
    };
    assert_eq!(
        (info.display_width.get(), info.display_height.get()),
        (3840, 2160)
    );

    let hevc = with(|p| video(p).codec = VideoCodec::Hevc);
    let built = VideoDetail {
        is_hdr: false,
        hevc_codec_string: Some("hvc1.1.6.L150.90".to_owned()),
    };
    let Ok(Route::Long(info)) = route_probe(&hevc, &built, true, Entry::Long, BOTH) else {
        panic!("a decodable HEVC source is not long");
    };
    assert_eq!(info.video_codec_string, "hvc1.1.6.L150.90");
    assert_eq!(info.video_codec, VideoCodec::Hevc);
    assert_eq!(
        route_probe(&hevc, &built, false, Entry::Long, BOTH),
        rejected(RejectReason::Hevc)
    );
}

#[test]
fn the_clip_page_with_high_res_keeps_v5s_duration_and_track_rules() {
    assert_eq!(
        route_probe(&long(), &sdr(), true, Entry::Clip, BOTH),
        rejected(RejectReason::Duration)
    );
    let short = with(|p| p.duration = DurMs::new(90_000));
    assert!(matches!(
        route_probe(&short, &sdr(), true, Entry::Clip, HIGH),
        Ok(Route::Clip(_))
    ));
    let two = with(|p| (p.duration, p.audio_tracks) = (DurMs::new(60_000), 2));
    assert_eq!(
        route_probe(&two, &sdr(), true, Entry::Clip, HIGH),
        rejected(RejectReason::MultiAudioTrack)
    );
    assert_eq!(
        route_probe(&short, &hdr(), true, Entry::Clip, HIGH),
        Err(RouteReject::Source(SourceReject::Hdr))
    );
}

#[test]
fn of_two_failing_rules_the_earlier_one_wins() {
    // HDR (11) and no decoder (12).
    assert_eq!(
        route_probe(&long(), &hdr(), false, Entry::Long, LONG),
        Err(RouteReject::Source(SourceReject::Hdr))
    );
    // Resolution (9) and frame rate (10).
    let p = with(|p| {
        video(p).coded_width = Px::new(4_000);
        video(p).avg_fps = FpsMilli::new(120_000);
    });
    assert_eq!(
        route_probe(&p, &sdr(), true, Entry::Long, BOTH),
        rejected(RejectReason::Resolution)
    );
    // Audio codec (8) before resolution (9): V5 has the same order.
    let p = with(|p| {
        p.audio.as_mut().unwrap().codec = AudioCodec::Opus;
        video(p).coded_width = Px::new(4_000);
    });
    assert_eq!(
        route_probe(&p, &sdr(), true, Entry::Long, LONG),
        rejected(RejectReason::AudioCodec)
    );
    // Too long (16) loses to HDR (11).
    let p = with(|p| p.duration = DurMs::new(3_000_000));
    assert_eq!(
        route_probe(&p, &hdr(), true, Entry::Long, LONG),
        Err(RouteReject::Source(SourceReject::Hdr))
    );
}

#[test]
fn the_long_page_with_its_flag_off_is_the_clip_page() {
    for p in [long(), with(|p| p.duration = DurMs::new(60_000))] {
        assert_eq!(
            route_probe(&p, &sdr(), true, Entry::Long, OFF),
            route_probe(&p, &sdr(), true, Entry::Clip, OFF)
        );
        assert_eq!(
            route_probe(&p, &sdr(), true, Entry::Long, HIGH),
            route_probe(&p, &sdr(), true, Entry::Clip, HIGH)
        );
    }
    assert_eq!(
        route_probe(&long(), &sdr(), true, Entry::Long, OFF),
        rejected(RejectReason::Duration)
    );
}

#[test]
fn a_long_source_with_two_audio_tracks_reads_the_first() {
    let p = with(|p| p.audio_tracks = 2);
    let Ok(Route::Long(info)) = route_probe(&p, &sdr(), true, Entry::Long, LONG) else {
        panic!("a long source with two audio tracks is refused");
    };
    assert_eq!(info.audio_track, 0);
    assert_eq!(info.audio_tracks, 2);
    assert_eq!(info.audio_codec_string, "mp4a.40.2");
    assert_eq!(info.duration, DurMs::new(600_000));
    assert!(!info.is_hdr);
}

#[test]
fn exactly_45_minutes_is_long_and_one_ms_more_is_refused() {
    let at = with(|p| p.duration = DurMs::new(2_700_000));
    assert!(is_long(&route_probe(&at, &sdr(), true, Entry::Long, LONG)));
    let over = with(|p| p.duration = DurMs::new(2_700_001));
    assert_eq!(
        route_probe(&over, &sdr(), true, Entry::Long, LONG),
        rejected(RejectReason::Duration)
    );
    let just_long = with(|p| p.duration = DurMs::new(90_001));
    assert!(is_long(&route_probe(
        &just_long,
        &sdr(),
        true,
        Entry::Long,
        LONG
    )));
}

/// V6.1's own fixtures (`fixtures/manifest-long.json`): what V5 says (the clip
/// page, flags off), what the long page says with both flags on, and what the
/// clip page says with `high_res` on.
#[test]
fn the_long_fixtures_route_as_their_manifest_says() {
    let text = std::fs::read_to_string(root().join("fixtures/manifest-long.json")).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&text).unwrap();
    let fixtures = manifest["fixtures"].as_array().unwrap();
    assert_eq!(fixtures.len(), 4);
    let outcome = |r: Result<Route, RouteReject>| -> String {
        match r {
            Ok(Route::Clip(_)) => "clip".to_owned(),
            Ok(Route::Long(_)) => "long".to_owned(),
            Err(reject) => serde_json::to_value(reject)
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned(),
        }
    };
    for f in fixtures {
        let name = f["name"].as_str().unwrap();
        let expect = &f["expect"];
        let (p, detail) = probed(name).unwrap_or_else(|| panic!("{name} does not open"));
        let v5 = outcome(route_probe(&p, &detail, true, Entry::Clip, OFF));
        let long = outcome(route_probe(&p, &detail, true, Entry::Long, BOTH));
        let high = outcome(route_probe(&p, &detail, true, Entry::Clip, HIGH));
        assert_eq!(v5, expect["v5"].as_str().unwrap(), "{name}: v5");
        assert_eq!(long, expect["long"].as_str().unwrap(), "{name}: long");
        assert_eq!(
            high,
            expect["high_res"].as_str().unwrap(),
            "{name}: high_res"
        );
    }
}

#[test]
fn the_hlg_fixture_is_hdr_main_10_and_the_others_are_not() {
    let (_, detail) = probed("rej_hdr_hlg.mov").unwrap();
    assert!(detail.is_hdr);
    let codec = detail.hevc_codec_string.unwrap();
    assert!(codec.starts_with("hvc1.2.4.L"), "{codec}");
    for name in ["long_12min.mp4", "rej_8k.mp4", "rej_too_long_46min.mp4"] {
        assert_eq!(probed(name).unwrap().1, VideoDetail::default(), "{name}");
    }
}

#[test]
fn long_12min_is_a_twelve_minute_long_source() {
    let (p, detail) = probed("long_12min.mp4").unwrap();
    let Ok(Route::Long(info)) = route_probe(&p, &detail, true, Entry::Long, LONG) else {
        panic!("long_12min.mp4 is not long");
    };
    assert!(
        info.duration.get().abs_diff(720_000) <= 50,
        "{:?}",
        info.duration
    );
    assert_eq!(
        (info.display_width.get(), info.display_height.get()),
        (320, 180)
    );
    assert_eq!(info.audio_codec_string, "mp4a.40.2");
    assert_eq!(
        (info.audio_sample_rate.get(), info.audio_channels),
        (48_000, 2)
    );
    assert_eq!(offcut_types::window_count(info.duration), 9);
}
