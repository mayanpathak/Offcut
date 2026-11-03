// J9 to J12 on the Free plan, in a real browser (v6implementation §15.4): the person
// meets the limits, signs in through a link opened in a second tab, and spends the
// three exports of the month, each put through all 11 checks of the independent
// verifier. The build under test holds the test key, so the fake's tokens verify.
//
// One clip, processed once, and the cases in the order they are written. A case that
// fails stops the ones after it. Each export needs an edit of its own: the same edit
// comes from the render cache, and spends nothing.

import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { type BrowserContext, type Download, expect, type Page, test } from "@playwright/test";

import { PRICES_USD } from "../src/config/pricing";
import { messages } from "../src/copy/messages";
import { LIMITS } from "../src/gen/domain";
import { type FakeApi, installFakeApi } from "./helpers/fake-api";
import {
  axe,
  dropClip,
  ensureModelCached,
  expectAllChecks,
  opfsList,
  patient,
  referenceClip,
  routeAssets,
  sendQueuedAnalytics,
  setStyle,
  sha256Of,
  sourceDurationMs,
  trackSaves,
  verifyMp4,
} from "./helpers/fixtures";

const EMAIL = "someone@example.com";
const EXPORT_TIMEOUT_MS = patient(240_000);
/** The white frames of the reference clip, which check 8 compares. */
const REFERENCE_FLASHES = 2;
const TOTAL = LIMITS.FREE_EXPORTS_PER_MONTH;

const canvas = (page: Page) => page.getByTestId("preview-canvas");
const button = (page: Page, name: string) => page.getByRole("button", { name, exact: true });
const exportButton = (page: Page) => button(page, messages.export.button);
const counter = (page: Page) => page.getByTestId("export-counter");
const prompt = (page: Page) => page.getByTestId("upgrade-prompt");
const violations = async (page: Page) => (await axe(page)).map((v) => `${v.id}: ${String(v.nodes.length)} node(s)`);

