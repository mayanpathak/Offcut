// One export: where it is, how far, and what went wrong (TS §12.1). Written
// by usecases/start-export.ts through the actions below.

import { create } from "zustand";

import type { ChangeSummary, ExportId } from "../gen/domain";
import type { AppFailure } from "../workers/protocol";
import type { BlockerCode } from "./blockers";
import { EXPORT_MACHINE, type ExportEvent, type ExportStatus } from "./machines/export-machine";
import { transition } from "./machines/transition";

/** What the gate opened instead of an export (v6implementation §12): a dialog, or the prompt at the limit. */
export type ExportGate = "sign_in" | "upgrade" | "confirm_last";

export type ExportState = {
  status: ExportStatus;
  exportId?: ExportId;
  /** Output frames encoded so far, and how many there will be. */
  done: number;
  total: number;
  /** When the export began, in milliseconds since the Unix epoch. */
  startedAt?: number;
  failure?: AppFailure;
  /** Open until it is closed or the next start. No export began, so it is no state of the machine. */
  gate: ExportGate | null;
  /** The gate could not fetch a token (D-310). Nothing began, so `failure`'s state does not apply. */
  gateFailure?: AppFailure;
  /** A finished export on Free offers the upgrade, once per export (PS §11). */
  upgradeOffer: boolean;
  /** Why the last start did not go on (v5implementation D-231): the next start clears it. */
  blocker?: BlockerCode;
  /** What this export changed, once it is done. */
  summary?: ChangeSummary;
  /** Whether this export's question has been answered (D-234). */
  answered: boolean;
};

/** What a state holds beside its status. */
type Fields = Omit<ExportState, "status">;

/** No export. */
function empty(): Fields {
  return { done: 0, total: 0, gate: null, upgradeOffer: false, answered: false };
}

/**
 * The exports whose question was answered, in this tab (D-234). It outlives
 * `reset`: a cache hit shares its export's id, and is not asked again.
 */
const answeredIds = new Set<ExportId>();
/** The exports whose upgrade offer was dismissed, in this tab: a cache hit shares its export's id. */
const dismissedOffers = new Set<ExportId>();

export const useExportStore = create<ExportState>(() => ({ status: "idle", ...empty() }));

/**
 * Applies `event`. `next` gives what the state holds with its new status,
 * from what it holds now. It is stored only if the transition was legal, and
 * it is stored whole: a field `next` leaves out is dropped.
 */
function apply(event: ExportEvent, next: (current: Fields) => Fields = (current) => current): void {
  const { status: from, ...current } = useExportStore.getState();
  const status = transition("export", EXPORT_MACHINE, from, event);
  // No legal transition leads back to the same state: an unchanged state
  // means the attempt was illegal and was reported.
  if (status !== from) {
    useExportStore.setState({ ...next(current), status }, true);
  }
}

/** idle → gating */
export function start(exportId: ExportId): void {
  apply("start", () => ({ ...empty(), exportId, startedAt: Date.now() }));
}

/** gating → idle: a blocker stands in the way. */
export function blocked(): void {
  apply("blocked", empty);
}

/** gating → idle, saying why (D-231). */
export function blockedBy(code: BlockerCode): void {
  apply("blocked", () => ({ ...empty(), blocker: code }));
}

/** gating → rendering */
export function clear(): void {
  apply("clear");
}

/** gating → saving: the file of an earlier export, under its id (TS §21.4). */
export function cacheHit(original: ExportId): void {
  apply("cache_hit", (current) => ({ ...current, exportId: original }));
}

/** How far the rendering is. Not a change of state; kept only while it runs. */
export function progress(done: number, total: number): void {
  if (useExportStore.getState().status === "rendering") {
    useExportStore.setState({ done, total });
  }
}

/** rendering → muxing: the last frame is encoded. */
export function encoded(): void {
  apply("encoded", (current) => ({ ...current, done: current.total }));
}

/** muxing → saving: the file is whole. */
export function finalized(): void {
  apply("finalized");
}

/** saving → done, with what was changed. */
export function saved(summary: ChangeSummary, alreadyAnswered: boolean): void {
  apply("saved", (current) => ({ ...current, summary, answered: alreadyAnswered }));
}

/** The question was answered for this export (D-234). Not a change of state. */
export function answered(): void {
  const { exportId } = useExportStore.getState();
  if (exportId !== undefined) {
    answeredIds.add(exportId);
    useExportStore.setState({ answered: true });
  }
}

/** Whether this export's question has been answered, in this tab (D-234). */
export function hasAnswered(id: ExportId): boolean {
  return answeredIds.has(id);
}

/** rendering | muxing | saving → failed */
export function fail(failure: AppFailure): void {
  apply("fail", (current) => ({ ...current, failure }));
}

/** done | failed → idle */
export function reset(): void {
  apply("reset", empty);
}

/** The gate stopped a start, or what it opened closed. What the last start left is gone. Not a change of state. */
export function setGate(gate: ExportGate | null): void {
  const next = { ...useExportStore.getState(), gate };
  delete next.blocker;
  delete next.gateFailure;
  useExportStore.setState(next, true);
}

/** The gate needed a new token and could not get one. Not a change of state. */
export function gateFailed(failure: AppFailure): void {
  const next = { ...useExportStore.getState(), gate: null, gateFailure: failure };
  delete next.blocker;
  useExportStore.setState(next, true);
}

/** A finished export on Free offers the upgrade, unless this export's offer was dismissed. */
export function offerUpgrade(): void {
  const { status, exportId } = useExportStore.getState();
  if (status === "done" && exportId !== undefined && !dismissedOffers.has(exportId)) {
    useExportStore.setState({ upgradeOffer: true });
  }
}

/** The prompt was dismissed, at the limit or after an export. */
export function dismissOffer(): void {
  const { exportId, gate } = useExportStore.getState();
  if (exportId !== undefined) {
    dismissedOffers.add(exportId);
  }
  useExportStore.setState({ upgradeOffer: false, gate: gate === "upgrade" ? null : gate });
}
