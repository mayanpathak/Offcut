// J7: the preview. The canvas is drawn by the render worker and the sound is
// played by the page; `PreviewPlayer` hands the canvas over when it appears
// and gives the preview up when it goes. It fills the stage's frame, shows a
// spinner when a seek's frame is slow, and carries the last play's
// statistics as `data-*` for the tests (D-155). Over the canvas,
// `PreviewGestures` takes the direct manipulation of D-183. Under the frame,
// `PreviewTransport` offers play and pause, the time, the scrub bar (D-148)
// and the watermark badge (v4implementation D-172).

import { useEffect, useRef, useState } from "react";

import { DEV_HOOKS } from "../../config/env";
import { messages } from "../../copy/messages";
import { useClipStore } from "../../state/clip-store";
import { type PreviewState, usePreviewStore } from "../../state/preview-store";
import { attach, detach, pause, play, scrub, seekDone, seekStart } from "../../usecases/control-preview";
import { cachedPlan } from "../../usecases/start-export";
import { TRANSCRIPT_ID } from "./CaptionEditor";
import { Pause, Play } from "../icons";
import { IconButton } from "../primitives/IconButton";
import { Slider } from "../primitives/Slider";
import { Spinner } from "../primitives/Spinner";
import styles from "../styles/preview.module.css";
import { PreviewGestures, useCaptionMirror } from "./PreviewGestures";

// The size the preview is drawn at (TS §20.1). The page shows it scaled.
const PREVIEW_WIDTH = 540;
const PREVIEW_HEIGHT = 960;
/** A seek's frame that takes longer than this shows a spinner (D-148). */
const SEEK_SPINNER_MS = 300;
/** The scrub bar moves one output frame at a time. */
const FRAME_MS = 1000 / 30;
const CANVAS_WORDS_ID = "preview-canvas-words";

// An attach and a detach each take a moment, and a player that goes and
// comes back at once would otherwise have both under way together.
let lifecycle: Promise<void> = Promise.resolve();

/** Runs `step` when every step before it is over, however it ended. */
function inTurn(step: () => Promise<void>): void {
  const previous = lifecycle;
  let release = (): void => undefined;
  lifecycle = new Promise((resolve) => {
    release = resolve;
  });
  // Not caught: a fault of the code reaches the console, and the next step still gets its turn.
  void previous.then(step).finally(release);
}

/** What the one button says and does in each state of the preview. */
function control(preview: PreviewState): { label: string; press: (() => Promise<void>) | null } {
  const copy = messages.preview;
  switch (preview.status) {
    case "playing":
      return { label: copy.pause, press: pause };
    case "paused":
      return { label: copy.play, press: play };
    case "stopped":
      // Stopped after a play: the clip has reached its end.
      return { label: preview.playedOnce ? copy.replay : copy.play, press: play };
    case "detached":
    case "seeking":
    case "locked":
      return { label: copy.play, press: null };
    default:
      return assertNever(preview.status);
  }
}

function assertNever(value: never): never {
  void value;
  throw new Error("a state of the preview this build does not know");
}

/** Whether a seek has been drawing for longer than `SEEK_SPINNER_MS`. */
function useSlowSeek(): boolean {
  const busy = usePreviewStore((state) => state.seekBusy);
  const [slow, setSlow] = useState(false);
  useEffect(() => {
    if (!busy) {
      return undefined;
    }
    const timer = setTimeout(() => {
      setSlow(true);
    }, SEEK_SPINNER_MS);
    return () => {
      clearTimeout(timer);
      setSlow(false);
    };
  }, [busy]);
  return slow;
}

/** Whether the device holds a Creator token: the badge is for everyone else (D-172). */
function useCreator(): boolean {
  const [creator, setCreator] = useState(false);
  useEffect(() => {
    let live = true;
    void cachedPlan().then((plan) => {
      if (live) {
        setCreator(plan === "creator");
      }
    });
    return () => {
      live = false;
    };
  }, []);
  return creator;
}

