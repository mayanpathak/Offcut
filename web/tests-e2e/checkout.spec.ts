// J12 and C-11, C-18 in a real browser, from `/account`: no GPU is needed, so this
// runs in CI's `pnpm e2e` (v6implementation §15.4). The API is a fake, and so is the
// merchant: its checkout and portal are pages of a host that resolves nowhere,
// answered here. The prompt in the editor is read in `export-free.spec.ts`.

import { readFile } from "node:fs/promises";

import { expect, type Page, test } from "@playwright/test";

import { PRICES_USD } from "../src/config/pricing";
import { messages } from "../src/copy/messages";
import { LIMITS } from "../src/gen/domain";
import { FAKE_CHECKOUT_URL, FAKE_PORTAL_URL, type FakeApi, installFakeApi } from "./helpers/fake-api";
import { axe, readIndexedDb, sendQueuedAnalytics } from "./helpers/fixtures";

const EMAIL = "someone@example.com";
/** TS C-11: the page asks every 2 s, for up to 60 s. Written here, as the use-case would bring the app's configuration with it. */
const POLL_EVERY_MS = 2_000;
const POLL_FOR_MS = 60_000;
const DAY_SECS = 24 * 3600;
const RENEWS_AT = Math.floor(Date.now() / 1000) + 20 * DAY_SECS;
const ACTIVE = { status: "active", interval: "monthly", current_period_end: RENEWS_AT, cancel_at_period_end: false };
/** How many presses of Tab may reach a control from the top of the page. */
const MAX_TABS = 20;

const violations = async (page: Page) => (await axe(page)).map((v) => `${v.id}: ${String(v.nodes.length)} node(s)`);
const button = (page: Page, name: string) => page.getByRole("button", { name, exact: true });
const monthly = (page: Page) => button(page, messages.upgrade.monthly(PRICES_USD.creatorMonthly));
const annual = (page: Page) => button(page, messages.upgrade.annual(PRICES_USD.creatorAnnual));
const notice = (page: Page) => page.getByTestId("checkout-notice");
const planLine = (page: Page) => page.getByTestId("account-plan-line");
const pathsOf = (api: FakeApi, path: string) => api.requests.filter((request) => request.path === path);

/** The merchant's two pages, each a page of its own that says which it is. */
async function routeMerchant(page: Page): Promise<void> {
  for (const [url, title] of [
    [FAKE_CHECKOUT_URL, "Fake checkout"],
    [FAKE_PORTAL_URL, "Fake portal"],
  ] as const) {
    await page.route(url, (route) => route.fulfill({ contentType: "text/html", body: `<!doctype html><title>${title}</title>` }));
  }
}

/** The plan the token on this device says, read from its claims. */
async function storedPlan(page: Page): Promise<unknown> {
  const record = (await readIndexedDb(page, "entitlement", "current")) as { value?: { token?: string } } | undefined;
  const payload = record?.value?.token?.split(".")[0];
  return payload === undefined ? undefined : (JSON.parse(Buffer.from(payload, "base64url").toString("utf8")) as { plan: unknown }).plan;
}

/** Presses Tab until `target` has the focus, as a person with a keyboard would reach it. */
async function tabTo(page: Page, target: ReturnType<typeof button>): Promise<void> {
  for (let i = 0; i < MAX_TABS; i += 1) {
    await page.keyboard.press("Tab");
    if (await target.evaluate((element) => element === document.activeElement)) {
      return;
    }
  }
  throw new Error(`no control took the focus after ${String(MAX_TABS)} presses of Tab`);
}

async function openSignedIn(page: Page, options: Parameters<typeof installFakeApi>[1] = {}): Promise<FakeApi> {
  const api = await installFakeApi(page, { session: { email: EMAIL }, ...options });
  await routeMerchant(page);
  await page.goto("/account");
  await expect(page.getByTestId("account-signed-in")).toBeVisible();
  return api;
}

test("Free: the plan and its count, the two offers, and the monthly one goes to the merchant's checkout", async ({ page }) => {
  const api = await openSignedIn(page);
  await expect(page.getByTestId("account-signed-in")).toContainText(messages.account.email(EMAIL));
  await expect(planLine(page)).toHaveText(messages.account.free(LIMITS.FREE_EXPORTS_PER_MONTH, LIMITS.FREE_EXPORTS_PER_MONTH));
  await expect(annual(page)).toBeVisible();
  await expect(button(page, messages.account.manage)).toHaveCount(0);
  expect(await violations(page), "axe on Free").toEqual([]);

  await monthly(page).click();
  await expect(page).toHaveURL(FAKE_CHECKOUT_URL);
  expect(pathsOf(api, "/billing/checkout").map((request) => request.body)).toEqual([{ offer: "creator_monthly" }]);
  await expect.poll(() => api.eventsOf("checkout_step")).toEqual([{ name: "checkout_step", props: { step: "started" } }]);
});

