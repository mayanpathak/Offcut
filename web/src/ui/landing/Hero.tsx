// Section 1: the hero (designimplementation §6). The headline, what Offcut
// does, the way in, whether this browser can run it (J3), where it works and
// that the video is not uploaded; beside them, the hero clip in a phone.

import { Link } from "react-router";

import { messages } from "../../copy/messages";
import { LANDING_CLIPS } from "../../net/asset-fetch";
import { type CapabilityState, useCapabilityStore } from "../../state/capability-store";
import styles from "../styles/landing.module.css";
import { LoopVideo } from "./LoopVideo";
import { Chip, CtaLink, Eyebrow, PhoneFrame } from "./parts";

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

export function Hero({ appPath, settingsPath }: { appPath: string; settingsPath: string }) {
  const capability = capabilityLine(useCapabilityStore());
  const copy = messages.landing;
  return (
    <section className={styles.hero} id="top" aria-labelledby="hero-heading">
      <div className={styles.heroBackdrop} aria-hidden="true" />
      <div className={[styles.container, styles.heroGrid].join(" ")}>
        <div className={styles.heroCopy}>
          <Eyebrow {...copy.eyebrows.hero} />
          <h1 className={styles.h1} id="hero-heading">
            {copy.hero}
          </h1>
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

        <div className={styles.heroVisual}>
          <PhoneFrame>
            <LoopVideo clip={LANDING_CLIPS.hero} label={copy.clipLabels.hero} eager />
          </PhoneFrame>
          {copy.heroChips.map((chip) => (
            <Chip key={chip} className={styles.heroChip} dot>
              {chip}
            </Chip>
          ))}
        </div>
      </div>
    </section>
  );
}
