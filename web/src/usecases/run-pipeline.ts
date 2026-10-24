// C-3, C-4: from an accepted clip to one that can be previewed. The audio is
// decoded, the speech model is made ready and hears it, the visual events are
// found and made the rules plan, and the scene is built from the plan on the
// clip's identity time map (TS §9, §40.8). Every step runs in a worker; this
// file puts them in order, keeps the clip store up to date and measures the
// stages around the calls (v2implementation D-35). Nothing here calls the
// API (INV-3), and nothing of what was said leaves in an analytics event.

import { track } from "../analytics/client";
import type { DownloadOutcome } from "../gen/api";
import type {
  ClipId,
  DetectedEvent,
  DurMs,
  EventKind,
  FailureStage,
  TimeMap,
  TimeMs,
  Transcript,
} from "../gen/domain";
import * as modelManager from "../models/model-manager";
import * as editsRepo from "../persistence/edits-repo";
import * as opfs from "../persistence/opfs";
import * as transcriptRepo from "../persistence/transcript-repo";
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

/** The one place of this file that turns a number into a time of the recording (v2implementation D-59). */
function recordingTime(ms: number): TimeMs {
  return ms as TimeMs;
}

/**
 * The time map of a clip played as it was recorded: the whole recording,
 * once, so that every program time is its recording time (TS §40.1). V4
 * builds no other map (D-164), and builds this one here only.
 */
