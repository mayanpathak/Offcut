// One export (TS §12.2): idle → gating → rendering → muxing → saving → done.
// A new export starts from `idle`, with a new id.
//
// The whole table is here from V2 on. V2 never fires `cache_hit`, `cancel`,
// `cancel_done` or `retry`: they belong to the render cache, the cancel and
// the retry of later versions.

import type { MachineDef } from "./transition";

export type ExportStatus = "idle" | "gating" | "rendering" | "muxing" | "saving" | "done" | "cancelled" | "failed";

export type ExportEvent =
  | "start"
  | "blocked"
  | "clear"
  | "cache_hit"
  | "encoded"
  | "finalized"
  | "saved"
  | "cancel"
  | "cancel_done"
  | "fail"
  | "retry"
  | "reset";

// Absent on purpose (TS §12.3): `rendering` does not lead to `rendering`, and
// `done` does not lead to `rendering`.
export const EXPORT_MACHINE: MachineDef<ExportStatus, ExportEvent> = {
  idle: { start: "gating" },
  gating: { blocked: "idle", clear: "rendering", cache_hit: "saving" },
  rendering: { encoded: "muxing", cancel: "cancelled", fail: "failed" },
  muxing: { finalized: "saving", cancel: "cancelled", fail: "failed" },
  saving: { saved: "done", fail: "failed" },
  done: { reset: "idle" },
  cancelled: { cancel_done: "idle" },
  failed: { retry: "gating", reset: "idle" },
};
