# remotion-prompts.md - the prompts for the landing page's clips

These prompts are for Claude Code, run in a separate Remotion project outside this repository. The clips they make fill the slots of `designimplementation.md` §9.

**Order.** Run Prompt 0 first, then Prompts 1-6 in any order. Clips 1-4 are required; 5 and 6 are optional, and their slots show CSS illustrations otherwise (D-325).

**What goes into `public/` first:**

- `public/footage/raw.mp4`: one raw talking-head take of the founder (landscape is best).
- `public/footage/export.mp4`: its Offcut export (Creator, so no watermark).
- `public/screens/*.png`: screenshots from `~/v6-review/` and `~/v4-review-2/` (`empty`, `ready`, `style-tab`, `editing-word`, `exporting`, `done`).

**After rendering.** The `out/` folder goes through `designimplementation.md` §9.2's checks before `scripts/upload-assets.sh media` uploads it.

## Prompt 0: project, brand, rules (run first)

```text
Create a new Remotion project at C:\Users\Mayan\Desktop\sh2clips\offcut-remotion (NOT inside the
offcut folder, and never modify anything in C:\Users\Mayan\Desktop\sh2clips\offcut). Use the latest
stable Remotion with TypeScript and React. Initialize git there.

PURPOSE
These are short, silent, looping product clips for the landing page of Offcut, a desktop web app.
Offcut turns a raw talking-head video into a finished vertical (9:16) short: it transcribes the
speech on the user's own computer, cleans the voice and normalizes loudness without changing the
length, adds animated word-by-word captions, adds visuals for numbers, lists and "from X to Y"
comparisons it hears, frames a landscape video to 9:16, and exports an MP4. The clips must show
exactly what the product does today, nothing it does not.

TRUTH RULES (the clips are marketing; every claim must be true today)
- Built today: clips up to 90 seconds, English; 3 caption styles named Clean, Bold, Mono; 4 visuals
  named NumberReveal, ListReveal, FromTo, KeywordPop; voice cleanup + loudness normalization with the
  duration unchanged; 9:16 framing with an adjustable crop; preview; editing words; export MP4.
  Free: 3 exports a month, 720p, small watermark. Creator: unlimited, 1080p, no watermark.
  Works in Chrome or Edge on Windows and Mac (desktop only).
- NOT built yet, never shown as available: long videos to several shorts, AI cuts, B-roll, music,
  templates, other languages, mobile, team features. Clip 6 may show "long video to shorts" only
  with an on-screen "Coming soon" label for its whole duration.
- No logos, names, UI or sounds of TikTok, Instagram, YouTube, CapCut, OpusClip or any other
  company. A generic phone frame is fine. No stock people. The only person on screen is the
  founder, from the footage in public/footage/.
- Never say "AI-powered", "100% private", "never leaves your device" or legal/compliance claims.
  The exact privacy wording allowed is: "Your video stays on your computer."

BRAND (match the Offcut app exactly)
- Background #0E0E11; surfaces #16161B and #1D1D23; borders #2A2A32; text #F4F4F6; secondary text
  #A3A3AD; brand violet #8B5CF6 (hover/darker #7C4DEE); success #34D399.
- UI font: Inter (download the Inter variable font files and bundle them in public/fonts; OFL
  licence; load with @remotion/fonts or a FontFace in code; never fetch fonts at render time).
  Also bundle JetBrains Mono 700 for the Mono caption style.
- Radii 6/10/16 px; soft shadows; generous spacing; motion eased with cubic-bezier(0.2, 0.8, 0.2, 1).
- Captions inside the vertical short (these are the product's real styles):
  * Clean (default): white Inter 800, dark stroke #101014, soft black drop shadow; the word being
    spoken sits on a rounded yellow pill #FFD60A with the word in #111114; the pill slides from
    word to word.
  * Bold: same colours, Inter 900, and a keyword that "pops" (scales up) in orange #FF5E3A.
  * Mono: JetBrains Mono 700, pale green text #E8F0EA on a dark green panel (#0B0F0C at 85%),
    the spoken word in green #3DDC84.
  * Captions sit in the lower part of the 9:16 frame, centred, 2 lines at most, about 6 words a line.
- Visuals inside the short (product behaviour): NumberReveal counts up from 0 over 0.5 s to a big
  number (e.g. 10,000) with a small label under it; ListReveal shows numbered items appearing one by
  one; FromTo shows two bars, the "to" bar in the accent colour and slightly larger; KeywordPop
  scales one caption word.

SHARED BUILDING BLOCKS (build these first, reuse in every clip)
- <PhoneFrame>: a clean, generic modern phone outline (no brand), 9:16 screen with rounded corners,
  subtle reflection, holds any child.
- <Caption> with styles clean | bold | mono, driven by a word list with start/end times in frames,
  pill/active word animated with spring().
- <NumberReveal>, <ListReveal>, <FromTo>, <KeywordPop> as above.
- <BrowserWindow>: a minimal dark browser/app window chrome for showing the editor.
- <Waveform>: animated bars, with a "noisy" and a "clean" mode.
- A <Label> chip: small rounded pill, surface #1D1D23, border #2A2A32, text #F4F4F6, optional violet dot.
Put the brand values in one theme.ts. Use spring()/interpolate() with the brand easing; no harsh
cuts; everything readable at small sizes.

FOOTAGE
public/footage/raw.mp4 is the founder's raw take; public/footage/export.mp4 is the same take as an
Offcut export (1080x1920). public/screens/*.png are real screenshots of the app at 1280x800 and
1920x1080. Use <OffthreadVideo> for footage, muted. If a file is missing, stop and tell me rather
than inventing a substitute.

OUTPUT RULES (every clip)
- 30 fps. Silent: render with --muted (no audio track at all).
- Seamless loop: the last frame must lead back into the first with no visible jump (design the
  animation as a cycle, or cross-fade the last 10 frames into frame 0).
- Two renders per clip into out/: a master and a web file.
  * Web file: H.264 MP4, yuv420p, CRF chosen so the file stays under the size cap given in the
    clip's prompt; then remux with ffmpeg "-c copy -movflags +faststart" so it starts playing
    before it has fully downloaded. Check with ffprobe that it has no audio stream.
  * Poster: the clip's first frame via "npx remotion still", converted to WebP at quality 80,
    under 80 kB, same size as the web file.
- Name files offcut-<clip-name>-<width>x<height>.mp4 / .webp.
- After rendering, write out/manifest.json listing each file with width, height, duration in
  seconds, frame count, size in bytes, and whether it loops, and print a short table.
- Add an npm script "render:all" that renders every clip and its poster.

Do Prompt 0 only now: set up the project, the theme, the shared components, a Studio preview of
each component, and the render script. Show me the Studio composition list when done.
```

