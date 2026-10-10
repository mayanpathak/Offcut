# experiments.md - Offcut V2: experiment records

**What this is.** The outcome of every experiment V2 runs: the technical experiments TE-n of `technicalspec.md` §37 and the product experiments E-n of `product.md` §19. The procedures and thresholds are in `v2implementation.md` §24.1. The M0 gate decision (§24.3) is written at the end of this file.

**How it is kept.** One entry per experiment, in the order they are run. An entry has five parts: the question, the method, the numbers, the date, the decision. A reading is recorded as measured and also normalised to 60 s (measured x 60,000 / 74,705), because the reference clip is 74.7 s long (D-64). R1 was to be the only reference machine (D-65). Since D-69 (2026-10-09) no reading is taken on R1 in V2: the gates are read on the development machine, D1, which is an Acer Aspire A715-76G: Intel Core i5-12450H (8 cores, 12 threads), 16 GB, Intel UHD Graphics and an NVIDIA GeForce GTX 1650, Windows 11. Chrome under Playwright draws with its integrated GPU. A reading taken on D1 for a gate, by the stated method, is labelled `d1`. A reading taken in passing is labelled `dev` and stands for nothing. No number in this file is R1's; the founder's reading of E-3 at S6 is of a laptop like R1.

| Experiment | Prompt | State |
|---|---|---|
| TE-7 re-check (ranged requests to the asset host, with the real model files) | 38 | Run on 2026-10-09: passed |
| TE-1 (no request outside the closed host list during transcription) | 44 | Run on 2026-10-09: passed, on WebGPU and on WASM |
| TE-2 (word probabilities at no more than 10% extra time) | 44 | Run on 2026-10-09: not passed, the runtime returns no probability. Fallback taken: `confidence` is 1.0 for every word |
| E-3 (transcription time on R1) | 44, 59 | S6: the founder's reading of 2026-10-09, about 150 s, accepted (D-67). S15: not run |
| TE-3 (frame capture method; render-only speed) | 51 | Run on 2026-10-09 on D1: passed. Method A; 130 frames a second or more render-only; an export with the page hidden passes |
| TE-4 (encoder ladder entry; AAC priming) | 51 | Run on 2026-10-09 on D1: passed. `avc1.640028` with hardware at both sizes; no priming |
| E-4 (render and encode time; on D1 since D-69) | 51, 59 | S11: run on 2026-10-09 on D1, median 29,641 ms against 112,000 ms: continue. S15: not run |
| TE-10 (the Windows media job in CI: does it run, and its minutes) | 58 | The minutes: read on 2026-10-10, a public repository pays none. The job: in `ci.yml` since 2026-10-10, **not run yet**; its first run is the push of Prompt 58 |
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

## TE-3: the frame capture method, the render speed, a hidden page

**Question.** Does the renderer draw a correct frame every time, to an `OffscreenCanvas` in a worker; which way of taking the frame off the canvas should the export use; does it render at 30 frames a second or more without the encoder; does an export finish with the page hidden (§24.1)?

**Method.** On D1 (D-69). A production build with the test public key, served by `vite preview` at `http://localhost:4173` with the headers of `vercel.json`; Chrome 155.0.8059.39 under Playwright, headless, WebGPU adapter `intel gen-12lp`; the reference clip, 2,242 frames, the Creator profile, 1080 x 1920. Temporary code in `export-loop.ts` and `render.worker.ts`, removed afterwards, added a second capture path and a pass that renders without encoding.

- **Method A:** `new VideoFrame(canvas)` straight after the draw.
- **Method B, as tried:** a readback of the pixels through the browser. The canvas is drawn onto a 2D canvas, the bytes are read with `getImageData`, and the frame is made from that buffer of RGBA bytes. This is not the `copyTextureToBuffer` of TS §21.4: that one sits inside the Rust renderer and would have had to be written for the trial. Both hand the encoder a buffer of RGBA bytes.
- **Ten frames** (0, 150, 300, 600, 900, 1,200, 1,500, 1,890, 1,946, 2,241) were decoded from one export of each method with `ffmpeg` and their stored brightness compared pixel by pixel.
- **Render only:** every frame decoded and drawn, no `VideoFrame` made and no encoder fed; then one pixel of the canvas read, which returns when the GPU has finished everything it was handed. A second pass also made and closed a `VideoFrame` for every frame.
- **Hidden:** an ordinary Chrome 155, with a window, started by hand and driven over its DevTools port: a page under Playwright always counts as focused and visible, and Playwright starts Chrome with the throttling of hidden pages switched off. The window was minimised 0.3 s after the export began.