test("by keyboard alone: Tab reaches the yearly offer, and Enter goes to the checkout for it", async ({ page }) => {
  const api = await openSignedIn(page);
  await tabTo(page, annual(page));
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(FAKE_CHECKOUT_URL);
  expect(pathsOf(api, "/billing/checkout").map((request) => request.body)).toEqual([{ offer: "creator_annual" }]);
});

test("back from a payment: asked every 2 s until the plan is Creator, then said; the address is clean and the device's token says Creator", async ({ page }) => {
  const api = await installFakeApi(page, { session: { email: EMAIL } });
  await page.goto("/account?checkout=success");
  await expect(page).toHaveURL(/\/account$/);
  await expect(notice(page)).toHaveText(messages.account.confirming);
  await expect.poll(() => pathsOf(api, "/me").length).toBeGreaterThanOrEqual(2);
  // The webhook arrives at the server.
  api.setSession({ email: EMAIL, plan: "creator", subscription: ACTIVE });
  await expect(notice(page)).toHaveText(messages.account.active, { timeout: 3 * POLL_EVERY_MS });
  await expect(planLine(page)).toHaveText(messages.account.creator);
  await expect(page.getByTestId("account-subscription")).toHaveText(
    messages.account.term(messages.account.interval.monthly, messages.account.renews(messages.account.date(RENEWS_AT))),
  );
  await expect(monthly(page)).toHaveCount(0);
  await expect.poll(() => storedPlan(page)).toBe("creator");
  expect(await violations(page), "axe on Creator, just bought").toEqual([]);
  await sendQueuedAnalytics(page);

  await page.reload();
  await expect(planLine(page)).toHaveText(messages.account.creator);
  await expect(notice(page)).toHaveCount(0);
  await sendQueuedAnalytics(page);
  expect(api.eventsOf("checkout_step")).toEqual([{ name: "checkout_step", props: { step: "returned_success" } }]);
});

test("no Creator within the minute: E_BILLING_PENDING, and Check again waits once more", async ({ page }) => {
  await page.clock.install();
  const api = await installFakeApi(page, { session: { email: EMAIL } });
  await page.goto("/account?checkout=success");
  await expect(notice(page)).toHaveText(messages.account.confirming);
  await expect.poll(() => pathsOf(api, "/me").length).toBeGreaterThanOrEqual(2);
  await page.clock.fastForward(POLL_FOR_MS + POLL_EVERY_MS);
  const { title, body } = messages.errors.E_BILLING_PENDING;
  await expect(notice(page)).toHaveAttribute("data-code", "E_BILLING_PENDING");
  await expect(notice(page)).toContainText(title);
  await expect(notice(page)).toContainText(body);
  expect(await violations(page), "axe on the pending notice").toEqual([]);

  api.setSession({ email: EMAIL, plan: "creator", subscription: ACTIVE });
  await button(page, messages.account.checkAgain).click();
  await expect(notice(page)).toHaveText(messages.account.active);
  await sendQueuedAnalytics(page);
  expect(api.eventsOf("checkout_step"), "the return is told once").toHaveLength(1);
});

test("a cancelled checkout: said, told once, and the plan as it was", async ({ page }) => {
  const api = await installFakeApi(page, { session: { email: EMAIL } });
  await page.goto("/account?checkout=cancel");
  await expect(page).toHaveURL(/\/account$/);
  await expect(notice(page)).toHaveText(messages.account.cancelled);
  await expect(planLine(page)).toHaveText(messages.account.free(LIMITS.FREE_EXPORTS_PER_MONTH, LIMITS.FREE_EXPORTS_PER_MONTH));
  await sendQueuedAnalytics(page);
  expect(api.eventsOf("checkout_step")).toEqual([{ name: "checkout_step", props: { step: "returned_cancel" } }]);
});

test("Creator: the renewal date, no offers, and Manage subscription opens the portal", async ({ page }) => {
  const api = await openSignedIn(page, { session: { email: EMAIL, plan: "creator", subscription: ACTIVE } });
  await expect(planLine(page)).toHaveText(messages.account.creator);
  await expect(page.getByTestId("account-subscription")).toContainText(messages.account.renews(messages.account.date(RENEWS_AT)));
  await expect(monthly(page)).toHaveCount(0);
  expect(await violations(page), "axe on Creator").toEqual([]);
  await button(page, messages.account.manage).click();
  await expect(page).toHaveURL(FAKE_PORTAL_URL);
  expect(pathsOf(api, "/billing/checkout")).toEqual([]);
});

