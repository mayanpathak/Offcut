// C-11's second half: the merchant sends the person back to `/account`. The plan
// changes when the merchant's webhook reaches the server, which can be after the
// person arrives, so a success is asked of `GET /me` until it shows Creator, for
// up to a minute. The account page also reads what the account holds through here.

import { track } from "../analytics/client";
import type { MeResponse } from "../gen/api";
import type { ErrorCode } from "../gen/domain";
import { getMe } from "../net/api-client";
import * as entitlementRepo from "../persistence/entitlement-repo";
import { setEmail } from "../state/auth-store";
import { setToken } from "../state/entitlement-store";

/** TS C-11: every 2 s, up to 60 s. */
export const POLL_EVERY_MS = 2_000;
export const POLL_FOR_MS = 60_000;

export type CheckoutReturn = "success" | "cancel";

export type AccountRead = { ok: true; me: MeResponse } | { ok: false; code: ErrorCode };

export type ConfirmOutcome =
  | { kind: "active"; me: MeResponse }
  /** No Creator within the minute: the webhook may still come (`E_BILLING_PENDING`). */
  | { kind: "pending" }
  | { kind: "cancelled" }
  | { kind: "failed"; code: ErrorCode };

const wait = (ms: number) =>
  new Promise<void>((resolve) => {
    setTimeout(resolve, ms);
  });

/** What the account holds now. Its token replaces the one on the page and on the device. */
export async function readAccount(): Promise<AccountRead> {
  const result = await getMe();
  if (!result.ok) {
    return { ok: false, code: result.failure.code };
  }
  setToken(result.data.entitlement_token);
  setEmail(result.data.email);
  try {
    await entitlementRepo.put(result.data.entitlement_token);
  } catch {
    // The page holds the token; a device that cannot store it asks again at its next start.
    track({ name: "client_error", props: { error_code: "E_STORAGE_IO", stage: "api" } });
  }
  return { ok: true, me: result.data };
}

/**
 * Asks until the plan is Creator or the minute is over. A failed read is asked
 * again, as the next may work; a session that has ended is not.
 */
export async function waitForCreator(): Promise<ConfirmOutcome> {
  const deadline = Date.now() + POLL_FOR_MS;
  for (;;) {
    const read = await readAccount();
    if (read.ok && read.me.plan === "creator") {
      return { kind: "active", me: read.me };
    }
    if (!read.ok && read.code === "E_AUTH_SESSION_EXPIRED") {
      return { kind: "failed", code: read.code };
    }
    if (Date.now() >= deadline) {
      return { kind: "pending" };
    }
    await wait(POLL_EVERY_MS);
  }
}

/** Once per return: the page has taken `?checkout=` out of its address before this runs. */
export function confirmCheckout(result: CheckoutReturn): Promise<ConfirmOutcome> {
  track({ name: "checkout_step", props: { step: result === "success" ? "returned_success" : "returned_cancel" } });
  return result === "success" ? waitForCreator() : Promise.resolve({ kind: "cancelled" });
}
