// Sign-in's three use-cases over a real `api-client` and `http` and a fake `fetch`
// (TS C-8, v6implementation §13, D-291).

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useAuthStore } from "../state/auth-store";
import { setToken, useEntitlementStore } from "../state/entitlement-store";
import { completeSignIn } from "./complete-sign-in";
import { requestMagicLink, stopWaitingForLink } from "./request-magic-link";
import { signOut } from "./sign-out";

const m = vi.hoisted(() => ({
  events: [] as unknown[],
  stored: undefined as string | undefined,
  failStorage: false,
  broadcasts: 0,
  flushes: 0,
}));

vi.mock("../analytics/client", () => ({ track: (event: unknown) => m.events.push(event) }));
vi.mock("../platform/broadcast", () => ({
  postSignedIn: () => {
    m.broadcasts += 1;
  },
}));
vi.mock("../persistence/receipt-outbox", () => ({
  flushOutbox: () => {
    m.flushes += 1;
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

const TOKEN = `${btoa(JSON.stringify({ v: 1, sub: "u", plan: "free", free_exports_remaining: 3, period_end: 3, iat: 1, exp: 2 }))}.signature`;
const SESSION = { access_token: "access", access_expires_at: 1, entitlement_token: TOKEN };

type Answer = { status: number; body?: unknown };
let answers: Map<string, Answer>;
let paths: string[];
let bodies: unknown[];

function fakeFetch(input: RequestInfo | URL, init: RequestInit = {}): Promise<Response> {
  const path = (input instanceof Request ? input.url : input.toString()).replace("/api/v1", "");
  paths.push(path);
  if (typeof init.body === "string") {
    bodies.push(JSON.parse(init.body));
  }
  const answer = answers.get(path) ?? { status: 200, body: {} };
  return Promise.resolve(new Response(answer.body === undefined ? null : JSON.stringify(answer.body), { status: answer.status }));
}

const status = () => useAuthStore.getState().status;
const stepsSent = () => m.events.filter((event) => (event as { name: string }).name === "signin_step");
const noWaking = () => undefined;

beforeEach(() => {
  answers = new Map();
  paths = [];
  bodies = [];
  m.events.length = 0;
  m.stored = undefined;
  m.failStorage = false;
  m.broadcasts = 0;
  m.flushes = 0;
  vi.stubGlobal("fetch", fakeFetch);
  vi.stubGlobal("navigator", { onLine: true });
  useAuthStore.setState(useAuthStore.getInitialState(), true);
  useEntitlementStore.setState(useEntitlementStore.getInitialState(), true);
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("requestMagicLink", () => {
  it("what cannot be an address asks nothing of the server", async () => {
    await expect(requestMagicLink("   no-at-sign ", noWaking)).resolves.toEqual({ ok: false, reason: "invalid_email" });
    expect(paths).toEqual([]);
    expect(status()).toBe("unknown");
  });

  it("an accepted request: trimmed, link_sent with the address, and one step that names no address", async () => {
    answers.set("/auth/magic-link", { status: 202 });
    await expect(requestMagicLink(" someone@example.com ", noWaking)).resolves.toEqual({ ok: true });
    expect(bodies).toEqual([{ email: "someone@example.com" }]);
    expect(useAuthStore.getState()).toMatchObject({ status: "link_sent", email: "someone@example.com" });
    expect(stepsSent()).toEqual([{ name: "signin_step", props: { step: "link_requested" } }]);
    expect(JSON.stringify(m.events)).not.toContain("example.com");
  });

  it("the server's refusal of the address reads as an invalid one, and nothing changes", async () => {
    answers.set("/auth/magic-link", { status: 400, body: { code: "bad_request", retry_after_secs: null } });
    await expect(requestMagicLink("a@b", noWaking)).resolves.toEqual({ ok: false, reason: "invalid_email" });
    expect(status()).toBe("unknown");
    expect(stepsSent()).toEqual([]);
  });

  it("each failure keeps its code, and a rate limit its wait; nothing changes", async () => {
    answers.set("/auth/magic-link", { status: 503, body: { code: "email_unavailable", retry_after_secs: null } });
    await expect(requestMagicLink("a@b.c", noWaking)).resolves.toEqual({ ok: false, reason: "failed", code: "E_AUTH_EMAIL_UNAVAILABLE" });
    answers.set("/auth/magic-link", { status: 429, body: { code: "rate_limited", retry_after_secs: 30 } });
    await expect(requestMagicLink("a@b.c", noWaking)).resolves.toEqual({ ok: false, reason: "failed", code: "E_API_RATE_LIMITED", retryAfterSecs: 30 });
    expect(status()).toBe("unknown");
  });

  it("accepted after another tab signed this one in: no throw, and still signed in", async () => {
    useAuthStore.setState({ status: "signed_in" });
    answers.set("/auth/magic-link", { status: 202 });
    await expect(requestMagicLink("a@b.c", noWaking)).resolves.toEqual({ ok: true });
    expect(status()).toBe("signed_in");
  });
});

describe("stopWaitingForLink", () => {
  it("ends the wait, and leaves every other state alone", () => {
    useAuthStore.setState({ status: "link_sent", email: "a@b.c" });
    stopWaitingForLink();
    expect(useAuthStore.getState()).toMatchObject({ status: "anonymous", email: null });
    for (const other of ["unknown", "anonymous", "anonymous_offline", "signed_in"] as const) {
      useAuthStore.setState({ status: other });
      stopWaitingForLink();
      expect(status()).toBe(other);
    }
  });
});

describe("completeSignIn", () => {
  it("a good link: signed in from unknown, the token held and stored, the others told, the outbox flushed", async () => {
    answers.set("/auth/verify", { status: 200, body: SESSION });
    await expect(completeSignIn("link-token", noWaking)).resolves.toEqual({ ok: true });
    expect(bodies).toEqual([{ token: "link-token" }]);
    expect(useAuthStore.getState()).toMatchObject({ status: "signed_in", accessToken: "access" });
    expect(useEntitlementStore.getState().token).toBe(TOKEN);
    expect(m.stored).toBe(TOKEN);
    expect(m.broadcasts).toBe(1);
    expect(m.flushes).toBe(1);
    expect(stepsSent()).toEqual([{ name: "signin_step", props: { step: "completed" } }]);
  });

  it("no refresh goes with it (D-296)", async () => {
    answers.set("/auth/verify", { status: 200, body: SESSION });
    await completeSignIn("link-token", noWaking);
    expect(paths).toEqual(["/healthz", "/auth/verify"]);
  });

  it.each([
    ["link_expired", "E_AUTH_LINK_EXPIRED"],
    ["link_invalid", "E_AUTH_LINK_INVALID"],
  ])("%s: its code; nobody signed in, nobody told", async (code, expected) => {
    answers.set("/auth/verify", { status: 400, body: { code, retry_after_secs: null } });
    await expect(completeSignIn("link-token", noWaking)).resolves.toEqual({ ok: false, code: expected });
    expect(status()).toBe("unknown");
    expect(m.broadcasts).toBe(0);
    expect(stepsSent()).toEqual([]);
  });

  it("storage that cannot be written does not undo a good sign-in", async () => {
    m.failStorage = true;
    answers.set("/auth/verify", { status: 200, body: SESSION });
    await expect(completeSignIn("link-token", noWaking)).resolves.toEqual({ ok: true });
    expect(status()).toBe("signed_in");
    expect(useEntitlementStore.getState().token).toBe(TOKEN);
    expect(m.events).toContainEqual({ name: "client_error", props: { error_code: "E_STORAGE_IO", stage: "api" } });
  });
});

describe("signOut (D-291)", () => {
  beforeEach(() => {
    useAuthStore.setState({ status: "signed_in", email: "a@b.c", accessToken: "access" });
    setToken(TOKEN);
    m.stored = TOKEN;
  });

  it("ends the session and clears the token from the page and the device", async () => {
    answers.set("/auth/logout", { status: 204 });
    await expect(signOut()).resolves.toEqual({ ok: true });
    expect(useAuthStore.getState()).toEqual({ status: "anonymous", email: null, accessToken: null });
    expect(useEntitlementStore.getState().token).toBeNull();
    expect(m.stored).toBeUndefined();
  });

  it("a logout that fails changes nothing: the cookie is still there", async () => {
    answers.set("/auth/logout", { status: 400, body: { code: "forbidden", retry_after_secs: null } });
    await expect(signOut()).resolves.toEqual({ ok: false, code: "E_INTERNAL" });
    expect(status()).toBe("signed_in");
    expect(useEntitlementStore.getState().token).toBe(TOKEN);
    expect(m.stored).toBe(TOKEN);
  });
});
