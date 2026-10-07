// Small helpers shared by the E2E tests.

import type { Page } from "@playwright/test";

// Longer than FLUSH_INTERVAL_MS of src/analytics/client.ts (10 s). That file
// cannot be imported here: it reads the build configuration of the app.
const PAST_FLUSH_INTERVAL_MS = 11_000;

/**
 * Makes the analytics client send what it has queued, by moving the page's
 * clock past its flush interval. The test must have called
 * `page.clock.install()` before it opened the page; no real time passes.
 */
export async function flushAnalytics(page: Page): Promise<void> {
  await page.clock.runFor(PAST_FLUSH_INTERVAL_MS);
}
