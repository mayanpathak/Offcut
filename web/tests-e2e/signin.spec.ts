// J9 and C-8 in a real browser, from `/account`: no GPU is needed, so this runs in
// CI's `pnpm e2e` (v6implementation §15.4). The API is a fake. The dialog that holds
// the same form is read in `export-free.spec.ts`, as it opens only on a clip.

import { expect, type Page, test } from "@playwright/test";

import { PRICES_USD } from "../src/config/pricing";
import { messages } from "../src/copy/messages";
import { LIMITS } from "../src/gen/domain";
import { FAKE_EXPIRED_LINK, FAKE_INVALID_LINK, installFakeApi, mintEntitlementToken, seedEntitlement } from "./helpers/fake-api";
import { axe, readIndexedDb, sendQueuedAnalytics } from "./helpers/fixtures";

const EMAIL = "someone@example.com";
const GOOD_LINK = "fake-good-link";

const violations = async (page: Page) => (await axe(page)).map((v) => `${v.id}: ${String(v.nodes.length)} node(s)`);
const limits = () => messages.signIn.limits(LIMITS.FREE_EXPORTS_PER_MONTH, PRICES_USD.creatorMonthly);
const emailField = (page: Page) => page.getByTestId("sign-in-email");
const submit = (page: Page) => page.getByRole("button", { name: messages.signIn.submit, exact: true });
const statusLine = (page: Page) => page.getByTestId("sign-in-status");

async function sendLink(page: Page): Promise<void> {
  await emailField(page).fill(EMAIL);
  await submit(page).click();
  await expect(page.getByTestId("sign-in-sent")).toBeVisible();
}

test("the limits come before the email field, and asking for nothing sends nothing", async ({ page }) => {
  const api = await installFakeApi(page);
  await page.goto("/account");
  await expect(page.getByTestId("sign-in-limits")).toHaveText(limits());
  const [above, field] = await Promise.all([page.getByTestId("sign-in-limits").boundingBox(), emailField(page).boundingBox()]);
  expect((above?.y ?? 0) + (above?.height ?? 0)).toBeLessThanOrEqual(field?.y ?? 0);
  expect(await violations(page), "axe on the form").toEqual([]);
  expect(api.requests.filter((request) => request.path === "/auth/magic-link")).toEqual([]);
});

test("what cannot be an address is said in place, and nothing is sent", async ({ page }) => {
  const api = await installFakeApi(page);
  await page.goto("/account");
  await emailField(page).fill("no-at-sign");
  await submit(page).click();
  await expect(statusLine(page)).toHaveText(messages.notifyMe.invalidEmail);
  expect(await violations(page), "axe with the error").toEqual([]);
  expect(api.requests.filter((request) => request.path === "/auth/magic-link")).toEqual([]);
});

test("a link asked for: the address goes to the server, the page says where to open it, and it can be asked again", async ({ page }) => {
  const api = await installFakeApi(page);
  await page.goto("/account");
  await sendLink(page);
  const sent = page.getByTestId("sign-in-sent");
  await expect(sent).toContainText(messages.signIn.sentTitle);
  await expect(sent).toContainText(messages.signIn.sentBody(EMAIL));
  await expect(sent).toContainText(messages.signIn.spamHint);
  expect(await violations(page), "axe on the sent state").toEqual([]);
  await page.getByRole("button", { name: messages.signIn.resend, exact: true }).click();
  await expect.poll(() => api.requests.filter((request) => request.path === "/auth/magic-link").map((request) => request.body)).toEqual([{ email: EMAIL }, { email: EMAIL }]);
  await expect(sent).toBeVisible();
  await sendQueuedAnalytics(page);
  await expect.poll(() => api.eventsOf("signin_step").length).toBe(2);
  expect(JSON.stringify(api.eventsOf("signin_step"))).not.toContain(EMAIL);
});

