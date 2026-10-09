// A fake API for the E2E tests. It answers every request under /api/v1 in the
// browser, before it reaches the network, so no test touches a real server.

import { createPrivateKey, createPublicKey, type KeyObject, sign } from "node:crypto";

import type { Page } from "@playwright/test";

import { DB_NAME, DB_VERSION, STORES } from "../../src/persistence/schema";

export type RecordedRequest = { method: string; path: string; body: unknown };

export type FakeApi = {
  /** Every request to /api/v1, in the order it arrived. `path` is relative to /api/v1. */
  requests: RecordedRequest[];
  /** Every analytics event of that name, from all recorded `POST /events` bodies. */
  eventsOf(name: string): unknown[];
};

export type FakeApiOptions = {
  /** How `POST /notify-me` answers. The default is 204 at once. */
  notifyMe?: { status: number; delayMs?: number };
  healthz?: { delayMs?: number };
};

const API_PREFIX = "/api/v1";
const JSON_TYPE = "application/json";

const wait = (ms: number) =>
  new Promise<void>((resolve) => {
    setTimeout(resolve, ms);
  });

function parse(text: string | null): unknown {
  if (text === null || text === "") {
    return undefined;
  }
  try {
    return JSON.parse(text) as unknown;
  } catch {
    return text;
  }
}

/** The events of one recorded body, if it has the shape of a batch. */
function eventsIn(body: unknown): unknown[] {
  if (typeof body === "object" && body !== null && "events" in body && Array.isArray(body.events)) {
    return body.events as unknown[];
  }
  return [];
}

/**
 * Installs the fake. `GET /healthz` answers 200, `POST /events` 204, and
 * `POST /notify-me` the configured status; an error status carries the body
 * the real server sends. Anything else is 404 `not_found`, as on the server.
 */
export async function installFakeApi(page: Page, options: FakeApiOptions = {}): Promise<FakeApi> {
  const requests: RecordedRequest[] = [];
  const notifyMe = options.notifyMe ?? { status: 204 };

  await page.route(`**${API_PREFIX}/**`, async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname.slice(API_PREFIX.length);
    requests.push({ method: request.method(), path, body: parse(request.postData()) });

    if (path === "/healthz") {
      await wait(options.healthz?.delayMs ?? 0);
      await route.fulfill({ status: 200, contentType: JSON_TYPE, body: '{"ok":true,"version":"fake"}' });
    } else if (path === "/events") {
      await route.fulfill({ status: 204 });
    } else if (path === "/notify-me") {
      await wait(notifyMe.delayMs ?? 0);
      if (notifyMe.status === 429) {
        const body = JSON.stringify({ code: "rate_limited", retry_after_secs: 705 });
        await route.fulfill({ status: 429, contentType: JSON_TYPE, headers: { "retry-after": "705" }, body });
      } else if (notifyMe.status >= 400) {
        const code = notifyMe.status >= 500 ? "internal" : "bad_request";
        const body = JSON.stringify({ code, retry_after_secs: null });
        await route.fulfill({ status: notifyMe.status, contentType: JSON_TYPE, body });
      } else {
        await route.fulfill({ status: notifyMe.status });
      }
    } else {
      const body = JSON.stringify({ code: "not_found", retry_after_secs: null });
      await route.fulfill({ status: 404, contentType: JSON_TYPE, body });
    }
  });

  return {
    requests,
    eventsOf: (name) =>
      requests
        .filter((request) => request.path === "/events")
        .flatMap((request) => eventsIn(request.body))
        .filter((event) => typeof event === "object" && event !== null && "name" in event && event.name === name),
  };
}

// --- Entitlement tokens ---------------------------------------------------------
//
// A token is `base64url(JSON claims) "." base64url(signature)`, both without
// padding, and the Ed25519 signature covers the ASCII bytes of the first
// segment (D-25). `crates/offcut-entitlement` reads the same form: one token
// minted here is a constant of its tests.

/**
 * The seed of the test signing key: 32 fixed bytes, made for the tests. It is
 * never a real key, and it is in no other file. A build accepts its tokens
 * only when it was made with VITE_ENTITLEMENT_TEST_PUBLIC_KEY, which no
 * deployment is (D-24).
 */
