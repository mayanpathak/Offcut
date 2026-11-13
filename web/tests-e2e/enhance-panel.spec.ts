// V6.2: the Enhance panel in a real browser (v6.2implementation §11, §13.2,
// D-486 to D-488; TS §47.5), on the flags-on build (`media-flags`). One clip,
// processed signed in with nothing remembered:
// - Motion off: every frame's main video shows the clip's crop again (the
//   dev hook's video mirror), where the drift had moved it;
// - Voice cleanup off: the program the preview plays is `lvl48.f32`'s, sample
//   for sample (the dev hook's program samples), and on again `prog48.f32`'s;
// - each switch is within the edit budget, by the page's own measure;
// - undo and redo take a switch back and bring it again;
// - the next clip starts with the switches as last set; settings' "Back to
//   recommended" forgets them.

import { expect, type Page, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import { DB_NAME } from "../src/persistence/schema";
import { dropClip, genFixture, opfsList, patient } from "./helpers/fixtures";
import { openClipSignedIn } from "./helpers/long-windows";

const CLIP = genFixture("speech/speech_60s_portrait.mp4");
/** D-488: a switch shows within the edit budget (stretched as every wait is on a slower machine). */
const EDIT_BUDGET_MS = 150;
/** The program's samples read: one a second from 5 s to 35 s, where the voice is. */
const SAMPLES = Array.from({ length: 31 }, (_, k) => (5 + k) * 48_000);
/** The times the main video is read at, every 2 s over the first 20 s. */
const TIMES = Array.from({ length: 10 }, (_, k) => k * 2_000);

type Rect = { x: number; y: number; w: number; h: number };
type Hooks = {
  videoMirror(at: number): Promise<{ src: Rect; crop: Rect } | null>;
  programSamples(at: number[]): number[];
};

const hooks = (page: Page) => ({
  mirror: (at: number) => page.evaluate((t) => (window as unknown as { __offcutDev: Hooks }).__offcutDev.videoMirror(t), at),
  program: () => page.evaluate((at) => (window as unknown as { __offcutDev: Hooks }).__offcutDev.programSamples(at), SAMPLES),
});

/** `file` of the clip's folder, at `SAMPLES`, as stored (little-endian f32). */
function stored(page: Page, dir: string, file: string): Promise<number[]> {
  return page.evaluate(
    async ({ dir, file, at }) => {
      let handle = await navigator.storage.getDirectory();
      for (const part of dir.split("/")) {
        handle = await handle.getDirectoryHandle(part);
      }
      const bytes = await (await (await handle.getFileHandle(file)).getFile()).arrayBuffer();
      const pcm = new Float32Array(bytes);
      return at.map((i) => pcm[i] ?? Number.NaN);
    },
    { dir, file, at: SAMPLES },
  );
}

/** The value stored in `meta` under `key`, or `undefined`. */
function meta(page: Page, key: string): Promise<unknown> {
  return page.evaluate(
    ({ name, key }) =>
      new Promise<unknown>((resolve) => {
        const open = indexedDB.open(name);
        open.onsuccess = () => {
          const read = open.result.transaction("meta").objectStore("meta").get(key);
          read.onsuccess = () => {
            open.result.close();
            resolve((read.result as { value?: unknown } | undefined)?.value);
          };
          read.onerror = () => {
            open.result.close();
            resolve(undefined);
          };
        };
        open.onerror = () => resolve(undefined);
      }),
    { name: DB_NAME, key },
  );
}

/** How long the page's last switch of `row` took to show, in ms. */
async function lastSwitchMs(page: Page, row: "voice" | "motion", count: number): Promise<number> {
  const name = `offcut:enhance:${row}`;
  await expect.poll(() => page.evaluate((n) => performance.getEntriesByName(n).length, name), { timeout: 10_000 }).toBe(count);
  return page.evaluate((n) => performance.getEntriesByName(n).at(-1)?.duration ?? Number.NaN, name);
}

const voiceSwitch = (page: Page) => page.getByTestId("enhance-voice");
const motionSwitch = (page: Page) => page.getByTestId("enhance-motion");

test("the Enhance switches: motion, voice, the budget, undo, the memory and Back to recommended", async ({ page }, testInfo) => {
  test.setTimeout(patient(900_000));
  await openClipSignedIn(page, CLIP);
  const [id] = await opfsList(page, "clips");
  const dir = `clips/${id ?? ""}`;
  expect(await opfsList(page, dir)).toEqual(["lvl48.f32", "out48.f32", "prog48.f32", "source"]);

  await page.getByRole("tab", { name: messages.panel.style }).click();
  await expect(page.getByTestId("enhance-panel")).toBeVisible();
  // Nothing remembered: the recommended start, Voice cleanup and Motion on.
  await expect(voiceSwitch(page)).toHaveAttribute("aria-checked", "true");
  await expect(motionSwitch(page)).toHaveAttribute("aria-checked", "true");
  const times: string[] = [];

  // Motion: the drift moves the main video off the crop at some times; off, every time shows the crop.
  const before = await Promise.all(TIMES.map((t) => hooks(page).mirror(t)));
  expect(before.filter((m) => m !== null && JSON.stringify(m.src) !== JSON.stringify(m.crop)).length).toBeGreaterThan(0);
  await motionSwitch(page).click();
  await expect(motionSwitch(page)).toHaveAttribute("aria-checked", "false");
  const motionMs = await lastSwitchMs(page, "motion", 1);
  times.push(`motion off ${motionMs.toFixed(0)} ms`);
  expect(motionMs).toBeLessThanOrEqual(patient(EDIT_BUDGET_MS));
  for (const t of TIMES) {
    const m = await hooks(page).mirror(t);
    expect(m?.src, `at ${String(t)} ms`).toEqual(m?.crop);
  }

  // Voice: the preview's program is the clean one, then the levelled voice's, sample for sample.
  const clean = await stored(page, dir, "prog48.f32");
  const levelled = await stored(page, dir, "lvl48.f32");
  expect(clean, "the two voices differ where the speech is").not.toEqual(levelled);
  expect(await hooks(page).program()).toEqual(clean);
  await voiceSwitch(page).click();
  await expect(voiceSwitch(page)).toHaveAttribute("aria-checked", "false");
  const voiceMs = await lastSwitchMs(page, "voice", 1);
  times.push(`voice off ${voiceMs.toFixed(0)} ms`);
  expect(voiceMs).toBeLessThanOrEqual(patient(EDIT_BUDGET_MS));
  await expect.poll(() => hooks(page).program()).toEqual(levelled);
  expect(await meta(page, "enhanceDefault")).toEqual({ voice: "level", motion: false });

  // One step each: undo brings the clean voice back, redo the levelled one; the memory stays.
  await page.keyboard.press("Control+z");
  await expect(voiceSwitch(page)).toHaveAttribute("aria-checked", "true");
  await expect.poll(() => hooks(page).program()).toEqual(clean);
  await page.keyboard.press("Control+Shift+z");
  await expect(voiceSwitch(page)).toHaveAttribute("aria-checked", "false");
  await expect.poll(() => hooks(page).program()).toEqual(levelled);
  expect(await meta(page, "enhanceDefault")).toEqual({ voice: "level", motion: false });
  testInfo.annotations.push({ type: "switch time", description: times.join(", ") });
  process.stdout.write(`      the Enhance switches took: ${times.join(", ")}\n`);

  // The next clip starts with the switches as last set, its preview on the levelled voice.
  await page.getByRole("button", { name: messages.editor.startOver, exact: true }).click();
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  await dropClip(page, CLIP);
  await expect(page.getByRole("button", { name: messages.preview.play, exact: true })).toBeEnabled({ timeout: patient(240_000) });
  await page.getByRole("tab", { name: messages.panel.style }).click();
  await expect(voiceSwitch(page)).toHaveAttribute("aria-checked", "false");
  await expect(motionSwitch(page)).toHaveAttribute("aria-checked", "false");
  const [next] = await opfsList(page, "clips");
  expect(await hooks(page).program()).toEqual(await stored(page, `clips/${next ?? ""}`, "lvl48.f32"));

  // Settings' Back to recommended forgets them; the clip open keeps its own.
  await page.goto("/settings");
  await page.getByTestId("enhance-reset").click();
  await expect(page.getByTestId("enhance-setting").getByRole("status")).toHaveText(messages.plan.enhance.resetDone);
  expect(await meta(page, "enhanceDefault")).toBeUndefined();
});
