// V6.2: the `plans` store (v6.2implementation §8.3, D-469): one record per
// clip, under its id, holding the plan chosen when its request was made. The
// only reader and writer of the store, and the only caller of `openPlansDb`,
// so only a build with `ai_plan` on (its callers check the flag) ever upgrades
// a database to structure version 3. A record whose `schemaVersion` is newer
// than this build is absent, and removed, as in every other repository
// (TS §23.3). A failed call is an `OpfsError` (D-103).

import type { ClipId } from "../gen/domain";
import { openPlansDb } from "./db";
import { OpfsError } from "./opfs";
import { DB_VERSION, PLAN_STORES, type PlanRecord, type Versioned } from "./schema";

function storageError(error: unknown): OpfsError {
  if (error instanceof OpfsError) {
    return error;
  }
  const quota = error instanceof DOMException && error.name === "QuotaExceededError";
  return new OpfsError(quota ? "quota" : "io", error);
}

async function guarded<T>(operation: () => Promise<T>): Promise<T> {
  try {
    return await operation();
  } catch (error) {
    throw storageError(error);
  }
}

/** Keeps the clip's plan, replacing the one before. */
export function put(clipId: ClipId, record: PlanRecord): Promise<void> {
  return guarded(async () => {
    const db = await openPlansDb();
    const stored: Versioned<PlanRecord> = { schemaVersion: DB_VERSION, value: record };
    await db.put(PLAN_STORES.plans, stored, clipId);
  });
}

/** The clip's plan, or `undefined` if none was kept. */
export function get(clipId: ClipId): Promise<PlanRecord | undefined> {
  return guarded(async () => {
    const db = await openPlansDb();
    const stored = await db.get(PLAN_STORES.plans, clipId);
    if (stored === undefined) {
      return undefined;
    }
    if (stored.schemaVersion > DB_VERSION) {
      // Written by a newer build: absent to this one, and removed (TS §23.3).
      await db.delete(PLAN_STORES.plans, clipId);
      return undefined;
    }
    return stored.value;
  });
}

/** Every plan, as the next import removes the clip before it (D-40). */
export function clear(): Promise<void> {
  return guarded(async () => {
    const db = await openPlansDb();
    await db.clear(PLAN_STORES.plans);
  });
}
