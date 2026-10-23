// The preview player (TS §12.2): detached → stopped → playing ⇄ paused, with
// `seeking` while the user scrubs and `locked` while an export runs. Written
// by usecases/control-preview.ts through the actions below.
//
// The machine's table is in this file, as the capability machine's is in its
// store (v2implementation D-19).

import { create } from "zustand";

import type { OutMs } from "../gen/domain";
import type { PreviewStats } from "../workers/protocol";
import { type MachineDef, transition } from "./machines/transition";

export type PreviewStatus = "detached" | "stopped" | "playing" | "paused" | "seeking" | "locked";

export type PreviewState = {
  status: PreviewStatus;
  /** The preview was played since it was attached. */
  playedOnce: boolean;
  /** Where the preview is in the program: each clock sync while it plays, and each seek. Unknown before either. */
  position: OutMs | undefined;
  /** A seek's frame is being drawn (D-148). */
  seekBusy: boolean;
  /** What the last play drew (D-155). */
  lastStats: PreviewStats | undefined;
};

type PreviewEvent =
  | "attach"
  | "play"
  | "pause"
  | "end"
  | "seek"
  | "seek_done_playing"
  | "seek_done_paused"
  | "lock"
  | "unlock"
  | "detach";

// `lock` and `detach` are accepted in every state. `detached` does not lead
// to `playing` (TS §12.3). The three seek events fire from the scrub bar
// (v4implementation D-148).
const MACHINE: MachineDef<PreviewStatus, PreviewEvent> = {
  detached: { attach: "stopped", lock: "locked", detach: "detached" },
  stopped: { play: "playing", lock: "locked", detach: "detached" },
  playing: { pause: "paused", end: "stopped", seek: "seeking", lock: "locked", detach: "detached" },
  paused: { play: "playing", seek: "seeking", lock: "locked", detach: "detached" },
  seeking: { seek_done_playing: "playing", seek_done_paused: "paused", lock: "locked", detach: "detached" },
  locked: { unlock: "paused", lock: "locked", detach: "detached" },
};

export const usePreviewStore = create<PreviewState>(() => ({
  status: "detached",
  playedOnce: false,
  position: undefined,
  seekBusy: false,
  lastStats: undefined,
}));

/** Applies `event`; `change` is stored only if the transition led to another state. */
function apply(event: PreviewEvent, change: Partial<Omit<PreviewState, "status">> = {}): void {
  const current = usePreviewStore.getState().status;
  const status = transition("preview", MACHINE, current, event);
  // An unchanged state is an illegal attempt, which was reported, or a
  // `lock` or `detach` in the state it leads to: nothing to store in either.
  if (status !== current) {
    usePreviewStore.setState({ status, ...change });
  }
}

/** detached → stopped: a canvas is attached. */
export function attached(): void {
  apply("attach", { playedOnce: false });
}

/** stopped | paused → playing */
export function play(): void {
  apply("play", { playedOnce: true });
}

/** playing → paused */
export function pause(): void {
  apply("pause");
}

/** playing → stopped: the end of the clip. */
export function ended(): void {
  apply("end");
}

/** any → locked: an export runs. */
export function lock(): void {
  apply("lock");
}

/** locked → paused */
export function unlock(): void {
  apply("unlock");
}

/** any → detached */
export function detached(): void {
  apply("detach", { playedOnce: false, position: undefined, seekBusy: false, lastStats: undefined });
}

/** playing | paused → seeking: the scrub bar is held. */
export function seekStart(): void {
  apply("seek");
}

/** seeking → playing or paused, as it was before: the scrub bar is let go. */
export function seekDone(playing: boolean): void {
  apply(playing ? "seek_done_playing" : "seek_done_paused");
}

/** Where the preview is now. Not a change of state. */
export function at(position: OutMs): void {
  usePreviewStore.setState({ position });
}

/** Whether a seek's frame is being drawn. */
export function seekBusy(busy: boolean): void {
  usePreviewStore.setState({ seekBusy: busy });
}

/** What the last play drew. */
export function played(stats: PreviewStats): void {
  usePreviewStore.setState({ lastStats: stats });
}
