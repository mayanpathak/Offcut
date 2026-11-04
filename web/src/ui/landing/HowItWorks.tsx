// Section 5: how it works (designimplementation §6). Three steps beside the
// how-it-works clip. On wide screens GSAP pins the section while the steps
// advance with the scroll (M9); otherwise, and under reduced motion, the
// steps stand stacked, every one of them in full.

import { useRef } from "react";

import { messages } from "../../copy/messages";
import { LANDING_CLIPS } from "../../net/asset-fetch";
import landing from "../styles/landing.module.css";
import styles from "../styles/landing-sections.module.css";
import { type GsapSetup, useGsap } from "./gsap";
import { LoopVideo } from "./LoopVideo";
import { SectionHead } from "./parts";

const stepNumber = (index: number) => String(index + 1).padStart(2, "0");

/** M9: the section stays pinned for this much of a screen's scroll (§8.2). */
const PIN_LENGTH = "+=150%";

/**
 * M9: on wide screens the section is pinned while the scroll moves its steps
 * on, one after another. Every step stays readable; the current one stands out.
 */
const pinSetup: GsapSetup = ({ trigger }, section) => {
  const steps = [...section.querySelectorAll<HTMLElement>("[data-step]")];
  const show = (current: number) => {
    steps.forEach((step, index) => {
      step.toggleAttribute("data-active", index === current);
    });
  };
  section.toggleAttribute("data-pinned", true);
  show(0);
  trigger.create({
    trigger: section,
    start: "top top",
    end: PIN_LENGTH,
    pin: true,
    onUpdate: (self) => {
      show(Math.min(steps.length - 1, Math.floor(self.progress * steps.length)));
    },
  });
  return () => {
    section.toggleAttribute("data-pinned", false);
    show(-1);
  };
};

export function HowItWorks() {
  const copy = messages.landing;
  const section = useRef<HTMLElement>(null);
  useGsap(section, pinSetup, "(min-width: 1024px)");
  return (
    <section className={[landing.section, styles.how].join(" ")} id="how" aria-labelledby="how-heading" ref={section}>
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
