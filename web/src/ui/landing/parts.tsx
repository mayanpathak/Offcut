// The small parts the landing page's sections share (designimplementation
// §7.4). The editor's primitives are not reused, so that the editor stays
// untouched (§2.1). Motion lives here and in the sections, never outside
// ui/landing/ (D-319).

import { domAnimation, LazyMotion, m, MotionConfig, useReducedMotion } from "motion/react";
import type { ReactNode, Ref } from "react";
import { Link } from "react-router";

import cards from "../styles/landing-cards.module.css";
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

/** The brand ease of tokens.css (`--ease`). */
export const EASE = [0.2, 0.8, 0.2, 1] as const;
/** M7: how far content rises into place, how long it takes, and the stagger between siblings (§8.2). */
const RISE_PX = 16;
const RISE_S = 0.6;
const STAGGER_S = 0.06;
/** The share of an element on screen that starts its entrance. */
const IN_VIEW = 0.3;

/** The props of M7's entrance for the `index`-th of a group of siblings. */
export type Rise = (index?: number) => {
  initial?: { y: number };
  whileInView?: { y: number };
  viewport?: { once: boolean; amount: number };
  transition?: { duration: number; ease: typeof EASE; delay: number };
};

/**
 * M7, the section entrances: content rises 16 px into place when it comes
 * into view, once. It only moves, it never fades (rule 2.5), and with
 * reduced motion it gets no animation at all.
 */
export function useRise(): Rise {
  const reduced = useReducedMotion() === true;
  return (index = 0) =>
    reduced
      ? {}
      : {
          initial: { y: RISE_PX },
          whileInView: { y: 0 },
          viewport: { once: true, amount: IN_VIEW },
          transition: { duration: RISE_S, ease: EASE, delay: index * STAGGER_S },
        };
}

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
  const rise = useRise();
  return (
    <m.div className={join(styles.sectionHead, center && styles.sectionHeadCenter)} {...rise()}>
      <Eyebrow {...eyebrow} />
      <h2 className={styles.h2} id={headingId}>
        {heading}
      </h2>
      {lead === undefined ? null : <p className={styles.sectionLead}>{lead}</p>}
    </m.div>
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

/** A card of the features' bento grid: its picture, then its title, its words and its chips. */
export function BentoCard({
  title,
  body,
  chips = [],
  index,
  className,
  children,
}: {
  title: string;
  body: string;
  chips?: readonly string[];
  /** The card's place in the grid, for M7's stagger. */
  index: number;
  className?: string | undefined;
  children: ReactNode;
}) {
  const rise = useRise();
  return (
    <m.article
      className={join(cards.card, className)}
      {...rise(index)}
      // M8: the spotlight follows the pointer, through two CSS variables
      // written with the CSS object model (the security policy allows it).
      onPointerMove={(event) => {
        const box = event.currentTarget.getBoundingClientRect();
        event.currentTarget.style.setProperty("--x", `${String(Math.round(event.clientX - box.left))}px`);
        event.currentTarget.style.setProperty("--y", `${String(Math.round(event.clientY - box.top))}px`);
      }}
    >
      {children}
      <div className={cards.cardText}>
        <h3 className={cards.cardTitle}>{title}</h3>
        <p className={cards.cardBody}>{body}</p>
        {chips.length === 0 ? null : (
          <ul className={cards.cardChips}>
            {chips.map((chip) => (
              <li key={chip}>
                <Chip>{chip}</Chip>
              </li>
            ))}
          </ul>
        )}
      </div>
    </m.article>
  );
}
