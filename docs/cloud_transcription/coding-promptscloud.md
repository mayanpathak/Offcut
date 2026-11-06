# Offcut cloud transcription — coding-promptscloud.md (Prompts 161–170)

Source documents: `docs/cloud_transcription/cloudimplementation.md` (written **§n**; its decisions are **D-n**, D-336 to D-382; its open founder questions **F-n**) and `docs/cloud_transcription/cloudbuildguide.md` (written **G n.m**, **G item n** for an item of its "Read this first"). `TS`, `PS`, `BP` are the v1.3 specs. These prompts were written after reading the code at `68d2ae2` on 2026-11-06.

**How to use.** Paste the **Standard Agent Block (SAB)**, then one prompt, into your coding agent. Run them in order: Setup, then 161 to 170. Do not start a prompt until the one before it has its commit and every acceptance box ticked. Commits are titled `CT P<nn>: <title>`. Steps marked **Human** are the founder's. Each prompt's **Effort** line is the reasoning effort to set: High where privacy, a protected proof or a third party's format is at stake; Medium for records and specified screens.

**Design skill.** In P166, if the agent has a frontend or design skill, it loads it before writing interface code. V4's tokens, primitives and copy rules win over any default it suggests.

---

# Standard Agent Block (SAB) — prepend to every prompt

