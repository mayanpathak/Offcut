// The routes over the real `http.ts`, a fake `fetch` and a fake clock: who is
// asked, in what order, and how often (v6implementation §11.2, D-287, D-288, D-297).

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import * as api from "./api-client";

type Answer = { status: number; body?: unknown } | { hang: true };
type Call = { at: number; method: string; path: string; headers: Record<string, string> };

const SESSION = { access_token: "new-access", access_expires_at: 1, entitlement_token: "entitlement" };
const JSON_OK = { status: 200, body: {} };
const NO_SESSION = { status: 401, body: { code: "session_expired", retry_after_secs: null } };
const UNAUTHORIZED = { status: 401, body: { code: "unauthorized", retry_after_secs: null } };

let calls: Call[];
let answers: Map<string, Answer[]>;
let accessToken: string | null;
let refreshed: string[];
let expired: number;

/** What a path answers: its list in order, the last repeating. Unlisted: `/healthz` 200, anything else a failed test. */
function answerFor(path: string): Answer {
  const list = answers.get(path);
  if (list === undefined) {
    if (path === "/healthz") {
      return JSON_OK;
    }
    throw new Error(`nothing answers ${path}`);
  }
  const next = list.length > 1 ? list.shift() : list[0];
  if (next === undefined) {
    throw new Error(`nothing answers ${path}`);
  }
  return next;
}

function fakeFetch(input: RequestInfo | URL, init: RequestInit = {}): Promise<Response> {
  const path = (input instanceof Request ? input.url : input.toString()).replace("/api/v1", "");
  calls.push({ at: Date.now(), method: init.method ?? "GET", path, headers: { ...(init.headers as Record<string, string> | undefined) } });
  const answer = answerFor(path);
  if ("hang" in answer) {
    return new Promise((_resolve, reject) => {
      init.signal?.addEventListener("abort", () => {
        reject(new DOMException("aborted", "AbortError"));
      });
    });
  }
  const text = answer.body === undefined ? null : JSON.stringify(answer.body);
  return Promise.resolve(new Response(text, { status: answer.status }));
}

function setLocks(locks: unknown): void {
  vi.stubGlobal("navigator", locks === undefined ? { onLine: true } : { onLine: true, locks });
}

const pathsOf = (path: string) => calls.filter((call) => call.path === path);

async function settled<T>(pending: Promise<T>): Promise<T> {
  await vi.runAllTimersAsync();
  return pending;
}

