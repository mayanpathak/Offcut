# Offcut V5 — coding-promptsv5.md (Prompts 129–136)

Source documents: `docs/v5/v5implementation.md` (written **§n**; its decisions are **D-n**, D-194 to D-239) and `docs/v5/v5buildguide.md` (written **G n.m** for Step n.m, **G M-n** for Milestone n, **G item n** for an item of its "Read this first", **G problem n** for a row of its table "The problems V5 must fix", **G (b) n** for a row of its table of gaps). `TS §n` = `docs/technicalspec.md` (spec v1.2; §40.9 holds the seams S-8 and S-9), `PS §n` = `docs/product.md`, `BP §n` = `docs/buildplan.md`, `V4 §n` = `docs/v4/v4implementation.md`, `V4 CL` = `docs/v4/v4changelog.md`, `V5 CL` = `docs/v5/v5changelog.md`. All were read end to end against each other and against the code at `0a8aa80` on 2026-10-31; these prompts were written from that reading. Where they settle something the two documents leave open, the prompt says so and the closing section "Gaps settled by these prompts" lists it.

**How to use.** Paste the **Standard Agent Block (SAB)** below, then one prompt, into your coding agent. Run them in order: the Setup prompt, then 129 to 136. Do not start a prompt until every "Acceptance Criteria" box of the one before it is ticked and its commit exists. Steps marked **Human** are yours (see "Who does what"). V4 was Prompts 91 to 128; V5 is **eight prompts, 129 to 136**, each also labelled with its place in V5 (01/08 to 08/08), plus an unnumbered Setup. Commits are titled `V5 P<nn>: <title>` (D-194).

**Recorded deviation from TS §34.1 and BP §1.3, extended to V5.** Those sections give each file its own specification. As in V1 to V4, the founder accepts the multi-file prompt format; the per-file contract is honoured by naming every file and implementing every signature §7 to §16 give exactly (§ Appendix A is the per-file brief).

**The order is the guide's, one prompt per phase.** Five rules decide it. **Speed first** (P129): every later prompt benefits. **The judge before the work it judges** (P130): the verifier exists before anything it will judge changes. **The stereo fix inside the verifier's prompt**: check 7 cannot pass without it, and a real export is read before and after. **The proof before the cache** (P133, P134): a file proved correct is the only one worth caching. **Measure on the finished build** (P136).

**Speed is part of the spec.** V4 ran the full media suite 22 times (about 8 hours). V5 runs it once, in P136 (D-197). So unlike V4's prompts, these do **not** all end in the same gate: each runs its tier (SAB, "The gate"). A prompt that fails a targeted run fixes it there; it does not widen to a full run. Three pushes before the merge, at P129, P134 and P136; only the last is waited on (D-198).

**One small human gate.** In P136 the founder looks at the done state's capture and listens to an export (D-235), and does the merge and the tag. **D1 is the reference machine (D-202):** every reading is taken on D1 by the agent and recorded under the label `d1`.

**Effort (for the human).** Each prompt has an **Effort** line under its heading: the reasoning effort to set on the coding agent before pasting it. **High** (5 prompts: 129-132, 134) is for shared test state, numerical checks, binary formats, a restart inside a loop, and a key that decides who gets which file. **Medium** (4 prompts: Setup, 133, 135, 136) is for records, exactly specified cases and components, and runs. When in doubt the rating is High. The line is a note for you; it changes no instruction to the agent.

**Design skill.** In P135, if the coding agent has a frontend or design skill available, it loads it before writing interface code. The tokens, primitives and copy of V4 still win over any default a skill suggests.

---

# Standard Agent Block (SAB) — prepend to every prompt

