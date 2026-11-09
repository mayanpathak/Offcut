//! V6.1: the long path's exports (v6.1implementation D-399, D-401). A dropped
//! `File` is read where it is, through any object with `size()` and
//! `readAt(offset, length)` (`fileRandomAccess` in the media worker): nothing
//! is copied. `route_probe` decides the path; `StreamResampler` resamples a
//! long track in pieces. `open_demuxer`, `probe_and_validate` and the OPFS
//! reader of `media_api.rs` are unchanged.
//!
//! As read (P173): routing is one pure export over a `ProbeInfo` and a
//! `VideoDetail`, so the same call serves the clip page's OPFS handle and the
//! long page's reader; and a source opens with `Demuxer::open_source`, which
//! expands 45 minutes of samples (`open` stops at 20,000, D-60).

use js_sys::{Object, Reflect, Uint8Array};
use offcut_dsp::resample::StreamResampler;
use offcut_mp4::demux::Demuxer;
use offcut_mp4::probe::{probe, video_detail};
use offcut_mp4::reader::RandomAccess;
use offcut_mp4::route::route_probe as route;
use offcut_mp4::{ContainerError, IoError};
use offcut_types::{
    Bytes, Entry, Hz, ProbeInfo, RejectReason, Route, RouteFlags, RouteReject, SourceErrorCode,
    VideoDetail,
};
use serde::Serialize;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{to_js, to_plain};

#[wasm_bindgen]
extern "C" {
    /// Any object with these two methods: the media worker's `fileRandomAccess(file)`.
    pub type JsByteReader;

    #[wasm_bindgen(method, structural)]
    fn size(this: &JsByteReader) -> f64;

    /// `length` bytes from `offset`; throws when the file cannot be read.
    #[wasm_bindgen(method, structural, catch, js_name = readAt)]
    fn read_at(this: &JsByteReader, offset: f64, length: u32) -> Result<Uint8Array, JsValue>;
}

/// The reader behind a source's demuxer. The JavaScript side keeps its own
/// block cache (D-400), so each read here is one call.
struct ReaderAccess {
    reader: JsByteReader,
    len: Bytes,
}

impl RandomAccess for ReaderAccess {
    fn len(&self) -> Bytes {
        self.len
    }

    fn read_at(&mut self, offset: Bytes, buf: &mut [u8]) -> Result<(), IoError> {
        if buf.is_empty() {
            return Ok(());
        }
        let length = u32::try_from(buf.len()).map_err(|_| IoError::Read)?;
        let bytes = self
            .reader
            .read_at(offset.get() as f64, length)
            .map_err(|_| IoError::Read)?;
        if bytes.length() != length {
            return Err(IoError::Read);
        }
        bytes.copy_to(buf);
        Ok(())
    }
}

/// `{ code, detail }` with a V6.1 code (`E_SOURCE_READ`, `E_SOURCE_AUDIO`).
#[derive(Serialize)]
struct SourceFailure {
    code: SourceErrorCode,
    detail: &'static str,
}

fn source_failure(code: SourceErrorCode, detail: &'static str) -> JsValue {
    to_plain(&SourceFailure { code, detail }).unwrap_or_else(|_| JsValue::from_str("E_INTERNAL"))
}

#[derive(Serialize)]
struct Rejected {
    rejected: RejectReason,
}

/// What a source that does not open becomes: a rejection for the file's own
/// structure, `E_SOURCE_READ` when its bytes cannot be read (moved, deleted).
fn open_failure(error: ContainerError) -> JsValue {
    let rejected = |rejected| to_js(&Rejected { rejected }).unwrap_or_else(|e| e);
    match error {
        ContainerError::NotIsoBmff | ContainerError::Fragmented => {
            rejected(RejectReason::Container)
        }
        ContainerError::Truncated
        | ContainerError::Malformed(_)
        | ContainerError::Unsupported(_) => rejected(RejectReason::Corrupt),
        ContainerError::Io(_) => source_failure(SourceErrorCode::SourceRead, "Read"),
    }
}

