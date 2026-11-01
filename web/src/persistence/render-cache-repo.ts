// The `renderCache` store (TS §23.1, §21.4, §40.9 S-9): the export of an edit,
// under a key that covers everything the file depends on. One row per clip
// (v5implementation D-225): a hit is "the same edit as the last export". The
// profile kind in the key is the one the worker verified (D-220, INV-9), and a
// row holds no file name (P-11): its path is `exports/<uuid>.mp4`. The cache is
// an optimization, so a read that fails is a miss and a write that fails is
// nothing.

import type { DetectedEvent, EditPlan, EditState, ProfileKind, TimeMap, Transcript } from "../gen/domain";
import { openDb } from "./db";
import { OpfsError, size } from "./opfs";
import { DB_VERSION, type RenderCacheRecord, STORES, type Versioned } from "./schema";

// Set by vite.config.ts from what the renderer is made of; absent where no bundler ran.
declare const __RENDERER_VERSION__: string | undefined;
/** What drew and encoded a file: a new deploy of either gives a new key (D-223). */
export const RENDERER_VERSION: string = typeof __RENDERER_VERSION__ === "undefined" ? "test" : __RENDERER_VERSION__;

export type CacheInputs = {
  sourceHashPrefix: string;
  transcript: Transcript;
  events: readonly DetectedEvent[];
  edit: EditState;
  plan: EditPlan;
  timeMap: TimeMap;
  /** "default" until a brand kit (V6.7): a function of the style and the renderer, both in the key (D-224). */
  theme: string;
  /** `null` until music (V6.3). */
  musicId: string | null;
  /** From the token the worker verified, and from nowhere else (D-220, INV-9). */
  profileKind: ProfileKind;
  rendererVersion: string;
};

/** `value` with the keys of every object in order, at every depth. */
function sorted(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(sorted);
  }
  if (typeof value !== "object" || value === null) {
    return value;
  }
  const entries = Object.entries(value).sort(([a], [b]) => (a < b ? -1 : 1));
  return Object.fromEntries(entries.map(([key, inner]) => [key, sorted(inner)]));
}

/**
 * Sorted by key at every depth; `undefined` left out, as JSON does. An edit's
 * `event_overrides` holds its keys in the order they were made, and the same
 * edit reached by another route must give the same key.
 */
export function canonicalJson(value: unknown): string {
  return JSON.stringify(sorted(value));
}

/** Lower-case hex SHA-256 of `canonicalJson(inputs)`, on the main thread: a few milliseconds (D-222). */
export async function cacheKey(inputs: CacheInputs): Promise<string> {
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(canonicalJson(inputs))));
  return Array.from(digest, (byte) => byte.toString(16).padStart(2, "0")).join("");
}

/** The row of `key`, if its file is there at its size. A row without its file is deleted and is a miss (TS §23.3). */
export async function lookup(key: string): Promise<RenderCacheRecord | undefined> {
  try {
    const db = await openDb();
    const stored = await db.get(STORES.renderCache, key);
    if (stored === undefined) {
      return undefined;
    }
    if (stored.schemaVersion <= DB_VERSION && (await size(stored.value.opfsPath)) === stored.value.bytes) {
      return stored.value;
    }
    // Written by a newer build (TS §23.3), or its file is gone: absent to this one, and removed.
    await db.delete(STORES.renderCache, key);
    return undefined;
  } catch {
    return undefined;
  }
}

/** After the file is in place (TS §23.3's write order). A failure is swallowed: the cache is an optimization. */
export async function put(key: string, record: RenderCacheRecord): Promise<void> {
  const stored: Versioned<RenderCacheRecord> = { schemaVersion: DB_VERSION, value: record };
  // Failing, the next export of this edit renders again.
  await openDb()
    .then((db) => db.put(STORES.renderCache, stored, key))
    .catch(() => undefined);
}

/** The bytes of every row: a miss deletes them, so they count as free (TS §12.5, "after eviction"). 0 when the store cannot be read. */
export async function heldBytes(): Promise<number> {
  try {
    const db = await openDb();
    const rows = await db.getAll(STORES.renderCache);
    return rows.reduce((sum, row) => sum + row.value.bytes, 0);
  } catch {
    return 0;
  }
}

/** Every row: deleted before a new render, and with a clip (D-225, D-228). */
export async function clear(): Promise<void> {
  try {
    const db = await openDb();
    await db.clear(STORES.renderCache);
  } catch (error) {
    const quota = error instanceof DOMException && error.name === "QuotaExceededError";
    throw new OpfsError(quota ? "quota" : "io", error);
  }
}
