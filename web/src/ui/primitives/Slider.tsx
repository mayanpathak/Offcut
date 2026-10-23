// A value on a range (v4implementation D-181): a native range input, styled,
// with its label and its value above it. The value shown follows the thumb
// while it moves; `onCommit` is called once it is let go (the input's
// `change` event), and once per key press. The filled part of the track is
// set through a custom property: the CSP allows no style attribute.
// Disabled with a reason, it stays focusable and its tooltip says why.

import { useEffect, useId, useRef, useState } from "react";

import styles from "../styles/controls.module.css";
import { Tooltip } from "./Tooltip";

const PERCENT = 100;

export type SliderProps = {
  min: number;
  max: number;
  step: number;
  value: number;
  onCommit: (value: number) => void;
  label: string;
  /** The value in words, from messages.ts. Without it the number is shown. */
  format?: (value: number) => string;
  disabled?: boolean;
  /** Why the slider is disabled, shown as its tooltip. */
  disabledReason?: string;
  "data-testid"?: string;
};

export function Slider({
  min,
  max,
  step,
  value,
  onCommit,
  label,
  format = String,
  disabled = false,
  disabledReason,
  "data-testid": testId,
}: SliderProps) {
  const id = useId();
  const input = useRef<HTMLInputElement>(null);
  const [live, setLive] = useState(value);
  const commit = useRef(onCommit);
  const blocked = useRef(disabled);
  const reason = disabled ? disabledReason : undefined;

  useEffect(() => {
    commit.current = onCommit;
    blocked.current = disabled;
  }, [onCommit, disabled]);

  // A new value from outside (an undo, another control) moves the thumb.
  useEffect(() => {
    setLive(value);
  }, [value]);

  useEffect(() => {
    const share = max > min ? (live - min) / (max - min) : 0;
    input.current?.style.setProperty("--fill", `${String(share * PERCENT)}%`);
  }, [live, min, max]);

  // React's onChange is the input event; the commit is the native change event.
  useEffect(() => {
    const el = input.current;
    if (el === null) {
      return undefined;
    }
    const onChange = (): void => {
      if (!blocked.current) {
        commit.current(Number(el.value));
      }
    };
    el.addEventListener("change", onChange);
    return () => {
      el.removeEventListener("change", onChange);
    };
  }, []);

  const range = (
    <input
      ref={input}
      id={id}
      type="range"
      className={styles.range}
      min={min}
      max={max}
      step={step}
      value={live}
      aria-valuetext={format(live)}
      disabled={disabled && reason === undefined}
      aria-disabled={reason === undefined ? undefined : true}
      data-testid={testId}
      // Focusable while disabled with a reason, but neither a key nor the pointer moves it.
      onKeyDown={(event) => {
        if (disabled && event.key !== "Tab") {
          event.preventDefault();
        }
      }}
      onPointerDown={(event) => {
        if (disabled) {
          event.preventDefault();
        }
      }}
      onChange={(event) => {
        if (!disabled) {
          setLive(Number(event.target.value));
        }
      }}
    />
  );

  return (
    <div className={styles.slider}>
      <div className={styles.sliderHead}>
        <label htmlFor={id} className={styles.sliderLabel}>
          {label}
        </label>
        <output htmlFor={id} className={styles.sliderValue}>
          {format(live)}
        </output>
      </div>
      {reason === undefined ? range : <Tooltip content={reason}>{range}</Tooltip>}
    </div>
  );
}