/// A read of a sample of an accepted source: its bytes gone is `E_SOURCE_READ`,
/// anything else wrong with it `E_SOURCE_AUDIO`.
fn sample_failure(error: ContainerError) -> JsValue {
    match error {
        ContainerError::Io(_) => source_failure(SourceErrorCode::SourceRead, "Read"),
        ContainerError::NotIsoBmff => source_failure(SourceErrorCode::SourceAudio, "NotIsoBmff"),
        ContainerError::Fragmented => source_failure(SourceErrorCode::SourceAudio, "Fragmented"),
        ContainerError::Truncated => source_failure(SourceErrorCode::SourceAudio, "Truncated"),
        ContainerError::Malformed(_) => source_failure(SourceErrorCode::SourceAudio, "Malformed"),
        ContainerError::Unsupported(_) => {
            source_failure(SourceErrorCode::SourceAudio, "Unsupported")
        }
    }
}

#[wasm_bindgen]
pub struct SourceDemuxerHandle {
    demuxer: Demuxer<ReaderAccess>,
    sample: Vec<u8>,
}

/// Opens a dropped file through `reader`, in place. Throws `{ rejected }` for a
/// file this version does not read, `{ code: "E_SOURCE_READ" }` when it cannot be read.
#[wasm_bindgen]
pub fn open_demuxer_reader(reader: JsByteReader) -> Result<SourceDemuxerHandle, JsValue> {
    let size = reader.size();
    if !size.is_finite() || size < 0.0 {
        return Err(source_failure(SourceErrorCode::SourceRead, "Size"));
    }
    let access = ReaderAccess {
        reader,
        len: Bytes::new(size as u64),
    };
    Demuxer::open_source(access)
        .map(|demuxer| SourceDemuxerHandle {
            demuxer,
            sample: Vec::new(),
        })
        .map_err(open_failure)
}

#[wasm_bindgen]
impl SourceDemuxerHandle {
    pub fn probe(&self, file_size: f64) -> Result<JsValue, JsValue> {
        to_js(&probe(&self.demuxer, Bytes::new(file_size as u64)))
    }

    /// The `VideoDetail` (`is_hdr`, `hevc_codec_string`).
    pub fn video_detail(&mut self) -> Result<JsValue, JsValue> {
        to_js(&video_detail(&mut self.demuxer))
    }

    /// The raw `hvcC` payload of an HEVC track; `undefined` for any other.
    pub fn hevc_description(&mut self) -> Option<Vec<u8>> {
        self.demuxer
            .video_entry_boxes()
            .ok()
            .and_then(|boxes| boxes.hvcc)
    }

    /// The raw `avcC` payload of an H.264 track.
    pub fn video_description(&self) -> Option<Vec<u8>> {
        self.demuxer.video_decoder_description().map(<[u8]>::to_vec)
    }

    /// The first audio track's `AudioSpecificConfig`.
    pub fn audio_description(&self) -> Option<Vec<u8>> {
        self.demuxer.audio_decoder_description().map(<[u8]>::to_vec)
    }

    pub fn audio_sample_count(&self) -> u32 {
        self.demuxer.audio_sample_count()
    }

    /// When sample `index` is presented, in microseconds on the source's
    /// timeline (negative before the first video frame); `undefined` past the end.
    pub fn audio_sample_time_us(&self, index: u32) -> Option<f64> {
        self.demuxer
            .audio_sample_meta(index)
            .map(|meta| meta.pts.get() as f64)
    }

