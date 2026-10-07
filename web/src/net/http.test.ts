import { describe, expect, it } from "vitest";

import {
  API_COLD_START_BUDGET_MS,
  API_TIMEOUT_MS,
  API_WAKING_NOTICE_MS,
  createHttp,
  type HttpDeps,
  type HttpRequest,
  type HttpResult,
  toAppFailure,
} from "./http";

/** What the fake `fetch` does on one attempt. The last step repeats. */
type Step =
  | { status: number; body?: unknown; text?: string; headers?: Record<string, string>; afterMs?: number }
  | { hang: true }
  | { throws: Error };

type Call = { at: number; url: string; init: RequestInit };

/**
 * `createHttp` over a fake clock. Nothing here waits in real time: `sleep`
 * and a delayed response are entries on one timeline, and `settle` moves the
 * clock from entry to entry until the request has an answer.
 */
function setup(steps: Step[], options: { onLine?: boolean } = {}) {
  let now = 0;
  const timeline: { at: number; fire: () => void }[] = [];
  const calls: Call[] = [];
  const after = (ms: number, fire: () => void) => {
    timeline.push({ at: now + ms, fire });
  };

  const respond = (step: Extract<Step, { status: number }>): Response => {
    const body = step.text ?? (step.body === undefined ? null : JSON.stringify(step.body));
    return new Response(body, step.headers ? { status: step.status, headers: step.headers } : { status: step.status });
  };

  const deps: HttpDeps = {
    now: () => now,
    onLine: () => options.onLine ?? true,
    sleep: (ms) =>
      new Promise((resolve) => {
        after(ms, resolve);
      }),
    fetch: (input, init = {}) => {
      calls.push({ at: now, url: input instanceof Request ? input.url : input.toString(), init });
      const step = steps[Math.min(calls.length, steps.length) - 1];
      if (step === undefined) {
        throw new Error("the test gave the fake fetch no step");
      }
      if ("throws" in step) {
        return Promise.reject(step.throws);
      }
      if ("hang" in step) {
        // Never answers. Rejects when aborted, as the real `fetch` does.
        return new Promise((_resolve, reject) => {
          init.signal?.addEventListener("abort", () => {
            reject(new DOMException("The operation was aborted.", "AbortError"));
          });
        });
      }
      return new Promise((resolve) => {
        after(step.afterMs ?? 0, () => {
          resolve(respond(step));
        });
      });
    },
  };

  /** One turn of the real event loop, which lets every pending promise callback run. */
  const turn = () =>
    new Promise((resolve) => {
      setTimeout(resolve, 0);
    });

  /** Fires the earliest entry of the timeline. False when the timeline is empty. */
  const step = (): boolean => {
    timeline.sort((a, b) => a.at - b.at);
    const next = timeline.shift();
    if (next === undefined) {
      return false;
    }
    now = Math.max(now, next.at);
    next.fire();
    return true;
  };

  /** Moves the clock forward until `pending` has settled, then returns its value. */
  async function settle<T>(pending: Promise<T>): Promise<T> {
    let done = false;
    const watched = pending.finally(() => {
      done = true;
    });
    for (;;) {
      await turn();
      if (done) {
        return watched;
      }
      if (!step()) {
        throw new Error("the request is waiting, and nothing is left on the timeline");
      }
    }
  }

  /** Fires everything still on the timeline: timers the request left behind. */
  async function drain(): Promise<void> {
    while (step()) {
      await turn();
    }
  }

  const http = createHttp(deps);
  const request = <T = unknown>(req: Partial<HttpRequest> = {}): Promise<HttpResult<T>> =>
    settle(http.request<T>({ method: "GET", path: "/healthz", retry: "none", ...req }));

  return { request, drain, calls, elapsed: () => now };
}

function failed<T>(result: HttpResult<T>): Extract<HttpResult<T>, { ok: false }> {
  if (result.ok) {
    throw new Error("expected a failure, got a success");
  }
  return result;
}

