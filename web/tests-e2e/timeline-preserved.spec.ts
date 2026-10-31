// INV-5 and INV-10 in a real browser, on the two clips with long pauses (TS
// §27.1): nothing moves the words, the preview or the export off the time they
// were recorded at. This is BP §7's demo as a test: 20 s in, 20 s out, the pause
// intact, in sync on both sides, by all 11 checks of the independent verifier.
// An encoder that fails starts the file again and costs no export.
//
// Each group processes its clip once and its cases go in the order they are
// written: a case's state is the previous one's. No word of a clip is printed,
// in a message or a report: a failure names a word by where it is, and gives
// numbers (SAB rule 9).

import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { type Browser, type BrowserContext, type Download, expect, type Page, test, type TestInfo } from "@playwright/test";

import { messages } from "../src/copy/messages";
import { type FakeApi, mintEntitlementToken, seedEntitlement } from "./helpers/fake-api";
import {
  captionMirror,
  editWord,
  expectAllChecks,
  forgetProgress,
  genFixture,
  openClip,
  opfsList,
  patient,
  previewAt,
  readIndexedDb,
  scrubBar,
  seekTo,
  setStyle,
  toggleItem,
  verifyMp4,
  watchExport,
} from "./helpers/fixtures";

type Word = { text: string; start_ms: number; end_ms: number };

const EXPORT_TIMEOUT_MS = patient(240_000);
/** Each fixture holds two white frames, which check 8 compares (`gen_fixtures.sh`). */
const FLASHES = 2;
/**
 * How far before a sound the recognizer may start a word: its times come in
 * steps of 20 ms, and it heard the first word after the long pause at 9,960 ms
 * (`fixtures/speech/README.md`).
 */
const RECOGNIZER_SLACK_MS = 40;

const button = (page: Page, name: string) => page.getByRole("button", { name, exact: true });

/** A page of its own that records what an export shows, on `fixture`, processed and paused. */
async function openFixture(browser: Browser, info: TestInfo, fixture: string): Promise<{ context: BrowserContext; page: Page; api: FakeApi; downloads: Download[] }> {
  const { baseURL } = info.project.use;
  const context = await browser.newContext({ acceptDownloads: true, ...(baseURL === undefined ? {} : { baseURL }) });
  const page = await context.newPage();
  const downloads: Download[] = [];
  page.on("download", (download) => downloads.push(download));
  await watchExport(page);
  const api = await openClip(page, fixture);
  return { context, page, api, downloads };
}

/** The words as the pipeline stored them. */
async function storedWords(page: Page): Promise<Word[]> {
  const [clipId] = await opfsList(page, "clips");
  const stored = (await readIndexedDb(page, "transcripts", clipId ?? "")) as { value: { transcript: { words: Word[] } } };
  return stored.value.transcript.words;
}

/** With a token seeded, presses Export and saves the file where no report picks it up. A failure on the page ends the wait, and says its code. */
async function exportAndSave(page: Page, api: FakeApi, scratch: string, name: string): Promise<string> {
  await seedEntitlement(page, mintEntitlementToken());
  const failuresBefore = await page.evaluate(() => window.e2eExportFailures ?? 0);
  const arriving = page.waitForEvent("download", { timeout: EXPORT_TIMEOUT_MS });
  const failing = page.waitForFunction((before) => (window.e2eExportFailures ?? 0) > before, failuresBefore, { timeout: EXPORT_TIMEOUT_MS });
  // One of the two ends the wait; the other is let go when the page closes.
  void arriving.catch(() => undefined);
  void failing.catch(() => undefined);
  await button(page, messages.export.button).click();
  const download = await Promise.race([arriving, failing.then(() => null)]);
  if (download === null) {
    throw new Error(`the export failed: ${JSON.stringify(api.eventsOf("export_failed").at(-1))}`);
  }
  const file = path.join(scratch, name);
  await download.saveAs(file);
  await expect(page.getByTestId("export-done")).toHaveText(messages.export.done, { timeout: 30_000 });
  return file;
}

/** Tells the render worker to fail its video encoder at `frame`, on the next `times` attempts (D-218). */
async function failEncoderAt(page: Page, frame: number, times: number): Promise<void> {
  await page.evaluate(
    ([f, t]) => {
      (window as unknown as { __offcutDev: { failEncoderAt(f: number, t: number): void } }).__offcutDev.failEncoderAt(f ?? 0, t ?? 0);
    },
    [frame, times],
  );
}

