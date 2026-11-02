// J11: what the export changed, and only that (PS §10; v5implementation §14), with
// the one question under it, at the top of the side panel while the export is done
// (D-232). It reads numbers from the worker's `ChangeSummary`, so no word of a clip
// is in it. Focus does not move when it appears: the person may be working.

import { useState } from "react";

import { messages } from "../../copy/messages";
import type { ChangeSummary, ExportId } from "../../gen/domain";
import { useExportStore } from "../../state/export-store";
import { Check, X } from "../icons";
import { IconButton } from "../primitives/IconButton";
import { PanelSection } from "../primitives/PanelSection";
import styles from "../styles/components.module.css";
import { PostExportQuestion } from "./PostExportQuestion";

/** The rows of J11, only those that happened (PS §10). One row per field: V6.3 and V6.7 each add one. */
export function summaryRows(s: ChangeSummary): string[] {
  const rows: string[] = [];
  if (s.voice_cleaned) {
    rows.push(messages.export.summary.voice);
  }
  if (s.captions_emphasized > 0) {
    rows.push(messages.export.summary.emphasized(s.captions_emphasized));
  }
  if (s.visual_moments > 0) {
    rows.push(messages.export.summary.visuals(s.visual_moments));
  }
  return rows;
}

/** What the editor's one live region says of a finished export (D-192): the block, or the thanks in its buttons' place. */
export function outcomeAnnouncement(summary: ChangeSummary, answered: boolean): string {
  const { summary: words, question } = messages.export;
  return (answered ? [question.thanks] : [words.title, ...summaryRows(summary), question.title]).join(", ");
}

export function ExportOutcome() {
  const status = useExportStore((state) => state.status);
  const exportId = useExportStore((state) => state.exportId);
  const summary = useExportStore((state) => state.summary);
  // Keyed by the export, so the next one shows again. `null`: nothing was closed.
  const [closed, setClosed] = useState<ExportId | undefined | null>(null);
  if (status !== "done" || summary === undefined || closed === exportId) {
    return null;
  }
  const rows = summaryRows(summary);
  return (
    <div className={styles.outcome} data-testid="export-outcome">
      <PanelSection title={messages.export.summary.title}>
        {rows.length === 0 ? null : (
          <ul className={styles.outcomeRows}>
            {rows.map((row) => (
              <li key={row} className={styles.outcomeRow}>
                <Check size={16} />
                {row}
              </li>
            ))}
          </ul>
        )}
        <PostExportQuestion />
      </PanelSection>
      <div className={styles.outcomeClose}>
        <IconButton
          icon={<X />}
          label={messages.export.summary.close}
          size="sm"
          onClick={() => {
            setClosed(exportId);
          }}
        />
      </div>
    </div>
  );
}
