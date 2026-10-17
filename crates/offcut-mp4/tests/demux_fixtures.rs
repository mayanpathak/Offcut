#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Every generated `ok_*` fixture and every derived speech clip opens,
//! validates and reads as `fixtures/manifest.json` says (v3implementation
//! §19.2), and no derived clip carries the reference clip's metadata (D-100).

use std::path::PathBuf;

use offcut_mp4::boxes::{BoxHeader, children, descend, read_hdlr, read_header, read_tkhd};
use offcut_mp4::demux::Demuxer;
use offcut_mp4::probe::probe;
use offcut_mp4::reader::{MemReader, RandomAccess};
use offcut_mp4::sample_table::{read_stss, read_stsz};
use offcut_mp4::validate::validate_probe;
use offcut_types::{Bytes, ClipInfo, Orientation, ProbeInfo, RejectReason, Rotation, TimeMs};
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

/// One entry of `fixtures/manifest.json` (§17.4).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    name: String,
    kind: String,
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
    accept: Option<Accept>,
    #[serde(rename = "reject")]
    _reject: Option<RejectReason>,
}

/// The fields of a `ClipInfo` a test asserts; `rotation` is `null` until observed.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Accept {
    orientation: Option<Orientation>,
    is_vfr: Option<bool>,
    duration_ms: Option<u32>,
    avg_fps_milli: Option<[u32; 2]>,
    rotation: Option<Rotation>,
    display_width: Option<u32>,
    display_height: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    #[serde(rename = "schema")]
    _schema: u32,
    fixtures: Vec<Entry>,
}

fn manifest() -> Vec<Entry> {
    let text = std::fs::read_to_string(root().join("fixtures/manifest.json")).unwrap();
    serde_json::from_str::<Manifest>(&text).unwrap().fixtures
}

fn accepted() -> Vec<(String, Accept)> {
    manifest()
        .into_iter()
        .filter_map(|e| e.expect.accept.map(|a| (e.name, a)))
        .collect()
}

fn accept_of(name: &str) -> Accept {
    accepted()
        .into_iter()
        .find(|(n, _)| n == name)
        .map(|(_, a)| a)
        .unwrap()
}

