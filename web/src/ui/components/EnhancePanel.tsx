// V6.2: the Enhance panel (v6.2implementation §11, D-485 to D-488; TS §47.5),
// under the Style tab's cards, with the `motion` flag on. Two switches, each
// with its line: Voice cleanup (off: the levelled voice, D-487) and Motion
// (off: no camera drift; the plan's zooms stay). Each is one review change,
// remembered for the next clip (set-enhance.ts). While an export runs they
// cannot be changed (TS §12.4).

import { messages } from "../../copy/messages";
import { useClipStore } from "../../state/clip-store";
import { usePreviewStore } from "../../state/preview-store";
import { ENHANCE_V5, setEnhance } from "../../usecases/set-enhance";
import { PanelSection } from "../primitives/PanelSection";
import { Switch } from "../primitives/Switch";

export function EnhancePanel() {
  const enhance = useClipStore((state) => state.edit.enhance) ?? ENHANCE_V5;
  const locked = usePreviewStore((state) => state.status) === "locked";
  const reason = locked ? { disabledReason: messages.editor.availableAfterExport } : {};
  const copy = messages.plan.enhance;
  return (
    <PanelSection title={copy.title}>
      <div data-testid="enhance-panel">
        <Switch
          checked={enhance.voice === "clean"}
          label={copy.voice.label}
          hint={copy.voice.help}
          disabled={locked}
          {...reason}
          onChange={(on) => {
            void setEnhance({ voice: on ? "clean" : "level" });
          }}
          data-testid="enhance-voice"
        />
        <Switch
          checked={enhance.motion}
          label={copy.motion.label}
          hint={copy.motion.help}
          disabled={locked}
          {...reason}
          onChange={(on) => {
            void setEnhance({ motion: on });
          }}
          data-testid="enhance-motion"
        />
      </div>
    </PanelSection>
  );
}
