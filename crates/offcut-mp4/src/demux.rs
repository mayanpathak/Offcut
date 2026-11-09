//! Opens a file, picks its video and its audio track, and hands out samples
//! with their times on the clip's own timeline: the first presented video
//! frame is at 0, and audio is shifted by the same amount.

use offcut_types::{Bytes, Micros, TimeMs};

use crate::ContainerError;
use crate::boxes::{self, BoxHeader, SampleEntry, Tkhd};
use crate::reader::RandomAccess;
use crate::sample_table::{self as tables, MAX_RESOLVED_SAMPLES, RawTables, Sample, TrackTable};

/// When a sample is shown and decoded, and for how long it is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SampleMeta {
    pub pts: Micros,
    pub dts: Micros,
    pub duration: Micros,
    pub is_keyframe: bool,
}

/// One chosen track: what its boxes say, and its samples.
pub(crate) struct Track {
    pub(crate) entry: SampleEntry,
    pub(crate) tkhd: Tkhd,
    /// The runs of `stts`, which the probe reads in place of the samples.
    pub(crate) stts: Vec<(u32, u32)>,
    pub(crate) table: TrackTable,
}

pub struct Demuxer<R> {
    reader: R,
    brand: Option<[u8; 4]>,
    video: Option<Track>,
    audio: Option<Track>,
    video_tracks: u32,
    audio_tracks: u32,
    /// The smallest presented video time, in ticks of the video timescale.
    t0: i64,
}

impl<R: RandomAccess> Demuxer<R> {
    pub fn open(mut reader: R) -> Result<Demuxer<R>, ContainerError> {
        let len = reader.len();
        let (brand, moov) = find_moov(&mut reader, len)?;
        let in_moov = boxes::children(&mut reader, &moov)?;
        if in_moov.iter().any(|h| &h.kind == b"mvex") {
            return Err(ContainerError::Fragmented);
        }
        let mvhd = find(&in_moov, b"mvhd").ok_or(ContainerError::Malformed("mvhd"))?;
        let movie_timescale = boxes::read_mvhd(&mut reader, &mvhd)?.timescale;

        // The first enabled video track, or the first one when none is
        // enabled; the first audio track. Every track of each kind is counted.
        let mut chosen_video: Option<(BoxHeader, bool)> = None;
        let mut chosen_audio: Option<BoxHeader> = None;
        let (mut video_tracks, mut audio_tracks) = (0u32, 0u32);
        for trak in in_moov.iter().filter(|h| &h.kind == b"trak") {
            let Some(hdlr) = boxes::descend(&mut reader, trak, &[*b"mdia", *b"hdlr"])? else {
                continue;
            };
            match &boxes::read_hdlr(&mut reader, &hdlr)? {
                b"vide" => {
                    video_tracks = video_tracks.saturating_add(1);
                    let enabled = read_tkhd(&mut reader, trak)?.flags & 1 == 1;
                    if chosen_video.is_none_or(|(_, was_enabled)| enabled && !was_enabled) {
                        chosen_video = Some((*trak, enabled));
                    }
                }
                b"soun" => {
                    audio_tracks = audio_tracks.saturating_add(1);
                    chosen_audio = chosen_audio.or(Some(*trak));
                }
                _ => {}
            }
        }

        let video = match chosen_video {
            Some((trak, _)) => Some(read_track(
                &mut reader,
                &trak,
                b"vide",
                movie_timescale,
                len,
            )?),
            None => None,
        };
        let audio = match chosen_audio {
            Some(trak) => Some(read_track(
                &mut reader,
                &trak,
                b"soun",
                movie_timescale,
                len,
            )?),
            None => None,
        };
        // With no video samples to look at, the timeline starts where the track does.
        let t0 = video
            .as_ref()
            .and_then(|v| {
                v.table
                    .samples
                    .iter()
                    .map(|s| s.pts.saturating_sub(v.table.edit_offset))
                    .min()
            })
            .unwrap_or(0);
        Ok(Demuxer {
            reader,
            brand,
            video,
            audio,
            video_tracks,
            audio_tracks,
            t0,
        })
    }

    /// The raw `avcC` payload.
    pub fn video_decoder_description(&self) -> Option<&[u8]> {
        self.video
            .as_ref()
            .and_then(|track| track.entry.config.as_deref())
    }

