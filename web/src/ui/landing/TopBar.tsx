// Section 0: the top bar (designimplementation §6). The wordmark, the links to
// the page's sections, "Start free", and the 2 px progress bar of a short
// under it (D-317, M2). Also the playhead down the left gutter (M3).

import { useMotionValueEvent, useScroll } from "motion/react";
import { useRef } from "react";

import { messages } from "../../copy/messages";
import styles from "../styles/landing.module.css";
import { type GsapSetup, useGsap } from "./gsap";
import { CtaLink } from "./parts";

/** The bar shows its hairline once the page has scrolled this far (§6, section 0). */
const SCROLLED_PX = 8;
/** How closely the bar and the playhead follow the scroll (§8.2, M2 and M3). */
const SCRUB_S = 0.3;
const DOT_PX = 10;

/** M2: the progress bar fills as the page scrolls, like a short's. */
const progressSetup: GsapSetup = ({ gsap, trigger }, header) => {
  const bar = header.querySelector("[data-progress]");
  gsap.fromTo(
    bar,
    { scaleX: 0 },
    { scaleX: 1, ease: "none", scrollTrigger: { trigger: document.documentElement, start: 0, end: "max", scrub: SCRUB_S } },
  );
  // Fonts that arrive late move the sections: measure again once they are in.
  void document.fonts.ready.then(() => {
    trigger.refresh();
  });
  return undefined;
};

/** M3: the playhead's dot travels the gutter with the scroll, and lights each section's timecode. */
const playheadSetup: GsapSetup = ({ gsap, trigger }, line) => {
  const dot = line.querySelector("[data-playhead-dot]");
  gsap.to(dot, {
    y: () => line.clientHeight - DOT_PX,
    ease: "none",
    scrollTrigger: { trigger: document.documentElement, start: 0, end: "max", scrub: SCRUB_S, invalidateOnRefresh: true },
  });
  for (const eyebrow of document.querySelectorAll<HTMLElement>("[data-eyebrow]")) {
    trigger.create({
      trigger: eyebrow.closest("section") ?? eyebrow,
      start: "top center",
      end: "bottom center",
      toggleClass: { targets: eyebrow, className: styles.isLive ?? "" },
    });
  }
  return undefined;
};

/** M3: a thin violet line down the left gutter, on wide screens only (D-317). */
export function Playhead() {
  const line = useRef<HTMLDivElement>(null);
  useGsap(line, playheadSetup, "(min-width: 1280px)");
  return (
    <div className={styles.playhead} ref={line} aria-hidden="true">
      <span className={styles.playheadDot} data-playhead-dot />
    </div>
  );
}

export function TopBar({ appPath }: { appPath: string }) {
  const copy = messages.landing;
  const bar = useRef<HTMLElement>(null);
  useGsap(bar, progressSetup);
  const { scrollY } = useScroll();
  useMotionValueEvent(scrollY, "change", (y) => {
    bar.current?.toggleAttribute("data-scrolled", y > SCROLLED_PX);
  });
  return (
    <header className={styles.topBar} ref={bar}>
      <div className={[styles.container, styles.topBarInner].join(" ")}>
        <a className={styles.wordmark} href="#top" aria-label={copy.homeLink}>
          <span className={styles.wordmarkO}>{copy.wordmark.slice(0, 1)}</span>
          {copy.wordmark.slice(1)}
        </a>
        <nav className={styles.nav} aria-label={copy.nav.label}>
          <a href="#features">{copy.nav.features}</a>
          <a href="#how">{copy.nav.how}</a>
          <a href="#pricing">{copy.nav.pricing}</a>
          <a href="#soon">{copy.nav.soon}</a>
        </nav>
        <CtaLink className={styles.topBarCta} variant="primary" size="small" to={appPath}>
          {copy.cta.startFree}
        </CtaLink>
      </div>
      <div className={styles.progress} aria-hidden="true">
        <div className={styles.progressBar} data-progress />
      </div>
    </header>
  );
}
