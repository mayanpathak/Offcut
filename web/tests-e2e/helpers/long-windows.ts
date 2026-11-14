// The fake transcription of `long_12min.mp4` (v6.1implementation D-431): each
// window's answer is built from the real adapter's answer for the reference
// clip (`fixtures/cloud/speech_60s_portrait.json`), shifted to where each loop
// sits. No network, real word shapes, the same answer every run.
//
// The fixture (gen_fixtures.sh, step 5c): loop `n` is the reference clip's
// whole sound, then 5 s of room tone, so it starts at `n × (74.517 + 5) s`;
// nine loops, then room tone to 12:00. `speech_60s_portrait.mp4` is the
// reference cut from 0.960 s, so its word at `t` sits at `n × 79.517 s +
// 0.960 s + t` in loop `n`.

import { mkdirSync, readFileSync } from "node:fs";
import path from "node:path";

import { expect, type Page } from "@playwright/test";

import { messages } from "../../src/copy/messages";
import type { CloudWord, TranscribeResponse } from "../../src/gen/api";
import { SOURCE_LIMITS } from "../../src/gen/domain";
import { DB_NAME } from "../../src/persistence/schema";
import { type FakeApi, type FakeApiOptions, installFakeApi } from "./fake-api";
import { dropClip, ensureModelCached, expectTestBuild, patient, routeAssets } from "./fixtures";

const ANSWER = JSON.parse(readFileSync(path.join(import.meta.dirname, "../fixtures/cloud/speech_60s_portrait.json"), "utf8")) as TranscribeResponse;

/** The fixture's length (`manifest-long.json`). */
export const LONG_12MIN_MS = 720_000;
/** One loop: the reference clip's whole sound (74.517 s) and 5 s of room tone. */
export const LOOP_MS = 79_517;
export const LOOPS = 9;
/** Where `speech_60s_portrait.mp4` starts in the reference clip. */
const CUT_AT_MS = 960;
/** Groq invents "Thank you." at that clip's silent end (TE-C2); the reference goes on speaking there, so the loops leave it out. */
const INVENTED = 2;
const LOOP_WORDS = ANSWER.words.slice(0, ANSWER.words.length - INVENTED);

const WINDOW_MS = SOURCE_LIMITS.CLOUD_WINDOW_S * 1000;
const STEP_MS = (SOURCE_LIMITS.CLOUD_WINDOW_S - SOURCE_LIMITS.CLOUD_OVERLAP_S) * 1000;

/** Every word of the fixture, on its own timeline. */
export function sourceWords(): CloudWord[] {
  return Array.from({ length: LOOPS }, (_, n) =>
    LOOP_WORDS.map((word) => ({
      text: word.text,
      start_ms: n * LOOP_MS + CUT_AT_MS + word.start_ms,
      end_ms: n * LOOP_MS + CUT_AT_MS + word.end_ms,
    })),
  ).flat();
}

/** The number of windows of `durationMs` (D-405 as read: `max(1, ⌈(D − overlap) ÷ step⌉)`). */
export function windowTotal(durationMs = LONG_12MIN_MS): number {
  return Math.max(1, Math.ceil((durationMs - SOURCE_LIMITS.CLOUD_OVERLAP_S * 1000) / STEP_MS));
}

/** What the provider would hear in window `index`: the words wholly inside it, from its start. */
export function windowAnswer(index: number, durationMs = LONG_12MIN_MS): TranscribeResponse {
  const from = index * STEP_MS;
  const to = Math.min(from + WINDOW_MS, durationMs);
  return {
    words: sourceWords()
      .filter((word) => word.start_ms >= from && word.end_ms <= to)
      .map((word) => ({ text: word.text, start_ms: word.start_ms - from, end_ms: word.end_ms - from })),
    duration_ms: to - from,
    model: ANSWER.model,
  };
}

/** The words of the first loop up to its first full stop: the master transcript's first sentence. */
export function firstSentence(): string {
  const end = LOOP_WORDS.findIndex((word) => /[.!?]$/.test(word.text));
  return LOOP_WORDS.slice(0, end + 1)
    .map((word) => word.text)
    .join(" ");
}

/** Every word of the fixture: what a master transcript with nothing lost or doubled holds. */
export const LONG_12MIN_WORDS = LOOPS * LOOP_WORDS.length;

// --- The long page, for the specs of the flags-on build ---------------------------

/** Signed in, the cloud on, and `/transcribe` answering each window of `long_12min.mp4`. */
export function signedInCloud(delayMs = 0): FakeApiOptions {
  return {
    session: {},
    healthz: { cloudTranscribe: true },
    transcribe: { response: {}, delayMs, windows: (index) => windowAnswer(index) },
  };
}

/** `/app/long` at 1280 × 800, with the fake API as `options` say. */
export async function openLong(page: Page, options: FakeApiOptions): Promise<FakeApi> {
  await page.setViewportSize({ width: 1280, height: 800 });
  const api = await installFakeApi(page, options);
  await routeAssets(page);
  await page.goto("/app/long");
  await expect(page.getByTestId("long-page")).toBeVisible({ timeout: 30_000 });
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  return api;
}

