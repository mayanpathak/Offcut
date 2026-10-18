// The `edits` store (TS §23.1): a clip's edit state (word edits, event
// toggles, style, crop), under the clip's id. A failed IndexedDB call is an
// `OpfsError`, so the use-cases map it as they map a failed OPFS call
// (v3implementation D-103).

import type { ClipId } from "../gen/domain";
import { openDb } from "./db";
import { OpfsError } from "./opfs";
import { DB_VERSION, type EditsRecord, STORES, type Versioned } from "./schema";

/** A failed IndexedDB call, as the error the use-cases map (D-103). */
function storageError(error: unknown): OpfsError {
  const quota = error instanceof DOMException && error.name === "QuotaExceededError";
  return new OpfsError(quota ? "quota" : "io", error);
}

export async function put(clipId: ClipId, record: EditsRecord): Promise<void> {
  const stored: Versioned<EditsRecord> = { schemaVersion: DB_VERSION, value: record };
  try {
    const db = await openDb();
    await db.put(STORES.edits, stored, clipId);
  } catch (error) {
    throw storageError(error);
  }
}

/** The clip's edit state, or `undefined` if there is none. */
export async function get(clipId: ClipId): Promise<EditsRecord | undefined> {
  try {
    const db = await openDb();
    const stored = await db.get(STORES.edits, clipId);
    if (stored === undefined) {
      return undefined;
    }
    if (stored.schemaVersion > DB_VERSION) {
      // Written by a newer build: absent to this one, and removed (TS §23.3).
      await db.delete(STORES.edits, clipId);
      return undefined;
    }
    return stored.value;
  } catch (error) {
    throw storageError(error);
  }
}

/**
 * Removes every record: the next import removes the edits of the clip
 * before (v3implementation §23, item 26). V7: `remove(clipId)` takes its place.
 */
export async function clear(): Promise<void> {
  try {
    const db = await openDb();
    await db.clear(STORES.edits);
  } catch (error) {
    throw storageError(error);
  }
}
