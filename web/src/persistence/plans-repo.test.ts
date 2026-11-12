// V6.2: the `plans` store at structure version 3 (v6.2implementation D-469).
// The real `db.ts` and `plans-repo.ts` over an in-memory `openDB` that keeps
// IndexedDB's version rules: a lower version than the stored one is a
// `VersionError`; no version opens the stored one (1 when there is none); a
// higher one runs `upgrade(db, old, new)` once and first asks every open
// connection to close (`blocking`); a closed connection refuses every call;
// creating a store twice is refused. A real browser read the same upgrades
// once in P192 (V6 CL Entry 54); no npm package was added for this (§3.1).

import { beforeEach, describe, expect, it, vi } from "vitest";

import type { EditPlan } from "../gen/domain";
import type { ClipRecord, PlanRecord } from "./schema";

type Row = Map<string, unknown>;
type Handlers = {
  upgrade?: (db: unknown, oldVersion: number, newVersion: number | null) => void;
  blocking?: (current: number, blocked: number | null, event: { target: { close(): void } }) => void;
};

const disk = vi.hoisted(() => ({
  version: 0,
  stores: new Map<string, Map<string, unknown>>(),
  /** Each `openDB` call's requested version, in order (`undefined`: the stored one). */
  asked: [] as (number | undefined)[],
  open: [] as { closed: boolean; handlers: Handlers }[],
  failPut: undefined as DOMException | undefined,
}));

vi.mock("idb", () => ({
  openDB: (_name: string, version: number | undefined, handlers: Handlers = {}) => {
    disk.asked.push(version);
    if (version !== undefined && version < disk.version) {
      return Promise.reject(new DOMException("lower than the stored version", "VersionError"));
    }
    const target = version ?? Math.max(disk.version, 1);
    const connection = { closed: false, handlers };
    const live = (): void => {
      if (connection.closed) {
        throw new DOMException("the connection is closed", "InvalidStateError");
      }
    };
    const store = (name: string): Row => {
      live();
      const found = disk.stores.get(name);
      if (found === undefined) {
        throw new DOMException(`no store ${name}`, "NotFoundError");
      }
      return found;
    };
    const db = {
      createObjectStore: (name: string) => {
        if (disk.stores.has(name)) {
          throw new DOMException(`${name} exists`, "ConstraintError");
        }
        disk.stores.set(name, new Map());
      },
      get: (name: string, key: unknown) => Promise.resolve(store(name).get(JSON.stringify(key))),
      put: (name: string, value: unknown, key: unknown) => {
        const rows = store(name);
        if (disk.failPut !== undefined) {
          return Promise.reject(disk.failPut);
        }
        rows.set(JSON.stringify(key), structuredClone(value));
        return Promise.resolve();
      },
      delete: (name: string, key: unknown) => {
        store(name).delete(JSON.stringify(key));
        return Promise.resolve();
      },
      clear: (name: string) => {
        store(name).clear();
        return Promise.resolve();
      },
      getAll: (name: string) => Promise.resolve([...store(name).values()]),
    };
    if (target > disk.version) {
      for (const other of disk.open.filter((c) => !c.closed)) {
        other.handlers.blocking?.(disk.version, target, { target: { close: () => (other.closed = true) } });
        if (!other.closed) {
          return Promise.reject(new Error("blocked: an open connection did not close"));
        }
      }
      const old = disk.version;
      disk.version = target;
      handlers.upgrade?.(db, old, target);
    }
    disk.open.push(connection);
    return Promise.resolve(db);
  },
}));

type Modules = {
  db: typeof import("./db");
  repo: typeof import("./plans-repo");
  schema: typeof import("./schema");
};

/** A fresh page: `db.ts`' cached connections start empty. */
async function page(): Promise<Modules> {
  vi.resetModules();
  return { db: await import("./db"), repo: await import("./plans-repo"), schema: await import("./schema") };
}

/** A clip's id, typed as the clips store gives it. */
function clipId(id: string): ClipRecord["clipId"] {
  return ({ clipId: id } as unknown as ClipRecord).clipId;
}

