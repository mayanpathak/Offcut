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

## Entry 3 - 2026-11-06 - P162: the transcribe route, its limits and DC-14, without a provider

**What changed.**

- **Types** (`offcut-api-types`):
  - `transcribe.rs`: `CloudWord`, `TranscribeResponse` and `HealthResponse`. `cloud_transcribe` is `#[serde(default)]`, so an older server's body reads as `false`.
  - `ApiErrorCode::TranscribeUnavailable` is the 14th code.
  - `AsrBackend::Cloud` and the prop enum `CloudOutcome` (7 values).
  - `AnalyticsEvent::AsrCloud {outcome, duration_ms}`, appended **last**, so no existing sample moves; it has its doc line.
  - `web/src/gen/api.ts` is regenerated.
- **Config:**
  - `TRANSCRIBE_ENABLED`, `TRANSCRIBE_API_KEY` (redacted) and `TRANSCRIBE_API_BASE_URL`. The URL is checked by a new `is_base_url`: `is_origin` plus a path of plain segments, no trailing slash.
  - `.env` gains the three names, with `false`, `unused` and Groq's URL; it now has 22.
  - `render.yaml` gains three `sync: false` entries.
- **`server/src/transcribe/`:**
  - `provider.rs`: the trait and `UnconfiguredTranscriber`.
  - `ogg.rs`: the BOS page's `OpusHead` and the EOS page that ends the file exactly. A packet holding "OggS" is not taken for a page.
  - `budget.rs`: per UTC hour and day, refused charges take nothing, compile-time check that it stays under the free tier.
  - `route.rs`: D-352's order, with the body read by `to_bytes(…, 1 MiB)` only after auth, the user's bucket and the switch.
  - Nothing there names the database.
- **Router:**
  - the route, after `/notify-me`, with `body(1 MiB)` and `TranscribeIp`;
  - `/healthz` returns `HealthResponse`;
  - `media_type()` is factored out of `is_media`;
  - `is_transcribe_upload` is DC-14's exception.
- **The rest:**
  - `rate_limit.rs`: `TranscribeUser` (20 per 24 h) and `TranscribeIp` (60 per hour).
  - `error.rs`: 503 `transcribe_unavailable`.
  - `main.rs`: `UnconfiguredTranscriber` and the production budget (P163 adds Groq).
  - `analytics/mod.rs`: `AsrCloud`'s `duration_ms` is capped like the other durations.
- **Web, the two lines the new types force:**
  - `http.ts`'s `API_ERROR_CODES` gains `transcribe_unavailable`;
  - `messages.settings.events` gains `asr_cloud: "Cloud transcription result"`.
  - The rest of D-373 is P165's.
