// J10: how far the export is. The stage, a bar, the time it has taken, and
// the line that says where the work is done: on this computer.

import { useEffect, useId, useRef, useState } from "react";

import { messages } from "../../copy/messages";
import { useExportStore } from "../../state/export-store";
import type { ExportStatus } from "../../state/machines/export-machine";
import styles from "../styles/components.module.css";

const PERCENT = 100;
const MS_PER_SECOND = 1000;

type Running = keyof typeof messages.export.stage;

/** The stage an export is in while it runs, or `null`. */
function runningStage(status: ExportStatus): Running | null {
  return status === "rendering" || status === "muxing" || status === "saving" ? status : null;
}

/** Whole seconds since `startedAt`, counted while `running`. */
function useElapsedSeconds(startedAt: number | undefined, running: boolean): number {
  const [seconds, setSeconds] = useState(0);
  useEffect(() => {
    if (!running || startedAt === undefined) {
      return undefined;
    }
    const read = (): void => {
      setSeconds(Math.max(0, Math.floor((Date.now() - startedAt) / MS_PER_SECOND)));
    };
    read();
    const timer = setInterval(read, MS_PER_SECOND);
    return () => {
      clearInterval(timer);
    };
  }, [startedAt, running]);
  return seconds;
}

export function ExportProgress() {
  const status = useExportStore((state) => state.status);
  const done = useExportStore((state) => state.done);
  const total = useExportStore((state) => state.total);
  const startedAt = useExportStore((state) => state.startedAt);
  const stage = runningStage(status);
  const seconds = useElapsedSeconds(startedAt, stage !== null);
  const stageId = useId();
  const bar = useRef<HTMLDivElement>(null);

  const percent = total > 0 ? Math.round(Math.min(done / total, 1) * PERCENT) : 0;
  // The width goes through a custom property: the CSP allows no style attribute.
  useEffect(() => {
    bar.current?.style.setProperty("--progress", `${String(percent)}%`);
  }, [percent, stage]);

  if (status === "done") {
    return (
      <p className={styles.statusDone} role="status" data-testid="export-done">
        {messages.export.done}
      </p>
    );
  }
  if (status === "failed") {
    return (
      <p className={styles.statusError} role="alert" data-testid="export-failed">
        {messages.editor.failedStub}
      </p>
    );
  }
  if (stage === null) {
    return null;
  }

  return (
    <section className={styles.exportProgress} data-testid="export-progress">
      <p id={stageId} className={styles.label} data-testid="export-stage">
        {messages.export.stage[stage]}
      </p>
      <div
        className={styles.progressTrack}
        role="progressbar"
        aria-labelledby={stageId}
        aria-valuemin={0}
        aria-valuemax={PERCENT}
        aria-valuenow={percent}
      >
        <div ref={bar} className={styles.progressBar} />
      </div>
      <p className={styles.notice}>{messages.export.elapsed(seconds)}</p>
      <p className={styles.notice}>{messages.export.renderingLocally}</p>
    </section>
  );
}
