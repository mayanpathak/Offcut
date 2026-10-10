// The device benchmark (TS §30): how long this machine takes for each stage
// of the reference clip. One run to warm up, then ten that are measured:
// each opens a page, drops the clip, waits for the preview, exports with a
// Creator token and waits for the download. The times are the app's own, read
// from the analytics events its fake API receives.
//
// It runs as the Playwright project `bench`, headed, against `vite preview`
// of a build made with the test key:
//
//   BENCH_DEVICE=<name> pnpm bench:device
//
// and writes bench/results/<name>-<yyyy-mm-dd>.json. The file holds numbers
// and words from fixed lists: nothing that was said in the clip.

import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import path from "node:path";

import { type BrowserContext, expect, test } from "@playwright/test";

import { messages } from "../web/src/copy/messages";
import { installFakeApi, mintEntitlementToken, seedEntitlement } from "../web/tests-e2e/helpers/fake-api";
import { dropClip, ensureModelCached, referenceClip, routeAssets, sourceDurationMs } from "../web/tests-e2e/helpers/fixtures";

declare global {
  interface Window {
    /** The largest reading of the page's memory so far, in bytes; `null` until one has come. */
    benchPeakBytes?: number | null;
  }
  interface Performance {
    // Chrome's, on a page that is cross-origin isolated. Not in the TypeScript library.
    measureUserAgentSpecificMemory?: () => Promise<{ bytes: number }>;
  }
}

const RUNS = 10;
const STAGES = ["probe_audio", "asr", "detect_scene", "render_encode", "mux"] as const;
type Stage = (typeof STAGES)[number];

/** How often the page's memory is read (v2implementation §23.10). */
const MEMORY_SAMPLE_MS = 2_000;
const READY_TIMEOUT_MS = 300_000;
const EXPORT_TIMEOUT_MS = 300_000;
/** Analytics sends what it has every 10 s. */
const FLUSH_WAIT_MS = 30_000;
/** A run is a pipeline and an export; eleven of them, with room for a slow machine. */
const BENCH_TIMEOUT_MS = 120 * 60_000;

const ROOT = path.resolve(import.meta.dirname, "..");

type Run = { stages: Record<Stage, number>; totalMs: number; asrBackend: string; peakBytes: number | null };
type Series = { ms: number[]; median: number; p90: number };

function median(values: readonly number[]): number {
  const sorted = [...values].sort((a, b) => a - b);
  const middle = sorted.length / 2;
  const upper = sorted[Math.floor(middle)] ?? 0;
  return sorted.length % 2 === 1 ? upper : ((sorted[middle - 1] ?? 0) + upper) / 2;
}

/** The value that nine in ten are at or under: the nearest rank. */
function p90(values: readonly number[]): number {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.ceil(sorted.length * 0.9) - 1] ?? 0;
}

const series = (ms: number[]): Series => ({ ms, median: median(ms), p90: p90(ms) });

function propsOf(event: unknown): Record<string, unknown> {
  if (typeof event === "object" && event !== null && "props" in event && typeof event.props === "object" && event.props !== null) {
    return { ...event.props };
  }
  return {};
}

function wholeNumber(value: unknown, what: string): number {
  if (typeof value !== "number" || !Number.isInteger(value)) {
    throw new Error(`${what} is not a whole number`);
  }
  return value;
}

