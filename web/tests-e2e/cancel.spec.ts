// Cancel in a real browser (C-12, v4implementation D-152, §20.4, G 16.4):
// the import, the model's download, the pipeline while it transcribes, an
// export, and a job whose worker does not answer, which is restarted after
// 2 s. Each ends in its scope's state, sends one `job_cancelled { stage }`
// and leaves nothing of the job on the device.
//
// The import and the download take no clip through the pipeline and carry
// `NO_GPU`: the Windows job of CI runs them on the production build (D-72).
// The other three share one clip, in the order written (as `pipeline-preview`
// does), on D1; the last needs the test build's `__offcutDev.hangWorker`.
//
// The render worker's count of open frames (INV-11) is not read from the
// worker (D-155): a frame left open fails the next call that closes the
// frames, as `FrameLeak` (render.worker.ts `closeFrames`), and a failure is
// shown and sent as `client_error`. So after each cancel no `client_error`
// was sent, and where there is a preview a play and a pause draw without a
// failure: the count was back to nothing.

import { type BrowserContext, type Download, expect, type Page, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import { DB_NAME, STORES } from "../src/persistence/schema";
import { type FakeApi, installFakeApi, mintEntitlementToken, seedEntitlement } from "./helpers/fake-api";
import {
  type AssetLog,
  type AssetOptions,
  type AssetRequest,
  NO_GPU,
  dropClip,
  ensureModelCached,
  expectTestBuild,
  genFixture,
  modelManifest,
  opfsList,
  opfsSize,
  patient,
  previewStats,
  referenceClip,
  routeAssets,
} from "./helpers/fixtures";

declare global {
  interface Window {
    /** Set by a case just before the drop: the first Cancel that comes onto the page is pressed at once. */
    e2eCancelOnSight?: boolean;
    /** Whether it was pressed. */
    e2eCancelPressed?: boolean;
  }
}

/** Analytics sends what it has every 10 s; a case waits that long for an event, and a little more. */
const FLUSH_WAIT_MS = 25_000;
/** `CANCEL_TIMEOUT_MS` of workers/pool.ts (D-152): a job that does not answer a cancel by then has its worker restarted. */
const CANCEL_TIMEOUT_MS = 2_000;
const MIB = 1024 * 1024;
/** One request of the downloader asks for at most this much (`MODEL_PART_BYTES`). */
const PART_BYTES = 8 * MIB;
const MODEL_DIR = `models/${modelManifest.modelId}`;
/** The largest `ok_*` fixture, a clip the import takes (90 s, about 5 MB). */
const LARGE_OK = "ok_90s_exact.mp4";

const button = (page: Page, name: string) => page.getByRole("button", { name, exact: true });
const canvas = (page: Page) => page.getByTestId("preview-canvas");
const cancelButton = (page: Page) => page.getByTestId("cancel-job");
const cancelledCard = (page: Page) => page.getByTestId("clip-cancelled");
const previewFailure = (page: Page) => page.getByTestId("preview-failed").or(page.getByText(messages.editor.failedStub));
/** A step of the processing card, in `STEPS`' order (audio, voice, captions, visuals), while it turns. */
const stepRunning = (page: Page, index: number) => page.getByTestId("processing").locator("ol > li").nth(index).locator("svg");
const CAPTIONS_STEP = 2;
const VISUALS_STEP = 3;

/** The name a file of the manifest has on the device. */
function localName(index: number): { path: string; name: string } {
  const file = modelManifest.files[index];
  if (file === undefined) {
    throw new Error(`the manifest has no file ${String(index)}`);
  }
  const stored = file.path.slice(file.path.lastIndexOf("/") + 1);
  return { path: file.path, name: stored.replace(/\.[0-9a-f]{16}(\.[^.]+)$/, "$1") };
}

/** The first byte a model request asks for. */
function rangeStart(request: AssetRequest | undefined): number {
  const range = request?.headers.find((header) => header.name === "range")?.value;
  const start = /^bytes=(\d+)-\d+$/.exec(range ?? "")?.[1];
  return start === undefined ? Number.NaN : Number(start);
}

/** Presses the first Cancel to come onto the page once `armCancel` was called: as early as a person could. */
async function watchCancel(page: Page): Promise<void> {
  await page.addInitScript(() => {
    window.e2eCancelOnSight = false;
    window.e2eCancelPressed = false;
    new MutationObserver(() => {
      const cancel = document.querySelector('[data-testid="cancel-job"]');
      if (window.e2eCancelOnSight === true && !window.e2eCancelPressed && cancel instanceof HTMLButtonElement) {
        window.e2eCancelPressed = true;
        cancel.click();
      }
    }).observe(document, { childList: true, subtree: true });
  });
}

async function armCancel(page: Page): Promise<void> {
  await page.evaluate(() => {
    window.e2eCancelOnSight = true;
  });
}

/** Opens the editor with a fake API and the asset host answered from this machine. */
async function openEditor(page: Page, assets: AssetOptions = {}): Promise<{ api: FakeApi; assets: AssetLog }> {
  const api = await installFakeApi(page);
  const log = await routeAssets(page, assets);
  await watchCancel(page);
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  return { api, assets: log };
}

/** The `job_cancelled` events once `count` of them were sent, by their stage. */
async function cancelledStages(api: FakeApi, count: number): Promise<string[]> {
  await expect.poll(() => api.eventsOf("job_cancelled").length, { timeout: FLUSH_WAIT_MS }).toBe(count);
  return api.eventsOf("job_cancelled").map((event) => (event as { props: { stage: string } }).props.stage);
}

/** How many records the render cache holds (`STORES.renderCache`), 0 when the database was not made. */
function renderCacheCount(page: Page): Promise<number> {
  return page.evaluate(
    ({ name, store }) =>
      new Promise<number>((resolve) => {
        const open = indexedDB.open(name);
        open.onerror = () => {
          resolve(0);
        };
        open.onsuccess = () => {
          const db = open.result;
          if (!db.objectStoreNames.contains(store)) {
            db.close();
            resolve(0);
            return;
          }
          const count = db.transaction(store).objectStore(store).count();
          count.onsuccess = () => {
            db.close();
            resolve(count.result);
          };
        };
      }),
    { name: DB_NAME, store: STORES.renderCache },
  );
}

/**
 * INV-11 after a cancel (see the top of this file): nothing failed, and when
 * the clip has a preview, a play and a pause draw and give their frames back.
 */
async function framesGivenBack(page: Page, api: FakeApi, preview: boolean): Promise<void> {
  if (preview) {
    await expect(button(page, messages.preview.play)).toBeEnabled({ timeout: 30_000 });
    await button(page, messages.preview.play).click();
    await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 30_000 });
    await page.waitForTimeout(1_500);
    await button(page, messages.preview.pause).click();
    await expect(button(page, messages.preview.play)).toBeVisible({ timeout: 30_000 });
    await expect(previewFailure(page)).toHaveCount(0);
    expect((await previewStats(page)).frames).toBeGreaterThan(0);
  }
  await expect(page.getByTestId("clip-failed")).toHaveCount(0);
  // Longer than analytics waits to send: a failure would have been sent by now.
  await page.waitForTimeout(11_000);
  expect(api.eventsOf("client_error")).toEqual([]);
}