    /// `{ data, ptsUs, durationUs }`, as `DemuxerHandle.read_audio_sample`.
    pub fn read_audio_sample(&mut self, index: u32) -> Result<JsValue, JsValue> {
        let meta = self
            .demuxer
            .read_audio_sample(index, &mut self.sample)
            .map_err(sample_failure)?;
        let fields = [
            (
                "data",
                JsValue::from(Uint8Array::from(self.sample.as_slice())),
            ),
            ("ptsUs", JsValue::from_f64(meta.pts.get() as f64)),
            ("durationUs", JsValue::from_f64(meta.duration.get() as f64)),
        ];
        let sample = Object::new();
        for (name, value) in fields {
            Reflect::set(&sample, &JsValue::from_str(name), &value)
                .map_err(|_| crate::failure(offcut_types::ErrorCode::Internal, "Serialize"))?;
        }
        Ok(sample.into())
    }
}

/// `{ clip: ClipInfo }`, `{ long: SourceInfo }` or `{ rejected: RouteReject }`.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum Routed {
    Clip(offcut_types::ClipInfo),
    Long(offcut_types::SourceInfo),
    Rejected(RouteReject),
}

fn routed(result: Result<Route, RouteReject>) -> Routed {
    match result {
        Ok(Route::Clip(clip)) => Routed::Clip(clip),
        Ok(Route::Long(source)) => Routed::Long(source),
        Err(rejected) => Routed::Rejected(rejected),
    }
}

/// `route_probe` over a probe and a detail from either handle (TS §43.1).
#[wasm_bindgen]
pub fn route_probe(
    probe: JsValue,
    detail: JsValue,
    decode_supported: bool,
    entry: JsValue,
    flags: JsValue,
) -> Result<JsValue, JsValue> {
    let bad = |_| crate::failure(offcut_types::ErrorCode::Internal, "Argument");
    let probe: ProbeInfo = serde_wasm_bindgen::from_value(probe).map_err(bad)?;
    let detail: VideoDetail = serde_wasm_bindgen::from_value(detail).map_err(bad)?;
    let entry: Entry = serde_wasm_bindgen::from_value(entry).map_err(bad)?;
    let flags: RouteFlags = serde_wasm_bindgen::from_value(flags).map_err(bad)?;
    to_js(&routed(route(
        &probe,
        &detail,
        decode_supported,
        entry,
        flags,
    )))
}

/// `StreamResampler` for JavaScript (D-401): `push` a decoded chunk, get what
/// it completes; `finish` with the total input count once every chunk is in.
#[wasm_bindgen(js_name = StreamResampler)]
pub struct StreamResamplerHandle {
    inner: StreamResampler,
}

#[wasm_bindgen(js_class = StreamResampler)]
impl StreamResamplerHandle {
    /// Throws `{ code: "E_SOURCE_AUDIO" }` for a rate of 0.
    #[wasm_bindgen(constructor)]
    pub fn new(from_hz: u32, to_hz: u32) -> Result<StreamResamplerHandle, JsValue> {
        StreamResampler::new(Hz::new(from_hz), Hz::new(to_hz))
            .map(|inner| StreamResamplerHandle { inner })
            .ok_or_else(|| source_failure(SourceErrorCode::SourceAudio, "SampleRate"))
    }

    pub fn push(&mut self, input: &[f32]) -> Vec<f32> {
        self.inner.push(input)
    }

    pub fn finish(self, total_input: f64) -> Vec<f32> {
        let total = if total_input.is_finite() && total_input > 0.0 {
            total_input as usize
        } else {
            0
        };
        self.inner.finish(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use offcut_types::SourceReject;

    #[test]
    fn a_route_serializes_as_the_worker_protocol_reads_it() {
        let rejected = routed(Err(RouteReject::Clip(RejectReason::Duration)));
        assert_eq!(
            serde_json::to_string(&rejected).unwrap(),
            r#"{"rejected":"REJECT_DURATION"}"#
        );
        let hdr = routed(Err(RouteReject::Source(SourceReject::Hdr)));
        assert_eq!(
            serde_json::to_string(&hdr).unwrap(),
            r#"{"rejected":"REJECT_HDR"}"#
        );
    }
}
