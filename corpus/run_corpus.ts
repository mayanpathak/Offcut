// The corpus runner (v3implementation §17.6, §19.20; D-88, D-112): every
// clip of corpus/manifest.json that `fetch_corpus.sh` put in corpus/.clips/
// is dropped on the app, one at a time, and what became of it is recorded.
// It runs as the Playwright project `corpus`, against `vite preview` of a
// build made with the test key:
//
//   CORPUS_MODE=<mode> pnpm corpus
//
//   demux        (default; TE-12) the outcome at import: accepted, refused, failed
//   export       the same, then a Creator export and the verifier's checks 1-6
//   wer          (E-10, a smoke reading in V3) the recognized words against
//                corpus/.clips/<id>.txt, as counts
//   transcripts  the founder's clips only: the recognized words and prosody
//                into the clip's labeled file, by a fixed map
//
// A test asserts only that its clip ended as the manifest expects, and that
// nothing crashed: rates are recorded, not judged. Clips of people other
// than the founder are private (D-112): a result file holds ids, codes,
// booleans and numbers only, the project keeps no screenshot, video or trace,
// and a failure names the clip's id and a code, never anything the page
// showed.

import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { type BrowserContext, expect, type Page, test } from "@playwright/test";

import { messages } from "../web/src/copy/messages";
import { ERROR_CODES, type ErrorCode, REJECT_REASONS, type RejectReason } from "../web/src/gen/domain";
import { type FakeApi, installFakeApi, mintEntitlementToken, seedEntitlement } from "../web/tests-e2e/helpers/fake-api";
import { dropClip, ensureModelCached, opfsList, patient, readIndexedDb, routeAssets, sourceDurationMs, verifyMp4 } from "../web/tests-e2e/helpers/fixtures";

const ROOT = path.resolve(import.meta.dirname, "..");
const CLIPS = path.join(ROOT, "corpus/.clips");
const RESULTS = path.join(ROOT, "corpus/.results");

type Owner = "founder" | "lent";
type Clip = { id: string; sha256: string; tags: string[]; expect: "accept" | RejectReason; owner: Owner };

const MODES = ["demux", "export", "wer", "transcripts"] as const;
type Mode = (typeof MODES)[number];
const MODE: Mode = MODES.find((mode) => mode === (process.env.CORPUS_MODE ?? "demux")) ?? fail("CORPUS_MODE is demux, export, wer or transcripts");

/**
 * `transcripts` mode's map (§17.6, D-99): the labeled file a founder's clip
 * gives its data to. `founder-01` is in it and is never written: `a01` stays
 * in the written form. Any other clip is refused before anything is read.
 */
const TRANSCRIPT_FILES: Readonly<Record<string, string | null>> = {
  "founder-01": null,
  "founder-02": "fixtures/labeled/a02-v3testclip.json",
};

/**
 * What became of a clip: accepted, the code it was refused with, or the code
 * it failed with. "failed" is a failure whose code the page does not show:
 * only the model's three failures have words of their own (D-57), and no
 * `client_error` is sent for the others before V4.
 */
type Outcome = "accept" | RejectReason | ErrorCode | "failed";

const CRASHES: readonly Outcome[] = ["E_INTERNAL", "E_WORKER_CRASH"];
const MODEL_FAILURES = ["E_MODEL_DOWNLOAD", "E_MODEL_HASH", "E_MODEL_STORAGE"] as const;
const READY_TIMEOUT_MS = patient(360_000);
const EXPORT_TIMEOUT_MS = patient(240_000);
/** Analytics sends what it has every 10 s. */
const FLUSH_WAIT_MS = 20_000;

function fail(message: string): never {
  throw new Error(message);
}

// The manifest is committed and the clips' names are their ids, so both may
// be read while this file loads; a clip's bytes are read only by its test.
const MANIFEST = JSON.parse(readFileSync(path.join(ROOT, "corpus/manifest.json"), "utf8")) as { schema: number; clips: Clip[] };
const PRESENT = MANIFEST.clips.filter((clip) => existsSync(path.join(CLIPS, `${clip.id}.mp4`)));

