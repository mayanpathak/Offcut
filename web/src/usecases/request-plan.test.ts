// V6.2: one clip's plan request (v6.2implementation §8.1, D-466, D-494,
// D-498), with a fake API client and the real auth store: off, signed out and
// offline send nothing; each answer to its outcome; the body is the heard
// transcript and nothing the person typed; the 20 s timer; never a throw.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { CATALOG_VERSION, type ClipInfo, type EditPlan, type EditState, type Prosody, type Transcript } from "../gen/domain";
import { useAuthStore } from "../state/auth-store";
import { useClipStore } from "../state/clip-store";
import { planFor, planRequestOf, type PlanInput } from "./request-plan";

const m = vi.hoisted(() => ({
  flags: { cloud_transcribe: true, long_video: false, high_res: false, ai_plan: true, motion: false },
  send: vi.fn(),
}));

vi.mock("../config/features", () => ({ FEATURES: m.flags }));
vi.mock("../net/api-client", () => ({ PLAN_CLIENT_TIMEOUT_MS: 20_000, requestPlan: m.send }));

const TRANSCRIPT = {
  words: [
    { text: "We", start_ms: 0, end_ms: 200, confidence: 0.9 },
    { text: "grew", start_ms: 200, end_ms: 500, confidence: 0.9 },
    { text: "fast.", start_ms: 500, end_ms: 900, confidence: 0.4 },
    { text: "Really.", start_ms: 1_000, end_ms: 1_400, confidence: 0.8 },
  ],
  sentences: [
    { words: { start: 0, end: 3 }, start_ms: 0, end_ms: 900 },
    { words: { start: 3, end: 4 }, start_ms: 1_000, end_ms: 1_400 },
  ],
  numbers: [],
  model_version: "groq:whisper-large-v3-turbo",
} as unknown as Transcript;

const PROSODY: Prosody = {
  per_word: [
    { energy_z: 0.5, pitch_z: -0.25 },
    { energy_z: 1.5, pitch_z: 0 },
    { energy_z: -1, pitch_z: 2 },
    { energy_z: 0, pitch_z: 0.75 },
  ],
};

// The clip's length arrives typed, as it does from `ClipInfo` in the pipeline.
const DURATION = ({ duration: 1_500 } as unknown as ClipInfo).duration;
const PLAN = { format: 1, catalog: CATALOG_VERSION, source: { kind: "model", provider: "deepseek", model: "deepseek-flash" }, items: [] } as unknown as EditPlan;

function input(change: Partial<PlanInput> = {}): PlanInput {
  return { transcript: TRANSCRIPT, prosody: PROSODY, duration: DURATION, switchOn: true, signal: new AbortController().signal, ...change };
}

function answers(data: unknown): void {
  m.send.mockResolvedValue({ ok: true, status: 200, data });
}

function fails(code: string, apiCode?: string): void {
  m.send.mockResolvedValue({
    ok: false,
    failure: { code, stage: "api", retryable: true },
    ...(apiCode === undefined ? {} : { apiError: { code: apiCode, retry_after_secs: null } }),
  });
}

