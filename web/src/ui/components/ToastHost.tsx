// The editor's toasts (v4implementation §17.3, D-181): one stack at the
// bottom centre, oldest on top, at most `SHOWN` at a time; one that waits
// shows when one above it goes, and its 5 s start then. The host sits in the
// editor's one `aria-live="polite"` region (D-192), so a toast is announced
// as it appears and only once.
//
// The editor's three toasts: "Change all" after a fix (D-175, the transcript
// editor's), a hidden word with "Undo", and the undo toast, which says that
// a change was undone or redone and offers the other (D-184). The last is
// shown by `stepHistory`, which the top bar's buttons and the shortcuts both
// call.

import { messages } from "../../copy/messages";
import { useClipStore } from "../../state/clip-store";
import { useUiStore } from "../../state/ui-store";
import { redo, undo } from "../../usecases/undo-redo";
import { Toast } from "../primitives/Toast";
import styles from "../styles/editor.module.css";

/** How many toasts are shown at once. */
const SHOWN = 3;
/** The undo toast's id: an undo replaces the toast of the redo before it, and the other way round. */
const HISTORY_TOAST = "history";

/** Undo or redo (D-184). A step that applied says so in the undo toast, with the other step as its action. */
export async function stepHistory(way: "undo" | "redo"): Promise<void> {
  const before = useClipStore.getState().edit;
  await (way === "undo" ? undo() : redo());
  if (useClipStore.getState().edit === before) {
    return;
  }
  const back = way === "undo" ? "redo" : "undo";
  useUiStore.getState().pushToast({
    id: HISTORY_TOAST,
    message: way === "undo" ? messages.editor.undone : messages.editor.redone,
    action: {
      label: back === "undo" ? messages.editor.undo : messages.editor.redo,
      run: () => {
        void stepHistory(back);
      },
    },
  });
}

export function ToastHost() {
  const toasts = useUiStore((state) => state.toasts);
  const dismiss = useUiStore((state) => state.dismissToast);
  if (toasts.length === 0) {
    return null;
  }
  return (
    <div className={styles.toasts} data-testid="toasts">
      {toasts.slice(0, SHOWN).map((toast) => (
        <Toast
          key={toast.key}
          message={toast.message}
          {...(toast.action === undefined ? {} : { action: toast.action.label, onAction: toast.action.run })}
          onDismiss={() => {
            dismiss(toast.id);
          }}
        />
      ))}
    </div>
  );
}