    /// The `AudioSpecificConfig`.
    pub fn audio_decoder_description(&self) -> Option<&[u8]> {
        self.audio
            .as_ref()
            .and_then(|track| track.entry.config.as_deref())
    }

    pub fn video_sample_count(&self) -> u32 {
        self.video
            .as_ref()
            .map_or(0, |track| track.table.sample_count)
    }

    pub fn audio_sample_count(&self) -> u32 {
        self.audio
            .as_ref()
            .map_or(0, |track| track.table.sample_count)
    }

    /// Samples are in decode order. `out` is cleared, then holds the sample.
    pub fn read_video_sample(
        &mut self,
        index: u32,
        out: &mut Vec<u8>,
    ) -> Result<SampleMeta, ContainerError> {
        let base = self.time_base();
        read_sample(&mut self.reader, self.video.as_ref(), index, out, base)
    }

    /// An audio sample may start before 0: audio that precedes the first video frame.
    pub fn read_audio_sample(
        &mut self,
        index: u32,
        out: &mut Vec<u8>,
    ) -> Result<SampleMeta, ContainerError> {
        let base = self.time_base();
        read_sample(&mut self.reader, self.audio.as_ref(), index, out, base)
    }

    /// The index of the last keyframe shown at or before `t`; 0 when `t`
    /// precedes every keyframe.
    pub fn keyframe_at_or_before(&self, t: TimeMs) -> u32 {
        let Some(track) = &self.video else { return 0 };
        let base = self.time_base();
        let mut best: Option<(Micros, u32)> = None;
        for (index, sample) in (0u32..).zip(&track.table.samples) {
            let pts = meta(sample, &track.table, base).pts;
            if sample.keyframe && pts <= t.to_micros() && best.is_none_or(|(at, _)| pts >= at) {
                best = Some((pts, index));
            }
        }
        best.map_or(0, |(_, index)| index)
    }

    /// `t0` and the timescale it is in.
    fn time_base(&self) -> (i64, u32) {
        (
            self.t0,
            self.video.as_ref().map_or(1, |track| track.table.timescale),
        )
    }

    pub(crate) fn brand(&self) -> Option<[u8; 4]> {
        self.brand
    }

    pub(crate) fn video(&self) -> Option<&Track> {
        self.video.as_ref()
    }

    pub(crate) fn audio(&self) -> Option<&Track> {
        self.audio.as_ref()
    }

    /// How many video and how many audio tracks the file has.
    pub(crate) fn track_counts(&self) -> (u32, u32) {
        (self.video_tracks, self.audio_tracks)
    }
}

/// The boxes of the chosen video track's sample entry that V6.1 reads and V5
/// did not: `hvcC` (D-396) and `colr` (D-397), as raw bodies.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VideoEntryBoxes {
    pub fourcc: Option<[u8; 4]>,
    pub hvcc: Option<Vec<u8>>,
    pub colr: Option<Vec<u8>>,
}

/// The most bytes of an `hvcC` or `colr` body. A real `hvcC` is a few kB.
const MAX_ENTRY_BOX: u64 = 1 << 20;

impl<R: RandomAccess> Demuxer<R> {
    /// V6.1 (additive): walks `moov` again to the video track `open` chose
    /// (the first enabled one, or the first), and reads its sample entry's
    /// `hvcC` and `colr`. `open` and everything it read are untouched.
    pub fn video_entry_boxes(&mut self) -> Result<VideoEntryBoxes, ContainerError> {
        let r = &mut self.reader;
        let len = r.len();
        let (_, moov) = find_moov(r, len)?;
        let mut chosen: Option<(BoxHeader, bool)> = None;
        for trak in boxes::children(r, &moov)?
            .iter()
            .filter(|h| &h.kind == b"trak")
        {
            let Some(hdlr) = boxes::descend(r, trak, &[*b"mdia", *b"hdlr"])? else {
                continue;
            };
            if &boxes::read_hdlr(r, &hdlr)? != b"vide" {
                continue;
            }
            let enabled = read_tkhd(r, trak)?.flags & 1 == 1;
            if chosen.is_none_or(|(_, was_enabled)| enabled && !was_enabled) {
                chosen = Some((*trak, enabled));
            }
        }
        let Some((trak, _)) = chosen else {
            return Ok(VideoEntryBoxes::default());
        };
        let Some(stsd) = boxes::descend(r, &trak, &[*b"mdia", *b"minf", *b"stbl", *b"stsd"])?
        else {
            return Ok(VideoEntryBoxes::default());
        };
        // Version, flags and the entry count come before the first entry; a
        // visual sample entry's own fields take 78 bytes before its boxes.
        let entry = boxes::read_header(r, Bytes::new(stsd.body.get().saturating_add(8)), stsd.end)?;
        let inner = BoxHeader {
            body: Bytes::new(entry.body.get().saturating_add(78).min(entry.end.get())),
            ..entry
        };
        let mut out = VideoEntryBoxes {
            fourcc: Some(entry.kind),
            ..VideoEntryBoxes::default()
        };
        for child in boxes::children(r, &inner)? {
            let slot = match &child.kind {
                b"hvcC" => &mut out.hvcc,
                b"colr" => &mut out.colr,
                _ => continue,
            };
            if slot.is_none() {
                if child.body_len() > MAX_ENTRY_BOX {
                    return Err(ContainerError::Malformed("sample entry"));
                }
                *slot = Some(boxes::load(r, &child, MAX_ENTRY_BOX)?);
            }
        }
        Ok(out)
    }
}

