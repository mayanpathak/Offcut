// V6.2: asking the API for one clip's AI plan (v6.2implementation §8.1,
// D-466, D-494). The first use-case that sends transcript text (P-1, P-14):
// the words as heard, their timings and how they were said, and nothing the
// person typed. Every failure is the rules plan with its reason, never an
// error: the plan never blocks a clip (INV-24). The plan that comes back is
// not yet validated; the render worker does that, as for the rules plan (P193).

import { FEATURES } from "../config/features";
import type { ApiErrorCode, PlanRequest } from "../gen/api";
import { CATALOG_VERSION, type DurMs, type EditPlan, type ErrorCode, type Prosody, type Transcript } from "../gen/domain";
import { PLAN_CLIENT_TIMEOUT_MS, requestPlan } from "../net/api-client";
import { useAuthStore } from "../state/auth-store";

/** Why a clip uses the rules plan (D-466; the outcomes of `plan_result`, D-490). */
export type PlanFallback = "off" | "signed_out" | "offline" | "quota" | "unavailable" | "invalid" | "timeout" | "catalog";

export type PlanChoice =
  | { source: "model"; plan: EditPlan; plansRemaining: number | null }
  | { source: "rules"; why: PlanFallback };

export type PlanInput = {
  /** The transcript as heard: `transcriptRepo`'s, never one with the person's word edits applied. */
  transcript: Transcript;
  prosody: Prosody;
  /** The clip's length. */
  duration: DurMs;
  /** The settings switch (P193). With `ai_plan` off it is not read. */
  switchOn: boolean;
  signal: AbortSignal;
};

/**
 * The body of `POST /plan` (TS §40.7): each word's heard text, timings and
 * prosody, and each sentence's words. Nothing else of the clip or the person.
 */
export function planRequestOf(transcript: Transcript, prosody: Prosody, duration: DurMs): PlanRequest {
  return {
    catalog: CATALOG_VERSION,
    duration,
    words: transcript.words.map((word, i) => ({
      text: word.text,
      start: word.start_ms,
      end: word.end_ms,
      // `measure_prosody` gives one entry per word; a missing one reads as the speaker's average.
      energy_z: prosody.per_word[i]?.energy_z ?? 0,
      pitch_z: prosody.per_word[i]?.pitch_z ?? 0,
    })),
    sentences: transcript.sentences.map((sentence) => sentence.words),
  };
}

/** The API's own answers, by their code (D-466). */
const BY_ANSWER: Partial<Record<ApiErrorCode, PlanFallback>> = {
  plan_quota: "quota",
  plan_catalog: "catalog",
  plan_invalid: "invalid",
  plan_unavailable: "unavailable",
};

/** A request with no such answer, by the client's failure. */
const BY_FAILURE: Partial<Record<ErrorCode, PlanFallback>> = {
  // A 401 after `authed`'s one refresh, or a refresh that found no session (D-466).
  E_AUTH_SESSION_EXPIRED: "signed_out",
  E_NET_OFFLINE: "offline",
};

/**
 * What a failed request tells about why. Anything else (a proxy's 502 to 504
 * without the API's body, a 400, a 429 of the rate groups) is `unavailable`:
 * no plan could be had now.
 */
function fallbackOf(failed: Extract<Awaited<ReturnType<typeof requestPlan>>, { ok: false }>): PlanFallback {
  const answered = failed.apiError === undefined ? undefined : BY_ANSWER[failed.apiError.code];
  return answered ?? BY_FAILURE[failed.failure.code] ?? "unavailable";
}

function isPlanResponse(data: unknown): data is { plan: EditPlan; plans_remaining: number | null } {
  return typeof data === "object" && data !== null && "plan" in data && typeof data.plan === "object" && data.plan !== null;
}

/**
 * The clip's AI plan, or the reason it uses the rules plan. Off, signed out
 * or offline: at once, with no request. Otherwise one `POST /plan`, given
 * `PLAN_CLIENT_TIMEOUT_MS` in all (D-498). A cancel by the caller aborts the
 * request and reads as `unavailable`; the caller, which cancelled, reads its
 * own signal and reports nothing.
 */
export async function planFor(input: PlanInput): Promise<PlanChoice> {
  if (!FEATURES.ai_plan || !input.switchOn) {
    return { source: "rules", why: "off" };
  }
  if (useAuthStore.getState().status !== "signed_in") {
    // D-494: AI plans are for signed-in people; nothing is sent.
    return { source: "rules", why: "signed_out" };
  }
  if (!navigator.onLine) {
    return { source: "rules", why: "offline" };
  }

  const stop = new AbortController();
  const signal = AbortSignal.any([stop.signal, input.signal]);
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timedOut = new Promise<"timeout">((resolve) => {
    timer = setTimeout(() => {
      resolve("timeout");
    }, PLAN_CLIENT_TIMEOUT_MS);
  });
  try {
    const body = planRequestOf(input.transcript, input.prosody, input.duration);
    const sent = await Promise.race([requestPlan(body, signal), timedOut]);
    if (sent === "timeout") {
      return { source: "rules", why: "timeout" };
    }
    if (!sent.ok) {
      return { source: "rules", why: fallbackOf(sent) };
    }
    if (!isPlanResponse(sent.data)) {
      return { source: "rules", why: "invalid" };
    }
    return { source: "model", plan: sent.data.plan, plansRemaining: sent.data.plans_remaining };
  } catch {
    // Nothing of a failed plan reaches the clip (INV-24).
    return { source: "rules", why: "unavailable" };
  } finally {
    clearTimeout(timer);
    // The request, if it is still out, is not wanted any more.
    stop.abort();
  }
}
