// Undo and redo of the clip's edit (v4implementation D-184, §16.1): the
// `EditState` before each review change, up to `HISTORY_MAX` of them, and
// the ones undone, for redo. A store of its own, so that the top bar reads
// whether each can be done. Written by usecases/review-update.ts and
// usecases/undo-redo.ts; cleared with the clip, and not kept across a reload
// (V7).

import { create } from "zustand";

import type { EditState } from "../gen/domain";

/** How many changes can be undone (assumption, Appendix D). */
export const HISTORY_MAX = 50;

export type HistoryState = {
  /** Oldest first: the edit before each change that can be undone. */
  past: readonly EditState[];
  /** The edits undone, the latest last: what redo brings back. */
  future: readonly EditState[];
  canUndo: boolean;
  canRedo: boolean;
};

export const useHistoryStore = create<HistoryState>(() => ({ past: [], future: [], canUndo: false, canRedo: false }));

function store(past: readonly EditState[], future: readonly EditState[]): void {
  useHistoryStore.setState({ past, future, canUndo: past.length > 0, canRedo: future.length > 0 });
}

/** A change was made from `edit`: it can be undone, and nothing undone before it can be redone. */
export function push(edit: EditState): void {
  store([...useHistoryStore.getState().past, edit].slice(-HISTORY_MAX), []);
}

/** The edit before `current`, which is kept for redo; `undefined` when there is nothing to undo. */
export function undo(current: EditState): EditState | undefined {
  const { past, future } = useHistoryStore.getState();
  const target = past.at(-1);
  if (target !== undefined) {
    store(past.slice(0, -1), [...future, current]);
  }
  return target;
}

/** The edit undone last, with `current` kept for undo; `undefined` when there is nothing to redo. */
export function redo(current: EditState): EditState | undefined {
  const { past, future } = useHistoryStore.getState();
  const target = future.at(-1);
  if (target !== undefined) {
    store([...past, current].slice(-HISTORY_MAX), future.slice(0, -1));
  }
  return target;
}

/** A new clip, or none: nothing to undo or redo. */
export function clear(): void {
  store([], []);
}
