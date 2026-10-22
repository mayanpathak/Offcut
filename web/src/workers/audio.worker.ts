// The audio worker: the voice chain and the prosody measure, off the main
// thread (TS §18; v3implementation §14.1). It holds the cleaned voice between
// `runChain` and `measureProsody` (TS §18.4) and nothing else. It decides
// nothing about the audio: no silence, no pause, no cut (TS §36); it changes
// no length (`out48.length === pcm48.length`, INV-10); it touches no storage
// and sends nothing.

import type { Lufs, Prosody, Sentence, Word } from "../gen/domain";
import { loadCore } from "../wasm/load-core";
import type { AudioWorkerApi } from "./protocol";
import { CANCELLED, type Handlers, type JobContext, serveWorker, WorkerFailure } from "./rpc-worker";

// The cleaned voice of the clip in work, from `runChain` until `measureProsody` hands it over.
let kept: Float32Array<ArrayBuffer> | null = null;

async function runChain(p: { pcm48: Float32Array }, ctx: JobContext): Promise<{ loudness: Lufs } | typeof CANCELLED> {
  // A buffer left by a clip that ended before `measureProsody` is dropped (D-90).
  kept = null;
  const core = await loadCore();
  const total = p.pcm48.length;
  const result = core.runChain(p.pcm48, (done) => {
    ctx.progress({ stage: "audio_chain", done, total });
    return ctx.isCancelled();
  });
  if ("cancelled" in result) {
    return CANCELLED;
  }
  kept = result.out48;
  return { loudness: result.loudness };
}

async function measureProsody(p: {
  words: Word[];
  sentences: Sentence[];
}): Promise<{ out48: Float32Array<ArrayBuffer>; prosody: Prosody }> {
  if (kept === null) {
    throw new WorkerFailure("E_INTERNAL", "NoAudio");
  }
  const core = await loadCore();
  const prosody = core.measureProsody(kept, p.words, p.sentences);
  const out48 = kept;
  kept = null;
  // `serveWorker` transfers `out48`: the worker keeps no audio after this (TS §18.4).
  return { out48, prosody };
}

const handlers: Handlers<AudioWorkerApi> = { runChain, measureProsody };

serveWorker(handlers, { stage: "audio_chain", oneWay: [], duringPreview: [] });
