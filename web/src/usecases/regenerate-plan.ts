// V6.2: "Regenerate plan" (v6.2implementation §8.3, D-469): the only request
// for a clip that already has a plan. It asks again with the transcript as
// heard, checks the answer as the pipeline does, and on success puts it in
// place of the clip's plan (`review-update.ts`, in the queue of changes) and
// keeps it for the clip. On any failure nothing changes, and the reason comes
// back for the interface to tell (P194). It never throws.

import { FEATURES } from "../config/features";
import type { PlanOutcome } from "../gen/api";
import { CATALOG_VERSION } from "../gen/domain";
import * as plansRepo from "../persistence/plans-repo";
import { setPlanExtras, useClipStore } from "../state/clip-store";
import { usePlanStore } from "../state/plan-store";
import { pool, whenFree } from "../workers/pool";
import { planFor } from "./request-plan";
import { replacePlan } from "./review-update";

/**
 * How a regeneration ended. `busy`: the clip was not ready, went, or could
 * not take the plan; `failed`: the render worker could not check it.
 */
export type RegenerateOutcome = { ok: true } | { ok: false; why: Exclude<PlanOutcome, "ok"> | "busy" | "failed" };

export async function regeneratePlan(): Promise<RegenerateOutcome> {
  if (!FEATURES.ai_plan) {
    return { ok: false, why: "off" };
  }
  const { status, clipId, transcript, prosody, clipInfo } = useClipStore.getState();
  if (status !== "ready" || clipId === undefined || transcript === undefined || prosody === undefined || clipInfo === undefined) {
    return { ok: false, why: "busy" };
  }
  const requestedAt = Date.now();
  const choice = await planFor({
    transcript,
    prosody,
    duration: clipInfo.duration,
    switchOn: usePlanStore.getState().switchOn,
    signal: new AbortController().signal,
  });
  if (choice.source === "rules") {
    return { ok: false, why: choice.why };
  }
  // The clip may have gone while the request was out.
  if (useClipStore.getState().clipId !== clipId) {
    return { ok: false, why: "busy" };
  }
  try {
    const wordEdits = useClipStore.getState().edit.word_edits;
    const checked = await whenFree(() => pool.render.validatePlan({ transcript, plan: choice.plan, wordEdits }));
    if ("cancelled" in checked) {
      return { ok: false, why: "busy" };
    }
    if ("refused" in checked) {
      return { ok: false, why: checked.refused === "PlanCatalog" ? "catalog" : "invalid" };
    }
    if (checked.kept === 0) {
      return { ok: false, why: "invalid" };
    }
    if (!(await replacePlan(choice.plan))) {
      return { ok: false, why: "busy" };
    }
    // P194: the panel shows the new plan's extras that passed (D-454).
    if (useClipStore.getState().clipId === clipId) {
      setPlanExtras(checked.extras);
    }
  } catch {
    // A worker that failed or crashed: the clip keeps the plan it has.
    return { ok: false, why: "failed" };
  }
  // A plan that cannot be kept still shows now; a run again would ask for it again.
  await plansRepo.put(clipId, { plan: choice.plan, source: "model", requestedAt, catalog: CATALOG_VERSION }).catch(() => undefined);
  return { ok: true };
}