test("import cancel: a large clip's import stopped at once leaves no clip, nothing on the device, and one job_cancelled", { tag: NO_GPU }, async ({ page }) => {
  const { api } = await openEditor(page);
  await armCancel(page);
  await dropClip(page, genFixture(LARGE_OK));

  await expect.poll(() => page.evaluate(() => window.e2eCancelPressed), { timeout: 30_000 }).toBe(true);
  // `idle`: the drop zone again, and no card of the clip.
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  await expect(page.getByTestId("processing")).toHaveCount(0);
  await expect(cancelledCard(page)).toHaveCount(0);
  await expect.poll(() => opfsList(page, "clips"), { timeout: 30_000 }).toEqual([]);
  expect(await cancelledStages(api, 1)).toEqual(["import"]);
  expect(api.eventsOf("clip_accepted")).toEqual([]);
  await framesGivenBack(page, api, false);
});

test("model cancel: the download stopped keeps its bytes, and the next run goes on from them", { tag: NO_GPU }, async ({ page }) => {
  // The first file is small; the second is held once 20 MiB have arrived, in its fourth part.
  const held = localName(1);
  const { api, assets } = await openEditor(page, { holdAfterBytes: 20 * MIB });
  await dropClip(page, referenceClip());
  await expect.poll(() => assets.modelRequests().some((request) => request.answer === "held"), { timeout: patient(120_000) }).toBe(true);
  await page.getByTestId("model-download").getByTestId("cancel-job").click();

  // The run waited for the model: it is cancelled with it, and the clip is `accepted`.
  await expect(cancelledCard(page)).toBeVisible({ timeout: 30_000 });
  await expect(page.getByTestId("model-download")).toHaveCount(0);
  const names = await opfsList(page, MODEL_DIR);
  expect(names).toContain(`${held.name}.part`);
  expect(names).not.toContain(held.name);
  // The three parts that arrived were kept (D-152).
  expect(await opfsSize(page, `${MODEL_DIR}/${held.name}.part`)).toBe(3 * PART_BYTES);
  expect(await cancelledStages(api, 1)).toEqual(["model"]);

  // "Process again", with every request answered: the download starts where it stopped.
  const resumed = await routeAssets(page);
  await button(page, messages.cancel.processAgain).click();
  await expect.poll(() => resumed.modelRequests().length, { timeout: 60_000 }).toBeGreaterThan(0);
  const first = resumed.modelRequests()[0];
  expect(first?.path).toBe(held.path);
  expect(rangeStart(first)).toBe(3 * PART_BYTES);
  await expect(page.getByTestId("model-download")).toHaveCount(0, { timeout: patient(240_000) });
  // A cancelled download is not reported (run-pipeline.ts); the finished one is, as resumed.
  await expect.poll(() => api.eventsOf("model_download"), { timeout: FLUSH_WAIT_MS }).toEqual([
    { name: "model_download", props: { outcome: "ok", resumed: true, duration_ms: expect.any(Number) } },
  ]);
  await framesGivenBack(page, api, false);
});

