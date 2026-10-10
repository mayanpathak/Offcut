// What is known about the speech model on this device, and how far its
// download is (TS §12.1). Written by models/model-manager.ts only.

import { create } from "zustand";

import type { Bytes, ErrorCode } from "../gen/domain";
import { MODEL_MACHINE, type ModelEvent, type ModelStatus } from "./machines/model-machine";
import { transition } from "./machines/transition";

export type ModelProgress = { done: Bytes; total: Bytes; etaSecs: number | null };

export type ModelState = ModelProgress & {
  status: ModelStatus;
  /** Why the download failed. Present in `failed` only. */
  error?: ErrorCode;
};

// Nothing is counted before the first inspection.
const ZERO_BYTES = 0 as Bytes;

export const useModelStore = create<ModelState>(() => ({
  status: "unknown",
  done: ZERO_BYTES,
  total: ZERO_BYTES,
  etaSecs: null,
}));

/**
 * Applies `event`. The state is stored only if the transition was legal, and
 * it is stored whole: an `error` that `change` does not name is dropped.
 */
function apply(event: ModelEvent, change: Partial<Omit<ModelState, "status">> = {}): void {
  const { status: current, done, total, etaSecs } = useModelStore.getState();
  const status = transition("model", MODEL_MACHINE, current, event);
  // No legal transition leads back to the same state: an unchanged state
  // means the attempt was illegal and was reported.
  if (status !== current) {
    useModelStore.setState({ done, total, etaSecs, ...change, status }, true);
  }
}

const INSPECTED: Record<"absent" | "partial" | "ready", ModelEvent> = {
  absent: "inspected_absent",
  partial: "inspected_partial",
  ready: "inspected_ready",
};

/** unknown → absent | partial | ready */
export function inspected(result: "absent" | "partial" | "ready"): void {
  apply(INSPECTED[result]);
}

/** absent | partial | failed → downloading */
export function start(): void {
  apply("start", { etaSecs: null });
}

/** How far the download is. Not a change of state; kept only while a download runs. */
export function progress(p: ModelProgress): void {
  const { status } = useModelStore.getState();
  if (status === "downloading" || status === "verifying") {
    useModelStore.setState({ done: p.done, total: p.total, etaSecs: p.etaSecs });
  }
}

/** downloading → verifying: the last file of the set has all its bytes. */
export function lastByte(): void {
  apply("last_byte", { etaSecs: null });
}

/** downloading → partial: the download was interrupted; its bytes are kept. */
export function netFail(): void {
  apply("net_fail", { etaSecs: null });
}

/**
 * downloading | verifying → failed(code).
 *
 * The machine has three ways into `failed`, and the set has several files.
 * A file that fails its hash check before the last one has arrived is
 * reported as the end of the download followed by a failed check, so that
 * `hash_bad` is always what a wrong hash goes through. From `verifying`,
 * `hash_bad` is the only way out, also for a storage failure there.
 */
export function fail(code: ErrorCode): void {
  if (code === "E_MODEL_HASH" && useModelStore.getState().status === "downloading") {
    apply("last_byte");
  }
  if (useModelStore.getState().status === "verifying") {
    apply("hash_bad", { etaSecs: null, error: code });
  } else {
    apply(code === "E_MODEL_DOWNLOAD" ? "fail_download" : "fail_storage", { etaSecs: null, error: code });
  }
}

/** verifying → ready */
export function hashOk(): void {
  const { total } = useModelStore.getState();
  apply("hash_ok", { done: total, etaSecs: null });
}

/** ready | partial → absent */
export function cleared(): void {
  apply("cleared", { done: ZERO_BYTES, total: ZERO_BYTES, etaSecs: null });
}
