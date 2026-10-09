// Why an action cannot start now (TS §12.5). This file is the only place
// these conditions are evaluated. It reads the stores and changes nothing.

import { useCapabilityStore } from "./capability-store";
import { useClipStore } from "./clip-store";
import { useExportStore } from "./export-store";
import type { ClipStatus } from "./machines/clip-machine";
import type { ExportStatus } from "./machines/export-machine";

// Every blocker, in the order of TS §12.5. scripts/check-copy-codes.mjs reads
// this list: one code on a line.
export const BLOCKER_CODES = [
  "B_UNSUPPORTED",
  "B_PIPELINE_BUSY",
  "B_EXPORT_IN_PROGRESS",
  "B_PIPELINE_NOT_READY",
  "B_STORAGE_LOW",
  "B_NOT_SIGNED_IN",
  "B_ENTITLEMENT_EXPIRED",
  "B_NO_FREE_EXPORTS",
] as const;

export type BlockerCode = (typeof BLOCKER_CODES)[number];

const CLIP_BUSY: readonly ClipStatus[] = ["importing", "processing", "updating"];
const EXPORT_RUNNING: readonly ExportStatus[] = ["gating", "rendering", "muxing", "saving"];

/** The first thing that stands in the way of importing a clip, or `null`. */
export function forImport(): BlockerCode | null {
  if (useCapabilityStore.getState().status !== "supported") {
    return "B_UNSUPPORTED";
  }
  if (CLIP_BUSY.includes(useClipStore.getState().status)) {
    return "B_PIPELINE_BUSY";
  }
  if (EXPORT_RUNNING.includes(useExportStore.getState().status)) {
    return "B_EXPORT_IN_PROGRESS";
  }
  return null;
}

/**
 * The first thing that stands in the way of an export, or `null`. It is
 * asked while the export is still `idle`.
 *
 * V2 has the two conditions that need nothing but the stores. V5 adds the
 * room on the device, and V6 the account and the plan.
 */
export function forExport(): BlockerCode | null {
  if (EXPORT_RUNNING.includes(useExportStore.getState().status)) {
    return "B_EXPORT_IN_PROGRESS";
  }
  if (useClipStore.getState().status !== "ready") {
    return "B_PIPELINE_NOT_READY";
  }
  return null;
}