- **TS §5:** rows for `transcribe.rs`, `server/src/transcribe/` (all seven files, two of them P163's), `transcribe_route.rs`, `groq_adapter.rs`, `fixtures/groq/` and `fixtures/ogg/`.

**DECLARED CHANGE DC-14,** `server/tests/no_media_routes.rs`, read line by line:

- **Unchanged:** every existing assertion, `MEDIA_TYPES`, `READ_BODY`, `IGNORE_BODY`, and the route walk read from `router.rs`.
- **Added:**
  - `AUDIO_BODY` (`/transcribe`, `audio/ogg`, 1 MiB) and `audio_route()`;
  - `the_table_is_read` asserts the route is in the table;
  - the media loop skips **only** the pair (`/transcribe`, its own type) and then asserts it gets 401 there, not 415, and 415 on `/events`;
  - the body-limit walk sends `/transcribe` 1 MiB + 1 of `audio/ogg` with a bearer and expects 413, and asserts the limit is at most 1 MiB.
- **A new POST route still fails** until it is named in one of the three lists.
- **The 413 request runs on a client of its own.** The server answers it without reading the whole body and closes that connection, and the next route of the walk (`/account/delete`) was sometimes sent on it. Seen once in three runs; three runs green after.

**REOPENED CONTRACT: `/healthz`'s body.** `notify_me.rs`'s exact-body assertion gains `"cloud_transcribe": true`, the value `TestApp` sets.

**Count assertions that had to rise** (each lists members, so a new one fails until placed; none is weakened):

| Test | Before | After |
|---|---|---|
| `api_error_code_has_*_snake_case_members` | 13 | 14 |
| `typescript_declares_every_type_once` | 39 names, 19 doc lines | 43, 20 |
| `SAMPLES` | 19 | 20 |
| `prop_enums_have_exactly_the_values_of_the_spec` | 16 | 17 |
| `rate_limit.rs`'s group table | 16 | 18 |
| `config.rs`'s required variables | 19 | 22 |
| `<redacted>` fields | 7 | 8 |
| `analytics_allowlist.rs` stored rows | 19 | 20 |
| `analytics_allowlist.rs` free-text bodies | 21 | 22 |

**New tests.**

- `transcribe_route.rs`, 12 cases:
  - success, with the exact bytes received;
  - no bearer, where a 2 MiB body is never read: the 401, or a reset while the client was still sending, and the provider saw nothing;
  - the switch off: 503, and `/healthz` reads `false`;
  - six other types: 415;
  - 1 MiB + 1: 413;
  - five bad files: 400, and exactly 92 s passes;
  - the provider's two failures;
  - a spent budget, where the provider is not called;
  - the 21st clip of a user is 429 while another user passes;
  - 60 per hour per address;
  - the switch is read on every request;
  - P-16: no `sqlx` or `state.db` under `transcribe/`.
- Unit tests:
  - `ogg.rs` (3), including P161's `ffmpeg` tone at 3,000 ms ± 20;
  - `budget.rs` (4);
  - `router.rs` (3, including `audio/ogg` refused on any other path);
  - `config.rs` (1).
- `log_redaction.rs`: the transcribe flow with a marker word in the answer and a marker run of bytes in the audio; neither is in the log, and the `kind=ok` line is.
- `analytics_allowlist.rs`: `asr_backend: "cloud"` and `asr_cloud` accepted; text, an over-cap duration or an extra prop refused.

**Checks.**

- `pnpm check`: fmt, clippy, deny (the duplicate-crate warnings are the existing ones), ESLint, both `tsc`, file tree and copy codes all pass. `check-gen-clean.sh` passes once `web/src/gen/api.ts` is committed with this.
- `cargo test -p offcut-api -p offcut-api-types`: 232 / 0 / 0.
- `cargo test --workspace`: **844 / 0 / 4** (floor 817).
- Vitest 522.

**Found, not fixed.** None.

## Entry 4 - 2026-11-06 - P163: the Groq adapter, punctuation, one real call

**What changed.**

- **`server/Cargo.toml`: `reqwest` gains `multipart`** (D-358). `cargo deny check` stays clean (advisories, bans, licences, sources). It brings `mime_guess` 2.0.5 and `unicase` 2.10.0, both MIT/Apache-2.0. `cargo tree` shows no `native-tls`, `openssl` or `aws-lc`, so the hand-written fallback was not needed. `Cargo.lock` +18 lines.
- **`transcribe/align.rs`** (D-360).
  - Word keys are lower-cased letters and digits.
  - Five rules, in order:
    1. a word with no key is kept as given;
    2. an equal token gives the word its written form;
    3. a token spread over 2-4 words: each word keeps its text and the last gets the token's trailing mark;
    4. the word's own token within the next 3 is taken, and the tokens before it are passed over;
    5. otherwise the word is kept as given.
  - Never adds, drops or reorders a word, and never touches a time. Nine unit tests ("$12,000.", "40%", "don't", "e-mail.", "U.S.", repeated words, missing words and tokens, a number written as words, empty inputs).
  - It is an identity on Groq's real words (asserted on the fixture).
- **`transcribe/groq.rs`** (§7.4, D-359).
  - The request: multipart with `file` (`audio.ogg`, `audio/ogg`, streamed with its length), `model`, `response_format=verbose_json`, `timestamp_granularities[]=word`, `language=en`, `temperature=0`, and a bearer. The timeout is 25 s.
  - Statuses 400, 413, 415 and 422 map to `BadAudio`. Any other non-2xx, a timeout, a network failure or an unreadable body maps to `Unavailable`.
  - **An error body is never read.** The log line names a kind and the status only, so no provider text can reach a log by construction. The plan's extra `log_redaction.rs` case through the mock was replaced by this rule (differs from the plan).
  - `parse` is pure:
    - seconds become rounded milliseconds;
    - a non-finite or negative time drops the word, counted and logged as a number;
    - `duration` falls back to the audio's own length;
    - `model` is `groq:whisper-large-v3-turbo`.
- **`main.rs`:** `GroqTranscriber::new` when `TRANSCRIBE_ENABLED=true`, else `UnconfiguredTranscriber`. A client that cannot be built stops the boot (`ServeError::Transcribe`), as billing does.
- **`groq_adapter.rs`**, 5 cases, with no network:
  - the reference clip's answer: 129 words, the first seven with their punctuation, the first times 380-600 ms, times rising, the last end 60,480 ms (the invented "Thank you.", which the client's guard drops);
  - every recorded answer parses (Opus with segments, FLAC, tone);
  - against a local axum mock: the exact form fields and the file's bytes unchanged; 429, 500, 503 and 401 give `Unavailable`; 400 and 413 give `BadAudio`; a body that is not JSON and a 2 s stall against a 0.5 s timeout give `Unavailable`;
  - a port with nothing listening gives `Unavailable`.

