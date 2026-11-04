// Section 11: the final call (designimplementation §6). One big line and the
// way in. GSAP puts the caption pill on its last word when it scrolls in
// (M10), an echo of the hero.

import { messages } from "../../copy/messages";
import landing from "../styles/landing.module.css";
import styles from "../styles/landing-sections.module.css";
import { CaptionedHeadline } from "./CaptionedHeadline";
import { CtaLink, Eyebrow } from "./parts";

export function FinalCall({ appPath }: { appPath: string }) {
  const copy = messages.landing;
  return (
    <section className={[landing.section, styles.final].join(" ")} id="start" aria-labelledby="final-heading">
      <div className={[landing.container, styles.finalInner].join(" ")}>
        <Eyebrow {...copy.eyebrows.final} />
        <CaptionedHeadline text={copy.final.line} restsOn={copy.final.restsOn} level={2} id="final-heading" className={styles.finalLine} />
        <CtaLink variant="primary" to={appPath}>
          {copy.cta.startFree}
        </CtaLink>
      </div>
    </section>
  );
}
