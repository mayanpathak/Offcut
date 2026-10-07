// C-16: join the waitlist, or ask to be told when a platform is supported.
//
// No analytics event is sent for this: none is on the allowlist, and an email
// address must never reach analytics.

import type { Wanted } from "../gen/api";
import { postNotifyMe } from "../net/api-client";

export type NotifyMeOutcome =
  | { ok: true }
  | {
      ok: false;
      reason: "invalid_email" | "rate_limited" | "offline" | "unavailable";
      /** With `rate_limited`, when the server said how long to wait. */
      retryAfterSecs?: number;
    };

/**
 * Sends the email to the waitlist. `onWaking` is called once if the API is
 * slow to answer, which is how a sleeping API looks (TS C-15).
 */
export async function submitNotifyMe(
  email: string,
  wanted: Wanted,
  onWaking: () => void,
): Promise<NotifyMeOutcome> {
  const trimmed = email.trim();
  // The server decides what an email is. This only saves a request that is sure to fail.
  if (!trimmed.includes("@")) {
    return { ok: false, reason: "invalid_email" };
  }

  const result = await postNotifyMe({ email: trimmed, wanted }, onWaking);
  if (result.ok) {
    return { ok: true };
  }
  if (result.apiError?.code === "bad_request") {
    return { ok: false, reason: "invalid_email" };
  }
  if (result.failure.code === "E_API_RATE_LIMITED") {
    const retryAfterSecs = result.apiError?.retry_after_secs;
    return typeof retryAfterSecs === "number"
      ? { ok: false, reason: "rate_limited", retryAfterSecs }
      : { ok: false, reason: "rate_limited" };
  }
  if (result.failure.code === "E_NET_OFFLINE") {
    return { ok: false, reason: "offline" };
  }
  return { ok: false, reason: "unavailable" };
}
