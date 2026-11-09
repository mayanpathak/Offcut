// The long page in a real browser, on the flags-on build (v6.1implementation
// §11.4, project `media-flags`): the two ways in, the gate, a 12-minute
// video read in place and transcribed through the fake's windows, the
// network rules of the long path, the hand-over of a short file, the
// refusals, and axe on every state. The fake API never reaches Groq: each
// window is answered from the real adapter's answer for the reference clip
// (`helpers/long-windows.ts`, D-431).

import { readFileSync, statSync } from "node:fs";

import { expect, type Page, test } from "@playwright/test";

import { longPageDuration } from "../src/copy/long";
import { messages } from "../src/copy/messages";
import { installFakeApi } from "./helpers/fake-api";
import { axe, dropClip, dropFile, ensureModelCached, genFixture, NO_GPU, patient, routeAssets } from "./helpers/fixtures";
import {
  capture,
  firstSentence,
  LONG_12MIN_WORDS,
  openLong,
  signedInCloud,
  sourceStoreCount,
  windowTotal,
  wordsShown,
} from "./helpers/long-windows";

const LONG = genFixture("long_12min.mp4");
/** From the drop to the transcript: about 40 s on D1, the audio step most of it; a runner is slower. */
const TRANSCRIBED_TIMEOUT_MS = patient(300_000);
const copy = messages.long;

const gate = (page: Page) => page.getByTestId("source-gate");
const problem = (page: Page) => page.getByTestId("source-problem");

test("from /app: the quiet link, and a 2-minute file's panel with its words unchanged and Make shorts from it", { tag: NO_GPU }, async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await installFakeApi(page);
  await routeAssets(page);
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  const link = page.getByTestId("long-link");
  await expect(link).toHaveText(copy.linkFromClip);
  await expect(link).toHaveAttribute("href", "/app/long");
  await capture(page, "app-link");
  expect(await axe(page)).toEqual([]);

  // DC-1: today's words for a clip over 90 s, and one more button.
  await dropFile(page, genFixture("rej_too_long_91s.mp4"));
  const panel = page.getByTestId("clip-rejected");
  await expect(panel).toBeVisible({ timeout: 30_000 });
  const words = messages.rejections.REJECT_DURATION;
  await expect(panel.getByRole("heading")).toHaveText(words.title);
  await expect(panel).toContainText(words.body);
  await expect(panel).toContainText(words.action);
  await expect(panel.getByTestId("make-shorts")).toHaveText(copy.makeShortsInstead);
  await capture(page, "app-too-long");
  expect(await axe(page)).toEqual([]);

  // The button carries the same file to the long page, which asks a signed-out person to sign in.
  await panel.getByTestId("make-shorts").click();
  await expect(page).toHaveURL(/\/app\/long$/);
  await expect(gate(page)).toHaveAttribute("data-reason", "signed_out", { timeout: 30_000 });
  await expect(gate(page)).toContainText(copy.gate.signedOut);
  await expect(gate(page).getByRole("button", { name: copy.gate.signIn, exact: true })).toBeVisible();
  await capture(page, "gate-signed-out");
  expect(await axe(page)).toEqual([]);
});

test("the long page, empty: its own words, no sample clip, no transcription card", { tag: NO_GPU }, async ({ page }) => {
  await openLong(page, {});
  const zone = page.getByTestId("drop-zone");
  await expect(zone).toContainText(copy.dropTitle);
  await expect(page.getByTestId("long-page")).toContainText(copy.dropHint);
  await expect(page.getByTestId("transcription-choice")).toHaveCount(0);
  await expect(page.getByRole("button", { name: messages.dropZone.sampleButton })).toHaveCount(0);
  await capture(page, "empty");
  expect(await axe(page)).toEqual([]);
});

