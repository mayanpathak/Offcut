// J1: the landing page. What it promises, the demo, where it works, that the
// video is not uploaded, and whether this browser can run it (J3). A clip
// dropped here is taken, and the page moves on to the editor with it
// (v2implementation D-48).

import { useEffect, useRef } from "react";
import { Link, useNavigate } from "react-router";

import { messages } from "../../copy/messages";
import { assetUrl, DEMO_VIDEO_PATH } from "../../net/asset-fetch";
import { type CapabilityState, useCapabilityStore } from "../../state/capability-store";
import { useClipStore } from "../../state/clip-store";
import { DropZone } from "../components/DropZone";
import { NotifyMeForm } from "../components/NotifyMeForm";
import styles from "../styles/pages.module.css";

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

export function LandingPage({ settingsPath, appPath }: { settingsPath: string; appPath: string }) {
  const capability = capabilityLine(useCapabilityStore());
  const copy = messages.landing;
  const navigate = useNavigate();
  const clipStatus = useClipStore((state) => state.status);
  const lastClipStatus = useRef(clipStatus);

  // The move to the editor follows a clip that arrives while this page is
  // shown. A clip that was there already does not send a visitor away who
  // came back to this page.
  useEffect(() => {
    const arrived = lastClipStatus.current === "idle" && clipStatus !== "idle";
    lastClipStatus.current = clipStatus;
    if (arrived) {
      void navigate(appPath);
    }
  }, [clipStatus, appPath, navigate]);

  return (
    <main className={styles.page}>
      <section className={styles.section}>
        <h1 className={styles.hero}>{copy.hero}</h1>
        <p className={styles.lead}>{copy.subhead}</p>
        <ul className={styles.facts}>
          <li>{copy.supportedBrowsers}</li>
          <li>
            {copy.privacyLine} <Link to={settingsPath}>{copy.whatLeavesLink}</Link>
          </li>
        </ul>
        <p
          className={capability.supported ? styles.supported : styles.muted}
          role="status"
          data-testid="capability-line"
        >
          {capability.text}
        </p>
      </section>

      <figure className={styles.demo}>
        <video
          className={styles.video}
          controls
          preload="metadata"
          crossOrigin="anonymous"
          src={assetUrl(DEMO_VIDEO_PATH)}
        />
        <figcaption className={styles.muted}>{copy.demoCaption}</figcaption>
      </figure>

      <DropZone />

      <section className={styles.section}>
        <h2 className={styles.heading}>{copy.waitlistHeading}</h2>
        <NotifyMeForm wanted={["launch"]} />
      </section>
    </main>
  );
}
