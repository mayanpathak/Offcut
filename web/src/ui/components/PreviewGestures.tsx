// The gestures on the preview (v4implementation D-183), over the canvas of
// `PreviewPlayer`: while the preview is paused, the cursor says what a press
// will do, a click on a caption word opens it, the caption block drags up
// and down with snaps, and the video of a wide clip drags sideways. Each
// drag commits once, on release (D-142). The outline of a word, the block's
// place, the snap line and the 9:16 frame are page chrome above the canvas,
// never drawn into a frame. The caption block is also a control the keyboard
// reaches (D-192). In development and test builds, `useCaptionMirror` gives
// the player the caption shown, for the tests (D-155).

import { type KeyboardEvent, type PointerEvent, useEffect, useLayoutEffect, useRef, useState } from "react";

import { DEV_HOOKS } from "../../config/env";
import { messages } from "../../copy/messages";
import { useClipStore } from "../../state/clip-store";
import { usePreviewStore } from "../../state/preview-store";
import {
  CAPTION_SNAPS,
  type CaptionGuide,
  captionGuide,
  type CaptionMirror,
  captionMirror,
  clickAt,
  dragCaptions,
  dragVideo,
  type Hit,
  hoverAt,
  hoverEnd,
  interactive,
  type VideoGuide,
  videoMoves,
} from "../../usecases/preview-interact";
import { setCaptionPosition } from "../../usecases/set-caption-position";
import { setCropOffset } from "../../usecases/set-crop-offset";
import styles from "../styles/preview.module.css";

/** A press that moves farther than this, in pixels, is a drag and not a click. */
const DRAG_START_PX = 4;
/** The arrow keys move the caption block by this much of its track (D-183). */
const CAPTION_KEY_STEP = 0.05;
/** How many times, and how far apart, the development mirror asks again when the worker was busy. */
const MIRROR_TRIES = 5;
const MIRROR_RETRY_MS = 100;
const PERCENT = 100;

/** A gesture on the canvas: a press that has not moved yet, or one of the two drags. */
type Gesture =
  | { kind: "press"; hit: Hit; x: number; y: number }
  | { kind: "captions"; fromY: number; guide: CaptionGuide }
  | { kind: "video"; fromX: number; guide: VideoGuide };

function pct(fraction: number): string {
  return `${String(fraction * PERCENT)}%`;
}

/** A piece of chrome over the canvas, placed by CSS custom properties: the page's CSP allows no inline style. */
function Chrome(props: { className: string | undefined; vars: Record<string, string>; testId?: string }) {
  const { className, vars, testId } = props;
  const element = useRef<HTMLSpanElement>(null);
  // After every render: setting a property to the value it has costs nothing.
  useLayoutEffect(() => {
    for (const [name, value] of Object.entries(vars)) {
      element.current?.style.setProperty(name, value);
    }
  });
  return <span ref={element} className={className} aria-hidden="true" data-testid={testId} />;
}

/** The caption block's place, and the snap line, while it is dragged or moved by the keys. */
function CaptionChrome({ guide }: { guide: CaptionGuide }) {
  return (
    <>
      <Chrome
        className={styles.blockGhost}
        vars={{ "--top": pct(guide.centre - guide.height / 2), "--height": pct(guide.height) }}
      />
      {guide.snap === null ? null : (
        <Chrome className={styles.snapGuide} vars={{ "--top": pct(guide.centre) }} testId="snap-guide" />
      )}
    </>
  );
}

/** What the handle's value says: the snap point's name, or how far down the block is. */
function placementText(p: number): string {
  const snap = CAPTION_SNAPS.find((s) => s.at === p);
  return snap === undefined ? messages.preview.captionAt(Math.round(p * PERCENT)) : messages.framing[snap.name];
}

