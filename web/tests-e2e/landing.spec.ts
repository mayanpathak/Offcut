// J1, J3 and C-16 in a real browser, against the built app served with the
// production headers. The API is a fake: no case here touches a real server,
// except `@smoke`, which is the one case also run against a deployment.
//
// The cases hold on a browser that can run Offcut and on one that cannot:
// the browser of CI may lack WebGPU.

import { expect, type Page, test } from "@playwright/test";

import { PRICES_USD } from "../src/config/pricing";
import { LEGAL_DOCS, messages } from "../src/copy/messages";
import { ANALYTICS_EVENT_DOCS, type CheckResult, type GpuVendor, type MemoryBucket, type Platform } from "../src/gen/api";
import { LIMITS, UNSUPPORTED_REASONS } from "../src/gen/domain";
import { installFakeApi } from "./helpers/fake-api";
import { axe, flushAnalytics } from "./helpers/fixtures";

declare global {
  interface Window {
    cspViolations?: string[];
  }
}

const EMAIL = "someone@example.com";
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;

const EVENT_NAMES: readonly string[] = ANALYTICS_EVENT_DOCS.map((doc) => doc.name);
const RESULTS: readonly CheckResult[] = ["pass", "fail"];
const GPU_VENDORS: readonly GpuVendor[] = ["intel", "amd", "nvidia", "apple", "qualcomm", "other", "unknown"];
const MEMORY_BUCKETS: readonly MemoryBucket[] = ["lt4", "gb4", "gb8plus", "unknown"];
const PLATFORMS: readonly Platform[] = ["windows", "macos", "linux", "chromeos", "android", "other", "unknown"];

type Batch = { anon_id: string; events: { name: string; props?: Record<string, unknown> }[] };

const emailField = (page: Page) => page.getByLabel(messages.notifyMe.emailLabel);
const submitButton = (page: Page) => page.getByRole("button", { name: messages.notifyMe.submit });
const formStatus = (page: Page) => page.locator("form").getByRole("status");
const dropZone = (page: Page) => page.getByTestId("drop-zone");

/** Waits until the capability check has an answer, whichever it is. */
async function capabilityChecked(page: Page): Promise<void> {
  await expect(page.getByTestId("capability-line")).not.toHaveText(messages.capability.checking, { timeout: 15_000 });
}

/** Asserts that a recorded body is an `EventsBatch` holding only allowlisted events, and returns it typed. */
function asBatch(body: unknown): Batch {
  expect(body).toEqual({ anon_id: expect.stringMatching(UUID), events: expect.any(Array) });
  const batch = body as Batch;
  expect(batch.events.length).toBeGreaterThan(0);
  for (const event of batch.events) {
    expect(Object.keys(event).filter((key) => key !== "name" && key !== "props")).toEqual([]);
    expect(EVENT_NAMES).toContain(event.name);
  }
  return batch;
}

test("J1: the landing content is there before any interaction", async ({ page }) => {
  await installFakeApi(page);
  await page.goto("/");

  await expect(page.getByRole("heading", { name: messages.landing.hero })).toBeVisible();
  await expect(page.getByText(messages.landing.supportedBrowsers)).toBeVisible();
  await expect(page.getByText(messages.landing.privacyLine)).toBeVisible();
  await expect(page.getByRole("link", { name: messages.landing.whatLeavesLink })).toBeVisible();
  await expect(dropZone(page)).toBeVisible();
  await expect(dropZone(page)).toHaveText(messages.dropZone.prompt(LIMITS));
  await expect(page.getByRole("button", { name: messages.dropZone.sampleButton })).toBeVisible();
  await expect(submitButton(page)).toBeVisible();
});

