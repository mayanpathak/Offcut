# v5changelog.md - record of every change made while building V5

**What this is.** A record of every change made while building V5: what was done, what differs from `v5implementation.md`, how it was checked, and what is still open. Entries are short (D-195) and in date order, oldest first. Every commit that changes the repository carries its entry. The two tables below are rewritten when an item opens or closes.

**References.** `PS §n` is a section of `product.md`, `TS §n` of `technicalspec.md`, `BP §n` of `buildplan.md`, `V4 CL` is `docs/v4/v4changelog.md`.

---

## Open items
| # | Item | Owner | Version |
|---|---|---|---|
| 1 | **Two V4 boxes only the founder can close** (V4 CL Entry 40): design review 2 and the five goldens, and whether the editor matches the reviewed screens. They do not block V5 | Founder | Any time |

## Known issues
| # | Issue | Where | Version |
|---|---|---|---|
| 1 | ~~**An export reads about 3 LU over the target.**~~ Closed in Entry 5: -11.4 LUFS before the fix, -14.3 after, read by `ffmpeg` on real exports | `offcut-dsp/src/loudness.rs` | P130 |
| 2 | ~~V4 CL known issue 33: a flake of `model-download.spec.ts`'s analytics poll on the hosted runner~~ Closed in Entry 13: no such failure in V5's four runs | `web/tests-e2e/model-download.spec.ts` | Closed |
| 3 | **V4 CL known issue 36 stays: `openEditor`'s wait for the drop zone on the hosted runner, after `page.clock.install()`.** D-236's `patient(30_000)` did not cure it. A first attempt failed in all four of V5's runs (41180419936, 41183840260, 41246761774, 41289198237; one or two cases each, not always the same) and passed on the retry, so CI is green. No other case of the suite shows it. Read for V6: whether the fake clock holds the capability check back there | `web/tests-e2e/model-download.spec.ts` | V6 |
| 4 | V4 CL known issue 37: `detect_scene` on a quiet D1, read at P136: median 255 ms, p90 291 ms, 10 ms over its line of about 245 ms. Stages nobody changed are 16-22% slower than on V3's day too. Not a V5 blocker (section 20); carried with these numbers | `docs/v5/experiments.md` | V9 |
| 5 | **Production `client_error` rows.** On 2026-10-29: `E_DECODE_VIDEO` at `preview` 4 times, within 71 s from 14:59:33 UTC, ten minutes after V4's deploy finished, which fits known issue 7 and cannot be proved from rows that hold no detail. `E_WORKER_CRASH` at `import` once, at 15:29:00, **still unexplained**. None since. P136 reads them again after the merge; a code that keeps appearing gets its own investigation | Production `analytics_events` | P136 |
| 6 | **`golden_frames` can abort when D1 is short of memory.** One `pnpm test` straight after two browser runs ended with "memory allocation of 2073600 bytes failed" in that test (exit `0xc0000409`), with two `rust-analyzer` processes holding about 7 GB of commit. The same tests, unchanged, then passed. Not seen in CI. If it comes back, run that test with `--test-threads=1` | `crates/offcut-render/tests/golden_frames.rs` | Watch |
| 7 | ~~**A clip's preview could fail at once in a fresh browser with `E_DECODE_VIDEO`.**~~ Closed in Entry 6: 6 of 14 fresh starts before the fix, 0 of 24 after | `web/src/workers/render/` | Fixed; reaches production with the next merge |
| 8 | ~~**`pnpm e2e` is load-sensitive on a busy D1.**~~ Closed in Entry 12: load, as it looked. At the default 6 workers on a quieter D1, 12 passed in 23.6 s, that case 9.0 s; in CI at `3145e7f` 12 passed, that case 1.9 s. The limit stays | `web/tests-e2e/landing.spec.ts` | Closed |

---

## Entry 0 - 2026-10-31 - `v5implementation.md` written (outside a prompt)

**What.** The plan for V5, at the founder's request of 2026-10-31, after reading BP §7, TS §12, §21, §27, §28, §40.9, V4's plan, changelog and hand-over, and the code at `0a8aa80`. One document holds the plan and the eight prompts (129 to 136), so V5 has no separate build guide or prompts file (D-195). The founder approved the speed rules and the five-run bench on the same day.

**Files.** `docs/v5/v5implementation.md` and this file are new. `docs/technicalspec.md` §5: three rows for `docs/v5/`.

**Found while reading, for the prompts named.** Known issue 1 above (P130). The cache key must use the worker's verified profile: with the page's unverified `planOf`, an expired Creator token would hit a stored 1080p file with no watermark (D-220, P134). `RENDERER_VERSION` exists nowhere in the code (D-223, P134). `export_done`'s `events_kept` counts a switched-off visual as kept (D-237, P135). Every fixture is encoded with `-bf 0`, so `ctts` needs a new one (D-212, P131). Dropping the same file again would hit the stored export unless the import clears `renderCache` (D-228, P134).

**Checked.** Documents only: nothing was built and no test was run. The loudness reading above is the one measurement. Not committed: this session was asked for the document only, and `v5-build` does not exist yet.

**Open.** Open items 1 to 3; known issues 1 to 4.

## Entry 0.1 - 2026-10-31 - `v5buildguide.md` and `coding-promptsv5.md` written (outside a prompt)

