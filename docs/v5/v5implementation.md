# v5implementation.md - Offcut V5 "Export that an independent tool accepts": implementation plan

**Implements:** `buildplan.md` section 7 (V5), which is step M1.6 of `technicalspec.md` (TS) §28, **together with the seams S-8 and S-9 of TS §40.9** and the development-speed rules the founder approved on 2026-10-31. Product references are to `product.md` (PS). "BP §n" is a section of `buildplan.md`; "V4 §n" of `docs/v4/v4implementation.md`; "V4 CL" is `docs/v4/v4changelog.md`; "V5 CL" is `docs/v5/v5changelog.md`. Decisions D-1 to D-193 are V1's to V4's; this document continues at D-194.

**Reader.** Whoever writes the V5 code. Everything needed to write a V5 file is here or in the TS section it names. `docs/v5/v5buildguide.md` gives the order of work, the commands and the milestones; `docs/v5/coding-promptsv5.md` gives the prompts an agent runs. This file gives the contracts and the decisions, and wins where the three disagree. V4 as built is the truth this plan builds on: where `v4implementation.md` and its §24.3 disagree, §24.3 wins. This document was written after reading the code at `0a8aa80`; every "today" below is a fact of that commit. Each decision the specs leave open is marked **(V5 decision)**. A change to something V4 froze (V4 §22.1) is marked **REOPENED CONTRACT**, with the cost on both sides. A number no experiment has measured is marked **(assumption)**.

**The one-sentence goal.** At the end of V5, a file Offcut exports passes all 11 checks of `verify_mp4.py`, a program that shares no code with the product: exactly as long as the map says, in sync at both ends, at the promised loudness and bitrate, with the watermark only where the verified token asks for it and with no metadata from the source; the same edit exported twice comes from the render cache at once; and the person sees what was changed and is asked one question.

**What you can demo (BP §7).** Export the long-pause fixture: 20 seconds in, 20 seconds out, the pause intact, audio and video in sync on both sides. Export the same edit twice: the second is served from the cache instantly.

**The second goal, which decides how V5 is built: speed.** V4 ran the full media suite 22 times (about 8 hours) and used 49 CI runs. The founder asked that V5 not repeat that, and approved tiered gates, a shared processed clip in `review-edit`, a docs-only CI skip, a compact document set and a five-run bench (section 6). Nothing that protects a release is removed: `pnpm check`, Rust tests, Vitest, goldens on lavapipe, the Windows `@no-gpu` job on code changes, one full media run before the merge and CI before the merge all stay.

**The third goal: V6 to V6.7 stay additive.** V5 is the last version that touches the export's shape. V6.2's cuts, V6.3's music, V6.6's end card and V6.7's brand kit must each reach the export by changing data. Section 23 lists this file by file. Nothing is reserved beyond S-8 and S-9 (TS §40.9).

---

## 0. Contents

1. Scope: what V5 is and is not
1A. Preconditions: what V5 starts from
2. Decisions this document makes (D-194 to D-239)
3. Tools, dependencies, environment variables
4. V5 file tree: what is created and changed
5. Build order: phases, prompts, time
6. How V5 is tested: the speed rules
7. The verifier: checks 7 to 11
8. Loudness: the stereo fix
9. The muxer: `ctts` and the overflow guard
10. The export loop: S-8 and the ladder restart
11. The render cache: S-9
12. The gate: blockers and the export machine
13. `start-export.ts`, step by step
14. `ChangeSummary` and "Would you post this?"
15. Analytics
16. Worker protocol changes
17. Tests, case by case
18. Fixtures, snapshots, goldens
19. CI, scripts, hosting
20. Measurements
21. Risks, each with its way out
22. The eight prompts (129 to 136), in brief
23. Hand-over: V6 and V6.1 to V6.7
24. Exit checklist
25. Spec issues found, decisions to copy back
Appendix A. Per-file specifications in brief
Appendix B. Invariants V5 touches, and where each is enforced
Appendix C. Named constants V5 introduces
Appendix D. Size estimate

---

## 1. Scope

### 1.1 In V5

| Area | What works at the end of V5 |
|---|---|
| Verifier | Checks 7 to 11 of TS §27.2 exist and run on every export a test makes: loudness, A/V sync and timeline against the source, watermark, bitrate, metadata. `verify/watermark_mask_720.png` is committed. A self-test proves that each check passes a good file and fails a bad one, in CI, without a browser (D-209) |
| Loudness | An exported file measures -14 LUFS as a stereo file. Today it would measure about -11 LUFS: the chain measures a mono programme and the export doubles it to two channels (section 8) |
| Muxer | `ctts` for frames that arrive out of presentation order, the video `elst` that goes with it, a test of the `moov` overflow guard (TS §21.3, §21.4) |
| Export over program time (S-8) | The loop renders frame `n` at `FrameIdx(n).to_out_ms()`, takes the source frame at `scene.source_at(t)`, gets its frame count from the scene (`out_duration`), and encodes `prog48`, read from a new `prog48.f32` |
| Encoder failure | An encoder that fails mid-export restarts from frame 0 on the next ladder entry, with the partial output deleted; an audio encoder failure retries once (TS §11, §21.4) |
| Render cache (S-9) | The key covers the source hash prefix, transcript, events, edit, plan, time map, theme, music id, the **verified** profile kind and the renderer version. A hit re-downloads the stored file under its original `ExportId`, with no render and no new receipt |
| Gate | `B_STORAGE_LOW` is evaluated in `blockers.ts`; `export-machine.ts` uses `cache_hit`; a blocked export returns to `idle` and says why |
| Summary and question (J11) | `ChangeSummary` shows only the changes that were made; `PostExportQuestion` asks "Would you post this as is?" and sends `post_export_answer` |
| Analytics | `export_done` carries the edit's real values (`word_edits`, `events_kept`, `events_disabled`, `style`, `crop_adjusted`, `from_cache`) instead of today's placeholders; `post_export_answer` starts (BP Appendix C) |
| Tests | `export-creator.spec.ts` with all 11 checks, `timeline-preserved.spec.ts` on both pause fixtures, `start-export.test.ts`, `export-machine.test.ts`, `blockers.test.ts`, `export-loop.test.ts`, the verifier self-test, the muxer's reorder and overflow tests |
| Speed | `review-edit.spec.ts` shares one processed clip across 13 of its 16 cases; docs-only pull requests skip the heavy jobs; the bench takes a run count; known issues 33 and 36 get their one-line fixes |
| Measurements | E-4 re-read on D1, `detect_scene` re-read on a quiet D1 (V4 CL known issue 37), the verifier's thresholds recorded in `docs/v5/experiments.md` and TS §39.3 item 14 |

### 1.2 Not in V5

| Left out | Stand-in in V5 | Arrives |
|---|---|---|
| Sign-in, the real token, the free counter, receipts, checkout | `entitlement-repo` and tests' minted tokens. The deployed V5 previews and answers "Exporting needs an account" (BP §7.2) | V6 |
| `export-free.spec.ts`, the free plan's flow end to end | A Free-token export inside `export-creator.spec.ts`, checked with the verifier's `free` profile | V6 |
| Any cut, any non-identity map | The loop reads the map; `set_scene` still refuses a map that is not the identity, and `TimeMap::from_segments` does not exist | V6.2 |
| Music | `prog48` is `out48` (identity, `program_identity.rs`); `musicId` in the cache key is `null` | V6.3 |
| Eviction, `quota.ts`, several kept exports | One export kept per clip, as today (D-40, D-225) | V7 |
| Resume after reload, `.persistent` copy | Retry starts over | V7 |
| The network-privacy test | V3's guards; V5 adds no request | V8 |
| A new design review | One founder look at the new state's capture (D-235) | - |

Nothing from TS §36 is built. V5 adds no route, no server code and no dependency in the product.

### 1.3 Journeys and use-case steps served

| Flow | Served in V5 | Missing until |
|---|---|---|
| J10 Render + export | Full: progress, cancel, ladder restart, verified output | - |
| J11 Download + summary | Full for V5's rows: voice, emphasized words, visual moments. The rows for cuts, music and chapters arrive with those features | V6.2, V6.3, V6.6 |
| J9 Export click | The gate's first half (running export, clip ready, storage) | V6 (sign-in, limits) |
| C-7 Gate | `B_EXPORT_IN_PROGRESS`, `B_PIPELINE_NOT_READY`, `B_STORAGE_LOW` | V6 (`B_NOT_SIGNED_IN`, `B_ENTITLEMENT_EXPIRED`, `B_NO_FREE_EXPORTS`) |
| C-9 Render | Full | - |
| C-10 Deliver | Full except the receipt | V6 |

---

## 1A. Preconditions: what V5 starts from

Do not start Prompt 129 until every box is ticked.

- [ ] 1. **V4 is closed and tagged.** The founder said on 2026-10-31 that V4 is closed. **When this plan was written the tag was not there:** `git ls-remote --tags origin` lists `v2` and `v3` only, `git tag` likewise; `v4implementation.md` still has 35 unticked boxes; `v4changelog.md` ends at Entry 39. Check: `git ls-remote --tags origin v4` prints a line. The closing entry and the boxes are the agent's (Prompt 128) once the founder has pushed the tag, and they go on `v4-build` or, since V4 is merged, in the first V5 commit (D-194).
- [x] 2. **V4 is merged and deployed.** `origin/main` is `0a8aa80`, the merge of PR 7. Its run 40889889605 was green in all four jobs. `healthz` answered `0a8aa80` on 2026-10-30.
- [x] 3. **The branch exists.** `v5-build`, from `origin/main` at or after `0a8aa80`, pushed to `origin` (D-194). This plan, `v5buildguide.md`, `coding-promptsv5.md` and `v5changelog.md` are its first commit.
- [ ] 4. **A draft pull request `v5-build` → `main` is open.** `ci.yml` runs on `pull_request`, so a pushed branch without one has no CI. As in V4, the founder opens it (V4 CL item 6).
- [x] 5. **The baseline counts are read on D1** and written into Entry 1 of `v5changelog.md`. V4's final gate: Rust 646 passed, 0 failed, 4 ignored; Vitest 175; `pnpm e2e` 12; media 77 passed + 1 skip (D-78). No count may fall during V5, except the cases this plan rewrites by name (D-201).
- [x] 6. **The verifier's tools are on D1.** `py -3.13 -c "import numpy"` reads 2.5.3; `ffmpeg` is 9.0.2. **Plain `python` on D1 is 3.9.13 and has no NumPy.** From Prompt 130 every command that runs the verifier puts Python 3.13 first on `PATH` (section 3.1).
- [x] 7. **The fixtures exist.** `fixtures/.gen/speech/` holds both pause fixtures. White frames: `speech_20s_long_pause.mp4` at frames 60 and 450, `speech_30s_sparse.mp4` at 15 and 885, `speech_60s_portrait.mp4` at 150 and 1650 (`gen_fixtures.sh`).
- [x] 8. **The V4 hand-over markers are where they should be.** `start-export.ts` lines 161 and 305, `export-loop.ts` line 352, `blockers.ts` line 46, `mux_roundtrip.rs` line 439, `verify_mp4.py` (`NOT_IMPLEMENTED`). Check: `git grep -nE "\bV5\b" -- web/src crates verify`.
- [ ] 9. **The D1 machine is quiet for the bench only.** Docker Desktop is closed at Prompt 136 (V4 CL known issue 37). Nothing else in V5 needs it.

**Carried over from V4.** Known issue 30 and open items 1 to 4 stay with V6.1 and the founder. Known issues 33, 36 and 37 are V5's (D-236).

---

## 2. Decisions this document makes

Numbering continues from V4's D-193. Each is the smallest choice that gets V5's gate passed and keeps V6.x additive. All are **(V5 decision)** unless they say otherwise.

### 2.1 Process and speed

| # | Decision | Why |
|---|---|---|
| D-194 | **All of V5 is built on `v5-build`**, from `origin/main`. Commits are titled `V5 P<nn>: <title>`. The agent commits there and pushes there. Opening and merging the pull request, pushing to `main` and tagging are the founder's | D-115 and the founder's standing rule. A push to `main` deploys |
| D-195 | **Three documents, as in V4, with compact records.** This plan holds the decisions and the contracts. `docs/v5/v5buildguide.md` holds the phases, steps and milestones, and `docs/v5/coding-promptsv5.md` the prompts (the founder asked for both on 2026-10-31, after first choosing one file). Section 22 stays as a one-table summary. `docs/v5/v5changelog.md` and `docs/v5/experiments.md` go beside them. Changelog entries stay short: what changed, what differs from this plan, the three counts, what is open | The founder's request. A build guide and prompts in V4's structure are what the V4 builds ran on; the speed rules (D-196 to D-201) still decide how much is run |
| D-196 | **Tiered gates** (section 6): each prompt runs what it can have broken, not everything | The founder's yes of 2026-10-31 |
| D-197 | **The full media suite runs once**, in Prompt 136, before the merge (D-72). A prompt that fails to pass a targeted run fixes it there; it does not widen to a full run. A follow-up fix after the full run re-runs the affected spec, not the suite, unless it touches the pipeline, a worker or the renderer | V4's 22 runs took 8 h |
| D-198 | **Three pushes before the merge:** at the end of Phases A, C and D (section 5). The agent does not wait on CI after the first two, reads the result when the next prompt starts, and fixes a red run before going further. It waits on the last | V4's waits cost about 25-35 min each. The risk: a CI-only failure surfaces later. A phase is short enough to afford that |
| D-199 | **A pull request that changes only documents skips `ci` and `e2e-media`.** A small `changes` job decides, and the secret scan moves into it, so documents are still scanned. A push to `main` always runs everything | A skipped job counts as passed; deploys are unaffected. See section 19.1 |
| D-200 | **The bench takes `BENCH_RUNS`**, default 10 as before. V5's reading uses 5, one warm-up plus five runs, with Docker closed | The founder's yes of 2026-10-31. A median of five is still meaningful; about 14 min instead of about 27 |
| D-201 | **`review-edit.spec.ts` shares one processed reference clip** across cases 1-4, 7, 9-16; cases 5, 6 and 8 keep their own (section 6.3). The case names and what each asserts do not change | Twelve fewer clip loads, about 12-15 min of each full run, and it compounds as V6.x adds review cases |
| D-202 | **D-69, D-74 and D-114 hold.** Every gate that names R1 is read on D1 and labelled `d1`; thresholds do not change | The founder's standing decision |

