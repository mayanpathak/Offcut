# experiments.md - outcome of every experiment V3 runs (v3implementation §20)

Every reading is taken on D1 and recorded under the label `d1` (D-74). One section per row of §20.1, filled when the experiment is run.

## TE-13 native
Question: does the voice chain (`run_chain`, denoise on) run natively in under 2 s per 60 s of audio, the early warning before the browser reading of Prompt 69 (§20.1, G 3.8)?
Method: `chain_timing` of `crates/offcut-dsp/tests/chain_no_clicks.rs`, release build, five runs, on `speech_60s_portrait.mp4.pcm48.f32` (60.0 s, 2,879,992 samples).
Numbers: 0.643, 0.655, 0.707, 0.722, 0.775 s; **median 0.707 s per 60 s** (`d1`). Also read: `measure_prosody` on 60 s with 150 words, 0.148 s in release (budget 2 s, TS §8's 1 s for the reference clip).
Date: 2026-10-17, D1 (`d1`).
Decision: continue. Under 2 s by a factor of 2.8; denoise stays on (`ChainConfig::default()` keeps `denoise: true`). The browser reading (TE-13 browser, at most 4 s per 60 s) is Prompt 69's. A pass on D1 does not show a pass on an R1-class laptop (D-74).

## TE-13 browser
Question: does the voice chain run in the browser, in the audio worker, in at most 4 s per 60 s of audio on D1, with `out48` exactly as long as the input and on target (PS §20.2, §20.2 of the plan, G Phase 4)?
Method: a temporary Playwright check (Chrome stable, D1): the page fetches `speech_60s_portrait.mp4.pcm48.f32` (2,879,992 samples, 60.0 s), calls `pool.audio.measureProsody` once with nothing kept, then three times `pool.audio.runChain({ pcm48 }, { transfer: [pcm48.buffer] })` timed with `performance.now()` from the call to its answer, each followed by `measureProsody({ words: [], sentences: [] })` to take `out48` back. In dev (the Vite dev server, cross-origin isolated by its headers, `/src/workers/pool.ts` imported in the page) and in the production build (`vite preview` with the headers of `vercel.json`, the pool reached through a temporary module imported from `main.tsx`, removed after).
Numbers: dev 866, 705, 733 ms, **median 733 ms per 60 s**; production build 855, 738, 828 ms, median 828 ms (13% over dev; `crossOriginIsolated` true). Every run: `out48.length === pcm48.length` (2,879,992), the input buffer detached by the transfer, loudness -14.30 LUFS. The first `measureProsody` failed with `E_INTERNAL` / `NoAudio`. The worker and its module were started by that first, failing call, so no timed run includes compiling the module; each includes the copy into and out of WASM memory (D-110) and the transfer.
Date: 2026-10-17, D1 (`d1`).
Decision: continue. At most 4,000 ms; the browser is about 1.1 times the native 0.707 s (TE-13 native), not the 1.5-2.5 the plan assumed. Denoise stays on. A pass on D1 does not show a pass on an R1-class laptop (D-74).

Question: (S12 reading, Prompt 88) does the bench confirm S6's decision (§20.2)?
Method: the `audio_chain` stage of `pnpm bench:device`, ten measured runs after one warm-up, on the production build (`E-3 re-read` has the details). This is the app's own stage time, from the chain's start to its answer. The chain runs beside the model wait and the recognizer (D-90), so it shares the machine with them, which S6's call did not.
Numbers (`d1`):
- portrait (`bench/results/d1-2026-10-19-portrait.json`): median **1,963 ms**, p90 2,393, from 703 to 2,422;
- `v3testclip` (`bench/results/d1-2026-10-19-v3testclip.json`): median 1,111 ms, p90 1,143, **844 ms per 60 s** (its budget is 5.26 s for 78.9 s).
Date: 2026-10-19, D1 (`d1`).
Decision: **S6's decision confirmed: denoise stays on.** Both readings are under the 4 s per 60 s budget, the portrait one by a factor of 2. The spread of the portrait runs (703 to 2,422 ms) is the chain sharing the machine with the recognizer. A pass on D1 does not show a pass on an R1-class laptop (D-74).

## DENOISE_DELAY_SAMPLES
Question: by how many samples does RNNoise (`nnnoiseless` 0.5.2) delay its output, so that `denoise.rs` can align the wet and dry paths and drop exactly that many samples (P2, INV-10)?
Method: `impulses_stay_aligned` of `crates/offcut-dsp/tests/duration_preserved.rs`: 19 impulses of 0.5 on a -40 dBFS noise floor, one every 0.5 s over 10 s; for each, the position of the largest output sample within 2,400 samples of the input impulse, for the denoiser alone at mix 1.0 (all wet, so the dry path cannot hide a misaligned wet one) and for the whole chain. Read with the constant at its planned 480, and again at 0.
Numbers: at 480, all 19 offsets are 0, for the denoiser alone and for the chain; at 0 (the drill), all 19 are +480. The delay is one RNNoise frame, exactly, and the same for every impulse.
Date: 2026-10-17, D1 (native, debug build).
Decision: `DENOISE_DELAY_SAMPLES = 480`, the measured value; denoise stays on (`ChainConfig::default()` keeps `denoise: true`). Native timing on the way: `run_chain` on the 60 s portrait speech PCM took 0.607-0.617 s in a release build (three runs, `chain_timing`); TE-13 is read in Prompt 68.

## Precision gate
Question: is the detector's overall precision at least 0.90 (`DETECTOR_PRECISION_TARGET`, PS §20.1) over at least 30 labeled transcripts written before its rules (D-87), at the default thresholds (§20.3, G 8.3)?
Method: `cargo test -p offcut-detect --test precision_recall -- --nocapture` (`crates/offcut-detect/tests/precision_recall.rs`, §19.12): every file of `fixtures/labeled/`; written files timed by §17.5's synthesis, `edits` applied; `detect` with `DetectorConfig::default()`; greedy matching per file and kind (IoU of the anchors at least 0.5, equal values and units). The sweep moves one kind's threshold at a time, the others at their defaults, and runs `detect` again at each step, so `resolve` sees what that threshold lets through. Default thresholds: NumberReveal 0.80, ListReveal 0.85, FromTo 0.85, KeywordPop 0.80.
Numbers (`d1`, default thresholds, printed as run):

```
precision_recall: 30 files read, 1 skipped
a02-v3testclip: skipped (no data yet)

kind            TP  FP  FN  precision  recall
number_reveal   12   3   0       0.80    1.00
list_reveal      8   0   0       1.00    1.00
from_to         11   0   1       1.00    0.92
keyword_pop      9   0   0       1.00    1.00
overall         40   3   1       0.93    0.98

sweep (precision/recall of the kind; its threshold alone moved, defaults 0.80 0.85 0.85 0.80)
threshold  number_reveal  list_reveal  from_to    keyword_pop
0.50       0.75/1.00    1.00/1.00    1.00/1.00    0.06/1.00
0.55       0.75/1.00    1.00/1.00    1.00/1.00    0.75/1.00
0.60       0.75/1.00    1.00/1.00    1.00/1.00    0.75/1.00
0.65       0.75/1.00    1.00/1.00    1.00/1.00    0.75/1.00
0.70       0.75/1.00    1.00/1.00    1.00/1.00    0.75/1.00
0.75       0.80/1.00    1.00/1.00    1.00/0.92    1.00/1.00
0.80       0.80/1.00    1.00/1.00    1.00/0.92    1.00/1.00
0.85       1.00/0.92    1.00/1.00    1.00/0.92    1.00/1.00
0.90       1.00/0.92    1.00/0.62    1.00/0.58    1.00/1.00
0.95       1.00/0.92    1.00/0.50    1.00/0.50    1.00/0.11
1.00       1.00/0.92    1.00/0.50    1.00/0.50    1.00/0.11

false positives (file: kind words (confidence))
w19-editing-workflow: number_reveal "20 minutes" (0.80)
w28-mistakes: number_reveal "40 seconds," (0.80)
w28-mistakes: number_reveal "3 minutes." (0.80)

missed (file: kind "say")
w28-mistakes: from_to "40 seconds, and fixing those pushed it up to 3 minutes"

labels and headers that differ (reported, not matched)
none
```

The false positives by file. `w19`: "about 20 minutes" in a sentence the file labels as no event (a `no_change_pair` trap); a time span of 10 or more scores 0.80, exactly the NumberReveal threshold. `w28`: "pushed it up to" is a change the labels expect as one FromTo; "pushed" is not in `CHANGE_VERBS`, so the pair stays under 0.85 and its two ends are shown as two NumberReveals instead, one miss and two false positives from one sentence. All three sit at 0.80, on the threshold.
Date: 2026-10-18, D1 (`d1`), debug build, 0.86 s for the whole test with its sweep.
Decision: **continue with the default thresholds** (§20.3: overall 0.93 at or above 0.90). No threshold changes; `config.rs` is unchanged. Margin for V4: NumberReveal alone is at 0.80 precision. At 0.85 the sweep gives it 1.00 precision and 0.92 recall (w28's two ends gone, one true NumberReveal lost), so that is the first move if a later reading falls under 0.90. Every kind's recall is at least 0.92. `a02` joins at Prompt 88 and makes 31 files. The gate is read on D1, which has no bearing on R1 (the detector is deterministic and timing-free; D-74 concerns speed only).

Question: (second reading, Prompt 88) with `a02-v3testclip.json` filled from the app (`CORPUS_MODE=transcripts`: 159 recognized words and their prosody, labels unchanged), is overall precision still at least 0.90 over 31 files (§20.3, G 12.3)?
Method: as above, on 31 files. First at the default thresholds; then, as §20.3 requires, again after the contingency.
Numbers (`d1`), at the default thresholds (NumberReveal 0.80):

```
precision_recall: 31 files read, 0 skipped

kind            TP  FP  FN  precision  recall
number_reveal   13   5   2       0.72    0.87
list_reveal      8   0   0       1.00    1.00
from_to         11   0   1       1.00    0.92
keyword_pop      9   1   0       0.90    1.00
overall         41   6   3       0.87    0.93

sweep (precision/recall of the kind; its threshold alone moved, defaults 0.80 0.85 0.85 0.80)
threshold  number_reveal  list_reveal  from_to    keyword_pop
0.50       0.65/0.87    1.00/1.00    0.92/1.00    0.06/1.00
0.55       0.68/0.87    1.00/1.00    0.92/1.00    0.50/1.00
0.60       0.68/0.87    1.00/1.00    0.92/1.00    0.53/1.00
0.65       0.68/0.87    1.00/1.00    0.92/1.00    0.53/1.00
0.70       0.68/0.87    1.00/1.00    1.00/1.00    0.60/1.00
0.75       0.72/0.87    1.00/1.00    1.00/0.92    0.90/1.00
0.80       0.72/0.87    1.00/1.00    1.00/0.92    0.90/1.00
0.85       1.00/0.80    1.00/1.00    1.00/0.92    1.00/1.00
0.90       1.00/0.80    1.00/0.62    1.00/0.58    1.00/1.00
0.95       1.00/0.80    1.00/0.50    1.00/0.50    1.00/0.11
1.00       1.00/0.80    1.00/0.50    1.00/0.50    1.00/0.11

false positives (file: kind words (confidence))
a02-v3testclip: number_reveal words 72-73 (0.80)
a02-v3testclip: number_reveal words 93-94 (0.80)
a02-v3testclip: keyword_pop words 152-152 (0.84)
w19-editing-workflow: number_reveal "20 minutes" (0.80)
w28-mistakes: number_reveal "40 seconds," (0.80)
w28-mistakes: number_reveal "3 minutes." (0.80)

missed (file: kind "say")
a02-v3testclip: number_reveal "500 paying users"
a02-v3testclip: number_reveal "500 paying customers"
w28-mistakes: from_to "40 seconds, and fixing those pushed it up to 3 minutes"

labels and headers that differ (reported, not matched)
a02-v3testclip: number_reveal "$2,000": labeled "in revenue", detected Some("in revenue in")
```

`a02`'s two new NumberReveal false positives are "three months," (words 72-73) and "four months" (93-94), which its labels call no events; its KeywordPop is "that's" (152). Both "500 paying …" are missed because "paying" stands between the number and its count noun (D-101). After the contingency (NumberReveal 0.85, the other three unchanged):

```
kind            TP  FP  FN  precision  recall
number_reveal   12   0   3       1.00    0.80
list_reveal      8   0   0       1.00    1.00
from_to         11   0   1       1.00    0.92
keyword_pop      9   1   0       0.90    1.00
overall         40   1   4       0.98    0.91

false positives: a02-v3testclip: keyword_pop words 152-152 (0.84)
missed: a02 "500 paying users", a02 "500 paying customers", w02-coding-bootcamp "16 weeks", w28 from_to (as before)
```

Date: 2026-10-19, D1 (`d1`), debug build.
Decision: **§20.3's contingency applied: `threshold_number` 0.80 → 0.85** in `crates/offcut-detect/src/config.rs`, the lowest value of the sweep at which NumberReveal reaches 0.90. Overall 0.98, the gate passes on 31 files. Recall after the raise: NumberReveal 0.80, ListReveal 1.00, FromTo 0.92, KeywordPop 1.00, every one at or above 0.30. ListReveal (0.85) and FromTo (0.85) are unchanged. KeywordPop (0.80) is unchanged too: its precision is 0.90, not under it. Its one false positive ("that's", 0.84) is the margin V4 starts from. What the raise costs: a quantity under 1,000 with its unit scores 0.80 and is no longer shown unless said with emphasis (+0.10). Large or suffixed amounts ("3,000 hours", "$12k", "$2k", 1.0) are unaffected. The labeled set loses one true NumberReveal ("16 weeks", `w02`). Among the founder's three script recordings of 2026-10-18 (`Second recording`), "3x", "400 ms" and "120 clients" are such quantities; this reading does not say whether they were emphasised. No label was changed: the founder's five-file reading (next paragraph) answered yes to all five.

**The founder's reading of five labels (G 12.6, D-87), 2026-10-19.** Five files, drawn with `ls fixtures/labeled/w*.json | shuf -n 5`. Each was shown to the founder as its sentences and `expected` list, with the question "would a person expect exactly these events?":

| File | Expected | Answer |
|---|---|---|
| `w22-ai-tool-review` | KeywordPop "excellent"; "GPT 4" no event | Yes |
| `w06-fitness` | ListReveal of 3 (First / Second / Third); "3 sets" no event | Yes |
| `w20-battery-life` | FromTo 11 hours → 6 hours; FromTo 14 minutes → 9 minutes; "two years" no event | Yes |
| `w21-writing-a-book` | NumberReveal "62,000 words"; KeywordPop "better"; "9 months" no event | Yes |
| `w04-newsletter` | NumberReveal "14,000 subscribers"; FromTo 26 percent → 52 percent; "second" no list marker | Yes |

Five yes, no label changed, so the gate above needs no re-run.

## TE-12
Question: does `offcut-mp4` demux the real clips of the corpus, at least 90% of them reaching their expected outcome, with none crashing (TS §15, §20.1; first run)?
Method: `sh corpus/fetch_corpus.sh`, then `CORPUS_MODE=demux pnpm corpus` against `vite preview` of the production build with the test key: each clip of `corpus/manifest.json` present in `corpus/.clips/` is dropped in a context of its own and followed to acceptance, refusal or failure (`corpus/run_corpus.ts`). `OFFCUT_CORPUS_DIR` holds no lent clip, so the two founder clips are the corpus.
Numbers (`d1`): 2 clips present, 2 accepted, 2 matching their `expect` (`accept`): 100%; 0 refused, 0 failed, 0 crashes (`E_INTERNAL`, `E_WORKER_CRASH`). 6.0 s and 5.1 s per clip. The result file is `corpus/.results/2026-10-19-demux.json` (local, git-ignored).
Date: 2026-10-19, D1 (`d1`).
Decision: **TE-12: inconclusive (2 clips)**, re-checked in V9 (D-88); fewer than 10 clips are present. The JS-demuxer contingency is not triggered.

## E-3 re-read
Question: how long does transcription take on D1 in the V3 production build, against the target of 20 s per 60 s of audio and D-67's line of 145 s per 60 s above which the base-size model applies (§20.1, PS §20.2)?
Method: the `asr` stage of `pnpm bench:device` (`bench/device-bench.ts`, one warm-up run then ten measured, each from the drop to the downloaded export, headed Chrome, `vite preview` of the production build with the test key), `BENCH_DEVICE=d1`, once with `BENCH_CLIP=portrait` and once with `BENCH_CLIP=v3testclip`. The `v3testclip` reading is normalised to 60 s (x 60,000 / 78,918, D-74). Files `bench/results/d1-2026-10-19-portrait.json` and `bench/results/d1-2026-10-19-v3testclip.json`.
Numbers (`d1`, Chrome 155.0.8059.82, `webgpu` in every run):
- portrait (60.0 s): median **69,469 ms**, p90 69,936, from 51,917 to 72,308;
- `v3testclip` (78.9 s): median 62,729 ms, p90 63,301, **47,692 ms per 60 s**;
- whole runs (pipeline and export): portrait median 102,690 ms, p90 103,290; `v3testclip` median 93,220 ms (70,874 per 60 s), p90 93,628;
- peak page memory: 1,088,978,386 and 1,169,350,544 bytes.
The bench stamps commit `c838c7f`; its build also held this prompt's uncommitted threshold change, which does not touch the recognizer.
Date: 2026-10-19, D1 (`d1`).
Decision: **continue, the target recorded as missed** (as in V2, D-67). Both readings are 3.5 and 2.4 times the 20 s target and well under the 145 s line, so the base-size model is not needed. Per 60 s the founder's recording is faster than the portrait clip. That is not investigated: the clips differ in video size (1080x1920 against 1280x720) and in how much of them is speech. A pass on D1 does not show a pass on an R1-class laptop (D-74): V2's one R1 reading was about 120 s per 60 s (D-67).

## E-4 re-read
Question: how long do render and encode take on D1 in the V3 production build, against 90 s per 60 s (PS §20.2), with 150 s per 60 s the line for the Canvas2D path (§20.1)?
Method: the `render_encode` and `mux` stages of the same two bench runs (`E-3 re-read`), Creator export (1080x1920).
Numbers (`d1`):
- `render_encode`: portrait median **30,848 ms**, p90 31,209; `v3testclip` median 29,010 ms, p90 29,182, **22,055 ms per 60 s**;
- `mux`: portrait median 16 ms, `v3testclip` 5.5 ms (at most 2 s);
- also read: `detect_scene` 195 ms (portrait) and 116 ms (88 per 60 s), and `probe_audio` 875 ms and 737 ms (560 per 60 s), against the 3 s of TS §15.
Date: 2026-10-19, D1 (`d1`).
Decision: **continue**: under 90 s per 60 s by a factor of about 3; the Canvas2D path is not needed. A pass on D1 does not show a pass on an R1-class laptop (D-74).

## E-10 smoke
Question: what word error rate does the recognizer give on the founder's two recordings, as a smoke reading of the `wer` mode (§20.1; not E-10)?
Method: `CORPUS_MODE=wer pnpm corpus`, with local reference texts that are never committed: `corpus/.clips/founder-01.txt`, Script A as written in `fixtures/speech/README.md` (159 words); `corpus/.clips/founder-02.txt`, the founder's words of `~/v3testclip-words.txt` (Prompt 65). Both sides are normalised as §17.6 says and aligned word by word (Levenshtein); counts only are kept.
Numbers (`d1`):

| Clip | Reference words | Recognized words | Substitutions | Deletions | Insertions | WER |
|---|---|---|---|---|---|---|
| `founder-01` | 159 | 157 | 13 | 3 | 1 | 0.1069 |
| `founder-02` | 169 | 165 | 12 | 5 | 1 | 0.1065 |

Script A spells its numbers ("three thousand hours"); the recognizer writes digits ("3,000"). Each such number counts as substitutions, so `founder-01`'s rate is above what a reader would call errors.
Date: 2026-10-19, D1 (`d1`).
Decision: **smoke, not E-10 (carried to V9, D-88).** Recorded, not judged: two clips by one speaker are not the 20-clip accent set. Both are under E-10's 12% line, which says nothing about E-10.

## E-10 (carried to V9, D-88)
Question: is the recognizer's word error rate at most 12% over 20 clips of English speakers with different accents, 10 US and 10 Indian or other (PS §19, TS §26)?
Method: `CORPUS_MODE=wer pnpm corpus` over the 20-clip accent set, each clip with its reference text in `corpus/.clips/<id>.txt` (never committed). Built in Prompt 86; the set does not exist.
Numbers: 
Date: not run in V3.
Decision: carried to V9 (D-88): no 20-clip accent set exists, and the founder records none in V3. The reading above (`E-10 smoke`) is two clips by one speaker and is not E-10.

## rej_h264_444 in Chrome
Question: does Chrome refuse to decode H.264 High 4:4:4 Predictive, so that `rej_h264_444.mp4` tests `REJECT_DECODE_UNSUPPORTED` with a real file (D-78)?
Method: `ffprobe` on the generated file; then `VideoDecoder.isConfigSupported({ codec, codedWidth: 1280, codedHeight: 720 })` in Chrome stable through Playwright on a page of the app's origin, for `avc1.f4001f` (High 4:4:4, level 3.1), D-78's fallback `avc1.6e001f` (High 10) and `avc1.64001f` (High) as a control.
Numbers: the file is `High 4:4:4 Predictive`, level 31. Chrome answers `supported: true` for all three. On CI's Windows runner (job `e2e-media / media` of run 39217492169, on `32da057`) the case asked the runner's Chrome the same question for `avc1.f4001f`, was told `true`, and marked itself `test.fixme`: the job read "24 passed, 1 skipped", the skip being this case.
Date: 2026-10-17, D1; 2026-10-19, CI's Windows runner (Prompt 87).
Decision: D-78's last branch: both files would decode, so the `REJECT_DECODE_UNSUPPORTED` case of `rejections.spec.ts` is written as `test.fixme` with this reason (Prompt 84), and the code path is covered by the inline unit test of `validate_probe`. The fixture stays, unchanged: `probe_rejections.rs` uses it natively with `decode_supported = false`. No `rej_h264_hi10.mp4` is made (it would decode too). The Windows runner's Chrome decodes the file as well (Prompt 87), so no Chrome this project runs today refuses it, and the case runs its assertions nowhere yet. It runs them by itself on a Chrome that refuses the file.

## rej_silent
Question: does the recognizer give fewer than `MIN_WORDS` (3) words for 5 s of digital silence, so that `rej_silent.mp4` is refused with `REJECT_NO_SPEECH` (D-111)?
Method: the file dropped in the V2 app (`vite preview` of the current build, model cached in OPFS), through Playwright on D1.
Numbers: the app showed the rejected state (`clip-rejected`).
Date: 2026-10-17, D1.
Decision: the fixture stays at 5 s.

## Derived clips' events
Question: (first reading, Prompt 68) does the measured prosody of "brutal" in the reference speech carry the KeywordPop of Script A, which V3's expected events give as conditional ("if the measured prosody carries it", `fixtures/speech/README.md`, §17.7)?
Method: `the_brutal_reading_of_the_portrait_speech` of `crates/offcut-dsp/tests/prosody.rs`: `measure_prosody` on `speech_60s_portrait.mp4.pcm48.f32` (the uncleaned derived PCM) with sentences 3 and 4 of "The V2 reference clip as recognized", moved 960 ms earlier for the portrait cut. The README gives sentence times only, so the words share their sentence's time by their letters: an approximation, printed and not asserted. The reading with the recognizer's word times on the cleaned voice comes at S8 (Prompt 82).
Numbers: sentence 4 ("This is literally brutal way to make anything."): "literally" energy_z +0.45, pitch_z +1.79; **"brutal" energy_z +0.91, pitch_z +1.26**; the others between -2.12 and +0.91 in energy and -0.95 and +0.50 in pitch. Sentence 3: "3000" energy_z +1.24, pitch_z +0.83.
Date: 2026-10-17, D1.
Decision: none yet. The pitch rise on "literally brutal" is measurable; whether "brutal" passes KeywordPop is read at S8 with real word times and the detector of Prompt 77.

Question: (S8 reading, Prompt 82) do the five derived clips give the events of §17.3 through the whole V3 pipeline (the chain, the recognizer on the cleaned voice, prosody, the detector)?
Method: each clip dropped once in `pnpm dev` on D1 (Chrome, the model on WebGPU, a persistent profile), waited for `ready`, the transcript read from IndexedDB by the clip's id, the feed recorded from before the page loaded, the analytics batches read from the in-page fake API. A temporary Playwright file, deleted after.
Numbers (`d1`):

| Clip | `ready` after | `audio_chain` | `asr` (`webgpu`, dev build) | Feed's "Found:" lines | `pipeline_done` counts (number, list, from_to, keyword) |
|---|---|---|---|---|---|
| `speech_60s_portrait.mp4` | 65.4 s | 1,141 ms | 61,808 ms | "3,000 hours", "90 minutes to 10 minutes" | 1, 0, 1, 0 |
| `speech_60s_landscape.mp4` | 60.1 s | 900 ms | 58,240 ms | the same | 1, 0, 1, 0 |
| `speech_20s_noevents.mp4` | 20.7 s | 431 ms | 19,606 ms | none | 0, 0, 0, 0 |
| `speech_20s_long_pause.mp4` | 21.3 s | 728 ms | 20,069 ms | "3,000 hours", "90 minutes to 10 minutes" | 1, 0, 1, 0 |
| `speech_30s_sparse.mp4` | 16.3 s | 962 ms | 14,782 ms | none | 0, 0, 0, 0 |

"brutal" on the cleaned voice with the recognizer's word times: energy_z +0.56, pitch_z +0.57 (both clips alike); "This", which opens its sentence, has pitch_z +2.21. No KeywordPop on any derived clip. The 60 s clips have no ListReveal: the recognizer writes the three items as one sentence ("First the captions write themselves, second the numbers animate automatically, third everything runs on my laptop."), so "second" and "third" are not at a sentence start (D-105) and "three reasons" is a partial list. The long-pause clip's second part starts at 9,960 ms, 40 ms before the 10,000 ms of §17.3. Each difference is written into `fixtures/speech/README.md` with its reason.
Date: 2026-10-18, D1 (`d1`).
Decision: the README's expectations follow the clips as recognized; the detector is not changed to fit (§20.1). V4 input: on the derived 60 s clips the list depends on how the recognizer punctuates three short items said in a row, and the KeywordPop on "brutal" is not carried by the measured prosody; Prompt 85's E2E cases take the README as written.

## Second recording
Question: is the founder's second recording, `testclips/v3testclip.mp4` (78.9 s, 1280x720, variable frame rate), accepted and taken to `ready` by the V3 pipeline, and which events does V3 find in it (§20.1, D-99)?
Method: as for the derived clips (above): one run in `pnpm dev` on D1, the transcript read from IndexedDB by its clip id. Its recognized sentences, numbers and events are in `fixtures/speech/README.md`, "The V3 test clip as recognized". Nothing about the file's location tag is written anywhere (D-100).
Numbers (`d1`): accepted (`clip_accepted`, `lte90`, landscape); `ready` after 88.2 s; `audio_chain` 2,180 ms, `asr` 84,350 ms (`webgpu`, dev build); `pipeline_done` 86,453 ms with 3 NumberReveals, 0 lists, 0 FromTo, 1 KeywordPop; 159 words, 159 prosody entries. Feed: "Cleaning voice", "Transcribing…", "Found: 3 months", "Found: $2k", "Found: 4 months", "Found: that's". Against the frozen labels of `a02-v3testclip.json`: `$2k` matches; "500 paying users" and "500 paying customers" are missed (the count noun is not right after its number, D-101); "3 months", "4 months" and "that's" are events the labels do not expect.

Further readings (2026-10-18), three more recordings by the founder of that day, read from `docs/v3/testvideo_scripts/` (saved as `testclips/v3testclip1 (1).mp4`, `v3testclip2 (2).mp4`, `v3testclip3 (3).mp4`, all 1280x720 webcam, variable frame rate; each carries a location tag and stays on D1). Run the same way; all three accepted and `ready`. Events only; their transcripts are not committed (D-112 names the founder's two clips):

| File | Script | Length; `ready` after | Found | Expected by the script, not found | Found, not expected |
|---|---|---|---|---|---|
| `v3testclip3 (3).mp4` | 1 (all four kinds) | 61.3 s; 73.3 s | 10,000 subscribers; 6 hours to 45 minutes; 4-item list; 3x | $80 (inside the list's last item: the recognizer runs "Fourth a fixed upload day the microphone costs $80 …" into one sentence, and a list holds its words); 40% (written `40 %`, two words: the `%` is not read as the number's unit); UNREAL, ENOUGH | none |
| `v3testclip1 (1).mp4` | 2 (the traps) | 68.8 s; 80.4 s | 400 ms; $240 to $60 | 1.5 million views (see the next column); the marker-only list (the recognizer runs "… here is what I learned first ship small second measure everything third …" into the sentence before); WHOLE | **"5 million views"**: the recognizer writes `1 .5 million`, and the number grammar reads `.5` as a new token, so 1.5 million is shown as 5 million |
| `v3testclip2 (2).mp4` | 3 (noise, pause) | 45.2 s; 57.7 s | ₹50k; 6 months to 2 weeks; 120 clients | 250 GB (written `250GB`, one word); CHEAPEST | **"7 minutes"**: written `4 .7 minutes`, read as 4 and 7 minutes, so 4.7 minutes is shown as 7 minutes |

No trap of script 2 or script 1 became an event: the versions (`3 .11`, `3 .12`, `iPhone 15 Pro`, "version two"), the years (`2019`, `2024`), the phone digits, the clock times (`9 a .m.`, `7 PM`), the small counts and "ten pounds".
Date: 2026-10-18, D1 (`d1`).
Decision: **accepted, `ready`: the second recording passes (§20.1).** Recorded, not tuned to (D-87, §20.1): the founder decides whether the two wrong values (a decimal written with its point as a separate word) are fixed in V3 or in V4; see `v3changelog.md`, Prompt 82's entry. `a02`'s labels stay as frozen and meet this transcript at Prompt 88, where its three unexpected events count as false positives of the precision gate.

Question: (S12 reading, Prompt 88) does the second recording still pass end to end on the finished V3 build, with the NumberReveal threshold the precision gate raised (G 12.2)?
Method: `pnpm e2e:media` on D1 against the production build with the test key, in the full run (49 of 50 cases, which include the two D1-only cases); and the bench's eleven runs of the clip (`E-3 re-read`).
Numbers (`d1`):
- "second recording": passed, 1.8 min. Accepted (`lte90`, landscape), `ready`; the feed is the README's, now "Found: $2k" and "Found: that's" (`3 months` and `4 months` score 0.80, under the raised threshold, `Precision gate`); the preview plays. The recognizer ran on `webgpu` (80,623 ms), the voice chain took 1,612 ms.
- "no metadata": passed, 2.0 min. The export carries no location tag, no source `creation_time`, `encoder` or `©xyz`, and the verifier passes checks 1-6.
- The bench took the clip through eleven pipelines and eleven Creator exports with no failure.
Date: 2026-10-19, D1 (`d1`).
Decision: **passes on the finished build (§20.1).**

## Bundle size
Question: how much does the voice chain (`nnnoiseless`, `ebur128` and `offcut-dsp`'s chain) add to `offcut_core_bg-*.wasm` (G 4.4; growth over 2 MB is the signal to find the crate)?
Method: `pnpm build`, the size of `web/dist/assets/offcut_core_bg-*.wasm`, against Entry 1's baseline.
Numbers: 459,281 bytes at the baseline (V2); 698,122 bytes with `audio_api.rs` exporting `run_chain` and `measure_prosody` (+238,841 bytes, +52%). `offcut_render_bg` unchanged at 6,033,574. The new `audio.worker` chunk: 15,146 bytes. `check-hosts.mjs` finds no new URL literal (the bundle's only host is still the asset host).
Date: 2026-10-17, D1.
Decision: accepted; no crate to look into (well under 2 MB of growth). No entry in `NON_NETWORK_LITERALS`.
