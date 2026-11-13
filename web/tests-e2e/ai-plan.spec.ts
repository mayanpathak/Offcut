// V6.2: the AI plan in a real browser (v6.2implementation §13.4, D-466 to
// D-469, D-494), on the flags-on build (`media-flags`). The fake API stands
// in for the server: the clip is transcribed "in the cloud" from the real
// adapter's answer with three marker words planted, and `/plan` answers with
// a model's recorded plan for those words (`helpers/plan-fixtures.ts`).
// - Signed in with the switch on: one `POST /plan`, carrying the words as
//   heard; the plan kept whole and drawn (the list, the export's summary, the
//   emoji's word under the pointer); each item switched off and undone; the
//   text for the post copied; run again with no request; "Regenerate plan".
// - Signed out: no request, the rules plan, the sign-in line once.
// - The switch off: no request, and no request carries a word.
// - The plan service down, or this month's plans used: the rules plan and
//   the line once.
// The network rules (D-468): no request but `POST /plan` carries a marker
// word, and no request carries 32 bytes of the media. D1 only, as
// `enhance-panel.spec.ts`: every case processes a clip (no @no-gpu case).

import { readFileSync, statSync } from "node:fs";

import { type Browser, type BrowserContext, expect, type Page, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import type { PlanRequest } from "../src/gen/api";
import { CATALOG_VERSION, type EventKind } from "../src/gen/domain";
import { type FakeApi, type FakeApiOptions, installFakeApi } from "./helpers/fake-api";
import {
  axe,
  captionMirror,
  dropClip,
  ensureModelCached,
  expectTestBuild,
  genFixture,
  hoverPreview,
  openTab,
  openWord,
  patient,
  routeAssets,
  saves,
  sendQueuedAnalytics,
  settled,
  trackSaves,
  wordField,
} from "./helpers/fixtures";
import { HEARD_WORDS, MARKERS, markedAnswer, PLAN_ANSWER, PLAN_KINDS, regeneratedAnswer, signedInPlan } from "./helpers/plan-fixtures";

const CLIP = genFixture("speech/speech_60s_portrait.mp4");
const READY_TIMEOUT_MS = patient(240_000);
const copy = messages.plan;
/** V4's kinds: what a rules plan holds. */
const RULES_KINDS: readonly EventKind[] = ["number_reveal", "list_reveal", "from_to", "keyword_pop"];
/** Where the caption block's centre sits at Bottom, the default (`CAPTION_TRACK`'s bottom, D-183). */
const BOTTOM_CENTRE = 1304 / 1920;
/** The emoji's word: "nobody", which the eyes emoji sits under. */
const EMOJI_WORD = 13;

// --- What the browser sent --------------------------------------------------------

type Sent = { method: string; url: string; body: Buffer };

/** Every request of `context`, from now on: the page's, its workers' and the assets'. */
function recordRequests(context: BrowserContext): Sent[] {
  const sent: Sent[] = [];
  context.on("request", (request) => {
    sent.push({ method: request.method(), url: request.url(), body: request.postDataBuffer() ?? Buffer.alloc(0) });
  });
  return sent;
}

/** The requests that carry a marker word, in the URL or the body, as `METHOD path`. */
function carryingWords(sent: readonly Sent[]): string[] {
  const markers = Object.values(MARKERS);
  return sent
    .filter((request) => markers.some((marker) => request.url.includes(marker) || request.body.includes(marker)))
    .map((request) => `${request.method} ${new URL(request.url).pathname}`);
}

/** 20 pieces of 32 bytes of the clip, spread over the file (as `route-long.spec.ts`). */
function mediaPieces(): Buffer[] {
  const video = readFileSync(CLIP);
  const size = statSync(CLIP).size;
  return Array.from({ length: 20 }, (_, n) => {
    const at = Math.floor(((n + 0.5) * size) / 20);
    return video.subarray(at, at + 32);
  });
}

function expectNoMedia(sent: readonly Sent[]): void {
  const pieces = mediaPieces();
  for (const request of sent) {
    for (const piece of pieces) {
      expect(request.body.includes(piece), `${request.url} carries bytes of the media`).toBe(false);
    }
  }
}

// --- The page ----------------------------------------------------------------------

/**
 * Counts every toast as it appears, from before the page loads: the text of
 * each new toast, in order (a toast that is replaced in place is not new).
 */
async function countToasts(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const w = window as unknown as { e2eToasts: string[] };
    w.e2eToasts = [];
    new MutationObserver((records) => {
      for (const record of records) {
        for (const node of record.addedNodes) {
          if (!(node instanceof HTMLElement)) {
            continue;
          }
          const host = node.matches("[data-testid=toasts]") ? [...node.children] : node.parentElement?.matches("[data-testid=toasts]") === true ? [node] : [];
          for (const toast of host) {
            w.e2eToasts.push(toast.textContent ?? "");
          }
        }
      }
    }).observe(document, { childList: true, subtree: true });
  });
}

