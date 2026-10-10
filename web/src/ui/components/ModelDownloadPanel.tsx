// The first-run download of the speech model (J4): what is being
// downloaded and why, how far it is, and what went wrong if it failed.
// It only reads the model store; nothing is started from here.

import { useEffect, useId, useRef } from "react";

import manifest from "../../config/model-manifest.json";
import { messages } from "../../copy/messages";
import type { ErrorCode } from "../../gen/domain";
import { useModelStore } from "../../state/model-store";
import styles from "../styles/components.module.css";

const BYTES_PER_MB = 1_000_000;
/** The size is said to the nearest 10 MB (TS §16.1). */
const SIZE_STEP_MB = 10;
const PERCENT = 100;

const SIZE_MB = Math.round(manifest.totalBytes / BYTES_PER_MB / SIZE_STEP_MB) * SIZE_STEP_MB;

/** The three failures of the download have words of their own (D-57). */
function failureCopy(code: ErrorCode | undefined) {
  if (code === "E_MODEL_DOWNLOAD" || code === "E_MODEL_HASH" || code === "E_MODEL_STORAGE") {
    return messages.errors[code];
  }
  return undefined;
}

export function ModelDownloadPanel() {
  const status = useModelStore((state) => state.status);
  const done = useModelStore((state) => state.done);
  const total = useModelStore((state) => state.total);
  const etaSecs = useModelStore((state) => state.etaSecs);
  const error = useModelStore((state) => state.error);
  const bodyId = useId();
  const bar = useRef<HTMLDivElement>(null);

  const percent = total > 0 ? Math.round(Math.min(done / total, 1) * PERCENT) : 0;
  // The width goes through a custom property: the CSP allows no style attribute.
  useEffect(() => {
    bar.current?.style.setProperty("--progress", `${String(percent)}%`);
  }, [percent]);

  const failure = status === "failed" ? failureCopy(error) : undefined;
  let line = "";
  if (status === "verifying") {
    line = messages.modelDownload.verifying;
  } else if (status === "downloading" && etaSecs !== null) {
    line = messages.modelDownload.progress(etaSecs);
  }

  return (
    <section className={styles.modelPanel}>
      <p id={bodyId} className={styles.notice}>
        {messages.modelDownload.body(SIZE_MB)}
      </p>
      <div
        className={styles.progressTrack}
        role="progressbar"
        aria-labelledby={bodyId}
        aria-valuemin={0}
        aria-valuemax={PERCENT}
        aria-valuenow={percent}
      >
        <div ref={bar} className={styles.progressBar} />
      </div>
      <p className={styles.status} role="status">
        {line}
      </p>
      {failure !== undefined && (
        <div className={styles.statusError} role="alert">
          <p className={styles.label}>{failure.title}</p>
          <p className={styles.notice}>{failure.body}</p>
          <p className={styles.notice}>{failure.action}</p>
        </div>
      )}
    </section>
  );
}
