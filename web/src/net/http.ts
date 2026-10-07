// Requests to the API on the app's own origin. This is the only file that
// calls `fetch` for `/api/v1`, and it never builds an absolute URL.
//
// The API runs on a free host that sleeps when idle and takes about a minute
// to wake (TS §22.9), so a request can be told to keep trying while it does.

import { env } from "../config/env";
import type { ApiError, ApiErrorCode } from "../gen/api";
import type { AppFailure } from "../workers/protocol";

export const API_TIMEOUT_MS = 10_000;
export const API_COLD_START_BUDGET_MS = 70_000;
export const API_WAKING_NOTICE_MS = 3_000;

const API_PREFIX = "/api/v1";
/** The waits before the second, third and fourth attempt. Every later wait is `MAX_RETRY_WAIT_MS`. */
const RETRY_WAITS_MS = [1_000, 2_000, 4_000];
const MAX_RETRY_WAIT_MS = 8_000;

export type HttpRequest = {
  method: "GET" | "POST";
  /** Relative to `/api/v1`; never an absolute URL. */
  path: `/${string}`;
  body?: unknown;
  /** `"cold-start"` keeps trying while the API may be waking; `"none"` makes one attempt. */
  retry: "cold-start" | "none";
  keepalive?: boolean;
  /** Called at most once, 3 s after the first attempt started, if the request is still going. */
  onWaking?: () => void;
  signal?: AbortSignal;
};

export type HttpResult<T> =
  | { ok: true; status: number; data: T }
  | { ok: false; failure: AppFailure; apiError?: ApiError };

export type HttpDeps = {
  fetch: typeof fetch;
  now: () => number;
  sleep: (ms: number) => Promise<void>;
  onLine: () => boolean;
};

/** `detail` is for development builds only and holds nothing from the request. */
function failure(code: AppFailure["code"], retryable: boolean, detail: string): AppFailure {
  const base: AppFailure = { code, stage: "api", retryable };
  return env.dev ? { ...base, detail } : base;
}

/** The failure for a request that got no usable answer, or for a status that is not 2xx. */
export function toAppFailure(cause: "offline" | "timeout" | "status", status?: number): AppFailure {
  if (cause === "offline") {
    return failure("E_NET_OFFLINE", true, "offline");
  }
  if (cause === "timeout" || status === undefined) {
    return failure("E_NET_TIMEOUT", true, "timeout");
  }
  const detail = `HTTP ${String(status)}`;
  if (status === 429) {
    return failure("E_API_RATE_LIMITED", true, detail);
  }
  if (isGatewayStatus(status)) {
    // The proxy answered for an API that is not up yet: the same as no answer.
    return failure("E_NET_TIMEOUT", true, detail);
  }
  if (status >= 500) {
    return failure("E_API_5XX", true, detail);
  }
  // A 4xx: the request itself is wrong. The caller branches on `apiError.code`.
  return failure("E_INTERNAL", false, detail);
}

function isGatewayStatus(status: number): boolean {
  return status === 502 || status === 503 || status === 504;
}

/** Every code the server can send. The type makes this list complete. */
const API_ERROR_CODES: Record<ApiErrorCode, true> = {
  bad_request: true,
  unauthorized: true,
  forbidden: true,
  not_found: true,
  unsupported_media_type: true,
  payload_too_large: true,
  rate_limited: true,
  link_invalid: true,
  link_expired: true,
  session_expired: true,
  email_unavailable: true,
  billing_unavailable: true,
  internal: true,
};

function isApiErrorCode(code: unknown): code is ApiErrorCode {
  return typeof code === "string" && Object.hasOwn(API_ERROR_CODES, code);
}

/** The body of an error response, if it is an `ApiError`. An absent retry time reads as `null`. */
function parseApiError(text: string): ApiError | undefined {
  let body: unknown;
  try {
    body = JSON.parse(text);
  } catch {
    return undefined;
  }
  if (typeof body !== "object" || body === null || !("code" in body) || !isApiErrorCode(body.code)) {
    return undefined;
  }
  const retryAfter = "retry_after_secs" in body ? body.retry_after_secs : null;
  return { code: body.code, retry_after_secs: typeof retryAfter === "number" ? retryAfter : null };
}

/** An attempt either settles the request or may be made again. */
type Attempt<T> =
  | { again: false; result: HttpResult<T> }
  | { again: true; result: Extract<HttpResult<T>, { ok: false }> };

/** What an attempt received: a whole response, the reason `fetch` gave for none, or nothing in time. */
type Received =
  | { status: number; text: string; retryAfter: string | null }
  | { error: unknown }
  | { timedOut: true };

