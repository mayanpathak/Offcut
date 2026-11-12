// One typed function per API route (TS §22.1). The merchant's webhook is the one
// route with none: only the merchant calls it.
//
// `net` imports no store, so the access token and what follows a refresh are
// handed in once, at app start, through `connectAuth` (v6implementation D-306).

import type {
  AccountExport,
  CheckoutRequest,
  EntitlementResponse,
  EventsBatch,
  HealthResponse,
  MagicLinkRequest,
  MeResponse,
  NotifyMeRequest,
  PlanRequest,
  PlanResponse,
  SessionResponse,
  TranscribeResponse,
  UrlResponse,
  UsageReceiptRequest,
  VerifyRequest,
} from "../gen/api";
import type { ErrorCode } from "../gen/domain";
import { http, type HttpRequest, type HttpResult } from "./http";

/** Inside the 30 s the server honours a rotated token: a later retry would read as reuse (D-297). */
export const REFRESH_BUDGET_MS = 25_000;
const REFRESH_LOCK = "offcut-auth-refresh";

export type AuthLinks = {
  accessToken(): string | null;
  /** A refresh succeeded. Called once per refresh, however many callers waited on it. */
  refreshed(session: SessionResponse): void;
  /** The server says there is no session. Called once per refresh that says so. */
  sessionExpired(): void;
};

let links: AuthLinks = {
  accessToken: () => null,
  refreshed: () => undefined,
  sessionExpired: () => undefined,
};

export function connectAuth(given: AuthLinks): void {
  links = given;
}

/** What `http.ts` leaves to this file: the 4xx answers a person can do something about (D-287). */
const ANSWERS: Partial<Record<string, ErrorCode>> = {
  link_invalid: "E_AUTH_LINK_INVALID",
  link_expired: "E_AUTH_LINK_EXPIRED",
  session_expired: "E_AUTH_SESSION_EXPIRED",
};

function mapped<T>(result: HttpResult<T>): HttpResult<T> {
  if (result.ok || result.apiError === undefined) {
    return result;
  }
  const code = ANSWERS[result.apiError.code];
  return code === undefined ? result : { ...result, failure: { ...result.failure, code, retryable: false } };
}

function isUnauthorized<T>(result: HttpResult<T>): boolean {
  return !result.ok && (result.apiError?.code === "unauthorized" || result.apiError?.code === "session_expired");
}

function waking(onWaking: (() => void) | undefined): { onWaking?: () => void } {
  return onWaking === undefined ? {} : { onWaking };
}

/**
 * `GET /healthz`, to start waking a sleeping API as early as possible. Its
 * answer also says whether cloud transcription will try (cloudimplementation
 * D-366); a failure is not reported: the first real request will show it.
 */
export function checkHealth(): Promise<HttpResult<HealthResponse>> {
  return http.request<HealthResponse>({ method: "GET", path: "/healthz", retry: "cold-start" });
}

/** `GET /healthz`, awaited: the API is up, or the reason it is not. */
export function ready(onWaking?: () => void): Promise<HttpResult<unknown>> {
  return http.request({ method: "GET", path: "/healthz", retry: "cold-start", ...waking(onWaking) });
}

/** `send`, once the API is up. A request with one attempt must not spend it on a sleeping API. */
async function afterReady<T>(send: () => Promise<HttpResult<T>>, onWaking?: () => void): Promise<HttpResult<T>> {
  const up = await ready(onWaking);
  return up.ok ? send() : up;
}

/**
 * `POST /events`. One attempt only: analytics must never hold the page up.
 * `keepalive` lets a batch sent as the tab is hidden still leave.
 */
export function postEvents(batch: EventsBatch): Promise<HttpResult<void>> {
  return http.request({ method: "POST", path: "/events", body: batch, retry: "none", keepalive: true });
}

/** `POST /notify-me`. It waits for a sleeping API, so a signup is not lost to a cold start. */
export function postNotifyMe(req: NotifyMeRequest, onWaking: () => void): Promise<HttpResult<void>> {
  return http.request({ method: "POST", path: "/notify-me", body: req, retry: "cold-start", onWaking });
}

/** `POST /auth/magic-link`. */
export function requestMagicLink(req: MagicLinkRequest, onWaking?: () => void): Promise<HttpResult<void>> {
  return afterReady(() => http.request({ method: "POST", path: "/auth/magic-link", body: req, retry: "none" }), onWaking);
}

/** `POST /auth/verify`. A link works once, so it is not sent twice. */
export async function verifyMagicLink(req: VerifyRequest, onWaking?: () => void): Promise<HttpResult<SessionResponse>> {
  return mapped(await afterReady(() => http.request<SessionResponse>({ method: "POST", path: "/auth/verify", body: req, retry: "none" }), onWaking));
}

/** The lock keeps two tabs' refreshes apart. A tab that waited sends whatever cookie the browser then holds (D-297). */
function underLock<T>(run: () => Promise<T>): Promise<T> {
  return "locks" in navigator ? navigator.locks.request(REFRESH_LOCK, run) : run();
}

async function sendRefresh(): Promise<HttpResult<SessionResponse>> {
  // The wake first: the budget below must not be spent on a sleeping API (D-297).
  const up = await ready();
  if (!up.ok) {
    return up;
  }
  const result = mapped(
    await underLock(() =>
      http.request<SessionResponse>({ method: "POST", path: "/auth/refresh", retry: "cold-start", budgetMs: REFRESH_BUDGET_MS, csrf: true }),
    ),
  );
  if (result.ok) {
    links.refreshed(result.data);
  } else if (result.failure.code === "E_AUTH_SESSION_EXPIRED") {
    links.sessionExpired();
  }
  return result;
}

