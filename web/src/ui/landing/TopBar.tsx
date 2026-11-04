// Section 0: the top bar (designimplementation §6). The wordmark, the links to
// the page's sections, "Start free", and the 2 px progress bar of a short
// under it (D-317).

import { useMotionValueEvent, useScroll } from "motion/react";
import { useRef } from "react";

import { messages } from "../../copy/messages";
import styles from "../styles/landing.module.css";
import { CtaLink } from "./parts";

/** The bar shows its hairline once the page has scrolled this far (§6, section 0). */
const SCROLLED_PX = 8;

export function TopBar({ appPath }: { appPath: string }) {
  const copy = messages.landing;
  const bar = useRef<HTMLElement>(null);
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
