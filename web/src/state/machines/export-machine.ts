// One export (TS §12.2): idle → gating → rendering → muxing → saving → done.
// A new export starts from `idle`, with a new id.
//
// The whole table is here from V2 on. `cache_hit` is the render cache's, from
// V5 on, and `cancel` and `cancel_done` the cancel's, from V4 on. `retry` is
// never fired: "Try again" starts a new export through `reset` (D-153).

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