**What.** At the founder's request of the same day, after they said V4 is done: a build guide in `v4buildguide.md`'s structure (Phases 0 to 8, steps, a milestone per phase) and the prompts in `coding-promptsv4.md`'s structure (a Setup and 129 to 136). This replaces Entry 0's "no separate build guide or prompts file": D-195 is rewritten, and `v5implementation.md` §22 is now a one-table summary. The speed rules stand. Each phase is one prompt, and the guide's table "The problems V5 must fix" maps Entry 0's six findings and open items 1 to 3 to a step and a proof.

**Files.** `docs/v5/v5buildguide.md`, `docs/v5/coding-promptsv5.md` are new. `docs/v5/v5implementation.md`: the plan's corrections below. `docs/technicalspec.md` §5: two more rows for `docs/v5/`, and one reworded.

**Found while reading the code for these, and applied to the plan (§25 rows 20 to 25).**
- `a_1khz_sine_of_peak_minus_20_reads_minus_23` in `loudness.rs` breaks under the stereo fix, and `pink_noise_reaches_the_target` may. The plan said the tests run unchanged. Both now have a stated rewrite (§8, P130).
- With a shared clip, a reset's undo steps reach the next case through the 10 s analytics flush and fail its `toEqual`. `flushAnalytics` needs `page.clock`, which that page does not install. `resetClip` now sends the queue by `visibilitychange` (§6.3, P129).
- `undo` has no test id; the reset finds it by role. The clean state reads the stored edit rather than opening the Style tab.
- `check-hosts.mjs`'s `DEV_MARKERS` is an explicit list, so D-218's "already fails" was wrong: `devFailEncoder` joins it (P132).
- `sourceHashPrefix` is in the clip's record, not the clip store; a miss's "cached bytes" had no reader, so `heldBytes` is added; the answered set moves to `export-store.ts` so no new use-case edge is needed; `fakeExportDone` is added for the done capture.
- A `FAIL 11` names a tag's key, never its value: `testclips/v3testclip.mp4` carries a location.
- Checked and unaffected: the recognizer hears `pcm16` from the media worker, not the chain, and prosody is z-scored, so the stereo fix moves no transcript or event. `main` has no branch protection, so a skipped CI job blocks nothing.

**Checked.** Documents only: nothing was built and no test was run. `git ls-remote --tags origin v4` printed nothing on this date, so §1A box 1 stays open. Not committed: `v5-build` does not exist yet.

**Open.** Open items 1 to 3; known issues 1 to 4.

## Entry 1 - 2026-10-31 - Setup, first part: branch and baseline

**What.** `v5-build` from `origin/main`, holding the V5 documents and the TS §5 rows, pushed so the founder can open the draft pull request. **Differs from the guide:** Setup is two commits, not one. V4's closing entry waits for the `v4` tag and the founder's answers (open items 1 and 2), and the branch did not need to wait for them. Prompt 129 still does not start until both are done.

| Item | Value |
|---|---|
| Code baseline | `0a8aa80` |
| `pnpm check`, `pnpm fixtures`, `pnpm build` | Exit 0 |
| Rust | 646 passed, 0 failed, 4 ignored |
| Vitest | 175 passed |
| `pnpm e2e` | 12 passed |
| `pnpm e2e:media` | 77 passed + 1 skip, carried from V4 CL Entry 39: the code is the merge's |
| App shell | 337.3 kB gzip |
| `verify_mp4.py`'s tools | NumPy 2.5.3 on Python 3.13, ffmpeg 9.0.2 (§1A box 6) |

**V4's 35 open boxes, checked against the records on this date, for Entry 40.** 28 are shown by evidence and can be ticked when Entry 40 is written: the final gate of V4 CL Entry 39, this baseline, run 40889889605 (all four jobs, with the header check and `@smoke`, which reads `crossOriginIsolated`), `healthz` answering `0a8aa80`, `experiments.md` (14 decisions, none empty), 16 goldens, no untagged `TODO`, no `fetch(` outside the four files. `line_breaks.rs`'s rows are unchanged since `01f406f`: its diff removes only the header comment, an import and the `#[ignore]`. **7 wait for the founder:** `v4implementation.md` lines 1568 (tag; three styles on the deployed page), 1603 (the editor matches the reviewed screens), 1620 (the deployed walk-through), 1621 (the Neon rows), 1626 (design review 2 and five goldens; review 1 is recorded), 1630 (follows 1568) and 1631 (tag).

**Open.** Open items 1 to 3; known issues 1 to 4.

## Entry 2 - 2026-10-31 - Setup, second part: V4 closed, the pull request open

**What.** At the founder's word ("you do all these"), the agent pushed the tag `v4` on `0a8aa80` (`c3bc239`) and opened draft pull request 8. It ran V4's deployed walk-through and wrote V4 CL Entry 40: 32 of 35 boxes ticked, 3 left for the founder. `§1A` boxes 1 and 4 ticked. **Differs from D-194:** the tag and the pull request are normally the founder's; this applies to these two only. The tag `v5`, the merge and `gh pr ready` stay the founder's.

**Checked.** The walk-through passed, with what it did not assert named in Entry 40. Counts unchanged from Entry 1. CI of the pull request is not waited on; P129 reads it first (D-198).

**Open.** Open item 1; known issues 1 to 4. Prompt 129 is next.

## Entry 3 - 2026-10-31 - The Neon rows read by the founder

**What.** The founder ran the query: enum props only, so V4's box is ticked (V4 CL Entry 40), and two V4 boxes remain. Known issue 5 opened for the five `client_error` rows. Documents only. Committed without a push: it goes with P129's (D-198).

