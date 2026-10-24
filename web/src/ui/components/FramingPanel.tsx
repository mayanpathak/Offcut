// The Framing tab (v4implementation D-187): where the captions sit, Top ·
// Middle · Bottom (the snap points of the drag, D-171), and the crop of a
// wide clip. A placement dragged between two snaps shows the nearest one.
// The panel is the keyboard's way; dragging on the preview is the other.

import { messages } from "../../copy/messages";
import { useClipStore } from "../../state/clip-store";
import { usePreviewStore } from "../../state/preview-store";
import { CAPTION_SNAPS, type Snap } from "../../usecases/preview-interact";
import { setCaptionPosition } from "../../usecases/set-caption-position";
import { PanelSection } from "../primitives/PanelSection";
import { SegmentedControl } from "../primitives/SegmentedControl";
import { CropOffsetControl } from "./CropOffsetControl";

/** The snap point nearest the placement `p`. */
function nearest(p: number): Snap {
  let best: (typeof CAPTION_SNAPS)[number] = CAPTION_SNAPS[0];
  for (const snap of CAPTION_SNAPS) {
    if (Math.abs(snap.at - p) < Math.abs(best.at - p)) {
      best = snap;
    }
  }
  return best.name;
}

export function FramingPanel() {
  const placement = useClipStore((state) => state.edit.caption_position);
  const locked = usePreviewStore((state) => state.status === "locked");
  return (
    <>
      <PanelSection title={messages.framing.captionPosition} hint={messages.framing.dragCaptions}>
        <SegmentedControl
          label={messages.framing.captionPosition}
          value={nearest(placement)}
          disabled={locked}
          {...(locked ? { disabledReason: messages.editor.availableAfterExport } : {})}
          onChange={(name) => {
            const snap = CAPTION_SNAPS.find((s) => s.name === name);
            if (snap !== undefined) {
              void setCaptionPosition(snap.at);
            }
          }}
          options={CAPTION_SNAPS.map((snap) => ({ value: snap.name, label: messages.framing[snap.name] }))}
        />
      </PanelSection>
      <CropOffsetControl />
    </>
  );
}
