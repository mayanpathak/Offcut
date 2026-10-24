// The render worker: finds the visual events of a transcript, makes them a
// plan, builds the scene from a plan it validates, draws it over the clip's
// frames, and exports the clip as an MP4 (TS §17 to §21, §40). The rules are
// in Rust, in `offcut_render.wasm`; this file is the browser glue around
// them: the session of one clip, and its handlers.
//
// This is the only worker file that reads the entitlement public keys: the
// size and the watermark of an export come from the verified token and from
// nowhere else (INV-9, D-27 c).

import { ENTITLEMENT_PUBLIC_KEYS } from "../config/entitlement-public-key";
import type {
  ClipId,
  ClipInfo,
  DetectedEvent,
  EditPlan,
  EditState,
  Prosody,
  SentenceIdx,
  TimeMap,
  Transcript,
  UnixSecs,
} from "../gen/domain";
import { OpfsError } from "../persistence/opfs";
import { loadRender, type RenderSession } from "../wasm/load-render";
import { openSource } from "./media/import";
import type { FailureStage, RenderWorkerApi, SceneReport } from "./protocol";
import { exportClip, type SceneState, type Session, type WorkerState } from "./render/export-job";
import {
  captionMirror,
  drawStill,
  hitTest,
  previewClock,
  previewForget,
  previewPause,
  previewPlay,
  previewProfile,
  previewSeek,
  styleThumbnails,
} from "./render/preview-job";
import { liveFrames, VideoSource } from "./render/video-source";
import { toAppFailure } from "./rpc";
import { type Handlers, type JobContext, serveWorker, WorkerFailure } from "./rpc-worker";

// The clip that is open, from `openSession` to `closeSession` (TS §14.3).
// The export and the preview are in `render/export-job.ts` and
// `render/preview-job.ts`; they reach this state through `worker` below.
let open: Session | undefined;
/** What the scene was last built from, without its profile. */
let scene: SceneState | undefined;
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

/**
 * The events of a whole transcript, with the user's word edits when there
 * are any (a replace-all, D-175). The open clip keeps what they were found
 * in, for `redetectSentence` (D-146): its handler takes no transcript.
 */
async function detect(p: {
  transcript: Transcript;
  prosody: Prosody;
  wordEdits?: Record<number, string>;
}): Promise<DetectedEvent[]> {
  try {
    const events = (await loadRender()).detect(p.transcript, p.prosody, p.wordEdits ?? {});
    if (open !== undefined) {
      open.detection = { transcript: p.transcript, prosody: p.prosody, events };
    }
    return events;
  } catch (thrown) {
    throw staged(thrown, "detect_scene");
  }
}

/**
 * The events after an edit in sentence `sentence` (D-146): those of the last
 * detection outside its scope are kept, the scope is detected again in the
 * transcript and the prosody `detect` kept. The result is kept in turn, for
 * the next edit.
 */
function redetectSentence(p: { sentence: SentenceIdx; wordEdits: Record<number, string> }): DetectedEvent[] {
  try {
    const state = opened();
    const kept = state.detection;
    if (kept === undefined) {
      throw new WorkerFailure("E_INTERNAL", "NoDetection");
    }
    const events = state.api.redetectSentence(kept.transcript, kept.prosody, p.wordEdits, p.sentence, kept.events);
    state.detection = { ...kept, events };
    return events;
  } catch (thrown) {
    throw staged(thrown, "detect_scene");
  }
}

/** The plan of the detector's events (D-124). It keeps nothing. */
async function rulesPlan(p: { transcript: Transcript; events: DetectedEvent[] }): Promise<EditPlan> {
  try {
    return (await loadRender()).rulesPlan(p.transcript, p.events);
  } catch (thrown) {
    throw staged(thrown, "detect_scene");
  }
}

/**
 * Validates the plan and builds the scene (TS §40.8); keeps what it was
 * built from, for the export's switch and back. A plan this build cannot
 * read, or a time map that is not the identity, is refused as `E_INTERNAL`.
 * While the preview is paused, the frame at its position is drawn again
 * with the new scene before this answers (D-173): a change shows at once.
 */
async function setScene(p: {
  transcript: Transcript;
  plan: EditPlan;
  edit: EditState;
  timeMap: TimeMap;
  profile: "preview" | { entitlementToken: string };
}): Promise<SceneReport> {
  let report: SceneReport;
  try {
    const state = opened();
    const profile =
      p.profile === "preview"
        ? previewProfile(state)
        : state.api.exportProfileFromToken(p.profile.entitlementToken, ENTITLEMENT_PUBLIC_KEYS, now());
    const kept = { transcript: p.transcript, plan: p.plan, edit: p.edit, timeMap: p.timeMap };
    report = state.session.set_scene({ ...kept, profile });
    scene = kept;
  } catch (thrown) {
    throw staged(thrown, "detect_scene");
  }
  try {
    await drawStill(worker);
  } catch (thrown) {
    throw staged(thrown, "preview");
  }
  return report;
}

/** Hands the canvas to the session and draws the frame at the position: the preview shows before its first play. */
async function attachPreview(p: { canvas: OffscreenCanvas }): Promise<void> {
  const state = opened();
  const size = previewProfile(state);
  // No other call on the session until this has answered.
  await state.session.attach_canvas(p.canvas, size.width, size.height);
  previewCanvas = p.canvas;
  await drawStill(worker);
}

/** Idempotent. */
function closeSession(): void {
  const state = open;
  open = undefined;
  scene = undefined;
  previewCanvas = undefined;
  previewForget();
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

/** This worker's state, as the two job files reach it (v4implementation D-150). */
const worker: WorkerState = {
  opened,
  open: () => open,
  scene: () => scene,
  previewCanvas: () => previewCanvas,
  closeFrames,
  staged,
  exportProfile: (entitlementToken) =>
    opened().api.exportProfileFromToken(entitlementToken, ENTITLEMENT_PUBLIC_KEYS, now()),
};

const handlers: Handlers<RenderWorkerApi> = {
  openSession,
  detect,
  redetectSentence: (p: Parameters<typeof redetectSentence>[0]) => Promise.resolve(redetectSentence(p)),
  rulesPlan,
  setScene,
  attachPreview,
  previewPlay: (p: Parameters<typeof previewPlay>[0]) => previewPlay(p, worker),
  previewClock,
  previewPause,
  previewSeek: (p: Parameters<typeof previewSeek>[0]) => previewSeek(p, worker),
  hitTest: (p: Parameters<typeof hitTest>[0]) => hitTest(p, worker),
  captionMirror: (p: Parameters<typeof captionMirror>[0]) => captionMirror(p, worker),
  styleThumbnails: (p: Parameters<typeof styleThumbnails>[0]) => styleThumbnails(p, worker),
  exportClip: (p: Parameters<typeof exportClip>[0], ctx: JobContext) => exportClip(p, ctx, worker),
  closeSession,
};

serveWorker(handlers, {
  stage: "preview",
  oneWay: ["previewClock"],
  // A hit test beside a play answers the video at once (D-183).
  duringPreview: ["previewClock", "previewPause", "previewSeek", "hitTest"],
});
