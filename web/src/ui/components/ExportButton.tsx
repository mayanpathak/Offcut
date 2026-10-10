// J9: the export button. V2 has no account and no counter: the button starts
// an export with the token this device holds, and says so when it holds none
// (v2implementation D-23).

import { messages } from "../../copy/messages";
import { type BlockerCode, forExport } from "../../state/blockers";
import { useClipStore } from "../../state/clip-store";
import { useExportStore } from "../../state/export-store";
import { startExport } from "../../usecases/start-export";
import styles from "../styles/components.module.css";

/** The words of a blocker. The ones no person can meet yet have none (D-57). */
function blockerText(code: BlockerCode): string {
  const copy: Partial<Record<BlockerCode, string>> = messages.blockers;
  return copy[code] ?? "";
}

export function ExportButton() {
  const clipId = useClipStore((state) => state.clipId);
  const unavailable = useExportStore((state) => state.unavailable);
  // Read so that the button is drawn again when what `forExport()` asks about changes.
  useClipStore((state) => state.status);
  useExportStore((state) => state.status);
  const blocker = forExport();

  let line = "";
  if (blocker !== null) {
    line = blockerText(blocker);
  } else if (unavailable) {
    line = messages.export.unavailable;
  }

  return (
    <div className={styles.exportPanel}>
      <button
        className={styles.button}
        type="button"
        disabled={blocker !== null || clipId === undefined}
        onClick={() => {
          if (clipId !== undefined) {
            void startExport(clipId);
          }
        }}
      >
        {messages.export.button}
      </button>
      <p className={styles.status} role="status" data-testid="export-status">
        {line}
      </p>
    </div>
  );
}