let refreshing: Promise<HttpResult<SessionResponse>> | undefined;

/** `POST /auth/refresh`. One at a time on this page: every caller shares the one in flight (D-288). */
export function refresh(): Promise<HttpResult<SessionResponse>> {
  refreshing ??= sendRefresh().finally(() => {
    refreshing = undefined;
  });
  return refreshing;
}

/**
 * Sends with the access token. On a 401 it refreshes once, then repeats once
 * (D-288). A second 401 means the session is gone whatever the refresh said.
 */
async function authed<T>(req: HttpRequest): Promise<HttpResult<T>> {
  const sent = links.accessToken();
  if (sent !== null) {
    const first = await http.request<T>({ ...req, bearer: sent });
    if (!isUnauthorized(first)) {
      return mapped(first);
    }
  }
  // Another caller may have refreshed while this one's request was out.
  let bearer = links.accessToken();
  if (bearer === null || bearer === sent) {
    const renewed = await refresh();
    if (!renewed.ok) {
      return renewed;
    }
    bearer = renewed.data.access_token;
  }
  const second = await http.request<T>({ ...req, bearer });
  if (isUnauthorized(second)) {
    links.sessionExpired();
    if (!second.ok) {
      return { ...second, failure: { ...second.failure, code: "E_AUTH_SESSION_EXPIRED", retryable: false } };
    }
  }
  return mapped(second);
}

/** One attempt of `POST /transcribe` may take this long: the upload and the provider (D-374). */
export const TRANSCRIBE_TIMEOUT_MS = 30_000;

/**
 * `POST /transcribe`: one clip's compressed sound, its words back (D-372).
 * One attempt; a failure is the caller's to fall back from, never a retry.
 */
export function transcribeAudio(
  ogg: Uint8Array<ArrayBuffer>,
  signal: AbortSignal,
  window?: { index: number; total: number },
): Promise<HttpResult<TranscribeResponse>> {
  return authed<TranscribeResponse>({
    method: "POST",
    path: "/transcribe",
    rawBody: { bytes: ogg, contentType: "audio/ogg" },
    retry: "none",
    attemptTimeoutMs: TRANSCRIBE_TIMEOUT_MS,
    signal,
    // V6.1 (D-413): a window of a long source says so; a clip's request is as before.
    ...(window === undefined ? {} : { window }),
  });
}

/** V6.2 (v6.2implementation D-466, D-498): how long the whole plan request may take. */
export const PLAN_CLIENT_TIMEOUT_MS = 20_000;

/**
 * V6.2 (D-466): `POST /plan`, one clip's transcript, its plan back. One
 * attempt, as `transcribeAudio`: a 502 or 504 with `plan_unavailable` or
 * `plan_invalid` is the running API's answer, read by its code, never waited
 * out as a waking API (V6 CL known issue 4). A 401 refreshes once (D-288).
 */
export function requestPlan(body: PlanRequest, signal: AbortSignal): Promise<HttpResult<PlanResponse>> {
  return authed<PlanResponse>({
    method: "POST",
    path: "/plan",
    body,
    retry: "none",
    attemptTimeoutMs: PLAN_CLIENT_TIMEOUT_MS,
    signal,
  });
}

/** `POST /auth/logout`. It answers 204 for any cookie. */
export function logout(): Promise<HttpResult<void>> {
  return afterReady(() => http.request({ method: "POST", path: "/auth/logout", retry: "none", csrf: true }));
}

/** `GET /me`. */
export function getMe(): Promise<HttpResult<MeResponse>> {
  return authed({ method: "GET", path: "/me", retry: "cold-start" });
}

/** `GET /entitlement`. */
export function getEntitlement(): Promise<HttpResult<EntitlementResponse>> {
  return authed({ method: "GET", path: "/entitlement", retry: "cold-start" });
}

/** `POST /usage/receipts`. One attempt: the outbox is the retry. */
export function postUsageReceipt(req: UsageReceiptRequest): Promise<HttpResult<EntitlementResponse>> {
  return authed({ method: "POST", path: "/usage/receipts", body: req, retry: "none" });
}

/** `POST /billing/checkout`. One attempt: each makes a session at the merchant. */
export function createCheckout(req: CheckoutRequest, onWaking?: () => void): Promise<HttpResult<UrlResponse>> {
  return afterReady(() => authed<UrlResponse>({ method: "POST", path: "/billing/checkout", body: req, retry: "none" }), onWaking);
}

/** `GET /billing/portal`. */
export function getPortal(): Promise<HttpResult<UrlResponse>> {
  return authed({ method: "GET", path: "/billing/portal", retry: "cold-start" });
}

/** `POST /account/delete`. One attempt: the person sees the result. */
export function deleteAccount(onWaking?: () => void): Promise<HttpResult<void>> {
  return afterReady(() => authed<void>({ method: "POST", path: "/account/delete", retry: "none" }), onWaking);
}

/** `GET /account/export`. */
export function exportAccount(): Promise<HttpResult<AccountExport>> {
  return authed({ method: "GET", path: "/account/export", retry: "cold-start" });
}
