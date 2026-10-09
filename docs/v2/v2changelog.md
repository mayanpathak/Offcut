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
| 3 | R1, a 2021-class Windows laptop with 8 GB and an integrated GPU: book it for the days of Prompts 51 and 59 (Prompt 44 takes no timing on it since D-67), and set it up once (G 0.6: Node, pnpm, Chrome, Python 3 with NumPy, `ffmpeg`; a clone; the reference clip copied by hand). Boxes 13 and 14 of §1A stay open until then. Both gates are read on R1 and on no other machine | Human | Prompt 44 |
| 4 | After every prompt marked **Push**: `git push` from `offcut/` (the repository is not the `sh2clips` folder), then read the `ci` run of pull request #2. The next prompt waits for green | Human | Prompts 35, 37, 41, 44, ... |
| 5 | Submit the merchant onboarding (TE-9) if it is not submitted. Approval can take two weeks | Human | V6 |
| 6 | Reset the Neon password (V1 item 36) and replace the demo video (V1 item 33) | Human | Before the page is announced |
| 7 | The file `.env.local` in the repository root holds one line with no name, a test-mode API key. Git ignores the file and no program reads that line. Move it into `.env.deploy` under a name | Human | Any time |
| 8 | Turn on branch protection for `main` on GitHub (Settings, Branches): require a pull request and the `ci` check. `main` is unprotected, and a push to it deploys | Human | Before Prompt 59 |
| 9 | Read the license of the speech model before the page is announced. Offcut now serves the model files from its own asset host. The repository they were taken from states no license of its own; the model's author says the code and the weights are under the MIT License, which asks for the notice to go with copies | Human | Before the page is announced |
| 10 | **The three-minute promise.** With the small-size model the median total for a 60 s clip on R1 is about 221 s on the numbers of D-67, and PS §20.2, §20.3, J10 and the pitch say three minutes. Change the promise or the model before the page says it to a visitor. The copy that states the time is not written yet | Human | Before the page is announced |
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
| 8 | **The secret scan of CI (gitleaks 8.18.4) takes a 44-character base64 literal assigned to a name with `KEY` in it for a secret.** Found in Prompt 33 with the same version run locally. Such a line must end with a `gitleaks:allow` comment **in the commit that first adds it**: the scan reads the whole history, so a comment added in a later commit does not clear the finding. The test public key (`fake-api.ts` in Prompt 46, `ci.yml` in Prompt 58) is such a literal. Before a commit that adds one, fetch the release archive of that version, check its SHA-256 against the published checksum file, and run `gitleaks detect --no-git --source <folder> --redact` | Prompts 46, 58 |
| 9 | Chrome under Playwright, headless, gives a module worker WebGPU, both encoders and a sync OPFS handle on the development machine (Prompt 33). The adapter is the integrated Intel GPU, not the GTX 1650. No headed run and no launch argument was needed | Every browser check |
| 10 | **`crates/offcut-mp4/src/boxes.rs` has 392 lines above its test module, `sample_table.rs` 387 and `demux.rs` 369; the limit is 400** (`cargo fmt` puts most signatures and struct literals on several lines). Code that must go into one of them needs room made first. The muxer has its own two files | Prompts 41, 48 |
| 11 | **What Prompt 37 builds on** (entries of Prompts 35 and 36). In a worker: `const core = await loadCore()`; `core.openDemuxer(syncHandle)` returns a `CoreDemuxer` or `{ rejected }` (test with `"rejected" in result`); `demuxer.probe(size)` gives the `ProbeInfo` whose `video.codec_string` and `demuxer.videoDescription()` go into `VideoDecoder.isConfigSupported`; `core.probeAndValidate(demuxer, size, supported)` gives `{ ok }` or `{ rejected }`. Audio: `audioDescription()`, `audioSampleCount()`, `readAudioSample(i)` with `ptsUs` (may be negative) and `durationUs`. A failure is thrown as the plain object `{ code, detail }`, not an `Error`. Call `demuxer.free()` when done, before closing the handle. `core.resample(pcm, from, to)` returns exactly `round(len x to / from)` samples. `offcut-mp4` also has a test-only module, `demux::fixture`, that builds small MP4 files | Prompts 37, 41 |
| 12 | `proptest` writes a folder `proptest-regressions/` beside the crate when a property fails. It is not in the tree of TS §5: delete it once the failure is fixed, or the file-tree check fails | Prompts 35, 36, 41 |
| 13 | **How code in a worker is written** (entry of Prompt 37). A failure with a name is thrown as `new WorkerFailure(code, detail, stage?)` from `workers/rpc.ts`: ESLint refuses a thrown plain object, and `toAppFailure` reads the error's `code`. A handler that saw its cancel flag returns `CANCELLED`. `ctx.isCancelled` and `ctx.progress` are passed on as `() => ctx.isCancelled()`, never unbound. A worker posts with `postMessage(message, { transfer })`; `serveWorker` does it and finds the `Float32Array`s and `OffscreenCanvas`es of a result itself. The lists `oneWay` and `duringPreview` are given to `serveWorker` in the worker's entry file: a worker may not import `pool.ts` | Prompts 43, 50, 53 |
| 14 | **A new worker gets one line in `pool.ts`:** `import url from "./<name>.worker.ts?worker&url"`, a row in `WORKERS` (script, stage) and a client in `pool`. That form gives the built script's URL, which the `modulepreload` link and `new Worker` both use; `new URL("./x.worker.ts", import.meta.url)` outside `new Worker(...)` would ship the TypeScript source as an asset | Prompts 43, 50 |
| 15 | **`offcut-wasm-render` needs the same read-ahead window in its own `JsRandomAccess`** (D-30 gives each binding crate its own): one call into the browser per video sample costs about 0.4 ms. The one in `offcut-wasm-core/src/media_api.rs` is the model | Prompt 48 |
| 16 | `web/src/workers/rpc.ts` has 374 lines, of which 45 are the two tables. V4 adds the progress throttle, the cancel timeout and the restart to this file and has 26 lines for them before the limit of 400 | V4 |
| 17 | A temporary Playwright case that waits for `networkidle` can hang: the start page streams the demo video from the asset host. Wait for what the case needs instead | Every browser check |
| 18 | **The model on the asset host runs on both backends** (entry of Prompt 43): the 214,647,815-byte set of the small-size English model, a 4-bit encoder and a 4-bit decoder with 16-bit floats. The runtime asks for its seven files and for nothing else. **It is slow:** on the development machine `load` and `transcribe` took 65 s on WebGPU and 102 s on WASM for the 74.7 s reference clip, against a target of 25 s on R1 (E-3). Since D-67 the fallback line is 180 s, and the founder's reading on an R1-class laptop is about 150 s. If a later reading on R1 is over 180 s, D-66 names the base-size model: steps 2 and 3 of Prompt 38 again with its files (the 4-bit pair is 145,199,758 bytes), `MODEL_DTYPE` in `whisper-runtime.ts` to match their names, and the transcript of `fixtures/speech/README.md` taken again. The files of the small-size set are in `C:\Users\Mayan\offcut-models\asr-en-v1`, outside the repository. In a production build (entry of Prompt 44) the same machine read 62 s to 96 s on WebGPU and 98 s to 123 s on WASM | Prompt 59 |
| 19 | **What Prompts 39, 40 and 43 build on** (entry of Prompt 38). `fetchAsset(path, { range?, signal })` returns `{ ok: true, status: 200 \| 206, response }` or `{ ok: false, cause, status? }` and reads no body. On the asset host: a range that ends past the end of a file answers 206 with the bytes that exist, and `Content-Range` gives the real last byte and the total; a path that does not exist answers 404 with the CORS headers, so it arrives as `cause: "status"`, not `"offline"`; with no `Range` header the answer is 200. The manifest lists seven files, the two large ones second and third; a stored name is `<stem>.<16 hex>.<extension>`. The plain names of the two large files, `encoder_model_q4.onnx` and `decoder_model_merged_q4f16.onnx`, are the ones the runtime is expected to ask for when each half is given its precision (4-bit; 4-bit with 16-bit floats). Not confirmed before Prompt 43 | Prompts 39, 40, 43 |
| 20 | **`pnpm e2e` can fail on the development machine when it is short of memory** (entry of Prompt 39). Playwright starts 6 browsers at once; with about 3 GB free the six cases that start first time out in the capability check of the start page, and the other six pass. `pnpm --filter web exec playwright test --project=non-media --workers=3` passes. Before reading such a failure as a fault of the code, close other browsers and run again, or run with fewer workers; the `ci` run is the check on a clean machine. `playwright.config.ts` was not changed. The gate of Prompt 40, an hour later and with 6 workers, passed 12 of 12. It came back once more, in the gate of Prompt 41, right after the workspace had been compiled (one case, the longest, timed out twice), and was gone in the gate of Prompt 42 | Every gate |
| 22 | **What the later prompts build on** (entries of Prompts 41 and 42). **Muxer:** `Mp4Muxer::new(sink, video, audio)`, `add_video_sample`, `add_audio_sample`, `finalize`; a sink may be owned or lent (`&mut sink`); the samples of the two tracks may come in any order between each other, and the `moov` of a 90 s clip written in turns took 100 kB of the 256 KiB kept for it (32 kB written one track after the other). `mux.rs` has 355 lines and `mux_boxes.rs` 263; V5 adds `ctts`. **Text:** `core.normalizeTranscript(raw, modelId)` takes `{ text, startMs, endMs, confidence }[]` with **whole milliseconds** (1.5 is refused) and returns a plain `Transcript`. In the browser an extra field of a raw word is ignored, not refused. A confidence comes back as a 32-bit value (0.98 reads 0.9800000190734863), which matters to anything that compares it with 0.80 exactly. `offcut_text::tokenize(words, edits)` and `parse_quantity(tokens)` are what `offcut-detect` reads; `format_quantity(value, &unit)` is the one formatter `offcut-scene` may call. The `dollars` form of D-42 is not in: Prompt 43 adds it, with its row of §8.5, only if the recognizer writes the amount without a `$`. A lone cardinal in words is a quantity ("one" is 1): Prompt 45 decides how captions show it | Prompts 43, 45, 46, 48, 50 |
| 23 | **What Prompt 44 and the later prompts build on** (entry of Prompt 43). `pool.asr.load({ modelId, backend })` answers `{ backend }`; `pool.asr.transcribe({ pcm16 }, { transfer: [pcm16.buffer], onProgress })` answers `{ ok: Transcript }` or `{ rejected: "NoSpeech" }`; `pool.asr.unload()` answers when the sessions are disposed. The pool has two rows. **`confidence` is 1 for every word, and stays so in V2:** TE-2 (entry of Prompt 44) found that the runtime returns no probability. No score is lowered by it; a prompt that reads `confidence` reads 1. **The dev server reloads the page once** the first time a browser loads the ASR worker after an install (Vite prepares the runtime): a temporary Playwright case fails with "Execution context was destroyed" and passes when run again. `whisper-runtime.ts` sets the runtime up so that it cannot make a request; the 13 literals it brings into the build are listed in `scripts/check-hosts.mjs`, each for one file, and **a new version of the runtime may bring others**: `pnpm build` then fails until each has an entry with its reason. The amount of the reference clip is words 132 and 133, `$12` and `,000`; the expected transcript is in `fixtures/speech/README.md` | Prompts 44, 45, 46, 54, 56, 57 |
| 24 | **A first transcription may be slower than the next** (entry of Prompt 44): in a new browser profile, straight after a build and the download of the model, `load` and `transcribe` took 95.5 s on WebGPU and 123.4 s on WASM; the second run took 61.9 s and 98.1 s. One pair of readings, cause not found. The bench of Prompt 59 should keep its first run apart, and E-3 at S15 should say which it reports. **Chrome makes requests of its own** (an update check, a push-message registration) while a page is open: a check that reads the browser's whole network log, and not the page's requests, sees them and must tell them apart by who started them | Prompts 57, 59 |
| 21 | **What the later prompts build on** (entries of Prompts 39 and 40). `inspect()` answers `absent`, `partial` or `ready` and tells the store on its first call; `start-app.ts` calls it as step 8 (Prompt 54). `ensureReady(onProgress, signal)` resolves on `ready` and rejects with a `ModelFailure`, whose `failure` is the `AppFailure` (`stage: "model"`), or with the abort itself when the signal aborted; a use-case imports `ModelFailure` from `models/model-manager.ts`. `useModelStore` holds `{ status, done, total, etaSecs, error? }`; its actions are called by the model manager only. `<ModelDownloadPanel />` takes no props and reads the store; `EditorPage` mounts it (Prompt 55). In OPFS the model is seven files with plain names under `models/asr-en-v1/`, which is where the cache adapter of Prompt 43 reads them. A cold download took 63 s on the development machine's line; the E2E helper `ensureModelCached` (Prompt 56) must fill OPFS from `fixtures/.cache/`, not from the asset host. In a test, a `Bytes` cannot be made by a cast: `download.test.ts` shows one way to get typed values | Prompts 43, 52, 54, 55, 56 |

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
