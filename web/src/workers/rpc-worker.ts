// The worker's side of the plumbing of `rpc.ts` (TS §14): `serveWorker`,
// the job context a handler gets, and what a handler throws or returns when
// it fails or is cancelled. Moved from `rpc.ts` (v4implementation D-150).

import type { ErrorCode, JobId } from "../gen/domain";
import type { Cancel, FailureStage, Progress, Req, Res } from "./protocol";
import { toAppFailure } from "./rpc";

export type JobContext = {
  jobId: JobId;
  isCancelled(): boolean;
  progress(p: Omit<Progress, "type" | "jobId">): void;
};

export type Handlers<Api> = { [M in keyof Api]: (params: never, ctx: JobContext) => Promise<unknown> | void };

/**
 * What code in a worker throws for a failure it can name: the code, a word
 * of detail for development builds, and its own stage when that is not the
 * worker's default (D-32).
 */
export class WorkerFailure extends Error {
  readonly code: ErrorCode;
  readonly detail: string;
  readonly stage?: FailureStage;

  constructor(code: ErrorCode, detail: string, stage?: FailureStage) {
    super(code);
    this.name = "WorkerFailure";
    this.code = code;
    this.detail = detail;
    if (stage !== undefined) {
      this.stage = stage;
    }
  }
}

/** A handler returns this after it has seen its cancel flag set. */
export const CANCELLED: unique symbol = Symbol("cancelled");


// What a worker's global scope offers this file. `self` is typed as a
// window, because the app is compiled with both libraries.
type WorkerScope = {
  postMessage(message: unknown, options?: { transfer?: Transferable[] }): void;
  addEventListener(type: "message", listener: (event: MessageEvent<Req<string, unknown> | Cancel>) => void): void;
};

type Job = { jobId: JobId; method: string; cancelled: boolean };

/** The buffers of a result that must move, not be copied (TS §14.3). */
function transferablesOf(result: unknown): Transferable[] {
  const fields: unknown[] = typeof result === "object" && result !== null ? Object.values(result) : [];
  return [result, ...fields].flatMap((value): Transferable[] => {
    if (value instanceof Float32Array && value.buffer instanceof ArrayBuffer) {
      return [value.buffer];
    }
    return value instanceof OffscreenCanvas || value instanceof ImageBitmap ? [value] : [];
  });
}

/**
 * Serves the methods of one worker. One job at a time: a second request
 * while one is pending is refused, unless it is listed in `duringPreview`
 * and the pending job is `previewPlay`. A method in `oneWay` is run and
 * never answered.
 */
export function serveWorker<Api>(
  handlers: Handlers<Api>,
  o: { stage: FailureStage; oneWay: readonly string[]; duringPreview: readonly string[] },
): void {
  const scope: WorkerScope = self;
  const table: Partial<Record<string, (params: never, ctx: JobContext) => Promise<unknown> | void>> = handlers;
  const running = new Map<JobId, Job>();
  let main: Job | undefined;

  const post = (message: Res<unknown> | Progress, transfer: Transferable[] = []): void => {
    scope.postMessage(message, { transfer });
  };
  const refuse = (jobId: JobId, detail: string): void => {
    post({ type: "res", jobId, ok: false, failure: { code: "E_INTERNAL", stage: o.stage, retryable: false, detail } });
  };

  scope.addEventListener("message", (event) => {
    const message = event.data;
    if (message.type === "cancel") {
      const job = running.get(message.jobId);
      if (job !== undefined) {
        job.cancelled = true;
      }
      return;
    }

    const { jobId, method, params } = message;
    const oneWay = o.oneWay.includes(method);
    const handler = table[method];
    const beside = main?.method === "previewPlay" && o.duringPreview.includes(method);
    if (handler === undefined || (main !== undefined && !beside)) {
      if (!oneWay) {
        refuse(jobId, handler === undefined ? "NoHandler" : "Busy");
      }
      return;
    }

    const job: Job = { jobId, method, cancelled: false };
    running.set(jobId, job);
    // A one-way message, or a call beside the preview, does not occupy the worker.
    const occupies = !oneWay && !beside;
    if (occupies) {
      main = job;
    }
    const ctx: JobContext = {
      jobId,
      isCancelled: () => job.cancelled,
      progress: (p) => {
        post({ type: "progress", jobId, ...p });
      },
    };
    const done = (): void => {
      running.delete(jobId);
      if (occupies) {
        main = undefined;
      }
    };

    void Promise.resolve()
      .then(() => handler(params as never, ctx))
      .then(
        (result) => {
          done();
          if (oneWay) {
            return;
          }
          if (result !== CANCELLED) {
            post({ type: "res", jobId, ok: true, result }, transferablesOf(result));
          } else if (job.cancelled) {
            post({ type: "res", jobId, cancelled: true });
          } else {
            refuse(jobId, "CancelledUnasked");
          }
        },
        (thrown: unknown) => {
          done();
          if (oneWay) {
            // No answer can carry it. An uncaught error reaches the client
            // as a crash of this worker, which is what it is.
            reportError(thrown);
          } else {
            post({ type: "res", jobId, ok: false, failure: toAppFailure(thrown, o.stage) });
          }
        },
      );
  });
}
