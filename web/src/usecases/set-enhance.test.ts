// V6.2: the Enhance panel's switches (v6.2implementation §11, D-485 to D-488,
// §13.1: undo; remembered; flags off ignore the memory). The real update
// queue, history and prefs over a `meta` held in memory; a fake render worker,
// preview and voice program. A Voice switch, its undo and its redo each swap
// the program's audio; a Motion switch builds the scene only.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ClipInfo, EditPlan, EditState, Enhance, Prosody, TimeMap, Transcript } from "../gen/domain";
import * as clipStore from "../state/clip-store";
import * as history from "../state/history";
import { ENHANCE_RECOMMENDED, resetEnhance, setEnhance, startEnhance } from "./set-enhance";
import { redo, undo } from "./undo-redo";

type Fakes = {
  flags: { cloud_transcribe: boolean; long_video: boolean; high_res: boolean; ai_plan: boolean; motion: boolean };
  meta: Map<string, unknown>;
  metaReads: number;
  scenes: EditState[];
  programs: string[];
  programFails: boolean;
  swapped: Float32Array[];
  tracked: string[];
};
const m = vi.hoisted(
  (): Fakes => ({
    flags: { cloud_transcribe: true, long_video: false, high_res: false, ai_plan: false, motion: true },
    meta: new Map(),
    metaReads: 0,
    scenes: [],
    programs: [],
    programFails: false,
    swapped: [],
    tracked: [],
  }),
);

vi.mock("../config/features", () => ({ FEATURES: m.flags }));
vi.mock("../analytics/client", () => ({
  track: (event: { name: string; props?: { action?: string } }) => {
    m.tracked.push(`${event.name}:${event.props?.action ?? ""}`);
  },
}));
vi.mock("../persistence/db", () => ({
  metaGet: (key: string) => {
    m.metaReads += 1;
    return Promise.resolve(m.meta.get(key));
  },
  metaSet: (key: string, value: unknown) => {
    m.meta.set(key, value);
    return Promise.resolve();
  },
  metaDelete: (key: string) => {
    m.meta.delete(key);
    return Promise.resolve();
  },
}));
vi.mock("../persistence/edits-repo", () => ({ put: () => Promise.resolve() }));
vi.mock("./control-preview", () => ({
  pause: () => Promise.resolve(),
  play: () => Promise.resolve(),
  swapAudio: (prog48: Float32Array) => {
    m.swapped.push(prog48);
  },
}));
vi.mock("./voice-program", async () => {
  const actual = await vi.importActual<typeof import("./voice-program")>("./voice-program");
  const { OpfsError } = await vi.importActual<typeof import("../persistence/opfs")>("../persistence/opfs");
  return {
    voiceOf: actual.voiceOf,
    programOn: (_clipId: unknown, voice: string) => {
      m.programs.push(voice);
      if (m.programFails) {
        return Promise.reject(new OpfsError("io", new Error("gone")));
      }
      // A program marked with its voice: 1 for clean, 2 for level.
      return Promise.resolve(new Float32Array(4).fill(voice === "clean" ? 1 : 2));
    },
  };
});
vi.mock("../workers/pool", async () => {
  const rpc = await vi.importActual<typeof import("../workers/rpc")>("../workers/rpc");
  return {
    WorkerCallError: rpc.WorkerCallError,
    whenFree: <T>(call: () => Promise<T>) => call(),
    pool: {
      render: {
        setScene: (params: { edit: EditState }) => {
          m.scenes.push(params.edit);
          return Promise.resolve({ dropped: [], chunks: [] });
        },
      },
    },
  };
});

const CLIP = "clip-1" as unknown as NonNullable<clipStore.ClipState["clipId"]>;
const START: Enhance = { voice: "clean", motion: true };

/** A ready clip that started on `enhance`, its preview's program the clean one. */
function ready(enhance: Enhance = START): void {
  clipStore.useClipStore.setState(
    {
      ...clipStore.useClipStore.getInitialState(),
      status: "ready",
      clipId: CLIP,
      clipInfo: { duration: 1_000 } as unknown as ClipInfo,
      transcript: { words: [], sentences: [], numbers: [], model_version: "t" } as unknown as Transcript,
      prosody: { per_word: [] } as unknown as Prosody,
      plan: { source: { kind: "rules" }, items: [] } as unknown as EditPlan,
      timeMap: { segments: [] } as unknown as TimeMap,
      prog48: new Float32Array(4).fill(1),
      edit: { ...clipStore.useClipStore.getInitialState().edit, enhance },
    },
    true,
  );
}

const clip = () => clipStore.useClipStore.getState();

beforeEach(() => {
  m.flags.motion = true;
  m.meta.clear();
  m.metaReads = 0;
  m.scenes.length = 0;
  m.programs.length = 0;
  m.programFails = false;
  m.swapped.length = 0;
  m.tracked.length = 0;
  history.clear();
});

