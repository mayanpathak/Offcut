# cloudimplementation.md - Offcut "Cloud transcription" release: implementation plan

**Implements:** the founder's cloud-transcription release of 2026-11-06. It sits between the design release (tag `design`, `main` at `68d2ae2`) and V6.1, as `buildplan.md` (BP) §0 already announces ("the cloud-transcription release that comes before V6.1"). It is built and shipped exactly like a version: one branch, one plan, one guide, one prompts file, one changelog, one merge, one tag. References: `technicalspec.md` (TS), `product.md` (PS), `buildplan.md` (BP), all v1.3. "V6 §n" is a section of `docs/v6/v6implementation.md`; "DCL" is `docs/design/designchangelog.md`; "CT CL" is `docs/cloud_transcription/cloudchangelog.md`; "G n.m" is a step of `docs/cloud_transcription/cloudbuildguide.md`. Decisions D-1 to D-335 belong to V1 to the design release; this document continues at **D-336**. The next declared change of TS §41.2 is **DC-14**. Prompts continue at **P161**.

**Reader.** Whoever writes the code. The guide gives the order of work and the commands; `coding-promptscloud.md` gives the prompts an agent runs. This file gives the contracts and the decisions, and wins where the three disagree. It was written after reading the code at `68d2ae2` on 2026-11-06: every "today" below is a fact of that commit. Each decision the specs leave open is marked **(CT decision)**. A change to something an earlier version froze is marked **REOPENED CONTRACT**. A number nobody has measured is marked **(assumption)**. A fact of a third party that its documents do not settle is marked **(read in P161)**.

**The one-sentence goal.** At the end of this release a signed-in person drops a clip and gets its captions in seconds, written by Groq's Whisper from the clip's compressed sound, with a toggle in the drop zone that explains the trade-off and switches to on-device transcription; anyone signed out, or anyone who switches it off, gets exactly today's product; any failure of the cloud path quietly finishes the clip on the device; and the video itself never leaves the computer.

**The second goal: nothing that works breaks (TS §41).** A clip transcribed on the device behaves exactly as at `68d2ae2`: the same transcript, events, frames, samples and exports. Every protected proof of TS §41.1 runs unchanged. The only protected file that changes is `server/tests/no_media_routes.rs`, through the new row **DC-14**, because one route must now take audio.

**What you can demo.** Sign in, drop the sample clip: no model download, "Writing captions" finishes in a few seconds, the captions are Whisper large-v3-turbo's. Flip the toggle to "On this device", drop it again: today's path, model download and all. Sign out: the toggle says "Sign in to use cloud transcription", and processing is today's.

---

## 0. Contents

1. Scope. 1A. Preconditions. 2. Decisions (D-336 to D-382). 3. Dependencies and environment. 4. File tree. 5. Build order. 6. The wire. 7. The server. 8. The client. 9. Interface and copy. 10. Privacy and the spec revisions. 11. Tests. 12. CI, deploy and rollout. 13. Experiments. 14. Risks. 15. Founder decisions still open. 16. Hand-over to V6.1. 17. Exit checklist. 18. Copy-back to the specs.

---

## 1. Scope

### 1.1 In this release

| Area | What works at the end |
|---|---|
| Server route | `POST /api/v1/transcribe`: bearer only, one compressed audio file (`audio/ogg`, Opus, mono), at most 1 MiB and 92 s; forwarded to Groq; the words come back. Nothing is stored or logged |
| Server provider | `Transcriber` trait; `GroqTranscriber` (`whisper-large-v3-turbo`, English, word timestamps); `UnconfiguredTranscriber`; a fake in tests. Punctuation is restored to Groq's word list in the adapter |
| Server limits | 20 cloud clips per user per day; 60 requests per IP per hour; a global audio budget under Groq's free tier (7,000 s an hour, 27,000 s a day) |
| Kill switches | `TRANSCRIBE_ENABLED` on Render (a restart, no rebuild); the build flag `cloud_transcribe` in `config/features.ts` (a redeploy) |
| Availability | `GET /healthz` says `cloud_transcribe: true/false`. The client offers cloud only after a server that says `true` |
| Client audio | The 16 kHz mono PCM the device recognizer already hears, encoded with WebCodecs Opus at 32 kbps (assumption, TE-C2) into an Ogg file written by our own muxer: about 0.37 MB for 90 s |
| Client pipeline | Cloud mode at run start for a signed-in person whose choice is "Cloud"; no model download; the same `postProcess` and `normalizeTranscript` as the device path; on any failure the device path, once, with a toast |
| Interface | A `TranscriptionChoice` card inside the drop zone (landing and stage), with the trade-off explained; "Sign in to use cloud transcription" when signed out; the processing card's line by mode; a fallback toast |
| Copy | New `copy/transcription.ts`; the settings table, the Privacy Policy and Terms updated; the landing page's privacy wording made true for the new default, saying nothing about audio (founder) |
| Landing clip | The privacy clip re-rendered without the cloud, uploaded once to a new path |
| Analytics | `asr_backend: "cloud"`; one new event `asr_cloud {outcome, duration_ms}` |
| Experiments | TE-C1 to TE-C5 (§13) |

### 1.2 Not in this release

| Left out | Why, or where it goes |
|---|---|
| Sources longer than 90 s, chunked upload | V6.1. The route, the trait and the encoder are written so that V6.1 calls them once per window (§16) |
| Any video, frame or image upload | Never (P-1 as revised keeps it) |
| Cloud for signed-out people | Founder's choice (b), 2026-11-06 (D-343) |
| A second provider, provider choice in settings | Not needed; the trait allows it later |
| Server-side storage of anything from this route | Never (D-347) |
| Per-export "mode used" display on the settings page (PS §12.6) | Replaced by the card shown before every drop, the processing card's line and the settings table (D-344, D-378) |
| Live payments, F-3's in-app offers | Unchanged; the founding release and DCL open item 1 |
| `privacy-network.spec.ts` | V8, as planned. This release adds network assertions of its own to `cloud-transcribe.spec.ts` (§11.4) |

### 1.3 What a person sees

| Who | At the drop zone | During processing | Result |
|---|---|---|---|
| Signed out | The card shows "On this device" selected, Cloud unavailable, and a "Sign in to use cloud transcription" button | Today's card, "Everything runs on your computer" | Today's product, byte for byte |
| Signed in, first time | "Cloud" selected, the trade-off line | "Writing captions" with "Your captions are being written in the cloud. Your video stays on your computer." | Captions in seconds; no model download |
| Signed in, switched to device | "On this device" selected (remembered on this device) | Today's card | Today's product |
| Signed in, cloud fails | As above | The card goes on; one toast: "Cloud transcription did not answer, so this clip is being transcribed on your computer." The model downloads if it must | Today's product for that clip |
| Any, server says unavailable | No card | Today's card | Today's product |

---

## 1A. Preconditions

- [x] 1. **The design release is closed.** `design` is tagged; `origin/main` is `68d2ae2` (Part B merged), deployed.
- [x] 2. **The Groq key exists.** `TRANSCRIBE_API_KEY` is a line of `.env.deploy` (checked 2026-11-06 by line count only; the value was never read).
- [x] 3. **The founder's product decisions are given** (2026-11-06): §2.2.
- [ ] 4. **F-1: real Groq calls during the build** with the reference clip's audio (the founder's own voice, already public as the sample clip), never `testclips/v3testclip.mp4` or a corpus clip. Needed by P161 (§15).
- [ ] 5. **F-2: Zero Data Retention on** in Groq's console (Data Controls). Needed before P166 writes the Privacy Policy, and before the merge (§15).
- [ ] 6. **F-3: the landing page's privacy wording** (§15). Needed by P166.
- [ ] 7. **F-4: legal review.** The founder approves the Privacy Policy and consent wording at G 6.6; a lawyer's review stays on the launch checklist (§15).
- [ ] 8. **Draft pull request open,** `cloud-build` → `main` (Setup).

---

## 2. Decisions this document makes

All are **(CT decision)** unless they say "founder".

### 2.1 Process

