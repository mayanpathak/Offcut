// The browser database (TS §23.1). This file is the only place that names its
// stores. All eight exist from version 1, although V1 uses only `meta` (D-6).

import type { DBSchema } from "idb";

import type {
  Bytes,
  ClipId,
  ClipInfo,
  DetectedEvent,
  EditState,
  ExportId,
  Prosody,
  Transcript,
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

export const META_KEYS = { anonId: "anonId", persistRequested: "persistRequested" } as const;

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
};

export type EntitlementRecord = { token: string; storedAt: number };

export type ReceiptOutboxRecord = { exportId: ExportId; enqueuedAt: number; attempts: number };

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
