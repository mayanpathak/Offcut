//! Writes an MP4 that a player can start before it has the whole file:
//! `ftyp`, then `moov`, then `mdat`. The samples are written as they arrive,
//! so `moov`, which is only known at the end, goes into space kept free for
//! it in front of them.
//!
//! Video is H.264 at exactly 30 frames per second, written in the order the
//! encoder gives it: an encoder with B-frames gives a frame before the frames
//! shown ahead of it. Audio is AAC at 48 kHz in two channels. No sample is held
//! in memory: each one goes through the sink at once.

use offcut_types::{Bytes, FrameIdx, Hz, Micros, Px};

use crate::mux_boxes::{self as boxes, Chunk};
use crate::{IoError, MuxError};

/// The space kept for `moov` between `ftyp` and `mdat`. The `moov` of a
/// 90 s clip takes about an eighth of it.
pub const MOOV_RESERVE: u64 = 256 * 1024;

const MOVIE_TIMESCALE: u32 = 1_000;
const VIDEO_TIMESCALE: u32 = 30_000;
/// 30,000 / 1,000 is exactly 30 frames per second.
const VIDEO_SAMPLE_DELTA: u32 = 1_000;
const AUDIO_TIMESCALE: u32 = 48_000;
const AUDIO_CHANNELS: u32 = 2;
/// Half a second of video, and about half a second of AAC frames.
const VIDEO_CHUNK_SAMPLES: u32 = 15;
const AUDIO_CHUNK_SAMPLES: u32 = 24;
/// The most table room asked for ahead, whatever the caller expects.
const MAX_HINT: u32 = 8_192;
/// H.264's limit on frames a decoder holds back (its DPB): a frame further
/// ahead of its turn than this cannot be a real reordering.
const MAX_REORDER: u32 = 16;
const MICROS_PER_SECOND: i128 = 1_000_000;

/// Where the bytes of the file go. In the browser it is a synchronous OPFS
/// handle; in a test it is [`MemSink`].
pub trait MuxSink {
    fn write_at(&mut self, offset: Bytes, data: &[u8]) -> Result<(), IoError>;
}

/// A muxer may write through a sink it does not own: `finalize` consumes the
/// muxer, and the caller keeps the sink.
impl<S: MuxSink + ?Sized> MuxSink for &mut S {
    fn write_at(&mut self, offset: Bytes, data: &[u8]) -> Result<(), IoError> {
        (**self).write_at(offset, data)
    }
}

pub struct VideoTrackSpec {
    pub width: Px,
    pub height: Px,
    pub avcc: Vec<u8>,
    pub frame_count_hint: u32,
}

pub struct AudioTrackSpec {
    pub sample_rate: Hz,
    pub channels: u32,
    pub asc: Vec<u8>,
}

/// A file held in memory, which grows on a write past its end. For tests only.
#[derive(Default)]
pub struct MemSink(pub Vec<u8>);

impl MuxSink for MemSink {
    fn write_at(&mut self, offset: Bytes, data: &[u8]) -> Result<(), IoError> {
        let start = usize::try_from(offset.get()).map_err(|_| IoError::OutOfBounds)?;
        let end = start.checked_add(data.len()).ok_or(IoError::OutOfBounds)?;
        if self.0.len() < end {
            self.0.resize(end, 0);
        }
        let target = self.0.get_mut(start..end).ok_or(IoError::OutOfBounds)?;
        target.copy_from_slice(data);
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Video,
    Audio,
}

/// The sizes of a track's samples, and the chunks they lie in.
struct TrackLog {
    sizes: Vec<u32>,
    chunks: Vec<Chunk>,
    per_chunk: u32,
}

impl TrackLog {
    fn new(per_chunk: u32, hint: u32) -> Self {
        let hint = hint.min(MAX_HINT) as usize;
        Self {
            sizes: Vec::with_capacity(hint),
            chunks: Vec::new(),
            per_chunk,
        }
    }

    fn count(&self) -> u32 {
        u32::try_from(self.sizes.len()).unwrap_or(u32::MAX)
    }

