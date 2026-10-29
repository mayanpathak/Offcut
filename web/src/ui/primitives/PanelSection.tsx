// A titled part of the side panel (v4implementation D-181): a small upper-case
// title, an optional hint under it, and its controls, with the same padding
// everywhere.

import { type ReactNode, useId } from "react";

import styles from "../styles/primitives.module.css";

export type PanelSectionProps = {
  title: string;
  hint?: string;
  children: ReactNode;
};

export function PanelSection({ title, hint, children }: PanelSectionProps) {
  const titleId = useId();
  return (
    <section className={styles.panelSection} aria-labelledby={titleId}>
      <h2 id={titleId} className={styles.panelTitle}>
        {title}
      </h2>
      {hint === undefined ? null : <p className={styles.hint}>{hint}</p>}
      <div className={styles.panelBody}>{children}</div>
    </section>
  );
}
