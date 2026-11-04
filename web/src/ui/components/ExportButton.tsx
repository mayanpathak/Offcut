// J9: the export button, the one primary action of the editor's top bar
// (v4implementation D-178). Disabled, with a tooltip that says why, only for what
// a click could not change; a click on any other blocker opens what mends it
// (v6implementation D-290). The counter stands beside it, before the click, and
// outside its accessible name and the status line (D-302). An export the gate
// stopped says why beside it, one action away from trying again (v5implementation D-231).

import { useEffect } from "react";

import { messages } from "../../copy/messages";
import { LIMITS } from "../../gen/domain";
import { useAuthStore } from "../../state/auth-store";
import { type BlockerCode, disablesExport, effectiveNow, forExport } from "../../state/blockers";
import { useClipStore } from "../../state/clip-store";
import { useEntitlementStore } from "../../state/entitlement-store";
import { useExportStore } from "../../state/export-store";
import { closeGate, confirmLastExport, startExport } from "../../usecases/start-export";
import { Download } from "../icons";
import { Button } from "../primitives/Button";
import styles from "../styles/components.module.css";
import { GateDialog, SignInDialog } from "./SignInDialog";

/** The words of a blocker. */
function blockerText(code: BlockerCode): string {
  return messages.blockers[code];
}

/** "2 of 3 left", on Free with an account behind it. A signed-out person meets the limits in the dialog (§13). */
function Counter() {
  const status = useAuthStore((state) => state.status);
  // Read so that the counter is drawn again when what `effectiveNow()` asks about changes.
  useEntitlementStore((state) => state.claims);
  useEntitlementStore((state) => state.outboxFree);
  const now = effectiveNow();
  if (now?.plan !== "free" || (status !== "signed_in" && status !== "anonymous_offline")) {
    return null;
  }
  return (
    <span className={styles.exportCounter} data-testid="export-counter">
      {messages.export.counter(now.freeExportsRemaining, LIMITS.FREE_EXPORTS_PER_MONTH)}
    </span>
  );
}

/** Before the third of three (PS §11): said before the click that spends it. */
function LastExportDialog({ onExport }: { onExport: () => void }) {
  const { title, body, go, notNow } = messages.export.lastFree;
  return (
    <GateDialog title={title} testId="last-export-dialog" onClose={closeGate}>
      <p className={styles.cardText}>{body}</p>
      <div className={styles.cardActions}>
        <Button variant="primary" onClick={onExport}>
          {go}
        </Button>
        <Button variant="secondary" onClick={closeGate}>
          {notNow}
        </Button>
      </div>
    </GateDialog>
  );
}

export function ExportButton() {
  const clipId = useClipStore((state) => state.clipId);
  const stopped = useExportStore((state) => state.blocker);
  const gate = useExportStore((state) => state.gate);
  const auth = useAuthStore((state) => state.status);
  // Read so that the button is drawn again when what `forExport()` asks about changes.
  useClipStore((state) => state.status);
  useExportStore((state) => state.status);
  useEntitlementStore((state) => state.claims);
  useEntitlementStore((state) => state.outboxFree);
  const blocker = forExport();
  const disabled = blocker !== null && disablesExport(blocker);

  // Signed in, here or in the tab the link opened: the dialog's work is done, and the person clicks Export again (TS C-8).
  useEffect(() => {
    if (gate === "sign_in" && auth === "signed_in") {
      closeGate();
    }
  }, [gate, auth]);

  return (
    <div className={styles.exportPanel}>
      <p className={styles.exportStatus} role="status" data-testid="export-status">
        {stopped === undefined ? "" : blockerText(stopped)}
      </p>
      <Counter />
      <Button
        variant="primary"
        icon={<Download />}
        disabled={disabled || clipId === undefined}
        {...(disabled ? { disabledReason: blockerText(blocker) } : {})}
        onClick={() => {
          if (clipId !== undefined) {
            void startExport(clipId);
          }
        }}
      >
        {messages.export.button}
      </Button>
      {gate === "sign_in" ? <SignInDialog onClose={closeGate} /> : null}
      {gate === "confirm_last" && clipId !== undefined ? (
        <LastExportDialog
          onExport={() => {
            void confirmLastExport(clipId);
          }}
        />
      ) : null}
    </div>
  );
}
