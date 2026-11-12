// The design reviews' captures (v4implementation D-191, G 10.8): the eight
// states of §17.2, V5's done state (D-235) and V6's sign-in, limits, prompts and
// account page (v6implementation §13) at the two reference widths, written to
// test-results/screens/<state>-<width>x<height>.png. D1 only: `pnpm screens`
// builds the page in development mode first, so that it has its development
// hooks (`__offcutDev`, D-155), and CI never runs this project. Each picture
// is taken once the fonts have loaded and the page's movement has ended; the
// pictures are never committed (SAB rule 9).

import { expect, type Page, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import { PRICES_USD } from "../src/config/pricing";
import { installFakeApi } from "./helpers/fake-api";
import { type AssetOptions, dropClip, ensureModelCached, modelManifest, patient, referenceClip, routeAssets } from "./helpers/fixtures";

declare global {
  interface Window {
    /** The page's development hooks (`web/src/ui/dev-hooks.ts`), in a development build only. */
    __offcutDev?: {
      openWordField(i: number): void;
      failClip(code: string): void;
      fakeExport(fraction: number): void;
      fakeExportDone(): void;
      fakeUpgrade(placement: "after_export" | "limit_reached"): void;
    };
  }
}

const WIDTHS = [
  { width: 1280, height: 800 },
  { width: 1920, height: 1080 },
];
const OUT = "test-results/screens";
/** Longer than the interface's slowest movement (`--base`, 200 ms) and a toast's entrance. */
const SETTLE_MS = 600;
/** The model download is held at about this share: its requests fail from there on. */
const DOWNLOAD_HELD_AT = 0.3;
/** The least share of the bar at which the download is pictured, in percent. */
const DOWNLOAD_SHOWN_FROM = 25;
/**
 * The error pictured. `E_GPU_LOST` (G 10.8) has no words until Prompt 119
 * and would show the general card; this code has its title, its sentence and
 * what to do (v4changelog open item 9).
 */
const ERROR_CODE = "E_WORKER_CRASH";
const EXPORT_FRACTION = 0.42;
/** How long the clip plays before the ready picture: past its first caption. */
const PLAYED_MS = 3000;
const FOUND = messages.feed.eventFound.number_reveal("");
const EMAIL = "someone@example.com";
const DAY_SECS = 24 * 3600;
/** A subscription that renews in 20 days, as the account page shows one. */
const ACTIVE = { status: "active", interval: "monthly", current_period_end: Math.floor(Date.now() / 1000) + 20 * DAY_SECS, cancel_at_period_end: false };
/** Past the minute the account page waits for a payment (TS C-11). */
const PAST_THE_WAIT_MS = 62_000;

/** Waits for the fonts and, unless the state passes quickly, for the movement to end; then takes the picture. */
async function picture(page: Page, state: string, settle = true): Promise<void> {
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  if (settle) {
    await page.waitForTimeout(SETTLE_MS);
  }
  const { width, height } = page.viewportSize() ?? { width: 0, height: 0 };
  await page.screenshot({ path: `${OUT}/${state}-${String(width)}x${String(height)}.png` });
}

async function openEditor(page: Page, assets: AssetOptions = {}): Promise<void> {
  await installFakeApi(page);
  await routeAssets(page, assets);
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
  const hooks = await page.evaluate(() => window.__offcutDev !== undefined);
  expect(hooks, "the page has no development hooks: run `pnpm screens`, which builds it in development mode").toBe(true);
}

// The processing picture can miss its moment on a fast run: one more try.
test.describe.configure({ retries: 1 });

for (const size of WIDTHS) {
  test.describe(`${String(size.width)} x ${String(size.height)}`, () => {
    test.use({ viewport: size });

    test("model download, at about 30%", async ({ page }) => {
      // A fresh context: the model is not on this device.
      const total = modelManifest.files.reduce((sum, file) => sum + file.bytes, 0);
      await openEditor(page, { failAfterBytes: Math.round(total * DOWNLOAD_HELD_AT) });
      await dropClip(page, referenceClip());
      const bar = page.getByTestId("model-download").getByRole("progressbar");
      await expect
        .poll(async () => Number((await bar.getAttribute("aria-valuenow")) ?? "0"), { timeout: patient(120_000) })
        .toBeGreaterThanOrEqual(DOWNLOAD_SHOWN_FROM);
      await picture(page, "model-download");
    });

    test("empty, processing, ready, editing a word, style tab, exporting, done, error", async ({ page }) => {
      await ensureModelCached(page.context());
      await openEditor(page);
      await picture(page, "empty");

      await dropClip(page, referenceClip());
      // Processing, once the second step is done: the visual moments are being
      // looked for, and the found ones are chips. That lasts a second or less,
      // so the page is read at every frame, and the picture taken at once.
      const reached = await page.waitForFunction(
        (found) => {
          const card = document.querySelector('[data-testid="processing"]');
          if (card !== null && [...card.querySelectorAll("li")].some((line) => (line.textContent ?? "").startsWith(found))) {
            return "processing";
          }
          return document.querySelector('[data-testid="preview-canvas"]') === null ? false : "ready";
        },
        FOUND,
        { polling: "raf", timeout: patient(240_000) },
      );
      expect(await reached.jsonValue(), "the processing card was gone before its picture").toBe("processing");
      await picture(page, "processing", false);

      // Ready, with a frame of the clip on the stage: the preview draws while
      // it plays (a paused frame is redrawn from Prompt 112, D-173).
      const play = page.getByRole("button", { name: messages.preview.play, exact: true });
      await expect(play).toBeEnabled({ timeout: patient(240_000) });
      // Clicked without moving the pointer: no tooltip in the picture.
      await play.dispatchEvent("click");
      await page.waitForTimeout(PLAYED_MS);
      await page.getByRole("button", { name: messages.preview.pause, exact: true }).dispatchEvent("click");
      await expect(play).toBeEnabled({ timeout: 30_000 });
      await picture(page, "ready");

      await page.evaluate((i) => window.__offcutDev?.openWordField(i), 0);
      await expect(page.getByRole("textbox", { name: messages.captions.wordField })).toBeFocused();
      await picture(page, "editing-word");

      await page.getByRole("tab", { name: messages.panel.style }).click();
      await picture(page, "style-tab");

      await page.evaluate((fraction) => window.__offcutDev?.fakeExport(fraction), EXPORT_FRACTION);
      await expect(page.getByTestId("export-progress")).toBeVisible();
      await picture(page, "exporting");

      await page.evaluate(() => window.__offcutDev?.fakeExportDone());
      await expect(page.getByTestId("export-outcome")).toBeVisible();
      await picture(page, "done");

      await page.evaluate((code) => window.__offcutDev?.failClip(code), ERROR_CODE);
      await expect(page.getByTestId("clip-failed")).toBeVisible();
      await picture(page, "error");
    });

    test("V6, the editor: the sign-in dialog, the link sent, the callback, the counter, the last export, both prompts", async ({ page, context }) => {
      await ensureModelCached(context);
      const api = await installFakeApi(page);
      await routeAssets(page);
      await page.goto("/app");
      await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
      await dropClip(page, referenceClip());
      await expect(page.getByRole("button", { name: messages.preview.play, exact: true })).toBeEnabled({ timeout: patient(240_000) });
      const exportButton = page.getByRole("button", { name: messages.export.button, exact: true });

      await exportButton.dispatchEvent("click");
      const dialog = page.getByTestId("sign-in-dialog");
      await expect(dialog).toBeVisible();
      await picture(page, "sign-in-dialog");
      await dialog.getByTestId("sign-in-email").fill(EMAIL);
      await dialog.getByRole("button", { name: messages.signIn.submit, exact: true }).click();
      await expect(dialog.getByTestId("sign-in-sent")).toBeVisible();
      await picture(page, "sign-in-sent");

      // The link opened in a tab of its own, for an account with one export left.
      api.setSession({ email: EMAIL, freeExportsRemaining: 1 });
      const tab = await context.newPage();
      await installFakeApi(tab, { account: api.account });
      await tab.goto("/auth/callback?token=fake-good-link");
      await expect(tab.getByTestId("callback-done")).toBeVisible();
      await picture(tab, "callback-signed-in");
      await tab.close();

      await expect(dialog).toHaveCount(0);
      await expect(page.getByTestId("export-counter")).toBeVisible();
      await picture(page, "counter");
      await exportButton.dispatchEvent("click");
      await expect(page.getByTestId("last-export-dialog")).toBeVisible();
      await picture(page, "last-export");
      await page.getByTestId("last-export-dialog").getByRole("button", { name: messages.export.lastFree.notNow }).click();

      await page.evaluate(() => window.__offcutDev?.fakeExportDone());
      await page.evaluate(() => window.__offcutDev?.fakeUpgrade("after_export"));
      await expect(page.getByTestId("upgrade-prompt")).toContainText(messages.upgrade.afterExport(PRICES_USD.creatorMonthly));
      await picture(page, "upgrade-after-export");
      await page.evaluate(() => window.__offcutDev?.fakeUpgrade("limit_reached"));
      await expect(page.getByTestId("upgrade-prompt")).toHaveAttribute("data-placement", "limit_reached");
      await picture(page, "upgrade-limit");
    });

    test("V6, the account page: signed out, Free, Creator, a payment not confirmed yet, the delete question", async ({ page }) => {
      await page.clock.install();
      const api = await installFakeApi(page);
      await page.goto("/account");
      await expect(page.getByTestId("sign-in-email")).toBeVisible();
      await picture(page, "account-signed-out");

      api.setSession({ email: EMAIL, freeExportsRemaining: 2 });
      await page.reload();
      await expect(page.getByTestId("account-plan-line")).toBeVisible();
      await picture(page, "account-free");

      api.setSession({ email: EMAIL, plan: "creator", subscription: ACTIVE });
      await page.reload();
      await expect(page.getByTestId("account-subscription")).toBeVisible();
      await picture(page, "account-creator");

      api.setSession({ email: EMAIL });
      await page.goto("/account?checkout=success");
      await expect(page.getByTestId("checkout-notice")).toHaveText(messages.account.confirming);
      // Past the session's restore first: the jump would time out a refresh still on its way.
      await expect(page.getByTestId("account-plan-line")).toBeVisible();
      await page.clock.fastForward(PAST_THE_WAIT_MS);
      await expect(page.getByTestId("checkout-notice")).toHaveAttribute("data-code", "E_BILLING_PENDING");
      await picture(page, "account-payment-pending");

      await page.getByRole("button", { name: messages.account.delete, exact: true }).click();
      await expect(page.getByTestId("delete-dialog")).toBeVisible();
      await picture(page, "account-delete-question");
    });
  });
}

// The landing page for design review 3 (designimplementation §12.5): the whole
// page at the two reference widths and a phone's, with movement and without,
// after a scroll through so that every entrance has played.
const LANDING_SIZES = [...WIDTHS, { width: 390, height: 844 }];
const MOTIONS = [
  { name: "normal", reducedMotion: "no-preference" },
  { name: "reduced", reducedMotion: "reduce" },
] as const;
/** Long enough for the headline's pill to come to rest (M1). */
const HEADLINE_MS = 3000;

for (const size of LANDING_SIZES) {
  for (const motion of MOTIONS) {
    test(`landing, ${motion.name} motion, ${String(size.width)} x ${String(size.height)}`, async ({ page }) => {
      await page.setViewportSize(size);
      await page.emulateMedia({ reducedMotion: motion.reducedMotion });
      await installFakeApi(page);
      await page.goto("/");
      await page.evaluate(() => document.fonts.ready.then(() => undefined));
      const height = await page.evaluate(() => document.documentElement.scrollHeight);
      for (let top = 0; top < height; top += Math.round(size.height * 0.6)) {
        await page.evaluate((y) => window.scrollTo(0, y), top);
        await page.waitForTimeout(200);
      }
      await page.evaluate(() => window.scrollTo(0, 0));
      await page.waitForTimeout(HEADLINE_MS);
      await page.screenshot({ path: `${OUT}/landing-${motion.name}-${String(size.width)}x${String(size.height)}.png`, fullPage: true });
      await page.screenshot({ path: `${OUT}/landing-${motion.name}-${String(size.width)}x${String(size.height)}-first-screen.png` });
    });
  }
}

// --- V6.2, P194: the AI plan's screens (v6.2implementation §12) ---------------
// On a development build with `ai_plan` on only:
// `OFFCUT_FEATURES_ON=ai_plan pnpm screens --grep "V6.2"`. The fake API has
// no `/plan` (P210), and nobody is signed in: the clip falls back to the rules
// and says so; the model's plan is put on the list by the `fakePlan` hook.
// Every state is read by axe (no serious or critical violation).

type PlanHooks = {
  flags(): { ai_plan: boolean };
  fakePlan(): void;
  fakePlanFallback(why: string): void;
};

const planHooks = (page: Page) => ({
  flags: () => page.evaluate(() => (window as unknown as { __offcutDev?: PlanHooks }).__offcutDev?.flags().ai_plan ?? false),
  fakePlan: () => page.evaluate(() => (window as unknown as { __offcutDev?: PlanHooks }).__offcutDev?.fakePlan()),
  fallback: (why: string) =>
    page.evaluate((w) => (window as unknown as { __offcutDev?: PlanHooks }).__offcutDev?.fakePlanFallback(w), why),
});

async function noSeriousViolation(page: Page, state: string): Promise<void> {
  const { axe } = await import("./helpers/fixtures");
  const found = (await axe(page)).map((v) => `${v.id}: ${String(v.nodes.length)} node(s)`);
  expect(found, `axe on ${state}`).toEqual([]);
}

for (const size of WIDTHS) {
  test(`V6.2, the plan: the visuals list, the text for your post, Copied, a fallback, regenerate, the settings switch, ${String(size.width)} x ${String(size.height)}`, async ({ page, context }) => {
    await page.setViewportSize(size);
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);
    await ensureModelCached(context);
    await openEditor(page);
    const hooks = planHooks(page);
    test.skip(!(await hooks.flags()), "a build with ai_plan on: OFFCUT_FEATURES_ON=ai_plan pnpm screens");

    await dropClip(page, referenceClip());
    // Signed out: no request, the rules plan, and the line that signing in gives one (D-494).
    await expect(page.getByText(messages.plan.fallback.signedOut)).toBeVisible({ timeout: patient(240_000) });
    await expect(page.getByRole("button", { name: messages.preview.play, exact: true })).toBeEnabled({ timeout: patient(240_000) });

    await page.getByRole("tab", { name: messages.panel.visuals }).click();
    await hooks.fakePlan();
    await expect(page.getByTestId("plan-items").locator('[data-kind="hook_title"]')).toBeVisible();
    await expect(page.getByTestId("regenerate-plan")).toBeVisible();
    await picture(page, "plan-visuals");
    await noSeriousViolation(page, "the visuals list");

    const extras = page.getByTestId("text-extras");
    await extras.scrollIntoViewIfNeeded();
    await expect(extras.getByTestId("extra-hashtags")).toBeVisible();
    await picture(page, "plan-text-extras");
    await noSeriousViolation(page, "the text for your post");

    const hook = extras.getByTestId("extra-hook");
    await hook.getByRole("button", { name: messages.plan.extras.copy, exact: true }).click();
    await expect(hook.getByRole("button", { name: messages.plan.extras.copied, exact: true })).toBeVisible();
    expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(await hook.locator("p").innerText());
    await picture(page, "plan-copied", false);

    await hooks.fallback("quota");
    await expect(page.getByText(messages.plan.fallback.quota)).toBeVisible();
    await picture(page, "plan-fallback-quota", false);
    await noSeriousViolation(page, "a fallback's toast");

    // Signed out, "Regenerate plan" asks nothing and says why; the plan stays.
    await page.getByTestId("regenerate-plan").click();
    await expect(page.getByText(messages.plan.regenerateWhy.signed_out)).toBeVisible();
    await expect(page.getByTestId("plan-items").locator('[data-kind="hook_title"]')).toBeVisible();
    await picture(page, "plan-regenerate-signed-out", false);
    await noSeriousViolation(page, "regenerate's toast");

    await page.goto("/settings");
    const setting = page.getByTestId("ai-plan-switch");
    await expect(setting).toHaveAttribute("aria-checked", "true");
    await expect(page.getByTestId("what-leaves")).toContainText("DeepSeek");
    await picture(page, "settings-ai-plan-on");
    await noSeriousViolation(page, "the settings switch on");
    await setting.click();
    await expect(setting).toHaveAttribute("aria-checked", "false");
    await picture(page, "settings-ai-plan-off");
    await noSeriousViolation(page, "the settings switch off");
    // Kept on this device: a reload reads it back.
    await page.reload();
    await expect(page.getByTestId("ai-plan-switch")).toHaveAttribute("aria-checked", "false");
  });
}
