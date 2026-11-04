// Section 6: try it now (designimplementation §6). The drop zone is the
// unchanged `DropZone` of V1-V6, restyled only from this section's frame
// (§11): its classes are shared with the sign-in dialog and other pages.

import { messages } from "../../copy/messages";
import { LIMITS } from "../../gen/domain";
import { DropZone } from "../components/DropZone";
import landing from "../styles/landing.module.css";
import styles from "../styles/landing-sections.module.css";
import { SectionHead } from "./parts";

export function TryIt() {
  const copy = messages.landing;
  return (
    <section className={landing.section} id="try" aria-labelledby="try-heading">
      <div className={landing.container}>
        <SectionHead eyebrow={copy.eyebrows.try} heading={copy.try.heading} headingId="try-heading" lead={copy.try.limits(LIMITS)} center />
        <div className={styles.tryFrame}>
          <DropZone />
        </div>
      </div>
    </section>
  );
}
