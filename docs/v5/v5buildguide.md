# v5buildguide.md - Offcut V5: step-by-step build guide

## What this is

`docs/v5/v5implementation.md` says what each V5 file contains and why. This guide says what to do next, which command to run, and how to know it worked. V5 is built on the branch `v5-build`, from `origin/main` at `0a8aa80` (the merge of V4's pull request 7) or later, with a draft pull request into `main` so that CI runs. V5 ends with that pull request merged, `ci.yml` deploying it, every box of §24 ticked, and the tag `v5` pushed.

V5 is the version in which an outside tool judges the export. At the end, a file Offcut exports passes all 11 checks of `verify_mp4.py`, which shares no code with the product: exactly as long as the map says, in sync at both ends, at the promised loudness and bitrate, with the watermark only where the verified token asks for it, with no metadata from the source. The same edit exported twice comes from the render cache at once. The person sees what was changed and is asked one question.

V5 is also the version that must not repeat V4's test time: 22 full media runs, about 8 hours. So this guide runs less than V4's did, on purpose. Each milestone names exactly what it runs (the tiers of Step 0.6), and the full media suite runs once, in Phase 8 (D-197). The one human gate is small: the founder looks at one capture and listens to one export in Phase 8 (D-235).

## How to read the references

| Notation | Means |
|---|---|
| `§n` | Section `n` of `docs/v5/v5implementation.md` |
| `TS §n` | Section `n` of `docs/technicalspec.md` (spec v1.2; §40.9 holds the seams S-8 and S-9) |
| `BP §n`, `PS §n` | `docs/buildplan.md`, `docs/product.md` |
| `V4 §n`, `V4 CL` | `docs/v4/v4implementation.md`; `docs/v4/v4changelog.md` |
| `V5 CL` | `docs/v5/v5changelog.md` |
| `D-n` | A decision: D-1 to D-193 are V1-V4's, D-194 to D-239 are V5's (§2) |
| `P<nn>` | A prompt of `docs/v5/coding-promptsv5.md` (129 to 136) |
| `S-8`, `S-9` | The seams of TS §40.9 V5 builds |
| Tier 0-4 | The test tiers of Step 0.6 (§6.1) |
| `INV-n`, `P-n`, `E-n`, `J<n>`, `C-<n>` | Invariants (TS §35), privacy promises (TS §25.1), experiments (TS §37), journeys and use-case steps (PS §10, TS §9) |
| D1 | The founder's Windows machine; every V5 reading is taken there and labelled `d1` (D-202) |
| "keyed build" | A web build made with `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` (D-72), `pnpm build:e2e` (`--mode test`) |
| "the frozen check" | The command of Step 0.5 |

## How the guide is organised

Nine phases, numbered 0 to 8. Each has a goal, numbered steps, a "Prove the guards" step where a guard is new, and ends with a milestone: commands with their expected output, a `Pass:` line and a commit message. **Do not start a phase while the previous milestone fails.** Each phase from 1 to 8 is one prompt (Phase 1 is P129, Phase 8 is P136) and one commit, titled as its milestone says (`V5 P<nn>: …`; D-194).

**Pushes differ from V4 (D-198).** There are three before the merge, at the end of Phases 1, 6 and 8, and the first two are not waited on: the next phase starts at once, and reads their result in its first step. A red run is fixed before going further. Only the last push is waited on.

## Phase overview

| Phase | Plan phase | Prompt | Covers | Agent time (estimate) | Milestone in one line |
|---|---|---|---|---|---|
| 0 Setup | - | Setup | §1A, §3 | 30 min | `v4` read on `origin`; V4's closing entry; `v5-build` with the V5 docs; baseline counts; draft pull request open |
| 1 The speed work | A | 129 | §6, §19.1, D-196 to D-201, D-236 | 2-2.5 h | `review-edit.spec.ts` green on 4 clip loads; docs-only CI skip; `BENCH_RUNS`; push 1 |
| 2 The verifier and the stereo fix | B | 130 | §7, §8, D-203 to D-210 | 2 h | 11 `PASS` lines on real Creator and Free exports; self-test proves each failure; -14 ± 1 LUFS read by `ffmpeg` |
| 3 The muxer | B | 131 | §9, D-211, D-212 | 1 h | `ctts` and video `elst` on real B-frames; overflow tested; no reordering stays byte-identical |
| 4 Program time and the ladder | B | 132 | §10, §16, D-213 to D-219, D-238 | 2 h | The loop reads `source_at` and `prog48.f32`; a failed encoder restarts; `export-loop.test.ts` green |
| 5 The proof | C | 133 | TS §27.1, §17.5, D-210 | 1.5 h | `timeline-preserved.spec.ts` green on both pause fixtures with all 11 checks |
| 6 The cache and the gate | C | 134 | §11, §12, §13, §16, D-220 to D-231 | 2 h | Same edit twice is instant; §11.3's table each a test; `B_STORAGE_LOW`; three unit-test files; push 2 |
| 7 What the person sees | D | 135 | §14, §15, D-232 to D-237 | 1.5 h | `ChangeSummary`, the question, real `export_done`; axe clean; no V5 marker left |
| 8 Measure, final gate, merge, deploy, tag | D | 136 | §20, §24, §25 | 2.5 h + 30-45 min of the founder's | Bench and one full media run recorded; specs copied back; merged, deployed, `v5` pushed |

**Total about 15-16 hours of agent time over 2 to 3 working days, plus about an hour of the founder's.** All time figures are estimates (§5).

## The problems V5 must fix, and where each is fixed

Six problems found by reading the code at `0a8aa80`, and the open items of `v5changelog.md`. A fix counts as done only when the proof in the last column has run.

| # | Problem | What would go wrong | Fix | Step | Proof |
|---|---|---|---|---|---|
| 1 | **Exports too loud** (D-204, §8) | The chain measures a mono programme and the export doubles it to stereo, so a file should read about -11 LUFS instead of -14. Check 7 would fail on every export. That -11 is inferred from a fixture's PCM (-23.2 as mono, -20.2 as stereo) | `Channel::DualMono` in `loudness.rs`. The product changes, not the check. The preview gets about 3 dB quieter and matches the export | 2.1, 2.2 | Step 2.1 reads a real export **before** the fix; check 7 on real exports after; `the_output_as_a_stereo_file_reads_the_target`; the founder listens in Step 8.7 |
| 2 | **Cache trusts an unverified plan** (D-220, INV-9) | With the page's unverified `planOf` in the key, an expired Creator token, or a forged one, could re-download a stored 1080p file with no watermark | The key's profile kind comes from `exportProfile`, verified in the worker. `planOf` never reaches the key | 6.2, 6.5 | The `Verified profile` Vitest group; the `forged` and `expired` cases of Step 6.7; guard drill of Step 6.8 |
| 3 | **`RENDERER_VERSION` does not exist** (D-223) | The spec relies on it but it is nowhere in the code, so the cache could keep serving an old look after a deploy | `vite.config.ts` computes it from the renderer's files | 6.1 | The key changes with the version (`render-cache` group); `grep` of `web/dist` in Step 6.9 |
| 4 | **Wrong `export_done` count** (D-237) | `kept` reads `event.enabled` alone, so a switched-off visual still counts as kept | `editValues` reads `edit.event_overrides[id] ?? event.enabled` | 7.3 | The `Real values` Vitest group; the `An edit leaves the timeline alone` case |
| 5 | **No fixture with B-frames** (D-212) | Every fixture is encoded with `-bf 0`, so `ctts` could only be tested on random bytes, never real reordered frames | `ok_bframes.mp4`, read by `mux_reorder.rs` | 3.3, 3.4 | `real_b_frames_come_back_at_30_fps`, through `ffprobe` and `ffmpeg` |
| 6 | **Re-dropping the same file** (D-228) | It would hit the old cached export, and skew the bench, whose runs re-drop one clip | `importClip`'s step 4 also clears `renderCache` | 6.6 | The `Import` Vitest case; the bench's `render_encode` readings in Step 8.2 |
| 7 | **The `v4` tag is not on `origin`** (§1A box 1; V5 CL open item 1) | V5's first precondition reads it. On 2026-10-31 `git ls-remote --tags origin` listed `v2` and `v3` only | The founder pushes it | 0.1 | `git ls-remote --tags origin v4` prints a line |
| 8 | **V4's closing entry and 35 open boxes** (V5 CL open item 2) | V4's record would never close | The agent writes the entry and ticks only what the founder vouches for | 0.2 | `v4changelog.md` Entry 40; a box nobody vouches for stays unticked and is named |
| 9 | **No CI without a pull request** (§1A box 4; V5 CL open item 3) | `ci.yml` runs on `pull_request` and `main` only | The founder opens a draft pull request | 0.4 | `gh pr view` shows it |
| 10 | **Watermark and sync thresholds are guesses** (§7, TS §39.3 item 14) | A check that fails good files, or passes bad ones | Read them on real exports, offline, with the rule of §7.3 | 2.6 | `INFO` lines recorded in the entry and TS §39.3 |

## Read this first: 10 things that will bite you

1. **The stereo fix breaks a test by design.** `a_1khz_sine_of_peak_minus_20_reads_minus_23` in `loudness.rs` asserts today's reading; it is rewritten by name to `..._reads_minus_20`. `pink_noise_reaches_the_target` asserts the gain is capped, and may no longer be: if that assertion fires, lower its input from -40 to -44 dBFS RMS and correct its comment. Any other failure in `offcut-dsp` is read before anything is edited (§8, Step 2.2).

2. **The recognizer does not hear the fix, so transcripts do not move.** It takes `pcm16` from the media worker, not the chain. The comment in `bench/device-bench.ts` ("V3 cleans the voice before the recognizer hears it") is out of date, so do not go looking for a transcript regression there. Prosody is z-scored per clip. If a transcript-dependent case fails after Phase 2, the cause is elsewhere.

3. **A shared clip leaks analytics between cases.** Analytics flushes every 10 s, so the undo steps of one reset reach the next case, whose `toEqual(["word_edit"])` then fails. `resetClip` must send what is queued before it forgets. `flushAnalytics` needs `page.clock`, which this page does not have, so reset overrides `visibilityState` and dispatches `visibilitychange` (§6.3, Step 1.1).

4. **`verifyMp4` runs `python`, and plain `python` on D1 is 3.9.13 with no NumPy.** From Phase 2 every terminal puts Python 3.13 first on `PATH` (the Quick reference line). D-203 makes the verifier exit 2 when it cannot import NumPy, so the mistake is loud.

5. **Serial groups and `--grep`.** `export-creator.spec.ts` is a serial group: a failed case stops the ones after it, and `--grep` runs only what it names. A case written to read the previous case's stored file must name that case in its grep too (Step 6.7).

6. **The render worker takes one job at a time, so the preview locks before the gate.** A playing preview would refuse `exportProfile` with `Busy`. `lockForExport` comes first, and a hit unlocks at once (D-221). The call goes through `whenFree`, as every other call that can meet the preview does.

7. **One cache entry per clip is why "undo everything, then export" is a miss.** A miss deletes the row and the file (D-225). Undoing back to an earlier export finds nothing until V7. Do not "fix" this: the test expects a miss.

8. **Goldens and snapshots do not change in V5.** V5 changes neither the scene nor the renderer. A changed `.png` or `.snap` in the diff is an error to explain, never to accept. The frozen check of Step 0.5 catches it.

9. **`B_STORAGE_LOW` is on `COPY_PENDING` today.** `check-copy-codes.mjs` fails when a code has copy and is still listed, so the copy and the removal go in one commit (Step 6.4).

10. **Commit and push only on `v5-build`, and never wait where this guide says not to.** `git branch --show-current` before every commit. Merging, pushing to `main`, marking the pull request ready and tagging are the founder's (D-194). A push to `main` deploys.

---

## Phase 0 - Setup (§1A, §3)

Goal: V4 is closed on the record, `v5-build` exists with the V5 documents, the baseline counts are recorded, a draft pull request gives CI, and D1 can run the verifier.

### Step 0.1 - Read the tag (§1A box 1)

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut
git fetch origin --tags --prune
git ls-remote --tags origin v4                               # expect: one line with refs/tags/v4
git rev-parse --short origin/main                            # expect: 0a8aa80 or later
```

If `ls-remote` prints nothing, **stop**: ask the founder to run this, and go no further.

```bash
git tag -a v4 -m "V4: the look and the editor" 0a8aa80 && git push origin v4
```

The tag is the founder's (D-194). The agent never pushes it.

### Step 0.2 - V4's closing entry (V5 CL open item 2)

The tree is on `v4-build`, with the V5 documents uncommitted. Carry them across in Step 0.3; the entry and the boxes go in the same first commit, since V4 is merged.

| Order | File | Watch for |
|---|---|---|
| 1 | `docs/v4/v4changelog.md` | Entry 40: "V4 closed", the merge `0a8aa80`, the date of the tag, the founder's words of 2026-10-31, the final counts of Entry 39 |
| 2 | `docs/v4/v4implementation.md` | Ask the founder which of the 35 boxes they vouch for (design review 2, the Neon rows, the deployed walk-through are unrecorded). Tick those. Name in Entry 40 every box left unticked |
| 3 | `docs/v5/v5changelog.md` | Open items 1 and 2 closed |

### Step 0.3 - The branch (§1A box 3, D-194)

```bash
git status --short                                           # expect: docs/technicalspec.md and docs/v5/ only
git switch -c v5-build origin/main                           # the docs come along, uncommitted
git branch --show-current                                    # expect: v5-build
git merge-base v5-build origin/main | cut -c1-7              # expect: 0a8aa80 or later
```

`git switch` refuses if a file would be overwritten; `origin/main`'s tree equals `v4-build`'s at `ccc258f` for the code, so it should not. If it does, stop and read the message.

### Step 0.4 - The baseline, the push, the pull request (§1A boxes 4, 5)

```bash
set -a; . ./.env; set +a
pnpm install --frozen-lockfile
pnpm check                                                   # expect: exit 0
pnpm fixtures                                                # expect: up to date, or a fresh build
cargo test --workspace 2>&1 | awk '/^test result:/ {p+=$4; f+=$6; i+=$8} END {print p" passed, "f" failed, "i" ignored"}'
# expect: 646 passed, 0 failed, 4 ignored (start Docker Desktop first: the server's tests use Postgres on port 9000)
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"   # expect: Tests  175 passed (175)
pnpm e2e 2>&1 | tail -2                                      # expect: 12 passed
```

V4's media figure (77 passed + 1 skip) is carried over, not re-run: the code is the merge's. Write the numbers into Entry 1 of `v5changelog.md` as a table (Item, Value): the baseline SHA, the four counts, the app-shell size from `node scripts/check-hosts.mjs` after `pnpm build`.

```bash
git add docs && git commit -m "V5 setup: branch, baseline, V4's closing entry"
git push -u origin v5-build
```

**Founder:** open a draft pull request `v5-build` → `main`. Record its number everywhere this guide writes `<pr>`. A docs-only first push runs the whole of CI until Phase 1 adds the skip; it is not waited on.

### Step 0.5 - Define the frozen check

Run at every milestone from now on. It lists the files §4 says V5 must not touch.

```bash
BASE=0a8aa80
git diff --name-only "$BASE" -- \
  server crates/offcut-types crates/offcut-api-types crates/offcut-plan crates/offcut-scene crates/offcut-render \
  crates/offcut-entitlement crates/offcut-text crates/offcut-detect crates/offcut-wasm-core crates/offcut-wasm-render \
  crates/offcut-dsp/src/chain.rs crates/offcut-dsp/src/config.rs crates/offcut-dsp/src/program.rs crates/offcut-dsp/src/prosody.rs \
  crates/offcut-dsp/tests/chain_no_clicks.rs crates/offcut-dsp/tests/duration_preserved.rs \
  crates/offcut-dsp/tests/program_identity.rs crates/offcut-dsp/tests/prosody.rs \
  web/vercel.json web/src/gen web/src/net web/src/models web/src/platform web/src/analytics \
  web/src/workers/audio.worker.ts web/src/workers/media.worker.ts web/src/workers/asr.worker.ts web/src/workers/media web/src/workers/asr \
  web/src/workers/rpc.ts web/src/workers/pool.ts \
  web/src/workers/render/preview-job.ts web/src/workers/render/preview-loop.ts web/src/workers/render/video-source.ts web/src/workers/render/frame-decoder.ts \
  web/src/usecases/control-preview.ts web/src/usecases/review-update.ts \
  web/tests-e2e/preview-sync.spec.ts fixtures/golden fixtures/labeled fixtures/speech corpus render.yaml
# expect: no output
```

`crates/offcut-dsp/src/loudness.rs` and `tests/loudness_target.rs` are deliberately not on the list (Phase 2).

And the positive view: `git diff --name-only "$BASE" | sort` lists only files of §4, the docs, and from Phase 8 `bench/results/`. It never lists `.env*`, `testclips/`, `fixtures/.gen/`, `fixtures/.cache/`, `corpus/.clips/`, `corpus/.results/`, `web/test-results/` or `web/playwright-report/`.

### Step 0.6 - The test tiers (§6.1, D-196, D-197)

| Tier | When | What runs | Typical time |
|---|---|---|---|
| 0 | While writing | The one test being written: `cargo test -p <crate> <name>`; `pnpm --filter web exec vitest run <file>`; `python verify/test_verify_mp4.py <Case>` | Seconds |
| 1 | Before each commit | `pnpm check`; `cargo test -p` each crate touched; `pnpm --filter web exec vitest run` | 3-8 min |
| 2 | Before a commit that touches `web/` behaviour or a worker | Tier 1, a build, and only the cases named: `pnpm e2e:media <spec> --grep "<case>" --max-failures=1` | 3-15 min |
| 3 | End of Phases 1 and 6, before a push | `pnpm check && pnpm fixtures && pnpm test`, `pnpm e2e` | About 15 min |
| 4 | Phase 8 only | `pnpm e2e:media` in full, once, on a keyed build; then CI, waited for | 30-40 min + about 30 min |

| Rule | Why |
|---|---|
| A new or changed case runs alone first | A hang is never discovered through a suite |
| Long runs go in the background; read only the tail and failures | A tool call ends at 10 minutes |
| Only `web/src` changed: `pnpm --filter web exec vite build --mode test` (seconds). Rust changed: `pnpm build:e2e` | The WASM rebuild is the slow part |
| `pnpm build` and `pnpm build:e2e` read the key from the environment; export it only in the terminal that builds for media | D-72; no key in a file |
| A prompt that fails a targeted run fixes it there, and does not widen to a full run | D-197 |
| After the full run, a follow-up fix re-runs the affected spec, unless it touches the pipeline, a worker or the renderer | D-197 |
| "Passes twice" is dropped, except for a timing suite whose code changed. V5 changes none | §6.1 |
| `--repeat-each` on one case, never a whole suite | §6.1 |

### Milestone 0

```bash
git branch --show-current                                    # expect: v5-build
git ls-remote --tags origin v4                               # expect: one line
gh pr view <pr> --json state,isDraft,baseRefName,headRefName --jq '.'   # expect: OPEN, true, main, v5-build
grep -c "Entry 1 " docs/v5/v5changelog.md                    # expect: 1
py -3.13 -c "import numpy; print(numpy.__version__)"         # expect: 2.5.3
ffmpeg -hide_banner -version | head -1                       # expect: 9.0.2
```

Pass: the tag is on `origin`, V4's entry is written, `v5-build` is pushed with the V5 documents, the counts equal V4's close, and the draft pull request exists.

Commit: `"V5 setup: branch, baseline, V4's closing entry"`.

Do not start Phase 1 until this passes.

---

## Phase 1 - The speed work (P129; §6, §19.1, D-196 to D-201, D-236)

Goal: the next seven prompts and every later version test faster, with no gate lost. Nothing in the product changes.

### Step 1.1 - The shared-clip helpers (§6.3)

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/tests-e2e/helpers/fake-api.ts` | §6.3 | `FakeApi` gains `forget()`: `requests.length = 0` |
| 2 | `web/tests-e2e/helpers/fixtures.ts` | §6.3 | `sendQueuedAnalytics(page)`: override `document.visibilityState` to `hidden` with `configurable: true`, dispatch `visibilitychange`, delete the override, wait 300 ms. `flushAnalytics` needs `page.clock`, which this page does not install (Read this first, item 3) |
| 3 | `web/tests-e2e/helpers/fixtures.ts` | §6.3 | `openShared(browser, info, fixture)`: a context with `baseURL`, 1280 x 800, `ensureModelCached`, a page, `installFakeApi`, `routeAssets`, `trackSaves`, `/app`, `expectTestBuild`, `dropClip`, canvas visible, Play enabled. Exactly what `openClip` does, on a page of its own |
| 4 | `web/tests-e2e/helpers/fixtures.ts` | §6.3 | `resetClip(page, api)`, the five steps of §6.3, in order. Undo is found by role and `messages.editor.undo`: it has no test id. Clean state asserted last: Undo disabled, no `preview-failed` card, and the stored edit (`readIndexedDb(page, "edits", clipId)`) equal to `clip-store`'s empty one. The Style tab is not opened: it would start a thumbnail job |

### Step 1.2 - `review-edit.spec.ts` on one clip (D-201)

```ts
test.describe("the editor (review-edit), on one processed clip", () => {
  test.describe.configure({ timeout: patient(420_000) });   // beforeAll has the test's timeout
  let shared: { context: BrowserContext; page: Page; api: FakeApi };
  test.beforeAll(async ({ browser }, info) => { shared = await openShared(browser, info, referenceClip()); });
  test.afterAll(async () => { await shared.context.close(); });
  test.beforeEach(async () => { await resetClip(shared.page, shared.api); });
  // cases 1-4, 7, 9-16, with `page` and `api` read from `shared`
});

test.describe("the editor (review-edit), on a clip of its own", () => { /* cases 5, 6, 8, as today */ });
```

| Watch for | Why |
|---|---|
| Not `serial` | A failed case must not stop the others. Playwright starts a new worker after a failure, and `beforeAll` runs again: a new clip |
| Case numbers and titles unchanged; what each asserts unchanged | D-201. A case that cannot share moves to its own load, and the changelog says which |
| Case 12's opening | Read on 2026-10-31: it makes three new changes first, and a new change drops the redo branch. It needs nothing |
| The header comment ("Sixteen cases, each with its own clip load") | Rewrite it, naming the four loads |
| No `{ page }` fixture in a shared case | It would open a second, empty page |

### Step 1.3 - Known issues 33 and 36 (D-236)

`web/tests-e2e/model-download.spec.ts`:

| Where | Change |
|---|---|
| `openEditor`, the `drop-zone` wait | `{ timeout: patient(30_000) }` |
| `modelDownloadEvent`, the poll | `expect.poll(…, { timeout: patient(5_000) })` |

An issue is closed only by what it should show: no first-attempt failure of those cases in V5's CI runs. Keep both open in the changelog until Phase 8 reads the runs.

### Step 1.4 - The docs-only skip (D-199, §19.1)

`.github/workflows/ci.yml`:

| Order | Change | Watch for |
|---|---|---|
| 1 | A new first job, `changes`: `runs-on: ubuntu-latest`; checkout with `fetch-depth: 0`; download gitleaks (`GITLEAKS_VERSION`, `GITLEAKS_SHA256`, moved from "Install the build tools", installed into `$RUNNER_TEMP`); `gitleaks detect --source . --redact --no-banner`; the `diff` step of §19.1; `outputs: code` | Every event scans, documents included |
| 2 | `ci`: `needs: changes`, `if: needs.changes.outputs.code == 'true'` | `e2e-media` already `needs: ci` and skips with it; `deploy-*` run on `push`, where `code` is always true |
| 3 | `ci`: drop the gitleaks lines from "Install the build tools", the `gitleaks version` line and the "Scan for secrets" step; drop `fetch-depth: 0` and its comment from checkout | The section comment "1. Checkout, caches, secret scan" follows |
| 4 | The header comment names the rule | |
| 5 | `.github/workflows/e2e-media.yml`: "all 30 cases" is stale | Comment only; no job change |

`main` has no branch protection (`gh api`, 404, 2026-10-31), so a skipped job blocks nothing.

Drill the `diff` script locally on both kinds of diff, in a scratch copy of the step:

```bash
others=$(git diff --name-only 0a8aa80...HEAD | grep -vE '^docs/|\.md$' || true); [ -n "$others" ] && echo code=true || echo code=false
# expect here: code=true (ci.yml changed)
others=$(printf 'docs/v5/x.md\nverify/README.md\n' | grep -vE '^docs/|\.md$' || true); [ -n "$others" ] && echo code=true || echo code=false
# expect: code=false
```

### Step 1.5 - `BENCH_RUNS` (D-200)

`bench/device-bench.ts`: `RUNS` read from `BENCH_RUNS`, a whole number from 1, default 10; anything else throws before a run starts. The test's name, the header comment ("ten"), and `BENCH_TIMEOUT_MS` as `(RUNS + 1) * 11 * 60_000` follow. Nothing else changes.

```bash
pnpm --filter web exec playwright test --project=bench --list | tail -2   # expect: one test, named for 10 runs
BENCH_RUNS=5 pnpm --filter web exec playwright test --project=bench --list | tail -2   # expect: named for 5 runs
```

### Step 1.6 - Run it (Tier 2)

```bash
export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=$(grep -E '^ +E2E_TEST_PUBLIC_KEY:' .github/workflows/ci.yml | awk '{print $2}')
pnpm build:e2e
pnpm e2e:media review-edit.spec.ts 2>&1 | tail -5            # in the background; expect: 16 passed. Note the time against V4's
pnpm e2e:media review-edit.spec.ts --grep " (1|12)\. " --repeat-each=2 --max-failures=1 2>&1 | tail -3   # the reset works repeatedly
unset VITE_ENTITLEMENT_TEST_PUBLIC_KEY
```

| If | Do |
|---|---|
| A case fails on `resetClip`'s clean-state assertion | Read what stayed: fix the reset, or move that case to its own load (D-201 allows it; the entry names it) |
| A case's own assertion fails | It assumed a clean page in a way the reset does not give. Read it; do not rerun blindly |
| `reviewActions` holds `undo` | `sendQueuedAnalytics` did not run before `forget()`, or the override was not `configurable` |

### Step 1.7 - Prove the guards

| Temporary edit | Rule that must fire |
|---|---|
| Comment out the Undo loop of `resetClip`; run case 3 then case 4 | The clean-state assertion fails by name |
| Comment out `sendQueuedAnalytics`; run case 1 then case 13 (`--grep " (1|13)\. "`) | Case 13's `toEqual(["caption_position"])` fails on case 1's late events |

Undo both.

### Milestone 1

```bash
grep -c "openShared\|openClip(page, " web/tests-e2e/review-edit.spec.ts   # read: 1 shared open plus 3 own loads
# Tier 3
pnpm check && pnpm fixtures && pnpm test                     # expect: exit 0; counts equal Entry 1's
pnpm e2e 2>&1 | tail -2                                      # expect: 12 passed
actionlint .github/workflows/ci.yml 2>/dev/null || true      # if at hand; otherwise the push proves it
# the frozen check                                           # expect: no output
git push                                                     # do not wait (D-198)
```

Pass: all 16 cases pass in one run on 4 clip loads, the `diff` script gives both answers, `BENCH_RUNS` lists both, and the suite's time is in the entry against V4's 41.8-54.9 min for the whole of media.

Commit: `"V5 P129: shared clip in review-edit, gates, docs-only CI skip"`. **Push, first of three. Do not wait.**

Do not start Phase 2 until this passes.

---

## Phase 2 - The verifier and the stereo fix (P130; §7, §8, D-203 to D-210)

Goal: the tool that judges everything exists, and has been shown to tell good from bad on real files. An export reads -14 LUFS as a stereo file.

First: read CI of Phase 1 (`gh pr checks <pr>`). Red: fix it now, with its own commit, before anything else.

### Step 2.1 - Read the problem on a real export, before the fix (problem 1)

The -11 LUFS of §8 is inferred. Read it before changing the product, so the entry has a real "before".

| Order | File | Watch for |
|---|---|---|
| 1 | `helpers/fixtures.ts` | `verifyMp4` copies each file it checks to `OFFCUT_KEEP_EXPORTS` when set. Never set in CI; the directory is outside the repository (§3.3, §6.2) |

```bash
export OFFCUT_KEEP_EXPORTS="$HOME/v5-exports"; mkdir -p "$OFFCUT_KEEP_EXPORTS"
export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=$(grep -E '^ +E2E_TEST_PUBLIC_KEY:' .github/workflows/ci.yml | awk '{print $2}')
pnpm e2e:media export-creator.spec.ts --grep "Creator export" --max-failures=1 2>&1 | tail -3   # on Phase 1's keyed build
ffmpeg -nostdin -i "$OFFCUT_KEEP_EXPORTS/creator.mp4" -map 0:a:0 -af ebur128=peak=none -f null - 2>&1 | grep -E "^ +I:" | tail -1
# expect: about -11 LUFS. Record the number in the entry as "before". If it is near -14, stop: section 8's reasoning is wrong
```

### Step 2.2 - The fix (D-204, §8)

| Order | File | Tests to write with it / Watch for |
|---|---|---|
| 1 | `crates/offcut-dsp/src/loudness.rs` | `Channel::DualMono`; the doc comment of `integrated` says "as a mono signal on two loudspeakers" |
| 2 | Same, inline | New `a_mono_programme_reads_3_lu_louder_as_two_loudspeakers` (3.01 ± 0.05 LU, against a local `Center` meter); `a_1khz_sine_of_peak_minus_20_reads_minus_23` rewritten by name to `..._reads_minus_20`, within 0.1 of -20 (Read this first, item 1) |
| 3 | `crates/offcut-dsp/tests/loudness_target.rs` | New `the_output_as_a_stereo_file_reads_the_target`: each existing fixture through `chain`, measured by an `EbuR128::new(2, …)` with `Left` and `Right`, within ±1 LU of -14 |

```bash
cargo test -p offcut-dsp                                     # expect: all pass, or pink_noise's "not capped" assertion (below)
```

| If this fires | Do |
|---|---|
| `pink_noise_reaches_the_target`: "the gain is not capped" | Input from -40 to -44 dBFS RMS, and its comment. What it proves stays. Name it in the entry |
| Anything else in `offcut-dsp` | Read it before editing anything. `chain_no_clicks.rs`, `duration_preserved.rs`, `program_identity.rs` must pass unchanged (frozen) |

### Step 2.3 - The mask (D-206, §7.3)

```bash
py -3.13 -m pip install pillow                               # once, on D1; not a dependency of the verifier
```

`verify/make_watermark_mask.py`, and a row for it in TS §5. It draws `verify/watermark_mask_720.png`: 720 x 1280, 8-bit grey, background 128, "Made with Offcut" in `crates/offcut-scene/assets/fonts/Inter-Variable.ttf` at weight 700 and 24 px, fill 255, outline 0 at 2 px, left at 40 px, baseline at about 993 px. Its head names where each number comes from (`watermark.rs`, `safe_area.rs`). It reads the font as data and shares no code with the renderer.

**Open the PNG and look at it:** the text must sit where a Free export's watermark does. Compare with a frame of a Free export scaled to 720 x 1280.

### Step 2.4 - Checks 7 to 11 (§7)

`verify/verify_mp4.py`, from 233 lines to about 550. `NOT_IMPLEMENTED` and its print go; `CHECKS` gains five functions; the printing and exit rules do not change. Lines that report without judging start with `INFO`.

| Order | Piece | Section | Watch for |
|---|---|---|---|
| 1 | `import numpy` at the top, exit 2 with `run pip install -r verify/requirements.txt` | D-203 | A check that silently cannot run must not look like a pass |
| 2 | `run_bytes()`, `pcm()`, `grey()` | §7 | Pipes, no temporary files |
| 3 | `check_7` | §7.1 | The stream as it is, stereo; parse the last `I:` line of stderr |
| 4 | `check_8` | §7.2 | 10 ms RMS envelopes; first and last 5,000 ms on the export's timeline; lags ±100 hops; parabola; `abs(offset) <= 20 ms` and peak `>= MIN_ENVELOPE_CORR`; a silent source window is `INFO`, not a failure; flashes over the whole file, same count, each within one frame |
| 5 | `check_9` | §7.3 | 15 frames evenly spaced, skipping flashes, scaled to 720 x 1280; mean; high-pass both by a 9 x 9 box blur; Pearson inside the mask's box. `free` at least 0.5, `creator` at most 0.2 |
| 6 | `check_10` | §7.4 | Packet sizes over the stream's duration; 12,000,000 and 6,000,000 b/s held in the verifier |
| 7 | `check_11` | §7.5 | D-100's rule; `©xyz` and `ISO6709` over 1 MiB pieces with 7 bytes of overlap. **A `FAIL 11` names the tag's key, never its value** |
| 8 | `--source` is read; without it, `INFO 8: no --source, not measured`, and likewise the source half of 11 | §7 | `expectAllChecks` refuses "not measured" |
| 9 | `verify/README.md`, `requirements.txt`'s comment, the docstring, the `V5:` marker at the argument | §7 | The docstring no longer says "standard library only" |

### Step 2.5 - The self-test (D-209, §7.6)

`verify/test_verify_mp4.py`, `unittest`, about 180 lines, one case at a time (Tier 0), and a row in TS §5. It makes a good file from `speech_20s_long_pause.mp4`, then the bad files of §7.6's table. Each case asserts its own `FAIL n` and every other check `PASS`, so a bad file fails for its own reason. It cleans its temporary directory. `pnpm verify:test` is a root script: `"verify:test": "python verify/test_verify_mp4.py"`.

```bash
export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:$PATH"
time python verify/test_verify_mp4.py 2>&1 | tail -3         # expect: OK; under 3 minutes
```

`ci.yml`, after "Generate the fixtures":

```yaml
      - name: Set up Python
        uses: actions/setup-python@5fda3b95a4ea91299a34e894583c3862153e4b97 # v7.0.0
        with:
          python-version: "3.13"

      - name: The verifier's own test
        run: |
          python -m pip install -r verify/requirements.txt
          python verify/test_verify_mp4.py
```

### Step 2.6 - Tune on real files, offline (§6.2, §7.3)

`helpers/fixtures.ts`:

```ts
export function verifyMp4(file: string, profile: "free" | "creator", expectedDurationMs: number, source?: string): Verdict;
/** Every check passed and none was not measured. `flashes`: how many white frames were compared. */
export function expectAllChecks(verdict: Verdict, o?: { flashes?: number }): void;
```

`export-creator.spec.ts`: Creator and Free with `referenceClip()` as source and `expectAllChecks`; `metadataProblems`, `tagsOf` and `SOURCE_TAGS` go (D-208), and the second-recording case reads `PASS 11`. As creator, the Free file now also reads `FAIL 9`, so the comment "its size is the one thing that differs" goes.

```bash
pnpm build:e2e                                               # the chain changed
pnpm e2e:media export-creator.spec.ts --grep "no token|another key|Creator export|Free-plan" --max-failures=1 2>&1 | tail -5
```

One Creator export of the long-pause fixture through a temporary case in `web/tests-e2e/zz-longpause.spec.ts`, deleted before the commit. Then, in seconds and as often as needed:

```bash
python verify/verify_mp4.py "$OFFCUT_KEEP_EXPORTS/creator.mp4" --profile creator --expected-duration-ms <ms> --source <referenceClip>
python verify/verify_mp4.py "$OFFCUT_KEEP_EXPORTS/free.mp4" --profile free --expected-duration-ms <ms> --source <referenceClip>
python verify/verify_mp4.py "$OFFCUT_KEEP_EXPORTS/longpause.mp4" --profile creator --expected-duration-ms 20000 --source fixtures/.gen/speech/speech_20s_long_pause.mp4
```

| Reading | Target | If it misses |
|---|---|---|
| `INFO 7` | -14 ± 1 LUFS | Read §8 again; do not move the check |
| `INFO 8` offsets | Within 20 ms; flashes within a frame | Look before moving anything. `MIN_ENVELOPE_CORR` is an (assumption); the windows are fixed by TS §27.2 |
| `INFO 9` free, creator | ≥ 0.5, ≤ 0.2, at least 0.3 apart | §7.3: widen the window to 31 frames, then read the mask's alignment. A threshold moves only while the two stay 0.3 apart, recorded in TS §39.3, and never to pass one file |
| `INFO 10` | ≤ 12 and ≤ 6 Mb/s | §21: `bitrateMode: "constant"` on that entry in `encoders.ts` |

Record every `INFO` line in the entry. **Then delete the saved files** (`rm -rf "$OFFCUT_KEEP_EXPORTS"; unset OFFCUT_KEEP_EXPORTS`): an exported file is never part of a report (TS §22.5).

### Step 2.7 - Prove the guards

| Temporary edit | Rule that must fire |
|---|---|
| `Channel::Center` back in `loudness.rs` | `the_output_as_a_stereo_file_reads_the_target` fails |
| Rename `numpy` in a local stub on `PYTHONPATH` | The self-test's NumPy case: exit 2 |
| `expectAllChecks` on a verdict with no `--source` | It fails on "not measured" |

Undo all three.

### Milestone 2

```bash
unset VITE_ENTITLEMENT_TEST_PUBLIC_KEY
cargo test -p offcut-dsp                                     # expect: all pass
python verify/test_verify_mp4.py 2>&1 | tail -1              # expect: OK
pnpm check                                                   # expect: exit 0 (check-file-tree finds the two new verify files listed)
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"   # expect: 175 passed, or more
git status --short | grep -c "zz[-_]"                        # expect: 0
ls "$HOME/v5-exports" 2>/dev/null | wc -l                    # expect: 0
# the frozen check                                           # expect: no output
```

Pass: `verify_mp4.py` prints 11 `PASS` lines on the Creator and the Free export; each bad file fails for its own reason; -14 ± 1 is read from a real stereo export and the "before" reading is recorded; the watermark readings are at least 0.3 apart and on the right sides of 0.5 and 0.2.

Commit: `"V5 P130: verifier checks 7-11, its self-test, and the stereo loudness fix"`. No push.

Do not start Phase 3 until this passes.

---

## Phase 3 - The muxer (P131; §9, D-211, D-212)

Goal: a stream with B-frames makes a correct file, and the overflow guard is tested. No browser and no WASM rebuild in this phase: `Writer.video` is unchanged and Phase 4 rebuilds.

### Step 3.1 - The tables (§9.1)

| Order | File | Section | Tests / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-mp4/src/mux_boxes.rs` | §9.1 | `ctts(&[(u32, u32)])`: version 0, run-length; placed after `stts`. `stbl()` takes `ctts: Option<&[(u32, u32)]>` |
| 2 | `crates/offcut-mp4/src/mux.rs` | §9.1 | `presented: Vec<u32>`, `seen: Vec<bool>`, `MAX_REORDER = 16` with its reason. `add_video_sample`'s signature does not change; `frame` means the presentation index |

`add_video_sample`:

| Today | After |
|---|---|
| `frame` must equal the sample number | `frame` is the presentation index |
| First sample must be a keyframe | The same, and it must be frame 0 |
| Anything else: `OutOfOrder` | A frame already seen, or more than `MAX_REORDER` beyond the sample number: `OutOfOrder` |
| `sync` holds sample numbers | The same: sample numbers, in decode order |

`finalize` first checks every index from 0 to the count has arrived; a gap is `OutOfOrder`.

### Step 3.2 - `moov()` (§9.1)

```text
for sample i (0-based, decode order):
   dts_i  = i x 1000                  (ticks of 1/30,000 s; stts stays one run)
   pts_i  = presented[i] x 1000
   delay  = max over i of (dts_i - pts_i), at least 0
   offset_i = pts_i - dts_i + delay   (always >= 0: ctts version 0)
```

When every `offset_i` is 0, nothing changes: no `ctts`, no edit, **the file is what it is today**. Otherwise the runs go to `ctts`, and the video `trak` gets `boxes::edts(video_duration, delay)`, the function audio priming already uses.

No line of the module may use `unwrap`, `expect` or slice indexing (non-test Rust).

### Step 3.3 - The fixture (D-212, §18)

| Order | File | Watch for |
|---|---|---|
| 1 | `fixtures/gen_fixtures.sh` | `enc ok_bframes $GLOBAL $(src 1280 720 30 2) $OUTF -c:v libx264 -bf 2 -g 30 -pix_fmt yuv420p $FAST $AAC $FS "$OUT/ok_bframes.mp4"`. `$H264` holds `-bf 0`, so it is not used here |
| 2 | `fixtures/manifest.json` | A row in `ok_vfr`'s shape: `"expect": { "accept": { "orientation": "landscape", "is_vfr": false } }`, `"probe": { "video_codec": "h264", "width": 1280, "height": 720, "frames": 60 }`, `"tests": ["demux_fixtures.rs", "mux_reorder.rs"]`. Step 8 of the generator and `demux_fixtures.rs` read the manifest, so both start checking it |
| 3 | TS §5 | "25 fixtures" becomes 26 |

```bash
pnpm fixtures && ffprobe -v error -select_streams v:0 -show_entries frame=pict_type -of csv=p=0 fixtures/.gen/ok_bframes.mp4 | sort | uniq -c
# expect: a B row: the fixture does hold B-frames
cargo test -p offcut-mp4 --test demux_fixtures               # expect: all pass, ok_bframes among them
```

### Step 3.4 - The tests (§9.2, §9.3, §17.2)

| Order | File | Case | Asserts |
|---|---|---|---|
| 1 | `tests/mux_roundtrip.rs` | The comment "no `ctts` before V5" | "…and no `ctts` when nothing is reordered". Its assertion stays |
| 2 | Same | `frames_written_in_decode_order_are_shown_in_presentation_order` | `ctts` runs, `elst` media time, demuxer's times, keyframes by sample number |
| 3 | Same | `no_reordering_writes_no_ctts_and_no_video_edit` | The six tables; byte equality with a run of today's order |
| 4 | Same | `a_gap_in_the_frames_is_out_of_order_at_finalize`, `a_frame_far_ahead_is_out_of_order_at_once` | `OutOfOrder`; `MAX_REORDER` |
| 5 | Same | `ninety_seconds_reordered_fit_the_space_kept_for_moov` | Fits |
| 6 | Same | `a_moov_over_the_reserve_is_refused` | 30,000 one-byte frames written in turns with audio, so each is its own chunk: `MoovOverflow`, nothing written over `mdat` |
| 7 | Same | `a_reserve_with_under_8_bytes_left_is_refused` | The `(1..8)` branch, sizes found by search in the test |
| 8 | Same | The proptest | Any permutation within the window round-trips |
| 9 | `tests/mux_reorder.rs` (new, TS §5 row) | `real_b_frames_come_back_at_30_fps` | `ok_bframes.mp4` through `Demuxer` and `Mp4Muxer` in decode order with presentation indexes and its `avcC`; the demuxer reads the same times; `ffprobe` deltas all 1/30 s; `ffmpeg` decodes with no error and the right frame count. A missing tool fails, on CI and D1 alike |

`a_frame_out_of_its_turn_is_out_of_order` stays as written.

### Step 3.5 - Prove the guards

| Temporary edit | Rule that must fire |
|---|---|
| `stbl` passed `None` for `ctts` | `real_b_frames_come_back_at_30_fps` fails on the deltas |
| Write `ctts` even when all offsets are 0 | `no_reordering_writes_no_ctts_and_no_video_edit` fails |

Undo both.

### Milestone 3

```bash
cargo test -p offcut-mp4                                     # expect: all pass
cargo clippy -p offcut-mp4 --all-targets -- -D warnings      # expect: clean
cargo fmt --check                                            # expect: clean
node scripts/check-file-tree.mjs                             # expect: exit 0
git diff "$BASE" -- crates/offcut-mp4/tests/mux_roundtrip.rs | grep -E '^-[^-]' # read: only the one comment line removed
# the frozen check                                           # expect: no output
```

Pass: the existing cases of `mux_roundtrip.rs` pass with only one comment changed, a file with no reordering is byte-identical to before (asserted), and `ffprobe` reads 1/30 s deltas on the re-muxed B-frame file.

Commit: `"V5 P131: ctts, video edit list, moov overflow guard tested"`. No push.

Do not start Phase 4 until this passes.

---

## Phase 4 - Program time and the ladder (P132; §10, §16, D-213 to D-219, D-238)

Goal: S-8 is built, and an encoder that fails does not fail the export.

### Step 4.1 - `prog48.f32` (D-213, §10.1)

| Order | File | Watch for |
|---|---|---|
| 1 | `web/src/persistence/opfs.ts` | `paths.clipProg48`, `writeProgram`, `readProgram`: `out48`'s three in the same shape |
| 2 | `web/src/usecases/run-pipeline.ts` | Step 13: after `const prog48 = owned(program.prog48)`, `run.stage = "storage"`, then `await opfs.writeProgram(clipId, prog48)`. A write copies and does not transfer, so `prog48` stays valid for `clipStore.ready`. The step's comment is corrected. `opfs.writeAudio` stays for `out48`, before `assemble`, because `assemble` takes it by transfer |
| 3 | `web/src/state/clip-store.ts` | The comment on `prog48` (line 62) |
| 4 | `web/src/usecases/run-pipeline.test.ts` | The OPFS mock gains `writeProgram`, and the order assertion gains it |

### Step 4.2 - `exportClip` takes `prog48` (D-214, §16)

**REOPENED CONTRACT `RenderWorkerApi.exportClip`.** One commit, both sides.

| Order | File | Watch for |
|---|---|---|
| 1 | `web/src/workers/protocol.ts` | `prog48: Float32Array`, with its comment (TS §40.9 S-8) |
| 2 | `web/src/workers/render/export-job.ts` | Both signatures and the call |
| 3 | `web/src/usecases/start-export.ts` | `opfs.readProgram`; `{ exportId, entitlementToken, prog48 }`, `transfer: [prog48.buffer]`; the `V5:` marker at line 161 goes |

### Step 4.3 - The loop on program time (D-215, §10.2)

`web/src/workers/render/export-loop.ts`. In the frame loop, in place of today's lines 349-357:

```ts
// Output frame n is the program at n x 1000 / 30 ms, in whole milliseconds;
// its own timestamp is the same in microseconds. The scene says what of the
// recording it shows (TS §40.9 S-8): with the identity map, the same time.
const t = Math.floor((n * 1000) / LIMITS.OUTPUT_FPS) as OutMs;
const timestamp = Math.floor((n * MICROS_PER_SEC) / LIMITS.OUTPUT_FPS);
const pos = a.session.source_at(t);
if (pos === null) {
  throw new WorkerFailure("E_INTERNAL", "NoScene");
}
// The frame stays the source's: it is not closed here.
const frame = await a.source.frameAtBlocking(recorded(pos));
a.session.render_frame(frame, t);
```

`recorded` comes from `./preview-loop`; the `TimeMs` import goes; the file's head says "over the program's timeline". `a.out48` becomes `a.prog48`.

### Step 4.4 - The ladder (D-216, D-217, §10.3)

| Order | File | Watch for |
|---|---|---|
| 1 | `web/src/workers/render/encoders.ts` | `supportedConfigs(profile)`, an async generator of the entries `isConfigSupported` accepts, in order. `pickVideoConfig` becomes "the first of them, or `E_ENCODE_VIDEO`" and keeps its signature |
| 2 | `web/src/workers/render/opfs-sink.ts` | `reset()`: `handle.truncate(0)`, tagged as `writeAt` is. Closed: `io` |
| 3 | `web/src/workers/render/export-loop.ts` | Today's body becomes `attempt(a, config)`, unchanged except its first line. The new `runExport` is §10.3's, verbatim. The encoders, `Watch`, `Writer` and the muxer are made inside `attempt`; its `finally` closes them |
| 4 | Same | `Writer.video`'s comment about "out of turn" is updated (D-211) |
| 5 | `web/src/ui/components/ExportProgress.tsx` | `useTimeLeft` resets `first.current` when `done` falls below `first.current.done` (D-238). `exportStore.progress` already accepts the fall |

Only `E_ENCODE_VIDEO` moves down the ladder; `E_ENCODE_AUDIO` retries once with the same entry. A sink error, `E_GPU_LOST`, `E_MUX` and a cancel are never restarted.

### Step 4.5 - The dev fault (D-218)

| Order | File | Watch for |
|---|---|---|
| 1 | `web/src/workers/rpc-worker.ts` | `devFault` gains `devFailEncoder` with `{ at, times }`, kept in module state; `export function takeEncoderFault(n: number): boolean`, `false` unless `DEV_FAULTS` and `n === at` and `times > 0`, which then lowers `times` |
| 2 | `web/src/workers/render/export-loop.ts` | Before `venc.encode`: `if (takeEncoderFault(n)) { throw new WorkerFailure("E_ENCODE_VIDEO", "DevFault"); }` |
| 3 | `web/src/usecases/dev-faults.ts`, `web/src/ui/dev-hooks.ts` | `failEncoderAt(frame, times)`, through `pool.render.notify` |
| 4 | `scripts/check-hosts.mjs` | `DEV_MARKERS` gains `devFailEncoder`: the list is explicit, so a name not on it proves nothing |

### Step 4.6 - The loop's test (D-219, §17.3)

`web/src/workers/render/export-loop.test.ts`, with fake `VideoEncoder`, `AudioEncoder`, `VideoFrame`, `AudioData`, a fake session, source and sink, `loadRender` mocked. A row in TS §5. Every case of §17.3's table, in this order: frame accounting, identity, **the seam (a map that jumps and holds)**, `null` position, audio, backpressure, video failure, all entries fail, unsupported entry, audio failure, not restarted, cancel, `liveFrames` is 0 after every case. This is where most of the proving happens, in seconds.

```bash
pnpm --filter web exec vitest run src/workers/render/export-loop.test.ts   # expect: all pass
pnpm --filter web exec vitest run src/usecases/run-pipeline.test.ts        # expect: all pass
```

### Step 4.7 - One real export (Tier 2)

The muxer changed, so this is a full keyed build.

```bash
export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=$(grep -E '^ +E2E_TEST_PUBLIC_KEY:' .github/workflows/ci.yml | awk '{print $2}')
pnpm build:e2e
pnpm e2e:media export-creator.spec.ts --grep "Creator export" --max-failures=1 2>&1 | tail -3   # expect: 1 passed, all 11 checks
unset VITE_ENTITLEMENT_TEST_PUBLIC_KEY
```

A real `failEncoderAt` run is Phase 5's case. Do not write it twice.

### Step 4.8 - Prove the guards

| Temporary edit | Rule that must fire |
|---|---|
| `frameAtBlocking(t as TimeMs)` back in the loop | The seam case of `export-loop.test.ts` fails |
| `a.sink.reset()` removed | The video-failure case fails |
| A restart on `E_GPU_LOST` added | "Not restarted" fails |

Undo all three.

### Milestone 4

```bash
git grep -n "out48" -- web/src/workers/render                # expect: no output
git grep -n "V5" -- web/src/workers/render/export-loop.ts    # expect: no output
wc -l web/src/workers/render/export-loop.ts                  # expect: under 700
pnpm check                                                   # expect: exit 0
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"   # expect: above 175, by the new file
# the frozen check                                           # expect: no output
```

Pass: the loop's Vitest table is green, a Creator export still passes all 11 checks, and no `out48` is left under `workers/render/`.

Commit: `"V5 P132 (S-8): export over program time, prog48, ladder restart"`. No push.

Do not start Phase 5 until this passes.

---

## Phase 5 - The proof (P133; TS §27.1, §17.5, D-210)

Goal: BP §7's demo is a test. 20 s in, 20 s out, in sync on both sides.

### Step 5.1 - The spec, without what Phases 6 and 7 add

`web/tests-e2e/timeline-preserved.spec.ts`. Two serial groups, one per fixture; each processes its clip once. `watchExport` moves from `export-creator.spec.ts` to `helpers/fixtures.ts`, so both specs read the bar. Word text is never printed (SAB rule 9).

`speech_20s_long_pause.mp4`:

| Order | Case | Asserts | Added in |
|---|---|---|---|
| 1 | Words keep their times | From `readIndexedDb(page, "transcripts", clipId)` (`clipId` from `opfsList(page, "clips")`): every word's `start_ms` and `end_ms` lies outside the pause or at its edges; the first word after it starts at 10,000 ms or later | Here |
| 2 | Caption at its spoken time | Seek to that word's start + 60 ms: the caption mirror has it. 1,000 ms before: it does not | Here |
| 3 | Preview's length | The scrub bar's `max` is the source's duration; played from 500 ms before the end, Play returns and `previewAt` is at or under it | Here |
| 4 | Export is 20 s | Creator: `verifyMp4(file, "creator", 20_000, fixture)`, `expectAllChecks(verdict, { flashes: 2 })`; 600 frames (check 3 reads them) | Here |
| 5 | An edit leaves the timeline alone | Fix a word, switch a visual off, choose Bold: `max` unchanged; export again; `expectAllChecks(…, { flashes: 2 })` | Here; `from_cache` in Phase 6; values and rows in Phase 7 |
| 6 | The same edit again | | Phase 6 |
| 7 | Undo everything, then export | | Phase 6 |
| 8 | The encoder fails | `__offcutDev.failEncoderAt(30, 1)`: the bar falls back; the file passes the verifier; no failure card | Here |
| 9 | Every entry fails | `failEncoderAt(30, 9)`: the card with `E_ENCODE_VIDEO`'s words, "Try again" and "Close"; no file in `exports/` or `exports/tmp/`; the preview plays again | Here |

`speech_30s_sparse.mp4`:

| Case | Asserts |
|---|---|
| Words stay in their pieces | Each word lies inside a piece placed at 0.5, 7.0, 13.5, 19.5 or 26.0 s (± 300 ms); at least three pieces hold a word |
| Export is 30 s | `verifyMp4(file, "creator", 30_000, fixture)`, `expectAllChecks(…, { flashes: 2 })`; 900 frames |

Until Phase 6 a token must be seeded before each export; with the cache absent every export renders. `web/playwright.config.ts`: the spec joins `MEDIA_SUITES`. Its row in TS §5 exists since v1.1.

### Step 5.2 - Finish `export-creator.spec.ts`

What Phase 2 left: `expectAllChecks` on every export, and the comment at the top of the file.

### Step 5.3 - Run it (Tier 2)

Only `web/tests-e2e` changed since Phase 4's build, so no build.

```bash
pnpm e2e:media timeline-preserved.spec.ts --grep "Words keep" --max-failures=1 2>&1 | tail -3
# …each case alone first (§6.1), then the file once:
pnpm e2e:media timeline-preserved.spec.ts 2>&1 | tail -3     # expect: passed; about 3 min (estimate)
```

| If | Do |
|---|---|
| A word lies inside the pause | The recognizer heard something in room tone. Read the fixture's text in `fixtures/speech/README.md` first. INV-5 says a word is not moved, not that one cannot be heard: if it stays at its own time and no word was moved earlier, the case's wording is too strict; record why |
| Check 8's correlation is low on the sparse fixture | §21: tune `MIN_ENVELOPE_CORR` on a saved export. Do not narrow the windows, which TS §27.2 fixes |
| `failEncoderAt(30, 1)` ends in the failure card | D1 has one supported entry at this size. Read `supportedConfigs`' result and record it. The Vitest case still proves the restart, so the browser case is skipped on that machine with the reason in its message, and the founder is told in the reply |

### Milestone 5

```bash
pnpm --filter web exec playwright test --project=media --list | tail -1   # read: 77 + the new cases of this phase + setup
pnpm check                                                   # expect: exit 0
# the frozen check                                           # expect: no output
```

Pass: both fixtures pass 11 checks with their flashes compared, the encoder-failure cases pass, and the word times hold.

Commit: `"V5 P133: timeline-preserved on both pause fixtures, all 11 checks"`. No push.

Do not start Phase 6 until this passes.

---

## Phase 6 - The cache and the gate (P134; §11, §12, §13, §16, D-220 to D-231)

Goal: S-9. The same edit twice is instant, and no file reaches someone the token does not entitle.

### Step 6.1 - The record, the repository, the version (D-222, D-223, D-226, §11)

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/src/persistence/schema.ts` | D-226 | `RenderCacheRecord.summary: ChangeSummary`. `DB_VERSION` stays 1: no build has written a row |
| 2 | `web/src/persistence/render-cache-repo.ts` | §11.1 | `CacheInputs`; `canonicalJson` (sorted at every depth, `undefined` left out); `cacheKey` by `crypto.subtle`; `lookup` (row plus `getFile().size`; a row without its file is deleted and is a miss; a failed IndexedDB call is a miss); `put` (a failure is swallowed); `heldBytes`; `clear`; `RENDERER_VERSION`. `schemaVersion > DB_VERSION` as `edits-repo.ts` treats it. `opfsPath` is `exports/<uuid>.mp4`: no file name of the user's (P-11) |
| 3 | `web/vite.config.ts` | D-223 | `define` gains `__RENDERER_VERSION__`: SHA-256 with Node's `crypto` of `src/wasm/pkg/render/offcut_render_bg.wasm`, `src/workers/render/export-loop.ts` and `src/workers/render/encoders.ts`, 16 hex digits; `"none"` when the WASM has not been built |

### Step 6.2 - `exportProfile` (D-220, §16)

**REOPENED CONTRACT `RenderWorkerApi`.** `workers/protocol.ts`: `exportProfile(p: { entitlementToken: string }): Promise<ExportProfile>`. `workers/render.worker.ts`: the handler of §16, on the object that already has `exportProfile` at line 236; a thrown failure goes through `staged(thrown, "render_encode")`, because the frozen case reads that stage. `exportClip` still verifies the token itself: the worker trusts nothing the page says about a profile.

### Step 6.3 - The gate (D-229, D-230, D-231, §12)

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/src/persistence/opfs.ts` | §12.2 | `freeBytes()`: `quota - usage` from `navigator.storage.estimate()`, `null` when the browser does not say |
| 2 | `web/src/state/blockers.ts` | §12.1 | `forExport(room?)`, `STORAGE_MARGIN = 1.2`, `ExportRoom`, the `V5` marker of its comment goes. `free = needed x 1.2` exactly is not blocked |
| 3 | `web/src/state/export-store.ts` | §12.3 | `cacheHit(original)`, `blockedBy(code)`, `saved(summary, answered)`, `answered()`, `hasAnswered(id)`. `empty()` drops `blocker`, `summary`, `answered`. The set of answered ids is a module-level `Set` here (D-234). The type import from `blockers.ts` is type-only, so no cycle at run time |
| 4 | `web/src/state/machines/export-machine.ts` | §12.3 | The comment at its head only: V5 fires `cache_hit`. The table does not change |
| 5 | `web/src/ui/components/ExportButton.tsx` | D-231 | `export-status` says `messages.blockers[blocker]` from the store when set. No `ErrorPanel`: nothing failed |

### Step 6.4 - The copy

`web/src/copy/messages.ts`: `B_STORAGE_LOW`, §14.3's words. `scripts/check-copy-codes.mjs`: remove it from `COPY_PENDING` in the same commit (Read this first, item 9).

```bash
node scripts/check-copy-codes.mjs                            # expect: ok; Pending one fewer
```

### Step 6.5 - `start-export.ts` (§13)

Replaces `startExport` and `renderAndDeliver`, in §13's twelve steps. The order is the contract:

| Step | Watch for |
|---|---|
| 3 | `exportStore.clear()` no longer follows `start`: the export stays in `gating` until step 9 |
| 4 | `lockForExport` before anything calls the worker (D-221) |
| 5 | `answered(await whenFree(() => pool.render.exportProfile({ entitlementToken })))`. `answered` is the three-line local helper of `control-preview.ts`, copied here as `review-update.ts` copied it |
| 6 | `clipsRepo.get(clipId)` for `sourceHashPrefix`. It is not in the clip store. A failed read gives `key = null`: no cache, the export goes on |
| 7 | A hit: `started = true` first (so a vanished file reports no `export_started`), `cacheHit`, deliver under today's name, `saved(hit.summary, hasAnswered(hit.exportId))`, unlock, `export_done { from_cache: true }`. No `export_started`, no `stage_timing` (D-227) |
| 8 | `freeBytes` plus `heldBytes`; a blocker is `blockedBy` then unlock then return, with nothing swept |
| 9 | `clear()`, then `export_started` with the verified kind |
| 10 | `renderCacheRepo.clear()` before the sweep: row, then file (TS §23.3's write order) |
| 11 | `put` after the file is in place, with `clipsRepo.toBytes(size)`; no `as Bytes` |
| Catch | Cancel during `gating` calls `blocked()`, since the table has no `gating → cancelled`. A failure during `gating`: `clear()`, then `fail`, with `export_started` (from `planOf`, D-230) then `export_failed`. The frozen cases read exactly this |

`export_done`'s placeholders stay until Phase 7, except `from_cache`. The `V5:` comment at line 305 is reduced to what is left.

**`planOf` never reaches `cacheKey`** (problem 2). The keys of `CacheInputs` are `theme: "default"`, `musicId: null` (D-224).

### Step 6.6 - The import (D-228, problem 6)

`web/src/usecases/import-clip.ts`, step 4: `renderCacheRepo.clear()` joins the `Promise.all`. Its layer already allows the import.

### Step 6.7 - The tests (§17.3, §17.5)

| Order | File | Watch for |
|---|---|---|
| 1 | `web/src/state/machines/export-machine.test.ts` | `clip-machine.test.ts`'s shape: every legal row of TS §12.2 through `transition` and the store's action; every other pair refused; the two named in §12.3 |
| 2 | `web/src/state/blockers.test.ts` | §17.3's list, including free = needed x 1.2 exactly, one byte under, no `room`, `room` in `gating` |
| 3 | `web/src/usecases/start-export.test.ts` | `run-pipeline.test.ts`'s scripted pool. `vi.mock` for the repos (no IndexedDB in Vitest); `crypto.subtle` is real. `document`, `URL.createObjectURL` stubbed. The groups of §17.3's table: **`render-cache` covers all ten inputs**, plus "does not change with the order of `word_edits`' keys, or an edit made and undone". S-9's proof |
| 4 | `web/tests-e2e/export-creator.spec.ts` | After "second export": same token, forged, no room, expired (§17.5). `no room` stubs `navigator.storage.estimate` on the page and deletes the stub after. Because the group is serial, run them with their stored file: `--grep "Free-plan|second export|same token|forged|no room|expired"` |
| 5 | `web/tests-e2e/timeline-preserved.spec.ts` | Case 5: a miss, `from_cache: false`. Case 6: a hit, same bytes (SHA-256), instant. Case 7: undo all, a miss (D-225), the file passes |

### Step 6.8 - Prove the guards

| Temporary edit | Rule that must fire |
|---|---|
| `planOf(token)` in place of `profile.kind` in the key | The `Verified profile` group fails: this is problem 2 |
| `JSON.stringify` in place of `canonicalJson` | The "order of `word_edits`' keys" case fails |
| `renderCacheRepo.clear()` removed from `importClip` | The `Import` case fails: problem 6 |
| `clear()` left out of the catch for a refused token | The refused-token case: an illegal transition is reported |

Undo all four.

### Step 6.9 - Run it (Tier 2, then Tier 3)

```bash
pnpm --filter web exec vitest run src/state src/usecases/start-export.test.ts   # expect: all pass
export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=$(grep -E '^ +E2E_TEST_PUBLIC_KEY:' .github/workflows/ci.yml | awk '{print $2}')
pnpm --filter web exec vite build --mode test                # seconds
grep -rl "__RENDERER_VERSION__" web/dist && echo LEFT || echo replaced   # expect: replaced
pnpm e2e:media export-creator.spec.ts --grep "Free-plan|second export|same token|forged|no room|expired" --max-failures=1 2>&1 | tail -3
pnpm e2e:media timeline-preserved.spec.ts --grep "An edit leaves|same edit|Undo everything" --max-failures=1 2>&1 | tail -3
pnpm e2e:media cancel.spec.ts --grep "export" --max-failures=1 2>&1 | tail -3   # it reads renderCacheCount
unset VITE_ENTITLEMENT_TEST_PUBLIC_KEY
```

### Milestone 6

```bash
# Tier 3
pnpm check && pnpm fixtures && pnpm test                     # expect: exit 0; Rust 0 failed; Vitest above Phase 4's
pnpm e2e 2>&1 | tail -2                                      # expect: 12 passed
# the frozen check                                           # expect: no output
git push                                                     # do not wait (D-198)
```

Pass: §11.3's table is green, each row a test; a hit's `total_ms` is under 2 s; the storage cases of D-40 are unchanged.

Commit: `"V5 P134 (S-9): render cache on the verified profile, the gate, B_STORAGE_LOW"`. **Push, second of three. Do not wait.**

Do not start Phase 7 until this passes.

---

## Phase 7 - What the person sees (P135; §14, §15, D-232 to D-237)

Goal: J11. The export tells the truth about what changed, and the founder gets the number PS §20.5 needs.

First: read CI of Phase 6 (`gh pr checks <pr>`). Red: fix it now.

### Step 7.1 - Copy (§14.3)

`web/src/copy/messages.ts`: `export.summary` and `export.question`. Every number passes through a function (INV-15). No absolute claim: "Voice cleaned" states that the chain ran.

### Step 7.2 - The components (D-232, D-233, D-234, §14)

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/src/usecases/answer-post-export.ts` | §14.5 | Needs `state` and `analytics` only. Once per `ExportId` |
| 2 | `web/src/ui/components/PostExportQuestion.tsx` | §14.1 | Three `Button`s (`secondary`, `sm`), `PostExportAnswer` from the generated type; the thank-you line after an answer |
| 3 | `web/src/ui/components/ChangeSummary.tsx` | §14.1, §14.2 | `summaryRows(summary)`, one row per field, zero absent; `ExportOutcome`, shown when `status === "done"` and not closed; closing is local, keyed by `exportId`. A `section` with `aria-labelledby`, announced through the editor's one live region (D-192). Focus does not move when it appears |
| 4 | `web/src/ui/layout/SidePanel.tsx` | D-232 | `.export` gains `<ExportOutcome />`, under `ExportFailure`. Nothing else on the page moves |
| 5 | `web/src/ui/styles/components.module.css` | §14.1 | Tokens only; nothing new in `tokens.css` |

### Step 7.3 - The real values (D-237, §15, problem 4)

`web/src/usecases/start-export.ts`: `editValues(clip)` as §15, computed when the export starts, not when it ends. `export_done` spreads it in both the hit and the miss path. The last `V5:` comment goes.

### Step 7.4 - The tests

| Order | File | Case | Asserts |
|---|---|---|---|
| 1 | `start-export.test.ts` | `Real values` | `word_edits` 2; one override off gives kept and disabled; `style`; `crop_adjusted`. **A switched-off visual is not kept** |
| 2 | Same | The question on a hit | A hit after an answer does not ask again |
| 3 | `export-creator.spec.ts` | Events | Values as today: the clip is unedited |
| 4 | Same | **new** The summary and the question | On the done state: `What we changed` with the voice row, zero rows absent. Answer "With small edits": `post_export_answer { answer: "small_edits" }` once; thanks line. `axe` on this state |
| 5 | `timeline-preserved.spec.ts` | An edit leaves the timeline alone | `word_edits: 1`, `events_disabled` as made, `style: "bold"`; the rows follow the edit |

Order within `export-creator`'s group (§17.5): no token, other key, Creator, progress, events, name, storage, quiet, preview after, Free, second export, **summary and question, same token, forged, no room, expired**.

### Step 7.5 - The capture (D-235)

| Order | File | Watch for |
|---|---|---|
| 1 | `web/src/ui/dev-hooks.ts` | `fakeExportDone()`: `useExportStore.setState({ status: "done", summary: { voice_cleaned: true, captions_emphasized: 3, visual_moments: 2 }, answered: false, … })`. `fakeExport` reaches `rendering` only |
| 2 | `web/tests-e2e/screens.spec.ts` | One capture, the done state, at both widths. Its `Window` declaration gains the hook |

```bash
pnpm screens                                                 # dev build; expect: 18 PNGs
mkdir -p ~/v5-review && cp web/test-results/screens/*done* ~/v5-review/
```

Look at both yourself first: 4 px grid, nothing clipped, the panel below stays usable and scrolls.

### Step 7.6 - Prove the guards

| Temporary edit | Rule that must fire |
|---|---|
| `kept` back to `event.enabled` alone | The `Real values` group fails: this is problem 4 |
| `summaryRows` pushes a row for a zero | The summary case: "zero rows absent" |
| `answered()` called without the check | "One answer, one event" fails |

Undo all three.

### Milestone 7

```bash
pnpm check                                                   # expect: exit 0
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"   # expect: above Phase 6's
export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=$(grep -E '^ +E2E_TEST_PUBLIC_KEY:' .github/workflows/ci.yml | awk '{print $2}')
pnpm --filter web exec vite build --mode test
pnpm e2e:media export-creator.spec.ts --grep "Creator export|events:|summary and the question" --max-failures=1 2>&1 | tail -3
unset VITE_ENTITLEMENT_TEST_PUBLIC_KEY
git grep -nE "\bV5\b" -- web/src crates verify               # expect: no marker of work to do; a file's history may stay
# the frozen check                                           # expect: no output
```

Pass: the rows follow the edit, zero rows are absent, one answer gives one event, `export_done`'s values are right after an edit, and the done state passes axe.

Commit: `"V5 P135: ChangeSummary, the question, real export_done values"`. No push.

Do not start Phase 8 until this passes.

---

## Phase 8 - Measure, final gate, merge, deploy, tag (P136; §20, §24, §25)

Goal: V5 is measured, recorded, merged, deployed and tagged. This is the one phase with a full media run, and the one that waits on CI.

### Step 8.1 - The bench (D-200, §20)

Docker closed, nothing else on the GPU (V4 CL known issue 37). A keyed production build, then about 14 minutes:

```bash
export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=$(grep -E '^ +E2E_TEST_PUBLIC_KEY:' .github/workflows/ci.yml | awk '{print $2}')
pnpm build
BENCH_DEVICE=d1 BENCH_CLIP=portrait BENCH_RUNS=5 pnpm bench:device 2>&1 | tail -3   # in the background
```

### Step 8.2 - The readings (§20)

`docs/v5/experiments.md` in V4's shape (Question, Method, Numbers, Date, Decision), all labelled `d1`, with a row in TS §5 already there.

| Reading | Method | Target | If it misses |
|---|---|---|---|
| E-4 `render_encode` | The bench | At most 90 s per 60 s. V4 read 38.8 s | Over 150 s: BP §4.2's Canvas2D path. Not expected |
| `mux` | Same | At most 2 s. V4 read 25 ms | Look at it |
| `detect_scene` | Same | About 245 ms | Still over on a quiet machine: an issue for V9 with the numbers, not a V5 blocker. Closes V4 CL issue 37 either way |
| Loudness | Phase 2's `INFO` | -14 ± 1 LUFS, and the "before" | §8 |
| Sync | Phase 2's `INFO` | Within 20 ms; within a frame | Look before moving anything |
| Watermark correlation | Phase 2's `INFO` | ≥ 0.5, ≤ 0.2 | §7.3 |
| Bitrate | Phase 2's `INFO` | ≤ 12 and ≤ 6 Mb/s | §21 |
| Hit time | `export_done.total_ms` with `from_cache: true` | Under 2 s | Look at `lookup` |
| `cacheKey` cost | A temporary `performance.measure`, removed | A few ms | Move it to a worker |
| App shell | `pnpm build` | ≤ 400 kB gzip | Look at what grew |

Every value goes to TS §39.3 item 14 ("the verifier's thresholds are V5's").

### Step 8.3 - Copy back (§25)

From the code, into: TS §5 (new files, 26 fixtures), §9 (C-7, C-9, C-10), §12.2's prose, §12.4, §12.5, §13.3, §14.2, §18.2, §21.1, §21.3, §21.4, §23.1, §27.1, §27.2, §33, §39.3 item 14, §40.9 (S-8 and S-9 as built). BP §7 "as built". `verify/README.md`. "V5 as built" as a table in `v5implementation.md`, as V4 §24.3 did.

```bash
node scripts/check-file-tree.mjs                             # expect: exit 0 (a §5 edit: docs-only CI would not see it)
```

### Step 8.4 - Tier 4, once (D-197)

```bash
pnpm install --frozen-lockfile && pnpm check                 # expect: exit 0
set -a; . ./.env; set +a
pnpm fixtures && pnpm test                                   # expect: exit 0; start Docker first for this one, then close it again
cargo test --workspace 2>&1 | awk '/^test result:/ {p+=$4; f+=$6; i+=$8} END {print p" passed, "f" failed, "i" ignored"}'
# expect: above 646, 0 failed, 4 ignored
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"   # expect: above 175
pnpm e2e 2>&1 | tail -2                                      # expect: 12 passed
python verify/test_verify_mp4.py 2>&1 | tail -1              # expect: OK
pnpm build:e2e
pnpm e2e:media 2>&1 | tail -3                                # in the background; expect: 93 passed, 1 skipped (77 + 16 new), about 30-40 min (estimate)
unset VITE_ENTITLEMENT_TEST_PUBLIC_KEY
pnpm build && node scripts/check-hosts.mjs 2>&1 | tail -3    # expect: exit 0, no development marker; the app shell's size
git grep -nE "\bfetch\(" -- web/src | grep -vE "net/http\.ts|net/asset-fetch\.ts|wasm/load-core\.ts|wasm/load-render\.ts"   # expect: no output
git diff --name-only 0a8aa80 -- fixtures/golden 'crates/**/*.snap'   # expect: no output
```

Write the final counts and the full suite's time against V4's 41.8-54.9 min into the changelog. Close known issues 33 and 36 by reading V5's runs: no first-attempt failure of those cases. If there was one, keep the issue open with the run's number.

### Step 8.5 - Push and wait

```bash
git push && gh pr checks <pr> --watch                        # third push; about 30 min; expect: every check pass
```

Red: fix it, targeted runs per D-197, push again. Commit: `"V5 P136: bench, experiments, copy-back, final gate"`.

### Step 8.6 - Merge and deploy (founder)

**Founder:** `gh pr ready <pr>`, then merge. `ci.yml` on `main` runs everything and deploys the API then the web app. Nothing in the server changed, so the API image rebuilds with the same code.

```bash
export APP_URL=https://offcut-one.vercel.app
git fetch origin && MERGE=$(git rev-parse origin/main)
gh run watch                                                 # expect: changes, ci, e2e-media, deploy-api, deploy-web all success
curl -s "$APP_URL/api/v1/healthz"                            # expect: 200 and the commit $MERGE
node scripts/check-headers.mjs "$APP_URL"                    # expect: the 7 headers, value for value
```

### Step 8.7 - On the deployed page, and the human gate (D-235)

| Check | Where | Expect |
|---|---|---|
| `crossOriginIsolated` | DevTools on `$APP_URL/app` | `true` |
| The reference clip | Deployed app | Previews |
| Export | Deployed app | "Exporting needs an account" (BP §7.2). No media leaves |
| BP §7's demo | Locally, keyed build, minted token | The long-pause fixture exports at 20 s; the second export is instant |
| **Founder** | `~/v5-review/` | One look at the done state, at both widths |
| **Founder** | An export from the local keyed build | One listen: the loudness is what they want. D-204 is a product change |

A requested change that is not cosmetic becomes a V6 item. A cosmetic one is fixed on a small branch from `main` with its own pull request, before the tag.

**Rollback.** If `healthz` does not report `$MERGE` within 15 minutes, or a deployed check fails, prepare `git revert -m 1 $MERGE` on a branch from `main` and a pull request for the founder; confirm the deployed app behaves as V4; then fix on `v5-build` and repeat Steps 8.4 to 8.7.

### Step 8.8 - Tick and tag

| §24 group | Where it was proven |
|---|---|
| Code and checks | Steps 8.4, 4.8, 6.8 |
| Speed | Phase 1, Step 8.4 |
| Measurements | Steps 8.1, 8.2 |
| Deployment | Steps 8.6, 8.7 |
| Records | Steps 8.3, 8.7, and §23.3 |

**Founder:**

```bash
git tag -a v5 -m "V5: export that an independent tool accepts" "$MERGE" && git push origin v5
```

Then the closing entry on the merge path: "V5 closed: <date>, merge <SHA>, final counts, the founder's look and listen".

### Milestone 8

```bash
curl -s "$APP_URL/api/v1/healthz"                            # expect: 200, the merge commit (or the closing entry's after it)
node scripts/check-headers.mjs "$APP_URL"                    # expect: exit 0
git ls-remote --tags origin v5                               # expect: one line
grep -c "^- \[ \]" docs/v5/v5implementation.md               # expect: 0, or each one named in the closing entry
git ls-files bench/results | grep "d1-2026-1[01]"            # expect: the new bench file
grep -c "^Decision:" docs/v5/experiments.md                  # expect: as many as §20 has rows, none empty
```

Pass: the deployed page previews and asks for an account, the demo works locally, the founder looked and listened, every box of §24 is ticked, and `v5` is pushed.

Commit: `"V5 P136: bench, experiments, copy-back, final gate"`; the closing entry after the tag.

V6 must not start until this passes (§23.3).

---

## Quick reference

### Start every terminal

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut && set -a; . ./.env; set +a
export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:/c/Users/Mayan/AppData/Local/Microsoft/WinGet/Links:$PATH"
export BENCH_DEVICE=d1; BASE=0a8aa80
git branch --show-current && python --version && git status --short | head -5   # v5-build; 3.13.x
```

Add the key only in a terminal that builds for the media suites, and unset it before the gate:

```bash
export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=$(grep -E '^ +E2E_TEST_PUBLIC_KEY:' .github/workflows/ci.yml | awk '{print $2}')
```

### After changing ... run ...

| After changing | Run |
|---|---|
| `offcut-dsp` | `cargo test -p offcut-dsp`, then `pnpm build:e2e`; the preview's loudness moves with it |
| `offcut-mp4` | `cargo test -p offcut-mp4`, then `pnpm build:e2e` before any export |
| `verify/verify_mp4.py` | `python verify/test_verify_mp4.py` (or `<Case>`), then the saved exports, offline |
| `web/src` only | `pnpm --filter web exec vite build --mode test`, then the named cases |
| `export-loop.ts`, `encoders.ts` | `vitest run src/workers/render/export-loop.test.ts`; the renderer version changes by itself |
| `start-export.ts`, the cache, the gate | `vitest run src/usecases/start-export.test.ts src/state` |
| `web/tests-e2e` only | No build. The case alone: `pnpm e2e:media <spec> --grep "<case>" --max-failures=1` |
| `fixtures/gen_fixtures.sh`, `manifest.json` | `pnpm fixtures`; `cargo test -p offcut-mp4 --test demux_fixtures` |
| `watermark.rs`, `safe_area.rs` | `make_watermark_mask.py` once more; the self-test's watermark case fails until you do |
| `web/src/copy/messages.ts` | `node scripts/check-copy-codes.mjs` |
| A new file under `crates/`, `web/`, `verify/`, `scripts/` | Its row in TS §5, then `node scripts/check-file-tree.mjs` |
| `docs/technicalspec.md` §5 | `node scripts/check-file-tree.mjs` before pushing: a docs-only pull request skips CI |
| `.github/workflows/*.yml` | Push, read the run at the next phase's start |
| Anything at all | The frozen check of Step 0.5 |

### When something fails

| When something fails | Likely cause |
|---|---|
| `verify_mp4` exits 2 | Plain `python` is 3.9 with no NumPy. The Quick reference's `PATH` line |
| `FAIL 7` at about -11 LUFS | The WASM predates the fix: `pnpm build:e2e`. Or `Center` is back in `loudness.rs` |
| `FAIL 8`: low correlation | Sparse speech; tune `MIN_ENVELOPE_CORR` on a saved export. Never narrow the windows |
| `FAIL 9` on a Free file | The mask is misaligned; compare it with a frame. Or the watermark moved: run the tool again |
| `FAIL 10` | A hardware encoder over 1.5x: `bitrateMode: "constant"` on that entry. TE-4 changes both copies of the constant or neither |
| `not measured` in `expectAllChecks` | `--source` missing, or a window with no speech |
| `review_action` holds `undo` in a shared case | `sendQueuedAnalytics` missing, or the 300 ms too short on a loaded machine |
| A toast's text from the case before | `resetClip` did not wait, or the pointer is on the toast, which pauses its timer |
| `E_MUX` `OutOfOrder` on a real export | A chunk beyond `MAX_REORDER`, or `Writer.video`'s frame is wrong. Read the timestamp |
| `Busy` from `exportProfile` | The call is not in `whenFree`, or the lock is after it (D-221) |
| An illegal transition `gating → failed` is reported | `clear()` missing before `fail` (D-230) |
| Second export is not a hit | Key input not stable: print `canonicalJson`'s diff (numbers and enums only, never a transcript) |
| A hit on the first export after a re-drop | `importClip` does not clear `renderCache` |
| Bench reports no `render_encode` | Same: its runs re-drop one clip |
| `exports/` holds two files | The sweep ran before the row was cleared, or a hit swept |
| `check-copy-codes`: "has copy and is still in COPY_PENDING" | Step 6.4's second half |
| `screens` has no done state | `fakeExportDone` missing, or its `Window` declaration in `screens.spec.ts` |
| CI's `changes` fails on `origin/$GITHUB_BASE_REF` | `fetch-depth: 0` missing on that job's checkout |
| `healthz` reports the old commit after the merge | The deploy failed or Render is still building; Step 8.7's rollback rule |

---

## Assumptions I made and gaps found

### (a) Environment values assumed

| Value | Assumed | How to correct |
|---|---|---|
| OS and shell | Windows 11 (D1), Git Bash for every command | Step 0.1 runs there |
| Repo path | `/c/Users/Mayan/Desktop/sh2clips/offcut` | Replace once in the Quick reference line |
| Python | 3.13 at `AppData/Local/Programs/Python/Python313`, NumPy 2.5.3; `ffmpeg` 9.0.2 | Read on 2026-10-31 (§1A box 6) |
| App URL | `https://offcut-one.vercel.app` | Read it from Vercel if it changed |
| Pull request number | `<pr>`, opened by the founder in Step 0.4 | Replace once known |
| Times and final media count | Estimates: 15-16 h; 93 passed + 1 skipped | Read them at Step 8.4 |

### (b) Gaps and contradictions found while writing this guide, and how they are resolved

Each is applied to `v5implementation.md` in the same change, and listed in its §25 (rows 20 to 25) where it changes the plan.

| # | Finding | Resolution | Where |
|---|---|---|---|
| 1 | `loudness.rs`'s `..._reads_minus_23` test asserts today's reading, and §8 said the tests run unchanged | Rewritten by name; `pink_noise` gets a stated way out | §8, Step 2.2 |
| 2 | A shared clip's undo events reach the next case through the 10 s flush, and `flushAnalytics` needs `page.clock` | `sendQueuedAnalytics`, by `visibilitychange`, before `forget()` | §6.3, Step 1.1 |
| 3 | §6.3 named `data-testid="undo"`, which does not exist | By role and `messages.editor.undo` | §6.3, Step 1.1 |
| 4 | D-218 said `check-hosts.mjs` already fails on the hook's name; `DEV_MARKERS` is an explicit list | `devFailEncoder` joins it | D-218, Step 4.5 |
| 5 | §13 assumed `sourceHashPrefix` is at hand; it is in the clip's record | `clipsRepo.get`; a failed read means no cache | §13, Step 6.5 |
| 6 | §12.2's "cached file's bytes" had no reader | `heldBytes()` | §11.1, Step 6.1 |
| 7 | The answered set in `answer-post-export.ts` needed a new use-case edge from `start-export.ts` | The set lives in `export-store.ts` | D-234, Step 6.3 |
| 8 | `fakeExport` reaches `rendering` only; the done capture needs a hook | `fakeExportDone` | §4, Step 7.5 |
| 9 | Check 11 could print a source's tag value, and `v3testclip.mp4` carries a location | A `FAIL 11` names the key only | §7.5, Step 2.4 |
| 10 | D-195 said one document; the founder asked on 2026-10-31 for a guide and prompts file | D-195 rewritten; §22 is a summary table | D-195, §22 |
| 11 | The -11 LUFS is inferred, and after the fix it could not be read any more | Step 2.1 reads a real export before the fix | Step 2.1 |
| 12 | `pnpm test` needs Docker running, and the bench needs it closed | Docker on for Step 8.4's Rust run; closed for the bench | Steps 8.1, 8.4 |

### (c) Work placed differently from §5

| Item | §5 | This guide | Reason |
|---|---|---|---|
| `OFFCUT_KEEP_EXPORTS` and the "before" reading | P130, step 5 | First step of Phase 2 | It must run before the fix |
| `watchExport` moved to helpers | Not mentioned | Phase 5 | Two specs read the bar |
| `fakeExportDone` | Not mentioned | Phase 7 | The capture needs it |
| `check-hosts.mjs` marker | Not mentioned | Phase 4, with the fault it guards | A guard lands with its subject |
