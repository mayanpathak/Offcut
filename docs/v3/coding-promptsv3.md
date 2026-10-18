# Offcut V3 — coding-promptsv3.md (Prompts 61–90)

Source documents: `docs/v3/v3implementation.md` (written **§n**; its decisions are **D-n**, D-73 to D-112) and `docs/v3/v3buildguide.md` (written **G n.m** for Step n.m, **G M-n** for Milestone n, **G item n** for an item of its "Read this first", **G (b) n** for a row of its table "Spec contradictions and omissions"). `TS §n` = `docs/technicalspec.md`, `PS §n` = `docs/product.md`, `BP §n` = `docs/buildplan.md`, `V2 §n` = `docs/v2/v2implementation.md`, `V2 CL item n` = a row of the "Open items" or "Known issues" table of `docs/v2/v2changelog.md`. Both V3 documents were reviewed against each other and against the V2 changelog on 2026-10-15 and agree; these prompts were written from that revision.

**How to use.** Paste the **Standard Agent Block (SAB)** below, then one prompt, into your coding agent. Run the prompts in order. Do not start Prompt N until every "Acceptance Criteria" box of Prompt N-1 is ticked and its commit exists. Steps marked **Human** are yours (see "Who does what"). V1 was Prompts 01 to 30 and V2 Prompts 31 to 60; V3 is **exactly thirty prompts, 61 to 90**, each also labelled with its place in V3 (01/30 to 30/30). Commits are titled `V3 P<nn>: <title>` (D-75).

**Recorded deviation from TS §34.1 and BP §1.3, extended to V3.** Those sections give each file its own specification. As in V1 and V2, the founder accepts the multi-file prompt format; the per-file contract is honoured by naming every file and implementing every signature the plan gives exactly (§ Appendix A of the plan is the per-file brief).

**The order is the guide's, not §5's.** S6 (the chain in the browser) runs right after S4, before S3 and S5, because TE-13 is the reading most likely to change the design (G item 8). S7 spans five prompts. The two experiments that can change the plan come early: TE-13 natively in Prompt 68 and in the browser in Prompt 69; the precision gate in Prompt 79.

**Three readings are gates.** Prompt 68 (native TE-13 under 2 s per 60 s), Prompt 69 (browser TE-13 at most 4 s per 60 s) and Prompt 79 (precision at least 0.90 over at least 30 labeled files). Each has a pre-agreed action (§20.2, §20.3) and none is a reason to change a label or a test. **D1 is the reference machine (D-69, D-74):** every reading is taken on D1 by the agent and recorded under the label `d1`.

**Effort (for the human, added 2026-10-16).** Each prompt has an **Effort** line under its heading: the reasoning effort to set on the coding agent before pasting it. **High** (20 prompts: 63–73, 75–79, 81, 84, 85, 89) is for numerical, parser, grammar or concurrency code, the labeled ground truth, the three gates, suites likely to expose app bugs, and the spec copy-back. **Medium** (9 prompts: 62, 74, 80, 82, 83, 86, 87, 88, 90) is for exactly specified glue, copy, tooling, CI and runs of measurements. When in doubt the rating is High. Two Medium prompts say when to switch to High partway (74, 88). The line is a note for you; it changes no instruction to the agent.

---

# Standard Agent Block (SAB) — prepend to every prompt

**Every terminal (Git Bash, never PowerShell or WSL; G item 2)**

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut && set -a; . ./.env; set +a      # the repository is offcut/, not sh2clips/
export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:/c/Users/Mayan/AppData/Local/Microsoft/WinGet/Links:$PATH"
export BENCH_DEVICE=d1
BASE=8a271a9                                                                # the V3 baseline every frozen-file check compares with
```

`python` must then be 3.13 with NumPy (`python -c "import numpy"`), and `ffmpeg -hide_banner -version | head -1` must print 9.0.2. A shell opened before winget installed ffmpeg lacks it on PATH; the export above fixes that.

**Before modifying anything**

1. Read the plan sections and guide steps this prompt cites, and every file it touches. Read the previous prompt's changelog entry in `docs/v3/v3changelog.md`.
2. Search the repository for existing types, helpers and constants before creating any (`git grep -n "<name>"`). Reuse them. Never create a second home for a value that has one.
3. Verify that the previous prompts' outputs exist and behave as this prompt assumes (each prompt lists what to verify under "Project State Before This Prompt"). If not: decide whether it is a bug or an intended boundary, make the smallest compatible correction, and report it in the changelog entry.
4. **Precedence.** A decision D-73 to D-112 of §2 is binding, also where it corrects a TS section (it says which); D-1 to D-72 stand unless a V3 decision replaces them. Otherwise TS wins over the plan, and PS over both. The guide gives the order of work and the commands. If two sources disagree in a way none of these settles, or a signature is missing, **stop and list it**. Do not guess (TS §34.1).

**During implementation**

1. Implement only this prompt's scope ("Implement"); leave everything under "Do Not Implement Yet" alone even if it looks easy. Create only the files the prompt names. A file not in the TS §5 tree fails `node scripts/check-file-tree.mjs`; a prompt that adds one says so, and TS §5 changes in the same commit.
2. **Untouched in V3** (§4, last paragraph): every file under `server/`, `crates/offcut-types/`, `crates/offcut-api-types/`, `crates/offcut-scene/`, `crates/offcut-render/`, `crates/offcut-entitlement/`; `offcut-mp4/src/{reader,boxes,demux,sps,validate,mux,mux_boxes}.rs`; `offcut-dsp/src/resample.rs`; `offcut-detect/src/event_id.rs`; `offcut-wasm-core/src/{media_api,hash_api,text_api}.rs`; `offcut-wasm-render/src/{lib,session,profile_api,mux_api}.rs`; `web/vercel.json`; `web/src/gen/**`; `persistence/{schema,db,opfs,entitlement-repo}.ts`; `net/**`; `analytics/client.ts`; `models/**`; `platform/**`; `state/**`; `usecases/{start-app,control-preview,start-export}.ts`; `workers/{rpc,media.worker,asr.worker,render.worker}.ts` and `workers/{media,asr,render}/**`; every UI file §4 does not list; `verify/**`; `render.yaml`; every script §4 does not list. The **reopened contracts** are exactly D-83 (`workers/protocol.ts`, one optional field), D-91 (three `pipeline-preview.spec.ts` cases, `bench/device-bench.ts`) and D-103 (`web/eslint.config.js`, one `MINTING` entry). Say **REOPENED CONTRACT** in the changelog when you touch one.
3. No source file over 400 lines above its test module. Rust non-test code: no `unwrap`, `expect`, `panic!`, slice indexing, `HashMap` (in `offcut-dsp` and `offcut-detect`), clock, randomness, global state or I/O in a pure crate. TypeScript: no `any`, no `!`, no `@ts-ignore`, no empty `catch`, no `as <Brand>` outside a `MINTING` helper. No disabled lint, no `#[allow]`/`eslint-disable` added to pass a check, no weakened, skipped or deleted test (the two V2 expectations V3 rewrites are named: §8.6, §9.9; the three D-91 cases). No dependency the prompt does not name; pin each new one at its latest stable release and record the version in the changelog (§3.2).
4. **Test files V3 may add** (and no others): `crates/offcut-mp4/tests/{demux_fixtures,probe_rejections}.rs`; `crates/offcut-dsp/tests/{loudness_target,chain_no_clicks,duration_preserved,prosody}.rs`; `crates/offcut-text/tests/{numbers_table,numbers_prop,sentences}.rs`; `crates/offcut-detect/tests/{rules_table,precision_recall,redetect,determinism}.rs`; `web/tests-e2e/rejections.spec.ts`; `corpus/run_corpus.ts` (a Playwright file). No new `*.test.ts` (§13.5). Every other Rust test is an inline `#[cfg(test)]` module.
5. Scripts are POSIX `sh` with LF line endings, invoked with `sh`. A temporary file is named `zz-…` or `zz_…` and is deleted before the gate; nothing temporary is committed. A temporary Playwright check needs `web/tests-e2e/zz-<name>.spec.ts` plus a temporary `web/zz-pw.config.ts` that spreads the real config and sets `projects: [{ name: "zz", testMatch: "zz-*.spec.ts", use: { channel: "chrome" } }]`, run with `pnpm --filter web exec playwright test --config zz-pw.config.ts` (V2 CL items 39, 41).
6. A TODO carries the version that removes it (`V4:`, `V7:`). The name of a dropped `File` is never read (P-11). `fetch` exists in the four allowed files only. User-facing text exists in `copy/messages.ts` only, limits interpolated from `LIMITS` (INV-15). Nothing in `offcut-dsp` detects silence, classifies a pause, cuts, crossfades or changes a length (TS §36, INV-5, INV-10).
7. **Privacy (D-99, D-100, D-112).** Never stage, commit, upload or paste anywhere: `testclips/**`, `fixtures/.gen/**`, `fixtures/.cache/**`, `corpus/.clips/**`, `corpus/.results/**`, `web/test-results/**`, `web/playwright-report/**`, `.env*`. `testclips/v3testclip.mp4` carries a GPS location and never leaves D1. A transcript of a clip lent by anyone but the founder is never written to the repository, a result file, a log or a reply.
8. **Secrets.** No seed, token, key or connection string is printed, logged or written to a file. `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` is exported only in the terminal that builds for the media suites (worked out from the seed in `web/tests-e2e/helpers/fake-api.ts`), never in a `.env*` file, and unset before the gate. Production values live in `.env.deploy`; the agent never queries the production database.
9. **Long runs** (`pnpm e2e:media` about 10+ minutes, the bench about 25): start detached, `( cmd > "$LOG" 2>&1; echo "exit=$?" >> "$LOG" ) > /dev/null 2>&1 &`, then poll the log with a bounded loop. Append to the changelog with the Edit tool, or by writing a scratchpad file and `cat >>`; never a heredoc holding backticks.
10. Do not perform a step marked **Human** (a push, a merge, a tag, a dashboard, a founder's decision or judgement). List it in your reply. There is no `gh` CLI: read CI with `curl -s https://api.github.com/repos/mayanpathak/Offcut/commits/<sha>/check-runs`; job logs need a login, so ask the human to paste them.

**When existing code is inconsistent with the plan.** Do not rewrite it blindly. If it blocks this prompt, fix the minimum and record why. If it does not, leave it and report it under "Found, not fixed" in the changelog entry.

**After implementation: the gate.** Every prompt ends with all of it, in this order.

```bash
git status --porcelain | grep -c "zz[-_]"                                   # 0: nothing temporary is left
env | grep -c VITE_ENTITLEMENT_TEST_PUBLIC_KEY                              # 0: the gate build carries no test key
[ -f fixtures/gen_fixtures.sh ] && pnpm fixtures                            # from Prompt 63 on: "fixtures are up to date" or a fresh build
pnpm check && pnpm test && pnpm build && pnpm e2e                           # all four green
git diff --name-only "$BASE" -- \
  server crates/offcut-types crates/offcut-api-types crates/offcut-scene crates/offcut-render crates/offcut-entitlement \
  crates/offcut-mp4/src/reader.rs crates/offcut-mp4/src/boxes.rs crates/offcut-mp4/src/demux.rs \
  crates/offcut-mp4/src/sps.rs crates/offcut-mp4/src/validate.rs crates/offcut-mp4/src/mux.rs crates/offcut-mp4/src/mux_boxes.rs \
  crates/offcut-dsp/src/resample.rs crates/offcut-detect/src/event_id.rs \
  crates/offcut-wasm-core/src/media_api.rs crates/offcut-wasm-core/src/hash_api.rs crates/offcut-wasm-core/src/text_api.rs \
  crates/offcut-wasm-render/src/lib.rs crates/offcut-wasm-render/src/session.rs \
  crates/offcut-wasm-render/src/profile_api.rs crates/offcut-wasm-render/src/mux_api.rs \
  web/vercel.json web/src/gen web/src/net web/src/models web/src/platform web/src/state web/src/analytics/client.ts \
  web/src/persistence/schema.ts web/src/persistence/db.ts web/src/persistence/opfs.ts web/src/persistence/entitlement-repo.ts \
  web/src/usecases/start-app.ts web/src/usecases/control-preview.ts web/src/usecases/start-export.ts \
  web/src/workers/rpc.ts web/src/workers/media.worker.ts web/src/workers/asr.worker.ts web/src/workers/render.worker.ts \
  web/src/workers/media web/src/workers/asr web/src/workers/render verify render.yaml          # (no output): the frozen check, G 1.7
git ls-files --cached | grep -E '^(testclips|fixtures/\.gen|fixtures/\.cache|corpus/\.clips|corpus/\.results)/|(^|/)\.env' # (no output)
```

1. Read the test counts: Rust passed and ignored (`cargo test --workspace 2>&1 | awk '/^test result:/ {p+=$4; i+=$8} END {print p, i}'`), Vitest passed, Playwright passed. None may be lower than in the previous changelog entry. The baseline (Entry 1) is V2's last: 262 Rust with 2 ignored (record whether the 262 counts the two, G 0.6), 92 Vitest, 12 Playwright, 30 media cases.
2. `pnpm e2e:media` is not part of every gate (it needs a keyed build and about 10 minutes). It is run where a prompt says so. **Between Prompts 73 and 85 the three D-91 cases of `pipeline-preview.spec.ts` may fail** because they assert V2's stand-ins; that is expected and is what Prompt 85 rewrites. Any other media failure is a regression of the prompt that caused it.
3. Fix every regression you caused. Tick each acceptance box with its evidence. Leave a box that depends on a **Human** step unticked and say so.
4. Append an entry to `docs/v3/v3changelog.md` (created in Prompt 61): date, prompt, what was added and changed, what differs from the plan or the guide and why, how it was checked (the counts), pinned versions, observations, what is open, "Found, not fixed". Commit on `v3-build` as `V3 P<nn>: <title>`, the entry in the same commit.
5. Reply with: files created and modified, commands run with their results, deviations, **Human** steps still open, issues found but not fixed.

---

# Who does what

The agent writes files, runs commands (including on D1, which is the machine it runs on) and drives Chrome through Playwright. A push, a merge, a tag, a dashboard, a production database read, and a founder's decision, words or judgement are **Human** steps.

| When | Human step | Plan / guide |
| --- | --- | --- |
| Before Prompt 61 | Optional: tag `8a271a9` as `v2` and push the tag. Turn on branch protection for `main` (V2 CL item 8): a push to `main` deploys | §1A carry-over; G 0.3 |
| By Prompt 64 | **Decide §23 item 27**: a word emphasized in a written transcript scores exactly 0.80, the KeywordPop threshold. Either written emphasis gets `pitch_z = 2.0` (score 0.90), or `lib.rs` compares scores with the threshold after rounding to 2 decimals | §23 item 27; G 0.9 |
| By Prompt 65 | **Write down the words spoken in `testclips/v3testclip.mp4`**, sentence by sentence, into `~/v3testclip-words.txt` (outside the repository) | D-99; G 0.9 |
| By Prompt 80 | **Decide §23 item 26**: after "start over", the clip's transcript stays in IndexedDB until V7. Accept that, or add a `clear()` to each of the three repos, called at import step 4 | §23 item 26; G 0.9 |
| Prompt 87 | `git push -u origin v3-build`; open the pull request to `main` (not merged); paste the `ci` job log lines the agent asks for; push the two throwaway guard commits and their reverts | G 11.9, 11.10 |
| Prompt 88 | Optional: lend corpus clips into `OFFCUT_CORPUS_DIR` (D-88). **Read five labeled files** the agent picks at random and answer, per file, "a person would expect these labels: yes / no, because …" | D-87; G 12.4, 12.6 |
| Prompt 90 | Mark the pull request ready and **merge** it after the agent's pre-merge run; watch the `main` run; run the SQL of G 13.5 in the Neon SQL editor; **confirm D-112** for `a01` and `a02`; push the tag `v3` | D-75, D-112; G 13.4–13.6 |
| Before every merge into `main` | `pnpm e2e:media` on D1 on a keyed build, all green (the agent runs it when asked; D-72, D-75) | V2 CL item 16 |

---

# Project Implementation Map

## Architecture Summary

V3 is "Real ingest, clean voice, real detector" (BP §5; TS §28 M1.1–M1.3). It extends the deployed V2 proof of concept, in which a dropped clip is demuxed (`offcut-mp4`), transcribed in an ASR worker, scanned for one event kind (NumberReveal), drawn by the GPU compositor and exported, all in the browser. V3 adds, without restructuring any V2 file:

- **Ingest hardening and rejections.** `offcut-mp4` accepts five edit-list shapes (`edit_list.rs`, D-92) and the probe hardening H1–H8; 25 generated and derived fixtures (`fixtures/gen_fixtures.sh`, D-76, D-77) prove every accept and every one of the 14 `REJECT_*` codes; each refusal shows its own message (`RejectionPanel`, D-80) and sends `clip_rejected` (D-97).
- **The voice chain.** `offcut-dsp` gains high-pass, RNNoise denoise (`nnnoiseless`), compressor, BS.1770 loudness to -14 LUFS (`ebur128`), a delay-free limiter at -1 dBFS, all length-preserving (`run_chain`, TS §34.3, P1–P8), and per-word prosody (`measure_prosody`, D-96, D-108). They run in a fourth worker, `audio.worker`, in parallel with ASR (D-90).
- **The complete number grammar** in `offcut-text` (spelled cardinals to trillions, decimals, currencies, percent, multipliers, time, data, count nouns; the 40-word sentence split).
- **The complete detector** in `offcut-detect`: NumberReveal, ListReveal, FromTo, KeywordPop, exclusions X1–X7, resolution, `redetect_sentence`, measured against 30+ labeled transcripts written before the rules (D-87).
- **Persistence**: three IndexedDB repos (`clips`, `transcripts`, `edits`), `sourceHashPrefix` (D-81).
- **Tooling**: fixture generator, corpus runner (D-88, D-112), bench stage `audio_chain`, CI that generates fixtures on Linux and runs 15 `@no-gpu` rejection cases on Windows (D-93, D-94).

The server, the shared types and the renderer are untouched. Every rule that affects output stays in Rust; TypeScript is glue (TS §2).

## Major Components

| Component | Location | V3 role |
| --- | --- | --- |
| Demux, probe, validate, mux | `crates/offcut-mp4` | `edit_list.rs` new; `sample_table.rs`, `probe.rs` changed; two integration tests |
| Voice chain, prosody, resampler | `crates/offcut-dsp` | Seven new modules, four integration tests |
| Tokens, numbers, sentences | `crates/offcut-text` | `cardinals.rs`, `units_lex.rs` new; full grammar; three integration tests |
| Detection | `crates/offcut-detect` | Five new modules, full `detect`, `redetect_sentence`; four integration tests |
| Bindings | `crates/offcut-wasm-core` (`audio_api.rs` new), `crates/offcut-wasm-render` (`detect_api.rs` + `redetect_sentence`) | Thin `wasm-bindgen` exports |
| Workers and pool | `web/src/workers/` | `audio.worker.ts` new; `pool.ts` fourth row; `protocol.ts` one optional field |
| WASM loaders | `web/src/wasm/load-core.ts`, `load-render.ts` | `runChain`, `measureProsody`, `redetectSentence` |
| Persistence | `web/src/persistence/` | `clips-repo.ts`, `transcript-repo.ts`, `edits-repo.ts` new |
| Use-cases | `web/src/usecases/import-clip.ts`, `run-pipeline.ts` | `hashPrefix`, `clip_rejected`, B2 and D for real, step F records |
| UI and copy | `web/src/ui/…`, `web/src/copy/messages.ts` | `RejectionPanel`, 14 messages, FromTo feed line |
| Fixtures and labels | `fixtures/` | Generator, manifest, `labeled/` set, README sections |
| Corpus, bench | `corpus/`, `bench/` | Runner, fetcher, manifest; `audio_chain` stage |
| CI | `.github/workflows/ci.yml`, `e2e-media.yml` | Pinned Linux ffmpeg; `pnpm fixtures`; rejection cases |
| Records | `docs/v3/`, `docs/technicalspec.md`, `docs/buildplan.md`, `docs/product.md` | Changelog, experiments, copy-back |

## Dependency Graph

```text
P61 preflight, docs, tree, ignores ─► P62 crates, reopened contracts (D-83, D-103), corpus manifest
        │
        ├─► P63 fixture generator (25 fixtures, STAMP) ──────────────┬──────────────────────────────┐
        │                                                           │                              │
        ├─► P64, P65 labeled set (frozen before any rule) ──────────┼───────────────┐              │
        │                                                           ▼               │              │
        ├─► P66 DSP blocks ─► P67 denoise + chain + P1–P8 tests ─► P68 prosody, TE-13 native [GATE]│
        │                                                  │                        │              │
        │                                                  ▼                        │              │
        │                       P69 audio_api, load-core, audio.worker, pool, TE-13 browser [GATE] │
        │                                                                           │              │
        ├─► P70 edit lists + probe H1–H8 ─► P71 demux/probe fixture tests ◄─────────┼──────────────┘
        │                                                                           │
        ├─► P72 lexicons, Digits, cardinals ─► P73 parse/format, split, per-sentence ─► P74 text tests
        │                                                                           │
        │        P75 detect config + number ─► P76 list, from_to, keyword ─► P77 exclusions, resolve, detect, rules_table
        │                                                                           │
        │                     P78 redetect + determinism ─► P79 precision gate [GATE] + render binding
        │                                                                           │
        └──────────► P80 repos + import-clip ─► P81 run-pipeline (needs P69, P79) ─► P82 "as recognized" README
                                                         │
                          P83 rejection copy + panel ─► P84 helpers + rejections.spec ─► P85 preview/export cases (48 on D1)
                                                         │
                          P86 bench + corpus tooling ─► P87 CI (Human push) ─► P88 measure (S12)
                                                         │
                                         P89 copy-back + final local gate ─► P90 merge, deploy, verify, tag (Human)
```

## Implementation Phases

| Guide phase | S-step | Prompts |
| --- | --- | --- |
| 0 Environment and V2 baseline; 1 Contracts, dependencies, docs | S1 | 61, 62 |
| 2 Fixtures and the frozen labeled set | S2 | 63, 64, 65 |
| 3 Voice chain and prosody, native | S4 | 66, 67, 68 |
| 4 Audio worker in the browser (TE-13) | S6 | 69 |
| 5 Demuxer hardening | S3 | 70, 71 |
| 6 Number grammar and sentences | S5 | 72, 73, 74 |
| 7 Detector rules | S7 part 1 | 75, 76, 77 |
| 8 Re-detection, determinism, precision gate | S7 part 2 | 78, 79 |
| 9 Records and the full pipeline | S8 | 80, 81, 82 |
| 10 Rejection panel and copy | S9 | 83 |
| 11 E2E suites, corpus tooling, CI | S10, S11 | 84, 85, 86, 87 |
| 12 Measure | S12 | 88 |
| 13 Copy-back, merge, deploy, tag | S13, S14, S15 | 89, 90 |

## Prompt Dependency Graph and Table

Strictly sequential. Allowed in parallel if two agents run: 70–71 (needs 63 only) beside 66–69; 72–74 (needs 62 only) beside 66–71. Commit order on the branch still follows the numbers.

| Prompt | V3 # | Major deliverable | Depends on | Produces | Guide |
| --- | --- | --- | --- | --- | --- |
| 61 | 01 | Preflight, baseline, branch, `docs/v3/*`, TS §5 tree, ignore rules, V2 M0 record | V2 on `main` at `8a271a9` | `v3-build`; Entry 1; changelog and experiments files | Phase 0, 1.1–1.4 |
| 62 | 02 | Two crates and dev-deps; D-83 and D-103 reopened; `ProcessingFeed`; corpus manifest | 61 | Compiling workspace with new crates; FromTo copy signature | 1.5–1.9, M-1 |
| 63 | 03 | `fixtures/manifest.json`, `gen_fixtures.sh`, `pnpm fixtures`, observations | 62 | 25 fixtures + 5 PCM + STAMP, deterministic | 2.1–2.4 |
| 64 | 04 | `labeled/SCHEMA.md`, `w01`–`w15` | 61, Human (item 27) | Half the written set | 2.5, 2.6 |
| 65 | 05 | `w16`–`w29`, `a01`, `a02`, README; labels frozen | 64, Human (words) | 31 labeled files committed | 2.6–2.8, M-2 |
| 66 | 06 | DSP `config`, high-pass, compressor, loudness, limiter | 62 | Chain building blocks with inline tests | 3.1–3.3 |
| 67 | 07 | `denoise.rs`, `chain.rs`, three P1–P8 test files | 63, 66 | `run_chain`; measured `DENOISE_DELAY_SAMPLES` | 3.4–3.6 |
| 68 | 08 | `prosody.rs`, `tests/prosody.rs`, native TE-13 **[gate]** | 67 | `measure_prosody`; native time | 3.7–3.9, M-3 |
| 69 | 09 | `audio_api.rs`, `CoreApi`, `audio.worker.ts`, pool row, browser TE-13 **[gate]** | 68 | The chain in the browser | Phase 4, M-4 |
| 70 | 10 | `edit_list.rs`, `sample_table.rs` delegation, `probe.rs` H1–H8 | 63 | E0–E4 shapes; hardened probe | 5.1, 5.2 |
| 71 | 11 | `demux_fixtures.rs`, `probe_rejections.rs`; observed manifest values | 70 | Every fixture accepted or refused natively | 5.3–5.5, M-5 |
| 72 | 12 | `units_lex.rs`, wider `Digits`, `cardinals.rs` | 62 | Lexicons and spelled cardinals | 6.1–6.3 |
| 73 | 13 | `parse_quantity`, `format_quantity`, 40-word split, per-sentence numbers | 72 | The full grammar | 6.4, 6.5 |
| 74 | 14 | `numbers_table.rs`, `numbers_prop.rs`, `sentences.rs` | 73 | 72 table rows, 5 properties, 8 cases | 6.6, 6.7, M-6 |
| 75 | 15 | Detector `config.rs`, `number.rs` (parts, labels, re-parse) | 65, 74 | Thresholds, lexicons, NumberReveal V3 | 7.1–7.3 (row 1) |
| 76 | 16 | `list.rs`, `from_to.rs`, `keyword.rs` | 75 | The three new finders | 7.3 (rows 2–4) |
| 77 | 17 | `exclusions.rs`, `resolve.rs`, full `detect`, `rules_table.rs` | 76 | 34 rule rows green | 7.3–7.6, M-7 |
| 78 | 18 | `redetect_sentence`, `redetect.rs`, `determinism.rs` | 77 | Re-detection with stable ids | 8.1, 8.2 |
| 79 | 19 | `precision_recall.rs`, the gate **[gate]**, render binding | 78 | Precision ≥ 0.90 recorded; `RenderApi.redetectSentence` | 8.3–8.5, M-8 |
| 80 | 20 | Three repos, `import-clip.ts` | 69, 79, Human (item 26) | Records and `clip_rejected` at import | 9.1–9.3 |
| 81 | 21 | `run-pipeline.ts` (D-90) | 80 | The full pipeline in dev | 9.4, 9.5 |
| 82 | 22 | README "as recognized" sections; derived clips' events | 81 | Recognized transcripts of 6 clips | 9.6, M-9 |
| 83 | 23 | `messages.rejections`, `RejectionPanel`, `EditorPage`, styles, copy check | 82 | 14 refusals with copy | Phase 10, M-10 |
| 84 | 24 | E2E helpers, `rejections.spec.ts`, `MEDIA_SUITES` | 83 | 16 rejection cases (15 `@no-gpu`) | 11.1, 11.2 |
| 85 | 25 | D-91 rewrites, second-recording cases, full media run | 84 | 48+ media cases on D1 | 11.3, 11.6 |
| 86 | 26 | Bench stage and file name; corpus fetcher, runner, project | 85 | Measurement tooling | 11.4, 11.5 |
| 87 | 27 | `ci.yml`, `e2e-media.yml`, `check-external-facts.mjs`; PR (Human) | 86 | Green pull request | 11.7–11.10, M-11 |
| 88 | 28 | Bench, TE-12, E-3/E-4 re-reads, `a02` data, E-10 smoke, founder reading | 87 | `experiments.md` complete; bench files | Phase 12, M-12 |
| 89 | 29 | Spec copy-back, final local gate, real values | 88 | Specs say what V3 built | 13.1–13.3 |
| 90 | 30 | Merge, deploy, deployed verification, exit checklist, tag (Human) | 89 | V3 live and tagged | 13.4–13.6, M-13 |

---

# Prompts

Every prompt ends with the SAB gate. The "Acceptance Criteria" are what is specific to the prompt; the SAB gate is always also required.

## Prompt 61 (V3 01/30) — Preflight, V2 baseline, branch, V3 documents, spec tree, ignore rules

**Effort: Done.** Committed as `fa67ab4`; nothing to run.

### Objective

Prove D1 has every tool V3 needs, record the V2 baseline counts, cut `v3-build` from `8a271a9`, and commit the V3 documents with the TS §5 tree, the ignore rules and the closing V2 M0 record, so that every later file has a place in the tree and every later prompt has a changelog to write to.

### Project State Before This Prompt

- `origin/main` at `8a271a9` ("Merge pull request #3 from mayanpathak/v2-build"), deployed. V2 is closed (2026-10-15).
- `docs/v3/v3implementation.md`, `docs/v3/v3buildguide.md` and `docs/v3/coding-promptsv3.md` exist in the working tree, not committed.
- `.gitignore` already ignores `testclips/`, `.env*`, `web/test-results/`, `web/playwright-report/`, `fixtures/.cache/`.
- `testclips/speech_scriptA_landscape_720p.mp4` and `testclips/v3testclip.mp4` are on D1; `fixtures/.cache/media/speech_scriptA_landscape_720p.7da948cecc39438a.mp4` may exist.

### Why This Comes Here

`check-file-tree.mjs` fails on any file the TS §5 tree does not list, so the tree must name `docs/v3/*`, `edit_list.rs` and `cardinals.rs` before they exist (D-73, D-92). Every frozen-file check compares with `8a271a9`, and no V3 count may fall below the baseline, so the baseline is read before any change (§1A items 1–4).

### Scope

#### Implement

- Phase 0 of the guide: G 0.1 (shell), G 0.2 (tool audit including `libopus` and the six lavfi filters), G 0.3 (branch), G 0.4 (V2 stand-in markers and V2 §27 copy-back audit), G 0.5 (both founder clips, their hashes, the location checks), G 0.6 (baseline counts and WASM sizes), G 0.7 (scratch dry run of `nnnoiseless` and `ebur128`), G 0.8 (Linux ffmpeg archive: URL, SHA-256, binary folder). Keep the readings in `~/v3-baseline.txt` (outside the repository) until Step 1.1.
- G 1.1: create `docs/v3/v3changelog.md` (Entry 1 with every row of the template, including "Docs review before S1" and "Founder's decisions on §23 items 26 and 27", the latter `pending` if not given) and `docs/v3/experiments.md` (the 14 headings of G 1.1, each with empty `Question:`, `Method:`, `Numbers:`, `Date:`, `Decision:` lines).
- G 1.2: append the M0 gate decision to `docs/v2/experiments.md`.
- G 1.3: TS §5 tree gains `crates/offcut-mp4/src/edit_list.rs`, `crates/offcut-text/src/cardinals.rs`, and under `docs/v3/`: `v3implementation.md`, `v3buildguide.md`, `coding-promptsv3.md`, `v3changelog.md`, `experiments.md`, each with a one-line purpose in the style of the `v2/` lines. Apply every V2 §27 copy-back item G 0.4 found unapplied. If `check-file-tree.mjs` fails on a listed-but-missing file, remove the two source lines now and re-add them in Prompts 70 and 72; record which behaviour the script has.
- G 1.4: `.gitignore` gains `fixtures/.gen/`, `corpus/.clips/`, `corpus/.results/` (D-98); do not touch the `.env*` line (V2 CL item 7).

#### Do Not Implement Yet

- Any `Cargo.toml`, `deny.toml`, TypeScript or ESLint change (Prompt 62). `corpus/manifest.json` (Prompt 62).
- Any fixture or the `fixtures` script (Prompt 63).
- Tagging `v2` (Human, optional).

### Existing Components To Reuse

- `scripts/check-file-tree.mjs` (lists files with `git ls-files --cached --others --exclude-standard`, so ignored paths are skipped, D-98).
- The changelog template of G 1.1; the V2 changelog's entry style.

### Files To Create

- `docs/v3/v3changelog.md`, `docs/v3/experiments.md`.

### Files To Modify

- `docs/technicalspec.md` (§5 tree; V2 §27 leftovers only), `docs/v2/experiments.md` (closing entry), `.gitignore`.
- Commit unchanged: `docs/v3/v3implementation.md`, `docs/v3/v3buildguide.md`, `docs/v3/coding-promptsv3.md`.

### Architecture Requirements

- The branch is `v3-build`, cut from `origin/main` at `8a271a9` (D-75). Never commit V3 work on `main`: a push to `main` deploys.
- Untracked `docs/v3/` files are carried across `git switch -c` untouched; `git status --short` before the switch may show only `?? docs/v3/`.

### Data Types / Interfaces

None (records only). The changelog has the two tables "Open items" and "Known issues" of V2's format, empty at first.

### Implementation Requirements

#### 1. Tool audit (G 0.2)
Run every row; the encoder loop must print `ok` for `libx264 libx265 libvpx-vp9 libmp3lame libopus aac` and for at least one of `libaom-av1`/`libsvtav1`; `ffmpeg -h encoder=libx264` must list `yuv444p`; the filter loop must print `ok` for `testsrc2 sine anullsrc anoisesrc drawbox concat`. A missing encoder: record the replacement (§1A item 6); the fixture's expected outcome does not change.

#### 2. The two founder clips (G 0.5)
`v3testclip.mp4`: 46,360,015 bytes, 78.918 s, a `location` tag present. The reference clip in `testclips/` and in `fixtures/.cache/`: **no** tag whose key matches `location|iso6709` (case-insensitive). If one is found, stop and report to the founder before anything else: that clip is public on the asset host (D-39). Record both SHA-256 values.

#### 3. Baseline (G 0.6)
Build with the test key only for `pnpm e2e:media` (export it in that terminal, unset it after). Record: Rust passed/failed/ignored exactly as printed, Vitest 92, Playwright 12, media 30, `offcut_core_bg-*.wasm` and `offcut_render_bg-*.wasm` sizes from `pnpm build`.

#### 4. Dependency dry run (G 0.7)
On `scratch/v3-deps`: `cargo add -p offcut-dsp nnnoiseless --no-default-features` and the same for `ebur128`; the purity grep of `cargo tree` must print nothing for `web-sys js-sys wasm-bindgen getrandom rand wgpu`; `cargo deny check licenses bans`; `cargo build -p offcut-dsp --target wasm32-unknown-unknown --release`. Record both versions and the tree; revert (`git checkout -- .`), delete the scratch branch. Apply the result table of G 0.7 (a forbidden crate that cannot be turned off: stop; `nnnoiseless` → TE-13 fallback from the start; `ebur128` → replace).

#### 5. Linux ffmpeg (G 0.8)
Choose a static Linux x86_64 GPL build whose URL path names `9.0.2` (or the nearest `9.0.x`), never a "latest" URL. Download to the scratchpad, take its SHA-256, list `ffmpeg` and `ffprobe` inside it. Record URL, hash, folder in Entry 1. The binary cannot run on D1.

#### 6. Docs commit
Entry 1 holds every row of G 1.1's table. `experiments.md` holds the 14 headings. The V2 M0 entry follows G 1.2 verbatim with today's date.

### Error Handling

- Any tool missing: install it the way the "Action if it fails" column of G 0.2 says, or stop and report; never continue to Prompt 62 with a missing encoder unrecorded.
- A baseline count different from V2's last entry: record the real number; it becomes the floor. Do not "fix" V2 code in this prompt.
- `git check-ignore -v testclips/v3testclip.mp4` prints nothing: stop; nothing may be staged with `git add -A` until `.gitignore` is fixed in this prompt.

### Concurrency / Async Requirements

None. Run `pnpm e2e:media` detached (SAB rule 9).

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

- `v3testclip.mp4` is never copied, uploaded or hashed into a committed file except its SHA-256 (a hash reveals nothing).
- The test key is exported only for the baseline media run and unset before the gate.
- No URL with a credential in Entry 1.

### Testing Requirements

#### Unit Tests
None new.

#### Integration Tests
The baseline: `pnpm check`, `cargo test --workspace`, Vitest, `pnpm e2e`, `pnpm e2e:media` (keyed).

#### Failure-Path Tests
G 1.9 row 3 now: `touch fixtures/.gen/x.mp4`; `git status --short` shows nothing for it; remove it.

### Validation Commands

```bash
git rev-parse --short HEAD; git branch --show-current              # 8a271a9 before the commit; v3-build
git merge-base v3-build origin/main | cut -c1-7                    # 8a271a9
node scripts/check-file-tree.mjs                                   # exit 0
mkdir -p fixtures/.gen corpus/.clips corpus/.results && touch fixtures/.gen/probe corpus/.clips/probe corpus/.results/probe
git status --short --untracked-files=all | grep -E 'fixtures/\.gen|corpus/\.|testclips'   # no output
rm fixtures/.gen/probe corpus/.clips/probe corpus/.results/probe
grep -cE "^## " docs/v3/experiments.md                             # 14
```

### Acceptance Criteria

- [ ] Every tool of G 0.2 prints its version; every encoder and filter is present or its replacement is recorded in Entry 1.
- [ ] Entry 1 of `v3changelog.md` holds the baseline SHA, `v2` tag status, the four counts exactly as printed, both clip hashes, both crate versions, the D1 ffmpeg line, the Linux archive URL and SHA-256, both WASM sizes, and the docs-review row.
- [ ] The reference clip has no location tag (or the stop is reported); `v3testclip.mp4` has one and is git-ignored.
- [ ] `docs/v2/experiments.md` ends with the M0 entry; the TS §5 tree names the five `docs/v3/` files and the two new source files (or the deferral is recorded); `check-file-tree.mjs` passes.
- [ ] The scratch branch is deleted and left no change; the SAB gate passes with the baseline counts.

### Agent Instructions

Before: read §1A, §2 D-73/D-75/D-98, §3, G Phase 0 and Steps 1.1–1.4. Confirm `git status` shows only `?? docs/v3/` before switching. During: change only the files listed; this prompt writes records, not code. After: run the SAB gate; commit `V3 P61: preflight, V3 documents, spec tree, ignore rules`. Report each Human item of "Who does what" that is still open (decisions on §23 items 26 and 27, the `v3testclip.mp4` words).

### Expected Result

`v3-build` has one commit above `8a271a9` holding `docs/v3/*` (five files), the TS §5 tree entries, the three ignore lines and the V2 M0 record. The working tree is otherwise clean. `~/v3-baseline.txt` holds nothing that is not also in Entry 1.

---

## Prompt 62 (V3 02/30) — New crates, the reopened contracts D-83 and D-103, the FromTo feed line, the corpus manifest

**Effort: Medium.** Config and one-line contract edits, all specified exactly; the checks (cargo deny, tsc, the three drills) are mechanical.

### Objective

Add the two third-party crates and the dev-dependencies V3 tests need, reopen the two contracts S1 owns (D-83: `FeedLine.displayTo` and the two-argument FromTo copy; D-103: one `MINTING` entry), make `ProcessingFeed.tsx` compile against the new copy signature, write `corpus/manifest.json`, and prove the guards — with every V2 count unchanged (G 1.5–1.9, M-1).

### Project State Before This Prompt

- Prompt 61 committed: `v3-build`, `docs/v3/*`, the TS §5 tree, `.gitignore`, Entry 1 with both crate versions from the dry run.
- Verify: `git log --oneline -1` shows `V3 P61`; `node scripts/check-file-tree.mjs` passes.

### Why This Comes Here

D-83 changes a copy signature that `ProcessingFeed.tsx` calls; done separately, `tsc` would fail from S1 to S9 (G item 4). The crates are pinned once, at S1, so every later prompt builds against the same versions (D-89), and their licences and purity are proven before code uses them. The corpus manifest belongs to S1 (D-99).

### Scope

#### Implement

- G 1.5: first check `grep -rn "unused_crate_dependencies" Cargo.toml crates/*/src/lib.rs clippy.toml`. If it prints nothing: add `nnnoiseless` and `ebur128` to `[workspace.dependencies]` with `default-features = false` at the versions of Entry 1, and `{ workspace = true }` in `crates/offcut-dsp/Cargo.toml`. If it prints a line: add the `offcut-dsp` lines in Prompts 66 (`ebur128`) and 67 (`nnnoiseless`) instead, and record it. Dev-dependencies: `sha2` in `offcut-dsp` and `offcut-mp4`; `proptest` (`features = ["std"]`) in `offcut-text`; `serde` (`derive`), `serde_json`, `proptest` in `offcut-detect` — all `{ workspace = true }` (§18.1).
- `deny.toml`: only the licences `cargo deny check licenses` names for the two crates' trees, each with its crate in a comment; no ban loosened.
- G 1.6 / D-83 (**REOPENED V1 CONTRACT**): `web/src/workers/protocol.ts` — the `event_found` arm of `FeedLine` gains `displayTo?: string`; no other line changes. `web/src/copy/messages.ts` — `feed.eventFound.from_to` becomes `(display: string, displayTo: string) => string`: `displayTo === ""` gives "Found: {display}", otherwise "Found: {display} to {displayTo}". `web/src/ui/components/ProcessingFeed.tsx` — `from_to` lines call `found.from_to(line.display, line.displayTo ?? "")`; the `assertNever` default stays.
- D-103 (**REOPENED V2 CONTRACT**): `web/eslint.config.js` gains one `MINTING` entry `{ brand: "Bytes", helper: "toBytes" }` scoped to `persistence/clips-repo.ts`, in the exact form of V2's entries (V2 CL item 4: the cast must be the last `return` of a function declaration named `toBytes`).
- G 1.8: `corpus/manifest.json` with two entries, `founder-01` and `founder-02`, exactly as §17.6 shows, SHA-256 from Entry 1, `expect: "accept"`, `owner: "founder"`.
- G 1.9: the three guard drills.

#### Do Not Implement Yet

- Any `use` of the new crates in code (Prompts 66, 67). `clips-repo.ts` itself (Prompt 80). `run-pipeline.ts` sending `displayTo` (Prompt 81). `corpus/fetch_corpus.sh`, `run_corpus.ts` (Prompt 86).
- Any change to `ci.yml` (Prompt 87).

### Existing Components To Reuse

- V2's `MINTING` entries in `web/eslint.config.js` (copy their shape).
- `found` in `ProcessingFeed.tsx` (the `messages.feed.eventFound` object), its `switch` over `eventKind`.
- Workspace dependencies `sha2`, `serde`, `serde_json`, `proptest` (already in `[workspace.dependencies]`; verify with `grep`).

### Files To Create

- `corpus/manifest.json`.

### Files To Modify

- `Cargo.toml`, `Cargo.lock`, `deny.toml`, `crates/offcut-dsp/Cargo.toml`, `crates/offcut-mp4/Cargo.toml`, `crates/offcut-text/Cargo.toml`, `crates/offcut-detect/Cargo.toml`.
- `web/src/workers/protocol.ts`, `web/src/copy/messages.ts`, `web/src/ui/components/ProcessingFeed.tsx`, `web/eslint.config.js`.
- `docs/v3/v3changelog.md`.

### Architecture Requirements

- The new crates enter only through `offcut-dsp`; the server does not depend on it (§12). `offcut-dsp` stays pure (`PURE_CRATES`).
- `FeedLine.display` and `displayTo` stay on the device: they never enter an analytics event (INV-2, INV-18).
- `corpus/manifest.json` holds ids, SHA-256, tags, `expect`, `owner` only: never a path or a person's name (D-88, D-112).

### Data Types / Interfaces

```ts
// workers/protocol.ts (D-83)
| { kind: "event_found"; eventKind: EventKind; display: string; displayTo?: string };
// copy/messages.ts
feed.eventFound.from_to: (display: string, displayTo: string) => string;
```

```json
{ "schema": 1, "clips": [
  { "id": "founder-01", "sha256": "<hex>", "tags": ["webcam", "vfr", "landscape"], "expect": "accept", "owner": "founder" },
  { "id": "founder-02", "sha256": "<hex>", "tags": ["webcam", "vfr", "low-fps", "landscape", "location-tag"], "expect": "accept", "owner": "founder" } ] }
