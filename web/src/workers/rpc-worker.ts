// The worker's side of the plumbing of `rpc.ts` (TS §14): `serveWorker`,
// the job context a handler gets, and what a handler throws or returns when
// it fails or is cancelled. Moved from `rpc.ts` (v4implementation D-150).
// Progress is throttled here (D-151): at most 10 messages a second per job,
// and never a feed line dropped. A development build's worker also takes the
// three faults of the page's development hooks (D-155, G 14.5); a production
// build has none of them (`__OFFCUT_DEV_FAULTS__`, vite.config.ts).

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

/** A progress without a feed line is sent at most this often per job (D-151, TS §14.3). */
export const PROGRESS_MIN_INTERVAL_MS = 100;

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
 * The progress of one job, throttled (D-151). A progress with a feed line is
 * sent at once; one without, inside `PROGRESS_MIN_INTERVAL_MS` of the last
 * one sent, is held, replaced by the next, and sent when the interval is
 * over, by the one timer of the job. `flush` sends what is held before the
 * job's answer; after it the job sends no more progress.
 */
function throttledProgress(jobId: JobId, send: (p: Progress) => void): {
  progress: JobContext["progress"];
  flush(): void;
} {
  let lastSent: number | undefined;
  let held: Progress | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let over = false;
  const sendNow = (p: Progress): void => {
    clearTimeout(timer);
    timer = undefined;
    held = undefined;
    lastSent = performance.now();
    send(p);
  };
  return {
    progress: (p) => {
      if (over) {
        return;
      }
      const message: Progress = { type: "progress", jobId, ...p };
      const now = performance.now();
      // A feed line is never held: the newer done and total it carries replace a held progress.
      if (p.feed !== undefined || lastSent === undefined || now - lastSent >= PROGRESS_MIN_INTERVAL_MS) {
        sendNow(message);
        return;
      }
      held = message;
      timer ??= setTimeout(() => {
        timer = undefined;
        if (held !== undefined) {
          sendNow(held);
        }
      }, lastSent + PROGRESS_MIN_INTERVAL_MS - now);
    },
    flush: () => {
      over = true;
      if (held !== undefined) {
        sendNow(held);
      }
      clearTimeout(timer);
      timer = undefined;
    },
  };
}


// Set by vite.config.ts from the build's mode, as `DEV_HOOKS` of config/env.ts
// is; absent where no bundler ran (the e2e suites import worker files in Node).
declare const __OFFCUT_DEV_FAULTS__: boolean | undefined;
/** The faults exist in development builds only: a production build folds this to `false` and drops them. */
const DEV_FAULTS = typeof __OFFCUT_DEV_FAULTS__ !== "undefined" && __OFFCUT_DEV_FAULTS__;

/** Blocks the worker's thread for good: only a restart ends it (`pool.restartWorker`). */
function hang(): never {
  for (;;) {
    // Nothing: the thread never takes the next message.
  }
}

/**
 * The development faults, one-way messages of any worker (`ui/dev-hooks.ts`):
 * `devCrash` throws where nothing catches it, which the page sees as the
 * worker's crash; `devHang` blocks the thread; `devFailNext` makes the next
 * call of `params.method` fail. `true` when `method` was one of them.
 */
function devFault(method: string, params: unknown, failNext: Set<string>): boolean {
  switch (method) {
    case "devCrash":
      setTimeout(() => {
        throw new Error("devCrash");
      }, 0);
      return true;
    case "devHang":
      return hang();
    case "devFailNext": {
      const target = typeof params === "object" && params !== null && "method" in params ? params.method : undefined;
      if (typeof target === "string") {
        failNext.add(target);
      }
      return true;
    }
    default:
      return false;
  }
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
  const failNext = new Set<string>();

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
    if (DEV_FAULTS && devFault(method, params, failNext)) {
      return;
    }
    if (DEV_FAULTS && failNext.delete(method)) {
      refuse(jobId, "DevFailure");
      return;
    }
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
    const throttle = throttledProgress(jobId, (p) => {
      post(p);
    });
    const ctx: JobContext = { jobId, isCancelled: () => job.cancelled, progress: throttle.progress };
    // The job is over: its held progress goes first, then its answer.
    const done = (): void => {
      throttle.flush();
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
