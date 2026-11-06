// Cloud transcription in a real browser (cloudimplementation §11.4): the
// card, the cloud path with the real adapter's answer for this clip's
// audio, the silence guard, the remembered choice, the fallbacks, an
// export from a cloud transcript, and the identity that matters most: with
// the server's switch on, a signed-out person's clip is processed exactly
// as before. The fake API never reaches Groq: `/transcribe` answers with
// `fixtures/cloud/speech_60s_portrait.json`, which the real adapter wrote
// from Groq's real answer (cloud CL Entry 4).

import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { type BrowserContext, expect, type Page, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import type { TranscribeResponse } from "../src/gen/api";
import { type FakeApi, type FakeApiOptions, installFakeApi } from "./helpers/fake-api";
import {
  axe,
  dropClip,
  ensureModelCached,
  expectAllChecks,
  genFixture,
  opfsList,
  patient,
  readIndexedDb,
  routeAssets,
  sendQueuedAnalytics,
  sourceDurationMs,
  verifyMp4,
} from "./helpers/fixtures";

const CLIP = genFixture("speech/speech_60s_portrait.mp4");
const ANSWER = JSON.parse(readFileSync(path.join(import.meta.dirname, "fixtures/cloud/speech_60s_portrait.json"), "utf8")) as TranscribeResponse;
const READY_TIMEOUT_MS = patient(240_000);
/** Groq invents "Thank you." in the silence at the clip's end (TE-C2); the silence guard drops both words (D-382). */
const INVENTED = 2;
/** The flashes `verify_mp4.py` compares in this fixture's exports: its two white frames and one more it reads as a flash (read 2026-11-06). */
const FLASHES = 3;
const copy = messages.transcription;

type Stored = { value: { transcript: { words: { text: string }[]; model_version: string }; prosody: unknown } };

const card = (page: Page) => page.getByTestId("transcription-choice");
const radio = (page: Page, name: string) => card(page).getByRole("radio", { name, exact: true });

/** A page of its own, at /app, with the fake API as `options` say and the assets served from this machine. */
async function openApp(context: BrowserContext, options: FakeApiOptions): Promise<{ page: Page; api: FakeApi }> {
  const page = await context.newPage();
  await page.setViewportSize({ width: 1280, height: 800 });
  const api = await installFakeApi(page, options);
  await routeAssets(page);
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  return { page, api };
}

/** Drops the clip and waits until it can be played. */
async function processed(page: Page): Promise<void> {
  await dropClip(page, CLIP);
  const canvas = page.getByTestId("preview-canvas");
  await expect(canvas.or(page.getByTestId("clip-failed"))).toBeVisible({ timeout: READY_TIMEOUT_MS });
  await expect(canvas, "the clip was processed").toBeVisible();
}

/** What the device keeps of the one clip: its transcript and prosody. */
async function stored(page: Page): Promise<Stored["value"]> {
  const clips = await opfsList(page, "clips");
  expect(clips).toHaveLength(1);
  const record = (await readIndexedDb(page, "transcripts", clips[0] ?? "")) as Stored | undefined;
  if (record === undefined) {
    throw new Error("no transcript was stored");
  }
  return record.value;
}

async function newContext(browser: import("@playwright/test").Browser, baseURL: string | undefined, cached: boolean): Promise<BrowserContext> {
  const context = await browser.newContext({ acceptDownloads: true, ...(baseURL === undefined ? {} : { baseURL }) });
  if (cached) {
    await ensureModelCached(context);
  }
  return context;
}

test("identity: with the server's switch on, a signed-out person's clip is processed exactly as before", async ({ browser }, info) => {
  const { baseURL } = info.project.use;
  // Before: the server says nothing of cloud transcription, as every existing spec's fake.
  const before = await newContext(browser, baseURL, true);
  const old = await openApp(before, {});
  await expect(card(old.page)).toHaveCount(0);
  await processed(old.page);
  const reference = await stored(old.page);
  await before.close();

  // After: the server says yes, nobody is signed in.
  const after = await newContext(browser, baseURL, true);
  const now = await openApp(after, { healthz: { cloudTranscribe: true } });
  await expect(card(now.page)).toBeVisible();
  await expect(radio(now.page, copy.device)).toHaveAttribute("aria-checked", "true");
  await expect(now.page.getByRole("button", { name: copy.signIn, exact: true })).toBeVisible();
  await expect(now.page.getByTestId("transcription-line")).toHaveText(copy.signedOutLine);
  expect(await axe(now.page)).toEqual([]);
  await processed(now.page);
  expect(await stored(now.page)).toEqual(reference);
  expect(now.api.requests.filter((r) => r.path === "/transcribe")).toEqual([]);
  expect(now.api.transcribeBodies).toEqual([]);
  await after.close();
});

test.describe("signed in, cloud by default", () => {
  test.describe.configure({ mode: "serial" });

  let context: BrowserContext;
  let page: Page;
  let api: FakeApi;
  let scratch: string;

  test.beforeAll(async ({ browser }, info) => {
    scratch = mkdtempSync(path.join(tmpdir(), "offcut-cloud-"));
    // No model in this context: the cloud path must not need one.
    context = await newContext(browser, info.project.use.baseURL, false);
    ({ page, api } = await openApp(context, {
      session: { email: "someone@example.com" },
      healthz: { cloudTranscribe: true },
      transcribe: { response: ANSWER },
    }));
  });

  test.afterAll(async () => {
    await context.close();
    rmSync(scratch, { recursive: true, force: true });
  });

  test("the card says Cloud, the clip is sent once as Ogg Opus, and its captions are the cloud's", async () => {
    await expect(radio(page, copy.cloud)).toHaveAttribute("aria-checked", "true");
    await expect(page.getByTestId("transcription-line")).toHaveText(copy.cloudLine);
    expect(await axe(page)).toEqual([]);

    const sent = page.waitForRequest((request) => request.url().endsWith("/api/v1/transcribe"));
    await dropClip(page, CLIP);
    const request = await sent;
    expect(request.headers()["content-type"]).toBe("audio/ogg");
    await expect(page.getByText(copy.processingCloud)).toBeVisible();
    await expect(page.getByTestId("preview-canvas")).toBeVisible({ timeout: READY_TIMEOUT_MS });

    // One request, an Ogg file under 1 MiB, as long as the clip.
    expect(api.transcribeBodies).toHaveLength(1);
    const body = api.transcribeBodies[0] ?? Buffer.alloc(0);
    expect(body.length).toBeLessThan(1024 * 1024);
    expect(body.subarray(0, 4).toString("latin1")).toBe("OggS");
    const file = path.join(scratch, "sent.ogg");
    writeFileSync(file, body);
    const probe = execFileSync(
      "ffprobe",
      ["-v", "error", "-show_entries", "stream=codec_name,channels:format=duration", "-of", "default=nw=1", file],
      { encoding: "utf8" },
    );
    expect(probe).toContain("codec_name=opus");
    expect(probe).toContain("channels=1");
    const seconds = Number(/duration=([0-9.]+)/.exec(probe)?.[1]);
    expect(Math.abs(seconds * 1000 - sourceDurationMs(CLIP))).toBeLessThan(40);

    // The cloud's words, less the two the silence guard drops; no model was needed.
    const { transcript } = await stored(page);
    expect(transcript.model_version).toBe(ANSWER.model);
    expect(transcript.words).toHaveLength(ANSWER.words.length - INVENTED);
    expect(transcript.words.at(-1)?.text).toBe("month.");
    expect(transcript.words.slice(0, 2).map((w) => w.text)).toEqual(["Last", "year,"]);
    await expect(page.getByTestId("model-download")).toHaveCount(0);

    await sendQueuedAnalytics(page);
    expect(api.eventsOf("asr_cloud")).toEqual([{ name: "asr_cloud", props: { outcome: "ok", duration_ms: expect.any(Number) } }]);
    const asr = api.eventsOf("stage_timing").filter((e) => (e as { props: { stage: string } }).props.stage === "asr");
    expect(asr).toEqual([{ name: "stage_timing", props: { stage: "asr", duration_ms: expect.any(Number), asr_backend: "cloud" } }]);
  });

  test("an export from a cloud transcript passes all 11 checks", async () => {
    const arriving = page.waitForEvent("download", { timeout: patient(240_000) });
    await page.getByRole("button", { name: messages.export.button, exact: true }).click();
    const download = await arriving;
    const file = path.join(scratch, "cloud.mp4");
    await download.saveAs(file);
    expectAllChecks(verifyMp4(file, "free", sourceDurationMs(CLIP), CLIP), { flashes: FLASHES });
  });
});

test("the choice is remembered, and On this device sends nothing", async ({ browser }, info) => {
  const context = await newContext(browser, info.project.use.baseURL, true);
  const options: FakeApiOptions = {
    session: { email: "someone@example.com" },
    healthz: { cloudTranscribe: true },
    transcribe: { response: ANSWER },
  };
  const first = await openApp(context, options);
  await radio(first.page, copy.device).click();
  await expect(first.page.getByTestId("transcription-line")).toHaveText(copy.deviceLine);
  await first.page.close();

  const { page, api } = await openApp(context, options);
  await expect(radio(page, copy.device)).toHaveAttribute("aria-checked", "true");
  expect(await axe(page)).toEqual([]);
  api.forget();
  await processed(page);
  // While processing: nothing but analytics (P-5).
  expect(api.requests.filter((r) => r.path !== "/events")).toEqual([]);
  expect(api.transcribeBodies).toEqual([]);
  expect((await stored(page)).transcript.model_version).not.toBe(ANSWER.model);
  await context.close();
});

test.describe("fallbacks: the clip finishes on the device, with one toast", () => {
  for (const [name, options, toast, outcome] of [
    ["429: today's cloud transcriptions are used", { failures: { "/transcribe": { status: 429, code: "rate_limited", retryAfterSecs: 3600 } } }, copy.limitToast, "limited"],
    ["no answer in 30 s", { transcribe: { response: ANSWER, delayMs: 31_000 } }, copy.fallbackToast, "timeout"],
  ] as const) {
    test(name, async ({ browser }, info) => {
      test.slow();
      const context = await newContext(browser, info.project.use.baseURL, true);
      const { page, api } = await openApp(context, {
        session: { email: "someone@example.com" },
        healthz: { cloudTranscribe: true },
        transcribe: { response: ANSWER },
        ...options,
      });
      // The toast comes as the cloud gives up and lasts 5 s: it is looked for then, not at the end.
      await dropClip(page, CLIP);
      await expect(page.getByText(toast)).toBeVisible({ timeout: 45_000 });
      await expect(page.getByTestId("preview-canvas")).toBeVisible({ timeout: READY_TIMEOUT_MS });
      expect((await stored(page)).transcript.model_version).not.toBe(ANSWER.model);
      await sendQueuedAnalytics(page);
      expect(api.eventsOf("asr_cloud")).toEqual([{ name: "asr_cloud", props: { outcome, duration_ms: expect.any(Number) } }]);
      await context.close();
    });
  }
});