/** Turns a received response into the result of its attempt. */
function interpret<T>({ status, text, retryAfter }: Extract<Received, { status: number }>): Attempt<T> {
  if (status >= 200 && status < 300) {
    if (text === "") {
      // 204, or any other success without a body.
      return { again: false, result: { ok: true, status, data: undefined as T } };
    }
    try {
      return { again: false, result: { ok: true, status, data: JSON.parse(text) as T } };
    } catch {
      return { again: false, result: { ok: false, failure: failure("E_INTERNAL", false, "bad body") } };
    }
  }

  const result: Extract<HttpResult<T>, { ok: false }> = { ok: false, failure: toAppFailure("status", status) };
  const apiError = parseApiError(text);
  if (apiError !== undefined) {
    result.apiError = apiError;
  } else if (status === 429) {
    // No readable body: the wait is still in the `Retry-After` header.
    const seconds = Number(retryAfter);
    const known = retryAfter !== null && Number.isFinite(seconds);
    result.apiError = { code: "rate_limited", retry_after_secs: known ? seconds : null };
  }
  // A 4xx, 429 included, is an answer. A 5xx may be an API that is still waking.
  return { again: status >= 500, result };
}

export function createHttp(deps: HttpDeps): { request<T>(req: HttpRequest): Promise<HttpResult<T>> } {
  async function attempt<T>(req: HttpRequest, timeoutMs: number): Promise<Attempt<T>> {
    const timeout = new AbortController();
    const init: RequestInit = {
      method: req.method,
      credentials: "same-origin",
      cache: "no-store",
      signal: req.signal === undefined ? timeout.signal : AbortSignal.any([timeout.signal, req.signal]),
    };
    if (req.body !== undefined) {
      init.headers = { "Content-Type": "application/json" };
      init.body = JSON.stringify(req.body);
    }
    if (req.keepalive === true) {
      init.keepalive = true;
    }

    // The body is read inside the timeout as well. This promise never
    // rejects, so a fetch that fails after its timeout is not an unhandled rejection.
    const received: Promise<Received> = deps
      .fetch(API_PREFIX + req.path, init)
      .then(async (response) => ({
        status: response.status,
        text: await response.text(),
        retryAfter: response.headers.get("retry-after"),
      }))
      .catch((error: unknown) => ({ error }));
    const timedOut: Promise<Received> = deps.sleep(timeoutMs).then(() => ({ timedOut: true }));
    const outcome = await Promise.race([received, timedOut]);

    if ("timedOut" in outcome) {
      timeout.abort();
      return { again: true, result: { ok: false, failure: toAppFailure("timeout") } };
    }
    if ("error" in outcome) {
      // A `TypeError` is how `fetch` reports that the network cannot be reached.
      // Anything else is the caller's own abort. Neither is tried again.
      const offline = outcome.error instanceof TypeError;
      const result = offline ? toAppFailure("offline") : failure("E_INTERNAL", false, "aborted");
      return { again: false, result: { ok: false, failure: result } };
    }
    return interpret<T>(outcome);
  }

  async function request<T>(req: HttpRequest): Promise<HttpResult<T>> {
    if (!deps.onLine()) {
      return { ok: false, failure: toAppFailure("offline") };
    }
    const deadline = deps.now() + API_COLD_START_BUDGET_MS;
    let finished = false;
    const { onWaking } = req;
    if (onWaking !== undefined) {
      void deps.sleep(API_WAKING_NOTICE_MS).then(() => {
        if (!finished) {
          onWaking();
        }
      });
    }

    try {
      for (let made = 0; ; made += 1) {
        const timeoutMs = Math.min(API_TIMEOUT_MS, deadline - deps.now());
        const { again, result } = await attempt<T>(req, timeoutMs);
        const waitMs = RETRY_WAITS_MS[made] ?? MAX_RETRY_WAIT_MS;
        // Try again only while the next attempt would start inside the budget.
        if (!again || req.retry === "none" || deps.now() + waitMs >= deadline) {
          return result;
        }
        await deps.sleep(waitMs);
      }
    } finally {
      finished = true;
    }
  }

  return { request };
}

/** The client the app uses, built with the browser's own functions. */
export const http = createHttp({
  fetch: (input, init) => fetch(input, init),
  now: () => Date.now(),
  sleep: (ms) =>
    new Promise((resolve) => {
      setTimeout(resolve, ms);
    }),
  onLine: () => navigator.onLine,
});
