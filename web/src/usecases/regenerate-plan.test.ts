// V6.2: "Regenerate plan" (v6.2implementation §8.3, D-469), with a fake
// request, a fake render worker and the real update queue: a plan that keeps
// an item replaces the clip's and is kept; every failure leaves the clip as it
// was and says why; it asks only for a ready clip, with the flag on.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { CATALOG_VERSION, type ClipInfo, type EditPlan, type EditState, type Prosody, type TimeMap, type Transcript } from "../gen/domain";
import * as clipStore from "../state/clip-store";
import * as history from "../state/history";
import { usePlanStore } from "../state/plan-store";
import { regeneratePlan } from "./regenerate-plan";

type Fakes = {
  flags: { cloud_transcribe: boolean; long_video: boolean; high_res: boolean; ai_plan: boolean; motion: boolean };
  calls: string[];
  choice: unknown;
  inputs: { transcript: unknown; switchOn: boolean; duration: number }[];
  check: unknown;
  checked: { plan: unknown; wordEdits: unknown }[];
  scene: (() => Promise<unknown>) | undefined;
  scenes: { plan: unknown; edit: unknown }[];
  puts: unknown[];
};
const m = vi.hoisted(
  (): Fakes => ({
    flags: { cloud_transcribe: true, long_video: false, high_res: false, ai_plan: true, motion: false },
    calls: [],
    choice: undefined,
    inputs: [],
    check: undefined,
    checked: [],
    scene: undefined,
    scenes: [],
    puts: [],
  }),
);

vi.mock("../config/features", () => ({ FEATURES: m.flags }));
vi.mock("../analytics/client", () => ({ track: () => undefined }));
vi.mock("./request-plan", () => ({
  planFor: (input: { transcript: unknown; switchOn: boolean; duration: number }) => {
    m.calls.push("plan.request");
    m.inputs.push(input);
    return Promise.resolve(m.choice);
  },
}));
vi.mock("../persistence/plans-repo", () => ({
  put: (_clipId: unknown, record: unknown) => {
    m.calls.push("plansRepo.put");
    m.puts.push(record);
    return Promise.resolve();
  },
}));
vi.mock("../persistence/edits-repo", () => ({ put: () => Promise.resolve() }));
vi.mock("./control-preview", () => ({ pause: () => Promise.resolve(), play: () => Promise.resolve() }));
vi.mock("../workers/pool", async () => {
  const rpc = await vi.importActual<typeof import("../workers/rpc")>("../workers/rpc");
  return {
    WorkerCallError: rpc.WorkerCallError,
    whenFree: <T>(call: () => Promise<T>) => call(),
    pool: {
      render: {
        validatePlan: (params: { plan: unknown; wordEdits: unknown }) => {
          m.calls.push("render.validatePlan");
          m.checked.push(params);
          return Promise.resolve(m.check);
        },
        setScene: (params: { plan: unknown; edit: unknown }) => {
          m.calls.push("render.setScene");
          m.scenes.push(params);
          return m.scene === undefined ? Promise.resolve({ dropped: [], chunks: [] }) : m.scene();
        },
      },
    },
  };
});

const { WorkerCallError } = await vi.importActual<typeof import("../workers/rpc")>("../workers/rpc");

const CLIP = "clip-1" as unknown as NonNullable<clipStore.ClipState["clipId"]>;
const INFO = { duration: 20_000 } as unknown as ClipInfo;
const TRANSCRIPT = { words: [{ text: "two" }, { text: "apples" }], sentences: [], numbers: [], model_version: "t" } as unknown as Transcript;
const PROSODY = { per_word: [] } as unknown as Prosody;
const IDENTITY = { segments: [] } as unknown as TimeMap;
const RULES = { source: { kind: "rules" }, items: [{ id: "1" }] } as unknown as EditPlan;
const MODEL = { source: { kind: "model", provider: "deepseek", model: "f" }, catalog: CATALOG_VERSION, items: [{ id: "7" }, { id: "8" }] } as unknown as EditPlan;

/** A ready clip on the rules plan, with one word edit and one visual turned off. */
function ready(): EditState {
  const edit = {
    ...clipStore.useClipStore.getInitialState().edit,
    word_edits: { 1: "pears" },
    event_overrides: { "1": false, "8": false },
  } as unknown as EditState;
  clipStore.useClipStore.setState(
    {
      ...clipStore.useClipStore.getInitialState(),
      status: "ready",
      clipId: CLIP,
      clipInfo: INFO,
      transcript: TRANSCRIPT,
      prosody: PROSODY,
      plan: RULES,
      timeMap: IDENTITY,
      prog48: new Float32Array(48),
      edit,
    },
    true,
  );
  return edit;
}

const clip = () => clipStore.useClipStore.getState();

