// J12: the one upgrade prompt (PS §10, §11; v6implementation §13, INV-14): after a
// watermarked export, or at the limit. It sits beside the outcome, not in it, and
// is no modal: it takes no focus and does not hold anything up (D-302). The way to
// buy arrives with the checkout (P148).

import { PRICES_USD } from "../../config/pricing";
import { messages } from "../../copy/messages";
import { LIMITS } from "../../gen/domain";
import { useExportStore } from "../../state/export-store";
import { dismissUpgrade } from "../../usecases/start-export";
import { Button } from "../primitives/Button";
import styles from "../styles/components.module.css";

export function UpgradePrompt() {
  const atLimit = useExportStore((state) => state.gate === "upgrade");
  const offered = useExportStore((state) => state.upgradeOffer);
  if (!atLimit && !offered) {
    return null;
  }
  const { afterExport, limitReached, dismiss } = messages.upgrade;
  return (
    <section className={styles.cardCompact} data-testid="upgrade-prompt" data-placement={atLimit ? "limit_reached" : "after_export"}>
      <p className={styles.cardText}>
        {atLimit ? limitReached(LIMITS.FREE_EXPORTS_PER_MONTH, PRICES_USD.creatorMonthly) : afterExport(PRICES_USD.creatorMonthly)}
      </p>
      <div className={styles.cardActions}>
        <Button variant="secondary" size="sm" onClick={dismissUpgrade}>
          {dismiss}
        </Button>
      </div>
    </section>
  );
}
