// The preview player (TS §12.2): detached → stopped → playing ⇄ paused, with
// `seeking` while the user scrubs and `locked` while an export runs. Written
// by usecases/control-preview.ts through the actions below.
//
// The machine's table is in this file, as the capability machine's is in its
// store (v2implementation D-19).

import { create } from "zustand";

import { type MachineDef, transition } from "./machines/transition";

export type PreviewStatus = "detached" | "stopped" | "playing" | "paused" | "seeking" | "locked";

export type PreviewState = {
  status: PreviewStatus;
  /** The preview was played since it was attached. */
  playedOnce: boolean;
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
// to `playing` (TS §12.3). The three seek events are here for V4, the first
// version to fire them.
const MACHINE: MachineDef<PreviewStatus, PreviewEvent> = {
  detached: { attach: "stopped", lock: "locked", detach: "detached" },
  stopped: { play: "playing", lock: "locked", detach: "detached" },
  playing: { pause: "paused", end: "stopped", seek: "seeking", lock: "locked", detach: "detached" },
  paused: { play: "playing", seek: "seeking", lock: "locked", detach: "detached" },
  seeking: { seek_done_playing: "playing", seek_done_paused: "paused", lock: "locked", detach: "detached" },
  locked: { unlock: "paused", lock: "locked", detach: "detached" },
};

export const usePreviewStore = create<PreviewState>(() => ({ status: "detached", playedOnce: false }));

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
  apply("detach", { playedOnce: false });
}
