# Speech fixtures: reading scripts

The committed speech clips are recorded by the founder reading the scripts below (`technicalspec.md` §26, "Fixture matrix"). Each section gives the script, how to read it, and the transcript and events the pipeline is expected to produce.

The scripts are written against the detector rules of `technicalspec.md` §17.2-§17.5. Changing a word can add or remove an event, so re-check the "Why it is worded this way" notes before editing a script.

| Clip | Script | Recorded for |
|---|---|---|
| `testclips/speech_scriptA_landscape_720p.mp4` (not in git) | Script A | V2: the reference clip. A webcam recording, 1280x720, 74.7 s (`docs/v2/v2implementation.md`, D-39 and D-64). Video-stream duration: **74,705 ms** (`ffprobe`, 74.705033 s, read on 2026-10-08). H.264 and AAC at 48 kHz stereo, 35,201,023 bytes. White-flash frames: 150 and 1950 |
| `speech_60s_portrait.mp4` | Script A | V3: derived in V3 from the reference clip (D-76): its 0.960-60.960 s, cropped to 9:16 and scaled to 1080x1920, 30 fps, white frames 150 and 1650. Generated into `fixtures/.gen/speech/` by `pnpm fixtures`, not in git |
| `speech_60s_landscape.mp4` | Script A | V3: derived in V3 from the reference clip (D-76): its 0.960-60.960 s scaled to 1920x1080, 30 fps, white frames 150 and 1650. Not in git |
| `speech_20s_noevents.mp4` | Script B | V3: derived in V3 from the reference clip (D-76): its 46.600-61.000 s (sentences 12-14), 14.4 s, 1280x720, 30 fps. It holds the reference clip's words, not Script B's. Not in git |
| `speech_20s_long_pause.mp4` | Script C | V3: derived in V3 from the reference clip (D-76): its 9.160-14.160 s, 5 s of generated room tone over the video of 14.160-19.160 s, then 18.100-28.100 s; 20 s, white frames 60 and 450. The reference clip's words, not Script C's; the silence is generated, not a recorded room. Not in git |
| `speech_30s_sparse.mp4` | Script D | V3: derived in V3 from the reference clip (D-76): five 1.1 s pieces of it (from 9.160, 29.040, 46.840, 55.420, 69.120 s) placed at 0.5, 7.0, 13.5, 19.5 and 26.0 s, generated room tone between them; 30 s, white frames 15 and 885. The reference clip's words, not Script D's. Not in git |
| `testclips/v3testclip.mp4` (not in git) | - | V3 test clip, founder's recording of 2026-10-13, not in git (D-99). A webcam recording, 1280x720, 78.918 s, about 16.8 fps on average (variable), H.264 and AAC at 48 kHz stereo, 46,360,015 bytes. It carries a location tag in its metadata and never leaves the development machine (D-100). Its words: "The V3 test clip as recognized" below |

## Recording rules for every clip

- H.264 video with one AAC audio track, 30 fps, in an MP4 container.
- One speaker, a quiet room, normal speaking distance. No music.
- Say every number as the words printed in the script. Do not shorten "three thousand" to "three K".
- Read at an even level. Raise your voice only on a word printed in CAPITALS.
- Do not add words. "Um" and restarts change the expected transcript; record another take instead.
- The white-flash frames listed for each clip are single fully white video frames. The verifier uses them to measure video sync (`technicalspec.md` §26, check 8).

## Script A: the 60-second reference clip

**Clips.** `speech_60s_portrait.mp4` (1080x1920) and `speech_60s_landscape.mp4` (1920x1080).
**Length.** 60 seconds, 159 words.
**White-flash frames.** Two.

> Last year I almost quit making videos. Editing ate every evening, and almost nobody watched.
>
> So I added it up. I had spent three thousand hours editing by hand.
>
> That is a really BRUTAL way to make anything. The slow part was always the rough cut.
>
> Then I changed my workflow, and the rough cut went from ninety minutes to ten minutes.
>
> That change gave me my evenings back. And honestly, it was not luck.
>
> There are three reasons it worked. First, the captions write themselves. Second, the numbers animate automatically. Third, everything runs on my laptop.
>
> I record a clip, drop it in, and read the result. If a caption is wrong, I fix the word and move on.
>
> Now I publish every weekday instead of every month. Last month those clips earned twelve thousand dollars in sales.
>
> So keep talking to the camera, and let the software handle the rest. Your ideas matter more than your timeline.