export function PreviewPlayer() {
  const canvas = useRef<HTMLCanvasElement>(null);
  const stats = usePreviewStore((state) => state.lastStats);
  const placement = useClipStore((state) => state.edit.caption_position);
  const slow = useSlowSeek();
  const mirror = useCaptionMirror();

  // The canvas is handed over once, when the player appears: an element
  // gives its drawing away for good. The audio is the clip's, read then.
  useEffect(() => {
    const element = canvas.current;
    const { prog48 } = useClipStore.getState();
    if (element !== null && prog48 !== null) {
      inTurn(() => attach(element, prog48));
    }
    return () => {
      inTurn(detach);
    };
  }, []);

  // A tab that is hidden does not go on playing (TS §12.4).
  useEffect(() => {
    const onVisibility = (): void => {
      if (document.hidden && usePreviewStore.getState().status === "playing") {
        void pause();
      }
    };
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      document.removeEventListener("visibilitychange", onVisibility);
    };
  }, []);

  return (
    <div
      className={styles.player}
      data-frames={stats?.frames}
      data-late={stats?.late}
      data-max-drift-ms={stats?.maxDriftMs}
      data-downscaled={stats?.downscaled}
      // The mirrors of the visible caption, for the tests of development builds only (D-155).
      {...(DEV_HOOKS
        ? {
            "data-caption-text": mirror?.text,
            "data-caption-colors": mirror?.colors.join(" "),
            "data-caption-position": placement,
          }
        : {})}
    >
      <canvas
        ref={canvas}
        className={styles.previewCanvas}
        width={PREVIEW_WIDTH}
        height={PREVIEW_HEIGHT}
        role="img"
        aria-label={messages.preview.canvas}
        aria-describedby={CANVAS_WORDS_ID}
        aria-details={TRANSCRIPT_ID}
        data-testid="preview-canvas"
      />
      {/* The canvas's text alternative is the transcript editor (D-192). */}
      <span id={CANVAS_WORDS_ID} className={styles.srOnly}>
        {messages.preview.canvasWords}
      </span>
      <PreviewGestures />
      {slow ? (
        <span className={styles.seekSpinner} aria-hidden="true">
          <Spinner />
        </span>
      ) : null}
    </div>
  );
}

/**
 * Under the frame: the one button, where the preview is and the program's
 * length, the scrub bar (D-148), and the badge that says Free exports carry
 * the watermark, unless the device holds a Creator token (D-172). The
 * program is as long as the clip in V4 (the identity time map).
 */
export function PreviewTransport() {
  const preview = usePreviewStore();
  const lengthMs = useClipStore((state) => state.clipInfo?.duration) ?? 0;
  const creator = useCreator();
  const { label, press } = control(preview);
  const locked = preview.status === "locked";
  const position = preview.position ?? 0;

  return (
    <div className={styles.transport}>
      <div className={styles.transportRow}>
        <IconButton
          icon={preview.status === "playing" ? <Pause /> : <Play />}
          label={label}
          shortcut={{ keys: [messages.keys.space] }}
          disabled={press === null}
          {...(locked ? { disabledReason: messages.editor.availableAfterExport } : {})}
          onClick={() => {
            void press?.();
          }}
        />
        <span className={styles.time}>{messages.preview.time(position, lengthMs)}</span>
        <span className={styles.scrub}>
          <Slider
            min={0}
            max={lengthMs}
            step={FRAME_MS}
            value={position}
            label={messages.preview.seek}
            labelHidden
            format={messages.preview.clock}
            disabled={locked || preview.status === "detached"}
            {...(locked ? { disabledReason: messages.editor.availableAfterExport } : {})}
            onGrab={() => {
              void seekStart();
            }}
            onInput={(ms) => {
              void scrub(ms);
            }}
            onRelease={() => {
              void seekDone();
            }}
            onCommit={() => {
              void seekDone();
            }}
            data-testid="preview-scrub"
          />
        </span>
      </div>
      {creator ? null : (
        <p className={styles.badge} data-testid="watermark-badge">
          {messages.preview.watermarkBadge}
        </p>
      )}
    </div>
  );
}
