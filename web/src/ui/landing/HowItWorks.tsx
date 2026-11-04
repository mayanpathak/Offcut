// Section 5: how it works (designimplementation §6). Three steps beside the
// how-it-works clip. On wide screens GSAP pins the section while the steps
// advance with the scroll (M9); otherwise, and under reduced motion, the
// steps stand stacked, every one of them in full.

import { messages } from "../../copy/messages";
import { LANDING_CLIPS } from "../../net/asset-fetch";
import landing from "../styles/landing.module.css";
import styles from "../styles/landing-sections.module.css";
import { LoopVideo } from "./LoopVideo";
import { SectionHead } from "./parts";

const stepNumber = (index: number) => String(index + 1).padStart(2, "0");

export function HowItWorks() {
  const copy = messages.landing;
  return (
    <section className={[landing.section, styles.how].join(" ")} id="how" aria-labelledby="how-heading" data-how>
      <div className={landing.container}>
        <div className={styles.howGrid}>
          <div className={styles.howText}>
            <SectionHead eyebrow={copy.eyebrows.how} heading={copy.how.heading} headingId="how-heading" lead={copy.how.lead} />
            <ol className={styles.howSteps}>
              {copy.how.steps.map((step, index) => (
                <li key={step.title} className={styles.howStep} data-step={index}>
                  <span className={styles.stepIndex}>{stepNumber(index)}</span>
                  <div>
                    <h3 className={styles.howStepTitle}>{step.title}</h3>
                    <p className={styles.howStepBody}>{step.body}</p>
                  </div>
                </li>
              ))}
            </ol>
          </div>
          <div className={styles.wideFrame}>
            <LoopVideo clip={LANDING_CLIPS.howItWorks} label={copy.clipLabels.howItWorks}>
              <div className={styles.appStill} aria-hidden="true">
                <span className={styles.appStillBar} />
                <span className={styles.appStillStage} />
                <span className={styles.appStillPanel} />
              </div>
            </LoopVideo>
          </div>
        </div>
      </div>
    </section>
  );
}
