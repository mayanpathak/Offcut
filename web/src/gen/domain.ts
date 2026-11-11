// Generated from crates/offcut-types by `sh scripts/gen-types.sh`. Do not edit.
// To change anything here, change the Rust source and run the script again.

/**
 * A position on the recording's timeline, in ms: what was said, when.
 */
export type TimeMs = number & { readonly __unit: "TimeMs" };
/**
 * A position on the program's timeline, in ms: the export's time, which the
 * scene, the clock and the frame index run on (D-125). In V4 the time map is
 * always the identity, so the two numbers are equal.
 */
export type OutMs = number & { readonly __unit: "OutMs" };
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
 * A stretch of the program's timeline. Invariant `start <= end`; build it
 * with [`OutSpan::new`].
 */
export type OutSpan = { start: OutMs, end: OutMs, };

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

/**
 * The kind of a detected event, and from V4 the kind of a plan item
 * (`PlanItem.kind`, v4implementation D-118): there is no second kind enum.
 * The four variants have the codes 0-3 of the event-id hash (D-47), fixed
 * forever. A release that draws a new kind appends a variant with the next
 * code; none is moved or removed. The order is the code order (a key of the
 * plan crate's per-kind limits).
 */
export type EventKind = "number_reveal" | "list_reveal" | "from_to" | "keyword_pop" | "emphasis" | "icon_pop" | "punch_in" | "emoji" | "hook_title";

/**
 * `at` is the time at which the item was spoken.
 */
export type ListItem = { ordinal: number, text: string, at: TimeMs, };

export type EventParams = { "kind": "number_reveal", value: Quantity, label: string | null, } | { "kind": "list_reveal", count: number, header: string, items: Array<ListItem>, } | { "kind": "from_to", from: Quantity, to: Quantity, label: string | null, } | { "kind": "keyword_pop", word: WordIdx, } | { "kind": "emphasis", word: WordIdx, treatment: Treatment, size: EmphasisSize, } | { "kind": "icon_pop", word: WordIdx, icon: IconId, } | { "kind": "punch_in", scale: ZoomScale, motion: ZoomMotion, } | { "kind": "emoji", word: WordIdx, emoji: EmojiId, place: EmojiPlace, } | { "kind": "hook_title", text: string, accent: number, };

export type DetectedEvent = { id: EventId, kind: EventKind, span: Span, anchors: WordRange, params: EventParams,
/**
 * Always at least the kind's threshold; lower ones are never constructed.
 */
confidence: Confidence, enabled: boolean, };

export type EditPlan = {
/**
 * `PLAN_FORMAT`.
 */
format: number,
/**
 * The `CATALOG_VERSION` the plan was produced against.
 */
catalog: number, source: PlanSource,
/**
 * Empty until V6.3.
 */
cuts: Array<Cut>, items: Array<PlanItem>,
/**
 * Empty until V6.2.
 */
chapters: Array<Chapter>,
/**
 * Empty until V6.2.
 */
extras: TextExtras,
/**
 * `None` until V6.7.
 */
music: MusicCue | null, };

/**
 * Who wrote the plan: set by the producer, never by a model.
 */
export type PlanSource = { "kind": "rules" } | { "kind": "model", provider: string, model: string, };

export type Cut = { "kind": "lead", keep: DurMs, } | { "kind": "trail", keep: DurMs, } | { "kind": "gap", after: WordIdx, keep: DurMs, } | { "kind": "keep_gap", after: WordIdx, } | { "kind": "words", range: WordRange, why: CutWhy, };

export type CutWhy = "filler" | "false_start";

/**
 * One visual of the plan. `kind` is the event kind (D-118); `params` repeat
 * it as their tag, and validation checks that the two agree.
 */
export type PlanItem = { id: EventId, kind: EventKind, anchors: WordRange, params: ItemParams,
/**
 * `Some` for an item of the rules (INV-6), `None` for one of a model.
 */
confidence: Confidence | null, };

/**
 * What an item shows. The four kinds point at words; `Detected` carries the
 * detector's own reading and is written by the rules producer only (D-120).
 */
