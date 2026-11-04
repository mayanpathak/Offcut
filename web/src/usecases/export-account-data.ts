// C-18: the person takes their account's data with them (PS §17): what the server
// holds of them, as one JSON file, downloaded. Session hashes are never in it
// (TS §22.7). Nothing of a video is: the server has none.

import type { ErrorCode } from "../gen/domain";
import { exportAccount } from "../net/api-client";

/** The file's name. It names no person: the address is inside. */
export const ACCOUNT_FILE_NAME = "offcut-account-data.json";
/** How long the page keeps the address of the file it handed over, as an export does (TS §31). */
const REVOKE_AFTER_MS = 60_000;
/** Indented, so that a person can read it. */
const JSON_INDENT = 2;

export type ExportDataOutcome = { ok: true } | { ok: false; code: ErrorCode };

export async function exportAccountData(): Promise<ExportDataOutcome> {
  const result = await exportAccount();
  if (!result.ok) {
    return { ok: false, code: result.failure.code };
  }
  const url = URL.createObjectURL(new Blob([JSON.stringify(result.data, null, JSON_INDENT)], { type: "application/json" }));
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = ACCOUNT_FILE_NAME;
  document.body.append(anchor);
  anchor.click();
  anchor.remove();
  setTimeout(() => {
    URL.revokeObjectURL(url);
  }, REVOKE_AFTER_MS);
  return { ok: true };
}
