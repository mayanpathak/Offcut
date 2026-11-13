// The message types of all four workers (TS §14.1, §14.2). Types only: this
// file has no function and no constant. It imports from gen/ and nothing else.

import type { CloudWord } from "../gen/api";
import type {
  ChangeSummary,
  ClipId,
  ClipInfo,
  DetectedEvent,
  Dropped,
  DurMs,
  EditPlan,
  EditState,
  ErrorCode,
  EventKind,
  ExportId,
  ExportProfile,
  FailureStage,
  JobId,
  Lufs,
  OutMs,
  OutSpan,
  PipelineStage,
  Prosody,
  RejectReason,
  Sentence,
  SentenceIdx,
  TimeMap,
  Transcript,
  Word,
  WordIdx,
} from "../gen/domain";
import type { Entry, RouteFlags, RouteReject, SourceErrorCode, SourceInfo } from "../gen/domain";
// V6.2 (D-487): the voice a program is assembled on.
import type { VoiceMode } from "../gen/domain";

export type { FailureStage } from "../gen/domain";

// --- Envelope (TS §14.1) -----------------------------------------------------

export type WorkerKind = "media" | "asr" | "audio" | "render";

export type Req<M extends string, P> = { type: "req"; jobId: JobId; method: M; params: P };

export type Res<R> =
  | { type: "res"; jobId: JobId; ok: true; result: R }
  | { type: "res"; jobId: JobId; ok: false; failure: AppFailure }
  | { type: "res"; jobId: JobId; cancelled: true };

export type Progress = {
  type: "progress";
  jobId: JobId;
  stage: PipelineStage | "import" | "model";
  done: number;
  total: number;
  feed?: FeedLine;
};

export type Cancel = { type: "cancel"; jobId: JobId };

/** A line of the processing feed (J6). It is rendered through messages.ts. */
export type FeedLine =
  | { kind: "transcribing" }
  | { kind: "cleaning_voice" }
  // `display` stays on the device.
  | { kind: "event_found"; eventKind: EventKind; display: string; displayTo?: string };

/** `detail` is shown only in development builds and is never sent anywhere. */
export type AppFailure = {
  code: ErrorCode;
  stage: FailureStage;
  retryable: boolean;
  detail?: string;
};

// --- Methods (TS §14.2) ------------------------------------------------------

export interface MediaWorkerApi {
  importAndProbe(p: { clipId: ClipId; file: File }): Promise<{ ok: ClipInfo } | { rejected: RejectReason }>;
  /** Both arrays are transferred. */
  extractAudio(p: { clipId: ClipId }): Promise<{ pcm48: Float32Array; pcm16: Float32Array }>;
  /** V6.1 (§6.4): which path a dropped file takes, read in place; nothing is copied. */
  routeFile(p: { file: File; entry: Entry; flags: RouteFlags }): Promise<Routed | SourceFailed>;
  /** V6.1 (D-427): `importAndProbe` for the clip page with `high_res` on: copy, then `route_probe`. */
  importAndRoute(p: { clipId: ClipId; file: File; flags: RouteFlags }): Promise<{ ok: ClipInfo } | { rejected: RouteReject }>;
  /**
   * V6.1 (§7.2): the source's sound as `sources/<id>/pcm16.f32`, from `fromSeconds` on. Progress is
   * `{ stage: "import", done, total }` in milliseconds of sound; `done` is flushed to disk.
   */
  extractSourcePcm16(p: {
    sourceId: SourceId;
    file: File;
    info: SourceInfo;
    fromSeconds: number;
  }): Promise<{ done: true; levelP95: number } | SourceFailed>;
}

/** V6.1: a long source's id (a UUID); `persistence/schema.ts` declares the same brand. */
export type SourceId = string & { readonly __unit: "SourceId" };

/** V6.1: which path a file takes, or why it is refused. */
export type Routed = { clip: ClipInfo } | { long: SourceInfo } | { rejected: RouteReject };

/**
 * V6.1: a source file that cannot be read (`E_SOURCE_READ`) or whose sound cannot be decoded
 * (`E_SOURCE_AUDIO`), returned as a value: `rpc.ts` carries only V5's `ErrorCode`s as failures.
 */
export type SourceFailed = { failed: SourceErrorCode; detail: string };

