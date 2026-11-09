// @vitest-environment jsdom
// The long job (v6.1implementation §9.5, §11.3): the stages in order, a resume
// from transcribing without the file, extracting without the file asks for it,
// the pauses, the wake lock taken and let go, the tab's title, and a cancel
// that removes the source.

import { beforeEach, describe, expect, it, vi } from "vitest";

import type { SourceInfo, Transcript } from "../gen/domain";
import type { SourceId, SourceRecord } from "../persistence/schema";

type Fake = {
  extract: unknown;
  transcribe: unknown;
  updates: Record<string, unknown>[];
  removed: string[];
  held: number;
  released: number;
  titles: string[];
  extractCalls: number;
};
const fake: Fake = vi.hoisted((): Fake => ({
  extract: undefined,
  transcribe: undefined,
  updates: [] as Record<string, unknown>[],
  removed: [] as string[],
  held: 0,
  released: 0,
  titles: [] as string[],
  extractCalls: 0,
}));

vi.mock("../workers/pool", () => ({
  pool: {
    media: {
      extractSourcePcm16: (_p: unknown, o?: { onProgress?: (p: { done: number }) => void }) => {
        fake.extractCalls += 1;
        o?.onProgress?.({ done: 30_000 });
        return Promise.resolve(fake.extract);
      },
    },
    cancel: () => Promise.resolve("cancelled"),
  },
  WorkerCallError: class extends Error {},
}));
vi.mock("../persistence/sources-repo", () => ({
  update: (_id: string, change: Record<string, unknown>) => {
    fake.updates.push(change);
    return Promise.resolve({ ...RECORD, ...change });
  },
  removeSource: (id: string) => {
    fake.removed.push(id);
    return Promise.resolve();
  },
}));
vi.mock("./transcribe-source", () => ({
  windowCount: () => 9,
  transcribeSourceWindows: (p: { onWindow: (d: number, t: number) => void }) => {
    p.onWindow(9, 9);
    return fake.transcribe instanceof Error ? Promise.reject(fake.transcribe) : Promise.resolve(fake.transcribe);
  },
}));
vi.mock("../platform/wake-lock", () => ({
  holdAwake: () => {
    fake.held += 1;
    return {
      release: () => {
        fake.released += 1;
      },
    };
  },
  showProgressInTitle: (percent: number | null, page: string) => {
    fake.titles.push(percent === null ? page : `${String(Math.round(percent))}%`);
  },
}));

const RECORD: SourceRecord = {
  sourceId: "s1" as SourceId,
  createdAt: 1,
  lastOpenedAt: 1,
  sourceInfo: { duration: 720_000 } as SourceInfo,
  sourceBytes: 8 as SourceRecord["sourceBytes"],
  sourceHashPrefix: "h",
  stage: "extracting_audio",
  pcm16Seconds: 0,
  levelP95: null,
  windowsDone: 0,
  windowsTotal: 0,
};

const { runSourcePipeline, cancelSource } = await import("./run-source-pipeline");
const { useSourceStore } = await import("../state/source-store");
const TRANSCRIPT = { words: [] } as unknown as Transcript;
const file = new File([new Uint8Array(8)], "x.mp4");

describe("run-source-pipeline", () => {
  beforeEach(() => {
    fake.extract = { done: true, levelP95: -21 };
    fake.transcribe = { done: true, transcript: TRANSCRIPT };
    fake.updates.length = 0;
    fake.removed.length = 0;
    fake.titles.length = 0;
    fake.held = 0;
    fake.released = 0;
    fake.extractCalls = 0;
    document.title = "Offcut";
    useSourceStore.setState({ status: "extracting_audio", sourceId: "s1" as SourceId, paused: null, failure: null });
  });

  it("writes the sound, then transcribes, then ends transcribed, holding the screen awake meanwhile", async () => {
    await runSourcePipeline(RECORD, file);
    const state = useSourceStore.getState();
    expect(state.status).toBe("transcribed");
    expect(state.transcript).toBe(TRANSCRIPT);
    // The checkpoint of 30 s, then the stage, then the end.
    expect(fake.updates).toContainEqual({ pcm16Seconds: 30 });
    expect(fake.updates).toContainEqual(expect.objectContaining({ stage: "transcribing", levelP95: -21, windowsTotal: 9 }));
    expect(fake.updates).toContainEqual({ stage: "transcribed" });
    expect(fake.held).toBe(1);
    expect(fake.released).toBeGreaterThanOrEqual(1);
    expect(fake.titles[0]).toMatch(/%$/);
    expect(fake.titles.at(-1)).toBe("Offcut");
  });

  it("continues a reloaded source from transcribing without its file", async () => {
    useSourceStore.setState({ status: "transcribing" });
    await runSourcePipeline({ ...RECORD, stage: "transcribing", levelP95: -21 }, undefined);
    expect(fake.extractCalls).toBe(0);
    expect(useSourceStore.getState().status).toBe("transcribed");
  });

  it("asks for the file when extraction has none, and fails with the worker's code", async () => {
    // A cancel lets go of the file an earlier run held.
    useSourceStore.setState({ status: "paused" });
    await cancelSource();
    useSourceStore.setState({ status: "extracting_audio" });
    await runSourcePipeline(RECORD, undefined);
    expect(useSourceStore.getState().status).toBe("failed");
    expect(useSourceStore.getState().failure?.code).toBe("E_SOURCE_READ");
    useSourceStore.setState({ status: "extracting_audio", failure: null });
    fake.extract = { failed: "E_SOURCE_AUDIO", detail: "EncodingError" };
    await runSourcePipeline(RECORD, file);
    expect(useSourceStore.getState().failure).toEqual({ code: "E_SOURCE_AUDIO", detail: "EncodingError" });
  });

  it("pauses with the reason and lets go of the screen", async () => {
    fake.transcribe = { paused: "busy" };
    await runSourcePipeline(RECORD, file);
    const state = useSourceStore.getState();
    expect(state.status).toBe("paused");
    expect(state.paused).toEqual({ reason: "busy", from: "transcribing" });
    expect(fake.released).toBeGreaterThanOrEqual(1);
    expect(fake.titles.at(-1)).toBe("Offcut");
  });

  it("a window step that throws pauses as offline when the browser is offline, and fails otherwise", async () => {
    // A worker whose script cannot be fetched offline (P180's walk).
    fake.transcribe = new Error("worker did not start");
    const online = vi.spyOn(navigator, "onLine", "get").mockReturnValue(false);
    await runSourcePipeline(RECORD, file);
    expect(useSourceStore.getState().status).toBe("paused");
    expect(useSourceStore.getState().paused).toEqual({ reason: "offline", from: "transcribing" });

    online.mockReturnValue(true);
    useSourceStore.setState({ status: "extracting_audio", paused: null, failure: null });
    await runSourcePipeline(RECORD, file);
    expect(useSourceStore.getState().status).toBe("failed");
    expect(useSourceStore.getState().failure?.code).toBe("E_INTERNAL");
    online.mockRestore();
  });

  it("cancel removes the source", async () => {
    useSourceStore.setState({ status: "paused" });
    await cancelSource();
    expect(useSourceStore.getState().status).toBe("cancelled");
    expect(fake.removed).toEqual(["s1"]);
  });
});