/// The most samples `open_source` expands in one track (V6.1). 45 minutes of
/// video at 60.5 fps is 163,350 samples and of 96 kHz AAC 253,125; a sample
/// costs about 40 bytes, so a track holds at most about 16 MB.
pub const MAX_SOURCE_SAMPLES: u32 = 400_000;

impl<R: RandomAccess> Demuxer<R> {
    /// V6.1 (additive): `open` for a long source. The same steps, the same
    /// choice of tracks and the same timeline, but each chosen track is
    /// expanded up to `MAX_SOURCE_SAMPLES` instead of 20,000, so 45 minutes of
    /// audio can be read. `open` and the clip path keep their limit (D-60).
    pub fn open_source(mut reader: R) -> Result<Demuxer<R>, ContainerError> {
        let len = reader.len();
        let (brand, moov) = find_moov(&mut reader, len)?;
        let in_moov = boxes::children(&mut reader, &moov)?;
        if in_moov.iter().any(|h| &h.kind == b"mvex") {
            return Err(ContainerError::Fragmented);
        }
        let mvhd = find(&in_moov, b"mvhd").ok_or(ContainerError::Malformed("mvhd"))?;
        let movie_timescale = boxes::read_mvhd(&mut reader, &mvhd)?.timescale;

        // As `open`: the first enabled video track, or the first one; the first audio track.
        let mut chosen_video: Option<(BoxHeader, bool)> = None;
        let mut chosen_audio: Option<BoxHeader> = None;
        let (mut video_tracks, mut audio_tracks) = (0u32, 0u32);
        for trak in in_moov.iter().filter(|h| &h.kind == b"trak") {
            let Some(hdlr) = boxes::descend(&mut reader, trak, &[*b"mdia", *b"hdlr"])? else {
                continue;
            };
            match &boxes::read_hdlr(&mut reader, &hdlr)? {
                b"vide" => {
                    video_tracks = video_tracks.saturating_add(1);
                    let enabled = read_tkhd(&mut reader, trak)?.flags & 1 == 1;
                    if chosen_video.is_none_or(|(_, was_enabled)| enabled && !was_enabled) {
                        chosen_video = Some((*trak, enabled));
                    }
                }
                b"soun" => {
                    audio_tracks = audio_tracks.saturating_add(1);
                    chosen_audio = chosen_audio.or(Some(*trak));
                }
                _ => {}
            }
        }
        let mut track = |trak: &BoxHeader, handler: &[u8; 4]| {
            read_track_up_to(
                &mut reader,
                trak,
                handler,
                movie_timescale,
                len,
                MAX_SOURCE_SAMPLES,
            )
        };
        let video = match chosen_video {
            Some((trak, _)) => Some(track(&trak, b"vide")?),
            None => None,
        };
        let audio = match chosen_audio {
            Some(trak) => Some(track(&trak, b"soun")?),
            None => None,
        };
        let t0 = video
            .as_ref()
            .and_then(|v| {
                v.table
                    .samples
                    .iter()
                    .map(|s| s.pts.saturating_sub(v.table.edit_offset))
                    .min()
            })
            .unwrap_or(0);
        Ok(Demuxer {
            reader,
            brand,
            video,
            audio,
            video_tracks,
            audio_tracks,
            t0,
        })
    }