const SECONDS = 1_000;
const at = (calls: Call[]) => calls.map((call) => call.at / SECONDS);

describe("a response", () => {
  it("200 with JSON gives the parsed data", async () => {
    const { request } = setup([{ status: 200, body: { ok: true, version: "abc" } }]);
    const result = await request<{ ok: boolean; version: string }>();
    expect(result).toEqual({ ok: true, status: 200, data: { ok: true, version: "abc" } });
  });

  it("204 gives undefined", async () => {
    const { request } = setup([{ status: 204 }]);
    const result = await request({ method: "POST", path: "/notify-me", body: {} });
    expect(result).toEqual({ ok: true, status: 204, data: undefined });
  });

  it("a success whose body is not JSON is E_INTERNAL, not a thrown error", async () => {
    const { request } = setup([{ status: 200, text: "<html>" }]);
    const { failure } = failed(await request());
    expect(failure).toMatchObject({ code: "E_INTERNAL", stage: "api", retryable: false });
  });
});

describe("offline", () => {
  it("does not call fetch when the browser says it is offline", async () => {
    const { request, calls } = setup([{ status: 204 }], { onLine: false });
    const { failure } = failed(await request({ retry: "cold-start" }));
    expect(failure).toMatchObject({ code: "E_NET_OFFLINE", stage: "api", retryable: true });
    expect(calls).toHaveLength(0);
  });

  it("a TypeError from fetch is E_NET_OFFLINE and is not tried again", async () => {
    const { request, calls } = setup([{ throws: new TypeError("Failed to fetch") }]);
    const { failure } = failed(await request({ retry: "cold-start" }));
    expect(failure).toMatchObject({ code: "E_NET_OFFLINE", stage: "api", retryable: true });
    expect(calls).toHaveLength(1);
  });
});

describe("cold start", () => {
  it("a first attempt that times out and a second that answers is a success", async () => {
    const { request, calls, elapsed } = setup([{ hang: true }, { status: 204 }]);
    let waking = 0;
    const result = await request({ retry: "cold-start", onWaking: () => (waking += 1) });

    expect(result).toEqual({ ok: true, status: 204, data: undefined });
    // 10 s of timeout, then a wait of 1 s.
    expect(at(calls)).toEqual([0, 11]);
    expect(elapsed()).toBe(11 * SECONDS);
    expect(waking).toBe(1);
    // The attempt that timed out was aborted; the one that answered was not.
    expect(calls.map((call) => call.init.signal?.aborted)).toEqual([true, false]);
  });

  it("gives up with E_NET_TIMEOUT inside the budget when every attempt times out", async () => {
    const { request, calls, elapsed, drain } = setup([{ hang: true }]);
    let waking = 0;
    const { failure, apiError } = failed(await request({ retry: "cold-start", onWaking: () => (waking += 1) }));

    expect(failure).toMatchObject({ code: "E_NET_TIMEOUT", stage: "api", retryable: true });
    expect(apiError).toBeUndefined();
    // Each attempt takes its full 10 s; the waits between them are 1, 2, 4 and 8 s.
    expect(at(calls)).toEqual([0, 11, 23, 37, 55]);
    // A sixth attempt would start at 73 s, past the budget.
    expect(elapsed()).toBe(65 * SECONDS);
    expect(elapsed()).toBeLessThanOrEqual(API_COLD_START_BUDGET_MS);
    expect(calls.every((call) => call.init.signal?.aborted === true)).toBe(true);

    await drain();
    expect(waking).toBe(1);
  });

  it("a timeout with retry none fails after one attempt", async () => {
    const { request, calls, elapsed } = setup([{ hang: true }, { status: 204 }]);
    const { failure } = failed(await request({ retry: "none" }));
    expect(failure).toMatchObject({ code: "E_NET_TIMEOUT", stage: "api", retryable: true });
    expect(calls).toHaveLength(1);
    expect(elapsed()).toBe(API_TIMEOUT_MS);
  });

  it("503 then 200 is a success", async () => {
    const { request, calls } = setup([{ status: 503 }, { status: 200, body: { ok: true } }]);
    const result = await request({ retry: "cold-start" });
    expect(result).toEqual({ ok: true, status: 200, data: { ok: true } });
    expect(at(calls)).toEqual([0, 1]);
  });

  it("502, 503 and 504 are a timeout: with retry none, E_NET_TIMEOUT at once", async () => {
    for (const status of [502, 503, 504]) {
      const { request, calls } = setup([{ status }]);
      const { failure } = failed(await request({ retry: "none" }));
      expect(failure.code, String(status)).toBe("E_NET_TIMEOUT");
      expect(calls).toHaveLength(1);
    }
  });

  it("500 on every attempt ends in E_API_5XX, with waits of 1, 2, 4, 8, 8 ... s", async () => {
    const body = { code: "internal", retry_after_secs: null };
    const { request, calls, elapsed } = setup([{ status: 500, body }]);
    const { failure, apiError } = failed(await request({ retry: "cold-start" }));

    expect(failure).toMatchObject({ code: "E_API_5XX", stage: "api", retryable: true });
    expect(apiError).toEqual(body);
    // The server answers at once, so the gaps are the waits themselves.
    expect(at(calls)).toEqual([0, 1, 3, 7, 15, 23, 31, 39, 47, 55, 63]);
    expect(elapsed()).toBeLessThanOrEqual(API_COLD_START_BUDGET_MS);
  });

  it("500 with retry none fails after one attempt", async () => {
    const { request, calls } = setup([{ status: 500 }, { status: 204 }]);
    const { failure } = failed(await request({ retry: "none" }));
    expect(failure.code).toBe("E_API_5XX");
    expect(calls).toHaveLength(1);
  });
});

