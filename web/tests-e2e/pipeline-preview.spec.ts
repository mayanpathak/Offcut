// J6 and J7 in a real browser: the reference clip goes through the pipeline
// and is previewed, against the built app served with the production
// headers. The asset host is answered from `fixtures/.cache/`, the API is a
// fake, and the model is on the device before the page opens.
//
// The clip is processed once. The cases of "one clip" read that one session
// in the order they are written, and the last of them lets the clip go; a
// case that fails stops the ones after it. The case of the founder's second
// recording runs on its own after them, on the development machine only, so
// that it cannot hide one of theirs (D-99).
//
// V4 (v4implementation D-154, D-156, D-160, D-193): "Preview plays" also
// reads the player's statistics and "Events" the plan's rows; three cases
// are new: the statistics of a 5 s play, the long-task limits while
// processing and editing, and a clip with nothing to show.

import { readFileSync } from "node:fs";
import path from "node:path";
import { inflateSync } from "node:zlib";

import { type BrowserContext, expect, type Locator, type Page, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import { ANALYTICS_EVENT_DOCS } from "../src/gen/api";
import { REJECT_REASONS, UNSUPPORTED_REASONS } from "../src/gen/domain";
import { DB_NAME, STORES } from "../src/persistence/schema";
import { type FakeApi, installFakeApi } from "./helpers/fake-api";
import {
  type AssetLog,
  NO_GPU,
  dragCaptions,
  dropClip,
  editWord,
  ensureModelCached,
  genFixture,
  openTab,
  openWord,
  opfsList,
  opfsSize,
  patient,
  player,
  previewStats,
  readIndexedDb,
  referenceClip,
  routeAssets,
  sampleClipPath,
  seekTo,
  setStyle,
  sourceDurationMs,
  trackSaves,
  transcriptWords,
  v3TestClip,
  wordField,
} from "./helpers/fixtures";

type WasmCall = { at: number; end: number; bundle: string };

declare global {
  interface Window {
    /** Every state the feed's list was in, in order. */
    e2eFeed?: string[][];
    /** The length in milliseconds of every main-thread task of 50 ms or more, with the time it started. */
    e2eLongTasks?: { at: number; ms: number }[];
    /** Every `WebAssembly.instantiate` or `instantiateStreaming` on the main thread: when, and of which file. */
    e2eWasm?: WasmCall[];
  }
}

/** The limits of a main-thread task: while processing (D-156, TS §13.5) and while editing (D-193). */
const PROCESSING_TASK_MS = 100;
const EDITING_TASK_MS = 50;
/** A play of this long draws more than this many frames at 30 fps (D-155's statistics). */
const STATS_PLAY_MS = 5_000;
const STATS_MIN_FRAMES = 100;

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
    // The one thing a long task is excused for: the first instantiation of a
    // WASM bundle on this thread (D-156). Each is recorded with its file.
    window.e2eWasm = [];
    for (const name of ["instantiate", "instantiateStreaming"] as const) {
      const original = WebAssembly[name] as (...args: unknown[]) => Promise<unknown>;
      (WebAssembly as unknown as Record<string, unknown>)[name] = (...args: unknown[]) => {
        const at = performance.now();
        const result = original.apply(WebAssembly, args);
        void Promise.resolve(args[0])
          .then((source) => (source instanceof Response ? source.url : `${name} of bytes`))
          .catch(() => `${name} of a failed fetch`)
          .then(async (bundle) => {
            await result.catch(() => undefined);
            window.e2eWasm?.push({ at, end: performance.now(), bundle });
          });
        return result;
      };
    }
  });
}

/**
 * The long tasks that started between `from` and `to`, less those that
 * overlap the first instantiation of a WASM bundle on the main thread
 * (D-156): nothing else is excused.
 */
