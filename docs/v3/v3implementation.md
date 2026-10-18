# v3implementation.md - Offcut V3 "Real ingest, clean voice, real detector": implementation plan

**Implements:** `buildplan.md` section 5 (V3), which is steps M1.1, M1.2 and M1.3 of `technicalspec.md` (TS) §28. Product references are to `product.md` (PS). "BP §n" is a section of `buildplan.md`; "V1 §n" of `docs/v1/v1implementation.md`; "V2 §n" of `docs/v2/v2implementation.md`; "V2 CL" is `docs/v2/v2changelog.md` (an "item n" there is a row of its tables "Open items" or "Known issues"). Decisions D-1 to D-72 are V1's and V2's; this document continues at D-73.

**Reader.** Whoever writes the V3 code, file by file. Everything needed to write a V3 file is in this document or in the TS section it names. V2 as built is the truth this plan builds on: where `v2implementation.md` and its changelog disagree, the changelog's "As built" wins, and this document cites it. Where the specs, V1, V2 and the build plan are silent or contradict each other, this document decides and says so in section 2; each decision is marked **(V3 decision)**.

**The one-sentence goal.** At the end of V3, any file a person drops is either accepted or refused with one of fourteen specific messages, the accepted clip's voice is cleaned to -14 LUFS without changing its length by one sample, and all four event kinds are detected by rules with a precision of at least 0.90 over at least 30 labeled transcripts, deployed and measured on D1 (BP §5).

**The second goal, equally important.** V4 and later versions must be additive. No V1, V2 or V3 file is restructured later: later versions fill in function bodies, add files and add rows to tables. Section 21 lists, file by file, what V4 to V7 add, and what V3 does today to make that possible.

---

## 0. Contents

1. Scope: what V3 is and is not
1A. Preconditions: what V3 starts from
2. Decisions this document makes (D-73 to D-112)
3. Tools, accounts, secrets, environment variables
4. V3 file tree
5. Build order (15 steps, each with its own check)
6. `offcut-mp4`: hardening against the fixture matrix
7. `offcut-dsp`: the voice chain and prosody
8. `offcut-text`: the complete number grammar, units, sentences
9. `offcut-detect`: four kinds, exclusions, resolution, re-detection
10. WASM binding crates: `audio_api.rs`, `redetect_sentence`
11. Code generation
12. Server: no changes
13. Web infrastructure: `CoreApi`, `RenderApi`, the pool row, the three repos
14. Workers: `audio.worker`
15. State and use-cases: `import-clip`, `run-pipeline`
16. UI, copy, styles
17. Fixtures, the labeled set and the corpus tooling
18. Root configuration, scripts, CI, hosting
19. Tests, case by case
20. Experiments, the precision gate, what V3 fixes
21. Hand-over: V4 and later
22. Exit checklist
23. Open references, spec issues found, and decisions to copy back
Appendix A. Per-file specifications in brief
Appendix B. Invariants V3 touches
Appendix C. Two per-file specifications written out
Appendix D. Named constants V3 introduces

---

## 1. Scope

### 1.1 In V3

| Area | What works at the end of V3 |
|---|---|
| Ingest and rejections | Every dropped file ends in one of two places: accepted (`clip_accepted`) or refused with one of the 14 codes of TS §15.3, each with its own message that names the limit and the advice (D-80), and one `clip_rejected { reject_reason }` event. `REJECT_NO_SPEECH` is sent from the pipeline. A refused clip leaves nothing in OPFS |
| Demuxer | `offcut-mp4` opens every `ok_*` fixture and refuses every `rej_*` fixture with its expected code; the three edit-list shapes of V2 grow to five (section 6, D-92) |
| Fixtures | `fixtures/gen_fixtures.sh` builds the 25 generated fixtures of section 17 into `fixtures/.gen/` (git-ignored) with a pinned ffmpeg: synthetic `ok_*` and `rej_*` clips, and five speech clips derived from the reference clip that is already on the asset host (D-76, D-77). `fixtures/manifest.json` names each with its expected outcome. Nothing new is recorded by a person |
| Voice | `run_chain`: high-pass, denoise, compressor, loudness to -14 LUFS, limiter at -1 dBFS, in 1 s blocks, length-preserving and delay-compensated (TS §18, §34.3). It runs in a new fourth worker, `audio.worker`, in parallel with transcription. The preview and the export play the cleaned voice |
| Prosody | `measure_prosody`: per-word energy and pitch z-scores within each sentence (TS §18.2, D-96). The NumberReveal energy bonus and KeywordPop become live |
| Text | The full grammar of TS §17.2: spelled cardinals to trillions, "point" decimals, k/m/b/"grand", USD/EUR/GBP/INR symbols and words, percent, multipliers, units of time, data size and count nouns; the 40-word sentence split |
| Detector | `detect` with all four kinds, the exclusions, the per-kind threshold filter and `resolve` (TS §17.3 to §17.5), number labels, list headers and items. `redetect_sentence` exists in Rust and in the render bundle; nothing calls it until V4 |
| Persistence | Step F of C-4 writes three IndexedDB records: `clips` (with `sourceHashPrefix`, D-81), `transcripts` (transcript and prosody), `edits` (the default `EditState`). No file name is stored |
| Feed | "Cleaning voice" while the chain runs; one "Found:" line per detected event of any kind, the FromTo line with both values ("Found: $2k to $20k", D-83) |
| Corpus tooling | `corpus/manifest.json`, `fetch_corpus.sh`, `run_corpus.ts` (a Playwright project, D-88): demux-and-export over whatever clips exist, and a `--wer` mode |
| CI | The Linux job installs a pinned ffmpeg and generates the fixtures before `cargo test` (D-94); the Windows media job runs the rejection suite's no-GPU cases |
| Analytics | `clip_rejected` starts (BP Appendix C); `stage_timing` gains `stage: "audio_chain"`; `pipeline_done` counts all four kinds |
| Second recording | `testclips/v3testclip.mp4`, the founder's recording of 2026-10-13 (D-99), goes through the whole pipeline on D1: its transcript is written into `fixtures/speech/README.md`, it is the second clip of the corpus, its transcript is a labeled file, and its export is checked to carry none of the source's metadata, its GPS location included (D-100) |
| Experiments | TE-12 (first run), TE-13, the precision gate of `precision_recall.rs`; E-3 and E-4 read again on the derived portrait clip and on the second recording; all on D1 (D-74). E-10 is carried to V9 (D-88) |

### 1.2 Not in V3

| Left out | Stand-in in V3 | Arrives |
|---|---|---|
| Drawing ListReveal, FromTo and KeywordPop | They are detected, stored, counted and shown in the feed; `events/mod.rs` still returns `Ok(None)` for them, so only NumberReveal is drawn (BP §5 "Not in V3") | V4 |
| Bold and Tech styles, the crop offset, the watermark | V2's stand-ins unchanged (V2 §1.2) | V4 |
| Review controls; the `redetectSentence` and `previewSeek` handlers of `render.worker` | The Rust function and its binding exist; the worker still answers `E_INTERNAL` (`NotImplemented`) | V4 |
| `NO_EVENTS_FOUND` copy for a clip with zero events | The preview shows with no event; the feed simply has no "Found:" line | V4 |
| Cancel buttons, cancel timeout, worker restart, `ErrorPanel`, retry, `client_error` | `messages.editor.failedStub` with "start over", as in V2. `E_DSP` keeps its pending copy entry | V4 |
| The measured value in a rejection message ("This clip is 2 minutes 14 seconds") | The message states the limit and the advice only (D-80) | Not planned; a spec issue (section 23) |
| `quota.ensureFree`, eviction, restore after reload, recent clips, the `events` record | The D-40 stand-ins stay; V3 writes `clips`, `transcripts`, `edits` and no `events` record (D-82) | V7 |
| Verifier check 7 (loudness) and checks 8-11 | Not implemented; `loudness_target.rs` holds the -14 LUFS promise inside Rust | V5 |
| E-10, the WER of the 20-clip accent set | The `--wer` mode is built and read on the reference clip alone, as a smoke reading. No accent set exists | V9 (D-88) |
| Real recordings of the TS §26 speech fixtures | Derived from the reference clip with ffmpeg, under the same names (D-76). A real recording later takes the same name with no code change | When the founder chooses; before V9 at the latest |
| The capability check refusing a software adapter (V2 CL, nineteenth push) | Unchanged | V8 (`capability.spec.ts`), if at all |

Nothing from TS §36 is built, including "temporary" versions. In V3 this matters most for the voice chain: no silence detection, no pause classification, no gap analysis, no crossfade, no time map, no threshold whose purpose is cutting (TS §18, INV-5, INV-10). The chain takes no word timings; `measure_prosody` takes them and returns numbers, never audio.

### 1.3 Journeys and use-case steps served

| Flow | Served in V3 | Missing until |
|---|---|---|
| J2 Drop a clip, sample clip (PS §10) | Full | - |
| J5 Validate | Full: 14 specific messages with limits and advice. The measured value is not stated (D-80) | Not planned |
| J6 Processing feed | Full: "Transcribing…", "Cleaning voice", one line per real detection of each kind | - |
| J7 Preview (AHA) | Captions, NumberReveal drawn, the cleaned voice heard. The three other kinds are not drawn; no zero-event message | V4 |
| J10, J11 Export and download | As V2, now with the cleaned voice and the stored records | V5, V6 |
| C-2 Drop and validate | Full except `quota.ensureFree` (V7): `forImport`, import, probe, validate, `clip_accepted` or `clip_rejected`, `clipsRepo.put` | V7 |
| C-4 Processing | Full: A, B1 and B2 in parallel, C, D, E, F. A failure ends in `failedStub` without `client_error` or retry | V4 |
| C-6 Review edits | `redetect_sentence` exists and is tested natively; no UI or worker handler | V4 |
| C-13 Restore | The records it needs exist; nothing reads them | V7 |

---

## 1A. Preconditions: what V3 starts from

Do not start S1 until every box is ticked.

- [x] 1. **V2 is closed.** `origin/main` is at `8a271a9` ("Merge pull request #3 from mayanpathak/v2-build"), deployed by `ci.yml`, and the founder checked the deployed app on 2026-10-15. Check: `git log -1 --format=%H origin/main`.
- [ ] 2. **The branch.** `v3-build` is cut from `origin/main` at `8a271a9` (D-75). That SHA is the V3 baseline: every frozen-file check compares with it. Check: `git merge-base v3-build origin/main` prints it.
- [ ] 3. **The test counts at the baseline** are taken on D1 and written into the first entry of `docs/v3/v3changelog.md`: at the last V2 entry they were 262 Rust (2 ignored), 92 Vitest, 12 Playwright (`pnpm e2e`) and 30 media cases (`pnpm e2e:media`). No count may fall during V3. Check: `pnpm test` and `pnpm e2e` on the baseline.
- [ ] 4. **The 22 GPU cases pass on D1 at the baseline** (D-72): `pnpm e2e:media` on a build with the test key, 30 of 30. Check: the run's summary line.
- [ ] 5. **The reference clip is reachable** as the source of the derived fixtures (D-76): `testclips/speech_scriptA_landscape_720p.mp4` on D1, and the sample clip at `SAMPLE_CLIP_PATH` on the asset host. Check: `fillAssetCache()` (`pnpm e2e:media --grep @no-gpu` runs it) leaves the clip in `fixtures/.cache/` with the SHA-256 its name carries.
- [ ] 5b. **The second recording is on D1** (D-99): `testclips/v3testclip.mp4`, 46,360,015 bytes, 78.918 s, 1280x720 H.264 Main, AAC-LC 48 kHz stereo, a variable frame rate averaging about 16.8 fps (1,329 frames), recorded 2026-10-13, with a `location` tag in its metadata. Check: `ffprobe -v error -show_entries format=duration,size -of compact testclips/v3testclip.mp4` prints those two values. Its SHA-256 is written into the first entry of `v3changelog.md` and into `corpus/manifest.json` at S1.
- [ ] 6. **ffmpeg on D1 is the pinned build and has the encoders the generator needs.** Version 9.0.2, the "essentials" archive already used by `e2e-media.yml` (V2 §22.5 as built), with `libx264` (including the High 4:4:4 profile), `libx265`, an AV1 encoder (`libaom-av1` or `libsvtav1`), `libvpx-vp9`, `libopus` (the audio of `rej_vp9.webm`), `libmp3lame`, and the `aac` encoder. Check: `ffmpeg -hide_banner -version` prints 9.0.2, and `ffmpeg -hide_banner -encoders` lists each name. If the essentials build lacks one, S1 records the encoder that replaces it in `v3changelog.md`; the fixture's expected outcome does not change.
- [ ] 7. **The V2 stand-in markers are where V2 §25.3 says.** Check: `git grep -n "V3: \|V7: " -- web/src/usecases` prints 5 lines (V2 CL item 37): the two V3 markers in `run-pipeline.ts`, the two D-40 markers, V1's `start-app.ts` marker.
- [ ] 8. **The V2 spec corrections are in `technicalspec.md`.** V2 §27 listed them and V2 §25.3 asked for them before V3. Check: TS §5 names `sps.rs`, `mux_boxes.rs`, `web/tests-e2e/tsconfig.json`, `media.setup.ts` and the six `docs/v2/` files (it does on 2026-10-15). The rest of V2 §27's "copy back" list that is not yet applied is applied at S1, in the same commit as `docs/v3/`.
- [ ] 9. **The three V3 dependencies pass `cargo deny`** on a scratch branch before S1 commits them: `nnnoiseless` (BSD-3-Clause), `ebur128` (MIT), and the `rubato` already present. Check: `cargo deny check licenses` with the crates added. A license outside the allowlist is added to `deny.toml` with its reason, or the crate is replaced (section 3.2).
- [ ] 10. **Python with NumPy and the verifier still work on D1.** Check: `pnpm verify` on a fresh export passes checks 1-6.