function today(): string {
  const now = new Date();
  return `${String(now.getFullYear())}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
}

/** A line of a result file: ids, codes, booleans and numbers only (D-112). */
type OutcomeEntry = { id: string; outcome: Outcome; expected: Clip["expect"]; match: boolean; verifier?: boolean[] };
type WerEntry = { id: string; words_ref: number; words_hyp: number; substitutions: number; deletions: number; insertions: number; wer: number };

/** Puts `entry` in today's result file of `kind`, in place of an earlier entry of the same clip. */
function record(kind: "demux" | "export" | "wer", entry: OutcomeEntry | WerEntry): void {
  const file = path.join(RESULTS, `${today()}-${kind}.json`);
  mkdirSync(RESULTS, { recursive: true });
  const before = existsSync(file) ? (JSON.parse(readFileSync(file, "utf8")) as { id: string }[]) : [];
  const after = [...before.filter((other) => other.id !== entry.id), entry].sort((a, b) => a.id.localeCompare(b.id));
  writeFileSync(file, `${JSON.stringify(after, null, 2)}\n`);
}

/** Runs `step`; a failure of it is reported as the clip's id and `what`, and nothing of the page. */
async function quietly<T>(id: string, what: string, step: () => Promise<T>): Promise<T> {
  try {
    return await step();
  } catch {
    throw new Error(`${id}: ${what}`);
  }
}

/**
 * Waits until the clip is refused, has failed, is ready, or (with `until`
 * "accepted") has been accepted: "Cleaning voice" is put in the feed once
 * the import has accepted the clip (Prompt 84's entry). The page gives back
 * a state and a code, never its text.
 */
async function settle(page: Page, id: string, until: "accepted" | "ready", pageErrors: () => number): Promise<Outcome> {
  const models = MODEL_FAILURES.map((code) => [code, messages.errors[code].title] as const);
  const state = await quietly(id, "no outcome in time", async () => {
    const handle = await page.waitForFunction(
      ({ until, cleaning, models }) => {
        const rejected = document.querySelector('[data-testid="clip-rejected"]');
        if (rejected !== null) {
          return `rejected:${rejected.getAttribute("data-reject") ?? ""}`;
        }
        const failed = document.querySelector('[data-testid="clip-failed"]');
        if (failed !== null) {
          const shown = failed.textContent ?? "";
          return `failed:${models.find(([, title]) => shown.includes(title))?.[0] ?? ""}`;
        }
        if (document.querySelector('[data-testid="preview-canvas"]') !== null) {
          return "ready";
        }
        const feed = [...document.querySelectorAll('[data-testid="feed"] li')].map((line) => line.textContent);
        return until === "accepted" && feed.includes(cleaning) ? "accepted" : false;
      },
      { until, cleaning: messages.feed.cleaningVoice, models },
      { timeout: READY_TIMEOUT_MS, polling: 250 },
    );
    return String(await handle.jsonValue());
  });
  if (state === "accepted" || state === "ready") {
    return "accept";
  }
  const [kind, code = ""] = state.split(":");
  const known = kind === "rejected" ? REJECT_REASONS.find((reason) => reason === code) : ERROR_CODES.find((error) => error === code);
  if (known !== undefined) {
    return known;
  }
  // A fault the app did not expect fails the clip as E_INTERNAL and is thrown on into the page.
  return kind === "failed" && pageErrors() > 0 ? "E_INTERNAL" : "failed";
}

/** Exports with a Creator token and runs the verifier: the six checks' results, or the export's error code. */
async function exportAndVerify(page: Page, api: FakeApi, clip: Clip, file: string): Promise<{ outcome: Outcome; verifier?: boolean[] }> {
  // The preview is attached before the export locks it (V3 changelog, known issue 4).
  await quietly(clip.id, "the preview did not start", () =>
    expect(page.getByRole("button", { name: messages.preview.play, exact: true })).toBeEnabled({ timeout: 30_000 }),
  );
  await seedEntitlement(page, mintEntitlementToken());
  const arriving = page.waitForEvent("download", { timeout: EXPORT_TIMEOUT_MS });
  const failing = page.waitForSelector('[data-testid="export-failed"]', { timeout: EXPORT_TIMEOUT_MS });
  // One of the two ends the wait; the other is let go when the context closes.
  void arriving.catch(() => undefined);
  void failing.catch(() => undefined);
  await page.getByRole("button", { name: messages.export.button, exact: true }).click();
  const download = await quietly(clip.id, "the export did not end in time", () => Promise.race([arriving, failing.then(() => null)]));
  if (download === null) {
    await quietly(clip.id, "the export failed without its event", () =>
      expect.poll(() => api.eventsOf("export_failed").length, { timeout: FLUSH_WAIT_MS }).toBeGreaterThan(0),
    );
    const props = (api.eventsOf("export_failed")[0] as { props?: { error_code?: unknown } }).props;
    return { outcome: ERROR_CODES.find((code) => code === props?.error_code) ?? "failed" };
  }
  const scratch = mkdtempSync(path.join(tmpdir(), "offcut-corpus-"));
  try {
    const exported = path.join(scratch, "export.mp4");
    await download.saveAs(exported);
    const passed = new Set([...verifyMp4(exported, "creator", sourceDurationMs(file)).output.matchAll(/^PASS (\d+)\s*$/gm)].map((line) => Number(line[1])));
    return { outcome: "accept", verifier: [1, 2, 3, 4, 5, 6].map((check) => passed.has(check)) };
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

/** §17.6's normalization: lower-case; commas inside digit groups removed; every character but letters, digits, `$`, `%` a space; spaces collapsed. */
function normalized(text: string): string[] {
  return text
    .toLowerCase()
    .replace(/(?<=\d),(?=\d)/g, "")
    .replace(/[^\p{L}\p{N}$%]/gu, " ")
    .split(" ")
    .filter((word) => word !== "");
}

type Errors = { cost: number; substitutions: number; deletions: number; insertions: number };

/** The fewest word edits that turn `ref` into `hyp` (Levenshtein over words), split by kind. */
function wordErrors(ref: readonly string[], hyp: readonly string[]): Errors {
  const cell = (row: readonly Errors[], at: number): Errors => row[at] ?? fail("wordErrors: outside the table");
  let previous: Errors[] = Array.from({ length: hyp.length + 1 }, (_, j) => ({ cost: j, substitutions: 0, deletions: 0, insertions: j }));
  for (const [r, refWord] of ref.entries()) {
    const current: Errors[] = [{ cost: r + 1, substitutions: 0, deletions: r + 1, insertions: 0 }];
    for (const [c, hypWord] of hyp.entries()) {
      const diagonal = cell(previous, c);
      const up = cell(previous, c + 1);
      const left = cell(current, c);
      const same = refWord === hypWord;
      const options: Errors[] = [
        { ...diagonal, cost: diagonal.cost + (same ? 0 : 1), substitutions: diagonal.substitutions + (same ? 0 : 1) },
        { ...up, cost: up.cost + 1, deletions: up.deletions + 1 },
        { ...left, cost: left.cost + 1, insertions: left.insertions + 1 },
      ];
      current.push(options.reduce((best, option) => (option.cost < best.cost ? option : best)));
    }
    previous = current;
  }
  return cell(previous, hyp.length);
}

type StoredTranscript = {
  value: {
    transcript: { words: { text: string; start_ms: number; end_ms: number; confidence: number }[] };
    prosody: { per_word: { energy_z: number; pitch_z: number }[] };
  };
};

/** The clip's transcript record, read from IndexedDB in the page: the one clip of this context. */
async function storedTranscript(page: Page, id: string): Promise<StoredTranscript["value"]> {
  const [clipId] = await opfsList(page, "clips");
  const stored = clipId === undefined ? undefined : ((await readIndexedDb(page, "transcripts", clipId)) as StoredTranscript | undefined);
  return stored?.value ?? fail(`${id}: no transcript was stored`);
}

/** A value in the layout of the labeled files: one line for a small value, one line per element for a long list. */
function inline(value: unknown): string {
  if (Array.isArray(value)) {
    return `[${value.map(inline).join(", ")}]`;
  }
  if (typeof value === "object" && value !== null) {
    return `{ ${Object.entries(value).map(([key, inner]) => `${JSON.stringify(key)}: ${inline(inner)}`).join(", ")} }`;
  }
  return JSON.stringify(value);
}

function labeledText(file: Record<string, unknown>): string {
  const rows = (items: unknown[], indent: string) => (items.length === 0 ? "[]" : `[\n${items.map((item) => `${indent}  ${inline(item)}`).join(",\n")}\n${indent}]`);
  const field = ([key, value]: [string, unknown]): string => {
    if (Array.isArray(value) && value.some((item) => typeof item === "object" && item !== null)) {
      return `  ${JSON.stringify(key)}: ${rows(value, "  ")}`;
    }
    if (key === "prosody" && typeof value === "object" && value !== null && "per_word" in value && Array.isArray(value.per_word)) {
      return `  "prosody": {\n    "per_word": ${rows(value.per_word, "    ")}\n  }`;
    }
    return `  ${JSON.stringify(key)}: ${inline(value)}`;
  };
  return `{\n${Object.entries(file).map(field).join(",\n")}\n}\n`;
}

/** Writes the clip's recognized words and prosody into its labeled file: data, not labels (D-87). */
function writeLabeled(target: string, data: StoredTranscript["value"]): number {
  const file = path.join(ROOT, target);
  const labeled = JSON.parse(readFileSync(file, "utf8")) as Record<string, unknown>;
  // An asr file keeps `sentences` null: the gate reads its words (fixtures/labeled/SCHEMA.md).
  labeled.words = data.transcript.words.map((word) => ({ text: word.text, startMs: word.start_ms, endMs: word.end_ms, confidence: word.confidence }));
  labeled.prosody = { per_word: data.prosody.per_word.map((word) => ({ energy_z: word.energy_z, pitch_z: word.pitch_z })) };
  writeFileSync(file, labeledText(labeled));
  return data.transcript.words.length;
}

// A failed test's `error-context.md` holds a snapshot of the text of the page
// left open (Playwright 1.63), which would be the feed of a lent clip. This
// switch, read by this worker when a context closes, leaves it out; and a
// test that ran out of time has its pages closed before its context, so
// there is no page to take one of.
process.env.PLAYWRIGHT_NO_COPY_PROMPT = "1";
let open: BrowserContext | null = null;
test.afterEach(async () => {
  for (const page of open?.pages() ?? []) {
    await page.close();
  }
  await open?.close();
  open = null;
});

for (const clip of PRESENT) {
  test(`corpus ${MODE}: ${clip.id}`, async ({ browser }, testInfo) => {
    let target: string | null = null;
    if (MODE === "transcripts") {
      // Refused before anything of the clip is read (D-112).
      if (clip.owner !== "founder" || !Object.hasOwn(TRANSCRIPT_FILES, clip.id)) {
        throw new Error(`${clip.id}: refused, transcripts mode takes the founder's clips of its map only`);
      }
      target = TRANSCRIPT_FILES[clip.id] ?? null;
      test.skip(target === null, `${clip.id}: never written, a01 stays in the written form`);
    }
    const reference = path.join(CLIPS, `${clip.id}.txt`);
    test.skip(MODE === "wer" && !existsSync(reference), `${clip.id}: no reference text`);

    const file = path.join(CLIPS, `${clip.id}.mp4`);
    const { baseURL } = testInfo.project.use;
    const context = await browser.newContext({ acceptDownloads: MODE === "export", ...(baseURL === undefined ? {} : { baseURL }) });
    open = context;
    try {
      // A clip that is to be processed finds the model on the device, as after a download. In
      // `demux` mode only a clip refused after the recognizer heard it (REJECT_NO_SPEECH) is.
      const throughPipeline = MODE !== "demux" || clip.expect === "REJECT_NO_SPEECH";
      if (throughPipeline) {
        await ensureModelCached(context);
      }
      const page = await context.newPage();
      let pageErrors = 0;
      page.on("pageerror", () => {
        pageErrors += 1;
      });
      const api = await installFakeApi(page);
      await routeAssets(page);
      await page.goto("/app");
      await quietly(clip.id, "the app did not open", () => page.getByTestId("drop-zone").waitFor({ timeout: 30_000 }));
      await dropClip(page, file);

      // Such a clip is followed to its end; in `demux` mode any other stops at its acceptance.
      let outcome = await settle(page, clip.id, throughPipeline ? "ready" : "accepted", () => pageErrors);
      if (MODE === "demux") {
        record("demux", { id: clip.id, outcome, expected: clip.expect, match: outcome === clip.expect });
      }
      if (MODE === "export") {
        const exported = outcome === "accept" ? await exportAndVerify(page, api, clip, file) : { outcome };
        outcome = exported.outcome;
        const verifier = exported.verifier === undefined ? {} : { verifier: exported.verifier };
        record("export", { id: clip.id, outcome, expected: clip.expect, match: outcome === clip.expect, ...verifier });
      }
      if (MODE === "wer" && outcome === "accept") {
        const ref = normalized(readFileSync(reference, "utf8"));
        const hyp = normalized((await storedTranscript(page, clip.id)).transcript.words.map((word) => word.text).join(" "));
        const errors = wordErrors(ref, hyp);
        const wer = (errors.substitutions + errors.deletions + errors.insertions) / Math.max(1, ref.length);
        record("wer", {
          id: clip.id,
          words_ref: ref.length,
          words_hyp: hyp.length,
          substitutions: errors.substitutions,
          deletions: errors.deletions,
          insertions: errors.insertions,
          wer: Math.round(wer * 10_000) / 10_000,
        });
      }
      if (MODE === "transcripts" && target !== null && outcome === "accept") {
        const words = writeLabeled(target, await storedTranscript(page, clip.id));
        process.stdout.write(`      ${clip.id}: ${String(words)} words into ${target}\n`);
      }

      if (CRASHES.includes(outcome) || outcome !== clip.expect) {
        throw new Error(`${clip.id}: ${outcome}`);
      }
    } finally {
      await context.close();
      open = null;
    }
  });
}
