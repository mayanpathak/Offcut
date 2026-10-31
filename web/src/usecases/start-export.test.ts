// The export's use-case with fake workers (v5implementation §13, §17.3): a
// scripted `pool` answers each call and records it, and a small device stands in
// for OPFS and IndexedDB. `render-cache-repo.ts` is the real one, so a row without
// its file and the key's inputs are its own doing; `crypto.subtle` is real. The
// `render-cache` group is S-9's proof, and each row of §11.3's table is a case.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ChangeSummary, ClipInfo, DetectedEvent, EditPlan, EditState, ExportProfile, TimeMap, Transcript } from "../gen/domain";
import type { CacheInputs } from "../persistence/render-cache-repo";
import { cacheKey, canonicalJson, RENDERER_VERSION } from "../persistence/render-cache-repo";
import { useCapabilityStore } from "../state/capability-store";
import { useClipStore } from "../state/clip-store";
import * as exportStore from "../state/export-store";
import { cancelJob } from "./cancel-job";
import { importClip } from "./import-clip";
import { dismissExport, downloadAgain, startExport } from "./start-export";

type Deferred = { promise: Promise<unknown>; resolve(value: unknown): void };

const m = vi.hoisted(() => {
  const calls: string[] = [];
  const events: { name: string; props?: Record<string, unknown> }[] = [];
  const held = new Map<string, Deferred>();
  const answers = new Map<string, (params: never) => Promise<unknown>>();
  /** The device: the files by path and size, and the render cache's rows. */
  const files = new Map<string, number>();
  const rows = new Map<string, { schemaVersion: number; value: { bytes: number } }>();
  const device = {
    dbFails: false,
    free: null as number | null,
    token: "" as string | undefined,
    clipRecordFails: false,
    vanishes: false,
    forcedSignal: undefined as AbortSignal | undefined,
  };
  const method =
    (name: string) =>
    (params?: never): Promise<unknown> => {
      calls.push(name);
      const hold = held.get(name);
      if (hold !== undefined) {
        return hold.promise;
      }
      const answer = answers.get(name);
      return answer === undefined || params === undefined ? Promise.resolve(undefined) : answer(params);
    };
  const pool = {
    media: { importAndProbe: method("media.importAndProbe") },
    render: { exportProfile: method("render.exportProfile"), exportClip: method("render.exportClip"), notify: () => undefined },
    cancel: () => {
      for (const [name, hold] of held) {
        held.delete(name);
        hold.resolve({ cancelled: true });
      }
      return Promise.resolve("cancelled");
    },
    restartWorker: () => undefined,
  };
  return { calls, events, held, answers, files, rows, device, pool };
});