/** How many records a store of structure version 2 holds (`sources`, `sourceTranscripts`, `masterTranscripts`). */
export function sourceStoreCount(page: Page, store: string): Promise<number> {
  return page.evaluate(
    ({ name, store }) =>
      new Promise<number>((resolve) => {
        const open = indexedDB.open(name);
        open.onsuccess = () => {
          const db = open.result;
          if (!db.objectStoreNames.contains(store)) {
            db.close();
            resolve(0);
            return;
          }
          const count = db.transaction(store).objectStore(store).count();
          count.onsuccess = () => {
            db.close();
            resolve(count.result);
          };
          count.onerror = () => {
            db.close();
            resolve(-1);
          };
        };
        open.onerror = () => resolve(-1);
      }),
    { name: DB_NAME, store },
  );
}

/**
 * `openClip` (helpers/fixtures.ts) with an account signed in (Free unless `plan`), so its
 * export downloads rather than asking to sign in: the clip page at `/app`,
 * `fixture` processed to a ready preview through the cached model.
 */
export async function openClipSignedIn(
  page: Page,
  fixture: string,
  // V6.2 (P209): `before` runs on the open page before the clip is dropped (a dev hook, say).
  o: { picked?: boolean; plan?: "free" | "creator"; before?: (page: Page) => Promise<void> } = {},
): Promise<FakeApi> {
  await page.setViewportSize({ width: 1280, height: 800 });
  await ensureModelCached(page.context());
  const api = await installFakeApi(page, { session: { plan: o.plan ?? "free" } });
  await routeAssets(page);
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  await expectTestBuild(page);
  if (o.before !== undefined) {
    await o.before(page);
  }
  if (o.picked === true) {
    // A large file through the zone's own file field, read from the disk as a
    // person's is: `dropFile` sends the bytes in one DevTools message, which
    // fails at about 80 MB.
    await pickFile(page, fixture);
  } else {
    await dropClip(page, fixture);
  }
  const canvas = page.getByTestId("preview-canvas");
  await expect(canvas.or(page.getByTestId("clip-failed"))).toBeVisible({ timeout: patient(240_000) });
  await expect(canvas, "the clip was processed").toBeVisible();
  await expect(page.getByRole("button", { name: messages.preview.play, exact: true })).toBeEnabled({ timeout: 30_000 });
  return api;
}

/** `file` chosen in the drop zone's own file field, read from the disk (any size). */
export async function pickFile(page: Page, file: string): Promise<void> {
  // The page has one: the zone's own, beside its target.
  await page.locator("input[type=file]").setInputFiles(file);
}

/** The words of the transcript on screen, in order. */
export async function wordsShown(page: Page): Promise<string[]> {
  const lines = await page.getByTestId("source-sentences").locator("li > span:nth-child(2)").allTextContents();
  return lines.flatMap((line) => line.split(/\s+/).filter((word) => word !== ""));
}

/** Every key, or every value, of a store of structure version 2; `[]` before the stores exist. */
export function sourceStoreAll(page: Page, store: string, part: "keys" | "values"): Promise<unknown[]> {
  return page.evaluate(
    ({ name, store, part }) =>
      new Promise<unknown[]>((resolve) => {
        const open = indexedDB.open(name);
        open.onsuccess = () => {
          const db = open.result;
          if (!db.objectStoreNames.contains(store)) {
            db.close();
            resolve([]);
            return;
          }
          const objects = db.transaction(store).objectStore(store);
          const all = part === "keys" ? objects.getAllKeys() : objects.getAll();
          all.onsuccess = () => {
            db.close();
            resolve(all.result as unknown[]);
          };
          all.onerror = () => {
            db.close();
            resolve([]);
          };
        };
        open.onerror = () => resolve([]);
      }),
    { name: DB_NAME, store, part },
  );
}

/** The indexes of the windows kept so far (`sourceTranscripts` is keyed `[sourceId, index]`). */
export async function keptWindows(page: Page): Promise<number[]> {
  const keys = await sourceStoreAll(page, "sourceTranscripts", "keys");
  return keys.flatMap((key) => (Array.isArray(key) && typeof key[1] === "number" ? [key[1]] : [])).sort((a, b) => a - b);
}

/** The one source's record, as stored (`{ schemaVersion, value }`), or `undefined`. */
export async function sourceRecord(page: Page): Promise<{ stage: string; pcm16Seconds: number } | undefined> {
  const [stored] = await sourceStoreAll(page, "sources", "values");
  return (stored as { value?: { stage: string; pcm16Seconds: number } } | undefined)?.value;
}

/**
 * The design review's captures of the long page (G 9.2, ON §5): with
 * `V61_REVIEW_DIR` set, the page as it is now at 1280 × 800 and 1920 × 1080,
 * into that directory. D1 only; CI never sets it. Script A's words only.
 */
export async function capture(page: Page, name: string): Promise<void> {
  const dir = process.env.V61_REVIEW_DIR;
  if (dir === undefined) {
    return;
  }
  mkdirSync(dir, { recursive: true });
  const size = page.viewportSize() ?? { width: 1280, height: 800 };
  for (const [width, height] of [
    [1280, 800],
    [1920, 1080],
  ] as const) {
    await page.setViewportSize({ width, height });
    await page.screenshot({ path: path.join(dir, `${name}-${String(width)}.png`), fullPage: true, animations: "disabled" });
  }
  await page.setViewportSize(size);
}
