# cloudchangelog.md - record of every change made while building the cloud transcription release

Entries are oldest first. Each states what changed, what differs from the plan, how it was checked, and what is open. The plan is `cloudimplementation.md`; Setup's prompt opens the build. Entry 0 below came before it.

## Open items

| # | Item | Owner | Where |
|---|---|---|---|
| 1 | ~~F-1~~: **answered yes** 2026-11-06 (Entry 1) | Founder | plan §15 |
| 2 | F-2: answered yes 2026-11-06. **To confirm at G 10.3** that Zero Data Retention shows as on in Groq's console, before the merge; the Privacy Policy's sentence depends on it | Founder | plan §15 |
| 3 | ~~F-3~~: **answered (a)**, "Your video is never uploaded." | Founder | plan §15 |
| 4 | F-4: answered yes. The approval itself is G 6.6 (P166); a lawyer's review stays on the launch checklist | Founder | plan §15 |
| 5 | ~~F-5~~: **answered yes** (switch off at the merge) | Founder | plan §12.2 |

## Known issues for later prompts

| # | Issue | Prompt |
|---|---|---|
| 1 | Carried from V6 (its known issue 3): `openEditor`'s wait for the drop zone flakes on the hosted runner, most often right after `cancel.spec.ts`. One re-run of the failed job is allowed | P170 |
| 2 | Carried from the design release: the lavapipe SIGSEGV in CI's "Golden frames" after every golden has passed. One re-run is allowed | P170 |
| 3 | Carried from the design release: D1's `pnpm e2e` is load-sensitive; run with `--workers=4` and rerun failures alone | Every prompt |

## Entry 0 - 2026-11-06 - The plan (outside a prompt)

**Why this release.** The founder asked for cloud transcription with Groq's Whisper, built and shipped as a version of its own, after the design release and before V6.1, as BP §0 already announced. Dodo Payments approved live payments the same day; live mode stays in the founding release after V6.6 (V6 D-285, D-303).

**The founder's decisions** (2026-11-06, in the conversation that produced this plan; recommendations offered first, answers below).

| Question | Answer |
|---|---|
| Model and tier | `whisper-large-v3-turbo`, Groq's free tier first, Dev tier when the cap bites (recommendation accepted) |
| Groq plan, limits, fallback, server retention, scope, legal wording, landing "Coming soon" card | Recommendations 2-4 and 6-11 accepted: 20 cloud clips per user a day and a global stop under Groq's cap; fall back to the device on any failure; audio in memory only; clips up to 90 s; the agent writes the legal wording and the founder approves it; the "Coming soon" card becomes live |
| Consent (recommendation 5) | **Changed by the founder:** no per-clip prompt. Cloud is the default, a toggle in the drop zone before upload, with a card: cloud is fast, on-device is slower but private. Then, asked to choose between the default and "we recommend on-device under 90 s" (every clip today is under 90 s): **(a) cloud on for every clip; the card explains the trade-off** |
| Who may use it | **(b) signed-in only;** signed-out people get the device path and the toggle says "Sign in for cloud" |
| The landing page's privacy claim | "Make the privacy claim that the user's video never gets uploaded and the video never leaves the device; just don't say anything about the audio on the landing page." Rule P-12 bans "never leaves your device" before a lawyer's review: F-3 offers the wording |
| The privacy clip (it shows an empty cloud) | **(a) re-render it without the cloud and upload it** |
| Compression | The founder asked for the audio to be compressed and asked whether Render can handle audio files. Read: Render's free service has 512 MB and about 0.1 CPU, sleeps after 15 minutes, and its included bandwidth was reported cut to 5 GB a month in 2026; uncompressed audio at Groq's daily cap would be about 28 GB a month. Hence Opus at about 32 kbps (8× smaller), the 1 MiB route limit, the wake at page load and the 45 s wait (D-370, D-374, §14) |
| Rules and tests | "You are allowed to bend the rules and tests as far as nothing breaks"; ask before every step that needs the founder, and whenever unsure; do it safely. Hence one declared change only (DC-14), the availability gate that keeps every protected suite on the device, and two kill switches |

**What was read to write the plan** (the code at `68d2ae2`):

- `run-pipeline.ts`: the model and the device recognizer, steps 2-8.
- `asr.worker.ts`, `whisper-runtime.ts`, `word-timestamps.ts`: confidence is always 1 on the device, so Groq's lack of one changes nothing.
- `offcut-text`'s `normalize.rs` and `sentences.rs`: sentences end at `.?!`, so Groq's words need their punctuation (D-360).
- Server:
  - `router.rs`: `media_guard` refuses `audio/*` everywhere (INV-4), and the default body limit is 16 kB;
  - `rate_limit.rs`, `config.rs`, `error.rs`, `billing/adapter.rs` (the client pattern);
  - `no_media_routes.rs`, and `notify_me.rs`'s exact `/healthz` body.
- Client:
  - `http.ts`: JSON bodies only, 10 s per attempt, 502-504 read as waking;
  - `api-client.ts`, `start-app.ts` (it already wakes the API);
  - `fake-api.ts` (its `/healthz` has no extra field);
  - `DropZone.tsx`, `ProcessingCard.tsx`, `WhatLeavesTable.tsx`;
  - `copy/messages.ts` (652 lines) and `copy/landing.ts`.
- Configuration and tooling:
  - `vercel.json`: `/api/v1` is rewritten to Render, so no new browser host is needed;
  - `render.yaml`, `vite.config.ts`, `env.ts`, `eslint.config.js`, `check-file-tree.mjs`.