```

### Implementation Requirements

#### 1. Crates
Versions exactly as recorded in Entry 1. Run the purity grep of G 1.5 after `cargo build --workspace`, and the `wasm32` build of `offcut-dsp`.

#### 2. Copy
The empty-`displayTo` branch exists for type completeness only (no old store shape can produce it, §16.3). No literal sentence in `ProcessingFeed.tsx` (ESLint `react/jsx-no-literals`).

#### 3. Callers
`git grep -n "eventFound.from_to\|from_to(" -- web/src` must show `messages.ts` and `ProcessingFeed.tsx` only. If `run-pipeline.ts` or a test calls it, update the call to pass `""` and record it.

### Error Handling

- `cargo deny` names a licence outside the allowlist that is not BSD-3-Clause/MIT/Apache-2.0-compatible: stop and report; do not add a copyleft licence.
- `tsc` errors elsewhere after the signature change: fix the caller with the minimal change, never by making `displayTo` optional in the copy function.

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

None (no server change).

### Security Requirements

- `deny.toml`: no ban loosened, no advisory ignored.
- The corpus manifest holds no path, no file name, no owner's name.

### Testing Requirements

#### Unit Tests
None new; every V2 test passes unchanged.

#### Integration Tests
`pnpm check` (both `tsc` programs, ESLint, `check-file-tree`, `check-copy-codes`).

#### Failure-Path Tests
G 1.9: (1) `touch crates/offcut-dsp/src/stray.rs` → `check-file-tree.mjs` names it; (2) `found.from_to(line.display)` with one argument → `tsc` fails; (3) `touch fixtures/.gen/x.mp4` → `git status --short` silent. Undo all three.

### Validation Commands

```bash
cargo build --workspace && cargo deny check
cargo tree -p offcut-dsp -e normal --prefix none | sort -u | grep -E '^(web-sys|js-sys|wasm-bindgen|getrandom|rand|wgpu) '   # no output
cargo build -p offcut-dsp --target wasm32-unknown-unknown --release
node -e 'const m=require("./corpus/manifest.json"); console.log(m.schema, m.clips.map(c=>c.id).join(","))'   # 1 founder-01,founder-02
pnpm check
```

### Acceptance Criteria

- [ ] `cargo deny check` passes; the purity grep is empty; `offcut-dsp` builds for `wasm32-unknown-unknown`; versions equal Entry 1 (or the deferral to Prompts 66/67 is recorded).
- [ ] `protocol.ts` diff is exactly one added optional field; `messages.ts` `from_to` takes two strings; `ProcessingFeed.tsx` passes `line.displayTo ?? ""`.
- [ ] `eslint.config.js` diff is one `MINTING` entry; no rule loosened.
- [ ] `corpus/manifest.json` prints `1 founder-01,founder-02`.
- [ ] The three drills fired and were undone; Rust, Vitest and Playwright counts equal the baseline; the frozen check prints nothing.

### Agent Instructions

Before: read D-83, D-89, D-103, §13.1, §16.2, §16.3, §17.6, §18.1, G 1.5–1.9; read `protocol.ts`, `messages.ts`, `ProcessingFeed.tsx`, `eslint.config.js` in full. During: no other line of `protocol.ts`; no new ESLint rule; no code using the crates. After: SAB gate; commit `V3 P62: crates, reopened contracts D-83 and D-103, corpus manifest`. Changelog: mark both **REOPENED CONTRACT** edits.

### Expected Result

The workspace compiles with `nnnoiseless` and `ebur128` available to `offcut-dsp`; the feed can carry a FromTo's second value; `clips-repo.ts` has its minting slot reserved; Milestone 1 of the guide passes.

---

## Prompt 63 (V3 03/30) — The fixture generator: manifest, `gen_fixtures.sh`, `pnpm fixtures`, observations

**Effort: High.** 25 ffmpeg recipes in POSIX sh that must be byte-identical across runs, an exit-code contract, hash checks, and two on-device observations that can replace fixtures (D-78, D-111). Every later fixture test reads its output.

### Objective

`pnpm fixtures` builds the 25 fixtures of §17.2 (20 synthetic) and §17.3 (5 derived) plus 5 PCM files and a `STAMP` into `fixtures/.gen/`, byte-identically on two runs on D1, with no founder metadata, and records every machine-dependent fact (G 2.1–2.4; D-76, D-77, D-100, D-111).

### Project State Before This Prompt

- Prompt 62 committed. `fixtures/.gen/` is ignored. ffmpeg 9.0.2 on PATH with the encoders of Entry 1.
- Verify: `git check-ignore -q fixtures/.gen/x && echo ok`; `fixtures/speech/README.md` has "The V2 reference clip as recognized" with sentence times (the derived cuts depend on them).

### Why This Comes Here

Six Rust test files (Prompts 67, 68, 71) and the rejection suite (Prompt 84) read these files; the derived speech PCM feeds TE-13 (Prompts 67–69). Nothing that reads a fixture can be written before the generator exists.

### Scope

#### Implement

- `fixtures/manifest.json` (§17.4 format): 25 entries in the order 7 `ok_*`, 13 `rej_*`, 5 `speech/*`. `"pcm": true` on the 5 speech entries only. `kind` `synthetic` or `derived`; `expect.accept` (only the fields a test asserts) or `expect.reject` (the `RejectReason`); `probe` (codec, width, height, frames where fixed); `tests` (the files that read it). `ok_rotated_90.mov` `expect.accept.rotation`, `ok_vfr.mp4` `duration_ms` and `avg_fps_milli` are `null` until observed.
- `fixtures/gen_fixtures.sh` (POSIX `sh`, LF): the nine steps of §17.1 and the skeleton of G 2.2 — version `case` accepting `ffmpeg version 9.0*` and `ffmpeg version n9.0*` (exit 2); reference clip chosen by `$OFFCUT_REFERENCE_CLIP`, `testclips/…`, `fixtures/.cache/<SAMPLE_CLIP_PATH>`, else `curl --fail --retry 3` from `$VITE_ASSET_BASE_URL/<SAMPLE_CLIP_PATH>`, with `SAMPLE_CLIP_PATH` read from `web/src/net/asset-fetch.ts` by `sed`, and its SHA-256 checked against the 16-hex segment of the asset name (exit 3); STAMP up-to-date check; clean; the synthetic recipes of §17.2 in manifest order (exit 4 with the recipe id); the derived recipes of §17.3; PCM extraction (`-map 0:a:0 -ac 1 -ar 48000 -f f32le <file>.pcm48.f32`); the ffprobe check of each manifest `probe` field, parsing the manifest with `node -e` (exit 5); `STAMP` written last. `GLOBAL` flags before the first `-i`, `OUTF` flags after the last `-i` (§17.1).
- Root `package.json` script `"fixtures": "sh fixtures/gen_fixtures.sh"` (the `corpus` script is Prompt 86).
- G 2.3: run twice from clean, diff the hashes, check up-to-date, the D-100 leak loop, the derived clips' width/height/frames/duration table.
- G 2.4 observations: `ok_vfr.mp4` duration and average rate into the manifest; `rej_h264_444.mp4` profile `High 4:4:4 Predictive` and Chrome's `VideoDecoder.isConfigSupported` through a browser check (dev) — `supported: false` keeps it, else the D-78 fallback chain; `rej_silent.mp4` dropped in the V2 app on D1 (model cached) — the V2 rejected stub keeps it, an acceptance shortens it to 2 s (D-111). Rotation and the disabled track stay `null` (Prompt 71).
- Changelog Entry 2: every observation, the encoders used per recipe, the generator's time.

#### Do Not Implement Yet

- Any Rust test that reads a fixture (Prompts 67, 71). The E2E helpers `genFixture`/`fixturesDir` (Prompt 84). CI steps (Prompt 87). `bad_*` fixtures (V7).

### Existing Components To Reuse

- `web/tests-e2e/helpers/fixtures.ts`'s way of reading `SAMPLE_CLIP_PATH` (the regex on `asset-fetch.ts`) — mirror it in `sed`, do not import it.
- `fixtures/.cache/` layout (each file under its asset path, V2 CL item 39).

### Files To Create

- `fixtures/manifest.json`, `fixtures/gen_fixtures.sh`.

### Files To Modify

- `package.json` (one script), `docs/v3/v3changelog.md`. If Git would check the script out with CRLF (`git ls-files --eol fixtures/gen_fixtures.sh` shows `w/crlf`), add one line to an **existing** `.gitattributes`; if none exists, stop and report (a new root file needs a TS §5 entry).

### Architecture Requirements

- The generator writes nothing outside `fixtures/.gen/` (and `fixtures/.cache/` on a download). FFmpeg stays a development tool (TS §36): no app code calls it.
- Every recipe carries `-bf 0 -threads 1 -g 30` (H.264) and the bitexact flags, `-map_metadata -1 -map_chapters -1` (D-100, D-111); `-movflags +faststart` on every MP4/MOV except `ok_moov_at_end.mp4`.
- Derived clips are cut from `REF` with the exact times of §17.3, `fps=30` first so frame numbers count output frames; room tone is `anoisesrc=color=pink:amplitude=0.001:sample_rate=48000:seed=7` joined with `concat` in one `filter_complex`, no crossfade.
- `rej_truncated.mp4`: faststart to a temporary name, then `head -c 1500`. `rej_hevc.mov`: add `pools=1:frame-threads=1` to `-x265-params` if two runs differ (record). `rej_av1.mp4`: libaom `-cpu-used 8 -row-mt 0` or libsvtav1 `-preset 12` (record which).

### Data Types / Interfaces

```json
{ "ffmpeg": "9.0.2", "script_sha256": "<hex>", "manifest_sha256": "<hex>", "reference_sha256": "<hex>", "generated": 25 }
```

`STAMP` is JSON (G M-2 reads `.generated`). Rust and TypeScript helpers later compare `manifest_sha256` with the SHA-256 of `fixtures/manifest.json`.

### Implementation Requirements

#### 1. Recipe table
Exactly the 20 rows of §17.2 and the 5 of §17.3, with their sizes, rates, durations and expected outcomes. `REJECT_FILE_SIZE` has no file (D-79).

#### 2. Exit codes
2 tool version, 3 reference hash, 4 recipe failure (prints the id), 5 probe mismatch (prints the name). Never a partial `STAMP`.

#### 3. Determinism
Two clean runs give identical SHA-256 for every file except `STAMP`. Cross-machine byte equality is not required (§17.1).

### Error Handling

- Any recipe fails: the script stops; no `STAMP`; the next run rebuilds from clean.
- The reference clip is missing everywhere and `VITE_ASSET_BASE_URL` is unset: exit 3 with a message naming both; never generate from another clip.

### Concurrency / Async Requirements

`-threads 1` and x265 pools keep encoders single-threaded so output is deterministic.

### Persistence Requirements

`fixtures/.gen/` is a cache with a stamp: any change to the script, the manifest, the reference clip or the ffmpeg version forces a full rebuild.

### API Requirements

None.

### Security Requirements

- No metadata of a founder recording reaches a generated file (D-100): the leak loop of G 2.3 prints no `LEAK` line.
- `v3testclip.mp4` is never an input of the generator (D-99).
- The `curl` uses `--fail` and the hash check; a mismatched download is rejected (exit 3).

### Testing Requirements

#### Unit Tests
None (the script is checked by its own step 8 and by the determinism run).

#### Integration Tests
G 2.3 as a whole; the browser check for `rej_h264_444.mp4`; the drop of `rej_silent.mp4` on D1.

#### Failure-Path Tests
Edit one character of `fixtures/manifest.json` → `pnpm fixtures` rebuilds (stamp stale); temporarily set an impossible `probe.width` → exit 5 naming the fixture; undo both.

### Validation Commands

```bash
time pnpm fixtures
find fixtures/.gen -type f | wc -l                                  # 31
( cd fixtures/.gen && find . -type f ! -name STAMP | sort | xargs sha256sum ) > "$TMP/gen1.txt"
rm -rf fixtures/.gen && pnpm fixtures
( cd fixtures/.gen && find . -type f ! -name STAMP | sort | xargs sha256sum ) | diff "$TMP/gen1.txt" - && echo deterministic
pnpm fixtures                                                       # fixtures are up to date
node -e 'const m=require("./fixtures/manifest.json").fixtures; console.log(m.length, m.filter(x=>x.expect.accept).length, m.filter(x=>x.expect.reject).length, m.filter(x=>x.pcm).length)'   # 25 12 13 5
node -e 'console.log(JSON.parse(require("fs").readFileSync("fixtures/.gen/STAMP","utf8")).generated)'   # 25
git ls-files --eol fixtures/gen_fixtures.sh                         # i/lf ... after staging
```

### Acceptance Criteria

- [ ] 25 fixtures, 5 PCM files and `STAMP` (31 files) generated twice byte-identically; a third run prints "fixtures are up to date".
- [ ] The five derived clips match the width/height/frames/duration table of G 2.3; no file carries `location`, `ISO6709` or `creation_time`.
- [ ] `ok_vfr.mp4`'s duration and rate are in the manifest; the decodability of `rej_h264_444.mp4` and the word count of `rej_silent.mp4` are observed on D1 and recorded with the fixture kept or replaced per D-78 / D-111.
- [ ] The script is LF in the index; `pnpm fixtures` runs from Git Bash; Entry 2 lists the encoders used and the generator's time.

### Agent Instructions

Before: read §17.1–§17.4, D-76–D-79, D-100, D-111, G item 2, G 2.1–2.4, and the README's sentence table. During: unquoted flag variables on purpose (no value holds a space); `\,` inside double-quoted `-vf`. Do not invent a recipe; a recipe the build cannot make is recorded with its replacement. After: SAB gate (from now on it runs `pnpm fixtures`); commit `V3 P63: fixture generator, manifest and observations`.

### Expected Result

Any machine with ffmpeg 9.0.x and the reference clip can run `pnpm fixtures` and obtain the 25 fixtures with a matching `STAMP`; the manifest is the single list of what each fixture must produce.

---

## Prompt 64 (V3 04/30) — The labeled set, part 1: `SCHEMA.md` and the written files `w01`–`w15`

**Effort: High.** Ground truth for the precision gate: natural sentences, traps and exact `say` matches. A sloppy label makes the 0.90 gate meaningless, and labels freeze at P65 (D-87).

### Objective

Write the labeled-file schema and the first fifteen written transcripts of the §17.5 file plan, each with the events **a person** would expect, before any detector rule exists (D-87).

### Project State Before This Prompt

- Prompt 63 committed. `crates/offcut-detect/src/` holds only V2's `lib.rs config.rs number.rs event_id.rs` (verify with `git ls-files`).
- **Human:** the founder's decision on §23 item 27 is in Entry 1 (or given in this prompt's message). If it is missing, stop before writing any file.

### Why This Comes Here

The labels must be fixed before `from_to.rs`, `list.rs`, `keyword.rs`, `exclusions.rs` or `resolve.rs` exist, so that nobody writes a label to suit a rule (D-87, BP §5.2). The split over two prompts keeps each file's writing careful; both commits precede Prompt 75.

### Scope

#### Implement

- `fixtures/labeled/SCHEMA.md`: the format of §17.5 and every row of its field table (`sentences`, word timings, `words`/`prosody`, `say`, `kind`, values, true positive, `tags`, `edits`, null data, `say` matching), plus:
  - the **emphasis values** by the founder's decision on item 27 (`energy_z 2.0, pitch_z 1.5`, or `pitch_z 2.0`; or rounding in `lib.rs` — then the values stay 2.0/1.5 and SCHEMA says so);
  - the **unit vocabulary** of `expected[].value/unit`: `none`, `usd`, `eur`, `gbp`, `inr`, `percent`, `times`, `milliseconds`, `seconds`, `minutes`, `hours`, `days`, `weeks`, `months`, `years`, `bytes`, `kilobytes`, `megabytes`, `gigabytes`, and `count:<noun>` (e.g. `count:subscribers`). The plan is silent on these strings; this prompt fixes them and `precision_recall.rs` (Prompt 79) maps them to `Unit`;
  - the **`tags` vocabulary** of §17.5 (`X1`–`X5` once per trap, `announcer`, `marker_only`, `from_frame`, `change_verb`, `no_change_pair`, `edit_number`, `hidden_word`, `partial_list`, `years_from_to`).
- `fixtures/labeled/w01-saas-pricing.json` … `w15-podcast.json`: the rows `w01`–`w15` of the §17.5 file plan, each 4 to 10 sentences of a technical creator talking to camera, `schema: 1`, `source: "written"`, `author: "agent"`, `frozen` = today, `words`/`prosody` `null`, `expected`, `tags`, `notes`, `edits` only in `w11`.

#### Do Not Implement Yet

- `w16`–`w29`, `a01`, `a02`, the README changes (Prompt 65). Any Rust that reads these files (Prompt 79).

### Existing Components To Reuse

- The JSON example of §17.5 (field order and names).
- Script A's wording in `fixtures/speech/README.md` as a model of the voice (do not copy it into a `w` file).

### Files To Create

- `fixtures/labeled/SCHEMA.md`, `fixtures/labeled/w01-saas-pricing.json` … `fixtures/labeled/w15-podcast.json` (ids exactly as §17.5: `w01-saas-pricing`, `w02-coding-bootcamp`, `w03-home-office`, `w04-newsletter`, `w05-cloud-costs`, `w06-fitness`, `w07-camera`, `w08-freelance-rates`, `w09-learning-rust`, `w10-app-launch`, `w11-first-year`, `w12-productivity`, `w13-data-migration`, `w14-sleep-focus`, `w15-podcast`).

### Files To Modify

- `docs/v3/v3changelog.md` (the decision on item 27 as applied).

### Architecture Requirements

- `fixtures/labeled/` is in the TS §5 tree as `labeled/*.json` and `labeled/SCHEMA.md` (verify; if the tree lists the directory differently, follow it and record).
- The files are data read by Rust tests only; they never ship in a bundle.

### Data Types / Interfaces

```json
{ "schema": 1, "id": "w07-camera", "source": "written", "author": "agent", "frozen": "<yyyy-mm-dd>",
  "sentences": ["…"], "words": null, "prosody": null,
  "expected": [ { "kind": "number_reveal", "say": "$2,400", "occurrence": 1, "value": 2400, "unit": "usd" } ],
  "tags": ["X1"], "notes": "…" }
```

No comments inside the JSON files (the comments in §17.5 explain fields; `serde_json` refuses comments).

### Implementation Requirements

#### 1. Write the sentences first
Per file: the topic of its row, natural speech, punctuation as a recognizer might give it (commas, full stops). Numbers in the forms the grammar reads (§8.3, §19.8): digits with separators, `$` forms, "percent", spelled cardinals, units. Emphasis: `*word*` only where the row asks for a KeywordPop.

#### 2. Then the expected list
What a person hearing it would expect to see, never what a rule would score. `say` is a run of the sentence's words that matches after the §17.5 normalisation; `occurrence` picks the n-th match. NumberReveal and FromTo carry values and units (after inheritance for FromTo).

#### 3. Coverage of this half
Follow each row's "Expected events", "Traps", "Edits" exactly (e.g. `w05` holds two FromTos and an X2 "version 2" trap; `w06` a marker-only list and an X5 "3 sets"; `w07` a "4K camera" trap; `w11` one `edits` entry that changes a number, its expected value being the edited one; `w12`, `w14` no events). Tag every trap and form.

#### 4. Content rules
No real person quoted; no real company's figures; no profanity; nothing from `v3testclip.mp4`.

### Error Handling

- A row of the plan that cannot be written naturally: write the closest natural form and say so in `notes`; never drop a required trap or event.

### Concurrency / Async Requirements

None.

### Persistence Requirements

Labels are frozen from this commit on: any later change needs a `v3changelog.md` line naming the file, the label and the reason, and "the detector disagrees" is not a reason (D-87).

### API Requirements

None.

### Security Requirements

No personal data; nothing derived from a founder or lent recording.

### Testing Requirements

#### Unit Tests
None (Prompt 79 reads the files).

#### Integration Tests
The node checker of G 2.6 (ids match file names, kinds valid, every `say` found in the normalised sentences, kinds and tags tallied).

#### Failure-Path Tests
Temporarily misspell one `say` → the checker reports `say not found`; undo.

### Validation Commands

```bash
for f in fixtures/labeled/*.json; do node -e 'JSON.parse(require("fs").readFileSync(process.argv[1],"utf8"))' "$f" || echo "BAD $f"; done
# then the G 2.6 checker; expect: 15 files, 2 without events (w12, w14), 0 problems
git ls-files crates/offcut-detect/src | sort                        # lib.rs config.rs number.rs event_id.rs only
```

### Acceptance Criteria

- [ ] `SCHEMA.md` holds every §17.5 field rule, the emphasis values of the founder's decision, the unit vocabulary and the tags vocabulary.
- [ ] Fifteen files parse; the G 2.6 checker prints 0 problems; each file's events, traps, tags and edits match its §17.5 row.
- [ ] `offcut-detect/src` still holds only V2's four files.

### Agent Instructions

Before: read D-87, §17.5 in full, §8.3–§8.4 (what the grammar reads), §9.7 (the traps), TS §17.6. During: one file at a time, sentences then labels; never open or think about detector code. After: SAB gate; commit `V3 P64: labeled set part 1 (schema, w01-w15)`.

### Expected Result

Half of the independent ground truth exists in the repository, dated and committed before any rule.

---

## Prompt 65 (V3 05/30) — The labeled set, part 2: `w16`–`w29`, `a01`, `a02`, the README, the freeze

**Effort: High.** Must hit the exact coverage totals over 31 files, copy `a01` word for word (the `$12` `,000` split) and freeze the set. Mistakes here are permanent.

### Objective

Complete the labeled set (31 files: 29 written, `a01` written-form from Script A as recognized, `a02` labels-only for the second recording), update `fixtures/speech/README.md` for V3, and freeze the labels (G 2.6–2.8, M-2; D-87, D-99).

### Project State Before This Prompt

- Prompt 64 committed (`SCHEMA.md`, `w01`–`w15`).
- **Human:** `~/v3testclip-words.txt` holds the founder's words for `v3testclip.mp4`. If it is missing, write everything else and leave `a02` and the README's founder-words section for a follow-up commit within this prompt once given; do not invent the words.

### Why This Comes Here

The coverage minimums of §17.5 are met only with all 29 written files and `a01`; the freeze must be in place before Prompt 75 writes detector code.

### Scope

#### Implement

- `w16-open-source` … `w29-morning-routine` by the §17.5 rows, as in Prompt 64.
- `a01-scriptA.json`: `source: "written"`, sentences copied word for word from "The V2 reference clip as recognized" in the README, including its recognizer forms ("3000 hours"; the amount as the two words `$12` `,000`, words 132–133, which the written-form test turns into two words exactly as the recognizer gave them, D-42), no `*` marks; expected NumberReveal `3,000 hours` and `$12k` (value 12000 usd), FromTo `90 minutes` → `10 minutes` (from-frame), ListReveal (announcer, count 3); **no KeywordPop** (written form carries no prosody); traps `8`, `90`, `10`, `three` alone; `notes` say it is the recognized text written out.
- `a02-v3testclip.json`: `source: "asr"`, `author: "founder"`, `sentences`, `words`, `prosody` all `null`; `expected` from the founder's words (what a person expects from what was said); `notes`: "data filled at S12 (Prompt 88) by CORPUS_MODE=transcripts; labels frozen here".
- `fixtures/speech/README.md` (§17.7): the five TS clips marked "derived in V3 from the reference clip (D-76)" with their cuts; a `v3testclip.mp4` row "V3 test clip, founder's recording of 2026-10-13, not in git (D-99)"; Script A's V3 expected events and expected feed; new sections "The derived clips as recognized" (to be filled at Prompt 82) and "The V3 test clip as recognized" holding the founder's words now and "recognized transcript: filled at S8 (Prompt 82)".
- G 2.8: confirm `offcut-detect/src` is V2's; after the commit, write its SHA into the changelog as "labels frozen at <SHA> on <date>" (in the next prompt's entry, since a commit cannot name itself).

#### Do Not Implement Yet

- Any reader of the files (Prompt 79). The "as recognized" content (Prompt 82). `a02`'s data (Prompt 88).

### Existing Components To Reuse

`SCHEMA.md`, the G 2.6 checker, the README's existing sections (append, never rewrite V2's recognized transcript).

### Files To Create

- `fixtures/labeled/w16-open-source.json` … `w29-morning-routine.json` (ids per §17.5), `fixtures/labeled/a01-scriptA.json`, `fixtures/labeled/a02-v3testclip.json`.

### Files To Modify

- `fixtures/speech/README.md`, `docs/v3/v3changelog.md`.

### Architecture Requirements

- `a01` and `a02` are the only founder-derived labeled files, and the only ones that may be (D-112).
- The README remains the authority for what each clip holds (§17.3, §17.7).

### Data Types / Interfaces

As Prompt 64. `a02` has `"sentences": null, "words": null, "prosody": null` and its `say` values are checked later against recognized words (Prompt 88).

### Implementation Requirements

#### 1. Rows w16–w29
Follow each row: `w19`, `w21` incompatible-unit pairs (`no_change_pair`); `w23` one `""` edit inside the second list item and no events (`hidden_word`, `partial_list`); `w24` two X2 traps; `w25` X1 and X3; `w26` two phone-like X4 runs; `w27` two X5 traps; `w28` announcer list plus change-verb FromTo; `w29` marker-only list plus KeywordPop.

#### 2. Coverage after the set
Run the G 2.6 checker: after `w29` — 29 files, 8 without events, 0 problems, kinds `number_reveal 10, list_reveal 7, from_to 11, keyword_pop 9`, tags `X1`–`X5` each ≥ 2, `announcer 4`, `marker_only 3`, `from_frame 5`, `change_verb 6`, `no_change_pair 2`, `edit_number 1`, `hidden_word 1`. After `a01`, `a02`: 31 files, kinds at least `number_reveal 12, list_reveal 8, from_to 12, keyword_pop 9`, `announcer 5`.

#### 3. README
Additive edits only; keep every V2 line.

### Error Handling

A minimum not met: add the missing case to the file whose row allows it (never a new file); record it.

### Concurrency / Async Requirements

None.

### Persistence Requirements

The freeze (D-87): from this commit, every label change has a changelog line. `a02`'s later data fill changes `words`/`prosody`/`sentences` only.

### API Requirements

None.

### Security Requirements

- The founder's words are committed only because they are the founder's own (D-112, confirmed at Prompt 90). Nothing of `v3testclip.mp4`'s location or metadata is written anywhere.

### Testing Requirements

#### Unit Tests
None.

#### Integration Tests
The G 2.6 checker over 31 files; G M-2's commands.

#### Failure-Path Tests
None new.

### Validation Commands

```bash
# G 2.6 checker → 31 files, 8 without events, 0 problems (a02's say values are not checked: no sentences)
rm -rf fixtures/.gen && pnpm fixtures && pnpm fixtures               # build; then "fixtures are up to date"
git ls-files crates/offcut-detect/src | grep -E 'from_to|list|keyword|exclusions|resolve' ; echo "none yet"
git status --short | grep -E 'testclips|\.gen|\.clips|\.results'    # no output
```

### Acceptance Criteria

- [ ] 31 labeled files committed; the checker's totals equal Requirement 2; every §17.5 minimum is met with `a01`.
- [ ] `a01` is written-form with no KeywordPop expected; `a02` has null data and labels from the founder's words.
- [ ] The README has the derived-clip rows, the `v3testclip.mp4` row, V3's expected events and feed, and the two new sections.
- [ ] No detector file beyond V2's four exists; G M-2 passes.

### Agent Instructions

Before: read §17.5, §17.7, D-87, D-99, D-112, G item 3, G 2.6–2.8. During: never look at what a rule would give. After: SAB gate; commit `V3 P65: labeled set part 2, a01, a02, README; labels frozen`.

### Expected Result

The ground truth of the precision gate is complete and frozen; the README states what V3 expects of every clip.

---

## Prompt 66 (V3 06/30) — `offcut-dsp` building blocks: constants, high-pass, compressor, loudness, limiter

**Effort: High.** Numerical DSP with tight tolerances (±0.05 dB, ±0.1 LU): RBJ biquad, log-domain soft-knee compressor, BS.1770 loudness, and a delay-free look-ahead limiter that must clamp zero samples.

### Objective

Write the DSP constants and the four stateless or block-stateful stages of the voice chain that do not need the denoiser, each with its inline tests (G 3.1–3.3; §7.1–§7.3, §7.5–§7.7).

### Project State Before This Prompt

- Prompt 65 committed. `offcut-dsp` holds V2's `lib.rs` (with `DspError`) and `resample.rs`; `ebur128` is available (Prompt 62, or added now if deferred).
- Verify: `cargo tree -p offcut-dsp -e normal | grep ebur128`.

### Why This Comes Here

S4 goes first among the pure crates because TE-13 can change the chain's design (§5). The stages are written and tested alone before `chain.rs` composes them (Prompt 67).

### Scope

#### Implement

- `src/config.rs` [ONLY DSP constants]: the 32 constants of §7.2 verbatim (23 chain, 9 prosody) and `ChainConfig` with `Default` (TS §34.3 fields; `denoise: true`), built without `unwrap` by the const-helper pattern of §7.2.
- `src/highpass.rs`: `HighPass::new(cutoff, rate)`, `process(&mut self, block)`; RBJ high-pass, Q = `HPF_Q`, Direct Form II transposed, f64 state.
- `src/compressor.rs`: `Compressor::new(rate)`, `process`; soft knee in the log domain, peak follower, no look-ahead, no make-up gain, gain ≤ 1 (§7.5).
- `src/loudness.rs`: `integrated`, `sample_peak`, `gain_db`, `apply_gain` (§7.6); `ebur128` mono, channel `Center`, mode I, fed in `BLOCK_SAMPLES` chunks.
- `src/limiter.rs`: `limit(samples, ceiling, rate) -> usize` (§7.7): delay-free look-ahead (moving minimum of required gain over `[n, n + LIMITER_LOOKAHEAD]`, smoothed by a moving average of the same length), release `exp(-1/(0.050 x rate))` never above the attack gain, a final clamp to `±c` that counts touched samples and returns the count.
- `src/lib.rs`: one `pub mod` per file as it lands; `pub use config::ChainConfig`. `chain`, `denoise`, `prosody` modules come later.

#### Do Not Implement Yet

- `denoise.rs`, `chain.rs`, the integration test files (Prompt 67); `prosody.rs` (Prompt 68); the binding (Prompt 69).

### Existing Components To Reuse

- `offcut-types` units `Hz`, `Lufs`, `Dbfs`, `SampleCount` (their constructors return `Option`, §7.2).
- V2's `DspError` (unchanged enum).

### Files To Create

- `crates/offcut-dsp/src/{config,highpass,compressor,loudness,limiter}.rs`.

### Files To Modify

- `crates/offcut-dsp/src/lib.rs`; `crates/offcut-dsp/Cargo.toml` only if `ebur128` was deferred.

### Architecture Requirements

- Pure: no clock, randomness, global state, I/O, logging, `HashMap`. f32 samples, f64 filter state and accumulators; fixed summation order (P8).
- No constant is a silence threshold for cutting, a pause length or a crossfade (TS §18.3). `ABSOLUTE_SILENCE_LUFS` only stops gain on an unmeasurable programme.
- Every constant lives in `config.rs` only.

### Data Types / Interfaces

```rust
pub struct HighPass { b0: f64, b1: f64, b2: f64, a1: f64, a2: f64, z1: f64, z2: f64 }
pub struct Compressor { env_db: f32, attack: f32, release: f32, threshold: f32, ratio: f32, knee: f32 }
pub fn integrated(samples: &[f32]) -> Option<Lufs>;
pub fn sample_peak(samples: &[f32]) -> Dbfs;            // -inf reported as -200.0
pub fn gain_db(measured: Option<Lufs>, cfg: &ChainConfig) -> f32;
pub fn apply_gain(samples: &mut [f32], db: f32);
pub fn limit(samples: &mut [f32], ceiling: Dbfs, rate: Hz) -> usize;
```

### Implementation Requirements

#### 1. High-pass
Cutoff 0 or ≥ rate/2: pass-through (b0 = 1, others 0). Use `iter_mut` (clippy denies indexing).

#### 2. Compressor
Formulas of §7.5 exactly; attack `exp(-1/(0.005 x 48,000))`, release `exp(-1/(0.080 x 48,000))`.

#### 3. Loudness
`gain_db`: `None` or below `absolute_silence` → 0; else `target - measured` capped at `+max_gain_db`, no cap on negative gain.

#### 4. Limiter
In place, length unchanged, no sample shifted; a signal under the ceiling is unchanged bit for bit.

### Error Handling

No function here returns an error: inputs are finite by contract (the chain checks `NonFinite` in Prompt 67). `integrated` returns `None` when no block passes the gate.

### Concurrency / Async Requirements

None (single-threaded, deterministic).

### Persistence Requirements

None.

### API Requirements

None (crate-internal; `lib.rs` re-exports as listed).

### Security Requirements

None beyond purity.

### Testing Requirements

#### Unit Tests
- `config.rs`: 1 case — `ChainConfig::default()` equals TS §18.3's values.
- `highpass.rs`: 6 cases of G 3.2 (block length; cutoff 0 / above Nyquist; DC decays; 1 kHz within 0.05 dB; 40 Hz down ≥ 10 dB; impulse response peaks at sample 0).
- `compressor.rs`: ≥ 3 (under the knee unchanged; gain never above 1; length unchanged).
- `loudness.rs`: the 5 cases of §7.6 (1 kHz sine at -20 dBFS peak reads -23.0 ± 0.1 LU; silence `None`; three `gain_db` values).
- `limiter.rs`: the 4 situations of §7.7 (unchanged under ceiling; square wave ≤ -1 dBFS; single spike with a 240-sample ramp before it; length), each asserting the returned count is 0.

#### Integration Tests
None yet.

#### Failure-Path Tests
`let x = v[0];` in `highpass.rs` → `cargo clippy -p offcut-dsp -- -D warnings` fails on `indexing_slicing`; undo.

### Validation Commands

```bash
grep -cE '^pub const [A-Z_0-9]+: ' crates/offcut-dsp/src/config.rs      # 32
cargo test -p offcut-dsp --lib                                          # config 1, highpass 6, compressor ≥3, loudness 5, limiter 4, plus V2's resample cases
cargo clippy -p offcut-dsp --all-targets -- -D warnings
cargo build -p offcut-dsp --target wasm32-unknown-unknown --release
```

### Acceptance Criteria

- [ ] 32 constants, equal to §7.2 and Appendix D; `ChainConfig::default()` built without `unwrap`.
- [ ] All inline cases listed pass; clippy clean; the wasm32 build succeeds.
- [ ] `limit` returns `usize` and every limiter case asserts 0 clamped samples.
- [ ] `git grep -niE 'silence|pause|gap' -- crates/offcut-dsp/src` shows only `ABSOLUTE_SILENCE` and comments saying what the crate does not do.

### Agent Instructions

Before: read §7.1–§7.3, §7.5–§7.7, TS §18, §34.3, Appendix D. During: one file and its tests at a time; `cargo test -p offcut-dsp --lib <module>` after each. After: SAB gate; commit `V3 P66: DSP constants, high-pass, compressor, loudness, limiter`.

### Expected Result

`offcut-dsp` has every stage of the chain except the denoiser, each proven alone, ready to be composed.

---

## Prompt 67 (V3 07/30) — Denoise, `run_chain`, and the length, loudness and click tests; measure the denoiser's delay

**Effort: High.** The riskiest DSP prompt: sample-exact denoiser delay compensation, length preservation proven by proptest (INV-10), click tests at 59 block boundaries, and a measured constant.

### Objective

Compose the full voice chain (`run_chain`, TS §34.3) with an RNNoise denoiser whose delay is compensated to the sample, and prove P1–P8 with the three integration test files that read the speech PCM fixtures; measure `DENOISE_DELAY_SAMPLES` (G 3.4–3.6; §7.4, §7.8, §19.4–§19.6).

### Project State Before This Prompt

- Prompts 63 and 66 committed: `fixtures/.gen/speech/*.pcm48.f32` exist with a matching `STAMP`; `config.rs`, `highpass.rs`, `compressor.rs`, `loudness.rs`, `limiter.rs` pass their inline tests.
- Verify: `ls fixtures/.gen/speech/*.pcm48.f32 | wc -l` is 5; `limit` returns `usize`.

### Why This Comes Here

The chain composes Prompt 66's stages; its tests need the derived speech PCM of Prompt 63. The delay constant must be measured before the browser reading (Prompt 69) means anything.

### Scope

#### Implement

- `src/denoise.rs` (§7.4): `Denoise::new(mix)`, `process(&mut self, input, out)`, `finish(&mut self, out)`; 480-sample frames scaled by `DENOISE_SCALE`; `nnnoiseless::DenoiseState::process_frame`; wet/dry mix with the dry path delayed by `DENOISE_DELAY_SAMPLES` through a ring; the first `DENOISE_DELAY_SAMPLES` mixed outputs dropped; `finish` pushes zeros until every input sample has its output. The VAD value is discarded.
- `src/chain.rs` (§7.8): `Flow`, `ChainOutput`, `run_chain(pcm48, cfg, on_block)` — steps 1–7: checks (`Empty`, `NonFinite`), one working buffer, per-1-s-block high-pass → denoise → compressor on the samples the denoiser produced, `on_block(SampleCount(done))` after each block with `Cancel` → `Ok(None)` before the next block, flush, loudness (measure, gain, apply), limiter, one corrective pass when the gate passed, the gain was not capped and the result is off by more than `LOUDNESS_RETRY_LU`, result with `loudness` (or `absolute_silence`) and `peak`. `cfg.denoise == false` skips the denoiser and compresses the high-passed block in place. `debug_assert_eq!` on lengths; no pad or trim branch.
- `src/lib.rs`: `pub mod chain; pub mod denoise;` and `pub use chain::{ChainOutput, Flow, run_chain};`.
- The fixture helper (§6.4, G item 1), 25 lines, copied into each of the three test files below: directory from `OFFCUT_FIXTURES_DIR` or `<CARGO_MANIFEST_DIR>/../../fixtures/.gen`; `STAMP`'s `manifest_sha256` against the SHA-256 (`sha2`) of `fixtures/manifest.json`; panic "the generated fixtures are stale or missing: run `pnpm fixtures`" on a mismatch or missing file.
- `tests/chain_no_clicks.rs` (§19.5): 7 cases + `chain_timing` (`#[ignore]`, prints, asserts nothing); the speech case on `speech_60s_portrait.mp4.pcm48.f32` at all 59 internal boundaries.
- `tests/duration_preserved.rs` (§19.6): the 7 cases with TS §34.3's names, including the proptest `prop_length_equals_input` over lengths 1–300,000 with the listed edge lengths.
- `tests/loudness_target.rs` (§19.4): 10 cases (six synthetic signals made in the test, three speech PCM files, the clamp count 0 across all).
- Measure `DENOISE_DELAY_SAMPLES` (G 3.5): run `impulses_stay_aligned --nocapture`, set the constant to the printed offset, re-run until it passes; write the value under `DENOISE_DELAY_SAMPLES` in `docs/v3/experiments.md`. If no constant works (offset varies), set `denoise: false` in `ChainConfig::default()` (TS §18 contingency) and record.

#### Do Not Implement Yet

- `prosody.rs`, `tests/prosody.rs`, TE-13 timing (Prompt 68). The binding and worker (Prompt 69).

### Existing Components To Reuse

Prompt 66's stages and constants; `nnnoiseless` (Prompt 62); `sha2` dev-dependency; `proptest`.

### Files To Create

- `crates/offcut-dsp/src/denoise.rs`, `crates/offcut-dsp/src/chain.rs`.
- `crates/offcut-dsp/tests/chain_no_clicks.rs`, `crates/offcut-dsp/tests/duration_preserved.rs`, `crates/offcut-dsp/tests/loudness_target.rs`.

### Files To Modify

- `crates/offcut-dsp/src/lib.rs`, `crates/offcut-dsp/src/config.rs` (the measured delay only), `crates/offcut-dsp/Cargo.toml` (only if `nnnoiseless` was deferred), `docs/v3/experiments.md`.

### Architecture Requirements

- **INV-10, INV-5:** output length equals input length for every input (P1–P3); no word timing enters any function that returns audio; no silence, pause or gap logic. The fixture-only words "silence" in test names are allowed in `tests/`.
- Memory: at most three clip-length `f32` buffers (§7.8); the denoiser's output vector becomes the working buffer after phase 3.
- Determinism (P8): same input, same bits, with and without denoise.

### Data Types / Interfaces

```rust
pub enum Flow { Continue, Cancel }
pub struct ChainOutput { pub samples: Vec<f32>, pub loudness: Lufs, pub peak: Dbfs }
pub fn run_chain(pcm48: &[f32], cfg: &ChainConfig, on_block: &mut dyn FnMut(SampleCount) -> Flow)
    -> Result<Option<ChainOutput>, DspError>;
```

### Implementation Requirements

#### 1. Delay compensation
Wet and dry are aligned before they are mixed; `mix = 0.0` gives the input exactly (sample-exact after compensation).

#### 2. Blocks
`BLOCK_SAMPLES` = 48,000; the last block may be shorter and is processed; each stage keeps its state across blocks, so a boundary is invisible (P6: step ≤ `CLICK_FACTOR` x local RMS).

#### 3. Loudness correction
Never more than one corrective pass (step 6).

### Error Handling

`Err(DspError::Empty)` on an empty input; `Err(DspError::NonFinite)` on any NaN or infinity, checked before processing. Cancel is `Ok(None)`, not an error. Digital silence is `Ok` with gain 0 dB and `loudness = absolute_silence` (TS §18.4).

### Concurrency / Async Requirements

Synchronous; the cancellation point is the `on_block` callback once per block (P7).

### Persistence Requirements

None.

### API Requirements

The `run_chain` signature and P1–P8 are frozen from here (§21.1).

### Security Requirements

None beyond purity; the tests read only `fixtures/.gen/`.

### Testing Requirements

#### Unit Tests
`denoise.rs`: 3 inline cases (total length over many calls plus `finish`; silence stays silent; `mix` 0 gives the input). `chain.rs`: the 4 `run_chain` cases of §7.10 (1 sample; 48,001 samples; cancel at first call → `Ok(None)` and called once; `denoise: false` same length and on target).

#### Integration Tests
The three test files: 7 + 1 ignored, 7, 10 cases.

#### Failure-Path Tests
G 3.9 rows 1–3: `DENOISE_DELAY_SAMPLES = 0` → `impulses_stay_aligned` fails; skip the last partial block in `chain.rs` → `prop_length_equals_input` fails with a shrunk length; change a byte of `fixtures/.gen/STAMP` → every fixture-reading DSP test panics "run `pnpm fixtures`". Undo all, run `pnpm fixtures`. Delete any `proptest-regressions/` folder a drill created (V2 CL known issue 12).

### Validation Commands

```bash
cargo test -p offcut-dsp --test duration_preserved -- --nocapture impulses_stay_aligned
cargo test -p offcut-dsp --test chain_no_clicks --test duration_preserved      # 13 passed, 1 ignored (chain_timing)
cargo test -p offcut-dsp --test loudness_target                                # 10 passed
cargo test -p offcut-dsp --lib                                                 # all passed
cargo clippy -p offcut-dsp --all-targets -- -D warnings
```

### Acceptance Criteria

- [ ] `chain_no_clicks` + `duration_preserved`: 13 passed, 1 ignored; `loudness_target`: 10 passed with 0 clamped samples everywhere.
- [ ] `DENOISE_DELAY_SAMPLES` holds a measured value with no "(assumption" comment, and `experiments.md` records it (or the denoise-off fallback is recorded).
- [ ] The helper with the stamp check is in all three test files; the drills fired and were undone.
- [ ] No pad/trim branch in `chain.rs`; clippy clean; wasm32 build still succeeds.

### Agent Instructions

Before: read §7.4, §7.8, §7.10, §19.4–§19.6, TS §34.3 (the exemplar), §6.4, G 3.4–3.6, G item 1. During: if a length test fails, fix `denoise.rs`'s bookkeeping, never pad in `chain.rs`. After: SAB gate; commit `V3 P67: denoise, run_chain, P1-P8 tests, measured denoiser delay`.

### Expected Result

A length-preserving, delay-compensated chain hitting -14 LUFS ±0.5 (±1 for the square wave) on synthetic and speech inputs, natively.

---

## Prompt 68 (V3 08/30) — Prosody, `tests/prosody.rs`, native TE-13 **[gate]**

**Effort: High.** An autocorrelation pitch tracker (decimation, parabolic refinement, per-sentence z-scores) plus the native TE-13 gate, whose result can change the design.

### Objective

Implement `measure_prosody` (per-word energy and pitch z-scores within each sentence, D-96, D-108) with its tests, read the native TE-13 time, and close Milestone 3 (G 3.7–3.9, M-3).

### Project State Before This Prompt

- Prompt 67 committed: `run_chain` passes P1–P8; `DENOISE_DELAY_SAMPLES` is measured.
- Verify: `cargo test -p offcut-dsp` is green with 1 ignored.

### Why This Comes Here

Prosody reads the cleaned audio (TS §18.1, §18.4); TE-13's native time is the earliest warning before the browser reading (§5 S4, §20.1). A native median of 2 s or more stops the build here.

### Scope

#### Implement

- `src/prosody.rs` (§7.9): `measure_prosody(out48, words, sentences) -> Prosody`; word ranges `[start_ms x 48, end_ms x 48)` clamped; energy `10 log10(mean(x²))` floored at `ENERGY_FLOOR_DB`; pitch on an 8 kHz copy (mean of each group of 6), 320-sample frames at hop 80 wholly inside the word, frames under `VOICED_FRAME_MIN_DB` skipped, normalised autocorrelation over lags 20–134, voiced when the best `r ≥ VOICING_MIN`, parabolic refinement, `F0 = 8,000 / τ`; word F0 = median of voiced frames (lower middle for an even count); per-sentence z-scores of energy and of `log2(F0)` (population sd); fewer than `PROSODY_MIN_WORDS` values or sd < 1e-6 → 0.0 for the sentence; a word without pitch → `pitch_z = 0.0`; `per_word.len() == words.len()`.
- `src/lib.rs`: `pub mod prosody; pub use prosody::measure_prosody;`.
- Inline tests: the 5 prosody cases of §7.10.
- `tests/prosody.rs` (§19.7): the 10 cases with the fixture helper (the speech case reads `speech_60s_portrait.mp4.pcm48.f32` with hand-copied times of sentences 3–4 from the README, prints the "brutal" reading and asserts nothing; the budget case asserts under 2 s only under `#[cfg(not(debug_assertions))]`).
- Copy the printed "brutal" z-scores into `experiments.md` (the input of the conditional KeywordPop, §17.7).
- G 3.8 TE-13 native: `chain_timing` in release, five runs, median for 60 s; write it under `TE-13 native` with date, label `d1`, decision.
- G 3.9 row 4 drill and M-3 checks.

#### Do Not Implement Yet

The binding, worker and browser reading (Prompt 69).

### Existing Components To Reuse

`config.rs` prosody constants; `offcut_types::{Word, Sentence, Prosody}` (generated since V1; do not add a type); the fixture helper of Prompt 67 (copy it into `tests/prosody.rs`).

### Files To Create

- `crates/offcut-dsp/src/prosody.rs`, `crates/offcut-dsp/tests/prosody.rs`.

### Files To Modify

- `crates/offcut-dsp/src/lib.rs`, `docs/v3/experiments.md`, `docs/v3/v3changelog.md`.

### Architecture Requirements

- **Reads only**: never returns or changes audio; never reads outside a word's range; makes no decision (§7.9 "Never").
- Budget: under 1 s for the reference clip (TS §8, assumption M1.2) — about 30 frames x 115 lags x 320 multiply-adds per word.

### Data Types / Interfaces

```rust
pub fn measure_prosody(out48: &[f32], words: &[Word], sentences: &[Sentence]) -> Prosody;
```

### Implementation Requirements

#### 1. Exactness
Decimate before the autocorrelation (D-108); no `HashMap`; fixed summation order.

#### 2. TE-13 decision (G 3.8)
Median under 2 s: continue. 2 s or more: look for per-sample work that belongs per block (allocations, `exp`/`log10` per sample, `ebur128` fed sample by sample); still ≥ 2 s → set `denoise: false` (§20.2) and record before Prompt 69.

### Error Handling

No error type: empty words → empty `per_word`; a range past the end → energy floor and no pitch, no panic.

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

The signature is frozen from here (§21.1).

### Security Requirements

None.

### Testing Requirements

#### Unit Tests
§7.10's five prosody cases (zero words; a 12 dB louder third word has the largest `energy_z` > 1.0; a 200 Hz word among 120 Hz words has the largest `pitch_z` > 1.0; two-word sentence 0/0; determinism).

#### Integration Tests
`tests/prosody.rs`: 10 cases as §19.7.

#### Failure-Path Tests
G 3.9 row 4 (indexing drill) if not done; a word range past the end (in the suite).

### Validation Commands

```bash
cargo test -p offcut-dsp --test prosody -- --nocapture           # all passed; prints the "brutal" reading
for i in 1 2 3 4 5; do cargo test -p offcut-dsp --release --test chain_no_clicks -- --ignored --nocapture chain_timing; done
pnpm fixtures
cargo test -p offcut-dsp 2>&1 | awk '/^test result:/ {f+=$6; i+=$8} END {print f" failed, "i" ignored"}'   # 0 failed, 1 ignored
cargo clippy -p offcut-dsp --all-targets -- -D warnings
cargo build -p offcut-dsp --target wasm32-unknown-unknown --release
git grep -niE 'silence|pause|gap' -- crates/offcut-dsp/src
grep -n "DENOISE_DELAY_SAMPLES" crates/offcut-dsp/src/config.rs
```

### Acceptance Criteria

- [ ] `measure_prosody` passes 5 inline and 10 integration cases; `per_word.len() == words.len()` for 0, 1, 150 words.
- [ ] `experiments.md` has `TE-13 native` (median of five, ms per 60 s, `d1`, decision) and the "brutal" reading.
- [ ] Median under 2 s, or the denoise-off fallback is set in `ChainConfig::default()` and recorded.
- [ ] G M-3 passes in full.

### Agent Instructions

Before: read §7.9, §7.10, §19.7, D-96, D-108, §20.1–§20.2, G 3.7–3.9. During: keep `prosody.rs` under 400 lines. After: SAB gate; commit `V3 P68: prosody and native TE-13`. If the gate stopped the build, say so first in the reply.

### Expected Result

`offcut-dsp` is complete natively: chain, prosody, measured delay and a native time that permits the browser step.

---

## Prompt 69 (V3 09/30) — The chain in the browser: `audio_api.rs`, `CoreApi`, `audio.worker.ts`, the pool row, browser TE-13 **[gate]**

**Effort: High.** Rust-to-WASM binding, a new worker with transfer and cancellation, a pool change, browser timing in dev and in the production build (COOP/COEP), bundle size, and the browser TE-13 gate.

### Objective

Run the voice chain inside a new fourth worker on D1, returning `out48` of exactly the input's length at -14 ±1 LUFS in at most 4 s per 60 s of audio, under the production build and headers (Phase 4 of the guide; §10.1, §13.2, §13.4, §14.1; D-110).

### Project State Before This Prompt

- Prompt 68 committed; native TE-13 permits continuing.
- V2: `offcut-wasm-core` depends on `offcut-dsp` (for `resample`); `serveWorker`, `WorkerFailure`, `CANCELLED` exist in `workers/rpc.ts`; `AudioWorkerApi` exists in `protocol.ts` since V1; `pool.ts` has three rows.
- Verify: `git grep -n "AudioWorkerApi" -- web/src/workers/protocol.ts`; `git grep -n "worker&url" -- web/src/workers/pool.ts`.

### Why This Comes Here

TE-13 in the browser is the reading most likely to change the design; it depends on S4 only (G item 8). Everything after it (pipeline, E2E) assumes the chain fits.

### Scope

#### Implement

- `crates/offcut-wasm-core/src/audio_api.rs` (§10.1): `run_chain(pcm48: &[f32], on_block: &js_sys::Function)` → `{ out48: Float32Array, loudness, peak }` or `{ cancelled: true }`, calling `offcut_dsp::run_chain` with `ChainConfig::default()`; a truthy callback return is `Cancel`; a throwing callback cancels and the call fails `{ code: "E_INTERNAL", detail: "Callback" }`; `out48` is a new `Float32Array` with its own buffer; errors `E_DSP` / `Empty` | `NonFinite`. `measure_prosody(out48, words, sentences)`: details `Words`/`Sentences` on decode failure (`E_INTERNAL`). Conversion through the crate's existing `to_js`/`from_js` (`json_compatible`, D-31).
- `crates/offcut-wasm-core/src/lib.rs`: `mod audio_api;` and nothing else.
- `web/src/wasm/load-core.ts` (§13.2): `ChainResult` type; `runChain(pcm48, onBlock)`, `measureProsody(out48, words, sentences)` on `CoreApi`; one `as ChainResult` cast of the decoded object; no `as <Brand>` of a primitive.
- `web/src/workers/audio.worker.ts` (§14.1, Appendix C): `serveWorker(handlers, { stage: "audio_chain", oneWay: [], duringPreview: [] })`; module state `kept`; `runChain({ pcm48 }, ctx)` — `kept = null` first, `loadCore`, run with `ctx.progress({ stage: "audio_chain", done, total })` and `ctx.isCancelled()` per block, `CANCELLED` on cancel, keep `out48`, return `{ loudness }`; `measureProsody({ words, sentences })` — `NoAudio` when nothing kept, return `{ out48, prosody }` transferred, `kept = null`. About 60 lines.
- `web/src/workers/pool.ts` (§13.4, V2 CL item 14): `import audioWorkerUrl from "./audio.worker.ts?worker&url"`, one `WORKERS` row (stage `audio_chain`), one `lazyClient<AudioWorkerApi>`; `preload()` now adds four links.
- Browser checks (G 4.3, 4.5) through a temporary zz spec: in dev, route `**/zz-pcm.f32` to `fixtures/.gen/speech/speech_60s_portrait.mp4.pcm48.f32`, build a `Float32Array` from it in the page, `const { pool } = await import("/src/workers/pool.ts")`; first `measureProsody` with nothing kept → `E_INTERNAL`/`NoAudio`; `runChain` with `transfer`, time it, then `measureProsody({ words: [], sentences: [] })`: `same: true`, `detached: true`, `n: 2880000`, loudness between -15 and -13, ms ≤ 4000; three runs, median. Then on `vite preview` (production build, COOP/COEP): `crossOriginIsolated === true`, one run within 20% of dev; there the pool is reached through a temporary `web/src/zz-spike.ts` imported from `main.tsx`, removed before the gate. If `vite preview` lacks V2's headers, use the way V2 checked the production build (its changelog); add no header to any config.
- G 4.4: `pnpm build` passes `check-hosts.mjs`; record `offcut_core_bg-*.wasm` before (Entry 1) and after; growth over 2 MB → find the crate. A URL literal from `nnnoiseless`/`ebur128` goes to `NON_NETWORK_LITERALS` scoped to `offcut_core_bg-*.wasm` with its reason (D-38), never globally.
- `experiments.md`: `TE-13 browser` (median, decision, date, `d1`), `Bundle size`.

#### Do Not Implement Yet

- Calling the audio worker from `run-pipeline.ts` (Prompt 81). `redetect_sentence` binding (Prompt 79).

### Existing Components To Reuse

`to_js`/`from_js`/failure helpers of `offcut-wasm-core`; `serveWorker`, `WorkerFailure`, `CANCELLED`, `lazyClient`, `WORKERS` (V2); `AudioWorkerApi` as it stands (V1, frozen: `runChain(p: { pcm48 }) → { loudness }`, `measureProsody(p: { words, sentences }) → { out48, prosody }`).

### Files To Create

- `crates/offcut-wasm-core/src/audio_api.rs`, `web/src/workers/audio.worker.ts`.

### Files To Modify

- `crates/offcut-wasm-core/src/lib.rs`, `web/src/wasm/load-core.ts`, `web/src/workers/pool.ts`; `scripts/check-hosts.mjs` only if a literal appears (one scoped entry); `docs/v3/experiments.md`, `docs/v3/v3changelog.md`.

### Architecture Requirements

- The worker imports only `wasm/load-core`, `workers/rpc`, `workers/protocol` (types), `gen/domain` (types). No state, UI, IndexedDB or network (TS §14). No product rule in the binding (TS §2).
- PCM crosses threads by transfer, never copy (TS §31); into and out of WASM memory by copy (D-110: 2 x 17.3 MB for 90 s).
- The pool holds at most four workers, none created before its first call.

### Data Types / Interfaces

```ts
export type ChainResult = { out48: Float32Array<ArrayBuffer>; loudness: Lufs; peak: Dbfs } | { cancelled: true };
runChain(pcm48: Float32Array, onBlock: (done: number) => boolean): ChainResult;
measureProsody(out48: Float32Array, words: readonly Word[], sentences: readonly Sentence[]): Prosody;
```

### Implementation Requirements

#### 1. Binding
No branch on a domain value beyond decoding arguments and mapping `DspError` to `{ code, detail }`; `detail` is an enum variant name, never audio or text.

#### 2. Worker
`kept = null` is the first statement of `runChain` (a buffer left by an earlier clip is dropped, D-90). Progress once per block (about 75 for the reference clip).

#### 3. Gate (§20.2)
Median ≤ 4,000 ms per 60 s: continue. Over: `denoise: false` in `ChainConfig::default()`, re-run `cargo test -p offcut-dsp` and the measurement, record both.

### Error Handling

`E_DSP` with its detail from the module; anything else `E_INTERNAL` with stage `audio_chain`; `measureProsody` without a kept buffer → `WorkerFailure("E_INTERNAL", "NoAudio")`. Never an empty `catch`.

### Concurrency / Async Requirements

One job at a time per worker (rpc). Cancellation per 1 s block via `ctx.isCancelled()` passed as a closure, never unbound (V2 CL item 13). The worker never keeps audio after `measureProsody` (TS §18.4; the early-end case is §23 item 25, accepted).

### Persistence Requirements

None (the worker touches no storage).

### API Requirements

`audio_api.rs` exports and `ChainResult` are frozen from here (§21.1).

### Security Requirements

`check-hosts.mjs` still passes; any new literal entry is scoped to one chunk with a reason. The worker logs nothing.

### Testing Requirements

#### Unit Tests
None (TS §5 has no test file for the binding; covered by the browser checks here and the media suites in Prompt 85).

#### Integration Tests
The two browser checks; `pnpm e2e:media --grep @no-gpu` (8 passed: V2's no-GPU cases with the fourth worker present).

#### Failure-Path Tests
`measureProsody` with nothing kept → `NoAudio` (in the browser check).

### Validation Commands

```bash
pnpm build:wasm
cargo clippy -p offcut-wasm-core --target wasm32-unknown-unknown -- -D warnings
pnpm check && pnpm build
ls -l web/dist/assets/ | grep -E 'offcut_core_bg-.*\.wasm'
pnpm --filter web exec playwright test --config zz-pw.config.ts      # the temporary checks; print ms, n, same, detached, loudness
pnpm e2e:media --grep @no-gpu                                        # 8 passed (keyed build not required for these)
```

### Acceptance Criteria

- [ ] In dev and in preview: `same: true`, `detached: true`, `n: 2880000`, loudness -15…-13; the first `measureProsody` failed with `NoAudio`; `crossOriginIsolated` true in preview.
- [ ] `experiments.md` has `TE-13 browser` (median of three, ≤ 4,000 ms or the recorded fallback) and `Bundle size` (before/after bytes).
- [ ] `pnpm build` passes `check-hosts.mjs`; any new literal entry is chunk-scoped.
- [ ] No `zz-` file remains; `main.tsx` is unchanged after the check; `git grep -n "modulepreload" -- web/src/workers/pool.ts` shows four scripts covered; G M-4 passes.

### Agent Instructions

Before: read §10.1, §13.1–§13.4, §14.1, Appendix C (audio.worker), D-31, D-110, V2 CL items 13 and 14, G Phase 4. During: no change to `protocol.ts` here. After: SAB gate; commit `V3 P69: audio worker and the voice chain in the browser (TE-13)`. If the gate applied the fallback, say so first.

### Expected Result

`pool.audio.runChain` and `pool.audio.measureProsody` work in the browser on D1 within the TE-13 budget; nothing in the app calls them yet.

---

## Prompt 70 (V3 10/30) — `offcut-mp4`: edit lists E0–E4 and probe hardening H1–H8

**Effort: High.** A refactor inside the parser of untrusted MP4 input: move code without changing E0-E2 behaviour, add E3/E4 with checked arithmetic, harden the probe, stay under 400 lines.

### Objective

Move edit-list resolution into `edit_list.rs` with two new shapes (E3 contiguous media edits, E4 a trailing empty edit), keep `sample_table.rs` under 400 lines, and harden `probe.rs` per H1–H8 — with no public signature change and `validate.rs` untouched (G 5.1, 5.2; §6.1–§6.3; D-92).

### Project State Before This Prompt

- Prompt 69 committed. V2's `sample_table.rs` has 387 lines above its tests and a private `edits()` with helpers `to_media` and `shown` (V2 CL known issue 10).
- Verify: `awk '/#\[cfg\(test\)\]/{print NR-1; exit}' crates/offcut-mp4/src/sample_table.rs` prints 387 (or the real number, recorded).

### Why This Comes Here

The fixture tests of Prompt 71 exercise these shapes and rules; the generator (Prompt 63) exists. Edit-list widening cannot fit in `sample_table.rs` (D-92).

### Scope

#### Implement

- `src/edit_list.rs` (crate-private, §6.1): `EditPlan { edit_offset, presented }` in media ticks; `resolve_edits(edits, media_timescale, movie_timescale, media_duration) -> Result<EditPlan, ContainerError>` trying E0–E4 in order; anything else `Unsupported("elst")`; at most `MAX_EDITS = 16` entries examined; checked additions → `Malformed("elst")`; `movie_timescale` 0 → `Malformed("mvhd")`; a `segment_duration` of 0 on the last media edit means "to the end of the media"; one tick of mismatch between contiguous edits accepted, two refused.
- `src/sample_table.rs`: the private `edits()` keeps its signature and becomes: sum the `stts` runs with V2's overflow checks, call `resolve_edits`, return `(plan.edit_offset, plan.presented)`; `to_media` and `shown` move to `edit_list.rs`; `Edit` stays here with its two fields. V2's two inline edit-list tests pass unchanged.
- `src/lib.rs`: `mod edit_list;` (private).
- `src/probe.rs` (§6.3): H1 matrix entries rounded to -1/0/1 in 16.16 when within 1/256, else `R0`; H3 the last sample's duration ignored for `max_fps` when there is more than one sample; H4 the 24-bit explicit rate of an ASC with frequency index 15. H2, H5–H8 are covered by fixture tests (Prompt 71); no behaviour change for them.
- TS §5: add the `edit_list.rs` line only if Prompt 61 deferred it.

#### Do Not Implement Yet

- `demux_fixtures.rs`, `probe_rejections.rs`, manifest observations (Prompt 71). Reading `media_rate` (open, §23 item 15). Any `validate.rs` edit (frozen).

### Existing Components To Reuse

`ContainerError` (V2), `sample_table::Edit`, the overflow-checked sum of V2's `edits()`, `demux::fixture` (V2's test-only MP4 builder, V2 CL item 11) for inline tests.

### Files To Create

- `crates/offcut-mp4/src/edit_list.rs`.

### Files To Modify

- `crates/offcut-mp4/src/sample_table.rs`, `crates/offcut-mp4/src/lib.rs`, `crates/offcut-mp4/src/probe.rs`; `docs/technicalspec.md` §5 only if deferred.

### Architecture Requirements

- No public signature of the crate changes (§6). `Unsupported("elst")` never reaches `validate_probe`: `media_api.rs` (frozen) maps it to `REJECT_CORRUPT`.
- No allocation from an untrusted count without bounding it by file size (D-60). `probe` still never fails.

### Data Types / Interfaces

```rust
pub(crate) struct EditPlan { pub edit_offset: i64, pub presented: i64 }
pub(crate) fn resolve_edits(edits: &[Edit], media_timescale: u32, movie_timescale: u32, media_duration: i64)
    -> Result<EditPlan, ContainerError>;