vi.mock("../workers/pool", async () => {
  const rpc = await vi.importActual<typeof import("../workers/rpc")>("../workers/rpc");
  return {
    pool: m.pool,
    WorkerCallError: rpc.WorkerCallError,
    whenFree: <T>(call: () => Promise<T>) => call(),
    whenReleased: <T>(call: () => Promise<T>) => call(),
  };
});
vi.mock("../analytics/client", () => ({
  track: (event: { name: string; props?: Record<string, unknown> }) => m.events.push(event),
}));
vi.mock("./control-preview", () => ({
  lockForExport: () => {
    m.calls.push("lock");
    return Promise.resolve();
  },
  unlockAfterExport: () => m.calls.push("unlock"),
  reloadPreview: () => {
    m.calls.push("reloadPreview");
    return Promise.resolve();
  },
}));
vi.mock("./cancel-job", async () => {
  const actual = await vi.importActual<typeof import("./cancel-job")>("./cancel-job");
  return { ...actual, signalFor: (scope: "export") => m.device.forcedSignal ?? actual.signalFor(scope) };
});
vi.mock("./run-pipeline", () => ({ runPipeline: () => Promise.resolve(), startTimer: () => undefined }));
vi.mock("../net/asset-fetch", () => ({ fetchAsset: () => Promise.resolve(), SAMPLE_CLIP_PATH: "sample" }));
vi.mock("../persistence/db", () => {
  const db = {
    get: (_store: string, key: string) => {
      m.calls.push("db.get");
      return m.device.dbFails ? Promise.reject(new Error("no database")) : Promise.resolve(m.rows.get(key));
    },
    getAll: () => (m.device.dbFails ? Promise.reject(new Error("no database")) : Promise.resolve([...m.rows.values()])),
    put: (_store: string, value: { schemaVersion: number; value: { bytes: number } }, key: string) => {
      m.calls.push("db.put");
      if (m.device.dbFails) {
        return Promise.reject(new Error("no database"));
      }
      m.rows.set(key, value);
      return Promise.resolve();
    },
    delete: (_store: string, key: string) => {
      m.calls.push("db.delete");
      m.rows.delete(key);
      return Promise.resolve();
    },
    clear: () => {
      m.calls.push("db.clear");
      m.rows.clear();
      return Promise.resolve();
    },
  };
  return { openDb: () => Promise.resolve(db) };
});
vi.mock("../persistence/entitlement-repo", () => ({
  get: () => Promise.resolve(m.device.token === undefined ? undefined : { token: m.device.token, storedAt: 0 }),
}));
vi.mock("../persistence/clips-repo", () => ({
  get: () =>
    m.device.clipRecordFails ? Promise.reject(new Error("no database")) : Promise.resolve({ sourceHashPrefix: "source-hash" }),
  toBytes: (n: number) => n,
  clear: () => Promise.resolve(),
  hashPrefix: () => Promise.resolve("source-hash"),
}));
vi.mock("../persistence/transcript-repo", () => ({ clear: () => Promise.resolve() }));
vi.mock("../persistence/edits-repo", () => ({ clear: () => Promise.resolve() }));
vi.mock("../persistence/opfs", () => {
  class OpfsError extends Error {
    kind = "io";
  }
  return {
    OpfsError,
    paths: {
      clipsRoot: () => "clips",
      clipDir: (id: string) => `clips/${id}`,
      exportsRoot: () => "exports",
      exportTmp: (id: string) => `exports/tmp/${id}.mp4`,
      exportFinal: (id: string) => `exports/${id}.mp4`,
    },
    remove: (path: string) => {
      m.calls.push(`opfs.remove:${path}`);
      for (const file of m.files.keys()) {
        if (file === path || file.startsWith(`${path}/`)) {
          m.files.delete(file);
        }
      }
      return Promise.resolve();
    },
    size: (path: string) => Promise.resolve(m.files.get(path) ?? null),
    getFile: (path: string) => {
      const bytes = m.device.vanishes ? undefined : m.files.get(path);
      return bytes === undefined ? Promise.reject(new OpfsError("no such file")) : Promise.resolve(new File([new Uint8Array(bytes)], "stored"));
    },
    readProgram: () => {
      m.calls.push("opfs.readProgram");
      return Promise.resolve(new Float32Array(4));
    },
    freeBytes: () => Promise.resolve(m.device.free),
  };
});

const { WorkerCallError } = await vi.importActual<typeof import("../workers/rpc")>("../workers/rpc");

const CLIP = "clip-1" as unknown as NonNullable<ReturnType<typeof useClipStore.getState>["clipId"]>;
const INFO = { duration: 20_000 } as unknown as ClipInfo;
const FILE_BYTES = 1234;
const SUMMARY: ChangeSummary = { voice_cleaned: true, captions_emphasized: 1, visual_moments: 2 };

const CREATOR = { kind: "creator", width: 1080, height: 1920, watermark: false, video_bitrate: 8_000_000, audio_bitrate: 160_000 } as unknown as ExportProfile;
const FREE = { kind: "free", width: 720, height: 1280, watermark: true, video_bitrate: 4_000_000, audio_bitrate: 160_000 } as unknown as ExportProfile;
/** What the Creator profile needs for `INFO`'s 20 s, with the margin of TS §12.5. */
const CREATOR_ROOM = ((8_000_000 + 160_000) * 20_000 * 1.2) / 8000;

