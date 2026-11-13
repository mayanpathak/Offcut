import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { defineConfig } from "@playwright/test";

// Locally and on a pull request the suite runs against `vite preview`, which
// serves the build with the production headers (D-15). After a deploy the
// same suite is pointed at the deployment: E2E_BASE_URL=<url> ... --grep @smoke
const deployedUrl = process.env.E2E_BASE_URL;
const PREVIEW_URL = "http://localhost:4173";
// V6.1 (v6.1implementation D-394): the same suites' build with `long_video`
// and `high_res` on, `pnpm build:e2e:flags` into dist-flags/, served on a
// second port. Its server starts only where that build exists.
const FLAGS_URL = "http://localhost:4174";
const FLAGS_BUILT = existsSync(fileURLToPath(new URL("dist-flags/index.html", import.meta.url)));

// The media suites take a clip through the pipeline in a real Chrome. They
// need a build made with the test key (v2implementation D-24), and the model
// in `fixtures/.cache/`, which `media-setup` puts there. On D1 the build is
// `pnpm build:e2e` (the key in the environment): a production bundle made in
// the `test` mode, which keeps the player's caption mirrors and the
// development hooks that `preview-sync` and `review-edit` read
// (v4implementation D-155, §20.4). CI's Windows job runs the `@no-gpu` cases
// on the production build of the ci job: of V4's suites, the import and the
// model-download cancels of `cancel.spec.ts` (§19.4).
const MEDIA_SUITES = [
  "model-download.spec.ts",
  "pipeline-preview.spec.ts",
  "export-creator.spec.ts",
  "export-free.spec.ts",
  "timeline-preserved.spec.ts",
  "rejections.spec.ts",
  "preview-sync.spec.ts",
  "review-edit.spec.ts",
  "cancel.spec.ts",
  "cloud-transcribe.spec.ts",
  // V6.2 (TE-30, D-474): the images drawn in the browser against their golden. D1 only (no @no-gpu case).
  "image-parity.spec.ts",
];
// V6.1 (§11.4): the suites of the flags-on build. `rejections.spec.ts` runs
// there unchanged, less the three cases whose file the flags-on clip page
// takes: 4K (DC-2, now `high-res.spec.ts`), "start over" (its refused file is
// that 4K one) and the 500 MB size limit (DC-3: 16 GB with `high_res`).
// V6.2 (P209): `enhance-panel.spec.ts`, the Enhance panel on the flags-on build.
const FLAG_SUITES = [
  "route-long.spec.ts",
  "high-res.spec.ts",
  "long-resume.spec.ts",
  "flags-identity.spec.ts",
  "rejections.spec.ts",
  "enhance-panel.spec.ts",
];
const FLAGS_TAKE = /REJECT_RESOLUTION: rej_4k\.mp4 |REJECT_FILE_SIZE: a file over the size limit|start over: from a refusal/;
// On a machine without a graphics processor a case may take this many times
// longer (`patient` of tests-e2e/helpers/fixtures.ts stretches its waits the same way).
const TIMEOUT_SCALE = Number(process.env.E2E_TIMEOUT_SCALE ?? "1");
const MEDIA_TIMEOUT_MS = 300_000 * TIMEOUT_SCALE;
// Arguments Chrome is started with for the media suites and the bench, where a
// machine needs some to give a page WebGPU: the hosted runner of CI, which has
// no graphics card (TE-10). Separated by spaces. None on a development machine.
const MEDIA_CHROME_ARGS = (process.env.E2E_CHROME_ARGS ?? "").split(" ").filter((argument) => argument !== "");
// A trace holds what the page showed: the lines of the feed, and frames of the
// clip with its captions. No report of these suites may carry that (§22.5).
const MEDIA_USE = { channel: "chrome", trace: "off", launchOptions: { args: MEDIA_CHROME_ARGS } } as const;

export default defineConfig({
  testDir: "tests-e2e",
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [["list"], ["html", { open: "never" }]] : "list",
  // Whether the suite runs against a real deployment. Tests read this, not the environment.
  metadata: { deployed: deployedUrl !== undefined },
  use: {
    baseURL: deployedUrl ?? PREVIEW_URL,
    channel: "chrome",
    trace: "retain-on-failure",
  },
  projects: [
    // `pnpm e2e`, and the Linux job of CI. No dependency: it never fetches the model.
    { name: "non-media", testMatch: ["landing.spec.ts", "signin.spec.ts", "checkout.spec.ts"] },
    // With the arguments of the suites it stands for: it asks whether their Chrome can run the app.
    { name: "media-setup", testMatch: "media.setup.ts", fullyParallel: false, workers: 1, use: MEDIA_USE },
    {
      name: "media",
      testMatch: MEDIA_SUITES,
      dependencies: ["media-setup"],
      timeout: MEDIA_TIMEOUT_MS,
      // One clip at a time: a case loads the speech model and the GPU.
      fullyParallel: false,
      workers: 1,
      use: MEDIA_USE,
    },
    // `pnpm e2e:media:flags` (V6.1, D-394): FLAG_SUITES against dist-flags/.
    // A spec knows the build it tests from this project's name.
    {
      name: "media-flags",
      testMatch: FLAG_SUITES,
      grepInvert: FLAGS_TAKE,
      dependencies: ["media-setup"],
      timeout: MEDIA_TIMEOUT_MS,
      fullyParallel: false,
      workers: 1,
      use: { ...MEDIA_USE, baseURL: deployedUrl ?? FLAGS_URL },
    },
    // `pnpm screens`, on D1 only: the eight states at the two reference widths
    // for the design reviews (v4implementation D-191). CI names its projects
    // and never runs this one; the pictures stay in test-results/.
    {
      name: "screens",
      testMatch: "screens.spec.ts",
      dependencies: ["media-setup"],
      timeout: MEDIA_TIMEOUT_MS,
      fullyParallel: false,
      workers: 1,
      use: MEDIA_USE,
    },
    {
      name: "bench",
      testDir: "../bench",
      testMatch: "device-bench.ts",
      dependencies: ["media-setup"],
      fullyParallel: false,
      workers: 1,
      use: { ...MEDIA_USE, headless: false },
    },
    // `pnpm corpus`: the clips of corpus/manifest.json this machine has. Some
    // are lent by people other than the founder, so nothing the page showed is
    // kept, not even of a failure (v3implementation §17.6, D-112).
    {
      name: "corpus",
      testDir: "../corpus",
      testMatch: "run_corpus.ts",
      dependencies: ["media-setup"],
      timeout: MEDIA_TIMEOUT_MS * 2,
      fullyParallel: false,
      workers: 1,
      use: { ...MEDIA_USE, screenshot: "off", video: "off", trace: "off" },
    },
  ],
  ...(deployedUrl === undefined && {
    webServer: [
      {
        command: "pnpm exec vite preview --port 4173 --strictPort",
        url: PREVIEW_URL,
        reuseExistingServer: !process.env.CI,
      },
      ...(FLAGS_BUILT
        ? [
            {
              command: "pnpm exec vite preview --port 4174 --strictPort --outDir dist-flags",
              url: FLAGS_URL,
              reuseExistingServer: !process.env.CI,
            },
          ]
        : []),
    ],
  }),
});
