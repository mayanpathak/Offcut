// C-12 (v4implementation D-152): stopping a long job. This file owns one
// abort controller per scope; the use-case of a scope reads its signal when
// the job starts and ends in the scope's state when it sees it aborted:
//
//   import   → the clip `idle`, its directory removed (import-clip.ts)
//   pipeline → the clip `accepted`, with "Process again" (run-pipeline.ts)
//   model    → the download's bytes kept for the next run, the clip as for the pipeline
//   export   → the export `idle`, no file under `exports/tmp/` (start-export.ts)
//
// `cancelJob` aborts the signal, cancels the scope's workers (a job that
// does not answer within 2 s has its worker restarted, workers/pool.ts),
// waits for the use-case to reach the scope's state and sends
// `job_cancelled { stage }`. It also keeps the one other record of
// recovery (C-14, D-153): which failures were shown, so that each sends
// `client_error` once.

import { track } from "../analytics/client";
import type { FailureStage } from "../gen/domain";
import { type ClipState, useClipStore } from "../state/clip-store";
import { type ExportState, useExportStore } from "../state/export-store";
import { type ModelState, useModelStore } from "../state/model-store";
import { pool } from "../workers/pool";
import type { AppFailure, WorkerKind } from "../workers/protocol";

export type CancelScope = "import" | "pipeline" | "model" | "export";

/** The workers that may hold a scope's job. The pipeline's may be any of them, and the model is waited for by a run. */
const WORKERS: Record<CancelScope, readonly WorkerKind[]> = {
  import: ["media"],
  pipeline: ["media", "asr", "audio", "render"],
  model: ["media", "asr", "audio", "render"],
  export: ["render"],
};

const controllers = new Map<CancelScope, AbortController>();
const cancelling = new Map<CancelScope, Promise<void>>();

/**
 * The signal of `scope`'s job. A job reads it once, when it starts, and
 * keeps it: a signal that was aborted is replaced for the next job.
 */
export function signalFor(scope: CancelScope): AbortSignal {
  let controller = controllers.get(scope);
  if (controller === undefined || controller.signal.aborted) {
    controller = new AbortController();
    controllers.set(scope, controller);
  }
  return controller.signal;
}

/** The stage a cancel of `scope` stops now, or `null` when no job of the scope runs. */
function runningStage(scope: CancelScope): FailureStage | null {
  const clip = useClipStore.getState();
  switch (scope) {
    case "import":
      return clip.status === "importing" ? "import" : null;
    case "pipeline":
      return clip.status === "processing" ? (clip.stage ?? "probe_audio") : null;
    case "model": {
      const { status } = useModelStore.getState();
      return status === "downloading" || status === "verifying" ? "model" : null;
    }
    case "export": {
      const { status } = useExportStore.getState();
      return status === "rendering" ? "render_encode" : status === "muxing" ? "mux" : null;
    }
    default:
      return assertNever(scope);
  }
}

function assertNever(value: never): never {
  void value;
  throw new Error("a scope of cancel this build does not know");
}

type Store<S> = { getState(): S; subscribe(listener: (state: S) => void): () => void };

/** Resolves when `store` holds a state `over` accepts. */
function until<S>(store: Store<S>, over: (state: S) => boolean): Promise<S> {
  return new Promise((resolve) => {
    if (over(store.getState())) {
      resolve(store.getState());
      return;
    }
    const stop = store.subscribe((state) => {
      if (over(state)) {
        stop();
        resolve(state);
      }
    });
  });
}

const clipOver = (busy: ClipState["status"]) => (state: ClipState) => state.status !== busy;
const exportOver = (state: ExportState) => !["gating", "rendering", "muxing", "cancelled"].includes(state.status);
const modelOver = (state: ModelState) => state.status !== "downloading";

/**
 * Waits for the use-case to end the job, and says whether it ended as
 * cancelled: a job that finished, or failed, before the cancel reached it
 * keeps its own end, and no `job_cancelled` is sent for it.
 */
async function ended(scope: CancelScope): Promise<boolean> {
  switch (scope) {
    case "import":
      // A cancelled import passes through `failed` on its way to `idle` in one moment (clip-store.ts): read after it.
      await until(useClipStore, clipOver("importing"));
      return useClipStore.getState().status === "idle";
    case "pipeline":
      return (await until(useClipStore, clipOver("processing"))).cancelled;
    case "model": {
      const [model, clip] = await Promise.all([
        until(useModelStore, modelOver),
        until(useClipStore, clipOver("processing")),
      ]);
      return model.status === "partial" || clip.cancelled;
    }
    case "export":
      return (await until(useExportStore, exportOver)).status === "idle";
    default:
      return assertNever(scope);
  }
}

async function stop(scope: CancelScope): Promise<void> {
  const stage = runningStage(scope);
  if (stage === null) {
    return;
  }
  controllers.get(scope)?.abort();
  // A run waits for the model, and the model is downloaded for a run: the one is not stopped without the other.
  const clip = useClipStore.getState();
  if (scope === "model" || (scope === "pipeline" && clip.waitingModel)) {
    controllers.get(scope === "model" ? "pipeline" : "model")?.abort();
  }
  await Promise.all(WORKERS[scope].map((kind) => pool.cancel(kind)));
  if (await ended(scope)) {
    track({ name: "job_cancelled", props: { stage } });
  }
}

/**
 * Cancels the job of `scope` (D-152) and resolves when it has ended in the
 * scope's state. Nothing happens when no job of the scope runs. A second
 * call while the first runs gets the same promise.
 */
export function cancelJob(scope: CancelScope): Promise<void> {
  let running = cancelling.get(scope);
  if (running === undefined) {
    running = stop(scope).finally(() => {
      cancelling.delete(scope);
    });
    cancelling.set(scope, running);
  }
  return running;
}

/** The failures shown so far, by identity: a failure is one object from where it was made to where it is shown. */
const shown = new WeakSet<AppFailure>();

/**
 * A failure is on the screen (D-153): `client_error { error_code, stage }`
 * is sent the first time, never again for the same failure, however often
 * the card that shows it is drawn.
 */
export function failureShown(failure: AppFailure): void {
  if (shown.has(failure)) {
    return;
  }
  shown.add(failure);
  track({ name: "client_error", props: { error_code: failure.code, stage: failure.stage } });
}