    /// V6.1 (additive): an audio sample's times without reading its bytes, so
    /// a region's first sample can be found by a binary search (D-402).
    pub fn audio_sample_meta(&self, index: u32) -> Option<SampleMeta> {
        let track = self.audio.as_ref().filter(|t| t.table.resolved)?;
        let sample = track.table.samples.get(usize::try_from(index).ok()?)?;
        Some(meta(sample, &track.table, self.time_base()))
    }
}

/// `read_track` with the expansion limit given (V6.1, `open_source`).
fn read_track_up_to<R: RandomAccess>(
    r: &mut R,
    trak: &BoxHeader,
    handler: &[u8; 4],
    movie_timescale: u32,
    file_len: Bytes,
    max_samples: u32,
) -> Result<Track, ContainerError> {
    let tkhd = read_tkhd(r, trak)?;
    let mdhd =
        boxes::descend(r, trak, &[*b"mdia", *b"mdhd"])?.ok_or(ContainerError::Malformed("mdhd"))?;
    let timescale = boxes::read_mdhd(r, &mdhd)?.timescale;
    if timescale == 0 {
        return Err(ContainerError::Malformed("mdhd"));
    }
    let stbl = boxes::descend(r, trak, &[*b"mdia", *b"minf", *b"stbl"])?
        .ok_or(ContainerError::Malformed("stbl"))?;
    let in_stbl = boxes::children(r, &stbl)?;
    let need = |kind: &[u8; 4], name| find(&in_stbl, kind).ok_or(ContainerError::Malformed(name));

    let entry = boxes::read_stsd(r, &need(b"stsd", "stsd")?, handler)?;
    let stts = tables::read_stts(r, &need(b"stts", "stts")?)?;
    let stsz = tables::read_stsz(r, &need(b"stsz", "stsz")?, max_samples)?;
    let expand = stsz.sample_count <= max_samples;
    let mut raw = RawTables {
        timescale,
        movie_timescale,
        file_len,
        stts,
        ctts: Vec::new(),
        stsc: Vec::new(),
        stsz,
        chunk_offsets: Vec::new(),
        stss: None,
        elst: Vec::new(),
    };
    if expand {
        if let Some(ctts) = find(&in_stbl, b"ctts") {
            raw.ctts = tables::read_ctts(r, &ctts)?;
        }
        if let Some(stsc) = find(&in_stbl, b"stsc") {
            raw.stsc = tables::read_stsc(r, &stsc)?;
        }
        if let Some(stco) = find(&in_stbl, b"stco") {
            raw.chunk_offsets = tables::read_stco(r, &stco)?;
        } else if let Some(co64) = find(&in_stbl, b"co64") {
            raw.chunk_offsets = tables::read_co64(r, &co64)?;
        }
        if let Some(stss) = find(&in_stbl, b"stss") {
            raw.stss = Some(tables::read_stss(r, &stss)?);
        }
    }
    if let Some(elst) = boxes::descend(r, trak, &[*b"edts", *b"elst"])? {
        raw.elst = tables::read_elst(r, &elst)?;
    }
    let table = tables::resolve_up_to(&raw, max_samples)?;
    Ok(Track {
        entry,
        tkhd,
        stts: raw.stts,
        table,
    })
}

fn find(headers: &[BoxHeader], kind: &[u8; 4]) -> Option<BoxHeader> {
    headers.iter().find(|h| &h.kind == kind).copied()
}

/// Step 1 of `open`: the major brand, when there is an `ftyp`, and `moov`,
/// wherever it is.
fn find_moov<R: RandomAccess>(
    r: &mut R,
    len: Bytes,
) -> Result<(Option<[u8; 4]>, BoxHeader), ContainerError> {
    // A file too short for one box header is not this kind of file at all,
    // and neither is one that starts with anything but a known box.
    let mut first = [0u8; 8];
    if len.get() < 8 {
        return Err(ContainerError::NotIsoBmff);
    }
    r.read_at(Bytes::new(0), &mut first)?;
    let known: [&[u8]; 7] = [
        b"ftyp", b"moov", b"mdat", b"free", b"wide", b"skip", b"moof",
    ];
    if !first.get(4..).is_some_and(|kind| known.contains(&kind)) {
        return Err(ContainerError::NotIsoBmff);
    }

    let mut brand = None;
    let mut at = 0u64;
    for _ in 0..boxes::MAX_CHILDREN {
        if len.get().saturating_sub(at) < 8 {
            break;
        }
        let header = boxes::read_header(r, Bytes::new(at), len)?;
        match &header.kind {
            b"moov" => return Ok((brand, header)),
            b"moof" => return Err(ContainerError::Fragmented),
            b"ftyp" => brand = Some(boxes::read_ftyp(r, &header)?.major),
            _ => {}
        }
        at = header.end.get();
    }
    Err(ContainerError::Malformed("moov"))
}

