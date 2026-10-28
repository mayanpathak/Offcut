// The plumbing between the main thread and a worker: a request, its one
// response, progress in between, and a cooperative cancel flag (TS §14).
// A worker that fails (`error`, `messageerror`) is dead: every pending call
// is rejected with `E_WORKER_CRASH` and the crash is told to whoever asked
// (v4implementation D-153); the pool makes a new worker for the next call.
// `createClient` is the main thread's side; the worker's side is in
// `rpc-worker.ts` (v4implementation D-150), so that the page imports no
// worker code. `toAppFailure` is used by both sides and stays here.

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

// A method the protocol gives no argument is called with none.
export type Client<Api> = {
  [M in keyof Api]: Api[M] extends () => Promise<infer R>
    ? (p?: undefined, o?: CallOptions) => Promise<R | Cancelled>
    : Api[M] extends (p: infer P) => Promise<infer R>
      ? (p: P, o?: CallOptions) => Promise<R | Cancelled>
      : never;
};

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
  /** Told how the call ended: `true` when it answered as cancelled. */
  ended: ((cancelled: boolean) => void)[];
};

/** The main thread's side of one worker (v4implementation §14.2). */
export type WorkerClient<Api> = {
  call: Client<Api>;
  notify(method: string, params: unknown): void;
  /** Asks the worker to stop job `jobId` (its cancel flag); the job answers when it sees it. */
  cancel(jobId: JobId): void;
  /** `cb` is told the failure when the worker crashes, once. Not when it is disposed. */
  onCrash(cb: (failure: AppFailure) => void): void;
  /** The worker crashed or was disposed: it answers nothing more. */
  isDead(): boolean;
  /** The calls waiting for an answer, oldest first. */
  pendingJobs(): JobId[];
  /** Resolves when job `jobId` has ended, with `true` when it answered as cancelled. At once (`false`) for a job not pending. */
  ended(jobId: JobId): Promise<boolean>;
  /** Ends the worker; every pending call is rejected with `E_WORKER_CRASH`. */
  dispose(): void;
};

export function createClient<Api>(worker: Worker, stage: FailureStage): WorkerClient<Api> {
  const pending = new Map<JobId, PendingCall>();
  const crashListeners: ((failure: AppFailure) => void)[] = [];
  let dead = false;
  let lastJob = 0;
  const nextJob = (): JobId => {
    lastJob += 1;
    return lastJob as JobId;
  };
  const crashFailure = (): AppFailure => ({ code: CRASH, stage, retryable: RETRYABLE[CRASH] });

  /** Ends a pending call: it leaves the table first, the single record of what waits. */
  const settle = (jobId: JobId, how: (call: PendingCall) => void, cancelled = false): void => {
    const call = pending.get(jobId);
    if (call === undefined) {
      return;
    }
    pending.delete(jobId);
    how(call);
    for (const tell of call.ended) {
      tell(cancelled);
    }
  };

  /** Nothing that is waiting will ever be answered: each call is rejected, and the table is empty after. */
  const rejectAll = (): void => {
    for (const jobId of [...pending.keys()]) {
      settle(jobId, (call) => {
        call.reject(new WorkerCallError(crashFailure()));
      });
    }
  };

  // The worker is gone, or sent something that cannot be read (TS C-14, D-153).
  const crash = (): void => {
    if (dead) {
      return;
    }
    dead = true;
    rejectAll();
    for (const cb of crashListeners) {
      cb(crashFailure());
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
    if ("cancelled" in message) {
      settle(message.jobId, (c) => c.resolve({ cancelled: true } satisfies Cancelled), true);
    } else if (message.ok) {
      const { result } = message;
      settle(message.jobId, (c) => c.resolve(result));
    } else {
      const { failure } = message;
      settle(message.jobId, (c) => c.reject(new WorkerCallError(failure)));
    }
  });

  const invoke = (method: string, params: unknown, o: CallOptions = {}): Promise<unknown> =>
    new Promise((resolve, reject) => {
      // A dead worker answers nothing: the call fails now, not never.
      if (dead) {
        reject(new WorkerCallError(crashFailure()));
        return;
      }
      const jobId = nextJob();
      const transfer = o.transfer ?? [];
      pending.set(jobId, { resolve, reject, onProgress: o.onProgress, ended: [] });
      const refuse = (thrown: unknown): void => {
        settle(jobId, (call) => {
          call.reject(new WorkerCallError(toAppFailure(thrown, stage)));
        });
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
      if (!dead) {
        worker.postMessage({ type: "req", jobId: nextJob(), method, params } satisfies Req<string, unknown>);
      }
    },
    cancel(jobId) {
      if (!dead && pending.has(jobId)) {
        worker.postMessage({ type: "cancel", jobId } satisfies Cancel);
      }
    },
    onCrash(cb) {
      crashListeners.push(cb);
    },
    isDead: () => dead,
    pendingJobs: () => [...pending.keys()],
    ended(jobId) {
      const call = pending.get(jobId);
      if (call === undefined) {
        return Promise.resolve(false);
      }
      return new Promise((resolve) => {
        call.ended.push(resolve);
      });
    },
    dispose() {
      // Terminated, it can send nothing late: what waits is rejected here.
      worker.terminate();
      dead = true;
      rejectAll();
    },
  };
}
