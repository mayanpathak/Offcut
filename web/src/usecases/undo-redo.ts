// Undo and redo (v4implementation D-184): the edit before the last change,
// or the one undone last, applied through the update rule, in mode `plan`
// when its word edits differ from the clip's (the events and the plan are
// found again) and in mode `scene` otherwise. The top bar's two buttons and,
// from V4 Prompt 117, Ctrl/⌘+Z and Shift+Ctrl/⌘+Z call these.

import { useClipStore } from "../state/clip-store";
import * as history from "../state/history";
import { sameWords, settled, update } from "./review-update";

async function step(way: "undo" | "redo"): Promise<void> {
  // A change in flight is part of the history first (D-147).
  await settled();
  const { status, edit: current } = useClipStore.getState();
  if (status !== "ready") {
    return;
  }
  const target = way === "undo" ? history.undo(current) : history.redo(current);
  if (target === undefined) {
    return;
  }
  await update(() => target, sameWords(current, target) ? "scene" : "plan", way);
  // Not applied (refused, or failed and restored): the history goes back to where it was.
  if (useClipStore.getState().edit !== target) {
    if (way === "undo") {
      history.redo(target);
    } else {
      history.undo(target);
    }
  }
}

/** Goes back one change. */
export function undo(): Promise<void> {
  return step("undo");
}

/** Brings back the change undone last. */
export function redo(): Promise<void> {
  return step("redo");
}
