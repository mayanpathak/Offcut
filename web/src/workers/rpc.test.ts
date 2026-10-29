// The RPC layer (v4implementation §14.2, §14.3, D-151 to D-153): both sides
// of `rpc.ts` and `rpc-worker.ts` against fakes, and the pool's cancel,
// restart and crash handling. A worker's global scope and the `Worker`
// class are scripted here; timers are fake wherever time matters.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { JobId } from "../gen/domain";
import { createClient, WorkerCallError } from "./rpc";
import { CANCELLED, type JobContext, PROGRESS_MIN_INTERVAL_MS, serveWorker, WorkerFailure } from "./rpc-worker";

vi.mock("../wasm/load-core", () => ({ preloadCore: () => Promise.resolve() }));
vi.mock("../wasm/load-render", () => ({ preloadRender: () => Promise.resolve() }));

type Posted = {
  type: string;
  jobId?: number;
  done?: number;
  feed?: unknown;
  cancelled?: boolean;
  ok?: boolean;
  result?: unknown;
  failure?: { code: string; stage: string; detail?: string };
};

/** A worker's global scope: what the worker posts, and a way to send it a message. */
function fakeScope() {
  const posted: Posted[] = [];
  let listener: ((e: { data: unknown }) => void) | undefined;
  const scope = {
    postMessage: (m: Posted) => posted.push(m),
    addEventListener: (_type: string, l: (e: { data: unknown }) => void) => {
      listener = l;
    },
  };
  vi.stubGlobal("self", scope);
  return { posted, send: (data: unknown) => listener?.({ data }) };
}

/** A job the test ends by hand, with its context. */
function heldJob() {
  let ctx: JobContext | undefined;
  let finish: (value: unknown) => void = () => undefined;
  const handler = (_p: never, c: JobContext) => {
    ctx = c;
    return new Promise((resolve) => {
      finish = resolve;
    });
  };
  return { handler, context: () => ctx, finish: (value: unknown) => finish(value) };
}

/** A `Worker` the test answers by hand. */
class FakeWorker extends EventTarget {
  static made: FakeWorker[] = [];
  posted: { type: string; jobId: JobId; method?: string; params?: unknown }[] = [];
  terminated = false;
  /** Answers a cancel as a worker that sees its flag would; off, the job never answers. */
  answersCancel = true;

  constructor() {
    super();
    FakeWorker.made.push(this);
  }

  postMessage(m: { type: string; jobId: JobId; method?: string; params?: unknown }): void {
    this.posted.push(m);
    if (m.type === "cancel" && this.answersCancel) {
      queueMicrotask(() => {
        this.answer({ type: "res", jobId: m.jobId, cancelled: true });
      });
    }
  }

  answer(data: unknown): void {
    this.dispatchEvent(new MessageEvent("message", { data }));
  }

  terminate(): void {
    this.terminated = true;
  }
}

type TestApi = { a(p: { n: number }): Promise<number>; b(): Promise<number> };

const codeOf = (error: unknown): string => (error instanceof WorkerCallError ? error.failure.code : "not a WorkerCallError");