**How to read it.** Say "BRUTAL" clearly louder and higher than the words around it. Pause for a normal breath at each blank line.

**Expected transcript.** The script, word for word.

**Expected events.** Five, in this order. The fixture matrix requires at least four.

| # | Kind | Spoken words | Expected content |
|---|---|---|---|
| 1 | NumberReveal | "three thousand hours" | value 3,000, unit hours, label "editing by hand" |
| 2 | KeywordPop | "BRUTAL" | the word "brutal" |
| 3 | FromTo | "from ninety minutes to ten minutes" | 90 minutes to 10 minutes |
| 4 | ListReveal | "three reasons … First … Second … Third" | header "3 REASONS"; items "the captions write themselves", "the numbers animate automatically", "everything runs on my laptop" |
| 5 | NumberReveal | "twelve thousand dollars" | value 12,000, currency USD, label "in sales" |

The exact display strings come from `format_quantity` (`technicalspec.md` §17.2). For the V2 reference clip they are in the section "The V2 reference clip as recognized" below; V2 produces the fifth of these events only.

**Why it is worded this way.**

- Each NumberReveal carries a unit or a currency and a magnitude of at least 1,000. That scores 1.0 before ASR confidence is applied, against a threshold of 0.80.
- Each NumberReveal sits in a sentence with no other quantity, so FromTo does not consume it.
- The FromTo uses the "from … to" frame, the same unit on both values and a one-word gap. That scores 1.0 against a threshold of 0.85.
- The list uses the announcer form ("three reasons") with three ordinal-word markers. Each item is five words or fewer, the limit of `LIST_ITEM_MAX_WORDS`.
- "BRUTAL" follows the intensifier "really". It still needs the louder, higher delivery to pass 0.80.
- The script contains no other number words, no years, no clock times, no version numbers, and no "first", "second" or "third" outside the list.
- At least two filler sentences separate neighbouring overlay events, so their display windows do not overlap (§17.5).

## The V2 reference clip as recognized

**Clip.** `testclips/speech_scriptA_landscape_720p.mp4`: the founder reading Script A to a webcam, 74,705 ms.
**Recognized on.** 2026-10-09 (Prompt 43), by the model `asr-en-v1` of `web/src/config/model-manifest.json` (the small-size English model: a 4-bit encoder, and a 4-bit decoder that computes in 16-bit floats), on the WebGPU backend and again on the WASM backend. The two transcripts are equal, word for word and millisecond for millisecond.

**Final for V2** (Prompt 56, 2026-10-10). The build of that day was given the clip once more and its clip store read: 157 words, 17 sentences, each with the word range, the two times and the text of the table below, the six numbers of the second table, and the one event. Nothing in this section was changed by that reading. It changes again only with the model (`web/src/config/model-manifest.json`), with `offcut-text`, or with the detector.

**Expected transcript.** 157 words in 17 sentences. A test compares with this text, not with the script: the recording and the script differ, and so does what the recognizer hears. Words are counted from 0; times are milliseconds from the first video frame.