**Carried over from V2 (the founder's statement of 2026-10-15: "V2 is closed").** V2's code is merged and deployed. The items below were not written down when V2 closed. None of them blocks a V3 step.

| V2 item | Effect on V3 |
|---|---|
| The `v2` tag | None on code. Tag `8a271a9` as `v2` when convenient; S1 notes in `v3changelog.md` whether it exists |
| The written M0 gate decision (V2 §24.3, Prompt 60) | The founder's decision is "continue". S1 writes it into `docs/v2/experiments.md` as its closing entry, with the readings that exist: E-3 the founder's ~150 s (D-67) and D1's 62-96 s; E-4 29,641 ms on D1; E-1 inconclusive (the page was not announced). The entry says that S15 was not run |
| The S15 bench file `bench/results/d1-<date>.json` and the E-3, E-4 readings of S15 | V3's S12 runs the bench and commits the first `d1` file, which then holds `audio_chain` too (section 20). It is V3's baseline for V4 |
| TE-14 (memory, 90 s 1080p60, ten runs) | None in V3. Re-checked in V9 (BP §11). V3's bench records `peak_memory_bytes` for the portrait clip |
| V2 CL open items 2, 5, 6, 7, 8, 9, 10, 12 (signing key, merchant onboarding, Neon password and demo video, the unnamed test-mode API key in `.env.local`, branch protection, model license, the three-minute promise, a laptop like R1) | None on V3 code. Item 8 (branch protection) matters more now: a push to `main` deploys. Item 7: `.env*` stays git-ignored; S1's `.gitignore` edit must not touch that line, and no V3 script reads `.env.local` |
| V2 CL open items 4, 16, 17 (push after each prompt, 30 of 30 before a merge, mark PR #2 ready) | Item 4 and 17 ended with V2's merge. Item 16 continues as D-75's rule for V3's merge (30+ of 30+ on D1) |
| V2 CL known issues 10, 16 and 35, 36 (`boxes.rs` 392, `rpc.ts` 377 since Prompt 53, `video-source.ts` 399 lines) | V3 does not touch `boxes.rs`, `rpc.ts` or `video-source.ts`. `sample_table.rs` gains no line (D-92) |
| V2 CL known issue 12 (`proptest-regressions/` breaks the file-tree check) | V3 adds `proptest` to `offcut-text` and `offcut-detect`: delete the folder after a fixed failure, as in V2 |

---

## 2. Decisions this document makes

Numbering continues from V2's D-72. Each is the smallest choice that keeps V4 additive. BP §3 states the rule for schedule differences: where an implementation document and the build plan differ on which version creates a file or runs an experiment, the implementation document is the more precise one; such decisions are marked **(schedule)**. Numbers no experiment has measured are marked **(assumption, ID)**. A decision that touches something V1 or V2 froze (V1 §15.1, V2 §25.1) is marked **REOPENED CONTRACT**, with the cost on both sides.

| # | Decision (all **(V3 decision)**) | Why |
|---|---|---|
| D-73 | `docs/v3/v3implementation.md`, `docs/v3/v3buildguide.md`, `docs/v3/coding-promptsv3.md` (Prompts 61–90), `docs/v3/v3changelog.md` and `docs/v3/experiments.md` are added to the TS §5 tree at S1 (the guide and the prompts file exist already). A notes file, if written later, is added to the tree in the commit that creates it | Same pattern as D-46. `check-file-tree.mjs` fails on a file the tree does not list |
| D-74 | **D-69 holds for V3.** Every V3 gate and measurement that names R1 is read on D1 and recorded under the label `d1` (`BENCH_DEVICE=d1`). Thresholds do not change. For a budget defined for a 60 s clip, a clip of another length is read against the budget times its length over 60 s, as D-64 did | The founder's decision of 2026-10-09, confirmed for V3 on 2026-10-15. Cost, as in D-69: a pass on D1 does not show a pass on an 8 GB laptop with an integrated GPU; TE-13 in particular may pass on D1 and miss on R1 |
| D-75 | **V3 is built on the branch `v3-build`, cut from `origin/main` at `8a271a9`.** Commits are titled `V3 S<n>: <title>` (or `V3 P<nn>` if a prompts file is written). Pushing, opening and merging the pull request, and tagging, are the founder's. Before the merge into `main`: `pnpm e2e:media` 30+ of 30+ on D1 on a keyed build (D-72) | A push to `main` deploys (`ci.yml`). Same practice as V2 |
| D-76 | **No new recording is needed for V3; the speech fixtures stay out of git.** The five speech fixtures of TS §26 that V3 uses are derived by `gen_fixtures.sh` from the reference clip of D-64 (taken from `testclips/` or from the asset-host copy in `fixtures/.cache/`, D-39) and written to `fixtures/.gen/speech/` under their TS names. Section 17.2 gives each one's exact recipe: the portrait and landscape clips are the cut 0.960-60.960 s of the reference, scaled to 1080x1920 and 1920x1080 at 30 fps CFR, with a white frame at 5.000 s and at 55.000 s; the no-events clip is sentences 12-14 of the reference; the long-pause and sparse clips splice reference speech with generated room tone. `fixtures/speech/README.md` says, per clip, that it is derived and from which seconds. A real recording made later is stored in `testclips/` under the same name, `gen_fixtures.sh` prefers it, and no code changes | The founder's decision of 2026-10-15: speech fixtures stay out of git as D-39 kept the reference clip, and no new clip is recorded ("I ain't gonna record 30 videos"). The reference clip is already a real recording of Script A and already on the asset host, so CI can derive the same files. Costs: the portrait clip is a 405-pixel strip enlarged 2.67 times (soft, and cheaper to decode well than a real phone clip, though its decoded size is the real 1080x1920); the derived clips repeat one speaker, one room and one microphone; the long-pause clip's silence is generated room tone, not a recorded room; the 60 s cut ends before the $12k sentence, so the portrait clip carries four of Script A's five events |
| D-77 | **Every `ok_*` and `rej_*` fixture is synthetic**, made from ffmpeg's `lavfi` sources (`testsrc2` video, a 440 Hz `sine` tone, `anullsrc` silence), never from a recording, by `fixtures/gen_fixtures.sh` with ffmpeg 9.0.2, into `fixtures/.gen/` (git-ignored). `fixtures/manifest.json` (committed) lists every generated and derived fixture: name, recipe id, expected outcome (`accept` with its expected `ClipInfo` fields, or the `RejectReason`), and the tests that read it. The generator writes `fixtures/.gen/STAMP` with the ffmpeg version and the SHA-256 of the script and the manifest; a test that reads a fixture first checks that the stamp matches, and fails with the message "run pnpm fixtures" otherwise | TS §26 says derived and synthetic fixtures are generated and not committed, and calls the `ok_*` ones "the reference re-encoded". Demux and rejection tests need container properties, not speech, and a synthetic source is the same on every machine. The stamp keeps a stale `.gen/` from passing for a current one |
| D-78 | **`REJECT_DECODE_UNSUPPORTED` is tested with a real file**, `rej_h264_444.mp4`: H.264 High 4:4:4 Predictive (`yuv444p`), 1280x720, 30 fps, AAC, 2 s (section 17.2). It passes rules 1-10 and Chrome's `VideoDecoder.isConfigSupported` reports it unsupported (assumption, checked at S2 on D1 and on the Windows runner). If Chrome supports it, the fixture becomes `rej_h264_hi10.mp4` (High 10, `yuv420p10le`), checked the same way; if both decode, the case is marked `test.fixme` with the reason and the code path is covered by the inline unit test of `validate_probe` (V2 §6.9). Not a mock of `isConfigSupported` | TS §15.3 says "mocked `isConfigSupported`", but the call runs inside `media.worker`, where a Playwright init script does not run and the app has no test hook. A file Chrome cannot decode tests the real path |
| D-79 | **The oversize case builds its file inside the page.** The test makes a 1 MiB `Blob` and a `File` of 501 references to it (525,336,576 bytes, over `MAX_FILE_SIZE` = 500,000,000), named `clip.mp4`, and dispatches it on the drop zone in a `DataTransfer`. The worker refuses it before reading a byte (TS §15.2) | A 500 MB file in `fixtures/` or through `dropClip`'s request would cost disk and time on every run. A `Blob` of repeated parts is not copied in memory, and `importAndProbe` reads `file.size` only |
| D-80 | **A rejection message states the limit and the advice, not the measured value.** Each of the 14 messages is `{ title, body, action }`; the body interpolates the limit from the generated `LIMITS` (INV-15), for example "Offcut handles clips up to {maxSeconds} seconds right now. Trim it in your phone's gallery and drop it again." `MediaWorkerApi`, `CoreApi` and the clip store do not change for this | The founder's decision of 2026-10-15. TS §15.3 asks for "actual size" and "actual duration" and PS §9.4 gives "This clip is 2 minutes 14 seconds" as its example, but `{ rejected: RejectReason }` carries no value and both types are frozen (V2 §25.1). Cost: the copy is less specific than PS §9.4's example; recorded as a spec issue (section 23) to be copied back into TS §15.3 and PS §9.4 |
| D-81 | **`sourceHashPrefix` is the lower-case hex SHA-256 of the first `min(4 MiB, size)` bytes of the source file followed by its size as an unsigned 64-bit little-endian integer.** It is computed on the main thread by `clips-repo.ts::hashPrefix(file)` with `crypto.subtle.digest("SHA-256", …)` over `file.slice(0, 4 MiB)`, before `importAndProbe` transfers the file, and stored in the `clips` record. No WASM runs for it | TS §23.1 defines the field ("SHA-256 over the first 4 MiB of the source plus its length") without a byte layout or an owner. WebCrypto is native and asynchronous; hashing 4 MiB once is not media work (INV-17), and `persistence` may use it without a new import edge. V7's restore compares it to tell a re-dropped file from another |
| D-82 | **V3 writes no `events` record.** On a restore (V7) the events are recomputed by `detect` from the stored transcript and prosody, which gives the same events and the same ids (INV-7, `determinism.rs`); toggles live in `EditState.event_overrides`, keyed by `EventId`. No `events-repo.ts` is created | TS §23.1 lists an `events` store, but TS §5 has no file to write it and C-4 step F writes `transcriptRepo` and `editsRepo` only. Recomputing is cheaper than keeping two copies consistent. Open for V7 (section 23): the store either gets a writer then or is dropped from TS §23.1 |
| D-83 | **REOPENED V1 CONTRACT (`workers/protocol.ts`, `FeedLine`).** The `event_found` arm gains one optional field: `{ kind: "event_found"; eventKind: EventKind; display: string; displayTo?: string }`. For a FromTo, `display` is `from.display` and `displayTo` is `to.display`; for the other three kinds `displayTo` is absent. `messages.feed.eventFound.from_to` becomes `(display: string, displayTo: string) => string`, "Found: {display} to {displayTo}" | PS §10 J6 gives "Found: $2k to $20k" and V2 §20.3 left it to V3: one string cannot carry two values, and joining them in a use-case would put the word "to" outside `messages.ts` (INV-15). The founder's decision of 2026-10-15. Cost on the V1 side: one optional field in a frozen type, which no V2 reader breaks on; cost on the copy side: one signature changes, and `ProcessingFeed`'s `switch` passes the second value |
| D-84 | **The lexicons of `offcut-detect/src/config.rs` and the unit table of `offcut-text/src/units_lex.rs` are the exact lists of sections 8.4 and 9.1**, all (assumption, M1.3), re-checked by E-7. Each list is a `const` slice of lower-case `&str`, sorted, matched against whole effective tokens. No list is read from a file at run time | TS §17.4 names list nouns, change verbs, intensifiers, an emphasis lexicon, stop words, a stop-list of verbs and fillers, and TS §17.2 units, without giving one word. A detector cannot be written or labeled against without them |
| D-85 | **FromTo unit rules.** Two quantities are compatible when, after inheritance (a `Unit::None` takes the other's unit), they have the same `Unit` variant, or both are in one dimension group: time (`Milliseconds` to `Years`), data (`Bytes` to `Gigabytes`), or the same currency. "+0.15 same explicit unit" applies only when both carry the identical variant from their own words; "+0.05 inherits" when one is `None`. `Count { noun }` is compatible only with an identical noun. `FromTo.label` is `None` in V3 | TS §17.4 says "compatible units" without a definition, and `EventParams::FromTo` has a `label` that TS §17.4's "Derived fields" never derives. A label rule invented now would be tuned without a visual to judge it; V4 draws no label row when it is `None` |
| D-86 | **`redetect_sentence` scope**, replacing "next `cfg.list_window` sentences" of TS §17.3: sentence `s`; every sentence touched by an event of `prev` whose anchors intersect `s`; and every sentence whose `start_ms` is less than `sentences[s].start_ms + LIST_WINDOW_MS` (45,000) and greater than it. Events of `prev` whose anchors lie wholly outside that scope are kept as they are; `detect`'s steps 1-5 run over the whole transcript, the candidates whose anchors lie wholly inside the scope are kept, and `resolve` runs over them with the kept events fixed (section 9.2), so the rules of TS §17.5 hold for the whole clip | `LIST_WINDOW` is a time (45 s, TS §17.4), not a sentence count. A list announced in `s` may find its markers up to 45 s later, so those sentences must be re-read |
| D-87 | **The labeled set is written text, not recordings, and is written before the detector exists.** At least 30 files in `fixtures/labeled/`, format in section 17.5: 29 written transcripts, the Script A transcript as recognized (`fixtures/speech/README.md`), written out in the written form (`a01`, no emphasis marks: V2 stores no word times or prosody), and `a02` for the second recording, whose labels are frozen at S2 with `words` and `prosody` null and whose data is filled after S8 (section 17.5). Written transcripts are composed by the agent at S2, from short scripts in the voice of a technical creator, covering every kind at least 8 times, every exclusion of TS §17.4 at least twice, every edge case of TS §17.6 that a person would label the way the spec decides (the others are `rules_table.rs` rows), and 6 files with no event. Word times are synthesized by the test (section 19.9); emphasis is marked per word in the file (`"emph": true`), which the test turns into `energy_z = 2.0, pitch_z = 2.0` (the founder's decision of 2026-10-16 on section 23, item 27), every other word 0.0. The labels are fixed in the commit of S2, before S7 writes `from_to.rs`, `list.rs`, `keyword.rs`, `exclusions.rs` or `resolve.rs`. After that a label changes only with a `v3changelog.md` line naming the file, the label and the reason, and never to match the detector's output. The founder reads five files chosen by the agent at random and says whether each label is what a person would expect; that reading is an exit box | BP §5.2 asks for 30 hand-labeled transcripts and says the labeling "cannot be delegated to the detector being tested". The founder's decision of 2026-10-15: no 30 recordings; "do what is optimal". Text is what the detector reads; a recording only adds the recognizer's errors, which E-10 measures separately. Writing the labels before the rules is what keeps them independent of the detector. Costs: KeywordPop precision is measured on synthetic emphasis, not on prosody measured from speech; recognizer errors (V2's "editing 8 every evening") are under-represented; a single author writes both the labels and, later, the rules, which the freeze and the founder's reading only partly offset. E-7 (V10 onward) measures events kept on real use |
| D-88 | **(schedule) E-10 is carried to V9.** No 20-clip accent set exists and the founder will not record one now. V3 builds `corpus/run_corpus.ts` with its `--wer` mode and reads it on the reference clip alone (one clip against the README's expected text), as a smoke reading that is not E-10. The runner is a Playwright test file run as project `corpus` of `web/playwright.config.ts` (`testDir: "../corpus"`), as D-63 did for the bench. It reads each transcript back from the IndexedDB `transcripts` store, which V3 writes. Corpus clips are read from the local directory `OFFCUT_CORPUS_DIR`, are listed in `corpus/manifest.json` by id, SHA-256 and tags only, and are never uploaded anywhere. TE-12 is "inconclusive" when fewer than 10 corpus clips exist, and is re-checked at V9 (BP §11.1) | The pipeline runs only in a browser. Clips of other people are private: an asset host would publish them. BP §5.3 lists E-10 for V3 but its input set does not exist |
| D-89 | Two new third-party crates: **`nnnoiseless`** (RNNoise in pure Rust) in `offcut-dsp` for `denoise.rs`, and **`ebur128`** (BS.1770 integrated loudness with gating) in `offcut-dsp` for `loudness.rs`. Both pinned at their latest stable release on the day S1 adds them, versions recorded in `v3changelog.md`, both built for `wasm32-unknown-unknown` without threads. The denoiser's delay is a named constant `DENOISE_DELAY_SAMPLES`, (assumption, measured at S4 by `impulses_stay_aligned`) | V2 §25.2 named both. TS §18.2 asks for an "RNNoise-class" denoiser and "BS.1770 gating": writing either by hand is more code than V3 can verify. `ebur128` is also what the verifier's check 7 uses, through ffmpeg, in V5 |
| D-90 | **The C-4 order as built.** After step A, `pool.audio.runChain({ pcm48 })` starts at once, in parallel with the model wait (C-3) and with ASR (B1); its feed line "Cleaning voice" is pushed when it starts. `stage_timing audio_chain` is measured on the main thread around the `runChain` call (D-35). Step D (`measureProsody`) waits for both B1 and B2. A failure of B2 while B1 runs is held until B1 has settled and `asr.unload()` has run (INV-12); then the clip fails with the first failure in time order. A `REJECT_NO_SPEECH` from B1 waits for B2 to settle, then the clip is rejected; `measureProsody` is not called and the worker is sent nothing, so its `out48` stays in the worker's memory until its next `runChain` replaces it (section 23, item 25) | TS C-4 puts B1 and B2 in parallel and says no more about their joins. The audio worker keeps `out48` until `measureProsody` (TS §18.4), so a rejected clip must not leave a second clip's chain reading a stale buffer |
| D-91 | **REOPENED V2 CONTRACT (`pipeline-preview.spec.ts` cases, `bench/device-bench.ts`).** Three cases of `pipeline-preview.spec.ts` that V2 made true of a pipeline without a voice chain become false in V3 and are rewritten: "Feed" (a "Cleaning voice" line now appears), "Events" (`stage_timing` now includes `audio_chain`; `pipeline_done` counts the other kinds: for the reference clip `n_number` 2, `n_list` 1, `n_from_to` 1, `n_keyword` 0 or 1), and "Real detections only" (the expected list now holds all five events of Script A). `device-bench.ts`'s `STAGES` gains `"audio_chain"`, so its result file has six stages, and the result file is named `d1-<date>-<clip>.json` (section 19.19), not V2's `d1-<date>.json`. **Extended 2026-10-18 (the founder's decision, Prompt 81):** the five assertions of `model-download.spec.ts` that the feed's first line is "Transcribing…" now assert that its first two lines are "Cleaning voice" and "Transcribing…", in either order (G 9.5's rule; the chain starts beside the model wait, D-90); nothing else in those cases changes | V2 §25.2 said these files gain nothing in V3; they assert the V2 stand-ins. Cost on the V2 side: three cases, one constant and one file-name rule change in files V2 called partial or final; on the V3 side none |
| D-92 | **Two new source files, to stay under 400 lines (TS §29).** `crates/offcut-mp4/src/edit_list.rs` takes the edit-list resolution out of `sample_table.rs` (387 lines above its tests, V2 CL item 10) and adds two shapes. `crates/offcut-text/src/cardinals.rs` holds the spelled-cardinal grammar out of `numbers.rs`, which stays the only file with `parse_quantity` and `format_quantity`. Both are added to the TS §5 tree at S1 | V3's edit-list widening and the full number grammar cannot fit in the existing files. A split by function keeps each file's `[ONLY]` role: `numbers.rs` remains the only formatter and the only entry to parsing |
| D-93 | **In the rejection suite, the 13 cases that refuse a file before the pipeline carry the tag `@no-gpu`**, and so do the "start over" case and the `ok_vfr.mp4` accept case, which stop before the pipeline: 15 tagged cases run in CI's Windows media job (D-72). `REJECT_NO_SPEECH`, which needs a whole ASR pass, has no tag and runs on D1 with the other GPU cases | D-72: a hosted runner has no graphics processor and a clip takes over 20 minutes there |
| D-94 | **The Linux `ci` job generates the fixtures.** Before step 4 (`cargo test`), it installs a pinned static ffmpeg 9.0.2 build for Linux (archive URL and SHA-256 in `ci.yml`, refused on mismatch), restores `fixtures/.cache/` (the reference clip, by the hash in its name), and runs `pnpm fixtures`. The Windows media job runs `pnpm fixtures` before its Playwright step, with the ffmpeg it already installs. No fixture is cached between runs: the generator takes under a minute (assumption, S2) | The founder's decision of 2026-10-15. `demux_fixtures.rs`, `probe_rejections.rs`, `loudness_target.rs`, `duration_preserved.rs`, `chain_no_clicks.rs` and `prosody.rs` (the last two through their speech PCM cases) read generated files and run in step 4. Cost: about one more minute per `ci` run, and one more pinned binary to re-verify |
| D-95 | **`ui.rs` in TS §28 M1.2 is satisfied by the existing trybuild test still passing.** No compile-fail case is added: `offcut-types` cannot depend on `offcut-dsp`, and the unit types the chain uses (`Hz`, `Lufs`, `Dbfs`, `SampleCount`) are already covered by `bare_ms_rejected.rs`'s rule | M1.2's definition of done lists `ui.rs` with no stated change. Recorded as a spec issue (section 23) |
| D-96 | **Prosody parameters**, all (assumption, M1.2): energy = RMS in dB over the word's samples, floor -100 dB; pitch from 40 ms frames with a 10 ms hop, normalized autocorrelation over lags for 60-400 Hz, a frame voiced when its peak is at least `VOICING_MIN = 0.45`, the word's F0 the median of its voiced frames, no voiced frame giving no pitch; z-scores within the sentence over words that have a value (a word without pitch gets `pitch_z = 0.0`); a sentence of fewer than 3 such words, or zero variance, gives 0.0 for all its words | TS §18.2 gives the method and none of these numbers |
| D-97 | **`clip_rejected` is sent from two places**: `import-clip.ts` for the 13 codes `importAndProbe` returns, and `run-pipeline.ts` for `REJECT_NO_SPEECH`. Its one prop is `reject_reason` (the generated type). `client_error` is not sent in V3 (BP Appendix C: V4) | TS C-2 and C-4 mark both places; V2 sent neither |
| D-98 | **Fixture and corpus folders that are not in git**: `fixtures/.gen/`, `corpus/.clips/` and `corpus/.results/` join `.gitignore` at S1. `check-file-tree.mjs` lists files with `git ls-files --cached --others --exclude-standard`, so ignored paths are skipped with no change to it | Generated and private files must never be committed (D-76, D-88, D-112) |
| D-99 | **The founder's second recording, `testclips/v3testclip.mp4`, is the V3 test clip.** It is not committed, not uploaded to the asset host and not derived into any generated fixture: it stays on D1 (D-100). It is used five ways, all on D1: (1) at S2 the words the founder says were spoken, which the founder supplies then, are written into `fixtures/speech/README.md`, section "The V3 test clip as recognized"; at S8, once V3 stores the transcript in IndexedDB and detects all four kinds, the clip is taken through the pipeline once and its recognized transcript, numbers and detected events are added there (V2 has no way to read a transcript out of the app); (2) it is the second clip of the corpus (`corpus/manifest.json`, id `founder-02`, tags `webcam`, `vfr`, `low-fps`, `landscape`, `location-tag`), the reference clip being `founder-01`; (3) its transcript and measured prosody become the labeled file `fixtures/labeled/a02-v3testclip.json` (D-87, D-112); (4) E-3 and E-4 are read on it as well as on the derived portrait clip; (5) one D1-only case of `export-creator.spec.ts` exports it and checks the output's metadata (D-100). A test that needs it calls `helpers/fixtures.ts::v3TestClip()`, which returns `null` when the file is absent, and the case is skipped with the reason "testclips/v3testclip.mp4 is on D1 only" | The founder added it on 2026-10-13 for V3. It is a second take, session and room, and the first real clip with an average frame rate near 17 fps (a webcam in low light): a VFR case no synthetic fixture produces. Its words are not known to this plan; the README says what it holds once it has been heard and recognized |
| D-100 | **No metadata of a founder recording reaches anything the app or the tests publish.** `gen_fixtures.sh` writes every derived fixture with `-map_metadata -1 -map_chapters -1 -fflags +bitexact`, so no `creation_time`, `location` or `encoder` tag of the reference clip is copied (a byte search of the asset-host copy on 2026-10-15 found no location atom in it; Step 0.5 of the build guide re-checks it with `ffprobe`). `v3testclip.mp4` carries a GPS location and is never uploaded anywhere. One D1-only case of `export-creator.spec.ts` exports it and asserts with `ffprobe` that the output has no tag, at format or stream level, whose key contains `location` or `ISO6709` (case-insensitive; this covers `location`, `location-eng` and Apple's `com.apple.quicktime.location.ISO6709`), and no `creation_time`, `title`, `comment` or `encoder` tag that came from the source; and that the file holds no `udta`, `meta` or XMP `uuid` box carrying `©xyz` or `ISO6709`: an early slice of verifier check 11, which arrives whole in V5 | The muxer writes no `udta` and timestamps of 0 (V2 §6.8), so an export should already be clean; the case proves it on the one real file that carries a location. A location in a public bucket would publish where the founder records |
| D-101 | **Count nouns.** `units_lex.rs` has a list of count nouns (section 8.4); a number followed by one gets `Unit::Count { noun }`, with the noun as spoken. A count earns the "explicit unit" bonus of TS §17.4 only when its value is at least `COUNT_UNIT_MIN = 100` (assumption, M1.3). The list nouns of `config.rs` (reasons, ways, tips, …) are never count nouns, so "three reasons" stays a bare 3 for the list announcer | A literal reading of TS §17.4 makes "3 tools" score 0.80 and appear as a large number, which no labeler expects; "10,000 subscribers" must still score 1.0. Keeping list nouns out of the count list keeps the announcer form readable |
| D-102 | **Two ambiguous unit words.** `Unit::Times` is given only by an `x` written after digits (`3x`) or by the word "times" followed by a word of `COMPARATIVES` ("ten times faster"); "three times" alone is no unit. "pound" and "pounds" are not a currency word (weight is as likely); `£` and "gbp" are | Both rules protect precision against frequent spoken phrases ("I tried three times", "lost ten pounds"). (assumption, M1.3), re-checked by E-7 |
| D-103 | **REOPENED V2 CONTRACT (`web/eslint.config.js`, `MINTING`).** `persistence/clips-repo.ts` gets one minting entry, `{ brand: "Bytes", helper: "toBytes" }`, for `ClipRecord.sourceBytes`, which is `file.size`. The three repos throw `opfs.OpfsError` (kind `quota` for a `QuotaExceededError`, `io` for any other IndexedDB failure), so `run-pipeline.ts::failureOf` maps them to `E_STORAGE_QUOTA` / `E_STORAGE_IO` with no change | `ClipRecord.sourceBytes: Bytes` (TS §23.1) must be minted from a browser number, and `ClipInfo` has no size field. Reusing V2's error class keeps the use-case's mapping in one function. Cost on the V2 side: one more entry in a list V2 froze (D-59); on the V3 side none |
| D-104 | **A number never crosses a sentence boundary.** `normalize_transcript` parses each sentence's tokens separately, so a `NormalizedSpan` lies inside one sentence. FromTo's "one sentence" rule and the list announcer then need no extra check | TS §17.4 puts FromTo inside a sentence and says nothing about a quantity that would straddle two, which split recognizer words (D-42) could produce |
| D-105 | **List markers.** A marker is an ordinal word (`first` … `seventh`, `firstly`, `secondly`, `thirdly`), the phrase "number" + a cardinal 1-7, or a cardinal 1-7 in digits or words followed by `,` `.` or `:`; it counts only as the first effective word of a sentence, after any of the fillers of section 9.1. The announcer form needs markers 1 to `count` in order, every one within `LIST_WINDOW_MS` of the announcer's start. The marker-only form needs at least 3 consecutive ordinal words starting at `first`; its +0.20 of TS §17.4 is given when no gap between two consecutive markers exceeds `LIST_MARKER_GAP_MS = 15,000` (assumption, M1.3) | TS §17.4 defines neither what a marker is nor when the marker-only bonus applies. "the first time" in the middle of a sentence must not start a list |
| D-106 | **`Candidate` gains `parts: Vec<WordRange>`**, the word range of each quantity or marker the candidate is built from (NumberReveal: its span; FromTo: from and to; ListReveal: the announcer's count and every marker; KeywordPop: none). `exclusions::apply` inspects the parts. A FromTo's two `Quantity` values carry the unit after inheritance, and their `display` is formatted again with it ("90" becomes "90 minutes") | `exclusions.rs` must zero a FromTo whose ends are years (TS §17.6), and V2's `Candidate` keeps only the anchors. `Candidate` is crate-internal: adding a field changes no frozen contract |
| D-107 | **Derived text fields.** A NumberReveal label is the effective text of up to `NUMBER_LABEL_MAX_WORDS` (3) Word tokens after the span, in the same sentence, stopping at punctuation or at a word of `LABEL_STOP_VERBS`; `None` if empty. A ListReveal header is `"{count} {NOUN}"` with the noun upper-cased ("3 REASONS"). A `ListItem.text` is up to `LIST_ITEM_MAX_WORDS` (5) Word tokens after the marker, leading fillers removed, stopping at `.` `?` `!` `;` `:`, at a `,` once 2 words are taken, or at the next marker; `ListItem.at` is the start of the marker's first word | TS §17.4 gives the outline; Script A's expected labels ("editing by hand", "in sales") and header ("3 REASONS") fix the details |
| D-108 | **Pitch is measured on an 8 kHz copy.** `measure_prosody` decimates each word's samples by 6 with a 6-tap box filter, computes the normalized autocorrelation over lags 20 to 134 (400 to 60 Hz) in 320-sample frames with an 80-sample hop, refines the peak by parabolic interpolation, and takes z-scores of `log2(F0)` (assumption, M1.2) | At 48 kHz the autocorrelation over the reference clip's words costs about 6 x 10^9 multiply-adds, far over the prosody allowance (under 1 s, TS §8); at 8 kHz about 2 x 10^8. F0 lies under 400 Hz, so the decimation keeps what is measured |
| D-109 | **An edited number is read again.** `number::find`, `from_to::find` and `list::find` read quantities from the effective tokens of each sentence with `parse_quantity`, not from `Transcript.numbers`. A span whose words were edited yields the quantity of the new text (TS §17.6); a hidden word inside it yields none. This replaces V2's rule that skipped an edited number (V2 §9.3) | TS §17.6 asks for re-parsing; V2 marked it for V3 |
| D-110 | **The chain's binding takes no configuration.** `run_chain(pcm48, on_block)` runs `ChainConfig::default()`; `config.rs` is the one place where TE-13's fallback (`denoise: false`) is written. PCM crosses the WASM boundary by copy (in: `&[f32]`; out: a new `Float32Array`); between threads it is transferred, never copied (TS §31) | TS §34.3 passes `cfg` to `run_chain`, but `AudioWorkerApi.runChain` (frozen) has no field for it. A copy into and out of WASM memory is unavoidable and costs 2 x 17.3 MB for 90 s |
| D-111 | **Generated fixtures are deterministic and small.** Every recipe passes `-bf 0 -threads 1 -g 30 -fflags +bitexact -flags:v +bitexact -flags:a +bitexact -map_metadata -1 -map_chapters -1` and uses the lowest size that shows its property (320x180 where size does not matter). Two derived clips keep TS's names though their lengths differ: `speech_20s_noevents.mp4` is 14.4 s, `speech_30s_sparse.mp4` is 30.0 s as named (section 17.2). `rej_silent.mp4` is 5 s of digital silence; that the recognizer returns fewer than `MIN_WORDS` words for it is (assumption, S2), and if it hallucinates three or more, the fixture becomes 2 s long and that is recorded | The fixtures are made on every CI run; a change in them must be a change in the script. Keeping TS's names means no later suite renames a file |
| D-112 | **Whose words may be committed.** A transcript of a clip from anyone but the founder is never written to the repository, a result file or a log: `run_corpus.ts` writes numbers, ids, codes and tags only. The founder's two recordings may give committed labeled files (`a01-scriptA.json`, `a02-v3testclip.json`), which the founder confirms at the exit (section 22) | INV-2 governs the product; the same promise is owed to people who lend clips to the corpus |

---

## 3. Tools, accounts, secrets, environment variables

Only what is new or changed since V2 §3.

### 3.1 Tools

| Tool | Used for | Where | New or changed |
|---|---|---|---|
| `ffmpeg`, `ffprobe` 9.0.2, Windows "essentials" archive | `fixtures/gen_fixtures.sh`; the verifier; reading fixture properties in tests | D1, the Windows CI runner | Already there since V2; now also a generator |
| `ffmpeg`, `ffprobe` 9.0.2, a static Linux x86_64 GPL build | `pnpm fixtures` in the Linux `ci` job, before `cargo test` (D-94) | The Linux CI runner | New. Archive URL and SHA-256 are written in `ci.yml` at S11 (chosen at Step 0.8 of the build guide), as `FFMPEG_LINUX_URL` and `FFMPEG_LINUX_SHA256`. If no 9.0.2 build exists for Linux, the nearest 9.0.x build is taken and its version recorded in `v3changelog.md`; each machine's `STAMP` holds the version that machine used, and the version check of section 17.1 accepts any 9.0.x, with or without the `n` prefix of Linux static builds |
| `sha256sum` (Git Bash on D1, coreutils on the runners) | `gen_fixtures.sh`, `fetch_corpus.sh` | All three | Already present |
| `curl` | `gen_fixtures.sh` fetches the reference clip from the asset host when neither `testclips/` nor `fixtures/.cache/` has it | CI runners; D1 rarely | Already present |
| Python 3.13 with NumPy | The verifier, unchanged | D1, the Windows runner | Unchanged |
| Chrome stable with WebGPU | Every media case, the corpus runner, the bench | D1 | Unchanged |

No tool runs in the product that did not run in V2. FFmpeg stays a development tool (TS §36).

### 3.2 New dependencies

Pin each at its latest stable release on the day S1 adds it and record the version in `docs/v3/v3changelog.md` (TS §3).

| Package | Crate or workspace | Features | Note |
|---|---|---|---|
| `nnnoiseless` | `offcut-dsp` (through `[workspace.dependencies]`) | `default-features = false` | RNNoise in pure Rust (D-89). No feature that brings in a WAV reader or a C ABI. License BSD-3-Clause, added to `deny.toml`'s allowlist at S1 if absent |
| `ebur128` | `offcut-dsp` | `default-features = false`; the integrated-loudness mode only | BS.1770 loudness with gating (D-89). MIT |
| `serde_json` (dev) | `offcut-detect` | none | `precision_recall.rs` reads `fixtures/labeled/*.json`. Already a workspace dependency |
| `serde` with `derive` (dev) | `offcut-detect` | `derive` | The labeled-file structs of `precision_recall.rs`. Already a workspace dependency |
| `proptest` (dev) | `offcut-text`, `offcut-detect` | as in V2 | `numbers_prop.rs`; `determinism.rs` uses it for random transcripts |

No web dependency is added. `idb` (V1) serves the three repos. `insta` arrives with V4.

**Purity check.** `nnnoiseless` and `ebur128` must not bring in a crate that `PURE_CRATES` forbids (V2 §22.4: `web-sys`, `js-sys`, `wasm-bindgen`, `getrandom`, `rand`, `wgpu`). S1 runs `cargo tree -p offcut-dsp -e normal` and lists the tree in `v3changelog.md`. A forbidden crate behind a default feature is turned off; one that cannot be turned off is a reason to replace the crate (TS §18 contingency: denoise off), not to loosen the check.

### 3.3 Accounts: what each must hand over

| Account | You need from it in V3 |
|---|---|
| Asset host | Nothing new. The reference clip is already there as the sample clip (D-39); the derived fixtures are made from it and never uploaded (D-76). `v3testclip.mp4` is never uploaded (D-100) |
| GitHub Actions | Linux minutes for one more step of about a minute (D-94). The repository is public, so the minutes count against nothing (TE-10, V2) |
| Model source | Nothing: the model does not change in V3 |
| Founder | At S2: what was said in `v3testclip.mp4` (D-99). At the exit: the reading of five labeled files (D-87) and the confirmation of D-112 for the two founder transcripts |

### 3.4 Environment variables

| Variable | Where | Value | New or changed |
|---|---|---|---|
| `OFFCUT_FIXTURES_DIR` | Rust tests, Playwright helpers, `gen_fixtures.sh` | Directory of the generated fixtures. Default: `fixtures/.gen` under the repository root, found from `CARGO_MANIFEST_DIR` (Rust) or from the helper file's own path (TypeScript) | New, optional |
| `OFFCUT_REFERENCE_CLIP` | `gen_fixtures.sh` | Path of the D-64 clip. Default: `testclips/speech_scriptA_landscape_720p.mp4`, then `fixtures/.cache/<SAMPLE_CLIP_PATH>`, then a download (section 17.1) | New, optional |
| `VITE_ASSET_BASE_URL` | `gen_fixtures.sh` (download of the reference clip on a cache miss) | The asset host's base URL, as V2 sets it | Unchanged; now also read by the generator. Must be in the `env` of every CI job that runs `pnpm fixtures` |
| `OFFCUT_CORPUS_DIR` | `corpus/fetch_corpus.sh` | A local directory holding corpus clips named by their manifest id | New; D1 only |
| `CORPUS_MODE` | `corpus/run_corpus.ts` | `demux` (default), `export`, `wer`, `transcripts` (section 17.5) | New |
| `BENCH_CLIP` | `bench/device-bench.ts` | `reference` (default since V2), `portrait`, `v3testclip` | New |
| `BENCH_DEVICE` | Bench | `d1` (D-74) | Unchanged |
| `FFMPEG_LINUX_URL`, `FFMPEG_LINUX_SHA256` | `ci.yml` `env` | Written at S1 | New |
| `VITE_ENTITLEMENT_TEST_PUBLIC_KEY`, `E2E_CHROME_ARGS`, `E2E_TIMEOUT_SCALE`, `E2E_REAL_ASSETS` | As in V2 | Unchanged | - |
| Render variables, Vercel variables | Hosting | Unchanged. The server is not touched in V3 | - |

No secret is added.

---

## 4. V3 file tree

Marks: **NEW** = created in V3. **F** = final in V3. **P** = partial; section 21 says what is added later. **CHANGED** = a V1 or V2 file V3 edits, with exactly what is added. V1 and V2 files not listed are untouched. Every path is in the TS §5 tree except the two files of D-92 and the five `docs/v3/` files of D-73, which S1 adds to it.

```text
<repo root>/
├── Cargo.toml                          CHANGED  [workspace.dependencies]: nnnoiseless, ebur128 (D-89)
├── deny.toml                           CHANGED  license entries the two crates need; nothing else
├── package.json                        CHANGED  scripts "fixtures", "corpus" (section 18.2)
├── .gitignore                          CHANGED  + fixtures/.gen/, corpus/.clips/, corpus/.results/ (D-98)
├── .github/workflows/
│   ├── ci.yml                          CHANGED  pinned Linux ffmpeg; fixture cache; `pnpm fixtures` before cargo test (D-94)
│   └── e2e-media.yml                   CHANGED  `pnpm fixtures` before Playwright; the @no-gpu rejection cases run (D-93)
├── crates/
│   ├── offcut-mp4/
│   │   ├── Cargo.toml                  CHANGED  dev-dependency sha2 (section 6.4)
│   │   ├── src/lib.rs                  CHANGED  `mod edit_list;` (private)
│   │   ├── src/edit_list.rs            NEW F    (D-92) five edit-list shapes; `resolve_edits`
│   │   ├── src/sample_table.rs         CHANGED  step 5 calls `edit_list::resolve_edits`; the old `edits` body leaves; F
│   │   ├── src/probe.rs                CHANGED  hardening of section 6.3; F
│   │   ├── tests/demux_fixtures.rs     NEW F
│   │   └── tests/probe_rejections.rs   NEW F
│   ├── offcut-dsp/
│   │   ├── Cargo.toml                  CHANGED  + nnnoiseless, ebur128; F
│   │   ├── src/lib.rs                  CHANGED  module list; re-exports; DspError now used; F
│   │   ├── src/config.rs               NEW F    [ONLY] every DSP constant (TS §18.3); ChainConfig and its Default
│   │   ├── src/highpass.rs             NEW F
│   │   ├── src/denoise.rs              NEW F
│   │   ├── src/compressor.rs           NEW F
│   │   ├── src/limiter.rs              NEW F
│   │   ├── src/loudness.rs             NEW F
│   │   ├── src/chain.rs                NEW F    run_chain (TS §34.3)
│   │   ├── src/prosody.rs              NEW F    measure_prosody (D-96, D-108)
│   │   ├── tests/loudness_target.rs    NEW F
│   │   ├── tests/chain_no_clicks.rs    NEW F
│   │   ├── tests/duration_preserved.rs NEW F
│   │   └── tests/prosody.rs            NEW F
│   ├── offcut-text/
│   │   ├── Cargo.toml                  CHANGED  dev-dependency proptest
│   │   ├── src/lib.rs                  CHANGED  + `mod cardinals; pub mod units_lex;`; F
│   │   ├── src/tokenize.rs             CHANGED  wider `Digits` classification (section 8.1); F. `Token` and `TokenKind` do not change
│   │   ├── src/cardinals.rs            NEW F    (D-92) spelled cardinals to trillions, "point" decimals
│   │   ├── src/numbers.rs              CHANGED  the full grammar and display rules; F
│   │   ├── src/units_lex.rs            NEW F    currencies, units, count nouns, comparatives (D-101, D-102)
│   │   ├── src/sentences.rs            CHANGED  the 40-word split; F
│   │   ├── src/normalize.rs            CHANGED  numbers per sentence (D-104); F
│   │   ├── tests/numbers_table.rs      NEW F
│   │   ├── tests/numbers_prop.rs       NEW F
│   │   └── tests/sentences.rs          NEW F
│   ├── offcut-detect/
│   │   ├── Cargo.toml                  CHANGED  dev-dependencies serde, serde_json, proptest; F
│   │   ├── src/lib.rs                  CHANGED  the full `detect`; `redetect_sentence`; F
│   │   ├── src/config.rs               CHANGED  every field and every lexicon (D-84); F
│   │   ├── src/number.rs               CHANGED  labels, re-parse (D-109), `parts` (D-106); F
│   │   ├── src/from_to.rs              NEW F
│   │   ├── src/list.rs                 NEW F
│   │   ├── src/keyword.rs              NEW F
│   │   ├── src/exclusions.rs           NEW F
│   │   ├── src/resolve.rs              NEW F
│   │   ├── tests/rules_table.rs        NEW F
│   │   ├── tests/precision_recall.rs   NEW F
│   │   ├── tests/redetect.rs           NEW F
│   │   └── tests/determinism.rs        NEW F
│   ├── offcut-wasm-core/
│   │   ├── src/lib.rs                  CHANGED  `mod audio_api;`
│   │   └── src/audio_api.rs            NEW F    run_chain, measure_prosody (D-110)
│   └── offcut-wasm-render/
│       └── src/detect_api.rs           CHANGED  + redetect_sentence; F
├── web/
│   ├── eslint.config.js                CHANGED  one MINTING entry for clips-repo.ts (D-103)
│   ├── playwright.config.ts            CHANGED  rejections.spec.ts joins MEDIA_SUITES; project "corpus"
│   ├── src/copy/messages.ts            CHANGED  `rejections` (14 entries, D-80); `feed.eventFound.from_to(display, displayTo)` (D-83)
│   ├── src/persistence/clips-repo.ts   NEW P    hashPrefix, put, get (D-81); list, touch, remove in V7
│   ├── src/persistence/transcript-repo.ts  NEW P  put, get; remove in V7
│   ├── src/persistence/edits-repo.ts   NEW P    put, get; remove in V7
│   ├── src/wasm/load-core.ts           CHANGED  CoreApi gains runChain, measureProsody
│   ├── src/wasm/load-render.ts         CHANGED  RenderApi gains redetectSentence
│   ├── src/workers/protocol.ts         CHANGED  FeedLine.event_found gains `displayTo?` (D-83)
│   ├── src/workers/pool.ts             CHANGED  the `audio` row and client; F
│   ├── src/workers/audio.worker.ts     NEW F
│   ├── src/usecases/import-clip.ts     CHANGED  hashPrefix, clipsRepo.put, clip_rejected (D-97); P (V7: quota)
│   ├── src/usecases/run-pipeline.ts    CHANGED  B2 and D for real (D-90); step F writes the two repos; NO_SPEECH event; P (V4: client_error, retry)
│   ├── src/ui/components/RejectionPanel.tsx   NEW F
│   ├── src/ui/components/ProcessingFeed.tsx   CHANGED  passes displayTo; P (V4: cancel)
│   ├── src/ui/pages/EditorPage.tsx     CHANGED  RejectionPanel replaces the rejected stub; P (V4: ReviewPanel, ErrorPanel)
│   ├── src/ui/styles/components.module.css    CHANGED  classes for RejectionPanel
│   ├── tests-e2e/tsconfig.json         CHANGED  `include` gains `../corpus` (section 18.3)
│   ├── tests-e2e/helpers/fixtures.ts   CHANGED  fixturesDir, genFixture, dropFile, dropOversize, v3TestClip, readIndexedDb, rejectCodeOf (section 19.15)
│   ├── tests-e2e/rejections.spec.ts    NEW F
│   ├── tests-e2e/pipeline-preview.spec.ts     CHANGED  three cases rewritten (D-91); one D1-only case for the second recording
│   └── tests-e2e/export-creator.spec.ts       CHANGED  one D1-only case: an export of the second recording carries no metadata (D-100)
├── fixtures/
│   ├── gen_fixtures.sh                 NEW F    (D-76, D-77, D-111)
│   ├── manifest.json                   NEW F    every generated fixture and its expected outcome
│   ├── .gen/                           not in git: generated fixtures, STAMP, extracted PCM
│   ├── speech/README.md                CHANGED  derived recipes; Script A's V3 expected events; "The V3 test clip as recognized"
│   ├── labeled/SCHEMA.md               NEW F
│   └── labeled/*.json                  NEW      at least 30: w01 to w29 written, a01 and a02 recognized (D-87)
├── corpus/
│   ├── manifest.json                   NEW P    founder-01, founder-02; more clips as people lend them (V9)
│   ├── fetch_corpus.sh                 NEW F
│   ├── run_corpus.ts                   NEW P    --compare against the last result in V9
│   ├── .clips/                         not in git
│   └── .results/                       not in git
├── bench/
│   ├── device-bench.ts                 CHANGED  STAGES gains "audio_chain" (D-91); BENCH_CLIP
│   └── results/d1-<date>.json          data: the first committed bench file (section 20)
├── testclips/v3testclip.mp4            not in git: the second recording (D-99)
├── scripts/
│   ├── check-copy-codes.mjs            CHANGED  the 14 REJECT_* codes leave COPY_PENDING
│   └── check-external-facts.mjs        CHANGED  one line for the Linux ffmpeg archive (section 18.4)
└── docs/
    ├── technicalspec.md                CHANGED  §5 tree; the corrections of section 23
    ├── v2/experiments.md               CHANGED  closing entry: the M0 decision as the founder took it (section 1A)
    └── v3/
        ├── v3implementation.md         this file
        ├── v3buildguide.md             NEW      step-by-step order of work, commands and checks
        ├── coding-promptsv3.md         NEW      Prompts 61-90 for the coding agent
        ├── v3changelog.md              NEW      one entry per change made while building V3
        └── experiments.md              NEW      outcomes of section 20
```

Untouched in V3: every file under `server/`, `crates/offcut-types/`, `crates/offcut-api-types/`, `crates/offcut-scene/`, `crates/offcut-render/`, `crates/offcut-entitlement/`; `offcut-mp4/src/{reader,boxes,demux,sps,validate,mux,mux_boxes}.rs`; `offcut-dsp/src/resample.rs`; `offcut-detect/src/event_id.rs`; `offcut-wasm-core/src/{media_api,hash_api,text_api}.rs`; `offcut-wasm-render/src/{lib,session,profile_api,mux_api}.rs`; `web/vercel.json`; `web/src/gen/*`; `persistence/{schema,db,opfs,entitlement-repo}.ts`; `net/*`; `analytics/client.ts`; `models/*`; `platform/*`; every `state/` file; `usecases/{start-app,control-preview,start-export}.ts`; `workers/{rpc,media.worker,asr.worker,render.worker}.ts` and every file under `workers/media/`, `workers/asr/`, `workers/render/`; every UI file not listed above; `verify/*`; `render.yaml`; every script not listed above.

Three of the untouched files are worth naming because a reader might expect a change: `state/clip-store.ts` already holds `prosody` and the rejection reason (V2 §18.2), so V3 adds no action; `workers/render.worker.ts` keeps answering `redetectSentence` with `E_INTERNAL` until V4 wires the handler (section 1.2); `persistence/schema.ts` already declares the three record types V3 writes (V1).

---

## 5. Build order

Fifteen steps. BP §0 gives V3 three days (Mon 2 to Wed 4 Nov 2026); the build is ahead of that calendar (V2 closed on 2026-10-15), so the day numbers below count from the real start, and the plan takes eight working days. Do not start a step while the previous step's check fails. Order: contracts, fixtures, pure crates (the risky one first), bindings, worker, persistence and use-cases, UI, tests and CI, measure, deploy. The two experiments that can change the plan are reached early: TE-13 natively on day 2 and in the browser on day 4; the precision gate as soon as the detector exists, on day 6.

| # | Day | Step | Produces | Check |
|---|---|---|---|---|
| S1 | 1 | Contracts and setup | Branch `v3-build` (D-75); `docs/v3/` (D-73); the TS §5 tree with `edit_list.rs`, `cardinals.rs` and `docs/v3/*`; the closing entry of `docs/v2/experiments.md`; workspace dependencies (D-89) with `cargo tree` recorded; `deny.toml`; `.gitignore` (D-98); the `MINTING` entry (D-103); `FeedLine.displayTo`, the `from_to` copy signature and the `ProcessingFeed.tsx` call that passes the second value (D-83: the signature does not compile without its caller); `corpus/manifest.json` (D-99); the SHA-256 of both founder clips recorded | `pnpm check` green; `cargo deny check` green; test counts equal to the baseline |
| S2 | 1-2 | Fixtures and labels | `gen_fixtures.sh`, `manifest.json` (section 17); `pnpm fixtures` script; `fixtures/speech/README.md` derived recipes and the founder's words for `v3testclip.mp4` (D-99); `labeled/SCHEMA.md`, the 29 written labeled files, `a01` and `a02` (labels only, data null), committed before any detector file changes (D-87); `rej_silent.mp4` and `rej_h264_444.mp4` checked in Chrome (D-78, D-111) | `pnpm fixtures` twice in a row gives byte-identical files (`sha256sum fixtures/.gen/*`); every manifest entry exists; `ffprobe` of each derived speech clip shows the duration and size of section 17.2 and no `location` or `creation_time` tag |
| S3 | 2 | `offcut-mp4` hardening | `edit_list.rs`; `probe.rs` items of section 6.3; `demux_fixtures.rs`; `probe_rejections.rs` | `cargo test -p offcut-mp4`; every `ok_*` opens, every `rej_*` gives its reason; `sample_table.rs` is under 400 lines above its tests |
| S4 | 2-3 | `offcut-dsp` | `config.rs` … `prosody.rs`; four test files; **native TE-13 timing**: `cargo test -p offcut-dsp --release -- --ignored chain_timing` prints the chain's time on the derived portrait PCM | `cargo test -p offcut-dsp`; the native time is written to `docs/v3/experiments.md`. Native over 2 s for 60 s of audio: stop and read section 20.2 before S6 |
| S5 | 3 | `offcut-text` | `tokenize.rs`, `cardinals.rs`, `numbers.rs`, `units_lex.rs`, `sentences.rs`, `normalize.rs`; three test files | `cargo test -p offcut-text`; the V2 tests of the crate still pass unchanged, except the one V2 case that asserted "ten million" stops at "ten" (section 8.6) |
| S6 | 4 | Audio in the browser | `audio_api.rs`; `CoreApi` additions; `audio.worker.ts`; the pool row; a temporary console call that runs the chain on the reference clip in `pnpm dev`. **TE-13 in the browser** on D1 | `pnpm build:wasm`; `tsc --noEmit`; the temporary call prints `out48.length === pcm48.length` and a loudness within ±1 LU of -14; the chain time is recorded. Over 4 s for 60 s of audio on D1: take TE-13's fallback (section 20.2) before S7. Remove the call |
| S7 | 4-6 | `offcut-detect` | `config.rs`, `number.rs`, `from_to.rs`, `list.rs`, `keyword.rs`, `exclusions.rs`, `resolve.rs`, `lib.rs`; four test files; `detect_api.rs` with `redetect_sentence`; `RenderApi` addition | `cargo test -p offcut-detect` with `precision_recall.rs` at 0.90 or above. **Below 0.90: stop and read section 20.3**: the threshold sweep decides, labels do not change (D-87) |
| S8 | 6 | Persistence and use-cases | The three repos; `import-clip.ts`; `run-pipeline.ts`; one run on D1 of each derived clip and of `v3testclip.mp4`, their recognized transcripts and events written to the README's "as recognized" sections (D-99, section 17.7) | `tsc --noEmit`; ESLint clean; `vitest run`; in `pnpm dev` the reference clip reaches `ready`, the feed shows "Cleaning voice" and four or five "Found:" lines (the KeywordPop depends on prosody), and DevTools shows the three IndexedDB records |
| S9 | 6-7 | UI and copy | `messages.rejections`; `RejectionPanel`; `EditorPage`; styles; `check-copy-codes.mjs` | `pnpm check` green (the copy check now sees 14 rejections with copy); in `pnpm dev` dropping `rej_4k.mp4` shows the 4K message and DevTools shows one `clip_rejected` |
| S10 | 7 | E2E | `rejections.spec.ts`; the helpers; the D-91 rewrites; the D1-only cases of D-99 and D-100 | `pnpm e2e:media` on D1 on a keyed build: every case green, the count at least 30 + 16 + 2 (V2's media cases, the rejection suite, the two D1-only cases of the second recording) |
| S11 | 7 | CI | `ci.yml`, `e2e-media.yml`, `playwright.config.ts`, `package.json`, `check-external-facts.mjs`; the corpus files and the bench change of section 17.6 and 19.19 | The pull request's `ci` and `e2e-media` jobs are green; the Linux job's log shows `pnpm fixtures` and the six fixture-reading Rust test files running, none skipped |
| S12 | 8 | Measure | `pnpm bench:device` with `BENCH_CLIP=portrait` and with `BENCH_CLIP=v3testclip`; TE-12 with `pnpm corpus`; `a02`'s words and prosody filled by `CORPUS_MODE=transcripts` (labels unchanged); the WER smoke reading; the five-file reading by the founder (D-87) | Section 20's records written; the bench file committed |
| S13 | 8 | Spec copy-back | The corrections of section 23 in `technicalspec.md`, `buildplan.md`, `product.md` | `node scripts/check-file-tree.mjs` green; every item of section 23's last table done or listed as open |
| S14 | 8 | Merge and deploy | Pull request ready; `pnpm e2e:media` on D1, all green (D-72); the founder merges | `ci.yml` green on `main`; the deployed page refuses `rej_4k.mp4` with its message |
| S15 | 8 | Tag | Section 22 ticked | Tagged `v3` |

**What can run in parallel.** S2's fixture script and its labeled files are independent of each other: write the labels first, because they must be frozen before S7 begins. S3 (demuxer), S4 (DSP) and S5 (text) are three pure crates with no edge between them; any order works, but S4 goes first because TE-13 can change the chain's design. S7 needs S5. S6 needs S4 only. S8 needs S6 and S7. If client work is blocked, the V6 server work still depends only on V1 (BP §0).

**If S4 or S6 misses the chain budget, stop and read section 20.2.** **If S7's precision is below 0.90, stop and read section 20.3.** Both have a pre-agreed action (TS §18 and §17 contingencies); neither is a reason to change a label or a test.

**What each step must not do.** No step edits a labeled file after S2 without a changelog line (D-87). No step adds a `TODO` without a version tag. No step touches a file of the "untouched" list of section 4; the frozen-file diff against `8a271a9` at each step lists only the files of section 4.

---

## 6. `offcut-mp4`: hardening against the fixture matrix

**Purpose** (TS §6). Unchanged from V2 §6: parse MP4/MOV, expose samples and codec configs, probe, validate, mux. V3 changes no public signature of the crate. It moves the edit-list resolution into a file of its own, widens the shapes it accepts, hardens the probe against the generated fixtures and the two founder recordings, and adds the two integration test files BP §5.1 A names.

**Crate rules** (TS §2, §7, V2 §6). Depends on `offcut-types`, `thiserror`; dev: `proptest`. No `web-sys`, `js-sys`, `wasm-bindgen`, `wgpu`, clock, randomness or global state. Non-test code has no `unwrap`, `expect`, `panic!` or slice indexing. The integration tests read files from `fixtures/.gen/` with `std::fs`, which is allowed in `tests/` only.

**Never.** Decodes or encodes a sample. Reads a whole file into memory in non-test code. Allocates from an untrusted count without first bounding it by the file size (D-60). Holds user-facing text. Logs. Returns a `RejectReason` other than through `validate_probe`, `NotIsoBmff` or `Fragmented` (V2 §6.1).

### 6.1 `edit_list.rs` (D-92)

```rust
// crate-private
pub(crate) struct EditPlan { pub edit_offset: i64, pub presented: i64 }      // both in media timescale ticks
pub(crate) fn resolve_edits(
    edits: &[Edit],             // `sample_table::Edit` as V2 reads it: { segment_duration, media_time }
    media_timescale: u32,
    movie_timescale: u32,
    media_duration: i64,        // sum of the stts runs, in media ticks
) -> Result<EditPlan, ContainerError>;
```

`sample_table.rs`'s private `edits()` keeps its signature and becomes three lines: sum the `stts` runs (with the V2 overflow checks), call `resolve_edits`, return `(plan.edit_offset, plan.presented)`. The two helpers `to_media` and `shown` move with it. `Edit` stays in `sample_table.rs` and keeps its two fields: V3 does not read `media_rate`, so a dwell edit (rate 0) cannot be told apart and is accepted as a media edit (section 23, item 15).

Supported shapes, in the order they are tried. "Media edit" is an entry with `media_time >= 0`; "empty edit" is an entry with `media_time == -1`. A `segment_duration` of 0 on the last media edit means "to the end of the media" (V2).

| # | Shape | `edit_offset` | `presented` | Since |
|---|---|---|---|---|
| E0 | No `elst` | 0 | `media_duration` | V2 |
| E1 | One media edit | its `media_time` | its duration in media ticks | V2 |
| E2 | One empty edit, then one media edit | `media_time - lead`, where `lead` is the empty edit's length in media ticks | the media edit's duration | V2 |
| E3 | Two or more media edits, each starting where the previous one ends in media time (`media_time[k+1] == media_time[k] + duration[k]` within 1 media tick), optionally after one leading empty edit | from the first media edit, as E1 or E2 | the sum of the media edits' durations | **V3** |
| E4 | Any of E1-E3 followed by exactly one trailing empty edit | as without it | as without it: the trailing empty edit shows nothing after the end and is ignored | **V3** |
| - | Anything else: two empty edits in a row, a gap or an overlap between media edits, an empty edit in the middle, a negative `media_time` other than -1 | - | - | `Unsupported("elst")`, which `validate_probe` never sees: `media_api.rs` maps it to `REJECT_CORRUPT` (V2 §6.1) |

**Bounds** (D-60). At most `MAX_EDITS = 16` entries are examined; a longer list is `Unsupported("elst")`. Every addition is checked; an overflow is `Malformed("elst")`.

**Why only these.** E3 is what an encoder writes when it joins two recordings without re-encoding, and what some phone apps write after an in-app trim; E4 is written by some screen recorders. Both appear in the corpus a technical creator brings (TS §15.4, TE-12). A shape outside this table is recorded by TE-12 with its count; a shape that occurs in 2 or more corpus clips is a V9 decision.

**Inline tests (`edit_list.rs`).**

| Case | Expect |
|---|---|
| E0: no edits, media 90,000 ticks | `{ 0, 90_000 }` |
| E1: `[{ duration 3,000 movie ticks at 1,000/s, media_time 2,000 }]`, media timescale 30,000 | `{ 2_000, 90_000 }` |
| E2: `[empty 500 ms, media media_time 0]`, media timescale 1,000 | `{ -500, … }` |
| E3: three contiguous media edits of 1 s each from `media_time` 1,000 at 1,000 ticks/s | `{ 1_000, 3_000 }` |
| E3 with one tick of mismatch between edits 1 and 2 | Accepted |
| E3 with two ticks of mismatch | `Unsupported("elst")` |
| E4: E1 then a trailing empty edit | Same as E1 |
| Two empty edits in a row | `Unsupported("elst")` |
| 17 contiguous media edits | `Unsupported("elst")` |
| `segment_duration` 0 on the last media edit, `media_time` 1,500 of 9,000 | `presented` 7,500 |
| A sum that overflows `i64` | `Malformed("elst")` |
| `movie_timescale` 0 | `Malformed("mvhd")` (as V2) |

### 6.2 `sample_table.rs`

No change of behaviour beyond the call of 6.1. The V2 inline tests "one media edit of two frames presents the first frame at zero" and "two media edits are unsupported" are kept: the second is still true, because its two edits are not contiguous (`media_time` 0 and 0). If either needs an edit to stay true, that is a sign the shape table is wrong, not the test.

### 6.3 `probe.rs`: the hardening list

V2 §6.6's field rules stand. V3 adds the following, each found necessary by a fixture or by one of the founder recordings and each with its test row in section 19.2 or inline:

| # | Situation | V2 result | V3 rule |
|---|---|---|---|
| H1 | `tkhd` matrix entries written as `0xFFFF0000` (-1.0) and `0x00010000` (1.0) by one muxer, and as `0xFFFF0001`-style values off by one unit in the last place by another | Exact comparison: an off-by-one matrix fell back to `R0` | Each of `a b c d` is rounded to the nearest of -1, 0, 1 in 16.16 fixed point before the comparison. A matrix whose entries are not within `1/256` of one of those three values is `R0` |
| H2 | A video track whose `tkhd` flag "enabled" is 0, before an enabled one | Handled by `demux.rs` (V2 §6.5 step 2) | Unchanged; now tested by `demux_fixtures.rs` with a fixture of two video tracks, the first disabled |
| H3 | `max_fps` from the smallest non-zero sample duration | The last sample of a webcam recording often has a duration of 1 tick, giving a `max_fps` of thousands | The last sample's duration is ignored when there is more than one sample. `max_fps` is reported, not validated (rule 10 uses `avg_fps`), so this changes no acceptance |
| H4 | `AudioSpecificConfig` with sampling-frequency index 15 (an explicit 24-bit rate) | Rate read from the sample entry | The 24-bit rate is read from the ASC |
| H5 | HE-AAC (object type 5) or HE-AACv2 (29), signalled explicitly in the ASC | `codec_string` `mp4a.40.5` / `mp4a.40.29`; `audio.codec` `Aac` | Unchanged and now tested: AAC of any object type is AAC (TS §15.2 rule 7). Whether the browser decodes it is `AudioDecoder`'s answer at `extractAudio`, which fails with `E_DECODE_AUDIO` if not |
| H6 | `avg_fps` of `v3testclip.mp4`: 1,329 frames in 78,918 ms | 16,840 `FpsMilli` | Unchanged: under 60.5 fps, accepted. Recorded as a demux row of the corpus (TE-12) |
| H7 | A `moov` after `mdat` (`ok_moov_at_end.mp4`) | Supported | Unchanged; now tested |
| H8 | `file_size` 0 or smaller than the end of `moov` | `Truncated` at open | Unchanged; tested by `rej_truncated.mp4` |

`probe` still never fails (V2 §6.6). No limit moves: `validate.rs` is not edited in V3 (V2 §25.2).

### 6.4 What the two test files need from the crate

Both test files read a fixture into memory with `std::fs::read` and open it with `MemReader` (V2 §6.2), which is public. Each test file has its own 25-line helper, `fixture(name) -> Vec<u8>`, which:

1. Finds the directory: `OFFCUT_FIXTURES_DIR`, else `<CARGO_MANIFEST_DIR>/../../fixtures/.gen`.
2. Reads `STAMP` there and compares its `manifest_sha256` with the SHA-256 of `fixtures/manifest.json`; a mismatch or a missing stamp panics with "the generated fixtures are stale or missing: run `pnpm fixtures`".
3. Reads the file; a missing file panics with the same message.

The helper is duplicated in the six Rust test files that read generated fixtures (`demux_fixtures.rs`, `probe_rejections.rs`, `loudness_target.rs`, `duration_preserved.rs`, and `chain_no_clicks.rs` and `tests/prosody.rs` for their speech PCM cases, sections 19.5 and 19.7) rather than shared, because TS §5 has no shared test-support file and a fifth file in `tests/` would be compiled as a test crate of its own. The SHA-256 of the manifest is computed with the `sha2` crate, already a workspace dependency, added as a dev-dependency of `offcut-mp4` and `offcut-dsp`.

**Budget.** Unchanged from V2: probe and audio extraction at most 3 s for 60 s on R1 (PS §20.2), 3.7 s for the reference clip (D-64), 3.95 s for `v3testclip.mp4` (78.9 / 60 x 3), all read on D1 (D-74).

---

## 7. `offcut-dsp`: the voice chain and prosody

**Purpose** (TS §6, §18). Resampling (V2), the voice enhancement chain, loudness, and per-word prosody. The chain changes the characteristics of the signal and never its timeline (INV-10).

**Crate rules** (TS §2, §7). Depends on `offcut-types`, `rubato`, `nnnoiseless`, `ebur128`, `thiserror`; dev: `proptest`, `sha2`. Pure: no clock, no randomness, no global state, no I/O, no logging. No `unwrap`, `expect`, `panic!`, slice indexing. No `HashMap`. Floating-point work is done in `f32` for samples and `f64` for filter state and accumulators; the order of every sum is fixed (no parallel reduction), so the same input gives the same bits (P8).

**Never.** Detects silence, classifies a pause, measures a gap for any purpose but prosody's z-scores, cuts, shortens, crossfades, concatenates segments, time-stretches, or returns audio of another length than it was given (TS §18 "Enhancement is not editing", TS §36). Takes word timings in any function that returns audio. Holds a threshold whose purpose is cutting. `ABSOLUTE_SILENCE` only stops the loudness stage from applying gain to a clip with no measurable programme (TS §18.3).

### 7.1 `lib.rs`

```rust
pub mod chain;
pub mod compressor;
pub mod config;
pub mod denoise;
pub mod highpass;
pub mod limiter;
pub mod loudness;
pub mod prosody;
pub mod resample;

pub use chain::{ChainOutput, Flow, run_chain};
pub use config::ChainConfig;
pub use prosody::measure_prosody;
pub use resample::resample_mono;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum DspError { Empty, NonFinite }      // V2's enum, unchanged; now returned by run_chain
```

### 7.2 `config.rs` [ONLY DSP constants]

Every value of TS §18.3 verbatim, plus the ones TS leaves to the implementation. All are (assumption, M1.2) unless marked derived; E-2 reviews them by ear.

```rust
pub const SAMPLE_RATE: Hz = Hz::new(48_000);                 // TS §18 (derived)
pub const BLOCK_SAMPLES: usize = 48_000;                      // 1 s blocks (TS §18.2)
pub const TARGET_LOUDNESS: f32 = -14.0;                       // LUFS (TS §18.3, PS §20.1)
pub const LIMIT_CEILING_DBFS: f32 = -1.0;
pub const MAX_GAIN_DB: f32 = 24.0;
pub const ABSOLUTE_SILENCE_LUFS: f32 = -70.0;
pub const LOUDNESS_RETRY_LU: f32 = 0.5;                       // TS §18.2 step 6
pub const HPF_HZ: Hz = Hz::new(80);
pub const HPF_Q: f64 = std::f64::consts::FRAC_1_SQRT_2;       // Butterworth (derived)
pub const DENOISE_MIX: f32 = 0.85;
pub const DENOISE_FRAME: usize = 480;                         // RNNoise frame at 48 kHz (derived)
pub const DENOISE_SCALE: f32 = 32_768.0;                      // RNNoise works on 16-bit-scaled floats (derived)
pub const DENOISE_DELAY_SAMPLES: usize = 480;                 // (assumption, measured at S4 by impulses_stay_aligned)
pub const COMP_THRESHOLD_DB: f32 = -18.0;
pub const COMP_RATIO: f32 = 3.0;
pub const COMP_ATTACK_MS: f32 = 5.0;
pub const COMP_RELEASE_MS: f32 = 80.0;
pub const COMP_KNEE_DB: f32 = 6.0;
pub const LIMITER_LOOKAHEAD: usize = 240;                     // 5 ms at 48 kHz
pub const LIMITER_RELEASE_MS: f32 = 50.0;
pub const ALIGN_TOLERANCE: usize = 48;                        // 1 ms (TS §18.3)
pub const CLICK_FACTOR: f32 = 2.0;                            // P6: a step at a block boundary <= 2 x local RMS
pub const CLICK_RMS_WINDOW: usize = 480;                      // local RMS over 10 ms either side of the boundary
// Prosody (D-96, D-108)
pub const PROSODY_DECIMATE: usize = 6;                        // 48 kHz -> 8 kHz
pub const PITCH_FRAME: usize = 320;                           // 40 ms at 8 kHz
pub const PITCH_HOP: usize = 80;                              // 10 ms at 8 kHz
pub const PITCH_MIN_LAG: usize = 20;                          // 400 Hz at 8 kHz
pub const PITCH_MAX_LAG: usize = 134;                         // 59.7 Hz at 8 kHz
pub const VOICING_MIN: f32 = 0.45;
pub const VOICED_FRAME_MIN_DB: f32 = -60.0;
pub const ENERGY_FLOOR_DB: f32 = -100.0;
pub const PROSODY_MIN_WORDS: usize = 3;                       // TS §18.2

#[derive(Clone, Debug, PartialEq)]
pub struct ChainConfig {                                      // TS §34.3, verbatim
    pub denoise: bool, pub hpf: Hz, pub denoise_mix: f32, pub target: Lufs,
    pub ceiling: Dbfs, pub max_gain_db: f32, pub absolute_silence: Lufs,
}
impl Default for ChainConfig { /* denoise: true (TE-13 may set false here, D-110), and the constants above */ }
```

`Lufs::new` and `Dbfs::new` return `Option` (they refuse a non-finite value); `Default` builds them from the constants with a match whose `None` arm is unreachable by construction and returns the value `-14.0` / `-1.0` once more through a `const` helper, so no `unwrap` is needed. No constant in this file is a silence threshold for cutting, a minimum pause, a kept-pause length or a crossfade (TS §18.3).

### 7.3 `highpass.rs`

```rust
pub struct HighPass { b0: f64, b1: f64, b2: f64, a1: f64, a2: f64, z1: f64, z2: f64 }
impl HighPass {
    pub fn new(cutoff: Hz, rate: Hz) -> HighPass;        // RBJ cookbook high-pass, Q = HPF_Q
    pub fn process(&mut self, block: &mut [f32]);        // in place, Direct Form II transposed; state carried across calls
}
```

| Situation | Result |
|---|---|
| Any block | Same length; state kept for the next block, so a block boundary is invisible (P6) |
| `cutoff` 0 or not below `rate / 2` | The filter passes the signal unchanged (b0 = 1, others 0) |
| DC input | Decays to 0 |
| A 1 kHz sine | Gain within 0.05 dB of 1 |
| A 40 Hz sine | Attenuated by at least 10 dB |

The filter adds no delay that matters to alignment: its impulse response peaks at sample 0 (inline test).

### 7.4 `denoise.rs`

```rust
pub struct Denoise { /* nnnoiseless::DenoiseState, pending input (< 480), dry delay line, mix, samples to skip */ }
impl Denoise {
    pub fn new(mix: f32) -> Denoise;
    /// Consumes `input`, appends finished output to `out`. Output lags input by DENOISE_DELAY_SAMPLES
    /// plus the samples still pending; the first DENOISE_DELAY_SAMPLES outputs are dropped here.
    pub fn process(&mut self, input: &[f32], out: &mut Vec<f32>);
    /// Pushes zeros through until every input sample has its output: after `finish`, the total
    /// appended to `out` equals the total consumed. Called once.
    pub fn finish(&mut self, out: &mut Vec<f32>);
}
```

Algorithm, per 480-sample frame: scale the frame by `DENOISE_SCALE`; `DenoiseState::process_frame`; scale back; mix `wet x mix + dry_delayed x (1 - mix)`, where `dry_delayed` is the input delayed by `DENOISE_DELAY_SAMPLES` through a ring of that length, so wet and dry are aligned before they are added. The first `DENOISE_DELAY_SAMPLES` mixed samples are dropped; `finish` feeds `DENOISE_DELAY_SAMPLES` + pending-padding zeros and keeps only as many outputs as are still owed. The VAD probability `process_frame` returns is discarded: it decides nothing (TS §36).

| Situation | Result |
|---|---|
| Input of `n` samples over any number of `process` calls, then `finish` | Exactly `n` samples appended in total |
| An impulse at sample `k` | The output's largest absolute sample is at `k ± ALIGN_TOLERANCE` (this is how S4 measures `DENOISE_DELAY_SAMPLES`; the test `impulses_stay_aligned` fails until the constant is right) |
| Digital silence | Digital silence or values below 1e-6 |
| `mix` 0.0 | Output equals input delayed and compensated: sample-exact |

### 7.5 `compressor.rs`

```rust
pub struct Compressor { env_db: f32, attack: f32, release: f32, threshold: f32, ratio: f32, knee: f32 }
impl Compressor {
    pub fn new(rate: Hz) -> Compressor;                 // constants from config.rs
    pub fn process(&mut self, block: &mut [f32]);       // in place, no look-ahead, no delay, no make-up gain
}
```

Gain computer (soft knee, log domain): with level `x = 20 log10(max(|s|, 1e-9))`, `T` threshold, `W` knee, `R` ratio: below `T - W/2` the output level is `x`; above `T + W/2` it is `T + (x - T) / R`; in between `x + (1/R - 1)(x - T + W/2)^2 / (2W)`. The detector is a peak follower in dB with `attack = exp(-1 / (0.005 x 48,000))` and `release = exp(-1 / (0.080 x 48,000))`. The gain applied is `10^((out_level - level) / 20)` of the smoothed level, never above 1. Make-up gain is the loudness stage's job.

### 7.6 `loudness.rs`

```rust
pub fn integrated(samples: &[f32]) -> Option<Lufs>;       // ebur128, 1 channel, 48 kHz, mode I; None when no block passes the gate
pub fn sample_peak(samples: &[f32]) -> Dbfs;               // 20 log10(max |s|); -inf is reported as Dbfs of -200.0
pub fn gain_db(measured: Option<Lufs>, cfg: &ChainConfig) -> f32;
pub fn apply_gain(samples: &mut [f32], db: f32);
```

`gain_db`: `None` or below `cfg.absolute_silence`: 0 dB. Otherwise `target - measured`, capped at `+cfg.max_gain_db` (no cap on a negative gain). `ebur128` is given the samples in chunks of `BLOCK_SAMPLES` so its memory stays bounded; a mono stream is weighted as channel `Center` (BS.1770).

| Case (inline) | Expect |
|---|---|
| A 1 kHz sine of peak -20 dBFS, 10 s | `integrated` within ±0.1 LU of -23.0 (BS.1770: a 1 kHz sine of peak 0 dBFS in one channel reads -3.01 LKFS) |
| Digital silence | `None` |
| `gain_db(Some(-50.0))` | +24.0 (capped) |
| `gain_db(Some(-80.0))` | 0.0 (below `ABSOLUTE_SILENCE`) |
| `gain_db(Some(-10.0))` | -4.0 |

### 7.7 `limiter.rs`

```rust
pub fn limit(samples: &mut [f32], ceiling: Dbfs, rate: Hz) -> usize;   // whole buffer, in place, length unchanged;
                                                                        // returns the samples the guard clamp touched
```

A look-ahead peak limiter written so that it adds no delay: for each sample the required gain `r[n] = min(1, c / |x[n]|)` (with `c = 10^(ceiling/20)`); the attack gain is the moving minimum of `r` over `[n, n + LIMITER_LOOKAHEAD]` smoothed by a moving average of the same length, which ramps down before a peak arrives and reaches `r` at the peak; the release recovers toward 1 with `exp(-1 / (0.050 x rate))` but never above the attack gain. `y[n] = x[n] x g[n]`. A final clamp to `±c` guards against rounding and counts the samples it touched; `limit` returns that count, because an integration test cannot read a private counter: `loudness_target.rs` expects 0 for every case. `chain.rs` ignores the value.

| Situation | Result |
|---|---|
| A signal under the ceiling everywhere | Unchanged, bit for bit |
| A full-scale square wave | Peak at most -1.0 dBFS; no sample above |
| A single sample spike at 0 dBFS in silence | The spike is at most -1.0 dBFS; the gain ramp starts 240 samples before it and recovers after it; no other sample moves by more than the ramp |
| Length | Unchanged; the look-ahead reads ahead and writes in place, so no sample shifts |

### 7.8 `chain.rs`

Signature, inputs, outputs, errors and postconditions: TS §34.3 verbatim (the worked exemplar), including `Flow` and `ChainOutput`.

```rust
pub enum Flow { Continue, Cancel }
pub struct ChainOutput { pub samples: Vec<f32>, pub loudness: Lufs, pub peak: Dbfs }
pub fn run_chain(pcm48: &[f32], cfg: &ChainConfig, on_block: &mut dyn FnMut(SampleCount) -> Flow)
    -> Result<Option<ChainOutput>, DspError>;
```

Algorithm (TS §34.3 steps 1-7, with V3's details):

| Step | What | Detail |
|---|---|---|
| 1 | Checks | Empty: `Err(Empty)`. Any non-finite sample: `Err(NonFinite)` |
| 2 | Working buffer | `work = pcm48.to_vec()`; one allocation; no other buffer of the clip's length except the denoiser's output vector, which is reused as `work` after phase 3 |
| 3 | Per 1 s block | High-pass in place; denoise (when `cfg.denoise`) appending to `den`; compressor on the samples `den` gained in this block. Each stage carries its state. After the block: `on_block(SampleCount(done))`; `Cancel` returns `Ok(None)` before the next block starts (P7). The last block may be shorter and is processed |
| 3b | Flush | `denoise.finish(&mut den)`, then the compressor on the flushed tail. `Denoise` guarantees `den.len() == pcm48.len()` (section 7.4); a `debug_assert_eq!` checks it, and `duration_preserved.rs` proves it for random lengths. There is no release-time branch that pads or trims: a length that could differ would be a bug of `denoise.rs`, caught by P1 |
| 4 | Loudness | `m = loudness::integrated(&work)`; `g = loudness::gain_db(m, cfg)`; `apply_gain` |
| 5 | Limiter | `limiter::limit(&mut work, cfg.ceiling, SAMPLE_RATE)` |
| 6 | Re-measure | When `m` passed the gate and `g` was not capped and `|integrated(&work) - target| > LOUDNESS_RETRY_LU`: one corrective `apply_gain` by the difference and one more `limit`. Never a second correction |
| 7 | Result | `debug_assert_eq!(work.len(), pcm48.len())`; `loudness` = the last measurement, or `cfg.absolute_silence` when nothing passed the gate; `peak` = `sample_peak(&work)` |

When `cfg.denoise` is false, step 3 skips the denoiser and the compressor runs on the high-passed block in place.

**Budget.** At most 4 s for the 60 s reference clip on R1 (PS §20.2, TE-13), read on D1 (D-74), 4.98 s for the reference clip (D-64), 5.26 s for `v3testclip.mp4`. The native timing of S4 is a warning line only: WASM without SIMD-specific code runs about 1.5 to 2.5 times slower than native (assumption), so a native time over 2 s for 60 s is the signal to stop and read section 20.2.

**Memory.** `pcm48` (borrowed), `work` (one copy), the denoiser's internal state (a few hundred kB) and its output vector (one more copy until it becomes `work`): at most three clip-length `f32` buffers at once, 52 MB for 90 s, inside the ASR-phase estimate of TS §31 (58 MB).

### 7.9 `prosody.rs`

```rust
pub fn measure_prosody(out48: &[f32], words: &[Word], sentences: &[Sentence]) -> Prosody;   // reads only (TS §18.1)
```

| Step | Rule |
|---|---|
| 1 | For each word: sample range `[start_ms x 48, end_ms x 48)`, clamped to `0..out48.len()`; an empty range gives energy `ENERGY_FLOOR_DB` and no pitch |
| 2 | Energy: `10 log10(mean(x^2))`, floor `ENERGY_FLOOR_DB` |
| 3 | Pitch (D-108): decimate the word's samples by `PROSODY_DECIMATE` (mean of each group of 6); for each frame of `PITCH_FRAME` samples at hop `PITCH_HOP` that lies wholly inside the word: remove its mean; skip it if its RMS is under `VOICED_FRAME_MIN_DB`; normalized autocorrelation `r(τ) = Σ x[n]x[n+τ] / sqrt(Σ x[n]^2 Σ x[n+τ]^2)` for `τ` in `PITCH_MIN_LAG..=PITCH_MAX_LAG`; the best `τ*` with `r ≥ VOICING_MIN` is a voiced frame; parabolic interpolation over `τ*-1, τ*, τ*+1` refines the lag; `F0 = 8,000 / τ_refined`. The word's pitch is the median F0 of its voiced frames (the lower middle for an even count), or none |
| 4 | Per sentence: the z-score of energy over the sentence's words, and of `log2(F0)` over the sentence's words that have a pitch; population standard deviation. Fewer than `PROSODY_MIN_WORDS` values, or a standard deviation below `1e-6`: 0.0 for every word of the sentence. A word without pitch gets `pitch_z = 0.0` |
| 5 | A word in no sentence (never happens with `segment_sentences`, which covers every word) gets 0.0 and 0.0 |
| 6 | `per_word.len() == words.len()`, in word order |

**Never.** Returns audio; changes `out48`; reads anything but the samples inside each word's range; makes a decision. Prosody is a measurement the detector reads (KeywordPop, the NumberReveal energy bonus).

**Budget.** Under 1 s for the reference clip on R1 (TS §8, assumption, M1.2), read on D1. The per-word work is about 30 frames x 115 lags x 320 multiply-adds; for 160 words, under 2 x 10^8.

### 7.10 Inline unit tests (besides those of 7.3-7.7)

| Case | Expect |
|---|---|
| `run_chain` on 1 sample | `Ok(Some(_))` of length 1 |
| `run_chain` on 48,001 samples (one block and one sample) | Length 48,001 |
| `run_chain` with `on_block` returning `Cancel` at the first call | `Ok(None)`; `on_block` called exactly once |
| `run_chain` with `denoise: false` | Same length; loudness on target |
| `measure_prosody` with zero words | Empty `per_word` |
| `measure_prosody`: a sentence of four words where the third is 12 dB louder | The third word's `energy_z` is the largest and above 1.0 |
| `measure_prosody`: a 200 Hz sawtooth word among 120 Hz sawtooth words | That word's `pitch_z` is the largest and above 1.0 |
| `measure_prosody`: a sentence of two words | Both 0.0, 0.0 |
| Same inputs twice | Identical `Prosody` |

---

## 8. `offcut-text`: the complete number grammar, units, sentences

**Purpose** (TS §6). Tokenize, segment sentences, parse and format numbers, produce `NormalizedSpan`s. **Never** decides what becomes a visual event; never mutates a `Transcript` after building it (TS §10.8).

**Crate rules.** Depends on `offcut-types`, `serde`; dev: `proptest`. Pure. `numbers.rs` stays the only file with `parse_quantity` and `format_quantity` [ONLY]; `cardinals.rs` is a private helper it calls (D-92). Every lexicon is a sorted `const` slice of lower-case `&str`, searched with `binary_search`; a test asserts each slice is sorted and free of duplicates.

The public surface does not change: `normalize_transcript`, `segment_sentences`, `parse_quantity`, `format_quantity`, `tokenize`, `Token`, `TokenKind`, `RawWord`, `SENTENCE_GAP`, `SENTENCE_MAX_WORDS` (V2 §8, frozen by V2 §25.1). `units_lex` becomes a public module because `offcut-detect` reads `COMPARATIVES` from it (D-102) and nothing else.

### 8.1 `tokenize.rs`: a wider `Digits`

`Token` and `TokenKind` are unchanged (V2 §8.2). What changes is which text is classified `Digits`. After lower-casing and splitting leading and trailing punctuation as in V2, a word is `Digits` when, after removing:

1. at most one leading currency symbol from `$ € £ ₹`,
2. at most one trailing sign or suffix from `% k m b x`,
3. thousands separators,

the rest parses as a decimal number by V2's `parse_decimal`. The token's `text` keeps the symbol and the suffix as written (lower-cased): `$1.5m`, `40%`, `3x`, `€500`, `10k`.

| Word as recognized | V2 tokens | V3 tokens |
|---|---|---|
| `$10,000.` | `Digits("$10,000")`, `Punct(".")` | Same |
| `10K` | `Word("10k")` | `Digits("10k")` |
| `$1.5M` | `Word("$1.5m")` | `Digits("$1.5m")` |
| `3x` | `Word("3x")` | `Digits("3x")` |
| `€500` | `Word("€500")` | `Digits("€500")` |
| `₹2,00,000` (Indian grouping) | `Word(…)` | `Word("₹2,00,000")`: Indian grouping is not read (assumption, M1.3); the number is not shown rather than misread |
| `10:30` | `Word("10:30")` | Same: a clock time is never a quantity |
| `5-10` | `Word("5-10")` | Same: a range is not parsed (TS §17.2) |
| `forty-two` | `Word("forty-two")` | Same; `cardinals.rs` reads it |
| `2024` | `Digits("2024")` | Same; `exclusions.rs` judges it a year |

### 8.2 `cardinals.rs` (D-92)

```rust
pub(crate) const MAX_CARDINAL_TOKENS: usize = 24;   // "nine hundred and ninety nine trillion … nine"
/// A spelled cardinal at the start of `words` (lower-cased Word texts): tokens used and its value.
pub(crate) fn cardinal(words: &[&str]) -> Option<(usize, u64)>;
/// "point" + one or more digit words after a cardinal: tokens used and the fraction (0.0..1.0).
pub(crate) fn point_fraction(words: &[&str]) -> Option<(usize, f64)>;
```

Grammar (a value up to 999,999,999,999,999):

```text
cardinal  := group (magnitude group?)* | "a" ("hundred" | magnitude) …        ; "a thousand" = 1,000
group     := 0..99 | teen-or-unit "hundred" ("and"? 1..99)? | teen "hundred" ("and"? 1..99)?   ; "fifteen hundred" = 1,500
0..99     := "zero" | unit | teen | tens | tens "-" unit (one token) | tens unit (two tokens)
magnitude := "thousand" | "million" | "billion" | "trillion"   ; strictly descending; each at most once
"and"     := consumed only when a 1..99 part follows it (V2)
```

| Words | Result |
|---|---|
| "ten thousand" | (2, 10,000) |
| "two hundred and fifty" | (4, 250) |
| "three million two hundred thousand" | (5, 3,200,000) |
| "one point five" | `cardinal` (1, 1); `point_fraction(["point","five"])` (2, 0.5) |
| "a hundred" | (2, 100) |
| "a" alone, "a lot" | `None` |
| "fifteen hundred" | (2, 1,500) |
| "fifteen hundred thousand" | (2, 1,500): a teen-hundred takes no magnitude after it (V2) |
| "million" alone | `None` |
| "thousand million" | (1, … ) → `None`: a magnitude cannot start a cardinal |
| "two thousand thousand" | (2, 2,000): the second "thousand" is not consumed |
| "nine hundred ninety nine trillion" | (6, 999 x 10^12) |
| "zero point zero five" | `cardinal` (1, 0); fraction (3, 0.05) |

### 8.3 `numbers.rs`: `parse_quantity`

Signature unchanged: `pub fn parse_quantity(tokens: &[Token]) -> Option<(usize, Quantity)>`. It reads one quantity at the start of `tokens` in four stages and never guesses: a sequence it cannot read gives `None`.

| Stage | Reads | Rules |
|---|---|---|
| 1. Core | A `Digits` token (symbol, number, suffix), or `cardinal` + optional `point_fraction`, or "a"/"an" + "hundred"/magnitude | A `Digits` token with both a currency symbol and `x`, or both `%` and a suffix, is `None`. V2's split groups (D-42) are read after a whole `Digits` core with no symbol after it, no suffix and no decimal point |
| 2. Magnitude | One following Word from `thousand million billion trillion`, when the core had no suffix and no magnitude of its own | Multiplies. "2 million" = 2,000,000; "1.5 billion" = 1.5 x 10^9; "three point five million" = 3,500,000 |
| 3. Grand | One following Word "grand" | Multiplies by 1,000; the unit becomes `Usd` unless a currency is already set. "five grand" = 5,000 USD |
| 4. Unit | At most one unit, from the core's own symbol or sign, or from the following one or two tokens, by `units_lex` (section 8.4) | Symbol `$ € £ ₹` → currency; `%` → `Percent`; `x` → `Times`; then, if no unit yet: "percent" / "per cent" / "pct" → `Percent`; a currency word → that currency; "times" + a word of `COMPARATIVES` → `Times`, consuming "times" only (D-102); a time, data or count word → that unit (count: D-101). A currency word after a currency symbol of the same currency is consumed ("$500 dollars"); of another currency it is not |
| Result | `(tokens used, Quantity { value, unit, display: format_quantity(value, &unit) })` | `value` is `f64`; integers up to 10^15 are exact |

**Suffix values.** `k` x 1,000; `m` x 1,000,000; `b` x 1,000,000,000. A suffix sets no unit: `10k` is 10,000 with `Unit::None`, `$10k` is 10,000 USD.

**Not parsed** (TS §17.2): fractions in words ("half", "a quarter"), ranges ("5 to 10" is two quantities, which FromTo may join), dates, ordinals ("third" is a word), Indian digit grouping, "dozen", Roman numerals.

### 8.4 `units_lex.rs`: every word list (D-84, D-101, D-102)

```rust
pub const CURRENCY_SYMBOLS: &[(char, CurrencyUnit)]  = &[('$', Usd), ('£', Gbp), ('€', Eur), ('₹', Inr)];   // sorted by code point: U+0024, U+00A3, U+20AC, U+20B9
pub const CURRENCY_WORDS:   &[(&str, CurrencyUnit)] = &[
    ("buck", Usd), ("bucks", Usd), ("dollar", Usd), ("dollars", Usd), ("eur", Eur), ("euro", Eur), ("euros", Eur),
    ("gbp", Gbp), ("inr", Inr), ("rs", Inr), ("rupee", Inr), ("rupees", Inr), ("usd", Usd)];
pub const PERCENT_WORDS:    &[&str] = &["pct", "percent"];             // and the two-token "per cent"
pub const MAGNITUDE_WORDS:  &[(&str, f64)] = &[("billion", 1e9), ("million", 1e6), ("thousand", 1e3), ("trillion", 1e12)];
pub const TIME_WORDS:       &[(&str, TimeUnit)] = &[
    ("day", Days), ("days", Days), ("hour", Hours), ("hours", Hours), ("hr", Hours), ("hrs", Hours),
    ("millisecond", Milliseconds), ("milliseconds", Milliseconds), ("min", Minutes), ("mins", Minutes),
    ("minute", Minutes), ("minutes", Minutes), ("month", Months), ("months", Months), ("ms", Milliseconds),
    ("sec", Seconds), ("second", Seconds), ("seconds", Seconds), ("secs", Seconds), ("week", Weeks),
    ("weeks", Weeks), ("year", Years), ("years", Years), ("yr", Years), ("yrs", Years)];
pub const DATA_WORDS:       &[(&str, DataUnit)] = &[
    ("byte", Bytes), ("bytes", Bytes), ("gb", Gigabytes), ("gigabyte", Gigabytes), ("gigabytes", Gigabytes),
    ("kb", Kilobytes), ("kilobyte", Kilobytes), ("kilobytes", Kilobytes), ("mb", Megabytes),
    ("megabyte", Megabytes), ("megabytes", Megabytes)];
pub const COUNT_NOUNS:      &[&str] = &[
    "apps", "calls", "clicks", "clients", "clips", "comments", "commits", "companies", "customers", "downloads",
    "emails", "employees", "followers", "impressions", "installs", "likes", "lines", "listeners", "members",
    "messages", "orders", "pages", "people", "posts", "projects", "readers", "requests", "sales", "shares",
    "signups", "stars", "students", "subscribers", "users", "videos", "viewers", "views", "visitors",
    "websites", "words"];
pub const COMPARATIVES:     &[&str] = &[
    "better", "bigger", "cheaper", "easier", "faster", "harder", "higher", "larger", "less", "longer",
    "lower", "more", "quicker", "shorter", "slower", "smaller", "stronger"];
pub const COUNT_UNIT_MIN: f64 = 100.0;     // D-101: a count earns the unit bonus from here (assumption, M1.3)
```

`CurrencyUnit`, `TimeUnit`, `DataUnit` are private enums of this file that convert into `offcut_types::Unit`; the shared `Unit` enum is not changed (V2 §25.1, generated types frozen). "second" as a unit word is read only after a number ("30 second clip"); "Second, …" at a sentence start is a list marker of `offcut-detect`, never a unit, because no number precedes it. "pound" and "pounds" are absent on purpose (D-102). "TB" and "terabytes" are absent because `Unit` has no variant for them: "2 TB" is 2 with no unit.

### 8.5 `format_quantity`: display rules

The only formatter (TS §17.2). `numbers_table.rs` is the authority; this table is what it checks.

| Unit | Value | Display | Examples |
|---|---|---|---|
| `None` | under 1,000,000 | Grouped by thousands; up to 2 decimals, trailing zeros removed | `10,000`, `3.5`, `0.05` |
| `None` | 1,000,000 and over | Value in millions, billions or trillions with up to 2 decimals, then the word | `2 million`, `1.5 billion`, `3 trillion` |
| Currency | under 1,000 | Symbol + the `None` form | `$950`, `€9.99` |
| Currency | 1,000 to 999,999, whole thousands | Symbol + thousands + `k` | `$10k`, `£250k` |
| Currency | 1,000 to 999,999, otherwise | Symbol + the `None` form | `$1,500`, `₹12,345` |
| Currency | 1,000,000 and over | Symbol + value in millions / billions / trillions, up to 2 decimals, + `m` / `b` / `t` | `$2m`, `$1.25m`, `€3b` |
| `Percent` | any | The `None` form + `%` | `40%`, `12.5%` |
| `Times` | any | The `None` form + `x` | `3x`, `2.5x` |
| `Milliseconds` | any | The `None` form + ` ms` | `800 ms` |
| Other time units | 1 exactly | `1 ` + singular word | `1 hour`, `1 minute` |
| Other time units | otherwise | The `None` form + ` ` + plural word | `3,000 hours`, `90 minutes`, `1.5 days` |
| `Bytes` | 1 / otherwise | `1 byte` / `{n} bytes` | `512 bytes` |
| `Kilobytes`, `Megabytes`, `Gigabytes` | any | The `None` form + ` KB` / ` MB` / ` GB` | `500 MB` |
| `Count { noun }` | any | The `None` form + ` ` + the noun as spoken | `10,000 subscribers`, `2 million views` |

A negative value is never produced by the parser; `format_quantity` of a negative value prefixes `-` to the form of its absolute value (tested, not used). A non-finite value formats as an empty string (tested; the parser never produces one). The count-up of `offcut-scene` (V2 §11.7) formats every intermediate value with the same function, so `$2m` counts up through `$1.2m`, and its widest text is measured by the scene, not here.

### 8.6 `sentences.rs` and `normalize.rs`

`segment_sentences`: V2's rule, then the split of TS §17.2: while a sentence has more than `SENTENCE_MAX_WORDS` words, split it at its longest internal gap (`words[i+1].start_ms - words[i].end_ms`), the earliest of equal gaps; each half is checked again. Every word still belongs to exactly one sentence, in order.

`normalize_transcript`: V2's five steps, with step 4 done per sentence (D-104): for each sentence, tokenize its words, then scan with `parse_quantity` and skip the tokens each quantity used. `Transcript.numbers` is in word order, as before.

**The one V2 inline test that changes.** V2 §8.5 asserts that "ten million" parses as 10 consuming one token ("the V2 subset"). In V3 it parses as 10,000,000 consuming two. The test is rewritten to the V3 value; it is the only V2 test of the crate whose expectation changes, because it asserted the absence of a V3 feature.

### 8.7 Inline unit tests

| Case | Expect |
|---|---|
| Every lexicon slice of `units_lex.rs` | Sorted, no duplicates, all lower-case |
| `tokenize` of `10K`, `$1.5M`, `3x`, `€500` | One `Digits` token each, text lower-cased |
| `format_quantity(-5.0, None)` | `-5` |
| `format_quantity(f64::NAN, None)` | `""` |
| 45 words, gaps of 100 ms, one gap of 900 ms after word 30 | Split after word 30 by V2's gap rule (`SENTENCE_GAP`, 700 ms) before the 40-word rule is reached: 30 and 15 words |
| The same with that gap at 650 ms | Split after word 30 by the 40-word rule (it is the longest internal gap): 30 and 15 words |
| 90 words with equal gaps | The earliest of equal gaps is always after the first word, so one word is split off at a time: 50 one-word sentences, then one of 40; none over 40, every word in one sentence (section 23, item 28) |
| `normalize_transcript` of "it was $12 / ,000 / in sales." where a sentence boundary falls between `$12` and `,000` (a 900 ms gap) | One number, `$12`: a quantity never crosses a sentence (D-104), and `,000` alone is no quantity |

---

## 9. `offcut-detect`: four kinds, exclusions, resolution, re-detection

**Purpose** (TS §6, §17). Turn a transcript, its prosody and the user's word edits into `DetectedEvent`s above threshold, with stable ids, and re-detect one sentence.

**Crate rules.** Depends on `offcut-types`, `offcut-text`. Pure. No `HashMap`. Every sort is stable with a total order (score, then `span.start`, then kind order NumberReveal < ListReveal < FromTo < KeywordPop of D-47, then `anchors.start`), so equal inputs give equal outputs (INV-7 for detection, `determinism.rs`).

**Never.** Constructs a `DetectedEvent` below its kind's threshold (INV-6). Lays out, renders or formats a quantity (`format_quantity` is called only through `offcut-text`'s parser, and once in `from_to.rs` for an inherited unit, D-106). Changes a `Transcript`. Holds a threshold outside `config.rs`.

### 9.1 `config.rs` [ONLY thresholds and lexicons] (D-84)

V2's ten fields keep their names and values (V2 §9.1); V3 appends fields and never renames one.

```rust
pub struct DetectorConfig {
    // V2
    pub threshold_from_to: f32, pub threshold_list: f32, pub threshold_number: f32, pub threshold_keyword: f32,
    pub number_base: f32, pub number_unit_bonus: f32, pub number_magnitude_bonus: f32, pub number_energy_bonus: f32,
    pub overlay_lead_ms: u32, pub number_hold_ms: u32,
    // V3: FromTo (TS §17.4)
    pub from_to_base: f32, pub from_to_frame_bonus: f32, pub from_to_change_verb_bonus: f32,
    pub from_to_same_unit_bonus: f32, pub from_to_inherit_bonus: f32, pub from_to_close_bonus: f32,
    pub from_to_max_gap_words: usize, pub from_to_close_gap_words: usize, pub from_to_hold_ms: u32,
    // V3: ListReveal
    pub list_base: f32, pub list_all_markers_bonus: f32, pub list_ordinal_bonus: f32,
    pub list_marker_only_base: f32, pub list_marker_only_bonus: f32,
    pub list_window_ms: u32, pub list_marker_gap_ms: u32, pub list_count_min: u8, pub list_count_max: u8,
    pub list_item_max_words: usize, pub list_hold_ms: u32,
    // V3: KeywordPop
    pub keyword_base: f32, pub keyword_energy_weight: f32, pub keyword_pitch_weight: f32,
    pub keyword_lexicon_bonus: f32, pub keyword_min_letters: usize, pub keyword_min_gap_ms: u32,
    // V3: NumberReveal and resolution
    pub number_label_max_words: usize, pub overlay_max_per_window: usize, pub overlay_window_ms: u32,
}
impl DetectorConfig {
    pub fn threshold(&self, kind: EventKind) -> f32;   // the four thresholds by kind
    pub fn hold_ms(&self, kind: EventKind) -> u32;     // number 1,400; from-to 1,700; list 1,700; keyword 0
}
```

| Field | Default | Source |
|---|---|---|
| `from_to_base`, `_frame_bonus`, `_change_verb_bonus`, `_same_unit_bonus`, `_inherit_bonus`, `_close_bonus` | 0.55, 0.25, 0.15, 0.15, 0.05, 0.05 | TS §17.4 |
| `from_to_max_gap_words`, `from_to_close_gap_words` | 8, 6 | TS §17.4 (`FROMTO_MAX_GAP`) |
| `from_to_hold_ms` | 1,700 | TS §19.5: hold 1,500 + fade 200 |
| `list_base`, `list_all_markers_bonus`, `list_ordinal_bonus` | 0.60, 0.30, 0.10 | TS §17.4 |
| `list_marker_only_base`, `list_marker_only_bonus` | 0.65, 0.20 | TS §17.4, D-105 |
| `list_window_ms`, `list_marker_gap_ms` | 45,000, 15,000 | TS §17.4 (`LIST_WINDOW`), D-105 |
| `list_count_min`, `list_count_max`, `list_item_max_words` | 2, 7, 5 | TS §17.4 |
| `list_hold_ms` | 1,700 | TS §19.5: hold 1,500 after the last item + fade 200 |
| `keyword_base`, `_energy_weight`, `_pitch_weight`, `_lexicon_bonus` | 0.50, 0.20, 0.20, 0.20 | TS §17.4 |
| `keyword_min_letters`, `keyword_min_gap_ms` | 4, 2,500 | TS §17.4, §17.5 |
| `number_label_max_words` | 3 | TS §17.4 |
| `overlay_max_per_window`, `overlay_window_ms` | 3, 10,000 | TS §17.5 (`OVERLAY_MAX_PER_10S`) |

All are (assumption, M1.3), tuned by the sweep of `precision_recall.rs` (section 20.3) and re-checked by E-7.

**Lexicons** (sorted `const` slices in this file; every word lower-case):

| Name | Words | Used by |
|---|---|---|
| `LIST_NOUNS` | benefits, examples, factors, facts, features, habits, ideas, kinds, lessons, levels, mistakes, myths, options, parts, phases, points, principles, problems, questions, reasons, rules, secrets, signs, stages, steps, things, tips, tools, tricks, types, ways | `list.rs` announcer |
| `ORDINAL_MARKERS` | fifth 5, first 1, firstly 1, fourth 4, second 2, secondly 2, seventh 7, sixth 6, third 3, thirdly 3 (sorted by word, as every lexicon is) | `list.rs` |
| `CHANGE_VERBS` | boosted, climbed, cut, declined, decreased, down, dropped, fell, grew, increased, jumped, lowered, moved, raised, reduced, rose, shrank, slashed, soared, up, went | `from_to.rs` |
| `INTENSIFIERS` | absolutely, completely, extremely, incredibly, insanely, literally, massively, really, ridiculously, seriously, so, super, totally, truly, very | `keyword.rs` |
| `EMPHASIS_WORDS` | brutal, crazy, critical, essential, exactly, fastest, huge, impossible, incredible, insane, massive, perfect, secret, terrible, worst, wrong | `keyword.rs` |
| `STOP_WORDS` | about, above, actually, after, again, against, also, another, back, basically, because, been, before, being, below, between, both, could, does, doing, done, down, during, each, either, even, ever, every, feel, from, going, gonna, have, here, into, just, kind, know, like, look, made, make, many, mean, might, mine, more, most, much, must, need, neither, only, onto, other, ours, over, really, right, said, same, says, should, some, sort, still, stuff, such, take, than, that, their, them, then, there, these, they, thing, things, think, this, those, through, too, under, until, upon, very, wanna, want, well, were, what, when, where, which, while, will, with, without, would, yeah, your, yours | `keyword.rs` |
| `LABEL_STOP_VERBS` | am, are, be, became, become, been, can, could, did, do, does, feels, felt, gave, get, gets, goes, got, had, has, have, is, made, make, means, said, says, should, takes, took, was, went, were, will, would | `number.rs` label (D-107) |
| `FILLERS` | actually, ah, also, and, basically, er, like, now, ok, okay, right, so, then, uh, um, well | `list.rs` (marker position, item text) |
| `VERSION_CONTEXT` | android, angular, build, chapter, chrome, claude, edition, episode, firefox, galaxy, gpt, ios, ipad, iphone, java, level, llama, macos, model, node, page, part, php, pixel, python, react, release, room, rust, season, series, ubuntu, update, v, version, vue, windows | `exclusions.rs` (X2) |
| `VERSION_AFTER` | max, mini, plus, pro, ultra | `exclusions.rs` (X2) |
| `CLOCK_WORDS` | a.m., am, o'clock, p.m., pm | `exclusions.rs` (X3) |

"always" and "never" are deliberately not emphasis words: Script A says "always" in a plain sentence, and either would put a KeywordPop on nearly every clip.

### 9.2 `lib.rs`

```rust
pub fn detect(t: &Transcript, p: &Prosody, edits: &BTreeMap<WordIdx, String>, cfg: &DetectorConfig)
    -> Vec<DetectedEvent>;                                                         // TS §17.1, unchanged
pub fn redetect_sentence(t: &Transcript, p: &Prosody, edits: &BTreeMap<WordIdx, String>, s: SentenceIdx,
                         prev: &[DetectedEvent], cfg: &DetectorConfig) -> Vec<DetectedEvent>;   // TS §17.1, new
pub fn event_id(kind: EventKind, anchors: WordRange) -> EventId;                  // V2, unchanged (D-47)
```

`detect`, TS §17.3 in full:

| # | Step | Detail |
|---|---|---|
| 1 | `toks = tokenize(&t.words, edits)` | V2 |
| 2 | Effective quantities | `number::quantities(&toks, t)`: per sentence, `parse_quantity` over the sentence's tokens; each `Found { words: WordRange, tokens: Range<usize>, quantity: Quantity, sentence: SentenceIdx, suffixed: bool }` (D-104, D-109) |
| 3 | Candidates | `from_to::find`, then `list::find`, then `number::find`, then `keyword::find`, concatenated in that order |
| 4 | Exclusions | `exclusions::apply(&mut cands, &toks, t, edits)` (section 9.7) |
| 5 | Threshold | `cands.retain(\|c\| c.score >= cfg.threshold(c.kind))` |
| 6 | Resolution | `resolve::resolve(cands, &[], cfg)` (section 9.8) |
| 7 | Events | Each kept candidate becomes `DetectedEvent { id: event_id(kind, anchors), kind, span, anchors, params, confidence: Confidence::new(score)?, enabled: true }`; output sorted by `span.start`, then kind order |

`redetect_sentence` (D-86):

| # | Step |
|---|---|
| 1 | `scope` = the word range from the first word of the earliest scope sentence to the last word of the latest, where scope sentences are: `s`; every sentence intersected by the anchors of an event of `prev` that intersects sentence `s`; every sentence whose `start_ms` lies in `(start_ms(s), start_ms(s) + list_window_ms)` |
| 2 | `outside` = the events of `prev` whose anchors lie wholly outside `scope` |
| 3 | Steps 1-5 of `detect` over the whole transcript; keep the candidates whose anchors lie wholly inside `scope` |
| 4 | `resolve::resolve(candidates, &outside, cfg)`: the `outside` events are fixed and kept; a candidate that conflicts with one of them is dropped |
| 5 | The result is `outside` plus the kept candidates as events, sorted as in `detect`. An event whose kind and anchors did not change has the same id as before (D-47) |

`s` out of range: the function returns `prev` unchanged. Budget: at most 50 ms for a 150-word transcript on R1 (TS §17.6 acceptance, assumption M1.5), read on D1 natively in `redetect.rs` as a printed time, not asserted.

### 9.3 `number.rs`

```rust
pub struct Candidate { pub kind: EventKind, pub anchors: WordRange, pub span: Span, pub score: f32,
                       pub params: EventParams, pub parts: Vec<WordRange> }              // `parts` new (D-106)
pub(crate) struct Found { pub words: WordRange, pub tokens: Range<usize>, pub quantity: Quantity,
                          pub sentence: SentenceIdx, pub suffixed: bool }
pub(crate) fn quantities(toks: &[Token], t: &Transcript) -> Vec<Found>;
pub fn find(toks: &[Token], t: &Transcript, p: &Prosody, cfg: &DetectorConfig) -> Vec<Candidate>;  // V2 signature
```

`suffixed` is true when the quantity's value came through a suffix (`k`, `m`, `b`), a magnitude word or "grand". One candidate per `Found`. Score, TS §17.4:

| Term | Value | Rule |
|---|---|---|
| Base | `number_base` 0.50 | |
| Unit | + `number_unit_bonus` 0.30 | `unit` is a currency, `Percent`, `Times`, a time unit, a data unit, or `Count` with value ≥ `COUNT_UNIT_MIN` (D-101) |
| Magnitude | + `number_magnitude_bonus` 0.20 | `abs(value) ≥ 1,000` or `suffixed` |
| Energy | + `number_energy_bonus` 0.10 | `energy_z ≥ 1.0` on any word of the span |
| Clamp | `min(1.0)`, then x the minimum `Word.confidence` of the span | 1.0 everywhere since TE-2 failed (V2) |

`anchors` = `span words` = `Found.words`; `span` = first word's start to last word's end; `parts` = `[Found.words]`; `params = NumberReveal { value: quantity, label }`, with `label` by D-107: the effective texts of the Word tokens that follow the quantity's last token in the same sentence, at most `number_label_max_words`, stopping at the first `Punct`, at a word of `LABEL_STOP_VERBS`, or at the sentence end; joined by one space; `None` when no word was taken.

| Words (Script A as recognized) | Quantity | Label |
|---|---|---|
| "I had spent 3000 hours editing by hand." | 3,000 `Hours`, display `3,000 hours` | `editing by hand` |
| "Last month those clips earned $12 ,000 in sales." | 12,000 `Usd`, `$12k` | `in sales` |
| "They grew to 10,000 subscribers, which was wild." | 10,000 `Count{subscribers}` | `None` (the comma follows at once) |
| "That is 40% of all views." | 40 `Percent` | `of all views` |
| "It hit $2m and stopped." | 2,000,000 `Usd` | `and stopped`: neither word is a stop verb. This is what TS §17.4's rule gives; E-2 judges whether such labels read well (section 23) |

### 9.4 `from_to.rs`

```rust
pub fn find(toks: &[Token], t: &Transcript, p: &Prosody, cfg: &DetectorConfig) -> Vec<Candidate>;
```

For each sentence, for each ordered pair of quantities `(a, b)` of that sentence with `a` before `b`:

| Check | Rule |
|---|---|
| Gap | The number of Word tokens strictly between `a`'s last token and `b`'s first is at most `from_to_max_gap_words` |
| Values | `a.value != b.value` |
| Units (D-85) | After inheritance (a `None` takes the other's unit): the same `Unit` variant, or both in one group: time (`Milliseconds`..`Years`), data (`Bytes`..`Gigabytes`); `Count` only with the identical noun; two different currencies are incompatible |
| From frame | The token right before `a` is "from", and the token right before `b` is "to": + `from_to_frame_bonus` |
| Change-verb frame | Otherwise: a word of `CHANGE_VERBS` among the 3 tokens before `a`, or between `a` and `b`, and the token right before `b` is "to": + `from_to_change_verb_bonus` |
| Unit bonus | Both carry the same explicit unit from their own words: + `from_to_same_unit_bonus`; one inherits: + `from_to_inherit_bonus` |
| Close | The gap is at most `from_to_close_gap_words`: + `from_to_close_bonus` |
| Base and clamp | `from_to_base` + bonuses, `min(1.0)`, x the minimum confidence of both quantities' words |

For each `a`, only the best `b` is kept (highest score, then nearest). `anchors` = `a.words.start .. b.words.end`; `span` = `a`'s first word start to `b`'s last word end; `parts` = `[a.words, b.words]`; `params = FromTo { from, to, label: None }` with `from` and `to` carrying the unit after inheritance and their `display` formatted again (D-106).

| Words | Score | Event (threshold 0.85) |
|---|---|---|
| "went from 90 minutes to 10 minutes" | 0.55 + 0.25 + 0.15 + 0.05 = 1.00 | Yes, `90 minutes` → `10 minutes` |
| "from $2k to $20k" | 0.55 + 0.25 + 0.15 + 0.05 = 1.00 | Yes |
| "$2k to $20k" | 0.55 + 0.15 + 0.05 = 0.75 | No (TS §17.6) |
| "revenue grew from 40 to 90%" | 0.55 + 0.25 + 0.05 (inherit) + 0.05 = 0.90 | Yes, `40%` → `90%` |
| "dropped to 5 minutes from 20 minutes" | "to" precedes `a`, not `b`: no frame; 0.55 + 0.15 + 0.05 = 0.75 | No |
| "from 2019 to 2024" | Candidate 1.00; both parts are years (X1): score 0 | No (TS §17.6) |
| "from 3 to 5 people" | 0.90 before exclusions (`3` inherits `Count{people}`); X5 sets it to 0, because `3` is bare and under 10 before inheritance | No |

### 9.5 `list.rs`

```rust
pub fn find(toks: &[Token], t: &Transcript, p: &Prosody, cfg: &DetectorConfig) -> Vec<Candidate>;
pub(crate) fn markers(toks: &[Token], t: &Transcript) -> Vec<Marker>;    // also read by exclusions.rs (X6)
pub(crate) struct Marker { pub ordinal: u8, pub words: WordRange, pub ordinal_word: bool, pub at: TimeMs, pub sentence: SentenceIdx }
```

**Markers** (D-105): the first effective Word token of a sentence, after any `FILLERS`, that is an `ORDINAL_MARKERS` word; or "number" followed by a cardinal or digits 1-7; or a cardinal word or digits 1-7 directly followed by a `Punct` token `,` `.` or `:`.

**Announcer form.** A `Found` quantity with integral value in `list_count_min..=list_count_max` and `Unit::None`, directly followed by a Word of `LIST_NOUNS`. Then the markers with ordinals 1, 2, … `count`, in that order, each in a sentence after the announcer's, each starting before `announcer start + list_window_ms`. If any is missing: no candidate (TS §17.6: a partial list is worse than none). Score `list_base` + `list_all_markers_bonus` (always, since all were found) + `list_ordinal_bonus` when every marker is an ordinal word: 0.90 or 1.00. `header` = `"{count} {NOUN}"` upper-cased noun (D-107).

**Marker-only form.** Three or more consecutive markers that are ordinal words with ordinals 1, 2, 3, …, not already used by an announcer candidate, each in a later sentence than the previous one, all within `list_window_ms` of the first. `count` = the number found, at most `list_count_max`. Score `list_marker_only_base` + `list_marker_only_bonus` when no gap between consecutive markers exceeds `list_marker_gap_ms`: 0.85 or 0.65. `header` = `""`.

**Items** (D-107). For each marker: the effective Word tokens after the marker, leading `FILLERS` dropped, at most `list_item_max_words`, stopping at `.` `?` `!` `;` `:`, at a `,` once two words are taken, or at the sentence end; text joined by one space; `ordinal` = the marker's; `at` = the marker word's start.

`anchors` = from the announcer's first word (or the first marker's) to the last word of the last item; `span` = the same words' times; `parts` = `[announcer count words, each marker's words]`; `params = ListReveal { count, header, items }`.

| Words | Result |
|---|---|
| Script A: "There are three reasons it worked. First the captions write themselves. Second the numbers animate automatically. Third everything runs on my laptop." | One candidate, 1.00: header `3 REASONS`; items `the captions write themselves`, `the numbers animate automatically`, `everything runs on my laptop` |
| "three reasons … First … Second …" (no third within 45 s) | None |
| "First, open the app. Second, drop a clip. Third, read the result." | Marker-only, 0.85, header empty, three items |
| "It was the first time I tried." | No marker: "first" is not at the sentence start |
| "I have 3 tips. 1, sleep. 2, eat. 3, ship." | Announcer form, digit markers, 0.90 (markers not ordinal words) |
| "eight reasons" + eight ordinal markers | None: 8 is over `list_count_max` |

### 9.6 `keyword.rs`

```rust
pub fn find(toks: &[Token], t: &Transcript, p: &Prosody, cfg: &DetectorConfig) -> Vec<Candidate>;
```

One candidate per effective Word token that: has at least `keyword_min_letters` alphabetic characters; is not in `STOP_WORDS`; is not a word of any `Found` quantity; is not a list marker word. Score `keyword_base` + `keyword_energy_weight` x clamp(`energy_z` - 1, 0, 1) + `keyword_pitch_weight` x clamp(`pitch_z` - 1, 0, 1) + `keyword_lexicon_bonus` when the token before it (in the same sentence) is in `INTENSIFIERS` or the word itself is in `EMPHASIS_WORDS`; `min(1.0)` x the word's confidence. `anchors` = that one word; `span` = its times; `parts` = `[]`; `params = KeywordPop { word }`.

| Word, prosody | Score | Event (0.80) |
|---|---|---|
| "literally brutal", `energy_z` 2.0, `pitch_z` 1.5 | 0.50 + 0.20 + 0.10 + 0.20 = 1.00 | Yes |
| "brutal", `energy_z` 1.2, `pitch_z` 0.0 | 0.50 + 0.04 + 0 + 0.20 = 0.74 | No |
| "workflow", `energy_z` 2.0, `pitch_z` 2.0 | 0.50 + 0.20 + 0.20 = 0.90 | Yes |
| "workflow", flat | 0.50 | No |
| "that" (stop word), loud | - | No candidate |

### 9.7 `exclusions.rs`

```rust
pub fn apply(cands: &mut [Candidate], toks: &[Token], t: &Transcript, edits: &BTreeMap<WordIdx, String>);
```

Sets `score = 0.0` (TS §17.3: "may zero a score"); the threshold step removes the candidate. A part is checked against the rules its candidate's kind lists:

| # | Rule | Applies to parts of | Test row |
|---|---|---|---|
| X1 | Year: `Unit::None`, an integer in 1900..=2099, written as one `Digits` token without a separator | NumberReveal, FromTo | "in 2024 we" |
| X2 | Version or product: the token before the part is in `VERSION_CONTEXT`, or the token after it is in `VERSION_AFTER`, or the part's text holds two decimal points | NumberReveal, FromTo | "Python 3.12", "iPhone 15 Pro", "version 2" |
| X3 | Clock time: the token after the part is in `CLOCK_WORDS` | NumberReveal, FromTo | "at 5 pm" |
| X4 | Phone-like: one `Digits` token of 7 or more digits with no separator and `Unit::None` | NumberReveal, FromTo | "call 5551234" |
| X5 | Bare small number: `Unit::None` (before inheritance) and `abs(value) < 10` | NumberReveal, FromTo | "3 apples", "from 3 to 5" |
| X6 | List marker: the part's words equal a marker's words (`list::markers`) | NumberReveal | "1, sleep" |
| X7 | A hidden word: an `edits` entry of `""` for any word inside the candidate's `anchors` (TS §17.6) | Every kind | Hiding one word of a list item |

A ListReveal's announcer count is exempt from X5 by design (its value is 2-7). X7 is the only rule that reads `edits` directly.

### 9.8 `resolve.rs`

```rust
pub fn resolve(cands: Vec<Candidate>, fixed: &[DetectedEvent], cfg: &DetectorConfig) -> Vec<Candidate>;
```

TS §17.5, with a fixed set for `redetect_sentence`:

| Step | Rule |
|---|---|
| 0 | Order: priority FromTo > ListReveal > NumberReveal > KeywordPop; within a kind, higher score; then earlier `span.start`; then lower `anchors.start` |
| 1 | Walk the ordered candidates; drop one whose anchors intersect the anchors of a fixed event or of a candidate already kept |
| 2 | Overlay kinds (FromTo, ListReveal, NumberReveal): display window `[span.start - overlay_lead_ms, span.end + hold_ms(kind)]`; a candidate whose window overlaps (or touches, V2 as built) a kept or fixed overlay's window is dropped. The walk order makes "higher priority, then higher score, then earlier" win |
| 3 | At most `overlay_max_per_window` overlays whose windows start inside any `overlay_window_ms` interval: sweep the kept overlays in time order; while a 10 s interval holds more, drop the lowest score among them (the later on a tie). Fixed events count and are never dropped |
| 4 | KeywordPop: at most one per sentence (the highest score, then the earliest); then, in time order, drop a KeywordPop whose `span.start` is under `keyword_min_gap_ms` after the previous kept one's, unless it has the higher score, in which case the previous one is dropped. Fixed KeywordPops count |
| 5 | Return the kept candidates (fixed events are not returned; the caller adds them) |

### 9.9 Inline unit tests (beyond the four test files of section 19)

The six V2 tests of `lib.rs` keep passing unchanged except one: "a number whose word was edited or hidden is not shown" (V2 §9.4) asserts that an edit of `$10,000` to `$20,000` gives no event. In V3 the edit is re-parsed (D-109) and gives an event of `$20k` with the id the old event had: an id hashes the kind and the anchors only (D-47), and an edit of one word keeps the anchors, so the user's toggle in `EditState.event_overrides` carries over to the corrected number (decided 2026-10-18 for the customer: a number the user switched off stays off when its text is corrected; V3 changelog, open item 14). The test is rewritten to assert that; the "hidden" half (edit to `""`) still gives none. It is the only V2 test of the crate whose expectation changes, because it asserted V2's stand-in rule.

| Case | Expect |
|---|---|
| `DetectorConfig::default()` | V2's ten values unchanged; every V3 default of the table above |
| `threshold` and `hold_ms` for each kind | The four thresholds; 1,400 / 1,700 / 1,700 / 0 |
| Every lexicon slice | Sorted, unique, lower-case |
| `markers` of "First, … It was the first time …" | One marker |
| `quantities` with an edit replacing `3000` by `4000` | One `Found` of 4,000 at the same words |
| A candidate with a hidden word inside a list item | X7: score 0 |
| `resolve` with a fixed NumberReveal at 10 s and a new one at 10.5 s | The new one is dropped |

---

## 10. WASM binding crates: `audio_api.rs`, `redetect_sentence`

**Rules for both** (TS §2, V2 §13). `wasm-bindgen` exports and value conversion only: no product rule, no branch on a domain value beyond decoding arguments and mapping an error enum to a code. Conversion uses `serde_wasm_bindgen::Serializer::json_compatible()` (D-31) through the crates' existing `to_js` / `from_js` helpers. Every fallible export returns `Result<_, JsValue>` whose error is `{ code: ErrorCode, detail: string }` with `detail` an enum variant name, never audio, text of a transcript or a word.

### 10.1 `offcut-wasm-core/src/audio_api.rs`

```rust
/// `{ out48: Float32Array, loudness: number, peak: number }`, or `{ cancelled: true }`.
#[wasm_bindgen] pub fn run_chain(pcm48: &[f32], on_block: &js_sys::Function) -> Result<JsValue, JsValue>;
/// `Prosody`: `{ per_word: { energy_z, pitch_z }[] }`.
#[wasm_bindgen] pub fn measure_prosody(out48: &[f32], words: JsValue, sentences: JsValue) -> Result<JsValue, JsValue>;
```

| Call | Behaviour |
|---|---|
| `run_chain` | Copies `pcm48` into WASM memory (wasm-bindgen does this for `&[f32]`), calls `offcut_dsp::run_chain(pcm48, &ChainConfig::default(), on_block)` (D-110). The closure calls `on_block(done_samples as f64)`; a truthy return is `Flow::Cancel`, anything else `Flow::Continue`; a throw from the callback is `Flow::Cancel` and the call then fails with `{ code: "E_INTERNAL", detail: "Callback" }` |
| `run_chain` result | `Ok(Some(out))`: a plain object with `out48` = a new `Float32Array` copied from `out.samples` (its own `ArrayBuffer`, so the worker can transfer it), `loudness` = `out.loudness.get()`, `peak` = `out.peak.get()`. `Ok(None)`: `{ cancelled: true }` |
| `run_chain` errors | `DspError::Empty` → `{ code: "E_DSP", detail: "Empty" }`; `NonFinite` → `{ code: "E_DSP", detail: "NonFinite" }` |
| `measure_prosody` | Deserializes `Word[]` and `Sentence[]` (`"Words"`, `"Sentences"` on failure, as `E_INTERNAL`), calls `offcut_dsp::measure_prosody`, serializes the `Prosody` |

`lib.rs` gains `mod audio_api;` and nothing else. The `offcut-dsp` dependency is already in `Cargo.toml` (V2, for `resample`).

### 10.2 `offcut-wasm-render/src/detect_api.rs`

```rust
#[wasm_bindgen] pub fn detect(transcript: JsValue, prosody: JsValue, edits: JsValue) -> Result<JsValue, JsValue>;   // V2, unchanged
#[wasm_bindgen] pub fn redetect_sentence(transcript: JsValue, prosody: JsValue, edits: JsValue,
                                        sentence: u32, prev: JsValue) -> Result<JsValue, JsValue>;            // DetectedEvent[]
```

`redetect_sentence` decodes the five arguments with the existing helpers (`"Transcript"`, `"Prosody"`, `"Edits"`, `"Events"` as `detail` on failure, all `E_INTERNAL`), calls `offcut_detect::redetect_sentence(…, SentenceIdx::new(sentence), &prev, &DetectorConfig::default())` and serializes the result. Nothing calls it in V3: the render worker's `redetectSentence` handler is V4's (section 1.2), and it is exported now so that V4 adds a worker handler and a use-case, not a binding.

---

## 11. Code generation

Nothing changes. `scripts/gen-types.sh`, the two generator tests and `web/src/gen/domain.ts` / `api.ts` are untouched: V3 adds no shared type. `Prosody`, `Word`, `Sentence`, `Lufs`, `Dbfs`, `RejectReason`, the analytics event `clip_rejected` and `PipelineStage::AudioChain` all exist in the generated files since V1. `ChainConfig`, `ChainOutput`, `Flow`, `Candidate`, `Found`, `Marker`, the lexicons and the labeled-file format are deliberately not generated: none crosses a TypeScript boundary as a type. `FeedLine.displayTo` (D-83) lives in `workers/protocol.ts`, which is hand-written (V1). `scripts/check-gen-clean.sh` must report no diff after every V3 commit.

| Code, event or key introduced or first used in V3 | Generated? | Where it lives |
|---|---|---|
| `E_DSP` | Yes, since V1 (`ERROR_CODES`) | Thrown by `audio_api.rs`; copy stays pending until V4's `ErrorPanel` |
| 14 `REJECT_*` | Yes, since V1 (`REJECT_REASONS`) | Copy in `messages.rejections` (section 16.3) |
| `clip_rejected { reject_reason }` | Yes, since V1 (`AnalyticsEvent`) | Sent by `import-clip.ts` and `run-pipeline.ts` (D-97) |
| `stage_timing { stage: "audio_chain" }` | Yes, since V1 (`PipelineStage`) | Sent by `run-pipeline.ts` |
| `FeedLine.displayTo` | No, hand-written | `workers/protocol.ts` (D-83) |

---

## 12. Server: no changes

No file under `server/`, `crates/offcut-api-types/` or `crates/offcut-types/` changes in V3. `clip_rejected` is already on the server's allowlist (`offcut-api-types/src/analytics.rs`, V1) and its `reject_reason` prop is the generated `RejectReason`; the server accepts it today. `deploy-api.yml` still runs on every merge to `main` and redeploys an unchanged image. `deny.toml` already bans the media crates for the package `offcut-api`; `nnnoiseless` and `ebur128` enter only through `offcut-dsp`, which the server does not depend on.

---

## 13. Web infrastructure

**Layer rules** are V2 §15's, with the edges of D-27 and D-58 and the minting entries of D-59, plus D-103. V3 adds no import edge between layers: the audio worker imports `wasm/load-core` (allowed for every worker), `workers/rpc` and `workers/protocol`; the repos are `persistence` files imported by use-cases (allowed); `RejectionPanel` imports `usecases/import-clip` (allowed for `ui`) and `copy`.

### 13.1 `workers/protocol.ts`: one optional field (D-83)

```ts
export type FeedLine =
  | { kind: "transcribing" }
  | { kind: "cleaning_voice" }
  // `display` and `displayTo` stay on the device.
  | { kind: "event_found"; eventKind: EventKind; display: string; displayTo?: string };
```

`AudioWorkerApi` (V1, TS §14.2) is used as it stands: `runChain(p: { pcm48 }) → { loudness }`, with `out48` kept in the worker; `measureProsody(p: { words, sentences }) → { out48, prosody }`, with `out48` transferred out. No other line of `protocol.ts` changes.

### 13.2 `wasm/load-core.ts`: `CoreApi` grows

```ts
export type ChainResult = { out48: Float32Array<ArrayBuffer>; loudness: Lufs; peak: Dbfs } | { cancelled: true };
export type CoreApi = {
  // V2: coreVersion, openDemuxer, probeAndValidate, resample, normalizeTranscript, newSha256 (unchanged)
  /** The voice chain (TS §18). `onBlock` gets the samples done so far once per second of audio; `true` cancels. */
  runChain(pcm48: Float32Array, onBlock: (done: number) => boolean): ChainResult;
  /** Per-word prosody of the cleaned audio. Reads only. */
  measureProsody(out48: Float32Array, words: readonly Word[], sentences: readonly Sentence[]): Prosody;
};
```

Both are thin wrappers over the exports of section 10.1, with the module's `{ code, detail }` throw passed on unchanged, as V2's wrappers do. `out48` arrives as the module's own `Float32Array`; `loudness` and `peak` arrive as numbers inside the plain object, and the wrapper types the whole object once with `as ChainResult`, as V2 types `handle.probe(fileSize) as ProbeInfo`: a cast of a decoded value to its generated type. No `as <Brand>` of a primitive occurs, so the brand rule (D-59) needs no new entry for this file.

### 13.3 `wasm/load-render.ts`: `RenderApi` grows

```ts
redetectSentence(transcript: Transcript, prosody: Prosody, edits: Record<number, string>,
                 sentence: SentenceIdx, prev: readonly DetectedEvent[]): DetectedEvent[];
```

Only `render.worker` calls `loadRender()`; in V3 no handler calls this method (V4).

### 13.4 `workers/pool.ts`: the fourth row

| Kind | Script | Stage tag | One-way | Accepted during `previewPlay` |
|---|---|---|---|---|
| `media` | `media.worker.ts` | `import` | none | none |
| `asr` | `asr.worker.ts` | `asr` | none | none |
| `audio` | `audio.worker.ts` | `audio_chain` | none | none |
| `render` | `render.worker.ts` | `preview` | `previewClock` | `previewClock`, `previewPause`, `previewSeek` |

One import line (`import audioWorkerUrl from "./audio.worker.ts?worker&url"`, V2 CL item 14), one row in `WORKERS`, one client `audio: lazyClient<AudioWorkerApi>(WORKERS.audio)` and its type in `pool`'s annotation. `preload()` now adds four `modulepreload` links. The pool holds at most four workers (TS §31); none is created before its first call.

### 13.5 The three repos

All three: one IndexedDB store each (`STORES.clips`, `.transcripts`, `.edits` of V1's `schema.ts`), key `clipId`, value wrapped as `Versioned<T>` with `schemaVersion: 1` (TS §23.1); a stored value with a newer `schemaVersion` is treated as absent and deleted (TS §23.3, as `metaGet` and `entitlement-repo.ts` do). A `DOMException` named `QuotaExceededError` is rethrown as `new OpfsError("quota")`; any other rejection as `new OpfsError("io")` (D-103). None logs; none reads `ui`, `state` or `workers`.

```ts
// persistence/clips-repo.ts
/** D-81: lower-case hex SHA-256 of the first min(4 MiB, size) bytes, then the size as u64 little-endian. */
export function hashPrefix(file: Blob): Promise<string>;
export function put(record: ClipRecord): Promise<void>;
export function get(clipId: ClipId): Promise<ClipRecord | undefined>;
export function toBytes(n: number): Bytes;        // the one cast of D-103
export function clear(): Promise<void>;            // section 23, item 26 (decided 2026-10-18)
// persistence/transcript-repo.ts
export function put(clipId: ClipId, record: TranscriptRecord): Promise<void>;
export function get(clipId: ClipId): Promise<TranscriptRecord | undefined>;
export function clear(): Promise<void>;
// persistence/edits-repo.ts
export function put(clipId: ClipId, record: EditsRecord): Promise<void>;
export function get(clipId: ClipId): Promise<EditsRecord | undefined>;
export function clear(): Promise<void>;
```

| Situation | Result |
|---|---|
| `hashPrefix` of a 10 MiB file | SHA-256 over 4,194,304 bytes then 8 bytes: `00 00 a0 00 00 00 00 00` |
| `hashPrefix` of a 1,000-byte file | SHA-256 over the 1,000 bytes then `e8 03 00 00 00 00 00 00` |
| `hashPrefix` of an empty blob | SHA-256 of eight zero bytes |
| `put` then `get` | The same record |
| `get` of an absent key | `undefined` |
| `clear()` | Every record of the store removed. `import-clip.ts` step 4 calls the three, so the device holds the records of one clip at most, as OPFS holds its media (D-40; section 23, item 26). "Start over" alone removes nothing; the next import does. V7's `remove(clipId)` and eviction take its place |
| A stored value with `schemaVersion: 2` | `get` returns `undefined` and the value is deleted |
| The browser refuses the write for lack of room | `OpfsError` with kind `quota` |
| Any other failure | `OpfsError` with kind `io` |
| `ClipRecord` | Holds `clipId`, `createdAt`, `lastOpenedAt`, `clipInfo`, `sourceBytes`, `sourceHashPrefix`. Never a file name (P-11), never a path |

A Vitest file for the repos is not in the TS §5 tree and is not added: the repos are covered by the E2E cases of section 19.10 that read IndexedDB, and by `restore.spec.ts` in V7. `hashPrefix`'s byte layout is covered by one E2E assertion that recomputes it in Node from the fixture's bytes.

---

## 14. Workers: `audio.worker`

### 14.1 `workers/audio.worker.ts`

**Rules** (TS §2, §14, V2 §16). Browser glue for one stage: no product rule; no `state`, `ui`, IndexedDB or network. The entry file is `serveWorker(handlers, { stage: "audio_chain", oneWay: [], duringPreview: [] })` and nothing else at top level besides the module state. Cancellation point: every 1 s block (TS §14.3).

Module state: `kept: Float32Array<ArrayBuffer> | null`, the cleaned audio between `runChain` and `measureProsody` (TS §18.4).

| Handler | Steps | Failure |
|---|---|---|
| `runChain({ pcm48 }, ctx)` | (1) `kept = null` (a buffer left by a clip that ended early is dropped, D-90). (2) `core = await loadCore()`. (3) `r = core.runChain(pcm48, (done) => { ctx.progress({ stage: "audio_chain", done, total: pcm48.length }); return ctx.isCancelled(); })`. (4) `r.cancelled`: return `CANCELLED`. (5) `kept = r.out48`; return `{ loudness: r.loudness }`. `pcm48` is dropped when the handler returns (TS §31) | `E_DSP` with its detail, passed through from the module; anything else `E_INTERNAL`. Stage `audio_chain` |
| `measureProsody({ words, sentences })` | (1) `kept === null`: throw `new WorkerFailure("E_INTERNAL", "NoAudio")`. (2) `prosody = core.measureProsody(kept, words, sentences)`. (3) `out48 = kept; kept = null`. (4) Return `{ out48, prosody }`; `serveWorker` transfers `out48` (V2 CL item 13) | `E_INTERNAL` (`NoAudio`, or `Words` / `Sentences` from the module) |

**Never.** Detects silence; looks at a pause; changes the length of `out48` (it is the module's output, passed on untouched, `out48.length === pcm48.length` by P1); keeps audio after `measureProsody` (TS §18.4; a clip that ends before `measureProsody` leaves `kept` until the next `runChain`, section 23 item 25); calls `fetch` except through `loadCore`.

**Progress.** One progress message per block, about 75 for the reference clip. V2's rule holds (one per cancellation point); V4's throttle of 10 per second (TS §14.3) caps them later without a change here.

---

## 15. State and use-cases

**State.** No store changes. `clip-store` already holds `prosody`, `reject` and the feed (V2 §18.2); `noSpeech()` already moves `processing` to `rejected` with `REJECT_NO_SPEECH`. No machine table changes.

**Use-case rules** are V2 §19's. A `WorkerCallError` is caught at the top of each use-case and stored; an `OpfsError` from a repo is mapped by `failureOf` (D-103); nothing is swallowed.

### 15.1 `import-clip.ts` (C-2)

Signatures unchanged (V2 §19.2). The steps, with V3's changes in bold:

| # | Step | On failure |
|---|---|---|
| 1 | `blockers.forImport()` non-null: return | - |
| 2 | Clip not `idle`: `dismissClip()`; ask the blockers again (V2 as built) | - |
| 3 | `clipId = newClipId()`; `clipStore.begin(clipId, source)` | - |
| 4 | Stand-in until V7 (D-40): remove `clips/`; **`clipsRepo.clear()`, `transcriptRepo.clear()`, `editsRepo.clear()`** (section 23, item 26) | Ignored |
| 5 | `startTimer(clipId)` | - |
| 6 | **`sourceHashPrefix = await clipsRepo.hashPrefix(file)`**: before the file is posted, so the main thread reads its first 4 MiB once | `OpfsError` is impossible here (no storage is written); a `NotReadableError` from the `Blob` is `E_STORAGE_IO`, stage `import`: `importFailed` |
| 7 | `r = pool.media.importAndProbe({ clipId, file })` | As V2 |
| 8 | `r.rejected`: `clipStore.rejected(reason)`; **`track clip_rejected { reject_reason: reason }`** (D-97); remove `clipDir`; return | - |
| 9 | **`clipsRepo.put({ clipId, createdAt: now, lastOpenedAt: now, clipInfo: r.ok, sourceBytes: toBytes(file.size), sourceHashPrefix })`** (TS C-2), `now = Date.now()`, while the clip is still `importing` | `clipStore.failed({ code: E_STORAGE_QUOTA or E_STORAGE_IO, stage: "storage", retryable: true })` (importing to failed); remove `clipDir`; return. No `clip_accepted` |
| 10 | `clipStore.accepted(r.ok)`; `track clip_accepted` | - |
| 11 | `void runPipeline(clipId)` | - |

The `File`'s name is still never read (P-11). `clip_rejected` carries the code alone (TS §15.3, INV-18). The record is written before `accepted` (Prompt 80 as built): the clip machine (TS §12.2, `state/**` untouched in V3) lets an `accepted` clip go only to `processing` or `idle`, so a failure after `accepted` could not be stored.

### 15.2 `run-pipeline.ts` (C-3, C-4)

Signature unchanged. The steps of V2 §19.3 with the two stand-ins replaced (D-22, D-90); new or changed steps in bold:

| # | Step (TS C-4 letter) | Store and events | On failure |
|---|---|---|---|
| 1 | `clipStore.processing("probe_audio", false)` | | |
| 2 | `model = modelReady()` (C-3) | `waitingModel` while the model is not ready | Read at step 6 |
| 3 | (A) `{ pcm48, pcm16 } = pool.media.extractAudio({ clipId })` | `stage_timing probe_audio` | `failed` |
| 4 | **(B2) `pushFeed({ kind: "cleaning_voice" })`; `chainAt = now`; `chain = settle(pool.audio.runChain({ pcm48 }, { transfer: [pcm48.buffer] }))`, where `settle` turns the promise into `{ ok: { loudness }, at } \| { failure, at }` and never rejects; when it settles with `ok`: `track stage_timing { stage: "audio_chain", duration_ms }`** | "Cleaning voice"; `stage_timing audio_chain` | Held in `chain` (step 10) |
| 5 | **The V2 line `out48 = owned(audio.pcm48)` is removed**: `pcm48` now belongs to the audio worker | | |
| 6 | `await model` | `model_download` as V2 | **`await chain`** (so no job is left running), then `failed(modelFailure)` |
| 7 | (B1) `processing("asr")`; `load`; `transcribe` with `transfer: [pcm16.buffer]` | "Transcribing…"; `stage_timing asr` | **`await chain`; `unload()`; the failure stored is the earlier in time of B1's and the chain's (by `at`, D-90)** |
| 8 | (C) `await pool.asr.unload()` (INV-12) | | As V2 |
| 9 | `heard.rejected` (fewer than `MIN_WORDS` words): **`await chain`**; `clipStore.noSpeech()`; **`track clip_rejected { reject_reason: "REJECT_NO_SPEECH" }`**; remove `clipDir`; return. The `clips` record of import step 9 stays until the next import clears it (section 23, item 26) | `clip_rejected` (D-97) | - |
| 10 | **`c = await chain`; `c.failure`: throw it** | | `failed(E_DSP …)`, stage `audio_chain` |
| 11 | **(D) `{ out48, prosody } = pool.audio.measureProsody({ words: transcript.words, sentences: transcript.sentences })`** | | `failed`, stage `audio_chain` |
| 12 | (E) `processing("detect_scene")`; `openSession`; `detect`; `setScene({ … profile: "preview" })` | One feed line per event, **FromTo with `displayTo` (D-83)**; `stage_timing detect_scene` | As V2 |
| 13 | (F) `run.stage = "storage"`; `opfs.writeAudio(clipId, out48)`; **`transcriptRepo.put(clipId, { transcript, prosody })`; `editsRepo.put(clipId, { editState: edit })`** | | `failed(E_STORAGE_QUOTA or E_STORAGE_IO)`, stage `storage` |
| 14 | `clipStore.ready({ transcript, prosody, events, out48 })` | `pipeline_done` with the four counts | |

`displayOf` (V2 as built) is unchanged for three kinds; for `from_to` it returns `params.from.display` and a second function `displayToOf` returns `params.to.display`, used only for that kind. The `assertNever` default stays.

**Timing.** `duration_ms` of `audio_chain` is measured from step 4's call to its settling, on the main thread (D-35); `total_ms` of `pipeline_done` still runs from the import's step 5 to step 14. Because B2 runs beside the model wait and ASR, the total rises by less than the chain's time, and by nothing when ASR is longer (TS §30: "can only lower the total").

**Size.** `run-pipeline.ts` has 298 lines (V2 as built). V3 adds about 60 (the `settle` helper, steps 4 and 9 to 11, the two repo calls, `displayToOf`) and removes 6, which leaves about 352. If it would pass 390, the comments that repeat this document's step table are shortened. No helper moves to another file: a new use-case file needs a TS §5 entry and a lint edge (D-58), which V3 does not take.

**Never.** Calls the API (INV-3); sends a transcript, a feed line, a label or a word in an event (INV-2, INV-18); opens the render session while the ASR session lives (INV-12); copies a PCM buffer (TS §31).

---

## 16. UI, copy, styles

### 16.1 Pages

| File | Reads | Renders | Calls |
|---|---|---|---|
| `ui/pages/EditorPage.tsx` (CHANGED) | clip store, model store, export store | As V2, except status `rejected`: `<RejectionPanel />` in place of the stub | `dismissClip()` (failed and ready, as V2) |

The `rejected` arm of `body()` becomes one line. `messages.editor.rejectedStub` is removed from `messages.ts` with its last reader; `failedStub` and `startOver` stay.

### 16.2 Components

| File | Reads | Behaviour | Calls |
|---|---|---|---|
| `RejectionPanel.tsx` (NEW) | clip store `reject` | A `<section role="alert" data-testid="clip-rejected" data-reject={code}>` with `messages.rejections[code].title` as its heading, `.body` and `.action` as two paragraphs, and a "start over" button (`messages.editor.startOver`). No retry button: a rejection is a value and shows instructions (TS §11.3). No code, size or file name is shown. A `switch` is not needed: the copy is a `Record<RejectReason, …>` read by key | `dismissClip()` |
| `ProcessingFeed.tsx` (CHANGED) | clip store `feed` | `from_to` lines call `found.from_to(line.display, line.displayTo ?? "")`; the copy handles the empty second value (16.3) | none |
| `DropZone.tsx` | unchanged | A refused file arrives as a rejection in the store, not here | - |

**Test ids.** `clip-rejected` keeps its V2 meaning (V2 §20.2 as built) and gains the attribute `data-reject` with the code, which tests read instead of matching sentences. It is a code, not content, and never leaves the page.

### 16.3 `copy/messages.ts` additions

| Section | Keys added or changed in V3 | Wording source |
|---|---|---|
| `rejections` | 14 entries, each `{ title, body, action }`, `satisfies Record<RejectReason, ErrorCopy>` | TS §15.3 "message must state" column; PS §9.4 examples; D-80 (no measured value) |
| `feed.eventFound.from_to` | `(display: string, displayTo: string) => string` | PS §10 J6: "Found: $2k to $20k" (D-83) |
| `editor.rejectedStub` | Removed | - |

The 14 messages. `{…}` is interpolated from `LIMITS` (INV-15); no limit is typed into a sentence. `BYTES_PER_MB = 1_000_000` (MB is 10^6 bytes, as `limits.rs` reads PS §9.4) and `FPS_MILLI_PER_FPS = 1000` are named constants of `messages.ts`, beside V2's `MS_PER_SECOND`.

| Code | Title | Body | Action |
|---|---|---|---|
| `REJECT_FILE_SIZE` | This file is too large | Offcut takes files up to {MAX_FILE_SIZE / BYTES_PER_MB} MB. | Trim the clip or export it at a lower resolution, then drop it again. |
| `REJECT_CONTAINER` | Offcut cannot open this kind of file | Offcut takes MP4 and MOV files with H.264 video and AAC audio. | Save or export the clip as an MP4, then drop it again. |
| `REJECT_CORRUPT` | This file looks damaged | Offcut could not read the inside of this file. It may be incomplete. | Export the clip again from your camera or editor, then drop the new file. |
| `REJECT_NO_VIDEO` | This file has no video | Offcut makes a short from a video of you talking, and this file holds sound only. | Drop the video file instead. |
| `REJECT_HEVC` | This video uses HEVC | This file uses HEVC, which this browser cannot decode here. | Export it as H.264, or choose "Most Compatible" in your camera settings and record again. |
| `REJECT_VIDEO_CODEC` | This video format is not supported | Offcut reads H.264 video, and this file uses another format. | Export the clip as H.264, then drop it again. |
| `REJECT_NO_AUDIO` | This video has no sound | Offcut needs your voice to write the captions, and this file has no audio track. | Record with the microphone on, then drop the new file. |
| `REJECT_MULTI_AUDIO_TRACK` | This video has more than one audio track | Offcut takes clips with one audio track. | Export the clip with a single audio track, then drop it again. |
| `REJECT_AUDIO_CODEC` | This audio format is not supported | Offcut reads AAC audio, and this file uses another format. | Export the clip with AAC audio, then drop it again. |
| `REJECT_DURATION` | This clip is too long | Offcut handles clips up to {MAX_CLIP_DURATION / MS_PER_SECOND} seconds right now. | Trim it in your phone's gallery or in your editor, then drop it again. |
| `REJECT_RESOLUTION` | This video is larger than Offcut handles | Offcut handles video up to {MAX_LONG_SIDE} pixels on its long side. | Record or export at a lower resolution, then drop it again. |
| `REJECT_FRAME_RATE` | This video has too many frames per second | Offcut handles up to {MAX_INPUT_FPS / FPS_MILLI_PER_FPS} frames per second. | Record or export at {same} frames per second or fewer, then drop it again. |
| `REJECT_DECODE_UNSUPPORTED` | This browser cannot play this video | The video uses settings this browser cannot decode. | Export it again with your editor's standard H.264 settings, then drop it again. |
| `REJECT_NO_SPEECH` | No English speech was found | Offcut could not hear enough English words in this clip to write captions. | Drop a clip of you talking in English. |

`feed.eventFound.from_to(display, displayTo)`: `displayTo === ""` gives "Found: {display}" (a line from an old store shape cannot occur, but the empty case is defined); otherwise "Found: {display} to {displayTo}".

**Rules that already apply** (V1 §11.11, V2 §20.3): no typed-in limit; format names ("MP4", "H.264", "AAC", "HEVC") are names, not limits, as V1's unsupported reasons already use them; none of the phrases "never leaves", "GDPR", "DPDP", "CCPA", "SOC 2", "compliant" (P-12). "Most Compatible" is the camera setting's name (PS §9.4).

### 16.4 Styles

New class names only, in `components.module.css`, with the tokens of `tokens.css`: `.rejection` (the panel), `.rejectionTitle`, `.rejectionBody`, `.rejectionAction`. The panel reuses V2's `.stub` spacing from `pages.module.css` through composition (`composes:`), so no V2 class changes. No `style` attribute string; no runtime CSS-in-JS (CSP `style-src 'self'`).

---

## 17. Fixtures, the labeled set and the corpus tooling

### 17.1 `fixtures/gen_fixtures.sh`

POSIX `sh`, run by `pnpm fixtures` from the repository root on D1 (Git Bash), the Linux runner and the Windows runner. It needs `ffmpeg`, `ffprobe`, `sha256sum` and, only when the reference clip is absent, `curl`. It writes nothing outside `fixtures/.gen/`.

| Step | What | Detail |
|---|---|---|
| 1 | Tools | `ffmpeg -version` must start with `ffmpeg version 9.0` or `ffmpeg version n9.0` (Linux static builds print the `n`; D-94: the Linux build may be 9.0.x); otherwise exit 2 with "ffmpeg 9.0 is needed, found …" |
| 2 | Reference clip | `$OFFCUT_REFERENCE_CLIP`, else `testclips/speech_scriptA_landscape_720p.mp4`, else `fixtures/.cache/<SAMPLE_CLIP_PATH>`; else download `$VITE_ASSET_BASE_URL/<SAMPLE_CLIP_PATH>` into `fixtures/.cache/` with `curl --fail --retry 3`. `SAMPLE_CLIP_PATH` is read from `web/src/net/asset-fetch.ts` with `sed`, as `referenceClip()` does (V2 §23.4). The clip's SHA-256 must start with the 16-hex hash segment of its asset name, else exit 3 |
| 3 | Up to date? | If `fixtures/.gen/STAMP` exists and holds the same ffmpeg version, the SHA-256 of this script, of `fixtures/manifest.json` and of the reference clip, and every manifest file exists: print "fixtures are up to date" and exit 0 |
| 4 | Clean | Remove `fixtures/.gen/` and create it, with `speech/` inside |
| 5 | Synthetic fixtures | The recipes of 17.2, in manifest order. A recipe that fails stops the script with exit 4 and the recipe id |
| 6 | Derived speech fixtures | The recipes of 17.3 |
| 7 | PCM | For every speech fixture (the five manifest entries with `"pcm": true`; the synthetic signals of `loudness_target.rs` are made inside the test, section 19.4): `ffmpeg -i <file> -map 0:a:0 -ac 1 -ar 48000 -f f32le <file>.pcm48.f32`. This is test data made with ffmpeg's own decoder and resampler: the Rust DSP tests read raw `f32` because Rust has no AAC decoder (D-77) |
| 8 | Check | For each manifest entry with `probe` fields, `ffprobe` must report them (codec, size, frame count range, duration within 50 ms); a mismatch exits 5 with the name |
| 9 | Stamp | Write `STAMP` (17.4) last, so that an interrupted run leaves no stamp |

Every recipe carries the global options `-y -hide_banner -loglevel error -nostdin` before its first `-i`, and the output options `-threads 1 -fflags +bitexact -flags:v +bitexact -flags:a +bitexact -map_metadata -1 -map_chapters -1` after its last `-i` (an output option placed before an input applies to that input, not to the file written), and, for H.264, `-bf 0 -g 30 -pix_fmt yuv420p -preset veryfast` unless the recipe says otherwise (D-111). AAC is `-c:a aac -b:a 128k -ar 48000 -ac 2`. `-movflags +faststart` is set for every MP4 and MOV except `ok_moov_at_end.mp4`.

**Determinism.** Two runs on one machine give byte-identical files (S2's check). Two machines may differ by encoder build; no test compares bytes across machines, only properties.

**Time.** About 40 s on D1 (assumption, S2): the 91 s, 90 s and 4K recipes dominate, at 320x180 where size does not matter.

### 17.2 Synthetic recipes

`SRC` below is `-f lavfi -i testsrc2=size=WxH:rate=R:duration=D -f lavfi -i sine=frequency=440:sample_rate=48000:duration=D`, with `W`, `H`, `R`, `D` per row. Every expected outcome is also in `fixtures/manifest.json`.

| Name | Recipe (beyond the common flags) | Expected | Read by |
|---|---|---|---|
| `ok_vfr.mp4` | `SRC` 1280x720, 30 fps, 10 s; `-vf "select='not(eq(mod(n\,4)\,3))'" -fps_mode passthrough` (every fourth frame dropped, timestamps kept) | Accepted; `is_vfr` true; `avg_fps` 22,500 ± 500 | `demux_fixtures.rs`, `rejections.spec.ts` (accept case) |
| `ok_rotated_90.mov` | `SRC` 1280x720, 30 fps, 3 s to an MP4, then `ffmpeg -display_rotation 90 -i that.mp4 -c copy ok_rotated_90.mov` | Accepted; `R90` (or `R270` by ffmpeg's sign convention, recorded at S2); display 720x1280; `Portrait` | `demux_fixtures.rs` |
| `ok_60fps.mp4` | `SRC` 1280x720, 60 fps, 3 s | Accepted; `avg_fps` 60,000 | `demux_fixtures.rs` |
| `ok_90s_exact.mp4` | `SRC` 320x180, 30 fps, 90 s | Accepted; duration 90,000 ms (TS §10.2) | `probe_rejections.rs` |
| `ok_moov_at_end.mp4` | `SRC` 320x180, 30 fps, 3 s, without `+faststart` | Accepted | `demux_fixtures.rs` |
| `ok_portrait_3x4.mp4` | `SRC` 1080x1440, 30 fps, 3 s | Accepted; `Portrait` | `demux_fixtures.rs` (`framing.rs` in V4) |
| `ok_two_video_first_disabled.mp4` | Two `testsrc2` inputs (320x180 and 640x360) and one sine; `-map 0:v -map 1:v -map 2:a -disposition:v:0 0 -disposition:v:1 default` | Accepted; the video track is the 640x360 one (TS §15.4, H2). That ffmpeg marks the first track disabled is (assumption, S2); if it does not, the row is removed from the manifest and H2 is covered by an inline test only | `demux_fixtures.rs` |
| `rej_too_long_91s.mp4` | `SRC` 320x180, 30 fps, 91 s | `REJECT_DURATION` | `probe_rejections.rs`, `rejections.spec.ts` |
| `rej_4k.mp4` | `SRC` 3840x2160, 30 fps, 1 s, `-preset ultrafast` | `REJECT_RESOLUTION` | same |
| `rej_hevc.mov` | `SRC` 1280x720, 30 fps, 2 s; `-c:v libx265 -tag:v hvc1 -x265-params log-level=error` (add `:pools=1:frame-threads=1` if two runs differ, recorded) | `REJECT_HEVC` | same |
| `rej_av1.mp4` | `SRC` 320x180, 30 fps, 2 s; `-c:v libaom-av1 -cpu-used 8 -row-mt 0` (or `libsvtav1 -preset 12` when the build lacks libaom, recorded) | `REJECT_VIDEO_CODEC` | same |
| `rej_vp9.webm` | `SRC` 320x180, 30 fps, 2 s; `-c:v libvpx-vp9 -deadline realtime -c:a libopus` | `REJECT_CONTAINER` | same |
| `rej_no_audio.mp4` | `testsrc2` only, 320x180, 2 s; `-an` | `REJECT_NO_AUDIO` | same |
| `rej_two_audio.mp4` | `SRC` 320x180, 2 s, plus a second `sine=frequency=880`; `-map 0:v -map 1:a -map 2:a` | `REJECT_MULTI_AUDIO_TRACK` | same |
| `rej_mp3_audio.mp4` | `SRC` 320x180, 2 s; `-c:a libmp3lame -b:a 128k` | `REJECT_AUDIO_CODEC` | same |
| `rej_120fps.mp4` | `SRC` 320x180, 120 fps, 2 s | `REJECT_FRAME_RATE` | same |
| `rej_no_video.m4a` | `sine` only, 2 s, AAC | `REJECT_NO_VIDEO` | same |
| `rej_truncated.mp4` | `SRC` 320x180, 2 s with `+faststart` to a temporary file, then its first 1,500 bytes (`head -c 1500`): the cut falls inside `moov`, which starts after a 32-byte `ftyp` and is several kB long | `REJECT_CORRUPT` | same |
| `rej_silent.mp4` | `testsrc2` 320x180, 5 s, with `anullsrc=channel_layout=stereo:sample_rate=48000` | `REJECT_NO_SPEECH` (D-111) | `rejections.spec.ts` (D1 only) |
| `rej_h264_444.mp4` | `SRC` 1280x720, 30 fps, 2 s; `-c:v libx264 -profile:v high444 -pix_fmt yuv444p` | `REJECT_DECODE_UNSUPPORTED` in Chrome (D-78); rules 1-10 pass | `probe_rejections.rs` (with `decode_supported = false`), `rejections.spec.ts` |

`REJECT_FILE_SIZE` has no file: the test builds it in the page (D-79). `bad_audio_payload.mp4` and `bad_video_payload.mp4` (TS §26) are V7's and are not generated now.

### 17.3 Derived speech recipes (D-76, D-100)

`REF` is the reference clip of step 2 (74,705 ms, 1280x720, about 30 fps VFR). Times are in seconds of `REF`, taken from the sentence table of `fixtures/speech/README.md` ("The V2 reference clip as recognized"). Every derived clip is re-encoded at a constant 30 fps (`-vf fps=30`), with AAC as above, and carries no metadata of `REF` (D-100).

| Name | Cut of `REF` | Video | White frames (output frame numbers) | Duration | Expected (README is the authority) |
|---|---|---|---|---|---|
| `speech_60s_portrait.mp4` | 0.960 to 60.960 | `crop=ih*9/16:ih,scale=1080:1920:flags=lanczos` | 150 and 1650, by `drawbox=x=0:y=0:w=iw:h=ih:color=white:t=fill:enable='eq(n\,150)+eq(n\,1650)'` | 60,000 ms | Accepted, portrait; sentences 1-14; four events: NumberReveal `3,000 hours`, KeywordPop `brutal` (if the prosody carries it), FromTo `90 minutes` to `10 minutes`, ListReveal `3 REASONS` |
| `speech_60s_landscape.mp4` | 0.960 to 60.960 | `scale=1920:1080:flags=lanczos` | 150 and 1650 | 60,000 ms | Accepted, landscape; the same events |
| `speech_20s_noevents.mp4` | 46.600 to 61.000 | `scale=1280:720` | none | 14,400 ms | Accepted; sentences 12-14 ("I record a clip …", "If a caption is wrong …", "Now I publish every week …"); zero events (assumption, S2: confirmed by running it) |
| `speech_20s_long_pause.mp4` | 9.160 to 14.160, then 5.000 s of room tone, then 18.100 to 28.100 | `scale=1280:720`; the video of the pause is `REF` 14.160 to 19.160 with its audio replaced | 60 (2.0 s) and 450 (15.0 s) | 20,000 ms | Accepted; words of the first part end before 5,000 ms, words of the second part start at 10,000 ms or later; export 600 frames (V5) |
| `speech_30s_sparse.mp4` | Five pieces of `REF`, each 1.100 s, from 9.160, 29.040, 46.840, 55.420, 69.120, placed at 0.5, 7.0, 13.5, 19.5 and 26.0 s; room tone elsewhere | `scale=1280:720`; the video between the pieces is `REF` continuing from each piece's end | 15 (0.5 s) and 885 (29.5 s) | 30,000 ms | Accepted; under 20% speech; export 900 frames (V5) |

**Room tone** is `anoisesrc=color=pink:amplitude=0.001:sample_rate=48000:seed=7` (about -60 dBFS, deterministic seed), mixed to stereo: a generated stand-in for a recorded room, named as such in the README (D-76). The pieces are joined with the `concat` filter in one `filter_complex`, so the cut points are sample-exact; no crossfade is applied (the fixture has hard cuts because it is a fixture; the product never cuts anything).

### 17.4 `fixtures/manifest.json` and `STAMP`

```json
{
  "schema": 1,
  "fixtures": [
    { "name": "rej_hevc.mov", "kind": "synthetic",
      "expect": { "reject": "REJECT_HEVC" },
      "probe": { "video_codec": "hevc", "width": 1280, "height": 720 },
      "tests": ["probe_rejections.rs", "rejections.spec.ts"] },
    { "name": "ok_vfr.mp4", "kind": "synthetic",
      "expect": { "accept": { "orientation": "landscape", "is_vfr": true, "duration_ms": 9967, "avg_fps_milli": [22000, 23000] } },
      "probe": { "video_codec": "h264", "width": 1280, "height": 720 },
      "tests": ["demux_fixtures.rs", "rejections.spec.ts"] },
    { "name": "speech/speech_60s_portrait.mp4", "kind": "derived",
      "expect": { "accept": { "orientation": "portrait", "is_vfr": false, "duration_ms": 60000 } },
      "probe": { "video_codec": "h264", "width": 1080, "height": 1920, "frames": 1800 },
      "pcm": true,
      "tests": ["loudness_target.rs", "duration_preserved.rs", "pipeline-preview.spec.ts", "device-bench.ts"] }
  ]
}
```

One entry per fixture of 17.2 and 17.3 (25 in all). `expect.accept` holds only the fields a test asserts; `duration_ms` is the presented video duration (D-34) that ffprobe confirms at step 8 (the `ok_vfr.mp4` value above is the one S2 records). `STAMP`:

```json
{ "ffmpeg": "9.0.2", "script_sha256": "<hex>", "manifest_sha256": "<hex>", "reference_sha256": "<hex>", "generated": 25 }
```

### 17.5 The labeled set (D-87)

**`fixtures/labeled/SCHEMA.md`** defines one JSON file per transcript (the comments below explain the fields; the files hold none):

```json
{
  "schema": 1,
  "id": "w07-saas-pricing",
  "source": "written",                       // "written" | "asr"
  "author": "agent",                         // "agent" | "founder"
  "frozen": "2026-10-16",                    // the date of S2's commit; a later change needs a changelog line
  "sentences": ["We went from $2k to $20k a month.", "That is *huge* for a team of two."],   // written only
  "words": null,                             // asr only: [{ "text", "startMs", "endMs", "confidence" }]
  "prosody": null,                           // asr only: { "per_word": [{ "energy_z", "pitch_z" }] }
  "expected": [
    { "kind": "from_to", "say": "from $2k to $20k", "occurrence": 1,
      "from": { "value": 2000, "unit": "usd" }, "to": { "value": 20000, "unit": "usd" } },
    { "kind": "keyword_pop", "say": "huge", "occurrence": 1 }
  ],
  "notes": "change-verb frame absent; from-frame present"
}
```

| Field | Rule |
|---|---|
| `sentences` (written) | Words are the whitespace-separated pieces. A word wrapped in `*…*` is emphasized: the asterisks are removed and the word gets `energy_z = 2.0`, `pitch_z = 2.0`; every other word 0.0, 0.0 (D-87; section 23, item 27) |
| Word timings (written) | Synthesized by the test, deterministically: a word lasts `clamp(120 + 45 x letters, 150, 700)` ms; 80 ms between words of a sentence; 400 ms after a word ending in `.` `?` `!`; the first word starts at 500 ms. Confidence 1.0 (TE-2) |
| `words`, `prosody` (asr) | Taken from the app's IndexedDB `transcripts` store by `run_corpus.ts` in `transcripts` mode (17.6), for the founder's two clips only (D-112) |
| `expected[].say` | A run of words, matched against the transcript's words lower-cased with punctuation stripped; `occurrence` picks the n-th match. The match's word range is the expected anchors |
| `expected[].kind` | `number_reveal`, `list_reveal`, `from_to`, `keyword_pop` |
| `expected[]` values | `value` and `unit` for a NumberReveal; `from` and `to` for a FromTo; `count` for a ListReveal; none for a KeywordPop. A `label` or `header` may be given and is reported, not matched |
| True positive (TS §17.6) | Same kind; anchors' intersection-over-union at least 0.5; for NumberReveal and FromTo the values equal (and the units, after inheritance) |
| `tags` (optional) | Array of strings that states the file's coverage, so the coverage check below can count it under `deny_unknown_fields`: `X1`-`X5` (one per trap, repeated for two traps), `announcer`, `marker_only`, `from_frame`, `change_verb`, `no_change_pair`, `edit_number`, `hidden_word`, `partial_list`, `years_from_to` |
| `edits` (optional) | Object from word index (as a string) to the replacement text; `""` hides the word (TS §17.6) |
| Null data | A `source: "asr"` file whose `words` is `null` is skipped by `precision_recall.rs`, printed as "skipped (no data yet)", and not counted toward the 30 files |
| `say` matching | Lower-case; keep letters, digits, `$` and `%`; every other character becomes a space; collapse spaces (the normalisation of the WER mode, 17.6) |

**Coverage the set must have** (asserted by `precision_recall.rs` before it computes anything):

| Requirement | Minimum |
|---|---|
| Files | 30 read (29 written + `a01-scriptA.json`; `a02-v3testclip.json` is committed at S2 with null data, skipped until S12 fills it, then makes 31) |
| Expected NumberReveal / ListReveal / FromTo / KeywordPop | 8 / 8 / 8 / 8 |
| Files with no expected event | 6 |
| Each exclusion X1-X5 present as a trap (a number that must not become an event) | 2 each |
| Edge cases of TS §17.6 a person would label the same way | Years in a from-to; a partial list; an edit that changes a number (`"edits"` in the file); a hidden word inside a list item (`"edits": { "<word>": "" }`). The frameless `$2k to $20k` is a `rules_table.rs` row, not a label (see the file plan) |
| List announcer forms and marker-only forms | 5 and 3 |
| FromTo with a from-frame; with a change-verb frame; two-quantity sentences that are no change (traps) | 4; 4; 2 |

**The 29 written files**, `w01` to `w29`, each 4 to 10 sentences of a technical creator talking to camera. Topics, to keep them varied: SaaS pricing, a coding bootcamp, home-office gear, a newsletter's growth, cloud costs, a fitness habit, a camera setup, freelancing rates, learning Rust, an app launch, a YouTube channel's first year, a productivity system, a data-pipeline migration, sleep and focus, a podcast workflow, an open-source project, interview tips, a side project's revenue, editing workflow, battery life of laptops, writing a book, an AI tool review, home studio acoustics, version upgrades (the X2 traps), a conference trip (dates and clock times: X1, X3), a support hotline (X4), a recipe (X5 small numbers), a list of mistakes (announcer form), morning routine (marker-only form). No file quotes a real person or names a real company's figures.

**The file plan.** What each written file must contain. The sentences themselves are written at S2; this table fixes what they cover, so that the coverage check above passes by construction and nobody writes a file to suit the detector.

| File | Topic | Expected events | Traps (no event expected) | Edits |
|---|---|---|---|---|
| `w01-saas-pricing` | Pricing a small SaaS | FromTo with a from-frame (two dollar amounts per month); NumberReveal of a dollar amount; KeywordPop on one emphasized word | - | - |
| `w02-coding-bootcamp` | A coding bootcamp | ListReveal, announcer form, ordinal markers ("three lessons"); NumberReveal of a time span of weeks over 10 | - | - |
| `w03-home-office` | A desk setup | NumberReveal of a price over $1,000; KeywordPop | X5: a bare small number ("2 cats") | - |
| `w04-newsletter` | Growing a newsletter | FromTo with a change verb (percent to percent); NumberReveal of a subscriber count over 1,000 | - | - |
| `w05-cloud-costs` | Cutting a cloud bill | FromTo with a from-frame; FromTo with a change verb ("cut … to") in another sentence | X2: "version 2" of a tool | - |
| `w06-fitness` | A fitness habit | ListReveal, marker-only form, three ordinal markers a few seconds apart | X5: "3 sets" said as a bare number | - |
| `w07-camera` | A camera upgrade | NumberReveal of a dollar price; KeywordPop | A "4K camera" (4,000 with no unit scores 0.70) | - |
| `w08-freelance-rates` | Freelance rates | FromTo with a change verb ("raised … to", two dollar rates); NumberReveal of a yearly dollar amount | - | - |
| `w09-learning-rust` | Learning Rust | KeywordPop | X2: "Rust 1.80"; X2: "version 2" | - |
| `w10-app-launch` | Launching an app | ListReveal, announcer form ("five steps"), ordinal markers; NumberReveal of downloads over 1,000 | - | - |
| `w11-first-year` | A channel's first year | FromTo with a from-frame (subscriber counts); NumberReveal of a dollar amount | - | One edit that changes a number to another valid number: the expectation is for the edited value (D-109) |
| `w12-productivity` | A productivity system | None | - | - |
| `w13-data-migration` | Moving a data pipeline | FromTo with a change verb (hours to hours); KeywordPop | - | - |
| `w14-sleep-focus` | Sleep and focus | None | X3: "at 10 pm" | - |
| `w15-podcast` | A podcast workflow | ListReveal, marker-only form | - | - |
| `w16-open-source` | An open-source project | NumberReveal of stars over 1,000 | X1: "since 2019" | - |
| `w17-interview-tips` | Interview tips | ListReveal, announcer form, digit markers ("4 tips. 1, … 2, …") | - | - |
| `w18-side-project` | A side project's revenue | FromTo with a from-frame (monthly dollars); KeywordPop | - | - |
| `w19-editing-workflow` | An editing workflow | None | A sentence with two quantities of incompatible units ("$5,000 on gear and 20 hours of setup"): no FromTo | - |
| `w20-battery-life` | Laptop battery life | FromTo with a from-frame (hours); FromTo with a change verb (minutes) in another sentence | - | - |
| `w21-writing-a-book` | Writing a book | NumberReveal of a word count over 1,000; KeywordPop | Two quantities of incompatible units in one sentence: no FromTo | - |
| `w22-ai-tool-review` | Reviewing an AI tool | KeywordPop | X2: "GPT 4" | - |
| `w23-studio-acoustics` | Home studio acoustics | None (the list is hidden by the edit) | X5: a bare small number | One `""` edit inside the second list item: TS §17.6, the list goes |
| `w24-version-upgrades` | Upgrading devices and tools | None | X2: "iPhone 15 Pro"; X2: "Python 3.12" | - |
| `w25-conference-trip` | A conference trip | None | X1: "in 2026"; X3: "at 9 am" | - |
| `w26-support-hotline` | A support hotline | None | X4: two phone-like digit runs | - |
| `w27-recipe` | A quick recipe | None | X5: "2 eggs"; X5: "3 spoons" | - |
| `w28-mistakes` | Mistakes made early | ListReveal, announcer form ("five mistakes"); FromTo with a change verb | - | - |
| `w29-morning-routine` | A morning routine | ListReveal, marker-only form; KeywordPop | - | - |
| `a01-scriptA` | Script A as recognized (V2's transcript), written out with `source: "written"` and no emphasis marks | NumberReveal x2, FromTo (from-frame), ListReveal (announcer). No KeywordPop: the written form carries no prosody, so "literally brutal" scores 0.70 and is not expected | The bare numbers `8`, `90`, `10`, `three` alone | - |
| `a02-v3testclip` | The second recording as recognized (`source: "asr"`; `words`, `prosody` and `sentences` null until S12) | What a person expects from the founder's words, written at S2 (D-99) | As the founder's words show | - |

Counted from the table, the 29 written files alone: NumberReveal 10, ListReveal 7 (4 announcer, 3 marker-only), FromTo 11 (5 from-frame, 6 change-verb), KeywordPop 9, files with no event 8, each exclusion X1-X5 at least twice, two non-change two-quantity sentences, one edit of a number, one hidden word. With `a01`: NumberReveal 12, ListReveal 8 (5 announcer), FromTo 12 (6 from-frame), KeywordPop 9, which meets every minimum above; `a02` adds to these once its data is filled. The coverage table's rows "FromTo with a from-frame, with a change-verb frame, and with no frame (a trap)" and "Each edge case of TS §17.6" are read against this table: the frameless "$2k to $20k" case lives in `rules_table.rs`, not here, because a person hearing it naturally might expect a from-to, and a label must say what a person expects.

**What is frozen.** S2's commit holds every written file, `a01`, and `a02`'s labels (its `expected`, from the founder's words); `a02`'s `words` and `prosody` are filled at S12 by `CORPUS_MODE=transcripts`, a change of data, not of labels, recorded as such in `v3changelog.md`. From then on, a change to any label needs a `v3changelog.md` line naming the file, the label and the reason, and "the detector disagrees" is not a reason (D-87).

### 17.6 The corpus tooling (D-88, D-112)

**`corpus/manifest.json`**

```json
{
  "schema": 1,
  "clips": [
    { "id": "founder-01", "sha256": "<hex of speech_scriptA_landscape_720p.mp4>", "tags": ["webcam", "vfr", "landscape"], "expect": "accept", "owner": "founder" },
    { "id": "founder-02", "sha256": "<hex of v3testclip.mp4>", "tags": ["webcam", "vfr", "low-fps", "landscape", "location-tag"], "expect": "accept", "owner": "founder" }
  ]
}
```

`expect` is `accept` or a `REJECT_*` code; `owner` is `founder` or `lent`. A lent clip's transcript is never written anywhere (D-112). Tags come from TS §26's list (`vfr`, `rotated`, `noisy`, `accent`) plus `webcam`, `phone`, `screen`, `low-fps`, `landscape`, `portrait`, `location-tag`.

**`corpus/fetch_corpus.sh`**: copies each manifest clip from `$OFFCUT_CORPUS_DIR/<id>.<ext>` (or, for the two founder clips, from `testclips/`) into `corpus/.clips/<id>.mp4`, checks its SHA-256, and prints a line per clip: `ok`, `missing` or `hash mismatch`. No network. Exit 0 when at least one clip is ready.

**`corpus/run_corpus.ts`**: a Playwright test file, project `corpus` of `web/playwright.config.ts` (`testDir: "../corpus"`, headed off, one worker, `MEDIA_USE`, a dependency on `media-setup`). One test per clip present in `corpus/.clips/`. Modes, by `CORPUS_MODE`:

| Mode | Per clip | Writes |
|---|---|---|
| `demux` (default; TE-12) | Drop the clip; wait for `accepted`, `rejected` or `failed` | `corpus/.results/<date>-demux.json`: per clip `{ id, outcome: "accept" \| reject code \| error code, expected, match }` |
| `export` (V9 adds it to TE-12; built now) | As `demux`, then wait for `ready`, seed a Creator token, export, run the verifier checks 1-6 | Per clip also `{ verifier: [6 booleans] }` |
| `wer` (E-10; smoke in V3) | As `demux`, then wait for `ready`; read `transcripts[clipId]` from IndexedDB in the page; compare its words with `corpus/.clips/<id>.txt` (a local reference text, never committed) after normalization: lower-case; remove every character but letters, digits, `$`, `%` and spaces; remove commas inside digit groups; collapse spaces | Per clip `{ id, words_ref, words_hyp, substitutions, deletions, insertions, wer }`: counts only, no text |
| `transcripts` (labeling, D-87) | For an `owner: "founder"` clip only: wait for `ready`; read `transcripts[clipId]` | The `words` and `prosody` fields of the clip's labeled file, by a fixed map: `founder-01` to `a01-scriptA.json` (not written: `a01` stays in the written form), `founder-02` to `a02-v3testclip.json` (D-99's name). A clip of `owner: "lent"`, or one not in the map, is refused and nothing is written |

Every result file holds ids, codes, booleans and numbers only. **No page content of a corpus clip is kept by Playwright:** the `corpus` project sets `screenshot: "off"`, `video: "off"`, `trace: "off"` (as `media` and `bench` do, V2 CL item 41), and a failing corpus test reports its clip's id and outcome code only, so that no page snapshot, caption or feed line of a lent clip reaches `web/test-results/` or `web/playwright-report/` (both git-ignored, but still files on disk that could be shared). A summary of the latest `demux` and `wer` runs (counts and rates, no ids of lent clips) is copied by hand into `docs/v3/experiments.md`.

### 17.7 `fixtures/speech/README.md` changes

| Section | Change |
|---|---|
| The table of clips | Each of the five TS clips is marked "derived in V3 from the reference clip (D-76)" with its cut; `v3testclip.mp4` gets a row: "V3 test clip, founder's recording of 2026-10-13, not in git (D-99)" |
| Script A, expected events | V3 expects all five events on the reference clip (NumberReveal `3,000 hours` with label `editing by hand`; KeywordPop `brutal`, if the measured prosody carries it; FromTo `90 minutes` → `10 minutes`; ListReveal `3 REASONS` with its three items; NumberReveal `$12k` with label `in sales`). The KeywordPop is written as conditional because the recognized text is "literally brutal" and the score needs prosody (section 9.6) |
| "Expected feed" | V3: "Transcribing…", "Cleaning voice", "Found: 3,000 hours", ("Found: brutal"), "Found: 90 minutes to 10 minutes", "Found: 3-item list", "Found: $12k", in an order fixed by the pipeline: "Cleaning voice" comes first or second (it starts before ASR, D-90), the "Found:" lines in time order |
| New: "The derived clips as recognized" | Filled at S8 from one run of each derived clip on D1 (V2 cannot show a transcript or three of the kinds) |
| New: "The V3 test clip as recognized" | The founder's words at S2 (D-99); at S8 the recognized sentences with times, the numbers, the events V3 detects, and where the transcript differs from what was said |

---

## 18. Root configuration, scripts, CI, hosting

Only what changes.

### 18.1 Cargo workspace

| File | Change |
|---|---|
| `Cargo.toml` | `[workspace.dependencies]` gains `nnnoiseless = { version = "<pinned>", default-features = false }` and `ebur128 = { version = "<pinned>", default-features = false }` (D-89). No member is added |
| `crates/offcut-dsp/Cargo.toml` | `nnnoiseless = { workspace = true }`, `ebur128 = { workspace = true }`; dev: `sha2 = { workspace = true }` |
| `crates/offcut-mp4/Cargo.toml` | dev: `sha2 = { workspace = true }` |
| `crates/offcut-text/Cargo.toml` | dev: `proptest = { workspace = true, features = ["std"] }` |
| `crates/offcut-detect/Cargo.toml` | dev: `serde = { workspace = true, features = ["derive"] }`, `serde_json`, `proptest` |
| `deny.toml` | The license allowlist gains what `cargo deny check licenses` names for the two crates' trees (BSD-3-Clause expected); every entry with its crate in a comment. No ban is loosened |
| `clippy.toml` | Unchanged |

Dependency edges stay those of V2 §22.1; `check-file-tree.mjs`'s crate-pair table is unchanged because V3 adds no edge between workspace crates.

### 18.2 Root `package.json` scripts

| Script | Does |
|---|---|
| `fixtures` | `sh fixtures/gen_fixtures.sh` |
| `corpus` | `sh corpus/fetch_corpus.sh && pnpm --filter web exec playwright test --project=corpus` |
| `test` | Unchanged text. Its `cargo test --workspace` now reads `fixtures/.gen/`: on a machine where `pnpm fixtures` has not run, the six fixture-reading test files fail with "run `pnpm fixtures`" (section 6.4), which is the intended message |

### 18.3 Web configuration files

| File | Change |
|---|---|
| `web/playwright.config.ts` | `MEDIA_SUITES` gains `rejections.spec.ts`. A fifth project, `corpus`: `testDir: "../corpus"`, `testMatch: "run_corpus.ts"`, `dependencies: ["media-setup"]`, `timeout: MEDIA_TIMEOUT_MS x 2`, `fullyParallel: false`, `workers: 1`, `use: { ...MEDIA_USE, screenshot: "off", video: "off", trace: "off" }` (17.6, D-112) |
| `web/tests-e2e/tsconfig.json` | `include` gains `../corpus` (Node types for the runner, as for `../bench`, D-63) |
| `web/eslint.config.js` | One `MINTING` entry (D-103). Nothing else |
| `.gitignore` | `fixtures/.gen/`, `corpus/.clips/`, `corpus/.results/` (D-98) |

### 18.4 Scripts

| Script | Contract change |
|---|---|
| `scripts/check-copy-codes.mjs` | `COPY_PENDING` loses the 14 `REJECT_*` entries and their comment. After V3 it holds the `E_*` codes of V2 §22.4 less none (E_DSP stays, V4) and the four blockers (V5, V6) |
| `scripts/check-file-tree.mjs` | No code change. The tree it parses gains `edit_list.rs`, `cardinals.rs` and `docs/v3/*` (S1) |
| `scripts/check-external-facts.mjs` | One line for the Linux ffmpeg archive (D-94): its URL and the date its checksum was taken, to re-verify with the other free-tier facts |

### 18.5 `ci.yml`

The step numbers are TS §33's, as in V2 §22.5. Changes:

| Where | Change |
|---|---|
| `env` | `FFMPEG_LINUX_URL`, `FFMPEG_LINUX_SHA256` (D-94) |
| Before step 4 (`cargo test`) | **"Install ffmpeg"**: `curl` the archive, `sha256sum --check`, extract, put its `bin` on `GITHUB_PATH`, print `ffmpeg -version`'s first line. **"Restore the reference clip"**: `actions/cache` of `fixtures/.cache` with the key of `e2e-media.yml` (V2), so the clip is downloaded at most once per key. **"Generate the fixtures"**: `pnpm fixtures`, with `VITE_ASSET_BASE_URL` already in `env` |
| Step 4 | Unchanged command. The log must show `demux_fixtures`, `probe_rejections`, `loudness_target`, `duration_preserved`, `chain_no_clicks` and `prosody` running (S11's check) |
| Everything else | Unchanged, including the test-key guard and the deploy order |

### 18.6 `e2e-media.yml`

| Where | Change |
|---|---|
| After "Install ffmpeg" | Move "Restore the model files and the reference clip" here (it is needed by the generator), then **"Generate the fixtures"**: `pnpm fixtures` |
| The Playwright step | Unchanged command, `pnpm e2e:media --grep @no-gpu`: it now also runs the 15 tagged cases of the rejection suite (D-93: the 13 import-time codes, "start over" and the accept case of `ok_vfr.mp4`), 23 cases in all; the job's 30-minute limit stays (the rejection cases take seconds each). The report kept on failure (`playwright-report-media`) is a public artifact of a public repository: it may hold only synthetic fixtures, never a derived speech clip or a founder recording |
| Comment at the head | Names the rejection suite among what the runner runs |

### 18.7 Hosting

`web/vercel.json` and `render.yaml` are not edited. Nothing is uploaded to the asset host in V3 (section 3.3). The deployed bundle grows by `audio.worker.ts` and the chain's code inside `offcut_core.wasm` (`nnnoiseless` carries its model weights, about 100 kB, compiled in; assumption, S6 records the size). `check-hosts.mjs` reads the new bundle: if `nnnoiseless` or `ebur128` carries a URL string (a repository link in an error text), it is added to `NON_NETWORK_LITERALS` with a `chunk` scoped to `offcut_core_bg-*.wasm` and its reason (D-38), not allowed everywhere.

---

## 19. Tests, case by case

### 19.1 What runs where

| Level | Files in V3 | Runs in | Needs |
|---|---|---|---|
| Rust unit (inline) | Sections 6.1, 7.3-7.10, 8.7, 9.9 | `cargo test --workspace`, CI step 4 | Nothing |
| Rust integration, pure | `numbers_table.rs`, `numbers_prop.rs`, `sentences.rs`, `rules_table.rs`, `redetect.rs`, `determinism.rs` (detect) | Same | Nothing |
| Rust integration, labeled | `precision_recall.rs` | Same | `fixtures/labeled/*.json` (committed) |
| Rust integration, generated fixtures | `demux_fixtures.rs`, `probe_rejections.rs`, `loudness_target.rs`, `duration_preserved.rs`, `chain_no_clicks.rs` and `prosody.rs` (each through one speech PCM case, the rest synthetic) | Same, after `pnpm fixtures` (D-94) | `fixtures/.gen/` with a matching `STAMP` |
| TypeScript unit | V1's and V2's files, unchanged | `vitest run`, CI step 7 | Nothing |
| E2E non-media | `landing.spec.ts` | CI step 9, Linux | Nothing new |
| E2E media, no GPU (`@no-gpu`) | V2's 8 cases; 15 cases of `rejections.spec.ts` (the 13 import-time codes, "start over", the `ok_vfr.mp4` accept case) | CI step 10, Windows (D-72, D-93) | `pnpm fixtures` on the runner |
| E2E media, GPU | Everything else in `model-download`, `pipeline-preview`, `export-creator`, `rejections` (`REJECT_NO_SPEECH`) | `pnpm e2e:media` on D1 before every merge (D-72) | A keyed build; `pnpm fixtures`; `testclips/` for the D1-only cases |
| Corpus | `corpus/run_corpus.ts` | `pnpm corpus` on D1, by hand (S12) | `OFFCUT_CORPUS_DIR` |
| Device benchmark | `bench/device-bench.ts` | `pnpm bench:device` on D1, by hand (S12) | A keyed build |
| Native timing | `chain_no_clicks.rs::chain_timing` (`#[ignore]`) | `cargo test -p offcut-dsp --release -- --ignored chain_timing` on D1 (S4) | `fixtures/.gen/speech/speech_60s_portrait.mp4.pcm48.f32` |

Test files BP assigns to later versions are not written now: `display_list_snapshots.rs`, `layout_safe_area.rs`, `framing.rs`, `determinism.rs` (scene), `golden_frames.rs`, `rpc.test.ts`, the machine tests, `run-pipeline.test.ts`, `preview-sync.spec.ts`, `review-edit.spec.ts`, `cancel.spec.ts` (V4), `timeline-preserved.spec.ts` (V5). The derived pause fixtures exist now so that V5 finds them.

**Hardware and length gating.** A media case that needs a processed clip has no `@no-gpu` tag and runs on D1 only (D-72). A case that needs `testclips/v3testclip.mp4` calls `v3TestClip()` and `test.skip(clip === null, "testclips/v3testclip.mp4 is on D1 only")`. `chain_timing` is `#[ignore]` because it measures, not asserts, and a debug build would be meaningless. `precision_recall.rs` runs everywhere: it is pure and takes under a second.

### 19.2 `crates/offcut-mp4/tests/demux_fixtures.rs`

Fixtures: the `ok_*` files of 17.2 and the derived speech clips, read whole with the helper of section 6.4 and opened with `MemReader`.

| Case | Expect |
|---|---|
| Every `ok_*` file and every derived speech clip | `Demuxer::open` succeeds; `probe` then `validate_probe(_, true)` gives `Ok(ClipInfo)` |
| `ok_vfr.mp4` | `is_vfr` true; `avg_fps` within the manifest's range; every video sample's `pts` strictly increasing after sorting by presentation |
| `ok_rotated_90.mov` | Rotation `R90` or `R270` (the value S2 recorded); display 720x1280; `Portrait` |
| `ok_60fps.mp4` | `avg_fps` 60,000 ± 100 |
| `ok_moov_at_end.mp4` | Opens; `moov` offset greater than `mdat` offset (read with `boxes::read_header`) |
| `ok_portrait_3x4.mp4` | 1080x1440, `Portrait` |
| `ok_two_video_first_disabled.mp4` (when in the manifest) | The video track is 640x360 |
| `speech_60s_portrait.mp4` | Duration 60,000 ± 1 ms (D-34); 1,800 video samples; audio `mp4a.40.2`, 48,000 Hz, 2 channels |
| `speech_20s_long_pause.mp4` | Duration 20,000 ± 1 ms |
| Each `ok_*`: every video sample read in order | `read_video_sample` returns `size` bytes for each; the keyframe flags match `stss` |
| `keyframe_at_or_before(t)` on `ok_60fps.mp4` for t = 0, 499, 500, 2,999 ms (`-g 30` at 60 fps: a keyframe every 500 ms) | 0, 0, 30, 150 |
| The derived clips' metadata (D-100) | No `udta` box with a `©xyz` (location) or `©day` atom, and no `meta` box whose `keys` hold `com.apple.quicktime.location.ISO6709` or `creationdate`, anywhere in the file (walked with `boxes::children`) |

### 19.3 `crates/offcut-mp4/tests/probe_rejections.rs`

Conformance with the PS §9.4 rejection table (TS §26). Each `rej_*` file of 17.2 except `rej_silent.mp4` (decided by ASR).

| Case | Expect |
|---|---|
| `rej_vp9.webm` | `Demuxer::open` gives `NotIsoBmff`, which the binding maps to `REJECT_CONTAINER` |
| `rej_truncated.mp4` | `open` gives `Truncated` or `Malformed`, mapped to `REJECT_CORRUPT` |
| `rej_no_video.m4a` | `validate_probe` gives `NoVideo` |
| `rej_hevc.mov` | `Hevc` |
| `rej_av1.mp4` | `VideoCodec` |
| `rej_no_audio.mp4` | `NoAudio` |
| `rej_two_audio.mp4` | `MultiAudioTrack` |
| `rej_mp3_audio.mp4` | `AudioCodec` |
| `rej_too_long_91s.mp4` | `Duration` |
| `rej_4k.mp4` | `Resolution` |
| `rej_120fps.mp4` | `FrameRate` |
| `rej_h264_444.mp4` with `decode_supported = true` | `Ok` (rules 1-10 pass) |
| `rej_h264_444.mp4` with `decode_supported = false` | `DecodeUnsupported` |
| `ok_90s_exact.mp4` | `Ok`: 90,000 ms is inside the limit (TS §10.2) |
| The mapping table | A table in the test of `(fixture, expected code)` read from `fixtures/manifest.json`; the test fails if a `rej_*` entry of the manifest has no case, so a new fixture cannot be forgotten |

The mapping of `ContainerError` to a code is in `media_api.rs` (V2 §6.1, frozen). The test reproduces it in one private function and asserts the same codes; `rejections.spec.ts` checks the real binding in the browser.

### 19.4 `crates/offcut-dsp/tests/loudness_target.rs` (P4, P5)

Six synthetic signals made in the test (no file), three speech PCM files from `fixtures/.gen/speech/`, and the guard-clamp count asserted across all of them: ten cases.

| Case | Expect |
|---|---|
| 1 kHz sine, peak -30 dBFS, 20 s | Loudness within ±0.5 LU of -14; peak at most -1.0 dBFS |
| Pink noise from a fixed-seed LCG, -40 dBFS RMS, 20 s | Same |
| Amplitude-modulated noise (4 Hz, speech-like), -20 dBFS RMS, 20 s | Same |
| A full-scale square wave at 200 Hz, 10 s | Peak at most -1.0 dBFS; loudness within ±1 LU of -14 (PS §20.1): after the one corrective pass of step 6 the limiter may hold it under the target; the reported loudness equals a fresh measurement of the output |
| 1 kHz sine, peak -50 dBFS, 20 s | Gain capped at +24 dB: loudness = input + 24 within ±0.5 LU (P4's exception) |
| `speech_60s_portrait.mp4.pcm48.f32` | Within ±1 LU of -14 (PS §20.1), and within ±0.5 LU unless capped; peak ≤ -1.0 dBFS; every sample finite |
| `speech_20s_long_pause.mp4.pcm48.f32` | Same |
| `speech_30s_sparse.mp4.pcm48.f32` | Same: a clip that is 80% room tone is measured on its gated speech (BS.1770 gating) |
| Pure digital silence, 10 s | Gain 0 dB; output is silence; `loudness` reported as `absolute_silence`; `Ok`, not an error (TS §18.4) |
| The limiter's guard clamp | Touched 0 samples in every case above |

### 19.5 `crates/offcut-dsp/tests/chain_no_clicks.rs` (P6, P7, P8, errors)

| Case | Expect |
|---|---|
| A 440 Hz sine at -20 dBFS, 5.5 s | At every 1 s block boundary `b`, `|out[b] - out[b-1]|` ≤ `CLICK_FACTOR` x the RMS of `out[b-480 .. b+480]` |
| White noise, fixed seed, -25 dBFS, 3.2 s | Same |
| `speech_60s_portrait.mp4.pcm48.f32` | Same, at all 59 internal block boundaries |
| Empty input | `Err(Empty)` |
| One `NaN` at sample 1,000; one `+inf` | `Err(NonFinite)` |
| `on_block` returns `Cancel` on its third call | `Ok(None)`; called exactly three times (P7) |
| The same input twice, with denoise and without | Byte-identical `samples` both times (P8) |
| `chain_timing` (`#[ignore]`) | Prints the time of `run_chain` on the 60 s portrait PCM and the samples per second; asserts nothing |

### 19.6 `crates/offcut-dsp/tests/duration_preserved.rs` (P1, P2, P3; INV-10)

The test names of TS §34.3, verbatim.

| Case | Expect |
|---|---|
| `length_equals_input_fixtures` | For each speech PCM of `fixtures/.gen/speech/`: `out.samples.len() == pcm48.len()` |
| `prop_length_equals_input` | proptest: lengths 1 to 300,000 (including 47,999, 48,000, 48,001, 479, 480, 481 and every non-multiple of the block and frame sizes), random content in ±0.5: the length rule holds, every sample finite |
| `speech_silence_speech_stays_20s` | Synthetic 0-5 s speech-like noise, 5-10 s digital silence, 10-20 s speech-like noise: output 960,000 samples; the output's RMS over 5.1-9.9 s is at least 40 dB below that over 0-5 s and 10-20 s (the silence stays where it was); never 720,000 samples (15 s) |
| `impulses_stay_aligned` | An impulse train (one impulse every 0.5 s on a -40 dBFS noise floor): each output impulse's largest sample is within `ALIGN_TOLERANCE` (48 samples) of its input position. This is the test that fixes `DENOISE_DELAY_SAMPLES` at S4 |
| `mostly_silent_clip_keeps_length` | 95% silence, 5% tone, 60 s: length unchanged; the tone's position unchanged within 48 samples |
| `digital_silence_keeps_length` | 90 s of zeros: length 4,320,000; all output zero; `Ok` |
| `long_pause_fixture_keeps_its_pause` | `speech_20s_long_pause.mp4.pcm48.f32`: length unchanged; the output's RMS over 5.2-9.8 s stays under -50 dBFS (room tone, perhaps lower after denoise) and over 0.5-4.5 s above -40 dBFS |

### 19.7 `crates/offcut-dsp/tests/prosody.rs`

Synthetic words: sawtooth bursts with a known F0 and level, separated by silence, with `Word`s whose times match the bursts.

| Case | Expect |
|---|---|
| Five words at 120 Hz, -20 dBFS; the fourth at -8 dBFS | The fourth's `energy_z` is the largest and ≥ 1.5; the others ≤ 0.5 |
| Five words at -20 dBFS; F0 120, 120, 200, 120, 120 Hz | The third's `pitch_z` is the largest and ≥ 1.5 |
| A word of white noise (unvoiced) among voiced words | Its `pitch_z` is 0.0 |
| A sentence of two words | Both 0.0 / 0.0 |
| A word whose range lies past the end of the audio | Energy floor; `energy_z` computed with the others; no panic |
| `per_word.len()` | Equals `words.len()` for 0, 1 and 150 words |
| The F0 estimate itself (via a test-only call through the public function: one sentence of three equal words at F0 f, compared through z-scores) | For f in 80, 150, 300 Hz, three words at f, f x 1.06, f x 1.12 (one and two semitones up) give strictly increasing `pitch_z` |
| Speech PCM of the portrait clip with its sentence table from the README (hand-copied times of sentences 3-4) | The word "brutal" (or "literally brutal" as recognized) has `energy_z` or `pitch_z` above 1.0 if the reading carried it; the result is printed and **not asserted**: it is the measured input of the KeywordPop expectation (section 17.7) |
| Same inputs twice | Equal |
| Budget | `measure_prosody` on 60 s with 150 synthetic words: printed time; asserted under 2 s in release, not in debug |

### 19.8 `crates/offcut-text/tests/numbers_table.rs`

The authority for parsing and display (section 8.5). Each row: the words (whitespace-separated, as recognized), the tokens consumed, the value, the unit, the display.

| Words | Used | Value | Unit | Display |
|---|---|---|---|---|
| `10,000` | 1 | 10,000 | none | `10,000` |
| `3.5` | 1 | 3.5 | none | `3.5` |
| `$10,000` | 1 | 10,000 | usd | `$10k` |
| `$1,500` | 1 | 1,500 | usd | `$1,500` |
| `$950` | 1 | 950 | usd | `$950` |
| `$9.99` | 1 | 9.99 | usd | `$9.99` |
| `40%` | 1 | 40 | percent | `40%` |
| `12.5%` | 1 | 12.5 | percent | `12.5%` |
| `10k` | 1 | 10,000 | none | `10,000` |
| `$10k` | 1 | 10,000 | usd | `$10k` |
| `$1.5m` | 1 | 1,500,000 | usd | `$1.5m` |
| `$2m` | 1 | 2,000,000 | usd | `$2m` |
| `$1.25m` | 1 | 1,250,000 | usd | `$1.25m` |
| `$3b` | 1 | 3,000,000,000 | usd | `$3b` |
| `€500` | 1 | 500 | eur | `€500` |
| `£250k` | 1 | 250,000 | gbp | `£250k` |
| `₹12,345` | 1 | 12,345 | inr | `₹12,345` |
| `3x` | 1 | 3 | times | `3x` |
| `ten times faster` | 2 | 10 | times | `10x` |
| `three times` | 1 | 3 | none | `3` |
| `ten thousand` | 2 | 10,000 | none | `10,000` |
| `two hundred and fifty` | 4 | 250 | none | `250` |
| `ten million` | 2 | 10,000,000 | none | `10 million` |
| `2 million` | 2 | 2,000,000 | none | `2 million` |
| `1.5 billion` | 2 | 1,500,000,000 | none | `1.5 billion` |
| `three point five` | 3 | 3.5 | none | `3.5` |
| `three point five million` | 4 | 3,500,000 | none | `3.5 million` |
| `zero point zero five` | 4 | 0.05 | none | `0.05` |
| `a thousand` | 2 | 1,000 | none | `1,000` |
| `a hundred people` | 3 | 100 | count people | `100 people` |
| `five grand` | 2 | 5,000 | usd | `$5k` |
| `twelve thousand dollars` | 3 | 12,000 | usd | `$12k` |
| `12,000 dollars` | 2 | 12,000 | usd | `$12k` |
| `$12` `,000` (two words, D-42) | 3 | 12,000 | usd | `$12k` |
| `$500 dollars` | 2 | 500 | usd | `$500` |
| `$500 euros` | 1 | 500 | usd | `$500` |
| `40 percent` | 2 | 40 | percent | `40%` |
| `40 per cent` | 3 | 40 | percent | `40%` |
| `3000 hours` | 2 | 3,000 | hours | `3,000 hours` |
| `1 hour` | 2 | 1 | hours | `1 hour` |
| `90 minutes` | 2 | 90 | minutes | `90 minutes` |
| `800 ms` | 2 | 800 | milliseconds | `800 ms` |
| `thirty seconds` | 2 | 30 | seconds | `30 seconds` |
| `1.5 days` | 2 | 1.5 | days | `1.5 days` |
| `500 MB` | 2 | 500 | megabytes | `500 MB` |
| `2 gigabytes` | 2 | 2 | gigabytes | `2 GB` |
| `512 bytes` | 2 | 512 | bytes | `512 bytes` |
| `10,000 subscribers` | 2 | 10,000 | count subscribers | `10,000 subscribers` |
| `2 million views` | 3 | 2,000,000 | count views | `2 million views` |
| `3 reasons` | 1 | 3 | none | `3` (a list noun is no count noun, D-101) |
| `ten pounds` | 1 | 10 | none | `10` (D-102) |
| `2 TB` | 1 | 2 | none | `2` |
| `forty-two` | 1 | 42 | none | `42` |
| `fifteen hundred` | 2 | 1,500 | none | `1,500` |
| `one` | 1 | 1 | none | `1` |
| `half` | - | `None` | | |
| `5-10` | - | `None` | | |
| `10:30` | - | `None` | | |
| `$3x` | - | `None` | | |
| `₹2,00,000` | - | `None` | | |
| `million` | - | `None` | | |
| `the` | - | `None` | | |

Display-only rows (`format_quantity` called directly): `(999,999, none)` → `999,999`; `(1,000,000, none)` → `1 million`; `(1,000,000, usd)` → `$1m`; `(999,000, usd)` → `$999k`; `(1,000, gbp)` → `£1k`; `(0.1, percent)` → `0.1%`; `(1, bytes)` → `1 byte`; `(1, minutes)` → `1 minute`; `(2.25, times)` → `2.25x`; `(999e12, usd)` → `$999t`. Larger values do not occur: they are above the parser's bound.

### 19.9 `crates/offcut-text/tests/numbers_prop.rs`

| Property | Generator | Expect |
|---|---|---|
| Digits round trip | Integers 0 to 10^12; written with separators | `parse_quantity(tokenize(format))` gives the same value, for `Unit::None` |
| Currency round trip | Integers 1 to 10^9, `$` | Parsing `format_quantity(v, Usd)` gives `v` when the display is exact (whole thousands, or under 1,000, or whole millions); otherwise the parse of the display is within 0.5% of `v` (the `k` and `m` forms round) |
| Spelled round trip | Integers 0 to 999,999,999 spelled by a test-only speller | `cardinal` gives the value and consumes every word |
| No panic | Random token sequences of up to 30 tokens from a vocabulary of number words, digits, symbols and fillers | `parse_quantity` returns `None` or `Some((n, q))` with `1 ≤ n ≤ tokens.len()` and `q.value` finite |
| Display is the formatter's | Any `Some((_, q))` | `q.display == format_quantity(q.value, &q.unit)` |

### 19.10 `crates/offcut-text/tests/sentences.rs`

| Case | Expect |
|---|---|
| "Hi." (0-400), "There" (500-900), "now" (1,700-2,000) | Three sentences (V2) |
| A question mark and an exclamation mark | Each ends a sentence |
| A gap of exactly 700 ms | Ends a sentence; 699 ms does not |
| 41 words, equal 100 ms gaps, no punctuation | Two sentences, split after word 1 (the earliest of equal gaps): 1 and 40 words |
| 41 words with one gap of 600 ms after word 25 | Split after word 25 |
| 120 words with no long gap | Every sentence has at most 40 words; every word is in exactly one; order kept |
| Empty input | No sentence |
| `start_ms` and `end_ms` of every sentence | Those of its first and last word |

### 19.11 `crates/offcut-detect/tests/rules_table.rs`

The TS §17.6 edge cases and one row per rule of section 9, on written transcripts built with the same timing synthesis as `precision_recall.rs` (a 40-line helper copied into this file). Prosody flat unless the row says. Each row: input → events (kind and display, or none).

| Input | Expect |
|---|---|
| "Last month those clips earned $12,000 in sales." | NumberReveal `$12k`, label `in sales`, 1.00 |
| "I had spent 3000 hours editing by hand." | NumberReveal `3,000 hours`, label `editing by hand` |
| "We have 3000 users." | NumberReveal `3,000 users`; score 1.00 (count ≥ 100) |
| "I have 3 tools." | None: "tools" is a list noun, never a count noun (D-101), so `3` is bare and X5 zeroes it; it would score 0.50 anyway. No list: an announcer with no markers |
| "It costs 40% less." | NumberReveal `40%`, 0.80 |
| "About 900 people came." | NumberReveal `900 people`, 0.80 |
| "About 900 came." | None (0.50) |
| "The rough cut went from 90 minutes to 10 minutes." | FromTo `90 minutes` → `10 minutes`, 1.00; no NumberReveal for either (anchors taken) |
| "Revenue went from $2k to $20k." | FromTo, 1.00 |
| "$2k to $20k." | No FromTo (0.75, TS §17.6). Two NumberReveal candidates of 1.00 whose windows overlap: `$2k`, the earlier, is kept and `$20k` dropped (TS §17.5 step 2) |
| "From 2019 to 2024 I worked there." | None (X1 on both parts; TS §17.6) |
| "Costs dropped 40% to 25%." | FromTo `40%` → `25%`: change verb + same unit + close: 0.55 + 0.15 + 0.15 + 0.05 = 0.90 |
| "There are three reasons it worked. First, … Second, … Third, …" | ListReveal `3 REASONS`, 1.00, three items |
| "There are three reasons. First, … Second, …" (no third) | None (TS §17.6) |
| "Here are 4 tips. 1, … 2, … 3, … 4, …" | ListReveal `4 TIPS`, 0.90 |
| "First, … Second, … Third, …" (no announcer, 5 s apart) | ListReveal, header empty, 0.85 |
| "First, … Second, … Third, …" with a 20 s gap between second and third | None (0.65) |
| "It was the first time. Then the second one came." | None (no marker at a sentence start) |
| "That is literally *brutal*." | KeywordPop `brutal`, 1.00 |
| "That is literally brutal." (flat) | None (0.70) |
| "*Workflow* matters." | KeywordPop `workflow`, 0.90: 0.50 + 0.20 (energy 2.0) + 0.20 (pitch 2.0); written emphasis per section 23, item 27 |
| "Upgrade to Python 3.12 now." | None (X2) |
| "The iPhone 15 Pro is here." | None (X2) |
| "Meet me at 5 pm." | None (X3) |
| "Call 5551234 today." | None (X4) |
| "I ate 3 apples." | None (X5) |
| Two KeywordPops in one sentence (both emphasized) | One, the higher score (then the earlier) |
| Two KeywordPops 2,000 ms apart in two sentences | One, the higher score |
| Four NumberReveals with currency inside 9 s, each 2.5 s apart | At most three (TS §17.5 step 3); the lowest score dropped, the later on a tie |
| A NumberReveal inside a list item ("First, we saved $10,000.") | The list is kept; the number is dropped (anchors intersect a higher priority) |
| Any event in any row | `confidence ≥ threshold(kind)` (INV-6) |
| An edit replacing word "$12,000" with "$20,000" in the first row | NumberReveal `$20k` with the same id as before (D-47, D-109; its toggle carries over, section 9.9) |
| An edit hiding "Second," in the list row | None for the list (X7) |
| Empty transcript | No event |

### 19.12 `crates/offcut-detect/tests/precision_recall.rs`

| Part | Rule |
|---|---|
| Input | Every `fixtures/labeled/*.json` (deserialized with `serde_json`, `deny_unknown_fields`); written files turned into `RawWord`s and prosody (section 17.5), asr files taken as they are, both through `offcut_text::normalize_transcript(raw, "labeled")`; `edits` applied when present |
| Coverage check | The table "Coverage the set must have" of section 17.5; a shortfall fails with the missing item named |
| Run | `detect(&t, &p, &edits, &DetectorConfig::default())` per file |
| Matching | Greedy per file and kind: each prediction is matched to the unmatched expected event of the same kind with the highest IoU ≥ 0.5 and equal values (TS §17.6); unmatched predictions are false positives, unmatched expectations false negatives |
| Output (printed always) | Per kind and overall: TP, FP, FN, precision, recall; then, per kind, a threshold sweep from 0.50 to 1.00 in steps of 0.05 with precision and recall at each; then the list of false positives as `file: kind "say"` lines |
| Assertion 1 | Overall precision ≥ `DETECTOR_PRECISION_TARGET` (0.90, `limits.rs`, PS §20.1) |
| Assertion 2 (INV-6) | Every predicted event has `confidence ≥ cfg.threshold(kind)` |
| Assertion 3 | At least 30 files were read |
| On failure | The message points to section 20.3: the sweep's lowest threshold per kind that reaches 0.90 is the contingency (TS §17); labels are not edited (D-87) |

### 19.13 `crates/offcut-detect/tests/redetect.rs`

| Case | Expect |
|---|---|
| Script A transcript (as recognized), `prev = detect(…)`; no edit; `redetect_sentence` of each sentence | Equals `prev` each time (ids, order, everything) |
| Edit word 23 (`3000`) to `4000`; redetect sentence 3 | The NumberReveal of sentence 3 has value 4,000 and keeps its id (same kind and anchors, D-47; section 9.9); every other event unchanged with the same id |
| Hide word 79 ("First"); redetect sentence 9 | The ListReveal is gone (a partial list, TS §17.6); the other events unchanged |
| Edit inside sentence 8 (the announcer), the list's markers are in 9-11 | The scope reaches sentences 9-11 (within 45 s, D-86): the list is re-detected |
| Edit in sentence 1 that creates a currency amount overlapping the window of the existing `3,000 hours` event | The new candidate is dropped (the fixed event wins, section 9.8) |
| `s` out of range | Returns `prev` unchanged |
| Timing | Printed: microseconds per call over 100 calls on Script A (native, release only); budget 50 ms on R1 (assumption, M1.5), not asserted |

### 19.14 `crates/offcut-detect/tests/determinism.rs`

| Case | Expect |
|---|---|
| Every labeled file, `detect` twice | Equal outputs, including order and ids |
| proptest: random transcripts of 0 to 200 words drawn from a vocabulary of number words, list nouns, ordinals, fillers, stop words, emphasis words and digits; random prosody in ±3 | Two calls equal; every event's `anchors` inside `0..words.len()`; `span.start ≤ span.end`; events sorted by `span.start`; no two overlay windows overlap; at most one KeywordPop per sentence; `confidence ≥ threshold(kind)` |
| A transcript built twice from the same raw words | Same events |

### 19.15 Helpers: `web/tests-e2e/helpers/fixtures.ts`

```ts
export function fixturesDir(): string;                                   // OFFCUT_FIXTURES_DIR or <repo>/fixtures/.gen
export function genFixture(name: string): string;                        // path of a generated fixture; throws "run pnpm fixtures" when absent or stale (STAMP)
export function dropFile(page: Page, path: string, name?: string): Promise<void>;   // V2's dropClip with a chosen file name (default "clip.mp4")
export function dropOversize(page: Page): Promise<void>;                 // D-79: builds the 525,336,576-byte File in the page and drops it
export function v3TestClip(): string | null;                             // testclips/v3testclip.mp4 or null (D-99)
export function readIndexedDb(page: Page, store: "clips" | "transcripts" | "edits", key: string): Promise<unknown>;
export function rejectCodeOf(page: Page): Promise<string | null>;        // the data-reject attribute of clip-rejected
```

`dropClip` (V2) stays and calls `dropFile`. `readIndexedDb` opens the app's database in the page and returns the stored `Versioned` value. Nothing here reads a transcript out of the page except into the test's memory (D-112).

### 19.16 `web/tests-e2e/rejections.spec.ts` (TS §27.1, J5)

Every case: `installFakeApi`, `routeAssets`, `/app`, drop, then wait for `clip-rejected` or the player. No model is needed for the import-time cases: rejection happens before the pipeline (they carry `@no-gpu`).

| Case | Tag | Expect |
|---|---|---|
| `REJECT_FILE_SIZE` (D-79) | `@no-gpu` | `data-reject="REJECT_FILE_SIZE"`; the title and body of `messages.rejections.REJECT_FILE_SIZE`, the body holding "500" interpolated from `LIMITS`; no request carried the file's bytes; `clips/` is empty |
| `REJECT_CONTAINER` (`rej_vp9.webm`, dropped as `clip.webm`) | `@no-gpu` | That code and its copy |
| `REJECT_CORRUPT` (`rej_truncated.mp4`) | `@no-gpu` | Same pattern |
| `REJECT_NO_VIDEO` (`rej_no_video.m4a`) | `@no-gpu` | Same |
| `REJECT_HEVC` (`rej_hevc.mov`) | `@no-gpu` | Same; the body names HEVC and the action names "Most Compatible" (PS §9.4) |
| `REJECT_VIDEO_CODEC` (`rej_av1.mp4`) | `@no-gpu` | Same |
| `REJECT_NO_AUDIO` | `@no-gpu` | Same |
| `REJECT_MULTI_AUDIO_TRACK` | `@no-gpu` | Same |
| `REJECT_AUDIO_CODEC` | `@no-gpu` | Same |
| `REJECT_DURATION` (`rej_too_long_91s.mp4`) | `@no-gpu` | Same; the body holds "90" |
| `REJECT_RESOLUTION` (`rej_4k.mp4`) | `@no-gpu` | Same; the body holds "1920" |
| `REJECT_FRAME_RATE` (`rej_120fps.mp4`) | `@no-gpu` | Same; the body holds "60" |
| `REJECT_DECODE_UNSUPPORTED` (`rej_h264_444.mp4`, D-78) | `@no-gpu` | Same. Checked on D1 and on the runner at S11; if Chrome decodes the file, the case is `test.fixme` with the reason, as D-78 says |
| `REJECT_NO_SPEECH` (`rej_silent.mp4`) | none (D1) | The model is cached with `ensureModelCached`; after "Transcribing…", `data-reject="REJECT_NO_SPEECH"`; `clips/` empty |
| Every rejection above | - | Exactly one `clip_rejected` event with `reject_reason` equal to the code and no other prop; no `clip_accepted`; no `client_error`; the event body has no file name, size or duration (INV-18, TS §15.3) |
| "Start over" from a rejection | `@no-gpu` | The drop zone is back; `clips/` is empty; a second drop of a good file is accepted |
| An accepted synthetic file (`ok_vfr.mp4`) | `@no-gpu` | `clip_accepted`; the clip leaves `importing`; IndexedDB `clips[clipId]` holds `sourceBytes` equal to the file's size and `sourceHashPrefix` equal to the D-81 hash computed in Node from the file; no field holds `clip.mp4` (P-11). The case stops before the pipeline needs the model |

The 13 import-time codes run as one parameterized loop over `fixtures/manifest.json`'s `rej_*` entries except `rej_silent.mp4` (12 files, whose case needs the model and has no tag), plus the oversize case, so a fixture added to the manifest is tested without a new case. The suite has 16 cases: these 13, `REJECT_NO_SPEECH`, "start over" and the accept case; 15 carry `@no-gpu` (D-93).

### 19.17 `web/tests-e2e/pipeline-preview.spec.ts`: the V3 changes (D-91)

| Case | V2 | V3 |
|---|---|---|
| Feed | "Transcribing…" first; a "Found:" line follows; no "Cleaning voice" | "Cleaning voice" and "Transcribing…" are the first two lines, in either order (D-90); then the "Found:" lines in time order |
| Real detections only | Every "Found:" amount is in the README's expected list | Every "Found:" line equals one of the README's "Expected feed" lines for V3 (17.7); the four always-expected ones are all present; "Found: brutal" may be present or absent (prosody-dependent) |
| Events | `stage_timing`: exactly `probe_audio`, `asr`, `detect_scene`; `pipeline_done` with `n_number ≥ 1` and the others 0 | `stage_timing`: `probe_audio`, `audio_chain`, `asr`, `detect_scene` (`audio_chain` and `asr` in either order); `pipeline_done` with `n_number` 2, `n_list` 1, `n_from_to` 1, `n_keyword` 0 or 1 |
| OPFS | `clips/<id>/source` and `out48.f32` | Unchanged; plus IndexedDB `clips`, `transcripts` and `edits` hold a record for the clip, `transcripts` with `prosody.per_word.length === transcript.words.length` |
| **New, D1 only: the second recording** | - | `v3TestClip()` present: drop it; it reaches `ready`; `clip_accepted` has `orientation: "landscape"`, `duration_bucket: "lte90"`; the feed's "Found:" lines equal the README's "The V3 test clip as recognized" expected feed; the preview plays. Absent: skipped with the D-99 reason |

The other V2 cases of the file (quiet processing, preview before sign-in, pause and resume, event shape, start over, long tasks, sample clip) are unchanged and must still pass: "event shape" now also sees `audio_chain` and `clip_rejected` values, both enums.

### 19.18 `web/tests-e2e/export-creator.spec.ts`: one D1-only case (D-100)

| Case | Expect |
|---|---|
| **New: an export of the second recording carries no metadata of it** | `v3TestClip()` present: drop it, wait for `ready`, seed a Creator token, export, save the file to the system temp directory; `ffprobe -show_entries format_tags:stream_tags` on it shows no tag whose key contains `location` or `ISO6709` (case-insensitive), and no `creation_time` (other than none at all or the epoch 0 of the muxer), `title`, `comment` or `encoder` tag carrying the source's value; a byte search of the file finds neither `©xyz` nor `ISO6709` (D-100); the verifier's checks 1-6 pass with `--expected-duration-ms` from `sourceDurationMs`. The file is deleted when the case ends, also when it fails (`finally`), and no screenshot or trace of the case is kept (the media project records none, V2 CL item 41) |

The eleven V2 cases are unchanged. The Creator export of the reference clip now carries cleaned audio; the verifier's checks 1-6 do not read loudness (check 7 is V5), so no expectation changes.

### 19.19 `bench/device-bench.ts`

| Change | Detail |
|---|---|
| `STAGES` | `["probe_audio", "audio_chain", "asr", "detect_scene", "render_encode", "mux"]` (D-91) |
| `BENCH_CLIP` | `reference` (the D-64 clip, default), `portrait` (`genFixture("speech/speech_60s_portrait.mp4")`), `v3testclip` (`v3TestClip()`, refused with a message when absent). The result's `clip` field names the fixture, never a path a person chose |
| Result file | As V2 §23.10 with six stages; `bench/results/d1-<date>-<clip>.json` so that two clips can be benched on one day |

### 19.20 `corpus/run_corpus.ts` as a test

It asserts nothing about rates: it records them. Per clip it asserts only that the clip ended in `accepted` with the expected outcome or in the expected rejection, and that no `E_INTERNAL` or `E_WORKER_CRASH` occurred ("none crash", PS §20.1). A clip whose outcome differs from `expect` fails its test and is listed in the summary.

---

## 20. Experiments, the precision gate, what V3 fixes

Write each outcome into `docs/v3/experiments.md` (question, method, numbers, date, decision), replace the matching (assumption) in TS §39.3, and label every reading `d1` (D-74). A reading on a clip that is not 60 s long is written as measured and normalised to 60 s (measured x 60,000 / duration).

### 20.1 Experiments and measurements

| ID | Step | Procedure | Pass | If it fails | Recorded in |
|---|---|---|---|---|---|
| **TE-13** (native) | S4 | `chain_timing` in release on the 60 s portrait PCM, five runs, the median | Under 2 s (a warning line: WASM is slower, section 7.8) | Read section 20.2 before S6; a native time this high usually means a stage does work per sample that should be per block | `experiments.md` |
| **TE-13** (browser) | S6, S12 | The `audio_chain` stage of `pnpm bench:device` (`BENCH_CLIP=portrait`, 10 runs, median), and S6's console call for a first number | At most 4 s for 60 s of audio (TS §37, PS §20.2) | Section 20.2: `ChainConfig.denoise = false` | `experiments.md`; `bench/results/d1-*-portrait.json` |
| `DENOISE_DELAY_SAMPLES` | S4 | `impulses_stay_aligned`: try the RNNoise frame delay (480); if the test fails, read the offset the test prints and set the constant to it | The test passes | A delay that varies with content (the test fails at every constant): the denoiser is not delay-stable, and TE-13's fallback (denoise off) applies regardless of time | `config.rs`; `experiments.md` |
| **Precision gate** | S7 | `precision_recall.rs` with the default config | Overall precision ≥ 0.90 over ≥ 30 files | Section 20.3 | `experiments.md`: the printed table, the sweep, the false positives by file |
| **TE-12** (first run) | S12 | `CORPUS_MODE=demux pnpm corpus` over every clip of `corpus/manifest.json` that is present | At least 90% of the clips present demux and reach their expected outcome; none crash | Fewer than 10 clips present: "inconclusive (n clips)", re-checked in V9 (D-88). Under 90% with at least 10: the TS §15 contingency (a JS demuxer behind `SampleMeta`) is a V4 decision, recorded with each failing clip's `ContainerError` | `experiments.md`; `corpus/.results/` (local) |
| **E-3** (re-read) | S12 | `asr` stage of the bench on the portrait clip and on `v3testclip.mp4` | Median at most 20 s per 60 s of audio; the D-67 line, 145 s per 60 s, above which the fallback applies | Over 145 s per 60 s on D1: the base-size model (D-66's first fallback) is a V4 decision | `bench/results/d1-*.json`; `experiments.md` |
| **E-4** (re-read) | S12 | `render_encode` stage, same two clips | At most 90 s per 60 s (PS §20.2) | Over 150 s per 60 s: the Canvas2D path (PS §20.4), a V4 decision | Same |
| **E-10** (smoke only) | S12 | `CORPUS_MODE=wer` on `founder-01` with Script A's text as reference, and on `founder-02` with the founder's words (D-99) | Recorded; not judged (two clips by one speaker are not the 20-clip set) | - | `experiments.md`, marked "smoke, not E-10" |
| **E-10** | - | Carried to V9 (D-88) | - | - | `experiments.md` names it open |
| Chrome and `rej_h264_444.mp4` | S2, S11 | `VideoDecoder.isConfigSupported` for the fixture's `avcC`, on D1 and on the runner | `supported: false` on both | D-78's fallback chain | `v3changelog.md` |
| `rej_silent.mp4` and the recognizer | S2 | One run on D1 | Fewer than 3 words | D-111: a 2 s silent clip | `v3changelog.md` |
| The derived clips' events | S8 | One run of each on D1 | The events of section 17.3 | The README records what was found; an expectation that does not hold is changed in the README with its reason, never by changing the detector to fit | `fixtures/speech/README.md` |
| The second recording | S8, S12 | One run at S8 (transcript, events); the bench at S12 | Accepted; `ready`; export passes checks 1-6 and carries no source metadata (D-100) | A rejection or a failure here is a bug of V3 and blocks the exit | README; `experiments.md` |
| Bundle size | S6 | `offcut_core_bg-*.wasm` before and after the chain | Recorded | Over 2 MB of growth: look at which crate; record | `v3changelog.md` |

### 20.2 TE-13: the decision

| Reading (browser, D1, per 60 s of audio) | Continue | Apply the fallback, then continue | Stop |
|---|---|---|---|
| Chain median | At most 4 s | Over 4 s: set `denoise: false` in `ChainConfig::default()` (TS §18 contingency, PS §20.4: high-pass, compressor, loudness and limiter only). Re-run `loudness_target.rs` and the bench; record both | Never: the fallback always fits (the remaining stages are a few multiply-adds per sample) |
| Chain on `v3testclip.mp4` | At most 5.26 s (78.9 s of audio) | As above | - |

The decision is taken at S6 and confirmed at S12. A chain that passes on D1 may miss on R1 (D-74): the reading is recorded with that note, and V9's measurements on testers' devices re-check it (BP §11.2).

### 20.3 The precision gate: the decision

| Reading | Continue | Apply the contingency, then continue | Stop |
|---|---|---|---|
| Overall precision | ≥ 0.90 | Under 0.90: for each kind whose precision is under 0.90, raise its threshold in `config.rs` to the lowest value of the printed sweep at which that kind reaches 0.90 (TS §17 contingency). Re-run; record the old and new thresholds and every kind's recall | - |
| A kind's recall after the raise | ≥ 0.30 | Under 0.30 (assumption, E-7): keep the kind shipped and record it as the E-7 input "tune rules or reconsider the wedge" (TS §17) | - |
| Every label | Unchanged since S2 | A label is wrong (a person would not expect it): fix it with a changelog line naming the file, the label and the reason; never because the detector disagrees (D-87) | - |

The gate is read on the default config first and recorded even when it passes, so V4 knows the margin.

### 20.4 The V3 gate

V3 has no product gate of its own in BP; its exit criteria are the tests and the three experiments. The table below says what a miss means for the next version.

| Reading | Continue to V4 | Continue with a recorded fallback | Hold V4 |
|---|---|---|---|
| Rejections | All 14 codes shown with their copy; `clip_rejected` carries the code alone | `REJECT_DECODE_UNSUPPORTED` covered by the unit test only (D-78) | A file that crashes the import or is accepted when its manifest says reject |
| Voice | -14 LUFS ±1 on the speech fixtures; length preserved on every test | Denoise off (20.2) | A length change anywhere (INV-10): a bug, fixed before V4 |
| Detector | Precision ≥ 0.90 at default thresholds | Thresholds raised by the sweep (20.3) | Precision under 0.90 at every threshold of the sweep for a kind: that kind is turned off by setting its threshold to 1.01 (no event can reach it, INV-6), recorded as an E-7 input, and V4 draws three kinds instead of four |
| TE-12 | ≥ 90% of ≥ 10 clips | Inconclusive with fewer than 10 clips (D-88) | Under 90% of ≥ 10 clips: the JS demuxer decision is taken before V4's preview work |

### 20.5 What V3 fixes for later versions

| Item | Fixed by | Written to |
|---|---|---|
| Whether the denoiser runs | TE-13 | `offcut-dsp/src/config.rs` (`ChainConfig::default().denoise`) |
| `DENOISE_DELAY_SAMPLES` | `impulses_stay_aligned` | `config.rs` |
| The four thresholds and the scoring constants | The precision gate and its sweep | `offcut-detect/src/config.rs`; TS §17.4 |
| The lexicons | D-84, the labeled set | `config.rs`, `units_lex.rs`; TS §17.2, §17.4 |
| The edit-list shapes accepted | TE-12, `demux_fixtures.rs` | `edit_list.rs`; TS §15.4 |
| The display forms of every unit | `numbers_table.rs` | `numbers.rs`; TS §17.2 |
| The fixture set and its recipes | S2 | `gen_fixtures.sh`, `manifest.json`; TS §26 |
| The first committed bench file | S12 | `bench/results/d1-<date>-portrait.json` and `-v3testclip.json`: V4 compares against them |

---

## 21. Hand-over: V4 and later

### 21.1 What V3 freezes

Contracts from the end of V3. Changing one later means touching both sides of a boundary.

| Frozen in V3 | Where |
|---|---|
| The `run_chain` and `measure_prosody` signatures and postconditions P1-P8 | `offcut-dsp`, TS §34.3 |
| The `audio_api.rs` exports and their return shapes (`{ out48, loudness, peak } \| { cancelled: true }`) | Section 10.1; `CoreApi` |
| `detect` and `redetect_sentence` signatures; the event pipeline order (finders, exclusions, threshold, resolve) | `offcut-detect/src/lib.rs` |
| `DetectorConfig`'s field names (values may be tuned) | `config.rs` |
| `parse_quantity`'s grammar and `format_quantity`'s display table | `numbers.rs`, `numbers_table.rs` |
| The labeled-file format and the true-positive rule | `fixtures/labeled/SCHEMA.md` |
| The fixture names, recipes' properties and the manifest format | `fixtures/manifest.json` |
| `FeedLine` with `displayTo` | `workers/protocol.ts` (D-83) |
| The pool table with four rows | `workers/pool.ts` |
| The three repos' signatures and the `ClipRecord` layout, `sourceHashPrefix` byte layout | `persistence/*-repo.ts` (D-81) |
| `messages.rejections` keys and shape | `copy/messages.ts` |
| `clip-rejected`'s `data-reject` attribute | `RejectionPanel.tsx` |
| The bench's six stages and the result file name | `bench/device-bench.ts` |
| The corpus result format (ids, codes, numbers; no text) | `corpus/run_corpus.ts` (D-112) |

V2's freezes (V2 §25.1) all stand. The two V2 tests V3 rewrote (section 8.6, 9.9) and the three cases of D-91 are the only places a V2 expectation changed.

### 21.2 File by file

| V3 file | V4 adds | V5 adds | V6 adds | V7 adds |
|---|---|---|---|---|
| `offcut-mp4/*` | Nothing | `ctts`, the overflow test (V2) | Nothing | Nothing |
| `offcut-dsp/*` | Nothing | Nothing (verifier check 7 reads its output) | Nothing | Nothing |
| `offcut-text/*` | Nothing | Nothing | Nothing | Nothing |
| `offcut-detect/*` | Nothing (thresholds only if E-2 asks, as values) | Nothing | Nothing | Nothing |
| `offcut-wasm-core/audio_api.rs` | Nothing | Nothing | Nothing | Nothing |
| `offcut-wasm-render/detect_api.rs` | Nothing: `redetect_sentence` is ready | Nothing | Nothing | Nothing |
| `workers/audio.worker.ts` | Nothing (the cancel sender arrives with `cancel-job.ts`; the worker already honours the flag) | Nothing | Nothing | Restart of a crashed worker resumes from the last completed stage (restart budget) |
| `workers/pool.ts` | `pool.cancel`, `restartWorker` (V2 §25.2) | Nothing | Nothing | Restart budget |
| `workers/render.worker.ts` | The `redetectSentence` handler, calling `RenderApi.redetectSentence` with the stored transcript and prosody; `previewSeek` | Nothing | Nothing | Nothing |
| `workers/protocol.ts` | Nothing | Nothing | Nothing | Nothing |
| `persistence/clips-repo.ts` | Nothing | Nothing | Nothing | `list()`, `touch(clipId)`, `remove(clipId)` for recent clips and eviction |
| `persistence/transcript-repo.ts`, `edits-repo.ts` | `editsRepo.put` on every edit (`edit-word.ts`, `toggle-event.ts`, `set-style.ts`, `set-crop-offset.ts`) | Nothing | Nothing | `remove(clipId)`; read by `restore-clip.ts` |
| `usecases/import-clip.ts` | Nothing | Nothing | Nothing | `quota.ensureFree` replaces the D-40 sweep |
| `usecases/run-pipeline.ts` | `client_error` on failure; retry from the last stage (C-14); the `AbortController` aborted by `cancel-job.ts`; `run-pipeline.test.ts` | Nothing | Nothing | Nothing |
| `copy/messages.ts` | `NO_EVENTS_FOUND`; review-panel copy; `ErrorPanel` copy (`E_DSP` among them, leaving `COPY_PENDING`) | Export copy | Sign-in, plan copy | Restore, clear-data copy |
| `ui/components/RejectionPanel.tsx` | Nothing | Nothing | Nothing | Nothing |
| `ui/components/ProcessingFeed.tsx` | Cancel button | Nothing | Nothing | Nothing |
| `ui/pages/EditorPage.tsx` | `ReviewPanel`; `ErrorPanel` replaces the failed stub | `ChangeSummary` | Sign-in dialog | `RecentClips` |
| `fixtures/gen_fixtures.sh`, `manifest.json` | Nothing (`golden/` is a separate directory of PNGs) | Nothing | Nothing | `bad_audio_payload.mp4`, `bad_video_payload.mp4` (two recipes, two manifest rows) |
| `fixtures/labeled/*` | Read by `layout_safe_area.rs` (TS §26) | Nothing | Nothing | Nothing |
| `tests-e2e/rejections.spec.ts` | Nothing | Nothing | Nothing | Nothing |
| `tests-e2e/pipeline-preview.spec.ts` | Zero-event clip (`speech_20s_noevents.mp4`) shows `NO_EVENTS_FOUND`; the long-task assertion | Nothing | Nothing | Nothing |
| `tests-e2e/export-creator.spec.ts` | Nothing | Checks 7-11; the cache case | Nothing | Nothing |
| `bench/device-bench.ts` | Nothing | Nothing | Nothing | Nothing (V9: `--compare`) |
| `corpus/*` | Nothing | Nothing | Nothing | Nothing (V9: more clips, `export` mode in TE-12, E-10) |

Every row is additive: a function body, a new file, a new case, a new row. No V3 file is restructured by V4 to V7.

### 21.3 What must be true on V4's first morning

- [ ] V3 is tagged `v3`, merged into `main` and deployed; the deployed page refuses `rej_4k.mp4` with its message.
- [ ] TE-13's decision is written and `ChainConfig::default()` says it.
- [ ] The precision gate's table, sweep and thresholds are in `docs/v3/experiments.md`, and TS §17.4 holds the thresholds in use.
- [ ] `fixtures/speech/README.md` lists the derived clips' and the second recording's events: V4's golden frames and snapshots are drawn from them.
- [ ] `fixtures/labeled/` holds at least 30 files: `layout_safe_area.rs` (V4) reads them in all three styles.
- [ ] The first `d1` bench files (portrait, v3testclip) are committed: V4 compares the preview's cost against them.
- [ ] `COPY_PENDING` has no `REJECT_*` entry.
- [ ] `git grep -n "V3: " -- web/src crates` prints nothing; `git grep -n "V4: "` lists the places V4 fills (the render worker's two handlers, `run-pipeline.ts`'s cancel and retry).
- [ ] No `TODO` without a version tag exists in V3 code.
- [ ] The spec corrections of section 23 are applied.

### 21.4 What V4 inherits without further work

- All four kinds are detected, with labels, headers, items, from and to values: V4's event visuals read `EventParams` as they are.
- `redetect_sentence` exists and is tested: the review edit flow needs a worker handler and a use-case, not detection work.
- Prosody is measured and stored: KeywordPop's in-caption scale (TS §19.4) has its word.
- The cleaned voice is in `out48`: the preview and the export already play it.
- The transcript, prosody and edits are in IndexedDB: V7's restore reads them; V4's edits write them through `editsRepo.put`.
- The derived landscape and portrait clips exist: `framing.rs` and `golden_frames.rs` have their inputs.
- `speech_20s_noevents.mp4` exists: the zero-event case of `pipeline-preview.spec.ts` has its fixture.
- The rejection suite runs in CI: V4's changes to the editor cannot break it unseen.

---

## 22. Exit checklist

V3 is done when every box is ticked (BP §1.2, BP §5.4, TS §28 M1.1-M1.3).

**Code and checks**

- [ ] `pnpm check` is green: fmt, clippy, deny, ESLint, both `tsc` programs, `check-file-tree`, `check-gen-clean`, `check-copy-codes`.
- [ ] `pnpm fixtures` then `pnpm test` are green, including every inline test of sections 6-10.
- [ ] `demux_fixtures.rs`, `probe_rejections.rs` pass (BP §5.4).
- [ ] `loudness_target.rs`, `chain_no_clicks.rs`, `duration_preserved.rs`, `prosody.rs` pass; `ui.rs` still passes (D-95).
- [ ] `numbers_table.rs`, `numbers_prop.rs`, `sentences.rs`, `rules_table.rs`, `redetect.rs`, `determinism.rs` (detect) pass.
- [ ] `precision_recall.rs` at 0.90 or above on at least 30 labeled transcripts; the thresholds used are those of `config.rs` and TS §17.4.
- [ ] `pnpm build` is green, including `check-hosts.mjs`.
- [ ] `pnpm e2e` is green.
- [ ] `pnpm e2e:media` on D1, on a keyed build: every case green, including `rejections.spec.ts` (14 codes), the rewritten cases of D-91, and the D1-only cases of D-99 and D-100.
- [ ] `ci.yml` runs the six fixture-reading Rust test files (not skipped) and `e2e-media.yml` runs the 15 `@no-gpu` cases of the rejection suite; both green on the pull request.
- [ ] No source file exceeds 400 lines, tests excluded; in particular `sample_table.rs`, `numbers.rs`, `run-pipeline.ts`.
- [ ] `git grep` finds no `fetch(` outside the four allowed files, and no word "silence", "pause" or "gap" in `offcut-dsp/src/` except in comments that say what the crate does not do and in `prosody.rs`'s z-score code.
- [ ] Test counts are at least the baseline's plus V3's new ones; none fell.

**Deployment**

- [ ] `ci.yml` is green on `main`; the API (unchanged image) and the web build are deployed, server first.
- [ ] The test-key guard passed.
- [ ] The header check and the `@smoke` case pass against the deployed URL; `crossOriginIsolated === true`.
- [ ] On the deployed page on D1: dropping `rej_4k.mp4` shows the 4K message; dropping `rej_hevc.mov` shows the HEVC message; dropping the reference clip shows "Cleaning voice" and the "Found:" lines; the preview plays the cleaned voice.
- [ ] `clip_rejected` rows appear in `analytics_events` with a `reject_reason` and nothing else; `stage_timing` rows with `audio_chain`.

**Experiments and records**

- [ ] TE-13 (native and browser), the precision gate, TE-12 (or "inconclusive" with the count), the E-3 and E-4 re-reads, and the E-10 smoke reading are in `docs/v3/experiments.md`, each labelled `d1`.
- [ ] `bench/results/d1-<date>-portrait.json` and `-v3testclip.json` are committed.
- [ ] `DENOISE_DELAY_SAMPLES` holds a measured value.
- [ ] The derived clips' and the second recording's sections of `fixtures/speech/README.md` are filled.
- [ ] The founder has read five labeled files chosen at random and the reading is recorded (D-87); the founder has confirmed D-112 for `a01` and `a02`.
- [ ] Every (assumption) V3 measured is replaced by its value in TS §39.3, or still listed there.
- [ ] `docs/v3/v3changelog.md` lists every pinned version, every recipe change, every label change after the freeze, and the ffmpeg build used on each machine.
- [ ] `docs/v2/experiments.md` has its closing M0 entry (section 1A).
- [ ] Section 21.3 is fully ticked.
- [ ] Tagged `v3`.

---

## 23. Open references and spec issues found

Nothing below was resolved by guessing. Items 1-14 are gaps or contradictions in the inputs that a decision of section 2 covers; the spec should be corrected so that the decision is no longer needed. Items 15-28 stay open, each with the version that handles it; items 25-28 were found in the review of 2026-10-15, before S1.

| # | Issue | Where | Handled by |
|---|---|---|---|
| 1 | Rejection messages are to state the measured size or duration, but a rejection is a bare `RejectReason` and the types that carry it are frozen | TS §15.3, PS §9.4 vs TS §14.2 | D-80. Correct TS §15.3's "Message must state" column and PS §9.4's first example |
| 2 | One `display` string cannot say "Found: $2k to $20k" | PS §10 J6 vs TS §14.1 | D-83. Correct TS §14.1 |
| 3 | `sourceHashPrefix` has no owner and no byte layout | TS §23.1 | D-81. Correct TS §23.1 |
| 4 | An `events` store with no writer and no repo file | TS §23.1 vs TS §5, C-4 F | D-82; open for V7 (item 18) |
| 5 | `redetect_sentence`'s scope counts sentences with a time constant | TS §17.3 vs §17.4 | D-86. Correct TS §17.3 |
| 6 | "Compatible units" and `FromTo.label` undefined | TS §17.4, §10.5 | D-85. Correct TS §17.4 |
| 7 | No lexicon is given, and no list marker is defined | TS §17.2, §17.4 | D-84, D-105. TS §17.4 gains the lists by reference to `config.rs` |
| 8 | A literal reading makes small counts and "N times" score as events | TS §17.4 | D-101, D-102. Correct TS §17.4 |
| 9 | Labels such as "and stopped" follow from TS §17.4's rule and may read poorly | TS §17.4 | E-2 judges in V4's preview; the rule is not changed in V3 |
| 10 | The chain's `cfg` argument has no channel through the frozen worker protocol | TS §34.3 vs §14.2 | D-110. Correct TS §18.1 to name `ChainConfig::default()` as the only configuration |
| 11 | `ui.rs` is in M1.2's definition of done with no stated change | TS §28 | D-95. Correct TS §28 |
| 12 | Speech fixtures are to be committed, against the founder's decision to keep recordings out of git; Rust tests need decoded speech and Rust has no AAC decoder | TS §4, §26 vs D-39 | D-76, D-77. Correct TS §4 and §26: derived, generated, not committed |
| 13 | `REJECT_DECODE_UNSUPPORTED` is tested by mocking a call inside a worker | TS §15.3 | D-78. Correct TS §15.3's fixture column |
| 14 | Pitch at 48 kHz over every word is far over the prosody allowance | TS §18.2, §8 | D-108. TS §18.2 names the decimation |
| 15 | `Edit` does not read `media_rate`: a dwell edit is taken as a media edit | V2 `sample_table.rs` | Open; TE-12 counts such clips; V9 decides |
| 16 | Indian digit grouping ("2,00,000") is not parsed; INR amounts written that way are not shown | TS §17.2 | Open (assumption, M1.3); E-7 |
| 17 | Terabytes have no `Unit` variant | TS §10.4 | Open; a shared-type change is a spec revision |
| 18 | The `events` store (item 4) | TS §23.1 | V7: a writer, or the store is removed from TS §23.1 and from `schema.ts` through a version-2 migration |
| 19 | E-10 needs a 20-clip accent set that does not exist | BP §5.3, PS §19 | D-88: V9 |
| 20 | KeywordPop precision is measured on synthetic emphasis | D-87 | E-7 from V10 on, on real use |
| 21 | D1 is not R1: TE-13 and the re-read budgets pass on a machine about twice as fast | D-69, D-74 | Testers' devices after the final deploy (V2 CL open item 12) |
| 22 | The derived portrait clip is an enlarged 405-pixel strip, cheaper to decode well than a real phone clip | D-76 | A real portrait recording when the founder chooses; the file takes the same name |
| 23 | The capability check admits a software adapter, on which a clip takes over 20 minutes (V2 CL, nineteenth push) | TS §13.2 | Open; V8's `capability.spec.ts` |
| 24 | `video-source.ts` has 399 lines and V4 adds `prefetch` and seek to it (V2 CL item 36) | V2 as built | V4 must make room first; recorded here so V4's plan starts from it |
| 25 | **The cleaned audio of a clip that ends early stays in `audio.worker`.** `kept` is cleared by the next `runChain` or by `measureProsody`; after a `REJECT_NO_SPEECH`, a failure of B1 or a later step, or "start over", 17 MB of the clip's audio stays in the worker's memory until the next clip or the page closes. It never reaches storage or the network, but TS §18.4 says the worker keeps audio only until `measureProsody`. `AudioWorkerApi` (frozen) has no release call | D-90, section 14.1, TS §18.4 | Open. V4: `pool.cancel` and the worker restart (section 21.2) end the job; a `release` call needs a protocol change. Until then accepted and named here |
| 26 | **IndexedDB records outlive their clip.** The D-40 stand-in removes `clips/` from OPFS at the next import, but nothing removes the `clips`, `transcripts` and `edits` records (`remove` is V7's, section 13.5). Every processed clip's transcript and prosody stay on the device until V7; a clip refused with `REJECT_NO_SPEECH` or failed after import keeps its `clips` record. Local only (INV-2 holds), but a person who presses "start over" may expect the words gone | Sections 13.5, 15.1, 15.2 | **Decided 2026-10-18: `clear()`.** Each of the three repos gains `clear(): Promise<void>`, and import step 4 calls the three beside the D-40 sweep, so the device holds the records of one clip at most, as OPFS holds its media. The customer's view: a clip's words and voice measurements should not outlive the clip on the device, and until V7 nothing reads a record of an earlier clip. Applied in Prompt 80 (sections 13.5 and 15.1 updated there) |
| 27 | **A written emphasis lands exactly on the KeywordPop threshold.** D-87's `energy_z 2.0, pitch_z 1.5` gives 0.50 + 0.20 + 0.10 = 0.80 for an emphasized word outside `EMPHASIS_WORDS` and without an intensifier, equal to `threshold_keyword`; in `f32` the sum can round either side of 0.80 by summation order (V2 CL known issue 22). The labeled set's KeywordPops of that shape pass or fail on rounding | D-87, sections 9.6, 17.5, 19.11 | **Decided 2026-10-16 by the founder: written emphasis gets `pitch_z = 2.0`** (score 0.90), so a labeled KeywordPop does not hang on `f32` summation order. No rounding is added: the detector keeps `score >= threshold` unchanged, which errs toward a missed event rather than a wrong one ("a missed animation is fine but a wrong animation makes the edit bad"). `rules_table.rs` asserts "*Workflow* matters." at 0.90 |
| 28 | **The 40-word split's tie-break makes one-word sentences.** "The earliest of equal gaps" splits one word off at a time when gaps are equal (90 words give 50 one-word sentences and one of 40, section 8.7). Recognizer output rarely has equal gaps, but captions and FromTo/list scope follow sentences | TS §17.2, section 8.6 | Open (assumption, M1.3). A tie-break nearest the middle would give 30/30/30; the tests of 8.7 and 19.10 name the rule, so a change is one rule and two rows |

### 23.1 Cross-check: BP §5 items against sections and tests

| BP §5 item | Section | Test or record | Exit box |
|---|---|---|---|
| A `validate.rs` (11 rules) | 6 (V2 file, tested) | `probe_rejections.rs` | Code and checks |
| A `probe.rs` complete | 6.3 | `demux_fixtures.rs` | Code and checks |
| A `gen_fixtures.sh`, `manifest.json` | 17.1-17.4 | S2 determinism check; every fixture-reading test | Code and checks |
| A 14 rejection messages | 16.3 | `rejections.spec.ts`; `check-copy-codes.mjs` | Code and checks |
| A `RejectionPanel.tsx`, clip machine | 16.2 (machine: V2, D-18) | `rejections.spec.ts` | Code and checks |
| A `db.ts`, `schema.ts`, the three repos | 13.5 (db, schema: V1) | `rejections.spec.ts` (accept case), `pipeline-preview.spec.ts` (OPFS case) | Code and checks |
| A `demux_fixtures.rs`, `probe_rejections.rs`, `rejections.spec.ts` | 19.2, 19.3, 19.16 | Themselves | Code and checks |
| A corpus manifest, fetch, runner | 17.6 | TE-12 | Experiments |
| B `offcut-dsp` modules | 7 | The four DSP test files | Code and checks |
| B `audio_api.rs`, `audio.worker.ts` | 10.1, 14.1 | `pipeline-preview.spec.ts` (feed, events); TE-13 | Code and checks; Experiments |
| B Four DSP test files | 19.4-19.7 | Themselves | Code and checks |
| B Fixtures `speech_20s_long_pause.mp4`, `speech_30s_sparse.mp4` | 17.3 (derived, D-76) | `duration_preserved.rs`, `loudness_target.rs` | Code and checks |
| C `numbers.rs`, `units_lex.rs`, `sentences.rs`, `normalize.rs` | 8 | `numbers_table.rs`, `numbers_prop.rs`, `sentences.rs` | Code and checks |
| C `from_to.rs`, `list.rs`, `keyword.rs`, `exclusions.rs`, `resolve.rs`, `config.rs` | 9 | `rules_table.rs`, `precision_recall.rs` | Code and checks |
| C `fixtures/labeled/*.json` (30+), `SCHEMA.md` | 17.5 | `precision_recall.rs` coverage check | Experiments (founder's reading) |
| C Text and detect test files | 19.8-19.14 | Themselves | Code and checks |
| C `run-pipeline.ts` complete | 15.2 | `pipeline-preview.spec.ts` | Code and checks |
| C `ProcessingFeed.tsx` complete | 16.2 | `pipeline-preview.spec.ts` (feed, real detections) | Code and checks |
| Non-code: 30 labeled transcripts | 17.5 (D-87) | `precision_recall.rs` | Experiments |
| Non-code: remaining speech fixtures | 17.3 (D-76) | S2 check | Code and checks |
| Non-code: 20-clip ASR set, 40+ corpus | 17.6 (D-88) | E-10 smoke, TE-12 first run | Experiments |
| Experiments TE-12, TE-13, E-10 | 20 | `experiments.md` | Experiments |
| Exit: tagged `v3` | 22 | - | Experiments and records |
| Analytics: `clip_rejected` (BP Appendix C) | 15.1, 15.2 | `rejections.spec.ts` | Deployment |

### 23.2 Decisions to copy back

**Into `technicalspec.md`:** D-73 and D-92 (§5 tree); D-76, D-77, D-111 (§4, §26); D-78 (§15.3 fixture column); D-80 (§15.3 message column); D-81 and D-82 (§23.1); D-83 (§14.1); D-84, D-85, D-101, D-102, D-105, D-106, D-107 (§17.2, §17.4); D-86 (§17.3); D-89 (§3); D-90 (§9 C-4); D-94 (§33); D-95 (§28); D-96, D-108 (§18.2, §18.3); D-104, D-109 (§17.2, §17.6); D-110 (§18.1, §34.3); D-74 beside D-69 (§28, §30, §37); D-100 (§25.1, a new line on recordings used as fixtures). §17.6's edit bullet: an edit that changes a number keeps the event's id when its kind and anchors are unchanged (D-47), and so its toggle; the old "a new id is created; its toggle state is not inherited" is replaced (V3 changelog, open item 14).

**Into `buildplan.md`:** D-74 (§5.3: the experiments are read on D1); D-76 (§5.1 B and §5.2: the speech fixtures are derived, nothing is recorded); D-87 (§5.2: the labeled set is written, frozen before the detector); D-88 (§5.3: E-10 moves to V9); D-93, D-94 (§5.4: where the rejection cases run).

**Into `product.md`:** D-80 (§9.4: the examples state the limit; the measured value is not shown). Nothing else.

---

## Appendix A. Per-file specifications in brief (BP §1.3, TS §34.2)

Before a file is written, its per-file spec is written from `docs/file-specs/TEMPLATE.md` (BP §1.3). The table gives each V3 file's spec in one row: what the implementer is handed besides TS §2, §7, §10, §11 and the generated types (TS §34.1). "Section" is the section of this document that is the file's contract.

| Path | Purpose | Allowed dependencies | Section | Judged by | Budget |
|---|---|---|---|---|---|
| `crates/offcut-mp4/src/edit_list.rs` | Resolve an edit list to an offset and a presented length | `crate::sample_table::Edit`, `crate::ContainerError` | 6.1 | Inline tests; `demux_fixtures.rs` | - |
| `crates/offcut-mp4/src/sample_table.rs` | Step 5 delegates to `edit_list` | As V2 | 6.2 | V2's inline tests | Under 400 lines |
| `crates/offcut-mp4/src/probe.rs` | The hardening of H1-H8 | As V2 | 6.3 | `demux_fixtures.rs`; inline | - |
| `crates/offcut-mp4/tests/demux_fixtures.rs` | Every accepted fixture demuxes and probes right | `offcut-mp4`, `offcut-types`, `sha2` (dev) | 19.2 | Itself | - |
| `crates/offcut-mp4/tests/probe_rejections.rs` | The PS §9.4 table, fixture by fixture | Same | 19.3 | Itself | - |
| `crates/offcut-dsp/src/config.rs` | Every DSP constant; `ChainConfig` | `offcut-types` | 7.2 | Inline (defaults) | - |
| `crates/offcut-dsp/src/highpass.rs` | 80 Hz Butterworth high-pass | `crate::config` | 7.3 | Inline | Part of the chain's 4 s |
| `crates/offcut-dsp/src/denoise.rs` | RNNoise with delay compensation and wet/dry mix | `nnnoiseless`, `crate::config` | 7.4 | Inline; `impulses_stay_aligned` | Part of the chain's 4 s |
| `crates/offcut-dsp/src/compressor.rs` | Soft-knee feed-forward compressor | `crate::config` | 7.5 | Inline | Same |
| `crates/offcut-dsp/src/loudness.rs` | Integrated loudness, peak, gain | `ebur128`, `offcut-types`, `crate::config` | 7.6 | Inline; `loudness_target.rs` | Same |
| `crates/offcut-dsp/src/limiter.rs` | Delay-free look-ahead limiter | `crate::config` | 7.7 | Inline; `loudness_target.rs` | Same |
| `crates/offcut-dsp/src/chain.rs` | `run_chain` | The modules above | 7.8 (TS §34.3) | `duration_preserved.rs`, `loudness_target.rs`, `chain_no_clicks.rs` | 4 s per 60 s on R1, read on D1 |
| `crates/offcut-dsp/src/prosody.rs` | `measure_prosody` | `offcut-types`, `crate::config` | 7.9 | `prosody.rs` (test) | Under 1 s (assumption) |
| `crates/offcut-dsp/src/lib.rs` | Module list and re-exports | - | 7.1 | Compiles | - |
| The four DSP test files | P1-P8; prosody | `offcut-dsp`, `offcut-types`, `proptest`, `sha2` | 19.4-19.7 | Themselves | - |
| `crates/offcut-text/src/tokenize.rs` | The wider `Digits` | As V2 | 8.1 | `numbers_table.rs`; inline | - |
| `crates/offcut-text/src/cardinals.rs` | Spelled cardinals, "point" decimals | none | 8.2 | `numbers_table.rs`, `numbers_prop.rs` | - |
| `crates/offcut-text/src/numbers.rs` | `parse_quantity`, `format_quantity` [ONLY] | `crate::cardinals`, `crate::units_lex`, `crate::tokenize`, `offcut-types` | 8.3, 8.5 | `numbers_table.rs`, `numbers_prop.rs` | - |
| `crates/offcut-text/src/units_lex.rs` | Unit, currency and count words | `offcut-types` | 8.4 | Inline (sorted, unique) | - |
| `crates/offcut-text/src/sentences.rs` | The 40-word split | As V2 | 8.6 | `sentences.rs` (test) | - |
| `crates/offcut-text/src/normalize.rs` | Numbers per sentence | As V2 | 8.6 | Inline; `precision_recall.rs` indirectly | - |
| `crates/offcut-detect/src/config.rs` | Every threshold and lexicon [ONLY] | `offcut-types` | 9.1 | Inline | - |
| `crates/offcut-detect/src/number.rs` | NumberReveal candidates; `quantities` | `offcut-text`, `offcut-types`, `crate::config` | 9.3 | `rules_table.rs` | Detection 1 s for 150 words (PS §20.2) |
| `crates/offcut-detect/src/from_to.rs` | FromTo candidates | Same, plus `offcut_text::format_quantity` | 9.4 | `rules_table.rs` | Same |
| `crates/offcut-detect/src/list.rs` | ListReveal candidates; markers | Same | 9.5 | `rules_table.rs` | Same |
| `crates/offcut-detect/src/keyword.rs` | KeywordPop candidates | Same | 9.6 | `rules_table.rs` | Same |
| `crates/offcut-detect/src/exclusions.rs` | X1-X7 | Same, plus `crate::list::markers` | 9.7 | `rules_table.rs` | Same |
| `crates/offcut-detect/src/resolve.rs` | TS §17.5 with a fixed set | `crate::config`, `offcut-types` | 9.8 | `rules_table.rs`, `determinism.rs`, `redetect.rs` | Same |
| `crates/offcut-detect/src/lib.rs` | `detect`, `redetect_sentence` | The modules above | 9.2 | All four test files | `redetect_sentence` 50 ms (assumption) |
| The four detect test files | Rules, precision, re-detection, determinism | `offcut-detect`, `offcut-text`, `offcut-types`, `serde`, `serde_json`, `proptest` | 19.11-19.14 | Themselves | - |
| `crates/offcut-wasm-core/src/audio_api.rs` | Bindings of the chain and prosody | `offcut-dsp`, `offcut-types`, `wasm-bindgen`, `js-sys`, `serde-wasm-bindgen`, `crate::{failure, to_js}` | 10.1 | `pipeline-preview.spec.ts` | Copy in and out: 2 x 17.3 MB for 90 s |
| `crates/offcut-wasm-render/src/detect_api.rs` | `redetect_sentence` binding | `offcut-detect`, `offcut-types`, crate helpers | 10.2 | `redetect.rs` covers the function; the binding is exercised in V4 | - |
| `web/src/workers/protocol.ts` | `displayTo` | As V1 | 13.1 | `tsc`; `pipeline-preview.spec.ts` (feed) | - |
| `web/src/wasm/load-core.ts` | `runChain`, `measureProsody` wrappers | As V2 | 13.2 | `tsc`; the media suites | - |
| `web/src/wasm/load-render.ts` | `redetectSentence` wrapper | As V2 | 13.3 | `tsc` | - |
| `web/src/workers/pool.ts` | The `audio` row | As V2 | 13.4 | The media suites | - |
| `web/src/workers/audio.worker.ts` | The chain and prosody in a worker | `wasm/load-core`, `workers/rpc`, `workers/protocol` (types), `gen/domain` (types) | 14.1 | `pipeline-preview.spec.ts`; TE-13 | 4 s per 60 s on R1 |
| `web/src/persistence/clips-repo.ts` | `hashPrefix`, `put`, `get` | `persistence/db`, `persistence/schema`, `persistence/opfs` (`OpfsError`), `gen/domain` | 13.5 | `rejections.spec.ts` (accept case) | `hashPrefix` under 50 ms for 4 MiB (assumption) |
| `web/src/persistence/transcript-repo.ts` | `put`, `get` | Same, without `opfs` beyond `OpfsError` | 13.5 | `pipeline-preview.spec.ts` (OPFS case) | - |
| `web/src/persistence/edits-repo.ts` | `put`, `get` | Same | 13.5 | Same | - |
| `web/src/usecases/import-clip.ts` | C-2 complete but for quota | As V2, plus `persistence/clips-repo` | 15.1 | `rejections.spec.ts` | - |
| `web/src/usecases/run-pipeline.ts` | C-4 complete but for retry and `client_error` | As V2, plus the two repos | 15.2 | `pipeline-preview.spec.ts`; `rejections.spec.ts` (no speech) | Under 400 lines |
| `web/src/copy/messages.ts` | The 14 rejection messages; the from-to line | `gen/domain`, `gen/api` (types and `LIMITS`) | 16.3 | `check-copy-codes.mjs`; `rejections.spec.ts` | - |
| `web/src/ui/components/RejectionPanel.tsx` | What a refused clip shows | `copy/messages`, `state/clip-store`, `usecases/import-clip`, styles | 16.2 | `rejections.spec.ts` | - |
| `web/src/ui/components/ProcessingFeed.tsx` | Passes `displayTo` | As V2 | 16.2 | `pipeline-preview.spec.ts` | - |
| `web/src/ui/pages/EditorPage.tsx` | `RejectionPanel` in the `rejected` arm | As V2, plus `RejectionPanel` | 16.1 | `rejections.spec.ts` | - |
| `web/tests-e2e/rejections.spec.ts` | J5 | Helpers | 19.16 | Itself | Each case under 30 s on the runner |
| `fixtures/gen_fixtures.sh` | Make every generated fixture | `ffmpeg`, `ffprobe`, `sha256sum`, `curl` | 17.1-17.3 | S2's determinism check | About 40 s on D1 (assumption) |
| `corpus/fetch_corpus.sh`, `corpus/run_corpus.ts` | The corpus | Shell; Playwright and the E2E helpers | 17.6 | TE-12 | - |

---

## Appendix B. Invariants V3 touches, and where each is enforced

| Invariant | V3 enforces it in | Test |
|---|---|---|
| INV-1 closed host list | No new host; `gen_fixtures.sh`'s download is a development tool on a runner, not the product; `check-hosts.mjs` reads the grown core bundle (18.7) | `check-hosts.mjs`; V2's network cases |
| INV-2 no content in requests | `clip_rejected` carries a code; the feed's `display`/`displayTo` stay local; `run_corpus.ts` writes no text of a lent clip (D-112) | `rejections.spec.ts` (event shape); `pipeline-preview.spec.ts` (event shape) |
| INV-3 import, processing, preview without the API | The chain, prosody, detection and the repos run offline | V2's quiet-processing case, unchanged |
| INV-5 timeline preservation | The chain is length-preserving and delay-compensated; nothing in V3 moves a word time | `duration_preserved.rs`; `prosody.rs` reads only |
| INV-6 no event below threshold | `detect`'s threshold step after exclusions; `resolve` never raises a score | `rules_table.rs`, `precision_recall.rs`, `determinism.rs` |
| INV-7 deterministic outputs | Stable sorts with total orders in `offcut-detect`; fixed sum order in `offcut-dsp` | `determinism.rs`; `chain_no_clicks.rs` (P8) |
| INV-10 audio duration preservation | `run_chain` P1-P3; `audio.worker` passes `out48` on untouched | `duration_preserved.rs`; the OPFS case's `out48.f32` size |
| INV-12 ASR and render sessions never coexist | Unchanged order: `unload` before `openSession`; the audio worker holds no GPU session | V2's ordering; `run-pipeline.test.ts` in V4 |
| INV-15 copy and numbers from `messages.ts` and limits | The 14 messages interpolate `LIMITS`; the from-to line has its own copy key (D-83) | `check-copy-codes.mjs`; `rejections.spec.ts` (limits in the text) |
| INV-16 every state change through `transition()` | No new state; `noSpeech()` and `rejected()` are V2's transitions | V2's stores |
| INV-17 the main thread does no media work | The chain and prosody run in `audio.worker`; `hashPrefix` is a 4 MiB WebCrypto hash, not media work (D-81) | V2's long-task record |
| INV-18 analytics without free text | `clip_rejected { reject_reason }` is an enum; `stage_timing.stage` an enum | `rejections.spec.ts`, `pipeline-preview.spec.ts` |
| P-11 no file name stored | `ClipRecord` has no name field; the accept case checks IndexedDB for `clip.mp4` | `rejections.spec.ts` (accept case) |
| TS §31 PCM transferred, never copied between threads | `pcm48` is transferred to `audio.worker` and `out48` back | `rpc.ts`'s detach check (V2) |
| TS §36 nothing from "What Not to Build" | No silence or pause logic in `offcut-dsp`; no time map | Exit checklist's `git grep`; code review |

---

## Appendix C. Two per-file specifications written out

The format of TS §34.2, for the two V3 files whose contracts are easiest to get subtly wrong. The others are written the same way at their step.

```text
Path:                 web/src/workers/audio.worker.ts
Purpose:              Run the voice chain and the prosody measure off the main thread.
Responsibility:       Holding the cleaned audio between runChain and measureProsody (TS §18.4).
Non-responsibilities: Deciding anything about the audio; silence, pauses, cuts (TS §36); storage; network.
Allowed dependencies: wasm/load-core (loadCore, CoreApi), workers/rpc (serveWorker, CANCELLED, WorkerFailure),
                      workers/protocol (AudioWorkerApi, types), gen/domain (types only).
Public API:           None besides the worker's message interface, AudioWorkerApi of TS §14.2, verbatim.
Types:                Consumes Word, Sentence, Prosody, Lufs (gen/domain); ChainResult (load-core).
Per-function contract:
  runChain({ pcm48 })            in: mono 48 kHz f32, aligned (TS §15.1). out: { loudness }.
                                 side effect: kept = out48 (out48.length === pcm48.length). cancel: CANCELLED, kept = null.
                                 errors: E_DSP (Empty, NonFinite) from the module; anything else E_INTERNAL.
  measureProsody({ words, sentences })
                                 in: the transcript's words and sentences. out: { out48, prosody }, out48 transferred.
                                 side effect: kept = null. errors: E_INTERNAL NoAudio when nothing is kept.
State ownership:      Owns `kept`. Reads nothing else. Must not touch IndexedDB, OPFS, state or ui.
Concurrency:          One job at a time (rpc). Cancellation point: every 1 s block, through the onBlock callback.
Performance:          Chain at most 4 s per 60 s on R1 (PS §20.2), read on D1 (D-74).
Security and privacy: Sees audio samples and word texts; logs nothing; sends nothing.
Constraints:          ESLint layer rules; no `as <Brand>`; under 400 lines (about 60 expected).
Definition of done:   pipeline-preview.spec.ts (feed, events, OPFS) green on D1; TE-13 recorded.
```

```text
Path:                 crates/offcut-detect/src/resolve.rs
Purpose:              Choose which candidates become events when they compete (TS §17.5).
Responsibility:       Priority, anchor intersection, display-window overlap, the 10 s overlay cap,
                      KeywordPop spacing; honouring fixed events for redetect_sentence.
Non-responsibilities: Scoring (the finders), thresholds (lib.rs step 5), exclusions (exclusions.rs).
Allowed dependencies: crate::config::DetectorConfig, crate::number::Candidate, offcut-types.
Public API:           pub fn resolve(cands: Vec<Candidate>, fixed: &[DetectedEvent], cfg: &DetectorConfig) -> Vec<Candidate>;
Per-function contract:
  in:   candidates already at or above their thresholds; fixed events (empty for detect()).
  out:  a subset of cands, in no particular order (lib.rs sorts); never a changed score or anchor.
  post: no two returned or fixed overlays have intersecting display windows;
        no two returned or fixed events have intersecting anchors;
        at most cfg.overlay_max_per_window overlay windows start inside any cfg.overlay_window_ms;
        at most one KeywordPop per sentence; KeywordPops at least cfg.keyword_min_gap_ms apart.
  determinism: the order of section 9.8 step 0 is total; equal inputs give equal outputs.
State ownership:      None.
Concurrency:          Pure; called on render.worker inside detect / redetectSentence.
Performance:          O(n log n + n x k) for n candidates and k kept; n is at most a few hundred.
Security and privacy: Sees word ranges and scores only.
Constraints:          No HashMap; no unwrap, expect, indexing; under 400 lines (about 150 expected).
Definition of done:   rules_table.rs, determinism.rs, redetect.rs, precision_recall.rs pass.
```

---

## Appendix D. Named constants V3 introduces

Every number V3 code uses is a named constant in the file that owns it. "Source" is the spec it comes from, or (assumption, ID) with the step or experiment that measures it.

| Name | Value | File | Source |
|---|---|---|---|
| `SAMPLE_RATE` | 48,000 Hz | `offcut-dsp/src/config.rs` | TS §18 (derived) |
| `BLOCK_SAMPLES` | 48,000 | same | TS §18.2 |
| `TARGET_LOUDNESS` | -14.0 LUFS | same | TS §18.3, PS §20.1 |
| `LIMIT_CEILING_DBFS` | -1.0 | same | TS §18.3 |
| `MAX_GAIN_DB` | 24.0 | same | TS §18.3 |
| `ABSOLUTE_SILENCE_LUFS` | -70.0 | same | TS §18.3 |
| `LOUDNESS_RETRY_LU` | 0.5 | same | TS §18.2 |
| `HPF_HZ` | 80 | same | TS §18.3 |
| `HPF_Q` | 1/√2 | same | Butterworth (derived) |
| `DENOISE_MIX` | 0.85 | same | TS §18.3 |
| `DENOISE_FRAME` | 480 | same | RNNoise (derived) |
| `DENOISE_SCALE` | 32,768.0 | same | RNNoise (derived) |
| `DENOISE_DELAY_SAMPLES` | 480 | same | (assumption, S4: `impulses_stay_aligned`) |
| `COMP_THRESHOLD_DB`, `COMP_RATIO`, `COMP_ATTACK_MS`, `COMP_RELEASE_MS`, `COMP_KNEE_DB` | -18, 3, 5, 80, 6 | same | TS §18.3 |
| `LIMITER_LOOKAHEAD` | 240 samples | same | TS §18.3 (5 ms) |
| `LIMITER_RELEASE_MS` | 50 | same | (assumption, M1.2) |
| `ALIGN_TOLERANCE` | 48 samples | same | TS §18.3 |
| `CLICK_FACTOR`, `CLICK_RMS_WINDOW` | 2.0, 480 | same | TS §18 acceptance (assumption, M1.2) |
| `PROSODY_DECIMATE`, `PITCH_FRAME`, `PITCH_HOP`, `PITCH_MIN_LAG`, `PITCH_MAX_LAG` | 6, 320, 80, 20, 134 | same | D-108 (assumption, M1.2) |
| `VOICING_MIN`, `VOICED_FRAME_MIN_DB`, `ENERGY_FLOOR_DB`, `PROSODY_MIN_WORDS` | 0.45, -60, -100, 3 | same | D-96, TS §18.2 |
| `MAX_EDITS` | 16 | `offcut-mp4/src/edit_list.rs` | D-60 style bound (assumption, TE-12) |
| `MAX_CARDINAL_TOKENS` | 24 | `offcut-text/src/cardinals.rs` | Derived from the grammar |
| `COUNT_UNIT_MIN` | 100 | `offcut-text/src/units_lex.rs` | D-101 (assumption, M1.3) |
| `SENTENCE_MAX_WORDS` | 40 | `offcut-text/src/sentences.rs` (V2) | TS §17.2 |
| The V3 fields of `DetectorConfig` | Section 9.1 | `offcut-detect/src/config.rs` | TS §17.4, §17.5, §19.5 (assumption, M1.3) |
| `BYTES_PER_MB` | 1,000,000 | `web/src/copy/messages.ts` | PS §9.4 as `limits.rs` reads it |
| `FPS_MILLI_PER_FPS` | 1,000 | same | `FpsMilli` (derived) |
| `HASH_PREFIX_BYTES` | 4,194,304 | `web/src/persistence/clips-repo.ts` | TS §23.1 (4 MiB) |
| `ROOM_TONE_AMPLITUDE`, `ROOM_TONE_SEED` | 0.001, 7 | `fixtures/gen_fixtures.sh` | D-76 (fixture only) |
| `OVERSIZE_PARTS`, `OVERSIZE_PART_BYTES` | 501, 1,048,576 | `web/tests-e2e/helpers/fixtures.ts` | D-79 |