export type ItemParams = { "kind": "number_reveal", "params": { value: QuantityRef, label: Label | null, } } | { "kind": "list_reveal", "params": { header: Label | null, items: Array<ListPoint>, } } | { "kind": "from_to", "params": { from: QuantityRef, to: QuantityRef, label: Label | null, } } | { "kind": "keyword_pop", "params": { word: WordIdx, } } | { "kind": "detected", "params": EventParams } | { "kind": "emphasis", "params": { word: WordIdx, treatment: Treatment, size: EmphasisSize, } } | { "kind": "icon_pop", "params": { word: WordIdx, icon: IconId, } } | { "kind": "punch_in", "params": { scale: ZoomScale, motion: ZoomMotion, } } | { "kind": "emoji", "params": { word: WordIdx, emoji: EmojiId, place: EmojiPlace, } } | { "kind": "hook_title", "params": { accent: number, } };

export type ListPoint = { at: WordIdx, label: Label, };

/**
 * A number as a pointer to the words that say it; `validate` reads the
 * number from them (D-121). One field, so that a later form can be added
 * beside `words` without breaking format 1.
 */
export type QuantityRef = { words: WordRange, };

/**
 * Text a plan shows. Its length and digit rules are `validate`'s (TS §40.4 V-4).
 */
export type Label = string;

export type Chapter = { starts_at: WordIdx, title: Label, };

/**
 * Text around the video. Every field may be left out (empty until V6.2).
 */
export type TextExtras = { hook: Label | null, post_caption: string | null, hashtags: Array<Label>, };

export type MusicEnergy = "calm" | "steady" | "upbeat";

export type MusicCue = { fits: boolean, energy: MusicEnergy, };

/**
 * What `offcut-plan`'s `validate` left out of a plan, and why (TS §40.4).
 * `kind` is `None` for a cut, a chapter, a text extra or a music cue. No
 * content: it may become an analytics event (V6.2).
 */
export type Dropped = { kind: EventKind | null, reason: DropReason, };

export type DropReason = "unknown_kind" | "wrong_producer" | "params_mismatch" | "anchors_out_of_range" | "anchor_rule" | "label_too_long" | "label_digits" | "number_unresolved" | "over_limit" | "too_close" | "zone_taken" | "not_in_this_build" | "density" | "hook_missing";

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

/**
 * Where the caption block's centre sits: 0.0 at the top of the band captions
 * may use, 1.0 at its bottom (v4implementation D-171). Checked in `new` and
 * when deserializing.
 */
export type CaptionPlacement = number;

/**
 * REOPENED in V4 (D-169, D-171): the last two fields. Their serde defaults
 * make a record V3 wrote read as V4's default.
 */
export type EditState = {
/**
 * Replacement text; "" hides the word; timing never changes.
 */
word_edits: { [key in WordIdx]: string },
/**
 * The enabled flag chosen by the user.
 */
event_overrides: { [key in EventId]: boolean }, style: StyleId, crop_offset: CropOffset, caption_position: CaptionPlacement,
/**
 * Numbers in the captions in the accent colour (D-169).
 */
highlight_numbers: boolean,
/**
 * REOPENED in V6.2 (v6.2implementation D-485, DC-16): the Enhance
 * settings. Left out of the JSON when it is `Enhance::v5()`, so a state
 * without a choice serializes as V6.1 wrote it, and a record without the
 * field reads as `v5()`.
 */
enhance?: Enhance, };

/**
 * One stretch of the program, in program order.
 */
export type Segment = { "kind": "source", src: Span, } | { "kind": "hold", at: TimeMs, dur: DurMs, };

/**
 * The program as a list of segments. The field is private: `identity` is the
 * one constructor in V4, and deserialization checks what it reads.
 */
export type TimeMap = { segments: Array<Segment>, };

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

/**
 * The page a file was dropped on (v1.4).
 */
export type Entry = "clip" | "long";

/**
 * The build's flags that change routing (`features.ts`); `hdr_ok` stays false in V6.1.
 */
export type RouteFlags = { long_video: boolean, high_res: boolean, hdr_ok: boolean, };

/**
 * What the video's sample entry says beyond `ProbeInfo` (D-396, D-397).
 */
export type VideoDetail = {
/**
 * From `colr`: `nclx` with transfer characteristics 16 (PQ) or 18 (HLG).
 */
is_hdr: boolean,
/**
 * The WebCodecs string built from `hvcC` (e.g. "hvc1.1.6.L150.90"); none for other codecs.
 */
hevc_codec_string: string | null, };