| # | Decision | Why |
|---|---|---|
| D-336 | **All of the release is on `cloud-build`,** made from `main` at `68d2ae2`. Commits are titled `CT P<nn>: <title>`. The agent commits and pushes there and opens the draft pull request. Marking it ready, merging, Render's and Groq's dashboards, production reads and the tag are the founder's unless the founder hands one over in writing | The standing rule of every version (memory: version-branch rule) |
| D-337 | **The documents are in `docs/cloud_transcription/`** (the founder's folder): this plan, `cloudbuildguide.md`, `coding-promptscloud.md`, `cloudchangelog.md` (CT CL), `experiments.md`. TS §5 gets their five rows in Setup | One folder per release; `check-file-tree.mjs` reads TS §5 |
| D-338 | **Speed rules hold** (V6 D-242, memory "one CI run per batch", "test-time budget"): tiered gates, targeted runs first, **one** full media run, in P170. Pushes: Setup; P165 (not waited on); P168 (not waited on); P170 (waited on) | The founder finds e2e and CI time excessive |
| D-339 | **The floor is `main` at `68d2ae2`.** DCL Entry 16 recorded Rust 817 / 0 / 4, Vitest 522, `pnpm e2e` 57; V6's last full media run 106 passed, 1 skipped; verifier self-test 16. Setup reads the quick counts again (Rust, Vitest, `pnpm e2e`) and records them; no full media run in Setup | Counts only rise (TS §41.1) |
| D-340 | **V6's closing entry reaches `main` with this merge.** Setup cherry-picks `8d94a06` (docs only, `docs/v6/v6changelog.md`) onto `cloud-build` | It has waited on `v6-build` since 2026-11-04 (memory) |
| D-341 | **Protected proofs and frozen paths** (G 0.4): `crates/offcut-{types,plan,scene,render,text,detect,dsp,mp4,wasm-core,wasm-render}/`, `fixtures/{golden,labeled,speech}`, `verify/`, `web/src/models/`, `server/migrations/`, every `*.snap` and golden PNG: an empty diff. In `web/src/workers/`, only `protocol.ts` (additive), `asr.worker.ts` (two handlers added, the three existing untouched) and two new files under `workers/asr/` change. `whisper-runtime.ts` and `word-timestamps.ts` do not change | TS §41.3.4: old callers keep their function; new paths call new functions |

### 2.2 The founder's product decisions (2026-11-06)

| # | Decision | Why |
|---|---|---|
| D-342 | **Founder: Groq, `whisper-large-v3-turbo`, on Groq's free tier first;** the Dev tier (card on file) when the daily cap starts to bite. Request: `language=en`, `temperature=0`, `response_format=verbose_json`, `timestamp_granularities[]=word` | 0.04 USD per hour of audio on paid tiers; about 320 clips of 90 s a day on the free tier (7,200 audio-s/hour, 28,800/day, third-party summaries, **read in P161** from the console's limits page) |
| D-343 | **Founder: signed-in people only** (choice b). Signed out: on-device, and the card says "Sign in to use cloud transcription" with a button that opens `SignInDialog` | Our key must not be open to anyone; the account is the rate key |
| D-344 | **Founder: cloud is the default** for a signed-in person, as a toggle in the drop zone before upload, with a card explaining the trade-off (choice a): "Cloud: fast" and "On this device: private, a little slower". Not a per-clip prompt. **REOPENED PS §12.6** | The founder's call; PS §12.6's per-clip prompt is replaced by the card, which is visible before every drop |
| D-345 | **Founder: one clip of at most 90 s** (every clip today). Long sources come with V6.1 | Scope |
| D-346 | **Founder: limits.** 20 cloud clips per user per day (token bucket `per(20, 24 h)`), and 60 requests per IP per hour. A global audio budget of **7,000 s an hour and 27,000 s a day** (`transcribe/budget.rs`, `[ONLY]`), just under Groq's free tier. Over the budget, the route answers 503 `transcribe_unavailable` | Abuse cannot cost money on the free tier; at worst the day's quota runs out and everyone falls back to the device |
| D-347 | **Founder: the server holds the audio in memory for the one request and forwards it only to Groq. Neither the audio nor the words returned are stored or logged.** A log line names a `kind`, a duration bucket and a status only | P-1 as revised (§10) |
| D-348 | **Founder: any failure falls back to the device once,** with one toast, never an error card | A clip never dead-ends on the cloud |
| D-349 | **Founder: the landing page claims only that the video is not uploaded, and says nothing about audio.** The wording is F-3 (§15). Every landing sentence that is false under the new default changes (§9.6) | "Still a very good privacy story" (founder). Audio is disclosed on the card, in the settings table and in the Privacy Policy, which the landing page links to |
| D-350 | **Founder: the privacy clip is re-rendered without the cloud** (choice a), with the same sentence, and uploaded once to a new path | The clip draws dotted lines failing to reach an empty cloud, which is no longer the whole truth |
| D-351 | **Founder: the agent writes the Privacy Policy, Terms and card wording; the founder approves them before the merge** (G 6.6). A lawyer's review is a launch-gate item (F-4) | PS §17's legal-review gate, applied at launch |

### 2.3 The server