## Entry 4 - 2026-10-31 - P129: shared clip in review-edit, gates, docs-only CI skip

**What.** `resetClip`, `openShared`, `sendQueuedAnalytics` and `FakeApi.forget()`; `openClip` moved from the spec to `helpers/fixtures.ts`, so both open a clip one way. `review-edit.spec.ts` in two groups. A `changes` job in `ci.yml` holds the secret scan and decides whether the heavy jobs run; `ci` lost the gitleaks lines and `fetch-depth: 0`. `BENCH_RUNS`. D-236's two waits (known issues 2 and 3 stay open until V5's CI runs are read at P136). `e2e-media.yml`'s stale comment. No file under `web/src` changed.

**Differs from the plan** (its D-201, §6, §25 rows 26, 27, the guide and the prompts are corrected in this commit):
- **5 clip loads, not 4: case 14 has its own** (D-201 allows it). The first full run read its style cards at 26 ms, against 56-62 ms in V4. The store keeps the last cards, so on a shared clip they show at once and the 400 ms budget measured nothing. Alone it read 118 ms, and 58 ms in the final run.
- Undo is found inside `banner`: after a redo the toast offers an Undo of its own, which a page-wide locator would trip on after case 12.
- `--repeat-each=2` on cases 1 and 12 was not run. It starts a new worker per repeat, so it loads a new clip and never resets twice. A run of the file resets 12 times.
- The wait for toasts is `patient(30_000)`, not 10 s: it is an upper bound.
- A backslash in `--grep` is doubled by `pnpm` on Windows and matches nothing, so the guide's patterns now use `[.]`.

**Checked** (`d1`).
| Check | Result |
|---|---|
| `pnpm e2e:media review-edit.spec.ts`, final code | 18 passed (16 + 2 setup), 8.0 min; the file 6.6 min. Before: 16 clip loads at about a minute each, plus the cases |
| Titles and bodies of the 16 cases against `0a8aa80` | All identical, apart from the lines that open a clip or read `shared` (a script compared them) |
| Drill: no Undo loop in `resetClip`, cases 3 and 4 | Case 4 fails: "resetClip: every change is undone" |
| Drill: no `sendQueuedAnalytics`, cases 1 and 13 | Case 13 fails: received `"undo"` beside `caption_position` |
| The `diff` script | `code=true` with `ci.yml` changed; `code=false` for documents only |
| `BENCH_RUNS` | `--list` names 5 with 5 and 10 by default; `x` throws |
| Tier 3 | `pnpm check` exit 0; Rust 646 / 0 / 4; Vitest 175; `pnpm e2e` 12; frozen check empty. The first `pnpm test` aborted (known issue 6), and a re-run with no change passed |

**Not checked.** `ci.yml`'s new job has not run yet: `actionlint` is not on D1 and no YAML parser was at hand, so this push is its first test, read at P130's start (D-198). A docs-only pull request has not been seen to skip. `model-download.spec.ts`'s waits run in CI's `e2e-media`.

**Open.** Open item 1; known issues 1 to 6.

## Entry 5 - 2026-10-31 - P130: verifier checks 7-11, its self-test, and the stereo loudness fix

**What.** `Channel::DualMono` in `loudness.rs`: **a product change.** An export is about 3 dB quieter, and so is the preview, which matches it. Checks 7 to 11 in `verify_mp4.py`; `watermark_mask_720.png` and the tool that drew it; `test_verify_mp4.py`, 16 cases, also in `ci.yml`. `verifyMp4(…, source?)`, `expectAllChecks`, `OFFCUT_KEEP_EXPORTS`. `metadataProblems` and `tagsOf` moved into check 11. `pnpm verify:test`. Two TS §5 rows. **Known issue 1 closed.**

**The readings** (`d1`, real exports, by `ffmpeg`).
| Export | Loudness | Audio offset, start / end, correlation | White frames | Watermark | Bitrate |
|---|---|---|---|---|---|
| Creator, reference, **before the fix** | **-11.4 LUFS**, fails check 7, and only check 7 | 1.5 / -1.1 ms, 0.90 / 0.97 | 2, off by 1 | -0.05 | 8.3 Mb/s |
| Creator, reference | -14.3 LUFS | 1.3 / -1.7 ms, 0.95 / 0.99 | 2, off by 1 | -0.05 | 8.3 Mb/s |
| Free, reference | -14.3 LUFS | 1.3 / -1.7 ms, 0.95 / 0.99 | 2, off by 1 | **0.74** | 4.1 Mb/s |
| Creator, long pause | -14.3 LUFS | -0.3 / 0.5 ms, 0.97 / 0.99 | 2, off by 0 | -0.01 | 8.3 Mb/s |
| Creator, second recording | Passes all 11, `PASS 11` on a source that carries a location; read before `pcm` was fixed: -14.1 LUFS, 8.5 Mb/s, watermark 0.06 | | none in it | | |

The stereo test reads -11.29 LUFS with `Center` put back. No threshold of checks 7, 9, 10 or `MIN_ENVELOPE_CORR` moved. `pink_noise_reaches_the_target` did not fire and is unchanged. `a_1khz_sine_of_peak_minus_20_reads_minus_23` is rewritten by name to `..._reads_minus_20`.

