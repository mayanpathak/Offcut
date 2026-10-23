// A short text about a control (v4implementation D-181). It shows after
// TOOLTIP_DELAY_MS of hover and at once when the control gets keyboard
// focus; Escape hides it. The pointer may move onto it without it closing
// (WCAG 1.4.13), and the control is described by it (`aria-describedby`).
//
// The bubble lives in the top layer (`popover="manual"`), so a scrolling
// panel never clips it. Its place is set through two custom properties: the
// CSP allows no style attribute.

import { cloneElement, type ReactElement, type ReactNode, useEffect, useId, useLayoutEffect, useRef, useState } from "react";

import styles from "../styles/primitives.module.css";

/** How long the pointer rests on a control before its tooltip shows (D-181). */
const TOOLTIP_DELAY_MS = 400;
/** How long the tooltip waits for the pointer to cross onto it before it hides. */
const HIDE_GRACE_MS = 120;
/** The space between the control and the bubble, and the bubble and the window's edge. */
const GAP_TOKEN = "--sp-2";

type Trigger = ReactElement<{ "aria-describedby"?: string | undefined }>;

export type TooltipProps = {
  content: ReactNode;
  /** One element that can take focus. */
  children: Trigger;
};

export function Tooltip({ content, children }: TooltipProps) {
  const id = useId();
  const anchor = useRef<HTMLSpanElement>(null);
  const bubble = useRef<HTMLDivElement>(null);
  const timer = useRef<number | undefined>(undefined);
  const [open, setOpen] = useState(false);

  const clearTimer = (): void => {
    window.clearTimeout(timer.current);
    timer.current = undefined;
  };
  const later = (ms: number, next: boolean): void => {
    clearTimer();
    timer.current = window.setTimeout(() => {
      timer.current = undefined;
      setOpen(next);
    }, ms);
  };
  const now = (next: boolean): void => {
    clearTimer();
    setOpen(next);
  };

  useEffect(() => clearTimer, []);

  // Shown, measured and placed before the browser paints, so it never appears in the wrong place.
  useLayoutEffect(() => {
    const el = bubble.current;
    const trigger = anchor.current?.firstElementChild;
    if (el === null || trigger === null || trigger === undefined) {
      return;
    }
    if (!open) {
      if (el.matches(":popover-open")) {
        el.hidePopover();
      }
      return;
    }
    if (!el.matches(":popover-open")) {
      el.showPopover();
    }
    const gap = parseFloat(getComputedStyle(el).getPropertyValue(GAP_TOKEN)) || 0;
    const at = trigger.getBoundingClientRect();
    const size = el.getBoundingClientRect();
    const left = Math.min(
      Math.max(at.left + at.width / 2 - size.width / 2, gap),
      window.innerWidth - size.width - gap,
    );
    // Above the control, or below it when there is no room above.
    const above = at.top - gap - size.height;
    const top = above >= gap ? above : at.bottom + gap;
    el.style.setProperty("--tip-x", `${String(Math.round(left))}px`);
    el.style.setProperty("--tip-y", `${String(Math.round(top))}px`);
  }, [open]);

  const described = [children.props["aria-describedby"], id].filter(Boolean).join(" ");

  return (
    <span
      ref={anchor}
      className={styles.tooltipAnchor}
      onPointerEnter={() => {
        later(TOOLTIP_DELAY_MS, true);
      }}
      onPointerLeave={() => {
        if (open) {
          later(HIDE_GRACE_MS, false);
        } else {
          now(false);
        }
      }}
      onFocus={(event) => {
        if (event.target.matches(":focus-visible")) {
          now(true);
        }
      }}
      onBlur={() => {
        now(false);
      }}
      onKeyDown={(event) => {
        if (event.key === "Escape" && open) {
          event.stopPropagation();
          now(false);
        }
      }}
    >
      {cloneElement(children, { "aria-describedby": described })}
      <div
        ref={bubble}
        id={id}
        role="tooltip"
        popover="manual"
        className={styles.tooltip}
        onPointerEnter={clearTimer}
        onPointerLeave={() => {
          now(false);
        }}
      >
        {content}
      </div>
    </span>
  );
}
