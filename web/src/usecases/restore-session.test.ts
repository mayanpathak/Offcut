// The session at start (v6implementation §11.3, D-268, D-289, D-291): the stores
// through a real `api-client` and `http`, over a fake `fetch`.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ExportId } from "../gen/domain";
import { getMe } from "../net/api-client";
import type { OutboxLinks } from "../persistence/receipt-outbox";
import { linkAccepted, useAuthStore, verified } from "../state/auth-store";
import { useEntitlementStore } from "../state/entitlement-store";
import { connectAccount, restoreSession } from "./restore-session";

const mocks = vi.hoisted(() => ({
  stored: { current: undefined as string | undefined, failRead: false, failWrite: false },
  flushes: { count: 0 },
  mirrors: { count: 0 },
  outbox: { links: undefined as OutboxLinks | undefined },
  broadcast: { handlers: [] as (() => void)[] },
  events: [] as unknown[],
}));

vi.mock("../analytics/client", () => ({ track: (event: unknown) => mocks.events.push(event) }));
vi.mock("../platform/broadcast", () => ({ onSignedIn: (handler: () => void) => mocks.broadcast.handlers.push(handler) }));
vi.mock("../persistence/entitlement-repo", () => ({
  get: () => {
    if (mocks.stored.failRead) {
      return Promise.reject(new Error("no storage"));
    }
    return Promise.resolve(mocks.stored.current === undefined ? undefined : { token: mocks.stored.current, storedAt: 0 });
  },
  put: (token: string) => {
    if (mocks.stored.failWrite) {
      return Promise.reject(new Error("no storage"));
    }
    mocks.stored.current = token;
    return Promise.resolve();
  },
}));
vi.mock("../persistence/receipt-outbox", () => ({
  connectOutbox: (links: OutboxLinks) => {
    mocks.outbox.links = links;
  },
  flushOutbox: () => {
    mocks.flushes.count += 1;
    return Promise.resolve();
  },
  mirrorOutbox: () => {
    mocks.mirrors.count += 1;
    return Promise.resolve();
  },
}));

const SUB = "0190f3a2-7b1c-7def-8a55-0123456789ab";

function tokenOf(over: Record<string, unknown> = {}): string {
  const claims = { v: 1, sub: SUB, plan: "free", free_exports_remaining: 3, period_end: 3, iat: 1, exp: 2, ...over };
  return `${btoa(JSON.stringify(claims)).replaceAll("=", "")}.signature`;
}

type Answer = { status: number; body?: unknown } | { offline: true };
const session = (over: Record<string, unknown> = {}) => ({
  status: 200,
  body: { access_token: "access", access_expires_at: 1, entitlement_token: tokenOf(over) },
});
const NO_SESSION = { status: 401, body: { code: "session_expired", retry_after_secs: null } };

let refreshAnswers: Answer[];
let otherAnswers: Map<string, Answer[]>;
let paths: string[];

function fakeFetch(input: RequestInfo | URL): Promise<Response> {
  const path = (input instanceof Request ? input.url : input.toString()).replace("/api/v1", "");
  paths.push(path);
  const list = path === "/auth/refresh" ? refreshAnswers : (otherAnswers.get(path) ?? [{ status: 200, body: {} }]);
  const answer = (list.length > 1 ? list.shift() : list[0]) ?? { status: 500 };
  if ("offline" in answer) {
    return Promise.reject(new TypeError("Failed to fetch"));
  }
  return Promise.resolve(new Response(answer.body === undefined ? null : JSON.stringify(answer.body), { status: answer.status }));
}

const status = () => useAuthStore.getState().status;
const links = (): OutboxLinks => {
  const connected = mocks.outbox.links;
  if (connected === undefined) {
    throw new Error("the outbox was not connected");
  }
  return connected;
};
const EXPORT_ID: ExportId = "0190f3a2-7b1c-7def-8a55-000000000001" as never;

let page: EventTarget;