test("the settings link leads to the table of what leaves the device", async ({ page }) => {
  await installFakeApi(page);
  await page.goto("/");
  await page.getByRole("link", { name: messages.landing.whatLeavesLink }).click();
  await expect(page).toHaveURL(/\/settings$/);

  const fixed = page.getByTestId("what-leaves").locator("tbody tr");
  await expect(fixed).toHaveCount(messages.settings.rows.length);
  await expect(fixed).toHaveCount(4);

  // One row per event of the generated allowlist, each with its wire name.
  const events = page.getByTestId("analytics-events").locator("tbody tr");
  await expect(events).toHaveCount(ANALYTICS_EVENT_DOCS.length);
  for (const [index, doc] of ANALYTICS_EVENT_DOCS.entries()) {
    await expect(events.nth(index)).toContainText(doc.name);
    await expect(events.nth(index)).toContainText(doc.description);
  }
});

test("landing_view is sent once, with the outcome headline", async ({ page }) => {
  const api = await installFakeApi(page);
  await page.clock.install();
  await page.goto("/");
  await capabilityChecked(page);
  await flushAnalytics(page);

  await expect.poll(() => api.eventsOf("landing_view")).toEqual([{ name: "landing_view", props: { hero_variant: "outcome" } }]);
  for (const request of api.requests.filter((entry) => entry.path === "/events")) {
    expect(request.method).toBe("POST");
    asBatch(request.body);
  }
});

test("capability_check is sent once, and every prop is a member of its enum", async ({ page }) => {
  const api = await installFakeApi(page);
  await page.clock.install();
  await page.goto("/");
  await capabilityChecked(page);
  await flushAnalytics(page);

  await expect.poll(() => api.eventsOf("capability_check").length).toBe(1);
  const batches = api.requests.filter((entry) => entry.path === "/events").map((entry) => asBatch(entry.body));
  const props = batches.flatMap((batch) => batch.events).find((event) => event.name === "capability_check")?.props ?? {};

  expect(RESULTS).toContain(props.result);
  expect(GPU_VENDORS).toContain(props.gpu_vendor);
  expect(MEMORY_BUCKETS).toContain(props.memory_bucket);
  expect(PLATFORMS).toContain(props.platform);
  // This browser may or may not be able to run Offcut. Either way the shape holds.
  if (props.result === "fail") {
    expect(UNSUPPORTED_REASONS).toContain(props.unsupported_reason);
    expect(Object.keys(props).sort()).toEqual(["gpu_vendor", "memory_bucket", "platform", "result", "unsupported_reason"]);
  } else {
    expect(Object.keys(props).sort()).toEqual(["gpu_vendor", "memory_bucket", "platform", "result"]);
  }
  // Every batch of the session carries the same anonymous id.
  expect(new Set(batches.map((batch) => batch.anon_id)).size).toBe(1);
});

test("waitlist: a valid email is posted and the success message shows", async ({ page }) => {
  const api = await installFakeApi(page);
  await page.goto("/");
  await emailField(page).fill(EMAIL);
  await submitButton(page).click();

  await expect(formStatus(page)).toHaveText(messages.notifyMe.success);
  expect(api.requests.filter((entry) => entry.path === "/notify-me")).toEqual([
    { method: "POST", path: "/notify-me", body: { email: EMAIL, wanted: "launch" } },
  ]);
});

test("waitlist: text without @ shows the invalid message and sends nothing", async ({ page }) => {
  const api = await installFakeApi(page);
  await page.goto("/");
  await emailField(page).fill("not an email");
  await submitButton(page).click();

  await expect(formStatus(page)).toHaveText(messages.notifyMe.invalidEmail);
  expect(api.requests.filter((entry) => entry.path === "/notify-me")).toEqual([]);
});

test("waitlist: a 429 shows the rate-limit message with the wait", async ({ page }) => {
  await installFakeApi(page, { notifyMe: { status: 429 } });
  await page.goto("/");
  await emailField(page).fill(EMAIL);
  await submitButton(page).click();

  // The fake sends the wait the real server would: 705 seconds.
  await expect(formStatus(page)).toHaveText(messages.notifyMe.rateLimited(705));
});

