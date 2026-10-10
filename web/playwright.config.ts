import { defineConfig } from "@playwright/test";

// Locally and on a pull request the suite runs against `vite preview`, which
// serves the build with the production headers (D-15). After a deploy the
// same suite is pointed at the deployment: E2E_BASE_URL=<url> ... --grep @smoke
const deployedUrl = process.env.E2E_BASE_URL;
const PREVIEW_URL = "http://localhost:4173";

// The media suites take a clip through the pipeline in a real Chrome. They
// need a build made with the test key (v2implementation D-24), and the model
// in `fixtures/.cache/`, which `media-setup` puts there.
const MEDIA_SUITES = ["model-download.spec.ts", "pipeline-preview.spec.ts", "export-creator.spec.ts"];
const MEDIA_TIMEOUT_MS = 300_000;

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
    { name: "non-media", testMatch: "landing.spec.ts" },
    { name: "media-setup", testMatch: "media.setup.ts" },
    {
      name: "media",
      testMatch: MEDIA_SUITES,
      dependencies: ["media-setup"],
      timeout: MEDIA_TIMEOUT_MS,
      // One clip at a time: a case loads the speech model and the GPU.
      fullyParallel: false,
      workers: 1,
      use: { channel: "chrome" },
    },
    {
      name: "bench",
      testDir: "../bench",
      testMatch: "device-bench.ts",
      dependencies: ["media-setup"],
      fullyParallel: false,
      workers: 1,
      use: { channel: "chrome", headless: false },
    },
  ],
  ...(deployedUrl === undefined && {
    webServer: {
      command: "pnpm exec vite preview --port 4173 --strictPort",
      url: PREVIEW_URL,
      reuseExistingServer: !process.env.CI,
    },
  }),
});
