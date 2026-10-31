#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! What the muxer writes, read back by the demuxer of the same crate: every
//! sample byte for byte, with its time and its keyframe flag
//! (v2implementation §23.2). The muxer and the demuxer never look inside a
//! sample, so the payloads are pseudo-random bytes from a fixed seed.

use offcut_mp4::MuxError;
use offcut_mp4::boxes::{self, BoxHeader};
use offcut_mp4::demux::Demuxer;
use offcut_mp4::mux::{AudioTrackSpec, MOOV_RESERVE, MemSink, Mp4Muxer, VideoTrackSpec};
use offcut_mp4::probe::probe;
use offcut_mp4::reader::MemReader;
use offcut_mp4::sample_table::{read_ctts, read_elst};
use offcut_types::{
    AudioCodec, Bytes, ContainerKind, FpsMilli, FrameIdx, Hz, Micros, Px, VideoCodec,
};
use proptest::prelude::*;
use sha2::{Digest, Sha256};

/// H.264 High at level 4.0. The muxer only needs it not to be empty.
const AVCC: [u8; 8] = [0x01, 0x64, 0x00, 0x28, 0xFF, 0xE1, 0x00, 0x00];
/// AAC-LC, 48 kHz, two channels.
const ASC: [u8; 2] = [0x11, 0x90];
/// One AAC frame: 1,024 samples at 48 kHz, in whole microseconds.
const AAC_FRAME: Micros = Micros::new(21_333);

const IDENTITY: [i32; 9] = [0x1_0000, 0, 0, 0, 0x1_0000, 0, 0, 0, 0x4000_0000];

fn video_spec() -> VideoTrackSpec {
    VideoTrackSpec {
        width: Px::new(1080),
        height: Px::new(1920),
        avcc: AVCC.to_vec(),
        frame_count_hint: 90,
    }
}

fn audio_spec() -> AudioTrackSpec {
    AudioTrackSpec {
        sample_rate: Hz::new(48_000),
        channels: 2,
        asc: ASC.to_vec(),
    }
}

/// Bytes that look random and are the same on every run.
struct Seeded(u64);

impl Seeded {
    fn bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len)
            .map(|_| {
                self.0 ^= self.0 << 13;
                self.0 ^= self.0 >> 7;
                self.0 ^= self.0 << 17;
                (self.0 >> 24) as u8
            })
            .collect()
    }
}

/// Where audio sample `i` starts when every sample before it is one AAC
/// frame and the first one starts at `first`.
fn audio_pts(first: Micros, i: u32) -> Micros {
    Micros::new(first.get() + i64::from(i) * 1_024 * 1_000_000 / 48_000)
}

struct Written {
    file: Vec<u8>,
    returned: Bytes,
    video: Vec<(Vec<u8>, bool)>,
    audio: Vec<Vec<u8>>,
}

/// Muxes `video` (payload size and keyframe flag per frame) and `audio`
/// (payload size per sample). The two tracks are written in turns, so that
/// their chunks lie between each other in the file.
fn mux(video: &[(usize, bool)], audio: &[usize], first_audio_pts: Micros) -> Written {
    let mut seeded = Seeded(0x9E37_79B9_7F4A_7C15);
    let mut sink = MemSink::default();
    let mut muxer = Mp4Muxer::new(&mut sink, video_spec(), audio_spec()).unwrap();
    let mut written = Written {
        file: Vec::new(),
        returned: Bytes::new(0),
        video: Vec::new(),
        audio: Vec::new(),
    };
    let (mut v, mut a) = (0u32, 0u32);
    while (v as usize) < video.len() || (a as usize) < audio.len() {
        // Video is a little ahead of audio, as an encoder's output is.
        let video_next = (a as usize) >= audio.len()
            || ((v as usize) < video.len() && u64::from(v) * 1_600 <= u64::from(a) * 1_024);
        if video_next {
            let (size, keyframe) = video[v as usize];
            let data = seeded.bytes(size);
            muxer
                .add_video_sample(&data, FrameIdx::new(v), keyframe)
                .unwrap();
            written.video.push((data, keyframe));
            v += 1;
        } else {
            let data = seeded.bytes(audio[a as usize]);
            muxer
                .add_audio_sample(&data, audio_pts(first_audio_pts, a), AAC_FRAME)
                .unwrap();
            written.audio.push(data);
            a += 1;
        }
    }
    written.returned = muxer.finalize().unwrap();
    written.file = sink.0;
    written
}

