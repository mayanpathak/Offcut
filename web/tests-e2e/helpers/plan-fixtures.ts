// V6.2: the AI plan's fixtures for `ai-plan.spec.ts` (v6.2implementation
// §13.4, D-468). The clip is transcribed in the cloud through the fake API,
// from the real adapter's answer for `speech_60s_portrait.mp4`
// (`fixtures/cloud/`), so its words, and the word indexes a plan points at,
// are the same every run. No test reaches DeepSeek: `/plan` answers with
// `fixtures/plan/speech_60s_portrait.json`, a model's plan for those words,
// made by hand from TE-15's DeepSeek Flash answers for the same script
// (`a01-scriptA`, P188 to P206c) on the items of P206's `all_kinds.rs`: the
// hook, two emoji, four emphases (one a punch word), two zooms and two icons.

import { readFileSync } from "node:fs";
import path from "node:path";

import type { PlanResponse, TranscribeResponse } from "../../src/gen/api";
import type { EventKind } from "../../src/gen/domain";
import type { FakeApiOptions, FakePlan } from "./fake-api";

const read = (file: string): unknown => JSON.parse(readFileSync(path.join(import.meta.dirname, "../fixtures", file), "utf8"));

const CLOUD = read("cloud/speech_60s_portrait.json") as TranscribeResponse;

/** The model's plan for the clip, as `POST /plan` answers it. */
export const PLAN_ANSWER = read("plan/speech_60s_portrait.json") as PlanResponse;

/** Groq invents "Thank you." in the clip's silent end; the silence guard drops both words (cloud D-382). */
const INVENTED = 2;
/** The words of the clip's transcript as heard: what `POST /plan` carries. */
export const HEARD_WORDS = CLOUD.words.length - INVENTED;

/**
 * Words the fake transcript plants in place of three of the clip's (D-468), at
 * words no item of the plan points at. Letters only, so no rule reads them as
 * a number; found in no request unless the transcript's words are in it.
 */
export const MARKERS: Readonly<Record<number, string>> = { 17: "Zorblax", 46: "Quintessa", 103: "Vexmoor" };

/** The cloud's answer for the clip, with `MARKERS` planted. */
export function markedAnswer(): TranscribeResponse {
  return {
    ...CLOUD,
    words: CLOUD.words.map((word, i) => {
      const marker = MARKERS[i];
      return marker === undefined ? word : { ...word, text: marker };
    }),
  };
}

/** The kinds of the plan's items, in its order. */
export const PLAN_KINDS: readonly EventKind[] = PLAN_ANSWER.plan.items.map((item) => item.kind);

/**
 * What "Regenerate plan" gets back: the same model's plan with three of its
 * items (the hook, the fire emoji and the laptop icon) and a new post
 * caption, so the list and the panel show that it changed.
 */
export function regeneratedAnswer(): PlanResponse {
  const kept = new Set(["1", "5", "11"]);
  return {
    plan: {
      ...PLAN_ANSWER.plan,
      items: PLAN_ANSWER.plan.items.filter((item) => kept.has(item.id)),
      extras: { ...PLAN_ANSWER.plan.extras, post_caption: "Three reasons my edits got faster, all on my laptop." },
    },
    plans_remaining: (PLAN_ANSWER.plans_remaining ?? 1) - 1,
  };
}

/**
 * P211a (D-504): the recorded plan with one more item the validation leaves
 * out, an emoji on a word past the transcript's end (`anchors_out_of_range`):
 * the list must not show it.
 */
export function withDroppedItem(): PlanResponse {
  const word = HEARD_WORDS + 50;
  const dropped = {
    id: "12",
    kind: "emoji",
    anchors: { start: word, end: word + 1 },
    params: { kind: "emoji", params: { word, emoji: "rocket", place: "beside" } },
    confidence: null,
  } as unknown as PlanResponse["plan"]["items"][number];
  return { ...PLAN_ANSWER, plan: { ...PLAN_ANSWER.plan, items: [...PLAN_ANSWER.plan.items, dropped] } };
}

/** Signed in, the cloud on and answering with `markedAnswer()`, and `/plan` as `plan` says. */
export function signedInPlan(plan: FakePlan | undefined = { response: PLAN_ANSWER }): FakeApiOptions {
  return {
    session: { email: "someone@example.com" },
    healthz: { cloudTranscribe: true },
    transcribe: { response: markedAnswer() },
    ...(plan === undefined ? {} : { plan }),
  };
}
