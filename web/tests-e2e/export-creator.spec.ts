// J9 and J10 in a real browser: the reference clip is exported and the file
// is put through all 11 checks of the independent verifier, against the built
// app served with the production headers. The build under test must have been made with the
// test key (v2implementation D-24): a token is one the tests mint.
//
// The clip is processed once, and the cases export it in the order they are
// written: with no token, with a token of another key, with a Creator token,
// with a Free token, then what the render cache does with what is stored
// (v5implementation D-220, §17.5). A case that fails stops the ones after it. The case of
// the founder's second recording runs on its own after them, on the
// development machine only (D-99, D-100).

import { mkdtempSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { type BrowserContext, type Download, expect, type Page, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import { type FakeApi, installFakeApi, mintEntitlementToken, seedEntitlement } from "./helpers/fake-api";
import {
  axe,
  dropClip,
  ensureModelCached,
  expectAllChecks,
  forgetProgress,
  opfsList,
  patient,
  referenceClip,
  routeAssets,
  sendQueuedAnalytics,
  sha256Of,
  sourceDurationMs,
  v3TestClip,
  verifyMp4,
  watchExport,
} from "./helpers/fixtures";

const UUID_MP4 = /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}\.mp4$/;
/** Analytics sends what it has every 10 s; a case waits that long for an event, and a little more. */
const FLUSH_WAIT_MS = 20_000;
const EXPORT_TIMEOUT_MS = patient(240_000);
/** A seed that is not the test key's: a token signed with it must be refused. */
const OTHER_SEED = new Uint8Array(32).fill(7);
/** The white frames of the reference clip, which check 8 compares (read in P130). */
const REFERENCE_FLASHES = 2;
/** A hit reads one row and hands over one file (TS §21.4, "instantly"). */
const HIT_LIMIT_MS = 2_000;

type Recorded = { method: string; url: string };

const canvas = (page: Page) => page.getByTestId("preview-canvas");
const button = (page: Page, name: string) => page.getByRole("button", { name, exact: true });
const exportButton = (page: Page) => button(page, messages.export.button);

test.describe("one clip, exported", () => {
  test.describe.configure({ mode: "serial" });

  let context: BrowserContext;
  let page: Page;
  let api: FakeApi;
  let downloads: Download[];
  let scratch: string;
  // Read when the group starts, not when the file is loaded: on a machine
  // without `testclips/` the clip is there only once the setup project ran.
  let durationMs: number;
  /** Everything the page and its workers asked for. */
  const requests: Recorded[] = [];
  /** What the Creator export left for the cases that follow it. */
  let creator: { name: string; file: string; stored: string; requests: Recorded[]; startOverDisabled: boolean; playDisabled: boolean };
  /** The Free export: the render cache's hits must be these bytes. */
  let freeFile: string;

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
    durationMs = sourceDurationMs(referenceClip());
    await dropClip(page, referenceClip());
    // The player, or the page that says the clip could not be processed: that one ends the wait at once.
    await expect(canvas(page).or(page.getByTestId("clip-failed")).or(page.getByTestId("clip-rejected"))).toBeVisible({ timeout: patient(240_000) });
    await expect(canvas(page), "the clip was processed").toBeVisible();
    await expect(exportButton(page)).toBeEnabled();
  });

  test.afterAll(async () => {
    await context.close();
    rmSync(scratch, { recursive: true, force: true });
  });

  test("no token: the sign-in dialog opens with the limits, and nothing is exported", async () => {
    await exportButton(page).click();
    await expect(page.getByTestId("sign-in-dialog").getByTestId("sign-in-limits")).toHaveText(/720p.*1080p/, { timeout: 30_000 });
    await expect(page.getByTestId("export-progress")).toHaveCount(0);
    // Longer than analytics waits to send: an `export_started` would have arrived.
    await page.waitForTimeout(11_000);
    expect(downloads).toEqual([]);
    expect(api.eventsOf("export_started")).toEqual([]);
    expect(await opfsList(page, "exports")).toEqual([]);
    await page.keyboard.press("Escape");
    // The button can be pressed again: nothing was begun.
    await expect(exportButton(page)).toBeEnabled();
  });

  test("a token signed with another key: the export fails and leaves no file", async () => {
    await seedEntitlement(page, mintEntitlementToken({ seed: OTHER_SEED }));
    await exportButton(page).click();
    // The card of a code with no words yet (V6) says the general sentence (D-190).
    await expect(page.getByTestId("export-failed")).toContainText(messages.editor.failedStub, { timeout: 60_000 });
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
    expectAllChecks(verifyMp4(file, "creator", durationMs, referenceClip()), { flashes: REFERENCE_FLASHES });
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

  test("events: the export is announced, timed in its two stages, and reported done", async ({}, testInfo) => {
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
    // How long the machine took is its own to say, and is written down.
    const timingLine = `the Creator export took: ${stages.map((props) => `${props.stage} ${String(props.duration_ms)} ms`).join(", ")}`;
    testInfo.annotations.push({ type: "export time", description: timingLine });
    process.stdout.write(`      ${timingLine}\n`);
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
    await expect(page.getByText(messages.editor.failedStub).or(page.getByTestId("preview-failed"))).toHaveCount(0);
  });

  test("Free-plan token: the file is 720 by 1280, as the token says", async () => {
    await seedEntitlement(page, mintEntitlementToken({ plan: "free" }));
    const { file } = await exportAndSave("free.mp4");
    freeFile = file;
    expectAllChecks(verifyMp4(file, "free", durationMs, referenceClip()), { flashes: REFERENCE_FLASHES });
    // The same file is not a Creator's: its size and its watermark say so (INV-9).
    const asCreator = verifyMp4(file, "creator", durationMs, referenceClip());
    expect(asCreator.status).toBe(1);
    expect(asCreator.output).toContain("FAIL 2");
    expect(asCreator.output).toContain("FAIL 9");
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

  test("summary and the question: what was changed, only that, and one answer gives one event", async () => {
    const outcome = page.getByTestId("export-outcome");
    await expect(outcome.getByRole("heading", { name: messages.export.summary.title })).toBeVisible();
    const rows = await outcome.getByRole("listitem").allTextContents();
    expect(rows[0], "the chain always runs").toBe(messages.export.summary.voice);
    // Each row names a number from one up: a zero is left out.
    for (const row of rows.slice(1)) {
      expect(row).toMatch(/^[1-9]\d* (words? emphasized|visual moments?)$/);
    }
    // The block is said once by the one live region, and takes no focus (D-192).
    await expect(page.locator('[aria-live="polite"]')).toContainText(messages.export.question.title);
    await expect(outcome.locator(":focus")).toHaveCount(0);
    expect((await axe(page)).map((v) => `${v.id}: ${String(v.nodes.length)} node(s)`), "axe on the done state").toEqual([]);

    await expect(outcome.getByText(messages.export.question.title)).toBeVisible();
    await button(page, messages.export.question.smallEdits).click();
    await expect(page.getByTestId("post-export-thanks")).toHaveText(messages.export.question.thanks);
    await expect(button(page, messages.export.question.smallEdits)).toHaveCount(0);
    await sendQueuedAnalytics(page);
    expect(api.eventsOf("post_export_answer")).toEqual([{ name: "post_export_answer", props: { answer: "small_edits" } }]);
  });

  test("same token: the stored file, at once, with nothing rendered", async () => {
    await sendQueuedAnalytics(page);
    const startedBefore = api.eventsOf("export_started").length;
    const timingsBefore = api.eventsOf("stage_timing").length;
    const stored = await opfsList(page, "exports");
    await forgetProgress(page);
    const { download, file } = await exportAndSave("free-again.mp4");
    await sendQueuedAnalytics(page);
    const done = api.eventsOf("export_done").at(-1) as { props: { total_ms: number } };
    expect(done.props).toMatchObject({ profile: "free", from_cache: true });
    expect(done.props.total_ms, "instant").toBeLessThan(HIT_LIMIT_MS);
    expect(api.eventsOf("export_started"), "no export_started").toHaveLength(startedBefore);
    expect(api.eventsOf("stage_timing"), "no stage_timing").toHaveLength(timingsBefore);
    const bar = (await page.evaluate(() => window.e2eExportBar ?? [])).map(Number);
    expect(bar.every((value) => value === 0), "no frame was rendered").toBe(true);
    expect(await sha256Of(file), "the Free file's bytes").toBe(await sha256Of(freeFile));
    expect(await opfsList(page, "exports"), "the one file").toEqual(stored);
    expect(download.suggestedFilename()).toMatch(/^offcut-\d{8}-\d{4}\.mp4$/);
    // It shares its export's id: the question stays answered, and no answer is sent again.
    await expect(page.getByTestId("post-export-thanks")).toBeVisible();
    expect(api.eventsOf("post_export_answer")).toHaveLength(1);
  });

  test("forged token: refused before any lookup; no file given, and the stored one stays", async () => {
    const stored = await opfsList(page, "exports");
    const downloadsBefore = downloads.length;
    await seedEntitlement(page, mintEntitlementToken({ plan: "free", seed: OTHER_SEED }));
    await exportButton(page).click();
    await expect(page.getByTestId("export-failed")).toContainText(messages.editor.failedStub, { timeout: 60_000 });
    await sendQueuedAnalytics(page);
    expect(api.eventsOf("export_failed").at(-1)).toEqual({ name: "export_failed", props: { error_code: "E_ENTITLEMENT_INVALID", stage: "render_encode" } });
    expect(downloads).toHaveLength(downloadsBefore);
    expect(await opfsList(page, "exports")).toEqual(stored);
  });

  test("no room: the export says why, sweeps nothing, and the button stays ready", async () => {
    const stored = await opfsList(page, "exports");
    await sendQueuedAnalytics(page);
    const startedBefore = api.eventsOf("export_started").length;
    await page.evaluate(() => {
      Object.defineProperty(navigator.storage, "estimate", { value: () => Promise.resolve({ quota: 1_000, usage: 1_000 }), configurable: true });
    });
    try {
      // A Creator token: a miss on the Free file that is stored.
      await seedEntitlement(page, mintEntitlementToken());
      await exportButton(page).click();
      await expect(page.getByTestId("export-status")).toHaveText(messages.blockers.B_STORAGE_LOW, { timeout: 30_000 });
    } finally {
      await page.evaluate(() => {
        delete (navigator.storage as unknown as { estimate?: unknown }).estimate;
      });
    }
    await expect(page.getByTestId("export-progress")).toHaveCount(0);
    await expect(page.getByTestId("export-failed")).toHaveCount(0);
    await expect(exportButton(page)).toBeEnabled();
    expect(await opfsList(page, "exports"), "nothing swept").toEqual(stored);
    await sendQueuedAnalytics(page);
    expect(api.eventsOf("export_started"), "nothing began").toHaveLength(startedBefore);
    await expect(button(page, messages.preview.play), "the preview is given back").toBeEnabled({ timeout: 30_000 });
  });

  test("expired Creator token: verified as Free, it hits the stored Free file, watermark and all (INV-9)", async () => {
    await seedEntitlement(page, mintEntitlementToken({ periodEnd: Math.floor(Date.now() / 1000) - 3_600 }));
    const { file } = await exportAndSave("expired.mp4");
    await sendQueuedAnalytics(page);
    expect((api.eventsOf("export_done").at(-1) as { props: unknown }).props).toMatchObject({ profile: "free", from_cache: true });
    expect(await sha256Of(file)).toBe(await sha256Of(freeFile));
    // 720 x 1280 with the watermark, by the verifier: what the token entitles it to, and not more.
    expectAllChecks(verifyMp4(file, "free", durationMs, referenceClip()), { flashes: REFERENCE_FLASHES });
    await expect(page.getByTestId("export-status")).toHaveText("");
  });

  test("close: the outcome goes, and the tabs under it stay", async () => {
    await page.getByTestId("export-outcome").getByRole("button", { name: messages.export.summary.close, exact: true }).click();
    await expect(page.getByTestId("export-outcome")).toHaveCount(0);
    await expect(page.getByTestId("export-done")).toHaveText(messages.export.done);
    await expect(page.getByRole("tab", { name: messages.panel.captions })).toBeEnabled();
  });
});

// The founder's second recording carries a location tag (D-99, D-100): its
// export must carry none of its metadata. On the development machine only.
// The file lives in the system's temp directory and is deleted however the
// case ends; no picture or trace of it is kept (V2 CL item 41).
test("no metadata: an export of the second recording carries none of the source's", async ({ browser }, testInfo) => {
  const clip = v3TestClip();
  test.skip(clip === null, "testclips/v3testclip.mp4 is on D1 only");
  test.setTimeout(patient(900_000));
  const source = clip ?? "";
  const { baseURL } = testInfo.project.use;
  const scratch = mkdtempSync(path.join(tmpdir(), "offcut-e2e-"));
  const context = await browser.newContext({ acceptDownloads: true, ...(baseURL === undefined ? {} : { baseURL }) });
  try {
    await ensureModelCached(context);
    const page = await context.newPage();
    await installFakeApi(page);
    await routeAssets(page);
    await page.goto("/app");
    await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
    await dropClip(page, source);
    await expect(canvas(page).or(page.getByTestId("clip-failed")).or(page.getByTestId("clip-rejected"))).toBeVisible({ timeout: patient(360_000) });
    await expect(canvas(page), "the second recording was processed").toBeVisible();
    // The preview is attached before the export locks it (V3 changelog, known issue 4).
    await expect(button(page, messages.preview.play)).toBeEnabled({ timeout: 30_000 });

    await seedEntitlement(page, mintEntitlementToken());
    const arriving = page.waitForEvent("download", { timeout: EXPORT_TIMEOUT_MS });
    await exportButton(page).click();
    const file = path.join(scratch, "second-recording.mp4");
    await (await arriving).saveAs(file);
    await expect(page.getByTestId("export-done")).toHaveText(messages.export.done, { timeout: 30_000 });

    // Check 11, against what this source carries (D-100, D-208).
    const verdict = verifyMp4(file, "creator", sourceDurationMs(source), source);
    expect(verdict.output).toContain("PASS 11");
    expectAllChecks(verdict);
  } finally {
    await context.close();
    rmSync(scratch, { recursive: true, force: true });
  }
});
