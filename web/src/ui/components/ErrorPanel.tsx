// What went wrong, in words people use (v4implementation D-190): an inline
// card where the failure happened, never a modal. A short title, what
// happened in one sentence, what to do, and "Start over". A code with no
// words yet gets the general sentence; no code or detail is ever shown. The
// primary action (Try again, Reload preview) is wired in Prompt 119, with the
// words of the codes that do not have them yet.

import { messages } from "../../copy/messages";
import type { ErrorCode } from "../../gen/domain";
import { dismissClip } from "../../usecases/import-clip";
import { AlertTriangle } from "../icons";
import { Button } from "../primitives/Button";
import styles from "../styles/components.module.css";

type ErrorCopy = { title: string; body: string; action: string };

/** The words of a failure. The codes no person can meet yet have none (D-57). */
function errorCopy(code: ErrorCode | undefined): ErrorCopy | undefined {
  const copy: Partial<Record<ErrorCode, ErrorCopy>> = messages.errors;
  return code === undefined ? undefined : copy[code];
}

export type ErrorPanelProps = {
  code: ErrorCode | undefined;
  testId: string;
};

export function ErrorPanel({ code, testId }: ErrorPanelProps) {
  const copy = errorCopy(code);
  return (
    <section className={styles.card} role="alert" data-testid={testId}>
      <span className={styles.cardIconDanger}>
        <AlertTriangle />
      </span>
      <h2 className={styles.cardTitle}>{copy === undefined ? messages.editor.failedStub : copy.title}</h2>
      {copy === undefined ? null : (
        <>
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
