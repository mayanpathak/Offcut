// A long source's sound to its master transcript, in the cloud only
// (v6.1implementation §8.1, D-405 to D-410, TS §43.6). The second use-case that
// sends audio, beside `transcribe-cloud.ts` (P-5): each window of
// `pcm16.f32` is encoded by `asr.worker`, sent to `POST /transcribe` with
// `X-Offcut-Window`, guarded against the source's level, and kept in
// `sourceTranscripts` before the next is counted. Two are in flight, sent in
// order. A failure retries or pauses with a reason; there is no device
// fallback. When every window is kept, the windows are merged.

import type { CloudWord } from "../gen/api";
import { SOURCE_LIMITS, type Transcript } from "../gen/domain";
import { checkHealth, transcribeAudio } from "../net/api-client";
import { readPcm16Window } from "../persistence/opfs";
import type { SourceId, SourceWindowRecord } from "../persistence/schema";
import * as sources from "../persistence/sources-repo";
import { pool } from "../workers/pool";
import type { WindowWord } from "../workers/protocol";

/** Why the windows stopped (D-410); `wake_lost` is the pipeline's own (D-418). */
export type CloudPauseReason = "unavailable" | "busy" | "signed_out" | "offline";

/** Windows sent at once (D-409, confirmed in Entry 23; see open item 24). */
export const CLOUD_LONG_CONCURRENCY = 2;
/** The waits before the second and third attempt at a window (D-410). */
export const RETRY_WAITS_MS: readonly number[] = [1_000, 4_000];

const STEP_S = SOURCE_LIMITS.CLOUD_WINDOW_S - SOURCE_LIMITS.CLOUD_OVERLAP_S;

/** `max(1, ⌈(D − overlap) ÷ step⌉)` windows for `durationMs` of sound (D-405 as read). */
export function windowCount(durationMs: number): number {
  const overlapMs = SOURCE_LIMITS.CLOUD_OVERLAP_S * 1000;
  return Math.max(1, Math.ceil(Math.max(0, durationMs - overlapMs) / (STEP_S * 1000)));
}

/** Where window `k` starts on the source, in ms. */
export function windowOffsetMs(k: number): number {
  return k * STEP_S * 1000;
}

type Sent = Awaited<ReturnType<typeof transcribeAudio>>;

/** What the use-case calls; the tests hand in fakes. */
export type TranscribeSourceDeps = {
  readWindow(id: SourceId, k: number): Promise<Float32Array<ArrayBuffer>>;
  encode(pcm16: Float32Array<ArrayBuffer>): Promise<{ ogg: Uint8Array } | { unsupported: true } | { cancelled: true }>;
  send(ogg: Uint8Array<ArrayBuffer>, signal: AbortSignal, window: { index: number; total: number }): Promise<Sent>;
  windowWords(p: { words: CloudWord[]; pcm16: Float32Array<ArrayBuffer>; refLevel: number }): Promise<
    { words: WindowWord[]; lengthMs: number } | { cancelled: true }
  >;
  buildMaster(p: {
    windows: { offsetMs: number; lengthMs: number; words: WindowWord[] }[];
    durationMs: number;
    modelVersion: string;
  }): Promise<{ ok: Transcript } | { rejected: "NoSpeech" } | { cancelled: true }>;
  windows(id: SourceId): Promise<Map<number, SourceWindowRecord>>;
  putWindow(id: SourceId, k: number, record: SourceWindowRecord): Promise<void>;
  putMaster(id: SourceId, transcript: Transcript): Promise<void>;
  /** `/healthz`'s `cloud_transcribe`, or `undefined` when it did not answer. */
  cloudOn(): Promise<boolean | undefined>;
  wait(ms: number): Promise<void>;
};

const browserDeps: TranscribeSourceDeps = {
  readWindow: (id, k) => readPcm16Window(id, k),
  encode: async (pcm16) => {
    const copy = pcm16.slice();
    const encoded = await pool.asr.encodeForCloud({ pcm16: copy }, { transfer: [copy.buffer] });
    return typeof encoded === "object" && encoded !== null && "ogg" in encoded
      ? { ogg: encoded.ogg }
      : typeof encoded === "object" && encoded !== null && "unsupported" in encoded
        ? { unsupported: true }
        : { cancelled: true };
  },
  send: (ogg, signal, window) => transcribeAudio(ogg, signal, window),
  windowWords: async (p) => {
    const copy = p.pcm16.slice();
    const r = await pool.asr.cloudWindowWords({ words: p.words, pcm16: copy, refLevel: p.refLevel }, { transfer: [copy.buffer] });
    return typeof r === "object" && r !== null && "words" in r ? r : { cancelled: true };
  },
  buildMaster: async (p) => {
    const r = await pool.asr.buildMasterTranscript(p);
    return typeof r === "object" && r !== null && ("ok" in r || "rejected" in r) ? r : { cancelled: true };
  },
  windows: (id) => sources.windows(id),
  putWindow: (id, k, record) => sources.putWindow(id, k, record),
  putMaster: (id, transcript) => sources.putMaster(id, transcript),
  cloudOn: async () => {
    const health = await checkHealth();
    return health.ok ? health.data.cloud_transcribe : undefined;
  },
  wait: (ms) => new Promise((resolve) => setTimeout(resolve, ms)),
};