/// 90 frames with a keyframe every 60, and 141 audio samples: 3 s of each.
fn three_seconds(first_audio_pts: Micros) -> Written {
    let video: Vec<(usize, bool)> = (0..90).map(|n| (500 + n * 7, n % 60 == 0)).collect();
    let audio: Vec<usize> = (0..141).map(|i| 200 + i * 3).collect();
    mux(&video, &audio, first_audio_pts)
}

/// Reads every sample back and compares it with what was written.
fn assert_round_trip(w: &Written, first_audio_pts: Micros) {
    let mut d = Demuxer::open(MemReader(w.file.clone())).unwrap();
    assert_eq!(d.video_sample_count() as usize, w.video.len());
    assert_eq!(d.audio_sample_count() as usize, w.audio.len());
    assert_eq!(d.video_decoder_description(), Some(&AVCC[..]));
    assert_eq!(d.audio_decoder_description(), Some(&ASC[..]));

    let mut out = Vec::new();
    for (n, (data, keyframe)) in (0u32..).zip(&w.video) {
        let meta = d.read_video_sample(n, &mut out).unwrap();
        assert_eq!(&out, data, "the bytes of video sample {n}");
        assert_eq!(meta.pts, FrameIdx::new(n).to_micros(), "video sample {n}");
        assert_eq!(meta.dts, meta.pts);
        assert_eq!(meta.is_keyframe, *keyframe, "video sample {n}");
    }
    // The priming length, to the nearest tick of 1/48,000 s.
    let priming = (-first_audio_pts.get() * 48_000 + 500_000).div_euclid(1_000_000);
    for (i, data) in (0u32..).zip(&w.audio) {
        let meta = d.read_audio_sample(i, &mut out).unwrap();
        assert_eq!(&out, data, "the bytes of audio sample {i}");
        assert_eq!(meta.duration, AAC_FRAME, "audio sample {i}");
        let ticks = i64::from(i) * 1_024 - priming;
        assert_eq!(
            meta.pts.get(),
            (ticks * 1_000_000).div_euclid(48_000),
            "audio sample {i}"
        );
        assert!(meta.is_keyframe);
    }
}

fn top_level(file: &[u8]) -> Vec<BoxHeader> {
    let mut reader = MemReader(file.to_vec());
    let len = Bytes::new(file.len() as u64);
    let mut found = Vec::new();
    let mut at = 0u64;
    while at < len.get() {
        let header = boxes::read_header(&mut reader, Bytes::new(at), len).unwrap();
        at = header.end.get();
        found.push(header);
    }
    found
}

fn find(headers: &[BoxHeader], kind: &[u8; 4]) -> BoxHeader {
    *headers.iter().find(|h| &h.kind == kind).unwrap()
}

/// The two `trak` boxes: video, then audio.
fn traks(file: &[u8]) -> (MemReader, BoxHeader, BoxHeader) {
    let mut reader = MemReader(file.to_vec());
    let moov = find(&top_level(file), b"moov");
    let in_moov = boxes::children(&mut reader, &moov).unwrap();
    let traks: Vec<BoxHeader> = in_moov.into_iter().filter(|h| &h.kind == b"trak").collect();
    assert_eq!(traks.len(), 2);
    (reader, traks[0], traks[1])
}

/// The type of every box in the file, at every depth.
fn all_kinds(file: &[u8]) -> Vec<[u8; 4]> {
    const CONTAINERS: [&[u8; 4]; 7] = [
        b"moov", b"trak", b"edts", b"mdia", b"minf", b"dinf", b"stbl",
    ];
    let mut reader = MemReader(file.to_vec());
    let mut pending = top_level(file);
    let mut kinds = Vec::new();
    while let Some(header) = pending.pop() {
        kinds.push(header.kind);
        if CONTAINERS.contains(&&header.kind) {
            pending.extend(boxes::children(&mut reader, &header).unwrap());
        }
    }
    kinds
}