| # | Words | Starts | Ends | Text |
|---|---|---|---|---|
| 1 | 0-14 | 1,560 | 8,880 | Last year, I almost quit making videos, editing 8 every evening at almost nobody watched. |
| 2 | 15-19 | 9,160 | 10,380 | So I added it up. |
| 3 | 20-27 | 10,580 | 14,100 | I had spent 3000 hours editing by hand. |
| 4 | 28-35 | 14,260 | 17,960 | This is literally brutal way to make anything. |
| 5 | 36-43 | 18,100 | 21,740 | The slow part was always a rough cut. |
| 6 | 44-59 | 22,100 | 28,580 | Then I checked my workflow and the rough cut went from 90 minutes to 10 minutes. |
| 7 | 60-72 | 29,040 | 34,880 | That change gave me my evenings back and honestly it was not luck. |
| 8 | 73-78 | 35,280 | 37,800 | There are three reasons it worked. |
| 9 | 79-83 | 38,120 | 40,680 | First the captions write themselves. |
| 10 | 84-88 | 41,080 | 44,060 | Second the numbers animate automatically. |
| 11 | 89-94 | 44,220 | 46,600 | Third everything runs on my laptop. |
| 12 | 95-105 | 46,840 | 50,480 | I record a clip, drop it in and read the result. |
| 13 | 106-117 | 50,580 | 55,020 | If a caption is wrong, I fix the word and move on. |
| 14 | 118-126 | 55,420 | 60,960 | Now I publish every week instead of every month. |
| 15 | 127-135 | 61,200 | 64,280 | Last month those clips earned $12 ,000 in sales. |
| 16 | 136-149 | 64,360 | 68,900 | So I keep talking to my camera and let the software handle the rest. |
| 17 | 150-156 | 69,120 | 71,520 | Your ideas matter more than your timeline. |

The first word starts at 1,560 ms and the last one ends at 71,520 ms. No word starts before the one before it has ended, and the shortest word is 120 ms long.

**Numbers in the transcript** (`Transcript.numbers`), as `offcut-text` reads them.

| Words | As written by the recognizer | Value | Unit | Display | Spoken at (ms) |
|---|---|---|---|---|---|
| 8-8 | `8` | 8 | none | `8` | 4,980 to 6,180 |
| 23-23 | `3000` | 3,000 | none | `3,000` | 11,880 to 12,560 |
| 55-55 | `90` | 90 | none | `90` | 26,040 to 26,320 |
| 58-58 | `10` | 10 | none | `10` | 26,820 to 27,180 |
| 75-75 | `three` | 3 | none | `3` | 36,080 to 36,400 |
| 132-133 | `$12 ,000` | 12,000 | usd | `$12k` | 62,660 to 63,540 |

The recognizer writes the amount as two words, `$12` and `,000`: it starts a new word at the thousands separator. `offcut-text` reads the two as one quantity (`docs/v2/v2implementation.md`, D-42 and section 8.4).

**Expected events of V2.** One. V2 detects NumberReveal only, and only a quantity with a currency and a magnitude of at least 1,000 clears the threshold of 0.80 (section 9.3 of the V2 plan).

| # | Kind | Words | Expected content |
|---|---|---|---|
| 1 | NumberReveal | 132-133, `$12 ,000` | value 12,000, unit USD, display `$12k`, no label in V2 |

The five other numbers have no unit, score 0.70 and must not become events (INV-6). That holds for the `8` too, which the recognizer wrote where the script says "ate".

The event's span is 62,660 ms to 63,540 ms, the time its two words are spoken, and its confidence is 1: the recognizer gives no probability for a word (TE-2).

**Expected feed.** What the page shows while it works on the clip, in this order, and nothing else. A test of the feed takes its list of amounts from here.

| # | Line | Amount |
|---|---|---|
| 1 | "Transcribing…" | - |
| 2 | "Found: $12k" | `$12k` |

V2 shows no "Cleaning voice" line: the voice chain arrives in V3.

**Expected events of V3** (`docs/v3/v3implementation.md` §17.7). All five events of Script A, on the same recognized transcript, in time order:

| # | Kind | Words | Expected content |
|---|---|---|---|
| 1 | NumberReveal | 23-24, `3000 hours` | value 3,000, unit hours, display `3,000 hours`, label "editing by hand" |
| 2 | KeywordPop | 31, `brutal` | the word "brutal", **if the measured prosody carries it**: the recognized text is "literally brutal", and the word passes only with its loudness and pitch (`docs/v3/v3implementation.md` §9.6) |
| 3 | FromTo | 54-59, `from 90 minutes to 10 minutes` | 90 minutes to 10 minutes |
| 4 | ListReveal | 75-94, `three reasons … First … Second … Third …` | header "3 REASONS"; items "the captions write themselves", "the numbers animate automatically", "everything runs on my laptop" |
| 5 | NumberReveal | 132-133, `$12 ,000` | value 12,000, unit USD, display `$12k`, label "in sales" |

