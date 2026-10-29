// Which part of a wide frame the 9:16 picture shows (v4implementation D-142):
// from the far left (-1) through the centre (0) to the far right (1). The
// drag of the video on the preview commits here once, on release (D-183);
// so does the Framing tab's slider (Prompt 116). A frame that is not
// wider than 9:16 has no offset to set.

import { cropMoves, useClipStore } from "../state/clip-store";
import { update } from "./review-update";

/** Sets the offset `o`, held to -1..1, on a landscape or square clip. */
export function setCropOffset(o: number): Promise<void> {
  const { clipInfo, edit } = useClipStore.getState();
  const offset = Math.min(Math.max(o, -1), 1);
  if (clipInfo === undefined || !cropMoves(clipInfo) || edit.crop_offset === offset) {
    return Promise.resolve();
  }
  return update((current) => ({ ...current, crop_offset: offset }), "scene", "crop_offset");
}