#[test]
fn ninety_frames_and_141_audio_samples_come_back_byte_for_byte() {
    let w = three_seconds(Micros::new(0));
    assert_round_trip(&w, Micros::new(0));

    let d = Demuxer::open(MemReader(w.file.clone())).unwrap();
    // A frame every 1/30 s, floored to microseconds; a keyframe every 60.
    assert_eq!(
        d.keyframe_at_or_before(offcut_types::TimeMs::new(2_500)),
        60
    );
    assert_eq!(FrameIdx::new(89).to_micros().get(), 2_966_666);
}

#[test]
fn the_file_probes_as_h264_at_30_fps_with_aac_at_48_khz() {
    let w = three_seconds(Micros::new(0));
    let d = Demuxer::open(MemReader(w.file.clone())).unwrap();
    let p = probe(&d, w.returned);

    assert_eq!(p.container, ContainerKind::Mp4);
    assert_eq!((p.video_tracks, p.audio_tracks), (1, 1));
    assert_eq!(p.duration.get(), 3_000);
    let video = p.video.unwrap();
    assert_eq!(video.codec, VideoCodec::H264);
    assert_eq!(video.codec_string, "avc1.640028");
    assert_eq!(
        (video.coded_width.get(), video.coded_height.get()),
        (1080, 1920)
    );
    assert_eq!(video.frame_count, 90);
    assert_eq!(video.avg_fps, FpsMilli::new(30_000));
    assert_eq!(video.max_fps, FpsMilli::new(30_000));
    assert!(!video.is_vfr);
    let audio = p.audio.unwrap();
    assert_eq!(audio.codec, AudioCodec::Aac);
    assert_eq!(audio.codec_string, "mp4a.40.2");
    assert_eq!((audio.sample_rate.get(), audio.channels), (48_000, 2));
}

#[test]
fn the_boxes_are_ftyp_then_moov_then_mdat() {
    let w = three_seconds(Micros::new(0));
    let order: Vec<[u8; 4]> = top_level(&w.file).iter().map(|h| h.kind).collect();
    // What is left of the space kept for `moov` is a `free` box.
    assert_eq!(order, [*b"ftyp", *b"moov", *b"free", *b"mdat"]);
}

#[test]
fn finalize_returns_the_length_of_the_file() {
    let w = three_seconds(Micros::new(0));
    assert_eq!(w.returned.get(), w.file.len() as u64);
    // `mdat` runs to the end of the file, and says so in its 64-bit size.
    let mdat = find(&top_level(&w.file), b"mdat");
    assert_eq!(mdat.end.get(), w.file.len() as u64);
    assert_eq!(mdat.body.get() - mdat.start.get(), 16);
}

#[test]
fn a_first_audio_sample_before_0_becomes_an_edit_of_its_length() {
    let first = Micros::new(-21_333);
    let w = three_seconds(first);
    let (mut reader, _, audio) = traks(&w.file);
    let elst = boxes::descend(&mut reader, &audio, &[*b"edts", *b"elst"])
        .unwrap()
        .unwrap();
    let edits = read_elst(&mut reader, &elst).unwrap();
    assert_eq!(edits.len(), 1);
    // 21,333 microseconds are 1,023.98 ticks of 1/48,000 s: 1,024.
    assert_eq!(edits[0].media_time, 1_024);
    // What is shown: 140 of the 141 frames, in milliseconds.
    assert_eq!(edits[0].segment_duration, 2_987);

    let mut d = Demuxer::open(MemReader(w.file.clone())).unwrap();
    let mut out = Vec::new();
    // 1,024 ticks before 0 are 21,333.3 microseconds, and the demuxer
    // rounds a time down.
    assert_eq!(d.read_audio_sample(0, &mut out).unwrap().pts.get(), -21_334);
    assert_eq!(d.read_audio_sample(1, &mut out).unwrap().pts.get(), 0);
    assert_round_trip(&w, first);
}

#[test]
fn a_first_audio_sample_at_0_writes_no_edit() {
    let w = three_seconds(Micros::new(0));
    let (mut reader, video, audio) = traks(&w.file);
    assert_eq!(boxes::child(&mut reader, &audio, b"edts").unwrap(), None);
    assert_eq!(boxes::child(&mut reader, &video, b"edts").unwrap(), None);
}

