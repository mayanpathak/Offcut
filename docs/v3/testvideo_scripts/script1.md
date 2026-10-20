# Test video script 1: phone, portrait, all four event kinds

**Save as** `testclips/v3script1.mp4` (git-ignored; never uploaded, never committed). **Length** about 65 s; it must stay under 90 s.

## Before you press record

- **Phone, held upright (portrait), 1080p at 30 fps.** Not 4K, not 60 fps or more.
- **H.264, not HEVC.** iPhone: Settings > Camera > Formats > **Most Compatible**. Android: turn off "HEVC" / "High efficiency video" in the camera's settings. An HEVC file is refused (`REJECT_HEVC`).
- **Location off for the camera** (iPhone: Settings > Privacy > Location Services > Camera > Never; Android: camera settings > location tags off). A file with a GPS tag must stay on this laptop (v3implementation D-100).
- A quiet room, the phone at arm's length, no music.
- Say every number as the words printed. Raise your voice only on a word in CAPITALS. If you stumble, record another take; do not add words.

## Script

> I run a small channel about building apps, and last quarter something changed.
>
> We crossed ten thousand subscribers. Honestly, that felt UNREAL.
>
> The reason is simple: my editing time went from six hours to forty five minutes per video.
>
> There are four tools I use every day. First, a script template. Second, a cheap microphone. Third, automatic captions. Fourth, a fixed upload day.
>
> The microphone cost eighty dollars, and it paid for itself in a week.
>
> Watch time is up forty percent, and new videos get found three times faster than before.
>
> In twenty twenty four I almost stopped. I tried three times to change the format, and version two of the channel failed.
>
> Now I film at seven pm, every Tuesday, and I do not miss it.
>
> Start with one tool. That is ENOUGH.

## What a person would expect

| # | Words | Kind | Expected content |
|---|---|---|---|
| 1 | "ten thousand subscribers" | NumberReveal | 10,000, count noun "subscribers" (D-101: a count of 100 or more) |
| 2 | "UNREAL" | KeywordPop | "unreal", if the recording carries the emphasis |
| 3 | "from six hours to forty five minutes" | FromTo | 6 hours to 45 minutes (both time units, D-85) |
| 4 | "four tools … First … Second … Third … Fourth" | ListReveal | header "4 TOOLS"; items "a script template", "a cheap microphone", "automatic captions", "a fixed upload day" |
| 5 | "eighty dollars" | NumberReveal | $80, USD |
| 6 | "forty percent" | NumberReveal | 40% |
| 7 | "three times faster" | NumberReveal | 3x ("times" followed by a comparative, D-102) |
| 8 | "ENOUGH" | KeywordPop | "enough", if the recording carries the emphasis |

**Traps: a person expects no event here.**

| Words | Why not |
|---|---|
| "twenty twenty four" | A year (X1), however the recognizer writes it |
| "I tried three times" | "times" without a comparative is no unit (D-102), and 3 is a bare small number (X5) |
| "version two" | A version (X2) |
| "at seven pm" | A clock time (X3) |
| "one tool" | A bare small number (X5) |
| "in a week" | No number |
