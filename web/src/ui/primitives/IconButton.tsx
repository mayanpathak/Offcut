// A button that is only an icon (v4implementation D-181). Its name is
// `label`, given to assistive technology as `aria-label` and shown in its
// tooltip with its shortcut, if it has one. `pressed` makes it a toggle.
// Disabled with a reason, it stays focusable and its tooltip says why.

import type { ReactNode } from "react";

import styles from "../styles/primitives.module.css";
import { Kbd, type KbdProps } from "./Kbd";
import { Tooltip } from "./Tooltip";

export type IconButtonProps = {
  icon: ReactNode;
  label: string;
  size?: "sm" | "md";
  shortcut?: KbdProps;
  /** For a toggle: whether it is on. */
  pressed?: boolean;
  disabled?: boolean;
  /** Why the button is disabled, shown as its tooltip. */
  disabledReason?: string;
  onClick?: () => void;
  "data-testid"?: string;
};

export function IconButton({
  icon,
  label,
  size = "md",
  shortcut,
  pressed,
  disabled = false,
  disabledReason,
  onClick,
  "data-testid": testId,
}: IconButtonProps) {
  const reason = disabled ? disabledReason : undefined;
  const tip =
    reason ??
    (shortcut === undefined ? (
      label
    ) : (
      <span className={styles.tooltipRow}>
        {label}
        <Kbd {...shortcut} />
      </span>
    ));
  return (
    <Tooltip content={tip}>
      <button
        type="button"
        className={[styles.iconButton, size === "sm" ? styles.iconSm : styles.iconMd].join(" ")}
        aria-label={label}
        aria-pressed={pressed}
        disabled={disabled && reason === undefined}
        aria-disabled={reason === undefined ? undefined : true}
        data-testid={testId}
        onClick={() => {
          if (!disabled) {
            onClick?.();
          }
        }}
      >
        {icon}
      </button>
    </Tooltip>
  );
}
