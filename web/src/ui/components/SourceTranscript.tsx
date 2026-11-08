// V6.1's end screen (v6.1implementation §10.4): the video is transcribed,
// with its length and its sentences at their times, on screen only. V6.4
// replaces it with the list of shorts.

import { useId } from "react";

import { messages } from "../../copy/messages";
import { useSourceStore } from "../../state/source-store";
import { Button } from "../primitives/Button";
import styles from "../styles/components.module.css";
import long from "../styles/long.module.css";

export function SourceTranscript() {
  const transcript = useSourceStore((state) => state.transcript);
  const info = useSourceStore((state) => state.info);
  const copy = messages.long.done;
  const heading = useId();
  if (transcript === null) {
    return null;
  }
  return (
    <section className={styles.card} data-testid="source-transcript">
      <h2 id={heading} className={styles.cardTitle}>
        {copy.title}
      </h2>
      {info === null ? null : <span className={long.duration}>{messages.preview.clock(info.duration)}</span>}
      {/* It scrolls, so the keyboard can reach it: named by the heading. */}
      <ol className={long.sentences} data-testid="source-sentences" tabIndex={0} aria-labelledby={heading}>
        {transcript.sentences.map((sentence) => (
          <li key={sentence.start_ms} className={long.sentence}>
            <span className={long.time}>{messages.preview.clock(sentence.start_ms)}</span>
            <span>
              {transcript.words
                .slice(sentence.words.start, sentence.words.end)
                .map((word) => word.text)
                .join(" ")}
            </span>
          </li>
        ))}
      </ol>
      <p className={long.hint}>{copy.next}</p>
      <div className={styles.cardActions}>
        <Button
          variant="secondary"
          onClick={() => {
            useSourceStore.getState().clear();
          }}
        >
          {messages.editor.startOver}
        </Button>
      </div>
    </section>
  );
}
