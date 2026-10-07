// Generated from crates/offcut-types by `sh scripts/gen-types.sh`. Do not edit.
// To change anything here, change the Rust source and run the script again.

/**
 * A position on the recording's timeline, in ms (the only timeline, INV-5).
 */
export type TimeMs = number & { readonly __unit: "TimeMs" };
/**
 * A length in ms.
 */
export type DurMs = number & { readonly __unit: "DurMs" };
/**
 * WebCodecs timestamps only.
 */
export type Micros = number & { readonly __unit: "Micros" };
/**
 * Output frame number at `OUTPUT_FPS`.
 */
export type FrameIdx = number & { readonly __unit: "FrameIdx" };
/**
 * PCM samples at a stated rate.
 */
export type SampleCount = number & { readonly __unit: "SampleCount" };
export type Hz = number & { readonly __unit: "Hz" };
/**
 * Frames per second x 1000 (59_940 = 59.94 fps).
 */
export type FpsMilli = number & { readonly __unit: "FpsMilli" };
export type Px = number & { readonly __unit: "Px" };
export type Bytes = number & { readonly __unit: "Bytes" };
export type BitsPerSec = number & { readonly __unit: "BitsPerSec" };
/**
 * Invariant `0.0..=1.0`, checked in `new` and when deserializing.
 */
export type Confidence = number & { readonly __unit: "Confidence" };
/**
 * Invariant: finite, checked in `new` and when deserializing.
 */
export type Lufs = number & { readonly __unit: "Lufs" };
/**
 * Invariant: finite, checked in `new` and when deserializing.
 */
export type Dbfs = number & { readonly __unit: "Dbfs" };
export type UnixSecs = number & { readonly __unit: "UnixSecs" };
export type ExportCount = number & { readonly __unit: "ExportCount" };
export type UsdCents = number & { readonly __unit: "UsdCents" };
/**
 * v4, created on import (in the browser).
 */
export type ClipId = string & { readonly __unit: "ClipId" };
/**
 * v7, created per export; the usage-receipt idempotency key.
 */
export type ExportId = string & { readonly __unit: "ExportId" };
export type UserId = string & { readonly __unit: "UserId" };
/**
 * Per-browser analytics id.
 */
export type AnonId = string & { readonly __unit: "AnonId" };
/**
 * Stable hash, TS §17.6. Serialized as a decimal string, because a JavaScript
 * number cannot hold 64 bits.
 */
export type EventId = string & { readonly __unit: "EventId" };
export type WordIdx = number & { readonly __unit: "WordIdx" };
export type SentenceIdx = number & { readonly __unit: "SentenceIdx" };
export type JobId = number & { readonly __unit: "JobId" };

/**
 * A stretch of the timeline. Invariant `start <= end`; build it with [`Span::new`].
 */
export type Span = { start: TimeMs, end: TimeMs, };

/**
 * Half-open: `start` is included, `end` is not.
 */
export type WordRange = { start: WordIdx, end: WordIdx, };

export type Rotation = "r0" | "r90" | "r180" | "r270";

/**
 * From the display size after rotation; square = `Landscape`.
 */
export type Orientation = "portrait" | "landscape";

export type ContainerKind = "mp4" | "mov" | "other";

export type VideoCodec = "h264" | "hevc" | "av1" | "vp9" | "pro_res" | "other";

export type AudioCodec = "aac" | "mp3" | "opus" | "pcm" | "other";

export type ProbeInfo = { container: ContainerKind, file_size: Bytes, duration: DurMs, video_tracks: number, audio_tracks: number, video: VideoTrackInfo | null, audio: AudioTrackInfo | null, };

export type VideoTrackInfo = { codec: VideoCodec,
/**
 * WebCodecs codec string, e.g. "avc1.640028".
 */
codec_string: string, coded_width: Px, coded_height: Px, rotation: Rotation, avg_fps: FpsMilli, max_fps: FpsMilli, is_vfr: boolean, frame_count: number, };

export type AudioTrackInfo = { codec: AudioCodec, codec_string: string, sample_rate: Hz, channels: number, };

/**
 * A validated clip; constructed only by `validate_probe()`.
 */
export type ClipInfo = { duration: DurMs,
/**
 * After rotation.
 */
display_width: Px,
/**
 * After rotation.
 */
display_height: Px, rotation: Rotation, orientation: Orientation, is_vfr: boolean, video_codec_string: string, audio_codec_string: string, audio_sample_rate: Hz, audio_channels: number, };

/**
 * Serialized as its `REJECT_*` code; also an analytics enum.
 */
export type RejectReason = "REJECT_CONTAINER" | "REJECT_CORRUPT" | "REJECT_NO_VIDEO" | "REJECT_NO_AUDIO" | "REJECT_MULTI_AUDIO_TRACK" | "REJECT_VIDEO_CODEC" | "REJECT_HEVC" | "REJECT_AUDIO_CODEC" | "REJECT_DURATION" | "REJECT_FILE_SIZE" | "REJECT_RESOLUTION" | "REJECT_FRAME_RATE" | "REJECT_DECODE_UNSUPPORTED" | "REJECT_NO_SPEECH";

