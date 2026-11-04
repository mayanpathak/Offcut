// Section 1: the hero (designimplementation §6). The headline, what Offcut
// does, the way in, whether this browser can run it (J3), where it works and
// that the video is not uploaded; beside them, the hero clip in a phone.

import { m, useReducedMotion } from "motion/react";
import { useRef } from "react";
import { Link } from "react-router";

import { messages } from "../../copy/messages";
import { LANDING_CLIPS } from "../../net/asset-fetch";
import { type CapabilityState, useCapabilityStore } from "../../state/capability-store";
import styles from "../styles/landing.module.css";
import { CaptionedHeadline } from "./CaptionedHeadline";
import { type GsapSetup, useGsap } from "./gsap";
import { LoopVideo } from "./LoopVideo";
import { Chip, CtaLink, EASE, Eyebrow, PhoneFrame } from "./parts";

/** The one line that says whether this browser can run Offcut. */
function capabilityLine(state: CapabilityState): { text: string; supported: boolean } {
  if (state.status === "supported") {
    return { text: messages.capability.supported, supported: true };
  }
  if (state.status === "unsupported" && state.reason !== undefined) {
    return { text: messages.unsupported.reasons[state.reason], supported: false };
  }
  return { text: messages.capability.checking, supported: false };
}

/** M5: the chips enter one after another, then drift a few pixels for ever (§8.2). */
const CHIP_ENTER_S = 0.5;
const CHIP_FIRST_S = 0.5;
const CHIP_STAGGER_S = 0.12;
const CHIP_RISE_PX = 12;
const DRIFT_PX = 6;
const DRIFT_S = 6;

/** One of the three chips around the hero's phone: decoration, so it may fade in. */
function FloatingChip({ text, index }: { text: string; index: number }) {
  const reduced = useReducedMotion() === true;
  const chip = (
    <Chip className={styles.heroChipFace} dot>
      {text}
    </Chip>
  );
  if (reduced) {
    return <span className={styles.heroChip}>{chip}</span>;
  }
  return (
    <m.span
      className={styles.heroChip}
      initial={{ opacity: 0, y: CHIP_RISE_PX }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: CHIP_ENTER_S, delay: CHIP_FIRST_S + index * CHIP_STAGGER_S, ease: EASE }}
    >
      <m.span
        className={styles.heroChipDrift}
        animate={{ y: [0, -DRIFT_PX, 0, DRIFT_PX, 0] }}
        transition={{ duration: DRIFT_S + index, delay: CHIP_FIRST_S + CHIP_ENTER_S, repeat: Infinity, ease: "easeInOut" }}
      >
        {chip}
      </m.span>
    </m.span>
  );
}

/** M4: the phone turns towards the pointer by up to this much (§8.2). */
const TILT_DEG = 6;
const TILT_S = 0.6;
const PERSPECTIVE_PX = 1200;

/** M4: the phone follows a fine pointer over the hero; touch screens keep it still. */
const tiltSetup: GsapSetup = ({ gsap }, visual) => {
  const phone = visual.querySelector("[data-phone]");
  const hero = visual.closest("section");
  if (phone === null || hero === null) {
    return undefined;
  }
  gsap.set(phone, { transformPerspective: PERSPECTIVE_PX });
  const turnX = gsap.quickTo(phone, "rotationX", { duration: TILT_S, ease: "power3" });
  const turnY = gsap.quickTo(phone, "rotationY", { duration: TILT_S, ease: "power3" });
  const onMove = (event: PointerEvent) => {
    const box = visual.getBoundingClientRect();
    const across = Math.max(-1, Math.min(1, ((event.clientX - box.left) / box.width - 0.5) * 2));
    const down = Math.max(-1, Math.min(1, ((event.clientY - box.top) / box.height - 0.5) * 2));
    turnY(across * TILT_DEG);
    turnX(-down * TILT_DEG);
  };
  const onLeave = () => {
    turnX(0);
    turnY(0);
  };
  hero.addEventListener("pointermove", onMove);
  hero.addEventListener("pointerleave", onLeave);
  return () => {
    hero.removeEventListener("pointermove", onMove);
    hero.removeEventListener("pointerleave", onLeave);
  };
};

export function Hero({ appPath, settingsPath }: { appPath: string; settingsPath: string }) {
  const capability = capabilityLine(useCapabilityStore());
  const copy = messages.landing;
  const visual = useRef<HTMLDivElement>(null);
  useGsap(visual, tiltSetup, "(pointer: fine)");
  return (
    <section className={styles.hero} id="top" aria-labelledby="hero-heading">
      <div className={styles.heroBackdrop} aria-hidden="true" />
      <div className={[styles.container, styles.heroGrid].join(" ")}>
        <div className={styles.heroCopy}>
          <Eyebrow {...copy.eyebrows.hero} />
          <CaptionedHeadline text={copy.hero} restsOn={copy.heroRestsOn} level={1} id="hero-heading" className={styles.h1} />
          <p className={styles.heroLead}>{copy.subhead}</p>
          <div className={styles.ctaRow}>
            <CtaLink variant="primary" to={appPath}>
              {copy.cta.startFree}
            </CtaLink>
            <CtaLink variant="secondary" href="#how">
              {copy.cta.howItWorks}
            </CtaLink>
          </div>
          <p
            className={[styles.capability, capability.supported ? styles.capabilitySupported : undefined].filter(Boolean).join(" ")}
            role="status"
            data-testid="capability-line"
          >
            {capability.text}
          </p>
          <ul className={styles.facts}>
            <li>{copy.supportedBrowsers}</li>
            <li>
              {copy.privacyLine} <Link to={settingsPath}>{copy.whatLeavesLink}</Link>
            </li>
          </ul>
        </div>

        <div className={styles.heroVisual} ref={visual}>
          <div data-phone>
            <PhoneFrame>
              <LoopVideo clip={LANDING_CLIPS.hero} label={copy.clipLabels.hero} eager />
            </PhoneFrame>
          </div>
          {copy.heroChips.map((chip, index) => (
            <FloatingChip key={chip} text={chip} index={index} />
          ))}
        </div>
      </div>
    </section>
  );
}
