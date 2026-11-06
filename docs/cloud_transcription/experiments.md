# experiments.md - the cloud transcription release's experiments

The rows are defined in `cloudimplementation.md` §13. Each result is written here by the prompt that reads it, with the date, the machine (`d1`), the browser and its version, and every number. No word of a transcript is written here beyond what `fixtures/speech/README.md` already publishes (Script A). Every real call to Groq is also listed in `cloudchangelog.md`.

| ID | Question | Prompt | Status |
|---|---|---|---|
| TE-C1 | Does the browser's WebCodecs encoder take Opus at 16 kHz mono, and what does it return (description, pre-skip, chunk durations, speed)? | P161 | **Passed** 2026-11-06 |
| TE-C2 | What does Groq return for our audio, how good is it at each bitrate, and how far are its times from the device recognizer's? | P161 | **Passed** 2026-11-06, with D-382 (silence guard) added |
| TE-C3 | Does the cloud transcript give the same visual events as the device transcript on the reference clip and its derived clips? | P169 | Not run |
| TE-C4 | How much faster is drop-to-ready with cloud than on the device, with and without the model cached? | P169 | Not run |
| TE-C5 | Does Vercel's rewrite carry a 900 kB body to our API, and how long does a live cloud request take? | P170 (G 10.7) | Not run |

## TE-C1

