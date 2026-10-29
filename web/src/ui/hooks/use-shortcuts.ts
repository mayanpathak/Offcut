// The editor's keyboard shortcuts (v4implementation D-185): one map, one
// listener on the window, mounted once by `EditorLayout`. Every shortcut is
// also a visible control: the play button and the scrub bar, the top bar's
// undo and redo, a field's or a menu's own Escape, the top bar's button that
// opens the sheet.
//
// A key the focused control takes stays that control's: in a text field
// only Escape reaches the map (Space types a space, Ctrl+Z undoes typing);
// Space presses a focused button; the arrow keys move a slider, a tab list,
// a segmented control or the transcript's words. A control that handled a
// key itself (its `preventDefault`, or a field's Escape stopped where it
// was pressed) is left alone. While an export runs, only the sheet opens
// (TS §12.4); while the sheet is open, the keys are the dialog's.

import { useEffect } from "react";

import { usePreviewStore } from "../../state/preview-store";
import { useUiStore } from "../../state/ui-store";
import { stepFrame, stepSecond, togglePlay } from "../../usecases/control-preview";
import { stepHistory } from "../components/ToastHost";

/** The `<input>` types that are typed into. */
const TEXT_INPUTS = new Set(["text", "search", "email", "url", "tel", "password", "number"]);
/** Roles whose element takes Space itself: it presses, checks or chooses it. */
const SPACE_ROLES = new Set(["button", "switch", "checkbox", "radio", "tab", "menuitem", "option", "link"]);
/** Roles whose element takes ← and → itself. */
const ARROW_ROLES = new Set(["slider", "tab", "tablist", "radio", "radiogroup", "menu", "menuitem", "listbox", "option", "spinbutton"]);

function isTextField(target: Element | null): boolean {
  if (target instanceof HTMLTextAreaElement) {
    return true;
  }
  if (target instanceof HTMLInputElement) {
    return TEXT_INPUTS.has(target.type);
  }
  return target instanceof HTMLElement && target.isContentEditable;
}

/** Whether Space does something of its own on `target`. The scrub bar does nothing with it: there it plays. */
function takesSpace(target: Element | null): boolean {
  if (target instanceof HTMLInputElement) {
    return target.type !== "range";
  }
  if (target instanceof HTMLButtonElement || target instanceof HTMLAnchorElement || target instanceof HTMLSelectElement) {
    return true;
  }
  return target !== null && SPACE_ROLES.has(target.getAttribute("role") ?? "");
}

function takesArrows(target: Element | null): boolean {
  if (target instanceof HTMLInputElement || target instanceof HTMLSelectElement) {
    return true;
  }
  return target !== null && ARROW_ROLES.has(target.getAttribute("role") ?? "");
}

function onKeyDown(e: KeyboardEvent): void {
  const ui = useUiStore.getState();
  if (e.defaultPrevented || e.altKey || ui.shortcuts) {
    return;
  }
  const target = e.target instanceof Element ? e.target : null;
  const mod = e.ctrlKey || e.metaKey;
  const key = e.key.length === 1 ? e.key.toLowerCase() : e.key;

  // A field whose focus went elsewhere is still open: Escape closes it. A
  // field or a menu that has the focus closed itself already.
  if (key === "Escape") {
    if (ui.editingWord !== null) {
      e.preventDefault();
      ui.editWord(null);
    }
    return;
  }
  if (isTextField(target)) {
    return;
  }
  if (key === "?" && !mod) {
    e.preventDefault();
    ui.showShortcuts(true);
    return;
  }
  if (usePreviewStore.getState().status === "locked") {
    return;
  }

  // Undo and redo go through the update rule's queue, as the top bar's buttons do (D-147).
  if (mod && key === "z") {
    e.preventDefault();
    void stepHistory(e.shiftKey ? "redo" : "undo");
    return;
  }
  if (e.ctrlKey && !e.shiftKey && key === "y") {
    e.preventDefault();
    void stepHistory("redo");
    return;
  }
  if (mod) {
    return;
  }

  if (key === " " && !e.shiftKey && !takesSpace(target)) {
    e.preventDefault();
    if (!e.repeat) {
      void togglePlay();
    }
    return;
  }
  if ((key === "ArrowLeft" || key === "ArrowRight") && !takesArrows(target)) {
    e.preventDefault();
    const direction = key === "ArrowLeft" ? -1 : 1;
    void (e.shiftKey ? stepSecond(direction) : stepFrame(direction));
  }
}

export function useShortcuts(): void {
  useEffect(() => {
    // On the window, in the bubbling phase: a control's own handler runs first.
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);
}