The labeled file `fixtures/labeled/a01-scriptA.json` holds this transcript in written form with events 1, 3, 4 and 5: a written transcript carries no prosody, so the KeywordPop is not expected there.

**Expected feed of V3.** "Transcribing…" and "Cleaning voice" first, in either order (the voice chain starts before the recognizer, D-90), then one line per event in time order:

| # | Line |
|---|---|
| 1, 2 | "Transcribing…", "Cleaning voice" |
| 3 | "Found: 3,000 hours" |
| 4 | "Found: brutal" (only if event 2 is detected) |
| 5 | "Found: 90 minutes to 10 minutes" |
| 6 | "Found: 3-item list" |
| 7 | "Found: $12k" |

**Where the transcript differs from Script A.** Which of these the speaker said and which the recognizer misheard has not been judged by ear.

| Script A | Recognized |
|---|---|
| "Editing ate every evening, and almost nobody watched." | "editing 8 every evening at almost nobody watched." |
| "three thousand hours" | "3000 hours" |
| "That is a really BRUTAL way" | "This is literally brutal way" |
| "always the rough cut" | "always a rough cut" |
| "Then I changed my workflow" | "Then I checked my workflow" |
| "from ninety minutes to ten minutes" | "from 90 minutes to 10 minutes" |
| "every weekday instead of every month" | "every week instead of every month" |
| "twelve thousand dollars" | "$12 ,000" |
| "So keep talking to the camera" | "So I keep talking to my camera" |

## Script B: 20 seconds with no events

**Clip.** `speech_20s_noevents.mp4`.
**Length.** 20 seconds, 50 words.

> I started filming short videos because writing felt slow. Talking is easier for me than typing.
>
> I sit down, press record, and explain an idea the way I would to a friend. Some takes are rough, and that is fine.
>
> The point is to share what I learned this week.

**How to read it.** Flat and even. Do not stress any word.

**Expected transcript.** The script, word for word.

**Expected events.** None. The editor shows `NO_EVENTS_FOUND`.

**Why it is worded this way.** No numbers, no list nouns, no ordinal words, no intensifiers.

## Script C: 20 seconds with one long pause

**Clip.** `speech_20s_long_pause.mp4`.
**Length.** 20 seconds: speech from 0 to 5 s, silence from 5 to 10 s, speech from 10 to 20 s.
**White-flash frames.** Two: one inside each speech part.

> Before I show you the result, give me a moment to think.
>
> *(Stay silent for five seconds. Keep recording; the room tone must stay in the clip.)*
>
> Okay, here it is. I recorded this clip in my kitchen, dropped it into the editor, and the captions were ready before my coffee was.

**How to read it.** Watch a timer. The second part must start at 10 seconds or later.

**Expected transcript.** The two spoken parts, word for word. Every word of the second part has a timestamp of 10 s or later.

**Expected result.** The export is 20 seconds long, never 15. Audio and video are in sync on both sides of the pause.

**Expected events.** The fixture matrix names none for this clip, and the script is written to produce none, so the test measures timing only.

## Script D: 30 seconds of sparse speech

**Clip.** `speech_30s_sparse.mp4`.
**Length.** 30 seconds. Five short phrases separated by four pauses of 3 to 6 seconds. Less than 20% of the clip is speech, which is less than 6 seconds in total.
**White-flash frames.** Two: one near each end of the clip.

| Start at | Say |
|---|---|
| 0.5 s | "Good morning." |
| 7.0 s | "Coffee is ready." |
| 13.5 s | "Camera is on." |
| 20.0 s | "Still thinking." |
| 26.5 s | "Okay, let's go." |

**How to read it.** Watch a timer. Each phrase takes about one second, which leaves pauses of about 5.5 seconds. Stay silent between phrases and keep recording until 30 seconds.

**Expected transcript.** The five phrases, word for word.

**Expected result.** The export is 30 seconds long and every pause keeps its length.