fn open(name: &str) -> Demuxer<MemReader> {
    Demuxer::open(MemReader(fixture(name))).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn probed(name: &str) -> (ProbeInfo, ClipInfo) {
    let bytes = fixture(name);
    let size = Bytes::new(bytes.len() as u64);
    let p = probe(
        &Demuxer::open(MemReader(bytes)).unwrap_or_else(|e| panic!("{name}: {e}")),
        size,
    );
    let clip = validate_probe(&p, true).unwrap_or_else(|r| panic!("{name}: {r:?}"));
    (p, clip)
}

#[test]
fn every_accepted_fixture_opens_and_validates_as_the_manifest_says() {
    for (name, a) in accepted() {
        let (p, clip) = probed(&name);
        let avg = p.video.as_ref().unwrap().avg_fps.get();
        println!(
            "{name}: rotation {:?}, {}x{}, {:?}, {} ms, avg_fps {avg}",
            clip.rotation,
            clip.display_width.get(),
            clip.display_height.get(),
            clip.orientation,
            clip.duration.get()
        );
        if let Some(o) = a.orientation {
            assert_eq!(clip.orientation, o, "{name}");
        }
        if let Some(v) = a.is_vfr {
            assert_eq!(clip.is_vfr, v, "{name}");
        }
        if let Some(d) = a.duration_ms {
            assert!(
                clip.duration.get().abs_diff(d) <= 1,
                "{name}: {} ms",
                clip.duration.get()
            );
        }
        if let Some([lo, hi]) = a.avg_fps_milli {
            assert!((lo..=hi).contains(&avg), "{name}: avg_fps {avg}");
        }
        if let Some(w) = a.display_width {
            assert_eq!(clip.display_width.get(), w, "{name}");
        }
        if let Some(h) = a.display_height {
            assert_eq!(clip.display_height.get(), h, "{name}");
        }
    }
}

#[test]
fn ok_vfr_is_variable_with_presentation_times_strictly_increasing() {
    let (p, clip) = probed("ok_vfr.mp4");
    assert!(clip.is_vfr);
    let [lo, hi] = accept_of("ok_vfr.mp4").avg_fps_milli.unwrap();
    assert!((lo..=hi).contains(&p.video.unwrap().avg_fps.get()));
    let mut d = open("ok_vfr.mp4");
    let mut out = Vec::new();
    let mut pts: Vec<i64> = (0..d.video_sample_count())
        .map(|i| d.read_video_sample(i, &mut out).unwrap().pts.get())
        .collect();
    pts.sort_unstable();
    assert!(
        pts.windows(2).all(|w| w[0] < w[1]),
        "two samples share a presentation time"
    );
}

#[test]
fn ok_rotated_90_is_a_recorded_quarter_turn_and_portrait() {
    let (_, clip) = probed("ok_rotated_90.mov");
    println!("ok_rotated_90.mov: rotation {:?}", clip.rotation);
    let recorded = accept_of("ok_rotated_90.mov").rotation;
    assert_eq!(
        Some(clip.rotation),
        recorded,
        "write the observed rotation into fixtures/manifest.json"
    );
    assert!(matches!(clip.rotation, Rotation::R90 | Rotation::R270));
    assert_eq!(
        (clip.display_width.get(), clip.display_height.get()),
        (720, 1280)
    );
    assert_eq!(clip.orientation, Orientation::Portrait);
}

#[test]
fn ok_60fps_averages_60_fps() {
    let avg = probed("ok_60fps.mp4").0.video.unwrap().avg_fps.get();
    assert!(avg.abs_diff(60_000) <= 100, "{avg}");
}

#[test]
fn ok_moov_at_end_opens_with_moov_after_mdat() {
    let mut r = MemReader(fixture("ok_moov_at_end.mp4"));
    let limit = r.len();
    let (mut at, mut moov, mut mdat) = (0u64, None, None);
    while at < limit.get() {
        let h = read_header(&mut r, Bytes::new(at), limit).unwrap();
        match &h.kind {
            b"moov" => moov = Some(at),
            b"mdat" => mdat = Some(at),
            _ => {}
        }
        at = h.end.get();
    }
    assert!(moov.unwrap() > mdat.unwrap());
    probed("ok_moov_at_end.mp4");
}

#[test]
fn ok_portrait_3x4_is_1080_by_1440_portrait() {
    let (_, clip) = probed("ok_portrait_3x4.mp4");
    assert_eq!(
        (clip.display_width.get(), clip.display_height.get()),
        (1080, 1440)
    );
    assert_eq!(clip.orientation, Orientation::Portrait);
}

#[test]
fn the_two_track_fixture_takes_the_enabled_640_by_360_track() {
    let (p, _) = probed("ok_two_video_first_disabled.mp4");
    let v = p.video.unwrap();
    println!(
        "ok_two_video_first_disabled.mp4: video track {}x{}",
        v.coded_width.get(),
        v.coded_height.get()
    );
    assert_eq!((v.coded_width.get(), v.coded_height.get()), (640, 360));
}

#[test]
fn speech_60s_portrait_is_60_s_with_1800_frames_and_aac_lc_stereo() {
    let (p, clip) = probed("speech/speech_60s_portrait.mp4");
    assert!(
        clip.duration.get().abs_diff(60_000) <= 1,
        "{}",
        clip.duration.get()
    );
    assert_eq!(p.video.unwrap().frame_count, 1_800);
    assert_eq!(
        (
            clip.audio_codec_string.as_str(),
            clip.audio_sample_rate.get(),
            clip.audio_channels
        ),
        ("mp4a.40.2", 48_000, 2)
    );
}

#[test]
fn speech_20s_long_pause_is_20_s() {
    let (_, clip) = probed("speech/speech_20s_long_pause.mp4");
    assert!(
        clip.duration.get().abs_diff(20_000) <= 1,
        "{}",
        clip.duration.get()
    );
}

/// The `stbl` of the track the demuxer reads: the first enabled video track.
fn video_stbl(r: &mut MemReader) -> BoxHeader {
    let limit = r.len();
    let mut at = 0u64;
    let moov = loop {
        let h = read_header(r, Bytes::new(at), limit).unwrap();
        if &h.kind == b"moov" {
            break h;
        }
        at = h.end.get();
    };
    for trak in children(r, &moov)
        .unwrap()
        .into_iter()
        .filter(|h| &h.kind == b"trak")
    {
        let hdlr = descend(r, &trak, &[*b"mdia", *b"hdlr"]).unwrap().unwrap();
        let tkhd = descend(r, &trak, &[*b"tkhd"]).unwrap().unwrap();
        if &read_hdlr(r, &hdlr).unwrap() == b"vide" && read_tkhd(r, &tkhd).unwrap().flags & 1 == 1 {
            return descend(r, &trak, &[*b"mdia", *b"minf", *b"stbl"])
                .unwrap()
                .unwrap();
        }
    }
    panic!("no enabled video track");
}

#[test]
fn every_video_sample_reads_whole_with_the_keyframes_of_stss() {
    let ok: Vec<String> = accepted()
        .into_iter()
        .map(|(n, _)| n)
        .filter(|n| n.starts_with("ok_"))
        .collect();
    for name in ok {
        let mut r = MemReader(fixture(&name));
        let stbl = video_stbl(&mut r);
        let stsz_box = descend(&mut r, &stbl, &[*b"stsz"]).unwrap().unwrap();
        let stsz = read_stsz(&mut r, &stsz_box, u32::MAX).unwrap();
        let stss_box = descend(&mut r, &stbl, &[*b"stss"]).unwrap();
        let sync = stss_box.map(|h| read_stss(&mut r, &h).unwrap());
        let mut d = open(&name);
        let mut out = Vec::new();
        for i in 0..d.video_sample_count() {
            let meta = d.read_video_sample(i, &mut out).unwrap();
            let size = if stsz.sample_size == 0 {
                stsz.sizes[i as usize]
            } else {
                stsz.sample_size
            };
            assert_eq!(out.len() as u32, size, "{name} sample {i}");
            let key = sync.as_ref().is_none_or(|s| s.contains(&(i + 1)));
            assert_eq!(meta.is_keyframe, key, "{name} sample {i}");
        }
    }
}

#[test]
fn keyframes_of_ok_60fps_are_every_500_ms() {
    let d = open("ok_60fps.mp4");
    let found: Vec<u32> = [0, 499, 500, 2_999]
        .iter()
        .map(|&t| d.keyframe_at_or_before(TimeMs::new(t)))
        .collect();
    assert_eq!(found, [0, 0, 30, 150]);
}

/// What D-100 forbids in a derived clip: a location or date atom of `udta`,
/// and the location or creation-date key of a QuickTime `meta`.
const FORBIDDEN: [&[u8]; 4] = [
    b"\xA9xyz",
    b"\xA9day",
    b"com.apple.quicktime.location.ISO6709",
    b"creationdate",
];
const CONTAINERS: [&[u8; 4]; 9] = [
    b"moov", b"trak", b"mdia", b"minf", b"stbl", b"udta", b"edts", b"dinf", b"ilst",
];

/// Walks `parent`'s children; every box that is not a container (and not
/// `mdat`, whose compressed bytes say nothing) is searched for FORBIDDEN.
fn walk(r: &mut MemReader, boxes: Vec<BoxHeader>, depth: u32, found: &mut Vec<String>) {
    assert!(depth < 12, "boxes nested too deep");
    for h in boxes {
        if FORBIDDEN.iter().any(|f| f.len() == 4 && h.kind == f[..4]) {
            found.push(String::from_utf8_lossy(&h.kind).into_owned());
        }
        if &h.kind == b"mdat" {
            continue;
        }
        if CONTAINERS.contains(&&h.kind) {
            let inner = children(r, &h).unwrap();
            walk(r, inner, depth + 1, found);
        } else if &h.kind == b"meta" {
            // A full box in MP4 (4 bytes of version and flags), a plain one in QuickTime.
            let full = BoxHeader {
                body: Bytes::new(h.body.get() + 4),
                ..h
            };
            let inner = children(r, &full)
                .or_else(|_| children(r, &h))
                .unwrap_or_default();
            walk(r, inner, depth + 1, found);
        } else {
            let mut body = vec![0u8; h.body_len() as usize];
            r.read_at(h.body, &mut body).unwrap();
            for f in FORBIDDEN {
                if body.windows(f.len()).any(|w| w == f) {
                    found.push(format!(
                        "{} in {}",
                        String::from_utf8_lossy(f),
                        String::from_utf8_lossy(&h.kind)
                    ));
                }
            }
        }
    }
}

#[test]
fn no_derived_clip_carries_the_reference_clips_metadata() {
    let derived: Vec<String> = manifest()
        .into_iter()
        .filter(|e| e.kind == "derived")
        .map(|e| e.name)
        .collect();
    assert_eq!(derived.len(), 5);
    for name in derived {
        let mut r = MemReader(fixture(&name));
        let limit = r.len();
        let (mut top, mut at) = (Vec::new(), 0u64);
        while at < limit.get() {
            let h = read_header(&mut r, Bytes::new(at), limit).unwrap();
            at = h.end.get();
            top.push(h);
        }
        let mut found = Vec::new();
        walk(&mut r, top, 0, &mut found);
        assert!(found.is_empty(), "{name}: {found:?}");
    }
}
