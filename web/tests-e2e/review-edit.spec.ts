// The editor in Chrome on D1 (v4implementation §20.4, G 16.3; D-145 to
// D-147, D-169 to D-187, D-192): every control a creator uses, through the
// gestures and keys they use, and an accessibility scan. Sixteen cases on five
// clip loads (v5implementation D-201): twelve share one processed clip, made
// clean between them by `resetClip`; a landscape clip, a portrait clip, the case
// that breaks a change on purpose and the one that times the first draw of the
// style cards have their own.
//
// What the preview shows is read from the player's mirrors of the visible
// caption (`data-caption-text`, `-colors`, `-position`), which the test build
// has (`pnpm build:e2e`, D-155), never from the pixels of a word (§24 item
// 13). Analytics is read from the fake API: names and enum props only. No
// word of a clip is written to the report or the console (SAB rule 9).

import { type Page, expect, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import type { FakeApi } from "./helpers/fake-api";
import {
  axe,
  captionMirror,
  dragCaptions,
  dragVideo,
  editWord,
  genFixture,
  hoverPreview,
  openClip,
  openShared,
  openTab,
  openWord,
  patient,
  previewAt,
  referenceClip,
  resetClip,
  saves,
  seekTo,
  setStyle,
  settled,
  type SharedClip,
  toggleItem,
  transcriptWords,
  wordButton,
  wordField,
} from "./helpers/fixtures";

/** Analytics sends what it has every 10 s; a case waits that long for an event, and a little more. */
const FLUSH_WAIT_MS = 25_000;
/** The style cards' three images (assumption, D-186; §21.1). */
const THUMBNAILS_MS = 400;
/** One output frame, at 30 fps. */
const FRAME_MS = 1000 / 30;
/** The caption block's centre at the bottom placement (the default): `CAPTION_TRACK.bottom` of preview-interact.ts. */
const BOTTOM_CENTRE = 1304 / 1920;

const button = (page: Page, name: string) => page.getByRole("button", { name, exact: true });
const canvas = (page: Page) => page.getByTestId("preview-canvas");
const failureCard = (page: Page) => page.getByTestId("preview-failed");

/** The `review_action` events the fake API received, by their action. */
async function reviewActions(api: FakeApi, atLeast: number): Promise<string[]> {
  await expect.poll(() => api.eventsOf("review_action").length, { timeout: FLUSH_WAIT_MS }).toBeGreaterThanOrEqual(atLeast);
  return api.eventsOf("review_action").map((event) => (event as { props: { action: string } }).props.action);
}

/** The words of a caption text, lower case, without punctuation. */
function tokens(text: string): string[] {
  return text
    .toLowerCase()
    .split(/[^\p{L}\p{N}]+/u)
    .filter((token) => token !== "");
}

/** A word of letters only, `min` long or more, heard once in the transcript, away from both ends of it. */
async function loneWord(page: Page, min = 5): Promise<{ index: number; text: string }> {
  const words = await transcriptWords(page);
  const counts = new Map<string, number>();
  for (const word of words) {
    counts.set(word.text.toLowerCase(), (counts.get(word.text.toLowerCase()) ?? 0) + 1);
  }
  const middle = words.slice(Math.floor(words.length / 4), Math.floor((words.length * 3) / 4));
  const found = middle.find((word) => /^\p{L}+$/u.test(word.text) && word.text.length >= min && counts.get(word.text.toLowerCase()) === 1);
  if (found === undefined) {
    throw new Error("the transcript has no word heard once");
  }
  return found;
}

/** Seeks to word `index` through the transcript and closes its field: the preview paused there, a caption shown. */
async function showWord(page: Page, index: number): Promise<void> {
  await openWord(page, index);
  await page.keyboard.press("Escape");
  await expect(wordField(page)).toHaveCount(0);
  await page.waitForTimeout(300);
}

/** Takes the focus off every control: a click on the stage's empty corner. */
async function blur(page: Page): Promise<void> {
  await page.locator("main").click({ position: { x: 4, y: 4 } });
  await expect.poll(() => page.evaluate(() => document.activeElement === document.body)).toBe(true);
}

/** The visible caption's text, once the mirror has caught up with what `accept` waits for. */
async function captionWhen(page: Page, accept: (text: string) => boolean, what: string): Promise<string> {
  await expect.poll(async () => accept((await captionMirror(page)).text), { timeout: 10_000, message: what }).toBe(true);
  return (await captionMirror(page)).text;
}

test.describe("the editor (review-edit), on one processed clip", () => {
  // Not "serial": a failed case must not stop the others. Playwright starts a
  // new worker after a failure, which runs beforeAll again: a new clip.
  test.describe.configure({ timeout: patient(420_000) });
  let shared: SharedClip;

  test.beforeAll(async ({ browser }, info) => {
    test.setTimeout(patient(420_000));
    shared = await openShared(browser, info, referenceClip());
  });

  test.afterAll(async () => {
    await shared.context.close();
  });

  test.beforeEach(async () => {
    await resetClip(shared.page, shared.api);
  });

  test("1. edit a number: 90 fixed to 9, the from-to's row reads 9 minutes, and the edit is counted", async () => {
    const { page, api } = shared;
    const ninety = (await transcriptWords(page)).find((word) => word.text === "90");
    expect(ninety, "the reference clip's transcript has the word 90").toBeDefined();
    await openTab(page, "Visuals");
    const row = page.locator('li[data-kind="from_to"]');
    await expect(row).toContainText("90 minutes");

    await editWord(page, ninety?.index ?? -1, "9");
    await openTab(page, "Visuals");
    await expect(row).toContainText("9 minutes");
    await expect(row).not.toContainText("90 minutes");
    expect(await reviewActions(api, 1)).toEqual(["word_edit"]);
  });

  test("2. hide a word: an empty commit takes it out of the caption, and the transcript strikes it through", async () => {
    const { page } = shared;
    const word = await loneWord(page);
    await showWord(page, word.index);
    await captionWhen(page, (text) => tokens(text).includes(word.text.toLowerCase()), "the caption shows the word");

    await editWord(page, word.index, "");
    await captionWhen(page, (text) => !tokens(text).includes(word.text.toLowerCase()), "the caption no longer shows the word");
    const struck = await wordButton(page, word.index).evaluate((element) => getComputedStyle(element).textDecorationLine);
    expect(struck).toContain("line-through");
  });

  test("3. toggle a visual: the list off leaves its frame, stays off after a word edit elsewhere, and comes back on", async () => {
    const { page } = shared;
    await openTab(page, "Visuals");
    const row = page.locator('li[data-kind="list_reveal"]').first();
    const control = row.getByRole("switch");
    await expect(control).toHaveAttribute("aria-checked", "true");
    // The list's time, and a second into it, where the list is drawn whole.
    await row.getByRole("button", { name: /^Go to / }).click();
    await page.waitForTimeout(400);
    const at = await previewAt(page);
    await seekTo(page, at + 1_000);
    await page.waitForTimeout(500);
    const withList = await canvas(page).screenshot();

    await toggleItem(page, "List", false);
    await page.waitForTimeout(300);
    const without = await canvas(page).screenshot();
    expect(withList.equals(without), "the frame at the list's time changed").toBe(false);
    await expect(page.getByText(/^3 of 4 visuals on$/)).toBeVisible();

    // A word edit elsewhere finds the events again; the choice made stays (D-145).
    const words = await transcriptWords(page);
    const far = words.find((word, i) => i > words.length - 8 && /^\p{L}{3,}$/u.test(word.text));
    await editWord(page, far?.index ?? words.length - 1, "fixed");
    await openTab(page, "Visuals");
    await expect(control).toHaveAttribute("aria-checked", "false");
    await expect(page.getByText(/^3 of 4 visuals on$/)).toBeVisible();

    await toggleItem(page, "List", true);
    await page.waitForTimeout(300);
    expect((await canvas(page).screenshot()).equals(without), "the list is drawn again").toBe(false);
  });

  test("4. style: Bold writes the caption in capitals", async () => {
    const { page } = shared;
    await showWord(page, (await loneWord(page)).index);
    const clean = await captionWhen(page, (text) => /\p{Ll}/u.test(text), "a caption in Clean, with small letters");
    await setStyle(page, "Bold");
    const bold = await captionWhen(page, (text) => text !== "" && text === text.toUpperCase() && /\p{Lu}/u.test(text), "the caption in capitals");
    expect(bold).toBe(clean.toUpperCase());
  });

  test("7. pause and restore: a change while playing pauses the preview and plays it again after", async () => {
    const { page } = shared;
    await button(page, messages.preview.play).click();
    await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 30_000 });
    await page.waitForTimeout(1_000);
    // What the transport's one button says, every time it changes.
    await page.evaluate(() => {
      const w = window as unknown as { e2eLabels: string[] };
      w.e2eLabels = [];
      new MutationObserver((records) => {
        for (const record of records) {
          const label = (record.target as Element).getAttribute("aria-label") ?? "";
          if (label === "Play" || label === "Pause") {
            w.e2eLabels.push(label);
          }
        }
      }).observe(document, { subtree: true, attributes: true, attributeFilter: ["aria-label"] });
    });
    await setStyle(page, "Bold");
    await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 10_000 });
    const labels = await page.evaluate(() => (window as unknown as { e2eLabels: string[] }).e2eLabels);
    expect(labels, "paused for the change, then playing again").toEqual(["Play", "Pause"]);
    const first = await previewAt(page);
    await page.waitForTimeout(800);
    expect(await previewAt(page)).toBeGreaterThan(first);
    await button(page, messages.preview.pause).click();
  });

  test("9. click a word in the transcript: the preview goes to it, paused; the field selects it; Tab commits and opens the next; Escape cancels", async () => {
    const { page } = shared;
    await button(page, messages.preview.play).click();
    await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 30_000 });
    const words = await transcriptWords(page);
    const pick = words.findIndex((word, i) => i > words.length / 2 && /^\p{L}{3,}$/u.test(word.text) && /^\p{L}{2,}$/u.test(words[i + 1]?.text ?? ""));
    const word = words[pick];
    const next = words[pick + 1];
    if (word === undefined || next === undefined) {
      throw new Error("no two words to fix one after the other");
    }

    // The block's time, the clock of its first word, read while the word is a button (its field takes its place).
    const blockAt = await wordButton(page, word.index).evaluate((element) => element.closest("div")?.querySelector("button")?.textContent ?? "");
    const [minutes, seconds] = blockAt.split(":").map(Number);
    const blockMs = ((minutes ?? 0) * 60 + (seconds ?? 0)) * 1000;
    await openWord(page, word.index);
    await expect(button(page, messages.preview.play)).toBeVisible({ timeout: 10_000 });
    // The preview is at the word: at or after its block's time.
    await expect.poll(() => previewAt(page)).toBeGreaterThanOrEqual(blockMs);
    expect(await previewAt(page)).toBeLessThan(blockMs + 20_000);
    const selection = await wordField(page).evaluate((element) => {
      const input = element as HTMLInputElement;
      return { value: input.value, start: input.selectionStart, end: input.selectionEnd };
    });
    expect(selection).toEqual({ value: word.text, start: 0, end: word.text.length });

    // Tab: the word is fixed, and the next one's field opens.
    await page.keyboard.type("alpha");
    const since = await saves(page);
    await page.keyboard.press("Tab");
    await settled(page, since);
    await expect(wordButton(page, word.index)).toHaveText("alpha");
    await expect(wordField(page)).toBeFocused();
    await expect(wordField(page)).toHaveValue(next.text);

    // Escape: nothing is fixed, and the focus is on the word again.
    await page.keyboard.type("zzz");
    await page.keyboard.press("Escape");
    await expect(wordField(page)).toHaveCount(0);
    await expect(wordButton(page, next.index)).toHaveText(next.text);
    await expect(wordButton(page, next.index)).toBeFocused();
  });

  test("10. click a caption word on the preview: the Captions tab opens on that word", async () => {
    const { page } = shared;
    await showWord(page, (await loneWord(page)).index);
    await openTab(page, "Style");
    let hit = false;
    for (const y of [BOTTOM_CENTRE, BOTTOM_CENTRE - 0.02, BOTTOM_CENTRE + 0.02]) {
      for (const x of [0.5, 0.42, 0.58, 0.34, 0.66, 0.28, 0.72]) {
        if (await hoverPreview(page, x, y, ["word"])) {
          hit = true;
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
    await expect(page.getByRole("tab", { name: "Captions", exact: true })).toHaveAttribute("aria-selected", "true");
    await expect(wordField(page)).toBeFocused();
    const opened = await wordField(page).inputValue();
    expect(tokens((await captionMirror(page)).text)).toContain(tokens(opened)[0]);
  });

  test("11. replace all: a word heard three times fixed once offers Change all 2; all three change; one undo gives all three back", async () => {
    const { page } = shared;
    const words = await transcriptWords(page);
    const groups = new Map<string, number[]>();
    for (const word of words) {
      if (/^\p{L}{3,}$/u.test(word.text)) {
        const key = word.text.toLowerCase();
        groups.set(key, [...(groups.get(key) ?? []), word.index]);
      }
    }
    const three = [...groups.values()].find((indices) => indices.length === 3);
    expect(three, "the reference clip's transcript has a word heard three times").toBeDefined();
    const [first, ...others] = three ?? [];
    const originals = await Promise.all((three ?? []).map((i) => wordButton(page, i).textContent()));

    await editWord(page, first ?? -1, "Zebra");
    const toast = page.getByTestId("toasts");
    await expect(toast).toContainText("Change all 2");
    const since = await saves(page);
    await toast.getByRole("button", { name: messages.captions.changeAll }).click();
    await settled(page, since);
    for (const i of three ?? []) {
      await expect(wordButton(page, i)).toHaveText("Zebra");
    }
    expect(others).toHaveLength(2);

    await blur(page);
    const before = await saves(page);
    await page.keyboard.press("ControlOrMeta+z");
    await settled(page, before);
    for (const [n, i] of (three ?? []).entries()) {
      await expect(wordButton(page, i)).toHaveText(originals[n] ?? "");
    }
  });

  test("12. undo and redo with the keyboard, across a word edit, a style change and a toggle", async () => {
    const { page } = shared;
    const word = await loneWord(page);
    await editWord(page, word.index, "edited");
    await setStyle(page, "Bold");
    await toggleItem(page, "List", false);

    const key = async (combo: string): Promise<void> => {
      await blur(page);
      const since = await saves(page);
      await page.keyboard.press(combo);
      await settled(page, since);
    };
    const listOn = async (): Promise<string | null> => {
      await openTab(page, "Visuals");
      return page.locator('li[data-kind="list_reveal"]').first().getByRole("switch").getAttribute("aria-checked");
    };
    const boldOn = async (): Promise<string | null> => {
      await openTab(page, "Style");
      return page.getByTestId("style-cards").getByRole("button", { name: /^Bold/ }).getAttribute("aria-pressed");
    };
    const wordText = async (): Promise<string | null> => {
      await openTab(page, "Captions");
      return wordButton(page, word.index).textContent();
    };

    await key("ControlOrMeta+z");
    expect(await listOn()).toBe("true");
    await key("ControlOrMeta+z");
    expect(await boldOn()).toBe("false");
    await key("ControlOrMeta+z");
    expect(await wordText()).toBe(word.text);

    await key("Shift+ControlOrMeta+z");
    expect(await wordText()).toBe("edited");
    await key("Shift+ControlOrMeta+z");
    expect(await boldOn()).toBe("true");
    await key("Shift+ControlOrMeta+z");
    expect(await listOn()).toBe("false");
  });

  test("13. drag the captions to the top: they snap at Top, and the move is one change", async () => {
    const { page, api } = shared;
    await showWord(page, (await loneWord(page)).index);
    expect((await captionMirror(page)).position).toBe(1);
    await dragCaptions(page, "top");
    await expect.poll(async () => (await captionMirror(page)).position).toBe(0.15);
    await openTab(page, "Framing");
    await expect(page.getByRole("radio", { name: messages.framing.top })).toHaveAttribute("aria-checked", "true");
    expect(await reviewActions(api, 1)).toEqual(["caption_position"]);
  });

  test("15. shortcuts: Space plays and pauses, types a space in a field; the arrows step one frame while paused", async () => {
    const { page } = shared;
    await blur(page);
    await page.keyboard.press("Space");
    await expect(button(page, messages.preview.pause)).toBeVisible({ timeout: 10_000 });
    await page.waitForTimeout(500);
    await page.keyboard.press("Space");
    await expect(button(page, messages.preview.play)).toBeVisible({ timeout: 10_000 });

    const word = await loneWord(page);
    await openWord(page, word.index);
    await page.keyboard.press("End");
    await page.keyboard.press("Space");
    await expect(wordField(page)).toHaveValue(`${word.text} `);
    await expect(button(page, messages.preview.play)).toBeVisible();
    await page.keyboard.press("Escape");

    await blur(page);
    await page.waitForTimeout(300);
    // The word's time may fall inside a frame: a first step lands on the next frame's start, from which one frame is
    // one step of the scrub bar (whose value is on that grid).
    const before = await previewAt(page);
    await page.keyboard.press("ArrowRight");
    await expect.poll(() => previewAt(page)).toBeGreaterThanOrEqual(before);
    await page.waitForTimeout(300);
    const at = await previewAt(page);
    await page.keyboard.press("ArrowRight");
    await expect.poll(() => previewAt(page)).toBeGreaterThan(at);
    const step = (await previewAt(page)) - at;
    expect(step, "one frame forward").toBeGreaterThan(FRAME_MS / 2);
    expect(step).toBeLessThan(FRAME_MS * 1.5 + 1);
    await page.keyboard.press("ArrowLeft");
    await expect.poll(async () => Math.abs((await previewAt(page)) - at)).toBeLessThan(FRAME_MS);
  });

  test("16. accessibility: no serious or critical axe violation, ready or editing; every control reachable by Tab with a visible focus ring", async ({}, testInfo) => {
    const { page } = shared;
    const ready = await axe(page);
    expect(ready.map((v) => `${v.id}: ${v.nodes.length} node(s)`), "axe on the ready state").toEqual([]);
    await openWord(page, (await loneWord(page)).index);
    const editing = await axe(page);
    expect(editing.map((v) => `${v.id}: ${v.nodes.length} node(s)`), "axe on the editing state").toEqual([]);
    await page.keyboard.press("Escape");

    for (const tab of ["Captions", "Style", "Visuals", "Framing"]) {
      await openTab(page, tab);
      await blur(page);
      // Every element that takes the focus by Tab: the page's own controls, not a roving group's other members.
      const expected = await page.evaluate(() => {
        const all = [...document.querySelectorAll<HTMLElement>("button, input, select, textarea, a[href], [tabindex]")];
        return all
          .filter((el) => el.tabIndex >= 0 && !(el as HTMLButtonElement).disabled && el.offsetParent !== null && el.closest("[inert]") === null)
          .map((el, i) => {
            el.dataset.e2eStop = String(i);
            return String(i);
          });
      });
      const reached = new Set<string>();
      const unreadable = new Set<string>();
      for (let presses = 0; presses < expected.length + 10; presses += 1) {
        await page.keyboard.press("Tab");
        const now = await page.evaluate(() => {
          const el = document.activeElement as HTMLElement | null;
          if (el === null || el === document.body) {
            return null;
          }
          const style = getComputedStyle(el);
          const ring = (style.outlineStyle !== "none" && parseFloat(style.outlineWidth) > 0) || style.boxShadow !== "none";
          return { stop: el.dataset.e2eStop ?? "", ring };
        });
        if (now === null) {
          continue;
        }
        reached.add(now.stop);
        if (!now.ring) {
          unreadable.add(now.stop);
        }
      }
      const missed = expected.filter((stop) => !reached.has(stop));
      expect(missed, `${tab}: controls Tab does not reach`).toEqual([]);
      // A ring the computed style does not show (a range's thumb draws its own): the pixels around the control,
      // focused from the keyboard and not, must differ.
      const ringless: string[] = [];
      for (const stop of unreadable) {
        const control = page.locator(`[data-e2e-stop="${stop}"]`);
        const box = await control.boundingBox();
        if (box === null) {
          continue;
        }
        const clip = { x: Math.max(0, box.x - 6), y: Math.max(0, box.y - 6), width: box.width + 12, height: box.height + 12 };
        await control.focus();
        await page.keyboard.press("Shift");
        const focused = await page.screenshot({ clip });
        await control.evaluate((el) => {
          (el as HTMLElement).blur();
        });
        const plain = await page.screenshot({ clip });
        if (focused.equals(plain)) {
          ringless.push((await control.getAttribute("aria-label")) ?? (await control.evaluate((el) => el.tagName)));
        }
      }
      testInfo.annotations.push({
        type: "tab stops",
        description: `${tab}: ${String(expected.length)} stops, all reached; ${String(unreadable.size)} ring(s) read from the pixels`,
      });
      expect(ringless, `${tab}: controls without a visible focus ring`).toEqual([]);
    }
  });
});

test.describe("the editor (review-edit), on a clip of its own", () => {
  test.describe.configure({ timeout: patient(420_000) });

  test("5. crop on a landscape clip: the control exists, and a drag of the video to the right edge is one change", async ({ page }) => {
    const api = await openClip(page, genFixture("speech/speech_60s_landscape.mp4"));
    await openTab(page, "Framing");
    const slider = page.getByTestId("crop-offset");
    await expect(slider).toBeVisible();
    expect(Number(await slider.inputValue())).toBe(0);
    // Grabbing the video and moving it left shows its right part: past one width, the window is at the right edge.
    await dragVideo(page, -1.15);
    await expect.poll(async () => Number(await slider.inputValue())).toBe(1);
    expect(await reviewActions(api, 1)).toEqual(["crop_offset"]);
  });

  test("6. no crop on a portrait clip: the control does not exist", async ({ page }) => {
    // The reference clip is wider than 9:16 and has the control: the 60 s portrait fixture has none.
    await openClip(page, genFixture("speech/speech_60s_portrait.mp4"));
    await openTab(page, "Framing");
    await expect(page.getByRole("radiogroup", { name: messages.framing.captionPosition })).toBeVisible();
    await expect(page.getByTestId("crop-offset")).toHaveCount(0);
  });

  test("8. undo after a failed edit: the card offers Undo this change, which restores the edit before", async ({ page }) => {
    await openClip(page, referenceClip());
    const word = await loneWord(page);
    await showWord(page, word.index);
    const before = await captionWhen(page, (text) => text !== "", "a caption");
    await openWord(page, word.index);
    await wordField(page).fill("broken");
    await page.evaluate(() => {
      (window as unknown as { __offcutDev: { failNextSetScene(): void } }).__offcutDev.failNextSetScene();
    });
    await page.keyboard.press("Enter");
    await expect(failureCard(page)).toBeVisible({ timeout: 10_000 });
    await expect(failureCard(page)).toContainText(messages.errors.E_INTERNAL.title);
    await expect(wordButton(page, word.index)).toHaveText(word.text);

    await failureCard(page).getByRole("button", { name: messages.recovery.undoChange }).click();
    await expect(failureCard(page)).toHaveCount(0, { timeout: 10_000 });
    await expect(wordButton(page, word.index)).toHaveText(word.text);
    expect((await captionMirror(page)).text).toBe(before);
    // The editor takes changes again.
    await editWord(page, word.index, "mended");
    await expect(wordButton(page, word.index)).toHaveText("mended");
  });

  // On its own load: the store keeps the cards of the last time the tab was open, and
  // on a shared clip they would show at once, and this time would measure nothing.
  test("14. style cards and Highlight numbers: three images within 400 ms; Highlight numbers off takes the accent off the number; Bold applies", async ({ page }, testInfo) => {
    await openClip(page, referenceClip());
    const ninety = (await transcriptWords(page)).find((word) => word.text === "90");
    await showWord(page, ninety?.index ?? -1);
    await captionWhen(page, (text) => tokens(text).includes("90"), "the caption shows the number");

    // The three images, timed from the tab's click in the page itself.
    await page.evaluate(() => {
      const w = window as unknown as { e2eCardsAt?: number; e2eTabAt?: number };
      new MutationObserver(() => {
        if (w.e2eCardsAt === undefined && document.querySelectorAll('[data-testid="style-cards"] canvas').length === 3) {
          w.e2eCardsAt = performance.now();
        }
      }).observe(document, { childList: true, subtree: true });
      document.addEventListener("pointerdown", () => {
        w.e2eTabAt ??= performance.now();
      }, true);
    });
    await openTab(page, "Style");
    await expect(page.getByTestId("style-cards").locator("canvas")).toHaveCount(3, { timeout: 5_000 });
    const ms = Math.round(
      await page.evaluate(() => {
        const w = window as unknown as { e2eCardsAt?: number; e2eTabAt?: number };
        return (w.e2eCardsAt ?? Number.NaN) - (w.e2eTabAt ?? Number.NaN);
      }),
    );
    testInfo.annotations.push({ type: "reading", description: `style cards: three images ${String(ms)} ms after the tab's click` });
    process.stdout.write(`      style cards: three images ${String(ms)} ms after the tab's click (target ${String(THUMBNAILS_MS)})\n`);
    expect(ms, `the three images came ${String(ms)} ms after the click`).toBeLessThanOrEqual(THUMBNAILS_MS);

    const on = (await captionMirror(page)).colors;
    const highlight = page.getByTestId("highlight-numbers");
    await expect(highlight).toHaveAttribute("aria-checked", "true");
    const since = await saves(page);
    await highlight.click();
    await settled(page, since);
    await expect.poll(async () => (await captionMirror(page)).colors.join(" ")).not.toBe(on.join(" "));
    const off = (await captionMirror(page)).colors;
    const accent = on.filter((color) => !off.includes(color));
    expect(accent.length, "a colour of the caption with numbers highlighted is gone without").toBeGreaterThan(0);

    await setStyle(page, "Bold");
    await captionWhen(page, (text) => text === text.toUpperCase() && /\p{Lu}/u.test(text), "Bold applied");
  });
});
