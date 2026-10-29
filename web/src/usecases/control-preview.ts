// C-5: the preview player. The page plays the clip's audio; the render worker
// draws the frame that belongs to the audio's time (TS §20). This file owns
// the one `AudioContext` and tells the worker the time four times a second,
// and brings a preview that stopped back (`reloadPreview`, D-153).

import { track } from "../analytics/client";
import { LIMITS, type OutMs } from "../gen/domain";
import { clearFailure, noteFailure, setChunks, useClipStore } from "../state/clip-store";
import * as previewStore from "../state/preview-store";
import { type Cancelled, pool, WorkerCallError, whenFree, whenReleased } from "../workers/pool";
import type { ClockSync, PreviewStats } from "../workers/protocol";

/** How often the worker is told the audio's time (TS §20.2). */
const CLOCK_SYNC_INTERVAL_MS = 250;
/** The rate of `prog48`. */
const AUDIO_RATE = 48_000;

/** What is attached: the audio of one clip, and where its playing is. */
type Player = {
  buffer: AudioBuffer;
  /** The node that sounds, while the preview plays. */
  node: AudioBufferSourceNode | undefined;
  /** The context's time at which the node was started, in seconds, and the place in the clip it started at. */
  startedAt: number;
  startOffsetMs: number;
  /** Where a play starts: where the preview was paused or scrubbed to; 0 at the start and after the end. */
  resumeAtMs: number;
  syncTimer: ReturnType<typeof setInterval> | undefined;
  /** Goes up with every play and every pause: what an older play still has to do is dropped. */
  turn: number;
  pausing: boolean;
  /** The scrub bar was taken while the preview played: it plays on when let go. */
  playAfterSeek: boolean;
};

let player: Player | undefined;

/** The seek being drawn, and the latest target asked for since: sent when it answers (D-148). */
let seekInFlight: Promise<void> | undefined;
let seekPending: OutMs | undefined;

/** The one place of this file that turns a number into a program time (v2implementation D-59, D-126). */
function toOutMs(ms: number): OutMs {
  return Math.max(0, Math.round(ms)) as OutMs;
}

function status(): previewStore.PreviewStatus {
  return previewStore.usePreviewStore.getState().status;
}

/** The place in the program that is sounding now, in milliseconds. */
function positionMs(p: Player): number {
  return (audioContext().currentTime - p.startedAt) * 1000 + p.startOffsetMs;
}

