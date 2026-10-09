// The render worker: finds the visual events of a transcript, builds the
// scene, draws it over the clip's frames, and exports the clip as an MP4
// (TS §17 to §21). The rules are in Rust, in `offcut_render.wasm`; this file
// is the browser glue around them: the session of one clip, and its handlers.
//
// This is the only worker file that reads the entitlement public keys: the
// size and the watermark of an export come from the verified token and from
// nowhere else (INV-9, D-27 c).

import { ENTITLEMENT_PUBLIC_KEYS } from "../config/entitlement-public-key";
import type {
  ChangeSummary,
  ClipId,
  ClipInfo,
  DetectedEvent,
  DurMs,
  EditState,
  ExportId,
  Prosody,
  Transcript,
  UnixSecs,
} from "../gen/domain";
import { move, OpfsError, paths } from "../persistence/opfs";
import { loadRender, type RenderApi, type RenderSession } from "../wasm/load-render";
import { openSource } from "./media/import";
import type { FailureStage, RenderWorkerApi, StageTiming } from "./protocol";
import { runExport } from "./render/export-loop";
import { OpfsSink } from "./render/opfs-sink";
import { liveFrames, VideoSource } from "./render/video-source";
import { CANCELLED, type Handlers, type JobContext, serveWorker, toAppFailure, WorkerFailure } from "./rpc";

type Exported = { opfsPath: string; summary: ChangeSummary; stageTimings: StageTiming[] };

// The clip that is open, from `openSession` to `closeSession` (TS §14.3).
type Session = {
  api: RenderApi;
  session: RenderSession;
  /** The clip's file. The session reads the video samples through it. */
  handle: FileSystemSyncAccessHandle;
  clipInfo: ClipInfo;
  source: VideoSource;
};
let open: Session | undefined;
/** What the scene was last built from, without its profile. */
let scene: { transcript: Transcript; events: DetectedEvent[]; edit: EditState } | undefined;
/** The canvas of the page's player, once it is attached. */
let previewCanvas: OffscreenCanvas | undefined;

/**
 * What was thrown, as the failure a handler throws, with the stage it
 * happened in unless it names its own. A failure of the muxer is of the
 * `mux` stage wherever it shows (v2implementation §16.8).
 */
function staged(thrown: unknown, stage: FailureStage): WorkerFailure {
  if (thrown instanceof OpfsError) {
    const detail = thrown.cause instanceof Error ? thrown.cause.name : thrown.name;
    return new WorkerFailure(thrown.kind === "quota" ? "E_STORAGE_QUOTA" : "E_STORAGE_IO", detail, stage);
  }
  const failure = toAppFailure(thrown, stage);
  return new WorkerFailure(failure.code, failure.detail ?? "", failure.code === "E_MUX" ? "mux" : failure.stage);
}

function opened(): Session {
  if (open === undefined) {
    throw new WorkerFailure("E_INTERNAL", "NoSession");
  }
  return open;
}

/** The time a token is judged at: this device's clock, in whole seconds. */
function now(): UnixSecs {
  return Math.floor(Date.now() / 1000) as UnixSecs;
}

/**
 * Closes every decoded frame, by closing the source and making a new one for
 * what comes next. Returns whether no frame was left open after it, and
 * starts the count again: a frame that leaked fails the call that leaked it,
 * and no later one (D-45).
 */
function closeFrames(state: Session): boolean {
  state.source.close();
  state.source = new VideoSource(state.session, state.clipInfo);
  const none = liveFrames.count === 0;
  liveFrames.count = 0;
  return none;
}

async function openSession(p: { clipId: ClipId; clipInfo: ClipInfo }): Promise<void> {
  // One clip at a time: a session that was left open gives its file back first.
  closeSession();
  try {
    const api = await loadRender();
    const handle = await openSource(p.clipId, { create: false });
    let session: RenderSession | undefined;
    try {
      session = await api.openSession(p.clipInfo, handle);
      open = { api, session, handle, clipInfo: p.clipInfo, source: new VideoSource(session, p.clipInfo) };
    } catch (thrown) {
      session?.free();
      handle.close();
      throw thrown;
    }
  } catch (thrown) {
    throw staged(thrown, "detect_scene");
  }
}

