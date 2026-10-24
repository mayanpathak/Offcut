// The message types of all four workers (TS §14.1, §14.2). Types only: this
// file has no function and no constant. It imports from gen/ and nothing else.

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
}

export interface AsrWorkerApi {
  load(p: { modelId: string; backend: "webgpu" | "wasm" }): Promise<{ backend: "webgpu" | "wasm" }>;
  transcribe(p: { pcm16: Float32Array }): Promise<{ ok: Transcript } | { rejected: "NoSpeech" }>;
  unload(): Promise<void>;
}

// The audio worker has no method that detects silence or plans a cut. Only
// `assemble` applies a time map, and in V4 the only map is the identity.
export interface AudioWorkerApi {
  /** `out48` is kept in the worker; `out48.length === pcm48.length`. */
  runChain(p: { pcm48: Float32Array }): Promise<{ loudness: Lufs }>;
  /** `out48` is transferred out, unmodified. */
  measureProsody(p: { words: Word[]; sentences: Sentence[] }): Promise<{ out48: Float32Array; prosody: Prosody }>;
  /** `out48` is transferred in and `prog48` out; the worker keeps nothing (D-127). */
  assemble(p: { out48: Float32Array; timeMap: TimeMap; speech: OutSpan[] }): Promise<{ prog48: Float32Array; loudness: Lufs }>;
  /** Fire-and-forget: drops any audio the worker holds (D-128). */
  release(): void;
}

export interface RenderWorkerApi {
  openSession(p: { clipId: ClipId; clipInfo: ClipInfo }): Promise<void>;
  /** With `wordEdits`, the events of the transcript as edited (a replace-all, D-175). */
  detect(p: { transcript: Transcript; prosody: Prosody; wordEdits?: Record<number, string> }): Promise<DetectedEvent[]>;
  redetectSentence(p: { sentence: SentenceIdx; wordEdits: Record<number, string> }): Promise<DetectedEvent[]>;
  /** The plan of the detector's events (D-124). Stateless. */
  rulesPlan(p: { transcript: Transcript; events: DetectedEvent[] }): Promise<EditPlan>;
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
  /** The caption shown at `at` as drawn; asked by development and test builds only (D-155). */
  captionMirror(p: { at: OutMs }): Promise<CaptionMirror | null>;
  exportClip(p: {
    exportId: ExportId;
    entitlementToken: string;
    out48: Float32Array;
  }): Promise<{ opfsPath: string; summary: ChangeSummary; stageTimings: StageTiming[] }>;
  closeSession(): Promise<void>;
}

/** `epochMs` is `performance.timeOrigin + performance.now()`. `audioMs` is program time (D-126). */
export type ClockSync = { audioMs: OutMs; epochMs: number };

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