    /// Records a sample written at `offset`. It opens a chunk when the last
    /// one is full, or when `apart`: the other track wrote after it.
    fn push(&mut self, offset: u64, size: u32, apart: bool) {
        let open = self
            .chunks
            .last_mut()
            .filter(|chunk| !apart && chunk.samples < self.per_chunk);
        match open {
            Some(chunk) => chunk.samples = chunk.samples.saturating_add(1),
            None => self.chunks.push(Chunk { offset, samples: 1 }),
        }
        self.sizes.push(size);
    }
}

pub struct Mp4Muxer<S> {
    sink: S,
    width: u16,
    height: u16,
    avcc: Vec<u8>,
    asc: Vec<u8>,
    /// Where `moov` will go: the first byte after `ftyp`.
    moov_at: u64,
    mdat_at: u64,
    /// The first byte after everything written so far.
    end: u64,
    last: Option<Kind>,
    video: TrackLog,
    /// The numbers, from 1, of the keyframes.
    sync: Vec<u32>,
    /// Where each video sample is shown, as an output frame, in the order written.
    presented: Vec<u32>,
    /// Which output frames have arrived.
    seen: Vec<bool>,
    audio: TrackLog,
    /// Runs of (sample count, duration in ticks of 1/48,000 s).
    audio_runs: Vec<(u32, u32)>,
    audio_ticks: u64,
    /// The ticks of audio before time 0: what the encoder puts in front.
    priming: u32,
    last_audio_pts: Option<Micros>,
}

/// Microseconds as ticks of 1/48,000 s, to the nearest. A negative time is 0.
fn audio_ticks(micros: i64) -> u32 {
    let ticks = (i128::from(micros.max(0)) * i128::from(AUDIO_TIMESCALE) + MICROS_PER_SECOND / 2)
        / MICROS_PER_SECOND;
    u32::try_from(ticks).unwrap_or(u32::MAX)
}

/// `ticks` of `timescale` as ticks of the movie timescale, to the nearest.
fn movie_ticks(ticks: u64, timescale: u32) -> u32 {
    let per = u64::from(timescale.max(1));
    let scaled = ticks.saturating_mul(u64::from(MOVIE_TIMESCALE));
    u32::try_from(scaled.saturating_add(per / 2) / per).unwrap_or(u32::MAX)
}

impl<S: MuxSink> Mp4Muxer<S> {
    /// Writes `ftyp`, the space for `moov`, and the header of `mdat`.
    pub fn new(
        sink: S,
        video: VideoTrackSpec,
        audio: AudioTrackSpec,
    ) -> Result<Mp4Muxer<S>, MuxError> {
        if video.avcc.is_empty() {
            return Err(MuxError::BadConfig("avcc"));
        }
        if audio.asc.is_empty() {
            return Err(MuxError::BadConfig("asc"));
        }
        let size = |px: Px| u16::try_from(px.get()).ok().filter(|&px| px > 0);
        let (Some(width), Some(height)) = (size(video.width), size(video.height)) else {
            return Err(MuxError::BadConfig("size"));
        };
        if audio.sample_rate.get() != AUDIO_TIMESCALE {
            return Err(MuxError::BadConfig("sample rate"));
        }
        if audio.channels != AUDIO_CHANNELS {
            return Err(MuxError::BadConfig("channels"));
        }

        let ftyp = boxes::ftyp();
        let moov_at = ftyp.len() as u64;
        let mdat_at = moov_at.saturating_add(MOOV_RESERVE);
        let mut muxer = Mp4Muxer {
            sink,
            width,
            height,
            avcc: video.avcc,
            asc: audio.asc,
            moov_at,
            mdat_at,
            end: 0,
            last: None,
            video: TrackLog::new(VIDEO_CHUNK_SAMPLES, video.frame_count_hint),
            sync: Vec::new(),
            presented: Vec::new(),
            seen: Vec::new(),
            audio: TrackLog::new(AUDIO_CHUNK_SAMPLES, 0),
            audio_runs: Vec::new(),
            audio_ticks: 0,
            priming: 0,
            last_audio_pts: None,
        };
        muxer.append(&ftyp)?;
        // The reserve is written, not skipped: the file has no hole in it.
        let mut reserve = vec![0u8; MOOV_RESERVE as usize];
        if let Some(head) = reserve.get_mut(..8) {
            head.copy_from_slice(&boxes::free_header(MOOV_RESERVE as u32));
        }
        muxer.append(&reserve)?;
        muxer.append(&boxes::mdat_header(16))?;
        Ok(muxer)
    }

