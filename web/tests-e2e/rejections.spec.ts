// J5 in a real browser: every file Offcut refuses gets its own words, one
// `clip_rejected` that carries the code alone, and leaves nothing on the
// device; a file it takes leaves its record (v3implementation §19.16).
//
// The 13 refusals at import run as one loop over `fixtures/manifest.json`'s
// `rej_*` entries, so a fixture added there is tested without a new case,
// plus the file over the size limit, made in the page (D-79). They and the
// "start over" and accept cases stop before the pipeline needs the model,
// and carry `@no-gpu` (D-93). `REJECT_NO_SPEECH` needs a whole pass of the
// recognizer and runs on a machine with a graphics processor.

import { createHash } from "node:crypto";
import { readFileSync, statSync } from "node:fs";
import path from "node:path";

import { type BrowserContext, expect, type Page, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import { LIMITS, type RejectReason } from "../src/gen/domain";
import { type FakeApi, installFakeApi } from "./helpers/fake-api";
import {
  dropFile,
  dropOversize,
  ensureModelCached,
  flushAnalytics,
  genFixture,
  NO_GPU,
  OVERSIZE_BYTES,
  opfsList,
  patient,
  readIndexedDb,
  rejectCodeOf,
  routeAssets,
} from "./helpers/fixtures";

// The manifest is committed, so it may be read while this file loads; the
// fixtures it names are read only by the cases (V2 CL item 40).
const MANIFEST = path.resolve(import.meta.dirname, "../../fixtures/manifest.json");
type Entry = { name: string; expect: { reject?: RejectReason } };
const REFUSED_AT_IMPORT = (JSON.parse(readFileSync(MANIFEST, "utf8")) as { fixtures: Entry[] }).fixtures.flatMap((entry) =>
  entry.name.startsWith("rej_") && entry.name !== "rej_silent.mp4" && entry.expect.reject !== undefined
    ? [{ file: entry.name, code: entry.expect.reject }]
    : [],
);

/** The limit each body states, worked out from `LIMITS` as the copy does: "500", "90", "1920", "60". */
const LIMIT_IN_BODY: Partial<Record<RejectReason, string>> = {
  REJECT_FILE_SIZE: String(LIMITS.MAX_FILE_SIZE / 1_000_000),
  REJECT_DURATION: String(LIMITS.MAX_CLIP_DURATION / 1000),
  REJECT_RESOLUTION: String(LIMITS.MAX_LONG_SIDE),
  REJECT_FRAME_RATE: String(LIMITS.MAX_INPUT_FPS / 1000),
};

/** The codec of `rej_h264_444.mp4`, H.264 High 4:4:4 Predictive at level 3.1, as read at S2 (`experiments.md`). */
const HIGH_444 = { codec: "avc1.f4001f", codedWidth: 1280, codedHeight: 720 };

/** Every request of the context, with the size of what it carried. */
type Sent = { url: string; bytes: number };

/** A case begins as V2's do (V2 CL item 39): the fake API, the asset host, the page's clock, `/app`. */
async function openApp(page: Page, context: BrowserContext): Promise<{ api: FakeApi; sent: Sent[] }> {
  const sent: Sent[] = [];
  context.on("request", (request) => sent.push({ url: request.url(), bytes: request.postDataBuffer()?.length ?? 0 }));
  const api = await installFakeApi(page);
  await routeAssets(page);
  await page.clock.install();
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  return { api, sent };
}

/** The bodies of the analytics batches, as sent. */
function eventBodies(api: FakeApi): string {
  return JSON.stringify(api.requests.filter((request) => request.path === "/events").map((request) => request.body));
}

/** What every refusal shows and sends, and leaves (§19.16, "Every rejection above"). */
async function expectRefused(page: Page, api: FakeApi, code: RejectReason, file: { name: string; bytes: number }): Promise<void> {
  const panel = page.getByTestId("clip-rejected");
  await expect(panel).toBeVisible({ timeout: 30_000 });
  expect(await rejectCodeOf(page)).toBe(code);
  const copy = messages.rejections[code];
  await expect(panel.getByRole("heading")).toHaveText(copy.title);
  await expect(panel).toContainText(copy.body);
  await expect(panel).toContainText(copy.action);
  const limit = LIMIT_IN_BODY[code];
  if (limit !== undefined) {
    expect(copy.body).toContain(limit);
  }
  // Nothing of the file is shown: neither its name nor its code.
  const shown = (await page.locator("body").textContent()) ?? "";
  expect(shown).not.toContain(file.name);
  expect(shown).not.toContain(code);

  // One event, the code its only property; and nothing else of the clip.
  await flushAnalytics(page);
  await expect.poll(() => api.eventsOf("clip_rejected")).toEqual([{ name: "clip_rejected", props: { reject_reason: code } }]);
  expect(api.eventsOf("client_error")).toEqual([]);
  const bodies = eventBodies(api);
  expect(bodies).not.toContain(file.name);
  expect(bodies).not.toContain(String(file.bytes));
  expect(await opfsList(page, "clips")).toEqual([]);
}

/**
 * The clip was taken: the pipeline's first line, "Cleaning voice", shows. The
 * feed alone shows from the import on, before the clip is accepted; this line
 * comes after `clip_accepted` was queued, and needs no graphics processor.
 */
async function expectAccepted(page: Page): Promise<void> {
  await expect(page.getByTestId("feed").getByText(messages.feed.cleaningVoice)).toBeVisible({ timeout: 30_000 });
}

/** D-81, worked out here: SHA-256 of the first 4 MiB, then the size as an unsigned 64-bit little-endian integer. */
function hashPrefixOf(file: string): string {
  const size = Buffer.alloc(8);
  size.writeBigUInt64LE(BigInt(statSync(file).size));
  return createHash("sha256").update(readFileSync(file).subarray(0, 4_194_304)).update(size).digest("hex");
}

for (const { file, code } of REFUSED_AT_IMPORT) {
  test(`${code}: ${file} is refused with its own words`, { tag: NO_GPU }, async ({ page, context }) => {
    const { api } = await openApp(page, context);
    if (code === "REJECT_DECODE_UNSUPPORTED") {
      // D-78: the case needs a file this Chrome cannot decode. Where it can,
      // the path is covered by `validate_probe`'s inline test instead.
      const decodes = await page.evaluate(async (config) => (await VideoDecoder.isConfigSupported(config)).supported === true, HIGH_444);
      test.fixme(decodes, "This Chrome decodes H.264 High 4:4:4 (D-78, experiments.md: rej_h264_444 in Chrome)");
    }
    // A WebM file keeps an extension of its kind; every other file is clip.mp4.
    const name = file.endsWith(".webm") ? "clip.webm" : "clip.mp4";
    const fixture = genFixture(file);
    await dropFile(page, fixture, name);
    await expectRefused(page, api, code, { name, bytes: statSync(fixture).size });
    expect(api.eventsOf("clip_accepted")).toEqual([]);
  });
}

test("REJECT_FILE_SIZE: a file over the size limit is refused before a byte of it is read", { tag: NO_GPU }, async ({ page, context }) => {
  const { api, sent } = await openApp(page, context);
  await dropOversize(page);
  await expectRefused(page, api, "REJECT_FILE_SIZE", { name: "clip.mp4", bytes: OVERSIZE_BYTES });
  expect(api.eventsOf("clip_accepted")).toEqual([]);
  // No request carried the file: the largest body is an analytics batch.
  expect(Math.max(0, ...sent.map((request) => request.bytes))).toBeLessThan(64 * 1024);
});

test("REJECT_NO_SPEECH: a clip without speech is refused after it was heard", async ({ page, context }) => {
  await ensureModelCached(context);
  await page.addInitScript(() => {
    const seen: string[] = [];
    Object.assign(window, { e2eFeedSeen: seen });
    new MutationObserver(() => {
      for (const line of document.querySelectorAll('[data-testid="feed"] li')) {
        const text = line.textContent ?? "";
        if (!seen.includes(text)) {
          seen.push(text);
        }
      }
    }).observe(document, { childList: true, subtree: true, characterData: true });
  });
  const { api } = await openApp(page, context);
  const fixture = genFixture("rej_silent.mp4");
  await dropFile(page, fixture);
  await expect(page.getByTestId("clip-rejected")).toBeVisible({ timeout: patient(120_000) });
  const seen = await page.evaluate(() => (window as unknown as { e2eFeedSeen: string[] }).e2eFeedSeen);
  expect(seen).toContain(messages.feed.transcribing);
  await expectRefused(page, api, "REJECT_NO_SPEECH", { name: "clip.mp4", bytes: statSync(fixture).size });
  // This clip was taken at import and refused after the recognizer heard it.
  await expect.poll(() => api.eventsOf("clip_accepted")).toHaveLength(1);
});

test("start over: from a refusal back to the drop zone, and a good file is taken", { tag: NO_GPU }, async ({ page, context }) => {
  const { api } = await openApp(page, context);
  await dropFile(page, genFixture("rej_4k.mp4"));
  await expect(page.getByTestId("clip-rejected")).toBeVisible({ timeout: 30_000 });
  await page.getByTestId("clip-rejected").getByRole("button", { name: messages.editor.startOver }).click();
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  expect(await opfsList(page, "clips")).toEqual([]);

  await dropFile(page, genFixture("ok_vfr.mp4"));
  await expect(page.getByTestId("clip-rejected")).toHaveCount(0);
  await expectAccepted(page);
  await flushAnalytics(page);
  await expect.poll(() => api.eventsOf("clip_accepted")).toHaveLength(1);
});

test("an accepted file leaves its record: its size and its D-81 hash, never its name", { tag: NO_GPU }, async ({ page, context }) => {
  const { api } = await openApp(page, context);
  const fixture = genFixture("ok_vfr.mp4");
  await dropFile(page, fixture);
  // The clip leaves `importing`. The case ends before the model is needed.
  await expectAccepted(page);
  await flushAnalytics(page);
  await expect.poll(() => api.eventsOf("clip_accepted")).toEqual([
    { name: "clip_accepted", props: { duration_bucket: "lt30", orientation: "landscape", source: "user" } },
  ]);
  expect(api.eventsOf("clip_rejected")).toEqual([]);

  const [clipId] = await opfsList(page, "clips");
  expect(clipId).toBeDefined();
  const stored = (await readIndexedDb(page, "clips", clipId ?? "")) as { schemaVersion: number; value: Record<string, unknown> } | undefined;
  expect(stored?.schemaVersion).toBe(1);
  expect(stored?.value.clipId).toBe(clipId);
  expect(stored?.value.sourceBytes).toBe(statSync(fixture).size);
  expect(stored?.value.sourceHashPrefix).toBe(hashPrefixOf(fixture));
  expect(JSON.stringify(stored)).not.toContain("clip.mp4");
});
