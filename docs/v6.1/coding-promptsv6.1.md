# Offcut V6.1 "Long source" — coding-promptsv6.1.md (Prompts 171–182)

Source documents: `docs/v6.1/v6.1implementation.md` (written **§n**; its decisions are **D-n**, D-383 to D-432; its founder questions **F-n**, all answered "yes to all" on 2026-11-08) and `docs/v6.1/v6.1buildguide.md` (written **G n.m**, **G item n** for an item of its "Read this first"). `ON §n` is `docs/v6.1/overnight-run.md`. `TS`, `PS`, `BP` are the v1.4 specs; `V6 CL` is `docs/v6/v6changelog.md`, this release's changelog. These prompts were written after reading the code at `afb82bf` on 2026-11-08.

**How to use.**

- **Overnight (the founder's choice for V6.1).** The founder pastes `overnight-run.md` into one fresh session and goes to sleep. That file makes the agent read this one and run **Setup, then 171 to 182, in order**, each with the SAB below, under ON's authorisations and safe paths. No prompt waits for a person: every founder step is either done before the night (ON §0), replaced by the agent (ON §5), or left for the morning (ON §8).
- **By hand.** Paste the SAB, then one prompt. Same order; where a prompt says **Overnight**, a person may do the step themselves instead.

Do not start a prompt until the one before has its commit and every acceptance box ticked. Commits are titled `V6.1 P<nn>: <title>`. Each prompt's **Effort** line is the reasoning effort to set: **High** where a protected proof, the file-reading design, privacy or a third party's format is at stake; **Medium** for records and specified screens.

**Think first (every prompt).** Before the first edit, write a short plan in the scratchpad: the files to touch, the existing functions that must not change (D-387), the tests to run before and after, and the one thing most likely to break. Read it again before the gate. A prompt whose plan finds a contradiction with this file or the plan follows the SAB's precedence and ON §6.

---

# Standard Agent Block (SAB) — prepend to every prompt

**Every terminal (Git Bash, never PowerShell or WSL)**

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut && set -a; . ./.env; set +a
export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:/c/Users/Mayan/AppData/Local/Microsoft/WinGet/Links:$PATH"
export BENCH_DEVICE=d1
BASE=2568a6e
git branch --show-current     # v6.1-build, always (Setup creates it)
```

Server tests need Postgres on port 9000, a Windows service on D1, not Docker. `gh auth status` shows a login. `$SCRATCH` is the session's scratchpad directory.

**Before modifying anything**

1. Read the plan sections and guide steps this prompt cites, every file it touches, and the previous prompt's entry in V6 CL.
2. Search before creating (`git grep -n "<name>"`). `offcut-types` holds the domain types and `limits.rs`; `offcut-mp4` the demuxer and `validate.rs`; `offcut-dsp/src/resample.rs` `resample_mono`; `offcut-wasm-core/src/media_api.rs` the demuxer bindings; `web/src/wasm/load-core.ts` `CoreApi`; `workers/protocol.ts` the worker APIs; `persistence/{db,schema,opfs,clips-repo}.ts`; `state/machines/transition.ts`; `state/blockers.ts`; `usecases/{import-clip,transcribe-cloud,start-app}.ts`; `net/api-client.ts` `transcribeAudio`, `authed()`; `server/src/transcribe/route.rs`, `rate_limit.rs`, `transcribe/budget.rs`; `tests-e2e/helpers/fake-api.ts`.
3. Verify that the previous prompts' outputs behave as this prompt assumes. If not, make the smallest compatible correction and report it.
4. **Precedence.** A decision D-383 to D-432 is binding, also where it refines TS or PS; D-1 to D-382 stand unless one of these replaces them. Otherwise TS wins over the plan and PS over both. If sources disagree in a way these do not settle, or a signature is missing: **by hand, stop and list it; overnight, ON §6.**
5. **TS §41 binds.** Any protected proof that would change beyond DC-1 to DC-4, any new runtime, browser host or paid service, any experiment whose row offers a choice, any change to what the product promises: **by hand, stop and ask with a recommendation; overnight, ON §6's safe path.**

**During implementation**

1. Implement only this prompt's scope. A new file under `crates/`, `server/`, `web/`, `scripts/`, `fixtures/`, `verify/` or `.github/` that TS §5 does not list fails `check-file-tree.mjs`; add its row in the same commit, only for the files §4 names.
2. **Frozen:** G 0.4's check stays empty; G 0.4's additive check shows only the removed lines it names. Goldens and snapshots do not change. No server migration. The functions D-387 names do not change.
3. **Flags default off** (D-393, G item 1). Nothing sets `OFFCUT_FEATURES_ON` outside `build:e2e:flags`.
4. **Reopened contracts and declared changes are exactly:** DC-1 to DC-4 (TS §41.2, v1.4); D-393 (`features.ts`, `vite.config.ts`); D-399 (`offcut-wasm-core` additions); D-407 (`dropSilentWords`' optional argument); D-411 (the route's IP check moves into the handler); D-412 (`budget.rs`' values); D-413 (`transcribeAudio`'s optional window); D-414 (`db.ts`); the additive members of `protocol.ts`. Say **REOPENED CONTRACT** or **DECLARED CHANGE** in the entry for each.
5. No source file over 700 lines above its test module. Rust non-test code: no `unwrap`, `expect`, `panic!`, slice indexing. TypeScript: no `any`, `!`, `@ts-ignore`, empty `catch`, or inline style. No disabled lint, no weakened or deleted test, **no edited existing test case** except the one `rejections.spec.ts` assertion DC-1 declares (P180). Dependencies: none new (§3.1).
6. User-facing text only in `copy/` (new strings in `copy/long.ts`, D-426, as approved in §10.5). `fetch` only in the allowed files. A temporary file is `zz-…` or `zz_…` and is deleted before the gate.
7. **Secrets.** Never print, log, commit or paste a value from `.env`, `.env.deploy`, a token, a cookie or a link. **Never source `.env.deploy` whole**; a script that needs the Groq key reads `TRANSCRIBE_API_KEY` alone (G quick reference). `git grep -nE "gsk_[A-Za-z0-9]{20}"` before every commit: no output. Never query the production database.
8. **Privacy of recordings (F-1, CT D-379).** Real Groq calls only in P171 and P181, only with the reference clip's audio (looped) or a tone, each listed in the entry (file, seconds of audio, status). Never `testclips/v3testclip*`, `testclips/WIN_*`, a corpus clip, or the founder's HDR clip. Never stage `testclips/**`, `testclips_private/**`, `fixtures/.gen/**`, `fixtures/.gen-local/**`, `corpus/**`, `web/test-results/**`, `web/dist*/**` or `.env*`. A `File`'s name is never read or stored (P-11).
9. Long runs start detached and are read from a log tail; no foreground command over 10 minutes. Append to the changelog with the Edit tool, never a heredoc holding backticks.
10. **Branch.** Commit on `v6.1-build` only. Push only where the prompt says. Never push to `main`, force-push, delete a branch or tag, or change Render, Groq or Vercel settings. **Marking ready, merging and tagging happen only in P182, only under ON §3 and §7.**

**When existing code disagrees with the plan.** If it blocks the prompt, fix the minimum and record why, with an "As read" note under the decision in the plan. If not, report it under "Found, not fixed".

**The gate.** G 0.5's tier, the new case alone first; then:

```bash
git branch --show-current                                   # v6.1-build
git status --porcelain | grep -c "zz[-_]"                   # 0
git grep -nE "gsk_[A-Za-z0-9]{20}" | wc -l                  # 0
env | grep -c VITE_ENTITLEMENT_TEST_PUBLIC_KEY              # 0 before the commit
# G 0.4's two commands: no output; G 0.4's additive check: only the named lines
git ls-files --cached | grep -E '(^|/)\.env|testclips|\.gen-local|dist-flags'   # no output
node scripts/check-file-tree.mjs                            # ok
```

Counts never fall below the previous entry's; the floor is D-386's. Add a short entry to V6 CL in the same commit: what changed, what differs from the plan, **REOPENED CONTRACT** and **DECLARED CHANGE** lines, every real Groq call, counts (Rust, Vitest, `pnpm e2e`, media cases run), what is open, "Found, not fixed". Rewrite V6 CL's two top tables when an item opens or closes. Commit with the trailer `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Reply with files changed, commands and results, deviations, steps left for the morning, issues not fixed.

---

# Who does what

| When | Founder | Overnight replacement |
|---|---|---|
| Before Setup | ON §0: power, sleep off, Postgres, the HDR clip (optional), start the session | — |
| P171, P181 | Nothing: F-1 is yes | The agent's real Groq calls, within G items 10-11 |
| P179 | Read the captures (G 9.4) | The agent's self-review with ON §5's rubric; F-5 approved the copy |
| P181 | The local walk; a real sleep test | The agent's scripted walk and TE-20 simulated; the real sleep test is a morning step |
| P182 | Ready, merge, tag | **The agent, under ON §3 and §7** |
| Morning | Read the report (ON §10); TE-20's real sleep; F-3: `high_res` on in production if TE-17's 4K read passed (§12.3); answer any open item | — |

---

# Prompts

## Setup prompt (V6.1 0/12) — The branch, the documents, the floor

**Effort: Medium.**

**Objective.** `v6.1-build` exists with the cloud release's closing entries, spec v1.4 and this release's documents as its first commits; the floor is read; a draft pull request lets CI run.

**Read first.** §1A, §2.1; G Phase 0; ON §1, §3; V6 CL's top tables and Entries 21-24; CT CL Entry 11 (the last counts).

**Do.**

1. G 0.2: fetch; check `origin/main` is `2568a6e` and the working tree holds exactly the documents G 0.2 lists; `git switch -c v6.1-build origin/main`; `git cherry-pick 429d8aa afb82bf`; check the picks touched `docs/cloud_transcription/cloudchangelog.md` only.
2. G 0.3: create `docs/v6.1/experiments.md` and its TS §5 row beside the four existing `docs/v6.1/` rows; `node scripts/check-file-tree.mjs` passes.
3. G 0.6: the three quick counts.
4. V6 CL Entry 25: the branch, base and picks; the founder's answers (Entry 24) by reference; the floor as V6.1's baseline (BP §8A.0 step 1); "built overnight under `overnight-run.md`".
5. Commit `V6.1 setup: spec v1.4 and the V6.1 documents`; push `origin v6.1-build`; `gh pr create --draft` (G 0.3).

**Acceptance.**

- [ ] `git log --oneline origin/main..v6.1-build` shows the two picks and the setup commit only.
- [ ] `git show --stat HEAD` lists docs only.
- [ ] Counts recorded: Rust 863 / 0 / 4, Vitest 568, `pnpm e2e` 57, or the entry says why not.
- [ ] Draft pull request open; its body ends with the attribution line.

---

## Prompt 171 (V6.1 1/12) — Probes: reading in place, HEVC, long AAC, cloud windows

**Effort: High.**

**Objective.** Every unknown the code depends on is read from the real thing before any code is written: how fast and how cheaply a worker reads a large `File` in place, which HEVC strings D1's Chrome decodes, how `AudioDecoder` behaves over 45 minutes, which audio track the demuxer reads, and what 32 real cloud windows cost in time and look like at their seams.

**Read first.** §6.3, §7.2, §8, §13; G Phase 1, G items 10, 11, 18; `server/src/transcribe/groq.rs` (the request's fields); `crates/offcut-mp4/src/demux.rs`.

**Do.**

1. G 1.1: the long local fixtures in `fixtures/.gen-local/` (git-ignored; add the line to `.gitignore` here if missing, which is a docs-level change recorded in the entry).
2. G 1.2: TE-17's first read.
3. G 1.3: HEVC strings and long AAC.
4. G 1.4: TE-18's first read through Groq (2,700 s of audio, the looped reference clip only).
5. G 1.5: the audio track.
6. `experiments.md`: the readings with numbers. If any reading changes a plan line (the read cache size, the HEVC strings accepted, the track, the window concurrency), write an "As read" note under that decision in the plan.

**Stop and ask (overnight: ON §6)** if: `FileReaderSync` cannot read a 4 GB+ file's late blocks; D1's Chrome decodes no HEVC at all (then `high_res` is H.264-4K only: record and go on); Groq's words at a window's edge are cut mid-word in more than 2 of 32 windows (then the overlap rises to 10 s: record and go on); any window answers with no words for speech.

**Acceptance.**

- [ ] TE-17 and TE-18 first reads recorded with numbers; HEVC strings; the audio-track finding.
- [ ] Every real call listed (window count, seconds of audio, statuses, total time); total under 3,000 s.
- [ ] No `zz` file; no key in any file; nothing of `fixtures/.gen-local/` staged.
- [ ] Commit `V6.1 P171: the probes`.

---

## Prompt 172 (V6.1 2/12) — Types, HEVC, colr and `route_probe`

**Effort: High.**

**Objective.** The long path's types exist; HEVC files yield an `hvcC` description and a WebCodecs codec string; HDR is read from `colr`; `route_probe` decides every file exactly as §6.2 says, and with both flags off it **is** `validate_probe`.

**Read first.** §6.1-§6.3, D-395 to D-397, D-387; G Phase 2; TS §43.1-§43.2; `validate.rs` in full; `probe.rs`, `demux.rs`.

**Do.**

1. G 2.1: `source.rs`, the `limits.rs`, `media.rs` and `error.rs` additions (enum members at the end; `is_hdr` with `#[serde(default)]`).
2. G 2.2: `hevc.rs`, `colr.rs`, their wiring as additions.
3. G 2.3: `route.rs`; one `pub use` line in `validate.rs`.
4. G 2.4: the tests; `pnpm gen:types`; the generated diff is additive.
5. The additive check of G 0.4 for every file touched.

**Acceptance.**

- [ ] `cargo test -p offcut-types -p offcut-mp4`: every existing case unchanged and passing; `route_probe.rs` passes, including rule 1's equality on every fixture's probe and `Entry::Long` with the flag off.
- [ ] `validate.rs`: `git diff $BASE` shows one added line.
- [ ] `web/src/gen/` changes are additions; `check-gen-clean.sh` passes.
- [ ] Commit `V6.1 P172: types, HEVC, colr and route_probe`.

---

## Prompt 173 (V6.1 3/12) — The stream resampler, the WASM entries, the flags, the fixtures

**Effort: High.**

**Objective.** A resampler that works in pieces and gives exactly `resample_mono`'s output; WASM entries that read a `File` through a structural reader; flags with defaults, off for V6.x; two keyed builds; the four CI fixtures.

**Read first.** §7.1, D-393, D-394, D-399, D-401, D-430, §11.6; G Phase 3, G items 1, 18, 19; `resample.rs`, `media_api.rs`, `load-core.ts`, `features.ts`, `vite.config.ts`, `gen_fixtures.sh`.

**Do.**

1. G 3.1: `StreamResampler`; `stream_resample.rs`. If exact equality is impossible, the "As read" fallback of G 3.1, recorded.
2. G 3.2: `source_api.rs`; `load-core.ts`; `pnpm build:wasm`.
3. G 3.3: `features.ts`, `vite.config.ts`, the two scripts, the Vitest case; check the production build's define holds both flags `false`.
4. G 3.4: fixtures, `scripts/gen-long-fixture.sh`, `.gitignore`, `manifest.json`; time `pnpm fixtures`.

**Stop and ask (overnight: ON §6)** if: the fixtures add more than 90 s to `pnpm fixtures`; the production build holds any V6.x flag `true`.

**Acceptance.**

- [ ] `stream_resample.rs` passes; `resample_mono`'s tests unchanged and passing.
- [ ] `pnpm build`'s bundle: `long_video` and `high_res` `false`; `cloud_transcribe` `true` (as today).
- [ ] `pnpm build:e2e:flags` writes `web/dist-flags/` with both `true`; `dist-flags` is git-ignored.
- [ ] The four fixtures generate; their `manifest.json` rows; the added time recorded.
- [ ] Commit `V6.1 P173: the stream resampler, the WASM entries, the flags, the fixtures`.

---

## Prompt 174 (V6.1 4/12) — The media worker reads a file in place

**Effort: High.**

**Objective.** The media worker routes a dropped file without copying it, copies and routes a clip under `high_res`, and turns a 45-minute source's sound into `pcm16.f32` in windows, resumably, with the source's reference level.

**Read first.** §6.4, §7.2, D-398, D-400, D-402 to D-404, D-427; G Phase 4, G items 2, 5, 6; `media.worker.ts`, `audio-decode.ts` (as the model to follow, not to change), `import.ts`, `opfs.ts`, `protocol.ts`, `export-loop.test.ts`'s WebCodecs fakes.

**Do.**

1. `file-random-access.ts` and its test.
2. `audio-region.ts` (`decodeAudioRegion`, `writeWholePcm16`, the histogram) and its test, including the resume-equals-whole case.
3. The handlers `routeFile`, `importAndRoute`, `extractSourcePcm16`; `protocol.ts` additions; the existing two handlers unchanged.
4. The scratch smoke of G Phase 4 on `long_12min.mp4`.

**Acceptance.**

- [ ] `file-random-access.test.ts`, `audio-region.test.ts` pass; every existing Vitest case unchanged and passing.
- [ ] G 0.4's frozen check: `import.ts`, `audio-decode.ts` untouched; `media.worker.ts`' diff adds code only.
- [ ] The smoke: `pcm16.f32`'s length exact; correlation above 0.99 on the first second; no buffer of the whole file seen in memory (peak recorded).
- [ ] Commit `V6.1 P174: the media worker reads a file in place`.

---

## Prompt 175 (V6.1 5/12) — The server: windows, budget, events, no-store

**Effort: High.**

**Objective.** `POST /transcribe` takes windows of a long source under their own limits, with today's clip requests unchanged; the global budget is configurable; the three analytics events are allowed; the API's `no-store` reaches the browser.

**Read first.** §8.3, D-411, D-412, D-428, D-429, §11.2; G Phase 5, G items 8, 9; `route.rs`, `router.rs`, `rate_limit.rs`, `budget.rs`, `config.rs`, `analytics.rs`, `web/vercel.json`, `scripts/check-headers.mjs`, `server/tests/{transcribe_route,rate_limit,analytics_allowlist,log_redaction}.rs`.

**Do.**

1. The header's parser, the two groups, the IP check in the handler (the layer removed from `router.rs`), the log field.
2. `budget.rs`, `config.rs`, `render.yaml`.
3. The three events in `offcut-api-types` and the server's allowlist; `pnpm gen:types`.
4. `vercel.json`'s rule; `check-headers.mjs`' assertion.
5. The tests of §11.2.

**Acceptance.**

- [ ] Every existing case of the four server test files passes **without a change to its code** (`git diff $BASE -- server/tests/` shows added cases only).
- [ ] The window cases pass, including the IP bucket still enforced for clips.
- [ ] `node scripts/check-headers.mjs` passes.
- [ ] Commit `V6.1 P175: the server, windows, budget, events, no-store`.

---

## Prompt 176 (V6.1 6/12) — Storage and state

**Effort: High.**

**Objective.** Three new stores in structure version 2, opened only by the long path, with every existing record unchanged; the source machine and store; blockers both ways.

**Read first.** §9.1, §9.2, §9.4, D-414 to D-417, §12.4; G Phase 6, G item 3; `db.ts`, `schema.ts`, `opfs.ts`, `clips-repo.ts`, `transition.ts`, `clip-machine.ts`, `clip-store.ts`, `blockers.ts`, `receipt-outbox.test.ts`.

**Do.**

1. `schema.ts`, `db.ts` (D-414 and the `VersionError` rule), `opfs.ts`.
2. `sources-repo.ts` and its test (upgrade keeps v1 records byte for byte; the clip path leaves version 1).
3. `source-machine.ts`, its test; `source-store.ts`; `blockers.ts`.
4. Tier 2 on `dist/`: the named protected cases of G Phase 6.
5. **Push 1** (not waited on).

**Acceptance.**

- [ ] `sources-repo.test.ts`, `source-machine.test.ts` pass; `receipt-outbox.test.ts` and every other repository test unchanged and passing.
- [ ] Tier 2's protected cases pass on `dist/`.
- [ ] Push 1 done; its CI run's id in the entry.
- [ ] Commit `V6.1 P176: storage and state`.

---

## Prompt 177 (V6.1 7/12) — Transcription in cloud windows

**Effort: High.**

**Objective.** A source's `pcm16.f32` becomes a master transcript through cloud windows: in order, two in flight, checkpointed, with the six failure paths of D-410 and no device fallback.

**Read first.** §8.1, §8.2, §8.4, D-405 to D-410, D-413; G Phase 7; `transcribe-cloud.ts` (the model), `asr.worker.ts`, `cloud-audio.ts`, `word-timestamps.ts`, `api-client.ts`, `cloud-store.ts`.

**Do.**

1. `cloud-audio.ts`: `dropSilentWords`' optional `refLevel` (D-407).
2. `asr.worker.ts`: `cloudWindowWords`, `buildMasterTranscript`; `protocol.ts` additions.
3. `api-client.ts`: `transcribeAudio`'s optional window (D-413).
4. `transcribe-source.ts` and its test (§11.3).

**Acceptance.**

- [ ] `transcribe-source.test.ts` passes; `transcribe-cloud.test.ts`, `cloud-audio.test.ts`, `run-pipeline.test.ts` and `api-client.test.ts`' existing cases unchanged and passing.
- [ ] A clip's `/transcribe` request is byte-identical to today's (an `api-client.test.ts` case).
- [ ] Commit `V6.1 P177: transcription in cloud windows`.

---

## Prompt 178 (V6.1 8/12) — The job: import, run, resume, hand-over

**Effort: High.**

**Objective.** A dropped long video becomes a running source job that survives a reload, asks for the file only when it must, hands a short file to the clip page, and runs the clip page under `high_res` without moving a line of today's flags-off path.

**Read first.** §9.3, §9.5, D-415, D-416, D-418 to D-421, D-427, §4.1; G Phase 8, G item 4; `import-clip.ts`, `start-app.ts`, `cancel-job.ts`, `run-pipeline.ts` (as the model), `eslint.config.js`.

**Do.**

1. `platform/wake-lock.ts`.
2. `import-source.ts`, `run-source-pipeline.ts`, `continue-source.ts` and their tests.
3. `start-app.ts` (D-419); `import-clip.ts` (D-421's hand-over; D-427's branch, today's code unmoved in the `else`).
4. `sentences_long.rs`.
5. The lint edges; Tier 2 on `dist/`.

**Acceptance.**

- [ ] The new Vitest files pass; every existing one unchanged and passing.
- [ ] `git diff -w $BASE -- web/src/usecases/import-clip.ts` shows today's lines unchanged, inside the flags-off branch.
- [ ] Tier 2: `pipeline-preview.spec.ts` and `rejections.spec.ts` pass unchanged on `dist/`.
- [ ] Commit `V6.1 P178: the job, import, run, resume, hand-over`.

---

## Prompt 179 (V6.1 9/12) — The interface

**Effort: Medium** (High for `RejectionPanel` and `DropZone`, which protected cases read).

**Objective.** The long page and its states, the two links into it, and the copy, all present only with `long_video` on; `/app` and the landing page unchanged with it off.

**Read first.** §10, D-422 to D-426; G Phase 9; ON §5; `routes.tsx`, `EditorPage.tsx`, `EditorLayout.tsx`, `TopBar.tsx`, `DropZone.tsx`, `RejectionPanel.tsx`, `ProcessingCard.tsx`, `copy/transcription.ts` (the model), the `screens` project.

**Do.**

1. G 9.1: routes, pages, components, styles, `copy/long.ts` exactly as §10.5.
2. G 9.2: captures to `~/v6.1-review/`.
3. G 9.3: axe.
4. G 9.4: the self-review (overnight) with ON §5's rubric, two rounds at most.
5. **Push 2** (not waited on).

**Acceptance.**

- [ ] On `dist/`: `/app` has no link and `REJECT_DURATION` no new button (a quick Tier 2 case); `landing.spec.ts`, `pipeline-preview.spec.ts`, `rejections.spec.ts` pass unchanged.
- [ ] Every long-page state captured; axe 0 violations; the rubric's answers recorded.
- [ ] `messages.ts` at or under 700 lines; no string in a component.
- [ ] Commit `V6.1 P179: the interface`.

---

## Prompt 180 (V6.1 10/12) — End to end, both builds, CI

**Effort: High.**

**Objective.** The long path proven in the browser on `dist-flags/`, the clip path proven unchanged on both builds, and CI running both.

**Read first.** §11.4, §11.5, §12.1, D-394, D-431; G Phase 10, G items 7, 13-15; `playwright.config.ts`, `fake-api.ts`, `cloud-transcribe.spec.ts` (the model), `rejections.spec.ts`, `ci.yml` and `e2e-media.yml`.

**Do.**

1. `fake-api.ts`'s `windows` option, defaults unchanged; `long-windows.ts`.
2. `playwright.config.ts`: two servers; `media-flags` with `FLAG_SUITES`.
3. The four specs and the declared `rejections.spec.ts` assertion; the SHA-256 question of `flags-identity.spec.ts` read first.
4. `ci.yml` and `e2e-media.yml` (§12.1 as read).
5. Tier 3; the four specs alone, twice each.

**Acceptance.**

- [ ] `FLAG_SUITES` pass twice on D1; `pnpm e2e` passes; the protected specs touched run unchanged on `dist/`.
- [ ] `git diff $BASE -- web/tests-e2e/` shows new files, `fake-api.ts`' added option and the one declared assertion only.
- [ ] Commit `V6.1 P180: end to end, both builds, CI`.

---

## Prompt 181 (V6.1 11/12) — Experiments and the scripted walk

**Effort: High.**

**Objective.** TE-17, TE-18, TE-20 and TE-21 read on D1 through the real product build, and the long page walked end to end as the founder would.

**Read first.** §13, §15, F-1 to F-3; G Phase 11 (its unattended setup first), G items 10, 11, 20, 21.

**Do.**

1. G 11.1: TE-17 and the peak memory, including the clip path's 4K read (F-3's condition).
2. G 11.2: TE-18 through the Groq proxy (2,700 s of audio).
3. G 11.3: TE-20 simulated.
4. G 11.4: TE-21; `hdr_ok` stays `false` in every committed build.
5. The scripted walk on `dist-flags/`: every row of §1.3's table, with a capture each into `~/v6.1-review/walk/`, read with the Read tool.

**Stop and ask (overnight: ON §6)** if: peak memory passes 1.5 GB on the 45-minute file; TE-18 loses or doubles a word at a seam; TE-20 shows a lost checkpoint.

**Acceptance.**

- [ ] `experiments.md` complete with numbers; every real call listed.
- [ ] F-3's condition stated plainly in the entry: "4K clip peak N MB: high_res may / may not be turned on".
- [ ] Commit `V6.1 P181: experiments and the walk`.

---

## Prompt 182 (V6.1 12/12) — Copy-back, the two runs, merge, dark deploy, tag

**Effort: High.**

**Objective.** The specs say what was built; both test paths are green; the pull request is merged and deployed with both flags off; the live site is unchanged for its users; the tag `v6.1` is pushed; the morning report is written.

**Read first.** §12, §17, §18; G Phase 12; ON §3, §7, §8, §10.

**Do.**

1. G 12.1: the copy-back.
2. G 12.2: Tier 4 (the full media run on `dist/` once, `FLAG_SUITES` on `dist-flags/` once; detached, log tails).
3. G 12.3: **push 3**; CI on the head, all green (one re-run for known issues 1-2).
4. ON §7.2's gates, each with its evidence written into V6 CL.
5. ON §7.3: ready, merge, the `main` run, the live checks of §12.2.
6. Tag `v6.1`; push it.
7. The closing entry and the morning report (ON §10); commit and push it on `v6.1-build`.

**Acceptance.**

- [ ] Every box of §17 ticked or named in the report as the founder's morning step.
- [ ] The merge commit, the `main` run's id, the live checks' results, the tag in the report.
- [ ] Commit `V6.1 P182: copy-back, merge, tag` (and the closing entry's commit).