    /// `frame` is the output frame this sample is shown as, and samples come
    /// in the order they are decoded. The first is frame 0 and a keyframe; each
    /// frame comes once, at most `MAX_REORDER` ahead of its turn. `finalize`
    /// finds a frame that never came.
    pub fn add_video_sample(
        &mut self,
        data: &[u8],
        frame: FrameIdx,
        is_keyframe: bool,
    ) -> Result<(), MuxError> {
        let number = self.video.count();
        let shown = frame.get();
        let too_far = shown > number.saturating_add(MAX_REORDER);
        let first_elsewhere = number == 0 && shown != 0;
        let again = self.seen.get(shown as usize).copied().unwrap_or(false);
        if too_far || first_elsewhere || again {
            return Err(MuxError::OutOfOrder);
        }
        if number == 0 && !is_keyframe {
            return Err(MuxError::BadConfig("first frame"));
        }
        let (offset, size, apart) = self.append_sample(data, Kind::Video)?;
        self.video.push(offset, size, apart);
        self.presented.push(shown);
        let needed = (shown as usize).saturating_add(1).max(self.seen.len());
        self.seen.resize(needed, false);
        if let Some(arrived) = self.seen.get_mut(shown as usize) {
            *arrived = true;
        }
        if is_keyframe {
            self.sync.push(number.saturating_add(1));
        }
        Ok(())
    }

    /// `pts` only has to not go backwards: where a sample lies in time comes
    /// from the durations before it. A first `pts` below 0 is the length of
    /// what the encoder put in front, which a player is told to leave out.
    pub fn add_audio_sample(
        &mut self,
        data: &[u8],
        pts: Micros,
        duration: Micros,
    ) -> Result<(), MuxError> {
        match self.last_audio_pts {
            Some(last) if pts < last => return Err(MuxError::OutOfOrder),
            Some(_) => {}
            None => self.priming = audio_ticks(pts.get().saturating_neg()),
        }
        let (offset, size, apart) = self.append_sample(data, Kind::Audio)?;
        self.audio.push(offset, size, apart);
        self.last_audio_pts = Some(pts);

        let ticks = audio_ticks(duration.get());
        self.audio_ticks = self.audio_ticks.saturating_add(u64::from(ticks));
        if let Some((run, delta)) = self.audio_runs.last_mut()
            && *delta == ticks
        {
            *run = run.saturating_add(1);
        } else {
            self.audio_runs.push((1, ticks));
        }
        Ok(())
    }

    /// Writes the size of `mdat`, then `moov` into the space kept for it.
    /// Returns the size of the file.
    pub fn finalize(mut self) -> Result<Bytes, MuxError> {
        // Each frame came once, so one that never came shows as a frame beyond the count.
        if self.seen.len() != self.video.sizes.len() {
            return Err(MuxError::OutOfOrder);
        }
        let mdat_size = self.end.saturating_sub(self.mdat_at);
        self.sink
            .write_at(Bytes::new(self.mdat_at), &boxes::mdat_header(mdat_size))?;

        let moov = self.moov();
        let left = MOOV_RESERVE
            .checked_sub(moov.len() as u64)
            .ok_or(MuxError::MoovOverflow)?;
        // What is left over becomes a `free` box, which needs a header.
        let left = u32::try_from(left).map_err(|_| MuxError::MoovOverflow)?;
        if (1..8).contains(&left) {
            return Err(MuxError::MoovOverflow);
        }
        self.sink.write_at(Bytes::new(self.moov_at), &moov)?;
        if left > 0 {
            let at = self.moov_at.saturating_add(moov.len() as u64);
            self.sink
                .write_at(Bytes::new(at), &boxes::free_header(left))?;
        }
        Ok(Bytes::new(self.end))
    }

    fn append(&mut self, data: &[u8]) -> Result<(), MuxError> {
        self.sink.write_at(Bytes::new(self.end), data)?;
        self.end = self.end.saturating_add(data.len() as u64);
        Ok(())
    }