beforeEach(() => {
  page = new EventTarget();
  vi.stubGlobal("window", page);
  vi.stubGlobal("navigator", { onLine: true });
  vi.stubGlobal("fetch", fakeFetch);
  refreshAnswers = [NO_SESSION];
  otherAnswers = new Map();
  paths = [];
  mocks.stored.current = undefined;
  mocks.stored.failRead = false;
  mocks.stored.failWrite = false;
  mocks.flushes.count = 0;
  mocks.mirrors.count = 0;
  mocks.broadcast.handlers.length = 0;
  mocks.events.length = 0;
  useAuthStore.setState(useAuthStore.getInitialState(), true);
  useEntitlementStore.setState(useEntitlementStore.getInitialState(), true);
  connectAccount();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

/** Lets a chain of answered promises finish. */
const settle = () => vi.waitFor(() => Promise.resolve(), { interval: 1 }).then(() => new Promise((resolve) => setTimeout(resolve, 5)));

describe("restoreSession", () => {
  it("200: signed in, the token held and stored, and the outbox flushed", async () => {
    refreshAnswers = [session()];
    await restoreSession();
    await settle();

    expect(useAuthStore.getState()).toMatchObject({ status: "signed_in", accessToken: "access" });
    expect(useEntitlementStore.getState().claims?.sub).toBe(SUB);
    expect(mocks.stored.current).toBe(tokenOf());
    expect(mocks.flushes.count).toBe(1);
  });

  it("401: anonymous, with no error reported and nothing flushed", async () => {
    await restoreSession();
    expect(status()).toBe("anonymous");
    expect(mocks.events).toEqual([]);
    expect(mocks.flushes.count).toBe(0);
  });

  it("D-291: a cached token is on the page before any answer, and stays after a 401", async () => {
    const cached = tokenOf({ plan: "creator" });
    mocks.stored.current = cached;
    await restoreSession();
    expect(status()).toBe("anonymous");
    expect(useEntitlementStore.getState().token).toBe(cached);
    expect(mocks.stored.current).toBe(cached);
    expect(mocks.mirrors.count).toBe(1);
  });

  it("no network: anonymous_offline, and `online` asks once more", async () => {
    refreshAnswers = [{ offline: true }, session()];
    await restoreSession();
    expect(status()).toBe("anonymous_offline");

    page.dispatchEvent(new Event("online"));
    await settle();
    expect(status()).toBe("signed_in");
    expect(paths.filter((path) => path === "/auth/refresh")).toHaveLength(2);
  });

  it("still no network at `online`: it stays offline and listens again; then a 401 is anonymous", async () => {
    refreshAnswers = [{ offline: true }, { offline: true }, NO_SESSION];
    await restoreSession();
    page.dispatchEvent(new Event("online"));
    await settle();
    expect(status()).toBe("anonymous_offline");

    page.dispatchEvent(new Event("online"));
    await settle();
    expect(status()).toBe("anonymous");
    page.dispatchEvent(new Event("online"));
    await settle();
    expect(paths.filter((path) => path === "/auth/refresh")).toHaveLength(3);
  });

  it("a server that cannot answer reads as offline and is not a lost session", async () => {
    refreshAnswers = [{ status: 400, body: { code: "bad_request", retry_after_secs: null } }];
    mocks.stored.current = tokenOf();
    await restoreSession();
    expect(status()).toBe("anonymous_offline");
    expect(useEntitlementStore.getState().token).toBe(tokenOf());
  });

  it("storage that cannot be read: reported, the refresh still runs, and it does not reject", async () => {
    mocks.stored.failRead = true;
    refreshAnswers = [session()];
    await expect(restoreSession()).resolves.toBeUndefined();
    expect(status()).toBe("signed_in");
    expect(mocks.events).toEqual([{ name: "client_error", props: { error_code: "E_STORAGE_IO", stage: "api" } }]);
  });

  it("storage that cannot be written: the page still has the token", async () => {
    mocks.stored.failWrite = true;
    refreshAnswers = [session()];
    await restoreSession();
    await settle();
    expect(useEntitlementStore.getState().token).toBe(tokenOf());
    expect(mocks.stored.current).toBeUndefined();
  });
});

describe("a broadcast (TS C-8)", () => {
  it("while the dialog waits: a good refresh signs this tab in", async () => {
    await restoreSession();
    linkAccepted("someone@example.com");
    refreshAnswers = [session()];

    mocks.broadcast.handlers[0]?.();
    await settle();
    expect(status()).toBe("signed_in");
    expect(mocks.flushes.count).toBe(1);
  });

  it("while the dialog waits: a refused refresh leaves it waiting, with no throw", async () => {
    await restoreSession();
    linkAccepted("someone@example.com");

    mocks.broadcast.handlers[0]?.();
    await settle();
    expect(status()).toBe("link_sent");
  });

  it("alone, without a good refresh, it signs nobody in", async () => {
    await restoreSession();
    expect(status()).toBe("anonymous");
    mocks.broadcast.handlers[0]?.();
    await settle();
    expect(status()).toBe("anonymous");
  });

  it("in a tab signed in: only the tokens change", async () => {
    refreshAnswers = [session()];
    await restoreSession();
    refreshAnswers = [session({ plan: "creator" })];
    mocks.broadcast.handlers[0]?.();
    await settle();
    expect(status()).toBe("signed_in");
    expect(useEntitlementStore.getState().claims?.plan).toBe("creator");
  });
});

describe("a session lost later (D-291)", () => {
  it("signed_in goes to anonymous, and the token stays for offline exports", async () => {
    refreshAnswers = [session()];
    await restoreSession();
    await settle();

    otherAnswers.set("/me", [{ status: 401, body: { code: "unauthorized", retry_after_secs: null } }]);
    refreshAnswers = [NO_SESSION];
    const result = await getMe();

    expect(!result.ok && result.failure.code).toBe("E_AUTH_SESSION_EXPIRED");
    expect(useAuthStore.getState()).toMatchObject({ status: "anonymous", accessToken: null });
    expect(useEntitlementStore.getState().token).toBe(tokenOf());
    expect(mocks.stored.current).toBe(tokenOf());
  });
});

describe("the outbox's links", () => {
  const answerReceipt = (answer: Answer) => {
    otherAnswers.set("/usage/receipts", [answer]);
    return links().send(EXPORT_ID);
  };

  it("whose rows go: the token's account, only while signed in (D-267, D-268)", () => {
    expect(links().signedInSub()).toBeNull();
    useEntitlementStore.setState({ claims: { sub: SUB } as never });
    expect(links().signedInSub()).toBeNull();
    verified("access");
    expect(links().signedInSub()).toBe(SUB);
  });

  it("a 2xx is sent, with the fresh token", async () => {
    verified("access");
    const token = tokenOf({ free_exports_remaining: 2 });
    await expect(answerReceipt({ status: 200, body: { entitlement_token: token } })).resolves.toEqual({ kind: "sent", token });
  });

  it("a 4xx it can never get past is refused", async () => {
    verified("access");
    for (const code of ["bad_request", "forbidden", "not_found", "payload_too_large"]) {
      await expect(answerReceipt({ status: 400, body: { code, retry_after_secs: null } }), code).resolves.toEqual({ kind: "refused" });
    }
  });

  it("D-307: a 429 waits, as does a 5xx, the network, and a 4xx with no readable body", async () => {
    verified("access");
    for (const answer of [
      { status: 429, body: { code: "rate_limited", retry_after_secs: 30 } },
      { status: 500, body: { code: "internal", retry_after_secs: null } },
      { offline: true as const },
      { status: 404 },
    ]) {
      await expect(answerReceipt(answer), JSON.stringify(answer)).resolves.toEqual({ kind: "later" });
    }
  });

  it("a lost session keeps the row, with no timer (D-268)", async () => {
    verified("access");
    refreshAnswers = [NO_SESSION];
    await expect(answerReceipt({ status: 401, body: { code: "unauthorized", retry_after_secs: null } })).resolves.toEqual({ kind: "no_session" });
    expect(status()).toBe("anonymous");
  });

  it("a fresh token from a receipt is held and stored", async () => {
    const token = tokenOf({ free_exports_remaining: 1 });
    links().tokenReceived(token);
    await settle();
    expect(useEntitlementStore.getState().claims?.free_exports_remaining).toBe(1);
    expect(mocks.stored.current).toBe(token);
  });

  it("the mirror reaches the store; `online` flushes", () => {
    links().rowsChanged({ [SUB]: 2 });
    expect(useEntitlementStore.getState().outboxFree).toEqual({ [SUB]: 2 });
    page.dispatchEvent(new Event("online"));
    expect(mocks.flushes.count).toBe(1);
  });
});
