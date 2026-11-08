// Why an action cannot start now (TS §12.5). This file is the only place
// these conditions are evaluated. It reads the stores and changes nothing.

import { type Effective, effective } from "../entitlement/entitlement";
import { useAuthStore } from "./auth-store";
import { useCapabilityStore } from "./capability-store";
import { useClipStore } from "./clip-store";
import { useEntitlementStore } from "./entitlement-store";
import { useExportStore } from "./export-store";
import type { ClipStatus } from "./machines/clip-machine";
import type { ExportStatus } from "./machines/export-machine";
import type { SourceStatus } from "./machines/source-machine";
import { useSourceStore } from "./source-store";

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
/** V6.1 (D-416): a source job that holds the workers. Paused, transcribed or waiting for its file, it does not. */
const SOURCE_WORKING: readonly SourceStatus[] = ["probing", "extracting_audio", "transcribing"];

/** The first thing that stands in the way of importing a clip, or `null`. */
export function forImport(): BlockerCode | null {
  if (useCapabilityStore.getState().status !== "supported") {
    return "B_UNSUPPORTED";
  }
  if (CLIP_BUSY.includes(useClipStore.getState().status)) {
    return "B_PIPELINE_BUSY";
  }
  if (SOURCE_WORKING.includes(useSourceStore.getState().status)) {
    return "B_PIPELINE_BUSY";
  }
  if (EXPORT_RUNNING.includes(useExportStore.getState().status)) {
    return "B_EXPORT_IN_PROGRESS";
  }
  return null;
}

/**
 * V6.1 (D-416): the first thing that stands in the way of starting or continuing a
 * long source, or `null`. One worker pool serves both pages.
 */
export function forSource(): BlockerCode | null {
  if (useCapabilityStore.getState().status !== "supported") {
    return "B_UNSUPPORTED";
  }
  if (CLIP_BUSY.includes(useClipStore.getState().status) || EXPORT_RUNNING.includes(useExportStore.getState().status)) {
    return "B_PIPELINE_BUSY";
  }
  return null;
}

/** A file may take this much more than its bitrate and length say (TS §12.5). */
const STORAGE_MARGIN = 1.2;

/** The room on the device, in bytes, and what an export needs: read by the use-case, since it takes a wait (v5implementation D-229). */
export type ExportRoom = { freeBytes: number; neededBytes: number };

const MS_PER_SECOND = 1000;

/** What the token on this page allows, less what the server has not been told of yet (v6implementation D-267). */
export function effectiveNow(): Effective | null {
  const { claims, outboxFree } = useEntitlementStore.getState();
  return effective(claims, outboxFree);
}

/**
 * The account and the plan. A signed-out person with only an expired token is
 * asked to sign in: the expired token is reached only while signed in, where a
 * new one can be fetched (v6implementation §12). So is a session with no token.
 */
function accountBlocker(): BlockerCode | null {
  const signedIn = useAuthStore.getState().status === "signed_in";
  const now = effectiveNow();
  const usable = now !== null && now.exp >= Date.now() / MS_PER_SECOND;
  if (!signedIn && !usable) {
    return "B_NOT_SIGNED_IN";
  }
  if (!usable) {
    return "B_ENTITLEMENT_EXPIRED";
  }
  return now.plan === "free" && now.freeExportsRemaining === 0 ? "B_NO_FREE_EXPORTS" : null;
}

/**
 * The first thing that stands in the way of an export, or `null`. It is
 * asked while the export is still `idle` or, with `room`, in `gating` after a
 * cache miss, when the account has been asked already and only the room is left
 * (v6implementation D-309).
 */
export function forExport(room?: ExportRoom): BlockerCode | null {
  const { status } = useExportStore.getState();
  // Gating with room is this export's own gate, not another one running.
  if (EXPORT_RUNNING.includes(status) && !(room !== undefined && status === "gating")) {
    return "B_EXPORT_IN_PROGRESS";
  }
  if (useClipStore.getState().status !== "ready") {
    return "B_PIPELINE_NOT_READY";
  }
  if (room !== undefined) {
    return room.freeBytes < room.neededBytes * STORAGE_MARGIN ? "B_STORAGE_LOW" : null;
  }
  return accountBlocker();
}

/** Whether a blocker disables the button. A click can mend any other, so it must get through to open what does (D-290). */
export function disablesExport(code: BlockerCode): boolean {
  return code === "B_EXPORT_IN_PROGRESS" || code === "B_PIPELINE_NOT_READY";
}

/** The first thing that stands in the way of buying: a cached token is no session (v6implementation §12). */
export function forCheckout(): BlockerCode | null {
  return useAuthStore.getState().status === "signed_in" ? null : "B_NOT_SIGNED_IN";
}