test("waitlist: a slow API shows the waking message, then success", async ({ page }) => {
  const api = await installFakeApi(page, { notifyMe: { status: 204, delayMs: 4_000 } });
  await page.goto("/");
  await emailField(page).fill(EMAIL);
  await submitButton(page).click();

  // Real time on purpose: the notice comes after 3 s, the answer after 4 s.
  await expect(formStatus(page)).toHaveText(messages.notifyMe.sending);
  await expect(formStatus(page)).toHaveText(messages.api.waking);
  await expect(formStatus(page)).toHaveText(messages.notifyMe.success);
  // One request: the wait was for an answer, not for a retry.
  expect(api.requests.filter((entry) => entry.path === "/notify-me")).toHaveLength(1);
});

test("a dropped file starts an import and opens /app, or is refused where Offcut cannot run", async ({ page, baseURL }) => {
  const api = await installFakeApi(page);
  const response = await page.goto("/");
  await capabilityChecked(page);
  await expect(dropZone(page)).not.toHaveAttribute("aria-disabled");
  // Let the start of the app finish its own requests first: the refresh, which
  // goes once the API is up, is the last one it always makes (v6implementation D-297).
  await expect.poll(() => api.requests.some((entry) => entry.path === "/auth/refresh")).toBe(true);

  const requests: { method: string; url: string }[] = [];
  page.on("request", (request) => requests.push({ method: request.method(), url: request.url() }));
  const transfer = await page.evaluateHandle(() => {
    const data = new DataTransfer();
    data.items.add(new File(["not a real clip"], "private-clip.mp4", { type: "video/mp4" }));
    return data;
  });
  await dropZone(page).dispatchEvent("dragover", { dataTransfer: transfer });
  await dropZone(page).dispatchEvent("drop", { dataTransfer: transfer });

  // Two outcomes, and the browser of CI may give either: the clip is taken
  // and the page moves on with it, or the drop is refused and the page stays.
  const refused = page.getByText(messages.blockers.B_UNSUPPORTED);
  await expect(page.getByTestId("editor").or(refused)).toBeVisible({ timeout: 15_000 });
  if (await refused.isVisible()) {
    await expect(page).toHaveURL(/\/$/);
  } else {
    await expect(page).toHaveURL(/\/app$/);
    // The file is no clip: the page says so in the words of its code, and
    // offers the way back. V3 (Prompt 83): the rejection panel in place of
    // V2's one-line stub; which code 15 bytes of text get is the reader's.
    const panel = page.getByTestId("clip-rejected");
    await expect(panel).toBeVisible({ timeout: 15_000 });
    const code = (await panel.getAttribute("data-reject")) ?? "";
    const copy = Object.entries(messages.rejections).find(([reason]) => reason === code)?.[1];
    expect(copy, `data-reject="${code}" has copy`).toBeDefined();
    await expect(panel.getByRole("heading")).toHaveText(copy?.title ?? "");
    await expect(page.getByRole("button", { name: messages.editor.startOver })).toBeVisible();
  }

  // The browser did not open the file, and the drop sent nothing anywhere:
  // what followed it are analytics, the wake-up call, and files that are
  // fetched from the page's own origin or from the asset host.
  await page.waitForTimeout(500);
  const origin = new URL(baseURL ?? "").origin;
  const csp = response?.headers()["content-security-policy"] ?? "";
  const assetHosts = (csp.match(/https?:\/\/[^\s;]+/g) ?? []).map((host) => new URL(host).origin);
  const unexpected = requests.filter(({ method, url }) => {
    const target = new URL(url);
    if (!/^https?:$/.test(target.protocol)) {
      return false;
    }
    if (target.origin === origin && target.pathname.startsWith("/api/")) {
      return !(
        (method === "POST" && target.pathname === "/api/v1/events") ||
        (method === "GET" && target.pathname === "/api/v1/healthz")
      );
    }
    return method !== "GET" || (target.origin !== origin && !assetHosts.includes(target.origin));
  });
  expect(unexpected).toEqual([]);
});

