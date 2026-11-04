// The person signs out: the session ends at the server, and the token leaves this
// device, so the next export asks for a sign-in (v6implementation D-291). A lost
// session keeps the token; this is the only path that clears it, but for clearing
// local data. Unsent receipts stay, for this account's next sign-in (D-267).

import { track } from "../analytics/client";
import type { ErrorCode } from "../gen/domain";
import { logout } from "../net/api-client";
import * as entitlementRepo from "../persistence/entitlement-repo";
import { signedOut } from "../state/auth-store";
import { setToken } from "../state/entitlement-store";

export type SignOutOutcome = { ok: true } | { ok: false; code: ErrorCode };

/**
 * A failure changes nothing: the cookie is one this page cannot remove, so the
 * next start would sign the person back in, and they should know they are not out.
 */
export async function signOut(): Promise<SignOutOutcome> {
  const result = await logout();
  if (!result.ok) {
    return { ok: false, code: result.failure.code };
  }
  signedOut();
  setToken(null);
  try {
    await entitlementRepo.clear();
  } catch {
    track({ name: "client_error", props: { error_code: "E_STORAGE_IO", stage: "api" } });
  }
  return { ok: true };
}
