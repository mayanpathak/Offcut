// The render worker: finds the visual events of a transcript, builds the
// scene, draws it over the clip's frames, and exports the clip as an MP4
// (TS §17 to §21). The rules are in Rust, in `offcut_render.wasm`; this file
// is the browser glue around them: the session of one clip, and its handlers.
//
// This is the only worker file that reads the entitlement public keys: the
// size and the watermark of an export come from the verified token and from
// nowhere else (INV-9, D-27 c).

import { ENTITLEMENT_PUBLIC_KEYS } from "../config/entitlement-public-key";
import type { ClipId, ClipInfo, DetectedEvent, EditState, Prosody, Transcript, UnixSecs } from "../gen/domain";
import { OpfsError } from "../persistence/opfs";
import { loadRender, type RenderSession } from "../wasm/load-render";
import { openSource } from "./media/import";
import type { FailureStage, RenderWorkerApi } from "./protocol";
import { exportClip, type SceneState, type Session, type WorkerState } from "./render/export-job";
import { previewClock, previewForget, previewPause, previewPlay } from "./render/preview-job";
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

/** A method of the protocol that this worker gets in a later prompt (D-32). */
function notYet(): never {
  throw new WorkerFailure("E_INTERNAL", "NotImplemented");
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
  redetectSentence: notYet,
  setScene,
  attachPreview,
  previewPlay: (p: Parameters<typeof previewPlay>[0]) => previewPlay(p, worker),
  previewClock,
  previewPause,
  previewSeek: notYet,
  exportClip: (p: Parameters<typeof exportClip>[0], ctx: JobContext) => exportClip(p, ctx, worker),
  closeSession,
};

serveWorker(handlers, {
  stage: "preview",
  oneWay: ["previewClock"],
  duringPreview: ["previewClock", "previewPause", "previewSeek"],
});
