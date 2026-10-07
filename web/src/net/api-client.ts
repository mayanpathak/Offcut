// One typed function per API route. V1 has 3 of the 15 (TS §22.1).

import type { EventsBatch, NotifyMeRequest } from "../gen/api";
import { http, type HttpResult } from "./http";

/**
 * `GET /healthz`, to start waking a sleeping API as early as possible. The
 * answer is not used and a failure is not reported: the first real request
 * will show it.
 */
export function wake(): void {
  void http.request({ method: "GET", path: "/healthz", retry: "cold-start" });
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