test.describe("one clip, cancelled three ways", () => {
  test.describe.configure({ mode: "serial" });

  let context: BrowserContext;
  let page: Page;
  let api: FakeApi;
  const downloads: Download[] = [];

  test.beforeAll(async ({ browser }, testInfo) => {
    const { baseURL } = testInfo.project.use;
    context = await browser.newContext({ acceptDownloads: true, ...(baseURL === undefined ? {} : { baseURL }) });
    await ensureModelCached(context);
    page = await context.newPage();
    page.on("download", (download) => downloads.push(download));
    ({ api } = await openEditor(page));
  });

  test.afterAll(async () => {
    await context.close();
  });

  test("pipeline cancel during ASR: the clip is accepted, and Process again takes it to the preview", async () => {
    test.setTimeout(patient(600_000));
    await dropClip(page, referenceClip());
    await expect(stepRunning(page, CAPTIONS_STEP)).toHaveCount(1, { timeout: patient(120_000) });
    await cancelButton(page).click();

    await expect(cancelledCard(page)).toBeVisible({ timeout: 30_000 });
    await expect(cancelledCard(page)).toContainText(messages.cancel.cancelled);
    expect(await cancelledStages(api, 1)).toEqual(["asr"]);
    expect(api.eventsOf("pipeline_done")).toEqual([]);

    await button(page, messages.cancel.processAgain).click();
    await expect(canvas(page).or(page.getByTestId("clip-failed"))).toBeVisible({ timeout: patient(240_000) });
    await expect(canvas(page), "the clip was processed again").toBeVisible();
    await expect.poll(() => api.eventsOf("pipeline_done").length, { timeout: FLUSH_WAIT_MS }).toBe(1);
    await framesGivenBack(page, api, true);
  });

  test("export cancel: the export goes back to idle, with no file, no temporary file and no cache entry", async () => {
    test.setTimeout(patient(600_000));
    await seedEntitlement(page, mintEntitlementToken());
    await button(page, messages.export.button).click();
    const progress = page.getByTestId("export-progress");
    await expect(progress).toBeVisible({ timeout: 60_000 });
    // Some frames are rendered: the cancel stops a running encode.
    await expect.poll(async () => Number(await progress.getByRole("progressbar").getAttribute("aria-valuenow")), { timeout: patient(120_000) }).toBeGreaterThanOrEqual(3);
    await page.getByTestId("export-cancel").click();

    await expect(progress).toHaveCount(0, { timeout: 30_000 });
    await expect(page.getByTestId("export-failed")).toHaveCount(0);
    await expect(page.getByTestId("export-done")).toHaveCount(0);
    await expect(button(page, messages.export.button)).toBeEnabled();
    expect(downloads).toEqual([]);
    expect(await opfsList(page, "exports/tmp")).toEqual([]);
    expect((await opfsList(page, "exports")).filter((name) => name !== "tmp")).toEqual([]);
    expect(await renderCacheCount(page)).toBe(0);
    expect(await cancelledStages(api, 2)).toEqual(["asr", "render_encode"]);
    expect(api.eventsOf("export_done")).toEqual([]);
    await framesGivenBack(page, api, true);
  });

  test("cancel timeout: a render worker that does not answer is restarted after 2 s; the clip is accepted and processes again", async () => {
    test.setTimeout(patient(600_000));
    await button(page, messages.editor.startOver).click();
    await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
    await expectTestBuild(page);
    // The render worker takes no message from now on: the run's first call to it (detect_scene) is never answered.
    await page.evaluate(() => {
      (window as unknown as { __offcutDev: { hangWorker(kind: string): void } }).__offcutDev.hangWorker("render");
    });
    await dropClip(page, referenceClip());
    await expect(stepRunning(page, VISUALS_STEP)).toHaveCount(1, { timeout: patient(240_000) });

    const pressed = Date.now();
    await cancelButton(page).click();
    await expect(cancelledCard(page)).toBeVisible({ timeout: 30_000 });
    const tookMs = Date.now() - pressed;
    process.stdout.write(`      the cancel of the hung render worker took ${String(tookMs)} ms\n`);
    // Not before the 2 s the job had to answer, and not much after.
    expect(tookMs).toBeGreaterThanOrEqual(CANCEL_TIMEOUT_MS);
    expect(tookMs).toBeLessThan(CANCEL_TIMEOUT_MS + 8_000);
    expect(await cancelledStages(api, 3)).toEqual(["asr", "render_encode", "detect_scene"]);

    // The fresh worker takes the clip to its preview.
    await button(page, messages.cancel.processAgain).click();
    await expect(canvas(page).or(page.getByTestId("clip-failed"))).toBeVisible({ timeout: patient(240_000) });
    await expect(canvas(page), "the clip was processed by the restarted worker").toBeVisible();
    await framesGivenBack(page, api, true);
  });
});
