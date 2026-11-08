// A long video's work (v6.1implementation §10.3): two stages with their own
// bars, "Reading the sound" and "Transcribing N of M", the time left once a
// stage is 10% done, and Cancel; paused, the reason and Continue; waiting for
// the file, a place to drop it and why. Only real progress is shown (J6).

import { useEffect, useRef } from "react";

import { messages } from "../../copy/messages";
import { useAuthStore } from "../../state/auth-store";
import { type SourceProgress as Progress, useSourceStore } from "../../state/source-store";
import { continueSource, dropSameFile } from "../../usecases/continue-source";
import { cancelSource } from "../../usecases/run-source-pipeline";
import { windowCount } from "../../usecases/transcribe-source";
import { Button } from "../primitives/Button";
import styles from "../styles/components.module.css";
import long from "../styles/long.module.css";
import { DropZone } from "./DropZone";

/** A stage's share, 0 to 100. */
function percentOf(p: Progress | null): number {
  return p === null || p.total <= 0 ? 0 : Math.min(100, (p.done / p.total) * 100);
}

/** Seconds left, once a tenth is done; `undefined` before. */
function secondsLeft(p: Progress | null): number | undefined {
  if (p === null || p.total <= 0 || p.done / p.total < 0.1) {
    return undefined;
  }
  const elapsed = (Date.now() - p.startedAt) / 1000;
  return Math.max(1, Math.round((elapsed * (p.total - p.done)) / p.done));
}

function Meter({ percent, label }: { percent: number; label: string }) {
  const bar = useRef<HTMLDivElement>(null);
  useEffect(() => {
    bar.current?.style.setProperty("--progress", `${String(percent)}%`);
  }, [percent]);
  return (
    <div className={styles.meter} role="progressbar" aria-label={label} aria-valuenow={Math.round(percent)} aria-valuemin={0} aria-valuemax={100}>
      <div ref={bar} className={styles.meterBar} />
    </div>
  );
}

export function SourceProgress({ onSignIn }: { onSignIn: () => void }) {
  const status = useSourceStore((state) => state.status);
  const progress = useSourceStore((state) => state.progress);
  const paused = useSourceStore((state) => state.paused);
  const differentFile = useSourceStore((state) => state.differentFile);
  const signedIn = useAuthStore((state) => state.status === "signed_in");
  const copy = messages.long;

  const transcribing = status === "transcribing" || (status === "paused" && paused?.from === "transcribing");
  const audio = progress?.stage === "audio" ? progress : null;
  const windows = progress?.stage === "transcribe" ? progress : null;
  // Before the windows start, their number is known from the duration (D-405): never "0 of 0".
  const info = useSourceStore((state) => state.info);
  const windowsTotal = windows?.total ?? (info === null ? 0 : windowCount(info.duration));
  const transcribeLabel = copy.stage.transcribe(windows?.done ?? 0, windowsTotal);
  const left = secondsLeft(progress);
  const working = status === "extracting_audio" || status === "transcribing";

  if (status === "awaiting_file") {
    return (
      <section className={styles.card} data-testid="source-awaiting-file">
        <p className={styles.cardText}>{copy.awaitingFile}</p>
        {differentFile ? (
          <p className={long.warn} role="alert">
            {copy.differentFile}
          </p>
        ) : null}
        <div className={long.stageSlot}>
          <DropZone
            variant="stage"
            kind="long"
            onFile={(file) => {
              void dropSameFile(file);
            }}
          />
        </div>
        <div className={styles.cardActions}>
          <Button
            variant="secondary"
            onClick={() => {
              void cancelSource();
            }}
            data-testid="source-cancel"
          >
            {messages.cancel.button}
          </Button>
        </div>
      </section>
    );
  }

  return (
    <section className={styles.card} data-testid="source-progress" data-status={status}>
      <div className={transcribing ? long.stepDone : long.step}>
        <p className={long.stepTitle}>{copy.stage.audio}</p>
        <Meter percent={transcribing ? 100 : percentOf(audio)} label={copy.stage.audio} />
      </div>
      <div className={long.step}>
        <p className={long.stepTitle}>
          {transcribeLabel}
        </p>
        <Meter percent={percentOf(windows)} label={transcribeLabel} />
      </div>
      {working && left !== undefined ? <p className={long.hint}>{messages.export.timeLeft(left)}</p> : null}
      {working ? <p className={long.hint}>{copy.keepOpen}</p> : null}
      {status === "paused" && paused !== null ? (
        <p className={long.warn} role="alert" data-reason={paused.reason}>
          {copy.paused[paused.reason]}
        </p>
      ) : null}
      <div className={styles.cardActions}>
        {status === "paused" ? (
          <Button
            variant="primary"
            onClick={() => {
              if (paused?.reason === "signed_out" && !signedIn) {
                onSignIn();
                return;
              }
              void continueSource();
            }}
            data-testid="source-continue"
          >
            {copy.continue}
          </Button>
        ) : null}
        {working || status === "paused" ? (
          <Button
            variant="secondary"
            onClick={() => {
              void cancelSource();
            }}
            data-testid="source-cancel"
          >
            {messages.cancel.button}
          </Button>
        ) : null}
      </div>
    </section>
  );
}
