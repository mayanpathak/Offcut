// The three style cards (v4implementation D-186): each the clip's own frame
// in that style, drawn by the real renderer with the current edits, its name
// and one line; the chosen one has a ring and `aria-pressed`. What a card
// shows is what the export will show (D-172). Until the frames come, each
// card holds a skeleton.

import { useLayoutEffect, useRef } from "react";

import { messages } from "../../copy/messages";
import type { StyleId } from "../../gen/domain";
import { THUMB_HEIGHT, THUMB_WIDTH } from "../../usecases/style-thumbnails";
import { Skeleton } from "../primitives/Skeleton";
import { Tooltip } from "../primitives/Tooltip";
import styles from "../styles/panel.module.css";

/** In the order the worker draws them (`THUMBNAIL_STYLES` of `render/preview-job.ts`). */
const STYLES: readonly { id: StyleId; name: string; line: string }[] = [
  { id: "clean", name: messages.style.clean, line: messages.style.cleanLine },
  { id: "bold", name: messages.style.bold, line: messages.style.boldLine },
  { id: "tech", name: messages.style.mono, line: messages.style.monoLine },
];

/** A card's frame: the bitmap drawn on a canvas of its size, kept by the store (which closes it when replaced). */
function Frame({ bitmap }: { bitmap: ImageBitmap }) {
  const canvas = useRef<HTMLCanvasElement>(null);
  useLayoutEffect(() => {
    const context = canvas.current?.getContext("2d");
    context?.clearRect(0, 0, THUMB_WIDTH, THUMB_HEIGHT);
    context?.drawImage(bitmap, 0, 0, THUMB_WIDTH, THUMB_HEIGHT);
  }, [bitmap]);
  return <canvas ref={canvas} className={styles.thumbImage} width={THUMB_WIDTH} height={THUMB_HEIGHT} aria-hidden="true" />;
}

export function StylePicker(props: {
  value: StyleId;
  thumbnails: readonly ImageBitmap[] | null;
  onChoose: (style: StyleId) => void;
  /** Why the cards cannot be chosen now: an export is running. */
  disabledReason?: string;
}) {
  const { value, thumbnails, onChoose, disabledReason } = props;
  return (
    <ul className={styles.cards} data-testid="style-cards">
      {STYLES.map((style, i) => {
        const bitmap = thumbnails?.[i];
        const card = (
          <button
            type="button"
            className={style.id === value ? styles.styleCardOn : styles.styleCard}
            aria-pressed={style.id === value}
            aria-disabled={disabledReason === undefined ? undefined : true}
            onClick={() => {
              if (disabledReason === undefined) {
                onChoose(style.id);
              }
            }}
          >
            <span className={styles.thumb}>{bitmap === undefined ? <Skeleton size="block" /> : <Frame bitmap={bitmap} />}</span>
            <span className={styles.styleName}>{style.name}</span>
            <span className={styles.styleLine}>{style.line}</span>
          </button>
        );
        return <li key={style.id}>{disabledReason === undefined ? card : <Tooltip content={disabledReason}>{card}</Tooltip>}</li>;
      })}
    </ul>
  );
}
