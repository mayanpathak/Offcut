# experiments.md - Offcut V2: experiment records

**What this is.** The outcome of every experiment V2 runs: the technical experiments TE-n of `technicalspec.md` §37 and the product experiments E-n of `product.md` §19. The procedures and thresholds are in `v2implementation.md` §24.1. The M0 gate decision (§24.3) is written at the end of this file.

**How it is kept.** One entry per experiment, in the order they are run. An entry has five parts: the question, the method, the numbers, the date, the decision. A reading is recorded as measured and also normalised to 60 s (measured x 60,000 / 74,705), because the reference clip is 74.7 s long (D-64). R1 is the only reference machine (D-65); a reading taken on the development machine is labelled `dev` and never stands for E-3 or E-4.

| Experiment | Prompt | State |
|---|---|---|
| TE-7 re-check (ranged requests to the asset host, with the real model files) | 38 | Run on 2026-10-09: passed |
| TE-1 (no request outside the closed host list during transcription) | 44 | Not run |
| TE-2 (word probabilities at no more than 10% extra time) | 44 | Not run |
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
