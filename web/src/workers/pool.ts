// The worker pool: one typed client per worker. A worker is created on the
// first call to its client and kept (TS §14.3). At app start `preload()`
// fetches what the workers will need (D-9), which also proves in production
// that the build, the MIME type and the CSP are right.
//
// Four workers: media, ASR, audio (V3) and render. At most four exist (TS §31).

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
} from "./protocol";
import renderWorkerUrl from "./render.worker.ts?worker&url";
import { type Client, createClient, WorkerCallError } from "./rpc";

// A use-case may import this file only, of all the workers' files (D-32).
export { type Cancelled, WorkerCallError } from "./rpc";

type PoolClient<Api> = Client<Api> & { notify(method: string, params: unknown): void };

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

/** A client whose worker is created by its first call. */
function lazyClient<Api>(row: { url: string; stage: FailureStage }): PoolClient<Api> {
  let client: ReturnType<typeof createClient<Api>> | undefined;
  const started = (): ReturnType<typeof createClient<Api>> => {
    client ??= createClient<Api>(new Worker(row.url, { type: "module" }), row.stage);
    return client;
  };
  // The methods of `Api` are not known when this runs, so every name is one.
  return new Proxy(
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
        return Reflect.get(started().call, name) as unknown;
      },
    },
  ) as PoolClient<Api>;
}

export const pool: {
  readonly media: PoolClient<MediaWorkerApi>;
  readonly asr: PoolClient<AsrWorkerApi>;
  readonly audio: PoolClient<AudioWorkerApi>;
  readonly render: PoolClient<RenderWorkerApi>;
} = {
  media: lazyClient<MediaWorkerApi>(WORKERS.media),
  asr: lazyClient<AsrWorkerApi>(WORKERS.asr),
  audio: lazyClient<AudioWorkerApi>(WORKERS.audio),
  render: lazyClient<RenderWorkerApi>(WORKERS.render),
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