beforeEach(() => {
  m.flags.ai_plan = true;
  m.calls.length = 0;
  m.inputs.length = 0;
  m.checked.length = 0;
  m.scenes.length = 0;
  m.puts.length = 0;
  m.choice = { source: "model", plan: MODEL, plansRemaining: 12 };
  m.check = { kept: 2, dropped: [] };
  m.scene = undefined;
  usePlanStore.setState({ switchOn: true });
  history.clear();
});

afterEach(() => {
  clipStore.useClipStore.setState(clipStore.useClipStore.getInitialState(), true);
});

describe("regeneratePlan (§8.3)", () => {
  it("a new plan that keeps its items replaces the clip's, with the edit as it was, and is kept", async () => {
    const edit = ready();
    expect(await regeneratePlan()).toEqual({ ok: true });
    expect(m.calls).toEqual(["plan.request", "render.validatePlan", "render.setScene", "plansRepo.put"]);
    // The transcript as heard; the edit's words only for the check, as the scene reads them.
    expect(m.inputs).toEqual([expect.objectContaining({ transcript: TRANSCRIPT, switchOn: true, duration: 20_000 })]);
    expect(m.checked).toEqual([{ transcript: TRANSCRIPT, plan: MODEL, wordEdits: { 1: "pears" } }]);
    expect(m.scenes).toEqual([expect.objectContaining({ plan: MODEL, edit })]);
    expect(clip()).toMatchObject({ status: "ready", plan: MODEL });
    // Overrides are kept: "8" now turns off one of the new plan's items; "1" touches nothing.
    expect(clip().edit).toBe(edit);
    expect(history.useHistoryStore.getState().past).toEqual([]);
    expect(m.puts).toEqual([expect.objectContaining({ plan: MODEL, source: "model", catalog: CATALOG_VERSION })]);
  });

  it.each(["signed_out", "quota", "unavailable", "invalid", "timeout", "offline", "catalog", "off"] as const)(
    "the request's %s: nothing changes, and the reason comes back",
    async (why) => {
      ready();
      m.choice = { source: "rules", why };
      expect(await regeneratePlan()).toEqual({ ok: false, why });
      expect(m.calls).toEqual(["plan.request"]);
      expect(clip()).toMatchObject({ status: "ready", plan: RULES });
    },
  );

  it.each([
    ["every item dropped", { kept: 0, dropped: [{ kind: "emoji", reason: "too_close" }] }, "invalid"],
    ["a newer catalog", { refused: "PlanCatalog" }, "catalog"],
    ["another format", { refused: "PlanFormat" }, "invalid"],
  ])("a plan with %s is not used: nothing changes", async (_, check, why) => {
    ready();
    m.check = check;
    expect(await regeneratePlan()).toEqual({ ok: false, why });
    expect(m.calls).toEqual(["plan.request", "render.validatePlan"]);
    expect(clip().plan).toBe(RULES);
  });

  it("a scene that cannot be built: the plan before stays, the failure is the edit's, nothing kept", async () => {
    ready();
    m.scene = () => Promise.reject(new WorkerCallError({ code: "E_INTERNAL", stage: "preview", retryable: false }));
    expect(await regeneratePlan()).toEqual({ ok: false, why: "busy" });
    expect(clip()).toMatchObject({ status: "ready", plan: RULES, failure: { code: "E_INTERNAL" }, failureDuring: "edit" });
    expect(m.puts).toEqual([]);
  });

  it("a worker that cannot check the plan: failed, nothing changes", async () => {
    ready();
    m.check = undefined;
    const pool = (await import("../workers/pool")).pool as unknown as { render: { validatePlan: () => Promise<unknown> } };
    const before = pool.render.validatePlan;
    pool.render.validatePlan = () => Promise.reject(new WorkerCallError({ code: "E_WORKER_CRASH", stage: "detect_scene", retryable: true }));
    try {
      expect(await regeneratePlan()).toEqual({ ok: false, why: "failed" });
    } finally {
      pool.render.validatePlan = before;
    }
    expect(clip().plan).toBe(RULES);
    expect(m.puts).toEqual([]);
  });

  it("a clip that went while the request was out is not touched", async () => {
    ready();
    m.choice = { source: "model", plan: MODEL, plansRemaining: 1 };
    const other = "clip-2" as unknown as NonNullable<clipStore.ClipState["clipId"]>;
    const pending = regeneratePlan();
    clipStore.useClipStore.setState({ clipId: other });
    expect(await pending).toEqual({ ok: false, why: "busy" });
    expect(m.calls).toEqual(["plan.request"]);
  });

  it("asks nothing for a clip that is not ready, nor with the flag off", async () => {
    ready();
    clipStore.useClipStore.setState({ status: "updating" });
    expect(await regeneratePlan()).toEqual({ ok: false, why: "busy" });
    ready();
    m.flags.ai_plan = false;
    expect(await regeneratePlan()).toEqual({ ok: false, why: "off" });
    expect(m.calls).toEqual([]);
  });
});
