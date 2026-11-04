// Section 2: the marquee (designimplementation §6). One honest line of what
// Offcut does, looping (M6). It is decoration: hidden from assistive
// technology, and the same facts are in the features section. The line is
// drawn twice so that the loop has no seam; under reduced motion it stands
// still.

import { useRef } from "react";

import { messages } from "../../copy/messages";
import styles from "../styles/landing-sections.module.css";

export function Marquee() {
  const scope = useRef<HTMLDivElement>(null);
  const line = messages.landing.marquee;
  return (
    <div className={styles.marquee} ref={scope} aria-hidden="true">
      <div className={styles.marqueeTrack} data-marquee>
        {[0, 1].map((copy) => (
          <ul className={styles.marqueeLine} key={copy}>
            {line.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        ))}
      </div>
    </div>
  );
}
