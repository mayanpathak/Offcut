// The Style tab (v4implementation D-186): the three cards, drawn from the
// clip at the preview's position, and the Highlight numbers switch (D-169).
// The cards are drawn when the tab opens and again, debounced, when what the
// frame shows changes: the position, the words, the placement, the framing,
// the visuals or the highlight. While an export runs they cannot be chosen
// (TS §12.4).

import { useEffect, useRef } from "react";

import { messages } from "../../copy/messages";
import { useClipStore } from "../../state/clip-store";
import { usePreviewStore } from "../../state/preview-store";
import { useUiStore } from "../../state/ui-store";
import { setHighlightNumbers } from "../../usecases/set-highlight-numbers";
import { setStyle } from "../../usecases/set-style";
import { forgetThumbnails, refreshThumbnails } from "../../usecases/style-thumbnails";
import { PanelSection } from "../primitives/PanelSection";
import { Switch } from "../primitives/Switch";
import { StylePicker } from "./StylePicker";

export function StylePanel() {
  const edit = useClipStore((state) => state.edit);
  const clip = useClipStore((state) => state.status);
  const position = usePreviewStore((state) => state.position);
  const preview = usePreviewStore((state) => state.status);
  const seekBusy = usePreviewStore((state) => state.seekBusy);
  const thumbnails = useUiStore((state) => state.thumbnails);
  const locked = preview === "locked";
  const reason = locked ? { disabledReason: messages.editor.availableAfterExport } : {};

  // Drawn at once when the tab opens; let go when it closes.
  useEffect(() => {
    refreshThumbnails(true);
    return forgetThumbnails;
  }, []);

  // Again when the frame would look different, or when a paused, ready clip can be drawn again;
  // not for the values the tab opened with, which the opening's draw shows.
  const { word_edits, event_overrides, caption_position, crop_offset, highlight_numbers } = edit;
  const shown = JSON.stringify([position, word_edits, event_overrides, caption_position, crop_offset, highlight_numbers]);
  const state = `${clip} ${preview} ${String(seekBusy)} ${shown}`;
  const drawnFor = useRef(state);
  useEffect(() => {
    if (drawnFor.current !== state) {
      drawnFor.current = state;
      refreshThumbnails();
    }
  }, [state]);

  return (
    <PanelSection title={messages.panel.style}>
      <StylePicker
        value={edit.style}
        thumbnails={thumbnails}
        onChoose={(style) => {
          void setStyle(style);
        }}
        {...reason}
      />
      <Switch
        checked={edit.highlight_numbers}
        label={messages.style.highlightNumbers}
        hint={messages.style.highlightNumbersHint}
        disabled={locked}
        {...reason}
        onChange={(on) => {
          void setHighlightNumbers(on);
        }}
        data-testid="highlight-numbers"
      />
    </PanelSection>
  );
}
