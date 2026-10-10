// The preview: the frame of the moment, drawn as the page's audio plays
// (TS §20.2). The audio clock is the master. This loop asks it for the time
// and draws what belongs to it; it never moves time itself, so the preview
// keeps the clip's own pauses and its length (TS §20.3).

import { type DurMs, LIMITS, type TimeMs } from "../../gen/domain";
import type { RenderSession } from "../../wasm/load-render";
import type { VideoSource } from "./video-source";

export type PreviewStats = {
  /** Frames drawn. */
  frames: number;
  /** Times the frame of the moment was not decoded yet, and the one before it was shown. */
  late: number;
  /** The most the time drawn lay from the time asked for. */
  maxDriftMs: DurMs;
};

/**
 * A worker of a hidden page gets no animation frame. The loop still takes a
 * step this often, so that a stop is seen and answered.
 */
const HIDDEN_STEP_MS = 100;

/** Resolves at the next animation frame, or after `HIDDEN_STEP_MS` when none comes. */
function nextStep(): Promise<void> {
  return new Promise((resolve) => {
    const timer = setTimeout(() => {
      cancelAnimationFrame(frame);
      resolve();
    }, HIDDEN_STEP_MS);
    const frame = requestAnimationFrame(() => {
      clearTimeout(timer);
      resolve();
    });
  });
}

/**
 * Draws the clip at the clock's time until the clock passes the clip's end,
 * or `isStopped` says so. `source` keeps every decoded frame: none is closed
 * here, and the caller closes the source's frames afterwards.
 */
export async function runPreview(
  session: RenderSession,
  source: VideoSource,
  clock: () => TimeMs,
  isStopped: () => boolean,
): Promise<PreviewStats> {
  const stats: PreviewStats = { frames: 0, late: 0, maxDriftMs: 0 as DurMs };
  const total = session.frame_count();
  let lastN = -1;
  let lastFrame: VideoFrame | null = null;

  while (!isStopped()) {
    const t = clock();
    const n = Math.max(0, Math.floor((t * LIMITS.OUTPUT_FPS) / 1000));
    if (n >= total) {
      break;
    }
    if (n !== lastN) {
      lastN = n;
      // Output time is source time (INV-5): frame n shows the clip at n x 1000 / 30 ms.
      const tn = Math.floor((n * 1000) / LIMITS.OUTPUT_FPS) as TimeMs;
      const decoded = source.frameAt(tn);
      if (decoded === null) {
        stats.late += 1;
      }
      // A late frame: the one before it is drawn again, with the scene of now.
      const frame: VideoFrame | null = decoded ?? lastFrame;
      if (frame !== null) {
        session.render_frame(frame, tn);
        lastFrame = frame;
        stats.frames += 1;
        stats.maxDriftMs = Math.max(stats.maxDriftMs, Math.abs(t - tn)) as DurMs;
      }
    }
    await nextStep();
  }
  return stats;
}