/**
 * A long source; constructed only by `route_probe`.
 */
export type SourceInfo = { duration: DurMs,
/**
 * After rotation.
 */
display_width: Px,
/**
 * After rotation.
 */
display_height: Px, rotation: Rotation, orientation: Orientation, is_vfr: boolean, is_hdr: boolean, avg_fps: FpsMilli, video_codec: VideoCodec, video_codec_string: string, audio_codec_string: string, audio_sample_rate: Hz, audio_channels: number,
/**
 * The track the long path reads: the first audio track in file order (P171).
 */
audio_track: number,
/**
 * Every audio track; the long path accepts more than one.
 */
audio_tracks: number, };

/**
 * Which path a file takes.
 */
export type Route = { "clip": ClipInfo } | { "long": SourceInfo };

/**
 * The reasons V6.1 adds to the V5 list, serialized as their `REJECT_*` codes.
 */
export type SourceReject = "REJECT_HDR";

/**
 * Why `route_probe` refuses a file: one V5 reason, or one V6.1 adds. It
 * serializes as the bare code, so the two sets read as one list.
 */
export type RouteReject = RejectReason | SourceReject;

/**
 * The error codes the long path adds (TS §43.9), serialized as their `E_*` codes.
 */
export type SourceErrorCode = "E_SOURCE_READ" | "E_SOURCE_AUDIO";

/**
 * How an `Emphasis` marks its word in the caption (TS §47.3).
 */
export type Treatment = "colour" | "underline" | "highlight";

/**
 * `Punch` shows the word alone, large (TS §47.3; at most one per 6 s, D-452).
 */
export type EmphasisSize = "normal" | "punch";

export type ZoomScale = "mild" | "strong";

/**
 * `Snap` jumps in on one strong word; `Push` eases in across a sentence (TS §47.2).
 */
export type ZoomMotion = "snap" | "push";

/**
 * Where an `Emoji` sits against its word.
 */
export type EmojiPlace = "beside" | "under";

/**
 * An icon of the bundle (24; D-495).
 */
export type IconId = "trending_up" | "trending_down" | "bar_chart" | "dollar" | "wallet" | "shopping_cart" | "briefcase" | "clock" | "calendar" | "lightbulb" | "target" | "check" | "cross" | "alert" | "arrow_right" | "heart" | "star" | "book" | "lock" | "shield" | "users" | "smartphone" | "laptop" | "mail";

/**
 * An emoji of the bundle (48; D-495).
 */
export type EmojiId = "laughing" | "heart_eyes" | "star_struck" | "mind_blown" | "thinking" | "shocked" | "crying" | "cool" | "partying" | "skeptical" | "eyes" | "angry" | "nervous" | "thumbs_up" | "thumbs_down" | "clap" | "raised_hands" | "muscle" | "folded_hands" | "handshake" | "money_bag" | "banknote" | "money_wings" | "gem" | "chart_up" | "chart_down" | "trophy" | "light_bulb" | "brain" | "target" | "crown" | "rocket" | "fire" | "hundred" | "sparkles" | "lightning" | "star" | "collision" | "party_popper" | "check" | "cross" | "warning" | "question" | "exclamation" | "alarm_clock" | "hourglass" | "heart" | "broken_heart";

/**
 * What the voice stage does.
 */
export type VoiceMode = "clean" | "level";

export type Enhance = { voice: VoiceMode,
/**
 * The camera drift and the rules' zooms (TS §47.2).
 */
motion: boolean, };

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

export const PLAN_FORMAT = 1;
export const CATALOG_VERSION = 1;

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

export const SOURCE_LIMITS = {
  MAX_SOURCE_DURATION: 2700000 as DurMs,
  MAX_INPUT_LONG_SIDE: 3840 as Px,
  MAX_FILE_SIZE_HIGH_RES: 16000000000 as Bytes,
  CLOUD_WINDOW_S: 90,
  CLOUD_OVERLAP_S: 10,
  MAX_SOURCE_WINDOWS: 35,
} as const;

export const SOURCE_REJECT_REASONS = [
  "REJECT_HDR",
] as const;

export const SOURCE_ERROR_CODES = [
  "E_SOURCE_READ",
  "E_SOURCE_AUDIO",
] as const;
