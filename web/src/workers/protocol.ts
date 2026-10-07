// The message types of all four workers (TS §14.1, §14.2). Types only: this
// file has no function and no constant. It imports from gen/ and nothing else.

import type {
  ChangeSummary,
  ClipId,
  ClipInfo,
  DetectedEvent,
  DurMs,
  EditState,
  ErrorCode,
  EventKind,
  ExportId,
  FailureStage,
  JobId,
  Lufs,
  PipelineStage,
  Prosody,
  RejectReason,
  Sentence,
  SentenceIdx,
  TimeMs,
  Transcript,
  Word,
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
  | { kind: "event_found"; eventKind: EventKind; display: string };

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

// The audio worker has no method that detects silence, plans or applies cuts,
// or changes the length of the audio.
export interface AudioWorkerApi {
  /** `out48` is kept in the worker; `out48.length === pcm48.length`. */
  runChain(p: { pcm48: Float32Array }): Promise<{ loudness: Lufs }>;
  /** `out48` is transferred out, unmodified. */
  measureProsody(p: { words: Word[]; sentences: Sentence[] }): Promise<{ out48: Float32Array; prosody: Prosody }>;
}

export interface RenderWorkerApi {
  openSession(p: { clipId: ClipId; clipInfo: ClipInfo }): Promise<void>;
  detect(p: { transcript: Transcript; prosody: Prosody }): Promise<DetectedEvent[]>;
  redetectSentence(p: { sentence: SentenceIdx; wordEdits: Record<number, string> }): Promise<DetectedEvent[]>;
  setScene(p: {
    transcript: Transcript;
    events: DetectedEvent[];
    edit: EditState;
    profile: "preview" | { entitlementToken: string };
  }): Promise<void>;
  /** The canvas is transferred. */
  attachPreview(p: { canvas: OffscreenCanvas }): Promise<void>;
  previewPlay(p: { clock: ClockSync }): Promise<void>;
  /** Fire-and-forget. */
  previewClock(p: { clock: ClockSync }): void;
  previewPause(): Promise<void>;
  previewSeek(p: { at: TimeMs }): Promise<void>;
  exportClip(p: {
    exportId: ExportId;
    entitlementToken: string;
    out48: Float32Array;
  }): Promise<{ opfsPath: string; summary: ChangeSummary; stageTimings: StageTiming[] }>;
  closeSession(): Promise<void>;
}

/** `epochMs` is `performance.timeOrigin + performance.now()`. */
export type ClockSync = { audioMs: TimeMs; epochMs: number };

export type StageTiming = { stage: PipelineStage; durationMs: DurMs };
