# Test video script 2: laptop webcam, landscape, the traps

**Save as** `testclips/v3script2.mp4` (git-ignored; never uploaded, never committed). **Length** about 70 s; it must stay under 90 s.

This script is mostly numbers a person would **not** animate: versions, years, phone numbers, clock times, small counts. It checks that the detector keeps quiet where an animation would be wrong, and still finds the few real ones.

## Before you press record

- **The laptop's webcam (landscape).** The Windows Camera app records H.264 MP4, which is what Offcut takes. 720p or 1080p, 30 fps.
- A quiet room, normal distance from the laptop, no music.
- Say every number as the words printed. Raise your voice only on a word in CAPITALS. If you stumble, record another take; do not add words.

## Script

> Quick update on the app I am building.
>
> I moved the backend from Python three point eleven to Python three point twelve, and I now test everything on an iPhone fifteen Pro. Nothing dramatic.
>
> The first version shipped in two thousand nineteen. A user once asked me to call five five five one two three four, which was not a real number.
>
> My stand-up is at nine am, and I keep it to two points.
>
> Now the part that matters. The new build loads in four hundred milliseconds instead of three seconds.
>
> Last month the app passed one point five million views on the store page.
>
> Here is what I learned. First, ship small. Second, measure everything. Third, delete what nobody uses.
>
> I lost ten pounds during the launch, and I drank too much coffee.
>
> The storage bill dropped from two hundred and forty dollars to sixty dollars a month.
>
> Shipping small is the WHOLE trick.

## What a person would expect

| # | Words | Kind | Expected content |
|---|---|---|---|
| 1 | "four hundred milliseconds" | NumberReveal | 400 ms |
| 2 | "one point five million views" | NumberReveal | 1.5M, count noun "views" |
| 3 | "First … Second … Third" | ListReveal | the marker-only form (three ordinals from "first", no announcer, D-105); items "ship small", "measure everything", "delete what nobody uses" |
| 4 | "from two hundred and forty dollars to sixty dollars" | FromTo | $240 to $60 |
| 5 | "WHOLE" | KeywordPop | "whole", if the recording carries the emphasis |

"three seconds" in "four hundred milliseconds instead of three seconds" is a second number in the same sentence, not a "from … to" pair: a person expects the 400 ms only.

**Traps: a person expects no event here.**

| Words | Why not |
|---|---|
| "Python three point eleven", "Python three point twelve" | Versions (X2) |
| "iPhone fifteen Pro" | A product name (X2) |
| "two thousand nineteen" | A year (X1) |
| "five five five one two three four" | Phone-like digits (X4) |
| "at nine am" | A clock time (X3) |
| "two points" | A bare small number (X5) |
| "ten pounds" | "pounds" is not a currency (D-102); 10 alone is no event |