test("the gate: checking until the server says whether it transcribes, then the line on what is sent", { tag: NO_GPU }, async ({ page }) => {
  // The page's first /healthz, start-app's, which says whether cloud
  // transcription will try, answers after 9 s; the session's own is answered
  // at once, so the person is signed in while the gate checks.
  await page.setViewportSize({ width: 1280, height: 800 });
  await installFakeApi(page, signedInCloud());
  let first = true;
  await page.route("**/api/v1/healthz", async (route) => {
    if (first) {
      first = false;
      await page.waitForTimeout(9_000).catch(() => undefined);
    }
    await route.fallback().catch(() => undefined);
  });
  await routeAssets(page);
  await page.goto("/app/long");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  await dropFile(page, LONG);
  await expect(gate(page)).toHaveAttribute("data-reason", "checking", { timeout: 30_000 });
  await expect(gate(page)).toContainText(copy.gate.checking);
  await capture(page, "gate-checking");
  expect(await axe(page)).toEqual([]);
  await expect(gate(page)).toHaveAttribute("data-reason", "ready", { timeout: 30_000 });
  await expect(gate(page).getByRole("button", { name: copy.gate.start, exact: true })).toBeVisible();
});

test("the gate: off when the server says cloud transcription is off", { tag: NO_GPU }, async ({ page }) => {
  await openLong(page, { session: {} });
  await dropFile(page, LONG);
  await expect(gate(page)).toHaveAttribute("data-reason", "unavailable", { timeout: 60_000 });
  await expect(gate(page)).toContainText(copy.gate.unavailable);
  await expect(page.getByTestId("source-start")).toHaveCount(0);
  await capture(page, "gate-unavailable");
  expect(await axe(page)).toEqual([]);
});

test("a 12-minute video, read in place: windows in order, the transcript, and nothing of the video sent", { tag: NO_GPU }, async ({ page, context }) => {
  const sent: { url: string; window: string | undefined; type: string | undefined; body: Buffer }[] = [];
  context.on("request", (request) => {
    sent.push({
      url: request.url(),
      window: request.headers()["x-offcut-window"],
      type: request.headers()["content-type"],
      body: request.postDataBuffer() ?? Buffer.alloc(0),
    });
  });
  // A second per window keeps each stage on screen long enough to read it.
  await openLong(page, signedInCloud(1_000));
  await dropFile(page, LONG);
  await expect(gate(page)).toHaveAttribute("data-reason", "ready", { timeout: 60_000 });
  await expect(gate(page)).toContainText(copy.gate.sends);
  await capture(page, "gate-ready");
  expect(await axe(page)).toEqual([]);
  await page.getByTestId("source-start").click();

  const progress = page.getByTestId("source-progress");
  await expect(progress).toBeVisible({ timeout: 30_000 });
  await expect(progress).toContainText(copy.stage.audio);
  await expect(progress).toContainText(copy.keepOpen);
  await expect(page).toHaveTitle(/^\d+% - Offcut$/, { timeout: 60_000 });
  await capture(page, "progress-audio");
  expect(await axe(page)).toEqual([]);
  await expect(progress).toHaveAttribute("data-status", "transcribing", { timeout: TRANSCRIBED_TIMEOUT_MS });
  await capture(page, "progress-transcribing");
  expect(await axe(page)).toEqual([]);

  const transcript = page.getByTestId("source-transcript");
  await expect(transcript).toBeVisible({ timeout: TRANSCRIBED_TIMEOUT_MS });
  await expect(transcript.getByRole("heading")).toHaveText(copy.done.title);
  await expect(transcript).toContainText(copy.done.next);
  await expect(page).not.toHaveTitle(/%/);
  await capture(page, "transcript");
  expect(await axe(page)).toEqual([]);

  // The master transcript: every word of the nine loops, none lost or doubled at a seam.
  const words = await wordsShown(page);
  expect(words).toHaveLength(LONG_12MIN_WORDS);
  const first = await page.getByTestId("source-sentences").locator("li > span:nth-child(2)").first().textContent();
  expect(first).toBe(firstSentence());
  expect(await sourceStoreCount(page, "masterTranscripts")).toBe(1);
  expect(await sourceStoreCount(page, "sourceTranscripts")).toBe(windowTotal());

  // Each window once, in order of index, marked, as Ogg under 1 MiB.
  const windows = sent.filter((request) => request.url.endsWith("/api/v1/transcribe"));
  const total = windowTotal();
  expect(windows.map((request) => request.window).sort()).toEqual(
    Array.from({ length: total }, (_, index) => `${String(index)}/${String(total)}`).sort(),
  );
  for (const request of windows) {
    expect(request.type).toBe("audio/ogg");
    expect(request.body.length).toBeGreaterThan(0);
    expect(request.body.length).toBeLessThan(1024 * 1024);
    expect(request.body.subarray(0, 4).toString("latin1")).toBe("OggS");
  }
  // No request carries any of 20 pieces of 32 bytes of the video, spread over the file.
  const video = readFileSync(LONG);
  const size = statSync(LONG).size;
  const pieces = Array.from({ length: 20 }, (_, n) => video.subarray(Math.floor(((n + 0.5) * size) / 20), Math.floor(((n + 0.5) * size) / 20) + 32));
  for (const request of sent) {
    for (const piece of pieces) {
      expect(request.body.includes(piece), `${request.url} carries bytes of the video`).toBe(false);
    }
  }
});

