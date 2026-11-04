// M1 and M10: a headline that plays like an Offcut Clean caption
// (designimplementation D-317). Each word lights up in turn under the yellow
// pill, which slides from word to word and rests on the key word.
//
// The words are spans with real spaces between them, so the heading's
// accessible name stays the sentence. The pill is decoration: hidden from
// assistive technology, and drawn only once GSAP has loaded and movement is
// welcome. Without it the headline is plain white text (rule 2.5).

import { Fragment, useRef } from "react";

import styles from "../styles/landing.module.css";
import { type GsapSetup, useGsap } from "./gsap";

/** How long the pill takes to reach a word, in the hero and in the final call (§8.2). */
const STEP_S = 0.28;
const FINAL_STEP_S = 0.16;
const START_S = 0.35;
/** The pill reaches past a word by this share of the font size on each side, and covers this share of its height. */
const PAD_X_EM = 0.12;
const TOP_SHARE = 0.14;
const HEIGHT_SHARE = 0.8;

/** The pill's place over a word, relative to the heading. */
function pillBox(word: HTMLElement, heading: HTMLElement) {
  const box = word.getBoundingClientRect();
  const outer = heading.getBoundingClientRect();
  const padX = parseFloat(getComputedStyle(heading).fontSize) * PAD_X_EM;
  return {
    x: box.left - outer.left - padX,
    y: box.top - outer.top + box.height * TOP_SHARE,
    width: box.width + 2 * padX,
    height: box.height * HEIGHT_SHARE,
  };
}

/** The timeline of the pill, from the first word to the one it rests on. */
function captionSetup(restsOn: string, mode: "hero" | "final"): GsapSetup {
  return ({ gsap }, heading) => {
    const words = [...heading.querySelectorAll<HTMLElement>("[data-word]")];
    const pill = heading.querySelector<HTMLElement>("[data-pill]");
    const found = words.map((word) => word.textContent).lastIndexOf(restsOn);
    const rest = found < 0 ? words.length - 1 : found;
    const restWord = words[rest];
    if (pill === null || restWord === undefined) {
      return undefined;
    }
    const light = (index: number) => {
      words.forEach((word, other) => {
        word.toggleAttribute("data-lit", other === index);
      });
    };

    const timeline = gsap.timeline(
      mode === "hero"
        ? { delay: START_S }
        : { scrollTrigger: { trigger: heading, start: "top 60%", once: true } },
    );
    words.slice(0, rest + 1).forEach((word, index) => {
      const box = () => pillBox(word, heading);
      if (index === 0) {
        timeline.set(pill, { x: () => box().x, y: () => box().y, width: () => box().width, height: () => box().height, opacity: 1 });
      } else {
        timeline.to(pill, {
          x: () => box().x,
          y: () => box().y,
          width: () => box().width,
          height: () => box().height,
          duration: mode === "hero" ? STEP_S : FINAL_STEP_S,
          ease: "power3.out",
        });
      }
      timeline.call(() => {
        light(index);
      });
    });

    // A new width moves the words: once the pill rests, it follows its word.
    const onResize = () => {
      if (timeline.progress() === 1) {
        gsap.set(pill, pillBox(restWord, heading));
      }
    };
    window.addEventListener("resize", onResize);
    return () => {
      window.removeEventListener("resize", onResize);
      light(-1);
    };
  };
}

export function CaptionedHeadline({
  text,
  restsOn,
  level,
  id,
  className,
}: {
  text: string;
  /** The word the pill rests on, as it is written in `text`. */
  restsOn: string;
  level: 1 | 2;
  id: string;
  className: string | undefined;
}) {
  const heading = useRef<HTMLHeadingElement>(null);
  useGsap(heading, captionSetup(restsOn, level === 1 ? "hero" : "final"));
  const Tag = level === 1 ? "h1" : "h2";
  // The words and the spaces between them, in order.
  const parts = text.split(/(\s+)/);
  return (
    <Tag className={className} id={id} ref={heading}>
      {parts.map((part, index) =>
        part.trim() === "" ? (
          <Fragment key={index}>{part}</Fragment>
        ) : (
          <span key={index} className={styles.captionWord} data-word>
            {part}
          </span>
        ),
      )}
      <span className={styles.captionPill} aria-hidden="true" data-pill />
    </Tag>
  );
}