/** The audio's time, with the moment it was read at: what the worker's clock runs on from. */
function clockNow(p: Player): ClockSync {
  return { audioMs: toOutMs(positionMs(p)), epochMs: performance.timeOrigin + performance.now() };
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

/**
 * A failure of the worker is stored where the page shows it; while one is
 * shown it stays, so the card says the first thing that went wrong. Anything
 * else is a fault of this file.
 */
function stored(error: unknown): void {
  if (!(error instanceof WorkerCallError)) {
    throw error;
  }
  if (useClipStore.getState().failure === undefined) {
    noteFailure(error.failure);
  }
}

let crashWatched = false;

/**
 * A render worker that crashes while nothing waits for it, the preview
 * paused, is shown as a call's failure would be (D-153): its session is
 * gone, and "Reload preview" brings it back. A change in flight shows its
 * own failure, and an export handles its own worker (start-export.ts).
 */
function watchCrash(): void {
  if (crashWatched) {
    return;
  }
  crashWatched = true;
  pool.render.onCrash((failure) => {
    const clip = useClipStore.getState();
    if (clip.status === "ready" && clip.failure === undefined && status() !== "locked") {
      noteFailure(failure);
    }
  });
}

let shared: AudioContext | undefined;

/**
 * The page's one `AudioContext`, made at the first play and kept: every
 * clip's preview plays through it. Chrome's first context in a page takes
 * about 250 ms of the main thread to make (D1, Prompt 124), and one made
 * while another lives takes almost none. A context made for each clip at
 * `ready` held the main thread that long at the end of every run
 * (v4implementation D-156); one made when the editor opened slowed every
 * page of a machine with no sound device (CI's Windows runner). The first
 * play follows a click, the moment a browser lets a page sound anyway.
 */
function audioContext(): AudioContext {
  if (shared === undefined || shared.state === "closed") {
    shared = new AudioContext();
  }
  return shared;
}

/**
 * Hands the page's canvas to the render worker and makes the clip's audio
 * ready to play. A preview that is attached already is detached first.
 */
export async function attach(canvas: HTMLCanvasElement, prog48: Float32Array): Promise<void> {
  watchCrash();
  await detach();
  // A buffer needs no context: none is made before the first play.
  const buffer = new AudioBuffer({ numberOfChannels: 1, length: prog48.length, sampleRate: AUDIO_RATE });
  buffer.getChannelData(0).set(prog48);
  const offscreen = canvas.transferControlToOffscreen();
  try {
    await pool.render.attachPreview({ canvas: offscreen }, { transfer: [offscreen] });
  } catch (error) {
    stored(error);
    return;
  }
  player = {
    buffer,
    node: undefined,
    startedAt: 0,
    startOffsetMs: 0,
    resumeAtMs: 0,
    syncTimer: undefined,
    turn: 0,
    pausing: false,
    playAfterSeek: false,
  };
  previewStore.attached();
  // The worker drew the first frame: the position is known, for the gestures too (D-183).
  previewStore.at(toOutMs(0));
}

/** What a play does when the worker's loop has returned: the clip's end, or a failure. */
async function afterPlay(p: Player, turn: number, loop: Promise<PreviewStats | Cancelled>): Promise<void> {
  try {
    const stats = await loop;
    // What the loop drew reaches the page, never the network (D-155).
    if (!("cancelled" in stats)) {
      previewStore.played(stats);
    }
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

/** The sound from `resumeAtMs`, the worker's loop after it, and the clock four times a second. */
function startPlaying(p: Player): void {
  const offsetMs = p.resumeAtMs;
  const context = audioContext();
  const node = context.createBufferSource();
  node.buffer = p.buffer;
  node.connect(context.destination);
  node.start(0, offsetMs / 1000);
  p.node = node;
  p.startedAt = context.currentTime;
  p.startOffsetMs = offsetMs;
  p.turn += 1;
  p.syncTimer = setInterval(() => {
    const clock = clockNow(p);
    pool.render.notify("previewClock", { clock });
    previewStore.at(clock.audioMs);
  }, CLOCK_SYNC_INTERVAL_MS);
  // The worker may be drawing the style cards: the play waits for it, and its
  // clock is read when it starts. Paused meanwhile, it does not start at all.
  const turn = p.turn;
  const loop = whenFree(() =>
    player === p && p.turn === turn
      ? pool.render.previewPlay({ clock: clockNow(p) })
      : Promise.resolve<Cancelled>({ cancelled: true }),
  );
  void afterPlay(p, turn, loop);
}

/**
 * Plays from where the preview was paused or scrubbed to, or from the start.
 * The first play follows a click, which is what lets the browser sound (TS
 * §20.3). Resolves when the preview is playing, not when it ends.
 */
export async function play(): Promise<void> {
  const p = player;
  if (p === undefined || (status() !== "stopped" && status() !== "paused")) {
    return;
  }
  await audioContext().resume();
  // The wait may have been long: the preview may be gone, or playing already.
  if (player !== p || (status() !== "stopped" && status() !== "paused")) {
    return;
  }
  const first = !previewStore.usePreviewStore.getState().playedOnce;
  previewStore.play();
  if (first) {
    track({ name: "preview_played" });
  }
  startPlaying(p);
}

/** Stops the sound and the drawing where they are. The last frame stays on the canvas. */
export async function pause(): Promise<void> {
  const p = player;
  if (p === undefined || p.pausing || status() !== "playing") {
    return;
  }
  p.pausing = true;
  p.resumeAtMs = positionMs(p);
  previewStore.at(toOutMs(p.resumeAtMs));
  p.turn += 1;
  stopSound(p);
  try {
    // Answers when the worker's loop has returned. The canvas then shows the
    // frame of where the sound stopped, as the time does: what a gesture on
    // the preview is about (D-183), and where a play goes on from.
    await whenFree(() => pool.render.previewPause());
    const at = toOutMs(p.resumeAtMs);
    await whenFree(() => pool.render.previewSeek({ at }));
  } catch (error) {
    stored(error);
  } finally {
    p.pausing = false;
  }
  if (player === p) {
    previewStore.pause();
  }
}

/**
 * The scrub bar is taken (D-148): a preview that plays stops its sound and
 * its loop, and plays on from the new place when the bar is let go.
 */
export async function seekStart(): Promise<void> {
  const p = player;
  if (p === undefined) {
    return;
  }
  if (status() === "playing") {
    await pause();
    p.playAfterSeek = player === p;
  }
  if (status() === "paused") {
    previewStore.seekStart();
  }
}

/**
 * Draws the frame of `at` (D-148). One seek is drawn at a time: a target
 * asked for meanwhile waits, and only the latest is drawn after it. A play
 * that follows starts at `at`. Resolves when the latest target is drawn.
 */
export async function seek(at: OutMs): Promise<void> {
  const p = player;
  if (p === undefined || status() === "locked" || status() === "detached") {
    return;
  }
  p.resumeAtMs = at;
  previewStore.at(at);
  if (seekInFlight !== undefined) {
    seekPending = at;
    return seekInFlight;
  }
  previewStore.seekBusy(true);
  seekInFlight = (async () => {
    let target: OutMs | undefined = at;
    while (target !== undefined) {
      seekPending = undefined;
      // The worker may be drawing the style cards or answering a change: the seek waits for it.
      const frame: OutMs = target;
      await whenFree(() => pool.render.previewSeek({ at: frame }));
      target = seekPending;
    }
  })();
  try {
    await seekInFlight;
  } catch (error) {
    // A decode failure shows as V3 shows it (E_DECODE_VIDEO); the targets after it are dropped.
    seekPending = undefined;
    stored(error);
  } finally {
    seekInFlight = undefined;
    previewStore.seekBusy(false);
  }
}

/** The scrub bar's value, in milliseconds: a seek there. From the keyboard while playing, the play goes on from it. */
export async function scrub(ms: number): Promise<void> {
  if (status() === "playing") {
    await seekStart();
    await seek(toOutMs(ms));
    await seekDone();
    return;
  }
  await seek(toOutMs(ms));
}

/** Space (D-185): plays a paused or stopped preview, pauses one that plays. */
export function togglePlay(): Promise<void> {
  return status() === "playing" ? pause() : play();
}

/** The program's length, in milliseconds: as long as its audio. */
function lengthMs(p: Player): number {
  return (p.buffer.length / AUDIO_RATE) * 1000;
}

/**
 * ← and → (D-185): one output frame back or forward, while the preview is
 * paused or stopped. Frame n is shown from n * 1000 / 30 ms; the time is
 * rounded up, because the worker takes the frame of a time rounded down.
 */
export async function stepFrame(direction: -1 | 1): Promise<void> {
  const p = player;
  if (p === undefined || (status() !== "paused" && status() !== "stopped")) {
    return;
  }
  const now = Math.floor(((previewStore.usePreviewStore.getState().position ?? 0) * LIMITS.OUTPUT_FPS) / 1000);
  const ms = Math.ceil((Math.max(0, now + direction) * 1000) / LIMITS.OUTPUT_FPS);
  if (ms <= lengthMs(p)) {
    await seek(toOutMs(ms));
  }
}

/** Shift+← and Shift+→ (D-185): one second back or forward. A preview that plays goes on playing from there. */
export async function stepSecond(direction: -1 | 1): Promise<void> {
  const p = player;
  if (p === undefined || status() === "seeking") {
    return;
  }
  const now = status() === "playing" ? positionMs(p) : (previewStore.usePreviewStore.getState().position ?? 0);
  await scrub(Math.min(Math.max(now + direction * 1000, 0), lengthMs(p)));
}

/** The scrub bar is let go: once its last frame is drawn, the preview plays on if it played before. */
export async function seekDone(): Promise<void> {
  const p = player;
  if (p === undefined || status() !== "seeking") {
    return;
  }
  await seekInFlight;
  if (player !== p || status() !== "seeking") {
    return;
  }
  const playing = p.playAfterSeek;
  p.playAfterSeek = false;
  previewStore.seekDone(playing);
  if (playing) {
    startPlaying(p);
  }
}

/** Gives up the preview: the sound stops and the audio is let go; the page's context waits, suspended, for the next clip. */
export async function detach(): Promise<void> {
  const p = player;
  if (p !== undefined) {
    await pause();
    player = undefined;
    stopSound(p);
    await shared?.suspend();
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

/** A worker's answer to a call nothing cancels: one that says it was cancelled has gone wrong. */
function answered<T>(result: T | Cancelled): T {
  if (typeof result === "object" && result !== null && "cancelled" in result) {
    throw new WorkerCallError({ code: "E_INTERNAL", stage: "preview", retryable: false, detail: "Cancelled" });
  }
  return result;
}

let reloading: Promise<void> | undefined;

async function reload(): Promise<void> {
  const clip = useClipStore.getState();
  const { clipId, clipInfo, transcript, prosody, plan, timeMap, edit } = clip;
  if (clip.status !== "ready" || clipId === undefined || clipInfo === undefined) {
    return;
  }
  if (transcript === undefined || prosody === undefined || plan === undefined || timeMap === undefined) {
    return;
  }
  await detach();
  // A fresh worker: a crashed one is gone already, and a lost graphics device is not used again.
  pool.restartWorker("render");
  try {
    // The worker before may still hold the clip's file a moment (D-152).
    answered(await whenReleased(() => pool.render.openSession({ clipId, clipInfo })));
    // The worker keeps what the events were found in, for the next word edit (D-146).
    answered(await pool.render.detect({ transcript, prosody, wordEdits: edit.word_edits }));
    const report = answered(await pool.render.setScene({ transcript, plan, edit, timeMap, profile: "preview" }));
    setChunks(report.chunks);
  } catch (error) {
    // The reload's own failure is the one to show now.
    clearFailure();
    stored(error);
    return;
  }
  clearFailure();
  // The player draws a new canvas and hands it over, as it does when it first appears.
  previewStore.newCanvas();
}

/**
 * "Reload preview" (D-153): a preview that stopped, because the graphics
 * device was lost or the render worker crashed, comes back with the clip's
 * edits and without processing again. A fresh render worker opens the
 * clip's session and sets its scene; the page then attaches a new canvas.
 * A second call while one runs gets the same promise; a failure is stored
 * where the page shows it, as any failure of the preview is.
 */
export function reloadPreview(): Promise<void> {
  reloading ??= reload().finally(() => {
    reloading = undefined;
  });
  return reloading;
}
