// Where the caption block sits (v4implementation D-171): one placement per
// clip, from 0 (the top of its track) to 1 (the bottom, the default). The
// drag on the preview and the arrow keys commit here once, on release
// (D-183); so does the Framing tab's control (Prompt 116).

import { useClipStore } from "../state/clip-store";
import { update } from "./review-update";

/** Sets the placement `p`, held to 0..1. A placement the clip has already changes nothing. */
export function setCaptionPosition(p: number): Promise<void> {
  const placement = Math.min(Math.max(p, 0), 1);
  if (useClipStore.getState().edit.caption_position === placement) {
    return Promise.resolve();
  }
  return update((edit) => ({ ...edit, caption_position: placement }), "scene", "caption_position");
}
