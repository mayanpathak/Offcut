// Section 2: the marquee (designimplementation §6). One honest line of what
// Offcut does, looping (M6). It is decoration: hidden from assistive
// technology, and the same facts are in the features section. The line is
// drawn twice so that the loop has no seam; under reduced motion it stands
// still.

import { useRef } from "react";

import { messages } from "../../copy/messages";
import styles from "../styles/landing-sections.module.css";
import { type GsapSetup, useGsap } from "./gsap";

/** M6: one pass of the line takes this long; on hover it slows to this share of its speed (§8.2). */
const LOOP_S = 40;
const HOVER_SPEED = 0.3;
const EASE_S = 0.6;

/** M6: the line moves left for ever, by one copy of itself, so the loop has no seam. */
const marqueeSetup: GsapSetup = ({ gsap, trigger }, band) => {
  const track = band.querySelector("[data-marquee]");
  const loop = gsap.to(track, { xPercent: -50, duration: LOOP_S, ease: "none", repeat: -1, paused: true });
  // It moves only while it is on screen.
  trigger.create({
    trigger: band,
    start: "top bottom",
    end: "bottom top",
    onToggle: (self) => {
      if (self.isActive) {
        loop.play();
      } else {
        loop.pause();
      }
    },
  });
  // Block bodies: a tween is "thenable", and a listener returns nothing.
  const slow = () => {
    gsap.to(loop, { timeScale: HOVER_SPEED, duration: EASE_S });
  };
  const fast = () => {
    gsap.to(loop, { timeScale: 1, duration: EASE_S });
  };
  band.addEventListener("pointerenter", slow);
  band.addEventListener("pointerleave", fast);
  return () => {
    band.removeEventListener("pointerenter", slow);
    band.removeEventListener("pointerleave", fast);
  };
};

export function Marquee() {
  const scope = useRef<HTMLDivElement>(null);
  const line = messages.landing.marquee;
  useGsap(scope, marqueeSetup);
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
