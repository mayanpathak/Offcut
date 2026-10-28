// The preview's timing in Chrome on D1 (TS §20 acceptance, v4implementation
// §20.4, §21.1, D-148, D-155, D-173): the first frame after `ready`, a seek
// to the portrait clip's white frame, a word edit's way to the screen, and
// the drift over the whole 60 s clip. Everything is read from the page: the
// player's `data-*` statistics, screenshots of its canvas, and the save
// state of the top bar; never a worker (D-155).
//
// One clip for the four cases, in the order written (as `pipeline-preview`
// does): a case that fails stops the ones after it. Each case's numbers are
// written to the report and to the console, for the changelog and Prompt
// 126's readings.

import { type BrowserContext, expect, type Page, test, type TestInfo } from "@playwright/test";

import { messages } from "../src/copy/messages";
import { installFakeApi } from "./helpers/fake-api";
import {
  dropClip,
  ensureModelCached,
  genFixture,
  meanBrightness,
  openWord,
  patient,
  pngPixels,
  previewStats,
  routeAssets,
  saves,
  seekTo,
  settled,
  trackSaves,
  transcriptWords,
  wordField,
} from "./helpers/fixtures";

declare global {
  interface Window {
    /** When the preview's canvas came onto the page (the clip `ready`), in the page's `performance.now()`. */
    e2eCanvasAt?: number | undefined;
    /** When its play button became enabled: `attachPreview` answered, which the worker does once the first frame is drawn. */
    e2eAttachedAt?: number | undefined;
    /** When Enter was pressed in a word's field, and when "Saving…" appeared after it. */
    e2eEnterAt?: number | undefined;
    e2eSavingAt?: number | undefined;
  }
}

/** TS §20's acceptance: the worst drift between the sound and the frame, and the share of late frames. */
const MAX_DRIFT_MS = 80;
const MAX_LATE_SHARE = 0.05;
/** The first frame after `ready` (assumption, M1.5; §21.1). */
const FIRST_FRAME_MS = 500;
/** A seek's frame (D-148; assumption, M1.5). */
const SEEK_FRAME_MS = 300;
/** A word edit's commit to the frame drawn with it (D-173, TS §13.5). */
const EDIT_TO_PREVIEW_MAX_MS = 150;
/** The portrait clip's white frames are frames 150 and 1650 (fixtures/gen_fixtures.sh): 5.0 s and 55.0 s. */
const WHITE_FRAME_MS = 55_000;
const WHITE_BRIGHTNESS = 200;
const EDITS = 20;
/** The 90th percentile of 20 sorted durations (Prompt 122). */
const P90_INDEX = 17;

const canvas = (page: Page) => page.getByTestId("preview-canvas");
const button = (page: Page, name: string) => page.getByRole("button", { name, exact: true });
/** A failure of the preview: a frame that was not given back fails the call that kept it (INV-11, D-45). */
const previewFailure = (page: Page) => page.getByTestId("preview-failed").or(page.getByText(messages.editor.failedStub));
const now = (page: Page) => page.evaluate(() => performance.now());

/**
 * A small picture of the middle of the canvas: a page screenshot clipped to
 * it costs about 60 ms on D1, where one of the whole canvas element costs
 * 150 ms to over a second and slows the worker's drawing meanwhile (found by
 * this prompt's first run).
 */
async function middleOfCanvas(page: Page): Promise<Buffer> {
  const box = await canvas(page).boundingBox();
  if (box === null) {
    throw new Error("the canvas is not on the page");
  }
  const side = Math.min(120, box.width / 2, box.height / 2);
  return page.screenshot({ clip: { x: box.x + (box.width - side) / 2, y: box.y + (box.height - side) / 2, width: side, height: side } });
}

/** Whether a picture shows a frame of the clip and not an empty canvas: a fifth of its pixels or more are not dark. */
function showsAFrame(png: Buffer): boolean {
  const { width, height, channels, data } = pngPixels(png);
  let lit = 0;
  for (let p = 0; p < width * height; p += 1) {
    const max = Math.max(data[p * channels] ?? 0, data[p * channels + 1] ?? 0, data[p * channels + 2] ?? 0);
    lit += max > 40 ? 1 : 0;
  }
  return lit / (width * height) >= 0.2;
}

/** The case's numbers: in the report, and on the console for the changelog. */
function record(testInfo: TestInfo, line: string): void {
  testInfo.annotations.push({ type: "reading", description: line });
  process.stdout.write(`      ${line}\n`);
}

