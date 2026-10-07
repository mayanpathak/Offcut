# Speech fixtures: reading scripts

The committed speech clips are recorded by the founder reading the scripts below (`technicalspec.md` §26, "Fixture matrix"). Each section gives the script, how to read it, and the transcript and events the pipeline is expected to produce.

The scripts are written against the detector rules of `technicalspec.md` §17.2-§17.5. Changing a word can add or remove an event, so re-check the "Why it is worded this way" notes before editing a script.

| Clip | Script | Recorded for |
|---|---|---|
| `speech_60s_portrait.mp4` | Script A | V2 |
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

The exact display strings come from `format_quantity` (`technicalspec.md` §17.2), which is written in V2. Fill them in here when the reference clip is first run through the pipeline.

**Why it is worded this way.**

- Each NumberReveal carries a unit or a currency and a magnitude of at least 1,000. That scores 1.0 before ASR confidence is applied, against a threshold of 0.80.
- Each NumberReveal sits in a sentence with no other quantity, so FromTo does not consume it.
- The FromTo uses the "from … to" frame, the same unit on both values and a one-word gap. That scores 1.0 against a threshold of 0.85.
- The list uses the announcer form ("three reasons") with three ordinal-word markers. Each item is five words or fewer, the limit of `LIST_ITEM_MAX_WORDS`.
- "BRUTAL" follows the intensifier "really". It still needs the louder, higher delivery to pass 0.80.
- The script contains no other number words, no years, no clock times, no version numbers, and no "first", "second" or "third" outside the list.
- At least two filler sentences separate neighbouring overlay events, so their display windows do not overlap (§17.5).

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