#[test]
fn ninety_seconds_fit_the_space_kept_for_moov() {
    let video: Vec<(usize, bool)> = (0..2_700).map(|n| (3_000, n % 60 == 0)).collect();
    let audio = vec![400usize; 4_220];
    let w = mux(&video, &audio, Micros::new(0));

    let top = top_level(&w.file);
    let (moov, mdat) = (find(&top, b"moov"), find(&top, b"mdat"));
    assert!(moov.end.get() <= mdat.start.get());
    // Written in turns, most samples are a chunk of their own, which is the
    // largest `moov` a clip of this length can have.
    assert!(moov.end.get() - moov.start.get() < MOOV_RESERVE);
    assert_eq!(mdat.start.get() - moov.start.get(), MOOV_RESERVE);

    let d = Demuxer::open(MemReader(w.file.clone())).unwrap();
    assert_eq!(probe(&d, w.returned).duration.get(), 90_000);
    assert_eq!(
        (d.video_sample_count(), d.audio_sample_count()),
        (2_700, 4_220)
    );
    assert_round_trip(&w, Micros::new(0));
}

#[test]
fn a_frame_out_of_its_turn_is_out_of_order() {
    let mut muxer = Mp4Muxer::new(MemSink::default(), video_spec(), audio_spec()).unwrap();
    assert_eq!(
        muxer.add_video_sample(&[1], FrameIdx::new(1), true),
        Err(MuxError::OutOfOrder)
    );
    muxer
        .add_video_sample(&[1], FrameIdx::new(0), true)
        .unwrap();
    assert_eq!(
        muxer.add_video_sample(&[1], FrameIdx::new(0), true),
        Err(MuxError::OutOfOrder)
    );

    // Audio may not go back in time either.
    muxer
        .add_audio_sample(&[1], Micros::new(21_333), AAC_FRAME)
        .unwrap();
    assert_eq!(
        muxer.add_audio_sample(&[1], Micros::new(0), AAC_FRAME),
        Err(MuxError::OutOfOrder)
    );
}

#[test]
fn a_first_frame_that_is_not_a_keyframe_is_a_bad_configuration() {
    let mut muxer = Mp4Muxer::new(MemSink::default(), video_spec(), audio_spec()).unwrap();
    assert!(matches!(
        muxer.add_video_sample(&[1], FrameIdx::new(0), false),
        Err(MuxError::BadConfig(_))
    ));
    // Nothing was recorded: the frame can still be given as a keyframe.
    muxer
        .add_video_sample(&[1], FrameIdx::new(0), true)
        .unwrap();
    muxer
        .add_video_sample(&[1], FrameIdx::new(1), false)
        .unwrap();
}

#[test]
fn an_empty_avcc_another_sample_rate_or_one_channel_is_a_bad_configuration() {
    let refused = |video: VideoTrackSpec, audio: AudioTrackSpec| {
        let mut sink = MemSink::default();
        let result = Mp4Muxer::new(&mut sink, video, audio).map(|_| ());
        // A refused configuration writes nothing.
        assert!(sink.0.is_empty());
        matches!(result, Err(MuxError::BadConfig(_)))
    };
    assert!(refused(
        VideoTrackSpec {
            avcc: Vec::new(),
            ..video_spec()
        },
        audio_spec()
    ));
    assert!(refused(
        video_spec(),
        AudioTrackSpec {
            sample_rate: Hz::new(44_100),
            ..audio_spec()
        }
    ));
    assert!(refused(
        video_spec(),
        AudioTrackSpec {
            channels: 1,
            ..audio_spec()
        }
    ));
    assert!(refused(
        video_spec(),
        AudioTrackSpec {
            asc: Vec::new(),
            ..audio_spec()
        }
    ));
    assert!(refused(
        VideoTrackSpec {
            width: Px::new(0),
            ..video_spec()
        },
        audio_spec()
    ));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn any_frames_with_any_sizes_and_keyframes_come_back(
        frames in prop::collection::vec((1usize..=2_000, any::<bool>()), 1..=300),
        audio in prop::collection::vec(1usize..=600, 0..=40),
    ) {
        let mut frames = frames;
        frames[0].1 = true;
        let w = mux(&frames, &audio, Micros::new(0));
        prop_assert_eq!(w.returned.get(), w.file.len() as u64);
        assert_round_trip(&w, Micros::new(0));
    }
}

