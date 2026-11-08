// A long source's cloud windows (v6.1implementation §8.1, §11.3): the count
// and bounds, order and two in flight, a resume sends only what is missing,
// every failure of D-410 to its retry or its pause, the merge, and NoSpeech.

import { describe, expect, it, vi } from "vitest";

import type { CloudWord } from "../gen/api";
import type { Transcript } from "../gen/domain";
import type { SourceId, SourceWindowRecord } from "../persistence/schema";

vi.mock("../workers/pool", () => ({ pool: {} }));
vi.mock("../net/api-client", () => ({ transcribeAudio: vi.fn(), checkHealth: vi.fn() }));
vi.mock("../persistence/opfs", () => ({ readPcm16Window: vi.fn() }));
vi.mock("../persistence/sources-repo", () => ({}));

const { transcribeSourceWindows, windowCount, windowOffsetMs, RETRY_WAITS_MS } = await import("./transcribe-source");
type Deps = Parameters<typeof transcribeSourceWindows>[1] & object;
type SendResult = Awaited<ReturnType<Deps["send"]>>;

const ID = "src-1" as SourceId;
const WORDS: CloudWord[] = [{ text: "hello", start_ms: 100, end_ms: 400 }];

type Answer =
  | "ok"
  | { status: 429 | 503 | 401 }
  | { code: "E_NET_TIMEOUT" | "E_NET_OFFLINE" | "E_API_5XX" };

function failed(answer: Exclude<Answer, "ok">): SendResult {
  if ("status" in answer) {
    const api = answer.status === 429 ? "rate_limited" : answer.status === 503 ? "transcribe_unavailable" : "unauthorized";
    const code = answer.status === 401 ? "E_AUTH_SESSION_EXPIRED" : answer.status === 429 ? "E_API_RATE_LIMITED" : "E_API_5XX";
    return { ok: false, failure: { code, stage: "api", retryable: false }, apiError: { code: api } } as unknown as SendResult;
  }
  return { ok: false, failure: { code: answer.code, stage: "api", retryable: true } } as unknown as SendResult;
}

/** Fakes for every dependency. `answers(k, attempt)` decides each request. */
function fakes(o: {
  answers?: (k: number, attempt: number) => Answer;
  kept?: number[];
  cloudOn?: boolean;
  master?: "ok" | "NoSpeech";
  encoder?: "ok" | "unsupported";
}) {
  const store = new Map<number, SourceWindowRecord>();
  for (const k of o.kept ?? []) {
    store.set(k, { words: [], lengthMs: 90_000, modelVersion: "m" });
  }
  const attempts = new Map<number, number>();
  const sent: number[] = [];
  const waits: number[] = [];
  let inFlight = 0;
  let maxInFlight = 0;
  const built: { windows: { offsetMs: number }[] }[] = [];
  const deps: Deps = {
    readWindow: () => Promise.resolve(new Float32Array(16)),
    encode: () => Promise.resolve(o.encoder === "unsupported" ? { unsupported: true } : { ogg: new Uint8Array([1, 2, 3]) }),
    send: async (_ogg, signal, window) => {
      sent.push(window.index);
      const attempt = attempts.get(window.index) ?? 0;
      attempts.set(window.index, attempt + 1);
      inFlight += 1;
      maxInFlight = Math.max(maxInFlight, inFlight);
      await new Promise((resolve) => setTimeout(resolve, 1));
      inFlight -= 1;
      const answer = o.answers?.(window.index, attempt) ?? "ok";
      if (answer !== "ok" || signal.aborted) {
        return signal.aborted ? failed({ code: "E_NET_TIMEOUT" }) : failed(answer as Exclude<Answer, "ok">);
      }
      return { ok: true as const, status: 200, data: { words: WORDS, duration_ms: 90_000, model: "groq:m" } };
    },
    windowWords: (p) => Promise.resolve({ words: p.words.map((w) => ({ text: w.text, startMs: w.start_ms, endMs: w.end_ms, confidence: 1 })), lengthMs: 90_000 }),
    buildMaster: (p) => {
      built.push(p);
      return Promise.resolve(o.master === "NoSpeech" ? { rejected: "NoSpeech" } : { ok: { words: [] } as unknown as Transcript });
    },
    windows: () => Promise.resolve(new Map(store)),
    putWindow: (_id, k, record) => {
      store.set(k, record);
      return Promise.resolve();
    },
    putMaster: () => Promise.resolve(),
    cloudOn: () => Promise.resolve(o.cloudOn),
    wait: (ms) => {
      waits.push(ms);
      return Promise.resolve();
    },
  };
  return { deps, store, sent, waits, built, maxInFlight: () => maxInFlight };
}

