// The Style tab's three cards (v4implementation D-186): the clip's frame at
// the preview's position, drawn by the render worker in Clean, Bold and Mono
// with the current edits. Asked at once when the tab opens, then again
// `THUMB_DEBOUNCE_MS` after the last change of what the frame shows; the
// latest call wins, and an answer that comes after a newer call, or after the
// tab has closed, is closed and dropped. Drawn only while the clip is ready
// and the preview paused or stopped: a call meanwhile waits for the next
// change of state (the panel asks again).

import { useClipStore } from "../state/clip-store";
import { usePreviewStore } from "../state/preview-store";
import { useUiStore } from "../state/ui-store";
import { pool, WorkerCallError } from "../workers/pool";

/** A change of what the frame shows is drawn this long after the last one (Appendix D). */
export const THUMB_DEBOUNCE_MS = 300;
/** The cards' frames, 9:16 (D-186). */
export const THUMB_WIDTH = 216;
export const THUMB_HEIGHT = 384;

let timer: ReturnType<typeof setTimeout> | undefined;
/** Goes up with every call: an answer of an older one is dropped. */
let turn = 0;

/** Whether the worker can draw the cards now. */
function drawable(): boolean {
  const { status, seekBusy } = usePreviewStore.getState();
  return (
    useClipStore.getState().status === "ready" &&
    (status === "paused" || status === "stopped") &&
    !seekBusy &&
    useUiStore.getState().tab === "style"
  );
}

async function draw(mine: number): Promise<void> {
  const at = usePreviewStore.getState().position;
  if (!drawable() || at === undefined) {
    return;
  }
  let frames: ImageBitmap[];
  try {
    const answer = await pool.render.styleThumbnails({ at, width: THUMB_WIDTH, height: THUMB_HEIGHT });
    if ("cancelled" in answer) {
      return;
    }
    frames = answer;
  } catch (error) {
    if (!(error instanceof WorkerCallError)) {
      throw error;
    }
    // The cards keep what they show (their skeletons at first); the next change asks again.
    return;
  }
  if (mine !== turn || useUiStore.getState().tab !== "style") {
    for (const frame of frames) {
      frame.close();
    }
    return;
  }
  useUiStore.getState().setThumbnails(frames);
}

/** Draws the cards again: at once (`now`, the tab has opened) or after the debounce. */
export function refreshThumbnails(now = false): void {
  clearTimeout(timer);
  turn += 1;
  const mine = turn;
  timer = setTimeout(
    () => {
      void draw(mine);
    },
    now ? 0 : THUMB_DEBOUNCE_MS,
  );
}

/** The tab has closed: nothing pending is drawn, and the frames are let go. */
export function forgetThumbnails(): void {
  clearTimeout(timer);
  turn += 1;
  useUiStore.getState().setThumbnails(null);
}
