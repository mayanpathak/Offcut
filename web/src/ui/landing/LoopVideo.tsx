// The one player of the landing page's clips (designimplementation §9.5).
//
// A clip is decoration: silent, looping, hidden from assistive technology,
// with a visually hidden line saying what it shows; the section's text says
// the same (D-326). It plays only while a quarter of it is on screen and the
// tab is visible. Under reduced motion or Save-Data it does not play by
// itself: its poster shows with a "Play the clip" button. A slot without a
// clip shows its still, never a broken video (D-325).

import { useReducedMotion } from "motion/react";
import { type ReactNode, useEffect, useRef, useState } from "react";

import { messages } from "../../copy/messages";
import { assetUrl, type LandingClip } from "../../net/asset-fetch";
import { Play } from "../icons";
import styles from "../styles/landing.module.css";

/** Whether the browser asks to save data (`navigator.connection.saveData`, not in every browser). */
function savesData(): boolean {
  const connection: unknown = Reflect.get(navigator, "connection");
  return typeof connection === "object" && connection !== null && Reflect.get(connection, "saveData") === true;
}

/** A clip plays while this share of it is on screen. */
const PLAY_THRESHOLD = 0.25;
/** A clip starts loading this far before it reaches the screen. */
const LOAD_MARGIN = "200px";
const PLAY_ICON_SIZE = 16;

export function LoopVideo({
  clip,
  poster,
  label,
  eager = false,
  className,
  children,
}: {
  clip: LandingClip | undefined;
  /** The bundled WebP still, which also gives the video its box before it loads. */
  poster?: string;
  /** What the clip shows, for a reader that cannot see it. */
  label: string;
  /** The hero: it loads at once (`preload="metadata"`); the others load near the screen. */
  eager?: boolean;
  className?: string | undefined;
  /** The section's still without a clip. */
  children?: ReactNode;
}) {
  const video = useRef<HTMLVideoElement>(null);
  const reduced = useReducedMotion() === true;
  const [near, setNear] = useState(eager);
  const [pressed, setPressed] = useState(false);
  const [refused, setRefused] = useState(false);
  const autoplay = !reduced && !savesData();
  const playing = (autoplay && !refused) || pressed;

  // React does not write `muted` to the element before the first play, so it is set here.
  useEffect(() => {
    const element = video.current;
    if (element === null) {
      return;
    }
    element.muted = true;
    if (near) {
      return;
    }
    const loader = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          setNear(true);
          loader.disconnect();
        }
      },
      { rootMargin: LOAD_MARGIN },
    );
    loader.observe(element);
    return () => {
      loader.disconnect();
    };
  }, [near, clip]);

  useEffect(() => {
    const element = video.current;
    if (element === null || !near || !playing) {
      return;
    }
    let inView = false;
    const play = () => {
      if (inView && document.visibilityState === "visible") {
        element.muted = true;
        void element.play().catch((error: unknown): void => {
          // The browser refused to start it: the poster stays, with the button.
          // A play cut short by a pause (`AbortError`, a quick scroll past) is no refusal.
          if (error instanceof DOMException && error.name === "NotAllowedError") {
            setRefused(true);
          }
        });
      }
    };
    const viewer = new IntersectionObserver(
      (entries) => {
        inView = entries.at(-1)?.isIntersecting ?? false;
        if (inView) {
          play();
        } else {
          element.pause();
        }
      },
      { threshold: PLAY_THRESHOLD },
    );
    viewer.observe(element);
    const onVisibility = () => {
      if (document.visibilityState === "hidden") {
        element.pause();
      } else {
        play();
      }
    };
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      viewer.disconnect();
      document.removeEventListener("visibilitychange", onVisibility);
      element.pause();
    };
  }, [near, playing]);

  if (clip === undefined) {
    return (
      <div className={[styles.clip, className].filter(Boolean).join(" ")}>
        {children ?? (poster === undefined ? null : <img className={styles.clipMedia} src={poster} alt="" />)}
        <span className={styles.srOnly}>{label}</span>
      </div>
    );
  }

  return (
    <div className={[styles.clip, className].filter(Boolean).join(" ")} data-clip>
      <video
        ref={video}
        className={styles.clipMedia}
        muted
        playsInline
        loop
        crossOrigin="anonymous"
        disablePictureInPicture
        aria-hidden="true"
        poster={poster}
        width={clip.width}
        height={clip.height}
        preload={eager ? "metadata" : "none"}
        src={near ? assetUrl(clip.path) : undefined}
      />
      <span className={styles.srOnly}>{label}</span>
      {playing ? null : (
        <button
          className={styles.playClip}
          type="button"
          onClick={() => {
            setNear(true);
            setPressed(true);
          }}
        >
          <Play size={PLAY_ICON_SIZE} />
          <span>{messages.landing.playClip}</span>
        </button>
      )}
    </div>
  );
}
