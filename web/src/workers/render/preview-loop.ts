// The preview: the frame of the moment, drawn as the page's audio plays
// (TS §20.2). The audio clock is the master. This loop asks it for the
// program time and draws what belongs to it; it never moves time itself, so
// the preview keeps the program's own pauses and its length (TS §20.3). The
// scene says which moment of the recording a program time shows (D-126).

import { type DurMs, LIMITS, type OutMs, type TimeMs } from "../../gen/domain";
import type { RenderSession, SourcePos } from "../../wasm/load-render";
import type { PreviewStats } from "../protocol";
import type { VideoSource } from "./video-source";

/** The recording time a source position shows: a held one shows its frame still. */
function recorded(pos: SourcePos): TimeMs {
  return "at" in pos ? pos.at : pos.held;
}

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
 * Draws the program at the clock's time until the clock passes its end, or
 * `isStopped` says so, or no scene is set. `source` keeps every decoded
 * frame: none is closed here, and the caller closes the source's frames
 * afterwards.
 */
export async function runPreview(
  session: RenderSession,
  source: VideoSource,
  clock: () => OutMs,
  isStopped: () => boolean,
): Promise<PreviewStats> {
  // V4 Prompt 112 adds the 360 x 640 fallback (D-149).
  const stats: PreviewStats = { frames: 0, late: 0, maxDriftMs: 0 as DurMs, downscaled: false };
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
      // Frame n is the program at n x 1000 / 30 ms; the scene says what of
      // the recording it shows.
      const tn = Math.floor((n * 1000) / LIMITS.OUTPUT_FPS) as OutMs;
      const pos = session.source_at(tn);
      if (pos === null) {
        break;
      }
      const decoded = source.frameAt(recorded(pos));
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