**Differs from the plan** (§7, §25 rows 28, 29, the guide and the prompts are corrected in this commit).
- **Check 8 was wrong on phone clips, and the product was right.** The second recording's audio starts 304.9 ms after its video. Raw `ffmpeg` output drops that gap, so a correct export read -305 ms at both ends. `pcm` now fills it (`aresample=async=1:first_pts=0`), with a self-test case. No fixture has a late start, so only a real recording could show this.
- **Flash numbers:** 225 and 30, not 200 and 80, which missed real white frames. The reference clip has two white frames; the plan said none.
- `not compared`, not `not measured`, for a source with no white frame, which `expectAllChecks` would otherwise refuse on any such clip.
- The self-test overlays the mask (`geq`, `alphamerge`) rather than `blend`, whose `eq(B,128)` misses a converted mask. It also has a good creator file, a no-source case and a 3 LU over case beyond §7.6's list. Good files use `-fflags +bitexact`, or the source's `encoder` tag would fail check 11.

**Checked.** Self-test 16 of 16 in 73 s, its directory gone. `export-creator.spec.ts --grep "no token|another key|Creator export|Free-plan"`: 4 passed, on a keyed build with the fix. `--grep "no metadata"`: passed. `cargo test -p offcut-dsp`: 39 + 7 + 7 + 11 + 3 + 10 passed, 1 ignored. Drills: `Center` back fails the stereo test; hidden NumPy exits 2; `expectAllChecks` refuses no `--source`, a `FAIL`, and missing flashes. A `FAIL 11` prints no value (the self-test asserts it). The mask, enlarged beside a real Free frame, lines up. Saved exports and `zz-` files are deleted.

**Not checked.** The new `ci.yml` steps have not run: no push in this prompt (D-198). A Free export of a pause fixture. How a person hears it: P136.

**Found, not fixed.** Known issue 7, the most important result of this prompt. `trackSaves`' init script reads `document.body` before it exists, a harmless `pageerror` in the tests only. `device-bench.ts`'s comment says the recognizer hears the cleaned voice; it hears `pcm16`.

**Open.** Open item 1; known issues 2 to 7.

## Entry 6 - 2026-10-31 - Known issue 7 fixed: the session read while `attach_canvas` holds it (D-240)

**At the founder's request, between P130 and P131.** A product fix outside the plan's prompts, so it has its own commit, and three files leave the frozen list at their word.

**The cause, read from the browser, not inferred.** A temporary log gave `recursive use of an object detected which would lead to unsafe aliasing in rust`. `attach_canvas` is an async Rust call holding the session mutably across its await. Since V4's Prompt 124 (`a573d68`) the decoder starts when the scene is set, before the canvas exists, to speed up the first frame. A `dequeue` during the wait called `read_video_sample`; wasm-bindgen threw a plain `Error`; `VideoSource` kept it as a permanent `E_DECODE_VIDEO` with the detail `Error`. A person saw "The picture of this clip could not be read" on a good file. Attaching is slowest with a cold GPU, hence fresh browsers. The media suites' setup page opens the app first, which may be why none saw it (unproven).

**The fix.** `VideoSource.whileSessionHeld(step)`: `FrameDecoder.feed` does nothing while it runs, and feeding goes on after it from where it was, which loses nothing. A second way in was closed too: a full-range clip's restart calls `keyframe_at_or_before`, so it is deferred; the old decoder's frames in between are dropped by the restart. All five `attach_canvas` callers use it. The early start stays. No change to Rust, the protocol or any test's expectation.

**Checked** (`d1`).
| Check | Result |
|---|---|
| Fresh Chrome starts, keyed build | Before: 6 failed of 14, long-pause fixture. After: **0 failed of 24** (18 long-pause, 6 landscape) |
| `video-source.test.ts`, new | 7 passed, with a session that throws as wasm-bindgen does; one case reproduces the bug unguarded |
| Each half of the fix removed in turn | 2 cases fail each time; restored |
| `preview-sync`, `pipeline-preview`, `cancel`, `export-creator`, `review-edit` | **54 passed, 0 failed**, 27.3 min: first frame 87 ms (limit 500), seek 83 ms (300), drift 24.4 ms (80), style cards 87 ms (400) |

**Not checked.** The deployed site still has the bug until a merge. `model-download` and `rejections` were not re-run: they draw no preview. Other machines: D1 only.

**Open.** Open item 1; known issues 2 to 6.

## Entry 7 - 2026-10-31 - P131: `ctts`, video edit list, moov overflow guard tested

**What.** `add_video_sample`'s `frame` is now the frame a sample is shown as, and samples come in decode order: first frame 0 and a keyframe, each frame once, at most `MAX_REORDER = 16` ahead of its turn. `finalize` refuses a gap before it writes anything. When frames came out of order, `moov` writes `ctts` version 0 after `stts` and a video `elst` starting at the largest delay; otherwise neither. `ok_bframes.mp4`, the 26th fixture, and `mux_reorder.rs`. **G problem 5 closed.** No browser and no WASM rebuild, as the prompt says: nothing on the page changes until P132's build carries this.

**"Byte-identical" is proved against the old code.** SHA-256 of two files (first audio at 0, and with priming) was taken from the muxer before any change, and is pinned in `no_reordering_writes_no_ctts_and_no_video_edit`.

**Differs from the plan.** `mux_roundtrip.rs` changed in its one comment and also in its imports (`read_ctts`, `sha2`); no existing case's body changed. `a_reserve_with_under_8_bytes_left_is_refused` needs no search over many sizes: `moov` is an odd length, so a test's `avcC` is one byte longer, and then 8 bytes left is accepted, 4 is refused, 0 is accepted. That last one is what shows the refusal is this rule and not a plain overflow.

