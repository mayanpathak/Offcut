// Helpers shared by the E2E tests: the analytics flush, the files of the asset
// host served from this machine, the reference clip, and the speech model put
// on the browser's device without a download.

import { execFileSync } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import { closeSync, createReadStream, createWriteStream, existsSync, mkdirSync, openSync, readFileSync, readSync, renameSync, rmSync, statSync } from "node:fs";
import path from "node:path";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";

import type { BrowserContext, Page, Route } from "@playwright/test";

import { paths } from "../../src/persistence/opfs";

// FLUSH_INTERVAL_MS of src/analytics/client.ts, and API_TIMEOUT_MS of
// src/net/http.ts: both 10 s. Neither file can be imported here: they read
// the build configuration of the app.
const FLUSH_INTERVAL_MS = 10_000;

/**
 * Makes the analytics client send what it has queued, by moving the page's
 * clock to its next flush. The test must have called `page.clock.install()`
 * before it opened the page; no real time passes.
 *
 * The clock is moved by a millisecond less than the interval, and not past
 * it. A request gives up after 10 s of the page's clock, and a flush that
 * fires early in a longer step would have its own request reach that limit
 * within the same step, before the browser has sent it: the batch would be
 * dropped and no test would ever see it. A flush that is due in the one
 * millisecond left out fires by itself, since the clock runs on. For the
 * same reason a second call waits for what the first one sent.
 */
export async function flushAnalytics(page: Page): Promise<void> {
  await page.clock.runFor(FLUSH_INTERVAL_MS - 1);
}

// --- Where things are ------------------------------------------------------------

const ROOT = path.resolve(import.meta.dirname, "../../..");
/** Model files and the reference clip, as the asset host has them. Not in git (D-41). */
const CACHE = path.join(ROOT, "fixtures/.cache");

type ManifestFile = { path: string; bytes: number; sha256: string };

/** The model this build downloads: `web/src/config/model-manifest.json`. */
export const modelManifest = JSON.parse(
  readFileSync(path.join(ROOT, "web/src/config/model-manifest.json"), "utf8"),
) as { modelId: string; modelVersion: string; files: ManifestFile[]; totalBytes: number };

/** The base URL of the asset host, as the build under test has it: no trailing slash. */
export function assetBaseUrl(): string {
  let raw = process.env.VITE_ASSET_BASE_URL;
  const envLocal = path.join(ROOT, "web/.env.local");
  if ((raw === undefined || raw === "") && existsSync(envLocal)) {
    raw = readFileSync(envLocal, "utf8")
      .split(/\r?\n/)
      .find((line) => line.startsWith("VITE_ASSET_BASE_URL="))
      ?.slice("VITE_ASSET_BASE_URL=".length)
      .trim();
  }
  if (raw === undefined || raw === "") {
    throw new Error("VITE_ASSET_BASE_URL is not set, in the environment or in web/.env.local");
  }
  return raw.replace(/\/+$/, "");
}

/**
 * `SAMPLE_CLIP_PATH` of src/net/asset-fetch.ts, read from its source: that
 * file cannot be imported here, because it reads the build configuration.
 */
export function sampleClipPath(): string {
  const source = readFileSync(path.join(ROOT, "web/src/net/asset-fetch.ts"), "utf8");
  const found = /^export const SAMPLE_CLIP_PATH = "([^"]+)";$/m.exec(source)?.[1];
  if (found === undefined) {
    throw new Error("SAMPLE_CLIP_PATH was not found in web/src/net/asset-fetch.ts");
  }
  return found;
}

// A file on the asset host is `<stem>.<16 hex>.<extension>`: the hex is the
// start of its SHA-256 (D-62).
const STORED_NAME = /^(?<stem>.+)\.(?<hash>[0-9a-f]{16})\.(?<extension>[^.]+)$/;

function storedName(assetPath: string): { plain: string; hash: string } {
  const groups = STORED_NAME.exec(path.posix.basename(assetPath))?.groups;
  if (groups?.stem === undefined || groups.hash === undefined || groups.extension === undefined) {
    throw new Error(`${assetPath}: not <name>.<hash>.<extension>`);
  }
  return { plain: `${groups.stem}.${groups.extension}`, hash: groups.hash };
}

/**
 * The reference clip (D-39): the file in `testclips/` when it is there, as on
 * the development machine; else the copy that `fillAssetCache()` downloads.
 */