/** Notes, from before the page loads, when the canvas appears, when Enter is pressed and when "Saving…" shows. */
async function watchPage(page: Page): Promise<void> {
  await page.addInitScript((saving) => {
    document.addEventListener(
      "keydown",
      (event) => {
        if (event.key === "Enter") {
          window.e2eEnterAt = performance.now();
          window.e2eSavingAt = undefined;
        }
      },
      true,
    );
    new MutationObserver(() => {
      window.e2eCanvasAt ??= document.querySelector('[data-testid="preview-canvas"]') === null ? undefined : performance.now();
      // The transport's one button: "Play" while detached (disabled) and once attached (enabled).
      const play = document.querySelector('button[aria-label="Play"]');
      if (window.e2eAttachedAt === undefined && play instanceof HTMLButtonElement && !play.disabled) {
        window.e2eAttachedAt = performance.now();
      }
      if (window.e2eEnterAt !== undefined && window.e2eSavingAt === undefined && (document.body.textContent ?? "").includes(saving)) {
        window.e2eSavingAt = performance.now();
      }
    }).observe(document, { childList: true, subtree: true, characterData: true, attributes: true });
  }, messages.editor.saving);
}

test.describe("the preview's timing, on the 60 s portrait clip", () => {
  test.describe.configure({ mode: "serial" });

  let context: BrowserContext;
  let page: Page;

  test.beforeAll(async ({ browser }, testInfo) => {
    const { baseURL } = testInfo.project.use;
    context = await browser.newContext({ ...(baseURL === undefined ? {} : { baseURL }), viewport: { width: 1280, height: 800 } });
    await ensureModelCached(context);
    page = await context.newPage();
    await installFakeApi(page);
    await routeAssets(page);
    await trackSaves(page);
    await watchPage(page);
    await page.goto("/app");
    await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  });

  test.afterAll(async () => {
    await context.close();
  });

  test("first frame: the canvas shows a frame of the clip within 500 ms of ready", async ({}, testInfo) => {
    await dropClip(page, genFixture("speech/speech_60s_portrait.mp4"));
    await expect(canvas(page).or(page.getByTestId("clip-failed"))).toBeVisible({ timeout: patient(240_000) });
    await expect(canvas(page), "the clip was processed").toBeVisible();
    await expect(button(page, messages.preview.play)).toBeEnabled({ timeout: 30_000 });
    // When: from the page alone, so that nothing slows the drawing it times.
    // The worker draws the first frame before `attachPreview` answers, and
    // the play button is enabled when it has.
    const { readyAt, attachedAt } = await page.evaluate(() => ({
      readyAt: window.e2eCanvasAt ?? Number.NaN,
      attachedAt: window.e2eAttachedAt ?? Number.NaN,
    }));
    const ms = Math.round(attachedAt - readyAt);
    // What: the picture on the canvas then, which a paused preview does not draw again.
    const shown = showsAFrame(await middleOfCanvas(page));
    record(testInfo, `first frame: drawn ${String(ms)} ms after ready, a frame of the clip on the canvas: ${String(shown)} (target ${String(FIRST_FRAME_MS)})`);
    expect(shown, "the canvas shows a frame of the clip, not an empty canvas").toBe(true);
    expect(ms, `the first frame was drawn ${String(ms)} ms after ready`).toBeLessThanOrEqual(FIRST_FRAME_MS);
  });

  test("seek: paused, a seek to 55.0 s shows the white frame within 300 ms; no frame is kept after play, seek and pause", async ({}, testInfo) => {
    await expect(button(page, messages.preview.play)).toBeEnabled({ timeout: 30_000 });
    // A play and a pause first, so the seek starts from a preview that has played.
    await button(page, messages.preview.play).click();
    await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 30_000 });
    await page.waitForTimeout(1_500);
    await button(page, messages.preview.pause).click();
    await expect(button(page, messages.preview.play)).toBeVisible({ timeout: 30_000 });
    expect(meanBrightness(await middleOfCanvas(page))).toBeLessThan(WHITE_BRIGHTNESS);

    const from = await now(page);
    await seekTo(page, WHITE_FRAME_MS);
    let brightAt = Number.NaN;
    let brightness = 0;
    for (let tries = 0; tries < 60 && Number.isNaN(brightAt); tries += 1) {
      const png = await middleOfCanvas(page);
      const at = await now(page);
      brightness = meanBrightness(png);
      if (brightness > WHITE_BRIGHTNESS) {
        brightAt = at;
      }
    }
    const ms = Math.round(brightAt - from);
    record(testInfo, `seek to 55.0 s: the white frame after ${String(ms)} ms, mean brightness ${brightness.toFixed(0)} (target ${String(SEEK_FRAME_MS)} ms, > ${String(WHITE_BRIGHTNESS)})`);
    expect(ms, `the white frame showed ${String(ms)} ms after the seek`).toBeLessThanOrEqual(SEEK_FRAME_MS);

    // INV-11: a play ends by closing every decoded frame, and one left open
    // fails that play (D-45). A play after the seek, a pause, and one more:
    // none may fail, so none kept a frame of the seek or of the play before.
    for (const playMs of [1_000, 1_000]) {
      await button(page, messages.preview.play).click();
      await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 30_000 });
      await page.waitForTimeout(playMs);
      await button(page, messages.preview.pause).click();
      await expect(button(page, messages.preview.play)).toBeVisible({ timeout: 30_000 });
      await expect(previewFailure(page)).toHaveCount(0);
    }
    const stats = await previewStats(page);
    expect(stats.frames, "the last play drew frames").toBeGreaterThan(0);
  });

  test("edit to preview: the 90th percentile of 20 word commits is within 150 ms", async ({}, testInfo) => {
    // Words of letters only, spread over the clip, none of them a figure a visual is made of.
    const words = (await transcriptWords(page)).filter((word) => /^[a-z]{3,}$/i.test(word.text));
    expect(words.length, "the transcript has enough words").toBeGreaterThanOrEqual(EDITS);
    const step = Math.floor(words.length / EDITS);
    const durations: number[] = [];
    for (let n = 0; n < EDITS; n += 1) {
      const word = words[n * step];
      if (word === undefined) {
        throw new Error("a word went missing");
      }
      await openWord(page, word.index);
      // The click seeks to the word: its frame is drawn before the commit is timed.
      await page.waitForTimeout(400);
      await wordField(page).fill(`${word.text}s`);
      const since = await saves(page);
      await page.keyboard.press("Enter");
      await settled(page, since);
      const ms = await page.evaluate(() => (window.e2eSavingAt ?? Number.NaN) - (window.e2eEnterAt ?? Number.NaN));
      expect(ms, `commit ${String(n + 1)}: "Saving…" was seen after Enter`).not.toBeNaN();
      durations.push(Math.round(ms));
    }
    const sorted = [...durations].sort((a, b) => a - b);
    const p90 = sorted[P90_INDEX] ?? Number.NaN;
    record(
      testInfo,
      `edit to preview: p90 ${String(p90)} ms, median ${String(sorted[9])} ms, max ${String(sorted.at(-1))} ms over ${String(EDITS)} commits [${durations.join(", ")}] (target ${String(EDIT_TO_PREVIEW_MAX_MS)})`,
    );
    expect(p90, `the 90th percentile of edit to preview is ${String(p90)} ms`).toBeLessThanOrEqual(EDIT_TO_PREVIEW_MAX_MS);
    await expect(previewFailure(page)).toHaveCount(0);
  });

  test("drift: the whole clip plays with the frame at most 80 ms from the sound, and 5% of frames late at most", async ({}, testInfo) => {
    test.setTimeout(patient(180_000));
    await seekTo(page, 0);
    await page.waitForTimeout(500);
    await button(page, messages.preview.play).click();
    await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 30_000 });
    // It plays to the end, where the one button says "Play again".
    await expect(button(page, messages.preview.replay)).toBeVisible({ timeout: 90_000 });
    const stats = await previewStats(page);
    const share = stats.late / stats.frames;
    record(
      testInfo,
      `drift over 60 s: max ${stats.maxDriftMs.toFixed(1)} ms, ${String(stats.late)} late of ${String(stats.frames)} frames (${(share * 100).toFixed(2)}%), downscaled ${String(stats.downscaled)} (targets ${String(MAX_DRIFT_MS)} ms, ${String(MAX_LATE_SHARE * 100)}%)`,
    );
    expect(stats.frames, "a 60 s play draws most of its 1,800 frames").toBeGreaterThan(1_500);
    expect(stats.maxDriftMs, `the worst drift was ${String(stats.maxDriftMs)} ms`).toBeLessThanOrEqual(MAX_DRIFT_MS);
    expect(share, `${(share * 100).toFixed(2)}% of the frames were late`).toBeLessThanOrEqual(MAX_LATE_SHARE);
    await expect(previewFailure(page)).toHaveCount(0);
  });
});