| # | Decision | Why |
|---|---|---|
| D-352 | **`POST /api/v1/transcribe`, bearer only.** Request body: the raw bytes of one Ogg Opus file, `Content-Type: audio/ogg` (a `codecs=opus` parameter allowed). Limits are checked **before the body is read**: auth, then the per-user and per-IP buckets, then the content type, then the body with a 1 MiB cap, then the Ogg check (mono `OpusHead`, duration 1 ms to `MAX_CLIP_DURATION` + 2 s), then the budget, then the provider. Success: 200 `TranscribeResponse` (§6.2) | A rejected request must cost neither bandwidth nor budget. The handler takes `Request` and reads the body itself (`axum::body::to_bytes(body, 1 MiB)`), so the checks come first |
| D-353 | **DECLARED CHANGE DC-14 (§10.3): the one exception to INV-4.** `router.rs`'s `media_guard` lets through exactly `POST /api/v1/transcribe` with `audio/ogg`. Every other route, and every other media type on this route, still gets 415. `no_media_routes.rs` names the route as the one exception and asserts its limits | INV-4's purpose (no media reaches the server) is narrowed to "no video, no image, no audio but the one compressed file of the one route" |
| D-354 | **New `ApiErrorCode::TranscribeUnavailable` (`transcribe_unavailable`, 503)** for the kill switch, the budget and any provider failure. 429 `rate_limited` is the user's or the IP's bucket; 400 `bad_request` a file that is not a mono Ogg Opus file of an allowed length; 413 `payload_too_large` over 1 MiB; 415 any other type. `errors.rs`'s count test goes from 13 to 14, with the new member's place | `http.ts` must read 503 `transcribe_unavailable` as an answer, not a waking API (as V6 D-287) |
| D-355 | **`GET /healthz` says whether cloud transcription is on:** `{"ok": true, "version": "...", "cloud_transcribe": <TRANSCRIBE_ENABLED and a configured provider>}`. Still no database. **REOPENED: the body of `/healthz`.** `notify_me.rs`'s exact-body assertion gains the field (`false` in `TestApp`) | One request the app already makes at start; it is the availability gate of D-366 |
| D-356 | **Three variables:** `TRANSCRIBE_ENABLED` (`true`/`false`, required), `TRANSCRIBE_API_KEY` (required, redacted in `Config`'s `Debug`), `TRANSCRIBE_API_BASE_URL` (required, `https://`, `https://api.groq.com/openai/v1` in production; a local mock in tests). `.env` gains three names; locally `TRANSCRIBE_ENABLED=false` | `config.rs` aborts on a missing variable, so Render holds them **before** the deploy that reads them (G 10.2). A base URL makes the adapter testable against a local server, as V6 did with `BILLING_API_BASE_URL` |
| D-357 | **`transcribe/provider.rs`: the `Transcriber` trait** (`async fn transcribe(&self, audio: Bytes) -> Result<CloudTranscript, TranscribeError>`; `TranscribeError::{Unavailable, BadAudio}`), `UnconfiguredTranscriber` (always `Unavailable`; used when `TRANSCRIBE_ENABLED=false`), and in tests `FakeTranscriber`. **`transcribe/groq.rs` is the only file that names the provider** | As `BillingProvider` (V6 D-298): the route and its tests never meet Groq |
| D-358 | **`reqwest` gains its `multipart` feature** (one existing dependency, one feature). `cargo deny` must stay clean; if the feature brings a crate `deny.toml` refuses, the body is written by hand in `groq.rs` (one boundary, two parts) and the feature is not added | Groq's endpoint takes `multipart/form-data` |
| D-359 | **Server → Groq: one attempt, 25 s timeout.** 429, 5xx, a timeout, a network failure or an unreadable body → `Unavailable` (503 to the client). 400 or 413 from Groq → `BadAudio` (400). Neither Groq's text nor its error body reaches a log | The client gives the request 30 s (D-374) |
| D-360 | **Punctuation is restored in the adapter** (`transcribe/align.rs`, §7.5). **As read in P161: Groq's words already carry it ("year,", "videos."), so `align` is an identity on them and stays as a guard against a change of provider or model.** Groq's `words` are matched in order to the whitespace tokens of its `text`, and a word whose letters and digits equal a token's takes the token's written form ("$12,000.", "you?", "40%"). The rule never adds, drops or reorders a word and never changes a time. **What Groq's words hold is read in P161 first;** if they carry punctuation already, `align` is the identity on them and stays as a guard | `offcut-text` ends a sentence after `.`, `?` or `!` and parses "$12,000." as one quantity (`sentences.rs`, `normalize.rs`): unpunctuated words would make one 90 s sentence split only at pauses, and lose `number_reveal`s. The on-device recognizer's words carry punctuation |
| D-361 | **`transcribe/ogg.rs` reads what the route needs, and nothing else:** the first page's capture pattern, BOS flag and `OpusHead` (version 1, 1 channel, pre-skip); the last page's granule position (a backwards search for `OggS` in the last 64 KiB). Duration = (granule − pre-skip) / 48 samples per ms. No CRC check, no decoding | The duration is what the budget and the 92 s rule need. The server stays off media crates (TS §7, `cargo deny`) |
| D-362 | **`transcribe/budget.rs` (`[ONLY]`):** two in-memory counters of accepted audio seconds, one per UTC hour and one per UTC day, reset when the hour or day changes, checked and charged before the provider call. Lost at a restart, which is accepted: Groq's own 429 is the last line | As the rate limiter (TS §24.5) |
| D-363 | **The route group table grows by two:** `TranscribeUser` `per(20, 24 h)`, checked in the handler with `RateKey::User` (V6 D-263); `TranscribeIp` `per(60, 1 h)`, as a layer. TS §22.1 gets the row | The per-user key is known only after `AuthUser` |
| D-364 | **Logging:** one `info` line per request with `kind` (`ok`, `unavailable`, `bad_audio`, `budget`, `disabled`), `duration_s_bucket` (`lt30`, `lt60`, `lte92`) and the status. `log_redaction.rs` gains a case: a marker word returned by `FakeTranscriber` and a marker byte string in the audio never appear in captured logs | P-14's rule applied to this route |

### 2.4 The client

| # | Decision | Why |
|---|---|---|
| D-365 | **`web/src/config/features.ts` is built now, with one flag, `cloud_transcribe`** (TS §41.3.1; V6.1 adds its own flags to the same file). The value comes from a Vite `define`, `__OFFCUT_FEATURES__`, which `vite.config.ts` builds from `OFFCUT_FEATURES_OFF` (a comma list, empty by default): every build has the flag on unless that variable names it | A redeploy with `OFFCUT_FEATURES_OFF=cloud_transcribe` removes the whole feature: the second kill switch. Never read from the network (TS §41.3.1) |
| D-366 | **The availability gate.** `state/cloud-store.ts` holds `availability: "unknown" | "available" | "unavailable"`, set once per page load from the `/healthz` answer that `start-app.ts` already requests (`wake()` becomes `checkHealth()`, which returns the parsed `HealthResponse`). A body without `cloud_transcribe: true`, or no answer within the cold-start budget, is `"unavailable"`. **The card is shown, and cloud attempted, only when the flag is on and availability is not `"unavailable"`** | An old server (no field), the kill switch and every existing e2e test (the fake's `/healthz` answers `{"ok":true,"version":"fake"}`) all read as unavailable: the protected suites run on the device with no edit |
| D-367 | **The choice is stored on this device:** IndexedDB `meta` store, key `transcription`, `Versioned<"cloud" | "device">`. Absent means `"cloud"`. No migration: the store exists since version 1. `persistence/prefs-repo.ts` is its only reader and writer | TS §23.1's database; nothing leaves the device |
| D-368 | **Cloud mode is decided once, at run start** (`runPipeline`, step 1): flag on, availability not `"unavailable"`, choice `"cloud"`, and the person signed in (`auth-store` `signed_in` with a token; `unknown` waits up to 3 s for `restoreSession`). "Process again" decides again | One run, one mode; a toggle flipped during a run changes the next one |
| D-369 | **In cloud mode the model is not downloaded or loaded.** `modelReady()` is not called at step 2. On fallback it is called then, and the clip shows `ModelDownloadPanel` while it downloads, as today | The download is the slowest part of a first run |
| D-370 | **The audio sent is `pcm16`,** the 16 kHz mono PCM the device recognizer hears, copied (`pcm16.slice()`) and transferred to `asr.worker`'s new `encodeForCloud`, which encodes it with WebCodecs `AudioEncoder` (`codec: "opus"`, 1 channel, 32 kbps (assumption, TE-C2), 20 ms frames) and writes it into Ogg with `workers/asr/ogg-opus.ts`. The original `pcm16` stays for a fallback. An encoder that is not supported answers `{unsupported: true}`, which falls back | Same input as the device path, so the two transcripts differ only by the recognizer. About 0.37 MB for 90 s, against 2.9 MB of 16-bit WAV: under Render's and Vercel's bandwidth (§14) |
| D-371 | **Cloud words become a `Transcript` exactly as device words do:** `asr.worker`'s new `fromCloud({words, durationMs, modelVersion})` runs `postProcess` (unchanged, imported from `word-timestamps.ts`), then `normalizeTranscript`, then the `MIN_WORDS` rule. Confidence is 1, as the device path's is today. `model_version` is `"groq:whisper-large-v3-turbo"` (from the response's `model`) | One builder of transcripts (TS §10.8); the device recognizer gives no confidence either (`word-timestamps.ts`) |
| D-382 | **As read in P161 (TE-C2): a silence guard on cloud words.** Whisper invents a closing "Thank you." or "Yes." in silence at the end of a clip. `fromCloud` drops a cloud word that starts at or after the end of the audio, or whose RMS level over its interval (clamped to the audio) is more than 30 dB below the clip's 95th-percentile sample level. On the reference clip this floor is 12.6 dB below the quietest real word and 15 dB above the invented one. `fromCloud` therefore receives a copy of `pcm16`. The device path is unchanged | A caption nobody said is worse than a missing one; the client holds the audio, the server does not decode it |
| D-372 | **`usecases/transcribe-cloud.ts` is the only use-case that sends audio.** It waits for availability (at most 45 s while `"unknown"`), encodes, sends through `api-client.ts`'s `transcribe()` (`authed()`, so one refresh and one repeat on 401), and turns the answer into a transcript or a fallback reason. One lint edge: `run-pipeline.ts` → `transcribe-cloud.ts` | TS §7's use-case rule; P-5 as revised names this file |
| D-373 | **REOPENED CONTRACT `http.ts`: `HttpRequest` gains `rawBody?: { bytes: Uint8Array; contentType: "audio/ogg" }` and `attemptTimeoutMs?: number`;** `PROVIDER_FAILURES` gains `transcribe_unavailable`, and `API_ERROR_CODES` the new code. Only `transcribe()` uses the two options | Today every body is JSON and every attempt has 10 s |
| D-374 | **Timeouts:** the request has one attempt of 30 s and no retry (`retry: "none"`); the wait for a waking API is at most 45 s; the whole cloud attempt is bounded by the pipeline's cancel signal. Any of them ends in fallback | Waiting 45 s for a cold API is still faster than a first-time model download of about 250 MB (PS J4) plus device transcription |
| D-375 | **REOPENED INV-3:** import, preview and review never wait on the API; **processing waits on it only in cloud mode,** at most D-374's bounds, and always completes on the device when it cannot | The invariant's purpose (the product works with the API down) holds |
| D-376 | **Cancel aborts the request** (the pipeline's signal is passed as `HttpRequest.signal`). The run ends `accepted`, as today (V4 D-152). A Groq call already started on the server finishes and its words are dropped | |
| D-377 | **Fallback reasons and analytics.** `asr_cloud {outcome, duration_ms}`, `outcome` one of `ok`, `unavailable` (503, or availability became `"unavailable"`), `limited` (429), `timeout`, `offline`, `unsupported` (no Opus encoder), `failed` (anything else, 400 or 413 included). `stage_timing` for `asr` carries `asr_backend: "cloud"` on success. **Both are sent only when availability is `"available"`** | An old server never meets a value its allowlist refuses (`deny_unknown_fields` would reject the whole batch) |
| D-378 | **`clip-store` gains `asrMode: "cloud" | "device" | undefined`,** set at run start and on fallback, read by the processing card. Additive | The card's line must be true |
| D-379 | **Real Groq calls during the build use only the reference clip's audio** (Script A, the founder's public sample clip) and its derived fixtures, and synthetic tones. **Never `testclips/v3testclip.mp4` (location tag, P-13) or a corpus clip (V3 D-112).** Each real call needs F-1 | The founder's rule on their recordings |

### 2.5 Interface, rollout

| # | Decision | Why |
|---|---|---|
| D-380 | **Copy goes in a new `copy/transcription.ts`,** imported into `messages.ts` as `landing.ts` is (designimplementation D-321). `messages.ts` is 652 of 700 lines today | The file limit |
| D-381 | **Rollout in two steps with the server switch** (§12.2): Render gets the three variables with `TRANSCRIBE_ENABLED=false` before the merge; after the merge and the checks (the app is unchanged: `/healthz` says `false`, no card), the founder sets `true`; the founder's live journey and the agent's checks follow; then the tag. Turning it off is the same switch, about a minute, no code | Production changes for real users only after it has been seen working there, and can go back without a deploy |

---

## 3. Dependencies and environment

### 3.1 Rust

| Crate | Change | For |
|---|---|---|
| `reqwest` (`server/Cargo.toml`) | + `multipart` feature (D-358) | Groq's form upload |
| `bytes` | Direct use if `axum`'s re-export does not suffice; already in `Cargo.lock` | The audio buffer |
| Nothing else | | |

`cargo deny check` after the change, in P163. No new npm dependency: WebCodecs and IndexedDB are the browser's.

### 3.2 Environment

| Variable | Change | Local value (`.env`) | Production (Render) |
|---|---|---|---|
| `TRANSCRIBE_ENABLED` | New, required, `true`/`false` | `false` | `false` at the merge, `true` at G 10.6 (D-381) |
| `TRANSCRIBE_API_KEY` | New, required, redacted | `unused` (a placeholder; local real calls read `.env.deploy` in the agent's shell only) | The Groq key, from `.env.deploy` |
| `TRANSCRIBE_API_BASE_URL` | New, required, `https://` (or `http://localhost:<port>` in tests) | `https://api.groq.com/openai/v1` | The same |
| `OFFCUT_FEATURES_OFF` | New, build-time, optional | unset | unset (set only to kill the feature) |

`render.yaml` gains the three `envVars` with `sync: false`. `.env` goes from 19 to 22 names. `TestApp` gets `FakeTranscriber` and `TRANSCRIBE_ENABLED=true` unless a case says otherwise; `GET /healthz` in `TestApp` therefore says `true`, and `notify_me.rs`'s assertion is written for the value `TestApp` sets (§11.1).

---

## 4. File tree

**Created, server.** `transcribe/{mod,provider,groq,align,ogg,budget,route}.rs`; tests `transcribe_route.rs`, `groq_adapter.rs`, `fixtures/groq/` (real responses, scrubbed of Groq's request id), `fixtures/ogg/` (two small files: a synthetic tone written by the client's encoder in P164, and one by `ffmpeg`).

**Changed, server.** `config.rs`, `state.rs`, `router.rs` (the route, `media_guard`'s exception, `/healthz`'s field), `rate_limit.rs` (two groups), `error.rs`, `lib.rs`, `main.rs`, `Cargo.toml`, `render.yaml`, `deny.toml` only if D-358 needs it; tests `common/mod.rs`, `no_media_routes.rs` (DC-14), `notify_me.rs` (one assertion), `log_redaction.rs` (one case), `rate_limit.rs` (two groups), `analytics_allowlist.rs` (new member and event).

**Created, crates.** `offcut-api-types/src/transcribe.rs` (`TranscribeResponse`, `CloudWord`, `HealthResponse`).

**Changed, crates.** `offcut-api-types/src/{errors,analytics,lib}.rs`. Then `pnpm gen:types` rewrites `web/src/gen/api.ts`.

**Created, web (9 source files, 3 test files).** `config/features.ts`; `copy/transcription.ts`; `state/cloud-store.ts`; `persistence/prefs-repo.ts`; `usecases/transcribe-cloud.ts`, `usecases/transcribe-cloud.test.ts`; `usecases/set-transcription.ts`; `workers/asr/ogg-opus.ts`, `workers/asr/ogg-opus.test.ts`; `workers/asr/cloud-audio.ts`, `workers/asr/cloud-audio.test.ts`; `ui/components/TranscriptionChoice.tsx`. Tests: `tests-e2e/cloud-transcribe.spec.ts`, `tests-e2e/fixtures/cloud/speech_60s_portrait.json`.

**Changed, web.** `vite.config.ts` (the define), `net/http.ts`, `net/api-client.ts`, `workers/protocol.ts`, `workers/asr.worker.ts`, `usecases/run-pipeline.ts`, `usecases/start-app.ts`, `state/clip-store.ts`, `persistence/schema.ts` (`META_KEYS.transcription`), `ui/components/{DropZone,ProcessingCard}.tsx`, `ui/styles/components.module.css` (or a new `transcription.module.css` if the former would pass 700), `copy/{messages,landing}.ts`, `net/asset-fetch.ts` (the privacy clip's path), `ui/assets/landing/privacy.webp`, `eslint.config.js` (two edges), `playwright.config.ts` (the new spec in `MEDIA_SUITES`), `tests-e2e/helpers/fake-api.ts` (options), `net/http.test.ts`, `net/api-client.test.ts`, `usecases/run-pipeline.test.ts`.

**Changed outside the repository.** `../offcut-remotion/src/clips/Privacy.tsx` (P168), rendered and uploaded once.

**TS §5 rows,** in the commit that creates each file: the server files and tests above; `features.ts` loses its "(V6.1)" mark and gains "(cloud transcription, one flag)"; the new web files; `docs/cloud_transcription/`'s five files (Setup).

**Frozen in this release** (G 0.4): D-341's list. No server migration: nothing is stored.

---

## 5. Build order

| Prompt | Deliverable | Gate tier | Push |
|---|---|---|---|
| Setup | `cloud-build`, documents, `8d94a06` picked, floor read, draft PR | Counts | Yes |
| P161 | Probes, no product code: Groq's real responses (TE-C2), the browser's Opus encoder (TE-C1), the bitrate; fixtures recorded | 0 | No |
| P162 | API types, config, `transcribe/` without Groq (trait, unconfigured, Ogg, budget, route), DC-14, `/healthz`, limits, errors, analytics members; server tests | 1 | No |
| P163 | `groq.rs`, `align.rs`, `reqwest` multipart, `main.rs` wiring; `groq_adapter.rs`; one real call through the local API | 1 | No |
| P164 | `features.ts`, `ogg-opus.ts`, `cloud-audio.ts`, the two worker handlers, `protocol.ts`; unit tests; the encoder's file read by `ffprobe`, Groq and `ogg.rs` | 1 | No |
| P165 | `http.ts`, `api-client.ts`, `cloud-store`, `prefs-repo`, `transcribe-cloud.ts`, `run-pipeline.ts`, `start-app.ts`, `clip-store`; unit tests | 1 + 2 | **First, not waited on** |
| P166 | `TranscriptionChoice`, copy, processing line, toast, settings, legal, landing wording; captures; founder's look | 2 | No |
| P167 | `fake-api.ts` options, `cloud-transcribe.spec.ts`, the identity case | 2 + 3 | No |
| P168 | The privacy clip: render, check, upload, path, poster | 2 | **Second, not waited on** |
| P169 | TE-C3, TE-C4 on D1 with real Groq; the founder's local walk | 1 | No |
| P170 | Copy-back, one full media run, merge, rollout (D-381), live checks, TE-C5, tag `cloud` | 4 | **Third, waited on** |

Ten prompts and Setup. Estimate: about 24 hours of agent time over 3-4 working days, plus the founder's steps (assumption). TS §41.4's "twice the estimate" stop is at 20 prompts.

---

## 6. The wire

### 6.1 The route (a new row of TS §22.1)

| Route | Auth | Purpose | Body limit | Rate limit | Idempotency |
|---|---|---|---|---|---|
| `POST /transcribe` (cloud transcription) | bearer | One clip's compressed sound (`audio/ogg`, Opus, mono, ≤ 92 s) → its words with times, from the transcription provider; the body is never stored or logged | **1 MiB** (DC-14) | 20 per 24 h per user; 60/h per IP; global audio budget (D-346) | None; each success counts against the budget |

### 6.2 Types (`offcut-api-types/src/transcribe.rs`)

```rust
/// One word as the provider heard it, with its written form restored (D-360).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct CloudWord {
    pub text: String,
    pub start_ms: u32,
    pub end_ms: u32,
}

/// The answer of `POST /transcribe`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct TranscribeResponse {
    pub words: Vec<CloudWord>,
    /// The duration of the audio the provider heard, in ms.
    pub duration_ms: u32,
    /// "groq:whisper-large-v3-turbo": becomes the transcript's `model_version`.
    pub model: String,
}

/// The body of `GET /healthz`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
pub struct HealthResponse {
    pub ok: bool,
    pub version: String,
    /// The route will try (TRANSCRIBE_ENABLED and a provider). Absent from an older server.
    #[serde(default)]
    pub cloud_transcribe: bool,
}
```

`CloudWord.text` and the response travel **from** the server; TS §25.1 P-1's "no free-text field in a DTO" is revised for this one response (§10.1).

### 6.3 Answers

| Case | Status | Body | Client reads it as |
|---|---|---|---|
| Success | 200 | `TranscribeResponse` | the words |
| No or bad bearer | 401 | `unauthorized` | `authed()` refreshes once, repeats once; then `failed` |
| Session revoked | 401 | `session_expired` | as today; `failed` |
| Per-user or per-IP bucket empty | 429 | `rate_limited` with `retry_after_secs` | `limited` |
| Not `audio/ogg` | 415 | `unsupported_media_type` | `failed` (a bug) |
| Over 1 MiB | 413 | `payload_too_large` | `failed` |
| Not a mono Ogg Opus file, or 0 ms, or over 92 s | 400 | `bad_request` | `failed` |
| `TRANSCRIBE_ENABLED=false`, budget spent, provider failed or timed out | 503 | `transcribe_unavailable` | `unavailable` (an answer, not a wake: D-373) |
| No answer in 30 s | | | `timeout` |
| Offline | | | `offline` |

---

## 7. The server

### 7.1 Module layout

```
server/src/transcribe/
├── mod.rs        re-exports; `pub mod route;`
├── provider.rs   Transcriber trait, CloudTranscript, TranscribeError, UnconfiguredTranscriber
├── groq.rs       the only file that names Groq: request, response, error mapping (D-357, D-359)
├── align.rs      written forms onto words (D-360)
├── ogg.rs        OpusHead and the last granule (D-361)
├── budget.rs     [ONLY] the audio budget (D-362)
└── route.rs      POST /transcribe (D-352)
```

`AppState` gains `transcriber: Arc<dyn Transcriber>` and `transcribe_budget: Arc<Budget>`. `main.rs` builds `GroqTranscriber::new(config)` when `TRANSCRIBE_ENABLED=true` (a client that cannot be built stops the boot, as `DodoBilling::new` does), otherwise `UnconfiguredTranscriber`.

```rust
// provider.rs
pub struct CloudTranscript { pub words: Vec<CloudWord>, pub duration_ms: u32, pub model: String }

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum TranscribeError {
    #[error("the transcription provider cannot be used")]
    Unavailable,
    #[error("the provider refused the audio")]
    BadAudio,
}

#[async_trait]
pub trait Transcriber: Send + Sync {
    /// Whether the route should try. `false` for `UnconfiguredTranscriber`.
    fn enabled(&self) -> bool;
    async fn transcribe(&self, audio: Bytes, duration_ms: u32) -> Result<CloudTranscript, TranscribeError>;
}
```

### 7.2 `media_guard` (DC-14)

Today (`router.rs`): any `multipart/*`, `video/*`, `audio/*` or `application/octet-stream` request gets 415 before routing reaches a handler. After:

```rust
async fn media_guard(req: Request, next: Next) -> Result<Response, AppError> {
    let content_type = req.headers().get(CONTENT_TYPE).and_then(|v| v.to_str().ok());
    if content_type.is_some_and(is_media) && !is_transcribe_upload(&req, content_type) {
        return Err(AppError::UnsupportedMediaType);
    }
    Ok(next.run(req).await)
}

/// The one exception (DC-14): exactly POST /api/v1/transcribe with audio/ogg.
fn is_transcribe_upload(req: &Request, content_type: Option<&str>) -> bool {
    req.method() == Method::POST
        && req.uri().path() == "/api/v1/transcribe"
        && content_type.is_some_and(|t| media_type(t) == "audio/ogg")
}
```

`media_type` is today's lower-cased, parameter-stripped reading, factored out of `is_media` and unit-tested: `audio/ogg`, `AUDIO/OGG; codecs=opus` pass on that path; `audio/wav`, `audio/webm`, `video/mp4`, `multipart/form-data`, `application/octet-stream` do not; `audio/ogg` on any other path does not. The layers keep their order. The route sets `DefaultBodyLimit::max(MiB)` on itself (layer 6) and reads the body with `to_bytes(body, MiB)`.

### 7.3 The handler, in order

```
1. AuthUser (bearer) ............................... 401
2. limit(TranscribeUser, User(id)) ................. 429   (TranscribeIp is the route's layer, before 1)
3. transcriber.enabled() ............................ 503 transcribe_unavailable   kind=disabled
4. content type is audio/ogg ........................ 415   (only reachable with audio/ogg; checked again)
5. to_bytes(body, 1 MiB) ............................ 413
6. ogg::read(&bytes) → duration_ms in 1..=92_000 .... 400   kind=bad_audio
7. budget.charge(duration_ms, now) .................. 503   kind=budget
8. transcriber.transcribe(bytes, duration_ms)
     Ok(t)            → 200 TranscribeResponse          kind=ok
     Err(Unavailable) → 503 transcribe_unavailable      kind=unavailable
     Err(BadAudio)    → 400 bad_request                 kind=bad_audio
```

The budget is charged at 7 and **not refunded** on a provider failure: Groq's own counter has usually moved too. The audio buffer is dropped when the handler returns.

### 7.4 Groq (`groq.rs`; shapes **read in P161** before this is written)

- `POST {TRANSCRIBE_API_BASE_URL}/audio/transcriptions`, `Authorization: Bearer {TRANSCRIBE_API_KEY}`, multipart: `file` (`audio.ogg`, `audio/ogg`), `model=whisper-large-v3-turbo`, `response_format=verbose_json`, `timestamp_granularities[]=word`, `language=en`, `temperature=0`.
- Expected response (to be confirmed in P161): `{ "text": "...", "words": [{"word": "...", "start": 0.0, "end": 0.32}, ...], "duration": 59.98, "x_groq": {"id": "..."} , ... }`. The adapter reads `text`, `words[].{word,start,end}` and `duration`, ignores the rest, and never logs any of it.
- Seconds become whole milliseconds (`round`). A word with a non-finite or negative time is dropped (logged as a count). `start > end` is kept; `postProcess` on the client orders it as it does device words.
- `align(text, words)` (§7.5), then `CloudTranscript { words, duration_ms, model: "groq:whisper-large-v3-turbo" }`.
- Error mapping per D-359. The 25 s timeout is the client's builder timeout.

### 7.5 `align.rs`

```
key(s)  = the lower-cased letters and digits of s, in order
tokens  = text.split_whitespace(), each with its key; tokens whose key is empty are skipped
for each word w (in order):
    if key(w) is empty                          → w.text trimmed, unchanged
    else if the next token t has key(t)==key(w) → t's text; advance both
    else if the next 2-4 words' keys joined equal key(t)
                                                → each of those words keeps its own text,
                                                  the last one gets t's trailing punctuation; advance
    else if some token among the next 3 has key == key(w)
                                                → skip to it, take its text
    else                                        → w.text trimmed, unchanged (token pointer stays)
```

Properties, each a unit test: the output has exactly as many words as the input, in the same order, with the same times; with `text` equal to the words joined by spaces it is the identity; a word whose key equals its token's takes the token's text whatever the case; dropping, adding or misspelling one token in `text` changes at most the words next to it. Cases from P161's fixtures, plus: "$12,000.", "40%", "don't", "U.S.", "e-mail", a repeated word ("the the"), a number Groq writes in its words as digits and in `text` as words (kept as the word, unchanged).

### 7.6 `ogg.rs`

```
read(bytes) -> Result<OggInfo { pre_skip: u16, duration_ms: u32 }, OggError>
  first 27 bytes: "OggS", version 0, header_type has BOS (0x02)
  first packet (from the lacing table) is 19 bytes: "OpusHead", version 1, channels == 1
  last page: rfind "OggS" in the last 64 KiB, header_type has EOS (0x04), granule_position i64 LE
  duration_ms = (granule - pre_skip) / 48, must be >= 1
```

Unit tests on hand-built pages, on `fixtures/ogg/ffmpeg_tone_3s.ogg` (made with `ffmpeg -f lavfi -i sine=frequency=440:duration=3 -ac 1 -ar 16000 -c:a libopus -b:a 32k`) and on `fixtures/ogg/client_tone_3s.ogg` (made by the browser encoder in P164): both 3,000 ms within 20 ms. Stereo, no BOS, truncated, empty, a PNG: each its error.

---

## 8. The client

### 8.1 `features.ts`

```ts
// The release's feature flags (TS §41.3.1), from a build-time define, never from the network.
declare const __OFFCUT_FEATURES__: { cloud_transcribe: boolean };
export type FeatureFlags = { readonly cloud_transcribe: boolean };
export const FEATURES: FeatureFlags = Object.freeze({ ...__OFFCUT_FEATURES__ });
```

`vite.config.ts`: `__OFFCUT_FEATURES__: JSON.stringify({ cloud_transcribe: !off.includes("cloud_transcribe") })`, where `off` is `process.env.OFFCUT_FEATURES_OFF` split on commas. `features.ts` is `[ONLY]`. Vitest gets the same define in `vitest.config.ts` if its config does not inherit Vite's (read in P164).

### 8.2 Availability and choice

```ts
// state/cloud-store.ts
export type CloudAvailability = "unknown" | "available" | "unavailable";
export type TranscriptionChoice = "cloud" | "device";
type CloudState = { availability: CloudAvailability; choice: TranscriptionChoice; choiceLoaded: boolean };
```

- `start-app.ts` step 6: `void checkHealth().then((h) => setAvailability(h.ok && h.data.cloud_transcribe === true ? "available" : "unavailable"))`. `checkHealth()` is `api-client.ts`'s `wake()` with the parsed body returned (same request: `GET /healthz`, `retry: "cold-start"`). A failure after the cold-start budget is `"unavailable"`.
- `start-app.ts` also loads the choice from `prefs-repo.ts` (`"cloud"` when absent). Nothing waits for either (INV-3).
- `usecases/set-transcription.ts` writes the choice to the store and the repository; `TranscriptionChoice.tsx` calls it.

### 8.3 Encoding (`asr.worker`)

```ts
// protocol.ts, AsrWorkerApi, two additions (the three existing members are unchanged)
encodeForCloud(p: { pcm16: Float32Array }): Promise<{ ogg: Uint8Array; durationMs: number } | { unsupported: true }>;
fromCloud(p: { words: CloudWord[]; pcm16: Float32Array; modelVersion: string }): Promise<{ ok: Transcript } | { rejected: "NoSpeech" }>;
```

(`durationMs` is `round(pcm16.length / 16)`; the `pcm16` is the copy kept for a fallback, transferred once the words have come back, for D-382's guard.)

`cloud-audio.ts`, `encodeOpus(pcm16, deps)`:

1. `AudioEncoder.isConfigSupported({ codec: "opus", sampleRate: 16000, numberOfChannels: 1, bitrate: 32000, opus: { frameDuration: 20000 } })`. If TE-C1 shows 16 kHz supported in Chrome and Edge, that is the only path, and unsupported means `{unsupported: true}`. If TE-C1 shows only 48 kHz supported, `cloud-audio.ts` upsamples `pcm16` by 3 with linear interpolation (no `OfflineAudioContext` exists in a worker) and encodes at 48 kHz; Groq resamples to 16 kHz anyway.
2. Feed `AudioData` of 1 s (16,000 frames, `f32`, mono, timestamps in µs) and `flush()`.
3. Collect `EncodedAudioChunk`s (bytes and `duration`), and `metadata.decoderConfig.description` from the first output. If that description is an `OpusHead`, its pre-skip is used; otherwise 312 (assumption, **read in P161/TE-C1**).
4. `ogg-opus.ts`, `writeOggOpus({ packets, preSkip, inputSampleRate: 16000, totalSamples48k })`:
   - page 0: `OpusHead` (19 bytes: magic, version 1, 1 channel, pre-skip, 16000, gain 0, mapping 0), granule 0, BOS;
   - page 1: `OpusTags` (vendor "offcut", no comments), granule 0;
   - audio pages: packets in order, at most 50 packets (about 1 s) or 255 lacing values per page; each page's granule is pre-skip plus the 48 kHz samples of the packets it completes; the last page is EOS with granule pre-skip + `pcm16.length × 3` (so the end trims exactly);
   - CRC-32 per page: polynomial `0x04C11DB7`, initial 0, no reflection, computed with the CRC field zeroed; one serial number per file (random); page sequence from 0.
5. `durationMs = round(pcm16.length / 16)`.

`fromCloud` drops the words D-382's guard refuses (`cloud-audio.ts`'s pure `dropSilentWords(words, pcm16)`), maps each remaining `CloudWord` to a `RawWord { text, startMs, endMs, confidence: 1 }`, runs `postProcess(raw, durationMs)` and `normalizeTranscript(raw, modelVersion)`, and refuses with `NoSpeech` below `LIMITS.MIN_WORDS`, exactly as `transcribe` does. Neither handler touches the model session (INV-12 holds trivially).

### 8.4 `transcribe-cloud.ts`

```ts
export type CloudFallback = "unavailable" | "limited" | "timeout" | "offline" | "unsupported" | "failed";
export async function transcribeInCloud(
  pcm16: Float32Array<ArrayBuffer>, signal: AbortSignal,
): Promise<{ ok: Transcript } | { rejected: "NoSpeech" } | { fallback: CloudFallback }>;
```

1. If availability is `"unknown"`, wait for it to settle, at most 45 s (D-374). `"unavailable"` → `fallback: "unavailable"`.
2. `pool.asr.encodeForCloud({ pcm16: pcm16.slice() }, { transfer })`. `unsupported` → `fallback: "unsupported"`.
3. `apiClient.transcribe(ogg, signal)`: `authed()`, `rawBody`, `attemptTimeoutMs: 30_000`, `retry: "none"`.
4. Map §6.3's answers to a fallback, or call `pool.asr.fromCloud({ words, durationMs, modelVersion: model })`.
5. Track `asr_cloud` with the outcome and the elapsed time, only when availability is `"available"` (D-377).

### 8.5 `run-pipeline.ts`

Steps 1-6 and 9-14 are unchanged, except as marked. Today's step 7-8 block moves, without a change of a line, into `hearOnDevice(run, signal, pcm16, chain)`.

```
1.  (unchanged) ... then: mode = await chooseMode()       // D-368
                   clipStore.setAsrMode(mode)
2.  model = mode === "device" ? modelReady() : undefined  // D-369
    if mode === "device" && model not ready: processing("probe_audio", true)   (as today)
3-5 (unchanged)
6.  if mode === "device": today's step 6 on `model`
7-8 if mode === "cloud":
        run.stage = "asr"; processing("asr", false); pushFeed({ kind: "transcribing" })
        r = await transcribeInCloud(pcm16, signal)
        if "fallback" in r:
            clipStore.setAsrMode("device"); toast once (D-348)
            model = modelReady(); processing("asr", <model not ready>)
            today's step 6 on `model`, then heard = await hearOnDevice(...)
        else heard = r
    else heard = await hearOnDevice(...)                  // today's code
9-14 (unchanged)
```

`stage_timing` `asr` is tracked once, with `asr_backend: "cloud"` on a cloud success and the device backend otherwise. A cancel during any wait ends the run as today. **The on-device branch executes exactly today's calls in today's order;** `run-pipeline.test.ts`'s existing cases pass unchanged, which is the unit-level proof.

---

## 9. Interface and copy

### 9.1 `TranscriptionChoice` (inside `DropZone`, both variants)

Rendered when `FEATURES.cloud_transcribe` and availability is not `"unavailable"`; otherwise `null`, and the drop zone's DOM is today's. In the stage variant it sits under the drop target, inside the frame ("in that rectangle", founder); in the landing variant, under the "Try a sample" button. A `role="radiogroup"` with two radios styled as a segmented switch (V4 primitives and tokens), and one line under it:

| State | Selected | Cloud radio | Line under it |
|---|---|---|---|
| Signed in, choice cloud | Cloud | enabled | `copy.cloudLine` |
| Signed in, choice device | On this device | enabled | `copy.deviceLine` |
| Signed out | On this device (shown; the stored choice is untouched) | disabled | `copy.signedOutLine` and a secondary button `copy.signIn` that opens `SignInDialog` |

The card is visible before any drop, so the disclosure always precedes an upload (D-344). Keyboard: arrow keys move between radios; axe at 0 violations.

### 9.2 Drafts of every string (`copy/transcription.ts`; the founder approves at G 6.6)

```ts
export const transcriptionCopy = {
  legend: "Captions",
  cloud: "Cloud",
  cloudHint: "Fast",
  device: "On this device",
  deviceHint: "Private, a little slower",
  cloudLine: "Captions in seconds. Only your clip's sound is sent to write them, and it is not kept. Your video stays on your computer.",
  deviceLine: "Nothing of your clip leaves this computer. It takes a little longer, and short clips are quick either way.",
  signedOutLine: "Cloud captions need a free account. Until then, everything runs on this computer.",
  signIn: "Sign in to use cloud transcription",
  processingCloud: "Your captions are being written in the cloud. Your video stays on your computer.",
  fallbackToast: "Cloud transcription did not answer, so this clip is being transcribed on your computer.",
  limitToast: "You have used today's cloud transcriptions, so this clip is being transcribed on your computer.",
};
```

`ProcessingCard` shows `processing.reassure` in device mode (today's line) and `processingCloud` in cloud mode. The two toasts go through `ToastHost` (V4 P117), once per run.

### 9.3 The settings table (`messages.settings.rows`)

| Data | Leaves your device? |
|---|---|
| Video, video frames, captions as edited, project edits | No |
| The sound of a clip, compressed, when "Cloud" captions are on (signed in only) | Yes: to our server, which sends it to Groq to write the captions and returns the words. Neither keeps the sound or the words after the request. Choose "On this device" and nothing of the clip leaves |
| Your email and plan status | Yes, to our servers |
| Count of exports (no content) | Yes |
| Anonymous usage events from a fixed list (for example stage timings and error codes) | Yes, and the whole list is shown below |

The event list gains `asr_cloud` automatically (`ANALYTICS_EVENT_DOCS`); `messages.settings.events` gains `asr_cloud: "Cloud transcription result"`.

### 9.4 Privacy Policy (`messages.legal.docs.privacy`), the changed sections

- **"Your video":** "Your video is not uploaded to our servers. Offcut processes it in your browser, on your computer, and keeps the video, its sound, the transcript, captions and edits in that browser's storage. You can remove them by clearing this site's data in your browser."
- **New, "Cloud captions":** "When you are signed in and "Cloud" captions are on, which is the default for an account, the sound of each clip you drop is compressed in your browser and sent through our server to Groq, which writes the words and their timings and sends them back. Neither we nor Groq keep the sound or the words after the request. Your video and its frames are never sent. Choose "On this device" before you drop a clip, and nothing of that clip leaves your computer."
- **"Who handles it":** adds "Groq writes the captions of clips sent for cloud transcription."
- `updated`: the merge's date.

(The sentence "Neither we nor Groq keep..." is true only with F-2's Zero Data Retention on; P166 does not write it before the founder confirms F-2.)

### 9.5 Terms (`messages.legal.docs.terms`, "Your videos")

"Your videos and the shorts you make stay yours. Offcut processes them on your computer; when cloud captions are on, the sound of a clip is transcribed by our provider. You are responsible for having the right to use what you put in and what you publish."

### 9.6 The landing page (`copy/landing.ts`), with F-3's recommended wording

| Key | Today | After |
|---|---|---|
| `privacy.heading` | "Not uploaded. Made in your browser." | "Your video is never uploaded." |
| `privacy.facts[0]` | "The video, its sound and its words stay in this browser." | "It is edited, rendered and exported in your browser, on your computer." |
| `beforeAfter.steps[0]` | "Transcribed on your computer" | "Transcribed in seconds" |
| `features.lead` | "Every card below works today, in your browser." | "Every card below works today." |
| `features.cards.captions.body` | "Word by word, timed to your speech. Three styles: Clean, Bold and Mono. Numbers can take the accent colour." | "Word by word, timed to your speech, written in seconds in the cloud or on your computer if you prefer. Three styles: Clean, Bold and Mono. Numbers can take the accent colour." |
| `clipLabels.privacy` | "The video stays inside the browser; nothing is uploaded." | "The video stays inside the browser." |
| `privacyLine` | "Your video stays on your computer." | unchanged |
| `marquee` "Runs in your browser" | | unchanged (rendering and export do) |
| `soon.next.items[0]` "Faster transcription, your choice" | Coming soon | removed: it is built. "Next" keeps "Long videos" |

No landing string mentions audio or sound (D-349), and no feature card is added, so the page's layout does not change. `landing.spec.ts` reads its strings from the copy, so most of its cases follow by themselves; any case that counts the "Coming soon" items (12 today, 11 after) is updated by name in P166 and recorded as a **DECLARED CHANGE** to a design-release case (not a TS §41.1 proof). Before editing, P166 lists every landing case that reads one of the keys above.

### 9.7 The privacy clip

`Privacy.tsx` in `offcut-remotion`: the cloud outline and the dotted paths are removed; the video file icon stays inside the laptop's screen with its glow and a small padlock; the text "Your video stays on your computer." is unchanged. Rendered with `npm run render:all -- privacy`, checked with designimplementation §9.2's list, uploaded once with `scripts/upload-assets.sh media`, its new path in `LANDING_CLIPS.privacy`, its poster replacing `ui/assets/landing/privacy.webp`. The old file stays on the asset host (nothing is deleted there).

---

## 10. Privacy and the spec revisions

### 10.1 TS §25.1, revised rows

| # | Promise, as revised | Structural | Automatic |
|---|---|---|---|
| P-1 | Video, video frames, music, brand assets, captions as edited and project edits are not uploaded. **A clip's sound leaves only as one compressed Opus file in the body of `POST /transcribe`, only for a signed-in person with "Cloud" chosen, and is forwarded only to the transcription provider; the words come back in its response.** Transcript text, word timings and prosody scores leave only in `POST /plan` (V6.2) | `media_guard` admits only that route and type (DC-14); only `usecases/transcribe-cloud.ts` sends audio; the server holds it in memory only | `no_media_routes.rs` (DC-14); `transcribe_route.rs`; `log_redaction.rs`; `cloud-transcribe.spec.ts`'s network cases |
| P-5 | No request during processing other than allowlisted beacons, the `GET /healthz` warm-up, **in cloud mode the one `POST /transcribe`**, and (plan on) the one `POST /plan` | Use-cases between import and `ready` import no `net` module except `transcribe-cloud.ts` and `request-plan.ts` | `cloud-transcribe.spec.ts` assertion "device mode sends nothing but `/events`" |
| P-12 | Unchanged rule; F-3 keeps the landing wording inside it ("never uploaded" is factual; "never leaves your device" stays banned) | | |
| P-16 (new) | The server never stores or logs a clip's sound or the words returned for it | `transcribe/` writes no table; `log.rs` logs no body | `log_redaction.rs` (marker word and marker bytes); a grep test that no `sqlx` call exists under `server/src/transcribe/` |

### 10.2 Invariants

- **INV-3,** revised: "Import, preview and review complete with the API unreachable. Processing waits on the API only in cloud mode, within D-374's bounds, and completes on the device when it cannot."
- **INV-4,** revised: "The server has no route that accepts more than 64 kB or a media content type, **except `POST /transcribe`: `audio/ogg` only, at most 1 MiB, bearer only** (DC-14)."

### 10.3 DC-14, the new row of TS §41.2

| # | Release | What changes | What stays | Proof |
|---|---|---|---|---|
| DC-14 | Cloud transcription | `media_guard` admits `POST /api/v1/transcribe` with `audio/ogg` (DC-14 is the only route and type); that route's body limit is 1 MiB. `no_media_routes.rs` names it as the one exception: its other media types get 415, `audio/ogg` passes the guard (401 without a bearer, 413 over 1 MiB with one). `/healthz`'s body gains `cloud_transcribe` (`notify_me.rs`'s exact-body assertion gains the field). Founder's decision of 2026-11-06 | Every other route: 415 for every media type, no body over 64 kB, the same assertions. The device path, its frames, samples, transcripts and exports | `no_media_routes.rs`, `router.rs`'s unit tests, `transcribe_route.rs`; every TS §41.1 proof unchanged |

### 10.4 PS, revised

- **Positioning** (line 3): "All of it renders on your computer: your video is not uploaded. To write captions fast, the clip's sound is sent for transcription when you choose Cloud, which is the default for an account."
- **§9.8, §12.6:** cloud transcription is built now, default for signed-in people, chosen in the drop zone; the per-clip prompt is replaced by the card shown before every drop (D-344); the provider is named in the settings table; retention: none (F-2); the legal review is a launch-gate item (F-4).
- **§12.7:** the table of §9.3.
- **§17:** "Legal-review gate": the cloud fallback's own review moves to the launch gate, with the founder's written acceptance until then (F-4).
- **§10 J4, J6:** J4's download happens only on the device path; J6's line by mode.

---

## 11. Tests

### 11.1 Server

| File | Cases |
|---|---|
| `transcribe_route.rs` (new; real Postgres for users and sessions, `FakeTranscriber`) | No bearer: 401, and a 2 MiB body is not read (the fake sees nothing); `TRANSCRIBE_ENABLED=false`: 503 `transcribe_unavailable` and `/healthz` `false`; each of `audio/wav`, `audio/webm`, `video/mp4`, `application/octet-stream`, `multipart/form-data`, `application/json`: 415; `audio/ogg` over 1 MiB: 413; random bytes, a stereo `OpusHead`, no BOS, 0 ms, 92,001 ms: 400; a good file: 200 with the fake's words, and the fake received the exact bytes; fake `Unavailable`: 503; fake `BadAudio`: 400; the 21st request of a user in a day: 429 with `retry_after_secs`, another user still 200; budget spent (a `Budget` built small in this test): 503 and the fake not called; `/healthz` `true` when enabled |
| `groq_adapter.rs` (new; no network: a local axum mock on `127.0.0.1`) | The request: path, bearer, multipart fields and their values, file name and type; each `fixtures/groq/*.json` parses to its expected words (count, first and last times, punctuation restored); 429, 500, 503 and a 26 s stall: `Unavailable`; 400 and 413: `BadAudio`; a body that is not JSON: `Unavailable`; no log line holds a fixture word |
| `align.rs`, `ogg.rs`, `budget.rs` (unit, inline) | §7.5's properties and cases; §7.6's cases; hour and day roll-over at UTC boundaries, charge and refuse |
| `no_media_routes.rs` (DC-14) | `the_table_is_read` sees the new route; every route but `/transcribe` exactly as today; `/transcribe`: the four other media types 415, `audio/ogg` 401 without a bearer, 413 over 1 MiB with one; a new list `AUDIO_BODY` beside `READ_BODY` and `IGNORE_BODY`, so a new POST route still fails until named |
| `notify_me.rs` | `/healthz`'s body is `{"ok":true,"version":"test-sha","cloud_transcribe":true}` |
| `log_redaction.rs` | The transcribe flow with a marker word and a marker byte run in the Ogg body: neither in captured logs |
| `rate_limit.rs` | The two new groups at their limits |
| `analytics_allowlist.rs` | `asr_backend: "cloud"` accepted; `asr_cloud` with each outcome accepted, with an extra field or a string refused |
| `errors.rs` (unit) | 14 codes in order |

### 11.2 Client (Vitest)

| File | Cases |
|---|---|
| `ogg-opus.test.ts` | CRC of a known page against a bit-by-bit reference implementation in the test; header fields; lacing of a 600-byte packet (255, 255, 90); a page breaks at 50 packets; BOS and EOS flags; granules rise; the last granule is pre-skip + total; a reader written in the test parses every page back and the packets are byte-identical |
| `cloud-audio.test.ts` | `dropSilentWords`: a word over silence dropped, a quiet real word kept (the reference clip's levels as synthetic PCM), a word starting at the end dropped, a word straddling the end clamped and judged on the part inside; with a fake `AudioEncoder` (as `export-loop.test.ts` fakes WebCodecs): 1 s `AudioData` in, 20 ms packets out; the pre-skip from an `OpusHead` description, and 312 without one; unsupported config → `{unsupported: true}`; the output's duration equals the input's |
| `transcribe-cloud.test.ts` | Each row of §6.3 to its outcome; unknown availability waits and then goes on, or gives `unavailable` after 45 s (fake clock); `unsupported`; a cancel aborts the request; `asr_cloud` tracked only when available |
| `run-pipeline.test.ts` | **Existing cases unchanged.** New: cloud success never calls `modelReady`, `asr.load` or `asr.transcribe`; cloud fallback calls them once, in today's order, and shows one toast; signed out, unavailable, flag off or choice device: today's calls only |
| `http.test.ts` | `rawBody` sends the exact bytes with its content type and no JSON; `attemptTimeoutMs`; 503 `transcribe_unavailable` is an answer, not retried |
| `api-client.test.ts` | `transcribe()` refreshes once on 401 and repeats once; `checkHealth()` parses the field and treats its absence as `false` |
| `cloud-store`, `prefs-repo` | Defaults; a stored `"device"` survives a reload (fake IndexedDB, as the existing repository tests) |

### 11.3 Crate

`offcut-api-types`: the new types' `ts-rs` output; `HealthResponse` without the field parses with `false`.

### 11.4 End to end

`fake-api.ts` gains options, defaults unchanged (so every existing spec behaves as today): `healthz.cloudTranscribe?: boolean` (default: the field absent), `transcribe?: { response?: object; status?: number; code?: string; delayMs?: number }` (default: 404, which no existing spec reaches). It records a request's raw body for `/transcribe` (`request.postDataBuffer()`).

`cloud-transcribe.spec.ts`, project `media`, clip `speech_60s_portrait.mp4`, serial where cases share a page:

| # | Case | Asserts |
|---|---|---|
| 1 | **Identity:** signed out, `cloudTranscribe: true` | The card shows signed-out state; no `/transcribe` request; the stored transcript and events equal those of the same clip with the field absent (read from IndexedDB in both runs) |
| 2 | Signed in, cloud (default) | One `/transcribe` request, `content-type: audio/ogg`, body under 1 MiB, starting `OggS`; `ffprobe` on the saved body: Opus, mono, duration within 40 ms of the clip's audio; the clip reaches `ready`; the stored transcript's `model_version` is `groq:whisper-large-v3-turbo` and its word count is the fixture's; no model file was requested from the asset host; the processing line was `processingCloud`; `asr_cloud` `ok` and `stage_timing` `asr_backend: "cloud"` sent |
| 3 | Signed in, "On this device" chosen, page reloaded | The choice is still "On this device"; no `/transcribe` request; requests during processing are `/events` only (P-5) |
| 4 | Fallbacks: `/transcribe` answers 503 `transcribe_unavailable`, then 429, then stalls 31 s | Each run reaches `ready` on the device; one toast each, the 429 one is `limitToast`; `asr_cloud` outcomes `unavailable`, `limited`, `timeout` |
| 5 | Export after a cloud transcript (signed in, Free) | 720 × 1280 with the watermark; `verify_mp4.py`'s 11 checks pass |
| 6 | The card's axe scan, both states, 0 violations; the drop zone with `cloudTranscribe` absent has no card |

The fixture `tests-e2e/fixtures/cloud/speech_60s_portrait.json` is the `TranscribeResponse` the real adapter produced from the real Groq response for that clip's audio (P163), so case 2 runs the real shape.

**The two-path proof of this release.** Path 1: every protected suite as it is: the fake's `/healthz` has no field, so every clip runs on the device, as at `68d2ae2`. Path 2: `cloud-transcribe.spec.ts`. Case 1 proves that turning the server's switch on changes nothing for a signed-out person.

---

## 12. CI, deploy and rollout

### 12.1 CI

`ci.yml` changes in nothing but what the new files need: `server/tests/fixtures` is already present; `cloud-transcribe.spec.ts` is in `MEDIA_SUITES`, so CI's `e2e-media` runs its `@no-gpu`-eligible cases as the other media specs. No secret is added to CI: no test reaches Groq. `SQLX_OFFLINE` is unaffected (no new query).

### 12.2 Rollout (D-381)

1. **G 10.2 (founder), before the merge:** Render → `offcut-api` → Environment: `TRANSCRIBE_ENABLED=false`, `TRANSCRIBE_API_KEY=<from .env.deploy>`, `TRANSCRIBE_API_BASE_URL=https://api.groq.com/openai/v1`. Saving restarts the current image, which ignores them.
2. **G 10.3 (founder):** Groq console: Zero Data Retention on (F-2), and a spending limit if the Dev tier is ever chosen.
3. **G 10.4 (founder):** mark the pull request ready; merge; watch the `main` run (`ci`, `e2e-media`, `deploy-api`, `deploy-web`).
4. **G 10.5 (agent):** `/healthz` reads `"cloud_transcribe": false`; `@smoke` passes; the live drop zone shows no card; the app is today's.
5. **G 10.6 (founder):** set `TRANSCRIBE_ENABLED=true` in Render; wait for the restart.
6. **G 10.7 (agent):** `/healthz` reads `true`; an unauthenticated `POST /api/v1/transcribe` with a 900 kB `audio/ogg` body through the Vercel origin answers 401 from our API (TE-C5: the rewrite carries the body); a 2 MB one answers 413 from our API, not from Vercel.
7. **G 10.8 (founder):** the live journey: sign in, drop the sample clip with Cloud on, see captions in seconds, export; switch to "On this device", drop again; sign out, see the signed-out card.
8. **G 10.9 (founder or agent at the founder's request):** tag `cloud` on the merge commit.

### 12.3 Kill switches and rollback

| Problem | Action | Time |
|---|---|---|
| Groq misbehaves, cost, abuse | `TRANSCRIBE_ENABLED=false` in Render | about 1 min; no code |
| The client side misbehaves | Redeploy with `OFFCUT_FEATURES_OFF=cloud_transcribe` (a Vercel environment variable, then a redeploy from a pushed commit, never the Redeploy button: memory) | minutes |
| Anything else | Revert pull request (`git revert -m 1`), as V6 §18 | one CI run |

No migration, so the old binary runs on the same database.

---

## 13. Experiments (`experiments.md` records them)

| ID | Who, when | How | Pass | If it fails |
|---|---|---|---|---|
| TE-C1 | Agent, P161 (D1, Chrome and Edge) | A throwaway page under `vite preview`: `AudioEncoder.isConfigSupported` for Opus at 16 kHz and 48 kHz, mono, 24/32/48 kbps; encode 60 s of the reference clip's `pcm16`; read the description (is it an `OpusHead`, what pre-skip), chunk durations, time to encode | 16 kHz supported in both browsers; 90 s encodes in under 2 s | 48 kHz input by interpolation (§8.3); if Opus is missing altogether: stop and ask (recommendation: 16-bit WAV with a 3 MiB limit, and the bandwidth cost of §14) |
| TE-C2 | Agent, P161, needs F-1 | The reference clip's audio as FLAC (lossless) and as Opus 16, 24, 32 kbps (`ffmpeg`), each sent to Groq once from D1 with the real key in the agent's shell; record the raw responses (scrubbed); word error rate against Script A's text (`fixtures/speech/README.md`); median absolute difference of word starts against the device recognizer's transcript of the same audio; latency | The chosen bitrate's WER within 1 point of FLAC's; median start difference from the measured speech onsets under 60 ms (as read: the device recognizer itself is 264 ms late, so the comparison is with the audio, not with it); latency under 5 s for 60 s. **Read 2026-11-06: passed** (6.2% vs 5.5%, 33 ms, 0.98 s) | A higher bitrate (cap 64 kbps, still under 1 MiB); a start offset over 60 ms: stop and ask |
| TE-C3 | Agent, P169, needs F-1 | The reference clip and the five derived speech fixtures, cloud and device: the events found (kind, figure) and the sentence count | Cloud finds every `number_reveal`, `list_reveal` and `from_to` the device finds on the reference clip, or the difference is explained by a better transcript | Stop and ask (a lost number is a lost visual) |
| TE-C4 | Agent, P169 | Drop to `ready` on D1: cloud; device with the model cached; device with no model | Cloud at least 2× faster than device with the model cached (assumption) | Recorded; not a stop |
| TE-C5 | Agent, G 10.7 | Through Vercel's rewrite to Render: a 900 kB body reaches our API; one real cloud clip's request time | 401 from our API; the live clip under 10 s for the request | Lower the body limit or the bitrate; stop and ask |

---

## 14. Risks

| Risk | Way out |
|---|---|
| Groq's free tier changes or its limits differ from the summaries | Read the console in P161; the budget is a constant in `budget.rs`; the Dev tier at 0.04 USD/hour with a spending limit |
| Render's free bandwidth (third-party sources say it fell to 5 GB a month in 2026) | 0.37 MB per clip; at the daily budget's ceiling (about 300 clips) roughly 3.4 GB a month of outbound audio to Groq; watch Render's bandwidth page monthly; a paid instance or the kill switch |
| Vercel Hobby's origin transfer (the audio passes the rewrite) | The same 3.4 GB at the ceiling (assumption: Hobby includes about 10 GB of Fast Origin Transfer; read in P170 from Vercel's usage page) |
| Render sleeps; the first clip after 15 minutes waits for the wake | `start-app` wakes it at page load; the use-case waits up to 45 s, then the device |
| Groq's words lack punctuation, or split numbers differently from the device recognizer | D-360, read in P161, tested with real fixtures; TE-C3 |
| Word times offset by the encoder's pre-skip | §8.3's pre-skip from the description; TE-C2's offset reading |
| A person signed in on a shared computer has their sound sent without noticing | The card is visible before every drop with the selected mode and its line; the choice is remembered per device |
| The landing claim reads as broader than the truth | F-3's wording names only the video; the card, settings and Privacy Policy say what is sent; the landing links to "What leaves your device" |
| An existing test meets the new card | D-366: no card unless `/healthz` says `true`, and the fake never does by default |
| `messages.ts` passes 700 lines | D-380 |
| A late webhook-style surprise: the server is updated but the client is not, or the reverse | The availability gate (D-366) and the analytics rule (D-377) make both orders safe |
| Groq retains audio | F-2 before the merge; the policy's sentence waits for it |

---

## 15. Founder decisions still open, with recommendations

| # | Question | Recommendation |
|---|---|---|
| F-1 | May the agent send the **reference clip's audio** (Script A, your voice, already public as the sample clip) and its derived fixtures to Groq during P161, P163 and P169 (a few minutes of audio in all)? `v3testclip.mp4` and corpus clips never | **Yes.** Without it the adapter is written from documents, which V6 D-279 refused for Dodo |
| F-2 | Turn on **Zero Data Retention** in Groq's console (Data Controls) and tell the agent | **Yes, before P166.** The Privacy Policy says nothing is kept; that must be true |
| F-3 | The landing wording. Rule P-12 (TS §25.1, PS §17) bans "never leaves your device" before a lawyer's review. Choose: (a) "Your video is never uploaded." with "It is edited, rendered and exported in your browser, on your computer."; (b) "Your video never leaves your device." (overrides P-12) | **(a).** Same strength, factual, inside the rule |
| F-4 | Legal review: approve the Privacy Policy, Terms and card wording yourself at G 6.6, and keep a lawyer's review of the cloud-transcription wording on the launch checklist (V9) | **Yes** |
| F-5 | The rollout of §12.2: merge with `TRANSCRIBE_ENABLED=false`, then switch it on after the checks | **Yes** |

---

## 16. Hand-over to V6.1

`Transcriber`, `POST /transcribe`, `encodeForCloud`, `fromCloud`, `ogg-opus.ts`, `transcribe-cloud.ts`, `cloud-store`, `features.ts` and the availability gate are frozen for V6.1. V6.1's windowed transcription of long sources calls `encodeForCloud` and `POST /transcribe` once per window of at most 90 s (the route's 92 s rule stays), offsets each window's words, and checkpoints them as it does device windows; the budget then counts a long video's windows. V6.1 adds its flags to `features.ts`. "Long videos need an account at the drop" (V6 §19) and "cloud needs an account" are the same check.

---

## 17. Exit checklist

- [ ] Every TS §41.1 proof holds; goldens and snapshots have an empty diff; DC-14 is the only row used.
- [ ] Counts at or above D-339's floor.
- [ ] TE-C1 to TE-C5 recorded in `experiments.md`.
- [ ] `cloud-transcribe.spec.ts` passes, all cases; the protected suites pass unchanged in the one full media run.
- [ ] F-1 to F-5 answered and recorded.
- [ ] Copy approved by the founder (G 6.6); Groq's Zero Data Retention on (F-2).
- [ ] Merged; deployed with the switch off; checks of G 10.5 passed; switched on; checks of G 10.7 passed; the founder's live journey done.
- [ ] Copy-back done (§18). Tagged `cloud`.

---

## 18. Copy-back to the specs (P170, plus TS §5 as files appear)

| Spec | Change |
|---|---|
| TS §1, §3 | The transcription provider as a server-side dependency |
| TS §5 | Every new file of §4; `features.ts` built |
| TS §7 | `fetch` rule unchanged; the two lint edges; P-5's use-case exception |
| TS §16 | "Seams (not built now)" → "Cloud transcription (built, cloud transcription release)": the worker handlers, the use-case, the availability gate; TDR's "Rejected: cloud ASR" becomes "Cloud ASR is the signed-in default, through our API" |
| TS §22.1 | The route's row (§6.1); `/healthz`'s field |
| TS §22.6 | `AsrBackend::Cloud`; `asr_cloud` |
| TS §22.8 | The three variables |
| TS §24.1 | From the server: the transcription provider's host (`api.groq.com`), `POST /transcribe` only |
| TS §24.5 | The two buckets and the budget |
| TS §25.1, §25.3 | P-1, P-5, P-16 as §10.1; the server inventory: nothing stored |
| TS INV-3, INV-4 | §10.2 |
| TS §37 | TE-C1 to TE-C5 |
| TS §41.2, §41.3, §41.5 | DC-14; `features.ts` exists from this release; a row "Cloud transcription: `Transcriber`, `/transcribe`, encoder, availability gate → V6.1" |
| TS rejected lists (lines "cloud transcription (P-1)", "Cloud transcription (audio upload), even temporarily") | Rewritten as built |
| PS line 3, §9.8, §12.6, §12.7, §17, J4, J6 | §10.4 |
| BP §0 | The release as built, its prompts and its tag |