test("two tabs: the link opened in a new tab signs the first in through the broadcast", async ({ context }) => {
  const first = await context.newPage();
  const firstApi = await installFakeApi(first);
  await first.goto("/account");
  await expect(submit(first)).toBeVisible();
  await expect.poll(() => firstApi.requests.some((request) => request.path === "/auth/refresh")).toBe(true);
  await sendLink(first);

  const second = await context.newPage();
  const secondApi = await installFakeApi(second, { account: firstApi.account });
  await second.goto(`/auth/callback?token=${GOOD_LINK}`);
  await expect(second.getByTestId("callback-done")).toContainText(messages.callback.signedIn);
  expect(await violations(second), "axe on the callback").toEqual([]);

  // No token crossed: the first tab asked the server for itself.
  await expect(first.getByTestId("account-signed-in")).toBeVisible();
  expect(firstApi.requests.filter((request) => request.path === "/auth/refresh")).toHaveLength(2);
  expect(firstApi.requests.filter((request) => request.path === "/auth/verify")).toEqual([]);
  expect(secondApi.requests.filter((request) => request.path === "/auth/verify").map((request) => request.body)).toEqual([{ token: GOOD_LINK }]);

  await second.getByRole("link", { name: messages.callback.openApp }).click();
  await expect(second).toHaveURL(/\/app$/);
});

test("the token leaves the address before the verify is sent, and no refresh goes from the callback (D-296)", async ({ page }) => {
  await page.addInitScript(() => {
    const send = window.fetch.bind(window);
    window.fetch = (input, init) => {
      if ((input instanceof Request ? input.url : input.toString()).endsWith("/auth/verify")) {
        window.addressAtVerify = window.location.href;
      }
      return send(input, init);
    };
  });
  const api = await installFakeApi(page);
  await page.goto(`/auth/callback?token=${GOOD_LINK}`);
  await expect(page.getByTestId("callback-done")).toBeVisible();
  const address = await page.evaluate(() => window.addressAtVerify);
  expect(address).toMatch(/\/auth\/callback$/);
  expect(page.url()).toMatch(/\/auth\/callback$/);
  await page.waitForTimeout(500);
  expect(api.requests.filter((request) => request.path === "/auth/refresh")).toEqual([]);
  expect(JSON.stringify(api.requests.filter((request) => request.path === "/events"))).not.toContain(GOOD_LINK);
});

for (const [name, link, code] of [
  ["an expired link", FAKE_EXPIRED_LINK, "E_AUTH_LINK_EXPIRED"],
  ["a used link", FAKE_INVALID_LINK, "E_AUTH_LINK_INVALID"],
] as const) {
  test(`${name}: its words, gone from the address, and a new link offered in place`, async ({ page }) => {
    const api = await installFakeApi(page);
    await page.goto(`/auth/callback?token=${link}`);
    const failed = page.getByTestId("callback-failed");
    await expect(failed.getByRole("heading")).toHaveText(messages.errors[code].title);
    await expect(failed).toContainText(messages.errors[code].body);
    expect(page.url()).toMatch(/\/auth\/callback$/);
    expect(await violations(page), "axe on the failed callback").toEqual([]);
    await sendLink(page);
    expect(api.requests.filter((request) => request.path === "/auth/magic-link")).toHaveLength(1);
  });
}

test("a link with no token is one that does not work, and asks nothing of the server", async ({ page }) => {
  const api = await installFakeApi(page);
  await page.goto("/auth/callback");
  await expect(page.getByTestId("callback-failed")).toHaveAttribute("data-code", "E_AUTH_LINK_INVALID");
  expect(api.requests.filter((request) => request.path === "/auth/verify")).toEqual([]);
});

