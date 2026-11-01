# experiments.md - outcome of every reading V5 takes (v5implementation section 20)

Every reading is taken on D1 and recorded under the label `d1` (D-202). One section per row of section 20. Real exports were made once on a keyed build of the finished code (`187020e`, with a temporary `performance.measure`, then removed) and judged by `verify/verify_mp4.py`; the files are deleted, apart from the one the founder listens to, which stays on D1.

## Render and encode (E-4)
Question: Is `render_encode` at most 90 s per 60 s of video, in the browser on D1?
Method: `BENCH_DEVICE=d1 BENCH_CLIP=portrait BENCH_RUNS=5 pnpm bench:device` (one warm-up, five measured runs of the 60 s portrait fixture, headed Chrome 156), on a keyed production build of `187020e`, Docker Desktop stopped; written to `bench/results/d1-2026-11-01-portrait.json` and compared with V4's `bench/results/d1-2026-10-29-portrait.json`.
Numbers: d1 — median 30,944 ms, p90 33,269 ms, runs 30,001 to 33,269 ms. V4 read 38,773 ms on a loaded machine, V3 30,848 ms. Total of the pipeline and export: median 113,999 ms, against V4's 140,494 ms. Peak memory 893 MB.
Date: 2026-11-01
Decision: Met with a wide margin, 30.9 s per 60 s against 90 s; BP §6.2's line of 150 s is far away. A pass on D1 is not a pass on R1 (D-202); V9 reads it there.

## Mux
Question: Does writing the file take at most 2 s?
Method: The same bench.
Numbers: d1 — median 21 ms, p90 31 ms. V4 read 25 ms.
Date: 2026-11-01
Decision: Met. `ctts` costs nothing here: D1's encoder never reorders.

## detect_scene on a quiet machine (V4 known issue 37)
Question: Is `detect_scene` within V4's line of about 245 ms (10% over V3's 195 ms, plus about 30 ms for the plan and the time map) on a quiet D1?
Method: The same bench, Docker Desktop stopped, as V4's reading could not be.
Numbers: d1 — median 255 ms, p90 291 ms, runs 237, 291, 255, 257, 220 ms. V4 read 300.5 ms loaded and 213 to 249 ms once settled; V3 195 ms. In the same runs `asr` read 80,401 ms against V3's 69,469 ms, 16% slower, and `probe_audio` 1,064 ms against 875 ms, 22% slower, in stages neither V4 nor V5 changed: D1 is a little slower today than on V3's day. V5 does not touch this stage's code.
Date: 2026-11-01
Decision: Missed by 10 ms on the median, and within the machine's own drift. Not a V5 blocker, as section 20 says. V4 CL known issue 37 is reworded and carried to V9 with these numbers, where R1 and M2.4 read it; the native `build_scene` is 6.3 ms, so no profiling is due until then.

## Loudness (D-204)
Question: Does an export read -14 ± 1 LUFS as a stereo file, by a tool that is not the product?
Method: Check 7's `INFO`, `ffmpeg ebur128` on the stream as it is, on four real exports: Creator and Free of the reference clip, Creator of both pause fixtures. The "before" was read in P130 on a real export, before the fix.
Numbers: d1 — all four -14.3 LUFS. Before the fix: -11.4 LUFS on the reference clip.
Date: 2026-11-01
Decision: Met. The product was fixed, not the check. How a person hears it is the founder's listen at P136, since the preview and the export are both about 3 dB quieter than V4's.

## Sync and timeline
Question: Against the source, is the audio within 20 ms over the first and the last 5 s, and every white frame within one frame?
Method: Check 8's `INFO` on the same four exports.
Numbers: d1 — audio offset, first window / last window, correlation: reference clip, both profiles, 1.3 / -1.7 ms, 0.95 / 0.99; long pause -0.3 / 0.5 ms, 0.97 / 0.99; sparse 0.0 / 0.3 ms, 1.00 / 1.00. White frames: 2 compared on each, largest offset 1 frame on the reference clip and 0 on both fixtures. The second recording, whose audio starts 304.9 ms after its video, passes with the gap filled (P130).
Date: 2026-11-01
Decision: Met, by a factor of ten. `MIN_ENVELOPE_CORR` stays 0.5; the lowest reading is 0.95. Flashes stay 225 and 30, read in P130, where 200 and 80 missed real white frames. No threshold moved to pass a file.