/** The caption shown, as development and test builds mirror it (D-155); never asked in production. */
export function useCaptionMirror(): CaptionMirror | null | undefined {
  const position = usePreviewStore((state) => state.position);
  const status = usePreviewStore((state) => state.status);
  const seekBusy = usePreviewStore((state) => state.seekBusy);
  const edit = useClipStore((state) => state.edit);
  const clip = useClipStore((state) => state.status);
  const [mirror, setMirror] = useState<CaptionMirror | null | undefined>(undefined);
  useEffect(() => {
    if (!DEV_HOOKS) {
      return undefined;
    }
    let live = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const ask = (tries: number): void => {
      void captionMirror().then((answer) => {
        if (!live) {
          return;
        }
        // The worker was drawing a frame, or a seek was: ask again shortly (and once more when the seek ends).
        if (answer === undefined && tries > 1 && usePreviewStore.getState().status !== "playing") {
          timer = setTimeout(() => {
            ask(tries - 1);
          }, MIRROR_RETRY_MS);
          return;
        }
        setMirror(answer);
      });
    };
    ask(MIRROR_TRIES);
    return () => {
      live = false;
      clearTimeout(timer);
    };
  }, [position, status, seekBusy, edit, clip]);
  return mirror;
}

/** The pointer's place as fractions of the layer, which covers the canvas, and the layer's size on screen. */
function where(e: PointerEvent): { x: number; y: number; width: number; height: number } {
  const box = e.currentTarget.getBoundingClientRect();
  const x = (e.clientX - box.left) / box.width;
  return { x, y: (e.clientY - box.top) / box.height, width: box.width, height: box.height };
}

/** The class of the cursor: what a press there will do, or what the drag under way does. */
function cursorOf(gesture: Gesture | null, hover: Hit | null): string | undefined {
  const kind = gesture?.kind === "press" ? gesture.hit.kind : (gesture?.kind ?? hover?.kind);
  if (kind === "word" && gesture === null) {
    return styles.overWord;
  }
  if (kind === "word" || kind === "captions") {
    return styles.overCaptions;
  }
  return kind === "video" && videoMoves() ? styles.overVideo : undefined;
}