**Checked** (`d1`).
| Check | Result |
|---|---|
| `ok_bframes.mp4` | 60 frames: 2 I, 26 P, 32 B; `demux_fixtures.rs` accepts it |
| `real_b_frames_come_back_at_30_fps` | The file really is reordered; the demuxer's times and bytes match; `ffprobe` reads time base 1/30000, first frame at 0, every delta 1,000 ticks; `ffmpeg` decodes with no error line |
| `cargo test -p offcut-mp4` | 87 passed: 39 + 12 + 1 + 20 + 15 |
| Drills | `ctts` left out: `real_b_frames_come_back_at_30_fps` fails. `ctts` written with nothing reordered: `no_reordering_writes_no_ctts_and_no_video_edit` fails |
| `pnpm check`, Vitest 182, frozen check | Pass |

**Not checked.** No real encoder has given this muxer reordered frames: D1's never reorders, so `Writer.video` to `ctts` runs only on the fixture's samples here. Phone playback of a reordered file is V10's checklist. The new tests have not run in CI: P134 pushes.

**Open.** Open item 1; known issues 2 to 6.

## Entry 8 - 2026-10-31 - P132 (S-8): export over program time, `prog48`, ladder restart

**What.** The loop renders at program time, asks the scene what each frame shows (`source_at`) and asks the source for that. The audio it encodes is `prog48`, read from a new `prog48.f32` beside `out48.f32`. `runExport` is now a loop over `attempt`: an `E_ENCODE_VIDEO` empties the file, sets the bar to 0 and starts from frame 0 on the next supported entry; an `E_ENCODE_AUDIO` gets one more try on the same entry. A sink error, `E_GPU_LOST`, `E_MUX` and a cancel never restart. `failEncoderAt`, behind `__OFFCUT_DEV_FAULTS__`. `useTimeLeft` starts again when the frames fall back (D-238). `export-loop.test.ts`, 18 cases.

**REOPENED CONTRACT** `RenderWorkerApi.exportClip`: `out48` becomes `prog48`, page and worker in this commit (D-214).

**Differs from the plan** (§10.3, §25 row 31, the guide and the prompts are corrected in this commit).
- **`VideoSource.rewind()`, new, and a real bug avoided.** The plan said `frameAtBlocking(0)` is a jump back. Under 100 ms it is not: `#ask` takes it for the clock stepping back, then `frameAtBlocking` hands out a later frame as "a clip that starts later". A restart after a failure in the first three frames would have begun with the wrong frames, with the frame count still right, so no check would have caught it. A preview paused in the first 100 ms and then exported already had this. Each attempt now rewinds.
- **`pickVideoConfig` is removed,** not kept "for the capability check": `capability.ts` reads the ladder directly, so nothing would call it.
- **`pipeline-preview.spec.ts`'s OPFS case** lists a clip's files exactly, and gains `prog48.f32` and its size. The plan missed it; it is added to SAB rule 4.
- `DEV_FAULTS && takeEncoderFault(n)`, with `DEV_FAULTS` exported: a call alone stayed in the production bundle.
- The fakes throw `WorkerFailure`, and each failure comes once, so a loop that wrongly restarts fails its case instead of hanging the run. A first drill did hang, which is how this was found.

**Checked** (`d1`).
| Check | Result |
|---|---|
| `export-loop.test.ts` | 18 passed: accounting, identity, the seam (0, 1,000, 1,033, held 2,000 twice), `null`, audio padded and never trimmed, backpressure reaches its limit, every restart path, cancel; `liveFrames` 0 and encoders closed after each |
| Drills | The source asked the program time, no `sink.reset`, a restart on `E_GPU_LOST`, no `rewind`: each fails its named cases |
| `export-creator.spec.ts --grep "no token\|another key\|Creator export\|Free-plan"` | 4 passed on a keyed build with the muxer, the stereo fix and this: all 11 checks on real files |
| `pipeline-preview.spec.ts --grep "OPFS"` | Passed, `prog48.f32` the same size as `out48.f32` |
| `pnpm build`, `check-hosts` | Exit 0; no `devFailEncoder` or `DevFault` in `web/dist`; app shell about 337.7 kB |
| `pnpm check`, Vitest, frozen check | See the commit's gate: pass |

**Not checked.** A real encoder failure in a browser: P133's `failEncoderAt` cases. The wider media suites on this change: P133 and P134 run theirs, P136 the whole. None of this in CI: P134 pushes.

**Found, not fixed.** `readAudio` has no caller now; it stays for V6.2's rebuild of `prog48` (D-213).

**Open.** Open item 1; known issues 2 to 6.

## Entry 9 - 2026-11-01 - P133: `timeline-preserved` on both pause fixtures, all 11 checks

**What.** `timeline-preserved.spec.ts`, two serial groups, 9 cases: 1-5, 8, 9 on the long pause, and both of the sparse clip. It joins `MEDIA_SUITES`. `watchExport` and the `Window` fields it writes move to `helpers/fixtures.ts`. Only tests changed, so no build; the prompt ran on P132's keyed build. **BP §7's demo is now a test.** The real restart P132 could not show is shown here.

