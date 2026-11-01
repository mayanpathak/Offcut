// C-7, C-9, C-10: the clip becomes an MP4, and the MP4 is downloaded. The render
// worker draws, encodes and writes the file; its size and its watermark come
// from the entitlement token, which the worker verifies (INV-9). This file
// starts the export, follows it in the export store and hands the file over,
// again when asked. The same edit exported again comes from the render cache
// (TS §40.9 S-9). An export can be cancelled (v4implementation D-152): it ends
// `idle`, with no file of it on this device.
//
// Until V6 the token is the one this device holds, and without one no export
// can be made (v2implementation D-23). Nothing here calls the API.

import { track } from "../analytics/client";
import type { ExportedAs } from "../gen/api";
import type { ChangeSummary, ClipId, DetectedEvent, ExportId, ExportProfile, ProfileKind } from "../gen/domain";
import * as clipsRepo from "../persistence/clips-repo";
import * as entitlementRepo from "../persistence/entitlement-repo";
import * as opfs from "../persistence/opfs";
import * as renderCacheRepo from "../persistence/render-cache-repo";
import type { RenderCacheRecord } from "../persistence/schema";
import { type BlockerCode, forExport } from "../state/blockers";
import { type ClipState, useClipStore } from "../state/clip-store";
import * as exportStore from "../state/export-store";
import { EXPORT_MACHINE } from "../state/machines/export-machine";
import { transition } from "../state/machines/transition";
import { type Cancelled, pool, WorkerCallError, whenFree, whenReleased } from "../workers/pool";
import type { AppFailure } from "../workers/protocol";
import { signalFor } from "./cancel-job";
import { lockForExport, reloadPreview, unlockAfterExport } from "./control-preview";

// The two caps of TS §22.6. The server refuses a whole batch for one value
// above them, so no measured value is sent uncapped.
const MAX_DURATION_MS = 3_600_000;
const MAX_COUNT = 10_000;

/** How long the page keeps the address of a file it handed over (TS §31). */
const REVOKE_AFTER_MS = 60_000;

const UUID_BYTES = 16;
const TWO_POW_32 = 2 ** 32;

/** The key's `theme` until a brand kit (V6.7): a function of the style and the renderer, both in the key (D-224). */
const DEFAULT_THEME = "default";
/** Bits per second times milliseconds, to bytes. */
const BITS_MS_PER_BYTE_S = 8000;

/** A time measured here, as analytics takes it: whole milliseconds, capped. */
function capMs(ms: number): number {
  return Math.min(MAX_DURATION_MS, Math.max(0, Math.round(ms)));
}

/**
 * A new export's id: a UUID of version 7 (v2implementation D-43). 48 bits of
 * the time in milliseconds, the version, 12 random bits, the variant, 62
 * random bits. The one place of this file that makes an id (D-59).
 */