**Numbers.**

| Read | Result |
|---|---|
| Method A, `render_encode`, three exports in one session | 30,943 ms, 28,566 ms, 35,596 ms |
| Method B, `render_encode`, two exports | 81,417 ms, 79,140 ms: 2.8 times method A |
| The verifier on one export of each | Six passes, both |
| The ten frames, B against A as stored | They differ by 8 to 20 of 255 on average. A's stored brightness is 0.859 times B's plus 16.0, on every one of the ten: B's file holds brightness from 0 to 255, A's from 16 to 235, and neither stream carries a range flag |
| The ten frames, with B's brightness brought to 16..235 | The same pictures: 0.05 to 0.72 of 255 apart on average; at most 0.15% of the pixels differ by more than 8 |
| The white frames in an export of method A | Frames 150 and 1,946, and no other |
| Render only, two passes | 15,054 ms and 17,202 ms for 2,242 frames: 149 and 130 frames a second |
| Render and capture, no encoder | 20,499 ms: 109 frames a second |
| Hidden: `document.visibilityState` | `hidden` from 0.3 s after the start to the end: 19 of 19 samples, taken every 2 s |
| Hidden: the export | `render_encode` 35,097 ms, `mux` 27 ms; all 2,243 progress messages reached the hidden page; the verifier passes the file on all six checks; white frames at 150 and 1,946 |

**Date.** 2026-10-09, on D1.

**Decision.** Passed. **Method A.** It is the faster of the two by 2.8, and it is the one whose file is right: a frame made from a buffer of RGBA bytes is encoded with full-range brightness into a stream that does not say so, which a player shows with too much contrast. The method is named in `export-loop.ts` and in TS §21.4. No Canvas2D backend is needed.

**What this does not show.** R1, or any machine but D1 (D-69). The `copyTextureToBuffer` readback itself was not built or timed; it could be faster than the readback tried here, and it would give the encoder the same kind of buffer. A page that Chrome has frozen or discarded after a long time in the background. A GPU other than the integrated one.

## TE-4: the encoder ladder entry and the AAC priming

**Question.** Which entry of the ladder does this machine encode with, at both sizes; does the export pass the verifier; how many samples does the AAC encoder put in front of the audio (§24.1, D-33)?

**Method.** On D1, in the same build and browser as TE-3. `pickVideoConfig` was read inside the render worker, for a Creator and for a Free export. The audio encoder's first and last chunk were read in the loop. The guide's `ffprobe` line (G 8.4) was run on an export. The decoded sound of an export was laid over the decoded sound of the source, sample by sample, with `ffmpeg` and NumPy.

**Numbers.**

| Read | Result |
|---|---|
| The ladder entry at 1080 x 1920 and 8,000,000 b/s | `avc1.640028`, `prefer-hardware`: the first entry |
| The ladder entry at 720 x 1280 and 4,000,000 b/s | The same entry |
| The video as written | H.264 High, `yuv420p`, no B-frames; 8.24 Mb/s and 4.10 Mb/s |
| The audio encoder's first chunk | Timestamp 0, duration 21,333 microseconds, 427 bytes |
| Its chunks | 3,504, for 3,503.1 AAC frames of input: none in front. The last has the timestamp 74,730,666 |
| `ffprobe`: `start_time`, `start_pts`; the first three packets | 0, 0; `pts` 0, 1,024, 2,048, each 1,024 long |
| The decoded sound against the source's | No shift. At a shift of 0 the two differ by 0.4% of the level (the median of 1,098 windows of 50 ms; 7.6% in the worst). The first sample that is not silence is sample 9,052 in both |
| Audio against video | The audio is 18.7 ms longer: `N x 1600` samples rounded up to whole AAC frames. Within one video frame |
| The verifier | Six passes on every export that was kept: six Creator, two Free |

