// Section 3: before and after (designimplementation §6). The raw take beside
// the finished short, and the three things that happened in between.

import { m } from "motion/react";

import { messages } from "../../copy/messages";
import { LANDING_CLIPS } from "../../net/asset-fetch";
import beforeAfterPoster from "../assets/landing/beforeAfter.webp";
import landing from "../styles/landing.module.css";
import styles from "../styles/landing-sections.module.css";
import { LoopVideo } from "./LoopVideo";
import { SectionHead, useRise } from "./parts";

/** "01", "02": a step's number, as a timeline would write it. */
const stepNumber = (index: number) => String(index + 1).padStart(2, "0");

export function BeforeAfter() {
  const copy = messages.landing;
  const rise = useRise();
  return (
    <section className={landing.section} id="before-after" aria-labelledby="before-after-heading">
      <div className={landing.container}>
        <SectionHead
          eyebrow={copy.eyebrows.beforeAfter}
          heading={copy.beforeAfter.heading}
          headingId="before-after-heading"
          lead={copy.beforeAfter.lead}
        />
        <div className={styles.wideFrame}>
          <LoopVideo clip={LANDING_CLIPS.beforeAfter} poster={beforeAfterPoster} label={copy.clipLabels.beforeAfter}>
            <div className={styles.beforeAfterStill} aria-hidden="true">
              <span className={styles.stillLandscape} />
              <span className={styles.stillArrow} />
              <span className={styles.stillPortrait} />
            </div>
          </LoopVideo>
        </div>
        <ol className={styles.steps}>
          {copy.beforeAfter.steps.map((step, index) => (
            <m.li key={step} className={styles.step} {...rise(index)}>
              <span className={styles.stepIndex}>{stepNumber(index)}</span>
              {step}
            </m.li>
          ))}
        </ol>
      </div>
    </section>
  );
}