## Prompt 1: the hero loop (slot `hero`, required)

```text
In offcut-remotion, add the composition "hero" (9:16).
Master 1080x1920, web file 720x1280, 10 seconds, seamless loop. Web file cap: 2.5 MB.

Story (it is the first thing a visitor sees; it must say "raw take in, finished short out" in under
3 seconds without any text):
0.0-1.5 s  The founder's raw landscape take (public/footage/raw.mp4) appears in a dark, flat
           16:9 frame, slightly dull, with a small "Raw take" Label chip. A thin noisy Waveform
           runs under it.
1.5-3.0 s  The 16:9 frame smoothly crops and reshapes into a vertical 9:16 screen inside a
           PhoneFrame (an animated crop: the sides slide in, the frame rotates/scales into the
           phone). The waveform calms into the clean mode as it passes.
3.0-9.0 s  Inside the phone, play public/footage/export.mp4 (the real Offcut export) from a
           segment where a number or a list is spoken; over it, if the export already has captions,
           do not draw extra captions. Around the phone, three Label chips float in one by one with
           a gentle stagger and slow drift: "Voice cleaned", "Captions, word by word",
           "10,000 → counted up" (use the actual number spoken in the footage if there is one).
9.0-10.0 s Everything eases back so frame 300 matches frame 0.
Background: #0E0E11 with a very subtle violet radial glow (#8B5CF6 at about 12%) behind the phone,
slowly breathing. No logo, no headline text (the page provides the headline).
Keep the phone centred with space around it; the page will place it to the right of the headline.
```

## Prompt 2: before and after (slot `beforeAfter`, required)

```text
In offcut-remotion, add the composition "before-after" (16:9).
Master 1920x1080, web file 1280x720, 12 seconds, seamless loop. Web file cap: 3 MB.

Left half: "Before". The raw landscape take (public/footage/raw.mp4) in a flat frame, no captions,
with a noisy Waveform under it and a muted Label "Raw take, 1 minute". Slightly desaturated.
Right half: "After". A PhoneFrame playing the same moment of public/footage/export.mp4, with a
clean Waveform and a Label "Finished short, same length".
Between them, an animated violet arrow (#8B5CF6) that draws itself left to right at 1.0-2.0 s, with
three small steps appearing along it in sequence: "Transcribed on your computer", "Voice cleaned",
"Captions and visuals added". Each step is a Label chip that lights up (violet dot) as the arrow
passes it.
Both sides play in sync so the viewer sees the same words before and after.
At 10.5-12 s the steps dim and the arrow retracts so the loop restarts cleanly.
Text sizes must be readable when the video is shown 800 px wide.
```

