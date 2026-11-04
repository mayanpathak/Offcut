// Section 10: the update list (designimplementation §6, D-328). The unchanged
// `NotifyMeForm` with `wanted={["launch"]}`: the same form and the same wire
// as the old waitlist. It is the page's one <form> (§12.1).

import { messages } from "../../copy/messages";
import { NotifyMeForm } from "../components/NotifyMeForm";
import landing from "../styles/landing.module.css";
import styles from "../styles/landing-sections.module.css";
import { Eyebrow } from "./parts";

export function Updates() {
  const copy = messages.landing;
  return (
    <section className={landing.section} id="updates" aria-labelledby="updates-heading">
      <div className={landing.container}>
        <div className={styles.updates}>
          <div className={styles.updatesText}>
            <Eyebrow {...copy.eyebrows.updates} />
            <h2 className={styles.updatesHeading} id="updates-heading">
              {copy.updatesHeading}
            </h2>
          </div>
          <NotifyMeForm wanted={["launch"]} />
        </div>
      </div>
    </section>
  );
}
