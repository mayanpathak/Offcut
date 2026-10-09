// C-5: the preview player. The page plays the clip's audio; the render worker
// draws the frame that belongs to the audio's time (TS §20). This file owns
// the one `AudioContext` and tells the worker the time four times a second.

import { track } from "../analytics/client";
import type { TimeMs } from "../gen/domain";
import { noteFailure } from "../state/clip-store";
import * as previewStore from "../state/preview-store";
import { pool, WorkerCallError } from "../workers/pool";
import type { ClockSync } from "../workers/protocol";

/** How often the worker is told the audio's time (TS §20.2). */
const CLOCK_SYNC_INTERVAL_MS = 250;
/** The rate of `out48`. */
const AUDIO_RATE = 48_000;

/** What is attached: the audio of one clip, and where its playing is. */
type Player = {
  context: AudioContext;
  buffer: AudioBuffer;
  /** The node that sounds, while the preview plays. */
  node: AudioBufferSourceNode | undefined;
  /** The context's time at which the node was started, in seconds, and the place in the clip it started at. */
  startedAt: number;
  startOffsetMs: number;
  /** Where a preview that was paused goes on. 0 for one that is stopped. */
  resumeAtMs: number;
  syncTimer: ReturnType<typeof setInterval> | undefined;
  /** Goes up with every play and every pause: what an older play still has to do is dropped. */
  turn: number;
  pausing: boolean;
};

let player: Player | undefined;

/** The one place of this file that turns a number into a time on the clip (v2implementation D-59). */
function toTimeMs(ms: number): TimeMs {
  return Math.max(0, Math.round(ms)) as TimeMs;
}

function status(): previewStore.PreviewStatus {
  return previewStore.usePreviewStore.getState().status;
}

/** The place in the clip that is sounding now, in milliseconds. */
function positionMs(p: Player): number {
  return (p.context.currentTime - p.startedAt) * 1000 + p.startOffsetMs;
}

/** The audio's time, with the moment it was read at: what the worker's clock runs on from. */
function clockNow(p: Player): ClockSync {
  return { audioMs: toTimeMs(positionMs(p)), epochMs: performance.timeOrigin + performance.now() };
}

function stopSound(p: Player): void {
  if (p.syncTimer !== undefined) {
    clearInterval(p.syncTimer);
    p.syncTimer = undefined;
  }
  if (p.node !== undefined) {
    p.node.stop();
    p.node.disconnect();
    p.node = undefined;
  }
}

/** A failure of the worker is stored where the page shows it. Anything else is a fault of this file. */
function stored(error: unknown): void {
  if (!(error instanceof WorkerCallError)) {
    throw error;
  }
  noteFailure(error.failure);
}

/**
 * Hands the page's canvas to the render worker and makes the clip's audio
 * ready to play. A preview that is attached already is detached first.
 */
export async function attach(canvas: HTMLCanvasElement, out48: Float32Array): Promise<void> {
  await detach();
  const context = new AudioContext();
  const buffer = context.createBuffer(1, out48.length, AUDIO_RATE);
  buffer.getChannelData(0).set(out48);
  const offscreen = canvas.transferControlToOffscreen();
  try {
    await pool.render.attachPreview({ canvas: offscreen }, { transfer: [offscreen] });
  } catch (error) {
    await context.close();
    stored(error);
    return;
  }
  player = {
    context,
    buffer,
    node: undefined,
    startedAt: 0,
    startOffsetMs: 0,
    resumeAtMs: 0,
    syncTimer: undefined,
    turn: 0,
    pausing: false,
  };
  previewStore.attached();
}

/** What a play does when the worker's loop has returned: the clip's end, or a failure. */
async function afterPlay(p: Player, turn: number, loop: Promise<unknown>): Promise<void> {
  try {
    await loop;
  } catch (error) {
    stored(error);
    await detach();
    return;
  }
  // A pause, or a detach, took this play's turn: the end is theirs to handle.
  if (player === p && p.turn === turn && status() === "playing") {
    stopSound(p);
    p.resumeAtMs = 0;
    previewStore.ended();
  }
}

/**
 * Plays from the start, or from where the preview was paused. The first play
 * follows a click, which is what lets the browser sound (TS §20.3). Resolves
 * when the preview is playing, not when it ends.
 */
export async function play(): Promise<void> {
  const p = player;
  if (p === undefined || (status() !== "stopped" && status() !== "paused")) {
    return;
  }
  await p.context.resume();
  // The wait may have been long: the preview may be gone, or playing already.
  if (player !== p || (status() !== "stopped" && status() !== "paused")) {
    return;
  }
  const offsetMs = status() === "paused" ? p.resumeAtMs : 0;
  const node = p.context.createBufferSource();
  node.buffer = p.buffer;
  node.connect(p.context.destination);
  node.start(0, offsetMs / 1000);
  p.node = node;
  p.startedAt = p.context.currentTime;
  p.startOffsetMs = offsetMs;
  p.turn += 1;

  const first = !previewStore.usePreviewStore.getState().playedOnce;
  previewStore.play();
  if (first) {
    track({ name: "preview_played" });
  }
  p.syncTimer = setInterval(() => {
    pool.render.notify("previewClock", { clock: clockNow(p) });
  }, CLOCK_SYNC_INTERVAL_MS);
  void afterPlay(p, p.turn, pool.render.previewPlay({ clock: clockNow(p) }));
}

/** Stops the sound and the drawing where they are. The last frame stays on the canvas. */
export async function pause(): Promise<void> {
  const p = player;
  if (p === undefined || p.pausing || status() !== "playing") {
    return;
  }
  p.pausing = true;
  p.resumeAtMs = positionMs(p);
  p.turn += 1;
  stopSound(p);
  try {
    // Answers when the worker's loop has returned.
    await pool.render.previewPause();
  } catch (error) {
    stored(error);
  } finally {
    p.pausing = false;
  }
  if (player === p) {
    previewStore.pause();
  }
}

/** Gives up the preview: the sound stops and the audio is let go. */
export async function detach(): Promise<void> {
  const p = player;
  if (p !== undefined) {
    await pause();
    player = undefined;
    stopSound(p);
    await p.context.close();
  }
  previewStore.detached();
}

/** Before an export: the preview stops and cannot be played until the export is over (TS §12.4). */
export async function lockForExport(): Promise<void> {
  await pause();
  previewStore.lock();
}

/** After an export. A preview that has a canvas is paused; one that has none is detached, as it was. */
export function unlockAfterExport(): void {
  previewStore.unlock();
  if (player === undefined) {
    previewStore.detached();
  }
}
