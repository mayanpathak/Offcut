// The long source on the page (v6.1implementation §9.4, §10). Every change
// of `status` goes through the source machine (INV-16). The use-cases write
// here; the long page reads. Nothing here is stored: `sources-repo.ts` holds
// what survives a reload.

import { create } from "zustand";

import type { RouteReject, SourceInfo, Transcript } from "../gen/domain";
import type { SourceId } from "../workers/protocol";
import { SOURCE_MACHINE, type SourceEvent, type SourceStatus } from "./machines/source-machine";
import { transition } from "./machines/transition";

/** Why the cloud work stopped (D-410, D-418). */
export type PauseReason = "unavailable" | "busy" | "signed_out" | "offline" | "wake_lost";

/** Why a dropped file waits before any work (§10.2). */
export type GateReason = "signed_out" | "checking" | "unavailable" | "storage_low";

/** Where a stage stands; `done` and `total` in the stage's own unit (ms of sound, windows). */
export type SourceProgress = { stage: "audio" | "transcribe"; done: number; total: number; startedAt: number };

export type SourceState = {
  status: SourceStatus;
  sourceId: SourceId | null;
  info: SourceInfo | null;
  rejected: RouteReject | null;
  gate: { reason: GateReason; neededBytes?: number } | null;
  paused: { reason: PauseReason; from: "extracting_audio" | "transcribing" } | null;
  /** A failure's code, from `messages.ts`' lists (`E_*`). */
  failure: { code: string; detail?: string } | null;
  progress: SourceProgress | null;
  transcript: Transcript | null;
  /** A source a reload left unfinished: "Continue your video" (D-419). */
  resumable: { sourceId: SourceId; stage: "extracting_audio" | "transcribing" } | null;
  /** "This is a different file" was the last answer to a drop while awaiting the file. */
  differentFile: boolean;
  send(event: SourceEvent): void;
  patch(change: Partial<Omit<SourceState, "status" | "send" | "patch" | "clear">>): void;
  /** Back to `idle` with nothing held, through the machine's `reset` where it allows one. */
  clear(): void;
};

const EMPTY = {
  sourceId: null,
  info: null,
  rejected: null,
  gate: null,
  paused: null,
  failure: null,
  progress: null,
  transcript: null,
  resumable: null,
  differentFile: false,
} as const;

export const useSourceStore = create<SourceState>()((set, get) => ({
  status: "idle",
  ...EMPTY,
  send(event) {
    set({ status: transition("source", SOURCE_MACHINE, get().status, event) });
  },
  patch(change) {
    set(change);
  },
  clear() {
    const { status } = get();
    const next = status === "idle" ? "idle" : transition("source", SOURCE_MACHINE, status, "reset");
    set({ status: next, ...EMPTY });
  },
}));