function tasksBetween(tasks: { at: number; ms: number }[], wasm: WasmCall[], from: number, to: number): { at: number; ms: number }[] {
  const first = new Map<string, WasmCall>();
  for (const call of [...wasm].sort((a, b) => a.at - b.at)) {
    if (!first.has(call.bundle)) {
      first.set(call.bundle, call);
    }
  }
  const excused = (task: { at: number; ms: number }) => [...first.values()].some((call) => task.at <= call.end && task.at + task.ms >= call.at);
  return tasks.filter((task) => task.at >= from && task.at <= to && !excused(task));
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

const README = path.resolve(import.meta.dirname, "../../fixtures/speech/README.md");

/** One "Found:" line the README expects, and whether the clip may lack it. */
type ExpectedLine = { line: string; optional: boolean };

/**
 * The "Found:" lines of the README's "Expected feed of V3" for the reference
 * clip, in time order (fixtures/speech/README.md, v3implementation §17.7). A
 * row marked "(only if …)" is a line the clip may lack: the KeywordPop.
 */
function expectedFeedV3(): ExpectedLine[] {
  const readme = readFileSync(README, "utf8");
  const feed = readme.slice(readme.indexOf("**Expected feed of V3.**"));
  return [...feed.slice(0, feed.indexOf("\n**", 2)).matchAll(/^\| \d+ \| "(Found: [^"]+)"( \(only if [^)]*\))? \|$/gm)].map((row) => ({
    line: row[1] ?? "",
    optional: row[2] !== undefined,
  }));
}

/**
 * The "Found:" lines the README expects for the second recording, in time
 * order ("The V3 test clip as recognized", its "Expected feed"). The README
 * says the KeywordPop may be absent on another run: its line is the one that
 * names a word, where the others name a quantity (a digit or a currency sign).
 */
function expectedFeedOfTestClip(): ExpectedLine[] {
  const readme = readFileSync(README, "utf8");
  const section = readme.slice(readme.indexOf("## The V3 test clip as recognized"));
  const paragraph = section.slice(section.indexOf("**Expected feed**"));
  const lines = [...paragraph.slice(0, paragraph.indexOf("\n")).matchAll(/"(Found: [^"]+)"/g)].map((match) => match[1] ?? "");
  return lines.map((line) => ({ line, optional: !/[\d$€£₹]/.test(line) }));
}

/** The feed's last state, from the recorder installed before the page loaded. */
async function lastFeed(page: Page): Promise<string[]> {
  return (await page.evaluate(() => window.e2eFeed ?? [])).at(-1) ?? [];
}

/**
 * The feed of a processed clip: "Cleaning voice" and "Transcribing…" first,
 * in either order (the chain starts before the recognizer, D-90), then the
 * "Found:" lines, which are those of `expected` in its order, each line the
 * clip may lack being present or absent.
 */
