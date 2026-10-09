// C-3, C-4: from an accepted clip to one that can be previewed. The audio is
// decoded, the speech model is made ready and hears it, the visual events are
// found and the scene is built (TS §9). Every step runs in a worker; this
// file puts them in order, keeps the clip store up to date and measures the
// stages around the calls (v2implementation D-35). Nothing here calls the
// API (INV-3), and nothing of what was said leaves in an analytics event.

import { track } from "../analytics/client";
import type { DownloadOutcome } from "../gen/api";
import type { ClipId, DetectedEvent, EventKind, FailureStage, Prosody, Transcript } from "../gen/domain";
import * as modelManager from "../models/model-manager";
import * as opfs from "../persistence/opfs";
import * as clipStore from "../state/clip-store";
import { useModelStore } from "../state/model-store";
import { type Cancelled, pool, WorkerCallError } from "../workers/pool";
import type { AppFailure } from "../workers/protocol";

// The two caps of TS §22.6. The server refuses a whole batch for one value
// above them, so no measured value is sent uncapped.
const MAX_DURATION_MS = 3_600_000;
const MAX_COUNT = 10_000;

/** A time measured here, as analytics takes it: whole milliseconds, capped. */
function capMs(ms: number): number {
  return Math.min(MAX_DURATION_MS, Math.max(0, Math.round(ms)));
}

// When the import of a clip began. `probe_audio` and the total are counted
// from there (D-35), and `importClip` is what knows the moment.
let importStarted: { clipId: ClipId; at: number } | undefined;

/** Called by `importClip` as the copy of the file begins: the clip's time is counted from now. */
export function startTimer(clipId: ClipId): void {
  importStarted = { clipId, at: performance.now() };
}

/**
 * The result of a job. Nothing sends a cancel before V4, so a job that says
 * it was cancelled has gone wrong.
 */
function answered<T>(result: T | Cancelled, stage: FailureStage): T {
  if (typeof result === "object" && result !== null && "cancelled" in result) {
    throw new WorkerCallError({ code: "E_INTERNAL", stage, retryable: false, detail: "Cancelled" });
  }
  return result;
}

/** PCM that came from a worker is in a buffer of its own, which can be handed on (TS §14.3). */
function owned(pcm: Float32Array): Float32Array<ArrayBuffer> {
  const { buffer } = pcm;
  if (!(buffer instanceof ArrayBuffer)) {
    throw new Error("PCM from a worker is not in a buffer of its own");
  }
  return new Float32Array(buffer, pcm.byteOffset, pcm.length);
}

/** What was thrown, as the failure the clip store keeps; `null` for what is a fault of this file. */
function failureOf(error: unknown): AppFailure | null {
  if (error instanceof WorkerCallError) {
    return error.failure;
  }
  if (error instanceof opfs.OpfsError) {
    return { code: error.kind === "quota" ? "E_STORAGE_QUOTA" : "E_STORAGE_IO", stage: "storage", retryable: true };
  }
  return null;
}

function assertNever(value: never): never {
  void value;
  throw new Error("a kind of event this build does not know");
}

/**
 * What a line of the feed shows of an event: the event's own figure, never a
 * sentence. It stays on the device (TS §14.1). V2 finds numbers only (D-49).
 */
function displayOf(event: DetectedEvent, transcript: Transcript): string {
  const params = event.params;
  switch (params.kind) {
    case "number_reveal":
      return params.value.display;
    case "from_to":
      return params.to.display;
    case "list_reveal":
      return String(params.count);
    case "keyword_pop":
      return transcript.words[params.word]?.text ?? "";
    default:
      return assertNever(params);
  }
}

// --- The speech model (C-3) --------------------------------------------------

function outcomeOf(failure: AppFailure | null): DownloadOutcome {
  if (failure === null) {
    return "ok";
  }
  return failure.code === "E_MODEL_HASH" ? "hash_mismatch" : "failed";
}

/**
 * Makes the model ready and says how that went: `null`, or the failure. It
 * never rejects. When files had to be downloaded, `model_download` is
 * tracked once the download is over, whatever became of the clip meanwhile.
 */
async function waitForModel(): Promise<AppFailure | null> {
  const startedAt = performance.now();
  // What was on this device when the download began. `undefined`: nothing was downloaded.
  let onDisk: number | undefined;
  let failure: AppFailure | null = null;
  try {
    // The signal is never aborted in V2: the cancel arrives in V4.
    await modelManager.ensureReady((p) => {
      onDisk ??= p.done;
    }, new AbortController().signal);
  } catch (error) {
    failure =
      error instanceof modelManager.ModelFailure
        ? error.failure
        : { code: "E_INTERNAL", stage: "model", retryable: false };
  }
  if (onDisk !== undefined || failure !== null) {
    track({
      name: "model_download",
      props: {
        outcome: outcomeOf(failure),
        duration_ms: capMs(performance.now() - startedAt),
        resumed: (onDisk ?? 0) > 0,
      },
    });
  }
  return failure;
}

let modelWait: Promise<AppFailure | null> | undefined;

/** One wait for one download: a second clip that comes while it runs joins it, and it is tracked once. */
function modelReady(): Promise<AppFailure | null> {
  modelWait ??= waitForModel().finally(() => {
    modelWait = undefined;
  });
  return modelWait;
}

// --- The pipeline (C-4) ------------------------------------------------------

/** What a run has got to: read when it fails. */
type Run = { stage: FailureStage; sessionOpened: boolean };