test("the start of the app asks once for a session, after the wake, and a refusal is not an error", async ({ page }) => {
  const api = await installFakeApi(page);
  await page.goto("/");
  await capabilityChecked(page);
  await expect.poll(() => api.requests.some((entry) => entry.path === "/auth/refresh")).toBe(true);
  await page.waitForTimeout(500);

  const paths = api.requests.map((entry) => entry.path);
  expect(paths.filter((path) => path === "/auth/refresh")).toHaveLength(1);
  expect(paths.indexOf("/healthz")).toBeLessThan(paths.indexOf("/auth/refresh"));
  expect(api.requests.find((entry) => entry.path === "/auth/refresh")?.method).toBe("POST");
  expect(api.eventsOf("client_error")).toEqual([]);
});

test("the page does not wait for a session: with no answer it still works", async ({ page }) => {
  await installFakeApi(page);
  // Added after the fake, so it goes first. It never answers.
  await page.route("**/api/v1/auth/refresh", () => undefined);
  const asked = page.waitForRequest("**/api/v1/auth/refresh");
  await page.goto("/");
  await capabilityChecked(page);
  await asked;
  await expect(page.getByRole("link", { name: messages.landing.whatLeavesLink })).toBeVisible();
  await emailField(page).fill(EMAIL);
  await submitButton(page).click();
  await expect(formStatus(page)).toHaveText(messages.notifyMe.success);
});

test("/app shows the drop zone or the unsupported page, never a blank screen", async ({ page }) => {
  await installFakeApi(page);
  await page.goto("/app");

  const unsupported = page.getByTestId("unsupported-reason");
  await expect(dropZone(page).or(unsupported)).toBeVisible({ timeout: 15_000 });
  // The page that cannot run Offcut is the one that still takes an email.
  if (await unsupported.isVisible()) {
    await expect(submitButton(page)).toBeVisible();
  } else {
    await expect(dropZone(page)).toHaveText(messages.dropZone.prompt(LIMITS));
    await expect(page.getByRole("button", { name: messages.dropZone.sampleButton })).toBeVisible();
  }
});

test("every request of a session goes to the page's own origin or to the asset host", async ({ page, baseURL }) => {
  const urls: string[] = [];
  page.on("request", (request) => urls.push(request.url()));
  await installFakeApi(page);

  const response = await page.goto("/");
  await capabilityChecked(page);
  await emailField(page).fill(EMAIL);
  await submitButton(page).click();
  await expect(formStatus(page)).toHaveText(messages.notifyMe.success);
  await page.getByRole("link", { name: messages.landing.whatLeavesLink }).click();
  await expect(page).toHaveURL(/\/settings$/);
  await page.goto("/app");
  await expect(dropZone(page).or(page.getByTestId("unsupported-reason"))).toBeVisible({ timeout: 15_000 });

  // The hosts the CSP allows are the hosts the app may contact.
  const csp = response?.headers()["content-security-policy"] ?? "";
  const allowed = new Set([new URL(baseURL ?? "").origin, ...(csp.match(/https?:\/\/[^\s;]+/g) ?? [])]);
  expect(allowed.size).toBe(2);

  const network = urls.filter((url) => /^https?:/.test(url));
  expect(network.length).toBeGreaterThan(3);
  expect(network.filter((url) => !allowed.has(new URL(url).origin))).toEqual([]);
});

