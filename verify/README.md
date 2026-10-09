# verify/ - the independent check of an exported MP4

`verify_mp4.py` reads a file that Offcut exported and says whether it is what the product promises. It is written apart from the product so that the two cannot agree by sharing a mistake.

## What it shares with the product

Nothing. It imports no code from `crates/` or `web/` and no library either of them uses. It runs the `ffprobe` and `ffmpeg` command-line tools as separate programs and reads the top-level boxes of the file itself. Neither tool is part of the product, which contains no ffmpeg code (`technicalspec.md` §36).

## How to run it

```text
python verify/verify_mp4.py <file> --profile free|creator --expected-duration-ms N [--source <fixture>]
pnpm verify <file> --profile creator --expected-duration-ms N
```

`N` is the duration of the source clip in milliseconds: an export is never shorter than its recording. `--source` is accepted and not used before V5.

It prints one line per check, `PASS <n>` or `FAIL <n>: <reason>`, then one line that names the checks this version does not make. The exit status is 0 only when every check that is made passes, 1 when one fails, and 2 when a tool is missing or the arguments are wrong.

## The checks

The list is `technicalspec.md` §27.2. This version makes the first six.

| # | Check |
|---|---|
| 1 | The container is MP4 with exactly one video and one audio stream and no other; `moov` comes before `mdat` |
| 2 | Video: H.264, `yuv420p`, square pixels, no rotation; 1080x1920 for `creator`, 720x1280 for `free` |
| 3 | Every frame comes exactly 1/30 s after the one before it; the frame count is `ceil(N x 30 / 1000)` |
| 4 | Audio: AAC-LC, 48,000 Hz, 2 channels |
| 5 | The video is within one frame (33.4 ms) of `N`; the audio is within one AAC frame (21.34 ms) of the video |
| 6 | `ffmpeg` decodes both streams from start to end and reports no error |
| 7 to 11 | Loudness, sync and timeline against the source, watermark, bitrate, metadata: V5 |

## What it needs

| Tool | Version it was written and tried with |
|---|---|
| Python | 3.13.5 |
| `ffprobe`, `ffmpeg` | 9.0.2, on the `PATH` |
| NumPy | 2.5.3 (`requirements.txt`). Not imported before V5, which uses it for checks 7 to 9 |

```text
python -m pip install -r verify/requirements.txt
```

On the development machine plain `python` is another Python, without pip; put the folder of Python 3.13 first in `PATH` (`docs/v2/v2buildguide.md`, "Read this first", item 2).