export function referenceClip(): string {
  const sample = sampleClipPath();
  const local = path.join(ROOT, "testclips", storedName(sample).plain);
  return existsSync(local) ? local : path.join(CACHE, sample);
}

/** The length of the video stream of `fixture`, in whole milliseconds, as `ffprobe` reads it. */
export function sourceDurationMs(fixture: string): number {
  const seconds = Number(
    execFileSync(
      "ffprobe",
      ["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=duration", "-of", "default=noprint_wrappers=1:nokey=1", fixture],
      { encoding: "utf8" },
    ).trim(),
  );
  if (!Number.isFinite(seconds) || seconds <= 0) {
    throw new Error(`ffprobe gave no duration for ${fixture}`);
  }
  return Math.round(seconds * 1000);
}

// --- The cache of the asset host's files -----------------------------------------

function sha256Of(file: string): Promise<string> {
  const hash = createHash("sha256");
  return pipeline(createReadStream(file), hash).then(() => hash.digest("hex"));
}

/** Makes `fixtures/.cache/<assetPath>` hold the file whose SHA-256 starts with `sha256`. */
async function cacheFile(assetPath: string, sha256: string): Promise<void> {
  const target = path.join(CACHE, assetPath);
  if (existsSync(target)) {
    if ((await sha256Of(target)).startsWith(sha256)) {
      return;
    }
    rmSync(target);
  }
  mkdirSync(path.dirname(target), { recursive: true });
  const response = await fetch(`${assetBaseUrl()}/${assetPath}`);
  if (!response.ok || response.body === null) {
    throw new Error(`${assetPath}: the asset host answered ${String(response.status)}`);
  }
  const part = `${target}.part`;
  await pipeline(Readable.fromWeb(response.body), createWriteStream(part));
  const got = await sha256Of(part);
  if (!got.startsWith(sha256)) {
    rmSync(part);
    throw new Error(`${assetPath}: downloaded a file whose SHA-256 is ${got}, not ${sha256}`);
  }
  renameSync(part, target);
}

/**
 * Downloads what `fixtures/.cache/` lacks from the asset host and checks each
 * file's SHA-256: the files of the model manifest and, where `testclips/`
 * does not hold it, the reference clip (D-39, D-41). A file that is there
 * with the right hash is not fetched again.
 */
export async function fillAssetCache(): Promise<void> {
  for (const file of modelManifest.files) {
    await cacheFile(file.path, file.sha256);
  }
  if (referenceClip().startsWith(CACHE)) {
    const sample = sampleClipPath();
    await cacheFile(sample, storedName(sample).hash);
  }
}

// --- The asset host, answered from this machine ----------------------------------

export type AssetRequest = {
  method: string;
  url: string;
  /** The path on the asset host, without the base URL. */
  path: string;
  /** Every header as the browser sent it, a repeated one twice. Names are in lower case. */
  headers: { name: string; value: string }[];
  hasBody: boolean;
  /** What came back: a status with the bytes of its body, a broken connection, or the real host's answer. */
  answer: { status: number; bytes: number } | "aborted" | "network";
};

export type AssetLog = {
  /** Every request to the asset host, in the order it was made. */
  requests: AssetRequest[];
  /** Those that ask for a file of the model manifest. */
  modelRequests(): AssetRequest[];
};

export type AssetOptions = {
  /** A file of the model whose path contains this is served with one byte changed. */
  corrupt?: string;
  /**
   * Once this many bytes of the model were served, a model request fails:
   * the connection breaks, or `status` is answered when that is given too.
   */
  failAfterBytes?: number;
  /** Without `failAfterBytes`: every model request is answered with this status. */
  status?: number;
};

const CONTENT_TYPES: Record<string, string> = {
  ".json": "application/json",
  ".mp4": "video/mp4",
  ".onnx": "application/octet-stream",
};

function readSlice(file: string, start: number, length: number): Buffer {
  const buffer = Buffer.alloc(length);
  const handle = openSync(file, "r");
  try {
    let done = 0;
    while (done < length) {
      const read = readSync(handle, buffer, done, length - done, start + done);
      if (read === 0) {
        break;
      }
      done += read;
    }
    return buffer.subarray(0, done);
  } finally {
    closeSync(handle);
  }
}

