// A long video's job survives a reload and the cloud's failures, on the
// flags-on build (v6.1implementation §11.4, project `media-flags`): a reload
// while transcribing goes on without the file and sends no kept window again;
// a reload while reading the sound asks for the same file and refuses another;
// each pause says why, and Continue finishes the job. The fake API's windows
// come from `helpers/long-windows.ts` (D-431); nothing reaches Groq.

import { type BrowserContext, expect, type Page, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import type { FakeApiOptions } from "./helpers/fake-api";
import { axe, dropFile, genFixture, NO_GPU, patient } from "./helpers/fixtures";
import {
  capture,
  keptWindows,
  LONG_12MIN_WORDS,
  openLong,
  signedInCloud,
  sourceRecord,
  windowTotal,
  wordsShown,
} from "./helpers/long-windows";

const LONG = genFixture("long_12min.mp4");
const TRANSCRIBED_TIMEOUT_MS = patient(300_000);
const copy = messages.long;

/** The `X-Offcut-Window` index of every `/transcribe` request the context sends, in order. */
function windowsSent(context: BrowserContext): number[] {
  const sent: number[] = [];
  context.on("request", (request) => {
    const window = request.headers()["x-offcut-window"];
    if (request.url().endsWith("/api/v1/transcribe") && window !== undefined) {
      sent.push(Number(window.split("/")[0]));
    }
  });
  return sent;
}

/** Drops the 12-minute video and presses Start at the gate. */
async function start(page: Page): Promise<void> {
  await dropFile(page, LONG);
  await expect(page.getByTestId("source-gate")).toHaveAttribute("data-reason", "ready", { timeout: 60_000 });
  await page.getByTestId("source-start").click();
}

/** The job ends with every word of the nine loops on screen. */
async function expectTranscribed(page: Page): Promise<void> {
  await expect(page.getByTestId("source-transcript")).toBeVisible({ timeout: TRANSCRIBED_TIMEOUT_MS });
  expect(await wordsShown(page)).toHaveLength(LONG_12MIN_WORDS);
}

/** Paused, with its reason's words and Continue. */
async function expectPaused(page: Page, reason: keyof typeof copy.paused): Promise<void> {
  const progress = page.getByTestId("source-progress");
  await expect(progress).toHaveAttribute("data-status", "paused", { timeout: TRANSCRIBED_TIMEOUT_MS });
  await expect(progress.getByRole("alert")).toHaveAttribute("data-reason", reason);
  await expect(progress.getByRole("alert")).toHaveText(copy.paused[reason]);
  await expect(page.getByTestId("source-continue")).toHaveText(copy.continue);
  await expect(page.getByTestId("source-cancel")).toBeVisible();
  // While paused the tab's title is the page's own again.
  await expect(page).not.toHaveTitle(/%/);
  await capture(page, `paused-${reason}`);
  expect(await axe(page)).toEqual([]);
}

test("reload while transcribing: Continue your video finishes without the file, and no kept window is sent again", { tag: NO_GPU }, async ({ page, context }) => {
  const sent = windowsSent(context);
  await openLong(page, signedInCloud(1_500));
  await start(page);
  // The progress is in the tab's title while the job works (D-418).
  await expect(page).toHaveTitle(/^\d+% - Offcut$/, { timeout: 60_000 });
  await expect.poll(() => keptWindows(page), { timeout: TRANSCRIBED_TIMEOUT_MS }).toEqual(expect.arrayContaining([0, 1, 2, 3]));
  const kept = await keptWindows(page);
  expect(kept.length).toBeLessThan(windowTotal());

  await page.reload();
  sent.length = 0;
  const resume = page.getByTestId("source-resume");
  await expect(resume).toHaveText(copy.continueVideo, { timeout: 30_000 });
  await capture(page, "resume");
  expect(await axe(page)).toEqual([]);
  await resume.click();
  await expectTranscribed(page);

  // Only the windows not kept before the reload were sent after it.
  for (const k of kept) {
    expect(sent, `window ${String(k)} was kept before the reload`).not.toContain(k);
  }
  expect(new Set([...kept, ...sent]).size).toBe(windowTotal());
});

test("reload while reading the sound: Continue asks for the file, refuses another, and goes on from the checkpoint", { tag: NO_GPU }, async ({ page }) => {
  await openLong(page, signedInCloud());
  await start(page);
  // The first checkpoint (D-403): ten seconds of sound written and flushed.
  await expect.poll(async () => (await sourceRecord(page))?.pcm16Seconds ?? 0, { timeout: 60_000, intervals: [50] }).toBeGreaterThan(0);
  await page.reload();
  const checkpoint = await sourceRecord(page);
  expect(checkpoint?.stage).toBe("extracting_audio");
  expect(checkpoint?.pcm16Seconds).toBeGreaterThan(0);

  await page.getByTestId("source-resume").click();
  const waiting = page.getByTestId("source-awaiting-file");
  await expect(waiting).toContainText(copy.awaitingFile, { timeout: 30_000 });
  await capture(page, "awaiting-file");
  expect(await axe(page)).toEqual([]);

  await dropFile(page, genFixture("rej_too_long_91s.mp4"));
  await expect(waiting.getByRole("alert")).toHaveText(copy.differentFile, { timeout: 30_000 });
  await capture(page, "different-file");
  expect(await axe(page)).toEqual([]);

  await dropFile(page, LONG);
  await expectTranscribed(page);
});

test("503 with the server's switch off pauses as unavailable; Continue, once it is back, finishes", { tag: NO_GPU }, async ({ page }) => {
  const options: FakeApiOptions = signedInCloud();
  await openLong(page, options);
  // From Start on, /transcribe answers 503 and /healthz says the switch is off.
  options.failures = { "/transcribe": { status: 503, code: "transcribe_unavailable" } };
  options.healthz = { cloudTranscribe: false };
  await start(page);
  await expectPaused(page, "unavailable");

  options.failures = {};
  options.healthz = { cloudTranscribe: true };
  await page.getByTestId("source-continue").click();
  await expectTranscribed(page);
});

test("429 pauses as busy; Continue finishes", { tag: NO_GPU }, async ({ page }) => {
  const options: FakeApiOptions = signedInCloud();
  await openLong(page, options);
  options.failures = { "/transcribe": { status: 429, code: "rate_limited", retryAfterSecs: 60 } };
  await start(page);
  await expectPaused(page, "busy");

  options.failures = {};
  await page.getByTestId("source-continue").click();
  await expectTranscribed(page);
});

test("offline pauses as offline after its retries; Continue, once online, finishes", { tag: NO_GPU }, async ({ page, context }) => {
  await openLong(page, signedInCloud());
  await start(page);
  await context.setOffline(true);
  await expectPaused(page, "offline");

  await context.setOffline(false);
  await page.getByTestId("source-continue").click();
  await expectTranscribed(page);
});

test("a session that ends pauses as signed out; Continue asks to sign in first", { tag: NO_GPU }, async ({ page }) => {
  const options: FakeApiOptions = signedInCloud();
  await openLong(page, options);
  // Every /transcribe answers 401, after the one refresh too: the session is gone (D-410).
  options.failures = { "/transcribe": { status: 401, code: "unauthorized" } };
  await start(page);
  await expectPaused(page, "signed_out");
  await page.getByTestId("source-continue").click();
  await expect(page.getByRole("dialog")).toBeVisible({ timeout: 30_000 });
});