describe("an error the server chose", () => {
  it("429 is E_API_RATE_LIMITED, carries the wait and is not tried again", async () => {
    const body = { code: "rate_limited", retry_after_secs: 30 };
    const { request, calls } = setup([{ status: 429, body }, { status: 204 }]);
    const { failure, apiError } = failed(await request({ retry: "cold-start" }));

    expect(failure).toMatchObject({ code: "E_API_RATE_LIMITED", stage: "api", retryable: true });
    expect(apiError).toEqual({ code: "rate_limited", retry_after_secs: 30 });
    expect(calls).toHaveLength(1);
  });

  it("429 without a readable body takes the wait from the Retry-After header", async () => {
    const { request } = setup([{ status: 429, text: "slow down", headers: { "Retry-After": "12" } }]);
    expect(failed(await request()).apiError).toEqual({ code: "rate_limited", retry_after_secs: 12 });
  });

  it("400 carries the parsed error and is not tried again", async () => {
    const body = { code: "bad_request", retry_after_secs: null };
    const { request, calls } = setup([{ status: 400, body }, { status: 204 }]);
    const result = failed(await request({ retry: "cold-start" }));

    expect(result.ok).toBe(false);
    expect(result.apiError?.code).toBe("bad_request");
    expect(result.failure).toMatchObject({ code: "E_INTERNAL", stage: "api", retryable: false });
    expect(calls).toHaveLength(1);
  });

  it("an error body is read only if its code is one the server defines", async () => {
    // A body without `retry_after_secs` reads as null.
    const missing = setup([{ status: 404, body: { code: "not_found" } }]);
    expect(failed(await missing.request()).apiError).toEqual({ code: "not_found", retry_after_secs: null });

    for (const text of ['{"code":"made_up"}', '{"code":7}', "[]", "null", "not json", ""]) {
      const { request } = setup([{ status: 400, text }]);
      const result = failed(await request());
      expect(result.apiError, text).toBeUndefined();
      expect(result.failure.code).toBe("E_INTERNAL");
    }
  });
});