fn read_tkhd<R: RandomAccess>(r: &mut R, trak: &BoxHeader) -> Result<Tkhd, ContainerError> {
    let tkhd = boxes::child(r, trak, b"tkhd")?.ok_or(ContainerError::Malformed("tkhd"))?;
    boxes::read_tkhd(r, &tkhd)
}

/// Steps 2 and 3 of `open` for one chosen track.
fn read_track<R: RandomAccess>(
    r: &mut R,
    trak: &BoxHeader,
    handler: &[u8; 4],
    movie_timescale: u32,
    file_len: Bytes,
) -> Result<Track, ContainerError> {
    let tkhd = read_tkhd(r, trak)?;
    let mdhd =
        boxes::descend(r, trak, &[*b"mdia", *b"mdhd"])?.ok_or(ContainerError::Malformed("mdhd"))?;
    let timescale = boxes::read_mdhd(r, &mdhd)?.timescale;
    if timescale == 0 {
        return Err(ContainerError::Malformed("mdhd"));
    }
    let stbl = boxes::descend(r, trak, &[*b"mdia", *b"minf", *b"stbl"])?
        .ok_or(ContainerError::Malformed("stbl"))?;
    let in_stbl = boxes::children(r, &stbl)?;
    let need = |kind: &[u8; 4], name| find(&in_stbl, kind).ok_or(ContainerError::Malformed(name));

    let entry = boxes::read_stsd(r, &need(b"stsd", "stsd")?, handler)?;
    let stts = tables::read_stts(r, &need(b"stts", "stts")?)?;
    let stsz = tables::read_stsz(r, &need(b"stsz", "stsz")?, MAX_RESOLVED_SAMPLES)?;
    // A track that will not be expanded does not have its per-sample tables read.
    let expand = stsz.sample_count <= MAX_RESOLVED_SAMPLES;
    let mut raw = RawTables {
        timescale,
        movie_timescale,
        file_len,
        stts,
        ctts: Vec::new(),
        stsc: Vec::new(),
        stsz,
        chunk_offsets: Vec::new(),
        stss: None,
        elst: Vec::new(),
    };
    if expand {
        if let Some(ctts) = find(&in_stbl, b"ctts") {
            raw.ctts = tables::read_ctts(r, &ctts)?;
        }
        if let Some(stsc) = find(&in_stbl, b"stsc") {
            raw.stsc = tables::read_stsc(r, &stsc)?;
        }
        if let Some(stco) = find(&in_stbl, b"stco") {
            raw.chunk_offsets = tables::read_stco(r, &stco)?;
        } else if let Some(co64) = find(&in_stbl, b"co64") {
            raw.chunk_offsets = tables::read_co64(r, &co64)?;
        }
        if let Some(stss) = find(&in_stbl, b"stss") {
            raw.stss = Some(tables::read_stss(r, &stss)?);
        }
    }
    if let Some(elst) = boxes::descend(r, trak, &[*b"edts", *b"elst"])? {
        raw.elst = tables::read_elst(r, &elst)?;
    }
    let table = tables::resolve(&raw)?;
    Ok(Track {
        entry,
        tkhd,
        stts: raw.stts,
        table,
    })
}

fn read_sample<R: RandomAccess>(
    r: &mut R,
    track: Option<&Track>,
    index: u32,
    out: &mut Vec<u8>,
    base: (i64, u32),
) -> Result<SampleMeta, ContainerError> {
    out.clear();
    let track = track.ok_or(ContainerError::Malformed("index"))?;
    if !track.table.resolved {
        return Err(ContainerError::Malformed("samples"));
    }
    let sample = usize::try_from(index)
        .ok()
        .and_then(|index| track.table.samples.get(index));
    let sample = sample.ok_or(ContainerError::Malformed("index"))?;
    // `resolve` has checked that the sample lies inside the file.
    out.resize(sample.size as usize, 0);
    r.read_at(sample.offset, out)?;
    Ok(meta(sample, &track.table, base))
}