**The readings** (`d1`, real exports, by `ffmpeg`).
| Export | Loudness | Audio offset, start / end, correlation | White frames | Watermark | Bitrate | Frames, H.264 profile |
|---|---|---|---|---|---|---|
| Long pause | -14.3 LUFS | -0.3 / 0.5 ms, 0.97 / 0.99 | 2, off by 0 | -0.01 | 8.3 Mb/s | 600, High |
| Long pause, edited (a word, a visual off, Bold) | -14.3 LUFS | -0.3 / 0.5 ms, 0.97 / 0.99 | 2, off by 0 | -0.01 | 8.3 Mb/s | 600, High |
| Long pause, encoder failed at frame 30 | -14.3 LUFS | -0.3 / 0.5 ms, 0.97 / 0.99 | 2, off by 0 | -0.01 | 8.3 Mb/s | 600, **Main** |
| Sparse | -14.3 LUFS | 0.0 / 0.3 ms, 1.00 / 1.00 | 2, off by 0 | -0.03 | 8.4 Mb/s | 900, High |

The restarted file being Main is the proof it came from the ladder's next entry. D1's Chrome supports all four entries at both sizes (`isConfigSupported`, read once), so case 8 is not skipped. The edited file's frames differ from the unedited one's at 3 s and 15.5 s: identical readings are not a sign the edit was lost.

**Differs from the plan** (§17.5, §25 row 32 and the guide are corrected in this commit).
- **Case 1: the first word after the pause may start 40 ms early.** The recognizer hears it at 9,960 ms, as `fixtures/speech/README.md` already recorded. "Moved earlier" is asserted directly: no word starts before the one before it has ended.
- **Sparse: "lies inside a piece" judged starts, and allows a word that runs on.** Read: words at 980-2,000 and 2,020-5,880 ms, and all five phrases end past their piece's 300 ms. The recognizer stretches a phrase over room tone; nothing moved it. The prompt's error handling asks for this tightening. A word after a gap must start in a piece, so a word really moved earlier still fails.
- **`forgetProgress` moves too,** not only `watchExport`. Without it case 8 would pass on the fall from the last export's 100 to this one's first value, whatever the restart did.
- **Case 3 waits for "Play again"**, the button's name at the end, and also asserts the preview got to within 500 ms of the end.
- **Case 9:** `exports/` reads `["tmp"]`, since the start of an export sweeps the file before it, and the case stays last: the worker keeps the fault's unused attempts.
- **G 5.2** had only the header comment left: P130 did the rest.

**Checked** (`d1`).
| Check | Result |
|---|---|
| Each case alone, then the file once | All pass; the file 9 + 2 setup, 2.3 min |
| `export-creator.spec.ts --grep "Creator export\|progress\|another key\|no token"` | 4 + 2 setup passed, with `watchExport` moved |
| `pnpm check`, Vitest, frozen check | Exit 0; 201 passed; empty |
| Media listed | 87 with setup |
| Saved exports, `zz-` files | Deleted |

**Not checked.** Rust is unchanged and was not run. Other machines: D1 only. Nothing of P130 to P133 has run in CI: P134 pushes.

**Open.** Open item 1; known issues 2 to 6.

## Entry 10 - 2026-11-01 - P134 (S-9): render cache on the verified profile, the gate, `B_STORAGE_LOW`

**What.** `render-cache-repo.ts`: SHA-256 over canonical JSON of the ten inputs. `__RENDERER_VERSION__`, computed by `vite.config.ts`. `startExport` in §13's twelve steps. `forExport(room?)`, `freeBytes`, `B_STORAGE_LOW`'s words, out of `COPY_PENDING`. The export store's `cacheHit`, `blockedBy`, `saved(summary, answered)`, `answered`, `hasAnswered`. A blocked export says why in `export-status`. `importClip` clears the cache. `export-machine.test.ts`, `blockers.test.ts`, `start-export.test.ts`. `export_done` keeps its placeholders except `from_cache`, until P135. **G problems 2, 3 and 6 closed.**

**REOPENED CONTRACT** `RenderWorkerApi` + `exportProfile`, page and worker in this commit (D-220). `exportClip` still verifies the token itself.

**Differs from the plan** (D-222, §17.5, §25 row 33, the guide and the prompts are corrected in this commit).
- **Cases 8 and 9 each choose a new style first.** They follow exports of the same edit, so with the cache they would hit, no encoder would run, and the fault would never fire. The plan missed this.
- **`word_edits` is not why the JSON is sorted.** Its keys are integers, and JavaScript orders those itself, so the plan's drill would not have fired there. `event_overrides` has string keys, in the order they were made: that is what the case and the drill use.
- **`keyOf` gives no key while the clip is not `ready`.** A change that began between the click and the lock is not yet in the worker's scene: its file must not be stored under the new edit. Not in §13; a case holds it.
- **The repository under test is the real one, over a fake `db`,** not a `vi.mock` of it. So a row without its file, one of another size and one of a newer build are the repository's own behaviour.
- **Case 6 also makes a real change and undoes it, then expects a hit.** The plan had this in Vitest only, where nothing rebuilds the events or the plan. A real undo does, and it holds: the same key, the same bytes.
- **`cancel.spec.ts --grep "export"` cannot run:** the group is serial and the case before it processes the clip, so the button stays disabled until the 10 min timeout. The grep now names both. The first attempt cost that 10 min.
- `put` swallows by `.catch`, since the lint rule refuses an empty `catch`. `deliverStored` and `roomBlocker` are split out to keep `startExport` readable. The hit case's `sha256Of` is the helper `fixtures.ts` had, now exported.

