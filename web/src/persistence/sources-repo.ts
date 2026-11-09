// The long path's three stores (v6.1implementation §9.1, D-414, D-415): one
// record per source, its cloud windows under `[sourceId, index]`, and its
// master transcript. The only reader and writer of them, and the only caller
// of `openSourcesDb`. No file name and no path is stored (P-11). A record
// whose `schemaVersion` is newer than this build is absent, and removed, as in
// every other repository (TS §23.3). A failed call is an `OpfsError` (D-103).

import type { Transcript } from "../gen/domain";
import { openSourcesDb } from "./db";
import { OpfsError, paths, remove } from "./opfs";
import {
  DB_VERSION,
  SOURCE_STORES,
  type SourceId,
  type SourceRecord,
  type SourceWindowRecord,
  type Versioned,
} from "./schema";

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

function wrap<T>(value: T): Versioned<T> {
  return { schemaVersion: DB_VERSION, value };
}

function readable<T>(stored: Versioned<T> | undefined): stored is Versioned<T> {
  return stored !== undefined && stored.schemaVersion <= DB_VERSION;
}

/** Every key of a source's windows: `[id, 0]` to `[id, +∞)`. */
function windowRange(id: SourceId): IDBKeyRange {
  return IDBKeyRange.bound([id, 0], [id, Number.MAX_SAFE_INTEGER]);
}

export function put(record: SourceRecord): Promise<void> {
  return guarded(async () => {
    const db = await openSourcesDb();
    await db.put(SOURCE_STORES.sources, wrap(record), record.sourceId);
  });
}

export function get(id: SourceId): Promise<SourceRecord | undefined> {
  return guarded(async () => {
    const db = await openSourcesDb();
    const stored = await db.get(SOURCE_STORES.sources, id);
    if (stored !== undefined && !readable(stored)) {
      await db.delete(SOURCE_STORES.sources, id);
      return undefined;
    }
    return stored?.value;
  });
}

/** Merges `change` into the record; nothing when there is none. */
export async function update(id: SourceId, change: Partial<Omit<SourceRecord, "sourceId">>): Promise<SourceRecord | undefined> {
  const record = await get(id);
  if (record === undefined) {
    return undefined;
  }
  const next = { ...record, ...change };
  await put(next);
  return next;
}

/** The most recently opened source, or `undefined` (V6.1 keeps one, D-415). */
export function newest(): Promise<SourceRecord | undefined> {
  return guarded(async () => {
    const db = await openSourcesDb();
    const all = await db.getAll(SOURCE_STORES.sources);
    let best: SourceRecord | undefined;
    for (const stored of all) {
      if (readable(stored) && (best === undefined || stored.value.lastOpenedAt > best.lastOpenedAt)) {
        best = stored.value;
      }
    }
    return best;
  });
}

export function putWindow(id: SourceId, index: number, record: SourceWindowRecord): Promise<void> {
  return guarded(async () => {
    const db = await openSourcesDb();
    await db.put(SOURCE_STORES.sourceTranscripts, wrap(record), [id, index]);
  });
}

/** The windows of a source that are done, by index. */
export function windows(id: SourceId): Promise<Map<number, SourceWindowRecord>> {
  return guarded(async () => {
    const db = await openSourcesDb();
    const range = windowRange(id);
    const [keys, values] = await Promise.all([
      db.getAllKeys(SOURCE_STORES.sourceTranscripts, range),
      db.getAll(SOURCE_STORES.sourceTranscripts, range),
    ]);
    const done = new Map<number, SourceWindowRecord>();
    keys.forEach((key, i) => {
      const stored = values[i];
      if (readable(stored)) {
        done.set(key[1], stored.value);
      }
    });
    return done;
  });
}

export function putMaster(id: SourceId, transcript: Transcript): Promise<void> {
  return guarded(async () => {
    const db = await openSourcesDb();
    await db.put(SOURCE_STORES.masterTranscripts, wrap({ transcript }), id);
  });
}

export function getMaster(id: SourceId): Promise<Transcript | undefined> {
  return guarded(async () => {
    const db = await openSourcesDb();
    const stored = await db.get(SOURCE_STORES.masterTranscripts, id);
    return readable(stored) ? stored.value.transcript : undefined;
  });
}

/** A source, its windows, its transcript and its folder of OPFS, gone. */
export function removeSource(id: SourceId): Promise<void> {
  return guarded(async () => {
    const db = await openSourcesDb();
    await db.delete(SOURCE_STORES.sources, id);
    await db.delete(SOURCE_STORES.sourceTranscripts, windowRange(id));
    await db.delete(SOURCE_STORES.masterTranscripts, id);
    await remove(paths.sourceDir(id));
  });
}

/** Every source, before a new one is made: V6.1 keeps one (D-415). Clips are not touched. */
export function removeAll(): Promise<void> {
  return guarded(async () => {
    const db = await openSourcesDb();
    for (const key of await db.getAllKeys(SOURCE_STORES.sources)) {
      await removeSource(key);
    }
    await db.clear(SOURCE_STORES.sourceTranscripts);
    await db.clear(SOURCE_STORES.masterTranscripts);
    await remove(paths.sourcesRoot());
  });
}
