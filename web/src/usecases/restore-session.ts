// The session this browser may still hold (TS C-1, v6implementation §11.3,
// D-289, D-296). `connectAccount` runs first thing on every page, as the callback
// tab needs it too; `restoreSession` runs at its marker, on every page but the
// callback, and nothing waits for it (INV-3).

import { track } from "../analytics/client";
import type { ErrorCode, ExportId, UserId } from "../gen/domain";
import { connectAuth, postUsageReceipt, refresh } from "../net/api-client";
import * as entitlementRepo from "../persistence/entitlement-repo";
import { connectOutbox, flushOutbox, mirrorOutbox, type SendOutcome } from "../persistence/receipt-outbox";
import { onSignedIn } from "../platform/broadcast";
import { refreshOk, refreshRefused, refreshUnreachable, useAuthStore } from "../state/auth-store";
import { setOutboxFree, setToken, useEntitlementStore } from "../state/entitlement-store";

function report(code: ErrorCode): void {
  track({ name: "client_error", props: { error_code: code, stage: "api" } });
}

/** Whose receipts may be sent: the token's account, and only with a session behind it (D-267, D-268). */
function signedInSub(): UserId | null {
  if (useAuthStore.getState().status !== "signed_in") {
    return null;
  }
  return useEntitlementStore.getState().claims?.sub ?? null;
}

/** A token on the page and on the device. A device that cannot store it still has it for this page. */
async function keepToken(token: string): Promise<void> {
  setToken(token);
  try {
    await entitlementRepo.put(token);
  } catch {
    report("E_STORAGE_IO");
  }
}

async function sendReceipt(exportId: ExportId): Promise<SendOutcome> {
  const result = await postUsageReceipt({ export_id: exportId });
  if (result.ok) {
    return { kind: "sent", token: result.data.entitlement_token };
  }
  if (result.failure.code === "E_AUTH_SESSION_EXPIRED") {
    return { kind: "no_session" };
  }
  // A readable 4xx can never succeed. 429 is one that can: the wait is over soon,
  // and a row dropped here would hand back a Free export (v6implementation D-307).
  return !result.failure.retryable && result.apiError !== undefined ? { kind: "refused" } : { kind: "later" };
}

/** Hands `net` and `persistence` what they may not import. Once, before anything can ask them. */
export function connectAccount(): void {
  connectAuth({
    accessToken: () => useAuthStore.getState().accessToken,
    refreshed: (session) => {
      refreshOk(session.access_token);
      void keepToken(session.entitlement_token);
    },
    // A lost session keeps the token: offline exports go on until `exp` (D-291).
    sessionExpired: refreshRefused,
  });
  connectOutbox({
    signedInSub,
    send: sendReceipt,
    tokenReceived: (token) => void keepToken(token),
    rowsChanged: setOutboxFree,
    report,
  });
  window.addEventListener("online", () => {
    void flushOutbox();
  });
}

/** One refresh. `net` has already told the stores of a good one, or of a lost session. */
async function tryRefresh(): Promise<void> {
  const result = await refresh();
  if (result.ok) {
    await flushOutbox();
    return;
  }
  if (result.failure.code === "E_AUTH_SESSION_EXPIRED") {
    return;
  }
  refreshUnreachable();
  if (useAuthStore.getState().status === "anonymous_offline") {
    window.addEventListener(
      "online",
      () => {
        void tryRefresh();
      },
      { once: true },
    );
  }
}

/** What this device already knows, before any answer: a cached token still exports offline (C-15). */
async function loadDevice(): Promise<void> {
  try {
    const record = await entitlementRepo.get();
    if (record !== undefined) {
      setToken(record.token);
    }
    await mirrorOutbox();
  } catch {
    // Storage that cannot be read holds no token: the person is asked to sign in.
    report("E_STORAGE_IO");
  }
}

/** C-1's step. Another tab's sign-in asks again (TS C-8). It never rejects. */
export async function restoreSession(): Promise<void> {
  onSignedIn(() => {
    void tryRefresh();
  });
  await loadDevice();
  await tryRefresh();
}
