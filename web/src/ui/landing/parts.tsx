// The small parts the landing page's sections share (designimplementation
// §7.4). The editor's primitives are not reused, so that the editor stays
// untouched (§2.1). Motion lives here and in the sections, never outside
// ui/landing/ (D-319).

import { domAnimation, LazyMotion, MotionConfig } from "motion/react";
import type { ReactNode, Ref } from "react";
import { Link } from "react-router";

import styles from "../styles/landing.module.css";

/** Motion for the whole page: the light `m` component, and reduced motion as the visitor asks (§8.1). */
export function MotionScope({ children }: { children: ReactNode }) {
  return (
    <LazyMotion features={domAnimation} strict>
      <MotionConfig reducedMotion="user">{children}</MotionConfig>
    </LazyMotion>
  );
}

const join = (...names: (string | false | undefined)[]) => names.filter(Boolean).join(" ");

/** A section's timecode eyebrow (D-317): "00:07  WHAT IT DOES". */
export function Eyebrow({ time, label }: { time: string; label: string }) {
  return (
    <p className={styles.eyebrow} data-eyebrow>
      <span className={styles.timecode}>{time}</span>
      <span>{label}</span>
    </p>
  );
}

/** The eyebrow, the section's heading and its lead. */
export function SectionHead({
  eyebrow,
  heading,
  headingId,
  lead,
  center = false,
}: {
  eyebrow: { time: string; label: string };
  heading: string;
  headingId: string;
  lead?: string;
  center?: boolean;
}) {
  return (
    <div className={join(styles.sectionHead, center && styles.sectionHeadCenter)}>
      <Eyebrow {...eyebrow} />
      <h2 className={styles.h2} id={headingId}>
        {heading}
      </h2>
      {lead === undefined ? null : <p className={styles.sectionLead}>{lead}</p>}
    </div>
  );
}

type CtaProps = {
  variant: "primary" | "secondary";
  size?: "small" | "large";
  children: ReactNode;
  className?: string | undefined;
} & ({ to: string; href?: never } | { href: string; to?: never });

/** A call to action: a route of the app (`to`, React Router) or a place on this page (`href`). */
export function CtaLink({ variant, size = "large", children, className, ...target }: CtaProps) {
  const names = join(
    styles.cta,
    variant === "primary" ? styles.ctaPrimary : styles.ctaSecondary,
    size === "small" && styles.ctaSmall,
    className,
  );
  if (target.to !== undefined) {
    return (
      <Link className={names} to={target.to}>
        {children}
      </Link>
    );
  }
  return (
    <a className={names} href={target.href}>
      {children}
    </a>
  );
}

/** A phone drawn in CSS around a 9:16 screen. */
export function PhoneFrame({ children, frameRef, className }: { children: ReactNode; frameRef?: Ref<HTMLDivElement>; className?: string | undefined }) {
  return (
    <div className={join(styles.phone, className)} ref={frameRef}>
      <div className={styles.screen}>
        {children}
        <span className={styles.notch} aria-hidden="true" />
      </div>
    </div>
  );
}

/** A small label in the accent face, with an optional dot. */
export function Chip({ children, dot = false, className }: { children: ReactNode; dot?: boolean; className?: string | undefined }) {
  return <span className={join(styles.chip, dot && styles.chipDot, className)}>{children}</span>;
}