test("a file of 90 s or less on the long page opens on /app as a clip", async ({ page }) => {
  await ensureModelCached(page.context());
  await openLong(page, {});
  await dropClip(page, genFixture("speech/speech_60s_portrait.mp4"));
  await expect(page).toHaveURL(/\/app$/, { timeout: 30_000 });
  await expect(page.getByText(copy.shortHandOver)).toBeVisible({ timeout: 30_000 });
  const canvas = page.getByTestId("preview-canvas");
  await expect(canvas.or(page.getByTestId("clip-failed"))).toBeVisible({ timeout: patient(240_000) });
  await expect(canvas, "the clip was processed").toBeVisible();
});

test("REJECT_DURATION over 45 minutes, in the long page's words", { tag: NO_GPU }, async ({ page }) => {
  await openLong(page, {});
  await dropFile(page, genFixture("rej_too_long_46min.mp4"));
  await expect(problem(page)).toHaveAttribute("data-code", "REJECT_DURATION", { timeout: 60_000 });
  await expect(problem(page).getByRole("heading")).toHaveText(longPageDuration.title);
  await expect(problem(page)).toContainText(longPageDuration.action);
  await capture(page, "rejected-duration");
  expect(await axe(page)).toEqual([]);
});

test("REJECT_RESOLUTION over 3840 pixels", { tag: NO_GPU }, async ({ page }) => {
  await openLong(page, {});
  await dropFile(page, genFixture("rej_8k.mp4"));
  await expect(problem(page)).toHaveAttribute("data-code", "REJECT_RESOLUTION", { timeout: 60_000 });
  await expect(problem(page).getByRole("heading")).toHaveText(messages.rejections.REJECT_RESOLUTION.title);
  await expect(problem(page)).toContainText("3840");
  expect(await axe(page)).toEqual([]);
});

// Rule 6 of route_probe (HEVC the browser cannot decode) comes before the HDR
// rule, so this case needs a Chrome that decodes HEVC: D1's, through its
// hardware (P171). Not @no-gpu: the hosted runner's Chrome has no HEVC.
test("REJECT_HDR: an HLG video is refused, in its own words", async ({ page }) => {
  await openLong(page, {});
  await dropFile(page, genFixture("rej_hdr_hlg.mov"), "clip.mov");
  await expect(problem(page)).toHaveAttribute("data-code", "REJECT_HDR", { timeout: 60_000 });
  await expect(problem(page).getByRole("heading")).toHaveText(messages.rejections.REJECT_HDR.title);
  await expect(problem(page)).toContainText(messages.rejections.REJECT_HDR.body);
  await capture(page, "rejected-hdr");
  expect(await axe(page)).toEqual([]);
});