```

### Implementation Requirements

#### 1. Shape table
E0 no `elst`; E1 one media edit; E2 one empty then one media; E3 two or more contiguous media edits (`media_time[k+1] == media_time[k] + duration[k]` within 1 tick), optionally after one leading empty; E4 any of E1–E3 followed by exactly one trailing empty edit, ignored.

#### 2. Size
`edit_list.rs`, `sample_table.rs`, `probe.rs` each under 400 lines above tests.

### Error Handling

`Unsupported("elst")` for shapes outside the table and lists over 16; `Malformed("elst")` on overflow; `Malformed("mvhd")` on a zero movie timescale. No panic, no indexing.

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

None public.

### Security Requirements

Bounds per D-60: the 16-entry cap protects against a crafted `elst`.

### Testing Requirements

#### Unit Tests
`edit_list.rs`: the 12 cases of §6.1. `probe.rs`: H1 (off by one unit → R90/R270; off by more than 1/256 → R0), H3, H4.

#### Integration Tests
V2's `mux_roundtrip.rs` and every V2 inline test of the crate pass unchanged.

#### Failure-Path Tests
G 5.5 row 2: `MAX_EDITS = 2` → the E3 inline case fails; undo.

### Validation Commands

```bash
cargo test -p offcut-mp4 --lib                        # V2's count + 12 (edit_list) + the probe cases, 0 failed
awk '/#\[cfg\(test\)\]/{print NR-1; exit}' crates/offcut-mp4/src/sample_table.rs    # under 400
git diff --name-only "$BASE" -- crates/offcut-mp4/src/validate.rs                    # no output
cargo clippy -p offcut-mp4 --all-targets -- -D warnings
pnpm build:wasm
```

### Acceptance Criteria

- [ ] 12 `edit_list` cases and the H1, H3, H4 cases pass; V2's two edit-list tests pass unchanged.
- [ ] `sample_table.rs` is under 400 lines above its tests; `validate.rs` unchanged; no public signature changed.
- [ ] The `MAX_EDITS` drill fired and was undone; `check-file-tree.mjs` passes.

### Agent Instructions

Before: read §6, §6.1–§6.3, D-60, D-92, V2 §6.4 and §6.6, and the V2 `sample_table.rs` and `probe.rs` in full. During: move code, do not rewrite it; keep V2's behaviour for E0–E2 bit-identical. After: SAB gate; commit `V3 P70: edit-list shapes E0-E4 and probe hardening H1-H8`.

### Expected Result

The demuxer accepts joined and screen-recorder edit lists and reads webcam timing and odd matrices correctly, ready for the fixture matrix.

---

## Prompt 71 (V3 11/30) — `demux_fixtures.rs`, `probe_rejections.rs`, the observed manifest values

**Effort: High.** The first time H2 and H5-H8 meet real ffmpeg output. A mismatch is a parser bug to fix in `probe.rs`/`edit_list.rs`, never a test to loosen. Includes the D-100 privacy metadata walk.

### Objective

Prove natively that every `ok_*` fixture and every derived speech clip opens and validates with its expected properties, and that every `rej_*` fixture except `rej_silent.mp4` gives its manifest code; fill the two observed manifest values (G 5.3–5.5, M-5; §19.2, §19.3).

### Project State Before This Prompt

- Prompts 63 and 70 committed. The manifest has `ok_rotated_90.mov` rotation `null` and the two-track fixture present.
- Verify: `pnpm fixtures` prints "fixtures are up to date".

### Why This Comes Here

The tests need both the generator and the hardened demuxer. They are BP §5.4's first exit tests (demux and rejection conformance).

### Scope

#### Implement

- `tests/demux_fixtures.rs` (§19.2): 12 cases; the fixture helper (stamp check) copied in; `expect.accept` read from `fixtures/manifest.json` with a small `serde_json` dev struct; files read with `std::fs::read` into `MemReader`; the metadata case walks boxes with `boxes::children` and finds no `udta` `©xyz`/`©day` and no `meta` key `com.apple.quicktime.location.ISO6709` or `creationdate` in any derived clip (D-100).
- Run with printing first (G 5.3), then write `R90` or `R270` (as the demuxer reports) into the manifest; if the two-video fixture's video track is not 640x360 (ffmpeg did not mark the first disabled), remove its entry from the manifest and keep H2 as an inline test only (§17.2). Each manifest edit → `pnpm fixtures` → test again. Record both observations.
- `tests/probe_rejections.rs` (§19.3): 15 cases; the `ContainerError` → code mapping of `media_api.rs` (frozen) reproduced in one private function; the manifest-coverage case fails if a `rej_*` entry other than `rej_silent.mp4` has no case.
- G 5.5 drills and M-5.

#### Do Not Implement Yet

The browser rejection suite (Prompt 84). Any change to `media_api.rs` or `validate.rs`.

### Existing Components To Reuse

`Demuxer::open`, `probe`, `validate_probe`, `MemReader`, `boxes::read_header`, `boxes::children`, `keyframe_at_or_before` (all V2, public); the helper from Prompt 67.

### Files To Create

- `crates/offcut-mp4/tests/demux_fixtures.rs`, `crates/offcut-mp4/tests/probe_rejections.rs`.

### Files To Modify

- `fixtures/manifest.json` (two observed values or one removal), `docs/v3/v3changelog.md`.

### Architecture Requirements

Tests may use `std::fs` (tests only). The crate gains no public item for the tests.

### Data Types / Interfaces

A dev-only `#[derive(Deserialize)] #[serde(deny_unknown_fields)]` mirror of the manifest entry in each test file (no shared test-support file: TS §5 has none).

