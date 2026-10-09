# v2implementation.md - Offcut V2 "Proof of concept: one clip in, one MP4 out": implementation plan

**Implements:** `buildplan.md` section 4 (V2), which is steps M0.2, M0.3 and M0.4 of `technicalspec.md` (TS) §28. Product references are to `product.md` (PS). "V1 §n" is a section of `docs/v1/v1implementation.md`; "BP §n" is a section of `buildplan.md`.

**Reader.** Whoever writes the V2 code, file by file. Everything needed to write a V2 file is in this document or in the TS section it names. Where the specs and V1 are silent and V2 needs an answer, this document decides and says so (section 2); each decision is marked **(V2 decision)**.

**The one-sentence goal.** At the end of V2, the reference clip goes in and a 1080x1920 MP4 with Clean-style captions and one NumberReveal comes out, entirely in the browser and deployed, and the two questions that can stop the project are answered with committed numbers: transcription time (E-3) and render + encode time (E-4) on R1 (BP §4).

**The second goal, equally important.** V3 and V4 must be additive. No V1 or V2 file is restructured later: later versions fill in function bodies, add files and add rows to tables. Section 25 lists, file by file, what V3 and V4 add, and what V2 does today to make that possible.

---

## 0. Contents

1. Scope: what V2 is and is not
1A. Preconditions: V1 deliverables V2 relies on
2. Decisions this document makes (D-18 to D-66)
3. Tools, accounts, secrets, environment variables
4. V2 file tree
5. Build order (16 steps, each with its own check)
6. `offcut-mp4`: demux, probe, validate, mux
7. `offcut-dsp`: resampling only
8. `offcut-text`: tokens, sentences, numbers (V2 subset)
9. `offcut-detect`: NumberReveal only
10. `offcut-entitlement`: token, verification, export profile
11. `offcut-scene`: Clean captions and NumberReveal
12. `offcut-render`: GPU compositor
13. WASM binding crates: `offcut-wasm-core` additions, `offcut-wasm-render`
14. Server: no changes
15. Web infrastructure: config, persistence, network, WASM loaders, RPC, pool
16. Workers: media, ASR, render
17. Model manager
18. State: stores, machines, blockers
19. Use-cases
20. UI, copy, styles
21. Code generation
22. Root configuration, scripts, CI, hosting
23. Tests, case by case
24. Experiments, measurements and the M0 gate
25. Hand-over: V3 and V4
26. Exit checklist
27. Open references and spec issues found

---

## 1. Scope

### 1.1 In V2

| Area | What works at the end of V2 |
|---|---|
| Ingest (happy path) | A dropped MP4/MOV is copied to OPFS in 4 MiB chunks, demuxed and probed in Rust, passed through `validate_probe`, and its AAC audio is decoded, aligned to video time 0 and resampled to 48 kHz and 16 kHz mono (TS §15.1, §15.2) |
| Model | The speech model downloads once from the asset host in resumable 8 MiB ranges into OPFS, is SHA-256-verified against the bundled manifest, and is skipped on the second session (TS §16.3, INV-20) |
| ASR | `asr.worker` transcribes the 16 kHz audio with word timestamps, loading model files from OPFS only (TS §16.4) |
| Text and detection | Sentences; digit numbers with `$` and `%`; simple spelled cardinals. NumberReveal events only, at or above the 0.80 threshold (TS §17.4, INV-6) |
| Scene and render | Clean caption style, NumberReveal visual, 9:16 framing with offset 0, Vello on WebGPU in `render.worker` (TS §19) |
| Preview | 540x960 preview driven by the audio clock: play, pause, replay. No scrubbing (TS §20) |
| Export | Frame-accurate loop: H.264 + AAC through WebCodecs, faststart MP4 from the Rust muxer, streamed to OPFS, downloaded. Output size comes from a verified entitlement token (TS §21, INV-9) |
| Verifier | `verify/verify_mp4.py` checks 1-6 (TS §27.2) |
| Workers | Three workers (`media`, `asr`, `render`) behind typed clients: request, response, failure, progress, cooperative cancel flag (TS §14) |
| CI | `e2e-media.yml` runs the media suites on `windows-latest`; `bench/device-bench.ts` writes stage timings |
| Analytics | Eight more events start: `clip_accepted`, `model_download`, `stage_timing`, `pipeline_done`, `preview_played`, `export_started`, `export_done`, `export_failed` (BP Appendix C) |
| Experiments | TE-1, TE-2, TE-3, TE-4, TE-10, TE-14, E-3, E-4 measured; the M0 gate decided on Sun 1 Nov 2026 (BP §4.3) |

### 1.2 Not in V2

| Left out | Stand-in in V2 | Arrives |
|---|---|---|
| Specific rejection messages, `RejectionPanel`, `clip_rejected`, the `rej_*` fixtures and their tests | `validate_probe` already returns the real `RejectReason` (D-21); the UI shows one stub message, `messages.editor.rejectedStub`. No analytics event is sent | V3 |
| Voice enhancement, prosody, `audio.worker` | `out48` is the unprocessed `pcm48`; prosody is all zeros (D-22) | V3 |
| ListReveal, FromTo, KeywordPop, exclusions, `resolve.rs`, unit lexicon, number labels, `redetect_sentence` | `detect()` runs `number::find`, the threshold filter and one overlap rule (D-49) | V3 |
| Bold and Tech styles, crop offset, watermark, `change_summary()` | `styles::spec()` returns the Clean row for every `StyleId`; the crop offset is read as 0; the watermark flag is ignored; `Scene::summary()` counts placed NumberReveals | V4 |
| Review controls, word edits, scrubbing, `NO_EVENTS_FOUND` copy | `EditState::default()` always; `redetectSentence` and `previewSeek` answer `E_INTERNAL` (D-32) | V4 |
| Cancel UI, cancel timeout, worker restart budget, progress throttle, `ErrorPanel`, retry from last stage, `client_error` for pipeline failures | A failure shows `messages.editor.failedStub` with one action: start over | V4 |
| Export gate (sign-in, expiry, free quota, storage), render cache, `ChangeSummary` component, post-export question, ladder restart on encoder failure, `ctts`, the `moov` overflow test | Two state blockers only (D-20); a token is read from IndexedDB or the export is unavailable (D-23) | V5 |
| Sign-in, real tokens, receipts, billing | Tests seed a Creator token signed with a test key (D-23, D-24). A real visitor to the deployed V2 can preview and cannot export. That is expected (BP §7.2 says the same of V5) | V6 |
| Clip repos, restore after reload, quota eviction, recent clips | One clip at a time, in memory; older clip directories are removed on import (D-40) | V3 (repos), V7 |
| Verifier checks 7-11 | Not implemented | V5 |

Nothing from TS §36 is built, including "temporary" versions. In particular: no silence detection, no pause handling, no time map (INV-5, INV-10), no Canvas2D backend unless TE-3 or E-4 fails.

### 1.3 Journeys and use-case steps served

| Flow | Served in V2 | Missing until |
|---|---|---|
| J2 Drop a clip, sample clip (PS §10) | Full | - |
| J4 First-run model download | Full, without a cancel button | V4 |
| J5 Validate | Rules run; one stub message | V3 |
| J6 Processing feed | "Transcribing…" and one line per NumberReveal. No "Cleaning voice" line: nothing is cleaned yet, and the feed shows real steps only (PS §10 J6) | V3 |
| J7 Preview (AHA) | Clean captions, NumberReveal, play and pause, before any sign-in. No zero-event message | V4 |
| J10 Render + export | Stage progress and the "rendering on your computer" line; token from tests only | V5, V6 |
| J11 Download | The file downloads; `export_done` fires. No summary panel | V5 |
| C-1 App start | Adds `modelManager.inspect()` at the position V1 §11.10 marked | V6, V7 |
| C-2 Drop and validate | `forImport()`, import, probe, validate, `clip_accepted`. No `quota.ensureFree`, no `clipsRepo` | V3, V7 |
| C-3 Model download | Full: resume, three retries, hash failure, quota failure | - |
| C-4 Processing | Steps A, B1, C, E and the audio half of F. B2 and D are stand-ins (D-22) | V3 |
| C-5 Preview | `attach`, `play`, `pause`, clock sync every 250 ms | V4 (`seek`) |
| C-9 Render and export | Token verify, profile, export loop, mux, progress. No cache lookup | V5 |
| C-10 Download | Download and `export_done`. No cache write, outbox, summary or question | V5, V6 |
| C-12 Cancel | Workers honour the cancel flag at every TS §14.3 cancellation point. Nothing sends it yet | V4 |

---

## 1A. Preconditions: V1 deliverables V2 relies on

Do not start S1 until every box is ticked. Items 1-9 are V1 §15.3, each with a check; the rest are what V2 additionally needs.

- [x] 1. The asset host is chosen (TE-7); its CORS serves the app origin and exposes `Content-Range`, `Accept-Ranges`, `Content-Length` (TS §24.3); its origin is in the CSP `connect-src` and `media-src` of `web/vercel.json`. Check: from the deployed page's console, a `fetch` with `Range: bytes=0-1023` to an uploaded file returns 206.
- [x] 2. `offcut_core.wasm` compiles in production under the CSP. Check: the V1 `@smoke` case passes against the deployment.
- [x] 3. The deployed page reports `crossOriginIsolated === true`.
- [x] 4. `web/src/workers/protocol.ts` type-checks against `web/src/gen/*`. Check: `tsc --noEmit` is clean.
- [x] 5. `VIDEO_ENCODE_LADDER`, `AAC_ENCODE_CONFIG`, `KEYFRAME_INTERVAL_FRAMES` and `videoConfigFor` exist in `workers/render/encoders.ts`, and `capability.ts` imports the first two.
- [x] 6. `pnpm e2e` runs against `vite preview` with the production CSP, locally and in CI (V1 D-15).
- [x] 7. `fixtures/speech/README.md` holds the 60-second reference script: about 150 words, two numbers, one three-item list, one from-to, one emphasized word (BP §2). **V2 adds one requirement, checked by reading the script:** at least one number is a dollar amount of 1,000 or more that is outside the from-to sentence and outside the list. Reason: section 9.3 (a bare number scores 0.70 and is never shown). S6 re-checks it on the real transcript: the recognizer must have written that amount with a `$` and digits. Script A passes the reading check with "twelve thousand dollars", in a sentence of its own (read on 2026-10-08). If the recognizer writes it another way, D-42 says what changes.
- [x] 8. `bench/results/` exists.
- [x] 9. Every lint boundary of TS §7 is on in `web/eslint.config.js`, `deny.toml` and `clippy.toml`. Check: `pnpm check` is green on `main`.
- [x] 10. The code, check and deployment boxes of V1 §16 hold. Check: `pnpm check && pnpm test && pnpm build && pnpm e2e` is green and `ci.yml` is green on `main` (both true on 2026-10-08, `docs/v1/v1changelog.md`). The other V1 exit boxes are **not** preconditions of V2; see "Carried over from V1" below.
- [x] 11. The eight IndexedDB stores exist at database version 1 (V1 D-6). Check: DevTools shows the `entitlement` store on the deployed page.
- [x] 12. The 32-byte public half of `ENTITLEMENT_SIGNING_KEY` is written down as base64. Check: it is derived from the seed stored in Render and decodes to 32 bytes. Needed at S1, for `config/entitlement-public-key.ts`: this is the one V1 leftover that V2 code depends on. Derived on 2026-10-08: `KnENazU2ypDUgG3mibKKiP0g4L5zgrgiWTCeCLNjeLg=`. If the key in Render is ever replaced, derive it again.
- [ ] 13. R1 is available on days 4, 7 and 10 (BP §1.5). R2 is not used in V2 (D-65).
- [ ] 14. Python with NumPy and the `ffmpeg`/`ffprobe` CLIs are installed on the development machine and R1 (development tools only, TS §36).

