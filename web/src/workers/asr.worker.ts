// The ASR worker: loads the speech model from OPFS, turns 16 kHz audio into
// words with their times, and hands them to the text rules in Rust (TS §16).
// The model and the render session never exist at the same time (INV-12):
// `unload` answers only when the model's memory has been given back.

import type { CloudWord } from "../gen/api";
import { type Confidence, type DurMs, LIMITS, type TimeMs, type Transcript } from "../gen/domain";
import { loadCore } from "../wasm/load-core";
import { dropSilentWords, encodeOpus } from "./asr/cloud-audio";
import { loadModel, type RawWord, transcribe as hear, unloadModel } from "./asr/whisper-runtime";
import { postProcess } from "./asr/word-timestamps";
import type { AsrWorkerApi } from "./protocol";
import { CANCELLED, type Handlers, type JobContext, serveWorker, WorkerFailure } from "./rpc-worker";

// The model that is loaded. It is also the transcript's `model_version` (D-50).
let loaded: string | undefined;

const OUT_OF_MEMORY = /out of memory|allocation failed|failed to allocate|bad_alloc/i;

/** What a failure of the runtime becomes. The detail is a fixed word, never the runtime's text. */
function failure(error: unknown, detail: string): WorkerFailure {
  if (error instanceof WorkerFailure) {
    return error;
  }
  const noMemory = error instanceof RangeError || (error instanceof Error && OUT_OF_MEMORY.test(error.message));
  return new WorkerFailure(noMemory ? "E_ASR_OOM" : "E_ASR_RUNTIME", detail);
}

async function load(p: { modelId: string; backend: "webgpu" | "wasm" }): Promise<{ backend: "webgpu" | "wasm" }> {
  loaded = undefined;
  try {
    const backend = await loadModel(p.modelId, p.backend);
    loaded = p.modelId;
    return { backend };
  } catch (error) {
    throw failure(error, "Load");
  }
}

async function transcribe(
  p: { pcm16: Float32Array },
  ctx: JobContext,
): Promise<{ ok: Transcript } | { rejected: "NoSpeech" } | typeof CANCELLED> {
  if (loaded === undefined) {
    throw new WorkerFailure("E_INTERNAL", "NoModel");
  }
  let raw: Awaited<ReturnType<typeof hear>>;
  try {
    raw = await hear(
      p.pcm16,
      (done, total) => {
        // The feed line goes out once, as the first window starts.
        ctx.progress({ stage: "asr", done, total, ...(done === 0 ? { feed: { kind: "transcribing" } } : {}) });
      },
      () => ctx.isCancelled(),
    );
  } catch (error) {
    throw failure(error, "Transcribe");
  }
  if (raw === CANCELLED) {
    return CANCELLED;
  }
  const transcript = (await loadCore()).normalizeTranscript(raw, loaded);
  return transcript.words.length < LIMITS.MIN_WORDS ? { rejected: "NoSpeech" } : { ok: transcript };
}

async function unload(): Promise<void> {
  loaded = undefined;
  try {
    await unloadModel();
  } catch (error) {
    throw failure(error, "Unload");
  }
}

// --- Cloud transcription (cloudimplementation §8.3) -----------------------------
// Neither handler touches the model session: INV-12 holds as before.

async function encodeForCloud(p: { pcm16: Float32Array }): Promise<{ ogg: Uint8Array; durationMs: number } | { unsupported: true }> {
  try {
    return await encodeOpus(p.pcm16);
  } catch {
    // An encoder that fails is one this clip cannot use: the device path takes it.
    return { unsupported: true };
  }
}

/** The 16 kHz samples in one millisecond. */
const SAMPLES_PER_MS = 16;

async function fromCloud(p: {
  words: CloudWord[];
  pcm16: Float32Array;
  modelVersion: string;
}): Promise<{ ok: Transcript } | { rejected: "NoSpeech" }> {
  const spoken = dropSilentWords(p.words, p.pcm16);
  // As the device path does: confidence 1, the words placed by `postProcess` (D-371).
  const raw: RawWord[] = spoken.map((w) => ({
    text: w.text,
    startMs: w.start_ms as TimeMs,
    endMs: w.end_ms as TimeMs,
    confidence: 1 as Confidence,
  }));
  const duration = Math.round(p.pcm16.length / SAMPLES_PER_MS) as DurMs;
  const transcript = (await loadCore()).normalizeTranscript(postProcess(raw, duration), p.modelVersion);
  return transcript.words.length < LIMITS.MIN_WORDS ? { rejected: "NoSpeech" } : { ok: transcript };
}

const handlers: Handlers<AsrWorkerApi> = { load, transcribe, unload, encodeForCloud, fromCloud };

serveWorker(handlers, { stage: "asr", oneWay: [], duringPreview: [] });