**Date.** 2026-10-09, on D1.

**Decision.** Passed. The ladder keeps its order and the two bitrates stay. **`AAC_PRIMING_SAMPLES` is 0, now as a measured value:** the encoder puts nothing in front, so the muxer writes no edit list. `encoders.ts` says so.

**What this does not show.** The priming is the Windows encoder's, on D1. No other platform and no other machine was measured (D-65, D-69): an encoder that does put samples in front would shift the sound by that much, and check 5 would not always see it, because it compares lengths. The software entries of the ladder were not exported with. Playback on a phone is M2.5.

## E-4, at S11: render and encode time on D1

**Question.** Does rendering and encoding the reference clip stay inside its budget (PS §19, §24.1)? Since D-69 it is read on D1.

**Method.** The code of Prompt 51 as committed, with the temporary hook and the test public key; `vite preview`, the production headers; Chrome 155 under Playwright, headless. Three Creator exports of the reference clip, each in a page that was just loaded, so with new workers, as a visitor's first export is. The time is the worker's own `render_encode` of `stageTimings`: from just before the loop is called to the end of the audio encoder's flush. The transcript was the one made once, earlier in the session.

**Numbers.**

| Read | Value |
|---|---|
| `render_encode`, the three exports | 28,286 ms, 29,641 ms, 37,631 ms |
| Median | **29,641 ms** |
| Normalised to 60 s (x 60,000 / 74,705) | 23,807 ms |
| The target for the reference clip (D-64) | 112,000 ms: met. The reading is a quarter of it |
| `mux`, the three exports | 55 ms, 85 ms, 91 ms; median 85 ms, against 2,500 ms |
| The verifier on the three files | Six passes each |

Other `render_encode` readings of the same day, labelled `dev`: 28,282 ms (Prompt 50); 30,943, 28,566 and 35,596 ms (TE-3); 38,820 ms for a fourth export in the third page; 35,097 ms with the page hidden; and 41,182, 43,067 and 39,455 ms in a Chrome with a window, the first tries at the hidden-page check, in which the page stayed visible. Twelve Creator exports in all, from 28.3 s to 43.1 s. A Free export, 720 x 1280: 15,445 ms in Prompt 50, 27,735 and 30,697 ms later in this session.

**Date.** 2026-10-09, on D1.

**Decision.** Continue: the reading falls in the first column of §24.3. No fallback is applied. The reading of S15 (`pnpm bench:device`, ten runs, Prompt 59) is still to be taken.

**What this reading does not show.** R1. The line is 3.8 times this reading, and D1 was about twice as fast as the one laptop like R1 at transcribing; nothing says what the factor is for drawing and encoding, which lean on the GPU and its encoder and not on the processor. **The time rose as the session went on,** by a third to a half from the first export to the last, and the Free export's time doubled; the cause was not looked for. Decoding here is of a 720p source, which is cheaper than a 1080 x 1920 phone clip (D-64).

## TE-10: the media suites on a hosted Windows runner

**Question.** Do the three media suites pass headless in Chrome on `windows-latest`, and does a month of runs fit the free minutes of GitHub Actions (§24.1)?

**State on 2026-10-10: half answered.** The minutes are read. The suites have not run on the runner: the job was written in Prompt 58 and its first run is that prompt's push, which is the founder's.

**Method.** `ci.yml` calls `e2e-media.yml` as the job `e2e-media`, after the job `ci` and before any deploy, on every pull request and on `main` (D-44). The job runs on `windows-latest`: Node, pnpm, Chrome through Playwright, Python 3.13 with `verify/requirements.txt`, and `ffmpeg` 9.0.2 from a release archive whose SHA-256 is written in the workflow. It takes the `web/dist` that the job `ci` built with the E2E test key, restores `fixtures/.cache/` from the Actions cache, and runs `pnpm e2e:media`. For the minutes: GitHub's page on the billing of GitHub Actions, read on 2026-10-10.

