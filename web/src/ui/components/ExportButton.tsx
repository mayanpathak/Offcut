// J9: the export button, the one primary action of the editor's top bar
// (v4implementation D-178). V2 has no account and no counter: the button
// starts an export with the token this device holds, and says so when it
// holds none (v2implementation D-23). While something stands in the way, the
// button is disabled and its tooltip says what.

import { messages } from "../../copy/messages";
import { type BlockerCode, forExport } from "../../state/blockers";
import { useClipStore } from "../../state/clip-store";
import { useExportStore } from "../../state/export-store";
import { startExport } from "../../usecases/start-export";
import { Download } from "../icons";
import { Button } from "../primitives/Button";
import styles from "../styles/components.module.css";

/** The words of a blocker. The ones no person can meet yet have none (D-57). */
function blockerText(code: BlockerCode): string | undefined {
  const copy: Partial<Record<BlockerCode, string>> = messages.blockers;
  return copy[code];
}

export function ExportButton() {
  const clipId = useClipStore((state) => state.clipId);
  const unavailable = useExportStore((state) => state.unavailable);
  // Read so that the button is drawn again when what `forExport()` asks about changes.
  useClipStore((state) => state.status);
  useExportStore((state) => state.status);
  const blocker = forExport();
  const reason = blocker === null ? undefined : blockerText(blocker);

  return (
    <div className={styles.exportPanel}>
      <p className={styles.exportStatus} role="status" data-testid="export-status">
        {unavailable ? messages.export.unavailable : ""}
      </p>
      <Button
        variant="primary"
        icon={<Download />}
        disabled={blocker !== null || clipId === undefined}
        {...(reason === undefined ? {} : { disabledReason: reason })}
        onClick={() => {
          if (clipId !== undefined) {
            void startExport(clipId);
          }
        }}
      >
        {messages.export.button}
      </Button>
    </div>
  );
}