**Every terminal (Git Bash, never PowerShell or WSL)**

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut && set -a; . ./.env; set +a
export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:/c/Users/Mayan/AppData/Local/Microsoft/WinGet/Links:$PATH"
export BENCH_DEVICE=d1
BASE=68d2ae2
git branch --show-current     # cloud-build, always (Setup creates it)
```

Server tests need Postgres on port 9000, a Windows service on D1, not Docker (V6 D-305). `gh auth status` shows a login.

**Before modifying anything**

1. Read the plan sections and guide steps this prompt cites, every file it touches, and the previous prompt's entry in `docs/cloud_transcription/cloudchangelog.md`.
2. Search before creating (`git grep -n "<name>"`). `offcut-api-types` holds every DTO and `ApiErrorCode`; `server/src/rate_limit.rs` the route groups; `auth/extract.rs` `AuthUser`; `net/api-client.ts` `authed()`; `workers/asr/word-timestamps.ts` `postProcess`; `wasm/load-core.ts` `normalizeTranscript`; `state/ui-store.ts` `pushToast`.
3. Verify that the previous prompts' outputs behave as this prompt assumes. If not, make the smallest compatible correction and report it.
4. **Precedence.** A decision D-336 to D-382 is binding, also where it refines TS or PS; D-1 to D-335 stand unless one of these replaces them. Otherwise TS wins over the plan and PS over both. If sources disagree in a way these do not settle, or a signature is missing, **stop and list it**.
5. **TS §41 binds.** Any protected proof that would change without DC-14, any new runtime, browser host or paid service, any experiment whose row offers a choice, any change to what the product promises about privacy beyond §9 and §10: **stop and ask, with a recommendation.**

**During implementation**

1. Implement only this prompt's scope. A new file under `crates/`, `server/`, `web/`, `scripts/`, `verify/` or `.github/` that TS §5 does not list fails `check-file-tree.mjs`; add its row in the same commit, only for the files §4 names.
2. **Frozen:** G 0.4's check stays empty. Goldens and snapshots do not change. No migration. `whisper-runtime.ts`, `word-timestamps.ts` and the existing members of `AsrWorkerApi` do not change (D-341).
3. **Reopened contracts and declared changes are exactly:** DC-14 (`media_guard`, `no_media_routes.rs`, `/healthz`'s body and `notify_me.rs`'s one assertion); D-373 (`http.ts`); D-375 (INV-3's text); the additive members of `protocol.ts`; the landing counts of §9.6. Say **REOPENED CONTRACT** or **DECLARED CHANGE** in the entry for each.
4. No source file over 700 lines above its test module. Rust non-test code: no `unwrap`, `expect`, `panic!`, slice indexing. TypeScript: no `any`, `!`, `@ts-ignore`, empty `catch`, or inline style. No disabled lint, no weakened or deleted test. Dependencies only as §3.1 lists.
5. User-facing text only in `copy/` (new strings in `copy/transcription.ts`, D-380). `fetch` only in the allowed files. A temporary file is `zz-…` or `zz_…` and is deleted before the gate.
6. **Secrets.** Never print, log, commit or paste a value from `.env`, `.env.deploy`, a token, a cookie or a link. A script that uses `.env.deploy` sources it in a subshell and prints names, status codes and times only. `git grep -nE "gsk_[A-Za-z0-9]{20}"` before every commit: no output. **Never query the production database.** A server log line names a `kind`, never provider text or a word of a transcript.
7. **Privacy of recordings (D-379).** Real Groq calls only after F-1, only with the reference clip's audio or its derived fixtures or a synthetic tone, and each one listed in the entry (file, seconds of audio, status). **Never** `testclips/v3testclip*.mp4`, `testclips/WIN_*`, or a corpus clip. Never stage `testclips/**`, `fixtures/.gen/**`, `corpus/**`, `web/test-results/**` or `.env*`. A transcript's words appear in a commit only inside the two fixture files of §4, whose words are Script A's, already public in `fixtures/speech/README.md`.
8. Long runs start detached and are read from a log tail. Append to the changelog with the Edit tool, never a heredoc holding backticks.
9. **Branch.** Commit on `cloud-build` only. Push only where the prompt says. Never push to `main`, merge, tag, mark the pull request ready, change Render, Groq or Vercel settings, or press Vercel's Redeploy button.

**When existing code disagrees with the plan.** If it blocks the prompt, fix the minimum and record why. If not, report it under "Found, not fixed".

**The gate.** G 0.5's tier, the new case alone first; then:

```bash
git branch --show-current
git status --porcelain | grep -c "zz[-_]"        # 0
git grep -nE "gsk_[A-Za-z0-9]{20}" | wc -l                        # 0
env | grep -c VITE_ENTITLEMENT_TEST_PUBLIC_KEY   # 0 before the commit
# G 0.4's two commands: no output
git ls-files --cached | grep -E '(^|/)\.env'     # no output
```

Counts never fall below the previous entry's; the floor is D-339's. Add a short entry to `cloudchangelog.md` in the same commit: what changed, what differs from the plan, **REOPENED CONTRACT** and **DECLARED CHANGE** lines, every real Groq call, counts, what is open, "Found, not fixed". Rewrite its two top tables when an item opens or closes. Commit with the trailer `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Reply with files changed, commands and results, deviations, **Human** steps open, issues not fixed.

---

# Who does what

| When | Human step |
|---|---|
| Before Setup | Answer F-1 to F-5 (G 0.1); turn on Groq's Zero Data Retention (F-2) |
| P161 | Nothing, if F-1 is yes. A stop-and-ask if Groq's words carry no times |
| P166 | Read `~/cloud-review/`, the copy, the legal sections and the landing diff; say yes or give changes (G 6.6) |
| P169 | Sign in locally once; walk the local build (G 9.2) |
| P170 | Render variables with the switch off (G 10.2); confirm Zero Data Retention (G 10.3); ready and merge (G 10.4); switch on (G 10.6); the live journey (G 10.8); the tag (G 10.10) |

---

# Prompts

## Setup prompt (CT 0/10) — The branch, the documents, the floor

**Effort: Medium.**

**Objective.** `cloud-build` exists with this release's documents and V6's closing entry as its first commits, the floor is read, and a draft pull request lets CI run.

**Read first.** §1A, §2.1; G Phase 0; `docs/design/designchangelog.md` Entry 16 (the last counts).

**Do.**

1. G 0.2: fetch; check `origin/main` is `68d2ae2`; `git switch -c cloud-build origin/main`; `git cherry-pick 8d94a06`; check the pick touched `docs/v6/v6changelog.md` only.
2. G 0.3: TS §5's five rows for `docs/cloud_transcription/` (`cloudimplementation.md`, `cloudbuildguide.md`, `coding-promptscloud.md`, `cloudchangelog.md`, `experiments.md`), beside the `docs/design/` rows, each with a few words of what it is. `node scripts/check-file-tree.mjs` passes.
3. CT CL Entry 1: the branch and base; the pick; the founder's answers to F-1 to F-5 so far, word for word (an unanswered one stays an open item); Zero Data Retention (F-2) yes or not yet.
4. G 0.6: the three quick counts. Record them in Entry 1.
5. Commit `CT setup: the cloud transcription documents`; push `origin cloud-build`; `gh pr create --draft` (G 0.3).

**Acceptance.**

- [ ] `git log --oneline origin/main..cloud-build` shows the pick and the setup commit only.
- [ ] Counts recorded: Rust 817 / 0 / 4, Vitest 522, `pnpm e2e` 57, or the entry says why not.
- [ ] Draft pull request open; its body ends with the attribution line.

---

## Prompt 161 (CT 1/10) — Probes: Groq's real answers, the browser's encoder

**Effort: High.**

**Objective.** Every unknown the code depends on is read from the real thing before any code is written (as V6 D-279 did for Dodo): what Groq's `verbose_json` holds and how good it is on our audio at each bitrate, and what Chrome's and Edge's WebCodecs Opus encoder accepts and returns.

**Read first.** §7.4, §7.5, §8.3, §13 (TE-C1, TE-C2); G Phase 1, G items 1, 2, 18; `fixtures/speech/README.md` "Script A" and "The V2 reference clip as recognized".

**Stop before starting** if F-1 is not yes. Write the open item and ask.

**Do.**

1. `pnpm fixtures` if `fixtures/.gen/speech/speech_60s_portrait.mp4` is missing. G 1.1's files in the scratchpad.
2. G 1.2: four real calls (FLAC, Opus 16, 24, 32 kbps), 60 s each. Read and record the six points of G 1.2 in `experiments.md` (TE-C2). For WER, normalise both texts (lower case, letters and digits, numbers as digits) and count substitutions, insertions and deletions against Script A; show the counts, not only the rate.
3. G 1.3: the two scrubbed Groq fixtures and `ffmpeg_tone_3s.ogg` copied into `server/tests/fixtures/` (they are committed in P162 with their rows; leave them untracked here, or commit them here with their TS §5 rows if `check-file-tree.mjs` allows a folder row: either is fine, say which).
4. G 1.4: the throwaway encoder page in Chrome and Edge (TE-C1). Delete it.
5. If the chosen bitrate is not 32 kbps, or 16 kHz input is unsupported, or Groq's `words` already carry punctuation: write which plan lines change (D-360, D-370, §8.3) as a short "As read" note under the decision in the plan, and say so in the entry.

**Stop and ask** if: Groq returns no `words` or no times; the median sentence-start difference against the device transcript is over 60 ms; Opus is unsupported in either browser; the free tier's audio limits in the console are below D-346's budget.

**Acceptance.**

- [ ] TE-C1 and TE-C2 recorded with numbers; the bitrate chosen with its reason.
- [ ] Every real call listed in the entry (file, 60 s, status, time); total under 300 s of audio.
- [ ] No `zz` file; no key in any file (`git grep -nE "gsk_[A-Za-z0-9]{20}"`).
- [ ] Commit `CT P161: the probes`.

---

## Prompt 162 (CT 2/10) — The transcribe route without a provider

**Effort: High.**

**Objective.** `POST /api/v1/transcribe` exists with every check of D-352 in order, the one INV-4 exception of DC-14, `/healthz`'s new field, the limits and the budget, all tested against `FakeTranscriber`. No Groq code yet.

**Read first.** §2.3, §3.2, §6, §7.1-§7.3, §7.6, §10.3, §11.1; G Phase 2, G items 5, 6, 7; `server/src/{router,rate_limit,config,error,state,main}.rs`; `server/src/billing/provider.rs` (the trait pattern); `server/tests/{common/mod,no_media_routes,notify_me,log_redaction,rate_limit,analytics_allowlist}.rs`.

**Do.**

1. G 2.1: the types; `pnpm gen:types`; commit `web/src/gen/api.ts` with them. `AsrCloud`'s doc line names what it tells (the settings table shows it).
2. G 2.2: config, `.env` names, `render.yaml`, state. `Config`'s `Debug` redacts the key; its test proves the value is not printed.
3. G 2.3: `transcribe/{mod,provider,ogg,budget,route}.rs`; the route in `router.rs` (TS §22.1's order: after `/notify-me`, before `/plan`'s place); `media_guard`'s exception with `media_type` factored out; `/healthz`'s field from `state.transcriber.enabled()`; two route groups; `AppError::TranscribeUnavailable` → 503; `UnconfiguredTranscriber` in `main.rs`.
4. The handler takes `Request` (or `Parts` and `Body`) and reads the body with `to_bytes(body, MiB)` **after** `AuthUser`, the per-user bucket and the switch (G item 6). Prove it: a 2 MiB body without a bearer answers 401 and `FakeTranscriber` saw nothing.
5. G 2.4: the tests of §11.1 that need no Groq. `budget.rs` is built with injectable limits (`Budget::new(hour_s, day_s)`), production values `pub const` in the same file; tests build a small one.
6. **DC-14 by the letter:** in `no_media_routes.rs`, keep every existing assertion; add `AUDIO_BODY: [(&str, usize); 1] = [("/transcribe", MiB)]`; in `every_route_refuses_media_and_uploads`, for `/transcribe` alone expect 415 for the four non-`audio/ogg` types and **not** 415 for `audio/ogg` (it gets 401, unauthenticated); in the body-limit test, a bearer and 1 MiB + 1 of `audio/ogg` gives 413. A new POST route still fails until named. Paste the diff summary into the entry.
7. `notify_me.rs`: the exact body with `"cloud_transcribe": true` (TestApp enables a fake). One more case there or in `transcribe_route.rs`: disabled → `false`.

**Acceptance.**

- [ ] `cargo test -p offcut-api-types`, `cargo test -p offcut-api --lib`, and the six server test files of G 2.4 pass; `pnpm check` passes.
- [ ] DC-14 recorded as a **DECLARED CHANGE** with the diff read line by line; `/healthz`'s body as **REOPENED CONTRACT**.
- [ ] `grep -rn "sqlx" server/src/transcribe` is empty (P-16).
- [ ] Commit `CT P162: the transcribe route, its limits and DC-14, without a provider`.

---

## Prompt 163 (CT 3/10) — The Groq adapter and the punctuation

**Effort: High.**

**Objective.** `GroqTranscriber` turns our Ogg file into `CloudTranscript` through Groq's real API shape, with the written forms restored to its words, every failure mapped, and nothing logged; proven on P161's real fixtures and by one real call.

**Read first.** §7.4, §7.5, D-358, D-359, D-360; `experiments.md` TE-C2; G Phase 3; `server/src/billing/adapter.rs` (client building, error handling without provider text).

**Do.**

1. G 3.1: `reqwest`'s `multipart`; `cargo deny check`; the fallback of D-358 if needed.
2. `transcribe/align.rs` first, test-driven: §7.5's properties and every case it lists, plus the cases P161's fixtures show (numbers, currency, percent, contractions as Groq writes them).
3. `transcribe/groq.rs`: `GroqTranscriber::new(config)`; the request of §7.4 with `TRANSCRIBE_API_BASE_URL`; `parse` (a pure function from the response bytes to `CloudTranscript`, so the fixtures test it without a network); error mapping of D-359; `model: "groq:whisper-large-v3-turbo"`. Times: `round(seconds × 1000)`, clamped to `u32`; non-finite or negative dropped and counted.
4. `main.rs`: `GroqTranscriber` when `TRANSCRIBE_ENABLED=true`, else `UnconfiguredTranscriber`; a client that cannot be built stops the boot with a `ServeError` naming the step only.
5. `groq_adapter.rs` (§11.1) with a local axum mock bound to `127.0.0.1:0`: it asserts the multipart fields it received and answers each fixture and each failure; one case stalls 26 s against a 25 s timeout (use a shorter timeout through a test-only constructor, `GroqTranscriber::with_timeout`, so the test takes about 1 s).
6. G 3.3: the one real call through the real adapter (F-1); the e2e fixture written by it; the temporary file deleted.
7. `log_redaction.rs`: a case through `groq.rs`'s mock with a marker word in the response: absent from logs.

**Acceptance.**

- [ ] `cargo test -p offcut-api --lib` and `--test groq_adapter --test log_redaction` pass; `pnpm check` passes; `cargo deny check` clean.
- [ ] `web/tests-e2e/fixtures/cloud/speech_60s_portrait.json` exists, written by the adapter, with its TS §5 row.
- [ ] The real call listed (60 s).
- [ ] Commit `CT P163: the Groq adapter, punctuation, one real call`.

---

## Prompt 164 (CT 4/10) — The flag and the Ogg Opus encoder

**Effort: High.**

**Objective.** `features.ts` exists with `cloud_transcribe`; `asr.worker` encodes `pcm16` into an Ogg Opus file that `ffprobe`, Groq and `ogg.rs` all read with the right duration, and turns cloud words into a `Transcript` through the same functions as device words.

**Read first.** §8.1, §8.3, D-341, D-365, D-370, D-371; `experiments.md` TE-C1; G Phase 4, G item 10; `web/src/workers/{protocol,asr.worker,rpc-worker}.ts`, `workers/asr/word-timestamps.ts`, `workers/render/export-loop.test.ts` (how WebCodecs is faked), `vite.config.ts`, `vitest.config.ts`.

**Do.**

1. G 4.1: `features.ts`, the define, Vitest's define if needed; TS §5's row updated.
2. `workers/asr/ogg-opus.ts` test-first (§8.3 step 4, §11.2): CRC table; page writer; lacing; granules; BOS and EOS; `OpusHead`; `OpusTags`.
3. `workers/asr/cloud-audio.ts`: `encodeOpus(pcm16, deps)` with `deps` holding `AudioEncoder` and `AudioData` constructors so the test fakes them; pre-skip from the description or 312 (or the value TE-C1 read); the 48 kHz path only if TE-C1 needs it.
4. `protocol.ts`: the two members of §8.3, `CloudWord` from `gen/api`. `asr.worker.ts`: `encodeForCloud` and `fromCloud` added to `handlers`; `load`, `transcribe`, `unload` unchanged (`git diff -w` shows only additions in that file). `fromCloud` runs `postProcess`, `normalizeTranscript`, `MIN_WORDS` exactly as `transcribe` does.
5. G 4.3: the real file read three ways; `client_tone_3s.ogg` committed with its row; `ogg.rs`'s test reads it.
6. Delete every `zz` file.

**Acceptance.**

- [ ] Vitest: `ogg-opus.test.ts`, `cloud-audio.test.ts` pass; the count rises.
- [ ] `ffprobe`: Opus, mono, 60 s within 40 ms; Groq: 200, WER like TE-C2's 32 kbps row; `ogg.rs`: 3,000 ms within 20 ms on the client's tone. Recorded.
- [ ] `git diff "$BASE" -- web/src/workers/asr.worker.ts web/src/workers/protocol.ts` is additions only.
- [ ] Commit `CT P164: features.ts, the Ogg Opus encoder in asr.worker`.

---

## Prompt 165 (CT 5/10) — Cloud mode and fallback in the pipeline

**Effort: High.**

**Objective.** A signed-in person whose choice is Cloud, on a server that says it is available, gets a cloud transcript with no model download; any failure finishes the clip on the device once with a toast; everyone else runs exactly today's calls in today's order.

**Read first.** §8.2, §8.4, §8.5, D-366 to D-378; G Phase 5, G items 3, 4, 8; `web/src/usecases/{run-pipeline,start-app,cancel-job}.ts`, `run-pipeline.test.ts`, `net/{http,api-client}.ts` and their tests, `state/{clip-store,auth-store,ui-store}.ts`, `persistence/{schema,db}.ts`, `eslint.config.js`.

**Do.**

1. G 5.1: `http.ts` (D-373) with tests first; `api-client.ts`'s `transcribe()` and `checkHealth()` with tests.
2. G 5.2: `cloud-store.ts`, `prefs-repo.ts`, `META_KEYS.transcription`, `set-transcription.ts`; `start-app.ts` sets availability from `checkHealth()` and loads the choice, awaiting neither.
3. G 5.3: `transcribe-cloud.ts` (§8.4) with `transcribe-cloud.test.ts` (§11.2) on a fake clock; `run-pipeline.ts` (§8.5): `chooseMode()`, the moved `hearOnDevice`, the cloud branch, the fallback; `clip-store`'s `asrMode`; the toast once per run; `asr_cloud` and `stage_timing` per D-377.
4. `eslint.config.js`: `run-pipeline.ts` → `transcribe-cloud.ts`. Any other edge the code needs is a stop-and-list.
5. `git diff -w "$BASE" -- web/src/usecases/run-pipeline.ts`: paste into the entry a short account showing the device block moved unchanged.
6. G 5.4 (Tier 2): keyed `pnpm build:e2e`; `pipeline-preview.spec.ts`, `model-download.spec.ts`, `cancel.spec.ts` pass unchanged.
7. Push 1; do not wait for CI.

**Acceptance.**

- [ ] `run-pipeline.test.ts`'s existing cases pass **unchanged**; the new cases of §11.2 pass.
- [ ] `http.ts` recorded as **REOPENED CONTRACT** (D-373); INV-3's text change (D-375) listed for copy-back.
- [ ] Tier 2's three specs pass; Vitest count rises.
- [ ] Commit `CT P165: cloud mode and fallback in the pipeline`; pushed.

---

## Prompt 166 (CT 6/10) — The card, the copy, the legal text, the landing wording

**Effort: High** (privacy wording and legal pages).

**Objective.** The drop zone shows the transcription card in its three states when cloud is available, and nothing new when it is not; every string of §9 is in place; the Privacy Policy, Terms, settings table and landing page say exactly what is true; the founder has approved them.

**Read first.** §9, §10.1, D-344, D-349, D-351, D-380; F-2, F-3, F-4's answers in CT CL; G Phase 6; `ui/components/{DropZone,ProcessingCard,SignInDialog,ToastHost,WhatLeavesTable}.tsx`, `ui/primitives/*`, `copy/{messages,landing}.ts`, `tests-e2e/landing.spec.ts`.

**Stop before starting** if F-3 is not answered (the landing wording), or if F-2 is not confirmed (then write everything but the Privacy Policy's "neither keeps" sentence, and leave an open item).

**Do.**

1. G 6.1: `TranscriptionChoice.tsx` (§9.1): a radiogroup with two radios, the line, the signed-out button opening `SignInDialog` through the existing use-case; rendered only under D-366's condition; inside `DropZone` in both variants.
2. G 6.2: `copy/transcription.ts`, imported in `messages.ts` as `landing.ts` is; `ProcessingCard`'s line by `asrMode`; the toasts; the settings rows and `events.asr_cloud`; the Privacy Policy and Terms sections; `legal.updated`; `landing.ts` per §9.6 with F-3's wording. No landing string mentions audio or sound (`git grep -n -i -E "audio|sound|voice is" web/src/copy/landing.ts` shows only today's voice-cleanup lines).
3. G 6.3: the landing cases that read the changed keys; counts updated by name; each a **DECLARED CHANGE** in the entry.
4. G 6.4, 6.5: captures to `~/cloud-review/`; axe 0 in all three states.
5. G 6.6: ask the founder for the approval; record it word for word; apply any change asked, and ask again if a change touches the legal text.

**Acceptance.**

- [ ] With the fake's default `/healthz`, the drop zone's DOM is today's (a case or a snapshot of its inner HTML proves it).
- [ ] axe 0 violations in the three states; the landing cases pass.
- [ ] `messages.ts` under 700 lines; `check-copy-codes.mjs` passes.
- [ ] The founder's approval recorded.
- [ ] Commit `CT P166: the card, the copy, the legal text, the landing wording`.

---

## Prompt 167 (CT 7/10) — End to end

**Effort: High.**

**Objective.** `cloud-transcribe.spec.ts` proves the cloud path, the fallbacks, the remembered choice, the export, the card's accessibility, and that a signed-out person's result is identical with the server's switch on or off; the protected suites stay as they are.

**Read first.** §11.4, D-366, D-377; G Phase 7, G items 3, 11, 12, 13; `tests-e2e/helpers/{fake-api,fixtures}.ts`, `tests-e2e/export-free.spec.ts` (how a signed-in export is set up), `playwright.config.ts`.

**Do.**

1. `fake-api.ts`: `healthz.cloudTranscribe`, `transcribe` options and the raw-body record, **defaults unchanged** (G item 3). Run `export-free.spec.ts`'s first case once to show nothing moved.
2. `cloud-transcribe.spec.ts`: the six cases of §11.4. Case 1 reads both runs' transcripts and events from IndexedDB and compares them as JSON. Case 2 saves the request body to the scratchpad and runs `ffprobe` on it (as `sourceDurationMs` does). Case 4 uses the fake's `delayMs` and the use-case's 30 s timeout; mark it `test.slow()`.
3. `playwright.config.ts`: the spec in `MEDIA_SUITES`; `@no-gpu` tags as the other media specs use them.
4. Tier 3 (G Phase 7). Run the new spec twice in a row.

**Acceptance.**

- [ ] The new spec green twice; Tier 3 green; `pnpm e2e` count unchanged or higher; media case count rises by six.
- [ ] Case 1 (identity) green: the switch changes nothing for a signed-out person.
- [ ] Commit `CT P167: cloud-transcribe.spec.ts`.

---

## Prompt 168 (CT 8/10) — The privacy clip without the cloud

**Effort: Medium.**

**Objective.** The landing page's privacy clip shows the video staying in the laptop, with no cloud, and the same sentence; it is uploaded once to a new path and wired in.

**Read first.** §9.7, D-350; G Phase 8, G item 17; `docs/design/designimplementation.md` §9; `../offcut-remotion/src/clips/Privacy.tsx`, `clips.json`, `scripts/render-all.mjs`; `scripts/upload-assets.sh`; `net/asset-fetch.ts`.

**Do.**

1. Edit `Privacy.tsx`: remove the cloud outline and the dotted paths; keep the laptop, the file icon and its glow; add a small padlock on the file icon in the accent colour; keep "Your video stays on your computer." and the 6 s seamless loop. Preview it in Remotion Studio and read it by eye.
2. G Phase 8's render, `ffprobe` checks and loop check (first and last frames). The `offcut-remotion` change is outside this repository: record its diff summary in the entry.
3. The upload, once (`grep -c '^ASSET_S3_' .env.deploy` first, a count only). The new path into `LANDING_CLIPS.privacy`; the new poster into `ui/assets/landing/privacy.webp` (under 80 kB).
4. The landing cases that load clips (L-6) and the privacy section's case. Push 2; do not wait.

**Acceptance.**

- [ ] The clip passes designimplementation §9.2's checks; no audio stream; under 1.5 MB.
- [ ] One upload; the printed path recorded; nothing on the host deleted or overwritten.
- [ ] Commit `CT P168: the privacy clip without the cloud`; pushed.

---

## Prompt 169 (CT 9/10) — Experiments and the founder's walk

**Effort: Medium.**

**Objective.** Cloud and device are compared on real audio for what the product shows (events) and for speed, and the founder has walked the local build.

**Read first.** §13 (TE-C3, TE-C4); G Phase 9; `fixtures/speech/README.md` "The derived clips as recognized".

**Do.**

1. G 9.1's two terminals (real Groq key and real mail relay from `.env.deploy`, never printed). The founder signs in once locally.
2. TE-C3: the reference clip (`testclips/speech_scriptA_landscape_720p.mp4` if present, else the cached reference) and the five derived clips, cloud and device: events (kind, figure), sentence and word counts. TE-C4: drop to `ready`, cloud, device with the model cached, device with none (clear the model through the settings page or a fresh profile).
3. Record in `experiments.md`. A lost number, list or from-to on the reference clip: **stop and ask**.
4. G 9.2: the founder's walk; record the answer.

**Acceptance.**

- [ ] TE-C3 and TE-C4 recorded; every real call listed (about 6 × 60 s).
- [ ] The founder's walk recorded.
- [ ] Commit `CT P169: experiments and the walk`.

---

## Prompt 170 (CT 10/10) — Copy-back, merge, switch on, tag

**Effort: High.**

**Objective.** The specs say what was built; the one full media run passes; the founder merges with the switch off; the deploy is checked; the founder switches it on; the live path is checked; the release is tagged.

**Read first.** §12, §17, §18; G Phase 10; V6's and the design release's closing entries.

**Do.**

1. G 10.1: the grep; copy-back §18 (TS, PS, BP), each row ticked in the entry; BP §0's paragraph rewritten as built; Tier 4 once (keyed build, `--workers=4`, failures rerun alone); push 3 and watch CI in the background.
2. Ask the founder for G 10.2 and G 10.3, and wait for "done".
3. Ask the founder for G 10.4; when merged, watch the `main` run; one re-run of a known flake only (lavapipe SIGSEGV in golden frames, or known issue 3).
4. G 10.5's checks; record.
5. Ask the founder for G 10.6; then G 10.7's checks (TE-C5) with `curl` and a generated 900 kB / 2 MB `audio/ogg` body (random bytes after a valid Ogg first page are fine: the 401 and 413 come before any parsing); record.
6. Ask the founder for G 10.8; record what they saw.
7. G 10.9: the closing entry on `cloud-build` (counts, TE-C5, the journey, open items, "Found, not fixed").
8. G 10.10: the founder tags, or the agent tags if the founder asks in writing.

**Acceptance.**

- [ ] §17 ticked, or each open box named in the closing entry.
- [ ] Production: `/healthz` `true`; the founder's live journey done; nothing else changed for signed-out people.
- [ ] Tag `cloud` on the merge commit.
- [ ] Commit `CT P170: copy-back and the close`.
