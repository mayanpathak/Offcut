# Speech fixtures: reading scripts

The committed speech clips are recorded by the founder reading the scripts below (`technicalspec.md` §26, "Fixture matrix"). Each section gives the script, how to read it, and the transcript and events the pipeline is expected to produce.

The scripts are written against the detector rules of `technicalspec.md` §17.2-§17.5. Changing a word can add or remove an event, so re-check the "Why it is worded this way" notes before editing a script.

| Clip | Script | Recorded for |
|---|---|---|
| `testclips/speech_scriptA_landscape_720p.mp4` (not in git) | Script A | V2: the reference clip. A webcam recording, 1280x720, 74.7 s (`docs/v2/v2implementation.md`, D-39 and D-64). Video-stream duration: **74,705 ms** (`ffprobe`, 74.705033 s, read on 2026-10-08). H.264 and AAC at 48 kHz stereo, 35,201,023 bytes. White-flash frames: 150 and 1950 |
| `speech_60s_portrait.mp4` | Script A | V3 |
| `speech_60s_landscape.mp4` | Script A | V3 |
| `speech_20s_noevents.mp4` | Script B | V3 |
| `speech_20s_long_pause.mp4` | Script C | V3 |
| `speech_30s_sparse.mp4` | Script D | V3 |

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
