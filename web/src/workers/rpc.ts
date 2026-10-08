// The plumbing between the main thread and a worker: a request, its one
// response, progress in between, and a cooperative cancel flag (TS §14).
// `createClient` is the main thread's side, `serveWorker` the worker's.

import { ERROR_CODES, type ErrorCode, type JobId } from "../gen/domain";
import type { AppFailure, Cancel, FailureStage, Progress, Req, Res } from "./protocol";

/** What a failed call rejects with. */
export class WorkerCallError extends Error {
  readonly failure: AppFailure;

  constructor(failure: AppFailure) {
    super(failure.code);
    this.name = "WorkerCallError";
    this.failure = failure;
  }
}

/** What a cancelled call resolves with: cancelling is not an error (TS §11.3). */
export type Cancelled = { cancelled: true };

export type CallOptions = {
  transfer?: Transferable[];
  onProgress?: (p: Progress) => void;
  onJob?: (jobId: JobId) => void;
};

export type Client<Api> = {
  [M in keyof Api]: Api[M] extends (p: infer P) => Promise<infer R>
    ? (p: P, o?: CallOptions) => Promise<R | Cancelled>
    : never;
};

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

// --- Failures ----------------------------------------------------------------

// The "Retryable" column of TS §11.2. This is its only copy.
const RETRYABLE: Record<ErrorCode, boolean> = {
  E_MODEL_DOWNLOAD: true,
  E_MODEL_HASH: true,
  E_MODEL_STORAGE: true,
  E_STORAGE_QUOTA: true,
  E_STORAGE_IO: true,
  E_DECODE_AUDIO: true,
  E_DECODE_VIDEO: true,
  E_ASR_RUNTIME: true,
  E_ASR_OOM: true,
  E_DSP: false,
  E_GPU_INIT: true,
  E_GPU_LOST: true,
  E_ENCODE_VIDEO: true,
  E_ENCODE_AUDIO: true,
  E_MUX: false,
  E_WORKER_CRASH: true,
  E_NET_OFFLINE: true,
  E_NET_TIMEOUT: true,
  E_API_5XX: true,
  E_API_RATE_LIMITED: true,
  E_AUTH_LINK_INVALID: false,
  E_AUTH_LINK_EXPIRED: false,
  E_AUTH_SESSION_EXPIRED: false,
  E_AUTH_EMAIL_UNAVAILABLE: true,
  E_BILLING_UNAVAILABLE: true,
  E_BILLING_PENDING: true,
  E_ENTITLEMENT_INVALID: false,
  E_ENTITLEMENT_EXPIRED: true,
  E_INTERNAL: false,
};

// Every stage a failure can name. A record, so that a stage added to the
// generated type is missed here at compile time.
const STAGES: Record<FailureStage, true> = {
  probe_audio: true,
  asr: true,
  audio_chain: true,
  detect_scene: true,
  render_encode: true,
  mux: true,
  import: true,
  model: true,
  preview: true,
  storage: true,
  api: true,
};

// Every panic in a WASM bundle throws an error that starts with this (V1 §8).
const CRASH = "E_WORKER_CRASH";

function isErrorCode(value: unknown): value is ErrorCode {
  const codes: readonly unknown[] = ERROR_CODES;
  return codes.includes(value);
}

function isStage(value: unknown): value is FailureStage {
  return typeof value === "string" && Object.hasOwn(STAGES, value);
}

/**
 * Whatever was thrown, as an `AppFailure`. A thrown `{ code, detail }` keeps
 * its code, and its own stage when it has one; `stage` is the default of the
 * worker it was thrown in. `detail` is for development builds only.
 */