**Checked** (`d1`).
| Check | Result |
|---|---|
| `timeline-preserved.spec.ts`, whole | 11 + 2 setup passed, 2.8 min. The hit's case 1.0 s, of which most is hashing two files |
| `export-creator.spec.ts`, the group to `expired` | 15 + 2 setup passed, 4.5 min: same token 1.4 s, forged, no room, expired verified as Free with the Free file's bytes and all 11 checks |
| `cancel.spec.ts --grep "pipeline cancel\|export cancel"` | 2 + 2 setup passed: `renderCacheCount` 0 |
| After the last code change (`put`) | Keyed build; `Words keep\|An edit leaves\|same edit`: passed |
| Drills | `planOf` in the key: the two `Verified profile` cases fail. `JSON.stringify`: 3 fail, among them the hit in the other order. No clear in `importClip`: the `Import` case fails. No `clear()` in the catch: 3 fail, among them the refused token. Each restored; files compared |
| `pnpm check`, `pnpm fixtures`, `pnpm test` | Exit 0. Rust 657 / 0 / 4 from that run's `pnpm test`; Vitest 327 |
| Production build | No `__RENDERER_VERSION__`, `devFailEncoder` or `DevFault` in `web/dist`; app shell 338.7 kB gzip |
| `check-copy-codes` | BLOCKER_CODES 5 of 8; pending 11 |
| Frozen check | Empty |

**Not green as written.** `pnpm e2e` at its default of 6 workers: 11 of 12, three runs out of three, on `the settings link leads to the table of what leaves the device`, past its 30 s. At `--workers=2`: **12 passed, 22.4 s**, that case 2 to 3 s; alone, 2.2 s. Nothing here changes that page. Known issue 8. CI's run of it at this push is what to read.

**Not checked.** This push's CI: read at P135's start (D-198). §11.3's "new deploy" and "the stored file deleted by hand" are Vitest cases, not browser ones, as the prompt says. Other machines: D1 only.

**Found, not fixed.** "Start over" (`dismissClip`) leaves the export file and its row until the next import clears them: harmless, as no other clip can reach them, and V7's `remove(clipId)` takes this over. `RENDERER_VERSION` hashes the three files D-223 names, so a change in `export-job.ts`, `opfs-sink.ts` or `video-source.ts` alone keeps old files valid: left as decided.

**Open.** Open item 1; known issues 2 to 6, 8.

## Entry 11 - 2026-11-01 - P135: `ChangeSummary`, the question, real `export_done` values

**What.** `summaryRows` and `ExportOutcome` in `ChangeSummary.tsx`, `PostExportQuestion.tsx`, `answer-post-export.ts`, `messages.export.summary` and `.question`. The outcome sits in the side panel's `.export` slot while the export is `done`. `editValues` is read when the export starts and goes into `export_done` on a render and on a hit, with the last V5 marker gone. `fakeExportDone`; the done state in `screens.spec.ts`. **G problem 4 closed.** P134's CI, run 41246761774 on `3145e7f`, was green in all three jobs, so there was nothing to fix first. No design skill was available; V4's tokens and primitives are used as they are.

**Differs from the plan** (the guide, the prompts and §4 are corrected in this commit).
- **The prompt's `--grep "Creator export|events:|summary and the question"` cannot pass:** `events:` counts the refused export's `export_started` and `export_failed`. It now names `another key` too. Likewise a run of `An edit leaves` needs `Words keep`, which finds its word.
- **`EditorLayout.tsx` changes, and the file list missed it:** §14.4 has the outcome said by the one live region, which is there. `outcomeAnnouncement` gives the block's words, then the thanks once answered.
- **`PanelSection` has no place for a close button,** so a wrapper holds it in the corner and takes back the slot's padding, leaving the primitive alone.
- **`close`, a case beyond §17.5,** last in the group: the outcome goes, the done line and the tabs stay. So the group grows by two cases, not one.
- **`answerPostExport` and `summaryRows`' zero are proved where they can fail:** the first in Vitest (the buttons are gone after one click, so no browser can answer twice), the second on the long-pause clip, which emphasizes no word: `Voice cleaned; 2 visual moments`, then `Voice cleaned; 1 visual moment` with its Big number off. Those rows are pinned in `timeline-preserved.spec.ts`.
- **A closed outcome stays closed on a hit of the same export,** which shares its id, as "keyed by `exportId`" says. The file still downloads; a new render shows it again.

**Checked** (`d1`).
| Check | Result |
|---|---|
| `export-creator.spec.ts`, the group | 17 + 2 setup passed, 4.2 min. Axe on the done state: no serious or critical violation. "With small edits": one `post_export_answer { answer: "small_edits" }`, the thanks line, and the hit after it sends none |
| `timeline-preserved.spec.ts`, whole | 11 + 2 setup passed, 3.0 min. Case 5: `word_edits` 1, `events_disabled` 1, one fewer kept, `style: "bold"`, all 11 checks |
| After the rows were pinned | `Words keep|Export is 20 s|An edit leaves`: 3 + 2 setup passed. `another key|Creator export|events:|summary and the question`: 4 + 2 setup passed |
| Drills | `kept` from `event.enabled` alone: 3 `real values` cases fail. **The first attempt did not fire:** the test's three events gave 2 kept and 1 disabled under either rule, so it now has four, which tell them apart. A row for a zero: `Export is 20 s` fails on `0 words emphasized`. `answered()` without the check: 2 cases fail. Each restored; files compared |
| The panel under the outcome, 1280 x 720, Captions tab, a temporary case | Scrolls: 423 px shown of 2,224, `scrollTop` 200 taken, its foot at the window's, the outcome does not move, the aside does not overflow. Deleted |
| `pnpm screens` | 18 PNGs. `done-1280x800.png`, `done-1920x1080.png` in `~/v5-review/`, looked at: on the grid, nothing clipped, the tabs usable below |
| `pnpm check`, Vitest | Exit 0; 336 passed, up from 327 |
| Production build, `check-hosts` | No `__offcutDev` or `fakeExportDone` in `web/dist`; app shell 339.6 kB gzip |
| `git grep "V5"` under `web/src`, `verify` | Two history notes (`export-machine.ts`, `video-source.test.ts`); no work to do. `crates` holds none, beyond two font files' bytes |
| Frozen check, temporary files | Empty; none |