function expectFeed(lines: string[], expected: ExpectedLine[]): void {
  expect(expected.length).toBeGreaterThan(0);
  expect(lines.slice(0, 2).sort()).toEqual([messages.feed.cleaningVoice, messages.feed.transcribing].sort());
  const found = lines.slice(2);
  expect(found.filter((line) => !line.startsWith(FOUND_PREFIX))).toEqual([]);
  // Every line is expected, and none comes out of its order.
  expect(found).toEqual(expected.map((row) => row.line).filter((line) => found.includes(line)));
  // And every line that is not optional is there.
  expect(expected.filter((row) => !row.optional && !found.includes(row.line)).map((row) => row.line)).toEqual([]);
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
  clip_rejected: { reject_reason: REJECT_REASONS },
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
    await trackSaves(page);
    await page.goto("/app");
    await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });

    await dropClip(page, referenceClip());
    processing.from = requests.length;
    processingMs.from = await page.evaluate(() => performance.now());
    // The player, or the page that says the clip could not be processed: that one ends the wait at once.
    await expect(canvas(page).or(page.getByTestId("clip-failed")).or(page.getByTestId("clip-rejected"))).toBeVisible({ timeout: patient(240_000) });
    await expect(canvas(page), "the clip was processed").toBeVisible();
    processing = { ...processing, to: requests.length };
    processingMs = { ...processingMs, to: await page.evaluate(() => performance.now()) };
  });

  test.afterAll(async () => {
    await context.close();
  });

  // V3 (D-91): the voice is cleaned beside the recognizer, so its line comes
  // first or second, and the "Found:" lines follow in time order.
  test("feed: \"Cleaning voice\" and \"Transcribing…\" come first, then the \"Found:\" lines in time order", async () => {
    const states = await page.evaluate(() => window.e2eFeed ?? []);
    const last = states.at(-1) ?? [];
    expect(last.slice(0, 2).sort()).toEqual([messages.feed.cleaningVoice, messages.feed.transcribing].sort());
    expect(last.slice(2).length).toBeGreaterThanOrEqual(1);
    expect(last.slice(2).filter((line) => !line.startsWith(FOUND_PREFIX))).toEqual([]);
    // In time order: the order of the README's expected feed.
    const order = expectedFeedV3().map((row) => row.line);
    expect(last.slice(2)).toEqual(order.filter((line) => last.includes(line)));
    // A line is added and never taken away or changed: each state begins with the one before it.
    for (const [index, state] of states.entries()) {
      expect(state.slice(0, states[index - 1]?.length ?? 0)).toEqual(states[index - 1] ?? []);
    }
  });

  // V3 (D-91): every line is one of the README's V3 expected feed; the four
  // that are always expected are all there; "Found: brutal" may be or not.
  test("real detections only: every \"Found:\" line is one the README expects of V3, and the four sure ones are there", async () => {
    expectFeed(await lastFeed(page), expectedFeedV3());
    expect(expectedFeedV3().filter((row) => !row.optional)).toHaveLength(4);
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

    // V3: the clip's three records, one each, under its id (v3implementation §19.17).
    const clipId = clips[0] ?? "";
    const counts = await page.evaluate(
      ({ name, stores }) =>
        new Promise<number[]>((resolve, reject) => {
          const open = indexedDB.open(name);
          open.onerror = () => {
            reject(new Error("the database could not be opened"));
          };
          open.onsuccess = () => {
            const tx = open.result.transaction(stores);
            const reads = stores.map((store) => tx.objectStore(store).count());
            tx.oncomplete = () => {
              open.result.close();
              resolve(reads.map((read) => read.result));
            };
          };
        }),
      { name: DB_NAME, stores: [STORES.clips, STORES.transcripts, STORES.edits] },
    );
    expect(counts).toEqual([1, 1, 1]);
    const stored = (await readIndexedDb(page, "transcripts", clipId)) as
      | { value: { transcript: { words: unknown[] }; prosody: { per_word: unknown[] } } }
      | undefined;
    expect(stored?.value.transcript.words.length).toBeGreaterThan(0);
    expect(stored?.value.prosody.per_word.length).toBe(stored?.value.transcript.words.length);
    expect(await readIndexedDb(page, "clips", clipId)).toMatchObject({ value: { clipId } });
    expect(await readIndexedDb(page, "edits", clipId)).toMatchObject({ value: { editState: expect.any(Object) } });
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
    // V4 (D-160): no play has drawn yet, so the player carries no statistics.
    expect(Number.isNaN((await previewStats(page)).frames)).toBe(true);
    await button(page, messages.preview.play).click();
    await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 30_000 });
    // Past the first second and a half, in which the speaker has not begun.
    await page.waitForTimeout(2_500);
    const [first, second] = await twoPictures(canvas(page), page, 1_000);
    expect(first.equals(second)).toBe(false);
    expect(hasDetail(first)).toBe(true);
    expect(hasDetail(second)).toBe(true);

    // V4 (D-155, D-160): the play's statistics are on the player once it stops; it plays on after.
    await button(page, messages.preview.pause).click();
    await expect(button(page, messages.preview.play)).toBeVisible({ timeout: 30_000 });
    await expect.poll(async () => (await previewStats(page)).frames, { timeout: 10_000 }).toBeGreaterThan(0);
    const stats = await previewStats(page);
    expect(Number.isFinite(stats.late)).toBe(true);
    expect(Number.isFinite(stats.maxDriftMs)).toBe(true);
    await button(page, messages.preview.play).click();
    await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 30_000 });
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
    await expect(page.getByText(messages.editor.failedStub).or(page.getByTestId("preview-failed"))).toHaveCount(0);
    await button(page, messages.preview.pause).click();
    await expect(button(page, messages.preview.play)).toBeVisible({ timeout: 30_000 });
    await expect(page.getByText(messages.editor.failedStub).or(page.getByTestId("preview-failed"))).toHaveCount(0);
  });

  // V4 (D-155): what a play drew is on the player, never sent.
  test("statistics: after a 5 s play the player says it drew more than 100 frames, and how far they drifted", async () => {
    // From the start, so that the clip does not end within the play.
    await seekTo(page, 0);
    await page.waitForTimeout(300);
    const before = (await previewStats(page)).frames;
    await button(page, messages.preview.play).click();
    await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 30_000 });
    await page.waitForTimeout(STATS_PLAY_MS);
    await button(page, messages.preview.pause).click();
    await expect(button(page, messages.preview.play)).toBeVisible({ timeout: 30_000 });
    await expect.poll(async () => (await previewStats(page)).frames, { timeout: 10_000 }).not.toBe(before);
    const stats = await previewStats(page);
    expect(stats.frames).toBeGreaterThan(STATS_MIN_FRAMES);
    expect(await player(page).getAttribute("data-max-drift-ms")).not.toBeNull();
    expect(Number.isFinite(stats.maxDriftMs)).toBe(true);
    process.stdout.write(`      a 5 s play drew ${String(stats.frames)} frames, ${String(stats.late)} late, the worst drift ${String(stats.maxDriftMs)} ms\n`);
  });

  // V3 (D-91): four stages, the chain's beside the recognizer's, and the
  // reference clip's counts of the four kinds.
  test("events: the clip, its four stages, the result, and one preview for two plays", async ({}, testInfo) => {
    await expect.poll(() => api.eventsOf("preview_played").length, { timeout: FLUSH_WAIT_MS }).toBe(1);
    expect(api.eventsOf("clip_accepted")).toEqual([
      { name: "clip_accepted", props: { source: "user", orientation: "landscape", duration_bucket: "lte90" } },
    ]);
    const timings = api.eventsOf("stage_timing") as { props: Record<string, unknown> }[];
    const stages = timings.map((event) => event.props.stage);
    // `audio_chain` and `asr` run side by side: either may end first.
    expect([
      ["probe_audio", "audio_chain", "asr", "detect_scene"],
      ["probe_audio", "asr", "audio_chain", "detect_scene"],
    ]).toContainEqual(stages);
    for (const event of timings) {
      expect(Number.isInteger(event.props.duration_ms)).toBe(true);
      expect(event.props.asr_backend === undefined || event.props.asr_backend === null).toBe(event.props.stage !== "asr");
    }
    const asr = timings.find((event) => event.props.stage === "asr");
    expect(["webgpu", "wasm"]).toContain(asr?.props.asr_backend);
    // Which of the two it was is the machine's to say, and is written down: a
    // graphics processor that cannot run the model leaves it to the processor.
    const chain = timings.find((event) => event.props.stage === "audio_chain");
    const backendLine = `the speech model ran on: ${String(asr?.props.asr_backend)}, ${String(asr?.props.duration_ms)} ms; the voice chain took ${String(chain?.props.duration_ms)} ms`;
    testInfo.annotations.push({ type: "asr backend", description: backendLine });
    process.stdout.write(`      ${backendLine}
`);
    expect(api.eventsOf("pipeline_done")).toEqual([
      { name: "pipeline_done", props: { total_ms: expect.any(Number), n_number: 2, n_list: 1, n_from_to: 1, n_keyword: expect.any(Number) } },
    ]);
    const done = api.eventsOf("pipeline_done")[0] as { props: { n_keyword: number } };
    // "brutal" is a KeywordPop only if the measured prosody carries it (README).
    expect([0, 1]).toContain(done.props.n_keyword);
    // The model was on the device: nothing was downloaded, and nothing failed.
    expect(api.eventsOf("model_download")).toEqual([]);
    expect(api.eventsOf("client_error")).toEqual([]);
    // Two plays, one event: it is sent for the first play of a preview.
    expect(api.eventsOf("preview_played")).toEqual([{ name: "preview_played" }]);

    // V4 (D-160): the scene is made from a plan, and the plan's rows are the
    // counted moments. A KeywordPop may be left out of the plan (the
    // validator's drops), so its rows are at most its count.
    await openTab(page, "Visuals");
    const rows = (kind: string) => page.getByTestId("plan-items").locator(`li[data-kind="${kind}"]`);
    await expect(rows("number_reveal")).toHaveCount(2);
    await expect(rows("list_reveal")).toHaveCount(1);
    await expect(rows("from_to")).toHaveCount(1);
    expect(await rows("keyword_pop").count()).toBeLessThanOrEqual(done.props.n_keyword);
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
    const shown = expectedFeedV3().map((row) => row.line.slice(FOUND_PREFIX.length));
    const feedText = [messages.feed.transcribing, messages.feed.cleaningVoice, FOUND_PREFIX.trim(), ...shown];
    const carrying = requests.filter(isHttp).filter((request) => {
      // An address is read as it is and as it reads once its escapes are undone.
      const sent = `${request.url}\n${decodeURI(request.url)}\n${request.body ?? ""}`;
      return feedText.some((text) => text !== "" && sent.includes(text));
    });
    expect(carrying.map((request) => `${request.method} ${request.url}`)).toEqual([]);
  });

  // V4 (D-156, D-193): after the events were read, since an edit sends its own.
  test("long tasks: none over 100 ms from the drop to ready, and none over 50 ms in a word edit, a style switch and a caption drag", async ({}, testInfo) => {
    const tasks = () => page.evaluate(() => window.e2eLongTasks ?? []);
    const wasm = () => page.evaluate(() => window.e2eWasm ?? []);
    const write = (line: string) => {
      testInfo.annotations.push({ type: "long task", description: line });
      process.stdout.write(`      ${line}\n`);
    };
    const main = await wasm();
    write(`WASM instantiations on the main thread: ${main.length === 0 ? "none" : main.map((call) => `${call.bundle.slice(call.bundle.lastIndexOf("/") + 1)} ${String(Math.round(call.end - call.at))} ms`).join(", ")}`);

    const processing = tasksBetween(await tasks(), main, processingMs.from, processingMs.to);
    write(`processing: ${String(processing.length)} tasks of 50 ms or more, the longest ${String(Math.round(Math.max(0, ...processing.map((task) => task.ms))))} ms`);
    expect(processing.filter((task) => task.ms > PROCESSING_TASK_MS).map((task) => Math.round(task.ms))).toEqual([]);

    const words = await transcriptWords(page);
    const word = words.find((candidate, i) => i > words.length / 2 && /^\p{L}{3,}$/u.test(candidate.text));
    expect(word, "the transcript has a word of letters in its second half").toBeDefined();
    const interactions: [string, () => Promise<void>][] = [
      ["a word edit", () => editWord(page, word?.index ?? 0, "fixed")],
      ["a style switch", () => setStyle(page, "Bold")],
      ["a caption drag", () => dragCaptions(page, "top")],
    ];
    for (const [what, act] of interactions) {
      if (what === "a caption drag") {
        // Paused on a word: a caption is shown to take hold of.
        await openWord(page, word?.index ?? 0);
        await page.keyboard.press("Escape");
        await expect(wordField(page)).toHaveCount(0);
        await page.waitForTimeout(300);
      }
      const from = await page.evaluate(() => performance.now());
      await act();
      const during = tasksBetween(await tasks(), await wasm(), from, await page.evaluate(() => performance.now()));
      write(`${what}: ${String(during.length)} tasks of 50 ms or more, the longest ${String(Math.round(Math.max(0, ...during.map((task) => task.ms))))} ms`);
      expect(during.filter((task) => task.ms > EDITING_TASK_MS).map((task) => Math.round(task.ms)), what).toEqual([]);
    }
  });

  test("start over: the drop zone comes back and the clip is gone from the device", async () => {
    await button(page, messages.editor.startOver).click();
    await expect(page.getByTestId("drop-zone")).toBeVisible();
    await expect(canvas(page)).toHaveCount(0);
    await expect.poll(() => opfsList(page, "clips"), { timeout: 30_000 }).toEqual([]);
    // The page is on the editor still, ready for the next clip.
    await expect(page).toHaveURL(/\/app$/);
    await expect(page.getByText(messages.editor.failedStub).or(page.getByTestId("preview-failed"))).toHaveCount(0);
  });
});

