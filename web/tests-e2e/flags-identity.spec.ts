// The clip path is the same with the flags on (v6.1implementation §11.4, the
// second half of the two-path proof; project `media-flags`): one clip, taken
// through the clip page of the flags-off build (dist/, port 4173) and of the
// flags-on build (dist-flags/, port 4174), leaves the same stored transcript,
// edit state and render-cache key (so the same events and plan), and exports
// the same file. As read on D1 (P180): the H.264 encoder is not byte-stable
// even within one build (two exports of dist/ differed in size, with the same
// 1,800 frames, identical audio packets, and video PSNR 50.8 dB average,
// 45.4 dB at the least), so "the same file" is: identical audio packets, the
// same frame count and size, and video PSNR of at least 40 dB.

import { execFileSync, spawnSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { type Browser, expect, type Page, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import { DB_NAME } from "../src/persistence/schema";
import { genFixture, opfsList, patient } from "./helpers/fixtures";
import { openClipSignedIn } from "./helpers/long-windows";

const CLIP = genFixture("speech/speech_60s_portrait.mp4");
const FLAGS_OFF = "http://localhost:4173";
const FLAGS_ON = "http://localhost:4174";

/** A stored value of a store of structure version 1, as stored; with `key` "keys", every key of the store. */
function stored(page: Page, store: string, key: string): Promise<unknown> {
  return page.evaluate(
    ({ name, store, key }) =>
      new Promise<unknown>((resolve) => {
        const open = indexedDB.open(name);
        open.onsuccess = () => {
          const db = open.result;
          const objects = db.transaction(store).objectStore(store);
          const read = key === "keys" ? objects.getAllKeys() : objects.get(key);
          read.onsuccess = () => {
            db.close();
            resolve(read.result);
          };
          read.onerror = () => {
            db.close();
            resolve(undefined);
          };
        };
        open.onerror = () => resolve(undefined);
      }),
    { name: DB_NAME, store, key },
  );
}

// V6.2 (P209, DC-16): with the flags on, the build's `ai_plan` (it opens the
// `plans` store at structure version 3, D-469) and, for case 2, where the
// main video shows the source at a few times.
type Run = { records: unknown; file: string; version: number; aiPlan: boolean; motion: MirrorAt[] };
type MirrorAt = { at: number; src: unknown; crop: unknown };

/** The dev hooks this spec reads (`web/src/ui/dev-hooks.ts`). */
type Hooks = {
  flags(): { ai_plan: boolean };
  startAsV5(): Promise<void>;
  videoMirror(at: number): Promise<{ src: unknown; crop: unknown } | null>;
};

/** The times case 2 reads the main video at: the clip's first 20 s, every 2 s. */
const MIRROR_TIMES = [0, 2_000, 4_000, 6_000, 8_000, 10_000, 12_000, 14_000, 16_000, 18_000];

/** The SHA-256 of the file's encoded audio packets, its video frame count and size. */
function streams(file: string): { audio: string; video: string } {
  const audio = execFileSync("ffmpeg", ["-v", "error", "-i", file, "-map", "0:a", "-c", "copy", "-f", "hash", "-hash", "sha256", "-"], { encoding: "utf8" }).trim();
  const video = execFileSync(
    "ffprobe",
    ["-v", "error", "-count_frames", "-select_streams", "v:0", "-show_entries", "stream=nb_read_frames,width,height", "-of", "csv=p=0", file],
    { encoding: "utf8" },
  ).trim();
  return { audio, video };
}

/** The least per-frame PSNR of `b`'s video against `a`'s, in dB: `Infinity` when every frame is the same. */
function minPsnr(a: string, b: string): number {
  // ffmpeg writes the PSNR summary to stderr.
  const run = spawnSync("ffmpeg", ["-i", a, "-i", b, "-lavfi", "[0:v][1:v]psnr", "-f", "null", "-"], { encoding: "utf8" });
  const least = /PSNR .* min:(inf|[0-9.]+)/.exec(run.stderr)?.[1];
  if (least === undefined) {
    throw new Error(`ffmpeg gave no PSNR: ${run.stderr.slice(-300)}`);
  }
  return least === "inf" ? Number.POSITIVE_INFINITY : Number(least);
}

/** The clip processed and exported on one build: what it stored, and where its export is. */
async function runOn(browser: Browser, baseURL: string, scratch: string, o: { asV5?: boolean } = {}): Promise<Run> {
  const context = await browser.newContext({ baseURL, acceptDownloads: true });
  try {
    const page = await context.newPage();
    // V6.2 (DC-16): case 1's flags-on clip is made as V5 made it, through the dev hook.
    const before = async (p: Page): Promise<void> => {
      await p.evaluate(() => (window as unknown as { __offcutDev: Hooks }).__offcutDev.startAsV5());
    };
    await openClipSignedIn(page, CLIP, o.asV5 === true ? { before } : {});
    const aiPlan = await page.evaluate(() => (window as unknown as { __offcutDev: Hooks }).__offcutDev.flags().ai_plan);
    const motion: MirrorAt[] = [];
    for (const at of MIRROR_TIMES) {
      const mirror = await page.evaluate((t) => (window as unknown as { __offcutDev: Hooks }).__offcutDev.videoMirror(t), at);
      motion.push({ at, src: mirror?.src, crop: mirror?.crop });
    }
    const [clipId] = await opfsList(page, "clips");
    const id = clipId ?? "";
    const clip = (await stored(page, "clips", id)) as { schemaVersion: number; value: Record<string, unknown> } | undefined;
    const version = await page.evaluate(
      (name) =>
        new Promise<number>((resolve) => {
          const open = indexedDB.open(name);
          open.onsuccess = () => {
            const v = open.result.version;
            open.result.close();
            resolve(v);
          };
          open.onerror = () => resolve(-1);
        }),
      DB_NAME,
    );
    const arriving = page.waitForEvent("download", { timeout: patient(240_000) });
    await page.getByRole("button", { name: messages.export.button, exact: true }).click();
    const file = path.join(scratch, `${new URL(baseURL).port}.mp4`);
    await (await arriving).saveAs(file);
    const records = {
      // The id and the times are this run's own; everything else must match.
      clip: { schemaVersion: clip?.schemaVersion, clipInfo: clip?.value.clipInfo, sourceBytes: clip?.value.sourceBytes, sourceHashPrefix: clip?.value.sourceHashPrefix },
      transcript: await stored(page, "transcripts", id),
      edits: await stored(page, "edits", id),
      // The export's cache key hashes the transcript, the detected events, the
      // edit, the plan, the time map, the profile and the renderer (D-223): the
      // same key is the same events and the same plan.
      renderCacheKeys: await stored(page, "renderCache", "keys"),
    };
    return { records, file, version, aiPlan, motion };
  } finally {
    await context.close();
  }
}

// The flags-off build's run, made once and read by both cases: a clip takes minutes.
const scratch = mkdtempSync(path.join(tmpdir(), "offcut-identity-"));
let flagsOff: Promise<Run> | undefined;
const offRun = (browser: Browser): Promise<Run> => (flagsOff ??= runOn(browser, FLAGS_OFF, scratch));
test.afterAll(() => {
  rmSync(scratch, { recursive: true, force: true });
});

test("one clip on both builds: the same stored records and the same export as read; the flags-off database at version 1", async ({ browser }) => {
  test.setTimeout(patient(900_000));
  {
    const off = await offRun(browser);
    // D-440 (8), DC-16 (P209): the flags-on clip made as V5 made it, AI plan off and `Enhance::v5()`.
    const on = await runOn(browser, FLAGS_ON, scratch, { asV5: true });
    expect(off.records).toMatchObject({ transcript: expect.anything(), edits: expect.anything(), renderCacheKeys: [expect.any(String)] });
    expect(on.records).toEqual(off.records);
    // The flags-off build never opens structure version 2 (D-414); the
    // flags-on build does at start, to look for a source to continue (D-419).
    // D-440 (8), P209: with `ai_plan` on it opens the `plans` store's 3 (D-469).
    expect(off.version).toBe(1);
    expect(on.version).toBe(on.aiPlan ? 3 : 2);
    // The same export, as one build's two exports are the same (above).
    expect(streams(on.file)).toEqual(streams(off.file));
    expect(minPsnr(off.file, on.file)).toBeGreaterThanOrEqual(40);
  }
});

// --- V6.2 (P209, DC-16): the flags-on defaults -------------------------------
// A clip with nothing remembered starts with the recommended Enhance (Voice
// cleanup and Motion on). Against V5's it differs only where the main video
// shows the source: the transcript, the voice and every frame's count and
// size are V5's; the edit differs by `enhance` alone, and so the cache key.

test("with the flags' defaults: the same clip, the same voice and frames, only the video's src moves", async ({ browser }) => {
  test.setTimeout(patient(900_000));
  const off = await offRun(browser);
  const on = await runOn(browser, FLAGS_ON, scratch);
  const records = (run: Run) => run.records as { clip: unknown; transcript: unknown; edits: { value: Record<string, unknown> }; renderCacheKeys: string[] };
  const [a, b] = [records(off), records(on)];
  expect(b.clip).toEqual(a.clip);
  expect(b.transcript).toEqual(a.transcript);
  const { enhance, ...rest } = b.edits.value.editState as Record<string, unknown>;
  expect(enhance).toEqual({ voice: "clean", motion: true });
  expect({ ...b.edits, value: { ...b.edits.value, editState: rest } }).toEqual(a.edits);
  expect(b.renderCacheKeys).toHaveLength(1);
  expect(b.renderCacheKeys).not.toEqual(a.renderCacheKeys);
  // The voice is V5's, packet for packet, and the frames as many and as large.
  expect(streams(on.file)).toEqual(streams(off.file));
  // Where the main video shows the source: V5's crop at every time, and with the
  // defaults the same crop, moved by the drift at some of them.
  for (const m of off.motion) {
    expect(m.src, `flags off at ${String(m.at)}`).toEqual(m.crop);
  }
  expect(on.motion.map((m) => m.crop)).toEqual(off.motion.map((m) => m.crop));
  expect(on.motion.filter((m) => JSON.stringify(m.src) !== JSON.stringify(m.crop)).length).toBeGreaterThan(0);
});