const TRANSCRIPT = { words: [{ text: "two", start_ms: 0, end_ms: 400 }], sentences: [], numbers: [], model_version: "test" } as unknown as Transcript;
const EVENTS = [{ id: "event-1", enabled: true }] as unknown as DetectedEvent[];
const PLAN = { format: 1, items: [], cuts: [] } as unknown as EditPlan;
const TIME_MAP = { segments: [{ kind: "source" }] } as unknown as TimeMap;
const EDIT: EditState = { word_edits: {}, event_overrides: {}, style: "clean", crop_offset: 0, caption_position: 1, highlight_numbers: true };

/** A token as `planOf` reads it, naming `plan`: what the worker answers is what counts. */
function tokenNaming(plan: string): string {
  return `${btoa(JSON.stringify({ plan }))}.signature`;
}

const anchors: { download: string; href: string }[] = [];

/** The worker answers `profile` for any token; `exportClip` writes a file and answers. */
function workerGives(profile: ExportProfile): void {
  m.answers.set("render.exportProfile", () => Promise.resolve(profile));
}

function refuse(name: string, code: string, stage = "render_encode"): void {
  m.answers.set(name, () => Promise.reject(new WorkerCallError({ code, stage, retryable: false } as never)));
}

const store = () => exportStore.useExportStore.getState();
const named = (name: string) => m.events.filter((event) => event.name === name);
const has = (call: string) => m.calls.includes(call);
const swept = () => m.calls.some((call) => call === "opfs.remove:exports");

function editIs(edit: Partial<EditState>): void {
  useClipStore.setState({ edit: { ...EDIT, ...edit } });
}

/** One export to its end, with the record of the one before it forgotten. */
async function exportNow(): Promise<void> {
  m.calls.length = 0;
  m.events.length = 0;
  await startExport(CLIP);
}

/** The clock's own timer: the tests fake the one a delivery leaves behind. */
const realSetTimeout = globalThis.setTimeout;

/** Lets what the export waits for (a real digest among it) come through, until `done` holds. */
async function until(done: () => boolean): Promise<void> {
  for (let i = 0; i < 200 && !done(); i += 1) {
    await new Promise((resolve) => realSetTimeout(resolve, 1));
  }
  expect(done()).toBe(true);
}