function run(durationMs: number, deps: Deps, onWindow: (d: number, t: number) => void = () => undefined) {
  return transcribeSourceWindows({ sourceId: ID, durationMs, levelP95: -20, signal: new AbortController().signal, onWindow }, deps);
}

describe("transcribe-source", () => {
  it("counts windows as D-405 says, with the 10 s overlap", () => {
    expect(windowCount(0)).toBe(1);
    expect(windowCount(90_000)).toBe(1);
    expect(windowCount(90_001)).toBe(2);
    expect(windowCount(720_000)).toBe(9);
    expect(windowCount(2_700_000)).toBe(34);
    expect(windowOffsetMs(3)).toBe(240_000);
  });

  it("sends the windows in order, two at a time, keeps each and builds the master from all", async () => {
    const f = fakes({});
    const seen: number[] = [];
    const result = await run(720_000, f.deps, (done, total) => {
      seen.push(done);
      expect(total).toBe(9);
    });
    expect("done" in result).toBe(true);
    expect(f.sent).toEqual([0, 1, 2, 3, 4, 5, 6, 7, 8]);
    expect(f.maxInFlight()).toBe(2);
    expect(seen).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9]);
    expect(f.built[0]?.windows.map((w) => w.offsetMs)).toEqual([0, 80_000, 160_000, 240_000, 320_000, 400_000, 480_000, 560_000, 640_000]);
  });

  it("resumes with only the windows not kept", async () => {
    const f = fakes({ kept: [0, 1, 2, 3] });
    await run(720_000, f.deps);
    expect(f.sent).toEqual([4, 5, 6, 7, 8]);
  });

  it("pauses at once on 429 as busy", async () => {
    const f = fakes({ answers: (k) => (k === 2 ? { status: 429 } : "ok") });
    expect(await run(720_000, f.deps)).toEqual({ paused: "busy" });
    expect(f.sent.filter((k) => k === 2)).toHaveLength(1);
    expect(f.store.has(2)).toBe(false);
  });

  it("retries a timeout after 1 s and 4 s and keeps the window that then answers", async () => {
    const f = fakes({ answers: (k, attempt) => (k === 0 && attempt < 2 ? { code: "E_NET_TIMEOUT" } : "ok") });
    const result = await run(90_000, f.deps);
    expect("done" in result).toBe(true);
    expect(f.waits).toEqual([...RETRY_WAITS_MS]);
  });

  it("pauses as offline after three offline attempts, and as busy after three failures", async () => {
    const offline = fakes({ answers: () => ({ code: "E_NET_OFFLINE" }) });
    expect(await run(90_000, offline.deps)).toEqual({ paused: "offline" });
    expect(offline.sent).toEqual([0, 0, 0]);
    const broken = fakes({ answers: () => ({ code: "E_API_5XX" }) });
    expect(await run(90_000, broken.deps)).toEqual({ paused: "busy" });
  });

  it("asks /healthz on 503: off is unavailable, on is retried and then busy", async () => {
    const off = fakes({ answers: () => ({ status: 503 }), cloudOn: false });
    expect(await run(90_000, off.deps)).toEqual({ paused: "unavailable" });
    expect(off.sent).toEqual([0]);
    const on = fakes({ answers: () => ({ status: 503 }), cloudOn: true });
    expect(await run(90_000, on.deps)).toEqual({ paused: "busy" });
    expect(on.sent).toEqual([0, 0, 0]);
  });

  it("pauses as signed out on a lost session", async () => {
    const f = fakes({ answers: () => ({ status: 401 }) });
    expect(await run(90_000, f.deps)).toEqual({ paused: "signed_out" });
  });

  it("a pause stops the other lane; what was kept before stays", async () => {
    const f = fakes({ answers: (k) => (k === 3 ? { status: 429 } : "ok") });
    expect(await run(720_000, f.deps)).toEqual({ paused: "busy" });
    expect([...f.store.keys()].every((k) => k < 5)).toBe(true);
    expect(f.sent.every((k) => k <= 4)).toBe(true);
  });

  it("gives NoSpeech when the merged words are too few, and fails on an encoder it cannot use", async () => {
    expect(await run(90_000, fakes({ master: "NoSpeech" }).deps)).toEqual({ rejected: "NoSpeech" });
    expect(await run(90_000, fakes({ encoder: "unsupported" }).deps)).toEqual({ failed: "encoder" });
  });
});