export const TEST_ENTITLEMENT_SEED: Uint8Array = new Uint8Array(
  Buffer.from("7f347b4b1b2701814d8288251a08d71f0ffde04599bdf4b6ed1e5b0a0eed148d", "hex"),
);

/** Every minted token is for this user, so that its claims are known in full. */
const TEST_USER_ID = "0190f3a2-7b1c-7def-8a55-0123456789ab";
const DAY_SECS = 24 * 3600;
const ED25519_KEY_BYTES = 32;
/** The PKCS #8 form of an Ed25519 private key is these 16 bytes, then the seed (RFC 8410). */
const PKCS8_ED25519_PREFIX = Buffer.from("302e020100300506032b657004220420", "hex");

function privateKey(seed: Uint8Array): KeyObject {
  if (seed.length !== ED25519_KEY_BYTES) {
    throw new Error(`an Ed25519 seed is ${ED25519_KEY_BYTES} bytes, not ${seed.length}`);
  }
  return createPrivateKey({ key: Buffer.concat([PKCS8_ED25519_PREFIX, seed]), format: "der", type: "pkcs8" });
}

/** The public key of the test seed, in standard base64: what VITE_ENTITLEMENT_TEST_PUBLIC_KEY must be for the build under test. */
export function testPublicKeyBase64(): string {
  // The SPKI form of an Ed25519 public key ends with the 32 bytes of the key.
  const spki = createPublicKey(privateKey(TEST_ENTITLEMENT_SEED)).export({ format: "der", type: "spki" });
  return spki.subarray(spki.length - ED25519_KEY_BYTES).toString("base64");
}

/**
 * A signed entitlement token. Without options: a Creator token issued now,
 * good for 7 days, in a period that ends in 30 days. Times are in seconds
 * since the Unix epoch. `seed` signs with another key than the test key.
 */
export function mintEntitlementToken(o?: {
  plan?: "free" | "creator";
  iat?: number;
  exp?: number;
  periodEnd?: number;
  freeExportsRemaining?: number;
  seed?: Uint8Array;
}): string {
  const now = Math.floor(Date.now() / 1000);
  // The field names of `EntitlementClaims` in Rust (TS §10.6).
  const claims = {
    v: 1,
    sub: TEST_USER_ID,
    plan: o?.plan ?? "creator",
    free_exports_remaining: o?.freeExportsRemaining ?? 3,
    period_end: o?.periodEnd ?? now + 30 * DAY_SECS,
    iat: o?.iat ?? now,
    exp: o?.exp ?? now + 7 * DAY_SECS,
  };
  const payload = Buffer.from(JSON.stringify(claims), "utf8").toString("base64url");
  const signature = sign(null, Buffer.from(payload, "ascii"), privateKey(o?.seed ?? TEST_ENTITLEMENT_SEED));
  return `${payload}.${signature.toString("base64url")}`;
}

/**
 * Stores `token` as the app stores the one it has: in the `entitlement` store
 * of its database, under the key "current" (D-23). The page must have opened
 * the app first: the app makes the database, and this never does.
 */
export async function seedEntitlement(page: Page, token: string): Promise<void> {
  const record = { schemaVersion: DB_VERSION, value: { token, storedAt: Date.now() } };
  await page.evaluate(
    ({ name, store, record }) =>
      new Promise<void>((resolve, reject) => {
        const open = indexedDB.open(name);
        // No database yet: it must not be made here, without its stores.
        open.onupgradeneeded = () => {
          open.transaction?.abort();
        };
        open.onerror = (event) => {
          event.preventDefault();
          reject(new Error("seedEntitlement: the app has not made its database yet"));
        };
        open.onsuccess = () => {
          const db = open.result;
          const write = db.transaction(store, "readwrite");
          write.objectStore(store).put(record, "current");
          write.oncomplete = () => {
            db.close();
            resolve();
          };
          write.onabort = () => {
            db.close();
            reject(write.error ?? new Error("seedEntitlement: the write was aborted"));
          };
        };
      }),
    { name: DB_NAME, store: STORES.entitlement, record },
  );
}
