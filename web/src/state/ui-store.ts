// What the editor's interface shows apart from the clip (v4implementation
// §16.1): the side panel's tab, the follow switch, the word being edited, the
// toast queue and the save state. Written by the use-cases and by the
// components' own actions (a tab chosen, the switch flipped); never by a
// worker. Nothing here is kept across a reload.

import { create } from "zustand";

import type { WordIdx } from "../gen/domain";

export type Tab = "captions" | "style" | "visuals" | "framing";

/** A short message at the bottom of the window, with at most one action (D-181). */
export type ToastItem = {
  id: string;
  message: string;
  action?: { label: string; run: () => void };
};

export type UiState = {
  tab: Tab;
  /** The transcript follows playback (D-182). On by default. */
  follow: boolean;
  /** The word whose field is open in the transcript editor. */
  editingWord: WordIdx | null;
  /** Oldest first; the host shows them one at a time. */
  toasts: readonly ToastItem[];
  /** Whether an edit is still being written to the device (D-184). */
  save: "saved" | "saving";
  // Arrow-typed, so a component may take one out of the store and call it alone.
  selectTab: (tab: Tab) => void;
  setFollow: (on: boolean) => void;
  editWord: (word: WordIdx | null) => void;
  pushToast: (toast: ToastItem) => void;
  dismissToast: (id: string) => void;
  setSave: (save: "saved" | "saving") => void;
};

export const useUiStore = create<UiState>((set) => ({
  tab: "captions",
  follow: true,
  editingWord: null,
  toasts: [],
  save: "saved",
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
    set((state) => ({ toasts: [...state.toasts.filter((queued) => queued.id !== toast.id), toast] }));
  },
  dismissToast: (id) => {
    set((state) => ({ toasts: state.toasts.filter((queued) => queued.id !== id) }));
  },
  setSave: (save) => {
    set({ save });
  },
}));
