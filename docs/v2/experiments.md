# experiments.md - Offcut V2: experiment records

**What this is.** The outcome of every experiment V2 runs: the technical experiments TE-n of `technicalspec.md` §37 and the product experiments E-n of `product.md` §19. The procedures and thresholds are in `v2implementation.md` §24.1. The M0 gate decision (§24.3) is written at the end of this file.

**How it is kept.** One entry per experiment, in the order they are run. An entry has five parts: the question, the method, the numbers, the date, the decision. A reading is recorded as measured and also normalised to 60 s (measured x 60,000 / 74,705), because the reference clip is 74.7 s long (D-64). R1 is the only reference machine (D-65); a reading taken on the development machine is labelled `dev` and never stands for E-3 or E-4.

| Experiment | Prompt | State |
|---|---|---|
| TE-7 re-check (ranged requests to the asset host, with the real model files) | 38 | Not run |
| TE-1 (no request outside the closed host list during transcription) | 44 | Not run |
| TE-2 (word probabilities at no more than 10% extra time) | 44 | Not run |
| E-3 (transcription time on R1) | 44, 59 | Not run |
| TE-3 (frame capture method; render-only speed) | 51 | Not run |
| TE-4 (encoder ladder entry; AAC priming) | 51 | Not run |
| E-4 (render and encode time on R1) | 51, 59 | Not run |
| TE-10 (the Windows media job in CI: does it run, and its minutes) | 58 | Not run |
| TE-14 (memory on a 90 s 1080p60 clip, ten runs) | 59 | Not run |
| E-1 (waitlist: visitors and joins) | 60 | Not run |
| M0 gate decision | 60 | Not taken |

---

No experiment has been run yet.
