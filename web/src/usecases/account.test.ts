// The account page's five use-cases over the real `api-client` and `http`, a fake
// `fetch` and a fake clock (TS C-11, C-18; v6implementation §13, §14, D-300).

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { connectAuth } from "../net/api-client";
import { accountDeleted, useAuthStore } from "../state/auth-store";
import { setToken, useEntitlementStore } from "../state/entitlement-store";
import { confirmCheckout, POLL_EVERY_MS, POLL_FOR_MS, waitForCreator } from "./confirm-checkout";
import { deleteAccount } from "./delete-account";
import { ACCOUNT_FILE_NAME, exportAccountData } from "./export-account-data";
import { openBillingPortal } from "./open-billing-portal";
import { startCheckout } from "./start-checkout";

const m = vi.hoisted(() => ({
  events: [] as { name: string; props?: Record<string, unknown> }[],
  stored: undefined as string | undefined,
  failStorage: false,
  /** How many navigations had happened at each flush. */
  flushedAt: [] as number[],
}));

vi.mock("../analytics/client", () => ({
  track: (event: { name: string; props?: Record<string, unknown> }) => m.events.push(event),
  flush: () => {
    m.flushedAt.push(assigned.length);
    return Promise.resolve();
  },
}));
vi.mock("../persistence/entitlement-repo", () => ({
  put: (token: string) => {
    if (m.failStorage) {
      return Promise.reject(new Error("no storage"));
    }
    m.stored = token;
    return Promise.resolve();
  },
  clear: () => {
    if (m.failStorage) {
      return Promise.reject(new Error("no storage"));
    }
    m.stored = undefined;
    return Promise.resolve();
  },
}));

const tokenOf = (plan: "free" | "creator") =>
  `${btoa(JSON.stringify({ v: 1, sub: "u", plan, free_exports_remaining: 3, period_end: 3, iat: 1, exp: 2 }))}.signature`;
const me = (plan: "free" | "creator") => ({ status: 200, body: { email: "someone@example.com", plan, subscription: null, entitlement_token: tokenOf(plan) } });
const failure = (status: number, code: string) => ({ status, body: { code, retry_after_secs: null } });
const CHECKOUT_URL = "https://checkout.example.invalid/session";
const PORTAL_URL = "https://portal.example.invalid/session";

type Answer = { status: number; body?: unknown } | { offline: true };
type Call = { at: number; method: string; path: string; body: unknown };

let calls: Call[];
let answers: Map<string, Answer[]>;
let assigned: string[];
let clicked: { href: string; download: string }[];

/** What `METHOD /path` answers: its list in order, the last repeating. Unlisted: `/healthz` 200, anything else a failed test. */
function answerFor(key: string): Answer {
  const list = answers.get(key);
  if (list === undefined) {
    if (key === "GET /healthz") {
      return { status: 200, body: {} };
    }
    throw new Error(`nothing answers ${key}`);
  }
  const next = list.length > 1 ? list.shift() : list[0];
  if (next === undefined) {
    throw new Error(`nothing answers ${key}`);
  }
  return next;
}

function fakeFetch(input: RequestInfo | URL, init: RequestInit = {}): Promise<Response> {
  const path = (input instanceof Request ? input.url : input.toString()).replace("/api/v1", "");
  const method = init.method ?? "GET";
  calls.push({ at: Date.now(), method, path, body: typeof init.body === "string" ? JSON.parse(init.body) : undefined });
  const answer = answerFor(`${method} ${path}`);
  if ("offline" in answer) {
    return Promise.reject(new TypeError("Failed to fetch"));
  }
  return Promise.resolve(new Response(answer.body === undefined ? null : JSON.stringify(answer.body), { status: answer.status }));
}

const callsTo = (path: string) => calls.filter((call) => call.path === path);
const named = (name: string) => m.events.filter((event) => event.name === name);

async function settled<T>(pending: Promise<T>): Promise<T> {
  await vi.runAllTimersAsync();
  return pending;
}

function signedIn(): void {
  useAuthStore.setState({ status: "signed_in", email: null, accessToken: "access" });
}

