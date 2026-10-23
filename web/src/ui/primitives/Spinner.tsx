// A turning arc: something is under way (v4implementation D-181). It is shown
// only for work that has already taken longer than 300 ms (§17.4); the
// control or region it sits in says what is busy (`aria-busy`), so the arc
// itself is hidden from assistive technology.

import styles from "../styles/primitives.module.css";

export type SpinnerProps = {
  size?: "sm" | "md";
};

export function Spinner({ size = "md" }: SpinnerProps) {
  return (
    <svg
      className={[styles.spinner, size === "sm" ? styles.spinnerSm : styles.spinnerMd].join(" ")}
      viewBox="0 0 24 24"
      fill="none"
      aria-hidden="true"
      focusable="false"
    >
      <circle className={styles.spinnerTrack} cx="12" cy="12" r="9" />
      <path className={styles.spinnerArc} d="M21 12a9 9 0 0 0-9-9" />
    </svg>
  );
}
