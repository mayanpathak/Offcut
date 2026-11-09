# experiments.md - V6.1 "Long source": the readings

Each experiment of `v6.1implementation.md` §13, read on D1 (`BENCH_DEVICE=d1`: Windows 11, Intel UHD and NVIDIA GTX 1650, Chrome as the media suites start it) unless it says otherwise. Every real Groq call is also listed in `docs/v6/v6changelog.md` with its seconds of audio. Numbers marked "(assumption)" in the plan are replaced here by what was read.

**The local long files (P171, G 1.1),** generated into `fixtures/.gen-local/` (git-ignored) from the reference clip's sound: one period is its whole audio track (74.517 s as decoded, not the video stream's 74.705 s) followed by 5 s of pink room tone at about -60 dBFS, repeated to 2,700 s, AAC-LC 128 kbps 48 kHz stereo. Video is a 10 s `testsrc2` segment copied in a loop.

| File | Video | Size |
|---|---|---|
| `long_45min_4k_hevc.mov` | 3840 × 2160 HEVC Main (`hvc1`), 4 Mbps, 30 fps | 1,479,378,583 B |
| `long_45min_1080p_h264.mp4` | 1920 × 1080 H.264, 16 Mbps (a phone's rate, so the file passes 4 GiB), 30 fps | 5,608,209,302 B |
| `hevc_main10_2s.mov` | 1920 × 1080 HEVC Main 10 (`hvc1`), 2 s | 929,388 B |

## TE-17 - Reading a file in place

### First read (P171, 2026-11-08, d1)

A throwaway page and worker (`$SCRATCH/probe/`), served with cross-origin isolation so `performance.measureUserAgentSpecificMemory()` answers, the `File` given by Playwright's `setInputFiles`. The worker walks the top-level boxes with 16-byte reads, reads `moov` whole, then 200 seeded random 1 MiB-aligned blocks of 1 MiB, the last block, and (on the 5.6 GB file) blocks at 4 GiB and 4 GiB − 512 KiB. The SHA-256 of up to seven blocks is compared with the same bytes read by Node.

| File | `moov` | `moov` read | 1 MiB block p50 / p90 / max | All reads | Hashes | Page memory before / peak / after |
|---|---|---|---|---|---|---|
| 4K HEVC, 1.48 GB | 3.07 MB at byte 20 | 8.7 ms | 6.8 / 8.6 / 43.8 ms | 1.44 s (201 blocks) | 2 of 2 equal | 0.87 / 1.02 / 1.08 MB |
| 1080p H.264, 5.61 GB | 2.99 MB at byte 32 | 6.9 ms | 3.0 / 6.4 / 32.6 ms | 0.79 s (203 blocks) | **7 of 7 equal, 5 of them above 4 GiB** | 1.25 / 1.25 / 1.03 MB |

**Reading.** `FileReaderSync` reads late blocks of a file over 4 GiB correctly (P171's stop condition does not fire), and reading holds no more memory than the block in hand. Both files had just been written, so the operating system's cache held much of them: these are warm reads, an upper bound on speed. The cold read (a file not touched since boot, and a USB drive) is P181's. D-400's 1 MiB blocks and 8-block cache stand as planned. The plan's "30 GB file" was not made: the 5.6 GB file already crosses the 32-bit boundary that matters, and 30 GB would have taken half the free disk.

### Second read (P181, 2026-11-08 23:50 IST, d1)

The real product: the keyed flags-on build (`dist-flags/`) in Chrome, signed in through `fake-api.ts`, the fake answering each window from the reference clip's cloud fixture (so this times reading and the audio step, not Groq: TE-18 does). Each file chosen in the long page's own file field, so Chrome reads it from the disk as a person's file is. Page memory is `performance.measureUserAgentSpecificMemory()` (the app is cross-origin isolated; it counts the page and its workers, not the GPU process), sampled about every 15-25 s, as each call takes that long to answer.

| File | Probe (pick to the gate's Start) | Audio step (Start to transcribing) | 34 windows, fake | Sentences | Page memory peak |
|---|---|---|---|---|---|
| 4K HEVC, 44:59, 1.48 GB | 0.16 s | **22 s** | 32 s | 544 | **34 MB** |
| 1080p H.264, 45:00, 5.61 GB | 0.15 s | **55 s** | 32 s | 544 | **37 MB** |

**The clip path with `high_res` (F-3's condition).** The 4K file's first 89 s, re-encoded as 3840 × 2160 H.264 (the clip page refuses HEVC, D-427 as read), on `/app`, signed in as Creator: processed in 97 s, exported at 1080 × 1920 in 52 s, **peak page memory 883 MB**. `rej_4k.mp4` (1 s, a tone): taken at import, refused `REJECT_NO_SPEECH` after it was heard, peak 5 MB.

**Reading.** Every pass line holds: the probe well under 2 s, the audio step well under 4 minutes, peak memory far under 1.5 GB. Reading in place holds almost nothing: 34-37 MB for a 45-minute source, against the 520 MB `decodeAudio` would hold for its sound alone. The 5.6 GB file takes longer because its bytes are 3.8 times as many: the demuxer reads every audio sample from among them. Both files were written today, so the operating system's cache may hold some of them: these are warm reads; a cold read and a USB drive are the founder's (R1, R2). **F-3: 4K clip peak 883 MB: `high_res` may be turned on** (the founder's morning step; the plan's condition is the 1.5 GB line).

**Found.** `long_45min_4k_hevc.mov`'s video track lasts 2,700.067 s, 67 ms over 45:00, and the long page refuses it as longer than 45 minutes: `MAX_SOURCE_DURATION` is exact, with no tolerance. A phone's "45:00" recording can run a fraction of a second over. Recorded for the founder (V6 CL, Found, not fixed); TE-17 read a 44:59 copy (`-t 2699 -c copy`).

## TE-18 - Cloud windows over 45 minutes

### First read (P171, 2026-11-08 05:06-05:09 IST, d1; F-1)

A throwaway script (`$SCRATCH/te18.mjs`) cut the 45-minute sound (16 kHz mono) into 32 windows of 90 s with 5 s overlap (`ffmpeg -ss 85k -t 90 -ac 1 -ar 16000 -c:a libopus -b:a 32k`, about 300 kB each) and sent them to Groq as `server/src/transcribe/groq.rs` does (`whisper-large-v3-turbo`, `language=en`, `temperature=0`, `verbose_json`, word timestamps), two in flight. The key was read alone from `.env.deploy` by the script's own process; nothing printed a value.

**Calls.** 33 requests: 32 answered 200, one 429 (window 27, `retry-after: 3`). Audio accepted: 31 × 90 s + 65 s = **2,855 s**, under the hour's 3,000 s (A-3). The first pass sent windows 0-28 (28 answered in 69.5 s of wall time) and stopped at the 429 as the script must; a second single pass at 05:08:43 sent windows 27, 29, 30 and 31 (12.5 s). No retry loop.

| Measure | Read |
|---|---|
| Latency per window | p50 2.8 s, p90 5.9 s, max 6.4 s (33 requests) |
| Time for 32 windows, two in flight | about 80 s of wall time, when not limited (pass 1: 28 windows in 69.5 s) |
| Words per 90 s window | 159-184 (Script A's speech is dense: the 74.5 s clip gives 155 words) |
| Groq's audio budget headers | `x-ratelimit-limit-audio-seconds: 7200` (an hour); remaining 7,110 after the first window, 4,597 after the last |
| Groq's request headers | `x-ratelimit-limit-requests: 2000` (a day) |
| **The 429** | After 27 requests in about 66 s, with 4,872 audio seconds still left in the hour: a per-minute request limit the headers do not name (the free tier's, as read; about 20 a minute). Two windows in flight at about 3 s each make about 40 requests a minute |

**The seams.** Analysed by `$SCRATCH/te18-seams.mjs`, which merges the windows as `mergeWindows` does (the middle of each overlap, a word straddling the line kept once: 4 such words) and compares every loop of the reference clip with loop 0's 155 words (counts only; no transcript is kept in the repository).

| | Read |
|---|---|
| Windows with a word starting in their first 0.15 s or ending in their last 0.15 s | 25 of 32 (the windows cut through continuous speech) |
| Word edits against loop 0, over 32 loops | 42 (0.85% of 4,960 words) |
| Edits within 2 s of one of the 31 seams | **5: 1 word lost, 2 doubled, 2 changed** |
| Edits elsewhere | 37: 26 changed, 8 lost, 3 added (the model hearing the same audio differently in another context) |
| Words placed in room tone, before D-382's guard | 1 in 45 minutes (at 1,190 s, the first instant of window 14) |

**Reading.** Against §13's pass lines: the time is well inside "6 min p90 after the audio step"; **"no word lost or doubled at a seam" fails narrowly** (3 in 31 seams); room tone is nearly clean before the guard. P171's stop condition ("words cut at a window's edge in more than 2 of 32 windows") also fires. **Safe path taken (ON §6): `CLOUD_OVERLAP_S` rises to 10 s,** so each seam's line sits 5 s from both windows' edges instead of 2.5 s; step 80 s, `⌈(D − 10) ÷ 80⌉` windows (34 for 45 minutes), `MAX_SOURCE_WINDOWS` 35. It costs 6% more audio per source (3,060 s of 90 s windows for 45 minutes instead of 2,880 s). The plan's D-405 and D-411 carry "As read" notes. P181's product run is the re-read, with 10 s.

**For the founder (not changed tonight).** On the free tier, the per-minute request limit binds before the hourly audio: two windows in flight (D-409, confirmed in Entry 23) will meet it on every 45-minute source, and D-410 turns the server's 503 into retries then a `busy` pause. Groq's Dev tier (open item 19) before `long_video` goes on removes it; otherwise one window in flight (about 2 minutes for 45 minutes at the latency read) stays under it.

### Second read (P181, 2026-11-09 00:54-00:56 IST, d1; F-1, A-3)

The real product: `dist-flags/` in Chrome, signed in through `fake-api.ts`, a 40-minute source (`long_45min_4k_hevc.mov`'s first 2,400 s, `-c copy`: 45 minutes with the 10 s overlap is 3,060 s of windows, over A-3's 3,000 s an hour) read in place; each window's `POST /transcribe` (its Ogg body, as the page sent it) answered by the test from a real Groq call made as `server/src/transcribe/groq.rs` does (the same model and fields, the times rounded to ms, the words trimmed: `align.rs` is the identity on Groq's words). The key was read alone from `.env.deploy` into that process; nothing printed a value.

| Measure | Read |
|---|---|
| Requests | 31: 30 answered 200, **one 429** (window 20/30, `retry-after: 3`) |
| Audio sent to Groq | **2,690 s** (29 × 90 s + 80 s); Groq's hour showed 7,110 s left after the first window, 4,690 s after the last |
| Latency per window | p50 0.72 s, p90 1.05 s, max 1.11 s |
| The audio step | 27 s |
| The windows, Start of transcription to the transcript | 98 s, of which 65 s was the `busy` pause |
| The 429 | The product paused **"Transcription paused: it is busy. Continue in a few minutes."**; Continue after a minute sent the missing windows and finished. Nothing was lost or sent twice |

**Reading.** Time: far inside "6 min p90 after the audio step" (33 s of windows without the pause). D-410's busy path works against the real provider. Groq's free tier still answers 429 at about 20 requests a minute (window 20 of 30 here, as P171 read at 27): open item 24 stands; Groq's Dev tier (open item 19), or one window in flight, before `long_video` goes on. An earlier attempt (started 23:51, stopped at 00:21) waited on a page state the test did not recognise and kept no record of its calls: **how much audio it sent is not known**, at most the same 2,690 s, and Groq's allowance refills as it goes, so its 7,110 s at 00:54 does not tell. If it sent all of it before 00:21, the hour from 23:56 to 00:56 held up to 5,380 s of these tests' audio, over A-3's 3,000 s (Groq's own limit, 7,200 s, was not reached). The harness now writes every call as it is made, and the next run waited an hour after this one.

**The seams (a second run at 01:57-01:59, the same 2,690 s: 31 requests, 30 × 200, one 429 at window 20 that paused and continued).** The master transcript held **4,677 words**. Each of the 29 whole loops was compared word by word with loop 0's 155 words (an edit distance, counts only; no transcript is kept in the repository): **96 edits in all** (2.1%: 30 lost, 26 added, 40 changed), of which **5 fall within 2 s of one of the 29 seams: 2 lost, 3 changed, none doubled**. The seams' 4 s are 5% of the time, so about 4.8 of the 96 would fall there by chance: at the 10 s overlap the merge adds nothing measurable, and the doubling the 5 s overlap showed (2 of 31 seams in P171) is gone. **No word was placed in room tone** after the guard. Groq's hour showed 4,684 s left after the last window. **Against §13: "no word lost or doubled at a seam" fails by 2 lost words**, at the base rate of the model's own variation: a stop-and-ask, recorded for the founder (V6 CL open item 27) and not a blocker for a dark merge (`long_video` stays off in production); V6.4's labelled long set re-reads it on real speech.

## TE-20 - A hidden tab and a sleep

### Simulated read (P181, 2026-11-09 00:30 IST, d1)

The product on `dist-flags/`, `long_12min.mp4` (9 windows), the fake answering each window after 20 s, headed Chrome.

| Step | Read |
|---|---|
| Visible | 10.6 s per window (20 s each, two in flight): as expected |
| **Reload after 3 windows kept**, then "Continue your video" | Finished without the file; **no kept window sent again**; 10 requests in all for 9 windows: only the one in flight was lost |
| Hidden: the page's own window minimised (`Browser.setWindowBounds`), and a second page in front | `document.visibilityState` stayed `visible`, in both Playwright's Chrome and one started without its `--disable-renderer-backgrounding`, `--disable-backgrounding-occluded-windows` and `--disable-background-timer-throttling`: 10.8 s per window, as visible |
| Frozen 60 s (`Page.setWebLifecycleState`) | Not taken: windows 6-8 were requested during the 60 s and the job finished |

**Reading.** The resume half of TE-20 passes: a reload loses at most the windows in flight (§13's line). **The hidden-tab and frozen halves could not be simulated faithfully on this machine's automated Chrome** (the page never became hidden, and a visible page is not frozen), so they are not read. **The real test is the founder's** (`overnight-run.md` §8 step 3): start a long video on `dist-flags/`, close the lid for five minutes, open it, and see the job pause and continue; and a run with the tab in the background. The copy already asks to keep the tab open (§10.5).

## TE-21 - HDR

No phone HDR clip was supplied (`overnight-run.md` §12, A-2); it is read on `rej_hdr_hlg.mov` only.

### Read (P181, 2026-11-09 00:25 IST, d1)

`rej_hdr_hlg.mov` (640 × 360 HEVC Main 10, BT.2020, HLG, `colr` written: the HLG is in the metadata; the picture is `testsrc2`'s saturated pattern) played in a `<video>` on the app's own origin, one frame at 0.5 s drawn to a 2D canvas (Chrome's own colour management, as a frame reaches the page), against ffmpeg's straight decode of the same frame (no conversion). The render path itself was not forced to `hdr_ok`: that would change frozen render code even in a scratch build (ON §3).

| Measure | Chrome | ffmpeg, straight |
|---|---|---|
| Mean R / G / B | 119.9 / 115.0 / 123.6 | 122.3 / 127.4 / 127.1 |
| Channels at 255 | 24.7% | 24.7% |
| Channels at 0 | 49.0% | 31.8% |
| Mean saturation | 0.994 | 0.959 |

**Reading.** Chrome maps the BT.2020 HLG frame into sRGB: not washed out (the levels and highlights match), but more channels clip to black, the gamut mapping of colours outside sRGB. On a real phone HDR clip (bright skies, skin) that is the risk §13 names. **`hdr_ok` stays `false`** (§15, A-2); a real clip is the founder's to read before HDR is accepted.

## HEVC in D1's Chrome (P171, G 1.3)

`VideoDecoder.isConfigSupported` with each file's own `hvcC` as the description, and with four written strings, headless and headed (the same answers):

| Source | Codec string (built from `hvcC`, ISO/IEC 14496-15 Annex E) | no-preference | prefer-hardware | prefer-software |
|---|---|---|---|---|
| `long_45min_4k_hevc.mov` (Main, 3840 × 2160) | `hvc1.1.6.L150.90` | yes | yes | no |
| `hevc_main10_2s.mov` (Main 10) | `hvc1.2.4.L120.90` | yes | yes | no |
| `rej_hevc.mov` (Main) | `hvc1.1.6.L93.90` | yes | yes | no |
| written | `hvc1.1.6.L93.B0`, `hvc1.1.6.L153.B0`, `hvc1.2.4.L153.B0`, `hev1.1.6.L93.B0` | yes | yes | no |

**Reading.** D1's Chrome decodes HEVC Main and Main 10 up to level 5.1 through the graphics hardware and has no software HEVC decoder, so a machine without hardware support answers no and gets `REJECT_HEVC` (D-398, as planned). P171's stop condition ("no HEVC at all") does not fire. x265's constraint bytes are `90` (progressive, frame-only); the string keeps them as the box gives them. CI's hosted runner has no graphics card, so `high-res.spec.ts`'s HEVC case there takes the `REJECT_HEVC` branch the spec already allows.

## Long AAC in `AudioDecoder` (P171, G 1.3)

The 45-minute track as ADTS (`ffmpeg -c copy -f adts`), every frame through one `AudioDecoder` in a worker, queue held under 256, each `AudioData` closed at once:

| Measure | Read |
|---|---|
| Frames in / out | 126,564 / 126,564, no error |
| Decoded rate | 48,000 Hz on every output (the container's) |
| Samples | 129,601,536 per channel = 2,700.03 s |
| Time | 5.3 s for 45 minutes |
| Page memory peak | 9.7 MB |

**Reading.** Decoding is not the long step; reading and resampling in a stream (D-401) keep the memory as low as this. HE-AAC could not be read: D1's ffmpeg has no HE-AAC encoder (its native `aac` writes AAC-LC only), so D-402's rate correction is proven by `audio-region.test.ts`'s fake decoder, and TE-12's re-read waits for a real HE-AAC file.

## The demuxer's audio track (P171, G 1.5)

`crates/offcut-mp4/src/demux.rs` (`Demuxer::open`) takes **the first audio track in file order**, whatever its `tkhd` enabled flag (video, by contrast, prefers the first enabled track), and counts every audio track. So `SourceInfo`'s first audio track (§6.1, D-402) is the track `read_audio_sample` already reads: no change needed.

## TE-12 (re-read) - `moof` and the HE-AAC rate on the long files

The three generated files have no `moof` (ffmpeg writes a plain `moov` and one `mdat`) and AAC-LC at 48 kHz. Real long recordings (V6.4's labelled set) are the re-read; HE-AAC is above.
