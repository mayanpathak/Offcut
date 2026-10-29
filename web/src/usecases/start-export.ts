// C-9, C-10: the clip becomes an MP4, and the MP4 is downloaded. The render
// worker draws, encodes and writes the file; its size and its watermark come
// from the entitlement token, which the worker verifies (INV-9). This file
// starts the export, follows it in the export store and hands the file over,
// again when asked. An export can be cancelled (v4implementation D-152): it
// ends `idle`, with no file of it on this device.
//
// V2 has no gate: the token is the one this device holds, and without one no
// export can be made (v2implementation D-23). Nothing here calls the API.

import { track } from "../analytics/client";
import type { ExportedAs } from "../gen/api";
import type { ClipId, ExportId } from "../gen/domain";
import * as entitlementRepo from "../persistence/entitlement-repo";
import * as opfs from "../persistence/opfs";
import { forExport } from "../state/blockers";
import { useClipStore } from "../state/clip-store";
import * as exportStore from "../state/export-store";
import { EXPORT_MACHINE } from "../state/machines/export-machine";
import { transition } from "../state/machines/transition";
import { pool, WorkerCallError, whenReleased } from "../workers/pool";
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

/**
 * Steps 6 to 10 of v2implementation §19.5. A failure is thrown; `startExport`
 * stores it. A cancel is thrown as `ExportCancelled`: the worker stopped,
 * or its file was finished and is not delivered.
 */
async function renderAndDeliver(
  clipId: ClipId,
  exportId: ExportId,
  entitlementToken: string,
  signal: AbortSignal,
): Promise<void> {
  // 6. V7: `quota.ensureFree` takes this sweep's place (v2implementation §25).
  //    Until then one export is kept (D-40). A failure is ignored.
  await opfs.remove(opfs.paths.exportsRoot()).catch(() => undefined);
  delivered = undefined;

  // 7. The export reads its own copy of the audio: a buffer that is handed
  //    to a worker is gone from this thread, and the preview keeps its own (D-51).
  //    V5: the export encodes `prog48` instead (TS §40.9 S-8; `program_identity.rs` proves it equals `out48`).
  const out48 = await opfs.readAudio(clipId);
  goOn(signal);

  // 8.
  const result = await pool.render.exportClip(
    { exportId, entitlementToken, out48 },
    {
      transfer: [out48.buffer],
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

  // 9. The file is whole. The worker measured its two stages itself (D-35).
  if (exportStore.useExportStore.getState().status === "rendering") {
    exportStore.encoded();
  }
  exportStore.finalized();
  for (const timing of result.stageTimings) {
    track({ name: "stage_timing", props: { stage: timing.stage, duration_ms: capMs(timing.durationMs) } });
  }

  // 10.
  deliver(await opfs.getFile(result.opfsPath));
  delivered = { clipId, path: result.opfsPath };
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
  exportStore.useExportStore.setState({ status: idle, done: 0, total: 0, unavailable }, true);
}

/**
 * Whether the render worker was ended under the export: it crashed, or the
 * cancel restarted it (Prompt 118). Its preview session went with it.
 */
function renderWorkerGone(error: unknown): boolean {
  return error instanceof WorkerCallError && error.failure.code === "E_WORKER_CRASH";
}

/**
 * Exports the clip and downloads the file: the eleven steps of
 * v2implementation §19.5. Resolves when the export is over, however it
 * ended; a failure is in the export store, and the card that shows it sends
 * `client_error` (cancel-job.ts). A cancelled export ends `idle`, with
 * neither its partial nor its finished file left (D-152).
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

  // 3.
  const last = exportStore.useExportStore.getState().status;
  if (last === "done" || last === "failed") {
    exportStore.reset();
  }
  const exportId = newExportId();
  exportStore.start(exportId);
  exportStore.clear();
  const signal = signalFor("export");
  const startedAt = performance.now();

  // 4.
  const profile = planOf(record.token);
  track({ name: "export_started", props: { profile } });

  // 5. The preview stops and stays locked until the export is over (TS §12.4).
  await lockForExport();

  try {
    goOn(signal);
    await renderAndDeliver(clipId, exportId, record.token, signal);
  } catch (error) {
    if (signal.aborted) {
      // The worker removes its partial file when it stops; one that was
      // restarted could not, and holds it a moment after. Neither file of
      // this export is kept.
      await whenReleased(() => opfs.remove(opfs.paths.exportTmp(exportId))).catch(() => undefined);
      await whenReleased(() => opfs.remove(opfs.paths.exportFinal(exportId))).catch(() => undefined);
      exportCancelled();
      unlockAfterExport();
      if (renderWorkerGone(error)) {
        await reloadPreview();
      }
      return;
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

  // 11.
  exportStore.saved();
  unlockAfterExport();
  const kept = useClipStore.getState().events.filter((event) => event.enabled).length;
  // V5: the render cache (TS §40.9 S-9) sets `from_cache`; the edit's own values take the place of
  // `word_edits`, `events_disabled`, `style` and `crop_adjusted`; `result.summary` is shown (ChangeSummary, PS §10 J11).
  track({
    name: "export_done",
    props: {
      total_ms: capMs(performance.now() - startedAt),
      profile,
      word_edits: 0,
      events_kept: Math.min(MAX_COUNT, kept),
      events_disabled: 0,
      style: "clean",
      crop_adjusted: false,
      from_cache: false,
    },
  });
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
