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

### Second read (P181)

Not read yet.

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

### Second read (P181)

Not read yet.

## TE-20 - A hidden tab and a sleep

Not read yet.

## TE-21 - HDR

Not read yet. No phone HDR clip was supplied (`overnight-run.md` §12, A-2); it is read on `rej_hdr_hlg.mov` only.

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
