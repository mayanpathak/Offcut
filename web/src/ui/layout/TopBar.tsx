// The editor's top bar (v4implementation D-178): the product's mark and the
// clip's length on the left; on the right the save state, undo and redo, a
// way to start over, and Export, the one primary action. Undo and redo stay
// disabled until Prompt 114 gives them a history; the save state is the
// interface store's, "saved" until edits write to the device.

import { messages } from "../../copy/messages";
import { type BlockerCode, forImport } from "../../state/blockers";
import { useClipStore } from "../../state/clip-store";
import { useExportStore } from "../../state/export-store";
import { useUiStore } from "../../state/ui-store";
import { dismissClip } from "../../usecases/import-clip";
import { ExportButton } from "../components/ExportButton";
import { Check, Redo, Undo } from "../icons";
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
      {working ? (
        <span className={styles.group}>
          <IconButton icon={<Undo />} label={messages.editor.undo} disabled />
          <IconButton icon={<Redo />} label={messages.editor.redo} disabled />
        </span>
      ) : null}
      {ready ? <StartOver /> : null}
      <ExportButton />
    </header>
  );
}
