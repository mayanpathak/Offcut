// The `clips` store (TS §23.1): one record per accepted clip, under its id.
// The record holds no file name and no path (P-11). A failed IndexedDB call
// is an `OpfsError`, so the use-cases map it as they map a failed OPFS call
// (v3implementation D-103).

import type { Bytes, ClipId } from "../gen/domain";
import { openDb } from "./db";
import { OpfsError } from "./opfs";
import { type ClipRecord, DB_VERSION, STORES, type Versioned } from "./schema";

/** How much of the source `hashPrefix` reads: 4 MiB (D-81). */
export const HASH_PREFIX_BYTES = 4_194_304;

/** The size in bytes the file's length is appended as: an unsigned 64-bit integer. */
const SIZE_BYTES = 8;

/**
 * D-81: the lower-case hex SHA-256 of the first min(4 MiB, size) bytes of
 * the file, then its size as an unsigned 64-bit little-endian integer. Read
 * on the main thread before the file is posted to the media worker. A file
 * the browser cannot read rejects with the browser's own error
 * (`NotReadableError`); nothing is stored here.
 */
export async function hashPrefix(file: Blob): Promise<string> {
  const head = new Uint8Array(await file.slice(0, HASH_PREFIX_BYTES).arrayBuffer());
  const data = new Uint8Array(head.length + SIZE_BYTES);
  data.set(head);
  new DataView(data.buffer).setBigUint64(head.length, BigInt(file.size), true);
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", data));
  return Array.from(digest, (byte) => byte.toString(16).padStart(2, "0")).join("");
}

/** A failed IndexedDB call, as the error the use-cases map (D-103). */
function storageError(error: unknown): OpfsError {
  const quota = error instanceof DOMException && error.name === "QuotaExceededError";
  return new OpfsError(quota ? "quota" : "io", error);
}

export async function put(record: ClipRecord): Promise<void> {
  const stored: Versioned<ClipRecord> = { schemaVersion: DB_VERSION, value: record };
  try {
    const db = await openDb();
    await db.put(STORES.clips, stored, record.clipId);
  } catch (error) {
    throw storageError(error);
  }
}

/** The clip's record, or `undefined` if there is none. */
export async function get(clipId: ClipId): Promise<ClipRecord | undefined> {
  try {
    const db = await openDb();
    const stored = await db.get(STORES.clips, clipId);
    if (stored === undefined) {
      return undefined;
    }
    if (stored.schemaVersion > DB_VERSION) {
      // Written by a newer build: absent to this one, and removed (TS §23.3).
      await db.delete(STORES.clips, clipId);
      return undefined;
    }
    return stored.value;
  } catch (error) {
    throw storageError(error);
  }
}

/**
 * Removes every record. Until V7 the device keeps one clip (D-40), so the
 * next import removes the records of the one before (v3implementation §23,
 * item 26). V7: `remove(clipId)` and eviction take its place.
 */
export async function clear(): Promise<void> {
  try {
    const db = await openDb();
    await db.clear(STORES.clips);
  } catch (error) {
    throw storageError(error);
  }
}

/** The one place of this file that turns a number into `Bytes` (D-103). */
export function toBytes(n: number): Bytes {
  return n as Bytes;
}
