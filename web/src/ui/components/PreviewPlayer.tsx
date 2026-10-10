// J7: the preview. The canvas is drawn by the render worker and the sound is
// played by the page; this component hands the canvas over when it appears,
// gives the preview up when it goes, and offers play and pause.

import { useEffect, useRef } from "react";

import { messages } from "../../copy/messages";
import { useClipStore } from "../../state/clip-store";
import { type PreviewState, usePreviewStore } from "../../state/preview-store";
import { attach, detach, pause, play } from "../../usecases/control-preview";
import styles from "../styles/components.module.css";

// The size the preview is drawn at (TS §20.1). The page shows it scaled.
const PREVIEW_WIDTH = 540;
const PREVIEW_HEIGHT = 960;

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

export function PreviewPlayer() {
  const preview = usePreviewStore();
  const failure = useClipStore((state) => state.failure);
  const canvas = useRef<HTMLCanvasElement>(null);

  // The canvas is handed over once, when the player appears: an element
  // gives its drawing away for good. The audio is the clip's, read then.
  useEffect(() => {
    const element = canvas.current;
    const { out48 } = useClipStore.getState();
    if (element !== null && out48 !== null) {
      inTurn(() => attach(element, out48));
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

  const { label, press } = control(preview);

  return (
    <div className={styles.player}>
      <canvas
        ref={canvas}
        className={styles.previewCanvas}
        width={PREVIEW_WIDTH}
        height={PREVIEW_HEIGHT}
        data-testid="preview-canvas"
      />
      <div className={styles.playerControls}>
        <button
          className={styles.button}
          type="button"
          disabled={press === null}
          onClick={() => {
            void press?.();
          }}
        >
          {label}
        </button>
      </div>
      {failure !== undefined && (
        <p className={styles.statusError} role="alert">
          {messages.editor.failedStub}
        </p>
      )}
    </div>
  );
}
