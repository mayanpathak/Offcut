// C-1: what happens once, when the app starts. V1 has the first steps; the
// later versions add theirs at the marked places, in this order.

import { initAnalytics, track } from "../analytics/client";
import { env } from "../config/env";
import type { AnonId } from "../gen/domain";
import * as modelManager from "../models/model-manager";
import { wake } from "../net/api-client";
import { metaGet, metaSet } from "../persistence/db";
import { META_KEYS } from "../persistence/schema";
import { runCapabilityCheck } from "../platform/capability";
import { beginCheck, setSupported, setUnsupported } from "../state/capability-store";
import { setIllegalTransitionReporter } from "../state/machines/transition";
import { preload } from "../workers/pool";

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;

/** An anonymous id is a UUID: the server refuses a batch whose id is anything else. */
function isAnonId(value: unknown): value is AnonId {
  return typeof value === "string" && UUID.test(value);
}

function newAnonId(): AnonId {
  const id: string = crypto.randomUUID();
  if (!isAnonId(id)) {
    throw new Error("crypto.randomUUID() did not return a UUID");
  }
  return id;
}

/**
 * The anonymous id kept in this browser, made on the first visit. If storage
 * cannot be used, the id lasts for this page load only: a browser that
 * cannot run Offcut must still be able to report why.
 */
async function anonymousId(): Promise<AnonId> {
  try {
    const stored = await metaGet<unknown>(META_KEYS.anonId);
    if (isAnonId(stored)) {
      return stored;
    }
    const fresh = newAnonId();
    await metaSet(META_KEYS.anonId, fresh);
    return fresh;
  } catch {
    return newAnonId();
  }
}

async function run(): Promise<void> {
  // 1. An illegal state transition throws in development and in tests. In a
  //    production build it is reported, and the state stays as it was.
  if (!env.dev) {
    setIllegalTransitionReporter(() => {
      track({ name: "client_error", props: { error_code: "E_INTERNAL", stage: "import" } });
    });
  }

  // 2, 3. Analytics. The id is handed over: analytics reads no store.
  initAnalytics({ anonId: await anonymousId() });

  // 4. Only "outcome" until the headline experiment (E-6) starts.
  track({ name: "landing_view", props: { hero_variant: "outcome" } });

  // 5. Can this browser run Offcut? The report is also the E-8 measurement.
  beginCheck();
  const report = await runCapabilityCheck();
  const reason = report.unsupported_reason;
  if (reason === undefined || reason === null) {
    setSupported(report);
  } else {
    setUnsupported(reason, report);
  }
  track({ name: "capability_check", props: report });

  // 6. Start waking a sleeping API now. Nothing waits for the answer.
  wake();

  // V6: restoreSession()

  // 7. Fetch and compile the WASM bundle, where it will be used.
  if (reason === undefined || reason === null) {
    const preloaded = await preload();
    if (!preloaded.ok) {
      track({ name: "client_error", props: { error_code: preloaded.failure.code, stage: preloaded.failure.stage } });
    }

    // 8. What of the speech model is on this device. Nothing waits for the
    //    answer, and a storage that cannot be read counts as no model.
    void modelManager.inspect();
  }

  // V7: restoreClip()
}

let started: Promise<void> | undefined;

/** Starts the app. A second call returns the promise of the first. */
export function startApp(): Promise<void> {
  started ??= run();
  return started;
}