## Watermark correlation
Question: Do checks at 0.5 and 0.2 tell a watermarked file from a clean one, at least 0.3 apart?
Method: Check 9's `INFO` on the same four exports: the mean of 15 evenly spaced frames scaled to 720 x 1280, high-passed, correlated with `watermark_mask_720.png` inside its box. The mask is drawn from the font by a tool of its own.
Numbers: d1 — free 0.74; creator -0.05 on the reference clip, -0.01 on the long pause, -0.03 on the sparse clip.
Date: 2026-11-01
Decision: Met: 0.79 apart, on the right sides of both. The thresholds stay. All on one Free file: V6's `export-free.spec.ts` reads more.

## Bitrate
Question: Is the video at most 12 Mb/s for Creator and 6 Mb/s for Free (1.5x the target)?
Method: Check 10's `INFO`, packet sizes over the stream's duration.
Numbers: d1 — Creator 8.3 Mb/s on the reference clip and the long pause, 8.4 Mb/s on the sparse clip; Free 4.1 Mb/s.
Date: 2026-11-01
Decision: Met. `bitrateMode` stays `variable`. D1's encoder only: TE-4 reads others on R1 and R2.

## Hit time
Question: Is the same edit exported again under 2 s (TS §21.4, "instantly")?
Method: `export_done.total_ms` with `from_cache: true`, five hits in a row of the reference clip's 79 MB Creator file, on the finished build; `total_ms` runs from the click to the file being handed to the browser.
Numbers: d1 — 43, 12, 14, 26, 12 ms. The render it replaces took 35,796 ms. `export-creator.spec.ts` and `timeline-preserved.spec.ts` hold each hit under 2,000 ms.
Date: 2026-11-01
Decision: Met by far. `lookup` needs nothing.

## cacheKey cost
Question: Does hashing the key on the main thread take a few milliseconds (D-222, INV-17)?
Method: A temporary `performance.measure` around `cacheKey` (canonical JSON and SHA-256), read on the reference clip, 1:14 of speech, seven exports; removed after.
Numbers: d1 — 0.9, 1.5, 0.5, 0.4, 1.6, 0.5, 0.8 ms.
Date: 2026-11-01
Decision: Met. It stays on the main thread.

## App shell
Question: Is the app shell within 400 kB gzip (TS §13.5)?
Method: `scripts/check-hosts.mjs`, printed by `pnpm build`.
Numbers: d1 — 339.6 kB gzip on the plain production build. V4 read 337.3 kB at V5's baseline: V5 added 2.3 kB.
Date: 2026-11-01
Decision: Met.

## Exports from the deployed build
Question: Does the deployed build, with its own key, headers and hosts, make the file the local builds make?
Method: A temporary spec on `https://offcut-one.vercel.app` at `354a706`, headed Chrome on D1, at the founder's word: tokens signed in the test's process with the production key, one hour long, never printed or written. Each file judged by `verify/verify_mp4.py`.
Numbers: d1 — all 11 checks on every file. Reference clip, Creator: -14.3 LUFS, audio 1.3 / -1.7 ms at 0.95 / 0.99, watermark -0.05, 8.3 Mb/s, `total_ms` 28,809. Free: -14.3 LUFS, watermark 0.74, 4.1 Mb/s. Edited: -14.3 LUFS, watermark -0.04, 8.3 Mb/s, `total_ms` 32,846. Expired Creator token: Free, watermark 0.72. Long pause: -14.3 LUFS, audio -0.3 / 0.5 ms at 0.97 / 0.99, white frames off by 0. Hits 11 and 24 ms.
Date: 2026-11-01
Decision: Met: the same readings as the local builds. A person's real token waits for V6.
