// The crop of a wide clip (v4implementation D-142, D-187): which part of the
// frame the 9:16 picture shows, from the far left (-1) through the centre to
// the far right (1), step 0.01, committed when the slider is let go. The
// keyboard's and the panel's way; dragging the video on the preview is the
// other (D-183). Shown for landscape and square clips only.

import { messages } from "../../copy/messages";
import { cropMoves, useClipStore } from "../../state/clip-store";
import { usePreviewStore } from "../../state/preview-store";
import { setCropOffset } from "../../usecases/set-crop-offset";
import { PanelSection } from "../primitives/PanelSection";
import { Slider } from "../primitives/Slider";

export function CropOffsetControl() {
  const info = useClipStore((state) => state.clipInfo);
  const offset = useClipStore((state) => state.edit.crop_offset);
  const locked = usePreviewStore((state) => state.status === "locked");
  if (info === undefined || !cropMoves(info)) {
    return null;
  }
  return (
    <PanelSection title={messages.framing.crop} hint={messages.framing.dragVideo}>
      <Slider
        label={messages.framing.crop}
        min={-1}
        max={1}
        step={0.01}
        value={offset}
        format={messages.framing.cropValue}
        disabled={locked}
        {...(locked ? { disabledReason: messages.editor.availableAfterExport } : {})}
        onCommit={(value) => {
          void setCropOffset(value);
        }}
        data-testid="crop-offset"
      />
    </PanelSection>
  );
}