**Not checked.** Rust is unchanged and was not run (Tier 2). A screen reader: the live region's words are read by a test, not heard. This commit's CI: P136 pushes. The founder's look: P136.

**Open.** Open item 1; known issues 2 to 6, 8.

## Entry 12 - 2026-11-01 - P136: bench, experiments, copy-back, final gate

**What.** `bench/results/d1-2026-11-01-portrait.json`, five runs, Docker stopped. `docs/v5/experiments.md`, a section per row of section 20. The copy-back of section 25 into TS §9 (C-7, C-9, C-10), §12.2, §12.4, §12.5, §13.3, §14.2, §18.2, §21.1, §21.3, §21.4, §23.1, §27.1, §27.2, §33, §39.3 items 14 and 24, §40.9 (S-8, S-9), BP §7, `verify/README.md`; TS §5 was kept right as V5 went. Section 24.1, "V5 as built". V4 CL known issue 37 reworded and carried to V9. **Known issue 8 closed.** No product code changed. This entry goes before the merge; the closing entry goes after the tag.

**Differs from the plan.**
- **Hits were read on the reference clip's 79 MB file,** not the pause fixtures: the larger file is the harder case. Five hits, 12 to 43 ms.
- **`detect_scene` missed by 10 ms on a quiet D1:** 255 ms against about 245 ms. Stages nobody changed were 16-22% slower than on V3's day too, so the machine has drifted. Section 20's rule: carried to V9, no V5 blocker.
- **94 passed against the estimate of 93,** the `close` case of P135.
- **"A docs-only pull request skips" cannot be seen on pull request 8,** which changes code. The closing entry's pull request shows it, so that box stays open until then.
- **Docker Desktop was stopped and started by the agent,** by `docker desktop stop` and `start`: no container was running. It is stopped now.

**The final gate** (`d1`, Tier 4, once).
| Check | Result |
|---|---|
| `pnpm install --frozen-lockfile`, `pnpm check`, `pnpm fixtures` | Exit 0 |
| Rust | 657 passed, 0 failed, 4 ignored (floor: 646 / 0 / 4) |
| Vitest | 336 passed in 13 files (floor: 175) |
| `pnpm e2e` | 12 passed, 23.6 s, at the default 6 workers |
| `python verify/test_verify_mp4.py` | 16 passed, 69.9 s |
| **`pnpm e2e:media`, keyed `pnpm build:e2e`, the one full run of V5** | **94 passed, 1 skipped, 30.6 min, none retried** (V4: 41.8-54.9 min, 22 runs). The skip is `REJECT_DECODE_UNSUPPORTED`, as in V4: Chrome decodes that fixture |
| Plain `pnpm build`, `check-hosts` | Exit 0; no `__offcutDev`, `fakeExportDone` or `devFailEncoder` in `web/dist`; app shell 339.6 kB gzip |
| `fetch(` outside the four files; goldens and snapshots against `0a8aa80`; frozen check; `zz-` files; key in the environment | None; none; empty; none; unset |

**Readings** (`docs/v5/experiments.md`). `render_encode` 30.9 s per 60 s (90), `mux` 21 ms (2 s). All four exports -14.3 LUFS; audio within 1.7 ms at a correlation of at least 0.95; watermark 0.74 Free, -0.05 to -0.01 Creator; 8.3-8.4 and 4.1 Mb/s. `cacheKey` 0.4-1.6 ms.

**Founder's steps next.** The pull request made ready and merged; one look at `~/v5-review/done-*.png`; one listen to `~/v5-review/listen-creator.mp4`, an export of the reference clip that stays on D1; the tag `v5`.

**Not checked.** Other machines: D1 only, as D-202 says. R1 stays unread. A hardware encoder that reorders.

**Open.** Open item 1; known issues 2 to 6.

## Entry 13 - 2026-11-01 - P136: CI read before the merge

**What.** Run 41289198237 on `06cd12f`, waited for: `changes`, `ci`, `e2e-media` all success; deploys skipped, as on a pull request. `pnpm e2e` 12 passed in 12.2 s, the settings link 1.6 s. Known issues read against V5's four runs: **issue 2 closed**, as no analytics poll failed; **issue 3 kept**, with its numbers in the table: `openEditor` fails a first attempt in every run and passes its retry. Three boxes of section 24 and section 23.3 that a script missed are ticked. Documents only, committed and **not pushed**: a push would run CI again, and the pull request's head stays the commit CI passed. It reaches `main` with the closing entry.

**Open.** Open item 1; known issues 3 to 6. The founder's merge, look, listen and tag.
