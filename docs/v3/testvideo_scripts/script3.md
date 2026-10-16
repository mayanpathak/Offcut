# Test video script 3: phone, landscape, a noisy room and a long pause

**Save as** `testclips/v3script3.mp4` (git-ignored; never uploaded, never committed). **Length** about 65 s with the pause; it must stay under 90 s.

This one tests the voice chain: steady background noise for the denoiser, a quiet and a loud passage for the loudness step, and a pause of about four seconds that Offcut must keep exactly as recorded (it never cuts silence).

## Before you press record

- **Phone, held sideways (landscape), 1080p at 30 fps.** Not 4K, not 60 fps or more.
- **H.264, not HEVC** (iPhone: Settings > Camera > Formats > **Most Compatible**; Android: turn off "HEVC"). **Location off for the camera.** As in script 1.
- **A steady noise in the room:** a fan, an air conditioner or a fridge. Not music and not a TV with voices.
- Read the first paragraph a little quieter than normal and the last one a little louder. Raise your voice on a word in CAPITALS.
- At **[pause]**, stay silent and still for a slow count of four. Do not stop the recording.

## Script

> (a little quieter) This is a recording with a fan running behind me, on purpose.
>
> My video folder holds two hundred and fifty gigabytes of raw footage, and most of it is never used.
>
> **[pause: count four in silence]**
>
> Sorry, I lost my place. Where was I.
>
> My first client paid me fifty thousand rupees for a short explainer. That was a big deal.
>
> We went from six months of planning to two weeks of planning, because the template does the boring part.
>
> Today the studio has a hundred and twenty clients, and the average video is four point seven minutes long.
>
> (a little louder) Clean audio matters more than a new camera. It is the CHEAPEST upgrade you can make.

## What a person would expect

| # | Words | Kind | Expected content |
|---|---|---|---|
| 1 | "two hundred and fifty gigabytes" | NumberReveal | 250 GB |
| 2 | "fifty thousand rupees" | NumberReveal | ₹50,000, INR |
| 3 | "from six months … to two weeks" | FromTo | 6 months to 2 weeks (both time units) |
| 4 | "a hundred and twenty clients" | NumberReveal | 120, count noun "clients" (100 or more, D-101) |
| 5 | "four point seven minutes" | NumberReveal | 4.7 minutes |
| 6 | "CHEAPEST" | KeywordPop | "cheapest", if the recording carries the emphasis |

**Traps: a person expects no event here.**

| Words | Why not |
|---|---|
| "My first client" | "first" in the middle of a sentence is not a list marker (D-105) |
| The pause | Nothing: the pause is kept as recorded, its length unchanged in the export |

**What else to check by eye and ear** after Offcut processes it: the fan is quieter but the voice is not muffled; the quiet and the loud paragraph end up at about the same level; the pause is still there, at the same place and the same length.