test("@smoke the page is isolated and the WASM bundle compiles under the CSP", async ({ page }, testInfo) => {
  // Against a deployment the real API answers, and only analytics are kept
  // out so the run adds no rows. Everywhere else the API is the fake.
  if (testInfo.config.metadata.deployed === true) {
    await page.route("**/api/v1/events", (route) => route.fulfill({ status: 204 }));
  } else {
    await installFakeApi(page);
  }
  await page.addInitScript(() => {
    window.cspViolations = [];
    document.addEventListener("securitypolicyviolation", (event) => {
      window.cspViolations?.push(`${event.violatedDirective} ${event.blockedURI}`);
    });
  });

  const response = await page.goto("/");
  expect(response?.headers()["content-security-policy"]).toContain("'wasm-unsafe-eval'");
  await capabilityChecked(page);
  expect(await page.evaluate(() => crossOriginIsolated)).toBe(true);

  // The app preloads the bundle only on a browser it can run in. This test
  // must also hold where it cannot, so it finds the file in the app's own
  // script and compiles it here, which is what needs the CSP's permission.
  const script = await page.locator("script[type=module][src]").first().getAttribute("src");
  expect(script).toMatch(/^\/assets\/.+\.js$/);
  const wasm = await page.evaluate(async (scriptUrl) => {
    const source = await (await fetch(scriptUrl)).text();
    const path = /\/assets\/[\w-]+\.wasm/.exec(source)?.[0];
    if (path === undefined) {
      return { path, status: 0, type: null, compiled: false };
    }
    const fetched = await fetch(path);
    const type = fetched.headers.get("content-type");
    const module = await WebAssembly.compileStreaming(fetched);
    return { path, status: fetched.status, type, compiled: module instanceof WebAssembly.Module };
  }, script ?? "");
  expect(wasm).toEqual({ path: expect.stringMatching(/\.wasm$/), status: 200, type: "application/wasm", compiled: true });

  expect(await page.evaluate(() => fetch("/api/v1/healthz").then((answer) => answer.status))).toBe(200);
  expect(await page.evaluate(() => window.cspViolations)).toEqual([]);
});

test("pricing: the plans show their prices and limits, say they are not on sale, and offer no way to buy, only a way in", async ({ page }) => {
  await installFakeApi(page);
  await page.goto("/");

  const pricing = page.getByTestId("pricing");
  await expect(pricing.getByRole("heading", { name: messages.pricing.heading })).toBeVisible();
  await expect(pricing.getByText(messages.pricing.notOnSale)).toBeVisible();
  await expect(pricing.getByText(messages.pricing.free.price(PRICES_USD.free), { exact: true })).toBeVisible();
  await expect(pricing.getByText(messages.pricing.creator.price(PRICES_USD.creatorMonthly, PRICES_USD.creatorAnnual))).toBeVisible();
  for (const feature of [...messages.pricing.free.features(LIMITS), ...messages.pricing.creator.features]) {
    await expect(pricing.getByText(feature)).toBeVisible();
  }
  await expect(pricing.getByRole("button")).toHaveCount(0);
  await expect(pricing.getByRole("link")).toHaveCount(1);
  await expect(pricing.getByRole("link", { name: messages.landing.cta.startFree, exact: true })).toHaveAttribute("href", "/app");
});

test("the footer reaches each legal page, and each has its title, its date and a contact link", async ({ page }) => {
  await installFakeApi(page);
  for (const doc of LEGAL_DOCS) {
    const copy = messages.legal.docs[doc];
    await page.goto("/");
    await page.getByTestId("site-footer").getByRole("link", { name: copy.linkLabel, exact: true }).click();
    await expect(page).toHaveURL(new RegExp(`/legal/${doc}$`));
    await expect(page.getByRole("heading", { name: copy.title, level: 1 })).toBeVisible();
    await expect(page.getByText(messages.legal.updated)).toBeVisible();
    await expect(page.getByRole("link", { name: messages.legal.contactEmail })).toHaveAttribute("href", `mailto:${messages.legal.contactEmail}`);
  }
});

test("the footer's contact link writes to the contact address", async ({ page }) => {
  await installFakeApi(page);
  await page.goto("/");
  await expect(page.getByTestId("site-footer").getByRole("link", { name: messages.legal.footerContact })).toHaveAttribute("href", `mailto:${messages.legal.contactEmail}`);
});

