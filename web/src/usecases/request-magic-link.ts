// C-8's first half: the person asks for a sign-in link. The address goes to the
// server and to this tab's store, and never to analytics (TS §22.6).

import { track } from "../analytics/client";
import type { ErrorCode } from "../gen/domain";
import { requestMagicLink as sendLink } from "../net/api-client";
import { dialogClosed, linkAccepted, useAuthStore } from "../state/auth-store";

export type LinkOutcome =
  | { ok: true }
  | { ok: false; reason: "invalid_email" }
  /** `retryAfterSecs` with `E_API_RATE_LIMITED`, when the server said how long to wait. */
  | { ok: false; reason: "failed"; code: ErrorCode; retryAfterSecs?: number };

/** `onWaking` is called once if the API is slow to answer, which is how a sleeping API looks (TS C-15). */
export async function requestMagicLink(email: string, onWaking: () => void): Promise<LinkOutcome> {
  const trimmed = email.trim();
  // The server decides what an email is. This only saves a request that is sure to fail.
  if (!trimmed.includes("@")) {
    return { ok: false, reason: "invalid_email" };
  }
  const result = await sendLink({ email: trimmed }, onWaking);
  if (!result.ok) {
    if (result.apiError?.code === "bad_request") {
      return { ok: false, reason: "invalid_email" };
    }
    const retryAfterSecs = result.apiError?.retry_after_secs;
    return typeof retryAfterSecs === "number"
      ? { ok: false, reason: "failed", code: result.failure.code, retryAfterSecs }
      : { ok: false, reason: "failed", code: result.failure.code };
  }
  // Another tab may have signed this one in while the request was out.
  if (useAuthStore.getState().status !== "signed_in") {
    linkAccepted(trimmed);
  }
  track({ name: "signin_step", props: { step: "link_requested" } });
  return { ok: true };
}

/** The dialog closed. A wait for the link ends with it (TS §12.2); no other state changes. */
export function stopWaitingForLink(): void {
  if (useAuthStore.getState().status === "link_sent") {
    dialogClosed();
  }
}