function identityTimeMap(duration: DurMs): TimeMap {
  return { segments: [{ kind: "source", src: { start: recordingTime(0), end: recordingTime(duration) } }] };
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

/** How a job that runs beside others ended, and when. Never a rejection (D-90). */
type Settled<T> = { ok: T; at: number } | { failure: unknown; at: number };

function settle<T>(job: Promise<T>): Promise<Settled<T>> {
  return job.then(
    (ok) => ({ ok, at: performance.now() }),
    (failure: unknown) => ({ failure, at: performance.now() }),
  );
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
 * sentence. It stays on the device (TS §14.1). A FromTo shows its first value
 * here and its second in `displayToOf` (D-83).
 */
function displayOf(event: DetectedEvent, transcript: Transcript): string {
  const params = event.params;
  switch (params.kind) {
    case "number_reveal":
      return params.value.display;
    case "from_to":
      return params.from.display;
    case "list_reveal":
      return String(params.count);
    case "keyword_pop":
      return transcript.words[params.word]?.text ?? "";
    default:
      return assertNever(params);
  }
}

/** The second value of a FromTo's line (D-83); the other kinds have none. */
function displayToOf(event: DetectedEvent): string | undefined {
  return event.params.kind === "from_to" ? event.params.to.display : undefined;
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

/** What a run has got to: read when it fails. `audioStarted`: the audio worker may hold the cleaned voice. */
type Run = { stage: FailureStage; sessionOpened: boolean; audioStarted: boolean };

/** The audio worker lets go of what it holds of this clip (D-128). One-way. */
function releaseAudio(run: Run): void {
  if (run.audioStarted) {
    pool.audio.notify("release", {});
  }
}

/** Steps 2 to 14 of v3implementation §15.2. A failure is thrown; `runPipeline` stores it. */
async function steps(clipId: ClipId, startedAt: number, run: Run): Promise<void> {
  const { clipInfo, edit } = clipStore.useClipStore.getState();
  if (clipInfo === undefined) {
    throw new Error("runPipeline: the clip has no ClipInfo");
  }

  // 2. The model, beside the audio (C-3). The clip waits for it at step 6.
  const model = modelReady();
  if (useModelStore.getState().status !== "ready") {
    clipStore.processing("probe_audio", true);
  }

  // 3. (A) The audio, at both rates. The stage began with the import (D-35).
  const audio = answered(await pool.media.extractAudio({ clipId }), run.stage);
  track({ name: "stage_timing", props: { stage: "probe_audio", duration_ms: capMs(performance.now() - startedAt) } });
  const pcm16 = owned(audio.pcm16);
  const pcm48 = owned(audio.pcm48);

  // 4, 5. (B2) The voice chain runs beside the model wait and ASR, and owns
  //    `pcm48` from here (D-90). Every path below waits for it.
  clipStore.pushFeed({ kind: "cleaning_voice" });
  const chainAt = performance.now();
  run.audioStarted = true;
  const chain = settle(
    pool.audio.runChain({ pcm48 }, { transfer: [pcm48.buffer] }).then((r) => answered(r, "audio_chain")),
  ).then((c) => {
    if ("ok" in c) {
      track({ name: "stage_timing", props: { stage: "audio_chain", duration_ms: capMs(c.at - chainAt) } });
    }
    return c;
  });

  // 6. A model that cannot be had ends the clip, once the chain has settled.
  const modelFailure = await model;
  if (modelFailure !== null) {
    await chain;
    releaseAudio(run);
    clipStore.failed(modelFailure);
    return;
  }

  // 7, 8. (B1, C) The model is loaded for this clip, hears it, and gives its
  //    memory back before the render session opens (INV-12).
  run.stage = "asr";
  clipStore.processing("asr", false);
  let heard: { ok: Transcript } | { rejected: "NoSpeech" };
  try {
    const { modelId } = await modelManager.cacheInfo();
    const asrStartedAt = performance.now();
    const { backend } = answered(await pool.asr.load({ modelId, backend: "webgpu" }), run.stage);
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
    track({
      name: "stage_timing",
      props: { stage: "asr", duration_ms: capMs(performance.now() - asrStartedAt), asr_backend: backend },
    });
    answered(await pool.asr.unload(), run.stage);
  } catch (error) {
    // The chain is joined and the model unloaded; the earlier failure is stored (D-90).
    const failedAt = performance.now();
    const c = await chain;
    await pool.asr.unload().catch(() => undefined);
    throw "failure" in c && c.at < failedAt && failureOf(error) !== null ? c.failure : error;
  }

  // 9. Too few words to caption: once the chain has settled, the clip cannot
  //    be used, and its file goes. Its record stays until the next import.
  if ("rejected" in heard) {
    await chain;
    releaseAudio(run);
    clipStore.noSpeech();
    track({ name: "clip_rejected", props: { reject_reason: "REJECT_NO_SPEECH" } });
    await opfs.remove(opfs.paths.clipDir(clipId)).catch(() => undefined);
    return;
  }
  const transcript = heard.ok;

  // 10. The chain's failure, if it had one.
  run.stage = "audio_chain";
  const c = await chain;
  if ("failure" in c) {
    throw c.failure;
  }

  // 11. (D) The prosody, on the cleaned voice; the cleaned voice comes back.
  const measured = answered(
    await pool.audio.measureProsody({ words: transcript.words, sentences: transcript.sentences }),
    run.stage,
  );
  const { prosody } = measured;
  const out48 = owned(measured.out48);

  // 12. (E) The events, their plan, and the scene built from it (D-124).
  run.stage = "detect_scene";
  clipStore.processing("detect_scene", false);
  const sceneStartedAt = performance.now();
  run.sessionOpened = true;
  answered(await pool.render.openSession({ clipId, clipInfo }), run.stage);
  const events = answered(await pool.render.detect({ transcript, prosody }), run.stage);
  for (const event of events) {
    const displayTo = displayToOf(event);
    clipStore.pushFeed({
      kind: "event_found",
      eventKind: event.kind,
      display: displayOf(event, transcript),
      ...(displayTo !== undefined && { displayTo }),
    });
  }
  const plan = answered(await pool.render.rulesPlan({ transcript, events }), run.stage);
  clipStore.setPlan(plan);
  const timeMap = identityTimeMap(clipInfo.duration);
  const report = answered(await pool.render.setScene({ transcript, plan, edit, timeMap, profile: "preview" }), run.stage);
  clipStore.setChunks(report.chunks);
  track({
    name: "stage_timing",
    props: { stage: "detect_scene", duration_ms: capMs(performance.now() - sceneStartedAt) },
  });

  // 13. (F) The audio the export will read (D-51, D-129), before it is
  //     handed on; then the program's audio from it, on the identity map
  //     (D-127); then the clip's records.
  run.stage = "storage";
  await opfs.writeAudio(clipId, out48);
  run.stage = "audio_chain";
  const program = answered(
    await pool.audio.assemble({ out48, timeMap, speech: [] }, { transfer: [out48.buffer] }),
    run.stage,
  );
  const prog48 = owned(program.prog48);
  run.stage = "storage";
  await transcriptRepo.put(clipId, { transcript, prosody });
  await editsRepo.put(clipId, { editState: edit });

  // 14.
  clipStore.ready({ transcript, prosody, events, timeMap, prog48 });
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

  const run: Run = { stage: "probe_audio", sessionOpened: false, audioStarted: false };
  try {
    await steps(clipId, startedAt, run);
  } catch (error) {
    // Every path that throws has let the chain settle (D-90): the audio is let
    // go. The model was unloaded where it failed; the render session is closed here.
    releaseAudio(run);
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
