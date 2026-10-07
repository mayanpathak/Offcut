// The worker pool. V1 has no worker yet: it only fetches and compiles the
// core WASM bundle at app start (D-9), which proves in production that the
// build, the MIME type and the CSP are right. V2 adds the four workers.

import { preloadCore } from "../wasm/load-core";
import type { AppFailure } from "./protocol";

/**
 * Fetches and compiles what the workers will need. A failure here means the
 * bundle could not be loaded: the same failure as a worker script that
 * cannot be loaded (TS §14.4).
 */
export async function preload(): Promise<{ ok: true } | { ok: false; failure: AppFailure }> {
  try {
    await preloadCore();
    return { ok: true };
  } catch {
    return { ok: false, failure: { code: "E_WORKER_CRASH", stage: "import", retryable: true } };
  }
}