afterEach(() => {
  clipStore.useClipStore.setState(clipStore.useClipStore.getInitialState(), true);
});

describe("a new clip's Enhance (D-486)", () => {
  it("starts recommended with nothing remembered, and from the memory once there is one", async () => {
    expect(await startEnhance()).toEqual(ENHANCE_RECOMMENDED);
    m.meta.set("enhanceDefault", { voice: "level", motion: false });
    expect(await startEnhance()).toEqual({ voice: "level", motion: false });
  });

  it("reads a stored value that is not an Enhance as none", async () => {
    for (const bad of ["level", { voice: "loud", motion: true }, { voice: "clean" }, null]) {
      m.meta.set("enhanceDefault", bad);
      expect(await startEnhance(), JSON.stringify(bad)).toEqual(ENHANCE_RECOMMENDED);
    }
  });

  it("with the flag off: no memory is read, the clip keeps V5's output, nothing is switched or forgotten", async () => {
    m.flags.motion = false;
    m.meta.set("enhanceDefault", { voice: "level", motion: false });
    expect(await startEnhance()).toBeUndefined();
    expect(m.metaReads).toBe(0);
    ready();
    await setEnhance({ voice: "level" });
    expect(clip().edit.enhance).toEqual(START);
    expect(m.scenes).toEqual([]);
    await resetEnhance();
    expect(m.meta.get("enhanceDefault")).toEqual({ voice: "level", motion: false });
  });
});

describe("setEnhance (§11, D-488)", () => {
  it("Motion: one step of the history, the scene built with it, no audio touched, remembered, no review_action", async () => {
    ready();
    await setEnhance({ motion: false });
    expect(clip().edit.enhance).toEqual({ voice: "clean", motion: false });
    expect(m.scenes.map((edit) => edit.enhance)).toEqual([{ voice: "clean", motion: false }]);
    expect(m.programs).toEqual([]);
    expect(m.swapped).toEqual([]);
    expect(history.useHistoryStore.getState().past).toHaveLength(1);
    expect(m.meta.get("enhanceDefault")).toEqual({ voice: "clean", motion: false });
    expect(m.tracked).toEqual([]);
  });

  it("Voice: the program on the levelled voice takes the preview's place, and the clip keeps it", async () => {
    ready();
    await setEnhance({ voice: "level" });
    expect(clip().edit.enhance).toEqual({ voice: "level", motion: true });
    expect(m.programs).toEqual(["level"]);
    expect(m.swapped).toHaveLength(1);
    expect(m.swapped[0]?.[0]).toBe(2);
    expect(clip().prog48?.[0]).toBe(2);
    expect(m.meta.get("enhanceDefault")).toEqual({ voice: "level", motion: true });
    expect(clip().status).toBe("ready");
  });

  it("undo and redo swap the voice back and forth; the memory keeps the last switch", async () => {
    ready();
    await setEnhance({ voice: "level" });
    await undo();
    expect(clip().edit.enhance).toEqual(START);
    expect(m.programs).toEqual(["level", "clean"]);
    expect(clip().prog48?.[0]).toBe(1);
    await redo();
    expect(clip().edit.enhance).toEqual({ voice: "level", motion: true });
    expect(m.programs).toEqual(["level", "clean", "level"]);
    expect(clip().prog48?.[0]).toBe(2);
    // An undo is not a switch: what was remembered stays.
    expect(m.meta.get("enhanceDefault")).toEqual({ voice: "level", motion: true });
    expect(m.tracked).toEqual(["review_action:undo", "review_action:redo"]);
  });

  it("the value the clip has already changes nothing", async () => {
    ready();
    await setEnhance({ voice: "clean", motion: true });
    expect(m.scenes).toEqual([]);
    expect(history.useHistoryStore.getState().past).toHaveLength(0);
    expect(m.meta.has("enhanceDefault")).toBe(false);
  });

  it("a voice that cannot be read leaves the clip as it was, says why, and is not remembered", async () => {
    ready();
    m.programFails = true;
    await setEnhance({ voice: "level" });
    expect(clip().edit.enhance).toEqual(START);
    expect(clip().failure).toMatchObject({ code: "E_STORAGE_IO", stage: "storage" });
    expect(clip().prog48?.[0]).toBe(1);
    expect(m.swapped).toEqual([]);
    expect(m.meta.has("enhanceDefault")).toBe(false);
    expect(history.useHistoryStore.getState().past).toHaveLength(0);
  });

  it("Back to recommended forgets the memory; the clip open now keeps its own", async () => {
    ready();
    await setEnhance({ motion: false });
    await resetEnhance();
    expect(m.meta.has("enhanceDefault")).toBe(false);
    expect(clip().edit.enhance).toEqual({ voice: "clean", motion: false });
    expect(await startEnhance()).toEqual(ENHANCE_RECOMMENDED);
  });
});
