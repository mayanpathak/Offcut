// J9 and J10 in a real browser: the reference clip is exported and the file
// is checked by the independent verifier, against the built app served with
// the production headers. The build under test must have been made with the
// test key (v2implementation D-24): a token is one the tests mint.
//
// The clip is processed once, and the cases export it in the order they are
// written: with no token, with a token of another key, with a Creator token,
// with a Free token. A case that fails stops the ones after it.

import { mkdtempSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { type BrowserContext, type Download, expect, type Page, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import { type FakeApi, installFakeApi, mintEntitlementToken, seedEntitlement } from "./helpers/fake-api";
import { dropClip, ensureModelCached, opfsList, referenceClip, routeAssets, sourceDurationMs, verifyMp4 } from "./helpers/fixtures";

declare global {
  interface Window {
    /** The stage labels the export's progress showed, in order, each once. */
    e2eStages?: string[];
    /** Every value its bar took, in order. */
    e2eExportBar?: string[];
    /** How often the progress was on the page without the "rendering on your computer" line. */
    e2eLocalLineMissing?: number;
    /** How many times the failure of an export came onto the page. */
    e2eExportFailures?: number;
  }
}

const UUID_MP4 = /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}\.mp4$/;
/** Analytics sends what it has every 10 s; a case waits that long for an event, and a little more. */
const FLUSH_WAIT_MS = 20_000;
const EXPORT_TIMEOUT_MS = 240_000;
/** A seed that is not the test key's: a token signed with it must be refused. */
const OTHER_SEED = new Uint8Array(32).fill(7);

type Recorded = { method: string; url: string };

const canvas = (page: Page) => page.getByTestId("preview-canvas");
const button = (page: Page, name: string) => page.getByRole("button", { name, exact: true });
const exportButton = (page: Page) => button(page, messages.export.button);

/** Records what the export's progress shows, from before the page loads: "Saving" is on it for a moment. */
async function watchExport(page: Page): Promise<void> {
  await page.addInitScript((localLine) => {
    window.e2eStages = [];
    window.e2eExportBar = [];
    window.e2eLocalLineMissing = 0;
    window.e2eExportFailures = 0;
    let failureShown = false;
    new MutationObserver(() => {
      // The failure of the export before stays on the page until the next one starts: a new one is one that comes.
      const failure = document.querySelector('[data-testid="export-failed"]') !== null;
      if (failure && !failureShown) {
        window.e2eExportFailures = (window.e2eExportFailures ?? 0) + 1;
      }
      failureShown = failure;
      const progress = document.querySelector('[data-testid="export-progress"]');
      if (progress === null) {
        return;
      }
      const stage = progress.querySelector('[data-testid="export-stage"]')?.textContent ?? "";
      if (stage !== window.e2eStages?.at(-1)) {
        window.e2eStages?.push(stage);
      }
      const now = progress.querySelector('[role="progressbar"]')?.getAttribute("aria-valuenow") ?? "";
      if (now !== window.e2eExportBar?.at(-1)) {
        window.e2eExportBar?.push(now);
      }
      if (!(progress.textContent ?? "").includes(localLine)) {
        window.e2eLocalLineMissing = (window.e2eLocalLineMissing ?? 0) + 1;
      }
    }).observe(document, { childList: true, subtree: true, characterData: true, attributes: true });
  }, messages.export.renderingLocally);
}

async function forgetProgress(page: Page): Promise<void> {
  await page.evaluate(() => {
    window.e2eStages = [];
    window.e2eExportBar = [];
    window.e2eLocalLineMissing = 0;
  });
}

test.describe("one clip, exported", () => {
  test.describe.configure({ mode: "serial" });

  let context: BrowserContext;
  let page: Page;
  let api: FakeApi;
  let downloads: Download[];
  let scratch: string;
  const durationMs = sourceDurationMs(referenceClip());
  /** Everything the page and its workers asked for. */
  const requests: Recorded[] = [];
  /** What the Creator export left for the cases that follow it. */
  let creator: { name: string; file: string; stored: string; requests: Recorded[]; startOverDisabled: boolean; playDisabled: boolean };

  /** Presses Export and waits for the file the browser is handed, saved where no report picks it up. */
  async function exportAndSave(name: string): Promise<{ download: Download; file: string; during: Recorded[] }> {
    const from = requests.length;
    const failedBefore = api.eventsOf("export_failed").length;
    const failuresBefore = await page.evaluate(() => window.e2eExportFailures ?? 0);
    const arriving = page.waitForEvent("download", { timeout: EXPORT_TIMEOUT_MS });
    const failing = page.waitForFunction((before) => (window.e2eExportFailures ?? 0) > before, failuresBefore, { timeout: EXPORT_TIMEOUT_MS });
    // One of the two ends the wait; the other is let go when the page closes.
    void arriving.catch(() => undefined);
    void failing.catch(() => undefined);
    await exportButton(page).click();
    const download = await Promise.race([arriving, failing.then(() => null)]);
    if (download === null) {
      // Say why, as far as the page tells: the code of the failure is in its event.
      await expect.poll(() => api.eventsOf("export_failed").length, { timeout: FLUSH_WAIT_MS }).toBeGreaterThan(failedBefore);
      throw new Error(`the export failed: ${JSON.stringify(api.eventsOf("export_failed").at(-1))}`);
    }
    const file = path.join(scratch, name);
    await download.saveAs(file);
    await expect(page.getByTestId("export-done")).toHaveText(messages.export.done, { timeout: 30_000 });
    return { download, file, during: requests.slice(from).filter((request) => /^https?:/.test(request.url)) };
  }

  test.beforeAll(async ({ browser }, testInfo) => {
    // Outside the test's own directory: an exported file is never part of a report (§22.5).
    scratch = mkdtempSync(path.join(tmpdir(), "offcut-e2e-"));
    const { baseURL } = testInfo.project.use;
    context = await browser.newContext({ acceptDownloads: true, ...(baseURL === undefined ? {} : { baseURL }) });
    await ensureModelCached(context);
    context.on("request", (request) => requests.push({ method: request.method(), url: request.url() }));
    page = await context.newPage();
    downloads = [];
    page.on("download", (download) => downloads.push(download));
    api = await installFakeApi(page);
    await routeAssets(page);
    await watchExport(page);
    await page.goto("/app");
    await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
    await dropClip(page, referenceClip());
    await expect(canvas(page)).toBeVisible({ timeout: 240_000 });
    await expect(exportButton(page)).toBeEnabled();
  });

  test.afterAll(async () => {
    await context.close();
    rmSync(scratch, { recursive: true, force: true });
  });

  test("no token: the button says exporting needs an account, and nothing is exported", async () => {
    await exportButton(page).click();
    await expect(page.getByTestId("export-status")).toHaveText(messages.export.unavailable, { timeout: 30_000 });
    await expect(page.getByTestId("export-progress")).toHaveCount(0);
    // Longer than analytics waits to send: an `export_started` would have arrived.
    await page.waitForTimeout(11_000);
    expect(downloads).toEqual([]);
    expect(api.eventsOf("export_started")).toEqual([]);
    expect(await opfsList(page, "exports")).toEqual([]);
    // The button can be pressed again: nothing was begun.
    await expect(exportButton(page)).toBeEnabled();
  });

  test("a token signed with another key: the export fails and leaves no file", async () => {
    await seedEntitlement(page, mintEntitlementToken({ seed: OTHER_SEED }));
    await exportButton(page).click();
    await expect(page.getByTestId("export-failed")).toHaveText(messages.editor.failedStub, { timeout: 60_000 });
    await expect.poll(() => api.eventsOf("export_failed"), { timeout: FLUSH_WAIT_MS }).toEqual([
      { name: "export_failed", props: { error_code: "E_ENTITLEMENT_INVALID", stage: "render_encode" } },
    ]);
    expect(downloads).toEqual([]);
    expect(await opfsList(page, "exports")).toEqual([]);
    // The preview is given back, and the clip can be exported again.
    await expect(button(page, messages.preview.play)).toBeEnabled({ timeout: 30_000 });
    await expect(exportButton(page)).toBeEnabled();
  });

  test("Creator export: the downloaded file passes the verifier", async () => {
    await seedEntitlement(page, mintEntitlementToken());
    await forgetProgress(page);
    // While the export runs, the preview and "start over" wait for it.
    const locked = page
      .getByTestId("export-progress")
      .waitFor({ state: "visible", timeout: 60_000 })
      .then(async () => ({
        startOverDisabled: await button(page, messages.editor.startOver).isDisabled(),
        playDisabled: await button(page, messages.preview.play).isDisabled(),
      }))
      // An export that ends before its progress shows has nothing to read here; the case fails where it says why.
      .catch(() => ({ startOverDisabled: false, playDisabled: false }));
    const { download, file, during } = await exportAndSave("creator.mp4");
    creator = { name: download.suggestedFilename(), file, stored: "", requests: during, ...(await locked) };

    expect(statSync(file).size).toBeGreaterThan(1_000_000);
    const verdict = verifyMp4(file, "creator", durationMs);
    expect(verdict.output).toContain("PASS 6");
    expect(verdict.output).not.toContain("FAIL");
    expect(verdict.status, verdict.output).toBe(0);
  });

  test("progress: the stages in order, the bar, and the line that says where the work is done", async () => {
    const stages = await page.evaluate(() => window.e2eStages ?? []);
    const { rendering, muxing, saving } = messages.export.stage;
    // "Saving" may pass between two looks at the page; the other two are always seen.
    expect([[rendering, muxing], [rendering, muxing, saving]]).toContainEqual(stages);
    const bar = (await page.evaluate(() => window.e2eExportBar ?? [])).map(Number);
    expect(bar.length).toBeGreaterThan(10);
    expect(bar.at(-1)).toBe(100);
    expect(bar.every((value, index) => index === 0 || value >= (bar[index - 1] ?? 0))).toBe(true);
    expect(await page.evaluate(() => window.e2eLocalLineMissing)).toBe(0);
    expect(creator.playDisabled).toBe(true);
    expect(creator.startOverDisabled).toBe(true);
    await expect(button(page, messages.editor.startOver)).toBeEnabled();
  });

  test("events: the export is announced, timed in its two stages, and reported done", async () => {
    await expect.poll(() => api.eventsOf("export_done").length, { timeout: FLUSH_WAIT_MS }).toBe(1);
    // The refused export was announced too, as a Creator's: the plan is read from the token as it is (D-52).
    expect(api.eventsOf("export_started")).toEqual([
      { name: "export_started", props: { profile: "creator" } },
      { name: "export_started", props: { profile: "creator" } },
    ]);
    const stages = (api.eventsOf("stage_timing") as { props: { stage: string; duration_ms: number } }[]).map((event) => event.props);
    expect(stages.filter((props) => props.stage === "render_encode" || props.stage === "mux")).toEqual([
      { stage: "render_encode", duration_ms: expect.any(Number) },
      { stage: "mux", duration_ms: expect.any(Number) },
    ]);
    const done = api.eventsOf("export_done")[0] as { props: Record<string, unknown> };
    expect(done.props).toEqual({
      total_ms: expect.any(Number),
      profile: "creator",
      word_edits: 0,
      events_kept: expect.any(Number),
      events_disabled: 0,
      style: "clean",
      crop_adjusted: false,
      from_cache: false,
    });
    expect(done.props.events_kept).toBeGreaterThanOrEqual(1);
    expect(api.eventsOf("export_failed")).toHaveLength(1);
  });

  test("download name: offcut, the day and the minute, and nothing of the source", () => {
    expect(creator.name).toMatch(/^offcut-\d{8}-\d{4}\.mp4$/);
    expect(downloads.map((download) => download.suggestedFilename())).toEqual([creator.name]);
  });

  test("storage: the device keeps the one export, and nothing unfinished", async () => {
    const names = await opfsList(page, "exports");
    expect(names).toEqual([expect.stringMatching(UUID_MP4), "tmp"]);
    expect(await opfsList(page, "exports/tmp")).toEqual([]);
    creator.stored = names[0] ?? "";
  });

  test("quiet export: while it runs the page sends analytics and nothing else", ({ baseURL }) => {
    const events = `${new URL(baseURL ?? "").origin}/api/v1/events`;
    expect(creator.requests.filter((request) => !(request.method === "POST" && request.url === events))).toEqual([]);
  });

  test("preview after export: the player plays again", async () => {
    await expect(button(page, messages.preview.play)).toBeEnabled({ timeout: 30_000 });
    await button(page, messages.preview.play).click();
    await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 30_000 });
    await page.waitForTimeout(2_500);
    const first = await canvas(page).screenshot();
    await page.waitForTimeout(1_000);
    expect(first.equals(await canvas(page).screenshot())).toBe(false);
    await button(page, messages.preview.pause).click();
    await expect(button(page, messages.preview.play)).toBeVisible({ timeout: 30_000 });
    await expect(page.getByText(messages.editor.failedStub)).toHaveCount(0);
  });

  test("Free-plan token: the file is 720 by 1280, as the token says", async () => {
    await seedEntitlement(page, mintEntitlementToken({ plan: "free" }));
    const { file } = await exportAndSave("free.mp4");
    const verdict = verifyMp4(file, "free", durationMs);
    expect(verdict.output).not.toContain("FAIL");
    expect(verdict.status, verdict.output).toBe(0);
    // The same file is not a Creator's: its size is the one thing that differs (INV-9).
    const asCreator = verifyMp4(file, "creator", durationMs);
    expect(asCreator.status).toBe(1);
    expect(asCreator.output).toContain("FAIL 2");
    await expect.poll(() => api.eventsOf("export_started").length, { timeout: FLUSH_WAIT_MS }).toBe(3);
    expect(api.eventsOf("export_started").at(-1)).toEqual({ name: "export_started", props: { profile: "free" } });
  });

  test("second export: the device holds the new file alone, under a new id", async () => {
    const names = await opfsList(page, "exports");
    expect(names).toEqual([expect.stringMatching(UUID_MP4), "tmp"]);
    expect(names[0]).not.toBe(creator.stored);
    expect(await opfsList(page, "exports/tmp")).toEqual([]);
    expect(downloads).toHaveLength(2);
  });
});
