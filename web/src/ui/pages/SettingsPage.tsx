// `/settings`. V1 has the "What leaves your device" table, which the landing
// page links to from its first day. The rest of PS §12.7 arrives in V8.
// V6.2 (P194, TS §40.8): with `ai_plan` on, the "AI edit plan" switch and the
// line that says what it sends, above the table.
// V6.2 (P208, D-486): with `motion` on, "Back to recommended": the next clip
// starts from the recommended Enhance settings again.

import { useState } from "react";
import { Link } from "react-router";

import { FEATURES } from "../../config/features";
import { messages } from "../../copy/messages";
import { usePlanStore } from "../../state/plan-store";
import { chooseAiPlan } from "../../usecases/set-ai-plan";
import { resetEnhance } from "../../usecases/set-enhance";
import { Button } from "../primitives/Button";
import { WhatLeavesTable } from "../components/WhatLeavesTable";
import { Switch } from "../primitives/Switch";
import styles from "../styles/pages.module.css";

function AiPlanSetting() {
  const switchOn = usePlanStore((state) => state.switchOn);
  const copy = messages.plan.settings;
  return (
    <div className={styles.setting} data-testid="ai-plan-setting">
      <Switch checked={switchOn} label={copy.label} hint={copy.help} onChange={chooseAiPlan} data-testid="ai-plan-switch" />
    </div>
  );
}

/** "Back to recommended" (D-486): it forgets the Enhance panel's last state; the clip open now keeps its own. */
function EnhanceSetting() {
  const [done, setDone] = useState(false);
  const copy = messages.plan.enhance;
  return (
    <div className={styles.setting} data-testid="enhance-setting">
      <h2 className={styles.heading}>{copy.title}</h2>
      <p className={styles.muted}>{copy.resetHint}</p>
      <Button
        variant="secondary"
        onClick={() => {
          void resetEnhance().then(() => {
            setDone(true);
          });
        }}
        data-testid="enhance-reset"
      >
        {copy.reset}
      </Button>
      <p className={styles.muted} role="status">
        {done ? copy.resetDone : ""}
      </p>
    </div>
  );
}

export function SettingsPage({ homePath }: { homePath: string }) {
  return (
    <main className={styles.page} data-theme="dark">
      <section className={styles.section}>
        <Link to={homePath}>{messages.settings.backLink}</Link>
        <h1 className={styles.title}>{messages.settings.title}</h1>
        {FEATURES.ai_plan ? <AiPlanSetting /> : null}
        {FEATURES.motion ? <EnhanceSetting /> : null}
        <WhatLeavesTable />
      </section>
    </main>
  );
}
