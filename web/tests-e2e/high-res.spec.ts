// The clip page with `high_res` on (v6.1implementation §11.4, DC-2, D-427 as
// read; project `media-flags`): a 4K H.264 clip is taken, processed and
// exported at 1080 × 1920 with every check of the verifier; HEVC stays
// refused on the clip page, whose render worker decodes H.264 only (open
// item 25); above 3840 pixels is still refused, in the words that say 3840.
//
// `rej_4k.mp4` (1 s of test pattern and a tone) is taken at import but has no
// speech to process, so the processed 4K clip is the reference speech clip
// scaled to 2160 × 3840 here, in a folder of the system's, never committed.

import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { expect, type Page, test } from "@playwright/test";

import { messages } from "../src/copy/messages";
import { installFakeApi } from "./helpers/fake-api";
import {
  dropFile,
  expectAllChecks,
  genFixture,
  NO_GPU,
  patient,
  rejectCodeOf,
  routeAssets,
  sourceDurationMs,
  verifyMp4,
} from "./helpers/fixtures";
import { openClipSignedIn } from "./helpers/long-windows";

const SPEECH = genFixture("speech/speech_60s_portrait.mp4");

/** `/app` with the fake API and the asset host, nothing signed in. */
async function openApp(page: Page): Promise<void> {
  await installFakeApi(page);
  await routeAssets(page);
  await page.goto("/app");
  await expect(page.getByTestId("drop-zone")).toBeVisible({ timeout: 30_000 });
}

test("a 4K file is taken at import (DC-2)", { tag: NO_GPU }, async ({ page }) => {
  await openApp(page);
  await dropFile(page, genFixture("rej_4k.mp4"));
  await expect(page.getByTestId("feed").getByText(messages.feed.cleaningVoice)).toBeVisible({ timeout: 30_000 });
  expect(await rejectCodeOf(page)).toBeNull();
});

test("HEVC stays refused on the clip page (D-427 as read)", { tag: NO_GPU }, async ({ page }) => {
  await openApp(page);
  await dropFile(page, genFixture("rej_hevc.mov"));
  await expect(page.getByTestId("clip-rejected")).toBeVisible({ timeout: 30_000 });
  expect(await rejectCodeOf(page)).toBe("REJECT_HEVC");
});

test("above 3840 pixels is refused, in the words that say 3840", { tag: NO_GPU }, async ({ page }) => {
  await openApp(page);
  await dropFile(page, genFixture("rej_8k.mp4"));
  const panel = page.getByTestId("clip-rejected");
  await expect(panel).toBeVisible({ timeout: 30_000 });
  expect(await rejectCodeOf(page)).toBe("REJECT_RESOLUTION");
  await expect(panel.getByRole("heading")).toHaveText(messages.rejections.REJECT_RESOLUTION.title);
  await expect(panel).toContainText("3840");
});

test("a 4K speech clip is processed and exported at 1080 × 1920, passing all 11 checks", async ({ page }) => {
  test.setTimeout(patient(900_000));
  const scratch = mkdtempSync(path.join(tmpdir(), "offcut-4k-"));
  try {
    const clip = path.join(scratch, "speech_4k.mp4");
    execFileSync("ffmpeg", [
      "-v", "error", "-y", "-i", SPEECH, "-t", "15",
      "-vf", "scale=2160:3840", "-c:v", "libx264", "-preset", "ultrafast", "-crf", "26", "-pix_fmt", "yuv420p",
      "-c:a", "copy", clip,
    ]);
    // Creator exports at 1080 × 1920 (Free at 720 × 1280).
    await openClipSignedIn(page, clip, { picked: true, plan: "creator" });
    const arriving = page.waitForEvent("download", { timeout: patient(480_000) });
    await page.getByRole("button", { name: messages.export.button, exact: true }).click();
    const download = await arriving;
    const file = path.join(scratch, "export.mp4");
    await download.saveAs(file);
    const size = execFileSync("ffprobe", ["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=p=0", file], {
      encoding: "utf8",
    }).trim();
    expect(size).toBe("1080,1920");
    expectAllChecks(verifyMp4(file, "creator", sourceDurationMs(clip), clip));
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
});