/// Step 4 of `open`: the times of a sample with the edit list applied and
/// the first presented video frame at 0.
fn meta(sample: &Sample, table: &TrackTable, (t0, t0_scale): (i64, u32)) -> SampleMeta {
    SampleMeta {
        pts: micros(
            sample.pts.saturating_sub(table.edit_offset),
            table.timescale,
            t0,
            t0_scale,
        ),
        dts: micros(
            sample.dts.saturating_sub(table.edit_offset),
            table.timescale,
            t0,
            t0_scale,
        ),
        duration: micros(i64::from(sample.duration), table.timescale, 0, 1),
        is_keyframe: sample.keyframe,
    }
}

/// `ticks / timescale` seconds less `t0 / t0_scale` seconds, in microseconds,
/// rounded down.
fn micros(ticks: i64, timescale: u32, t0: i64, t0_scale: u32) -> Micros {
    let (timescale, t0_scale) = (i128::from(timescale.max(1)), i128::from(t0_scale.max(1)));
    let ticks = i128::from(ticks) * t0_scale - i128::from(t0) * timescale;
    let micros = (ticks * 1_000_000).div_euclid(timescale * t0_scale);
    Micros::new(i64::try_from(micros).unwrap_or(i64::MAX))
}

#[cfg(test)]
pub(crate) mod fixture {
    //! Small MP4 files built in memory, for the tests of this crate.

    /// A box with a 32-bit size.
    pub(crate) fn boxed(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut bytes = (body.len() as u32 + 8).to_be_bytes().to_vec();
        bytes.extend_from_slice(kind);
        bytes.extend_from_slice(body);
        bytes
    }

    /// A box whose body starts with a version and flags, then these 32-bit fields.
    fn full(kind: &[u8; 4], flags: u32, fields: &[u32]) -> Vec<u8> {
        let body: Vec<u8> = [flags]
            .iter()
            .chain(fields)
            .flat_map(|v| v.to_be_bytes())
            .collect();
        boxed(kind, &body)
    }

    pub(crate) const IDENTITY: [i32; 9] = [0x1_0000, 0, 0, 0, 0x1_0000, 0, 0, 0, 0x4000_0000];

    /// One track: `sample_count` samples of `sample_size` bytes, each lasting `delta` ticks.
    pub(crate) struct TrackSpec {
        pub handler: [u8; 4],
        pub fourcc: [u8; 4],
        pub timescale: u32,
        pub sample_count: u32,
        pub delta: u32,
        pub sample_size: u32,
        pub matrix: [i32; 9],
        pub width: u16,
        pub height: u16,
        /// The `avcC` payload of a video track; the `AudioSpecificConfig` of
        /// an audio track, which then gets an `esds` with object type 0x40.
        pub config: Vec<u8>,
    }

    /// H.264 High at level 4.0, 1280x720, 30 fps.
    pub(crate) fn video(sample_count: u32) -> TrackSpec {
        TrackSpec {
            handler: *b"vide",
            fourcc: *b"avc1",
            timescale: 30_000,
            sample_count,
            delta: 1_000,
            sample_size: 4,
            matrix: IDENTITY,
            width: 1280,
            height: 720,
            config: vec![0x01, 0x64, 0x00, 0x28, 0xFF, 0xE1, 0x00, 0x00],
        }
    }

    /// 48 kHz stereo. `mp4a` is AAC-LC in frames of 1,024 samples; any other
    /// code is given one sample per tick and no config, as PCM has.
    pub(crate) fn audio(fourcc: &[u8; 4], sample_count: u32) -> TrackSpec {
        let aac = fourcc == b"mp4a";
        TrackSpec {
            handler: *b"soun",
            fourcc: *fourcc,
            timescale: 48_000,
            sample_count,
            delta: if aac { 1_024 } else { 1 },
            sample_size: 4,
            matrix: IDENTITY,
            width: 0,
            height: 0,
            config: if aac { vec![0x11, 0x90] } else { Vec::new() },
        }
    }