/** Steps 2 to 12 of v2implementation §19.3. A failure is thrown; `runPipeline` stores it. */
async function steps(clipId: ClipId, startedAt: number, run: Run): Promise<void> {
  const { clipInfo, edit } = clipStore.useClipStore.getState();
  if (clipInfo === undefined) {
    throw new Error("runPipeline: the clip has no ClipInfo");
  }

  // 2. The model, beside the audio (C-3). The clip waits for it at step 5.
  const model = modelReady();
  if (useModelStore.getState().status !== "ready") {
    clipStore.processing("probe_audio", true);
  }

  // 3. (A) The audio, at both rates. The stage began with the import (D-35).
  const audio = answered(await pool.media.extractAudio({ clipId }), run.stage);
  track({ name: "stage_timing", props: { stage: "probe_audio", duration_ms: capMs(performance.now() - startedAt) } });
  const pcm16 = owned(audio.pcm16);

  // 4. V3: the voice chain's output takes the place of `pcm48` (v2implementation §25).
  //    Until then nothing is cleaned, and the feed has no line that says so (D-22).
  const out48 = owned(audio.pcm48);

  // 5. A model that cannot be had ends the clip. `pcm16` is dropped with it.
  const modelFailure = await model;
  if (modelFailure !== null) {
    clipStore.failed(modelFailure);
    return;
  }

  // 6. (B1) The model is loaded for this clip and hears it.
  run.stage = "asr";
  clipStore.processing("asr", false);
  const { modelId } = await modelManager.cacheInfo();
  const asrStartedAt = performance.now();
  let backend: "webgpu" | "wasm";
  let heard: { ok: Transcript } | { rejected: "NoSpeech" };
  try {
    ({ backend } = answered(await pool.asr.load({ modelId, backend: "webgpu" }), run.stage));
    heard = answered(
      await pool.asr.transcribe(
        { pcm16 },
        {
          transfer: [pcm16.buffer],
          onProgress: (p) => {
            // "Transcribing…" comes with the first message.
            if (p.feed !== undefined) {
              clipStore.pushFeed(p.feed);
            }
          },
        },
      ),
      run.stage,
    );
  } catch (error) {
    // Step 7 runs all the same, and the first failure is the one that is stored.
    await pool.asr.unload().catch(() => undefined);
    throw error;
  }
  track({
    name: "stage_timing",
    props: { stage: "asr", duration_ms: capMs(performance.now() - asrStartedAt), asr_backend: backend },
  });

  // 7. (C) The model's memory is given back before the render session opens (INV-12).
  answered(await pool.asr.unload(), run.stage);

  // 8. Too few words to caption: the clip cannot be used, and its file goes.
  if ("rejected" in heard) {
    clipStore.noSpeech();
    await opfs.remove(opfs.paths.clipDir(clipId)).catch(() => undefined);
    return;
  }
  const transcript = heard.ok;

  // 9. V3: `measureProsody` takes this line's place (v2implementation §25).
  //    Until then no word stands out from its sentence (D-22).
  const prosody: Prosody = { per_word: transcript.words.map(() => ({ energy_z: 0, pitch_z: 0 })) };

  // 10. (E) The events and the scene.
  run.stage = "detect_scene";
  clipStore.processing("detect_scene", false);
  const sceneStartedAt = performance.now();
  run.sessionOpened = true;
  answered(await pool.render.openSession({ clipId, clipInfo }), run.stage);
  const events = answered(await pool.render.detect({ transcript, prosody }), run.stage);
  for (const event of events) {
    clipStore.pushFeed({ kind: "event_found", eventKind: event.kind, display: displayOf(event, transcript) });
  }
  answered(await pool.render.setScene({ transcript, events, edit, profile: "preview" }), run.stage);
  track({
    name: "stage_timing",
    props: { stage: "detect_scene", duration_ms: capMs(performance.now() - sceneStartedAt) },
  });

  // 11. (F) The audio the export will read (D-51).
  run.stage = "storage";
  await opfs.writeAudio(clipId, out48);

  // 12.
  clipStore.ready({ transcript, prosody, events, out48 });
  const count = (kind: EventKind): number =>
    Math.min(MAX_COUNT, events.filter((event) => event.kind === kind).length);
  track({
    name: "pipeline_done",
    props: {
      total_ms: capMs(performance.now() - startedAt),
      n_number: count("number_reveal"),
      n_list: count("list_reveal"),
      n_from_to: count("from_to"),
      n_keyword: count("keyword_pop"),
    },
  });
}

/**
 * Takes the accepted clip to `ready`, or to `failed` or `rejected`. A failure
 * of a worker, of the model or of the storage is stored on the clip; no
 * `client_error` is sent for it before V4.
 */
export async function runPipeline(clipId: ClipId): Promise<void> {
  if (clipStore.useClipStore.getState().clipId !== clipId) {
    throw new Error("runPipeline: this is not the clip in work");
  }
  // A clip that did not come through `importClip` is timed from here.
  const startedAt = importStarted?.clipId === clipId ? importStarted.at : performance.now();

  // 1. accepted → processing. The store refuses the step from any other state (INV-16).
  clipStore.processing("probe_audio", false);
  if (clipStore.useClipStore.getState().status !== "processing") {
    return;
  }

  const run: Run = { stage: "probe_audio", sessionOpened: false };
  try {
    await steps(clipId, startedAt, run);
  } catch (error) {
    // The model was unloaded where it failed; the render session is closed here.
    if (run.sessionOpened) {
      await pool.render.closeSession().catch(() => undefined);
    }
    const failure = failureOf(error);
    clipStore.failed(failure ?? { code: "E_INTERNAL", stage: run.stage, retryable: false });
    if (failure === null) {
      // A fault of this file. The clip is shown as failed, and the fault is not hidden behind that.
      throw error;
    }
  }
}
