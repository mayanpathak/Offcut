// J4 in a real browser: the first-run download of the speech model, against
// the built app served with the production headers. The asset host is
// answered from `fixtures/.cache/` and the API is a fake, so no case here
// leaves this machine.
//
// A case starts with nothing on the device unless it says otherwise: every
// test has a browser context, and so a file system, of its own.

import { statSync } from "node:fs";

import { expect, type Page, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import { type FakeApi, installFakeApi } from "./helpers/fake-api";
import {
  type AssetLog,
  type AssetOptions,
  type AssetRequest,
  assetBaseUrl,
  dropClip,
  ensureModelCached,
  flushAnalytics,
  modelManifest,
  opfsList,
  opfsSize,
  referenceClip,
  routeAssets,
} from "./helpers/fixtures";

declare global {
  interface Window {
    /** Every value the bar of the model panel took, in order. */
    e2eBar?: string[];
    /** Whether the model panel was on the page at any time. */
    e2ePanelSeen?: boolean;
  }
}

const MIB = 1024 * 1024;
/** One request of the downloader asks for at most this much (`MODEL_PART_BYTES`). */
const PART_BYTES = 8 * MIB;
/** A request that fails is made again three times (`MODEL_RETRIES`). */
const ATTEMPTS = 4;

// The size the panel names: the manifest's total, to the nearest 10 MB (TS §16.1).
const SIZE_MB = Math.round(modelManifest.totalBytes / 1_000_000 / 10) * 10;
const PANEL_TEXT = messages.modelDownload.body(SIZE_MB);
const MODEL_DIR = `models/${modelManifest.modelId}`;

/** A file of the manifest, with the name it has on the device. */
function manifestFile(index: number): { path: string; bytes: number; localName: string } {
  const file = modelManifest.files[index];
  if (file === undefined) {
    throw new Error(`the manifest has no file ${String(index)}`);
  }
  const stored = file.path.slice(file.path.lastIndexOf("/") + 1);
  return { path: file.path, bytes: file.bytes, localName: stored.replace(/\.[0-9a-f]{16}(\.[^.]+)$/, "$1") };
}

const panel = (page: Page) => page.getByText(PANEL_TEXT);
const feedLines = (page: Page) => page.getByTestId("feed").locator("li");
const failure = (page: Page) => page.getByTestId("clip-failed");

const header = (request: AssetRequest, name: string) => request.headers.filter((entry) => entry.name === name);
/** The first byte a model request asks for. */
function rangeStart(request: AssetRequest | undefined): number {
  const range = request === undefined ? undefined : header(request, "range")[0]?.value;
  const start = /^bytes=(\d+)-\d+$/.exec(range ?? "")?.[1];
  return start === undefined ? Number.NaN : Number(start);
}

/** Records what the model panel does, from before the page loads: a look at one moment would miss it. */
async function watchPanel(page: Page): Promise<void> {
  await page.addInitScript((text) => {
    window.e2eBar = [];
    window.e2ePanelSeen = false;
    new MutationObserver(() => {
      const bar = document.querySelector('[data-testid="editor"] [role="progressbar"]');
      if (!(document.querySelector("body")?.textContent ?? "").includes(text) || bar === null) {
        return;
      }
      window.e2ePanelSeen = true;
      const now = bar.getAttribute("aria-valuenow") ?? "";
      if (now !== window.e2eBar?.at(-1)) {
        window.e2eBar?.push(now);
      }
    }).observe(document, { childList: true, subtree: true, attributes: true, characterData: true });
  }, PANEL_TEXT);
}

/** Opens the editor with a fake API, the asset host answered from this machine, and a clock the test can move. */
async function openEditor(page: Page, assets: AssetOptions = {}): Promise<{ api: FakeApi; assets: AssetLog }> {
  const api = await installFakeApi(page);
  const log = await routeAssets(page, assets);
  await watchPanel(page);
  await page.clock.install();
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  return { api, assets: log };
}

/** The one `model_download` event of the session, once analytics has sent it. */
async function modelDownloadEvent(page: Page, api: FakeApi): Promise<unknown> {
  await flushAnalytics(page);
  await expect.poll(() => api.eventsOf("model_download").length).toBeGreaterThan(0);
  return api.eventsOf("model_download").at(-1);
}

test("first run: the panel says what is downloaded, its bar moves, and the feed follows", async ({ page }) => {
  const { api } = await openEditor(page);
  await dropClip(page, referenceClip());

  // The J4 sentence, with the size worked out from the manifest.
  await expect(panel(page)).toBeVisible({ timeout: 30_000 });
  expect(PANEL_TEXT).toContain(`about ${String(SIZE_MB)} MB`);
  await expect(panel(page)).toBeHidden({ timeout: 180_000 });
  // "Advances at least twice": three different values or more.
  const bar = await page.evaluate(() => window.e2eBar ?? []);
  expect(new Set(bar).size).toBeGreaterThanOrEqual(3);
  expect(bar.at(-1)).toBe("100");

  await expect(feedLines(page).first()).toHaveText(messages.feed.transcribing, { timeout: 180_000 });
  expect(await modelDownloadEvent(page, api)).toEqual({
    name: "model_download",
    props: { outcome: "ok", resumed: false, duration_ms: expect.any(Number) },
  });
  expect(api.eventsOf("model_download")).toHaveLength(1);
});

test("requests: every model request is a GET for one range, with no cookie, no query and no body", async ({ page, context }) => {
  // What the page and its workers ask any host for, besides what the route of the asset host records.
  const everywhere: string[] = [];
  context.on("request", (request) => everywhere.push(request.url()));
  const { assets } = await openEditor(page);
  await dropClip(page, referenceClip());
  await expect(panel(page)).toBeVisible({ timeout: 30_000 });
  await expect(panel(page)).toBeHidden({ timeout: 180_000 });

  const requests = assets.modelRequests();
  // Every file of the manifest was asked for, and nothing else under the model's folder.
  expect([...new Set(requests.map((request) => request.path))].sort()).toEqual(modelManifest.files.map((file) => file.path).sort());
  expect(assets.requests.filter((request) => request.path.startsWith("models/"))).toEqual(requests);
  // A file of the model is asked of the asset host and of no other: the
  // requests that name one, to whatever host, are the ones recorded there.
  const storedNames = modelManifest.files.map((file) => file.path.slice(file.path.lastIndexOf("/") + 1));
  expect(everywhere.filter((url) => storedNames.some((name) => url.includes(name)))).toEqual(requests.map((request) => request.url));
  for (const request of requests) {
    expect(request.method).toBe("GET");
    expect(request.url).toBe(`${assetBaseUrl()}/${request.path}`);
    expect(request.url).not.toMatch(/[?#]/);
    expect(header(request, "range")).toEqual([{ name: "range", value: expect.stringMatching(/^bytes=\d+-\d+$/) }]);
    expect(header(request, "cookie")).toEqual([]);
    expect(header(request, "authorization")).toEqual([]);
    expect(request.hasBody).toBe(false);
    expect(request.answer).toEqual({ status: 206, bytes: expect.any(Number) });
  }
  // No request asks for more than one part.
  const sizes = requests.map((request) => (typeof request.answer === "object" ? request.answer.bytes : 0));
  expect(Math.max(...sizes)).toBe(PART_BYTES);
  expect(sizes.reduce((sum, bytes) => sum + bytes, 0)).toBe(modelManifest.totalBytes);
});

test("an interrupted download goes on from the bytes it has", async ({ page, context }) => {
  // The first file is small. The second is cut off: once 20 MiB have
  // arrived, which is in its third part, the next request finds no connection.
  const interrupted = manifestFile(1);
  const { assets } = await openEditor(page, { failAfterBytes: 20 * MIB });
  await dropClip(page, referenceClip());
  await expect.poll(() => assets.requests.some((request) => request.answer === "aborted"), { timeout: 120_000 }).toBe(true);
  await page.close();

  const second = await context.newPage();
  const { api, assets: resumed } = await openEditor(second);
  const names = await opfsList(second, MODEL_DIR);
  expect(names).toContain(`${interrupted.localName}.part`);
  expect(names).not.toContain(interrupted.localName);
  const partBytes = await opfsSize(second, `${MODEL_DIR}/${interrupted.localName}.part`);
  // Three parts arrived before the connection broke, and all were kept.
  expect(partBytes).toBe(3 * PART_BYTES);

  await dropClip(second, referenceClip());
  await expect.poll(() => resumed.modelRequests().length, { timeout: 60_000 }).toBeGreaterThan(0);
  const first = resumed.modelRequests()[0];
  expect(first?.path).toBe(interrupted.path);
  expect(rangeStart(first)).toBe(3 * PART_BYTES);
  await expect(feedLines(second).first()).toHaveText(messages.feed.transcribing, { timeout: 240_000 });
  expect(await modelDownloadEvent(second, api)).toEqual({
    name: "model_download",
    props: { outcome: "ok", resumed: true, duration_ms: expect.any(Number) },
  });
});

test("a file with a wrong hash is removed and downloaded again", async ({ page }) => {
  const damaged = manifestFile(0);
  const { api } = await openEditor(page, { corrupt: damaged.path });
  await dropClip(page, referenceClip());

  const copy = messages.errors.E_MODEL_HASH;
  await expect(failure(page)).toContainText(copy.title, { timeout: 120_000 });
  await expect(failure(page)).toContainText(copy.body);
  await expect(failure(page)).toContainText(copy.action);
  // Neither the file nor a part of it is kept (INV-20).
  const names = await opfsList(page, MODEL_DIR);
  expect(names.filter((name) => name.startsWith(damaged.localName))).toEqual([]);
  expect(await modelDownloadEvent(page, api)).toEqual({
    name: "model_download",
    props: { outcome: "hash_mismatch", resumed: false, duration_ms: expect.any(Number) },
  });

  // The right bytes this time.
  const assets = await routeAssets(page);
  await page.getByRole("button", { name: messages.editor.startOver }).click();
  await expect(page.getByTestId("drop-zone")).toBeVisible();
  await dropClip(page, referenceClip());
  await expect.poll(() => assets.modelRequests().length, { timeout: 60_000 }).toBeGreaterThan(0);
  const again = assets.modelRequests().find((request) => request.path === damaged.path);
  expect(rangeStart(again)).toBe(0);
  // The model is ready when the recognizer has loaded it.
  await expect(feedLines(page).first()).toHaveText(messages.feed.transcribing, { timeout: 240_000 });
  expect(await opfsList(page, MODEL_DIR)).toEqual(modelManifest.files.map((_, index) => manifestFile(index).localName).sort());
});

test("a second session downloads nothing and shows no panel", async ({ page, context }) => {
  await ensureModelCached(context);
  const { api, assets } = await openEditor(page);
  await dropClip(page, referenceClip());

  await expect(feedLines(page).first()).toHaveText(messages.feed.transcribing, { timeout: 180_000 });
  expect(assets.modelRequests()).toEqual([]);
  expect(await page.evaluate(() => window.e2ePanelSeen)).toBe(false);
  // Nothing was downloaded, so nothing is reported.
  await flushAnalytics(page);
  await expect.poll(() => api.eventsOf("clip_accepted").length).toBe(1);
  expect(api.eventsOf("model_download")).toEqual([]);
});

test("a download that keeps failing ends with its message, and what arrived is kept", async ({ page }) => {
  // One part of the second file arrives; every request after it is refused.
  const failing = manifestFile(1);
  const { api, assets } = await openEditor(page, { failAfterBytes: PART_BYTES, status: 503 });
  await dropClip(page, referenceClip());
  const refused = () => assets.modelRequests().filter((request) => typeof request.answer === "object" && request.answer.status === 503);
  await expect.poll(() => refused().length, { timeout: 120_000 }).toBeGreaterThan(0);

  // The three waits of 1, 3 and 9 s pass on the page's clock, not on the wall's.
  const copy = messages.errors.E_MODEL_DOWNLOAD;
  await expect(async () => {
    await page.clock.runFor(3_000);
    await expect(failure(page)).toContainText(copy.title, { timeout: 1_000 });
  }).toPass({ timeout: 120_000 });
  await expect(failure(page)).toContainText(copy.action);

  // One request and its three retries, each for the same bytes.
  expect(refused()).toHaveLength(ATTEMPTS);
  expect(new Set(refused().map((request) => `${request.path} ${String(rangeStart(request))}`))).toEqual(
    new Set([`${failing.path} ${String(PART_BYTES)}`]),
  );
  expect(await modelDownloadEvent(page, api)).toEqual({
    name: "model_download",
    props: { outcome: "failed", resumed: false, duration_ms: expect.any(Number) },
  });
  expect(await opfsList(page, MODEL_DIR)).toContain(`${failing.localName}.part`);
});

test("no room for the model: the storage message", async ({ page, context, baseURL }) => {
  // Room for the clip and for a part of the model, not for the model.
  const quota = statSync(referenceClip()).size + 3 * PART_BYTES;
  expect(quota).toBeLessThan(modelManifest.totalBytes);
  const devtools = await context.newCDPSession(page);
  await devtools.send("Storage.overrideQuotaForOrigin", { origin: new URL(baseURL ?? "").origin, quotaSize: quota });

  const { api } = await openEditor(page);
  await dropClip(page, referenceClip());
  const copy = messages.errors.E_MODEL_STORAGE;
  await expect(failure(page)).toContainText(copy.title, { timeout: 180_000 });
  await expect(failure(page)).toContainText(copy.action);
  expect(await modelDownloadEvent(page, api)).toEqual({
    name: "model_download",
    props: { outcome: "failed", resumed: false, duration_ms: expect.any(Number) },
  });
});

test("loading and transcribing: nothing leaves the page but analytics", async ({ page, context, baseURL }) => {
  await ensureModelCached(context);
  const origin = new URL(baseURL ?? "").origin;
  // Everything the page and its workers ask for, from before the app starts.
  const requests: { method: string; url: string }[] = [];
  context.on("request", (request) => requests.push({ method: request.method(), url: request.url() }));
  await installFakeApi(page);
  const assets = await routeAssets(page);
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });

  await dropClip(page, referenceClip());
  await expect(feedLines(page).first()).toHaveText(messages.feed.transcribing, { timeout: 180_000 });
  const transcribingFrom = requests.length;
  await expect(page.getByTestId("preview-canvas")).toBeVisible({ timeout: 240_000 });
  const all = requests.filter((request) => /^https?:/.test(request.url));

  // The recognizer was loaded from this device and from the app's own origin:
  // not from the asset host, and not from the hosts its runtime names as its
  // defaults, which check-hosts.mjs lists (D-38, TE-1).
  expect(assets.modelRequests()).toEqual([]);
  expect(all.filter((request) => new URL(request.url).origin !== origin)).toEqual([]);
  // From the first word of the feed to the player, the page sent analytics
  // and nothing else. What it fetched besides are files of its own build:
  // the worker that draws the preview and the two WASM bundles.
  const during = requests.slice(transcribingFrom).filter((request) => /^https?:/.test(request.url));
  const ownFile = (request: { method: string; url: string }): boolean =>
    request.method === "GET" && /^\/(assets|ort)\//.test(new URL(request.url).pathname);
  const analytics = (request: { method: string; url: string }): boolean =>
    request.method === "POST" && request.url === `${origin}/api/v1/events`;
  expect(during.filter((request) => !ownFile(request) && !analytics(request))).toEqual([]);
  expect(during.filter((request) => new URL(request.url).pathname.startsWith("/api/") && !analytics(request))).toEqual([]);
});
