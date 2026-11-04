// C-11's last line: "Manage subscription" goes to the merchant's portal, where the
// person changes, cancels or resumes what they pay for. What changes there reaches
// the server by webhook, and this page by its next read.

import type { ErrorCode } from "../gen/domain";
import { getPortal } from "../net/api-client";

export type PortalOutcome = { ok: true } | { ok: false; code: ErrorCode };

/** On success the page is already on its way to the portal. */
export async function openBillingPortal(): Promise<PortalOutcome> {
  const result = await getPortal();
  if (!result.ok) {
    return { ok: false, code: result.failure.code };
  }
  window.location.assign(result.data.url);
  return { ok: true };
}
