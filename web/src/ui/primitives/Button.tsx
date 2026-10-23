// A button of the editor (v4implementation D-181): primary, secondary, ghost
// or danger, small or medium. A busy button shows a spinner in place of its
// label once the action has taken BUSY_DELAY_MS, so a quick action never
// flickers (§17.4); the label stays in place, unseen, so the width holds.
//
// A button disabled with a reason stays focusable (`aria-disabled`) and says
// why in a tooltip; one disabled without a reason is simply disabled.

import { type ReactNode, useEffect, useState } from "react";

import styles from "../styles/primitives.module.css";
import { Spinner } from "./Spinner";
import { Tooltip } from "./Tooltip";

/** How long an action runs before its button shows a spinner (§17.4). */
const BUSY_DELAY_MS = 300;

const VARIANT_CLASS = {
  primary: styles.primary,
  secondary: styles.secondary,
  ghost: styles.ghost,
  danger: styles.danger,
} as const;

export type ButtonProps = {
  children: ReactNode;
  variant?: keyof typeof VARIANT_CLASS;
  size?: "sm" | "md";
  /** An icon before the label. */
  icon?: ReactNode;
  busy?: boolean;
  disabled?: boolean;
  /** Why the button is disabled, shown as its tooltip. */
  disabledReason?: string;
  onClick?: () => void;
  "data-testid"?: string;
};

export function Button({
  children,
  variant = "secondary",
  size = "md",
  icon,
  busy = false,
  disabled = false,
  disabledReason,
  onClick,
  "data-testid": testId,
}: ButtonProps) {
  const [late, setLate] = useState(false);
  useEffect(() => {
    if (!busy) {
      return undefined;
    }
    const timer = window.setTimeout(() => {
      setLate(true);
    }, BUSY_DELAY_MS);
    return () => {
      window.clearTimeout(timer);
      setLate(false);
    };
  }, [busy]);

  const reason = disabled ? disabledReason : undefined;
  const button = (
    <button
      type="button"
      className={[styles.button, VARIANT_CLASS[variant], size === "sm" ? styles.sm : styles.md].join(" ")}
      disabled={disabled && reason === undefined}
      aria-disabled={reason === undefined ? undefined : true}
      aria-busy={busy ? true : undefined}
      data-testid={testId}
      onClick={() => {
        if (!disabled && !busy) {
          onClick?.();
        }
      }}
    >
      <span className={late ? styles.labelHidden : styles.label}>
        {icon}
        {children}
      </span>
      {late ? (
        <span className={styles.busy}>
          <Spinner size="sm" />
        </span>
      ) : null}
    </button>
  );
  return reason === undefined ? button : <Tooltip content={reason}>{button}</Tooltip>;
}