## Prompt 3: caption styles (slot `captionStyles`, required)

```text
In offcut-remotion, add the composition "caption-styles" (9:16).
Master 1080x1920, web file 720x1280, 9 seconds, seamless loop. Web file cap: 2 MB.

A PhoneFrame shows the same 3-second moment of public/footage/export.mp4 three times in a row,
each time with a different caption style drawn by our <Caption> component over the video (use a
moment of the export where there are no burned-in captions, or a frame region you can cover; if
every frame already has burned-in captions, use public/footage/raw.mp4 cropped to 9:16 instead):
0-3 s Clean, 3-6 s Bold (with one KeywordPop in orange), 6-9 s Mono.
The sentence is one the founder actually says in the footage; time each word to the speech by eye
(list the word timings in a constants file so I can adjust them).
Under the phone, a segmented control like the app's Style tab (three pills: Clean, Bold, Mono) with
a violet highlight that slides to the active style at each switch.
The style switch is a quick 8-frame cross-fade of the caption layer only; the video keeps playing.
```

## Prompt 4: visuals (slot `visuals`, required)

```text
In offcut-remotion, add the composition "visuals" (1:1).
Master 1080x1080, web file 960x960, 12 seconds, seamless loop. Web file cap: 2.5 MB.

A 2x2 grid of four rounded cards (#16161B, border #2A2A32, radius 16), each a small 9:16 or square
"screen" demonstrating one visual exactly as the product does it, each starting 0.4 s after the
previous and then looping its own animation:
1. NumberReveal: "10,000" counts up from 0 in 0.5 s, big white bold number, label "subscribers"
   beneath, holds, fades.
2. ListReveal: header "3 things I changed", items "1  Shorter intro", "2  Better light",
   "3  One idea per video" appear one by one, stacked.
3. FromTo: "from 2 to 9" — two vertical bars grow, the second in yellow #FFD60A and 1.15x scale,
   values on top.
4. KeywordPop: a Clean caption line "This is the ONE thing that matters" where "ONE" pops (scale
   1.0 -> 1.25 -> 1.0) in yellow.
Each card has a small caption Label under it with the visual's plain name: "Numbers", "Lists",
"Before → after", "Key words".
Calm, premium motion; no more than two things moving at once.
```

## Prompt 5: how it works (slot `howItWorks`, optional)

```text
In offcut-remotion, add the composition "how-it-works" (16:9).
Master 1920x1080, web file 1280x720, 12 seconds, seamless loop. Web file cap: 2.5 MB.

A BrowserWindow centred on #0E0E11, showing three steps in sequence, each 4 s, using the real
screenshots in public/screens/ as the window content (do not redraw the app's UI from imagination):
1. "Drop" (empty.png): a video file icon glides in from the left and drops into the drop zone; a
   soft violet ripple.
2. "Review" (ready.png then editing-word.png): a cursor clicks a word in the caption editor; the
   word changes; a small check appears.
3. "Export" (exporting.png then done.png): a progress bar fills; a downloaded MP4 file icon slides
   out of the window to the bottom right.
Across the top, a 3-step indicator (1 Drop · 2 Review · 3 Export) with the current step in violet.
Use gentle zoom-ins (Ken Burns, 1.0 -> 1.06) on the screenshots so they feel alive.
```

## Prompt 6: privacy and coming soon (slots `privacy` and `comingSoon`, optional)

```text
In offcut-remotion, add two compositions, both 16:9, master 1920x1080, web file 1280x720,
seamless loops, cap 1.5 MB each.

A) "privacy", 6 seconds: a simple line drawing of a laptop (strokes #A3A3AD, accents violet). A
   video file icon sits inside the laptop screen with a soft glow; small dotted lines try to leave
   the laptop toward a cloud icon and fade out before reaching it, the cloud stays empty. Text
   under the laptop, Inter 600, #F4F4F6: "Your video stays on your computer." Nothing else.
B) "coming-soon", 8 seconds: a long horizontal timeline strip (a 20-minute video, drawn as a strip
   of thumbnails from public/footage/raw.mp4) from which 5 highlighted segments lift up and turn
   into 5 small vertical phone screens in a row. A "Coming soon" Label chip with a violet dot stays
   visible in the top-left corner for the whole 8 seconds. Text: "One long video. Several finished
   shorts." No dates, no numbers of hours.
```
