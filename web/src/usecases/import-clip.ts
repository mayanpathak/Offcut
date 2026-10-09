// C-2: a clip arrives. A dropped file, or the sample clip from the asset
// host, is copied to this device, probed and validated in the media worker;
// an accepted clip goes on to the pipeline. The name of a dropped file is
// never read, stored or sent (TS §25.1 P-11).

import { track } from "../analytics/client";
import type { ClipSource, DurationBucket } from "../gen/api";
import type { ClipId, DurMs } from "../gen/domain";
import { fetchAsset, SAMPLE_CLIP_PATH } from "../net/asset-fetch";
import * as opfs from "../persistence/opfs";
import { forImport } from "../state/blockers";
import * as clipStore from "../state/clip-store";
import * as exportStore from "../state/export-store";
import { pool, WorkerCallError } from "../workers/pool";
import type { AppFailure } from "../workers/protocol";
import { runPipeline, startTimer } from "./run-pipeline";

// Where a clip's length falls, for `clip_accepted`.
const BUCKET_LT30_MS = 30_000;
const BUCKET_LT60_MS = 60_000;

/** The name given to the sample clip's file. A name is never read back. */
const SAMPLE_FILE_NAME = "sample";

/** The one place of this file that makes a clip's id (v2implementation D-59). */
export function newClipId(): ClipId {
  return crypto.randomUUID() as ClipId;
}

function bucketOf(duration: DurMs): DurationBucket {
  if (duration < BUCKET_LT30_MS) {
    return "lt30";
  }
  return duration < BUCKET_LT60_MS ? "lt60" : "lte90";
}

function status(): clipStore.ClipState["status"] {
  return clipStore.useClipStore.getState().status;
}

/**
 * Whether a render session was opened for the clip: it is `ready`, or it
 * failed while its scene was built or after. The pipeline closes the session
 * of a clip that fails; a second close does nothing.
 */
function sessionOpened(clip: clipStore.ClipState): boolean {
  if (clip.status === "ready" || clip.status === "updating") {
    return true;
  }
  const stage = clip.failure?.stage;
  return clip.status === "failed" && (stage === "detect_scene" || stage === "storage");
}

/**
 * "Start over": lets go of the clip that is here, and of what it left on this
 * device. Nothing happens while a clip is being imported or processed, or
 * while an export runs: there is no way back to `idle` from there.
 */
export async function dismissClip(): Promise<void> {
  const clip = clipStore.useClipStore.getState();
  if (clip.status === "idle" || forImport() !== null) {
    return;
  }
  const hadSession = sessionOpened(clip);

  // The page lets go of the clip at once: the player unmounts and detaches the preview.
  clipStore.reset();
  // An export that is over was this clip's.
  const exported = exportStore.useExportStore.getState().status;
  if (exported === "done" || exported === "failed") {
    exportStore.reset();
  }

  // What follows is tidying up. A failure in it is stored nowhere and stops
  // nothing: the next import removes the directory again.
  if (hadSession) {
    // The worker does one thing at a time: a preview that still plays would refuse the close.
    await pool.render.previewPause().catch(() => undefined);
    await pool.render.closeSession().catch(() => undefined);
  }
  if (clip.clipId !== undefined) {
    await opfs.remove(opfs.paths.clipDir(clip.clipId)).catch(() => undefined);
  }
}

/** The clip could not be brought in: it fails, and what was copied of it goes. */
async function importFailed(clipId: ClipId, failure: AppFailure): Promise<void> {
  clipStore.failed(failure);
  await opfs.remove(opfs.paths.clipDir(clipId)).catch(() => undefined);
}

/**
 * Brings a clip in: the nine steps of v2implementation §19.2. Resolves when
 * the clip is accepted, rejected or failed; the pipeline of an accepted clip
 * runs on after that.
 */
export async function importClip(file: File, source: ClipSource): Promise<void> {
  // 1. The component shows the words of the blocker.
  if (forImport() !== null) {
    return;
  }
  // 2. One clip at a time: the one that is here goes first.
  if (status() !== "idle") {
    await dismissClip();
  }
  // That took a moment: another clip may have come meanwhile.
  if (forImport() !== null || status() !== "idle") {
    return;
  }

  // 3.
  const clipId = newClipId();
  clipStore.begin(clipId, source);

  // 4. V7: `quota.ensureFree` takes this sweep's place (v2implementation §25).
  //    Until then one clip is kept (D-40). A failure is ignored: step 6 reports a full disk.
  await opfs.remove(opfs.paths.clipsRoot()).catch(() => undefined);

  // 5. `probe_audio` and the total are counted from here (D-35).
  startTimer(clipId);

  // 6.
  let probed: Awaited<ReturnType<typeof pool.media.importAndProbe>>;
  try {
    probed = await pool.media.importAndProbe({ clipId, file });
  } catch (error) {
    if (!(error instanceof WorkerCallError)) {
      await importFailed(clipId, { code: "E_INTERNAL", stage: "import", retryable: false });
      throw error;
    }
    await importFailed(clipId, error.failure);
    return;
  }
  if ("cancelled" in probed) {
    // Nothing sends a cancel before V4.
    await importFailed(clipId, { code: "E_INTERNAL", stage: "import", retryable: false, detail: "Cancelled" });
    return;
  }

  // 7. A clip that cannot be used is not an error (TS §11.3). No event before V3.
  if ("rejected" in probed) {
    clipStore.rejected(probed.rejected);
    await opfs.remove(opfs.paths.clipDir(clipId)).catch(() => undefined);
    return;
  }

  // 8.
  clipStore.accepted(probed.ok);
  track({
    name: "clip_accepted",
    props: { duration_bucket: bucketOf(probed.ok.duration), orientation: probed.ok.orientation, source },
  });

  // 9. Not waited for: the page shows the feed while it runs.
  void runPipeline(clipId);
}

/** The sample clip as a file, or why it could not be fetched. */
async function fetchSample(): Promise<File | { offline: boolean }> {
  // The signal is never aborted in V2: the cancel arrives in V4.
  const result = await fetchAsset(SAMPLE_CLIP_PATH, { signal: new AbortController().signal });
  if (!result.ok) {
    return { offline: result.cause === "offline" };
  }
  try {
    return new File([await result.response.blob()], SAMPLE_FILE_NAME);
  } catch (error) {
    // A `TypeError` is how a body reports that the network went away under it.
    if (error instanceof TypeError) {
      return { offline: true };
    }
    throw error;
  }
}

async function runSampleImport(): Promise<void> {
  if (forImport() !== null) {
    return;
  }
  const sample = await fetchSample();
  if (sample instanceof File) {
    await importClip(sample, "sample");
    return;
  }
  // The fetch took a while: a clip that came meanwhile is left alone.
  if (forImport() !== null || status() !== "idle") {
    return;
  }
  // idle → importing → failed: the page shows the failure where it shows any other.
  clipStore.begin(newClipId(), "sample");
  clipStore.failed(
    sample.offline
      ? { code: "E_NET_OFFLINE", stage: "import", retryable: true }
      : { code: "E_INTERNAL", stage: "import", retryable: false },
  );
}

let sampleImport: Promise<void> | undefined;

/**
 * Fetches the sample clip from the asset host and imports it (D-48). A
 * second call while the first still fetches joins it: the clip is fetched
 * once.
 */
export function importSampleClip(): Promise<void> {
  sampleImport ??= runSampleImport().finally(() => {
    sampleImport = undefined;
  });
  return sampleImport;
}
