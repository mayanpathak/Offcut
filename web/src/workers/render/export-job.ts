// The export of the render worker (TS §21, v2implementation §16.8): the
// profile from the token, the scene and a canvas at its size, the loop, the
// file moved to its place, and the tidying after. Moved from
// `render.worker.ts` (v4implementation D-150). The worker's state reaches it
// through one argument, `WorkerState`; this file never imports the worker.

import type {
  ChangeSummary,
  ClipInfo,
  DetectedEvent,
  DurMs,
  EditState,
  ExportId,
  ExportProfile,
  Transcript,
} from "../../gen/domain";
import { move, paths } from "../../persistence/opfs";
import type { RenderApi, RenderSession } from "../../wasm/load-render";
import type { FailureStage, StageTiming } from "../protocol";
import { CANCELLED, type JobContext, WorkerFailure } from "../rpc-worker";
import { runExport } from "./export-loop";
import { OpfsSink } from "./opfs-sink";
import type { VideoSource } from "./video-source";

export type Exported = { opfsPath: string; summary: ChangeSummary; stageTimings: StageTiming[] };

// The clip that is open, from `openSession` to `closeSession` (TS §14.3).
export type Session = {
  api: RenderApi;
  session: RenderSession;
  /** The clip's file. The session reads the video samples through it. */
  handle: FileSystemSyncAccessHandle;
  clipInfo: ClipInfo;
  source: VideoSource;
};

/** What the scene was last built from, without its profile. */
export type SceneState = { transcript: Transcript; events: DetectedEvent[]; edit: EditState };

/** The render worker's state as its job files see it. `render.worker.ts` owns it. */
export type WorkerState = {
  /** The open session; a failure when there is none. */
  opened(): Session;
  /** The open session, if there is one. */
  open(): Session | undefined;
  scene(): SceneState | undefined;
  /** The canvas of the page's player, once it is attached. */
  previewCanvas(): OffscreenCanvas | undefined;
  /** Closes every decoded frame; whether none was left open (D-45). */
  closeFrames(state: Session): boolean;
  /** What was thrown, as the failure a handler throws. */
  staged(thrown: unknown, stage: FailureStage): WorkerFailure;
  /** The profile of a verified token: only the worker file reads the keys (INV-9, D-27 c). */
  exportProfile(entitlementToken: string): ExportProfile;
};

/** What one export has got to: read when it fails, and when it is tidied up after. */
type ExportRun = {
  stage: FailureStage;
  /** The scene and the canvas are the export's, not the preview's. */
  switched: boolean;
  sink: OpfsSink | undefined;
};

/**
 * Steps 1 to 6 and 8 of v2implementation §16.8: the profile from the token,
 * the scene and a canvas at the profile's size, the loop, and the file moved
 * to its place. What must be undone when it does not get there is in `run`.
 */
async function writeExport(
  p: { exportId: ExportId; entitlementToken: string; out48: Float32Array },
  ctx: JobContext,
  run: ExportRun,
  w: WorkerState,
): Promise<Exported | typeof CANCELLED> {
  const state = w.opened();
  const { session } = state;
  const scene = w.scene();
  if (scene === undefined) {
    throw new WorkerFailure("E_INTERNAL", "NoScene");
  }
  // The token is verified before anything is drawn or written (INV-9).
  const profile = w.exportProfile(p.entitlementToken);

  run.switched = true;
  session.set_scene({ ...scene, profile });
  const canvas = new OffscreenCanvas(profile.width, profile.height);
  await session.attach_canvas(canvas, profile.width, profile.height);

  const temporary = paths.exportTmp(p.exportId);
  const final = paths.exportFinal(p.exportId);
  const sink = await OpfsSink.open(temporary);
  run.sink = sink;

  const started = performance.now();
  let encoded = started;
  const result = await runExport({
    session,
    source: state.source,
    profile,
    out48: p.out48,
    sink,
    canvas,
    onProgress: (done, total) => {
      ctx.progress({ stage: "render_encode", done, total });
    },
    onEncoded: () => {
      encoded = performance.now();
      run.stage = "mux";
      // The page moves the export to `muxing` on this message.
      ctx.progress({ stage: "mux", done: 0, total: 1 });
    },
    isCancelled: () => ctx.isCancelled(),
  });
  if ("cancelled" in result) {
    return CANCELLED;
  }
  // Every frame the export obtained must be closed by now (INV-11, D-45).
  if (!w.closeFrames(state)) {
    throw new WorkerFailure("E_INTERNAL", "FrameLeak", "render_encode");
  }

  await sink.close();
  await move(temporary, final);
  run.sink = undefined;
  const muxed = performance.now();

  const summary = session.summary();
  if (summary === null) {
    throw new WorkerFailure("E_INTERNAL", "NoScene");
  }
  return {
    opfsPath: final,
    summary,
    stageTimings: [
      { stage: "render_encode", durationMs: Math.round(encoded - started) as DurMs },
      { stage: "mux", durationMs: Math.round(muxed - encoded) as DurMs },
    ],
  };
}

/**
 * Steps 5, 7 and 9: after an export, however it ended. Nothing of one that
 * did not finish stays, no frame and no file, and the scene and the canvas
 * are the preview's again, when a preview canvas was attached.
 */
async function afterExport(run: ExportRun, w: WorkerState): Promise<void> {
  const state = w.open();
  if (state !== undefined) {
    w.closeFrames(state);
  }
  await run.sink?.abort();
  const scene = w.scene();
  const previewCanvas = w.previewCanvas();
  if (run.switched && state !== undefined && scene !== undefined && previewCanvas !== undefined) {
    const profile = state.api.previewProfile();
    state.session.set_scene({ ...scene, profile });
    await state.session.attach_canvas(previewCanvas, profile.width, profile.height);
  }
}

export async function exportClip(
  p: { exportId: ExportId; entitlementToken: string; out48: Float32Array },
  ctx: JobContext,
  w: WorkerState,
): Promise<Exported | typeof CANCELLED> {
  const run: ExportRun = { stage: "render_encode", switched: false, sink: undefined };
  let result: Exported | typeof CANCELLED | undefined;
  let failure: WorkerFailure | undefined;
  try {
    result = await writeExport(p, ctx, run, w);
  } catch (thrown) {
    failure = w.staged(thrown, run.stage);
  }
  try {
    await afterExport(run, w);
  } catch (thrown) {
    // The first failure is the one that is reported.
    failure ??= w.staged(thrown, run.stage);
  }
  if (failure !== undefined || result === undefined) {
    throw failure ?? new WorkerFailure("E_INTERNAL", "NoResult", run.stage);
  }
  return result;
}
