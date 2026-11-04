// J12: the one upgrade prompt (PS §10, §11; v6implementation §13, INV-14): after a
// watermarked export, once per export, or at the limit. It sits beside the
// outcome, not in it, and is no modal: it takes no focus and holds nothing up
// (D-302). Each offer goes to the merchant's checkout (TS C-11); there is no
// founding offer while it is off (D-303).

import { useState } from "react";

import { PRICES_USD } from "../../config/pricing";
import { messages } from "../../copy/messages";
import type { Offer, PromptPlacement } from "../../gen/api";
import { type ErrorCode, LIMITS } from "../../gen/domain";
import { useExportStore } from "../../state/export-store";
import { dismissClip } from "../../usecases/import-clip";
import { startCheckout } from "../../usecases/start-checkout";
import { dismissUpgrade } from "../../usecases/start-export";
import { Button } from "../primitives/Button";
import styles from "../styles/components.module.css";

type Status = { kind: "idle" } | { kind: "opening"; offer: Offer } | { kind: "failed"; code: ErrorCode };

function statusText(status: Status): string {
  if (status.kind === "idle") {
    return "";
  }
  if (status.kind === "opening") {
    return messages.upgrade.opening;
  }
  const copy: Partial<Record<ErrorCode, { title: string; action: string }>> = messages.errors;
  const words = copy[status.code];
  return words === undefined ? messages.editor.failedStub : `${words.title}. ${words.action}`;
}

/** One showing of the prompt: what its line says starts afresh with the next. */
function Prompt({ placement }: { placement: PromptPlacement }) {
  const [status, setStatus] = useState<Status>({ kind: "idle" });
  const atLimit = placement === "limit_reached";
  const { afterExport, limitReached, monthly, annual, anotherClip, dismiss } = messages.upgrade;
  const choose = (offer: Offer) => {
    if (status.kind === "opening") {
      return;
    }
    setStatus({ kind: "opening", offer });
    void startCheckout(offer, placement).then((outcome) => {
      // On success the page is leaving for the merchant's: the line says so until it has.
      if (!outcome.ok) {
        setStatus({ kind: "failed", code: outcome.code });
      }
    });
  };
  const opening = (offer: Offer) => status.kind === "opening" && status.offer === offer;
  return (
    <section className={styles.cardCompact} data-testid="upgrade-prompt" data-placement={placement}>
      <p className={styles.cardText}>
        {atLimit ? limitReached(LIMITS.FREE_EXPORTS_PER_MONTH, PRICES_USD.creatorMonthly) : afterExport(PRICES_USD.creatorMonthly)}
      </p>
      <div className={styles.cardActions}>
        <Button
          variant="primary"
          size="sm"
          busy={opening("creator_monthly")}
          onClick={() => {
            choose("creator_monthly");
          }}
        >
          {monthly(PRICES_USD.creatorMonthly)}
        </Button>
        <Button
          variant="secondary"
          size="sm"
          busy={opening("creator_annual")}
          onClick={() => {
            choose("creator_annual");
          }}
        >
          {annual(PRICES_USD.creatorAnnual)}
        </Button>
      </div>
      <div className={styles.cardActions}>
        {atLimit ? null : (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => {
              void dismissClip();
            }}
          >
            {anotherClip}
          </Button>
        )}
        <Button variant="ghost" size="sm" onClick={dismissUpgrade}>
          {dismiss}
        </Button>
      </div>
      <p className={styles.cardHint} role="status" data-testid="upgrade-status">
        {statusText(status)}
      </p>
    </section>
  );
}

export function UpgradePrompt() {
  const atLimit = useExportStore((state) => state.gate === "upgrade");
  const offered = useExportStore((state) => state.upgradeOffer);
  const exportId = useExportStore((state) => state.exportId);
  if (!atLimit && !offered) {
    return null;
  }
  const placement: PromptPlacement = atLimit ? "limit_reached" : "after_export";
  return <Prompt key={`${placement} ${exportId ?? ""}`} placement={placement} />;
}
