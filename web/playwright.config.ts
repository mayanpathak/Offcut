import { defineConfig } from "@playwright/test";

// Locally and on a pull request the suite runs against `vite preview`, which
// serves the build with the production headers (D-15). After a deploy the
// same suite is pointed at the deployment: E2E_BASE_URL=<url> ... --grep @smoke
const deployedUrl = process.env.E2E_BASE_URL;
const PREVIEW_URL = "http://localhost:4173";

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
  projects: [{ name: "non-media" }],
  ...(deployedUrl === undefined && {
    webServer: {
      command: "pnpm exec vite preview --port 4173 --strictPort",
      url: PREVIEW_URL,
      reuseExistingServer: !process.env.CI,
    },
  }),
});