/** How many toasts so far said `line`. */
function toastsSaying(page: Page, line: string): Promise<number> {
  return page.evaluate((text) => (window as unknown as { e2eToasts: string[] }).e2eToasts.filter((toast) => toast.includes(text)).length, line);
}

const FALLBACK_LINES = Object.values(copy.fallback);

async function newContext(browser: Browser, baseURL: string | undefined, o: { model: boolean }): Promise<BrowserContext> {
  const context = await browser.newContext({ acceptDownloads: true, ...(baseURL === undefined ? {} : { baseURL }) });
  if (o.model) {
    await ensureModelCached(context);
  }
  return context;
}

/** A page of its own at /app, 1280 × 800, the fake API as `options` say, its saves and toasts counted. */
async function openApp(context: BrowserContext, options: FakeApiOptions): Promise<{ page: Page; api: FakeApi }> {
  const page = await context.newPage();
  await page.setViewportSize({ width: 1280, height: 800 });
  const api = await installFakeApi(page, options);
  await routeAssets(page);
  await trackSaves(page);
  await countToasts(page);
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  await expectTestBuild(page);
  return { page, api };
}

/** Waits until the clip on the page is processed and can be played. */
async function ready(page: Page): Promise<void> {
  const canvas = page.getByTestId("preview-canvas");
  await expect(canvas.or(page.getByTestId("clip-failed"))).toBeVisible({ timeout: READY_TIMEOUT_MS });
  await expect(canvas, "the clip was processed").toBeVisible();
  await expect(page.getByRole("button", { name: messages.preview.play, exact: true })).toBeEnabled({ timeout: 30_000 });
}

const rows = (page: Page) => page.getByTestId("plan-items").locator("li");

/** The kinds of the visuals list's rows, in its order (time). */
async function listedKinds(page: Page): Promise<string[]> {
  await openTab(page, "Visuals");
  return rows(page).evaluateAll((elements) => elements.map((element) => (element as HTMLElement).dataset.kind ?? ""));
}

/** The kinds of the plan's items as the list orders them: by their first word, the plan's order where two share one. */
function kindsInTime(items: readonly { kind: EventKind; anchors: { start: number } }[]): EventKind[] {
  return [...items].sort((a, b) => a.anchors.start - b.anchors.start).map((item) => item.kind);
}

/** The one `plan_result` the run sent. */
async function planResult(page: Page, api: FakeApi): Promise<unknown[]> {
  await sendQueuedAnalytics(page);
  return api.eventsOf("plan_result");
}

const planRequests = (api: FakeApi) => api.requests.filter((request) => request.method === "POST" && request.path === "/plan");

// --- Signed in, the switch on ---------------------------------------------------------

