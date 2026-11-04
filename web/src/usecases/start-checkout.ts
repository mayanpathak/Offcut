// C-11's first half: the person chooses an offer and goes to the merchant's page,
// a full navigation with no script of theirs on ours. A person who already pays
// is refused a second subscription by the server, and goes to the portal instead
// (v6implementation D-300).

import { flush, track } from "../analytics/client";
import type { Offer, PromptPlacement } from "../gen/api";
import type { ErrorCode } from "../gen/domain";
import { createCheckout, getPortal } from "../net/api-client";
import { forCheckout } from "../state/blockers";

export type CheckoutOutcome = { ok: true } | { ok: false; code: ErrorCode };

/**
 * `from` is the prompt's placement when the offer was chosen there. On success the
 * page is already on its way to the merchant: nothing after it runs for long.
 */
export async function startCheckout(offer: Offer, from?: PromptPlacement, onWaking?: () => void): Promise<CheckoutOutcome> {
  if (from !== undefined) {
    track({ name: "upgrade_prompt", props: { placement: from, action: "clicked" } });
  }
  // A token on this device is no session (v6implementation §12).
  if (forCheckout() !== null) {
    return { ok: false, code: "E_AUTH_SESSION_EXPIRED" };
  }
  track({ name: "checkout_step", props: { step: "started" } });
  const checkout = await createCheckout({ offer }, onWaking);
  if (checkout.ok) {
    // On its way before the page goes: the funnel's step is not left to the send as it unloads.
    void flush();
    window.location.assign(checkout.data.url);
    return { ok: true };
  }
  if (checkout.apiError?.code !== "forbidden") {
    return { ok: false, code: checkout.failure.code };
  }
  // A subscription is live already: what it holds is changed in the portal.
  const portal = await getPortal();
  if (!portal.ok) {
    return { ok: false, code: portal.failure.code };
  }
  window.location.assign(portal.data.url);
  return { ok: true };
}
