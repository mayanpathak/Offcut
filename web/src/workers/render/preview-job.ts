// The preview of the render worker (TS §20): play until the clip ends or a
// pause, the page's audio clock, and the program time it implies; the frame
// of a time while paused (a seek, a change, the first frame); the 360 x 640
// fallback. Moved from `render.worker.ts` (v4implementation D-150). This file
// owns the clock, the loop and its stop flag, and the position last drawn;
// the worker's state reaches it through one argument, `WorkerState`, and it
// never imports the worker.

import { type ExportProfile, LIMITS, type OutMs, type Px, type StyleId } from "../../gen/domain";
import type { CaptionMirror, ClockSync, Hit, PreviewStats } from "../protocol";
import { WorkerFailure } from "../rpc-worker";
import type { Session, WorkerState } from "./export-job";
import { recorded, runPreview } from "./preview-loop";

/** Over this mean cost of drawing a frame, the preview draws at 360 x 640 (D-149). */
const DOWNSCALE_MEAN_MS = 28;
/** The mean is taken over this many frames, the last ones drawn. */
const DOWNSCALE_WINDOW = 60;
/** 540 x 960 becomes 360 x 640. */
const DOWNSCALE_FACTOR = 2 / 3;

/** The page's audio clock as the page last told it, and the preview loop that follows it. */
let clock: ClockSync | undefined;
let stopPreview = false;
let previewLoop: Promise<unknown> | undefined;
/** The program time of the frame on the canvas, or of the next one to draw: where a still frame is drawn. */
let position = 0 as OutMs;
/** What drawing the last frames cost, in milliseconds, for the fallback. */
let costs: number[] = [];

/**
 * The program time that sounds now: where the audio was when the page last
 * said so, and what has passed since. The page's clock and this worker's
 * count from different starts; the sum of start and time passed is the same
 * moment in both (TS §20.2).
 */
function previewTime(): OutMs {
  if (clock === undefined) {
    return 0 as OutMs;
  }
  const since = performance.timeOrigin + performance.now() - clock.epochMs;
  return Math.max(0, clock.audioMs + since) as OutMs;
}

/** The preview's profile for this clip: 540 x 960, or 360 x 640 once it was downscaled, for good (D-149). */
export function previewProfile(state: Session): ExportProfile {
  const profile = state.api.previewProfile();
  if (state.downscaled !== true) {
    return profile;
  }
  return {
    ...profile,
    width: Math.round(profile.width * DOWNSCALE_FACTOR) as Px,
    height: Math.round(profile.height * DOWNSCALE_FACTOR) as Px,
  };
}

/** Draws at 360 x 640 from now on, with the scene set again at that size. The page's canvas keeps its CSS size. */
function downscale(w: WorkerState): void {
  const state = w.opened();
  state.downscaled = true;
  const profile = previewProfile(state);
  state.session.resize(profile.width, profile.height);
  const scene = w.scene();
  if (scene !== undefined) {
    state.session.set_scene({ ...scene, profile });
  }
}

/** Notes a frame the loop drew: the position, and its cost while the mean may still call for the fallback. */
function drawn(w: WorkerState, tn: OutMs, costMs: number): void {
  position = tn;
  if (w.opened().downscaled === true) {
    return;
  }
  costs.push(costMs);
  if (costs.length > DOWNSCALE_WINDOW) {
    costs.shift();
  }
  const mean = costs.reduce((sum, cost) => sum + cost, 0) / costs.length;
  if (costs.length === DOWNSCALE_WINDOW && mean > DOWNSCALE_MEAN_MS) {
    downscale(w);
  }
}

/** The time of the output frame that shows `at`, within the program. */
function frameTime(state: Session, at: OutMs): OutMs {
  const last = Math.max(0, state.session.frame_count() - 1);
  const n = Math.min(Math.max(0, Math.floor((at * LIMITS.OUTPUT_FPS) / 1000)), last);
  return Math.floor((n * 1000) / LIMITS.OUTPUT_FPS) as OutMs;
}

/**
 * Draws the frame at the position, waiting for the decoder: the first frame
 * after a canvas is attached, a seek, and the redraw after a change while
 * paused (D-173). Nothing while the loop runs: it draws the next frame
 * itself. The frame stays the source's, for the next still or play.
 */
export async function drawStill(w: WorkerState): Promise<void> {
  const state = w.open();
  if (previewLoop !== undefined || state === undefined || w.scene() === undefined) {
    return;
  }
  const pos = state.session.source_at(position);
  if (pos === null) {
    return;
  }
  if (w.previewCanvas() === undefined) {
    // The scene is set before the page has a canvas: its frame is decoded
    // now, so that the first still finds it (§15.3). Decoded only when the
    // canvas came, it took 600 ms of the first frame on D1 (Prompt 124).
    state.source.prefetch(recorded(pos));
    return;
  }
  const frame = await state.source.frameAtBlocking(recorded(pos));
  state.session.render_frame(frame, position);
}

/**
 * Draws the preview until the clip ends or `previewPause` stops it. Answers
 * when the loop has returned, with what it drew.
 */
