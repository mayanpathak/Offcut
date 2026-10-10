// The setup of the media suites and of the bench. It is a project of its
// own, which only `media` and `bench` depend on, so that `pnpm e2e` never
// downloads a model. Two things, in this order:
//
//   1. Chrome on this machine must be able to run Offcut. If it cannot, the
//      suites are not started: every case would wait for a drop zone that
//      never comes. The setup says what is missing instead, and what the
//      browser answers when it is started with other arguments (TE-10).
//   2. The files the asset host would serve are put into `fixtures/.cache/`,
//      once, and checked against their hashes.

import { chromium, expect, type Page, test as setup } from "@playwright/test";

import { AAC_DECODE_PROBE, H264_DECODE_PROBE } from "../src/platform/capability";
import { AAC_ENCODE_CONFIG, CREATOR_VIDEO_BITRATE, VIDEO_ENCODE_LADDER, videoConfigFor } from "../src/workers/render/encoders";
import { LIMITS } from "../src/gen/domain";
import { installFakeApi } from "./helpers/fake-api";
import { fillAssetCache } from "./helpers/fixtures";

// The model is up to 260 MB, and the first run fetches all of it.
const DOWNLOAD_TIMEOUT_MS = 30 * 60_000;
const CAPABILITY_TIMEOUT_MS = 5 * 60_000;
/** A page of the app's origin that is not the app: a place to ask the browser from. */
const BLANK_PAGE = "/e2e-fixture/blank";

// What Chrome is started with to find out whether it can be given WebGPU on
// a machine without a graphics card. Tried only when the app says it cannot run.
const CANDIDATE_ARGS: readonly (readonly string[])[] = [
  ["--enable-unsafe-webgpu"],
  ["--enable-unsafe-webgpu", "--ignore-gpu-blocklist"],
  ["--enable-unsafe-webgpu", "--use-webgpu-adapter=swiftshader"],
  ["--enable-unsafe-webgpu", "--enable-unsafe-swiftshader", "--use-webgpu-adapter=swiftshader"],
  ["--enable-unsafe-webgpu", "--enable-features=Vulkan", "--use-angle=swiftshader", "--use-webgpu-adapter=swiftshader"],
];

// The questions the app's own capability check asks (TS §13.2), with its own
// configurations, each answered by itself: the check stops at the first "no".
const QUESTIONS = {
  h264Decode: H264_DECODE_PROBE,
  videoEncode: VIDEO_ENCODE_LADDER.map((entry) => ({
    entry: `${entry.codec} ${entry.hardwareAcceleration}`,
    config: videoConfigFor(entry, LIMITS.CREATOR_WIDTH, LIMITS.CREATOR_HEIGHT, CREATOR_VIDEO_BITRATE),
  })),
  aacDecode: AAC_DECODE_PROBE,
  aacEncode: AAC_ENCODE_CONFIG,
};

/** What the browser of `page` answers to each question. Words and numbers of the machine only. */
function ask(page: Page): Promise<Record<string, unknown>> {
  return page.evaluate(async (questions) => {
    const said = async (answer: () => Promise<unknown>): Promise<unknown> => {
      try {
        return await answer();
      } catch (error) {
        return `threw ${error instanceof Error ? error.name : "something"}`;
      }
    };
    const adapter = (options?: GPURequestAdapterOptions) =>
      said(async () => {
        const found = await navigator.gpu.requestAdapter(options);
        return found === null ? null : `${found.info.vendor} ${found.info.architecture} ${found.info.description}`.trim();
      });
    const hints: Navigator & { deviceMemory?: number } = navigator;
    return {
      crossOriginIsolated,
      deviceMemoryGb: hints.deviceMemory,
      cores: navigator.hardwareConcurrency,
      webgpu: "gpu" in navigator,
      adapter: "gpu" in navigator ? await adapter() : null,
      fallbackAdapter: "gpu" in navigator ? await adapter({ forceFallbackAdapter: true }) : null,
      h264Decode: await said(async () => (await VideoDecoder.isConfigSupported(questions.h264Decode)).supported === true),
      videoEncode: Object.fromEntries(
        await Promise.all(
          questions.videoEncode.map(async ({ entry, config }) => [
            entry,
            await said(async () => (await VideoEncoder.isConfigSupported(config)).supported === true),
          ]),
        ),
      ) as Record<string, unknown>,
      aacDecode: await said(async () => (await AudioDecoder.isConfigSupported(questions.aacDecode)).supported === true),
      aacEncode: await said(async () => (await AudioEncoder.isConfigSupported(questions.aacEncode)).supported === true),
    };
  }, QUESTIONS);
}

setup("Chrome on this machine can run Offcut", async ({ page, baseURL }, testInfo) => {
  setup.setTimeout(CAPABILITY_TIMEOUT_MS);
  const api = await installFakeApi(page);
  await page.goto("/app");
  const refused = page.getByTestId("unsupported-reason");
  await expect(page.getByTestId("drop-zone").or(refused)).toBeVisible({ timeout: 60_000 });
  if (await page.getByTestId("drop-zone").isVisible()) {
    return;
  }

  // It cannot. Say why, and what could be done about it.
  const lines: string[] = [`The app says: "${(await refused.textContent()) ?? ""}"`];
  await expect.poll(() => api.eventsOf("capability_check").length, { timeout: 20_000 }).toBeGreaterThan(0).catch(() => undefined);
  lines.push(`capability_check: ${JSON.stringify(api.eventsOf("capability_check").at(-1) ?? null)}`);
  const started = testInfo.project.use.launchOptions?.args ?? [];
  lines.push(`Chrome ${page.context().browser()?.version() ?? "?"} started with [${started.join(" ")}] answers: ${JSON.stringify(await ask(page))}`);
  for (const args of CANDIDATE_ARGS) {
    const browser = await chromium.launch({ channel: "chrome", args: [...args] });
    try {
      const other = await browser.newPage(baseURL === undefined ? {} : { baseURL });
      await other.route(`**${BLANK_PAGE}`, (route) => route.fulfill({ contentType: "text/html", body: "<!doctype html><title>e2e</title>" }));
      await other.goto(BLANK_PAGE);
      const answers = await ask(other);
      lines.push(`  with [${args.join(" ")}]: adapter ${JSON.stringify(answers.adapter)}, fallback adapter ${JSON.stringify(answers.fallbackAdapter)}`);
    } catch (error) {
      lines.push(`  with [${args.join(" ")}]: ${error instanceof Error ? error.message.split("\n")[0] ?? "" : "failed"}`);
    } finally {
      await browser.close();
    }
  }
  const report = lines.join("\n");
  process.stdout.write(`\n${report}\n\n`);
  throw new Error(`Chrome on this machine cannot run Offcut, so the media suites were not started.\n${report}`);
});

setup("the asset cache holds the model and the reference clip", async () => {
  setup.setTimeout(DOWNLOAD_TIMEOUT_MS);
  await fillAssetCache();
});
