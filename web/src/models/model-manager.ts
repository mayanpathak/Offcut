// The speech model on this device: whether its files are here, and getting
// them here (TS §16). This file downloads and checks the files of
// `config/model-manifest.json` and keeps the model store up to date. It
// never runs the model, and it removes a checked file only in `clear()` or
// when the manifest names another model.

import manifest from "../config/model-manifest.json";
import type { Bytes, ErrorCode } from "../gen/domain";
import { metaGet, metaSet } from "../persistence/db";
import * as opfs from "../persistence/opfs";
import * as modelStore from "../state/model-store";
import { loadCore } from "../wasm/load-core";
import type { AppFailure } from "../workers/protocol";
import { fetchRanged, localName, type ManifestFile, ModelError, type VerifyDeps, verifyAndFinalize } from "./download";

type ModelProgress = modelStore.ModelProgress;

/** Why `ensureReady` did not end in a model that is ready. A use-case reads `failure`. */
export class ModelFailure extends Error {
  readonly failure: AppFailure;

  constructor(failure: AppFailure, cause: unknown) {
    super(failure.code, { cause });
    this.name = "ModelFailure";
    this.failure = failure;
  }
}

/** No time remaining is given before a download has run this long. */
const ETA_AFTER_MS = 2_000;
/** The time remaining is worked out from the bytes of this long a stretch. */
const RATE_WINDOW_MS = 5_000;
const MS_PER_SECOND = 1_000;

// The key under which `meta` holds that persistent storage was asked for:
// `META_KEYS.persistRequested` of persistence/schema.ts, which this layer may
// not import.
const PERSIST_REQUESTED = "persistRequested";

const MODEL_DIR = opfs.paths.modelDir(manifest.modelId);
/** The directory that holds one directory per model. */
const MODELS_ROOT = MODEL_DIR.slice(0, MODEL_DIR.lastIndexOf("/"));

const VERIFY_DEPS: VerifyDeps = {
  opfs,
  newSha256: async () => (await loadCore()).newSha256(),
  pause: () =>
    new Promise((resolve) => {
      setTimeout(resolve, 0);
    }),
};

/** The one place of this file that turns a number into `Bytes` (D-59). */
function toBytes(n: number): Bytes {
  return n as Bytes;
}

const finalPath = (file: ManifestFile): string => opfs.paths.modelFile(manifest.modelId, localName(file));
const partPath = (file: ManifestFile): string => opfs.paths.modelPart(manifest.modelId, localName(file));
const status = () => modelStore.useModelStore.getState().status;

async function look(): Promise<"absent" | "partial" | "ready"> {
  // A directory of another model is one this build has no use for (TS §16.5).
  for (const name of await opfs.list(MODELS_ROOT)) {
    if (name !== manifest.modelId) {
      await opfs.remove(opfs.paths.modelDir(name));
    }
  }
  let present = 0;
  let final = 0;
  for (const file of manifest.files) {
    const finalSize = await opfs.size(finalPath(file));
    if (finalSize === file.bytes) {
      final += 1;
    }
    if (finalSize !== null || (await opfs.size(partPath(file))) !== null) {
      present += 1;
    }
  }
  if (final === manifest.files.length) {
    return "ready";
  }
  return present > 0 ? "partial" : "absent";
}

/**
 * What of the model is on this device. The first call also tells the model
 * store. A storage that cannot be read counts as no model.
 */
export async function inspect(): Promise<"absent" | "partial" | "ready"> {
  let result: "absent" | "partial" | "ready";
  try {
    result = await look();
  } catch {
    result = "absent";
  }
  if (status() === "unknown") {
    modelStore.inspected(result);
  }
  return result;
}

/** Asks the browser, once per device, not to evict what Offcut stores. The answer changes nothing here. */
async function requestPersistence(): Promise<void> {
  if ((await metaGet<boolean>(PERSIST_REQUESTED)) === true) {
    return;
  }
  await navigator.storage.persist();
  await metaSet(PERSIST_REQUESTED, true);
}