/** The file of this machine that answers for `assetPath`, or `undefined` for one the cache does not hold. */
function localFile(assetPath: string): string | undefined {
  if (modelManifest.files.some((file) => file.path === assetPath)) {
    return path.join(CACHE, assetPath);
  }
  return assetPath === sampleClipPath() ? referenceClip() : undefined;
}

/**
 * Answers one request as the asset host does: the whole file with 200, or the
 * bytes of a `Range` with 206 and `Content-Range`. A range that ends past
 * the file gets the bytes there are. Returns the status and the body's size.
 */
async function answerFromFile(route: Route, file: string, corrupt: boolean): Promise<{ status: number; bytes: number }> {
  const request = route.request();
  const size = statSync(file).size;
  const headers: Record<string, string> = {
    "access-control-allow-origin": (await request.headerValue("origin")) ?? "*",
    "access-control-expose-headers": "Content-Range, Accept-Ranges, Content-Length",
    "accept-ranges": "bytes",
    "cross-origin-resource-policy": "cross-origin",
    "content-type": CONTENT_TYPES[path.extname(file)] ?? "application/octet-stream",
  };
  const range = /^bytes=(\d+)-(\d*)$/.exec((await request.headerValue("range")) ?? "");
  if (range === null && !corrupt) {
    await route.fulfill({ status: 200, headers, path: file });
    return { status: 200, bytes: size };
  }
  const start = range === null ? 0 : Number(range[1]);
  const end = range === null || range[2] === "" ? size - 1 : Math.min(Number(range[2]), size - 1);
  if (start > end) {
    await route.fulfill({ status: 416, headers: { ...headers, "content-range": `bytes */${String(size)}` } });
    return { status: 416, bytes: 0 };
  }
  const body = readSlice(file, start, end - start + 1);
  if (corrupt && body.length > 0) {
    body[0] = (body[0] ?? 0) ^ 0xff;
  }
  if (range === null) {
    await route.fulfill({ status: 200, headers, body });
    return { status: 200, bytes: body.length };
  }
  await route.fulfill({
    status: 206,
    headers: { ...headers, "content-range": `bytes ${String(start)}-${String(end)}/${String(size)}` },
    body,
  });
  return { status: 206, bytes: body.length };
}

/**
 * Answers the page's requests to the asset host from `fixtures/.cache/` (the
 * model) and `referenceClip()` (the sample clip), and records every one. The
 * page still asks the asset host's URL, so what a test asserts about hosts
 * holds (D-41). A path the cache does not hold, the demo video for one, goes
 * to the real host. With `E2E_REAL_ASSETS=1` every request does, and is only
 * recorded. A second call for the same page takes the place of the first.
 */
