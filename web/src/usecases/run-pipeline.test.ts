// The pipeline and the update rule with fake workers (v4implementation §16.2,
// §20.3, D-90, D-127, D-147, D-152, D-153): a scripted `pool` answers each
// call and records it; a call can be held, and a cancel answers what is
// held as a worker that sees its flag would. No real worker, timer or
// network.

import { beforeEach, describe, expect, it, vi } from "vitest";

import type { ClipInfo, EditPlan, EditState, Prosody, TimeMap, Transcript } from "../gen/domain";
import * as clipStore from "../state/clip-store";
import * as history from "../state/history";
import { useModelStore } from "../state/model-store";
import { cancelJob } from "./cancel-job";
import { update } from "./review-update";
import { runPipeline } from "./run-pipeline";

type Deferred = { promise: Promise<unknown>; resolve(value: unknown): void; reject(error: unknown): void; kind: string };

const m = vi.hoisted(() => {
  const calls: string[] = [];
  const held = new Map<string, Deferred>();
  const answers = new Map<string, (params: unknown) => Promise<unknown>>();
  const events: { name: string; props?: unknown }[] = [];
  /** A worker method: recorded, then held if the test holds it, else answered by its script. */
  const method =
    (name: string) =>
    (params?: unknown): Promise<unknown> => {
      calls.push(name);
      const hold = held.get(name);
      if (hold !== undefined) {
        return hold.promise;
      }
      const answer = answers.get(name);
      return answer === undefined ? Promise.resolve(undefined) : answer(params);
    };
  const pool = {
    media: { extractAudio: method("media.extractAudio") },
    asr: { load: method("asr.load"), transcribe: method("asr.transcribe"), unload: method("asr.unload") },
    audio: {
      runChain: method("audio.runChain"),
      measureProsody: method("audio.measureProsody"),
      assemble: method("audio.assemble"),
      notify: (name: string) => calls.push(`audio.notify:${name}`),
    },
    render: {
      openSession: method("render.openSession"),
      detect: method("render.detect"),
      redetectSentence: method("render.redetectSentence"),
      rulesPlan: method("render.rulesPlan"),
      setScene: method("render.setScene"),
      closeSession: method("render.closeSession"),
      previewPause: method("render.previewPause"),
      previewSeek: method("render.previewSeek"),
      onCrash: () => undefined,
    },
    /** What a worker that sees its cancel flag does: every held call of `kind` answers as cancelled. */
    cancel: (kind: "media" | "asr" | "audio" | "render") => {
      let any = false;
      for (const [name, hold] of held) {
        if (hold.kind === kind) {
          held.delete(name);
          hold.resolve({ cancelled: true });
          any = true;
        }
      }
      return Promise.resolve(any ? "cancelled" : "idle");
    },
    restartWorker: () => undefined,
  };
  return { calls, held, answers, events, pool };
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
  track: (event: { name: string; props?: unknown }) => m.events.push(event),
}));
vi.mock("../models/model-manager", () => ({
  ensureReady: () => Promise.resolve(),
  cacheInfo: () => Promise.resolve({ modelId: "model", modelVersion: "1", bytes: 0 }),
  ModelFailure: class ModelFailure extends Error {},
}));
vi.mock("../persistence/opfs", () => ({
  writeAudio: () => {
    m.calls.push("opfs.writeAudio");
    return Promise.resolve();
  },
  remove: (path: string) => {
    m.calls.push(`opfs.remove:${path}`);
    return Promise.resolve();
  },
  paths: { clipDir: (id: string) => `clips/${id}` },
  OpfsError: class OpfsError extends Error {
    kind = "io";
  },
}));
vi.mock("../persistence/edits-repo", () => ({
  put: () => {
    m.calls.push("editsRepo.put");
    return Promise.resolve();
  },
}));
vi.mock("../persistence/transcript-repo", () => ({
  put: () => {
    m.calls.push("transcriptRepo.put");
    return Promise.resolve();
  },
}));

const { WorkerCallError } = await vi.importActual<typeof import("../workers/rpc")>("../workers/rpc");

