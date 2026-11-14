// The browser database (TS §23.1). This file is the only place that names its
// stores. All eight exist from version 1, although V1 uses only `meta` (D-6).

import type { DBSchema } from "idb";

import type {
  Bytes,
  ChangeSummary,
  ClipId,
  ClipInfo,
  DetectedEvent,
  EditPlan,
  EditState,
  ExportId,
  Plan,
  Prosody,
  SourceInfo,
  Transcript,
  UserId,
} from "../gen/domain";

export const DB_NAME = "offcut";
export const DB_VERSION = 1;

export const STORES = {
  clips: "clips",
  transcripts: "transcripts",
  events: "events",
  edits: "edits",
  renderCache: "renderCache",
  entitlement: "entitlement",
  receiptOutbox: "receiptOutbox",
  meta: "meta",
} as const;

/** V6.2 adds `aiPlan`, the settings switch "AI edit plan" (v6.2implementation D-466, P193). */
export const META_KEYS = {
  anonId: "anonId",
  persistRequested: "persistRequested",
  transcription: "transcription",
  aiPlan: "aiPlan",
  /** V6.2 (D-486, P208): the Enhance panel's last state, the start of the next clip. */
  enhanceDefault: "enhanceDefault",
} as const;

/**
 * Every stored value is wrapped this way. A value whose `schemaVersion` is
 * newer than the code was written by a newer build: it is treated as absent
 * and deleted (TS §23.3).
 */
export type Versioned<T> = { schemaVersion: number; value: T };

// A time below is in milliseconds since the Unix epoch, as `Date.now()` gives it.

/** No file name and no path: the clip is known by its id only. */
export type ClipRecord = {
  clipId: ClipId;
  createdAt: number;
  lastOpenedAt: number;
  clipInfo: ClipInfo;
  sourceBytes: Bytes;
  /** SHA-256, in hex, over the first 4 MiB of the source plus its length. */
  sourceHashPrefix: string;
};

export type TranscriptRecord = { transcript: Transcript; prosody: Prosody };

export type EditsRecord = { editState: EditState };

export type RenderCacheRecord = {
  clipId: ClipId;
  exportId: ExportId;
  opfsPath: string;
  bytes: Bytes;
  createdAt: number;
  /** What was changed: a hit does not reach the worker, so the row keeps it (v5implementation D-226). */
  summary: ChangeSummary;
};

export type EntitlementRecord = { token: string; storedAt: number };

/** The plan it was exported under and whose it is: `effective()` and `flushOutbox` read them (v6implementation D-267). */
export type ReceiptOutboxRecord = { exportId: ExportId; enqueuedAt: number; attempts: number; plan: Plan; sub: UserId };

/** One entry per store: its key and its value. Keys are given on every write, not read from the value. */
export interface OffcutDb extends DBSchema {
  clips: { key: ClipId; value: Versioned<ClipRecord> };
  transcripts: { key: ClipId; value: Versioned<TranscriptRecord> };
  events: { key: ClipId; value: Versioned<DetectedEvent[]> };
  edits: { key: ClipId; value: Versioned<EditsRecord> };
  /** The key is the cache key, in hex. */
  renderCache: { key: string; value: Versioned<RenderCacheRecord> };
  /** The only key is "current". */
  entitlement: { key: "current"; value: Versioned<EntitlementRecord> };
  receiptOutbox: { key: ExportId; value: Versioned<ReceiptOutboxRecord> };
  /** The keys are those of `META_KEYS`. */
  meta: { key: string; value: Versioned<unknown> };
}

// --- V6.1: the long path (v6.1implementation §9.1, D-414) --------------------

/**
 * The database's structure version (what `openDB` is given), apart from
 * `DB_VERSION`, which is the records' `schemaVersion` and stays 1 so every
 * stored record reads as before. Version 2 adds the three source stores; only
 * `sources-repo.ts` opens it, so a build with `long_video` off never upgrades
 * anyone's database (D-414, §12.4).
 */
export const DB_STRUCTURE_VERSION = 2;

/** The stores of structure version 2. Kept apart from `STORES`, which upgrade 1 creates. */
export const SOURCE_STORES = {
  sources: "sources",
  sourceTranscripts: "sourceTranscripts",
  masterTranscripts: "masterTranscripts",
} as const;

/** A long source's id (a UUID); `workers/protocol.ts` declares the same brand. */
export type SourceId = string & { readonly __unit: "SourceId" };

/** Where a source's work stands; `transcribed` is V6.1's end (D-417). */
export type SourceStage = "extracting_audio" | "transcribing" | "transcribed";

/** No file name and no path: the source is known by its id, its size and its hash prefix. */
export type SourceRecord = {
  sourceId: SourceId;
  createdAt: number;
  lastOpenedAt: number;
  sourceInfo: SourceInfo;
  sourceBytes: Bytes;
  /** SHA-256, in hex, over the first 4 MiB of the file plus its length (D-81, D-419). */
  sourceHashPrefix: string;
  stage: SourceStage;
  /** Whole seconds of `pcm16.f32` written and flushed (D-403). */
  pcm16Seconds: number;
  /** The source-wide 95th-percentile level in dBFS, once the sound is written (D-404). */
  levelP95: number | null;
  windowsDone: number;
  windowsTotal: number;
};

/** One cloud window's words, relative to the window, not yet merged (D-406). */
export type SourceWindowRecord = { words: RawWordRecord[]; lengthMs: number; modelVersion: string };

/** A word as the recognizer gave it (`load-core.ts`' `RawWord`). */
export type RawWordRecord = { text: string; startMs: number; endMs: number; confidence: number };

export type MasterTranscriptRecord = { transcript: Transcript };

/** The three stores of structure version 2, with their keys. */
export interface OffcutSourcesDb extends OffcutDb {
  sources: { key: SourceId; value: Versioned<SourceRecord> };
  /** The key is `[sourceId, windowIndex]`. */
  sourceTranscripts: { key: [SourceId, number]; value: Versioned<SourceWindowRecord> };
  masterTranscripts: { key: SourceId; value: Versioned<MasterTranscriptRecord> };
}

// --- V6.2: the AI plan (v6.2implementation §8, D-469) -------------------------

/**
 * Structure version 3 adds the `plans` store. Only `plans-repo.ts` opens it,
 * so a build with `ai_plan` off never upgrades anyone's database (D-469). The
 * long path still opens version 2 (`DB_STRUCTURE_VERSION`), and the clip path 1.
 */
export const PLANS_STRUCTURE_VERSION = 3;

/** The store of structure version 3. Kept apart from `STORES` and `SOURCE_STORES`. */
export const PLAN_STORES = { plans: "plans" } as const;

/**
 * A clip's plan as it was chosen when its request was made (D-469): the model's
 * plan, or the rules plan the clip fell back to. Restore reads it and never
 * requests again.
 */
export type PlanRecord = {
  plan: EditPlan;
  /** Whose plan it is: `"model"` when the AI plan was used, `"rules"` when the clip fell back. */
  source: "model" | "rules";
  /** When the request was made, in milliseconds since the Unix epoch. */
  requestedAt: number;
  /** The `CATALOG_VERSION` the request was made with. */
  catalog: number;
};

/** Structure version 3: every store of version 2, and `plans`, keyed by the clip. */
export interface OffcutPlansDb extends OffcutSourcesDb {
  plans: { key: ClipId; value: Versioned<PlanRecord> };
}
