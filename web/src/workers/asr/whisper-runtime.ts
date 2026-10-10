// The speech recognizer. This is the only file that knows which runtime runs
// it (TDR-3): no other file may import the runtime's package.
//
// The runtime is set up so that it can reach nothing but this device: its
// model files come from OPFS through `model-cache-adapter.ts`, its own code
// from `/ort/<version>/` on the app's origin, and any request it would make
// besides is refused here before it is made (TS §16.4, §24.1).

import { type AutomaticSpeechRecognitionPipeline, env, pipeline } from "@huggingface/transformers";

import type { Confidence, DurMs, TimeMs } from "../../gen/domain";
import { CANCELLED } from "../rpc";
import { createModelCache } from "./model-cache-adapter";
import { type AsrWindow, mergeWindows, postProcess } from "./word-timestamps";

export type RawWord = { text: string; startMs: TimeMs; endMs: TimeMs; confidence: Confidence };

/** The rate the recognizer hears at, and the rate of `pcm16`. */
const ASR_SAMPLE_RATE = 16_000;
/** The recognizer hears 30 s at a time; two windows share 5 s. */
const ASR_WINDOW_S = 30;
const ASR_OVERLAP_S = 5;
const MS_PER_SECOND = 1_000;

// How each half of the model is stored: the encoder with 4-bit weights, the
// decoder with 4-bit weights and 16-bit floats. The runtime turns each into
// the name of a file (`encoder_model_q4.onnx`,
// `decoder_model_merged_q4f16.onnx`), which is the name the file has in the
// manifest and in OPFS (D-62).
const MODEL_DTYPE = { encoder_model: "q4", decoder_model_merged: "q4f16" } as const;

/** The build of ONNX Runtime the WebGPU entry of the runtime loads: its loader and its module. */
const ORT_BUILD = "ort-wasm-simd-threaded.asyncify";
const MIN_THREADS = 1;
const MAX_THREADS = 4;
/** Two cores are left to the page and the browser. */
const CORES_KEPT_FREE = 2;

let session: AutomaticSpeechRecognitionPipeline | undefined;

/** Any request the runtime tries to make is refused: a file it lacks is a failure, never a download. */
function refuse(): Promise<never> {
  return Promise.reject(new Error("the recognizer may not make a request"));
}

/** Turns off every way the runtime has to reach the network, and points it at this device. */
function configure(modelId: string): void {
  env.allowRemoteModels = false;
  // The runtime will not start with remote and local loading both off. A
  // "local" file is asked of the cache first, and the cache is OPFS; what
  // the cache does not have would be fetched from this origin, and is refused.
  env.allowLocalModels = true;
  env.fetch = refuse;
  env.useBrowserCache = false;
  env.useFSCache = false;
  env.useWasmCache = false;
  env.useCustomCache = true;
  env.customCache = createModelCache(modelId);

  const onnx = env.backends.onnx;
  if (onnx.wasm === undefined || onnx.versions?.web === undefined) {
    throw new Error("the runtime has no WebAssembly backend");
  }
  // The copy that `vite.config.ts` makes of the runtime's own files (D-37).
  const base = `${self.location.origin}/ort/${onnx.versions.web}/${ORT_BUILD}`;
  onnx.wasm.wasmPaths = { mjs: `${base}.mjs`, wasm: `${base}.wasm` };
  // No helper worker made from a `blob:` address, which the CSP refuses.
  onnx.wasm.proxy = false;
  // Read once, when the first session is made: it holds for both backends.
  const spare = navigator.hardwareConcurrency - CORES_KEPT_FREE;
  onnx.wasm.numThreads = Math.min(Math.max(spare, MIN_THREADS), MAX_THREADS);
}

/**
 * Whether the graphics processor can run this model. Its decoder computes in
 * 16-bit floats (`q4f16`), which a graphics processor offers as the feature
 * `shader-f16` or not at all. One without it still makes a session, and then
 * fails in the middle of the first clip: so it is asked before, not after.
 */
async function gpuRunsModel(): Promise<boolean> {
  if (!("gpu" in navigator)) {
    return false;
  }
  const adapter = await navigator.gpu.requestAdapter();
  return adapter !== null && adapter.features.has("shader-f16");
}

