// The clip being worked on (TS §12.2): idle → importing → accepted →
// processing → ready ⇄ updating. A clip that cannot be used ends in
// `rejected`, and a failure on this device in `failed`. A new clip must first
// move the old one to `idle`.
//
// The whole table is here from V2 on. V2 fires `import`, `accept`, `reject`,
// `fail`, `run`, `done`, `no_speech` and `reset`; the others belong to the
// cancel, the edits, the retry and the restore of later versions.

import type { MachineDef } from "./transition";

export type ClipStatus =
  | "idle"
  | "importing"
  | "accepted"
  | "processing"
  | "ready"
  | "updating"
  | "rejected"
  | "failed";

export type ClipEvent =
  | "import"
  | "accept"
  | "reject"
  | "fail"
  | "run"
  | "done"
  | "no_speech"
  | "cancel"
  | "edit_start"
  | "edit_done"
  | "retry"
  | "reset"
  | "restore";

// Absent on purpose (TS §12.3): a rejected clip is never processed; an
// importing clip is not processed before it is accepted; a clip that is
// processing or ready does not start importing.
export const CLIP_MACHINE: MachineDef<ClipStatus, ClipEvent> = {
  idle: { import: "importing", restore: "ready" },
  importing: { accept: "accepted", reject: "rejected", fail: "failed" },
  accepted: { run: "processing", reset: "idle" },
  processing: { done: "ready", no_speech: "rejected", fail: "failed", cancel: "accepted" },
  ready: { edit_start: "updating", reset: "idle" },
  updating: { edit_done: "ready" },
  rejected: { reset: "idle" },
  failed: { retry: "processing", reset: "idle" },
};
