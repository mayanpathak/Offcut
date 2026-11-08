// Going on with a long source (v6.1implementation D-419, §9.4, §10.3): after a
// pause, after a failure, after a reload ("Continue your video"), and when the
// same file is dropped again for a source that was still extracting. A file is
// the same when its size and its hash prefix both match the record (D-81);
// another is refused with "This is a different file".

import { FEATURES } from "../config/features";
import * as clipsRepo from "../persistence/clips-repo";
import type { SourceRecord } from "../persistence/schema";
import * as sources from "../persistence/sources-repo";
import { forSource } from "../state/blockers";
import { useSourceStore } from "../state/source-store";
import { fileInHand, runSourcePipeline } from "./run-source-pipeline";

function store() {
  return useSourceStore.getState();
}

async function record(): Promise<SourceRecord | undefined> {
  const id = store().sourceId;
  return id === null ? undefined : sources.get(id).catch(() => undefined);
}

/**
 * At start (D-419): the newest source a reload left. One still working is
 * offered as "Continue your video"; a transcribed one shows its transcript.
 * Never called with `long_video` off, so production never opens the source stores.
 */
export async function restoreSource(): Promise<void> {
  if (!FEATURES.long_video || store().status !== "idle") {
    return;
  }
  const newest = await sources.newest().catch(() => undefined);
  if (newest === undefined) {
    return;
  }
  if (newest.stage === "transcribed") {
    const transcript = await sources.getMaster(newest.sourceId).catch(() => undefined);
    if (transcript !== undefined && store().status === "idle") {
      store().patch({ sourceId: newest.sourceId, info: newest.sourceInfo, transcript });
      store().send("restore_transcribed");
    }
    return;
  }
  store().patch({ resumable: { sourceId: newest.sourceId, stage: newest.stage }, info: newest.sourceInfo });
}

/** "Continue your video": transcription goes on at once; extraction waits for the file (D-419). */
export async function resumeSource(): Promise<void> {
  const resumable = store().resumable;
  if (resumable === null || store().status !== "idle" || forSource() !== null) {
    return;
  }
  const row = await sources.get(resumable.sourceId).catch(() => undefined);
  if (row === undefined) {
    store().patch({ resumable: null });
    return;
  }
  store().patch({ sourceId: row.sourceId, info: row.sourceInfo, resumable: null });
  await sources.update(row.sourceId, { lastOpenedAt: Date.now() }).catch(() => undefined);
  if (row.stage === "transcribing") {
    store().send("restore_transcribe");
    await runSourcePipeline(row, undefined);
  } else {
    store().send("restore_audio");
  }
}

/** A file dropped while the source waits for it: the same file continues; another is refused. */
export async function dropSameFile(file: File): Promise<"continued" | "different" | "ignored"> {
  if (store().status !== "awaiting_file" || forSource() !== null) {
    return "ignored";
  }
  const row = await record();
  if (row === undefined) {
    return "ignored";
  }
  let hash: string;
  try {
    hash = await clipsRepo.hashPrefix(file);
  } catch {
    store().patch({ differentFile: true });
    return "different";
  }
  if (file.size !== row.sourceBytes || hash !== row.sourceHashPrefix) {
    store().patch({ differentFile: true });
    return "different";
  }
  store().patch({ differentFile: false, failure: null });
  store().send("file_again");
  await runSourcePipeline(row, file);
  return "continued";
}

/** Continue, from a pause or a failure: back to the stage it left, the file asked for when it is gone. */
export async function continueSource(): Promise<void> {
  const state = store();
  if (forSource() !== null) {
    return;
  }
  const row = await record();
  if (row === undefined) {
    return;
  }
  if (state.status === "paused") {
    const from = state.paused?.from ?? "transcribing";
    store().patch({ paused: null });
    if (from === "extracting_audio" && fileInHand() === undefined) {
      // A pause while the sound was read, and the file is no longer in hand: ask for it.
      store().send("fail");
      store().send("retry_audio");
      return;
    }
    store().send(from === "extracting_audio" ? "continue_audio" : "continue_transcribe");
    await runSourcePipeline(row, fileInHand());
    return;
  }
  if (state.status === "failed") {
    store().patch({ failure: null });
    if (row.stage === "extracting_audio") {
      store().send("retry_audio");
      return;
    }
    store().send("retry_transcribe");
    await runSourcePipeline(row, undefined);
  }
}
