// The processing card (v4implementation D-188): inside the stage's frame, the
// four steps in plain words (done ones ticked, the running ones turning, the
// rest muted), the moments found so far, the line that says where the work
// happens, and Cancel. The voice is cleaned while the captions are written,
// so two steps can run at once. Cancel and the cancelled card's two actions
// are wired in Prompt 119.

import { messages } from "../../copy/messages";
import type { PipelineStage } from "../../gen/domain";
import { type ClipState, useClipStore } from "../../state/clip-store";
import { Check } from "../icons";
import { Button } from "../primitives/Button";
import { Spinner } from "../primitives/Spinner";
import styles from "../styles/components.module.css";
import { ProcessingFeed } from "./ProcessingFeed";

type Step = keyof typeof messages.processing.steps;
type StepState = "done" | "running" | "waiting";

export const STEPS: readonly Step[] = ["audio", "voice", "captions", "visuals"];

const STEP_CLASS = { done: styles.stepDone, running: styles.stepRunning, waiting: styles.step } as const;

/**
 * Where each step is, from the pipeline's stage and its feed: the audio is
 * ready once the voice is being cleaned; the voice and the captions are done
 * once the visuals are being looked for.
 */
export function stepStates(stage: PipelineStage | undefined, feed: ClipState["feed"]): Record<Step, StepState> {
  const voice = feed.some((line) => line.kind === "cleaning_voice");
  const visuals = stage === "detect_scene";
  return {
    audio: voice || stage === "asr" || visuals ? "done" : "running",
    voice: visuals ? "done" : voice ? "running" : "waiting",
    captions: visuals ? "done" : stage === "asr" ? "running" : "waiting",
    visuals: visuals ? "running" : "waiting",
  };
}

function StepMark({ state }: { state: StepState }) {
  return (
    <span className={styles.stepMark} aria-hidden="true">
      {state === "done" ? <Check size={16} /> : state === "running" ? <Spinner size="sm" /> : null}
    </span>
  );
}

function Cancelled() {
  return (
    <section className={styles.card} data-testid="clip-cancelled">
      <h2 className={styles.cardTitle}>{messages.cancel.cancelled}</h2>
      <div className={styles.cardActions}>
        <Button variant="primary">{messages.cancel.processAgain}</Button>
        <Button variant="secondary">{messages.editor.startOver}</Button>
      </div>
    </section>
  );
}

export function ProcessingCard({ cancelled }: { cancelled: boolean }) {
  const stage = useClipStore((state) => state.stage);
  const feed = useClipStore((state) => state.feed);
  if (cancelled) {
    return <Cancelled />;
  }
  const states = stepStates(stage, feed);
  return (
    <section className={styles.card} aria-busy="true" data-testid="processing">
      <h2 className={styles.cardTitle}>{messages.processing.title}</h2>
      <ol className={styles.steps}>
        {STEPS.map((step) => (
          <li key={step} className={STEP_CLASS[states[step]]}>
            <StepMark state={states[step]} />
            {messages.processing.steps[step]}
          </li>
        ))}
      </ol>
      <ProcessingFeed />
      <p className={styles.cardHint}>{messages.processing.reassure}</p>
      <div className={styles.cardActions}>
        <Button variant="ghost" size="sm">
          {messages.cancel.button}
        </Button>
      </div>
    </section>
  );
}
