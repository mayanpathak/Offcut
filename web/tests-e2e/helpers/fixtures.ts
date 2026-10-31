// Helpers shared by the E2E tests: the analytics flush, the files of the asset
// host served from this machine, the reference clip, the generated fixtures,
// the speech model put on the browser's device without a download, what
// the page keeps in its storage, and (v4implementation G 16.1) the editor's
// controls driven as a person drives them, the preview's statistics, the
// pixels of a screenshot and the accessibility scan; and (v5implementation
// §6.3) one processed clip shared by a suite's cases, made clean between them.

import { execFileSync, spawnSync } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import { closeSync, createReadStream, createWriteStream, existsSync, mkdirSync, openSync, readFileSync, readSync, renameSync, rmSync, statSync } from "node:fs";
import path from "node:path";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";
import { inflateSync } from "node:zlib";

import AxeBuilder from "@axe-core/playwright";
import { type Browser, type BrowserContext, expect, type Page, type Route, type TestInfo } from "@playwright/test";

import { messages } from "../../src/copy/messages";
import { paths } from "../../src/persistence/opfs";
import { DB_NAME } from "../../src/persistence/schema";
import { type FakeApi, installFakeApi } from "./fake-api";

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

/**
 * The tag of a media case that takes no clip through the whole pipeline, and
 * so passes on a machine without a graphics card: the hosted runner of CI
 * runs the cases that carry it, `--grep @no-gpu`, and no other. Processing a
 * clip there takes longer than a pull request can wait (v2implementation
 * D-72); every case, tagged or not, runs on a development machine.
 */
export const NO_GPU = "@no-gpu";

/**
 * How long a wait for the pipeline or for an export may take on this machine.
 * The suites are written for a machine with a graphics processor. One without,
 * like the hosted runner of CI, draws with the processor and is several times
 * slower: there `E2E_TIMEOUT_SCALE` says by how much a long wait is stretched.
 */
