// Whether this browser can run Offcut (TS §12.2): unchecked → checking →
// supported | unsupported. `unsupported` is final until the page is reloaded.

import { create } from "zustand";

import type { AnalyticsEvent } from "../gen/api";
import type { UnsupportedReason } from "../gen/domain";
import { type MachineDef, transition } from "./machines/transition";

/** The props of the `capability_check` event, so the store, the check and the event share one shape. */
export type CapabilityReport = Extract<AnalyticsEvent, { name: "capability_check" }>["props"];

export type CapabilityStatus = "unchecked" | "checking" | "supported" | "unsupported";

export type CapabilityState = {
  status: CapabilityStatus;
  reason?: UnsupportedReason;
  report?: CapabilityReport;
};

type CapabilityEvent = "begin" | "pass" | "fail";

const MACHINE: MachineDef<CapabilityStatus, CapabilityEvent> = {
  unchecked: { begin: "checking" },
  checking: { pass: "supported", fail: "unsupported" },
  supported: {},
  unsupported: {},
};

export const useCapabilityStore = create<CapabilityState>(() => ({ status: "unchecked" }));

/** Applies `event`; `change` is stored only if the transition was legal. */
function apply(event: CapabilityEvent, change: Omit<CapabilityState, "status">): void {
  const current = useCapabilityStore.getState().status;
  const status = transition("capability", MACHINE, current, event);
  // No legal transition leads back to the same state: an unchanged state
  // means the attempt was illegal and was reported.
  if (status !== current) {
    useCapabilityStore.setState({ status, ...change });
  }
}

/** unchecked → checking */
export function beginCheck(): void {
  apply("begin", {});
}

/** checking → supported */
export function setSupported(report: CapabilityReport): void {
  apply("pass", { report });
}

/** checking → unsupported */
export function setUnsupported(reason: UnsupportedReason, report: CapabilityReport): void {
  apply("fail", { reason, report });
}