test("a connection that fails offers another try with the same link, which is not in the address", async ({ page }) => {
  const api = await installFakeApi(page, { failures: { "/auth/verify": { status: 500, code: "internal" } } });
  await page.goto(`/auth/callback?token=${GOOD_LINK}`);
  const failed = page.getByTestId("callback-failed");
  await expect(failed).toHaveAttribute("data-code", "E_API_5XX");
  expect(page.url()).toMatch(/\/auth\/callback$/);
  await failed.getByRole("button", { name: messages.recovery.tryAgain }).click();
  await expect.poll(() => api.requests.filter((request) => request.path === "/auth/verify").map((request) => request.body)).toEqual([{ token: GOOD_LINK }, { token: GOOD_LINK }]);
});

test("a sleeping API: the waking line after 3 s, then the link is sent", async ({ page }) => {
  await installFakeApi(page, { healthz: { delayMs: 4_500 } });
  await page.goto("/account");
  await emailField(page).fill(EMAIL);
  await submit(page).click();
  await expect(statusLine(page)).toHaveText(messages.signIn.sending);
  await expect(statusLine(page)).toHaveText(messages.api.waking, { timeout: 4_000 });
  await expect(page.getByTestId("sign-in-sent")).toBeVisible({ timeout: 10_000 });
});

test("the email service down: its words in place, one attempt, and the form still there", async ({ page }) => {
  const api = await installFakeApi(page, { failures: { "/auth/magic-link": { status: 503, code: "email_unavailable" } } });
  await page.goto("/account");
  await emailField(page).fill(EMAIL);
  await submit(page).click();
  const { title, action } = messages.errors.E_AUTH_EMAIL_UNAVAILABLE;
  await expect(statusLine(page)).toHaveText(`${title}. ${action}`);
  await expect(emailField(page)).toHaveValue(EMAIL);
  expect(api.requests.filter((request) => request.path === "/auth/magic-link")).toHaveLength(1);
});

test("too many requests: the wait is said", async ({ page }) => {
  await installFakeApi(page, { failures: { "/auth/magic-link": { status: 429, code: "rate_limited", retryAfterSecs: 90 } } });
  await page.goto("/account");
  await emailField(page).fill(EMAIL);
  await submit(page).click();
  await expect(statusLine(page)).toHaveText(messages.notifyMe.rateLimited(90));
});

test("a session the browser still holds signs in at start; signing out ends it and takes the token off the device", async ({ page }) => {
  const api = await installFakeApi(page, { session: {} });
  await page.goto("/account");
  await expect(page.getByTestId("account-signed-in")).toBeVisible();
  expect(await violations(page), "axe signed in").toEqual([]);
  await expect.poll(() => readIndexedDb(page, "entitlement", "current")).toBeDefined();

  await page.getByRole("button", { name: messages.account.signOut }).click();
  await expect(submit(page)).toBeVisible();
  await expect.poll(() => readIndexedDb(page, "entitlement", "current")).toBeUndefined();
  expect(api.requests.filter((request) => request.path === "/auth/logout")).toHaveLength(1);

  await page.reload();
  await expect(submit(page)).toBeVisible();
  await expect(page.getByTestId("account-signed-in")).toHaveCount(0);
});

test("a sign-out that fails says so, and the person is still signed in", async ({ page }) => {
  await installFakeApi(page, { session: {}, failures: { "/auth/logout": { status: 500, code: "internal" } } });
  await page.goto("/account");
  await page.getByRole("button", { name: messages.account.signOut }).click();
  const { title, action } = messages.errors.E_API_5XX;
  await expect(page.getByTestId("sign-out-status")).toHaveText(`${title}. ${action}`);
  await expect(page.getByTestId("account-signed-in")).toBeVisible();
});

test("a lost session keeps the token on the device (D-291)", async ({ page }) => {
  await installFakeApi(page);
  await page.goto("/account");
  await expect(submit(page)).toBeVisible();
  const token = mintEntitlementToken({ plan: "free" });
  await seedEntitlement(page, token);
  await page.reload();
  await expect(submit(page)).toBeVisible();
  await page.waitForTimeout(500);
  expect(await readIndexedDb(page, "entitlement", "current")).toMatchObject({ value: { token } });
});

declare global {
  interface Window {
    addressAtVerify?: string;
  }
}

