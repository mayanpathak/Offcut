// The design reviews' captures (v4implementation D-191, G 10.8): the eight
// states of §17.2 at the two reference widths, written to
// test-results/screens/<state>-<width>x<height>.png. D1 only: `pnpm screens`
// builds the page in development mode first, so that it has its development
// hooks (`__offcutDev`, D-155), and CI never runs this project. Each picture
// is taken once the fonts have loaded and the page's movement has ended; the
// pictures are never committed (SAB rule 9).

import { expect, type Page, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import { installFakeApi } from "./helpers/fake-api";
import { type AssetOptions, dropClip, ensureModelCached, modelManifest, patient, referenceClip, routeAssets } from "./helpers/fixtures";

declare global {
  interface Window {
    /** The page's development hooks (`web/src/ui/dev-hooks.ts`), in a development build only. */
    __offcutDev?: { openWordField(i: number): void; failClip(code: string): void; fakeExport(fraction: number): void };
  }
}

const WIDTHS = [
  { width: 1280, height: 800 },
  { width: 1920, height: 1080 },
];
const OUT = "test-results/screens";
/** Longer than the interface's slowest movement (`--base`, 200 ms) and a toast's entrance. */
const SETTLE_MS = 600;
/** The model download is held at about this share: its requests fail from there on. */
const DOWNLOAD_HELD_AT = 0.3;
/** The least share of the bar at which the download is pictured, in percent. */
const DOWNLOAD_SHOWN_FROM = 25;
/**
 * The error pictured. `E_GPU_LOST` (G 10.8) has no words until Prompt 119
 * and would show the general card; this code has its title, its sentence and
 * what to do (v4changelog open item 9).
 */
const ERROR_CODE = "E_WORKER_CRASH";
const EXPORT_FRACTION = 0.42;
/** How long the clip plays before the ready picture: past its first caption. */
const PLAYED_MS = 3000;
const FOUND = messages.feed.eventFound.number_reveal("");

/** Waits for the fonts and, unless the state passes quickly, for the movement to end; then takes the picture. */
async function picture(page: Page, state: string, settle = true): Promise<void> {
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  if (settle) {
    await page.waitForTimeout(SETTLE_MS);
  }
  const { width, height } = page.viewportSize() ?? { width: 0, height: 0 };
  await page.screenshot({ path: `${OUT}/${state}-${String(width)}x${String(height)}.png` });
}

async function openEditor(page: Page, assets: AssetOptions = {}): Promise<void> {
  await installFakeApi(page);
  await routeAssets(page, assets);
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  const hooks = await page.evaluate(() => window.__offcutDev !== undefined);
  expect(hooks, "the page has no development hooks: run `pnpm screens`, which builds it in development mode").toBe(true);
}

// The processing picture can miss its moment on a fast run: one more try.
test.describe.configure({ retries: 1 });

for (const size of WIDTHS) {
  test.describe(`${String(size.width)} x ${String(size.height)}`, () => {
    test.use({ viewport: size });

    test("model download, at about 30%", async ({ page }) => {
      // A fresh context: the model is not on this device.
      const total = modelManifest.files.reduce((sum, file) => sum + file.bytes, 0);
      await openEditor(page, { failAfterBytes: Math.round(total * DOWNLOAD_HELD_AT) });
      await dropClip(page, referenceClip());
      const bar = page.getByTestId("model-download").getByRole("progressbar");
      await expect
        .poll(async () => Number((await bar.getAttribute("aria-valuenow")) ?? "0"), { timeout: patient(120_000) })
        .toBeGreaterThanOrEqual(DOWNLOAD_SHOWN_FROM);
      await picture(page, "model-download");
    });

    test("empty, processing, ready, editing a word, style tab, exporting, error", async ({ page }) => {
      await ensureModelCached(page.context());
      await openEditor(page);
      await picture(page, "empty");

      await dropClip(page, referenceClip());
      // Processing, once the second step is done: the visual moments are being
      // looked for, and the found ones are chips. That lasts a second or less,
      // so the page is read at every frame, and the picture taken at once.
      const reached = await page.waitForFunction(
        (found) => {
          const card = document.querySelector('[data-testid="processing"]');
          if (card !== null && [...card.querySelectorAll("li")].some((line) => (line.textContent ?? "").startsWith(found))) {
            return "processing";
          }
          return document.querySelector('[data-testid="preview-canvas"]') === null ? false : "ready";
        },
        FOUND,
        { polling: "raf", timeout: patient(240_000) },
      );
      expect(await reached.jsonValue(), "the processing card was gone before its picture").toBe("processing");
      await picture(page, "processing", false);

      // Ready, with a frame of the clip on the stage: the preview draws while
      // it plays (a paused frame is redrawn from Prompt 112, D-173).
      const play = page.getByRole("button", { name: messages.preview.play, exact: true });
      await expect(play).toBeEnabled({ timeout: patient(240_000) });
      // Clicked without moving the pointer: no tooltip in the picture.
      await play.dispatchEvent("click");
      await page.waitForTimeout(PLAYED_MS);
      await page.getByRole("button", { name: messages.preview.pause, exact: true }).dispatchEvent("click");
      await expect(play).toBeEnabled({ timeout: 30_000 });
      await picture(page, "ready");

      await page.evaluate((i) => window.__offcutDev?.openWordField(i), 0);
      await expect(page.getByRole("textbox", { name: messages.captions.wordField })).toBeFocused();
      await picture(page, "editing-word");

      await page.getByRole("tab", { name: messages.panel.style }).click();
      await picture(page, "style-tab");

      await page.evaluate((fraction) => window.__offcutDev?.fakeExport(fraction), EXPORT_FRACTION);
      await expect(page.getByTestId("export-progress")).toBeVisible();
      await picture(page, "exporting");

      await page.evaluate((code) => window.__offcutDev?.failClip(code), ERROR_CODE);
      await expect(page.getByTestId("clip-failed")).toBeVisible();
      await picture(page, "error");
    });
  });
}