const CLIP = clipId("clip-1");
const PLAN = { format: 1, catalog: 2, source: { kind: "model", provider: "deepseek", model: "deepseek-flash" }, items: [] } as unknown as EditPlan;
const RECORD: PlanRecord = { plan: PLAN, source: "model", requestedAt: 1_700_000_000_000, catalog: 2 };

/** Every row of every store, as plain data. */
function everything(): Record<string, Record<string, unknown>> {
  return Object.fromEntries([...disk.stores].map(([name, rows]) => [name, Object.fromEntries(rows)]));
}

/** What a V6.1 person's browser holds: a clip's records and `meta`, written through the clip path at version 1. */
async function aClipAtVersion1(m: Modules): Promise<void> {
  const db = await m.db.openDb();
  const v = (value: unknown) => ({ schemaVersion: 1, value });
  await db.put("clips", v({ clipId: CLIP, createdAt: 1 }) as never, CLIP);
  await db.put("transcripts", v({ transcript: { words: [] }, prosody: { per_word: [] } }) as never, CLIP);
  await db.put("edits", v({ editState: { style: "clean" } }) as never, CLIP);
  await db.put("receiptOutbox", v({ attempts: 2 }) as never, "export-1" as never);
  await m.db.metaSet("anonId", "anon-123");
}

/** A long source's records, written through the long path at version 2. */
async function aSourceAtVersion2(m: Modules): Promise<void> {
  const db = await m.db.openSourcesDb();
  const v = (value: unknown) => ({ schemaVersion: 1, value });
  await db.put("sources", v({ sourceId: "s-1", stage: "transcribing" }) as never, "s-1" as never);
  await db.put("sourceTranscripts", v({ words: [], lengthMs: 90_000, modelVersion: "m" }) as never, ["s-1", 0] as never);
  await db.put("masterTranscripts", v({ transcript: { words: [] } }) as never, "s-1" as never);
}

beforeEach(() => {
  disk.version = 0;
  disk.stores.clear();
  disk.asked.length = 0;
  disk.open.length = 0;
  disk.failPut = undefined;
});

describe("the upgrade to structure version 3 (D-469)", () => {
  it("1 → 3 keeps every record, and the clip path still reads them", async () => {
    const m = await page();
    await aClipAtVersion1(m);
    expect(disk.version).toBe(1);
    const before = everything();

    await m.repo.put(CLIP, RECORD);

    expect(disk.version).toBe(3);
    // Every record of version 1 is there, unchanged; the stores of 2 and 3 are new.
    const after = everything();
    for (const [store, rows] of Object.entries(before)) {
      expect(after[store]).toEqual(rows);
    }
    expect(Object.keys(after).sort()).toEqual(
      [...Object.keys(before), "sources", "sourceTranscripts", "masterTranscripts", "plans"].sort(),
    );
    expect(await m.repo.get(CLIP)).toEqual(RECORD);
    // The clip path's own connection was asked to close; it opens again, at 3.
    expect(await m.db.metaGet("anonId")).toBe("anon-123");
    expect((await (await m.db.openDb()).get("clips", CLIP))?.value).toEqual({ clipId: CLIP, createdAt: 1 });
  });

  it("2 → 3 keeps every record, of the clip path and of the long path", async () => {
    const m = await page();
    await aClipAtVersion1(m);
    await aSourceAtVersion2(m);
    expect(disk.version).toBe(2);
    const before = everything();

    await m.repo.put(CLIP, RECORD);

    expect(disk.version).toBe(3);
    const after = everything();
    for (const [store, rows] of Object.entries(before)) {
      expect(after[store]).toEqual(rows);
    }
    expect(Object.keys(after).sort()).toEqual([...Object.keys(before), "plans"].sort());
    // Both older openers still read what they wrote, on the version-3 database.
    expect((await (await m.db.openSourcesDb()).get("sources", "s-1" as never))?.value).toEqual({ sourceId: "s-1", stage: "transcribing" });
    expect(await m.db.metaGet("anonId")).toBe("anon-123");
    expect(await m.repo.get(CLIP)).toEqual(RECORD);
  });

  it("the clip path still opens version 1, and the long path 2: neither makes the plans store", async () => {
    const m = await page();
    await aClipAtVersion1(m);
    expect(disk.asked).toEqual([1]);
    expect(disk.version).toBe(1);
    expect(disk.stores.has("plans")).toBe(false);
    await aSourceAtVersion2(m);
    expect(disk.asked).toEqual([1, 2]);
    expect(disk.version).toBe(2);
    expect(disk.stores.has("plans")).toBe(false);
  });

  it("on a database at 3, a new page's clip path asks for 1, is refused, and opens it as it is", async () => {
    const first = await page();
    await first.repo.put(CLIP, RECORD);
    disk.asked.length = 0;

    const next = await page();
    await next.db.metaSet("anonId", "anon-456");
    expect(disk.asked).toEqual([1, undefined]);
    expect(disk.version).toBe(3);
    expect(await next.db.metaGet("anonId")).toBe("anon-456");
  });

  it("a database a later build took to 4 is opened as it is, and the plan still reads", async () => {
    const m = await page();
    await m.repo.put(CLIP, RECORD);
    disk.version = 4;
    const next = await page();
    disk.asked.length = 0;
    expect(await next.repo.get(CLIP)).toEqual(RECORD);
    expect(disk.asked).toEqual([3, undefined]);
    expect(disk.version).toBe(4);
  });
});