**Expected events.** The fixture matrix names none for this clip, and the script is written to produce none.

## The derived clips as recognized

The five clips of the table above are derived in V3 from the reference clip by `fixtures/gen_fixtures.sh` (`docs/v3/v3implementation.md` §17.3, D-76): same speaker, same room, same words as the reference clip, cut and scaled.

**Read on 2026-10-18** (Prompt 82): one run of each clip through the V3 pipeline in `pnpm dev` on the development machine (Chrome, the speech model on WebGPU), the transcript read from IndexedDB by the clip's id. Words are counted from 0; times are milliseconds from the clip's first frame. The feed is every line the page showed, recorded from before the page loaded. It changes again only with the model, with `offcut-text`, with `offcut-dsp` (the recognizer hears the cleaned voice) or with the detector.

| Clip | Accepted as | Feed | §17.3 expected | As recognized |
|---|---|---|---|---|
| `speech_60s_portrait.mp4` | 60,000 ms, 1080x1920, portrait, constant frame rate | "Cleaning voice", "Transcribing…", "Found: 3,000 hours", "Found: 90 minutes to 10 minutes" | 3,000 hours, (brutal), 90 to 10 minutes, 3-item list | **No list** (below); no KeywordPop |
| `speech_60s_landscape.mp4` | 60,000 ms, 1920x1080, landscape | The same four lines | The same | The same words, times and events as the portrait clip |
| `speech_20s_noevents.mp4` | 14,400 ms, 1280x720, landscape | "Cleaning voice", "Transcribing…" | No "Found:" line | As expected |
| `speech_20s_long_pause.mp4` | 20,000 ms, 1280x720, landscape | "Cleaning voice", "Transcribing…", "Found: 3,000 hours", "Found: 90 minutes to 10 minutes" | Words of the first part end before 5,000 ms; words of the second part start at 10,000 ms or later | First part ends at 5,000 ms; **the second starts at 9,960 ms** (below) |
| `speech_30s_sparse.mp4` | 30,000 ms, 1280x720, landscape | "Cleaning voice", "Transcribing…" | Under 20% speech | As built: five 1.1 s pieces, 5.5 s of 30 s (18%). The recognizer hears four words or fewer of each, one sentence per piece, starting within 0.5 s of each piece (80 to 460 ms after it); it stretches the end of the first sentence over the room tone (580-5,880 ms), so its times are not a measure of speech |

**Why the 60 s clips have no list.** The recognizer writes the three list items of the cut as one sentence, "First the captions write themselves, second the numbers animate automatically, third everything runs on my laptop.", where for the whole reference clip it wrote three (sentences 9-11 above). A list marker counts only at the start of a sentence (D-105), so "second" and "third" are not markers; "three reasons" is then announced with one marker, a partial list, which is no event (TS §17.6). The detector does what its rules say on what was recognized; the expectation of §17.3 is changed to this reading here, and the detector is not changed to fit (§20.1). A KeywordPop on "brutal" was conditional: the measured prosody does not carry it.

**Why the long pause ends at 9,960 ms.** The room tone runs from 5.000 s to 10.000 s; the recognizer puts the first word after it 40 ms early, within the 20 ms steps its times come in and the length of a word's onset. It also hears the start of the second part as "Then the slow part …" (the reference clip says "The slow part …") and the first as "Before I added it up, …" (the cut begins on "So"). The expectation is read as "at 10,000 ms, within the recognizer's timing (40 ms)".

`speech_60s_portrait.mp4` and `speech_60s_landscape.mp4`, 127 words in 10 sentences:

