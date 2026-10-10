# v2changelog.md - Offcut V2: record of changes

**What this is.** A record of every change made while building V2: what was done, what differs from the plan or the guide, how it was checked, and what is still open. `v2implementation.md` says what each file contains; `v2buildguide.md` gives the order of work; `coding-promptsv2.md` holds the prompts (31 to 60). This file says what happened.

**How it is kept.** One entry per prompt, or per change made outside a prompt. Entries are in date order, oldest first; new entries go at the end. Every commit that changes the repository carries its entry. The two tables below are rewritten when an item opens or closes. What happened before Prompt 31 (the V2 documents, the reference clip, the decision on R2) is in `docs/v1/v1changelog.md`, entries of 2026-10-08.

**References.** `§n` is a section of `v2implementation.md`, `G n.m` a step of `v2buildguide.md`, `D-n` a decision of §2, `TS §n` of `technicalspec.md`, `V1 item n` an open item or known issue of `v1changelog.md`.

**The baseline.** V2 starts from `main` at `322c7d307e86cb21ee0a9af341ce2d1657f66b04`, the deployed V1. Every frozen-file check compares with that commit. The test counts there: 170 Rust tests, 76 Vitest tests, 12 Playwright cases. No count may fall.

---

## Open items

| # | Item | Who | Needed by |
|---|---|---|---|
| 1 | **Closed on 2026-10-09.** The asset bucket's CORS policy allows `http://localhost:4173` (D-41); the entry of that date has the check | - | - |
| 2 | Decide whether to replace `ENTITLEMENT_SIGNING_KEY` in Render: its private half was shown in a chat on 2026-10-08 (`v1changelog.md`). Prompt 33 wrote the public half of the present key into `web/src/config/entitlement-public-key.ts`. If the key is replaced, that literal and §1A item 12 must be derived again in the same change; a token signed with the new key is otherwise refused by every browser. Nothing is signed before V6 | Human | Before V6; sooner is cheaper |
| 3 | **Closed by D-69 on 2026-10-09.** R1 is not available and is not booked. The gates of V2 are read on the development machine, D1; the entry of that date says what that costs. Boxes 13 and 14 of §1A are read as D1 | - | - |
| 4 | After every prompt marked **Push**: `git push` from `offcut/` (the repository is not the `sh2clips` folder), then read the `ci` run of pull request #2. The next prompt waits for green | Human | Prompts 35, 37, 41, 44, ... |
| 5 | Submit the merchant onboarding (TE-9) if it is not submitted. Approval can take two weeks | Human | V6 |
| 6 | Reset the Neon password (V1 item 36) and replace the demo video (V1 item 33) | Human | Before the page is announced |
| 7 | The file `.env.local` in the repository root holds one line with no name, a test-mode API key. Git ignores the file and no program reads that line. Move it into `.env.deploy` under a name | Human | Any time |
| 8 | Turn on branch protection for `main` on GitHub (Settings, Branches): require a pull request and the `ci` check. `main` is unprotected, and a push to it deploys | Human | Before Prompt 59 |
| 9 | Read the license of the speech model before the page is announced. Offcut now serves the model files from its own asset host. The repository they were taken from states no license of its own; the model's author says the code and the weights are under the MIT License, which asks for the notice to go with copies | Human | Before the page is announced |
| 10 | **The three-minute promise.** With the small-size model the median total for a 60 s clip on R1 is about 221 s on the numbers of D-67, and PS §20.2, §20.3, J10 and the pitch say three minutes. Change the promise or the model before the page says it to a visitor. The copy that states the time is not written yet | Human | Before the page is announced |
| 11 | **Closed by D-70 on 2026-10-10.** A full-range clip is drawn as its file states: the fix on the decode side was chosen. Building it is open item 13 | - | - |
| 12 | **Measure on a laptop like R1 before the page promises a time** (D-69). Nothing of V2 is measured on a machine with 8 GB and an integrated GPU alone: not E-4, not TE-14's memory, and of E-3 only the founder's one reading. The founder's plan: after the final version is deployed, on testers' devices. Until then every time in `experiments.md` is D1's, and a user's may be about twice it | Human | After the deploy; before the page is announced |
| 13 | **Closed on 2026-10-10.** D-70 is built: the entry of that date has the measurements | - | - |
| 14 | **TE-10: the job `e2e-media`.** Its first run, on 2026-10-10, failed before any test ran, by a fault of a test file that is corrected (the entry of that date); the workflow's own steps passed. After the next push: read the job, and tell the agent what it says. 30 cases green: its minutes and the date go into `experiments.md` and `check-external-facts.mjs`. A failure at the capability check names what the runner lacks: for WebGPU, Chrome's arguments go into `E2E_CHROME_ARGS` of `e2e-media.yml`; for an encoder, the fallback of §22.5 | Human, then the agent | Before Prompt 59 |
## Known issues for later prompts

| # | Issue | Affects |
|---|---|---|
| 1 | A `src/lib.rs` of 0 bytes fails `cargo fmt --check`: rustfmt wants one line break. The eight skeletons hold one line break (LF) | Any prompt that creates an empty Rust file |
| 2 | `coding-promptsv2.md` does not say which branch Prompts 59 and 60 work on after the merge. On `main`, the line `BASE=$(git merge-base main HEAD)` of the Standard Agent Block gives `HEAD`, and the frozen-file diff then compares nothing. From the merge on, use the baseline SHA above in place of `$BASE`, and put the commits made after the merge on a branch of their own | Prompts 59, 60 |
| 3 | `cargo deny check` passes with warnings, as in V1 (V1 item 6): `unused-wrapper` for crates that are not dependencies yet, and `duplicate`. They are warnings, not errors | Prompts 34 to 48 |
| 4 | **The one cast each minting file may hold is tied to a name** in `web/eslint.config.js` (`MINTING`). It must be the last `return` of a function declaration with that name: `toBytes` in `persistence/opfs.ts`, `models/download.ts` and `models/model-manager.ts`; `newClipId` in `usecases/import-clip.ts`; `newExportId` in `usecases/start-export.ts`; `toTimeMs` in `usecases/control-preview.ts`. In `state/model-store.ts` it is the top-level line `const ZERO_BYTES = 0 as Bytes;`, not exported. An arrow function, another name or a cast anywhere else in the file fails lint | Prompts 37, 39, 40, 53, 54 |
| 5 | `boundaries/dependencies` reports an import only when the imported file exists. An import of a file that is not written yet is reported by `tsc`, and by another ESLint rule, not as a layer violation. A drill on a layer edge needs its target file | Any lint drill |
| 6 | `web/vite.config.ts` and `web/vitest.config.ts` are type-checked by `web/tests-e2e/tsconfig.json`, with Node's types, and no longer by `web/tsconfig.json` (entry of Prompt 32). Prompt 43 adds a build-start step to `vite.config.ts` that uses Node's file API: it type-checks there without a further change | Prompt 43 |
| 7 | No ESLint rule covers `vite.config.ts`, `vitest.config.ts` or `playwright.config.ts`, as in V1. They are type-checked only | Prompts 43, 56 |
| 8 | **The secret scan of CI (gitleaks 8.18.4) takes a 44-character base64 literal assigned to a name with `KEY` in it for a secret.** Found in Prompt 33 with the same version run locally. Such a line must end with a `gitleaks:allow` comment **in the commit that first adds it**: the scan reads the whole history, so a comment added in a later commit does not clear the finding. The test public key (`fake-api.ts` in Prompt 46, `ci.yml` in Prompt 58) is such a literal. Before a commit that adds one, fetch the release archive of that version, check its SHA-256 against the published checksum file, and run `gitleaks detect --no-git --source <folder> --redact`. **Prompt 46 added none:** `fake-api.ts` works the test public key out from the seed and holds no base64 literal, the Rust test holds its 32 bytes as numbers, and the scan found nothing in the seed, the two test tokens or the changelog. The literal first appears in `ci.yml` (Prompt 58) | Prompts 46, 58 |
| 9 | Chrome under Playwright, headless, gives a module worker WebGPU, both encoders and a sync OPFS handle on the development machine (Prompt 33). The adapter is the integrated Intel GPU, not the GTX 1650. No headed run and no launch argument was needed | Every browser check |
| 10 | **`crates/offcut-mp4/src/boxes.rs` has 392 lines above its test module, `sample_table.rs` 387 and `demux.rs` 369; the limit is 400** (`cargo fmt` puts most signatures and struct literals on several lines). Code that must go into one of them needs room made first. The muxer has its own two files | Prompts 41, 48 |
| 11 | **What Prompt 37 builds on** (entries of Prompts 35 and 36). In a worker: `const core = await loadCore()`; `core.openDemuxer(syncHandle)` returns a `CoreDemuxer` or `{ rejected }` (test with `"rejected" in result`); `demuxer.probe(size)` gives the `ProbeInfo` whose `video.codec_string` and `demuxer.videoDescription()` go into `VideoDecoder.isConfigSupported`; `core.probeAndValidate(demuxer, size, supported)` gives `{ ok }` or `{ rejected }`. Audio: `audioDescription()`, `audioSampleCount()`, `readAudioSample(i)` with `ptsUs` (may be negative) and `durationUs`. A failure is thrown as the plain object `{ code, detail }`, not an `Error`. Call `demuxer.free()` when done, before closing the handle. `core.resample(pcm, from, to)` returns exactly `round(len x to / from)` samples. `offcut-mp4` also has a test-only module, `demux::fixture`, that builds small MP4 files | Prompts 37, 41 |
| 12 | `proptest` writes a folder `proptest-regressions/` beside the crate when a property fails. It is not in the tree of TS §5: delete it once the failure is fixed, or the file-tree check fails | Prompts 35, 36, 41 |
| 13 | **How code in a worker is written** (entry of Prompt 37). A failure with a name is thrown as `new WorkerFailure(code, detail, stage?)` from `workers/rpc.ts`: ESLint refuses a thrown plain object, and `toAppFailure` reads the error's `code`. A handler that saw its cancel flag returns `CANCELLED`. `ctx.isCancelled` and `ctx.progress` are passed on as `() => ctx.isCancelled()`, never unbound. A worker posts with `postMessage(message, { transfer })`; `serveWorker` does it and finds the `Float32Array`s and `OffscreenCanvas`es of a result itself. The lists `oneWay` and `duringPreview` are given to `serveWorker` in the worker's entry file: a worker may not import `pool.ts` | Prompts 43, 50, 53 |
| 14 | **A new worker gets one line in `pool.ts`:** `import url from "./<name>.worker.ts?worker&url"`, a row in `WORKERS` (script, stage) and a client in `pool`. That form gives the built script's URL, which the `modulepreload` link and `new Worker` both use; `new URL("./x.worker.ts", import.meta.url)` outside `new Worker(...)` would ship the TypeScript source as an asset | Prompts 43, 50 |
| 15 | **`offcut-wasm-render` needs the same read-ahead window in its own `JsRandomAccess`** (D-30 gives each binding crate its own): one call into the browser per video sample costs about 0.4 ms. The one in `offcut-wasm-core/src/media_api.rs` is the model. **Done in Prompt 48:** `session.rs` has it | Prompt 48 |
| 16 | `web/src/workers/rpc.ts` has 374 lines, of which 45 are the two tables. V4 adds the progress throttle, the cancel timeout and the restart to this file and has 26 lines for them before the limit of 400 | V4 |
| 17 | A temporary Playwright case that waits for `networkidle` can hang: the start page streams the demo video from the asset host. Wait for what the case needs instead | Every browser check |
| 18 | **The model on the asset host runs on both backends** (entry of Prompt 43): the 214,647,815-byte set of the small-size English model, a 4-bit encoder and a 4-bit decoder with 16-bit floats. The runtime asks for its seven files and for nothing else. **It is slow:** on the development machine `load` and `transcribe` took 65 s on WebGPU and 102 s on WASM for the 74.7 s reference clip, against a target of 25 s on R1 (E-3). Since D-67 the fallback line is 180 s, and the founder's reading on an R1-class laptop is about 150 s. If a later reading on R1 is over 180 s, D-66 names the base-size model: steps 2 and 3 of Prompt 38 again with its files (the 4-bit pair is 145,199,758 bytes), `MODEL_DTYPE` in `whisper-runtime.ts` to match their names, and the transcript of `fixtures/speech/README.md` taken again. The files of the small-size set are in `C:\Users\Mayan\offcut-models\asr-en-v1`, outside the repository. In a production build (entry of Prompt 44) the same machine read 62 s to 96 s on WebGPU and 98 s to 123 s on WASM | Prompt 59 |
| 19 | **What Prompts 39, 40 and 43 build on** (entry of Prompt 38). `fetchAsset(path, { range?, signal })` returns `{ ok: true, status: 200 \| 206, response }` or `{ ok: false, cause, status? }` and reads no body. On the asset host: a range that ends past the end of a file answers 206 with the bytes that exist, and `Content-Range` gives the real last byte and the total; a path that does not exist answers 404 with the CORS headers, so it arrives as `cause: "status"`, not `"offline"`; with no `Range` header the answer is 200. The manifest lists seven files, the two large ones second and third; a stored name is `<stem>.<16 hex>.<extension>`. The plain names of the two large files, `encoder_model_q4.onnx` and `decoder_model_merged_q4f16.onnx`, are the ones the runtime is expected to ask for when each half is given its precision (4-bit; 4-bit with 16-bit floats). Not confirmed before Prompt 43 | Prompts 39, 40, 43 |
| 20 | **`pnpm e2e` can fail on the development machine when it is short of memory** (entry of Prompt 39). Playwright starts 6 browsers at once; with about 3 GB free the six cases that start first time out in the capability check of the start page, and the other six pass. `pnpm --filter web exec playwright test --project=non-media --workers=3` passes. Before reading such a failure as a fault of the code, close other browsers and run again, or run with fewer workers; the `ci` run is the check on a clean machine. `playwright.config.ts` was not changed. The gate of Prompt 40, an hour later and with 6 workers, passed 12 of 12. It came back once more, in the gate of Prompt 41, right after the workspace had been compiled (one case, the longest, timed out twice), and was gone in the gate of Prompt 42 **Since Prompt 51 the first run after `pnpm build` fails one case, the same one both times:** "landing_view is sent once" (entries of Prompts 51 and 52). Its trace shows the batch posted and not yet handed to the fake API when the 5 s of the case ran out. The request is a `keepalive` fetch, the only one the app makes; whether that is why it is slow to be intercepted under load is not known. A run made a minute later passes. `ci` retries a failed case once. **In the gate of Prompt 54 the six cases that start first ran 25 to 37 s, against the 30 s a case may take, in run after run, with 3.5 to 5.7 GB free:** a build without that prompt's change did the same, and both were back at 8 to 17 s ten minutes later. `--workers=3` passed at once. When the six first cases are slow, wait or use three workers before reading it as a fault. **The failures of "landing_view is sent once" very likely had another cause, corrected in Prompt 56:** `flushAnalytics` moved the page's clock 11 s, and a flush that fired in the first second of that step had its own request reach the 10 s limit of `http.ts` within the step, before the browser sent it. It moves 9,999 ms now | Every gate |
| 22 | **What the later prompts build on** (entries of Prompts 41 and 42). **Muxer:** `Mp4Muxer::new(sink, video, audio)`, `add_video_sample`, `add_audio_sample`, `finalize`; a sink may be owned or lent (`&mut sink`); the samples of the two tracks may come in any order between each other, and the `moov` of a 90 s clip written in turns took 100 kB of the 256 KiB kept for it (32 kB written one track after the other). `mux.rs` has 355 lines and `mux_boxes.rs` 263; V5 adds `ctts`. **Text:** `core.normalizeTranscript(raw, modelId)` takes `{ text, startMs, endMs, confidence }[]` with **whole milliseconds** (1.5 is refused) and returns a plain `Transcript`. In the browser an extra field of a raw word is ignored, not refused. A confidence comes back as a 32-bit value (0.98 reads 0.9800000190734863), which matters to anything that compares it with 0.80 exactly. `offcut_text::tokenize(words, edits)` and `parse_quantity(tokens)` are what `offcut-detect` reads; `format_quantity(value, &unit)` is the one formatter `offcut-scene` may call. The `dollars` form of D-42 is not in: Prompt 43 adds it, with its row of §8.5, only if the recognizer writes the amount without a `$`. A lone cardinal in words is a quantity ("one" is 1): a caption shows it as the digit, as TS §17.2 says (entry of Prompt 45) | Prompts 43, 45, 46, 48, 50 |
| 23 | **What Prompt 44 and the later prompts build on** (entry of Prompt 43). `pool.asr.load({ modelId, backend })` answers `{ backend }`; `pool.asr.transcribe({ pcm16 }, { transfer: [pcm16.buffer], onProgress })` answers `{ ok: Transcript }` or `{ rejected: "NoSpeech" }`; `pool.asr.unload()` answers when the sessions are disposed. The pool has two rows. **`confidence` is 1 for every word, and stays so in V2:** TE-2 (entry of Prompt 44) found that the runtime returns no probability. No score is lowered by it; a prompt that reads `confidence` reads 1. **The dev server reloads the page once** the first time a browser loads the ASR worker after an install (Vite prepares the runtime): a temporary Playwright case fails with "Execution context was destroyed" and passes when run again. `whisper-runtime.ts` sets the runtime up so that it cannot make a request; the 13 literals it brings into the build are listed in `scripts/check-hosts.mjs`, each for one file, and **a new version of the runtime may bring others**: `pnpm build` then fails until each has an entry with its reason. The amount of the reference clip is words 132 and 133, `$12` and `,000`; the expected transcript is in `fixtures/speech/README.md` | Prompts 44, 45, 46, 54, 56, 57 |
| 24 | **A first transcription may be slower than the next** (entry of Prompt 44): in a new browser profile, straight after a build and the download of the model, `load` and `transcribe` took 95.5 s on WebGPU and 123.4 s on WASM; the second run took 61.9 s and 98.1 s. One pair of readings, cause not found. The bench of Prompt 59 should keep its first run apart, and E-3 at S15 should say which it reports. **Chrome makes requests of its own** (an update check, a push-message registration) while a page is open: a check that reads the browser's whole network log, and not the page's requests, sees them and must tell them apart by who started them | Prompts 57, 59 |
| 21 | **What the later prompts build on** (entries of Prompts 39 and 40). `inspect()` answers `absent`, `partial` or `ready` and tells the store on its first call; `start-app.ts` calls it as step 8 (Prompt 54). `ensureReady(onProgress, signal)` resolves on `ready` and rejects with a `ModelFailure`, whose `failure` is the `AppFailure` (`stage: "model"`), or with the abort itself when the signal aborted; a use-case imports `ModelFailure` from `models/model-manager.ts`. `useModelStore` holds `{ status, done, total, etaSecs, error? }`; its actions are called by the model manager only. `<ModelDownloadPanel />` takes no props and reads the store; `EditorPage` mounts it (Prompt 55). In OPFS the model is seven files with plain names under `models/asr-en-v1/`, which is where the cache adapter of Prompt 43 reads them. A cold download took 63 s on the development machine's line; the E2E helper `ensureModelCached` (Prompt 56) must fill OPFS from `fixtures/.cache/`, not from the asset host. In a test, a `Bytes` cannot be made by a cast: `download.test.ts` shows one way to get typed values | Prompts 43, 52, 54, 55, 56 |
| 25 | **What Prompts 47 and 48 build on** (entry of Prompt 45). `build_scene(SceneInput { transcript, events, edit, clip, profile })` gives a `Scene`; `frame_at(t)` gives a `DisplayList` in pixels of the output canvas, and `crop()` the part of the source frame to show. **How a `GlyphRun` is drawn:** the glyphs by id, from the file `fonts::bytes(font)` at the weight `fonts::weight(font)` and at `size`, each at its `x`, `y` in the run's own space; filled, then stroked with `stroke.width`, a length of that same space; all of it mapped to the canvas by `transform` (`a b c d e f`). A fading reveal is between `PushLayer { opacity, clip: None }` and `PopLayer`. V2 emits no `FillRect` and no `FillPath`. Pin `vello` 0.11.0 with `wgpu` 30: it shares `skrifa` 0.44 with `parley` 0.11.1. The crate builds for `wasm32-unknown-unknown`. The font files hold addresses in their name tables: `check-hosts` reads them once they are in `offcut_render_bg.wasm` (D-38), and the bundle grows by the 3.2 MB of the fonts and by parley's text data | Prompts 47, 48 |
| 26 | **What the later prompts build on** (entry of Prompt 46). **Detector:** `offcut_detect::detect(&transcript, &prosody, &edits, &DetectorConfig::default())` gives the events in time order; on the reference clip it must give one, the `$12k` of words 132 and 133. `event_id(kind, anchors)`. **Entitlement:** `verify_token(token, &[[u8; 32]])` gives `EntitlementClaims` or a `TokenError` (all five are `E_ENTITLEMENT_INVALID`); `export_profile(Some(&claims), now)` and `export_profile(None, now)`, with `now` as `UnixSecs` from the caller's clock; `PREVIEW_WIDTH` and `PREVIEW_HEIGHT` for `preview_profile()`. `ParsedToken` has no `Debug`, on purpose. `CREATOR_VIDEO_BITRATE` of `profile.rs` must stay equal to the one in `encoders.ts` (D-26). **Tests:** `mintEntitlementToken(o?)` and `seedEntitlement(page, token)` of `helpers/fake-api.ts`; the page must have opened the app before `seedEntitlement`, which rejects otherwise. The test public key is in the entry of Prompt 46: a build for the media suites is made with it in `VITE_ENTITLEMENT_TEST_PUBLIC_KEY`, exported in the terminal and written in no `.env` file. **The crate `offcut-entitlement` states the version of `ed25519-dalek` itself:** when the workspace's version changes, that line changes with it. **The E2E case "a slow API shows the waking message" runs in real time** and can miss its one-second window on a busy machine; run the suite again before reading it as a fault | Prompts 48, 49, 50, 54, 56, 57, 58 |
| 27 | **What Prompts 49 to 54 build on** (entries of Prompts 47 and 48). `const render = await loadRender()` in the render worker; the main thread calls `preloadRender()` only, through `pool.preload()`. `render.detect(transcript, prosody, {})`; `render.previewProfile()`; `render.exportProfileFromToken(token, keys, now)`, which throws `{ code: "E_ENTITLEMENT_INVALID", detail }`; `render.newMuxer(sink, { width, height, avcc, frameCountHint }, asc)` with `add_video_sample(data, frame, isKeyframe)`, `add_audio_sample(data, ptsUs, durationUs)` and `finalize()`, which returns the bytes and frees the muxer. `const session = await render.openSession(clipInfo, syncHandle)`; `session.set_scene({ transcript, events, edit, profile })`; **`await session.attach_canvas(canvas, width, height)`, and no other call on the session until it has answered;** `session.render_frame(frame, tMs)`; `frame_count()`, `summary()`; `video_description()`, `video_sample_count()`, `read_video_sample(i)` (`{ data, ptsUs, durationUs, isKeyframe }`), `keyframe_at_or_before(tMs)`; `session.free()`, then close the sync handle. The session closes no frame. A failure is thrown as the plain object `{ code, detail }`. **A canvas can be read back in a worker** by drawing it on a 2D `OffscreenCanvas` straight after `render_frame`, and a frame for a check can be made with `new VideoFrame(canvas, { timestamp })`: the drive of Prompt 48 did both. A `web_sys::VideoFrame` has two clones (entry of Prompt 47). A glyph run is stroked first and filled over the stroke since D-68; the outline that shows is half the stroke's width. The render bundle is 6.0 MB and every release build of it takes about two minutes | Prompts 49, 50, 51, 53, 54, 55 |
| 28 | **What Prompts 50 to 53 build on** (entry of Prompt 49). `new VideoSource(session, clipInfo)`: `await source.frameAtBlocking(t)` for the export and `source.frameAt(t)` for the preview, `t` in milliseconds; **the frame stays the source's: the loop does not close it,** and it stays open until the source hands out a newer one; `source.close()` closes everything. `liveFrames.count` goes up for every frame a decoder gives out and down at every `close()` in `video-source.ts`; a frame the loop makes itself (`new VideoFrame(canvas)`) is counted and closed by the loop. One `VideoSource` can serve a preview and then an export: a time before what it holds starts decoding again. A failure is a `WorkerFailure` with `E_DECODE_VIDEO`. `await pickVideoConfig(profile)` gives the encoder configuration or throws `E_ENCODE_VIDEO`; on the development machine it is `avc1.640028`, `prefer-hardware`. `const sink = await OpfsSink.open(paths.exportTmp(id))`, given to `render.newMuxer(sink, ...)`; `await sink.close()` on success, `await sink.abort()` otherwise; one sink per path at a time. **The verifier:** `python verify/verify_mp4.py <file> --profile creator --expected-duration-ms 74705`, with Python 3.13 first in `PATH`; it wants `yuv420p`, 2,242 frames exactly 1/30 s apart, and audio within 21.34 ms of the video. **On the reference clip:** output frames 150 and 1,946 are the two white frames; 80 output times repeat a source frame; decoding alone ran at 652 frames a second on the development machine | Prompts 50, 51, 53, 57 |
| 29 | **What Prompts 51 to 54 build on** (entry of Prompt 50). `pool.render` exists. `openSession({ clipId, clipInfo })`, `detect({ transcript, prosody })`, `setScene({ transcript, events, edit, profile: "preview" })`, `attachPreview({ canvas }, { transfer: [canvas] })`, `exportClip({ exportId, entitlementToken, out48 }, { transfer: [out48.buffer], onProgress })`, `closeSession()`. `exportClip` answers `{ opfsPath, summary, stageTimings: [render_encode, mux] }`; its progress is one `render_encode` message for each frame, `done` from 1 to `N`, then one with `stage: "mux"`. It rejects with `E_INTERNAL` and the detail `NoSession` or `NoScene` when called too early. After an export the worker closes the `VideoSource` and makes a new one: **`previewPlay` (Prompt 53) must read the source from the module's state each time and not keep it.** `runExport` takes `canvas` and `onEncoded` beside the arguments of TS §21.1. The frame in the loop is closed by `closeFrame(vf);`; without that line an export fails with `FrameLeak` (run in Prompts 50 and 51). **`export-loop.ts` has 396 lines of the 400.** Since Prompt 51 the capture method is method A and `AAC_PRIMING_SAMPLES` is a measured 0. `previewClock` is dropped without an answer until Prompt 53 gives it a handler; that handler must not throw. Two exports of Prompt 50 are in `testclips/renders/`. A check that needs the model makes its own profile under `fixtures/.cache/zz-profile-4173/`: Prompts 50 and 51 removed theirs. `vite build` alone takes a second when the two bundles are built already, so a keyed build for a check needs no `pnpm build:wasm` unless Rust changed | Prompts 51, 52, 53, 54 |
| 30 | **Closed on 2026-10-10: D-70 is built, and a full-range clip is drawn and exported with its own brightness.** It was: **Chrome reads a full-range clip as limited-range** when the clip carries the range flag and no colour description, as the reference clip does (entry of Prompt 50): a `VideoFrame` of it has `fullRange: false`, and the picture is drawn, previewed and exported with its brightness stretched from 16..235 to 0..255. E-4's time does not depend on it. The picture a person judges at Prompt 55 does. A `VideoDecoderConfig` takes a `colorSpace` that overrides what the browser read; `video-source.ts` gives none, and `ClipInfo` has no field to carry the flag | Prompts 55, 57; V3 |
| 31 | **Closed in Prompt 53:** the type takes no argument for a method that has none. It was: **`Client<Api>` of `rpc.ts` asks for one argument on every method,** also on one the protocol gives none: `pool.asr.unload()`, `pool.render.closeSession()` and `pool.render.previewPause()` fail `tsc` with "Expected 1-2 arguments, but got 0" (tried in Prompt 50 with a throwaway file). `pool.asr.unload(undefined)` compiles and does the same. A change of the type is a change of `rpc.ts`, which has 374 lines | Prompts 53, 54 |
| 32 | **A `VideoFrame` made from a buffer of RGBA bytes is encoded with full-range brightness, and the stream does not say so** (entry of Prompt 51, TE-3): white is stored as 255, not 235, and a player shows the file with too much contrast. A frame made from the canvas is right. Any path that hands the encoder pixels from a buffer, the readback of TS §21.4 or the Canvas2D backend of the contingency, must give its frames a `colorSpace` and be checked against a file of method A | V5; any contingency of TS §19 or §21 |
| 33 | **A page under Playwright is never hidden.** `document.visibilityState` stays `visible` with another tab in front and with the window minimised, and Playwright starts Chrome without the throttling of hidden pages (entry of Prompt 51). A test of what the app does when the tab hides (`PreviewPlayer` pauses, Prompt 55) cannot hide the page: it has to send the `visibilitychange` event itself, or drive an ordinary Chrome over its DevTools port, as TE-3 did. **Export times on D1 rise over a session,** from 28 s to about 40 s for the same clip: a reading taken late in a long run is not the machine's best, and the bench's ten runs will show it | Prompts 55, 57, 59 |
| 34 | **What Prompts 53 to 55 build on** (entry of Prompt 52). The stores are `useClipStore`, `usePreviewStore`, `useExportStore`; a use-case calls the actions each file exports, a component reads with the hook and writes nothing. Clip: `begin(clipId, source)`, `accepted(info)`, `rejected(reason)`, `failed(failure)`, `processing(stage, waitingModel)`, `pushFeed(line)`, `ready({ transcript, prosody, events, out48 })`, `noSpeech()`, `reset()`, and `noteFailure(failure)` for a failure that leaves the clip `ready`. Export: `start(exportId)`, `blocked()`, `clear()`, `progress(done, total)`, `encoded()`, `finalized()`, `saved()`, `fail(failure)`, `reset()`, `setUnavailable(flag)`; `start` clears `unavailable`. Preview: `attached()`, `play()`, `pause()`, `ended()`, `lock()`, `unlock()`, `detached()`; `lock` and `detach` are legal in every state. **An illegal action throws `IllegalTransitionError` in a development build and in a test,** and is reported and ignored in production: a use-case must not call `failed` on a `ready` clip, `begin` on a clip that is not `idle`, or `start` on an export that is `done` (call `reset()` first). `forImport()` and `forExport()` of `state/blockers.ts` return a `BlockerCode` or `null`; `messages.blockers` has the words of four codes and is typed by its keys, not by `BlockerCode`, which `copy/` may not import. `entitlement-repo.ts`: `get()` gives `{ token, storedAt }` or `undefined`, `put(token)` | Prompts 53, 54, 55 |
| 35 | **What Prompts 54 to 57 build on** (entry of Prompt 53). `usecases/control-preview.ts`: `attach(canvas, out48)` (the canvas is handed over for good: an element can be attached once), `play()`, `pause()`, `detach()`, `lockForExport()`, `unlockAfterExport()`. `play()` resolves when the preview is playing; it needs a click before it the first time, or the browser keeps the `AudioContext` silent and `play()` does not resolve. All six do nothing when there is nothing to do, and none throws for a failure of the worker: that is stored with `noteFailure` on the clip store. `start-export.ts` calls `lockForExport()` before `exportClip` and `unlockAfterExport()` after it, whatever the outcome. **The worker refuses `exportClip`, `closeSession` and every other call while `previewPlay` runs** (`E_INTERNAL`, `Busy`): pause or lock first. `preview_played` is tracked in `control-preview.ts`, once for each `attach`. On D1 the loop drew 30 frames a second with under 1% late and at most 17 ms from the audio's time. `rpc.ts` has 377 lines and `render.worker.ts` 372 | Prompts 54, 55, 56, 57 |
| 36 | **`web/src/workers/render/video-source.ts` has 399 lines of the 400** since D-70 was built, and it is now the longest source file. V4 adds `prefetch` and the seek to this file: room must be made first, and the place to take it from is the handling of the colour range, which could move into a file of its own with a line in the tree of TS §5. **An export of a full-range clip is now less contrasty than every export before 2026-10-10:** a check that compares a new file with an old one, or with a number written down before that date, sees the difference. `DemuxerHandle` has five methods; a test double of it needs `video_full_range()` | V4; Prompts 56, 57 |
| 37 | **What Prompts 55 to 58 build on** (entry of Prompt 54). `importClip(file, source)` resolves when the clip is accepted, rejected or failed, and the pipeline runs on after that: a caller reads the clip store, not the promise. `importSampleClip()` fetches 35 MB before the clip store leaves `idle`, and says nothing meanwhile; a second call joins the first. `dismissClip()` does nothing for a clip that is `idle`, being imported or being processed, and nothing while an export runs; it stops a preview that plays, closes the session, removes the clip's directory and resets an export that is over. `startExport(clipId)` resolves when the export is over, however it ended; with no token it sets `unavailable` and changes nothing else. All of them return without a word when a blocker stands: the component shows the blocker's copy from `forImport()` or `forExport()`. **While the model downloads** the clip is `processing` at `probe_audio` with `waitingModel: true`. **A promise of these calls that rejects is a fault of the code,** never a failure of the device: those are in the stores. **For the suites:** `exports/` holds `<exportId>.mp4` and an empty directory `tmp/`, and does not exist after an export that was refused; `stage_timing` for `asr` is sent for a clip without speech too; `model_download` is sent only when files were downloaded; one `preview_played` for each `attach`. `git grep -n "V3: \|V7: " -- web/src/usecases` prints 5 lines, one of them V1's | Prompts 55, 56, 57, 58 |
| 38 | **What Prompts 56 to 58 build on** (entry of Prompt 55). **Test ids:** `drop-zone`, `editor`, `feed` (an `<ol>`, one `<li>` for each line), `preview-canvas`, `export-status` (the line under the export button: a blocker's words or the unavailable message), `export-progress`, `export-stage`, `export-done`, `export-failed`, `clip-rejected`, `clip-failed`. The model panel has none: find it by `messages.modelDownload.body(sizeMb)` and its bar by the role `progressbar`. **Buttons by name:** `messages.preview.play`, `.pause`, `.replay` (match exactly: "Play" is also the start of "Play again"), `messages.export.button`, `messages.editor.startOver`, `messages.dropZone.sampleButton`. **The feed is gone once the clip is `ready`,** and "Found: $12k" is on the page for half a second or less: a case must record the list with a `MutationObserver` installed before the page loads, not look for the line. The stage labels of an export need the same; "Saving" lasts a moment. **A drop on `/` moves to `/app`; a drop on `/app` stays there.** The drop zone also holds a hidden `<input type="file">`. **`ready` has a "start over" since D-71** (the entry of 2026-10-10), disabled while an export runs. **To make Chrome unsupported in a test,** define `Navigator.prototype.userAgentData` with `mobile: true` in an init script; Playwright's `isMobile` and `--disable-blink-features=WebGPU` do not do it. The first press of play must be a real click | Prompts 56, 57, 58 |
| 39 | **What Prompts 57 to 59 build on** (entry of Prompt 56). **Helpers** of `web/tests-e2e/helpers/fixtures.ts`: `routeAssets(page, o?)` gives `{ requests, modelRequests() }`, a request being `{ method, url, path, headers, hasBody, answer }` with `headers` a list of `{ name, value }` in lower case; `dropClip(page, referenceClip())`; `await ensureModelCached(context)` before the page opens, which writes the model into OPFS and starts no download (the whole case of the second session takes 11 s with it); `opfsList(page, dir)`, sorted names, `[]` for no directory; `sourceDurationMs(fixture)`, which starts `ffprobe` from PATH; `modelManifest`, `assetBaseUrl()`, `sampleClipPath()`. **A case begins:** `installFakeApi(page)`, `routeAssets(page)`, `page.clock.install()`, `page.goto("/app")`, wait for the test id `drop-zone`. **Analytics:** `await flushAnalytics(page)`, then `expect.poll` on `api.eventsOf(name)`; do not flush twice without waiting for what the first sent. With the clock installed time still runs on by itself. **`pnpm e2e:media`** runs `media-setup` and then `media`, one worker; the suites of Prompt 57 are matched by name already (`pipeline-preview.spec.ts`, `export-creator.spec.ts`). **A temporary `zz-*.spec.ts` matches no project:** bring a temporary `web/zz-pw.config.ts` that spreads the real configuration and sets `projects: [{ name: "zz", testMatch: "zz-*.spec.ts" }]`, and run `playwright test --config zz-pw.config.ts`. **Each test has its own browser context,** so its own OPFS: a case that needs the model calls `ensureModelCached` itself. **`fixtures/.cache/`** holds each file under its path on the asset host. A quota for a case is set with `Storage.overrideQuotaForOrigin` through `context.newCDPSession(page)`. `bench/device-bench.ts` (Prompt 58) is matched by the project `bench` already | Prompts 57, 58, 59 |
| 40 | **What Prompts 58 and 59 build on** (entry of Prompt 57). `pnpm e2e:media` is 30 cases and takes 9.7 minutes on D1, headless, one worker; the files run in the order of their names: `export-creator`, `model-download`, `pipeline-preview`. **It needs on PATH:** `python` with the verifier's requirements, and `ffprobe` and `ffmpeg`; on D1 the Python of the SAB must come first. **It needs a build with the test key:** on a plain build `export-creator` fails at "Creator export" with `E_ENTITLEMENT_INVALID` and 8 cases do not run; `model-download` and `pipeline-preview` pass on either. **Nothing may read the reference clip while a test file is loaded:** on a machine without `testclips/` the clip is in the cache only once the setup project has run, and Playwright loads every file before that (the first run of `e2e-media`, 2026-10-10). Read it in a test or in `beforeAll`. **The cases of `pipeline-preview` and of `export-creator` are one group each, in order, in one page:** one that fails stops the rest of its group, and a retry runs the group again from the drop. **An exported file is written under the system's temporary directory** and removed by the suite; nothing is written under `web/test-results` but what Playwright writes itself for a failed case, which can hold text of the page. Helpers added: `verifyMp4(file, profile, expectedDurationMs)` gives `{ status, output }`; `opfsSize(page, file)` gives a number or `null`. For the bench: a page with `installFakeApi`, `routeAssets`, a dropped clip, `seedEntitlement(page, mintEntitlementToken())` and a click on `messages.export.button` ends in a `download` event; the timings are in `api.eventsOf("stage_timing")`, `"pipeline_done"` and `"export_done"`, which arrive within 10 s of the export's end | Prompts 58, 59 |
| 41 | **What Prompts 59 and 60 build on** (entry of Prompt 58). **The bench:** `BENCH_DEVICE=d1 pnpm bench:device`, in a terminal with the test key exported and after a build made with it; headed; one warm-up and ten runs; 24.5 minutes on D1; writes `bench/results/d1-<yyyy-mm-dd>.json` and prints the total's median and p90. It needs `python` nowhere, and `ffprobe` on PATH. On D1 the check run gave `asr` 83,948 ms, `render_encode` 35,443 ms, total 120,778 ms, peak memory 936 MB. **`ci.yml`:** the jobs are `ci`, `e2e-media`, `deploy-api`, `deploy-web`; the test public key is the one line `E2E_TEST_PUBLIC_KEY` of its `env`; the artifact `web-dist` is the keyed build; a deploy stops at the guard of step 14 when `.vercel/output` holds the key or holds nothing. **Prompt 59's check "no `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` in Vercel" is what makes the guard pass:** `vercel pull` brings the project's variables into the build. **TE-10 is half open** (open item 14): `e2e-media.yml` has never run. **The root `package.json` is `"type": "module"`:** a `.js` file added at the root is an ES module. **`media` and `bench` record no trace;** `E2E_CHROME_ARGS` gives Chrome its arguments for both | Prompts 59, 60 |

---

## 2026-10-08 - Prompt 31: preflight, branch, V2 documents, D-65, crate skeletons

**Branch.** `v2-build`, created from `main` at the baseline. The pending documents were committed first and alone, as `V2: documents` (`28d010e`): `docs/v2/v2implementation.md`, `v2implementation-notes.md`, `v2buildguide.md`, `coding-promptsv2.md`, `docs/v1/v1changelog.md` and `fixtures/speech/README.md`. `main` is untouched: a push to it deploys.

**Tools (the audit of G 0.1).**

| Tool | Version |
|---|---|
| `bash` | `/usr/bin/bash` (Git Bash) |
| Git | 2.48.1.windows.1 |
| Rust, Cargo | 1.99.0, with the target `wasm32-unknown-unknown` |
| `wasm-bindgen` CLI | 0.2.129 |
| `wasm-opt` | version 133 |
| `cargo-deny` | 0.20.2 |
| Node, pnpm | 24.19.0, 10.15.0 |
| Python, NumPy | 3.13.5, 2.5.3 (with the Python 3.13 folder first in PATH) |
| `ffmpeg`, `ffprobe` | 9.0.2 |
| TypeScript, Vite, Playwright | 6.0.3, 8.3.3, 1.63.0 |
| Docker, `psql` | 28.3.2, 18.4 |
| `gh` | Not installed (open item 4) |

**The baseline gate, on the branch before any V2 change.** `pnpm install --frozen-lockfile`, then `pnpm check`, `pnpm test`, `pnpm build` and `pnpm e2e`: all green. 170 Rust tests, 76 Vitest tests, 12 Playwright cases.

**The preconditions of §1A.**

| # | Checked how | Result |
|---|---|---|
| 1 | A temporary Playwright case on the deployed page: `fetch` of an uploaded file with `Range: bytes=0-1023`. Also `curl` from the terminal | 206, `Content-Range: bytes 0-1023/10376359` |
| 2 | `E2E_BASE_URL=https://offcut-one.vercel.app`, the `@smoke` case | 1 passed |
| 3 | The same temporary case: `crossOriginIsolated` | `true` |
| 4 | `pnpm -C web exec tsc --noEmit` | No output |
| 5 | `grep` for the four names in `encoders.ts`; the imports of `capability.ts` | 4 found; `capability.ts` imports `VIDEO_ENCODE_LADDER` and `AAC_ENCODE_CONFIG` |
| 6 | `pnpm e2e` locally; the last `ci` run on `main` | 12 passed; green on `322c7d3` |
| 7 | "twelve thousand dollars" in `fixtures/speech/README.md` | Found (2 lines). Prompt 43 reads how the recognizer writes it |
| 8 | `ls bench/results` | `.gitkeep` |
| 9 | `pnpm check` | Green |
| 10 | The four-command gate; `ci` on `main` | Green; green |
| 11 | The same temporary case: the IndexedDB database of the deployed page | `offcut`, version 1, eight stores, `entitlement` among them |
| 12 | The base64 value of §1A item 12 | Decodes to 32 bytes. Whether the key in Render was replaced since is open item 2 |
| 13 | - | **Open**, the human's (open item 3) |
| 14 | The audit above | Holds on the development machine. **Open** for R1 (open item 3) |

Also checked: `/api/v1/healthz` on Render and through Vercel both report `322c7d3`; `node scripts/check-headers.mjs https://offcut-one.vercel.app` passes, seven headers value for value. The `curl` with `Origin: http://localhost:4173` gets no `Access-Control-Allow-Origin` (open item 1).

Boxes 1 to 12 of §1A are ticked in the plan. Boxes 13 and 14 are not.

**Added.**

| File | Content |
|---|---|
| `crates/offcut-{mp4,dsp,text,detect,entitlement,scene,render,wasm-render}/Cargo.toml` | The `[package]` header of `offcut-types` and `[lints] workspace = true`. No dependency. `offcut-wasm-render` is a `cdylib`; `offcut-entitlement` declares the feature `sign`, empty until V6 |
| The eight `src/lib.rs` | One line break each (known issue 1) |
| `docs/v2/experiments.md` | The header, the form of an entry, and the list of the experiments V2 runs, none run yet |
| `docs/v2/v2changelog.md` | This file |

**Changed.**

| File | Change |
|---|---|
| `Cargo.toml` | The eight new members; the list is now one path per line. 12 members |
| `Cargo.lock` | Eight package entries, no dependency |
| `docs/technicalspec.md` §5 | The tree gains `crates/offcut-mp4/src/mux_boxes.rs` (D-55), `web/tests-e2e/tsconfig.json` (D-63) and the six files of `docs/v2/` (D-46) |
| `docs/v2/v2implementation.md` | D-65 added to §2: R2 is not measured in V2 and R1 is the only reference machine, with the reason and the cost. §0 says "D-18 to D-65". D-46 names all six files of `docs/v2/`, and §4 says "six". Every line that named R2 is reworded for R1 (list below). Boxes 1 to 12 of §1A ticked |

The lines reworded for R1: §1A boxes 13 and 14 and the row for V1 known issue 30; §3.1 (two rows); §3.4 (`BENCH_DEVICE`); §5 (S15); §22.2 (`e2e:device`); §23.1 (two rows); §24.1 (E-3, TE-3, TE-4, E-4; TE-4 now passes when a ladder entry is supported on R1); §25.3 (one box); §26 (three boxes). The edit was made by a script that refuses a replacement unless its old text occurs exactly once: 35 of 35 applied. The file keeps its CRLF line endings.

**Differs from the prompt or the guide.**

- **More R2 lines than the prompt lists.** Prompt 31 and G 1.1 name §1A, §5, §23.1, §24.1, §25.3 and §26. The plan also named R2 in the carried-over table of §1A, in §3.1, §3.4 and §22.2. Those five lines were reworded too; without them the prompt's own check (`grep -nw "R2"`) could not pass.
- **`lib.rs` is not 0 bytes** (known issue 1).
- **The documents were committed on the branch, not on `main`.** G 0.2 commits them on `main` first. The prompt does it this way, and it keeps `main` at the deployed commit. The empty commit "V2 S0" of the guide's Milestone 0 was not made: the commit `V2: documents` marks where V2 starts.
- **The test key pair of G 0.5 was not generated.** Prompt 46 generates it where it is first written down (the prompts file says so).
- `v2implementation-notes.md` still says "D-18 to D-64"; it describes the plan as it was on 2026-10-08 and was left as written.

**Checked.**

- `cargo metadata --no-deps --format-version 1` lists 12 packages.
- `grep -nw "R2" docs/v2/v2implementation.md` shows two lines: box 13 of §1A ("R2 is not used in V2") and D-65.
- `node scripts/check-file-tree.mjs`: 138 files in the tree of TS §5, 12 crates within the graph of TS §7. The script already held all 12 crates, and `deny.toml` already named them.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green; the frozen-file diff against the baseline is empty. **170 Rust, 76 Vitest, 12 Playwright**, the same as the baseline.
- The temporary case `web/tests-e2e/zz-pre.spec.ts` answered `/api/v1/events` itself, so it added no row to production. It was deleted before the gate.

**Not done.** Nothing was pushed. No dependency, no logic and no rule change: those are Prompt 32 and later.

## 2026-10-08 - Prompt 32: tree, purity and ban rules; test tooling; lint edges

**REOPENED V1 CONTRACT: `web/eslint.config.js`** (D-27, D-58, D-59). Nothing in it was loosened: the diff adds lines and removes none.

**Changed.**

| File | Change |
|---|---|
| `scripts/check-file-tree.mjs` | `ALLOWED_CRATE_DEPS["offcut-scene"]` gains `offcut-text` (D-28). `PURE_CRATES` gains `offcut-mp4`, `offcut-dsp`, `offcut-text`, `offcut-detect`, `offcut-scene` and `offcut-entitlement`: the `cargo tree` check now runs on eight crates |
| `deny.toml` | The `offcut-text` ban gains the wrapper `offcut-scene` (D-28). Nothing else |
| `.gitignore` | `web/public/ort/` and `fixtures/.cache/` |
| `package.json` | Dev-dependency `@playwright/test` at exactly `1.63.0`, the version `pnpm-lock.yaml` resolves for `web` (D-63). `check` runs a second `tsc --noEmit -p tests-e2e/tsconfig.json` after the first |
| `web/package.json` | Dev-dependency `@types/node` `^24.19.1`, the Node major of `engines` (D-63). Resolved: 24.19.1 |
| `web/tsconfig.json` | `include` is `src` alone (see "Differs") |
| `web/eslint.config.js` | 15 entries, listed below |
| `.github/workflows/ci.yml` | Step 6 gains the second `tsc` pass (see "Differs") |
| `docs/v2/v2implementation.md` | D-63, §4, §22.3 and §22.5 say what was built (5 replacements, each applied once) |

**Added.** `web/tests-e2e/tsconfig.json`: extends `../tsconfig.json`, sets `types` to the two of the app plus `node`, and includes `tests-e2e`, `playwright.config.ts`, `vite.config.ts`, `vitest.config.ts` and `../bench`.

**The 15 entries of `web/eslint.config.js`.**

| # | Entry | Kind |
|---|---|---|
| 1 | D-27 a: `usecases/import-clip.ts` may import `net/asset-fetch.ts` | Layer policy |
| 2 | D-27 b: `workers/**` may import `persistence/opfs.ts` | Layer policy |
| 3 | D-27 c: `workers/render.worker.ts` may import `config/entitlement-public-key.ts` | Layer policy |
| 4 | D-27 d: `models/**` may import `state/model-store.ts` and `config/model-manifest.json` | Layer policy |
| 5 | D-27 e: `models/model-manager.ts` may import `persistence/db.ts` | Layer policy |
| 6 | D-27 f: `state/**`, `models/**` and `usecases/**` may import `workers/protocol.ts`, types only | Layer policy |
| 7 | D-58: `usecases/import-clip.ts` may import `usecases/run-pipeline.ts` | Layer policy |
| 8 | D-58: `usecases/start-export.ts` may import `usecases/control-preview.ts` | Layer policy |
| 9 | D-59: `persistence/opfs.ts`, one cast to `Bytes`, in `toBytes()` | Cast override |
| 10 | D-59: `models/download.ts`, one cast to `Bytes`, in `toBytes()` | Cast override |
| 11 | D-59: `models/model-manager.ts`, one cast to `Bytes`, in `toBytes()` | Cast override |
| 12 | D-59: `state/model-store.ts`, one cast to `Bytes`, in `const ZERO_BYTES = 0 as Bytes` | Cast override |
| 13 | D-59: `usecases/import-clip.ts`, one cast to `ClipId`, in `newClipId()` | Cast override |
| 14 | D-59: `usecases/start-export.ts`, one cast to `ExportId`, in `newExportId()` | Cast override |
| 15 | D-59: `usecases/control-preview.ts`, one cast to `TimeMs`, in `toTimeMs()` | Cast override |

Entries 1 to 8 are eight objects at the end of `layerPolicies`. Entries 9 to 15 are the seven rows of the table `MINTING`, each of which becomes one configuration object.

**How a cast override works.** It does not switch the brand-cast rule off for its file. It replaces the rule's selectors by three: a cast to any other unit or id type is refused; a cast to the file's one type is refused unless it is the expression of the last `return` of the named helper (in `model-store.ts`: unless it is the top-level `const ZERO_BYTES = 0 as Bytes`); an angle-bracket cast is refused. So a second cast in the file fails, also inside the helper. `newClipId` and `newExportId` are the names §19.2 and §19.5 give; `toBytes`, `toTimeMs` and `ZERO_BYTES` were chosen here (known issue 4).

**Differs from the prompt, the guide or the plan.**

- **`web/tsconfig.json` includes `src` alone; `vite.config.ts` and `vitest.config.ts` moved to `web/tests-e2e/tsconfig.json`.** D-63 and the prompt only take `tests-e2e` out of the first. Done that way, the drill of G 1.4 did not fire: with `@types/node` installed, `console.log(process.cwd())` in `web/src/main.tsx` passed `tsc`. The cause, read with `tsc --explainFiles`: `vite/dist/node/index.d.ts` holds a reference to Node's types, and the two configuration files import Vite, so one program with `src` gave app code every Node global. Before V2 the reference found nothing, because `@types/node` was not installed. With the two files in the second program the drill fires, and the sentence of §22.3, "App code therefore never sees a Node global", is true. The test files under `src` import `vitest` and do not bring Node's types in (measured: `src` alone, tests included, refuses `process`). D-63, §4 and §22.3 of the plan are corrected.
- **`ci.yml` step 6 runs the second `tsc` pass.** Neither the prompt nor §22.5 asks for it. CI calls `tsc` directly, not `pnpm check`, so without the step CI would no longer type-check `tests-e2e`, nor now the three configuration files. One step added; §22.5 says so.
- **The drill "`persistence/opfs` imported from `state/`" (G 1.5)** does not reach the layer rule as the guide writes it: `opfs.ts` does not exist before Prompt 37, and the plugin classifies only an import it can resolve (known issue 5). As written, ESLint still fails, on `@typescript-eslint/no-unsafe-assignment`. With a one-line stub `opfs.ts` in place, `boundaries/dependencies` refuses the import. Both runs are in the table below.
- **`engines` in the root `package.json`** was put back on one line after `pnpm add` had spread it over three.

**The nine drills.** Each edit was undone after the rule fired; a frozen file was restored from a copy.

| # | Temporary edit | What fired |
|---|---|---|
| 1 | `crates/offcut-mp4/src/zz.rs` | `check-file-tree`: "not in the file tree of TS §5" |
| 2 | `offcut-text` as a dependency of `offcut-mp4` | `check-file-tree`: "offcut-mp4 may not depend on offcut-text (TS §7)" |
| 3 | `rand = "*"` in `offcut-entitlement` | `check-file-tree`: "resolves rand" and "resolves getrandom"; `cargo deny check bans`: "crate 'rand = 0.10.3' is explicitly banned" |
| 4 | `web-sys = "*"` in `offcut-scene` | `cargo deny check bans`: "crate 'web-sys = 0.3.106' is explicitly banned"; `check-file-tree` too |
| 5 | `console.log(process.cwd())` in `web/src/main.tsx` | `tsc --noEmit`: TS2591, cannot find name `process` (after the change above) |
| 6 | The same line in `web/tests-e2e/helpers/fixtures.ts` | Nothing, as intended: `tsc --noEmit -p tests-e2e/tsconfig.json` stays clean |
| 7 | `1 as Bytes` in `web/src/ui/pages/LandingPage.tsx` | `no-restricted-syntax`: the brand-cast rule |
| 8 | `import type { AppFailure } from "../../workers/protocol"` in the same file | `boundaries/dependencies`: no policy from `ui` to `workers` |
| 9 | `import { paths } from "../persistence/opfs"` in `web/src/state/capability-store.ts` | With a stub `opfs.ts`: `boundaries/dependencies`: no policy from `state` to `persistence`. Without it: `no-unsafe-assignment` |

**Both sides of the 15 entries, proven now and not when each file is first written.** A script wrote 12 stub files under the real names (the entries are keyed by name), linted them, changed one thing at a time, and deleted every stub.

- Positive: the 12 stubs, which use every new edge and each allowed cast once, lint with 0 problems.
- Negative, each refused by the rule named: a second cast outside the helper; a second `return` with a cast that is not the last statement; a nested cast inside the helper; another brand inside the helper; the helper's name in a file that is not listed; `ZERO_BYTES` with a value other than 0; a second constant; an angle-bracket cast (all `no-restricted-syntax`). Another use-case importing `asset-fetch`; `state` importing `persistence/opfs`; `media.worker.ts` importing the public key; `models` importing another state file; `download.ts` importing `persistence/db`; a value import of `workers/protocol` from `state`; a use-case pair that is not one of the two; an allowed pair the other way round (all `boundaries/dependencies`). 16 of 16.

**Checked.**

- `git check-ignore web/public/ort/x fixtures/.cache/x testclips/x` echoes the three paths.
- `git diff web/eslint.config.js`: 88 lines added, none removed. `web/eslint.config.js` is 358 lines, under the limit of 400.
- `node scripts/check-file-tree.mjs`: 139 files in the tree of TS §5; 12 crates within the graph; the eight pure crates resolve no browser or randomness crate.
- `pnpm --filter web exec tsc --noEmit -p tests-e2e/tsconfig.json` type-checks `tests-e2e/**`, `playwright.config.ts`, `vite.config.ts` and `vitest.config.ts`. `playwright.config.ts` was type-checked by nothing before.
- The gate: no `zz-` file; no test key in the environment; `pnpm check` (with two `tsc` passes), `pnpm test`, `pnpm build`, `pnpm e2e` green; the frozen-file diff against the baseline is empty. **170 Rust, 76 Vitest, 12 Playwright**, the same as the baseline.

**Pinned versions.** `@playwright/test` 1.63.0 (root, exact); `@types/node` 24.19.1 (`web`, `^24.19.1`; 26.6.4 is the latest release, and 24 is the major of `engines`).

**Not checked.** The `ci` run: nothing was pushed. The new step of `ci.yml` runs for the first time on the first push of `v2-build`, after Prompt 33. No YAML linter is installed; the step was read by eye.

## 2026-10-08 - Prompt 33: config contracts, reference clip, worker probe

**Added.** `web/src/config/entitlement-public-key.ts`: `ENTITLEMENT_PUBLIC_KEYS`, a `readonly Uint8Array[]`. It holds the production key, a base64 literal (the value of §1A item 12), and the test key only when `env.entitlementTestPublicKey` is set. No file imports it yet; the render worker is its one reader (D-27 c, Prompt 50).

**Changed.**

| File | Change |
|---|---|
| `web/src/config/env.ts` | One new field, `entitlementTestPublicKey: string \| null`: the value of `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` when it is set and not empty, else `null`. A value that is not base64, or does not decode to 32 bytes, throws when the module loads |
| `fixtures/speech/README.md` | The row of the reference clip gives the video-stream duration, 74,705 ms, the codecs, the size and the two white frames |

**The production key.** The literal is the public half of the seed that `.env.deploy` holds as `PROD_ENTITLEMENT_SIGNING_KEY`: derived again in a script that printed only "equal" or "not equal"; equal. Whether the key in Render has been replaced since 2026-10-08 is not known to the agent (open item 2). No seed was printed or written.

**The secret scan.** gitleaks 8.18.4, the version of `ci.yml`, was fetched into a scratch folder (the Windows archive; its SHA-256 equals the published checksum) and run. On the new file it reported one finding, the line of the literal. The line now ends with `// gitleaks:allow`, and the scan reports none. It was added before the line was ever committed, which matters: see known issue 8. The history of the branch, scanned the way CI does it: 48 commits, no finding.

**The reference clip (G 1.7).** `testclips/speech_scriptA_landscape_720p.mp4`:

| Read | Value |
|---|---|
| Video | `h264`, 1280x720 |
| Audio | `aac`, 48,000 Hz, 2 channels |
| Video-stream duration | 74.705033 s: 74,705 ms |
| Size | 35,201,023 bytes (limit 40,000,000) |
| Frames 150 and 1950 | `YAVG` 255 and 255 (at least 230 asked) |

**The worker probe (G 1.8), under the production CSP.** A temporary module worker, `web/src/workers/zz-probe.worker.ts`, started by one line in `main.tsx`; a production build served by `vite preview` with the headers of `web/vercel.json`; the console line read by a temporary Playwright case in Chrome, headless.

| Probe | Result |
|---|---|
| `webgpuCanvas` (adapter, device, a `webgpu` context on a 1080x1920 `OffscreenCanvas`, configured) | `true`; adapter `intel / gen-12lp` |
| `h264` (`avc1.640028`, 1080x1920, 8 Mbit/s, 30 fps) | `true` |
| `aac` (`mp4a.40.2`, 48 kHz, stereo, 160 kbit/s) | `true` |
| `opfsSync` (a sync access handle, opened and closed) | `true` |
| `crossOriginIsolated` on the page | `true` |
| Console lines starting "Refused to"; page errors | None |

No `false` and no error key: the gate of this prompt is passed and nothing goes to `experiments.md`. The worker file, the line in `main.tsx` and the Playwright case were removed the same hour; `git grep -n "zz-probe" -- web` prints nothing.

**Browser check (dev).** `vite` on port 5173, started three times; the page imported `/src/config/entitlement-public-key.ts` and returned the key lengths.

| `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` | Result |
|---|---|
| Not set | `[32]` |
| The base64 of 32 random bytes (made for the run, kept nowhere) | `[32, 32]` |
| `abc` | The import rejects: "VITE_ENTITLEMENT_TEST_PUBLIC_KEY must decode to 32 bytes" |

**Differs from the prompt or the guide.**

- **`// gitleaks:allow` was added before the push, not after a failed `ci` run.** The prompt adds it only "if the secret scan names the public key line". The scan was run locally and does name it.
- The probe worker of the guide was given `try`/`catch` around the two encoder questions and one more value, the adapter's vendor and architecture. It is temporary code either way.
- The probe build was `pnpm build:vite`, not `pnpm build`: no Rust file had changed. The gate ran the whole `pnpm build` afterwards.

**Checked.**

- G M-1: 12 workspace packages; `pnpm check` and `pnpm test` green; `git grep -n "zz-probe" -- web` empty; the frozen-file diff empty.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green. **170 Rust, 76 Vitest, 12 Playwright**, the same as the baseline.

**"Done when".**

- [x] The probe printed `webgpuCanvas`, `h264`, `aac`, `opfsSync` all `true` under `vite preview`, with no line starting "Refused to".
- [x] The clip reads 1280x720, H.264 and AAC, 48 kHz stereo, 74,705 ms, at most 40,000,000 bytes, two white frames.
- [x] `git grep -n "zz-probe" -- web` is empty; G M-1 passes; the draft pull request is open and `ci` is green (pull request #2, run of 2026-10-08 on `3ad4c06`; see the entry "The first push").

## 2026-10-09 - Prompt 34: `offcut-mp4`: errors, reader, boxes, sample tables

**Started before the `ci` run of Prompt 33 was read.** The prompts file asks for every "Done when" box of the previous prompt first. The push is the human's and had not been made; the human asked for Prompts 33 and 34 in one sitting. Open item 4 stands.

**Added.**

| File | Content |
|---|---|
| `crates/offcut-mp4/src/reader.rs` | `RandomAccess` (TS §15.1) and `MemReader`, the file in memory that tests use: a read past `len()` is `IoError::OutOfBounds`. `Cursor`: big-endian fields from bytes already in memory; a read past their end is `Truncated` |
| `crates/offcut-mp4/src/boxes.rs` | `BoxHeader`, `read_header`, `children` (§6.3); `child` and `descend`; the readers `read_ftyp`, `read_mvhd`, `read_mdhd`, `read_tkhd`, `read_hdlr`, `read_stsd` with their structs `Ftyp`, `TimeHeader`, `Tkhd`, `SampleEntry` |
| `crates/offcut-mp4/src/sample_table.rs` | `Sample`, `TrackTable`, `RawTables`, `resolve` (§6.4, steps 1 to 6), `MAX_RESOLVED_SAMPLES` (20,000); the readers `read_stts`, `read_ctts`, `read_stsc`, `read_stsz`, `read_stco`, `read_co64`, `read_stss`, `read_elst` with `StscEntry`, `Stsz`, `Edit` |

**Changed.**

| File | Change |
|---|---|
| `crates/offcut-mp4/src/lib.rs` | The three modules; `IoError` (3 variants), `ContainerError` (6), `MuxError` (4), as §6.1. `Malformed` and `Unsupported` carry a `&'static str`, so they cannot carry bytes of the file |
| `crates/offcut-mp4/Cargo.toml` | `offcut-types`, `thiserror`; dev `proptest` with the feature `std` |
| `Cargo.toml` | `[workspace.dependencies]`: `proptest = { version = "1.11.0", default-features = false }` |
| `deny.toml` | Two wrappers, both third-party crates (G 2.1): `proptest` for `rand`, `rand_core` for `getrandom`. `cargo deny check` named `rand 0.9.5` and `getrandom 0.3.4`; their direct parents were read with `cargo tree -i` |
| `docs/v2/v2implementation.md` | §6.2 and §6.3 say what was built (2 replacements, each applied once) |

**Pinned.** `proptest` 1.11.0, the latest release (`cargo search`), without its default features: no fork, no timeout, no temporary files. `thiserror` was already in the workspace (2.0.21).

**The bounds, and where each is.**

| Bound | Where |
|---|---|
| A box must end at or before its limit, and the limit is never past the end of the file | `read_header` |
| At most 4,096 children in one container: `Malformed("children")` | `children` |
| A path of more than 16 nested boxes: `Malformed("depth")` | `descend` |
| A table cannot have more records than its box has bytes for: `Malformed(<box>)`, checked before the records are read | `load_table` |
| A header box is read up to the bytes its fields need, never whole | `load` |
| An `avcC` or `esds` box over 1 MiB: `Malformed` | `load_descriptor` |
| A track of more than 20,000 samples is left unresolved: no per-sample memory, and no error | `resolve`; `read_stsz` does not load its size table |

**Decided here, where the plan gives a name and no shape, or is silent.** Each is the smallest choice that keeps Prompt 35 able to write `demux.rs` and `probe.rs` as §6.5 and §6.6 describe them.

- **`BoxHeader.body` is an offset,** the first byte after the header, like `start` and `end`. `body_len()` gives the length.
- **`RawTables`** (§6.4 names it and gives no fields): `timescale` (from `mdhd`), `movie_timescale` (from `mvhd`; an edit's length is in it), `file_len`, `stts`, `ctts`, `stsc`, `stsz`, `chunk_offsets` (from `stco` or `co64`), `stss`, `elst`. An absent `ctts` or `elst` is an empty list; an absent `stss` is `None`.
- **`SampleEntry`** (§6.3 lists its content): the four-character code; width and height; channel count and sample rate; `object_type`, the object type indication of `esds`, which §6.6 needs to tell AAC from MP3; `config`, the raw `avcC` payload or the `AudioSpecificConfig`.
- **`read_stsd` takes the track's handler type** (`vide`, `soun`). A sample entry does not say which of the two layouts it has; `hdlr` does. For audio it reads the three layouts QuickTime and ISO files use (versions 0, 1 and 2) and finds `esds` directly in the entry or inside `wave`.
- **`read_stsz` takes the largest size table it may load.** The count is always returned.
- **A `ctts` offset is an `i64`:** version 0 is read unsigned and version 1 signed, into one type.
- **Fewer than 8 bytes left in a container end its list of children** without an error: padding, or the terminator QuickTime writes. §6.3 does not name the case.
- **An edit of length 0** shows the media from its start time to its end. §6.4 does not name the case; read literally it would give a track of length 0.
- **An edit's length is converted to the media timescale to the nearest tick.**
- **Lenient where a table is longer or shorter than the track:** a `ctts` that covers fewer samples leaves the rest at `pts = dts`; a number in `stss` past the last sample is ignored. Chunks that run out before the samples do are `Malformed("stsc")`; a size table whose length is not the sample count is `Malformed("stsz")`.

**Differs from the prompt, the guide or the plan.**

- **The readers of the eight sample-table boxes are in `sample_table.rs`, not in `boxes.rs`** as §6.3 lists them. With all fourteen readers `boxes.rs` had 565 lines after `cargo fmt`, over the limit of 400 (TS §29). The tree of TS §5 already describes `sample_table.rs` by those eight boxes, so no file was added. §6.3 of the plan says so now.
- **`RandomAccess` has one provided method more than TS §15.1, `is_empty()`.** Copied verbatim, the trait fails clippy (`len_without_is_empty`), and no lint is switched off to pass a gate. An implementation does not write the method. To copy back into TS §15.1 (Prompt 60).
- **The depth bound is in `descend`,** not in `children`: the signature §6.3 gives `children` has no depth, and `BoxHeader` has no field for one. Nothing in the crate walks boxes recursively, so `descend` is the only place a file could ask for depth.
- **`proptest` is used already,** in one of the seven cases, so the dependency is not dead until Prompt 41: any bytes, any offset and any limit never make `read_header` panic, a header it returns lies inside its limit, and `children` and four readers run on it without a panic.

**Tests (inline): the seven rows of §6.9 for this prompt.**

| Row of §6.9 | Test |
|---|---|
| Header with 32-bit size, 64-bit size, size 0 | `boxes::a_header_with_a_32_bit_size_a_64_bit_size_and_size_0` (also `children` and `descend` on the three boxes, and the depth bound) |
| Box that ends past the limit | `boxes::a_box_that_ends_past_the_limit_is_truncated` (also a size under the header length, a cut header, a 64-bit size of `u64::MAX`, 4,097 children, and the property above) |
| `stts` counts that do not sum to the `stsz` count | `sample_table::stts_counts_that_do_not_sum_to_the_stsz_count_are_malformed` |
| `ctts` version 1 with a negative offset | `sample_table::ctts_version_1_with_a_negative_offset_puts_pts_before_dts` (the box is built in bytes and read with `read_ctts`) |
| No `stss` | `sample_table::without_stss_every_sample_is_a_keyframe` |
| One media edit with `media_time` = 2 frames | `sample_table::one_media_edit_of_two_frames_presents_the_first_frame_at_zero` (also a leading empty edit) |
| Two media edits | `sample_table::two_media_edits_are_unsupported` |

**The drill.** `let _ = v[0];` in the non-test code of `boxes.rs`: `cargo clippy -p offcut-mp4 --all-targets -- -D warnings` fails with "indexing may panic". Reverted from a copy.

**A real file, read with a temporary test (not asked for until Prompt 35; never committed).** The readers and `resolve` on `testclips/speech_scriptA_landscape_720p.mp4`, beside `ffprobe`:

| Read | This crate | `ffprobe` |
|---|---|---|
| Top-level boxes | `ftyp`, `moov`, `free`, `mdat` | - |
| Video samples | 2,246, resolved | 2,246 frames |
| Video keyframes (`stss`) | 10 | 10 |
| Video, presented | 2,241,151 ticks of 30,000: 74.705033 s | 74.705033 s |
| Video edit list | One media edit, `media_time` 2,393; the first frame's `pts` minus the offset is 0, and it is the smallest | Start time 0 |
| `avcC` | 41 bytes, starting `01 4d 40 1f` (Main) | Main |
| Video entry, `tkhd` | `avc1`, 1280x720; identity matrix; enabled | `avc1` |
| Audio samples | 3,493, resolved, every one a keyframe | 3,493 frames |
| Audio, presented | 3,576,841 ticks of 48,000: 74.517521 s | 74.517521 s |
| Audio entry | `mp4a`, 2 channels, 48,000 Hz, object type 0x40, `AudioSpecificConfig` `11 90` | `mp4a`, LC |
| Last video sample | Ends at byte 35,201,023, the length of the file | - |

Every value agrees. The test and its name are gone: `git grep -n zz_real -- crates` prints nothing.

**Checked.**

- `cargo test -p offcut-mp4 --lib`: 7 passed. `cargo clippy -p offcut-mp4 --all-targets -- -D warnings`: clean.
- `cargo deny check`: advisories, bans, licenses, sources ok. `node scripts/check-file-tree.mjs`: 143 files in the tree, and `offcut-mp4` checked as a pure crate (its normal dependencies resolve no browser or randomness crate; `proptest` is a dev-dependency).
- Lines above the test module: `boxes.rs` 392, `sample_table.rs` 387, `reader.rs` 97, `lib.rs` 54.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green; the frozen-file diff against the baseline is empty. **177 Rust, 76 Vitest, 12 Playwright** (Rust was 170; the seven are new).

**"Done when".**

- [x] `cargo test -p offcut-mp4 --lib` and `cargo clippy -p offcut-mp4 --all-targets -- -D warnings` pass; the seven rows are covered.
- [x] `cargo deny check` and `node scripts/check-file-tree.mjs` pass with `offcut-mp4` now checked as a pure crate.

**Not checked.** `ci`: nothing was pushed (open item 4). `cargo deny` and clippy ran on Windows only.

## 2026-10-09 - The first push: draft pull request #2, `ci` green

**Done by the human.** `git push -u origin v2-build` and a draft pull request, #2, `v2-build` into `main`. The first attempt ran in `sh2clips`, which is not a repository; the repository is `sh2clips/offcut`.

**Read by the agent** (the public API of GitHub; `gh` is not installed).

| Read | Result |
|---|---|
| `origin/v2-build` | `3ad4c06`, the commit of Prompt 34, equal to the local branch |
| Pull request #2 | Open, draft |
| The `ci` run on `3ad4c06` | Success. Job `ci`: 2.7 minutes. Jobs `deploy-api` and `deploy-web`: skipped, as on every pull request |
| Steps that ran on V2 code for the first time | The secret scan (the public key line with its `gitleaks:allow`), `cargo clippy`, `cargo deny` (the two new wrappers), `cargo test` (177), both `tsc` steps (the second is new in Prompt 32), the file tree, the hosts check, Playwright: all success |
| Production | Unchanged: `/api/v1/healthz` reports `322c7d3`; `main` is at `322c7d3` |

**Closes.** The last "Done when" box of Prompt 33, and with it G M-1. The three things the entry of Prompt 32 and 33 left unchecked hold on Linux: the new `tsc` step, `cargo deny` with `proptest`, and the secret scan.

**Changed.** This file only: open item 4 is now the standing push step, open item 8 (branch protection) is new, the box of Prompt 33 is ticked.

## 2026-10-09 - Prompt 35: `offcut-mp4`: demux, probe, validate

**Added.**

| File | Content |
|---|---|
| `crates/offcut-mp4/src/demux.rs` | `SampleMeta` and `Demuxer<R>` with the eight signatures of TS §15.1; `open`, steps 1 to 4 of §6.5; crate-private accessors for the probe (`brand`, `video`, `audio`, `track_counts`). After the first `#[cfg(test)]`: `fixture`, a module that builds small MP4 files in memory for the tests of this crate |
| `crates/offcut-mp4/src/probe.rs` | `probe`, every field rule of §6.6. It reads the sample count, the runs of `stts` and `presented`, never the sample list |
| `crates/offcut-mp4/src/validate.rs` | `validate_probe`: the 11 rules of TS §15.2 in that order, and the one place a `ClipInfo` is built |

**Changed.** `crates/offcut-mp4/src/lib.rs`: the three modules.

**`open`, as built.**

| Step | Behaviour |
|---|---|
| 1 | A file shorter than 8 bytes is `NotIsoBmff` (the rule of G 2.2), and so is one whose first box is not `ftyp`, `moov`, `mdat`, `free`, `wide` or `skip`. Top-level boxes are read until `moov`: it may come after `mdat`. A `moof` before it, or `mvex` inside it, is `Fragmented`. At most 4,096 top-level boxes are read |
| 2 | Every `vide` and `soun` track is counted. The video track is the first enabled one; the audio track is the first one |
| 3 | The chosen tracks are resolved. For a track of more than 20,000 samples only `stsd`, `stts`, the count of `stsz` and `elst` are read |
| 4 | `t0` is the smallest `pts - edit_offset` of the video track, in its own ticks. A time is `(pts - edit_offset) / timescale - t0`, computed in 128-bit integers and rounded down to microseconds. With no video samples to look at, `t0` is 0 |

**Decided here, where §6.5 and §6.6 are silent.**

- **No enabled video track:** the first video track is used. Some files leave the flag unset on their only track; refusing them as "no video" would be wrong, and the count of tracks is the same either way.
- **No `moov` at all** (an interrupted recording): `Malformed("moov")`, which becomes `REJECT_CORRUPT`.
- **A `trak` with no `hdlr`** is skipped and not counted.
- **A sample index with no track of that kind** is `Malformed("index")`.
- **A file with no `ftyp`** is reported as `Mp4`: §6.6 gives `Mov` for the brand `qt  ` and `Mp4` otherwise, and that is what is built. A classic QuickTime file has no `ftyp`; V3 may want it to say `Mov` (TE-12).
- **Sample rate and channel count of AAC** come from the `AudioSpecificConfig`; a channel configuration of 0 there (the layout is described elsewhere) falls back to the sample entry.
- **`is_vfr` counts every sample duration,** the last one too, as §6.6 says. A file whose last frame is shorter by more than 1 ms than the others reads as variable. The flag changes no rule; it is carried to `ClipInfo`.
- **`validate_probe` with `video_tracks > 0` and no `video`** (or the same for audio) answers `NoVideo` (`NoAudio`). `probe` never produces that; a hand-made `ProbeInfo` can.

**Tests (inline): the other nine rows of §6.9 and the short-file case. 17 in the crate.**

| Row of §6.9 | Test |
|---|---|
| `tkhd` matrices for 0, 90, 180, 270 degrees | `probe::the_tkhd_matrices_of_the_four_quarter_turns` |
| `avcC` bytes `64 00 28` | `probe::avcc_bytes_64_00_28_give_the_codec_string_avc1_640028` (also the sizes, both frame rates, the audio fields and the duration of that file) |
| Each of the 11 rules violated alone | `validate::each_of_the_eleven_rules_violated_alone_gives_its_reason` |
| Two rules violated | `validate::of_two_violated_rules_the_earlier_one_wins` (three pairs) |
| Exactly 90,000 ms; 60,500 `FpsMilli` | `validate::exactly_90_000_ms_and_60_500_fps_milli_are_accepted` (also a long side of exactly 1,920) |
| A 1920x1080 probe with `R90` | `validate::a_1920x1080_probe_turned_90_degrees_is_a_1080x1920_portrait_clip` (also unturned, and a square) |
| A video track of 30,000 samples in one `stts` run | `probe::a_video_track_of_30_000_samples_opens_unresolved_and_is_too_long`: 1,000,000 ms, 30,000 `FpsMilli`, `Duration` |
| A valid video track beside an `lpcm` track of 4,320,000 samples | `probe::an_lpcm_audio_track_of_4_320_000_samples_opens_and_is_the_wrong_codec`: `AudioCodec` |
| `read_video_sample(0, ..)` on an unresolved track | `demux::read_video_sample_on_an_unresolved_track_is_malformed_samples` (also a resolved track read: bytes, times, the keyframe to seek to, an index out of range) |
| The short-file case (the guide's) | `demux::a_file_shorter_than_one_box_header_is_not_iso_bmff` (6 bytes, 0 bytes, 20 bytes of text; also `moof`, `mvex`, and no `moov`) |

**The two drills.** Each was undone from a copy.

| Temporary edit | What fired |
|---|---|
| `.unwrap()` in the non-test code of `demux.rs` | clippy: "used `unwrap()` on an `Option` value" |
| `let _ = std::time::Instant::now();` in `probe.rs` | clippy: "use of a disallowed method `std::time::Instant::now`", with the reason of `clippy.toml`, "pure crates must not read a clock (TS §7)" |

**The real clip (the temporary test `zz_real_clip` of G 2.3; never committed).**

| Field | Printed | Expected |
|---|---|---|
| `duration` | 74,705 | 74,705 within 1 |
| `video.codec`, `codec_string` | `H264`, `avc1.4d401f` | `H264`, starts with `avc1.4d` |
| `coded_width`, `coded_height`, `rotation` | 1280, 720, `R0` | The same |
| `frame_count` | 2,246 | 2,246 |
| `is_vfr` | `true` | `true` |
| `avg_fps`, `max_fps` | 30,064 and 32,930 | Not fixed by the guide |
| `audio` | `Aac`, `mp4a.40.2`, 48,000 Hz, 2 channels | The same |
| `validate_probe(&p, true)` | `Ok(ClipInfo)`: 1280x720, `Landscape`, 74,705 ms | `Ok(ClipInfo)` with `Landscape` |

The test also read every video sample and printed more than the guide asks, to check `open` step 4 and the seek against `ffprobe`:

| Read | This crate | `ffprobe` |
|---|---|---|
| Smallest video `pts` | 0 | Start time 0 |
| End of the last video frame | 74,705,033 microseconds | Duration 74.705033 s |
| First video sample | `pts` 0, `dts` -79,767, keyframe, 91,736 bytes | The first packet is a keyframe at 0 |
| End of the last audio frame | 74,517,520 microseconds | Duration 74.517521 s |
| `keyframe_at_or_before` 0 ms, 30 s, 74 s | Samples 0, 896, 2040 | Keyframes are packets 0, 896 (29.807867 s) and 2040 (67.855600 s) |

No expectation was adjusted. `git grep -n zz_real_clip -- crates web scripts` prints nothing.

**Checked.**

- G M-2: `cargo test -p offcut-mp4 --lib`, 17 passed; `cargo clippy -p offcut-mp4 --all-targets -- -D warnings` clean; `cargo deny check` ok; `pnpm check` green; no `zz_real_clip` outside the documents that describe it.
- `validate_probe` has 11 `return Err(RejectReason::...)`, numbered 1 to 11 in the code and read against TS §15.2: `Container`, `NoVideo`, `Hevc`, `VideoCodec`, `NoAudio`, `MultiAudioTrack`, `AudioCodec`, `Duration`, `Resolution`, `FrameRate`, `DecodeUnsupported`. It cannot return `FileSize`, `Corrupt` or `NoSpeech`. Its limits are the four it imports from `offcut_types::limits`.
- Lines above the test module: `demux.rs` 369, `probe.rs` 180, `validate.rs` 85. `boxes.rs` and `sample_table.rs` were not touched.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green; the frozen-file diff against the baseline is empty. **187 Rust, 76 Vitest, 12 Playwright** (Rust was 177; the ten are new).

**"Done when".**

- [x] `cargo test -p offcut-mp4 --lib`: the 16 rows of §6.9 and the short-file case pass.
- [x] The real clip printed the values above; `git grep -n zz_real_clip` finds it in the two documents that describe the test and nowhere else.
- [x] `validate_probe` has 11 rejecting branches, read against TS §15.2 line by line; G M-2 passes.

**Human.** Push; read `ci` (open item 4).

## 2026-10-09 - Prompt 36: resampler, core bindings, `CoreApi`

**Added.**

| File | Content |
|---|---|
| `crates/offcut-dsp/src/resample.rs` | `resample_mono(input, from, to)` (§7, TS §18.1) and its four inline tests |
| `crates/offcut-wasm-core/src/hash_api.rs` | `Sha256Stream`: `new`, `update`, `finalize_hex` (64 lower-case hex digits) |
| `crates/offcut-wasm-core/src/media_api.rs` | `DemuxerHandle`, `open_demuxer`, `probe_and_validate`, `resample`, and the five methods of D-53 (`probe`, `video_description`, `audio_description`, `audio_sample_count`, `read_audio_sample`); the private `JsRandomAccess` over a `FileSystemSyncAccessHandle` |

**Changed.**

| File | Change |
|---|---|
| `crates/offcut-dsp/src/lib.rs` | `pub mod resample;` and `DspError { Empty, NonFinite }`, unused until V3 |
| `crates/offcut-dsp/Cargo.toml` | `offcut-types`, `rubato` (feature `fft_resampler`), `thiserror`; dev `proptest` |
| `crates/offcut-wasm-core/Cargo.toml` | `offcut-mp4`, `offcut-dsp`, `js-sys`, `web-sys` (features `FileSystemSyncAccessHandle`, `FileSystemReadWriteOptions`), `serde`, `serde-wasm-bindgen`, `sha2`. Not `offcut-text` (Prompt 42) |
| `crates/offcut-wasm-core/src/lib.rs` | `mod hash_api; mod media_api;` and three helpers: `failure(code, detail)`, the plain object `{ code, detail }`; `to_plain` and `to_js`, every conversion through `Serializer::json_compatible()` (D-31). The panic hook and `core_version` are untouched |
| `Cargo.toml` | `[workspace.dependencies]`: `offcut-mp4`, `offcut-dsp` (paths), `rubato`, `js-sys`, `web-sys`, `serde-wasm-bindgen` |
| `deny.toml` | One wrapper, a third-party crate: `serde-wasm-bindgen` for `js-sys` and for `wasm-bindgen` (named by `cargo deny check`; parents read with `cargo tree -i`) |
| `web/src/wasm/load-core.ts` | `CoreApi` of §13.3 without `normalizeTranscript`; the types `CoreDemuxer` and `AudioSample`; the comment on `loadCore()` corrected (D-36): the model manager is the one caller on the main thread |

**Pinned.** `rubato` 5.0.1 (latest; without its default features, then `fft_resampler`), `serde-wasm-bindgen` 0.6.5 (latest), `js-sys` 0.3.106 and `web-sys` 0.3.106 (the releases that go with `wasm-bindgen` 0.2.129, and what `Cargo.lock` already held). `sha2` 0.11.0 was in the workspace. Brought in by `rubato`: `realfft` 3.5.0, `rustfft` 6.4.1, `audioadapter` 5.0.0. No license was added to `deny.toml`.

**The `wasm-bindgen` pair.** After `cargo check -p offcut-wasm-core --target wasm32-unknown-unknown`: `Cargo.lock` has the crate at 0.2.129 and `wasm-bindgen --version` prints 0.2.129. No dependency asked for a newer one; the pin was not touched.

**The resampler.** `rubato`'s synchronous FFT resampler, 1,024 frames per call, one channel. The whole clip goes through `process_all`, which removes the resampler's start-up delay; the result is then cut or padded with silence to exactly `round(len x to / from)`, computed in integers. `from == to` is a copy. Empty input or a rate of 0 is an empty vector. The signature returns no error (TS §18.1), and with rates above 0 the library has none to give; if it ever did, the function returns silence of the promised length, so that the length rule holds for any input.

**Shapes, as built (§13.2).**

| Call | Success | Failure |
|---|---|---|
| `open_demuxer` | `DemuxerHandle` | Throws `{ rejected: "REJECT_CONTAINER" }` for `NotIsoBmff` and `Fragmented`, `{ rejected: "REJECT_CORRUPT" }` for `Truncated`, `Malformed`, `Unsupported`; `{ code: "E_STORAGE_IO", detail }` for `Io`, and when the size of the file cannot be read |
| `probe` | `ProbeInfo` | `{ code: "E_INTERNAL", detail: "Serialize" }` |
| `probe_and_validate` | `{ ok: ClipInfo }` or `{ rejected: RejectReason }`, both returned | As `probe` |
| `read_audio_sample` | `{ data: Uint8Array, ptsUs, durationUs }` | `{ code: "E_DECODE_AUDIO", detail }` |

`detail` is the name of the error's variant (`"Malformed"`, `"Read"`), written out in two `match` statements; the box name a `Malformed` carries is not passed on. `JsRandomAccess` reads the size once, in `open_demuxer`, and calls `read(buffer, { at })` per read; fewer bytes than asked for is `IoError::Read`.

**`load-core.ts`.** `openDemuxer` returns a thrown `{ rejected }` as a value, after checking the reason against `REJECT_REASONS`, and rethrows anything else unchanged. Decided here, because §13.3 uses the name `CoreDemuxer` and does not define it: it is an object with `probe`, `videoDescription`, `audioDescription`, `audioSampleCount`, `readAudioSample` and `free`, typed with the generated types, so that a worker never handles the `any` the module's own declarations give. The module's handle behind it is kept in a `WeakMap`; `probeAndValidate` on a demuxer that was freed throws an `Error`.

**Differs from the prompt, the guide or the plan.**

- **Two inline tests in `offcut-wasm-core`,** which the prompt does not ask for: the digest of `"abc"` fed in two pieces, and the table of §6.1 (which failure of `open` is which rejection or code). Both run natively; neither calls into JavaScript.
- **`DemuxerHandle` keeps the bytes of the last sample read,** so that reading 3,493 audio frames does not allocate 3,493 times. The object returned holds its own copy.
- **A browser check of the media exports, which the prompt leaves to Prompt 37** (below).

**Tests (inline): the four rows of §7.**

| Row of §7 | Test |
|---|---|
| proptest: lengths 0 to 200,000; (44,100; 48,000), (48,000; 16,000), (48,000; 48,000) | `the_output_length_is_the_rounded_input_length_times_the_rate_ratio` (48 cases) |
| A 1 kHz sine at 48 kHz resampled to 16 kHz | `a_1_khz_sine_keeps_its_level_and_its_phase`: peak within 1% of 1.0; the falling zero crossing within one output sample of its place |
| An impulse at sample 4,800 | `an_impulse_stays_where_it_was_in_time`: the output peak is at 1,600 plus or minus 1 |
| Same input twice | `the_same_input_twice_gives_byte_identical_output` (also `from == to`, empty input, a rate of 0, and one exact length: 30,000 samples from 44,100 to 48,000 give 32,653) |

**Browser check (dev).** `vite` on port 5173, a temporary Playwright case.

| Asked | Result |
|---|---|
| The SHA-256 of `"abc"` through `core.newSha256()` | `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad` |
| `core.resample(new Float32Array(48000), 48000, 16000).length` | 16000 |

Not asked, and run with a temporary module worker (`zz-demux.worker.ts`): the reference clip written to OPFS and opened through a synchronous handle, so the path a file takes in Prompt 37 was exercised once here.

| Read in the worker | Result |
|---|---|
| `probe(size)` | A plain object with the fields of the generated `ProbeInfo`: `container: "mp4"`, `duration: 74705`, `video.codec_string: "avc1.4d401f"`, `rotation: "r0"`, `frame_count: 2246`, `is_vfr: true`, `audio.codec_string: "mp4a.40.2"`, 48,000 Hz, 2 channels |
| `probeAndValidate(d, size, true)` | `{ ok: { duration: 74705, display_width: 1280, display_height: 720, orientation: "landscape", ... } }` |
| `probeAndValidate(d, size, false)` | `{ rejected: "REJECT_DECODE_UNSUPPORTED" }`, returned |
| Descriptions | `avcC` 41 bytes; `AudioSpecificConfig` `[17, 144]` |
| `readAudioSample(0)` | 512 bytes in a `Uint8Array`, `ptsUs` 0, `durationUs` 21333; 3,493 samples; the last one ends at 74,517,520 microseconds |
| `readAudioSample(3493)` | Throws the plain object `{ code: "E_DECODE_AUDIO", detail: "Malformed" }` |
| A 6-byte text file | `{ rejected: "REJECT_CONTAINER" }`, returned |
| 3,293,491 samples from 44,100 to 48,000 Hz | 3,584,752 samples, a `Float32Array` |

Both temporary files are deleted.

**Checked.**

- `cargo test -p offcut-dsp`: 4 passed, one of them the proptest. `pnpm build:wasm` writes `web/src/wasm/pkg/core/`: `offcut_core_bg.wasm` is 371,738 bytes after `wasm-opt`.
- `pnpm gen:types && sh scripts/check-gen-clean.sh`: no diff. V2 adds no shared type here.
- `cargo deny check`: ok. `node scripts/check-file-tree.mjs`: 149 files; `offcut-dsp` is checked as a pure crate, and `rubato` with the FFT resampler resolves no browser or randomness crate.
- `pnpm build`: `check-hosts` reads the larger bundle and finds no new host literal.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green; the frozen-file diff against the baseline is empty. **193 Rust, 76 Vitest, 12 Playwright** (Rust was 187: four in `offcut-dsp`, two in `offcut-wasm-core`).

**"Done when".**

- [x] `cargo test -p offcut-dsp` passes with its proptest; `pnpm build:wasm` writes `web/src/wasm/pkg/core/`.
- [x] `pnpm gen:types && sh scripts/check-gen-clean.sh` shows no diff; both browser values match.

**Not measured.** How long `resample_mono` takes on the reference clip in the browser. It is part of the `probe_audio` budget, read in Prompt 37 and on R1.

## 2026-10-09 - The second push: `ci` green on Prompts 35 and 36

**Done by the human.** `git push` of `v2-build` at `bb3abee`, the commit of Prompt 36.

**Read by the agent** (the public API of GitHub).

| Read | Result |
|---|---|
| `origin/v2-build` | `bb3abee`, equal to the local branch |
| The `ci` run on `bb3abee` (pull request #2) | Success. Job `ci`: 3.0 minutes. `deploy-api` and `deploy-web`: skipped |
| Steps that ran on new code | The secret scan, `cargo clippy`, `cargo deny` (with `rubato`, `serde-wasm-bindgen` and the three new wrappers), `cargo test` (193), the WASM bundle with the media and hash exports, both `tsc` steps, the hosts check on the larger bundle, Playwright: all success |
| Production | Unchanged: `/api/v1/healthz` reports `322c7d3` |

**Closes.** The push that Prompt 35 ends with. `rubato` and the binding dependencies build and pass `cargo deny` on Linux, which the entries of Prompts 34 to 36 had left unchecked.

**Changed.** This file only.

## 2026-10-09 - Prompt 37: `opfs.ts`, `rpc.ts`, `pool.ts`, media worker

**Added.**

| File | Content |
|---|---|
| `web/src/persistence/opfs.ts` | `paths`, the ten functions of TS §23.1; `fileHandle`, `getFile`, `size`, `writeAt`, `truncate`, `move`, `remove`, `list`, `writeAudio`, `readAudio` (§15.2); `OpfsError`, tagged `quota` or `io`; the one `Bytes` cast, in `toBytes()` |
| `web/src/workers/rpc.ts` | `createClient`, `serveWorker`, `toAppFailure`, `CANCELLED`, `WorkerCallError`, the types of §15.6; `WorkerFailure`; the "Retryable" column of TS §11.2 as the table `RETRYABLE` |
| `web/src/workers/media/import.ts` | `importToOpfs`: 4 MiB chunks, the cancel flag read before each, progress in bytes after each; `openSource`; `byteSize` |
| `web/src/workers/media/audio-decode.ts` | `decodeAudio`: one `AudioDecoder`, mono by the mean of the channels, placed on the clip's timeline by timestamps only |
| `web/src/workers/media.worker.ts` | `importAndProbe` (steps 1 to 6 of §16.1) and `extractAudio`; the last statement, and the only one with an effect, is `serveWorker(...)` |

**Changed.**

| File | Change |
|---|---|
| `web/src/workers/pool.ts` | The table `WORKERS` with one row, `media`; `pool.media`, a client whose worker is created by its first call; `preload()` keeps its V1 signature and now also adds one `modulepreload` link per worker script; re-exports `WorkerCallError` and `Cancelled` |
| `crates/offcut-wasm-core/src/media_api.rs` | `JsRandomAccess` reads through a window of 1 MiB (see "Two things measured") |
| `docs/v2/v2implementation.md` | §13.1, §15.6 and §16.3 say what was built (3 replacements, each applied once) |

**Browser check (dev), on the reference clip.** `vite` on port 5173; a temporary Playwright case that calls `pool.media` as a use-case will. The dropped `File` was named `My Holiday Video.mp4`.

| Asked | Result |
|---|---|
| `duration` | 74,705 (the value of Prompt 33) |
| `pcm48.length` and `round(duration x 48)` | 3,585,840 and 3,585,840 |
| `pcm16.length` and `round(duration x 16)` | 1,195,280 and 1,195,280 |
| OPFS | `clips/<clipId>/` holds one entry, `source`, of 35,201,023 bytes. No name from the dropped file |
| A `File` of 500,000,001 bytes | `{ rejected: "REJECT_FILE_SIZE" }`; no directory was made for its clip id |
| A 6-byte text file | `{ rejected: "REJECT_CONTAINER" }` |
| `paths.modelFile("m", "../x")` | Throws |
| `await size("nope")` | `null` |

**Not asked, and checked in the same run.**

| Checked | Result |
|---|---|
| **The decoded audio against ffmpeg's** (`ffmpeg -af "pan=mono\|c0=0.5*c0+0.5*c1" -ar 48000 -f f32le`), sample for sample | **Identical:** over all 3,576,832 samples ffmpeg gives, the largest difference is 0. No lag. The last non-zero sample is number 3,571,711 in both. The 9,008 samples after ffmpeg's end are silence: the audio track is 187 ms shorter than the video |
| The 16 kHz output against ffmpeg's own resampler | No lag; a difference of 2.3% of the signal's level, largest single sample 0.05. Two different low-pass filters; the length rule and the timing are the same |
| Progress | 9 messages, the last one at 35,201,023 of 35,201,023 bytes |
| A second `extractAudio` for the same clip; one after the worker had rejected another file | The same lengths (the second one probes the source again) |
| `extractAudio` for a clip id that has no file | Rejects with `{ code: "E_STORAGE_IO", stage: "import", retryable: true }` |
| `opfs.ts` helpers | `writeAt` twice, `size` 6; `truncate` to 5; `move` from the `.part` name to the final name, the bytes intact and the `.part` gone; `list`; `writeAudio` and `readAudio` give the same three floats back; `remove` twice (the second on nothing) |
| Cancel, through `createClient` directly (`pool` has no cancel before V4) | A call cancelled as soon as its job id is known resolves with `{ cancelled: true }` |
| A second call while one is pending | Rejects with `E_INTERNAL`, `retryable: false`, detail `Busy`; the first call still answers |
| A method the worker does not have | Rejects with `E_INTERNAL`, detail `NoHandler` |
| **A production build, served with the headers of `vercel.json`** (a temporary hook in `main.tsx`) | The same five numbers; `crossOriginIsolated` true; no CSP violation. The worker is the chunk `assets/media.worker-<hash>.js` (15.6 kB), and the `modulepreload` link names that file |

**Two things measured, and what was changed for them.** The first run took 3.6 s for `extractAudio` on the development machine, which is faster than R1; the budget for probe and audio together is 3.7 s on R1 (D-64).

| Found | Changed | After |
|---|---|---|
| **One read of the OPFS file costs about 0.4 ms, whatever its size.** The demuxer read the 3,493 audio frames one at a time: 1.35 s | `JsRandomAccess` keeps a window of 1 MiB read ahead. A read the window does not hold moves the window to start there; a read as large as the window bypasses it. A read past the end still fails as `IoError::Read` | 0.11 to 0.15 s for the same reads |
| **The decoder works through a shallow queue slowly.** With "32 at a time, waiting for `dequeue` while the queue is above 32" (§16.3), decoding took 1.7 s; with every frame queued at once, 0.4 s | 32 frames per batch as before, with the cancel check; after each batch one turn of the event loop (outputs are delivered, a cancel message is read); waiting for `dequeue` only above 256 queued frames | 0.4 to 0.6 s. The output is still identical to ffmpeg's |

`extractAudio` now takes 0.8 to 1.4 s and `importAndProbe` 0.5 to 0.6 s on the development machine (dev server and production build alike). These are `dev` readings and stand for nothing on R1.

**Decided here, where the plan is silent or cannot be built as written.**

- **`WorkerFailure`,** an `Error` with `code`, `detail` and an optional `stage`. §15.6 has a worker throw the plain object `{ code, detail, stage? }`; ESLint (`only-throw-error`, part of the type-checked set V1 turned on) refuses that in TypeScript. `toAppFailure` reads `code` from either. The objects the WASM bundle throws are plain objects, as before.
- **The transfer check runs in every build** and fails the call with `E_INTERNAL`. §15.6 asks for it in development builds only, and `workers/` can neither read `import.meta.env` nor import `config/env.ts` (V1's lint). It is one comparison per transferred buffer.
- **A one-way handler that throws** makes the worker report an uncaught error, which the client sees as a crash of that worker. No answer can carry the failure, and it must not be swallowed.
- **A handler that returns `CANCELLED` without its flag set** is answered with `E_INTERNAL` (detail `CancelledUnasked`).
- **`importToOpfs` has a fourth, optional parameter,** the progress function: TS §15.1 gives it three, and §16.2 asks it for progress once per chunk. It returns the handle or `CANCELLED`.
- **`decodeAudio` opens its own demuxer on the handle it is given** and frees it. TS §15.1 gives it the handle, not a demuxer. A decoder that answers at another sample rate than the container states (which an implicit SBR stream would) is `E_DECODE_AUDIO`, detail `SampleRate`, not audio of the wrong length.
- **Decoded samples are written straight to their place** in a buffer of the final length; what lies before video time 0 or after the video's end is not kept. One buffer, in place of a decoded copy and an aligned copy.
- **`pool.ts` takes the worker's URL from `./media.worker.ts?worker&url`** (known issue 14). §15.7 writes `new Worker(new URL(...))`; the preload link needs the same URL, and outside `new Worker(...)` that form does not give the built script.
- **The table of `pool.ts` holds script and stage.** The one-way list and the list of methods accepted beside the preview are arguments of `serveWorker` in each worker's entry file, because a worker may not import `pool.ts` (TS §7). For the media worker both are empty.
- **`move()` uses `FileSystemFileHandle.move`,** which Chrome has for OPFS files and the TypeScript library does not declare yet. Where it is missing the call fails as `io`.
- **`OpfsError`** is the tagged error §15.2 describes without naming.

**The two lint checks of "Done when".**

| Check | Result |
|---|---|
| ESLint on `src/persistence` and `src/workers` | Clean. `media/import.ts` and `media.worker.ts` import `persistence/opfs.ts`: edge D-27 b, positive side |
| `export const zzSecond = 1 as Bytes;` added to `opfs.ts` | `no-restricted-syntax`: "In this file one cast is allowed: to Bytes, in the last return of toBytes() (D-59)". Reverted |
| The cast moved out of `toBytes()` into `size()` | The same rule fires. Reverted |

**Checked.**

- G M-3: `cargo test -p offcut-dsp -p offcut-mp4` green; `pnpm build:wasm && pnpm check && pnpm build` green, `check-hosts` passes with the worker chunk (6 files in `web/dist`); `wasm-bindgen` 0.2.129 in `Cargo.lock` and from the CLI; `web/src/gen` and `workers/protocol.ts` unchanged.
- The bundle: `offcut_core_bg.wasm` 372,779 bytes. The app shell, gzip: 118.2 kB, of which the worker chunk is 6.0 kB.
- Lines: `rpc.ts` 374, `opfs.ts` 219, `audio-decode.ts` 161, `media.worker.ts` 117, `pool.ts` 86, `import.ts` 65; `media_api.rs` 273.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green; the frozen-file diff against the baseline is empty. **193 Rust, 76 Vitest, 12 Playwright**, as after Prompt 36: this prompt adds no test file (`rpc.test.ts` is V4).
- Every temporary file is gone: three Playwright cases, the hook `web/src/zz-spike.ts` and its line in `main.tsx`, and the timing lines that were in two worker files for the measurements.

**"Done when".**

- [x] The five numbers and the two rejections match; G M-3 passes.
- [x] ESLint is clean for `persistence`, `workers`: edge D-27 b proves its positive side; a second `Bytes` cast in `opfs.ts` fails lint.

**Not checked.** A cancel in the middle of a copy or of a decode: the one cancel tested arrives before the first chunk. The quota path (`E_STORAGE_QUOTA`): no full disk was made. Both have tests in V4 and V7 (`rpc.test.ts`, `failure-recovery.spec.ts`).

**Human.** Push; read `ci` (open item 4).

## 2026-10-09 - The third push: `ci` green on Prompt 37

**Done by the human.** `git push` of `v2-build` at `3d2c244`, the commit of Prompt 37.

**Read by the agent** (the public API of GitHub).

| Read | Result |
|---|---|
| `origin/v2-build` | `3d2c244`, equal to the local branch |
| The `ci` run on `3d2c244` (pull request #2, run 10) | Success. Job `ci`: 2.7 minutes. `deploy-api` and `deploy-web`: skipped |
| Pull request #2 | Open, draft, no conflict with `main` |
| Production | Unchanged: `/api/v1/healthz` reports `322c7d3` |

**Closes.** The push that Prompt 37 ends with. G M-3 holds on Linux: the worker chunk builds there, and the hosts check passes with it.

**Changed.** This file only. The entry was written with the next commit, not on the day of the run.

## 2026-10-09 - Outside a prompt: the CORS origin; D-66, the small-size speech model

No code changed. Seven documents changed, all under `docs/`.

**Done by the human.** The origin `http://localhost:4173` was added to the CORS policy of the asset bucket, in the host's dashboard (open item 1, D-41).

**Read by the agent.** `curl` with `Range: bytes=0-1023` on the demo video of the asset host, once per origin.

| `Origin` sent | Status | `Access-Control-Allow-Origin` | Exposed headers |
|---|---|---|---|
| `http://localhost:4173` | 206 | `http://localhost:4173` | `Accept-Ranges`, `Content-Length`, `Content-Range` |
| `http://localhost:5173` | 206 | `http://localhost:5173` | The same three |
| `https://offcut-one.vercel.app` | 206 | `https://offcut-one.vercel.app` | The same three |
| `https://example.com` | 206 | None: a browser refuses the answer | None |

The preflight (`OPTIONS`, `Access-Control-Request-Method: GET`, `Access-Control-Request-Headers: range`) from `http://localhost:4173` answers 204 and allows `GET, HEAD` and the header `range`. Open item 1 is closed.

**The decision (founder, 2026-10-09): D-66.** The V2 speech model is the small-size English model, not the base-size one, and the model files may total 260,000,000 bytes, not 150,000,000. The founder's reason: a better transcript is worth a larger download and a longer wait. The agent had advised to start with the base-size model and to compare the two on R1 at Prompt 44; the founder decided otherwise.

**Why 260,000,000.** It is the largest size that keeps the first-run budget of PS §20.2, which is not changed: 260 x 8 / 25 = 83 s of download at 25 Mbps, plus 5 s of initialization, is 88 s of the 90 s allowed.

**Changed.**

| File | Change |
|---|---|
| `docs/product.md` | §10 J4: the copy says "about 250 MB". §20.2: the first-run line is worked for a 260 MB model (88 s) |
| `docs/technicalspec.md` | §16.1: the small-size model is the default candidate, base-size the first fallback and tiny-size the second; `totalBytes` at most 260 MB, 260,000,000 bytes. §16, contingency: the smaller model is base-size first. §31: the estimate of the model session is 1,200 MB and the ASR phase about 1.46 GB (both assumptions; TE-14 measures); the disk line says 260 MB |
| `docs/buildplan.md` | §4: the manifest total; the E-3 fallback names base-size first |
| `docs/v2/v2implementation.md` | D-66 added to §2, with its costs; §0 says "D-18 to D-66". D-41, §3.3, §15.1, §23.3 and the E-3 row of §24.1 follow it. 7 replacements |
| `docs/v2/coding-promptsv2.md` | The Standard Agent Block says "D-18 to D-66". Prompt 38: the small-size model, 260,000,000 in step 2 and in its "Done when" box. Prompt 56: the size of the download the setup test waits for |
| `docs/v2/v2buildguide.md` | The table of references names D-66. Steps 0.4, 4.2 and 11.1, Milestone 4 and table (c): the model and the number |
| `docs/v2/v2changelog.md` | Open item 1 closed; known issue 18 added; the two entries of this date |

The edit was made by a script that refuses a replacement unless its old text occurs exactly once, and that writes nothing unless every replacement can be applied: 28 of 28. Each file keeps its line endings.

**Not changed, and why.**

- **No file outside `docs/`.** No source file, script or workflow holds the number 150,000,000: `web/src`, `crates`, `server`, `scripts` and `.github` were searched. The first code that holds the limit is the manifest assertion of `download.test.ts` (Prompt 39), which reads it from §23.3.
- **`modelId` stays `asr-en-v1`.** Paths, tests and prompts name it.
- **The bands of E-3 (§24.3).** The founder said on this date that a transcription time above the 25 s target does not stop V2. The plan already says so: 25 to 50 s is "continue and record the miss". Over 50 s still means the smaller model, which is now the base-size one. No new number was given for that line, so none was written.
- **The TE-7 lines that say "a file of 150 MB"** (`scripts/check-external-facts.mjs`, TS §37, §24.1 of the plan, BP §3): they describe what was tested. Whether a larger single file is uploaded depends on the set Prompt 38 takes (known issue 18).
- **The typed-in `150 MB` of Prompt 40 and G 4.5.** It is a sample string for a drill, not the limit.
- **The render figures of TS §31** that happen to be 150 MB, the V1 documents and `v2implementation-notes.md`.

**What was measured for the decision.** File sizes from the listing of one candidate source, `onnx-community/whisper-small.en_timestamped`, read on this date; each total includes the five configuration files (2,692,839 bytes).

| Set | Encoder | Decoder | Total | First run at 25 Mbps |
|---|---|---|---|---|
| 4-bit encoder, mixed 4-bit and 16-bit decoder | 66,178,491 | 145,776,485 | 214,647,815 | 74 s |
| The 8-bit pair | 92,240,508 | 156,794,981 | 251,728,328 | 86 s |
| The 4-bit pair | 66,178,491 | 233,418,140 | 302,289,470 | 102 s: over the limit and over the budget |

For comparison, the 4-bit pair of the base-size model is 145,199,758 bytes. The model's own documentation gives 244 million parameters and a relative speed of about 4 for small, 74 million and about 7 for base.

**Checked.**

- `node scripts/check-file-tree.mjs` passes: the tree of TS §5 was not touched.
- `git diff --stat`: seven files, all under `docs/`. The frozen-file diff against the baseline is empty.
- `git grep -n "150,000,000\|150000000"` finds the old limit in two places only, both of which describe the change: this file, and D-66 of the plan ("the limit was 150,000,000").
- The four-command gate was not run: no file that a build or a test reads was changed. The test counts are those of Prompt 37: **193 Rust, 76 Vitest, 12 Playwright**.

**Open.**

- Prompt 38 chooses the file set (known issue 18). Nothing was downloaded or uploaded.
- E-3 is more likely to miss its 25 s target, and the ASR phase is estimated 40 MB under the memory budget. Both are read on R1 (Prompts 44 and 59).
- TS §16.1 had made the small-size model depend on a word error rate above 12% (E-10). That rate has not been measured for either model.

## 2026-10-09 - Prompt 38: upload script, model on the asset host, manifest, `fetchAsset`

**Step 2 was done by the agent.** The prompts file gives the choice of the model, the download and the uploads to the human, "or the agent when this prompt's message says so". The founder chose the 214 MB set and said so on this date: the agent downloads and uploads, the sample clip included. The commit of D-66 (`6e5cc40`) had not been pushed when this prompt started; the last `ci` run read is the one on `3d2c244`.

**Added.** `web/src/config/model-manifest.json`: the shape of TS §16.1; `modelId` `asr-en-v1`; seven files; `totalBytes` 214,647,815.

**Changed.**

| File | Change |
|---|---|
| `scripts/upload-assets.sh` | The folder may also be `models/<modelId>`: one more segment of lower-case letters, digits and hyphens (D-62). The usage line and the header say so. The content-type table is unchanged: the model's files end in `.onnx` and `.json`, and it has both |
| `web/src/net/asset-fetch.ts` | `SAMPLE_CLIP_PATH`, `AssetResult` and `fetchAsset`, as §15.4. The comment at the top no longer says the file makes no request |
| `docs/v2/experiments.md` | The record of the TE-7 re-check |

**The model.** The small-size English model with word timestamps (D-66), from `onnx-community/whisper-small.en_timestamped` at revision `80853938`, into `C:\Users\Mayan\offcut-models\asr-en-v1`, outside the repository.

| File | Bytes | Checked against the source's listing |
|---|---|---|
| `encoder_model_q4.onnx` | 66,178,491 | SHA-256 |
| `decoder_model_merged_q4f16.onnx` | 145,776,485 | SHA-256 |
| `tokenizer.json` | 2,405,679 | Git blob id |
| `tokenizer_config.json` | 282,662 | Git blob id |
| `config.json` | 2,203 | Git blob id |
| `generation_config.json` | 1,956 | Git blob id |
| `preprocessor_config.json` | 339 | Git blob id |
| **Total** | **214,647,815** | The limit is 260,000,000 |

Seven of seven match. Read before the upload: the decoder file names its `cross_attentions` outputs, and `generation_config.json` holds `alignment_heads` (19 pairs). Both are what word timestamps are computed from. `preprocessor_config.json` asks for 16,000 Hz and 30 s windows, which is what §16.5 feeds it.

**The uploads.** The four `ASSET_S3_*` variables were read from `.env.deploy` inside a subshell that ran the two upload commands and ended; no value was printed, and the shell of the next command held none (counted: 0). The script sends the SHA-256 of each file with it, so the host refuses bytes that differ. It printed:

```text
models/asr-en-v1/config.8825c4174cb86f94.json
models/asr-en-v1/decoder_model_merged_q4f16.0d38a3ab3d034990.onnx
models/asr-en-v1/encoder_model_q4.f5a068d9ec94f60d.onnx
models/asr-en-v1/generation_config.5490747ca976d6b3.json
models/asr-en-v1/preprocessor_config.a6a76d28c93edb27.json
models/asr-en-v1/tokenizer.5eb60cec1e77aeeb.json
models/asr-en-v1/tokenizer_config.93879c3dccdd4b97.json
media/speech_scriptA_landscape_720p.7da948cecc39438a.mp4
```

The last line is the sample clip, the founder's webcam recording: it is public from this date. A file on the host cannot be changed; it can be removed in the host's dashboard.

**The manifest.** Written by a one-off script kept outside the repository: it read the seven printed paths, found each file on disk by taking the hash segment out of its stored name, refused a path whose hash segment is not the start of the file's SHA-256, and wrote `path`, `bytes` and `sha256`. No value was typed. `node -e` of G M-4 prints `asr-en-v1 true true`.

**TE-7 re-check: passed.** On the largest file, `Range: bytes=0-8388607` answers `206 8388608`; the header check shows `Cache-Control: public, max-age=31536000, immutable` and the three exposed headers. The full record is in `experiments.md`. The largest single file is 145,776,485 bytes, under the 150 MB of V1's TE-7: the lines known issue 18 named need no correction.

**The three drills.** No credential was in the shell, and the clip was the file argument.

| Folder argument | Result |
|---|---|
| `models/../media` | Exit 1: "a model id may hold only lower-case letters, digits and '-'" |
| `models/ASR_EN` | Exit 1, the same message |
| `models/asr-en-v1/extra` | Exit 1, the same message |
| `models/asr-en-v1` | Passes the folder rule and stops at the next one: "ASSET_S3_ENDPOINT is not set" |

Nothing was uploaded by any of the four.

**Browser check (dev).** `vite` on port 5173; a temporary Playwright case; the request headers read through the DevTools protocol.

| Asked | Result |
|---|---|
| `fetchAsset(SAMPLE_CLIP_PATH, { range: 0 to 1023 })` | `[true, 206]`, 1,024 bytes |
| That request | `Range: bytes=0-1023`; no cookie; no `Authorization`; no query string |
| The same call offline | `navigator.onLine` false; `{ ok: false, cause: "offline" }` |

Not asked, and checked in the same run:

| Checked | Result |
|---|---|
| 8 MiB of the largest model file | `[true, 206]`, 8,388,608 bytes; `Content-Range` readable |
| The same range on a file of 2,405,679 bytes | 206 with the whole file; `Content-Range: bytes 0-2405678/2405679` |
| A file with no range | `[true, 200]`, 339 bytes; the request has no `Range` header |
| A path that is not on the host | `{ ok: false, cause: "status", status: 404 }` |
| A signal aborted before the call, and one aborted during it | `{ ok: false, cause: "aborted" }`, twice |
| A path with a query string | `assetUrl` throws; no request is made |
| `document.cookie` on the page | Empty |

**Decided here, where the plan is silent.**

- **`modelVersion` is `small.en-timestamped-q4-q4f16@80853938`:** the model, the two precisions, and the revision of the source. TS §16.1 asks for a "runtime+weights version string"; the runtime is not chosen before Prompt 43, which may add its version. Nothing reads the value before `cacheInfo()` (Prompt 40); `Transcript.model_version` is the `modelId` (D-50).
- **The files are listed in the order the script printed them,** which is the order of their names. The downloader takes them in that order.
- **`fetchAsset` looks at the signal first, then at `navigator.onLine`.** A call that is both aborted and offline answers `aborted`.
- **A rejection of `fetch` that is neither an abort nor a `TypeError` is thrown on.** The table of §15.4 names those two; `fetch` has no third.
- **Which seven files.** The two halves of the model and the five configuration files the runtime is expected to read. The source also holds `vocab.json`, `merges.txt`, `added_tokens.json`, `special_tokens_map.json` and `normalizer.json`, which `tokenizer.json` makes unnecessary. Prompt 43 shows what the runtime asks for; if it asks for more, steps 2 and 3 are repeated.

**Differs from the prompt or the guide.**

- **`models/` with nothing after it is refused too,** with the same message. The prompt names three bad arguments.
- **The integrity of two uploads was read back,** which the prompt does not ask for: the 2.4 MB file and the 66 MB file were fetched whole and their SHA-256 equals the manifest's.

**Checked.**

- "Done when": the manifest total is 214,647,815, at most 260,000,000, and equals the sum of `files[].bytes`; every `path` starts with `models/asr-en-v1/`.
- `pnpm --filter web exec eslint src/net` and `tsc --noEmit` are clean. `node scripts/check-file-tree.mjs` passes: `model-manifest.json` was already in the tree of TS §5.
- `git grep -n "fetch(" -- web/src` finds the call in the four allowed files only.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green; the frozen-file diff against the baseline is empty. **193 Rust, 76 Vitest, 12 Playwright**, as after Prompt 37: this prompt adds no test.
- The secret scan, before the commit (known issue 8): gitleaks 8.18.4, the Windows archive with its SHA-256 equal to the published one, run with `--no-git` on the five files of this commit. No finding: the 64-digit hashes of the manifest are not taken for secrets.
- The temporary Playwright case is deleted and the dev server stopped.

**"Done when".**

- [x] The manifest total is at most 260,000,000 (the limit of D-66) and equals the sum of `files[].bytes`; every `path` starts with `models/asr-en-v1/`.
- [x] TE-7 is recorded; the ranged fetch from the dev page answers 206.

**Not checked.**

- **Whether this set runs.** Nothing loads the model before Prompt 43 (known issue 18).
- The billing page of the asset host.
- The license of the source (open item 9).

**Human.** Push `6e5cc40` and this commit; read `ci`. Prompt 38 is not marked **Push**, but two commits are waiting.

## 2026-10-09 - The fourth push: `ci` green on D-66 and Prompt 38

**Done by the human.** `git push` of `v2-build` at `1757fe6`, the commit of Prompt 38; the commit of D-66 (`6e5cc40`) went with it.

**Read by the agent** (the public API of GitHub).

| Read | Result |
|---|---|
| `origin/v2-build` | `1757fe6`, equal to the local branch |
| The `ci` run on `1757fe6` (pull request #2, run 11) | Success. Job `ci`: 2.5 minutes, 35 steps passed, 2 skipped, none failed. `deploy-api` and `deploy-web`: skipped |
| Pull request #2 | Open, draft, no conflict with `main` |
| Production | Unchanged: `/api/v1/healthz` reports `322c7d3` |

**Closes.** What the entry of Prompt 38 left for Linux: the secret scan accepts `model-manifest.json`, and the file-tree check, both `tsc` steps and the hosts check pass with the new files.

**Changed.** This file only. The entry was written with the next commit.

## 2026-10-09 - Prompt 39: model machine, store, downloader

Prompts 39 and 40 were asked for in one sitting; each has its own gate and its own commit.

**Added.**

| File | Content |
|---|---|
| `web/src/state/machines/model-machine.ts` | `ModelStatus` (7 states), `ModelEvent` (11 events), `MODEL_MACHINE`: the 14 pairs of §18.1 |
| `web/src/state/model-store.ts` | `useModelStore`, `ModelState`, `ModelProgress`, and the eight actions `inspected`, `start`, `progress`, `lastByte`, `netFail`, `fail`, `hashOk`, `cleared`. The one cast of the file is `const ZERO_BYTES = 0 as Bytes` |
| `web/src/models/download.ts` | `MODEL_PART_BYTES`, `MODEL_RETRIES`, `MODEL_BACKOFF_MS`, `VERIFY_SLICE_BYTES`; `ManifestFile`, `DownloadDeps`, `VerifyDeps`, `ModelError`; `createDownloader`, `fetchRanged`, `verifyAndFinalize`, `localName`. The one cast is in `toBytes()` |
| `web/src/models/download.test.ts` | The 16 cases of §23.3 |

**Changed.** `docs/v2/v2implementation.md`: §17.2 says what was built (1 replacement, applied once).

**The 14 pairs,** counted in the file against §18.1: `unknown` 3, `absent` 1, `partial` 2, `ready` 1, `downloading` 4, `verifying` 2, `failed` 1. `verifying` has no `start` and no way to `downloading`.

**The eight rules of §17.2, and the test that holds each.**

| Rule | Test |
|---|---|
| 1. `have` starts at `resumeFrom`; a part larger than the file is truncated | "resumes at the size of the part"; "truncates a part that is larger than the file and starts at 0" |
| 2. Ranges of 8 MiB, the last one shorter | "downloads a 20 MiB file into an empty part in three ranged requests" |
| 3. A 206 is written at `have`, counted, and sets the failure count back | The same; "counts failures in a row: a success sets the count back" |
| 4. A 200 truncates the part and writes the whole body | "starts the part again from 0 when the host answers 200 with the whole file" |
| 5. A 206 with no bytes, or with more than was asked for, is a failed attempt | "counts a 206 with no bytes, and a body longer than the range, as failed attempts" |
| 6. Waits of 1, 3 and 9 s; the fourth failure in a row is `E_MODEL_DOWNLOAD`; the part is kept | "waits 1 s after one offline answer and asks for the same range again"; "gives up with E_MODEL_DOWNLOAD after four failures in a row, and keeps the part" |
| 7. An abort is thrown on as the abort; the part is kept; no wait | "rejects with the abort when the signal aborts mid-file, keeps the part and does not wait" |
| 8. A full disk is `E_MODEL_STORAGE`, any other storage failure `E_STORAGE_IO` | "names a full disk E_MODEL_STORAGE and any other storage failure E_STORAGE_IO" |

The other five cases: every request names the manifest path and a range; a file with the manifest's hash gets its final name; a file with one flipped byte is removed and has no final name (INV-20); a 9 MiB file is hashed in slices of 4, 4 and 1 MiB with two pauses; `localName`; the manifest (total, limit of 260,000,000, 64 hex digits, path prefix).

**Decided here, where the plan gives a name and no shape, or cannot be built as written.**

- **`ModelError`,** an `Error` with a `code` (`E_MODEL_DOWNLOAD`, `E_MODEL_HASH`, `E_MODEL_STORAGE` or `E_STORAGE_IO`). §17.2 has the downloader `throw { code }`; ESLint refuses a thrown plain object (`only-throw-error`), as it did in Prompt 37, and `models/` may not import the `WorkerFailure` of `workers/rpc.ts`. §17.2 of the plan says so now.
- **`VerifyDeps`** (§17.2 names it): `opfs` with `getFile`, `move` and `remove`; `newSha256()`, which gives a stream with `update` and `finalizeHex`; `pause()`, one turn of the event loop. The model manager passes the stream of `loadCore()` (Prompt 40). **`download.ts` does not import `wasm/load-core.ts`:** a unit test that loaded it would need the built bundle, which the test job of CI builds but a fresh clone does not have.
- **`ManifestFile`** is `{ path, bytes, sha256 }` with `bytes` a plain number: it is the type of an entry of the JSON file.
- **The part file's path comes from the manifest path:** `fetchRanged(file, o)` has no model id (TS §16.2), so the id is the second segment of `models/<modelId>/<file>`. A path of another form throws.
- **`localName` throws on a name without a hash segment** of 16 hex digits. It never guesses a name.
- **A 200 whose body is not exactly `file.bytes` long is a failed attempt,** and the part is back at 0 bytes. §17.2 names the longer body of a 206 only.
- **A connection that breaks while a body is read** is a failed attempt, like an answer that never came.
- **The signal is looked at before every request.** A wait that has started is not cut short: the abort is seen at the next request, after at most 9 s. `fetchRanged` has no timer to cancel in V2; V4 adds the cancel.
- **`DownloadDeps.opfs.size` is not called.** The caller passes the size of the part as `resumeFrom` (§17.2, step 1); the type is as §17.2 gives it.
- **The store's `fail(code)` picks the event.** The machine has three ways into `failed`, and the set has seven files, of which six are checked while the store still says `downloading`. A wrong hash of one of those six is stored as `last_byte` followed by `hash_bad`, so that a wrong hash always goes through `hash_bad`. From `verifying` the only way to `failed` is `hash_bad`, also for a storage failure there; the state then holds the real code. No pair was added.
- **`progress(p)` is kept only while the status is `downloading` or `verifying`.** It is not a transition. `hashOk()` sets `done` to `total`.
- **A legal transition stores the whole state,** so that `error` is gone after `start`.

**Differs from the prompt, the guide or the plan.**

- **The hash in the three `verifyAndFinalize` cases is not SHA-256** but a small digest written in the test: the real one is in the WASM bundle (see above). The cases prove the slices, the comparison, the move and the removal. The real SHA-256 against the real manifest is the browser check of Prompt 40.
- **The test makes no `Bytes`.** A cast to a unit type is refused in a test as anywhere else (D-59), and the cases need 0, 8 MiB and 16 MiB as `Bytes`. They are taken from the downloader in `beforeAll`: started with `LIMITS.MAX_FILE_SIZE` as the size of its part, it truncates the part to 0 and writes at 0, 8 MiB and 16 MiB, and the fake file system records those offsets.

**The drill.** `import { http } from "../net/http";` in `download.ts`: ESLint, `boundaries/dependencies`, "There is no policy allowing dependencies from elements of type "models" to elements of type "net"". Restored from a copy.

**Checked.**

- `pnpm --filter web exec vitest run src/models/download.test.ts`: 16 passed.
- `pnpm --filter web exec eslint src/models src/state`: clean. Edge D-27 d is used by `download.test.ts` (the manifest); edge f is not used yet (see "Not checked").
- Lines: `download.ts` 276, `model-store.ts` 105, `model-machine.ts` 32.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test` and `pnpm build` green; the frozen-file diff against the baseline is empty. **193 Rust, 92 Vitest** (Vitest was 76; the 16 are new).
- **`pnpm e2e` was not green as the gate runs it.** With Playwright's default of 6 workers, 6 of the 12 cases failed in each of four runs on this date: the six that start first time out in the capability check of the start page ("Checking your browser…", or the storage check answering no after its 2 s limit, V1 known issue 30). With `--workers=3`, and with `--workers=2`, **12 of 12 pass**, in 28 s and 35 s. The cause is the machine, not this prompt: the app that was served is the build of Prompt 38, file for file (`assets/index-TZ9rx-jv.js`, `assets/media.worker-wI5sawug.js`, the same content-hashed names), because no file of this prompt is imported by the app yet; `landing.spec.ts` and `playwright.config.ts` are unchanged; the same suite passed with 6 workers after Prompt 38, four hours earlier. At the time of the runs the machine had 2.6 to 4.4 GB of memory free of 15.7, with two other browsers and the editor open. Nothing was changed to make the suite pass. The `ci` run of the next push is the check on a clean machine.

**"Done when".**

- [x] `pnpm --filter web exec vitest run src/models/download.test.ts`: 16 cases pass; the Vitest total is 92.
- [x] ESLint is clean for `src/models` and `src/state`. Edge D-27 d proves its positive side. **Edge f is first used in Prompt 40,** by `model-manager.ts`: neither file of this prompt needs the `AppFailure` type, and an import that nothing uses fails lint.

**Not checked.** The store against a real download, and the hash with the real SHA-256: both are the browser check of Prompt 40.

## 2026-10-09 - Prompt 40: model manager, download panel, copy

**Added.**

| File | Content |
|---|---|
| `web/src/models/model-manager.ts` | `inspect`, `ensureReady`, `cacheInfo`, `clear` with the signatures of TS §16.2; `ModelFailure`, the `Error` with a `failure: AppFailure` that `ensureReady` rejects with. The one cast is in `toBytes()` |
| `web/src/ui/components/ModelDownloadPanel.tsx` | The J4 panel: the sentence with the size, a progress bar, the time remaining, the "checking" line, and the words of the three model failures. It reads the model store and calls nothing |

**Changed.**

| File | Change |
|---|---|
| `web/src/copy/messages.ts` | `modelDownload.body(sizeMb)`, `.progress(etaSecs)`, `.verifying`; `errors.E_MODEL_DOWNLOAD`, `E_MODEL_HASH`, `E_MODEL_STORAGE`, each with title, body and action |
| `scripts/check-copy-codes.mjs` | Those three codes leave `COPY_PENDING`: 34 pending, was 37 (D-57, V1 D-13) |
| `web/src/ui/styles/components.module.css` | Three classes: `modelPanel`, `progressTrack`, `progressBar` |
| `docs/v2/v2implementation.md` | §17.1 and §17.3 say what was built (2 replacements, each applied once) |

**Browser check (dev, persistent profile).** `vite` on port 5173; Chrome under Playwright with a profile in `fixtures/.cache/zz-profile-5173/`, made new for each run; the model files come from the real asset host. Two runs: the first went through the four scenarios, and the second repeated the interrupted download with exact counts, because the first run's request filter also counted two files of the dev server and its "30%" was closed at 23%.

| Asked | Result |
|---|---|
| A cold download shows only 206 responses from the asset host and ends `ready` | 32 requests to the asset host over the two halves of the second run, **all 206** (11 before the page was closed, 21 after). First run, in one go: 31 requests, the number of 8 MiB ranges of the seven files; the store went `unknown`, `absent`, `downloading`, `verifying`, `ready`; **62.9 s** for 214,647,815 bytes, hashing included |
| OPFS `models/asr-en-v1/` holds plain names and no `.part` | `config.json` 2,203; `decoder_model_merged_q4f16.onnx` 145,776,485; `encoder_model_q4.onnx` 66,178,491; `generation_config.json` 1,956; `preprocessor_config.json` 339; `tokenizer.json` 2,405,679; `tokenizer_config.json` 282,662. No `.part` |
| After a reload `inspect()` gives `ready` with zero model requests | `ready`; 0 requests to the asset host for a model file; `ensureReady` returns without one call of `onProgress` |
| The page closed at about 30% of a second cold download; reopened; `ensureReady`: the first `Range` starts at the `.part` size | Closed at 31.3% (67,111,067 bytes counted). Reopened: `config.json` final and `decoder_model_merged_q4f16.onnx.part` of **75,497,472** bytes, nine parts of 8 MiB. `inspect()` gives `partial`. The first request is `Range: bytes=75497472-83886079`. The download ends `ready` with seven final files |
| One hex digit of a manifest `sha256` changed | Done on `preprocessor_config.json` (339 bytes), with its final file removed first so that it is downloaded again: one request, 206; the store goes `partial`, `downloading`, `verifying`, `failed` with `E_MODEL_HASH`; `ensureReady` rejects with a `ModelFailure` whose `failure` is `{ code: "E_MODEL_HASH", stage: "model", retryable: true }`; **no final file and no `.part` exists for that name**. The digit restored (`git diff --quiet` on the manifest passes): one request, and `ready` again, from `failed` |

Not asked, and read in the same runs:

| Read | Result |
|---|---|
| The hash is the real SHA-256 | Yes: `verifyAndFinalize` ran with the stream of `offcut_core.wasm` on the main thread, and seven files matched the hashes that `sha256sum` wrote into the manifest in Prompt 38. The unit tests of Prompt 39 used a stand-in |
| The store and the caller see the same count | In 10 of 10 calls of `onProgress` the store's `done` was the `done` of the call |
| Time remaining | `null` at first, then 71, 57, 48, 47, 52 s in the first run |
| `cacheInfo()` | `{ modelId: "asr-en-v1", modelVersion: "small.en-timestamped-q4-q4f16@80853938", bytes: 214647815 }` |
| `clear()` | `models/` is empty; the store goes `ready` to `absent`; `inspect()` gives `absent` |
| Persistent storage | Asked for once: `meta` holds `persistRequested: true`. Chrome's answer was `false` (`navigator.storage.persisted()`), as expected for a page nobody has used; the download does not depend on it |

**The two drills.** Each was undone from a copy.

| Temporary edit | What fired |
|---|---|
| `E_MODEL_HASH` back in `COPY_PENDING` | `check-copy-codes`: "E_MODEL_HASH: has copy and is still in COPY_PENDING. Remove it from the list." Exit 1 |
| `150 MB` typed into `modelDownload.body` in place of the parameter | No rule about digits: ESLint fired on the parameter that was now unused (`no-unused-vars`), which is a rule about something else |
| The same, with the parameter still used ("at most 150 MB" added to the sentence) | **Nothing fires:** ESLint and `check-copy-codes` are clean. V1 has no tool for a typed-in digit before `messages.test.ts` (V8). The gap is as G 4.5 says |

The new strings were read for a digit: none. The size and the time are parameters.

**Decided here, where the plan is silent or cannot be built as written.**

- **The key `persistRequested` is written in `model-manager.ts` as a string of its own,** with a comment that names `META_KEYS.persistRequested`. The prompt asks for the guard by that name, which is in `persistence/schema.ts`; edge D-27 e lets the model manager import `persistence/db.ts` only, and `db.ts` (frozen) does not hand the key on. A sixteenth lint entry would have reopened a V1 contract that this prompt does not name. Cost: the name is in two files. If they ever differ, the browser is asked once more; nothing else reads the key. To settle when the specs are corrected (Prompt 60): widen edge e to `schema.ts`, or leave the copy.
- **`ModelFailure`** is the "`Error` that has a `failure: AppFailure` field" of §17.1. `retryable` is true for the four codes of `ModelError` (TS §11.2) and false for `E_INTERNAL`, which is what any other error becomes.
- **An abort rejects with the abort itself,** not with a `ModelFailure`: no error code means "the user stopped it". The store goes to `partial` (`net_fail`). V4 sends the first abort.
- **`ensureReady` inspects first when the store is still `unknown`,** so that `start` is a legal transition. `inspect()` writes to the store on its first call only; later calls answer and leave the store alone.
- **A file counts as final when a file of its final name has its manifest size.** That is the rule of `inspect()` (§17.1), used again in `ensureReady`.
- **`done` starts at the bytes on disk** (final files, and each part up to the size of its file) and never passes `total`.
- **The time remaining** is worked out from the oldest sample that is at least 5 s old, or from the start while the download is younger than that.
- **`cacheInfo().bytes` counts part files too:** it is what the model takes on the device.
- **`clear()` tells the store only from `ready` or `partial`,** the two states `cleared` is legal in.
- **The directory of all models** is the parent of `paths.modelDir(modelId)`: `opfs.ts` has no function for it (ten functions, TS §23.1), and no other file builds a path.
- **The panel shows the words of a failure** only for the three codes that have copy for it (D-57); for any other code it shows the sentence and the bar, and the page's stub says the rest (Prompt 55).
- **The bar's label** is the sentence above it (`aria-labelledby`): §20.3 gives the panel three strings, and none is a label.
- **The size shown is 210 MB:** 214,647,815 bytes, in units of 1,000,000, to the nearest 10 (TS §16.1). PS §10 J4 says "about 250 MB", which is the limit of D-66 and not this set.

**Differs from the prompt, the guide or the plan.**

- **The wrong-hash check changes the hash of the smallest file** and removes that one file first, so that 339 bytes are downloaded and not the whole model. The prompt does not say which file.
- **`import { http } from "../net/http";` in `model-manager.ts`** (G 4.4) was not repeated: the same drill fired in `download.ts` in Prompt 39, and the rule is one rule for the layer.

**Checked.**

- `node scripts/check-copy-codes.mjs`: ok, `ERROR_CODES` 9 of 29 with copy (was 6), pending 34 (was 37).
- `pnpm --filter web exec eslint .` and both `tsc` passes are clean. `model-manager.ts` uses edge D-27 d (the store, the manifest), e (`persistence/db.ts`) and f (the type `AppFailure`). The panel is mounted nowhere, and lints and type-checks.
- Lines: `model-manager.ts` 233, `ModelDownloadPanel.tsx` 78, `messages.ts` 207.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test` and `pnpm build` green; the frozen-file diff against the baseline is empty. **193 Rust, 92 Vitest**, as after Prompt 39.
- **`pnpm e2e` is green as the gate runs it:** 12 of 12 with Playwright's 6 workers, in 26 s. **12 Playwright.** The six failures of Prompt 39's gate did not come back, with nothing changed for them (known issue 20). This run holds the code of both prompts, so the whole gate is now green on the code of Prompt 39 as well. With `--workers=3`: 12 of 12.
- The app shell, gzip: 118.6 kB, was 118.2: the new copy and the three classes. The model manager and the panel are not in it yet: `start-app.ts` calls `inspect()` from Prompt 54 on, and the panel is mounted in Prompt 55.
- The temporary Playwright case, the browser profile and the dev server are gone; the manifest is as committed.

**"Done when".**

- [x] The four browser results hold; `node scripts/check-copy-codes.mjs` passes with three fewer pending codes.
- [x] Edge D-27 e is used and lints clean; the panel lints and type-checks without being mounted.

**Not checked.**

- **A full disk** (`E_MODEL_STORAGE`) and **three failed tries** (`E_MODEL_DOWNLOAD`) in a browser: both have unit tests (Prompt 39) and E2E cases in `model-download.spec.ts` (Prompt 56).
- **The panel on a page.** It is mounted in Prompt 55; until then nobody has seen it.
- The download on R1, and on a slower line than this one (about 27 Mbit/s).

**Human.** Push; read `ci`. Prompts 39 and 40 are not marked **Push**; two commits are waiting, and Prompt 41 ends with one.

## 2026-10-09 - The fifth push: `ci` green on Prompts 39 and 40

**Done by the human.** `git push` of `v2-build` at `1204389`, the commit of Prompt 40; the commit of Prompt 39 (`7534d13`) went with it.

**Read by the agent** (the public API of GitHub).

| Read | Result |
|---|---|
| `origin/v2-build` | `1204389`, equal to the local branch |
| The `ci` run on `1204389` (pull request #2, run 12) | Success. Job `ci`: 2.5 minutes, 35 steps passed, 2 skipped, none failed. `deploy-api` and `deploy-web`: skipped |
| Steps that ran on new code | Vitest (92, with the 16 of `download.test.ts`), the copy check (34 pending), ESLint and both `tsc` steps with the model manager and the panel, the secret scan, Playwright: all success |
| Production | Unchanged: `/api/v1/healthz` reports `322c7d3` |

**Closes.** The question the gate of Prompt 39 left open (known issue 20): on a clean Linux machine Playwright passes with its default workers. The six time-outs of that gate were the development machine.

**Changed.** This file only. The entry was written with the next commit.

## 2026-10-09 - Prompt 41: MP4 muxer

Prompts 41 and 42 were asked for in one sitting; each has its own gate and its own commit.

**Added.**

| File | Content |
|---|---|
| `crates/offcut-mp4/src/mux.rs` | `MuxSink`, `VideoTrackSpec`, `AudioTrackSpec` and `Mp4Muxer<S>` with the four signatures of TS §21.1; `MemSink`; `MOOV_RESERVE` |
| `crates/offcut-mp4/src/mux_boxes.rs` | One writer per box: `ftyp`, the headers of `free` and `mdat`, `mvhd`, `tkhd`, `edts` with its `elst`, `mdhd`, `hdlr`, `vmhd`, `smhd`, `dinf`, the two `stsd` (`avc1` with `avcC`, `mp4a` with `esds`), `stts`, `stss`, `stsc`, `stsz`, `co64`, and the containers `stbl`, `mdia` and `trak`. No `udta`, no rotation, no `ctts` |
| `crates/offcut-mp4/tests/mux_roundtrip.rs` | The 12 cases of §23.2, one of them proptest |

**Changed.**

| File | Change |
|---|---|
| `crates/offcut-mp4/src/lib.rs` | `pub mod mux;` and `mod mux_boxes;`; the first lines say the crate also writes |
| `docs/v2/v2implementation.md` | §6.8 and §23.2 say what was built (2 replacements, each applied once) |

No dependency was added.

**The file, as written.** `ftyp` (24 bytes, brands `isom` and `mp42`); the 256 KiB kept for `moov`; `mdat` with a 64-bit size, then the samples in the order they arrive. `finalize` writes the size of `mdat`, puts `moov` at the start of the reserve and a `free` box over what is left.

| Call | As built |
|---|---|
| `new` | `BadConfig` for an empty `avcc` or `asc`, a width or height of 0 or over 65,535, a sample rate other than 48,000, a channel count other than 2. Checked before the first write: a refused configuration writes nothing |
| `add_video_sample` | `OutOfOrder` unless `frame` is the number of frames so far; `BadConfig` when frame 0 is not a keyframe, and nothing is recorded, so the frame can be given again. A chunk holds 15 frames |
| `add_audio_sample` | `OutOfOrder` when `pts` goes back. The duration is kept in ticks of 1/48,000 s, to the nearest. A first `pts` below 0 is the priming length, to the nearest tick. A chunk holds 24 samples |
| `finalize` | `MoovOverflow` when `moov` is larger than the reserve, or leaves 1 to 7 bytes, which no `free` box fits. Returns the size of the file |

**The 12 cases of §23.2.**

| Row of §23.2 | Test |
|---|---|
| 90 video samples and 141 audio samples, muxed then demuxed | `ninety_frames_and_141_audio_samples_come_back_byte_for_byte` |
| The same file probed | `the_file_probes_as_h264_at_30_fps_with_aac_at_48_khz` |
| Box order | `the_boxes_are_ftyp_then_moov_then_mdat` |
| `finalize()` return value | `finalize_returns_the_length_of_the_file` |
| First audio sample with `pts = -21,333` | `a_first_audio_sample_before_0_becomes_an_edit_of_its_length` |
| First audio `pts = 0` | `a_first_audio_sample_at_0_writes_no_edit` |
| 2,700 video samples of 3,000 bytes and 4,220 audio samples | `ninety_seconds_fit_the_space_kept_for_moov` |
| Frame 1 first; frame 0 twice | `a_frame_out_of_its_turn_is_out_of_order` (also audio that goes back) |
| First video sample not a keyframe | `a_first_frame_that_is_not_a_keyframe_is_a_bad_configuration` |
| Empty `avcc`; 44,100; 1 channel | `an_empty_avcc_another_sample_rate_or_one_channel_is_a_bad_configuration` (also an empty `asc` and a width of 0) |
| proptest | `any_frames_with_any_sizes_and_keyframes_come_back`: 48 cases of 1 to 300 frames of 1 to 2,000 bytes with any keyframe pattern after the first, and 0 to 40 audio samples |
| Video metadata | `the_video_track_has_its_size_no_rotation_and_no_user_data` (also: no `udta`, `meta` or `ilst` at any depth; the six tables of the video track and no `ctts`) |

**Second opinion: `ffprobe` 9.0.2,** on the bytes of the first case, written for the check to `target/zz_mux.mp4`, and on the same clip with a first audio `pts` of -21,333.

| Read | No priming | Priming of 1,024 ticks |
|---|---|---|
| Video | `h264`, 1080x1920, 30/1, time base 1/30000, **90 frames**, 3.000 s | The same |
| Audio | `aac`, LC, 48,000 Hz, 2 channels, time base 1/48000, **141 frames**, 3.008 s | The same, **2.987 s**, start time 0: the edit is read |
| Top-level boxes | `ftyp` 24, `moov` 4,299, `free` 257,845, `mdat` 130,853 | - |

The payloads are random, so `ffprobe` reports that it cannot decode them, as G 4.6 expects; it reports no fault of the structure. The line that wrote the files is gone, and so are the files: `git grep -n zz_mux -- crates web scripts` prints nothing.

**The size of `moov` for 90 s** (2,700 frames, 4,220 audio samples), measured with a temporary line: 31,723 bytes when one track is written after the other, and **100,383 bytes when the two are written in turns**, where most samples are a chunk of their own. The reserve is 262,144.

**Decided here, where the plan is silent or cannot be built as written.**

- **A chunk also ends when the other track writes.** §6.8 starts a chunk every 15 or 24 samples; the samples of a chunk must lie one after the other in the file, and the two tracks write into one `mdat`. `stsc` holds one record wherever the number of samples in a chunk changes.
- **`MuxSink` is implemented for `&mut S` as well.** `finalize(self)` consumes the muxer and its sink with it, and the tests, like G 4.6, read the sink afterwards. The binding of Prompt 48 may own its sink or lend it.
- **What is left of the reserve is a `free` box,** so the top-level order is `ftyp`, `moov`, `free`, `mdat`.
- **The reserve is written as 256 KiB of zeros,** not skipped over: no file system has to fill a hole.
- **`frame_count_hint` sizes one table,** up to 8,192 entries, and limits nothing.
- **Durations.** A track's length in the movie timescale is rounded to the nearest millisecond. The audio track's `tkhd` and its `elst` both give the length without the priming; its `mdhd` gives all of it.
- **Small fixed values:** track ids 1 and 2; `hdlr` with an empty name; language undetermined; the `esds` holds stream id 0, object type 0x40, and no bit rates; `tkhd` flags "enabled, in the movie".
- **A sample of more than 4 GiB** is `BadConfig`: `stsz` holds 32 bits.
- **Audio that is not one run of equal frames** is written as it is given: `stts` holds one run per change of duration.

**Differs from the prompt, the guide or the plan.**

- **The demuxed priming frame reports -21,334 microseconds, not -21,333** (§23.2). 1,024 ticks of 1/48,000 s are 21,333.3 microseconds, and the demuxer rounds a time down (§6.5, as built in Prompt 35). The test says -21,334, and §23.2 of the plan says so now. The `elst` holds 1,024, as the row asks.
- **`mux_roundtrip.rs` starts with the same `#![allow(...)]` of four lints as V1's two integration test files.** `clippy.toml` lets a test function unwrap and index; a helper function in a `tests/` file is not a test function to clippy. No lint is switched off for code that ships.
- **The writers of `trak`, `mdia` and `stbl` are in `mux_boxes.rs`.** With them in `mux.rs` that file had 392 lines, and V5 adds `ctts` to it. Now: `mux.rs` 355, `mux_boxes.rs` 263.

**Checked.**

- `cargo test -p offcut-mp4`: lib 17 passed, `mux_roundtrip` 12 passed. `cargo clippy -p offcut-mp4 --all-targets -- -D warnings`: clean. `cargo deny check`: ok.
- `node scripts/check-file-tree.mjs`: 164 files; the three new files were in the tree of TS §5 since Prompt 31.
- G M-4: `cargo test -p offcut-mp4 -p offcut-dsp` green; Vitest green with `download.test.ts` at 16 cases; `pnpm check` and `pnpm build` green; the manifest line prints `asr-en-v1 true true` (against 260,000,000, D-66); `git grep -n zz_mux` empty. Its three browser checks are those of Prompt 40.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test` and `pnpm build` green; the frozen-file diff against the baseline is empty. **205 Rust, 92 Vitest** (Rust was 193; the 12 are new).
- **`pnpm e2e`, as the gate runs it, passed 11 of 12 in two runs.** The case that failed both times is "the settings link leads to the table of what leaves the device": it timed out at 30 s. It is the longest case, with about forty checks one after the other, and it takes 6 s when the machine is free. With `--workers=3`: **12 of 12**, in 20 s. This is known issue 20 again, on a machine that had just compiled the workspace. Nothing of this prompt is in the app: the muxer is exported by no bundle before Prompt 48, and `landing.spec.ts` is unchanged. Nothing was changed to make the suite pass; the `ci` run of the next push is the check.

**"Done when".**

- [x] `cargo test -p offcut-mp4`: lib and `mux_roundtrip` pass; box order is `ftyp`, `moov`, `mdat`, with a `free` box between the last two.
- [x] `ffprobe` agreed; `git grep -n zz_mux` is empty; G M-4 passes.

**Not checked.**

- **Real encoder output.** The payloads are random bytes; the first H.264 and AAC chunks reach the muxer in Prompt 50, and `verify/verify_mp4.py` reads the result.
- **The priming length of a real encoder** (TE-4, Prompt 51): the mechanism is tested with 1,024 ticks.
- The time `finalize` takes on R1 (budget 2.5 s, D-64).

**Human.** Push; read `ci`. This prompt is marked **Push**.

## 2026-10-09 - Prompt 42: `offcut-text` and its binding

**Added.**

| File | Content |
|---|---|
| `crates/offcut-text/src/tokenize.rs` | `TokenKind`, `Token`, `tokenize(words, edits)` (§8.2, D-42) |
| `crates/offcut-text/src/sentences.rs` | `segment_sentences`, `SENTENCE_GAP` (700 ms), `SENTENCE_MAX_WORDS` (40, declared and not used before V3) |
| `crates/offcut-text/src/numbers.rs` | `parse_quantity` and `format_quantity` (§8.4); `parse_decimal`, which the tokenizer also uses to tell a number in digits |
| `crates/offcut-text/src/normalize.rs` | `RawWord`, `normalize_transcript`: the only place a `Transcript` is built |
| `crates/offcut-wasm-core/src/text_api.rs` | The export `normalize_transcript(raw, model_version)` |

**Changed.**

| File | Change |
|---|---|
| `crates/offcut-text/src/lib.rs` | The four modules, and the names of TS §17.1 exported at the root |
| `crates/offcut-text/Cargo.toml` | `offcut-types`, `serde` with `derive`. Nothing else |
| `crates/offcut-wasm-core/Cargo.toml`, `src/lib.rs` | The dependency `offcut-text`; `mod text_api;`, the third module |
| `Cargo.toml` | `[workspace.dependencies]`: `offcut-text` (path) |
| `Cargo.lock` | The two new edges |
| `web/src/wasm/load-core.ts` | `CoreApi.normalizeTranscript(raw, modelVersion)`; the type `RawWord` |
| `docs/v2/v2implementation.md` | §8.1 and §8.4 say what was built (2 replacements, each applied once) |

No third-party dependency was added, and `deny.toml` was not touched: `offcut-wasm-core` was already a wrapper of `offcut-text`.

**Tests (inline): the 11 unconditional rows of §8.5.**

| Row of §8.5 | Test |
|---|---|
| `"$10,000."` | `tokenize::a_dollar_amount_with_a_full_stop_is_a_digits_token_and_a_punct_token` |
| `"40%"` | `numbers::forty_percent_is_one_digits_token_with_its_sign` |
| "ten thousand" | `numbers::ten_thousand_in_words_is_two_tokens_and_10_000` |
| "two hundred and fifty" | `numbers::two_hundred_and_fifty_is_four_tokens` |
| "three" | `numbers::three_is_3_with_no_unit` (also the whole range from zero to 999,999, and words that are not a number) |
| "ten million", "three point five", "2k" | `numbers::what_v3_will_read_is_not_guessed_at` |
| `$10,000`; `$1,500`; `$950` | `numbers::dollars_are_short_only_in_whole_thousands` (also every display rule of §8.4) |
| An edit replacing word 3 with `""` | `tokenize::an_edit_that_empties_a_word_leaves_no_token_for_it` |
| "Hi.", "There", "now" | `sentences::a_full_stop_and_a_pause_of_700_ms_each_end_a_sentence` (also 699 ms against 700 ms) |
| An empty-text word and a confidence of 1.4 | `normalize::a_word_without_text_is_dropped_and_a_confidence_of_1_4_becomes_1` |
| Same raw words twice | `normalize::the_same_raw_words_twice_give_equal_transcripts` |

One test more, in `offcut-wasm-core`: `text_api::a_raw_word_is_read_in_camel_case_and_nothing_else`, which reads a `RawWord` from JSON natively.

**Browser check (dev),** which the prompt does not ask for: `vite` on port 5173, a temporary Playwright case that calls `(await loadCore()).normalizeTranscript(...)` on nine raw words.

| Read | Result |
|---|---|
| The value returned | A plain object with the field names of the generated `Transcript`: `words` (`text`, `start_ms`, `end_ms`, `confidence`), `sentences`, `numbers`, `model_version` |
| The numbers | `$12,000` is `{ value: 12000, unit: "usd", display: "$12k" }` over words 2 to 3; "ten thousand" is `{ value: 10000, unit: "none", display: "10,000" }` over words 4 to 6 |
| The sentences | Two: words 0 to 7, which end with a full stop, and the word after it |
| The word with no text; a confidence of 1.4; one of `NaN` | Dropped; 1; 1 |
| The same words twice | The same JSON |
| No words | `{ words: [], sentences: [], numbers: [], model_version }` |
| A time of 1.5 ms; a string in place of the array; a word with only its text | Each throws the plain object `{ code: "E_INTERNAL", detail: "RawWords" }` |
| **A word with a fifth field** | **Not refused** (see "Differs") |

A confidence comes back as the 32-bit value it is kept in: 0.98 reads as 0.9800000190734863.

**Decided here, where the plan is silent.**

- **One `Punct` token per punctuation character.** "..." is three tokens.
- **Punctuation is anything that is not a letter or a digit,** at the two ends of a run of characters only. What is inside stays: `10,000`, `3.5`, `don't`, `forty-two`.
- **An edit may hold several words;** each becomes a token with the index of the word that was edited.
- **A number in digits:** digits, with groups of exactly three after each comma and at most three before the first, and at most one point with digits on both sides. `1,00` and `10,0000` are not numbers.
- **`$40%`** is one `Digits` token and no quantity.
- **A cardinal in words** is read from at most 11 tokens, the length of the longest one below a million. `forty-two` is one token and is read. Eleven to nineteen may stand before "hundred": "fifteen hundred" is 1,500, with no "thousand" after it.
- **An "and" that no number follows is not consumed:** "two hundred and then" is 200 in two tokens.
- **A cardinal followed by the word "point" is `None`,** which is what makes "three point five" `None` as row 6 asks, while "ten million" stays 10 in one token.
- **`format_quantity`:** a value that is not finite is written `0`; a value that rounds to zero has no minus sign; the part before the point is grouped also when there are decimals (`1,234.5`). The match names all 20 units, so a unit added later does not compile until it has a rule.
- **A confidence of `-inf` becomes 0 and of `+inf` 1;** `NaN` becomes 1, as §8.1 says.
- **A span's words** run from the word of its first token to the word of its last, so a number spoken in four words covers four.
- **A failure of the binding** is `{ code: "E_INTERNAL", detail: "RawWords" }`: the caller is the app's own worker, and a wrong shape is its fault.

**Differs from the prompt, the guide or the plan.**

- **`deny_unknown_fields` has no effect in the browser.** `RawWord` carries the attribute, and read from JSON an extra field is refused (the test in `offcut-wasm-core`). Through `serde-wasm-bindgen` it is not: that crate reads the four fields a struct names from the JavaScript object and never looks at another. The check on the dev page showed it. Nothing was added to refuse it by hand: the binding crate holds no rule of its own (TS §2), and the TypeScript type `RawWord` is what the one caller is compiled against. §8.1 of the plan says so now.
- **The conditional `dollars` form of D-42 is not added,** as the prompt says. Prompt 43 reads how the recognizer writes the amount.

**Checked.**

- `cargo test -p offcut-text`: 11 passed. `cargo clippy -p offcut-text -p offcut-wasm-core --all-targets -- -D warnings`: clean.
- `pnpm build:wasm`: `offcut_core_bg.wasm` is 458,236 bytes after `wasm-opt`, was 372,699. `wasm-bindgen` is 0.2.129 in `Cargo.lock` and from the CLI. `tsc --noEmit` and ESLint on `src/wasm` are clean.
- `node scripts/check-file-tree.mjs`: 169 files; `offcut-text` is checked as a pure crate, and the edge `offcut-wasm-core` to `offcut-text` is within the graph of TS §7. `cargo deny check`: advisories, bans, licenses, sources ok.
- `pnpm gen:types && sh scripts/check-gen-clean.sh`: no diff. `RawWord`, `Token` and `TokenKind` are not shared types (D-31).
- Lines above the test module: `numbers.rs` 262, `tokenize.rs` 87, `normalize.rs` 82, `sentences.rs` 40.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test` and `pnpm build` green; the frozen-file diff against the baseline is empty. **217 Rust, 92 Vitest** (Rust was 205: eleven in `offcut-text`, one in `offcut-wasm-core`).
- **`pnpm e2e` is green as the gate runs it:** 12 of 12 with Playwright's 6 workers, in 41 s. **12 Playwright.** With `--workers=3`: 12 of 12. This run holds the code of Prompt 41 too, so the whole gate is green on the muxer as well; the one time-out of that prompt's gate did not come back (known issue 20).
- The app shell, gzip: 120.1 kB, was 118.6: the media worker's chunk is 7.5 kB, was 6.0, with the glue of the new export.

**"Done when".**

- [x] `cargo test -p offcut-text` passes the 11 rows; `pnpm build:wasm` and `tsc` are clean.
- [x] `node scripts/check-file-tree.mjs` and `cargo deny check` accept the edge `offcut-wasm-core` to `offcut-text`.

**Not checked.** The words of a real recognizer: Prompt 43. How it writes the twelve thousand dollars of the reference clip decides whether D-42's conditional form is needed.

**Found, and not for this prompt.** A cardinal in words is read wherever it stands, as row 5 of §8.5 asks ("three" is 3), so the "one" of "this one is better" is a quantity too, with the display `1`. Captions show a quantity's display in place of its words (TS §17.2). Whether a lone small number in words should be shown as a digit is a question for the captions of Prompt 45 and for V3's rules, not for the parser.

**Human.** Push; read `ci`. Prompt 41 is marked **Push**; this commit goes with it.

## 2026-10-09 - The sixth push: `ci` green on Prompts 41 and 42

**Done by the human.** `git push` of `v2-build` at `792662f`, the commit of Prompt 42; the commit of Prompt 41 (`cc02715`) went with it.

**Read by the agent** (the public API of GitHub).

| Read | Result |
|---|---|
| `origin/v2-build` | `792662f`, equal to the local branch |
| The `ci` run on `792662f` (pull request #2, run 13) | Success |
| Production | Unchanged: `/api/v1/healthz` reports `322c7d3` |

**Closes.** The push that Prompt 41 ends with. The muxer, `offcut-text` and the larger core bundle build and pass on Linux; Playwright passes there with its default workers, as it did in the gate of Prompt 42.

**Changed.** This file only. The entry was written with the next commit.

## 2026-10-09 - Prompt 43: ORT copy step, ASR worker, host entries, first transcript

**REOPENED V1 CONTRACT: `scripts/check-hosts.mjs`** (D-38) **and `web/eslint.config.js`** (one rule). Nothing in either was loosened for a file that had passed before: the ESLint diff adds 19 lines and removes none, and every new entry of the hosts list holds in one named file only.

**The first transcript. The reference clip, in Chrome, with the model read from OPFS.** 157 words in 17 sentences; the first word at 1,560 ms, the last one ending at 71,520 ms; the amount as one `Usd` quantity with the display `$12k`. The WebGPU backend and the WASM backend give the same transcript, word for word. It is in `fixtures/speech/README.md`.

**It is too slow, and that is for Prompt 44 to judge.** On the development machine `load` and `transcribe` together took **65 s on WebGPU and 102 s on WASM** for the 74.7 s clip. The target of E-3 is 25 s on R1 and the line at which D-66's fallback applies is 50 s; R1 is the weaker machine. These are `dev` readings and decide nothing, but nobody should be surprised on R1. Details below.

**Added.**

| File | Content |
|---|---|
| `web/src/workers/asr/word-timestamps.ts` | `mergeWindows`, `postProcess`, `ASR_MIN_WORD` (40 ms), the type `AsrWindow` |
| `web/src/workers/asr/model-cache-adapter.ts` | `createModelCache(modelId)`: `match` answers from OPFS or throws; `put` does nothing |
| `web/src/workers/asr/whisper-runtime.ts` | `loadModel`, `transcribe`, `unloadModel`, the type `RawWord` (TS §16.2). The only importer of the runtime |
| `web/src/workers/asr.worker.ts` | `load`, `transcribe`, `unload`; its last statement is `serveWorker(...)` |

**Changed.**

| File | Change |
|---|---|
| `web/package.json` | `@huggingface/transformers` at exactly `4.3.1` |
| `package.json` | `pnpm.overrides`: two dependencies of that package are not installed (see "Differs") |
| `pnpm-lock.yaml` | The runtime with `@huggingface/jinja` 0.5.10, `@huggingface/tokenizers` 0.2.0, `onnxruntime-web` 1.31.0-dev.20260914-8d85527a0 and what that needs |
| `web/vite.config.ts` | The plugin `offcut:copy-ort` (at build start: two files into `web/public/ort/<version>/`, other version directories removed); one resolve condition (see "Differs") |
| `web/src/workers/pool.ts` | The row `asr` and the client `pool.asr`. Two rows now |
| `web/eslint.config.js` | The rule of TDR-3: `no-restricted-imports` for `@huggingface/transformers` and `onnxruntime-web`, and their subpaths, in every file of `src`; off in `workers/asr/whisper-runtime.ts` alone |
| `scripts/check-hosts.mjs` | An entry may carry `chunk`, a pattern for the path of an output file; 13 entries that carry one |
| `crates/offcut-text/src/numbers.rs`, `normalize.rs` | The split form of D-42 and its test row (see "The amount") |
| `fixtures/speech/README.md` | The section "The V2 reference clip as recognized": the transcript by sentence with times, its six numbers, the one expected event of V2, and where the transcript differs from Script A |
| `docs/v2/v2implementation.md` | D-42, §8.4, §8.5, §16.5, and two rows of §22.3 say what was built (6 replacements, each applied once) |

**Pinned.** `@huggingface/transformers` 4.3.1 (latest; Apache-2.0), exact. It resolves `onnxruntime-web` 1.31.0-dev.20260914-8d85527a0 (MIT), the version it names itself; that string is the name of the directory under `/ort/`.

**Every option of the runtime that is used,** read in the package's own source (`src/env.js`, `src/utils/hub.js`, `src/utils/cache.js`, `src/backends/onnx.js`, `src/pipelines/automatic-speech-recognition.js`), not taken from memory: the package is one major version past what the plan was written against.

| Option | Set to | What it does |
|---|---|---|
| `env.allowRemoteModels` | `false` | No file is asked of the model host |
| `env.allowLocalModels` | `true` | Has to be: with both kinds of loading off the runtime refuses to start. A "local" file is asked of the cache first |
| `env.fetch` | A function that rejects | Whatever the cache does not have would be fetched from `/models/...` on the app's own origin. It is refused before it is made |
| `env.useBrowserCache` | `false` | The runtime keeps nothing in the Cache API |
| `env.useFSCache` | `false` | No file system cache (there is none in a browser) |
| `env.useWasmCache` | `false` | The runtime does not fetch and keep ONNX Runtime's files itself |
| `env.useCustomCache`, `env.customCache` | `true`, the OPFS adapter | The model cache is OPFS |
| `env.backends.onnx.wasm.wasmPaths` | `{ mjs, wasm }` under `<origin>/ort/<version>/` | Where ONNX Runtime loads its loader and its module from; the default is a CDN |
| `env.backends.onnx.versions.web` | Read | The `<version>` of that path |
| `env.backends.onnx.wasm.proxy` | `false` | No helper worker from a `blob:` address |
| `env.backends.onnx.wasm.numThreads` | `clamp(hardwareConcurrency - 2, 1, 4)` | Threads of the WebAssembly module |
| `pipeline("automatic-speech-recognition", modelId, { device, dtype })` | `device` `"webgpu"` or `"wasm"`; `dtype` `{ encoder_model: "q4", decoder_model_merged: "q4f16" }` | The backend, and which file each half of the model is |
| The call `recognizer(audio, { return_timestamps: "word" })` | - | Word timestamps. `audio` is a `Float32Array` at 16 kHz |
| `pipeline.dispose()` | - | Releases the sessions |

Not used: `chunk_length_s` and `stride_length_s` (the windows are cut here, so that each is a cancellation point and a progress step), `language` and `task` (an English-only model takes neither), `env.remoteHost`, `env.localModelPath`.

**What the runtime asks its cache for** (read with a temporary line, since removed): seven names, each one of the seven files of the manifest, and nothing else. `config.json`, `tokenizer_config.json`, `preprocessor_config.json`, `tokenizer.json`, `onnx/encoder_model_q4.onnx`, `onnx/decoder_model_merged_q4f16.onnx`, `generation_config.json`, each asked as `/models/asr-en-v1/<name>`. The manifest of Prompt 38 is complete, and nothing was uploaded again.

**Browser check (dev, model cached).** `vite` on port 5173; Chrome under Playwright with a profile of its own; the model downloaded into OPFS by the model manager (47 s); the reference clip imported by `pool.media`.

| Asked | Result |
|---|---|
| About 150 to 190 words | **157** |
| Times increasing; the first under 3,000 ms; the last under 74,705 | No word starts before the one before it ended; first 1,560; the last word ends at 71,520 |
| `numbers` holds one `Usd` span for the amount, display `$12k` | `{ value: 12000, unit: "usd", display: "$12k" }` over words 132 and 133, after the split form was added (below). Five other numbers, none with a unit |
| `backend` | `webgpu` when asked for; `wasm` when asked for |

| Read in the same runs | WebGPU | WASM, 4 threads |
|---|---|---|
| `load` | 4.8 s and 5.9 s | 6.8 s |
| `transcribe` | 60.7 s and 58.9 s | 94.8 s |
| The three windows (30 s, 30 s, 24.7 s of audio) | 21 to 24 s, 22 s, 15 s | 35 s, 35 s, 25 s |
| Transcript | 157 words | The same, word for word and millisecond for millisecond |
| `unload` | 0.05 s; a second `unload` answers too; `transcribe` after it fails with `E_INTERNAL`, detail `NoModel` | The same |
| Progress | `{ stage: "asr", done: 0, total: 3, feed: { kind: "transcribing" } }`, then 1, 2 and 3 of 3 without a feed line | The same |
| `pcm16` after the call | Detached: it was transferred | The same |
| Requests while loading and transcribing | Only to `localhost:5173`: the two files under `/ort/1.31.0-dev.20260914-8d85527a0/` and the dev server's modules. Two more went to the asset host, for the demo video of the start page, which is not the recognizer's | The same |
| Workers | The media worker, the ASR worker, and three threads of ONNX Runtime started from `/ort/.../ort-wasm-simd-threaded.asyncify.mjs` | The same |

This is the dev server, which has no CSP. TE-1, on a production build with the CSP, is Prompt 44.

**The amount, and the form added for it (D-42).** The recognizer writes the twelve thousand dollars with a `$` and digits, as D-42 hoped, but as **two words: `$12` and `,000`**. The runtime starts a new word at a punctuation token, and the thousands separator is one. The first run therefore gave two quantities, `$12` and `0`. The prompt names two forms the amount might arrive in and asks for the form of §8.4 with its test row; this is a third form, handled the same way:

- **`numbers.rs`:** a word that begins with a comma and exactly three digits, straight after a whole number in digits, is the next group of that number. `$12` then `,000` is 12,000 dollars in three tokens, and the span covers both words. Nothing else that is written begins a word with a comma and a digit: a comma that belongs to the sentence ends the word before it, and `12,` then `000` stays 12.
- **Test row:** `numbers::a_number_the_recognizer_split_at_its_separator_is_one_number` (the form, its repetition, and seven cases that are not it), and three lines in the first test of `normalize.rs` (one span over both words). 12 tests in `offcut-text`, was 11.
- **The `dollars` form is not added.** The transcript has the `$`.
- It is in Rust and not in `word-timestamps.ts`: a rule about how a number is written belongs to `offcut-text` (TS §6), a worker file has no test in V2, and the words of the transcript stay what the recognizer returned.

**The three drills.** Each was undone from a copy.

| Temporary edit | What fired |
|---|---|
| `@huggingface/transformers` and `onnxruntime-web/webgpu` imported in `asr.worker.ts` | ESLint, `no-restricted-imports`, on both lines: "The speech runtime is imported by workers/asr/whisper-runtime.ts only (TDR-3)" |
| `https://huggingface.co/`, a listed literal, written into `web/src/main.tsx` | `check-hosts`, 1 problem: `web/dist/assets/index-DWT86Cga.js: https://huggingface.co/` |
| The `chunk` of that entry pointed at `assets/zz-nothing.js` | `check-hosts`, 1 problem: `web/dist/assets/asr.worker-B0hYuSuN.js: https://huggingface.co/` |

A fourth, not asked for and needed here (see "Differs"): `console.log(process.cwd())` in `web/src/main.tsx` fails `tsc` with TS2591 again.

**The 13 entries of `check-hosts.mjs`,** one per literal the failing build named. Two are addresses the runtime could ask; eleven are text.

| Literal | Where it may be | Why it is not a request |
|---|---|---|
| `https://huggingface.co/` (exactly) | The ASR worker's chunk | The runtime's default model host. Remote loading is off and `env.fetch` refuses; TE-1 (Prompt 44) proves no request |
| `https://cdn.jsdelivr.net/npm/onnxruntime-web@$` (exactly) | The same | The runtime's default place for ONNX Runtime's files, replaced by `/ort/<version>/` before the first session; TE-1 |
| `https://huggingface.co/docs/`, `https://github.com/huggingface/transformers.js/issues/`, `https://gist.github.com/hollance/`, `https://developer.mozilla.org/en-US/docs/Web/API/Cache`, `https://web.dev/cross-origin-isolation-guide/`, `https://rolldown.rs/in-depth/bundling-cjs` | The same | Addresses in the text of errors and warnings, of the runtime, of ONNX Runtime and of the bundler |
| `https://tinyurl.com/sudb9s96`, `https://docs.nvidia.com/cuda/cublas/`, `https://github.com/google/re2/wiki/Syntax`, `https://ieeexplore.ieee.org/document/1163711`, `https://arxiv.org/abs/1502.03167` | `ort/<version>/ort-wasm-simd-threaded.asyncify.wasm` | Text of operator descriptions compiled into ONNX Runtime |

The CSP was not touched: it names neither host, and a request to one would be refused by the browser.

**Decided here, where the plan is silent.**

- **The windows start every 25 s and are 30 s long;** the last one ends where the audio does. 74.7 s is three windows. A clip of 30 s or less is one.
- **`mergeWindows`:** in the 5 s two windows share, the earlier window's words are kept up to the middle and the later window's from there on; that is "the instance farther from a window edge". A word the two windows placed on either side of the middle, with the same text and overlapping times, is kept once.
- **`postProcess`:** a word that starts before the one before it ended starts where that one ended; a word shorter than 40 ms is lengthened to 40 ms, or to the start of the next word if that comes first.
- **`transcribe` returns the `CANCELLED` sentinel** when its flag is set, as `importToOpfs` does; TS §16.2 gives it no way to say so. `onWindow(0, total)` is called once before the first window, so that the feed line goes out as transcription starts.
- **A time is a whole millisecond,** rounded from the runtime's seconds: `normalize_transcript` refuses a fraction (Prompt 42). A word the runtime gives no end is given its start.
- **The runtime's answer is read as an unknown value** and checked field by field: the package types it as `any`.
- **`numThreads` is set for WebGPU too.** ONNX Runtime reads it once, when its module starts, so it cannot be set later for the fallback.
- **Failures:** anything `loadModel` or `transcribe` throws is `E_ASR_RUNTIME`; a `RangeError`, or a message that speaks of memory, is `E_ASR_OOM`. The detail is `Load`, `Transcribe` or `Unload`, never the runtime's text.
- **A part file is never read:** a name that ends in `.part` is refused before OPFS is asked.

**Differs from the prompt, the guide or the plan.**

- **Two dependencies of the runtime are not installed: `onnxruntime-node` and `sharp`** (`pnpm.overrides` in the root `package.json`). Neither is in the browser build of the runtime. **`sharp` broke a guard of Prompt 32:** the runtime's type files import it, its own type file asks for Node's types, and with it installed `console.log(process.cwd())` in `web/src/main.tsx` passed `tsc`. Found with the drill of G 1.4, which fires again. `onnxruntime-node` is 288 MB of native binaries that a browser app never loads.
- **`resolve.conditions` gains `onnxruntime-web-use-extern-wasm`** in `vite.config.ts`. §22.3 says nothing else changes. Without it the bundler followed a reference inside ONNX Runtime's default build and wrote a second copy of the module, 26.9 MB, to `web/dist/assets/`. With it there is one copy, under `/ort/`.
- **Two files are copied, not every `.wasm` of the package** (it has four builds, 86 MB): the loader and the module of the `asyncify` build, which is the one the WebGPU entry loads, for either backend.
- **The split form of D-42** (above) is a form the prompt does not name.
- **The lookups log** that showed which files the runtime asks for was a temporary line in `model-cache-adapter.ts`, removed with the temporary Playwright case.

**Checked.**

- `pnpm build` passes `check-hosts` with the runtime in `web/dist`: 9 files; `ls web/dist/ort/` shows one directory, `1.31.0-dev.20260914-8d85527a0`, with the two files; `web/dist/assets/` holds no ONNX Runtime module.
- ESLint, both `tsc` passes and `node scripts/check-file-tree.mjs` (173 files) are clean. `git diff web/eslint.config.js`: 19 lines added, none removed; the file has 377 lines.
- The secret scan (gitleaks 8.18.4, `--no-git`) on the changed files: no finding in 14 files.
- Lines: `whisper-runtime.ts` 180, `asr.worker.ts` 75, `word-timestamps.ts` 71, `model-cache-adapter.ts` 45, `pool.ts` 89, `vite.config.ts` 106, `check-hosts.mjs` 167; `numbers.rs` 297 above its tests.
- The app shell, gzip: **266.0 kB**, was 120.1: the ASR worker's chunk is 145.9 kB of it. It is fetched at app start by the `modulepreload` link, as every worker script is (D-32). The limit is 400 kB (TS §13.5, an assumption).
- `offcut_core_bg.wasm`: 459,281 bytes, was 458,236.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green, the last one with Playwright's 6 workers; the frozen-file diff against the baseline is empty. **218 Rust, 92 Vitest, 12 Playwright** (Rust was 217; the one is the new row).
- The temporary Playwright case, the browser profile, the two saved transcripts and the dev server are gone.

**"Done when".**

- [x] The three drills fired; `pnpm build` passes `check-hosts` with the runtime in `web/dist`; `ls web/dist/ort/` shows one version directory.
- [x] The transcript carries the dollar amount as a `Usd` quantity; every runtime option name used is in this entry.

**Not checked.**

- **The production CSP.** Everything above ran on the dev server. Whether ONNX Runtime's threads, its `import()` of the loader and its WebAssembly start under `script-src 'self' 'wasm-unsafe-eval'; worker-src 'self'` is TE-1.
- **R1.** Every time here is from the development machine, whose WebGPU adapter under Playwright is the integrated Intel one (known issue 9).
- **A cancel in the middle of a window:** the flag is read between windows only, so a cancel is seen after at most one window, 15 to 35 s here. V4 sends the first one.
- **Whether the transcript's nine differences from Script A are the recognizer's or the speaker's.** That needs a person to listen.
- A second run in the same worker without a reload, and memory.

**Human.** Read the timing above before Prompt 44: on these numbers the small-size model will not meet E-3 on R1, and D-66 names the base-size model as the fallback. Prompt 44 measures it properly; nothing is decided here.

## 2026-10-09 - Outside a prompt: D-67, a transcription time of up to 180 s is accepted

No code changed. Eight documents changed, all under `docs/`.

**The decision (founder, 2026-10-09): D-67.** The small-size model stays. A transcription time of up to 180 s on R1 for the reference clip is accepted in V2; the target stays 25 s and is recorded as missed; the base-size model is the fallback above 180 s, where the line had been 50 s. The founder's reason: a user can wait a minute or two longer for a better transcript. **E-3 at S6 is read from the founder's own reading,** about 150 s on an R1-class laptop, and Prompt 44 takes no timing on R1.

**What the agent had said first.** After Prompt 43 it reported 65 s on WebGPU and 102 s on WASM on the development machine and expected the small-size model to miss E-3 on R1. Asked whether speed mattered, it advised to compare the two model sizes on the same clip, for errors and for time, before deciding, because nothing has yet shown that the small-size model's transcript is the better one: the transcript of Prompt 43 differs from the script in nine places, and the base-size model has never been run. The founder decided without that comparison.

**The three answers the numbers come from** (asked before anything was written).

| Asked | Answered |
|---|---|
| What was the 150-second reading? | An R1-class laptop, 8 GB and an integrated GPU; `load` and `transcribe` of the reference clip |
| The new line above which the fallback applies | 180 s for the reference clip |
| How E-3 is recorded | As done now, from that one reading; Prompt 44 skips the timing on R1 |

**Derived by the agent from those answers, not given by the founder.** For a 60 s clip the line is 145 s (180 x 60 / 74.7), which is what the three specs now say where they said 40 s. The p90 total of E-3 is read against 379 s for the reference clip (224 - 25 + 180). Both follow by arithmetic; either can be set otherwise.

**Changed.**

| File | Change |
|---|---|
| `docs/v2/v2implementation.md` | D-67 added to §2, with its costs; §0 says "D-18 to D-67". D-64, D-66, §16.5 and the E-3 rows of §24.1 and §24.3 follow it. 7 replacements |
| `docs/v2/coding-promptsv2.md` | The Standard Agent Block says "D-18 to D-67". Prompt 44: no timing on R1; its "Done when" box asks for the founder's reading, at most 180,000 ms. The note on the two gates and the row of "Who does what" say so. 5 replacements |
| `docs/v2/v2buildguide.md` | The table of references; Step 5.6 (not run since D-67; its table reads 25,000 to 180,000 and over 180,000); Milestone 5; the two E-3 rows of Step 12.4. 8 replacements |
| `docs/product.md` | §20.4: the fallback applies above 145 s for a 60 s clip. §20.2: a status note, which says that the transcription budget is not met and that the three-minute promise does not hold on these numbers |
| `docs/technicalspec.md` | §16: the acceptance line names D-67; the contingency applies above 145 s. §30: the same row |
| `docs/buildplan.md` | §4 (the E-3 row and the gate row) and §11: 145 s where they said 40 s |
| `docs/v2/experiments.md` | The record of E-3 at S6, with what was and was not recorded |
| `docs/v2/v2changelog.md` | This entry; open item 3 rewritten; open item 10 added; known issue 18 corrected |

The edit of the six specification files was made by a script that refuses a replacement unless its old text occurs exactly once, and that writes nothing unless every replacement can be applied: 28 of 28. Each file keeps its line endings.

**Not changed, and why.**

- **The targets of PS §20.2** (20 s of transcription, 121 s median, 180 s p90) and the promise built on them. They are the product's targets; D-67 says that one of them is missed and accepted. A status note says what follows. Whether the promise or the model changes is open item 10.
- **The pitch and J10,** which say three minutes. The same open item.
- **E-4 and its bands.** D-67 is about transcription only.
- **The reading of S15** (Prompt 59: `pnpm bench:device` on R1, ten runs, the bench file of §26). It is still taken.
- **No code.** `MODEL_DTYPE`, the manifest and the files on the asset host are those of Prompts 38 and 43.

**What the record of E-3 lacks,** by the founder's choice and said in `experiments.md`: the median of five runs; the backend; the version of Chrome; the build that was run. **The agent does not know what was run on that laptop:** the commit of Prompt 43, the first one that can transcribe, had not been pushed when the reading was reported, so the laptop cannot have had it from GitHub. If the reading came from another program that runs the same model, it does not include what Offcut adds to the time (word timestamps, three windows with an overlap) or its settings.

**Checked.**

- `node scripts/check-file-tree.mjs` passes: the tree of TS §5 was not touched.
- `git diff --stat`: eight files, all under `docs/`. The frozen-file diff against the baseline is empty.
- `grep -n "50,000\|25-50 s\|above 40 s\|> 40 s" docs/` finds the old limits only where a sentence says what they were, and two numbers that are not limits of E-3.
- The four-command gate was not run: no file that a build or a test reads was changed. The test counts are those of Prompt 43: **218 Rust, 92 Vitest, 12 Playwright**.

**Open.**

- The S15 reading on R1, and TE-14: the estimate of the ASR phase is 40 MB under the memory budget (D-66).
- Whether the small-size model's transcript is better than the base-size model's. Nothing has compared them.

## 2026-10-09 - Prompt 44: TE-1, TE-2, first E-3 (gate)

**The gate says go.** TE-1 passed on both backends. TE-2 did not pass and its agreed fallback is taken. E-3 at S6 is the founder's reading of D-67, about 150,000 ms against a line of 180,000 ms, already in `experiments.md`; this prompt took no timing on R1.

**Before the prompt.** `v2-build` was two commits ahead of GitHub: Prompt 43 (`98e9238`) and D-67 (`beb0cc0`) had not been pushed, so `ci` has not run on either. The last green run is #13, on `792662f` (Prompt 42).

**Changed.**

| File | Change |
|---|---|
| `docs/v2/experiments.md` | The records of TE-1 and TE-2; their two rows of the table |
| `docs/technicalspec.md` | §39.3 item 7: the outcome of TE-2 (the place §24.1 names) |
| `web/src/workers/asr/word-timestamps.ts` | One comment line: the confidence is 1 because the recognizer returns no figure, not "until it is read" |
| `docs/v2/v2changelog.md` | This entry; known issues 18 and 23 corrected; known issue 24 added |

No other code changed. The hook `web/src/zz-spike.ts`, its import in `main.tsx`, the temporary Playwright case, one temporary line in `whisper-runtime.ts`, the browser profile and Chrome's network log were made for the experiments and are gone: `git status --porcelain` showed nothing before the first edit of this entry.

**TE-1: passed.** A production build under the CSP of `vercel.json`, served by `vite preview` at `http://localhost:4173`; Chrome 155.0.8059.39, headless; two runs, each on WebGPU and on WASM.

| Read | WebGPU | WASM |
|---|---|---|
| Requests of the page and its workers from `load` to `unload` | 7 and 8, all to `localhost:4173` | 8 and 8, all to `localhost:4173` |
| Console lines, page errors | 0 | 0 |
| The transcript | 157 words, equal to `fixtures/speech/README.md` | The same |
| The amount | `$12k`, words 132 and 133 | The same |
| `load` + `transcribe`, run 1 and run 2 (`dev`) | 95.5 s, 61.9 s | 123.4 s, 98.1 s |

The requests are the two worker scripts, `offcut_core_bg.wasm` twice, the two files under `/ort/<version>/`, the clip the test handed over, and the app's own `POST /api/v1/events`. The three thread workers of ONNX Runtime start from the `.mjs` file on the app's origin, none from a `blob:` address. The model came from the asset host once (31 requests, 70.8 s) and from OPFS after that.

**A second record, and what it showed.** Chrome's own network log of the whole browser, read for the same time windows: 9 requests to `localhost:4173` in each, started by the app's origin, and 3 to hosts of Google (an update check and the push-message registration), each marked by the log as started by no page. Those are the browser's own and are in no record of the page or its workers. They are written into `experiments.md` so that nobody later reads "no request" as "the browser is silent": TE-1 is about what Offcut asks for.

**TE-2: not passed; `confidence` stays 1.0.** The runtime computes the probability of each token it picks and does not return it: the last lines of its generation loop are a `TODO` for `scores` and `logits`, `output_scores` is declared and read nowhere, and the result of the speech pipeline has `text` and `chunks`, each chunk `text` and `timestamp`. Seen in the source and in the running build: with `output_scores: true` the fields were the same in 6 of 6 windows. The fallback of §24.1 was agreed in advance, so this is no new decision. The dollar span has a confidence of 1.0.

**What that costs.** Nothing in V2 lowers a score for a word the recognizer was unsure of. V3's detector multiplies by word confidence (TS §17.4) and will multiply by 1 unless TE-2 is run again there. `experiments.md` says how a probability could be had and why it was not built: a `logits_processor` gives one per token, and nothing the package exports says which tokens make which word.

**The option names** (§24.1 asks for them here). They are those of the entry of Prompt 43, and they hold in a production build under the CSP: `env.allowRemoteModels = false`, `env.allowLocalModels = true`, `env.fetch` set to a function that refuses, `env.useBrowserCache`, `env.useFSCache` and `env.useWasmCache` all `false`, `env.useCustomCache = true` with `env.customCache`, `env.backends.onnx.wasm.wasmPaths = { mjs, wasm }` under `/ort/<version>/`, `wasm.proxy = false`, `wasm.numThreads` from 1 to 4; `pipeline("automatic-speech-recognition", modelId, { device, dtype })`; the call option `return_timestamps: "word"`.

**Seen and not explained.** The first transcription in a new browser profile was slower than the second, by about half on WebGPU and a quarter on WASM. It followed a build and the download of the model, which may be all of it. Known issue 24.

**Gate.**

| Step | Result |
|---|---|
| `git status --porcelain \| grep -c "zz[-_]"` | 0 |
| `env \| grep -c VITE_ENTITLEMENT_TEST_PUBLIC_KEY` | 0 |
| `pnpm check` | Green |
| `pnpm test` | Green: **218 Rust** (2 ignored, as before), **92 Vitest** |
| `pnpm build` | Green; `check-hosts: ok`, 9 files in `web/dist`, the only host is the asset host |
| `pnpm e2e` | Green: **12 Playwright**, with the 6 workers of the default, in 26 s |
| The frozen-file diff against `322c7d3` | Empty |

The counts are those of Prompt 43: this prompt adds no test. The gate ran before its own two tables were written into this entry; nothing else changed after it.

**Milestone 5 (G M-5).**

| Command | Result |
|---|---|
| `cargo test -p offcut-text` | 12 passed, 0 failed |
| `pnpm check`, `pnpm build` | Green (the gate above) |
| `ls web/dist/ort/` | One directory, `1.31.0-dev.20260914-8d85527a0` |
| The removal check: `git grep` for the hook's two marks in `web/src` | No output |
| The count of lines of `experiments.md` that name TE-1, TE-2 or E-3 (at least 3) | 15 |

**Done when.**

- [x] TE-1 and TE-2 each have a recorded outcome: TE-1 passed, TE-2's fallback taken. The dollar span's confidence is 1.0.
- [x] The hook is gone: `git grep -n "zz-spike" -- web/src` is empty; G M-5 passes.
- [x] E-3: the founder's reading of D-67 is in `experiments.md`, about 150,000 ms, not above 180,000 ms.

**Open, for the human.** Push `v2-build` (three commits: Prompt 43, D-67, Prompt 44) and read `ci` on pull request #2. Prompt 45 waits for green.

## 2026-10-09 - The seventh push: `ci` green on Prompts 43 and 44

**Done by the human.** `git push` of `v2-build` at `ece9d60`, the commit of Prompt 44; the commits of Prompt 43 (`98e9238`) and of D-67 (`beb0cc0`) went with it.

**Read by the agent** (the public API of GitHub).

| Read | Result |
|---|---|
| `origin/v2-build` | `ece9d60`, equal to the local branch |
| The `ci` run on `ece9d60` (pull request #2, run 14) | Success, in 179 s. The two deploy jobs were skipped, as on every run of the branch |
| Production | Unchanged: `/api/v1/healthz` reports `322c7d3`, on the API and through the web host |

**Closes.** The push that Prompt 44 ends with, and the one Prompt 43 had left open. The speech runtime, the copy step of its files, the host entries and the split-number rule build and pass on Linux for the first time: `ci` had not run on any of the three commits before. Playwright passes there with its default workers.

**Not shown by this run.** `ci` transcribes nothing: no case loads the model. TE-1 and the transcript were read on the development machine only (entry of Prompt 44); the media job that runs them in CI is Prompt 58.

**Changed.** This file only. The entry was written with the next commit.

## 2026-10-09 - Prompt 45: `offcut-scene`

Prompts 45 and 46 were asked for in one sitting; each has its own gate and its own commit. The entry "The seventh push" above is committed with this one.

**Added.**

| File | Content |
|---|---|
| `crates/offcut-scene/assets/fonts/Inter-Variable.ttf`, `JetBrainsMono-Variable.ttf`, `NotoEmoji-Variable.ttf` | The three variable fonts, unchanged apart from the file name |
| `crates/offcut-scene/assets/fonts/LICENSES.md` | Where each file was taken from, its SHA-256, and the licence file shipped with each, as shipped |
| `src/safe_area.rs` | The constants of TS §19.1: canvas 1080 x 1920 lp; insets 250, 420, 60, 120; `SAFE_AREA`, `EVENT_ZONE` (y 860 to 1180), `CAPTION_ZONE` (y 1220 to 1500), `WATERMARK_ANCHOR` |
| `src/easing.rs` | `Easing { Linear, EaseOutCubic, BackOut }`, `ease(e, x)` |
| `src/anim.rs` | `Track { keys, easing }`, `eval(track, t)`: the value is computed from `t` alone |
| `src/display_list.rs` | `DisplayList` and `DrawCmd` of TS §19.2; `Rect`, `Rgba`, `PathEl`, `PositionedGlyph`, `Stroke`, `Affine`, `FontId` (D-29); the output scale of a command |
| `src/styles.rs` | `StyleSpec`, `Case`, `spec(id)` with three arms; `Bold` and `Tech` return the Clean row |
| `src/fonts.rs` | The three files with `include_bytes!`; `bytes(FontId)`, `weight(FontId)`; the parley contexts |
| `src/layout.rs` | `ShapedRun`, `shape(text, font, size)` |
| `src/framing.rs` | `crop_rect(clip, offset)`, the formula of TS §19.3 with the offset read as 0.0 on one marked line |
| `src/captions.rs` | `CaptionWord`, `LineWord`, `Line`, `Chunk`; `words(transcript, edits)`, `chunk(words, sentences, style)`, `draw(chunk, t, style, out)` |
| `src/events/mod.rs` | `EventTrack`, `layout(e, style)` with the four `EventParams` arms written out, `draw(track, t, out)` |
| `src/events/number_reveal.rs` | `Track`, `layout`, `draw`, `visible_at`, and the constants of TS §19.5 |

**Changed.**

| File | Change |
|---|---|
| `crates/offcut-scene/src/lib.rs` | The ten modules; `SceneError { Font, Layout }`, `SceneInput`, `CropRect`, `Scene`, `build_scene`; `duration`, `frame_count`, `crop`, `frame_at`, `summary` |
| `crates/offcut-scene/Cargo.toml` | `offcut-types`, `offcut-text`, `parley` (feature `std`), `serde` (`derive`), `thiserror` |
| `Cargo.toml` | `[workspace.dependencies]`: `parley = { version = "0.11.1", default-features = false }` |
| `Cargo.lock` | `parley` and the 15 packages it brings |
| `docs/v2/v2implementation.md` | §11.3, §11.4, §11.5 and §11.7 say what was built (4 replacements, each applied once) |

`deny.toml` was not touched: no new license, and no wrapper was needed.

**Pinned.** `parley` 0.11.1, the latest release, without its default features and with `std`. It resolves `fontique` 0.11.1, `harfrust` 0.12.0, `skrifa` 0.44.0, `read-fonts` 0.41.0 and the `icu_*` crates at 2.3.

**The set for Prompt 47, written down now as the prompt asks.** `vello` 0.11.0, the latest release; it asks for `wgpu ^30.0.0` (the newest 30 is 30.0.1), `peniko ^0.6.1` and `skrifa ^0.44.0`. `parley` 0.11.1 asks for `skrifa ^0.44.0` too: the one crate the two share resolves to one version. Both are above the installed Rust's minimum (1.89 and 1.88; installed 1.99).

**`parley` is taken without its feature `system`.** The default reads the fonts of the machine: DirectWrite on Windows, fontconfig on Linux, which the CI runner would have had to have installed. A scene would then depend on the machine it was built on. Without it the collection holds the three embedded files and nothing else, and it is built with `system_fonts: false` as well.

**The fonts.**

| File | Font | Taken from | Licence |
|---|---|---|---|
| `Inter-Variable.ttf`, 879,708 bytes | Inter 4.1 | `InterVariable.ttf` in `Inter-4.1.zip`, release `v4.1` of `rsms/inter` | SIL OFL 1.1 (`LICENSE.txt` of the archive) |
| `JetBrainsMono-Variable.ttf`, 303,144 bytes | JetBrains Mono 2.304 | `fonts/variable/JetBrainsMono[wght].ttf` in `JetBrainsMono-2.304.zip`, release `v2.304` of `JetBrains/JetBrainsMono` | SIL OFL 1.1 (`OFL.txt` of the archive) |
| `NotoEmoji-Variable.ttf`, 1,982,596 bytes | Noto Emoji 3.002, monochrome | `ofl/notoemoji/NotoEmoji[wght].ttf` of `google/fonts` at commit `8b0a1d0f` | SIL OFL 1.1 (`OFL.txt` beside it) |

The licence was read in each of the three files, not assumed: all three are SIL OFL 1.1, and none names a Reserved Font Name. **Noto Emoji is not taken from a release of its own project:** the repository that `google/fonts` names as its source (`googlefonts/emoji-bw`) answers "not found". The Google Fonts repository is where the variable file is published; it is pinned by commit. The version strings inside the files read 4.001, 2.304 and 3.002.

**Decided here, where the plan gives a name and no shape, or is silent.**

- **What a glyph run means.** A glyph's `x` and `y` are in the run's own space, from the start of its baseline; `transform` puts the run on the canvas. `size`, the glyph positions and the stroke width are lengths of that space. The output scale (`profile.width / 1080`) multiplies every length of a command and the translation of a transform, and leaves the scale part of a transform as it is; so at a width of 540 every stored length is half. TS §19.2 names the fields and not their spaces.
- **`fonts::bytes(FontId)` and `fonts::weight(FontId)`.** A glyph id means nothing without its file and its weight: all three files are variable. `Inter700` and `JetBrainsMono700` are weight 700, `Inter900` is 900, and `NotoEmoji` is 700, the end of its axis and the weight every style asks for or more. The renderer of Prompt 47 reads both.
- **The context is built once per thread,** on the first shaping call, and kept in a `thread_local`. §11.3 says "once per scene", and gives `shape(text, font, size)` no context to take. The signatures of §11.3, §11.5 and §11.7 are as given; the result of a shaping call does not depend on the calls before it, and the test of row 8 builds one scene with a new context and one with a used one.
- **Which font a run used** is read from the id of its data, which is the id of the file that was registered.
- **A missing character.** parley sets a character the style font lacks in Noto Emoji (seen with a rocket: three runs). A character both lack comes back as glyph 0: it is left out with its text, and takes no room. Seen with a Chinese character.
- **`CaptionWord`** is `{ text, start, end, word }`; `word` is the first transcript word it stands for, which is how its sentence is found. **`Line`** is `{ words, x, baseline, width, scale }`.
- **When a chunk is shown.** From 80 ms before its first word to 120 ms after its last, start included and end not. Where two chunks would overlap, the earlier one stays until its own last word has ended and the later one starts then, or 80 ms before its first word if that is later. Words 20 ms apart across a full stop: the first chunk ends and the second starts at the end of the first one's last word.
- **A word too long for a line** (over 18 characters) has a line of its own.
- **The active colour** is a yellow, `rgb(255, 214, 10)`, on white text with a dark stroke of 6 lp. The app's accent is a blue made for a light page; on video it would not read. An assumption like every value of TS §19.4 (E-2).
- **`Case`** is in `StyleSpec`, and `chunk` applies it, so that V4 only writes the two rows.
- **NumberReveal.** The count-up has 15 texts, one for each output frame that starts inside its 500 ms, and the display of the quantity from 500 ms on: 16 at most. It uses the style's easing. A number scales in about the middle of its own box. The fill and the stroke are the style's. The reveal is shown from the start of its first word until the fade has ended, 1,400 ms after its last word. The fade is one layer with an opacity.
- **The label row is drawn,** at 48 lp under the number, when `label` is `Some`; the block of number and label is centred in the zone. V2's detector gives none.
- **An event is drawn when its `enabled` is true,** or the user's override for its id is.

**Differs from the prompt, the guide or the plan.**

- **A number is fitted by the widest text it shows, not by its end.** §11.7 fits the final display. `$250k` counts up through `$249,999`, which is 967 lp wide at 200 lp: it would leave the safe area for half a second, against TS §19.6. The size is the one at which every text of the count-up fits, so it does not change during the reveal. For the reference clip nothing changes: `$12k` passes `$11,480`, 771 lp.
- **The room is 900 lp less the stroke width,** for the number and for a caption line, so that the stroke of the outermost glyph stays in the safe area too.
- **A caption line wider than the zone is scaled down.** The limit of a line is 18 characters, and 18 wide capitals at 64 lp are 1,219 lp. The plan does not name the case; TS §19.6 forbids the result.
- **A word that would need a third line starts a new chunk.** §11.5 limits a chunk to `max_chars_per_line x max_lines` characters; three words of 10 characters are 32 and still need three lines.
- **`visible_at`** on a chunk and on a reveal, and **`captions::words`,** which is step 3 of `build_scene`: `lib.rs` would have held it otherwise.
- **`crop_rect` compares the two sides in whole numbers** (`width x 16 > height x 9`), so that a 1080 x 1920 clip is exactly 9:16 and not a rounding away from it.
- **The transcripts of the tests are built by `normalize_transcript`.** D-28 lets the crate use `offcut-text` for `format_quantity` only; that holds for the crate's code. A test that made a `Transcript` by hand would have had to write its sentences and numbers itself, and TS §10.8 makes that function the only builder.

**Tests (inline): the 11 rows of §11.8.**

| Row of §11.8 | Test |
|---|---|
| A 60,000 ms clip; a 20,033 ms clip | `tests::a_60_000_ms_clip_has_1_800_frames_and_a_20_033_ms_clip_601` (also 90 s: 2,700; the reference clip: 2,242; 0 ms and 1 ms) |
| 12 words in one sentence, Clean | `captions::tests::twelve_words_in_one_sentence_make_chunks_of_at_most_5_words_and_2_lines_of_18_characters` (also: the order of the words; no two chunks overlap) |
| Words at 1,000-1,300 ms and 5,000-5,300 ms | `captions::tests::words_a_pause_apart_are_two_chunks_and_nothing_shows_in_the_pause` (shown 920 to 1,420 and 4,920 to 5,420; also 349 ms against 350 ms, and a full stop) |
| `frame_at(t)` inside a chunk | `captions::tests::inside_a_chunk_exactly_one_glyph_run_uses_the_active_colour` (none between two words) |
| NumberReveal `$10k`, span 2,000-2,400 ms | `events::number_reveal::tests::ten_thousand_dollars_counts_up_holds_and_goes`: nothing at 1,900; `$0` at 2,000; `$4,880` at 2,100; `$10k` from 2,500; half faded at 3,700; nothing at 3,800 and 3,900 (also a number that scales in, and a label) |
| A NumberReveal whose display cannot fit at 96 lp | `events::number_reveal::tests::a_number_that_cannot_fit_at_96_lp_is_dropped`: `summary().visual_moments` is 0 and no frame draws it (also `$250k`, made smaller, with every text of its count-up in the room) |
| Every command of every frame of the above | `tests::every_command_of_every_frame_is_inside_the_safe_area`: 300 frames with captions, a line of 18 wide capitals, a word of 20 characters, and three reveals |
| `build_scene` twice, all frames | `tests::two_builds_of_one_input_draw_the_same_frames` (also a frame asked for again, out of order) |
| 1920x1080 clip, any offset | `framing::tests::a_1920x1080_clip_is_cropped_to_its_centre_strip_at_any_offset` (five offsets; a square) |
| 1080x1920 clip | `framing::tests::a_1080x1920_clip_keeps_the_whole_frame` (also 900 x 1920: top and bottom go) |
| Profile width 540 | `tests::at_a_profile_width_of_540_every_coordinate_is_half`: the size, the stroke width, and where every glyph lands |

**The three drills.** Each was undone from a copy.

| Temporary edit | What fired |
|---|---|
| `wgpu = "*"` in `offcut-scene` | `check-file-tree`: "offcut-scene: resolves wgpu, which a pure crate must not use (TS §7)". `cargo deny check bans`: "crate 'wgpu = 30.0.1' is explicitly banned", and the same for `web-sys`, `js-sys` and `wasm-bindgen` |
| A `HashMap` in `captions.rs` | **Nothing fires,** as the prompt expects: `clippy.toml` has no `disallowed-types` entry, and the file is frozen in V2. The crate was read for it instead: `grep -c HashMap` is 0 in every file |
| A `.ttf` copied to `web/public/zz.ttf` | The font `grep` of G 6.1 prints 4 lines |

**Checked.**

- `cargo test -p offcut-scene`: 11 passed. `cargo clippy -p offcut-scene --all-targets -- -D warnings`: clean.
- `node scripts/check-file-tree.mjs`: 188 files; `offcut-scene` is checked as a pure crate and its edge to `offcut-text` is within the graph. `cargo deny check`: advisories, bans, licenses, sources ok.
- `cargo check -p offcut-scene --target wasm32-unknown-unknown` passes, and the tree for that target holds no browser or randomness crate. Not asked for; Prompt 48 builds on it.
- The font `grep`: 3 files, all under `crates/offcut-scene/assets/fonts/`.
- Lines above the test module: `captions.rs` 288, `number_reveal.rs` 235, `display_list.rs` 180, `lib.rs` 150, `fonts.rs` 129, `layout.rs` 109.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green, the last one with Playwright's 6 workers; the frozen-file diff against the baseline is empty. **229 Rust, 92 Vitest, 12 Playwright** (Rust was 218; the eleven are new).
- The app is unchanged: no bundle holds the crate before Prompt 48. The app shell is 266.0 kB, as after Prompt 43.

**"Done when".**

- [x] `cargo test -p offcut-scene` passes the 11 rows; clippy is clean; `check-file-tree` accepts `offcut-scene` to `offcut-text`.
- [x] `git ls-files | grep -ci "\.\(ttf\|otf\|woff2\?\)$"` is 3 once the files are committed, all under `crates/offcut-scene/assets/fonts/`.

**Not checked.**

- **How long a scene takes to build.** The budget is 1.25 s for the reference clip on R1 (D-64). Nothing was timed: a pure crate may not read a clock, and a reading on this machine, natively, says little about WASM on R1. The bench of Prompt 58 reads it.
- **What the frames look like.** No pixel is drawn before Prompt 47; the positions were checked by numbers only. The colours and sizes are assumptions until a person looks (Prompt 55, E-2).
- **The size the crate adds to `offcut_render.wasm`:** 3.2 MB of fonts, and the text data parley brings. Read in Prompt 48.

**Found, and not for this prompt.**

- **A caption shows a number spoken as one small word as a digit.** Row 2's twelve words hold "three", and the caption says "3" (TS §17.2: a caption shows the display of a number in place of its words). On the reference clip "three reasons" becomes "3 reasons". Known issue 22 left the choice to this prompt; the spec is followed, and a rule that keeps small numbers as words belongs to V3's number rules.
- **The display of a number has no punctuation.** "$10,000." is shown as `$10k`: the full stop that was part of the word is gone. Keeping it means knowing which characters of a word belong to the number, which is `offcut-text`'s to say (TS §6), not the scene's.
- **The caption and the reveal show the amount at once:** `$12k` in the caption line and `$12k` large above it. TS does not say that a caption leaves out what a reveal shows.

## 2026-10-09 - Prompt 46: `offcut-detect`, `offcut-entitlement`, token vector

**The token format is proven across the two languages.** A token minted in TypeScript by `mintEntitlementToken` verifies in Rust with the test public key, and the same token with one character of its first segment changed is refused with `Signature`.

**Added.**

| File | Content |
|---|---|
| `crates/offcut-detect/src/config.rs` | `DetectorConfig` with its ten fields and the defaults of §9.1 |
| `crates/offcut-detect/src/event_id.rs` | `event_id(kind, anchors)`: FNV-1a 64 over the nine bytes of D-47 |
| `crates/offcut-detect/src/number.rs` | `Candidate`, `find(toks, t, p, cfg)` |
| `crates/offcut-entitlement/src/claims.rs` | `EntitlementClaims` of TS §10.6, with `deny_unknown_fields`; `CLAIMS_VERSION` |
| `crates/offcut-entitlement/src/token.rs` | `ParsedToken`, `decode`, `encode`, `signing_input`, `MAX_TOKEN_CHARS` (D-25, D-60) |
| `crates/offcut-entitlement/src/verify.rs` | `verify_token(token, public_keys)` |
| `crates/offcut-entitlement/src/profile.rs` | The five constants of §10.4 and `export_profile(claims, now)`, the only builder of an `ExportProfile` |

**Changed.**

| File | Change |
|---|---|
| `crates/offcut-detect/src/lib.rs` | The three modules; `detect(t, p, edits, cfg)`, the five steps of §9.2 |
| `crates/offcut-detect/Cargo.toml` | `offcut-types`, `offcut-text` |
| `crates/offcut-entitlement/src/lib.rs` | The four modules; `TokenError` with five variants; the test vector |
| `crates/offcut-entitlement/Cargo.toml` | `offcut-types`, `base64`, `ed25519-dalek` (see "Differs"), `serde` (`derive`), `serde_json`, `thiserror` |
| `Cargo.lock` | The edges of the two crates. No package is new: all of them were in the lock for the server |
| `web/tests-e2e/helpers/fake-api.ts` | `TEST_ENTITLEMENT_SEED`, `testPublicKeyBase64`, `mintEntitlementToken`, `seedEntitlement` (§23.4) |
| `docs/v2/v2implementation.md` | §9.2, §10 (crate rules), §10.3 and §23.4 say what was built (4 replacements, each applied once) |

`Cargo.toml` of the workspace and `deny.toml` were not touched.

**Pinned.** Nothing new. `ed25519-dalek` 3.0.0, `base64` 0.23.1 and `serde_json` 1.0.151 are the versions the workspace already had for the server.

**The test key.** The seed was made in this prompt, with `crypto.randomBytes(32)`; the step G 0.5 had left it for here (entry of Prompt 31). It is a constant of `fake-api.ts` and of no other file. **The test public key, a public value, in standard base64, is `mCydb6uLK58faqtJ0v3Ol8tnmNboOoH6t/RD6rcXxxM=`.** It is what `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` must be for a build the E2E tests run against (Prompts 56 to 58), and what the test-key guard of Prompt 58 looks for in a deployed bundle. No deployment accepts a token of this key.

**The vector.** Minted through the temporary `web/tests-e2e/zz-mint.spec.ts` of G 6.5, with `{ plan: "creator", iat: 1760000000, exp: 1760604800, periodEnd: 1762592000 }`. Its claims: `v` 1, `sub` `0190f3a2-7b1c-7def-8a55-0123456789ab`, plan `creator`, 3 free exports remaining. The token (293 characters), the 32 bytes of the public key and the claims are constants of the test module of `crates/offcut-entitlement/src/lib.rs`. A second token, for the plan `free` with 2 exports remaining, is there too. Three things the constants show beyond the row of §10.5:

- The first segment is exactly what `signing_input` writes for the same claims: `JSON.stringify` in TypeScript and `serde_json` in Rust give the same bytes for them, field for field.
- `encode(claims, signature)` gives the token back, character for character. V6's signer builds on those two functions.
- Minting twice gives the same token: an Ed25519 signature has no random part.

**`seedEntitlement`, tried in a browser** (the same temporary file; a production build under `vite preview`).

| Asked | Result |
|---|---|
| The app opened, then `seedEntitlement(page, token)`; the record read back | `{ schemaVersion: 1, value: { token, storedAt } }` under the key `current` of the store `entitlement`; the database is still at version 1 with its eight stores |
| The same call on a page of the origin where the app never ran (`/robots.txt`) | Rejects with "the app has not made its database yet", and `indexedDB.databases()` is empty afterwards: nothing was made |

**Decided here, where the plan is silent.**

- **Whether a number still has its words** is read from the tokens: `find` is given the effective tokens and no edits, so it compares the tokens of the number's words with the tokens of what was transcribed. An edit that leaves them the same (a capital letter) is no edit to the detector.
- **The score is added in the order of the table** (base, unit, magnitude, energy), clamped to 1.0, then multiplied by the lowest confidence of the anchor words. The order matters in 32-bit numbers: 0.50 + 0.30 is exactly the threshold 0.80, which is why `40%` at confidence 1.0 is an event.
- **"Magnitude" is the absolute value:** -2,000 earns the bonus.
- **A candidate whose score is not within 0 to 1 makes no event.** `Confidence::new` refuses it; no score of V2 is.
- **Two display windows that only touch count as overlapping** (D-49 says "overlaps"): a number that ends its fade at 2,700 ms and one that would appear at 2,700 ms are not both kept.
- **The length of a token is counted in characters,** as §10.2 says, and a token of more than 8,192 bytes is refused before it is counted, so that the count reads a bounded number of bytes.
- **The order of the checks in `decode`:** the length; the two segments; base64url of the first, then of the second, then its 64 bytes; the JSON; the version.
- **`verify_strict`,** which also refuses a key of small order and a signature that is not in its canonical form. A token of the TypeScript minter passes it.
- **`ParsedToken` has no `Debug`.** §10 says a token is never formatted; with `Debug`, a failed `unwrap` would have printed one.
- **`TokenError`'s messages** name the fault and hold no part of the token.
- **`mintEntitlementToken`:** `sub` is one fixed user id and `freeExportsRemaining` defaults to 3. The signature of §23.4 has no option for `sub`, and G 6.5 asks that the claims be known in full.
- **`seedEntitlement` never makes the database.** Opening a database that does not exist would make it, at version 1 and without its stores, and the app would then find it broken. The helper aborts that and rejects.

**Differs from the prompt, the guide or the plan.**

- **`ed25519-dalek` is not inherited from the workspace.** The crate's `Cargo.toml` states `version = "3.0.0", default-features = false` itself, against the rule at the top of the workspace file that a version is written once. The workspace entry has the default features (`fast`, `zeroize`), which the server signs with; a member cannot turn off the default features of a dependency it inherits; and `server/Cargo.toml` is frozen, so the entry could not be changed and the server given its features back. The two versions must be kept equal by hand. To settle when V6 opens the server: `default-features = false` in the workspace, and the features named in `server/Cargo.toml`.
- **A first segment that is a JSON list is refused as `Json`.** The reader would otherwise take the seven values in order, without their names, for the claims (tried: with the check off, such a payload is read). D-25 says the claims are written with their field names, and no signer writes a list. One line, and one case of the test.
- **`seedEntitlement` reads `DB_NAME`, `DB_VERSION` and `STORES` from `src/persistence/schema.ts`,** and does not write the three values again. The file is frozen; it is read, not edited.
- **The seed is written as 64 hex digits,** not as a list of bytes: the check of G 6.5 searches for its first 16 hex digits.

**Tests (inline): the 9 rows of §9.4.**

| Row of §9.4 | Test |
|---|---|
| "$10,000" at confidence 0.9 | `tests::ten_thousand_dollars_at_confidence_0_9_is_one_number_reveal` (also the split form `$12` `,000` of the reference clip; an edited and a hidden word; an edit elsewhere) |
| "ten thousand" (no unit) | `tests::ten_thousand_in_words_has_no_unit_and_is_no_event` (also `3000`, `three`, and no words at all) |
| "40%" at confidence 1.0; at 0.99 | `tests::forty_percent_is_an_event_at_confidence_1_and_none_at_0_99` (also: the least certain word of a number counts) |
| Two `$` amounts 600 ms apart | `tests::of_two_amounts_600_ms_apart_only_the_earlier_one_is_kept` (also the line itself: 2,850 ms and 2,851 ms) |
| Two `$` amounts 5 s apart | `tests::two_amounts_5_s_apart_are_both_kept_in_time_order` |
| Any returned event | `tests::no_returned_event_is_below_the_threshold_of_its_kind`: ten numbers of every score and four confidences (also the energy bonus) |
| Same inputs twice | `tests::the_same_inputs_twice_give_the_same_events_and_the_same_ids` |
| `event_id(NumberReveal, 5..6)` | `event_id::tests::the_id_of_a_number_at_words_5_to_6_is_the_fnv_1a_64_of_its_nine_bytes`: `0x8b13f71d9a4f303c`, worked out by another program; the hash against its three published values |
| Same anchors, different kind | `event_id::tests::the_same_anchors_with_another_kind_have_another_id` |

**Tests (inline): the 9 rows of §10.5.**

| Row of §10.5 | Test |
|---|---|
| The cross-language vector | `verify::tests::a_token_minted_in_typescript_verifies_with_the_test_public_key` (also the Free token) |
| One payload character changed | `verify::tests::the_token_with_one_payload_character_changed_has_no_good_signature` (also one character of the signature; the signature of another token) |
| A different key; `[wrong, right]` | `verify::tests::another_key_does_not_verify_and_the_right_key_after_a_wrong_one_does` (also no key; bytes that are no key, first and last) |
| `"abc"`, `"a.b.c"`, `".sig"`, padded base64 | `token::tests::what_is_not_two_segments_of_unpadded_base64url_is_refused` (also the other alphabet, a signature of 63 and of 65 bytes, 2,048 and 2,049 characters, a million characters) |
| An extra field; `v: 2` | `token::tests::a_payload_with_an_extra_field_is_not_the_claims_and_v_2_is_another_version` (also a missing field, wrong types, an unknown plan, a list; and the token written again from its claims) |
| `export_profile(None, _)` | `profile::tests::without_claims_the_profile_is_the_preview` |
| Creator claims, `now` before both ends | `profile::tests::creator_claims_before_both_ends_give_1080x1920_without_a_watermark` (the last second of each end still counts) |
| Creator claims, `now > exp`; `now > period_end` | `profile::tests::creator_claims_past_either_end_give_the_free_profile` |
| Free claims | `profile::tests::free_claims_give_the_free_profile` |

**The drill.** One character changed in the first segment of the pasted token (`MDAw` to `MDAx` in the time of issue, so that it is still base64url and still the claims): the vector test fails with `left: Err(Signature)`. Restored from a copy.

**The secret scan, before the commit** (known issue 8): gitleaks 8.18.4, the Windows archive with its SHA-256 equal to the published one, run with `--no-git` on the files of this commit and of the commit of Prompt 45. No finding: not the seed, not the two tokens, not the public key in this entry. No `gitleaks:allow` comment was needed.

**Checked.**

- `cargo test -p offcut-detect -p offcut-entitlement`: 9 and 9 passed. `cargo clippy` on both, with all targets: clean.
- `cargo tree -p offcut-entitlement -e normal | grep -ci "rand\|getrandom"`: 0. The crate has neither of `ed25519-dalek`'s default features.
- `git grep -n "7f347b4b1b270181" -- web/src crates`: no output. The seed is in `web/tests-e2e/helpers/fake-api.ts` and nowhere else.
- `cargo check -p offcut-detect -p offcut-entitlement --target wasm32-unknown-unknown` passes. Not asked for; Prompt 48 builds on it.
- `node scripts/check-file-tree.mjs`: 195 files; both crates are checked as pure crates. `cargo deny check`: ok. `sh scripts/check-gen-clean.sh`: no diff; `EntitlementClaims` is not generated.
- G M-6: the three crates pass 11, 9 and 9 rows; clippy is clean on the three; `pnpm check` is green; 3 font files are tracked; no `zz-` file is tracked under `web/tests-e2e`; the generated types are unchanged.
- Lines above the test module: `number.rs` 102, `token.rs` 79, `lib.rs` of the detector 62, `profile.rs` 58.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test` and `pnpm build` green; the frozen-file diff against the baseline is empty. **247 Rust, 92 Vitest, 12 Playwright** (Rust was 229: nine in each crate).
- **`pnpm e2e`, as the gate runs it, passed 11 of 12 in its first run and 12 of 12 in the four runs after it** (three times with Playwright's 6 workers, once with 3). The last of the four is the run of the whole gate, repeated after the last change to a source file of this prompt. The case that failed is "waitlist: a slow API shows the waking message, then success", which runs in real time on purpose: the message it waits for is on the page between the third and the fourth second, and the run, straight after a build, saw "Sending" and then the success text. Nothing was changed between the runs. The app that was served is the build of Prompt 45, file for file (the same content-hashed names): neither crate is in a bundle before Prompt 48, and the case uses none of the new helpers. This is the machine again (known issue 20); the `ci` run of the push is the check.

**"Done when".**

- [x] `cargo test -p offcut-detect -p offcut-entitlement` passes 18 rows; `cargo tree -p offcut-entitlement -e normal | grep -ci "rand\|getrandom"` is 0.
- [x] No returned event is below `threshold_number` (INV-6); the same inputs give the same ids.
- [x] The seed occurs in `fake-api.ts` only (`git grep` over `web/src` and `crates` is empty); `sh scripts/check-gen-clean.sh` shows no diff; G M-6 passes.

**Not checked.**

- **`ci`.** Nothing was pushed: the commits of Prompts 45 and 46 are local. The font files, `parley` on Linux, and the secret scan of the whole history run there for the first time.
- **A token of the server's signer.** V6 writes it; until then the only signer is the TypeScript helper.
- **The detector on the transcript of the reference clip.** The split form of its amount is a case of the first test; the 157 words themselves reach `detect` in Prompt 48, through the binding.

**Found, and not for this prompt.**

- **An edited number is not shown at all in V2.** A user who corrects `$10,000` to `$20,000` loses the reveal (§9.3: "a span with an edited or hidden word is skipped in V2; V3 re-parses it"). V2 has no way to edit a word, so nobody can meet it before V3.
- **The scene and the detector disagree by 150 ms on when a number is on the screen.** The detector keeps two numbers apart by windows that start 150 ms before the first word (§9.2); the scene shows a number from the first word (§11.7, "Enter: at `span.start`"). The detector's window is the wider one, so nothing overlaps.

**Open, for the human.** Push `v2-build` (two commits: Prompt 45 and Prompt 46) and read `ci` on pull request #2. Prompt 47 waits for green.

## 2026-10-09 - The eighth push: `ci` green on Prompts 45 and 46

**Done by the human.** `git push` of `v2-build` at `9a00b52`, the commit of Prompt 46; the commit of Prompt 45 (`89fea78`) went with it.

**Read by the agent** (the public API of GitHub).

| Read | Result |
|---|---|
| `origin/v2-build` | `9a00b52`, equal to the local branch |
| The `ci` run on `9a00b52` (pull request #2, run 15) | Success, in 204 s: 35 steps passed, 2 skipped, none failed. The two deploy jobs were skipped, as on every run of the branch |
| Steps that ran on new code | The secret scan (the test seed, the two test tokens, the font files), `cargo clippy`, `cargo deny` (with `parley` and what it brings), `cargo test` (247), the file tree with the three new pure crates, Playwright: all success |
| Pull request #2 | Open, draft, no conflict with `main` |
| Production | Unchanged: `/api/v1/healthz` reports `322c7d3`; `main` is at `322c7d3` |

**Closes.** The push that Prompt 46 ends with, and what its entry and the entry of Prompt 45 left for Linux: `parley` builds there without its `system` feature and so without fontconfig; the font files and `LICENSES.md` pass the file tree and the secret scan; the history with the seed and the tokens holds no finding; Playwright passes with its default workers, the real-time waitlist case included. The run took 204 s, where run 14 took 179 s: the new crates compile in it.

**Changed.** This file only. The entry was written with the next commit.

## 2026-10-09 - Prompt 47: `offcut-render`

Prompts 47 and 48 were asked for in one sitting; each has its own gate and its own commit. The entry "The eighth push" above is committed with this one.

**The flag: not needed.** No `--cfg=web_sys_unstable_apis`, no `.cargo/config.toml`, and `scripts/build-wasm.sh` is as it was. The test of G 7.1: in the locked `web-sys` (0.3.106) the types `VideoFrame` and `OffscreenCanvas` carry no gate (the count is 0 for both), and `cargo check -p offcut-render --target wasm32-unknown-unknown` passes with `video_pass.rs` written and importing a frame. In `wgpu` 30 the variant that takes a `VideoFrame` is not gated either.

**Added.**

| File | Content |
|---|---|
| `crates/offcut-render/src/gpu.rs` | `Gpu`: one instance, one adapter, one device and queue, the surface of the canvas and its configuration; the device-lost flag; `resize`, `set_canvas`, `target` |
| `crates/offcut-render/src/shaders/video.wgsl` | One vertex shader (a triangle over the whole target) and two fragment shaders: `video`, the frame through its placement, and `over`, the overlay |
| `crates/offcut-render/src/video_pass.rs` | `VideoPass`: the frame's texture, the copy of a `VideoFrame` into it, `placement(crop, rotation, frame)`, the draw; the helpers both pipelines share |
| `crates/offcut-render/src/vello_backend.rs` | `Overlay`: a display list to a `vello::Scene`, command by command, rendered into the overlay texture |
| `crates/offcut-render/src/composite.rs` | `Compositor`: one render pass to the canvas, the frame and then the overlay, and the present |

**Changed.**

| File | Change |
|---|---|
| `crates/offcut-render/src/lib.rs` | The four modules; `pub use offcut_scene as scene;` (D-61); `RenderError` with six variants; `Renderer` with `new`, `resize`, `render` of TS §19.2, and `set_canvas` |
| `crates/offcut-render/Cargo.toml` | `offcut-types`, `offcut-scene`, `thiserror`, `vello` (feature `wgpu`), `wgpu` (features `std`, `webgpu`, `wgsl`), `web-sys` (features `OffscreenCanvas`, `VideoFrame`) |
| `Cargo.toml` | `[workspace.dependencies]`: `offcut-scene` (path), `vello`, `wgpu`, both with `default-features = false` |
| `Cargo.lock` | 46 packages: `vello`, `wgpu`, the shader compiler and what they bring |
| `deny.toml` | Four wrappers, all third-party crates (below) |
| `crates/offcut-scene/src/layout.rs` | `normalized_coords(FontId)`, and one test for it (see "Differs") |
| `docs/v2/v2implementation.md` | §12 says what was built (1 replacement, applied once) |

**Pinned.** `vello` 0.11.0 and `wgpu` 30.0.1, the pair written down in Prompt 45; `parley` stays at 0.11.1. `vello` asks for `wgpu ^30.0.0`, and 30.0.1 is the one `wgpu` in the lock: `cargo tree -d -p offcut-render` names no `wgpu`, no `peniko` and no `skrifa` twice. `wgpu` asks for `wasm-bindgen` 0.2.127 or later and `web-sys` 0.3.104 or later: the pins of the workspace (0.2.129, 0.3.106) did not move, and the CLI still matches. No license was added to `deny.toml`.

**`wgpu` has the browser's WebGPU as its only backend.** Without its default features no Vulkan, Metal, DirectX or OpenGL backend is built. The crate still compiles natively, for `cargo clippy --workspace` and `cargo test --workspace`; a renderer made there answers `NoAdapter`, before `wgpu` is asked for an instance, which it would refuse with a panic when no backend is built.

**The wrappers added to `deny.toml`,** each named by `cargo deny check`:

| Banned crate | New wrappers | Why |
|---|---|---|
| `wgpu` | `vello` | Vello renders through it |
| `web-sys`, `js-sys` | `wgpu`, `wgpu-types` | They hold browser objects (a canvas, a video frame) when built for wasm |
| `wasm-bindgen` | `wgpu` | The same |

Only `offcut-render` depends on `vello` or `wgpu`.

**Decided here, where the plan is silent.**

- **Where a point of the target lies in the frame** is six numbers, worked out on the CPU by `placement(crop, rotation, frame)` and given to the shader. `crop` is in display pixels, as TS §19.7 says; the display size is the stored frame's, with its sides exchanged for `R90` and `R270`. `R90` is a frame shown turned a quarter clockwise, which is what the probe reads from the matrix `0 1 -1 0`.
- **The size of a frame** is its `displayWidth` and `displayHeight`, which is what the browser's external copy copies. A frame with a size of 0 (a closed one) is `FrameImport`.
- **No colour is converted.** The frame's texture and the overlay's are `Rgba8Unorm` with sRGB-encoded values, and the surface is given a format that does not convert on writing.
- **The adapter is asked for with no power preference:** the browser picks it, as it does for the rest of the page. A request for the fast GPU of a machine with two could put the renderer on another GPU than the one that decodes the video.
- **The device is asked for with the default limits and no feature,** as Vello's own helper does without its optional ones.
- **Vello is set up for one antialiasing method,** area, which is the one a frame is rendered with.
- **A `PushLayer` without a clip** is clipped to the whole target. A pop without a push is ignored, and a layer left open is closed at the end of the list, so that no list can reach into the next frame.
- **A canvas of 0 by 0** is `Surface`.
- **A second canvas whose format is another one** than the first is `Surface`: the pipelines are built for one format.
- **Which surface texture states are which error:** lost is `DeviceLost`; a timeout, an occluded or outdated surface and a validation fault are `Surface`.

**Differs from the prompt, the guide or the plan.**

- **`offcut_scene::layout::normalized_coords(FontId)` is new, in a file of Prompt 45.** Vello draws a glyph of a variable font at given axis coordinates, and has to be told them: a glyph id alone is the outline at the font's default weight, 400, and not the 700 it was measured at. Vello does not hand on the crate that works coordinates out of a weight, and the prompt names no such dependency. The scene crate already has them: they are what parley shaped with. It returns them per font: `[0, 8848]` for `Inter700`, `[0, 16384]` for `Inter900`, `[9585]` for `JetBrainsMono700`, `[16384]` for `NotoEmoji`. One test, the twelfth of that crate.
- **`Renderer::set_canvas`,** which TS §19.2 does not have. §13.5 has a session attach a canvas more than once ("a new surface on the same device, then `resize`"), and a surface is made from a canvas.
- **The frame's texture is not made in `new`** (§12: "textures are created in `new` and `resize` only"). `new` is given the output size, not the source's. It is made for the first frame and kept: a clip has one frame size.
- **A frame with an empty display list draws no overlay,** and Vello is not run for it.
- **Two inline tests,** where §12 says no V2 test runs this crate natively. Neither needs a GPU. One is the placement for the four rotations and two crops. The other reads `video.wgsl` with the shader compiler the browser build uses: the file is valid, and each entry point uses the bindings its pipeline gives it and no other. Without it the first fault of the shader would show in Prompt 50.
- **Both fragment shaders are in `shaders/video.wgsl`.** The tree of TS §5 has one shader file.

**A trap, for whoever touches `video_pass.rs`.** `frame.clone()` on a `web_sys::VideoFrame` is the browser's `VideoFrame.clone()`: a new frame, which somebody must close. `wgpu` takes the frame by value, so the file passes `Clone::clone(frame)`, Rust's clone of the handle: a second reference to the same object, which closes nothing when dropped. The first draft had the other one; the compiler showed it, because the browser's returns a `Result`.

**Checked.**

- `cargo check -p offcut-render --target wasm32-unknown-unknown` passes; so does `cargo clippy` for that target, which lints the code that exists in the browser only.
- `cargo clippy -p offcut-render --all-targets -- -D warnings`, natively: clean. `cargo test -p offcut-render`: 2 passed.
- `cargo deny check`: advisories, bans, licenses, sources ok. `node scripts/check-file-tree.mjs`: 200 files; the edge `offcut-render` to `offcut-scene` is within the graph.
- Lines above the test module: `video_pass.rs` 317, `vello_backend.rs` 235, `gpu.rs` 166, `lib.rs` 126, `composite.rs` 121.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test` and `pnpm build` green; the frozen-file diff against the baseline is empty. **250 Rust, 92 Vitest** (Rust was 247: two in `offcut-render`, one in `offcut-scene`).
- **`pnpm e2e`, as the gate runs it, passed 11 of 12 in its first two runs and 12 of 12 in the third; with `--workers=3` it passed 12 of 12 in between.** The case that failed both times is "the settings link leads to the table of what leaves the device", the longest one: it ran out of its 30 s while the six cases that start first took 24 to 29 s each, where they take 3 to 6 s on a free machine. This is known issue 20, on a machine that had just built `wgpu` and Vello natively and was running a disk scan beside the suite. The app that was served is the build of Prompt 46, file for file (the same content-hashed names): the renderer is in no bundle before Prompt 48, and no file of the suite changed. Nothing was changed between the runs. **12 Playwright**; the `ci` run of the next push is the check on a clean machine.

**"Done when".**

- [x] `cargo check -p offcut-render --target wasm32-unknown-unknown` and native `cargo clippy -p offcut-render --all-targets -- -D warnings` both pass.
- [x] The flag outcome and the three pinned versions are in this entry: no flag; `vello` 0.11.0, `wgpu` 30.0.1, `parley` 0.11.1.

**Not checked.**

- **No pixel was drawn in this prompt.** The crate compiles for the browser and its shader is valid; whether an adapter, a device, the copy of a frame, Vello and the pass work together shows when a browser runs it.
- **Linux.** `wgpu` without a backend compiled natively on Windows; the `ci` run builds it on Linux for the first time, and will take longer: the native test build of this crate alone took 78 s here.
- The render time on R1 (TE-3, Prompt 51).

**Found, and not for this prompt.**

- **Stroked after it is filled, a caption's letters get thinner.** The display list says fill, then stroke, and the renderer draws in that order (TS §19.2, §12). A stroke lies half inside the outline: at 64 lp with a stroke of 6 lp, 3 lp of dark cover each edge of the white letter. Captions on video are usually stroked first and filled over it. Nobody has seen a frame yet; this is for the look at Prompt 55 and for E-2, and it is one swap in `vello_backend.rs` if the order of the list is changed.

## 2026-10-09 - Prompt 48: `offcut-wasm-render`, bundle, loader, preload

**REOPENED V1 CONTRACT: `scripts/check-hosts.mjs`** (D-38). Six entries, each one whole literal, each allowed in the render bundle only. Nothing was loosened for a file that had passed before.

**The first frames.** The prompt asks for no drawing, and nothing had drawn a pixel since Prompt 45. A temporary worker drove the new bundle in Chrome and read the canvas back: the crop is exact to the pixel, a frame marked `R90` is shown turned a quarter clockwise, the captions and the `$12k` reveal are drawn in their zones and nowhere else, and a second canvas at 1080 x 1920 works on the same device. The table is below.

**Added.**

| File | Content |
|---|---|
| `crates/offcut-wasm-render/src/detect_api.rs` | The export `detect(transcript, prosody, edits)`, with `DetectorConfig::default()` |
| `crates/offcut-wasm-render/src/profile_api.rs` | `export_profile_from_token(token, public_keys, now_unix_secs)`, `preview_profile()` |
| `crates/offcut-wasm-render/src/mux_api.rs` | `Mp4MuxerHandle` with `new`, `add_video_sample`, `add_audio_sample`, `finalize`; the private `JsMuxSink` |
| `crates/offcut-wasm-render/src/session.rs` | `RenderSession` with the methods of TS §19.2 and the four of §13.5; the export `open_session`; the private `JsRandomAccess` with its window of 1 MiB (known issue 15) |
| `web/src/wasm/load-render.ts` | `RenderApi`, `RenderSession`, `Mp4MuxerHandle`, `SceneInput`, `VideoSample`; `preloadRender()`, `loadRender()` (§15.5) |

**Changed.**

| File | Change |
|---|---|
| `crates/offcut-wasm-render/src/lib.rs` | The four modules; the panic hook and `init` of V1; `failure`, `to_plain`, `to_js`, `from_js`, `word_edits` |
| `crates/offcut-wasm-render/Cargo.toml` | `offcut-types`, `offcut-render`, `offcut-detect`, `offcut-mp4`, `offcut-entitlement`, `wasm-bindgen`, `wasm-bindgen-futures`, `js-sys`, `web-sys` (four features), `serde`, `serde-wasm-bindgen`; dev `serde_json`. Not `offcut-scene` (D-61) |
| `Cargo.toml` | `[workspace.dependencies]`: `offcut-detect`, `offcut-render`, `offcut-entitlement` (paths); `wasm-bindgen-futures = "0.4.79"` |
| `Cargo.lock` | The edges of the crate. No package is new |
| `scripts/build-wasm.sh` | `BUNDLES` is `offcut-wasm-core:core offcut-wasm-render:render`. The `RUSTFLAGS` line is unchanged |
| `scripts/check-hosts.mjs` | `RENDER_MODULE`, `VELLO_SHADER_COMMENT`, six entries |
| `web/src/workers/pool.ts` | `preload()` fetches and compiles both bundles, side by side |
| `docs/v2/v2implementation.md` | §13.4, §13.5 and §15.5 say what was built (3 replacements, each applied once) |

`deny.toml` was not touched: the crate was a wrapper already, and `wasm-bindgen-futures` too.

**Pinned.** `wasm-bindgen-futures` 0.4.79, the release that goes with `wasm-bindgen` 0.2.129 and the one the lock already held for `wgpu`. An `async fn` export needs it (D-54).

**The bundle.** `web/src/wasm/pkg/render/offcut_render_bg.wasm` is **6,029,674 bytes** after `wasm-opt`, 2,861,729 gzipped; 3,165,448 of them are the three fonts. `offcut_core_bg.wasm` is 459,281, as before. The crate in `Cargo.lock` and the CLI are both `wasm-bindgen` 0.2.129. The release build of the bundle took 2 min 19 s on the development machine. The app shell is 266.6 kB gzipped, was 266.0: the glue of the second bundle.

**Every visitor now downloads the render bundle at app start,** 2.9 MB compressed: `preload()` fetches both (D-9, §15.7). It was 177 kB for the core bundle alone.

**The six entries of `check-hosts.mjs`.** `pnpm build` failed with 17 findings, six literals, all in `offcut_render_bg-*.wasm`. Each was found in the source it comes from before it was listed: five in `fine.wgsl` and one in `draw_leaf.wgsl` of `vello_shaders` 0.11.0. They are comments in Vello's shader sources, which the module holds as text for the GPU's shader compiler.

| Literal | In |
|---|---|
| `https://skia.org/docs/dev/design/conical/` | `draw_leaf.wgsl` |
| `https://raphlinus.github.io/graphics/2020/04/21/blurred-rounded-rects.html` | `fine.wgsl` |
| `https://github.com/gfx-rs/naga/issues/1930` | `fine.wgsl` |
| `https://github.com/linebender/vello/issues/1061` | `fine.wgsl` |
| `https://github.com/google/skia/blob/30bb...6b/src/opts/SkRasterPipeline_opts.h#L5859` | `fine.wgsl` |
| `https://en.wikipedia.org/wiki/Carry-less_product` | `fine.wgsl` |

**No entry for the fonts, and why.** D-38 expected addresses from the fonts' name tables. They are in the module (`https://rsms.me/`, `https://github.com/JetBrains/JetBrainsMono`, `http://www.google.com/get/noto/`), but a name table holds its text with two bytes to a character, and `check-hosts` reads a file byte by byte: it does not see them. An entry that matches nothing would be noise. They are text of a font file; nothing requests them.

**The three drills.** Each was undone from a copy.

| Temporary edit | What fired |
|---|---|
| `offcut-scene` as a dependency of `offcut-wasm-render` | `check-file-tree`: "offcut-wasm-render may not depend on offcut-scene (TS §7)". `cargo deny check bans`: "crate 'offcut-scene = 0.1.0' is explicitly banned" |
| `offcut-dsp` as a dependency | `check-file-tree`: "offcut-wasm-render may not depend on offcut-dsp (TS §7)"; `cargo deny` bans it too |
| The `chunk` of the last new entry pointed at `offcut_core_bg-` | `check-hosts`, 1 problem: `web/dist/assets/offcut_render_bg-Wuppzrxx.wasm: https://en.wikipedia.org/wiki/Carry-less_product` |

**The review of `profile_api.rs`: checked.** The token is the first argument of `export_profile_from_token` and is used once, as the first argument of `verify_token`. It is in no `format!`, no error and no returned value: the `detail` of a failure is one of five fixed names, and the function returns the profile, which holds nothing of the token.

**Browser check (dev).** `vite` on port 5173; a temporary Playwright case.

| Asked | Result |
|---|---|
| Both `offcut_core_bg` and `offcut_render_bg` are fetched once at app start | One `fetch` of each. (The dev server also serves one small module per bundle, the `?url` import that names the file; in a build that is a string) |
| `crossOriginIsolated` | `true` |

Not asked: the same on the production build under `vite preview`, with the headers of `vercel.json`. Both files answer 200 as `application/wasm`, once each; `crossOriginIsolated` is `true`; no console line.

**The drive of the bundle, not asked for** (dev server; a temporary module worker, `zz-render.worker.ts`; Chrome under Playwright, headless, adapter `intel / gen-12lp`). The reference clip was written to OPFS and opened on a sync handle. The frame drawn was a test picture of the clip's size, made in the worker: magenta left of the centre strip, cyan right of it, the strip red above and blue below, a green square in the strip's top-left corner. The canvas was read back through a 2D canvas.

| Asked | Result |
|---|---|
| `detect` on six words with `$12` `,000` | One event: `number_reveal`, words 2 to 4, `$12k`, id `"2892511656290198329"` (a string), confidence 1 |
| `detect` with an edit of another word; of a word of the number | 1 event; 0 |
| `detect` with a key that is no word number; with a transcript of another shape | `{ code: "E_INTERNAL", detail: "Edits" }`; `detail: "Transcript"` |
| `previewProfile()` | `preview`, 540 x 960, no watermark, bitrates 0 |
| `exportProfileFromToken` with the Creator token of the tests; the same token 116 days later; the Free token | `creator` 1080 x 1920 at 8,000,000 and 160,000; `free` 720 x 1280 with the watermark; `free` |
| Keys `[3 bytes, a string, the key]` | `creator`: the first two are passed over |
| Another key; no key; one character of the token changed | `E_ENTITLEMENT_INVALID`, `detail: "Signature"`, three times |
| `"abc"` as the token; `NaN` as the time | `E_ENTITLEMENT_INVALID`, `Format`; `E_INTERNAL`, `Now` |
| A muxer with two video samples and one audio sample | `finalize()` returns 262,344, which is where the last write ended; the file starts with `ftyp`; 9 writes reached the sink |
| A frame out of its turn; a sink without `writeAt`; a sink that throws | `E_MUX` `OutOfOrder`; `E_INTERNAL` `Sink`; `E_MUX` `Write` |
| `openSession` on the reference clip | 2,246 video samples; `avcC` of 41 bytes; sample 0 is a keyframe of 91,736 bytes at 0; the keyframe at or before 30 s is sample 896, as `ffprobe` said in Prompt 35; a sample past the end is `E_DECODE_VIDEO` |
| Before a scene | `frame_count()` 0, `summary()` `null`; `render_frame` is `E_INTERNAL` `NoScene` |
| After `set_scene`, before a canvas | 2,242 frames; `visual_moments` 1; `render_frame` is `E_INTERNAL` `NoCanvas` |
| `attach_canvas(canvas, 540, 960)`, the first | 63 ms: the adapter, the device, Vello's shaders |

| Frame | Read back |
|---|---|
| At 2,600 ms, 540 x 960 | Red above, blue below; the pixel in the top-left corner is the green square; **no magenta and no cyan pixel anywhere.** Event zone: 5,231 white pixels and 3,561 of the stroke's colour, the `$12k`. Caption zone: 260 white, 4,083 dark and 68 yellow pixels, the two lines with "sales." active. No white or yellow pixel outside the two zones |
| At 1,700 ms | The count-up: 8,395 white pixels in the event zone, a wider text; the caption's active word is the number |
| At 3,700 ms and at 9,000 ms | The frame alone: no pixel of the overlay. The path without an overlay pass draws the video |
| With word 5 hidden and the event switched off by its id | `visual_moments` 0; nothing in the event zone; a caption with no active word |
| A second canvas, 1080 x 1920, with the Creator profile | The same picture at twice the size: 21,733 white pixels in the event zone. 30 frames were handed to the GPU in 55 ms, which says the calls are cheap and nothing about the render time |
| The frame after all of it | Still open: its `displayWidth` reads 1280. Nothing closed it |
| A clip that says `R90`, the same stored frame | Magenta on top, cyan below, each 328 rows of 540 pixels: the stored frame's left and right bands, 34.2% of its width each. The stored frame's bottom is the display's left |

No console line and no page error in any of it.

**Decided here, where the plan is silent or cannot be built as written.**

- **`open_session` is the export, and `RenderSession::open` is not.** TS §19.2 gives `open(clip: ClipInfo, source: JsRandomAccess)`, and JavaScript can pass neither type. `open` keeps that signature; the export takes a `JsValue` and the sync handle, reads them and calls it.
- **`attach_canvas` is an `async fn`,** with the size as two `u32`. TS §19.2 writes `-> js_sys::Promise` with `&mut self`; a renderer is made by awaiting, and the session must be written to afterwards. JavaScript gets the same promise. **While it is pending the session cannot be used:** another call on it throws. The render worker awaits it.
- **`render_frame` takes the time as `u32` milliseconds,** and the width and height of every call are plain numbers: the unit types are Rust's.
- **Before a scene is set** `frame_count()` is 0 and `summary()` is `null`.
- **A JavaScript object's keys are strings.** `{ 3: "text" }` reaches Rust as the key `"3"`, and the converter does not read a number out of a string. The binding reads word edits with text keys and turns each into a word number: digits only, as JavaScript writes a number. Any other key is `E_INTERNAL`. This holds for `detect` and for the `edit` of a scene input, and it was needed for neither in V2, where no word is edited; it was tried with one.
- **An argument of another shape** is `E_INTERNAL` with the argument's name as `detail`: `Transcript`, `Prosody`, `Edits`, `ClipInfo`, `SceneInput`, `Sink`, `Now`. The caller is the app's own worker.
- **A time that is not a finite number is refused.** Read as a number it would be 0, the year 1970, when no token has run out yet.
- **A file that does not open in `open`** is `E_STORAGE_IO` when it cannot be read and `E_INTERNAL` otherwise: the clip was accepted before a session is opened, so it is never a rejection.
- **`read_video_sample` fails with `E_DECODE_VIDEO`,** as `read_audio_sample` does with `E_DECODE_AUDIO`.
- **`RenderSession` in TypeScript is the module's own class** with the types of `set_scene`, `summary` and `read_video_sample` written out; the names are the module's, because §16.7 reads a session through `video_description`, `read_video_sample` and the other two.
- **`preload()` starts both downloads at once** and answers when both are compiled.

**Differs from the prompt, the guide or the plan.**

- **Six entries, none for a font** (above). The prompt names two kinds of literal, "font name table, dependency error text"; what the build named is a third, shader comments.
- **`Renderer::set_canvas` of Prompt 47 is what a second `attach_canvas` calls,** where §13.5 says "a new surface on the same device, then `resize`". It is that, in one call.
- **The drive of the bundle** and the check on the production build, neither asked for.
- **Five inline tests,** none asked for: the crash message; the word-edit keys; the names of the three kinds of failure this crate maps (token, muxer, renderer). All run natively.

**Checked.**

- G M-7: `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo test --workspace` 0 failed; `pnpm build:wasm` writes `pkg/core` and `pkg/render`; `pnpm check` green; `pnpm build` green, `check-hosts` passes with the render bundle in `web/dist` (10 files); `ls web/dist/assets | grep -c "offcut_render_bg-.*\.wasm$"` is 1.
- `cargo clippy -p offcut-wasm-render --target wasm32-unknown-unknown -- -D warnings`: clean. `pnpm --filter web exec tsc --noEmit` and ESLint on `src/wasm` and `src/workers`: clean.
- `node scripts/check-file-tree.mjs`: 205 files. `cargo deny check`: ok. `sh scripts/check-gen-clean.sh`: no diff.
- Lines above the test module: `session.rs` 341, `mux_api.rs` 122, `lib.rs` 116, `profile_api.rs` 70, `detect_api.rs` 23; `load-render.ts` 118, `pool.ts` 90, `check-hosts.mjs` 179.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green, the last one with Playwright's 6 workers at the first run; the frozen-file diff against the baseline is empty. **255 Rust, 92 Vitest, 12 Playwright** (Rust was 250; the five are new).
- The two temporary files and the dev server are gone: `git grep -n "zz-render" -- web crates scripts` prints nothing.

**"Done when".**

- [x] `pnpm build:wasm` writes both bundles; the `wasm-bindgen` crate and CLI versions are equal (0.2.129); the render bundle's size is in this entry.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass; `ls web/dist/assets | grep -c "offcut_render_bg-.*\.wasm$"` is 1; G M-7 passes.

**Not checked.**

- **A real video frame.** The frame of the drive was made from a canvas. A frame out of a `VideoDecoder` reaches the renderer in Prompt 49 and Prompt 50.
- **The renderer under the production CSP.** The drive ran on the dev server; only the preload ran on the production build. Prompt 50 runs an export there.
- **How long a frame takes to render** (TE-3, Prompt 51), and the scene build on R1.
- **R1 and Linux.** The adapter was the integrated Intel GPU of the development machine. `ci` builds the bundle on Linux for the first time; it draws nothing.

**Found, and not for this prompt.**

- **A caption is mostly stroke.** At 540 x 960 the caption of the drive has 260 white pixels and 4,083 of the stroke's colour: the stroke, drawn after the fill, covers most of each letter (entry of Prompt 47). At 1080 x 1920 it is 1,563 to 18,391. The large number is less affected: 5,231 to 3,561. This is what the display list asks for; it wants eyes before Prompt 55, and probably the stroke first and the fill over it.
- **6 MB at app start.** The render bundle is fetched by every visitor of the landing page, whether a clip follows or not. D-9 decided that for a bundle of 177 kB. Half of the bundle is the three fonts, and Noto Emoji alone is 2.0 MB for a fallback no transcript has needed. Not a V2 exit criterion; a question for before the page is announced.

**Open, for the human.** Push `v2-build` (two commits: Prompt 47 and Prompt 48) and read `ci` on pull request #2. Prompt 49 waits for green. The run will be longer than the last: `wgpu` and Vello are built natively for the tests and for wasm32 for the bundle.

## 2026-10-09 - The ninth push: `ci` green on Prompts 47 and 48

**Done by the human.** `git push` of `v2-build` at `69fec96`, the commit of Prompt 48; the commit of Prompt 47 (`1a411b1`) went with it.

**Read by the agent** (the public API of GitHub).

| Read | Result |
|---|---|
| `origin/v2-build` | `69fec96`, equal to the local branch |
| The `ci` run on `69fec96` (pull request #2, run 16) | Success, in 304 s: 35 steps passed, 2 skipped, none failed. The two deploy jobs were skipped, as on every run of the branch |
| The steps that grew | `cargo test` 54 s; "Build the WASM bundle" 87 s, now two bundles; `cargo clippy` 24 s. The whole run took 204 s on the push before |
| Steps that ran on new code | The secret scan, `cargo clippy` and `cargo test` (255) with `wgpu` and Vello built natively, `cargo deny` with the four new wrappers, the file tree, the hosts check with the six new entries on the render bundle, Playwright: all success |
| Pull request #2 | Open, draft, no conflict with `main` |
| Production | Unchanged: `/api/v1/healthz` reports `322c7d3`; `main` is at `322c7d3` |

**Closes.** The push that Prompt 48 ends with, and what the entries of Prompts 47 and 48 left for Linux: `wgpu` without a graphics backend builds and tests natively there; the render bundle builds for wasm32 with the CLI of the workflow; `check-hosts` passes on the bundle built there, so the six literals are the same six; Playwright passes with its default workers, with the page now compiling both bundles at its start.

**Not shown by this run.** `ci` draws nothing: no case opens a session. The frames of Prompt 48 were read on the development machine only.

**Changed.** This file only. The entry was written with the next commit.

## 2026-10-09 - Outside a prompt: D-68, a glyph run is stroked first and filled over it

**The decision (founder, 2026-10-09): D-68.** A glyph run is drawn with its stroke first and its fill over the stroke. The agent recommended it after the first frames of Prompt 48, where a caption was mostly stroke; the founder agreed.

**Changed.**

| File | Change |
|---|---|
| `crates/offcut-render/src/vello_backend.rs` | The two draws of a glyph run change places: the stroke, then the fill. Nothing else |
| `crates/offcut-scene/src/display_list.rs` | Two comments: what order a glyph run is drawn in, and that a stroke's width is its whole width, half on each side of the glyph's edge |
| `docs/v2/v2implementation.md` | D-68 added to §2; §0 says "D-18 to D-68"; the row of `vello_backend.rs` in §12; D-68 joins the decisions to copy back into TS |
| `docs/technicalspec.md` | §19.4: one sentence, the order and what shows of a stroke |
| `docs/v2/coding-promptsv2.md` | The Standard Agent Block says "D-18 to D-68"; Prompt 47 names the new order |
| `docs/v2/v2buildguide.md` | The table of references names D-68; the row of `vello_backend.rs` in Step 7.2 |
| `docs/v2/v2changelog.md` | This entry; known issue 27 corrected |

The documents were edited by a script that refuses a replacement unless its old text occurs exactly once, and that writes nothing unless every replacement can be applied: 8 of 8, and the row of D-68. Each file keeps its line endings. The display list itself is unchanged: `DrawCmd::GlyphRun` has the same fields, and no test of `offcut-scene` reads the order.

**Measured, before and after.** The same scene as in the entry of Prompt 48 (six words with `$12` `,000`, at 2,600 ms), drawn by the rebuilt bundle in Chrome through a temporary worker and read back. The picture under the text was another one this time, a grey with a lighter band, so that white and dark both show against it; the counts are of the text's own colours.

| Canvas | Zone | Fill first (Prompt 48) | Stroke first (now) |
|---|---|---|---|
| 540 x 960 | Caption: white, dark, yellow | 260, 4,083, 68 | 2,015, 1,031, 689 |
| 540 x 960 | Reveal: white, dark | 5,231, 3,561 | 7,597, 740 |
| 1080 x 1920 | Caption: white, dark, yellow | 1,563, 18,391, 514 | 8,747, 7,185, 3,099 |
| 1080 x 1920 | Reveal: white, dark | 21,733, 15,512 | 31,190, 5,663 |

No white pixel outside the two zones, as before.

**Looked at.** The worker also handed the frame back as a picture, and the agent read it: white letters with a thin dark outline, "sales." in yellow, `$12k` large above the caption, both centred on the middle of the safe area. It is the first frame of the output anybody has looked at. It is not committed.

**What it costs.** The outline that shows is the outer half of the stroke: 3 lp of the 6 lp that TS §19.4 gives the Clean style. If the outline should be heavier, that is a change of the style's number, in `styles.rs`, and not of the order.

**Checked.**

- `cargo clippy` on `offcut-render` and `offcut-scene`, natively and for wasm32: clean. `cargo test` on both: 2 and 12 passed.
- The bundle: `offcut_render_bg.wasm` is 6,029,652 bytes, was 6,029,674.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green, the last one with Playwright's 6 workers at the first run; the frozen-file diff against the baseline is empty. **255 Rust, 92 Vitest, 12 Playwright**, as after Prompt 48.
- The temporary worker and the temporary Playwright case are deleted.

**Not checked.** The outline on a real video frame, and at the size a phone shows it: Prompt 55 is where a person looks.

## 2026-10-09 - Prompt 49: verifier, encoders, sink, video source

**Added.**

| File | Content |
|---|---|
| `verify/verify_mp4.py` | Checks 1 to 6 of §23.9; the invocation of TS §27.2; one line per check and one that names checks 7 to 11 as not implemented |
| `verify/requirements.txt` | `numpy==2.5.3`, not imported before V5 |
| `verify/README.md` | The invocation, the checks, the tool versions, and that the directory shares nothing with the product |
| `web/src/workers/render/opfs-sink.ts` | `OpfsSink` with `open`, `writeAt`, `close`, `abort` (TS §21.1) |
| `web/src/workers/render/video-source.ts` | `DemuxerHandle` (D-30), `VideoSource` with `frameAt`, `frameAtBlocking`, `prefetch`, `close` (TS §20.1), `liveFrames` (D-45), `PREVIEW_QUEUE` |

**Changed.**

| File | Change |
|---|---|
| `web/src/workers/render/encoders.ts` | Three names added: `ENCODE_QUEUE_MAX = 4`, `AAC_PRIMING_SAMPLES = 0`, `pickVideoConfig(profile)`. Two imports for them: the type `ExportProfile`, and `WorkerFailure`. No V1 constant, and no V1 line, was touched |
| `package.json` | The script `verify`: `python verify/verify_mp4.py` |
| `docs/v2/v2implementation.md` | §16.7, §16.9 (twice) and §23.9 say what was built (4 replacements, each applied once) |

**Pinned.** NumPy 2.5.3, the latest release (`pip index versions`) and the one installed. Tried with Python 3.13.5 and `ffmpeg` and `ffprobe` 9.0.2.

**The two verifier drills.**

| Run | Printed |
|---|---|
| The reference clip, `--profile creator --expected-duration-ms 74705` | `PASS 1`; `FAIL 2: the pixel format is yuvj420p, not yuv420p; the size is 1280x720, expected 1080x1920 for creator`; `FAIL 3: frame 1 comes 239/5000 s after frame 0, not 1/30 s`; `PASS 4`; `FAIL 5: the audio is 74517.521 ms long and the video 74705.033 ms: 187.512 ms apart`; `PASS 6`; the line for 7 to 11; exit 1 |
| A 2-second copy made with `-movflags -faststart`, the first line | `FAIL 1: moov after mdat: moov at byte 927574, mdat at byte 40` |

**The verifier can also pass,** which the prompt does not ask to show and a judge must: `ffmpeg` made a file that is everything the checks ask (1080x1920, 30 frames a second, `libx264` in `yuv420p`, AAC-LC at 48 kHz in stereo, 2 s, faststart). Six times `PASS`, exit 0. The same file fails where it should when the question changes:

| Run | Fails |
|---|---|
| The good file as `--profile free` | 2: the size |
| The good file with `--expected-duration-ms 2100` | 3: 60 frames, expected 63. 5: the video is 100 ms from the expected |
| 29.97 frames a second, one channel at 44,100 Hz | 3 and 4 |
| The good file cut off after 60,000 bytes | 3 and 6 (`ffmpeg` reports 3 error lines). Check 1 passes: its `moov` is whole |
| A file that is not there | All six, with what `ffprobe` said |
| `--profile pro` | Exit 2, the message of the argument parser |

**Browser check (dev), not asked for.** `vite` on port 5173; a temporary module worker, `zz-source.worker.ts`; Chrome under Playwright, headless. The reference clip in OPFS; a session of the render bundle as the `DemuxerHandle`.

| Asked | Result |
|---|---|
| A muxer writing through an `OpfsSink` to `exports/tmp/<id>.mp4` | `finalize()` returns 263,694; the file on disk is 263,694 bytes and starts with `ftyp` |
| `writeAt` after `close`; a second `close` | `OpfsError`, `io`; nothing |
| A second sink on the same path while the first is open | `OpfsError`, `io`: one handle per file |
| `abort`, and again | The file is gone (`size` is `null`); nothing |
| `pickVideoConfig` for the Creator and the Free profile | `avc1.640028`, `prefer-hardware`, the first entry of the ladder, at 1080x1920 and 8,000,000, and at 720x1280 and 4,000,000 |
| For the preview profile; for 16,384 x 16,384 | `E_ENCODE_VIDEO`, detail `NoSupportedConfig`, both |

**The video source on the whole clip.** `frameAtBlocking` for every output time, `n x 1000 / 30` ms for `n` from 0 to 2,241, as the export will ask. The right frame of a time was worked out apart from the source, from the sorted presentation times of the 2,246 samples.

| Read | Result |
|---|---|
| Output times whose frame was not the latest at or before it | **0 of 2,242** |
| A frame older than the one before it | 0 |
| Output times that got the same frame as the one before | 80: the clip has gaps of 48 ms, longer than an output frame |
| The last frame handed out | The clip's last, at 74,671,700 microseconds |
| Frames open at once, at most | 8. One at the end of the loop, the one handed out; **0 after `close()`** |
| Time for the 2,242 | 3.4 s: 652 frames a second, decoding and reading alone, on the development machine |
| A time past the end, 80 s | The last frame again |
| `frameAt` after `close()` | `E_DECODE_VIDEO`, detail `Closed` |

**The two white frames of the clip land where they are in time.** Thirteen frames around each were drawn through the session and a patch read back. Output frame 150 (5,000 ms) is white, 255 where its neighbours are 39 and 48; output frame 1,946 (64,866 ms) is white where its neighbours are 44 and 48. D-64 puts them near 4.99 s and 64.85 s. The second is frame 1,950 of the source: by then four more source frames than output times have passed, which is the variable frame rate.

**As the preview asks: `frameAt` every 8 ms against a running clock.**

| Run | Result |
|---|---|
| From 0, for 2 s | The first frame after 30 ms; 3 of 227 calls answered `null`; every frame after that was the right one |
| A jump to 40 s, for 3 s | The first frame after 42 ms; 6 calls got a frame still catching up, none a frame from after its time |
| A jump back to 5 s, for 2.5 s | The first frame after 48 ms |
| One frame back, then forward again | `null` for the step, and the same frame as before after it: no new start |
| After `close()` | 0 frames open |

Nine frames were open at most: the six ahead, the one handed out, and what the decoder gave before it was asked to stop.

| A source that fails | Result |
|---|---|
| A demuxer that throws `{ code, detail: "Malformed" }` | `frameAt` and `frameAtBlocking` throw `E_DECODE_VIDEO`, `Malformed` |
| Five samples of bytes that are no video | `frameAtBlocking` rejects with `E_DECODE_VIDEO`, `EncodingError`: the decoder's own error. It does not hang |
| Frames open after both | 0 |

**A frame of the real clip was looked at.** Output frame 1,895 was drawn at 540 x 960 with a caption and handed back as a picture: the speaker upright, the centre strip of the 1280x720 frame, colours as in the clip, white caption with a thin dark outline and the active word in yellow. It is not committed.

**Decided here, where the plan is silent or cannot be built as written.**

- **A jump makes a new decoder.** §16.7 says "one `VideoDecoder`"; one is alive at a time. A decoder that is reset may still deliver a frame it had finished, and nothing on a frame says which start it belongs to. With a new decoder each start has its own callbacks, and a frame of an old one is closed unseen.
- **Which frames are closed goes by ownership.** TS §20.2 closes "frames with pts < t - one frame". The frame of a time can be older than that: the clip's gaps are 48 ms. Of the decoded frames at or before the time asked for, the latest is kept and the ones before it are closed; the frame that was handed out is closed when a newer one is handed out, and by `close()`.
- **A jump ahead starts again only when the keyframe of the new time has not been fed.** TS §20.2 starts again for any time more than 1 s past the horizon. After a start at a keyframe several seconds before the time asked for (the clip has a keyframe about every 7.5 s) the horizon is behind the time for a while; starting again then would decode the same frames again, each time.
- **A step back of less than 100 ms is not a jump.** The preview's clock is corrected four times a second and may step back by a frame. `frameAt` answers `null` for it and the loop draws the frame it has.
- **The queue limit counts frames after the time asked for.** Six for the preview, eight for the export. What the decoder holds inside itself is not counted and cannot be: it is fed while it holds fewer than three samples and the queue has room.
- **The decoder is also fed when it takes work off its queue,** not only when a frame is asked for: after a jump it catches up at its own speed, not at 3 samples per call.
- **`frameAtBlocking` for a time before the clip's first frame** hands out the first frame; when the clip has no frame at all it rejects with `NoFrame`.
- **After the last sample the decoder is flushed,** in both modes, so that the frames it still holds come out.
- **Whatever goes wrong in the source is `E_DECODE_VIDEO`,** thrown as a `WorkerFailure`: the decoder's error by its name, the demuxer's by its detail, a call after `close()` as `Closed`. After a failure every later call throws the same.
- **`pickVideoConfig`:** a configuration the browser refuses to read counts as not supported. The preview profile has a bitrate of 0, and the browser throws for it where it would answer "not supported".
- **`OpfsSink.open` empties a file that is already there.** A short write is `quota`: the file system took part of the bytes and no more.
- **The verifier fails check 1 for a stream that is neither video nor audio,** names every fault of a check on its line, and lets the audio be 21.34 ms from the video: one AAC frame, 21.33 ms, rounded up as §23.9 rounds the video frame up to 33.4 ms. Read as 21.3 ms, an export whose audio ends one whole AAC frame after the video would fail. Exit status 2 for a missing tool or a wrong argument.

**Differs from the prompt, the guide or the plan.**

- **`encoders.ts` gained two imports** beside the three names: the type of the profile, and `WorkerFailure`, which is what a worker throws (known issue 13). The file is also read by the capability check on the main thread, where `rpc.ts` already is.
- **`FAIL 2` on the reference clip names two faults,** not the one of the guide: the clip is `yuvj420p`, full-range, and the check asks for `yuv420p`.
- **The browser checks,** and the verifier's own trials beyond the two drills.

**Checked.**

- `git grep -n "import " verify/verify_mp4.py | grep -c "crates\|web"`: 0. Its seven imports are of the standard library.
- `grep -c "8_000_000\|8000000"`: 1 in `encoders.ts` and 2 in `profile.rs`; the Creator bitrate is 8,000,000 in both.
- `pnpm --filter web exec tsc --noEmit` and ESLint on `src/workers` and `src/platform`: clean. `opfs-sink.ts` imports `persistence/opfs.ts`: edge D-27 b.
- `node scripts/check-file-tree.mjs`: 210 files; the three files of `verify/` and the two of `render/` were in the tree of TS §5.
- Lines: `video-source.ts` 362, `verify_mp4.py` 233, `opfs-sink.ts` 88, `encoders.ts` 83.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green, the last one with Playwright's 6 workers at the first run; the frozen-file diff against the baseline is empty. **255 Rust, 92 Vitest, 12 Playwright**, as before: V2 has no test file for these (§23.1).
- The app shell is 266.5 kB gzipped. The temporary worker, the temporary Playwright case and the dev server are gone.

**"Done when".**

- [x] Both verifier drills print the stated failures; `git grep -n "import " verify/verify_mp4.py | grep -c "crates\|web"` is 0.
- [x] `grep -c "8_000_000\|8000000"` finds the Creator bitrate in both `encoders.ts` and `profile.rs`, with equal values; `tsc` and ESLint are clean.

**Not checked.**

- **The verifier on an export of the app.** There is none before Prompt 50. Every file it has judged was made by `ffmpeg` or is the source clip.
- **A cancel in the middle of `frameAtBlocking`,** and the source under the production CSP. Both come with the export loop.
- **A clip turned a quarter, and one with a frame rate above 30.** The reference clip is neither. The decoder is configured with the stored size for a turned clip; nothing has decoded one.
- R1. The decoder of the development machine did 652 frames a second on its own.

**Found, and not for this prompt.**

- **The verifier asks for `yuv420p`, and whether the browser's encoder writes that is not known.** The source is full-range (`yuvj420p`). If an export comes out marked full-range, check 2 fails on a file that plays everywhere. Prompt 50 shows which it is; the check is TS §27.2's and was not softened in advance.
- **The caption lies across the speaker's mouth** in the frame that was looked at: the caption zone is rows 1,220 to 1,500 of 1,920, and the clip is a close webcam shot. The zone is TS §19.1's, an assumption for E-2; a person should look at it at Prompt 55.

## 2026-10-09 - Prompt 50: export loop, render worker, first export

**The first export.** The reference clip went the whole way in a production build, under the production CSP, and came out as an MP4: 1080 x 1920, 2,242 frames, 78.7 MB. The verifier passes it on all six checks, check 5 included. A Free token gives 720 x 1280, and that file passes all six too. Both files are in `testclips/renders/`, which git ignores: `p50-export-creator-1080x1920.mp4` and `p50-export-free-720x1280.mp4`.

**One thing about the picture needs a decision** (open item 11, known issue 30): the export has more contrast than the source file, because Chrome reads this webcam clip as limited-range video though the file says full range. It is not a fault of the loop, and Chrome's own player shows the clip the same way.

**Added.**

| File | Content |
|---|---|
| `web/src/workers/render/export-loop.ts` | `runExport` (TS §21.3 with the fixed points of §16.8): the frame loop, the two encoders, the audio, the muxer. 395 lines |
| `web/src/workers/render.worker.ts` | `openSession`, `detect`, `setScene`, `attachPreview`, `exportClip` (the nine steps of §16.8), `closeSession`; the five methods that wait for a later prompt. 319 lines |

**Changed.**

| File | Change |
|---|---|
| `web/src/workers/pool.ts` | The `render` row (script, stage `preview`) and `pool.render`: the table has three rows, and `preload()` adds a third `modulepreload` link |
| `docs/v2/v2implementation.md` | §16.6 and §16.8 say what was built; §27 gains items 27 and 28, two gaps of TS §21 (5 replacements, each applied once) |
| `docs/v2/v2changelog.md` | This entry; open item 11; known issues 29, 30 and 31 |

No dependency was added. No Rust file changed.

**The drill.** `import { ENTITLEMENT_PUBLIC_KEYS } from "../config/entitlement-public-key";` in `media.worker.ts`: ESLint, `boundaries/dependencies`, "There is no policy allowing dependencies from elements of type "workers" to elements of type "config"". Undone from a copy.

**Browser check (preview, keyed build).** `vite build` with the test public key exported in that one shell; `vite preview` on port 4173 with the headers of `vercel.json`; the hook `zz-spike.ts`; Chrome 155.0.8059.39 under Playwright, headless, a new profile; a temporary case that ran the script of G 8.3 and printed what it read. The tokens were minted by `mintEntitlementToken` and printed nowhere.

| Read | Result |
|---|---|
| The model | `absent`; downloaded from the asset host in 50.2 s |
| `importAndProbe` + `extractAudio` | 1.8 s; `pcm48` has 3,585,840 samples |
| `load` + `transcribe` + `unload`, WebGPU | 63.0 s; 157 words |
| `openSession` + `detect` + `setScene` | 205 ms |
| `events` | 1: `number_reveal`, `$12k`, words 132 and 133, from 62,660 ms to 63,540 ms |
| `exportClip` with one character of the token changed | Rejects with `E_ENTITLEMENT_INVALID` (detail `Json`), stage `render_encode`. After it the directory `exports/` does not exist: nothing was written |
| `exportClip` with the Creator token | `render_encode` 28,282 ms, `mux` 9 ms; `summary` is `{ voice_cleaned: false, captions_emphasized: 0, visual_moments: 1 }`; `exports/<id>.mp4` is there and `exports/tmp/` is empty |
| The same with a Free token | `render_encode` 15,445 ms, `mux` 66 ms; 720 x 1280 |
| Progress of an export | 2,243 messages: `render_encode` with `done` from 1 to 2,242, each one more than the last, then one `mux` |
| Console lines and page errors | 0. No line with "Refused to" |

These are `dev` readings and stand for nothing in E-4.

**The verifier**, `python verify/verify_mp4.py <file> --profile <p> --expected-duration-ms 74705`, Python 3.13.5:

| File | Printed |
|---|---|
| The Creator export, `--profile creator` | `PASS 1` to `PASS 6`, the line for 7 to 11, exit 0 |
| The Free export, `--profile free` | `PASS 1` to `PASS 6`, exit 0 |

**What `ffprobe` says of the Creator export.** H.264 High, `yuv420p`, 1080 x 1920, 30/1, time base 1/30,000, 2,242 frames, 74.733 s, 8.24 Mb/s; 38 keyframes, one every 60 frames; every `pts` equals its `dts`, so no frame is out of order. AAC-LC, 48,000 Hz, 2 channels, 3,504 packets of 1,024 samples, 74.752 s, 160 kb/s. `moov` is in front of `mdat`. No edit list: the priming is 0. The Free export is the same at 720 x 1280 and 4.10 Mb/s. The pixel format is what the probe before this prompt said it would be.

**The pictures and the sound are the right ones, not only a valid file.** Read with `ffmpeg` and NumPy, outside the app:

| Asked | Result |
|---|---|
| Which output frames are white | Frames 150 and 1,946, and no other, in both files: the two frames Prompt 49 worked out. Their neighbours are not white (192 and 191; 229 and 234, of 255) |
| The sound against the source's own, sample by sample | No shift. At a shift of 0 the two differ by 0.4% of the level (median of the 1,098 windows of 50 ms that hold sound; 7.6% in the worst), which is what AAC costs. The first sample that is not silence is sample 9,052 in both |
| The audio's length | 3,588,096 samples: `N x 1600` = 3,587,200, rounded up to whole AAC frames. 18.7 ms longer than the video |
| Three frames, looked at | The speaker upright in the centre strip; white captions with the active word in yellow; at 63.0 s the number counting up, `$11,556` on its way to `$12k`, as the amount is spoken |

**Check 5 passes with `AAC_PRIMING_SAMPLES = 0`.** The prompt allowed it to fail until Prompt 51. On the development machine the encoder's output decodes with no shift, so there is nothing to edit out here. TE-4 reads it on R1, and R1's encoder may differ.

**Checked beyond the prompt,** in the same build, most of it on a 6-second cut of the clip with a hand-made transcript (186 frames, no model):

| Asked | Result |
|---|---|
| `exportClip` before `openSession`; before `setScene`; after `closeSession` | `E_INTERNAL`, detail `NoSession`; `NoScene`; `NoSession`. Stage `render_encode` |
| `previewPlay`, `previewPause`, `previewSeek`, `redetectSentence` | `E_INTERNAL`, `NotImplemented`, stage `preview` |
| `notify("previewClock", ...)`, then `detect` | The worker still answers: 1 event |
| `closeSession` twice; then `extractAudio` in the media worker; then `openSession` again | No failure; the audio again; a new session. The clip's file was given back |
| Two exports one after the other on one session | Both written. The second is faster: 2.8 s after 7.1 s on the short clip |
| A preview canvas attached before the exports (`attachPreview` with a canvas of the page) | Two exports written and no failure: the canvas is given back to the session after each. Not tried with a cancel |
| Cancel at once; after 60 frames; after 185 of 186 | `{ cancelled: true }` each time, in 0.1 s, 2.7 s and 3.8 s; `exports/` gains no file and `exports/tmp/` is empty. An export after the three is written |
| **The frame count can fail:** the line `closeFrame(vf);` taken out of the loop | `exportClip` rejects with `E_INTERNAL`, detail `FrameLeak`, stage `render_encode`; no file in `exports/`, none in `exports/tmp/`. This is the drill Prompt 51 names. Restored from a copy |
| The verifier on the short exports, `--expected-duration-ms 6192` | Six passes, Creator and Free |

The cancel went through a client made for the test, in the hook: `pool` has no cancel before V4.

**Decided here, where the plan is silent or cannot be built as written.**

- **`runExport` takes two arguments more than TS §21.1 gives it.** `canvas`: method A makes each frame from the export canvas, and the signature hands the loop none. `onEncoded`, called once before `finalize()`: the handler must post the `mux` progress message and return both timings, and nothing in the signature says where one stage ends. The return type is the one of TS §21.1.
- **The muxer is made when the first chunk of each encoder has come.** TS §21.3 and §16.8 make it "on the first video chunk", and both encode the audio after `venc.flush()`. But `Mp4Muxer::new` takes the `avcC` and the `asc` together, and the `asc` comes with the audio encoder's first chunk. Built as written, all 2,242 video chunks, 75 MB, would wait in memory for the end of the video, which TS §31 forbids. So the first 47 AAC frames, one second, are encoded before the first video frame, and the rest after `venc.flush()` as the plan says. The few chunks that come before the muxer exists wait in a list.
- **A chunk's frame number is read from its timestamp.** Counted in the order of arrival, a chunk out of its turn would be written in the wrong place and no one would know. Read from the timestamp, the muxer refuses it (`E_MUX`, `OutOfOrder`), which is what §16.8 asks for.
- **After both flushes the written frames are counted:** not `frame_count()` is `E_ENCODE_VIDEO`, `FrameCount`.
- **The audio encoder is given a frame only while it holds fewer than 32.** TS §31 has a number for the video encoder's queue and none for this one. Without a bound the whole clip, 28 MB of samples, is handed over at once.
- **A write the sink refuses is `E_STORAGE_QUOTA` or `E_STORAGE_IO`.** The muxer makes `E_MUX` (`Write`) of any throw of the sink and drops the reason. The loop keeps the sink's own error and reports that.
- **`render_encode` starts just before `runExport` is called,** as step 4 of §16.8 has it: it includes `pickVideoConfig` and the two `configure` calls. The table of §16.8 says "start of the loop". The user waits for both.
- **The preview is given back after every outcome,** a cancelled and a failed export too, once the scene was set to the export's profile; and only when a preview canvas is attached, as the guide says. §16.8 names the restore for a success and a failure and not for a cancel.
- **"Close every frame" is done by closing the `VideoSource` and making a new one.** The source owns the decoded frames and has no call that drops them and stays usable. The count is read after that and before the file is moved, so a leak leaves no file. The count then starts again at 0: a leak fails the export that leaked, not every export after it.
- **A failure while tidying up** is reported only when nothing failed before it. The first failure is the cause.
- **`openSession` closes a session that is still open.** Otherwise the first clip's file stays locked.
- **Stages.** `openSession`, `detect` and `setScene` fail with `detect_scene`, the stage `run-pipeline.ts` calls them in. `E_MUX` is of the stage `mux` wherever it shows. Everything else in `exportClip` is `render_encode` before `onEncoded` and `mux` after it.

**Differs from the prompt, the guide or the plan.**

- **`previewClock` does not answer `E_INTERNAL`: it is dropped.** It is a one-way message and nothing can answer it. A failure thrown in a one-way handler is reported with `reportError`, which reaches the page as an error of the worker, and `rpc.ts` then fails every call that is waiting with `E_WORKER_CRASH`. The other four answer `E_INTERNAL` as the prompt says.
- **The one-way and during-preview lists are in `render.worker.ts`,** given to `serveWorker`, and not in the row of `pool.ts`: a worker may not import `pool.ts` (known issue 13). The row has the script and the stage, like the other two.
- **`render.worker.ts` opens the clip's file with `openSource` of `media/import.ts`,** which already turns a storage failure into `E_STORAGE_IO`.
- **The capture method and the checks beyond the prompt,** above.

**Checked.**

- `pnpm --filter web exec tsc --noEmit` and ESLint on `src/workers`: clean.
- `node scripts/check-file-tree.mjs`: 212 files; both new files were in the tree of TS §5. The longest source file is now `export-loop.ts`, 395 lines.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green, the last one with Playwright's 6 workers at the first run; the frozen-file diff against `322c7d3` is empty. **255 Rust** (2 ignored), **92 Vitest**, **12 Playwright**, as before: V2 has no test file for these (§23.1).
- `grep -rl` for the test public key in `web/dist` after the plain build: 0 files. `check-hosts`: ok, 11 files.
- The app shell is 284.1 kB gzipped, was 266.5: the render worker's script.
- The hook, its import in `main.tsx`, the temporary case, the browser profile and the short clips are gone. `git grep -n "zz-spike\|zz_spike" -- web` prints nothing.

**"Done when".**

- [x] The first export passes verifier checks 1 to 4 and 6 (and 5); the wrong token is refused before anything is written.
- [x] `git grep -n "zz-spike\|zz_spike" -- web` is empty; the gate build holds no test key.

**Not checked.**

- **R1.** Every time here is the development machine's. Its encoder is the first entry of the ladder, `avc1.640028` with hardware.
- **That the preview draws again after an export.** The canvas is handed back without a failure; nothing can draw to it before Prompt 53.
- **An export with the tab hidden** (TE-3), a GPU device that is lost, a disk that fills, an encoder that fails in the middle. The paths are written; none was provoked.
- **A clip turned a quarter, or above 30 frames a second.**
- **The exported file in a player, by a person.** The agent looked at three frames. Nobody has heard it.

**Found, and not for this prompt.**

- **The export has more contrast than the source file** (known issue 30, open item 11). The reference clip is marked full-range (`yuvj420p`) and carries no colour description. Chrome gives its frames `fullRange: false`, BT.709, in the decoder and in its own `<video>` element alike, so it stretches them: a wall stored at 216 is shown at 231, a shadow stored at 77 at 71, and everything stored above 235 becomes white. The renderer draws what Chrome gives, and the encoder writes that. Measured on the stored brightness of three frames: the export's is the source's own (a fit gives 0.94 times it plus 4, bent by the highlights that were cut off), where a right export would store the source's times 0.86 plus 16. `ffmpeg` and VLC read the flag and show the clip flatter. The export does match what Chrome's player shows of the source, and the preview will match the export. The fix is on the decode side: give the `VideoDecoder` a `colorSpace` with `fullRange: true` when the clip says so, which needs the demuxer to read that flag from the SPS. Nothing in V2's plan reads it.
- **The file is not interleaved.** One second of audio, then all the video, then the rest of the audio: the order the plan encodes in. A player that reads a local file does not care. One that streams it must seek. The muxer takes samples in any order, so feeding the audio in step with the video would fix it; that is a change of §16.8.
- **The first export of a session is slower:** 7.1 s then 2.8 s on the short clip. The bench and E-4 should say which they report.
- **The hardware encoder writes no colour tags** (the probe before this prompt). A player assumes BT.709 for this size, which is what the pixels are.
- **A method without an argument cannot be called through `pool` as TypeScript reads it** (known issue 31): `pool.asr.unload()`, `pool.render.closeSession()` and `pool.render.previewPause()` are each "Expected 1-2 arguments, but got 0". Every call so far was made from a browser console or a test page, where nothing checks types. Prompts 53 and 54 are the first to write such a call in a source file.

**Open, for the human.** Nothing for this prompt: Prompt 51 is the next one marked **Push**. Before it: R1 (open item 3). Before Prompt 55: the decision of open item 11.

## 2026-10-09 - Outside a prompt: D-69, the gates of V2 are read on the development machine

No code changed. Five documents changed, all under `docs/v2/`.

**The decision (founder, 2026-10-09): D-69.** R1 is not available, and there is no second device. V2 is finished on the development machine, and the gates are read there. The founder's reasons: the build is ahead of the calendar, so there is time after the final version is deployed to try other devices and to change things on what testers report; and a user's laptop is expected to be at least as able as the founder's, which is two years old.

**What the agent had said first.** Asked whether Prompt 51 could start, it answered that its own half could and that the prompt could not close without R1: the capture method, the ladder entry and the three export timings of the gate were R1's to give. The founder then decided as above.

**The machine, D1.** An Acer Aspire A715-76G: Intel Core i5-12450H (8 cores, 12 threads), 16 GB, Intel UHD Graphics and an NVIDIA GeForce GTX 1650, Windows 11. Chrome under Playwright draws with the integrated GPU (known issue 9). R1 is a laptop of 2021 with 8 GB and an integrated GPU.

**What it costs,** written into D-69:

- **D1 is faster than R1, by about two on the one number there is.** Transcription of the reference clip took 62 to 96 s on D1 and about 150 s on the laptop like R1 of D-67.
- **A gate that passes on D1 does not show that it passes on R1.** One that fails on D1 fails on R1 too.
- **TE-14 is not answered.** Its question is whether the peak stays under 1.5 GB on a machine with 8 GB; D1 has 16.
- **The product's budgets are R1's** (PS §20.2), and an 8 GB laptop with an integrated GPU is a supported device (PS §9.3, §20.1). Nothing in V2 will have run on one. The M0 decision is taken on D1's numbers and must say so. Open item 12.

**Derived by the agent, not given by the founder.** Either can be set otherwise.

- **The name `d1`.** A reading taken for a gate is recorded under it, and the bench file is `bench/results/d1-<date>.json`, so that no number of this machine is ever filed as R1's. `dev` stays the label of a reading taken in passing.
- **The thresholds stay.** E-4 is still read against 112,000 ms and 187,000 ms, TE-3 against 30 frames a second, E-3 against 180,000 ms. They were set for R1; on D1 they are easier to meet.
- **All of V2's steps on R1 move, not only Prompt 51's:** the check of the deployed page, `pnpm e2e:device`, the bench, E-3 at S15 and TE-14 of Prompt 59, and the M0 gate of Prompt 60.
- **The agent takes the readings.** They were the human's because R1 was another machine. Pushes, the merge and the M0 decision stay the human's.

**Changed.**

| File | Change |
|---|---|
| `docs/v2/v2implementation.md` | D-69 added to §2, with its costs; §0 says "D-18 to D-69" |
| `docs/v2/coding-promptsv2.md` | The Standard Agent Block says "D-18 to D-69"; the note on the two gates; the rows of Prompts 51 and 59 in "Who does what"; Prompt 51 (the readings are taken on D1 by the agent, the push stays the human's, the third box); Prompt 59 (`BENCH_DEVICE=d1`, `d1-<date>.json`); Prompt 60 (D-69 is copied back beside D-65); the risk row "R1 not available" |
| `docs/v2/v2buildguide.md` | The table of references names D-69; item 12 of "Read this first" says where the readings of Steps 8.4, 8.5 and 12.3 to 12.7 are taken |
| `docs/v2/experiments.md` | "How it is kept": the machine, and the labels `d1` and `dev` |
| `docs/v2/v2changelog.md` | This entry; open item 3 closed; open item 12 added |

The edit was made by a script that refuses a replacement unless its old text occurs the stated number of times, and that writes nothing unless every replacement can be applied: 20 of 20. Each file keeps its line endings.

**Not changed, and why.**

- **`product.md`, `technicalspec.md`, `buildplan.md`.** They name R1 about 70 times, as the device the product is measured on. Prompt 60 copies D-65 into them, and now D-69 beside it.
- **The other places in the three V2 documents that say "R1".** There are about a hundred. D-69 and the note on the gates say how each is read.
- **The thresholds,** above.
- **The founder's reading of E-3** (D-67). It is the only number of a laptop like R1 and stays on record as that.

**Checked.**

- `node scripts/check-file-tree.mjs` passes: the tree of TS §5 was not touched.
- `git diff --stat`: five files, all under `docs/v2/`. The frozen-file diff against the baseline is empty.
- The four-command gate was not run: no file that a build or a test reads was changed. The test counts are those of Prompt 50: **255 Rust, 92 Vitest, 12 Playwright**.

**Open.** Open item 12: the timings on a laptop like R1, after the deploy and before the page promises a time.

## 2026-10-09 - Prompt 51: TE-3, TE-4, first E-4 (gate)

**The gate says go, read on D1.** Rendering and encoding the reference clip took 29,641 ms at the median of three exports, against a line of 112,000 ms. TE-3 passed: method A stays. TE-4 passed: the first entry of the ladder, and no priming. Every reading is the development machine's (D-69). None of them shows what a laptop like R1 does.

**Changed.**

| File | Change |
|---|---|
| `web/src/workers/render/export-loop.ts` | The comment at its head names the method TE-3 chose, and why. 396 lines. No line of code changed |
| `web/src/workers/render/encoders.ts` | The comment on `AAC_PRIMING_SAMPLES` says what TE-4 measured, and on which machine; the line itself ends with `// TE-4: measured`. The value is 0, as before |
| `docs/v2/experiments.md` | The records of TE-3, TE-4 and E-4 at S11; their three rows of the table |
| `docs/technicalspec.md` | §21.4: the capture method TE-3 picked, in one sentence |
| `docs/v2/v2implementation.md` | §16.8 (the capture row) and §16.9 (the comment on the priming) say what was found (2 replacements, each applied once) |
| `docs/v2/v2changelog.md` | This entry; known issues 29 corrected, 32 and 33 added |

**Made for the experiments, and gone.** The hook `web/src/zz-spike.ts` and its import in `main.tsx`; a temporary Playwright case; in `export-loop.ts` and `render.worker.ts`, a second capture path, a pass that renders without encoding, and the reading of the encoder's choice and of its first audio chunk; the browser profile; every export; a 6-second cut of the clip. `git status` showed the two comment changes and the documents, and nothing else, before the gate.

**TE-3: passed. Method A.** The record is in `experiments.md`.

| Read | Result |
|---|---|
| Method A, `render_encode`, three exports | 30,943 ms, 28,566 ms, 35,596 ms |
| A readback of the pixels, two exports | 81,417 ms, 79,140 ms: 2.8 times as long |
| Ten frames of one against the other | The same pictures once the brightness range is matched: 0.05 to 0.72 of 255 apart. As stored they are not the same: the readback's file holds brightness from 0 to 255 in a stream that carries no range flag |
| Render only, no encoder | 149 and 130 frames a second; 109 with a `VideoFrame` made and closed for each. The line is 30 |
| An export with the page hidden | Written in 35,097 ms; the page was hidden from 0.3 s after the start to the end; six passes |

**TE-4: passed.** `pickVideoConfig` returns `avc1.640028` with `prefer-hardware`, the first entry, at 1080 x 1920 and at 720 x 1280. The audio encoder's first chunk has the timestamp 0; it gives 3,504 chunks for 3,503.1 frames of input, none in front; the decoded sound of an export lies on the source's with no shift. The bitrates stay: the files hold 8.24 Mb/s and 4.10 Mb/s. `encoders.ts` and `profile.rs` both still say 8,000,000.

**E-4 at S11: 29,641 ms. Continue.**

| Read | Value |
|---|---|
| `render_encode`, three exports, each in a page just loaded | 28,286 ms, 29,641 ms, 37,631 ms |
| Median; normalised to 60 s | 29,641 ms; 23,807 ms |
| The line for the reference clip | 112,000 ms. Over 187,000 ms is a stop |
| `mux`, median | 85 ms, against 2,500 ms |
| The verifier on the three files | Six passes each. White frames at 150 and 1,946 |

**The drill.** With the line `closeFrame(vf);` taken out of the loop, `exportClip` rejects with `E_INTERNAL`, detail `FrameLeak`, stage `render_encode`, and `exports/` holds nothing but an empty `tmp`. It is the `vf.close()` the prompt names (known issue 29). Restored from a copy.

**Method B was not built as TS §21.4 names it.** `copyTextureToBuffer` runs inside the Rust renderer: JavaScript cannot reach the session's GPU device. Trying it means new Rust for a path that is only the fallback of method A, and method A had passed every check of Prompt 50 at more than twice the speed the line asks. In its place the second capture read the same pixels back through the browser, a 2D canvas and `getImageData`, and made the frame from that buffer of RGBA bytes, which is what the readback of TS would hand the encoder too. It gave the comparison the experiment wants, a second and independent way to the same ten frames, and it gave a time. `experiments.md` says so, and says that the readback of TS itself was not timed.

**The comparison found something the plan did not ask about.** A frame made from a buffer of RGBA bytes is encoded with brightness from 0 to 255, and the stream does not say so. A frame made from the canvas is encoded from 16 to 235, which is what a stream without a flag is taken to hold. So the readback's file is shown with too much contrast by every player, and method A's is right. Whoever builds a readback later must give its frames a colour space (known issue 32).

**The hidden-page check took three tries, and the first two did not count.** A second tab brought to the front, and then the window minimised, both left `document.visibilityState` at `visible`: a page under Playwright always counts as focused and visible, and Playwright starts Chrome with the throttling of hidden pages switched off. The export ran each time and passed the verifier, but that shows nothing about a hidden page. The third try started an ordinary Chrome by hand and drove it over its DevTools port, with a script kept outside the repository. There the page was `hidden` in all 19 samples, and the export was written. Known issue 33.

**Differs from the prompt, the guide or the plan.**

- **The readings are D1's and the agent took them,** where the prompt put them on R1 and in the human's hands. D-69, recorded before this prompt.
- **Method B,** above.
- **"Export again" after the priming is set.** The value did not change, so no export was made for that alone. The three exports of E-4 were made with the code as committed and pass all six checks.
- **The three exports of E-4 each ran in a page just loaded.** The guide says "run the export three times". A visitor's export is the first of its page, and a later one in the same session was not faster here.
- **TS §39.3 is not touched.** §24 asks for each measured assumption to be replaced there; Prompt 60 does that for all of V2 in one edit. TS §21.2 is unchanged because nothing in it changed.

**Checked.**

- G M-8: `pnpm check` green; `pnpm build` green, made without the test key, and `grep -rl` for the key in `web/dist` finds 0 files; the verifier passes a spike export on all six checks, exit 0; `grep -n "AAC_PRIMING_SAMPLES" web/src/workers/render/encoders.ts` shows the value with its TE-4 comment; `git grep -n "zz-spike\|zz_spike" -- web` prints nothing; `grep -c "TE-3\|TE-4\|E-4" docs/v2/experiments.md` is 8.
- `grep -c "8_000_000\|8000000"`: 1 in `encoders.ts`, 2 in `profile.rs`, as before.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test` and `pnpm build` green; the frozen-file diff against `322c7d3` is empty. **255 Rust** (2 ignored), **92 Vitest**, **12 Playwright**.
- **`pnpm e2e` needed a second run.** The first, straight after the release build of both bundles, passed 11 of 12: "landing_view is sent once" waited 5 s for its event and did not get it, in a run where the first six cases took 14 to 19 s each. The second run, with the same 6 workers, passed 12 of 12 in 24 s. It is known issue 20. Nothing this prompt changed is read by that case: the two changes are comments.

**"Done when".**

- [x] A spike export made after `AAC_PRIMING_SAMPLES` was set passes all of checks 1 to 6; the white frame near 4.99 s is white at the same second in the output (frame 150, at 5.000 s).
- [x] TE-3 and TE-4 are recorded with the method and the ladder entry; no build output contains the test key; G M-8 passes.
- [x] E-4: the median on D1 is recorded, 29,641 ms, and is not above 187,000 ms (D-69).

**Not checked.**

- **R1, or any second machine** (D-69, open item 12). The line is 3.8 times D1's reading. Nothing says what the factor between D1 and a laptop like R1 is for drawing and encoding.
- **The readback of TS §21.4 itself;** the software entries of the ladder; an encoder other than this machine's.
- **A page that Chrome has frozen or discarded** after a long time in the background.
- **Why the time rose over the session,** from 28 s to about 40 s, and the Free export's from 15 s to 30 s. Heat is the likely cause on a laptop; it was not measured. The bench of Prompt 59 runs ten times in a row and will meet it.

**Found, and not for this prompt.**

- **The first case of `pnpm e2e` to fail under load is now a different one each time.** Every page of the suite compiles the 6 MB render bundle at its start since Prompt 48, in six browsers at once, and the cases that wait 5 s for an event have little room left on this machine. `ci` retries a failed case once. Not a fault of a case; a cost of the preload (entry of Prompt 48).

**Open, for the human.** Push `v2-build` (three commits: Prompt 50, D-69, Prompt 51) and read `ci` on pull request #2. Prompt 52 waits for green. Before Prompt 55: open item 11, how a full-range clip should look.

## 2026-10-09 - The tenth and eleventh pushes: `ci` green on Prompts 49 to 51

**Done by the human.** Two pushes of `v2-build`: at `1685731`, the commit of Prompt 49, and at `a146350`, the commit of Prompt 51, with Prompt 50 (`945c9a2`) and D-69 (`36c9a60`). The first had no entry of its own.

**Read by the agent** (the public API of GitHub).

| Read | Result |
|---|---|
| `origin/v2-build` | `a146350`, equal to the local branch |
| The `ci` run on `1685731` (pull request #2, run 17) | Success, in 204 s |
| The `ci` run on `a146350` (run 18) | Success, in 520 s. The two deploy jobs were skipped, as on every run of the branch |
| Why run 18 took longer | One step: "Install Chrome for Playwright" took 331 s, where it took 24 s in run 17. The restore of the Cargo cache took 48 s for 23 s. Every step that runs the repository's code took the same or less: `cargo test` 21 s, "Build the WASM bundle" 36 s, Playwright 11 s |

**Closes.** The push that Prompt 51 ends with. `ci` built and tested the render worker and the export loop on Linux for the first time; it exports nothing, since no case opens a session.

**Not known.** Why the download of Chrome was slow on that runner: the job's log needs a login. The step downloads Chrome and its system packages on every run, so its time is the network's of that day. If it happens again, the cure is in `ci.yml`, which Prompt 58 changes anyway: use the Chrome of the runner's image, or keep the download in a cache.

**Changed.** This file only. The entry was written with the next commit.

## 2026-10-09 - Prompt 52: machines, stores, blockers, entitlement repo

**Added.**

| File | Content |
|---|---|
| `web/src/state/machines/clip-machine.ts` | `ClipStatus` (8), `ClipEvent` (13), `CLIP_MACHINE`: the 17 pairs of TS §12.2 |
| `web/src/state/machines/export-machine.ts` | `ExportStatus` (8), `ExportEvent` (12), `EXPORT_MACHINE`: the 16 pairs |
| `web/src/state/clip-store.ts` | `useClipStore` with the state of §18.2, `out48` in it; `begin`, `accepted`, `rejected`, `failed`, `processing`, `pushFeed`, `ready`, `noSpeech`, `reset`; and `noteFailure` |
| `web/src/state/preview-store.ts` | `usePreviewStore` (`status`, `playedOnce`), its table of 25 pairs inside the file (D-19); `attached`, `play`, `pause`, `ended`, `lock`, `unlock`, `detached` |
| `web/src/state/export-store.ts` | `useExportStore` with `unavailable`; `start`, `blocked`, `clear`, `progress`, `encoded`, `finalized`, `saved`, `fail`, `reset`, `setUnavailable` |
| `web/src/state/blockers.ts` | `BLOCKER_CODES` (eight, in the order of TS §12.5), `BlockerCode`, `forImport()`, `forExport()` |
| `web/src/persistence/entitlement-repo.ts` | `get()` and `put(token)` on the store `entitlement`, key `"current"` |

**Changed.**

| File | Change |
|---|---|
| `scripts/check-copy-codes.mjs` | Reads a fourth list, `BLOCKER_CODES`, from `web/src/state/blockers.ts`; a key of `messages.ts` that starts with `B_` counts as copy; `COPY_PENDING` gains `B_STORAGE_LOW`, `B_NOT_SIGNED_IN`, `B_ENTITLEMENT_EXPIRED`, `B_NO_FREE_EXPORTS`. It now prints "BLOCKER_CODES 4 of 8. Pending: 38" |
| `web/src/copy/messages.ts` | `blockers`: `B_UNSUPPORTED`, `B_PIPELINE_BUSY`, `B_EXPORT_IN_PROGRESS`, `B_PIPELINE_NOT_READY` |
| `docs/v2/v2implementation.md` | §18.2, §18.3 and §15.3 say what was built (3 replacements, each applied once) |
| `docs/v2/v2changelog.md` | This entry, and the one of the two pushes before it; known issue 34 |

No dependency was added.

**The three drills.** Each was undone from a copy, and the check passes after the last.

| Temporary edit | `node scripts/check-copy-codes.mjs` |
|---|---|
| `messages.blockers.B_PIPELINE_BUSY` deleted | "B_PIPELINE_BUSY: has no copy in messages.ts and is not in COPY_PENDING.", exit 1 |
| `B_UNSUPPORTED` added to `COPY_PENDING` | "B_UNSUPPORTED: has copy and is still in COPY_PENDING. Remove it from the list.", exit 1 |
| A ninth code, `B_NINTH`, added to `BLOCKER_CODES` | "B_NINTH: has no copy in messages.ts and is not in COPY_PENDING.", exit 1 |

**Browser check (dev).** `vite` on port 5173 with the bundles of the last build; Chrome under Playwright, headless; a script kept outside the repository that imports the modules from `/src/` and prints what it reads.

| Asked | Result |
|---|---|
| `accepted(info)` on an `idle` clip store | `IllegalTransitionError`: "clip: "accept" is not allowed in state "idle""; the status stays `idle` and no `clipInfo` is stored |
| Export store: `start(id)`, `clear()`, `encoded()`, `finalized()`, `saved()` | `gating`, `rendering`, `muxing`, `saving`, `done` |
| The repo: `get()`; `put("a.b")`, then `get()` | `undefined`; `{ token: "a.b", storedAt }`, `storedAt` a number |
| The record in IndexedDB | One, under the key `current`: `{ schemaVersion: 1, value: { token, storedAt } }` |
| The record deleted, then `get()` | `undefined` |

**Checked beyond the prompt,** in the same page:

| Asked | Result |
|---|---|
| The pairs of the two tables, counted from the modules | 17 and 16 |
| A clip that works: `begin`, `accepted`, `processing("probe_audio", true)`, `processing("asr", false)`, two `pushFeed`, `ready`, `reset` | `importing`, `accepted`, `processing` at `probe_audio` and waiting, `processing` at `asr`, `ready`, `idle`. In `ready`: no stage, not waiting, both feed lines in their order, `out48` kept, the clip id, source and `ClipInfo` kept. After `reset`: exactly the first state, `out48` `null` |
| `rejected(reason)`, then `processing`; `noSpeech()`; `failed(failure)` in `processing` | `rejected` with the reason, and `processing` refused; `rejected` with `REJECT_NO_SPEECH`; `failed` with the failure, no stage left |
| `begin` while `importing`, and while `ready`; `processing` while `importing` | Refused, each |
| `noteFailure(failure)` on a `ready` clip | The failure is stored and the status stays `ready` |
| `forImport()` and `forExport()`: no clip; a clip importing; a clip ready; an export in `gating` | `null` and `B_PIPELINE_NOT_READY`; `B_PIPELINE_BUSY` and `B_PIPELINE_NOT_READY`; `null` and `null`; `B_EXPORT_IN_PROGRESS` and `B_EXPORT_IN_PROGRESS` |
| Export store: `clear()` in `rendering`; `clear()` and `start()` in `done` | Refused, each |
| `progress(10, 100)` in `rendering`, then `encoded()` | 10 of 100; then `done` equals `total` |
| `fail(failure)` in `rendering`; `reset()`; `start`, `blocked()`; `setUnavailable(true)` | `failed` with the failure; `idle` and empty; `idle`; the flag set, the status unchanged |
| The preview store: `play` while `detached`; `attached`, `play`, `pause`, `play`, `ended`; `pause` while `stopped`; `lock` twice, `unlock`; `detached` twice; `lock` while `detached` | Refused; `stopped`, `playing`, `paused`, `playing`, `stopped`; refused; `locked`, `locked`, `paused`; `detached`, `detached`; `locked` |
| A record with `schemaVersion: 2` put in by hand, then `get()` | `undefined`, and the record is gone |

No console line and no page error.

**Decided here, where the plan is silent or cannot be built as written.**

- **`noteFailure(failure)` is a tenth action of the clip store.** §19.4 says a failure of the preview "stores the failure on the clip store", and the clip is `ready` then. The machine has no way from `ready` to `failed`, so `failed(failure)` would be refused. `noteFailure` sets the field and leaves the status. It is not a transition and goes through no table.
- **`processing(stage, waitingModel)` is the `run` event only for a clip that is not processing yet.** `run-pipeline.ts` calls it three times, once for each stage; on the second and third the stage and the wait change and the state does not.
- **`pushFeed` appends in every state.** "Never dropped" has no exception in TS §14.3. `begin` and `reset` start the feed again.
- **`lock` and `detach` are in the row of every state of the preview machine, `locked` and `detached` too.** §18.3 says "any state". So a preview that is not attached can be locked, which `start-export.ts` does when no player is on the page; `unlock` then leads to `paused`, and `control-preview.ts` detaches it again (Prompt 53).
- **`playedOnce` starts again at `attached()` and at `detached()`.** The store does not know which clip it shows; a new canvas is a new preview.
- **`start(exportId)` reads the clock** for `startedAt`, in milliseconds since the Unix epoch, and clears `unavailable`: an export that starts has a token.
- **`progress` is kept only in `rendering`;** `encoded()` sets `done` to `total`.
- **`get()` removes a record a newer build wrote,** as TS §23.3 says and `metaGet` does. §15.3 says only that it returns `undefined`.
- **The words of the four blockers** are the agent's, from the conditions of TS §12.5. `B_PIPELINE_BUSY` says to wait and does not offer a cancel, which arrives in V4.

**Differs from the prompt, the guide or the plan.**

- **The dev server was `vite` alone,** not `pnpm dev`: `pnpm dev` first replaces both release bundles with development builds, and nothing of this prompt is in a bundle.
- **The checks beyond the prompt,** and `noteFailure`, above.

**Checked.**

- `grep -o '"B_[A-Z_]*"' web/src/state/blockers.ts | sort -u | wc -l`: 8.
- `pnpm --filter web exec tsc --noEmit`; ESLint on `src/state`, `src/persistence` and `src/copy`: clean. `clip-store.ts` and `export-store.ts` import `AppFailure` and `FeedLine` from `workers/protocol.ts` as types (D-27 f).
- `node scripts/check-file-tree.mjs`: 219 files; the seven new files were in the tree of TS §5.
- Lines: `clip-store.ts` 148, `export-store.ts` 102, `preview-store.ts` 90, `blockers.ts` 57, `clip-machine.ts` 49, `export-machine.ts` 37, `entitlement-repo.ts` 29.
- No banned phrase in the copy: `git grep -n "never leaves\|GDPR\|DPDP\|CCPA\|SOC 2\|compliant" -- web/src/copy` prints nothing.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test` and `pnpm build` green; the frozen-file diff against `322c7d3` is empty; no test key in `web/dist`. **255 Rust** (2 ignored), **92 Vitest**, **12 Playwright**. The app shell is 284.3 kB gzipped.
- **`pnpm e2e` needed a second run again, and it was the same case as in Prompt 51:** "landing_view is sent once". Straight after the release build 11 of 12 passed; two runs after it passed 12 of 12 each, with the same 6 workers, in 25 s. The trace of the failed run was read this time (known issue 20): the page did post its batch, at the moment the case began to wait, and the fake API had not been handed that request when the wait of 5 s ended. Nothing this prompt added is loaded by the landing page: no component reads the new stores yet. **The entry of Prompt 51 said that a different case fails each time. That was wrong: it was this case both times.**

**"Done when".**

- [x] The three drills fired; `grep -o '"B_[A-Z_]*"' web/src/state/blockers.ts | sort -u | wc -l` is 8.
- [x] ESLint is clean for `src/state`: the type-only imports of `workers/protocol.ts` are accepted (D-27 f).

**Not checked.**

- **A production build's answer to an illegal transition:** there the attempt is reported and the state stays (V1). The check ran in a development build, where it throws.
- **Nothing calls these stores yet** but the check. `DropZone` reads `forImport()` from Prompt 55.
- No machine test and no `blockers.test.ts`: they are V4 and V5 (§23.1).

## 2026-10-10 - Prompt 53: preview loop, preview handlers, `control-preview`

**The preview plays.** With the reference clip taken to a scene by hand, `attach` and a click on play draw the clip with its captions as its audio sounds; a pause holds the picture; a second play goes on from there.

**Added.**

| File | Content |
|---|---|
| `web/src/workers/render/preview-loop.ts` | `runPreview(session, source, clock, isStopped)` and `PreviewStats` (TS §20.1, §20.2). 81 lines |
| `web/src/usecases/control-preview.ts` | `attach`, `play`, `pause`, `detach`, `lockForExport`, `unlockAfterExport`; the one `AudioContext`; `toTimeMs`, the one cast to `TimeMs` (D-59). 206 lines |

**Changed.**

| File | Change |
|---|---|
| `web/src/workers/render.worker.ts` | The real `previewPlay`, `previewClock` and `previewPause`; the clock, the stop flag and the running loop in the module's state; `closeSession` sets the stop flag. `redetectSentence` and `previewSeek` still answer `E_INTERNAL`. 372 lines |
| `web/src/workers/rpc.ts` | The type `Client<Api>`: a method the protocol gives no argument is called with none. No line that runs changed. 377 lines |
| `docs/v2/v2implementation.md` | §15.6, §16.6, §16.7 and §19.4 say what was built (4 replacements, each applied once) |
| `docs/v2/v2changelog.md` | This entry; known issue 31 closed, 35 added |

No dependency was added.

**Browser check (dev, model cached).** `vite` on port 5173 with the bundles of the last build; Chrome 155 under Playwright, headless, a new profile; a script kept outside the repository. The model was downloaded once (45.7 s), the clip transcribed (64.8 s, 157 words) and taken to a scene with `pool`; then a canvas of 540 x 960 was put on the page, with two buttons, because a play must follow a click.

| Asked | Result |
|---|---|
| `attach(canvas, out48)` | Answers in 388 ms; the preview is `stopped` |
| A click on play; two screenshots of the canvas 1 s apart | They differ. Neither is one flat colour: 3,685 and 3,501 different colours in a sample of each |
| A click on pause; two screenshots 500 ms apart | Identical. The preview is `paused`. No failure is stored on the clip store |
| A click on play again; screenshots | The picture differs from the paused one and goes on changing: it resumes |

**The loop keeps up.** `previewPlay` returns nothing, so for this reading one temporary line made it return the loop's `PreviewStats`; it is gone. The reference clip, a hand-made scene, the clock run from the page as `control-preview.ts` runs it:

| Played | Frames drawn | Late | Largest drift |
|---|---|---|---|
| 10 s from the start | 298 | 2 | 10 ms |
| 10 s from 10 s | 299 | 1 | 17 ms |
| 5 s from 40 s, a jump ahead | 150 | 1 | 17 ms |
| 3 s from 4 s, a jump back | 89 | 1 | 16 ms |
| 5 s from 60 s, the clock told once and never again | 149 | 1 | 17 ms |
| From 73 s to the end of the clip | 51; the loop returned by itself after 1,750 ms, which is the 1,733 ms that were left | 1 | 17 ms |

TS §20.3 asks for a drift of at most 80 ms and at most 5% of the frames late. These are 17 ms and under 1%, on D1. The one late frame of each play is the first: nothing is decoded yet. A pause was answered in 6 to 18 ms.

**Checked beyond the prompt.**

| Asked | Result |
|---|---|
| `play()` with nothing attached; `previewPlay` sent to a worker that has no canvas | Nothing happens; `E_INTERNAL`, detail `NoCanvas` |
| A click on play while playing; two `pause()` at once | Still `playing`, one loop; `paused`, no failure |
| `lockForExport()`; `play()` while locked; `unlockAfterExport()`; a click on play | `locked`; still `locked`; `paused`; it plays on from where it was |
| `lockForExport()` while playing | `locked`: the play was paused first. Then `paused` |
| `detect` on the worker after the plays | It answers |
| `detach()`, twice; `play()` after it | `detached`, `playedOnce` false; nothing happens |
| A 6-second clip played to its end | `stopped` 6,227 ms after the click, for a clip of 6,192 ms; the picture is still after it; no failure |
| A click on play after the end | It plays from the start |
| `preview_played` in the batches the page posted | 2, one for each preview that was attached; the first was played three times and the second twice. The event is `{ "name": "preview_played" }`, with no `props` |
| **A pause while the page is hidden** (an ordinary Chrome with its window minimised, driven over its DevTools port: known issue 33) | Answered in 4 ms, the loop returned, no failure. The same for a play that was started while hidden |
| Two screenshots looked at | The speaker in the centre strip, the caption in white with a dark outline, and on the short clip `$12k` large above "We made $12k in sales." |

No console line and no page error in any of it.

**Decided here, where the plan is silent or cannot be built as written.**

- **The end of the clip is the scene's frame count.** TS §20.2 ends the loop "if t >= clip duration", and `runPreview` is given no duration. The loop ends when the clock reaches frame `session.frame_count()`, which is the clip's end rounded up to a whole frame.
- **A step of the loop waits for the next animation frame, or for 100 ms when none comes.** TS says "per requestAnimationFrame". If a hidden page's worker got none, the loop would not see its stop flag, and `previewPause` would not answer until the page is shown again. In the Chrome tried here a pause in a minimised window was answered in 4 and 6 ms, sooner than the 100 ms would explain, so its worker seems to get animation frames still; the 100 ms are for a browser that gives none.
- **Nothing is drawn while no frame was ever decoded.** "Reuse the last frame" has nothing to reuse on the first step of a play. It is counted late.
- **After every play the worker closes the `VideoSource` and makes a new one,** as after an export, and then reads the frame count: not 0 is `E_INTERNAL`, `FrameLeak` (D-45). The picture stays on the canvas; a play after a pause decodes again from the keyframe before its time, which took one late frame in every reading above.
- **`previewPlay` without a scene or without a canvas is `E_INTERNAL`,** `NoScene` or `NoCanvas`.
- **`previewPause` answers when the loop has returned, however it ended.** A failure of the loop is the answer of `previewPlay`, and is reported once.
- **`play()` resolves when the preview is playing, not when it ends.** §19.4 gives `play(): Promise<void>` and does not say which. A button cannot wait 90 s for its click to finish. What follows the end, `ended()` and the stop of the sound, runs when the worker's loop returns.
- **`pause()` waits for the worker before the store says `paused`,** the order §19.4 writes. A play clicked in between finds the preview still `playing` and does nothing; otherwise it would ask the worker for a second loop while the first has not returned, which the worker refuses.
- **`attach` first detaches a preview that is attached.** One `AudioContext` at a time.
- **A failure of `attachPreview` or of `previewPause` is stored like one of `previewPlay`:** on the clip store, with `noteFailure` (Prompt 52). After a failed play the preview is detached, as §19.4 says.
- **`unlockAfterExport()` detaches a preview that has no canvas.** The table of TS §12.2 leads from `locked` to `paused` only, and `start-export.ts` locks also when no player is on the page.
- **`preview_played` is sent on the first play after each `attach`.** The store's `playedOnce` starts again there (Prompt 52).
- **The audio goes into the `AudioBuffer` with `getChannelData(0).set(out48)`:** one copy, which D-51 counted.

**Differs from the prompt, the guide or the plan.**

- **`rpc.ts` was changed, which the prompt does not name** (known issue 31). `pool.render.previewPause()` did not compile: the type `Client<Api>` of Prompt 37 asked for one argument on every method. `control-preview.ts` is the first source file to call a method without one. The fix is in the type alone; `pool.asr.unload()` and `pool.render.closeSession()` compile now too. `rpc.ts` has 377 lines, 23 short of the limit, and V4 adds to it (known issue 16).
- **The dev server was `vite` alone,** not `pnpm dev`, as in Prompt 52: the release bundles of the last build were used as they were.
- **The readings of the loop, and the checks beyond the prompt.**

**Checked.**

- `grep -c "previewPlay\|previewClock\|previewPause" web/src/workers/render.worker.ts`: 10. `grep -c "redetectSentence\|previewSeek"`: 3; both still answer `E_INTERNAL`.
- `pnpm --filter web exec tsc --noEmit`; ESLint on `src/workers` and `src/usecases`: clean. The one cast to `TimeMs` in `control-preview.ts` is the last `return` of `toTimeMs`, which is where the rule of D-59 allows it; `control-preview.ts` imports `ClockSync` from `workers/protocol.ts` as a type (D-27 f).
- `node scripts/check-file-tree.mjs`: 221 files; both new files were in the tree of TS §5.
- Lines: `control-preview.ts` 206, `preview-loop.ts` 81, `render.worker.ts` 372, `rpc.ts` 377.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test` and `pnpm build` green; the frozen-file diff against `322c7d3` is empty; no test key in `web/dist`. **255 Rust** (2 ignored), **92 Vitest**, **12 Playwright**. The app shell is 284.6 kB gzipped.
- **`pnpm e2e` needed a second run, for the third gate in a row, and this time it was worse:** straight after the release build the six cases that start first each ran into the case's limit of 30 s, and the other six passed. That is the picture of known issue 20, a machine with too little to spare. Two runs after it passed 12 of 12 each, with the same 6 workers, in 42 s and 31 s, where a rested machine takes 25 s. The machine had 4.4 GB of 15.7 free, with a disk scanner, two other browsers and the editor running beside the gate, after several hours of builds and exports. The landing page loads nothing this prompt added: no file imports `control-preview.ts` yet, and a worker's script is fetched at the start and run only when a clip arrives. `ci` is the check on a clean machine; it has not run on Prompts 52 and 53.
- The script of the check, the browser profile, the short clip, the screenshots and the temporary line in `render.worker.ts` are gone. `git status` showed the four files of this prompt and nothing else before the gate.

**"Done when".**

- [x] The three preview results hold; `grep -c "previewPlay\|previewClock\|previewPause" web/src/workers/render.worker.ts` is at least 3.
- [x] `tsc` and ESLint are clean for `src/workers` and `src/usecases`.

**Not checked.**

- **By ear.** Headless Chrome plays to no loudspeaker. That the sound and the picture agree was read from the clock: the frame drawn lay at most 17 ms from the audio's time. Nobody has heard it; Prompt 55 is where a person does.
- **A preview, then an export, then a preview.** The dev server ran without the test key, so no export could be made in this check. Prompt 54's check makes one with a player attached.
- **The preview in a production build under the CSP.** The check ran on the dev server. `AudioContext` and the worker's animation frames need nothing the CSP names.
- **A real `PreviewPlayer`.** The canvas and the buttons were the check's own.
- **R1** (D-69). The numbers are D1's, with a 720p source.

**Found, and not for this prompt.**

- **`PreviewStats` reaches nobody.** `runPreview` returns it and `previewPlay` answers `void`, as the protocol says. V4's `preview-sync.spec.ts` is to assert the drift and the late frames (TS §20.3) and will need a way to read them: a field of the protocol, which is frozen, or a second answer.
- **A paused preview keeps no decoded frame,** so the first frame after a resume comes one step late. It shows as one late frame in 150. If V4's seek wants the frame at once, the source must be kept across a pause, and the frame count of D-45 read another way.

**Open, for the human.** Nothing: Prompt 54 is the next one marked **Push**.

## 2026-10-10 - Outside a prompt: D-70, a clip is drawn with the brightness range its file states

No code changed. Four documents changed, all under `docs/v2/`.

**The decision (founder, 2026-10-10): D-70.** A clip should look exactly as it was shot. For the question of open item 11 that means: the export's contrast is put right. A clip whose file says it is full-range is decoded as full-range, so the preview and the export show the recording as the file stores it, and not stretched as Chrome's own player shows it.

**How the answer came.** The agent had asked how a full-range clip should look. The founder's first answer was about the shape of the picture: a clip shot upright should stay upright, and later a user should be able to choose between the whole webcam picture and the upright short. "Full-range" had been read as "the full picture". The agent said that the two are different things, and that the founder's rule, applied to brightness, means the fix; the founder agreed. The choice of an output shape is a separate wish and is not decided here: TS §21 names 1:1 and 16:9 output as an extension that is not built, and it would belong to V4.

**Tried before it was written down: the decoder obeys.** A script outside the repository gave Chrome 155 the first frames of the reference clip three times and read the first frame back.

| The decoder was told | The frame says | A wall stored at 216.1 | A shadow stored at 82.0 | A highlight stored at 247.0 |
|---|---|---|---|---|
| Nothing, as `video-source.ts` does today | `fullRange: false`, BT.709 | 230.7 | 76.8 | 255.0 |
| `fullRange: true`, BT.709 | `fullRange: true`, BT.709 | 216.3 | 82.0 | 246.8 |
| `fullRange: true`, BT.601 | `fullRange: true`, BT.601 | 217.5 | 81.6 | 247.5 |

So one field of the decoder's configuration puts the brightness right. The two matrices differ by about one in 255 here, in a picture with little colour.

**Changed.**

| File | Change |
|---|---|
| `docs/v2/v2implementation.md` | D-70 added to §2, with how it is built and its costs; §0 says "D-18 to D-70" |
| `docs/v2/coding-promptsv2.md` | The Standard Agent Block says "D-18 to D-70" |
| `docs/v2/v2buildguide.md` | The table of references names D-70 |
| `docs/v2/v2changelog.md` | This entry; open item 11 closed; open item 13 added; known issue 30 says what was decided |

The edit was made by a script that refuses a replacement unless its old text occurs exactly once, and that writes nothing unless every replacement can be applied: 7 of 7. Each file keeps its line endings.

**Not built.** The founder asked for no code change on the day of the decision. What it takes is in D-70 and in open item 13: the flag read in Rust from the `avcC`, one more method on `RenderSession`, and `colorSpace` in the configuration of `VideoSource`. No frozen type changes.

**What it costs,** written into D-70:

- **The export will not look like the same file in Chrome's player,** which keeps the stretch. It will look like the file in VLC and in `ffmpeg`.
- **New Rust:** a reader of the SPS. The SPS of a High-profile stream has scaling lists in front of the field that is wanted.
- **The colours of an unlabelled clip stay a guess.** The file says how bright, and not with which matrix it was made.

**Checked.**

- `node scripts/check-file-tree.mjs` passes: the tree of TS §5 was not touched.
- `git diff --stat`: four files, all under `docs/v2/`. The frozen-file diff against the baseline is empty.
- The four-command gate was not run: no file that a build or a test reads was changed. The test counts are those of Prompt 53: **255 Rust, 92 Vitest, 12 Playwright**.

**Open.** Open item 13, before Prompt 55.

## 2026-10-10 - Outside a prompt: D-70 built, a full-range clip keeps its brightness

**The founder's word (2026-10-10): "build it now".** Open item 13 is closed. An export of the reference clip now stores the brightness the recording has, and its highlights are no longer cut to white.

**Added.**

| File | Content |
|---|---|
| `crates/offcut-mp4/src/sps.rs` | `full_range(avcc) -> Option<bool>`: whether an H.264 stream says it uses the whole brightness range. 189 lines above its test module. **New in the tree of TS §5**, added there and in §4 of the plan in this commit |

**Changed.**

| File | Change |
|---|---|
| `crates/offcut-mp4/src/lib.rs` | `pub mod sps;` |
| `crates/offcut-wasm-render/src/session.rs` | `RenderSession::video_full_range()`, a fifth method beside the four of D-30 |
| `web/src/workers/render/video-source.ts` | `DemuxerHandle.video_full_range()`; `VideoSource` tells the decoder the range when a frame it decoded says limited-range, and decodes again. 399 lines |
| `docs/technicalspec.md` | §5: the line of `sps.rs` in the tree |
| `docs/v2/v2implementation.md` | §4 (the tree), a new §6.10, §13.5, §16.7, and the row of D-70 (6 edits, each applied once) |
| `docs/v2/v2changelog.md` | This entry; open item 13 and known issue 30 closed; known issue 36 |

No frozen type changed and no dependency was added: the flag travels as one more method of the session.

**Two things the probes showed, which changed how it is built.** Both were tried on Chrome 155 before a line was written, with scripts outside the repository.

| Asked | Found |
|---|---|
| Does the decoder obey a colour space that names the range alone, `{ fullRange: true }`? | No. The frame still says `fullRange: false` and the picture is still stretched. It obeys when all four of range, matrix, primaries and transfer are given |
| Does Chrome read the range by itself when the stream also describes its colours? | No. Five streams, full and limited, described and not, BT.709 and BT.601: every frame came out as `fullRange: false`, BT.709. D-70 had expected the fault only in a stream without a description |

So the rule is: **when the stream says full-range and a frame comes out of the decoder as limited-range, the decoder is told the range, together with the matrix, primaries and transfer that frame had, and decoding starts again from the same place.** It happens once for a clip, on its first frame; what was told is kept, and a source made later for the same clip starts with it. A browser that reads the range by itself is never told anything.

**Measured: the stored brightness of an export against its source.** Three clips, each exported at 1080 x 1920 in a keyed production build under the CSP; three frames of each; blocks of the top half of the picture, above the captions, compared on the brightness plane as stored.

| Clip | A right export stores | Before | Now |
|---|---|---|---|
| The reference clip: full-range, no description | source x 0.859 + 16 | x 0.940 + 4, on average 6.9 of 255 from the right value and up to 15; everything above 235 cut off | x 0.860 + 15.4, on average 0.6 from the right value; the source's 251 is stored as 232 |
| The same picture as a full-range stream that describes its colours (libx264, High) | source x 0.859 + 16 | not exported before | x 0.860 + 15.4, 0.5 from the right value |
| The same picture as a limited-range stream | the source's own values | not exported before | x 1.001 - 0.6, 0.5 from the right value: unchanged, as it must be |

The remaining half a step of 255 is what encoding twice costs.

| Also read | Result |
|---|---|
| The verifier on the three exports | Six passes each |
| The white frames of the reference export | Frames 150 and 1,946, and no other; their neighbours are now 188 and 187, 223 and 228, where they were 192 and 191, 229 and 234 |
| The preview of each clip, 1.5 s, then the canvas read where the wall is | Played, no failure; 208, 210 and 210 of 255 for the three clips, which are one picture |
| A second export on the same session | Written: the decoder starts with what it was told |
| `render_encode` of the reference clip | 38,933 ms and 37,738 ms, on a machine that had been at work for hours; 28 to 43 s before the change (entry of Prompt 51). The change costs the decoding of one frame, once for a clip |

The reference export is kept beside the one of Prompt 50, in `testclips/renders/`, which git ignores: `d70-export-creator-1080x1920.mp4` and `p50-export-creator-1080x1920.mp4`. Played one after the other they show the difference.

**The reader in Rust.** The flag is one bit of the sequence parameter set, behind fields of varying length: the chroma format and the scaling lists of the High profiles, the picture order count, the cropping, the aspect ratio. `sps.rs` reads those only to get past them, after taking out the bytes that guard a start code. Seven inline tests:

| Test | With |
|---|---|
| The reference clip says full range | Its `avcC`, 41 bytes |
| Full range in every profile, and behind a written-out aspect ratio | Three streams of libx264: High with a colour description, Constrained Baseline, High with a 15:16 aspect ratio |
| Limited range is not full range | High, `color_range=tv` |
| A stream that does not say gives `None` | libx264 with nothing asked; an export of Offcut itself, from the browser's hardware encoder |
| The flag behind scaling lists, fields and cropping | A parameter set written bit by bit in the test: no encoder at hand puts scaling lists into the sequence parameter set |
| The guard bytes are left out | `00 00 03 01` |
| Bytes that are no parameter set | Empty, cut off at every length, the wrong NAL type, all zeros, all ones: `None`, and no panic |

The `avcC` bytes in the tests are the settings of a stream; they hold nothing of a picture.

**Decided here.**

- **The decoder is told only after a frame showed that it is needed,** and with that frame's colours. D-70 said the matrix, primaries and transfer "stay what the browser assumes"; the browser's assumption cannot be asked for, only seen on a frame. Guessing it in the code, BT.709 above a size and BT.601 below, would have been a second rule to keep in step with the browser.
- **`None` from the reader means "leave it to the decoder".** A stream that says nothing is not touched, and neither is one the reader cannot follow.
- **Where the frame names no matrix, primaries or transfer, BT.709 is given.** The browser obeys nothing less than all four.
- **`video_full_range()` is a method of the session,** not a field of `ClipInfo`, which is frozen.

**Differs from D-70 as written.** The decision named a stream "that states the range and no colour description". The fault is wider, and the rule covers the wider case. The row of D-70 says so now.

**Checked.**

- `cargo test -p offcut-mp4 sps`: 7 passed. `cargo clippy` on `offcut-mp4` and `offcut-wasm-render`, all targets, `-D warnings`: clean.
- `pnpm --filter web exec tsc --noEmit` and ESLint on `src/workers`, with the rebuilt bundle: clean.
- `node scripts/check-file-tree.mjs`: 222 files; `sps.rs` is in the tree.
- The render bundle is 6,033,574 bytes, was 6,029,652.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build` and `pnpm e2e` green, the last one with Playwright's 6 workers at the first run, in 33 s; the frozen-file diff against `322c7d3` is empty; no test key in `web/dist`. **262 Rust** (2 ignored; it was 255, the seven are new), **92 Vitest**, **12 Playwright**. The app shell is 284.8 kB gzipped.
- The hook, its import in `main.tsx`, the temporary Playwright case, the browser profile, the two clips made for the check and their exports are gone. `git status` showed the files of this change and nothing else before the gate.

**Not checked.**

- **By eye.** The brightness was measured, not looked at. The two files are there to be played.
- **A browser that reads the range by itself.** None was at hand; there the rule does nothing.
- **A full-range clip whose colours are not BT.709.** Chrome assumed BT.709 also for a stream described as BT.601, and the rule keeps the browser's assumption: the brightness is right, and the colours of such a clip may be a little off, as they were before.
- **A clip turned a quarter, HEVC, a second machine.** HEVC is refused at import.

**Found, and not for this change.**

- **Chrome reads no colour information from an H.264 stream in this decoder,** not the range and not the matrix, whatever the stream says. D-70 corrects the range. Telling the decoder the stream's own matrix as well would need the reader to return the description, and a way to say it in the terms of the decoder's configuration.
- **`video-source.ts` is at 399 lines** (known issue 36).

## 2026-10-10 - The twelfth and thirteenth pushes: `ci` green on Prompts 52 and 53 and on D-70

**Done by the human.** Two pushes of `v2-build`: at `e09ede7`, the commit of Prompt 53, with Prompt 52 (`ff5f962`); and at `477b589`, the commit that built D-70, with its decision (`796b87d`).

**Read by the agent** (the public API of GitHub).

| Read | Result |
|---|---|
| `origin/v2-build` | `477b589`, equal to the local branch |
| The `ci` run on `e09ede7` (pull request #2, run 19) | Success |
| The `ci` run on `477b589` (run 20) | Success |

**Closes.** The line "`ci` ... has not run on Prompts 52 and 53" of the entry of Prompt 53. The times of the two runs were not read.

**Changed.** This file only. The entry was written with the commit of Prompt 54.

## 2026-10-10 - Prompt 54: use-cases: pipeline, import, export, app start

**One call each way.** `importClip(file, "user")` takes the reference clip from a file to a clip that is `ready`, with its transcript, its one event and its audio. `startExport(clipId)`, with a seeded Creator token, ends in a downloaded MP4 of 1080 x 1920 that passes the verifier's six checks. Nothing on a page calls either yet: that is Prompt 55.

**Added.**

| File | Content |
|---|---|
| `web/src/usecases/run-pipeline.ts` | `runPipeline(clipId)`: the 12 steps of §19.3; the wait for the model, with `model_download`; `startTimer(clipId)`. 298 lines |
| `web/src/usecases/import-clip.ts` | `importClip` (9 steps), `importSampleClip`, `dismissClip`, `newClipId`, the one cast to `ClipId` (D-59). 210 lines |
| `web/src/usecases/start-export.ts` | `startExport(clipId)`: the 11 steps of §19.5; `newExportId`, a UUID of version 7 and the one cast to `ExportId`; `planOf`; the download. 218 lines |

**Changed.**

| File | Change |
|---|---|
| `web/src/usecases/start-app.ts` | Step 8: `void modelManager.inspect();`, inside the branch of a supported browser, after step 7; the import of the model manager. The marker `// V2: modelManager.inspect()` is gone. 102 lines |
| `docs/v2/v2implementation.md` | §19.2, §19.3 and §19.5 say what was built; §27 gains items 29, 30 and 31 (4 edits) |
| `docs/v2/v2changelog.md` | This entry, and the one of the two pushes before it; known issue 37; a line in known issue 20 |

No dependency was added. No Rust file, no worker and no store changed.

**The three drills.** Each was undone from a copy, and ESLint is clean after the last.

| Temporary edit | ESLint |
|---|---|
| `import { startExport } from "./start-export";` in `run-pipeline.ts` | `boundaries/dependencies`: "There is no policy allowing dependencies from elements of type "usecases" to elements of type "usecases"" |
| A second cast, `"x" as ClipId`, in `import-clip.ts` | `no-restricted-syntax`: "In this file one cast is allowed: to ClipId, in the last return of newClipId() (D-59). Every other value must arrive already typed" |
| `fetch("/x")` in `start-export.ts` | `no-restricted-globals`: "Unexpected use of 'fetch'. fetch is not available here. Network requests go through net/http.ts or net/asset-fetch.ts" |

**Browser check (dev).** `vite` on port 5173 with the bundles of the last build, started with the test public key in the environment of that one process; Chrome under Playwright, headless, a new profile under `fixtures/.cache/`; a temporary case, `zz-p54.spec.ts`, that imports the modules from `/src/`, calls them and prints what it reads; the fake API of the E2E helpers, so every analytics batch could be read. The key was worked out from the seed in `fake-api.ts` and printed nowhere; the tokens were minted by `mintEntitlementToken`.

The four results the prompt names:

| Asked | Result |
|---|---|
| `importClip(file, "user")`, the model cached | `ready` after 60.4 s. `events.length` 1; `feed` is `transcribing`, then `event_found` with `$12k`; `out48.length` 3,585,840; 157 words, 157 entries of prosody. Requests from the call to `ready`: one, `POST /api/v1/events`, and no other of any kind |
| `startExport(clipId)` with no `entitlement` record | `unavailable: true`; the status stays `idle`; the directory `exports/` does not exist; no `export_started` is sent |
| `startExport(clipId)` with a seeded Creator token | `done`, 2,242 of 2,242 frames. The download is named `offcut-20261010-0158.mp4`, 78,965,811 bytes. `python verify/verify_mp4.py <file> --profile creator --expected-duration-ms 74705`: `PASS 1` to `PASS 6`, exit 0 |
| `dismissClip()` | The clip store is `idle`, `out48` is `null`; `clips/` is empty |

What the stores and the analytics said on the way:

| Asked | Result |
|---|---|
| The model store after the app's start, in a new profile | `absent`: step 8 ran. After a reload with the model on the device: `ready` |
| The first import, with no model on the device | The clip is `processing` at `probe_audio` with `waitingModel: true` while the model store says `downloading`; then `asr`, `detect_scene`, `ready`. 110 s in all. `model_download { outcome: "ok", duration_ms: 47975, resumed: false }` |
| The events of the cached import | `clip_accepted { duration_bucket: "lte90", orientation: "landscape", source: "user" }`; `stage_timing` for `probe_audio` (623 ms), `asr` (59,805 ms, `asr_backend: "webgpu"`) and `detect_scene` (18 ms); `pipeline_done { total_ms: 60538, n_number: 1, n_list: 0, n_from_to: 0, n_keyword: 0 }`. No `model_download`, none for `audio_chain` |
| OPFS after `ready` | `clips/<id>/source`, 35,201,023 bytes, and `clips/<id>/out48.f32`, 14,343,360 bytes, which is 3,585,840 x 4 |
| The Creator export, started while a preview played | The preview store is `locked` while the export renders and `paused` after it. `export_started { profile: "creator" }`; `stage_timing` for `render_encode` (25,454 ms) and `mux` (19 ms); `export_done` with `total_ms: 25535`, `profile: "creator"`, `events_kept: 1`, `style: "clean"`, the other counts 0 and both flags `false`. Requests during it: one, `POST /api/v1/events`. `exports/` holds `<exportId>.mp4` and an empty `tmp/` |
| The export's id | `01a1225a-4e4b-70f8-83ea-782f754964a5`: the version is 7, the variant `10`, and the first 48 bits are the time |
| A click on play after the export | It plays: two screenshots of the canvas 1 s apart differ, and no failure is stored |
| The eight events of §1.1 | All eight were sent in this run: `clip_accepted`, `model_download`, `stage_timing`, `pipeline_done`, `preview_played`, `export_started`, `export_done`, `export_failed` |

No console line of the kind error or warning and no page error in that run.

**Checked beyond the prompt,** in the same run and in a second one with a 12-second cut of the clip and a silent 8-second one:

| Asked | Result |
|---|---|
| A second export, with a Free token | `done`; the file is 720 x 1280, 40,060,567 bytes, and passes the six checks with `--profile free`. Its id is new, and `exports/` holds this file alone: the Creator file is gone (D-40) |
| A token signed with another key | The export is `failed` with `E_ENTITLEMENT_INVALID`, stage `render_encode`; `export_started`, then `export_failed { error_code: "E_ENTITLEMENT_INVALID", stage: "render_encode" }`; `exports/` does not exist; the preview is `paused` again, not locked |
| `dismissClip()` while the preview plays | `idle`; the failed export is gone from the export store; `clips/` is empty. The worker's loop was stopped first, so the close was not refused |
| A text file named `x.mp4` | `rejected` with `REJECT_CONTAINER`; `clips/` is empty; no event. The next `importClip`, on top of the rejected clip, works |
| `importSampleClip()` called twice at once | One request to the asset host; `clip_accepted` with `source: "sample"`, once; the clip is `processing` |
| `importClip` called twice at once | One clip, one `clip_accepted` |
| A 12-second clip with no amount in it | `ready` with 0 events and 28 words; the feed is `transcribing` alone; `pipeline_done` with four zeros; `duration_bucket: "lt30"` |
| A clip with no sound | `rejected` with `REJECT_NO_SPEECH`; `clips/` is empty; `stage_timing` for `probe_audio` and `asr`, no `pipeline_done` |
| The model's files removed from OPFS while the store says `ready` | The clip is `failed` with `E_ASR_RUNTIME`, detail `Load`, stage `asr`; its directory stays until `dismissClip()`, after which `clips/` is empty. The runtime writes two warnings to the console. After a reload the model store says `absent` |
| An export whose `out48.f32` was removed, asked for twice at once | One export: `failed` with `E_STORAGE_IO`, stage `storage`; one `export_started` and one `export_failed`; the preview, which had no canvas, is `detached` again |
| `startExport` again after that failure | The store goes from `failed` through `idle` to a new export, which fails the same way |
| `startExport` with the id of another clip; `dismissClip()` with no clip | Nothing happens, each |

**Decided here, where the plan is silent or cannot be built as written.** Each is in §19 of the plan now.

- **`run-pipeline.ts` exports `startTimer(clipId)`.** Step 5 of `importClip` starts a timer that `runPipeline` reads, and the plan gives it no home: `runPipeline(clipId)` carries no time and the clip store has no field for one.
- **`dismissClip` sends `previewPause` before `closeSession`.** The plan resets the store and closes the session. But the player is detached by its component, which runs after the reset, and the worker does one job at a time: a close that arrives beside a running preview is refused, the clip's file stays locked, and its directory cannot be removed. `previewPause` is accepted beside a preview and answers when the loop has returned.
- **`dismissClip` does nothing for a clip that is `idle`, being imported or being processed, and nothing while an export runs.** The machine has no `reset` from those states, and a session cannot be closed under a pipeline or an export. It asks `forImport()`, which is where those conditions are.
- **`dismissClip` resets an export that is `done` or `failed`.** It was the dismissed clip's; the next clip would otherwise show it.
- **The blockers are asked a second time after a wait:** in `importClip` after the old clip is dismissed, in `startExport` after the token is read from the database. Two clicks would otherwise both pass the first check.
- **`startExport` and `runPipeline` look at the clip id they are given.** `startExport` returns when it is not the clip in the store; `runPipeline` throws.
- **Every failed export goes the way of step 8:** `fail`, `export_failed`, `unlockAfterExport()`. The plan names the three for step 8 and only `fail(E_STORAGE_IO)` for steps 7 and 10, which would leave the preview locked.
- **The two sweeps remove the directory itself,** `clips/` and `exports/`. That removes every entry, and no path is built outside `opfs.ts`.
- **One wait for one download of the model,** shared by clips that overlap, and `model_download` is tracked when the download ends, also when the clip failed meanwhile. A download counts as having happened when the manager reported progress or failed.
- **After a failure of `load` or `transcribe`, `unload()` runs and its own failure is dropped.** The first failure is the one stored.
- **`stage_timing` for `asr` is sent for a clip without speech too.** The stage ran.
- **A feed line is made for an event of any kind.** §25.4 says a kind that V3 detects shows a line at once, and the plan gives `display` for a NumberReveal only. The other three are the `to` quantity of a FromTo, the count of a ListReveal and the word of a KeywordPop. V2 produces none of them; V3 should confirm them against the copy of Prompt 55.
- **A job that answers `{ cancelled: true }` is a failure with `E_INTERNAL`.** Nothing sends a cancel before V4.
- **Something thrown that is no failure of a worker, of the model or of the storage** is stored as `E_INTERNAL`, and thrown again: the page shows the stub, and a fault of the code still reaches the console.
- **`importSampleClip`:** a call made while another fetches gets the same promise; a body that breaks off counts as `offline`; on a failure, a clip that arrived meanwhile is left alone.

**Differs from the prompt, the guide or the plan.**

- **`git grep -n "V3: \|V7: " -- web/src/usecases | wc -l` is 5, not 4.** The four of this prompt are there: two `V3:` lines in `run-pipeline.ts`, one `V7:` line each in `import-clip.ts` and `start-export.ts`, each naming §25. The fifth is `// V7: restoreClip()` in `start-app.ts`, which V1 wrote. The guide's count did not include it (§27, item 31).
- **`start-app.ts` gained two lines,** the call and the import of the model manager; the prompt says one.
- **One dev server, started with the test key, served all four results.** The prompt starts a keyed server for the third only. The first, second and fourth do not depend on the key.
- **The dev server was `vite` alone,** not `pnpm dev`, as in Prompts 52 and 53.
- **The caps of TS §22.6 are a constant in two files,** `run-pipeline.ts` and `start-export.ts`: 3,600,000 ms and 10,000. A use-case may not import another, and the generated file holds only `MAX_EVENTS_PER_BATCH`.

**Checked.**

- `pnpm --filter web exec tsc --noEmit`; ESLint on `src/usecases`: clean. The three new files import `AppFailure` from `workers/protocol.ts` as a type (D-27 f); `import-clip.ts` imports `net/asset-fetch` (D-27 a) and `run-pipeline.ts` (D-58); `start-export.ts` imports `control-preview.ts` (D-58).
- `git grep -n "file\.name\|webkitRelativePath" -- web/src`: nothing, with the new files added.
- `node scripts/check-file-tree.mjs`: 225 files; the three new files were in the tree of TS §5.
- G M-9: `tsc`, ESLint on the whole of `web`, Vitest and `pnpm check` green; the frozen web files are unchanged. `grep -c "media.worker\|asr.worker\|render.worker" web/src/workers/pool.ts` prints 4, where the guide expects 3: the three rows, and the comment in line 6, whose words "render worker" the unescaped dot of the pattern also matches. With the dots escaped it prints 3. `pool.ts` was not touched, and the count was 4 before this prompt.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build` and `pnpm e2e` green; the frozen-file diff against `322c7d3` is empty; no file of `web/dist` holds the test key. **262 Rust** (2 ignored), **92 Vitest**, **12 Playwright**. The app shell is 286.8 kB gzipped, was 284.8: the model manager is now loaded at the start.
- **`pnpm e2e` failed three times straight after the build, and it was not this change.** One case of 12 timed out, then six, then two; each time the cases were among the six that start first, and they ran 25 to 37 s against a limit of 30 s. The suite was then run on a build without the change to `start-app.ts` and with it, in turns: without, three runs of which one failed the same way; with, three slow runs; then without and with twice more, and no case took longer than 17 s in any of those four. With 3 workers the changed build passed 12 of 12 in 15 s. The gate's own run, after a new `pnpm build`, passed 12 of 12 in 23.6 s with 6 workers. The machine had 3.5 to 5.7 GB free and two other browsers open; a model had been downloaded and three clips exported in the minutes before (known issue 20).
- Both temporary cases, the browser profile, the two downloads, the two cut clips and `web/test-results` are gone. `git status` shows the five files of this prompt and nothing else.

**"Done when".**

- [x] The four browser results hold; `git grep -n "V3: \|V7: " -- web/src/usecases | wc -l` is 5: the four of this prompt, and V1's marker in `start-app.ts`.
- [x] `git grep -n "file\.name\|webkitRelativePath" -- web/src` is empty; the three drills fired; G M-9 passes.

**Not checked.**

- **The download under the production CSP.** The check ran on the dev server, as the prompt says. The click on an anchor with a `blob:` address has not run under the CSP of `vercel.json`; Prompt 55's walk-through in a keyed production build is where it does.
- **A real page.** No component calls these functions yet. That a player's own `detach()` and `dismissClip()` get along was reasoned and tried with a canvas of the check, not with `PreviewPlayer`.
- **A download of the model that fails, or goes on from bytes on the device:** `model_download` with `failed`, `hash_mismatch` or `resumed: true`. A sample clip that cannot be fetched. A full disk, a lost GPU device, a worker that crashes.
- **By eye and by ear.** The exported files were verified, not watched.
- **INV-12 by a test.** `unload()` is awaited before `openSession` on every path; nothing in V2 asserts it (Prompt 57 notes the same).
- **R1** (D-69). Every time here is D1's, on a dev server, and stands for nothing in E-3 or E-4.

**Found, and not for this prompt.**

- **While the sample clip is fetched, no store says so.** The plan has the clip store go to `importing` only when the 35 MB are here, so the landing page stays as it is for that long, and a click on the button shows nothing. Prompt 55 wires the button and will meet it.
- **`assertNever` has no home.** `run-pipeline.ts` has one of its own; `ProcessingFeed.tsx` needs it twice in Prompt 55, and TS §11.3 asks for it in every such `switch`.
- **Model files that vanish behind the store's back** (the browser clears the site's data while the page is open) leave the model store at `ready` until a reload, and every import fails with `E_ASR_RUNTIME` until then.
- **The first six Playwright cases of a run are slow on D1 whenever the machine has been busy,** near or over the 30 s a case may take. `ci` runs them with a retry. Prompt 58 changes the Playwright config anyway.

**Open, for the human.** Push `v2-build` and read `ci` (open item 4): Prompt 55 waits for green.

## 2026-10-10 - The fourteenth push: `ci` green on Prompt 54

**Done by the human.** One push of `v2-build`, at `26129c3`, the commit of Prompt 54.

**Read by the agent** (the public API of GitHub).

| Read | Result |
|---|---|
| `origin/v2-build` | `26129c3`, equal to the local branch |
| The check runs on `26129c3` | `ci`: success. `deploy-api` and `deploy-web`: skipped, as on every pull request |

**Closes.** The line "Push `v2-build` and read `ci`" of the entry of Prompt 54. The number of the run and its time were not read.

**Changed.** This file only. The entry was written with the commit of Prompt 55.

## 2026-10-10 - Prompt 55: copy, components, pages, landing navigation

**The one path, on a page.** A clip dropped on `/` takes the visitor to `/app`, where the feed says "Transcribing…" and then "Found: $12k", and the player follows. The preview plays, pauses, goes on and offers "Play again" at its end. With no token the export button answers that exporting needs an account. In a production build made with the test key, with a seeded Creator token, the button ends in a downloaded MP4 of 1080 x 1920 that passes the verifier's six checks.

**Added.**

| File | Content |
|---|---|
| `web/src/ui/components/ProcessingFeed.tsx` | One line for each `FeedLine` of the clip store, in order; a `switch` over `kind` and one over `eventKind`, each ending in `assertNever`. 59 lines |
| `web/src/ui/components/PreviewPlayer.tsx` | The canvas of 540 x 960; `attach` when it appears and `detach` when it goes, one after the other; `pause()` when the tab hides; one button for play, pause and "play again", disabled while `detached`, `seeking` or `locked`; the failure stub when the preview failed. 118 lines |
| `web/src/ui/components/ExportButton.tsx` | The button; the words of the blocker of `forExport()`; `messages.export.unavailable`. No counter and no upgrade prompt. 52 lines |
| `web/src/ui/components/ExportProgress.tsx` | The stage, the bar (its width through `style.setProperty`), the elapsed seconds, the "rendering on your computer" line; the words for `done` and `failed`. 93 lines |

**Changed.**

| File | Change |
|---|---|
| `web/src/copy/messages.ts` | `feed`, `preview`, `export`, `editor`; `dropZone.fetchingSample`; `dropZone.notReady` is gone (D-56). 254 lines |
| `web/src/ui/components/DropZone.tsx` | A dropped file, the first of several, goes to `importClip(file, "user")`; the button to `importSampleClip()`; a click on the zone opens the file chooser; the words of a blocker; no `aria-disabled`. 120 lines |
| `web/src/ui/pages/LandingPage.tsx` | The prop `appPath`; `navigate(appPath)` when a clip arrives while the page is shown |
| `web/src/routes.tsx` | Passes `appPath` to `LandingPage` (G item 6c) |
| `web/src/ui/pages/EditorPage.tsx` | The body by the clip's status (the table of §20.1); "start over" calls `dismissClip()`. The placeholder and its `NotifyMeForm` are gone. 125 lines |
| `web/src/ui/styles/components.module.css`, `pages.module.css` | New classes only: `pending`, `feed`, `feedLine`, `player`, `previewCanvas`, `playerControls`, `exportPanel`, `exportProgress`; `editor`, `stub`, `problem`. Every value is a token of `tokens.css` |
| `web/tests-e2e/landing.spec.ts` | **REOPENED V1 CONTRACT (D-56).** The two cases of §23.5 take the place of "the drop zone is inactive" and "/app shows the not-ready panel". One locator of a third case changed (below) |
| `docs/v2/v2implementation.md` | §4 lists `routes.tsx` as changed; §20.1, §20.2 and §20.3 say what was built; §27 gains items 32 to 35 |
| `docs/v2/v2changelog.md` | This entry and the one of the push before it; known issue 38 |

No dependency was added. No use-case, no store, no worker and no Rust file changed.

**The three drills.** Each was undone from a copy, and ESLint is clean after the last.

| Temporary edit | Result |
|---|---|
| `<p>Rendering</p>` in `ExportProgress.tsx` | `react/jsx-no-literals`: "Strings not allowed in JSX files: "Rendering"" |
| `import { pool } from "../../workers/pool";` in `PreviewPlayer.tsx` | `boundaries/dependencies`: "There is no policy allowing dependencies from elements of type "ui" to elements of type "workers"" |
| `useClipStore.setState({ waitingModel: true });` in `ExportButton.tsx` | **Nothing fires:** ESLint and `tsc` pass. V1 has no rule for it, as the prompt expected; the gap is §27, item 34 |

**Browser check (dev), the rows of G 10.4.** `vite` on port 5173 with the bundles of the last build; Chrome under Playwright, headless, a new profile under `fixtures/.cache/`; a temporary case that drives the page as a person would and prints what it reads; the fake API of the E2E helpers. The feed was read with an observer of the page that keeps every state the list was in.

| Do | Read |
|---|---|
| Open `/`, drop the reference clip, no model on the device | The address becomes `/app`. The model panel says "about 210 MB"; its bar took 28 values from 0 to 100; its line went from "About 2 minutes left." to "About 1 second left."; then the panel went and the feed's first line was "Transcribing…". `model_download { outcome: "ok", duration_ms: 61700, resumed: false }` |
| The same with the model on the device | `/app`; no model panel at any time; the feed was "Transcribing…", then "Transcribing…" and "Found: $12k", and no other state; no "Cleaning voice" line; then the player |
| The canvas | 540 x 960, shown at 320 x 569 |
| Play; pause; play | Two pictures of the canvas 1 s apart differ; paused, two pictures 0.5 s apart are equal; playing again, they differ |
| The tab is hidden (the event sent by the test, known issue 33) | The preview pauses |
| Let it reach the end | The button says "Play again"; the preview store says `stopped`. One `preview_played` for the three plays |
| Export, with no token | "Exporting needs an account, and accounts are not open yet."; no download; no `export_started` |
| A text file named `.mp4`, dropped on `/app` | "This clip could not be used." and "Start over", nothing else: no code. "Start over" brings the drop zone back |
| The sample button | "Getting the sample clip…" while it is fetched; one request to the asset host, `GET /media/speech_scriptA_landscape_720p.7da948cecc39438a.mp4`; then `/app`, the feed and the player. `clip_accepted` with `source: "sample"` |

No console line of the kind error or warning and no page error in that run, which took 6.5 minutes.

**Browser check (production build with the test key), the rows of G 10.5.** `vite build` with `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` exported in that one terminal, the key worked out from the seed in `fake-api.ts` and printed nowhere; `vite preview` on port 4173, so the CSP of `vercel.json`; a new profile; a Creator token from `mintEntitlementToken()`, seeded with `seedEntitlement`.

| Do | Read |
|---|---|
| Drop the clip, wait for the player, play | The player; two pictures 1 s apart differ |
| Press Export while it plays | The play button and the export button are disabled; the button's line says "An export is running. Wait until it has finished." |
| The stage labels | "Rendering", "Writing the file", "Saving", in that order and no other |
| The bar | 101 values, 0 to 100 |
| "Everything is rendering on your computer." | In every state the progress panel was in |
| The download | `offcut-20261010-1248.mp4`, 78,965,863 bytes. `python verify/verify_mp4.py <file> --profile creator --expected-duration-ms 74705`: `PASS 1` to `PASS 6`, exit 0 |
| After it | "Your video is ready. Your browser has downloaded it." Play works: two pictures 1 s apart differ |
| The events | `export_started { profile: "creator" }`; `stage_timing` for `render_encode` (39,744 ms) and `mux` (22 ms); `export_done` with `total_ms: 39939`, `profile: "creator"`, `events_kept: 1`, `style: "clean"`; no `export_failed`. `asr` took 88,458 ms on WebGPU in that run |
| The console | No "Refused to" line, no `securitypolicyviolation` event, no line of the kind error or warning |

That closes the first line under "Not checked" of Prompt 54: the click on an anchor with a `blob:` address downloads under the production CSP.

**The landing suite on a browser that cannot run Offcut.** D1's Chrome can, so `pnpm e2e` takes one of the two ways through the two new cases. A temporary copy of the suite, in which the page reports itself as a phone before the app starts (`navigator.userAgentData.mobile`), took the other: the capability line was "Offcut does not work on phones and tablets yet.", the drop was refused with the words of `B_UNSUPPORTED`, the page stayed on `/`, and 12 of 12 cases passed. Two things that do **not** make this Chrome unsupported, tried first: Playwright's `isMobile` (`userAgentData.mobile` stays `false`) and the launch argument `--disable-blink-features=WebGPU`.

**Decided here, where the plan is silent.** Each is in §20 of the plan now.

- **A click on the drop zone opens the browser's file chooser,** and so do Enter and the space bar. The zone has the role of a button and can be reached with the keyboard; without this it would do nothing for a person who cannot drag. The chosen file goes the way of a dropped one, and its name is not read.
- **The words of a blocker are shown once a clip was offered.** While the browser is still being checked, `forImport()` already answers `B_UNSUPPORTED`; shown at once, the page would say "cannot run Offcut" for the second the check takes.
- **`LandingPage` moves on when a clip arrives while it is shown,** not whenever the clip is not `idle`. Otherwise a visitor with a clip in work who goes back to `/` would be sent to `/app` again, and could not get back.
- **The sample button says that it fetches.** The found issue of Prompt 54: 35 MB are fetched before any store changes. The button is marked busy and the zone shows `messages.dropZone.fetchingSample`, a key the table of §20.3 does not have.
- **One button for play, pause and "play again".** The plan says "play / pause / replay buttons"; at each moment one of the three is the one that can be pressed.
- **`attach` and `detach` run in turn.** A player that is taken away and put back at once would otherwise start a second attach while the first runs, and the second would find the preview attached already, which the preview machine refuses.
- **A failure of the preview is shown under the player,** with `messages.editor.failedStub`: `control-preview.ts` stores it on the clip, which stays `ready`, and the table of §20.1 has no place for it.

**Differs from the prompt, the guide or the plan.**

- **A third case of `landing.spec.ts` changed, by one locator.** "every request of a session goes to the page's own origin or to the asset host" waited on `/app` for the test id `not-ready`, which belonged to the placeholder that is gone. It waits for the drop zone or the unsupported page now. D-56 names two cases (§27, item 32).
- **The first new case asserts more than "navigates to `/app`":** on a supported browser the dropped text is no clip, so the page must show the rejection stub and "start over".
- **The dev server was `vite` alone,** not `pnpm dev`, as in Prompts 52 to 54.
- **G 10.5 builds with `pnpm build:wasm` first.** The bundles were built already and no Rust file changed: `vite build` alone.

**Checked.**

- `git grep -n "notReady\|zz-" -- web`: nothing. `git grep -n "never leaves\|GDPR\|DPDP\|CCPA\|SOC 2\|compliant" -- web/src/copy`: nothing. `node scripts/check-copy-codes.mjs`: 9 of 29 errors, 4 of 8 blockers, 38 pending, as before.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build` and `pnpm e2e` green; the frozen-file diff against `322c7d3` is empty; no file of `web/dist` holds the test key. **262 Rust** (2 ignored), **92 Vitest**, **12 Playwright** (23.5 s, 6 workers, 6.4 GB free). `check-file-tree`: 229 files. The app shell is 299.2 kB gzipped, was 286.8: the four use-cases and the new components are reached from the first page now.
- Both temporary cases, the temporary copy of the landing suite, the two temporary Playwright configurations, the two browser profiles, the screenshots and `web/test-results` are gone. The exported file is kept, outside git, as `testclips/renders/p55-creator-1080x1920.mp4`, for the founder to watch.

**"Done when".**

- [x] `git grep -n "notReady" -- web` is empty; `pnpm e2e` passes 12 cases with the two replaced ones.
- [ ] Both walk-throughs match every row; the exported file passes checks 1 to 6; G M-10 passes. **Every row an agent can read matches, the file passes, and the command block of G M-10 is green. The box waits for the founder's eyes and ears:** the row "Play" of G 10.4 (a 9:16 centre crop, captions word by word in the lower third, sound in sync, the reveal as the amount is spoken).
- [x] No banned phrase in the copy: the `git grep` above is empty.

**Not checked.**

- **By eye and by ear** (Human). A picture of the page while it played shows the centre crop and a caption in the lower third with one word in yellow; that the sound is in sync and that the reveal comes as the amount is spoken was not judged.
- **The first run of the dev walk-through was stopped after 13 minutes.** Its output went through a pipe and was not visible, so where it stood is not known. The second run, the one reported, printed as it went and took 6.5 minutes. Whether the first hung or was slow is open.
- **A failure of the preview on the page.** The stub under the player was not seen: nothing failed.
- **The four pending blockers and the three event kinds V2 does not detect** have no words or no line that a run could show.
- **R1** (D-69). Every time here is D1's.

**Found, and not for this prompt.**

- **A person can hardly read "Found: $12k".** The line appears when the events are detected and goes when the clip is `ready`, a moment later: 545 ms for `detect_scene` in the production run above, 18 ms in Prompt 54's. PS §10 J6 calls the feed what Offcut found. Whether the feed stays under the player is the founder's to decide (§27, item 33); the store keeps the lines.
- **`ready` has no "start over".** The table of §20.1 gives it to `rejected` and `failed` alone. After an export a visitor reaches the drop zone by a reload or by going back to `/`.
- **A FromTo's line would say "Found: $20k",** not "Found: $2k to $20k" as PS §10 J6 has it: `run-pipeline.ts` hands over the `to` quantity alone. V2 detects none; V3 must put both figures into `display`.
- **`assertNever` is in four files now** (§27, item 35).
- **The page is taller than a small window:** the player, the play button and the export button need about 760 px, and the export's progress is below that.

**Open, for the human.** Watch the preview and `testclips/renders/p55-creator-1080x1920.mp4` once. Push `v2-build` and read `ci` (open item 4).

## 2026-10-10 - Prompt 56: E2E helpers, setup project, `model-download.spec.ts`

**One command.** `pnpm e2e:media` fills `fixtures/.cache/` from the asset host when it has to, then runs the eight cases of the model download in a real Chrome, with the asset host answered from this machine. `pnpm e2e` is as it was: 12 cases, and it never fetches the model.

**Prompt 55 was not pushed when this began.** The founder asked for Prompts 55 and 56 in one go. `ci` has not run on either, and the "Human" steps of Prompt 55 are open.

**Added.**

| File | Content |
|---|---|
| `web/tests-e2e/media.setup.ts` | One Playwright setup test that calls `fillAssetCache()`, with 30 minutes for it. **New in the tree:** TS §5 and §4 of the plan have it (G item 6b). 16 lines |
| `web/tests-e2e/model-download.spec.ts` | The 8 cases of §23.6. 323 lines |

**Changed.**

| File | Change |
|---|---|
| `web/tests-e2e/helpers/fixtures.ts` | `routeAssets`, `dropClip`, `ensureModelCached`, `opfsList`, `sourceDurationMs`, `referenceClip`, `fillAssetCache`; also `modelManifest`, `assetBaseUrl`, `sampleClipPath`. `flushAnalytics` moves the clock 9,999 ms, was 11,000 (below). 426 lines |
| `web/playwright.config.ts` | Projects `media-setup`, `media` (the three suites, 300 s, one worker, `dependencies: ["media-setup"]`) and `bench` (`testDir: "../bench"`, headed, the same dependency); `non-media` gets `testMatch: "landing.spec.ts"`. No `globalSetup` |
| `package.json` | `e2e:media`, `e2e:device`, `bench:device` |
| `web/src/persistence/opfs.ts` | **A fault of Prompt 37, corrected:** a write that fails keeps its own error. `writeAt`, `truncate` and `writeAudio` go through one helper. 226 lines |
| `fixtures/speech/README.md` | The section of the V2 reference clip is marked final, with how it was confirmed; the span of the event; the expected feed |
| `docs/technicalspec.md` | §5: `web/tests-e2e/media.setup.ts` |
| `docs/v2/v2implementation.md` | §4: `media.setup.ts`, the projects of `playwright.config.ts`, the note on `opfs.ts`; §15.2, §22.2, §22.3, §23.4 and §23.6 say what was built; §27 gains items 36 to 38 |
| `docs/v2/v2changelog.md` | This entry; known issue 39; the cause in known issue 20 |

No dependency was added.

**The fault in `opfs.ts`, found by the quota case.** With the origin's quota set to the clip's size and 24 MiB, the page showed "Something went wrong on this device." and not the words of `E_MODEL_STORAGE`. The clip store, read in a dev build, held `E_STORAGE_IO`. A probe of Chrome on an empty page showed why:

| Where the room runs out | What Chrome throws | What `opfs.ts` made of it |
|---|---|---|
| In `createWritable({ keepExistingData: true })`, which copies the file first | `QuotaExceededError` | `quota`, right |
| In `write()` | `QuotaExceededError` from `write()`, and then `TypeError: Cannot close a ERRORED writable stream` from the `close()` in the `finally` | The `TypeError` took the place of the first error: `io`, wrong |

So which message a person with a full disk got depended on where in an 8 MiB part the room ran out, and the same held for `E_STORAGE_QUOTA` when the clip's audio is written. Now a change that fails gives up its stream with `abort()` and throws its own error. After the correction, in a dev build, quotas of the clip's size and 2, 3, 4 and 8 parts all gave `E_MODEL_STORAGE` and "There is no room for the speech model"; the one with 3 parts is the one that gave `E_STORAGE_IO` before. No test file was added for it (§23.1): the quota case is its test.

**Why `flushAnalytics` moves the clock less.** The case of the interrupted download failed once in a full run and passed 5 times of 5 alone: no `model_download` event arrived within 5 s of the flush. `flushAnalytics` moved the page's clock 11 s. The flush fires somewhere in that step; the request it makes gives up after 10 s of the same clock (`API_TIMEOUT_MS`); and when the flush fires in the first second of the step, the rest of the step carries the request to its limit before the browser has sent it. The batch is dropped, analytics does not send it again, and the fake API never sees it. A temporary case that jumps the clock to shortly before the first flush and then moves it showed this: with 11,000 ms the batch was lost in 2 runs of 11, both from the same position; with 9,999 ms it arrived in 12 of 12. A step under 10 s cannot reach the limit of a request it started, and a flush that is due in the millisecond left out fires by itself, because the clock runs on. **This is very likely the cause of the case "landing_view is sent once" failing after a build** (known issue 20, entries of Prompts 51 and 52): there the trace showed "the batch posted and not yet handed to the fake API", and a slow start of the app is what puts the flush into the first second of the step. `landing.spec.ts` itself was not touched.

**The two "must fail" checks.**

| Check | Result |
|---|---|
| `fixtures/.cache/` moved away, then `pnpm e2e` | 12 of 12 passed; the project `media-setup` did not run; the directory did not come back. `fixtures/.cache/` was put back afterwards |
| In `routeAssets`, one model request recorded as answered for `https://not-the-asset-host.example` (a temporary edit) | The "Requests" case failed: `expect(request.url).toBe("https://pub-….r2.dev/models/asr-en-v1/config.8825c4174cb86f94.json")`, received the other host. Undone from a copy |

The second check showed that the case leaned on the helper's own record alone. It now also listens to every request the page and its workers make, to any host, and asserts that the ones naming a model file are exactly the ones the asset host's route recorded.

**The runs.** Each is the whole project: the setup test and the 8 cases, one worker, headless Chrome on D1.

| Build | Result |
|---|---|
| With the test key, before the two corrections above | 8 of 8 in 2.9 min |
| Plain, before the correction of `flushAnalytics` | 7 of 8: the interrupted download, as described |
| Plain, final | 8 of 8, twice in a row, 2.9 min each |
| With the test key, final | 8 of 8 (the gate's run, below) |

The first run of all filled the cache from the asset host in 37.5 s; a run with the cache there spends 0.5 s on hashing it. The cases take 2 to 22 s each, and the last one, which transcribes the whole clip, about 1.1 minutes.

**The reference clip against its README.** A temporary case in a dev build put the model on the device with `ensureModelCached`, dropped the clip and read the clip store: 157 words; 17 sentences, each with the word range, the two times and the text of the README's table (0 of 17 differ); the six numbers; one event, `number_reveal`, words 132 and 133, 12,000 `usd`, `$12k`, span 62,660 to 63,540 ms, confidence 1; the feed `transcribing`, then `event_found` with `$12k`. The README is marked final on that reading.

**Decided here, where the plan is silent or cannot be built as written.** Each is in §22 and §23 of the plan now.

- **`ensureModelCached` writes the files, it does not download them.** The plan says it "runs one download into the context's OPFS"; a download needs a dropped clip and the whole app. The helper opens an empty page of the app's origin and writes the seven files under their plain names from `fixtures/.cache/` (known issue 21 asked for the cache, not the host).
- **`failAfterBytes` is a threshold.** A route cannot cut an answer in the middle, so the request that follows the threshold fails whole. With the 20 MiB of §23.6 three parts arrive and the `.part` holds 25,165,824 bytes.
- **"Retries exhausted" lets one part arrive first.** §23.6 answers 503 from the start and then expects a `.part`, which no byte would have made. `failAfterBytes` and `status` together: 8 MiB arrive, then every request gets 503.
- **A path the cache does not hold goes to the real host,** the demo video of the landing page for one. No case of this suite opens that page.
- **The media project has one worker.** A case loads the model and uses the GPU, and the cases of the landing suite already get slow on D1 when six run side by side (known issue 20).
- **"Loading" allows the page its own files.** From the first line of the feed to the player the page fetches the render worker's script and the two WASM bundles from its own origin. The case allows a `GET` under `/assets/` or `/ort/` and `POST /api/v1/events`, and nothing to another origin from the page's load on (§27, item 38).

**Differs from the prompt, the guide or the plan.**

- **`opfs.ts` and `flushAnalytics` changed,** which the prompt does not name. Both were faults that two of its cases met.
- **The quota case and the "Requests" case assert more than §23.6,** as said above.
- **A temporary `zz-*.spec.ts` matches no project any more,** because `non-media` has a `testMatch`. The SAB's browser checks need a temporary configuration from now on (known issue 39).
- **`fixtures/.cache/` holds a file under its path on the asset host** (`models/asr-en-v1/<stored name>`), not in one flat directory.

**Checked.**

- `pnpm --filter web exec tsc --noEmit -p tests-e2e/tsconfig.json`: clean. `pnpm --filter web exec playwright test --list --project media`: 9 tests in 2 files, no error of the configuration.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build` and `pnpm e2e` green; the frozen-file diff against `322c7d3` is empty; no file of `web/dist` holds the test key. **262 Rust** (2 ignored), **92 Vitest**, **12 Playwright**, and **8 media cases** on the keyed build before it. `check-file-tree`: 231 files. The app shell is 299.2 kB gzipped, as before.
- Every temporary case and configuration, the edit of the drill and `web/test-results` are gone. `fixtures/.cache/` holds the seven model files, 205 MB, outside git.

**"Done when".**

- [x] `pnpm e2e:media` passes 8 cases; `pnpm e2e` passes 12 and downloads no model.
- [x] `pnpm --filter web exec tsc --noEmit -p tests-e2e/tsconfig.json` is clean.

**Not checked.**

- **The Linux runner and the Windows runner.** Nothing of this ran in CI: the media job is Prompt 58's, and neither commit is pushed.
- **`E2E_REAL_ASSETS=1`.** The mode in which `routeAssets` only records was not run.
- **The reference clip from the cache.** On D1 `referenceClip()` is the file in `testclips/`; the branch that downloads the sample clip and checks its hash segment has not run.
- **`sourceDurationMs`.** No case of this suite calls it; Prompt 57's do.
- **Whether the correction of `flushAnalytics` ends the failures of "landing_view is sent once".** The reasoning says so and 12 runs of 12 agree; the case was not failing today, so nothing was seen to stop.
- **R1** (D-69).

**Found, and not for this prompt.**

- **Chrome leaves `<name>.crswap` behind** when `createWritable` fails for lack of room: its copy of the file, which counts against the quota. After such a failure the model's directory holds the `.part` and the `.crswap`, and nothing removes the second (§27, item 37).
- **A model failure of this kind keeps what arrived,** so "start over" and a new drop go on from the `.part`; with the disk still full it fails the same way. V4's retry and V7's `quota.ts` are where that is handled.
- **`page.clock.install()` lets time run on.** A case that installs the clock still sees timers fire by themselves; only `runFor` and `fastForward` add to it.

**Open, for the human.** Push `v2-build` (two commits, Prompts 55 and 56) and read `ci` (open item 4). Prompt 57 waits for green, and for the founder's look at the preview and the exported file of Prompt 55.

## 2026-10-10 - The fifteenth push: `ci` green on Prompts 55 and 56

**Done by the human.** One push of `v2-build`, at `0e9b33f`, the commit of Prompt 56, with Prompt 55 (`f97e81a`).

**Read by the agent** (the public API of GitHub).

| Read | Result |
|---|---|
| `origin/v2-build` | `0e9b33f`, equal to the local branch |
| The check runs on `0e9b33f` | `ci`: success. `deploy-api` and `deploy-web`: skipped, as on every pull request |
| The check runs on `f97e81a` | None: the two commits went up in one push, and the run on `0e9b33f` covers both |

**Closes.** The line "Push `v2-build` (two commits, Prompts 55 and 56) and read `ci`" of the entry of Prompt 56. The landing suite with its two new cases, and the new projects of the Playwright configuration, have now run on the Linux runner. The number of the run and its time were not read. Whether the founder has watched the preview and the exported file of Prompt 55 is not known to the agent: that box of Prompt 55 stays as it is.

**Changed.** This file only. The entry was written with the commit of D-71.

## 2026-10-10 - Outside a prompt: D-71, a clip that is ready can be started over

**The decision.** The founder's, on 2026-10-10, asked after Prompt 56: the page of a clip that is `ready` gets the "Start over" button too. It is D-71 in §2 of the plan.

**Why it was asked.** The table of §20.1 gave the button to a rejected and to a failed clip alone. Two things followed, both written down in the entry of Prompt 55. A person who has seen the preview, or made an export, could reach the drop zone only by a reload or by going back to `/`. And the "Start over" case of §23.7 would have had only a rejected clip to start over from, whose directory is removed before the button is there: it would have shown nothing.

**Built.**

| File | Change |
|---|---|
| `web/src/ui/pages/EditorPage.tsx` | One `StartOver` button, used by the stub and by the page of a `ready` clip, under the preview, the export button and its progress. It calls `dismissClip()`. It is disabled while `forImport()` names a blocker, which on this page is an export that runs: `dismissClip` does nothing then. 145 lines |
| `web/src/ui/styles/pages.module.css` | The class `startOver` |
| `docs/v2/v2implementation.md` | D-71 in §2; the row of `ready` in the table of §20.1, and the note under it |
| `docs/v2/coding-promptsv2.md`, `docs/v2/v2buildguide.md` | The range of decisions names D-71 |
| `docs/v2/v2changelog.md` | This entry and the one of the push before it; known issue 38 |

No copy was added: the button says `messages.editor.startOver`, as on the stub. No use-case and no store changed: `dismissClip` was written for a `ready` clip in Prompt 54, and tried there with a preview that plays.

**Checked.** `tsc` and ESLint are clean. The button was first pressed by a test: the "Start over" case of `pipeline-preview.spec.ts` (Prompt 57) presses it on a `ready` clip, after a preview that was played and paused, and reads the drop zone and an empty `clips/`; the "Progress" case of `export-creator.spec.ts` reads it disabled while an export runs and enabled after. The gate was run once, on the tree with Prompt 57 in it, and is in that entry: this commit, without the two suites, was not gated by itself.

**Not checked.** By eye: where the button stands under the export's progress on a small window (the page was already taller than 760 px, entry of Prompt 55).

## 2026-10-10 - Prompt 57: `pipeline-preview.spec.ts`, `export-creator.spec.ts`

**The one path, asserted.** `pnpm e2e:media` runs 30 cases: the 8 of the model download, 11 that take the reference clip from a drop to a preview that plays, and 11 that export it, with the verifier started from inside the suite on both files it downloads.

**Added.**

| File | Content |
|---|---|
| `web/tests-e2e/pipeline-preview.spec.ts` | The 11 cases of §23.7. 389 lines |
| `web/tests-e2e/export-creator.spec.ts` | The 11 cases of §23.8. 293 lines |

**Changed.**

| File | Change |
|---|---|
| `web/tests-e2e/helpers/fixtures.ts` | `verifyMp4(file, profile, expectedDurationMs)`, which starts `python verify/verify_mp4.py`, and `opfsSize(page, file)`. 470 lines |
| `web/tests-e2e/model-download.spec.ts` | Reads the size of the `.part` with `opfsSize`; nothing else |
| `docs/v2/v2implementation.md` | §23.7 and §23.8 say what was built |
| `docs/v2/v2changelog.md` | This entry; known issue 40 |

No file of the app changed in this prompt. No dependency was added.

**How the suites are built.** Each of the two processes the clip once. Its cases are one group that runs in order in one page, with the model put on the device by `ensureModelCached` before the page opens; a case that fails stops the ones after it. A case of its own for each row would take the clip through the pipeline 22 times, a minute and more each. "Sample clip" alone has a page of its own, and ends when `clip_accepted` has arrived. Neither suite installs the page's clock: the preview keeps time with the render worker by the page's own clock, and a moved clock would put them apart. An event is waited for until analytics sends it, at most 10 s.

| Suite | Order of its cases |
|---|---|
| `pipeline-preview` | Feed; real detections only; quiet processing; OPFS; long tasks; preview before sign-in; pause and resume; events; event shape; start over. Then, in a page of its own: sample clip |
| `export-creator` | No token; a token signed with another key; Creator export; progress; events; download name; storage; quiet export; preview after export; Free-plan token; second export |

**The runs.** Headless Chrome on D1, one worker.

| Run | Result |
|---|---|
| The two new suites, on a build with the test key, as first written | 22 of 22 in 5.0 min |
| All three, after a change to the export helper (below) | 22 passed, 1 failed, 8 did not run: "Creator export". A fault of the helper, not of the app |
| `export-creator` alone, the helper corrected | 11 of 11 in 3.8 min |
| **`pnpm e2e:media`, final, on a build with the test key** | **30 of 30 in 9.7 min** (31 with the setup test) |

The Creator export case takes about 45 s with the verifier, the Free one about 35 s. The longest task of the main thread while the clip was processed, written by the "long tasks" case and not asserted: 241, 250 and 226 ms in three runs, each time the one task of 50 ms or more.

**The fault in the helper, and what it showed.** To make a failed export say why, `exportAndSave` was changed to stop waiting for the download when the failure stub is on the page. But the stub of the export before, the one with the wrong key, stays on the page until the next export has read its token and begun: the helper saw that stub at once and reported a failure that had not happened. It now counts, with an observer of the page, how often a failure *comes* onto the page, and waits for one more. Worth knowing for V4: between the click on Export and the start of the new export the page still shows the last export's failure.

**The two checks of the prompt.**

| Check | Result |
|---|---|
| A build without the test key, then `export-creator.spec.ts` | "No token" and "a token signed with another key" pass; "Creator export" fails with `Error: the export failed: {"name":"export_failed","props":{"error_code":"E_ENTITLEMENT_INVALID","stage":"render_encode"}}`; the 8 cases after it do not run. Run twice, the second time with the corrected helper, on the plain build of the gate |
| `answered(await pool.asr.unload(), run.stage);` commented out in `run-pipeline.ts`, a build with the test key, then `pipeline-preview.spec.ts` | 11 of 11 pass. **Nothing in V2 asserts INV-12:** that the speech model's memory is given back before the render session opens is covered by review until V4's `run-pipeline.test.ts`. The line was restored from a copy; `git diff` of the file is empty |

**Decided here, where the plan is silent or cannot be built as written.** Each is in §23.7 and §23.8 of the plan now.

- **"Quiet processing" allows the page its own files,** as "Loading" of Prompt 56 does: from the drop to the player it fetches its workers, the two WASM bundles and the recognizer's runtime from its own origin (§27, item 38). **"Quiet export" allows nothing but `POST /api/v1/events`,** and holds: no file is fetched while an export runs.
- **"No property is a free string" is a table.** Each event the session may send is listed with what each property may be: a whole number, a boolean, or a member of a fixed list of words. An event or a property that is not in the table fails the case.
- **"No feed text in any request"** looks for "Transcribing…", "Found:" and the amounts of the README's expected feed in the address and the body of every request the page and its workers made.
- **"Neither is a single flat color"** is read from the picture's bytes without a library: a flat picture holds a handful of different byte values.
- **An exported file is saved outside `test-results`,** in a directory under the system's temporary one that is removed when the suite ends, so that no report can pick it up (§22.5).
- **"No token" waits 11 s** before it reads that no `export_started` came: longer than analytics takes to send.
- **"Second export" is the Free export.** It is the second export that succeeds, and its file's name is compared with the Creator export's.
- **The start-over case uses the button of D-71,** on the `ready` clip, after the preview was played and paused.

**Differs from the prompt, the guide or the plan.**

- **The cases of a suite are not independent tests.** §23.7 and §23.8 list rows; they do not say each has a clip of its own. On a retry in CI the whole group runs again.
- **Three `export_started` events, not one per successful export:** the refused export is announced as a Creator's, because `planOf` reads the plan from the token as it is (D-52). The "Events" case asserts the two that have been sent by then.
- **`verifyMp4` and `opfsSize` are in the helpers,** beside the functions §23.4 lists.

**Checked.**

- `pnpm --filter web exec tsc --noEmit -p tests-e2e/tsconfig.json` and ESLint on `tests-e2e`: clean.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build` and `pnpm e2e` green; the frozen-file diff against `322c7d3` is empty. **262 Rust** (2 ignored), **92 Vitest**, **12 Playwright**, and **30 media cases** on the keyed build before it. `check-file-tree`: 233 files. The app shell is 299.3 kB gzipped, was 299.2: the button of D-71.
- After the plain build, the search for the test public key in `web/dist` finds 0 files.
- `web/test-results` is gone; the directory of exported files under the system's temporary one is removed by the suite itself.

**"Done when".**

- [x] `pnpm e2e:media` passes 30 cases (8, 11, 11); no feed text appears in any request (the "event shape" case).
- [x] After the plain build, `grep -rl "<test public key>" web/dist | wc -l` is 0.

**Not checked.**

- **CI.** The media suites have run on D1 only. The Windows job is Prompt 58's.
- **The suites with `E2E_REAL_ASSETS=1`,** and headed (`pnpm e2e:device`).
- **A retry of a group.** No case failed in a way that made Playwright run a group again.
- **By eye and by ear.** The verifier and the pictures say the export is well formed and the preview moves; that the sound is in sync is still the founder's to judge (Prompt 55).
- **R1** (D-69).

**Found, and not for this prompt.**

- **The page shows the last export's failure until the next export begins** (above).
- **A report of a failed case can hold text of the page.** Playwright writes a snapshot of the page beside a failure, and a trace holds the page's text and pictures. Prompt 58 must turn traces off for the media project before a report is uploaded (§22.5, step 10.5).

**Open, for the human.** Nothing new. Prompt 58 follows in the same sitting, and its push is the one that counts.

## 2026-10-10 - Prompt 58: bench, media job in CI, test-key guard

**What it adds.** `pnpm bench:device` measures the stages of the reference clip on a machine, ten times. `ci.yml` builds the app the tests run against with the E2E test key, hands that build to a new job that runs the 30 media cases in Chrome on Windows, and refuses to deploy a build that holds the key. **The new job has never run:** its first run is this prompt's push, which is the founder's.

**Added.**

| File | Content |
|---|---|
| `bench/device-bench.ts` | One warm-up run and ten measured ones; `bench/results/<BENCH_DEVICE>-<yyyy-mm-dd>.json` with the fields of §23.10. 180 lines |
| `.github/workflows/e2e-media.yml` | Reusable, `windows-latest`, `defaults.run.shell: bash`; steps 10.1 to 10.5 of §22.5. 135 lines |

**Changed.**

| File | Change |
|---|---|
| `.github/workflows/ci.yml` | The test public key, once, in `env`; step 8 builds with it and uploads `web/dist` on every event; the job `e2e-media` (step 10); `deploy-api` waits for it; the Vercel step is three: build, guard, deploy. 448 lines |
| `web/playwright.config.ts` | The projects `media` and `bench` record no trace, and start Chrome with the arguments of `E2E_CHROME_ARGS` |
| `package.json` | `"type": "module"` (below) |
| `scripts/check-external-facts.mjs` | TE-10: the date the minutes were read; `checked` stays `null` until the job has run |
| `docs/v2/experiments.md` | TE-10: what is read, what is open, and what each outcome of the first run decides |
| `docs/v2/v2implementation.md` | §4, §22.3, §22.5 and §23.10 say what was built; §27 gains item 39 |
| `docs/v2/v2changelog.md` | This entry; open item 14; known issue 41 |

**Pinned.** `actions/setup-python` at `5fda3b95a4ea91299a34e894583c3862153e4b97`, the commit of v7.0.0, read from GitHub's API on 2026-10-10. `ffmpeg` 9.0.2, `ffmpeg-9.0.2-essentials_build.zip` of the builder whose full build D1 has, 114,768,076 bytes, SHA-256 `60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba`: the archive was downloaded and hashed here, and the hash is the one GitHub publishes for it. Python 3.13. The other actions are at the commits `ci.yml` already uses. No dependency of the app or of the tests was added.

**The bench, run once on D1.** `BENCH_DEVICE=dev pnpm bench:device` on a build with the test key, headed, 24.5 minutes. It is a check that the bench works, not a reading: the name is `dev`, and the file was deleted.

| Asked | Result |
|---|---|
| The line of G 11.3 | `10 probe_audio,asr,detect_scene,render_encode,mux 10 number` |
| `grep -c "clip.mp4\|twelve"` in the file | 0 |
| `device`, `clip`, `clip_duration_ms`, `asr_backend`, `chrome` | `dev`, `speech_scriptA_landscape_720p.mp4`, 74,705, `webgpu`, 155.0.8059.39 |
| `probe_audio`: median, p90 | 806 ms, 889 ms |
| `asr` | 83,948 ms, 85,671 ms (83,164 to 87,268) |
| `detect_scene` | 190 ms, 214 ms |
| `render_encode` | 35,443 ms, 35,624 ms (34,637 to 35,704) |
| `mux` | 17 ms, 23 ms |
| `total` | 120,778 ms, 122,647 ms |
| `peak_memory_bytes` | 936,182,945 |

The ten runs lie within 5% of each other. `asr` at 84 s is slower than the 60 s of Prompt 54's one import and inside the 62 to 96 s of Prompt 44; this was a headed browser doing eleven clips in a row. Prompt 59 takes the reading that counts, as `d1`.

**The guard, run here with the step's own text.** The shell of the step "Refuse a build that holds the test key" was taken out of `ci.yml` by a script and run with `bash --noprofile --norc -eo pipefail`, as GitHub runs a step, on a directory `.vercel/output` made from a build.

| The directory holds | The step |
|---|---|
| A build made with the test key | Fails, exit 1: "The build for production holds the E2E test key", and names the two files, `assets/index-….js` and `assets/render.worker-….js` |
| A plain build | Passes: "The test key is in none of the 11 files of .vercel/output." |
| Nothing | Fails: "vercel build wrote no file to .vercel/output, so there is nothing to search." |
| A plain build, and no key to search for | Fails: "E2E_TEST_PUBLIC_KEY is empty, so there is nothing to search for." |

The search of the prompt, `grep -rl "<test public key>" web/dist | wc -l`: 2 on the keyed build, 0 on the plain one.

**The workflows, read by tools.** Neither tool is installed on D1; each was taken as a release archive, checked against its published SHA-256, and kept outside the repository.

| Tool | Result |
|---|---|
| `actionlint` 1.7.12 on `ci.yml`, `e2e-media.yml`, `deploy-api.yml` | No finding |
| `gitleaks` 8.18.4, the version of CI, `detect --no-git` on a copy of the working tree | "no leaks found" |
| The same on `ci.yml` with the `gitleaks:allow` comment taken off | "no leaks found" too: the rule does not take this key's text for a secret. The comment stays, as known issue 8 asks: the scan reads the whole history, and a later version of the rule may differ |

**Decided here, where the plan is silent or cannot be built as written.** Each is in §22 and §23 of the plan now.

- **The key is written once, at the top of `ci.yml`, as `E2E_TEST_PUBLIC_KEY`.** The prompt writes it "literally in step 8". Step 14 needs the same text to search for; two copies could drift apart, and one name for both cannot. Under that name Vite does not read it, so the build that is deployed cannot pick it up from the workflow's `env`.
- **The root `package.json` says `"type": "module"`.** Without it Playwright took `bench/device-bench.ts` for CommonJS and stopped at "Cannot use 'import.meta' outside a module": the file is outside `web/`, as D-63 puts it, and imports helpers that are ES modules (§27, item 39). The root has no `.js` file and its scripts are `.mjs`; `node -e` with `require`, which `ci.yml` uses, works as before.
- **No trace for the media suites and the bench.** §22.5 forbids uploading a trace with transcript text. A trace of these suites holds the feed and frames of the clip, so none is made; then the report of a failed run can be uploaded.
- **`E2E_CHROME_ARGS`.** The arguments TE-10 may find necessary for WebGPU on the runner have a place: one line of `e2e-media.yml`. It is empty.
- **`e2e-media.yml` has no manual trigger.** Started by hand it would have no `web/dist`. If TE-10 ends in its fallback, the workflow gains what a manual run needs then.
- **The cache of `fixtures/.cache/` is keyed by two files,** the manifest and `net/asset-fetch.ts`, which names the sample clip. A key that is too old costs a download, never a wrong file: the setup test checks every hash.
- **TE-10's date.** `checked` of `check-external-facts.mjs` is the date an experiment was read. This one is half read: the minutes, on 2026-10-10. That date is in the entry's note, and `checked` stays `null` until the job has run.
- **The bench uses the browser context of its one test,** with the model put there by `ensureModelCached`, and a new page for each run. §23.10 says a persistent context; the model is on the device either way.

**Differs from the prompt, the guide or the plan.**

- **Step 9 runs the landing suite on the keyed build now,** because step 8 makes only that one. The suite does not look at the key.
- **`deploy-web` was split in three, not two:** build, guard, deploy. The header check and the smoke case follow as before.
- **The step numbers in `ci.yml`** follow TS §33: 10 is the media job, 11 the API, 12 to 15 the web app.

**Checked.**

- `pnpm --filter web exec tsc --noEmit -p tests-e2e/tsconfig.json`: clean, the bench in it.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build` and `pnpm e2e` green; the frozen-file diff against `322c7d3` is empty; no file of `web/dist` holds the test key. **262 Rust** (2 ignored), **92 Vitest**, **12 Playwright**, and before it **30 media cases** on the keyed build with the final configuration. `check-file-tree`: 235 files.
- `bench/results/` holds `.gitkeep` alone. The tools, the copies made for the scans and the guard, and `web/test-results` are gone or outside the repository.

**"Done when".**

- [x] The local chain is green with 30 media cases; no `dev-*.json` is left in `bench/results/`.
- [ ] The pull request is green including the Windows job, or the fallback is in the workflow and recorded; TE-10 has minutes and a date; G M-11 passes (Human). **Waits for the push.** What is done of it: the minutes of TE-10 are read and dated.

**Not checked.** All of it is what only a run on GitHub can show.

- **`e2e-media.yml` on a runner.** Not one step of it has run: `playwright install chrome` on Windows, the Python action, `unzip` and `cygpath` in the runner's bash, the hand-over of `web-dist` from the job `ci`, the cache.
- **WebGPU, the H.264 encoder and the AAC encoder on the runner** (TE-10). A runner without a graphics card may give a page no WebGPU; then every case stops at the capability check.
- **Time on the runner.** A case may take 300 s and the job 45 minutes. Transcription without a GPU took 98 to 123 s on D1 (Prompt 44); a hosted runner has fewer cores.
- **Steps 13 to 15.** They run on `main` only. That `vercel build` writes to `.vercel/output` in the directory the step runs in is what the guide says and V1's step relied on; the guard fails, and deploys nothing, if it finds no file there.
- **The secret scan in CI,** which reads the history and not a copy of the tree.

**Found, and not for this prompt.**

- **With a retry, a failed group runs again from the drop:** `retries: 1` in CI is one more pipeline for the suite that failed, four minutes or so on D1.
- **The report of a failed case can hold text of the page:** Playwright keeps a snapshot of the page beside a failure of a test that uses its own `page`. For the reference clip that text is what `fixtures/speech/README.md` already publishes.
- **`asr` is 84 s on D1 in the bench,** against the 25 s of E-3's first column and inside D-67's 180 s. Open item 10, the three-minute promise, stands.

**Open, for the human.** Push `v2-build` (three commits: D-71, Prompt 57, Prompt 58). Mark the pull request ready. Read every job, the new `e2e-media` among them, and its minutes (open item 14). Tell the agent what it says: the outcome fills TE-10, and a failure at the capability check names what the runner lacks.

## 2026-10-10 - The sixteenth push: the first run of `e2e-media`, which failed, and its correction

**Done by the human.** One push of `v2-build`, at `45dbea2`, the commit of Prompt 58, with D-71 (`674652f`) and Prompt 57 (`3316fbf`). The founder handed over the log of the step that failed.

**Read by the agent** (the public API of GitHub, run 38044970567, and the log the founder pasted).

| Job | Result |
|---|---|
| `ci` | Success, 3 min 54 s. Steps 1 to 9 on the Linux runner, the build with the test key and the upload of `web-dist` among them |
| `e2e-media / media` | **Failure,** 2 min 30 s |
| `deploy-api`, `deploy-web` | Skipped, as on every pull request |

**What the first run of `e2e-media.yml` showed.** Every step before the suites passed on `windows-latest`, each for the first time:

| Step | Result, time |
|---|---|
| Set up Node; pnpm; restore the pnpm cache; install the dependencies | Success, 31 s together |
| Install Chrome for Playwright | Success, 66 s |
| Set up Python; install what the verifier needs | Success, 27 s |
| Install ffmpeg (`curl`, the checksum, `unzip`, `cygpath`) | Success, 5 s |
| Take the build of the ci job | Success, 2 s: the artifact `web-dist` reaches the job |
| Restore the model files and the reference clip | Success, nothing to restore yet |
| Playwright, the media suites | **Failure after 4 s**, before any test ran |
| Keep the Playwright report of a failed run | Success |

**The failure.** `ffprobe` was asked for the length of `fixtures/.cache/media/speech_scriptA_landscape_720p.7da948cecc39438a.mp4`, which was not there: "No such file or directory", from `sourceDurationMs` at line 96 of `export-creator.spec.ts`.

**The cause, a fault of Prompt 57.** That line stood in the body of the group, `const durationMs = sourceDurationMs(referenceClip());`, so it ran when Playwright loaded the file to list its tests: before the setup project, which is what downloads the clip on a machine without `testclips/`. On D1 the clip is in `testclips/`, so the line always found it. This is the branch the entry of Prompt 56 named under "Not checked": "the reference clip from the cache ... has not run". It had not, and it was broken.

**Corrected.**

| File | Change |
|---|---|
| `web/tests-e2e/export-creator.spec.ts` | `durationMs` is read in the group's `beforeAll`, when the setup project has run. 296 lines |
| `docs/v2/experiments.md` | TE-10: what the first run showed |
| `docs/v2/v2changelog.md` | This entry; open item 14; a line in known issue 40 |

No other place reads the clip while a file is loaded: `pipeline-preview.spec.ts`, `model-download.spec.ts` and the bench read it inside a test.

**Checked, in the runner's situation.** On D1, with the file in `testclips/` moved away and no clip in `fixtures/.cache/`, so that `referenceClip()` answers the path in the cache as on the runner:

| Asked | Result |
|---|---|
| `playwright test --list --project media`, the file as it was pushed | The same error as on the runner, word for word but for the directory |
| The same, the file corrected | "Total: 31 tests in 4 files" |
| `CI=true pnpm e2e:media`, on a build with the test key | The setup test downloaded the clip from the asset host and checked its hash segment, 6.5 s, 35,201,023 bytes; then **30 of 30 in 6.8 min**, every suite with the clip from the cache |

The file in `testclips/` was put back, 35,201,023 bytes. That run is also the first of `fillAssetCache()`'s branch for the sample clip, of `referenceClip()` from the cache, and of the suites with `CI` set (one retry, the HTML report).

- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build` and `pnpm e2e` green; the frozen-file diff against `322c7d3` is empty; no file of `web/dist` holds the test key. **262 Rust** (2 ignored), **92 Vitest**, **12 Playwright**.

**Still not known.** What the entry of Prompt 58 listed, less what this run showed: whether Chrome on the runner gives a page WebGPU, whether it has the two encoders, and how long a clip takes there. The suites did not get as far as opening a page.

**Open, for the human.** Push `v2-build` (one commit) and read the job `e2e-media` again (open item 14). If it fails, the first failing case and its message are what the agent needs.