/**
 * Loads the model of `modelId` from OPFS. `webgpu` falls back to `wasm` when
 * the graphics processor cannot run the model, or when a session cannot be
 * made on it. Returns the backend in use.
 */
export async function loadModel(modelId: string, backend: "webgpu" | "wasm"): Promise<"webgpu" | "wasm"> {
  await unloadModel();
  configure(modelId);
  const create = (device: "webgpu" | "wasm"): Promise<AutomaticSpeechRecognitionPipeline> =>
    pipeline("automatic-speech-recognition", modelId, { device, dtype: MODEL_DTYPE });

  if (backend === "webgpu" && (await gpuRunsModel())) {
    try {
      session = await create("webgpu");
      return "webgpu";
    } catch {
      // No adapter, or one that cannot run this model: the processor can.
      session = undefined;
    }
  }
  session = await create("wasm");
  return "wasm";
}

/** One word as the runtime returns it: its text, and its start and end in seconds. */
type Heard = { text: string; start: number; end: number | null };

/** The words in what the runtime returned. The shape is the runtime's own, so it is read with care. */
function heardIn(output: unknown): Heard[] {
  const first: unknown = Array.isArray(output) ? output[0] : output;
  if (typeof first !== "object" || first === null || !("chunks" in first) || !Array.isArray(first.chunks)) {
    return [];
  }
  const heard: Heard[] = [];
  for (const chunk of first.chunks as unknown[]) {
    if (typeof chunk !== "object" || chunk === null || !("text" in chunk) || !("timestamp" in chunk)) {
      continue;
    }
    const times: unknown[] = Array.isArray(chunk.timestamp) ? chunk.timestamp : [];
    const [start, end] = times;
    if (typeof chunk.text === "string" && typeof start === "number" && Number.isFinite(start)) {
      // The last word of a window may come without an end.
      heard.push({ text: chunk.text, start, end: typeof end === "number" && Number.isFinite(end) ? end : null });
    }
  }
  return heard;
}

/** The words of one window, with times counted from the start of the window. */
async function hear(recognizer: AutomaticSpeechRecognitionPipeline, audio: Float32Array): Promise<RawWord[]> {
  const output: unknown = await recognizer(audio, { return_timestamps: "word" });
  return heardIn(output).map((word) => {
    const startMs = Math.round(word.start * MS_PER_SECOND);
    const endMs = word.end === null ? startMs : Math.round(word.end * MS_PER_SECOND);
    return { text: word.text, startMs: startMs as TimeMs, endMs: endMs as TimeMs, confidence: 1 as Confidence };
  });
}

/**
 * The words of `pcm16`, 16 kHz mono, with the time each was spoken.
 * `onWindow(done, total)` is called before the first window and after each
 * one; `isCancelled()` is read before each.
 */
export async function transcribe(
  pcm16: Float32Array,
  onWindow: (done: number, total: number) => void,
  isCancelled: () => boolean,
): Promise<RawWord[] | typeof CANCELLED> {
  if (session === undefined) {
    throw new Error("no model is loaded");
  }
  const windowSamples = ASR_WINDOW_S * ASR_SAMPLE_RATE;
  const stepSamples = (ASR_WINDOW_S - ASR_OVERLAP_S) * ASR_SAMPLE_RATE;
  const starts: number[] = [];
  for (let at = 0; at < pcm16.length; at += stepSamples) {
    starts.push(at);
    if (at + windowSamples >= pcm16.length) {
      break;
    }
  }

  const samplesPerMs = ASR_SAMPLE_RATE / MS_PER_SECOND;
  const windows: AsrWindow[] = [];
  onWindow(0, starts.length);
  for (const start of starts) {
    if (isCancelled()) {
      return CANCELLED;
    }
    const audio = pcm16.slice(start, start + windowSamples);
    windows.push({
      offsetMs: start / samplesPerMs,
      lengthMs: audio.length / samplesPerMs,
      words: await hear(session, audio),
    });
    onWindow(windows.length, starts.length);
  }
  return postProcess(mergeWindows(windows), Math.round(pcm16.length / samplesPerMs) as DurMs);
}

/** Gives the model's memory back. It resolves when the sessions are gone (INV-12). */
export async function unloadModel(): Promise<void> {
  const loaded = session;
  session = undefined;
  if (loaded !== undefined) {
    await loaded.dispose();
  }
}
