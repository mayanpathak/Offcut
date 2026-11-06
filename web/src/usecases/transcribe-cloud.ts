// Cloud transcription of one clip (cloudimplementation §8.4, D-372): the
// only use-case that sends audio. It waits for a waking API a little, has
// `asr.worker` encode the clip's 16 kHz sound, sends it once, and has the
// worker build the transcript from the words that come back. Any failure
// is a fallback, never an error: the caller finishes the clip on the device.
// The `pcm16` it is given stays the caller's: the workers get copies.

import { track } from "../analytics/client";
import type { CloudOutcome } from "../gen/api";
import type { Transcript } from "../gen/domain";
import { transcribeAudio } from "../net/api-client";
import { settledAvailability } from "../state/cloud-store";
import { pool } from "../workers/pool";

/** How long a clip waits for an API that has not yet said whether it can transcribe (D-374). */
export const CLOUD_WAKE_WAIT_MS = 45_000;

/** Why a clip fell back to the device. */
export type CloudFallback = Exclude<CloudOutcome, "ok">;

export type CloudResult = { ok: Transcript } | { rejected: "NoSpeech" } | { fallback: CloudFallback };

const MAX_DURATION_MS = 3_600_000;

function capMs(ms: number): number {
  return Math.min(MAX_DURATION_MS, Math.max(0, Math.round(ms)));
}

function isCancelled(value: unknown): boolean {
  return typeof value === "object" && value !== null && "cancelled" in value;
}

/** What a failed request tells about why. */
function outcomeOf(failure: { failure: { code: string }; apiError?: { code: string } }): CloudFallback {
  if (failure.apiError?.code === "transcribe_unavailable") {
    return "unavailable";
  }
  if (failure.apiError?.code === "rate_limited") {
    return "limited";
  }
  if (failure.failure.code === "E_NET_OFFLINE") {
    return "offline";
  }
  if (failure.failure.code === "E_NET_TIMEOUT") {
    return "timeout";
  }
  return "failed";
}

export async function transcribeInCloud(pcm16: Float32Array<ArrayBuffer>, signal: AbortSignal): Promise<CloudResult> {
  const startedAt = performance.now();
  // A server that has not said yes is never sent anything, and is never sent
  // an analytics value it may not know (D-377).
  if ((await settledAvailability(CLOUD_WAKE_WAIT_MS)) !== "available") {
    return { fallback: "unavailable" };
  }
  const report = (outcome: CloudOutcome): void => {
    if (!signal.aborted) {
      track({ name: "asr_cloud", props: { outcome, duration_ms: capMs(performance.now() - startedAt) } });
    }
  };
  const fallBack = (why: CloudFallback): CloudResult => {
    report(why);
    return { fallback: why };
  };

  try {
    const copy = pcm16.slice();
    const encoded = await pool.asr.encodeForCloud({ pcm16: copy }, { transfer: [copy.buffer] });
    if (isCancelled(encoded) || signal.aborted) {
      return { fallback: "failed" };
    }
    if (!("ogg" in encoded)) {
      return fallBack("unsupported");
    }
    const sent = await transcribeAudio(new Uint8Array(encoded.ogg), signal);
    if (signal.aborted) {
      return { fallback: "failed" };
    }
    if (!sent.ok) {
      return fallBack(outcomeOf(sent));
    }
    const words = pcm16.slice();
    const heard = await pool.asr.fromCloud(
      { words: sent.data.words, pcm16: words, modelVersion: sent.data.model },
      { transfer: [words.buffer] },
    );
    if (isCancelled(heard) || signal.aborted || !("ok" in heard || "rejected" in heard)) {
      return { fallback: "failed" };
    }
    report("ok");
    return heard;
  } catch {
    // A worker that failed or crashed: the device path is the way on.
    return signal.aborted ? { fallback: "failed" } : fallBack("failed");
  }
}