/** One clip, from the drop to the downloaded file, in a new page of `context`. */
async function oneRun(context: BrowserContext): Promise<Run> {
  const page = await context.newPage();
  try {
    await page.addInitScript((everyMs) => {
      window.benchPeakBytes = null;
      const measure = performance.measureUserAgentSpecificMemory?.bind(performance);
      if (measure === undefined) {
        return;
      }
      setInterval(() => {
        // A reading arrives when the browser next collects garbage, which may be seconds later.
        void measure().then(
          (reading) => {
            window.benchPeakBytes = Math.max(window.benchPeakBytes ?? 0, reading.bytes);
          },
          () => undefined,
        );
      }, everyMs);
    }, MEMORY_SAMPLE_MS);
    const api = await installFakeApi(page);
    await routeAssets(page);
    await page.goto("/app");
    await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });

    await dropClip(page, referenceClip());
    await expect(page.getByTestId("preview-canvas")).toBeVisible({ timeout: READY_TIMEOUT_MS });
    await seedEntitlement(page, mintEntitlementToken());
    const arriving = page.waitForEvent("download", { timeout: EXPORT_TIMEOUT_MS });
    await page.getByRole("button", { name: messages.export.button, exact: true }).click();
    const download = await arriving;
    // The file is not what is measured: it is let go as soon as it is whole.
    await download.path();
    await download.delete();
    await expect.poll(() => api.eventsOf("export_done").length, { timeout: FLUSH_WAIT_MS }).toBe(1);

    const stages = {} as Record<Stage, number>;
    let asrBackend = "unknown";
    for (const event of api.eventsOf("stage_timing")) {
      const props = propsOf(event);
      const stage = STAGES.find((name) => name === props.stage);
      if (stage !== undefined) {
        stages[stage] = wholeNumber(props.duration_ms, `stage_timing.${stage}`);
      }
      if (stage === "asr" && typeof props.asr_backend === "string") {
        asrBackend = props.asr_backend;
      }
    }
    for (const stage of STAGES) {
      wholeNumber(stages[stage], `the time of ${stage}`);
    }
    const pipelineMs = wholeNumber(propsOf(api.eventsOf("pipeline_done")[0]).total_ms, "pipeline_done.total_ms");
    const exportMs = wholeNumber(propsOf(api.eventsOf("export_done")[0]).total_ms, "export_done.total_ms");
    const peakBytes = await page.evaluate(() => window.benchPeakBytes ?? null);
    return { stages, totalMs: pipelineMs + exportMs, asrBackend, peakBytes };
  } finally {
    await page.close();
  }
}

test("device bench: one warm-up, then ten measured runs of the reference clip", async ({ context, browser }) => {
  test.setTimeout(BENCH_TIMEOUT_MS);
  const device = process.env.BENCH_DEVICE;
  if (device === undefined || !/^[a-z0-9]+$/.test(device)) {
    throw new Error("BENCH_DEVICE names the machine in lower-case letters and digits, as in BENCH_DEVICE=d1");
  }
  await ensureModelCached(context);

  // The first run of a browser is slower than the ones after it, and is kept out of the numbers.
  await oneRun(context);
  const runs: Run[] = [];
  for (let index = 0; index < RUNS; index += 1) {
    runs.push(await oneRun(context));
  }

  const peaks = runs.map((run) => run.peakBytes).filter((bytes) => bytes !== null);
  const now = new Date();
  const date = `${String(now.getFullYear())}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
  const clip = referenceClip();
  const result = {
    device,
    date,
    commit: execFileSync("git", ["rev-parse", "HEAD"], { cwd: ROOT, encoding: "utf8" }).trim(),
    chrome: browser.version(),
    clip: path.basename(clip),
    clip_duration_ms: sourceDurationMs(clip),
    asr_backend: runs[0]?.asrBackend ?? "unknown",
    runs: runs.length,
    stages: Object.fromEntries(STAGES.map((stage) => [stage, series(runs.map((run) => run.stages[stage]))])),
    total: series(runs.map((run) => run.totalMs)),
    peak_memory_bytes: peaks.length === 0 ? null : Math.max(...peaks),
  };
  const file = path.join(ROOT, "bench/results", `${device}-${date}.json`);
  writeFileSync(file, `${JSON.stringify(result, null, 2)}\n`);
  process.stdout.write(`      wrote ${path.relative(ROOT, file)}: total median ${String(result.total.median)} ms, p90 ${String(result.total.p90)} ms\n`);
  // Every run of the ten used the same backend: a mix would make the medians mean nothing.
  expect(new Set(runs.map((run) => run.asrBackend)).size).toBe(1);
});