export function PreviewGestures() {
  const status = usePreviewStore((state) => state.status);
  const placement = useClipStore((state) => state.edit.caption_position);
  const [hover, setHover] = useState<Hit | null>(null);
  const [gesture, setGesture] = useState<Gesture | null>(null);
  /** Where the arrow keys have moved the block to, until the key is let go. */
  const [keyed, setKeyed] = useState<number | null>(null);

  // A player that goes drops the hover that waits for its frame.
  useEffect(() => hoverEnd, []);

  // Direct manipulation is for a paused preview: a play drops what a press would do.
  useEffect(() => {
    if (status === "playing") {
      setHover(null);
      setGesture(null);
    }
  }, [status]);

  // A hover is of the frame it was asked on: an edit or a seek draws another.
  const edit = useClipStore((state) => state.edit);
  const position = usePreviewStore((state) => state.position);
  useEffect(() => {
    setHover(null);
  }, [edit, position]);

  const onPointerDown = (e: PointerEvent<HTMLDivElement>): void => {
    // The cursor said what a press does: the hover's hit is the press's.
    if (e.button !== 0 || hover === null || !interactive()) {
      return;
    }
    e.currentTarget.setPointerCapture(e.pointerId);
    const { x, y } = where(e);
    setGesture({ kind: "press", hit: hover, x, y });
  };

  const onPointerMove = (e: PointerEvent<HTMLDivElement>): void => {
    const { x, y, width, height } = where(e);
    if (gesture === null) {
      hoverAt(x, y, setHover);
    } else if (gesture.kind === "captions") {
      setGesture({ ...gesture, guide: dragCaptions(gesture.fromY, y, height) });
    } else if (gesture.kind === "video") {
      const guide = dragVideo(gesture.fromX, x);
      setGesture(guide === null ? null : { ...gesture, guide });
    } else if (Math.hypot((x - gesture.x) * width, (y - gesture.y) * height) > DRAG_START_PX) {
      // A press that moves is a drag: of the captions from a word or the block, else of the video.
      if (gesture.hit.kind !== "video") {
        setGesture({ kind: "captions", fromY: gesture.y, guide: dragCaptions(gesture.y, y, height) });
      } else {
        const guide = dragVideo(gesture.x, x);
        setGesture(guide === null ? null : { kind: "video", fromX: gesture.x, guide });
      }
    }
  };

  const onPointerUp = (e: PointerEvent<HTMLDivElement>): void => {
    const ended = gesture;
    setGesture(null);
    if (ended?.kind === "press" && ended.hit.kind === "word") {
      void clickAt(ended.x, ended.y);
    } else if (ended?.kind === "captions") {
      // One drag, one change (D-142): committed here, on release.
      void setCaptionPosition(ended.guide.placement);
    } else if (ended?.kind === "video") {
      void setCropOffset(ended.guide.offset);
    }
    const { x, y } = where(e);
    hoverAt(x, y, setHover);
  };

  const onPointerLeave = (): void => {
    if (gesture === null) {
      hoverEnd();
      setHover(null);
    }
  };

  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>): void => {
    if (e.key !== "ArrowUp" && e.key !== "ArrowDown") {
      return;
    }
    e.preventDefault();
    if (!interactive()) {
      return;
    }
    const step = e.key === "ArrowUp" ? -CAPTION_KEY_STEP : CAPTION_KEY_STEP;
    // Whole hundredths: steps of 0.05 from a snap land on the others exactly.
    setKeyed(Math.round(Math.min(Math.max((keyed ?? placement) + step, 0), 1) * PERCENT) / PERCENT);
  };

  const onKeyUp = (e: KeyboardEvent<HTMLDivElement>): void => {
    if ((e.key === "ArrowUp" || e.key === "ArrowDown") && keyed !== null) {
      setKeyed(null);
      void setCaptionPosition(keyed);
    }
  };

  const shown = captionGuide(keyed ?? placement);
  const moving = gesture?.kind === "captions" ? gesture.guide : keyed === null ? null : shown;

  return (
    <>
      {/* The mouse's way; the handle below and the Framing tab are the keyboard's (D-192). */}
      <div
        className={`${styles.pointerLayer} ${cursorOf(gesture, hover) ?? ""}`}
        aria-hidden="true"
        data-testid="preview-pointer"
        data-hit={hover?.kind}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={() => {
          setGesture(null);
        }}
        onPointerLeave={onPointerLeave}
      />
      {hover?.kind === "word" && gesture === null ? (
        <Chrome
          className={styles.wordOutline}
          vars={{
            "--left": pct(hover.rect.x),
            "--top": pct(hover.rect.y),
            "--width": pct(hover.rect.w),
            "--height": pct(hover.rect.h),
          }}
          testId="word-outline"
        />
      ) : null}
      {moving === null ? null : <CaptionChrome guide={moving} />}
      {gesture?.kind === "video" ? (
        <>
          <span className={styles.frameOutline} aria-hidden="true" data-testid="frame-outline" />
          <span className={styles.cropMap} aria-hidden="true">
            <Chrome
              className={styles.cropWindow}
              vars={{ "--left": pct(gesture.guide.left), "--width": pct(gesture.guide.width) }}
            />
          </span>
        </>
      ) : null}
      <CaptionHandle guide={shown} onKeyDown={onKeyDown} onKeyUp={onKeyUp} />
    </>
  );
}

/** The caption block as a control the keyboard reaches with Tab: the arrow keys move it (D-183, D-192). */
function CaptionHandle(props: {
  guide: CaptionGuide;
  onKeyDown: (e: KeyboardEvent<HTMLDivElement>) => void;
  onKeyUp: (e: KeyboardEvent<HTMLDivElement>) => void;
}) {
  const element = useRef<HTMLDivElement>(null);
  const { guide } = props;
  useLayoutEffect(() => {
    element.current?.style.setProperty("--top", pct(guide.centre - guide.height / 2));
    element.current?.style.setProperty("--height", pct(guide.height));
  }, [guide.centre, guide.height]);
  return (
    <div
      ref={element}
      className={styles.captionHandle}
      role="slider"
      tabIndex={0}
      aria-label={messages.preview.captionHandle}
      aria-orientation="vertical"
      aria-valuemin={0}
      aria-valuemax={PERCENT}
      aria-valuenow={Math.round(guide.placement * PERCENT)}
      aria-valuetext={placementText(guide.placement)}
      data-testid="caption-handle"
      onKeyDown={props.onKeyDown}
      onKeyUp={props.onKeyUp}
    />
  );
}
