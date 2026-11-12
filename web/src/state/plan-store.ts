// V6.2: the settings switch "AI edit plan" on this page (v6.2implementation
// D-466, §8; TS §40.8: on by default). It is kept in IndexedDB's `meta` by
// `persistence/prefs-repo.ts`; this store holds it for the page. With the
// flag `ai_plan` off nothing reads or writes it.

import { create } from "zustand";

export type PlanSwitchState = {
  /** Whether a clip may ask for an AI plan. Read when a run reaches its prosody. */
  switchOn: boolean;
};

export const usePlanStore = create<PlanSwitchState>(() => ({ switchOn: true }));

export function setPlanSwitch(switchOn: boolean): void {
  usePlanStore.setState({ switchOn });
}