### Implementation Requirements

#### 1. `demux_fixtures.rs` rows
All `ok_*` and derived open and validate; `ok_vfr.mp4` `is_vfr` and `avg_fps` in range with strictly increasing presentation times; `ok_rotated_90.mov` the recorded rotation, display 720x1280, Portrait; `ok_60fps.mp4` 60,000 ± 100; `ok_moov_at_end.mp4` `moov` after `mdat`; `ok_portrait_3x4.mp4` 1080x1440 Portrait; the two-track fixture 640x360; `speech_60s_portrait.mp4` 60,000 ± 1 ms, 1,800 video samples, `mp4a.40.2` 48 kHz stereo; `speech_20s_long_pause.mp4` 20,000 ± 1 ms; every video sample readable with keyframe flags matching `stss`; `keyframe_at_or_before` on `ok_60fps.mp4` for 0, 499, 500, 2,999 ms → 0, 0, 30, 150; the metadata walk.

#### 2. `probe_rejections.rs` rows
As §19.3, including `rej_h264_444.mp4` with `decode_supported` true (`Ok`) and false (`DecodeUnsupported`), and `ok_90s_exact.mp4` `Ok`.

### Error Handling

A fixture whose observed property differs from the manifest: the manifest is corrected only for the two observed fields; any other mismatch is a demuxer bug to fix in `probe.rs`/`edit_list.rs`, never an expectation to loosen.

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

The metadata case (D-100) is part of the privacy promise; it must pass.

### Testing Requirements

#### Unit Tests
None new.

#### Integration Tests
12 + 15 cases.

#### Failure-Path Tests
G 5.5: remove the `rej_mp3_audio.mp4` row from the test's mapping → the coverage case names it; move `fixtures/.gen` aside → `demux_fixtures.rs` panics "run `pnpm fixtures`". Undo both.

### Validation Commands

```bash
cargo test -p offcut-mp4 --test demux_fixtures -- --nocapture 2>&1 | grep -E "rotation|track|ok_vfr"
pnpm fixtures && cargo test -p offcut-mp4 --test demux_fixtures      # 12 passed (11 if the two-track entry was removed)
cargo test -p offcut-mp4 --test probe_rejections                     # 15 passed
for f in crates/offcut-mp4/src/edit_list.rs crates/offcut-mp4/src/sample_table.rs crates/offcut-mp4/src/probe.rs; do
  printf '%s ' "$f"; awk '/#\[cfg\(test\)\]/{print NR-1; f=1; exit} END{if(!f) print NR}' "$f"; done   # each under 400
cargo clippy -p offcut-mp4 --all-targets -- -D warnings
```

### Acceptance Criteria

- [ ] `demux_fixtures`: 12 (or 11) passed; `probe_rejections`: 15 passed; both read through the stamp-checking helper.
- [ ] The manifest holds the observed rotation; the two-track decision is recorded.
- [ ] The two drills fired and were undone; G M-5 passes.

### Agent Instructions

Before: read §6.3–§6.4, §19.2–§19.3, D-78, D-100, G 5.3–5.5. During: never edit `validate.rs` or `media_api.rs`. After: SAB gate; commit `V3 P71: demux and rejection fixture tests`.

### Expected Result

Native proof that the demuxer meets the PS §9.4 rejection table and the accept matrix.

---

## Prompt 72 (V3 12/30) — `offcut-text`: word lists, the wider `Digits`, spelled cardinals

**Effort: High.** The spelled-cardinal grammar (descending magnitudes, the "and" rule, teen-hundred, "a hundred", the 24-token cap) is where off-by-one bugs hide, and P74's property test will hammer it up to 999,999,999.

### Objective

Add every unit, currency, count and comparative word list (`units_lex.rs`), classify symbol- and suffix-bearing numbers as `Digits`, and read spelled cardinals to the trillions and "point" decimals (`cardinals.rs`) — the foundations of the full grammar (G 6.1–6.3; §8.1, §8.2, §8.4; D-84, D-92, D-101, D-102).

### Project State Before This Prompt

- Prompt 71 committed. V2's `offcut-text` has `lib.rs`, `tokenize.rs`, `numbers.rs` (262 lines above tests), `sentences.rs`, `normalize.rs`; its public surface is frozen (V2 §25.1).
- Verify: `git grep -n "pub fn" -- crates/offcut-text/src` lists `tokenize`, `parse_quantity`, `format_quantity`, `segment_sentences`, `normalize_transcript`.

### Why This Comes Here

`parse_quantity` (Prompt 73) reads these lists and the cardinal grammar; `offcut-detect` (Prompt 75) reads `COMPARATIVES`. A split by function keeps `numbers.rs` the only formatter and the only parsing entry (D-92).

### Scope

#### Implement

- `src/units_lex.rs` (public module, §8.4) with the 8 lists and `COUNT_UNIT_MIN = 100.0`, exactly as §8.4 (note `CURRENCY_SYMBOLS` sorted by code point: `$ £ € ₹`); private enums `CurrencyUnit`, `TimeUnit`, `DataUnit` converting into `offcut_types::Unit`; "pound(s)", "tb", "terabytes" absent on purpose.
- `src/tokenize.rs` (§8.1): after V2's lower-casing and punctuation split, a word is `Digits` when, after removing at most one leading `$ € £ ₹`, at most one trailing `% k m b x`, and thousands separators, the rest parses with V2's `parse_decimal`; the token keeps its text as written (lower-cased). Indian grouping, `10:30`, `5-10`, `forty-two` stay `Word`. `Token`/`TokenKind` unchanged.
- `src/cardinals.rs` (private, §8.2): `MAX_CARDINAL_TOKENS = 24`, `cardinal(words) -> Option<(usize, u64)>`, `point_fraction(words) -> Option<(usize, f64)>`; the grammar of §8.2 (magnitudes strictly descending, each at most once; "and" consumed only before 1–99; teen-hundred takes no magnitude; "a" + hundred/magnitude; value up to 999,999,999,999,999).
- `src/lib.rs`: `mod cardinals; pub mod units_lex;`.
- TS §5: the `cardinals.rs` line only if Prompt 61 deferred it.

#### Do Not Implement Yet

`parse_quantity`/`format_quantity` changes, the sentence split, per-sentence normalisation (Prompt 73); the three integration test files (Prompt 74).

### Existing Components To Reuse

V2's `parse_decimal`, `Token`, `TokenKind`, `RawWord`; `offcut_types::Unit` (generated, unchanged; no TB variant, §23 item 17).

### Files To Create

- `crates/offcut-text/src/units_lex.rs`, `crates/offcut-text/src/cardinals.rs`.

### Files To Modify

- `crates/offcut-text/src/tokenize.rs`, `crates/offcut-text/src/lib.rs`; `docs/technicalspec.md` §5 only if deferred.

### Architecture Requirements

- Pure; every lexicon a sorted `const` slice of lower-case `&str` (or `(&str, T)` / `(char, T)` sorted by key) searched with `binary_search` (§8 crate rules). No list read at run time.
- `units_lex` is public only because `offcut-detect` reads `COMPARATIVES` (D-102).

### Data Types / Interfaces

```rust
pub(crate) const MAX_CARDINAL_TOKENS: usize = 24;
pub(crate) fn cardinal(words: &[&str]) -> Option<(usize, u64)>;
pub(crate) fn point_fraction(words: &[&str]) -> Option<(usize, f64)>;
pub const COUNT_UNIT_MIN: f64 = 100.0;
```

### Implementation Requirements

#### 1. Lists
Copy §8.4 verbatim; one inline test per slice asserting sorted, unique, lower-case.

#### 2. Digits
The 10 rows of §8.1's table as inline cases.

#### 3. Cardinals
The 13 rows of §8.2 as inline cases.

### Error Handling

No error type: unreadable input returns `None` ("never guesses").

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

Public surface of the crate unchanged plus the `units_lex` module.

### Security Requirements

Bounded parsing: `MAX_CARDINAL_TOKENS` caps the look-ahead.

### Testing Requirements

#### Unit Tests
Lists (8 slice tests), Digits (10), cardinals (13).

#### Integration Tests
V2's crate tests pass unchanged (the "ten million" test changes in Prompt 73, not here).

#### Failure-Path Tests
G 6.7 row 1: swap two adjacent `COUNT_NOUNS` words → the sorted test fails; undo.

### Validation Commands

```bash
grep -cE '^pub const [A-Z_]+' crates/offcut-text/src/units_lex.rs     # 9 (8 lists + COUNT_UNIT_MIN)
cargo test -p offcut-text --lib
cargo clippy -p offcut-text --all-targets -- -D warnings
node scripts/check-file-tree.mjs
```

### Acceptance Criteria

- [ ] 9 public constants in `units_lex.rs`; every slice test passes.
- [ ] The 10 tokenize rows and 13 cardinal rows pass; V2's tests pass unchanged.
- [ ] The sorted-list drill fired and was undone.

### Agent Instructions

Before: read §8.1, §8.2, §8.4, D-84, D-92, D-101, D-102. During: do not touch `numbers.rs` yet. After: SAB gate; commit `V3 P72: word lists, wider Digits, spelled cardinals`.

### Expected Result

The text crate can recognise every token form and spelled number the V3 grammar reads.

---

## Prompt 73 (V3 13/30) — `offcut-text`: the full `parse_quantity`, `format_quantity`, the 40-word split, per-sentence numbers

**Effort: High.** The four-stage `parse_quantity`, the display table, the recursive 40-word split and per-sentence normalisation, with `numbers.rs` under 400 lines. Every event's value comes from here.

### Objective

Complete the number grammar and display rules of TS §17.2 in `numbers.rs`, split sentences over 40 words, and normalise numbers per sentence so no quantity crosses a sentence (G 6.4, 6.5; §8.3, §8.5, §8.6; D-104).

### Project State Before This Prompt

- Prompt 72 committed (`units_lex`, `cardinals`, wider `Digits`).
- Verify: `cargo test -p offcut-text` green.

### Why This Comes Here

The detector (Prompts 75–77) reads quantities through `parse_quantity` and spans through `normalize_transcript`; the scene's count-up formats with `format_quantity`.

### Scope

#### Implement

- `src/numbers.rs`: `parse_quantity(tokens) -> Option<(usize, Quantity)>` in four stages (§8.3: core — `Digits` with symbol/suffix, or cardinal + `point_fraction`, or "a/an" + hundred/magnitude, V2's split groups D-42 after a plain core; magnitude word; "grand" → ×1,000 and USD unless a currency is set; one unit from the core's own symbol/sign or the next one or two tokens, with "times" only before a `COMPARATIVES` word consuming "times" only, D-102, and a same-currency word after a symbol consumed). Suffix values k/m/b; a suffix sets no unit. `None` for `$3x`, `%`+suffix, ranges, clock times, Indian grouping, fractions in words, "dozen", Roman numerals. `format_quantity(value, &unit)` by the table of §8.5, negative → `-` + form of the absolute value, non-finite → `""`. **Rewrite exactly one V2 inline test**: "ten million" parses as 10,000,000 using 2 tokens (§8.6).
- `src/sentences.rs`: V2's rule, then while a sentence has more than `SENTENCE_MAX_WORDS` (40) words, split at its longest internal gap, the earliest of equal gaps, recursively; every word in exactly one sentence, in order. (The tie-break's one-word sentences are §23 item 28, open; implement the plan's rule unless the founder decided otherwise, recorded in Entry 1.)
- `src/normalize.rs`: V2's five steps with step 4 per sentence (tokenize each sentence's words, scan with `parse_quantity`); `Transcript.numbers` in word order.
- The inline cases of §8.7 that concern these files.

#### Do Not Implement Yet

`numbers_table.rs`, `numbers_prop.rs`, `tests/sentences.rs` (Prompt 74). Any detector change.

### Existing Components To Reuse

`cardinals`, `units_lex`, `tokenize`, V2's `Quantity`, `Unit`, `SENTENCE_GAP`, `SENTENCE_MAX_WORDS`, `RawWord`.

### Files To Create

None.

### Files To Modify

- `crates/offcut-text/src/numbers.rs`, `sentences.rs`, `normalize.rs`.

### Architecture Requirements

- `numbers.rs` is the only file with `parse_quantity` and `format_quantity` [ONLY] and stays under 400 lines above tests.
- `normalize_transcript` never mutates a `Transcript` after building it (TS §10.8). Integers up to 10^15 are exact in `f64`.

### Data Types / Interfaces

Signatures unchanged (V2 §8, frozen): `parse_quantity(tokens: &[Token]) -> Option<(usize, Quantity)>`; `format_quantity(value: f64, unit: &Unit) -> String`; `segment_sentences`; `normalize_transcript`.

### Implementation Requirements

#### 1. Display
Exactly §8.5: currency whole thousands → `k`; millions/billions/trillions → `m`/`b`/`t` up to 2 decimals; `None` ≥ 1,000,000 → "2 million"; time units singular for exactly 1; `Count { noun }` with the noun as spoken.

#### 2. Sentence split cases (§8.7)
45 words with a 900 ms gap after word 30 → V2's gap rule splits (30/15); at 650 ms → the 40-word rule splits after word 30; 90 equal gaps → 50 one-word sentences and one of 40; the `$12 / ,000` boundary case → one number `$12`.

### Error Handling