export function patient(ms: number): number {
  const scale = Number(process.env.E2E_TIMEOUT_SCALE ?? "1");
  if (!Number.isFinite(scale) || scale < 1) {
    throw new Error("E2E_TIMEOUT_SCALE is a number of 1 or more");
  }
  return ms * scale;
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

// --- The generated fixtures (v3implementation §19.15) ---------------------------

const MANIFEST = path.join(ROOT, "fixtures/manifest.json");

/** Where `pnpm fixtures` writes: `OFFCUT_FIXTURES_DIR`, or `fixtures/.gen` of this repository. */
export function fixturesDir(): string {
  const dir = process.env.OFFCUT_FIXTURES_DIR;
  return dir === undefined || dir === "" ? path.join(ROOT, "fixtures/.gen") : dir;
}

/**
 * The path of the generated fixture `name` (as `fixtures/manifest.json` names
 * it). A fixture that is absent, or that was made from another manifest than
 * this one (its `STAMP`), fails the case with "run pnpm fixtures"; it is never
 * skipped. Call it in a test, never while a file loads (V2 CL item 40).
 */
export function genFixture(name: string): string {
  const file = path.join(fixturesDir(), name);
  const stamp = path.join(fixturesDir(), "STAMP");
  if (!existsSync(file) || !existsSync(stamp)) {
    throw new Error(`${name} is not in ${fixturesDir()}: run pnpm fixtures`);
  }
  const made = (JSON.parse(readFileSync(stamp, "utf8")) as { manifest_sha256?: unknown }).manifest_sha256;
  const now = createHash("sha256").update(readFileSync(MANIFEST)).digest("hex");
  if (made !== now) {
    throw new Error(`${name} was made from another fixtures/manifest.json: run pnpm fixtures`);
  }
  return file;
}

/** The founder's second recording (D-99), on the development machine only: its path, or `null`. */
export function v3TestClip(): string | null {
  const clip = path.join(ROOT, "testclips/v3testclip.mp4");
  return existsSync(clip) ? clip : null;
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

/**
 * Runs the independent verifier on an exported file (TS §27.2) and gives its
 * exit status and what it printed: 0 only when every check it makes passes.
 * `python` is the one on PATH, which must be the Python of `verify/README.md`.
 */
export function verifyMp4(file: string, profile: "free" | "creator", expectedDurationMs: number): { status: number | null; output: string } {
  const run = spawnSync(
    "python",
    [path.join(ROOT, "verify/verify_mp4.py"), file, "--profile", profile, "--expected-duration-ms", String(expectedDurationMs)],
    { encoding: "utf8" },
  );
  if (run.error !== undefined) {
    throw run.error;
  }
  return { status: run.status, output: `${run.stdout}${run.stderr}` };
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
  /** What came back: a status with the bytes of its body, a broken connection, no answer yet, or the real host's answer. */
  answer: { status: number; bytes: number } | "aborted" | "held" | "network";
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
  /**
   * Once this many bytes of the model were served, a model request is never
   * answered: the download waits on it until the page gives it up, as a
   * cancel does (D-152). A later `routeAssets` serves the requests after it.
   */
  holdAfterBytes?: number;
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
      if (o.holdAfterBytes !== undefined && served >= o.holdAfterBytes) {
        entry.answer = "held";
        // Never settles: the page's abort ends the request, and the context's close ends this handler.
        await new Promise<never>(() => undefined);
      }
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
 * Drops `file` on the drop zone as a person would: a `DataTransfer` that holds
 * the file's bytes under `name`. Every case uses a name of its own choosing,
 * never the fixture's, so that a test can assert it appears nowhere (P-11).
 */
export async function dropFile(page: Page, file: string, name = "clip.mp4"): Promise<void> {
  // The bytes reach the page through a request the test answers itself.
  const address = `/e2e-fixture/${randomUUID()}`;
  const pattern = `**${address}`;
  await page.route(pattern, (route) => route.fulfill({ path: file, contentType: "video/mp4" }));
  const transfer = await page.evaluateHandle(
    async ({ url, name }) => {
      const blob = await (await fetch(url)).blob();
      const data = new DataTransfer();
      data.items.add(new File([blob], name, { type: "video/mp4" }));
      return data;
    },
    { url: address, name },
  );
  await page.unroute(pattern);
  await page.getByTestId("drop-zone").dispatchEvent("drop", { dataTransfer: transfer });
  await transfer.dispose();
}

/** `dropFile` under the name "clip.mp4", the same for every fixture. */
export async function dropClip(page: Page, fixture: string): Promise<void> {
  await dropFile(page, fixture);
}

const MIB = 1024 * 1024;
const OVERSIZE_PARTS = 501;
/** The size of the file `dropOversize` drops: 525,336,576 bytes, over `MAX_FILE_SIZE` (D-79). */
export const OVERSIZE_BYTES = OVERSIZE_PARTS * MIB;

/**
 * Drops a file over the size limit, made in the page (D-79): 501 references
 * to one 1 MiB `Blob`, which a `File` does not copy, named "clip.mp4". The
 * worker refuses it by its size before it reads a byte, so nothing is
 * written to disk or sent anywhere for it.
 */
export async function dropOversize(page: Page): Promise<void> {
  const transfer = await page.evaluateHandle(
    ({ mib, parts }) => {
      const part = new Blob([new Uint8Array(mib)]);
      const data = new DataTransfer();
      data.items.add(new File(Array.from({ length: parts }, () => part), "clip.mp4", { type: "video/mp4" }));
      return data;
    },
    { mib: MIB, parts: OVERSIZE_PARTS },
  );
  await page.getByTestId("drop-zone").dispatchEvent("drop", { dataTransfer: transfer });
  await transfer.dispose();
}

/**
 * The value stored under `key` in `store` of the app's database, as stored:
 * the `Versioned` wrapper and its value. `undefined` when there is none, or
 * when the app has not made its database yet (it is not made here).
 */
export function readIndexedDb(page: Page, store: "clips" | "transcripts" | "edits", key: string): Promise<unknown> {
  return page.evaluate(
    ({ name, store, key }) =>
      new Promise<unknown>((resolve, reject) => {
        const open = indexedDB.open(name);
        open.onupgradeneeded = () => {
          open.transaction?.abort();
        };
        open.onerror = (event) => {
          event.preventDefault();
          resolve(undefined);
        };
        open.onsuccess = () => {
          const db = open.result;
          const read = db.transaction(store, "readonly").objectStore(store).get(key);
          read.onsuccess = () => {
            db.close();
            resolve(read.result);
          };
          read.onerror = () => {
            db.close();
            reject(read.error ?? new Error(`readIndexedDb: ${store} ${key}`));
          };
        };
      }),
    { name: DB_NAME, store, key },
  );
}

/** The code the rejection panel shows (its `data-reject`), or `null` when the page shows none. */
export async function rejectCodeOf(page: Page): Promise<string | null> {
  const panel = page.getByTestId("clip-rejected");
  return (await panel.count()) === 0 ? null : panel.getAttribute("data-reject");
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

/** The size in bytes of a file of the page's origin private file system, or `null` when it is not there. */
export function opfsSize(page: Page, file: string): Promise<number | null> {
  return page.evaluate(
    async (segments) => {
      let handle = await navigator.storage.getDirectory();
      try {
        for (const segment of segments.slice(0, -1)) {
          handle = await handle.getDirectoryHandle(segment);
        }
        return (await (await handle.getFileHandle(segments.at(-1) ?? "")).getFile()).size;
      } catch {
        return null;
      }
    },
    file.split("/").filter((segment) => segment !== ""),
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

// --- The editor (v4implementation G 16.1) -----------------------------------------
//
// Each helper drives the page the way a person does: the scrub bar, the
// transcript's words and their field, the switches, the cards and drags on
// the preview. None reaches into a worker or a store (D-155); what the
// preview shows is read through the player's `data-*` attributes, which a
// test build (`pnpm build:e2e`) has besides the statistics every build has.
// A spec that changes the clip calls `trackSaves(page)` before it opens it.

/** The words of the save state while an edit is written, and after (§17.5's `editor.saving`, `editor.saved`). */
const SAVING = "Saving…";
const SAVED = "Saved on this device";

/** The element that holds the preview's canvas and its `data-*` attributes (D-155). */
export function player(page: Page) {
  return page.locator("div:has(> [data-testid=preview-canvas])");
}

/** What the last play drew (D-155): `NaN` for a number no play has written yet. */
export async function previewStats(
  page: Page,
): Promise<{ frames: number; late: number; maxDriftMs: number; downscaled: boolean }> {
  const root = player(page);
  const read = async (name: string): Promise<number> => Number((await root.getAttribute(name)) ?? Number.NaN);
  return {
    frames: await read("data-frames"),
    late: await read("data-late"),
    maxDriftMs: await read("data-max-drift-ms"),
    downscaled: (await root.getAttribute("data-downscaled")) === "true",
  };
}

/** The caption the preview shows, as the test build mirrors it (D-155): its text, its colours, its placement. */
export async function captionMirror(page: Page): Promise<{ text: string; colors: string[]; position: number }> {
  const root = player(page);
  const text = (await root.getAttribute("data-caption-text")) ?? "";
  const colors = ((await root.getAttribute("data-caption-colors")) ?? "").split(" ").filter((c) => c !== "");
  return { text, colors, position: Number((await root.getAttribute("data-caption-position")) ?? Number.NaN) };
}

/** Fails at once, with what to do, when the page was built without the test build's mirrors and hooks. */
export async function expectTestBuild(page: Page): Promise<void> {
  const hooks = await page.evaluate(() => "__offcutDev" in window);
  expect(hooks, "this case needs the test build's mirrors and hooks: build with pnpm build:e2e").toBe(true);
}

/**
 * Counts, from before the page loads, every time the top bar's save state
 * turns to "Saving…": the update rule shows it when a change has been drawn
 * and is being written (D-184). A change's end is then "a save after the
 * count read before it, and none in progress" (`settled`). Looking at the
 * save state alone races the change: right after a commit it still says
 * "Saved", from the change before (found by this prompt's first run).
 */
export async function trackSaves(page: Page): Promise<void> {
  await page.addInitScript((saving) => {
    const w = window as unknown as { e2eSaves: number; e2eSaving: boolean };
    w.e2eSaves = 0;
    w.e2eSaving = false;
    new MutationObserver(() => {
      const now = (document.body.textContent ?? "").includes(saving);
      if (now && !w.e2eSaving) {
        w.e2eSaves += 1;
      }
      w.e2eSaving = now;
    }).observe(document, { childList: true, subtree: true, characterData: true });
  }, SAVING);
}

/** How many saves the page has begun (`trackSaves`). */
export async function saves(page: Page): Promise<number> {
  const count = await page.evaluate(() => (window as unknown as { e2eSaves?: number }).e2eSaves);
  if (count === undefined) {
    throw new Error("trackSaves(page) must be called before the page is opened");
  }
  return count;
}

/**
 * Waits for the update rule to be done with the change made after `since`
 * (`saves(page)` read before the change): a save begun after it, and the
 * save state back to "Saved" (§17.5).
 */
export async function settled(page: Page, since: number): Promise<void> {
  await expect.poll(() => saves(page), { timeout: 30_000, message: "the change was written" }).toBeGreaterThan(since);
  await expect(page.getByText(SAVING, { exact: true })).toHaveCount(0, { timeout: 30_000 });
  await expect(page.getByText(SAVED, { exact: true })).toBeVisible({ timeout: 30_000 });
}

// --- One processed clip, shared by a suite's cases (v5implementation §6.3) -------

/** `clip-store.ts`'s `empty().edit`: what a clip's stored edit is when nothing was changed. */
const EMPTY_EDIT = { word_edits: {}, event_overrides: {}, style: "clean", crop_offset: 0, caption_position: 1, highlight_numbers: true };

export type SharedClip = { context: BrowserContext; page: Page; api: FakeApi };

/** Opens the editor on `page` and takes `fixture` to a paused, ready preview, through the cached model. */
export async function openClip(page: Page, fixture: string): Promise<FakeApi> {
  await page.setViewportSize({ width: 1280, height: 800 });
  await ensureModelCached(page.context());
  const api = await installFakeApi(page);
  await routeAssets(page);
  await trackSaves(page);
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  await expectTestBuild(page);
  await dropClip(page, fixture);
  const canvas = page.getByTestId("preview-canvas");
  await expect(canvas.or(page.getByTestId("clip-failed"))).toBeVisible({ timeout: patient(240_000) });
  await expect(canvas, "the clip was processed").toBeVisible();
  await expect(page.getByRole("button", { name: messages.preview.play, exact: true })).toBeEnabled({ timeout: 30_000 });
  return api;
}

/** `openClip` on a context and a page of its own, which the suite closes when its cases are done. */
export async function openShared(browser: Browser, info: TestInfo, fixture: string): Promise<SharedClip> {
  const { baseURL } = info.project.use;
  const context = await browser.newContext(baseURL === undefined ? {} : { baseURL });
  const page = await context.newPage();
  const api = await openClip(page, fixture);
  return { context, page, api };
}

/**
 * Sends what analytics holds now: the page is told it is hidden, on which
 * `analytics/client.ts` flushes. `flushAnalytics` needs `page.clock`, and a fake
 * clock on a shared page would touch every timer of every case. `PreviewPlayer`
 * reads `document.hidden`, which this leaves alone, so nothing else reacts.
 */
export async function sendQueuedAnalytics(page: Page): Promise<void> {
  await page.evaluate(() => {
    Object.defineProperty(document, "visibilityState", { value: "hidden", configurable: true });
    document.dispatchEvent(new Event("visibilitychange"));
    delete (document as unknown as { visibilityState?: string }).visibilityState;
  });
  // For the request to reach the route: it leaves at once, and the route is on this machine.
  await page.waitForTimeout(300);
}

/**
 * Makes a shared clip what it was when it was processed: nothing open, nothing
 * playing, every change undone, at the start, on the Captions tab, no toast,
 * and nothing recorded by the fake API. Asserts it last: an unclean clip fails
 * the case that finds it, by this name, and does not confuse the one after.
 */
export async function resetClip(page: Page, api: FakeApi): Promise<void> {
  await page.keyboard.press("Escape");
  await expect(wordField(page)).toHaveCount(0);

  // The transport has one button, whose name changes a moment after a press: a
  // press on a Play that was just Pause starts it again, and the next try stops it.
  const pause = page.getByRole("button", { name: messages.preview.pause, exact: true });
  const play = page.getByRole("button", { name: messages.preview.play, exact: true });
  await expect(async () => {
    if (await pause.isVisible()) {
      await pause.click();
    }
    await expect(play).toBeVisible({ timeout: 2_000 });
  }).toPass({ timeout: 15_000 });

  // The top bar's: the toast after a redo offers an Undo of its own.
  const undo = page.getByRole("banner").getByRole("button", { name: messages.editor.undo, exact: true });
  await expect(page.getByText(SAVING, { exact: true })).toHaveCount(0, { timeout: 30_000 });
  while (await undo.isEnabled()) {
    const since = await saves(page);
    await undo.click();
    await settled(page, since);
  }

  await seekTo(page, 0);
  await expect.poll(() => previewAt(page)).toBe(0);
  await openTab(page, "Captions");
  // A toast does not time out while the pointer is on it. `ToastHost` renders nothing with none.
  await page.mouse.move(0, 0);
  await expect(page.getByTestId("toasts")).toHaveCount(0, { timeout: patient(30_000) });

  // In this order: the events of the undo steps must not reach the next case's counts.
  await sendQueuedAnalytics(page);
  api.forget();

  await expect(undo, "resetClip: every change is undone").toBeDisabled();
  await expect(page.getByTestId("preview-failed"), "resetClip: no failure is on the page").toHaveCount(0);
  const [clipId] = await opfsList(page, "clips");
  const stored = (await readIndexedDb(page, "edits", clipId ?? "")) as { value?: { editState?: unknown } } | undefined;
  expect(stored?.value?.editState, "resetClip: the stored edit is the empty one").toEqual(EMPTY_EDIT);
}

/** The scrub bar (D-148). */
export function scrubBar(page: Page) {
  return page.getByTestId("preview-scrub");
}

/**
 * Seeks the paused preview to `ms` through the scrub bar, as a drag that
 * ends there does: the value is set, the bar's `input` moves the preview and
 * its `change` lets it go. Resolves when the value is set, not when the
 * frame is drawn: a case that times the frame watches the canvas.
 */
export async function seekTo(page: Page, ms: number): Promise<void> {
  await scrubBar(page).evaluate((element, value) => {
    const input = element as HTMLInputElement;
    // React tracks the value it last set: the native setter makes the change one it sees.
    const descriptor = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value");
    descriptor?.set?.call(input, String(value));
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
  }, ms);
}

/** Where the preview is, in milliseconds: the scrub bar's value. */
export async function previewAt(page: Page): Promise<number> {
  return Number(await scrubBar(page).inputValue());
}

/** Opens tab `name` of the side panel ("Captions", "Style", "Visuals", "Framing"). */
export async function openTab(page: Page, name: string): Promise<void> {
  const tab = page.getByRole("tab", { name, exact: true });
  await tab.click();
  await expect(tab).toHaveAttribute("aria-selected", "true");
}

/** The transcript's button of word `index` (D-182). */
export function wordButton(page: Page, index: number) {
  return page.getByTestId("caption-editor").locator(`button[data-word="${String(index)}"]`);
}

/** The field a word is fixed in (§17.5's `captions.wordField`). */
export function wordField(page: Page) {
  return page.getByRole("textbox", { name: "Fix this word" });
}

/** The words of the transcript as their buttons read now, by index, from the Captions tab (which it opens). */
export async function transcriptWords(page: Page): Promise<{ index: number; text: string }[]> {
  await openTab(page, "Captions");
  await expect(page.getByTestId("caption-editor").locator("button[data-word]").first()).toBeVisible();
  return page
    .getByTestId("caption-editor")
    .locator("button[data-word]")
    .evaluateAll((elements) =>
      elements.map((element) => ({ index: Number((element as HTMLElement).dataset.word), text: element.textContent ?? "" })),
    );
}

/**
 * Opens word `index` in its field from the Captions tab: a click on its
 * button, which also pauses the preview and seeks to the word (D-182).
 */
export async function openWord(page: Page, index: number): Promise<void> {
  await openTab(page, "Captions");
  const button = wordButton(page, index);
  await button.scrollIntoViewIfNeeded();
  await button.click();
  await expect(wordField(page)).toBeFocused();
}

/**
 * Fixes word `index` to `text` (D-182): opened, its text replaced, committed
 * with Enter; resolves when the update rule is done with it. An empty
 * `text` hides the word.
 */
export async function editWord(page: Page, index: number, text: string): Promise<void> {
  await openWord(page, index);
  await wordField(page).fill(text);
  const since = await saves(page);
  await page.keyboard.press("Enter");
  await expect(wordField(page)).toHaveCount(0);
  await settled(page, since);
}

/**
 * Switches the visual whose row is labelled `label` (`visuals.kind`'s words:
 * "Big number", "List", "Before → after", "Emphasis") to `on`, from the
 * Visuals tab; resolves when the update rule is done with it.
 */
export async function toggleItem(page: Page, label: string, on: boolean): Promise<void> {
  await openTab(page, "Visuals");
  const control = page.getByTestId("plan-items").getByRole("switch", { name: label }).first();
  if ((await control.getAttribute("aria-checked")) !== String(on)) {
    const since = await saves(page);
    await control.click();
    await expect(control).toHaveAttribute("aria-checked", String(on));
    await settled(page, since);
  }
}

/** Chooses the style card named `name` ("Clean", "Bold", "Mono") on the Style tab; resolves once it applies. */
export async function setStyle(page: Page, name: string): Promise<void> {
  await openTab(page, "Style");
  const card = page.getByTestId("style-cards").getByRole("button", { name: new RegExp(`^${name}`) });
  if ((await card.getAttribute("aria-pressed")) !== "true") {
    const since = await saves(page);
    await card.click();
    await expect(card).toHaveAttribute("aria-pressed", "true");
    await settled(page, since);
  }
}

/** The pointer layer over the canvas, which takes the gestures (D-183). */
function pointerLayer(page: Page) {
  return page.getByTestId("preview-pointer");
}

/**
 * Moves the mouse to (`x`, `y`), fractions of the preview, and waits until
 * the hover's hit test has said what is there (`data-hit`) and it is one of
 * `kinds`: a press acts on the hit the cursor showed, so a drag starts only
 * once it is known.
 */
export async function hoverPreview(page: Page, x: number, y: number, kinds: readonly string[]): Promise<boolean> {
  const box = await pointerLayer(page).boundingBox();
  if (box === null) {
    throw new Error("the preview is not on the page");
  }
  await page.mouse.move(box.x + x * box.width, box.y + y * box.height, { steps: 2 });
  const deadline = Date.now() + 1_500;
  while (Date.now() < deadline) {
    const hit = (await pointerLayer(page).getAttribute("data-hit")) ?? "";
    if (kinds.includes(hit)) {
      return true;
    }
    await page.waitForTimeout(50);
  }
  return false;
}

/** A press where the mouse is, a move to (`x`, `y`) in small steps as a hand moves, and the release. */
async function dragTo(page: Page, x: number, y: number): Promise<void> {
  const box = await pointerLayer(page).boundingBox();
  if (box === null) {
    throw new Error("the preview is not on the page");
  }
  await page.mouse.down();
  await page.mouse.move(box.x + x * box.width, box.y + y * box.height, { steps: 12 });
  await page.mouse.up();
}

// The caption block's track on the 1080 x 1920 canvas: `CAPTION_TRACK` of
// usecases/preview-interact.ts (D-183), and its snaps (`CAPTION_SNAPS`).
const TRACK = { top: 390 / 1920, bottom: 1304 / 1920 };
export const CAPTION_SNAP_AT = { top: 0.15, middle: 0.55, bottom: 1.0 } as const;

/** The centre of the caption block at `placement`, as a fraction of the preview's height. */
function blockCentre(placement: number): number {
  return TRACK.top + placement * (TRACK.bottom - TRACK.top);
}

/**
 * Drags the caption block on the paused preview to `to` (a snap's name, or a
 * placement from 0 to 1), as a person does (D-183); resolves when the one
 * change of the drag is applied. The block is taken at its centre, from the
 * player's mirror of where it is; a caption must be shown at this time.
 */
export async function dragCaptions(page: Page, to: keyof typeof CAPTION_SNAP_AT | number): Promise<void> {
  const from = blockCentre((await captionMirror(page)).position);
  const target = blockCentre(typeof to === "number" ? to : CAPTION_SNAP_AT[to]);
  let x: number | undefined;
  for (const candidate of [0.5, 0.42, 0.58, 0.34, 0.66, 0.26, 0.74]) {
    if (await hoverPreview(page, candidate, from, ["block", "word"])) {
      x = candidate;
      break;
    }
  }
  if (x === undefined) {
    throw new Error("no caption block under the pointer: is a caption shown at this time?");
  }
  const since = await saves(page);
  await dragTo(page, x, target);
  await settled(page, since);
}

/**
 * Drags the video of a wide clip sideways on the paused preview by `dx`, a
 * fraction of the preview's width (positive is to the right), as a person
 * does (D-183); resolves when the one change of the drag is applied. The
 * video is taken in its upper part, away from the captions.
 */
export async function dragVideo(page: Page, dx: number): Promise<void> {
  const start = { x: 0.5, y: 0.1 };
  if (!(await hoverPreview(page, start.x, start.y, ["video"]))) {
    throw new Error("no video under the pointer: the preview must be paused on a wide clip");
  }
  const since = await saves(page);
  await dragTo(page, start.x + dx, start.y);
  await settled(page, since);
}

export type AxeViolation = Awaited<ReturnType<AxeBuilder["analyze"]>>["violations"][number];

/** The page's serious and critical accessibility violations (D-192), by `@axe-core/playwright`. */
export async function axe(page: Page): Promise<AxeViolation[]> {
  const results = await new AxeBuilder({ page }).analyze();
  return results.violations.filter((v) => v.impact === "serious" || v.impact === "critical");
}

// --- The pixels of a screenshot -----------------------------------------------------

/** The pixels of an 8-bit, non-interlaced RGB or RGBA PNG, as a screenshot is: decoded and unfiltered. */
export function pngPixels(png: Buffer): { width: number; height: number; channels: number; data: Uint8Array } {
  const width = png.readUInt32BE(16);
  const height = png.readUInt32BE(20);
  const depth = png.readUInt8(24);
  const colorType = png.readUInt8(25);
  const channels = colorType === 6 ? 4 : colorType === 2 ? 3 : 0;
  if (depth !== 8 || channels === 0 || png.readUInt8(28) !== 0) {
    throw new Error("only 8-bit RGB or RGBA PNGs without interlacing are read here");
  }
  const chunks: Buffer[] = [];
  for (let at = 8; at + 8 <= png.length; ) {
    const length = png.readUInt32BE(at);
    if (png.toString("latin1", at + 4, at + 8) === "IDAT") {
      chunks.push(png.subarray(at + 8, at + 8 + length));
    }
    at += 12 + length;
  }
  const raw = inflateSync(Buffer.concat(chunks));
  const stride = width * channels;
  const data = new Uint8Array(height * stride);
  for (let y = 0; y < height; y += 1) {
    const filter = raw[y * (stride + 1)] ?? 0;
    const line = y * (stride + 1) + 1;
    for (let i = 0; i < stride; i += 1) {
      const value = raw[line + i] ?? 0;
      const a = i >= channels ? (data[y * stride + i - channels] ?? 0) : 0;
      const b = y > 0 ? (data[(y - 1) * stride + i] ?? 0) : 0;
      const c = y > 0 && i >= channels ? (data[(y - 1) * stride + i - channels] ?? 0) : 0;
      let predicted = 0;
      if (filter === 1) {
        predicted = a;
      } else if (filter === 2) {
        predicted = b;
      } else if (filter === 3) {
        predicted = Math.floor((a + b) / 2);
      } else if (filter === 4) {
        const p = a + b - c;
        const [pa, pb, pc] = [Math.abs(p - a), Math.abs(p - b), Math.abs(p - c)];
        predicted = pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
      }
      data[y * stride + i] = (value + predicted) & 0xff;
    }
  }
  return { width, height, channels, data };
}

/** The mean brightness of a screenshot, 0 to 255: the mean of its pixels' R, G and B. */
export function meanBrightness(png: Buffer): number {
  const { width, height, channels, data } = pngPixels(png);
  let sum = 0;
  for (let p = 0; p < width * height; p += 1) {
    sum += (data[p * channels] ?? 0) + (data[p * channels + 1] ?? 0) + (data[p * channels + 2] ?? 0);
  }
  return sum / (width * height * 3);
}