#[test]
fn the_video_track_has_its_size_no_rotation_and_no_user_data() {
    let w = three_seconds(Micros::new(-21_333));
    let (mut reader, video, audio) = traks(&w.file);
    let tkhd = boxes::child(&mut reader, &video, b"tkhd").unwrap().unwrap();
    let tkhd = boxes::read_tkhd(&mut reader, &tkhd).unwrap();
    assert_eq!((tkhd.width >> 16, tkhd.height >> 16), (1080, 1920));
    assert_eq!(tkhd.matrix, IDENTITY);
    assert_eq!(tkhd.flags & 1, 1);

    let audio_tkhd = boxes::child(&mut reader, &audio, b"tkhd").unwrap().unwrap();
    assert_eq!(
        boxes::read_tkhd(&mut reader, &audio_tkhd).unwrap().matrix,
        IDENTITY
    );

    let kinds = all_kinds(&w.file);
    assert!(kinds.contains(b"stbl") && kinds.contains(b"elst"));
    for unwanted in [b"udta", b"meta", b"ilst", b"\xa9nam", b"\xa9xyz"] {
        assert!(!kinds.contains(unwanted), "found {unwanted:?}");
    }
    // The six tables of the video track, and no `ctts` when nothing is reordered.
    let stbl = boxes::descend(&mut reader, &video, &[*b"mdia", *b"minf", *b"stbl"])
        .unwrap()
        .unwrap();
    let tables: Vec<[u8; 4]> = boxes::children(&mut reader, &stbl)
        .unwrap()
        .iter()
        .map(|h| h.kind)
        .collect();
    assert_eq!(
        tables,
        [*b"stsd", *b"stts", *b"stss", *b"stsc", *b"stsz", *b"co64"]
    );
}

// --- Frames that come out of the order they are shown (v5implementation §9) ---

/// The muxer's output on the code before `ctts` existed: a file with no
/// reordering must stay this, to the byte (D-211).
const BEFORE_REORDERING: [(i64, &str); 2] = [
    (
        0,
        "2061536314eb0431583754289fac7e4e9d4f1339fc95f6aec7c9d91eb1c68dd0",
    ),
    (
        -21_333,
        "99b09c4bc358a74060e4e8bedda496d3a0ce8e81addeef30297f0f5e8dd0a838",
    ),
];

/// A keyframe, then three frames at a time written as an encoder with B-frames
/// gives them: the last of the three first, then the two shown before it.
fn b_frame_order(frames: u32) -> Vec<u32> {
    let mut order = vec![0];
    let mut next = 1;
    while next + 2 < frames {
        order.extend([next + 2, next, next + 1]);
        next += 3;
    }
    order.extend(next..frames);
    order
}

/// Muxes samples shown as `order` says, in turns with `audio_samples` of audio.
fn mux_shown(order: &[u32], audio_samples: u32) -> (Vec<u8>, Vec<Vec<u8>>) {
    let mut seeded = Seeded(0x9E37_79B9_7F4A_7C15);
    let mut sink = MemSink::default();
    let mut muxer = Mp4Muxer::new(&mut sink, video_spec(), audio_spec()).unwrap();
    let mut written = Vec::new();
    let mut audio = 0u32;
    for (number, &frame) in (0u32..).zip(order) {
        let data = seeded.bytes(300 + (frame as usize % 7));
        muxer
            .add_video_sample(&data, FrameIdx::new(frame), frame % 60 == 0)
            .unwrap();
        written.push(data);
        while audio < audio_samples && u64::from(audio) * 1_024 < u64::from(number + 1) * 1_600 {
            muxer
                .add_audio_sample(&[1, 2, 3], audio_pts(Micros::new(0), audio), AAC_FRAME)
                .unwrap();
            audio += 1;
        }
    }
    muxer.finalize().unwrap();
    (sink.0, written)
}

/// Each sample comes back with its bytes, in the order written, at the time it is shown.
fn assert_shown(file: &[u8], order: &[u32], written: &[Vec<u8>]) {
    let mut d = Demuxer::open(MemReader(file.to_vec())).unwrap();
    assert_eq!(d.video_sample_count() as usize, order.len());
    let mut out = Vec::new();
    for (n, (&frame, data)) in (0u32..).zip(order.iter().zip(written)) {
        let meta = d.read_video_sample(n, &mut out).unwrap();
        assert_eq!(&out, data, "the bytes of video sample {n}");
        assert_eq!(meta.pts, FrameIdx::new(frame).to_micros(), "sample {n}");
        assert_eq!(meta.is_keyframe, frame % 60 == 0, "sample {n}");
    }
}

