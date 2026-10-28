// The worker pool: one typed client per worker. A worker is created on the
// first call to its client and kept until it crashes or is restarted; the
// next call then creates a fresh one (TS §14.3, v4implementation D-152,
// D-153). `pool.cancel(kind)` stops a worker's pending job, and restarts the
// worker when the job does not answer within `CANCEL_TIMEOUT_MS`. At app
// start `preload()` fetches what the workers will need (D-9), which also
// proves in production that the build, the MIME type and the CSP are right.
//
// Four workers: media, ASR, audio (V3) and render. At most four exist (TS §31):
// a dead worker is terminated before its replacement is made.

import { preloadCore } from "../wasm/load-core";
import { preloadRender } from "../wasm/load-render";
import asrWorkerUrl from "./asr.worker.ts?worker&url";
import audioWorkerUrl from "./audio.worker.ts?worker&url";
import mediaWorkerUrl from "./media.worker.ts?worker&url";
import type {
  AppFailure,
  AsrWorkerApi,
  AudioWorkerApi,
  FailureStage,
  MediaWorkerApi,
  RenderWorkerApi,
  WorkerKind,
} from "./protocol";
import renderWorkerUrl from "./render.worker.ts?worker&url";
import { type Client, createClient, WorkerCallError, type WorkerClient } from "./rpc";

// A use-case may import this file only, of all the workers' files (D-32).
export { type Cancelled, WorkerCallError } from "./rpc";

/** How long a cancelled job has to answer before its worker is restarted (D-152; assumption, M1.5). */
export const CANCEL_TIMEOUT_MS = 2000;

type PoolClient<Api> = Client<Api> & {
  notify(method: string, params: unknown): void;
  /** `cb` is told the failure when this worker crashes, whichever worker the pool has made by then. */
  onCrash(cb: (failure: AppFailure) => void): void;
  /** The worker crashed: the next call makes a new one. `false` before the first call. */
  isDead(): boolean;
};

/** What the pool keeps of one worker: its client, made on demand, and how to end it. */
type Row<Api> = {
  client: PoolClient<Api>;
  /** The client of the worker that exists now, if one does. */
  current(): WorkerClient<Api> | undefined;
  /** Terminates the worker, if one exists; its pending calls are rejected with `E_WORKER_CRASH`. */
  restart(): void;
};

// One row per worker: its script, and the stage a failure in it is tagged
// with unless the failure names its own. The lists of one-way methods and of
// methods accepted beside a running preview are given to `serveWorker` by
// each worker's own entry file.
const WORKERS = {
  media: { url: mediaWorkerUrl, stage: "import" },
  asr: { url: asrWorkerUrl, stage: "asr" },
  audio: { url: audioWorkerUrl, stage: "audio_chain" },
  render: { url: renderWorkerUrl, stage: "preview" },
} as const satisfies Record<string, { url: string; stage: FailureStage }>;

/** A client whose worker is created by its first call, and again by the first call after it died. */
function lazyRow<Api>(row: { url: string; stage: FailureStage }): Row<Api> {
  let client: WorkerClient<Api> | undefined;
  const crashListeners: ((failure: AppFailure) => void)[] = [];
  const started = (): WorkerClient<Api> => {
    if (client?.isDead() === true) {
      // A crashed worker may still run: it is ended before another is made.
      client.dispose();
      client = undefined;
    }
    if (client === undefined) {
      const made = createClient<Api>(new Worker(row.url, { type: "module" }), row.stage);
      made.onCrash((failure) => {
        for (const cb of crashListeners) {
          cb(failure);
        }
      });
      client = made;
    }
    return client;
  };
  // The methods of `Api` are not known when this runs, so every name is one.
  const proxy = new Proxy(
    {},
    {
      get: (_target, name) => {
        if (typeof name !== "string" || name === "then") {
          return undefined;
        }
        if (name === "notify") {
          return (method: string, params: unknown) => {
            started().notify(method, params);
          };
        }
        if (name === "onCrash") {
          return (cb: (failure: AppFailure) => void) => {
            crashListeners.push(cb);
          };
        }
        if (name === "isDead") {
          return () => client?.isDead() ?? false;
        }
        return Reflect.get(started().call, name) as unknown;
      },
    },
  ) as PoolClient<Api>;
  return {
    client: proxy,
    current: () => client,
    restart: () => {
      client?.dispose();
      client = undefined;
    },
  };
}

