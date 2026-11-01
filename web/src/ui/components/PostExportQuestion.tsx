// "Would you post this as is?" (PS §20.5; v5implementation D-234): three answers,
// and once one is given, a line of thanks in their place.

import { messages } from "../../copy/messages";
import type { PostExportAnswer } from "../../gen/api";
import { useExportStore } from "../../state/export-store";
import { answerPostExport } from "../../usecases/answer-post-export";
import { Button } from "../primitives/Button";
import styles from "../styles/components.module.css";

const ANSWERS: readonly { answer: PostExportAnswer; label: string }[] = [
  { answer: "yes", label: messages.export.question.yes },
  { answer: "small_edits", label: messages.export.question.smallEdits },
  { answer: "no", label: messages.export.question.no },
];

export function PostExportQuestion() {
  const answered = useExportStore((state) => state.answered);
  if (answered) {
    return (
      <p className={styles.outcomeThanks} data-testid="post-export-thanks">
        {messages.export.question.thanks}
      </p>
    );
  }
  return (
    <div className={styles.outcomeQuestion}>
      <p className={styles.outcomeAsk}>{messages.export.question.title}</p>
      <div className={styles.outcomeAnswers}>
        {ANSWERS.map(({ answer, label }) => (
          <Button
            key={answer}
            variant="secondary"
            size="sm"
            onClick={() => {
              answerPostExport(answer);
            }}
          >
            {label}
          </Button>
        ))}
      </div>
    </div>
  );
}