export type Word = { text: string, start_ms: TimeMs, end_ms: TimeMs, confidence: Confidence, };

/**
 * `words` is a range, because sentences never overlap.
 */
export type Sentence = { words: WordRange, start_ms: TimeMs, end_ms: TimeMs, };

export type Unit = "none" | "usd" | "eur" | "gbp" | "inr" | "percent" | "milliseconds" | "seconds" | "minutes" | "hours" | "days" | "weeks" | "months" | "years" | "times" | "bytes" | "kilobytes" | "megabytes" | "gigabytes" | { "count": { noun: string, } };

/**
 * `display` is the caption form: "$10k", "800 ms", "10,000".
 */
export type Quantity = { value: number, unit: Unit, display: string, };

/**
 * A run of spoken words that is one quantity: "ten thousand" → 10,000.
 */
export type NormalizedSpan = { words: WordRange, quantity: Quantity, };

export type Transcript = { words: Array<Word>, sentences: Array<Sentence>, numbers: Array<NormalizedSpan>, model_version: string, };

export type EventKind = "number_reveal" | "list_reveal" | "from_to" | "keyword_pop";

/**
 * `at` is the time at which the item was spoken.
 */
export type ListItem = { ordinal: number, text: string, at: TimeMs, };

export type EventParams = { "kind": "number_reveal", value: Quantity, label: string | null, } | { "kind": "list_reveal", count: number, header: string, items: Array<ListItem>, } | { "kind": "from_to", from: Quantity, to: Quantity, label: string | null, } | { "kind": "keyword_pop", word: WordIdx, };

export type DetectedEvent = { id: EventId, kind: EventKind, span: Span, anchors: WordRange, params: EventParams,
/**
 * Always at least the kind's threshold; lower ones are never constructed.
 */
confidence: Confidence, enabled: boolean, };

/**
 * Both values are z-scores against the word's sentence.
 */
export type WordProsody = { energy_z: number, pitch_z: number, };

/**
 * `per_word.len()` equals `transcript.words.len()`.
 */
export type Prosody = { per_word: Array<WordProsody>, };

/**
 * Caption style (PS §12.2).
 */
export type StyleId = "clean" | "bold" | "tech";

/**
 * Horizontal crop position: -1.0 (left) ..= 1.0 (right); 0.0 is the centre.
 * Checked in `new` and when deserializing.
 */
export type CropOffset = number;

export type EditState = {
/**
 * Replacement text; "" hides the word; timing never changes.
 */
word_edits: { [key in WordIdx]: string },
/**
 * The enabled flag chosen by the user.
 */
event_overrides: { [key in EventId]: boolean }, style: StyleId, crop_offset: CropOffset, };

export type Plan = "free" | "creator";

export type ProfileKind = "preview" | "free" | "creator";

export type ExportProfile = { kind: ProfileKind, width: Px, height: Px, watermark: boolean, video_bitrate: BitsPerSec, audio_bitrate: BitsPerSec, };

export type ChangeSummary = {
/**
 * The enhancement chain ran (TS §18).
 */
voice_cleaned: boolean,
/**
 * Enabled KeywordPop events.
 */
captions_emphasized: number,
/**
 * Enabled NumberReveal + ListReveal + FromTo events.
 */
visual_moments: number, };

export type PipelineStage = "probe_audio" | "asr" | "audio_chain" | "detect_scene" | "render_encode" | "mux";

/**
 * Every `E_*` code, in the order of TS §11.2.
 */
export type ErrorCode = "E_MODEL_DOWNLOAD" | "E_MODEL_HASH" | "E_MODEL_STORAGE" | "E_STORAGE_QUOTA" | "E_STORAGE_IO" | "E_DECODE_AUDIO" | "E_DECODE_VIDEO" | "E_ASR_RUNTIME" | "E_ASR_OOM" | "E_DSP" | "E_GPU_INIT" | "E_GPU_LOST" | "E_ENCODE_VIDEO" | "E_ENCODE_AUDIO" | "E_MUX" | "E_WORKER_CRASH" | "E_NET_OFFLINE" | "E_NET_TIMEOUT" | "E_API_5XX" | "E_API_RATE_LIMITED" | "E_AUTH_LINK_INVALID" | "E_AUTH_LINK_EXPIRED" | "E_AUTH_SESSION_EXPIRED" | "E_AUTH_EMAIL_UNAVAILABLE" | "E_BILLING_UNAVAILABLE" | "E_BILLING_PENDING" | "E_ENTITLEMENT_INVALID" | "E_ENTITLEMENT_EXPIRED" | "E_INTERNAL";