export async function routeAssets(page: Page, o: AssetOptions = {}): Promise<AssetLog> {
  const base = assetBaseUrl();
  const pattern = `${base}/**`;
  const real = process.env.E2E_REAL_ASSETS === "1";
  const requests: AssetRequest[] = [];
  const isModel = (assetPath: string): boolean => modelManifest.files.some((file) => file.path === assetPath);
  let served = 0;

  await page.unroute(pattern);
  await page.route(pattern, async (route) => {
    const request = route.request();
    const assetPath = request.url().slice(base.length + 1).split(/[?#]/)[0] ?? "";
    const entry: AssetRequest = {
      method: request.method(),
      url: request.url(),
      path: assetPath,
      headers: (await request.headersArray()).map(({ name, value }) => ({ name: name.toLowerCase(), value })),
      hasBody: request.postDataBuffer() !== null,
      answer: "network",
    };
    requests.push(entry);

    const file = localFile(assetPath);
    if (real || file === undefined) {
      await route.continue();
      return;
    }
    if (isModel(assetPath)) {
      const failing = o.failAfterBytes === undefined ? o.status !== undefined : served >= o.failAfterBytes;
      if (failing && o.status !== undefined) {
        entry.answer = { status: o.status, bytes: 0 };
        await route.fulfill({ status: o.status, headers: { "access-control-allow-origin": "*" } });
        return;
      }
      if (failing) {
        entry.answer = "aborted";
        await route.abort("connectionreset");
        return;
      }
    }
    entry.answer = await answerFromFile(route, file, o.corrupt !== undefined && assetPath.includes(o.corrupt));
    if (isModel(assetPath)) {
      served += entry.answer.bytes;
    }
  });

  return { requests, modelRequests: () => requests.filter((entry) => isModel(entry.path)) };
}

// --- The page's device -----------------------------------------------------------

/**
 * Drops `fixture` on the drop zone as a person would: a `DataTransfer` that
 * holds the file's bytes under the name "clip.mp4". The name is the same for
 * every fixture, so that a test can assert it appears nowhere (P-11).
 */
export async function dropClip(page: Page, fixture: string): Promise<void> {
  // The bytes reach the page through a request the test answers itself.
  const address = `/e2e-fixture/${randomUUID()}`;
  const pattern = `**${address}`;
  await page.route(pattern, (route) => route.fulfill({ path: fixture, contentType: "video/mp4" }));
  const transfer = await page.evaluateHandle(async (url) => {
    const blob = await (await fetch(url)).blob();
    const data = new DataTransfer();
    data.items.add(new File([blob], "clip.mp4", { type: "video/mp4" }));
    return data;
  }, address);
  await page.unroute(pattern);
  await page.getByTestId("drop-zone").dispatchEvent("drop", { dataTransfer: transfer });
  await transfer.dispose();
}

/** The names in a directory of the page's origin private file system, sorted. An absent directory gives `[]`. */
export function opfsList(page: Page, dir: string): Promise<string[]> {
  return page.evaluate(
    async (segments) => {
      let handle = await navigator.storage.getDirectory();
      for (const segment of segments) {
        try {
          handle = await handle.getDirectoryHandle(segment);
        } catch {
          return [];
        }
      }
      const names: string[] = [];
      for await (const name of handle.keys()) {
        names.push(name);
      }
      return names.sort();
    },
    dir.split("/").filter((segment) => segment !== ""),
  );
}

/** How many bytes of a model file a page fetches at a time while the model is put on its device. */
const CACHE_CHUNK_BYTES = 16 * 1024 * 1024;
const BLANK_PAGE = "/e2e-fixture/blank";

/**
 * Puts the speech model on the device of `context`, as a finished download
 * leaves it: every file of the manifest under its plain name in
 * `models/<modelId>/`. The bytes come from `fixtures/.cache/`, never from
 * the asset host, and the app does not run while they are written. Files
 * that are there already are left alone.
 */
export async function ensureModelCached(context: BrowserContext): Promise<void> {
  const base = assetBaseUrl();
  const files = modelManifest.files.map((file) => ({
    url: `${base}/${file.path}`,
    name: storedName(file.path).plain,
    bytes: file.bytes,
  }));
  const page = await context.newPage();
  try {
    // A page of the app's origin that is not the app: only the origin matters.
    await page.route(`**${BLANK_PAGE}`, (route) => route.fulfill({ contentType: "text/html", body: "<!doctype html><title>e2e</title>" }));
    await page.route(`${base}/**`, async (route) => {
      const assetPath = route.request().url().slice(base.length + 1);
      await answerFromFile(route, path.join(CACHE, assetPath), false);
    });
    await page.goto(BLANK_PAGE);
    await page.evaluate(
      async ({ dir, files, chunk }) => {
        let handle = await navigator.storage.getDirectory();
        for (const segment of dir.split("/")) {
          handle = await handle.getDirectoryHandle(segment, { create: true });
        }
        for (const file of files) {
          const present = await handle
            .getFileHandle(file.name)
            .then((existing) => existing.getFile())
            .then((existing) => existing.size)
            .catch(() => -1);
          if (present === file.bytes) {
            continue;
          }
          const writable = await (await handle.getFileHandle(file.name, { create: true })).createWritable();
          for (let start = 0; start < file.bytes; start += chunk) {
            const end = Math.min(start + chunk, file.bytes) - 1;
            const response = await fetch(file.url, { headers: { Range: `bytes=${String(start)}-${String(end)}` } });
            if (response.status !== 206) {
              throw new Error(`${file.name}: the cache answered ${String(response.status)}`);
            }
            await writable.write(await response.arrayBuffer());
          }
          await writable.close();
          // What an interrupted download left of this file has no use any more.
          await handle.removeEntry(`${file.name}.part`).catch(() => undefined);
        }
      },
      { dir: paths.modelDir(modelManifest.modelId), files, chunk: CACHE_CHUNK_BYTES },
    );
  } finally {
    await page.close();
  }
}
