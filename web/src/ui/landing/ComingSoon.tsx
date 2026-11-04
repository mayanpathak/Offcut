// Section 9: what is coming (designimplementation §5, §6). Every post-V6
// feature, each with the chip "Coming soon" and no date (rule 2.4): the next
// two first, then the rest. A footnote says that plans may change.

import { messages } from "../../copy/messages";
import { LANDING_CLIPS } from "../../net/asset-fetch";
import landing from "../styles/landing.module.css";
import styles from "../styles/landing-cards.module.css";
import sections from "../styles/landing-sections.module.css";
import { LoopVideo } from "./LoopVideo";
import { Chip, SectionHead } from "./parts";

export function ComingSoon() {
  const copy = messages.landing;
  const soon = copy.soon;
  return (
    <section className={landing.section} id="soon" aria-labelledby="soon-heading">
      <div className={landing.container}>
        <SectionHead eyebrow={copy.eyebrows.soon} heading={soon.heading} headingId="soon-heading" lead={soon.lead} />
        {LANDING_CLIPS.comingSoon === undefined ? null : (
          <div className={sections.wideFrame}>
            <LoopVideo clip={LANDING_CLIPS.comingSoon} label={copy.clipLabels.comingSoon} />
          </div>
        )}
        {[soon.next, soon.later].map((group, groupIndex) => (
          <div className={styles.soonGroup} key={group.title}>
            <h3 className={styles.soonGroupTitle}>{group.title}</h3>
            <ul className={groupIndex === 0 ? styles.soonGridNext : styles.soonGrid}>
              {group.items.map((item) => (
                <li className={styles.soonCard} key={item.title} data-testid="soon-card">
                  <Chip className={styles.soonChip} dot>
                    {soon.chip}
                  </Chip>
                  <h4 className={styles.soonTitle}>{item.title}</h4>
                  <p className={styles.soonLine}>{item.line}</p>
                </li>
              ))}
            </ul>
          </div>
        ))}
        <p className={styles.soonFootnote}>{soon.footnote}</p>
      </div>
    </section>
  );
}