| # | Words | Starts | Ends | Text |
|---|---|---|---|---|
| 1 | 0-14 | 600 | 7,900 | Last year, I almost quit making videos, editing 8 every evening at almost nobody watched. |
| 2 | 15-19 | 8,200 | 9,400 | So I added it up. |
| 3 | 20-27 | 9,620 | 13,140 | I had spent 3000 hours editing by hand. |
| 4 | 28-35 | 13,300 | 17,000 | This is literally brutal way to make anything. |
| 5 | 36-43 | 17,140 | 20,780 | The slow part was always a rough cut. |
| 6 | 44-59 | 21,140 | 27,660 | Then I checked my workflow and the rough cut went from 90 minutes to 10 minutes. |
| 7 | 60-72 | 28,040 | 33,900 | That change gave me my evenings back and honestly it was not luck. |
| 8 | 73-78 | 34,300 | 36,800 | There are three reasons it worked. |
| 9 | 79-94 | 37,040 | 45,660 | First the captions write themselves, second the numbers animate automatically, third everything runs on my laptop. |
| 10 | 95-126 | 45,880 | 60,000 | I record a clip, drop it in and read the result if a caption is wrong I fix the word and move on now I publish every week instead of every month |

Numbers: `8` (word 8), `3000 hours` (23-24, `3,000 hours`), `90 minutes` (55-56), `10 minutes.` (58-59), `three` (75, `3`). Events: NumberReveal `3,000 hours` (23-24), FromTo `90 minutes` to `10 minutes` (55-59).

`speech_20s_noevents.mp4`, 32 words in 3 sentences, no number, no event:

| # | Words | Starts | Ends | Text |
|---|---|---|---|---|
| 1 | 0-15 | 260 | 6,000 | I record a clip, drop it in and read the result if a caption is wrong. |
| 2 | 16-22 | 6,200 | 8,460 | I fix the word and move on. |
| 3 | 23-31 | 8,800 | 14,400 | Now I publish every week instead of every month. |

`speech_20s_long_pause.mp4`, 38 words in 3 sentences:

| # | Words | Starts | Ends | Text |
|---|---|---|---|---|
| 1 | 0-12 | 60 | 5,000 | Before I added it up, I had spent 3000 hours editing by hand. |
| 2 | 13-21 | 9,960 | 13,600 | Then the slow part was always a rough cut. |
| 3 | 22-37 | 13,980 | 20,000 | Then I checked my workflow and the rough cut went from 90 minutes to 10 minutes. |

Numbers: `3000 hours` (8-9), `90 minutes` (33-34), `10 minutes.` (36-37). Events: NumberReveal `3,000 hours`, FromTo `90 minutes` to `10 minutes`.

`speech_30s_sparse.mp4`, 16 words in 5 sentences, no number, no event:

| # | Words | Starts | Ends | Text |
|---|---|---|---|---|
| 1 | 0-3 | 580 | 5,880 | I added it up |
| 2 | 4-7 | 7,280 | 8,760 | Change gave me my |
| 3 | 8-11 | 13,740 | 15,280 | Record a clip drop |
| 4 | 12-13 | 19,920 | 20,960 | I publish |
| 5 | 14-15 | 26,460 | 27,920 | Ideas matter |

## The V3 test clip as recognized

**Clip.** `testclips/v3testclip.mp4`, the founder's recording of 2026-10-13 (D-99): not in git, never uploaded, and never an input of `gen_fixtures.sh`.

**The founder's words** (given on 2026-10-17, sentence by sentence, as the founder wrote them down):

1. Hello everyone, this is Mayan Pathak, the site.
2. This is just a recording, sample recording for everything that I'm going to do with my new build that I've built, which is offcut, right.
3. So here's a little bit about myself.
4. My name is Mayan, I'm an aspiring computer science engineer and a SaaS developer.
5. My major goal is to develop one of my SaaS that I launch it, get 500 paying users by the end of three months, right.
6. And the target is to have about 500 paying customers and $2,000 in revenue in three to four months of launch.
7. I'll see what I can do.
8. The first plan is to generate content, right.
9. A lot of content, talking head videos, and then eventually reaching out to creators to try my new product.
10. And then from that feedback, after I will take that feedback, I will also, you know, improve the product further.
11. Yes, so that's a plan.
12. Let's see what happens.

**Expected events** (what a person expects from these words; `fixtures/labeled/a02-v3testclip.json`, frozen 2026-10-17):

