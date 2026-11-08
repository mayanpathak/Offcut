// The long job (v6.1implementation §9.5, D-403, D-418, TS §43.7-§43.8): the
// sound written as `pcm16.f32` while the file is in hand, then the cloud
// windows, then the master transcript. Each stage records how far it got, so a
// reload continues: transcription without the file, extraction from its last
// checkpoint once the same file is dropped again. The wake lock and the tab's
// title follow the work. A cancel removes the source (D-415).

import type { SourceInfo } from "../gen/domain";
import { holdAwake, showProgressInTitle, type WakeHold } from "../platform/wake-lock";
import type { SourceRecord } from "../persistence/schema";
import * as sources from "../persistence/sources-repo";
import { type PauseReason, useSourceStore } from "../state/source-store";
import { pool, WorkerCallError } from "../workers/pool";
import { transcribeSourceWindows, windowCount } from "./transcribe-source";

/** A checkpoint is whole seconds of written sound, in steps of 10 (D-403). */
const CHECKPOINT_SECONDS = 10;

let controller: AbortController | undefined;
/** The dropped file while this page lives: Continue after a pause needs it. Memory only (P-11). */
let inHand: File | undefined;

export function fileInHand(): File | undefined {
  return inHand;
}
let wake: WakeHold | undefined;
let pageTitle: string | undefined;

function store() {
  return useSourceStore.getState();
}

function title(percent: number | null): void {
  pageTitle ??= document.title;
  showProgressInTitle(percent, pageTitle);
}

/** Lets go of the wake lock and the title: on pause, cancel, failure and the end (D-418). */
function settle(): void {
  wake?.release();
  wake = undefined;
  title(null);
}

function pause(reason: PauseReason, from: "extracting_audio" | "transcribing"): void {
  settle();
  store().patch({ paused: { reason, from } });
  store().send("pause");
}

function fail(code: string, detail?: string): void {
  settle();
  store().patch({ failure: detail === undefined ? { code } : { code, detail } });
  store().send("fail");
}

/** The sound, from the checkpoint on. Returns the record as written, or `undefined` when the job stopped. */
async function extract(record: SourceRecord, file: File, signal: AbortSignal): Promise<SourceRecord | undefined> {
  const info: SourceInfo = record.sourceInfo;
  const startedAt = Date.now();
  let checkpoint = record.pcm16Seconds;
  let result: Awaited<ReturnType<typeof pool.media.extractSourcePcm16>>;
  try {
    result = await pool.media.extractSourcePcm16(
      { sourceId: record.sourceId, file, info, fromSeconds: record.pcm16Seconds },
      {
        onProgress: (p) => {
          const doneMs = Math.min(p.done, info.duration);
          store().patch({ progress: { stage: "audio", done: doneMs, total: info.duration, startedAt } });
          title((doneMs / info.duration) * 50);
          // What the worker has flushed, in whole steps of 10 s (D-403).
          const seconds = Math.floor(doneMs / 1000 / CHECKPOINT_SECONDS) * CHECKPOINT_SECONDS;
          if (seconds > checkpoint) {
            checkpoint = seconds;
            void sources.update(record.sourceId, { pcm16Seconds: seconds }).catch(() => undefined);
          }
        },
      },
    );
  } catch (error) {
    if (signal.aborted) {
      return undefined;
    }
    const failure = error instanceof WorkerCallError ? error.failure : undefined;
    fail(failure?.code ?? "E_INTERNAL", failure?.detail);
    return undefined;
  }
  if (signal.aborted || typeof result !== "object" || result === null || "cancelled" in result) {
    return undefined;
  }
  if ("failed" in result) {
    // A file that moved or changed asks for the file again (§9.5); a sound that cannot be decoded fails.
    fail(result.failed, result.detail);
    return undefined;
  }
  const written = await sources.update(record.sourceId, {
    pcm16Seconds: Math.floor(info.duration / 1000),
    levelP95: result.levelP95,
    stage: "transcribing",
    windowsTotal: windowCount(info.duration),
  });
  return written;
}

/** The windows, then the master transcript. */
async function transcribe(record: SourceRecord, signal: AbortSignal): Promise<void> {
  const total = windowCount(record.sourceInfo.duration);
  const startedAt = Date.now();
  store().patch({ progress: { stage: "transcribe", done: record.windowsDone, total, startedAt } });
  const result = await transcribeSourceWindows({
    sourceId: record.sourceId,
    durationMs: record.sourceInfo.duration,
    levelP95: record.levelP95 ?? Number.NEGATIVE_INFINITY,
    signal,
    onWindow: (done, all) => {
      store().patch({ progress: { stage: "transcribe", done, total: all, startedAt } });
      title(50 + (done / all) * 50);
      void sources.update(record.sourceId, { windowsDone: done, windowsTotal: all }).catch(() => undefined);
    },
  }).catch(() => ({ failed: "error" as const }));
  if (signal.aborted || "cancelled" in result) {
    return;
  }
  if ("paused" in result) {
    pause(result.paused, "transcribing");
    return;
  }
  if ("rejected" in result) {
    settle();
    store().patch({ rejected: "REJECT_NO_SPEECH" });
    fail("REJECT_NO_SPEECH");
    return;
  }
  if ("failed" in result) {
    fail(result.failed === "encoder" ? "E_SOURCE_AUDIO" : "E_INTERNAL");
    return;
  }
  await sources.update(record.sourceId, { stage: "transcribed" }).catch(() => undefined);
  settle();
  store().patch({ transcript: result.transcript, progress: null });
  store().send("transcribed");
}

/**
 * Runs the job from the stage the store is in: `extracting_audio` (with the
 * file) or `transcribing` (without it). Resolves when the job ends, pauses,
 * fails or is cancelled; the store says which.
 */
export async function runSourcePipeline(record: SourceRecord, file: File | undefined): Promise<void> {
  inHand = file ?? inHand;
  controller = new AbortController();
  const signal = controller.signal;
  wake?.release();
  wake = holdAwake(() => {
    const status = store().status;
    if (status === "extracting_audio" || status === "transcribing") {
      controller?.abort();
      void pool.cancel("media");
      pause("wake_lost", status);
    }
  });
  let current: SourceRecord | undefined = record;
  if (store().status === "extracting_audio") {
    file ??= inHand;
    if (file === undefined) {
      fail("E_SOURCE_READ", "NoFile");
      return;
    }
    current = await extract(record, file, signal);
    if (current === undefined) {
      if (!signal.aborted && store().status === "extracting_audio") {
        settle();
      }
      return;
    }
    store().send("audio_done");
  }
  if (store().status === "transcribing") {
    await transcribe(current, signal);
  }
}

/** Stops the job and removes the source: nothing of it is left (D-415). */
export async function cancelSource(): Promise<void> {
  const { status, sourceId } = store();
  if (status !== "extracting_audio" && status !== "transcribing" && status !== "paused" && status !== "awaiting_file" && status !== "failed") {
    return;
  }
  controller?.abort();
  settle();
  store().send("cancel");
  if (status === "extracting_audio") {
    await pool.cancel("media").catch(() => undefined);
  }
  inHand = undefined;
  if (sourceId !== null) {
    await sources.removeSource(sourceId).catch(() => undefined);
  }
}
