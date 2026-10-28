// The editor's top bar (v4implementation D-178): the product's mark and the
// clip's length on the left; on the right the save state, undo and redo, a
// way to start over, the shortcuts sheet's button, and Export, the one
// primary action. Undo and redo are enabled by the history (D-184) while the
// clip is ready and no export has the preview; their tooltips name their
// shortcuts (D-185). The save state is the interface store's, "Saving…"
// while an edit is written to the device.

import { messages } from "../../copy/messages";
import { type BlockerCode, forImport } from "../../state/blockers";
import { useClipStore } from "../../state/clip-store";
import { useExportStore } from "../../state/export-store";
import { useHistoryStore } from "../../state/history";
import { usePreviewStore } from "../../state/preview-store";
import { useUiStore } from "../../state/ui-store";
import { dismissClip } from "../../usecases/import-clip";
import { ExportButton } from "../components/ExportButton";
import { stepHistory } from "../components/ToastHost";
import { Check, Keyboard, Redo, Undo } from "../icons";
import { Button } from "../primitives/Button";
import { IconButton } from "../primitives/IconButton";
import { Spinner } from "../primitives/Spinner";
import styles from "../styles/editor.module.css";
import type { StageView } from "./Stage";

/** Why the clip cannot be let go now. While an export runs, the words of §17.2. */
function waitText(code: BlockerCode): string | undefined {
  if (code === "B_EXPORT_IN_PROGRESS") {
    return messages.editor.availableAfterExport;
  }
  const copy: Partial<Record<BlockerCode, string>> = messages.blockers;
  return copy[code];
}

/** Lets go of the clip and brings the drop zone back. It waits for an export that runs. */
function StartOver() {
  // Read so that the button is drawn again when an export starts and when it ends.
  useExportStore((state) => state.status);
  const blocker = forImport();
  const reason = blocker === null ? undefined : waitText(blocker);
  return (
    <Button
      variant="ghost"
      disabled={blocker !== null}
      {...(reason === undefined ? {} : { disabledReason: reason })}
      onClick={() => {
        void dismissClip();
      }}
    >
      {messages.editor.startOver}
    </Button>
  );
}

/** Undo and redo (D-184): each enabled when the history has a step and the clip can take a change. */
function UndoRedo({ ready }: { ready: boolean }) {
  const canUndo = useHistoryStore((state) => state.canUndo);
  const canRedo = useHistoryStore((state) => state.canRedo);
  const locked = usePreviewStore((state) => state.status === "locked");
  const usable = ready && !locked;
  const reason = locked ? { disabledReason: messages.editor.availableAfterExport } : {};
  return (
    <span className={styles.group}>
      <IconButton
        icon={<Undo />}
        label={messages.editor.undo}
        disabled={!usable || !canUndo}
        {...reason}
        shortcut={{ keys: [messages.keys.z], mod: true }}
        onClick={() => {
          void stepHistory("undo");
        }}
        data-testid="undo"
      />
      <IconButton
        icon={<Redo />}
        label={messages.editor.redo}
        disabled={!usable || !canRedo}
        {...reason}
        shortcut={{ keys: [messages.keys.shift, messages.keys.z], mod: true }}
        onClick={() => {
          void stepHistory("redo");
        }}
        data-testid="redo"
      />
    </span>
  );
}

function SaveState() {
  const save = useUiStore((state) => state.save);
  return (
    <span className={styles.save}>
      {save === "saving" ? <Spinner size="sm" /> : <Check size={16} />}
      {save === "saving" ? messages.editor.saving : messages.editor.saved}
    </span>
  );
}

export function TopBar({ view }: { view: StageView }) {
  const lengthMs = useClipStore((state) => state.clipInfo?.duration);
  const showShortcuts = useUiStore((state) => state.showShortcuts);
  const ready = view === "ready";
  // A clip is on its way or ready: the controls that act on it are shown, disabled until they can.
  const working = ready || view === "processing" || view === "download" || view === "cancelled";
  return (
    <header className={styles.topBar}>
      <span className={styles.mark}>{messages.editor.mark}</span>
      {lengthMs === undefined ? null : (
        <span className={styles.length}>
          <span className={styles.srOnly}>{messages.editor.lengthLabel(lengthMs)}</span>
          <span aria-hidden="true">{messages.preview.clock(lengthMs)}</span>
        </span>
      )}
      <span className={styles.spacer} />
      {ready ? <SaveState /> : null}
      {working ? <UndoRedo ready={ready} /> : null}
      {ready ? <StartOver /> : null}
      <IconButton
        icon={<Keyboard />}
        label={messages.shortcuts.open}
        shortcut={{ keys: [messages.keys.question] }}
        onClick={() => {
          showShortcuts(true);
        }}
        data-testid="shortcuts-open"
      />
      <ExportButton />
    </header>
  );
}