describe("plans-repo (D-469)", () => {
  it("keeps one record per clip, replaces it, and clears them all", async () => {
    const { repo } = await page();
    const other = clipId("clip-2");
    expect(await repo.get(CLIP)).toBeUndefined();
    await repo.put(CLIP, RECORD);
    await repo.put(other, { ...RECORD, source: "rules" });
    await repo.put(CLIP, { ...RECORD, requestedAt: 5 });
    expect((await repo.get(CLIP))?.requestedAt).toBe(5);
    expect((await repo.get(other))?.source).toBe("rules");
    expect(disk.stores.get("plans")?.get(JSON.stringify(CLIP))).toEqual({ schemaVersion: 1, value: { ...RECORD, requestedAt: 5 } });
    await repo.clear();
    expect(await repo.get(CLIP)).toBeUndefined();
    expect(await repo.get(other)).toBeUndefined();
  });

  it("treats a record written by a newer build as absent, and removes it (TS §23.3)", async () => {
    const { repo } = await page();
    await repo.put(CLIP, RECORD);
    disk.stores.get("plans")?.set(JSON.stringify(CLIP), { schemaVersion: 2, value: RECORD });
    expect(await repo.get(CLIP)).toBeUndefined();
    expect(disk.stores.get("plans")?.size).toBe(0);
  });

  it("a failed write is an OpfsError, quota or io (D-103)", async () => {
    const { repo } = await page();
    const { OpfsError } = await import("./opfs");
    disk.failPut = new DOMException("full", "QuotaExceededError");
    await expect(repo.put(CLIP, RECORD)).rejects.toSatisfy((e) => e instanceof OpfsError && e.kind === "quota");
    disk.failPut = new DOMException("broken", "UnknownError");
    await expect(repo.put(CLIP, RECORD)).rejects.toSatisfy((e) => e instanceof OpfsError && e.kind === "io");
  });
});

describe("only plans-repo.ts opens version 3 (D-469)", () => {
  /** Every source file under `src/`, as text: tests and generated files left out. */
  const SOURCES = import.meta.glob<string>(["/src/**/*.{ts,tsx}", "!/src/**/*.test.{ts,tsx}", "!/src/gen/**"], {
    query: "?raw",
    import: "default",
    eager: true,
  });

  it("openPlansDb is named only in db.ts and plans-repo.ts, and version 3 only in schema.ts and db.ts", () => {
    const naming = (word: string) =>
      Object.entries(SOURCES)
        .filter(([, text]) => text.includes(word))
        .map(([file]) => file.replace(/^\/src\//, ""))
        .sort();
    expect(Object.keys(SOURCES).length).toBeGreaterThan(100);
    expect(naming("openPlansDb")).toEqual(["persistence/db.ts", "persistence/plans-repo.ts"]);
    expect(naming("PLANS_STRUCTURE_VERSION")).toEqual(["persistence/db.ts", "persistence/schema.ts"]);
  });
});