    fn sample_entry(t: &TrackSpec) -> Vec<u8> {
        let mut body = vec![0u8; 6];
        body.extend_from_slice(&1u16.to_be_bytes()); // data reference index
        if &t.handler == b"vide" {
            body.extend_from_slice(&[0; 16]);
            body.extend_from_slice(&t.width.to_be_bytes());
            body.extend_from_slice(&t.height.to_be_bytes());
            body.extend_from_slice(&[0; 50]); // resolution, frame count, compressor name, depth
            if !t.config.is_empty() {
                body.extend_from_slice(&boxed(b"avcC", &t.config));
            }
        } else {
            body.extend_from_slice(&[0; 8]); // version 0, revision, vendor
            body.extend_from_slice(&2u16.to_be_bytes()); // channels
            body.extend_from_slice(&[0, 16, 0, 0, 0, 0]); // sample size, compression id, packet size
            body.extend_from_slice(&(48_000u32 << 16).to_be_bytes());
            if !t.config.is_empty() {
                let asc_len = t.config.len() as u8;
                let mut esds = vec![0, 0, 0, 0, 0x03, 23 + asc_len, 0, 1, 0];
                esds.extend_from_slice(&[0x04, 15 + asc_len, 0x40, 0x15]);
                esds.extend_from_slice(&[0; 11]); // buffer size and two bitrates
                esds.extend_from_slice(&[0x05, asc_len]);
                esds.extend_from_slice(&t.config);
                esds.extend_from_slice(&[0x06, 1, 2]);
                body.extend_from_slice(&boxed(b"esds", &esds));
            }
        }
        boxed(&t.fourcc, &body)
    }

    fn trak(t: &TrackSpec, id: u32, chunk_offset: u32) -> Vec<u8> {
        let duration = t.sample_count.saturating_mul(t.delta);
        let mut tkhd = vec![0, 0, 0, 3]; // version 0; enabled, in the movie
        for field in [0, 0, id, 0, duration, 0, 0, 0, 0] {
            tkhd.extend_from_slice(&field.to_be_bytes());
        }
        for value in t.matrix {
            tkhd.extend_from_slice(&value.to_be_bytes());
        }
        tkhd.extend_from_slice(&(u32::from(t.width) << 16).to_be_bytes());
        tkhd.extend_from_slice(&(u32::from(t.height) << 16).to_be_bytes());

        let mut hdlr = vec![0u8; 8];
        hdlr.extend_from_slice(&t.handler);
        hdlr.extend_from_slice(&[0; 13]);

        let mut stsd = vec![0, 0, 0, 0, 0, 0, 0, 1];
        stsd.extend_from_slice(&sample_entry(t));
        let stbl = [
            boxed(b"stsd", &stsd),
            full(b"stts", 0, &[1, t.sample_count, t.delta]),
            full(b"stsc", 0, &[1, 1, t.sample_count, 1]),
            full(b"stsz", 0, &[t.sample_size, t.sample_count]),
            full(b"stco", 0, &[1, chunk_offset]),
        ]
        .concat();
        let mdia = [
            full(b"mdhd", 0, &[0, 0, t.timescale, duration, 0]),
            boxed(b"hdlr", &hdlr),
            boxed(b"minf", &boxed(b"stbl", &stbl)),
        ]
        .concat();
        boxed(
            b"trak",
            &[boxed(b"tkhd", &tkhd), boxed(b"mdia", &mdia)].concat(),
        )
    }

    /// `ftyp`, `mdat`, `moov`. Each track is one chunk in `mdat`, one after
    /// the other; sample `i` is filled with the byte `i`. A track too long to
    /// be expanded gets no bytes: nothing reads them.
    pub(crate) fn file(tracks: &[TrackSpec]) -> Vec<u8> {
        let ftyp = boxed(b"ftyp", b"isom\0\0\0\0isom");
        let mut payload = Vec::new();
        let mut moov = full(b"mvhd", 0, &[0, 0, 1_000, 0]);
        for (id, t) in (1u32..).zip(tracks) {
            let offset = (ftyp.len() + 8 + payload.len()) as u32;
            if t.sample_count <= crate::sample_table::MAX_RESOLVED_SAMPLES {
                for i in 0..t.sample_count {
                    payload.extend(std::iter::repeat_n(i as u8, t.sample_size as usize));
                }
            }
            moov.extend_from_slice(&trak(t, id, offset));
        }
        [ftyp, boxed(b"mdat", &payload), boxed(b"moov", &moov)].concat()
    }
}

