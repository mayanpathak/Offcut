// The audio worker: the voice chain, the prosody measure and the program's
// audio, off the main thread (TS §18, §40.6; v3implementation §14.1). It
// holds the cleaned voice between `runChain` and `measureProsody` (TS §18.4)
// and nothing else; `release` drops it (D-128). It decides nothing about the
// audio: no silence, no pause, no cut (TS §36); the chain changes no length
// (`out48.length === pcm48.length`, INV-10). `assemble` applies a time map it
// is given; in V4 the only map is the identity. It touches no storage and
// sends nothing.

import type { Lufs, OutSpan, Prosody, Sentence, TimeMap, Word } from "../gen/domain";
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

/**
 * The program's audio from `out48`, which was transferred in. Nothing is
 * kept: `serveWorker` transfers `prog48` out (D-127).
 */
async function assemble(p: {
  out48: Float32Array;
  timeMap: TimeMap;
  speech: OutSpan[];
}): Promise<{ prog48: Float32Array<ArrayBuffer>; loudness: Lufs }> {
  const core = await loadCore();
  return core.assemble(p.out48, p.timeMap, p.speech);
}

/** One-way: the clip ended early, and the cleaned voice it left is let go (D-128). */
function release(): void {
  kept = null;
}

const handlers: Handlers<AudioWorkerApi> = { runChain, measureProsody, assemble, release };

serveWorker(handlers, { stage: "audio_chain", oneWay: ["release"], duringPreview: [] });