export async function previewPlay(p: { clock: ClockSync }, w: WorkerState): Promise<PreviewStats> {
  const state = w.opened();
  const scene = w.scene();
  const previewCanvas = w.previewCanvas();
  if (scene === undefined || previewCanvas === undefined) {
    throw new WorkerFailure("E_INTERNAL", scene === undefined ? "NoScene" : "NoCanvas");
  }
  clock = p.clock;
  stopPreview = false;
  costs = [];
  const loop = runPreview(
    state.session,
    state.source,
    previewTime,
    () => stopPreview,
    (tn, costMs) => {
      drawn(w, tn, costMs);
    },
  );
  previewLoop = loop;
  let closed = false;
  let stats: PreviewStats;
  try {
    stats = await loop;
  } finally {
    previewLoop = undefined;
    // The last frame stays on the canvas; no decoded frame stays open (INV-11).
    closed = w.closeFrames(state);
  }
  if (!closed) {
    throw new WorkerFailure("E_INTERNAL", "FrameLeak");
  }
  // A play or a seek that follows finds the frames of where this one stopped decoded.
  const stoppedAt = state.session.source_at(position);
  if (stoppedAt !== null) {
    state.source.prefetch(recorded(stoppedAt));
  }
  return { ...stats, downscaled: state.downscaled === true };
}

/**
 * One-way: nobody waits for an answer, and a failure thrown here would
 * reach the page as a crash of the worker. It only notes the time.
 */
export function previewClock(p: { clock: ClockSync }): void {
  clock = p.clock;
}

/** Stops the loop and answers when it has returned, however it ended: its failure is `previewPlay`'s to report. */
export async function previewPause(): Promise<void> {
  stopPreview = true;
  await previewLoop?.then(
    () => undefined,
    () => undefined,
  );
}

/**
 * The frame of `at`, drawn while the preview is paused or scrubbed (D-148):
 * a loop that runs is stopped first, so no two loops ever run, and the
 * decoder starts again at the keyframe at or before `at` when it lies
 * outside what is decoded (TS §20.2). A play that follows starts there.
 */
export async function previewSeek(p: { at: OutMs }, w: WorkerState): Promise<void> {
  await previewPause();
  const state = w.opened();
  if (w.scene() === undefined || w.previewCanvas() === undefined) {
    throw new WorkerFailure("E_INTERNAL", w.scene() === undefined ? "NoScene" : "NoCanvas");
  }
  position = frameTime(state, p.at);
  await drawStill(w);
}

/**
 * What the point `(x, y)`, fractions of the canvas, is on at `at` (D-174): at
 * the output frame that shows `at`, the frame a still of `at` draws. While
 * the loop runs, the video, without work: the page manipulates a paused
 * preview only (D-183), and the loop has the session.
 */
export async function hitTest(p: { at: OutMs; x: number; y: number }, w: WorkerState): Promise<Hit> {
  const state = w.open();
  if (previewLoop !== undefined || state === undefined) {
    return Promise.resolve({ kind: "video" });
  }
  return Promise.resolve(state.session.hit(frameTime(state, p.at), p.x, p.y) ?? { kind: "video" });
}

/** The caption of the frame that shows `at`, as it is drawn: what development builds of the page mirror (D-155). */
export async function captionMirror(p: { at: OutMs }, w: WorkerState): Promise<CaptionMirror | null> {
  const state = w.open();
  return Promise.resolve(state === undefined ? null : state.session.caption_mirror(frameTime(state, p.at)));
}

/** The three styles of the cards, in their order on the page (D-186). */
const THUMBNAIL_STYLES: readonly StyleId[] = ["clean", "bold", "tech"];

/**
 * Three frames of `at` in Clean, Bold and Mono with the current edits
 * (D-186): one canvas of `width` by `height`, the scene built at that size
 * for each style, an `ImageBitmap` taken after each render. The preview's
 * scene and canvas are given back before this answers and its frame is drawn
 * again, so the next frame is the one before the call. Paused only: the
 * worker refuses it beside a play.
 */
export async function styleThumbnails(
  p: { at: OutMs; width: number; height: number },
  w: WorkerState,
): Promise<ImageBitmap[]> {
  const state = w.opened();
  const scene = w.scene();
  const previewCanvas = w.previewCanvas();
  if (scene === undefined || previewCanvas === undefined) {
    throw new WorkerFailure("E_INTERNAL", scene === undefined ? "NoScene" : "NoCanvas");
  }
  const t = frameTime(state, p.at);
  const pos = state.session.source_at(t);
  if (pos === null) {
    throw new WorkerFailure("E_INTERNAL", "NoScene");
  }
  const frame = await state.source.frameAtBlocking(recorded(pos));
  const profile = { ...previewProfile(state), width: p.width as Px, height: p.height as Px };
  const canvas = new OffscreenCanvas(p.width, p.height);
  const bitmaps: ImageBitmap[] = [];
  try {
    await state.session.attach_canvas(canvas, profile.width, profile.height);
    for (const style of THUMBNAIL_STYLES) {
      state.session.set_scene({ ...scene, edit: { ...scene.edit, style }, profile });
      state.session.render_frame(frame, t);
      bitmaps.push(canvas.transferToImageBitmap());
    }
  } catch (thrown) {
    for (const bitmap of bitmaps) {
      bitmap.close();
    }
    throw thrown;
  } finally {
    // The preview as it was: its scene, its canvas, its frame (as the export's switch back).
    const back = previewProfile(state);
    state.session.set_scene({ ...scene, profile: back });
    await state.session.attach_canvas(previewCanvas, back.width, back.height);
    await drawStill(w);
  }
  return bitmaps;
}

/** What `closeSession` does to the preview: the loop is told to stop, and the clock and the position are forgotten. */
export function previewForget(): void {
  stopPreview = true;
  clock = undefined;
  position = 0 as OutMs;
  costs = [];
}
