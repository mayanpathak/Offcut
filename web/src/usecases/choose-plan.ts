// V6.2: which plan a clip's run builds its scene from (v6.2implementation
// D-467, D-469, D-490, §8.2). Step 11 of `run-pipeline.ts` starts `startPlan`
// once the prosody is measured; step 12 hands its answer and the rules plan
// to `choosePlan`. A model's plan is used when it keeps at least one item
// against the transcript as edited; otherwise the rules plan, and the reason
// is told once. Only with `ai_plan` on: the pipeline calls neither otherwise.

import { track } from "../analytics/client";
import type { LatencyBucket, PlanOutcome } from "../gen/api";
import {
  CATALOG_VERSION,
  type ClipId,
  type DropReason,
  type Dropped,
  type DurMs,
  type EditPlan,
  type Prosody,
  type Transcript,
} from "../gen/domain";
import * as plansRepo from "../persistence/plans-repo";
import type { PlanRecord } from "../persistence/schema";
import * as clipStore from "../state/clip-store";
import { usePlanStore } from "../state/plan-store";
import type { PlanCheck } from "../workers/protocol";
import { type PlanChoice, planFor, reasonNotToAsk } from "./request-plan";

/** `plan_result`'s caps (TS §22.6, D-490): the items, and each count of drops. */
const MAX_COUNT = 10_000;
const MAX_DROPPED = 99;

/** What step 11 started: the plan an earlier run of this clip kept, or a choice made now. */
export type Planned =
  | { stored: PlanRecord }
  | { choice: PlanChoice; asked: boolean; latencyMs: number; requestedAt: number };

/** The server's buckets of the provider's time (`plan/route.rs`), for the whole request. */
function latencyBucket(ms: number): LatencyBucket {
  if (ms < 2_000) {
    return "lt2";
  }
  if (ms < 5_000) {
    return "lt5";
  }
  if (ms < 10_000) {
    return "lt10";
  }
  return ms < 20_000 ? "lt20" : "gte20";
}

/** Each reason's count of drops, capped (D-490). */
function droppedByReason(dropped: readonly Dropped[]): Partial<Record<DropReason, number>> {
  const counts: Partial<Record<DropReason, number>> = {};
  for (const { reason } of dropped) {
    counts[reason] = Math.min(MAX_DROPPED, (counts[reason] ?? 0) + 1);
  }
  return counts;
}

/**
 * Step 11b: the plan an earlier run of this clip kept, used with no request
 * (Retry, Process again; D-469), or `planFor`'s answer now. A stored plan is
 * read only while the switch is on. Never rejects: a store that cannot be
 * read asks again.
 */
export async function startPlan(
  clipId: ClipId,
  transcript: Transcript,
  prosody: Prosody,
  duration: DurMs,
  signal: AbortSignal,
): Promise<Planned> {
  const { switchOn } = usePlanStore.getState();
  if (switchOn) {
    const stored = await plansRepo.get(clipId).catch(() => undefined);
    if (stored !== undefined) {
      return { stored };
    }
  }
  const asked = reasonNotToAsk(switchOn) === undefined;
  const requestedAt = Date.now();
  const startedAt = performance.now();
  const choice = await planFor({ transcript, prosody, duration, switchOn, signal });
  return { choice, asked, latencyMs: performance.now() - startedAt, requestedAt };
}

/**
 * Step 12b: the plan the scene is built from (D-467). `check` is the render
 * worker's `validatePlan` with the clip's word edits, as the pipeline calls
 * it. A plan asked for now is kept for this clip, the rules plan too when the
 * model's could not be used, so a run again asks for nothing (D-469). A
 * stored plan sends no `plan_result`: its run sent one.
 */
export async function choosePlan(
  planned: Planned,
  rules: EditPlan,
  clipId: ClipId,
  check: (plan: EditPlan) => Promise<PlanCheck>,
): Promise<EditPlan> {
  if ("stored" in planned) {
    if (planned.stored.source === "rules") {
      return rules;
    }
    const kept = await check(planned.stored.plan);
    return "kept" in kept && kept.kept > 0 ? planned.stored.plan : rules;
  }

  const { choice, asked, latencyMs, requestedAt } = planned;
  let plan = rules;
  let outcome: PlanOutcome;
  let dropped: readonly Dropped[] = [];
  let items = rules.items.length;
  if (choice.source === "model") {
    const checked = await check(choice.plan);
    if ("refused" in checked) {
      // DC-7: a plan of a newer catalog than this build's is refused whole.
      outcome = checked.refused === "PlanCatalog" ? "catalog" : "invalid";
    } else if (checked.kept > 0) {
      dropped = checked.dropped;
      outcome = "ok";
      plan = choice.plan;
      items = checked.kept;
    } else {
      // Every item dropped: the rules plan, and the drops are still counted.
      dropped = checked.dropped;
      outcome = "invalid";
    }
  } else {
    outcome = choice.why;
  }

  const source = plan === rules ? "rules" : "model";
  if (asked) {
    const record: PlanRecord = { plan, source, requestedAt, catalog: CATALOG_VERSION };
    // A plan that cannot be kept is still this run's: a later run asks again.
    await plansRepo.put(clipId, record).catch(() => undefined);
  }
  if (outcome !== "ok" && outcome !== "off") {
    clipStore.planFellBack(outcome);
  }
  track({
    name: "plan_result",
    props: {
      source,
      outcome,
      ...(asked && { latency_bucket: latencyBucket(latencyMs) }),
      items: Math.min(MAX_COUNT, items),
      dropped_by_reason: droppedByReason(dropped),
    },
  });
  return plan;
}
