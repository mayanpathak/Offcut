// The caption style (v4implementation D-186): Clean, Bold or Mono, chosen
// from the Style tab's cards, through the update rule in mode `scene`.

import type { StyleId } from "../gen/domain";
import { useClipStore } from "../state/clip-store";
import { update } from "./review-update";

/** Sets the style. The style the clip has already changes nothing. */
export function setStyle(style: StyleId): Promise<void> {
  if (useClipStore.getState().edit.style === style) {
    return Promise.resolve();
  }
  return update((edit) => ({ ...edit, style }), "scene", "style");
}