test("sample clip: the button fetches the clip from the asset host and imports it", { tag: NO_GPU }, async ({ page }) => {
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

// V4 (D-154): a clip with speech and nothing to show is ready, with captions only.
test("zero events: a clip with no number or list reaches ready, and the Visuals tab says so, with no row", async ({ page, context }) => {
  test.setTimeout(patient(420_000));
  await ensureModelCached(context);
  const api = await installFakeApi(page);
  await routeAssets(page);
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });

  await dropClip(page, genFixture("speech/speech_20s_noevents.mp4"));
  await expect(canvas(page).or(page.getByTestId("clip-failed")).or(page.getByTestId("clip-rejected"))).toBeVisible({ timeout: patient(240_000) });
  await expect(canvas(page), "the clip was processed").toBeVisible();
  await openTab(page, "Visuals");
  await expect(page.getByTestId("no-events-found")).toHaveText(messages.visuals.noEventsFound);
  await expect(page.getByTestId("plan-items")).toHaveCount(0);
  await expect(page.locator("li[data-kind]")).toHaveCount(0);
  await expect.poll(() => api.eventsOf("pipeline_done"), { timeout: FLUSH_WAIT_MS }).toEqual([
    { name: "pipeline_done", props: { total_ms: expect.any(Number), n_number: 0, n_list: 0, n_from_to: 0, n_keyword: 0 } },
  ]);
});

