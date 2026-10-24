// Gestures on the preview (v4implementation D-183): what a press there will
// do, a click on a caption word, and the guides of the two drags, the caption
// block up and down and the video sideways. Hit tests run in the render
// worker (D-193), on a paused or stopped preview only. Nothing here commits:
// a drag's value goes to `setCaptionPosition` or `setCropOffset` on release,
// once (D-142, Appendix A).

import { cropMoves, useClipStore } from "../state/clip-store";
import { usePreviewStore } from "../state/preview-store";
import { useUiStore } from "../state/ui-store";
import { pool, WorkerCallError } from "../workers/pool";
import type { CaptionMirror, Hit } from "../workers/protocol";

export type { CaptionMirror, Hit };

/**
 * The snap points of the caption drag, as placements (D-171). Their home is
 * `crates/offcut-scene/src/placement.rs` (`SNAP_TOP`, `SNAP_MIDDLE`,
 * `SNAP_BOTTOM`): the page cannot import a constant of the scene, so they
 * are mirrored here once, with the track below, and the drag case of
 * `review-edit.spec.ts` guards them (a snap to Top reads 0.15).
 */
export const CAPTION_SNAPS = [
  { name: "top", at: 0.15 },
  { name: "middle", at: 0.55 },
  { name: "bottom", at: 1.0 },
] as const;

/**
 * The caption block's centre at placement 0 and at 1, and its height, as
 * fractions of the canvas's height: `CAPTION_TOP` (390), `CAPTION_BOTTOM`
 * (1,304) and `CAPTION_BLOCK_HEIGHT` (280) of 1,920 logical pixels, in the
 * same file, whose test asserts the first two.
 */
const CAPTION_TRACK = { top: 390 / 1920, bottom: 1304 / 1920, height: 280 / 1920 };

/** A drag lands on a snap point within this many pixels of the screen (D-183). */
export const SNAP_DISTANCE_PX = 16;

export type Snap = (typeof CAPTION_SNAPS)[number]["name"];

/** Where the caption block would go: its placement, and its centre and height as fractions of the canvas. */
export type CaptionGuide = { placement: number; centre: number; height: number; snap: Snap | null };

/** Where the video would go: its offset, and the 9:16 window's place and width as fractions of the source's width. */
export type VideoGuide = { offset: number; left: number; width: number };

/** Whether the preview can be manipulated: a ready clip, a preview attached and neither playing nor scrubbed. */
export function interactive(): boolean {
  const { status: clip } = useClipStore.getState();
  const { status, seekBusy } = usePreviewStore.getState();
  return (clip === "ready" || clip === "updating") && (status === "paused" || status === "stopped") && !seekBusy;
}

/** What `(x, y)` is on now; `null` when the preview cannot answer: playing, or the worker busy drawing a frame. */
export async function hitAt(x: number, y: number): Promise<Hit | null> {
  const at = usePreviewStore.getState().position;
  if (!interactive() || at === undefined) {
    return null;
  }
  try {
    const hit = await pool.render.hitTest({ at, x, y });
    return "cancelled" in hit ? null : hit;
  } catch (error) {
    if (error instanceof WorkerCallError) {
      // One job at a time: the worker refused this one while it drew a frame.
      return null;
    }
    throw error;
  }
}

/** The point the pointer is over, waiting for the next animation frame, and whom to tell what it is on. */
let hoverWanted: { x: number; y: number; show: (hit: Hit | null) => void } | undefined;
let hoverFrame: number | undefined;
let hoverAsking = false;

function hoverTick(): void {
  hoverFrame = undefined;
  const wanted = hoverWanted;
  if (hoverAsking || wanted === undefined) {
    return;
  }
  hoverWanted = undefined;
  hoverAsking = true;
  void hitAt(wanted.x, wanted.y)
    .then(wanted.show)
    .finally(() => {
      hoverAsking = false;
      // A point that came meanwhile is asked on the next frame.
      if (hoverWanted !== undefined) {
        hoverFrame ??= requestAnimationFrame(hoverTick);
      }
    });
}