### 2.2 The verifier

| # | Decision | Why |
|---|---|---|
| D-203 | **The verifier imports NumPy at the top and exits 2 with `run pip install -r verify/requirements.txt` when it cannot.** It stays one file; no other library is added | TS §27.2. A check that silently cannot run must not look like a pass |
| D-204 | **The product is fixed, not the check:** the chain measures its mono programme as `Channel::DualMono` (section 8) | PS §20.1 promises -14 LUFS in the file. Measuring a downmix in the verifier would make the check pass and leave every export 3 LU too loud |
| D-205 | **Check 8** compares 10 ms RMS envelopes of the first and the last 5 s, searching lags within ±1 s, and white-flash positions over the whole file, when the source has any (section 7.2). A window with no sound in the source is reported, not failed | TS §27.2 fixes the windows and limits, not the method |
| D-206 | **Check 9** correlates the mean of 15 evenly spaced frames, scaled to 720x1280, with the mask inside the mask's bounding box. The mask is a full 720x1280 grey PNG drawn by a one-time tool from the font file, not by the renderer (section 7.3) | TS §5: "watermark template drawn independently of the renderer" |
| D-207 | **Check 10** reads the video stream's packet sizes over its duration: at most 12 Mb/s for `creator`, 6 Mb/s for `free`, values the verifier holds itself | TS §27.2: 1.5x the target. The verifier imports nothing from the product |
| D-208 | **Check 11** moves `metadataProblems` (D-100) from `export-creator.spec.ts` into the verifier. It compares with `--source` when given; without it the source-derived rule is reported as not read | One place for the rule; every export gets it |
| D-209 | **A self-test, `verify/test_verify_mp4.py`**, makes a good file and one bad file per failure with `ffmpeg`, and runs in `ci.yml`'s Linux job and on D1 | The hosted runner cannot make a real export, so without this CI never runs checks 7-11. It also tunes the checks in seconds, with no browser |
| D-210 | **The test passes the expected duration on the command line** (`--expected-duration-ms`), computed by the test, and `--source` separately. S-8's "writes the expected duration beside the export" is met by that argument; no sidecar file is written. Copy back to TS §40.9 | Same independence, one less file. From V6.2 the test computes it from the map and the two values differ |

### 2.3 The muxer

| # | Decision | Why |
|---|---|---|
| D-211 | **`ctts` version 0, plus a video `elst` that starts at the largest offset**, written only when a frame arrived out of order. Decode times stay `n x 1000`; `add_video_sample`'s `frame` means "presentation index" and its signature does not change | Version 0 with an edit list is what players read most widely; version 1 would need a brand change. A file with no reordering stays byte-identical to today's |
| D-212 | **A new fixture `ok_bframes.mp4`** (libx264, `-bf 2`; every fixture today has `-bf 0`), read by a new test that re-muxes its real samples and asks `ffprobe` and `ffmpeg` about the result | A payload of random bytes proves the tables only. Real reordering cannot be provoked on D1's encoder, which has never reordered |

### 2.4 The export

| # | Decision | Why |
|---|---|---|
| D-213 | **`run-pipeline.ts` writes `clips/<id>/prog48.f32` beside `out48.f32`, and the export reads `prog48.f32`.** `out48.f32` stays | V4 §22.2's row. V6.2 and V6.3 rebuild `prog48` from `out48` when a cut is restored or music changes, so both are needed. Reading the store's in-memory copy would break at V7's restore |
| D-214 | **REOPENED CONTRACT `RenderWorkerApi.exportClip`: the field `out48` becomes `prog48`.** Protocol, page and worker change in one commit. V4 §22.2's protocol row says "-", which missed this. Cost: one rename on both sides | A field called `out48` that holds `prog48` would mislead V6.2 |
| D-215 | **The loop reads `session.source_at(t)` and asks the source for `recorded(pos)`**, as `preview-job.ts` already does; `recorded` comes from `preview-loop.ts`. A `null` position is `E_INTERNAL` `NoScene` | S-8. `At` and `Held` share one helper, so V6.6's end card needs nothing here |
| D-216 | **Ladder restart:** `runExport` keeps today's body as one attempt. On an `E_ENCODE_VIDEO` failure it truncates the sink, reports progress 0, and starts a new attempt on the next supported entry. No entry left: the last failure is thrown. Sink, cancel, GPU and mux failures never restart | TS §21.4, §11 |
| D-217 | **`E_ENCODE_AUDIO` restarts once with the same video entry.** A second failure is thrown | TS §11's row: "Retry once" |
| D-218 | **A new dev fault, `failEncoderAt(frame, times)`**, as V4's faults are: a one-way message handled in `rpc-worker.ts`, read by the loop through `takeEncoderFault(n)`, folded away in production builds by `__OFFCUT_DEV_FAULTS__`. `devFailEncoder` joins `DEV_MARKERS` in `check-hosts.mjs`, whose list is explicit: without it nothing proves the worker's half is gone from `web/dist` | The only way to drive a real restart in a browser |
| D-219 | **`web/src/workers/render/export-loop.test.ts`**, with fake WebCodecs, covers the frame count, `source_at` (at, held and a jump), audio padding, every restart path and cancel | Takes seconds. It proves the V6.2 seam, which no browser test can reach while `set_scene` refuses other maps |

### 2.5 The render cache

| # | Decision | Why |
|---|---|---|
| D-220 | **The profile kind in the key comes from the token the worker verified.** REOPENED CONTRACT: `RenderWorkerApi` gains `exportProfile({ entitlementToken }) → ExportProfile`. Cost: one method and one handler; the handler already exists as `w.exportProfile` | INV-9. With the unverified `planOf`, an expired Creator token would hit a stored 1080p file with no watermark, and a forged one too |
| D-221 | **The preview locks when the gate starts, not after it** (`lockForExport` before `exportProfile`). A hit unlocks at once | The render worker takes one job at a time; a playing preview would refuse the call. TS §12.4 says locked "in `rendering`"; a lock a few milliseconds earlier is invisible. Copy back |
| D-222 | **The key is SHA-256 by `crypto.subtle` over a canonical JSON of the inputs**, sorted by key at every depth, in `render-cache-repo.ts`. TS §21.4 says "through `hash_api`"; `Sha256Stream` lives in the core bundle, which the main thread does not load. Copy back | `hashPrefix` already hashes this way on the main thread. Sorting is required: `word_edits` holds its keys in the order they were edited, and the same edit reached by another route must give the same key |
| D-223 | **`RENDERER_VERSION` is computed, not written by hand:** `vite.config.ts` hashes `offcut_render_bg.wasm`, `export-loop.ts` and `encoders.ts` and defines `__RENDERER_VERSION__` (16 hex). Where no bundler ran it reads `"test"`. TS names the constant but the code has none | A hand-bumped number is forgotten once, and then a device serves an old look forever. A needless re-render after a deploy costs under a minute |
| D-224 | **`theme` is `"default"` and `musicId` is `null`**, each a named key input. `Theme` is a function of the style and the renderer in V5, both already in the key | S-9 wants them in the key. V6.7 puts the kit's hash in `theme`, V6.3 the music's id in `musicId`; neither changes the key's shape |
| D-225 | **One entry per clip.** A miss deletes the row, then the file, then renders (TS §23.3's write order). Today's sweep of `exports/` stays. A hit is therefore "the same edit as the last export". Undoing back to an earlier export misses until V7 | TS §23.3: "exports other than the newest per clip" go. At up to 91.8 MB a file, keeping more is V7's `quota.ensureFree`. The existing storage cases stay true |
| D-226 | **`RenderCacheRecord` gains `summary: ChangeSummary`.** `DB_VERSION` stays 1: no build has written a row | A hit skips the worker, so the summary must come from the row |
| D-227 | **A hit:** gating → saving → done, the file delivered under the name of today's minute, its original `ExportId` in the store; `export_done` with `from_cache: true`; no `export_started`, no `stage_timing`, no receipt (V6) | TS §9 C-9, §21.4. `export_started` follows the lookup in C-9 |
| D-228 | **`importClip`'s step 4 also clears `renderCache`.** Without it, dropping the same file again would hit the stored export | TS §23.3: deleting a clip removes its rows. The bench needs it too: its runs re-drop one clip, and a hit has no `render_encode` to read |

### 2.6 The gate

| # | Decision | Why |
|---|---|---|
| D-229 | **`forExport(room?)`**: with `{ freeBytes, neededBytes }` it also evaluates `B_STORAGE_LOW` as `freeBytes < neededBytes x 1.2`. `ExportButton` calls it without, so the button is not disabled by a reading that may be stale; `startExport` calls it with, after a cache miss. No estimate available: not blocked | `blockers.ts` stays the one place. `navigator.storage.estimate()` is async and `forExport` is read during render |
| D-230 | **A failure during `gating`** (the token refused) goes `gating → rendering → failed`, through `clear` then `fail`, and is announced with `export_started` then `export_failed`, as today. The machine's table does not change | TS §12.2 has no `gating → failed`. The frozen cases in `export-creator.spec.ts` read exactly this |
| D-231 | **A blocked export says why in the `export-status` line** beside the button, from `messages.blockers`. The store holds the code until the next start. Not an `ErrorPanel`: nothing failed, and TS §9's "ErrorPanel, stop" is copied back | One action away from retrying, with no card to dismiss |

### 2.7 Interface and analytics

| # | Decision | Why |
|---|---|---|
| D-232 | **The summary and the question sit at the top of the side panel**, under the failed-export slot, while the export is `done`, with a close button. Nothing else on the page moves | The success state is already beside the Export button; the panel is where the failed card lives (D-190) |
| D-233 | **Rows are data:** `summaryRows(summary)` maps a `ChangeSummary` to rows, leaving out a zero. V6.2 adds `pauses_shortened` and `time_removed` as one row each | PS J11: "only changes that were made" |
| D-234 | **The question is asked once per `ExportId`.** A hit shares its export's id, so an answered export is not asked again. Three buttons: yes, small edits, no. Answering sends `post_export_answer` and shows a thank-you line. The set of answered ids lives in `export-store.ts`, so `start-export.ts` can read it without a new use-case edge in `eslint.config.js` | PS §20.5 |
| D-235 | **No third design review.** The panel uses existing tokens and primitives. `screens.spec.ts` gains one capture, the done state at both widths; the founder looks at it once at Prompt 136 | V5 adds one small state to a reviewed editor |
| D-236 | **Known issues.** 33: `modelDownloadEvent`'s poll takes `patient(5_000)`. 36: `openEditor` waits `patient(30_000)`. 37: re-read at Prompt 136. An issue is closed only by what it should show: no first-attempt failure of those cases in V5's CI runs | Both are one line; the runner's scale does not reach these waits today |
| D-237 | **`export_done`'s values are computed from the clip at the moment the export starts**, not when it ends; `events_kept` and `events_disabled` read `edit.event_overrides[id] ?? event.enabled` | Today's `kept` reads `event.enabled` alone, so a switched-off visual still counts as kept |
| D-238 | **`ExportProgress`'s time left starts again when the frames fall back to 0** after a restart | Otherwise it divides by a stale start |
| D-239 | **What V5 builds for V6.x, the complete list:** the loop on `source_at`; `prog48.f32`; the expected duration as an argument separate from the source; `theme` and `musicId` in the key; `summaryRows`; a new render distinguished from a cache hit in one place; the shared-clip helper. Nothing else | TS §40.9: nothing else is reserved |

---

## 3. Tools, dependencies, environment variables

### 3.1 Tools

No new tool in the product. For the verifier on D1, put Python 3.13 first:

```sh
export PATH="$(py -3.13 -c 'import os,sys;print(os.path.dirname(sys.executable))'):$PATH"
python --version   # 3.13.x
python -c "import numpy; print(numpy.__version__)"   # 2.5.3
```

The one-time mask tool (D-206) needs Pillow. It is not a dependency of the verifier and is not in `requirements.txt`: `py -3.13 -m pip install pillow` once, on D1, in Prompt 130.

### 3.2 New dependencies

None in `Cargo.toml` or `package.json`. `numpy==2.5.3` is already pinned in `verify/requirements.txt`. `Channel::DualMono` is in the `ebur128` crate the workspace already has.

### 3.3 Environment variables

| Name | Where | Purpose |
|---|---|---|
| `BENCH_RUNS` | `bench/device-bench.ts` | How many measured runs; default 10 (D-200) |
| `OFFCUT_KEEP_EXPORTS` | `helpers/fixtures.ts` | A directory `verifyMp4` copies each checked file to, so the verifier can be tuned on real exports offline (section 6.2). Never set in CI; outside the repository |

No production variable changes.

---

## 4. V5 file tree: what is created and changed

Files marked **new** do not exist today. TS §5 has listed most of them since v1.1 (`render-cache-repo.ts`, `ChangeSummary.tsx`, `PostExportQuestion.tsx`, `answer-post-export.ts`, the three unit-test files, `timeline-preserved.spec.ts`, the mask). Only **four** need a new row, in the commit that creates each, or `check-file-tree.mjs` fails: `verify/make_watermark_mask.py`, `verify/test_verify_mp4.py`, `web/src/workers/render/export-loop.test.ts`, `crates/offcut-mp4/tests/mux_reorder.rs`.

```text
crates/
  offcut-dsp/src/loudness.rs                     Channel::DualMono (section 8); its 1 kHz test rewritten by name
  offcut-dsp/tests/loudness_target.rs            a stereo reading beside the mono one
  offcut-mp4/src/mux.rs                          presentation index, reorder bookkeeping, video edit
  offcut-mp4/src/mux_boxes.rs                    ctts(); stbl() takes it
  offcut-mp4/tests/mux_roundtrip.rs              reorder and overflow cases; the "no ctts before V5" comment
  offcut-mp4/tests/mux_reorder.rs           new  real samples of ok_bframes.mp4 through the muxer, read by ffprobe
fixtures/
  gen_fixtures.sh, manifest.json                 ok_bframes.mp4 (26 fixtures)
verify/
  verify_mp4.py                                  checks 7-11
  watermark_mask_720.png                    new  the mask
  make_watermark_mask.py                    new  the one-time tool that drew it
  test_verify_mp4.py                        new  the self-test
  README.md, requirements.txt                    updated
web/src/
  copy/messages.ts                               B_STORAGE_LOW, export.summary.*, export.question.*
  persistence/opfs.ts                            clipProg48, writeProgram, readProgram, freeBytes
  persistence/render-cache-repo.ts          new  canonicalJson, cacheKey, lookup, put, remove, clear
  persistence/schema.ts                          RenderCacheRecord.summary
  state/blockers.ts                              forExport(room?)
  state/blockers.test.ts                    new
  state/export-store.ts                          cacheHit, blockedBy, summary, answered
  state/machines/export-machine.ts               comment only (the table is unchanged)
  state/machines/export-machine.test.ts     new
  usecases/start-export.ts                       the gate, the cache, real values (section 13)
  usecases/start-export.test.ts             new
  usecases/answer-post-export.ts            new
  usecases/run-pipeline.ts                       writes prog48.f32
  usecases/run-pipeline.test.ts                  its mock gains writeProgram
  usecases/import-clip.ts                        clears renderCache (D-228)
  usecases/dev-faults.ts                         failEncoderAt
  ui/dev-hooks.ts                                failEncoderAt, fakeExportDone
  ui/components/ChangeSummary.tsx           new
  ui/components/PostExportQuestion.tsx      new
  ui/components/ExportButton.tsx                 shows the blocker's words
  ui/components/ExportProgress.tsx               time left restarts (D-238)
  ui/layout/SidePanel.tsx                        mounts the outcome
  ui/styles/components.module.css                the outcome's rules
  workers/protocol.ts                            exportProfile; exportClip's prog48
  workers/rpc-worker.ts                          devFailEncoder, takeEncoderFault
  workers/render.worker.ts                       the exportProfile handler
  workers/render/export-job.ts                   prog48; sink passed on
  workers/render/export-loop.ts                  source_at, attempts, ladder
  workers/render/export-loop.test.ts        new
  workers/render/encoders.ts                     supportedConfigs (pickVideoConfig stays for the capability check)
  workers/render/opfs-sink.ts                    reset()
web/
  vite.config.ts                                 __RENDERER_VERSION__
  playwright.config.ts                           timeline-preserved.spec.ts in MEDIA_SUITES
web/tests-e2e/
  helpers/fixtures.ts                            verifyMp4(source), expectAllChecks, openShared, resetClip, watchExport (moved from export-creator.spec.ts), OFFCUT_KEEP_EXPORTS
  helpers/fake-api.ts                            forget()
  export-creator.spec.ts                         all 11 checks; cache, storage, summary, question cases
  timeline-preserved.spec.ts                new
  review-edit.spec.ts                            the shared clip
  model-download.spec.ts                         patient waits
  screens.spec.ts                                the done state
bench/device-bench.ts                            BENCH_RUNS
bench/results/d1-<date>-portrait.json       new
.github/workflows/ci.yml                         changes job, verifier self-test
.github/workflows/e2e-media.yml                  comment; nothing else
scripts/check-copy-codes.mjs                     B_STORAGE_LOW leaves COPY_PENDING
scripts/check-hosts.mjs                          devFailEncoder joins DEV_MARKERS
docs/
  technicalspec.md, buildplan.md                 tree rows now; copy-back at Prompt 136
  v5/v5implementation.md                    new  this file
  v5/v5buildguide.md                        new  phases, steps, milestones
  v5/coding-promptsv5.md                    new  the prompts
  v5/v5changelog.md                         new
  v5/experiments.md                         new  at Prompt 136
```

**Unchanged on purpose.** `server/`, `offcut-plan`, `offcut-scene`, `offcut-render`, `offcut-entitlement` (its `profile.rs` is already what BP §7 asks, and only now the cache relies on it), `audio.worker.ts`, the preview, the review use-cases, `export-machine.ts`'s table, `gen/` (no Rust type changes).

---

## 5. Build order: phases, prompts, time

Eight prompts, numbered on from V4's 128, in four phases. Each prompt is one commit (a fix to it is a second). `coding-promptsv5.md` has each prompt in full; `v5buildguide.md` numbers the same work as Phases 0 to 8.

| Phase | Prompt | Work | Browser time | Agent time |
|---|---|---|---|---|
| Setup | - | Branch, baseline, V4's closing entry | none | 30 min |
| **A. Speed** | 129 | Shared clip, gates, docs-only skip, known issues 33 and 36 | one `review-edit` run | 2-2.5 h |
| *push 1* | | CI: read at the start of 130 | | |
| **B. Correct file** | 130 | Verifier 7-11, mask, self-test, stereo fix | three exports | 2 h |
| | 131 | Muxer: `ctts`, overflow, `ok_bframes` | none | 1 h |
| | 132 | Program-time export, `prog48.f32`, ladder restart | one or two exports | 2 h |
| **C. Proof and cache** | 133 | `timeline-preserved`, `export-creator` with 11 checks | both new describes | 1.5 h |
| | 134 | Render cache, gate, three unit-test files | export cases only | 2 h |
| *push 2* | | CI: read at the start of 135 | | |
| **D. Finish** | 135 | Summary, question, real analytics | the new cases only | 1.5 h |
| | 136 | Bench, full media, copy-back, push 3, merge, deploy | bench + one full run | 2.5 h |

**Total about 15-16 hours of agent time over 2 to 3 working days, plus about an hour of the founder's** (reading CI's green, the merge, the deployed check, one look at a capture, the tag). All time figures are estimates, not measurements. The same work at V4's pace is estimated at about 30 hours. BP's slot for V5 is 9-11 Nov.

