// Section 3: before and after (designimplementation §6). The raw take beside
// the finished short, and the three things that happened in between.

import { messages } from "../../copy/messages";
import { LANDING_CLIPS } from "../../net/asset-fetch";
import landing from "../styles/landing.module.css";
import styles from "../styles/landing-sections.module.css";
import { LoopVideo } from "./LoopVideo";
import { SectionHead } from "./parts";

/** "01", "02": a step's number, as a timeline would write it. */
const stepNumber = (index: number) => String(index + 1).padStart(2, "0");

export function BeforeAfter() {
  const copy = messages.landing;
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
          <LoopVideo clip={LANDING_CLIPS.beforeAfter} label={copy.clipLabels.beforeAfter}>
            <div className={styles.beforeAfterStill} aria-hidden="true">
              <span className={styles.stillLandscape} />
              <span className={styles.stillArrow} />
              <span className={styles.stillPortrait} />
            </div>
          </LoopVideo>
        </div>
        <ol className={styles.steps}>
          {copy.beforeAfter.steps.map((step, index) => (
            <li key={step} className={styles.step}>
              <span className={styles.stepIndex}>{stepNumber(index)}</span>
              {step}
            </li>
          ))}
        </ol>
      </div>
    </section>
  );
}