test.describe("signed in, the switch on: the model's plan", () => {
  test.describe.configure({ mode: "serial" });

  let context: BrowserContext;
  let page: Page;
  let api: FakeApi;
  let sent: Sent[];

  test.beforeAll(async ({ browser }, info) => {
    // No model in this context: the cloud transcribes, and nothing here may need one.
    context = await newContext(browser, info.project.use.baseURL, { model: false });
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);
    sent = recordRequests(context);
    ({ page, api } = await openApp(context, signedInPlan()));
  });

  test.afterAll(async () => {
    await context.close();
  });

  test("1. one POST /plan with the words as heard; the plan kept whole; no other request carries a word, none the media", async () => {
    test.setTimeout(patient(600_000));
    await dropClip(page, CLIP);
    await ready(page);

    // One request, its body the transcript as heard: the cloud's words, the markers among them.
    const asked = planRequests(api);
    expect(asked).toHaveLength(1);
    const body = asked[0]?.body as PlanRequest;
    expect(body.catalog).toBe(CATALOG_VERSION);
    expect(body.words.map((word) => word.text)).toEqual(markedAnswer().words.slice(0, HEARD_WORDS).map((word) => word.text));

    // Every item kept, none dropped; the outcome told once, with no fallback line.
    expect(await planResult(page, api)).toEqual([
      {
        name: "plan_result",
        props: { source: "model", outcome: "ok", latency_bucket: expect.any(String), items: PLAN_KINDS.length, dropped_by_reason: {} },
      },
    ]);
    for (const line of FALLBACK_LINES) {
      expect(await toastsSaying(page, line), line).toBe(0);
    }

    // D-468: only `POST /plan` carries the transcript's words; no request carries the media.
    expect(carryingWords(sent)).toEqual(["POST /api/v1/plan"]);
    expectNoMedia(sent);

    // The list is the model's plan, all on; the panel shows its text for the post.
    expect(await listedKinds(page)).toEqual(kindsInTime(PLAN_ANSWER.plan.items));
    await expect(page.getByText(messages.visuals.count(PLAN_KINDS.length, PLAN_KINDS.length), { exact: true })).toBeVisible();
    const extras = PLAN_ANSWER.plan.extras;
    await expect(page.getByTestId("extra-hook")).toContainText(extras.hook ?? "");
    await expect(page.getByTestId("extra-caption")).toContainText(extras.post_caption ?? "");
    await expect(page.getByTestId("extra-hashtags")).toContainText(extras.hashtags.join(" "));
    await expect(page.getByTestId("regenerate-plan")).toBeEnabled();
    expect(await axe(page)).toEqual([]);
  });

  test("2. the text for the post: each Copy puts its text on the clipboard", async () => {
    const extras = PLAN_ANSWER.plan.extras;
    for (const [testId, text] of [
      ["extra-hook", extras.hook ?? ""],
      ["extra-caption", extras.post_caption ?? ""],
      ["extra-hashtags", extras.hashtags.join(" ")],
    ] as const) {
      const group = page.getByTestId(testId);
      await group.getByRole("button", { name: copy.extras.copy, exact: true }).click();
      await expect(group.getByRole("button", { name: copy.extras.copied, exact: true })).toBeVisible();
      expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(text);
    }
  });

  test("3. the emoji's word: at its time the caption shows it, and the pointer finds it with the emoji drawn", async () => {
    await openWord(page, EMOJI_WORD);
    const heard = (await wordField(page).inputValue()).toLowerCase();
    expect(heard).toBe("nobody");
    await page.keyboard.press("Escape");
    await expect(wordField(page)).toHaveCount(0);
    await page.waitForTimeout(300);
    expect((await captionMirror(page)).text.toLowerCase()).toContain(heard);

    await openTab(page, "Visuals");
    let hit = false;
    for (const y of [BOTTOM_CENTRE, BOTTOM_CENTRE - 0.02, BOTTOM_CENTRE + 0.02]) {
      for (const x of [0.5, 0.42, 0.58, 0.34, 0.66, 0.28, 0.72]) {
        hit = await hoverPreview(page, x, y, ["word"]);
        if (hit) {
          break;
        }
      }
      if (hit) {
        break;
      }
    }
    expect(hit, "a caption word under the pointer").toBe(true);
    await page.mouse.down();
    await page.mouse.up();
    await expect(wordField(page)).toBeFocused();
    const opened = (await wordField(page).inputValue()).toLowerCase().replace(/[^\p{L}\p{N}]+/gu, "");
    expect((await captionMirror(page)).text.toLowerCase()).toContain(opened);
    await page.keyboard.press("Escape");
    await expect(wordField(page)).toHaveCount(0);
  });

  test("4. each item switched off, then undone", async () => {
    await openTab(page, "Visuals");
    const total = PLAN_KINDS.length;
    await expect(rows(page)).toHaveCount(total);
    for (let i = 0; i < total; i += 1) {
      const control = rows(page).nth(i).getByRole("switch");
      let since = await saves(page);
      await control.click();
      await expect(control).toHaveAttribute("aria-checked", "false");
      await settled(page, since);
      await expect(page.getByText(messages.visuals.count(total - 1, total), { exact: true })).toBeVisible();
      since = await saves(page);
      await page.keyboard.press("Control+z");
      await expect(control).toHaveAttribute("aria-checked", "true");
      await settled(page, since);
      await expect(page.getByText(messages.visuals.count(total, total), { exact: true })).toBeVisible();
    }
  });

  test("5. the export draws the plan: the summary's counts, and export_done's", async () => {
    test.setTimeout(patient(600_000));
    api.forget();
    const arriving = page.waitForEvent("download", { timeout: patient(300_000) });
    await page.getByRole("button", { name: messages.export.button, exact: true }).click();
    await arriving;
    // KeywordPop and Emphasis mark caption words; every other item drawn is a visual moment (`summary.rs`).
    const emphasized = PLAN_KINDS.filter((kind) => kind === "emphasis").length;
    const summary = messages.export.summary;
    await expect(page.getByText(summary.emphasized(emphasized), { exact: true })).toBeVisible({ timeout: 30_000 });
    await expect(page.getByText(summary.visuals(PLAN_KINDS.length - emphasized), { exact: true })).toBeVisible();
    await sendQueuedAnalytics(page);
    const [done] = api.eventsOf("export_done") as { props: Record<string, unknown> }[];
    expect(done?.props).toMatchObject({ items_kept: PLAN_KINDS.length, items_disabled: 0 });
  });

  test("6. run again (Try again): the plan kept for the clip, with no request", async () => {
    test.setTimeout(patient(600_000));
    // A run that fails after its plan was kept (step 12b keeps it, then the
    // scene is built): a new clip, whose import clears the last one's plan,
    // and its scene refused once. A ready clip forced to `failed` would hold
    // its files open, which no real failure does (read in P210's first run).
    await page.getByRole("button", { name: messages.editor.startOver, exact: true }).click();
    await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
    api.forget();
    await dropClip(page, CLIP);
    await page.evaluate(() => (window as unknown as { __offcutDev: { failNextSetScene(): void } }).__offcutDev.failNextSetScene());
    const failed = page.getByTestId("clip-failed");
    await expect(failed).toBeVisible({ timeout: READY_TIMEOUT_MS });
    expect(planRequests(api)).toHaveLength(1);
    expect(await planResult(page, api)).toEqual([
      { name: "plan_result", props: expect.objectContaining({ source: "model", outcome: "ok", items: PLAN_KINDS.length }) },
    ]);

    api.forget();
    await failed.getByRole("button", { name: messages.recovery.tryAgain, exact: true }).click();
    await expect(failed).toHaveCount(0);
    await ready(page);
    expect(planRequests(api)).toEqual([]);
    expect(await listedKinds(page)).toEqual(kindsInTime(PLAN_ANSWER.plan.items));
    // A stored plan sends no `plan_result`: the run that asked sent one.
    expect(await planResult(page, api)).toEqual([]);
  });

  test("7. Regenerate plan: one request, the new plan in place; a refused one changes nothing and says why", async () => {
    api.forget();
    const next = regeneratedAnswer();
    api.setPlan({ response: next });
    await openTab(page, "Visuals");
    await page.getByTestId("regenerate-plan").click();
    await expect(rows(page)).toHaveCount(next.plan.items.length, { timeout: 30_000 });
    expect(planRequests(api)).toHaveLength(1);
    expect(await listedKinds(page)).toEqual(kindsInTime(next.plan.items));
    await expect(page.getByTestId("extra-caption")).toContainText(next.plan.extras.post_caption ?? "");

    api.setPlan({ failure: { status: 429, code: "plan_quota" } });
    await page.getByTestId("regenerate-plan").click();
    await expect(page.getByText(copy.regenerateLine("quota"), { exact: true })).toBeVisible({ timeout: 30_000 });
    expect(planRequests(api)).toHaveLength(2);
    expect(await listedKinds(page)).toEqual(kindsInTime(next.plan.items));
    await expect(page.getByTestId("extra-caption")).toContainText(next.plan.extras.post_caption ?? "");
    expect(await axe(page)).toEqual([]);
  });
});

