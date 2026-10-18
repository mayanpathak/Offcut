// The `transcripts` store (TS §23.1): a processed clip's transcript and the
// prosody measured on it, under the clip's id. A failed IndexedDB call is an
// `OpfsError`, so the use-cases map it as they map a failed OPFS call
// (v3implementation D-103).

import type { ClipId } from "../gen/domain";
import { openDb } from "./db";
import { OpfsError } from "./opfs";
import { DB_VERSION, STORES, type TranscriptRecord, type Versioned } from "./schema";

/** A failed IndexedDB call, as the error the use-cases map (D-103). */
function storageError(error: unknown): OpfsError {
  const quota = error instanceof DOMException && error.name === "QuotaExceededError";
  return new OpfsError(quota ? "quota" : "io", error);
}

export async function put(clipId: ClipId, record: TranscriptRecord): Promise<void> {
  const stored: Versioned<TranscriptRecord> = { schemaVersion: DB_VERSION, value: record };
  try {
    const db = await openDb();
    await db.put(STORES.transcripts, stored, clipId);
  } catch (error) {
    throw storageError(error);
  }
}

/** The clip's transcript and prosody, or `undefined` if there are none. */
export async function get(clipId: ClipId): Promise<TranscriptRecord | undefined> {
  try {
    const db = await openDb();
    const stored = await db.get(STORES.transcripts, clipId);
    if (stored === undefined) {
      return undefined;
    }
    if (stored.schemaVersion > DB_VERSION) {
      // Written by a newer build: absent to this one, and removed (TS §23.3).
      await db.delete(STORES.transcripts, clipId);
      return undefined;
    }
    return stored.value;
  } catch (error) {
    throw storageError(error);
  }
}

/**
 * Removes every record: the next import removes the words of the clip
 * before (v3implementation §23, item 26). V7: `remove(clipId)` takes its place.
 */
export async function clear(): Promise<void> {
  try {
    const db = await openDb();
    await db.clear(STORES.transcripts);
  } catch (error) {
    throw storageError(error);
  }
}