#[cfg(test)]
mod tests {
    use super::fixture::{audio, boxed, file, video};
    use super::*;
    use crate::reader::MemReader;

    #[test]
    fn open_source_reads_what_open_reads() {
        // The long tracks are read by demux_fixtures.rs on long_12min.mp4: this
        // builder gives no bytes to a track over 20,000 samples.
        let bytes = file(&[video(90), audio(b"mp4a", 3)]);
        let mut short = Demuxer::open(MemReader(bytes.clone())).unwrap();
        let mut long = Demuxer::open_source(MemReader(bytes)).unwrap();
        assert_eq!(long.track_counts(), short.track_counts());
        let (mut a, mut b) = (Vec::new(), Vec::new());
        for i in 0..90 {
            assert_eq!(
                long.read_video_sample(i, &mut a),
                short.read_video_sample(i, &mut b)
            );
            assert_eq!(a, b);
        }
        for i in 0..3 {
            let read = long.read_audio_sample(i, &mut a).unwrap();
            assert_eq!(Ok(read), short.read_audio_sample(i, &mut b));
            assert_eq!(a, b);
            assert_eq!(long.audio_sample_meta(i), Some(read));
        }
        assert_eq!(long.audio_sample_meta(3), None);
    }

    #[test]
    fn read_video_sample_on_an_unresolved_track_is_malformed_samples() {
        // 30,000 video samples are not expanded; the three audio samples are.
        let mut d = Demuxer::open(MemReader(file(&[video(30_000), audio(b"mp4a", 3)]))).unwrap();
        let mut out = vec![9u8; 2];
        assert_eq!(d.video_sample_count(), 30_000);
        assert_eq!(
            d.read_video_sample(0, &mut out),
            Err(ContainerError::Malformed("samples"))
        );
        assert!(out.is_empty());

        // The other track still reads: its bytes, and its times in microseconds.
        let second = d.read_audio_sample(1, &mut out).unwrap();
        assert_eq!(out, [1, 1, 1, 1]);
        assert_eq!(
            (second.pts.get(), second.duration.get(), second.is_keyframe),
            (21_333, 21_333, true)
        );
        assert_eq!(
            d.read_audio_sample(3, &mut out),
            Err(ContainerError::Malformed("index"))
        );
        assert_eq!(d.audio_decoder_description(), Some(&[0x11, 0x90][..]));

        // A track that is expanded: decode order, and the keyframe to seek to.
        let mut d = Demuxer::open(MemReader(file(&[video(90)]))).unwrap();
        let last = d.read_video_sample(89, &mut out).unwrap();
        assert_eq!(
            (out.as_slice(), last.pts.get()),
            (&[89u8; 4][..], 2_966_666)
        );
        assert_eq!(d.keyframe_at_or_before(TimeMs::new(1_000)), 30);
        assert_eq!(
            d.read_audio_sample(0, &mut out),
            Err(ContainerError::Malformed("index"))
        );
    }

    #[test]
    fn a_file_shorter_than_one_box_header_is_not_iso_bmff() {
        let open = |bytes: Vec<u8>| Demuxer::open(MemReader(bytes)).map(|_| ());
        assert_eq!(open(b"hello\n".to_vec()), Err(ContainerError::NotIsoBmff));
        assert_eq!(open(Vec::new()), Err(ContainerError::NotIsoBmff));
        // Long enough, and still not a box this kind of file starts with.
        assert_eq!(
            open(b"This is a text file.".to_vec()),
            Err(ContainerError::NotIsoBmff)
        );

        // A file that is one, and is fragmented or has no `moov`.
        let ftyp = boxed(b"ftyp", b"isom\0\0\0\0");
        let fragment = [ftyp.clone(), boxed(b"moof", &[])].concat();
        assert_eq!(open(fragment), Err(ContainerError::Fragmented));
        let extends = [ftyp.clone(), boxed(b"moov", &boxed(b"mvex", &[]))].concat();
        assert_eq!(open(extends), Err(ContainerError::Fragmented));
        let no_moov = [ftyp, boxed(b"mdat", &[0; 4])].concat();
        assert_eq!(open(no_moov), Err(ContainerError::Malformed("moov")));
    }
}
