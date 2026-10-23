// An on/off setting (v4implementation D-181): `role="switch"` with
// `aria-checked`. The whole row is the control: its label, its hint and the
// switch itself, so a click anywhere on it, or Enter or Space, flips it.
// Disabled with a reason, it stays focusable and its tooltip says why.

import { useId } from "react";

import styles from "../styles/controls.module.css";
import { Tooltip } from "./Tooltip";

export type SwitchProps = {
  checked: boolean;
  label: string;
  hint?: string;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  /** Why the switch is disabled, shown as its tooltip. */
  disabledReason?: string;
  "data-testid"?: string;
};

export function Switch({
  checked,
  label,
  hint,
  onChange,
  disabled = false,
  disabledReason,
  "data-testid": testId,
}: SwitchProps) {
  const labelId = useId();
  const hintId = useId();
  const reason = disabled ? disabledReason : undefined;
  const row = (
    <button
      type="button"
      role="switch"
      className={styles.switchRow}
      aria-checked={checked}
      aria-labelledby={labelId}
      aria-describedby={hint === undefined ? undefined : hintId}
      disabled={disabled && reason === undefined}
      aria-disabled={reason === undefined ? undefined : true}
      data-testid={testId}
      onClick={() => {
        if (!disabled) {
          onChange(!checked);
        }
      }}
    >
      <span className={styles.switchText}>
        <span id={labelId} className={styles.switchLabel}>
          {label}
        </span>
        {hint === undefined ? null : (
          <span id={hintId} className={styles.switchHint}>
            {hint}
          </span>
        )}
      </span>
      <span className={styles.switchTrack} aria-hidden="true">
        <span className={styles.switchThumb} />
      </span>
    </button>
  );
  return reason === undefined ? row : <Tooltip content={reason}>{row}</Tooltip>;
}