test("a legal document that does not exist goes back to the landing page", async ({ page }) => {
  await installFakeApi(page);
  await page.goto("/legal/nothing");
  await expect(page.getByRole("heading", { name: messages.landing.hero })).toBeVisible();
  await expect(page).toHaveURL(/\/$/);
});

// --- The design release (designimplementation §12.3) ---------------------------
// New cases only: every case above is as it was, except the pricing case's
// declared change (§12.4).

/** Where "Start free" leads into the app: the top bar, the hero, pricing and the final call. */
const START_FREE_LINKS = 4;
const startFree = (scope: Pick<Page, "getByRole">) => scope.getByRole("link", { name: messages.landing.cta.startFree, exact: true });

test("L-3: every Start free leads to /app, which shows the drop zone or the unsupported page", async ({ page }) => {
  await installFakeApi(page);
  await page.goto("/");

  const links = startFree(page);
  await expect(links).toHaveCount(START_FREE_LINKS);
  for (const link of await links.all()) {
    await expect(link).toHaveAttribute("href", "/app");
  }
  await startFree(page.getByRole("banner")).click();
  await expect(page).toHaveURL(/\/app$/);
  await expect(dropZone(page).or(page.getByTestId("unsupported-reason"))).toBeVisible({ timeout: 15_000 });
});

test("L-10: each string the tests look for is on the page once, by its name and by a part of it", async ({ page }) => {
  await installFakeApi(page);
  await page.goto("/");
  const pricing = page.getByTestId("pricing");
  const footer = page.getByTestId("site-footer");

  for (const exact of [false, true]) {
    await expect(page.getByRole("heading", { name: messages.landing.hero, exact })).toHaveCount(1);
    await expect(page.getByText(messages.landing.supportedBrowsers, { exact })).toHaveCount(1);
    await expect(page.getByRole("link", { name: messages.landing.whatLeavesLink, exact })).toHaveCount(1);
    await expect(page.getByRole("button", { name: messages.dropZone.sampleButton, exact })).toHaveCount(1);
    await expect(page.getByRole("button", { name: messages.notifyMe.submit, exact })).toHaveCount(1);
    await expect(page.getByLabel(messages.notifyMe.emailLabel, { exact })).toHaveCount(1);
    await expect(pricing.getByRole("heading", { name: messages.pricing.heading, exact })).toHaveCount(1);
    await expect(page.getByText(messages.pricing.notOnSale, { exact })).toHaveCount(1);
    for (const doc of LEGAL_DOCS) {
      await expect(footer.getByRole("link", { name: messages.legal.docs[doc].linkLabel, exact })).toHaveCount(1);
    }
    await expect(footer.getByRole("link", { name: messages.legal.footerContact, exact })).toHaveCount(1);
  }
  // The privacy line shares its list item with the link, so only a part of that item is the line.
  await expect(page.getByText(messages.landing.privacyLine)).toHaveCount(1);
  // The waitlist cases read the status of the page's one form.
  await expect(page.locator("form")).toHaveCount(1);
});

/** The sentences of the old waitlist, which no longer appear anywhere on the page (§13). */
const WAITLIST_WORDS = [
  "Offcut is not open yet. Leave your email and we will tell you when it is.",
  "not open yet",
  "Leave your email and we will tell you when it is",
];

/** The clips of the page, one per slot of designimplementation §9.1, all uploaded (Entry 9). */
const CLIP_SLOTS = 7;

/** The page's widths in the design reviews, and a phone's. */
const REVIEW_SIZES = [
  { width: 1280, height: 800 },
  { width: 1920, height: 1080 },
  { width: 390, height: 844 },
];

