// A placeholder in the shape of content that is not there yet (v4implementation
// D-181): a line of text, a short line, or a block. Its shimmer stops under
// reduced motion. It is decoration; the region that holds it says it is busy.

import styles from "../styles/primitives.module.css";

const SIZE_CLASS = {
  line: styles.skeletonLine,
  short: styles.skeletonShort,
  block: styles.skeletonBlock,
} as const;

export type SkeletonProps = {
  size?: keyof typeof SIZE_CLASS;
};

export function Skeleton({ size = "line" }: SkeletonProps) {
  return <span className={[styles.skeleton, SIZE_CLASS[size]].join(" ")} aria-hidden="true" />;
}
