// C-16: this browser cannot run Offcut. The page says exactly what is
// missing, where Offcut does work, shows the demo anyway, and offers to
// write when the visitor's platform is supported (PS §9.3).

import { messages } from "../../copy/messages";
import type { Wanted } from "../../gen/api";
import type { UnsupportedReason } from "../../gen/domain";
import { assetUrl, DEMO_VIDEO_PATH } from "../../net/asset-fetch";
import { NotifyMeForm } from "../components/NotifyMeForm";
import styles from "../styles/pages.module.css";

/** What an unsupported visitor can ask to be told about. "launch" is for the landing page. */
const PLATFORMS: readonly Wanted[] = ["safari", "firefox", "mobile", "linux"];
/** A visitor on a phone is most likely waiting for phones. */
const MOBILE_FIRST: { preselect: Wanted } = { preselect: "mobile" };

export function UnsupportedPage({ reason }: { reason: UnsupportedReason }) {
  const copy = messages.unsupported;
  return (
    <main className={styles.page}>
      <section className={styles.section}>
        <h1 className={styles.title}>{copy.title}</h1>
        <p className={styles.lead} data-testid="unsupported-reason">
          {copy.reasons[reason]}
        </p>
        <p className={styles.text}>{copy.supportedList}</p>
      </section>

      <section className={styles.section}>
        <h2 className={styles.heading}>{copy.demoHeading}</h2>
        <figure className={styles.demo}>
          <video
            className={styles.video}
            controls
            preload="metadata"
            crossOrigin="anonymous"
            src={assetUrl(DEMO_VIDEO_PATH)}
          />
          <figcaption className={styles.muted}>{messages.landing.demoCaption}</figcaption>
        </figure>
      </section>

      <NotifyMeForm wanted={PLATFORMS} {...(reason === "UNSUPPORTED_MOBILE" ? MOBILE_FIRST : {})} />
    </main>
  );
}