const rows = {
  media: lazyRow<MediaWorkerApi>(WORKERS.media),
  asr: lazyRow<AsrWorkerApi>(WORKERS.asr),
  audio: lazyRow<AudioWorkerApi>(WORKERS.audio),
  render: lazyRow<RenderWorkerApi>(WORKERS.render),
} satisfies Record<WorkerKind, unknown>;

/**
 * Ends worker `kind` now (D-152, D-153): it is terminated, its pending calls
 * are rejected with `E_WORKER_CRASH`, and the next call creates a fresh one.
 * A restarted render worker has no session: whoever needs one opens it again.
 */
function restartWorker(kind: WorkerKind): void {
  rows[kind].restart();
}

/**
 * Stops the pending job of worker `kind` (D-152): `"idle"` when it has none;
 * `"cancelled"` when the job answers within `CANCEL_TIMEOUT_MS`, as cancelled
 * or because it ended first (the worker is free either way); `"restarted"`
 * when it does not, after `restartWorker(kind)`.
 */
async function cancel(kind: WorkerKind): Promise<"cancelled" | "restarted" | "idle"> {
  const client = rows[kind].current();
  const jobs = client === undefined || client.isDead() ? [] : client.pendingJobs();
  if (client === undefined || jobs.length === 0) {
    return "idle";
  }
  for (const jobId of jobs) {
    client.cancel(jobId);
  }
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timedOut = new Promise<true>((resolve) => {
    timer = setTimeout(() => {
      resolve(true);
    }, CANCEL_TIMEOUT_MS);
  });
  const answered = Promise.all(jobs.map((jobId) => client.ended(jobId))).then(() => false);
  const late = await Promise.race([answered, timedOut]);
  clearTimeout(timer);
  // Only the worker that was asked is restarted: one made since is left alone.
  if (late && rows[kind].current() === client) {
    restartWorker(kind);
  }
  return late ? "restarted" : "cancelled";
}

export const pool: {
  readonly media: PoolClient<MediaWorkerApi>;
  readonly asr: PoolClient<AsrWorkerApi>;
  readonly audio: PoolClient<AudioWorkerApi>;
  readonly render: PoolClient<RenderWorkerApi>;
  cancel(kind: WorkerKind): Promise<"cancelled" | "restarted" | "idle">;
  restartWorker(kind: WorkerKind): void;
} = {
  media: rows.media.client,
  asr: rows.asr.client,
  audio: rows.audio.client,
  render: rows.render.client,
  cancel,
  restartWorker,
};

/** How often a call a worker refused as busy is asked again, and for how long at most. */
const BUSY_RETRY_MS = 16;
const BUSY_WAIT_MS = 2000;

/**
 * Runs `call`, asked again while its worker refuses it as busy: a worker
 * takes one job at a time (`rpc-worker.ts`). The render worker has several
 * callers that do not wait for each other (a change, a seek, a play, a hit
 * test, the style cards, a development build's caption mirror), and a call
 * that must not fail waits for the others this way (found by V4 Prompt
 * 117's walk-through). Any other failure, or a worker still busy after
 * `BUSY_WAIT_MS`, is the call's failure. A call that can be dropped, like a
 * hit test, is made without it.
 */
export async function whenFree<T>(call: () => Promise<T>): Promise<T> {
  const until = performance.now() + BUSY_WAIT_MS;
  for (;;) {
    try {
      return await call();
    } catch (error) {
      const busy = error instanceof WorkerCallError && error.failure.detail === "Busy";
      if (!busy || performance.now() >= until) {
        throw error;
      }
    }
    await new Promise((resolve) => setTimeout(resolve, BUSY_RETRY_MS));
  }
}

let scriptsPreloaded = false;

/** One `modulepreload` link per worker script, so each is in the HTTP cache before its worker is created. */
function preloadScripts(): void {
  if (scriptsPreloaded) {
    return;
  }
  scriptsPreloaded = true;
  for (const row of Object.values(WORKERS)) {
    const link = document.createElement("link");
    link.rel = "modulepreload";
    link.href = row.url;
    document.head.append(link);
  }
}

/**
 * Fetches and compiles what the workers will need: both WASM bundles, and
 * the worker scripts. A failure here means a bundle could not be loaded: the
 * same failure as a worker script that cannot be loaded (TS §14.4).
 */
export async function preload(): Promise<{ ok: true } | { ok: false; failure: AppFailure }> {
  preloadScripts();
  try {
    await Promise.all([preloadCore(), preloadRender()]);
    return { ok: true };
  } catch {
    return { ok: false, failure: { code: "E_WORKER_CRASH", stage: "import", retryable: true } };
  }
}