describe("the request", () => {
  it("goes to /api/v1 plus the path, with same-origin credentials and no cache", async () => {
    const { request, calls } = setup([{ status: 204 }]);
    await request({ method: "GET", path: "/healthz" });
    await request({ method: "POST", path: "/notify-me", body: { email: "a@b.c", wanted: "launch" } });
    await request({ method: "POST", path: "/events", body: { events: [] }, keepalive: true });

    expect(calls.map((call) => call.url)).toEqual(["/api/v1/healthz", "/api/v1/notify-me", "/api/v1/events"]);
    for (const { url, init } of calls) {
      // Never absolute: no scheme and no host.
      expect(url.startsWith("/api/v1/")).toBe(true);
      expect(url).not.toMatch(/^[a-z]+:|^\/\//i);
      expect(init.credentials).toBe("same-origin");
      expect(init.cache).toBe("no-store");
    }

    const [get, post, events] = calls.map((call) => call.init);
    expect(get).toMatchObject({ method: "GET" });
    // The JSON content type is sent with a body and only with a body.
    expect(get?.headers).toBeUndefined();
    expect(get?.body).toBeUndefined();
    expect(post).toMatchObject({ method: "POST", headers: { "Content-Type": "application/json" } });
    expect(post?.body).toBe('{"email":"a@b.c","wanted":"launch"}');
    expect(post?.keepalive).toBeUndefined();
    expect(events?.keepalive).toBe(true);
  });

  it("stops without another attempt when the caller aborts it", async () => {
    const caller = new AbortController();
    const { request, calls } = setup([{ hang: true }]);
    const pending = request({ retry: "cold-start", signal: caller.signal });
    caller.abort();
    const { failure } = failed(await pending);
    expect(failure).toMatchObject({ code: "E_INTERNAL", retryable: false });
    expect(calls).toHaveLength(1);
  });
});

describe("the waking notice", () => {
  it("is never given for a response that arrives within 3 s", async () => {
    const { request, drain } = setup([{ status: 204, afterMs: API_WAKING_NOTICE_MS - 1 }]);
    let waking = 0;
    const result = await request({ retry: "cold-start", onWaking: () => (waking += 1) });
    expect(result.ok).toBe(true);
    // Not even later, when the 3 s timer of the finished request fires.
    await drain();
    expect(waking).toBe(0);
  });

  it("is given once for a slow answer, 3 s after the first attempt started", async () => {
    const { request, elapsed } = setup([{ status: 204, afterMs: 8 * SECONDS }]);
    const noticedAt: number[] = [];
    const result = await request({ retry: "cold-start", onWaking: () => noticedAt.push(elapsed()) });
    expect(result.ok).toBe(true);
    expect(noticedAt).toEqual([API_WAKING_NOTICE_MS]);
  });
});

describe("toAppFailure", () => {
  it("maps each cause and status to its code", () => {
    const code = (...args: Parameters<typeof toAppFailure>) => {
      const { code, stage, retryable } = toAppFailure(...args);
      expect(stage).toBe("api");
      return [code, retryable];
    };
    expect(code("offline")).toEqual(["E_NET_OFFLINE", true]);
    expect(code("timeout")).toEqual(["E_NET_TIMEOUT", true]);
    expect(code("status", 429)).toEqual(["E_API_RATE_LIMITED", true]);
    expect(code("status", 502)).toEqual(["E_NET_TIMEOUT", true]);
    expect(code("status", 503)).toEqual(["E_NET_TIMEOUT", true]);
    expect(code("status", 504)).toEqual(["E_NET_TIMEOUT", true]);
    expect(code("status", 500)).toEqual(["E_API_5XX", true]);
    expect(code("status", 400)).toEqual(["E_INTERNAL", false]);
    expect(code("status", 404)).toEqual(["E_INTERNAL", false]);
    expect(code("status", 413)).toEqual(["E_INTERNAL", false]);
  });
});
