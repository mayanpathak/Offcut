// TE-30 (v6.2implementation D-474, §9.1 step 3): the browser draws the
// images as the native renderer does. The render worker of a build with
// development hooks (`pnpm build:e2e`) draws the frame of the `image_basic`
// golden, from the definition the native golden asserts is its own
// (`offcut_render::check`), on this machine's WebGPU; the canvas read back is
// compared with `fixtures/golden/image_basic.png` by the golden test's own
// SSIM (helpers/ssim.ts). D1 only: CI's Windows job runs the `@no-gpu`
// cases, and CI's Linux job checks the same golden natively on lavapipe.
// The drawn frame is kept in test-results/image-parity/ to be read by eye.

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

import { expect, test } from "@playwright/test";

import { installFakeApi } from "./helpers/fake-api";
import { pngPixels, routeAssets } from "./helpers/fixtures";
import { ssim } from "./helpers/ssim";

/** The least similarity of a frame to its golden: the golden test's `GOLDEN_SSIM_MIN`. */
const GOLDEN_SSIM_MIN = 0.98;
const GOLDEN = path.resolve(import.meta.dirname, "../../fixtures/golden/image_basic.png");
const OUT = "test-results/image-parity";

/** The one development hook this spec calls (`web/src/ui/dev-hooks.ts`). */
type Hooks = { imageCheck(): Promise<{ width: number; height: number; pixels: Uint8Array; decodeMs: number }> };

test("TE-30: the browser draws image_basic as the native golden", async ({ page }, testInfo) => {
  await installFakeApi(page);
  await routeAssets(page);
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  const hooks = await page.evaluate(() => (window as unknown as { __offcutDev?: Hooks }).__offcutDev !== undefined);
  expect(hooks, "the page has no development hooks: build it with `pnpm build:e2e`").toBe(true);

  // The pixels cross to the test as base64; the page also makes a PNG of them to keep.
  const drawn = await page.evaluate(async () => {
    const hooks = (window as unknown as { __offcutDev?: Hooks }).__offcutDev;
    if (hooks === undefined) {
      throw new Error("no development hooks");
    }
    const check = await hooks.imageCheck();
    let binary = "";
    for (const byte of check.pixels) {
      binary += String.fromCharCode(byte);
    }
    const canvas = new OffscreenCanvas(check.width, check.height);
    canvas.getContext("2d")?.putImageData(new ImageData(new Uint8ClampedArray(check.pixels), check.width), 0, 0);
    const png = await canvas.convertToBlob({ type: "image/png" });
    let pngBinary = "";
    for (const byte of new Uint8Array(await png.arrayBuffer())) {
      pngBinary += String.fromCharCode(byte);
    }
    return { width: check.width, height: check.height, pixels: btoa(binary), png: btoa(pngBinary), decodeMs: check.decodeMs };
  });

  mkdirSync(OUT, { recursive: true });
  writeFileSync(path.join(OUT, "image_basic-browser.png"), Buffer.from(drawn.png, "base64"));
  const golden = pngPixels(readFileSync(GOLDEN));
  expect(golden.channels, "the golden is RGBA").toBe(4);
  const browser = { width: drawn.width, height: drawn.height, data: new Uint8Array(Buffer.from(drawn.pixels, "base64")) };
  const similarity = ssim(golden, browser);
  const adapter = await page.evaluate(async () => {
    const info = (await navigator.gpu.requestAdapter())?.info;
    return info === undefined ? "no adapter" : `${info.vendor} ${info.architecture} ${info.description}`.trim();
  });
  const reading = `TE-30 ssim image_basic browser = ${similarity.toFixed(4)} on ${adapter}; the 72 images decoded in ${String(drawn.decodeMs)} ms`;
  testInfo.annotations.push({ type: "TE-30", description: reading });
  process.stdout.write(`      ${reading}
`);
  expect(similarity).toBeGreaterThanOrEqual(GOLDEN_SSIM_MIN);
});