beforeEach(() => {
  m.flags.ai_plan = true;
  m.send.mockReset();
  answers({ plan: PLAN, plans_remaining: 39 });
  useAuthStore.setState({ status: "signed_in", email: "a@example.invalid", accessToken: "token" });
  vi.stubGlobal("navigator", { onLine: true });
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe("planFor: no request (D-466, D-494)", () => {
  it("with ai_plan off, the rules plan at once, whatever the switch and the session", async () => {
    m.flags.ai_plan = false;
    expect(await planFor(input())).toEqual({ source: "rules", why: "off" });
    expect(m.send).not.toHaveBeenCalled();
  });

  it("with the settings switch off, the rules plan at once", async () => {
    expect(await planFor(input({ switchOn: false }))).toEqual({ source: "rules", why: "off" });
    expect(m.send).not.toHaveBeenCalled();
  });

  it.each(["unknown", "anonymous", "anonymous_offline", "link_sent"] as const)(
    "signed out (%s): the rules plan at once, and nothing is sent",
    async (status) => {
      useAuthStore.setState({ status, email: null, accessToken: null });
      expect(await planFor(input())).toEqual({ source: "rules", why: "signed_out" });
      expect(m.send).not.toHaveBeenCalled();
    },
  );

  it("offline: the rules plan at once", async () => {
    vi.stubGlobal("navigator", { onLine: false });
    expect(await planFor(input())).toEqual({ source: "rules", why: "offline" });
    expect(m.send).not.toHaveBeenCalled();
  });
});

describe("planFor: the request and its outcomes (D-466)", () => {
  it("signed in: one request, and the model's plan with the plans left", async () => {
    expect(await planFor(input())).toEqual({ source: "model", plan: PLAN, plansRemaining: 39 });
    expect(m.send).toHaveBeenCalledTimes(1);
    expect(m.send.mock.calls[0]?.[0]).toEqual(planRequestOf(TRANSCRIPT, PROSODY, DURATION));
  });

  it("a Creator's answer has no count of plans left", async () => {
    answers({ plan: PLAN, plans_remaining: null });
    expect(await planFor(input())).toEqual({ source: "model", plan: PLAN, plansRemaining: null });
  });

  it.each([
    ["429 plan_quota", "E_API_RATE_LIMITED", "plan_quota", "quota"],
    ["409 plan_catalog", "E_INTERNAL", "plan_catalog", "catalog"],
    ["502 plan_invalid", "E_NET_TIMEOUT", "plan_invalid", "invalid"],
    ["502 plan_unavailable", "E_NET_TIMEOUT", "plan_unavailable", "unavailable"],
    ["504 plan_unavailable", "E_NET_TIMEOUT", "plan_unavailable", "unavailable"],
    ["a 401 after the one refresh", "E_AUTH_SESSION_EXPIRED", "unauthorized", "signed_out"],
    ["a refresh that found no session", "E_AUTH_SESSION_EXPIRED", "session_expired", "signed_out"],
    ["no network", "E_NET_OFFLINE", undefined, "offline"],
    ["a proxy's 502 with no body", "E_NET_TIMEOUT", undefined, "unavailable"],
    ["429 rate_limited", "E_API_RATE_LIMITED", "rate_limited", "unavailable"],
    ["400 bad_request", "E_INTERNAL", "bad_request", "unavailable"],
    ["a 500", "E_API_5XX", "internal", "unavailable"],
  ])("%s: the rules plan, as %s", async (_, code, apiCode, why) => {
    fails(code, apiCode);
    expect(await planFor(input())).toEqual({ source: "rules", why });
    expect(m.send).toHaveBeenCalledTimes(1);
  });

  it("a 200 without a plan is invalid", async () => {
    answers({ plans_remaining: 3 });
    expect(await planFor(input())).toEqual({ source: "rules", why: "invalid" });
    answers(undefined);
    expect(await planFor(input())).toEqual({ source: "rules", why: "invalid" });
  });

  it("never throws: a client that throws is unavailable (INV-24)", async () => {
    m.send.mockRejectedValue(new Error("boom"));
    expect(await planFor(input())).toEqual({ source: "rules", why: "unavailable" });
    m.send.mockImplementation(() => {
      throw new Error("sync boom");
    });
    expect(await planFor(input())).toEqual({ source: "rules", why: "unavailable" });
  });

  it("a cancel by the caller reaches the request's signal", async () => {
    const caller = new AbortController();
    let seen: AbortSignal | undefined;
    m.send.mockImplementation((_body: unknown, signal: AbortSignal) => {
      seen = signal;
      return new Promise((resolve) => {
        signal.addEventListener("abort", () => {
          resolve({ ok: false, failure: { code: "E_INTERNAL", stage: "api", retryable: false } });
        });
      });
    });
    const pending = planFor(input({ signal: caller.signal }));
    caller.abort();
    expect(await pending).toEqual({ source: "rules", why: "unavailable" });
    expect(seen?.aborted).toBe(true);
  });
});

describe("planFor: the timer (D-498)", () => {
  it("no answer in 20 s: the rules plan as timeout, and the request is aborted", async () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    let seen: AbortSignal | undefined;
    m.send.mockImplementation((_body: unknown, signal: AbortSignal) => {
      seen = signal;
      return new Promise(() => undefined);
    });
    const pending = planFor(input());
    let settled = false;
    void pending.then(() => {
      settled = true;
    });
    await vi.advanceTimersByTimeAsync(19_999);
    expect(settled).toBe(false);
    expect(seen?.aborted).toBe(false);
    await vi.advanceTimersByTimeAsync(1);
    expect(await pending).toEqual({ source: "rules", why: "timeout" });
    expect(seen?.aborted).toBe(true);
  });

  it("an answer just inside 20 s is used, and the timer is cleared", async () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    m.send.mockImplementation(
      () =>
        new Promise((resolve) => {
          setTimeout(() => {
            resolve({ ok: true, status: 200, data: { plan: PLAN, plans_remaining: 1 } });
          }, 19_999);
        }),
    );
    const pending = planFor(input());
    await vi.advanceTimersByTimeAsync(19_999);
    expect(await pending).toEqual({ source: "model", plan: PLAN, plansRemaining: 1 });
    expect(vi.getTimerCount()).toBe(0);
  });
});

describe("planRequestOf: the body (TS §40.7, P-1)", () => {
  it("is the heard words, their timings and prosody, and each sentence's words, and nothing else", () => {
    const body = planRequestOf(TRANSCRIPT, PROSODY, DURATION);
    expect(body).toEqual({
      catalog: CATALOG_VERSION,
      duration: 1_500,
      words: [
        { text: "We", start: 0, end: 200, energy_z: 0.5, pitch_z: -0.25 },
        { text: "grew", start: 200, end: 500, energy_z: 1.5, pitch_z: 0 },
        { text: "fast.", start: 500, end: 900, energy_z: -1, pitch_z: 2 },
        { text: "Really.", start: 1_000, end: 1_400, energy_z: 0, pitch_z: 0.75 },
      ],
      sentences: [
        { start: 0, end: 3 },
        { start: 3, end: 4 },
      ],
    });
    // No confidence, no model name, no numbers: only the four fields the server reads.
    expect(Object.keys(body).sort()).toEqual(["catalog", "duration", "sentences", "words"]);
  });

  it("carries no word the person typed: their edits in the clip store never reach the request", async () => {
    useClipStore.setState({
      edit: { ...useClipStore.getState().edit, word_edits: { 1: "SHRANK", 2: "" } as EditState["word_edits"] },
    });
    try {
      await planFor(input());
      const sent = JSON.stringify(m.send.mock.calls[0]?.[0]);
      expect(sent).not.toContain("SHRANK");
      expect(sent).toContain('"text":"grew"');
      expect(sent).toContain('"text":"fast."');
    } finally {
      useClipStore.setState(useClipStore.getInitialState(), true);
    }
  });

  it("a word without a prosody entry reads as the speaker's average", () => {
    const short: Prosody = { per_word: PROSODY.per_word.slice(0, 2) };
    const words = planRequestOf(TRANSCRIPT, short, DURATION).words;
    expect(words[2]).toEqual({ text: "fast.", start: 500, end: 900, energy_z: 0, pitch_z: 0 });
  });
});