`None` for anything not read; never a panic on any token sequence (Prompt 74's property test checks it).

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

The grammar and display table are frozen from Prompt 74 (§21.1).

### Security Requirements

None.

### Testing Requirements

#### Unit Tests
§8.7: `format_quantity(-5.0, None)` → `-5`; NaN → `""`; the three split cases; the `$12 / ,000` case; the rewritten "ten million" case.

#### Integration Tests
V2's crate tests otherwise unchanged; `pnpm build:wasm` (the text binding rebuilds).

#### Failure-Path Tests
G 6.7 row 3: make `normalize.rs` scan the whole transcript → the `$12 / ,000` case fails; undo.

### Validation Commands

```bash
git diff "$BASE" -- crates/offcut-text/src | grep -cE '^-\s*assert'      # 1 (only the "ten million" expectation removed)
cargo test -p offcut-text --lib
awk '/#\[cfg\(test\)\]/{print NR-1; f=1; exit} END{if(!f) print NR}' crates/offcut-text/src/numbers.rs   # under 400
pnpm build:wasm
```

### Acceptance Criteria

- [ ] Exactly one V2 assertion removed (the "ten million" one), rewritten to the V3 value.
- [ ] The §8.7 cases pass; `numbers.rs` under 400 lines.
- [ ] `pnpm build:wasm` succeeds; the SAB gate passes (media D-91 cases may now differ, SAB note 2).

### Agent Instructions

Before: read §8.3, §8.5, §8.6, §8.7, D-42, D-101, D-102, D-104, TS §17.2, V2 §8. During: keep `numbers.rs` the only formatter; if it nears 400 lines, move private helpers into `cardinals.rs` only if they are cardinal logic, otherwise stop and report. After: SAB gate; commit `V3 P73: full number grammar, display rules, sentence split`.

### Expected Result

Any recognised number form becomes one correctly displayed quantity inside its sentence.

---

## Prompt 74 (V3 14/30) — `numbers_table.rs`, `numbers_prop.rs`, `tests/sentences.rs`

**Effort: Medium.** Mostly transcribing 72 rows and 5 properties onto a grammar already built at high effort. Switch to High if more than a couple of rows fail, since the fixes go into the grammar.

### Objective

Make the grammar's authority executable: 62 parse rows and 10 display rows, 5 properties, 8 sentence cases; close Milestone 6 (G 6.6, 6.7, M-6; §19.8–§19.10).

### Project State Before This Prompt

- Prompt 73 committed.
- Verify: `cargo test -p offcut-text` green; `numbers.rs` under 400 lines.

### Why This Comes Here

The detector must build on a grammar whose every row is pinned; any failing row is fixed in `numbers.rs` now, not discovered through detection.

### Scope

#### Implement

- `tests/numbers_table.rs`: the 62 parse rows of §19.8 (words split on whitespace and tokenized by `tokenize`, tokens consumed, value, unit, display) and the 10 display-only rows, as two data tables, one test each, printing the failing row.
- `tests/numbers_prop.rs`: the 5 properties of §19.9 (digits round trip 0–10^12; currency round trip 1–10^9 with the exact/within-0.5% rule; spelled round trip 0–999,999,999 by a test-only speller; no panic on 30-token random sequences; display is the formatter's).
- `tests/sentences.rs`: the 8 cases of §19.10.
- G 6.7 row 2 drill.

#### Do Not Implement Yet

Anything in `offcut-detect`.

### Existing Components To Reuse

`tokenize`, `parse_quantity`, `format_quantity`, `segment_sentences` (public); `proptest` dev-dependency.

### Files To Create

- `crates/offcut-text/tests/numbers_table.rs`, `crates/offcut-text/tests/numbers_prop.rs`, `crates/offcut-text/tests/sentences.rs`.

### Files To Modify

`crates/offcut-text/src/*.rs` only to fix a failing row (never the row).

### Architecture Requirements

Tests use only the public surface (`cardinal` is private: the spelled round trip goes through `parse_quantity` of the spelled words, which consumes every word).

### Data Types / Interfaces

Rows as tuples `(&str, Option<(usize, f64, &str /*unit*/, &str /*display*/)>)` on one line each (G 6.6 counts 72 lines starting `("`).

### Implementation Requirements

#### 1. Row fidelity
Every row exactly as §19.8, including `$12` `,000` (3 tokens, D-42), `$500 euros` (1 token, USD), `ten pounds` (1 token, none), `2 TB` (1 token, none), `3 reasons` (1 token, none).

#### 2. Failures
A failing row is a bug of `numbers.rs`/`tokenize.rs`/`cardinals.rs`; fix it there and record it.

### Error Handling

N/A (tests).

### Concurrency / Async Requirements

None.

### Persistence Requirements

None. Delete any `proptest-regressions/` folder after fixing a found case (V2 CL known issue 12).

### API Requirements

None.

### Security Requirements

None.

### Testing Requirements

#### Unit Tests
None new.

#### Integration Tests
72 rows, 5 properties, 8 cases.

#### Failure-Path Tests
G 6.7 row 2: add `"pounds"` to `CURRENCY_WORDS` (sorted) → the `ten pounds` row fails; undo.

### Validation Commands

```bash
grep -cE '^\s*\("' crates/offcut-text/tests/numbers_table.rs        # 72
cargo test -p offcut-text 2>&1 | awk '/^test result:/ {p+=$4; f+=$6} END {print p" passed, "f" failed"}'   # 0 failed
cargo clippy -p offcut-text --all-targets -- -D warnings
pnpm build:wasm && pnpm check
```

### Acceptance Criteria

- [ ] 72 table rows, 5 properties, 8 sentence cases pass; no row was edited to pass.
- [ ] The drill fired and was undone; no `proptest-regressions/` folder remains; G M-6 passes.

### Agent Instructions

Before: read §19.8–§19.10, G 6.6–6.7. During: copy rows exactly. After: SAB gate; commit `V3 P74: number table, properties and sentence tests`.

### Expected Result

`offcut-text` is final for V3 (§21.2: V4–V7 add nothing).

---

## Prompt 75 (V3 15/30) — `offcut-detect`: configuration, lexicons, and NumberReveal V3

**Effort: High.** 29 new config fields and 11 lexicons, and NumberReveal rewritten: effective tokens with edits, `parts`, scoring, label derivation (D-107). P76-P79 build on it.

### Objective

Give the detector every threshold, scoring constant and lexicon (`config.rs`, D-84), and rewrite NumberReveal to read quantities from effective tokens per sentence (D-109), carry `parts` (D-106) and derive labels (D-107) — confirming first that the labels were frozen before any rule (G 7.1–7.3 row 1; §9.1, §9.3).

### Project State Before This Prompt

- Prompts 65 and 74 committed: 31 labeled files frozen; `offcut-text` final.
- `offcut-detect/src` holds V2's `lib.rs`, `config.rs` (10 fields), `number.rs`, `event_id.rs`.
- Verify (G 7.1): `git log --format=%ct -1 -- fixtures/labeled` exists; `git ls-files crates/offcut-detect/src | grep -E 'from_to|list|keyword|exclusions|resolve'` prints nothing; `git status --short fixtures/labeled` prints nothing. Write "labels frozen at <SHA of P65's commit> on <date>" in this prompt's changelog entry.

### Why This Comes Here

Every finder reads `DetectorConfig`; NumberReveal's `Found` and `Candidate.parts` are what the other finders and the exclusions use.

### Scope

#### Implement

- `src/config.rs` [ONLY thresholds and lexicons]: V2's 10 fields unchanged (names and values); the 29 V3 fields of §9.1 appended with the defaults of its table; `threshold(kind)` and `hold_ms(kind)` (number 1,400; from-to 1,700; list 1,700; keyword 0); the 11 lexicons of §9.1 as sorted `const` slices (`ORDINAL_MARKERS` sorted by word: fifth, first, firstly, fourth, second, secondly, seventh, sixth, third, thirdly). If the founder chose rounding for §23 item 27, add the one named rounding rule's constant here (`SCORE_DECIMALS = 2`) and record.
- `src/number.rs` (§9.3): `Candidate` gains `parts: Vec<WordRange>`; `Found { words, tokens, quantity, sentence, suffixed }`; `quantities(toks, t)` per sentence with `parse_quantity` over effective tokens (an edited span yields the edited text's quantity; a hidden word yields none); `find` keeps V2's signature and scores base 0.50 + unit 0.30 (currency, percent, times, time, data, or `Count` ≥ `COUNT_UNIT_MIN`) + magnitude 0.20 (≥ 1,000 or `suffixed`) + energy 0.10 (`energy_z ≥ 1.0` on any span word), clamp 1.0, × minimum word confidence; `params = NumberReveal { value, label }` with D-107's label rule (up to `number_label_max_words` Word tokens after the quantity in the sentence, stopping at `Punct` or `LABEL_STOP_VERBS`).
- **Rewrite V2's "edited number" inline test** (§9.9): an edit of `$10,000` to `$20,000` gives an event `$20k` with a new id; the hidden half still gives none.
- Inline tests: §9.9 rows for config (defaults, thresholds, holds, lexicons sorted/unique/lower-case) and `quantities` with an edit `3000` → `4000`; the Script A label rows of §9.3 through `number::find`.

#### Do Not Implement Yet

`list.rs`, `from_to.rs`, `keyword.rs` (Prompt 76); `exclusions.rs`, `resolve.rs`, the full `detect`, `rules_table.rs` (Prompt 77). V2's `lib.rs::detect` keeps working with the V3 `number::find` until Prompt 77 replaces it. **Deviation from G 7.3, recorded:** `rules_table.rs` is written whole in Prompt 77, because rows that need exclusions or resolution cannot pass at this commit and a failing row cannot be committed (SAB rule 3).

### Existing Components To Reuse

`offcut_text::{tokenize, parse_quantity, units_lex::COUNT_UNIT_MIN}`; V2's `event_id`, `WordRange`, `Span`, `EventParams`; V2's `config.rs` field values.

### Files To Create

None.

### Files To Modify

- `crates/offcut-detect/src/config.rs`, `crates/offcut-detect/src/number.rs`, `crates/offcut-detect/src/lib.rs` (only what compiles the new `Candidate` field; V2's `detect` otherwise unchanged).

### Architecture Requirements

- Pure; no `HashMap`; every threshold and lexicon in `config.rs` only (§9 "Never": no threshold outside `config.rs`).
- `format_quantity` is called only through `offcut-text`'s parser here (and once in `from_to.rs` later, D-106).
- `DetectorConfig` field names are frozen from here (§21.1); values may be tuned by the gate only.

### Data Types / Interfaces

```rust
pub struct Candidate { pub kind: EventKind, pub anchors: WordRange, pub span: Span, pub score: f32,
                       pub params: EventParams, pub parts: Vec<WordRange> }
pub(crate) struct Found { pub words: WordRange, pub tokens: Range<usize>, pub quantity: Quantity,
                          pub sentence: SentenceIdx, pub suffixed: bool }
pub(crate) fn quantities(toks: &[Token], t: &Transcript) -> Vec<Found>;
pub fn find(toks: &[Token], t: &Transcript, p: &Prosody, cfg: &DetectorConfig) -> Vec<Candidate>;
impl DetectorConfig { pub fn threshold(&self, kind: EventKind) -> f32; pub fn hold_ms(&self, kind: EventKind) -> u32; }
```

### Implementation Requirements

#### 1. Field count
39 `pub` fields; 11 `pub const …: &[` lexicons.

#### 2. Labels
"I had spent 3000 hours editing by hand." → `editing by hand`; "…earned $12 ,000 in sales." → `in sales`; "They grew to 10,000 subscribers, which was wild." → `None`; "That is 40% of all views." → `of all views`; "It hit $2m and stopped." → `and stopped`.

### Error Handling

`Confidence::new(score)` failure is impossible for a clamped finite score; propagate with `?` where the type requires it, never `unwrap`.

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

`find`'s signature unchanged (V2).

### Security Requirements

None.

### Testing Requirements

#### Unit Tests
Config (defaults; four thresholds; holds 1,400/1,700/1,700/0; 11 lexicons sorted, unique, lower-case); `quantities` with an edit; the five label rows; the rewritten V2 edited-number test.

#### Integration Tests
None yet in `tests/` for this crate.

#### Failure-Path Tests
Swap two words of `ORDINAL_MARKERS` → the sorted test fails; undo.

### Validation Commands

```bash
grep -cE '^\s+pub [a-z_]+:' crates/offcut-detect/src/config.rs          # 39
grep -cE '^pub const [A-Z_]+: &\[' crates/offcut-detect/src/config.rs   # 11
cargo test -p offcut-detect --lib
cargo clippy -p offcut-detect --all-targets -- -D warnings
git status --short fixtures/labeled                                     # no output
```

### Acceptance Criteria

- [ ] The label-freeze confirmation of G 7.1 is in the changelog with the freeze SHA.
- [ ] 39 fields, 11 lexicons, `threshold`/`hold_ms`; V2's ten values unchanged.
- [ ] NumberReveal reads effective tokens per sentence, carries `parts`, derives labels; exactly one V2 detect test rewritten as §9.9 says.
- [ ] No label file changed.

### Agent Instructions

Before: read §9 (crate rules, "Never"), §9.1, §9.3, §9.9, D-84, D-101, D-106, D-107, D-109, G 7.1–7.3. Do **not** open `fixtures/labeled/*.json` while writing rules (independence of D-87). During: one file at a time. After: SAB gate; commit `V3 P75: detector configuration, lexicons and NumberReveal V3`.

### Expected Result

The detector's single home of numbers and words exists, and NumberReveal is V3's.

---

## Prompt 76 (V3 16/30) — `offcut-detect`: ListReveal, FromTo, KeywordPop finders

**Effort: High.** Three finders with exact scoring formulas, list windows, unit-inheritance compatibility and marker rules; scores must match the plan's tables to the hundredth.

### Objective

Write the three new finders exactly as §9.4–§9.6 define them, each with inline tests from its table (G 7.3 rows 2–4; D-85, D-105, D-106, D-107).

### Project State Before This Prompt

- Prompt 75 committed: `config.rs` complete, `Candidate.parts`, `Found`, `quantities`.
- Verify: `cargo test -p offcut-detect --lib` green.

### Why This Comes Here

`exclusions.rs` reads `list::markers` (X6) and every candidate's `parts`; `resolve.rs` orders candidates of all four kinds (Prompt 77).

### Scope

#### Implement

- `src/list.rs` (§9.5, D-105, D-107): `pub(crate) struct Marker { ordinal, words, ordinal_word, at, sentence }`; `pub(crate) fn markers(toks, t)` — the first effective Word token of a sentence after any `FILLERS` that is an `ORDINAL_MARKERS` word, or "number" + a cardinal/digits 1–7, or a cardinal/digits 1–7 directly followed by `,` `.` `:`; `find`: announcer form (integral `Found` 2–7, `Unit::None`, followed by a `LIST_NOUNS` word; markers 1..count in order in later sentences, each before announcer start + `list_window_ms`; any missing → no candidate; score 0.60 + 0.30 + 0.10 when every marker is an ordinal word; header `"{count} {NOUN}"` upper-cased) and marker-only form (≥ 3 consecutive ordinal-word markers starting at `first`, not used by an announcer, each in a later sentence, within the window; count ≤ 7; 0.65 + 0.20 when no gap exceeds `list_marker_gap_ms`; header `""`); items (up to `list_item_max_words` Word tokens after the marker, leading fillers dropped, stopping at `. ? ! ; :`, at `,` once two words are taken, at the sentence end); anchors, span, parts as §9.5.
- `src/from_to.rs` (§9.4, D-85, D-106): per sentence, each ordered pair (a before b), gap ≤ `from_to_max_gap_words`, values differ, units compatible after inheritance (same variant; time group; data group; identical count noun; different currencies incompatible); from-frame (+0.25: "from" before a, "to" before b) or else change-verb frame (+0.15: a `CHANGE_VERBS` word among the 3 tokens before a or between, and "to" before b); same explicit unit +0.15 or inherit +0.05; close (gap ≤ 6) +0.05; base 0.55; clamp; × min confidence; best b per a (score, then nearest); `params = FromTo { from, to, label: None }` with inherited units and `display` re-formatted by `format_quantity`.
- `src/keyword.rs` (§9.6): one candidate per effective Word token with ≥ 4 letters, not a stop word, not in a `Found`, not a marker word; score 0.50 + 0.20·clamp(energy_z−1) + 0.20·clamp(pitch_z−1) + 0.20 when the previous token in the sentence is an `INTENSIFIERS` word or the word is in `EMPHASIS_WORDS`; clamp; × confidence; `parts = []`.
- `src/lib.rs`: `mod list; mod from_to; mod keyword;` (crate-internal) — V2's `detect` still calls only `number::find`.

#### Do Not Implement Yet

Exclusions, resolution, the full `detect`, `rules_table.rs` (Prompt 77).

### Existing Components To Reuse

`number::{Candidate, Found, quantities}`, `config.rs` lexicons and fields, `offcut_text::{format_quantity, units_lex::COMPARATIVES}` (COMPARATIVES is used by the parser; finders read the quantity's unit only).

### Files To Create

- `crates/offcut-detect/src/list.rs`, `from_to.rs`, `keyword.rs`.

### Files To Modify

- `crates/offcut-detect/src/lib.rs`.

### Architecture Requirements

Pure; no `HashMap`; no threshold or word outside `config.rs`; `from_to.rs` is the one place outside `offcut-text` that calls `format_quantity` (D-106). Each file under 400 lines.

### Data Types / Interfaces

```rust
pub fn find(toks: &[Token], t: &Transcript, p: &Prosody, cfg: &DetectorConfig) -> Vec<Candidate>;   // in each finder
pub(crate) fn markers(toks: &[Token], t: &Transcript) -> Vec<Marker>;
```

### Implementation Requirements

#### 1. List cases (inline, §9.5 table)
Script A → one candidate 1.00, header `3 REASONS`, three items; no third → none; "First, open the app…" → marker-only 0.85; "It was the first time I tried." → no marker; "I have 3 tips. 1, sleep…" → announcer, digit markers, 0.90; "eight reasons" → none.

#### 2. FromTo cases (inline, §9.4 table)
The seven rows with their scores (exclusion-dependent rows assert the pre-exclusion score only: "from 2019 to 2024" 1.00, "from 3 to 5 people" 0.90).

#### 3. Keyword cases (inline, §9.6 table)
The five rows.

### Error Handling

None (finders return candidates; no failure mode).

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

None public beyond `find`.

### Security Requirements

None.

### Testing Requirements

#### Unit Tests
The three tables above.

#### Integration Tests
None yet.

#### Failure-Path Tests
Break the "first word of a sentence" rule (allow mid-sentence markers) → the "first time" case fails; undo.

### Validation Commands

```bash
cargo test -p offcut-detect --lib
cargo clippy -p offcut-detect --all-targets -- -D warnings
for f in crates/offcut-detect/src/*.rs; do printf '%s ' "$f"; awk '/#\[cfg\(test\)\]/{print NR-1; f=1; exit} END{if(!f) print NR}' "$f"; done
```

### Acceptance Criteria

- [ ] Every row of §9.4, §9.5, §9.6's tables passes as an inline test.
- [ ] `markers` is `pub(crate)`; items stop at `,` only after two words.
- [ ] Each detector file under 400 lines; no label file changed.

### Agent Instructions

Before: read §9.4–§9.6, D-85, D-105, D-106, D-107, TS §17.4. Do not open the labeled files. During: implement the rules as written, not as the labels might like. After: SAB gate; commit `V3 P76: ListReveal, FromTo and KeywordPop finders`.

### Expected Result

All four kinds produce scored candidates; nothing is excluded or resolved yet.

---

## Prompt 77 (V3 17/30) — Exclusions, resolution, the full `detect`, `rules_table.rs`

**Effort: High.** Exclusions X1-X7, total-order resolution with display windows and caps, the full `detect` and 34 rule rows under INV-6/INV-7. The detector's correctness lives here.

### Objective

Complete the detection pipeline of TS §17.3 (finders → exclusions → threshold → resolve → events) and pin it with the 34 rows of `rules_table.rs` (G 7.3 rows 5, 7.4–7.6, M-7; §9.2, §9.7, §9.8, §19.11).

### Project State Before This Prompt

- Prompt 76 committed.
- Verify: the four finders' inline tests pass.

### Why This Comes Here

Exclusions need every finder's `parts`; resolution needs every kind; the table needs the full `detect`.

### Scope

#### Implement

- `src/exclusions.rs` (§9.7): `apply(cands, toks, t, edits)` sets `score = 0.0` for X1 (year 1900–2099 as one `Digits` token, `Unit::None`), X2 (`VERSION_CONTEXT` before, `VERSION_AFTER` after, or two decimal points), X3 (`CLOCK_WORDS` after), X4 (one `Digits` token of ≥ 7 digits without separator, `Unit::None`), X5 (`Unit::None` before inheritance and |value| < 10) — on NumberReveal and FromTo parts; X6 (a NumberReveal part equals a marker's words) — NumberReveal; X7 (an `edits` entry `""` inside the anchors) — every kind. The ListReveal announcer count is exempt from X5.
- `src/resolve.rs` (§9.8, Appendix C): `resolve(cands, fixed, cfg) -> Vec<Candidate>` — total order (priority FromTo > ListReveal > NumberReveal > KeywordPop; score; earlier `span.start`; lower `anchors.start`); drop on anchor intersection with kept or fixed; overlay display windows `[start - overlay_lead_ms, end + hold_ms(kind)]` overlapping or touching (V2 as built) → drop; at most `overlay_max_per_window` overlay windows starting in any `overlay_window_ms` (drop lowest score, the later on a tie; fixed count and are never dropped); KeywordPop one per sentence and `keyword_min_gap_ms` apart with the higher score winning; returns kept candidates only.
- `src/lib.rs` (§9.2): `detect` in seven steps (tokenize with edits; `number::quantities`; candidates in order from_to, list, number, keyword; `exclusions::apply`; `retain(score >= threshold(kind))` — or, if the founder chose rounding for item 27, compare after rounding to `SCORE_DECIMALS`, in this one place; `resolve(cands, &[], cfg)`; events with `event_id(kind, anchors)`, sorted by `span.start` then kind order). `pub fn redetect_sentence` is **not** added yet (Prompt 78).
- `tests/rules_table.rs` (§19.11): the 34 rows with a 40-line timing-synthesis helper (§17.5's rule) copied in; prosody flat unless the row says (emphasis by the founder's item-27 values); each row asserts events by kind and display (or none), INV-6 for every event. Row "*Workflow* matters." asserts the event exists and `confidence >= threshold` (not a fixed value) until item 27's choice makes the score unambiguous; record which.
- G 7.6 drills; M-7.

#### Do Not Implement Yet

`redetect_sentence`, `redetect.rs`, `determinism.rs` (Prompt 78); `precision_recall.rs` and the binding (Prompt 79).

### Existing Components To Reuse

All finders; `list::markers`; `event_id` (V2, frozen bytes D-47); `config.rs`.

### Files To Create

- `crates/offcut-detect/src/exclusions.rs`, `crates/offcut-detect/src/resolve.rs`, `crates/offcut-detect/tests/rules_table.rs`.

### Files To Modify

- `crates/offcut-detect/src/lib.rs`.

### Architecture Requirements

- **INV-6:** no `DetectedEvent` below its kind's threshold is constructed; `resolve` never raises a score or changes an anchor.
- **INV-7:** every sort stable with a total order; equal inputs give equal outputs.
- `exclusions.rs` is the only file that reads `edits` directly (X7).

### Data Types / Interfaces

```rust
pub fn apply(cands: &mut [Candidate], toks: &[Token], t: &Transcript, edits: &BTreeMap<WordIdx, String>);
pub fn resolve(cands: Vec<Candidate>, fixed: &[DetectedEvent], cfg: &DetectorConfig) -> Vec<Candidate>;
pub fn detect(t: &Transcript, p: &Prosody, edits: &BTreeMap<WordIdx, String>, cfg: &DetectorConfig) -> Vec<DetectedEvent>;
```

### Implementation Requirements

#### 1. Rows
Exactly §19.11's 34 rows, including: "$2k to $20k." → no FromTo, `$2k` kept and `$20k` dropped (overlapping windows); "Costs dropped 40% to 25%." → FromTo 0.90; "I have 3 tools." → none (X5); "Four NumberReveals with currency inside 9 s, 2.5 s apart" → at most three; "First, we saved $10,000." → list kept, number dropped; edit `$12,000` → `$20,000` → `$20k` with the same id as before (open item 14, decided 2026-10-18); hide "Second," → no list.

#### 2. Guards (G 7.6)
Comment out X1 → row 11 fails; let touching windows pass → row 10 fails (both kept); `threshold_list = 0.60` → row 17 fails. Undo all.

### Error Handling

`detect` never fails: an empty transcript gives no event.

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

`detect`'s signature unchanged (TS §17.1).

### Security Requirements

None.

### Testing Requirements

#### Unit Tests
§9.9's remaining rows (X7 on a hidden list-item word; `resolve` with a fixed NumberReveal at 10 s drops a new one at 10.5 s; `markers` of "First, … It was the first time …" gives one).

#### Integration Tests
`rules_table.rs`: 34 rows.

#### Failure-Path Tests
G 7.6's three drills.

### Validation Commands

```bash
cargo test -p offcut-detect --lib --test rules_table -- --nocapture
cargo clippy -p offcut-detect --all-targets -- -D warnings
for f in crates/offcut-detect/src/*.rs; do printf '%s ' "$f"; awk '/#\[cfg\(test\)\]/{print NR-1; f=1; exit} END{if(!f) print NR}' "$f"; done   # each under 400
git status --short fixtures/labeled                                       # no output
pnpm build:wasm && pnpm check
```

### Acceptance Criteria

- [ ] All 34 rows pass (one test printing none failed, or 34 tests); every inline test of the crate passes.
- [ ] The three drills fired and were undone; no label changed; each source file under 400 lines.
- [ ] The rounding decision of item 27, if taken, lives in one place (`lib.rs` step 5) with its constant in `config.rs`.

### Agent Instructions

Before: read §9.2, §9.7, §9.8, §9.9, §19.11, Appendix C (`resolve.rs`), TS §17.3–§17.6. A failing row is fixed in the rule, never by editing the row without a changelog line naming the spec section that decides it. After: SAB gate; commit `V3 P77: exclusions, resolution, full detect, rules table`.

### Expected Result

`detect` gives V3's events for any transcript, deterministically and never below threshold.

---

## Prompt 78 (V3 18/30) — `redetect_sentence`, `redetect.rs`, `determinism.rs`

**Effort: High.** Re-detection scoped by time that keeps unchanged ids, plus a proptest determinism harness over random transcripts.

### Objective

Re-detect one sentence's scope after an edit while keeping every unchanged event's id, and prove detection deterministic over the labeled set and random transcripts (G 8.1, 8.2; §9.2, D-86; §19.13, §19.14).

### Project State Before This Prompt

- Prompt 77 committed. `a01-scriptA.json` exists (its sentences are Script A as recognized).
- Verify: `cargo test -p offcut-detect` green.

### Why This Comes Here

V4's review flow calls `redetect_sentence` (§21.4); it must exist and be tested before the binding (Prompt 79).

### Scope

#### Implement

- `src/lib.rs`: `redetect_sentence(t, p, edits, s, prev, cfg)` — scope = `s`, every sentence intersected by the anchors of a `prev` event that intersects `s`, every sentence whose `start_ms` lies in `(start_ms(s), start_ms(s) + list_window_ms)`; `outside` = `prev` events wholly outside; steps 1–5 of `detect` over the whole transcript, keep candidates wholly inside scope; `resolve(cands, &outside, cfg)`; result = `outside` + kept as events, sorted as `detect`; `s` out of range → `prev` unchanged.
- `tests/redetect.rs` (§19.13): 7 cases on Script A as recognized (the sentences of `a01-scriptA.json` with synthesized timing; the word indices 23 = `3000`, 79 = `First`, sentences 3, 8, 9–11 are the README's): equal to `prev` without edits; edit 23 → 4,000 with the same id (open item 14, decided 2026-10-18), others unchanged; hide 79 → list gone; edit in the announcer sentence reaches 9–11; a new currency in sentence 1 overlapping the `3,000 hours` window is dropped; out of range; timing printed in release, not asserted.
- `tests/determinism.rs` (§19.14): 3 cases (every labeled file twice; proptest random transcripts of 0–200 words from a mixed vocabulary with random prosody ±3, checking equality, anchors in range, sorted output, no overlapping overlay windows, ≤ 1 KeywordPop per sentence, INV-6; same raw words built twice). Labeled files with null data (`a02`) are skipped.

#### Do Not Implement Yet

`precision_recall.rs`, the binding, `RenderApi` (Prompt 79).

### Existing Components To Reuse

`detect`'s steps, `resolve` with `fixed`, the timing-synthesis helper (copy the 40 lines from `rules_table.rs`), `serde_json` (dev) to read labeled files.

### Files To Create

- `crates/offcut-detect/tests/redetect.rs`, `crates/offcut-detect/tests/determinism.rs`.

### Files To Modify

- `crates/offcut-detect/src/lib.rs`.

### Architecture Requirements

An event whose kind and anchors did not change keeps its id (D-47). Budget 50 ms per call for 150 words (assumption M1.5), printed only.

### Data Types / Interfaces

```rust
pub fn redetect_sentence(t: &Transcript, p: &Prosody, edits: &BTreeMap<WordIdx, String>, s: SentenceIdx,
                         prev: &[DetectedEvent], cfg: &DetectorConfig) -> Vec<DetectedEvent>;
```

### Implementation Requirements

#### 1. Scope by time
`LIST_WINDOW` is 45 s of time, not a sentence count (D-86).

#### 2. Determinism vocabulary
Number words, list nouns, ordinals, fillers, stop words, emphasis words, digits (G 8.2).

### Error Handling

Out-of-range `s` returns `prev` unchanged; no panic on any transcript.

### Concurrency / Async Requirements

None.

### Persistence Requirements

Delete `proptest-regressions/` after fixing any found case.

### API Requirements

Signature frozen from here (§21.1).

### Security Requirements

None.

### Testing Requirements

#### Unit Tests
None new.

#### Integration Tests
7 + 3 cases.

#### Failure-Path Tests
G 8.5 row 2: make `resolve` sort by score only → `determinism.rs` or an order test fails; undo.

### Validation Commands

```bash
cargo test -p offcut-detect --release --test redetect -- --nocapture       # 7 passed; record µs per call
cargo test -p offcut-detect --test determinism                             # 3 passed
cargo clippy -p offcut-detect --all-targets -- -D warnings
```

### Acceptance Criteria

- [ ] 7 redetect cases and 3 determinism cases pass; the per-call time is in the changelog.
- [ ] The drill fired and was undone; no `proptest-regressions/` folder remains.

### Agent Instructions

Before: read §9.2 (`redetect_sentence` table), D-86, D-47, §19.13, §19.14, G 8.1–8.2. After: SAB gate; commit `V3 P78: redetect_sentence and determinism`.

### Expected Result

The detector is complete in Rust, including re-detection for V4.

---

## Prompt 79 (V3 19/30) — The precision gate **[gate]**, the `redetect_sentence` binding, `RenderApi`

**Effort: High.** The precision gate: IoU and unit matching, the coverage check, the threshold sweep, and the pre-agreed actions if precision is under 0.90. Also the render binding.

### Objective

Measure precision over the labeled set (≥ 0.90 over ≥ 30 read files, BP §5.2, PS §20.1), apply §20.3 if needed, record the result, and export `redetect_sentence` through `offcut-wasm-render` and `RenderApi` (G 8.3–8.5, M-8; §10.2, §13.3, §19.12, §20.3).

### Project State Before This Prompt

- Prompt 78 committed. 31 labeled files; `a02` has null data.
- Verify: `cargo test -p offcut-detect` green; `git status --short fixtures/labeled` empty.

### Why This Comes Here

The gate decides the thresholds every later step ships; the binding completes S7 so the render bundle carries V3's detector before the pipeline uses it (Prompt 81).

### Scope

#### Implement

- `tests/precision_recall.rs` (§19.12): read every `fixtures/labeled/*.json` with `serde_json` and `#[serde(deny_unknown_fields)]` (optional `tags`, `edits`); skip a `source: "asr"` file whose `words` is null and print `skipped (no data yet)`; turn written files into `RawWord`s and prosody by §17.5 (word durations `clamp(120 + 45 x letters, 150, 700)` ms, 80 ms between words, 400 ms after `. ? !`, first word at 500 ms, confidence 1.0; `*word*` → the item-27 emphasis values); asr files as they are; `normalize_transcript(raw, "labeled")`; apply `edits`; map SCHEMA's unit strings to `Unit`; the coverage check of §17.5 (from `tags` and `expected`), failing with the missing item named; `detect` with the default config; greedy matching per file and kind (IoU ≥ 0.5, equal values and units); print per kind and overall TP/FP/FN/precision/recall, the sweep 0.50–1.00 by 0.05 per kind, and every false positive as `file: kind "say"`; assert overall precision ≥ `DETECTOR_PRECISION_TARGET` (0.90, `limits.rs`), INV-6 for every prediction, ≥ 30 files read.
- Run the gate (G 8.3): ≥ 0.90 → copy the table, sweep and false positives into `experiments.md` under `Precision gate` ("default thresholds"). Under 0.90 → for each kind under 0.90 raise its threshold in `config.rs` to the lowest sweep value reaching 0.90, re-run, record old/new thresholds and every recall; a kind under 0.90 at every threshold → threshold 1.01 (off, INV-6), recorded, V4 draws three kinds (§20.4); recall under 0.30 after a raise → keep the kind, record as E-7 input. **Never edit a label to pass**; a label a person would not expect is fixed only with a changelog line (D-87).
- `crates/offcut-wasm-render/src/detect_api.rs` (§10.2): `redetect_sentence(transcript, prosody, edits, sentence: u32, prev)` with details `Transcript`/`Prosody`/`Edits`/`Events` on decode failure (`E_INTERNAL`), `DetectorConfig::default()`.
- `web/src/wasm/load-render.ts` (§13.3): `RenderApi.redetectSentence(transcript, prosody, edits, sentence, prev)`. Nothing calls it in V3; `render.worker.ts`'s V2 line `redetectSentence: notYet,` stays.
- G 8.5 drills; M-8.

#### Do Not Implement Yet

The render worker handler (V4). `a02`'s data (Prompt 88).

### Existing Components To Reuse

`detect`, `normalize_transcript`, the timing helper, `limits.rs::DETECTOR_PRECISION_TARGET` (verify the name with `git grep`; if absent, stop and report — do not add a second home), `detect_api.rs`'s existing decode helpers.

### Files To Create

- `crates/offcut-detect/tests/precision_recall.rs`.

### Files To Modify

- `crates/offcut-wasm-render/src/detect_api.rs`, `web/src/wasm/load-render.ts`, `crates/offcut-detect/src/config.rs` (thresholds, only if the gate requires), `docs/v3/experiments.md`, `docs/v3/v3changelog.md`.

### Architecture Requirements

- The test reads committed files only; it runs everywhere in under a second (§19.1).
- Thresholds changed here are copied into TS §17.4 in Prompt 89; note them in the changelog now.

### Data Types / Interfaces

```rust
#[wasm_bindgen] pub fn redetect_sentence(transcript: JsValue, prosody: JsValue, edits: JsValue,
                                        sentence: u32, prev: JsValue) -> Result<JsValue, JsValue>;
```
```ts
redetectSentence(transcript: Transcript, prosody: Prosody, edits: Record<number, string>,
                 sentence: SentenceIdx, prev: readonly DetectedEvent[]): DetectedEvent[];
```

### Implementation Requirements

#### 1. Gate output
`30 files read, 1 skipped` today (31 after Prompt 88).

#### 2. Binding
No product rule in the binding; `SentenceIdx::new(sentence)`.

### Error Handling

A coverage shortfall fails with the missing item named; a labeled file that fails to parse fails the test with its file name (never silently skipped).

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

`RenderApi.redetectSentence` is frozen from here; V4 adds the worker handler.

### Security Requirements

The test prints `say` text of written files only (agent-written); `a02` prints nothing while skipped.

### Testing Requirements

#### Unit Tests
None new.

#### Integration Tests
`precision_recall.rs` (one test, three assertions).

#### Failure-Path Tests
G 8.5: `threshold_number = 0.50` → precision falls below 0.90 and lists new false positives; edit one `expected` value without a changelog line → not caught by a test (the D-87 review rule): confirm `git status` shows it, then undo.

### Validation Commands

```bash
cargo test -p offcut-detect --test precision_recall -- --nocapture 2>&1 | tee "$TMP/precision.txt" | tail -40
cargo test -p offcut-detect 2>&1 | awk '/^test result:/ {f+=$6} END {print f" failed"}'        # 0 failed
pnpm build:wasm && pnpm check
git grep -n "redetectSentence" -- web/src            # load-render.ts, protocol.ts, render.worker.ts (V2 notYet line, unchanged)
```

### Acceptance Criteria

- [ ] Overall precision ≥ 0.90 over ≥ 30 read files (or the §20.3 action applied and recorded with old/new thresholds and recalls); every prediction ≥ its threshold.
- [ ] `experiments.md` holds the printed table, the sweep and the false positives by file, labelled `d1`, dated.
- [ ] `redetect_sentence` is exported from the render bundle and typed on `RenderApi`; no handler calls it.
- [ ] The drills fired and were undone; no label changed without a changelog line; G M-8 passes.

### Agent Instructions

Before: read §17.5, §19.12, §20.3, §20.4, D-87, §10.2, §13.3, G 8.3–8.5. During: if precision is low, look at the false positives and apply §20.3 exactly; do not tune a rule to a labeled file. After: SAB gate; commit `V3 P79: precision gate and the redetect binding`. Say the gate's result first in the reply.

### Expected Result

V3's detector ships with measured precision, recorded thresholds, and a binding ready for V4's review flow.

---

## Prompt 80 (V3 20/30) — The three IndexedDB repos and `import-clip.ts` (C-2)

**Effort: Medium.** Three small IndexedDB repos and ordered edits to `import-clip.ts`; signatures and the hash's byte layout are given exactly. The browser check is straightforward.

### Objective

Persist each accepted clip's record (with `sourceHashPrefix`, D-81, and `sourceBytes`, D-103), give the pipeline its transcript and edits repos, and send `clip_rejected { reject_reason }` for the 13 import-time codes (G 9.1–9.3; §13.5, §15.1; D-81, D-82, D-97, D-103).

### Project State Before This Prompt

- Prompt 79 committed. V1's `persistence/schema.ts` declares `STORES.clips`, `.transcripts`, `.edits` and the `Versioned<T>` wrapper; `db.ts` opens the database; `opfs.ts` exports `OpfsError` (kinds include `quota`, `io`); the `MINTING` entry for `clips-repo.ts` exists (Prompt 62).
- **Human:** the founder's decision on §23 item 26 is in Entry 1 or this prompt's message. If missing, implement the plan as written (no `clear()`) and report the open decision.
- Verify: `git grep -n "transcripts\|edits\|clips" -- web/src/persistence/schema.ts`; `git grep -n "class OpfsError" -- web/src/persistence/opfs.ts`.

### Why This Comes Here

`run-pipeline.ts` (Prompt 81) writes through these repos; `import-clip.ts` must hash the file before transferring it to the media worker.

### Scope

#### Implement

- `web/src/persistence/clips-repo.ts`: `HASH_PREFIX_BYTES = 4_194_304`; `hashPrefix(file: Blob)` — SHA-256 (WebCrypto) over `file.slice(0, 4 MiB)` followed by the size as u64 little-endian (`DataView.setBigUint64(0, BigInt(size), true)`), lower-case hex; `put(record)`, `get(clipId)`; `toBytes(n)` — the one cast, as the last `return` of a function declaration named `toBytes` (V2 CL item 4).
- `web/src/persistence/transcript-repo.ts`: `put(clipId, { transcript, prosody })`, `get(clipId)`.
- `web/src/persistence/edits-repo.ts`: `put(clipId, { editState })`, `get(clipId)`.
- All three: key `clipId`, value `Versioned<T>` with `schemaVersion: 1`; a stored value with a newer `schemaVersion` is treated as absent and deleted (TS §23.3, as `metaGet` and `entitlement-repo.ts` do); `QuotaExceededError` → `new OpfsError("quota")`, any other failure → `new OpfsError("io")`; no logging; no import of `ui`, `state` or `workers`.
- If the founder chose `clear()` for item 26: one `clear()` per repo, called by `import-clip.ts` step 4 beside the D-40 sweep, and §13.5, §15.1 of the plan updated in this commit; otherwise none.
- `web/src/usecases/import-clip.ts` (§15.1): step 6 `sourceHashPrefix = await clipsRepo.hashPrefix(file)` before `importAndProbe` (a `NotReadableError` → `E_STORAGE_IO`, stage `import`); step 8 on `r.rejected` → `clipStore.rejected(reason)`, `track clip_rejected { reject_reason: reason }`, remove `clipDir`, return; step 10 after `clip_accepted` → `clipsRepo.put({ clipId, createdAt, lastOpenedAt, clipInfo, sourceBytes: toBytes(file.size), sourceHashPrefix })` with `now = Date.now()`; a failed `put` → `clipStore.failed({ code: E_STORAGE_QUOTA or E_STORAGE_IO, stage: "storage", retryable: true })`, remove `clipDir`, return. The file name is never read.
- Browser check (dev): drop `fixtures/.gen/rej_4k.mp4` → one `clip_rejected` with `{"reject_reason":"REJECT_RESOLUTION"}` in the events request and no `clip_accepted` (V2's rejected stub still shows); drop `ok_vfr.mp4` → IndexedDB `clips` holds `sourceBytes` = file size, a 64-hex `sourceHashPrefix` equal to the D-81 hash computed in Node from the file, and no field holding `clip.mp4`.

#### Do Not Implement Yet

- `run-pipeline.ts` changes, `REJECT_NO_SPEECH`'s event (Prompt 81). `RejectionPanel` (Prompt 83). `list`, `touch`, `remove`, quota, eviction (V7). An `events` repo (D-82: none).

### Existing Components To Reuse

`db.ts`'s open helper, `schema.ts`'s `STORES` and `Versioned`, `OpfsError`, the `track` function V2's use-cases use, `newClipId`, `startTimer`, the D-40 stand-in, `clipStore` actions (`rejected`, `accepted`, `failed`).

### Files To Create

- `web/src/persistence/clips-repo.ts`, `web/src/persistence/transcript-repo.ts`, `web/src/persistence/edits-repo.ts`.

### Files To Modify

- `web/src/usecases/import-clip.ts`, `docs/v3/v3changelog.md` (and §13.5/§15.1 of the plan only under the `clear()` decision).

### Architecture Requirements

- Layer rules of V2 §15 plus D-27, D-58, D-59, D-103: repos are `persistence` files imported by use-cases; no new import edge (§13). `hashPrefix` on the main thread is a 4 MiB WebCrypto hash, not media work (INV-17, D-81).
- `ClipRecord` never holds a file name or path (P-11).

### Data Types / Interfaces

```ts
export function hashPrefix(file: Blob): Promise<string>;
export function put(record: ClipRecord): Promise<void>;
export function get(clipId: ClipId): Promise<ClipRecord | undefined>;
export function toBytes(n: number): Bytes;
// transcript-repo.ts / edits-repo.ts
export function put(clipId: ClipId, record: TranscriptRecord | EditsRecord): Promise<void>;
export function get(clipId: ClipId): Promise<TranscriptRecord | EditsRecord | undefined>;
```

Use the record types `schema.ts` declares (V1); do not declare new ones.

### Implementation Requirements

#### 1. Hash layout (§13.5 table)
10 MiB file → SHA-256 over 4,194,304 bytes then `00 00 a0 00 00 00 00 00`; 1,000-byte file → over the 1,000 bytes then `e8 03 00 00 00 00 00 00`; empty blob → SHA-256 of eight zero bytes.

#### 2. Order in `importClip`
Hash before the file is posted (the worker may take ownership).

### Error Handling

Repo failures are `OpfsError` (`quota`/`io`), mapped by `run-pipeline.ts::failureOf` without change (D-103) and by `import-clip.ts` step 10 as above. A `WorkerCallError` is caught at the top and stored; nothing is swallowed.

### Concurrency / Async Requirements

All repo calls are awaited; `importClip` keeps V2's blocker and dismiss rules (steps 1–2).

### Persistence Requirements

- Schema version 1; forward-incompatible values deleted on read (TS §23.3).
- **Records outlive their clip until V7** (§23 item 26) unless the founder chose `clear()`. Record the choice.
- No `events` record (D-82).

### API Requirements

`clip_rejected { reject_reason }` is already on the server allowlist (V1); its one prop is the generated `RejectReason`.

### Security Requirements

- No file name, size or duration in the event (INV-18). `sourceHashPrefix` never leaves the device.
- `toBytes` is the file's only cast (the `MINTING` rule rejects a second one).

### Testing Requirements

#### Unit Tests
None (no Vitest file for the repos, §13.5).

#### Integration Tests
The browser check; the full E2E coverage comes in Prompt 84 (accept case) and 85 (OPFS/IDB case).

#### Failure-Path Tests
G 9.1 drill: write `const b = 5 as Bytes;` in `transcript-repo.ts` → `pnpm check` fails on `MINTING`; undo.

### Validation Commands

```bash
pnpm check
pnpm --filter web exec playwright test --config zz-pw.config.ts      # the dev browser check (prints events and the IDB record)
wc -l web/src/usecases/import-clip.ts web/src/persistence/*-repo.ts  # each under 400
```

### Acceptance Criteria

- [ ] Three repos with exactly the signatures above; quota and io mapped to `OpfsError`; newer `schemaVersion` deleted.
- [ ] `hashPrefix` matches a Node recomputation on `ok_vfr.mp4`; the IDB `clips` record holds `sourceBytes`, `sourceHashPrefix`, no name.
- [ ] Dropping `rej_4k.mp4` sends exactly one `clip_rejected` with the code alone and no `clip_accepted`.
- [ ] The minting drill fired and was undone; item 26's decision is recorded and applied (or reported open).

### Agent Instructions

Before: read §13, §13.5, §15.1, D-81, D-82, D-97, D-103, TS §23.1–§23.3, V2 CL item 4, and `import-clip.ts`, `schema.ts`, `db.ts`, `entitlement-repo.ts` in full. After: SAB gate; commit `V3 P80: IndexedDB repos and import-clip records and rejection event`.

### Expected Result

Every accepted clip has a durable record; every import-time refusal is counted by its code alone.

---

## Prompt 81 (V3 21/30) — `run-pipeline.ts`: the voice chain beside ASR, prosody, all four kinds, step F records (C-4, D-90)

**Effort: High.** Concurrency: the chain runs beside the model wait and ASR, with joins on every failure path, earliest-failure ordering, transfer ownership and INV-12. A listed high-risk area.

### Objective

Replace V2's two stand-ins: the chain runs in `audio.worker` from right after audio extraction, in parallel with the model wait and ASR; prosody is measured on the cleaned audio; detection gets real prosody and yields all four kinds; step F writes OPFS and the two repos; `REJECT_NO_SPEECH` sends `clip_rejected`; FromTo feed lines carry `displayTo` (G 9.4, 9.5; §15.2; D-83, D-90, D-97).

### Project State Before This Prompt

- Prompts 69, 79, 80 committed: `pool.audio`, the V3 render bundle (`detect` of all kinds), the repos.
- V2's `run-pipeline.ts` has 298 lines and two `V3:` markers (verify: `git grep -n "V3: " -- web/src/usecases` prints 2 lines in `run-pipeline.ts`).

### Why This Comes Here

It is the first place the chain, prosody and the V3 detector meet; everything visible in the app (feed, preview audio, records) depends on it.

### Scope

#### Implement

`web/src/usecases/run-pipeline.ts` per the 14-step table of §15.2:

- Step 4 (B2): `pushFeed({ kind: "cleaning_voice" })`; `chainAt = now`; `chain = settle(pool.audio.runChain({ pcm48 }, { transfer: [pcm48.buffer] }))` where `settle` never rejects and yields `{ ok, at } | { failure, at }`; on `ok`, `track stage_timing { stage: "audio_chain", duration_ms }` measured on the main thread (D-35).
- Step 5: remove V2's `out48 = owned(audio.pcm48)` (the audio worker now owns `pcm48`).
- Step 6: on a model failure, `await chain` before `failed(modelFailure)`.
- Step 7 (B1): on an ASR failure, `await chain`, `unload()`, store the earlier failure by `at` (D-90).
- Step 9: on `heard.rejected`, `await chain`, `clipStore.noSpeech()`, `track clip_rejected { reject_reason: "REJECT_NO_SPEECH" }`, remove `clipDir`, return (the `clips` record stays, §23 item 26; the worker's kept buffer stays until the next `runChain`, item 25).
- Step 10: `c = await chain`; a failure → `failed`, stage `audio_chain`.
- Step 11 (D): `{ out48, prosody } = pool.audio.measureProsody({ words, sentences })`.
- Step 12 (E): as V2, with the real prosody; one feed line per event; for `from_to`, `displayOf` returns `params.from.display` and a new `displayToOf` returns `params.to.display`, put in `displayTo` (D-83); `assertNever` default stays.
- Step 13 (F): `opfs.writeAudio(clipId, out48)`; `transcriptRepo.put(clipId, { transcript, prosody })`; `editsRepo.put(clipId, { editState: edit })`; failures → `E_STORAGE_QUOTA`/`E_STORAGE_IO`, stage `storage`.
- Step 14: `clipStore.ready({ transcript, prosody, events, out48 })`; `pipeline_done` with the four counts.
- Remove both `V3:` markers; keep the `V4:` places (client_error, retry, cancel).

Browser check (dev, D1, model cached in a persistent profile): G 9.5 rows — feed's first two lines "Cleaning voice" and "Transcribing…" in either order; then `Found: 3,000 hours`, optionally `Found: brutal`, `Found: 90 minutes to 10 minutes`, `Found: 3-item list`, `Found: $12k` (record with a `MutationObserver` installed before load: the feed vanishes at `ready`, V2 CL item 38); events request: `stage_timing` for `probe_audio`, `audio_chain`, `asr`, `detect_scene`, `pipeline_done` `n_number 2, n_list 1, n_from_to 1, n_keyword 0|1`, no word of the transcript in any body; IndexedDB one record each in `clips`, `transcripts` (with `prosody.per_word.length === transcript.words.length`), `edits`; OPFS `out48.f32` size 14,343,360 bytes. **Human (optional):** listen to the preview: the voice sounds cleaner and in sync.

#### Do Not Implement Yet

Retry, `client_error`, cancel (V4); drawing ListReveal/FromTo/KeywordPop (V4: `events/mod.rs` still returns `Ok(None)` for them); the zero-event message (V4).

### Existing Components To Reuse

`pool.audio` (Prompt 69), `transcriptRepo`, `editsRepo` (Prompt 80), V2's `failureOf`, `pushFeed`, `track`, `modelReady`, `opfs.writeAudio`, `displayOf`, `clipStore.noSpeech`.

### Files To Create

None.

### Files To Modify

- `web/src/usecases/run-pipeline.ts`.

### Architecture Requirements

- **INV-12:** `asr.unload()` before `openSession`; the audio worker holds no GPU session.
- **TS §31:** PCM transferred, never copied; `pcm16` to ASR, `pcm48` to the audio worker, `out48` back.
- **INV-2/INV-18:** no transcript, feed line, label or word in any request.
- No helper moves to another file (a new use-case file needs a TS §5 entry and a lint edge, §15.2 "Size"); stay under 400 lines (about 352).

### Data Types / Interfaces

`AudioWorkerApi` (V1, frozen): `runChain({ pcm48 }) → { loudness }`, `measureProsody({ words, sentences }) → { out48, prosody }`. `FeedLine` with `displayTo?` (Prompt 62).

### Implementation Requirements

#### 1. Joins (D-90)
No path leaves the chain running unawaited; the stored failure is the first in time; a NO_SPEECH waits for B2 to settle.

#### 2. Timing
`audio_chain` measured from the call to its settling; `total_ms` unchanged in definition.

### Error Handling

Every failure path ends in exactly one store transition: `failed(…)`, `noSpeech()`, or `ready(…)`; `E_DSP` keeps its pending copy entry (V4). A promise of the use-case that rejects is a code fault (V2 CL item 37).

### Concurrency / Async Requirements

B2 runs concurrently with C-3 (model wait) and B1 (ASR); joins as above; the audio worker's `kept` is reset by its next `runChain` (D-90).

### Persistence Requirements

Step F writes OPFS first, then the two repos; a failure after the OPFS write fails the clip with `E_STORAGE_*` (the D-40 sweep removes the folder at the next import).

### API Requirements

`stage_timing` `audio_chain`, `clip_rejected` `REJECT_NO_SPEECH`, `pipeline_done` counts — all generated types since V1 (§11).

### Security Requirements

As INV-2/INV-18 above; verified in the events request bodies.

### Testing Requirements

#### Unit Tests
None (`run-pipeline.test.ts` is V4's).

#### Integration Tests
The browser check; `pnpm e2e:media --grep @no-gpu` (8 V2 cases still pass).

#### Failure-Path Tests
Drop `fixtures/.gen/rej_silent.mp4` in dev → V2's rejected stub, one `clip_rejected` with `REJECT_NO_SPEECH`, `clips/` empty in OPFS.

### Validation Commands

```bash
wc -l web/src/usecases/run-pipeline.ts                          # under 400 (about 352)
git grep -n "V3: " -- web/src crates                            # no output
git grep -n "V4: " -- web/src                                   # render.worker handlers, run-pipeline cancel and retry
pnpm check
pnpm --filter web exec playwright test --config zz-pw.config.ts  # the G 9.5 browser check
```

### Acceptance Criteria

- [ ] The reference clip reaches `ready` with the expected feed (four or five "Found:" lines including the FromTo with both values), `audio_chain` in `stage_timing`, the counts above, three IDB records and a 14,343,360-byte `out48.f32`.
- [ ] `rej_silent.mp4` gives `REJECT_NO_SPEECH` with one `clip_rejected`.
- [ ] No `V3:` marker remains anywhere; `run-pipeline.ts` under 400 lines; no request body holds a transcript word.

### Agent Instructions

Before: read §15.2 (all of it), D-35, D-83, D-90, D-97, TS C-3/C-4, V2 §19.3, V2 CL items 37–38, and `run-pipeline.ts` in full. During: change only the listed steps; keep comments short (§15.2 "Size"). After: SAB gate; commit `V3 P81: run-pipeline with the voice chain, prosody and records`.

### Expected Result

The app processes a clip end to end with V3's chain, prosody, detector and records; only NumberReveal is drawn.

---

## Prompt 82 (V3 22/30) — The README's "as recognized" sections; the derived clips' events; Milestone 9

**Effort: Medium.** Operational: run six clips through the pipeline, read their transcripts from IndexedDB, write the README. Little new code, and the rule is "record, never change the detector".

### Objective

Run each derived speech clip and the second recording through the V3 pipeline on D1, write their recognized transcripts, numbers and events into `fixtures/speech/README.md`, and confirm the derived clips give the events of §17.3 (G 9.6, M-9; D-99; §17.7, §20.1).

### Project State Before This Prompt

- Prompt 81 committed. `README.md` has the empty sections "The derived clips as recognized" and "The V3 test clip as recognized" (the founder's words already in it, Prompt 65).
- Verify: `git grep -n "as recognized" -- fixtures/speech/README.md`.

### Why This Comes Here

The pipeline now stores transcripts (IndexedDB) and detects all kinds; the README is the authority for the E2E case of the second recording (Prompt 85) and for V4's golden frames.

### Scope

#### Implement

- For `speech_60s_portrait.mp4`, `speech_60s_landscape.mp4`, `speech_20s_noevents.mp4`, `speech_20s_long_pause.mp4`, `speech_30s_sparse.mp4` (from `fixtures/.gen/speech/`) and `testclips/v3testclip.mp4`: drop in dev on D1 (temporary zz spec, persistent profile so the model stays cached), wait for `ready`, read the newest clip's transcript **by its `clipId`** from IndexedDB (G 9.6 snippet; never "Clear site data", which deletes the 215 MB model), and record the feed lines.
- README: per derived clip, sentences with times and the feed lines; expectations of §17.3 — portrait and landscape: 3,000 hours, (brutal), 90 → 10 minutes, 3-item list; no-events: no "Found:" line (if one appears, record it and change the README expectation with its reason, never the detector); long-pause and sparse: words after the pause start at 10,000 ms or later (long-pause). For `v3testclip.mp4`: recognized sentences with times, numbers, events, and where the text differs from the founder's words; its expected feed for Prompt 85.
- `experiments.md`: `Derived clips' events`, `Second recording` (accepted, `ready`).
- G M-9 table rows: `rej_silent.mp4` and `rej_4k.mp4` checks repeated with the events request.

#### Do Not Implement Yet

E2E cases for these clips (Prompt 85); the bench (Prompt 88); `a02`'s data (Prompt 88).

### Existing Components To Reuse

The zz browser-check pattern; `fixtures/speech/README.md`'s existing table format.

### Files To Create

None.

### Files To Modify

- `fixtures/speech/README.md`, `docs/v3/experiments.md`, `docs/v3/v3changelog.md`.

### Architecture Requirements

The README stays the single authority for what each clip holds (§17.3: "README is the authority").

### Data Types / Interfaces

None.

### Implementation Requirements

#### 1. Privacy
Only the founder's two recordings and clips derived from the public reference clip are transcribed here (D-112). The transcript of `v3testclip.mp4` may be committed (D-112, the founder confirms at Prompt 90); nothing about its location.

#### 2. A failure of `v3testclip.mp4`
A rejection or failure is a V3 bug and blocks the exit (§20.1): stop, record it, report it.

### Error Handling

An expectation that does not hold is recorded in the README with its reason; the detector is never changed to fit (§20.1).

### Concurrency / Async Requirements

One clip at a time (one Playwright worker).

### Persistence Requirements

The persistent profile keeps the model; the database accumulates records (item 26) — read by `clipId`.

### API Requirements

None.

### Security Requirements

The zz spec and profile are deleted (profile under `fixtures/.cache/zz-profile-*`, git-ignored) and nothing is committed but the README and records.

### Testing Requirements

#### Unit Tests
None.

#### Integration Tests
Six pipeline runs on D1.

#### Failure-Path Tests
None new (G M-9's two rejection rows).

### Validation Commands

```bash
pnpm build:wasm && pnpm check
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"                    # 92 passed
cargo test --workspace 2>&1 | awk '/^test result:/ {f+=$6} END {print f" failed"}'  # 0 failed
pnpm e2e && pnpm e2e:media --grep @no-gpu                                           # 12; 8
git grep -c "as recognized" -- fixtures/speech/README.md
```

### Acceptance Criteria

- [ ] The README has six "as recognized" entries (five derived, one V3 test clip) with sentences, times and feed lines.
- [ ] The derived clips' events match §17.3, or each difference is recorded with its reason; `v3testclip.mp4` reached `ready`.
- [ ] `experiments.md` has `Derived clips' events` and `Second recording`; G M-9 passes.

### Agent Instructions

Before: read §17.3, §17.7, §20.1 rows "derived clips" and "second recording", D-99, D-112, G 9.6. After: SAB gate; commit `V3 P82: recognized transcripts of the derived clips and the V3 test clip`.

### Expected Result

Every speech clip V3 tests against has a written, recognized expectation.

---

## Prompt 83 (V3 23/30) — The 14 rejection messages, `RejectionPanel`, the copy check (S9)

**Effort: Medium.** A copy table copied verbatim, one small component, CSS, a check-script edit and three drills. Well bounded.

### Objective

Show every refused file its own title, body (the limit, interpolated from `LIMITS`) and action, replacing V2's rejected stub, and remove the 14 `REJECT_*` codes from `COPY_PENDING` (Phase 10 of the guide; §16; D-80).

### Project State Before This Prompt

- Prompt 82 committed. `messages.ts` has `editor.rejectedStub` and V2's `MS_PER_SECOND`; `check-copy-codes.mjs` lists the 14 codes in `COPY_PENDING`; `EditorPage.tsx` renders the stub for `rejected`.
- Verify: `git grep -n "rejectedStub" -- web/src`; `git grep -c "REJECT_" scripts/check-copy-codes.mjs` is 14 or more.

### Why This Comes Here

The rejection suite (Prompt 84) asserts this copy; the import and pipeline already set the store's `reject` and send the events (Prompts 80, 81).

### Scope

#### Implement

- `web/src/copy/messages.ts`: `rejections` — 14 entries `{ title, body, action }` exactly as the table of §16.3, `satisfies Record<RejectReason, ErrorCopy>`; `BYTES_PER_MB = 1_000_000` and `FPS_MILLI_PER_FPS = 1000` beside `MS_PER_SECOND`; every limit interpolated from `LIMITS` (`MAX_FILE_SIZE`, `MAX_CLIP_DURATION`, `MAX_LONG_SIDE`, `MAX_INPUT_FPS`), none typed in a sentence; `editor.rejectedStub` removed.
- `scripts/check-copy-codes.mjs`: remove the 14 `REJECT_*` entries and their comment from `COPY_PENDING` (it keeps V2's `E_*` codes including `E_DSP` and the four blockers).
- `web/src/ui/components/RejectionPanel.tsx`: `<section role="alert" data-testid="clip-rejected" data-reject={code}>`, heading `.title`, two paragraphs `.body` and `.action`, a "start over" button (`messages.editor.startOver`) calling `dismissClip()`; no retry; no code, size or file name shown; no `switch` (copy read by key).
- `web/src/ui/pages/EditorPage.tsx`: the `rejected` arm of `body()` becomes `<RejectionPanel />`.
- `web/src/ui/styles/components.module.css`: `.rejection`, `.rejectionTitle`, `.rejectionBody`, `.rejectionAction`, composing V2's `.stub` from `pages.module.css` (no V2 class changes; no `style` attribute).
- Browser check (dev): the four refusals of G 10.3 (`rej_4k.mp4` title and `1920`; `rej_hevc.mov` HEVC and "Most Compatible"; `rej_too_long_91s.mp4` `90`; `rej_120fps.mp4` `60`), "start over" then `ok_vfr.mp4` accepted, `data-reject` attribute, no file name in the page text.
- G 10.4 drills; M-10.

#### Do Not Implement Yet

`NO_EVENTS_FOUND`, `ErrorPanel`, `E_DSP` copy (V4). The measured value in a message (not planned, D-80).

### Existing Components To Reuse

`LIMITS` (generated), `ErrorCopy` type (V1/V2 copy), `dismissClip()`, `messages.editor.startOver`, V2's `.stub` class, `useClipStore` (read only).

### Files To Create

- `web/src/ui/components/RejectionPanel.tsx`.

### Files To Modify

- `web/src/copy/messages.ts`, `scripts/check-copy-codes.mjs`, `web/src/ui/pages/EditorPage.tsx`, `web/src/ui/styles/components.module.css`.

### Architecture Requirements

`RejectionPanel` imports `copy/messages`, `state/clip-store` (read), `usecases/import-clip` (`dismissClip`), styles — all allowed for `ui` (§13). CSP `style-src 'self'`: no runtime CSS-in-JS.

### Data Types / Interfaces

```ts
rejections: { [K in RejectReason]: { title: string; body: string; action: string } }   // satisfies Record<RejectReason, ErrorCopy>
```

### Implementation Requirements

#### 1. Wording
The 14 rows of §16.3 verbatim, with `{…}` interpolations computed from `LIMITS` and the named constants.

#### 2. Forbidden phrases (P-12)
None of "never leaves", "GDPR", "DPDP", "CCPA", "SOC 2", "compliant".

### Error Handling

A rejection is a value, not an error: no retry, instructions only (TS §11.3).

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

`data-reject` holds a code, never content, and never leaves the page.

### Testing Requirements

#### Unit Tests
None (`messages.test.ts` is V8's).

#### Integration Tests
The browser check; `node scripts/check-copy-codes.mjs`.

#### Failure-Path Tests
G 10.4: delete the `REJECT_AUDIO_CODEC` entry → `tsc` (`satisfies`) and `check-copy-codes.mjs` both fail naming it; type `90` into the `REJECT_DURATION` body → found by `grep -n "90 seconds"` (review rule, INV-15); a literal sentence in `RejectionPanel.tsx` JSX → ESLint `react/jsx-no-literals` fails. Undo all three.

### Validation Commands

```bash
grep -oE '\bREJECT_[A-Z_]+:' web/src/copy/messages.ts | sort -u | wc -l     # 14
grep -c "REJECT_" scripts/check-copy-codes.mjs                              # 0
grep -niE "never leaves|gdpr|dpdp|ccpa|soc 2|compliant" web/src/copy/messages.ts   # no output
git grep -n "rejectedStub" -- web/src                                       # no output
node scripts/check-copy-codes.mjs && pnpm check && pnpm build
pnpm e2e && pnpm e2e:media --grep @no-gpu
```

### Acceptance Criteria

- [ ] 14 messages with interpolated limits; `COPY_PENDING` has no `REJECT_*`; `rejectedStub` gone.
- [ ] The four refusals show their titles and limits; "start over" returns the drop zone; `data-reject` carries the code.
- [ ] The three drills fired and were undone; G M-10 passes.

### Agent Instructions

Before: read §16, D-80, V1 §11.11, V2 §20.2–§20.3, PS §9.4, and the current `messages.ts`, `EditorPage.tsx`. After: SAB gate; commit `V3 P83: rejection panel and the 14 rejection messages`.

### Expected Result

Every refusal tells the person what the limit is and what to do, in the product's own words.

---

## Prompt 84 (V3 24/30) — E2E helpers and `rejections.spec.ts` (16 cases, 15 `@no-gpu`)

**Effort: High.** 16 E2E cases that must pass on the Windows CI runner without a GPU: the 525 MB in-page `File`, a hash recomputed in Node, IndexedDB reads. V2 had flakiness here (V2 CL items 38-41).

### Objective

Make every refusal and the accept path testable in the browser: helpers for generated fixtures, in-page oversize files, IndexedDB reads and the second recording; the rejection suite of §19.16 joined to `MEDIA_SUITES` (G 11.1, 11.2; D-78, D-79, D-93).

### Project State Before This Prompt

- Prompt 83 committed. V2's `web/tests-e2e/helpers/fixtures.ts` has `dropClip`, `routeAssets`, `ensureModelCached`, `opfsList`, `sourceDurationMs`, `NO_GPU`, `referenceClip()` and its own `SAMPLE_CLIP_PATH` reader; `playwright.config.ts` has `MEDIA_SUITES` and projects `non-media`, `media-setup`, `media`, `bench`.
- Verify: `git grep -n "export function\|export const" -- web/tests-e2e/helpers/fixtures.ts`.

### Why This Comes Here

The panel, events and records it asserts exist (Prompts 80–83); CI (Prompt 87) runs its `@no-gpu` cases.

### Scope

#### Implement

- `web/tests-e2e/helpers/fixtures.ts` (§19.15): `fixturesDir()` (`OFFCUT_FIXTURES_DIR` or `<repo>/fixtures/.gen`, from the helper file's own path); `genFixture(name)` (throws "run pnpm fixtures" when absent or when `STAMP.manifest_sha256` differs from the manifest's SHA-256); `dropFile(page, path, name = "clip.mp4")` (`dropClip` now calls it); `dropOversize(page)` (D-79: in the page, a 1 MiB `Blob` and a `File` of 501 references to it, 525,336,576 bytes, named `clip.mp4`, dispatched on the drop zone in a `DataTransfer`); `v3TestClip()` (`testclips/v3testclip.mp4` or `null`); `readIndexedDb(page, store, key)` (returns the stored `Versioned` value); `rejectCodeOf(page)` (the `data-reject` attribute). Nothing reads a fixture while a test file is being loaded (V2 CL item 40).
- `web/tests-e2e/rejections.spec.ts` (§19.16): 16 cases — one parameterized loop over the manifest's `rej_*` entries **excluding `rej_silent.mp4`** (12 files) plus the oversize case: 13 import-time codes, each `@no-gpu`, asserting `data-reject`, the panel's title and body from `messages.rejections` (limits from `LIMITS`: "500", "90", "1920", "60"), exactly one `clip_rejected` with the code as its only prop, no `clip_accepted`, no `client_error`, no file name/size/duration in the event body, `clips/` empty in OPFS, and for the oversize case that no request carried the file's bytes; `rej_vp9.webm` dropped as `clip.webm`; `REJECT_DECODE_UNSUPPORTED` → `test.fixme` with the reason only if Chrome decodes the file (D-78, as recorded at Prompt 63); `REJECT_NO_SPEECH` (untagged, D1: `ensureModelCached`, after "Transcribing…", the code; `clips/` empty); "start over" (`@no-gpu`: drop zone back, `clips/` empty, a good file accepted); the `ok_vfr.mp4` accept case (`@no-gpu`: `clip_accepted`, leaves `importing`, IDB `clips[clipId]` with `sourceBytes` = size and `sourceHashPrefix` = the D-81 hash computed in Node, no field holding `clip.mp4`; stops before the model).
- `web/playwright.config.ts`: `MEDIA_SUITES` gains `rejections.spec.ts`.

#### Do Not Implement Yet

The D-91 rewrites and second-recording cases (Prompt 85); the corpus project (Prompt 86); CI (Prompt 87).

### Existing Components To Reuse

`installFakeApi`, `routeAssets`, `flushAnalytics`, `api.eventsOf`, `opfsList`, `ensureModelCached`, `NO_GPU`, `MEDIA_USE`, the `MutationObserver` pattern (V2 CL items 38–39).

### Files To Create

- `web/tests-e2e/rejections.spec.ts`.

### Files To Modify

- `web/tests-e2e/helpers/fixtures.ts`, `web/playwright.config.ts`.

### Architecture Requirements

Tests drive the app only through the page; no test hook in app code. Case begin: `installFakeApi`, `routeAssets`, `page.clock.install()`, `goto("/app")`, wait for `drop-zone` (V2 CL item 39).

### Data Types / Interfaces

```ts
export function fixturesDir(): string;
export function genFixture(name: string): string;
export function dropFile(page: Page, path: string, name?: string): Promise<void>;
export function dropOversize(page: Page): Promise<void>;
export function v3TestClip(): string | null;
export function readIndexedDb(page: Page, store: "clips" | "transcripts" | "edits", key: string): Promise<unknown>;
export function rejectCodeOf(page: Page): Promise<string | null>;
```

### Implementation Requirements

#### 1. Case count
`pnpm e2e:media --list | grep -c rejections.spec.ts` = 16; `@no-gpu` total = 23 (V2's 8 + 15).

#### 2. Each case under 30 s on the runner (Appendix A).

### Error Handling

A missing or stale fixture fails with "run pnpm fixtures", never a skip. Only the D1-only and D-78 cases may skip/fixme, with their recorded reasons.

### Concurrency / Async Requirements

Media projects run one worker; each test has its own context and OPFS.

### Persistence Requirements

Each case leaves no OPFS or IDB state beyond its context.

### API Requirements

The fake API's event intake is the assertion surface for `clip_rejected` and `clip_accepted`.

### Security Requirements

The oversize case proves no request carries file bytes (INV-2). The helpers never read a lent clip.

### Testing Requirements

#### Unit Tests
None.

#### Integration Tests
16 cases.

#### Failure-Path Tests
Temporarily delete one `rej_*` entry's file from `fixtures/.gen/` → its case fails "run pnpm fixtures"; regenerate.

### Validation Commands

```bash
pnpm e2e:media --list 2>&1 | grep -c "rejections.spec.ts"            # 16
pnpm e2e:media --grep @no-gpu --list 2>&1 | tail -1                   # Total: 23 tests (plus the setup's)
pnpm e2e:media --grep @no-gpu                                         # 23 passed
pnpm e2e:media --grep "NO_SPEECH"                                     # 1 passed on D1 (keyed build not needed)
pnpm check
```

### Acceptance Criteria

- [ ] 16 rejection cases; 15 tagged `@no-gpu`; the loop excludes `rej_silent.mp4`.
- [ ] `--grep @no-gpu` passes 23; the `REJECT_NO_SPEECH` case passes on D1.
- [ ] Each case asserts copy, code, the single event with no other prop, and empty OPFS; the accept case recomputes the hash in Node.

### Agent Instructions

Before: read §19.15, §19.16, D-78, D-79, D-93, V2 CL items 38–41, and the helpers file in full. After: SAB gate; commit `V3 P84: rejection suite and E2E helpers`.

### Expected Result

Every one of the 14 codes is proven end to end in Chrome; 15 of the cases can run on a runner without a GPU.

---

## Prompt 85 (V3 25/30) — `pipeline-preview` D-91 rewrites, the second-recording cases, the full media run (48+ on D1)

**Effort: High.** Rewrites a reopened V2 contract, adds two D1-only privacy cases (ffprobe metadata) and must pass the full 48+ media run, where any other failure is an app regression to root-cause.

### Objective

Rewrite the three V2 cases that asserted the V2 stand-ins, extend the OPFS case with the three records, add the D1-only case of the second recording to `pipeline-preview.spec.ts` and the D1-only metadata case to `export-creator.spec.ts`, and pass the whole media suite on D1 on a keyed build (G 11.3, 11.6; D-91, D-99, D-100; §19.17, §19.18).

### Project State Before This Prompt

- Prompt 84 committed. The README holds the V3 expected feed and the V3 test clip's expected feed (Prompts 65, 82).
- Verify: `pnpm e2e:media --list | tail -1` shows 30 V2 cases + 16.

### Why This Comes Here

These cases assert the full pipeline (Prompt 81) and the README written from it (Prompt 82). The full run here is the first time all 48 run together.

### Scope

#### Implement

- `web/tests-e2e/pipeline-preview.spec.ts` (**REOPENED V2 CONTRACT**, D-91): "Feed" — "Cleaning voice" and "Transcribing…" are the first two lines in either order, then the "Found:" lines in time order; "Real detections only" — every "Found:" line equals one of the README's V3 expected feed lines, the four always-expected ones all present, "Found: brutal" optional; "Events" — `stage_timing` exactly `probe_audio`, `audio_chain`, `asr`, `detect_scene` (`audio_chain`/`asr` either order), `pipeline_done` with `n_number 2, n_list 1, n_from_to 1, n_keyword` 0 or 1; the OPFS case also asserts one record each in IDB `clips`, `transcripts` (`prosody.per_word.length === transcript.words.length`), `edits`; new D1-only case (`v3TestClip()`; skip reason "testclips/v3testclip.mp4 is on D1 only"): drop, `ready`, `clip_accepted` with `orientation: "landscape"` and `duration_bucket: "lte90"`, feed equals the README's V3 test clip expected feed, the preview plays. Other V2 cases unchanged ("event shape" now also sees `audio_chain` and `clip_rejected` values).
- `web/tests-e2e/export-creator.spec.ts`: one D1-only case (D-100) — drop `v3testclip.mp4`, `ready`, seed a Creator token, export to the system temp directory; `ffprobe -show_entries format_tags:stream_tags` shows no tag key containing `location` or `ISO6709` (case-insensitive), no source `creation_time` (none, or the muxer's epoch 0), `title`, `comment`, `encoder`; a byte search finds neither `©xyz` nor `ISO6709`; `verifyMp4` checks 1–6 pass with `--expected-duration-ms` from `sourceDurationMs`; the file is deleted in a `finally`. The eleven V2 cases unchanged.
- Run the whole media suite on D1 on a keyed build (export the test key in that terminal only; unset it after; detached run): 48 or more passed, 0 failed, 0 skipped.

#### Do Not Implement Yet

The bench and corpus (Prompt 86). The zero-event case (V4). Checks 7–11 (V5).

### Existing Components To Reuse

`v3TestClip`, `readIndexedDb`, `verifyMp4`, `sourceDurationMs`, `seedEntitlement`, `mintEntitlementToken`, `ensureModelCached`, the `MutationObserver` feed recorder.

### Files To Create

None.

### Files To Modify

- `web/tests-e2e/pipeline-preview.spec.ts`, `web/tests-e2e/export-creator.spec.ts`.

### Architecture Requirements

The `pipeline-preview` and `export-creator` cases are one group each in one page (V2 CL item 40): place the new cases so a failure in them does not hide V2's. The media project records no trace (V2 CL item 41).

### Data Types / Interfaces

None new.

### Implementation Requirements

#### 1. Only the three named cases change expectation
Any other V2 case that fails is a regression to fix in app code, not in the test.

#### 2. Founder clip handling
The exported file and any copy live only in the OS temp directory and are deleted; no screenshot of the case is kept.

### Error Handling

A skip in the full run means `testclips/v3testclip.mp4` was not found: fix the path, never accept a skip.

### Concurrency / Async Requirements

One worker; long run detached (SAB rule 9).

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

D-100's case is a privacy promise; the test key never enters a file.

### Testing Requirements

#### Unit Tests
None.

#### Integration Tests
Three rewritten cases, one extended, two new D1-only cases; full suite 48+.

#### Failure-Path Tests
Point the metadata assertion, in a temporary zz copy of the case, at `testclips/v3testclip.mp4` itself: it must fail, naming the location tag. This proves the assertion can fire without changing app code. Delete the copy.

### Validation Commands

```bash
pnpm build:wasm && pnpm fixtures
( pnpm e2e:media > "$TMP/media.log" 2>&1; echo "exit=$?" >> "$TMP/media.log" ) > /dev/null 2>&1 &   # keyed build, D1
tail -5 "$TMP/media.log"                                            # 48 or more passed, 0 failed, 0 skipped
pnpm e2e:media --grep "second recording\|no metadata"               # 2 passed
```

### Acceptance Criteria

- [ ] The three D-91 cases assert V3 behaviour; the OPFS case checks the three records.
- [ ] Both D1-only cases pass; the metadata assertion is shown to fire on the source file itself.
- [ ] `pnpm e2e:media` on D1, keyed: 48+ passed, 0 failed, 0 skipped; the test key is unset afterwards.

### Agent Instructions

Before: read §19.17, §19.18, D-91, D-99, D-100, G 11.3, 11.6, V2 CL items 38–41. After: SAB gate; commit `V3 P85: pipeline and export cases for V3; full media suite on D1`. Changelog: mark **REOPENED CONTRACT** (D-91).

### Expected Result

The media suite describes V3, and passes in full on D1.

---

## Prompt 86 (V3 26/30) — Bench stage and file name; corpus fetcher, runner and project

**Effort: Medium.** Tooling: the bench stage list and file name, a copy-and-hash shell script, a Playwright runner with four modes. Fully specified, and mistakes surface in P88. Keep D-112 in view.

### Objective

Make the device bench measure `audio_chain` and name its file by clip, and build the corpus tooling (TE-12, the E-10 smoke reading, `a02`'s data) with its privacy rules (G 11.4, 11.5; D-88, D-91, D-112; §17.6, §18.2, §18.3, §19.19, §19.20).

### Project State Before This Prompt

- Prompt 85 committed. `bench/device-bench.ts` (V2) has `STAGES` of five, writes `bench/results/d1-<yyyy-mm-dd>.json`, is matched by the project `bench`; `corpus/manifest.json` exists (Prompt 62).
- Verify: `git grep -n "STAGES" -- bench/device-bench.ts`.

### Why This Comes Here

Prompt 88 measures with these tools. They need the helpers (Prompt 84) and a pipeline that stores transcripts (Prompt 81).

### Scope

#### Implement

- `bench/device-bench.ts` (**REOPENED V2 CONTRACT**, D-91): `STAGES = ["probe_audio", "audio_chain", "asr", "detect_scene", "render_encode", "mux"]`; `BENCH_CLIP` = `reference` (default), `portrait` (`genFixture("speech/speech_60s_portrait.mp4")`), `v3testclip` (`v3TestClip()`, refused with a message when absent); result file `bench/results/d1-<date>-<clip>.json`; the result's `clip` field names the fixture, never a path.
- `corpus/fetch_corpus.sh` (POSIX `sh`, LF): copy each manifest clip from `$OFFCUT_CORPUS_DIR/<id>.<ext>` (founder clips from `testclips/`) into `corpus/.clips/<id>.mp4`, check SHA-256, print `ok`/`missing`/`hash mismatch` per clip; no network; exit 0 when at least one clip is ready.
- `corpus/run_corpus.ts` (a Playwright test file): one test per clip present; modes by `CORPUS_MODE` — `demux` (default: drop, wait for accepted/rejected/failed; `corpus/.results/<date>-demux.json` per clip `{ id, outcome, expected, match }`), `export` (built now; verifier 1–6 booleans), `wer` (wait `ready`, read `transcripts[clipId]` from IDB in the page, compare with `corpus/.clips/<id>.txt` after the §17.6 normalisation; counts only), `transcripts` (founder clips only, fixed map `founder-02` → `fixtures/labeled/a02-v3testclip.json` `words`/`prosody`/`sentences`; `founder-01` never written — `a01` stays written-form; any `lent` clip refused). Each test asserts only the expected outcome and no `E_INTERNAL`/`E_WORKER_CRASH`; failure messages name id and code only.
- `web/playwright.config.ts`: project `corpus` — `testDir: "../corpus"`, `testMatch: "run_corpus.ts"`, `dependencies: ["media-setup"]`, timeout `MEDIA_TIMEOUT_MS x 2`, `fullyParallel: false`, `workers: 1`, `use: { ...MEDIA_USE, screenshot: "off", video: "off", trace: "off" }`.
- `web/tests-e2e/tsconfig.json`: `include` gains `../corpus`.
- Root `package.json`: `"corpus": "sh corpus/fetch_corpus.sh && pnpm --filter web exec playwright test --project=corpus"`.
- One bench run with `BENCH_CLIP=portrait` (shortened: confirm six stages in the result, then delete that trial file — the committed runs are Prompt 88's).

#### Do Not Implement Yet

The measurement runs and records (Prompt 88). `--compare` (V9).

### Existing Components To Reuse

The bench's existing run loop and result shape (V2 §23.10); `genFixture`, `v3TestClip`, `readIndexedDb`, `verifyMp4`, `ensureModelCached`; `MEDIA_USE`.

### Files To Create

- `corpus/fetch_corpus.sh`, `corpus/run_corpus.ts`.

### Files To Modify

- `bench/device-bench.ts`, `web/playwright.config.ts`, `web/tests-e2e/tsconfig.json`, `package.json`.

### Architecture Requirements

- Corpus clips are read from `OFFCUT_CORPUS_DIR`, listed by id, SHA-256 and tags only, never uploaded (D-88).
- **D-112:** no transcript of a lent clip is written anywhere — result files hold ids, codes, booleans, numbers; no screenshot, video or trace of a corpus page exists on disk.
- Both `tsc` programs see `corpus/` (Node types, as `../bench`, D-63).

### Data Types / Interfaces

Result files: `{ id, outcome: "accept" | RejectReason | ErrorCode, expected, match }` (+ `verifier: boolean[6]`), `{ id, words_ref, words_hyp, substitutions, deletions, insertions, wer }`.

### Implementation Requirements

#### 1. WER normalisation
Lower-case; keep letters, digits, `$`, `%`, spaces; remove commas inside digit groups; collapse spaces.

#### 2. The `transcripts` map is a constant
Two entries; anything else refused before any read.

### Error Handling

`fetch_corpus.sh` exits non-zero only when no clip is ready; a hash mismatch is reported and that clip skipped.

### Concurrency / Async Requirements

One worker; one clip at a time.

### Persistence Requirements

`corpus/.clips/`, `corpus/.results/` are git-ignored (D-98); `bench/results/*.json` are committed (Prompt 88).

### API Requirements

None.

### Security Requirements

As D-112 above; `fetch_corpus.sh` makes no network call.

### Testing Requirements

#### Unit Tests
None.

#### Integration Tests
`sh corpus/fetch_corpus.sh` (founder-01 ok, founder-02 ok); `--list` of the corpus project; one shortened bench run.

#### Failure-Path Tests
Set `CORPUS_MODE=transcripts` with a fake `lent` entry in a scratch copy of the manifest (not committed) → refused, nothing written.

### Validation Commands

```bash
export OFFCUT_CORPUS_DIR="$HOME/offcut-corpus"; mkdir -p "$OFFCUT_CORPUS_DIR"
sh corpus/fetch_corpus.sh                                                   # founder-01 ok, founder-02 ok
pnpm --filter web exec playwright test --project=corpus --list 2>&1 | tail -1   # Total: 2 tests
git ls-files --eol corpus/fetch_corpus.sh                                    # lf
pnpm check
git status --short corpus                                                   # only the two new source files
```

### Acceptance Criteria

- [ ] `STAGES` has six entries; `BENCH_CLIP` works for `reference`, `portrait`, `v3testclip`; the file name carries the clip.
- [ ] The corpus fetcher, runner (four modes), project, tsconfig include and script exist; `--list` shows one test per present clip.
- [ ] The `lent` refusal works; no screenshot/trace setting is on for `corpus`.

### Agent Instructions

Before: read §17.6, §18.2–§18.3, §19.19–§19.20, D-88, D-91, D-112, G 11.4–11.5. After: SAB gate; commit `V3 P86: bench audio_chain stage and corpus tooling`. Changelog: **REOPENED CONTRACT** (D-91, bench).

### Expected Result

V3 can measure itself on D1 and on lent clips without any lent words leaving the run.

---

## Prompt 87 (V3 27/30) — CI: pinned Linux ffmpeg, fixtures before `cargo test`, the rejection cases on Windows

**Effort: Medium.** A few named workflow steps and one facts line; most of the time is waiting on CI and on your pushes.

### Objective

Make both CI jobs generate the fixtures and run every fixture-reading Rust test and the 15 `@no-gpu` rejection cases, with a checksum-pinned Linux ffmpeg, and get the pull request green (G 11.7–11.10, M-11; D-93, D-94; §18.4–§18.6).

### Project State Before This Prompt

- Prompt 86 committed. `ci.yml` (V2) has jobs `ci`, `e2e-media`, `deploy-api`, `deploy-web`, the gitleaks step, the test-key guard; `e2e-media.yml` installs ffmpeg 9.0.2 essentials on Windows and restores `fixtures/.cache`; Entry 1 holds `FFMPEG_LINUX_URL` and `FFMPEG_LINUX_SHA256`.
- Verify: `git grep -n "actions/cache@" -- .github/workflows` (pinned SHA, v6).

### Why This Comes Here

All test files exist; the branch must be green before measurement and merge.

### Scope

#### Implement

- `.github/workflows/ci.yml` (§18.5): `env` gains `FFMPEG_LINUX_URL`, `FFMPEG_LINUX_SHA256`; before step 4 (`cargo test`), after pnpm is set up: "Install ffmpeg (Linux, pinned)" (`curl -fsSL`, `sha256sum --check`, extract to `$RUNNER_TEMP/ffmpeg`, append the binaries' folder to `$GITHUB_PATH`), "ffmpeg version" (prints the first line), "Restore the reference clip" (`actions/cache` at the same pinned SHA and key `e2e-media.yml` uses for `fixtures/.cache`), "Generate the fixtures" (`pnpm fixtures`, `VITE_ASSET_BASE_URL` in the job's `env`). Step 4's command unchanged. Everything else unchanged, including the guard and the deploy order.
- `.github/workflows/e2e-media.yml` (§18.6): move "Restore the model files and the reference clip" directly after "Install ffmpeg", then `pnpm fixtures`; the Playwright command unchanged (`--grep @no-gpu`, now 23 cases); the head comment names the rejection suite; the failure report artifact stays (it can only hold synthetic fixtures).
- `scripts/check-external-facts.mjs` (§18.4): one line for the Linux ffmpeg archive: URL and the date its checksum was taken.
- **Human:** `git push -u origin v3-build`; open the pull request to `main` (not merged). The agent reads results with `curl -s https://api.github.com/repos/mayanpathak/Offcut/commits/<sha>/check-runs` and asks the human to paste the `ci` log lines of G 11.9 (the ffmpeg version line; the six test binaries `demux_fixtures`, `probe_rejections`, `loudness_target`, `duration_preserved`, `chain_no_clicks`, `prosody` running) and the Windows job's `REJECT_DECODE_UNSUPPORTED` result (record under `rej_h264_444 in Chrome`).
- G 11.10 guards: two throwaway commits (one hex digit of `FFMPEG_LINUX_SHA256` changed → the Linux job fails at `sha256sum --check`; "Generate the fixtures" removed → step 4 fails "run `pnpm fixtures`"), each pushed by the human and reverted with `git revert` (no force-push unless the human chooses it). End state green.

#### Do Not Implement Yet

Merging (Prompt 90). Any deploy change. Any new job.

### Existing Components To Reuse

The pinned `actions/cache` and `actions/upload-artifact` SHAs already in the workflows; V2's ffmpeg install step pattern in `e2e-media.yml`; `check-external-facts.mjs`'s existing fact format.

### Files To Create

None.

### Files To Modify

- `.github/workflows/ci.yml`, `.github/workflows/e2e-media.yml`, `scripts/check-external-facts.mjs`.

### Architecture Requirements

- A pull request runs steps 1–9 and deploys nothing; a push to `main` deploys (memory, V2): never push to `main`.
- Every downloaded binary is checksum-verified (refused on mismatch); no "latest" URL.

### Data Types / Interfaces

None.

### Implementation Requirements

#### 1. Order
pnpm installed before `pnpm fixtures`; fixtures before `cargo test`.

#### 2. Version string
The generator's `case` accepts the Linux build's `n9.0.x` prefix (Prompt 63).

### Error Handling

A cache miss downloads the reference clip from the asset host with its hash checked by the generator (exit 3 on mismatch).

### Concurrency / Async Requirements

`cancel-in-progress` for pull requests is V2's; unchanged.

### Persistence Requirements

The `fixtures/.cache` cache key is shared with `e2e-media.yml`; generated fixtures are never cached (D-94).

### API Requirements

None.

### Security Requirements

- Checksum pinning of the ffmpeg archive (D-94); `actionlint` clean if available (V2 used 1.7.12).
- No secret added; `FFMPEG_LINUX_*` are public values. The gitleaks scan still passes.

### Testing Requirements

#### Unit Tests
None.

#### Integration Tests
The two CI jobs on the pull request.

#### Failure-Path Tests
The two throwaway commits of G 11.10.

### Validation Commands

```bash
actionlint .github/workflows/ci.yml .github/workflows/e2e-media.yml     # if installed; else record that it was not run
pnpm check && pnpm fixtures && pnpm test && pnpm build && pnpm e2e
curl -s "https://api.github.com/repos/mayanpathak/Offcut/commits/$(git rev-parse HEAD)/check-runs" | node -e '
let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{for(const r of JSON.parse(s).check_runs) console.log(r.name, r.status, r.conclusion)})'
```

### Acceptance Criteria

- [ ] The pull request's `ci` and `e2e-media` jobs are green (read through the API) after the guard drills were reverted.
- [ ] The pasted Linux log shows the pinned ffmpeg version line and the six fixture-reading test binaries running, none skipped.
- [ ] The Windows job ran 23 `@no-gpu` cases; the `REJECT_DECODE_UNSUPPORTED` outcome is recorded.
- [ ] Both guard drills failed where expected (Human pushed); G M-11 passes.

### Agent Instructions

Before: read §18.4–§18.6, D-93, D-94, G 11.7–11.10, and both workflow files in full. During: change only the steps named. After: SAB gate; commit `V3 P87: CI fixtures, pinned Linux ffmpeg, rejection cases`. List the Human push and PR steps first in the reply.

### Expected Result

CI proves on every pull request that fixtures generate and that the demuxer, DSP and rejection behaviour hold.

---

## Prompt 88 (V3 28/30) — Measure: bench, TE-12, E-3/E-4 re-reads, `a02` data, E-10 smoke, the founder's reading (S12)

**Effort: Medium.** Running the measurements and recording them; the decisions are pre-agreed (§20). Switch to High if the precision re-run on 31 files falls under 0.90.

### Objective

Read every V3 measurement of §20.1 on D1 in the production build, commit the first `d1` bench files, fill `a02`'s data and re-run the precision gate on 31 files, and record each result with its decision (Phase 12; §20; D-74, D-87, D-88, D-112).

### Project State Before This Prompt

- Prompt 87 committed; the pull request is green.
- `~/v3testclip-words.txt` exists (Human, Prompt 65); `OFFCUT_CORPUS_DIR` may hold lent clips.
- Verify: `sh corpus/fetch_corpus.sh` prints `ok` twice.

### Why This Comes Here

Measurements are taken on the finished V3 build (§5 S12); V4 compares against these files (§20.5).

### Scope

#### Implement

- G 12.1 bench (keyed build, D1, detached, ~25 min each): `BENCH_CLIP=portrait pnpm bench:device` and `BENCH_CLIP=v3testclip pnpm bench:device` → `bench/results/d1-<date>-portrait.json`, `…-v3testclip.json`, 10 runs, six stages. Normalise `v3testclip` readings to 60 s (× 60,000 / 78,918). Record under `TE-13 browser` (confirm the S6 decision; §20.2), `E-3 re-read` (asr: target 20 s, line 145 s per 60 s, D-67), `E-4 re-read` (render_encode: 90 s; over 150 s → Canvas2D, a V4 decision), each with `d1` and the D-74 caveat (D1 is not R1).
- G 12.2: `pnpm e2e:media --grep "second recording\|no metadata"` 2 passed; `Second recording` updated.
- G 12.3: `CORPUS_MODE=transcripts pnpm corpus -- --grep founder-02` → `a02-v3testclip.json` gains `sentences`, `words`, `prosody`; `git diff fixtures/labeled` changes only that file and no `"expected"` line; precision gate re-run: 31 files read, 0 skipped, ≥ 0.90 (else §20.3). Changelog line: "a02-v3testclip.json: data filled from IndexedDB, labels unchanged (D-87, D-112)". If `a02`'s `say` values do not all match the recognized words, record each (a label stays as the founder's words; a mismatch is a recall miss, not a reason to edit).
- G 12.4 TE-12: `CORPUS_MODE=demux pnpm corpus`; fewer than 10 clips → "TE-12: inconclusive (n clips)", re-check in V9 (D-88); ≥ 10 under 90% → each failing clip's code recorded, the JS-demuxer decision before V4's preview work (§20.4). Copy counts and rates only (no lent ids) into `experiments.md`.
- G 12.5 E-10 smoke: `corpus/.clips/founder-01.txt` from Script A's text, `founder-02.txt` from `~/v3testclip-words.txt` (local, never committed); `CORPUS_MODE=wer pnpm corpus`; record both WERs as "smoke, not E-10 (carried to V9, D-88)".
- G 12.6 **Human**: pick five `w*.json` at random (`ls fixtures/labeled/w*.json | shuf -n 5`); the founder answers per file; record names and answers. A "no" → fix per D-87 with a changelog line, re-run the gate.
- G 12.7: in `docs/technicalspec.md` §39.3 and the sections they come from, replace each (assumption) V3 measured with its value and "measured in V3, docs/v3/experiments.md": `DENOISE_DELAY_SAMPLES`, TE-13's decision, thresholds changed by the gate, the generator's time, `rej_silent.mp4` length, `rej_h264_444.mp4` decodability; leave unmeasured ones listed.

#### Do Not Implement Yet

Spec copy-back of decisions (Prompt 89). Merge (Prompt 90).

### Existing Components To Reuse

The bench, corpus runner and fetcher (Prompt 86); `precision_recall.rs` (Prompt 79).

### Files To Create

- `bench/results/d1-<date>-portrait.json`, `bench/results/d1-<date>-v3testclip.json`.

### Files To Modify

- `fixtures/labeled/a02-v3testclip.json` (data only), `docs/v3/experiments.md`, `docs/v3/v3changelog.md`, `docs/technicalspec.md` (§39.3 and source sections, measured values only).

### Architecture Requirements

Every reading labelled `d1` (D-74); budgets read against the 60 s budget scaled by length (D-64, D-74).

### Data Types / Interfaces

Bench result shape of V2 §23.10 with six stages.

### Implementation Requirements

#### 1. Experiments file complete
Every heading of Prompt 61 has `Numbers:` filled except `E-10` (carried to V9).

#### 2. Privacy
`corpus/.clips/*.txt` and `corpus/.results/` stay ignored; no lent id in `experiments.md`.

### Error Handling

A bench run that crashes: rerun once; a second crash is a V3 bug to report (`E_WORKER_CRASH` must not occur, §19.20).

### Concurrency / Async Requirements

Detached long runs; one bench at a time; close other browsers (V2 CL item 20).

### Persistence Requirements

Bench files are committed data; `a02` gets data, not labels.

### API Requirements

None.

### Security Requirements

The test key only in the bench terminal; no lent transcript anywhere (D-112).

### Testing Requirements

#### Unit Tests
None.

#### Integration Tests
Bench x2, the two D1-only cases, corpus demux and wer, the precision gate on 31 files.

#### Failure-Path Tests
None new.

### Validation Commands

```bash
ls bench/results/ | grep -E '^d1-.*-(portrait|v3testclip)\.json$' | wc -l           # 2
grep -cE "^## " docs/v3/experiments.md                                             # 14
grep -c "Numbers: $" docs/v3/experiments.md                                        # 1 (E-10 only)
cargo test -p offcut-detect --test precision_recall -- --nocapture 2>&1 | grep -iE "overall|files read"   # 31 files read; >= 0.90
git status --short corpus fixtures/labeled                                         # only a02-v3testclip.json
```

### Acceptance Criteria

- [ ] Both bench files committed with 10 runs and six stages; TE-13, E-3, E-4 recorded (measured and normalised) with decisions.
- [ ] `a02` has data, its labels unchanged; the gate passes on 31 files.
- [ ] TE-12 recorded (or inconclusive with the count); E-10 smoke recorded; the founder's five-file reading recorded (Human).
- [ ] TS §39.3 holds the measured values; G M-12 passes.

### Agent Instructions

Before: read §20 in full, D-64, D-67, D-74, D-87, D-88, D-112, G Phase 12, V2 CL items 20, 24, 33. After: SAB gate; commit `V3 P88: measurements, experiment records and the first d1 bench files`.

### Expected Result

V3's numbers exist, labelled, dated and committed; V4 has a baseline to compare against.

---

## Prompt 89 (V3 29/30) — Spec copy-back and the final local gate (S13)

**Effort: High.** Faithful copy-back of about 40 decisions into the right TS/BP/PS sections without paraphrase drift, plus the full final gate and four audits. Errors here mislead V4-V10.

### Objective

Make the specifications say what V3 built (every decision of §23.2 copied into TS, BP, PS; §23 items 1–14 corrected; thresholds in use in TS §17.4), confirm every value that must be real, and pass the complete local gate including the full media suite before the merge (G 13.1–13.3; §23, §22 "Code and checks").

### Project State Before This Prompt

- Prompt 88 committed; `experiments.md` complete.
- Verify: `git grep -n "V3: " -- web/src crates` empty; the pull request green.

### Why This Comes Here

The merge (Prompt 90) deploys what the specs describe; S13 precedes S14 (§5).

### Scope

#### Implement

- `docs/technicalspec.md`: D-73 and D-92 (§5 tree, already); D-76, D-77, D-111 (§4, §26); D-78 (§15.3 fixture column); D-80 (§15.3 message column); D-81, D-82 (§23.1); D-83 (§14.1); D-84, D-85, D-101, D-102, D-105, D-106, D-107 (§17.2, §17.4); D-86 (§17.3); D-89 (§3); D-90 (§9 C-4); D-94 (§33); D-95 (§28); D-96, D-108 (§18.2, §18.3); D-104, D-109 (§17.2, §17.6); D-110 (§18.1, §34.3); D-74 beside D-69 (§28, §30, §37); D-100 (§25.1); the thresholds in use (§17.4); §23 items 1–14; the corrections found while planning (six fixture-reading tests; split ffmpeg flags; `limit` returns the clamp count; SCHEMA's `tags`, `edits`, null data, written `a01`; the `transcripts` file map; 15 `@no-gpu` cases).
- `docs/buildplan.md`: D-74 (§5.3), D-76 (§5.1 B, §5.2), D-87 (§5.2), D-88 (§5.3), D-93, D-94 (§5.4); V3's eight days.
- `docs/product.md`: D-80 (§9.4). Nothing else.
- `docs/v3/v3implementation.md`: §23 items 25–28 hold their outcome (decided with the decision, or open with their version); any D-n that changed during the build.
- G 13.2 final local gate: `pnpm install --frozen-lockfile && pnpm check`; `pnpm fixtures && pnpm test`; Rust counts (0 failed; passed well above the baseline; ignored = baseline + `chain_timing`); Vitest ≥ 92; `pnpm build`; `pnpm e2e` 12; `pnpm e2e:media` on D1 keyed 48+ / 0 / 0 (this is also the D-72/D-75 pre-merge run); the `fetch(` audit; the TODO audit; the 400-line audit over every changed source file; the frozen check. Write the final counts into the changelog; none below Entry 1.
- G 13.3: check each real value (Linux ffmpeg URL/hash and the CI version line; corpus manifest hashes; the manifest's observed values; `DENOISE_DELAY_SAMPLES` and `denoise`; thresholds equal to TS §17.4; no test key in the production build; Render and Vercel unchanged).
- Tick every box of §22 "Code and checks" and "Experiments and records" that has its evidence (deployment boxes in Prompt 90).

#### Do Not Implement Yet

The merge, deploy and tag (Prompt 90).

### Existing Components To Reuse

§23.2's list; the audits of G 13.2.

### Files To Create

None.

### Files To Modify

- `docs/technicalspec.md`, `docs/buildplan.md`, `docs/product.md`, `docs/v3/v3implementation.md` (checkboxes and outcomes only), `docs/v3/v3changelog.md`.

### Architecture Requirements

Copy-back changes documents only; no code changes in this prompt. If an audit finds a code problem, fix it minimally, record it, and rerun the whole gate.

### Data Types / Interfaces

None.

### Implementation Requirements

#### 1. Copy-back fidelity
Each copied decision cites "v3implementation D-n"; wording matches the decision, not a paraphrase that changes it.

#### 2. Audits

```bash
git grep -nE "\bfetch\(" -- web/src | grep -vE "net/http\.ts|net/asset-fetch\.ts|wasm/load-core\.ts|wasm/load-render\.ts"   # no output
git grep -n "TODO" -- crates web/src | grep -vE "TODO\(V[0-9]+\)|V[0-9]+:"                                                  # no output
for f in $(git diff --name-only "$BASE" -- crates web/src | grep -E '\.(rs|ts|tsx)$' | grep -v '/tests'); do
  n=$(awk '/#\[cfg\(test\)\]/{print NR-1; f=1; exit} END{if(!f) print NR}' "$f"); [ "$n" -ge 400 ] && echo "TOO LONG $f $n"
done; echo lengths checked
git grep -niE 'silence|pause|gap' -- crates/offcut-dsp/src                                                                  # comments and ABSOLUTE_SILENCE only
```

### Error Handling

A failing audit blocks the prompt.

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

No test key in `web/dist`; `ENTITLEMENT_PUBLIC_KEYS` holds the production key only in a plain build.

### Testing Requirements

#### Unit Tests
All existing.

#### Integration Tests
The full local gate and the full media suite.

#### Failure-Path Tests
None new.

### Validation Commands

The G 13.2 block and the audits above; `node scripts/check-file-tree.mjs`.

### Acceptance Criteria

- [ ] Every item of §23.2 is in its named spec section; §23 items 25–28 have outcomes.
- [ ] The final local gate is green, the media suite 48+ / 0 failed / 0 skipped on D1 keyed, all four audits print nothing; final counts recorded.
- [ ] Every real value of G 13.3 checked with evidence; §22's non-deployment boxes ticked.

### Agent Instructions

Before: read §22, §23, §23.2, G 13.1–13.3. After: SAB gate; commit `V3 P89: spec copy-back and close-out checks`. Then tell the human the branch is ready to merge (Prompt 90).

### Expected Result

A merge-ready branch whose specs, records and tests all agree.

---

## Prompt 90 (V3 30/30) — Merge, deploy, verify the deployed system, exit checklist, tag `v3` (S14, S15)

**Effort: Medium.** Mostly your steps (merge, SQL, tag) and verification commands against production; the rollback rule is already written.

### Objective

V3 live on `main`, verified from outside on D1 (rejections with their copy, the cleaned voice, the analytics rows), every box of §22 and §21.3 ticked with evidence, and `v3` tagged on the deployed commit (G 13.4–13.6, M-13).

### Project State Before This Prompt

- Prompt 89 committed; the pre-merge `pnpm e2e:media` run on D1 passed 48+ / 0 / 0 on the head commit.
- Verify: the pull request's checks are green through the API for the head SHA.

### Why This Comes Here

It is the end of V3 (BP §5.4): deployed, measured, recorded, tagged; V4 starts from it (§21.3).

### Scope

#### Implement

- **Human:** mark the pull request ready; merge it into `main` (D-75). `ci.yml` on `main` deploys the server first, waits for its version, then the web build (TS §33).
- Agent, after the merge: `git fetch origin`; `MERGE=$(git rev-parse origin/main)`; poll the check runs of `$MERGE` through the API until `ci` completes; `curl -s "$APP_URL/api/v1/healthz"` reports `$MERGE` (the field `wait-for-version.mjs` reads); `node scripts/check-headers.mjs <the arguments ci.yml passes>` exits 0; `E2E_BASE_URL=$APP_URL pnpm --filter web exec playwright test --project=non-media --grep @smoke` passes; the deployed scripts hold no test key (search the deployed bundle as V2 did).
- Deployed checks on D1 (G 13.5), through a temporary zz spec against `$APP_URL`: `crossOriginIsolated === true`; drop `fixtures/.gen/rej_4k.mp4` → "This video is larger than Offcut handles", body containing `1920`, `data-reject="REJECT_RESOLUTION"`; drop `rej_hevc.mov` → the HEVC message with "Most Compatible"; "Try with a sample clip" → "Cleaning voice", "Transcribing…", the four always-expected "Found:" lines, the preview plays (Human may listen).
- **Human:** in the Neon SQL editor, the query of G 13.5 → `clip_rejected` rows with props `{"reject_reason":"REJECT_RESOLUTION"}` and `{"reject_reason":"REJECT_HEVC"}` and no other key; `stage_timing` rows with `"stage":"audio_chain"`. The agent never queries production.
- **Rollback rule (G 13.5):** `healthz` not reporting `$MERGE` within 15 minutes, or any deployed check failing → open a pull request with `git revert -m 1 $MERGE` for the human to merge, confirm `healthz` reports the revert and the app behaves as V2, then fix on `v3-build` and repeat Prompts 89–90.
- **Human:** confirm D-112 for `a01` and `a02` (record the answer).
- Tick every remaining box of §22 (deployment) and §21.3 with its evidence; final changelog entry "V3 closed: <date>, merge <SHA>, final counts"; commit it on a branch and open a small pull request if branch protection requires one (Human merges).
- **Human:** `git tag -a v3 -m "V3: real ingest, clean voice, real detector" <deployed commit>`; `git push origin v3`. Agent verifies with `git ls-remote --tags origin v3`.

#### Do Not Implement Yet

Anything of V4 (review controls, drawing the three new kinds, `NO_EVENTS_FOUND`, `ErrorPanel`, cancel). The fixes of §23 items 25–28 unless decided and done in Prompt 89.

### Existing Components To Reuse

`scripts/check-headers.mjs`, `wait-for-version.mjs` (via `ci.yml`), the `@smoke` case, the zz spec pattern.

### Files To Create

None (temporary zz files only, deleted).

### Files To Modify

- `docs/v3/v3implementation.md` (§22, §21.3 ticks), `docs/v3/v3changelog.md`.

### Architecture Requirements

- The server is unchanged (§12): `deploy-api.yml` redeploys the same image; `git diff --name-only 8a271a9 <merge> -- server` is empty.
- Deploy order server-first; the test-key guard passes on `main`.

### Data Types / Interfaces

None.

### Implementation Requirements

#### 1. Evidence
Each ticked box names its proof (command output, run id, SQL result pasted by the human).

#### 2. No agent push, merge, tag or production query
All four are Human (memory: pushes and merges are the user's).

### Error Handling

Any deployed failure → the rollback rule, never a hot fix on `main`.

### Concurrency / Async Requirements

Poll the deploy with a bounded loop; no busy waiting.

### Persistence Requirements

Production analytics are read by the human only.

### API Requirements

`/api/v1/healthz` reports the deployed commit; analytics props are enums only (INV-18).

### Security Requirements

No test key in the deployed bundle or in Vercel's environment; headers (COOP/COEP/CSP) verified; no new CSP host (INV-1).

### Testing Requirements

#### Unit Tests
None.

#### Integration Tests
Deployed checks above; `@smoke` against production.

#### Failure-Path Tests
None (the rollback rule is the failure path).

### Validation Commands

```bash
curl -s "$APP_URL/api/v1/healthz"                                       # 200, commit = merge (or the changelog commit after it)
node scripts/check-headers.mjs <ci.yml arguments>                       # exit 0
git ls-remote --tags origin v3                                          # one line
git diff --name-only 8a271a9 "$MERGE" -- server                         # no output
grep -c "\- \[ \]" docs/v3/v3implementation.md                          # 0 (every box of §1A, §21.3, §22 ticked)
```

### Acceptance Criteria

- [ ] `ci` green on `main`; `healthz` reports the merge; headers, `@smoke` and `crossOriginIsolated` hold; no test key deployed.
- [ ] The deployed page refuses `rej_4k.mp4` and `rej_hevc.mov` with their messages and processes the sample clip with "Cleaning voice" and the four always-expected events.
- [ ] Analytics rows carry the code alone and `audio_chain` (Human SQL).
- [ ] D-112 confirmed for `a01`, `a02`; every box of §22 and §21.3 ticked; final changelog entry; `v3` tag on the deployed commit (Human push).

### Agent Instructions

Before: read §21.3, §22, D-75, D-112, G 13.4–13.6, M-13. During: perform no Human step; list them in order for the human. After: final changelog commit (via pull request if required); report the tag check and every box's evidence.

### Expected Result

V3 is deployed, verified, recorded and tagged; V4 starts on its first morning with every precondition of §21.3 true.

---

# Cross-Prompt Contracts

| Contract | Owner → consumers | Shape | Compatibility / change rule |
| --- | --- | --- | --- |
| Baseline and counts | P61 → every gate | `BASE=8a271a9`; Entry 1 counts (262 Rust/2 ignored as printed, 92, 12, 30) | Counts may only rise; baseline never moves in V3 |
| TS §5 tree | P61 (+P70, P72 if deferred) → `check-file-tree` | Lists `docs/v3/*` (5), `edit_list.rs`, `cardinals.rs` | A new file needs its tree line in the same commit |
| `FeedLine.event_found.displayTo?` | P62 → P81 (sender), `ProcessingFeed` (reader) | Optional string; FromTo only | Frozen from P62 (§21.1); REOPENED V1 contract, D-83 |
| `feed.eventFound.from_to(display, displayTo)` | P62 → `ProcessingFeed` | Two strings; `""` gives one-value line | Frozen copy key and shape |
| `MINTING` entry for `clips-repo.ts` | P62 → P80 | One cast in `toBytes` | Exactly one entry (D-103) |
| `corpus/manifest.json` | P62 → P86, P88 | `{ schema, clips: [{ id, sha256, tags, expect, owner }] }` | Ids, hashes, tags only; never a path or name |
| Fixture manifest and `STAMP` | P63 → P67, P68, P71, P84, P86, P87 | §17.4 format; `STAMP.manifest_sha256` | Any edit regenerates; readers check the stamp |
| Fixture names and recipes' properties | P63 → all fixture readers, V4, V5, V7 | 25 names of §17.2–§17.3 | Frozen names (D-111); a real recording later takes the same name |
| Labeled-file format, unit and tag vocabularies | P64 → P65, P79, P88 | `SCHEMA.md` | Frozen from P64; labels change only with a changelog line (D-87) |
| `limit(...) -> usize` | P66 → P67 tests | Clamp count | Frozen (copied to TS in P89) |
| `run_chain`, `ChainConfig`, `Flow`, `ChainOutput`, P1–P8 | P67 → P69, V5 | TS §34.3 | Frozen (§21.1); `denoise` default set by TE-13 only |
| `DENOISE_DELAY_SAMPLES` | P67 → P69, P88 | Measured constant in `config.rs` | Changed only by re-measurement |
| `measure_prosody` | P68 → P69, P81 | Reads only; `per_word.len() == words.len()` | Frozen |
| `audio_api.rs` exports, `CoreApi.runChain/measureProsody`, `ChainResult` | P69 → P81 | §10.1, §13.2 | Frozen; additive only later |
| Pool `audio` row; `AudioWorkerApi` (V1) | P69 → P81, V4 (cancel/restart) | Stage `audio_chain` | V4 adds cancel and restart only |
| `resolve_edits`, `EditPlan` | P70 → `sample_table.rs` | Crate-private | E0–E4; shapes widen only by a V9 decision |
| `parse_quantity` grammar, `format_quantity` display table | P73/P74 → P75–P77, scene count-up | §8.3, §8.5, `numbers_table.rs` | Frozen (§21.1) |
| `units_lex::COMPARATIVES` (public) | P72 → parser | Sorted slice | Public only for this read |
| `Candidate.parts`, `Found`, `quantities` | P75 → P76, P77 | Crate-internal | Additive fields only |
| `DetectorConfig` field names, `threshold()`, `hold_ms()` | P75 → P76–P79, V4 | 39 fields | Names frozen; values tuned only by the gate (P79) or E-2 |
| `list::markers` | P76 → P77 (X6), keyword skip | `pub(crate)` | Internal |
| `detect`, `redetect_sentence` signatures; pipeline order | P77, P78 → P79 binding, V4 | TS §17.1 | Frozen (§21.1) |
| `RenderApi.redetectSentence` | P79 → V4 render worker handler | §13.3 | Frozen |
| Repos' signatures, `ClipRecord`, `sourceHashPrefix` layout | P80 → P81, P84, V4 (`editsRepo.put` on edits), V7 (restore, eviction) | §13.5, D-81 | Frozen; V7 adds `list/touch/remove` |
| `clip_rejected { reject_reason }` | P80, P81 → P84, server allowlist (V1) | Enum prop only | Frozen (INV-18) |
| `messages.rejections` keys and shape; `data-reject` | P83 → P84, V4+ | `Record<RejectReason, ErrorCopy>`; attribute on `clip-rejected` | Frozen (§21.1) |
| E2E helpers of §19.15 | P84 → P85, P86 | Function signatures | Additive only |
| Bench stages and file name | P86 → P88, V4 compare | Six stages; `d1-<date>-<clip>.json` | Frozen; REOPENED V2 contract (D-91) |
| Corpus result format | P86 → P88, V9 | Ids, codes, booleans, numbers | No text ever (D-112) |
| CI env `FFMPEG_LINUX_URL`, `FFMPEG_LINUX_SHA256` | P61 (chosen) → P87 | Version-named URL + SHA-256 | Changed only with `check-external-facts.mjs` |

# High-Risk Implementation Areas

| Risk | Why it matters | Prompt responsible | Validation strategy |
| --- | --- | --- | --- |
| TE-13: chain too slow in WASM | Over 4 s per 60 s on D1 adds to the user's wait and may miss on R1 | 68, 69, 88 | Native median < 2 s; browser median ≤ 4 s; fallback `denoise: false` pre-agreed (§20.2) |
| Denoiser delay not constant | Misaligned wet/dry mix smears transients; a timeline shift would break INV-5 | 67 | `impulses_stay_aligned` within 48 samples; if no constant works, denoise off |
| A length change anywhere in the chain | Breaks INV-10, captions drift from voice | 67 | `duration_preserved.rs` property test over 1–300,000 samples; no pad/trim branch |
| Labeled set written to suit the rules | Precision would be meaningless | 64, 65, 75 | Labels committed before any rule file; G 7.1 timestamp check; agent does not open labels while writing rules; founder's five-file reading (P88) |
| KeywordPop exactly at threshold (§23 item 27) | f32 rounding decides written-emphasis events | 64, 77, 79 | Founder decides before P64; one comparison site in `lib.rs` |
| Precision under 0.90 | V3's gate fails; V4 would draw wrong events | 79 | Sweep printed; §20.3 contingency; never edit a label to pass |
| Fixture nondeterminism across shells and ffmpeg builds | Flaky tests; stale `.gen` passing | 63, 87 | Two clean runs byte-identical; `STAMP` with manifest and script hashes; tests refuse a stale stamp |
| Linux ffmpeg version string / archive drift | CI generator refuses or a moving URL changes outputs | 61, 63, 87 | Version-named URL, SHA-256 checked; `case` accepts `n9.0.x` |
| GPS location of `v3testclip.mp4` leaking | Publishes where the founder records | 61, 63, 71, 85 | Never uploaded or committed; derived fixtures stripped; metadata assertions incl. `ISO6709`; reference clip checked clean |
| Lent clips' words leaking | Breaks the promise to lenders (D-112) | 86, 88 | Results hold numbers only; corpus project records no screenshot/trace; `transcripts` mode refuses `lent` |
| Join ordering in `run-pipeline.ts` (D-90) | A stale buffer, a swallowed failure, or ASR and render sessions coexisting (INV-12) | 81 | Every failure path awaits `chain`; earliest failure stored; `unload` before `openSession`; feed and events checked in browser |
| Cleaned audio left in the worker (§23 item 25) | Memory held after a refused clip | 69, 81 | Accepted and named; reset at the next `runChain`; V4's cancel/restart |
| IndexedDB records outliving "start over" (§23 item 26) | A person may expect their words gone | 80 | Founder decision before P80; `clear()` per repo or accept until V7 |
| `nnnoiseless` URL literals in the WASM | `check-hosts.mjs` fails the build | 69 | Chunk-scoped `NON_NETWORK_LITERALS` entry with reason, never global |
| `rej_h264_444.mp4` decodable by Chrome | The `DECODE_UNSUPPORTED` case loses its file | 63, 84, 87 | Observed on D1 and runner; D-78 fallback chain (`hi10`, then `test.fixme` + unit test) |
| `rej_silent.mp4` hallucinated words | `NO_SPEECH` case accepts | 63, 84 | Observed on D1; D-111 2 s fallback |
| Files near 400 lines (`sample_table.rs` 387, `numbers.rs`, `run-pipeline.ts` ~352, `video-source.ts` 399) | A file over the limit fails review and V4's additions | 70, 73, 81, 89 | Split by function per D-92; length audit in every milestone |
| D1 is not R1 (D-74) | Passing budgets may miss on an 8 GB laptop | 68, 69, 88 | Every reading labelled `d1` with the caveat; V9 re-checks on testers' devices |
| Reading CI without `gh` | Missed failures before merge | 87, 90 | API check-runs per SHA; human pastes job logs |
| Media cases failing mid-build (D-91) | Confusing regressions between P73 and P85 | 73–85 | SAB note: only the three D-91 cases may fail before P85; any other is a regression |

# Architectural Invariants

1. **The recording's timeline is never changed.** No sample, frame or timestamp is dropped, moved or added because of what the audio contains; `run_chain` output length equals input length; no silence detection, pause classification, gap analysis, crossfade or time map anywhere (INV-5, INV-10, TS §36).
2. **Enhancement is not editing.** The chain takes no word timings; `measure_prosody` takes them and returns numbers, never audio (§1.2).
3. **No event below its kind's threshold** is ever constructed (INV-6); `resolve` never raises a score.
4. **Deterministic outputs:** equal inputs give equal events, ids and audio bits; every sort has a total order; fixed summation order (INV-7, P8).
5. **One home per value:** DSP constants in `offcut-dsp/src/config.rs`; thresholds and lexicons in `offcut-detect/src/config.rs`; unit words in `units_lex.rs`; limits in `limits.rs` (`LIMITS` in TS); copy in `messages.ts`; OPFS paths in `opfs.ts`; `parse_quantity`/`format_quantity` only in `numbers.rs`.
6. **Pure crates stay pure:** `offcut-mp4`, `-dsp`, `-text`, `-detect` read no clock, randomness, global state or browser API; `nnnoiseless` and `ebur128` bring none of `web-sys js-sys wasm-bindgen getrandom rand wgpu`.
7. **Every rule that affects output is in Rust;** bindings and workers are glue (TS §2).
8. **The browser contacts the app origin and the asset host only** (INV-1); no request carries media, transcript, captions, event parameters, a file name or a hash prefix (INV-2); import, processing and preview never call the API except analytics (INV-3); analytics carry enums and integers only (INV-18).
9. **The main thread does no media work** (INV-17); `hashPrefix` is a WebCrypto hash of 4 MiB.
10. **ASR and render sessions never coexist** (INV-12); the audio worker holds no GPU session.
11. **PCM crosses threads by transfer, never by copy** (TS §31).
12. **The name of a dropped file is never read or stored** (P-11).
13. **V1 and V2 files are not restructured:** V3 fills bodies, adds files and rows; reopened contracts are exactly D-83, D-91, D-103.
14. **Founder recordings and lent clips never leave D1** except the committed transcripts of the founder's two clips (D-99, D-100, D-112).

# Forbidden Implementation Shortcuts

- No silence detection, pause logic, gap measurement for any purpose but prosody's z-scores, cut, crossfade or time-stretch in `offcut-dsp` — not as a stand-in, not "temporarily".
- No padding or trimming in `chain.rs` to fix a length; fix `denoise.rs`.
- No label edited to match the detector; no threshold lowered to make a clip show an event; no `rules_table` row changed without a changelog line naming the spec section.
- No test weakened, skipped, `#[ignore]`d or deleted to pass a gate (only `chain_timing` is `#[ignore]`, by design; only D-78's `test.fixme` and the D1-only skips are allowed, with their recorded reasons).
- No mocked `isConfigSupported` (D-78); no 500 MB file on disk for the oversize case (D-79).
- No new CSP host, no remote model or runtime, no `fetch` outside the four allowed files, no global `NON_NETWORK_LITERALS` entry.
- No second copy of a constant, lexicon, limit or message; no limit typed into a sentence.
- No `HashMap` in `offcut-dsp`/`offcut-detect`; no `unwrap`/`expect`/`panic!`/indexing in non-test Rust; no `any`/`!`/`@ts-ignore`/empty `catch`.
- No edit to an untouched file to make something pass; no new import edge between layers; no new use-case file (it needs a tree line and a lint edge).
- No committed fixture, founder clip, corpus clip, result file, Playwright report, `.env*`, or temporary `zz-` file; no upload of `v3testclip.mp4` or a derived fixture.
- No test key in a file, in Vercel, or in a gate build; no seed printed.
- No push, merge, tag or production database query by the agent.
- No new dependency beyond `nnnoiseless`, `ebur128` and the named dev-dependencies; no loosened `deny.toml` ban.

# Requirement → Prompt Traceability

| Requirement (BP §5, TS §28 M1.1–M1.3, plan §1.1) | Prompt(s) | Implementation | Validation |
| --- | --- | --- | --- |
| A `validate.rs` 11 rules tested against fixtures | 63, 71 | V2 file (untouched); fixtures | `probe_rejections.rs` 15 cases |
| A `probe.rs` complete (H1–H8) | 70, 71 | `probe.rs` | inline H1/H3/H4; `demux_fixtures.rs` |
| A edit lists widened (E3, E4) | 70 | `edit_list.rs` | 12 inline cases; fixtures |
| A `gen_fixtures.sh`, `manifest.json` | 63 | generator, manifest, STAMP | determinism run; step-8 probe check |
| A 14 rejection messages, `RejectionPanel` | 83 | `messages.rejections`, panel | `rejections.spec.ts`; `check-copy-codes.mjs` |
| A three repos, `sourceHashPrefix` | 80 | `*-repo.ts` | accept case (hash recomputed in Node); OPFS/IDB case |
| A `demux_fixtures.rs`, `probe_rejections.rs`, `rejections.spec.ts` | 71, 84 | test files | 12 + 15 + 16 cases |
| A corpus manifest, fetch, runner | 62, 86, 88 | `corpus/*` | TE-12 first run |
| B DSP modules (chain, loudness, limiter, denoise, high-pass, compressor) | 66, 67 | `offcut-dsp/src/*` | inline; P1–P8 tests |
| B prosody | 68 | `prosody.rs` | inline + `tests/prosody.rs` |
| B `audio_api.rs`, `audio.worker.ts`, pool row | 69 | binding, worker, pool | browser TE-13; `pipeline-preview` feed/events |
| B four DSP test files | 67, 68 | `loudness_target`, `chain_no_clicks`, `duration_preserved`, `prosody` | 10, 7+1, 7, 10 cases |
| B speech fixtures (long pause, sparse, portrait, landscape, no-events) | 63, 82 | derived recipes; README | step-8 check; "as recognized" runs |
| C full number grammar, units, sentences | 72, 73, 74 | `units_lex`, `cardinals`, `numbers`, `sentences`, `normalize` | 72 table rows, 5 properties, 8 cases |
| C four kinds, exclusions, resolution | 75, 76, 77 | detector modules | 34 `rules_table` rows |
| C `redetect_sentence` | 78, 79 | `lib.rs`, `detect_api.rs`, `RenderApi` | `redetect.rs` 7 cases |
| C precision ≥ 0.90 over ≥ 30 labeled transcripts | 64, 65, 79, 88 | labeled set; `precision_recall.rs` | gate on 30, then 31 files |
| C determinism | 78 | stable sorts | `determinism.rs` |
| C `run-pipeline.ts` complete (B1 ‖ B2, D, E, F) | 81 | use-case | browser check; `pipeline-preview` |
| C `ProcessingFeed` complete (FromTo two values) | 62, 81 | copy, protocol, component | feed cases |
| Persistence (clips, transcripts, edits) | 80, 81 | repos; step F | IDB assertions |
| Analytics: `clip_rejected`, `audio_chain` stage, `pipeline_done` four counts | 80, 81 | use-cases | rejection and events cases; Neon rows (P90) |
| Second recording through the pipeline, no metadata in its export | 82, 85, 88 | README; D1-only cases | 2 cases; bench |
| CI generates fixtures; Windows runs the rejection cases | 87 | workflows | green PR; pasted logs |
| TE-13 (native, browser) | 68, 69, 88 | timing | `experiments.md` |
| TE-12, E-3/E-4 re-read, E-10 smoke | 88 | corpus, bench | `experiments.md` |
| Spec copy-back | 61 (tree), 88 (§39.3), 89 | TS, BP, PS | `check-file-tree`; G 13.1 |
| Deployed, server first, header check, `@smoke`, tagged `v3` | 90 | CI on `main`, Human merge/tag | healthz, headers, deployed checks, `ls-remote` |

# Final Coverage Audit

- [x] **Every BP §5 / §23.1 item has a prompt** (table above), including the non-code items: 30+ labeled transcripts (64, 65), remaining speech fixtures (63), the 20-clip ASR set and 40+ corpus (carried to V9 by D-88; the tooling and smoke reading in 86, 88).
- [x] **Every file of the §4 tree is named in a prompt:** `Cargo.toml`, `deny.toml`, crate `Cargo.toml`s (62); `package.json` (63, 86); `.gitignore` (61); workflows (87); `edit_list.rs`, `sample_table.rs`, `probe.rs`, mp4 tests (70, 71); dsp files (66–68); text files (72–74); detect files (75–79); `audio_api.rs` (69), `detect_api.rs` (79); `eslint.config.js` (62); `playwright.config.ts` (84, 86); `messages.ts` (62, 83); repos (80); `load-core.ts` (69), `load-render.ts` (79); `protocol.ts` (62); `pool.ts`, `audio.worker.ts` (69); `import-clip.ts` (80), `run-pipeline.ts` (81); `RejectionPanel.tsx`, `EditorPage.tsx`, styles (83); `ProcessingFeed.tsx` (62); `tests-e2e/tsconfig.json` (86); helpers, `rejections.spec.ts` (84); `pipeline-preview.spec.ts`, `export-creator.spec.ts` (85); fixtures generator and manifest (63); `speech/README.md` (65, 82); labeled set (64, 65, 88); corpus files (62, 86); bench (86, 88); `check-copy-codes.mjs` (83); `check-external-facts.mjs` (87); `technicalspec.md` (61, 88, 89); `v2/experiments.md` (61); `docs/v3/*` (61).
- [x] **Every decision is implemented:** D-73 (61); D-74 (68, 69, 88); D-75 (61, 90); D-76, D-77 (63); D-78 (63, 84, 87); D-79 (84); D-80 (83); D-81 (80, 84); D-82 (80); D-83 (62, 81); D-84 (72, 75); D-85 (76); D-86 (78); D-87 (64, 65, 79, 88); D-88 (86, 88); D-89 (62, 66, 67); D-90 (81); D-91 (85, 86); D-92 (61, 70, 72); D-93 (84, 87); D-94 (87); D-95 (89: `ui.rs` still passes in the full gate); D-96 (68); D-97 (80, 81); D-98 (61); D-99 (62, 65, 82, 85, 88); D-100 (63, 71, 85); D-101, D-102 (72, 73); D-103 (62, 80); D-104 (73); D-105 (76); D-106 (75, 76); D-107 (75, 76); D-108 (68); D-109 (75); D-110 (66, 67, 69); D-111 (63); D-112 (86, 88, 90).
- [x] **Every experiment has a prompt:** TE-13 native (68), browser (69, 88); `DENOISE_DELAY_SAMPLES` (67); precision gate (79, 88); TE-12 (88); E-3, E-4 re-reads (88); E-10 smoke (88; E-10 carried to V9); `rej_h264_444` (63, 87); `rej_silent` (63); derived clips' events (82); second recording (82, 85, 88); bundle size (69).
- [x] **Tests are incremental:** each Rust test file lands with its code (67, 68, 71, 74, 77, 78, 79); the E2E suite lands with the UI it checks (84, 85); every prompt runs the same gate, so a regression is caught in the prompt that caused it.
- [x] **Persistence, configuration, errors, security, performance, observability, deployment** are covered: repos and records (80, 81); constants and thresholds in their single homes (66, 75); the error mapping `DspError → E_DSP`, `OpfsError → E_STORAGE_*`, `WorkerFailure` details (69, 80, 81); privacy and secrets (SAB 7–8; 61, 63, 85, 86, 88); TE-13 and the bench (68, 69, 88); analytics `stage_timing`/`clip_rejected` (80, 81, 90); CI and deploy (87, 90).
- [x] **Documentation:** changelog every prompt; experiments (61–88); README (65, 82); SCHEMA (64); spec copy-back (89).
- [x] **Prompt dependencies are valid:** every prompt needs only earlier prompts, the V3 documents and the listed Human steps; the two allowed parallel tracks (70–71, 72–74) depend only on 63 and 62.
- [x] **Exactly 30 prompts (61–90); Prompt 90 yields a deployed, verified, recorded and tagged V3** with every box of §22 and §21.3 ticked.

**Gaps in the source documents that these prompts settle.** (1) The labeled files' unit strings are not defined anywhere: Prompt 64 fixes the vocabulary in `SCHEMA.md`. (2) `rules_table.rs` cannot be committed row by row (G 7.3) while rows depend on later files: written whole in Prompt 77, finders tested inline before. (3) G 9.5/9.6 cleared site data, which deletes the model: transcripts are read by `clipId` instead (Prompt 82). (4) The guide's repository path omits `offcut/` and uses `gh`, which this machine lacks: the SAB uses the real path and the GitHub API. (5) The `transcripts` mode writes only `a02`; `a01` stays written-form (Prompt 86). (6) §23 items 26 and 27 need the founder's decision before Prompts 80 and 64.