**Carried over from V1 (founder's decision, 2026-10-08).** V1's code is complete and deployed, and the build is ahead of the calendar of BP §1, so V2 starts now. The V1 items below are done later. None of them blocks a V2 step.

| V1 item | Effect on V2 |
|---|---|
| The `v1` tag; `docs/v1/experiments.md` with TE-5, TE-6 and TE-11 | None. `TRUSTED_PROXY_HOPS` keeps its provisional value; the server is not touched in V2 |
| TE-7 (asset host) | Measured in V2 anyway, at S5, with the real model files (section 24.1) |
| E-1 outreach, the three demo clips of Offcut, TE-9 | The M0 gate reads E-1 as inconclusive until the page has been announced (section 24.3) |
| V1 known issue 30 (the 2 s timeout of one capability check) | Measure the cold check on R1 during the S15 bench session and record it in `docs/v2/experiments.md` |
| The reset of the Neon database password (V1 open item 36) | None on the code. Do it before the page is announced |

---

## 2. Decisions this document makes

Numbering continues from V1's D-17. Each is the smallest choice that keeps V3 and V4 additive. BP §3 states the rule for schedule differences: where an implementation document and the build plan differ on which version creates a file, the implementation document is the more precise one. Decisions that move a file are marked **(schedule)**. Decisions that touch something V1 froze (V1 §15.1) or marked F are marked **REOPENED V1 CONTRACT** with the cost on both sides.

| # | Decision (all **(V2 decision)**) | Why |
|---|---|---|
| D-18 | **(schedule)** `state/machines/clip-machine.ts` and `export-machine.ts` are created in V2 holding the complete transition tables of TS §12.2. BP creates them in V3 and V5, and V1 §15.2 lists the clip machine under V3. Their test files stay in V4 and V5 | V1 §15.2 also says every V2 store uses `transition()`, which needs a table. A table inside the store would have to move to the machine file later, which is a restructure |
| D-19 | The preview machine's table lives inside `state/preview-store.ts` | TS §5 lists no preview-machine file. V1 did the same for the capability machine (V1 §11.4) |
| D-20 | **(schedule)** `state/blockers.ts` is created in V2 (BP: V5) with `BLOCKER_CODES` (all eight codes of TS §12.5), `forImport()` complete, and `forExport()` holding its two state-only conditions. `check-copy-codes.mjs` reads `BLOCKER_CODES` from this file | V1 §15.2 has `DropZone` read blockers in V2. TS §12.5 makes this file the only place for the conditions. Blocker codes are in no generated array (V1 §9), so the copy check needs a source (section 27, item 6) |
| D-21 | **(schedule)** `offcut-mp4/src/validate.rs` is created in V2 (BP: V3) with all 11 rules of TS §15.2. V3 adds `probe_rejections.rs`, the fixtures, the 14 messages, `RejectionPanel` and `clip_rejected` | BP §4.1 A already puts `probe_and_validate` in V2's `media_api.rs`, and `ClipInfo` may be constructed only by `validate_probe` (TS §10.8). The full body is about 40 lines and keeps oversized input out of the decoders during TE-14 |
| D-22 | V2 has no `audio.worker` (BP: V3). `run-pipeline.ts` uses `pcm48` as `out48` and builds a neutral `Prosody`: `per_word.len() == words.len()`, every z-score 0.0. `pool` exposes three clients and preloads three worker scripts. `ChangeSummary.voice_cleaned` is `false` | V1 §15.2 says "four worker scripts"; only three exist in V2 (section 27, item 3). V3 replaces two marked lines of `run-pipeline.ts` and adds one row to the pool table |
| D-23 | **(schedule)** In V2 the entitlement token is read from the IndexedDB `entitlement` store through `persistence/entitlement-repo.ts`, created now (BP: V6) with `get()` and `put()`. Tests write the record. No record: the export is unavailable and says so. No API route function is added | `api-client.ts` gains nothing before V6 (V1 §15.2); `GET /entitlement` needs a bearer token that does not exist yet; `exportClip` requires a token (TS §14.2). Reading a cached token from this store is final behaviour (TS C-1, offline branch) |
| D-24 | **(schedule)** `config/entitlement-public-key.ts` is created in V2 (BP: V6). It exports an array (TS §24.2): the production key, plus the test key only when the build variable `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` is set. `config/env.ts` gains the one field that reads it. CI fails a deployment whose bundle contains the test key | The render worker verifies the token (TS §21.3) and E2E runs against a production build (V1 D-15). A test key in the deployed bundle would let anyone mint tokens |
| D-25 | Token wire format: `base64url(JSON claims) "." base64url(signature)`, both without padding. The Ed25519 signature covers the ASCII bytes of the first segment. Claims JSON uses the Rust field names of TS §10.6; `plan` is `"free"` or `"creator"` (V1 D-2); `sub` is a UUID string | TS §10.8 gives the two-segment form and not the signed bytes. Signing the encoded segment needs no JSON canonicalization. V6 `sign.rs` must produce exactly this |
| D-26 | `offcut-entitlement/src/profile.rs` holds the bitrates of TS §21.2 and the Preview profile: 540x960 (TS §20.3), `watermark: false`, both bitrates 0. `pickVideoConfig` takes the video bitrate from the `ExportProfile` it is given. V1's `CREATOR_VIDEO_BITRATE` in `encoders.ts` stays, as the bitrate of the capability probe only; it must equal the `profile.rs` constant, and TE-4 changes both or neither | `ExportProfile` carries bitrates and `profile.rs` is the only place that builds one (TS §10.8). `limits.rs` is frozen at 21 constants. TS gives the Preview kind no field values |
| D-27 | **REOPENED V1 CONTRACT (`web/eslint.config.js`, frozen layer boundaries).** Six allow edges, all absent from V1's matrix as built: (a) `usecases/import-clip.ts` to `net/asset-fetch`; (b) `workers/**` to `persistence/opfs`; (c) `workers/render.worker.ts` to `config/entitlement-public-key`; (d) `models/**` to `state/model-store` and `config/model-manifest.json`; (e) `models/model-manager.ts` to `persistence/db` (`metaGet`, `metaSet`); (f) type-only imports of `workers/protocol.ts` from `state/**`, `models/**` and `usecases/**` | (a) TS §15.4 has the sample clip fetched by `asset-fetch.ts` and handed to `importClip`; TS §2 gives use-cases only `net/api-client`. (b) TS §23.1 makes `opfs.ts` the only path builder and has workers write those paths. (c) TS §21.3 verifies in the worker and the protocol has no key parameter. (d) TS §12.1 has the model manager write the model store. (e) `ensureReady` records `META_KEYS.persistRequested`, and V1's matrix gives `models` only `opfs.ts` from `persistence`. (f) The clip and export stores hold `FeedLine` and `AppFailure`, and the model manager and the use-cases pass an `AppFailure` on; both types live in `protocol.ts`, which V1 lets only `net/` name (V1 D-8). No runtime code crosses that edge. Cost on the V1 side: six policy entries. Cost on the spec side: TS §2 and §7 rows updated (section 27, item 1) |
| D-28 | `offcut-scene` depends on `offcut-text`, for `format_quantity` only. The dependency check in `check-file-tree.mjs` allows that pair, and the `offcut-text` ban in `deny.toml` gains the wrapper `offcut-scene` | TS §19.5 formats count-up values with `format_quantity`, and TS §5 makes `numbers.rs` the only formatter. The TS §7 diagram omits the edge. It creates no cycle |
| D-29 | `display_list.rs` defines `Rect`, `Rgba`, `PathEl`, `PositionedGlyph`, `Stroke`, `Affine` and a closed `FontId` enum with four members (section 11.4) | TS §19.2 uses these names without defining them. `GlyphRun` has no weight field, so the font instance must be in the id |
| D-30 | `DemuxerHandle` (TS §20.1) is a TypeScript interface in `video-source.ts`; `RenderSession` satisfies it through four extra exported methods (section 13.5). Each binding crate has its own private `JsRandomAccess` wrapping a `FileSystemSyncAccessHandle` | TS §5 gives `RenderSession` ownership of the demuxer and TS §19.2 gives it no way to read a sample. The render worker loads only `offcut_render.wasm` (TS §1). No shared binding crate exists in TS §5 |
| D-31 | All Rust-to-JS conversion uses `serde_wasm_bindgen::Serializer::json_compatible()`. `RawWord` is a Rust struct in `offcut-text/src/normalize.rs` that deserializes from the camelCase shape of TS §16.2; it is not generated | Maps must arrive as plain objects to match the generated types. `RawWord` crosses only the worker-to-WASM call and TS §16.2 defines it in `whisper-runtime.ts`, so it is not a shared type and `offcut-types` stays frozen |
| D-32 | RPC client surface (TS gives none): a call resolves with the method's result or `{ cancelled: true }`; a failure rejects with `WorkerCallError` carrying the `AppFailure`; `previewClock` is a one-way `notify`. A value thrown in a worker may carry its own `stage`. A protocol method a worker does not implement yet answers `E_INTERNAL`, `retryable: false`. Worker scripts are preloaded with `<link rel="modulepreload">`. `pool.ts` re-exports `WorkerCallError` and `Cancelled` from `rpc.ts`, because V1's lint lets a use-case import only `pool.ts` from `workers` | TS C-14 says "rpc rejects all pending calls"; TS §11.3 says a cancelled job resolves. `pool.ts` may not call `fetch` (TS §7), and TS §33 still requires the scripts in the HTTP cache at app start |
| D-33 | **(schedule)** The muxer writes the audio edit list in V2 (BP: V5). Mechanism, with no signature change: when the first audio sample's `pts` is negative, `elst.media_time = -pts` in audio timescale units. `export-loop.ts` supplies that `pts` from the priming count that TE-4 records in `encoders.ts` as `AAC_PRIMING_SAMPLES` (assumption, TE-4; 0 until measured). V5 keeps `ctts` and the overflow test | Verifier check 5 is a V2 exit criterion (BP §4.4) and allows 21.3 ms; unedited priming exceeds it. TE-4 (V2) already tests the A/V offset, and TS §21.3 states the mechanism |
| D-34 | `ProbeInfo.duration` is the presented duration of the video track with edit lists applied, rounded to the nearest millisecond. `pcm48.len() == round(duration_ms x 48)` exactly; the media worker pads or truncates each resampler output by at most one sample to hold it | TS §15.1 aligns audio to "the video duration" without defining which duration the probe reports. Two roundings (decode, then resample) can differ by one sample |
| D-35 | Pipeline stage timings are measured on the main thread around each worker call. `probe_audio` = `importAndProbe` + `extractAudio`. `asr` = `load` + `transcribe`. Export timings come from the worker (`stageTimings`) | TS §32 says timings come from workers, and the protocol returns them only from `exportClip` (section 27, item 4). The session is loaded per clip (INV-12), so load time is part of what the user waits for |
| D-36 | Model parts are written on the main thread with the async OPFS API: one `createWritable({ keepExistingData: true })`, seek, write, close per 8 MiB part. The hash runs on the main thread through `loadCore()` in 4 MiB reads with a macrotask yield between reads | TS §2 lets `models/` use `persistence/opfs` and `wasm/load-core` and names no worker for it. A close per part is what makes a part survive a closed tab (TS §16.5). This is the one main-thread caller of `loadCore()`; V1's comment there ("the main thread never runs WASM") is corrected to say so. Hashing is not media work (INV-17) |
| D-37 | ONNX Runtime files are copied to `web/public/ort/<runtime version>/`, and the runtime is pointed at that directory | `vercel.json` (frozen) marks `/ort/*` immutable for a year. An unversioned path would serve a stale runtime after an upgrade |
| D-38 | **REOPENED V1 CONTRACT (`scripts/check-hosts.mjs`, marked F).** An entry of `NON_NETWORK_LITERALS` (as built: `{ prefix, reason, exact? }`) may carry a `chunk` pattern; such an entry is allowed only in output files whose path matches. Three groups are listed that way, each entry with its reason: the ASR runtime's remote-host literals in the `asr.worker` chunk (found at S6, citing TE-1); literals inside the runtime files copied to `ort/<version>/` (S6); literals inside `offcut_render_bg-*.wasm`, such as vendor and license addresses in the embedded fonts' name tables and addresses in dependency error texts (S10) | V1's script reads every byte of `web/dist`, the `.wasm` files and `ort/` included. The runtime package carries its default hub and CDN host names as strings. Remote loading is off and the CSP blocks those hosts; the script must still fail on such a literal anywhere else. Cost: about ten lines in a script V1 called final; INV-1's static guard gains an exception that TE-1 and the CSP back |
| D-39 | The reference clip (D-64) is not committed. It stays in `testclips/` (git-ignored) on the development machine, at 40 MB or less, and is uploaded at S5 as the sample clip; its path is `SAMPLE_CLIP_PATH` in `net/asset-fetch.ts`. Tests reach it through `helpers/fixtures.ts::referenceClip()`: the `testclips/` file when it exists, else a copy in `fixtures/.cache/` that `globalSetup` downloads from the asset host and checks against the hash segment of its name | The founder's decision of 2026-10-08 is to keep the recording out of the repository. V1 §15.2 makes the sample button active in V2, so the clip is on the asset host anyway and CI can fetch it from there. TS §4 commits the speech fixtures: the committed set starts in V3 |
| D-40 | Until the clip repos and `quota.ts` exist, V2 keeps one clip: `importClip` removes every other directory under `clips/`, and `startExport` removes every file under `exports/` first | No restore exists before V7 and no eviction before V7; without this, repeated imports fill the quota. Both calls are marked stand-ins that V7's `quota.ensureFree` replaces |
| D-41 | E2E serves asset-host URLs from a local cache directory (`fixtures/.cache/`) through Playwright routing, with Range support. `E2E_REAL_ASSETS=1` turns routing off for the first-run timing on R1; for that run the asset bucket's CORS policy gains the origin `http://localhost:4173` (`vite preview`) | Up to 260 MB from a free host on every CI run is slow and spends its allowance. The page still requests the asset-host URL, so host assertions stay valid. As built, the bucket allows the app origin and `http://localhost:5173` only, which would block a real-asset run on the preview port |
| D-42 | `Token` and `TokenKind` (section 8.2), the V2 number subset and the display rules of `format_quantity` (section 8.4), all (assumption, M1.3). One conditional form: if the S6 transcript shows the reference amount without a `$` (for example `12,000 dollars`), the subset gains "a parsed number followed by the word `dollar` or `dollars`", giving `Unit::Usd`. The script is not re-recorded and no threshold changes. **As built (Prompt 43):** the recognizer writes the reference amount with a `$`, but as two words, `$12` and `,000`: it starts a new word at the thousands separator. The subset gained a third form for that, the split form of section 8.4, and the `dollars` form was not needed and is not in | TS §17.1 uses `Token` without defining it and gives three display examples without rules. BP §4.1 D limits V2 to "digits and simple spelled numbers". V2's one visible event depends on how the recognizer writes that amount (section 9.3), so the plan names what happens when it writes it differently |
| D-43 | `ExportId` is a UUIDv7 built in `start-export.ts::newExportId()` from `Date.now()` and `crypto.getRandomValues` | TS C-9 names the function; `crypto.randomUUID()` yields v4 and V1 §6.2 forbids id generation in Rust |
| D-44 | `e2e-media.yml` is a reusable workflow called by `ci.yml` after step 9 and before the deploy steps. It downloads the `web/dist` artifact built in step 8 | TS §33 orders it as step 10, before deploy. Rebuilding Rust and WASM on the Windows runner would spend the minutes TE-10 has to count |
| D-45 | The live-frame counter (INV-11) runs in every build. A non-zero count at the end of an export or after a preview stop fails that call with `E_INTERNAL` | TS §21.4 asserts a dev-build counter from E2E, and E2E runs production builds. This needs no test hook and no protocol change. Cost: a frame leak fails a real export instead of passing unnoticed |
| D-46 | `docs/v2/v2implementation.md`, `docs/v2/v2implementation-notes.md`, `docs/v2/v2buildguide.md`, `docs/v2/coding-promptsv2.md`, `docs/v2/experiments.md` and `docs/v2/v2changelog.md` are added to the TS §5 tree | Same pattern as `docs/v1/`. The notes file, the guide and the prompts file were written after the plan and are listed with it |
| D-47 | The event-id hash input is 9 bytes: kind as `u8` (NumberReveal 0, ListReveal 1, FromTo 2, KeywordPop 3), then `anchors.start` and `anchors.end` as little-endian `u32` | TS §17.1 says FNV-1a 64 over (kind, start, end) without a byte layout. Ids are stored from V3 on, so the layout is fixed now |
| D-48 | A drop or sample click on `/` starts the import, and the page navigates to `/app` when the clip leaves `idle`. `import-clip.ts` exports `importSampleClip()` next to `importClip()`. `LandingPage` gets the path as a prop, `appPath`, from `routes.tsx`, as V1's pages do (a page that imported `ROUTES` would import the file that imports it) | TS §13.1 puts `DropZone` on `/` and J2-J12 on `/app`, and names no moment for the move. TS §5 has no use-case file for the sample fetch |
| D-49 | V2 `detect()` sets `NumberReveal.label` to `None` and applies one inline rule in place of `resolve.rs`: a candidate whose display window overlaps an earlier kept one is dropped | Label derivation needs the stop-list lexicon and resolution needs `resolve.rs`, both V3 (BP §5.1 C). Without any rule two overlays could draw on top of each other |
| D-50 | `Transcript.model_version` is the manifest `modelId` | `AsrWorkerApi.load` carries `modelId` and no version (TS §14.2), and workers do not import the manifest |
| D-51 | `clip-store` holds a reference to `out48` while the clip is `ready`. `startExport` reads `out48` back from `clips/<clipId>/out48.f32` and transfers that buffer | `controlPreview.attach(canvas, out48)` is called from a component (TS §20.1) and UI may not read persistence. A transferred buffer is detached, and TS §31 forbids copying PCM. Cost: 17.3 MB stays referenced on the main thread beside the `AudioBuffer` |
| D-52 | For `export_started.profile` and `export_done.profile`, V2 reads the `plan` claim from the token payload in `start-export.ts` without verifying it. The value is used for analytics only | The protocol returns no profile from `exportClip`, and `entitlement-store` with decoded claims arrives in V6. Size and watermark still come only from the verified token in the worker (INV-9) |
| D-53 | Exported binding surface beyond the names in TS §5: `DemuxerHandle` methods for probing and audio samples, `validate`, `preview_profile()`, and the error-value shapes of section 13.2. `ContainerError::Fragmented` exists and maps to `REJECT_CONTAINER` | TS §5 names `open_demuxer, probe_and_validate, resample` and no way to read an audio sample or to obtain `ProbeInfo` for the `isConfigSupported` call of TS §15.2 |
| D-54 | Third-party crates TS §3 does not list: `base64` and `serde_json` (`offcut-entitlement`), `js-sys` and `wasm-bindgen-futures` (binding crates) | The token format of TS §10.8 needs the first two; `Promise` returns and typed arrays need the others |
| D-55 | New file `crates/offcut-mp4/src/mux_boxes.rs` (box writers), added to the TS §5 tree | `mux.rs` with the sample tables, the public API and every box writer would exceed 400 lines (TS §29) |
| D-56 | **REOPENED V1 CONTRACT (`web/tests-e2e/landing.spec.ts`, marked F).** Two cases are replaced: "Inactive drop zone" becomes "Drop starts an import and opens `/app`"; "`/app`" becomes "shows the drop zone or the unsupported page". `messages.dropZone.notReady` is deleted | V1 §15.2 itself wires the drop zone and replaces the editor placeholder in V2, which makes both V1 cases false. Cost: two cases rewritten in a file V1 called final |
| D-57 | Copy that leaves `COPY_PENDING` in V2: `E_MODEL_DOWNLOAD`, `E_MODEL_HASH`, `E_MODEL_STORAGE`. Blocker copy written in V2: `B_UNSUPPORTED`, `B_PIPELINE_BUSY`, `B_EXPORT_IN_PROGRESS`, `B_PIPELINE_NOT_READY`; the other four blocker codes enter `COPY_PENDING` (due V5, V6) | The model panel shows these three failures. Every other V2 failure shows the stub, so its code keeps its pending entry (V1 D-13: a pending code must not already have copy) |
| D-58 | **REOPENED V1 CONTRACT (`web/eslint.config.js`).** Two use-case-to-use-case imports are allowed: `import-clip.ts` to `run-pipeline.ts`, and `start-export.ts` to `control-preview.ts`. `control-preview.ts` also exports `detach()`, `lockForExport()` and `unlockAfterExport()`; `import-clip.ts` also exports `dismissClip()` | TS C-2 has `importClip` call `runPipeline` and TS C-9 has `startExport` call `controlPreview.pause()`, while TS §7 lets use-cases import only `cancel-job.ts` and `restore-clip.ts` (section 27, item 2). Components may read stores and not write them (TS §2), so "start over" and preview teardown need use-case functions. Cost: two override lines |
| D-59 | **REOPENED V1 CONTRACT (`web/eslint.config.js`).** The `as <Brand>` rule gets a per-file override, for one minting line each, in `persistence/opfs.ts`, `models/download.ts`, `models/model-manager.ts` and `state/model-store.ts` (`Bytes`; in the store, the zero of the initial state), `usecases/import-clip.ts` (`ClipId`), `usecases/start-export.ts` (`ExportId`), `usecases/control-preview.ts` (`TimeMs`) | TS §10.1 allows brand casts only in `gen/` and `workers/`, yet TS §16.2, §20.1 and C-9 have main-thread code produce `Bytes`, `TimeMs` and `ExportId` from raw browser values, and the generated files hold no constructor functions (V1 §9). Cost: seven override entries; the rule still fails anywhere else |
| D-60 | Defensive input bounds, all (assumption, TE-12): box nesting at most 16; at most 4,096 children per container; at most 20,000 samples per track are expanded, and a longer track is left unresolved so that `validate_probe`, not the demuxer, names the reason (section 6.4); edit lists limited to the three shapes of section 6.4; an entitlement token longer than 2,048 characters is rejected before decoding | The demuxer and the token parser read untrusted bytes and TS gives no bounds. 90 s at 60 fps is 5,400 video samples, so the limits cannot reject a valid clip. V3 widens the edit-list shapes against the corpus |
| D-61 | `offcut-render/src/lib.rs` re-exports the scene crate (`pub use offcut_scene as scene;`). `offcut-wasm-render` has no direct dependency on `offcut-scene`: `session.rs` names `build_scene`, `SceneInput` and `Scene` through `offcut_render::scene` | TS §19.2 has the session call `build_scene`, while the TS §7 graph, the table in `check-file-tree.mjs` and the `offcut-scene` ban in `deny.toml` all let only `offcut-render` depend on `offcut-scene`. A re-export changes none of the three |
| D-62 | `scripts/upload-assets.sh` (partial in V1) is edited: the folder argument may also be `models/<modelId>` (one more segment of lower-case letters, digits and hyphens), and the content-type table gains a row for any extension in the model's file list that it lacks. A stored name keeps its hash segment (`models/asr-en-v1/<stem>.<hash>.<ext>`), and that printed path is the manifest's `files[].path`. In OPFS a model file is stored under the last path segment without the hash segment, computed by `download.ts::localName(file)` | As built, the script accepts `media` and `models` only and writes one flat level, while TS §16.1 and TS §23.1 put the files under `models/<modelId>/`. The runtime asks the cache adapter for the plain name (`encoder_model.onnx`), and workers do not import the manifest (D-50), so the name on disk must be the plain one |
| D-63 | Test tooling. (a) `@types/node` is a dev-dependency of `web`, visible only through a new `web/tests-e2e/tsconfig.json` (added to the TS §5 tree) that extends `../tsconfig.json`, adds `node` to `types` and includes `tests-e2e`, `playwright.config.ts`, `vite.config.ts`, `vitest.config.ts` and `../bench`; `web/tsconfig.json` includes `src` alone (corrected in Prompt 32: the two configuration files import Vite, whose types bring in Node's, so in one program with `src` they made every Node global type-check in app code); `pnpm check` and step 6 of `ci.yml` run `tsc --noEmit` for both. (b) The root `package.json` gains the dev-dependency `@playwright/test` at the version `web` pins, and `bench/device-bench.ts` runs as project `bench` of `web/playwright.config.ts` (`testDir: "../bench"`) | (a) The helpers of section 23.4 use Node's crypto, files and child processes. V1 has no Node types and type-checks `tests-e2e` with the app's configuration, where a Node global must never type-check. (b) The pnpm workspace holds `web` alone, so a file under the root `bench/` (TS §5) cannot resolve the package otherwise. `bench/` is outside `web/` and is not linted |
| D-64 | **The V2 reference clip is the founder's webcam recording of Script A**, `testclips/speech_scriptA_landscape_720p.mp4`: 1280x720 landscape, 74.705 s, H.264 Main with AAC at 48 kHz stereo, a variable frame rate (about 30 fps on average; frame gaps of 32 ms and 48 ms), 35.2 MB, with one fully white frame near 4.99 s and one near 64.85 s (frames 150 and 1950). It takes the place of the 60-second 1080x1920 portrait clip of TS §26; a portrait recording of Script A joins the fixtures in V3. Every budget of PS §20.2 and every threshold of E-3 and E-4 is defined for a 60 s clip and is read in V2 **multiplied by 74.7 / 60 = 1.245**: probe and audio 3.7 s; transcription 25 s; scene build 1.25 s; render and encode 112 s; mux 2.5 s; p90 total 224 s; the audio-chain allowance 5 s; the fallback bands 25-50 s (E-3) and 112-187 s (E-4). `docs/v2/experiments.md` records each reading as measured and also normalised to 60 s | The founder's decision of 2026-10-08: the reference should be what a user really drops in. The clip passes all 11 rules of TS §15.2 as it is, and it exercises the landscape crop and a variable frame rate from the first day. Costs: the output is a centre strip 405 pixels wide enlarged to 1080, so it looks soft; a 720p source is cheaper to decode than a 1080x1920 one, so E-4 here understates the decoding cost of a portrait phone clip, which V3 reads again on the portrait fixture; transcription and render time scale with length, which the factor covers |
| D-65 | **R2 is not measured in V2; R1 is the only reference machine.** TE-3, TE-4, E-3 and E-4 are read on R1 alone. The `pnpm e2e:device` run, the verified export and the bench file are R1's, and every box of sections 1A, 25.3 and 26 that named R2 names R1 only. TE-4 passes when a ladder entry is supported on R1. To copy back: BP §1.5, §4.4 and §12.1; TS §28 (M0.2 to M0.4), §30 and §37 (TE-3, TE-4); PS A-7 and §20.1 | The founder's decision of 2026-10-08: no Mac is available (R2 is an Apple M1 with 8 GB). Costs: PS §9.3 lists macOS as supported, PS §20.1 and BP §12.1 ask for a run on R2 at the launch gate, and nothing in V2 will have shown that the pipeline runs on a Mac; the AAC priming count of D-33 is measured for the Windows encoder only |
| D-66 | **The V2 speech model is the small-size English model, and the model files may total 260,000,000 bytes.** It takes the place of the base-size model that TS §16.1 named as the default candidate; the limit was 150,000,000. `modelId` stays `asr-en-v1`. The file set is chosen in Prompt 38 from the exports that fit the limit, and confirmed in Prompt 43, when the runtime loads it. The base-size model is now the first fallback of E-3 and TE-14, the tiny-size model the second. A transcription time above the 25 s target is accepted for the sake of the transcript; the bands of section 24.3 are not changed (25 to 50 s was already "continue and record the miss", and over 50 s still means the smaller model). Copied back in the same commit: PS §10 J4 and §20.2; TS §16.1, the contingency of §16, and §31; BP §4 | The founder's decision of 2026-10-09: a better transcript is worth a larger download and a longer wait. 260,000,000 bytes is the largest size that keeps the first-run budget of PS §20.2: 260 x 8 / 25 = 83 s at 25 Mbps, plus 5 s of initialization, is 88 s of the 90 s allowed. Costs: the first-run download grows from about 53 s to as much as 88 s, with 2 s of margin; the model has 244 million parameters against 74 million and is slower, so E-3 is more likely to miss its 25 s target; the estimate of the ASR phase in TS §31 rises from about 0.96 GB to about 1.46 GB of the 1.5 GB budget (an assumption until TE-14); TS §16.1 had made the small-size model depend on the word error rate of E-10, which has not been measured |

---

## 3. Tools, accounts, secrets, environment variables

Only what is new or changed since V1 §3.

### 3.1 Tools

| Tool | Used for | Where |
|---|---|---|
| `ffmpeg`, `ffprobe` CLI | `verify/verify_mp4.py`; re-encoding the reference clip; the TE-14 stress clip | Dev machine, R1, the Windows CI runner. Never in the product (TS §36) |
| Python 3 + NumPy | `verify/verify_mp4.py` | Same |
| Chrome stable with WebGPU | Everything in this version | R1, the Windows CI runner |

### 3.2 New dependencies

Pin each at its latest stable release on the day it is added and record the version in `docs/v2/v2changelog.md` (TS §3).

| Package | Crate or workspace | Note |
|---|---|---|
| `rubato` | `offcut-dsp` | Resampling (TS §3) |
| `parley` | `offcut-scene` | Shaping and line breaking; pinned with `vello` (TS §3) |
| `wgpu` (WebGPU backend only), `vello` | `offcut-render` | Pinned as a matching pair (TS §3) |
| `ed25519-dalek` (no `rand_core` feature), `base64`, `serde_json` | `offcut-entitlement` | Verification only; no signing code (D-54) |
| `sha2` | `offcut-wasm-core` | `Sha256Stream` |
| `serde-wasm-bindgen`, `js-sys`, `web-sys`, `wasm-bindgen-futures` | The two binding crates only | `deny.toml` wrappers already restrict them (V1 §12.1) |
| `thiserror` | Each new pure crate | Error enums (TS §3) |
| `proptest` | `offcut-mp4`, `offcut-dsp` (dev) | TS §26 |
| `@huggingface/transformers` (brings ONNX Runtime Web) | `web` | Candidate; TE-1 decides (TS §3) |
| `@types/node` (dev) | `web` | Tests and bench only (D-63) |
| `@playwright/test` (dev) | Root package | Same version as `web`; lets `bench/device-bench.ts` resolve it (D-63) |

### 3.3 Accounts: what each must hand over

| Account | You need from it in V2 |
|---|---|
| Asset host (chosen in TE-7) | The model files under `models/asr-en-v1/` and the sample clip under `media/`, uploaded with `scripts/upload-assets.sh`; the public path each upload prints. CORS origins: the app origin and `http://localhost:5173` (there since V1), plus `http://localhost:4173` (D-41) |
| Model source | The files of the small-size English model (D-66) with word-timestamp support (TS §16.1), downloaded once to your machine, hashed, uploaded to the asset host. The browser never contacts the source (TS §24.1) |
| GitHub Actions | A `windows-latest` runner for `e2e-media.yml`; the minutes one run used (TE-10) |

### 3.4 Environment variables

| Variable | Where | Value | New or changed |
|---|---|---|---|
| `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` | CI build step 8; local E2E and bench builds. **Never** the Vercel build | Base64 of the 32-byte test public key (D-24) | New |
| `E2E_REAL_ASSETS` | Local, R1 | `1` to fetch from the real asset host (D-41) | New |
| `BENCH_DEVICE` | R1 | `r1`; names the result file | New |
| `VITE_ASSET_BASE_URL` | Vercel, CI | Unchanged | - |
| Render variables | Render | Unchanged. The server is not touched in V2 | - |

No secret is added. The test signing seed is a constant in `web/tests-e2e/helpers/fake-api.ts`; it signs nothing a deployment accepts.

---

## 4. V2 file tree

Marks: **NEW** = created in V2. **F** = final in V2. **P** = partial; section 25 says what is added later. **CHANGED** = a V1 file V2 edits, with exactly what is added. V1 files not listed are untouched. Every path is in the TS §5 tree except `mux_boxes.rs` (D-55), `web/tests-e2e/tsconfig.json` (D-63) and the six `docs/v2/` files (D-46).

```text
<repo root>/
├── Cargo.toml                          CHANGED  8 members (V1 §15.2) + shared deps of section 3.2
├── deny.toml                           CHANGED  license entries; ban wrappers per section 22.1
├── package.json                        CHANGED  scripts e2e:media, e2e:device, bench:device, verify; dev-dep @playwright/test (D-63)
├── .gitignore                          CHANGED  + web/public/ort/, fixtures/.cache/
├── .github/workflows/
│   ├── ci.yml                          CHANGED  artifact upload, media job, test-key guard (section 22.5)
│   └── e2e-media.yml                   NEW F    reusable; windows-latest (D-44)
├── crates/
│   ├── offcut-mp4/
│   │   ├── Cargo.toml                  NEW F
│   │   ├── src/lib.rs                  NEW F    error enums, re-exports
│   │   ├── src/reader.rs               NEW F
│   │   ├── src/boxes.rs                NEW F
│   │   ├── src/sample_table.rs         NEW F
│   │   ├── src/demux.rs                NEW F
│   │   ├── src/probe.rs                NEW P    hardened against the fixture matrix in V3
│   │   ├── src/validate.rs             NEW F    (D-21) probe_rejections.rs arrives in V3
│   │   ├── src/mux.rs                  NEW P    ctts and the overflow test in V5
│   │   ├── src/mux_boxes.rs            NEW P    (D-55) ctts writer in V5
│   │   └── tests/mux_roundtrip.rs      NEW F
│   ├── offcut-dsp/
│   │   ├── Cargo.toml                  NEW P    deps grow in V3
│   │   ├── src/lib.rs                  NEW P    DspError; module list grows in V3
│   │   └── src/resample.rs             NEW F
│   ├── offcut-text/
│   │   ├── Cargo.toml                  NEW F
│   │   ├── src/lib.rs                  NEW P    + units_lex in V3
│   │   ├── src/tokenize.rs             NEW F
│   │   ├── src/sentences.rs            NEW P    40-word split in V3
│   │   ├── src/numbers.rs              NEW P    V2 subset (section 8.4)
│   │   └── src/normalize.rs            NEW P
│   ├── offcut-detect/
│   │   ├── Cargo.toml                  NEW F
│   │   ├── src/lib.rs                  NEW P    detect() gains stages and redetect_sentence in V3
│   │   ├── src/config.rs               NEW P    lexicons and windows in V3
│   │   ├── src/number.rs               NEW P    labels and the consumed-span rule in V3
│   │   └── src/event_id.rs             NEW F    (D-47)
│   ├── offcut-entitlement/
│   │   ├── Cargo.toml                  NEW F    feature "sign" declared, empty until V6
│   │   ├── src/lib.rs                  NEW P    sign module in V6
│   │   ├── src/claims.rs               NEW F
│   │   ├── src/token.rs                NEW F
│   │   ├── src/verify.rs               NEW F
│   │   └── src/profile.rs              NEW F
│   ├── offcut-scene/
│   │   ├── Cargo.toml                  NEW F
│   │   ├── assets/fonts/Inter-Variable.ttf, JetBrainsMono-Variable.ttf,
│   │   │   NotoEmoji-Variable.ttf, LICENSES.md          NEW F
│   │   ├── src/lib.rs                  NEW P    watermark and summary modules in V4
│   │   ├── src/safe_area.rs            NEW F
│   │   ├── src/styles.rs               NEW P    Clean row now; Bold, Tech rows in V4
│   │   ├── src/fonts.rs                NEW F
│   │   ├── src/layout.rs               NEW F
│   │   ├── src/captions.rs             NEW P    KeywordPop scale in V4
│   │   ├── src/easing.rs               NEW F
│   │   ├── src/anim.rs                 NEW F
│   │   ├── src/display_list.rs         NEW F
│   │   ├── src/framing.rs              NEW P    crop offset honoured in V4
│   │   ├── src/events/mod.rs           NEW P    three more arms in V4
│   │   └── src/events/number_reveal.rs NEW P    label row drawn from V3 data; tuned in V4
│   ├── offcut-render/
│   │   ├── Cargo.toml                  NEW F
│   │   ├── src/lib.rs                  NEW P    re-exports offcut_scene (D-61); render_to_image in V4
│   │   ├── src/gpu.rs                  NEW F
│   │   ├── src/video_pass.rs           NEW F
│   │   ├── src/vello_backend.rs        NEW F
│   │   ├── src/composite.rs            NEW F
│   │   └── src/shaders/video.wgsl      NEW F
│   ├── offcut-wasm-core/
│   │   ├── Cargo.toml                  CHANGED  + offcut-mp4, -dsp, -text, sha2, serde-wasm-bindgen, js-sys, web-sys
│   │   ├── src/lib.rs                  CHANGED  + three module declarations; error-value helper
│   │   ├── src/media_api.rs            NEW F
│   │   ├── src/text_api.rs             NEW F
│   │   └── src/hash_api.rs             NEW F
│   └── offcut-wasm-render/
│       ├── Cargo.toml                  NEW F
│       ├── src/lib.rs                  NEW F    panic hook, init, error-value helper
│       ├── src/session.rs              NEW F
│       ├── src/detect_api.rs           NEW P    redetect_sentence in V3
│       ├── src/profile_api.rs          NEW F
│       └── src/mux_api.rs              NEW F
├── web/
│   ├── package.json                    CHANGED  + @huggingface/transformers; dev-dep @types/node (D-63)
│   ├── vite.config.ts                  CHANGED  ORT copy step (D-37)
│   ├── playwright.config.ts            CHANGED  + projects "media" and "bench", globalSetup
│   ├── tsconfig.json                   CHANGED  "include" is src alone (D-63)
│   ├── eslint.config.js                CHANGED  the edges of D-27 and D-58; overrides of D-59
│   ├── src/config/env.ts               CHANGED  + entitlementTestPublicKey
│   ├── src/config/model-manifest.json  NEW F    values filled at S5
│   ├── src/config/entitlement-public-key.ts   NEW F  (D-24)
│   ├── src/copy/messages.ts            CHANGED  sections of 20.3; dropZone.notReady removed
│   ├── src/net/asset-fetch.ts          CHANGED  + fetchAsset, SAMPLE_CLIP_PATH
│   ├── src/models/model-manager.ts     NEW F
│   ├── src/models/download.ts          NEW F
│   ├── src/models/download.test.ts     NEW F
│   ├── src/persistence/opfs.ts         NEW P    sweepTemp, removeAll in V7
│   ├── src/persistence/entitlement-repo.ts    NEW P  (D-23) clear() in V6
│   ├── src/state/model-store.ts        NEW F
│   ├── src/state/clip-store.ts         NEW P    edit actions in V4
│   ├── src/state/preview-store.ts      NEW P    position while seeking in V4
│   ├── src/state/export-store.ts       NEW P    summary, prompt dismissal in V5
│   ├── src/state/blockers.ts           NEW P    (D-20)
│   ├── src/state/machines/model-machine.ts    NEW F
│   ├── src/state/machines/clip-machine.ts     NEW F  (D-18)
│   ├── src/state/machines/export-machine.ts   NEW F  (D-18)
│   ├── src/usecases/start-app.ts       CHANGED  + step 8 modelManager.inspect()
│   ├── src/usecases/import-clip.ts     NEW P
│   ├── src/usecases/run-pipeline.ts    NEW P
│   ├── src/usecases/control-preview.ts NEW P    seek in V4
│   ├── src/usecases/start-export.ts    NEW P    gate, cache, deliver in V5
│   ├── src/wasm/load-core.ts           CHANGED  CoreApi grows (V1 §11.9); comment per D-36
│   ├── src/wasm/load-render.ts         NEW F
│   ├── src/workers/rpc.ts              NEW P    throttle, cancel timeout, crash handling in V4
│   ├── src/workers/pool.ts             CHANGED  clients, lazy creation, preload table, re-exports (D-32)
│   ├── src/workers/media.worker.ts     NEW F
│   ├── src/workers/media/import.ts     NEW F
│   ├── src/workers/media/audio-decode.ts      NEW F
│   ├── src/workers/asr.worker.ts       NEW F
│   ├── src/workers/asr/whisper-runtime.ts     NEW P  empty-output retry rule in V4
│   ├── src/workers/asr/model-cache-adapter.ts NEW F
│   ├── src/workers/asr/word-timestamps.ts     NEW F
│   ├── src/workers/render.worker.ts    NEW P    redetect in V3; seek in V4
│   ├── src/workers/render/video-source.ts     NEW P  prefetch in V4
│   ├── src/workers/render/preview-loop.ts     NEW P  resize rule in V4
│   ├── src/workers/render/export-loop.ts      NEW P  ladder restart in V5
│   ├── src/workers/render/encoders.ts  CHANGED  + pickVideoConfig, ENCODE_QUEUE_MAX, AAC_PRIMING_SAMPLES
│   ├── src/workers/render/opfs-sink.ts NEW F
│   ├── src/ui/pages/EditorPage.tsx     CHANGED  real shell replaces the placeholder body
│   ├── src/ui/pages/LandingPage.tsx    CHANGED  navigates to /app when an import starts (D-48)
│   ├── src/ui/components/DropZone.tsx  CHANGED  wired to importClip and importSampleClip
│   ├── src/ui/components/ModelDownloadPanel.tsx   NEW P  cancel in V4
│   ├── src/ui/components/ProcessingFeed.tsx       NEW P  cancel in V4
│   ├── src/ui/components/PreviewPlayer.tsx        NEW P  scrubber in V4
│   ├── src/ui/components/ExportButton.tsx         NEW P  counter in V6
│   ├── src/ui/components/ExportProgress.tsx       NEW P  cancel in V4
│   ├── src/ui/styles/pages.module.css, components.module.css   CHANGED  classes for the new components
│   ├── tests-e2e/tsconfig.json         NEW F    Node types for tests, bench and tool configuration only (D-63)
│   ├── tests-e2e/helpers/fake-api.ts   CHANGED  + mintEntitlementToken, seedEntitlement
│   ├── tests-e2e/helpers/fixtures.ts   CHANGED  + asset routing, clip drop, model cache
│   ├── tests-e2e/landing.spec.ts       CHANGED  two cases replaced (D-56)
│   ├── tests-e2e/model-download.spec.ts       NEW F
│   ├── tests-e2e/pipeline-preview.spec.ts     NEW P  remaining cases in V4
│   └── tests-e2e/export-creator.spec.ts       NEW P  checks 7-11, cache case in V5
├── verify/
│   ├── verify_mp4.py                   NEW P    checks 1-6; 7-11 in V5
│   ├── requirements.txt                NEW F
│   └── README.md                       NEW P
├── testclips/speech_scriptA_landscape_720p.mp4   not in git: the reference clip (D-39, D-64)
├── bench/
│   ├── device-bench.ts                 NEW P    --compare in V9
│   └── results/<device>-<date>.json    data
├── scripts/
│   ├── build-wasm.sh                   CHANGED  + BUNDLES entry offcut-wasm-render:render
│   ├── check-copy-codes.mjs            CHANGED  reads BLOCKER_CODES; COPY_PENDING per D-57
│   ├── check-hosts.mjs                 CHANGED  chunk-scoped entries (D-38)
│   ├── check-file-tree.mjs             CHANGED  allowed pair scene -> text (D-28); PURE_CRATES grows
│   ├── upload-assets.sh                CHANGED  models/<modelId> folder, content types (D-62)
│   └── check-external-facts.mjs        CHANGED  TE-10 line with its date
└── docs/
    ├── technicalspec.md                CHANGED  §5 tree, §2 and §7 rows per section 27
    └── v2/
        ├── v2implementation.md         this file
        ├── experiments.md              NEW      outcomes of section 24
        └── v2changelog.md              NEW      one entry per change made while building V2
```

Untouched in V2: every file under `server/`, `crates/offcut-types/`, `crates/offcut-api-types/`; `web/vercel.json`, `workers/protocol.ts`, `persistence/schema.ts`, `persistence/db.ts`, `net/http.ts`, `net/api-client.ts`, `analytics/client.ts`, `platform/*`, `state/capability-store.ts`, `state/machines/transition.ts`, `render.yaml`.

---

## 5. Build order

Sixteen steps over ten working days (Mon 19 Oct to Fri 30 Oct 2026), gate on Sun 1 Nov. Those are the dates of BP §1. The build may start earlier (V1's code was finished on 8 Oct); the day numbers then count from the real start, and the gate is read no later than Sun 1 Nov. Do not start a step while the previous check fails. Order: contracts, pure crates, bindings, workers, use-cases, UI, tests and CI, deploy and measure. The two gate questions are reached as early as the dependencies allow: E-3 on day 4, E-4 on day 7.

| # | Step | Produces | Check |
|---|---|---|---|
| S1 (day 1) | Contracts and setup | The reference clip in `testclips/`, with its two white frames (D-39, D-64; prepared on 2026-10-08); 8 new workspace members with empty `lib.rs`; the lint changes of D-27, D-58, D-59; the test tooling of D-63; `env.ts` field; `entitlement-public-key.ts`; the `check-file-tree` pair and `PURE_CRATES`, and the `deny.toml` wrapper (D-28); TS §5 tree updated; `docs/v2/` | `cargo metadata` lists 12 members; `pnpm check` green; `ffprobe` shows the clip as H.264/AAC, 1280x720, 74.7 s, at most 40 MB |
| S2 (day 1) | `offcut-mp4` read side | `reader`, `boxes`, `sample_table`, `demux`, `probe`, `validate` (section 6) | `cargo test -p offcut-mp4 --lib`; clippy clean |
| S3 (day 2) | Resampler; core bindings | `resample.rs`; `media_api.rs`, `hash_api.rs`; `CoreApi` in `load-core.ts` | `cargo test -p offcut-dsp`; `pnpm build:wasm`; `tsc --noEmit` |
| S4 (day 2) | Worker plumbing and ingest | `opfs.ts`, `rpc.ts`, `pool.ts`, `media.worker.ts`, `media/*` | In `pnpm dev`, a temporary console call imports the reference clip and prints a `ClipInfo` whose duration is within 1 ms of `ffprobe`'s video duration, and `pcm48.length === Math.round(durationMs * 48)`. Remove the call |
| S5 (day 3) | Model delivery | The `upload-assets.sh` edit (D-62); model files on the asset host; `model-manifest.json`; `fetchAsset`; `download.ts`; `model-manager.ts`; model store and machine; `ModelDownloadPanel` | `download.test.ts` green; in the dev page the model reaches `ready`, and after a reload `inspect()` reports `ready` with no model request |
| S6 (days 3-4) | ASR spike (M0.2) | `asr.worker.ts`, `asr/*`; ORT copy step; `text_api.rs` with `offcut-text` (section 8) | TE-1 and TE-2 pass or their fallback is taken (section 24); a `Transcript` of the reference clip with word timestamps is printed, and it is read for how the dollar amount is written (precondition 7, D-42); `pnpm build` passes `check-hosts.mjs` with the runtime in `web/dist` (D-38); first E-3 reading on R1 written to `docs/v2/experiments.md` |
| S7 (day 3, parallel) | Muxer | `mux.rs`, `mux_boxes.rs`, `mux_roundtrip.rs` | `cargo test -p offcut-mp4` |
| S8 (days 4-5) | `offcut-scene` | Section 11 | `cargo test -p offcut-scene` |
| S9 (day 5) | Detection and entitlement | `offcut-detect`; `offcut-entitlement`; token helpers in `fake-api.ts` (sections 9, 10, 23.4) | `cargo test -p offcut-detect -p offcut-entitlement`, including the cross-language token vector (section 10.5) |
| S10 (days 5-6) | `offcut-render`, `offcut-wasm-render` | Sections 12 and 13, with all four binding files (`session.rs`, `detect_api.rs`, `profile_api.rs`, `mux_api.rs`); `load-render.ts`; `build-wasm.sh` entry | `pnpm build:wasm` writes both bundles; `cargo clippy --workspace --all-targets -- -D warnings`; `pnpm build` passes `check-hosts.mjs` with the render bundle in `web/dist` (D-38) |
| S11 (days 6-7) | Render and encode spike (M0.3) | `render.worker.ts`, `video-source.ts`, `encoders.ts` (complete), `export-loop.ts`, `opfs-sink.ts`; `verify/*` | A spike export of the reference clip passes `verify_mp4.py` checks 1-6; TE-3 and TE-4 recorded; first E-4 reading on R1 |
| S12 (day 8) | State and use-cases | Stores, machines, `blockers.ts`, `entitlement-repo.ts`, `import-clip`, `run-pipeline`, `control-preview`, `start-export`, `preview-loop.ts`, `start-app` step 8 | `tsc --noEmit`; ESLint clean; `vitest run` |
| S13 (days 8-9) | UI and copy | Components, `EditorPage`, `DropZone`, `LandingPage`, `messages.ts`, styles, the eight analytics events | `pnpm dev`: drop the clip, watch the feed, play the preview; with a seeded token, export and play the MP4 |
| S14 (day 9) | E2E, CI, bench | Three spec files; `landing.spec.ts` cases; helpers; `e2e-media.yml`; `ci.yml` changes; `device-bench.ts` | `pnpm check && pnpm test && pnpm build && pnpm e2e && pnpm e2e:media` locally; the pull request is green including the media job (TE-10) |
| S15 (day 10) | Deploy and measure | Merge to `main`; bench on R1; TE-14 | Section 26, deployment and experiment groups |
| S16 (Sun 1 Nov) | M0 gate | The decision of section 24.3, written down | Tagged `v2` |

**What can run in parallel.** S7 (muxer) needs only S2 and fills any wait in S5-S6. S8 (`offcut-scene`) is pure Rust and independent of S4-S6 once `format_quantity` exists. S9 is pure Rust too and needs only `offcut-text`. It comes before S10 because the render bundle and the spike need an `ExportProfile` and the detection and profile bindings. Uploading the model (S5) and recording the clip (S1) are waits; start both on the morning of day 1. If client work is blocked, start the V6 server work, which depends only on V1 (BP §0).

**If S6 or S11 shows a miss, stop and read section 24.3 before continuing.** The steps after them are only worth building if the gate can pass.

---

## 6. `offcut-mp4`: demux, probe, validate, mux

**Purpose** (TS §6). Parse MP4/MOV, expose samples and codec configs, probe, validate the PS §9.4 limits, write a faststart MP4.

**Crate rules** (TS §2, §7). Depends on `offcut-types`, `thiserror`; dev: `proptest`. No `web-sys`, `js-sys`, `wasm-bindgen`, `wgpu`, clock, randomness or global state. Non-test code has no `unwrap`, `expect`, `panic!` or slice indexing: every read goes through `get(..)` or a cursor that returns `ContainerError::Truncated`.

**Never.** Decodes or encodes a sample. Reads a whole file into memory. Allocates from an untrusted count without first bounding it by the file size. Holds user-facing text. Logs.

### 6.1 `lib.rs`: errors

```rust
pub enum IoError { Read, Write, OutOfBounds }
pub enum ContainerError {
    NotIsoBmff,                   // no ftyp/moov/mdat/free/wide/skip box at offset 0
    Fragmented,                   // a moof box, or mvex inside moov (TS §15.4)
    Truncated,                    // a box or table runs past the end of the file
    Malformed(&'static str),      // the name of the box that failed, never file content
    Unsupported(&'static str),    // a structure V2 does not read (section 6.4)
    Io(IoError),
}
pub enum MuxError { Io(IoError), BadConfig(&'static str), OutOfOrder, MoovOverflow }
```

| Error | Becomes | Where mapped |
|---|---|---|
| `NotIsoBmff`, `Fragmented` | `REJECT_CONTAINER` (TS §15.2, §15.4, D-53) | `media_api.rs` |
| `Truncated`, `Malformed`, `Unsupported` | `REJECT_CORRUPT` (TS §15.2: "other Err") | `media_api.rs` |
| `Io` while demuxing | `E_STORAGE_IO` | `media_api.rs`, `session.rs` |
| Any `MuxError` | `E_MUX` (TS §11.2); `Io` during a write to a full disk is reported by the sink as `E_STORAGE_QUOTA` before the muxer sees it (section 16.9) | `mux_api.rs` |

### 6.2 `reader.rs`

Copy the `RandomAccess` trait of TS §15.1 verbatim. As built (Prompt 34) it has one provided method more, `is_empty()`, which clippy requires beside `len()`; an implementation does not write it. `reader.rs` also holds `Cursor`, which reads big-endian fields from bytes already in memory and returns `Truncated` past their end. Add `pub struct MemReader(pub Vec<u8>)` implementing `RandomAccess`; its counterpart `pub struct MemSink(pub Vec<u8>)` implementing `MuxSink` lives in `mux.rs` and grows the vector on a write past the end. Both are used by tests and by nothing else. A read past `len()` returns `IoError::OutOfBounds`.

### 6.3 `boxes.rs`

```rust
pub struct BoxHeader { pub kind: [u8; 4], pub start: Bytes, pub body: Bytes, pub end: Bytes }
pub fn read_header<R: RandomAccess>(r: &mut R, at: Bytes, limit: Bytes) -> Result<BoxHeader, ContainerError>;
pub fn children<R: RandomAccess>(r: &mut R, parent: &BoxHeader) -> Result<Vec<BoxHeader>, ContainerError>;
```

| Situation | Result |
|---|---|
| 32-bit size 1 | Read the 64-bit `largesize` |
| 32-bit size 0 | The box runs to `limit` |
| `end > limit`, or size smaller than the header | `Truncated` |
| Nesting deeper than 16 (D-60) | `Malformed("depth")` |
| More than 4,096 children in one container (D-60) | `Malformed("children")` |
| Unknown box type | Skipped by its size; never an error |

Typed readers, each taking a `BoxHeader` and returning a plain struct: `ftyp` (major brand, compatible brands), `mvhd` (timescale, duration), `tkhd` (flags, matrix, width, height), `mdhd` (timescale, duration), `hdlr` (handler type), `stsd` (first sample entry: four-character code, coded width and height or channel count and sample rate, and the raw `avcC` payload or the `AudioSpecificConfig` extracted from `esds`), `stts`, `ctts`, `stsc`, `stsz`, `stco`, `co64`, `stss`, `elst`. `stsz` with a constant sample size is supported. Table entry counts are bounded by `body length / entry size` before any allocation. As built (Prompt 34): the readers of the eight sample-table boxes (`stts`, `ctts`, `stsc`, `stsz`, `stco`, `co64`, `stss`, `elst`) are in `sample_table.rs`, beside `resolve`, because `boxes.rs` with all fourteen readers exceeds 400 lines (TS §29); `read_stsd` also takes the track's handler type, which says how to read the entry; `read_stsz` also takes the largest size table it may load, so that a track left unresolved (section 6.4, step 6) costs no memory; an `avcC` or `esds` box over 1 MiB is `Malformed`; and the depth bound of D-60 is in `descend`, the one function that follows a path of nested boxes.

### 6.4 `sample_table.rs`

```rust
pub struct Sample { pub offset: Bytes, pub size: u32, pub dts: i64, pub pts: i64, pub duration: u32, pub keyframe: bool }  // media timescale
pub struct TrackTable { pub timescale: u32, pub sample_count: u32, pub resolved: bool, pub samples: Vec<Sample>,
                        pub edit_offset: i64, pub presented: i64 }
pub fn resolve(t: &RawTables) -> Result<TrackTable, ContainerError>;
```

`resolve`, in order:

1. Expand `stts` into per-sample durations and decode times. Sum of counts must equal the `stsz` count, else `Malformed("stts")`.
2. Apply `ctts` offsets (version 0 unsigned, version 1 signed) to get `pts`. Absent: `pts = dts`.
3. Walk `stsc` with `stco`/`co64` and `stsz` to give each sample its file offset. An offset plus size beyond the file length gives `Truncated`.
4. Mark keyframes from `stss`. Absent `stss`: every sample is a keyframe (audio, and intra-only video).
5. Edit list. Supported shapes: none; one media edit; one leading empty edit followed by one media edit. `edit_offset = media_time` of the media edit minus the empty edit's length converted to the media timescale. `presented` = the media edit's duration in the media timescale, or the sum of sample durations when there is no edit list. Any other shape gives `Unsupported("elst")` (V3 widens this against the corpus, TE-12).
6. Bounds (D-60). `sample_count` is read from `stsz` before step 1. A track with more than 20,000 samples (90 s at 60 fps is 5,400 video samples; 90 s of AAC is 4,219 frames) is **left unresolved**: steps 1-4 are skipped, `samples` stays empty and `resolved` is `false`. `timescale`, `edit_offset` and `presented` are still filled, because step 5 needs only `elst`, `mdhd` and the `stts` runs (count x delta), not per-sample data. This is not an error. Such a file is too long, or carries a track V2 does not accept (a PCM track has one sample per audio frame: 4,320,000 in 90 s), and `validate_probe` must be the one that says which (TS §15.2). A clip that passes validation does not reach the bound in practice; if one does, the first read of that track fails (section 6.5).

### 6.5 `demux.rs`

Copy `SampleMeta` and the `Demuxer<R>` signatures of TS §15.1 verbatim. Add crate-private accessors for `probe.rs`.

`Demuxer::open`, in order:

1. Read top-level headers until `moov` is found. A `moof` seen first, or `mvex` inside `moov`, gives `Fragmented`. No recognizable box at offset 0 gives `NotIsoBmff`. `moov` after `mdat` is supported (TS §15.4).
2. For each `trak`: read `tkhd`, `mdhd`, `hdlr`, `stsd` and the sample tables. Video track = the first enabled `vide` track (TS §15.4); audio track = the first `soun` track. Count all `vide` and `soun` tracks for the probe.
3. `resolve` the chosen tracks. A file with no video track still opens (the probe reports it), and so does a file with an unresolved track (section 6.4, step 6).
4. Time base (TS §15.1). `t0` = the smallest video `pts` after subtracting `edit_offset`. Every reported time is `(pts - edit_offset) / timescale - t0`, converted to `Micros` with floor division. Audio uses its own `edit_offset` and timescale and the same `t0`, so an audio sample may report a negative `pts`. With an unresolved video track, `t0` is 0.

| Method | Contract |
|---|---|
| `read_video_sample(i, out)`, `read_audio_sample(i, out)` | Clears `out`, reads exactly the sample bytes with one `read_at`, returns its `SampleMeta`. `i` out of range: `Malformed("index")`. A track left unresolved: `Malformed("samples")`. Samples are in decode order |
| `keyframe_at_or_before(t)` | The index of the last keyframe whose `pts <= t`; 0 when `t` precedes every keyframe |
| `video_decoder_description()`, `audio_decoder_description()` | The raw `avcC` payload; the `AudioSpecificConfig`. `None` when the track or the box is absent |

### 6.6 `probe.rs`

`probe(d, file_size) -> ProbeInfo` never fails; a missing piece is reported as a count of 0 or `None`. It reads the box-level data (`sample_count`, the `stts` runs, `presented`), never the expanded sample list, so every field is also right for an unresolved track (section 6.4, step 6). Field rules:

| Field | Rule |
|---|---|
| `container` | `Mov` when the major brand is `qt  `, else `Mp4`. `Other` is not produced in V2: a non-ISO file fails at `open` |
| `duration` | D-34: `presented` of the video track, in milliseconds, rounded to nearest. No video track: the audio track's; neither: 0 |
| `video_tracks`, `audio_tracks` | Counts from step 2 of `open` |
| `video.codec` | `avc1`/`avc3`: `H264`. `hvc1`/`hev1`: `Hevc`. `av01`: `Av1`. `vp09`: `Vp9`. `apch`, `apcn`, `apcs`, `apco`, `ap4h`: `ProRes`. Else `Other` |
| `video.codec_string` | H.264: `"avc1."` + bytes 1-3 of `avcC` as six lower-case hex digits. Else the four-character code |
| `video.coded_width`, `coded_height` | From the sample entry |
| `video.rotation` | From the `tkhd` matrix: identity `R0`; (0,1,-1,0) `R90`; (-1,0,0,-1) `R180`; (0,-1,1,0) `R270`; anything else `R0` |
| `video.frame_count` | Sample count |
| `video.avg_fps` | `frame_count x 1,000,000 / duration_ms`, as `FpsMilli`, integer division; 0 when the duration is 0 |
| `video.max_fps` | From the smallest non-zero sample duration |
| `video.is_vfr` | True when two sample durations differ by more than 1 ms (TS §15.4) |
| `audio.codec` | `mp4a` with object type 0x40: `Aac`; `mp4a` with 0x69 or 0x6B, or `.mp3`: `Mp3`; `Opus`: `Opus`; `lpcm`, `sowt`, `twos`: `Pcm`; else `Other` |
| `audio.codec_string` | AAC: `"mp4a.40."` + the audio object type from the `AudioSpecificConfig`. Else the four-character code |
| `audio.sample_rate`, `channels` | From the `AudioSpecificConfig` when present, else the sample entry |

### 6.7 `validate.rs` [ONLY PS §9.4 limits]

`validate_probe(p, decode_supported)`: copy the 11 rules of TS §15.2 verbatim, in that order, first failure wins. The file-size rule is not here: `importAndProbe` checks it before any copy (TS §15.2). On success build `ClipInfo`: display size is the coded size, swapped for `R90` and `R270`; `orientation` is `Portrait` when display height exceeds display width, else `Landscape` (square is landscape, TS §10.3); the other fields are copied from the probe. This is the only constructor of `ClipInfo` (TS §10.8). It reads limits from `offcut_types::limits` only and never returns `FileSize`, `Corrupt` or `NoSpeech`.

### 6.8 `mux.rs` and `mux_boxes.rs`

Copy `MuxSink`, `VideoTrackSpec`, `AudioTrackSpec` and the `Mp4Muxer<S>` signatures of TS §21.1 verbatim. Configuration is TS §21.2: brands `isom`/`mp42`; video timescale 30,000 with sample delta 1,000; audio timescale 48,000; chunks of 0.5 s; `moov` before `mdat`.

| Call | Behaviour |
|---|---|
| `new` | Rejects empty `avcc` or `asc`, zero size, a sample rate other than 48,000 or a channel count other than 2 with `BadConfig`. Writes `ftyp`, then a `free` box of `MOOV_RESERVE = 256 KiB` (TS §21.3), then an `mdat` header with a 64-bit size field |
| `add_video_sample(data, frame, is_keyframe)` | `frame` must equal the number of video samples already added, else `OutOfOrder` (reordered output and `ctts` are V5). The first sample must be a keyframe, else `BadConfig`. Appends the bytes to `mdat`; records size and sync flag. A new chunk starts every 15 samples |
| `add_audio_sample(data, pts, duration)` | `pts` must not go backwards, else `OutOfOrder`. Records size and the duration converted to 48 kHz ticks, rounded to nearest. The first sample's `pts`, when negative, is kept as the priming length in ticks (D-33). A new chunk starts every 24 samples |
| `finalize` | Patches the `mdat` size; builds `moov`; writes it at the start of the reserved region and a `free` box over the remainder; returns the total file size. A `moov` larger than the reserve gives `MoovOverflow` |

`moov` contents (`mux_boxes.rs` holds one writer per box and nothing else): `mvhd` (timescale 1,000; duration = the longer track; creation and modification time 0); a video `trak` with `tkhd` (identity matrix, width and height), `mdhd`, `hdlr` `vide`, `vmhd`, `dinf`, and `stbl` with `stsd` (`avc1` + `avcC`), `stts` (one run, delta 1,000), `stss`, `stsc`, `stsz`, `co64`; an audio `trak` with `smhd` and `stbl` with `stsd` (`mp4a` + `esds` wrapping the `AudioSpecificConfig`), `stts`, `stsc`, `stsz`, `co64`, and an `edts`/`elst` with `media_time` = the priming length when it is non-zero and `segment_duration` = the track duration minus priming. No `udta`, no title, no location, no rotation (verifier checks 2 and 11).

As built (Prompt 41): a chunk also ends when the other track writes, because the samples of a chunk must lie one after the other in `mdat`; `stsc` holds one record wherever the number of samples in a chunk changes. What is left of the reserve after `moov` is a `free` box, so the order of the top-level boxes is `ftyp`, `moov`, `free`, `mdat`. `MuxSink` is also implemented for `&mut S`: a caller keeps its sink when `finalize` consumes the muxer. The writers of `trak`, `mdia` and `stbl` are in `mux_boxes.rs` with those of the other boxes.

**Never.** Buffers sample payloads in memory: each `add_*` writes through the sink at once (TS §31). Reads a clock: timestamps in the file are 0.

**Budget.** Mux + finalize at most 2 s for a 60 s clip on R1 (PS §20.2), which is 2.5 s for the reference clip (D-64); read from `stageTimings`.

### 6.9 Inline unit tests

| Case | Expect |
|---|---|
| Header with 32-bit size, 64-bit size, size 0 | Correct `body` and `end` |
| Box that ends past the limit | `Truncated` |
| `stts` whose counts do not sum to the `stsz` count | `Malformed("stts")` |
| `ctts` version 1 with a negative offset | `pts < dts` for that sample |
| No `stss` | Every sample is a keyframe |
| One media edit with `media_time = 2 frames` | The first presented frame reports `pts` 0 |
| Two media edits | `Unsupported("elst")` |
| `tkhd` matrices for 0, 90, 180, 270 degrees | The four `Rotation` values |
| `avcC` bytes `64 00 28` | `codec_string == "avc1.640028"` |
| `validate_probe` with each of the 11 rules violated alone, built from a hand-made `ProbeInfo` | The rule's `RejectReason` |
| Two rules violated | The earlier one in TS §15.2 order |
| Exactly 90,000 ms; 60,500 `FpsMilli` | Accepted (TS §10.2) |
| A 1920x1080 probe with `R90` | `ClipInfo` 1080x1920, `Portrait` |
| A video track of 30,000 samples in one `stts` run at 30 fps (1,000 s) | Opens, unresolved; the probe reports 1,000,000 ms and 30,000 `FpsMilli`; `validate_probe` gives `Duration`, not a corrupt file |
| A valid video track beside an `lpcm` audio track of 4,320,000 samples | Opens; `validate_probe` gives `AudioCodec` |
| `read_video_sample(0, ..)` on an unresolved track | `Malformed("samples")` |

---

## 7. `offcut-dsp`: resampling only

**Crate rules** (TS §2, §7). Depends on `offcut-types`, `rubato`, `thiserror`; dev: `proptest`. Pure. V3 adds every other module of TS §5.

`lib.rs` declares `pub mod resample;` and `pub enum DspError { Empty, NonFinite }` (TS §34.3), unused until V3.

```rust
// resample.rs
pub fn resample_mono(input: &[f32], from: Hz, to: Hz) -> Vec<f32>;       // TS §18.1
```

| Situation | Result |
|---|---|
| Any input | `output.len() == round(input.len() x to / from)` exactly (TS §18.1) |
| `from == to` | A copy |
| Empty input, or either rate 0 | An empty vector |
| Resampler latency | Removed: the filter's group delay is trimmed from the head and the tail is zero-padded, so sample `i` of the output is time `i / to` of the input |
| Same input twice | Identical output (fixed parameters, no randomness) |

**Never.** Looks at the audio content to decide anything; changes the duration; takes word timings (INV-10).

**Budget.** Part of the 3 s probe + audio extraction budget (PS §20.2), which is 3.7 s for the reference clip (D-64); read from `stage_timing stage=probe_audio`.

| Inline test | Expect |
|---|---|
| proptest: random lengths 0 to 200,000, rate pairs (44,100; 48,000), (48,000; 16,000), (48,000; 48,000) | The length rule holds |
| A 1 kHz sine at 48 kHz resampled to 16 kHz | Peak within 1% of the input; first zero crossing within one output sample of its expected position |
| An impulse at sample 4,800 of a 48 kHz buffer, resampled to 16 kHz | The output peak is at sample 1,600 plus or minus 1 |
| Same input twice | Byte-identical output |

---

## 8. `offcut-text`: tokens, sentences, numbers (V2 subset)

**Crate rules.** Depends on `offcut-types`, `serde`. Pure. **Never** decides what becomes a visual event (TS §6); never mutates a `Transcript` after building it (TS §10.8).

### 8.1 `normalize.rs`

```rust
#[derive(Deserialize)] #[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RawWord { pub text: String, pub start_ms: u32, pub end_ms: u32, pub confidence: f32 }   // D-31
pub fn normalize_transcript(raw: Vec<RawWord>, model_version: &str) -> Transcript;                  // TS §17.1
```

1. Trim each text; drop words whose text is empty.
2. Build `Word`s. A confidence outside `0.0..=1.0` or not finite is clamped into range (NaN becomes 1.0). `end_ms < start_ms` is raised to `start_ms`.
3. `sentences = segment_sentences(&words)`.
4. `numbers`: tokenize; at each token index call `parse_quantity`; on `Some((n, q))` push `NormalizedSpan { words: range of the n tokens' words, quantity: q }` and skip `n` tokens.
5. Return `Transcript { words, sentences, numbers, model_version }`. This is the only constructor of a `Transcript`.

As built (Prompt 42): `deny_unknown_fields` refuses an extra field when a `RawWord` is read from JSON, and has no effect in the browser, where `serde-wasm-bindgen` reads the four fields it knows by name and never sees another. There the TypeScript type `RawWord` of `load-core.ts` is what keeps the shape. A time that is not a whole number of milliseconds, or a missing field, is refused on both paths.

### 8.2 `tokenize.rs` (D-42)

```rust
pub enum TokenKind { Word, Digits, Punct }
pub struct Token { pub kind: TokenKind, pub text: String, pub word: WordIdx }
pub fn tokenize(words: &[Word], edits: &BTreeMap<WordIdx, String>) -> Vec<Token>;
```

Effective text is `edits[idx]` when present, else `Word.text` (TS §17.2); an empty edit yields no token. Text is lower-cased. Leading and trailing punctuation is split into `Punct` tokens, except that a leading `$` and a trailing `%` stay attached to a `Digits` token. A token is `Digits` when, after removing one leading `$`, one trailing `%` and thousands separators, it parses as a decimal number. Every token keeps the index of the word it came from.

### 8.3 `sentences.rs`

`segment_sentences(words)`: a boundary after a word whose text ends in `.`, `?` or `!`, or before a word that starts at least `SENTENCE_GAP = 700 ms` after the previous word ended (TS §17.2). The `SENTENCE_MAX_WORDS = 40` split arrives in V3; the constant is declared now. Sentences cover every word exactly once, in order; `start_ms` and `end_ms` are those of the first and last word.

### 8.4 `numbers.rs` [ONLY parser and formatter]

```rust
pub fn parse_quantity(tokens: &[Token]) -> Option<(usize, Quantity)>;    // tokens consumed, value (TS §17.1)
pub fn format_quantity(value: f64, unit: &Unit) -> String;               // the only formatter
```

**Parsed in V2** (BP §4.1 D):

| Form | Example | Result |
|---|---|---|
| Digits with optional thousands separators and one decimal point | `10,000`, `3.5` | value, `Unit::None` |
| `$` prefix | `$10,000` | value, `Unit::Usd` |
| `%` suffix | `40%` | value, `Unit::Percent` |
| Spelled cardinal from zero to 999,999 built from units, teens, tens, "hundred", "thousand", with an optional "and" | "ten thousand", "two hundred and fifty" | value, `Unit::None`; consumes every token of the phrase |

**Not parsed until V3** (TS §17.2): millions and above, "point" decimals, the suffixes k, m, b and "grand", currency words (but see the conditional form below) and other currency symbols, "percent" as a word, multipliers, units from `units_lex.rs`. A token sequence V2 cannot parse returns `None`; it never returns a guess.

As built (Prompt 42): a tens word and a units word may be one hyphenated token (`forty-two`); eleven to nineteen may stand before "hundred" ("fifteen hundred" is 1,500, and takes no "thousand" after it); an "and" that no number follows is not consumed; a cardinal followed by the word "point" returns `None`, so that "three point five" is not read as 3. A `Digits` token with both a `$` and a `%` returns `None`.

**Conditional form (D-42).** Added only if the S6 transcript of the reference clip shows its dollar amount without a `$`: a number of the table above followed by the token `dollar` or `dollars` gives `Unit::Usd` and consumes that token too (`12,000 dollars`, "twelve thousand dollars"). Record in `v2changelog.md` whether it was added. V3's currency words then start from this row. Not added: the S6 transcript has the `$`.

**Split form (D-42, added in Prompt 43).** A word that begins with a comma and exactly three digits, straight after a whole number in digits, is the next group of that number: the words `$12` and `,000` are 12,000 dollars, in three tokens, and the span covers both words. It repeats (`3`, `,000`, `,000`). It does not apply after a number with a decimal point or a `%`, to a group that is not three digits, or to a comma that ends the word before (`12,` then `000` stays 12): only a recognizer's word boundary puts a comma at the start of a word.

**Display rules of `format_quantity`** (assumption, M1.3; `numbers_table.rs` in V3 is the authority):

| Value and unit | Display |
|---|---|
| Integer, `None` | Grouped by thousands: `10,000` |
| Non-integer, `None` | Up to 2 decimals, trailing zeros removed |
| `Usd`, whole thousands from 1,000 to 999,000 | `$10k` |
| `Usd`, otherwise | `$` + the `None` form |
| `Percent` | The `None` form + `%` |
| Any other unit | The `None` form (V3 adds the suffixes) |

`Quantity.display` is always `format_quantity(value, &unit)`.

### 8.5 Inline unit tests

| Case | Expect |
|---|---|
| `"$10,000."` | Tokens: `Digits("$10,000")`, `Punct(".")`, same `WordIdx` |
| `"40%"` | One `Digits` token; parses to 40, `Percent`, display `40%` |
| "ten thousand" | Consumes 2 tokens; 10,000; display `10,000` |
| "two hundred and fifty" | Consumes 4 tokens; 250 |
| "three" | 3, `None` |
| "ten million", "three point five", "2k" | "ten" alone parses as 10 and consumes 1; `None`; `None` |
| `$10,000`; `$1,500`; `$950` | `$10k`; `$1,500`; `$950` |
| An edit replacing word 3 with `""` | No token carries `WordIdx(3)` |
| Words "Hi." (0-400 ms), "There" (500-900), "now" (1700-2000) | Three sentences |
| `normalize_transcript` with an empty-text word and a confidence of 1.4 | The word is dropped; the confidence is 1.0 |
| Same raw words twice | Equal transcripts |
| Only with the conditional form of D-42: `12,000 dollars`; "twelve thousand dollars" | Consumes 2 tokens; 3 tokens. Both give 12,000, `Usd`, display `$12k` |
| The split form of D-42 (added in Prompt 43): the words `$12` and `,000` | Consumes 3 tokens; 12,000, `Usd`, display `$12k`; one span over both words. `12,` then `000` is 12 |

---

## 9. `offcut-detect`: NumberReveal only

**Crate rules.** Depends on `offcut-types`, `offcut-text`. Pure. **Never** constructs a `DetectedEvent` below its kind's threshold (INV-6); never lays out or renders (TS §6).

### 9.1 `config.rs` [ONLY thresholds and lexicons]

```rust
pub struct DetectorConfig {
    pub threshold_from_to: f32, pub threshold_list: f32, pub threshold_number: f32, pub threshold_keyword: f32,
    pub number_base: f32, pub number_unit_bonus: f32, pub number_magnitude_bonus: f32, pub number_energy_bonus: f32,
    pub overlay_lead_ms: u32, pub number_hold_ms: u32,
}
impl Default for DetectorConfig { /* TS §17.4, §17.5, §19.5 values */ }
```

Defaults: thresholds 0.85, 0.85, 0.80, 0.80; number scoring 0.50, 0.30, 0.20, 0.10 (TS §17.4); `overlay_lead_ms` 150 (TS §17.5); `number_hold_ms` 1,400 (the 1,200 ms hold plus the 200 ms fade of TS §19.5). All four thresholds exist now so V3 adds fields and never renames one.

### 9.2 `lib.rs` and `event_id.rs`

```rust
pub fn detect(t: &Transcript, p: &Prosody, edits: &BTreeMap<WordIdx, String>, cfg: &DetectorConfig) -> Vec<DetectedEvent>;  // TS §17.1
pub fn event_id(kind: EventKind, anchors: WordRange) -> EventId;         // FNV-1a 64, byte layout of D-47
```

`redetect_sentence` is not in V2; V3 adds it beside `detect`.

`detect` in V2, a strict subset of TS §17.3:

1. `toks = tokenize(&t.words, edits)`.
2. `cands = number::find(&toks, t, p, cfg)`.
3. Keep `c.score >= cfg.threshold_number`.
4. **(D-49)** Sort by `span.start`. Drop a candidate whose window `[span.start - overlay_lead_ms, span.end + number_hold_ms]` overlaps the window of the last kept candidate. V3 replaces steps 3-4 with `exclusions::apply`, the per-kind filter and `resolve::resolve`.
5. Map each to `DetectedEvent { id: event_id(kind, anchors), kind, span, anchors, params, confidence: Confidence(score), enabled: true }`.

Output is ordered by `span.start`. An empty transcript gives an empty list (TS §17.6).

### 9.3 `number.rs`

```rust
pub struct Candidate { pub kind: EventKind, pub anchors: WordRange, pub span: Span, pub score: f32, pub params: EventParams }
pub fn find(toks: &[Token], t: &Transcript, p: &Prosody, cfg: &DetectorConfig) -> Vec<Candidate>;
```

One candidate per `NormalizedSpan` whose words all still have their original effective text (a span with an edited or hidden word is skipped in V2; V3 re-parses it). Score, per TS §17.4:

| Term | Value | V2 note |
|---|---|---|
| Base | 0.50 | |
| Currency, percent or explicit unit | +0.30 | V2 can only see `$` and `%` |
| Magnitude at least 1,000, or a suffix | +0.20 | Suffixes are V3 |
| `energy_z >= 1.0` on any anchor word | +0.10 | Always 0 in V2: prosody is neutral (D-22) |
| Clamp to 1.0, then multiply by the minimum ASR confidence of the anchor words | | 1.0 when TE-2 fails |

`span` = start of the first anchor word to end of the last. `params = NumberReveal { value: quantity, label: None }` (D-49).

**What this means for V2.** A bare number reaches at most 0.70 and is never shown. `40%` reaches 0.80 only when its word confidence is exactly 1.0. `$10,000` reaches 1.00 and is shown while its word confidence is 0.80 or more. That is why precondition 7 asks for a dollar amount of 1,000 or more, and why S6 checks the transcript for it. If the recognizer writes the amount without the `$`, the conditional form of section 8.4 is added (D-42).

### 9.4 Inline unit tests

| Case | Expect |
|---|---|
| Transcript with "$10,000" at confidence 0.9 | One event; kind `NumberReveal`; confidence 0.9; display `$10k` |
| "ten thousand" (no unit) | No event (0.70) |
| "40%" at confidence 1.0; at 0.99 | One event; none |
| Two `$` amounts 600 ms apart | Only the earlier one (step 4) |
| Two `$` amounts 5 s apart | Both, in time order |
| Any returned event | `confidence >= threshold_number` (INV-6) |
| Same inputs twice | Identical output, identical ids |
| `event_id(NumberReveal, 5..6)` | Equals the FNV-1a 64 of bytes `00 05 00 00 00 06 00 00 00` |
| Same anchors, different kind | Different id |

---

## 10. `offcut-entitlement`: token, verification, export profile

**Crate rules** (TS §2). Depends on `offcut-types`, `ed25519-dalek`, `serde`, `serde_json`, `base64`, `thiserror` (D-54). No network, no storage, no clock (the caller passes `now`), no randomness: `ed25519-dalek` is built without its `rand_core` feature. `Cargo.toml` declares `[features] sign = []`; no code is behind it until V6.

**Never.** Counts exports. Logs or formats a token. Returns a profile from an unverified token: `export_profile` takes claims, and the only way to get claims is `verify_token`.

### 10.1 `lib.rs`, `claims.rs`

```rust
pub enum TokenError { Format, Base64, Json, Version, Signature }
```

`claims.rs`: copy `EntitlementClaims` of TS §10.6 verbatim. It derives `Clone, Debug, PartialEq, Serialize, Deserialize` with `#[serde(deny_unknown_fields)]`. It is not generated to TypeScript in V2.

### 10.2 `token.rs` (D-25)

```rust
pub struct ParsedToken<'a> { pub signed: &'a [u8], pub signature: [u8; 64], pub claims: EntitlementClaims }
pub fn decode(token: &str) -> Result<ParsedToken<'_>, TokenError>;
pub fn encode(claims: &EntitlementClaims, signature: &[u8; 64]) -> String;    // used by tests now, by sign.rs in V6
pub fn signing_input(claims: &EntitlementClaims) -> String;                    // base64url(JSON), no padding
```

| Input | Result |
|---|---|
| Not exactly two non-empty segments separated by one `.` | `Format` |
| A segment that is not unpadded base64url; a signature that is not 64 bytes | `Base64` |
| A first segment that is not the JSON of the claims (unknown field, missing field, wrong type) | `Json` |
| `v != 1` | `Version` |
| Longer than 2,048 characters (D-60) | `Format`, before any decoding |

### 10.3 `verify.rs`

```rust
pub fn verify_token(token: &str, public_keys: &[[u8; 32]]) -> Result<EntitlementClaims, TokenError>;
```

`decode`, then verify the signature over `signed` against each key in order; the first match returns the claims; none gives `Signature`. A key that is not a valid Ed25519 point is skipped. Expiry is not checked here: `export_profile` handles it. Every `TokenError` becomes `E_ENTITLEMENT_INVALID` (TS §11.2), mapped in `profile_api.rs`.

### 10.4 `profile.rs` [ONLY builder of `ExportProfile`]

```rust
pub const CREATOR_VIDEO_BITRATE: BitsPerSec = BitsPerSec::new(8_000_000);   // TS §21.2 (assumption, TE-4)
pub const FREE_VIDEO_BITRATE: BitsPerSec    = BitsPerSec::new(4_000_000);
pub const AUDIO_BITRATE: BitsPerSec         = BitsPerSec::new(160_000);
pub const PREVIEW_WIDTH: Px = Px::new(540);  pub const PREVIEW_HEIGHT: Px = Px::new(960);   // TS §20.3
pub fn export_profile(claims: Option<&EntitlementClaims>, now: UnixSecs) -> ExportProfile;  // TS §21.1
```

| Claims | Profile |
|---|---|
| `None` | `Preview`, 540x960, no watermark, bitrates 0 (D-26) |
| `plan == Creator` and `now <= period_end` and `now <= exp` | `Creator`, `CREATOR_WIDTH` x `CREATOR_HEIGHT`, no watermark, 8,000,000 and 160,000 b/s |
| Anything else | `Free`, `FREE_WIDTH` x `FREE_HEIGHT`, watermark, 4,000,000 and 160,000 b/s |

Sizes come from `offcut_types::limits`. This function is INV-9: nothing else chooses a size or a watermark flag.

### 10.5 Inline unit tests

BP places `tests/token_roundtrip.rs` and `tests/profile.rs` in V6; V2 covers the same ground inline.

| Case | Expect |
|---|---|
| **Cross-language vector.** A token minted by `fake-api.ts::mintEntitlementToken` with the fixed test seed and fixed claims, pasted into the test as a constant, verified with the test public key | `Ok`, claims equal the fixed claims |
| The same token with one payload character changed | `Signature` |
| The same token verified with a different key; with `[wrong, right]` | `Signature`; `Ok` |
| `"abc"`, `"a.b.c"`, `".sig"`, padded base64 | `Format` or `Base64` as in 10.2 |
| Payload JSON with an extra field; with `v: 2` | `Json`; `Version` |
| `export_profile(None, _)` | `Preview`, 540x960 |
| Creator claims, `now` before both ends | 1080x1920, no watermark |
| Creator claims, `now > exp`; `now > period_end` | `Free`, 720x1280, watermark |
| Free claims | `Free` |

---

## 11. `offcut-scene`: Clean captions and NumberReveal

**Crate rules** (TS §2, §19.6). Depends on `offcut-types`, `offcut-text` (D-28, `format_quantity` only), `parley`, `serde`, `thiserror`. No `HashMap` (use `BTreeMap`), no accumulated float time, no clock, no randomness. **Never** touches a GPU, a pixel or a decoder; never moves a timestamp (INV-5): every time in a scene is a transcript time used unchanged.

### 11.1 `lib.rs`

Copy `SceneInput`, `build_scene`, the `Scene` methods and `CropRect` of TS §19.2 verbatim. `pub enum SceneError { Font, Layout }`; both map to `E_INTERNAL` in `session.rs`.

`build_scene`, following TS §19.3:

1. `crop = framing::crop_rect(clip, edit.crop_offset)`.
2. `style = styles::spec(edit.style)`.
3. Effective words: apply `edit.word_edits`, drop hidden words, and replace the words of each `NormalizedSpan` by one caption word carrying `quantity.display` with the span's first start and last end (TS §17.2).
4. `chunks = captions::chunk(&words, &transcript.sentences, &style)`, each chunk shaped once.
5. For each event with `enabled` (after overlaying `edit.event_overrides`): `events::layout(event, &style)`; `None` drops the event (TS §19.7).
6. Store `scale = profile.width / 1080` (TS §19.1). `profile.watermark` is not read in V2.

`frame_at(t)`: draw the chunk visible at `t`, then each event visible at `t`, then scale every command by `scale`. It is a pure function of `(scene, t)` and allocates only the returned list. `duration()` is `clip.duration`; `frame_count()` is `ceil(duration_ms x 30 / 1000)` in integer arithmetic. `summary()` in V2 returns `ChangeSummary { voice_cleaned: false, captions_emphasized: 0, visual_moments: <events placed in step 5> }` (D-22).

### 11.2 `safe_area.rs` [ONLY], `styles.rs` [ONLY], `easing.rs`, `anim.rs`

`safe_area.rs`: the constants of TS §19.1 verbatim (canvas 1080x1920 lp; insets top 250, bottom 420, left 60, right 120; `EVENT_ZONE` y 860-1180; `CAPTION_ZONE` y 1220-1500; `WATERMARK_ANCHOR`).

`styles.rs`: `pub struct StyleSpec` with one field per column of the TS §19.4 table (font, size, case, `max_chars_per_line`, `max_lines`, `max_words`, fill, stroke width and color, `active_color`, `pop_color`, easing, motion duration). `pub fn spec(id: StyleId) -> StyleSpec` matches all three ids explicitly; in V2 the `Bold` and `Tech` arms return the Clean row. V4 replaces those two arm bodies with their rows.

`easing.rs`: `pub enum Easing { Linear, EaseOutCubic, BackOut }` and `pub fn ease(e: Easing, x: f32) -> f32` on `0.0..=1.0`, clamped.

`anim.rs`: `pub struct Track { pub keys: Vec<(TimeMs, f32)>, pub easing: Easing }` and `pub fn eval(track: &Track, t: TimeMs) -> f32`: before the first key its value, after the last key its value, between two keys the eased interpolation. Time is always computed from `t`, never accumulated.

### 11.3 `fonts.rs`, `layout.rs`

`fonts.rs` embeds the three files of `assets/fonts/` with `include_bytes!` and builds the parley font context once per scene. `layout.rs`:

```rust
pub struct ShapedRun { pub font: FontId, pub size: f32, pub glyphs: Vec<PositionedGlyph>, pub width: f32,
                       pub ascent: f32, pub descent: f32, pub text: String }
pub fn shape(text: &str, font: FontId, size: f32) -> Result<Vec<ShapedRun>, SceneError>;
```

A glyph missing from the style font is shaped with `FontId::NotoEmoji`; missing there too, it is skipped (TS §19.6). One call may therefore return more than one run. No other font file may exist in the repository (TS §19.4); `LICENSES.md` lists the three.

### 11.4 `display_list.rs`

Copy `DisplayList` and `DrawCmd` of TS §19.2 verbatim. Supporting types (D-29), all `Clone, Debug, PartialEq, Serialize`:

```rust
pub struct Rect { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }
pub struct Rgba { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }                  // straight alpha
pub enum   PathEl { MoveTo(f32, f32), LineTo(f32, f32), QuadTo(f32, f32, f32, f32),
                    CurveTo(f32, f32, f32, f32, f32, f32), Close }
pub struct PositionedGlyph { pub id: u32, pub x: f32, pub y: f32 }               // baseline origin
pub struct Stroke { pub width: f32, pub color: Rgba }
pub struct Affine { pub m: [f32; 6] }                                           // a b c d e f, column-major 2x3
pub enum   FontId { Inter700, Inter900, JetBrainsMono700, NotoEmoji }           // closed; order is part of snapshots
```

V2 emits `Inter700` and `NotoEmoji` only. The enum is complete now so V4's snapshots never see a renumbering.

### 11.5 `captions.rs`

```rust
pub struct Chunk { pub words: Vec<CaptionWord>, pub show: Span, pub lines: Vec<Line> }
pub fn chunk(words: &[CaptionWord], sentences: &[Sentence], style: &StyleSpec) -> Result<Vec<Chunk>, SceneError>;
pub fn draw(chunk: &Chunk, t: TimeMs, style: &StyleSpec, out: &mut Vec<DrawCmd>);
```

Chunking and timing are TS §19.4 verbatim: a new chunk at a sentence boundary, at a gap of `CHUNK_GAP = 350 ms` or more, when the characters would exceed `max_chars_per_line x max_lines`, or at `max_words`. A chunk is visible from its first word start minus 80 ms to its last word end plus 120 ms, clamped so chunks never overlap. Lines are centred in `CAPTION_ZONE` and broken greedily at `max_chars_per_line`. The active word (`start <= t < end`) is drawn in `active_color`; the rest in the fill color, with the style's stroke. A pause between two words draws nothing early (TS §19.7). The KeywordPop scale is V4.

### 11.6 `framing.rs` [ONLY crop rectangle]

`pub fn crop_rect(clip: &ClipInfo, offset: CropOffset) -> CropRect`: the formula of TS §19.3 verbatim, with `offset` replaced by 0.0 in V2 (BP §4.1 C: "portrait pass-through"). V4 removes that one substitution. A landscape or square clip is therefore centre-cropped; a portrait clip narrower than 9:16 is cropped top and bottom.

### 11.7 `events/mod.rs`, `events/number_reveal.rs`

```rust
// mod.rs
pub enum EventTrack { NumberReveal(number_reveal::Track) }                 // V4 adds three variants
pub fn layout(e: &DetectedEvent, style: &StyleSpec) -> Result<Option<EventTrack>, SceneError>;
pub fn draw(track: &EventTrack, t: TimeMs, out: &mut Vec<DrawCmd>);
```

`layout` matches every `EventParams` variant explicitly; in V2 the `ListReveal`, `FromTo` and `KeywordPop` arms return `Ok(None)`.

`number_reveal.rs`, per TS §19.5:

| Phase | Rule |
|---|---|
| Fit | Shape the final display at 200 lp. Wider than 900 lp: scale the size down to fit. Below 96 lp: return `None` (TS §19.7) |
| Enter | At `span.start`. Integer value of at least 10: count-up from 0 over 500 ms, displayed value `round(lerp(0, value, ease(x)))` formatted by `format_quantity(v, &unit)`. Otherwise scale 0.8 to 1.0 over 200 ms |
| Body | Centred in `EVENT_ZONE`; unit inline (it is part of the display string). The label row at 48 lp is drawn only when `label` is `Some`, which V2 never produces |
| Exit | Hold to `span.end + 1,200 ms`; fade over 200 ms |

Count-up strings are shaped at layout time, one run per output frame of the 500 ms window (at most 16), keyed by frame index; `draw` picks the run for `floor((t - span.start) x 30 / 1000)`. `frame_at` therefore never shapes text.

### 11.8 Budgets and inline tests

Scene build at most 1 s for a 60 s clip (PS §20.2), which is 1.25 s for the reference clip (D-64); `frame_at` average at most 2 ms on R1 (assumption, M2.4). Both are read by `bench/device-bench.ts` through `stage_timing stage=detect_scene` and the render stage; V2 records them and does not gate on the second.

| Inline test | Expect |
|---|---|
| A 60,000 ms clip; a 20,033 ms clip | `frame_count()` 1,800; 601 |
| 12 words in one sentence, Clean | Chunks of at most 5 words, at most 2 lines of 18 characters |
| Words at 1,000-1,300 ms and 5,000-5,300 ms | Two chunks; nothing visible at 3,000 ms |
| `frame_at(t)` inside a chunk | Exactly one glyph run uses `active_color` |
| NumberReveal `$10k`, span 2,000-2,400 ms | Nothing at 1,900; a run at 2,100 whose text is a smaller amount; `$10k` at 2,600; nothing at 3,900 |
| A NumberReveal whose display cannot fit at 96 lp | Dropped; `summary().visual_moments` is 0 |
| Every command of every frame of the above | Bounding box inside the safe area |
| `build_scene` twice, all frames | Equal display lists (INV-7; the full `determinism.rs` is V4) |
| 1920x1080 clip, any offset | `CropRect { x: 656.25, y: 0, w: 607.5, h: 1080 }` |
| 1080x1920 clip | The whole frame |
| Profile width 540 | Every coordinate is half its 1080 value |

---

## 12. `offcut-render`: GPU compositor

**Crate rules** (TS §2). Depends on `offcut-types`, `offcut-scene`, `wgpu`, `vello`, `web-sys`, `thiserror`. **Never** detects, runs DSP, muxes or uses the network; never knows what a scene means; never closes a `VideoFrame` (TS §19.6: the caller does).

Copy the `Renderer` signatures of TS §19.2 verbatim. `render_to_image` is V4 (with `golden_frames.rs`).

```rust
pub enum RenderError { NoAdapter, DeviceRequest, DeviceLost, Surface, FrameImport, Overlay }
```

| Error | Code | Mapped in |
|---|---|---|
| `NoAdapter`, `DeviceRequest`, `Surface` | `E_GPU_INIT` | `session.rs` |
| `DeviceLost` | `E_GPU_LOST` | `session.rs` |
| `FrameImport`, `Overlay` | `E_INTERNAL` | `session.rs` |

| File | Does |
|---|---|
| `gpu.rs` | Requests one adapter and one device, creates the surface from the `OffscreenCanvas`, configures it at the given size. Registers the device-lost callback, which sets a flag that the next `render` reports as `DeviceLost` |
| `video_pass.rs` + `shaders/video.wgsl` | Copies the `VideoFrame` into a texture with the queue's external-image copy, then draws it to the target with a transform built from `crop` and `rotation`: sampling axes are swapped for `R90`/`R270` and `crop` is in display coordinates (TS §19.7) |
| `vello_backend.rs` | Translates a `DisplayList` to a `vello::Scene` command by command (`FillRect`, `FillPath`, `GlyphRun` with fill then stroke, `PushLayer`/`PopLayer`) and renders it to the overlay texture. Font data comes from `offcut-scene`'s embedded bytes by `FontId` |
| `composite.rs` | Draws the video pass, then the overlay with premultiplied-alpha blending, to the surface texture, and presents |
| `lib.rs` | `Renderer::new`, `resize` (reallocates the overlay and target only when the size changes, TS §31), `render`; `pub use offcut_scene as scene;` (D-61) |

One device per `Renderer`; textures are created in `new` and `resize` only. **Budget:** TE-3 requires at least 30 frames per second render-only on R1 (section 24). No test in V2 runs this crate natively.

---

## 13. WASM binding crates

**Rules for both** (TS §2). `wasm-bindgen` exports and value conversion only: no product rule, no branch on a domain value beyond decoding arguments and mapping an error enum to a code. Conversion uses D-31. Every fallible export returns `Result<_, JsValue>` whose error is the plain object `{ code: ErrorCode, detail: string }` (TS §11.1); `detail` holds an enum variant name and never file content, a token or transcript text. Both crates install the V1 panic hook (V1 §8).

### 13.1 `offcut-wasm-core/src/media_api.rs`

```rust
#[wasm_bindgen] pub struct DemuxerHandle { /* Demuxer<JsRandomAccess> */ }
#[wasm_bindgen] pub fn open_demuxer(handle: web_sys::FileSystemSyncAccessHandle) -> Result<DemuxerHandle, JsValue>;
#[wasm_bindgen] pub fn probe_and_validate(d: &DemuxerHandle, file_size: f64, decode_supported: bool) -> Result<JsValue, JsValue>;
#[wasm_bindgen] pub fn resample(input: &[f32], from_hz: u32, to_hz: u32) -> Vec<f32>;
#[wasm_bindgen] impl DemuxerHandle {                                              // D-53
    pub fn probe(&self, file_size: f64) -> Result<JsValue, JsValue>;              // ProbeInfo
    pub fn video_description(&self) -> Option<Vec<u8>>;
    pub fn audio_description(&self) -> Option<Vec<u8>>;
    pub fn audio_sample_count(&self) -> u32;
    pub fn read_audio_sample(&mut self, index: u32) -> Result<JsValue, JsValue>;  // { data: Uint8Array, ptsUs: number, durationUs: number }
}
```

`JsRandomAccess` (private, D-30) calls `handle.getSize()` once and reads through `handle.read(buffer, { at })`; a short read is `IoError::Read`. As built (Prompt 37) it keeps one window of 1 MiB read ahead: a read smaller than the window that the window does not hold moves the window to start there, so reading a clip's audio frames one after the other costs a few dozen calls into the browser, not one per frame (one call was measured at about 0.4 ms, whatever its size). The file must not change while a demuxer is open on it.

### 13.2 Return and error shapes

| Call | Success | Failure |
|---|---|---|
| `open_demuxer` | `DemuxerHandle` | `{ rejected: "REJECT_CONTAINER" }` or `{ rejected: "REJECT_CORRUPT" }` per section 6.1; `{ code: "E_STORAGE_IO", detail }` for `Io` |
| `probe` | `ProbeInfo` | `{ code: "E_INTERNAL", detail }` (serialization only) |
| `probe_and_validate` | `{ ok: ClipInfo }` or `{ rejected: RejectReason }` (TS §11.3: a rejection is a value) | As `probe` |
| `read_audio_sample` | The object above | `{ code: "E_DECODE_AUDIO", detail }`. A failure after acceptance is an error, never a rejection |

### 13.3 `hash_api.rs`, `text_api.rs`, `load-core.ts`

```rust
#[wasm_bindgen] pub struct Sha256Stream { /* sha2::Sha256 */ }
#[wasm_bindgen] impl Sha256Stream { #[wasm_bindgen(constructor)] pub fn new() -> Sha256Stream;
                                    pub fn update(&mut self, chunk: &[u8]);
                                    pub fn finalize_hex(self) -> String; }        // lower-case hex
#[wasm_bindgen] pub fn normalize_transcript(raw: JsValue, model_version: &str) -> Result<JsValue, JsValue>;   // Transcript
```

```ts
// wasm/load-core.ts: CoreApi grows (V1 §11.9)
export type CoreApi = {
  coreVersion(): string;
  openDemuxer(handle: FileSystemSyncAccessHandle): CoreDemuxer | { rejected: RejectReason };   // section 13.2
  probeAndValidate(d: CoreDemuxer, fileSize: Bytes, decodeSupported: boolean): { ok: ClipInfo } | { rejected: RejectReason };
  resample(input: Float32Array, from: Hz, to: Hz): Float32Array;
  normalizeTranscript(raw: readonly { text: string; startMs: number; endMs: number; confidence: number }[], modelVersion: string): Transcript;   // the RawWord shape of TS §16.2
  newSha256(): { update(chunk: Uint8Array): void; finalizeHex(): string };
};
```

`load-core.ts` stays the only instantiation site of `offcut_core.wasm`. It wraps each export so that a thrown `{ rejected }` value is returned as a value and a thrown `{ code, detail }` is rethrown unchanged for `rpc.ts::toAppFailure`.

### 13.4 `offcut-wasm-render`: `lib.rs`, `detect_api.rs`, `profile_api.rs`, `mux_api.rs`

```rust
// detect_api.rs
#[wasm_bindgen] pub fn detect(transcript: JsValue, prosody: JsValue, edits: JsValue) -> Result<JsValue, JsValue>;   // DetectedEvent[]; DetectorConfig::default()
// profile_api.rs
#[wasm_bindgen] pub fn export_profile_from_token(token: &str, public_keys: js_sys::Array, now_unix_secs: f64) -> Result<JsValue, JsValue>;  // ExportProfile
#[wasm_bindgen] pub fn preview_profile() -> JsValue;                               // export_profile(None, _)  (D-53)
// mux_api.rs
#[wasm_bindgen] pub struct Mp4MuxerHandle { /* Mp4Muxer<JsMuxSink> */ }
#[wasm_bindgen] impl Mp4MuxerHandle {
    #[wasm_bindgen(constructor)]
    pub fn new(sink: JsValue, width: u32, height: u32, avcc: &[u8], frame_count_hint: u32, asc: &[u8]) -> Result<Mp4MuxerHandle, JsValue>;
    pub fn add_video_sample(&mut self, data: &[u8], frame: u32, is_keyframe: bool) -> Result<(), JsValue>;
    pub fn add_audio_sample(&mut self, data: &[u8], pts_us: f64, duration_us: f64) -> Result<(), JsValue>;
    pub fn finalize(self) -> Result<f64, JsValue>;                                  // total bytes
}
```

- `export_profile_from_token`: each array element must be a 32-byte `Uint8Array`, others are skipped. `verify_token` then `export_profile(Some(&claims), now)`. Any `TokenError` gives `{ code: "E_ENTITLEMENT_INVALID", detail: <variant> }`. The audio track is always 48,000 Hz, 2 channels (TS §21.2).
- `JsMuxSink` calls `sink.writeAt(offset, data)` on the object it was given (an `OpfsSink`). A throw becomes `IoError::Write`.
- `MuxError` gives `{ code: "E_MUX", detail }`.

### 13.5 `session.rs`

Copy `RenderSession` of TS §19.2 verbatim, and add the four methods that make it a `DemuxerHandle` (D-30):

```rust
#[wasm_bindgen] impl RenderSession {
    pub fn video_description(&self) -> Option<Vec<u8>>;
    pub fn video_sample_count(&self) -> u32;
    pub fn read_video_sample(&mut self, index: u32) -> Result<JsValue, JsValue>;   // { data: Uint8Array, ptsUs, durationUs, isKeyframe }
    pub fn keyframe_at_or_before(&self, t_ms: u32) -> u32;
}
```

| Method | Behaviour |
|---|---|
| `open(clip, source)` | Opens the demuxer on the sync handle. No GPU work yet |
| `set_scene(input)` | `input` is `{ transcript, events, edit, profile: ExportProfile }`; the `ClipInfo` is the one given to `open`. Calls `build_scene`; replaces the stored scene. The renderer is not touched |
| `attach_canvas(canvas, w, h)` | First call: `Renderer::new`. Later calls: a new surface on the same device, then `resize` |
| `render_frame(frame, t)` | `scene.frame_at(t)`, then `renderer.render(frame, scene.crop(), clip.rotation, &list)`. No scene or no canvas: `{ code: "E_INTERNAL" }` |
| `frame_count()`, `summary()` | From the scene; `summary()` returns `ChangeSummary` |

The session owns the device, the Vello renderer, the overlay texture, the scene and the demuxer (TS §19.6). It closes no `VideoFrame`. `build_scene`, `SceneInput` and `Scene` are named through `offcut_render::scene` (D-61); this crate's `Cargo.toml` does not list `offcut-scene`.

---

## 14. Server: no changes

No file under `server/`, `crates/offcut-api-types/` or `crates/offcut-types/` changes in V2 (V1 §15.2). `offcut-api` does not gain a dependency on `offcut-entitlement` until V6. `deny.toml` already bans the media crates for package `offcut-api`; the first V2 commit that adds them to the workspace is covered by that ban (V1 §12.1). `deploy-api.yml` still runs on every merge to `main` and redeploys an unchanged image.

---

## 15. Web infrastructure

**Layer rules every file below obeys** (TS §2, §7, V1 §11), with the edges of D-27:

- `ui` imports only `usecases`, `state` (read and subscribe), `copy`, `config`, and the two names V1 lets it take from `net/asset-fetch` (`assetUrl`, `DEMO_VIDEO_PATH`).
- `usecases` never call `fetch`, WebCodecs, WebGPU or a WASM export, and never import each other (TS §7), except the two imports of D-58.
- `state` imports only `gen` types, other `state` files and the types of `workers/protocol.ts` (D-27 f), and has no side effects.
- `persistence` never imports `net`, `ui` or `workers`. `models` never imports `net/http` or `ui`.
- Workers never touch `state`, `ui`, IndexedDB or another worker, and never call `fetch` except through `wasm/load-*.ts`.
- `fetch` exists only in `net/http.ts`, `net/asset-fetch.ts`, `wasm/load-core.ts`, `wasm/load-render.ts`.
- No user-facing sentence outside `copy/messages.ts`. No `any`, no non-null `!`, no `@ts-ignore`, no empty `catch`. Every `switch` over a generated union ends in `assertNever`.
- `as <Brand>` casts only in `gen/` and `workers/` (TS §10.1), plus the single minting line in each file listed in D-59.

### 15.1 `config/env.ts` [ONLY reader of `import.meta.env`], `config/entitlement-public-key.ts`, `config/model-manifest.json`

```ts
// env.ts: one field added
export const env: { readonly assetBaseUrl: string; readonly dev: boolean; readonly entitlementTestPublicKey: string | null };
// entitlement-public-key.ts
export const ENTITLEMENT_PUBLIC_KEYS: readonly Uint8Array[];   // [production] or [production, test]
```

- `entitlementTestPublicKey` is `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` when set and non-empty, else `null`. A set value that does not decode to 32 bytes throws at module load.
- The production key is a base64 literal in `entitlement-public-key.ts` (precondition 12). It is a public value (TS §24.2).
- `model-manifest.json`: the shape of TS §16.1 verbatim, filled at S5. Each `files[].path` is the path `upload-assets.sh` printed for that file (D-62). `modelId` is `"asr-en-v1"`. `totalBytes` must be at most 260,000,000 (TS §16.1, D-66); a unit assertion in `download.test.ts` reads the file and checks it and that `totalBytes` equals the sum of `files[].bytes`.

**Never.** `entitlement-public-key.ts` never holds a private key or seed. `env.ts` never exposes a variable that is not listed here.

### 15.2 `persistence/opfs.ts` [ONLY builder of OPFS paths]

```ts
export const paths: {
  clipsRoot(): string;                               // "clips"
  clipDir(id: ClipId): string;                       // "clips/<clipId>"
  clipSource(id: ClipId): string;                    // "clips/<clipId>/source"
  clipOut48(id: ClipId): string;                     // "clips/<clipId>/out48.f32"
  exportsRoot(): string;                             // "exports"
  exportTmp(id: ExportId): string;                   // "exports/tmp/<exportId>.mp4"
  exportFinal(id: ExportId): string;                 // "exports/<exportId>.mp4"
  modelDir(modelId: string): string;                 // "models/<modelId>"
  modelPart(modelId: string, name: string): string;  // "models/<modelId>/<name>.part"
  modelFile(modelId: string, name: string): string;  // "models/<modelId>/<name>"
};
export function fileHandle(path: string, o: { create: boolean }): Promise<FileSystemFileHandle>;
export function getFile(path: string): Promise<File>;
export function size(path: string): Promise<Bytes | null>;                   // null when absent
export function writeAt(path: string, offset: Bytes, data: Uint8Array): Promise<void>;   // D-36
export function truncate(path: string, length: Bytes): Promise<void>;
export function move(from: string, to: string): Promise<void>;
export function remove(path: string): Promise<void>;                         // file or directory, recursive; absent is not an error
export function list(dir: string): Promise<readonly string[]>;               // entry names; absent directory gives []
export function writeAudio(id: ClipId, out48: Float32Array): Promise<void>;  // raw little-endian f32
export function readAudio(id: ClipId): Promise<Float32Array>;                // D-51
```

The path table is TS §23.1 verbatim. Names are ids and fixed words only; no path contains a file name supplied by the user (TS §25.1 P-11). `modelFile` and `modelPart` reject a `name` containing `/` or `..`. A `DOMException` named `QuotaExceededError` is rethrown as an error tagged `quota`; any other storage failure is tagged `io`; callers map them to `E_STORAGE_QUOTA` / `E_MODEL_STORAGE` and `E_STORAGE_IO`. Workers import this file for `paths` and `fileHandle` only (D-27 b) and call `createSyncAccessHandle()` themselves.

**Never.** Imports `net`, `ui` or `workers`. Opens a sync access handle. Deletes the model directory except through `remove` called by the model manager.

### 15.3 `persistence/entitlement-repo.ts` (D-23)

```ts
export function get(): Promise<OffcutDb["entitlement"]["value"]["value"] | undefined>;   // { token, storedAt } as typed in schema.ts
export function put(token: string): Promise<void>;
```

Store `entitlement`, key `"current"`, value wrapped as `Versioned` with `schemaVersion: 1` (TS §23.1). `get` returns `undefined` for an absent record and for a record with a newer `schemaVersion` (TS §23.3). It does not parse or verify the token. `clear()` arrives in V6.

### 15.4 `net/asset-fetch.ts` [ONLY fetch to the asset host]

```ts
export const SAMPLE_CLIP_PATH: string;                                  // "media/<content-hashed name>.mp4" (D-39)
export type AssetResult = { ok: true; status: 200 | 206; response: Response }
                        | { ok: false; cause: "offline" | "status" | "aborted"; status?: number };
export function fetchAsset(path: string, o: { range?: { start: Bytes; endInclusive: Bytes }; signal: AbortSignal }): Promise<AssetResult>;
```

| Situation | Result |
|---|---|
| Any call | `GET assetUrl(path)` with `credentials: "omit"`, `mode: "cors"`, `cache: "no-store"`, no body, no query string (TS §24.1 row 2). `assetUrl` already rejects `?` and `..` (V1 §11.6) |
| `range` given | Header `Range: bytes=<start>-<endInclusive>` and no other custom header |
| `navigator.onLine === false`, or `fetch` rejects with `TypeError` | `cause: "offline"` |
| The signal aborts | `cause: "aborted"` |
| Status 200 or 206 | `ok: true`; the body is not read here |
| Any other status | `cause: "status"` with the status |

It does no retry and maps nothing to an `ErrorCode`: the caller knows the context. **Never** sends a credential, a cookie or user data; never contacts a host other than the asset host.

### 15.5 `wasm/load-render.ts` [ONLY instantiation of `offcut_render.wasm`]

```ts
export type RenderApi = {
  detect(transcript: Transcript, prosody: Prosody, edits: Record<number, string>): DetectedEvent[];
  exportProfileFromToken(token: string, keys: readonly Uint8Array[], now: UnixSecs): ExportProfile;
  previewProfile(): ExportProfile;
  openSession(clip: ClipInfo, source: FileSystemSyncAccessHandle): Promise<RenderSession>;
  newMuxer(sink: { writeAt(offset: number, data: Uint8Array): void }, v: { width: Px; height: Px; avcc: Uint8Array; frameCountHint: number }, asc: Uint8Array): Mp4MuxerHandle;
};
export function preloadRender(): Promise<void>;     // fetch + WebAssembly.compileStreaming; caches the Module; idempotent
export function loadRender(): Promise<RenderApi>;   // instantiates from the cached Module
```

Same construction as `load-core.ts` (V1 §11.9): glue from `web/src/wasm/pkg/render/`, the `.wasm` as a Vite `?url` asset. The main thread calls only `preloadRender`; only `render.worker` calls `loadRender`.

### 15.6 `workers/rpc.ts`

```ts
export class WorkerCallError extends Error { readonly failure: AppFailure }
export type Cancelled = { cancelled: true };
export type CallOptions = { transfer?: Transferable[]; onProgress?: (p: Progress) => void; onJob?: (jobId: JobId) => void };
export type Client<Api> = { [M in keyof Api]: Api[M] extends (p: infer P) => Promise<infer R>
                              ? (p: P, o?: CallOptions) => Promise<R | Cancelled> : never };
export function createClient<Api>(worker: Worker, stage: FailureStage): {
  call: Client<Api>; notify(method: string, params: unknown): void; cancel(jobId: JobId): void; dispose(): void };
export type Handlers<Api> = { [M in keyof Api]: (params: never, ctx: JobContext) => Promise<unknown> | void };
export type JobContext = { jobId: JobId; isCancelled(): boolean; progress(p: Omit<Progress, "type" | "jobId">): void };
export function serveWorker<Api>(handlers: Handlers<Api>, o: { stage: FailureStage; oneWay: readonly string[];
                                 duringPreview: readonly string[] }): void;
export function toAppFailure(thrown: unknown, stage: FailureStage): AppFailure;
export const CANCELLED: unique symbol;      // a handler returns it after seeing its cancel flag
```

Client side:

| Situation | Result |
|---|---|
| `call.m(p, o)` | Allocates the next `JobId` (a counter per client), posts `Req` with `o.transfer` as the transfer list, calls `o.onJob` |
| `Res` with `ok: true` | Resolves with `result` |
| `Res` with `cancelled: true` | Resolves with `{ cancelled: true }` (TS §11.3) |
| `Res` with `ok: false` | Rejects with `WorkerCallError(failure)` (D-32) |
| `Progress` for a pending job | `o.onProgress(p)` |
| A message for an unknown or finished `jobId` | Dropped (TS §14.4) |
| Worker `error` or `messageerror` event | Every pending call rejects with `{ code: "E_WORKER_CRASH", stage, retryable: true }` (TS C-14). V4 adds the restart |
| `cancel(jobId)` | Posts `Cancel`. V2 has no caller and no timeout (V4) |
| After posting, in every build | Every `ArrayBuffer` in the transfer list must be detached (TS §14.3); if one is not, the call rejects with `E_INTERNAL`. (Prompt 37: `workers/` may not read `import.meta.env` or import `config/`, so the file cannot tell a development build; the check costs nothing, as with D-45) |

Worker side (`serveWorker`):

| Situation | Result |
|---|---|
| `Req` while a job is pending | `Res` with `E_INTERNAL`, `retryable: false` (TS §14.3), unless the method is listed in `duringPreview` and the pending job is `previewPlay` |
| A method with no handler | `Res` with `E_INTERNAL`, `retryable: false` (D-32) |
| A method listed in `oneWay` | Runs the handler; posts nothing |
| Handler resolves | `Res ok: true`. A result holding `Float32Array`s or an `OffscreenCanvas` is posted with them in the transfer list |
| Handler resolves while its cancel flag is set and it returned the `CANCELLED` sentinel | `Res cancelled: true` |
| Handler throws | `Res ok: false` with `toAppFailure(thrown, stage)` |
| `Cancel` | Sets the flag for that `jobId`; `ctx.isCancelled()` reads it |

`toAppFailure`: a thrown `{ code, detail, stage? }` object keeps its code, and its `stage` when present, else the worker's default stage from the pool table; a thrown object already shaped like `AppFailure` passes through; an `Error` whose message starts with `E_WORKER_CRASH` (the panic hook, V1 §8) gives `E_WORKER_CRASH`; anything else gives `E_INTERNAL`. `retryable` follows the "Retryable" column of TS §11.2, held as one table in this file. `detail` carries the thrown value's `detail` or the error's name; it is shown only in dev builds and is never sent anywhere (TS §11.1).

**Never.** Copies a buffer that could be transferred. Throws across the boundary for a rejection (`{ rejected }` is a result). Swallows a failure. Progress throttling (10 per second) is V4; V2 handlers post at most once per cancellation point.

### 15.7 `workers/pool.ts`

```ts
export function preload(): Promise<{ ok: true } | { ok: false; failure: AppFailure }>;   // V1 signature, unchanged
export { WorkerCallError, type Cancelled } from "./rpc";                                  // D-32
export const pool: { readonly media: PoolClient<MediaWorkerApi>; readonly asr: PoolClient<AsrWorkerApi>;
                     readonly render: PoolClient<RenderWorkerApi> };
type PoolClient<Api> = Client<Api> & { notify(method: string, params: unknown): void };
```

One table drives everything:

| Kind | Script | Stage tag | One-way | Accepted during `previewPlay` |
|---|---|---|---|---|
| `media` | `media.worker.ts` | `import` | none | none |
| `asr` | `asr.worker.ts` | `asr` | none | none |
| `render` | `render.worker.ts` | `preview` | `previewClock` | `previewClock`, `previewPause`, `previewSeek` (TS §14.3) |

- A worker is created on the first call to its client (`new Worker(new URL(...), { type: "module" })`) and kept (TS §14.3). V3 adds the `audio` row.
- `preload()` (V1 D-9) now does three things, each idempotent: `preloadCore()`; `preloadRender()`; one `<link rel="modulepreload">` per script in the table (D-32). A failure of either WASM preload returns the V1 failure value.
- `pool.ts` holds no job state and no retry. Restart, restart budget and `pool.cancel` are V4.

**Never.** Calls `fetch`. Lets one worker message another. Exposes the raw `Worker`.

---

## 16. Workers

**Rules for all three** (TS §2, §14). Browser API glue for one stage; no product rule; no `state`, `ui`, IndexedDB or network. Every obtained `VideoFrame` and `AudioData` is closed by the code that obtained it, in `finally` (TS §31). Each entry file is: `serveWorker(handlers, options)` and nothing else at top level. Cancellation points are TS §14.3 verbatim.

### 16.1 `media.worker.ts`

| Handler | Steps |
|---|---|
| `importAndProbe({ clipId, file })` | TS §15.2, in order: (1) `file.size > LIMITS.MAX_FILE_SIZE`: return `{ rejected: "REJECT_FILE_SIZE" }` before any copy. (2) `handle = importToOpfs(clipId, file, ctx.isCancelled)`. (3) `d = core.openDemuxer(handle)`; a `{ rejected }` value is returned as the result. (4) `p = d.probe(size)`. (5) `supported` = `p.video` is H.264 and `VideoDecoder.isConfigSupported({ codec: p.video.codec_string, description, codedWidth, codedHeight })` resolves `supported: true`; a throw counts as false. (6) return `core.probeAndValidate(d, size, supported)`. `finally`: free the demuxer, close the sync handle (TS §31: the source is closed before `render.worker` opens it). On a rejection or failure the caller removes the directory |
| `extractAudio({ clipId })` | (1) Open the source sync handle and a demuxer. (2) `mono = decodeAudio(handle, info, isCancelled)`. (3) `pcm48 = core.resample(mono, rate, 48000)`, `pcm16 = core.resample(mono, rate, 16000)`. (4) Pad with zeros or truncate each by at most one sample to `round(duration_ms x 48)` and `round(duration_ms x 16)` (D-34). (5) Return both, transferred. `finally`: close the handle |

`extractAudio` needs the `ClipInfo`; the protocol passes only `clipId` (TS §14.2), so the worker keeps the `ClipInfo` of its last accepted `importAndProbe` in a module variable, keyed by `clipId`. A call for another `clipId` re-probes the source.

### 16.2 `media/import.ts`

`importToOpfs(clipId, file, isCancelled)` (signature: TS §15.1): create `paths.clipSource(clipId)`, open a sync access handle, copy `file.slice(offset, offset + 4 MiB)` chunk by chunk with `handle.write(chunk, { at: offset })`, `flush()` at the end, return the open handle. `isCancelled()` is read before each chunk; when true, close the handle and return the `CANCELLED` sentinel. Progress `{ stage: "import", done, total }` in bytes once per chunk. `QuotaExceededError` becomes `E_STORAGE_QUOTA`; any other write failure `E_STORAGE_IO` (TS §8). It never reads the file name and never holds more than one chunk in memory.

### 16.3 `media/audio-decode.ts`

`decodeAudio(handle, info, isCancelled)` (signature: TS §15.1) returns mono `Float32Array` at the source rate:

1. Configure one `AudioDecoder` with `{ codec: info.audio_codec_string, sampleRate: info.audio_sample_rate, numberOfChannels: info.audio_channels, description }`.
2. Feed samples in decode order as `EncodedAudioChunk`s (`type: "key"`, `timestamp: ptsUs`), 32 at a time. Cancellation check per batch; after each batch one turn of the event loop, so that outputs are delivered and a cancel message is read; then wait for `dequeue` while `decodeQueueSize > 256`. (Corrected in Prompt 37: with the bound at 32 the decoder's queue stayed shallow and 75 s of audio took 1.7 s to decode; with 256 it takes about 0.5 s, and the output is the same, sample for sample.)
3. For each `AudioData`: copy every channel as `f32-planar`, average the channels into mono, append, `close()` in `finally`.
4. `flush()`, then `close()` the decoder.
5. Align (TS §15.1): let `first` be the `pts` of audio sample 0. Positive: prepend `round(first x rate / 1e6)` zero samples. Negative: drop that many samples from the head. Then pad with zeros or truncate the tail to `round(duration_ms x rate / 1000)`. This looks at timestamps only, never at audio content (INV-5).

The decoder's error callback, or a `decode` throw, gives `E_DECODE_AUDIO` (retry is V7). Peak memory: the mono buffer plus the two outputs, about 17.3 + 5.8 MB at 90 s plus the source-rate copy (TS §8).

### 16.4 `asr.worker.ts`

| Handler | Steps |
|---|---|
| `load({ modelId, backend })` | `loadModel(modelId, backend)`; returns the backend actually in use. Stores `modelId` for `model_version` (D-50). Failure: `E_ASR_RUNTIME`; an allocation failure (`RangeError` or an out-of-memory message from the runtime): `E_ASR_OOM` |
| `transcribe({ pcm16 })` | `raw = transcribe(pcm16, onWindow, ctx.isCancelled)` with progress `{ stage: "asr", done, total, feed: { kind: "transcribing" } }` on the first window and plain progress after; `t = core.normalizeTranscript(raw, modelId)`; `t.words.length < LIMITS.MIN_WORDS`: return `{ rejected: "NoSpeech" }`, else `{ ok: t }`. `pcm16` is dropped when the handler returns (TS §31). No model loaded: `E_INTERNAL` |
| `unload()` | `unloadModel()`; resolves only after the session is disposed (INV-12, TS §16.5). Idempotent |

### 16.5 `asr/whisper-runtime.ts`, `asr/model-cache-adapter.ts`, `asr/word-timestamps.ts`

`whisper-runtime.ts` is the only file that imports the ASR runtime package (TDR-3). Signatures: TS §16.2 verbatim.

`loadModel`, per TS §16.4 (exact option names are confirmed in TE-1 and recorded in `v2changelog.md`):

1. Disable remote model loading and the runtime's own browser cache.
2. Install `model-cache-adapter.ts` as the model cache.
3. Point the runtime's `.wasm` location at `/ort/<runtime version>/` on the app origin (D-37).
4. Try `webgpu`; on failure build the session on `wasm` with `threads = clamp(hardwareConcurrency - 2, 1, 4)`.
5. Request word-level timestamps and, when TE-2 passes, token probabilities.

`transcribe`: windows of `ASR_WINDOW = 30 s` with `ASR_OVERLAP = 5 s` (TS §16.4); `isCancelled()` before each window; `onWindow(done, total)` after each; then `mergeWindows` and `postProcess`.

As built (Prompt 43), with `@huggingface/transformers` 4.3.1 and the `onnxruntime-web` it resolves. Step 1 is `env.allowRemoteModels = false`, `env.useBrowserCache = false`, `env.useFSCache = false`, `env.useWasmCache = false`, and `env.fetch` replaced by a function that refuses; `env.allowLocalModels` has to be `true`, because the runtime will not start with both kinds of loading off, and a "local" file is asked of the cache first. Step 2 is `env.useCustomCache = true` with `env.customCache`. Step 3 is `env.backends.onnx.wasm.wasmPaths = { mjs, wasm }`, the two files of the `asyncify` build under `/ort/<env.backends.onnx.versions.web>/`, with `wasm.proxy = false`. Step 4 is the `device` option of `pipeline("automatic-speech-recognition", modelId, { device, dtype })`; `wasm.numThreads` is set once, before the first session, and holds for both backends. `dtype` is `{ encoder_model: "q4", decoder_model_merged: "q4f16" }`, which is how the runtime comes to ask for the two file names of the manifest. Step 5 is the call option `return_timestamps: "word"`. Each window is one call with at most 30 s of audio; the runtime's own chunking (`chunk_length_s`) is not used. `unloadModel` is `pipeline.dispose()`.

`model-cache-adapter.ts`: implements the cache interface the runtime expects. A lookup maps the requested file's base name to `paths.modelFile(modelId, name)` (files are stored under the name the runtime asks for, D-62) and returns the OPFS `File` as a `Response`. A miss throws; it never fetches (TS §16.4). A write request from the runtime is ignored. It reads final files only, never `.part` files (INV-20: a final name exists only after the hash matched).

`word-timestamps.ts`:

```ts
export function mergeWindows(windows: readonly { offsetMs: number; lengthMs: number; words: RawWord[] }[]): RawWord[];
export function postProcess(words: RawWord[], durationMs: DurMs): RawWord[];
```

`mergeWindows`: shift each window's words by its offset; in an overlap keep the instance farther from a window edge (TS §16.4). `postProcess`: clamp to `[0, duration]`; enforce `start_i >= end_(i-1)`; raise a word shorter than `ASR_MIN_WORD = 40 ms` to 40 ms where the next word allows; `confidence` = the mean token probability when available, else `1.0` (TE-2).

**Never.** Requests any URL other than `/ort/...` on the app origin. Writes to OPFS. Drops, merges or shifts words because of a pause: timestamps are the spoken times (INV-5).

**Budget.** Median `load` + `transcribe` at most 20 s for a 60 s clip on R1 (PS §20.2, E-3), which is 25 s for the reference clip (D-64); model session at most 700 MB (assumption, TE-14).

### 16.6 `render.worker.ts`

Module state: the `RenderApi`, one `RenderSession`, one `VideoSource`, the preview canvas, the last scene input, the latest `ClockSync`, a stop flag, the live-frame counter.

| Handler | Steps | Failure |
|---|---|---|
| `openSession({ clipId, clipInfo })` | `loadRender()`; open `paths.clipSource(clipId)` as a sync handle; `api.openSession(clipInfo, handle)`; `new VideoSource(session, clipInfo)` | `E_STORAGE_IO`, `E_INTERNAL` |
| `detect({ transcript, prosody })` | `api.detect(transcript, prosody, {})` | `E_INTERNAL` |
| `setScene({ transcript, events, edit, profile })` | `profile === "preview"`: `api.previewProfile()`; else `api.exportProfileFromToken(token, ENTITLEMENT_PUBLIC_KEYS, now)`. Keep `{ transcript, events, edit }`. `session.set_scene(...)` | `E_ENTITLEMENT_INVALID`, `E_INTERNAL` |
| `attachPreview({ canvas })` | Keep the canvas; `session.attach_canvas(canvas, 540, 960)` | `E_GPU_INIT` |
| `previewPlay({ clock })` | Store the clock; clear the stop flag; `runPreview(...)`; resolve when it ends. Then check the counter (D-45) | `E_DECODE_VIDEO`, `E_GPU_LOST`, `E_INTERNAL` |
| `previewClock({ clock })` | Replace the stored clock. One-way | none |
| `previewPause()` | Set the stop flag; resolve after the loop has returned | none |
| `exportClip({ exportId, entitlementToken, out48 })` | Section 16.8 | Section 16.8 |
| `closeSession()` | Stop the loop; `source.close()`; free the session; close the source handle. Idempotent | none |
| `redetectSentence`, `previewSeek` | Not implemented in V2: `E_INTERNAL` (D-32) | |

`now` for the token is `Math.floor(Date.now() / 1000)`. This file is the only worker file that imports `config/entitlement-public-key` (D-27 c).

### 16.7 `render/video-source.ts`, `render/preview-loop.ts`

```ts
export interface DemuxerHandle {                       // D-30; satisfied by RenderSession
  video_description(): Uint8Array | undefined;
  video_sample_count(): number;
  read_video_sample(index: number): { data: Uint8Array; ptsUs: number; durationUs: number; isKeyframe: boolean };
  keyframe_at_or_before(tMs: number): number;
}
```

`VideoSource`: class signature of TS §20.1 verbatim. The file also exports `liveFrames: { count: number }`: every code path that obtains a `VideoFrame` (decoder output, `new VideoFrame(canvas)`) increments it and every `close()` decrements it (INV-11, D-45).

| Method | Behaviour |
|---|---|
| Constructor | One `VideoDecoder` configured with `{ codec: info.video_codec_string, description, codedWidth, codedHeight }` (coded size = display size, swapped for `R90`/`R270`). Output frames go to a queue ordered by timestamp |
| `frameAt(t)` | TS §20.2: keep up to `PREVIEW_QUEUE = 6` frames ahead; feed while `decodeQueueSize < 3`; close frames older than `t` minus one frame; return the latest frame with `pts <= t`, or `null`. `t` before the queue head or more than 1 s past its horizon: flush and restart at `keyframe_at_or_before(t)` |
| `frameAtBlocking(t)` | Export only. Queue limit 8 (TS §31). Feeds and awaits output until a frame with `pts <= t` exists whose successor has `pts > t`, or the stream has ended. Never returns a stale frame; never skips an output time |
| `prefetch(from)` | Declared; does nothing in V2 (V4) |
| `close()` | Closes every queued frame, then the decoder |

The decoder's error callback rejects the pending `frameAtBlocking` and makes the next `frameAt` throw, both with `E_DECODE_VIDEO`. The source never closes a frame it has handed out as the "current" frame until a newer one replaces it; the loop that received it does not close it either.

`runPreview(session, source, clock, isStopped)`: TS §20.2 verbatim, per `requestAnimationFrame` in the worker. `clock()` = `audioMs + (performance.timeOrigin + performance.now() - epochMs)`. Ends when `t >= clip duration` (resolve, the preview goes to `stopped`) or when `isStopped()` (resolve). A late frame reuses the last frame and counts in `PreviewStats.late`. No resize rule in V2 (V4). The loop never drives time: the audio clock is the master (TS §20.3).

### 16.8 `render/export-loop.ts` and the `exportClip` handler

`exportClip`, around `runExport`:

1. `profile = api.exportProfileFromToken(token, ENTITLEMENT_PUBLIC_KEYS, now)`. Failure: `E_ENTITLEMENT_INVALID`; nothing has been written.
2. `session.set_scene({ ...lastSceneInput, profile })`; create an `OffscreenCanvas(profile.width, profile.height)`; `session.attach_canvas(it, w, h)`.
3. `sink = OpfsSink.open(paths.exportTmp(exportId))`.
4. `t0`; `r = runExport({ session, source, profile, out48, sink, onProgress, isCancelled })`.
5. Cancelled: `sink.abort()`; return the `CANCELLED` sentinel.
6. `sink.close()`; `move(exportTmp, exportFinal)`.
7. Restore the preview: `set_scene` with the Preview profile and `attach_canvas(previewCanvas, 540, 960)`.
8. Return `{ opfsPath: paths.exportFinal(exportId), summary: session.summary(), stageTimings: [render_encode, mux] }`.
9. Any failure after step 3: close every frame, close both encoders, `sink.abort()`, do step 7, rethrow.

`runExport`: TS §21.3 verbatim, with these fixed points:

| Point | Rule |
|---|---|
| Frame count | `N = session.frame_count()` |
| Per frame | `isCancelled()`; `t = n x 1000 / 30` (integer); `frame = await source.frameAtBlocking(t)`; `session.render_frame(frame, t)`; `vf = new VideoFrame(canvas, { timestamp: n x 1_000_000 / 30, duration: 33_333 })`; wait while `encodeQueueSize > ENCODE_QUEUE_MAX`; `encode(vf, { keyFrame: n % KEYFRAME_INTERVAL_FRAMES === 0 })`; `vf.close()` in `finally`; `onProgress(n + 1, N)` |
| Capture | Method A above unless TE-3 chose method B (texture readback); the choice is local to this file (TS §21.4) |
| Muxer | Created on the first video chunk, from `metadata.decoderConfig.description` (the `avcC`); the audio `asc` comes from the first audio chunk's `decoderConfig.description`. Chunks that arrive before the muxer exists are queued (at most the first few) |
| Video chunks | `add_video_sample(bytes, index, chunk.type === "key")` in arrival order. V2 assumes arrival order equals frame order; a mismatch surfaces as `E_MUX` (`OutOfOrder`) and is the V5 `ctts` work |
| Audio | After `venc.flush()`: pad `out48` with zeros to `N x 1600` samples (never remove one); build 2-channel `f32-planar` `AudioData` in 1,024-sample frames (mono duplicated), timestamps `i x 1024 x 1e6 / 48000`; `aenc.flush()`. Each chunk goes to `add_audio_sample(bytes, pts, duration)` with `pts = chunk.timestamp - AAC_PRIMING_SAMPLES x 1e6 / 48000` (D-33) |
| Stage timings | `render_encode` = start of the loop to the end of `aenc.flush()`; `mux` = `finalize()` plus the sink close and the move. One progress message with `stage: "mux"` is posted before `finalize()` so the main thread can move the export to `muxing` |
| Frame accounting | Every output frame `0..N` is rendered and encoded exactly once (TS §21.4). At the end the live-frame counter must be 0, else `E_INTERNAL` (D-45) |

| Failure | Code |
|---|---|
| `VideoEncoder` error callback, or `pickVideoConfig` finds no supported entry | `E_ENCODE_VIDEO`. V2 does not restart on the next ladder entry (V5) |
| `AudioEncoder` error callback | `E_ENCODE_AUDIO` |
| Decoder error | `E_DECODE_VIDEO` |
| Device lost | `E_GPU_LOST` |
| Muxer | `E_MUX` |
| Sink quota; other sink failure | `E_STORAGE_QUOTA`; `E_STORAGE_IO` |

Every failure thrown inside `exportClip` carries `stage: "render_encode"`, except a muxer, finalize or move failure, which carries `"mux"`. In the same way `extractAudio` failures carry `"probe_audio"` and the three ASR handlers `"asr"`.

**Never.** Skips or repeats an output frame because of what the audio contains; trims audio or video (INV-5, INV-10); assembles the output in memory (TS §31); keeps an `EncodedVideoChunk` after copying it into the muxer.

**Budget.** Render + encode at most 90 s and mux at most 2 s for a 60 s clip on R1 (PS §20.2, E-4), which is 112 s and 2.5 s for the reference clip (D-64).

### 16.9 `render/encoders.ts` [ONLY WebCodecs encoder configs], `render/opfs-sink.ts`

```ts
// encoders.ts: added in V2
export const ENCODE_QUEUE_MAX = 4;                                             // TS §21.3, §31
export const AAC_PRIMING_SAMPLES: number;                                      // 0 until TE-4 records the value (D-33)
export function pickVideoConfig(profile: ExportProfile): Promise<VideoEncoderConfig>;   // TS §21.1
```

`pickVideoConfig`: for each entry of `VIDEO_ENCODE_LADDER` in order, `videoConfigFor(entry, profile.width, profile.height, profile.video_bitrate)`; return the first for which `VideoEncoder.isConfigSupported` reports `supported`; none: throw `{ code: "E_ENCODE_VIDEO" }`. The V1 constants are not edited unless TE-4 reorders the ladder, in which case the new order is recorded in `docs/v2/experiments.md` and TS §21.2. `pickVideoConfig` does not read V1's `CREATOR_VIDEO_BITRATE`; that constant stays the bitrate of the capability probe (D-26).

`opfs-sink.ts`: class signature of TS §21.1 verbatim. `open` creates the file and a sync access handle. `writeAt` is a synchronous positional write; a short write or a throw is rethrown tagged `quota` or `io`. `close` flushes and closes. `abort` closes and removes the file. One handle per file (TS §31).

---

## 17. Model manager

`models/` gets verified model files into OPFS and runs nothing (TS §6). It may import `net/asset-fetch`, `persistence/opfs`, `persistence/db` (`metaGet`/`metaSet`), `wasm/load-core`, `config/model-manifest.json`, `state/model-store` (D-27 d, e), and the `AppFailure` type from `workers/protocol.ts` (D-27 f). **Never** imports `net/http` or `ui`; never starts the model; never deletes a verified file except in `clear()` or on a `modelId` change.

### 17.1 `model-manager.ts`

Signatures: TS §16.2 verbatim.

| Function | Behaviour |
|---|---|
| `inspect()` | (1) List `models/`; remove every directory whose name is not the manifest `modelId` (TS §16.5). (2) Every manifest file exists under its final name with its manifest size: `ready`. (3) Any `.part` file or any final file exists: `partial`. (4) Else `absent`. Writes the result to the store through `inspected(...)`. A storage failure reports `absent` |
| `ensureReady(onProgress, signal)` | State `ready`: return. A call while another is running returns the same promise. Otherwise: `navigator.storage.persist()` once (guarded by `META_KEYS.persistRequested`); store `start`; for each manifest file not yet final, `fetchRanged`; then verify (17.3); all final: store `hash_ok`. Resolves on `ready`; rejects with an `Error` that has a `failure: AppFailure` field (`stage: "model"`) otherwise. That is the field a use-case reads from a `WorkerCallError`, which `models/` may not import |
| `cacheInfo()` | `{ modelId, modelVersion, bytes }` where `bytes` is the sum of the sizes of the files present |
| `clear()` | Removes `models/`; store `cleared`. No V2 caller (V7, V8) |

As built (Prompt 40): the error `ensureReady` rejects with is a `ModelFailure`; an abort rejects with the abort itself and leaves the store `partial`; `ensureReady` calls `inspect()` first while the store is `unknown`, and `inspect()` writes to the store on its first call only. The `meta` key is the string `"persistRequested"` written in `model-manager.ts`: edge D-27 e reaches `persistence/db.ts` and not `persistence/schema.ts`, where `META_KEYS` is.

`onProgress` receives `{ done, total, etaSecs }`: `done` counts bytes already on disk at the start plus bytes received; `etaSecs` is `null` for the first 2 s, then remaining bytes divided by the mean rate of the last 5 s.

### 17.2 `download.ts`

```ts
export const MODEL_PART_BYTES = 8 * 1024 * 1024;       // TS §16.3
export const MODEL_RETRIES = 3;
export const MODEL_BACKOFF_MS = [1_000, 3_000, 9_000] as const;
export type DownloadDeps = { fetchAsset: typeof fetchAsset; opfs: Pick<typeof import("../persistence/opfs"), "size" | "writeAt" | "truncate">;
                             sleep: (ms: number) => Promise<void> };
export function createDownloader(deps: DownloadDeps): { fetchRanged: typeof fetchRanged };
export function fetchRanged(file: ManifestFile, o: { resumeFrom: Bytes; signal: AbortSignal; onBytes: (n: Bytes) => void }): Promise<void>;   // TS §16.2
export function verifyAndFinalize(modelId: string, file: ManifestFile, deps: VerifyDeps): Promise<void>;   // 17.3; here so download.test.ts covers INV-20
export function localName(file: ManifestFile): string;   // D-62: the last segment of file.path without the upload hash, e.g. "encoder_model.onnx"
```

As built (Prompt 39): a failure is thrown as a `ModelError`, an `Error` with a `code`, because ESLint refuses a thrown plain object; where this section says `throw { code }`, read `throw new ModelError(code)`. `VerifyDeps` is `{ opfs: Pick<..., "getFile" | "move" | "remove">; newSha256(): Promise<{ update, finalizeHex }>; pause(): Promise<void> }`; the model manager passes the stream of `loadCore()`, and `download.ts` itself does not import `wasm/load-core`. `ManifestFile` is `{ path, bytes, sha256 }`, and the part file's path is built from the second segment of `path`.

Algorithm, TS §16.3:

1. `have = resumeFrom` (the caller passes the `.part` size, 0 when absent). `have > file.bytes`: truncate the part to 0 and restart.
2. While `have < file.bytes`: request `bytes=have-(min(have + MODEL_PART_BYTES, file.bytes) - 1)`.
3. Status 206: read the whole body; `writeAt(part, have, body)`; `have += body.length`; `onBytes(body.length)`; reset the retry counter.
4. Status 200 (the host ignored Range): truncate the part to 0, stream the full body to it in 8 MiB writes, `have = file.bytes`.
5. A body longer than requested, or a 206 with zero bytes: treated as a failure of that attempt.
6. `cause: "offline"` or `"status"`: wait the next backoff and retry; after `MODEL_RETRIES` consecutive failures throw `{ code: "E_MODEL_DOWNLOAD" }`. The part file is kept.
7. `cause: "aborted"`: throw the abort; the part file is kept.
8. A `quota` storage error: throw `{ code: "E_MODEL_STORAGE" }`. An `io` error: `{ code: "E_STORAGE_IO" }`.

### 17.3 Verification and the store

`download.ts::verifyAndFinalize`, called by the model manager after a file's last byte (the store gets `last_byte` when the last file of the set starts verifying): read the part in 4 MiB slices through `getFile(...).slice()`, `update` a `Sha256Stream` from `loadCore()`, yield to the event loop between slices (D-36). Match: `move(part, final)`, where the final name is `localName(file)`. Mismatch: `remove(part)`, throw `{ code: "E_MODEL_HASH" }` (TS §16.3). A final name therefore exists only for verified bytes (INV-20).

| Outcome | Store event | State |
|---|---|---|
| Network retries exhausted | `fail_download` | `failed(E_MODEL_DOWNLOAD)`; the part file is kept, so the next `start` resumes (TS §11.2) |
| The signal aborts (V4 cancel) | `net_fail` | `partial` |
| Quota | `fail_storage` | `failed(E_MODEL_STORAGE)` |
| Hash mismatch | `hash_bad` | `failed(E_MODEL_HASH)` |
| All files verified | `hash_ok` | `ready` |

As built (Prompt 39): six of the seven files are checked while the store still says `downloading`. A wrong hash of one of them is stored as `last_byte` followed by `hash_bad`. From `verifying`, `hash_bad` is also what a storage failure goes through, and the state holds the real code. The 14 pairs are unchanged.

---

## 18. State: stores, machines, blockers

Every store is a Zustand store exporting a hook and action functions; use-cases write through actions only (TS §7). Every state change goes through `transition()` (INV-16, V1 §11.4). Stores hold no derived data another store owns (TS §12.1). `FeedLine` and `AppFailure` are type-only imports from `workers/protocol.ts` (D-27 f).

### 18.1 `machines/model-machine.ts`, `state/model-store.ts`

```ts
export type ModelStatus = "unknown" | "absent" | "partial" | "ready" | "downloading" | "verifying" | "failed";
export type ModelEvent = "inspected_absent" | "inspected_partial" | "inspected_ready" | "start" | "last_byte"
                       | "net_fail" | "fail_download" | "fail_storage" | "hash_ok" | "hash_bad" | "cleared";
export const MODEL_MACHINE: MachineDef<ModelStatus, ModelEvent>;
```

| From | Event | To (TS §12.2) |
|---|---|---|
| `unknown` | `inspected_absent` / `inspected_partial` / `inspected_ready` | `absent` / `partial` / `ready` |
| `absent`, `partial`, `failed` | `start` | `downloading` |
| `downloading` | `last_byte` | `verifying` |
| `downloading` | `net_fail` | `partial` |
| `downloading` | `fail_download`, `fail_storage` | `failed` |
| `verifying` | `hash_ok` | `ready` |
| `verifying` | `hash_bad` | `failed` |
| `ready`, `partial` | `cleared` | `absent` |

`verifying` to `downloading` is absent on purpose (TS §12.3). Store: `{ status, done: Bytes, total: Bytes, etaSecs: number | null, error?: ErrorCode }` with actions `inspected(r)`, `start()`, `progress(p)`, `lastByte()`, `netFail()`, `fail(code)`, `hashOk()`, `cleared()`. Written only by `models/model-manager.ts` (TS §12.1).

### 18.2 `machines/clip-machine.ts`, `state/clip-store.ts` (D-18)

```ts
export type ClipStatus = "idle" | "importing" | "accepted" | "processing" | "ready" | "updating" | "rejected" | "failed";
export type ClipEvent = "import" | "accept" | "reject" | "fail" | "run" | "done" | "no_speech" | "cancel"
                      | "edit_start" | "edit_done" | "retry" | "reset" | "restore";
```

| From | Event | To (TS §12.2) |
|---|---|---|
| `idle` | `import` | `importing` |
| `importing` | `accept` / `reject` / `fail` | `accepted` / `rejected` / `failed` |
| `accepted` | `run` | `processing` |
| `processing` | `done` / `no_speech` / `fail` / `cancel` | `ready` / `rejected` / `failed` / `accepted` |
| `ready` | `edit_start` | `updating` |
| `updating` | `edit_done` | `ready` |
| `failed` | `retry` | `processing` |
| `rejected`, `failed`, `ready`, `accepted` | `reset` | `idle` |
| `idle` | `restore` | `ready` |

The whole table exists now; V2 fires `import`, `accept`, `reject`, `fail`, `run`, `done`, `no_speech`, `reset`. Absent pairs include `rejected` to `processing`, `importing` to `processing`, `processing` or `ready` to `importing` (TS §12.3).

```ts
export type ClipState = {
  status: ClipStatus; clipId?: ClipId; source?: ClipSource; clipInfo?: ClipInfo;
  stage?: PipelineStage; waitingModel: boolean;
  transcript?: Transcript; prosody?: Prosody; events: readonly DetectedEvent[]; edit: EditState;
  feed: readonly FeedLine[]; reject?: RejectReason; failure?: AppFailure;
  out48: Float32Array | null;                                   // D-51
};
```

Actions: `begin(clipId, source)`, `accepted(info)`, `rejected(reason)`, `failed(failure)`, `processing(stage, waitingModel)`, `pushFeed(line)`, `ready({ transcript, prosody, events, out48 })`, `noSpeech()`, `reset()` (which also drops `out48`). `edit` is `EditState` default and never changes in V2. Feed lines are appended, never dropped (TS §14.3).

### 18.3 `state/preview-store.ts` (D-19)

`PreviewStatus = "detached" | "stopped" | "playing" | "paused" | "seeking" | "locked"`. Table from TS §12.2: `detached` `attach` `stopped`; `stopped`, `paused` `play` `playing`; `playing` `pause` `paused`; `playing` `end` `stopped`; `playing`, `paused` `seek` `seeking`; `seeking` `seek_done_playing` / `seek_done_paused` `playing` / `paused`; any state `lock` `locked`; `locked` `unlock` `paused`; any state `detach` `detached`. `detached` to `playing` is absent (TS §12.3). Store: `{ status, playedOnce: boolean }` with actions `attached()`, `play()`, `pause()`, `ended()`, `lock()`, `unlock()`, `detached()`. V2 never fires the seek events.

### 18.4 `machines/export-machine.ts`, `state/export-store.ts` (D-18)

```ts
export type ExportStatus = "idle" | "gating" | "rendering" | "muxing" | "saving" | "done" | "cancelled" | "failed";
export type ExportEvent = "start" | "blocked" | "clear" | "cache_hit" | "encoded" | "finalized" | "saved"
                        | "cancel" | "cancel_done" | "fail" | "retry" | "reset";
```

| From | Event | To (TS §12.2) |
|---|---|---|
| `idle` | `start` | `gating` |
| `gating` | `blocked` / `clear` / `cache_hit` | `idle` / `rendering` / `saving` |
| `rendering` | `encoded` | `muxing` |
| `muxing` | `finalized` | `saving` |
| `saving` | `saved` | `done` |
| `rendering`, `muxing` | `cancel` | `cancelled` |
| `cancelled` | `cancel_done` | `idle` |
| `rendering`, `muxing`, `saving` | `fail` | `failed` |
| `failed` | `retry` | `gating` |
| `done`, `failed` | `reset` | `idle` |

`rendering` to `rendering` and `done` to `rendering` are absent (TS §12.3). Store: `{ status, exportId?: ExportId, done: number, total: number, startedAt?: number, failure?: AppFailure, unavailable: boolean }` with actions `start(exportId)`, `blocked()`, `clear()`, `progress(done, total)`, `encoded()`, `finalized()`, `saved()`, `fail(failure)`, `reset()`, `setUnavailable(flag)`. V2 never fires `cache_hit`, `cancel`, `cancel_done`, `retry`.

### 18.5 `state/blockers.ts` [ONLY blocker codes and conditions] (D-20)

```ts
export const BLOCKER_CODES = ["B_UNSUPPORTED", "B_PIPELINE_BUSY", "B_EXPORT_IN_PROGRESS", "B_PIPELINE_NOT_READY",
                              "B_STORAGE_LOW", "B_NOT_SIGNED_IN", "B_ENTITLEMENT_EXPIRED", "B_NO_FREE_EXPORTS"] as const;
export type BlockerCode = (typeof BLOCKER_CODES)[number];
export function forImport(): BlockerCode | null;
export function forExport(): BlockerCode | null;
```

Each returns the first blocker in the order of TS §12.5:

| Function | Order in V2 | Missing until |
|---|---|---|
| `forImport` | `B_UNSUPPORTED` (capability not `supported`); `B_PIPELINE_BUSY` (clip in `importing`, `processing`, `updating`); `B_EXPORT_IN_PROGRESS` (export in `gating`, `rendering`, `muxing`, `saving`) | Complete |
| `forExport` | `B_EXPORT_IN_PROGRESS`; `B_PIPELINE_NOT_READY` (clip not `ready`) | `B_STORAGE_LOW` (V5), `B_NOT_SIGNED_IN`, `B_ENTITLEMENT_EXPIRED`, `B_NO_FREE_EXPORTS` (V5, V6) |

`forExport` is evaluated while the export is still `idle`. `forCheckout` and `forClearLocalData` are V6 and V7. This file reads stores with `getState()` and has no side effect. `blockers.test.ts` is V5.

---

## 19. Use-cases

One function per user action (TS §2). They may import `state`, `workers/pool`, `persistence`, `models`, `net/api-client`, `analytics/client`, and `import-clip.ts` alone may import `net/asset-fetch` (D-27 a). `WorkerCallError` is imported from `workers/pool` (D-32) and `AppFailure` as a type (D-27 f). **Never** `fetch`, WebCodecs, WebGPU, a WASM export, a component or another use-case. A `WorkerCallError` is caught at the top of each use-case and stored as the failure; nothing is swallowed.

### 19.1 `start-app.ts`: one step added

Step 8, at the position V1 §11.10 marked (after step 7, supported browsers only): `void modelManager.inspect()`. It is not awaited and a failure leaves the model `absent`. V6 inserts `restoreSession()` between 6 and 7; V7 appends `restoreClip()`.

### 19.2 `import-clip.ts` (C-2)

```ts
export function importClip(file: File, source: ClipSource): Promise<void>;
export function importSampleClip(): Promise<void>;                      // D-48
export function dismissClip(): Promise<void>;                           // D-58: "start over"
export function newClipId(): ClipId;                                    // crypto.randomUUID(), the one cast of D-59
```

`importClip`:

| # | Step | On failure |
|---|---|---|
| 1 | `blockers.forImport()` non-null: return. The component already shows that blocker's copy | - |
| 2 | Clip not `idle` (it is `ready`, `rejected`, `failed` or `accepted`): run the `dismissClip` steps | - |
| 3 | `clipId = newClipId()`; `clipStore.begin(clipId, source)` (idle to importing) | - |
| 4 | **Stand-in until V7 (D-40):** remove every entry of `clips/` | Ignored; step 6 will report quota if it matters |
| 5 | `timer.start("probe_audio")` | - |
| 6 | `r = pool.media.importAndProbe({ clipId, file })` | `clipStore.failed(failure)`; remove `clipDir` |
| 7 | `r.rejected`: `clipStore.rejected(reason)`; remove `clipDir`; return. No event in V2 (`clip_rejected` is V3) | - |
| 8 | `clipStore.accepted(r.ok)`; `track clip_accepted { duration_bucket, orientation, source }` | - |
| 9 | `void runPipeline(clipId)` | - |

`duration_bucket`: under 30,000 ms `lt30`; under 60,000 ms `lt60`; else `lte90`. The `File` object's name is never read, stored or sent (P-11). Step 9 is the import of `run-pipeline.ts` that TS C-2 requires (D-58).

`dismissClip`: (1) remember whether a render session was opened (status `ready`, `updating`, or `failed` after stage `detect_scene`); (2) `clipStore.reset()`, which unmounts the player and detaches the preview; (3) `pool.render.closeSession()` when (1) was true; (4) remove `clipDir`. Failures in (3) and (4) are stored nowhere and block nothing: the next import removes the directory again.

`importSampleClip`: `fetchAsset(SAMPLE_CLIP_PATH, { signal })`; on `ok`, `new File([await response.blob()], "sample")`, then `importClip(file, "sample")`. On failure the clip store goes `idle` to `importing` to `failed` with `{ code: "E_NET_OFFLINE", stage: "import", retryable: true }` for `offline`, and `{ code: "E_INTERNAL", stage: "import", retryable: false }` for any other cause.

### 19.3 `run-pipeline.ts` (C-3, C-4)

```ts
export function runPipeline(clipId: ClipId): Promise<void>;
```

| # | Step (TS C-4 letter) | Store and events | On failure |
|---|---|---|---|
| 1 | `clipStore.processing("probe_audio", false)` (accepted to processing) | | |
| 2 | Start in parallel: `modelReady = modelManager.ensureReady(onProgress, signal)` (C-3); `signal` belongs to an `AbortController` that V2 never aborts (V4 cancel) | `waitingModel = true` while the model store is not `ready` | Handled at step 5 |
| 3 | (A) `{ pcm48, pcm16 } = pool.media.extractAudio({ clipId })` | `stage_timing { stage: "probe_audio", duration_ms }` (from the timer started in `importClip`, D-35) | `failed` |
| 4 | **Stand-in until V3 (D-22):** `out48 = pcm48` | No "Cleaning voice" line, no `audio_chain` timing | |
| 5 | `await modelReady` | A download happened: `model_download { outcome, duration_ms, resumed }` with `outcome` `ok`, `failed` (`E_MODEL_DOWNLOAD`, `E_MODEL_STORAGE`) or `hash_mismatch`; `resumed` = bytes were on disk at the start | `failed(failure)`; `pcm16` is dropped |
| 6 | (B1) `clipStore.processing("asr", false)`; `{ modelId } = modelManager.cacheInfo()`; `{ backend } = pool.asr.load({ modelId, backend: "webgpu" })`; `r = pool.asr.transcribe({ pcm16 }, { transfer: [pcm16.buffer], onProgress })` | Feed line `transcribing` from the first progress message; `stage_timing { stage: "asr", duration_ms, asr_backend }` | `failed`; step 7 still runs |
| 7 | (C) `await pool.asr.unload()`, always, before step 9 (INV-12) | | `failed` |
| 8 | `r.rejected`: `clipStore.noSpeech()` (processing to rejected, `REJECT_NO_SPEECH`); remove `clipDir`; return | No event in V2 | |
| 9 | **Stand-in until V3 (D-22):** (D) `prosody = { per_word: words.map(() => ({ energy_z: 0, pitch_z: 0 })) }` | | |
| 10 | (E) `clipStore.processing("detect_scene", false)`; `pool.render.openSession({ clipId, clipInfo })`; `events = pool.render.detect({ transcript, prosody })`; `pool.render.setScene({ transcript, events, edit, profile: "preview" })` | One feed line `{ kind: "event_found", eventKind, display }` per event; `stage_timing { stage: "detect_scene", duration_ms }` | `failed` |
| 11 | (F) `opfs.writeAudio(clipId, out48)`. `transcriptRepo.put` and `editsRepo.put` are V3 | | `failed(E_STORAGE_QUOTA or E_STORAGE_IO)` |
| 12 | `clipStore.ready({ transcript, prosody, events, out48 })` (processing to ready) | `pipeline_done { total_ms, n_number: events.length, n_list: 0, n_from_to: 0, n_keyword: 0 }` | |

`display` for a feed line is the event's `Quantity.display`; it stays on the device and is never put in an analytics event (TS §14.1). `total_ms` runs from the start of `importClip` step 5 to step 12. Every `*_ms` value is capped at 3,600,000 before `track` (TS §22.6). No step calls the API (INV-3). No `client_error` is sent for a pipeline failure in V2 (BP Appendix C: V4). On any `failed`, a still-open ASR session is unloaded and the render session closed before returning.

### 19.4 `control-preview.ts` (C-5)

```ts
export function attach(canvas: HTMLCanvasElement, out48: Float32Array): Promise<void>;   // TS §20.1
export function play(): Promise<void>;
export function pause(): Promise<void>;
export function detach(): Promise<void>;                 // D-58
export function lockForExport(): Promise<void>;          // D-58: pause if playing, then any state to locked
export function unlockAfterExport(): void;               // locked to paused
```

| Function | Steps |
|---|---|
| `attach` | Create one `AudioContext` and an `AudioBuffer` (1 channel, 48,000 Hz) from `out48`; `pool.render.attachPreview({ canvas: canvas.transferControlToOffscreen() }, { transfer })`; `previewStore.attached()` |
| `play` | Status `stopped`: offset 0; `paused`: the stored offset. `ctx.resume()`; new `AudioBufferSourceNode`, `start(0, offset)`; `previewStore.play()`; start a 250 ms interval that calls `pool.render.notify("previewClock", { clock })` (TS §20.2); `pool.render.previewPlay({ clock })` and, when it resolves without a pause, `previewStore.ended()`, stop the node, clear the interval. First play of a clip: `track preview_played`, once |
| `pause` | Store the offset `(ctx.currentTime - startCtxTime) x 1000 + startOffset`; stop the node; clear the interval; `pool.render.previewPause()`; `previewStore.pause()` |
| `detach` | `pause` if playing; close the `AudioContext`; `previewStore.detached()` |
| `lockForExport`, `unlockAfterExport` | `pause` if playing, then `previewStore.lock()`; `previewStore.unlock()` (TS §12.2, §12.4) |

`clock = { audioMs, epochMs: performance.timeOrigin + performance.now() }`. The first `play` follows a user gesture, which unlocks the `AudioContext` (TS §20.3). `seek` is V4. A failure of `previewPlay` stores the failure on the clip store and detaches.

### 19.5 `start-export.ts` (C-9, C-10; no gate yet)

```ts
export function startExport(clipId: ClipId): Promise<void>;
export function newExportId(): ExportId;                                  // UUIDv7 (D-43)
```

| # | Step | On failure |
|---|---|---|
| 1 | `blockers.forExport()` non-null: return (the component shows its copy). The export stays `idle` | |
| 2 | **Stand-in until V5/V6 (D-23):** `rec = entitlementRepo.get()`; `undefined`: `exportStore.setUnavailable(true)`; return | |
| 3 | Export status `done` or `failed`: `exportStore.reset()` first. `exportId = newExportId()`; `exportStore.start(exportId)` (idle to gating); `exportStore.clear()` (gating to rendering) | |
| 4 | `profile = planOf(rec.token)` (D-52); `track export_started { profile }` | An unreadable payload counts as `free` |
| 5 | `await controlPreview.lockForExport()` (TS C-9; the import TS C-9 requires, D-58) | |
| 6 | **Stand-in until V7 (D-40):** remove every entry of `exports/` | Ignored |
| 7 | `out48 = opfs.readAudio(clipId)` (D-51) | `fail(E_STORAGE_IO)` |
| 8 | `r = pool.render.exportClip({ exportId, entitlementToken: rec.token, out48 }, { transfer: [out48.buffer], onProgress })`. Progress with `stage: "render_encode"` updates `done/total`; the first progress with `stage: "mux"` fires `encoded()` (rendering to muxing) | `fail(failure)`; `track export_failed { error_code, stage }`; `controlPreview.unlockAfterExport()` |
| 9 | `exportStore.finalized()` (muxing to saving); one `stage_timing` per entry of `r.stageTimings` | |
| 10 | Deliver (C-10): `file = opfs.getFile(r.opfsPath)`; object URL; click a temporary anchor with `download = "offcut-<yyyymmdd-hhmm>.mp4"` in local time; revoke the URL after 60 s (TS §31). The name is never derived from the source (TS §21.4) | `fail(E_STORAGE_IO)` |
| 11 | `exportStore.saved()` (saving to done); `controlPreview.unlockAfterExport()`; `track export_done { total_ms, profile, word_edits: 0, events_kept: <enabled events>, events_disabled: 0, style: "clean", crop_adjusted: false, from_cache: false }` | |

`planOf(token)`: base64url-decode the first segment, parse JSON, return `"creator"` when `plan === "creator"`, else `"free"`. It is not a verification and nothing but the two analytics props reads it (INV-9 is enforced in the worker). No receipt, no render cache, no summary panel in V2.

`newExportId()`: 48 bits of `Date.now()`, version nibble 7, 12 random bits, variant bits `10`, 62 random bits, formatted as a UUID string; the cast to `ExportId` is the one line D-59 allows in this file. Randomness comes from `crypto.getRandomValues`.

---

## 20. UI, copy, styles

### 20.1 Pages

| File | Reads | Renders | Calls |
|---|---|---|---|
| `ui/pages/LandingPage.tsx` (CHANGED) | capability store, clip store | As V1. When the clip status leaves `idle`, `navigate(appPath)`, with `appPath` a prop from `routes.tsx` (D-48) | none directly |
| `ui/pages/EditorPage.tsx` (CHANGED) | clip store, model store, export store | By clip status, table below. The V1 placeholder body and its `NotifyMeForm` are removed | `dismissClip()` |

`EditorPage` body by clip status:

| Status | Shows |
|---|---|
| `idle` | `DropZone` |
| `importing`, `accepted`, `processing` | `ProcessingFeed`; above it `ModelDownloadPanel` while `waitingModel` and the model is `downloading`, `verifying` or `failed` |
| `ready`, `updating` | `PreviewPlayer`, `ExportButton`, `ExportProgress` |
| `rejected` | `messages.editor.rejectedStub` and a "start over" button |
| `failed` | `messages.errors[code]` when the failure code is `E_MODEL_DOWNLOAD`, `E_MODEL_HASH` or `E_MODEL_STORAGE`; otherwise `messages.editor.failedStub`. A "start over" button |

"Start over" calls `dismissClip()`: components read stores and never write them (TS §2). The stub shows no code and no file detail.

### 20.2 Components

| File | Reads | Behaviour | Calls |
|---|---|---|---|
| `DropZone.tsx` (CHANGED) | clip store, `blockers.forImport()` | A drop target and the sample button, no longer `aria-disabled`. With a blocker: shows `messages.blockers[code]` and ignores the drop. Accepts the first dropped file only. Shows `messages.dropZone.prompt(limits)` (one speaker and English are stated here, PS §9.4) | `importClip(file, "user")`, `importSampleClip()` |
| `ModelDownloadPanel.tsx` | model store | J4 copy with the size from the manifest rounded to 10 MB (TS §16.1); a progress bar; time remaining when `etaSecs` is known; a `verifying` line | none. Retry through `runPipeline` (TS §13.3) is V4; in V2 a model failure ends the clip as `failed` and the page offers "start over" |
| `ProcessingFeed.tsx` | clip store `feed` | One line per `FeedLine`, through `messages.feed`; a `switch` over `kind` and over `eventKind` with `assertNever`. Lines appear in arrival order and are never invented (PS §10 J6) | none |
| `PreviewPlayer.tsx` | preview store, clip store | A `<canvas>` of 540x960 CSS-scaled to fit; play / pause / replay buttons. On mount: `attach(canvas, out48)` with the buffer read from the clip store (D-51). On unmount: `detach()`. On `visibilitychange` to hidden while playing: `pause()` (TS §12.4). When the preview store is `locked` the buttons are disabled | `controlPreview.attach`, `play`, `pause`, `detach` |
| `ExportButton.tsx` | export store, `blockers.forExport()` | Idle: the export label. `unavailable`: `messages.export.unavailable`. With a blocker: disabled, with that blocker's copy. No counter (V6) | `startExport(clipId)` |
| `ExportProgress.tsx` | export store | While `rendering`, `muxing`, `saving`: the stage name, `done / total` as a bar, elapsed seconds, and `messages.export.renderingLocally`. `done`: `messages.export.done`. `failed`: `messages.editor.failedStub` | none |

No component contains a literal sentence (`react/jsx-no-literals`), a number that belongs to a limit (INV-15) or a call into `workers`, `net`, `persistence`, `wasm`, `models` or `analytics`. No upgrade prompt exists (INV-14).

### 20.3 `copy/messages.ts` additions

| Section | Keys added in V2 | Wording source |
|---|---|---|
| `modelDownload` | `body(sizeMb)`, `progress(etaSecs)`, `verifying` | PS §10 J4: "One-time setup: downloading the speech model (about {sizeMb} MB). It stays in your browser, and your video is not uploaded." |
| `feed` | `transcribing`, `cleaningVoice`, `eventFound: Record<EventKind, (display: string) => string>` | PS §10 J6: "Transcribing…", "Cleaning voice", "Found: {display}". `cleaningVoice` and three of the four `eventFound` entries are written now because the switches are exhaustive; V2 never shows them |
| `preview` | `play`, `pause`, `replay` | - |
| `export` | `button`, `unavailable`, `stage: Record<"rendering" \| "muxing" \| "saving", string>`, `elapsed(seconds)`, `renderingLocally`, `done` | PS §10 J10: "Everything is rendering on your computer." |
| `editor` | `rejectedStub`, `failedStub`, `startOver` | Plain: the clip could not be used; something went wrong on this device |
| `blockers` | `B_UNSUPPORTED`, `B_PIPELINE_BUSY`, `B_EXPORT_IN_PROGRESS`, `B_PIPELINE_NOT_READY` | TS §12.5 conditions |
| `errors` | `E_MODEL_DOWNLOAD`, `E_MODEL_HASH`, `E_MODEL_STORAGE`, each `{ title, body, action }` | TS §11.2 recovery column |

Removed: `dropZone.notReady` (D-56). Rules that already apply (V1 §11.11): no typed-in digit (sizes and seconds are parameters), none of the phrases "never leaves", "GDPR", "DPDP", "CCPA", "SOC 2", "compliant" (P-12). `export.unavailable` says that exporting needs an account and that accounts are not open yet; it makes no promise about a date.

### 20.4 Styles

New class names only, in the two existing CSS Modules files, using the tokens of `tokens.css`. The progress bars set their width through a CSS custom property written with `element.style.setProperty`, which the CSP allows; no `style` attribute string and no runtime CSS-in-JS (CSP `style-src 'self'`, V1 §11.14).

---

## 21. Code generation

Nothing changes. `scripts/gen-types.sh`, the two generator tests and `web/src/gen/domain.ts` / `api.ts` are untouched: V2 adds no shared type (V1 §15.2: "`offcut-types/*`: Nothing"). `EntitlementClaims`, `RawWord`, `Token`, the display-list types and `BLOCKER_CODES` are deliberately not generated (D-20, D-29, D-31, D-42). `scripts/check-gen-clean.sh` must still report no diff after every V2 commit.

---

## 22. Root configuration, scripts, CI, hosting

Only what changes.

### 22.1 Cargo workspace

| File | Change |
|---|---|
| `Cargo.toml` | Members `crates/offcut-mp4`, `-dsp`, `-text`, `-detect`, `-scene`, `-render`, `-entitlement`, `-wasm-render` (V1 §15.2). `[workspace.dependencies]` gains the crates of section 3.2. Every new crate has `[lints] workspace = true`. `[profile.release]` is unchanged; `panic = "abort"` stays unset (V1 §12.1) |
| `deny.toml` | License allowlist entries required by the new dependency trees, decided on the first `cargo deny check`. `[bans]`: the `offcut-text` entry gains the wrapper `offcut-scene` (D-28). A third-party crate that depends directly on a banned crate is added as a wrapper of that entry when `cargo deny check` names it (for example `vello` for `wgpu`, `serde-wasm-bindgen` for `js-sys` and `wasm-bindgen`), as V1 did for `reqwest`, `uuid` and `ring`; each one is recorded in `v2changelog.md`. No workspace crate other than the binding and renderer crates becomes a wrapper of a browser, GPU or randomness crate; `offcut-api` enters no media-crate list; `offcut-entitlement` is not added to the wrappers of `rand` or `getrandom` |
| `clippy.toml` | Unchanged |

`offcut-wasm-core/Cargo.toml` and `offcut-wasm-render/Cargo.toml` keep `crate-type = ["cdylib"]`. Dependency edges must match TS §7 plus D-28: `wasm-core` to `mp4`, `text`, `dsp`; `wasm-render` to `render`, `detect`, `mp4`, `entitlement` (it reaches `scene` through `render`, D-61); `render` to `scene`; `detect` to `text`; `scene` to `text`; all to `types`.

### 22.2 Root `package.json` scripts added (TS §33)

| Script | Does |
|---|---|
| `e2e:media` | Playwright project `media` against `vite preview`: `model-download`, `pipeline-preview`, `export-creator`; each export is checked by `verify/verify_mp4.py` from inside the suite |
| `e2e:device` | Projects `non-media` and `media`, headed, on this machine (R1) |
| `bench:device` | Playwright project `bench`, whose only test file is `bench/device-bench.ts` (D-63); writes `bench/results/<BENCH_DEVICE>-<yyyy-mm-dd>.json` |
| `verify` | `python verify/verify_mp4.py` with the arguments passed through |

`fixtures` and `corpus` are added by V3. Also changed: `check` runs a second `tsc --noEmit`, for `web/tests-e2e/tsconfig.json`, and the root package gains the dev-dependency `@playwright/test` at the version `web` pins (D-63).

### 22.3 Web configuration files

| File | Change |
|---|---|
| `web/package.json` | Dependency `@huggingface/transformers`, pinned (4.3.1 since Prompt 43; the root `package.json` removes two of its dependencies, `onnxruntime-node` and `sharp`, with `pnpm.overrides`: neither is used in a browser, and the types of `sharp` bring Node's globals into the app's type-check). No other runtime dependency. Dev-dependency `@types/node`, at the Node major of `engines` (D-63) |
| `web/tsconfig.json`, `web/tests-e2e/tsconfig.json` | D-63: the `include` of the first is `src` alone. The second is new: it extends the first, adds `node` to `types` and includes `tests-e2e`, `playwright.config.ts`, `vite.config.ts`, `vitest.config.ts` and `../bench`. App code therefore never sees a Node global |
| `web/vite.config.ts` | A build-start step copies the ONNX Runtime `.wasm` (and any `.mjs` loader the runtime needs) from the pinned package into `web/public/ort/<version>/` and removes other version directories (D-37). The version string is read from the package's own `package.json`. Nothing else changes: `assetsInlineLimit: 0` and `worker.format: "es"` are already set (V1 §12.3). As built (Prompt 43): two files are copied, `ort-wasm-simd-threaded.asyncify.mjs` and `.wasm`, the ones the WebGPU entry of the runtime loads; and one more thing does change, `resolve.conditions` gains `onnxruntime-web-use-extern-wasm`, without which the bundler follows a reference inside ONNX Runtime and ships a second copy of the 27 MB module under `assets/` |
| `web/playwright.config.ts` | Project `media`: `testMatch` of the three media suites, `channel: "chrome"`, `timeout` 300 s, launch arguments for WebGPU as recorded by TE-10. `globalSetup` fills `fixtures/.cache/` (section 23.4). Project `non-media` keeps `landing.spec.ts`. Project `bench`: `testDir` `../bench`, the one file `device-bench.ts`, headed (D-63) |
| `web/eslint.config.js` | The six edges of D-27 and the two of D-58, none of which V1's matrix has, and the seven cast overrides of D-59. No existing rule is loosened for any other file. The `tests-e2e` block needs no change: its files find the new `tsconfig.json` by themselves |
| `.gitignore` | `web/public/ort/`, `fixtures/.cache/` |

### 22.4 Scripts

| Script | Contract change |
|---|---|
| `scripts/build-wasm.sh` | `BUNDLES` gains `offcut-wasm-render:render`, output `web/src/wasm/pkg/render/`. If the pinned `wgpu` needs a build `cfg` for its WebGPU backend, it is set for this bundle only, here, and recorded in `v2changelog.md` |
| `scripts/check-copy-codes.mjs` | Also reads `BLOCKER_CODES` from `web/src/state/blockers.ts` (a plain `as const` array it can parse) and requires `messages.blockers[code]` or a pending entry. `COPY_PENDING` after V2: every `REJECT_*` (V3); every `E_*` except the six of V1 §11.11 and the three of D-57; `B_STORAGE_LOW`, `B_NOT_SIGNED_IN`, `B_ENTITLEMENT_EXPIRED`, `B_NO_FREE_EXPORTS` |
| `scripts/check-hosts.mjs` | D-38: an entry (`{ prefix, reason, exact? }` as built) may also carry `chunk`; with it, the literal is allowed only in output files whose path matches. Entries with `chunk` cover the `asr.worker` chunk, `ort/<version>/` and `offcut_render_bg-*.wasm`. The script still fails on the same literal elsewhere, on any host in the main chunk, and on a CSP host mismatch |
| `scripts/check-file-tree.mjs` | Allowed crate pair `offcut-scene` to `offcut-text` (D-28). `PURE_CRATES` gains `offcut-mp4`, `-dsp`, `-text`, `-detect`, `-scene` and `-entitlement`, so the `cargo tree` check V1 runs on its two pure crates covers the new ones; if a third-party dependency brings in a listed crate through a default feature, turn that feature off. The tree it parses comes from TS §5, which S1 updates with `mux_boxes.rs`, `web/tests-e2e/tsconfig.json` and `docs/v2/*` |
| `scripts/check-external-facts.mjs` | The TE-10 line gets its date |
| `scripts/upload-assets.sh` | D-62. The folder argument may be `media`, `models` or `models/<modelId>` (lower-case letters, digits and hyphens); the content-type table gains a row for any extension in the model's file list that it lacks. Names keep their hash segment. First used for `models/asr-en-v1/*` and `media/<sample>.mp4`; the printed paths go into `model-manifest.json` and `SAMPLE_CLIP_PATH` |

### 22.5 CI (`ci.yml` and `e2e-media.yml`)

Steps 1-7 are V1 §12.7, with one addition: step 6 also runs `tsc --noEmit -p tests-e2e/tsconfig.json`, the second program of D-63 (added in Prompt 32). The new Rust tests run inside step 4 automatically.

As built, `ci.yml` has three jobs: `ci` (steps 1-9), `deploy-api` and `deploy-web`. The step numbers below are those of TS §33, and each step lands in the job that already holds its neighbours. Three things change in the existing jobs. The `web/dist` upload is new and runs on every event, pull requests included. The `wasm-pkg` artifact that `ci` hands to `deploy-web` on `main` keeps its path and now carries both bundles. The one step that runs `vercel build` and `vercel deploy` is split in two, so that the guard of step 14 sits between them. `wait-for-version.mjs` and `check-headers.mjs` are unchanged.

8. `vite build` with `VITE_ASSET_BASE_URL` and `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` (the public key of the fixed test seed; a public value, written in the workflow file); `check-hosts.mjs`; upload `web/dist` as an artifact.
9. Playwright `non-media` against `vite preview`: `landing.spec.ts`.
10. Call `e2e-media.yml` (D-44). Its job, on `windows-latest`:
    1. Checkout; Node, pnpm, `pnpm install --frozen-lockfile`; Playwright's Chrome channel; Python with `verify/requirements.txt`; the `ffmpeg` CLI.
    2. Download the `web/dist` artifact.
    3. Restore `fixtures/.cache/` from the Actions cache, keyed by the SHA-256 of `model-manifest.json`.
    4. `pnpm e2e:media` against `vite preview`.
    5. On failure upload the Playwright report. Never upload an exported MP4 or a trace containing transcript text.

On `main` only, after step 10 passes:

11. Call `deploy-api.yml`.
12. Wait until `/api/v1/healthz` reports the commit (V1 step 11).
13. `vercel build --prod` **without** `VITE_ENTITLEMENT_TEST_PUBLIC_KEY`.
14. **Test-key guard:** search the build output for the base64 test public key; any hit fails the job before deploy (D-24).
15. `vercel deploy --prebuilt --prod`; header check; `@smoke`.

**Ordering rules and why.** The media job runs before any deploy, as TS §33 step 10. The guard sits between build and deploy so a wrong variable can never reach production. Server first, then web (TS §33). If TE-10 shows the Windows minutes do not fit, step 10 runs on `main` and on a manual trigger only, and `pnpm e2e:device` covers pull requests locally (TS §33, BP Appendix D).

### 22.6 Hosting

`web/vercel.json` and `render.yaml` are not edited: `/ort/*` caching and the asset origin are already there (V1 §12.4). On the asset host: the model files and the sample clip are uploaded with `Cache-Control: public, max-age=31536000, immutable` under content-hashed names; CORS already serves the app origin (precondition 1) and gains `http://localhost:4173` for local real-asset runs (D-41). No new host, no new header, no function.

---

## 23. Tests, case by case

### 23.1 What runs where

| Level | Files in V2 | Runs in |
|---|---|---|
| Rust unit (inline) | Sections 6.9, 7, 8.5, 9.4, 10.5, 11.8 | `cargo test --workspace`, CI step 4 |
| Rust integration | `offcut-mp4/tests/mux_roundtrip.rs` | Same |
| TypeScript unit | `models/download.test.ts` (plus V1's two) | `vitest run`, CI step 7 |
| E2E non-media | `landing.spec.ts` | CI step 9, Linux |
| E2E media | `model-download.spec.ts`, `pipeline-preview.spec.ts`, `export-creator.spec.ts` | CI step 10 on Windows; `pnpm e2e:device` on R1 |
| Independent verification | `verify/verify_mp4.py` checks 1-6 | Called by `export-creator.spec.ts` and by hand |
| Device benchmark | `bench/device-bench.ts` | By hand on R1 |

Test files BP assigns to later versions are not written now: `rpc.test.ts`, the machine tests, `run-pipeline.test.ts` (V4); `start-export.test.ts`, `blockers.test.ts` (V5); `token_roundtrip.rs`, `profile.rs` (V6).

### 23.2 `crates/offcut-mp4/tests/mux_roundtrip.rs`

Fixtures are built in the test: an arbitrary non-empty `avcc`, a 2-byte AAC-LC 48 kHz stereo `asc`, and payloads of pseudo-random bytes from a fixed seed (the muxer and demuxer never parse payloads). Sink: `MemSink`. Reader: `MemReader`.

| Case | Expect |
|---|---|
| 90 video samples (keyframe every 60) and 141 audio samples of 1,024 ticks, muxed then demuxed | Same counts; every sample's bytes equal; video `pts` of sample `n` is `n x 1,000,000 / 30` (floor); keyframe flags equal; audio durations 21,333 us |
| The same file probed | H.264 with the given `avcC` string; 30,000 `FpsMilli`; not VFR; one video and one audio track; AAC, 48,000 Hz, 2 channels; `Mp4` |
| Box order | `ftyp`, then `moov`, then `mdat` (faststart) |
| `finalize()` return value | Equals the sink length |
| First audio sample with `pts = -21,333` us | `elst.media_time` is 1,024 (rounded to the nearest tick); the demuxed audio sample 0 reports `pts = -21,334` and sample 1 reports 0 (D-33). As built (Prompt 41): 1,024 ticks before 0 are 21,333.3 microseconds, and the demuxer rounds a time down (section 6.5) |
| First audio `pts = 0` | No `edts` box in the audio track |
| 2,700 video samples of 3,000 bytes and 4,220 audio samples | `finalize` succeeds; `moov` ends before the `mdat` header; duration 90,000 ms |
| `add_video_sample` with frame 1 first; frame 0 twice | `OutOfOrder` |
| First video sample not a keyframe | `BadConfig` |
| Empty `avcc`; sample rate 44,100; 1 channel | `BadConfig` |
| proptest: 1 to 300 frames, payload sizes 1 to 2,000, random keyframe pattern starting with a keyframe | Round trip holds |
| Video metadata | Width, height as given; identity matrix; no `udta` box anywhere |

### 23.3 `web/src/models/download.test.ts`

Uses `createDownloader` with a fake `fetchAsset` backed by a byte array, a fake OPFS (a `Map` of byte arrays) and a fake `sleep` that records its arguments.

| Case | Expect |
|---|---|
| 20 MiB file, empty part | Three requests with `Range` `0-8388607`, `8388608-16777215`, `16777216-20971519`; the part equals the source; `onBytes` sums to 20 MiB |
| `resumeFrom` 8 MiB with those bytes on disk | The first request starts at 8388608; two requests in total |
| The host answers 200 with the full body | The part is truncated first, then holds the full body; one request |
| One `offline` result, then success | One `sleep(1000)`; the retried request has the same range |
| Four consecutive failures | Sleeps 1000, 3000, 9000; then `E_MODEL_DOWNLOAD`; the part keeps its bytes |
| Failure, success, failure, success | Each failure sleeps 1000 (the counter resets on success) |
| A 206 with zero bytes; a body longer than the range | Counted as a failed attempt |
| The signal aborts mid-file | Rejects with the abort; the part keeps its bytes; no sleep |
| `writeAt` throws a `quota` error; an `io` error | `E_MODEL_STORAGE`; `E_STORAGE_IO` |
| `resumeFrom` larger than `file.bytes` | The part is truncated to 0 and the download restarts at 0 |
| Every request | Path equals `file.path`; no query; one `Range` header |
| `verifyAndFinalize`, matching hash | The part is moved to the final name; no `.part` remains |
| `verifyAndFinalize`, one flipped byte | `E_MODEL_HASH`; the part is removed; no final file exists (INV-20) |
| `verifyAndFinalize` on a 9 MiB file | The hash is fed in slices of at most 4 MiB |
| `localName` of a file whose path is `models/asr-en-v1/encoder_model.8465fcc35d96d468.onnx` | `encoder_model.onnx` (D-62) |
| `model-manifest.json` | `totalBytes` equals the sum of `files[].bytes` and is at most 260,000,000; every `sha256` is 64 hex characters; every `path` starts with `models/<modelId>/` |

### 23.4 `web/tests-e2e/helpers/fake-api.ts`, `helpers/fixtures.ts`

```ts
// fake-api.ts: added
export const TEST_ENTITLEMENT_SEED: Uint8Array;                    // fixed 32 bytes; never a real key
export function testPublicKeyBase64(): string;                     // must equal VITE_ENTITLEMENT_TEST_PUBLIC_KEY of the build under test
export function mintEntitlementToken(o?: { plan?: "free" | "creator"; iat?: number; exp?: number; periodEnd?: number;
                                           freeExportsRemaining?: number; seed?: Uint8Array }): string;   // D-25, Node crypto Ed25519
export function seedEntitlement(page: Page, token: string): Promise<void>;   // writes { schemaVersion: 1, value: { token, storedAt } } to store "entitlement", key "current"

// fixtures.ts: added
export function routeAssets(page: Page, o?: { corrupt?: string; failAfterBytes?: number; status?: number }): Promise<AssetLog>;
export function dropClip(page: Page, fixture: string): Promise<void>;        // DataTransfer with the fixture bytes, named "clip.mp4"
export function ensureModelCached(context: BrowserContext): Promise<void>;   // runs one download into the context's OPFS
export function opfsList(page: Page, dir: string): Promise<string[]>;
export function sourceDurationMs(fixture: string): number;                   // ffprobe on the fixture's video stream
export function referenceClip(): string;                                     // D-39: the testclips/ file, else its copy in fixtures/.cache/
```

`routeAssets` answers every asset-host URL from `fixtures/.cache/` (models) and `referenceClip()` (the sample clip, D-39), honouring `Range` with 206 and `Content-Range`, and records method, URL, headers and byte counts. With `E2E_REAL_ASSETS=1` it only records (D-41). `globalSetup` downloads any manifest file missing from `fixtures/.cache/` from the real asset host and checks its SHA-256. When `testclips/` does not hold the reference clip (as on the CI runner), it downloads `SAMPLE_CLIP_PATH` the same way and checks that the file's SHA-256 starts with the hash segment of its name. `mintEntitlementToken` defaults: `plan: "creator"`, `iat = now`, `exp = now + 7 days`, `periodEnd = now + 30 days`. Every media suite calls `installFakeApi` (V1) so no test reaches a real server.

### 23.5 `web/tests-e2e/landing.spec.ts`: the two replaced cases (D-56)

| Case | Expect |
|---|---|
| Drop starts an import | Dropping a small file on `/` gives one of two outcomes, and the test accepts either because the CI browser may be unsupported: the page navigates to `/app` (capability `supported`), or the drop zone shows the `B_UNSUPPORTED` copy and stays on `/`. In both, no request other than events, `healthz` and asset GETs is made |
| `/app` | Shows the drop zone or the unsupported page, never a blank screen |

The other V1 cases are unchanged and still pass. This suite must keep passing on the Linux runner, where the capability check may fail.

### 23.6 `web/tests-e2e/model-download.spec.ts` (TS §27.1, J4)

| Case | Expect |
|---|---|
| First run | After a drop the panel shows the J4 sentence with the size derived from the manifest; the bar advances at least twice; the panel disappears and the feed shows "Transcribing…". A `model_download` event carries `outcome: "ok"`, `resumed: false` and a `duration_ms` |
| Requests | Every model request is a GET to the asset host with one `Range` header, no cookie, no query string, no body (TS §25.2 assertion 2) |
| Interrupted download resumes | Route with `failAfterBytes` of 20 MiB and close the page mid-download; in a new page of the same context, `opfsList("models/asr-en-v1")` shows a `.part`; after a new drop the first model request's `Range` starts at that part's size, not 0; the event carries `resumed: true` |
| Hash mismatch re-downloads | Route with one corrupted file: the page shows the `E_MODEL_HASH` message; no file, final or `.part`, exists for it; the event carries `outcome: "hash_mismatch"`. After "start over" and a drop with correct bytes, that file's first request starts at 0 and the model becomes ready |
| Second session skips | With the model ready, reload and drop again: zero requests to model URLs; the panel never appears |
| Retries exhausted | Route answering 503: after the three backoffs (page clock advanced) the page shows the `E_MODEL_DOWNLOAD` message; the event carries `outcome: "failed"`; the `.part` file still exists |
| Quota | With the origin quota overridden through the DevTools protocol to less than the model size: the `E_MODEL_STORAGE` message |
| Loading | During transcription no request leaves the page except `POST /api/v1/events`; in particular none to any host named in the D-38 list (TE-1) |

### 23.7 `web/tests-e2e/pipeline-preview.spec.ts` (V2 cases; TS §27.1, J6-J7)

Model pre-cached with `ensureModelCached`. Fixture: the reference clip, through `referenceClip()` (D-39, D-64).

| Case | Expect |
|---|---|
| Feed | The first line is "Transcribing…"; at least one "Found:" line follows; no "Cleaning voice" line |
| Real detections only | Every "Found:" line's amount appears in the expected-events list of `fixtures/speech/README.md` |
| Preview before sign-in | With no `entitlement` record and no session, the player appears; after Play, two screenshots of the canvas 1 s apart differ, and neither is a single flat color |
| Pause and resume | Pause: two screenshots 500 ms apart are identical. Play again: they differ. No failure message appears (D-45: a leaked frame would fail the pause) |
| Events | `clip_accepted` with `source: "user"`, `orientation: "landscape"`, `duration_bucket: "lte90"` (D-64); `stage_timing` for `probe_audio`, `asr` (with `asr_backend`) and `detect_scene`; none for `audio_chain`; `pipeline_done` with `n_number >= 1` and the other three counts 0; exactly one `preview_played` after two plays |
| Event shape | Every event body parses as `EventsBatch`; no property is a free string; no feed text appears in any request (INV-2) |
| Quiet processing | Between `clip_accepted` and `pipeline_done` the only requests are `POST /api/v1/events` (TS §25.2 assertion 4) |
| Sample clip | Clicking the sample button requests `SAMPLE_CLIP_PATH` from the asset host and yields `clip_accepted` with `source: "sample"` |
| OPFS | After `ready`: `clips/<id>/source` and `clips/<id>/out48.f32` exist; `out48.f32` is `round(duration_ms x 48) x 4` bytes; no entry name contains `clip.mp4` (P-11) |
| Start over | "Start over" returns to the drop zone and `clips/` is empty |
| Long tasks | The longest main-thread task while processing is written to the test output. It is not asserted in V2 (TS §13.5: assumption, M2.4) |

### 23.8 `web/tests-e2e/export-creator.spec.ts` (V2 cases; TS §27.1)

| Case | Expect |
|---|---|
| Creator export passes the verifier | With a seeded Creator token: Export, wait for the download, then `verify_mp4.py <file> --profile creator --expected-duration-ms <sourceDurationMs>` exits 0 (checks 1-6) |
| Progress | The stage labels for rendering and muxing appear in that order; the "rendering on your computer" line is visible throughout |
| Events | `export_started { profile: "creator" }`; `stage_timing` for `render_encode` and `mux`; `export_done` with `profile: "creator"`, `from_cache: false`, `word_edits: 0`, `events_disabled: 0`, `events_kept >= 1`, `style: "clean"` |
| Download name | Matches `^offcut-\d{8}-\d{4}\.mp4$` |
| Storage | `exports/` holds exactly one `.mp4` and `exports/tmp/` is empty |
| Free-plan token | Output is 720x1280 (`--profile free`, checks 1-6); `export_started.profile` is `"free"` (INV-9: the size follows the token) |
| No token | The button area shows the unavailable message; no download; no `export_started` |
| Token signed with another seed | The failure stub appears; `export_failed { error_code: "E_ENTITLEMENT_INVALID", stage: "render_encode" }`; `exports/` is empty |
| Preview after export | Play works again and the canvas changes (the preview canvas and profile were restored) |
| Second export | `exports/` again holds exactly one file, named after a new `ExportId`; the first export's file is gone (D-40) |
| Quiet export | Between `export_started` and `export_done` the only requests are `POST /api/v1/events` |

### 23.9 `verify/verify_mp4.py` (checks 1-6)

Invocation: TS §27.2 verbatim. `--source` is accepted and unused until V5. Output: one line per check, `PASS <n>` or `FAIL <n>: <reason>`; a final line listing checks 7-11 as not implemented; exit status 0 only when checks 1-6 pass. It runs `ffprobe` and `ffmpeg` as subprocesses, parses top-level box headers itself for the faststart check, and imports nothing from `crates/` or `web/` (TS §27.2).

| # | Implementation |
|---|---|
| 1 | `ffprobe` reports format `mov,mp4,...`, exactly one video and one audio stream; the `moov` box offset is smaller than the `mdat` offset |
| 2 | Video codec `h264`, `pix_fmt` `yuv420p`, sample aspect ratio 1:1 or absent, no rotation tag or display-matrix side data; size 1080x1920 (`creator`) or 720x1280 (`free`) |
| 3 | From `ffprobe -show_frames`: every PTS delta equals 1/30 s exactly in the stream time base; frame count equals `ceil(expected_duration_ms x 30 / 1000)` |
| 4 | Audio codec `aac`, profile `LC`, 48,000 Hz, 2 channels |
| 5 | Video duration within one frame (33.4 ms) of `expected_duration_ms`; audio duration within 21.3 ms of the video duration |
| 6 | `ffmpeg -v error -i <file> -f null -` writes nothing to stderr and exits 0 |

`verify/requirements.txt` pins NumPy (used from V5). `verify/README.md` states the invocation, the tool versions used, and that this directory shares no code with the product.

### 23.10 `bench/device-bench.ts`

A Playwright test file run headed with `channel: "chrome"` against `vite preview` of a build that carries the test key. One warm-up run, then 10 measured runs (TS §30): each run opens a page in a persistent context whose model is cached, drops the reference clip, waits for `ready`, seeds a Creator token, exports, waits for the download. Timings are read from the `stage_timing`, `pipeline_done` and `export_done` events captured by the fake API.

Result file `bench/results/<BENCH_DEVICE>-<yyyy-mm-dd>.json`:

```json
{ "device": "r1", "date": "2026-10-30", "commit": "<sha>", "chrome": "<version>", "clip": "speech_scriptA_landscape_720p.mp4", "clip_duration_ms": 74705,
  "asr_backend": "webgpu", "runs": 10,
  "stages": { "probe_audio": { "ms": [], "median": 0, "p90": 0 }, "asr": {}, "detect_scene": {}, "render_encode": {}, "mux": {} },
  "total": { "ms": [], "median": 0, "p90": 0 },
  "peak_memory_bytes": null }
```

`total` is `pipeline_done.total_ms + export_done.total_ms`. `peak_memory_bytes` is the largest `performance.measureUserAgentSpecificMemory()` reading sampled every 2 s, or `null` when the API is unavailable. The file holds numbers and enums only: no transcript, no file name. `--compare` is V9.

---

## 24. Experiments, measurements and the M0 gate

Write each outcome into `docs/v2/experiments.md` (question, method, numbers, date, decision), replace the matching (assumption) in TS §39.3, and update `check-external-facts.mjs` for TE-10.

### 24.1 Experiments

| ID | Step | Procedure | Pass (TS §37) | If it fails | Recorded in |
|---|---|---|---|---|---|
| **TE-1** | S6 | Run `asr.worker` in a production build under the production CSP with the model in OPFS. Capture every request with the DevTools protocol while loading and transcribing the reference clip, once per backend | Zero requests outside TS §24.1; word timestamps for the reference clip; WebGPU and WASM backends both run | Second runtime behind the same `whisper-runtime.ts` interface (TS §16 contingency) | `experiments.md`; the option names in `v2changelog.md`; the D-38 list |
| **TE-2** | S6 | Inspect the runtime's output for a per-token probability; time transcription with and without it | Available at no more than 10% time cost | `Confidence(1.0)` for every word | `experiments.md`; TS §39.3 item 7 |
| **E-3** | S6, S15 | `pnpm bench:device` on R1, stage `asr`, 10 runs | For a 60 s clip: median at most 20 s; p90 total at most 180 s (PS §19). For the reference clip (D-64): 25 s and 224 s. In V2 the total lacks the audio chain; add its allowance (5 s) when reading the p90 | 25-50 s: continue and record the miss. Over 50 s: the base-size model, then the tiny-size one, and stronger caption editing (PS §20.4, D-66) | `bench/results/r1-*.json` |
| **TE-3** | S11 | In `render.worker`, render every output frame of the reference clip (2,242) to an `OffscreenCanvas` at 1080x1920 with capture method A (`new VideoFrame(canvas)`) and method B (texture readback); encode both and compare ten sampled frames between the two outputs, and inspect one captioned frame by eye; repeat with the tab hidden; on R1 | A correct frame every time; at least 30 frames per second render-only on R1; works hidden | The other capture method; then the Canvas2D backend (TS §19 contingency) | `experiments.md`; the chosen method named in `export-loop.ts` and TS §21.4 |
| **TE-4** | S11 | Log which ladder entry `pickVideoConfig` returns on R1 for 1080x1920 and 720x1280; export the reference clip; run the verifier; measure the audio priming (first chunk timestamp, decoded leading silence) and set `AAC_PRIMING_SAMPLES` | A ladder entry supported on R1; verifier checks 1-6 pass; A/V offset within one frame. Phone playback and the 11/11 run are M2.5 | Next ladder entries; explicit silent pre-roll with a matching `elst`; bitrate change (TS §21 contingency) | `experiments.md`; ladder order and bitrates in TS §21.2; TS §39.3 item 16 |
| **E-4** | S11, S15 | `pnpm bench:device` on R1, stage `render_encode` | For a 60 s clip: at most 90 s (PS §19). For the reference clip (D-64): 112 s | 112-187 s: continue and record. Over 187 s: Canvas2D overlay path; cap input at 60 s and 30 fps (PS §20.4) | `bench/results/` |
| **TE-10** | S14 | Run `e2e-media.yml` on a pull request; read the minutes used and the monthly allowance | Suites pass headless; a month of expected runs fits the free minutes | Media job on `main` and manual trigger only; `pnpm e2e:device` locally | `experiments.md`; `check-external-facts.mjs` |
| **TE-14** | S15 | Make a 90 s, 1080p, 60 fps clip from the reference with the ffmpeg CLI (looped to 90 s and scaled up; not committed). Run import, pipeline and export 10 times on R1, reading memory per phase with `measureUserAgentSpecificMemory` and the browser task manager | Peak under 1.5 GB; no tab crash in 10 runs | Smaller queues; preview at 360x640; smaller model (TS §31) | `experiments.md`; TS §39.3 item 23 |
| **TE-7** (re-check) | S5 | The real model files, up to 150 MB, are fetched by range from the deployed page | 206 responses; no egress charge appears | The other candidate; split files under the host's limit | `experiments.md` |
| **E-1** (read only) | S16 | Count `landing_view` rows and `platform_waitlist` rows with `wanted = 'launch'` since V1 went public | At least 1,000 visitors and at least 5% joined | Section 24.3 | `experiments.md` |

### 24.2 What V2 fixes for later versions (BP §4.4)

| Item | Fixed by | Written to |
|---|---|---|
| The speech model and its files | TE-1, E-3 | `model-manifest.json` |
| The ASR backend order and thread count | TE-1 | `whisper-runtime.ts` |
| Whether word confidence exists | TE-2 | `word-timestamps.ts`; TS §39.3 |
| The frame-capture method | TE-3 | `export-loop.ts` |
| The encoder ladder order, bitrates, AAC priming | TE-4 | `encoders.ts`, `profile.rs`; TS §21.2 |
| Whether the media job runs on pull requests | TE-10 | `ci.yml` |

### 24.3 The M0 gate (Sun 1 Nov 2026)

Decide from three readings (BP §4.3, PS §9.9, PS §20.4):

| Reading | Continue | Apply the fallback, then continue | Stop |
|---|---|---|---|
| E-1 waitlist | At least 1,000 visitors and at least 5% join | Under 1,000 visitors: inconclusive, keep going and keep measuring. 3-5%: rewrite the pitch once and re-run | Under 3% again after the rewrite, with at least 1,000 visitors |
| E-3 ASR on R1 (reference clip, D-64) | Median at most 25 s | 25-50 s: ship and record the miss. Over 50 s: smaller model | Still failing after the fallback: cloud transcription would become launch-blocking (PS §19), which the 0 USD rule cannot fund |
| E-4 render + encode on R1 (reference clip, D-64) | At most 112 s | 112-187 s: ship and record. Over 187 s: Canvas2D path, 60 s / 30 fps cap | No path under budget |

**E-1 on the gate day.** Outreach starts after V2's build work (section 1A). If the page has not been announced by the gate, or has had fewer than 1,000 visitors, E-1 falls in the middle column as inconclusive: record that with the visitor count, decide on E-3 and E-4 alone, and keep measuring, as that column says.

The E-3 and E-4 numbers in this table are the 60 s thresholds of PS §19 and PS §20.4 multiplied by 1.245 (D-64). Write both the measured value and the value normalised to 60 s (measured x 60,000 / 74,705).

Write the decision into `docs/v2/experiments.md` as: the three numbers, the column each fell in, the action taken, the date. V3 does not start without it (BP §5: "Depends on V2 and a passed M0 gate").

---

## 25. Hand-over: V3 and V4

### 25.1 What V2 freezes

Contracts from the end of V2. Changing one later means touching both sides of a boundary.

| Frozen in V2 | Where |
|---|---|
| The RPC client surface: results, `{ cancelled: true }`, `WorkerCallError`, `notify`, the thrown `{ code, detail, stage? }` and `{ rejected }` shapes | `workers/rpc.ts`, section 13.2 |
| The pool table shape (kind, script, stage tag, one-way, during-preview) | `workers/pool.ts` |
| The exported names and shapes of both WASM bundles | `CoreApi` in `load-core.ts`, `RenderApi` in `load-render.ts` |
| The OPFS path layout | `persistence/opfs.ts` |
| The entitlement token wire format and the profile table | `offcut-entitlement` (D-25, D-26) |
| The event-id byte layout | `event_id.rs` (D-47) |
| The display-list types and the `FontId` order | `display_list.rs` (D-29) |
| The four machine tables and their event names | `machines/*.ts`, `preview-store.ts` |
| `BLOCKER_CODES` and its order | `state/blockers.ts` |
| The model manifest shape; part size, retries, backoff | `model-manifest.json`, `download.ts` |
| The MP4 layout the muxer writes and its API | `mux.rs`, `mux_boxes.rs` |
| `RawWord`, `Token`, `TokenKind`; `Transcript.model_version = modelId` | `offcut-text`, D-50 |
| The lint edges and cast overrides | `eslint.config.js` (D-27, D-58, D-59) |
| The verifier's command line and output format | `verify/verify_mp4.py` |
| The benchmark result shape | `bench/device-bench.ts` |

### 25.2 File by file

| V2 file | V3 adds | V4 adds |
|---|---|---|
| `Cargo.toml` | Dependencies `nnnoiseless` (TE-13), `ebur128`. No member | `insta` |
| `offcut-mp4/src/probe.rs`, `sample_table.rs` | Hardening against the fixture matrix and the corpus (rotation, VFR, track counts, more `elst` shapes); `tests/demux_fixtures.rs`, `tests/probe_rejections.rs` | Nothing |
| `offcut-mp4/src/validate.rs` | Nothing: it is tested, not changed | Nothing |
| `offcut-mp4/src/mux.rs`, `mux_boxes.rs` | Nothing | Nothing (`ctts`, overflow test in V5) |
| `offcut-dsp` | `config.rs`, `highpass.rs`, `denoise.rs`, `compressor.rs`, `limiter.rs`, `loudness.rs`, `chain.rs`, `prosody.rs`; four test files | Nothing |
| `offcut-text` | `numbers.rs` complete, `units_lex.rs`, the 40-word split, `normalize.rs` complete; three test files. `Token` and `RawWord` do not change | Nothing |
| `offcut-detect` | `from_to.rs`, `list.rs`, `keyword.rs`, `exclusions.rs`, `resolve.rs`; `config.rs` gains lexicons and windows; `detect()` steps 3-4 are replaced by the full pipeline of TS §17.3; `redetect_sentence`; labels; four test files | Nothing |
| `offcut-entitlement` | Nothing | Nothing (`sign.rs` and the two test files in V6) |
| `offcut-scene` | Nothing. The label row appears on its own once V3 supplies labels | `styles.rs` Bold and Tech rows; `events/list_reveal.rs`, `from_to.rs`, `keyword_pop.rs` and their arms in `events/mod.rs`; the offset in `framing.rs`; `watermark.rs`; `summary.rs`; five test files and snapshots |
| `offcut-render` | Nothing | `render_to_image`; `tests/golden_frames.rs` |
| `offcut-wasm-core` | `audio_api.rs`; `CoreApi` gains `runChain`, `measureProsody` | Nothing |
| `offcut-wasm-render` | `detect_api.rs` gains `redetect_sentence`; `RenderApi` gains it | Nothing |
| `workers/protocol.ts` | Nothing | Nothing |
| `workers/rpc.ts` | Nothing | Progress throttle, cancel timeout, crash handling; `rpc.test.ts` |
| `workers/pool.ts` | One table row: `audio`; `audio.worker.ts` appears beside the others | `pool.cancel`, `restartWorker` (restart budget in V7) |
| `workers/media.worker.ts`, `media/*` | Nothing | Nothing |
| `workers/asr.worker.ts`, `asr/*` | Nothing | The empty-output retry on `wasm` (TS §16.6) |
| `workers/render.worker.ts` | Nothing | Handlers `redetectSentence`, `previewSeek` |
| `render/video-source.ts`, `preview-loop.ts` | Nothing | `prefetch`, seek to keyframe; the 360x640 resize rule |
| `render/export-loop.ts`, `encoders.ts`, `opfs-sink.ts` | Nothing | Nothing (V5) |
| `wasm/load-core.ts`, `load-render.ts` | Type additions above | Nothing |
| `persistence/opfs.ts` | Nothing; `clips-repo.ts`, `transcript-repo.ts`, `edits-repo.ts` appear beside it | Nothing (`sweepTemp`, `removeAll` in V7) |
| `persistence/entitlement-repo.ts` | Nothing | Nothing (V6) |
| `models/*` | Nothing | The pipeline's `AbortController` is finally aborted by `cancel-job.ts`; `model-machine.test.ts` |
| `state/model-store.ts`, machines | Nothing | `clip-machine.test.ts`, `model-machine.test.ts` |
| `state/clip-store.ts` | Nothing: `reject` already carries the reason | Edit actions (`setEdit`, `setEvents`), `edit_start` / `edit_done` |
| `state/preview-store.ts` | Nothing | Seek events fired |
| `state/export-store.ts`, `blockers.ts` | Nothing | Nothing (V5) |
| `usecases/start-app.ts` | Nothing | Nothing |
| `usecases/import-clip.ts` | `clipsRepo.put` after acceptance; `track clip_rejected`; the D-40 stand-in stays until V7 | Nothing |
| `usecases/run-pipeline.ts` | The two marked stand-ins are replaced: `pool.audio.runChain` in parallel with ASR and `pool.audio.measureProsody` after it (C-4 B2, D); the "cleaning_voice" line; `stage_timing audio_chain`; `transcriptRepo.put`, `editsRepo.put`; `clip_rejected` for no speech | `client_error`; retry from the last stage; `run-pipeline.test.ts` |
| `usecases/control-preview.ts` | Nothing | `seek` |
| `usecases/start-export.ts` | Nothing | Nothing (V5) |
| `copy/messages.ts` | The 14 `REJECT_*` messages; they leave `COPY_PENDING` | `NO_EVENTS_FOUND`; review-panel copy; the error copy `ErrorPanel` shows |
| `ui/pages/EditorPage.tsx` | `RejectionPanel` replaces the rejected stub | `ReviewPanel`; `ErrorPanel` replaces the failed stub |
| `ui/components/DropZone.tsx` | Nothing (rejections surface through the store) | Nothing |
| `ui/components/ModelDownloadPanel.tsx`, `ProcessingFeed.tsx`, `ExportProgress.tsx` | `ProcessingFeed`: nothing, the copy for every line already exists | Cancel buttons; retry on the model panel |
| `ui/components/PreviewPlayer.tsx` | Nothing | Scrubber; position display |
| `ui/components/ExportButton.tsx` | Nothing | Nothing (V5, V6) |
| `web/eslint.config.js` | Nothing | Edges for the four edit use-cases only if the matrix lacks them |
| `tests-e2e/helpers/*` | Fixture-manifest helpers | `network-capture.ts` is V8; helpers for edits |
| `tests-e2e/pipeline-preview.spec.ts` | Nothing | The remaining cases: zero-event clip, long-task assertion |
| `tests-e2e/export-creator.spec.ts` | Nothing | Nothing (V5) |
| `.github/workflows/e2e-media.yml` | `rejections.spec.ts` joins the `media` project; a `pnpm fixtures` step | Four more suites |
| `scripts/check-copy-codes.mjs` | Pending list shrinks by 14 | Pending list shrinks |
| `verify/*`, `bench/device-bench.ts` | The `audio_chain` stage appears in results with no code change | Nothing |
| `fixtures/speech/*` | The committed fixture set starts here: a portrait recording of Script A (phone, 1080x1920), the two pause fixtures and the no-events clip; `gen_fixtures.sh`, `manifest.json`, `labeled/`. E-3 and E-4 are read again on the portrait clip (D-64) | `golden/` |

### 25.3 What must be true on V3's first morning

- [ ] The M0 gate decision is written down and says continue.
- [ ] The model, the capture method and the ladder order are fixed and recorded (section 24.2).
- [ ] TE-2's outcome is known. V3's detector multiplies every score by word confidence (TS §17.4).
- [ ] `validate_probe` returns every `RejectReason` it owns in its inline tests. V3 adds fixtures and copy, not rules.
- [ ] `fixtures/speech/README.md` holds the expected transcript and expected events of the recorded reference clip (TS §5).
- [ ] `e2e-media.yml` is green on `main` and its cost in minutes is known.
- [ ] The R1 baseline file is in `bench/results/`.
- [ ] The two stand-ins in `run-pipeline.ts` and the two D-40 stand-ins are each marked with a `V3:` or `V7:` comment naming this section.
- [ ] The pool table has three rows and nothing else in `pool.ts` names a worker.
- [ ] `COPY_PENDING` matches section 22.4.
- [ ] `AAC_PRIMING_SAMPLES` holds a measured value.
- [ ] The spec corrections of section 27 are applied to `docs/technicalspec.md`.
- [ ] No `TODO` without a version tag exists in V2 code.

### 25.4 What V3 inherits without further work

- `importAndProbe` already returns `{ rejected: RejectReason }` for all 11 rules, the size rule, `REJECT_CONTAINER` and `REJECT_CORRUPT`; `clip-store` already holds the reason.
- `resample_mono`, the exact-length rule (D-34) and `out48.f32` on disk: the voice chain only has to take `pcm48` and return a buffer of the same length (INV-10).
- A neutral `Prosody` of the right length already flows into `detect`; real values change scores, not plumbing.
- `number::find` already takes `Prosody` and `DetectorConfig`; the energy bonus becomes live with no signature change.
- `EventParams` arms for all four kinds are matched explicitly in `events/mod.rs`, in the feed copy and in `ProcessingFeed`, so a new kind detected in V3 shows a feed line at once and draws nothing until V4.
- `pipeline_done` already carries four counts; `stage_timing` already accepts `audio_chain`.
- Worker progress, feed lines and cancellation flags already work; V4 only adds a sender.
- The media CI job, the verifier and the benchmark run unchanged on every later version.

---

## 26. Exit checklist

V2 is done when every box is ticked (BP §1.2, BP §4.4, TS §28 M0.2-M0.4).

**Code and checks**

- [ ] `pnpm check` is green: fmt, clippy, deny, ESLint, `tsc`, `check-file-tree`, `check-gen-clean`, `check-copy-codes`.
- [ ] `pnpm test` is green, including `mux_roundtrip.rs`, `download.test.ts` and every inline test of sections 6-11.
- [ ] `pnpm build` is green, including `check-hosts.mjs`; both WASM bundles are produced.
- [ ] `pnpm e2e` is green: `landing.spec.ts` with its two replaced cases.
- [ ] `model-download.spec.ts` passes.
- [ ] `pipeline-preview.spec.ts` and `export-creator.spec.ts` pass on the reference clip with `pnpm e2e:device` on R1.
- [ ] `e2e-media.yml` runs the three media suites in CI and is green on `main`, or TE-10's fallback is in place and recorded.
- [ ] `verify_mp4.py` checks 1-6 pass on an export made on R1.
- [ ] No source file exceeds 400 lines, tests excluded.
- [ ] `git grep` finds no `fetch(` outside the four allowed files and no font file outside `crates/offcut-scene/assets/fonts/`.

**Deployment**

- [ ] `ci.yml` is green on `main`; the API and the web build are deployed, server first.
- [ ] The test-key guard passed: the deployed bundle does not contain the test public key.
- [ ] The header check and the `@smoke` case pass against the deployed URL; `crossOriginIsolated === true`.
- [ ] On the deployed page in Chrome on R1: dropping the reference clip downloads the model from the asset host, shows the feed and plays the preview. Export shows the unavailable message.
- [ ] `/ort/<version>/` files are served from the app origin with `application/wasm` and the immutable cache header.
- [ ] The model files and the sample clip are on the asset host; the sample button works on the deployed page.
- [ ] `clip_accepted`, `model_download`, `stage_timing`, `pipeline_done` and `preview_played` rows appear in `analytics_events` with enum and integer props only.

**Experiments and records**

- [ ] `bench/results/r1-<date>.json` is committed, with `asr` and `render_encode` medians and p90 from 10 runs.
- [ ] TE-1, TE-2, TE-3, TE-4, TE-10, TE-14 outcomes are in `docs/v2/experiments.md`; each failed one has its fallback applied.
- [ ] The model, the capture method and the encoder ladder order are fixed (section 24.2).
- [ ] Every (assumption) V2 measured is replaced by its value in TS §39.3, or still listed there.
- [ ] The M0 gate decision is written down with the three readings.
- [ ] Any decision of section 2 that changed during the build is corrected here and in the specs; the spec corrections of section 27 are applied.
- [ ] `docs/v2/v2changelog.md` lists every pinned version and every third-party option name confirmed during the build.
- [ ] Section 25.3 is fully ticked.
- [ ] Tagged `v2`.

---

## 27. Open references and spec issues found

Nothing below was resolved by guessing. Items 1-10 are contradictions or gaps in the inputs that a decision in section 2 covers; the spec should be corrected so the decision is no longer needed. Items 11-18 stay open. Items 19-25 are places where the first draft of this plan did not match V1 as built, or itself; they were found on 2026-10-08 by reading the V1 code and `docs/v1/v1changelog.md`, and each is covered by the decision named. Item 26 is the founder's choice of reference clip.

| # | Issue | Where | Handled by |
|---|---|---|---|
| 1 | The boundary table and the dependency diagram lack edges that the spec's own flows need: the sample clip fetch from a use-case (§15.4), workers using the OPFS path builder (§23.1), the render worker reading the public key (§21.3), the model manager writing its store (§12.1) and reading `meta` for the persist request; the stores, the model manager and the use-cases naming `FeedLine` and `AppFailure` | TS §2, §7 | D-27. Correct the two TS tables |
| 2 | Use-cases may import only `cancel-job.ts` and `restore-clip.ts`, yet C-2 has `importClip` call `runPipeline` and C-9 has `startExport` call `controlPreview.pause()` | TS §7 vs TS §9 | D-58. Correct TS §7 |
| 3 | The V1 hand-over says V2 preloads "four worker scripts"; `audio.worker.ts` is created in V3 | V1 §15.2 vs BP §5.1 B | D-22. Three scripts in V2 |
| 4 | Stage timings are said to come from workers, but the protocol returns timings only from `exportClip` | TS §32 vs TS §14.2 | D-35. Correct TS §32 |
| 5 | Brand casts are allowed only in `gen/` and `workers/`, yet main-thread signatures produce `Bytes`, `TimeMs` and `ExportId`, and the generated files hold no constructors | TS §10.1 vs TS §16.2, §20.1, C-9; V1 §9 | D-59 |
| 6 | The copy check must cover blockers, but blocker codes are in no generated array and `blockers.ts` is scheduled for V5 while `DropZone` reads blockers in V2 | TS §11.3, V1 §9, V1 §15.2, BP §7.1 | D-20 |
| 7 | `offcut-scene` needs `format_quantity`, which lives in `offcut-text`; the diagram has no such edge | TS §7 vs TS §19.5 | D-28. Correct the diagram |
| 8 | Names used and never defined: `Rect`, `Rgba`, `PathEl`, `PositionedGlyph`, `Stroke`, `Affine`, `FontId` (§19.2); `Token` (§17.1); `DemuxerHandle` (§20.1); `JsRandomAccess` (§19.2); the Rust `RawWord` (§17.1) | TS | D-29, D-30, D-31, D-42 |
| 9 | The live-frame counter is a dev-build assertion read by E2E suites, and E2E runs production builds | TS §21.4, INV-11 vs V1 D-15 | D-45 |
| 10 | Files this plan creates earlier than the build plan: clip and export machines, `blockers.ts`, `validate.rs`, `entitlement-repo.ts`, `entitlement-public-key.ts`; the audio `elst` | BP §4-§8 | D-18, D-20, D-21, D-23, D-24, D-33. BP §3 already gives the implementation document precedence |
| 11 | `Scene::summary()` returns `ChangeSummary.voice_cleaned`, but a scene has no input that says whether the voice chain ran | TS §10.6, §19.2 | Open for V4. V2 returns `false` (D-22) |
| 12 | C-3 hashes while downloading ("opfs append then `Sha256Stream.update()`"); §16.3 hashes the finished part file in 4 MiB reads. Only the second works after a resume | TS §9 C-3 vs TS §16.3 | This plan follows §16.3. Correct C-3 |
| 13 | The V1 hand-over says `fake-api.ts` "returns" a Creator token; no route can return it before V6 (`GET /entitlement` needs a bearer token, `api-client.ts` is frozen until V6) | V1 §15.2 | D-23: the helper mints and seeds |
| 14 | P-5's structural proof says use-cases between import and `ready` import no `net` module; `import-clip.ts` imports `net/asset-fetch` for the sample clip. The fetch happens before `clip_accepted`, so assertion 4 of TS §25.2 is unaffected | TS §25.1 P-5 vs TS §15.4 | D-27 a. Reword P-5 |
| 15 | Values the specs defer to V2 experiments and that this plan therefore cannot state: the runtime's option names (TE-1), the AAC priming count (TE-4), Chrome's launch arguments for WebGPU on the CI runner (TE-10), the model file list and hashes | TS §16.4, §21.3, §33 | Recorded at S5, S6, S11, S14 |
| 16 | Whether the 20 s transcription budget includes loading the session. The session is loaded for every clip (INV-12) | PS §20.2, TS §30 | D-35 counts it. Confirm in PS or TS |
| 17 | E-3's "p90 total at most 180 s" cannot be measured in V2: the audio chain does not exist | PS §19, BP §4.2 | Section 24.1 reads the V2 total plus the audio-chain allowance (5 s for the reference clip, D-64); final reading in V9 |
| 18 | `EntitlementClaims` is not generated to TypeScript, and V6's `entitlement-store` is to hold decoded claims on the main thread, where use-cases may not call WASM | TS §12.1, §10.6 | Open for V6. V2 needs only the unverified `plan` read of D-52 |
| 19 | `session.rs` calls `build_scene`, but the crate graph, `check-file-tree.mjs` and `deny.toml` let only `offcut-render` depend on `offcut-scene` | TS §7 vs TS §19.2 | D-61 |
| 20 | `check-hosts.mjs` reads every byte of `web/dist`. The fonts inside the render bundle and the runtime files under `ort/` can hold URL strings, not only the ASR worker chunk | V1 §12.6 as built | D-38 |
| 21 | `upload-assets.sh` accepts the folders `media` and `models` only and writes one flat level; the manifest paths are `models/<modelId>/<file>` | V1 as built vs TS §16.1 | D-62 |
| 22 | The asset bucket's CORS policy allows the app origin and `http://localhost:5173`; E2E and the bench run on `vite preview`, port 4173 | V1 changelog, 2026-10-08 | D-41 |
| 23 | `tests-e2e` is type-checked with the app's `tsconfig.json`, which has no Node types; the pnpm workspace is `web` alone, so `bench/device-bench.ts` cannot resolve `@playwright/test` | V1 as built vs TS §5 | D-63 |
| 24 | V1 added `CREATOR_VIDEO_BITRATE` to `encoders.ts` for the capability probe; V1's pages take paths as props; the comment on `loadCore()` rules out the main thread; a use-case may import only `pool.ts` from `workers`; `deny.toml` wrappers must name third-party parents | V1 changelog (Prompts 13, 18, 20, 23), `eslint.config.js` | D-26, D-48, D-36, D-32, section 22.1 |
| 25 | A demuxer bound that fails at `open` would report a clip that is too long, or one with PCM audio, as corrupt. The build order had the spike before the crates it needs. The one visible event had no fallback | First draft of sections 5, 6.4 and 8.4 | D-60 as revised; S9 to S11; D-42 |
| 26 | The reference clip of TS §26 and PS §20.2 is a 60-second 1080x1920 portrait clip; V2 uses a 74.7-second 1280x720 webcam recording | TS §26, PS §20.2 vs the founder's decision of 2026-10-08 | D-64, D-39. TS §26 gains a row for this clip when the portrait one is recorded in V3 |

**Decisions to copy back into `technicalspec.md`:** D-25 (§10.8), D-26 (§21.1), D-27 and D-58 (§2, §7), D-28 (§7), D-29 (§19.2), D-30 (§19.2, §20.1), D-34 (§15.1), D-35 (§32), D-37 (§16.4, §24.6), D-42 (§17.1), D-45 (§21.4, INV-11), D-47 (§17.1), D-50 (§16.1), D-53 (§5), D-55 and D-46 (§5), D-59 (§10.1). **Into `buildplan.md`:** the schedule moves of D-18, D-20, D-21, D-23, D-24, D-33. **Into `product.md`:** none.