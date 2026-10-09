// The ASR worker: loads the speech model from OPFS, turns 16 kHz audio into
// words with their times, and hands them to the text rules in Rust (TS §16).
// The model and the render session never exist at the same time (INV-12):
// `unload` answers only when the model's memory has been given back.

import { LIMITS, type Transcript } from "../gen/domain";
import { loadCore } from "../wasm/load-core";
import { loadModel, transcribe as hear, unloadModel } from "./asr/whisper-runtime";
import type { AsrWorkerApi } from "./protocol";
import { CANCELLED, type Handlers, type JobContext, serveWorker, WorkerFailure } from "./rpc";

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

const handlers: Handlers<AsrWorkerApi> = { load, transcribe, unload };

serveWorker(handlers, { stage: "asr", oneWay: [], duringPreview: [] });
