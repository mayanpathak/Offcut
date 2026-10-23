// A clip Offcut cannot use: what the limit is and what to do (TS §11.3). A
// rejection is not an error, so there is no retry; the way on is another
// clip. The code is read from the clip store and only marks the panel for
// tests (`data-reject`); no code, size or name of the file is shown. A card
// in the stage's frame since V4 (v4implementation §17.2), V3's words unchanged.

import { messages } from "../../copy/messages";
import { useClipStore } from "../../state/clip-store";
import { dismissClip } from "../../usecases/import-clip";
import { AlertTriangle } from "../icons";
import { Button } from "../primitives/Button";
import styles from "../styles/components.module.css";

export function RejectionPanel() {
  const code = useClipStore((state) => state.reject);
  const copy = code === undefined ? undefined : messages.rejections[code];
  return (
    <section className={styles.card} role="alert" data-testid="clip-rejected" data-reject={code}>
      <span className={styles.cardIconWarn}>
        <AlertTriangle />
      </span>
      {copy !== undefined && (
        <>
          <h2 className={styles.cardTitle}>{copy.title}</h2>
          <p className={styles.cardText}>{copy.body}</p>
          <p className={styles.cardHint}>{copy.action}</p>
        </>
      )}
      <div className={styles.cardActions}>
        <Button
          variant="secondary"
          onClick={() => {
            void dismissClip();
          }}
        >
          {messages.editor.startOver}
        </Button>
      </div>
    </section>
  );
}
