// J6 and J7 in a real browser: the reference clip goes through the pipeline
// and is previewed, against the built app served with the production
// headers. The asset host is answered from `fixtures/.cache/`, the API is a
// fake, and the model is on the device before the page opens.
//
// The clip is processed once. The cases of "one clip" read that one session
// in the order they are written, and the last of them lets the clip go; a
// case that fails stops the ones after it.

import { readFileSync } from "node:fs";
import path from "node:path";
import { inflateSync } from "node:zlib";

import { type BrowserContext, expect, type Locator, type Page, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import { ANALYTICS_EVENT_DOCS } from "../src/gen/api";
import { UNSUPPORTED_REASONS } from "../src/gen/domain";
import { DB_NAME, STORES } from "../src/persistence/schema";
import { type FakeApi, installFakeApi } from "./helpers/fake-api";
import {
  type AssetLog,
  dropClip,
  ensureModelCached,
  opfsList,
  opfsSize,
  referenceClip,
  routeAssets,
  sampleClipPath,
  sourceDurationMs,
} from "./helpers/fixtures";

declare global {
  interface Window {
    /** Every state the feed's list was in, in order. */
    e2eFeed?: string[][];
    /** The length in milliseconds of every main-thread task of 50 ms or more, with the time it started. */
    e2eLongTasks?: { at: number; ms: number }[];
  }
}

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
/** Analytics sends what it has every 10 s; a case waits that long for an event, and a little more. */
const FLUSH_WAIT_MS = 20_000;
const FOUND_PREFIX = messages.feed.eventFound.number_reveal("");

type Recorded = { method: string; url: string; body: string | null };
type Batch = { anon_id: string; events: { name: string; props?: Record<string, unknown> }[] };

const canvas = (page: Page) => page.getByTestId("preview-canvas");
const button = (page: Page, name: string) => page.getByRole("button", { name, exact: true });
const isHttp = (request: Recorded) => /^https?:/.test(request.url);

/** Records what the page does, from before it loads: the feed is on it for a moment only. */
async function watchPage(page: Page): Promise<void> {
  await page.addInitScript(() => {
    window.e2eFeed = [];
    window.e2eLongTasks = [];
    new MutationObserver(() => {
      const lines = [...document.querySelectorAll('[data-testid="feed"] li')].map((line) => line.textContent ?? "");
      if (lines.length > 0 && JSON.stringify(lines) !== JSON.stringify(window.e2eFeed?.at(-1))) {
        window.e2eFeed?.push(lines);
      }
    }).observe(document, { childList: true, subtree: true, characterData: true });
    new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        window.e2eLongTasks?.push({ at: entry.startTime, ms: entry.duration });
      }
    }).observe({ type: "longtask", buffered: true });
  });
}

/** The pixels of a PNG as its file holds them, filter bytes included. */
function pngData(png: Buffer): Buffer {
  const chunks: Buffer[] = [];
  // After the 8 bytes of the signature: length, type, data, checksum.
  for (let at = 8; at + 8 <= png.length; ) {
    const length = png.readUInt32BE(at);
    if (png.toString("latin1", at + 4, at + 8) === "IDAT") {
      chunks.push(png.subarray(at + 8, at + 8 + length));
    }
    at += 12 + length;
  }
  return inflateSync(Buffer.concat(chunks));
}

/** Whether a picture shows more than one flat colour: a flat one holds a handful of different bytes at most. */
function hasDetail(png: Buffer): boolean {
  return new Set(pngData(png)).size > 32;
}

/** Two pictures of an element, `gapMs` apart. */
async function twoPictures(target: Locator, page: Page, gapMs: number): Promise<[Buffer, Buffer]> {
  const first = await target.screenshot();
  await page.waitForTimeout(gapMs);
  return [first, await target.screenshot()];
}

