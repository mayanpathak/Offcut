#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Real B-frames through the muxer (v5implementation D-212, §9.3): the samples
//! of `ok_bframes.mp4`, which libx264 encoded out of the order they are shown,
//! written in decode order, then read by the tools the muxer shares nothing
//! with. Random bytes would prove the tables only. A missing `ffprobe` or
//! `ffmpeg` fails: CI and the development machine both have them.

use std::path::PathBuf;
use std::process::Command;

use offcut_mp4::demux::Demuxer;
use offcut_mp4::mux::{AudioTrackSpec, MemSink, Mp4Muxer, VideoTrackSpec};
use offcut_mp4::reader::MemReader;
use offcut_types::{FrameIdx, Hz, Micros, Px};
use sha2::{Digest, Sha256};

const STALE: &str = "the generated fixtures are stale or missing: run `pnpm fixtures`";
const FIXTURE: &str = "ok_bframes.mp4";
/// 2 s at 30 fps.
const FRAMES: usize = 60;
/// One frame in ticks of 1/30,000 s, the muxer's video timescale.
const FRAME_TICKS: i64 = 1_000;

/// A generated fixture's bytes, after checking that `fixtures/.gen/` was built
/// from the manifest of this checkout (v3implementation §6.4).
fn fixture(name: &str) -> Vec<u8> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = std::env::var_os("OFFCUT_FIXTURES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("fixtures/.gen"));
    let stamp = std::fs::read_to_string(dir.join("STAMP")).expect(STALE);
    let manifest = std::fs::read(root.join("fixtures/manifest.json")).expect(STALE);
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

/// A file of the test's, removed however the test ends.
struct TempFile(PathBuf);

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

struct Remuxed {
    file: Vec<u8>,
    /// Each sample's output frame, in decode order.
    shown: Vec<u32>,
    samples: Vec<Vec<u8>>,
}

/// The fixture's samples through `Mp4Muxer`, in the order they are decoded.
fn remux() -> Remuxed {
    let mut d = Demuxer::open(MemReader(fixture(FIXTURE))).unwrap();
    let count = d.video_sample_count();
    let mut out = Vec::new();

    let mut video = Vec::new();
    let mut times = Vec::new();
    for n in 0..count {
        let meta = d.read_video_sample(n, &mut out).unwrap();
        video.push((out.clone(), meta.is_keyframe));
        times.push(meta.pts);
    }
    // A frame's number is its place among the times it is shown at.
    let mut sorted = times.clone();
    sorted.sort_unstable();
    let shown: Vec<u32> = times
        .iter()
        .map(|time| sorted.binary_search(time).unwrap() as u32)
        .collect();

    let mut audio: Vec<(Vec<u8>, Micros, Micros)> = Vec::new();
    for i in 0..d.audio_sample_count() {
        let meta = d.read_audio_sample(i, &mut out).unwrap();
        audio.push((out.clone(), meta.pts, meta.duration));
    }

    let spec = VideoTrackSpec {
        width: Px::new(1280),
        height: Px::new(720),
        avcc: d.video_decoder_description().unwrap().to_vec(),
        frame_count_hint: count,
    };
    let audio_spec = AudioTrackSpec {
        sample_rate: Hz::new(48_000),
        channels: 2,
        asc: d.audio_decoder_description().unwrap().to_vec(),
    };
    let mut sink = MemSink::default();
    let mut muxer = Mp4Muxer::new(&mut sink, spec, audio_spec).unwrap();
    // In turns, as an export writes them.
    let mut next_audio = audio.iter();
    let per_frame = audio.len().div_ceil(video.len().max(1));
    for ((data, keyframe), &frame) in video.iter().zip(&shown) {
        muxer
            .add_video_sample(data, FrameIdx::new(frame), *keyframe)
            .unwrap();
        for (bytes, pts, duration) in next_audio.by_ref().take(per_frame) {
            muxer.add_audio_sample(bytes, *pts, *duration).unwrap();
        }
    }
    for (bytes, pts, duration) in next_audio {
        muxer.add_audio_sample(bytes, *pts, *duration).unwrap();
    }
    muxer.finalize().unwrap();
    Remuxed {
        file: sink.0,
        shown,
        samples: video.into_iter().map(|(data, _)| data).collect(),
    }
}

/// Runs a tool on a copy of `file` in the system's temporary directory.
fn tool(name: &str, file: &[u8], args: impl Fn(&str) -> Vec<String>) -> (String, String) {
    let path = std::env::temp_dir().join(format!(
        "offcut-mux-reorder-{name}-{}.mp4",
        std::process::id()
    ));
    let temp = TempFile(path);
    std::fs::write(&temp.0, file).unwrap();
    let done = Command::new(name)
        .args(args(temp.0.to_str().unwrap()))
        .output()
        .unwrap_or_else(|error| panic!("{name} could not be run: {error}"));
    assert!(done.status.success(), "{name} failed");
    (
        String::from_utf8_lossy(&done.stdout).into_owned(),
        String::from_utf8_lossy(&done.stderr).into_owned(),
    )
}

#[test]
fn real_b_frames_come_back_at_30_fps() {
    let remuxed = remux();
    // The fixture does what it is for: some sample comes before a frame shown ahead of it.
    assert_eq!(remuxed.shown.len(), FRAMES);
    assert!(
        remuxed.shown.windows(2).any(|pair| pair[1] < pair[0]),
        "the fixture has no reordered frames"
    );

    // The crate's own demuxer reads back the same times and bytes.
    let mut d = Demuxer::open(MemReader(remuxed.file.clone())).unwrap();
    let mut out = Vec::new();
    for (n, &frame) in (0u32..).zip(&remuxed.shown) {
        let meta = d.read_video_sample(n, &mut out).unwrap();
        assert_eq!(meta.pts, FrameIdx::new(frame).to_micros(), "sample {n}");
        assert_eq!(&out, &remuxed.samples[n as usize], "sample {n}");
    }

    // ffprobe, which shares nothing with the muxer: every frame 1/30 s after the one before it.
    let (json, _) = tool("ffprobe", &remuxed.file, |path| {
        [
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=time_base:frame=pts",
            "-of",
            "json",
            path,
        ]
        .map(String::from)
        .to_vec()
    });
    let info: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(info["streams"][0]["time_base"], "1/30000");
    let stamps: Vec<i64> = info["frames"]
        .as_array()
        .unwrap()
        .iter()
        .map(|frame| frame["pts"].as_i64().unwrap())
        .collect();
    assert_eq!(stamps.len(), FRAMES);
    assert_eq!(stamps[0], 0, "the first frame is shown at 0");
    for (index, pair) in stamps.windows(2).enumerate() {
        assert_eq!(pair[1] - pair[0], FRAME_TICKS, "after frame {index}");
    }

    // ffmpeg decodes the whole file with no error.
    let (_, errors) = tool("ffmpeg", &remuxed.file, |path| {
        ["-v", "error", "-nostdin", "-i", path, "-f", "null", "-"]
            .map(String::from)
            .to_vec()
    });
    assert_eq!(errors.trim(), "", "ffmpeg reported errors");
}
