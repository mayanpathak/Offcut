// Section 7: privacy (designimplementation §6, F18). Three facts, and a link
// to the full list. The hero's privacy line and its "What leaves your device"
// link are not repeated here: each must be on the page once (D-334).

import { Link } from "react-router";

import { messages } from "../../copy/messages";
import { LANDING_CLIPS } from "../../net/asset-fetch";
import { Check } from "../icons";
import landing from "../styles/landing.module.css";
import styles from "../styles/landing-sections.module.css";
import { LoopVideo } from "./LoopVideo";
import { SectionHead } from "./parts";

const CHECK_SIZE = 18;

export function Privacy({ settingsPath }: { settingsPath: string }) {
  const copy = messages.landing;
  return (
    <section className={landing.section} id="privacy" aria-labelledby="privacy-heading">
      <div className={landing.container}>
        <div className={styles.privacy}>
          <div className={styles.privacyText}>
            <SectionHead eyebrow={copy.eyebrows.privacy} heading={copy.privacy.heading} headingId="privacy-heading" />
            <ul className={styles.privacyFacts}>
              {copy.privacy.facts.map((fact) => (
                <li key={fact}>
                  <span className={styles.privacyCheck}>
                    <Check size={CHECK_SIZE} />
                  </span>
                  {fact}
                </li>
              ))}
            </ul>
            <Link className={styles.privacyLink} to={settingsPath}>
              {copy.privacy.fullList}
            </Link>
          </div>
          <div className={styles.wideFrame}>
            <LoopVideo clip={LANDING_CLIPS.privacy} label={copy.clipLabels.privacy}>
              <div className={styles.privacyStill} aria-hidden="true">
                <span className={styles.privacyShield} />
              </div>
            </LoopVideo>
          </div>
        </div>
      </div>
    </section>
  );
}