beforeEach(() => {
  calls = [];
  answers = new Map();
  assigned = [];
  clicked = [];
  m.events.length = 0;
  m.stored = undefined;
  m.failStorage = false;
  m.flushedAt.length = 0;
  vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout", "Date"] });
  vi.setSystemTime(0);
  vi.stubGlobal("fetch", fakeFetch);
  vi.stubGlobal("navigator", { onLine: true });
  vi.stubGlobal("window", {
    location: {
      assign: (url: string) => {
        assigned.push(url);
      },
    },
  });
  useAuthStore.setState(useAuthStore.getInitialState(), true);
  useEntitlementStore.setState(useEntitlementStore.getInitialState(), true);
  connectAuth({
    accessToken: () => useAuthStore.getState().accessToken,
    refreshed: () => undefined,
    sessionExpired: () => undefined,
  });
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe("startCheckout (C-11)", () => {
  it("signed in: once the API is up, one checkout for the offer, then the merchant's page", async () => {
    signedIn();
    answers.set("POST /billing/checkout", [{ status: 200, body: { url: CHECKOUT_URL } }]);
    expect(await settled(startCheckout("creator_annual"))).toEqual({ ok: true });
    expect(calls.map((call) => `${call.method} ${call.path}`)).toEqual(["GET /healthz", "POST /billing/checkout"]);
    expect(callsTo("/billing/checkout")[0]?.body).toEqual({ offer: "creator_annual" });
    expect(assigned).toEqual([CHECKOUT_URL]);
    expect(m.events).toEqual([{ name: "checkout_step", props: { step: "started" } }]);
    // Sent before the page leaves, not left to the send as it unloads.
    expect(m.flushedAt).toEqual([0]);
  });

  it("from the prompt: the click is told with its placement, before the checkout starts", async () => {
    signedIn();
    answers.set("POST /billing/checkout", [{ status: 200, body: { url: CHECKOUT_URL } }]);
    await settled(startCheckout("creator_monthly", "limit_reached"));
    expect(m.events).toEqual([
      { name: "upgrade_prompt", props: { placement: "limit_reached", action: "clicked" } },
      { name: "checkout_step", props: { step: "started" } },
    ]);
  });

  it("a token with no session: nothing is asked, and nobody is sent anywhere", async () => {
    useAuthStore.setState({ status: "anonymous_offline" });
    setToken(tokenOf("free"));
    expect(await settled(startCheckout("creator_monthly", "after_export"))).toEqual({ ok: false, code: "E_AUTH_SESSION_EXPIRED" });
    expect(calls).toEqual([]);
    expect(assigned).toEqual([]);
    expect(named("checkout_step")).toEqual([]);
    expect(named("upgrade_prompt")).toHaveLength(1);
  });

  it("a subscription already live: refused, so the portal instead (D-300)", async () => {
    signedIn();
    answers.set("POST /billing/checkout", [failure(403, "forbidden")]);
    answers.set("GET /billing/portal", [{ status: 200, body: { url: PORTAL_URL } }]);
    expect(await settled(startCheckout("creator_monthly"))).toEqual({ ok: true });
    expect(assigned).toEqual([PORTAL_URL]);
  });

  it("the merchant down: one attempt, its own failure, no navigation (D-287)", async () => {
    signedIn();
    answers.set("POST /billing/checkout", [failure(502, "billing_unavailable")]);
    expect(await settled(startCheckout("creator_monthly"))).toEqual({ ok: false, code: "E_BILLING_UNAVAILABLE" });
    expect(callsTo("/billing/checkout")).toHaveLength(1);
    expect(assigned).toEqual([]);
  });
});

describe("confirmCheckout and waitForCreator (C-11)", () => {
  it("a cancel: told once, nothing asked", async () => {
    signedIn();
    expect(await settled(confirmCheckout("cancel"))).toEqual({ kind: "cancelled" });
    expect(calls).toEqual([]);
    expect(m.events).toEqual([{ name: "checkout_step", props: { step: "returned_cancel" } }]);
  });

  it("a success: asked every 2 s until the plan is Creator, whose token the page and the device then hold", async () => {
    signedIn();
    answers.set("GET /me", [me("free"), me("free"), me("creator")]);
    const outcome = await settled(confirmCheckout("success"));
    expect(outcome).toMatchObject({ kind: "active", me: { plan: "creator" } });
    expect(callsTo("/me").map((call) => call.at)).toEqual([0, POLL_EVERY_MS, 2 * POLL_EVERY_MS]);
    expect(m.stored).toBe(tokenOf("creator"));
    expect(useEntitlementStore.getState().claims?.plan).toBe("creator");
    expect(useAuthStore.getState().email).toBe("someone@example.com");
    expect(m.events).toEqual([{ name: "checkout_step", props: { step: "returned_success" } }]);
  });

  it("no Creator within the minute: pending, after 31 reads from 0 s to 60 s", async () => {
    signedIn();
    answers.set("GET /me", [me("free")]);
    expect(await settled(confirmCheckout("success"))).toEqual({ kind: "pending" });
    const reads = callsTo("/me").map((call) => call.at);
    expect(reads).toHaveLength(POLL_FOR_MS / POLL_EVERY_MS + 1);
    expect(reads.at(-1)).toBe(POLL_FOR_MS);
  });

  it("asking again starts another minute, and tells nothing more", async () => {
    signedIn();
    answers.set("GET /me", [me("free"), me("creator")]);
    expect(await settled(waitForCreator())).toMatchObject({ kind: "active" });
    expect(m.events).toEqual([]);
  });

  it("a read that fails is asked again: here the rate limit, which a page asking every 2 s can meet", async () => {
    signedIn();
    answers.set("GET /me", [failure(429, "rate_limited"), me("creator")]);
    expect(await settled(waitForCreator())).toMatchObject({ kind: "active" });
  });

  it("a session that has ended stops the wait", async () => {
    signedIn();
    answers.set("GET /me", [failure(401, "unauthorized")]);
    answers.set("POST /auth/refresh", [failure(401, "session_expired")]);
    expect(await settled(waitForCreator())).toEqual({ kind: "failed", code: "E_AUTH_SESSION_EXPIRED" });
    expect(callsTo("/me")).toHaveLength(1);
  });

  it("a device that cannot store the token still shows the plan, and says so once", async () => {
    signedIn();
    m.failStorage = true;
    answers.set("GET /me", [me("creator")]);
    expect(await settled(waitForCreator())).toMatchObject({ kind: "active" });
    expect(useEntitlementStore.getState().claims?.plan).toBe("creator");
    expect(m.events).toEqual([{ name: "client_error", props: { error_code: "E_STORAGE_IO", stage: "api" } }]);
  });
});

describe("openBillingPortal (C-11)", () => {
  it("goes to the portal", async () => {
    signedIn();
    answers.set("GET /billing/portal", [{ status: 200, body: { url: PORTAL_URL } }]);
    expect(await settled(openBillingPortal())).toEqual({ ok: true });
    expect(assigned).toEqual([PORTAL_URL]);
  });

  it("the merchant down: said, and nobody leaves", async () => {
    signedIn();
    answers.set("GET /billing/portal", [failure(502, "billing_unavailable")]);
    expect(await settled(openBillingPortal())).toEqual({ ok: false, code: "E_BILLING_UNAVAILABLE" });
    expect(assigned).toEqual([]);
  });
});

describe("deleteAccount (C-18)", () => {
  it("deleted: signed out, the token gone from the page and the device", async () => {
    signedIn();
    setToken(tokenOf("free"));
    m.stored = tokenOf("free");
    answers.set("POST /account/delete", [{ status: 204 }]);
    expect(await settled(deleteAccount())).toEqual({ ok: true });
    expect(calls.map((call) => `${call.method} ${call.path}`)).toEqual(["GET /healthz", "POST /account/delete"]);
    expect(useAuthStore.getState()).toMatchObject({ status: "anonymous", accessToken: null, email: null });
    expect(useEntitlementStore.getState().token).toBeNull();
    expect(m.stored).toBeUndefined();
  });

  it("the cancel at the merchant failed: one attempt, and nothing changes here either (D-271)", async () => {
    signedIn();
    setToken(tokenOf("creator"));
    m.stored = tokenOf("creator");
    answers.set("POST /account/delete", [failure(502, "billing_unavailable")]);
    expect(await settled(deleteAccount())).toEqual({ ok: false, code: "E_BILLING_UNAVAILABLE" });
    expect(callsTo("/account/delete")).toHaveLength(1);
    expect(useAuthStore.getState().status).toBe("signed_in");
    expect(useEntitlementStore.getState().token).toBe(tokenOf("creator"));
    expect(m.stored).toBe(tokenOf("creator"));
  });

  it("a device that cannot forget the token: deleted all the same, and said once", async () => {
    signedIn();
    m.failStorage = true;
    answers.set("POST /account/delete", [{ status: 204 }]);
    expect(await settled(deleteAccount())).toEqual({ ok: true });
    expect(useAuthStore.getState().status).toBe("anonymous");
    expect(m.events).toEqual([{ name: "client_error", props: { error_code: "E_STORAGE_IO", stage: "api" } }]);
  });

  it("the state machine has the row: signed in, then anonymous", () => {
    signedIn();
    accountDeleted();
    expect(useAuthStore.getState().status).toBe("anonymous");
  });
});

describe("exportAccountData (C-18)", () => {
  const data = { user: { email: "someone@example.com", created_at: 1 }, sessions: [], subscription: null, usage_receipts: [] };

  beforeEach(() => {
    const body = { append: () => undefined };
    vi.stubGlobal("document", {
      body,
      createElement: () => {
        const anchor = {
          href: "",
          download: "",
          click: () => {
            clicked.push({ href: anchor.href, download: anchor.download });
          },
          remove: () => undefined,
        };
        return anchor;
      },
    });
  });

  it("one JSON file of what the server holds, under a name with no person in it", async () => {
    signedIn();
    const made: Blob[] = [];
    vi.spyOn(URL, "createObjectURL").mockImplementation((blob) => {
      made.push(blob as Blob);
      return "blob:account";
    });
    const revoked = vi.spyOn(URL, "revokeObjectURL").mockImplementation(() => undefined);
    answers.set("GET /account/export", [{ status: 200, body: data }]);
    expect(await settled(exportAccountData())).toEqual({ ok: true });
    expect(clicked).toEqual([{ href: "blob:account", download: ACCOUNT_FILE_NAME }]);
    expect(JSON.parse(await (made[0] ?? new Blob()).text())).toEqual(data);
    expect(made[0]?.type).toBe("application/json");
    expect(revoked).toHaveBeenCalledWith("blob:account");
  });

  it("offline: said, and no file", async () => {
    signedIn();
    vi.stubGlobal("navigator", { onLine: false });
    answers.set("GET /account/export", [{ offline: true }]);
    expect(await settled(exportAccountData())).toMatchObject({ ok: false, code: "E_NET_OFFLINE" });
    expect(clicked).toEqual([]);
  });
});