- Specs: TS §5, §16, §22, §24, §25, §41; PS §12.6, §12.7, §17.
- Fixtures and assets:
  - `fixtures/speech/README.md`: the device transcript of the reference clip, by sentence;
  - the Remotion `Privacy.tsx` and the upload script.

**Facts of third parties, from their documents or summaries, to be read again in P161:**

- Groq:
  - `whisper-large-v3-turbo` costs 0.04 USD per hour of audio;
  - the free tier allows about 7,200 audio seconds an hour and 28,800 a day;
  - uploads are capped at 25 MB on the free tier;
  - `verbose_json` with `timestamp_granularities[]=word` returns per-word `start` and `end`;
  - audio endpoints keep data up to 30 days unless Zero Data Retention is on.
- Render free: about 1 minute to wake after 15 minutes idle.

**Documents written:** `cloudimplementation.md` (D-336 to D-381, DC-14, F-1 to F-5), `cloudbuildguide.md`, `coding-promptscloud.md` (Setup, P161-P170), this changelog, `experiments.md`. Uncommitted: Setup commits them on `cloud-build`.

**Open.** Items 1-5 above.

## Entry 1 - 2026-11-06 - Setup: the branch, the documents, the floor

**The founder's answers** (2026-11-06, to F-1 to F-5 as listed in the plan's §15): "yes to all". Read as:

- F-1 yes;
- F-2 yes, to be seen on in the console at G 10.3;
- F-3 (a), the recommended wording;
- F-4 yes;
- F-5 yes.

The founder also asked to "start working on the feature like we regularly do, open a draft PR and then build".

**Branch.** `cloud-build` from `origin/main` at `68d2ae2`.

**Differs from the plan (D-340).** `8d94a06` (V6's closing entry) was not picked. `git cherry-pick` came out empty and `git merge-base --is-ancestor 8d94a06 origin/main` holds: it reached `main` with an earlier merge. The memory note that said otherwise was stale.

**Documents.**

- The five files of `docs/cloud_transcription/`.
- TS §5 gains their five rows, after `v6/experiments.md`. `check-file-tree.mjs` does not check `docs/`; the rows are for the reader, as V6's are.

**The floor** (D-339), read on D1:

| Suite | Count | Floor |
|---|---|---|
| Rust (`cargo test --workspace`) | **817 / 0 / 4** | 817 / 0 / 4 |
| Vitest | **522** in 21 files | 522 |
| `pnpm e2e` (keyed `pnpm build:e2e`, `--workers=4`) | **57 passed** in 1.6 min | 57 |

**One commit for Setup and P161.** Both changed only these documents, and the probes ran while the floor was being read, so they share the commit `CT setup and P161`.

**Open.** F-2's console check (open item 2); F-4's approval at G 6.6.

## Entry 2 - 2026-11-06 - P161: the probes

No product code. Everything is recorded in `experiments.md`, TE-C1 and TE-C2.

**Real calls to Groq** (F-1; the reference clip's audio only, as `speech_60s_portrait.mp4`'s track; never a private recording):

| # | File | Audio | Status | Time |
|---|---|---|---|---|
| 1 | FLAC, 16 kHz mono | 60 s | 200 | 2.16 s |
| 2 | Opus 16 kbps | 60 s | 200 | 0.80 s |
| 3 | Opus 24 kbps | 60 s | 200 | 0.93 s |
| 4 | Opus 32 kbps | 60 s | 200 | 0.98 s |
| 5 | Opus 32 kbps, word and segment granularity | 60 s | 200 | 1.10 s |
| 6 | A 440 Hz tone, Opus 32 kbps | 3 s | 200 | 0.40 s |

Total: 303 s of audio.

**What was read.**

- **Groq's words carry their punctuation;** numbers come as digits. `align` (D-360) is an identity on them, kept as a guard ("as read" note in the plan).
- **WER:** 32 kbps 6.2%, FLAC 5.5%. **32 kbps chosen** (D-370 unchanged). A 90 s clip is about 0.32 MB.
- **Timing.** Groq's sentence starts are a median 33 ms from the speech onsets measured in the audio; the device recognizer's are 264 ms late. TE-C2's rule now reads against the measured onsets, and cloud captions will sit about 0.3 s earlier, closer to the voice.
- **New decision D-382: a silence guard.** Whisper invents a closing "Thank you." / "Yes." in the silence at the end. `fromCloud` drops a word that starts after the audio ends, or whose RMS is more than 30 dB below the clip's 95th-percentile level. On this clip the invented word sat at −64.8 dBFS against −37.2 for the quietest real word. `fromCloud` takes a `pcm16` copy. Plan §2.4, §8.3 and §11.2 updated.
- **Groq's limits,** from its response headers: 7,200 audio-s per rolling hour, 2,000 requests per day (D-346 holds).
- **Browser encoder (TE-C1):** Chrome 156 and Edge 156 both support Opus at 16 kHz mono at 24, 32 and 48 kbps. 60 s encodes in 0.25-0.34 s. The description is a 19-byte `OpusHead` with pre-skip 104. The 48 kHz path is not built.

**Fixtures** (committed with their TS §5 rows in P162):

- `server/tests/fixtures/groq/`: four responses, with `x_groq.id` replaced by `req_fixture`. Their words are Script A's, already public in `fixtures/speech/README.md`.
- `server/tests/fixtures/ogg/ffmpeg_tone_3s.ogg`.

**Stop-and-ask triggers:** none met. No `words`, no times, an onset offset over 60 ms, unsupported Opus and limits below the budget were all checked, and none happened.

**Found, not fixed.** Groq heard "wiki" for "week". It is one word, and a person corrects it in review as with any caption.