/// The types of the tables of the video track.
fn video_tables(file: &[u8]) -> Vec<[u8; 4]> {
    let (mut reader, video, _) = traks(file);
    let stbl = boxes::descend(&mut reader, &video, &[*b"mdia", *b"minf", *b"stbl"])
        .unwrap()
        .unwrap();
    boxes::children(&mut reader, &stbl)
        .unwrap()
        .iter()
        .map(|h| h.kind)
        .collect()
}

#[test]
fn frames_written_in_decode_order_are_shown_in_presentation_order() {
    let order = [0, 3, 1, 2, 6, 4, 5];
    let (file, written) = mux_shown(&order, 10);
    assert_shown(&file, &order, &written);

    let (mut reader, video, _) = traks(&file);
    let stbl = boxes::descend(&mut reader, &video, &[*b"mdia", *b"minf", *b"stbl"])
        .unwrap()
        .unwrap();
    let ctts = boxes::child(&mut reader, &stbl, b"ctts").unwrap().unwrap();
    // Sample 2 is frame 1, a frame behind: that is the delay, 1,000 ticks. Each
    // sample's offset is its frame's time, less its decode time, plus the delay.
    assert_eq!(
        read_ctts(&mut reader, &ctts).unwrap(),
        [(1, 1_000), (1, 3_000), (2, 0), (1, 3_000), (2, 0),]
    );
    let elst = boxes::descend(&mut reader, &video, &[*b"edts", *b"elst"])
        .unwrap()
        .unwrap();
    let edits = read_elst(&mut reader, &elst).unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].media_time, 1_000);
    // Seven frames at 30 fps, in milliseconds.
    assert_eq!(edits[0].segment_duration, 233);
    assert_eq!(
        video_tables(&file),
        [
            *b"stsd", *b"stts", *b"ctts", *b"stss", *b"stsc", *b"stsz", *b"co64"
        ]
    );
    // The keyframes are numbered by sample, in decode order.
    let d = Demuxer::open(MemReader(file.clone())).unwrap();
    assert_eq!(d.keyframe_at_or_before(offcut_types::TimeMs::new(100)), 0);
}

#[test]
fn no_reordering_writes_no_ctts_and_no_video_edit() {
    for (first, expected) in BEFORE_REORDERING {
        let w = three_seconds(Micros::new(first));
        let hash: String = Sha256::digest(&w.file)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(hash, expected, "the file changed, first audio at {first}");
        assert_eq!(
            video_tables(&w.file),
            [*b"stsd", *b"stts", *b"stss", *b"stsc", *b"stsz", *b"co64"]
        );
        let (mut reader, video, _) = traks(&w.file);
        assert_eq!(boxes::child(&mut reader, &video, b"edts").unwrap(), None);
    }
}

#[test]
fn a_gap_in_the_frames_is_out_of_order_at_finalize() {
    let mut sink = MemSink::default();
    let mut muxer = Mp4Muxer::new(&mut sink, video_spec(), audio_spec()).unwrap();
    muxer
        .add_video_sample(&[1], FrameIdx::new(0), true)
        .unwrap();
    muxer
        .add_video_sample(&[1], FrameIdx::new(2), false)
        .unwrap();
    assert_eq!(muxer.finalize(), Err(MuxError::OutOfOrder));
}

#[test]
fn a_frame_far_ahead_is_out_of_order_at_once() {
    let mut muxer = Mp4Muxer::new(MemSink::default(), video_spec(), audio_spec()).unwrap();
    muxer
        .add_video_sample(&[1], FrameIdx::new(0), true)
        .unwrap();
    // Sample 1 may be shown up to 16 frames ahead of its turn, and no further.
    assert_eq!(
        muxer.add_video_sample(&[1], FrameIdx::new(18), false),
        Err(MuxError::OutOfOrder)
    );
    muxer
        .add_video_sample(&[1], FrameIdx::new(17), false)
        .unwrap();
}