test("a subscription that is ending says when, and a failed payment says what to do", async ({ page }) => {
  const api = await openSignedIn(page, { session: { email: EMAIL, plan: "creator", subscription: { ...ACTIVE, cancel_at_period_end: true } } });
  await expect(page.getByTestId("account-subscription")).toContainText(messages.account.ends(messages.account.date(RENEWS_AT)));
  api.setSession({ email: EMAIL, plan: "creator", subscription: { ...ACTIVE, status: "past_due" } });
  await page.reload();
  await expect(page.getByTestId("account-subscription")).toContainText(messages.account.pastDue);
  await expect(button(page, messages.account.manage)).toBeVisible();
});

test("a subscription begun elsewhere: the checkout is refused, so the offer opens the portal (D-300)", async ({ page }) => {
  const ended = { ...ACTIVE, status: "canceled" };
  const api = await openSignedIn(page, { session: { email: EMAIL, subscription: ended }, failures: { "/billing/checkout": { status: 403, code: "forbidden" } } });
  await monthly(page).click();
  await expect(page).toHaveURL(FAKE_PORTAL_URL);
  expect(pathsOf(api, "/billing/checkout")).toHaveLength(1);
});

test("the payment service down: said in place, one attempt, and nobody leaves", async ({ page }) => {
  const api = await openSignedIn(page, { failures: { "/billing/checkout": { status: 502, code: "billing_unavailable" } } });
  await monthly(page).click();
  const { title, action } = messages.errors.E_BILLING_UNAVAILABLE;
  await expect(page.getByTestId("billing-status")).toHaveText(`${title}. ${action}`);
  await expect(page).toHaveURL(/\/account$/);
  expect(pathsOf(api, "/billing/checkout")).toHaveLength(1);
  expect(await violations(page), "axe with the error").toEqual([]);
});

test("export my data: one JSON file of what the server holds", async ({ page }) => {
  await openSignedIn(page);
  const arriving = page.waitForEvent("download");
  await button(page, messages.account.exportData).click();
  const download = await arriving;
  expect(download.suggestedFilename()).toBe("offcut-account-data.json");
  const data = JSON.parse(await readFile(await download.path(), "utf8")) as { user: { email: string } };
  expect(data.user.email).toBe(EMAIL);
  await expect(page.getByTestId("data-status")).toHaveText(messages.account.exported);
});

test("delete, by keyboard: asked first, with the focus on keeping it; Escape keeps it; deleting signs out and takes the token off the device", async ({ page }) => {
  const api = await openSignedIn(page);
  await expect.poll(() => readIndexedDb(page, "entitlement", "current")).toBeDefined();
  await tabTo(page, button(page, messages.account.delete));
  await page.keyboard.press("Enter");
  const dialog = page.getByTestId("delete-dialog");
  await expect(dialog).toContainText(messages.account.deleteDialog.body);
  await expect(button(page, messages.account.deleteDialog.keep)).toBeFocused();
  expect(await violations(page), "axe on the question").toEqual([]);
  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);
  expect(pathsOf(api, "/account/delete")).toEqual([]);

  await tabTo(page, button(page, messages.account.delete));
  await page.keyboard.press("Enter");
  await page.keyboard.press("Tab");
  await expect(button(page, messages.account.deleteDialog.confirm)).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.getByTestId("account-deleted")).toHaveText(messages.account.deleted);
  await expect(page.getByTestId("sign-in-email")).toBeVisible();
  await expect.poll(() => readIndexedDb(page, "entitlement", "current")).toBeUndefined();
  expect(pathsOf(api, "/account/delete")).toHaveLength(1);
  expect(await violations(page), "axe after the deletion").toEqual([]);
});

test("a deletion the merchant would not cancel: said in the question, and the account stays", async ({ page }) => {
  await openSignedIn(page, { failures: { "/account/delete": { status: 502, code: "billing_unavailable" } } });
  await button(page, messages.account.delete).click();
  await button(page, messages.account.deleteDialog.confirm).click();
  const { title, action } = messages.errors.E_BILLING_UNAVAILABLE;
  await expect(page.getByTestId("delete-status")).toHaveText(`${title}. ${action}`);
  await button(page, messages.account.deleteDialog.keep).click();
  await expect(page.getByTestId("account-signed-in")).toBeVisible();
  expect(await readIndexedDb(page, "entitlement", "current")).toBeDefined();
});