| # | Kind | Words | Expected content |
|---|---|---|---|
| 1 | NumberReveal | "500 paying users" | value 500, users |
| 2 | NumberReveal | "500 paying customers" | value 500, customers |
| 3 | NumberReveal | "$2,000" | value 2,000, USD, label "in revenue" |

No event for "three months", for "three to four months" (a range, not a change from one value to another), or for "The first plan" (not a list).

**Recognized transcript** (read on 2026-10-18, Prompt 82: one run in `pnpm dev` on the development machine, the transcript read from IndexedDB by the clip's id). Accepted as 78,914 ms, 1280x720, landscape, variable frame rate; `ready` after 88 s. 159 words in 12 sentences; words are counted from 0, times in milliseconds from the first frame:

| # | Words | Starts | Ends | Text |
|---|---|---|---|---|
| 1 | 0-7 | 2,080 | 5,000 | Hello everyone, this is Mayan Bhattag, the site. |
| 2 | 8-28 | 5,720 | 13,600 | This is just a sample recording for everything that I'm doing with my new little bit of build, just off cut. |
| 3 | 29-35 | 15,420 | 17,680 | So here's a little bit about myself. |
| 4 | 36-49 | 17,780 | 22,660 | My name is Mayan, I'm an aspiring computer science engineer and a SAS developer. |
| 5 | 50-73 | 22,980 | 32,560 | My major goal is to develop one of my SaaS that I launch it, get 500 paying users by the end of three months, |
| 6 | 74-74 | 33,380 | 33,500 | right? |
| 7 | 75-96 | 34,000 | 48,120 | And the target is to have about 500 paying customers and $2 ,000 in revenue in three to four months of launch. |
| 8 | 97-102 | 48,620 | 49,700 | I'll see what I can do. |
| 9 | 103-110 | 49,920 | 53,860 | The first plan is to generate content, right? |
| 10 | 111-149 | 54,080 | 75,780 | A lot of content, talking head videos and then eventually reaching out to creators to try my new product and then from that feedback after I will take that feedback I will also you know improve the product further. |
| 11 | 150-154 | 75,840 | 77,580 | Yes, So that's the plan. |
| 12 | 155-158 | 77,580 | 78,520 | Let's see what happens. |

**Where it differs from the founder's words.** "Mayan Pathak" is heard as "Mayan Bhattag"; sentence 2 as "… a sample recording for everything that I'm doing with my new little bit of build, just off cut." (said: "… a recording, sample recording for everything that I'm going to do with my new build that I've built, which is offcut, right."); "SaaS developer" as "SAS developer"; the "right" that ends the founder's sentence 5 is a sentence of its own (6); "$2,000" is written as two words, `$2` and `,000` (as the reference clip's `$12 ,000`, D-42), read as one quantity; "Yes, so that's a plan." as "Yes, So that's the plan."; the founder's sentences 9 and 10 are one (10).

**Numbers** (`Transcript.numbers`): `one` (word 56, `1`), `500` (65), `three months,` (72-73, `3 months`), `500` (82), `$2 ,000` (86-87, `$2k`), `three` (91, `3`), `four months` (93-94, `4 months`).

**Events V3 detects**, in time order: NumberReveal `3 months` (72-73); NumberReveal `$2k` (86-87); NumberReveal `4 months` (93-94); KeywordPop `that's` (152; pitch_z 1.68). Against the expected events above: `$2k` is the third expected event; **"500 paying users" and "500 paying customers" are not found**: a count noun counts as a unit only right after its number (D-101), and here "paying" stands between, so each `500` is a bare number (0.70, under the threshold); **"3 months", "4 months" and "that's" are events a person would not expect**: a span of months scores as a time quantity, the range "three to four months" is not a change, so its second end stands alone, and "that's" carries the pitch of the sentence's stress. These are recorded, not tuned to (D-87, §20.1); `a02-v3testclip.json` keeps its labels, and they meet this transcript at Prompt 88.

**Expected feed** (the E2E case of Prompt 85; local only, D-99): "Cleaning voice" and "Transcribing…", in either order; then "Found: 3 months", "Found: $2k", "Found: 4 months", "Found: that's". The KeywordPop depends on the measured prosody and may be absent on another run of the model.