beforeEach(() => {
  calls = [];
  answers = new Map();
  accessToken = "old-access";
  refreshed = [];
  expired = 0;
  vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout", "Date"] });
  vi.setSystemTime(0);
  vi.stubGlobal("fetch", fakeFetch);
  setLocks(undefined);
  api.connectAuth({
    accessToken: () => accessToken,
    refreshed: (session) => {
      refreshed.push(session.access_token);
      accessToken = session.access_token;
    },
    sessionExpired: () => {
      expired += 1;
    },
  });
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe("authed (D-288)", () => {
  it("sends the access token, and a good answer ends it", async () => {
    answers.set("/me", [JSON_OK]);
    const result = await settled(api.getMe());
    expect(result.ok).toBe(true);
    expect(pathsOf("/me")[0]?.headers).toEqual({ Authorization: "Bearer old-access" });
    expect(pathsOf("/auth/refresh")).toEqual([]);
  });

  it("a 401 refreshes once and repeats once, with the new token", async () => {
    answers.set("/me", [UNAUTHORIZED, JSON_OK]);
    answers.set("/auth/refresh", [{ status: 200, body: SESSION }]);
    const result = await settled(api.getMe());

    expect(result.ok).toBe(true);
    expect(pathsOf("/me").map((call) => call.headers.Authorization)).toEqual(["Bearer old-access", "Bearer new-access"]);
    expect(pathsOf("/auth/refresh")).toHaveLength(1);
    expect(refreshed).toEqual(["new-access"]);
  });

  it("two callers meeting a 401 cause one refresh, and both go on", async () => {
    answers.set("/me", [UNAUTHORIZED, JSON_OK]);
    answers.set("/entitlement", [UNAUTHORIZED, JSON_OK]);
    answers.set("/auth/refresh", [{ status: 200, body: SESSION }]);
    const [me, entitlement] = await settled(Promise.all([api.getMe(), api.getEntitlement()]));

    expect(me.ok && entitlement.ok).toBe(true);
    expect(pathsOf("/auth/refresh")).toHaveLength(1);
    expect(refreshed).toEqual(["new-access"]);
  });

  it("a caller whose 401 arrives after the refresh ended uses the new token and does not refresh again", async () => {
    answers.set("/me", [UNAUTHORIZED, JSON_OK]);
    answers.set("/auth/refresh", [{ status: 200, body: SESSION }]);
    const pending = api.getMe();
    // Another caller has refreshed while this one's request was out.
    accessToken = "new-access";
    const result = await settled(pending);

    expect(result.ok).toBe(true);
    expect(pathsOf("/auth/refresh")).toEqual([]);
    expect(pathsOf("/me").map((call) => call.headers.Authorization)).toEqual(["Bearer old-access", "Bearer new-access"]);
  });

  it("no token at all goes to the refresh first", async () => {
    accessToken = null;
    answers.set("/me", [JSON_OK]);
    answers.set("/auth/refresh", [{ status: 200, body: SESSION }]);
    await settled(api.getMe());
    expect(pathsOf("/me").map((call) => call.headers.Authorization)).toEqual(["Bearer new-access"]);
  });

  it("a refresh answering 401 is E_AUTH_SESSION_EXPIRED: told once, not repeated", async () => {
    answers.set("/me", [UNAUTHORIZED]);
    answers.set("/entitlement", [UNAUTHORIZED]);
    answers.set("/auth/refresh", [NO_SESSION]);
    const [me, entitlement] = await settled(Promise.all([api.getMe(), api.getEntitlement()]));

    for (const result of [me, entitlement]) {
      expect(!result.ok && result.failure).toMatchObject({ code: "E_AUTH_SESSION_EXPIRED", retryable: false });
    }
    expect(pathsOf("/me")).toHaveLength(1);
    expect(pathsOf("/auth/refresh")).toHaveLength(1);
    expect(expired).toBe(1);
  });

  it("a 401 after a good refresh is E_AUTH_SESSION_EXPIRED, and there is no third attempt", async () => {
    answers.set("/me", [UNAUTHORIZED]);
    answers.set("/auth/refresh", [{ status: 200, body: SESSION }]);
    const result = await settled(api.getMe());

    expect(!result.ok && result.failure.code).toBe("E_AUTH_SESSION_EXPIRED");
    expect(pathsOf("/me")).toHaveLength(2);
    expect(pathsOf("/auth/refresh")).toHaveLength(1);
    expect(expired).toBe(1);
  });

  it("a refresh that gets no answer is not a lost session", async () => {
    answers.set("/me", [UNAUTHORIZED]);
    answers.set("/auth/refresh", [{ status: 400, body: { code: "bad_request", retry_after_secs: null } }]);
    const result = await settled(api.getMe());
    expect(!result.ok && result.failure.code).toBe("E_INTERNAL");
    expect(expired).toBe(0);
  });
});

describe("refresh (D-297)", () => {
  it("wakes the API first, then asks with the CSRF header, under the lock", async () => {
    const request = vi.fn((name: string, run: () => Promise<unknown>) => {
      calls.push({ at: Date.now(), method: "LOCK", path: name, headers: {} });
      return run();
    });
    setLocks({ request });
    answers.set("/auth/refresh", [{ status: 200, body: SESSION }]);
    const result = await settled(api.refresh());

    expect(result.ok).toBe(true);
    expect(calls.map((call) => call.path)).toEqual(["/healthz", "offcut-auth-refresh", "/auth/refresh"]);
    expect(pathsOf("/auth/refresh")[0]).toMatchObject({ method: "POST", headers: { "X-Offcut-Csrf": "1" } });
    expect(request).toHaveBeenCalledTimes(1);
  });

  it("waits for another tab's lock, and only then sends", async () => {
    let release: () => void = () => undefined;
    const held = new Promise<void>((resolve) => {
      release = resolve;
    });
    setLocks({ request: async (_name: string, run: () => Promise<unknown>) => held.then(run) });
    answers.set("/auth/refresh", [{ status: 200, body: SESSION }]);
    const pending = api.refresh();

    await vi.advanceTimersByTimeAsync(20_000);
    expect(pathsOf("/healthz")).toHaveLength(1);
    expect(pathsOf("/auth/refresh")).toEqual([]);
    release();
    expect((await settled(pending)).ok).toBe(true);
    expect(pathsOf("/auth/refresh")).toHaveLength(1);
  });

  it("runs without the lock where the browser has none", async () => {
    setLocks(undefined);
    answers.set("/auth/refresh", [{ status: 200, body: SESSION }]);
    expect((await settled(api.refresh())).ok).toBe(true);
  });

  it("stops retrying at 25 s, counted from the send and not from the wake", async () => {
    answers.set("/healthz", [{ status: 503 }, JSON_OK]);
    answers.set("/auth/refresh", [{ hang: true }]);
    const result = await settled(api.refresh());

    expect(!result.ok && result.failure.code).toBe("E_NET_TIMEOUT");
    // The wake took 1 s: a 503, a wait, a 200.
    const sentAt = pathsOf("/auth/refresh").map((call) => call.at / 1_000);
    expect(sentAt).toEqual([1, 12, 24]);
    expect(Date.now()).toBe(1_000 + api.REFRESH_BUDGET_MS);
    expect(api.REFRESH_BUDGET_MS).toBeLessThan(30_000);
  });

  it("does not send when the API cannot be woken", async () => {
    vi.stubGlobal("navigator", { onLine: false });
    const result = await settled(api.refresh());
    expect(!result.ok && result.failure.code).toBe("E_NET_OFFLINE");
    expect(calls).toEqual([]);
  });

  it("callers while one is in flight share it; a later call sends again", async () => {
    answers.set("/auth/refresh", [{ status: 200, body: SESSION }]);
    await settled(Promise.all([api.refresh(), api.refresh(), api.refresh()]));
    expect(pathsOf("/auth/refresh")).toHaveLength(1);
    await settled(api.refresh());
    expect(pathsOf("/auth/refresh")).toHaveLength(2);
  });

  it("a 401 tells that the session is gone; a failure to reach it does not", async () => {
    answers.set("/auth/refresh", [NO_SESSION]);
    const result = await settled(api.refresh());
    expect(!result.ok && result.failure).toMatchObject({ code: "E_AUTH_SESSION_EXPIRED", retryable: false });
    expect(expired).toBe(1);
    expect(refreshed).toEqual([]);
  });
});

describe("a provider that is down (D-287)", () => {
  it("a 502 billing_unavailable makes one attempt and gives E_BILLING_UNAVAILABLE", async () => {
    answers.set("/billing/checkout", [{ status: 502, body: { code: "billing_unavailable", retry_after_secs: null } }, { status: 200, body: { url: "x" } }]);
    const result = await settled(api.createCheckout({ offer: "creator_monthly" }));
    expect(!result.ok && result.failure.code).toBe("E_BILLING_UNAVAILABLE");
    expect(pathsOf("/billing/checkout")).toHaveLength(1);
  });

  it("the portal, which retries a waking API, still makes one attempt", async () => {
    answers.set("/billing/portal", [{ status: 502, body: { code: "billing_unavailable", retry_after_secs: null } }, { status: 200, body: { url: "x" } }]);
    const result = await settled(api.getPortal());
    expect(!result.ok && result.failure.code).toBe("E_BILLING_UNAVAILABLE");
    expect(pathsOf("/billing/portal")).toHaveLength(1);
  });

  it("a 503 email_unavailable makes one attempt and gives E_AUTH_EMAIL_UNAVAILABLE", async () => {
    answers.set("/auth/magic-link", [{ status: 503, body: { code: "email_unavailable", retry_after_secs: null } }]);
    const result = await settled(api.requestMagicLink({ email: "someone@example.com" }));
    expect(!result.ok && result.failure.code).toBe("E_AUTH_EMAIL_UNAVAILABLE");
    expect(pathsOf("/auth/magic-link")).toHaveLength(1);
  });
});

describe("the routes (§11.1, §11.2)", () => {
  it("verify maps its two refusals, and tells no one the session is gone", async () => {
    for (const [code, expected] of [
      ["link_invalid", "E_AUTH_LINK_INVALID"],
      ["link_expired", "E_AUTH_LINK_EXPIRED"],
    ] as const) {
      answers.set("/auth/verify", [{ status: 400, body: { code, retry_after_secs: null } }]);
      const result = await settled(api.verifyMagicLink({ token: "t" }));
      expect(!result.ok && result.failure).toMatchObject({ code: expected, retryable: false });
    }
    expect(expired).toBe(0);
  });

  it("each flow that has one attempt wakes the API first", async () => {
    answers.set("/auth/magic-link", [{ status: 202 }]);
    answers.set("/auth/verify", [{ status: 200, body: SESSION }]);
    answers.set("/auth/logout", [{ status: 204 }]);
    answers.set("/billing/checkout", [{ status: 200, body: { url: "x" } }]);
    answers.set("/account/delete", [{ status: 204 }]);
    const flows: (() => Promise<unknown>)[] = [
      () => api.requestMagicLink({ email: "a@b.c" }),
      () => api.verifyMagicLink({ token: "t" }),
      () => api.logout(),
      () => api.createCheckout({ offer: "creator_annual" }),
      () => api.deleteAccount(),
    ];
    for (const flow of flows) {
      calls = [];
      await settled(flow());
      expect(calls.map((call) => call.path)[0]).toBe("/healthz");
      expect(calls).toHaveLength(2);
    }
  });

  it("no attempt is made on an API that cannot be woken", async () => {
    answers.set("/healthz", [{ status: 500 }]);
    const result = await settled(api.requestMagicLink({ email: "a@b.c" }));
    expect(!result.ok && result.failure.code).toBe("E_API_5XX");
    expect(pathsOf("/auth/magic-link")).toEqual([]);
  });

  it("the waking notice of a flow comes from its wake", async () => {
    answers.set("/healthz", [{ hang: true }, JSON_OK]);
    answers.set("/auth/magic-link", [{ status: 202 }]);
    let noticed = 0;
    await settled(api.requestMagicLink({ email: "a@b.c" }, () => (noticed += 1)));
    expect(noticed).toBe(1);
  });

  it("only the two cookie routes send the CSRF header, and neither sends a bearer", async () => {
    answers.set("/auth/logout", [{ status: 204 }]);
    await settled(api.logout());
    expect(pathsOf("/auth/logout")[0]?.headers).toEqual({ "X-Offcut-Csrf": "1" });

    answers.set("/auth/verify", [{ status: 200, body: SESSION }]);
    await settled(api.verifyMagicLink({ token: "t" }));
    expect(pathsOf("/auth/verify")[0]?.headers).toEqual({ "Content-Type": "application/json" });
  });

  it("the routes that need a person send the bearer token, to their method and path", async () => {
    answers.set("/me", [JSON_OK]);
    answers.set("/entitlement", [JSON_OK]);
    answers.set("/usage/receipts", [JSON_OK]);
    answers.set("/billing/checkout", [JSON_OK]);
    answers.set("/billing/portal", [JSON_OK]);
    answers.set("/account/delete", [{ status: 204 }]);
    answers.set("/account/export", [JSON_OK]);
    const flows: [string, string, () => Promise<unknown>][] = [
      ["GET", "/me", () => api.getMe()],
      ["GET", "/entitlement", () => api.getEntitlement()],
      ["POST", "/usage/receipts", () => api.postUsageReceipt({ export_id: "x" as never })],
      ["POST", "/billing/checkout", () => api.createCheckout({ offer: "creator_monthly" })],
      ["GET", "/billing/portal", () => api.getPortal()],
      ["POST", "/account/delete", () => api.deleteAccount()],
      ["GET", "/account/export", () => api.exportAccount()],
    ];
    for (const [method, path, flow] of flows) {
      await settled(flow());
      expect(pathsOf(path)[0]).toMatchObject({ method, headers: { Authorization: "Bearer old-access" } });
    }
  });

  it("a receipt is one attempt: a 500 is not tried again", async () => {
    answers.set("/usage/receipts", [{ status: 500 }, JSON_OK]);
    const result = await settled(api.postUsageReceipt({ export_id: "x" as never }));
    expect(!result.ok && result.failure.code).toBe("E_API_5XX");
    expect(pathsOf("/usage/receipts")).toHaveLength(1);
  });

  it("a read keeps trying through a waking API", async () => {
    answers.set("/me", [{ status: 503 }, JSON_OK]);
    const result = await settled(api.getMe());
    expect(result.ok).toBe(true);
    expect(pathsOf("/me")).toHaveLength(2);
  });
});

describe("cloud transcription (cloudimplementation D-366, D-372, D-373)", () => {
  it("checkHealth reads cloud_transcribe, and an older server's body has none", async () => {
    answers.set("/healthz", [{ status: 200, body: { ok: true, version: "v", cloud_transcribe: true } }]);
    const yes = await settled(api.checkHealth());
    expect(yes.ok && yes.data.cloud_transcribe).toBe(true);
    answers.set("/healthz", [{ status: 200, body: { ok: true, version: "old" } }]);
    const old = await settled(api.checkHealth());
    expect(old.ok && old.data.cloud_transcribe).toBeUndefined();
  });

  it("transcribeAudio sends the bytes once, as audio/ogg, with the token, and a 401 refreshes once", async () => {
    const words = { words: [], duration_ms: 1_000, model: "groq:whisper-large-v3-turbo" };
    answers.set("/transcribe", [UNAUTHORIZED, { status: 200, body: words }]);
    answers.set("/auth/refresh", [{ status: 200, body: SESSION }]);
    const ogg = new Uint8Array([0x4f, 0x67, 0x67, 0x53]);
    const result = await settled(api.transcribeAudio(ogg, new AbortController().signal));

    expect(result.ok && result.data).toEqual(words);
    expect(pathsOf("/transcribe").map((call) => call.headers)).toEqual([
      { "Content-Type": "audio/ogg", Authorization: "Bearer old-access" },
      { "Content-Type": "audio/ogg", Authorization: "Bearer new-access" },
    ]);
    expect(pathsOf("/auth/refresh")).toHaveLength(1);
  });

  it("transcribeAudio's 503 transcribe_unavailable is one answer, read by its code, not a waking API", async () => {
    answers.set("/transcribe", [{ status: 503, body: { code: "transcribe_unavailable", retry_after_secs: null } }, JSON_OK]);
    const result = await settled(api.transcribeAudio(new Uint8Array([1]), new AbortController().signal));
    expect(!result.ok && result.apiError?.code).toBe("transcribe_unavailable");
    expect(pathsOf("/transcribe")).toHaveLength(1);
  });

  it("transcribeAudio waits 30 s for its one attempt, not 10", async () => {
    answers.set("/transcribe", [{ hang: true }]);
    const pending = api.transcribeAudio(new Uint8Array([1]), new AbortController().signal);
    await vi.advanceTimersByTimeAsync(29_000);
    expect(pathsOf("/transcribe")).toHaveLength(1);
    const result = await settled(pending);
    expect(!result.ok && result.failure.code).toBe("E_NET_TIMEOUT");
    expect(Date.now()).toBeGreaterThanOrEqual(api.TRANSCRIBE_TIMEOUT_MS);
    expect(pathsOf("/transcribe")).toHaveLength(1);
  });

  // V6.1 (D-413): a clip's request is today's, header for header; a window adds its one header.
  it("transcribeAudio sends a clip exactly as before and a window with X-Offcut-Window", async () => {
    const words = { words: [], duration_ms: 1_000, model: "groq:whisper-large-v3-turbo" };
    answers.set("/transcribe", [{ status: 200, body: words }, { status: 200, body: words }]);
    const ogg = new Uint8Array([0x4f, 0x67, 0x67, 0x53]);
    await settled(api.transcribeAudio(ogg, new AbortController().signal));
    await settled(api.transcribeAudio(ogg, new AbortController().signal, { index: 3, total: 34 }));
    expect(pathsOf("/transcribe").map((call) => call.headers)).toEqual([
      { "Content-Type": "audio/ogg", Authorization: "Bearer old-access" },
      { "Content-Type": "audio/ogg", Authorization: "Bearer old-access", "X-Offcut-Window": "3/34" },
    ]);
  });
});