beforeEach(() => {
  vi.useFakeTimers({ toFake: ["setTimeout"] });
  m.calls.length = 0;
  m.events.length = 0;
  m.held.clear();
  m.answers.clear();
  m.files.clear();
  m.rows.clear();
  Object.assign(m.device, { dbFails: false, free: null, token: tokenNaming("creator"), clipRecordFails: false, vanishes: false, forcedSignal: undefined });
  anchors.length = 0;
  vi.stubGlobal("document", {
    createElement: () => {
      const anchor = { download: "", href: "", click: () => m.calls.push("deliver"), remove: () => undefined };
      anchors.push(anchor);
      return anchor;
    },
    body: { append: () => undefined },
  });
  vi.spyOn(URL, "createObjectURL").mockReturnValue("blob:offcut");
  vi.spyOn(URL, "revokeObjectURL").mockReturnValue(undefined);
  workerGives(CREATOR);
  m.answers.set("render.exportClip", (p: { exportId: string }) => {
    const opfsPath = `exports/${p.exportId}.mp4`;
    m.files.set(opfsPath, FILE_BYTES);
    return Promise.resolve({
      opfsPath,
      summary: SUMMARY,
      stageTimings: [
        { stage: "render_encode", durationMs: 100 },
        { stage: "mux", durationMs: 10 },
      ],
    });
  });
  exportStore.useExportStore.setState(exportStore.useExportStore.getInitialState(), true);
  useClipStore.setState(
    {
      ...useClipStore.getInitialState(),
      status: "ready",
      clipId: CLIP,
      clipInfo: INFO,
      transcript: TRANSCRIPT,
      events: EVENTS,
      plan: PLAN,
      timeMap: TIME_MAP,
      edit: EDIT,
    },
    true,
  );
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("render-cache: the key (S-9)", () => {
  const BASE: CacheInputs = {
    sourceHashPrefix: "source-hash",
    transcript: TRANSCRIPT,
    events: EVENTS,
    edit: EDIT,
    plan: PLAN,
    timeMap: TIME_MAP,
    theme: "default",
    musicId: null,
    profileKind: "creator",
    rendererVersion: "version-1",
  };

  it.each<[string, Partial<CacheInputs>]>([
    ["the source's hash", { sourceHashPrefix: "another" }],
    ["the transcript", { transcript: { ...TRANSCRIPT, model_version: "other" } }],
    ["the events", { events: [] }],
    ["the edit", { edit: { ...EDIT, style: "bold" } }],
    ["the plan", { plan: { ...PLAN, format: 2 } }],
    ["the time map", { timeMap: { segments: [] } }],
    ["the theme", { theme: "a kit" }],
    ["the music", { musicId: "a song" }],
    ["the profile kind", { profileKind: "free" }],
    ["the renderer version", { rendererVersion: "version-2" }],
  ])("changes with %s", async (_input, change) => {
    expect(await cacheKey({ ...BASE, ...change })).not.toBe(await cacheKey(BASE));
  });

  it.each<[string, Partial<EditState>]>([
    ["a word", { word_edits: { 3: "fixed" } as EditState["word_edits"] }],
    ["a visual off", { event_overrides: { "event-1": false } as EditState["event_overrides"] }],
    ["the style", { style: "tech" }],
    ["the crop", { crop_offset: 0.5 }],
    ["the caption place", { caption_position: 0.2 }],
    ["Highlight numbers", { highlight_numbers: false }],
  ])("changes with each part of the edit: %s", async (_part, change) => {
    expect(await cacheKey({ ...BASE, edit: { ...EDIT, ...change } })).not.toBe(await cacheKey(BASE));
  });

  it("is lower-case hex SHA-256", async () => {
    expect(await cacheKey(BASE)).toMatch(/^[0-9a-f]{64}$/);
  });

  it("does not change with the order the keys were made in, at any depth", async () => {
    const one = { ...EDIT, event_overrides: { "event-1": false, "event-2": true } } as unknown as EditState;
    const other = { ...EDIT, event_overrides: { "event-2": true, "event-1": false } } as unknown as EditState;
    expect(await cacheKey({ ...BASE, edit: other })).toBe(await cacheKey({ ...BASE, edit: one }));
    const turned = Object.fromEntries(Object.entries(BASE).reverse()) as unknown as CacheInputs;
    expect(await cacheKey(turned)).toBe(await cacheKey(BASE));
  });

  it("does not change with the order of word_edits' keys, or with an edit made and undone", async () => {
    const one = { ...EDIT, word_edits: { 3: "a", 7: "b" } } as unknown as EditState;
    const other = { ...EDIT, word_edits: { 7: "b", 3: "a" } } as unknown as EditState;
    expect(await cacheKey({ ...BASE, edit: other })).toBe(await cacheKey({ ...BASE, edit: one }));
    const made: Record<number, string> = { 3: "a" };
    delete made[3];
    expect(await cacheKey({ ...BASE, edit: { ...EDIT, word_edits: made } })).toBe(await cacheKey(BASE));
  });

  it("canonicalJson sorts at every depth, leaves out undefined and keeps an array's order", () => {
    expect(canonicalJson({ b: [{ d: 1, c: 2 }, 1], a: undefined, e: null })).toBe('{"b":[{"c":2,"d":1},1],"e":null}');
  });

  it("the version reads \"test\" where no bundler ran", () => {
    expect(RENDERER_VERSION).toBe("test");
  });
});

describe("a miss", () => {
  it("runs in order: lock, the verified profile, the lookup, the program, the render, the row, the delivery, the unlock", async () => {
    const path: string[] = [];
    const stop = exportStore.useExportStore.subscribe((state) => {
      if (state.status !== path.at(-1)) {
        path.push(state.status);
      }
    });
    await exportNow();
    stop();
    const order = ["lock", "render.exportProfile", "db.get", "db.clear", "opfs.remove:exports", "opfs.readProgram", "render.exportClip", "db.put", "deliver", "unlock"];
    expect(m.calls).toEqual(order);
    expect(path).toEqual(["gating", "rendering", "muxing", "saving", "done"]);
    expect(store().summary).toEqual(SUMMARY);
    expect(named("export_started")).toEqual([{ name: "export_started", props: { profile: "creator" } }]);
    expect(named("stage_timing")).toHaveLength(2);
    expect(named("export_done")[0]?.props).toMatchObject({ profile: "creator", from_cache: false });
    expect(m.rows.size).toBe(1);
    const row = [...m.rows.values()][0]?.value;
    expect(row).toMatchObject({ clipId: CLIP, exportId: store().exportId, bytes: FILE_BYTES, summary: SUMMARY });
    expect(JSON.stringify(row)).toMatch(/"opfsPath":"exports\/[^"/]+\.mp4"/);
  });

  it("names the download by the day and the minute, and nothing of the source", async () => {
    await exportNow();
    expect(anchors[0]?.download).toMatch(/^offcut-\d{8}-\d{4}\.mp4$/);
  });

  it("a word edited is a miss", async () => {
    await exportNow();
    editIs({ word_edits: { 0: "three" } as EditState["word_edits"] });
    await exportNow();
    expect(has("render.exportClip")).toBe(true);
    expect(m.rows.size).toBe(1);
    expect(m.files.size).toBe(1);
  });

  it.each<[string, Partial<EditState>]>([
    ["a visual off", { event_overrides: { "event-1": false } as EditState["event_overrides"] }],
    ["the style", { style: "bold" }],
    ["the crop", { crop_offset: 0.5 }],
    ["the caption place", { caption_position: 0.2 }],
    ["Highlight numbers", { highlight_numbers: false }],
  ])("%s is a miss", async (_part, change) => {
    await exportNow();
    editIs(change);
    await exportNow();
    expect(has("render.exportClip")).toBe(true);
  });

  it("a stored file deleted by hand: the row goes, and it renders", async () => {
    await exportNow();
    m.files.clear();
    await exportNow();
    expect(has("db.delete")).toBe(true);
    expect(has("render.exportClip")).toBe(true);
    expect(store().status).toBe("done");
    expect(m.rows.size).toBe(1);
  });

  it("a file of another size is not the stored one", async () => {
    await exportNow();
    for (const file of m.files.keys()) {
      m.files.set(file, FILE_BYTES - 1);
    }
    await exportNow();
    expect(has("render.exportClip")).toBe(true);
  });

  it("a row of a newer build is absent, and removed", async () => {
    await exportNow();
    for (const row of m.rows.values()) {
      row.schemaVersion = 99;
    }
    await exportNow();
    expect(has("db.delete")).toBe(true);
    expect(has("render.exportClip")).toBe(true);
  });

  it("a database that fails: a miss, the export goes on, and no row is needed", async () => {
    m.device.dbFails = true;
    await exportNow();
    expect(has("render.exportClip")).toBe(true);
    expect(store().status).toBe("done");
    expect(m.rows.size).toBe(0);
  });

  it("the clip's record cannot be read: no key, no lookup, no row, and the export goes on", async () => {
    m.device.clipRecordFails = true;
    await exportNow();
    expect(has("db.get")).toBe(false);
    expect(has("db.put")).toBe(false);
    expect(store().status).toBe("done");
  });

  it("a clip that is changing has no key", async () => {
    useClipStore.setState({ status: "updating" });
    await startExport(CLIP);
    expect(has("render.exportProfile")).toBe(false);
    useClipStore.setState({ status: "ready" });
    m.answers.set("render.exportProfile", () => {
      useClipStore.setState({ status: "updating" });
      return Promise.resolve(CREATOR);
    });
    await exportNow();
    expect(has("db.get")).toBe(false);
    expect(has("db.put")).toBe(false);
  });
});

describe("a hit", () => {
  it("nothing changed: no render, no sweep, no export_started, no stage_timing; the file under its own id", async () => {
    await exportNow();
    const original = store().exportId;
    const firstFile = [...m.files.keys()];
    await exportNow();
    expect(has("render.exportClip")).toBe(false);
    expect(has("opfs.readProgram")).toBe(false);
    expect(swept()).toBe(false);
    expect(has("db.clear")).toBe(false);
    expect(has("db.put")).toBe(false);
    expect(named("export_started")).toEqual([]);
    expect(named("stage_timing")).toEqual([]);
    expect(named("export_done")[0]?.props).toMatchObject({ from_cache: true, profile: "creator" });
    expect(store().status).toBe("done");
    expect(store().exportId).toBe(original);
    expect(store().summary).toEqual(SUMMARY);
    expect([...m.files.keys()]).toEqual(firstFile);
    expect(has("deliver")).toBe(true);
    expect(has("unlock")).toBe(true);
    expect(anchors.at(-1)?.download).toMatch(/^offcut-\d{8}-\d{4}\.mp4$/);
  });

  it("goes gating, saving, done", async () => {
    await exportNow();
    const path: string[] = [];
    const stop = exportStore.useExportStore.subscribe((state) => {
      if (state.status !== path.at(-1)) {
        path.push(state.status);
      }
    });
    await exportNow();
    stop();
    expect(path).toEqual(["idle", "gating", "saving", "done"]);
  });

  it("a word edited and then undone is a hit", async () => {
    await exportNow();
    editIs({ word_edits: { 0: "three" } as EditState["word_edits"] });
    editIs({ word_edits: {} });
    await exportNow();
    expect(has("render.exportClip")).toBe(false);
  });

  it("two changes made in the other order are a hit", async () => {
    editIs({ event_overrides: { a: true, b: false } as unknown as EditState["event_overrides"] });
    await exportNow();
    editIs({ event_overrides: { b: false, a: true } as unknown as EditState["event_overrides"] });
    await exportNow();
    expect(has("render.exportClip")).toBe(false);
  });

  it("a hit after the question was answered does not ask again; a new render does", async () => {
    await exportNow();
    exportStore.answered();
    await exportNow();
    expect(store().answered).toBe(true);
    editIs({ style: "bold" });
    await exportNow();
    expect(store().answered).toBe(false);
  });

  it("Download again hands over the stored file", async () => {
    await exportNow();
    await exportNow();
    m.calls.length = 0;
    await downloadAgain();
    expect(m.calls).toEqual(["deliver"]);
  });

  it("a file that vanishes after the lookup: failed, with export_failed and no export_started", async () => {
    await exportNow();
    // The lookup reads the size; the file is gone when it is opened.
    m.device.vanishes = true;
    await exportNow();
    expect(store().status).toBe("failed");
    expect(store().failure?.code).toBe("E_STORAGE_IO");
    expect(named("export_started")).toEqual([]);
    expect(named("export_failed")).toEqual([{ name: "export_failed", props: { error_code: "E_STORAGE_IO", stage: "storage" } }]);
    expect(has("unlock")).toBe(true);
  });
});

describe("the verified profile (INV-9, D-220)", () => {
  it("a token that names creator and that the worker verifies as free: the key has free, and analytics says free", async () => {
    m.device.token = tokenNaming("creator");
    workerGives(FREE);
    await exportNow();
    const inputs: CacheInputs = {
      sourceHashPrefix: "source-hash",
      transcript: TRANSCRIPT,
      events: EVENTS,
      edit: EDIT,
      plan: PLAN,
      timeMap: TIME_MAP,
      theme: "default",
      musicId: null,
      profileKind: "free",
      rendererVersion: RENDERER_VERSION,
    };
    expect([...m.rows.keys()]).toEqual([await cacheKey(inputs)]);
    expect(named("export_started")[0]?.props).toEqual({ profile: "free" });
    expect(named("export_done")[0]?.props).toMatchObject({ profile: "free" });
  });

  it("Creator, then the same token after its period ended: a miss. The Creator file is not given", async () => {
    await exportNow();
    const creatorFile = [...m.files.keys()];
    // The token's words have not changed; what the worker verifies has.
    workerGives(FREE);
    await exportNow();
    expect(has("render.exportClip")).toBe(true);
    expect(swept()).toBe(true);
    for (const file of creatorFile) {
      expect(m.files.has(file)).toBe(false);
    }
  });

  it("Creator, then Free: a miss", async () => {
    await exportNow();
    m.device.token = tokenNaming("free");
    workerGives(FREE);
    await exportNow();
    expect(has("render.exportClip")).toBe(true);
    expect(named("export_started")[0]?.props).toEqual({ profile: "free" });
  });

  it("a forged token after a hit's row exists: no lookup, nothing swept, and the stored file stays usable", async () => {
    await exportNow();
    const stored = [...m.files.keys()];
    refuse("render.exportProfile", "E_ENTITLEMENT_INVALID");
    await exportNow();
    expect(has("db.get")).toBe(false);
    expect(swept()).toBe(false);
    expect(has("deliver")).toBe(false);
    expect(store().status).toBe("failed");
    expect([...m.files.keys()]).toEqual(stored);
    workerGives(CREATOR);
    await exportNow();
    expect(has("render.exportClip")).toBe(false);
    expect(store().status).toBe("done");
  });
});

describe("a token refused (D-230)", () => {
  it("goes gating, rendering, failed; export_started from the token's words, then export_failed; nothing touched", async () => {
    const path: string[] = [];
    const stop = exportStore.useExportStore.subscribe((state) => {
      if (state.status !== path.at(-1)) {
        path.push(state.status);
      }
    });
    m.device.token = tokenNaming("creator");
    refuse("render.exportProfile", "E_ENTITLEMENT_INVALID");
    await exportNow();
    stop();
    expect(path).toEqual(["gating", "rendering", "failed"]);
    expect(store().failure?.code).toBe("E_ENTITLEMENT_INVALID");
    expect(m.events.map((event) => event.name)).toEqual(["export_started", "export_failed"]);
    expect(named("export_started")[0]?.props).toEqual({ profile: "creator" });
    expect(named("export_failed")[0]?.props).toEqual({ error_code: "E_ENTITLEMENT_INVALID", stage: "render_encode" });
    expect(has("db.get")).toBe(false);
    expect(swept()).toBe(false);
    expect(has("unlock")).toBe(true);
  });

  it("no token at all: nothing begins", async () => {
    m.device.token = undefined;
    await exportNow();
    expect(store().unavailable).toBe(true);
    expect(store().status).toBe("idle");
    expect(m.calls).toEqual([]);
  });

  it("a cancel that lands in the gate: back to idle, not cancelled", async () => {
    const stopped = new AbortController();
    stopped.abort();
    m.device.forcedSignal = stopped.signal;
    await exportNow();
    expect(store().status).toBe("idle");
    expect(has("render.exportProfile")).toBe(false);
    expect(m.events).toEqual([]);
    expect(has("unlock")).toBe(true);
  });
});

describe("the room on the device (D-229, D-231)", () => {
  it("too little: back to idle with the code, nothing swept, nothing announced, unlocked; the next start clears it", async () => {
    m.device.free = CREATOR_ROOM - 1;
    await exportNow();
    expect(store().status).toBe("idle");
    expect(store().blocker).toBe("B_STORAGE_LOW");
    expect(swept()).toBe(false);
    expect(has("db.clear")).toBe(false);
    expect(has("render.exportClip")).toBe(false);
    expect(m.events).toEqual([]);
    expect(has("unlock")).toBe(true);
    m.device.free = CREATOR_ROOM;
    await exportNow();
    expect(store().blocker).toBeUndefined();
    expect(store().status).toBe("done");
  });

  it("no estimate: it goes on", async () => {
    m.device.free = null;
    await exportNow();
    expect(store().status).toBe("done");
  });

  it("the cached file's bytes count as free: a miss deletes them", async () => {
    await exportNow();
    editIs({ style: "bold" });
    m.device.free = CREATOR_ROOM - FILE_BYTES;
    await exportNow();
    expect(store().status).toBe("done");
    editIs({ style: "tech" });
    m.device.free = CREATOR_ROOM - FILE_BYTES - 1;
    await exportNow();
    expect(store().blocker).toBe("B_STORAGE_LOW");
    // The row and the file stay: the edit before is still one export away.
    expect(m.rows.size).toBe(1);
    expect(m.files.size).toBe(1);
  });

  it("a hit needs no room", async () => {
    await exportNow();
    m.device.free = 0;
    await exportNow();
    expect(store().status).toBe("done");
    expect(store().blocker).toBeUndefined();
  });
});

describe("failures of the render", () => {
  it.each(["E_ENCODE_VIDEO", "E_ENCODE_AUDIO", "E_STORAGE_QUOTA"])("%s ends failed, with no row", async (code) => {
    refuse("render.exportClip", code);
    await exportNow();
    expect(store().status).toBe("failed");
    expect(store().failure?.code).toBe(code);
    expect(m.rows.size).toBe(0);
    expect(m.events.map((event) => event.name)).toEqual(["export_started", "export_failed"]);
    expect(has("reloadPreview")).toBe(false);
    dismissExport();
    expect(store().status).toBe("idle");
  });

  it("the worker gone: failed, and the preview comes back", async () => {
    refuse("render.exportClip", "E_WORKER_CRASH");
    await exportNow();
    expect(store().failure?.code).toBe("E_WORKER_CRASH");
    expect(has("reloadPreview")).toBe(true);
  });

  it("the worker gone in the gate: failed, and the preview comes back", async () => {
    refuse("render.exportProfile", "E_WORKER_CRASH");
    await exportNow();
    expect(store().status).toBe("failed");
    expect(has("reloadPreview")).toBe(true);
  });

  it("a fault of this file: shown as failed, and not hidden", async () => {
    m.answers.set("render.exportClip", () => Promise.reject(new Error("a bug")));
    await expect(startExport(CLIP)).rejects.toThrow("a bug");
    expect(store().status).toBe("failed");
    expect(store().failure?.code).toBe("E_INTERNAL");
  });
});

describe("cancel", () => {
  it("ends idle, with both files removed, no row and no export_done", async () => {
    let resolve: (value: unknown) => void = () => undefined;
    m.held.set("render.exportClip", { promise: new Promise((res) => (resolve = res)), resolve: (value) => { resolve(value); } });
    const running = startExport(CLIP);
    await until(() => has("render.exportClip"));
    expect(store().status).toBe("rendering");
    const id = store().exportId;
    await cancelJob("export");
    await running;
    expect(store().status).toBe("idle");
    expect(store().answered).toBe(false);
    expect(has(`opfs.remove:exports/tmp/${String(id)}.mp4`)).toBe(true);
    expect(has(`opfs.remove:exports/${String(id)}.mp4`)).toBe(true);
    expect(m.rows.size).toBe(0);
    expect(named("export_done")).toEqual([]);
    expect(named("job_cancelled")).toHaveLength(1);
    expect(has("unlock")).toBe(true);
  });
});

describe("a second click", () => {
  it("meets the gate: one export runs", async () => {
    const first = startExport(CLIP);
    const second = startExport(CLIP);
    await Promise.all([first, second]);
    expect(m.calls.filter((call) => call === "render.exportClip")).toHaveLength(1);
  });
});

describe("a new import of the same file (D-228)", () => {
  it("clears the render cache, so the next export renders", async () => {
    await exportNow();
    expect(m.rows.size).toBe(1);
    useCapabilityStore.setState({ status: "supported" });
    useClipStore.setState(useClipStore.getInitialState(), true);
    exportStore.reset();
    m.answers.set("media.importAndProbe", () => Promise.resolve({ rejected: "REJECT_HEVC" }));
    await importClip(new File([new Uint8Array(1)], "clip.mp4"), "user");
    expect(m.rows.size).toBe(0);

    useClipStore.setState({ status: "ready", clipId: CLIP, clipInfo: INFO, transcript: TRANSCRIPT, events: EVENTS, plan: PLAN, timeMap: TIME_MAP, edit: EDIT });
    await exportNow();
    expect(has("render.exportClip")).toBe(true);
  });
});