test.describe("the long pause: 20 s in, 20 s out", () => {
  test.describe.configure({ mode: "serial" });

  const DURATION_MS = 20_000;
  const PAUSE = { from: 5_000, to: 10_000 };

  let fixture: string;
  let scratch: string;
  let context: BrowserContext;
  let page: Page;
  let api: FakeApi;
  let downloads: Download[];
  /** The first word after the pause: read by the first case, used by the ones after it. */
  let firstAfter: { index: number; word: Word };

  test.beforeAll(async ({ browser }, info) => {
    fixture = genFixture("speech/speech_20s_long_pause.mp4");
    // Outside the test's own directory: an exported file is never part of a report (TS §22.5).
    scratch = mkdtempSync(path.join(tmpdir(), "offcut-e2e-"));
    ({ context, page, api, downloads } = await openFixture(browser, info, fixture));
  });

  test.afterAll(async () => {
    await context.close();
    rmSync(scratch, { recursive: true, force: true });
  });

  test("Words keep their times", async () => {
    const words = await storedWords(page);
    expect(words.length, "the clip has words").toBeGreaterThan(0);
    words.forEach((word, index) => {
      const before = word.end_ms <= PAUSE.from + RECOGNIZER_SLACK_MS;
      const after = word.start_ms >= PAUSE.to - RECOGNIZER_SLACK_MS;
      expect(before || after, `word ${String(index)}, ${String(word.start_ms)} to ${String(word.end_ms)} ms, lies in the pause`).toBe(true);
      // None was moved earlier: each starts where the one before it has ended, or later.
      expect(word.start_ms, `word ${String(index)} starts after the one before it`).toBeGreaterThanOrEqual(words[index - 1]?.end_ms ?? 0);
    });
    const index = words.findIndex((word) => word.start_ms >= PAUSE.to - RECOGNIZER_SLACK_MS);
    const word = words[index];
    if (word === undefined) {
      throw new Error("no word after the pause");
    }
    expect(index, "words before the pause").toBeGreaterThan(0);
    firstAfter = { index, word };
  });

  test("Caption at its spoken time", async () => {
    const { text, start_ms: start } = firstAfter.word;
    const showsIt = async (): Promise<boolean> => (await captionMirror(page)).text.includes(text);
    await seekTo(page, start + 60);
    await expect.poll(showsIt, { message: "the first word after the pause is shown when it is spoken" }).toBe(true);
    await seekTo(page, start - 1_000);
    await expect.poll(showsIt, { message: "the first word after the pause is not shown a second before" }).toBe(false);
  });

  test("Preview's length", async () => {
    expect(await scrubBar(page).getAttribute("max"), "the bar is as long as the source").toBe(String(DURATION_MS));
    await seekTo(page, DURATION_MS - 500);
    await button(page, messages.preview.play).click();
    await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 30_000 });
    // At the end the one button says "Play again".
    await expect(button(page, messages.preview.replay), "the preview ended by itself").toBeVisible({ timeout: 30_000 });
    const at = await previewAt(page);
    expect(at, "the preview got to the end").toBeGreaterThan(DURATION_MS - 500);
    expect(at, "the preview stopped at the source's end").toBeLessThanOrEqual(DURATION_MS);
  });

  test("Export is 20 s", async () => {
    const file = await exportAndSave(page, api, scratch, "long-pause.mp4");
    expectAllChecks(verifyMp4(file, "creator", DURATION_MS, fixture), { flashes: FLASHES });
  });

  test("An edit leaves the timeline alone", async () => {
    await editWord(page, firstAfter.index, "Fixed");
    await toggleItem(page, "Big number", false);
    await setStyle(page, "Bold");
    expect(await scrubBar(page).getAttribute("max"), "the bar did not move").toBe(String(DURATION_MS));
    const file = await exportAndSave(page, api, scratch, "edited.mp4");
    expectAllChecks(verifyMp4(file, "creator", DURATION_MS, fixture), { flashes: FLASHES });
  });

  test("The encoder fails", async () => {
    await forgetProgress(page);
    const failuresBefore = await page.evaluate(() => window.e2eExportFailures ?? 0);
    await failEncoderAt(page, 30, 1);
    const file = await exportAndSave(page, api, scratch, "restarted.mp4");

    const bar = (await page.evaluate(() => window.e2eExportBar ?? [])).map(Number);
    expect(bar.some((value, index) => value < (bar[index - 1] ?? 0)), `the bar fell back: ${bar.join(" ")}`).toBe(true);
    expect(bar.at(-1)).toBe(100);
    expect(await page.evaluate(() => window.e2eExportFailures), "no failure came onto the page").toBe(failuresBefore);
    await expect(page.getByTestId("export-failed")).toHaveCount(0);
    expectAllChecks(verifyMp4(file, "creator", DURATION_MS, fixture), { flashes: FLASHES });
  });

  // Last in the group: the ladder has fewer entries than this asks for, so the worker keeps the rest of the fault.
  test("Every entry fails", async () => {
    const downloadsBefore = downloads.length;
    await failEncoderAt(page, 30, 9);
    await seedEntitlement(page, mintEntitlementToken());
    await button(page, messages.export.button).click();

    const card = page.getByTestId("export-failed");
    const words = messages.errors.E_ENCODE_VIDEO;
    await expect(card).toBeVisible({ timeout: EXPORT_TIMEOUT_MS });
    await expect(card).toContainText(words.title);
    await expect(card).toContainText(words.body);
    await expect(card).toContainText(words.action);
    await expect(button(page, messages.recovery.tryAgain)).toBeVisible();
    await expect(button(page, messages.recovery.close)).toBeVisible();

    expect(downloads).toHaveLength(downloadsBefore);
    // The file of the export before went when this one started, and this one left none.
    expect(await opfsList(page, "exports")).toEqual(["tmp"]);
    expect(await opfsList(page, "exports/tmp")).toEqual([]);

    // The preview is given back, and it moves.
    await expect(button(page, messages.preview.play)).toBeEnabled({ timeout: 30_000 });
    await seekTo(page, 1_000);
    await button(page, messages.preview.play).click();
    await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 30_000 });
    await expect.poll(() => previewAt(page), { timeout: 30_000, message: "the preview plays" }).toBeGreaterThan(1_000);
    await button(page, messages.preview.pause).click();
    await expect(button(page, messages.preview.play)).toBeVisible({ timeout: 30_000 });
    await expect(page.getByTestId("preview-failed")).toHaveCount(0);
  });
});