test("L-1: every feature card of the page is named, and the page never scrolls sideways", async ({ page }) => {
  await installFakeApi(page);
  await page.goto("/");
  for (const card of Object.values(messages.landing.features.cards)) {
    await expect(page.getByRole("heading", { name: card.title, exact: true })).toBeVisible();
  }
  for (const size of REVIEW_SIZES) {
    await page.setViewportSize(size);
    await expect.poll(() => page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth)).toBeLessThanOrEqual(0);
  }
});

test("L-2: every feature after V6 is on the page, each marked Coming soon", async ({ page }) => {
  await installFakeApi(page);
  await page.goto("/");
  const soon = messages.landing.soon;
  const items = [...soon.next.items, ...soon.later.items];
  const cards = page.getByTestId("soon-card");
  await expect(cards).toHaveCount(items.length);
  for (const [index, item] of items.entries()) {
    await expect(cards.nth(index)).toContainText(soon.chip);
    await expect(cards.nth(index).getByRole("heading", { name: item.title, exact: true })).toBeVisible();
    await expect(cards.nth(index)).toContainText(item.line);
  }
  await expect(page.getByText(soon.footnote)).toBeVisible();
});

test("L-4: no sentence on the page says that Offcut is not open", async ({ page }) => {
  await installFakeApi(page);
  await page.goto("/");
  await expect(page.getByRole("heading", { name: messages.landing.updatesHeading })).toBeVisible();
  const text = await page.locator("body").innerText();
  for (const words of WAITLIST_WORDS) {
    expect(text).not.toContain(words);
  }
});

/** Scrolls from the top of the page to its end in `steps` steps, as a reader would. */
async function scrollThrough(page: Page, steps = 20): Promise<void> {
  const height = await page.evaluate(() => document.documentElement.scrollHeight - window.innerHeight);
  for (let step = 1; step <= steps; step += 1) {
    await page.evaluate((top) => window.scrollTo(0, top), Math.round((height * step) / steps));
    await page.waitForTimeout(150);
  }
}

test("L-5: with reduced motion the page is complete and still", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await installFakeApi(page);
  await page.goto("/");
  await scrollThrough(page);
  await page.evaluate(() => window.scrollTo(0, 0));
  await page.waitForTimeout(1000);

  expect(await page.evaluate(() => document.getAnimations().length)).toBe(0);
  await expect(page.getByRole("heading", { name: messages.landing.hero })).toBeVisible();
  await expect(page.getByRole("heading", { name: messages.landing.final.line })).toBeAttached();
  await expect(page.locator(".pin-spacer")).toHaveCount(0);
  // No caption pill: the headlines are plain text (M1, M10).
  for (const pill of await page.locator("[data-pill]").all()) {
    await expect(pill).toHaveCSS("opacity", "0");
  }
  await expect(page.locator("[data-lit]")).toHaveCount(0);
  // Every clip stands still on its poster, with a button to play it (D-326).
  const clips = page.locator("[data-clip]");
  await expect(clips).toHaveCount(CLIP_SLOTS);
  for (const clip of await clips.all()) {
    const video = clip.locator("video");
    expect(await video.evaluate((element) => (element as HTMLVideoElement).paused)).toBe(true);
    await expect(video).toHaveAttribute("poster", /\/assets\/.+\.webp$/);
    await expect(clip.getByRole("button", { name: messages.landing.playClip })).toBeVisible();
  }
});

test("L-7: scrolling the whole page breaks no rule of the security policy, and the page stays isolated", async ({ page }) => {
  await installFakeApi(page);
  await page.addInitScript(() => {
    window.cspViolations = [];
    document.addEventListener("securitypolicyviolation", (event) => {
      window.cspViolations?.push(`${event.violatedDirective} ${event.blockedURI}`);
    });
  });
  await page.goto("/");
  await capabilityChecked(page);
  await scrollThrough(page);
  await page.waitForTimeout(500);

  expect(await page.evaluate(() => window.cspViolations)).toEqual([]);
  expect(await page.evaluate(() => crossOriginIsolated)).toBe(true);
});