    /// Writes a sample. Returns where it lies, its size, and whether the
    /// other track wrote since this one last did.
    fn append_sample(&mut self, data: &[u8], kind: Kind) -> Result<(u64, u32, bool), MuxError> {
        let size = u32::try_from(data.len()).map_err(|_| MuxError::BadConfig("sample"))?;
        let offset = self.end;
        self.append(data)?;
        let apart = self.last != Some(kind);
        self.last = Some(kind);
        Ok((offset, size, apart))
    }

    /// When frames came out of the order they are shown: the `ctts` runs, and
    /// how far the first frame lies into the media, which an edit takes out.
    /// `None` when they did not, and then the track has neither.
    fn reordering(&self) -> Option<(Vec<(u32, u32)>, u32)> {
        let decoded = (0u64..).map(|number| number * u64::from(VIDEO_SAMPLE_DELTA));
        let shown = |frame: u32| u64::from(frame) * u64::from(VIDEO_SAMPLE_DELTA);
        let times = || self.presented.iter().zip(decoded.clone());
        let delay = times()
            .map(|(&frame, dts)| dts.saturating_sub(shown(frame)))
            .max()
            .unwrap_or(0);
        if delay == 0 {
            // No frame is shown before its decode time, so none is shown after it either.
            return None;
        }
        let mut runs: Vec<(u32, u32)> = Vec::new();
        for (&frame, dts) in times() {
            let offset = shown(frame).saturating_add(delay).saturating_sub(dts);
            let offset = u32::try_from(offset).unwrap_or(u32::MAX);
            if let Some((run, last)) = runs.last_mut()
                && *last == offset
            {
                *run = run.saturating_add(1);
            } else {
                runs.push((1, offset));
            }
        }
        Some((runs, u32::try_from(delay).unwrap_or(u32::MAX)))
    }

    fn moov(&self) -> Vec<u8> {
        let video_ticks = u64::from(self.video.count()) * u64::from(VIDEO_SAMPLE_DELTA);
        let video_duration = movie_ticks(video_ticks, VIDEO_TIMESCALE);
        // The audio a player shows: all of it but what lies before time 0.
        let shown_ticks = self.audio_ticks.saturating_sub(u64::from(self.priming));
        let audio_duration = movie_ticks(shown_ticks, AUDIO_TIMESCALE);
        let media = |ticks: u64| u32::try_from(ticks).unwrap_or(u32::MAX);

        let video_runs = match self.video.count() {
            0 => Vec::new(),
            count => vec![(count, VIDEO_SAMPLE_DELTA)],
        };
        let reordering = self.reordering();
        let video = boxes::trak(
            boxes::tkhd(1, video_duration, Some((self.width, self.height))),
            reordering
                .as_ref()
                .map(|&(_, delay)| boxes::edts(video_duration, delay)),
            boxes::mdia(
                boxes::mdhd(VIDEO_TIMESCALE, media(video_ticks)),
                b"vide",
                boxes::stbl(
                    boxes::stsd_video(self.width, self.height, &self.avcc),
                    &video_runs,
                    reordering.as_ref().map(|(runs, _)| runs.as_slice()),
                    Some(&self.sync),
                    &self.video.sizes,
                    &self.video.chunks,
                ),
            ),
        );
        let edit = (self.priming > 0).then(|| boxes::edts(audio_duration, self.priming));
        let audio = boxes::trak(
            boxes::tkhd(2, audio_duration, None),
            edit,
            boxes::mdia(
                boxes::mdhd(AUDIO_TIMESCALE, media(self.audio_ticks)),
                b"soun",
                boxes::stbl(
                    boxes::stsd_audio(AUDIO_TIMESCALE, AUDIO_CHANNELS as u16, &self.asc),
                    &self.audio_runs,
                    None,
                    None,
                    &self.audio.sizes,
                    &self.audio.chunks,
                ),
            ),
        );
        let duration = video_duration.max(audio_duration);
        boxes::container(
            b"moov",
            &[boxes::mvhd(MOVIE_TIMESCALE, duration, 3), video, audio],
        )
    }
}
