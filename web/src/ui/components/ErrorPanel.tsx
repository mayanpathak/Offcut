// What went wrong, in words people use (v4implementation D-190): an inline
// card where the failure happened, never a modal. A short title, what
// happened in one sentence, what to do, one primary action chosen by where
// the failure happened, and a secondary one. A code with no words yet gets
// the general sentence; no code, stage or file name is ever shown, except in
// a development build's collapsed "Details". The first time a failure is
// drawn, `client_error` is sent for it (D-153), once.

import { useEffect } from "react";

import { DEV_HOOKS } from "../../config/env";
import { messages } from "../../copy/messages";
import type { ErrorCode } from "../../gen/domain";
import { type ClipState, useClipStore } from "../../state/clip-store";
import { failureShown } from "../../usecases/cancel-job";
import { reloadPreview } from "../../usecases/control-preview";
import { dismissClip } from "../../usecases/import-clip";
import { undoFailedChange } from "../../usecases/review-update";
import { runPipeline } from "../../usecases/run-pipeline";
import { dismissExport, startExport } from "../../usecases/start-export";
import { AlertTriangle } from "../icons";
import { Button } from "../primitives/Button";
import styles from "../styles/components.module.css";

type ErrorCopy = { title: string; body: string; action: string };
type Failure = NonNullable<ClipState["failure"]>;

/**
 * Where the failure happened: a clip that failed on its way, the preview
 * of a ready clip, a review change, or an export.
 */
export type ErrorContext = "clip" | "preview" | "edit" | "export";

/** The words of a failure. The codes no person can meet yet have none (D-57). */
function errorCopy(code: ErrorCode | undefined): ErrorCopy | undefined {
  const copy: Partial<Record<ErrorCode, ErrorCopy>> = messages.errors;
  return code === undefined ? undefined : copy[code];
}

/** The failures of a ready clip that only a new session mends, whatever was being done (D-153). */
const RELOAD_CODES: ReadonlySet<ErrorCode> = new Set(["E_GPU_LOST", "E_GPU_INIT", "E_WORKER_CRASH"]);

/** What the failure of a ready clip asks for: the change undone, or the preview reloaded. */
export function readyContext(failure: Failure, during: "edit" | "preview" | undefined): ErrorContext {
  return during === "edit" && !RELOAD_CODES.has(failure.code) ? "edit" : "preview";
}

type Action = { label: string; run: () => void };

const startOver: Action = {
  label: messages.editor.startOver,
  run: () => {
    void dismissClip();
  },
};

/** The card's two actions (D-190): the one that mends this failure, and the way out. */
function actionsFor(context: ErrorContext): { primary: Action | undefined; secondary: Action | undefined } {
  const { clipId, clipInfo } = useClipStore.getState();
  switch (context) {
    case "clip":
      // A clip that was never brought in has nothing to run again: a new one is the way.
      if (clipId === undefined || clipInfo === undefined) {
        return { primary: startOver, secondary: undefined };
      }
      return {
        primary: {
          label: messages.recovery.tryAgain,
          run: () => {
            void runPipeline(clipId);
          },
        },
        secondary: startOver,
      };
    case "preview":
      return {
        primary: {
          label: messages.recovery.reloadPreview,
          run: () => {
            void reloadPreview();
          },
        },
        secondary: startOver,
      };
    case "edit":
      return {
        primary: {
          label: messages.recovery.undoChange,
          run: () => {
            void undoFailedChange();
          },
        },
        secondary: startOver,
      };
    case "export":
      // The clip is fine: the way out of a failed export is back to the editor, not a new clip.
      return {
        primary:
          clipId === undefined
            ? undefined
            : {
                label: messages.recovery.tryAgain,
                run: () => {
                  void startExport(clipId);
                },
              },
        secondary: { label: messages.recovery.close, run: dismissExport },
      };
    default:
      return assertNever(context);
  }
}

function assertNever(value: never): never {
  void value;
  throw new Error("a place of failure this build does not know");
}

export type ErrorPanelProps = {
  failure: Failure | undefined;
  context: ErrorContext;
  testId: string;
  /** A smaller card, for the side panel and over the preview. */
  compact?: boolean;
};

export function ErrorPanel({ failure, context, testId, compact = false }: ErrorPanelProps) {
  const copy = errorCopy(failure?.code);
  useEffect(() => {
    if (failure !== undefined) {
      failureShown(failure);
    }
  }, [failure]);
  const { primary, secondary } = actionsFor(context);
  // The action line is left out when it only repeats the button (E_GPU_LOST's "Reload preview").
  const action = copy === undefined || copy.action === primary?.label ? undefined : copy.action;
  return (
    <section className={compact ? styles.cardCompact : styles.card} role="alert" data-testid={testId}>
      <span className={styles.cardIconDanger}>
        <AlertTriangle />
      </span>
      <h2 className={styles.cardTitle}>{copy === undefined ? messages.editor.failedStub : copy.title}</h2>
      {copy === undefined ? null : <p className={styles.cardText}>{copy.body}</p>}
      {action === undefined ? null : <p className={styles.cardHint}>{action}</p>}
      <div className={styles.cardActions}>
        {primary === undefined ? null : (
          <Button variant="primary" onClick={primary.run}>
            {primary.label}
          </Button>
        )}
        {secondary === undefined ? null : (
          <Button variant="secondary" onClick={secondary.run}>
            {secondary.label}
          </Button>
        )}
      </div>
      {DEV_HOOKS && failure !== undefined ? (
        <details className={styles.cardDetails}>
          <summary>{messages.recovery.details}</summary>
          <code>{[failure.code, failure.stage, failure.detail].filter((part) => part !== undefined).join(" · ")}</code>
        </details>
      ) : null}
    </section>
  );
}