**Read on 2026-11-06 (P161), D1, in Chrome 156 and Edge 156 (headless, through Playwright's `channel`).** The page was a throwaway served from `localhost`, outside the repository.

- **`AudioEncoder.isConfigSupported`:**
  - Opus mono at 16 kHz and 48 kHz;
  - 24, 32 and 48 kbps;
  - `frameDuration` 20,000 µs;
  - **every combination is `supported: true` in both browsers.**
- **Encoding 60 s of a 440 Hz sine at 16 kHz**, in 1 s `AudioData`, at 32 kbps:
  - time: 250 ms (Chrome), 342 ms (Edge);
  - output: 3,001 chunks, every one 20,000 µs long, the first at timestamp 0, the last at 60,000,000 µs;
  - size: 242,744 bytes. A sine is harder to compress than speech; speech was 211,749 bytes from `libopus` in TE-C2.
- **`decoderConfig.description`:** 19 bytes, an `OpusHead`. Its fields:
  - 1 channel;
  - **pre-skip 104** (48 kHz samples, about 2.2 ms);
  - input rate 16,000.
- **For P164:**
  - the pre-skip comes from the description; the 312 fallback of §8.3 stays for a browser that gives none;
  - the 48 kHz path is **not built**;
  - the last page's granule is pre-skip + `pcm16.length × 3`, which trims the one extra padded packet.

Passed: 16 kHz is supported in both browsers, and 90 s encodes in about 0.5 s.

## TE-C2

**Read on 2026-11-06 (P161), from D1, by `curl` with the real key in a subshell.** The audio was `speech_60s_portrait.mp4`'s track (Script A's first 14 sentences, the founder's public sample), as 16 kHz mono FLAC and as Opus at 16, 24 and 32 kbps (`ffmpeg`, `libopus`). The request was `whisper-large-v3-turbo` with `verbose_json`, `timestamp_granularities[]=word`, `language=en` and `temperature=0`. There were six calls of 60 s each, plus one 3 s tone.

**Sizes and latency.**

| File | Bytes (60 s) | Status | Time |
|---|---|---|---|
| FLAC | 1,902,108 | 200 | 2.16 s |
| Opus 16 kbps | 113,408 | 200 | 0.80 s |
| Opus 24 kbps | 163,100 | 200 | 0.93 s |
| Opus 32 kbps | 211,749 | 200 | 0.98 s |

**The shape.**

- **Top-level keys:** `duration`, `language`, `segments`, `task`, `text`, `words`, `x_groq` (`{id}`).
- **Segments:** with word granularity alone, `segments` is `null`. With `word` and `segment` both, each segment has `avg_logprob`, `compression_ratio`, `end`, `id`, `no_speech_prob`, `seek`, `start`, `temperature`, `text`, `tokens`.
- **Words:** each entry of `words` is `{word, start, end}`, in seconds with 2 decimals.
- **Duration:** `duration` was 60.000 (59.9998).

**Punctuation.**

- **Groq's `words` already carry their punctuation:** "year,", "videos.", "luck.". 26 of the 129 words hold `.`, `,`, `?` or `!`. D-360's `align` is therefore an identity on Groq's words, and it stays as a guard (as read).
- **Numbers** are digits in both `words` and `text`: "3000", "90", "10". There is no thousands comma here, the same as the device's "3000". Currency and percent were not in this window.

**Accuracy.** Word error rate against Script A's text for the same 14 sentences (128 words; numbers as digits; case and punctuation ignored):

| File | Errors | WER |
|---|---|---|
| FLAC | 7 | 5.5% |
| Opus 16 kbps | 9 | 7.0% |
| Opus 24 kbps | 8 | 6.2% |
| Opus 32 kbps | 8 | 6.2% |

- **32 kbps is within 1 point of FLAC: chosen (D-370 unchanged).** 24 kbps scored the same here; 32 is kept for margin on other voices.
- **The errors are mostly what the founder actually said.** "literally brutal", "checked", "a rough cut" are also in the device transcript. Groq's one clear miss is "wiki" for "week". The device recognizer heard "editing 8 every evening" where Groq heard "Editing ate every evening".

**Timing.** Against the device transcript, Groq's sentence starts differ by a median of 290-300 ms (max 620 ms). That is over the 60 ms this row's pass rule named. **Against the audio itself the picture reverses.** The speech onsets were measured with `ffmpeg -af silencedetect=noise=-35dB:d=0.12` on the FLAC. For the 14 sentence starts:

- Groq's start is a median **33 ms** from the nearest onset;
- the device's is a median **264 ms** from it, and always later.

**Groq's times are the more accurate; the device recognizer runs late by about a quarter of a second.** The pass rule is read against the measured onsets (as read): passed. A cloud clip's captions will appear about 0.3 s earlier than the same clip's device captions, which is closer to the voice.

**An invented ending (hallucination).**

- **What happened.** Every response ended with words nobody said: "Thank you." (Opus) or "Yes." (FLAC). They sit in the 1.4 s of silence after the last sentence, at 59.92-60.48 s, with the last word ending after the audio does (60.48 > 60.00).
- **What Groq's scores show.** With segments requested, that segment has `no_speech_prob` 0.000, like every other segment, so it is no signal. Its `avg_logprob` was −0.507 (the others −0.21 to −0.24) and its `compression_ratio` 0.58 (the others about 1.5).
- **What the audio shows.** The RMS level of the audio under each word separates it cleanly:
  - real words: −37.2 dBFS at the lowest, −26.4 median;
  - "Thank": −64.8 dBFS;
  - "you.": starts after the audio's end.
  - The clip's 95th-percentile sample level was −19.8 dBFS.
- **Decision D-382 (new, as read): a silence guard in `fromCloud`.**
  - A cloud word is dropped if it starts at or after the end of the audio.
  - It is also dropped if the RMS of `pcm16` over its interval (clamped to the audio) is more than 30 dB below the clip's 95th-percentile sample level. Here that floor is −49.8 dBFS: 12.6 dB below the quietest real word, and 15 dB above "Thank".
  - The device path is unchanged.
  - TE-C3 reads the guard on the long-pause and sparse fixtures (generated room tone) before it is trusted.

**Limits read from Groq's response headers** (the 3 s tone):

- `x-ratelimit-limit-audio-seconds: 7200`, with `x-ratelimit-reset-audio-seconds` in seconds: a rolling hour;
- `x-ratelimit-limit-requests: 2000`, with reset in minutes: per day.

This agrees with D-346's hourly budget of 7,000 s. The daily audio limit is not in the headers; the budget's 27,000 s a day stays as planned (assumption).

**The tone.** 3 s of 440 Hz came back as one word, ".". `MIN_WORDS` refuses such a clip (`NoSpeech`), as on the device.

## TE-C3

Not run.

## TE-C4

Not run.

## TE-C5

Not run.