test.describe("sparse speech: 30 s in, 30 s out", () => {
  test.describe.configure({ mode: "serial" });

  const DURATION_MS = 30_000;
  /** Where `gen_fixtures.sh` places its five pieces, each this long. */
  const PIECES_AT = [500, 7_000, 13_500, 19_500, 26_000];
  const PIECE_MS = 1_100;
  const PIECE_MARGIN_MS = 300;

  let fixture: string;
  let scratch: string;
  let context: BrowserContext;
  let page: Page;
  let api: FakeApi;

  test.beforeAll(async ({ browser }, info) => {
    fixture = genFixture("speech/speech_30s_sparse.mp4");
    scratch = mkdtempSync(path.join(tmpdir(), "offcut-e2e-"));
    ({ context, page, api } = await openFixture(browser, info, fixture));
  });

  test.afterAll(async () => {
    await context.close();
    rmSync(scratch, { recursive: true, force: true });
  });

  test("Words stay in their pieces", async () => {
    const words = await storedWords(page);
    const pieceOf = (word: Word): number =>
      PIECES_AT.findIndex((at) => word.start_ms >= at - PIECE_MARGIN_MS && word.start_ms <= at + PIECE_MS + PIECE_MARGIN_MS);
    words.forEach((word, index) => {
      // The recognizer runs a phrase's last words on over the room tone (`fixtures/speech/README.md`):
      // such a word follows the one before it at once. One after a gap starts in a piece, or it was moved.
      const runsOn = word.start_ms - (words[index - 1]?.end_ms ?? 0) <= RECOGNIZER_SLACK_MS;
      expect(runsOn || pieceOf(word) >= 0, `word ${String(index)}, from ${String(word.start_ms)} ms, starts in a piece`).toBe(true);
      expect(word.start_ms, `word ${String(index)} starts after the one before it`).toBeGreaterThanOrEqual(words[index - 1]?.end_ms ?? 0);
    });
    const held = new Set(words.map(pieceOf).filter((piece) => piece >= 0));
    expect(held.size, "pieces that hold a word").toBeGreaterThanOrEqual(3);
  });

  test("Export is 30 s", async () => {
    const file = await exportAndSave(page, api, scratch, "sparse.mp4");
    expectAllChecks(verifyMp4(file, "creator", DURATION_MS, fixture), { flashes: FLASHES });
  });
});