---

## 6. How V5 is tested: the speed rules

### 6.1 The tiers

| Tier | When | What runs | Typical time |
|---|---|---|---|
| 0 | While writing | The one test being written; `cargo test -p <crate> <name>`; `vitest run <file>`; `python verify/test_verify_mp4.py <Case>` | seconds |
| 1 | Before each commit | `pnpm check`; `cargo test -p` each crate touched; `vitest run` | 3-8 min |
| 2 | Before a commit that touches `web/` behaviour or a worker | Tier 1, plus a build, plus only the cases named in that prompt: `pnpm e2e:media <spec> --grep "<case>" --max-failures=1` | 3-15 min |
| 3 | End of a phase, before a push | `pnpm check && pnpm fixtures && pnpm test` (whole Rust and Vitest), `pnpm e2e` | about 15 min |
| 4 | Prompt 136 only | `pnpm e2e:media` in full, once, on a keyed build; then CI, waited for | 30-40 min + about 30 min |

Rules that go with them:

- A new or changed case runs alone first. A hang is never discovered through a suite.
- Long runs go in the background; only the tail and failures are read.
- **Build cost.** Only `web/src` changed: `pnpm --filter web exec vite build --mode test` (seconds). Rust changed: `pnpm build:e2e`. The keyed variable is in the environment, as in V4.
- **"Passes twice" is dropped,** except for a timing suite whose code changed. V5 changes no timing suite: `preview-sync.spec.ts` is untouched.
- `--repeat-each` on one case, not a whole suite, to measure flakiness.

### 6.2 The verifier is tuned offline

`OFFCUT_KEEP_EXPORTS=<dir>` makes `verifyMp4` copy each file it checks there. Three real exports are made once, in Prompt 130: Creator and Free of the reference clip, Creator of the long-pause fixture. Then `python verify/verify_mp4.py <dir>/x.mp4 --profile ... --source ...` runs in seconds as often as needed, with no browser and no rebuild. The directory stays outside the repository and is deleted when the prompt ends: an exported file is never part of a report (TS §22.5).

### 6.3 The shared clip in `review-edit.spec.ts`

Today each of the 16 cases opens its own page and processes a clip, about a minute each. After Prompt 129:

```ts
test.describe("the editor (review-edit), on one processed clip", () => {
  // Not "serial": a failed case must not stop the others. Playwright starts a
  // new worker after a failure, which runs beforeAll again: a new clip.
  let shared: { context: BrowserContext; page: Page; api: FakeApi };
  test.beforeAll(async ({ browser }, info) => { shared = await openShared(browser, info, referenceClip()); });
  test.afterAll(async () => { await shared.context.close(); });
  test.beforeEach(async () => { await resetClip(shared.page, shared.api); });
  // cases 1-4, 7, 9-16, with `page` and `api` read from `shared`
});
```

`resetClip(page, api)` in `helpers/fixtures.ts`:

1. Stops anything open: Escape in a word field; pauses a playing preview.
2. Presses Undo (`getByRole("button", { name: messages.editor.undo, exact: true })`; it has no test id) until it is disabled, waiting for `settled` after each. The history holds up to 50 steps and no case makes that many.
3. Seeks to 0, opens the Captions tab, moves the pointer off the page (a hovered toast does not time out) and waits for the toasts to leave.
4. **Sends what is queued, then forgets it.** Analytics flushes every 10 s, so the undo steps' `review_action`s would otherwise reach the next case, whose `toEqual(["word_edit"])` then fails. `page.clock` is not installed on this page, so `flushAnalytics` cannot be used. The page is told it is hidden: `visibilityState` is overridden to `hidden`, `visibilitychange` is dispatched (`analytics/client.ts` flushes on it), the override is removed, and 300 ms pass for the request to reach the route. Then `api.forget()` (new in `fake-api.ts`) empties `requests`, so `eventsOf` counts from here and the cases that read their `review_action`s are unchanged.
5. **Asserts the clean state**: Undo disabled, no error card, and the stored edit (`readIndexedDb(page, "edits", clipId)`, written by the update rule) equal to `clip-store`'s empty one. Reading the store rather than the Style tab starts no thumbnail job. An unclean state fails the case by name instead of confusing the next one.

| Cases | Clip | Why |
|---|---|---|
| 1-4, 7, 9-16 | Shared reference clip | They undo cleanly |
| 5 | Own: `speech_60s_landscape.mp4` | Another clip |
| 6 | Own: `speech_60s_portrait.mp4` | Another clip |
| 8 | Own: reference | It breaks a change on purpose (`failNextSetScene`) |

Redo stays enabled after a reset. Case 12 does not assume otherwise: read on 2026-10-31, it makes three new changes first, and a new change drops the redo branch. Case 14's 400 ms budget and case 16's scans are unaffected. Observers that cases 7 and 14 leave on the shared page do nothing to later cases. `preview-sync` and `cancel` keep their own loads: they measure the first frame and kill workers.

**What this buys.** Twelve fewer clip loads. The suite was 41.8-54.9 min in V4; expect about 30-40 min with V5's additions (estimate).

### 6.4 What a test tier does not replace

CI still runs on every code push. The Windows `@no-gpu` job stays: it caught `warmAudio` in V4. Goldens run on lavapipe in CI. The full suite runs before the merge.

---

## 7. The verifier: checks 7 to 11

`verify/verify_mp4.py`, today 233 lines, grows to about 550. `NOT_IMPLEMENTED` and its line go. `CHECKS` gains five functions; the printing and exit rules do not change. Lines that report a reading without judging it start with `INFO`, so that a person tuning thresholds sees numbers and a test reading `FAIL` is unaffected. `--source` becomes meaningful and stays optional, because a Free or Creator run without a fixture must still work: without it, check 8 prints `PASS 8` with `INFO 8: no --source, not measured`, and the source-derived half of check 11 likewise. **The tests always give `--source`.** `expectAllChecks` asserts the output does not contain `not measured`.

Audio and frames are read by piping `ffmpeg` into NumPy, with no temporary files:

```python
def pcm(path):       # mono float32 at 48 kHz, ffmpeg applying the file's edit lists
    run(["ffmpeg","-v","error","-nostdin","-i",path,"-map","0:a:0","-ac","1","-ar","48000","-f","f32le","-"])
def grey(path, filt, width, height):   # frames as bytes through a filter
    run(["ffmpeg","-v","error","-nostdin","-i",path,"-map","0:v:0","-vf",filt+",format=gray","-f","rawvideo","-"])
```

`run()` today decodes stdout as text; a `run_bytes()` beside it returns bytes.

### 7.1 Check 7, loudness

`ffmpeg -nostdin -i <file> -map 0:a:0 -af ebur128=peak=none -f null -`, reads the summary's last `I:` line from stderr (one line of parsing, tested in the self-test). Passes when `abs(I + 14) <= 1`. The stream is measured as it is, in stereo: this is what a platform measures.

`INFO 7: integrated -14.2 LUFS` is printed.

### 7.2 Check 8, sync and timeline (D-205)

Needs `--source`. Audio part:

1. Decode both to mono at 48 kHz. An envelope is the RMS over 480-sample hops (10 ms), linear, not in dB, so that speech dominates room tone, which the chain's denoiser removes.
2. Two windows on the **export's** timeline: its first 5,000 ms and its last 5,000 ms. A clip of 10 s or less uses one window, the whole clip.
3. For each window, normalized cross-correlation of the export's envelope with the source's at lags of ±100 hops (±1 s), where the source's samples are taken around the same place. Find the best lag; refine with a parabola through the peak and its two neighbours. Multiply by 10 ms.
4. Pass when `abs(offset) <= 20 ms` and the peak's correlation is at least `MIN_ENVELOPE_CORR = 0.5` **(assumption, read in Prompt 130)**. A low peak fails: the audio there is not the source's, which is what a removed pause looks like when the removal is longer than the search.
5. A window whose source envelope has no speech (peak RMS below -50 dBFS): `INFO 8: window at <t> s is silent in the source, not measured`. A clip silent at both ends cannot be judged at the ends; the corpus (V9) tells whether this needs more.

