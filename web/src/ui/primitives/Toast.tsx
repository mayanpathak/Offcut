// A short message at the bottom centre of the window, with at most one
// action (v4implementation D-181). It goes away after TOAST_MS; the time
// stops while the pointer is on it or the focus is in it, and Escape closes
// it. It fades and rises in over --base (§17.4).
//
// It is announced by where it is put: the host renders it inside the
// editor's one `aria-live="polite"` region (D-192), so the toast has no
// live region of its own and nothing is announced twice.

import { useEffect, useRef } from "react";

import styles from "../styles/primitives.module.css";
import { Button } from "./Button";

/** How long a toast stays when nothing holds it (D-181). */
const TOAST_MS = 5000;

export type ToastProps = {
  message: string;
  /** The words of the one action, from messages.ts. */
  action?: string;
  onAction?: () => void;
  /** Called when the time is up, on Escape, and after the action. */
  onDismiss: () => void;
};

export function Toast({ message, action, onAction, onDismiss }: ToastProps) {
  const timer = useRef<number | undefined>(undefined);
  const left = useRef(TOAST_MS);
  const startedAt = useRef(0);
  const holds = useRef({ pointer: false, focus: false });
  const dismiss = useRef(onDismiss);

  useEffect(() => {
    dismiss.current = onDismiss;
  }, [onDismiss]);

  const run = (): void => {
    if (timer.current !== undefined || holds.current.pointer || holds.current.focus) {
      return;
    }
    startedAt.current = performance.now();
    timer.current = window.setTimeout(() => {
      timer.current = undefined;
      dismiss.current();
    }, left.current);
  };
  const hold = (): void => {
    if (timer.current === undefined) {
      return;
    }
    window.clearTimeout(timer.current);
    timer.current = undefined;
    left.current = Math.max(0, left.current - (performance.now() - startedAt.current));
  };

  // Started once, when the toast appears; `run` reads only refs.
  useEffect(() => {
    run();
    return () => {
      window.clearTimeout(timer.current);
      timer.current = undefined;
    };
  }, []);

  return (
    <div
      className={styles.toast}
      onPointerEnter={() => {
        holds.current.pointer = true;
        hold();
      }}
      onPointerLeave={() => {
        holds.current.pointer = false;
        run();
      }}
      onFocus={() => {
        holds.current.focus = true;
        hold();
      }}
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget)) {
          holds.current.focus = false;
          run();
        }
      }}
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.stopPropagation();
          hold();
          dismiss.current();
        }
      }}
    >
      <p className={styles.toastMessage}>{message}</p>
      {action === undefined ? null : (
        <Button
          variant="ghost"
          size="sm"
          onClick={() => {
            hold();
            onAction?.();
            dismiss.current();
          }}
        >
          {action}
        </Button>
      )}
    </div>
  );
}