// The founder's second recording (D-99), on the development machine only. It
// runs on its own, after the group above, so that it hides none of its cases.
test("second recording: it is taken, processed, its feed is the README's, and its preview plays", async ({ page, context }) => {
  const clip = v3TestClip();
  test.skip(clip === null, "testclips/v3testclip.mp4 is on D1 only");
  test.setTimeout(patient(600_000));
  await ensureModelCached(context);
  const api = await installFakeApi(page);
  await routeAssets(page);
  await watchPage(page);
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });

  await dropClip(page, clip ?? "");
  await expect(canvas(page).or(page.getByTestId("clip-failed")).or(page.getByTestId("clip-rejected"))).toBeVisible({ timeout: patient(360_000) });
  await expect(canvas(page), "the second recording was processed").toBeVisible();
  await expect.poll(() => api.eventsOf("clip_accepted"), { timeout: FLUSH_WAIT_MS }).toEqual([
    { name: "clip_accepted", props: { source: "user", orientation: "landscape", duration_bucket: "lte90" } },
  ]);
  expectFeed(await lastFeed(page), expectedFeedOfTestClip());

  // The preview plays: two pictures a second apart differ, and are kept nowhere.
  await expect(button(page, messages.preview.play)).toBeEnabled({ timeout: 30_000 });
  await button(page, messages.preview.play).click();
  await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 30_000 });
  await page.waitForTimeout(2_500);
  const [first, second] = await twoPictures(canvas(page), page, 1_000);
  expect(first.equals(second)).toBe(false);
  expect(hasDetail(second)).toBe(true);
  await button(page, messages.preview.pause).click();
});
