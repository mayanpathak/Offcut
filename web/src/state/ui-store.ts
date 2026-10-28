// What the editor's interface shows apart from the clip (v4implementation
// §16.1): the side panel's tab, the follow switch, the word being edited, the
// toast queue, the save state and whether the shortcuts sheet is open.
// Written by the use-cases and by the components' own actions (a tab chosen,
// the switch flipped); never by a worker. Nothing here is kept across a
// reload.

import { create } from "zustand";

import type { WordIdx } from "../gen/domain";

export type Tab = "captions" | "style" | "visuals" | "framing";

/** A short message at the bottom of the window, with at most one action (D-181). */
export type ToastItem = {
  id: string;
  message: string;
  action?: { label: string; run: () => void };
};

/** A toast in the queue. `key` is new with every push, so a toast pushed again starts its time again. */
export type QueuedToast = ToastItem & { key: number };

let pushes = 0;

export type UiState = {
  tab: Tab;
  /** The transcript follows playback (D-182). On by default. */
  follow: boolean;
  /** The word whose field is open in the transcript editor. */
  editingWord: WordIdx | null;
  /** Oldest first; the host shows the first few as a stack (D-181). */
  toasts: readonly QueuedToast[];
  /** Whether an edit is still being written to the device (D-184). */
  save: "saved" | "saving";
  /** The Style tab's three frames, Clean, Bold, Mono (D-186); `null` until drawn. */
  thumbnails: readonly ImageBitmap[] | null;
  /** The shortcuts sheet is open (D-185). */
  shortcuts: boolean;
  // Arrow-typed, so a component may take one out of the store and call it alone.
  selectTab: (tab: Tab) => void;
  setFollow: (on: boolean) => void;
  editWord: (word: WordIdx | null) => void;
  pushToast: (toast: ToastItem) => void;
  dismissToast: (id: string) => void;
  setSave: (save: "saved" | "saving") => void;
  /** Replaces the thumbnails; the ones they replace are closed (they hold GPU memory). */
  setThumbnails: (thumbnails: readonly ImageBitmap[] | null) => void;
  showShortcuts: (open: boolean) => void;
};

export const useUiStore = create<UiState>((set) => ({
  tab: "captions",
  follow: true,
  editingWord: null,
  toasts: [],
  save: "saved",
  thumbnails: null,
  shortcuts: false,
  selectTab: (tab) => {
    set({ tab });
  },
  setFollow: (follow) => {
    set({ follow });
  },
  editWord: (editingWord) => {
    set({ editingWord });
  },
  // A toast with the id of one already queued replaces it.
  pushToast: (toast) => {
    pushes += 1;
    const queued = { ...toast, key: pushes };
    set((state) => ({ toasts: [...state.toasts.filter((old) => old.id !== toast.id), queued] }));
  },
  dismissToast: (id) => {
    set((state) => ({ toasts: state.toasts.filter((queued) => queued.id !== id) }));
  },
  setSave: (save) => {
    set({ save });
  },
  setThumbnails: (thumbnails) => {
    set((state) => {
      for (const old of state.thumbnails ?? []) {
        if (!thumbnails?.includes(old)) {
          old.close();
        }
      }
      return { thumbnails };
    });
  },
  showShortcuts: (shortcuts) => {
    set({ shortcuts });
  },
}));