/**
 * Why the browser cannot run Offcut, in the check order of TS §13.2.
 * The first failing check in this order is the reported reason.
 */
export type UnsupportedReason = "UNSUPPORTED_MOBILE" | "UNSUPPORTED_WEBCODECS" | "UNSUPPORTED_THREADS" | "UNSUPPORTED_WASM_SIMD" | "UNSUPPORTED_STORAGE" | "UNSUPPORTED_LOW_MEMORY" | "UNSUPPORTED_WEBGPU" | "UNSUPPORTED_H264_DECODE" | "UNSUPPORTED_H264_ENCODE" | "UNSUPPORTED_AAC_DECODE" | "UNSUPPORTED_AAC_ENCODE";

/**
 * Where a failure happened: the six pipeline stages, then five places outside the pipeline.
 */
export type FailureStage = "probe_audio" | "asr" | "audio_chain" | "detect_scene" | "render_encode" | "mux" | "import" | "model" | "preview" | "storage" | "api";

export const LIMITS = {
  MAX_CLIP_DURATION: 90000 as DurMs,
  MAX_FILE_SIZE: 500000000 as Bytes,
  MAX_LONG_SIDE: 1920 as Px,
  MAX_INPUT_FPS: 60000 as FpsMilli,
  INPUT_FPS_TOLERANCE: 500 as FpsMilli,
  OUTPUT_FPS: 30,
  CREATOR_WIDTH: 1080 as Px,
  CREATOR_HEIGHT: 1920 as Px,
  FREE_WIDTH: 720 as Px,
  FREE_HEIGHT: 1280 as Px,
  FREE_EXPORTS_PER_MONTH: 3 as ExportCount,
  CREATOR_FAIR_USE_PER_MONTH: 100 as ExportCount,
  ENTITLEMENT_OFFLINE_TTL_SECS: 604800,
  MIN_DEVICE_MEMORY_GB: 4,
  CAPABILITY_CHECK_BUDGET: 3000 as DurMs,
  DETECTOR_PRECISION_TARGET: 0.9,
  PREVIEW_MAX_DRIFT: 80 as DurMs,
  LOUDNESS_TOLERANCE_LU: 1,
  ANALYTICS_RETENTION_DAYS: 90,
  MIN_WORDS: 3,
  MAX_RECENT_CLIPS: 5,
} as const;

export const ERROR_CODES = [
  "E_MODEL_DOWNLOAD",
  "E_MODEL_HASH",
  "E_MODEL_STORAGE",
  "E_STORAGE_QUOTA",
  "E_STORAGE_IO",
  "E_DECODE_AUDIO",
  "E_DECODE_VIDEO",
  "E_ASR_RUNTIME",
  "E_ASR_OOM",
  "E_DSP",
  "E_GPU_INIT",
  "E_GPU_LOST",
  "E_ENCODE_VIDEO",
  "E_ENCODE_AUDIO",
  "E_MUX",
  "E_WORKER_CRASH",
  "E_NET_OFFLINE",
  "E_NET_TIMEOUT",
  "E_API_5XX",
  "E_API_RATE_LIMITED",
  "E_AUTH_LINK_INVALID",
  "E_AUTH_LINK_EXPIRED",
  "E_AUTH_SESSION_EXPIRED",
  "E_AUTH_EMAIL_UNAVAILABLE",
  "E_BILLING_UNAVAILABLE",
  "E_BILLING_PENDING",
  "E_ENTITLEMENT_INVALID",
  "E_ENTITLEMENT_EXPIRED",
  "E_INTERNAL",
] as const;

export const REJECT_REASONS = [
  "REJECT_CONTAINER",
  "REJECT_CORRUPT",
  "REJECT_NO_VIDEO",
  "REJECT_NO_AUDIO",
  "REJECT_MULTI_AUDIO_TRACK",
  "REJECT_VIDEO_CODEC",
  "REJECT_HEVC",
  "REJECT_AUDIO_CODEC",
  "REJECT_DURATION",
  "REJECT_FILE_SIZE",
  "REJECT_RESOLUTION",
  "REJECT_FRAME_RATE",
  "REJECT_DECODE_UNSUPPORTED",
  "REJECT_NO_SPEECH",
] as const;

export const UNSUPPORTED_REASONS = [
  "UNSUPPORTED_MOBILE",
  "UNSUPPORTED_WEBCODECS",
  "UNSUPPORTED_THREADS",
  "UNSUPPORTED_WASM_SIMD",
  "UNSUPPORTED_STORAGE",
  "UNSUPPORTED_LOW_MEMORY",
  "UNSUPPORTED_WEBGPU",
  "UNSUPPORTED_H264_DECODE",
  "UNSUPPORTED_H264_ENCODE",
  "UNSUPPORTED_AAC_DECODE",
  "UNSUPPORTED_AAC_ENCODE",
] as const;