// --- No request ------------------------------------------------------------------------

test("signed out: no request to /plan, the rules plan, and the sign-in line once", async ({ browser }, info) => {
  test.setTimeout(patient(600_000));
  // On the device: signed out, the cloud does not transcribe. The fake would answer a plan; nothing asks.
  const context = await newContext(browser, info.project.use.baseURL, { model: true });
  const sent = recordRequests(context);
  const { page, api } = await openApp(context, { healthz: { cloudTranscribe: true }, plan: { response: PLAN_ANSWER } });
  await dropClip(page, CLIP);
  await ready(page);

  expect(planRequests(api)).toEqual([]);
  expect((await listedKinds(page)).every((kind) => (RULES_KINDS as string[]).includes(kind))).toBe(true);
  expect(await toastsSaying(page, copy.fallback.signedOut)).toBe(1);
  expect(await planResult(page, api)).toEqual([
    { name: "plan_result", props: { source: "rules", outcome: "signed_out", items: expect.any(Number), dropped_by_reason: {} } },
  ]);
  expectNoMedia(sent);
  await context.close();
});

test("the switch off: no request to /plan, and no request carries a word of the transcript", async ({ browser }, info) => {
  test.setTimeout(patient(600_000));
  const context = await newContext(browser, info.project.use.baseURL, { model: false });
  const sent = recordRequests(context);
  const { page, api } = await openApp(context, signedInPlan());
  await page.goto("/settings");
  const toggle = page.getByTestId("ai-plan-switch");
  await toggle.click();
  await expect(toggle).toHaveAttribute("aria-checked", "false");
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  await dropClip(page, CLIP);
  await ready(page);

  expect(planRequests(api)).toEqual([]);
  expect((await listedKinds(page)).every((kind) => (RULES_KINDS as string[]).includes(kind))).toBe(true);
  await expect(page.getByTestId("regenerate-plan")).toHaveCount(0);
  for (const line of FALLBACK_LINES) {
    expect(await toastsSaying(page, line), line).toBe(0);
  }
  expect(await planResult(page, api)).toEqual([
    { name: "plan_result", props: { source: "rules", outcome: "off", items: expect.any(Number), dropped_by_reason: {} } },
  ]);
  expect(carryingWords(sent)).toEqual([]);
  expectNoMedia(sent);
  await context.close();
});