**The one real call through the real code** (F-1; G 3.3):

- **The call:** a temporary `server/tests/zz_groq_live.rs`, run once with `.env` and `.env.deploy` sourced in a subshell and `TRANSCRIBE_ENABLED=true`. It posted `ref32.ogg` (60 s of the reference clip's audio) through `GroqTranscriber::transcribe`.
- **The answer:** 129 words, 60,000 ms, the test finished in 2.6 s.
- **The fixture:** it wrote `web/tests-e2e/fixtures/cloud/speech_60s_portrait.json` (10,672 bytes, a `TranscribeResponse`; Script A's words only).
- **Cleanup:** the file was deleted, and no word was printed.
- **Total real audio so far:** 363 s.

**TS §5:**

- `tests-e2e/cloud-transcribe.spec.ts` and `tests-e2e/fixtures/cloud/`;
- P164's four `workers/asr/` rows: `check-file-tree.mjs` reads untracked files too, and they were already in the tree;
- `features.ts` loses "(V6.1)".

**Checks.**

- `cargo test -p offcut-api --lib --test groq_adapter --test transcribe_route --test log_redaction`: 108 + 5 + 12 + 1, all pass.
- `cargo clippy --workspace --all-targets -- -D warnings` clean.
- `cargo fmt` applied.

**Found, not fixed.** None.

## Entry 5 - 2026-11-06 - P164: features.ts, the Ogg Opus encoder in asr.worker

**What changed.**

- **`web/src/config/features.ts`** (D-365, TS §41.3.1), the file V6.1 was to bring, built now with one flag, `cloud_transcribe`.
  - `vite.config.ts` defines `__OFFCUT_FEATURES__` from `OFFCUT_FEATURES_OFF`, a comma list. Every flag is on by default, and an unknown name stops the build.
  - Where no bundler ran (Vitest, Node), every flag is on, as `render-cache-repo.ts` treats `__RENDERER_VERSION__`.
- **`workers/asr/ogg-opus.ts`:**
  - the Ogg CRC, from a table (polynomial `0x04C11DB7`, initial 0, not reflected);
  - pages with lacing, at most 50 packets or 255 lacing values per page;
  - `OpusHead` (version 1, one channel, the encoder's pre-skip, 16,000) and `OpusTags` (vendor "offcut");
  - the last page is EOS with granule pre-skip + `pcm16.length × 3`, which trims the padded last packet;
  - nothing decodes or changes the audio.
- **`workers/asr/cloud-audio.ts`:**
  - `encodeOpus`:
    - the WebCodecs encoder, handed in as `EncoderDeps` so the test can fake it;
    - TE-C1's config: Opus at 16 kHz, mono, 32 kbps, 20 ms frames;
    - one-second `AudioData`;
    - the pre-skip from the `OpusHead` description, else 312;
    - `{unsupported: true}` when the browser says so.
  - `dropSilentWords`: D-382's guard.
    - Measures each word's RMS against the clip's 95th-percentile sample level.
    - Drops a word more than 30 dB under it, or one starting at or after the end.
    - Measures a word under 40 ms over 40 ms.
    - In digital silence, drops every word.
- **`protocol.ts`** gains two `AsrWorkerApi` members, `encodeForCloud` and `fromCloud`. **`asr.worker.ts`** gains their handlers. `load`, `transcribe` and `unload` are unchanged; `git diff` shows additions only, apart from three import lines.
  - `encodeForCloud`: an encoder that throws is answered `unsupported`, so the device path takes the clip.
  - `fromCloud`: guard, then `postProcess`, then `normalizeTranscript`, then `MIN_WORDS`, as `transcribe` does. Confidence is 1.
  - Neither touches the model session.
- **The result's `Uint8Array` is copied, not transferred:** `rpc-worker.ts`'s `transferablesOf` moves only `Float32Array` buffers, and the file stays frozen. A 90 s file is about 0.32 MB.
- **TS §5:** the four rows came with P163's commit (Entry 4).

**Tests.**

- `ogg-opus.test.ts`, 9 cases:
  - the CRC against a bit-by-bit reference;
  - a reader written in the test reads every page back, with a good CRC, sequence, serial, `OpusHead`, `OpusTags`, and the packets byte for byte;
  - BOS and EOS on the first and last pages only;
  - 50, 50, 20 packets per page, with rising granules and the clip's end on the last;
  - lacing of 600 bytes (255, 255, 90) and of 510 (255, 255, 0);
  - a new page when lacing would pass 255;
  - no packets;
  - a count mismatch throws.
- `cloud-audio.test.ts`, 7 cases:
  - one-second frames, TE-C1's config and the trimmed end;
  - the pre-skip from the description and the default without one;
  - `unsupported`, and an encoder error throws;
  - the guard at the reference clip's levels keeps a −37 dBFS word and drops one over silence;
  - a word starting at or after the end is dropped, and one straddling it is judged on the part inside;
  - the 40 ms window;
  - digital silence and a steady clip.

**The file is real (G 4.3).**

- **The probe:** a throwaway Node script, outside the repository, transpiled the real `cloud-audio.ts` and `ogg-opus.ts` (types stripped) and loaded them in Chrome 156.
- **The input:** the reference clip's 16 kHz audio (`speech_60s_portrait.mp4.pcm48.f32`, each three samples averaged).
- **The encode:** 60 s became 210,960 bytes in 0.63 s.
- **Three readers agree:**
  - `ffprobe`: Opus, mono, 60.002 s (the exact length 59.9998 s);
  - Groq (one real call, 60 s): 200 in 1.65 s, 129 words, the same WER as `ffmpeg`'s 32 kbps file (8 of 128), the first word at the same 0.38 s, so the pre-skip is right;
  - `ogg.rs`: a 3 s tone from the same code, committed as `server/tests/fixtures/ogg/client_tone_3s.ogg`, reads 3,000 ms ± 20 with pre-skip 104 (new unit test).
- **Total real audio so far:** 423 s.

**Counts.**

- Vitest 538 (522 + 16).
- `offcut-api` lib 109.
- ESLint, both `tsc`, and the file tree pass.

**Found, not fixed.** None.

## Entry 6 - 2026-11-06 - P165: cloud mode and fallback in the pipeline

**What changed.**

- **`net/http.ts` (REOPENED CONTRACT, D-373):** `HttpRequest` gains `rawBody` (bytes plus `audio/ogg`, sent as they are) and `attemptTimeoutMs`.
  - **Differs from the plan:** `PROVIDER_FAILURES` is not changed. It maps to an `ErrorCode`, and a new one would touch the frozen `offcut-types`. With `retry: "none"` a 503 is already one answer, read by its `apiError.code`. `API_ERROR_CODES` gained the code in P162.
- **`net/api-client.ts`:**
  - `wake()` becomes `checkHealth()`: the same `GET /healthz` with `retry: "cold-start"`, its `HealthResponse` returned.
  - `transcribeAudio(ogg, signal)`: through `authed()`, the raw Ogg body, one attempt of 30 s (`TRANSCRIBE_TIMEOUT_MS`), the pipeline's signal.
- **State and storage:**
  - `state/cloud-store.ts` holds `availability` (`unknown`, `available`, `unavailable`) and `choice` (`cloud`, `device`). `settledAvailability(ms)` waits for `/healthz` at most `ms`.
  - `persistence/prefs-repo.ts` keeps the choice in the `meta` store under `META_KEYS.transcription`; absent or unreadable means `cloud`.
  - `usecases/set-transcription.ts` writes the choice to both.
  - `start-app.ts` step 6 sets `availability` from `checkHealth()` and loads the choice, awaiting neither.
- **`usecases/transcribe-cloud.ts`** (D-372), the one use-case that sends audio:
  1. waits up to 45 s for a waking API;
  2. has `asr.worker` encode a copy of `pcm16`;
  3. sends it once;
  4. has the worker build the transcript from the words and another copy of `pcm16`, passing through D-382's guard;
  5. reports `asr_cloud`, only once the server has said yes (D-377).
  - Every failure is a fallback reason: `unavailable`, `limited`, `timeout`, `offline`, `unsupported` or `failed`. A cancel is answered without a report.
- **`state/clip-store.ts`:** `asrMode` (D-378), and **also `asrFallback`**, set by `cloudFellBack()`. Use-cases may not import copy, so the interface shows the toast from it (P166). Both are set only while processing.
- **`usecases/run-pipeline.ts`** (§8.5):
  - `chooseMode()` picks cloud only with the flag on, a server that has not said no, the card on Cloud, and a person `signed_in` **now**.
  - **Differs from the plan (D-368): no 3 s wait** for a session being restored. Every existing pipeline test runs with the session unknown, and a session still restoring means a cold API, when availability is unknown anyway.
  - In cloud mode the model is not asked for (D-369).
  - On a fallback, `cloudFellBack()` is called, then the model is asked for (with `ModelDownloadPanel` while it downloads), then today's path runs.
  - `stage_timing` for `asr` says `cloud` on a cloud success.
- **The device path moved, it did not change.** Today's step 6 is now `modelOrEnd()` and steps 7-8 are `hearOnDevice()`. A script compared the old file's blocks with the new file's: steps 3-5, step 6, steps 7-8 (`try`…`catch`) and steps 9-14 are present byte for byte. Step 6's `return;` became `return false;`, the helper's way to say the run ended.
- **`eslint.config.js`:** one edge, `run-pipeline.ts` to `transcribe-cloud.ts`.
- **TS §5:** rows for `cloud-store.ts`, `prefs-repo.ts`, `transcribe-cloud.ts` and its test, and `set-transcription.ts`.

**Tests.**

- **`run-pipeline.test.ts`:**
  - **Its 20 existing cases pass unchanged.** The file's import line gains `afterEach`; no case was edited.
  - 9 new cases in their own block:
    - a cloud success: no `asr.*` call, the rest of the run is today's, `asr_backend: "cloud"`;
    - a fallback: today's calls in today's order after the cloud call, with `asrFallback: "limited"`;
    - no speech in the cloud's words;
    - five conditions that each give today's run exactly: signed out, a session being restored, the server said no, the card on device, the flag off;
    - a cancel while the cloud answers ends `accepted`, with no device call.
- **`transcribe-cloud.test.ts`, 15 cases:**
  - encoding a copy, sending once, building from the words, `ok`;
  - six answers to their outcomes;
  - unsupported;
  - a worker failure;
  - no speech;
  - a server that said no: nothing is sent and no analytics value either;
  - a waking API: going on when it says yes, and giving up at 45 s, both on a fake clock;
  - a cancel;
  - a cancelled worker call.
- **`api-client.test.ts`, 4 cases:**
  - `checkHealth` with and without the field;
  - `transcribeAudio`'s headers, and one refresh on 401;
  - a 503 `transcribe_unavailable` is one answer;
  - a 30 s attempt.
- **`http.test.ts`, 2 cases:** a raw body, and its own attempt timeout.
- **Vitest 568** (538 + 30). ESLint and both `tsc` pass.

**Tier 2** (keyed `pnpm build:e2e`; the protected specs that run the pipeline, unchanged): see below.

**Tier 2 result.** Keyed `pnpm build:e2e`, then `pnpm e2e:media pipeline-preview.spec.ts model-download.spec.ts cancel.spec.ts` (the media project runs one worker): **30 passed in 11.9 min**, no case edited. The fake's `/healthz` has no `cloud_transcribe`, so every run took the device path.

**Push 1** (D-338), CI not waited on.

**Found, not fixed.** None.

## Entry 7 - 2026-11-06 - P166: the card, the copy, the legal text, the landing wording

**What changed.**

- **`ui/components/TranscriptionChoice.tsx`** (§9.1), inside `DropZone`'s two variants, under "Try a sample":
  - **When it shows:** only with the flag on and a server that has not said no; otherwise it renders nothing, and the drop zone is today's.
  - **What it uses:** V4's `SegmentedControl` (a radiogroup with arrow keys), the line under it, and, signed out, the device option shown with the control disabled (its reason is the signed-out line) plus a "Sign in to use cloud transcription" button that opens the existing `SignInDialog`.
  - **Where it sits:** outside the drop target, so a click on the card never opens the file chooser.
  - **The label wrap:** one rule scoped to the card keeps each choice on one line (`white-space: nowrap`); the primitive is unchanged.
- **`copy/transcription.ts`** (D-380), imported by `messages.ts` as `messages.transcription`; `messages.ts` is at 668 of 700 lines.
  - **Differs from §9.2's drafts:**
    - the legend is "How captions are written";
    - the labels are "Cloud · fast" and "On device · private" (the founder's words, 2026-11-06); "On this device · private" wrapped in the control.
  - Every other line is §9.2's.
- **`ProcessingCard`:**
  - in cloud mode it says `processingCloud`, otherwise today's "Everything runs on your computer" line;
  - a hook pushes one toast when `asrFallback` is set: the limit toast for `limited`, the "did not answer" toast otherwise.
- **The settings table, Privacy Policy and Terms:**
  - **Settings table:** five rows; the first is split, and the new row says what is sent, to whom, and that neither keeps it.
  - **Privacy Policy:** "Your video" now names "its sound" among what is kept in the browser; a new section, "Cloud captions"; "Who handles it" names Groq.
  - **Terms, "Your videos":** one clause about cloud captions.
  - **`legal.updated`:** 6 November 2026.
- **Landing** (`copy/landing.ts`, §9.6, F-3 (a)):
  - the privacy heading is "Your video is never uploaded.";
  - the first privacy fact is "It is edited, rendered and exported in your browser, on your computer.";
  - the step is "Transcribed in seconds";
  - the features lead is "Every card below works today.";
  - the captions card names the cloud;
  - the privacy clip's label is "The video stays inside the browser.";
  - "Faster transcription, your choice" leaves "Coming soon" (12 items become 11).
  - No landing string mentions audio or sound beyond today's preview and music lines.
- **TS §5:** rows for `copy/transcription.ts` and `TranscriptionChoice.tsx`.

**DECLARED CHANGE** to a design-release case, not a TS §41.1 proof: `landing.spec.ts`, "the settings link leads to the table of what leaves the device", pins the fixed rows at 4. It now pins 5, with a comment. L-2 reads the "Coming soon" items from the copy, so it follows by itself.

**Captures** for the founder (G 6.4), in `~/cloud-review/`:

1. signed in, Cloud;
2. signed in, On device;
3. signed out;
4. a server that says no: today's drop zone;
5. processing in cloud mode;
6. the landing page's try-it section;
7. the landing page's privacy section with the new clip.

They are taken from the keyed build against an API answered in the capture script (no server, no Groq).

**Checks.** ESLint, both `tsc`, the copy codes and the file tree pass. The landing and settings cases pass (Entry 8).

**Open.**

- **G 6.6:** the founder's approval of the card, the processing line, the toasts, the settings table, the Privacy Policy and Terms sections, and the landing wording (F-4).
- **F-2:** the console check that Zero Data Retention is on; the Privacy Policy's "Neither we nor Groq keep…" depends on it.

## Entry 8 - 2026-11-06 - P167: cloud-transcribe.spec.ts

**What changed.**

- **`tests-e2e/helpers/fake-api.ts`.** Defaults are unchanged, so every existing spec meets the same fake. Three additions:
  - `healthz.cloudTranscribe` adds `"cloud_transcribe":true`;
  - `transcribe: {response, delayMs}` answers `/transcribe`, with 401 when there is no bearer; without the option the route is a 404, as before;
  - `transcribeBodies` records the raw bytes of every upload, whose JSON body is not parsed.
- **`tests-e2e/cloud-transcribe.spec.ts`**, media project, `speech_60s_portrait.mp4`, 8 cases:
  1. **Identity.** One clip processed twice in two fresh contexts: with no `cloud_transcribe` (no card), then with the server's switch on and nobody signed in. The card's signed-out state is shown, axe reports 0, no `/transcribe` request is made and no bytes are sent, and **the stored transcript and prosody are equal as JSON** to the first run's.
  2. **Cloud** (serial with case 3), in a context with **no model**:
     - the card on Cloud, with axe at 0;
     - one request, `content-type: audio/ogg`, under 1 MiB, starting `OggS`;
     - `ffprobe` on the saved body reads Opus, mono, with its duration within 40 ms of the clip's;
     - the processing line is the cloud one;
     - the stored transcript's `model_version` is Groq's, with **129 − 2 words: the silence guard dropped "Thank you." in a real browser**, the last word "month.";
     - no model panel;
     - `asr_cloud` `ok` and `stage_timing` `asr_backend: "cloud"`.
  3. **The export** from that cloud transcript passes `verify_mp4.py`'s 11 checks. It compares 3 flashes in this fixture's exports, the two white frames and one more it reads as a flash, so `FLASHES` is 3.
  4. **The choice is remembered.** "On device" chosen, then a reload: still chosen, axe 0, and while processing only `/events`, never `/transcribe` (P-5).
  5. **A fallback on a 429:** the limit toast, the device transcript, and `asr_cloud` `limited`.
  6. **A fallback on no answer in 30 s:** the "did not answer" toast, the device transcript, and `asr_cloud` `timeout`.
  - **Differs from the plan (§11.4 case 4):** two fallback runs, not three. The 503 mapping is covered by `transcribe-cloud.test.ts`, and each device run costs about a minute on D1.
  - The toasts are looked for right after the drop: a toast lasts 5 s and the clip is ready about a minute later. The first run looked at the end and missed both.
- **`playwright.config.ts`:** the spec is in `MEDIA_SUITES`.

**Runs.**

- First run: 5 passed and 3 failed. The export expected 2 flashes and the verifier compared 3; both toasts were looked for too late.
- After the two fixes: **8 passed in 6.4 min**; again on the final build (with P168's clip), **8 passed in 6.7 min**. Green twice in a row.
- Tier 3's `pnpm e2e` on that build: **57 passed** (1.6 min).
- CI for push 1 (run 42190565424): `ci` and `e2e-media` green.
- `pnpm e2e`: 56 passed and 1 failed before the declared change of Entry 7, then the case alone passed.

## Entry 9 - 2026-11-06 - P168: the privacy clip without the cloud

**What changed.**

- **The composition** `../offcut-remotion/src/clips/Privacy.tsx`, outside this repository; the original is kept in the scratchpad.
  - The cloud outline and the three dotted paths are removed.
  - A violet padlock sits on the file's lower right corner, in a layer above the icon, breathing with the glow.
  - The laptop, the glow and the sentence "Your video stays on your computer." are unchanged. The clip is still 6 s, 180 frames, a seamless loop (both animations are periodic over the loop).
- **The render and re-encode:** `npm run render:all -- privacy`, then the design release's re-encode (designbuildguide Step 6.1) from the master into `~/cloud-upload/`.
  - Read with `ffprobe`: H.264 High, `yuv420p`, limited range, BT.709, 30 fps, 180 frames, 6.000 s, 41,900 bytes (cap 1.5 MB), video only, `moov` first.
  - A middle frame was read by eye.
- **The upload, once** (founder's choice 3a): `sh scripts/upload-assets.sh media` printed `media/offcut-privacy-1280x720.f6af0adb3438b1da.mp4`.
  - The public URL answers 200 with `Cache-Control: public, max-age=31536000, immutable`.
  - The old file is untouched on the host.
- **Wiring:** `LANDING_CLIPS.privacy.path` is the new path; `ui/assets/landing/privacy.webp` is the new poster (8,936 bytes).
- **Capture 7** shows the new clip on the landing page.

## Entry 10 - 2026-11-06 - P169: experiments and the review, by the agent

**The founder's word** (2026-11-06): "I am very busy today, please start the app yourself, review it thoroughly, check the logic part and see if everything works correctly end to end and see if anything breaks… properly check everything like the CSS, how it looks." So G 9.1 and G 9.2 were done by the agent.

**Differs from the guide (G 9.1): no real sign-in.** The founder's mailbox was not available. Instead:

- A temporary Playwright spec (deleted with its config) ran the real keyed build against the real local API on `:8080`.
- The local API was started with `.env` then `.env.deploy` sourced in a subshell and `TRANSCRIBE_ENABLED=true`.
- The session was faked in the page only.
- `/healthz` and `/transcribe` were sent to the real API with an access token signed by the **local** `.env` key (never a production key).
- Everything about transcription was real: the encoder, the upload, the route's checks and budget, Groq, the silence guard, the transcript, the events and the export path.
- The API was stopped when the run ended.

**TE-C3 and TE-C4: passed** (`experiments.md`).

- Over six clips, the cloud finds **every event the device finds**, and the three-item list on both 60 s clips, which the device never found.
- Cloud is **13-14× faster** on a minute of speech (4-5 s against 57-66 s, with the model cached).
- The local API logged six `kind=ok` lines and no warning.
- About 300 s more real audio; about 725 s for the build.

**The review, and what it found and fixed.**

1. **The fallback toast could be told twice.** The hook was in `ProcessingCard`, which the stage replaces with `ModelDownloadPanel` while a fallback downloads the model, then shows again. The hook is now called by `Stage`, which stays mounted, so a fallback is told once per run.
2. **On the landing page the chosen option did not show.** `landing-sections.module.css` styles every `button` in the try-it frame as a pill, which covered the control's checked state. Three rules for the frame's `[role="radio"]` give the choices their own look, with the chosen one in the page's violet tint. The shared primitive is unchanged.
3. **The settings table and the Privacy Policy said "On this device";** the card says "On device". The two now match.

**Checked and found right.**

- The card in all three states.
- The server-says-no drop zone, identical to today's.
- The processing line in cloud mode.
- The limit toast over the download panel during a fallback.
- The settings table with the new row and the `asr_cloud` event.
- The Privacy Policy's "Cloud captions" section, and Groq in "Who handles it".
- The landing page's try-it card at 1280 and at 390 px (phone).
- The stage at 1024 and 768 px.
- The arrow keys moving the choice.
- The landing privacy section with the new clip.

All fourteen captures are in `~/cloud-review/` (1-14).

**Logic read again, with no change needed:**

- the mode chosen once per run, "Process again" choosing again;
- a run cancelled while the cloud answers;
- a server that has not said yes never meets an analytics value it does not know;
- the user's bucket checked before the body is read;
- the budget charged before the provider;
- the cloud words passing the same `postProcess` and `normalizeTranscript` as device words;
- the device path byte for byte (Entry 6).

**Checks:** see below.

- Vitest 568; ESLint and both `tsc` pass.
- On the final keyed build: `cloud-transcribe.spec.ts` with the protected `review-edit.spec.ts`, **24 passed** (13.7 min), the editor unchanged; `pnpm e2e` **57 passed**.

**Open for the founder.**

- **G 6.6:** the wording approval (the captures and the copy files).
- **F-2:** Zero Data Retention seen on in the console.

**Found, not fixed.** None.

## Entry 11 - 2026-11-06 - P170 (agent's part): copy-back and the one full media run

**Copy-back** (plan §18):

- **TS:**
  - §5 rows (done with each prompt);
  - §7: the lint edge;
  - §16: the model paragraph and "Cloud transcription is built";
  - §22.1: `/healthz`'s field, the `POST /transcribe` row, and DC-14's exception under the table;
  - §22.8: the three variables;
  - §24.1: the provider's host from the server;
  - §24.5: the two buckets and the budget;
  - §25.1: P-1 and P-5 revised, P-16 new;
  - §25.3: the inventory row and "Built";
  - INV-3 and INV-4;
  - the two rejected-alternative lines;
  - §41.2 DC-14, §41.3's flag list, and §41.5's row;
  - §37: TE-C1 to TE-C5.
- **PS:**
  - the positioning line;
  - the four P1 tables' rows;
  - J4: the device path only;
  - §12.6, "as built" above the superseded text;
  - §12.7: the sound row;
  - §17: the legal gate.
- **BP §0:** the release's paragraph.

**Tier 4** (keyed build of P169's last commit, D1, Chrome 156):

| Suite | Result |
|---|---|
| `pnpm e2e:media`, the full run, once (D-338) | **112 passed, 1 skipped** (the existing skip), 0 failed, in 40.5 min |
| `cargo test --workspace` | **863 / 0 / 4** (floor 817) |
| Vitest | 568 (floor 522) |
| `pnpm e2e` | 57 (floor 57) |

**Push 3**, CI waited on: see below.

- CI for push 3 (run 42223029927): `ci` and `e2e-media` green, no re-run needed.

**Open for the founder** (G 10.2 onwards): the wording approval (G 6.6), Zero Data Retention seen on (F-2), the Render variables with the switch off, the merge, the switch on, the live journey, the tag.