const CLIP = "clip-1" as unknown as NonNullable<clipStore.ClipState["clipId"]>;
const INFO = { duration: 20_000 } as unknown as ClipInfo;
const TRANSCRIPT = {
  words: [
    { text: "two", start_ms: 0, end_ms: 400, confidence: 0.9 },
    { text: "apples", start_ms: 400, end_ms: 900, confidence: 0.9 },
  ],
  sentences: [{ words: { start: 0, end: 2 }, start_ms: 0, end_ms: 900 }],
  numbers: [],
  model_version: "test",
} as unknown as Transcript;
const PROSODY = { per_word: [] } as unknown as Prosody;
const PLAN = { items: [] } as unknown as EditPlan;
const IDENTITY: TimeMap = { segments: [{ kind: "source", src: { start: 0, end: 20_000 } }] } as unknown as TimeMap;

/** Word edits from plain indices. */
function words(edits: Record<number, string>): EditState["word_edits"] {
  return edits;
}

function deferred(kind: string): Deferred {
  let resolve: (value: unknown) => void = () => undefined;
  let reject: (error: unknown) => void = () => undefined;
  const promise = new Promise((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject, kind };
}

/** Holds `name` until the test answers it, or a cancel of its worker does. */
function hold(name: string): Deferred {
  const d = deferred(name.split(".")[0] ?? "");
  m.held.set(name, d);
  return d;
}

function answer(name: string, result: unknown): void {
  m.answers.set(name, () => Promise.resolve(result));
}

function fail(name: string, code: string, stage: string): void {
  m.answers.set(name, () => Promise.reject(new WorkerCallError({ code, stage, retryable: true } as never)));
}

/** The answers of a run that goes well. */
function script(): void {
  m.answers.clear();
  answer("media.extractAudio", { pcm16: new Float32Array(16), pcm48: new Float32Array(48) });
  answer("audio.runChain", { loudness: -14 });
  answer("asr.load", { backend: "webgpu" });
  answer("asr.transcribe", { ok: TRANSCRIPT });
  answer("audio.measureProsody", { prosody: PROSODY, out48: new Float32Array(48) });
  answer("render.detect", []);
  answer("render.redetectSentence", []);
  answer("render.rulesPlan", PLAN);
  answer("render.setScene", { chunks: [] });
  answer("audio.assemble", { prog48: new Float32Array(48), loudness: -14 });
}

const clip = () => clipStore.useClipStore.getState();
const named = (name: string) => m.events.filter((event) => event.name === name);

function accepted(): void {
  clipStore.useClipStore.setState(
    { ...clipStore.useClipStore.getInitialState(), status: "accepted", clipId: CLIP, clipInfo: INFO },
    true,
  );
}

beforeEach(() => {
  m.calls.length = 0;
  m.events.length = 0;
  m.held.clear();
  script();
  history.clear();
  useModelStore.setState({ status: "ready" });
  accepted();
});

describe("runPipeline", () => {
  it("runs A, B1 ‖ B2, C, D, E with the rules plan, F with the audio written before it is assembled", async () => {
    await runPipeline(CLIP);
    expect(m.calls).toEqual([
      "media.extractAudio", // A
      "audio.runChain", // B2, started before the speech model hears the clip
      "asr.load", // B1
      "asr.transcribe",
      "asr.unload", // C: the model's memory back before the session opens
      "audio.measureProsody", // D
      "render.openSession", // E
      "render.detect",
      "render.rulesPlan",
      "render.setScene",
      "opfs.writeAudio", // F: the export's copy first (D-127)
      "audio.assemble",
      "transcriptRepo.put",
      "editsRepo.put",
    ]);
    expect(clip()).toMatchObject({ status: "ready", plan: PLAN, timeMap: IDENTITY, transcript: TRANSCRIPT });
    expect(clip().prog48).toBeInstanceOf(Float32Array);
    expect(named("pipeline_done")).toHaveLength(1);
    expect(named("client_error")).toEqual([]);
  });

  it("REJECT_NO_SPEECH: the clip is refused once the chain has settled, its file removed, the audio let go", async () => {
    answer("asr.transcribe", { rejected: "NoSpeech" });
    await runPipeline(CLIP);
    expect(clip()).toMatchObject({ status: "rejected", reject: "REJECT_NO_SPEECH" });
    expect(m.calls).toContain("audio.notify:release");
    expect(m.calls).toContain("opfs.remove:clips/clip-1");
    expect(m.calls).not.toContain("render.openSession");
    expect(named("clip_rejected")).toEqual([{ name: "clip_rejected", props: { reject_reason: "REJECT_NO_SPEECH" } }]);
  });

  it("a failure of the voice chain (B2) is held until the speech model (B1) has settled (D-90)", async () => {
    fail("audio.runChain", "E_DSP", "audio_chain");
    const heard = hold("asr.transcribe");
    const run = runPipeline(CLIP);
    await vi.waitFor(() => expect(m.calls).toContain("asr.transcribe"));
    // The chain has failed already; the clip still processes while the model listens.
    await Promise.resolve();
    expect(clip().status).toBe("processing");
    heard.resolve({ ok: TRANSCRIPT });
    await run;
    expect(clip()).toMatchObject({ status: "failed", failure: { code: "E_DSP", stage: "audio_chain" } });
    expect(m.calls).toContain("asr.unload");
    expect(m.calls).not.toContain("audio.measureProsody");
    expect(m.calls).toContain("audio.notify:release");
    expect(named("client_error")).toEqual([{ name: "client_error", props: { error_code: "E_DSP", stage: "audio_chain" } }]);
  });

  // Five stages a cancel can reach (D-152), after the voice chain has started: the run ends `accepted`, the audio let go.
  it.each([
    ["B1, the speech model listening", "asr.transcribe", "asr", false],
    ["B2, the voice chain still running", "audio.runChain", "asr", false],
    ["D, the prosody", "audio.measureProsody", "asr", false],
    ["E, the scene", "render.setScene", "detect_scene", true],
    ["F, the program's audio", "audio.assemble", "detect_scene", true],
  ])("a cancel during %s ends accepted, with release called", async (_label, name, stage, session) => {
    hold(name);
    const run = runPipeline(CLIP);
    await vi.waitFor(() => expect(m.calls).toContain(name));
    await cancelJob("pipeline");
    await run;
    expect(clip()).toMatchObject({ status: "accepted", cancelled: true, clipInfo: INFO });
    expect(clip().failure).toBeUndefined();
    expect(m.calls).toContain("audio.notify:release");
    expect(m.calls.includes("render.closeSession")).toBe(session);
    expect(named("job_cancelled")).toEqual([{ name: "job_cancelled", props: { stage } }]);
    expect(named("client_error")).toEqual([]);
    expect(named("pipeline_done")).toEqual([]);
  });

  it("a cancel during A, before the chain has started, ends accepted with nothing to let go", async () => {
    hold("media.extractAudio");
    const run = runPipeline(CLIP);
    await vi.waitFor(() => expect(m.calls).toContain("media.extractAudio"));
    await cancelJob("pipeline");
    await run;
    expect(clip()).toMatchObject({ status: "accepted", cancelled: true });
    expect(m.calls).not.toContain("audio.runChain");
    expect(named("job_cancelled")).toEqual([{ name: "job_cancelled", props: { stage: "probe_audio" } }]);
  });

  it("a second cancel while the first runs gets the same promise", async () => {
    hold("asr.transcribe");
    const run = runPipeline(CLIP);
    await vi.waitFor(() => expect(m.calls).toContain("asr.transcribe"));
    const first = cancelJob("pipeline");
    expect(cancelJob("pipeline")).toBe(first);
    await first;
    await run;
    expect(named("job_cancelled")).toHaveLength(1);
  });

  it("a worker crash ends failed, with client_error sent once", async () => {
    fail("asr.transcribe", "E_WORKER_CRASH", "asr");
    await runPipeline(CLIP);
    expect(clip()).toMatchObject({ status: "failed", failure: { code: "E_WORKER_CRASH", stage: "asr" } });
    expect(m.calls).toContain("audio.notify:release");
    expect(named("client_error")).toEqual([{ name: "client_error", props: { error_code: "E_WORKER_CRASH", stage: "asr" } }]);
  });

  it("retry from failed runs again from A and reaches ready", async () => {
    fail("render.setScene", "E_GPU_INIT", "detect_scene");
    await runPipeline(CLIP);
    expect(clip().status).toBe("failed");
    expect(m.calls).toContain("render.closeSession");

    m.calls.length = 0;
    script();
    await runPipeline(CLIP);
    expect(m.calls[0]).toBe("media.extractAudio");
    expect(clip()).toMatchObject({ status: "ready" });
    expect(clip().failure).toBeUndefined();
  });

  it("Process again after a cancel runs from A", async () => {
    hold("asr.transcribe");
    const run = runPipeline(CLIP);
    await vi.waitFor(() => expect(m.calls).toContain("asr.transcribe"));
    await cancelJob("pipeline");
    await run;
    m.calls.length = 0;
    await runPipeline(CLIP);
    expect(m.calls[0]).toBe("media.extractAudio");
    expect(clip()).toMatchObject({ status: "ready", cancelled: false });
  });
});

describe("review-update (D-147)", () => {
  function ready(): void {
    clipStore.useClipStore.setState(
      {
        ...clipStore.useClipStore.getInitialState(),
        status: "ready",
        clipId: CLIP,
        clipInfo: INFO,
        transcript: TRANSCRIPT,
        prosody: PROSODY,
        plan: PLAN,
        timeMap: IDENTITY,
        prog48: new Float32Array(48),
      },
      true,
    );
  }

  beforeEach(() => {
    ready();
  });

  it("mode plan finds the sentence's events again and their plan before the scene", async () => {
    const before = clip().edit;
    await update((edit) => ({ ...edit, word_edits: words({ 0: "three" }) }), "plan", "word_edit");
    expect(m.calls).toEqual(["render.redetectSentence", "render.rulesPlan", "render.setScene", "editsRepo.put"]);
    expect(clip()).toMatchObject({ status: "ready", edit: { word_edits: { 0: "three" } } });
    expect(history.useHistoryStore.getState().past).toEqual([before]);
    expect(named("review_action")).toEqual([{ name: "review_action", props: { action: "word_edit" } }]);
  });

  it("mode scene builds the scene only", async () => {
    await update((edit) => ({ ...edit, style: "bold" }), "scene", "style");
    expect(m.calls).toEqual(["render.setScene", "editsRepo.put"]);
    expect(clip().edit.style).toBe("bold");
    expect(history.useHistoryStore.getState().canUndo).toBe(true);
  });

  it("a failure restores the edit before, keeps the history as it was, and sends no review action", async () => {
    const before = clip().edit;
    fail("render.setScene", "E_INTERNAL", "preview");
    await update((edit) => ({ ...edit, style: "tech" }), "scene", "style");
    expect(clip()).toMatchObject({ status: "ready", edit: before, failure: { code: "E_INTERNAL" }, failureDuring: "edit" });
    expect(history.useHistoryStore.getState()).toMatchObject({ past: [], canUndo: false });
    expect(m.calls).not.toContain("editsRepo.put");
    expect(named("review_action")).toEqual([]);
  });

  it("a failed word edit gives the worker back the events of the edit before", async () => {
    fail("render.setScene", "E_INTERNAL", "preview");
    await update((edit) => ({ ...edit, word_edits: words({ 1: "pears" }) }), "plan", "word_edit");
    expect(clip().edit.word_edits).toEqual({});
    expect(m.calls).toEqual(["render.redetectSentence", "render.rulesPlan", "render.setScene", "render.detect"]);
  });

  it("the history is pushed only for a change that applied", async () => {
    await update((edit) => ({ ...edit, style: "bold" }), "scene", "style");
    fail("render.setScene", "E_INTERNAL", "preview");
    await update((edit) => ({ ...edit, style: "tech" }), "scene", "style");
    expect(history.useHistoryStore.getState().past).toHaveLength(1);
  });
});