export function newExportId(): ExportId {
  const bytes = crypto.getRandomValues(new Uint8Array(UUID_BYTES));
  const view = new DataView(bytes.buffer);
  const now = Date.now();
  view.setUint16(0, Math.floor(now / TWO_POW_32));
  view.setUint32(2, now % TWO_POW_32);
  view.setUint8(6, 0x70 | (view.getUint8(6) & 0x0f));
  view.setUint8(8, 0x80 | (view.getUint8(8) & 0x3f));
  const hex = Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}` as ExportId;
}

/**
 * The plan a token names, read without verifying it (D-52). It feeds the two
 * analytics properties and nothing else: what the export is allowed comes
 * from the worker's verification alone. A token that cannot be read counts
 * as free.
 */
function planOf(token: string): ExportedAs {
  try {
    const payload = (token.split(".")[0] ?? "").replaceAll("-", "+").replaceAll("_", "/");
    const claims: unknown = JSON.parse(atob(payload));
    const creator = typeof claims === "object" && claims !== null && "plan" in claims && claims.plan === "creator";
    return creator ? "creator" : "free";
  } catch {
    return "free";
  }
}

/**
 * The plan of the token this device holds, read as `planOf` reads it; `free`
 * without one, or when it cannot be read. The preview's watermark badge
 * shows for `free` (v4implementation D-172): what an export carries is still
 * the worker's verification alone.
 */
export async function cachedPlan(): Promise<ExportedAs> {
  try {
    const record = await entitlementRepo.get();
    return record === undefined ? "free" : planOf(record.token);
  } catch {
    // A device whose storage cannot be read holds no token the badge can trust.
    return "free";
  }
}

/** `offcut-<yyyymmdd-hhmm>.mp4`, in local time. Never derived from the source (TS §21.4). */
function downloadName(at: Date): string {
  const two = (n: number): string => String(n).padStart(2, "0");
  const day = `${String(at.getFullYear())}${two(at.getMonth() + 1)}${two(at.getDate())}`;
  return `offcut-${day}-${two(at.getHours())}${two(at.getMinutes())}.mp4`;
}

/** Hands the file to the browser as a download (C-10). */
function deliver(file: File): void {
  const url = URL.createObjectURL(file);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = downloadName(new Date());
  document.body.append(anchor);
  anchor.click();
  anchor.remove();
  setTimeout(() => {
    URL.revokeObjectURL(url);
  }, REVOKE_AFTER_MS);
}

/** What was thrown, as the failure the export store keeps; `null` for what is a fault of this file. */
function failureOf(error: unknown): AppFailure | null {
  if (error instanceof WorkerCallError) {
    return error.failure;
  }
  // The audio could not be read, or the finished file could not be opened.
  if (error instanceof opfs.OpfsError) {
    return { code: "E_STORAGE_IO", stage: "storage", retryable: true };
  }
  return null;
}

/** What an export throws when it sees its signal aborted: it is not a failure. */
class ExportCancelled extends Error {
  constructor() {
    super("The export was cancelled.");
    this.name = "ExportCancelled";
  }
}

/** Stops the export here if it was cancelled: read after every step that waited. */
function goOn(signal: AbortSignal): void {
  if (signal.aborted) {
    throw new ExportCancelled();
  }
}

/** The file the last export delivered, for "Download again" (D-189). One export is kept (D-40). */
let delivered: { clipId: ClipId; path: string } | undefined;

/** A worker's answer to a call nothing cancels: one that says it was cancelled has gone wrong. */
function answered<T>(result: T | Cancelled): T {
  if (typeof result === "object" && result !== null && "cancelled" in result) {
    throw new WorkerCallError({ code: "E_INTERNAL", stage: "render_encode", retryable: false, detail: "Cancelled" });
  }
  return result;
}

/**
 * The key of the clip as it stands, for the profile the worker verified; `null`
 * when it cannot be made, and then the export goes on with no cache. The
 * source's hash is in the clip's record, not the clip store. Nothing the page
 * read from the token itself reaches this (v5implementation D-220, INV-9).
 */
async function keyOf(clipId: ClipId, profileKind: ProfileKind): Promise<string | null> {
  const record = await clipsRepo.get(clipId).catch(() => undefined);
  const { status, transcript, events, edit, plan, timeMap } = useClipStore.getState();
  // A change that began before the preview locked is not yet in the worker's scene.
  if (record === undefined || status !== "ready" || transcript === undefined || plan === undefined || timeMap === undefined) {
    return null;
  }
  return renderCacheRepo.cacheKey({
    sourceHashPrefix: record.sourceHashPrefix,
    transcript,
    events,
    edit,
    plan,
    timeMap,
    theme: DEFAULT_THEME,
    musicId: null,
    profileKind,
    rendererVersion: renderCacheRepo.RENDERER_VERSION,
  });
}

/** Whether the device has room for this export, the cached file counted as free: a miss deletes it (D-229). */
async function roomBlocker(profile: ExportProfile): Promise<BlockerCode | null> {
  const free = await opfs.freeBytes();
  if (free === null) {
    return null;
  }
  const durationMs = useClipStore.getState().clipInfo?.duration ?? 0;
  const neededBytes = ((profile.video_bitrate + profile.audio_bitrate) * durationMs) / BITS_MS_PER_BYTE_S;
  return forExport({ freeBytes: free + (await renderCacheRepo.heldBytes()), neededBytes });
}

/**
 * Steps 10 and 11 of v5implementation §13. A failure is thrown; `startExport`
 * stores it. A cancel is thrown as `ExportCancelled`: the worker stopped,
 * or its file was finished and is not delivered.
 */
async function renderAndDeliver(
  clipId: ClipId,
  exportId: ExportId,
  entitlementToken: string,
  key: string | null,
  signal: AbortSignal,
): Promise<ChangeSummary> {
  // V7: `quota.ensureFree` takes this sweep's place (v2implementation §25).
  // Until then one export is kept (D-40): the row goes, then the file (TS
  // §23.3's write order). A failure is ignored: a row without its file is a miss.
  await renderCacheRepo.clear().catch(() => undefined);
  await opfs.remove(opfs.paths.exportsRoot()).catch(() => undefined);
  delivered = undefined;

  // The export reads its own copy of the program's audio: a buffer that is
  // handed to a worker is gone from this thread, and the preview keeps its
  // own (D-51; TS §40.9 S-8).
  const prog48 = await opfs.readProgram(clipId);
  goOn(signal);

  const result = await pool.render.exportClip(
    { exportId, entitlementToken, prog48 },
    {
      transfer: [prog48.buffer],
      onProgress: (p) => {
        if (signal.aborted) {
          return;
        }
        if (p.stage !== "mux") {
          exportStore.progress(p.done, p.total);
        } else if (exportStore.useExportStore.getState().status === "rendering") {
          exportStore.encoded();
        }
      },
    },
  );
  goOn(signal);
  if ("cancelled" in result) {
    // Only this export's signal asks for a cancel: one that comes otherwise has gone wrong.
    throw new WorkerCallError({ code: "E_INTERNAL", stage: "render_encode", retryable: false, detail: "Cancelled" });
  }

  // The file is whole. The worker measured its two stages itself (D-35).
  if (exportStore.useExportStore.getState().status === "rendering") {
    exportStore.encoded();
  }
  exportStore.finalized();
  for (const timing of result.stageTimings) {
    track({ name: "stage_timing", props: { stage: timing.stage, duration_ms: capMs(timing.durationMs) } });
  }

  const file = await opfs.getFile(result.opfsPath);
  // After the file is in place (TS §23.3's write order).
  if (key !== null) {
    await renderCacheRepo.put(key, {
      clipId,
      exportId,
      opfsPath: result.opfsPath,
      bytes: clipsRepo.toBytes(file.size),
      createdAt: Date.now(),
      summary: result.summary,
    });
  }
  deliver(file);
  delivered = { clipId, path: result.opfsPath };
  return result.summary;
}

/** A hit (D-227): the stored file, under today's name and its own id. No render, no `export_started`, no `stage_timing`. */
async function deliverStored(clipId: ClipId, hit: RenderCacheRecord): Promise<void> {
  exportStore.cacheHit(hit.exportId);
  deliver(await opfs.getFile(hit.opfsPath));
  delivered = { clipId, path: hit.opfsPath };
  exportStore.saved(hit.summary, exportStore.hasAnswered(hit.exportId));
}

/**
 * rendering | muxing → cancelled → idle: the export machine's two events
 * (TS §12.2). `export-store.ts` is unchanged in this version (§4) and has no action
 * for them, so the transitions are taken here, through the same table.
 */
function exportCancelled(): void {
  const { status, unavailable } = exportStore.useExportStore.getState();
  const stopped = transition("export", EXPORT_MACHINE, status, "cancel");
  if (stopped === status) {
    return;
  }
  exportStore.useExportStore.setState({ status: stopped });
  const idle = transition("export", EXPORT_MACHINE, stopped, "cancel_done");
  exportStore.useExportStore.setState({ status: idle, done: 0, total: 0, unavailable, answered: false }, true);
}

/**
 * Whether the render worker was ended under the export: it crashed, or the
 * cancel restarted it (Prompt 118). Its preview session went with it.
 */
function renderWorkerGone(error: unknown): boolean {
  return error instanceof WorkerCallError && error.failure.code === "E_WORKER_CRASH";
}

/**
 * The edit as analytics reports it, in counts and an enum (INV-18): read when
 * the export starts, not when it ends, as the person may keep editing. A visual
 * that was switched off is not kept (v5implementation D-237).
 */
function editValues(clip: ClipState) {
  const on = (event: DetectedEvent) => clip.edit.event_overrides[event.id] ?? event.enabled;
  const kept = clip.events.filter(on).length;
  return {
    word_edits: Math.min(MAX_COUNT, Object.keys(clip.edit.word_edits).length),
    events_kept: Math.min(MAX_COUNT, kept),
    events_disabled: Math.min(MAX_COUNT, clip.events.length - kept),
    style: clip.edit.style,
    crop_adjusted: clip.edit.crop_offset !== 0,
  };
}

/** The export is over and its file was handed over: analytics says so. */
function exportDone(values: ReturnType<typeof editValues>, profile: ExportedAs, startedAt: number, fromCache: boolean): void {
  track({
    name: "export_done",
    props: { ...values, total_ms: capMs(performance.now() - startedAt), profile, from_cache: fromCache },
  });
}

/**
 * Exports the clip and downloads the file: the twelve steps of
 * v5implementation §13. Resolves when the export is over, however it ended; a
 * failure is in the export store, and the card that shows it sends
 * `client_error` (cancel-job.ts). A cancelled export ends `idle`, with neither
 * its partial nor its finished file left (D-152). A blocked one ends `idle`
 * with its reason in the store, and nothing on the device touched (D-231).
 */
export async function startExport(clipId: ClipId): Promise<void> {
  // 1. The component shows the words of the blocker.
  if (forExport() !== null || useClipStore.getState().clipId !== clipId) {
    return;
  }

  // 2. V6: the token comes from the account. Until then it is the one this
  //    device holds, and tests are what put one there (D-23).
  const record = await entitlementRepo.get();
  if (record === undefined) {
    exportStore.setUnavailable(true);
    return;
  }
  // The read took a moment: a second click may have started an export meanwhile.
  if (forExport() !== null) {
    return;
  }

  // 3. The export stays in `gating` until step 9.
  const last = exportStore.useExportStore.getState().status;
  if (last === "done" || last === "failed") {
    exportStore.reset();
  }
  const exportId = newExportId();
  exportStore.start(exportId);
  const signal = signalFor("export");
  const startedAt = performance.now();
  const values = editValues(useClipStore.getState());

  // 4. The preview stops and stays locked until the export is over (TS §12.4).
  //    Before the gate: the worker takes one job at a time, and a playing
  //    preview would refuse the next call (D-221).
  await lockForExport();

  /** `export_started` was sent, or does not belong to this export: a hit has none (D-227). */
  let started = false;
  let exportedAs: ExportedAs;
  let summary: ChangeSummary;
  try {
    goOn(signal);
    // 5. The token as the worker verifies it: this, and never `planOf`, says
    //    what the device may be given (D-220, INV-9).
    const entitlementToken = record.token;
    const profile = answered(await whenFree(() => pool.render.exportProfile({ entitlementToken })));
    exportedAs = profile.kind === "creator" ? "creator" : "free";

    // 6.
    const key = await keyOf(clipId, profile.kind);
    const hit = key === null ? undefined : await renderCacheRepo.lookup(key);

    // 7.
    if (hit !== undefined) {
      started = true;
      await deliverStored(clipId, hit);
      unlockAfterExport();
      exportDone(values, exportedAs, startedAt, true);
      return;
    }

    // 8. Nothing has been touched yet.
    const blocker = await roomBlocker(profile);
    if (blocker !== null) {
      exportStore.blockedBy(blocker);
      unlockAfterExport();
      return;
    }

    // 9.
    exportStore.clear();
    track({ name: "export_started", props: { profile: exportedAs } });
    started = true;

    // 10, 11.
    summary = await renderAndDeliver(clipId, exportId, entitlementToken, key, signal);
  } catch (error) {
    const during = exportStore.useExportStore.getState().status;
    if (signal.aborted) {
      // The worker removes its partial file when it stops; one that was
      // restarted could not, and holds it a moment after. Neither file of
      // this export is kept.
      await whenReleased(() => opfs.remove(opfs.paths.exportTmp(exportId))).catch(() => undefined);
      await whenReleased(() => opfs.remove(opfs.paths.exportFinal(exportId))).catch(() => undefined);
      if (during === "gating") {
        // The table has no `gating → cancelled`.
        exportStore.blocked();
      } else {
        exportCancelled();
      }
      unlockAfterExport();
      if (renderWorkerGone(error)) {
        await reloadPreview();
      }
      return;
    }
    // A refused token is a failure of the gate: the table has no `gating →
    // failed`, and it is announced as an export that failed (D-230).
    if (during === "gating") {
      exportStore.clear();
    }
    if (!started) {
      track({ name: "export_started", props: { profile: planOf(record.token) } });
    }
    const known = failureOf(error);
    const failure = known ?? { code: "E_INTERNAL", stage: "render_encode", retryable: false };
    exportStore.fail(failure);
    track({ name: "export_failed", props: { error_code: failure.code, stage: failure.stage } });
    unlockAfterExport();
    if (renderWorkerGone(error)) {
      await reloadPreview();
    }
    if (known === null) {
      // A fault of this file. The export is shown as failed, and the fault is not hidden behind that.
      throw error;
    }
    return;
  }

  // 12.
  exportStore.saved(summary, false);
  unlockAfterExport();
  exportDone(values, exportedAs, startedAt, false);
}

/**
 * "Download again" (D-189): hands the last export's file over once more. A
 * file that is no longer on this device ends the done state: the clip can be
 * exported again.
 */
export async function downloadAgain(): Promise<void> {
  const file = delivered;
  const done = exportStore.useExportStore.getState().status === "done";
  if (file === undefined || !done || file.clipId !== useClipStore.getState().clipId) {
    return;
  }
  try {
    deliver(await opfs.getFile(file.path));
  } catch (error) {
    if (!(error instanceof opfs.OpfsError)) {
      throw error;
    }
    delivered = undefined;
    exportStore.reset();
  }
}

/** "Close" on a failed export's card (D-190): back to the editor, the Export button ready again. */
export function dismissExport(): void {
  if (exportStore.useExportStore.getState().status === "failed") {
    exportStore.reset();
  }
}
