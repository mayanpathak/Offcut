// The worker pool: one typed client per worker. A worker is created on the
// first call to its client and kept (TS §14.3). At app start `preload()`
// fetches what the workers will need (D-9), which also proves in production
// that the build, the MIME type and the CSP are right.
//
// V2 has the media worker. The ASR and render workers get their rows with
// their scripts; V3 adds the audio worker.

import { preloadCore } from "../wasm/load-core";
import mediaWorkerUrl from "./media.worker.ts?worker&url";
import type { AppFailure, FailureStage, MediaWorkerApi } from "./protocol";
import { type Client, createClient } from "./rpc";

// A use-case may import this file only, of all the workers' files (D-32).
export { type Cancelled, WorkerCallError } from "./rpc";

type PoolClient<Api> = Client<Api> & { notify(method: string, params: unknown): void };

// One row per worker: its script, and the stage a failure in it is tagged
// with unless the failure names its own. The lists of one-way methods and of
// methods accepted beside a running preview are given to `serveWorker` by
// each worker's own entry file.
const WORKERS = {
  media: { url: mediaWorkerUrl, stage: "import" },
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

export const pool: { readonly media: PoolClient<MediaWorkerApi> } = {
  media: lazyClient<MediaWorkerApi>(WORKERS.media),
};

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
 * Fetches and compiles what the workers will need. A failure here means the
 * bundle could not be loaded: the same failure as a worker script that
 * cannot be loaded (TS §14.4).
 */
export async function preload(): Promise<{ ok: true } | { ok: false; failure: AppFailure }> {
  preloadScripts();
  try {
    await preloadCore();
    return { ok: true };
  } catch {
    return { ok: false, failure: { code: "E_WORKER_CRASH", stage: "import", retryable: true } };
  }
}