export interface AsrWorkerApi {
  load(p: { modelId: string; backend: "webgpu" | "wasm" }): Promise<{ backend: "webgpu" | "wasm" }>;
  transcribe(p: { pcm16: Float32Array }): Promise<{ ok: Transcript } | { rejected: "NoSpeech" }>;
  unload(): Promise<void>;
  /** Cloud transcription (cloudimplementation §8.3): `pcm16` as an Ogg Opus file. No model is involved. */
  encodeForCloud(p: { pcm16: Float32Array }): Promise<{ ogg: Uint8Array; durationMs: number } | { unsupported: true }>;
  /** The provider's words as a transcript, as `transcribe` builds one, after the silence guard (D-382). */
  fromCloud(p: { words: CloudWord[]; pcm16: Float32Array; modelVersion: string }): Promise<{ ok: Transcript } | { rejected: "NoSpeech" }>;
  /**
   * V6.1 (D-406): one window's words through the silence guard, judged against the source's level
   * (D-407), as words relative to the window: not merged, not post-processed.
   */
  cloudWindowWords(p: { words: CloudWord[]; pcm16: Float32Array; refLevel: number }): Promise<{ words: WindowWord[]; lengthMs: number }>;
  /** V6.1 (D-408): the windows merged (`mergeWindows`), placed (`postProcess`) and normalized. */
  buildMasterTranscript(p: {
    windows: { offsetMs: number; lengthMs: number; words: WindowWord[] }[];
    durationMs: number;
    modelVersion: string;
  }): Promise<{ ok: Transcript } | { rejected: "NoSpeech" }>;
}

/** V6.1: a word as the recognizer gave it, times in whole ms from its window's start (`RawWord`). */
export type WindowWord = { text: string; startMs: number; endMs: number; confidence: number };

// The audio worker has no method that detects silence or plans a cut. Only
// `assemble` applies a time map, and in V4 the only map is the identity.
export interface AudioWorkerApi {
  /**
   * `out48` is kept in the worker; `out48.length === pcm48.length`. V6.2
   * (D-487): with `level`, the levelled voice of the same `pcm48` is kept
   * too, for `takeLevel`; the page sets it with `motion` on.
   */
  runChain(p: { pcm48: Float32Array; level?: boolean }): Promise<{ loudness: Lufs }>;
  /** `out48` is transferred out, unmodified. */
  measureProsody(p: { words: Word[]; sentences: Sentence[] }): Promise<{ out48: Float32Array; prosody: Prosody }>;
  /** `out48` is transferred in and `prog48` out; the worker keeps nothing (D-127). */
  assemble(p: { out48: Float32Array; timeMap: TimeMap; speech: OutSpan[] }): Promise<{ prog48: Float32Array; loudness: Lufs }>;
  /** Fire-and-forget: drops any audio the worker holds (D-128). */
  release(): void;
  /**
   * V6.2 (D-487): `lvl48`, made by `runChain` from the same `pcm48` when the
   * build has `motion` on, transferred out; `null` when none was made. The
   * worker keeps nothing after.
   */
  takeLevel(): Promise<{ lvl48: Float32Array | null }>;
  /** V6.2 (D-487): `assemble` on the voice `voice` names; `voice48` is transferred in and `prog48` out. */
  assembleVoice(p: {
    voice48: Float32Array;
    voice: VoiceMode;
    timeMap: TimeMap;
    speech: OutSpan[];
  }): Promise<{ prog48: Float32Array; loudness: Lufs }>;
}