/** The amounts of the README's expected feed: what a "Found:" line may name for the reference clip. */
function expectedAmounts(): string[] {
  const readme = readFileSync(path.resolve(import.meta.dirname, "../../fixtures/speech/README.md"), "utf8");
  const feed = readme.slice(readme.indexOf("**Expected feed.**"));
  return [...feed.slice(0, feed.indexOf("\n## ")).matchAll(/^\| \d+ \| "Found: [^"]+" \| `([^`]+)` \|$/gm)].map((row) => row[1] ?? "");
}

/** The bodies of the page's `POST /api/v1/events`, each checked to be a batch. */
function batches(api: FakeApi): Batch[] {
  return api.requests
    .filter((entry) => entry.path === "/events")
    .map((entry) => {
      expect(entry.method).toBe("POST");
      expect(entry.body).toEqual({ anon_id: expect.stringMatching(UUID), events: expect.any(Array) });
      return entry.body as Batch;
    });
}

// What a property of an event may be: a whole number, a yes or no, or one of
// a fixed list of words. Nothing a person said or named fits any of them.
const INT = "int";
const BOOL = "bool";
type PropKind = typeof INT | typeof BOOL | readonly string[];
const STAGES = ["probe_audio", "asr", "audio_chain", "detect_scene", "render_encode", "mux"];
/** The events this session may send, with every property each may carry. A `?` marks one that may be absent or null. */
const EVENT_PROPS: Record<string, Record<string, PropKind>> = {
  landing_view: { hero_variant: ["outcome", "privacy"] },
  capability_check: {
    result: ["pass", "fail"],
    "unsupported_reason?": UNSUPPORTED_REASONS,
    gpu_vendor: ["intel", "amd", "nvidia", "apple", "qualcomm", "other", "unknown"],
    memory_bucket: ["lt4", "gb4", "gb8plus", "unknown"],
    platform: ["windows", "macos", "linux", "chromeos", "android", "other", "unknown"],
  },
  clip_accepted: { duration_bucket: ["lt30", "lt60", "lte90"], orientation: ["portrait", "landscape"], source: ["user", "sample"] },
  stage_timing: { stage: STAGES, duration_ms: INT, "asr_backend?": ["webgpu", "wasm"] },
  pipeline_done: { total_ms: INT, n_number: INT, n_list: INT, n_from_to: INT, n_keyword: INT },
  preview_played: {},
};

function expectAllowedProps(event: Batch["events"][number]): void {
  const allowed = EVENT_PROPS[event.name];
  expect(allowed, `the event ${event.name} is not one this session may send`).toBeDefined();
  const props = event.props ?? {};
  const known = Object.keys(allowed ?? {}).map((key) => key.replace(/\?$/, ""));
  expect(Object.keys(props).filter((key) => !known.includes(key))).toEqual([]);
  for (const [key, kind] of Object.entries(allowed ?? {})) {
    const optional = key.endsWith("?");
    const value = props[key.replace(/\?$/, "")];
    if (optional && (value === undefined || value === null)) {
      continue;
    }
    if (kind === INT) {
      expect(Number.isInteger(value), `${event.name}.${key} is a whole number`).toBe(true);
    } else if (kind === BOOL) {
      expect(typeof value, `${event.name}.${key} is a yes or no`).toBe("boolean");
    } else {
      expect(kind, `${event.name}.${key}`).toContain(value);
    }
  }
}

test.describe("one clip, from the drop to the preview", () => {
  test.describe.configure({ mode: "serial" });

  let context: BrowserContext;
  let page: Page;
  let api: FakeApi;
  let assets: AssetLog;
  /** Everything the page and its workers asked for, with what a request carried. */
  const requests: Recorded[] = [];
  /** Where in `requests` the processing began and ended: after the drop, and when the player appeared. */
  let processing = { from: 0, to: 0 };
  let processingMs = { from: 0, to: 0 };

  test.beforeAll(async ({ browser }, testInfo) => {
    const { baseURL } = testInfo.project.use;
    context = await browser.newContext(baseURL === undefined ? {} : { baseURL });
    await ensureModelCached(context);
    context.on("request", (request) => requests.push({ method: request.method(), url: request.url(), body: request.postData() }));
    page = await context.newPage();
    api = await installFakeApi(page);
    assets = await routeAssets(page);
    await watchPage(page);
    await page.goto("/app");
    await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });

    await dropClip(page, referenceClip());
    processing.from = requests.length;
    processingMs.from = await page.evaluate(() => performance.now());
    await expect(canvas(page)).toBeVisible({ timeout: 240_000 });
    processing = { ...processing, to: requests.length };
    processingMs = { ...processingMs, to: await page.evaluate(() => performance.now()) };
  });

  test.afterAll(async () => {
    await context.close();
  });

  test("feed: \"Transcribing…\" comes first, a \"Found:\" line follows, and no voice is said to be cleaned", async () => {
    const states = await page.evaluate(() => window.e2eFeed ?? []);
    const last = states.at(-1) ?? [];
    expect(last[0]).toBe(messages.feed.transcribing);
    expect(last.slice(1).filter((line) => line.startsWith(FOUND_PREFIX)).length).toBeGreaterThanOrEqual(1);
    expect(states.flat()).not.toContain(messages.feed.cleaningVoice);
    // A line is added and never taken away or changed: each state begins with the one before it.
    for (const [index, state] of states.entries()) {
      expect(state.slice(0, states[index - 1]?.length ?? 0)).toEqual(states[index - 1] ?? []);
    }
  });

  test("real detections only: every \"Found:\" line names an amount the README expects", async () => {
    const lines = (await page.evaluate(() => window.e2eFeed ?? [])).at(-1) ?? [];
    const amounts = lines.filter((line) => line.startsWith(FOUND_PREFIX)).map((line) => line.slice(FOUND_PREFIX.length));
    const expected = expectedAmounts();
    expect(expected.length).toBeGreaterThan(0);
    expect(amounts.filter((amount) => !expected.includes(amount))).toEqual([]);
    // And no line that is neither of the two kinds V2 has.
    expect(lines.filter((line) => line !== messages.feed.transcribing && !line.startsWith(FOUND_PREFIX))).toEqual([]);
  });

  test("quiet processing: from the drop to the player the page sends analytics and nothing else", ({ baseURL }) => {
    const origin = new URL(baseURL ?? "").origin;
    const during = requests.slice(processing.from, processing.to).filter(isHttp);
    // The model is on the device: nothing is asked of the asset host, or of any other host.
    expect(assets.modelRequests()).toEqual([]);
    expect(during.filter((request) => new URL(request.url).origin !== origin)).toEqual([]);
    // What the page fetches besides are files of its own build: its workers,
    // the two WASM bundles and the recognizer's runtime.
    const ownFile = (request: Recorded) => request.method === "GET" && /^\/(assets|ort)\//.test(new URL(request.url).pathname);
    const analytics = (request: Recorded) => request.method === "POST" && request.url === `${origin}/api/v1/events`;
    expect(during.filter((request) => !ownFile(request) && !analytics(request))).toEqual([]);
  });

  test("OPFS: the clip and its audio are on the device, under names that are not the file's", async () => {
    const clips = await opfsList(page, "clips");
    expect(clips).toEqual([expect.stringMatching(UUID)]);
    const dir = `clips/${clips[0] ?? ""}`;
    expect(await opfsList(page, dir)).toEqual(["out48.f32", "source"]);
    // 48 kHz mono, four bytes a sample, exactly as long as the recording (D-34).
    expect(await opfsSize(page, `${dir}/out48.f32`)).toBe(Math.round(sourceDurationMs(referenceClip()) * 48) * 4);
    expect(await opfsSize(page, `${dir}/source`)).toBeGreaterThan(0);
    // The name the file was dropped under is in no entry of the device (P-11).
    const names = await page.evaluate(async () => {
      const all: string[] = [];
      const walk = async (handle: FileSystemDirectoryHandle): Promise<void> => {
        for await (const [name, entry] of handle.entries()) {
          all.push(name);
          if (entry.kind === "directory") {
            await walk(entry);
          }
        }
      };
      await walk(await navigator.storage.getDirectory());
      return all;
    });
    expect(names.filter((name) => name.includes("clip.mp4"))).toEqual([]);
  });

  test("long tasks: the longest main-thread task while processing is written down", async ({}, testInfo) => {
    const tasks = await page.evaluate(() => window.e2eLongTasks ?? []);
    const during = tasks.filter((task) => task.at >= processingMs.from && task.at <= processingMs.to);
    const longest = Math.round(Math.max(0, ...during.map((task) => task.ms)));
    const line = `longest main-thread task while processing: ${String(longest)} ms (${String(during.length)} tasks of 50 ms or more)`;
    // Written, not asserted: the limit of TS §13.5 is an assumption until M2.4.
    testInfo.annotations.push({ type: "long task", description: line });
    process.stdout.write(`      ${line}\n`);
  });

  test("preview before sign-in: with no token on the device the player plays", async () => {
    const record = await page.evaluate(
      ({ name, store }) =>
        new Promise<unknown>((resolve, reject) => {
          const open = indexedDB.open(name);
          open.onerror = () => {
            reject(new Error("the database could not be opened"));
          };
          open.onsuccess = () => {
            const read = open.result.transaction(store).objectStore(store).get("current");
            read.onsuccess = () => {
              open.result.close();
              resolve(read.result ?? null);
            };
          };
        }),
      { name: DB_NAME, store: STORES.entitlement },
    );
    expect(record).toBeNull();

    await expect(button(page, messages.preview.play)).toBeEnabled({ timeout: 30_000 });
    await button(page, messages.preview.play).click();
    await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 30_000 });
    // Past the first second and a half, in which the speaker has not begun.
    await page.waitForTimeout(2_500);
    const [first, second] = await twoPictures(canvas(page), page, 1_000);
    expect(first.equals(second)).toBe(false);
    expect(hasDetail(first)).toBe(true);
    expect(hasDetail(second)).toBe(true);
  });

  test("pause and resume: a paused preview stands still, and goes on when played", async () => {
    await button(page, messages.preview.pause).click();
    await expect(button(page, messages.preview.play)).toBeVisible({ timeout: 30_000 });
    const [stillOne, stillTwo] = await twoPictures(canvas(page), page, 500);
    expect(stillOne.equals(stillTwo)).toBe(true);

    await button(page, messages.preview.play).click();
    await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 30_000 });
    await page.waitForTimeout(500);
    const [movingOne, movingTwo] = await twoPictures(canvas(page), page, 1_000);
    expect(movingOne.equals(movingTwo)).toBe(false);
    // A frame that was not given back fails the pause, and the page would say so (D-45).
    await expect(page.getByText(messages.editor.failedStub)).toHaveCount(0);
    await button(page, messages.preview.pause).click();
    await expect(button(page, messages.preview.play)).toBeVisible({ timeout: 30_000 });
    await expect(page.getByText(messages.editor.failedStub)).toHaveCount(0);
  });

  test("events: the clip, its three stages, the result, and one preview for two plays", async () => {
    await expect.poll(() => api.eventsOf("preview_played").length, { timeout: FLUSH_WAIT_MS }).toBe(1);
    expect(api.eventsOf("clip_accepted")).toEqual([
      { name: "clip_accepted", props: { source: "user", orientation: "landscape", duration_bucket: "lte90" } },
    ]);
    const timings = api.eventsOf("stage_timing") as { props: Record<string, unknown> }[];
    expect(timings.map((event) => event.props.stage)).toEqual(["probe_audio", "asr", "detect_scene"]);
    for (const event of timings) {
      expect(Number.isInteger(event.props.duration_ms)).toBe(true);
      expect(event.props.asr_backend === undefined || event.props.asr_backend === null).toBe(event.props.stage !== "asr");
    }
    expect(["webgpu", "wasm"]).toContain(timings[1]?.props.asr_backend);
    expect(api.eventsOf("pipeline_done")).toEqual([
      { name: "pipeline_done", props: { total_ms: expect.any(Number), n_number: expect.any(Number), n_list: 0, n_from_to: 0, n_keyword: 0 } },
    ]);
    const done = api.eventsOf("pipeline_done")[0] as { props: { n_number: number } };
    expect(done.props.n_number).toBeGreaterThanOrEqual(1);
    // The model was on the device: nothing was downloaded, and nothing failed.
    expect(api.eventsOf("model_download")).toEqual([]);
    expect(api.eventsOf("client_error")).toEqual([]);
    // Two plays, one event: it is sent for the first play of a preview.
    expect(api.eventsOf("preview_played")).toEqual([{ name: "preview_played" }]);
  });

  test("event shape: every batch is a batch, no property is free text, and nothing of the feed is sent", () => {
    const names: readonly string[] = ANALYTICS_EVENT_DOCS.map((doc) => doc.name);
    const all = batches(api);
    expect(all.length).toBeGreaterThan(0);
    expect(new Set(all.map((batch) => batch.anon_id)).size).toBe(1);
    for (const event of all.flatMap((batch) => batch.events)) {
      expect(Object.keys(event).filter((key) => key !== "name" && key !== "props")).toEqual([]);
      expect(names).toContain(event.name);
      expectAllowedProps(event);
    }

    // Nothing the feed showed is in any request the page or its workers made:
    // not in an address, not in a body (INV-2).
    const amounts = expectedAmounts();
    const feedText = [messages.feed.transcribing, FOUND_PREFIX.trim(), ...amounts];
    const carrying = requests.filter(isHttp).filter((request) => {
      // An address is read as it is and as it reads once its escapes are undone.
      const sent = `${request.url}\n${decodeURI(request.url)}\n${request.body ?? ""}`;
      return feedText.some((text) => text !== "" && sent.includes(text));
    });
    expect(carrying.map((request) => `${request.method} ${request.url}`)).toEqual([]);
  });

  test("start over: the drop zone comes back and the clip is gone from the device", async () => {
    await button(page, messages.editor.startOver).click();
    await expect(page.getByTestId("drop-zone")).toBeVisible();
    await expect(canvas(page)).toHaveCount(0);
    await expect.poll(() => opfsList(page, "clips"), { timeout: 30_000 }).toEqual([]);
    // The page is on the editor still, ready for the next clip.
    await expect(page).toHaveURL(/\/app$/);
    await expect(page.getByText(messages.editor.failedStub)).toHaveCount(0);
  });
});

test("sample clip: the button fetches the clip from the asset host and imports it", async ({ page }) => {
  const api = await installFakeApi(page);
  const assets = await routeAssets(page);
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });

  await button(page, messages.dropZone.sampleButton).click();
  await expect(page.getByTestId("feed")).toBeVisible({ timeout: 60_000 });
  const sample = assets.requests.filter((request) => request.path === sampleClipPath());
  expect(sample.map((request) => request.method)).toEqual(["GET"]);
  expect(sample[0]?.headers.filter((header) => header.name === "range" || header.name === "cookie")).toEqual([]);
  await expect.poll(() => api.eventsOf("clip_accepted"), { timeout: FLUSH_WAIT_MS }).toEqual([
    { name: "clip_accepted", props: { source: "sample", orientation: "landscape", duration_bucket: "lte90" } },
  ]);
});