**Numbers.**

| Read | Value |
|---|---|
| The allowance | "GitHub Actions usage is free for self-hosted runners and for public repositories that use standard GitHub-hosted runners." The repository is public, and `windows-latest` is a standard hosted runner: the job's minutes count against nothing. The same page says a larger runner is always charged for; none is used |
| The suites on D1, for scale | 30 cases in 9.7 minutes, headless, one worker, with a GPU (entry of Prompt 57) |
| The first run of the job, 2026-10-10 (run 38044970567, commit `45dbea2`) | Every step before the suites passed on `windows-latest`: Node and pnpm; Chrome through Playwright, 66 s; Python and the verifier's requirements, 27 s; `ffmpeg` from its archive, 5 s; the `web-dist` artifact of the job `ci`, 2 s. **The suites then stopped after 4 s, before any test ran:** a fault of `export-creator.spec.ts`, which asked `ffprobe` for the reference clip while the file was being loaded, before the setup project had downloaded it. Corrected the same day (`v2changelog.md`) |
| The second run, 2026-10-10 (run 38053613126, commit `72878ba`) | The setup test passed: the model and the reference clip were downloaded on the runner and their hashes hold, 15 s. **Then every case failed at the same place, 11 failed and 19 did not run, 12.6 min:** no drop zone on `/app` within 30 s, which is what the page shows when the app's capability check refuses the browser. The log does not say which capability. Since that day the setup asks first and says it |
| The job on the runner: did the 30 cases pass | **No.** The app does not start there as Chrome is started now |
| Chrome's software WebGPU adapter, on D1 (`--enable-unsafe-webgpu --use-webgpu-adapter=swiftshader`), for what a machine without a graphics card can do at best | The model download: not tried. The pipeline: the speech model first failed, for want of 16-bit floats on that adapter, which was a fault of the app and is corrected; then `ready` after 82 s with the model on WASM, and `pipeline-preview.spec.ts` 11 of 11 in 2.8 min. **The export: 17% of 2,242 frames after 180 s, about 2.1 frames a second, so about 18 minutes for one Creator file,** against 35 s with the graphics processor. D1 has 12 threads; a hosted runner has 4 |
| The job's minutes | The failed run took 2 min 30 s, of which about 2 min 20 s were the tools. A run of the 30 cases: not known yet |
| WebGPU on the runner, and Chrome's arguments if it needs any | **Not known yet.** The setup test prints it on the next run |
| The H.264 and the AAC encoder on the runner | **Not known.** A Windows Server image may lack the system encoders Chrome uses |

**Date.** 2026-10-10 for the allowance. The run: open.

**Decision.** Not taken, and since 2026-10-10 it is plain that it is not the choice G 11.5 lays out. Whatever the runner lacks, two exports of the reference clip cannot be drawn without a graphics card in a time a pull request waits for, and the fallback of §22.5, the job "on `main`", would put that wait, or that failure, before every deploy. The choices are in open item 15 of `v2changelog.md`; the founder takes one. The table below is what G 11.5 says, kept for the record:

| Outcome of the first run | Then |
|---|---|
| 30 cases pass | The job stays where it is. Write its minutes and the date here and the date into `scripts/check-external-facts.mjs` |
| The capability check answers `UNSUPPORTED_WEBGPU` | Put the arguments that give Chrome WebGPU on the runner into `E2E_CHROME_ARGS` of `e2e-media.yml`, which the `media` project hands to Chrome; record them here and in `v2changelog.md` as the arguments of TE-10 |
| It answers `UNSUPPORTED_H264_ENCODE` or `UNSUPPORTED_AAC_ENCODE` | No argument adds an encoder. The fallback of §22.5: the job on `main` and on a manual trigger only, and `pnpm e2e:device` on D1 before every merge. Record the reason |
| The cases run but a clip takes longer than a case may | The runner transcribes without a GPU. Read how long, and decide between a longer limit for the job and the fallback |

**What this does not show yet.** Whether a page on the runner gets WebGPU and the two encoders, and how long a clip takes there. The workflow's own steps have run once and work.