export interface RenderWorkerApi {
  openSession(p: { clipId: ClipId; clipInfo: ClipInfo }): Promise<void>;
  /** With `wordEdits`, the events of the transcript as edited (a replace-all, D-175). */
  detect(p: { transcript: Transcript; prosody: Prosody; wordEdits?: Record<number, string> }): Promise<DetectedEvent[]>;
  redetectSentence(p: { sentence: SentenceIdx; wordEdits: Record<number, string> }): Promise<DetectedEvent[]>;
  /** The plan of the detector's events (D-124). Stateless. */
  rulesPlan(p: { transcript: Transcript; events: DetectedEvent[] }): Promise<EditPlan>;
  /**
   * V6.2 (v6.2implementation D-467, §8.1): what the validation keeps of a
   * model's plan against the transcript as edited, before the page chooses
   * it. Stateless: the session and its scene are not touched.
   */
  validatePlan(p: { transcript: Transcript; plan: EditPlan; wordEdits: Record<number, string> }): Promise<PlanCheck>;
  /** The plan is validated against the transcript as edited before the scene is built (TS §40.4). */
  setScene(p: {
    transcript: Transcript;
    plan: EditPlan;
    edit: EditState;
    timeMap: TimeMap;
    profile: "preview" | { entitlementToken: string };
  }): Promise<SceneReport>;
  /** The canvas is transferred. */
  attachPreview(p: { canvas: OffscreenCanvas }): Promise<void>;
  previewPlay(p: { clock: ClockSync }): Promise<PreviewStats>;
  /** Fire-and-forget. */
  previewClock(p: { clock: ClockSync }): void;
  previewPause(): Promise<void>;
  previewSeek(p: { at: OutMs }): Promise<void>;
  /**
   * What the point `(x, y)`, fractions of the canvas, is on at `at` (D-174).
   * While the preview plays, `{ kind: "video" }` at once: direct manipulation
   * is for a paused preview (D-183).
   */
  hitTest(p: { at: OutMs; x: number; y: number }): Promise<Hit>;
  /**
   * Three frames of `at` in Clean, Bold and Mono with the current edits, at
   * `width` x `height`; transferred (D-186). While paused only; the preview
   * is as it was when it answers.
   */
  styleThumbnails(p: { at: OutMs; width: number; height: number }): Promise<ImageBitmap[]>;
  /** The caption shown at `at` as drawn; asked by development and test builds only (D-155). */
  captionMirror(p: { at: OutMs }): Promise<CaptionMirror | null>;
  /** The profile a token gives now, verified here (INV-9, v5implementation D-220). Keeps nothing. */
  exportProfile(p: { entitlementToken: string }): Promise<ExportProfile>;
  exportClip(p: {
    exportId: ExportId;
    entitlementToken: string;
    /** The program's audio; transferred (TS §40.9 S-8). */
    prog48: Float32Array;
  }): Promise<{ opfsPath: string; summary: ChangeSummary; stageTimings: StageTiming[] }>;
  closeSession(): Promise<void>;
  /**
   * V6.2 (TE-30, D-474): draws `image_basic`'s frame in this worker and reads
   * it back. A development build's check only; a production build refuses it.
   */
  devImageCheck(): Promise<ImageCheckResult>;
}

/** What the image check read back: RGBA rows from the top-left, and the images' decode time. */
export type ImageCheckResult = { width: number; height: number; pixels: Uint8Array; decodeMs: number };

/** `epochMs` is `performance.timeOrigin + performance.now()`. `audioMs` is program time (D-126). */
export type ClockSync = { audioMs: OutMs; epochMs: number };

/**
 * V6.2: what `validatePlan` answers: how many items the plan keeps and what
 * it left out, or why a plan of another build is refused whole (DC-7).
 */
export type PlanCheck = { kept: number; dropped: Dropped[]; extras: PlanExtras } | { refused: "PlanFormat" | "PlanCatalog" };

/** V6.2 (P194, D-454): a plan's text extras that passed its validation, for the text extras panel. */
export type PlanExtras = { hook: string | null; post_caption: string | null; hashtags: string[] };

/** What `setScene` answers: what the plan's validation left out (D-124), and the chunks of the captions (D-182). */
export type SceneReport = { dropped: Dropped[]; chunks: SceneChunk[] };

/**
 * A chunk of the captions, as the scene made it: the transcript words from
 * `start` (a number's other words, and words hidden after it, run to the
 * next chunk's start), and the program time it is shown from.
 */
export type SceneChunk = { start: WordIdx; end: WordIdx; at: OutMs };

/** What a play drew, when it ended (D-155). */
export type PreviewStats = {
  /** Frames drawn. */
  frames: number;
  /** Times the frame of the moment was not decoded yet, and the one before it was shown. */
  late: number;
  /** The most the time drawn lay from the time asked for. */
  maxDriftMs: DurMs;
  /** The preview was drawn at 360 x 640 (D-149). Always `false` before V4 Prompt 112. */
  downscaled: boolean;
};

/** A box of the canvas, as fractions of its width and height. */
export type CanvasBox = { x: number; y: number; w: number; h: number };

/** What a point of the canvas is on: a word with its glyph box, the caption block with its box, or the video (D-174). */
export type Hit =
  | { kind: "word"; word: WordIdx; rect: CanvasBox }
  | { kind: "captions"; rect: CanvasBox }
  | { kind: "video" };

/** The caption shown at a time: its words joined by spaces, and each word's fill as `#rrggbbaa` (D-155). */
export type CaptionMirror = { text: string; colors: string[] };

export type StageTiming = { stage: PipelineStage; durationMs: DurMs };
