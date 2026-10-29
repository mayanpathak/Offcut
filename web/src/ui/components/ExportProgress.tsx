// J10: how far the export is, as the pill the Export button turns into
// (v4implementation D-189): the stage in words, the percent, a bar, the time
// left once 3 s of frames give a rate, and Cancel. The line that says where
// the work is done is the bar's description. When the file is downloaded, a
// success state offers "Download again"; a failure is the export's error
// card in the side panel (D-190).

import { useEffect, useId, useRef, useState } from "react";

import { messages } from "../../copy/messages";
import { useExportStore } from "../../state/export-store";
import type { ExportStatus } from "../../state/machines/export-machine";
import { cancelJob } from "../../usecases/cancel-job";
import { downloadAgain } from "../../usecases/start-export";
import { Check } from "../icons";
import { Button } from "../primitives/Button";
import styles from "../styles/components.module.css";
import { ErrorPanel } from "./ErrorPanel";

const PERCENT = 100;
const MS_PER_SECOND = 1000;
/** Frames are counted this long before a time left is said (D-189). */
const RATE_AFTER_MS = 3000;
/** The time left is said in steps of this many seconds, as the model download's is. */
const LEFT_STEP_S = 5;
/** A higher time left is said only once it is this share above the one shown. */
const LEFT_RISE = 0.25;

type Stage = "rendering" | "muxing";

/** Whether an export runs: the pill stands in for the Export button. */
export function exportRunning(status: ExportStatus): boolean {
  return status === "gating" || status === "rendering" || status === "muxing" || status === "saving";
}

/** The stage in words: the frames are drawn, then the file is finished ("Saving" is part of finishing it). */
function stageOf(status: ExportStatus): Stage {
  return status === "muxing" || status === "saving" ? "muxing" : "rendering";
}

/**
 * The seconds left, once the frames encoded over RATE_AFTER_MS give a rate;
 * `null` before, and once the frames are all drawn. Counted from the first
 * frame, not from the start: the setup before it says nothing of the rate.
 * Said in steps, falling freely and rising only when the export has really
 * slowed, as the model download's time left is (design review 1).
 */
function useTimeLeft(done: number, total: number, rendering: boolean): number | null {
  const first = useRef<{ at: number; done: number } | undefined>(undefined);
  const [shown, setShown] = useState<number | null>(null);
  useEffect(() => {
    if (!rendering || total === 0 || done >= total) {
      first.current = undefined;
      setShown(null);
      return;
    }
    const now = performance.now();
    first.current ??= { at: now, done };
    const start = first.current;
    if (now - start.at < RATE_AFTER_MS || done <= start.done) {
      return;
    }
    const perSecond = ((done - start.done) * MS_PER_SECOND) / (now - start.at);
    const stepped = Math.ceil((total - done) / perSecond / LEFT_STEP_S) * LEFT_STEP_S;
    setShown((last) => (last === null || stepped < last || stepped > last * (1 + LEFT_RISE) ? stepped : last));
  }, [done, total, rendering]);
  return shown;
}

/** The pill (D-189), while an export runs. */
export function ExportProgress() {
  const status = useExportStore((state) => state.status);
  const done = useExportStore((state) => state.done);
  const total = useExportStore((state) => state.total);
  const stage = stageOf(status);
  const left = useTimeLeft(done, total, status === "rendering");
  const [cancelling, setCancelling] = useState(false);
  const stageId = useId();
  const localId = useId();
  const bar = useRef<HTMLDivElement>(null);

  const percent = total > 0 ? Math.round(Math.min(done / total, 1) * PERCENT) : 0;
  // The width goes through a custom property: the CSP allows no style attribute.
  useEffect(() => {
    bar.current?.style.setProperty("--progress", `${String(percent)}%`);
  }, [percent]);

  return (
    <section className={styles.pill} aria-label={messages.export.label} data-testid="export-progress">
      <div className={styles.pillBody}>
        <p className={styles.pillLine}>
          <span id={stageId} className={styles.pillStage} data-testid="export-stage">
            {messages.export.stage[stage]}
          </span>
          <span className={styles.pillPercent}>{messages.export.percent(percent)}</span>
          {left === null ? null : <span className={styles.pillLeft}>{messages.export.timeLeft(left)}</span>}
        </p>
        <div
          className={styles.meter}
          role="progressbar"
          aria-labelledby={stageId}
          aria-describedby={localId}
          aria-valuemin={0}
          aria-valuemax={PERCENT}
          aria-valuenow={percent}
        >
          <div ref={bar} className={styles.meterBar} />
        </div>
        <span id={localId} className={styles.srOnly}>
          {messages.export.renderingLocally}
        </span>
      </div>
      <Button
        variant="ghost"
        size="sm"
        busy={cancelling}
        // A file being saved is whole: it is no longer cancelled (TS §12.2).
        disabled={cancelling || status === "saving" || status === "gating"}
        onClick={() => {
          setCancelling(true);
          void cancelJob("export").finally(() => {
            setCancelling(false);
          });
        }}
        data-testid="export-cancel"
      >
        {messages.cancel.button}
      </Button>
    </section>
  );
}

/** The success state (D-189), beside the Export button once the file is downloaded. */
export function ExportDone() {
  return (
    <span className={styles.exportDone} role="status">
      <Check size={16} />
      <span data-testid="export-done">{messages.export.done}</span>
      <Button
        variant="ghost"
        size="sm"
        onClick={() => {
          void downloadAgain();
        }}
        data-testid="download-again"
      >
        {messages.export.downloadAgain}
      </Button>
    </span>
  );
}

/** A failed export's card, at the top of the side panel (D-190). */
export function ExportFailure() {
  const status = useExportStore((state) => state.status);
  const failure = useExportStore((state) => state.failure);
  if (status !== "failed") {
    return null;
  }
  return <ErrorPanel failure={failure} context="export" testId="export-failed" compact />;
}