// What a worker's result may hold besides PCM: absent from Node, so stood in for.
beforeEach(() => {
  vi.stubGlobal("OffscreenCanvas", class {});
  vi.stubGlobal("ImageBitmap", class {});
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe("request and response", () => {
  it("a call is answered by its own job, and a progress reaches its caller only", async () => {
    const w = new FakeWorker();
    const c = createClient<TestApi>(w as unknown as Worker, "asr");
    const seen: number[] = [];
    const first = c.call.a({ n: 1 }, { onProgress: (p) => seen.push(p.done) });
    const second = c.call.a({ n: 2 });
    const [one, two] = w.posted;
    expect(one).toMatchObject({ type: "req", method: "a", params: { n: 1 } });
    expect(two?.jobId).not.toBe(one?.jobId);

    w.answer({ type: "progress", jobId: one?.jobId, stage: "asr", done: 5, total: 10 });
    w.answer({ type: "res", jobId: two?.jobId, ok: true, result: 22 });
    w.answer({ type: "res", jobId: one?.jobId, ok: true, result: 11 });
    expect(await first).toBe(11);
    expect(await second).toBe(22);
    expect(seen).toEqual([5]);
    expect(c.pendingJobs()).toEqual([]);
  });

  it("a failure rejects with the worker's failure", async () => {
    const w = new FakeWorker();
    const c = createClient<TestApi>(w as unknown as Worker, "asr");
    const call = c.call.b();
    const failure = { code: "E_ASR_RUNTIME", stage: "asr", retryable: true, detail: "Boom" };
    w.answer({ type: "res", jobId: w.posted[0]?.jobId, ok: false, failure });
    await expect(call).rejects.toMatchObject({ failure });
  });

  it("the worker's side answers a result, a named failure and an unknown method", async () => {
    const { posted, send } = fakeScope();
    serveWorker<{ ok(): Promise<unknown>; bad(): Promise<unknown> }>(
      {
        ok: () => Promise.resolve({ value: 3 }),
        bad: () => Promise.reject(new WorkerFailure("E_DSP", "Clipped", "audio_chain")),
      },
      { stage: "asr", oneWay: [], duringPreview: [] },
    );
    send({ type: "req", jobId: 1, method: "ok", params: undefined });
    await vi.waitFor(() => expect(posted).toHaveLength(1));
    send({ type: "req", jobId: 2, method: "bad", params: undefined });
    await vi.waitFor(() => expect(posted).toHaveLength(2));
    send({ type: "req", jobId: 3, method: "missing", params: undefined });
    expect(posted[0]).toEqual({ type: "res", jobId: 1, ok: true, result: { value: 3 } });
    expect(posted[1]).toMatchObject({ jobId: 2, ok: false, failure: { code: "E_DSP", stage: "audio_chain", detail: "Clipped" } });
    expect(posted[2]).toMatchObject({ jobId: 3, ok: false, failure: { code: "E_INTERNAL", detail: "NoHandler" } });
  });

  it("the worker's side refuses a second job while one runs", async () => {
    const { posted, send } = fakeScope();
    const job = heldJob();
    serveWorker<{ work(): Promise<unknown> }>({ work: job.handler }, { stage: "preview", oneWay: [], duringPreview: [] });
    send({ type: "req", jobId: 1, method: "work", params: undefined });
    send({ type: "req", jobId: 2, method: "work", params: undefined });
    expect(posted).toHaveLength(1);
    expect(posted[0]).toMatchObject({ jobId: 2, ok: false, failure: { detail: "Busy" } });
    // The handler starts a moment after the message: the first job is ended once it runs.
    await vi.waitFor(() => expect(job.context()).toBeDefined());
    job.finish(1);
    await vi.waitFor(() => expect(posted).toHaveLength(2));
  });
});

describe("the progress throttle (D-151)", () => {
  beforeEach(() => {
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout", "performance"] });
  });

  it("30 progresses in 1 s arrive as at most 10, every feed line arrives, the held one before res", async () => {
    const { posted, send } = fakeScope();
    const job = heldJob();
    serveWorker<{ work(): Promise<unknown> }>({ work: job.handler }, { stage: "asr", oneWay: [], duringPreview: [] });
    send({ type: "req", jobId: 1, method: "work", params: undefined });
    await vi.advanceTimersByTimeAsync(0);
    const ctx = job.context();
    // One every 33 ms for a second; three of them carry a feed line.
    for (let i = 0; i < 30; i += 1) {
      const feed = i === 4 || i === 14 || i === 24 ? { feed: { kind: "transcribing" } as const } : {};
      ctx?.progress({ stage: "asr", done: i, total: 30, ...feed });
      vi.advanceTimersByTime(33);
    }
    job.finish({ fine: true });
    await vi.advanceTimersByTimeAsync(0);

    const progress = posted.filter((m) => m.type === "progress");
    const lines = progress.filter((m) => m.feed !== undefined);
    const plain = progress.filter((m) => m.feed === undefined);
    expect(lines.map((m) => m.done)).toEqual([4, 14, 24]);
    expect(plain.length).toBeLessThanOrEqual(10);
    // The last progress reaches the page, before the answer, and nothing comes after it.
    expect(progress.at(-1)?.done).toBe(29);
    expect(posted.at(-1)?.type).toBe("res");
    vi.advanceTimersByTime(1000);
    expect(posted.at(-1)?.type).toBe("res");
  });

  it(`a progress inside ${String(PROGRESS_MIN_INTERVAL_MS)} ms is held and replaced; a feed line is never held`, async () => {
    const { posted, send } = fakeScope();
    const job = heldJob();
    serveWorker<{ work(): Promise<unknown> }>({ work: job.handler }, { stage: "asr", oneWay: [], duringPreview: [] });
    send({ type: "req", jobId: 1, method: "work", params: undefined });
    await vi.advanceTimersByTimeAsync(0);
    const ctx = job.context();
    const p = (done: number, feed = false) =>
      ctx?.progress({ stage: "asr", done, total: 100, ...(feed ? { feed: { kind: "transcribing" } as const } : {}) });
    p(1); // sent
    vi.advanceTimersByTime(10);
    p(2); // held
    vi.advanceTimersByTime(10);
    p(3); // replaces 2
    vi.advanceTimersByTime(85); // the timer at 100 ms sends 3
    p(4); // held
    p(5, true); // sent at once; 4 is dropped, 5 is newer
    p(6); // held
    job.finish(1);
    await vi.advanceTimersByTimeAsync(0);
    const seq = posted.map((m) => (m.type === "progress" ? `p${String(m.done)}${m.feed === undefined ? "" : "f"}` : "res"));
    expect(seq).toEqual(["p1", "p3", "p5f", "p6", "res"]);
  });
});

describe("cancel (D-152)", () => {
  it("the worker's side: the flag is seen, the job answers cancelled", async () => {
    const { posted, send } = fakeScope();
    serveWorker<{ work(): Promise<unknown> }>(
      {
        work: async (_p, ctx) => {
          while (!ctx.isCancelled()) {
            await new Promise((resolve) => setTimeout(resolve, 1));
          }
          return CANCELLED;
        },
      },
      { stage: "import", oneWay: [], duringPreview: [] },
    );
    send({ type: "req", jobId: 4, method: "work", params: undefined });
    send({ type: "cancel", jobId: 4 });
    await vi.waitFor(() => expect(posted).toEqual([{ type: "res", jobId: 4, cancelled: true }]));
  });

  it("the client's side: a cancel answered resolves the call as cancelled, and ended() says so", async () => {
    const w = new FakeWorker();
    const c = createClient<TestApi>(w as unknown as Worker, "preview");
    const call = c.call.b();
    const [job] = c.pendingJobs();
    if (job === undefined) {
      throw new Error("no pending job");
    }
    const ended = c.ended(job);
    c.cancel(job);
    expect(await call).toEqual({ cancelled: true });
    expect(await ended).toBe(true);
    expect(await c.ended(99 as JobId)).toBe(false);
  });
});

describe("crash (D-153)", () => {
  it("rejects every pending call with E_WORKER_CRASH and the row's stage, tells onCrash once, refuses what follows", async () => {
    const w = new FakeWorker();
    const c = createClient<TestApi>(w as unknown as Worker, "preview");
    const crashes: string[] = [];
    c.onCrash((f) => crashes.push(`${f.code}/${f.stage}`));
    const a = c.call.a({ n: 1 }).catch((e: unknown) => (e instanceof WorkerCallError ? `${e.failure.code}/${e.failure.stage}` : "?"));
    const b = c.call.b().catch(codeOf);
    w.dispatchEvent(new Event("error"));
    w.dispatchEvent(new Event("messageerror"));
    expect(await a).toBe("E_WORKER_CRASH/preview");
    expect(await b).toBe("E_WORKER_CRASH");
    expect(crashes).toEqual(["E_WORKER_CRASH/preview"]);
    expect(c.isDead()).toBe(true);
    expect(c.pendingJobs()).toEqual([]);
    expect(await c.call.b().catch(codeOf)).toBe("E_WORKER_CRASH");
    // A late answer of a job the crash ended is dropped.
    w.answer({ type: "res", jobId: 1, ok: true, result: 5 });
  });
});

describe("the transfer check (TS §14.3)", () => {
  it("a buffer listed for transfer that is still here after postMessage fails the call", async () => {
    const w = new FakeWorker();
    const c = createClient<{ take(p: { pcm: Float32Array }): Promise<void> }>(w as unknown as Worker, "audio_chain");
    const pcm = new Float32Array(16);
    // The fake worker copies nothing away: the buffer stays attached, as a copy would leave it.
    const failure = await c.call.take({ pcm }, { transfer: [pcm.buffer] }).catch((e: unknown) => (e instanceof WorkerCallError ? e.failure : null));
    expect(failure).toMatchObject({ code: "E_INTERNAL", stage: "audio_chain" });
    expect(c.pendingJobs()).toEqual([]);
  });

  it("a buffer that was moved passes", async () => {
    const w = new FakeWorker();
    const c = createClient<{ take(p: { pcm: Float32Array }): Promise<number> }>(w as unknown as Worker, "audio_chain");
    const pcm = new Float32Array(16);
    // Moved away before the call, as `postMessage` moves it: what the list holds is detached.
    structuredClone(pcm.buffer, { transfer: [pcm.buffer] });
    const call = c.call.take({ pcm }, { transfer: [pcm.buffer] });
    w.answer({ type: "res", jobId: w.posted[0]?.jobId, ok: true, result: 1 });
    expect(await call).toBe(1);
  });
});

describe("the pool (D-152, D-153)", () => {
  beforeEach(() => {
    FakeWorker.made = [];
    vi.stubGlobal("Worker", FakeWorker);
  });

  it("cancel: idle, cancelled, restarted after the timeout; a fresh worker after a restart and after a crash", async () => {
    const { pool, CANCEL_TIMEOUT_MS } = await import("./pool");
    expect(await pool.cancel("audio")).toBe("idle");

    // The job answers the cancel.
    const first = pool.audio.runChain({ pcm48: new Float32Array(0) });
    const w1 = FakeWorker.made[0];
    expect(await pool.cancel("audio")).toBe("cancelled");
    expect(await first).toEqual({ cancelled: true });
    expect(await pool.cancel("audio")).toBe("idle");

    // It does not: "restarted" at the timeout, not before; the call rejected, the worker ended.
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    if (w1 !== undefined) {
      w1.answersCancel = false;
    }
    const second = pool.audio.runChain({ pcm48: new Float32Array(0) }).catch(codeOf);
    let answer: string | undefined;
    const cancelling = pool.cancel("audio").then((a) => {
      answer = a;
      return a;
    });
    await vi.advanceTimersByTimeAsync(CANCEL_TIMEOUT_MS - 1);
    expect(answer).toBeUndefined();
    await vi.advanceTimersByTimeAsync(1);
    expect(await cancelling).toBe("restarted");
    expect(await second).toBe("E_WORKER_CRASH");
    expect(w1?.terminated).toBe(true);
    vi.useRealTimers();

    // The next call makes a fresh worker.
    void pool.audio.runChain({ pcm48: new Float32Array(0) }).catch(codeOf);
    expect(FakeWorker.made).toHaveLength(2);

    // A crash: told, dead; the next call makes a third, and the second is ended first.
    const told: string[] = [];
    pool.audio.onCrash((f) => told.push(f.code));
    FakeWorker.made[1]?.dispatchEvent(new Event("error"));
    expect(pool.audio.isDead()).toBe(true);
    expect(told).toEqual(["E_WORKER_CRASH"]);
    void pool.audio.runChain({ pcm48: new Float32Array(0) }).catch(codeOf);
    expect(FakeWorker.made).toHaveLength(3);
    expect(FakeWorker.made[1]?.terminated).toBe(true);
    expect(pool.audio.isDead()).toBe(false);

    // restartWorker with nothing pending, then a call: a fourth.
    pool.restartWorker("audio");
    expect(FakeWorker.made[2]?.terminated).toBe(true);
    void pool.audio.runChain({ pcm48: new Float32Array(0) }).catch(codeOf);
    expect(FakeWorker.made).toHaveLength(4);
  });

  it("whenFree asks again while the worker is busy; whenReleased while a file is locked", async () => {
    const { whenFree, whenReleased } = await import("./pool");
    vi.useFakeTimers({ toFake: ["setTimeout", "performance"] });
    const busy = new WorkerCallError({ code: "E_INTERNAL", stage: "preview", retryable: false, detail: "Busy" });
    const locked = new WorkerCallError({ code: "E_STORAGE_IO", stage: "storage", retryable: true, detail: "NoModificationAllowedError" });
    let tries = 0;
    const freed = whenFree(() => (++tries < 3 ? Promise.reject(busy) : Promise.resolve("drawn")));
    await vi.advanceTimersByTimeAsync(100);
    expect(await freed).toBe("drawn");
    expect(tries).toBe(3);

    tries = 0;
    const released = whenReleased(() => (++tries < 4 ? Promise.reject(locked) : Promise.resolve("opened")));
    await vi.advanceTimersByTimeAsync(1000);
    expect(await released).toBe("opened");

    // Another failure is the call's at once.
    const other = new WorkerCallError({ code: "E_GPU_LOST", stage: "preview", retryable: true });
    await expect(whenFree(() => Promise.reject(other))).rejects.toBe(other);
  });
});