test("L-8: the editor's route downloads no GSAP, and the landing page does", async ({ page }) => {
  await installFakeApi(page);
  const scripts: Promise<string>[] = [];
  page.on("response", (response) => {
    if (new URL(response.url()).pathname.endsWith(".js")) {
      scripts.push(response.text().catch(() => ""));
    }
  });
  await page.goto("/app");
  await expect(dropZone(page).or(page.getByTestId("unsupported-reason"))).toBeVisible({ timeout: 15_000 });
  await page.waitForTimeout(1000);
  const onApp = await Promise.all(scripts);
  expect(onApp.length).toBeGreaterThan(0);
  expect(onApp.filter((body) => body.includes("ScrollTrigger"))).toEqual([]);

  scripts.length = 0;
  await page.goto("/");
  await scrollThrough(page, 5);
  await expect.poll(async () => (await Promise.all(scripts)).some((body) => body.includes("ScrollTrigger"))).toBe(true);
});

test("L-9: leaving for /app inside the app leaves nothing of the landing page behind", async ({ page }) => {
  await installFakeApi(page);
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await scrollThrough(page);
  await page.evaluate(() => window.scrollTo(0, 0));
  await page.waitForTimeout(500);

  await startFree(page.getByRole("banner")).click();
  await expect(page).toHaveURL(/\/app$/);
  await expect(dropZone(page).or(page.getByTestId("unsupported-reason"))).toBeVisible({ timeout: 15_000 });
  await expect(page.locator(".pin-spacer")).toHaveCount(0);
  await expect(page.locator("[data-landing]")).toHaveCount(0);
  expect(errors).toEqual([]);
});

test("L-6: every clip loads from the asset host under the isolation headers, and plays", async ({ page }) => {
  await installFakeApi(page);
  const response = await page.goto("/");
  const csp = response?.headers()["content-security-policy"] ?? "";
  // The policy names the asset host in more than one directive: one host.
  const assetHosts = [...new Set((csp.match(/https?:\/\/[^\s;]+/g) ?? []).map((host) => new URL(host).origin))];
  expect(assetHosts).toHaveLength(1);

  const videos = page.locator("[data-clip] video");
  await expect(videos).toHaveCount(CLIP_SLOTS);
  for (const video of await videos.all()) {
    await video.scrollIntoViewIfNeeded();
    await expect
      .poll(() => video.evaluate((element) => (element as HTMLVideoElement).readyState), { timeout: 20_000 })
      .toBeGreaterThanOrEqual(2);
    const state = await video.evaluate((element) => {
      const clip = element as HTMLVideoElement;
      return { error: clip.error === null ? null : clip.error.code, paused: clip.paused, origin: new URL(clip.currentSrc).origin };
    });
    expect(state).toEqual({ error: null, paused: false, origin: assetHosts[0] });
  }
  expect(await page.evaluate(() => crossOriginIsolated)).toBe(true);
});

/** An accessibility violation, as the reviews quote one: its rule and how many elements break it. */
const violations = async (page: Page) => (await axe(page)).map((v) => `${v.id}: ${String(v.nodes.length)} node(s)`);

test("L-12: the settings and a legal page, on the dark system, have no serious or critical accessibility violation", async ({ page }) => {
  await installFakeApi(page);
  for (const path of ["/settings", "/legal/terms"]) {
    await page.goto(path);
    await expect(page.locator("main[data-theme='dark']")).toBeVisible();
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
    expect(await violations(page), `axe on ${path}`).toEqual([]);
  }
});

test("L-11: the landing page has no serious or critical accessibility violation, at the top or at the end", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await installFakeApi(page);
  await page.goto("/");
  await capabilityChecked(page);
  expect(await violations(page), "axe at the top").toEqual([]);
  await scrollThrough(page);
  await page.waitForTimeout(500);
  expect(await violations(page), "axe at the end").toEqual([]);
});
