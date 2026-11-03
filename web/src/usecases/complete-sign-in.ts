// C-8's second half: the link was opened, in a tab of its own. The server turns
// its token into a session; the other tabs are told, and ask for themselves.

import { track } from "../analytics/client";
import type { ErrorCode } from "../gen/domain";
import { verifyMagicLink } from "../net/api-client";
import * as entitlementRepo from "../persistence/entitlement-repo";
import { flushOutbox } from "../persistence/receipt-outbox";
import { postSignedIn } from "../platform/broadcast";
import { verified } from "../state/auth-store";
import { setToken } from "../state/entitlement-store";

export type SignInOutcome = { ok: true } | { ok: false; code: ErrorCode };

export async function completeSignIn(token: string, onWaking: () => void): Promise<SignInOutcome> {
  const result = await verifyMagicLink({ token }, onWaking);
  if (!result.ok) {
    return { ok: false, code: result.failure.code };
  }
  setToken(result.data.entitlement_token);
  verified(result.data.access_token);
  try {
    await entitlementRepo.put(result.data.entitlement_token);
  } catch {
    // The session is good, and the page holds the token. A device that cannot
    // store it asks the server again at its next start.
    track({ name: "client_error", props: { error_code: "E_STORAGE_IO", stage: "api" } });
  }
  track({ name: "signin_step", props: { step: "completed" } });
  postSignedIn();
  // Receipts that waited for this account (v6implementation D-268).
  void flushOutbox();
  return { ok: true };
}
