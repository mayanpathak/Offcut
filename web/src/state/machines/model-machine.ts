// The speech model on this device (TS §12.2): unknown → absent | partial |
// ready; → downloading → verifying → ready. A download that fails ends in
// `failed`, and one that is interrupted in `partial`, with its bytes kept.

import type { MachineDef } from "./transition";

export type ModelStatus = "unknown" | "absent" | "partial" | "ready" | "downloading" | "verifying" | "failed";

export type ModelEvent =
  | "inspected_absent"
  | "inspected_partial"
  | "inspected_ready"
  | "start"
  | "last_byte"
  | "net_fail"
  | "fail_download"
  | "fail_storage"
  | "hash_ok"
  | "hash_bad"
  | "cleared";

// `verifying` has no way back to `downloading`: a file that fails its check
// goes through `failed` first (TS §12.3).
export const MODEL_MACHINE: MachineDef<ModelStatus, ModelEvent> = {
  unknown: { inspected_absent: "absent", inspected_partial: "partial", inspected_ready: "ready" },
  absent: { start: "downloading" },
  partial: { start: "downloading", cleared: "absent" },
  ready: { cleared: "absent" },
  downloading: { last_byte: "verifying", net_fail: "partial", fail_download: "failed", fail_storage: "failed" },
  verifying: { hash_ok: "ready", hash_bad: "failed" },
  failed: { start: "downloading" },
};