// --- The fallbacks ---------------------------------------------------------------------

test.describe("fallbacks: the rules plan, and the line once", () => {
  for (const [name, failure, line, outcome] of [
    ["the plan service down: 502", { status: 502, code: "plan_unavailable" }, copy.fallback.unavailable, "unavailable"],
    ["this month's AI plans used: 429", { status: 429, code: "plan_quota" }, copy.fallback.quota, "quota"],
  ] as const) {
    test(name, async ({ browser }, info) => {
      test.setTimeout(patient(600_000));
      const context = await newContext(browser, info.project.use.baseURL, { model: false });
      const { page, api } = await openApp(context, signedInPlan({ failure }));
      await dropClip(page, CLIP);
      await ready(page);

      expect(planRequests(api)).toHaveLength(1);
      expect((await listedKinds(page)).every((kind) => (RULES_KINDS as string[]).includes(kind))).toBe(true);
      await expect(page.getByTestId("text-extras")).toHaveCount(0);
      expect(await toastsSaying(page, line)).toBe(1);
      expect(await planResult(page, api)).toEqual([
        { name: "plan_result", props: { source: "rules", outcome, latency_bucket: expect.any(String), items: expect.any(Number), dropped_by_reason: {} } },
      ]);
      await context.close();
    });
  }
});
