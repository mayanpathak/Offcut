// C-18: the person deletes their account. The server cancels any subscription first,
// and deletes nothing if it cannot (v6implementation D-271); then the session and
// the token leave this device. Unsent receipts stay: they belong to an account that
// is gone, and no other account sends them (D-267).

import { track } from "../analytics/client";
import type { ErrorCode } from "../gen/domain";
import { deleteAccount as sendDelete } from "../net/api-client";
import * as entitlementRepo from "../persistence/entitlement-repo";
import { accountDeleted } from "../state/auth-store";
import { setToken } from "../state/entitlement-store";

export type DeleteOutcome = { ok: true } | { ok: false; code: ErrorCode };

/** A failure changes nothing here, as it changed nothing at the server. */
export async function deleteAccount(onWaking?: () => void): Promise<DeleteOutcome> {
  const result = await sendDelete(onWaking);
  if (!result.ok) {
    return { ok: false, code: result.failure.code };
  }
  accountDeleted();
  setToken(null);
  try {
    await entitlementRepo.clear();
  } catch {
    // The page has forgotten it. A device that kept it exports until its `exp`, as an offline one would.
    track({ name: "client_error", props: { error_code: "E_STORAGE_IO", stage: "api" } });
  }
  return { ok: true };
}