/** Reports how far the download is, with the time remaining once it can be told. */
function meter(total: Bytes, onProgress: (p: ModelProgress) => void): (done: number) => void {
  const startedAt = performance.now();
  const samples: { at: number; done: number }[] = [];
  return (done) => {
    const at = performance.now();
    samples.push({ at, done });
    // Keep one sample from before the window: the rate is measured from it.
    while (samples.length > 2 && at - (samples[1]?.at ?? at) >= RATE_WINDOW_MS) {
      samples.shift();
    }
    const first = samples[0];
    let etaSecs: number | null = null;
    if (first !== undefined && at - startedAt >= ETA_AFTER_MS && at > first.at && done > first.done) {
      const bytesPerSecond = ((done - first.done) * MS_PER_SECOND) / (at - first.at);
      etaSecs = Math.ceil((total - done) / bytesPerSecond);
    }
    const p: ModelProgress = { done: toBytes(Math.min(done, total)), total, etaSecs };
    modelStore.progress(p);
    onProgress(p);
  };
}

/** What a failed run leaves in the store, and what `ensureReady` rejects with. */
function settle(error: unknown): unknown {
  const isAbort = error instanceof DOMException && error.name === "AbortError";
  if (isAbort && status() === "downloading") {
    // The bytes that arrived are kept: the next run goes on from them.
    modelStore.netFail();
    return error;
  }
  const code: ErrorCode = error instanceof ModelError ? error.code : "E_INTERNAL";
  if (status() === "downloading" || status() === "verifying") {
    modelStore.fail(code);
  }
  return new ModelFailure({ code, stage: "model", retryable: code !== "E_INTERNAL" }, error);
}

async function run(onProgress: (p: ModelProgress) => void, signal: AbortSignal): Promise<void> {
  if (status() === "unknown") {
    await inspect();
  }
  if (status() === "ready") {
    return;
  }
  // Best effort (TS §16.3): a browser that will not answer must not stop the download.
  await requestPersistence().catch(() => undefined);

  modelStore.start();
  try {
    const total = toBytes(manifest.totalBytes);
    const pending: { file: ManifestFile; resumeFrom: Bytes }[] = [];
    let done = 0;
    for (const file of manifest.files) {
      if ((await opfs.size(finalPath(file))) === file.bytes) {
        done += file.bytes;
        continue;
      }
      const resumeFrom = (await opfs.size(partPath(file))) ?? toBytes(0);
      pending.push({ file, resumeFrom });
      done += Math.min(resumeFrom, file.bytes);
    }

    const report = meter(total, onProgress);
    report(done);
    for (const [index, { file, resumeFrom }] of pending.entries()) {
      await fetchRanged(file, {
        resumeFrom,
        signal,
        onBytes: (n) => {
          done += n;
          report(done);
        },
      });
      if (index === pending.length - 1) {
        modelStore.lastByte();
      }
      await verifyAndFinalize(manifest.modelId, file, VERIFY_DEPS);
    }
    if (pending.length === 0) {
      modelStore.lastByte();
    }
    modelStore.hashOk();
  } catch (error) {
    throw settle(error);
  }
}

let running: Promise<void> | undefined;

/**
 * Resolves once every file of the model is on this device under its final
 * name, downloading and checking what is missing. A call made while another
 * runs gets the same promise. Rejects with a `ModelFailure`, or with the
 * abort when `signal` aborted the download.
 */
export function ensureReady(onProgress: (p: ModelProgress) => void, signal: AbortSignal): Promise<void> {
  if (status() === "ready") {
    return Promise.resolve();
  }
  running ??= run(onProgress, signal).finally(() => {
    running = undefined;
  });
  return running;
}

/** `bytes` is what the files of the model take on this device, parts included. */
export async function cacheInfo(): Promise<{ modelId: string; modelVersion: string; bytes: Bytes }> {
  let bytes = 0;
  for (const name of await opfs.list(MODEL_DIR)) {
    bytes += (await opfs.size(opfs.paths.modelFile(manifest.modelId, name))) ?? 0;
  }
  return { modelId: manifest.modelId, modelVersion: manifest.modelVersion, bytes: toBytes(bytes) };
}

/** Removes every model file. */
export async function clear(): Promise<void> {
  await opfs.remove(MODELS_ROOT);
  if (status() === "ready" || status() === "partial") {
    modelStore.cleared();
  }
}