#[test]
fn ninety_seconds_reordered_fit_the_space_kept_for_moov() {
    let order = b_frame_order(2_700);
    let (file, written) = mux_shown(&order, 4_220);
    let top = top_level(&file);
    let (moov, mdat) = (find(&top, b"moov"), find(&top, b"mdat"));
    assert!(moov.end.get() - moov.start.get() < MOOV_RESERVE);
    assert_eq!(mdat.start.get() - moov.start.get(), MOOV_RESERVE);
    assert_shown(&file, &order, &written);
}

/// One-byte frames in their order, a chunk of their own when `turns`, written
/// in turns with as many audio samples. What `finalize` says, and the file.
fn mux_small(frames: u32, turns: bool) -> (Result<Bytes, MuxError>, Vec<u8>) {
    let mut sink = MemSink::default();
    // One byte longer than `AVCC`: the rest of `moov` is an odd length, and this
    // makes it a multiple of 4, so that a count of frames can fill the space exactly.
    let spec = VideoTrackSpec {
        avcc: [&AVCC[..], &[0]].concat(),
        ..video_spec()
    };
    let mut muxer = Mp4Muxer::new(&mut sink, spec, audio_spec()).unwrap();
    for n in 0..frames {
        muxer
            .add_video_sample(&[1], FrameIdx::new(n), n == 0)
            .unwrap();
        if turns {
            muxer
                .add_audio_sample(&[2], audio_pts(Micros::new(0), n), AAC_FRAME)
                .unwrap();
        }
    }
    let result = muxer.finalize();
    (result, sink.0)
}

/// How much of the space kept for `moov` a file left over: the size of its `free` box.
fn left_over(file: &[u8]) -> u64 {
    top_level(file)
        .iter()
        .find(|h| &h.kind == b"free")
        .map_or(0, |h| h.end.get() - h.start.get())
}

#[test]
fn a_moov_over_the_reserve_is_refused() {
    let (result, file) = mux_small(30_000, true);
    assert_eq!(result, Err(MuxError::MoovOverflow));
    // Nothing was written over the samples: the space is still as it was kept.
    let top = top_level(&file);
    let order: Vec<[u8; 4]> = top.iter().map(|h| h.kind).collect();
    assert_eq!(order, [*b"ftyp", *b"free", *b"mdat"]);
    let free = find(&top, b"free");
    assert_eq!(free.end.get() - free.start.get(), MOOV_RESERVE);
    let body = &file[free.body.get() as usize..free.end.get() as usize];
    assert!(body.iter().all(|&byte| byte == 0));
}

#[test]
fn a_reserve_with_under_8_bytes_left_is_refused() {
    // A frame in an open chunk adds 4 bytes to `moov`, so one count of frames
    // leaves 8 bytes, the next 4, too few for a `free` box, and the next none.
    let (first, file) = mux_small(50_000, false);
    first.unwrap();
    // 15 frames take a chunk: 15 x 4 bytes in `stsz` and 8 in `co64`.
    let mut frames = 50_000 + (left_over(&file) as u32 - 16) / 68 * 15;
    let mut before = None;
    let refused = loop {
        match mux_small(frames, false) {
            (Ok(_), file) => before = Some(left_over(&file)),
            (Err(error), _) => break error,
        }
        frames += 1;
        assert!(frames < 62_000, "no count of frames was refused");
    };
    assert_eq!(refused, MuxError::MoovOverflow);
    // It was refused with room left, and not for want of room: one frame more
    // fills the space exactly, and is accepted.
    assert_eq!(before, Some(8));
    let (fits, file) = mux_small(frames + 1, false);
    fits.unwrap();
    assert_eq!(left_over(&file), 0);
    let top = top_level(&file);
    let moov = find(&top, b"moov");
    assert_eq!(moov.end.get() - moov.start.get(), MOOV_RESERVE);
}

/// `count` frames, with each group of `window` after the first frame turned by `turn`.
fn permuted(count: u32, window: usize, turn: usize) -> Vec<u32> {
    let mut order: Vec<u32> = (0..count).collect();
    if let Some(rest) = order.get_mut(1..) {
        for group in rest.chunks_mut(window) {
            group.rotate_left(turn % group.len());
        }
    }
    order
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn any_permutation_within_the_window_comes_back(
        count in 1u32..=240,
        window in 1usize..=16,
        turn in 0usize..16,
        audio in 0u32..=40,
    ) {
        let order = permuted(count, window, turn);
        let (file, written) = mux_shown(&order, audio);
        assert_shown(&file, &order, &written);
    }
}
