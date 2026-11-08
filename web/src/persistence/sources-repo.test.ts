// The source stores' repository (v6.1implementation §9.1, §11.3): records by
// id, windows by `[sourceId, index]`, a newer `schemaVersion` absent and
// removed, one source's removal leaving the others, and the sweep (D-415). The
// database is an in-memory map; the real upgrade from version 1 to 2 is read in
// a browser (P176's scratch check, and the long-path specs of P180).

import { beforeEach, describe, expect, it, vi } from "vitest";

import type { SourceInfo, Transcript } from "../gen/domain";
import type { SourceId, SourceRecord } from "./schema";

type Range = { lower: [string, number]; upper: [string, number] };
const memory = vi.hoisted(() => ({
  stores: new Map<string, Map<string, { key: unknown; value: unknown }>>(),
  removed: [] as string[],
}));

function table(name: string) {
  let t = memory.stores.get(name);
  if (t === undefined) {
    t = new Map();
    memory.stores.set(name, t);
  }
  return t;
}

function inRange(key: unknown, range: Range): boolean {
  if (!Array.isArray(key)) {
    return false;
  }
  const [id, n] = key as [string, number];
  return id === range.lower[0] && n >= range.lower[1] && n <= range.upper[1];
}

function isRange(q: unknown): q is Range {
  return typeof q === "object" && q !== null && "lower" in q;
}

vi.mock("./db", () => ({
  openSourcesDb: () =>
    Promise.resolve({
      put: (store: string, value: unknown, key: unknown) => {
        table(store).set(JSON.stringify(key), { key, value });
        return Promise.resolve();
      },
      get: (store: string, key: unknown) => Promise.resolve(table(store).get(JSON.stringify(key))?.value),
      delete: (store: string, q: unknown) => {
        for (const [k, row] of table(store)) {
          if (isRange(q) ? inRange(row.key, q) : k === JSON.stringify(q)) {
            table(store).delete(k);
          }
        }
        return Promise.resolve();
      },
      getAll: (store: string, q?: Range) =>
        Promise.resolve([...table(store).values()].filter((r) => q === undefined || inRange(r.key, q)).map((r) => r.value)),
      getAllKeys: (store: string, q?: Range) =>
        Promise.resolve([...table(store).values()].filter((r) => q === undefined || inRange(r.key, q)).map((r) => r.key)),
      clear: (store: string) => {
        table(store).clear();
        return Promise.resolve();
      },
    }),
}));

vi.mock("./opfs", async (actual) => ({
  ...(await actual<typeof import("./opfs")>()),
  remove: (path: string) => {
    memory.removed.push(path);
    return Promise.resolve();
  },
}));

vi.stubGlobal("IDBKeyRange", { bound: (lower: unknown, upper: unknown) => ({ lower, upper }) });

const repo = await import("./sources-repo");

function record(id: string, lastOpenedAt: number): SourceRecord {
  return {
    sourceId: id as SourceId,
    createdAt: lastOpenedAt,
    lastOpenedAt,
    sourceInfo: { duration: 600_000 } as SourceInfo,
    sourceBytes: 1_000_000 as SourceRecord["sourceBytes"],
    sourceHashPrefix: "ab".repeat(32),
    stage: "extracting_audio",
    pcm16Seconds: 0,
    levelP95: null,
    windowsDone: 0,
    windowsTotal: 8,
  };
}

const A = "a-source" as SourceId;
const B = "b-source" as SourceId;

describe("sources-repo", () => {
  beforeEach(() => {
    memory.stores.clear();
    memory.removed.length = 0;
  });

  it("keeps a record, merges a change and finds the newest", async () => {
    await repo.put(record(A, 10));
    await repo.put(record(B, 20));
    expect((await repo.get(A))?.lastOpenedAt).toBe(10);
    expect((await repo.newest())?.sourceId).toBe(B);
    const updated = await repo.update(A, { pcm16Seconds: 30, stage: "transcribing" });
    expect(updated?.pcm16Seconds).toBe(30);
    expect((await repo.get(A))?.stage).toBe("transcribing");
    expect(await repo.update("missing" as SourceId, { pcm16Seconds: 1 })).toBeUndefined();
  });

  it("treats a record written by a newer build as absent, and removes it", async () => {
    table("sources").set(JSON.stringify(A), { key: A, value: { schemaVersion: 2, value: record(A, 1) } });
    expect(await repo.get(A)).toBeUndefined();
    expect(table("sources").size).toBe(0);
  });

  it("keeps each source's windows under their own compound keys", async () => {
    await repo.putWindow(A, 0, { words: [], lengthMs: 90_000, modelVersion: "m" });
    await repo.putWindow(A, 3, { words: [{ text: "hi", startMs: 1, endMs: 2, confidence: 1 }], lengthMs: 90_000, modelVersion: "m" });
    await repo.putWindow(B, 0, { words: [], lengthMs: 50_000, modelVersion: "m" });
    const a = await repo.windows(A);
    expect([...a.keys()].sort()).toEqual([0, 3]);
    expect(a.get(3)?.words[0]?.text).toBe("hi");
    expect([...(await repo.windows(B)).keys()]).toEqual([0]);
  });

  it("removes one source whole and leaves the other", async () => {
    await repo.put(record(A, 1));
    await repo.put(record(B, 2));
    await repo.putWindow(A, 0, { words: [], lengthMs: 1, modelVersion: "m" });
    await repo.putWindow(B, 0, { words: [], lengthMs: 1, modelVersion: "m" });
    await repo.putMaster(A, { words: [] } as unknown as Transcript);
    await repo.removeSource(A);
    expect(await repo.get(A)).toBeUndefined();
    expect((await repo.windows(A)).size).toBe(0);
    expect(await repo.getMaster(A)).toBeUndefined();
    expect(await repo.get(B)).toBeDefined();
    expect((await repo.windows(B)).size).toBe(1);
    expect(memory.removed).toEqual(["sources/a-source"]);
  });

  it("sweeps every source before a new one, and nothing of the clips", async () => {
    await repo.put(record(A, 1));
    await repo.put(record(B, 2));
    await repo.putWindow(B, 1, { words: [], lengthMs: 1, modelVersion: "m" });
    await repo.removeAll();
    expect(await repo.newest()).toBeUndefined();
    expect((await repo.windows(B)).size).toBe(0);
    expect(memory.removed).toContain("sources");
    expect(memory.removed.every((p) => p.startsWith("sources"))).toBe(true);
  });
});