export function toAppFailure(thrown: unknown, stage: FailureStage): AppFailure {
  if (typeof thrown === "object" && thrown !== null && "code" in thrown && isErrorCode(thrown.code)) {
    const failure: AppFailure = {
      code: thrown.code,
      stage: "stage" in thrown && isStage(thrown.stage) ? thrown.stage : stage,
      retryable: RETRYABLE[thrown.code],
    };
    if ("detail" in thrown && typeof thrown.detail === "string") {
      failure.detail = thrown.detail;
    }
    return failure;
  }
  const crashed =
    (thrown instanceof Error && thrown.message.startsWith(CRASH)) ||
    (typeof thrown === "string" && thrown.startsWith(CRASH));
  const code = crashed ? CRASH : "E_INTERNAL";
  return { code, stage, retryable: RETRYABLE[code], detail: thrown instanceof Error ? thrown.name : typeof thrown };
}

// --- The main thread's side --------------------------------------------------

type PendingCall = {
  resolve: (result: unknown) => void;
  reject: (error: WorkerCallError) => void;
  onProgress: ((p: Progress) => void) | undefined;
};

export function createClient<Api>(
  worker: Worker,
  stage: FailureStage,
): {
  call: Client<Api>;
  notify(method: string, params: unknown): void;
  cancel(jobId: JobId): void;
  dispose(): void;
} {
  const pending = new Map<JobId, PendingCall>();
  let lastJob = 0;
  const nextJob = (): JobId => {
    lastJob += 1;
    return lastJob as JobId;
  };

  // The worker is gone, or sent something that cannot be read: nothing that
  // is waiting will ever be answered (TS C-14).
  const crash = (): void => {
    const calls = [...pending.values()];
    pending.clear();
    for (const call of calls) {
      call.reject(new WorkerCallError({ code: CRASH, stage, retryable: RETRYABLE[CRASH] }));
    }
  };
  worker.addEventListener("error", crash);
  worker.addEventListener("messageerror", crash);

  worker.addEventListener("message", (event: MessageEvent<Res<unknown> | Progress>) => {
    const message = event.data;
    const call = pending.get(message.jobId);
    // A message for a job that is unknown or finished is dropped (TS §14.4).
    if (call === undefined) {
      return;
    }
    if (message.type === "progress") {
      call.onProgress?.(message);
      return;
    }
    pending.delete(message.jobId);
    if ("cancelled" in message) {
      call.resolve({ cancelled: true } satisfies Cancelled);
    } else if (message.ok) {
      call.resolve(message.result);
    } else {
      call.reject(new WorkerCallError(message.failure));
    }
  });

  const invoke = (method: string, params: unknown, o: CallOptions = {}): Promise<unknown> =>
    new Promise((resolve, reject) => {
      const jobId = nextJob();
      const transfer = o.transfer ?? [];
      pending.set(jobId, { resolve, reject, onProgress: o.onProgress });
      const refuse = (thrown: unknown): void => {
        pending.delete(jobId);
        reject(new WorkerCallError(toAppFailure(thrown, stage)));
      };
      try {
        worker.postMessage({ type: "req", jobId, method, params } satisfies Req<string, unknown>, transfer);
      } catch (thrown) {
        refuse(thrown);
        return;
      }
      o.onJob?.(jobId);
      // A buffer in the transfer list must be gone from this thread now
      // (TS §14.3). Checked in every build: a copy of 17 MB of audio is a
      // bug worth a failed call, and the check costs nothing.
      if (transfer.some((item) => item instanceof ArrayBuffer && item.byteLength > 0)) {
        refuse(new Error("a buffer in the transfer list was copied, not transferred"));
      }
    });

  // The methods of `Api` are not known when this runs, so every name is one.
  const call = new Proxy(
    {},
    {
      get: (_target, method) =>
        typeof method !== "string" || method === "then"
          ? undefined
          : (params: unknown, o?: CallOptions) => invoke(method, params, o),
    },
  ) as Client<Api>;

  return {
    call,
    notify(method, params) {
      worker.postMessage({ type: "req", jobId: nextJob(), method, params } satisfies Req<string, unknown>);
    },
    cancel(jobId) {
      worker.postMessage({ type: "cancel", jobId } satisfies Cancel);
    },
    dispose() {
      worker.terminate();
      crash();
    },
  };
}

// --- The worker's side -------------------------------------------------------

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
    return value instanceof OffscreenCanvas ? [value] : [];
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
