// J1, J3 and C-16 in a real browser, against the built app served with the
// production headers. The API is a fake: no case here touches a real server,
// except `@smoke`, which is the one case also run against a deployment.
//
// The cases hold on a browser that can run Offcut and on one that cannot:
// the browser of CI may lack WebGPU.

import { expect, type Page, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import { ANALYTICS_EVENT_DOCS, type CheckResult, type GpuVendor, type MemoryBucket, type Platform } from "../src/gen/api";
import { LIMITS, UNSUPPORTED_REASONS } from "../src/gen/domain";
import { installFakeApi } from "./helpers/fake-api";
import { flushAnalytics } from "./helpers/fixtures";

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
  // Let the start of the app finish its own requests first: the wake-up
  // call is the last one it always makes.
  await expect.poll(() => api.requests.some((entry) => entry.path === "/healthz")).toBe(true);

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
    // The file is no clip: the page says so, and offers the way back.
    await expect(page.getByText(messages.editor.rejectedStub)).toBeVisible({ timeout: 15_000 });
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
