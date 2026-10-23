// One choice out of a few, side by side (v4implementation D-181), with the
// semantics of a radio group: the checked option is in the page's tab order,
// and the arrow keys, Home and End move to another option and choose it.
// Disabled with a reason, the checked option stays focusable and its tooltip
// says why.

import { type KeyboardEvent, type ReactNode, useRef } from "react";

import styles from "../styles/controls.module.css";
import { Tooltip } from "./Tooltip";

export type SegmentOption<T extends string> = {
  value: T;
  label: string;
  icon?: ReactNode;
};

export type SegmentedControlProps<T extends string> = {
  options: readonly SegmentOption<T>[];
  value: T;
  onChange: (value: T) => void;
  /** The name of the group, for assistive technology. */
  label: string;
  disabled?: boolean;
  /** Why the control is disabled, shown as the checked option's tooltip. */
  disabledReason?: string;
};

export function SegmentedControl<T extends string>({
  options,
  value,
  onChange,
  label,
  disabled = false,
  disabledReason,
}: SegmentedControlProps<T>) {
  const refs = useRef<(HTMLButtonElement | null)[]>([]);
  const reason = disabled ? disabledReason : undefined;

  const choose = (index: number): void => {
    const option = options[index];
    if (option === undefined || disabled) {
      return;
    }
    refs.current[index]?.focus();
    if (option.value !== value) {
      onChange(option.value);
    }
  };

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>): void => {
    const current = options.findIndex((option) => option.value === value);
    const last = options.length - 1;
    let next: number;
    switch (event.key) {
      case "ArrowRight":
      case "ArrowDown":
        next = current >= last ? 0 : current + 1;
        break;
      case "ArrowLeft":
      case "ArrowUp":
        next = current <= 0 ? last : current - 1;
        break;
      case "Home":
        next = 0;
        break;
      case "End":
        next = last;
        break;
      default:
        return;
    }
    event.preventDefault();
    choose(next);
  };

  return (
    <div role="radiogroup" aria-label={label} className={styles.segmented} onKeyDown={onKeyDown}>
      {options.map((option, index) => {
        const checked = option.value === value;
        const segment = (
          <button
            key={option.value}
            ref={(el) => {
              refs.current[index] = el;
            }}
            type="button"
            role="radio"
            className={styles.segment}
            aria-checked={checked}
            tabIndex={checked ? 0 : -1}
            disabled={disabled && (reason === undefined || !checked)}
            aria-disabled={reason !== undefined && checked ? true : undefined}
            onClick={() => {
              choose(index);
            }}
          >
            {option.icon}
            {option.label}
          </button>
        );
        return checked && reason !== undefined ? (
          <Tooltip key={option.value} content={reason}>
            {segment}
          </Tooltip>
        ) : (
          segment
        );
      })}
    </div>
  );
}