/**
 * The pointer is over `(x, y)`, fractions of the canvas: `show` is told what
 * it is on. At most one hit test per animation frame and one in flight; a
 * newer point replaces one that waits (D-183).
 */
export function hoverAt(x: number, y: number, show: (hit: Hit | null) => void): void {
  hoverWanted = { x, y, show };
  hoverFrame ??= requestAnimationFrame(hoverTick);
}

/** The pointer has left: a point that waits is dropped. */
export function hoverEnd(): void {
  hoverWanted = undefined;
  if (hoverFrame !== undefined) {
    cancelAnimationFrame(hoverFrame);
    hoverFrame = undefined;
  }
}

/** A click at `(x, y)`: on a caption word, the Captions tab opens on it (D-183). */
export async function clickAt(x: number, y: number): Promise<void> {
  const hit = await hitAt(x, y);
  if (hit?.kind !== "word") {
    return;
  }
  const ui = useUiStore.getState();
  ui.selectTab("captions");
  ui.editWord(hit.word);
}

/** The guide of the caption block at placement `p`, snapped or not. */
export function captionGuide(p: number, snap: Snap | null = null): CaptionGuide {
  const placement = Math.min(Math.max(p, 0), 1);
  const centre = CAPTION_TRACK.top + placement * (CAPTION_TRACK.bottom - CAPTION_TRACK.top);
  return { placement, centre, height: CAPTION_TRACK.height, snap };
}

/**
 * The caption block dragged from `fromY` to `y`, fractions of the canvas
 * whose height on screen is `heightPx`: its centre follows the pointer along
 * its track, and lands on a snap point within `SNAP_DISTANCE_PX` of it.
 */
export function dragCaptions(fromY: number, y: number, heightPx: number): CaptionGuide {
  const start = captionGuide(useClipStore.getState().edit.caption_position);
  const span = CAPTION_TRACK.bottom - CAPTION_TRACK.top;
  const moved = captionGuide((start.centre + y - fromY - CAPTION_TRACK.top) / span);
  for (const snap of CAPTION_SNAPS) {
    const at = captionGuide(snap.at);
    if (Math.abs(at.centre - moved.centre) * heightPx <= SNAP_DISTANCE_PX) {
      return captionGuide(snap.at, snap.name);
    }
  }
  return moved;
}

/** Whether the clip's video can be dragged sideways: a frame wider than 9:16 (D-142). */
export function videoMoves(): boolean {
  const { clipInfo } = useClipStore.getState();
  return clipInfo !== undefined && cropMoves(clipInfo);
}

/**
 * The video dragged from `fromX` to `x`, fractions of the canvas: the
 * picture follows the pointer, so the window moves the other way, by the
 * share of the source the canvas shows (`crop_rect` in
 * `offcut-scene/src/framing.rs`). `null` for a clip that cannot move.
 */
export function dragVideo(fromX: number, x: number): VideoGuide | null {
  const { clipInfo, edit } = useClipStore.getState();
  if (clipInfo === undefined || !cropMoves(clipInfo)) {
    return null;
  }
  const width = (clipInfo.display_height * 9) / 16 / clipInfo.display_width;
  // An offset of 2 moves the window by what the source has beside it.
  const perCanvas = (2 * width) / (1 - width);
  const offset = Math.min(Math.max(edit.crop_offset - (x - fromX) * perCanvas, -1), 1);
  return { offset, left: ((1 - width) / 2) * (1 + offset), width };
}

/**
 * The caption shown now as drawn, for development and test builds (D-155);
 * `undefined` when the worker could not answer (it is asked again on the next change).
 */
export async function captionMirror(): Promise<CaptionMirror | null | undefined> {
  const at = usePreviewStore.getState().position;
  if (!interactive() || at === undefined) {
    return undefined;
  }
  try {
    const mirror = await pool.render.captionMirror({ at });
    return mirror !== null && "cancelled" in mirror ? undefined : mirror;
  } catch (error) {
    if (error instanceof WorkerCallError) {
      return undefined;
    }
    throw error;
  }
}
