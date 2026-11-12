// V6.2: the settings switch "AI edit plan" was changed (v6.2implementation
// D-466, §8; TS §40.8). The choice holds for this page at once, and for the
// next visit once it is written. The next run reads it; a run already
// asking keeps its request. Off: no clip sends its words for a plan.

import { setAiPlanSwitch } from "../persistence/prefs-repo";
import { setPlanSwitch } from "../state/plan-store";

export function chooseAiPlan(on: boolean): void {
  setPlanSwitch(on);
  void setAiPlanSwitch(on);
}
