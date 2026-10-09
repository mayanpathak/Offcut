# experiments.md - Offcut V2: experiment records

**What this is.** The outcome of every experiment V2 runs: the technical experiments TE-n of `technicalspec.md` §37 and the product experiments E-n of `product.md` §19. The procedures and thresholds are in `v2implementation.md` §24.1. The M0 gate decision (§24.3) is written at the end of this file.

**How it is kept.** One entry per experiment, in the order they are run. An entry has five parts: the question, the method, the numbers, the date, the decision. A reading is recorded as measured and also normalised to 60 s (measured x 60,000 / 74,705), because the reference clip is 74.7 s long (D-64). R1 is the only reference machine (D-65); a reading taken on the development machine is labelled `dev` and never stands for E-3 or E-4.

| Experiment | Prompt | State |
|---|---|---|
| TE-7 re-check (ranged requests to the asset host, with the real model files) | 38 | Run on 2026-10-09: passed |
| TE-1 (no request outside the closed host list during transcription) | 44 | Run on 2026-10-09: passed, on WebGPU and on WASM |
| TE-2 (word probabilities at no more than 10% extra time) | 44 | Run on 2026-10-09: not passed, the runtime returns no probability. Fallback taken: `confidence` is 1.0 for every word |
| E-3 (transcription time on R1) | 44, 59 | S6: the founder's reading of 2026-10-09, about 150 s, accepted (D-67). S15: not run |
| TE-3 (frame capture method; render-only speed) | 51 | Not run |
| TE-4 (encoder ladder entry; AAC priming) | 51 | Not run |
| E-4 (render and encode time on R1) | 51, 59 | Not run |
| TE-10 (the Windows media job in CI: does it run, and its minutes) | 58 | Not run |
| TE-14 (memory on a 90 s 1080p60 clip, ten runs) | 59 | Not run |
| E-1 (waitlist: visitors and joins) | 60 | Not run |
| M0 gate decision | 60 | Not taken |

---

## TE-7 re-check: ranged requests to the asset host, with the real model files

**Question.** Does the asset host serve the real model files by byte range, to a page on another origin, with the headers the downloader reads and a cache header that lets a file be kept for ever (§24.1)?

**Method.** The seven files of the model (D-66: the small-size English model, 214,647,815 bytes) and the sample clip were uploaded with `scripts/upload-assets.sh`. Then, on the largest file, `models/asr-en-v1/decoder_model_merged_q4f16.0d38a3ab3d034990.onnx` (145,776,485 bytes): the two `curl` lines of G 4.2 with `Origin: http://localhost:4173`. Then, from the dev page in Chrome (`http://localhost:5173`), `fetchAsset` on four of the uploaded files, with every request header read through the DevTools protocol.

**Numbers.**

| Asked | Answer |
|---|---|
| `Range: bytes=0-8388607` on the largest file, `curl` | 206, 8,388,608 bytes |
| The last 1,024 bytes of the largest file, `curl` | 206, 1,024 bytes |
| `HEAD` on the largest file | `Cache-Control: public, max-age=31536000, immutable`; `Accept-Ranges: bytes`; `Access-Control-Expose-Headers: Accept-Ranges,Content-Length,Content-Range`; `Content-Type: application/octet-stream`; `Content-Length: 145776485` |
| The same range from the dev page | 206, 8,388,608 bytes, `Content-Range: bytes 0-8388607/145776485` readable by the page |
| `Range: bytes=0-1023` on the sample clip, from the dev page | 206, 1,024 bytes, `Content-Range: bytes 0-1023/35201023` |
| `Range: bytes=0-8388607` on a file of 2,405,679 bytes, from the dev page | 206 with the whole file, `Content-Range: bytes 0-2405678/2405679` |
| A file with no `Range` header, from the dev page | 200, 339 bytes |
| A path that does not exist, from the dev page | 404, readable by the page: not a network failure |
| Every uploaded file, `HEAD` | 200, with the length of the file on disk; `application/json` for the five configuration files, `video/mp4` for the clip |
| Two files fetched whole and hashed (2,405,679 and 66,178,491 bytes) | Each SHA-256 equals the one in `model-manifest.json` |

**Date.** 2026-10-09, from the development machine.

**Decision.** Passed. The asset host stays; no file is split. The largest single file is under the 150 MB that V1's TE-7 tested, so its record needs no correction (known issue 18 of `v2changelog.md` is closed).

