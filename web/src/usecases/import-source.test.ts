// A long video dropped (v6.1implementation §9.3, §11.3): each step's outcome,
// the gates in order, the hand-over of a short file, the sweep before a new
// source, and the blockers both ways (D-416).

import { beforeEach, describe, expect, it, vi } from "vitest";

import type { SourceInfo } from "../gen/domain";

type Fake = { routed: unknown; free: number | null; cloud: "available" | "unavailable"; calls: string[] };
const fake: Fake = vi.hoisted((): Fake => ({ routed: undefined, free: 100e9, cloud: "available", calls: [] }));

vi.mock("../config/features", () => ({ FEATURES: { cloud_transcribe: true, long_video: true, high_res: false } }));
vi.mock("../workers/pool", () => ({
  pool: { media: { routeFile: () => Promise.resolve(fake.routed) } },
  WorkerCallError: class extends Error {},
}));
vi.mock("../persistence/clips-repo", () => ({ hashPrefix: () => Promise.resolve("hash") }));
vi.mock("../persistence/opfs", () => ({ freeBytes: () => Promise.resolve(fake.free) }));
vi.mock("../persistence/sources-repo", () => ({
  removeAll: () => {
    fake.calls.push("removeAll");
    return Promise.resolve();
  },
  put: () => {
    fake.calls.push("put");
    return Promise.resolve();
  },
}));
vi.mock("../state/cloud-store", () => ({ settledAvailability: () => Promise.resolve(fake.cloud) }));
vi.mock("./run-source-pipeline", () => ({
  runSourcePipeline: () => {
    fake.calls.push("run");
    return Promise.resolve();
  },
}));

const { dropSource, recheckGate, startSource, takeHandOff, neededBytes } = await import("./import-source");
const { useSourceStore } = await import("../state/source-store");
const { useAuthStore } = await import("../state/auth-store");
const { useCapabilityStore } = await import("../state/capability-store");
const { useClipStore } = await import("../state/clip-store");
const { forImport, forSource } = await import("../state/blockers");

const INFO = { duration: 720_000 } as SourceInfo;
const file = new File([new Uint8Array(8)], "never-read.mp4");

function signedIn(yes: boolean): void {
  useAuthStore.setState({ status: yes ? "signed_in" : "anonymous" });
}

describe("import-source", () => {
  beforeEach(() => {
    fake.routed = { long: INFO };
    fake.free = 100e9;
    fake.cloud = "available";
    fake.calls.length = 0;
    useSourceStore.setState({ status: "idle" });
    useSourceStore.getState().clear();
    useCapabilityStore.setState({ status: "supported" });
    useClipStore.setState({ status: "idle" });
    signedIn(true);
  });

  it("gates a signed-out person first, and writes nothing", async () => {
    signedIn(false);
    expect(await dropSource(file)).toBe("gated");
    expect(useSourceStore.getState().status).toBe("gated");
    expect(useSourceStore.getState().gate).toEqual({ reason: "signed_out" });
    expect(fake.calls).toEqual([]);
  });

  it("gates on the cloud being off, then on the disk, then is ready", async () => {
    fake.cloud = "unavailable";
    await dropSource(file);
    expect(useSourceStore.getState().gate?.reason).toBe("unavailable");
    fake.cloud = "available";
    fake.free = 1e9;
    await recheckGate();
    expect(useSourceStore.getState().gate).toEqual({ reason: "storage_low", neededBytes: neededBytes(INFO.duration) });
    fake.free = null;
    await recheckGate();
    expect(useSourceStore.getState().gate?.reason).toBe("ready");
  });

  it("starts on Start: the previous source swept, the record made, the job run", async () => {
    await dropSource(file);
    await startSource();
    expect(fake.calls).toEqual(["removeAll", "put", "run"]);
    const state = useSourceStore.getState();
    expect(state.status).toBe("extracting_audio");
    expect(state.sourceId).not.toBeNull();
    expect(state.gate).toBeNull();
  });

  it("hands a short file to the clip page, once", async () => {
    fake.routed = { clip: {} };
    expect(await dropSource(file)).toBe("handed_over");
    expect(useSourceStore.getState().status).toBe("idle");
    expect(takeHandOff("long")).toBeUndefined();
    expect(takeHandOff("clip")).toBe(file);
    expect(takeHandOff("clip")).toBeUndefined();
  });

  it("shows a refusal, and a file that cannot be read as a failure", async () => {
    fake.routed = { rejected: "REJECT_HDR" };
    expect(await dropSource(file)).toBe("rejected");
    expect(useSourceStore.getState().rejected).toBe("REJECT_HDR");
    useSourceStore.getState().clear();
    fake.routed = { failed: "E_SOURCE_READ", detail: "Read" };
    expect(await dropSource(file)).toBe("failed");
    expect(useSourceStore.getState().failure?.code).toBe("E_SOURCE_READ");
  });

  it("is blocked by a busy clip, and a working source blocks a clip (D-416)", async () => {
    useClipStore.setState({ status: "processing" });
    expect(forSource()).toBe("B_PIPELINE_BUSY");
    expect(await dropSource(file)).toBe("blocked");
    useClipStore.setState({ status: "idle" });
    for (const status of ["probing", "extracting_audio", "transcribing"] as const) {
      useSourceStore.setState({ status });
      expect(forImport()).toBe("B_PIPELINE_BUSY");
    }
    for (const status of ["paused", "awaiting_file", "transcribed", "idle"] as const) {
      useSourceStore.setState({ status });
      expect(forImport()).toBeNull();
    }
  });
});