**Every terminal (Git Bash, never PowerShell or WSL)**

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut && set -a; . ./.env; set +a      # the repository is offcut/, not sh2clips/; .env has the 15 local values only
export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:/c/Users/Mayan/AppData/Local/Microsoft/WinGet/Links:$PATH"
export BENCH_DEVICE=d1
BASE=0a8aa80                                                                # the V5 code baseline every frozen-file check compares with
git branch --show-current                                                  # v5-build, always (the Setup prompt creates it)
python --version                                                           # 3.13.x; plain python on D1 is otherwise 3.9 with no NumPy
```

Docker Desktop must be running before `cargo test --workspace` (the server's tests use the local Postgres on port 9000), and closed for the bench. `ffmpeg -hide_banner -version | head -1` prints 9.0.x; `gh auth status` shows a login.

**Before modifying anything**

1. Read the plan sections and guide steps this prompt cites, and every file it touches. Read the previous prompt's entry in `docs/v5/v5changelog.md`.
2. Search the repository for existing types, helpers and constants before creating any (`git grep -n "<name>"`). Reuse them. Never create a second home for a value that has one (§ Appendix C names the home of every V5 constant).
3. Verify that the previous prompts' outputs exist and behave as this prompt assumes (each prompt lists what to verify under "Project State Before This Prompt"). If not: decide whether it is a bug or an intended boundary, make the smallest compatible correction, and report it in the changelog entry.
4. **Precedence.** A decision D-194 to D-239 of §2 is binding, also where it refines a TS section (it says which); D-1 to D-193 stand unless a V5 decision replaces them. Otherwise TS (v1.2) wins over the plan, and PS over both. The guide gives the order of work and the commands. If two sources disagree in a way none of these settles, or a signature is missing, **stop and list it**. Do not guess (TS §34.1).

**During implementation**

1. Implement only this prompt's scope ("Implement"); leave everything under "Do Not Implement Yet" alone even if it looks easy. Create only the files the prompt names. A new file under `crates/`, `server/`, `web/`, `scripts/`, `verify/`, `corpus/`, `bench/` or `.github/` that the TS §5 tree does not list fails `node scripts/check-file-tree.mjs`: exactly four need a new row, in the commit that creates each (`verify/make_watermark_mask.py`, `verify/test_verify_mp4.py`, `web/src/workers/render/export-loop.test.ts`, `crates/offcut-mp4/tests/mux_reorder.rs`).
2. **Untouched in V5** (§4, last paragraph; the frozen check below enforces it): everything under `server/`; `crates/offcut-{types,api-types,plan,scene,render,entitlement,text,detect,wasm-core,wasm-render}/`; `offcut-dsp/src/{chain,config,program,prosody}.rs`; `offcut-dsp/tests/{chain_no_clicks,duration_preserved,program_identity,prosody}.rs`; `web/vercel.json`; `web/src/{gen,net,models,platform,analytics}/`; `web/src/workers/{audio,media,asr}.worker.ts`, `web/src/workers/{media,asr}/`, `web/src/workers/{rpc,pool}.ts`; `web/src/workers/render/{preview-job,preview-loop,video-source,frame-decoder}.ts`; `web/src/usecases/{control-preview,review-update}.ts`; `web/tests-e2e/preview-sync.spec.ts`; `fixtures/{golden,labeled,speech}`; `corpus/`; `render.yaml`. `export-machine.ts`'s table does not change: its comment does (P134).
3. **The reopened contracts** are exactly these, each said as **REOPENED CONTRACT** in the changelog entry of the prompt that touches it, both sides in one commit, both copied to TS §14.2 in P136: `RenderWorkerApi.exportClip`'s `out48` → `prog48` (D-214, P132); `RenderWorkerApi` + `exportProfile` (D-220, P134).
4. **Expectations V5 may rewrite**, and no others: `a_1khz_sine_of_peak_minus_20_reads_minus_23` in `loudness.rs`, renamed `..._reads_minus_20` (P130); `pink_noise_reaches_the_target`'s input level, if its "not capped" assertion fires, and only then (P130); `export-creator.spec.ts`'s verifier assertions and comments (P130); `mux_roundtrip.rs`'s one comment (P131); `review-edit.spec.ts`'s grouping and how each case gets its `page`, never what a case asserts (P129). Any other failure is a regression of the prompt that caused it.
5. No source file over 700 lines above its test module. Rust non-test code: no `unwrap`, `expect`, `panic!`, slice indexing, `HashMap` in a pure crate, clock, randomness, global state or I/O in a pure crate. TypeScript: no `any`, no `!`, no `@ts-ignore`, no empty `catch`, no `as <Brand>` outside a `MINTING` helper (the casts `export-loop.ts` already has stay), no inline style (CSP `style-src 'self'`). No disabled lint, no `#[allow]`/`eslint-disable` added to pass a check, no weakened, skipped or deleted test. **No new dependency** in `Cargo.toml` or `package.json` (§3.2).
6. **Goldens and display-list snapshots do not change.** V5 changes neither the scene nor the renderer. A changed `.png` or `.snap` in the diff is an error to explain, never to accept.
7. Scripts are POSIX `sh` with LF line endings. A temporary file is named `zz-…` or `zz_…` and is deleted before the gate; nothing temporary is committed. A temporary Playwright check needs `web/tests-e2e/zz-<name>.spec.ts` plus a temporary `web/zz-pw.config.ts` that spreads the real config and sets `projects: [{ name: "zz", testMatch: "zz-*.spec.ts", use: { channel: "chrome" } }]`. Files written by Python on Windows are normalised to LF (`sed -i 's/\r$//' <file>`) before `git add`.
8. A TODO carries the version that removes it (`V6:`, `V6.2:`). User-facing text exists in `copy/messages.ts` only (INV-15), including tooltips, `aria-label`s and toasts; numbers are interpolated. `fetch` exists in the four allowed files only; V5 adds no request. No cut, mix or non-identity `TimeMap` is constructed outside tests.
9. **Privacy.** Never stage, commit, upload or paste anywhere: `testclips/**`, `fixtures/.gen/**`, `fixtures/.cache/**`, `corpus/.clips/**`, `corpus/.results/**`, `web/test-results/**`, `web/playwright-report/**`, `.env*`. `testclips/v3testclip.mp4` carries a GPS location and never leaves D1. No word of a clip is printed in a test, a report or a reply. **An exported file is never part of a report** (TS §22.5): `OFFCUT_KEEP_EXPORTS` is a directory outside the repository, and it is deleted when the prompt ends. A verifier line names a tag's key, never its value.
10. **Secrets.** No seed, token, key or connection string is printed, logged or written to a file. `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` is exported only in a terminal that builds for the media suites, from `ci.yml`'s `E2E_TEST_PUBLIC_KEY` (G Quick reference), never in a `.env*` file, and unset before a gate build. Production values live in `.env.deploy`; the agent never queries the production database.
11. **Long runs** (`pnpm e2e:media` about 30-40 minutes in full, `review-edit.spec.ts` about 15-20, the bench about 14, `gh pr checks --watch` about 30): start detached, `( cmd > "$LOG" 2>&1; echo "exit=$?" >> "$LOG" ) > /dev/null 2>&1 &`, then poll the log with a bounded loop, and read only its tail and failures. `sleep` is blocked in the Bash tool; wait with `ping -n 2 127.0.0.1`. Append to the changelog with the Edit tool, or by writing a scratchpad file and `cat >>`; never a heredoc holding backticks (Bash quoting breaks on them).
12. **Branch and push (D-194, D-198).** Commit on `v5-build` only (`git branch --show-current` before every commit). Push to `origin v5-build` yourself only where the prompt says. Read CI with `gh pr checks <pr>`; read a failing job with `gh run view <run-id> --log-failed`. Never push to `main`, never merge, never tag, never mark the pull request ready: those are **Human** steps. Do not perform any other **Human** step (a founder's decision, reading or judgement; a dashboard; a production database read). List it in your reply.

**When existing code is inconsistent with the plan.** Do not rewrite it blindly. If it blocks this prompt, fix the minimum and record why. If it does not, leave it and report it under "Found, not fixed" in the changelog entry.

**After implementation: the gate.** Every prompt ends with its tier, then the checks every prompt shares. A new or changed test runs alone first (Tier 0).

| Tier | Runs | Used by |
|---|---|---|
| 1 | `pnpm check`; `cargo test -p` each crate touched; `pnpm --filter web exec vitest run` | Every prompt |
| 2 | Tier 1, a build, and only the cases the prompt names: `pnpm e2e:media <spec> --grep "<case>" --max-failures=1` (write `[.]`, never a backslash: `pnpm` doubles it on Windows). Only `web/src` changed: `pnpm --filter web exec vite build --mode test`. Rust changed: `pnpm build:e2e` | 129, 130, 132, 133, 134, 135 |
| 3 | `pnpm check && pnpm fixtures && pnpm test`, `pnpm e2e` | 129, 134 |
| 4 | Tier 3, then `pnpm e2e:media` in full on a keyed build, then CI, waited for | 136 |

Shared by every prompt, in this order:

```bash
git branch --show-current                                                  # v5-build
git status --porcelain | grep -c "zz[-_]"                                  # 0: nothing temporary is left
env | grep -c VITE_ENTITLEMENT_TEST_PUBLIC_KEY                             # 0: unset before the commit
git diff --name-only "$BASE" -- \
  server crates/offcut-types crates/offcut-api-types crates/offcut-plan crates/offcut-scene crates/offcut-render \
  crates/offcut-entitlement crates/offcut-text crates/offcut-detect crates/offcut-wasm-core crates/offcut-wasm-render \
  crates/offcut-dsp/src/chain.rs crates/offcut-dsp/src/config.rs crates/offcut-dsp/src/program.rs crates/offcut-dsp/src/prosody.rs \
  crates/offcut-dsp/tests/chain_no_clicks.rs crates/offcut-dsp/tests/duration_preserved.rs \
  crates/offcut-dsp/tests/program_identity.rs crates/offcut-dsp/tests/prosody.rs \
  web/vercel.json web/src/gen web/src/net web/src/models web/src/platform web/src/analytics \
  web/src/workers/audio.worker.ts web/src/workers/media.worker.ts web/src/workers/asr.worker.ts web/src/workers/media web/src/workers/asr \
  web/src/workers/rpc.ts web/src/workers/pool.ts \
  web/src/workers/render/preview-job.ts web/src/workers/render/preview-loop.ts web/src/workers/render/video-source.ts web/src/workers/render/frame-decoder.ts \
  web/src/usecases/control-preview.ts web/src/usecases/review-update.ts \
  web/tests-e2e/preview-sync.spec.ts fixtures/golden fixtures/labeled fixtures/speech corpus render.yaml
# the frozen check (G 0.5): no output
git ls-files --cached | grep -E '^(testclips|fixtures/\.gen|fixtures/\.cache|corpus/\.clips|corpus/\.results|web/test-results|web/playwright-report)/|(^|/)\.env'   # no output
```

1. Read the counts the tier ran: Rust passed, failed and ignored (`cargo test --workspace 2>&1 | awk '/^test result:/ {p+=$4; f+=$6; i+=$8} END {print p" passed, "f" failed, "i" ignored"}'`, from that run's log alone), Vitest passed (`pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"`), Playwright passed. None may fall below the previous entry's, except a case SAB rule 4 rewrites by name. The floor (Entry 1) is V4's close: 646 passed, 0 failed, 4 ignored; 175 Vitest; 12 Playwright; media 77 passed + 1 skip.
2. `pnpm e2e:media` in full is run once, in P136. After it, a follow-up fix re-runs the affected spec, unless it touches the pipeline, a worker or the renderer (D-197).
3. Fix every regression you caused. Tick each acceptance box with its evidence. Leave a box that depends on a **Human** step unticked and say so.
4. Append a short entry to `docs/v5/v5changelog.md` (D-195): `## Entry <n> - <date> - P<nn>: <title>`, then what changed, what differs from the plan and why, **REOPENED CONTRACT** lines, the counts, readings labelled `d1`, what is open, "Found, not fixed". Rewrite the two tables at the top when an item opens or closes. The entry is in the same commit as the work (the changelog rule).
5. Commit on `v5-build` as the prompt says, with the trailer `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Push only where the prompt says. A red run found at the next prompt's start is fixed first, with its own commit.
6. Reply with: files created and modified, commands run with their results, the CI result if read, deviations, **Human** steps still open, issues found but not fixed.

---

# Who does what

The agent writes files, runs commands (including on D1, which is the machine it runs on), drives Chrome through Playwright, commits on `v5-build`, pushes to `origin v5-build` where a prompt says and reads CI with `gh`. A merge, a push to `main`, a tag, opening or marking a pull request ready, a dashboard, a production database read, and a founder's decision, words or judgement are **Human** steps.

| When | Human step | Plan / guide |
| --- | --- | --- |
| Before Setup | **Push the tag `v4`.** On 2026-10-31 `git ls-remote --tags origin` listed `v2` and `v3` only. Setup cannot pass without it: `git tag -a v4 -m "V4: the look and the editor" 0a8aa80 && git push origin v4` | §1A box 1; G 0.1, problem 7 |
| Setup | **Say which of V4's 35 open boxes you vouch for** (design review 2, the Neon rows, the deployed walk-through are unrecorded). What nobody vouches for stays unticked and is named | §1A box 1; G 0.2, problem 8 |
| Setup | **Open a draft pull request `v5-build` → `main`**, and tell the agent its number. CI runs on pull requests only | §1A box 4; G 0.4, problem 9 |
| Prompt 136 | When the agent reports the pre-merge run green: `gh pr ready <pr>` (or say "ready" and the agent runs it), **merge**, and watch the `main` run | D-194; G 8.6 |
| Prompt 136 | **One look** at `~/v5-review/` (the done state at two widths) and **one listen** to an export from the agent's keyed build. D-204 is a product change | D-235; G 8.7 |
| Prompt 136 | **Push the tag `v5`** on the merge | D-194; G 8.8 |

---

# Project Implementation Map

## Architecture Summary

V5 is "Export that an independent tool accepts" (BP §7; TS §28 M1.6), together with the seams S-8 and S-9 of TS §40.9 and the speed rules the founder approved on 2026-10-31. It starts from the deployed V4 at `0a8aa80`, in which a clip is processed, previewed and edited in a premium editor, and exported with a loop on the recording's time, with no render cache, a placeholder `export_done` and a six-check verifier. V5 makes the export correct by an outside measure, and fast the second time:

- **The verifier.** Checks 7 to 11 (loudness, sync and timeline, watermark, bitrate, metadata), a mask drawn independently of the renderer, and a self-test that runs in CI without a browser (D-203 to D-210).
- **The stereo fix.** The chain measures its mono programme as `DualMono`, so a stereo file reads -14 LUFS rather than about -11 (D-204).
- **The muxer.** `ctts` version 0 with a video `elst` when frames arrive out of order, a real B-frame fixture, the `moov` overflow guard tested (D-211, D-212).
- **Export on program time (S-8).** The loop reads `source_at`, the audio is `prog48.f32`, and an encoder that fails restarts on the next ladder entry (D-213 to D-219, D-238).
- **The render cache (S-9).** SHA-256 over canonical JSON of ten inputs, with the profile kind from the token **the worker verified** and a computed renderer version (D-220 to D-228).
- **The gate.** `B_STORAGE_LOW` in `blockers.ts`; a blocked export says why (D-229 to D-231).
- **What the person sees (J11).** `ChangeSummary`, the question, real analytics (D-232 to D-237).
- **Speed.** Tiered gates, a shared clip in `review-edit.spec.ts`, a docs-only CI skip, `BENCH_RUNS` (D-196 to D-201).

Every rule that affects output stays in Rust or in the render worker; TypeScript on the page is glue (TS §2). The server does not change. Nothing beyond S-8 and S-9 is reserved for V6.x (D-239).

## Major Components

| Component | Location | V5 role |
| --- | --- | --- |
| Verifier | `verify/` | Checks 7-11, mask and its tool, self-test |
| Voice chain | `crates/offcut-dsp/src/loudness.rs` | `DualMono`; two test files |
| Muxer | `crates/offcut-mp4/src/{mux,mux_boxes}.rs` | `ctts`, video edit list; `mux_reorder.rs` |
| Fixtures | `fixtures/gen_fixtures.sh`, `manifest.json` | `ok_bframes.mp4`: 26 fixtures |
| Export worker | `web/src/workers/render/{export-loop,export-job,encoders,opfs-sink}.ts` | `source_at`, `prog48`, attempts, ladder, `reset` |
| Worker protocol | `web/src/workers/{protocol,render.worker,rpc-worker}.ts` | Two reopened contracts; the dev fault |
| Cache | `web/src/persistence/{render-cache-repo,schema,opfs}.ts`, `web/vite.config.ts` | Key, row, version, `prog48.f32`, `freeBytes` |
| State | `web/src/state/{blockers,export-store}.ts`, `machines/export-machine.ts` | `forExport(room?)`, new actions; three test files |
| Use-cases | `web/src/usecases/{start-export,answer-post-export,run-pipeline,import-clip,dev-faults}.ts` | §13; the answer; `writeProgram`; clears the cache |
| Interface | `web/src/ui/components/{ChangeSummary,PostExportQuestion,ExportButton,ExportProgress}.tsx`, `ui/layout/SidePanel.tsx`, `copy/messages.ts` | The outcome; the blocker's words; time left |
| Tests | `web/tests-e2e/` | `timeline-preserved.spec.ts`, `export-creator.spec.ts` with 11 checks, the shared clip |
| Tooling and CI | `.github/workflows/ci.yml`, `bench/device-bench.ts`, `scripts/check-{copy-codes,hosts}.mjs` | `changes` job, self-test step, `BENCH_RUNS`, markers |
| Records | `docs/v5/`, `docs/technicalspec.md`, `docs/buildplan.md` | Changelog, experiments, copy-back |

## Dependency Graph

```text
Setup: v4 tag read, V4's closing entry, v5-build, baseline, [Human: draft PR]
        │
        ▼
P129 speed: shared clip, docs-only skip, BENCH_RUNS, known issues 33, 36 ── push 1 (not waited on)
        │
        ▼
P130 verifier 7-11, mask, self-test, stereo fix (a real export read before and after)
        │
        ├─► P131 muxer: ctts, video elst, overflow, ok_bframes (no browser)
        │        │
        │        ▼
        └─► P132 (S-8) source_at, prog48.f32, attempts, ladder, dev fault, export-loop.test
                 │
                 ▼
        P133 timeline-preserved on both pause fixtures, 11 checks
                 │
                 ▼
        P134 (S-9) cache on the verified profile, version, gate, three unit-test files ── push 2 (not waited on)
                 │
                 ▼
        P135 ChangeSummary, the question, real export_done, the capture
                 │
                 ▼
        P136 bench, experiments, copy-back, full media once ── push 3 (waited on)
                 ─► [Human: merge] ─► deployed checks ─► [Human: look, listen, tag]
```

## Implementation Phases

| Guide phase | Plan phase | Prompt |
| --- | --- | --- |
| 0 Setup | - | Setup |
| 1 The speed work | A | 129 |
| 2 The verifier and the stereo fix | B | 130 |
| 3 The muxer | B | 131 |
| 4 Program time and the ladder | B | 132 |
| 5 The proof | C | 133 |
| 6 The cache and the gate | C | 134 |
| 7 What the person sees | D | 135 |
| 8 Measure, final gate, merge, deploy, tag | D | 136 |

## Prompt Dependency Graph and Table

Strictly sequential.

| Prompt | V5 # | Major deliverable | Depends on | Produces | Guide | Tier | Push |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Setup | 00 | Tag read, V4's Entry 40, `v5-build`, baseline, frozen check defined | Human (tag, boxes, PR) | `v5-build`, Entry 1 | Phase 0 | Counts only | Yes |
| 129 | 01/08 | `openShared`, `resetClip`, `sendQueuedAnalytics`, `forget`; `review-edit` on 4 loads; `changes` job; `BENCH_RUNS`; issues 33, 36 | Setup | Every later run faster | Phase 1 | 2, 3 | First, not waited on |
| 130 | 02/08 | `DualMono`; mask and its tool; checks 7-11; self-test; `verifyMp4(source)`, `expectAllChecks` | 129 | 11 checks on real files | Phase 2 | 2 | No |
| 131 | 03/08 | `ctts`, video `elst`, `ok_bframes.mp4`, `mux_reorder.rs`, overflow tests | 130 | A correct file with B-frames | Phase 3 | 1 | No |
| 132 | 04/08 | `prog48.f32`, `exportClip`'s `prog48`, `source_at`, `attempt`, `supportedConfigs`, `reset`, `failEncoderAt`, `export-loop.test.ts` | 131 | S-8 | Phase 4 | 2 | No |
| 133 | 05/08 | `timeline-preserved.spec.ts`, `watchExport` in helpers | 132 | BP §7's demo as a test | Phase 5 | 2 | No |
| 134 | 06/08 | `render-cache-repo.ts`, `__RENDERER_VERSION__`, `exportProfile`, `forExport(room?)`, `start-export.ts` of §13, three test files | 133 | S-9 and the gate | Phase 6 | 2, 3 | Second, not waited on |
| 135 | 07/08 | `ChangeSummary`, `PostExportQuestion`, `answer-post-export`, `editValues`, `fakeExportDone`, the capture | 134 | J11 and real analytics | Phase 7 | 2 | No |
| 136 | 08/08 | Bench, `experiments.md`, copy-back, full media once; merge, deploy, look, listen, tag | 135, Human | V5 live and tagged | Phase 8 | 4 | Third, waited on |

---

# Prompts

Every prompt ends with the SAB gate at its tier. The "Acceptance Criteria" are what is specific to the prompt; the SAB gate is always also required.

## Setup prompt (V5 00) — The `v4` tag, V4's closing entry, `v5-build`, the baseline

**Effort: Medium.** Records and runs; the judgement on V4's boxes is the founder's.

### Objective

Prove V4 is closed on the record, create `v5-build` from `origin/main` with the V5 documents as its first commit, record the baseline counts as the floor every later prompt compares with, and have a draft pull request open so CI runs.

### Project State Before This Prompt

- The tree is on `v4-build`, with `docs/v5/` untracked and `docs/technicalspec.md` modified (five §5 rows for `docs/v5/`). `origin/main` is `0a8aa80`.
- `v4changelog.md` ends at Entry 39. `v4implementation.md` has 35 unticked boxes.
- `fixtures/.gen/speech/` holds both pause fixtures; `py -3.13 -c "import numpy"` reads 2.5.3 (§1A boxes 6, 7).
- **Human, before this prompt:** the tag `v4`. On 2026-10-31 it was not on `origin`.

### Why This Comes Here

V4 §22.3 says V5 must not start until V4 is tagged, and every frozen-file check and count compares with this baseline (§1A).

### Scope

#### Implement

- G 0.1: `git ls-remote --tags origin v4` prints a line. If not, **stop** and ask the founder for the command in "Who does what".
- G 0.2: V4 CL Entry 40 ("V4 closed", the merge `0a8aa80`, the date of the tag, the founder's words of 2026-10-31, Entry 39's final counts). Ask the founder which of the 35 boxes they vouch for; tick those in `v4implementation.md`; name in Entry 40 every box left unticked. Close V5 CL open items 1 and 2.
- G 0.3: `git switch -c v5-build origin/main`, carrying the documents.
- G 0.4: the baseline. Entry 1 of `v5changelog.md` as a table (Item, Value): baseline SHA; Rust 646/0/4; Vitest 175; Playwright 12; media 77 + 1 skip, carried from V4's final gate and not re-run, since the code is the merge's; the app-shell size from `node scripts/check-hosts.mjs` after `pnpm build`. Commit, push with `-u`.
- **Human:** the draft pull request; record its number in Entry 1. Close V5 CL open item 3.
- G 0.5, G 0.6: the frozen check and the tiers exist in the SAB; run the check once and paste its (empty) output into Entry 1.

#### Do Not Implement Yet

- Any code. `pnpm e2e:media` (D-197). The `changes` job (P129).

### Existing Components To Reuse

- `scripts/check-hosts.mjs` for the shell size; V4's Entry 39 for the counts.

### Files To Create

None. (`docs/v5/v5implementation.md`, `v5buildguide.md`, `coding-promptsv5.md`, `v5changelog.md` exist, uncommitted.)

### Files To Modify

- `docs/v4/v4changelog.md` (Entry 40), `docs/v4/v4implementation.md` (boxes), `docs/v5/v5changelog.md` (Entry 1, tables), `docs/v5/v5implementation.md` (§1A boxes ticked).

### Architecture Requirements

- The branch comes from `origin/main`, not `v4-build`. `git merge-base v5-build origin/main` is `0a8aa80` or later (D-194).

### Data Types / Interfaces

None.

### Implementation Requirements

#### 1. Order
Tag read → founder's answer on the boxes → branch → Entry 40 → baseline → commit → push → founder's draft pull request → number recorded.

#### 2. A box nobody vouches for
Stays unticked, and Entry 40 names it. Do not tick to clear the list.

### Error Handling

- A count that differs from V4's close: record it as it is, as the floor, and report it. Do not fix V4 code here.
- `origin/main` past `0a8aa80` (a hotfix): the new SHA becomes `BASE`, here and in the SAB's line.

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

No key in Entry 1. `.env*` not staged.

### Testing Requirements

#### Unit Tests
None new.

#### Integration Tests
`pnpm check`, `pnpm fixtures`, `pnpm test`, `pnpm e2e`.

#### Failure-Path Tests
None.

### Validation Commands

```bash
git ls-remote --tags origin v4                                    # one line
git merge-base v5-build origin/main | cut -c1-7                   # 0a8aa80 or later
git log --oneline -1                                              # V5 setup: branch, baseline, V4's closing entry
gh pr view <pr> --json state,isDraft,baseRefName,headRefName --jq '.'   # OPEN, true, main, v5-build
```

### Acceptance Criteria

- [ ] **Human:** `v4` is on `origin`.
- [ ] V4 CL Entry 40 is written; every unticked box of `v4implementation.md` is named in it.
- [ ] `v5-build` is pushed, holding the V5 documents and the TS §5 rows.
- [ ] Entry 1 holds the baseline SHA, the counts as printed and the shell size.
- [ ] **Human:** the draft pull request is open and its number recorded.

### Agent Instructions

Before: read §1A, §1.1, D-194, G Phase 0. During: stop at the tag and at the boxes; do not guess the founder's words. After: commit `V5 setup: branch, baseline, V4's closing entry` and push. Report what is open.

### Expected Result

V4 closed on the record, `v5-build` pushed, a draft pull request open, the floor recorded.

---

## Prompt 129 (V5 01/08) — The speed work: a shared clip, the docs-only skip, `BENCH_RUNS`

**Effort: High.** Shared state between tests fails in ways that look like app bugs, and a workflow's job graph can only be read after a push.

### Objective

Make every later run faster, with no gate lost: `review-edit.spec.ts` shares one processed clip across 12 of its 16 cases, a pull request that changes only documents skips the heavy CI jobs while the secret scan still runs, the bench takes a run count, and known issues 33 and 36 get their one-line fixes.

### Project State Before This Prompt

- `v5-build` holds the Setup commit; a draft pull request is open.
- `web/tests-e2e/review-edit.spec.ts` is one `test.describe` of 16 cases, each calling `openClip(page, fixture)`, about a minute each.
- `helpers/fake-api.ts`'s `FakeApi` has `requests` and `eventsOf`. `helpers/fixtures.ts` has `saves`, `settled`, `trackSaves`, `openTab`, `seekTo`, `readIndexedDb`, `opfsList`, `expectTestBuild`, `ensureModelCached`, `routeAssets`, `dropClip`.
- `ci.yml`'s `ci` job does the scan inside; `e2e-media` `needs: ci`; `deploy-api` `needs: [ci, e2e-media]`.
- `bench/device-bench.ts` has `const RUNS = 10`.

### Why This Comes Here

The next seven prompts run `review-edit`-style cases and CI; speed compounds from the first one (D-196, D-201). It changes no product code, so a failure here cannot be confused with a V5 behaviour.

### Scope

#### Implement

- G 1.1: `FakeApi.forget()`; `sendQueuedAnalytics(page)`; `openShared(browser, info, fixture)`; `resetClip(page, api)`.
- G 1.2: `review-edit.spec.ts` in two groups. Cases 1-4, 7, 9-13, 15, 16 on the shared clip; cases 5, 6, 8, 14 on their own. Numbers, titles and what each asserts unchanged. Header comment rewritten.
- G 1.3: D-236's two lines in `model-download.spec.ts`.
- G 1.4: `ci.yml`'s `changes` job, `ci`'s `needs` and `if`, gitleaks moved, `fetch-depth` dropped from `ci`; `e2e-media.yml`'s stale "all 30 cases".
- G 1.5: `BENCH_RUNS`.
- G 1.6, G 1.7: the runs and the guard drills.

#### Do Not Implement Yet

- The verifier's self-test step in `ci.yml` (P130). `OFFCUT_KEEP_EXPORTS` (P130). Anything in `web/src`.

### Existing Components To Reuse

- `openClip` of `review-edit.spec.ts` is the body of `openShared`. `saves`, `settled`, `seekTo`, `openTab`, `readIndexedDb`, `opfsList`, `patient`.
- `clip-store.ts`'s `empty().edit` is the value the clean state compares with: `{ word_edits: {}, event_overrides: {}, style: "clean", crop_offset: 0, caption_position: 1, highlight_numbers: true }`.
- `analytics/client.ts` flushes on `visibilitychange` when `document.visibilityState === "hidden"`. `PreviewPlayer.tsx` also listens, on `document.hidden`, which the override does not change, and the preview is paused first anyway.

### Files To Create

None.

### Files To Modify

- `web/tests-e2e/helpers/fake-api.ts`, `helpers/fixtures.ts`, `review-edit.spec.ts`, `model-download.spec.ts`; `bench/device-bench.ts`; `.github/workflows/ci.yml`, `e2e-media.yml`; `docs/v5/v5changelog.md`.

### Architecture Requirements

- `test.describe.configure({ timeout: patient(420_000) })` on the shared group: `beforeAll` has the test's timeout, and the clip takes minutes.
- The shared group is **not** `serial`: a failed case must not stop the others. Playwright starts a new worker after a failure, and `beforeAll` runs again: a new clip.
- A shared case reads `page` and `api` from `shared`, never the `{ page }` fixture: that would open a second, empty page.
- `changes` needs `fetch-depth: 0`, for the scan and for `git diff origin/$GITHUB_BASE_REF...HEAD`. On `push`, `code` is always true, so deploys never depend on the diff.

### Data Types / Interfaces

```ts
// helpers/fake-api.ts
export type FakeApi = {
  requests: RecordedRequest[];
  eventsOf(name: string): unknown[];
  /** Forgets everything recorded so far: `eventsOf` counts from here (V5 §6.3). */
  forget(): void;
};

// helpers/fixtures.ts
export type SharedClip = { context: BrowserContext; page: Page; api: FakeApi };
export function openShared(browser: Browser, info: TestInfo, fixture: string): Promise<SharedClip>;
/** Sends what analytics holds now, without `page.clock`: the page is told it is hidden (V5 §6.3). */
export function sendQueuedAnalytics(page: Page): Promise<void>;
/** A clean clip, or the case fails by name (V5 §6.3). */
export function resetClip(page: Page, api: FakeApi): Promise<void>;
```

### Implementation Requirements

#### 1. `sendQueuedAnalytics`
```ts
await page.evaluate(() => {
  Object.defineProperty(document, "visibilityState", { value: "hidden", configurable: true });
  document.dispatchEvent(new Event("visibilitychange"));
  delete (document as unknown as { visibilityState?: string }).visibilityState;
});
await page.waitForTimeout(300);   // for the request to reach the route
```
`flushAnalytics` needs `page.clock`, which this page does not install: the fake clock would touch every timer of 13 cases.

#### 2. `resetClip`, in this order (§6.3)
1. Escape, for a word field; if Pause is visible, press it and wait for Play.
2. `undo = page.getByRole("banner").getByRole("button", { name: messages.editor.undo, exact: true })`: the toast after a redo offers an Undo of its own. While enabled: read `saves(page)`, click, `settled(page, since)`. It has no test id.
3. `seekTo(page, 0)`; `openTab(page, "Captions")`; `page.mouse.move(0, 0)` (a hovered toast does not time out); wait until `getByTestId("toasts")` is gone (`ToastHost` renders nothing with no toast) within `patient(30_000)`.
4. `sendQueuedAnalytics(page)`, then `api.forget()`.
5. **Assert**: Undo disabled; `preview-failed` count 0; `readIndexedDb(page, "edits", clipId)`'s `value.editState` deep-equals the empty edit (`clipId` from `opfsList(page, "clips")`). Do not open the Style tab: it would start a thumbnail job.

#### 3. The cases
Move case 5, 6, 8, 14 bodies unchanged into the second group, where `openClip` stays. In the first, each case begins `const { page, api } = shared;` and loses its `openClip` line. A case that used `const api = await openClip(…)` keeps the name.

#### 4. `changes`
§19.1's YAML. gitleaks into `$RUNNER_TEMP`, by `GITLEAKS_VERSION` and `GITLEAKS_SHA256` of the workflow's `env`, refused unless its checksum matches, as "Install the build tools" does today. The step's comment names the reason.

#### 5. `BENCH_RUNS`
```ts
const RUNS = Number(process.env.BENCH_RUNS ?? "10");
if (!Number.isInteger(RUNS) || RUNS < 1) { throw new Error("BENCH_RUNS must be a whole number from 1"); }
```
The test's name and the header follow. `BENCH_TIMEOUT_MS = (RUNS + 1) * 11 * 60_000`.

### Error Handling

- `resetClip`'s clean-state assertion fails: read what stayed. Fix the reset, or move the case to its own load (D-201), and name it in the entry.
- A case's own assertion fails: it assumed a clean page in a way the reset does not give. Read it; do not rerun blindly.

### Concurrency / Async Requirements

`sendQueuedAnalytics` before `forget()`, always: the flush is asynchronous and the route records on arrival. Undo steps one at a time, each `settled`, or the next is refused while the clip is `updating`.

### Persistence Requirements

None. The shared clip stays in its context's OPFS and IndexedDB; the context closes in `afterAll`.

### API Requirements

None.

### Security Requirements

- `resetClip` prints no word. `changes` has `contents: read` only (the workflow's default) and needs no secret.

### Testing Requirements

#### Unit Tests
None new (`tests-e2e` helpers have none).

#### Integration Tests
`pnpm e2e:media review-edit.spec.ts` once, whole. `model-download.spec.ts` is not re-run for one wait: its cases run in CI's `e2e-media`, which is what the issues are about.

#### Failure-Path Tests
G 1.7's two drills. The `diff` script on both kinds of diff (G 1.4).

### Validation Commands

```bash
pnpm --filter web exec playwright test --project=media --list review-edit.spec.ts | tail -1   # 16 tests
BENCH_RUNS=5 pnpm --filter web exec playwright test --project=bench --list | tail -2           # the name says 5
BENCH_RUNS=x pnpm --filter web exec playwright test --project=bench --list 2>&1 | tail -2      # the error
git grep -c "gitleaks" -- .github/workflows/ci.yml                                             # only in env and the changes job
```

### Acceptance Criteria

- [ ] All 16 cases pass in one run, on 5 clip loads; the file's time is in the entry against V4's.
- [ ] Case numbers, titles and assertions are unchanged (`git diff` read), or a move to an own load is named.
- [ ] `resetClip` failed by name in the drill; `sendQueuedAnalytics`'s drill failed case 13.
- [ ] The `diff` script gives `code=true` and `code=false`.
- [ ] `BENCH_RUNS` accepts 5 and refuses `x`; the default is 10.
- [ ] Tier 3 is green; the frozen check prints nothing; counts equal Entry 1's.

### Agent Instructions

Before: read §6, §19.1, D-196 to D-201, D-236, G Phase 1, and the whole of `review-edit.spec.ts`. During: run one case alone first; a hang is never discovered through a suite. After: SAB gate at Tier 3; commit `V5 P129: shared clip in review-edit, gates, docs-only CI skip`; **push (first of three); do not wait.**

### Expected Result

`review-edit.spec.ts` takes well under half its old time, a docs-only pull request costs a minute, and the bench takes a count. The product has not changed.

---

## Prompt 130 (V5 02/08) — The verifier: checks 7 to 11, its self-test, and the stereo loudness fix

**Effort: High.** Numerical checks whose thresholds are assumptions, a product change to loudness, and the tool everything later is judged by.

### Objective

Build checks 7 to 11, prove each one tells good from bad, fix the product so a stereo file reads -14 LUFS, and read all of it on real exports. **First read a real export before the fix**, so the entry has a real "before" for G problem 1.

### Project State Before This Prompt

- First: `gh pr checks <pr>` for P129's run. Red: fix it now, with its own commit, before anything else.
- `verify/verify_mp4.py` has checks 1-6 and `NOT_IMPLEMENTED`; `--source` is accepted and unused.
- `crates/offcut-dsp/src/loudness.rs` meters `Channel::Center`. `ebur128` 0.1.10 has `Channel::DualMono`.
- `helpers/fixtures.ts`'s `verifyMp4(file, profile, expectedDurationMs)` runs `python`.
- P129's keyed build is in `web/dist` (if not, `pnpm build:e2e` first, **with the fix not yet made**).
- `fixtures/.gen/speech/` holds the pause fixtures; white frames of the long-pause one at 60 and 450.

### Why This Comes Here

The tool that judges everything comes before the work it judges (§5). Check 7 fails on every export until the fix, so both belong in one commit, and D-204 says the product is fixed, not the check.

### Scope

#### Implement

- G 2.1: `OFFCUT_KEEP_EXPORTS` in `verifyMp4`; one Creator export on the old build; `ffmpeg ebur128` read directly; record "before". Near -14 means §8's reasoning is wrong: **stop and report**.
- G 2.2: `DualMono`, the doc comment, `a_mono_programme_reads_3_lu_louder_as_two_loudspeakers`, the rename to `a_1khz_sine_of_peak_minus_20_reads_minus_20`, `the_output_as_a_stereo_file_reads_the_target`; `pink_noise` only if its "not capped" assertion fires (SAB rule 4).
- G 2.3: `make_watermark_mask.py` and the PNG; TS §5 row. Look at it.
- G 2.4: checks 7-11, `run_bytes`, `pcm`, `grey`, D-203's import. `NOT_IMPLEMENTED` goes. Docstring, `README.md`, `requirements.txt`'s comment. `--source` read.
- G 2.5: `test_verify_mp4.py`, one case at a time; TS §5 row; `"verify:test"` root script; `ci.yml`'s two steps.
- G 2.6: `verifyMp4(…, source?)`, `expectAllChecks`; `export-creator.spec.ts`: Creator and Free with the source, the "as creator" case reads `FAIL 2` and `FAIL 9`, `metadataProblems`, `tagsOf` and `SOURCE_TAGS` go, the second-recording case reads `PASS 11`. `pnpm build:e2e`. A temporary long-pause export. Tune offline. Record every `INFO` line. **Delete the saved files.**
- G 2.7: the drills.

#### Do Not Implement Yet

- Anything in the muxer (P131) or `export-loop.ts` (P132). `timeline-preserved.spec.ts` (P133). The `summary` or cache cases (P134, P135). TS §39.3's values (P136; the entry holds them until then).

### Existing Components To Reuse

- `verify_mp4.py`'s `run`, `probe`, `only_stream`, `seconds`, `Failed`, `CHECKS` and its printing rules.
- `export-creator.spec.ts`'s `metadataProblems` is the rule of D-100: read it, move the rule, keep what it asserts. `fixtures.ts`'s `genFixture`, `referenceClip`, `v3TestClip`.
- `watermark.rs` and `safe_area.rs` for the mask's numbers: read them as data. V4 CL Entry 17: the founder moved the band.

### Files To Create

- `verify/make_watermark_mask.py`, `verify/watermark_mask_720.png`, `verify/test_verify_mp4.py`.

### Files To Modify

- `crates/offcut-dsp/src/loudness.rs`, `crates/offcut-dsp/tests/loudness_target.rs`; `verify/verify_mp4.py`, `README.md`, `requirements.txt`; `web/tests-e2e/helpers/fixtures.ts`, `export-creator.spec.ts`; `.github/workflows/ci.yml`; `package.json`; `docs/technicalspec.md` (two §5 rows); `docs/v5/v5changelog.md`.

### Architecture Requirements

- **The verifier imports nothing from `crates/` or `web/`, and no library they use** (TS §27.2): the standard library, NumPy, `ffmpeg`, `ffprobe`. One file.
- The mask is drawn by a tool that reads the font as data and shares no code with the renderer (D-206). Pillow is not in `requirements.txt` and not in CI.
- The expected duration and the source are two arguments (D-210). No sidecar file.
- All thresholds are named constants at the head (§ Appendix C).

### Data Types / Interfaces

```python
LOUDNESS_TARGET, LOUDNESS_TOLERANCE = -14.0, 1.0
ENVELOPE_HOP_SAMPLES = 480
SYNC_WINDOW_MS, SYNC_LAG_HOPS, SYNC_LIMIT_MS = 5000, 100, 20
MIN_ENVELOPE_CORR = 0.5        # (assumption, P130)
SILENT_WINDOW_DBFS = -50.0     # (assumption)
FLASH_MIN, FLASH_ABOVE = 200, 80   # (assumption)
WATERMARK_FRAMES = 15          # (assumption)
WATERMARK_FREE_MIN, WATERMARK_CREATOR_MAX = 0.5, 0.2   # (assumption, TS §39.3 item 14)
BITRATE_LIMIT = {"creator": 12_000_000, "free": 6_000_000}
```

```ts
export type Verdict = { status: number | null; output: string };
export function verifyMp4(file: string, profile: "free" | "creator", expectedDurationMs: number, source?: string): Verdict;
/** Every check passed and none was not measured. `flashes`: how many white frames were compared. */
export function expectAllChecks(verdict: Verdict, o?: { flashes?: number }): void;
```

`expectAllChecks` asserts: `PASS 1` to `PASS 11`, no `FAIL`, status 0, no `not measured`; with `flashes`, the line `INFO 8: <n> flashes compared`.

### Implementation Requirements

#### 1. Check 7 (§7.1)
`ffmpeg -nostdin -i <file> -map 0:a:0 -af ebur128=peak=none -f null -`; the summary's last `I:` line of stderr; `abs(I + 14) <= 1`. The stream as it is, stereo. `INFO 7: integrated -14.2 LUFS`.

#### 2. Check 8 (§7.2)
Audio: both decoded to mono at 48 kHz; RMS over 480-sample hops, linear, not dB. Two windows on the **export's** timeline: first and last 5,000 ms; a clip of 10 s or less uses one window. Normalized cross-correlation at lags ±100 hops, around the same place in the source; best lag; a parabola through the peak and its neighbours; times 10 ms. Pass when `abs(offset) <= 20 ms` and the peak is at least `MIN_ENVELOPE_CORR`. A source window whose peak RMS is under -50 dBFS: `INFO 8: window at <t> s is silent in the source, not measured`.

Video: one mean brightness per frame (`-vf fps=30,scale=16:16:flags=area` for the source, which may be VFR; `scale=16:16` for the export; grey). A flash is at least 200 and at least 80 above the median of the 30 frames around it. Same count, each within one frame. None in the source: `INFO 8: no white frame in the source, video offset not measured`. Otherwise `INFO 8: 2 flashes compared, largest offset 0 frames`.

#### 3. Check 9 (§7.3)
The mask's box: pixels not 128, plus 4 px. 15 frames evenly spaced by index, skipping flashes, scaled to 720 x 1280 (`select`, `scale=720:1280:flags=area`); their mean inside the box. Subtract a 9 x 9 box blur from both the mean and the mask. Pearson over the box. `free` ≥ 0.5, `creator` ≤ 0.2. `INFO 9: watermark correlation 0.78`.

**If the readings are not well apart** (free under 0.6, or creator over 0.1, on the three real exports): first 31 frames, then the mask's alignment. A threshold moves only while free and creator stay at least 0.3 apart, and never to make one file pass.

#### 4. Check 10 (§7.4)
`ffprobe -select_streams v:0 -show_entries packet=size -of csv=p=0`, summed, x 8, over the video stream's duration. `INFO 10: 3.1 Mb/s`. The average; a peak is not judged.

#### 5. Check 11 (§7.5)
No format or stream tag whose key contains `location` or `iso6709`. No `title`, `comment`, `description`, `artist`, `album` tag with a non-empty value. `creation_time` absent or the epoch. With `--source`: no `title`, `comment`, `encoder` or `creation_time` tag carrying the source's value. Neither the bytes of `©xyz` nor `ISO6709`, read in 1 MiB pieces with an overlap of 7 bytes. **A `FAIL 11` names the key, never the value.**

#### 6. The self-test (§7.6)
`unittest`, the verifier by subprocess. The good file: scale and crop to 720 x 1280, `ultrafast`, AAC-LC stereo, `-map_metadata -1`, `+faststart`, and a `volume` filter computed by measuring first, so it reads -14 LUFS. A second burns in the mask with `blend=all_expr='if(eq(B,128),A,0.6*B+0.4*A)'` for `free`. A creator-size one at 1080 x 1920. Then §7.6's eleven bad files; each asserts its `FAIL n` and every other check `PASS`. It honours `OFFCUT_FIXTURES_DIR`, cleans its directory, and prints no path from the repo's media. Under 3 minutes.

#### 7. The temporary long-pause export
`web/tests-e2e/zz-longpause.spec.ts` and `web/zz-pw.config.ts` (SAB rule 7), which export the fixture once under `OFFCUT_KEEP_EXPORTS` and are deleted before the commit.

### Error Handling

- NumPy absent: exit 2 with `run pip install -r verify/requirements.txt` (D-203).
- A check that throws `OSError`, `ValueError`, `KeyError`: `FAIL n`, as today.
- `pink_noise` is the only `offcut-dsp` failure with a pre-agreed response. Any other is read before anything is edited: `chain_no_clicks.rs`, `duration_preserved.rs`, `program_identity.rs` are frozen and must pass.
- The recognizer hears `pcm16`, not the chain, and prosody is z-scored per clip, so no transcript or event moves. A transcript-dependent failure has another cause.

### Concurrency / Async Requirements

None. The verifier is synchronous; pipes, no temporary files except the self-test's directory.

### Persistence Requirements

`OFFCUT_KEEP_EXPORTS` is outside the repository and gone by the end of this prompt.

### API Requirements

None. The command line and its `PASS`, `FAIL`, `INFO` lines freeze here (§23.1).

### Security Requirements

- A file of `testclips/` is never written anywhere. A verifier line carries numbers and keys only.
- The self-test's temporary directory is deleted however it ends.

### Testing Requirements

#### Unit Tests
`a_mono_programme_reads_3_lu_louder_as_two_loudspeakers`, `a_1khz_sine_of_peak_minus_20_reads_minus_20`, `the_output_as_a_stereo_file_reads_the_target`.

#### Integration Tests
`test_verify_mp4.py`. `pnpm e2e:media export-creator.spec.ts --grep "no token|another key|Creator export|Free-plan" --max-failures=1`. The verifier by hand on the three saved exports.

#### Failure-Path Tests
§7.6's bad files; G 2.7's drills. The second-recording case, on D1 only, by `--grep "no metadata"` once: `PASS 11` on a file whose source carries a location.

### Validation Commands

```bash
cargo test -p offcut-dsp                                          # all pass
python verify/test_verify_mp4.py 2>&1 | tail -1                   # OK
git grep -n "NOT_IMPLEMENTED\|metadataProblems\|tagsOf" -- verify web/tests-e2e    # no output
git status --short | grep -E "zz[-_]"                             # no output
ls "$HOME/v5-exports" 2>/dev/null | wc -l                         # 0
```

### Acceptance Criteria

- [ ] The "before" reading on a real export is in the entry (about -11 LUFS, or stop).
- [ ] `verify_mp4.py` prints 11 `PASS` lines on the Creator and the Free export, and on the long-pause export with 2 flashes compared.
- [ ] -14 ± 1 LUFS is read from a real stereo export, by `ffmpeg`, not the product's meter.
- [ ] The watermark readings are at least 0.3 apart and on the right sides of 0.5 and 0.2.
- [ ] Each bad file fails for its own reason; the self-test takes under 3 minutes and is in `ci.yml`.
- [ ] `FAIL 11` names no value (read in the self-test's output).
- [ ] The saved exports and the `zz-` files are gone. Tier 2 and the frozen check pass.

### Agent Instructions

Before: read §7, §8, D-203 to D-210, TS §27.2, G Phase 2, and P129's CI. During: Tier 0 throughout; tune offline; never rebuild to read a number. After: SAB gate at Tier 2; commit `V5 P130: verifier checks 7-11, its self-test, and the stereo loudness fix`. No push.

### Expected Result

An outside tool judges all 11 checks, has been shown to tell good from bad, and passes real exports. Known issue 1 is closed. An export reads -14 LUFS; the preview is about 3 dB quieter, which matches it.

---

## Prompt 131 (V5 03/08) — The muxer: `ctts`, the video edit list, the overflow guard

**Effort: High.** Binary format arithmetic, and a file with no reordering must not change by one byte.

### Objective

A stream with B-frames makes a correct file, read by `ffprobe` and `ffmpeg` on real samples, and the `moov` overflow guard is tested. A file with no reordering stays byte-identical to today's.

### Project State Before This Prompt

- P130's commit exists. CI of P129 was read.
- `Mp4Muxer::add_video_sample` refuses `frame != number`. `boxes::edts` exists for audio priming. `demux.rs` and `sample_table.rs` already read `ctts`.
- `gen_fixtures.sh`'s `$H264` holds `-bf 0`. `fixtures/manifest.json` has 25 fixtures; `demux_fixtures.rs` reads every `ok_*` as the manifest says.

### Why This Comes Here

It stands alone in Rust and needs no browser, so it costs little. Phase 4 rebuilds the WASM and carries it (§22).

### Scope

#### Implement

- G 3.1, G 3.2: `ctts`, `stbl`'s new argument, `presented`, `seen`, `MAX_REORDER`, `moov()`'s arithmetic, the gap check in `finalize`, the module's header ("in the order the frames are shown" goes).
- G 3.3: `ok_bframes.mp4`, its manifest row, TS §5's "26 fixtures"; `pnpm fixtures`.
- G 3.4: every case of §17.2 for this crate, and `mux_reorder.rs` with its TS §5 row.
- G 3.5: the drills.

#### Do Not Implement Yet

- `Writer.video` in `export-loop.ts` (its comment is P132's; its code does not change). Any browser run. `pnpm build:wasm`.

### Existing Components To Reuse

- `boxes::edts`, `TrackLog`, `MemSink`, `Demuxer`, the test helpers of `mux_roundtrip.rs` (copy what `mux_reorder.rs` needs; a test file cannot reach another's).
- `enc`, `src`, `$OUTF`, `$FAST`, `$AAC`, `$FS` of `gen_fixtures.sh`; `ok_vfr`'s manifest row as the shape.

### Files To Create

- `crates/offcut-mp4/tests/mux_reorder.rs`.

### Files To Modify

- `crates/offcut-mp4/src/mux.rs`, `mux_boxes.rs`, `tests/mux_roundtrip.rs`; `fixtures/gen_fixtures.sh`, `manifest.json`; `docs/technicalspec.md`; `docs/v5/v5changelog.md`.

### Architecture Requirements

- `add_video_sample`'s signature does not change; `frame` means the presentation index (D-211). `offcut-wasm-render/src/mux_api.rs` is frozen.
- `ctts` version 0, plus a video `elst` starting at the largest offset, written **only** when a frame arrived out of order. No brand change.
- The muxer holds no sample in memory (TS §31): `presented` and `seen` are indexes.
- Non-test code uses `get`, `saturating_*`, no slice indexing.

### Data Types / Interfaces

```rust
/// H.264's DPB limit: a frame further ahead than this cannot be a real reorder.
const MAX_REORDER: u32 = 16;

// In Mp4Muxer, beside what it has:
/// Presentation index of each sample, in the order written.
presented: Vec<u32>,
/// Which presentation indexes have arrived.
seen: Vec<bool>,

// mux_boxes.rs
pub fn ctts(runs: &[(u32, u32)]) -> Vec<u8>;                 // version 0
pub fn stbl(/* … */ ctts: Option<&[(u32, u32)]>, /* … */) -> Vec<u8>;
```

### Implementation Requirements

#### 1. Rules of `add_video_sample`
First sample is a keyframe and frame 0. A frame already seen, or more than `MAX_REORDER` beyond the sample number: `OutOfOrder`. `sync` keeps sample numbers from 1, in decode order.

#### 2. `moov()`
G 3.2's arithmetic. All offsets 0: no `ctts`, no video `edts`, the same bytes as today. Otherwise `ctts` run-length after `stts`, and `boxes::edts(video_duration, delay)` on the video `trak`.

#### 3. Overflow tests (§9.2)
At 90 s the largest `moov` is about an eighth of the 256 KiB reserve, plus `ctts` at worst 8 bytes x 2,700 = 21.6 kB. `a_moov_over_the_reserve_is_refused`: 30,000 one-byte frames written in turns with audio, so each is a chunk of its own; `MoovOverflow` from `finalize`, nothing over `mdat`. `a_reserve_with_under_8_bytes_left_is_refused`: the `(1..8)` branch, sizes found by search in the test.

#### 4. `mux_reorder.rs` (§9.3)
`ok_bframes.mp4` with `Demuxer`; its samples through `Mp4Muxer` in decode order with their presentation indexes and the fixture's `avcC`; audio a dummy track from the same fixture. Read the file's own `STALE` message pattern from `demux_fixtures.rs` for a missing fixture. A missing `ffprobe` or `ffmpeg` fails: CI and D1 both have them.

### Error Handling

`OutOfOrder`, `MoovOverflow` as today; `E_MUX` to the page, unchanged. No new error.

### Concurrency / Async Requirements

None.

### Persistence Requirements

None. `ok_bframes.mp4` lives in `fixtures/.gen/`, never committed.

### API Requirements

None.

### Security Requirements

None beyond SAB rule 9: the fixture is derived, with no new recording (the V3 rule).

### Testing Requirements

#### Unit Tests
`frames_written_in_decode_order_are_shown_in_presentation_order`, `no_reordering_writes_no_ctts_and_no_video_edit`, `a_gap_in_the_frames_is_out_of_order_at_finalize`, `a_frame_far_ahead_is_out_of_order_at_once`, `ninety_seconds_reordered_fit_the_space_kept_for_moov`, `a_moov_over_the_reserve_is_refused`, `a_reserve_with_under_8_bytes_left_is_refused`, the proptest over permutations within the window. `a_frame_out_of_its_turn_is_out_of_order` stays as written.

#### Integration Tests
`real_b_frames_come_back_at_30_fps`; `demux_fixtures.rs` reading the new row.

#### Failure-Path Tests
G 3.5's two drills.

### Validation Commands

```bash
pnpm fixtures && ffprobe -v error -select_streams v:0 -show_entries frame=pict_type -of csv=p=0 fixtures/.gen/ok_bframes.mp4 | sort | uniq -c   # a B row
cargo test -p offcut-mp4                                          # all pass
cargo clippy -p offcut-mp4 --all-targets -- -D warnings           # clean
git diff "$BASE" -- crates/offcut-mp4/tests/mux_roundtrip.rs | grep -E '^-[^-]'   # the one comment line
```

### Acceptance Criteria

- [ ] The fixture holds real B-frames; `demux_fixtures.rs` accepts it.
- [ ] `ffprobe` reads every delta as 1/30 s, and `ffmpeg` decodes with no error, on the re-muxed file.
- [ ] A file with no reordering is byte-identical to before (asserted).
- [ ] `MoovOverflow` is tested on both branches.
- [ ] The existing cases of `mux_roundtrip.rs` pass with one comment changed.
- [ ] `cargo test -p offcut-mp4`, clippy, `check-file-tree` and the frozen check pass. **No browser and no WASM rebuild in this prompt.**

### Agent Instructions

Before: read §9, D-211, D-212, TS §21.3, G Phase 3, and all of `mux.rs` and `mux_boxes.rs`. During: Tier 0 by name. After: SAB gate at Tier 1; commit `V5 P131: ctts, video edit list, moov overflow guard tested`. No push.

### Expected Result

G problem 5 is closed. A machine whose encoder reorders frames gets a correct file instead of `E_MUX`, and the proof uses real B-frames.

---

## Prompt 132 (V5 04/08) — S-8: export over program time, `prog48`, the ladder restart

**Effort: High.** A restart inside a loop that holds encoders, frames and a file; one protocol field changes on both sides.

### Objective

The export loop renders at program time, takes each source frame at `source_at(t)`, encodes `prog48` read from a new `prog48.f32`, and an encoder that fails restarts from frame 0 on the next ladder entry with the partial output gone. An audio encoder failure retries once.

### Project State Before This Prompt

- P131's commit exists. `verify_mp4.py` has 11 checks.
- `start-export.ts` line 161, `export-loop.ts` line 352 carry `V5` markers. `runExport` is one body, 160 lines. `session.source_at(t)` exists on the type; `recorded(pos)` is exported from `preview-loop.ts`.
- `opfs.ts` has `clipOut48`, `writeAudio`, `readAudio`. `pickVideoConfig` is called by `export-loop.ts` only; `capability.ts` reads `VIDEO_ENCODE_LADDER`.
- `rpc-worker.ts` has `devFault` with `devCrash`, `devHang`, `devFailNext`. `check-hosts.mjs`'s `DEV_MARKERS` is an explicit list.

### Why This Comes Here

S-8 is what makes V6.2's cuts additive (§10.2). The verifier exists, so a real export proves the identity case; `export-loop.test.ts` proves the seam no browser can reach while `set_scene` refuses other maps.

### Scope

#### Implement

- G 4.1: `paths.clipProg48`, `writeProgram`, `readProgram`; `run-pipeline.ts` step 13; `clip-store.ts`'s comment; `run-pipeline.test.ts`'s mock and its order.
- G 4.2: **REOPENED CONTRACT** `exportClip`'s `prog48`: `protocol.ts`, `export-job.ts`, `start-export.ts`, one commit. The marker at line 161 goes.
- G 4.3: the loop on `source_at`. The marker at line 352 goes.
- G 4.4: `supportedConfigs`; `OpfsSink.reset`; `attempt` and `runExport`; `Writer.video`'s comment; `useTimeLeft` (D-238).
- G 4.5: `devFailEncoder`, `takeEncoderFault`, `failEncoderAt`, `DEV_MARKERS`.
- G 4.6: `export-loop.test.ts`, with its TS §5 row. **Every case of §17.3's table.**
- G 4.7: `pnpm build:e2e`; one Creator export.
- G 4.8: the drills.

#### Do Not Implement Yet

- `timeline-preserved.spec.ts`, or a browser run of `failEncoderAt` (P133). The cache, `exportProfile`, the gate (P134). Any `map` other than the identity outside the test's fake session.

### Existing Components To Reuse

- `Watch`, `Writer`, `encoding`, `flushed`, `closeFrame`, `liveFrames` of `export-loop.ts`: each attempt makes its own.
- `recorded` of `preview-loop.ts`, as `preview-job.ts` uses it. `frameAtBlocking`'s jump handling (`#ask`, `#frames.jumped`, `#dropDue`) needs nothing.
- `devFailNext`'s handling as the shape of the new fault; `pool[kind].notify`.
- `run-pipeline.test.ts`'s scripted mocks.

### Files To Create

- `web/src/workers/render/export-loop.test.ts`.

### Files To Modify

- `web/src/persistence/opfs.ts`; `web/src/usecases/run-pipeline.ts`, `run-pipeline.test.ts`, `start-export.ts`, `dev-faults.ts`; `web/src/state/clip-store.ts` (comment); `web/src/workers/protocol.ts`, `rpc-worker.ts`; `web/src/workers/render/export-job.ts`, `export-loop.ts`, `encoders.ts`, `opfs-sink.ts`; `web/src/ui/dev-hooks.ts`, `ui/components/ExportProgress.tsx`; `scripts/check-hosts.mjs`; `docs/technicalspec.md`; `docs/v5/v5changelog.md`.

### Architecture Requirements

- `out48.f32` stays: V6.2 and V6.3 rebuild `prog48` from it (D-213).
- `runExport`'s attempt-and-ladder shape freezes here (§23.1). A sink, cancel, GPU or mux failure is never restarted.
- The encoders, `Watch`, `Writer` and the muxer are made inside `attempt`; its `finally` closes them. `liveFrames` balances there (INV-11).
- `takeEncoderFault` is `false` in a production build and is folded away by `__OFFCUT_DEV_FAULTS__`. `export-loop.ts` calling it adds no import cycle: it already imports `rpc-worker.ts`.
- The worker does not compute the loudness or touch `prog48` beyond encoding it. No sample is removed; padding only (INV-10).

### Data Types / Interfaces

```ts
// workers/protocol.ts
exportClip(p: {
  exportId: ExportId;
  entitlementToken: string;
  /** The program's audio; transferred (TS §40.9 S-8). */
  prog48: Float32Array;
}): Promise<{ opfsPath: string; summary: ChangeSummary; stageTimings: StageTiming[] }>;

// render/encoders.ts
/** The ladder's entries the browser supports for this profile, in order: what an export may try (TS §21.2, §21.4). */
export async function* supportedConfigs(profile: ExportProfile): AsyncGenerator<VideoEncoderConfig>;

// render/opfs-sink.ts
/** Empties the file: what a failed attempt wrote goes (TS §21.4). */
reset(): void;

// workers/rpc-worker.ts
/** Whether a development fault fails the encoder at this frame. `false` in a production build. */
export function takeEncoderFault(n: number): boolean;

// ui/dev-hooks.ts
/** The video encoder fails at `frame`, the next `times` attempts (D-218). */
failEncoderAt(frame: number, times: number): void;
```

### Implementation Requirements

#### 1. `runExport`
§10.3's body, verbatim: a loop over `supportedConfigs(a.profile)`, an inner loop that retries an audio failure once, `a.sink.reset()` and `a.onProgress(0, a.session.frame_count())` before each restart, `lastFailure` thrown when the ladder ends. `attempt(a, config)` is today's body with `const config = await pickVideoConfig(a.profile)` removed.

#### 2. What an attempt starts from
The scene and canvas are unchanged. `frameAtBlocking(0)` after being at frame `k` is a jump back, which `#ask` handles as a seek does. `onEncoded` runs once, on the attempt that succeeds. `export-job.ts`'s `run.stage` stays `render_encode` through a restart; `render_encode`'s timing includes failed attempts, which is honest.

#### 3. `useTimeLeft`
`if (first.current !== undefined && done < first.current.done) { first.current = undefined; setShown(null); }` before the rate is read. The bar falls to 0.

#### 4. `export-loop.test.ts` seam case
A fake `source_at` that goes 0, 1,000 (jump), 1,033, then `held` 2,000 twice: the source is asked exactly those, in that order. Read `SourcePos`'s shape from `gen/domain.ts`.

### Error Handling

| Failure | Result |
|---|---|
| `E_ENCODE_VIDEO` | Next supported entry from frame 0; no entry left: the last failure |
| `E_ENCODE_AUDIO` | Once more on the same entry; twice: thrown |
| `source_at` returns `null` | `E_INTERNAL` `NoScene` |
| A sink error, `E_GPU_LOST`, `E_MUX` | Thrown, never restarted |
| Cancel | `{ cancelled: true }`, no restart |

### Concurrency / Async Requirements

`attempt`'s `finally` runs before a restart: encoders closed, muxer freed, `watch.wake()`. Only then `reset`. A late encoder callback belongs to its attempt's `Watch` and cannot reach the next one.

### Persistence Requirements

`clips/<id>/prog48.f32` beside `out48.f32`: at most 2 x 17 MB for 90 s. A write copies and does not transfer. No new IndexedDB field.

### API Requirements

None. No new request.

### Security Requirements

`devFailEncoder` is absent from `web/dist`, checked by `DEV_MARKERS`. No file name of the user's in a path (P-11).

### Testing Requirements

#### Unit Tests
`export-loop.test.ts`: frame accounting, identity, seam, `null`, audio, backpressure, video failure, all entries fail, unsupported entry, audio failure, not restarted, cancel, `liveFrames` 0 after every case.

#### Integration Tests
`run-pipeline.test.ts`. `pnpm e2e:media export-creator.spec.ts --grep "Creator export" --max-failures=1`: all 11 checks still pass.

#### Failure-Path Tests
G 4.8's three drills.

### Validation Commands

```bash
git grep -n "out48" -- web/src/workers/render                     # no output
git grep -nE "\bV5\b" -- web/src/workers/render/export-loop.ts    # no output
wc -l web/src/workers/render/export-loop.ts                       # under 700
pnpm --filter web exec vitest run src/workers/render/export-loop.test.ts src/usecases/run-pipeline.test.ts   # all pass
pnpm build && git grep -c devFailEncoder -- scripts/check-hosts.mjs && node scripts/check-hosts.mjs | tail -2   # exit 0, no marker in dist
```

### Acceptance Criteria

- [ ] S-8: the loop reads `source_at`; `prog48.f32` is written and encoded.
- [ ] **REOPENED CONTRACT** `exportClip` is recorded; page and worker change in one commit.
- [ ] The loop's Vitest table is green, including the seam and every restart path.
- [ ] A Creator export still passes all 11 checks.
- [ ] `git grep -n "out48" web/src/workers/render` prints nothing; `export-loop.ts` is under 700 lines.
- [ ] `devFailEncoder` is in `DEV_MARKERS`, and a plain `pnpm build` holds none.
- [ ] Tier 2 and the frozen check pass.

### Agent Instructions

Before: read §10, §16, D-213 to D-219, D-238, TS §21.3-§21.4, §40.9, G Phase 4. During: write `export-loop.test.ts` alongside the loop: it takes seconds, and most of the proving happens there. After: SAB gate at Tier 2; commit `V5 P132 (S-8): export over program time, prog48, ladder restart`. No push.

### Expected Result

With the identity map the file is what it was, and now V6.2's map, V6.3's music and V6.6's end card can each reach the export by changing data.

---

## Prompt 133 (V5 05/08) — The proof: `timeline-preserved.spec.ts`

**Effort: Medium.** Cases exactly specified; the running is the work. Switch to High if a word-time case fails: it could be the recogniser or the rule.

### Objective

BP §7's demo is a test: 20 s in, 20 s out, the pause intact, audio and video in sync on both sides, on both pause fixtures, with all 11 checks, and an encoder failure that does not fail the export.

### Project State Before This Prompt

- P132's commit and its keyed build. `failEncoderAt` exists. No `timeline-preserved.spec.ts`.
- `speech_20s_long_pause.mp4` has speech 0-5 s and 10-20 s with room tone between, white frames at 60 and 450. `speech_30s_sparse.mp4` has five 1.1 s pieces at 0.5, 7.0, 13.5, 19.5, 26.0 s, white frames at 15 and 885.

### Why This Comes Here

A file proved correct is the only one worth caching (§5). The cases that read the cache and the summary join them in P134 and P135.

### Scope

#### Implement

- G 5.1: the spec, two serial groups, cases 1-5, 8, 9 and both of the sparse fixture. Case 5 without `from_cache`, values or rows. `watchExport` moves to `helpers/fixtures.ts`, and `export-creator.spec.ts` imports it. `playwright.config.ts`'s `MEDIA_SUITES`.
- G 5.2: what P130 left in `export-creator.spec.ts`.
- G 5.3: each case alone, then the file once.

#### Do Not Implement Yet

- Cases 6 and 7; case 5's cache and value assertions (P134, P135). The summary (P135).

### Existing Components To Reuse

- `export-creator.spec.ts`'s `exportAndSave` as the shape (a local copy here: its closures differ), `watchExport`, `seedEntitlement`, `mintEntitlementToken`, `routeAssets`, `ensureModelCached`.
- `readIndexedDb`, `opfsList`, `captionMirror`, `seekTo`, `previewAt`, `scrubBar`, `editWord`, `toggleItem`, `setStyle`, `transcriptWords`, `genFixture`.
- `messages.errors.E_ENCODE_VIDEO`, `messages.recovery.tryAgain`, `recovery.close`.

### Files To Create

- `web/tests-e2e/timeline-preserved.spec.ts`.

### Files To Modify

- `web/tests-e2e/helpers/fixtures.ts`, `export-creator.spec.ts`; `web/playwright.config.ts`; `docs/v5/v5changelog.md`.

### Architecture Requirements

- The test computes the expected duration itself and passes it separately from the source (D-210): 20,000 and 30,000 here, and from V6.2 they differ.
- Scratch directory outside the test's own folder; deleted in `afterAll` (TS §22.5).
- Word text is never printed: an assertion message says "the first word after the pause", not the word (SAB rule 9).

### Data Types / Interfaces

The stored transcript as `readIndexedDb` gives it: `{ schemaVersion: number; value: { transcript: { words: { text: string; start_ms: number; end_ms: number }[] }; prosody: unknown } }`.

### Implementation Requirements

#### 1. The cases
G 5.1's two tables, verbatim. `clipId` comes from `opfsList(page, "clips")`. A token is seeded before each export.

#### 2. Case 8, `failEncoderAt(30, 1)`
```ts
await page.evaluate(() => (window as unknown as { __offcutDev: { failEncoderAt(f: number, t: number): void } }).__offcutDev.failEncoderAt(30, 1));
```
Read `e2eExportBar`: at some point a value falls below the one before it. The downloaded file passes `expectAllChecks(…, { flashes: 2 })`. No `export-failed`.

#### 3. Case 9, `failEncoderAt(30, 9)`
More than the ladder has. `export-failed` shows `messages.errors.E_ENCODE_VIDEO`'s words, "Try again" and "Close"; `opfsList(page, "exports")` holds no file, `exports/tmp` nothing; the preview plays again.

### Error Handling

G 5.3's table. **A word inside the pause:** INV-5 says a word is not moved, not that one cannot be heard. Read `fixtures/speech/README.md`. If it stays at its own time and no word was moved earlier, the case's wording is too strict: record why, and tighten to "no word moved earlier". **One supported entry on D1:** record it, and skip case 8 on that machine with the reason in its message; `export-loop.test.ts` proves the restart.

### Concurrency / Async Requirements

The groups are serial: a case's state is the previous one's. A failed case stops the ones after it, which is what the order wants.

### Persistence Requirements

None. A file is saved to the scratch directory and removed in `afterAll`.

### API Requirements

None. `quiet` is `export-creator.spec.ts`'s case.

### Security Requirements

SAB rule 9. No `.mp4` under `web/test-results`.

### Testing Requirements

#### Unit Tests
None new.

#### Integration Tests
Every case alone with `--grep` and `--max-failures=1`, then the file once.

#### Failure-Path Tests
Cases 8 and 9.

### Validation Commands

```bash
pnpm --filter web exec playwright test --project=media --list timeline-preserved.spec.ts | tail -1   # 9 tests now; 11 after P134
pnpm e2e:media timeline-preserved.spec.ts 2>&1 | tail -3          # passed; about 3 min (estimate)
git status --short | grep -E "\.mp4$"                             # no output
```

### Acceptance Criteria

- [ ] `timeline-preserved.spec.ts` passes on both fixtures, 600 and 900 frames, with their flashes compared and all 11 checks.
- [ ] Word times hold; the caption appears at its spoken time; the preview ends at the source's duration.
- [ ] A failed encoder restarts and its file passes; a ladder that runs out shows the card and leaves no file.
- [ ] `timeline-preserved.spec.ts` is in `MEDIA_SUITES`.
- [ ] Tier 2 and the frozen check pass.

### Agent Instructions

Before: read TS §27.1's row, §17.5, D-210, G Phase 5. During: only `web/tests-e2e` changes, so no build. After: SAB gate at Tier 2; commit `V5 P133: timeline-preserved on both pause fixtures, all 11 checks`. No push.

### Expected Result

The exit-checklist line of BP §7.3 for `timeline-preserved.spec.ts` is true, and the demo is a test.

---

## Prompt 134 (V5 06/08) — S-9: the render cache on the verified profile, the gate, `B_STORAGE_LOW`

**Effort: High.** This key decides who gets which file. Getting it wrong gives a 1080p file with no watermark to a token that does not entitle it.

### Objective

The same edit exported twice comes from the render cache instantly; the key covers all ten inputs, with the profile kind from the token the worker verified and a computed renderer version; re-importing a file clears it; `B_STORAGE_LOW` is evaluated in the one place; a blocked export says why; and three test files prove it.

### Project State Before This Prompt

- P133's commit. `start-export.ts` still has `renderAndDeliver` with `forExport()` in one form, and calls `exportStore.start` then `clear` at once.
- `schema.ts` has `RenderCacheRecord` (`clipId`, `exportId`, `opfsPath`, `bytes`, `createdAt`), `STORES.renderCache`; no repository exists. `cancel.spec.ts` reads `renderCacheCount`.
- `sourceHashPrefix` is in the clip's record (`clipsRepo.get`), not the clip store.
- `render.worker.ts` line 236 has `exportProfile` on the job files' state object. `scripts/check-copy-codes.mjs` lists `B_STORAGE_LOW` in `COPY_PENDING`.

### Why This Comes Here

S-9 needs a file proved correct. The gate belongs in the same rewrite of `start-export.ts`, since both happen between `gating` and `rendering` (§13).

### Scope

#### Implement

- G 6.1: `RenderCacheRecord.summary`; `render-cache-repo.ts` in full; `__RENDERER_VERSION__`.
- G 6.2: **REOPENED CONTRACT** `exportProfile`.
- G 6.3: `freeBytes`; `forExport(room?)`; `export-store.ts`'s actions; `export-machine.ts`'s comment; `ExportButton.tsx`.
- G 6.4: `B_STORAGE_LOW`'s words, out of `COPY_PENDING`.
- G 6.5: `start-export.ts`, §13's twelve steps. `export_done` keeps its placeholders except `from_cache`, until P135.
- G 6.6: `importClip` clears the cache.
- G 6.7: `export-machine.test.ts`, `blockers.test.ts`, `start-export.test.ts`. `export-creator.spec.ts`: same token, forged, no room, expired. `timeline-preserved.spec.ts`: cases 5, 6, 7.
- G 6.8, G 6.9: the drills, the runs.

#### Do Not Implement Yet

- `editValues`, the summary and the question on screen (P135). `result.summary` is stored in the row here and read by nothing yet but the row's own test. `quota.ensureFree`, several rows, eviction (V7). `retry` stays unfired (D-153).

### Existing Components To Reuse

- `edits-repo.ts` as the repository's shape (`Versioned`, `schemaVersion > DB_VERSION`, `storageError`). `clipsRepo.get`, `clipsRepo.toBytes`, `hashPrefix`'s `crypto.subtle` use. `opfs.getFile`.
- `whenFree`, `whenReleased`; `answered` of `control-preview.ts` as the three lines to copy.
- `run-pipeline.test.ts`'s scripted pool; `clip-machine.test.ts`'s shape.
- `exportCancelled`, `renderWorkerGone`, `failureOf`, `goOn`, `ExportCancelled`, `deliver`, `planOf`, `newExportId` of `start-export.ts`: all stay.

### Files To Create

- `web/src/persistence/render-cache-repo.ts`, `web/src/state/blockers.test.ts`, `web/src/state/machines/export-machine.test.ts`, `web/src/usecases/start-export.test.ts`. (All already in TS §5.)

### Files To Modify

- `web/src/persistence/schema.ts`, `opfs.ts`; `web/src/state/blockers.ts`, `export-store.ts`, `machines/export-machine.ts`; `web/src/usecases/start-export.ts`, `import-clip.ts`; `web/src/workers/protocol.ts`, `render.worker.ts`; `web/src/ui/components/ExportButton.tsx`; `web/src/copy/messages.ts`; `web/vite.config.ts`; `scripts/check-copy-codes.mjs`; `web/tests-e2e/export-creator.spec.ts`, `timeline-preserved.spec.ts`; `docs/v5/v5changelog.md`.

### Architecture Requirements

- **INV-9: the profile kind in the key comes from `exportProfile`, verified in the worker.** `planOf` stays for the failed-token announcement and `cachedPlan`, and never reaches `cacheKey`.
- The key is SHA-256 by `crypto.subtle` over a canonical JSON, on the main thread (D-222); INV-17 names decoding, inference, filtering, rendering and encoding, and a hash is none of them.
- `RENDERER_VERSION` is computed (D-223), and reads `"test"` where no bundler ran.
- One entry per clip. A miss deletes the row, then the file, then renders (D-225, TS §23.3's write order). A hit is "the same edit as the last export".
- `render-cache-repo.ts` is the only file naming `renderCache` besides `schema.ts` (and `cancel.spec.ts`'s read). No file name of the user's (P-11).
- `blockers.ts` stays the only place the conditions are evaluated; it changes nothing.
- The machine's table does not change: `gating → rendering → failed` for a refused token (D-230).

### Data Types / Interfaces

```ts
// persistence/render-cache-repo.ts
export type CacheInputs = {
  sourceHashPrefix: string;
  transcript: Transcript;
  events: readonly DetectedEvent[];
  edit: EditState;
  plan: EditPlan;
  timeMap: TimeMap;
  /** "default" until a brand kit (V6.7): a function of the style and the renderer, both in the key (D-224). */
  theme: string;
  /** `null` until music (V6.3). */
  musicId: string | null;
  /** From the token the worker verified, and from nowhere else (D-220, INV-9). */
  profileKind: ProfileKind;
  rendererVersion: string;
};
export function canonicalJson(value: unknown): string;
export function cacheKey(inputs: CacheInputs): Promise<string>;
export function lookup(key: string): Promise<RenderCacheRecord | undefined>;
export function put(key: string, record: RenderCacheRecord): Promise<void>;
export function heldBytes(): Promise<number>;
export function clear(): Promise<void>;
export const RENDERER_VERSION: string;

// state/blockers.ts
export type ExportRoom = { freeBytes: number; neededBytes: number };
export function forExport(room?: ExportRoom): BlockerCode | null;

// state/export-store.ts
export type ExportState = { /* … */ blocker?: BlockerCode; summary?: ChangeSummary; answered: boolean };
export function cacheHit(original: ExportId): void;      // gating → saving
export function blockedBy(code: BlockerCode): void;      // gating → idle
export function saved(summary: ChangeSummary, answered: boolean): void;   // saving → done
export function answered(): void;                        // not a change of state
export function hasAnswered(id: ExportId): boolean;

// persistence/opfs.ts
export function freeBytes(): Promise<number | null>;

// workers/protocol.ts
exportProfile(p: { entitlementToken: string }): Promise<ExportProfile>;
```

### Implementation Requirements

#### 1. `start-export.ts`
§13's twelve steps, in order; G 6.5's table says what to watch at each. `neededBytes = (profile.video_bitrate + profile.audio_bitrate) x clipInfo.duration / 8000`. `exportedAs = profile.kind === "creator" ? "creator" : "free"`.

#### 2. `canonicalJson`
Sorted by key at every depth; `undefined` left out, as JSON does; arrays keep their order. Required: `word_edits` holds its keys in the order they were edited, and the same edit reached by another route must give the same key.

#### 3. `export-store.ts`
`answered: boolean` is required in `Fields`, so `empty()` gives `answered: false`. **`exportCancelled` in `start-export.ts` replaces the whole state and must give it too**; `tsc` will say so. The set of answered ids is a module-level `Set<ExportId>`; `answered()` adds the current `exportId` and sets the field. The set survives `reset`, which is why a hit is not asked again; `saved` takes `hasAnswered` from the caller.

#### 4. §11.3's table, each a test

| Between the two exports | Result | Where |
|---|---|---|
| Nothing | Hit | `timeline-preserved` case 6; Vitest |
| A word edited and then undone | Hit | Vitest |
| Two words edited in the other order | Hit | Vitest (canonical) |
| A word edited | Miss | `timeline-preserved` case 5; Vitest |
| A visual off, the style, the crop, the caption place, Highlight numbers | Miss | Vitest |
| Creator token, then Free token | Miss | `export-creator` (Free after Creator) |
| Creator token, then the same token after `period_end` | Miss | Vitest |
| New deploy with another renderer | Miss | Vitest |
| New import of the same file | Miss | Vitest (`Import`) |
| The stored file deleted by hand | Miss, row deleted | Vitest |
| A forged token | `E_ENTITLEMENT_INVALID`, no lookup | `export-creator`; Vitest |

#### 5. The `export-creator` cases (§17.5)
After "second export", in this order: **same token** (instant: `total_ms` under 2,000; no frames rendered; `from_cache: true`; no new `export_started` or `stage_timing`; SHA-256 equal to the Free file's; one file in storage). **Forged** (`E_ENTITLEMENT_INVALID` before any lookup: no file given, the stored one stays usable). **No room** (`navigator.storage.estimate` stubbed on the page, Creator token, a miss: `export-status` shows the words, nothing swept, `idle`, button enabled; stub deleted). **Expired** (`mintEntitlementToken({ periodEnd: <past> })`: verifies as Free, **hits** the stored Free file, 720 x 1280, watermark present, INV-9; the unit test holds the Creator-file direction).

### Error Handling

| Failure | Result |
|---|---|
| IndexedDB fails in `lookup` | A miss |
| `put` fails | Swallowed: the cache is an optimization |
| `clipsRepo.get` fails | `key = null`: no lookup, no `put`, the export goes on |
| No estimate | Not blocked (D-229) |
| Token refused in `gating` | `clear()`, `export_started` from `planOf`, `fail`, `export_failed` (D-230) |
| Signal aborted in `gating` | `blocked()`: the table has no `gating → cancelled` |
| A stored file vanishes after `lookup` | `E_STORAGE_IO`: `saving → failed`; `export_failed` with no `export_started` |
| `E_STORAGE_QUOTA`, `E_WORKER_CRASH`, `E_ENCODE_*` | As today |

### Concurrency / Async Requirements

`lockForExport` before `exportProfile` (D-221): the worker takes one job at a time. `exportProfile` through `whenFree`. A second click during the gate meets `forExport() !== null`. `cacheKey` is a few milliseconds between two awaits (§11.2).

### Persistence Requirements

`renderCache` rows `{ clipId, exportId, opfsPath, bytes, createdAt, summary }`, wrapped as `Versioned`. `DB_VERSION` stays 1: no build has written a row. `import-clip.ts` clears them; TS §23.3's write order on a miss.

### API Requirements

None. No request added; the hit makes none.

### Security Requirements

- INV-9 by the key; the `forged` and `expired` cases. `exportClip` still verifies itself.
- No file name in a row (P-11). A diff of `canonicalJson`, when debugging, is never printed with a transcript in it.

### Testing Requirements

#### Unit Tests
`export-machine.test.ts`, `blockers.test.ts`, `start-export.test.ts` as §17.3's table. `vi.mock` the repos: Vitest has no IndexedDB. `crypto.subtle` is real. `document`, `URL.createObjectURL` are stubbed.

#### Integration Tests
The `export-creator` and `timeline-preserved` cases by `--grep`. `cancel.spec.ts --grep "export"` for `renderCacheCount`.

#### Failure-Path Tests
G 6.8's four drills. The token-refused, storage-low, row-without-file, quota, worker-gone and cancel groups.

### Validation Commands

```bash
pnpm --filter web exec vitest run src/state src/usecases/start-export.test.ts   # all pass
node scripts/check-copy-codes.mjs                                 # ok; Pending one fewer
grep -rl "__RENDERER_VERSION__" web/dist && echo LEFT || echo replaced          # replaced, after a build
git grep -n "planOf" -- web/src/usecases/start-export.ts          # the helper, cachedPlan, the failed announcement: never near cacheKey
```

### Acceptance Criteria

- [ ] S-9: `start-export.test.ts`'s `render-cache` group covers all ten inputs.
- [ ] §11.3's table is green, each row a test.
- [ ] G problem 2: the `Verified profile` group, `forged` and `expired` pass; the `planOf` drill failed.
- [ ] G problem 3: `RENDERER_VERSION` is computed; the key changes with it.
- [ ] G problem 6: `importClip` clears the cache.
- [ ] A hit's `total_ms` is under 2 s; storage holds one file; D-40's storage cases are unchanged.
- [ ] `B_STORAGE_LOW` at free = needed x 1.2 is not blocked, and one byte under is.
- [ ] `export-machine.test.ts`, `blockers.test.ts`, `start-export.test.ts` pass (BP §7.3).
- [ ] **REOPENED CONTRACT** `exportProfile` is recorded. Tier 3 and the frozen check pass.

### Agent Instructions

Before: read §11, §12, §13, §16, D-220 to D-231, TS §12.5, §21.4, §23.3, G Phase 6. During: write the `render-cache` group first; it is S-9's proof. After: SAB gate at Tier 3; commit `V5 P134 (S-9): render cache on the verified profile, the gate, B_STORAGE_LOW`; **push (second of three); do not wait.**

### Expected Result

The demo's second half is true, and no file reaches someone the token does not entitle.

---

## Prompt 135 (V5 07/08) — J11: `ChangeSummary`, the question, real `export_done` values

**Effort: Medium.** Small interface on existing primitives, specified to the word. Switch to High if the outcome's panel disturbs the tabs' scrolling.

### Objective

After an export the person sees what was changed, and only that, and is asked one question. `export_done` reports the edit's real values, and the founder gets the number PS §20.5 needs.

### Project State Before This Prompt

- First: `gh pr checks <pr>` for P134's run. Red: fix it now, with its own commit.
- `exportStore.saved(summary, answered)` and `hasAnswered` exist; the row holds the summary. `export_done` has placeholders except `from_cache`.
- `SidePanel.tsx`'s `.export` slot holds `ExportFailure`. `PanelSection`, `Button`, `IconButton`, `Check` exist. `dev-hooks.ts`'s `fakeExport` reaches `rendering` only.

### Why This Comes Here

It reads what P134 stored. It comes last before the gate so the capture and the axe scan see the finished state.

### Scope

#### Implement

- G 7.1: `messages.export.summary`, `messages.export.question`.
- G 7.2: `answer-post-export.ts`, `PostExportQuestion.tsx`, `ChangeSummary.tsx` (`summaryRows`, `ExportOutcome`), `SidePanel.tsx`, the CSS. Their rows in TS §13.3 wait for P136.
- G 7.3: `editValues`; `export_done` real in both paths. The last V5 marker goes.
- G 7.4: `start-export.test.ts`'s `Real values` and the question on a hit; `export-creator.spec.ts`'s new case; `timeline-preserved.spec.ts`'s case 5 values and rows.
- G 7.5: `fakeExportDone`; `screens.spec.ts`; `pnpm screens`; `~/v5-review/`.
- G 7.6: the drills.

#### Do Not Implement Yet

- The rows for cuts, music, chapters (V6.2, V6.3, V6.6). Any new token. `metrics.sql` reading the answer (V6). A third design review (D-235).

### Existing Components To Reuse

- `PanelSection`, `Button` (`secondary`, `sm`), `IconButton`, `Check`; the editor's one live region (D-192); `useExportStore`; `track`.
- `whatLeaves` already lists `post_export_answer`. `offcut-api-types/src/analytics.rs` already takes every value. No server change.
- `PostExportAnswer` from `gen/api.ts`.

### Files To Create

- `web/src/usecases/answer-post-export.ts`, `web/src/ui/components/ChangeSummary.tsx`, `PostExportQuestion.tsx`. (All in TS §5.)

### Files To Modify

- `web/src/copy/messages.ts`; `web/src/usecases/start-export.ts`, `start-export.test.ts`; `web/src/ui/layout/SidePanel.tsx`, `ui/styles/components.module.css`, `ui/dev-hooks.ts`; `web/tests-e2e/export-creator.spec.ts`, `timeline-preserved.spec.ts`, `screens.spec.ts`; `docs/v5/v5changelog.md`.

### Architecture Requirements

- Every word from `messages`; every number through a function (INV-15). No absolute claim.
- One row per field; zero absent (`summaryRows` freezes here, §23.1). V6.2 adds one row per field.
- Analytics carries enums and counts only (INV-18). `answer-post-export.ts` needs `state` and `analytics` only: no new use-case edge in `eslint.config.js`.
- Asked once per `ExportId`. A hit shares its export's id. In memory only: a reload asks again.
- Focus does not move when the block appears: the person may be working.

### Data Types / Interfaces

```ts
// ui/components/ChangeSummary.tsx
export function summaryRows(s: ChangeSummary): string[];
export function ExportOutcome(): JSX.Element | null;

// usecases/answer-post-export.ts
export function answerPostExport(answer: PostExportAnswer): void;

// usecases/start-export.ts
function editValues(clip: ClipState): { word_edits: number; events_kept: number; events_disabled: number; style: StyleId; crop_adjusted: boolean };

// ui/dev-hooks.ts
/** Shows the done state with a summary. Nothing was exported (D-235). */
fakeExportDone(): void;
```

### Implementation Requirements

#### 1. Copy (§14.3)
```ts
summary: {
  title: "What we changed", close: "Close", voice: "Voice cleaned",
  emphasized: (n: number) => (n === 1 ? "1 word emphasized" : `${String(n)} words emphasized`),
  visuals: (n: number) => (n === 1 ? "1 visual moment" : `${String(n)} visual moments`),
},
question: { title: "Would you post this as is?", yes: "Yes", smallEdits: "With small edits", no: "No", thanks: "Thanks for telling us." },
```

#### 2. `editValues` (D-237, §15)
```ts
const on = (e: DetectedEvent) => clip.edit.event_overrides[e.id] ?? e.enabled;
const kept = clip.events.filter(on).length;
return {
  word_edits: Math.min(MAX_COUNT, Object.keys(clip.edit.word_edits).length),
  events_kept: Math.min(MAX_COUNT, kept),
  events_disabled: Math.min(MAX_COUNT, clip.events.length - kept),
  style: clip.edit.style,
  crop_adjusted: clip.edit.crop_offset !== 0,
};
```
Computed at step 3 of §13, when the export starts, not when it ends: the person may keep editing.

#### 3. `ExportOutcome`
Renders when `status === "done"` and the person has not closed it. Closing is local, keyed by `exportId`. After an answer the three buttons are replaced by the thanks line. A `section` with `aria-labelledby`. `.export` is a slot above the tabs; the tabs below stay usable and scroll as today.

#### 4. The capture
`fakeExportDone` sets `status: "done"` with a `summary` and `answered: false`. `screens.spec.ts`'s `Window` declaration gains it. Eight states at two widths become nine: 18 PNGs.

### Error Handling

`answerPostExport` does nothing unless `done`, an id, and not yet answered. A summary with no row still shows the question.

### Concurrency / Async Requirements

`answerPostExport` is synchronous; `track` never waits.

### Persistence Requirements

None. The answered set is in memory, in one tab.

### API Requirements

`post_export_answer` is sent through the existing batch. No request added.

### Security Requirements

Counts and enums only. No word of a clip in the summary: it reads numbers from `ChangeSummary`.

### Testing Requirements

#### Unit Tests
`start-export.test.ts`: `word_edits` 2; one override off gives kept and disabled; `style`; `crop_adjusted`; a hit after an answer does not ask again.

#### Integration Tests
`export-creator` "summary and the question": `What we changed` with the voice row, zero rows absent; answer "With small edits": `post_export_answer { answer: "small_edits" }` once; thanks line; `axe` on this state, no serious or critical violation. `timeline-preserved` case 5: `word_edits: 1`, `events_disabled` as made, `style: "bold"`; the rows follow the edit. The "events" case: values as today, since the clip is unedited.

#### Failure-Path Tests
G 7.6's drills.

### Validation Commands

```bash
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"   # above P134's
pnpm --filter web exec vite build --mode test
pnpm e2e:media export-creator.spec.ts --grep "Creator export|events:|summary and the question" --max-failures=1 2>&1 | tail -3
git grep -nE "\bV5\b" -- web/src crates verify                    # no marker of work to do
pnpm screens && ls web/test-results/screens | wc -l               # 18
```

### Acceptance Criteria

- [ ] G problem 4: a switched-off visual is not counted as kept; `export_done`'s values are right after an edit.
- [ ] The rows follow the edit; zero rows are absent.
- [ ] One answer gives one event; a hit on an answered export does not ask again.
- [ ] The state passes axe.
- [ ] `~/v5-review/` holds the done state at both widths, looked at by the agent.
- [ ] No V5 marker of work to do is left (§23.3). Tier 2 and the frozen check pass.

### Agent Instructions

Before: read §14, §15, D-232 to D-237, PS §10 J11, §20.5, G Phase 7, and P134's CI. During: load a design skill if one is available; V4's tokens and primitives win. After: SAB gate at Tier 2; commit `V5 P135: ChangeSummary, the question, real export_done values`. No push.

### Expected Result

The export tells the truth about what changed, and the founder's "Would you post this?" number starts.

---

## Prompt 136 (V5 08/08) — Bench, the full run, copy-back, merge, deploy, tag

**Effort: Medium.** A sequence of runs and records around the founder's merge, look, listen and tag.

### Objective

V5 is measured, recorded, merged, deployed and tagged. This is the one prompt with a full media run, and the one that waits on CI.

### Project State Before This Prompt

- P135's commit. CI of P134 was read at P135's start. `experiments.md` does not exist; §20's first readings are in P130's entry.

### Why This Comes Here

Measure and review on the finished build. D-72 and D-197: the full media suite runs once, before the merge.

### Scope

#### Implement

- G 8.1: Docker closed, keyed production build, `BENCH_DEVICE=d1 BENCH_CLIP=portrait BENCH_RUNS=5 pnpm bench:device`. Commit the file.
- G 8.2: `docs/v5/experiments.md`, §20's table, all `d1`. TS §39.3 item 14. V4 CL known issue 37 closed or reworded.
- G 8.3: the copy-back of §25. "V5 as built" in `v5implementation.md`.
- G 8.4: Tier 4, once. Final counts; the suite's time against V4's 41.8-54.9 min; known issues 33 and 36 read against V5's runs.
- G 8.5: push; **wait** for CI. Red: fix with targeted runs (D-197), push again.
- G 8.6 (**Human:** ready, merge): watch the run; `healthz`; `check-headers`.
- G 8.7: the deployed checks; the demo locally. **Human:** the look and the listen.
- G 8.8: §24's boxes, §23.3; **Human:** the tag; the closing entry.

#### Do Not Implement Yet

- Anything of V6. A non-cosmetic request from the founder becomes a V6 item.

### Existing Components To Reuse

- V4's `experiments.md` as the shape. `check-headers.mjs`, the `@smoke` case, the deploy jobs. `bench/results/d1-*` as the "before".

### Files To Create

- `docs/v5/experiments.md`, `bench/results/d1-<date>-portrait.json`.

### Files To Modify

- `docs/technicalspec.md`, `docs/buildplan.md`, `verify/README.md`, `docs/v5/v5implementation.md`, `docs/v5/v5changelog.md`; `docs/v4/v4changelog.md` (issue 37's note); only what a fix needs.

### Architecture Requirements

- The API deploys first, then the web app, by `ci.yml`. Nothing in the server changed: the image rebuilds with the same code.
- The agent never merges, pushes to `main` or tags (D-194).

### Data Types / Interfaces

None. The bench file holds numbers and words from fixed lists, nothing of the clip.

### Implementation Requirements

#### 1. Order
Quiet bench → readings → copy-back → Tier 4 → entry → push, wait → **Human** merge → deploy watched → deployed checks → **Human** look, listen → fixes if any → boxes → **Human** tag → closing entry.

#### 2. Docker
Closed for the bench (V4 CL issue 37). On for `pnpm test` in Tier 4. Bench first, then Tier 4.

#### 3. Copy-back
From the code, never from memory, into exactly the sections G 8.3 names. Rows 1-25 of §25 are the checklist. `node scripts/check-file-tree.mjs` after §5.

#### 4. The `cacheKey` reading
A temporary `performance.measure`, read once, removed. It is not committed.

#### 5. The closing entry
Final counts (Rust, Vitest, Playwright, media), the merge SHA, the deployed commit, the founder's look and listen in their words, known issues closed or carried, the number of full media runs (one, or each extra explained).

### Error Handling

| Reading | If it misses |
|---|---|
| `render_encode` over 150 s | BP §4.2's Canvas2D path. Stop for the founder. Not expected: V4 read 38.8 s |
| `detect_scene` still over on a quiet machine | An issue for V9 with the numbers, not a V5 blocker |
| App shell over 400 kB gzip | Look at what grew |
| `healthz` not on `$MERGE` in 15 min, or a deployed check fails | Rollback: `git revert -m 1 $MERGE` on a branch from `main`, a pull request for the founder, confirm V4's behaviour is back; fix on `v5-build`; repeat G 8.4 to 8.7 |

### Concurrency / Async Requirements

Bench and media run alone on the GPU. No other push to `v5-build` during `gh run watch`.

### Persistence Requirements

None.

### API Requirements

The deployed API reports the merge commit at `/api/v1/healthz`.

### Security Requirements

The deployed V5 makes no export for a real visitor (BP §7.2). The test key is never in the production build: `ci.yml`'s guard. The agent never queries the production database. No exported file in a report; the file the founder listens to stays on D1.

### Testing Requirements

#### Unit Tests
Whole: `pnpm test`, `python verify/test_verify_mp4.py`.

#### Integration Tests
`pnpm e2e`; `pnpm e2e:media`, once; `@smoke` on the deployed URL; `node scripts/check-hosts.mjs` on a plain `pnpm build`.

#### Failure-Path Tests
The rollback rule, only if needed.

### Validation Commands

```bash
pnpm e2e:media 2>&1 | tail -3                                     # 93 passed, 1 skipped (77 + 16; estimate)
git diff --name-only "$BASE" -- fixtures/golden 'crates/**/*.snap'   # no output
git grep -nE "\bfetch\(" -- web/src | grep -vE "net/http\.ts|net/asset-fetch\.ts|wasm/load-core\.ts|wasm/load-render\.ts"   # no output
gh pr checks <pr>                                                 # every check pass
curl -s "$APP_URL/api/v1/healthz"                                 # 200, the merge commit
node scripts/check-headers.mjs "$APP_URL"                         # exit 0
git ls-remote --tags origin v5                                    # one line (after the founder pushes it)
grep -c "^- \[ \]" docs/v5/v5implementation.md                    # 0, or each named in the closing entry
```

### Acceptance Criteria

- [ ] `bench/results/d1-<date>-portrait.json` is committed (5 runs); `render_encode` is within E-4's line.
- [ ] §20's readings are in `experiments.md`, labelled `d1`; TS §39.3 item 14 holds the verifier's values.
- [ ] The copy-back of §25 is applied.
- [ ] `export-creator.spec.ts` passes with all 11 checks; `timeline-preserved.spec.ts` on both fixtures; the three unit-test files; INV-11's counter is 0 after an export and a cancelled export (BP §7.3).
- [ ] Full media runs in V5: one, or each extra explained. The suite's time is recorded.
- [ ] No golden or snapshot changed. No source file over 700 lines. No request added.
- [ ] **Human:** merged; ci, e2e-media, deploy-api, deploy-web green on `main`; `healthz` reports the merge.
- [ ] On the deployed page: the reference clip previews; Export says the account line; `crossOriginIsolated === true`.
- [ ] Locally: the long-pause fixture exports at 20 s, and the second export is instant.
- [ ] **Human:** one look, one listen, recorded in their words.
- [ ] Every box of §24 and §23.3 ticked.
- [ ] **Human:** `v5` pushed on the merge.

### Agent Instructions

Before: read §20, §24, §25, BP §1.2, §7.3, G Phase 8. During: run what is yours, stop at each Human step and say exactly what is needed. After: SAB gate at Tier 4; commit `V5 P136: bench, experiments, copy-back, final gate`; push; wait. The closing entry goes after the tag. Reply with the deployed state, each box's evidence, and anything carried to V6.

### Expected Result

V5 is deployed, measured, recorded and tagged. V6's first morning has every line of §23.3 true.

---

# Cross-Prompt Contracts

| Contract | Owner → consumers | Shape | Compatibility / change rule |
| --- | --- | --- | --- |
| Baseline and counts | Setup → every gate | `BASE=0a8aa80`; 646/0/4 Rust, 175 Vitest, 12 Playwright, 77 + 1 skip media | Counts only rise, except SAB rule 4's named rewrites |
| The frozen check | Setup → every gate | G 0.5's path list | No diff under those paths; a needed one is stopped and reported |
| The tiers | Setup → every prompt | §6.1 | A prompt runs its tier; the full suite once |
| `openShared`, `resetClip`, `sendQueuedAnalytics`, `forget` | P129 → V6.x's review cases | §6.3 | Frozen meaning: a clean clip, or the case fails by name |
| `changes` job | P129 → every later pull request | `code` output; scan inside | A push to `main` always runs everything |
| `BENCH_RUNS` | P129 → P136, V9 | Default 10; V5 reads 5 | V9 and R1/R2 read 10 |
| `verify_mp4.py`'s command line and lines | P130 → every export test, V9's corpus | `PASS n`, `FAIL n: …`, `INFO n: …`; `--expected-duration-ms`, `--source` | Frozen (§23.1). A `FAIL 11` names the key, never the value |
| `verifyMp4`, `expectAllChecks`, `Verdict` | P130 → P133-P136, V6 | §17.5 | Additive |
| `watermark_mask_720.png` | P130 → check 9 | 720 x 1280 grey; tool reads the font as data | Re-run the tool when `watermark.rs` or `safe_area.rs` moves the text |
| `integrated()` reads `DualMono` | P130 → chain, `program.rs`, V6.3's mix | -14 means a stereo file | Copied to TS §18.2 at P136 |
| `add_video_sample`'s `frame` is the presentation index | P131 → `Writer.video` | Signature unchanged | A file with no reordering stays byte-identical |
| `ok_bframes.mp4` | P131 → `mux_reorder.rs`, `demux_fixtures.rs` | 26th fixture | Manifest row |
| `clips/<id>/prog48.f32`, `writeProgram`, `readProgram` | P132 → `start-export`, V6.2, V6.3, V7 | Same shape as `out48`'s | `out48.f32` stays |
| `exportClip`'s `prog48` | P132 → `start-export` | REOPENED | Frozen again |
| `runExport`, `attempt`, `supportedConfigs`, `OpfsSink.reset` | P132 → P133, V6.x | §10.3 | Attempt-and-ladder shape frozen |
| `failEncoderAt`, `takeEncoderFault`, `DEV_MARKERS` | P132 → P133 | D-218 | Dev builds only; `check-hosts.mjs` fails on the name in `dist` |
| `watchExport` | P133 → both export specs | Moved to helpers | Additive |
| `CacheInputs`, `canonicalJson`, `cacheKey` | P134 → V6.1, V6.2, V6.3, V6.7 | Ten fields | V6.x may set them; a new input is appended |
| `RENDERER_VERSION` | P134 → the key | 16 hex; `"test"`, `"none"` | Computed, never by hand |
| `exportProfile` | P134 → `start-export`, V6 | REOPENED; keeps nothing | Frozen again; the only source of a key's profile |
| `forExport(room?)` | P134 → `ExportButton`, `start-export`, V6 | §12.1 | Order frozen; V6 appends three blockers |
| `export-store` actions, `hasAnswered` | P134 → P135 | §12.3 | Additive |
| `summaryRows` | P135 → V6.2, V6.3, V6.6 | One row per field, zero absent | Rule frozen |
| `editValues`, `export_done` | P135 → V6.1-V6.7 | §15 | Fields appended |

# High-Risk Implementation Areas

| Risk | Why it matters | Prompt responsible | Validation strategy |
| --- | --- | --- | --- |
| An expired or forged token hits a stored 1080p file | A Free file's rights reach a Creator file, with no watermark (INV-9) | 134 | Verified kind in the key; `Verified profile` group; `forged` and `expired` cases; the `planOf` drill |
| The cache serves an old look after a deploy | Wrong output forever on that device | 134 | Computed `RENDERER_VERSION`; key test; `grep` of `dist` |
| The stereo fix changes more than loudness | The clip's sound is the product | 130 | `duration_preserved.rs`, `chain_no_clicks.rs`, `program_identity.rs` frozen and passing; check 7; the founder listens |
| Check 8 or 9 is tuned to pass one file | The judge stops judging | 130 | Offline readings recorded; the 0.3 gap rule; thresholds only with TS §39.3; a self-test bad file per check |
| Shared clip leaks state between cases | A flake that looks like an app bug, which would cost V4's time again | 129 | `resetClip` asserts and fails by name; `sendQueuedAnalytics`; `--repeat-each` on two cases; drills |
| The verifier prints a source's location | `v3testclip.mp4` carries GPS | 130 | `FAIL 11` names the key only; read in the self-test |
| A restart leaves the source, encoders or a file in a bad state | A corrupt file, or frames left open (INV-11) | 132, 133 | `attempt`'s `finally`; `sink.reset`; `liveFrames` in every Vitest case; `failEncoderAt(30, 1)` through the verifier |
| `ctts` plays wrongly somewhere | D1 does not reorder, so the first real case is a user's | 131 | `mux_reorder.rs` on real B-frames through `ffmpeg`; phone playback is V10's checklist |
| A file with no reordering changes | Every export today would change | 131 | Byte equality asserted |
| A CI-only failure surfaces later, because the agent did not wait | Time lost after the fact | 129, 134 | The next prompt reads CI first, and fixes before going on (D-198) |
| A docs-only pull request breaks the TS §5 tree | Caught only on `main`, where it blocks a deploy | 129, 136 | `node scripts/check-file-tree.mjs` by hand before pushing §5 |
| Time overrun | D-197's point | any | Cut in this order: `ok_bframes`'s `ffmpeg` half, then `failEncoderAt(30, 9)`. Never check 7, 8 or 9, or the cache's verified key |

# Architectural Invariants

1. **The judge shares nothing with what it judges** (TS §27.2): the verifier imports nothing from `crates/` or `web/`, and no library they use; the mask is drawn by a tool of its own.
2. **The product is fixed, not the check** (D-204). A threshold moves only under §7.3's rule, never to pass a file.
3. **Output size and watermark come from the verified token only** (INV-9), and so does the cache key's profile (D-220).
4. **The export is exactly as long as the map says** (INV-5, INV-8, INV-10). Every output frame once; no sample removed; padding only.
5. **Every `VideoFrame` is closed** (INV-11), through a restart and a cancel.
6. **A file with no reordering is byte-identical to today's** (D-211).
7. **`blockers.ts` is the only place an export's blockers are evaluated** (TS §12.5).
8. **Every state change through `transition()`** (INV-16); the export machine's table does not change (D-230).
9. **Only changes that were made are shown** (PS J11, D-233). All copy from `messages.ts` (INV-15).
10. **Analytics carry enums and counts only** (INV-18); V5 adds no request (TS §1.4).
11. **The main thread does no media work** (INV-17); a hash is not media work (§11.2).
12. **Nothing is reserved beyond S-8 and S-9** (D-239). Goldens and snapshots do not change.

# Forbidden Implementation Shortcuts

- No measuring a downmix in the verifier to make check 7 pass. No moving a threshold to make one file pass. No narrowing check 8's windows.
- No `planOf` in the cache key. No hand-written `RENDERER_VERSION`. No `JSON.stringify` for the key.
- No `#[ignore]`, `test.skip` or `test.fixme` to get past a failure, except the one stated case of G 5.3, with its reason in its message.
- No widening a targeted run to a full one. No second full media run without its reason in the changelog.
- No changed golden or snapshot. No edit under the frozen paths.
- No new dependency. Pillow is one-time on D1, and not in `requirements.txt`.
- No exported file, `zz-` file, `testclips/`, generated fixture, screenshot or Playwright report committed; no exported file left in `OFFCUT_KEEP_EXPORTS`; no key in a file.
- No word of a clip printed. No tag value from a source printed.
- No `unwrap`, `expect`, `panic!` or indexing in non-test Rust; no `any`, `!`, `@ts-ignore`, empty `catch`; no disabled lint; no skipped or weakened test.
- No push to `main`, merge, tag, `gh pr ready` or production database query by the agent; no branch pushed but `v5-build`; no push where a prompt says none.

# Requirement → Prompt Traceability

| Requirement (BP §7, TS §28 M1.6, TS §40.9, §1A, the founder's speed rules) | Prompt(s) | Implementation | Validation |
| --- | --- | --- | --- |
| `export-loop.ts` complete | 132 | `attempt`, ladder, backpressure, cancel, frames closed | `export-loop.test.ts`, verifier |
| `mux.rs` complete: `ctts`, reserve | 131 | §9 | `mux_roundtrip.rs`, `mux_reorder.rs` |
| `profile.rs` in use | 134 | `exportProfile` | `Verified profile`, `expired` |
| `blockers.ts` complete, `blockers.test.ts` | 134 | V5's half of §12.5; V6 adds three | `blockers.test.ts` |
| `export-machine.test.ts` | 134 | Table unchanged, `cache_hit` fired | `export-machine.test.ts` |
| `start-export.ts` complete, `start-export.test.ts` | 132, 134, 135 | §13 | `start-export.test.ts` |
| Download name | 134 | `downloadName` unchanged | `Download` group; `download name` case |
| `render-cache-repo.ts` | 134 | §11 | `render-cache` group |
| `ChangeSummary`, `PostExportQuestion`, `answer-post-export` | 135 | §14 | `export-creator` summary case, axe |
| `verify_mp4.py` 7-11, mask | 130 | §7 | Self-test, real exports |
| `export-creator.spec.ts` complete | 130, 134, 135 | 11 checks | Tier 2, Tier 4 |
| `timeline-preserved.spec.ts` | 133, 134, 135 | Both fixtures | 600 and 900 frames |
| S-8 | 132 | §10 | `export-loop.test.ts` seam, verifier 5 and 8 |
| S-9 | 134 | §11 | `render-cache` group, ten inputs |
| E-4 re-measured | 136 | §20 | `bench/results/`, `experiments.md` |
| INV-11 after export and cancel | 132, 136 | `liveFrames` | `export-loop.test.ts`; `cancel.spec.ts` |
| Demo: 20 s in, 20 s out; same edit twice | 133, 134, 136 | §17.5 | Cases 4, 6; G 8.7 |
| G problem 1 (exports too loud) | 130 | D-204 | "Before" read; check 7; stereo test; listen in 136 |
| G problem 2 (unverified plan in the cache) | 134 | D-220 | `Verified profile`, `forged`, `expired`, drill |
| G problem 3 (`RENDERER_VERSION`) | 134 | D-223 | Key test; `dist` grep |
| G problem 4 (`export_done` count) | 135 | D-237 | `Real values`; `timeline-preserved` case 5 |
| G problem 5 (no B-frame fixture) | 131 | D-212 | `real_b_frames_come_back_at_30_fps` |
| G problem 6 (re-dropping the same file) | 134 | D-228 | `Import` case; bench readings |
| G problem 7 (`v4` tag) | Setup | Human | `git ls-remote` |
| G problem 8 (V4's boxes) | Setup | Entry 40 | Named in the entry |
| G problem 9 (no CI without a PR) | Setup | Human | `gh pr view` |
| G problem 10 (thresholds are guesses) | 130, 136 | Offline tuning | `INFO` lines; TS §39.3 |
| Speed rules | 129, every gate, 136 | D-196 to D-201 | 5 clip loads; one full run; docs-only skip |
| Known issues 33, 36, 37 | 129, 136 | D-236 | V5's runs; bench |
| Spec copy-back | 136 | §25 | `check-file-tree`, G 8.3 |
| Deployed, verified, tagged | 136 | Merge, deploy, checks | `healthz`, headers, `@smoke`, `ls-remote` |

# Final Coverage Audit

- [x] **Every BP §7 item and both seams have a prompt** (table above), including the non-code items: the bench (136), the look and the listen (136).
- [x] **Every file of §4 is named in a prompt:** `loudness.rs`, `loudness_target.rs` (130); `mux.rs`, `mux_boxes.rs`, `mux_roundtrip.rs`, `mux_reorder.rs` (131); `gen_fixtures.sh`, `manifest.json` (131); `verify/*` (130, 136); `messages.ts` (134, 135); `opfs.ts` (132, 134); `render-cache-repo.ts`, `schema.ts` (134); `blockers.ts`, `blockers.test.ts`, `export-store.ts`, `export-machine.ts`, `export-machine.test.ts` (134); `start-export.ts` (132, 134, 135); `start-export.test.ts` (134, 135); `answer-post-export.ts` (135); `run-pipeline.ts`, `run-pipeline.test.ts` (132); `import-clip.ts` (134); `dev-faults.ts`, `dev-hooks.ts` (132, 135); `ChangeSummary.tsx`, `PostExportQuestion.tsx`, `SidePanel.tsx`, `components.module.css` (135); `ExportButton.tsx` (134); `ExportProgress.tsx` (132); `protocol.ts` (132, 134); `rpc-worker.ts` (132); `render.worker.ts` (134); `export-job.ts`, `export-loop.ts`, `export-loop.test.ts`, `encoders.ts`, `opfs-sink.ts` (132); `vite.config.ts` (134); `playwright.config.ts` (133); `helpers/*` (129, 130, 133); `export-creator.spec.ts` (130, 133-135); `timeline-preserved.spec.ts` (133-135); `review-edit.spec.ts`, `model-download.spec.ts` (129); `screens.spec.ts` (135); `device-bench.ts` (129); `bench/results/` (136); `ci.yml` (129, 130); `e2e-media.yml` (129); `check-copy-codes.mjs` (134); `check-hosts.mjs` (132); docs (Setup, 136).
- [x] **Every decision is implemented:** D-194 (every prompt); D-195 (Setup); D-196, D-197 (every gate, 136); D-198 (129, 134, 136); D-199 (129); D-200 (129, 136); D-201 (129); D-202 (136); D-203 to D-209 (130); D-210 (130, 133); D-211, D-212 (131); D-213 to D-219 (132); D-220 to D-231 (134); D-232, D-233 (135); D-234 (134, 135); D-235 (135, 136); D-236 (129, 136); D-237 (135); D-238 (132); D-239 (every prompt: nothing else is built).
- [x] **Every reading of §20 has a prompt:** loudness, sync, watermark, bitrate (130, 136); hit time (134, 136); E-4, `mux`, `detect_scene`, `cacheKey`, app shell (136).
- [x] **Tests are incremental:** each Rust test lands with its code (130, 131); Vitest with its code (132, 134, 135); each E2E case after what it checks (130, 133-135); each prompt runs its tier.
- [x] **Persistence, configuration, errors, security, performance, observability, deployment** are covered: `prog48.f32`, `renderCache` (132, 134); constants (§ Appendix C; 130, 131, 134); errors and copy (132, 134); privacy and secrets (SAB 9-10; 130); budgets (136); analytics (134, 135); CI and deploy (129, 130, 136).
- [x] **Documentation:** a short changelog entry per prompt; `experiments.md` (136); spec copy-back (136); "As built" (136).
- [x] **Prompt dependencies are valid:** every prompt needs only earlier prompts, the V5 documents and the listed Human steps.
- [x] **Eight prompts (129-136) and a Setup; Prompt 136 yields a deployed, verified, recorded and tagged V5** with every box of §1A, §23.3 and §24 ticked.

# Gaps settled by these prompts

Found while reading `v5implementation.md`, `v5buildguide.md` and the code together. Each is applied in the prompt named. Those that changed the plan are its §25 rows 20 to 25.

| # | Gap | Settled as | Prompt |
| --- | --- | --- | --- |
| 1 | §6.3 asserted "the Clean card selected", which needs the Style tab and starts a thumbnail job that could collide with the next case | The stored edit, by `readIndexedDb`, deep-equals the empty one | 129 |
| 2 | `flushAnalytics` needs `page.clock`, not installed on the shared page; a fake clock would touch every timer of 13 cases | `sendQueuedAnalytics`, by `visibilitychange` | 129 |
| 3 | `PreviewPlayer.tsx` also listens for `visibilitychange` | It reads `document.hidden`, which the override does not change, and the preview is paused first | 129 |
| 4 | §4's `BENCH_TIMEOUT_MS` was a fixed 120 min for eleven runs | `(RUNS + 1) * 11 * 60_000` | 129 |
| 5 | §8's -11 LUFS is inferred, and could not be read once fixed | A real export is read first, on P129's build | 130 |
| 6 | `exportCancelled` replaces the export store's whole state, and `answered` is required in it | It gives `answered: false`; `tsc` enforces it | 134 |
| 7 | A serial group's `--grep` runs only what it names, and "same token" needs a stored file | The cases' grep names `Free-plan` and `second export` too | 134 |
| 8 | §11.3's "a stored file deleted by hand" and "a new deploy" cannot be made in a browser test | Vitest cases, each named in the table of P134 | 134 |
| 9 | `exportedAs` when `profile.kind` is `preview` | A token never verifies as `preview`; mapped to `free`, so the type holds | 134 |
| 10 | Eight screen states at two widths become nine | 18 PNGs; only the done state's two go to `~/v5-review/` | 135 |
| 11 | `pnpm test` needs Docker, the bench needs it closed | Bench first, Docker closed; then Docker on for Tier 4 | 136 |
| 12a | Case 14's budget on a shared clip, an ambiguous `Undo`, `--repeat-each` in a shared group, a backslash in `--grep` (found running P129) | Plan §25 rows 26, 27 | 129 |
| 12 | `v5-build` does not exist, and the documents are uncommitted on `v4-build` | They go in the Setup commit, with V4's Entry 40, since V4 is merged | Setup |
