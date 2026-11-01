# verify/ - the independent check of an exported MP4

`verify_mp4.py` reads a file that Offcut exported and says whether it is what the product promises. It is written apart from the product so that the two cannot agree by sharing a mistake.

## What it shares with the product

Nothing. It imports no code from `crates/` or `web/` and no library either of them uses. It runs the `ffprobe` and `ffmpeg` command-line tools as separate programs, reads the top-level boxes of the file itself, and does its arithmetic with NumPy. None of these is part of the product, which contains no ffmpeg code (`technicalspec.md` §36). `watermark_mask_720.png` is drawn by a tool of its own from the font file, not by the renderer.

## How to run it

```text
python verify/verify_mp4.py <file> --profile free|creator --expected-duration-ms N [--source <fixture>]
pnpm verify <file> --profile creator --expected-duration-ms N
```

`N` is how long the export should be, in milliseconds, worked out by the test. `--source` is what checks 8 and 11 compare with; the two are separate, because from V6.2 they differ. Without `--source` those parts are reported as not measured.

It prints one line per check, `PASS <n>` or `FAIL <n>: <reason>`. A line `INFO <n>: <reading>` reports a number and judges nothing. A reason names a tag's key, never its value. The exit status is 0 only when every check passes, 1 when one fails, and 2 when a tool or NumPy is missing or the arguments are wrong.

## The checks

The list is `technicalspec.md` §27.2.

| # | Check |
|---|---|
| 1 | The container is MP4 with exactly one video and one audio stream and no other; `moov` comes before `mdat` |
| 2 | Video: H.264, `yuv420p`, square pixels, no rotation; 1080x1920 for `creator`, 720x1280 for `free` |
| 3 | Every frame comes exactly 1/30 s after the one before it; the frame count is `ceil(N x 30 / 1000)` |
| 4 | Audio: AAC-LC, 48,000 Hz, 2 channels |
| 5 | The video is within one frame (33.4 ms) of `N`; the audio is within one AAC frame (21.34 ms) of the video |
| 6 | `ffmpeg` decodes both streams from start to end and reports no error |
| 7 | `ffmpeg`'s integrated loudness of the stereo stream is within 1 LU of -14 LUFS |
| 8 | Against the source: the audio's 10 ms envelope within 20 ms over the first and the last 5 s, so a removed pause anywhere between fails; white frames within one frame |
| 9 | What stays still over 15 frames correlates with `watermark_mask_720.png` at least 0.5 for `free`, at most 0.2 for `creator` |
| 10 | The video's bitrate over the clip is at most 12 Mb/s for `creator`, 6 Mb/s for `free` |
| 11 | No title, comment, description, artist, album or location; `creation_time` absent or the epoch; no tag carrying the source's value; neither `©xyz` nor `ISO6709` in the file's bytes |

The thresholds of checks 8 and 9 are assumptions, read on real exports (`docs/v5/experiments.md`, `technicalspec.md` §39.3 item 14). One moves only while `free` and `creator` stay well apart, and never to make one file pass.

## Its own test

```text
python verify/test_verify_mp4.py
pnpm verify:test
```

It makes a good file and one bad file per failure with `ffmpeg` from a generated fixture, in a temporary directory it deletes, and each bad file must fail its own check and pass every other. It needs `pnpm fixtures`, and runs in CI, which cannot make a real export.

## The mask

`make_watermark_mask.py` drew `watermark_mask_720.png`, with Pillow, which is the tool's alone and is not in `requirements.txt`. Run it again only when `watermark.rs` or `safe_area.rs` moves the text.

## What it needs

| Tool | Version it was written and tried with |
|---|---|
| Python | 3.13.5 |
| `ffprobe`, `ffmpeg` | 9.0.2, on the `PATH` |
| NumPy | 2.5.3 (`requirements.txt`), for checks 8 and 9 |

```text
python -m pip install -r verify/requirements.txt
```

On the development machine plain `python` is another Python, without pip; put the folder of Python 3.13 first in `PATH` (`docs/v2/v2buildguide.md`, "Read this first", item 2).