**Not read.** The billing page of the host: whether an egress charge appears is the human's to look at. The host's terms say egress is free (V1's TE-7).

## E-3, at S6: transcription time on R1 (the founder's reading)

**Question.** Does local transcription of the reference clip stay inside its time budget on R1 (PS §19, §24.1)?

**Method.** Not the method of §24.1. The founder gave one reading on 2026-10-09 and decided that it stands for E-3 at this step (D-67). What the founder reported: `load` and `transcribe` of the reference clip on an R1-class laptop, 8 GB of memory and an integrated GPU. Not recorded: the build that was run, the backend (WebGPU or WASM), the version of Chrome, the number of runs. The procedure of §24.1 asks for the median of five runs with those four things written down; none of that was done, by the founder's choice.

**Numbers.**

| Read | Value |
|---|---|
| `load` + `transcribe`, reference clip (74,705 ms), R1-class laptop, as reported | about 150,000 ms |
| Normalised to 60 s (x 60,000 / 74,705) | about 120,500 ms |
| The target (D-64) | 25,000 ms: missed |
| The line above which the fallback applies | 180,000 ms since D-67 (it was 50,000 ms): not reached |

For comparison, `dev` readings of Prompt 43, which stand for nothing here: 65,000 ms on WebGPU and 102,000 ms on WASM with four threads, on the development machine.

**Date.** Reported on 2026-10-09.

**Decision.** Continue with the small-size model (D-66) and record the miss: the reading falls in the middle column of §24.3 as D-67 redrew it. The base-size model stays the fallback above 180,000 ms. Prompt 44 takes no timing on R1. The reading of S15 (`pnpm bench:device` on R1, ten runs, Prompt 59) is still to be taken and is judged against the same line.

**What this reading does not show.** That the time is the same on another R1-class machine, or on a second run; which backend a user's browser will get; anything about the quality of the transcript, which no experiment has compared between the two model sizes.

## TE-1: no request outside the closed host list during transcription

**Question.** In a production build under the production CSP, with the model in OPFS, does the speech runtime load and transcribe the reference clip without a request to any host but the app's own, on WebGPU and on WASM (§24.1)?

**Method.** The code of Prompt 43 (`98e9238`) with the temporary hook of G 5.5, built with `pnpm build:wasm` and `vite build`, served by `vite preview` at `http://localhost:4173`, which sends the headers of `vercel.json`. Chrome 155.0.8059.39, headless, driven by a temporary Playwright case that was deleted afterwards. For each backend a new page: open the start page, make the model ready with `modelManager.ensureReady`, then `importAndProbe`, `extractAudio`, `pool.asr.load`, `transcribe`, `unload`. Recorded: every request of the page and of each of its workers (the DevTools protocol, through Playwright's request events of the browser context), every console line and page error, the workers that were started. The clip was given to the page by the test and `POST /api/v1/events` was answered by the test, so neither reached the network. Two runs. The first started with an empty browser profile, so it also downloaded the model. The second kept the profile and added a second record that does not depend on the first: Chrome's own network log of the whole browser process (`--log-net-log`), read for the two time windows of `load` to `unload`.

**Numbers.**

| Read | WebGPU | WASM |
|---|---|---|
| Headers of the page | The CSP of `vercel.json`; `Cross-Origin-Opener-Policy: same-origin`; `Cross-Origin-Embedder-Policy: require-corp`; `crossOriginIsolated` is `true` | The same |
| Backend the worker reports | `webgpu` | `wasm`, 4 threads (`hardwareConcurrency` is 12) |
| Requests while loading and transcribing, run 1 | 7, all to `localhost:4173` | 8, all to `localhost:4173` |
| Requests while loading and transcribing, run 2 | 8, all to `localhost:4173` | 8, all to `localhost:4173` |
| Console lines and page errors, both runs | 0; none starts "Refused to" | 0 |
| Words, sentences | 157, 17 | 157, 17 |
| The text, word for word, against `fixtures/speech/README.md` | Equal | Equal |
| The amount | Words 132 and 133, `$12` and `,000`: 12,000 `usd`, display `$12k` | The same |
| `load`, `transcribe`, run 1 (`dev`) | 9,573 ms, 85,919 ms | 10,699 ms, 112,667 ms |
| `load`, `transcribe`, run 2 (`dev`) | 4,380 ms, 57,496 ms | 4,688 ms, 93,434 ms |

What the requests were, the same on both backends: the script of `media.worker` and of `asr.worker`; `offcut_core_bg.wasm`, once for each of the two workers; from `/ort/1.31.0-dev.20260914-8d85527a0/`, `ort-wasm-simd-threaded.asyncify.mjs` and `ort-wasm-simd-threaded.asyncify.wasm`; the clip the test handed over; and, in three of the four readings, one `POST /api/v1/events`, the app's own event, the one request the plan allows during transcription. Workers started: `media.worker`, `asr.worker`, and three thread workers of ONNX Runtime from the same `.mjs` file on the app's origin; none from a `blob:` address.

The model, run 1: `inspect()` answered `absent`; `ensureReady` took 70.8 s and made 31 requests to the asset host and 1 to the app's origin (the run did not list which). On every later page `inspect()` answered `ready` and the model phase made no request. The runtime had its files from OPFS: no request for a model file is in any window.

The second record, Chrome's network log of run 2 (46 requests in the whole log):

| Started by | To | Count | When |
|---|---|---|---|
| The app's origin | `localhost:4173` | 28 | 9 inside each of the two windows: the list above without the two the test answered, and the `.mjs` file four times (the worker and its three threads) |
| The app's origin | The asset host, `media/demo.<hash>.mp4` | 6 | When the start page opened; none inside a window. It is the demo video of the start page, a host of the closed list |
| The browser itself (the log says "not an origin"), the two navigations | `localhost:4173` | 2 | When each page opened |
| The browser itself ("not an origin") | Five hosts of Google, among them the account list, an update check, form filling and the push-message registration | 10 | 3 of them inside the windows: 2 on WebGPU, 1 on WASM |

**Date.** 2026-10-09, on the development machine. Under headless Chrome its WebGPU adapter is the integrated one (known issue 9 of `v2changelog.md`).

**Decision.** Passed. Every request that the page or one of its workers made while the model loaded and the clip was transcribed went to the app's own origin; no CSP refusal; both backends ran and gave the same transcript. The runtime stays: `@huggingface/transformers` 4.3.1 behind `whisper-runtime.ts`, with the options of the entry of Prompt 43 in `v2changelog.md`. No CSP host was added and the D-38 list of `scripts/check-hosts.mjs` did not change.

**What this does not show.** The ten requests to Google's hosts are Chrome's own: the log marks each as started by no page, they are not in the DevTools record of the page and its workers, and the CSP does not govern them. A user's Chrome makes them with any page open. They are not requests of Offcut, and this experiment does not claim that a browser running Offcut is silent. The timings are `dev` readings: they stand for nothing in E-3. Not tested: a browser with no WebGPU adapter (the fall from `webgpu` to `wasm` inside `loadModel` did not happen here), Edge, and a deployed build on the real host.

**Seen and not explained.** The first transcription in the new profile was slower than the second on both backends: 95.5 s against 61.9 s on WebGPU, 123.4 s against 98.1 s on WASM, `load` included. Run 1 followed a build and the download and hashing of 214 MB; that may be the whole cause, or a first run may cost more. One pair of readings. The bench of Prompt 59 should keep its first run apart from the others.

## TE-2: word probabilities at no more than 10% extra time

**Question.** Does the runtime give a probability for each token, or each word, at no more than 10% extra time, so that `confidence` can be the mean per word (§24.1; TS §16.4)?

**Method.** Three readings. (1) The source of `@huggingface/transformers` 4.3.1 as installed: the generation loop, the Whisper word-timestamp path and the speech pipeline. (2) In run 2 of TE-1, with a temporary line in `whisper-runtime.ts` that was removed afterwards: the option `output_scores: true` was added to the call, and the names of the fields of what the pipeline returned were sent to the page, for each of the three windows on each backend. (3) The times of run 1, option off, against run 2, option on.

**Numbers.**

| Read | Found |
|---|---|
| The generation loop (`src/models/modeling_utils.js`) | It computes the log-probability of each token it picks and adds it to one sum per input. It returns `sequences`, `past_key_values` and the attentions. Its last lines read `// TODO: // scores, // logits,` |
| `output_scores` | Declared in the generation configuration with the value `false`; read nowhere in the package |
| The word-timestamp path (`src/models/whisper/modeling_whisper.js`) | Adds `token_timestamps` to that result and nothing else |
| The speech pipeline (`src/pipelines/automatic-speech-recognition.js`) | Takes `sequences` and `token_timestamps`; its result is built by the tokenizer from those two |
| The fields of the result, option on, 6 of 6 windows | `text` and `chunks`; every chunk has `text` and `timestamp` and nothing else |
| `confidence` of the 157 words, both backends, both runs | 1 for every word |
| Time, option off against option on | No comparison can be made: the option changes nothing. The two runs differ for another reason (see TE-1) |

**Date.** 2026-10-09.

**Decision.** Not passed: the runtime gives no probability, so the question of its cost does not arise. The fallback of §24.1 is taken: `Confidence(1.0)` for every word. `whisper-runtime.ts` and `word-timestamps.ts` already do that; the comment in `word-timestamps.ts` now says why. The dollar span of the reference clip therefore has a confidence of 1.0, which is not below 0.80: the NumberReveal of Prompt 46 is not held back by it. No decision of §2 was needed, since the fallback was agreed before the experiment.

**What follows.** A score in V2 is never lowered by how sure the recognizer was: a number it heard wrongly is shown with the same weight as one it heard well, and V3's detector, which multiplies every score by word confidence (TS §17.4), multiplies by 1. The caption editor is where a wrong word is put right.

**A way to a probability that was not taken.** The runtime lets a caller pass a `logits_processor`, which sees the scores of every step and could keep the probability of the token that greedy decoding then picks. That gives a probability per token. It does not give one per word: the pipeline returns words with no token ids, and the grouping of tokens into words is done inside the tokenizer by methods the package does not export. Matching the two lists from outside would be a guess that can be wrong without a sign. Not built in V2. For V3: call the model and the tokenizer without the pipeline, or take a version of the runtime that returns `scores`, and run TE-2 again.
