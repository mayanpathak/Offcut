#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Every generated `rej_*` fixture is refused with its code (PS §9.4, TS §26;
//! v3implementation §19.3), except `rej_silent.mp4`, which the recognizer
//! decides. The browser binding's own mapping is checked by `rejections.spec.ts`.

use std::path::PathBuf;

use offcut_mp4::ContainerError;
use offcut_mp4::demux::Demuxer;
use offcut_mp4::probe::probe;
use offcut_mp4::reader::MemReader;
use offcut_mp4::validate::validate_probe;
use offcut_types::{Bytes, ClipInfo, RejectReason};
use serde::Deserialize;
use sha2::{Digest, Sha256};

const STALE: &str = "the generated fixtures are stale or missing: run `pnpm fixtures`";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A generated fixture's bytes, after checking that `fixtures/.gen/` was built
/// from the manifest of this checkout (v3implementation §6.4).
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

/// One entry of `fixtures/manifest.json` (§17.4); only the reject code is read.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    name: String,
    #[serde(rename = "kind")]
    _kind: String,
    expect: Expect,
    #[serde(rename = "probe")]
    _probe: Option<serde_json::Value>,
    #[serde(rename = "pcm")]
    _pcm: Option<bool>,
    #[serde(rename = "tests")]
    _tests: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Expect {
    #[serde(rename = "accept")]
    _accept: Option<serde_json::Value>,
    reject: Option<RejectReason>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    #[serde(rename = "schema")]
    _schema: u32,
    fixtures: Vec<Entry>,
}

/// What `media_api.rs` (V2, frozen) does with a file the demuxer cannot open:
/// no ISO base media file, or a fragmented one, is the wrong container; one
/// that is cut short, malformed, or of a structure this version does not
/// read is corrupt. A failure to read is no rejection at all.
fn rejection_of(error: ContainerError) -> RejectReason {
    match error {
        ContainerError::NotIsoBmff | ContainerError::Fragmented => RejectReason::Container,
        ContainerError::Truncated
        | ContainerError::Malformed(_)
        | ContainerError::Unsupported(_) => RejectReason::Corrupt,
        ContainerError::Io(io) => panic!("reading the fixture failed: {io:?}"),
    }
}

/// The outcome the app would reach before the recognizer: open, probe, validate.
fn outcome(name: &str, decode_supported: bool) -> Result<ClipInfo, RejectReason> {
    let bytes = fixture(name);
    let size = Bytes::new(bytes.len() as u64);
    let d = Demuxer::open(MemReader(bytes)).map_err(rejection_of)?;
    validate_probe(&probe(&d, size), decode_supported)
}

/// Every `rej_*` fixture the recognizer does not decide, with the
/// `decode_supported` the test gives it and the code it must give.
const CASES: [(&str, bool, RejectReason); 12] = [
    ("rej_vp9.webm", true, RejectReason::Container),
    ("rej_truncated.mp4", true, RejectReason::Corrupt),
    ("rej_no_video.m4a", true, RejectReason::NoVideo),
    ("rej_hevc.mov", true, RejectReason::Hevc),
    ("rej_av1.mp4", true, RejectReason::VideoCodec),
    ("rej_no_audio.mp4", true, RejectReason::NoAudio),
    ("rej_two_audio.mp4", true, RejectReason::MultiAudioTrack),
    ("rej_mp3_audio.mp4", true, RejectReason::AudioCodec),
    ("rej_too_long_91s.mp4", true, RejectReason::Duration),
    ("rej_4k.mp4", true, RejectReason::Resolution),
    ("rej_120fps.mp4", true, RejectReason::FrameRate),
    ("rej_h264_444.mp4", false, RejectReason::DecodeUnsupported),
];

fn refused(name: &str, expected: RejectReason) {
    let (_, decode_supported, _) = CASES.iter().find(|(n, ..)| *n == name).unwrap();
    assert_eq!(outcome(name, *decode_supported), Err(expected), "{name}");
}

#[test]
fn vp9_in_webm_is_the_wrong_container() {
    assert_eq!(
        Demuxer::open(MemReader(fixture("rej_vp9.webm"))).err(),
        Some(ContainerError::NotIsoBmff)
    );
    refused("rej_vp9.webm", RejectReason::Container);
}

#[test]
fn a_truncated_file_is_corrupt() {
    let opened = Demuxer::open(MemReader(fixture("rej_truncated.mp4"))).err();
    assert!(
        matches!(
            opened,
            Some(ContainerError::Truncated | ContainerError::Malformed(_))
        ),
        "{opened:?}"
    );
    refused("rej_truncated.mp4", RejectReason::Corrupt);
}

#[test]
fn audio_alone_has_no_video() {
    refused("rej_no_video.m4a", RejectReason::NoVideo);
}

#[test]
fn hevc_is_refused_as_hevc() {
    refused("rej_hevc.mov", RejectReason::Hevc);
}

#[test]
fn av1_is_the_wrong_video_codec() {
    refused("rej_av1.mp4", RejectReason::VideoCodec);
}

#[test]
fn video_alone_has_no_audio() {
    refused("rej_no_audio.mp4", RejectReason::NoAudio);
}

#[test]
fn two_audio_tracks_are_refused() {
    refused("rej_two_audio.mp4", RejectReason::MultiAudioTrack);
}

#[test]
fn mp3_audio_is_the_wrong_audio_codec() {
    refused("rej_mp3_audio.mp4", RejectReason::AudioCodec);
}

#[test]
fn ninety_one_seconds_is_too_long() {
    refused("rej_too_long_91s.mp4", RejectReason::Duration);
}

#[test]
fn four_k_is_too_large() {
    refused("rej_4k.mp4", RejectReason::Resolution);
}

#[test]
fn a_hundred_and_twenty_fps_is_too_fast() {
    refused("rej_120fps.mp4", RejectReason::FrameRate);
}

#[test]
fn high_444_passes_every_rule_when_it_can_be_decoded() {
    assert!(outcome("rej_h264_444.mp4", true).is_ok());
    refused("rej_h264_444.mp4", RejectReason::DecodeUnsupported);
}

#[test]
fn high_444_is_refused_when_the_browser_cannot_decode_it() {
    assert_eq!(
        outcome("rej_h264_444.mp4", false),
        Err(RejectReason::DecodeUnsupported)
    );
}

#[test]
fn exactly_ninety_seconds_is_inside_the_limit() {
    let clip = outcome("ok_90s_exact.mp4", true).unwrap();
    assert_eq!(clip.duration.get(), 90_000);
}

#[test]
fn every_rejection_of_the_manifest_has_a_case() {
    let text = std::fs::read_to_string(root().join("fixtures/manifest.json")).unwrap();
    let manifest: Manifest = serde_json::from_str(&text).unwrap();
    for entry in manifest
        .fixtures
        .into_iter()
        .filter(|e| e.name.starts_with("rej_") && e.name != "rej_silent.mp4")
    {
        let case = CASES.iter().find(|(n, ..)| *n == entry.name);
        let Some((_, decode_supported, expected)) = case else {
            panic!("{} has no case in probe_rejections.rs", entry.name);
        };
        assert_eq!(
            entry.expect.reject,
            Some(*expected),
            "{}: the manifest and the case disagree",
            entry.name
        );
        assert_eq!(
            outcome(&entry.name, *decode_supported),
            Err(*expected),
            "{}",
            entry.name
        );
    }
}
