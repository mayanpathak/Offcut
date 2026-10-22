// The preview of the render worker (TS §20): play until the clip ends or a
// pause, the page's audio clock, and the program time it implies. Moved
// from `render.worker.ts` (v4implementation D-150). This file owns the
// clock, the loop and its stop flag; the worker's state reaches it through
// one argument, `WorkerState`, and it never imports the worker.

import type { OutMs } from "../../gen/domain";
import type { ClockSync, PreviewStats } from "../protocol";
import { WorkerFailure } from "../rpc-worker";
import type { WorkerState } from "./export-job";
import { runPreview } from "./preview-loop";

/** The page's audio clock as the page last told it, and the preview loop that follows it. */
let clock: ClockSync | undefined;
let stopPreview = false;
let previewLoop: Promise<unknown> | undefined;

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
  const loop = runPreview(state.session, state.source, previewTime, () => stopPreview);
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
  return stats;
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

/** What `closeSession` does to the preview: the loop is told to stop, and the clock is forgotten. */
export function previewForget(): void {
  stopPreview = true;
  clock = undefined;
}