export type TranscribeSourceResult =
  | { done: true; transcript: Transcript }
  | { paused: CloudPauseReason }
  | { rejected: "NoSpeech" }
  | { failed: "encoder" }
  | { cancelled: true };

/** One window's outcome. */
type Outcome = "kept" | "aborted" | { pause: CloudPauseReason } | { failed: "encoder" };

/** What a failed answer means for a window (D-410). */
async function classify(sent: Extract<Sent, { ok: false }>, deps: TranscribeSourceDeps): Promise<"retry" | { pause: CloudPauseReason }> {
  const api = sent.apiError?.code;
  const code = sent.failure.code;
  if (api === "rate_limited") {
    return { pause: "busy" };
  }
  if (code.startsWith("E_AUTH")) {
    return { pause: "signed_out" };
  }
  if (api === "transcribe_unavailable") {
    // The switch, the budget and the provider all answer 503: `/healthz` tells the switch apart.
    return (await deps.cloudOn()) === false ? { pause: "unavailable" } : "retry";
  }
  return "retry";
}

/**
 * Transcribes every window `sources` does not hold yet, then builds and keeps
 * the master transcript. `onWindow(done, total)` follows each window kept.
 */
export async function transcribeSourceWindows(
  p: {
    sourceId: SourceId;
    durationMs: number;
    levelP95: number;
    signal: AbortSignal;
    onWindow: (done: number, total: number) => void;
  },
  deps: TranscribeSourceDeps = browserDeps,
): Promise<TranscribeSourceResult> {
  const total = windowCount(p.durationMs);
  const kept = await deps.windows(p.sourceId);
  const missing: number[] = [];
  for (let k = 0; k < total; k += 1) {
    if (!kept.has(k)) {
      missing.push(k);
    }
  }
  let done = total - missing.length;
  // A pause stops every lane; the request still in flight is aborted.
  const stop = new AbortController();
  const signal = AbortSignal.any([p.signal, stop.signal]);
  let stopped: { pause: CloudPauseReason } | { failed: "encoder" } | undefined;

  const one = async (k: number): Promise<Outcome> => {
    const pcm16 = await deps.readWindow(p.sourceId, k);
    const encoded = await deps.encode(pcm16);
    if ("cancelled" in encoded) {
      return "aborted";
    }
    if ("unsupported" in encoded) {
      return { failed: "encoder" };
    }
    const ogg = new Uint8Array(encoded.ogg);
    for (let attempt = 0; ; attempt += 1) {
      if (signal.aborted) {
        return "aborted";
      }
      const sent = await deps.send(ogg, signal, { index: k, total });
      if (sent.ok) {
        const words = await deps.windowWords({ words: sent.data.words, pcm16, refLevel: p.levelP95 });
        if ("cancelled" in words) {
          return "aborted";
        }
        // An answer that came in before a pause is kept (§8.1, step 3).
        await deps.putWindow(p.sourceId, k, { words: words.words, lengthMs: words.lengthMs, modelVersion: sent.data.model });
        return "kept";
      }
      if (signal.aborted) {
        return "aborted";
      }
      const next = await classify(sent, deps);
      if (next !== "retry") {
        return next;
      }
      const waitMs = RETRY_WAITS_MS[attempt];
      if (waitMs === undefined) {
        // Retries spent: offline says so; anything else is the service being busy.
        return { pause: sent.failure.code === "E_NET_OFFLINE" ? "offline" : "busy" };
      }
      await deps.wait(waitMs);
    }
  };

  const queue = [...missing];
  const lane = async (): Promise<void> => {
    for (let k = queue.shift(); k !== undefined && !signal.aborted; k = queue.shift()) {
      const outcome = await one(k);
      if (outcome === "kept") {
        done += 1;
        p.onWindow(done, total);
      } else if (outcome !== "aborted") {
        stopped ??= outcome;
        stop.abort();
      }
    }
  };
  await Promise.all(Array.from({ length: Math.min(CLOUD_LONG_CONCURRENCY, Math.max(1, queue.length)) }, lane));

  if (stopped !== undefined) {
    return "pause" in stopped ? { paused: stopped.pause } : stopped;
  }
  if (p.signal.aborted) {
    return { cancelled: true };
  }
  const all = await deps.windows(p.sourceId);
  if (all.size < total) {
    // A window was lost between the lanes and here: the caller may continue.
    return { paused: "busy" };
  }
  const windows = [...all.entries()]
    .filter(([k]) => k < total)
    .sort(([a], [b]) => a - b)
    .map(([k, w]) => ({ offsetMs: windowOffsetMs(k), lengthMs: w.lengthMs, words: w.words }));
  const modelVersion = all.get(0)?.modelVersion ?? "";
  const master = await deps.buildMaster({ windows, durationMs: p.durationMs, modelVersion });
  if ("cancelled" in master) {
    return { cancelled: true };
  }
  if ("rejected" in master) {
    return master;
  }
  await deps.putMaster(p.sourceId, master.ok);
  return { done: true, transcript: master.ok };
}