A shortened pause moves the last window by its length, so it fails even when check 5 has been given a matching expected duration. The self-test has this case.

Video part, over the whole file:

1. One mean brightness per frame: `-vf fps=30,scale=16:16:flags=area` for the source (it may be VFR) and `scale=16:16` for the export, as grey.
2. A frame is a flash when its mean is at least `FLASH_MIN = 200` and at least 80 above the median of the 30 frames around it **(assumption, read in Prompt 130)**.
3. Export and source must have the same number of flashes, each within one frame. The source has none (the reference clip): `INFO 8: no white frame in the source, video offset not measured`. Otherwise `INFO 8: 2 flashes compared, largest offset 0 frames`.

`expectAllChecks` takes `{ flashes: n }` and asserts that line for the pause fixtures.

### 7.3 Check 9, watermark (D-206)

**The mask.** `verify/make_watermark_mask.py`, run once, draws a 720x1280 8-bit grey PNG: background 128, the text "Made with Offcut" in `crates/offcut-scene/assets/fonts/Inter-Variable.ttf` at weight 700 and 24 px (36 lp x 2/3), fill 255, outline 0 at 2 px (3 lp x 2/3), its left at 40 px, its baseline at about 993 px (`SAFE_AREA` bottom 1,500 lp minus 10.2 lp, x 2/3). It uses Pillow (`set_variation_by_axes`) and shares no code with the renderer; it reads the font as data. The numbers in its head come from `watermark.rs` and `safe_area.rs` as built (V4 §24.3's D-143). The founder moved that band in V4 (V4 CL Entry 17), so a future move re-runs the tool, and the self-test's watermark case then fails until it has been.

**The check.**

1. The mask's bounding box: the pixels not 128, plus a margin of 4 px.
2. 15 frames evenly spaced by index, scaled to 720x1280 (`select`, `scale=720:1280:flags=area`), skipping flashes. Their mean, inside the box. Averaging keeps what stays still.
3. Subtract a 9x9 box-blur from both the mean and the mask (high-pass): a talking head's static, low-frequency background must not count.
4. Pearson correlation of the two over the box.
5. `free`: at least 0.5. `creator`: at most 0.2. **(assumption, TS §39.3 item 14; Prompt 130 reads them.)**

`INFO 9: watermark correlation 0.78`.

**If the two readings are not well apart** (free under 0.6, or creator over 0.1, on the three real exports): first widen the window (31 frames), then look at the mask's alignment. Moving a threshold is allowed, and recorded in TS §39.3, only while free and creator stay at least 0.3 apart. The threshold never moves to make one file pass.

### 7.4 Check 10, bitrate

`ffprobe -select_streams v:0 -show_entries packet=size -of csv=p=0`, summed, x 8, divided by the video stream's duration. At most 1.5 x the target: 12,000,000 b/s for `creator`, 6,000,000 for `free`, constants in the verifier. `INFO 10: 3.1 Mb/s`. Over a whole clip this is the average; a peak is not judged. D-207.

### 7.5 Check 11, metadata

The rule of `metadataProblems` (D-100), moved here and removed from the spec:

- No format or stream tag whose key contains `location` or `iso6709`.
- No `title`, `comment`, `description`, `artist`, `album` tag with a non-empty value.
- `creation_time` absent or the epoch.
- With `--source`: no `title`, `comment`, `encoder` or `creation_time` tag carrying the source's value.
- Neither the bytes of `©xyz` nor `ISO6709` anywhere in the file. The file is read in 1 MiB pieces with an overlap of 7 bytes: a 90 s export is up to 91.8 MB.
- A `FAIL 11` names the tag's key, never its value: `testclips/v3testclip.mp4` carries a location that must not reach a report or a log (D-99, SAB rule 9).

The `no metadata` case of `export-creator.spec.ts` stays, on `testclips/v3testclip.mp4` on D1, and now reads `PASS 11`.

### 7.6 The self-test (D-209)

`verify/test_verify_mp4.py`, `unittest`, standard library plus NumPy, about 180 lines. It takes `speech_20s_long_pause.mp4` from `pnpm fixtures`, in one temporary directory, which it deletes:

**Good file.** Scale and crop to 720x1280, `ultrafast`, AAC-LC stereo, `-map_metadata -1`, `+faststart`, and a `volume` filter computed by measuring first, so that it reads -14 LUFS. A second good file burns in the mask with `blend=all_expr='if(eq(B,128),A,0.6*B+0.4*A)'` for the `free` profile. The creator-size good file scales to 1080x1920.

| Bad file (a copy with one change) | Must print | Encode? |
|---|---|---|
| Volume -6 dB | `FAIL 7` | audio |
| Audio delayed 100 ms (`adelay`) | `FAIL 8` | audio |
| 1 s removed from the middle of the pause, expected duration also 19 s | `FAIL 8` | both |
| The white frame moved by two frames (video `setpts` in that stretch) | `FAIL 8` | video |
| Free file with no watermark | `FAIL 9` | none: use the plain good file |
| Creator file with the watermark | `FAIL 9` | video |
| `-b:v 20M` with `-x264-params` filler | `FAIL 10` | video |
| `-metadata title=x` | `FAIL 11` | copy |
| `-metadata location=+27.1+78.0/` | `FAIL 11` | copy |
| Source's `creation_time` copied | `FAIL 11` | copy |
| NumPy hidden (`PYTHONPATH` to a stub that raises) | exit 2 | none |

Each case also asserts the other checks still print `PASS`, so a bad file fails for its own reason. Budget: at most 3 minutes on the runner. Most cases copy streams or encode audio only.

---

## 8. Loudness: the stereo fix

**The finding.** `crates/offcut-dsp/src/loudness.rs` measures the mono programme as channel `Center`, and the chain brings it to `TARGET_LOUDNESS = -14`. `export-loop.ts` then writes the same samples into both channels. BS.1770 sums the channels' energy, so a mono signal duplicated to two channels reads about 3 LU higher. Measured on 2026-10-31, on a fixture's PCM, with the same `ffmpeg ebur128` check 7 uses:

| `speech_20s_long_pause.mp4.pcm48.f32` | Integrated |
|---|---|
| As one channel | -23.2 LUFS |
| The same samples in two channels | -20.2 LUFS |

So an export today should read about -11 LUFS as a stereo file, and **check 7 would fail on every one**. That figure is inferred from this reading, not taken from an export; Prompt 130 reads real files. `loudness_target.rs` cannot see this: it reads the chain with the chain's own meter. The earlier "-14.30 LUFS" readings in V3's experiments are mono readings of `out48`.

**The fix (D-204).** One line:

```rust
meter.set_channel(0, Channel::DualMono).ok()?;   // was Channel::Center
```

`DualMono` is `ebur128`'s name for a mono signal played on two loudspeakers, and it adds the 3 dB. The chain then aims so that the stereo file reads -14. Everything downstream is unchanged: the gain is computed from `integrated()`, `program.rs` reports the same function, and `out48.len()` does not change (INV-10). The recognizer is not affected: it hears `pcm16` from the media worker, not the chain's output, so no transcript moves. Prosody is z-scored per clip, so a constant gain does not move it either.

**What this moves.** `out48` and `prog48` get about 3 dB quieter. So does the preview, which then matches the export. The limiter works less, which is good. No golden holds audio. V6.3's mix loudness uses the same function, so it is right from its start.

**The tests.**
- `loudness_target.rs` gains one reading: the output, as two channels, through a `Stereo` meter pair in the test, within ±1 LU of -14.
- A test in `loudness.rs` pins the difference: the same signal reads 3.01 ± 0.05 LU higher under `DualMono` than under `Center`.
- **`a_1khz_sine_of_peak_minus_20_reads_minus_23` in `loudness.rs` breaks by design** and is rewritten by name: `a_1khz_sine_of_peak_minus_20_reads_minus_20`, the same signal within 0.1 LU of -20. The doc comment of `integrated` follows.
- **`pink_noise_reaches_the_target` may stop being capped.** It asserts `TARGET - given > 24`, and `given` now reads about 3 LU higher. If that assertion fires, lower the input from -40 to -44 dBFS RMS and correct its comment; what it proves (the capped gain) stays. Nothing else in the file should change, and any other failure is read before anything is edited.
- `chain_no_clicks.rs`, `duration_preserved.rs`, `program_identity.rs` run unchanged.
- Independently: check 7 on real exports.

The fix is a product change, and it is recorded as one in the changelog and at TS §18.2.

---

## 9. The muxer: `ctts` and the overflow guard

### 9.1 Reordering (D-211)

WebCodecs gives chunks in decode order, each with a presentation timestamp, and gives no decode time. On D1 they have never been out of order; a machine with another encoder may have B-frames, and today it gets `E_MUX` `OutOfOrder`.

`Mp4Muxer` keeps, beside what it has:

```rust
/// Presentation index of each sample, in the order written.
presented: Vec<u32>,
/// Which presentation indexes have arrived.
seen: Vec<bool>,
```

`add_video_sample(data, frame, is_keyframe)`:

| Today | After |
|---|---|
| `frame` must equal the sample number | `frame` is the presentation index |
| First sample must be a keyframe | The same, and it must be frame 0 |
| Anything else: `OutOfOrder` | A frame already seen, or more than `MAX_REORDER = 16` beyond the sample number (H.264's DPB limit): `OutOfOrder` |
| `sync` holds sample numbers | The same: sample numbers, in decode order |

`finalize` first checks that every index from 0 to the count has arrived; a gap is `MuxError::OutOfOrder`.

`moov()`:

```text
for sample i (0-based, decode order):
   dts_i  = i x 1000                  (ticks of 1/30,000 s; stts stays one run)
   pts_i  = presented[i] x 1000
   delay  = max over i of (dts_i - pts_i), at least 0
   offset_i = pts_i - dts_i + delay   (always >= 0: ctts version 0)
```

When every `offset_i` is 0, nothing changes: no `ctts`, no edit, **the file is what it is today**. Otherwise:

- `boxes::ctts(&runs)`: run-length `(count, offset)`, version 0, placed after `stts` in `stbl`. `stbl()` takes `ctts: Option<&[(u32,u32)]>`.
- The video `trak` gets `boxes::edts(video_duration, delay)`: presentation starts at the first frame. The function exists for audio priming.

`Writer.video` in `export-loop.ts` already computes `frame` from the chunk's timestamp and needs no change. Its comment about "out of turn" is updated.

### 9.2 Moov overflow

`MOOV_RESERVE = 256 KiB`. At 90 s the largest `moov` is about an eighth of it, plus `ctts` at worst 8 bytes x 2,700 = 21.6 kB, so it still fits. BP §7 asks for the guard to be tested:

- `ninety_seconds_fit_the_space_kept_for_moov` is repeated with reordered frames.
- `a_moov_over_the_reserve_is_refused`: 30,000 frames of one byte, written in turns with audio so each is a chunk of its own, returns `MuxError::MoovOverflow` from `finalize`, and nothing has been written over `mdat`.
- `a_reserve_with_under_8_bytes_left_is_refused`: the `(1..8)` branch, sizes found by search in the test.

### 9.3 Test `mux_reorder.rs` (D-212)

Reads `ok_bframes.mp4` with `Demuxer`, then writes its samples through `Mp4Muxer` in decode order with their presentation indexes and the fixture's `avcC`. Asserts:

1. The demuxer reads back the same presentation times.
2. If `ffprobe` is on the `PATH` (CI and D1): every frame-to-frame delta is 1/30 s, in the same way as check 3.
3. If `ffmpeg` is on the `PATH`: a full decode reports no error, and the frame count is right.
4. Audio is a dummy track from the same fixture.

A missing tool fails on CI and D1 alike: `CI` set or not, the fixtures already need them.

The line `// The six tables of the video track, and no ctts before V5` in `mux_roundtrip.rs` becomes `...and no ctts when nothing is reordered`. Its assertion stays.

---

## 10. The export loop: S-8 and the ladder restart

### 10.1 S-8 in the pipeline

`run-pipeline.ts`, step 13, after `assemble`:

```ts
const prog48 = owned(program.prog48);
run.stage = "storage";
await opfs.writeProgram(clipId, prog48);   // new: the audio the export encodes (TS §40.9 S-8, D-213)
await transcriptRepo.put(clipId, { transcript, prosody });
```

`opfs.writeAudio` stays for `out48` and is called before `assemble`, as today, because `assemble` takes `out48` by transfer. A write copies the samples and does not transfer the buffer, so `prog48` stays valid for `clipStore.ready`. `paths.clipProg48`, `writeProgram`, `readProgram` are `out48`'s three in the same shape. Disk: at most 2 x 17 MB for 90 s; the clip's directory holds both until the clip goes.

`start-export.ts` step 7:

```ts
const prog48 = await opfs.readProgram(clipId);
// ...
await pool.render.exportClip({ exportId, entitlementToken, prog48 }, { transfer: [prog48.buffer], onProgress });
```

`program_identity.rs` already proves that with the identity map the two files are equal, sample for sample.

### 10.2 The loop on program time

In `runExport`'s frame loop, in place of today's lines 349-357:

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

`total = a.session.frame_count()` is already `Scene::frame_count()`, which is `ceil(out_duration x 30 / 1000)`: that half of S-8 is true today. The `TimeMs` import goes.

**With the identity map nothing changes:** `to_src(t)` returns `At(t)` for every `t` under the duration, and the last frame's `t` is under it. `timeline-preserved.spec.ts` and the verifier prove the outcome.

**For V6.2.** A jump forward in the source: `frameAtBlocking` calls `#ask`, which calls `#frames.jumped`, which restarts at the keyframe for a jump beyond the decoder's horizon, and `#dropDue` handles a short one. `export-loop.test.ts` exercises the loop's half with a fake source. V6.2's `cuts_applied.spec.ts` proves the rest with real cuts. `Held` repeats a frame: `#take` returns `#shown` when it is still the latest.

### 10.3 The ladder

`encoders.ts` gains:

```ts
/** The ladder's entries the browser supports for this profile, in order: what an export may try (TS §21.2, §21.4). */
export async function* supportedConfigs(profile: ExportProfile): AsyncGenerator<VideoEncoderConfig> { ... }
```

`pickVideoConfig` becomes "the first of `supportedConfigs`, or `E_ENCODE_VIDEO`" and keeps its signature for the capability check.

`export-loop.ts`: today's `runExport` body becomes `attempt(a, config)`, unchanged except for its first line. The new `runExport`:

```ts
export async function runExport(a: ExportArgs): Promise<{ bytes: Bytes } | { cancelled: true }> {
  let lastFailure: unknown = new WorkerFailure("E_ENCODE_VIDEO", "NoSupportedConfig");
  let audioRetried = false;
  for await (const config of supportedConfigs(a.profile)) {
    for (;;) {
      try {
        return await attempt(a, config);
      } catch (thrown) {
        const code = thrown instanceof WorkerFailure ? thrown.code : undefined;
        if (code !== "E_ENCODE_VIDEO" && (code !== "E_ENCODE_AUDIO" || audioRetried)) {
          throw thrown;
        }
        lastFailure = thrown;
        a.sink.reset();              // the partial file goes (TS §21.4)
        a.onProgress(0, a.session.frame_count());
        if (code === "E_ENCODE_VIDEO") {
          break;                     // next entry of the ladder
        }
        audioRetried = true;         // the same entry once more (TS §11)
      }
    }
  }
  throw lastFailure;
}
```

What each attempt starts clean: the encoders, `Watch`, `Writer` and the muxer are made inside `attempt`, and its `finally` closes them. `liveFrames` balances in its `finally`. The source: `frameAtBlocking(0)` after being at frame `k` is a jump back, which `#ask` handles as a seek does. The scene and the canvas are unchanged.

`OpfsSink.reset()`: `handle.truncate(0)`, tagged as `writeAt` is. A sink error is never restarted.

`takeEncoderFault(n)` (D-218), in the frame loop before `venc.encode`: in a production build it is `false` and is folded away.

`ExportProgress.tsx` (D-238): `useTimeLeft` resets `first.current` when `done` falls below `first.current.done`. The bar falls to 0, which is honest: the work did start again. `exportStore.progress` already accepts it.

---

## 11. The render cache: S-9

### 11.1 `persistence/render-cache-repo.ts`

```ts
export type CacheInputs = {
  sourceHashPrefix: string;
  transcript: Transcript;
  events: readonly DetectedEvent[];
  edit: EditState;
  plan: EditPlan;
  timeMap: TimeMap;
  /** "default" until a brand kit (V6.7): a function of the style and the renderer, both in the key (D-224). */
  theme: string;
  /** `null` until music (V6.3). */
  musicId: string | null;
  /** From the token the worker verified, and from nowhere else (D-220, INV-9). */
  profileKind: ProfileKind;
  rendererVersion: string;
};

/** Sorted by key at every depth; `undefined` left out, as JSON does. */
export function canonicalJson(value: unknown): string;

/** Lower-case hex SHA-256 of `canonicalJson(inputs)`. */
export function cacheKey(inputs: CacheInputs): Promise<string>;

/** The row of `key`, if its file is there at its size. A row without its file is deleted and is a miss (TS §23.3). */
export function lookup(key: string): Promise<RenderCacheRecord | undefined>;

/** After the file is in place (TS §23.3's write order). A failure is swallowed: the cache is an optimization. */
export function put(key: string, record: RenderCacheRecord): Promise<void>;

/** The bytes of every row: a miss deletes them, so they count as free (TS §12.5, "after eviction"). 0 when the store cannot be read. */
export function heldBytes(): Promise<number>;

/** Every row: deleted before a new render, and with a clip (D-225, D-228). */
export function clear(): Promise<void>;
```

Failed IndexedDB calls in `lookup` count as a miss. `schemaVersion > DB_VERSION` is treated as in the other repos. No path or file name of the user's is stored (P-11): `opfsPath` is `exports/<uuid>.mp4`.

`rendererVersion`:

```ts
declare const __RENDERER_VERSION__: string | undefined;
export const RENDERER_VERSION = typeof __RENDERER_VERSION__ === "undefined" ? "test" : __RENDERER_VERSION__;
```

`vite.config.ts` defines it by hashing, with Node's `crypto`, `src/wasm/pkg/render/offcut_render_bg.wasm`, `src/workers/render/export-loop.ts` and `src/workers/render/encoders.ts`, taking 16 hex digits, or `"none"` when the WASM has not been built. Since `pnpm build` builds the WASM before Vite, a production build always has it.

### 11.2 Cost

A transcript of 90 s is tens of kB of JSON. `canonicalJson` plus SHA-256 is a few milliseconds, on the main thread, between two awaits. `lookup` is one IndexedDB read and one `getFile().size`. INV-17 names decoding, inference, filtering, rendering and encoding; a hash is none of these. `hashPrefix` already does more on the same thread.

### 11.3 What is and is not a hit

| Between the two exports | Key | Result |
|---|---|---|
| Nothing | Same | Hit |
| A word edited and then undone | Same (`word_edits` empty again) | Hit |
| Two words edited in the other order | Same (canonical) | Hit |
| A word edited | Different | Miss |
| A visual off, the style, the crop, the caption place, Highlight numbers | Different | Miss |
| Creator token, then Free token | Different kind | Miss |
| Creator token, then the same token after `period_end` | Different kind: verified as Free | Miss |
| New deploy with another renderer | Different version | Miss |
| New import of the same file | No row (D-228) | Miss |
| The stored file deleted by hand | Row deleted | Miss |
| A forged token | `E_ENTITLEMENT_INVALID` before the key | No lookup |

---

## 12. The gate: blockers and the export machine

### 12.1 `state/blockers.ts`

```ts
/** A file may take this much more than its bitrate and length say (TS §12.5). */
const STORAGE_MARGIN = 1.2;

export type ExportRoom = { freeBytes: number; neededBytes: number };

/**
 * The first thing that stands in the way of an export, or `null`. It is asked
 * while the export is still `idle` or, with `room`, in `gating` after a cache
 * miss. V6 adds the account and the plan.
 */
export function forExport(room?: ExportRoom): BlockerCode | null {
  const status = useExportStore.getState().status;
  // Gating with room is this export's own gate, not another one running.
  if (EXPORT_RUNNING.includes(status) && !(room !== undefined && status === "gating")) {
    return "B_EXPORT_IN_PROGRESS";
  }
  if (useClipStore.getState().status !== "ready") {
    return "B_PIPELINE_NOT_READY";
  }
  if (room !== undefined && room.freeBytes < room.neededBytes * STORAGE_MARGIN) {
    return "B_STORAGE_LOW";
  }
  return null;
}
```

### 12.2 `persistence/opfs.ts`

```ts
/** What the browser says is left for this site, or `null` when it does not say (D-229). */
export async function freeBytes(): Promise<number | null>;  // quota - usage from navigator.storage.estimate()
```

`neededBytes` in `start-export.ts`: `(profile.video_bitrate + profile.audio_bitrate) x clipInfo.duration / 8000`. `freeBytes` adds `renderCacheRepo.heldBytes()`, because the miss deletes that file ("after eviction", TS §12.5). `clipInfo.duration` over-counts a clip with cuts, which is on the safe side.

### 12.3 The export machine

The table does not change. This version starts firing `cache_hit` (D-227). `retry` stays unfired: "Try again" starts a new export through `reset`, as today (D-153 stands). The comment at the head of the file is corrected. `export-store.ts` gains:

```ts
export type ExportState = { /* ... */ blocker?: BlockerCode; summary?: ChangeSummary; answered: boolean };

/** gating → saving: the file of an earlier export, under its id (TS §21.4). */
export function cacheHit(original: ExportId): void;
/** gating → idle, saying why (D-231). */
export function blockedBy(code: BlockerCode): void;
/** saving → done, with what was changed. */
export function saved(summary: ChangeSummary, answered: boolean): void;
/** The question was answered for this export (D-234). Not a change of state. */
export function answered(): void;
/** Whether this export's question has been answered, in this tab (D-234). */
export function hasAnswered(id: ExportId): boolean;
```

`empty()` drops `blocker`, `summary` and `answered`, so a new start clears them. `state` importing a type from `blockers.ts`, which imports `export-store.ts`: type-only, no cycle at run time.

---

## 13. `start-export.ts`, step by step

Replaces `startExport` and `renderAndDeliver`. Line numbers of the old file do not apply. This is the file's one contract.

```text
startExport(clipId):
 1  forExport() !== null or another clip → return.
 2  record = entitlementRepo.get(). None → setUnavailable(true), return. forExport() again.
 3  last status done | failed → reset(). exportId = newExportId(). start(exportId)   ⇒ idle → gating
    signal = signalFor("export"); startedAt = now.
    values = editValues(clip)                                       (D-237)
 4  await lockForExport()                                            (D-221)
 5  try:
      goOn(signal)
      profile = answered(await whenFree(() => pool.render.exportProfile({ entitlementToken: record.token })))
      exportedAs = profile.kind === "creator" ? "creator" : "free"
 6    clipRecord = clipsRepo.get(clipId), failure read as none   (sourceHashPrefix lives there, not in the clip store)
      key = clipRecord !== undefined ? cacheKey(inputs) : null
      hit = key !== null ? lookup(key) : undefined
 7    HIT:
        cacheHit(hit.exportId)                                       ⇒ gating → saving
        deliver(getFile(hit.opfsPath)); delivered = {...}
        saved(hit.summary, hasAnswered(hit.exportId))                ⇒ saving → done
        unlockAfterExport()
        track export_done { ...values, profile: exportedAs, total_ms, from_cache: true }
        return
 8    MISS, the room:
        free = await freeBytes()
        if free !== null: blocker = forExport({ freeBytes: free + heldBytes(), neededBytes })
        if blocker !== null: blockedBy(blocker); unlockAfterExport(); return
 9    clear()                                                        ⇒ gating → rendering
      track export_started { profile: exportedAs }; started = true
10    renderCacheRepo.clear(); opfs.remove(exportsRoot); delivered = undefined
      prog48 = readProgram(clipId); goOn(signal)
      result = pool.render.exportClip({ exportId, entitlementToken, prog48 }, ...)
      goOn(signal); "cancelled" in result → E_INTERNAL
11    encoded(); finalized(); stage_timing x 2
      if key !== null: put(key, { clipId, exportId, opfsPath, bytes: size(opfsPath), createdAt, summary })
      deliver(getFile(result.opfsPath)); delivered = {...}
    catch:
      (as today: a cancel removes both files, exportCancelled, unlock, reloadPreview if the worker went)
      status === "gating" → clear()                                  (D-230)
      !started → track export_started { profile: planOf(token) }     (D-230)
      fail(failure); track export_failed; unlock; reloadPreview if gone; rethrow a fault of this file
12  saved(result.summary, false); unlockAfterExport()
    track export_done { ...values, profile: exportedAs, total_ms, from_cache: false }
```

Notes:

- `planOf` stays for the failed-token announcement and for `cachedPlan` (D-172). It is never used for the key.
- `answered(...)` in step 5 is the same three-line local helper `control-preview.ts` and `review-update.ts` each have: a worker's answer to a call nothing cancels. `exportStore.answered()` is another name in another namespace.
- On a hit `started` is set before the delivery, so a file that vanishes between `lookup` and `getFile` reports `export_failed` with no `export_started`.
- A stored file's size goes through `clipsRepo.toBytes`: no `as Bytes` in this file.
- The pill's Cancel button is disabled in `gating`, so a cancel cannot normally land there. The table has no `gating → cancelled`: if the signal is aborted while the status is `gating`, the catch calls `blocked()` (`gating → idle`) instead of `exportCancelled()`.
- `MAX_COUNT` caps every count; `capMs` the times.
- The eslint edge from `start-export.ts` to `control-preview.ts` exists. `start-export.ts` importing `persistence/render-cache-repo` and `state/blockers` is allowed by its layer. `answer-post-export.ts` needs `state` and `analytics` only.

---

## 14. `ChangeSummary` and "Would you post this?"

### 14.1 Placement (D-232)

`SidePanel`'s `.export` slot today holds `ExportFailure`. It gains `<ExportOutcome />`, a small wrapper in `ChangeSummary.tsx` that renders when `status === "done"` and the person has not closed it. Closing is local to the wrapper, keyed by `exportId`. The tabs below stay usable and scroll as today.

```text
┌ What we changed ─────────────────── [x] ┐
│ ✓ Voice cleaned                          │
│ ✓ 3 words emphasized                     │
│ ✓ 2 visual moments                       │
│                                          │
│ Would you post this as is?               │
│ [ Yes ]  [ With small edits ]  [ No ]    │
└──────────────────────────────────────────┘
```

After an answer the three buttons are replaced by "Thanks for telling us." Primitives: `PanelSection`, `Button` (`secondary`, `sm`), `IconButton`, `Check`. Tokens only; nothing new in `tokens.css`.

### 14.2 `summaryRows`

```ts
/** The rows of J11, only those that happened (PS §10). One row per field: V6.2, V6.3 and V6.6 each add one. */
export function summaryRows(s: ChangeSummary): string[] {
  const rows: string[] = [];
  if (s.voice_cleaned) rows.push(messages.export.summary.voice);
  if (s.captions_emphasized > 0) rows.push(messages.export.summary.emphasized(s.captions_emphasized));
  if (s.visual_moments > 0) rows.push(messages.export.summary.visuals(s.visual_moments));
  return rows;
}
```

### 14.3 Copy (`copy/messages.ts`)

```ts
blockers: { /* ... */ B_STORAGE_LOW: "There is not enough room on this device for this video. Free some disk space, then try again." },
export: {
  // ...
  summary: {
    title: "What we changed",
    close: "Close",
    voice: "Voice cleaned",
    emphasized: (n: number) => (n === 1 ? "1 word emphasized" : `${String(n)} words emphasized`),
    visuals: (n: number) => (n === 1 ? "1 visual moment" : `${String(n)} visual moments`),
  },
  question: {
    title: "Would you post this as is?",
    yes: "Yes",
    smallEdits: "With small edits",
    no: "No",
    thanks: "Thanks for telling us.",
  },
},
```

Every number passes through a function here (INV-15). No absolute claim: "Voice cleaned" states that the chain ran.

### 14.4 Accessibility

The block is a `section` with `aria-labelledby`. It is announced through the editor's one live region (D-192), not one of its own. `ExportOutcome` is reachable by Tab after the top bar, and the case below runs axe on the done state. Focus does not move when it appears: the person may be working.

### 14.5 `usecases/answer-post-export.ts`

```ts
export function answerPostExport(answer: PostExportAnswer): void {
  const { exportId, status } = useExportStore.getState();
  if (status !== "done" || exportId === undefined || hasAnswered(exportId)) return;
  answered();   // export-store.ts adds the id to its set and sets the state's `answered`
  track({ name: "post_export_answer", props: { answer } });
}
```

In memory only, as one tab's session. A reload asks again; V7's restore does not bring back an export's done state anyway.

---

## 15. Analytics

| Event | Today | After V5 |
|---|---|---|
| `export_started` | Sent before anything else | After a cache miss; or just before `export_failed` when the token was refused (D-230). `profile` is the verified kind when known |
| `export_done` | `word_edits: 0, events_disabled: 0, style: "clean", crop_adjusted: false, from_cache: false`, `events_kept` wrong for a switched-off visual | Real values (below) |
| `post_export_answer` | Never sent | Once per export (D-234) |
| `stage_timing` | `render_encode`, `mux` | The same; none on a hit |

```ts
function editValues(clip: ClipState) {
  const on = (e: DetectedEvent) => clip.edit.event_overrides[e.id] ?? e.enabled;
  const kept = clip.events.filter(on).length;
  return {
    word_edits: Math.min(MAX_COUNT, Object.keys(clip.edit.word_edits).length),
    events_kept: Math.min(MAX_COUNT, kept),
    events_disabled: Math.min(MAX_COUNT, clip.events.length - kept),
    style: clip.edit.style,
    crop_adjusted: clip.edit.crop_offset !== 0,   // the default (clip-store's empty())
  };
}
```

Counts and enums only (INV-18). No server change: `offcut-api-types/src/analytics.rs` already takes every value. `whatLeaves` already lists `post_export_answer`. `scripts/metrics.sql` reads it from V6 on.

---

## 16. Worker protocol changes

`workers/protocol.ts`. Both REOPENED, both one commit with both sides, both copied to TS §14.2.

```ts
export interface RenderWorkerApi {
  // ...
  /** The profile a token gives now, verified here (INV-9, D-220). Keeps nothing. */
  exportProfile(p: { entitlementToken: string }): Promise<ExportProfile>;
  exportClip(p: {
    exportId: ExportId;
    entitlementToken: string;
    /** The program's audio; transferred (TS §40.9 S-8). */
    prog48: Float32Array;
  }): Promise<{ opfsPath: string; summary: ChangeSummary; stageTimings: StageTiming[] }>;
}
```

`render.worker.ts`:

```ts
async function exportProfile(p: { entitlementToken: string }): Promise<ExportProfile> {
  try {
    return await Promise.resolve(worker.exportProfile(p.entitlementToken));
  } catch (thrown) {
    throw staged(thrown, "render_encode");   // the frozen case reads this stage
  }
}
```

`exportClip` still verifies the token itself: the worker trusts nothing the page says about a profile. The page's call is only so that the key is honest. `run-pipeline.test.ts`'s fake pool gains nothing but `writeProgram` in its OPFS mock.

---

## 17. Tests, case by case

### 17.1 What runs where

| Suite | D1 | CI |
|---|---|---|
| `cargo test` (new cases included) | Tier 1 and 3 | `ci` |
| Vitest (three new files + `export-loop.test.ts`) | Tier 1 and 3 | `ci` |
| `verify/test_verify_mp4.py` | Prompt 130, Tier 3 | `ci`, new step |
| `timeline-preserved.spec.ts`, `export-creator.spec.ts` | Tier 2 by case, Tier 4 | Not run: they need a processed clip (D-72) |
| `review-edit.spec.ts` | Prompt 129, Tier 4 | Not run |
| `@no-gpu` cases | Tier 4 | `e2e-media` |

### 17.2 Rust

| File | Case | Asserts |
|---|---|---|
| `offcut-dsp/src/loudness.rs` | `a_mono_programme_reads_3_lu_louder_as_two_loudspeakers` | D-204's difference, 3.01 ± 0.05 LU |
| `offcut-dsp/src/loudness.rs` | `a_1khz_sine_of_peak_minus_20_reads_minus_20` | Rewritten by name from `..._reads_minus_23` (section 8) |
| `offcut-dsp/tests/loudness_target.rs` | `the_output_as_a_stereo_file_reads_the_target` | Every existing fixture, as two channels, within ±1 LU of -14 |
| `offcut-mp4/tests/mux_roundtrip.rs` | `frames_written_in_decode_order_are_shown_in_presentation_order` | `ctts` runs, `elst` media time, demuxer's times, keyframes by sample number |
| | `no_reordering_writes_no_ctts_and_no_video_edit` | The six tables; byte equality with a run of today's order |
| | `a_gap_in_the_frames_is_out_of_order_at_finalize` | |
| | `a_frame_far_ahead_is_out_of_order_at_once` | `MAX_REORDER` |
| | `ninety_seconds_reordered_fit_the_space_kept_for_moov` | |
| | `a_moov_over_the_reserve_is_refused` | `MoovOverflow` |
| | `a_reserve_with_under_8_bytes_left_is_refused` | |
| | Proptest extended | Any permutation within the window round-trips |
| `offcut-mp4/tests/mux_reorder.rs` | `real_b_frames_come_back_at_30_fps` | Section 9.3 |

`a_frame_out_of_its_turn_is_out_of_order` stays as written: its first assertion is "first sample must be frame 0", its second a duplicate.

### 17.3 TypeScript (Vitest)

**`state/machines/export-machine.test.ts`**, in `clip-machine.test.ts`'s shape: every legal row of TS §12.2 through `transition` and through the store's action; every other pair refused; the two named in §12.3 (`rendering → rendering`, `done → rendering`).

**`state/blockers.test.ts`**: `forExport` for each status of the export machine and each of the clip's; the order of the codes; `B_STORAGE_LOW` at free = needed x 1.2 exactly (not blocked), one byte under (blocked); no `room`, no storage blocker; with `room` in `gating`, no `B_EXPORT_IN_PROGRESS`. `forImport` unchanged. `BLOCKER_CODES` holds the eight of TS §12.5.

**`usecases/start-export.test.ts`**, with the scripted pool of `run-pipeline.test.ts`:

| Group | Cases |
|---|---|
| Happy path | The order of calls: lock, `exportProfile`, lookup, `readProgram`, `exportClip`, `put`, deliver, saved, unlock; machine path idle → gating → rendering → muxing → saving → done |
| Download | Name `offcut-<yyyymmdd-hhmm>.mp4`, nothing of the source |
| `render-cache` (S-9's proof) | The key changes with each of: source hash, transcript, events, edit, plan, time map, theme, music id, profile kind, renderer version. It does not change with the order of `word_edits`' keys, or with an edit made and undone |
| Hit | No `exportClip`, no `readProgram`, no sweep; `from_cache: true`; no `export_started`, no `stage_timing`; original `ExportId` in the store; the row's summary shown; a hit after the question was answered does not ask again |
| Row without its file | Row deleted, miss |
| Verified profile | A token whose payload says creator and whose worker answers free: the key has `free`, `export_done` says `free` (INV-9) |
| Token refused | `E_ENTITLEMENT_INVALID`, `export_started` then `export_failed`; no lookup; no sweep; `failed` |
| Storage low | `B_STORAGE_LOW`, back to `idle`, nothing swept, unlocked, the code in the store; the next start clears it. No estimate: goes on. Cached file's bytes count as free |
| Encoder and audio failures | `E_ENCODE_VIDEO`, `E_ENCODE_AUDIO` end in `failed` with their copy (TS §11's rows) |
| Cancel | `idle`, both files removed, no row, no `export_done` |
| Quota | `E_STORAGE_QUOTA` from the worker: `failed` |
| Worker gone | `E_WORKER_CRASH`: `reloadPreview` runs |
| Real values | `word_edits` 2, one override off gives kept and disabled, `style`, `crop_adjusted` |
| Import | `importClip` clears `renderCache` (in this file or `import-clip`'s coverage in `run-pipeline.test.ts`) |

**`workers/render/export-loop.test.ts`**, with fake `VideoEncoder`, `AudioEncoder`, `VideoFrame`, `AudioData`, a fake session, source and sink, and `loadRender` mocked:

| Case | Asserts |
|---|---|
| Frame accounting | Exactly `frame_count` frames encoded, timestamps `floor(n x 1e6 / 30)`, keyframe every 60 |
| Identity | `source_at` is asked for each `t`, and `frameAtBlocking` gets `recorded(pos)` |
| Seam: a map that jumps and holds | With a fake `source_at` that goes 0, 1,000 (jump), 1,033, then `held` 2,000 twice: the source is asked exactly those, in that order |
| `null` position | `E_INTERNAL` `NoScene` |
| Audio | Every sample of `prog48` reaches the encoder; padded to `N x 1600`; never trimmed; both channels equal |
| Backpressure | Never more than `ENCODE_QUEUE_MAX` plus one queued |
| Video failure | Next config, new encoders, `sink.reset`, `onProgress(0, N)`, frames from 0, success |
| All entries fail | The last failure is thrown; every frame closed |
| Unsupported entry | Skipped by `supportedConfigs` |
| Audio failure | Once more with the same config; twice thrown |
| Not restarted | A sink error, `E_GPU_LOST`, `E_MUX` |
| Cancel | `{ cancelled: true }`, no restart, frames closed |
| `liveFrames` | 0 after every case above (INV-11) |

### 17.4 Python

Section 7.6.

### 17.5 End to end (Playwright, D1, keyed `pnpm build:e2e`)

**`helpers/fixtures.ts`:**

```ts
export function verifyMp4(file: string, profile: "free" | "creator", expectedDurationMs: number, source?: string): Verdict;
/** Every check passed and none was not measured. `flashes`: how many white frames were compared. */
export function expectAllChecks(verdict: Verdict, o?: { flashes?: number }): void;
```

**`export-creator.spec.ts`** keeps its clip and serial order. Changed or added:

| Case | Change |
|---|---|
| Creator export | `verifyMp4(file, "creator", durationMs, referenceClip())`; `expectAllChecks`; `PASS 11` |
| Events | `from_cache: false` and the values as today: the clip is unedited |
| Free-plan token | The same with `"free"`; as creator also `FAIL 9`. The comment about size being the one difference goes |
| **new** The summary and the question | On the done state: `What we changed` with the voice row, zero rows absent. Answer "With small edits": `post_export_answer { answer: "small_edits" }` once; thanks line. Axe on this state |
| **new** Same token again | Instant: `total_ms` under 2,000, no frames rendered, `from_cache: true`, no new `export_started` or `stage_timing`; the file's SHA-256 equals the Free file's; storage holds that one file; the question is not asked again |
| **new** Forged token after a hit | `E_ENTITLEMENT_INVALID` before any lookup: no file given, the stored one stays usable with the good token |
| **new** No room | `navigator.storage.estimate` stubbed low on the page; Creator token (a miss): `export-status` shows the words, nothing swept, stays `idle`, button enabled; stub removed; the next click would proceed (not run here: it is the next case's export) |
| **new** Expired Creator token | `periodEnd` in the past, with a valid signature, right after a Creator file is stored would be the risky case; here the stored file is Free's, so it asserts the opposite direction: expired Creator verifies as Free, **hits** the stored Free file, 720x1280, watermark present (INV-9). The unit test holds the Creator-file direction |
| `no metadata` case | `PASS 11`; `metadataProblems` and `tagsOf` removed |

Order within the group: no token, other key, Creator, progress, events, name, storage, quiet, preview after, Free, second export, **summary and question, same token, forged, no room, expired**. The added cases cost seconds; none renders.

**`timeline-preserved.spec.ts`** (new): TS §27.1. Two serial groups, one per fixture; each processes its clip once, about 45 s to 1 min **(estimate)**.

*`speech_20s_long_pause.mp4`:*

| Case | Asserts |
|---|---|
| Words keep their times | From `readIndexedDb(page, "transcripts", clipId)`: every word's start and end lies outside the pause or at its edges; the first word after it starts at 10,000 ms or later; none is moved earlier. Word text is never printed (SAB rule 9) |
| Caption at its spoken time | Seek to that word's start + 60 ms: the caption mirror has it. Seek 1,000 ms before: it does not |
| Preview's length | The scrub bar's `max` is the source's duration; played from 500 ms before the end, the play button returns and `previewAt` is at or under it |
| Export is 20 s | Creator: `verifyMp4(file, "creator", 20_000, fixture)`, `expectAllChecks(verdict, { flashes: 2 })`; 600 frames (check 3 reads them) |
| An edit leaves the timeline alone | Fix a word, switch a visual off, choose Bold: the scrub bar's `max` unchanged; `export` again; **a miss**: `from_cache: false`, `word_edits: 1`, `events_disabled` as made, `style: "bold"`; `expectAllChecks(..., { flashes: 2 })`; the summary's rows follow the edit |
| The same edit again | **Hit**: instant, same bytes |
| Undo everything, then export | A miss (D-225), and the file passes |
| The encoder fails | `__offcutDev.failEncoderAt(30, 1)`: the bar falls back; the file passes the verifier; no failure card |
| Every entry fails | `failEncoderAt(30, 9)`: the card with `E_ENCODE_VIDEO`'s words, "Try again" and "Close"; no file in `exports/` or `exports/tmp/`; the preview plays again |

*`speech_30s_sparse.mp4`:*

| Case | Asserts |
|---|---|
| Words stay in their pieces | Each word lies inside a piece placed at 0.5, 7.0, 13.5, 19.5 or 26.0 s (± 300 ms); at least three pieces hold a word |
| Export is 30 s | `verifyMp4(file, "creator", 30_000, fixture)`, `expectAllChecks(..., { flashes: 2 })`; 900 frames |

**`review-edit.spec.ts`:** section 6.3. **`model-download.spec.ts`:** D-236. **`cancel.spec.ts`:** unchanged; its "no cache entry" assertion, written in V4, now means something.

**INV-11.** The live-frame counter is asserted inside `exportClip` on every export (D-45). A cancelled export is read through the page, as in V4 (`framesGivenBack`).

---

## 18. Fixtures, snapshots, goldens

- **`ok_bframes.mp4`** (D-212): `$(src 1280 720 30 2) $OUTF -c:v libx264 -bf 2 -g 30 -pix_fmt yuv420p $FAST $AAC $FS`. Its manifest row names `mux_reorder.rs`. The generator's step 8 checks its properties. 26 fixtures.
- **No new recording** (the V3 rule): everything is derived.
- **Goldens and display-list snapshots do not change.** V5 changes neither the scene nor the renderer. A changed `.png` or `.snap` in V5's diff is an error to explain.
- **`verify/watermark_mask_720.png`**: committed, about 3 kB. `verify/make_watermark_mask.py` recreates it; run again only if `watermark.rs` or `safe_area.rs` moves the text.

---

## 19. CI, scripts, hosting

### 19.1 `ci.yml`

```yaml
jobs:
  # Whether the heavy jobs are needed, and the secret scan, which documents need too.
  changes:
    runs-on: ubuntu-latest
    outputs:
      code: ${{ steps.diff.outputs.code }}
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7
        with: { fetch-depth: 0 }
      - name: Scan for secrets
        run: |  # the gitleaks download of today's "Install the build tools", moved here
          ...
          gitleaks detect --source . --redact --no-banner
      - id: diff
        run: |
          if [ "$GITHUB_EVENT_NAME" != pull_request ]; then echo code=true >> "$GITHUB_OUTPUT"; exit 0; fi
          others=$(git diff --name-only "origin/$GITHUB_BASE_REF...HEAD" | grep -vE '^docs/|\.md$' || true)
          [ -n "$others" ] && echo code=true >> "$GITHUB_OUTPUT" || echo code=false >> "$GITHUB_OUTPUT"
  ci:
    needs: changes
    if: needs.changes.outputs.code == 'true'
```

`e2e-media` needs `ci` and skips with it. `deploy-*` run on `push` only, where `code` is always true. The header comment names the rule. `verify/README.md` matches `\.md$` and is a document, which is correct. `main` has no branch protection (`gh api`, 2026-10-31: 404), so a skipped job blocks nothing. The `ci` job's checkout loses `fetch-depth: 0`, which only the scan needed, and its tool step loses the gitleaks lines. A docs-only pull request that breaks the TS §5 tree is not caught until `main`'s run: run `node scripts/check-file-tree.mjs` locally before pushing an edit of §5.

A new step in `ci`, after "Generate the fixtures":

```yaml
      - name: Set up Python
        uses: actions/setup-python@5fda3b95a4ea91299a34e894583c3862153e4b97 # v7.0.0
        with: { python-version: "3.13" }
      - name: The verifier's own test
        run: |
          python -m pip install -r verify/requirements.txt
          python verify/test_verify_mp4.py
```

The drill of Prompt 129: a commit changing only a `.md` skips `ci` and `e2e-media`, and the scan runs. This is read on the first docs-only push, or by running the `diff` step's script locally on both kinds of diff.

### 19.2 `e2e-media.yml`

No job change. It already installs Python and `requirements.txt`. `@no-gpu` cases do not call the verifier. Its header's "all 30 cases" is stale, and corrected while here.

### 19.3 `playwright.config.ts`, `package.json`

`timeline-preserved.spec.ts` joins `MEDIA_SUITES`. `pnpm verify` is unchanged. A root script `"verify:test": "python verify/test_verify_mp4.py"`.

### 19.4 Hosting

No change. The merge rebuilds the API image with the same code. Deployed V5: previews, "Exporting needs an account" on the Export button (BP §7.2), the seven headers, `crossOriginIsolated`. The app shell should stay under 400 kB gzip: V4's was 337.3 kB, V5 adds about 5 kB **(estimate)**; `check-hosts.mjs` prints it.

---

## 20. Measurements

Written to `docs/v5/experiments.md` in V4's shape (Question, Method, Numbers, Date, Decision), all labelled `d1`.

| Reading | Method | Target | If it misses |
|---|---|---|---|
| E-4 `render_encode` | `BENCH_DEVICE=d1 BENCH_CLIP=portrait BENCH_RUNS=5 pnpm bench:device`, keyed production build, Docker closed | At most 90 s per 60 s. V4: 38.8 s | Over 150 s: BP §4.2's Canvas2D path. Not expected |
| `mux` | Same | At most 2 s. V4: 25 ms | Look at it |
| `detect_scene` (V4 known issue 37) | Same | About 245 ms. V4: 300.5 ms loaded, 213-249 ms settled | If still over on a quiet machine: an issue for V9 with the numbers. It is not a V5 blocker |
| Loudness | Check 7's `INFO` on the three real exports | -14 ± 1 LUFS | Section 8 |
| Sync | Check 8's `INFO` on the pause fixtures | Within 20 ms; within a frame | Look before moving anything |
| Watermark correlation | Check 9's `INFO`, free and creator | ≥ 0.5, ≤ 0.2 | Section 7.3 |
| Bitrate | Check 10's `INFO`, both profiles | ≤ 12 / ≤ 6 Mb/s | Section 21 |
| Hit time | `export_done.total_ms` with `from_cache: true` | Under 2 s | Look at `lookup` |
| `cacheKey` cost | A temporary `performance.measure`, removed | A few ms | Move it to a worker |
| App shell | `pnpm build` | ≤ 400 kB gzip | Look at what grew |

TS §39.3 item 14 ("the verifier's thresholds are V5's") gets the measured values.

---

## 21. Risks, each with its way out

| Risk | Likelihood | Way out |
|---|---|---|
| Check 8's envelope correlation is weak on sparse audio after denoising | Medium | The windows hold speech on both fixtures. Tune `MIN_ENVELOPE_CORR` on saved exports; it is an (assumption). Do not narrow the windows, which TS §27.2 fixes |
| Check 9's correlation is too close to call on a busy background | Medium | Section 7.3's steps. The mask reads as a pattern of fill and outline, which a blurred background does not mimic |
| Check 10 finds a hardware encoder over 1.5x on D1 | Low | First `bitrateMode: "constant"` on that entry in `encoders.ts`. TE-4 changes both copies of the constant or neither (TS §21.2) |
| The stereo fix makes a quiet clip less loud than a person expects | Low | It is the promised -14. The gain cap and silence rules are unchanged. The founder hears it once at the deployed check |
| `ctts` plays wrongly somewhere | Low on the first release: D1 does not reorder | `mux_reorder.rs` on real B-frames through `ffmpeg`; manual phone playback is V10's checklist |
| A restart leaves the source in a bad state | Low | `frameAtBlocking(0)` is the same jump a scrub back makes, tested since V4; the case `failEncoderAt(30, 1)` proves it |
| The render cache serves a wrong file | Would be serious | Verified kind in the key (D-220); computed version (D-223); the case table of section 11.3, each a test |
| Canonical JSON misses a float that differs by rounding | Low | An edit reached two ways gives the same number only if it is the same number; a miss is safe, and only a hit could be wrong |
| Sharing `review-edit`'s clip leaks state between cases | Medium, on the first run | `resetClip` asserts the clean state and fails by name; a case that must have a clean page moves to its own load, as case 8 did |
| A CI-only failure shows up later because the agent did not wait | Medium | The next prompt reads it first and fixes it before going on (D-198) |
| V4's closing work finds something | Low | Preconditions block; fix it on `v5-build` with its own entry |
| Time overrun | Medium | Cut in this order: the `ok_bframes` real-sample test's `ffmpeg` half, then `failEncoderAt(30, 9)`. Never check 7, 8 or 9, or the cache's verified key |

---

## 22. The eight prompts (129 to 136), in brief

The prompts are in `docs/v5/coding-promptsv5.md` and the same work, step by step with its commands and milestones, is in `docs/v5/v5buildguide.md`. Each prompt is one commit on `v5-build`. This table is the summary.

| Prompt | Plan phase | Guide phase | Goal | Commit title | Push | Time (estimate) |
|---|---|---|---|---|---|---|
| Setup | - | 0 | Branch, baseline, V4's closing entry | `V5 setup: branch, baseline, V4's closing entry` | Yes; the founder opens the draft pull request | 30 min |
| 129 | A | 1 | Later prompts test faster, with no gate lost | `V5 P129: shared clip in review-edit, gates, docs-only CI skip` | First of three; not waited on | 2-2.5 h |
| 130 | B | 2 | The tool that judges everything exists, and tells good from bad on real files; an export reads -14 LUFS | `V5 P130: verifier checks 7-11, its self-test, and the stereo loudness fix` | No | 2 h |
| 131 | B | 3 | A stream with B-frames makes a correct file; the overflow guard is tested | `V5 P131: ctts, video edit list, moov overflow guard tested` | No | 1 h |
| 132 | B | 4 | S-8 is built; a failing encoder does not fail the export | `V5 P132 (S-8): export over program time, prog48, ladder restart` | No | 2 h |
| 133 | C | 5 | BP §7's demo is a test | `V5 P133: timeline-preserved on both pause fixtures, all 11 checks` | No | 1.5 h |
| 134 | C | 6 | S-9: the same edit twice is instant, and no file reaches someone the token does not entitle | `V5 P134 (S-9): render cache on the verified profile, the gate, B_STORAGE_LOW` | Second of three; not waited on | 2 h |
| 135 | D | 7 | J11, and the real `export_done` | `V5 P135: ChangeSummary, the question, real export_done values` | No | 1.5 h |
| 136 | D | 8 | Measured, recorded, merged, deployed, tagged | `V5 P136: bench, experiments, copy-back, final gate` | Third; waited on | 2.5 h, plus 30-45 min of the founder's |

---

## 23. Hand-over: V6 and V6.1 to V6.7

### 23.1 What V5 freezes

| Frozen in V5 | Where |
|---|---|
| The verifier's command line and its `PASS` / `FAIL` / `INFO` lines | `verify_mp4.py` |
| `runExport`'s attempt-and-ladder shape; `prog48` as the audio it encodes | `export-loop.ts` |
| `exportProfile`, `exportClip`'s `prog48` | `protocol.ts` |
| `CacheInputs`' fields (V6.x may set them; a new input is appended) | `render-cache-repo.ts` |
| `forExport(room?)`'s order | `blockers.ts` |
| `summaryRows`' rule: one row per field, zero absent | `ChangeSummary.tsx` |
| `resetClip`'s meaning: a clean clip, or the case fails by name | `helpers/fixtures.ts` |

V4's freezes stand except the two reopened by D-214 and D-220.

### 23.2 File by file: what each later version adds

| V5 file | V6 | V6.1 | V6.2 | V6.3 | V6.4-6.5 | V6.6 | V6.7 | V7 | V9 |
|---|---|---|---|---|---|---|---|---|---|
| `verify_mp4.py` | - | - | Tests give `--expected-duration-ms` from the map; check 8's windows take the map's joins | Check 7 on a mix | - | End-card duration | - | - | Runs on the corpus (M2.3) |
| `start-export.ts` | Sign-in gate; receipt on a **new** render only, in one place | Plan source in `done` | - | - | - | - | - | `quota.ensureFree` replaces the sweep | - |
| `render-cache-repo.ts` | - | Stored plan's hash, by the same field | Map's hash, by the same field | `musicId` | - | - | `theme` = kit hash; INV-9: the profile kind stays in the key | Several rows, eviction | - |
| `export-loop.ts` | - | - | Nothing, if the map is right; forward jumps go through `frameAtBlocking` | Nothing: `prog48` holds the mix | - | `Held` works already | - | - | - |
| `opfs.ts` `prog48.f32` | - | - | Rewritten after a restore | Rewritten after a music change | - | Longer with the tail | - | Restore reads it | - |
| `blockers.ts` | `B_NOT_SIGNED_IN`, `B_ENTITLEMENT_EXPIRED`, `B_NO_FREE_EXPORTS` | - | - | - | - | - | - | Eviction before `B_STORAGE_LOW` | - |
| `ChangeSummary.tsx` | - | - | `pauses_shortened`, `time_removed` | `music` | - | `chapters` | - | - | - |
| `PostExportQuestion.tsx` | - | E-11 splits answers by plan source, in `metrics.sql`, not here | - | - | - | - | - | - | - |
| `export_done` | - | `items_kept`, `items_disabled` | `cuts_kept`, `cuts_restored` | `music_on` | - | - | `brand_kit_on` | - | - |
| `review-edit.spec.ts` | - | Regenerate plan, on the shared clip | Restore a cut | Music | Presets | - | Kit | - | - |
| `device-bench.ts` | - | - | - | - | Heavier frames, against V5's baseline | - | - | - | R1 and R2 at 10 runs |

### 23.3 What must be true on V6's first morning

- [ ] V5 is tagged `v5`, merged and deployed.
- [ ] `verify_mp4.py` passes all 11 on a real export, and its self-test is in CI.
- [ ] `git grep -nE "\bV5\b" -- web/src crates verify` shows no marker of work to do.
- [ ] `start-export.ts` has one place where a new render is told from a cache hit, so V6's receipt goes there.
- [ ] `docs/v5/experiments.md` holds section 20's readings.

### 23.4 What V6 to V6.7 inherit without further work

- Any map the scene accepts is exported correctly: as long as the map says, frame by frame, with the audio `assemble` wrote.
- Any change to `assemble`'s output reaches the export.
- Any new visual reaches the export: the loop draws what the scene says.
- A wrong file cannot pass quietly: 11 outside checks, on every export a test makes.
- A review case costs seconds, not a minute.

---

## 24. Exit checklist

V5 is done when every box is ticked (BP §1.2, BP §7.3, TS §28 M1.6, TS §40.9 S-8, S-9).

**Code and checks**

- [ ] `pnpm check` green: fmt, clippy, deny, ESLint, both `tsc` programs, `check-file-tree`, `check-gen-clean`, `check-copy-codes`.
- [ ] `pnpm fixtures` then `pnpm test` green on D1 and in CI.
- [ ] BP §7.3: `export-creator.spec.ts` passes with all 11 verifier checks.
- [ ] BP §7.3: `timeline-preserved.spec.ts` passes on `speech_20s_long_pause.mp4` and `speech_30s_sparse.mp4` (600 and 900 frames).
- [ ] BP §7.3: `start-export.test.ts`, `export-machine.test.ts`, `blockers.test.ts` pass.
- [ ] BP §7.3: the live-frame counter is 0 after an export and after a cancelled export (INV-11).
- [ ] `export-loop.test.ts`, `mux_reorder.rs`, `test_verify_mp4.py` pass; the last in CI.
- [ ] S-8: the loop reads `source_at`; `prog48.f32` is encoded; `git grep "out48" web/src/workers/render` prints nothing.
- [ ] S-9: `start-export.test.ts`'s `render-cache` group covers all ten inputs.
- [ ] A file reads -14 ± 1 LUFS as stereo (`ffmpeg`, not the product's meter).
- [ ] No goldens or display-list snapshots changed.
- [ ] No source file over 700 lines. No `fetch(` outside the four files; no request added.
- [ ] Test counts at least the baseline's plus V5's; none fell except cases this plan rewrites by name.

**Speed**

- [ ] `review-edit.spec.ts` makes 4 clip loads.
- [ ] A docs-only pull request skips `ci` and `e2e-media`; the scan still runs.
- [ ] The full suite's time is recorded against V4's 41.8-54.9 min.
- [ ] Full media runs in V5: one, or each extra one explained in the changelog.

**Measurements**

- [ ] `bench/results/d1-<date>-portrait.json` committed (5 runs); `render_encode` within E-4's line.
- [ ] Section 20's readings in `docs/v5/experiments.md`, labelled `d1`.
- [ ] TS §39.3 item 14 holds the verifier's values.

**Deployment**

- [ ] `ci.yml` green on `main`; API then web deployed; header check and `@smoke` pass; `crossOriginIsolated === true`.
- [ ] On the deployed page: the reference clip previews; Export says the account line (BP §7.2).
- [ ] Locally, with a minted token: the long-pause fixture exports at 20 s, and the second export is instant (BP §7's demo).

**Records**

- [ ] `v5changelog.md` has an entry per prompt and closes the open items it can.
- [ ] Known issues 33 and 36: read against V5's CI runs; 37: read at the bench.
- [ ] Copy-back of section 25 applied.
- [ ] The founder has looked at the done state, and heard an export.
- [ ] Section 23.3 ticked.
- [ ] Tagged `v5`.

---

## 25. Spec issues found, decisions to copy back

| # | Issue | Where | Handled by |
|---|---|---|---|
| 1 | The chain measures mono and the file is stereo: a file reads about 3 LU over PS §20.1's target; `loudness_target.rs` cannot see it | TS §18.2, §18.3 | D-204. Copy back: "measured as dual mono" |
| 2 | TS §21.4 says the key is computed "through `hash_api`", which the main thread does not load | TS §21.4 | D-222 |
| 3 | `RENDERER_VERSION` is named and exists nowhere | TS §21.4, §19.6 | D-223: computed by the build |
| 4 | TS §21.4's key takes `ProfileKind` without saying whose. The unverified `planOf` would break INV-9 | TS §21.4 | D-220 |
| 5 | TS §14.2's `exportClip` has `out48`; V4 §22.2's protocol row says nothing changes | TS §14.2, V4 §22.2 | D-214 |
| 6 | TS §12.4 locks the preview at `rendering`; the gate needs the worker free first | TS §12.4 | D-221 |
| 7 | TS §12.5 evaluates `B_STORAGE_LOW` synchronously; `estimate()` is async | TS §12.5 | D-229 |
| 8 | TS §9 says "`B_STORAGE_LOW` → ErrorPanel" | TS §9 | D-231 |
| 9 | TS §12.2 has no `gating → failed`, and a refused token now fails there | TS §12.2 | D-230: table unchanged |
| 10 | TS §40.9 S-8: "writes the expected duration beside the export" | TS §40.9 | D-210 |
| 11 | TS §27.2 says `N` is always the source duration; true for the identity map only | TS §27.2 | Copy back: "the map's duration" for V6.2 |
| 12 | TS §27.2 fixes check 8's and 9's limits but not their method | TS §27.2 | D-205, D-206. Copy back the method |
| 13 | TS §5 says the mask is "drawn independently of the renderer", and says nothing of how | TS §5 | D-206 |
| 14 | TS §13.3's table lacks the two new components | TS §13.3 | Copy back |
| 15 | TS §23.1's `renderCache` row lacks `summary` | TS §23.1 | D-226 |
| 16 | BP §7 and TS §28 name R1 for E-4 | BP §7.3 | D-202 |
| 17 | BP §7's table says `render/export-loop.ts complete`, with no mention of an audio retry; TS §11's row asks for one | TS §11 | D-217 |
| 18 | TS §33 lists every pull request running the same steps | TS §33 | D-199. Copy back |
| 19 | Bench is defined as ten runs | TS §30, BP §4 | D-200: 5 for V5, 10 stays the default and V9's |
| 20 | `loudness.rs`'s `..._reads_minus_23` test and possibly `pink_noise_reaches_the_target` assert today's reading, and this plan first said they run unchanged | V5 §8 | Section 8: rewritten by name |
| 21 | Analytics flushes every 10 s, so a shared clip's undo events would reach the next case and fail its `toEqual` | V5 §6.3 | Section 6.3, step 4 |
| 22 | `undo` has no test id; `check-hosts.mjs`'s markers are an explicit list that does not hold a fault's name | V5 §6.3, D-218 | Section 6.3; D-218 |
| 23 | `sourceHashPrefix` is in the clip's record, not the clip store; "the cached file's bytes" had no reader | V5 §12.2, §13 | `clipsRepo.get`, `heldBytes` |
| 24 | The answered set in `answer-post-export.ts` needed a use-case edge from `start-export.ts` | V5 §14.5 | D-234: in `export-store.ts` |
| 25 | `fakeExport` reaches `rendering` only; `screens.spec.ts`'s done state needs a hook | V5 §4 | `fakeExportDone` in `dev-hooks.ts` |

**To copy back at Prompt 136, from the code:** TS §5 (new files), §9 (C-7, C-9, C-10), §12.2's prose, §12.4, §12.5, §13.3, §14.2, §18.2, §21.1, §21.3, §21.4, §23.1, §27.1, §27.2, §33, §39.3 item 14, §40.9 (S-8, S-9 as built). BP §7: "as built". Corrections to `verify/README.md`. V5 as built goes in a table in this document, as V4 §24.3 did.

---

## Appendix A. Per-file specifications in brief

Per BP §1.3. The implementer gets this row, the section named, TS §2, §7, §10, §11, and the signatures of the imports.

| File | Purpose | May import | Rules |
|---|---|---|---|
| `verify/verify_mp4.py` | Judges an export | Standard library, NumPy, `ffmpeg`, `ffprobe` | Imports nothing from `crates/` or `web/`, no library they use (TS §27.2) |
| `verify/test_verify_mp4.py` | Proves each check | The same, and `verify_mp4` by subprocess | Cleans its directory; prints no paths from the repo's media |
| `verify/make_watermark_mask.py` | Draws the mask once | Pillow | Not run in CI; numbers named with their source |
| `persistence/render-cache-repo.ts` | Key, row | `db`, `opfs`, `schema`, `gen` | The only file that names `renderCache` besides `schema.ts`; no file names |
| `state/blockers.ts` | Why an export cannot start | The stores | The only place the conditions are evaluated; changes nothing |
| `usecases/start-export.ts` | C-7, C-9, C-10 | `pool`, the repos, the stores, `analytics`, `control-preview`, `cancel-job` | Section 13. The profile in a key is the worker's |
| `usecases/answer-post-export.ts` | The answer | `state`, `analytics` | Once per `ExportId` |
| `workers/render/export-loop.ts` | Frames and audio to a file | `gen`, `load-render`, `rpc`, `rpc-worker`, `encoders`, `opfs-sink`, `video-source`, `preview-loop` | Every frame once; closes what it opens; only video and audio-once failures restart |
| `workers/render/encoders.ts` | The ladder | `gen`, `rpc-worker` | The only file defining encoder configurations |
| `crates/offcut-mp4/src/mux.rs` | The file | `offcut-types`, `mux_boxes` | Holds no sample in memory; a file with no reordering is unchanged |
| `ui/components/ChangeSummary.tsx` | J11 | `messages`, `export-store`, primitives, `PostExportQuestion` | Only changes that were made; every word from `messages` |
| `ui/components/PostExportQuestion.tsx` | The question | `messages`, `answer-post-export`, primitives | Three answers from the generated type |

---

## Appendix B. Invariants V5 touches, and where each is enforced

| Invariant | Where | Test |
|---|---|---|
| INV-5 Timeline | The loop over the map | `timeline-preserved.spec.ts`, check 8 |
| INV-8 Exact frames at 30 fps, passes the verifier | The loop, the muxer | Check 3, `export-loop.test.ts` |
| INV-9 Size and watermark from the verified token only | `export-job.ts`, `exportProfile`, the key | Check 9 both ways; the verified-profile cases |
| INV-10 Audio duration | `run-pipeline.ts`'s `assemble` unchanged; padding only | `duration_preserved.rs`, check 5, `export-loop.test.ts` |
| INV-11 Frames closed | `attempt`'s `finally`, `closeFrames` | `export-loop.test.ts`, in every export |
| INV-15 Numbers through copy | `messages.export.summary` | `check-copy-codes`, review |
| INV-17 Main thread does no media work | The hash is not media work | Section 11.2 |
| INV-18 Analytics enums and counts | `editValues`, `answerPostExport` | Server's allowlist, `analytics_allowlist.rs` |
| TS §1.4 No media leaves | No request added | `quiet export` case, `check-hosts` |
| P-11 No user file name in a path | `opfsPath` is a UUID | Storage cases |

---

## Appendix C. Named constants V5 introduces

| Constant | Value | Where | Status |
|---|---|---|---|
| `LOUDNESS_TARGET`, `LOUDNESS_TOLERANCE` | -14, 1 | `verify_mp4.py` | TS §27.2 |
| `ENVELOPE_HOP_SAMPLES` | 480 | `verify_mp4.py` | D-205 |
| `SYNC_WINDOW_MS` | 5,000 | `verify_mp4.py` | TS §27.2 |
| `SYNC_LAG_HOPS` | 100 (±1 s) | `verify_mp4.py` | (assumption) |
| `SYNC_LIMIT_MS` | 20 | `verify_mp4.py` | TS §27.2 |
| `MIN_ENVELOPE_CORR` | 0.5 | `verify_mp4.py` | (assumption, Prompt 130) |
| `SILENT_WINDOW_DBFS` | -50 | `verify_mp4.py` | (assumption) |
| `FLASH_MIN`, `FLASH_ABOVE` | 200, 80 | `verify_mp4.py` | (assumption) |
| `WATERMARK_FRAMES` | 15 | `verify_mp4.py` | (assumption) |
| `WATERMARK_FREE_MIN`, `WATERMARK_CREATOR_MAX` | 0.5, 0.2 | `verify_mp4.py` | (assumption, TS §39.3 item 14) |
| `BITRATE_LIMIT` | 12 and 6 Mb/s | `verify_mp4.py` | TS §27.2 |
| `MAX_REORDER` | 16 | `mux.rs` | H.264's DPB limit |
| `STORAGE_MARGIN` | 1.2 | `blockers.ts` | TS §12.5 |
| `RENDERER_VERSION` | 16 hex, computed | `vite.config.ts`, `render-cache-repo.ts` | D-223 |
| `MAX_COUNT`, `MAX_DURATION_MS` | as today | `start-export.ts` | TS §22.6 |

---

## Appendix D. Size estimate

Estimates, from reading the files each row touches, and calibrated against V3 (code +11,866 lines) and V4 (code +27,275 lines).

| Area | Product lines | Test lines |
|---|---|---|
| Verifier and mask tool | 350 | 180 |
| Muxer | 110 | 200 |
| Loudness | 1 | 25 |
| Export loop, encoders, sink, job, protocol, faults | 150 | 280 |
| Program audio file, pipeline | 25 | 5 |
| Render cache, version | 130 | - |
| Gate: blockers, store, opfs | 70 | 100 |
| `start-export.ts` | 130 net | 300 |
| Summary, question, copy, CSS | 170 | 60 |
| Speed: helpers, CI, bench | 110 | 30 net |
| E2E: `timeline-preserved`, `export-creator` | - | 330 |
| **Total** | **about 1,250** | **about 1,500** |

About 2,750 lines of code and tests, roughly a tenth of V4. Documents: this file, a short changelog and a short experiments file.

**Complexity: medium, and mostly filling seams V4 left.** The difficult parts are few: tuning checks 8 and 9, the restart inside the loop, and the cache's key. Section 21 has their ways out.