async function detect(p: { transcript: Transcript; prosody: Prosody }): Promise<DetectedEvent[]> {
  try {
    // V2 has no word edits (V4).
    return (await loadRender()).detect(p.transcript, p.prosody, {});
  } catch (thrown) {
    throw staged(thrown, "detect_scene");
  }
}

function setScene(p: {
  transcript: Transcript;
  events: DetectedEvent[];
  edit: EditState;
  profile: "preview" | { entitlementToken: string };
}): void {
  try {
    const { api, session } = opened();
    const profile =
      p.profile === "preview"
        ? api.previewProfile()
        : api.exportProfileFromToken(p.profile.entitlementToken, ENTITLEMENT_PUBLIC_KEYS, now());
    session.set_scene({ transcript: p.transcript, events: p.events, edit: p.edit, profile });
    scene = { transcript: p.transcript, events: p.events, edit: p.edit };
  } catch (thrown) {
    throw staged(thrown, "detect_scene");
  }
}

async function attachPreview(p: { canvas: OffscreenCanvas }): Promise<void> {
  const { api, session } = opened();
  const size = api.previewProfile();
  // No other call on the session until this has answered.
  await session.attach_canvas(p.canvas, size.width, size.height);
  previewCanvas = p.canvas;
}

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
): Promise<Exported | typeof CANCELLED> {
  const state = opened();
  const { api, session } = state;
  if (scene === undefined) {
    throw new WorkerFailure("E_INTERNAL", "NoScene");
  }
  // The token is verified before anything is drawn or written (INV-9).
  const profile = api.exportProfileFromToken(p.entitlementToken, ENTITLEMENT_PUBLIC_KEYS, now());

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
  if (!closeFrames(state)) {
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
async function afterExport(run: ExportRun): Promise<void> {
  const state = open;
  if (state !== undefined) {
    closeFrames(state);
  }
  await run.sink?.abort();
  if (run.switched && state !== undefined && scene !== undefined && previewCanvas !== undefined) {
    const profile = state.api.previewProfile();
    state.session.set_scene({ ...scene, profile });
    await state.session.attach_canvas(previewCanvas, profile.width, profile.height);
  }
}

async function exportClip(
  p: { exportId: ExportId; entitlementToken: string; out48: Float32Array },
  ctx: JobContext,
): Promise<Exported | typeof CANCELLED> {
  const run: ExportRun = { stage: "render_encode", switched: false, sink: undefined };
  let result: Exported | typeof CANCELLED | undefined;
  let failure: WorkerFailure | undefined;
  try {
    result = await writeExport(p, ctx, run);
  } catch (thrown) {
    failure = staged(thrown, run.stage);
  }
  try {
    await afterExport(run);
  } catch (thrown) {
    // The first failure is the one that is reported.
    failure ??= staged(thrown, run.stage);
  }
  if (failure !== undefined || result === undefined) {
    throw failure ?? new WorkerFailure("E_INTERNAL", "NoResult", run.stage);
  }
  return result;
}

/** Idempotent. */
function closeSession(): void {
  const state = open;
  open = undefined;
  scene = undefined;
  previewCanvas = undefined;
  if (state === undefined) {
    return;
  }
  try {
    state.source.close();
    state.session.free();
  } finally {
    state.handle.close();
  }
}

/** A method of the protocol that this worker gets in a later prompt (D-32). */
function notYet(): never {
  throw new WorkerFailure("E_INTERNAL", "NotImplemented");
}

/**
 * One-way: nobody waits for an answer, and a failure thrown here would
 * reach the page as a crash of the worker. Until the preview loop reads the
 * clock, the message is dropped.
 */
function previewClock(): void {
  // Nothing reads the clock yet.
}

const handlers: Handlers<RenderWorkerApi> = {
  openSession,
  detect,
  redetectSentence: notYet,
  setScene,
  attachPreview,
  previewPlay: notYet,
  previewClock,
  previewPause: notYet,
  previewSeek: notYet,
  exportClip,
  closeSession,
};

serveWorker(handlers, {
  stage: "preview",
  oneWay: ["previewClock"],
  duringPreview: ["previewClock", "previewPause", "previewSeek"],
});