test.describe("three free exports", () => {
  test.describe.configure({ mode: "serial" });

  let context: BrowserContext;
  let page: Page;
  let api: FakeApi;
  let scratch: string;
  let durationMs: number;
  const downloads: Download[] = [];
  let lastFile: string;

  const receipts = () => api.requests.filter((request) => request.path === "/usage/receipts");

  /** Waits for the file the page hands over after `click`, and saves it where no report picks it up. */
  async function saved(name: string, click: () => Promise<void>): Promise<string> {
    const arriving = page.waitForEvent("download", { timeout: EXPORT_TIMEOUT_MS });
    await click();
    const download = await arriving;
    const file = path.join(scratch, name);
    await download.saveAs(file);
    await expect(page.getByTestId("export-done")).toHaveText(messages.export.done, { timeout: 30_000 });
    return file;
  }

  function verifiedAsFree(file: string): void {
    expectAllChecks(verifyMp4(file, "free", durationMs, referenceClip()), { flashes: REFERENCE_FLASHES });
    // The same file is not a Creator's: its size and its watermark say so (INV-9).
    const asCreator = verifyMp4(file, "creator", durationMs, referenceClip());
    expect(asCreator.output).toContain("FAIL 2");
    expect(asCreator.output).toContain("FAIL 9");
  }

  test.beforeAll(async ({ browser }, testInfo) => {
    scratch = mkdtempSync(path.join(tmpdir(), "offcut-e2e-"));
    const { baseURL } = testInfo.project.use;
    context = await browser.newContext({ acceptDownloads: true, ...(baseURL === undefined ? {} : { baseURL }) });
    await ensureModelCached(context);
    page = await context.newPage();
    await page.setViewportSize({ width: 1280, height: 800 });
    page.on("download", (download) => downloads.push(download));
    api = await installFakeApi(page);
    await routeAssets(page);
    await trackSaves(page);
    await page.goto("/app");
    await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
    durationMs = sourceDurationMs(referenceClip());
    await dropClip(page, referenceClip());
    await expect(canvas(page).or(page.getByTestId("clip-failed")).or(page.getByTestId("clip-rejected"))).toBeVisible({ timeout: patient(240_000) });
    await expect(canvas(page), "the clip was processed").toBeVisible();
    await expect(button(page, messages.preview.play)).toBeEnabled({ timeout: 30_000 });
  });

  test.afterAll(async () => {
    await context.close();
    rmSync(scratch, { recursive: true, force: true });
  });

  test("signed out: the button is enabled, no counter shows, and the click opens the dialog with the limits", async () => {
    await expect(exportButton(page)).toBeEnabled();
    await expect(counter(page)).toHaveCount(0);
    await exportButton(page).click();
    const dialog = page.getByTestId("sign-in-dialog");
    await expect(dialog).toBeVisible();
    await expect(dialog.getByTestId("sign-in-limits")).toHaveText(messages.signIn.limits(TOTAL, PRICES_USD.creatorMonthly));
    await expect(dialog.getByTestId("sign-in-email")).toBeFocused();
    expect(await violations(page), "axe on the dialog").toEqual([]);
    expect(await opfsList(page, "exports")).toEqual([]);
  });

  test("the keys are the dialog's: typing a space in it does not play the video, and Escape closes it", async () => {
    await page.getByTestId("sign-in-email").pressSequentially("a b");
    await expect(page.getByTestId("sign-in-email")).toHaveValue("a b");
    await expect(button(page, messages.preview.pause)).toHaveCount(0);
    await page.keyboard.press("Escape");
    await expect(page.getByTestId("sign-in-dialog")).toHaveCount(0);
    await expect(button(page, messages.preview.play)).toBeEnabled();
    await expect(exportButton(page)).toBeEnabled();
  });

  test("the link opened in a new tab signs this one in: the dialog closes, and the counter is there before the click", async () => {
    await exportButton(page).click();
    const dialog = page.getByTestId("sign-in-dialog");
    await dialog.getByTestId("sign-in-email").fill(EMAIL);
    await button(page, messages.signIn.submit).click();
    await expect(dialog.getByTestId("sign-in-sent")).toContainText(messages.signIn.sentBody(EMAIL));
    expect(await violations(page), "axe on the sent state").toEqual([]);

    const tab = await context.newPage();
    await installFakeApi(tab, { account: api.account });
    await tab.goto("/auth/callback?token=fake-good-link");
    await expect(tab.getByTestId("callback-done")).toBeVisible();
    await tab.close();

    await expect(dialog).toHaveCount(0);
    await expect(counter(page)).toHaveText(messages.export.counter(3, TOTAL));
    // Outside the button's name (D-302): protected cases press "Export" by that exact name.
    await expect(exportButton(page)).toHaveAccessibleName(messages.export.button);
    await expect(exportButton(page)).toBeEnabled();
    await expect(page.getByTestId("export-status")).toHaveText("");
  });

  test("first export: 720 by 1280 with the watermark, all 11 checks; the counter falls; one receipt", async () => {
    await expect(prompt(page)).toHaveCount(0);
    const file = await saved("first.mp4", () => exportButton(page).click());
    lastFile = file;
    verifiedAsFree(file);
    await expect(counter(page)).toHaveText(messages.export.counter(2, TOTAL));
    await expect.poll(receipts).toHaveLength(1);
    expect(receipts()[0]?.method).toBe("POST");
    expect(Object.keys(receipts()[0]?.body as object)).toEqual(["export_id"]);
    const [stored] = await opfsList(page, "exports");
    expect(stored).toBe(`${String((receipts()[0]?.body as { export_id: string }).export_id)}.mp4`);
  });

  test("after a watermarked export: one prompt, beside the outcome, that takes no focus and passes axe (D-302)", async () => {
    await expect(prompt(page)).toHaveCount(1);
    await expect(prompt(page)).toHaveAttribute("data-placement", "after_export");
    await expect(prompt(page)).toContainText(messages.upgrade.afterExport(PRICES_USD.creatorMonthly));
    await expect(page.getByTestId("export-outcome").getByTestId("upgrade-prompt")).toHaveCount(0);
    await expect(prompt(page).locator(":focus")).toHaveCount(0);
    expect(await violations(page), "axe with the prompt").toEqual([]);
    await prompt(page).getByRole("button", { name: messages.upgrade.dismiss }).click();
    await expect(prompt(page)).toHaveCount(0);
  });

  test("the same edit again: the stored file, no receipt, the counter as it was, and the prompt stays dismissed", async () => {
    const file = await saved("first-again.mp4", () => exportButton(page).click());
    expect(await sha256Of(file)).toBe(await sha256Of(lastFile));
    await page.waitForTimeout(1_000);
    expect(receipts()).toHaveLength(1);
    await expect(counter(page)).toHaveText(messages.export.counter(2, TOTAL));
    await expect(prompt(page)).toHaveCount(0);
  });

  test("second export, of another edit: all 11 checks; the counter falls; two receipts", async () => {
    await setStyle(page, messages.style.bold);
    const file = await saved("second.mp4", () => exportButton(page).click());
    lastFile = file;
    verifiedAsFree(file);
    await expect(counter(page)).toHaveText(messages.export.counter(1, TOTAL));
    await expect.poll(receipts).toHaveLength(2);
  });

  test("before the third: the last-export dialog, and Not now begins nothing", async () => {
    await setStyle(page, messages.style.mono);
    await sendQueuedAnalytics(page);
    const startedBefore = api.eventsOf("export_started").length;
    await exportButton(page).click();
    const dialog = page.getByTestId("last-export-dialog");
    await expect(dialog).toContainText(messages.export.lastFree.title);
    expect(await violations(page), "axe on the dialog").toEqual([]);
    // D-311: the focus is on no field here, and Space still does not reach the player.
    await page.keyboard.press("Space");
    await page.waitForTimeout(500);
    await expect(button(page, messages.preview.pause)).toHaveCount(0);
    await dialog.getByRole("button", { name: messages.export.lastFree.notNow }).click();
    await expect(dialog).toHaveCount(0);
    await expect(page.getByTestId("export-progress")).toHaveCount(0);
    await sendQueuedAnalytics(page);
    expect(api.eventsOf("export_started")).toHaveLength(startedBefore);
    expect(receipts()).toHaveLength(2);
  });

  test("third export, after saying yes: all 11 checks; none left; three receipts", async () => {
    await exportButton(page).click();
    const dialog = page.getByTestId("last-export-dialog");
    const file = await saved("third.mp4", () => dialog.getByRole("button", { name: messages.export.lastFree.go }).click());
    lastFile = file;
    await expect(dialog).toHaveCount(0);
    verifiedAsFree(file);
    await expect(counter(page)).toHaveText(messages.export.counter(0, TOTAL));
    await expect.poll(receipts).toHaveLength(3);
  });

  test("the fourth: the prompt at the limit, nothing rendered, nothing spent, and the file still there to download", async () => {
    await prompt(page)
      .getByRole("button", { name: messages.upgrade.dismiss })
      .click();
    await setStyle(page, messages.style.clean);
    await sendQueuedAnalytics(page);
    const startedBefore = api.eventsOf("export_started").length;
    const stored = await opfsList(page, "exports");
    const handedOver = downloads.length;

    await exportButton(page).click();
    await expect(prompt(page)).toHaveAttribute("data-placement", "limit_reached");
    await expect(prompt(page)).toContainText(messages.upgrade.limitReached(TOTAL, PRICES_USD.creatorMonthly));
    await page.waitForTimeout(2_000);
    await expect(page.getByTestId("export-progress")).toHaveCount(0);
    await expect(exportButton(page), "the button stays ready").toBeEnabled();
    await sendQueuedAnalytics(page);
    expect(api.eventsOf("export_started"), "nothing began").toHaveLength(startedBefore);
    expect(downloads).toHaveLength(handedOver);
    expect(await opfsList(page, "exports"), "nothing swept").toEqual(stored);
    expect(receipts()).toHaveLength(3);

    // "Download again" is the file the person has already paid a count for.
    const again = await saved("third-again.mp4", () => page.getByTestId("download-again").click());
    expect(await sha256Of(again)).toBe(await sha256Of(lastFile));
    expect(receipts()).toHaveLength(3);
  });
});
