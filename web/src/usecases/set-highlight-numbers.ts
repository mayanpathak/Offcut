// Whether the numbers of the captions take the accent colour
// (v4implementation D-169), the Style tab's switch, through the update rule
// in mode `scene`.

import { useClipStore } from "../state/clip-store";
import { update } from "./review-update";

/** Turns the highlight on or off. The value the clip has already changes nothing. */
export function setHighlightNumbers(on: boolean): Promise<void> {
  if (useClipStore.getState().edit.highlight_numbers === on) {
    return Promise.resolve();
  }
  return update((edit) => ({ ...edit, highlight_numbers: on }), "scene", "highlight_numbers");
}
