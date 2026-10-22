# Offcut V4 — coding-promptsv4.md (Prompts 91–128)

Source documents: `docs/v4/v4implementation.md` (written **§n**; its decisions are **D-n**, D-113 to D-193) and `docs/v4/v4buildguide.md` (written **G n.m** for Step n.m, **G M-n** for Milestone n, **G item n** for an item of its "Read this first", **G (b) n** for a row of its table "Gaps and contradictions found", **G (c)** for its table of placements that differ from §5). `TS §n` = `docs/technicalspec.md` (spec v1.2; §40 is the edit-plan architecture), `PS §n` = `docs/product.md`, `BP §n` = `docs/buildplan.md`, `V3 §n` = `docs/v3/v3implementation.md`, `V3 CL item n` and `V4 CL item n` = a row of the "Open items" or "Known issues" table of `docs/v3/v3changelog.md` or `docs/v4/v4changelog.md`. Both V4 documents were read end to end against each other, against the V4 changelog (Entries 0 to 0.4) and against `scripts/check-file-tree.mjs` on 2026-10-22; these prompts were written from that reading. Where they settle something the two documents leave open, the prompt says so and the closing section "Gaps settled by these prompts" lists it.

**How to use.** Paste the **Standard Agent Block (SAB)** below, then one prompt, into your coding agent. Run the prompts in order. Do not start Prompt N until every "Acceptance Criteria" box of Prompt N-1 is ticked, its commit exists on `origin/v4-build`, and CI on pull request #7 is green. Steps marked **Human** are yours (see "Who does what"). V1 was Prompts 01 to 30, V2 31 to 60, V3 61 to 90; V4 is **thirty-eight prompts, 91 to 128**, each also labelled with its place in V4 (01/38 to 38/38). Commits are titled `V4 P<nn> (S<n>): <title>` (D-115 with the guide's S-number, G "How the guide is organised").

**Recorded deviation from TS §34.1 and BP §1.3, extended to V4.** Those sections give each file its own specification. As in V1 to V3, the founder accepts the multi-file prompt format; the per-file contract is honoured by naming every file and implementing every signature §6 to §17 give exactly (§ Appendix A is the per-file brief, Appendix C writes two out in full).

**The order is the guide's.** The guide's twenty phases are followed one by one; a phase too large for one session is split across prompts at a step boundary, never inside a step. Five rules decide the order (D-116, G items 1-3, 6, 8): **contracts first and proven to change no pixel** (Prompts 92-100: the identity reference is written by V3's code in Prompt 93 and must match in Prompt 100); **the line-break table before the algorithm** (Prompt 102 writes it, the founder reads ten rows, Prompt 103 writes the search); **goldens after the look** (two video-only goldens in Prompt 101, the other fourteen in Prompt 107); **the design reviewed before the editor is wired** (Prompts 108-110 build the shell, Prompt 111 is design review 1, Prompts 112-117 wire it); **measure and review on the finished build** (Prompts 126-128).

**Three human gates.** Prompt 102 ends with ten rows of the line-break table for the founder to read (G 8.2). Prompt 111 is design review 1 on sixteen captured screens (G Phase 11). Prompt 128 holds design review 2 on the deployed editor and the reading of five goldens (G 19.5). A gate is a milestone: the next prompt does not start until it passes. **D1 is the reference machine (D-114):** every reading is taken on D1 by the agent and recorded under the label `d1`.

**Effort (for the human).** Each prompt has an **Effort** line under its heading: the reasoning effort to set on the coding agent before pasting it. **High** (29 prompts: 93-109, 112-116, 118, 119, 121-124, 127) is for contract switches across languages, anything that must be byte-identical to V3, numerical and layout code, the caption intelligence, the design system and every interaction, concurrency (RPC, cancel, crash), and the suites likely to expose app bugs. **Medium** (9 prompts: 91, 92, 110, 111, 117, 120, 125, 126, 128) is for records, exactly specified configuration, tooling, CI, runs of measurements and the gates whose judgement is the founder's. Two Medium prompts say when to switch to High partway (111, 117). When in doubt the rating is High. The line is a note for you; it changes no instruction to the agent.

**Design skill.** The founder's brief (§2.10) makes the interface's design a gate. In Prompts 108-111, 113, 115-117, 119 and 127, if the coding agent has a frontend or design skill available, it loads it before writing interface code. The tokens of D-179, the primitives of D-181 and the copy of §17.5 still win over any default a skill suggests; a skill's suggestion that changes a token goes to the founder at a design review, not into the code.

---

# Standard Agent Block (SAB) — prepend to every prompt

**Every terminal (Git Bash, never PowerShell or WSL)**

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut && set -a; . ./.env; set +a      # the repository is offcut/, not sh2clips/; .env has the 15 local values only
export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:/c/Users/Mayan/AppData/Local/Microsoft/WinGet/Links:$PATH"
export BENCH_DEVICE=d1
BASE=550ff8f                                                                # the V4 code baseline every frozen-file check compares with (§1A item 3)
git branch --show-current                                                  # v4-build, always
```

Docker Desktop must be running before `cargo test --workspace` (the server's tests use the local Postgres on port 9000). `ffmpeg -hide_banner -version | head -1` prints 9.0.x; `gh auth status` shows a login.

**Before modifying anything**

1. Read the plan sections and guide steps this prompt cites, and every file it touches. Read the previous prompt's entry in `docs/v4/v4changelog.md`.
2. Search the repository for existing types, helpers and constants before creating any (`git grep -n "<name>"`). Reuse them. Never create a second home for a value that has one (Appendix D names the home of every V4 constant).
3. Verify that the previous prompts' outputs exist and behave as this prompt assumes (each prompt lists what to verify under "Project State Before This Prompt"). If not: decide whether it is a bug or an intended boundary, make the smallest compatible correction, and report it in the changelog entry.
4. **Precedence.** A decision D-113 to D-193 of §2 is binding, also where it refines a TS section (it says which); D-1 to D-112 stand unless a V4 decision replaces them. Otherwise TS (v1.2) wins over the plan, and PS over both. The guide gives the order of work and the commands; where the guide and §5 place a step differently, the guide wins (G (c)). If two sources disagree in a way none of these settles, or a signature is missing, **stop and list it**. Do not guess (TS §34.1).

**During implementation**

1. Implement only this prompt's scope ("Implement"); leave everything under "Do Not Implement Yet" alone even if it looks easy. Create only the files the prompt names. A new file under `crates/`, `server/`, `web/`, `scripts/`, `verify/`, `corpus/`, `bench/` or `.github/` that the TS §5 tree does not list fails `node scripts/check-file-tree.mjs`; Prompt 92 lists every V4 file, so a file a prompt adds that is not there means the tree changes in the same commit. `docs/` and `fixtures/` are not checked by the script; their TS §5 lines are still kept true by hand (D-113).
2. **Untouched in V4** (§4, last paragraph; the frozen check below enforces it): every file under `server/`; `crates/offcut-entitlement/`, `crates/offcut-mp4/`, `crates/offcut-text/`, `crates/offcut-detect/`; `offcut-api-types/src/{lib,errors}.rs`; `offcut-dsp/src/{chain,config,highpass,denoise,compressor,limiter,loudness,prosody,resample}.rs` and `offcut-dsp/tests/` except the new `program_identity.rs`; `offcut-wasm-core/src/{media_api,hash_api,text_api}.rs`; `offcut-wasm-render/src/{lib,profile_api,mux_api,detect_api}.rs` (except the one line `mod scene_api;` in `lib.rs`, Prompt 94); `web/vercel.json`; (`web/src/workers/{media,asr}.worker.ts` change by their import line only, Prompt 94); `web/src/{persistence,net,models,platform}/`; `web/src/analytics/client.ts`; `web/src/state/{capability-store,model-store,export-store,blockers}.ts`; `web/src/state/machines/{transition,clip-machine,model-machine,export-machine}.ts`; `web/src/workers/{media.worker,asr.worker}.ts`, `web/src/workers/{media,asr}/`, `web/src/workers/render/{encoders,opfs-sink}.ts`; `verify/`; `bench/device-bench.ts`; `corpus/`; `fixtures/{gen_fixtures.sh,manifest.json,labeled,speech}`; `render.yaml`.
3. **The reopened contracts** are exactly these, each said as **REOPENED CONTRACT** in the changelog entry of the prompt that touches it and closed in the "Known issues" row Prompt 92 opens for it: `offcut-types/src/units.rs` (D-125, P95); `offcut-types/src/edit.rs` `EditState` (D-169, D-171, P95); `offcut-scene/src/display_list.rs` `FontId` and `DrawCmd` (D-130, D-132, P98); `offcut-scene/src/styles.rs` (D-136, P98); `Renderer::render` (D-133, P98); `web/src/workers/protocol.ts` (D-124, D-126, D-127, D-128, D-174, D-186; P99, P100, P113, P116); `AudioWorkerApi` (D-127, D-128, P100); `web/eslint.config.js` `MINTING` (D-126, P99) and the use-case edges (§4, P114); `offcut-api-types/src/analytics.rs` `ReviewActionKind` (§13, P120).
4. **Expectations V4 may rewrite** (D-160), and no others: `pipeline-preview.spec.ts` "Preview plays" and "Events"; the calls (not the expected values) of `offcut-scene`'s inline tests in `lib.rs`, `captions.rs`, `framing.rs`, `number_reveal.rs`, except `framing.rs`'s "any offset" case, whose expectation now asserts the offset. **Between Prompts 98 and 99** the media cases that reach `setScene` fail because the page still sends `events`; that is expected and Prompt 99 ends it. Any other failure is a regression of the prompt that caused it.
5. No source file over 400 lines above its test module (`check-file-tree.mjs` counts `.rs .ts .tsx .js .mjs .cjs .sh .css .py .sql`). Rust non-test code: no `unwrap`, `expect`, `panic!`, slice indexing, `HashMap` in a pure crate, clock, randomness, global state (the `thread_local!` font and shaping caches of D-132 and D-173 are the named exceptions) or I/O in a pure crate. TypeScript: no `any`, no `!`, no `@ts-ignore`, no empty `catch`, no `as <Brand>` outside a `MINTING` helper, no inline style (CSP `style-src 'self'`). No disabled lint, no `#[allow]`/`eslint-disable` added to pass a check, no weakened, skipped or deleted test (the only `#[ignore]`s are V3's three, `scene_timing`, and the temporary one of Prompt 98 that Prompt 100 removes). No dependency the prompt does not name; pin each new one at its latest stable release and record the version in the changelog (§3.2).
6. **Test files V4 may add** (§4), and no others: `crates/offcut-plan/tests/{validate,plan_from_rules,validate_rules_plans,catalog_snapshot,ui}.rs` and `tests/ui/valid_plan_private.rs` (+ its `.stderr`); `crates/offcut-scene/tests/{display_list_snapshots,layout_safe_area,framing,determinism,styles_use_theme,timemap_identity,catalog_complete,line_breaks,parity,placement,scene_cache,hit}.rs` and the temporary `v3_reference.rs`; `crates/offcut-render/tests/golden_frames.rs` and `tests/common/ssim.rs`; `crates/offcut-dsp/tests/program_identity.rs`; `web/src/workers/rpc.test.ts`, `web/src/state/history.test.ts`, `web/src/state/machines/{clip,model}-machine.test.ts`, `web/src/usecases/run-pipeline.test.ts`; `web/tests-e2e/{preview-sync,review-edit,cancel,screens}.spec.ts`. Every other Rust test is an inline `#[cfg(test)]` module.
7. Scripts are POSIX `sh` with LF line endings. A temporary file is named `zz-…` or `zz_…` and is deleted before the gate; nothing temporary is committed. A temporary Playwright check needs `web/tests-e2e/zz-<name>.spec.ts` plus a temporary `web/zz-pw.config.ts` that spreads the real config and sets `projects: [{ name: "zz", testMatch: "zz-*.spec.ts", use: { channel: "chrome" } }]`. Files written by Python on Windows are normalised to LF (`sed -i 's/\r$//' <file>`) before `git add`.
8. A TODO carries the version that removes it (`V5:`, `V6.1:`). User-facing text exists in `copy/messages.ts` only (INV-15), including tooltips, `aria-label`s and toasts; numbers are interpolated. `fetch` exists in the four allowed files only; V4 adds no request (D-164). No cut, mix or non-identity `TimeMap` is constructed outside tests (D-164). Snapshots and goldens are reviewed, never blessed (G item 5): a `.snap.new` is read before it is accepted; a changed snapshot or golden after its first commit gets a changelog line with the reason.
9. **Privacy.** Never stage, commit, upload or paste anywhere: `testclips/**`, `fixtures/.gen/**`, `fixtures/.cache/**`, `corpus/.clips/**`, `corpus/.results/**`, `web/test-results/**` (the design-review screenshots live there and stay on D1), `web/playwright-report/**`, `.env*`. `testclips/v3testclip.mp4` carries a GPS location and never leaves D1. A transcript of a clip lent by anyone but the founder is never written to the repository, a result file, a log or a reply. Golden frames are drawn from a synthetic pattern, never a recording (D-135).
10. **Secrets.** No seed, token, key or connection string is printed, logged or written to a file. `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` is exported only in the terminal that builds for the media suites (worked out from the seed in `web/tests-e2e/helpers/fake-api.ts`, as V2 and V3 did), never in a `.env*` file, and unset before the gate. Production values live in `.env.deploy`; the agent never queries the production database.
11. **Long runs** (`pnpm e2e:media` about 15-25 minutes in V4, the bench about 25, `gh pr checks 7 --watch` about 30): start detached, `( cmd > "$LOG" 2>&1; echo "exit=$?" >> "$LOG" ) > /dev/null 2>&1 &`, then poll the log with a bounded loop. Append to the changelog with the Edit tool, or by writing a scratchpad file and `cat >>`; never a heredoc holding backticks (Bash quoting breaks on them).
12. **Branch and push (D-115).** Commit on `v4-build` only (`git branch --show-current` before every commit), push to `origin v4-build` yourself after each prompt's commit, and read CI with `gh pr checks 7 --watch`; read a failing job with `gh run view <run-id> --log-failed`. Never push to `main`, never merge, never tag, never mark the pull request ready: those are **Human** steps. Do not perform any other **Human** step (a founder's decision, reading or judgement; a dashboard; a production database read). List it in your reply.

**When existing code is inconsistent with the plan.** Do not rewrite it blindly. If it blocks this prompt, fix the minimum and record why. If it does not, leave it and report it under "Found, not fixed" in the changelog entry.

**After implementation: the gate.** Every prompt ends with all of it, in this order (a prompt that changes no tracked file says so and skips the commit).

```bash
git branch --show-current                                                  # v4-build
git status --porcelain | grep -c "zz[-_]"                                  # 0: nothing temporary is left
env | grep -c VITE_ENTITLEMENT_TEST_PUBLIC_KEY                             # 0: the gate build carries no test key
pnpm fixtures                                                              # "fixtures are up to date" or a fresh build
pnpm check && pnpm test && pnpm build && pnpm e2e                          # all four green (pnpm check includes check-file-tree, check-gen-clean, check-copy-codes)
git diff --name-only "$BASE" -- \
  server crates/offcut-entitlement crates/offcut-mp4 crates/offcut-text crates/offcut-detect \
  crates/offcut-api-types/src/lib.rs crates/offcut-api-types/src/errors.rs \
  crates/offcut-dsp/src/chain.rs crates/offcut-dsp/src/config.rs crates/offcut-dsp/src/highpass.rs crates/offcut-dsp/src/denoise.rs \
  crates/offcut-dsp/src/compressor.rs crates/offcut-dsp/src/limiter.rs crates/offcut-dsp/src/loudness.rs crates/offcut-dsp/src/prosody.rs \
  crates/offcut-dsp/src/resample.rs crates/offcut-dsp/tests \
  crates/offcut-wasm-core/src/media_api.rs crates/offcut-wasm-core/src/hash_api.rs crates/offcut-wasm-core/src/text_api.rs \
  crates/offcut-wasm-render/src/lib.rs crates/offcut-wasm-render/src/profile_api.rs crates/offcut-wasm-render/src/mux_api.rs \
  crates/offcut-wasm-render/src/detect_api.rs \
  web/vercel.json web/src/persistence web/src/net web/src/analytics/client.ts web/src/models web/src/platform \
  web/src/state/capability-store.ts web/src/state/model-store.ts web/src/state/export-store.ts web/src/state/blockers.ts \
  web/src/state/machines/transition.ts web/src/state/machines/clip-machine.ts web/src/state/machines/model-machine.ts \
  web/src/state/machines/export-machine.ts \
  web/src/workers/media.worker.ts web/src/workers/asr.worker.ts web/src/workers/media web/src/workers/asr \
  web/src/workers/render/encoders.ts web/src/workers/render/opfs-sink.ts \
  verify bench/device-bench.ts corpus fixtures/gen_fixtures.sh fixtures/manifest.json fixtures/labeled fixtures/speech render.yaml
# the frozen check (G 1.6): no output, except seven recorded one-line diffs from Prompt 94 on (check each with git diff "$BASE" -- <file>):
# crates/offcut-wasm-render/src/lib.rs (`mod scene_api;`), web/src/workers/media.worker.ts and asr.worker.ts (the import from ./rpc-worker, D-150),
# and, found in Prompt 94 (v4changelog Entry 4), the same import line of web/src/workers/media/import.ts, media/audio-decode.ts,
# asr/whisper-runtime.ts and render/encoders.ts; and crates/offcut-dsp/tests/program_identity.rs from Prompt 100 on (a new file)
git ls-files --cached | grep -E '^(testclips|fixtures/\.gen|fixtures/\.cache|corpus/\.clips|corpus/\.results|web/test-results|web/playwright-report)/|(^|/)\.env'   # no output
```

1. Read the test counts: Rust passed, failed and ignored (`cargo test --workspace 2>&1 | awk '/^test result:/ {p+=$4; f+=$6; i+=$8} END {print p" passed, "f" failed, "i" ignored"}'`), Vitest passed (`pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"`), Playwright passed. None may be lower than in the previous changelog entry. The baseline (Entry 1) is V3's close: 461 passed, 0 failed, 3 ignored; 92 Vitest; 12 Playwright; 48 media cases (49 passed with the setup cases, 1 `test.fixme` skip).
2. `pnpm e2e:media` is not part of every gate (it needs a keyed build and a long run). It is run where a prompt says so, and before every merge (D-72).
3. Fix every regression you caused. Tick each acceptance box with its evidence. Leave a box that depends on a **Human** step unticked and say so.
4. Append an entry to `docs/v4/v4changelog.md`: `## Entry <n> - <date> - P<nn> (S<n>): <title>`, then what was added and changed, what differs from the plan or the guide and why, **REOPENED CONTRACT** lines, how it was checked (the counts), pinned versions, observations and readings (labelled `d1`), what is open, "Found, not fixed". The entry is in the same commit as the work (the changelog rule).
5. Commit on `v4-build` as `V4 P<nn> (S<n>): <title>` with the trailer `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`; `git push`; `gh pr checks 7 --watch` until both jobs finish. A red job is fixed in this prompt, with a follow-up commit.
6. Reply with: files created and modified, commands run with their results, the CI result, deviations, **Human** steps still open, issues found but not fixed.

---

# Who does what

The agent writes files, runs commands (including on D1, which is the machine it runs on), drives Chrome through Playwright, commits on `v4-build`, pushes to `origin v4-build` and reads CI with `gh`. A merge, a push to `main`, a tag, marking the pull request ready, a dashboard, a production database read, and a founder's decision, words or judgement are **Human** steps.

| When | Human step | Plan / guide |
| --- | --- | --- |
| Before Prompt 91 | **Decided 2026-10-22: yes.** The agent may push one throwaway branch, `scratch/v4-adapter`, to `origin` for the runner's adapter probe and must delete it after (locally and on `origin`). Prompt 91 reads lavapipe before S1. This is the only branch besides `v4-build` the agent may push | §1A item 7; G 0.5; "Gaps settled" item 1 |
| By Prompt 125 | **Decide V3 CL item 7** (V4 CL open item 5): move the Linux ffmpeg URL in Prompt 125, or leave it (it dies about 2026-11-27) | G 0.8, 17.1 |
| After Prompt 102 | **Read ten rows** of the line-break table the agent shows you, each as it would sit on a phone, and answer per row "I would break it this way: yes / no, because …". Prompt 103 records your answers before any algorithm exists | D-167; G 8.2 |
| Prompt 111 | **Design review 1** (about 30 minutes): sixteen screenshots, four questions per screen, the brand colour (keep the violet of D-179 or name another), and whether "Mono" keeps its name. Then look at the changed screens and say "go" | D-191; G Phase 11 |
| Prompt 128 | When the agent reports the pre-merge run green: run `gh pr ready 7` (or say "ready" and the agent runs it), **merge PR #7**, and watch the `main` run | D-115; G 19.4 |
| Prompt 128 | **Design review 2** on the deployed editor: take the sample clip from drop to export alone, the four questions on the eight states, and five goldens the agent picks at random ("does each look as intended?") | D-191, §23; G 19.5 |
| Prompt 128 | Run the SQL of G 19.6 in the Neon SQL editor and paste the rows (enum props only); push the tag `v4` (G 19.7) | G 19.6, 19.7 |
| Before every merge into `main` | `pnpm e2e:media` on D1 on a keyed build, all green (the agent runs it; D-72) | D-115 |

---

# Project Implementation Map

## Architecture Summary

V4 is "The look and the editor" (BP §6; TS §28 M1.4-M1.5), together with the seven edit-plan seams S-1 to S-7 of TS §40.9 (spec v1.2). It starts from the deployed V3 at `550ff8f`, in which a dropped clip is demuxed, cleaned by the voice chain, transcribed, scanned by the full detector (four kinds), drawn by a GPU compositor in one caption style with NumberReveal only, and exported. V4 changes how every short looks and how it feels to make one, and makes every later version additive:

- **The edit plan (S-2).** A new pure crate, `offcut-plan`, holds the catalog of the four kinds, the generated LLM prompt and schema (snapshot-tested, sent nowhere until V6.1), the rules producer `from_rules`, and `validate`, the only constructor of a `ValidPlan`, which resolves every number from the effective transcript (D-118 to D-124, D-165). The scene reads a `ValidPlan` and nothing else describes the edit.
- **Two timelines (S-1).** `OutMs` beside `TimeMs`, `TimeMap::identity`; the scene, the clock, the scrub bar and the frame index run on program time (D-125, D-126). Every map is the identity in V4.
- **Video as a command, fonts as a registry, colours as a theme, motion as data (S-4, S-5).** `DrawCmd::Video`, `FontKey`/`FontSet`, `Theme` roles, `CaptionMotion` (D-130 to D-139). A recorded identity proof shows the switch moved no pixel (D-116, D-117).
- **Program audio (S-6).** `offcut-dsp::program::assemble` in the audio worker; the preview plays `prog48`, equal to the cleaned voice (D-127, D-128).
- **Caption intelligence.** Phrase-aware line breaks by a least-cost search, a 700 ms reading floor taken from silence, three emphasis layers, draggable placement with the visuals following, preview equal to export by test, a shaping cache and a redraw while paused for a 150 ms edit-to-preview, hit testing (D-167 to D-176).
- **The look.** Clean as the premium default (Inter 800, shadow, a sliding highlight pill), Bold, Mono; all four visuals; the crop offset; the watermark; the summary; 16 golden frames on software adapters (D-134 to D-145, D-170).
- **The editor (S-7).** A dark design system (tokens, the Inter interface font as a hashed asset, primitives, icons), a layout of top bar, stage and four tabs, a transcript editor with click-to-fix and "Change all", direct manipulation on the preview, undo and redo, live style cards, keyboard shortcuts, processing and export progress, plain-word error cards, accessibility checked by axe, two design reviews by the founder (D-177 to D-193).
- **Robustness.** Progress throttle, cancel with a 2 s timeout, crash recovery, retry (D-151 to D-153); the analytics enum widened by five review actions (§13).

Every rule that affects output stays in Rust; TypeScript is glue (TS §2). The server changes only by the widened enum.

## Major Components

| Component | Location | V4 role |
| --- | --- | --- |
| Shared types | `crates/offcut-types` | `OutMs`, `OutSpan`, `timemap.rs`, `plan.rs` (wire format), `EditState` + two fields |
| Edit plan | `crates/offcut-plan` (new) | Catalog, prompt, config, resolve, validate, from_rules; five test files |
| Scene | `crates/offcut-scene` | Plan-driven, program time, `FontSet`, `Theme`, motion, readability, placement, hit, the four visuals, watermark, summary; twelve test files |
| Renderer | `crates/offcut-render` | `render(sources, list, fonts)`, the V4 support rule, headless rendering, golden frames |
| Program audio | `crates/offcut-dsp` | `program.rs`, `silence.rs`, `program_identity.rs` |
| Bindings | `crates/offcut-wasm-render` (`scene_api.rs` new), `crates/offcut-wasm-core` (`audio_api.rs`) | `rules_plan`, `set_scene(plan)`, `source_at`, `hit`, `resize`; `assemble` |
| Analytics enum | `crates/offcut-api-types/src/analytics.rs` | Five `ReviewActionKind` members |
| Workers and RPC | `web/src/workers/` | Four splits; protocol reopened; throttle, cancel, crash; preview seek, stats, resize; hit test; style thumbnails; assemble, release |
| State and use-cases | `web/src/state/`, `web/src/usecases/` | `history.ts`, `ui-store.ts`; `review-update.ts` and nine edit use-cases; `cancel-job.ts`; `preview-interact.ts`; `style-thumbnails.ts` |
| Interface | `web/src/ui/` | Tokens, font asset, primitives, icons, layout, state cards, transcript editor, panels, player chrome, error panel, export pill |
| Tests | `web/src/**/*.test.ts`, `web/tests-e2e/` | Five Vitest files; three new media suites and the completed `pipeline-preview.spec.ts`; `screens.spec.ts` for the reviews |
| Tooling and CI | `scripts/`, `.github/workflows/`, `package.json`, `web/playwright.config.ts` | Tree and purity rules for `offcut-plan`; copy codes; lavapipe and `INSTA_UPDATE=no`; the `screens` project |
| Records | `docs/v4/`, `docs/technicalspec.md`, `docs/buildplan.md`, `docs/product.md` | Changelog, experiments, copy-back |

## Dependency Graph

```text
P91 preflight, baseline, font, Entry 1 ─► P92 S1: offcut-plan crate, deps, tree, graph, frozen check
        │
        ▼
P93 S2 identity reference (V3 code writes it) ─► P94 S3 four splits (no behaviour change)
        │
        ▼
P95 types: OutMs, TimeMap, plan.rs, EditState ─► P96 plan: catalog, config, resolve, validate, from_rules
        ─► P97 plan: prompt, five test files, snapshots
        │
        ▼
P98 Rust contract switch (scene, renderer, binding) ─► P99 TS contract switch, V3 look in dev
        ─► P100 S6 identity proof [must match] + program audio
        │
        ▼
P101 S7 native rendering, 2 goldens, Mesa line, reference retired
        │
        ├─► P102 line-break table ─► [Human: ten rows] ─► P103 readability, break search, floor
        │        ─► P104 emphasis, placement, hit, cache, parity
        │                  │
        │                  ▼
        │        P105 fonts, theme, styles, pill ─► P106 visuals, framing, watermark, summary, snapshots
        │                  ─► P107 14 goldens + parity, look moving
        │
        └─► P108 tokens, font, primitives, icons ─► P109 layout, state cards, copy ─► P110 screens, self-review
                                                                                       │
                                                         P111 design review 1 [Human] ◄┘
                                                                  │
          P112 seek, prefetch, resize, stats, redraw ─► P113 hit test, gestures, drags
                 ─► P114 history, review-update, word edit, replace-all ─► P115 transcript editor
                 ─► P116 style cards, visuals, framing ─► P117 shortcuts, toasts, creator walk-through
                                                                  │
          P118 rpc throttle, cancel, crash, pool ─► P119 cancel everywhere, errors, export pill
                 ─► P120 analytics enum, break-it session
                                                                  │
          P121 Vitest: rpc, machines, pipeline, history ─► P122 helpers, preview-sync
                 ─► P123 review-edit (16, axe) ─► P124 cancel, pipeline-preview, dev-hook check, full media run
                                                                  │
          P125 CI ─► P126 measure ─► P127 copy-back, final gate, review 2 prepared ─► P128 merge, deploy, review 2, tag [Human]
```

## Implementation Phases

| Guide phase | S-step | Prompts |
| --- | --- | --- |
| 0 Environment and V3 baseline; 1 Contracts, dependencies, docs | S1 | 91, 92 |
| 2 The identity reference | S2 | 93 |
| 3 The four splits | S3 | 94 |
| 4 Types and the plan crate | S4 | 95, 96, 97 |
| 5 Scene and renderer on the new contract | S5 | 98, 99 |
| 6 The identity proof and program audio | S6 | 100 |
| 7 Native rendering | S7 | 101 |
| 8 Caption intelligence | S8 | 102, 103, 104 |
| 9 The look | S9 | 105, 106, 107 |
| 10 Design system and editor shell | S10 | 108, 109, 110 |
| 11 Design review 1 | S10A | 111 |
| 12 Preview and direct manipulation | S11 | 112, 113 |
| 13 Editing | S12 | 114, 115, 116, 117 |
| 14 Robustness and its interface | S13 | 118, 119, 120 |
| 15 Unit and machine tests | S14 | 121 |
| 16 End-to-end suites | S15 | 122, 123, 124 |
| 17 CI | S16 | 125 |
| 18 Measure | S17 | 126 |
| 19 Copy-back, design review 2, merge, deploy, tag | S18, S19 | 127, 128 |

## Prompt Dependency Graph and Table

Strictly sequential. Allowed in parallel if two agents run (§5 "What can run in parallel"): 108-110 (the design system needs no Rust) beside 102-107; commit order on the branch still follows the numbers, and Prompt 111 needs both tracks finished.

| Prompt | V4 # | Major deliverable | Depends on | Produces | Guide |
| --- | --- | --- | --- | --- | --- |
| 91 | 01 | Tools, V3 baseline counts, adapter probe, dependency dry run, interface font file, Entry 1, `experiments.md` | `v4-build` at `6d897fa` or later; Human (scratch push) | Baseline records; `~/inter-latin.woff2` | Phase 0, 1.1 |
| 92 | 02 | `offcut-plan` skeleton, workspace deps, TS §5/§7/§2, tree-script rules, axe dev dep, reopened-contract rows, frozen check, guards | 91 | `pnpm check` green, counts unchanged | 1.2-1.7, M-1 |
| 93 | 03 | `v3_reference.rs` and its six JSON files written by V3's scene | 92 | The identity reference, frozen | Phase 2, M-2 |
| 94 | 04 | `rpc-worker.ts`, `frame-decoder.ts`, `export-job.ts`, `preview-job.ts`, `scene_api.rs` | 93 | Every split file under 400 lines; 48 media cases unchanged | Phase 3, M-3 |
| 95 | 05 | `OutMs`, `OutSpan`, `timemap.rs`, `plan.rs`, `EditState` fields, generated types, `clip-store` defaults | 94 | The V4 shared types | 4.1-4.3 |
| 96 | 06 | `catalog.rs`, `config.rs`, `resolve.rs`, `validate.rs` (+ `validate_params.rs`), `from_rules.rs`, `lib.rs` | 95 | `validate`, the only door to `ValidPlan` | 4.4 rows 1-5, 7 |
| 97 | 07 | `prompt.rs`; `validate.rs`, `plan_from_rules.rs`, `validate_rules_plans.rs`, `catalog_snapshot.rs`, `ui.rs` tests; snapshots read | 96 | `offcut-plan` complete and green | 4.4 rows 6, 8-11, 4.5, M-4 |
| 98 | 08 | Scene on `ValidPlan`, `OutMs`, `FontSet`, `Theme`, `CaptionMotion`, zones, `Video`; renderer `render(sources, list, fonts)`; `scene_api.rs` | 97 | The Rust side of the V4 contract, V3's look | 5.1 |
| 99 | 09 | `protocol.ts`, `load-render.ts`, worker, jobs, loops, `run-pipeline.ts` rules plan, `MINTING`, `clip-store` | 98 | The page on the V4 contract; V3's preview in dev | 5.2, 5.3, M-5 |
| 100 | 10 | Identity proof; `program.rs`, `silence.rs`, `program_identity.rs`, `assemble`, `release`, `prog48` | 99 | "6 inputs, n frames, identical"; 48 media cases green | Phase 6, M-6 |
| 101 | 11 | Reference retired; `new_headless`, `render_to_image`, SSIM, two goldens; Mesa line in CI | 100 | Native rendering on WARP and lavapipe | Phase 7, M-7 |
| 102 | 12 | `line_breaks.rs` table (30 rows), committed before any algorithm; ten rows for the founder | 101 | The frozen table | 8.1, 8.2 (prepare) |
| 103 | 13 | The founder's answers recorded; `readability.rs`, the break search, the reading floor | 102, Human (ten rows) | Phrase-aware captions | 8.2 (record), 8.3 |
| 104 | 14 | `placement.rs`, `captions_draw.rs` (emphasis), `hit.rs`, shaping cache, `parity.rs`, `scene_api::hit` | 103 | Caption intelligence complete | 8.4, 8.5, M-8 |
| 105 | 15 | Inter 800 key 4, the theme table, `ActiveTreatment::Pill`, `ShadowSpec`, Clean/Bold/Mono, the pill and shadow drawing | 104 | The premium default | 9.1 |
| 106 | 16 | ListReveal, FromTo, KeywordPop, framing offset, watermark, summary; snapshot, safe-area, determinism, catalog tests | 105 | All four visuals | 9.2 |
| 107 | 17 | Fourteen goldens and the parity case; the look checked moving | 106 | 16 goldens, 17 passing on both adapters | 9.3, 9.4, M-9 |
| 108 | 18 | `tokens.css`, the font asset, `web/LICENSES.md`, the twelve primitives, `icons.tsx` | 92 | The design system | 10.1-10.4 |
| 109 | 19 | `EditorLayout`, `TopBar`, `Stage`, `SidePanel`, `ui-store`, the state cards, every §17.5 key | 108, 107 | Every state of §17.2 in the shell | 10.5-10.7 |
| 110 | 20 | `screens.spec.ts`, `__offcutDev` hooks, the `screens` project and script, self-review | 109 | Sixteen captures | 10.8, 10.9, M-10 |
| 111 | 21 | Design review 1 prepared, held, changes made, recaptured | 110, Human (review) | The founder's go | Phase 11, M-11 |
| 112 | 22 | Seek, prefetch, keyframe restart, resize rule, statistics, scrub bar, redraw while paused | 111 | A scrubbable preview | 12.1, 12.2 |
| 113 | 23 | `hitTest`, `preview-interact.ts`, minimal `review-update.ts`, caption and video drags, hover chrome, badge | 112 | Direct manipulation | 12.3, 12.4, M-12 |
| 114 | 24 | `history.ts`, full `review-update.ts`, eslint edges, `undo-redo.ts`, `redetectSentence` handler, `edit-word.ts`, `replace-all.ts` | 113 | The edit flow and undo | 13.1, 13.2 rows 1-3 |
| 115 | 25 | `CaptionEditor.tsx`, the replace-all toast, edit-to-preview timing | 114 | Click a word to fix it | 13.2 rows 4-5 |
| 116 | 26 | `styleThumbnails`, `StylePanel`/`StylePicker`, `PlanItemList`, `FramingPanel`, `CropOffsetControl`, `CutsSection` | 115 | The other three tabs | 13.3 |
| 117 | 27 | `use-shortcuts.ts`, `ShortcutsSheet`, `ToastHost`; the creator walk-through | 116 | The editor complete | 13.4, 13.5, M-13 |
| 118 | 28 | Throttle, `cancel`, `onCrash`, `isDead`, `pool.cancel`, `restartWorker` | 117 | A robust RPC layer | 14.1 |
| 119 | 29 | `cancel-job.ts` and every cancel path; the 14 error entries; `ErrorPanel`; `reloadPreview`; the export pill | 118 | Recovery in plain words | 14.2, 14.3 |
| 120 | 30 | `ReviewActionKind` + 5, generated types; the break-it session | 119 | Server accepts the new actions | 14.4, 14.5, M-14 |
| 121 | 31 | `rpc.test.ts`, the two machine tests, `run-pipeline.test.ts`, `history.test.ts` | 120 | About 160 Vitest | Phase 15, M-15 |
| 122 | 32 | E2E helpers; `preview-sync.spec.ts` (4) | 121 | Timing proven in Chrome | 16.1, 16.2 |
| 123 | 33 | `review-edit.spec.ts` (16, axe) | 122 | The editor proven in Chrome | 16.3 |
| 124 | 34 | `cancel.spec.ts` (5), `pipeline-preview.spec.ts` complete, config, no dev hook in `dist`, two full runs | 123 | 76 media cases green on D1 | 16.4-16.7, M-16 |
| 125 | 35 | `ci.yml` completed, guard drills in CI, optional ffmpeg URL | 124, Human (item 7 decision) | CI runs goldens on lavapipe | Phase 17, M-17 |
| 126 | 36 | Every §21.1 reading, the bench, BP §6.2's decisions | 125 | `experiments.md` complete | Phase 18, M-18 |
| 127 | 37 | Spec copy-back, final local gate, design review 2 prepared (QA list) | 126 | Specs say what V4 built | 19.1-19.3 |
| 128 | 38 | Merge (Human), deploy, verify, design review 2 (Human), SQL (Human), close-out, tag (Human) | 127 | V4 live and tagged | 19.4-19.7, M-19 |

---

# Prompts

Every prompt ends with the SAB gate. The "Acceptance Criteria" are what is specific to the prompt; the SAB gate is always also required.

## Prompt 91 (V4 01/38) — Preflight, V3 baseline, software adapters, dependency dry run, the interface font, Entry 1

**Effort: Medium.** Tool audits, test runs and records, plus one 30-line probe on a scratch branch and one font subset. Nothing it writes is product code.

### Objective

Prove D1 has every tool V4 needs, record the V3 baseline counts and sizes on `v4-build`, prove a software graphics adapter renders natively (D1 now; the runner now or in Prompt 101), dry-run the new dependencies, make the interface font file outside the repository, and commit the records as Entry 1 of the changelog with an empty `docs/v4/experiments.md`, so that every later prompt has a floor to compare with.

### Project State Before This Prompt

- `v4-build` at `6d897fa` or later, pushed, upstream `origin/v4-build`; it holds spec v1.2 (`ef1c42d`), the plan (`16b3a4c`, revised `1b9db25`), `origin/main` at `550ff8f` merged in (`81474bf`), the guide (`6d897fa`) and this file. Draft pull request #7 `v4-build` → `main` is open; its last run is green.
- The founder allowed the scratch push for the runner probe on 2026-10-22 ("Who does what"; V4 changelog Entry 0.6).
- `docs/v4/v4changelog.md` holds Entries 0 to 0.5. No V4 code exists.
- `bench/results/d1-2026-10-19-portrait.json` and `-v3testclip.json` exist (§1A item 9).

### Why This Comes Here

No V4 count may fall below V3's close, and every frozen-file check compares with `550ff8f`, so both are read before any change (§1A items 3-4). The golden frames (Prompt 101) and the whole native-rendering decision (D-134) depend on a software adapter existing on D1 and on the runner (§1A item 7); finding out now keeps a failure from surfacing six prompts later.

### Scope

#### Implement

- G 0.1: shell, branch, `git merge-base v4-build origin/main` = `550ff8f`, upstream, `gh pr view 7` (OPEN, draft, `main` ← `v4-build`). If `origin/main` moved past `550ff8f`, stop and report: the founder decides whether to merge it (G 0.1 then rewrites `BASE` everywhere).
- §1A item 1: `git log --oneline -3 origin/main` shows `550ff8f`, `9122eaf`, `7f091fb`; `git ls-remote --tags origin v3` points at `7f091fb`.
- G 0.2: the tool audit table, every row; install `cargo-insta` and `fonttools brotli` if missing (D1 only).
- G 0.3: the baseline run on a keyed build (export the key in this terminal only), the four counts, both wasm sizes, the app-shell size `check-hosts.mjs` reports.
- G 0.4: the 21 V3 markers (11 in `crates`, 10 in `web/src`) listed into the records; the four file lengths (372, 377, 399, 381); no untagged TODO.
- G 0.5, settled as "Gaps settled" item 1: on `scratch/v4-adapter` (local), `crates/offcut-render/examples/adapter_probe.rs` with the temporary dev-dependencies of G 0.5, run on D1 (expect WARP, `Cpu`, the circle's colour at the centre pixel). **Runner half, only if the founder said yes before this prompt:** add a throwaway workflow on the scratch branch triggered by `on: push: branches: [scratch/v4-adapter]` (a `workflow_dispatch` workflow does not run from a branch that is not the default one), `ubuntu-latest`, `sudo apt-get install -y mesa-vulkan-drivers`, `dpkg -s mesa-vulkan-drivers | grep Version`, the `cargo run`; push the scratch branch, read the log with `gh run view --log`, then delete the branch locally and on `origin`. If the founder said no: record "runner probe deferred to P101 (golden_frames is the probe)".
- G 0.6: the dependency dry run on `scratch/v4-deps` (local only): `insta` (json), `png`, `@axe-core/playwright`; `cargo deny check licenses bans`; the wasm32 normal-tree grep for native backends; revert and delete.
- G 0.7: the Latin subset of the scene's Inter TTF to `~/inter-latin.woff2` (under 120 kB), the Inter version from the name table.
- G 1.1: Entry 1 of `docs/v4/v4changelog.md` (every row of G 1.1's table, plus "Runner adapter probe: done / deferred to P101", "Founder's answer on the scratch push", "`gh` version"), and `docs/v4/experiments.md` with the twelve reading headings and the two decision headings of G 1.1, each with empty `Question:`, `Method:`, `Numbers:`, `Date:`, `Decision:` lines.
- G 0.8: write the human-input table into Entry 1 (ten line-break rows on day 5, design review 1 on day 8, design review 2 on day 13, the SQL after deploy, V3 CL item 7's decision before Prompt 125).

#### Do Not Implement Yet

- Any `Cargo.toml`, `deny.toml`, `package.json`, TS §5 or script change on `v4-build` (Prompt 92). The font file in the repository (Prompt 108).
- Any push of a branch other than `v4-build`, except `scratch/v4-adapter` when the founder allowed it.

### Existing Components To Reuse

- `scripts/check-hosts.mjs` (bundle sizes), the V3 baseline commands of V3's Entry 1, `web/tests-e2e/helpers/fake-api.ts` (the seed from which the test key is worked out, never printed).

### Files To Create

- `docs/v4/experiments.md`.
- Outside the repository: `~/v4-baseline.txt` (scratch records), `~/inter-latin.woff2`.

### Files To Modify

- `docs/v4/v4changelog.md` (Entry 1).

### Architecture Requirements

- The probe example and the throwaway workflow exist only on scratch branches and never reach `v4-build`. `git status --short` on `v4-build` after this prompt shows only the two docs.
- The font is a subset of the **same** Inter release as `crates/offcut-scene/assets/fonts/Inter-Variable.ttf` (D-180), weight axis kept, features `kern liga calt tnum ss01`.

### Data Types / Interfaces

None. Entry 1 is a two-column table (`Item`, `Value`) as G 1.1 shows.

### Implementation Requirements

#### 1. Baseline (G 0.3)
`pnpm install --frozen-lockfile`, `pnpm check`, `pnpm fixtures`, the awk count (expect 461/0/3), Vitest 92, `pnpm e2e` 12, `pnpm e2e:media` (expect 49 passed, 1 skipped; run detached, SAB rule 11), `pnpm build`, the two `offcut_*_bg-*.wasm` sizes, `check-hosts.mjs`'s size line. Unset the key afterwards. A count different from V3's close is recorded as it is and becomes the floor; do not fix V3 code here.

#### 2. Adapter probe (G 0.5)
The example asks `wgpu::Instance` for an adapter with `force_fallback_adapter: true` (backends `DX12 | VULKAN`), prints `adapter.get_info()`, creates a 64 x 64 `Rgba8Unorm` texture with `COPY_SRC | STORAGE_BINDING | RENDER_ATTACHMENT` as Vello needs, renders a `vello::Scene` holding one filled circle, copies to a buffer (rows padded to 256 bytes) and prints the centre pixel. `pollster` is the example's `block_on`. Apply G 0.5's result table: D1 fails and the runner works → D-134's fallback recorded; the runner fails → stop for the founder.

#### 3. Dependency dry run (G 0.6)
Record the versions `cargo add` chose. A licence `cargo deny` refuses: record which entry `deny.toml` would need (Prompt 92 adds it). A native backend crate (`ash`, `d3d12`, `windows`, `gpu-alloc-*`) in the wasm32 normal tree: stop and report.

#### 4. Font (G 0.7)
`pyftsubset` with the unicode list and features of G 0.7; `--flavor=woff2`. Record size and the Inter version string.

### Error Handling

- A missing tool: install it as G 0.2's last column says, or stop. Never continue to Prompt 92 with the adapter result unrecorded.
- The marker count is not 21: list them all into the records; each must be gone or re-tagged by Prompt 127.

### Concurrency / Async Requirements

None. Long runs detached (SAB rule 11).

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

- The test key exists in one terminal for the media run only and is unset before the gate.
- No URL with a credential, no runner log line with a token, in Entry 1.
- The scratch workflow has no secret and no permission beyond `contents: read`.

### Testing Requirements

#### Unit Tests
None new.

#### Integration Tests
The baseline: `pnpm check`, `cargo test --workspace`, Vitest, `pnpm e2e`, `pnpm e2e:media` (keyed).

#### Failure-Path Tests
None (Prompt 92 proves the guards).

### Validation Commands

```bash
git merge-base v4-build origin/main | cut -c1-7                  # 550ff8f
git branch -a | grep -c scratch/                                 # 0 (local and remote scratch branches deleted)
git status --short                                               # only docs/v4/v4changelog.md and docs/v4/experiments.md before the commit
grep -cE "^## " docs/v4/experiments.md                           # 14
grep -c "^Decision:" docs/v4/experiments.md                      # 14 (all empty)
ls -l ~/inter-latin.woff2                                        # under 120 kB
```

### Acceptance Criteria

- [ ] Every tool of G 0.2 prints its version; `cargo insta` and `fonttools` installed.
- [ ] Entry 1 holds the baseline SHA, the four counts exactly as printed, both wasm sizes, the shell size, the 21 markers, the four file lengths, the D1 adapter name and pixel, the runner adapter name, Mesa version and pixel **or** "deferred to P101" with the founder's answer, the `insta`, `png` and `@axe-core/playwright` versions, the font's Inter version and size, and the human-input table.
- [ ] `docs/v4/experiments.md` holds fourteen headings with empty fields.
- [ ] No scratch branch exists locally or on `origin`; `v4-build` carries no probe or workflow file.
- [ ] The SAB gate passes with the baseline counts; the commit is pushed and CI is green.

### Agent Instructions

Before: read §1A, §2 D-113 to D-117, §3, G Phase 0 and G 1.1, and the founder's answer on the scratch push. During: this prompt writes records and two scratch branches; nothing on `v4-build` but the two docs changes. After: SAB gate; commit `V4 P91 (S1): baseline records, changelog Entry 1, experiments.md`; push; read CI. Report the Human items of "Who does what" still open.

### Expected Result

`v4-build` has one new commit holding Entry 1 and `experiments.md`. `~/inter-latin.woff2` exists. Every number later prompts compare with is in Entry 1.

---

## Prompt 92 (V4 02/38) — The `offcut-plan` crate in the workspace, dependencies, the spec tree and graph, the frozen check, the reopened-contract rows

**Effort: Medium.** Configuration, one empty crate, spec text and script tables, all specified exactly; the guard drills are mechanical.

### Objective

Make the repository ready for every V4 file: the TS §5 tree lists them all, TS §7's graph and TS §2's row name `offcut-plan`, the tree script knows the crate's edges and purity, the workspace holds an empty `offcut-plan`, the dependencies are pinned, the native `wgpu` backends are test-only, and the changelog lists every contract V4 will reopen. Nothing changes behaviour.

### Project State Before This Prompt

- Prompt 91's Entry 1 exists with the counts 461/0/3, 92, 12, 48 (or the real floor) and the dependency versions.
- `scripts/check-file-tree.mjs` checks the roots `crates/ server/ web/ scripts/ verify/ corpus/ bench/ .github/` (not `docs/`, not `fixtures/`), ignores `*.snap` and `*.stderr`, has a hand-written `ALLOWED_CRATE_DEPS` table that counts `[dev-dependencies]` too, and a `PURE_CRATES` list.

### Why This Comes Here

`check-file-tree.mjs` fails on any unlisted file and on any crate edge its table does not allow, and every later prompt adds files and edges; listing them now keeps each later prompt about its own work (D-113, D-122). The frozen check (G 1.6) must exist before the first code change so that every later gate can run it.

### Scope

#### Implement

- G 1.2 rows 1-4: TS §5 gains every **NEW** path of §4 with a one-line purpose in the tree's style, including `crates/offcut-plan/src/validate_params.rs` (Appendix C.1's allowed split), `crates/offcut-plan/tests/ui/valid_plan_private.rs` and `.stderr`, the temporary `crates/offcut-scene/tests/v3_reference.rs` and the six files `crates/offcut-scene/tests/snapshots/v3_reference/{busy_1080,busy_540,reference_1080,a01_1080,w01_1080,w07_1080}.json` marked "temporary, V4 S2 to S7", `web/src/ui/assets/inter-latin.woff2`, `web/LICENSES.md`, `web/src/ui/primitives/*.tsx` (each of the twelve by name, plus `index.ts` if used), the four new CSS modules, `fixtures/golden/` (one line), `docs/v4/experiments.md`. Rename `EventToggleList.tsx` to `PlanItemList.tsx`; replace `ReviewPanel.tsx` with `ui/layout/SidePanel.tsx` (and the other three layout files). TS §7's graph gains `offcut-plan` with the edges of D-122 plus the dev-only edges below; TS §2 gains the `offcut-plan` row.
- `scripts/check-file-tree.mjs`: `ALLOWED_CRATE_DEPS` gains `"offcut-plan": ["offcut-text", "offcut-types", "offcut-detect"]` and adds `"offcut-plan"` and `"offcut-detect"` to `offcut-scene`'s list and `"offcut-plan"` to `offcut-wasm-render`'s list, each `offcut-detect` edge with the comment "dev-dependency only: the labeled-set tests (§3.2)"; `PURE_CRATES` gains `offcut-plan`. (The script counts dev edges, so they must be allowed: "Gaps settled" item 2.)
- G 1.3: `crates/offcut-plan/Cargo.toml` (`[lints] workspace = true`; `offcut-types`, `offcut-text`, `serde`, `thiserror`, optional `serde_json`; `[features] default = ["prompt"]`, `prompt = ["dep:serde_json"]`) and `src/lib.rs` holding only the crate doc comment; root `Cargo.toml` member and `[workspace.dependencies]` `offcut-plan`, `insta` (json), `png` at Prompt 91's versions; `offcut-render`'s `[dev-dependencies]` `wgpu = { workspace = true, features = ["vulkan", "dx12", "wgsl"] }` and `png`; `deny.toml` only what `cargo deny` names. If the workspace denies `unused_crate_dependencies`, each dependency is added by the prompt whose code uses it instead (V3's rule); record which.
- G 1.4: `web/package.json` dev dependency `@axe-core/playwright` at Prompt 91's version; `pnpm install`.
- G 1.5: the changelog's "Known issues" table gains one row per reopened contract of SAB rule 3, with the prompt that closes it.
- G 1.6: the frozen check exists in the SAB; run it once and paste its (empty) output into the entry.
- G 1.7: the three guard drills, then undo.
- Close V4 CL known issue 1 (`AudioWorkerApi`) as "recorded as REOPENED, D-127; closed by P100" and keep known issue 2 open for Prompt 127.

#### Do Not Implement Yet

- Any code in `offcut-plan` beyond the doc comment (Prompts 96, 97). Any dev-dependency of `offcut-scene` (Prompt 93) or `offcut-plan` (Prompt 97) if the workspace denies unused ones.
- `package.json`'s `screens` script and the `screens` Playwright project (Prompt 110).

### Existing Components To Reuse

- The style of the existing TS §5 lines and the `v3/` doc lines; V3's `PURE_CRATES` entries and the `D-28` comment style in `ALLOWED_CRATE_DEPS`.

### Files To Create

- `crates/offcut-plan/Cargo.toml`, `crates/offcut-plan/src/lib.rs`.

### Files To Modify

- `docs/technicalspec.md` (§2, §5, §7), `Cargo.toml`, `Cargo.lock`, `crates/offcut-render/Cargo.toml`, `deny.toml` (only if needed), `scripts/check-file-tree.mjs`, `web/package.json`, `pnpm-lock.yaml`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- `offcut-plan` is pure (D-122, §3.2): `cargo tree -p offcut-plan -e normal` resolves none of `web-sys js-sys wasm-bindgen getrandom rand wgpu`, and it builds for `wasm32-unknown-unknown` with `--no-default-features`.
- The native `wgpu` backends are dev-only (D-134, G item 4): the wasm32 normal tree of `offcut-render` has no `ash`, `d3d12`, `windows` or `gpu-alloc-*`.

### Data Types / Interfaces

None in code. The TS §2 row: `offcut-plan` may use `offcut-types`, `offcut-text`, `serde`, `thiserror`, and `serde_json` behind the feature `prompt`.

### Implementation Requirements

#### 1. The tree lines
One line per file, in the tree's alphabetical-by-role order, each purpose naming its decision (`timemap.rs [ONLY] recording ↔ program time; identity only in V4 (D-125)`). The `docs/v4/` block lists `v4implementation.md`, `v4buildguide.md`, `coding-promptsv4.md`, `v4changelog.md`, `experiments.md`.

#### 2. The checks of G 1.3
`cargo build --workspace`; `cargo deny check`; the two tree greps; `cargo build -p offcut-plan --target wasm32-unknown-unknown --no-default-features --release`.

#### 3. Guards (G 1.7)
`touch crates/offcut-plan/src/stray.rs` → the tree check names it; `web-sys` in `offcut-plan` → the purity check fails; `vulkan` in the normal `wgpu` → the wasm32 grep prints. Paste each failure line into the entry, then undo.

### Error Handling

- `check-file-tree.mjs` fails on a listed but missing file: it does not (G 1.2's note); if it starts to, record it and list the file in the prompt that creates it instead.
- A licence `cargo deny` refuses beyond Prompt 91's record: stop and report.

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

None beyond the SAB.

### Testing Requirements

#### Unit Tests
None new.

#### Integration Tests
The full gate; counts equal to Entry 1.

#### Failure-Path Tests
The three guard drills (G 1.7).

### Validation Commands

```bash
node scripts/check-file-tree.mjs                                  # exit 0; "13 crates within the graph of TS §7"
cargo tree -p offcut-render -e normal --target wasm32-unknown-unknown --prefix none | grep -iE '^(ash|d3d12|windows|gpu-alloc-)'   # no output
cargo tree -p offcut-plan -e normal --prefix none | sort -u | grep -E '^(web-sys|js-sys|wasm-bindgen|getrandom|rand|wgpu) '      # no output
cargo build -p offcut-plan --target wasm32-unknown-unknown --no-default-features --release                                     # Finished
grep -c "v4/" docs/technicalspec.md                               # 5 doc lines (plus the tree's source lines)
```

### Acceptance Criteria

- [ ] TS §5 lists every NEW path of §4 and the temporary reference files; TS §7 and TS §2 name `offcut-plan`; the tree check passes with 13 crates.
- [ ] `offcut-plan` is in the workspace, pure, and builds for wasm32 without its default feature.
- [ ] `insta`, `png`, `@axe-core/playwright` are pinned at the recorded versions; native `wgpu` backends are dev-only.
- [ ] The "Known issues" table lists every reopened contract of SAB rule 3 with its closing prompt.
- [ ] The three guards fired and were undone; the frozen check prints nothing; counts equal Entry 1; CI green.

### Agent Instructions

Before: read §2 D-113, D-122, §3, §4, §19.1-§19.2, G Phase 1 and `scripts/check-file-tree.mjs` whole. During: change only the files listed. After: SAB gate; commit `V4 P92 (S1): offcut-plan crate, dependencies, spec tree and graph, frozen check`; push; read CI.

### Expected Result

The repository accepts every V4 file and crate edge; nothing behaves differently; `pnpm check` and CI are green with V3's counts.

---

## Prompt 93 (V4 03/38) — The identity reference, written by the V3 scene (D-117)

**Effort: High.** The six inputs must be built exactly as V3 builds them, the serialization must be deterministic, and the files become the proof that Prompts 98-99 changed no pixel. A mistake here makes the proof meaningless.

### Objective

Write `crates/offcut-scene/tests/v3_reference.rs`, which builds six scenes through **V3's** `build_scene` and writes every 7th frame's display list to six JSON files, compare them on a second run, and commit both while every scene source is still V3's.

### Project State Before This Prompt

- Prompt 92 committed; TS §5 lists `v3_reference.rs` and the six JSON paths as temporary.
- `git diff --name-only 550ff8f -- crates/offcut-scene/src` prints nothing.
- `fixtures/speech/README.md` holds "The V2 reference clip as recognized" with word times; `fixtures/labeled/` holds the 31 files; `crates/offcut-detect/tests/precision_recall.rs` shows how V3 synthesizes word times (V3 §19.12).

### Why This Comes Here

No snapshot or golden exists from V2 or V3 (D-117), so a reference recorded by the old code is the only way to prove the contract switch is a pure refactor (D-116). It must be written before Prompt 95 touches `units.rs` (which renames `to_time_ms`), so it is the first code of V4.

### Scope

#### Implement

- G 2.1: the test with the six inputs of G 2.1's table (`busy_1080`, `busy_540`, `reference_1080`, `a01_1080`, `w01_1080`, `w07_1080`), each built with V3's `build_scene(SceneInput { transcript, events, edit: &EditState::default(), clip, profile })`; frames `0, 7, 14, …` to the last frame (every 7th: a prime step samples every chunk and event phase; G 2.1's "0, 7, 33" is a typo for "0, 7, 14", "Gaps settled" item 3); each frame serialized with `serde_json::to_string(&scene.frame_at(FrameIdx::new(n).to_time_ms()))`; one JSON array per input to `tests/snapshots/v3_reference/<name>.json`, written only when `OFFCUT_V3_REFERENCE_WRITE=1`, otherwise read and compared value for value.
- `offcut-scene`'s dev-dependencies `offcut-detect` and `serde_json` (§3.2).
- G 2.2: write, list, compare.
- G 2.3: stage exactly the test, the six files and `Cargo.toml`.

#### Do Not Implement Yet

- Any change under `crates/offcut-scene/src` (Prompt 98). The comparison form through the V4 contract (Prompt 98 writes it, Prompt 100 makes it pass).
- `insta` in `offcut-scene` (Prompt 98 or the first snapshot test).

### Existing Components To Reuse

- `lib.rs`'s `busy()` test transcript and its three NumberReveals (copied into the test: a test file cannot reach `#[cfg(test)]` helpers).
- `offcut_detect::detect` with zero prosody for the reference clip; with the labeled file's prosody marks for `a01`, `w01`, `w07`; the time synthesis of `precision_recall.rs`.
- V3's preview profile constructor (1080 and 540 widths).

### Files To Create

- `crates/offcut-scene/tests/v3_reference.rs`; the six `tests/snapshots/v3_reference/*.json`.

### Files To Modify

- `crates/offcut-scene/Cargo.toml` (dev-dependencies), `Cargo.lock`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- The writer touches nothing outside `tests/snapshots/v3_reference/` (G 2.1).
- The JSON is `serde_json`'s compact output of V3's `DisplayList` (V3's `Serialize` derive): the field names and the `FontId` variant names are what Prompt 100's normalisation maps (D-117).
- `CARGO_MANIFEST_DIR` locates the fixtures; no absolute path.

### Data Types / Interfaces

```rust
// tests/v3_reference.rs (temporary, deleted in Prompt 101)
struct Input { name: &'static str, transcript: Transcript, events: Vec<DetectedEvent>, clip: ClipInfo, width: u32 }
fn inputs() -> Vec<Input>;                       // the six of G 2.1
fn frames(scene: &Scene) -> Vec<serde_json::Value>;   // every 7th frame
#[test] fn v3_reference();                        // write when OFFCUT_V3_REFERENCE_WRITE=1, else compare
```

### Implementation Requirements

#### 1. The reference clip's input
The words and times are copied from `fixtures/speech/README.md` into a `const` table in the test (the README is documentation; the test must not parse Markdown). Clip 1920 x 1080, 74,705 ms.

#### 2. Labeled inputs
Synthesized times and prosody exactly as V3's `precision_recall.rs` makes them; clip 1080 x 1920 with the synthesized length.

#### 3. Comparison
On read, a frame count mismatch or any value mismatch fails with the input name, the frame index and the first differing JSON path.

### Error Handling

Test code: `expect` with a message is allowed. A missing file without the write variable fails with "run with OFFCUT_V3_REFERENCE_WRITE=1 once, at P93 only".

### Concurrency / Async Requirements

None.

### Persistence Requirements

The six JSON files are committed and must not change until Prompt 101 deletes them (G 2.3).

### API Requirements

None.

### Security Requirements

The founder's clip text is the committed "as recognized" transcript of the reference clip (D-99 allowed it in V3); nothing from `v3testclip.mp4` is used.

### Testing Requirements

#### Unit Tests
None.

#### Integration Tests
`v3_reference` writes, then compares equal.

#### Failure-Path Tests
Edit one number in one JSON file by hand: the compare run fails naming the input and frame; restore with `git checkout`.

### Validation Commands

```bash
OFFCUT_V3_REFERENCE_WRITE=1 cargo test -p offcut-scene --test v3_reference   # 1 passed; six files written
ls -l crates/offcut-scene/tests/snapshots/v3_reference/                      # six non-empty .json files
cargo test -p offcut-scene --test v3_reference                                # 1 passed (compared)
git diff --name-only 550ff8f -- crates/offcut-scene/src                       # no output
```

### Acceptance Criteria

- [ ] Six JSON files exist, written by V3's scene code, every 7th frame of each input.
- [ ] A second run compares equal; the hand-edit drill fails and is restored.
- [ ] No scene source changed; the frozen check prints nothing; the Rust count rose by exactly 1.
- [ ] The changelog entry records "identity reference frozen at <this commit's SHA>" (written after the commit, in Prompt 94's entry if the SHA is not known yet) and the frame count per input.

### Agent Instructions

Before: read D-116, D-117, §8.1, G Phase 2 and V3's `offcut-scene/src/lib.rs` tests. During: do not open any scene source for editing. After: SAB gate; commit `V4 P93 (S2): the identity reference, written by the V3 scene`; push; read CI.

### Expected Result

The identity reference is in git, written by V3, and compares equal. It is the yardstick of Prompt 100.

---

## Prompt 94 (V4 04/38) — The four splits, no behaviour change (D-150)

**Effort: High.** Pure moves across nine files in two languages, but the render worker's state must reach two job files without an import cycle, and the 48 media cases must pass unchanged.

### Objective

Make room in the four full files: split `rpc.ts`, `video-source.ts`, `render.worker.ts` and `session.rs` by function into nine files, each under 400 lines, with no change of behaviour, so that V4's additions fit.

### Project State Before This Prompt

- Prompt 93 committed; the four files are 377, 399, 372 and 381 lines (Entry 1).
- No test under `web/tests-e2e` or `crates/*/tests` has changed since `550ff8f` except `v3_reference.rs`.

### Why This Comes Here

V3 §23 item 24 and TS §29's 400-line rule: V4 adds the throttle and cancel to RPC, prefetch and seek to the video source, four handlers to the render worker and the plan to the session. Splitting first, with the V3 tests as the proof, keeps "moved" and "changed" in separate commits (§5 S3).

### Scope

#### Implement

- G 3.1: `web/src/workers/rpc-worker.ts` takes `serveWorker`, `WorkerFailure`, `CANCELLED`, `toAppFailure`, `JobContext`, `Handlers` and their private helpers; `rpc.ts` keeps `createClient`, `Client`, `WorkerCallError`, `Cancelled` and the shared message types and re-exports nothing from `rpc-worker.ts`; every `*.worker.ts` and `render/*.ts` imports the worker side from `rpc-worker`.
- G 3.2: `web/src/workers/render/frame-decoder.ts`, a class `FrameDecoder` owning the `VideoDecoder` (create, configure with the colour-space memory of D-70, feed, drain, restart at a keyframe index, close, the output callback, the generation counter); `video-source.ts` keeps the queue, `frameAt`, `frameAtBlocking`, the shown frame, `liveFrames`, the `prefetch` stub and `close`, and delegates decoding. Its public surface is unchanged.
- G 3.3: `web/src/workers/render/export-job.ts` (`writeExport`, `afterExport`, `exportClip`, `ExportRun`) and `web/src/workers/render/preview-job.ts` (`previewPlay`, `previewClock`, `previewPause`, `previewTime`, the stop flag), each receiving the worker's state through one argument (`state: () => Session`, `scene: () => SceneState`), never importing the worker file; `render.worker.ts` keeps `open`, `scene`, `previewCanvas`, `openSession`, `detect`, `setScene`, `attachPreview`, `closeSession`, `closeFrames`, `staged` and the handler table.
- G 3.4: `crates/offcut-wasm-render/src/scene_api.rs`, a second `#[wasm_bindgen] impl RenderSession` block holding `set_scene` and `summary` and the `EditInput` and `SceneInputValue` structs; `mod scene_api;` in `lib.rs` (its only edit); `session.rs`'s fields become `pub(crate)`.
- G 3.5: lengths, all suites, the 48 media cases.

#### Do Not Implement Yet

- The throttle, `cancel`, `onCrash` (Prompt 118); `prefetch`'s body, seek restart (Prompt 112); `rules_plan`, `source_at`, `hit`, `resize` (Prompts 98, 104, 112); any new handler.

### Existing Components To Reuse

Everything moves as it is: same names, same bodies, same comments (with "moved from …" only where a comment named its old file).

### Files To Create

- `web/src/workers/rpc-worker.ts`, `web/src/workers/render/frame-decoder.ts`, `web/src/workers/render/export-job.ts`, `web/src/workers/render/preview-job.ts`, `crates/offcut-wasm-render/src/scene_api.rs`.

### Files To Modify

- `web/src/workers/rpc.ts`, `web/src/workers/render/video-source.ts`, `web/src/workers/render.worker.ts`, `web/src/workers/{media,asr,audio}.worker.ts` (import lines only; `media.worker.ts` and `asr.worker.ts` are frozen except for this import line, recorded as such), the `render/*.ts` files that import the worker side, `crates/offcut-wasm-render/src/session.rs`, `crates/offcut-wasm-render/src/lib.rs` (one line).

### Architecture Requirements

- A page-side import of `workers/rpc.ts` pulls no worker code (check with `pnpm build` and the boundaries lint).
- No import cycle: the job files import types only from the worker's state module, never `render.worker.ts`.
- `wasm-bindgen` accepts two `impl` blocks of one exported struct; the generated JavaScript is the same class (G 3.4): `git diff` of the generated `.d.ts` after `pnpm build:wasm` shows no API change.

### Data Types / Interfaces

No exported signature changes. `type Session`, `type SceneState` are the worker's existing internal types, exported from where they are declared for the job files.

### Implementation Requirements

#### 1. One split per step
Run `pnpm check` after each of G 3.1-3.4; do not start the next with a red check.

#### 2. The frozen-file exception
The import-line edits in `media.worker.ts` and `asr.worker.ts` are the only V4 changes to those files; the frozen check will list them. Record in the entry: "frozen check: media.worker.ts, asr.worker.ts import line only (rpc-worker split, D-150)". From this prompt on, the SAB gate's frozen check accepts exactly these two one-line diffs and `lib.rs`'s `mod scene_api;`.

### Error Handling

Unchanged behaviour by definition. A test that fails after a move is a moving mistake, fixed here.

### Concurrency / Async Requirements

The preview loop's stop flag and the export's run state keep their single owner (now the job file); the worker's handler table calls into them as before.

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

None beyond the SAB.

### Testing Requirements

#### Unit Tests
The existing ones, unchanged.

#### Integration Tests
`pnpm e2e` 12; `pnpm e2e:media` 49 passed, 1 skipped (D1, keyed).

#### Failure-Path Tests
None new.

### Validation Commands

```bash
for f in web/src/workers/rpc.ts web/src/workers/rpc-worker.ts web/src/workers/render/video-source.ts web/src/workers/render/frame-decoder.ts \
         web/src/workers/render.worker.ts web/src/workers/render/export-job.ts web/src/workers/render/preview-job.ts \
         crates/offcut-wasm-render/src/session.rs crates/offcut-wasm-render/src/scene_api.rs; do printf '%5s %s\n' "$(wc -l < "$f")" "$f"; done
# every number under 400; the four old files under 300 where possible
git diff --stat 550ff8f -- web/tests-e2e crates/*/tests ':!crates/offcut-scene/tests/v3_reference.rs' ':!crates/offcut-scene/tests/snapshots'   # no output
pnpm e2e:media 2>&1 | tail -3                                     # 49 passed, 1 skipped (keyed build; unset the key after)
```

### Acceptance Criteria

- [ ] Nine files exist, each under 400 lines; no exported API changed (generated `.d.ts` unchanged).
- [ ] No test changed; Rust, Vitest, Playwright counts equal Prompt 93's; the 48 media cases pass on D1.
- [ ] The frozen check lists only the three recorded one-line exceptions.
- [ ] The entry records each file's line count before and after.

### Agent Instructions

Before: read D-150, §14.2, §15.1, §15.3, §11.1, G Phase 3, and the four files whole. During: move, never rewrite; one split at a time. After: SAB gate plus the media run; commit `V4 P94 (S3): split rpc, video-source, render.worker and session (no behaviour change)`; push; read CI.

### Expected Result

The four full files have room; behaviour, tests and media cases are V3's.

---

## Prompt 95 (V4 05/38) — Program time, the time map, the plan's wire format, `EditState`'s two fields (S4 part 1)

**Effort: High.** Two reopened V1 contracts, a validating deserializer, and serde tagging that becomes a contract with V6.1's server; the generated TypeScript must come out clean.

### Objective

Add `OutMs`, `OutSpan` and `FrameIdx::to_out_ms`; the time map with its identity constructor and checked deserialization; the complete plan wire format at `PLAN_FORMAT = 1`; `CaptionPlacement` and `EditState`'s `caption_position` and `highlight_numbers` with serde defaults; and regenerate the TypeScript, fixing the page's defaults.

### Project State Before This Prompt

- Prompt 94 committed; the identity reference compares equal.
- `offcut-types` has V3's `units.rs` (with `to_time_ms`), `edit.rs`, `events.rs`.

### Why This Comes Here

`offcut-plan` (Prompts 96-97) and the scene (Prompt 98) are written against these types. Fixing the wire format once, with every arm of TS §40.2, means the generated TypeScript type does not change shape in every V6.x release (D-119).

### Scope

#### Implement

- G 4.1: `units.rs` §6.1 (`OutMs`, `OutSpan` through the same macro as `TimeMs`; `to_time_ms` renamed `to_out_ms` returning `OutMs`; its callers fixed, including the identity reference test, which keeps compiling through the rename only: it still calls V3's `build_scene`); `timemap.rs` §6.2 whole (`Segment`, `SrcPos`, `TimeMap` with a private field and a validating `Deserialize`, `SilenceProfile`) with its inline tests; `lib.rs` re-exports.
- G 4.2: `plan.rs` §6.3 whole (every type, the serde rules: `deny_unknown_fields` everywhere, `tag = "kind"` with snake_case, `ItemParams` adjacently tagged `tag = "kind", content = "params"`, `Label` transparent, `TextExtras` default; `Label::as_str`, `EditPlan::empty_rules(catalog)`), with its inline tests; `edit.rs` §6.4 (`CaptionPlacement` checked like `CropOffset`, `bottom()`, the two fields with defaults) with its tests; `events.rs` the doc comment of D-118 only.
- G 4.3: `pnpm gen:types`, `check-gen-clean.sh`; `clip-store.ts`'s `empty().edit` gains `caption_position: 1` and `highlight_numbers: true`; every other place that builds an `EditState` (`git grep -n "crop_offset: 0" -- web/src web/tests-e2e`).
- A test that `OutMs` and `TimeMs` compare only through numbers (§6.1).

#### Do Not Implement Yet

- Anything in `offcut-plan` (Prompt 96). Any caller of `TimeMap` outside tests (Prompt 98). `PLAN_FORMAT` and `CATALOG_VERSION` as generated constants: `PLAN_FORMAT` now; `CATALOG_VERSION` when Prompt 96 defines it.

### Existing Components To Reuse

- The unit macro of `units.rs`, `CropOffset`'s checked newtype pattern for `CaptionPlacement`, `EventParams` (V3, frozen) inside `ItemParams::Detected`, `WordRange`, `WordIdx`, `EventId`, `Confidence`, the `ts-rs` export used by `pnpm gen`.

### Files To Create

- `crates/offcut-types/src/timemap.rs`, `crates/offcut-types/src/plan.rs`.

### Files To Modify

- `crates/offcut-types/src/{units,edit,events,lib}.rs`; the callers of `to_time_ms` (`cargo check --workspace` names them); `web/src/gen/domain.ts` (generated); `web/src/state/clip-store.ts`; any `EditState` literal in `web/`; `docs/v4/v4changelog.md`.

### Architecture Requirements

- `TimeMap`'s only public constructor in V4 is `identity` (D-125; Appendix A: no non-identity map outside `from_segments`, V6.2). Its deserializer accepts only a non-empty list of increasing non-empty `Source` spans and non-zero `Hold`s.
- Integer milliseconds; no floats in time arithmetic (§6.2).
- **REOPENED V1 CONTRACT** `units.rs` (D-125) and `edit.rs` (D-171): one changelog line each; the "Known issues" rows closed.

### Data Types / Interfaces

As §6.1-§6.4 exactly; repeated here only where a detail is easy to miss:

```rust
#[serde(tag = "kind", content = "params", rename_all = "snake_case")]
pub enum ItemParams { NumberReveal { value: QuantityRef, label: Option<Label> }, ListReveal { header: Option<Label>, items: Vec<ListPoint> },
                      FromTo { from: QuantityRef, to: QuantityRef, label: Option<Label> }, KeywordPop { word: WordIdx }, Detected(EventParams) }
pub struct QuantityRef { pub words: WordRange }   // JSON: { "words": { "start": 3, "end": 5 } }
#[serde(default = "CaptionPlacement::bottom")] pub caption_position: CaptionPlacement;   // 1.0
#[serde(default = "yes")] pub highlight_numbers: bool;                                   // true
```

### Implementation Requirements

#### 1. `TimeMap` tests (§6.2)
Identity round trips at 0, 1, 33, 59,999, 60,000 ms; `to_out(duration) == Some(out_duration)`; `out_span` of spans of a 60 s identity map equals its numbers; one deserialization refusal per rule (empty list, empty span, decreasing spans, zero hold); `joins` of the identity is empty; `to_src` of the identity is `At(t)`.

#### 2. `plan.rs` tests (§6.3)
A round trip with every arm of every enum; `deny_unknown_fields` refusal at the plan, an item, a param block, `QuantityRef`, `TextExtras`, `MusicCue`; the **exact JSON** of a one-item rules plan pinned as a string (a contract, as V3 pinned `EventParams`); `Detected(EventParams)` round trips without a tag clash.

#### 3. `edit.rs` tests (§6.4)
V3's JSON `{"word_edits":{},"event_overrides":{},"style":"clean","crop_offset":0}` reads with `caption_position` 1.0 and `highlight_numbers` true; `CaptionPlacement::new` refuses NaN, ±infinity, -0.01, 1.01; deserialization refuses the same.

### Error Handling

`OutSpan::new` and `CaptionPlacement::new` return `Option`/`Result`; no panic. A refused deserialization gives a serde error naming the rule.

### Concurrency / Async Requirements

None.

### Persistence Requirements

V3's `edits` IndexedDB records read unchanged thanks to the serde defaults (TS §10.6). `DB_VERSION` stays 1.

### API Requirements

None (no route; the plan's JSON is a future server contract, pinned by the test).

### Security Requirements

None beyond the SAB.

### Testing Requirements

#### Unit Tests
Inline in `units.rs`, `timemap.rs`, `plan.rs`, `edit.rs` as above.

#### Integration Tests
`check-gen-clean.sh`; `pnpm check` (tsc on the new defaults).

#### Failure-Path Tests
The refusal cases above.

### Validation Commands

```bash
cargo test -p offcut-types                                        # all pass; V3's tests unchanged except the rename
pnpm gen:types && sh scripts/check-gen-clean.sh                   # exit 0 after the generated file is committed
git diff --stat web/src/gen                                       # domain.ts grows by OutMs … DropReason (§12)
git grep -n "to_time_ms" -- crates web/src                        # no output
cargo test -p offcut-scene --test v3_reference                    # 1 passed (still V3's scene)
```

### Acceptance Criteria

- [ ] `OutMs`, `OutSpan`, `to_out_ms`, `TimeMap` (identity only), `SilenceProfile`, the whole plan wire format and `CaptionPlacement` exist as §6 specifies, with every listed inline test.
- [ ] V3's stored `EditState` JSON reads with the two defaults; the exact one-item plan JSON is pinned.
- [ ] Generated TypeScript clean; `clip-store.ts` and every `EditState` literal carry the two fields; `pnpm check` green.
- [ ] The identity reference still compares equal; REOPENED CONTRACT lines for `units.rs` and `edit.rs`; their known-issue rows closed.

### Agent Instructions

Before: read D-118 to D-120, D-125, D-169, D-171, §6, §12, G 4.1-4.3, TS §40.1-§40.2. During: write the types and their tests together. After: SAB gate; commit `V4 P95 (S4): OutMs, TimeMap, the plan wire format, EditState fields`; push; read CI.

### Expected Result

The shared types of V4 exist, are tested, and are generated to TypeScript; nothing calls them yet.

---

## Prompt 96 (V4 06/38) — `offcut-plan`: catalog, config, resolve, validate, from_rules (S4 part 2)

**Effort: High.** `validate` is the only door to `ValidPlan` and the only place numbers are parsed for the screen (INV-21, INV-22); six ordered steps, twelve drop reasons, no panic and no indexing.

### Objective

Write the edit plan's logic: the catalog of the four kinds, the limits, number resolution over the effective transcript, the validator with every V-rule and drop reason, and the rules producer; tested inline as each file lands.

### Project State Before This Prompt

- Prompt 95 committed: `plan.rs`, `timemap.rs`, `EditState` fields exist.
- `crates/offcut-plan/src/lib.rs` holds only its doc comment.

### Why This Comes Here

The scene (Prompt 98) reads a `ValidPlan`, which only `validate` builds; the rules plan (`from_rules`) is what the page sends from Prompt 99 on.

### Scope

#### Implement

- G 4.4 rows 1-5 and 7, in order, `cargo test -p offcut-plan` after each:
  1. `catalog.rs` §7.2 [ONLY]: `CATALOG_VERSION = 1`, `Producers`, `AnchorRule`, `ParamTy`, `ParamSpec`, `ItemSpec` (with `zone: &'static str`), the four rows of §7.2's table exactly, `catalog()`, `spec(kind)`; inline test: rows in `EventKind` code order and unique.
  2. `config.rs` §7.4 [ONLY]: `PlanConfig`, `KindLimits`, `default()` from the catalog; the cut fields at 150 ms, 400 ‰, 6.
  3. `resolve.rs` §7.5: `effective_tokens`, `quantity_at` (must consume every token of the range except trailing punctuation), `inherit` (D-85, display re-formatted with `format_quantity`), `label_digits_ok`; its four inline tests.
  4. `validate.rs` §7.6 [ONLY constructor]: `ValidPlan` (private fields; `source`, `items`, `is_empty`), `ValidItem` (with `marks`), `Dropped`, `DropReason` (twelve members, snake_case), `PlanRejected`, `RejectReason`; the algorithm's six steps in order; `marks` for FromTo (§8.7: for a `Detected` FromTo, the start of the second quantity found by parsing from each effective token inside the anchors). If it nears 400 lines, the per-kind parameter checks go to `validate_params.rs` (already in the tree).
  5. `from_rules.rs` §7.7.
  7. `lib.rs` §7.1 module list and re-exports (`prompt` behind its feature arrives in Prompt 97).
- `PLAN_FORMAT` and `CATALOG_VERSION` as generated constants beside `LIMITS` (§12); `pnpm gen:types`.

#### Do Not Implement Yet

- `prompt.rs` and the five integration test files (Prompt 97). Any caller outside the crate (Prompt 98).

### Existing Components To Reuse

- `offcut_text::{tokenize, parse_quantity, format_quantity}`; `offcut-types`' plan, transcript, events, units and ids. The detector's unit inheritance (D-85) is the model for `inherit`, not a dependency.

### Files To Create

- `crates/offcut-plan/src/{catalog,config,resolve,validate,from_rules}.rs`, and `validate_params.rs` only if needed.

### Files To Modify

- `crates/offcut-plan/src/lib.rs`, `crates/offcut-plan/Cargo.toml` (only if deps were deferred in Prompt 92), `web/src/gen/domain.ts` (generated constants), `docs/v4/v4changelog.md`.

### Architecture Requirements

- Pure, `BTreeMap` only, no `unwrap`/`expect`/`panic!`/indexing in non-test code (§7 crate rules). An arithmetic overflow on a time is `AnchorsOutOfRange`.
- `ValidPlan` has no public constructor and no public fields (INV-21; trybuild in Prompt 97).
- In V4, every cut, chapter, `extras.hook`, `extras.post_caption`, hashtag and `music` is dropped with `kind: None, reason: NotInThisBuild` (D-121, D-164).
- The zone names `"event"` and `"caption"` match `offcut-scene`'s `ZoneId` names (checked by `catalog_complete.rs`, Prompt 106).

### Data Types / Interfaces

As §7.1-§7.7 exactly:

```rust
pub fn validate(plan: EditPlan, t: &Transcript, edits: &BTreeMap<WordIdx, String>, cfg: &PlanConfig)
    -> Result<(ValidPlan, Vec<Dropped>), PlanRejected>;
pub fn from_rules(events: &[DetectedEvent], t: &Transcript) -> EditPlan;   // t unused in V4 (D-163)
```

### Implementation Requirements

#### 1. Validation order (§7.6, Appendix C.1)
V-1 (`Err(Format)`, `Err(Catalog)`); non-item drops; tokens once; per item V-2 (kind known; `Detected` only from `Rules`, word-pointing arms only from `Model`; arm matches kind; required params) → V-3 (anchors non-empty, inside the transcript, anchor rule's word count, `at`/`word` inside the anchors, a `Detected` arm's own times inside the transcript) → V-4 (labels by characters; digits) → V-5 (`QuantityRef` resolves; FromTo through `inherit`; `EventParams` built as §7.6 says, ListReveal items with ordinals and word starts) → span → marks; then V-6 per kind in anchor order (`OverLimit`, `TooClose`, `ZoneTaken` with the holds 1,200 ms NumberReveal, 1,500 ms other event-zone kinds; the caption zone is never taken); then sort by `(span.start, kind code, anchors.start)`.

#### 2. `Detected` arms
Their texts are not checked by V-4; their anchors are checked by V-3; the `span` is the event's own; their `EventParams` pass through. A `Detected` FromTo without a second quantity inside its anchors is dropped with `NumberUnresolved`.

### Error Handling

Only V-1 is an `Err`; every other failure is a `Dropped`. A rule never mutates the input; it builds the output.

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

None beyond the SAB.

### Testing Requirements

#### Unit Tests
`catalog.rs` order test; `resolve.rs`'s four (§7.5); `validate.rs` inline smoke tests of each step (the full table is Prompt 97's `tests/validate.rs`); `from_rules.rs`: one event → one item with `Detected` params and `enabled` not copied.

#### Integration Tests
None yet.

#### Failure-Path Tests
Inline: a plan with `format: 2` → `Err(Format)`; a `Model` plan with a `Detected` item → `WrongProducer`.

### Validation Commands

```bash
cargo test -p offcut-plan                                         # all pass
cargo clippy -p offcut-plan --all-targets -- -D warnings         # clean
cargo build -p offcut-plan --target wasm32-unknown-unknown --no-default-features --release   # Finished
wc -l crates/offcut-plan/src/*.rs                                 # each under 400 above its tests
sh scripts/check-gen-clean.sh                                     # exit 0
```

### Acceptance Criteria

- [ ] The five files and `lib.rs` exist with §7's signatures; the catalog rows equal §7.2's table.
- [ ] `validate` follows the six steps in order and returns each of the twelve drop reasons where §7.6 says; no `unwrap`, `expect`, `panic!` or indexing in non-test code.
- [ ] `marks` is computed for every FromTo; non-item fields drop with `NotInThisBuild`.
- [ ] `PLAN_FORMAT` and `CATALOG_VERSION` are generated constants; generated types clean.

### Agent Instructions

Before: read D-118 to D-124, D-165, §7.1-§7.7, §8.7's "time of FromTo's to value", Appendix C.1, TS §40.2-§40.4. During: one file at a time with its tests. After: SAB gate; commit `V4 P96 (S4): offcut-plan catalog, config, resolve, validate, from_rules`; push; read CI.

### Expected Result

`offcut-plan` validates any plan into a `ValidPlan` and turns detector events into a rules plan; nothing outside the crate calls it yet.

---

## Prompt 97 (V4 07/38) — `offcut-plan`: the prompt, the five test files, the snapshots read (S4 part 3)

**Effort: High.** Twenty-one rule cases with the smallest plan each, the labeled-set sweep, a trybuild guard, and a system prompt a model will read from V6.1 on, which must be read by a person before it is accepted.

### Objective

Write `prompt.rs` (system prompt, output schema, user message, snapshot-tested and sent nowhere) and the crate's five test files, prove the guards, and finish S4 with every snapshot read.

### Project State Before This Prompt

- Prompt 96 committed; `validate`, `from_rules`, the catalog exist with inline tests.

### Why This Comes Here

The prompt must be generated from the catalog before the first request exists (D-123), and the validator's table of cases is the contract V6.1's model plans will meet. Both close S4 (G M-4).

### Scope

#### Implement

- G 4.4 row 6: `prompt.rs` §7.3 behind `#[cfg(feature = "prompt")]`: `PromptInput`, `system_prompt()` (fixed preamble, one paragraph per catalog row with name, purpose, anchor rule and param limits, and the three rules "Never write a number: point at the words that say it", "Every item points at word indexes of the transcript", "Return JSON matching the schema; no prose"; no example number of its own), `output_schema()` (JSON Schema restricted to the catalog; `ItemParams` lists only the word-pointing arms, never `detected`; no `source`), `user_message()` (`DURATION {ms}` first line, one tab-separated line per word with `energy_z` and `pitch_z` to one decimal, `PAUSE {ms}` before a gap of at least `PAUSE_MARK_MS = 300`, a blank line at each sentence end; ASR text, not edits).
- G 4.4 rows 8-11: `tests/validate.rs` (21 rule cases, the whole model plan of the four word-pointing kinds on `a01`, the edited number of INV-22, the non-item drops); `tests/plan_from_rules.rs` (the reference clip's five events → five items with the same ids, kinds, anchors, params; validated with the detector's spans; an empty event list gives a valid empty plan); `tests/validate_rules_plans.rs` (31 labeled files: detect, `from_rules`, `validate` → nothing dropped; prints "31 files, 0 items dropped"); `tests/catalog_snapshot.rs` (three `insta` snapshots named with the version, `system_prompt@v1`, `output_schema@v1`, `user_message@v1` on `a01` with zero prosody); `tests/ui.rs` + `tests/ui/valid_plan_private.rs` (+ committed `.stderr`).
- Dev-dependencies of `offcut-plan`: `insta` (json), `offcut-detect`, `trybuild`, `serde_json`.
- G 4.5: the three guard drills, then undo.
- **Read the three snapshots before accepting them** (G 4.4 "Read the prompt"), and paste the system prompt's first paragraph and one catalog paragraph into the entry.

#### Do Not Implement Yet

- Any caller of `prompt.rs` outside tests (V6.1). Any route, request or provider (D-164).

### Existing Components To Reuse

- `fixtures/labeled/` and the time synthesis of V3's `precision_recall.rs` (copy the helper into a `tests/common/` module only if the tree lists it; otherwise inline in each file); `fixtures/speech/README.md`'s reference events; the workspace `trybuild` pin.

### Files To Create

- `crates/offcut-plan/src/prompt.rs`; `crates/offcut-plan/tests/{validate,plan_from_rules,validate_rules_plans,catalog_snapshot,ui}.rs`; `crates/offcut-plan/tests/ui/valid_plan_private.rs` and `.stderr`; `crates/offcut-plan/tests/snapshots/*.snap`.

### Files To Modify

- `crates/offcut-plan/src/lib.rs` (`#[cfg(feature = "prompt")] pub mod prompt;`), `crates/offcut-plan/Cargo.toml`, `Cargo.lock`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- The render bundle carries no schema code: `offcut-wasm-render` will depend on `offcut-plan` with `default-features = false` (Prompt 98); `prompt.rs` compiles only with the feature.
- The snapshot names carry `CATALOG_VERSION`, so a prompt change without a version bump fails (§7.3).

### Data Types / Interfaces

As §7.3:

```rust
pub struct PromptInput<'a> { pub duration: DurMs, pub words: &'a [Word], pub prosody: &'a [WordProsody], pub sentences: &'a [Sentence] }
pub fn system_prompt() -> String;
pub fn output_schema() -> serde_json::Value;
pub fn user_message(input: &PromptInput<'_>) -> String;
```

### Implementation Requirements

#### 1. The 21 rule cases
One per: V-1 format, V-1 catalog, `UnknownKind`, `WrongProducer` (Detected in Model), `WrongProducer` (word arm in Rules), `ParamsMismatch` (arm vs kind), `ParamsMismatch` (required missing), `AnchorsOutOfRange` (empty), `AnchorsOutOfRange` (past the end), `AnchorRule` (word count), `AnchorRule` (`at` outside anchors), `LabelTooLong` (characters, a multi-byte label at the limit passes), `LabelDigits`, `NumberUnresolved` (range not exactly a number), `NumberUnresolved` (FromTo second missing), `OverLimit`, `TooClose`, `ZoneTaken`, KeywordPop never `ZoneTaken`, `NotInThisBuild` (each non-item field), sort order. Each with the smallest hand-written plan.

#### 2. INV-22 case
The model plan on `a01`; edit the word that says the number; validate again; the `ValidItem`'s `EventParams` show the edited value.

### Error Handling

Test code may `expect`. A labeled file that fails to load fails the sweep naming the file.

### Concurrency / Async Requirements

None.

### Persistence Requirements

Snapshots committed under `crates/offcut-plan/tests/snapshots/` (ignored by the tree script as `*.snap`).

### API Requirements

The output schema is the contract V6.1's server will send; pinned by its snapshot.

### Security Requirements

The user message holds only the founder-owned labeled transcript `a01`; no lent clip's words (SAB rule 9).

### Testing Requirements

#### Unit Tests
`prompt.rs` inline: deterministic output (two calls equal).

#### Integration Tests
The five files above.

#### Failure-Path Tests
G 4.5: make `ValidPlan`'s fields `pub` → `tests/ui.rs` fails; change one word of `system_prompt()` → `catalog_snapshot.rs` fails; a `Detected` item in a `Model` plan → `WrongProducer`.

### Validation Commands

```bash
cargo test -p offcut-plan                                         # pending snapshots on the first run
cargo insta review                                                # read all three; accept
cargo test -p offcut-plan                                         # all pass, no pending snapshot
cargo test -p offcut-plan --test validate_rules_plans -- --nocapture   # "31 files, 0 items dropped"
cargo build -p offcut-plan --target wasm32-unknown-unknown --no-default-features --release   # Finished
INSTA_UPDATE=no CI=true cargo test -p offcut-plan                 # passes (what CI will run)
```

### Acceptance Criteria

- [ ] `prompt.rs` produces the three outputs of §7.3; the prompt names the four kinds with purposes, carries the "never write a number" rule and no example number; the schema has no `detected` arm and no `source`.
- [ ] `tests/validate.rs` has the 21 cases plus the model-plan, INV-22 and non-item cases; all pass.
- [ ] `validate_rules_plans.rs` prints "31 files, 0 items dropped" (D-165); `plan_from_rules.rs` passes on the reference events.
- [ ] Three snapshots committed after a reading; `tests/ui.rs` passes with its committed `.stderr`; the three guards fired and were undone.
- [ ] Rust count above Prompt 96's by the new tests; CI green.

### Agent Instructions

Before: read D-123, D-163 to D-165, §7.3, §7.8, §18, G 4.4-4.5, M-4. During: write each test file with the smallest inputs; read every snapshot. After: SAB gate; commit `V4 P97 (S4): plan prompt, validation tests, catalog snapshots`; push; read CI.

### Expected Result

`offcut-plan` is complete and green: catalog, prompt, schema, validation, rules producer, five test files, three read snapshots.

---

## Prompt 98 (V4 08/38) — The Rust contract switch: scene, renderer and binding on the plan, V3's look (S5 part 1)

**Effort: High.** One contract switch across three crates where nothing compiles in between, with every visual value kept exactly V3's; the identity reference is the instrument that says whether anything moved.

### Objective

Move the scene to `SceneInput` v1.2 (valid plan, time map, theme, font set, program time), make the video a draw command, replace `FontId` with `FontKey`/`FontSet`, put colours behind theme roles and motion into data, change the renderer to `render(sources, list, fonts)` with the V4 support rule, and give the binding `rules_plan` and a validating `set_scene`, all while drawing exactly V3's display lists.

### Project State Before This Prompt

- Prompt 97 committed; `offcut-plan` complete.
- The identity reference compares equal with V3's scene (Prompt 93).

### Why This Comes Here

D-116: contracts first, proven to change nothing, before any look changes. The Rust side switches together because `FontId` and `Renderer::render` cross three crates (G item 2).

### Scope

#### Implement

G 5.1 rows 1-11 in order, `cargo check -p <crate>` after each row, `cargo test -p <crate>` at the end of each crate:

1. `display_list.rs` §8.2: `FontKey(pub u16)` for `FontId`; `CropRect` moved here; `VideoSlot { Main }`, `ImageKey`, `DrawCmd::Video`, `DrawCmd::Image` appended in §8.2's order; `scale` leaves `Video` alone and scales an `Image`'s rect. **REOPENED V2 CONTRACT.**
2. `fonts.rs`, `layout.rs` §8.3: `FontSet`, `FontFace` (data, family, weight, coords), `embedded()` with keys 0-3 in V2's order (Inter 700, Inter 900, JetBrains Mono 700, Noto Emoji; key 4 is Prompt 105's), `get`, `EMOJI`; `shape(text, key, size, fonts)`; `normalized_coords` moved into `FontFace.coords` (its test moves to `fonts.rs`).
3. `theme.rs` [ONLY colour values], `styles.rs`, `motion.rs` §8.4-§8.5: `Theme`, `Role`, `FontRole { Display, Body, Mono }`, `default_for` returning **V3's colours and fonts for all three styles** (V3 drew Bold and Tech as Clean); `StyleSpec` on roles and `FontRole`; `CaptionMotion` holding V3's single motion (ease-out cubic, 120 ms, `Highlight`); `MotionUnit`, `MotionPreset`, `ActiveTreatment { Highlight, Cursor }`, `enter_at`. **REOPENED V2 CONTRACT (`styles.rs`).**
4. `safe_area.rs` D-138: `ZoneId { Event, Caption, Full }`, `zone(id)` with V3's numbers (Event 860-1180, Caption 1220-1500).
5. `anim.rs`, `captions.rs`, `events/mod.rs`, `events/number_reveal.rs`: `OutMs`/`OutSpan` throughout; layout from `ValidItem` (the `Detected` params); colours from roles.
6. `video.rs` §8.8: `Track::fixed(crop)`, `cmd(t, canvas)`.
7. `lib.rs` §8.1: `SceneInput` v1.2, `build_scene` (without placement and readability: Prompts 103-104), `frame_at(OutMs)` with `Video` first, `source_at`, `fonts()`, `duration() == time_map.out_duration()`; `crop()` removed; `summary()` kept as V2's until Prompt 106.
8. Inline tests moved to the new calls, **no expected value changes** (D-160); the "frames" helper uses `to_out_ms`.
9. `tests/timemap_identity.rs` §8.10.
9b. `tests/v3_reference.rs`: rewritten into its comparison form (G 6.1: rules plan → validate → `build_scene` on `TimeMap::identity`, `Theme::default_for(Clean)`, `FontSet::embedded()`; normalisation: drop a leading `Video`, read `FontKey(n)` as V3's `FontId` of index `n`; `f32` equality) and marked `#[ignore = "identity proof, S6"]`. Run it with `--ignored` while working.
10. `offcut-render` §9.1-§9.3: `RenderError::Unsupported`; `VideoSources`; `render(sources, list, fonts)`; `composite.rs` with the V4 support rule (exactly one `Video`, first, slot `Main`, opacity 1.0, no mask, `dst` inside the canvas; any `Image` → `Unsupported`; black outside `dst`); `video_pass.rs` `src` per call and `dst` uniform; `vello_backend.rs` glyphs by `FontKey` through the `FontSet` (one cached `peniko::Font` per key), skipping `Video` and `Image`. **REOPENED V2 CONTRACT.**
11. `offcut-wasm-render` §11.1: `Cargo.toml` gains `offcut-plan = { workspace = true, default-features = false }`; `scene_api.rs`: `rules_plan(transcript, events)`, `set_scene(input)` validating the plan (refusal → `E_INTERNAL` with detail `PlanFormat` or `PlanCatalog`), refusing a non-identity map (`TimeMapNotIdentity`), taking `Theme::default_for(edit.style)` and `FontSet::embedded()`, answering `SceneReport { dropped }`; `source_at(t_out_ms)`; `session.rs` `render_frame(frame, t_out_ms)` building `VideoSources { main: (frame, clip.rotation) }`.

`offcut-scene/Cargo.toml` gains `offcut-plan` and dev `insta` (json) if not yet present.

#### Do Not Implement Yet

- The TypeScript side (Prompt 99). Program audio (Prompt 100). Any new visual, style value, placement, readability, emphasis, pill, key 4 (Prompts 103-107). `hit`, `resize` (Prompts 104, 112). Headless rendering (Prompt 101).

### Existing Components To Reuse

V2/V3's layout, chunking, number-reveal layout and fit rules, `framing::crop_rect` (offset still the V2 stand-in until Prompt 106), the video pass's rotation handling, the vello backend's run drawing.

### Files To Create

- `crates/offcut-scene/src/{theme,motion,video}.rs`, `crates/offcut-scene/tests/timemap_identity.rs`.

### Files To Modify

- `crates/offcut-scene/src/{display_list,fonts,layout,styles,safe_area,anim,captions,lib}.rs`, `src/events/{mod,number_reveal}.rs`, `crates/offcut-scene/tests/v3_reference.rs`, `crates/offcut-scene/Cargo.toml`; `crates/offcut-render/src/{lib,composite,video_pass,vello_backend}.rs`; `crates/offcut-wasm-render/src/{scene_api,session}.rs`, `crates/offcut-wasm-render/Cargo.toml`; `Cargo.lock`; `docs/v4/v4changelog.md`.

### Architecture Requirements

- **No visual value changes** (D-116, G item 2). If `cargo test -p offcut-scene --test v3_reference -- --ignored` shows a difference, find it now in this prompt's files.
- The scene parses no number (D-121) and moves no time except through the map (D-126); `frame_at` takes `OutMs` only.
- The renderer has no crop parameter; the crop is in the `Video` command (D-131).
- `offcut-wasm-render` builds the plan crate without `prompt` (D-122): the release wasm carries no `serde_json`-schema code.

### Data Types / Interfaces

§8.1, §8.2, §8.3, §8.4 (`Theme`, `StyleSpec`, `PanelSpec`), §8.5, §9.1, §11.1 exactly. The binding's `set_scene` input: `{ transcript, plan: EditPlan, edit: EditInput, profile, timeMap: TimeMap }`.

### Implementation Requirements

#### 1. Order and checks
G 5.1's order; a crate's tests pass before the next crate starts. `cargo clippy --workspace --all-targets -- -D warnings` at the end.

#### 2. The identity reference while working
Run `cargo test -p offcut-scene --test v3_reference -- --ignored --nocapture` after rows 5, 7 and 10; record the result in the entry (identical already, or the differences Prompt 100 must fix).

#### 3. The wasm size
`pnpm build:wasm`; record `offcut_render_bg`'s size against Entry 1 (§21.1: over 300 kB growth is investigated now).

### Error Handling

`SceneError::{Font, Layout, TimeMap}`; `RenderError::Unsupported` → `E_INTERNAL` detail `Unsupported`; the binding maps plan refusals and a non-identity map to `E_INTERNAL` with the details above.

### Concurrency / Async Requirements

None (the font set and the shaping of V2 stay `thread_local!`).

### Persistence Requirements

None.

### API Requirements

The binding's JS surface changes: `rules_plan` (new), `set_scene` (input and answer), `source_at` (new), `render_frame(frame, t_out_ms)`. The TypeScript side follows in Prompt 99; **until then the media cases that reach `setScene` fail** (SAB rule 4).

### Security Requirements

None beyond the SAB.

### Testing Requirements

#### Unit Tests
Every V2/V3 inline test in `offcut-scene` and `offcut-render` on the new calls with V3's expected values; `theme.rs` inline: `default_for` equals V3's constants for each style.

#### Integration Tests
`timemap_identity.rs`; `v3_reference.rs` with `--ignored` (information).

#### Failure-Path Tests
`composite.rs` inline: a list with an `Image`, a list with two `Video`s, a `Video` not first, a mask → `Unsupported`. `scene_api` inline (native-compilable part): a non-identity map refused.

### Validation Commands

```bash
cargo test -p offcut-scene -p offcut-render -p offcut-plan        # all pass with V3's expected values
cargo test -p offcut-scene --test v3_reference -- --ignored --nocapture   # information only; record it
cargo clippy --workspace --all-targets -- -D warnings             # clean
git grep -n "FontId\|fn crop()" -- crates                         # no output
pnpm build:wasm && ls -l web/src/wasm/pkg/*render*_bg.wasm 2>/dev/null || ls -l web/dist/assets | grep render   # record the size
```

### Acceptance Criteria

- [ ] The scene takes `SceneInput` v1.2 and draws on `OutMs`; `Video` is the first command of every frame; `FontKey`/`FontSet`, `Theme`, `CaptionMotion`, `ZoneId` exist; no `FontId`, no `crop()`.
- [ ] The renderer draws `render(sources, list, fonts)` under the V4 support rule; `Unsupported` cases tested.
- [ ] The binding exposes `rules_plan`, a validating `set_scene` answering `SceneReport`, `source_at`, `render_frame(frame, t_out_ms)`; built without the `prompt` feature.
- [ ] Every inline expectation is V3's; `timemap_identity.rs` passes; the reference result is recorded; the wasm size recorded.
- [ ] REOPENED CONTRACT lines for `display_list.rs`/`FontId`, `styles.rs`, `Renderer::render`; their rows closed. SAB gate green (non-media); CI green.

### Agent Instructions

Before: read D-116, D-117, D-121, D-126, D-130 to D-138, §8.1-§8.8, §9.1-§9.3, §11.1, G Phase 5 and G item 2. During: crate by crate; never change a value to make a test pass. After: SAB gate (media not run: SAB rule 4); commit `V4 P98 (S5): scene, renderer and binding on the plan contract (V3 look)`; push; read CI.

### Expected Result

The Rust side of the V4 contract compiles and passes with V3's values; the page still speaks V3's protocol until Prompt 99.

---

## Prompt 99 (V4 09/38) — The page on the V4 contract: protocol, loaders, worker, pipeline, `OutMs` (S5 part 2)

**Effort: High.** A reopened V1 protocol and a `MINTING` change across eight files; the preview must look exactly as in V3 at the end.

### Objective

Switch the TypeScript side to the V4 contract: `rulesPlan`, `setScene({ transcript, plan, edit, timeMap, profile }) → SceneReport`, program time in the clock and seek types, the plan computed after detection and held by the clip store, and the identity time map built in one named helper; then look at the preview and confirm it is V3's.

### Project State Before This Prompt

- Prompt 98 committed; the binding speaks the V4 contract; media cases that reach `setScene` fail.

### Why This Comes Here

D-124: the rules plan takes the road V6.1's model plan will take (page → `setScene` → validated in Rust). Finishing S5 here ends the expected media failures of SAB rule 4.

### Scope

#### Implement

G 5.2 rows 1-8 and G 5.3:

1. `workers/protocol.ts` §14.1: `rulesPlan`; `setScene` with `plan` and `timeMap`, answering `SceneReport`; `ClockSync.audioMs: OutMs`; `previewSeek({ at: OutMs })` (type only; its handler is Prompt 112's); `previewPlay → PreviewStats` (the loop returns `{ frames, late, maxDriftMs, downscaled: false }` now). `assemble`, `release` (Prompt 100) and `hitTest`, `styleThumbnails` (Prompts 113, 116) are not added yet. **REOPENED V1 CONTRACT.**
2. `wasm/load-render.ts` §14.4: `RenderApi.rulesPlan`; `RenderSession.set_scene → SceneReport`, `source_at`.
3. `render.worker.ts` §15.1: `rulesPlan` handler (stateless); `setScene` passes `plan` and `timeMap` and keeps the input; `detect` keeps `{ transcript, prosody, events }` (D-146).
4. `render/preview-job.ts`, `render/preview-loop.ts` §15.2: `OutMs`; `session.source_at(tn)` → `source.frameAt(at)`; statistics returned (no resize rule yet).
5. `render/export-job.ts`, `render/export-loop.ts` D-129: the plan and the map in the profile switch and back; `render_frame` with the same number typed `OutMs`.
6. `usecases/run-pipeline.ts` §16.2: after `detect`, `rulesPlan` → `clipStore.setPlan`; the identity map from one helper named for what it is (`identityTimeMap(duration)`); `setScene` with both.
7. `usecases/control-preview.ts`, `web/eslint.config.js` D-126: `toTimeMs` becomes `toOutMs`; the `MINTING` entry moves from `TimeMs` to `OutMs`. **REOPENED V2 CONTRACT (`eslint.config.js`).**
8. `state/clip-store.ts` §16.1: `plan?`, `timeMap?`, `setPlan`, `setEvents`.

G 5.3: `pnpm dev`, the sample clip, the four checks of G 5.3's table.

#### Do Not Implement Yet

- `assemble`, `release`, `prog48` (Prompt 100). Seek's handler, resize rule (Prompt 112). Review flows (Prompts 113-117). Cancel and crash (Prompts 118-119).

### Existing Components To Reuse

- `pool.render` calls, `clipStore` actions, V3's `run-pipeline.ts` step order, the generated `EditPlan`, `TimeMap`, `OutMs`, `Dropped` types.

### Files To Create

None.

### Files To Modify

- `web/src/workers/protocol.ts`, `web/src/wasm/load-render.ts`, `web/src/workers/render.worker.ts`, `web/src/workers/render/{preview-job,preview-loop,export-job,export-loop}.ts`, `web/src/usecases/{run-pipeline,control-preview}.ts`, `web/eslint.config.js`, `web/src/state/clip-store.ts`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- `OutMs` is minted in exactly one place (`control-preview.ts`'s helper); the lint's branded list reads it from the generated file (§12).
- The identity map is built in one helper and nowhere else in `web/src` (D-164; `git grep` proves it).
- Nothing is stored: the plan lives in memory in the clip store (D-124, D-82).

### Data Types / Interfaces

§14.1's `RenderWorkerApi` for the members listed above; `SceneReport = { dropped: Dropped[] }`; `PreviewStats = { frames; late; maxDriftMs: DurMs; downscaled }`.

### Implementation Requirements

#### 1. The pipeline (§16.2)
`detect` → `rulesPlan({ transcript, events })` → `clipStore.setPlan(plan)` → `setScene({ transcript, plan, edit, timeMap: identityTimeMap(clipInfo.duration), profile: "preview" })`. The `pipeline_done` counts stay as V3 computes them (D-160).

#### 2. The look check (G 5.3)
Captions in V3's Clean look in all three styles; NumberReveal counts up; the other three kinds not drawn; console clean; `setScene`'s answer `dropped: []`; the feed's lines as V3. If anything differs, stop and find it before committing.

### Error Handling

A `setScene` refusal (`E_INTERNAL` `PlanFormat`/`PlanCatalog`/`TimeMapNotIdentity`) reaches the existing failure path as V3's `E_INTERNAL` does.

### Concurrency / Async Requirements

Unchanged: one `setScene` at a time, the preview loop as V3.

### Persistence Requirements

None.

### API Requirements

None (no network).

### Security Requirements

None beyond the SAB.

### Testing Requirements

#### Unit Tests
Vitest 92, unchanged.

#### Integration Tests
`pnpm e2e` 12; `pnpm e2e:media` (keyed): 49 passed, 1 skipped (the V3 expectations; D-160's two rewrites come in Prompt 124 with the statistics, and "Events" passes unchanged because the counts are the same).

#### Failure-Path Tests
None new.

### Validation Commands

```bash
pnpm build:wasm && pnpm check                                     # exit 0
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"   # 92 passed
git grep -n "segments: \[{ kind: \"source\"" -- web/src           # one line: the identity helper
git grep -n "toTimeMs" -- web/src                                 # no output
pnpm e2e:media 2>&1 | tail -3                                     # 49 passed, 1 skipped (keyed; unset the key after)
cargo test -p offcut-scene --test v3_reference -- --ignored       # information: identical already, or Prompt 100's list
```

### Acceptance Criteria

- [ ] The page computes the rules plan after detection, holds it in the clip store and sends it with the identity map; `setScene` answers `dropped: []` on the sample clip.
- [ ] `ClockSync.audioMs` and `previewSeek.at` are `OutMs`; the `MINTING` entry mints `OutMs`.
- [ ] In `pnpm dev` the preview looks as in V3 (G 5.3's four checks, recorded).
- [ ] The 48 media cases pass again; REOPENED CONTRACT lines for `protocol.ts` and `eslint.config.js`; CI green.

### Agent Instructions

Before: read D-124, D-126, D-129, §14.1, §14.4, §15.1-§15.2, §16.1-§16.2, G 5.2-5.3, M-5. During: types first (`protocol.ts`), then loaders, worker, jobs, use-cases, store; `pnpm check` after each file. After: SAB gate plus the media run; commit `V4 P99 (S5): protocol, worker and pipeline on the plan contract (V3 look)`; push; read CI.

### Expected Result

The whole app runs on the V4 contract and looks exactly like V3.

---

## Prompt 100 (V4 10/38) — The identity proof and program audio (S6)

**Effort: High.** The proof must pass value for value with `f32` equality, and a failure is fixed in the switch, never in the reference; program audio reopens a V3-frozen worker contract and changes the order of step F.

### Objective

Make the identity reference pass frame for frame, add program audio (`assemble`, `silence_profile`, `program_identity.rs`), expose it through the binding and the audio worker with a one-way `release`, play `prog48` in the preview, and prove the 48 V3 media cases still pass.

### Project State Before This Prompt

- Prompt 99 committed; the page runs on the V4 contract; the `#[ignore]`d reference shows identical or a recorded list of differences.

### Why This Comes Here

D-116, D-117: the look must not change before the proof passes. S-6 (program audio) is the last seam that changes a pipeline step; with the identity map it changes no sample, which the same milestone proves.

### Scope

#### Implement

- G 6.1: remove the `#[ignore]`; make `v3_reference.rs` pass: 6 inputs, every 7th frame, identical after the normalisation. Fix any difference in Prompt 98-99's files (G 6.1's "where to look" table), never in the reference.
- G 6.2 rows 1-7:
  1. `offcut-dsp/src/silence.rs`, `program.rs` [ONLY place that changes duration], `lib.rs` §10: `ProgramConfig`, `MusicInput`, `ProgramAudio`, `assemble` (identity map and no music → `out48.to_vec()` and the chain meter's loudness; anything else → `DspError::Unsupported`, mapped to `E_DSP` detail `Unsupported`), `silence_profile` (RMS dB per 10 ms window, floor -100 dB).
  2. `offcut-dsp/tests/program_identity.rs` §10 (reads `fixtures/.gen/speech/*.pcm48.f32` through V3's stamp helper).
  3. `offcut-wasm-core/src/audio_api.rs` §11.2: `assemble(out48, time_map, speech)` → `{ prog48, loudness }`.
  4. `wasm/load-core.ts`: `CoreApi.assemble`.
  5. `workers/protocol.ts`, `audio.worker.ts` §14.1, §15.4: `assemble` (transfer in and out, keep nothing), `release` (one-way; `kept = null`), `serveWorker`'s `oneWay: ["release"]`; the worker's comment gains D-127's sentence. **REOPENED V3 CONTRACT (`AudioWorkerApi`).**
  6. `usecases/run-pipeline.ts` §16.2, D-127, D-128: step F writes `opfs.writeAudio(out48)` first, then `pool.audio.assemble` with `out48` transferred, then `clipStore.ready({ …, prog48 })`; every early end calls `pool.audio.release()`.
  7. `state/clip-store.ts`, `usecases/control-preview.ts`, `ui/components/PreviewPlayer.tsx`: `out48` → `prog48` in the store and in `attach`.
- G 6.3: `pnpm e2e:media`; in `pnpm dev`, `prog48.length` equals duration x 48, and the preview sounds as V3.

#### Do Not Implement Yet

- The export reading `prog48` (V5, D-129). Any non-identity map, cut or mix (D-164). Cancel's use of `release` (Prompt 119; the early ends of V3 call it now).

### Existing Components To Reuse

- `loudness.rs`'s meter; V3's fixtures stamp helper; `audio.worker.ts`'s `kept` buffer and transfer pattern (D-110); `opfs.writeAudio`.

### Files To Create

- `crates/offcut-dsp/src/{program,silence}.rs`, `crates/offcut-dsp/tests/program_identity.rs`.

### Files To Modify

- `crates/offcut-scene/tests/v3_reference.rs`, any of Prompt 98-99's files a difference points to, `crates/offcut-dsp/src/lib.rs`, `crates/offcut-wasm-core/src/audio_api.rs`, `web/src/wasm/load-core.ts`, `web/src/workers/{protocol,audio.worker}.ts`, `web/src/usecases/{run-pipeline,control-preview}.ts`, `web/src/state/clip-store.ts`, `web/src/ui/components/PreviewPlayer.tsx`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- `assemble` is the only function of `offcut-dsp` that may return another length (INV-10, TS §40.6); in V4 it never does.
- The audio worker holds no audio after `assemble` (D-128); `release` is fire-and-forget.
- PCM crosses threads by transfer (TS §31); `out48` is written to OPFS **before** it is transferred.
- The export still reads `out48.f32` (D-129).

### Data Types / Interfaces

```rust
pub fn assemble(out48: &[f32], map: &TimeMap, speech: &[OutSpan], music: Option<MusicInput<'_>>, cfg: &ProgramConfig) -> Result<ProgramAudio, DspError>;
pub fn silence_profile(out48: &[f32]) -> SilenceProfile;
```

```ts
assemble(p: { out48: Float32Array; timeMap: TimeMap; speech: OutSpan[] }): Promise<{ prog48: Float32Array; loudness: Lufs }>;
release(): void;
```

### Implementation Requirements

#### 1. The proof
Print "6 inputs, <n> frames, identical"; write it into the entry as "identity proof passed at S6". `git diff --stat <Prompt 93's commit> -- crates/offcut-scene/tests/snapshots/v3_reference` prints nothing.

#### 2. `program_identity.rs` (§10)
On the derived portrait and long-pause PCM: sample-for-sample equality and loudness within 0.01 LU of `run_chain`'s; a non-identity map → `Unsupported`; a music input → `Unsupported`; `silence_profile` of 1 s of zeros = 100 windows at -100 dB; of a full-scale 1 kHz sine = 100 windows near -3 dB.

### Error Handling

`DspError::Unsupported` → `E_DSP` detail `Unsupported`. An `assemble` failure in the pipeline follows V3's `E_DSP` path and calls `release()`.

### Concurrency / Async Requirements

Step F is sequential: write, then assemble, then ready. `release` is posted on every early end (refusal, failure, start over).

### Persistence Requirements

OPFS `out48.f32` exactly as V3 writes it.

### API Requirements

None (no network).

### Security Requirements

None beyond the SAB.

### Testing Requirements

#### Unit Tests
`program.rs`, `silence.rs` inline smoke tests.

#### Integration Tests
`v3_reference.rs` (no ignore), `program_identity.rs`, the media suite.

#### Failure-Path Tests
The two `Unsupported` cases.

### Validation Commands

```bash
cargo test -p offcut-scene --test v3_reference -- --nocapture     # 1 passed; "6 inputs, <n> frames, identical"
cargo test -p offcut-dsp --test program_identity                  # all pass
cargo test --workspace 2>&1 | awk '/^test result:/ {p+=$4; f+=$6; i+=$8} END {print p" passed, "f" failed, "i" ignored"}'   # 0 failed; 3 ignored
pnpm e2e:media 2>&1 | tail -3                                     # 49 passed, 1 skipped (keyed)
git diff --name-only "$BASE" -- crates/offcut-dsp/tests           # only program_identity.rs
```

### Acceptance Criteria

- [ ] The identity reference passes with no `#[ignore]`; the six JSON files are unchanged since Prompt 93.
- [ ] `assemble` and `silence_profile` exist and pass `program_identity.rs`; `assemble` refuses any non-identity map or music.
- [ ] The audio worker exposes `assemble` and one-way `release`; step F writes, then assembles; the preview plays `prog48` (length checked in dev).
- [ ] The 48 media cases pass; REOPENED CONTRACT line for `AudioWorkerApi`; V4 CL known issue 1 closed; V3 §23 item 25 recorded as closed (D-128); CI green.

### Agent Instructions

Before: read D-116, D-117, D-127 to D-129, §10, §11.2, §14.1, §15.4, §16.2, G Phase 6 and its failure table. During: the proof first; program audio second. **If the reference differs anywhere beyond the normalisation, fix Prompts 98-99's files.** After: SAB gate plus the media run; commit `V4 P100 (S6): identity proof passed; program audio (identity) in the worker and the preview`; push; read CI.

### Expected Result

The seams S-1 to S-6 are in place and proven to change nothing: same display lists, same audio, same media results.

---

## Prompt 101 (V4 11/38) — Native rendering, SSIM, the two video-only goldens, the identity reference retired (S7)

**Effort: High.** GPU readback with row padding, a similarity measure written by hand, and the first reading across two software adapters; a wrong answer here decides how every later golden is stored.

### Objective

Retire the identity reference, let the renderer draw into memory on a software adapter, write the SSIM measure, commit the two video-only goldens that the look cannot change, add the Mesa line to CI, and record the similarity between D1's WARP and the runner's lavapipe.

### Project State Before This Prompt

- Prompt 100 committed; the identity proof passed and is recorded.
- Entry 1 says whether the runner's adapter was probed in Prompt 91 or deferred here.

### Why This Comes Here

The goldens (D-135) need native rendering (D-134). Only the two goldens whose list holds a `Video` command and nothing else are written now: the look of Prompts 105-107 would change the other fourteen (G item 3, G (b) 4).

### Scope

#### Implement

- G 7.1: `git rm` the reference test and its six files; remove their TS §5 lines in the same commit (D-113).
- G 7.2 rows 1-5: `gpu.rs` headless target (`Rgba8Unorm` with `COPY_SRC`) and readback buffer (rows padded to 256 bytes, padding removed on read); `video_pass.rs` `load_image(&RgbaImage, rotation, src)`; `lib.rs` `new_headless(width, height)` (`force_fallback_adapter: true`), `render_to_image(source, rotation, list, fonts)`, `RgbaImage`, all behind `#[cfg(not(target_arch = "wasm32"))]`; `tests/common/ssim.rs` (BT.709 luma, 8 x 8 windows, C1 = (0.01 x 255)², C2 = (0.03 x 255)², the mean over windows); `tests/golden_frames.rs` with the synthetic `pattern(w, h)` (diagonal gradient, 8 x 8 grid of 2-pixel lines, a filled disc at a third of the width), the adapter-name assertion (a software adapter or fail with its name), `OFFCUT_GOLDEN_UPDATE`, `GOLDEN_SSIM_MIN = 0.98`, the missing-golden message, and the two cases `crop_landscape_050` (1920 x 1080 source, one `Video { src: the 0.5-offset crop, dst: full }`) and `punch_in_110` (portrait source, `src` the centre 1/1.1), both at 540 x 960; the test writes its output images to `target/golden-out/` on failure.
- G 7.3: `ci.yml`'s Linux job, before `cargo test`: `sudo apt-get install -y mesa-vulkan-drivers`, the version printed (`dpkg -s mesa-vulkan-drivers | grep Version`), `INSTA_UPDATE: "no"` and `CI: "true"` in the step's `env`, and `actions/upload-artifact` for `target/golden-out/` on failure (Phase 17 completes and checks it). Push, read the log's per-golden line (`ssim <name> = … on llvmpipe`). If Prompt 91 deferred the runner probe, this run is the probe: record the adapter name and the Mesa version as Prompt 91 would have.
- Apply G 7.3's reading table: both ≥ 0.98 → record in `experiments.md` "Golden similarity across adapters"; one below → per-adapter folders `fixtures/golden/{warp,lavapipe}/` (lavapipe images from the CI artifact), the reason recorded, the threshold never lowered.

#### Do Not Implement Yet

- The other fourteen goldens and the parity case (Prompt 107). Any scene change.

### Existing Components To Reuse

- The renderer's existing pipeline (`composite.rs`, `video_pass.rs`, `vello_backend.rs`) shared by both paths; the `png` dev-dependency; `FontSet::embedded()`.

### Files To Create

- `crates/offcut-render/tests/golden_frames.rs`, `crates/offcut-render/tests/common/ssim.rs`, `fixtures/golden/crop_landscape_050.png`, `fixtures/golden/punch_in_110.png`.

### Files To Modify

- `crates/offcut-render/src/{gpu,video_pass,lib}.rs`, `.github/workflows/ci.yml`, `docs/technicalspec.md` (§5: the temporary lines removed), `docs/v4/experiments.md`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- Native code never reaches the browser bundle: every native function behind `cfg(not(wasm32))`; the wasm32 tree check of Prompt 92 still prints nothing.
- Both paths share every line but the source and the target (§9.1).
- A golden's source is synthetic (D-135); no recording is read.

### Data Types / Interfaces

§9.1:

```rust
#[cfg(not(target_arch = "wasm32"))]
impl Renderer {
    pub async fn new_headless(width: Px, height: Px) -> Result<Renderer, RenderError>;
    pub fn render_to_image(&mut self, source: &RgbaImage, rotation: Rotation, list: &DisplayList, fonts: &FontSet) -> Result<RgbaImage, RenderError>;
}
pub struct RgbaImage { pub width: u32, pub height: u32, pub data: Vec<u8> }
```

### Implementation Requirements

#### 1. Readback
`bytes_per_row` rounded up to 256; copy texture → buffer; map; strip the padding row by row into `RgbaImage`.

#### 2. Look at the two PNGs (G 7.2)
The crop shows the right-of-centre strip of the pattern; the punch-in shows the centre enlarged with thicker grid lines. Record what you saw.

#### 3. Two pushes
The first commit (`V4 P101 (S7, wip): golden_frames and two video-only goldens`) is pushed so the Linux job can read the images (G 7.3); the prompt's final commit follows after the reading.

### Error Handling

`new_headless` with no software adapter → `RenderError::NoAdapter`; the test prints the adapter info it got and fails. D-134's fallback (skip on the machine that has none, with the reason printed) only if Prompt 91 recorded it.

### Concurrency / Async Requirements

The test drives the async constructor with a small `block_on` written in the test file (no new dependency; `pollster` was the probe's only).

### Persistence Requirements

`fixtures/golden/*.png` committed; `target/golden-out/` never committed.

### API Requirements

None.

### Security Requirements

None beyond the SAB.

### Testing Requirements

#### Unit Tests
`ssim.rs`: identical images → 1.0; an image against its inverse → well below 0.5.

#### Integration Tests
`golden_frames.rs`: 2 passed on D1 and on lavapipe.

#### Failure-Path Tests
A golden darkened by hand (locally, not committed) fails with its SSIM.

### Validation Commands

```bash
OFFCUT_GOLDEN_UPDATE=1 cargo test -p offcut-render --test golden_frames -- --nocapture   # 2 written; prints the adapter name
cargo test -p offcut-render                                       # all pass on D1
git ls-files fixtures/golden                                      # the two PNGs (or two per adapter folder)
git ls-files crates/offcut-scene/tests | grep v3_reference        # no output
gh run view --log --job <ci job id> | grep -E "golden_frames|ssim|llvmpipe|mesa"          # 2 passed on llvmpipe; the version line
```

### Acceptance Criteria

- [ ] The identity reference and its files are gone, with their TS §5 lines.
- [ ] `render_to_image` works on WARP (D1) and lavapipe (CI); two goldens pass on both, or per-adapter folders hold both with the reason recorded.
- [ ] `ci.yml` installs Mesa, prints its version, sets `INSTA_UPDATE=no` and `CI=true`, uploads `target/golden-out/` on failure.
- [ ] The cross-adapter SSIM values, both adapter names and the Mesa version are in `experiments.md` and the entry; CI green.

### Agent Instructions

Before: read D-134, D-135, §9.1, §9.3, §9.4, §21.1's golden row, G Phase 7. During: look at every PNG you write. After: SAB gate; commit `V4 P101 (S7): native rendering, SSIM, two video-only goldens; identity reference retired`; push; read the Linux log.

### Expected Result

The renderer can be tested without a browser on two software adapters, and two goldens guard the video command.

---

## Prompt 102 (V4 12/38) — The line-break table, before the algorithm (S8 part 1, human gate)

**Effort: High.** Thirty rows of ground truth decided by reading, not by code; once committed they change only with a changelog line, so a sloppy row is permanent.

### Objective

Write and commit the 30-row table of `crates/offcut-scene/tests/line_breaks.rs` (sentences, word times and the expected chunks and lines in Clean and Bold) before `readability.rs` exists, then present ten random rows to the founder as they would sit on a phone, and stop.

### Project State Before This Prompt

- Prompt 101 committed. No `readability.rs` exists; `captions::chunk` is V3's.

### Why This Comes Here

D-167 and G item 6: like V3's labels, the expected breaks must be decided before the algorithm, by reading, so the algorithm is measured against what a person would do and never the other way round.

### Scope

#### Implement

- G 8.1: the table part of `line_breaks.rs`: 30 rows, each a sentence with word times (300 ms per word; a 400 ms pause where the row says), and the expected chunks and lines in Clean (16 characters x 2 lines, 4 words, as spoken, D-170's values) and Bold (12 x 2, 3 words, upper case). Cover every case of G 8.1's table at least twice (article never ends a line; preposition never ends a line; break after a comma; clause starter begins a chunk; balanced two lines; one-word beat kept alone between pauses; one word not stranded otherwise; numbers stay whole; sentence end is a hard break; long words), plus five rows of everyday creator speech (a recipe step, a fitness cue, a product review line, a travel tip, a reaction; D-177).
- The test function that will compare is written now and marked `#[ignore = "the break search arrives in P103"]` so the commit compiles and CI stays green; the property checks over the labeled set are Prompt 103's.
- Commit and push the table alone (`V4 P102 (S8, table): the line-break table, before the algorithm`).
- G 8.2 preparation: `shuf -n 10` over the row numbers (record the seed's output), and write a reply that shows each chosen row as a phone would: the sentence, then each chunk as its two lines in a fixed-width block, Clean first, Bold second, with the question "Would you break it this way?" for each. Stop there.

#### Do Not Implement Yet

- `readability.rs`, the break search, the reading floor, the property checks (Prompt 103). Recording the founder's answers (Prompt 103).

### Existing Components To Reuse

- V2/V3's caption word rules (TS §17.2: a number is one caption word; sentence ends; `CHUNK_GAP_MS`), the test transcript builders of `offcut-scene`'s inline tests.

### Files To Create

- `crates/offcut-scene/tests/line_breaks.rs` (table and an ignored comparison).

### Files To Modify

- `docs/v4/v4changelog.md`.

### Architecture Requirements

- The rows are data in a `const` table: sentence, pause positions, `clean: &[&[&str]]` (chunks of lines), `bold: &[&[&str]]`.
- The expected outputs are decided by reading the sentence aloud, never by running any code.

### Data Types / Interfaces

```rust
struct Row { id: u8, sentence: &'static str, pause_before: &'static [usize], clean: &'static [&'static [&'static str]], bold: &'static [&'static [&'static str]] }
const ROWS: [Row; 30] = [...];
#[test] #[ignore = "the break search arrives in P103"] fn table();
```

### Implementation Requirements

#### 1. Each row checked by hand against the style limits
Every expected line fits its style's characters per line and every chunk its word limit; Bold's lines are upper case. A row that cannot fit its own rule is rewritten now.

#### 2. The ten rows for the founder
Shown exactly as the table holds them; nothing explained about costs or the algorithm.

### Error Handling

None (data).

### Concurrency / Async Requirements

None.

### Persistence Requirements

The table's commit SHA is recorded; from it on a row changes only with a changelog line naming the row and the reason (§23).

### API Requirements

None.

### Security Requirements

Sentences are written for the table; no lent clip's words.

### Testing Requirements

#### Unit Tests
None run (the comparison is ignored until Prompt 103).

#### Integration Tests
The SAB gate (the ignored count rises by 1, recorded).

#### Failure-Path Tests
None.

### Validation Commands

```bash
cargo test -p offcut-scene --test line_breaks                     # 0 passed, 1 ignored
grep -c "id:" crates/offcut-scene/tests/line_breaks.rs            # 30
git log --oneline -1 -- crates/offcut-scene/tests/line_breaks.rs  # the table commit
git ls-files crates/offcut-scene/src | grep readability           # no output
```

### Acceptance Criteria

- [ ] Thirty rows covering every case of G 8.1 at least twice and five creator rows; each row fits its style's limits.
- [ ] The table is committed and pushed before any break-search code exists; CI green.
- [ ] Ten rows are shown to the founder in phone layout; **Human:** the founder's answers are pending (left unticked).

### Agent Instructions

Before: read D-167, D-168, D-170 (Clean's limits), D-177, §8.9, §8.10, G 8.1-8.2, G item 6. During: decide every break by reading. After: SAB gate; commit (title above); push; read CI; then reply with the ten rows and stop. The next prompt starts with the founder's answers.

### Expected Result

A frozen, read-by-a-person line-break table on `origin/v4-build`, and ten rows waiting for the founder.

---

## Prompt 103 (V4 13/38) — The founder's reading recorded; readability, the break search, the reading floor (S8 part 2)

**Effort: High.** A dynamic-programming search with hard breaks, binder and clause costs, one-word rules and balanced lines, which must pass a table it was not written against, deterministically.

### Objective

Record the founder's ten answers (changing a row only where the answer was "no", with the reason), then write `readability.rs` and the break search of D-167 and the reading floor of D-168, until the 30 rows and the property checks over the labeled set pass.

### Project State Before This Prompt

- Prompt 102 committed and pushed; the founder has answered the ten rows (pasted into this session).

### Why This Comes Here

The reading is a gate (G 8.2): an answer that changes a row must change it before the algorithm exists. Then the search can be written against a fixed target.

### Scope

#### Implement

- G 8.2 record: a changelog block "Line-break reading (D-167)" with the ten row ids, each "yes" or "no, because <the founder's words>"; each "no" row's expectation changed now with that reason, in its own commit before any code (`V4 P103 (S8, reading): the founder's ten rows`).
- G 8.3 rows 1-3:
  1. `readability.rs` [ONLY] §8.9: `CLAUSE_STARTERS` (about 40) and `BINDERS` (about 70: articles, possessives, prepositions, "to", auxiliaries and modals, "not", "very", "really", cardinals that are not the last word of a number) as sorted lower-case `const` slices; the costs 0 (after `,` `;` `:` `—`), 1 (before a clause starter), 3 (plain), 12 (after a binder), 8 (a one-word chunk not isolated by pauses of `ISOLATED_WORD_PAUSE_MS = 250` on both sides), the imbalance rule (a third), `MIN_CHUNK_MS = 700`; an inline test that both lists are sorted and lower case.
  2. `captions.rs` §8.9: the search between hard breaks (sentence end, gap ≥ `CHUNK_GAP_MS`), at most 40 words per segment, `best[i]` over `j < i` where the chunk fits the style, the best line split inside each chunk, ties to the earlier break; the reading floor after chunking (`show.end = max(show.end, show.start + MIN_CHUNK_MS)` capped at the next chunk's `show.start` and at the last word's end plus `CHUNK_TAIL_MS` plus the silence after it). No word moves (INV-5).
  3. `tests/line_breaks.rs`: remove the `#[ignore]`; the property checks over the 31 labeled transcripts (no line ends with a binder unless the chunk allows no other split; no chunk exceeds its style's limits; every word exactly once and in order).
- G 8.3's rule: a failing row is fixed in the costs or the search, never in the row.

#### Do Not Implement Yet

- Emphasis, placement, hit testing, the cache, parity (Prompt 104). Clean's final look (Prompt 105): the table already uses Clean's final limits, so the tests set the style limits they need through `StyleSpec` values of D-170 if `styles.rs` still holds V3's (record which).

### Existing Components To Reuse

- `captions::words` (unchanged rule), the sentence list, the style's `max_words`, `max_chars_per_line`, `max_lines`; the labeled-set loader pattern of Prompt 97.

### Files To Create

- `crates/offcut-scene/src/readability.rs`.

### Files To Modify

- `crates/offcut-scene/src/captions.rs`, `crates/offcut-scene/src/lib.rs` (module list), `crates/offcut-scene/tests/line_breaks.rs`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- Deterministic (INV-7): a total order on ties; no float costs.
- Matching is on the lower-cased effective text of a caption word, punctuation stripped; binary search on the sorted slices.
- `captions.rs` stays under 400 lines (drawing moves out in Prompt 104).

### Data Types / Interfaces

```rust
// readability.rs [ONLY]
pub(crate) const CLAUSE_STARTERS: &[&str]; pub(crate) const BINDERS: &[&str];
pub(crate) const COST_AFTER_PUNCT: u32 = 0; pub(crate) const COST_CLAUSE: u32 = 1; pub(crate) const COST_PLAIN: u32 = 3;
pub(crate) const COST_AFTER_BINDER: u32 = 12; pub(crate) const COST_ONE_WORD: u32 = 8;
pub(crate) const MIN_CHUNK_MS: u32 = 700; pub(crate) const ISOLATED_WORD_PAUSE_MS: u32 = 250;
pub(crate) fn break_cost(prev: &str, next: &str) -> u32;
pub(crate) fn imbalance(a: u32, b: u32) -> u32;
```

### Implementation Requirements

#### 1. Complexity
O(n x max_words) per segment; a segment of one word is one chunk.

#### 2. V3's inline caption tests
They keep passing at V3's style values where they assert chunking that the new search agrees with; a V3 inline expectation that the new search changes is **not** silently edited: list it in the entry with the reason (D-160 does not cover it), and ask before changing it.

### Error Handling

No panic; a chunk that cannot fit even one word (a very long word) follows V2's scale rule and stays one chunk.

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

None beyond the SAB.

### Testing Requirements

#### Unit Tests
`readability.rs`: sorted and lower case; `break_cost` cases; `imbalance` at the threshold.

#### Integration Tests
`line_breaks.rs`: 30 rows + property checks.

#### Failure-Path Tests
None yet (Prompt 104 drills the binder cost).

### Validation Commands

```bash
cargo test -p offcut-scene --test line_breaks -- --nocapture       # 30 rows passed + property checks; prints the chunks of any failing row
cargo test -p offcut-scene                                         # all pass
git log --diff-filter=A --format=%h -- crates/offcut-scene/src/readability.rs   # newer than the table's and the reading's commits
```

### Acceptance Criteria

- [ ] The founder's ten answers are in the changelog; every "no" row changed before any code, with the reason.
- [ ] `readability.rs` holds the lists and costs as named constants; the search and the floor are in `captions.rs`.
- [ ] All 30 rows and the three property checks pass; no row changed to match the output.
- [ ] Any V3 inline caption expectation the search changes is listed and approved, not silently edited; CI green.

### Agent Instructions

Before: read D-167, D-168, §8.6, §8.9, G 8.2-8.3, and the founder's answers. During: change the costs or the search, never a row. After: SAB gate; commit `V4 P103 (S8): readability, the break search, the reading floor`; push; read CI.

### Expected Result

Captions break at phrase boundaries with balanced lines and stay up long enough to read, measured against a table a person decided first.

---

## Prompt 104 (V4 14/38) — Emphasis, placement, hit testing, the shaping cache, parity (S8 part 3)

**Effort: High.** Layout rules that must be exact at three snap points, a cache that must never change output, and a parity proof at three output sizes.

### Objective

Finish caption intelligence: the caption block and event zone from `CaptionPlacement`, the emphasis order in a new `captions_draw.rs`, `Scene::hit`, a deterministic LRU shaping cache, the preview-equals-export parity test, and the binding's `hit`.

### Project State Before This Prompt

- Prompt 103 committed; `line_breaks.rs` passes.

### Why This Comes Here

Placement, emphasis and hit testing read the chunks the break search produces; the cache is what makes the 150 ms edit-to-preview of D-173 possible before the editor exists.

### Scope

#### Implement

G 8.4 rows 1-6 and G 8.5:

1. `placement.rs` [ONLY] §8.9, D-171: `CAPTION_TOP = SAFE.top + 140`, `CAPTION_BOTTOM = SAFE.bottom - 140`, `SNAP_TOP = 0.15`, `SNAP_MIDDLE = 0.55`, `SNAP_BOTTOM = 1.0`; `caption_block(p, style) -> Rect` (280 lp tall, centre `CAPTION_TOP + p x (CAPTION_BOTTOM - CAPTION_TOP)`); `event_zone(block) -> Rect` (centre at or below 960: ends 40 lp above the block; above: starts 40 lp below); `safe_area::zone(id)` reads them for the clip's placement; `build_scene` uses them. `tests/placement.rs` (§8.10: at 0.0, 0.15, 0.55, 1.0; at 1.0 the event zone is exactly 860-1180).
2. `captions_draw.rs` §8.6, §8.9, D-169: the chunk drawing moved out of `captions.rs`; each word's fill in the order active → KeywordPop (`pop`) → number (when `edit.highlight_numbers`, `accent`) → `fill`; a word is a number when `captions::words` made it from a `NormalizedSpan` or its text parses as the captions show it.
3. `hit.rs`, `lib.rs` D-174: `Scene::hit(t, x, y) -> Option<Hit>`, `Hit::{Word(WordIdx), Captions, Video}`, glyph boxes padded 8 lp, no shaping; `tests/hit.rs`.
4. `layout.rs` D-173: the shaping cache in the `thread_local!` (a `BTreeMap` keyed by `(String, FontKey, u32 size bits)` with a use counter; `SHAPE_CACHE_ENTRIES = 2048`, least recently used evicted) and a test-only switch to disable it; `tests/scene_cache.rs` (six scenes cold, warm, disabled; every frame's list equal).
5. `tests/parity.rs` D-172: six scenes at 540, 720, 1080; equal after scaling within 0.01 lp.
6. `scene_api.rs` §11.1: `hit(t_out_ms, x, y)` with `x`, `y` as fractions of the canvas.

G 8.5's three guard drills.

#### Do Not Implement Yet

- The pill, the shadow, Bold's pop, Mono's panel and cursor values (Prompt 105). The worker's `hitTest` handler (Prompt 113). The watermark band test at the snap points (Prompt 106 with `watermark.rs`; `placement.rs`'s D-171 test checks the band's position from `SAFE` now).

### Existing Components To Reuse

- V2's layout-once rule, the chunk and line boxes `captions.rs` already computes, `safe_area`'s constants, `EditState.caption_position` and `highlight_numbers` (Prompt 95).

### Files To Create

- `crates/offcut-scene/src/{placement,captions_draw,hit}.rs`; `crates/offcut-scene/tests/{placement,hit,scene_cache,parity}.rs`.

### Files To Modify

- `crates/offcut-scene/src/{captions,safe_area,layout,lib}.rs`, `crates/offcut-wasm-render/src/scene_api.rs`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- `placement.rs` never reads `profile.watermark` (D-171, Appendix A): preview and export must place captions identically.
- The cache is deterministic: shaping a key always gives the same runs (INV-7); `scene_cache.rs` proves it.
- At placement 1.0 every V3 inline expectation still holds.

### Data Types / Interfaces

```rust
pub fn caption_block(p: CaptionPlacement, style: &StyleSpec) -> Rect;
pub fn event_zone(caption_block: Rect) -> Rect;
impl Scene { pub fn hit(&self, t: OutMs, x: f32, y: f32) -> Option<Hit>; }
pub enum Hit { Word(WordIdx), Captions, Video }
```

### Implementation Requirements

#### 1. Timing
`cargo test -p offcut-scene --release --test scene_cache -- --nocapture` prints build times cold and warm; record both in the entry.

#### 2. Guards (G 8.5)
Binder cost 0 → `line_breaks.rs` fails on the article rows; `placement.rs` reading `profile.watermark` → its D-171 test fails; the cache key without the size → `scene_cache.rs` fails. Undo all three.

### Error Handling

`hit` returns `None` when no chunk is visible and the point is outside the canvas; never panics on a point outside.

### Concurrency / Async Requirements

The cache is per thread (the render worker's one thread); no lock.

### Persistence Requirements

None.

### API Requirements

The binding gains `hit`.

### Security Requirements

None beyond the SAB.

### Testing Requirements

#### Unit Tests
`placement.rs` inline: the formula at the extremes; `captions_draw.rs` inline: the precedence order.

#### Integration Tests
`placement.rs`, `hit.rs`, `scene_cache.rs`, `parity.rs`, plus every scene test.

#### Failure-Path Tests
The three drills.

### Validation Commands

```bash
cargo test -p offcut-scene                                        # all pass (V3 expectations hold at placement 1.0)
cargo test -p offcut-scene --release --test scene_cache -- --nocapture   # cold and warm build times
git log --oneline -3 -- crates/offcut-scene/tests/line_breaks.rs  # table commit precedes readability.rs's
pnpm build:wasm && pnpm check && pnpm test                        # exit 0
```

### Acceptance Criteria

- [ ] Captions can be placed anywhere in the band; the event zone follows; at 1.0 it is V3's 860-1180.
- [ ] Emphasis follows the order active → KeywordPop → number → fill, with numbers only when `highlight_numbers`.
- [ ] `Scene::hit` and the binding's `hit` exist; `hit.rs` passes.
- [ ] The cache changes no output (cold, warm, off equal); parity at 540/720/1080 within 0.01 lp.
- [ ] The three guards fired and were undone; build times recorded; CI green (G M-8 complete with the founder's reading recorded in Prompt 103).

### Agent Instructions

Before: read D-169 to D-174, §8.6, §8.9, §8.10, §11.1, G 8.4-8.5, M-8. During: one file and its test at a time. After: SAB gate; commit `V4 P104 (S8): emphasis, placement, hit test, shaping cache, parity`; push; read CI.

### Expected Result

Caption intelligence is complete in the scene: phrase-aware, readable, emphasized, placeable, hit-testable, cached and size-independent.

---

## Prompt 105 (V4 15/38) — The premium default: Inter 800, the theme table, the sliding pill, Bold, Mono (S9 part 1)

**Effort: High.** The pill must slide (interpolated from the previous word's box over 90 ms) and never jump, never appear in a pause, and sit behind exactly one word; the shadow must not change the layout; every colour must stay in `theme.rs`.

### Objective

Give Clean its premium look (Inter 800 at key 4, 72 lp, shadow, the active word on a sliding highlight pill), Bold its pop, Mono (Tech) its panel and cursor, with every colour a theme role and every motion value data.

### Project State Before This Prompt

- Prompt 104 committed; `captions_draw.rs` draws V3's look with the emphasis order.

### Why This Comes Here

The look comes after the identity proof (D-116) and after caption intelligence, so that the goldens of Prompt 107 are written once on final values (G item 3).

### Scope

#### Implement

G 9.1 rows 1-6:

1. `fonts.rs` D-170: key 4 = Inter 800, appended; coordinates computed like the others.
2. `theme.rs` §8.4: the table of §8.4 for Clean, Bold, Tech (text, stroke, accent, accent_alt, panel at 85%, display/body/mono fonts; Clean's pill colours; the shadow 0 x 4 lp black 45% for Clean and Bold).
3. `motion.rs` D-137, D-170: `ActiveTreatment::Pill` appended; `ShadowSpec { offset, role, opacity }`; `enter_at`.
4. `styles.rs` D-170, D-139: Clean (Inter 800 via `Display`, 72 lp, as spoken, 16 x 2, 4 words, stroke 5 lp, shadow, active `Pill`, `Fade` 120 ms ease-out cubic); Bold (Inter 900, upper case, 84 lp, 12 x 2, 3 words, stroke 10 lp, `Pop` 1.0 → 1.12 → 1.0 over 140 ms back-out, active in `accent`); Tech (JetBrains Mono 700, 56 lp, 22 x 2, 5 words, no stroke, panel radius 24 lp padding 32 lp at 85%, `Cursor` 0.6 em, `None`, linear 80 ms); `StyleSpec.shadow`. A style with `MotionUnit::Glyph` is refused by `styles::spec`'s test.
5. `captions_draw.rs` §8.9: the pill (`FillRect` radius 14 lp, padding 12 x 4 lp, in `accent`, word in `#111114` via the theme; its rectangle at `t` interpolated over `PILL_SLIDE_MS = 90` from the previous word's box to the active word's, starting at the active word's start; scale 0.9 → 1.0 over the same time; none between words or in a pause); the shadow (the run drawn again before the stroke, offset, black at `opacity`); Bold's pop; Tech's panel and cursor; KeywordPop's scale 1.0 → 1.25 at 90 ms → 1.10 at 180 ms, held to the word's end.
6. `tests/styles_use_theme.rs` D-136: no `Rgba {` literal in `offcut-scene/src` outside `theme.rs` and `display_list.rs`.

#### Do Not Implement Yet

- The visuals, framing offset, watermark, summary and their snapshot tests (Prompt 106). Goldens (Prompt 107).

### Existing Components To Reuse

- `FontSet::embedded()`'s registration code; `easing.rs`; the anim tracks; `captions_draw.rs`'s word boxes.

### Files To Create

- `crates/offcut-scene/tests/styles_use_theme.rs`.

### Files To Modify

- `crates/offcut-scene/src/{fonts,theme,motion,styles,captions_draw}.rs`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- `theme.rs` is the only home of a colour value (D-136); `styles.rs` the only home of a style's numbers (Appendix D).
- The shadow and the pill change no layout box: placement, hit testing and the safe-area test see the same boxes (§8.9).
- Every value is (assumption, E-2), listed in Appendix D; none appears in two files.

### Data Types / Interfaces

§8.4, §8.5 plus D-170:

```rust
pub enum ActiveTreatment { Highlight, Cursor, Pill }
pub struct ShadowSpec { pub offset: (f32, f32), pub role: Role, pub opacity: f32 }
pub struct StyleSpec { …, pub shadow: Option<ShadowSpec>, pub motion: CaptionMotion }
```

### Implementation Requirements

#### 1. The pill's motion
At the active word's start the pill is at the previous word's box (or, for the chunk's first word, at the word's own box at scale 0.9); at +90 ms it is on the active word at scale 1.0. Eased with ease-out cubic.

#### 2. Snapshot impact
Existing inline tests assert V3's values in places; those that assert Clean's look are updated to D-170's values with a changelog line each (this is the redesign D-170 decides, not a silent change).

### Error Handling

None new.

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

None beyond the SAB.

### Testing Requirements

#### Unit Tests
`motion.rs` `enter_at` at 0, half, full; the pill's rectangle at the word start, +45 ms, +90 ms; no pill between two words separated by a pause.

#### Integration Tests
`styles_use_theme.rs`; every scene test.

#### Failure-Path Tests
Add `Rgba { r: 1.0, … }` to `captions_draw.rs` temporarily → `styles_use_theme.rs` fails; undo.

### Validation Commands

```bash
cargo test -p offcut-scene                                        # all pass
git grep -n "Rgba {" -- crates/offcut-scene/src | grep -vE "theme.rs|display_list.rs"   # no output
pnpm build:wasm && pnpm dev                                       # look: Clean's pill glides; Bold pops; Mono's panel (switch the default style temporarily, then revert)
```

### Acceptance Criteria

- [ ] Key 4 is Inter 800; the theme table of §8.4 is in `theme.rs`; `Pill` and `ShadowSpec` exist.
- [ ] Clean, Bold and Tech have D-170's and D-139's values; the pill slides and is absent in pauses (unit-tested).
- [ ] `styles_use_theme.rs` passes and its drill fired; every changed inline expectation is listed with D-170 as its reason; CI green.

### Agent Instructions

Before: read D-136 to D-139, D-170, §8.4, §8.5, §8.9 (the pill), Appendix D, G 9.1. During: values only from the decisions. After: SAB gate; commit `V4 P105 (S9): Inter 800, theme table, Clean with the sliding pill, Bold, Mono`; push; read CI.

### Expected Result

A creator who never opens the Style tab gets the premium default; the three looks are data on roles.

---

## Prompt 106 (V4 16/38) — The four visuals, the crop offset, the watermark, the summary, the scene's test suite (S9 part 2)

**Effort: High.** Three new visual layouts with exact sizes and timings, the FromTo mark through the time map, a watermark that must be proven never to meet a caption at three placements, and about 45 exact snapshots to read.

### Objective

Draw ListReveal, FromTo and KeywordPop; use the crop offset; draw the watermark when the profile asks; count what is drawn in the summary; and add the scene's snapshot, safe-area, framing, determinism and catalog-completeness tests, reading every snapshot.

### Project State Before This Prompt

- Prompt 105 committed; the three styles have their final values.

### Why This Comes Here

All four visuals and the watermark must exist before the goldens (Prompt 107) and before the Visuals tab (Prompt 116); the snapshot suite freezes the scene's output for every later change.

### Scope

#### Implement

G 9.2 rows 1-7:

1. `events/list_reveal.rs` D-140: header 56 lp (display font) when not empty; up to five items at 48 lp, `"{ordinal}. {text}"`, 16 lp apart, left at the zone's left + 40 lp; from six items every text shrinks by one factor to fit (minimum 32 lp; below it the extra items are not drawn and the event counts as drawn); each item fades in and rises 24 lp over 200 ms ease-out from `map.to_out(item.at)`; hold to the last item's end + 1,500 ms; fade 200 ms.
2. `events/from_to.rs` D-141, §8.7: two columns; value 96 lp auto-fit (minimum 56 lp, below it not drawn); bars `value / max(from, to)` of 180 lp, minimum 8%; the "from" column at the first anchor; the arrow (a filled path) and the "to" column (in `accent`, scale 1.15) at `ValidItem.marks[0]` through the map; no label row; a zero or negative value draws 8%; hold to `span.end + 1,500 ms`; fade 200 ms.
3. `events/keyword_pop.rs`, `events/mod.rs` §8.7: the four arms; KeywordPop hands its word and program start to the captions; nothing returns `Ok(None)` for a kind except an undrawable item.
4. `framing.rs` D-131: the offset used (V2's stand-in removed); `tests/framing.rs` (§8.10's cases; the inline "any offset" case now asserts the offset, D-160).
5. `watermark.rs` D-143: `WATERMARK_TEXT = "Made with Offcut"`, Inter 700 36 lp, white 60% with a 3 lp dark stroke, baseline 6 lp above the safe area's bottom, left at its left; drawn only when `profile.watermark` (INV-9); its band below the lowest caption position.
6. `summary.rs`, `lib.rs` D-144: `voice_cleaned: true`; `captions_emphasized` = drawn KeywordPops; `visual_moments` = drawn items of the other kinds; disabled or undrawable not counted. Item visibility per D-145.
7. Tests §8.10: `display_list_snapshots.rs` (each style x each kind at three instants, watermark on and off, a landscape crop at -1, 0, 1, a word edit that changes a number, a disabled item absent), `layout_safe_area.rs` (D-158: 31 labeled files x 3 styles every 33 ms at the three snap points; no box leaves the safe area; the watermark meets no caption), `determinism.rs` (six scenes twice, byte-equal; `frame_at` out of order equal), `catalog_complete.rs` (INV-23: catalog kinds = drawn kinds; zone names match `ZoneId`). Every new snapshot read with `cargo insta review` before it is accepted.

#### Do Not Implement Yet

- Goldens (Prompt 107). Any interface (Prompts 108 on).

### Existing Components To Reuse

- `number_reveal.rs`'s fit and fade code as the pattern; `format_quantity` through `EventParams` display strings (the scene parses nothing, D-121); `safe_area::zone`; `placement::event_zone`.

### Files To Create

- `crates/offcut-scene/src/events/{list_reveal,from_to,keyword_pop}.rs`, `crates/offcut-scene/src/{watermark,summary}.rs`; `crates/offcut-scene/tests/{display_list_snapshots,layout_safe_area,framing,determinism,catalog_complete}.rs`; `crates/offcut-scene/tests/snapshots/*.snap`.

### Files To Modify

- `crates/offcut-scene/src/{events/mod,framing,lib}.rs`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- `from_to.rs` never tokenizes (Appendix A); the second value's time comes from `marks`.
- `watermark.rs` reads nothing but the profile flag (INV-9).
- Shaping happens at layout, never in `draw` (V2's layout-once rule; Appendix A for `list_reveal.rs`).

### Data Types / Interfaces

`events::layout(item: &ValidItem, span: OutSpan, t: &Transcript, map: &TimeMap, style, theme, fonts) -> Result<Option<EventTrack>, SceneError>`; `draw(track, t: OutMs, out)`.

### Implementation Requirements

#### 1. Snapshot inputs
Fixed small transcripts per kind (the same inputs the goldens will use in Prompt 107), at 1080 wide.

#### 2. FromTo marks over the labeled set
A test (in `layout_safe_area.rs` or inline in `from_to.rs`) that every FromTo of the 31 labeled files gets one mark later than its first word (§8.7).

### Error Handling

An item whose program span is `None` → `SceneError::TimeMap` (cannot happen with the identity map; tested with a hand-made map in a unit test only).

### Concurrency / Async Requirements

None.

### Persistence Requirements

Snapshots committed and read.

### API Requirements

None.

### Security Requirements

Snapshot inputs are written for the tests or are the founder-owned labeled set.

### Testing Requirements

#### Unit Tests
Each visual's layout at the shrink and fit thresholds; summary counts with an item disabled.

#### Integration Tests
The five new test files and every earlier scene test.

#### Failure-Path Tests
A snapshot changed by hand fails (`INSTA_UPDATE=no`); restore.

### Validation Commands

```bash
cargo test -p offcut-scene                                        # pending snapshots on the first run
cargo insta review                                                # read each: kinds, styles, instants; accept
INSTA_UPDATE=no CI=true cargo test -p offcut-scene                # all pass, nothing pending
cargo test -p offcut-scene --test layout_safe_area -- --nocapture # 93 scenes x 3 placements, 0 problems
```

### Acceptance Criteria

- [ ] All four kinds draw as D-140, D-141 and §8.7 say; FromTo's second value appears at its mark.
- [ ] The crop offset moves the `Video.src`; `framing.rs` passes with the offset asserted.
- [ ] The watermark draws only when asked and never meets a caption at Top, Middle or Bottom over the labeled set.
- [ ] The summary counts drawn items only; `catalog_complete.rs` and `determinism.rs` pass.
- [ ] Every snapshot was read before acceptance; the count of snapshots is recorded; CI green.

### Agent Instructions

Before: read D-140 to D-145, D-158, D-160, §8.7, §8.8, §8.10, G 9.2. During: one visual at a time with its inline tests; snapshots last. After: SAB gate; commit `V4 P106 (S9): ListReveal, FromTo, KeywordPop, framing, watermark, summary, scene snapshots`; push; read CI.

### Expected Result

The scene draws everything V4 promises, and its output is frozen by read snapshots and property tests.

---

## Prompt 107 (V4 17/38) — The fourteen goldens, the parity golden, the look in motion (S9 part 3)

**Effort: High.** Sixteen images a person will look at, a parity case at SSIM ≥ 0.99, two adapters, and a look that must be checked moving, not only in stills.

### Objective

Write the remaining fourteen golden frames and the 1080-versus-540 parity case, look at every PNG against G 9.3's checklist, check the three styles and the visuals moving in `pnpm dev`, and close S9.

### Project State Before This Prompt

- Prompt 106 committed; the scene and its snapshots are final.

### Why This Comes Here

The goldens are written once, on final values (G item 3); the parity golden completes D-172's proof.

### Scope

#### Implement

- G 9.3: in `golden_frames.rs`, fourteen cases at 540 x 960 on the synthetic pattern: `<style>_<kind>.png` for 3 styles x 4 kinds (the frame inside each visual at a moment when a caption word is active, so Clean's pill shows), `clean_number_watermark.png`, `clean_number_nowatermark.png`; the parity case (three styles at 1080 box-filtered to 540 against 540, SSIM ≥ 0.99). One case renders a portrait scene with `ok_rotated_90.mov`'s display size (720 x 1280) to check rotation handling (§18).
- `OFFCUT_GOLDEN_UPDATE=1` once on D1; then compare; push and read lavapipe's results (17 passed, or per-adapter folders as in Prompt 101).
- **Look at all 16 PNGs** with G 9.3's checklist: the pill behind exactly one word and clear of the next word's stroke; the shadow below the text; nothing clipped by the safe area; Bold's upper case fits; Mono's panel has even padding; the list's items align; the from-to's bars start on one baseline; the watermark legible and away from the captions. A golden that looks wrong is fixed in code, then rewritten.
- G 9.4: `pnpm build:wasm && pnpm dev`, the sample clip, each style (temporarily change `empty().edit.style` in `clip-store.ts`, reload, revert before the commit); G 9.4's four checks.
- G M-9: `pnpm e2e:media` (the D-160 rewrites only; no expectation on looks).

#### Do Not Implement Yet

- Any interface work (Prompts 108 on).

### Existing Components To Reuse

- Prompt 101's harness, pattern and SSIM; Prompt 106's snapshot inputs.

### Files To Create

- `fixtures/golden/` the fourteen PNGs (or two adapter folders).

### Files To Modify

- `crates/offcut-render/tests/golden_frames.rs`, `docs/v4/experiments.md` (golden similarity), `docs/v4/v4changelog.md`.

### Architecture Requirements

- `GOLDEN_SSIM_MIN = 0.98` for the goldens and 0.99 for parity; never lowered (§21.1).
- The box filter for parity is written in the test file (2 x 2 mean).

### Data Types / Interfaces

None new.

### Implementation Requirements

#### 1. What to record
Per golden: name, the D1 SSIM against itself on rerun (1.0), the lavapipe SSIM from the CI log; what you looked for and saw.

#### 2. The style default
`git diff web/src/state/clip-store.ts` shows nothing at the commit.

### Error Handling

None new.

### Concurrency / Async Requirements

None.

### Persistence Requirements

Sixteen PNGs committed (or 32 in adapter folders).

### API Requirements

None.

### Security Requirements

Synthetic sources only (D-135).

### Testing Requirements

#### Unit Tests
None new.

#### Integration Tests
`golden_frames.rs`: 17 passed (16 + parity) on D1 and lavapipe.

#### Failure-Path Tests
None new (Prompt 125 drills a darkened golden in CI).

### Validation Commands

```bash
OFFCUT_GOLDEN_UPDATE=1 cargo test -p offcut-render --test golden_frames -- --nocapture   # 14 written
cargo test -p offcut-render --test golden_frames                 # 17 passed
git ls-files fixtures/golden | wc -l                              # 16 (or 32)
pnpm e2e:media 2>&1 | tail -3                                     # green (keyed)
gh run view --log --job <ci job id> | grep -E "golden_frames|ssim"   # 17 passed on llvmpipe
```

### Acceptance Criteria

- [ ] Sixteen goldens and the parity case pass on both adapters (or per-adapter folders with the reason).
- [ ] Every PNG was looked at against G 9.3's checklist, with the observations recorded.
- [ ] G 9.4's four checks pass in `pnpm dev`; `clip-store.ts` unchanged at the commit.
- [ ] Media suite green; CI green (G M-9).

### Agent Instructions

Before: read D-135, D-170, D-172, §9.4, G 9.3-9.4, M-9. During: fix in code, never by rewriting a golden to hide a problem. After: SAB gate plus the media run; commit `V4 P107 (S9): fourteen goldens and the parity case; the look checked moving`; push; read CI.

### Expected Result

The look is final, proven at two sizes and on two adapters, and guarded by sixteen images.

---

## Prompt 108 (V4 18/38) — The design system: tokens, the interface font, primitives, icons (S10 part 1)

**Effort: High.** Twelve accessible primitives (focus, roving tab index, radio semantics, live regions) and a token set every later component depends on; the font must come out as a hashed asset or production serves HTML in its place.

### Objective

Build the editor's design system: the dark tokens of D-179 scoped to the editor, Inter as a self-hosted hashed interface font, the twelve primitives of D-181 with every state, the Lucide icon set, and the licence file.

### Project State Before This Prompt

- Prompt 107 committed (or, on the parallel track, Prompt 92). `~/inter-latin.woff2` exists (Prompt 91).
- `web/src/ui/styles/tokens.css` holds the landing page's variables and a first comment saying "system fonts only".

### Why This Comes Here

D-191: the shell is reviewed before any editing flow depends on it, and the shell is made of these parts. A frontend design skill, if the agent has one, is loaded now.

### Scope

#### Implement

- G 10.1: `tokens.css` [ONLY design values]: the landing page's variables kept; the editor's tokens under `[data-theme="dark"]` (colour, type 12/13/14/16/20/24/32 px and weights 400-700 with `--lh-tight 1.2`, `--lh-body 1.5`, space 4-48, radii 6/10/16, the two elevations, motion `--fast 120ms`, `--base 200ms`, `--ease cubic-bezier(.2,.8,.2,1)` with the reduced-motion override, `--focus`, layout `--topbar-h 56px`, `--panel-w 360px`, `--panel-w-sm 320px`), values exactly as G 10.1's table; the first comment corrected (D-180).
- G 10.2: `web/src/ui/assets/inter-latin.woff2` copied from `~`; `@font-face` "Inter UI" in `tokens.css` (`url("../assets/inter-latin.woff2")`, weight 100 900, `font-display: swap`); `--font-ui`; `web/LICENSES.md` with Inter (OFL-1.1 and its copyright line from the font) and Lucide (ISC, the pinned version).
- G 10.3: `web/src/ui/primitives/` one file each, styles in `ui/styles/primitives.module.css`: `Button` (primary/secondary/ghost/danger; sm/md; `busy` spinner after 300 ms keeping the width), `IconButton` (`aria-label`, tooltip with `Kbd`), `Tooltip` (400 ms hover, immediate on focus; `role="tooltip"`, `aria-describedby`), `Tabs` (`role="tablist"`, roving `tabindex`, ← → Home End, Enter/Space), `Switch` (`role="switch"`, `aria-checked`, whole row clickable), `SegmentedControl` (radio-group semantics, arrows), `Slider` (native range, `onCommit` on `change`, live value), `Toast` (bottom centre, 5 s, paused on hover and focus, announced), `Spinner`, `Skeleton` (shimmer off under reduced motion), `PanelSection` (`--fs-13` `--fw-600` `--text-2` upper-case tracking, `--sp-4` padding), `Kbd` (`⌘` on macOS via `navigator.userAgentData.platform`, else `Ctrl`). Each: focus ring from `--focus`, hover, pressed, disabled with a reason as tooltip; text from props only.
- G 10.4: `web/src/ui/icons.tsx`, one component per icon of G 10.4's list (22), `viewBox 0 0 24 24`, `stroke="currentColor"`, `stroke-width 1.75`, `fill="none"`, 20 px default, `aria-hidden="true"`; paths from Lucide at the pinned version.

#### Do Not Implement Yet

- The layout, state cards, copy keys and `ui-store` (Prompt 109). Any component that calls a use-case.

### Existing Components To Reuse

- The landing page's tokens (kept, unchanged); `messages.ts` for every label a caller passes; the existing `react/jsx-no-literals` lint.

### Files To Create

- `web/src/ui/assets/inter-latin.woff2`, `web/LICENSES.md`, `web/src/ui/primitives/{Button,IconButton,Tooltip,Tabs,Switch,SegmentedControl,Slider,Toast,Spinner,Skeleton,PanelSection,Kbd}.tsx`, `web/src/ui/styles/primitives.module.css`, `web/src/ui/icons.tsx`.

### Files To Modify

- `web/src/ui/styles/tokens.css`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- No component library, no CDN font (TS §36); the font is served from our origin under `/assets/` with a hash (D-180, G item 9); `web/vercel.json` untouched.
- No colour literal outside `tokens.css` in any `web/src/ui/**/*.css`.
- Hit targets at least 32 x 32 px; AA contrast (D-179); reduced motion respected (D-179, §17.4).
- Primitives hold no product logic and render no text not passed in (Appendix A).

### Data Types / Interfaces

Props as G 10.3's table; each primitive exports one component and its props type.

### Implementation Requirements

#### 1. Font check
`pnpm build` emits exactly one `inter-latin-<hash>.woff2` under `web/dist/assets/`; `git grep -n "public/fonts" -- web` prints nothing.

#### 2. A temporary gallery
A `zz-` dev-only route or Storybook-free page to look at each primitive in every state at both widths, deleted before the gate.

### Error Handling

None (presentational).

### Concurrency / Async Requirements

`Button`'s 300 ms busy delay and `Tooltip`'s 400 ms delay use one timer each, cleared on unmount.

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

CSP holds: no inline style, `font-src 'self'`.

### Testing Requirements

#### Unit Tests
None required by §20.3; keyboard behaviour is proven in Prompt 123's axe and keyboard case.

#### Integration Tests
`pnpm check` (jsx-no-literals, boundaries), `pnpm build`.

#### Failure-Path Tests
A hex colour added to `primitives.module.css` temporarily → the G 10.1 grep prints it; undo.

### Validation Commands

```bash
grep -cE '^\s+--[a-z0-9-]+:' web/src/ui/styles/tokens.css        # record the count
git grep -nE '#[0-9a-fA-F]{3,8}\b' -- 'web/src/ui/**/*.css' ':!web/src/ui/styles/tokens.css'   # no output
pnpm build && ls web/dist/assets | grep -E '^inter-latin-.*\.woff2$'   # one hashed file
git grep -n "public/fonts" -- web                                 # no output
```

### Acceptance Criteria

- [ ] The tokens of D-179 exist under `[data-theme="dark"]`; the landing page is unchanged (`pnpm e2e` 12).
- [ ] The font is one hashed `/assets/` file; `web/LICENSES.md` names Inter and Lucide with versions.
- [ ] Twelve primitives with focus, hover, pressed and disabled states and the keyboard behaviour of G 10.3; 22 icons.
- [ ] No colour literal outside `tokens.css`; the gallery deleted; CI green.

### Agent Instructions

Before: read D-177 to D-181, D-192, §17.1, §17.6, G 10.1-10.4, G item 9; load a design skill if available. During: tokens first, then primitives, then icons. After: SAB gate; commit `V4 P108 (S10): tokens, interface font, primitives, icons`; push; read CI.

### Expected Result

A small owned design system from which every editor screen will be built.

---

## Prompt 109 (V4 19/38) — The editor shell: layout, stage, side panel, state cards, every copy key (S10 part 2)

**Effort: High.** Every state of §17.2 must render in one layout that never jumps, with the old suites' selectors kept, and every word must already be the final copy so the founder reviews real words.

### Objective

Build `EditorLayout` (top bar, stage, side panel), `ui-store`, the stage's state cards (empty, model download, processing, cancelled, rejected, error, ready with the player), the four tabs with static placeholders where flows do not exist yet, and every key of §17.5 in `messages.ts`.

### Project State Before This Prompt

- Prompt 108 committed; primitives and icons exist. Prompt 107 committed (the look is final).

### Why This Comes Here

D-191's first review looks at these screens; G (c): the copy keys are written now so the review reads real words.

### Scope

#### Implement

- G 10.5 rows 1-6: `ui/layout/EditorLayout.tsx` + `ui/styles/editor.module.css` (grid: top bar row `--topbar-h`; stage `1fr` and panel `--panel-w`, `--panel-w-sm` below 1280 px; viewport height; the panel scrolls alone; `data-theme="dark"` on the root; the one `aria-live="polite"` region); `ui/layout/TopBar.tsx` (mark, clip length, static save state "Saved on this device", disabled undo/redo `IconButton`s, primary Export wired to the existing export logic); `ui/layout/Stage.tsx` (a 9:16 frame, `aspect-ratio: 9 / 16`, `max-height: min(960px, 100% - space)`, `--r-panel`, `--shadow-2`, on `--bg`, holding the state's card or `PreviewPlayer`); `ui/layout/SidePanel.tsx` + `ui/styles/panel.module.css` (`Tabs`: Captions, Style, Visuals, Framing; disabled with a `Skeleton` while processing; tab in `ui-store`); `state/ui-store.ts` §16.1 (tab, follow, editing word, toasts, save state); `ui/pages/EditorPage.tsx` renders `EditorLayout` for every clip state.
- G 10.6 rows 1-6: `DropZone` restyled as the empty stage; `ProcessingCard.tsx` (new) with `ProcessingFeed` chips (D-188: four steps, `Check`, `Spinner`, muted rest; the reassurance line; Cancel `ghost`, wired in Prompt 119); `ModelDownloadPanel` restyled (size, progress on `--brand`, "only the first time", Cancel); `RejectionPanel` restyled (V3's copy and `data-reject` unchanged); `ErrorPanel.tsx` (new: title, one sentence, one action, "Start over"; D-190); `PreviewPlayer` (canvas fills the frame; transport under it: play/pause `IconButton`, `--fs-13` tabular time, a placeholder scrub bar; the watermark badge when not on Creator).
- G 10.7: every key of §17.5 added to `messages.ts` in D-177's tone (including the ones later prompts use, the cancelled card's "Cancelled.", "Process again", "Start over", the export pill's stage words, `visuals.whatWeLookFor`, the shortcut keys); `node scripts/check-copy-codes.mjs`.

#### Do Not Implement Yet

- Any flow: transcript editing, style cards, toggles, framing controls (Prompts 113-117); cancel wiring (Prompt 119); the 14 error copies (Prompt 119 moves them out of `COPY_PENDING`; the keys may exist now with their final text); `screens.spec.ts` and dev hooks (Prompt 110).

### Existing Components To Reuse

- `ExportButton` logic, `ProcessingFeed`'s feed lines (V3), `RejectionPanel`'s content, `DropZone`'s file handling (P-11: the name of a dropped file is never read), the clip store's states, the primitives and icons.

### Files To Create

- `web/src/ui/layout/{EditorLayout,TopBar,Stage,SidePanel}.tsx`, `web/src/ui/styles/{editor,panel}.module.css`, `web/src/state/ui-store.ts`, `web/src/ui/components/{ProcessingCard,ErrorPanel}.tsx`.

### Files To Modify

- `web/src/ui/pages/EditorPage.tsx`, `web/src/ui/components/{DropZone,ProcessingFeed,ModelDownloadPanel,RejectionPanel,PreviewPlayer}.tsx`, `web/src/ui/styles/components.module.css`, `web/src/copy/messages.ts`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- Every `data-testid` the V2/V3 suites use is kept (G M-10: the media suites still find their elements).
- The 9:16 frame does not move between processing and ready (D-178, G 10.9).
- No jargon anywhere (D-177's list); copy only from `messages.ts` (INV-15).

### Data Types / Interfaces

```ts
// state/ui-store.ts
type Tab = "captions" | "style" | "visuals" | "framing";
interface UiState { tab: Tab; follow: boolean; editingWord: WordIdx | null; toasts: ToastItem[]; save: "saved" | "saving";
  selectTab(t: Tab): void; setFollow(on: boolean): void; editWord(w: WordIdx | null): void; pushToast(t: ToastItem): void; dismissToast(id: string): void; setSave(s: "saved" | "saving"): void; }
```

### Implementation Requirements

#### 1. Every state reachable in `pnpm dev`
Empty, model download (fresh profile), processing, cancelled (static until Prompt 119), ready, rejected (a rejection fixture), error (static until Prompt 110's hook), exporting (the existing export).

#### 2. Shell size
`node scripts/check-hosts.mjs`: record the shell size against Entry 1 (D-180's budget is read in Prompt 126).

### Error Handling

`ErrorPanel` shows copy only, never a code (D-190); its actions are wired in Prompt 119.

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

P-11 holds in the restyled `DropZone`.

### Testing Requirements

#### Unit Tests
`messages.test.ts` (existing) passes with the new keys.

#### Integration Tests
`pnpm e2e` 12 (landing unchanged); `pnpm e2e:media` green (selectors kept).

#### Failure-Path Tests
None new.

### Validation Commands

```bash
pnpm check                                                        # exit 0 (jsx-no-literals, copy codes, boundaries)
node scripts/check-hosts.mjs 2>&1 | tail -3                       # record the shell size
pnpm e2e 2>&1 | tail -2                                           # 12 passed
pnpm e2e:media 2>&1 | tail -3                                     # green (keyed)
```

### Acceptance Criteria

- [ ] The editor is one dark layout (top bar, stage, side panel with four tabs) for every clip state; the frame never jumps.
- [ ] The state cards of §17.2 exist and look finished; V3's rejection copy and `data-reject` unchanged.
- [ ] Every key of §17.5 is in `messages.ts` in D-177's tone; copy check green.
- [ ] The media suites find their elements; CI green.

### Agent Instructions

Before: read D-177, D-178, D-188 to D-190, §17.2-§17.5, G 10.5-10.7; load a design skill if available. During: build from tokens and primitives only. After: SAB gate plus the media run; commit `V4 P109 (S10): editor layout, state cards, copy`; push; read CI.

### Expected Result

The editor's shell shows every state in real words, ready to be captured and reviewed.

---

## Prompt 110 (V4 20/38) — The screen capture tool, dev-only hooks, self-review (S10 part 3)

**Effort: Medium.** A Playwright project of its own and three dev-only hooks that production must strip; the self-review list is mechanical.

### Objective

Write `screens.spec.ts` and its `screens` project and script, the dev-only `__offcutDev` hooks it needs, capture sixteen screenshots (eight states at 1280 x 800 and 1920 x 1080), and fix everything on G 10.9's self-review list before the founder sees them.

### Project State Before This Prompt

- Prompt 109 committed; every state renders in the shell.

### Why This Comes Here

Design review 1 (Prompt 111) reviews these captures; the hooks reach states whose flows do not exist yet (G (b) 6).

### Scope

#### Implement

- G 10.8: `web/tests-e2e/screens.spec.ts` (D1 only); `web/playwright.config.ts` project `screens` (`testMatch: "screens.spec.ts"`, `dependencies: ["media-setup"]`, `use: MEDIA_USE`, one worker); root `package.json` script `"screens": "pnpm --filter web exec playwright test --project=screens"`. For each state of G 10.8's table and both viewports: reach it, `await document.fonts.ready`, wait for animations to settle, `page.screenshot({ path: "test-results/screens/<state>-<w>x<h>.png" })`.
- The page's dev hooks in one new file, `web/src/ui/dev-hooks.ts` (the `ui` layer may read stores and call use-cases, as components do), its TS §5 line added in the same commit ("Gaps settled" item 5): `__offcutDev.openWordField(i)`, `__offcutDev.failClip(code)`, `__offcutDev.fakeExport(fraction)` now; Prompts 119 and 124 add `crash(kind)`, `failNextSetScene()` and `hangWorker(kind)` to the same file (their worker halves live in the worker files behind the same condition). `EditorPage.tsx` installs them only when `import.meta.env.MODE !== "production"`, so Vite's dead-code elimination removes the module from `web/dist` (D-155's rule).
- G 10.9: open the sixteen captures and fix: off-grid spacing, two greys for one role, text under AA (`--text-2` on `--surface-2` is the floor), a control missing a state, anything brighter than `--text` or the brand colour beyond the one primary action and the selected tab, jargon, layout shift between processing and ready.
- G M-10.

#### Do Not Implement Yet

- The production check for the hooks' absence (Prompt 124, G 16.6), though `grep` on a local `pnpm build` is run now as a preview.

### Existing Components To Reuse

- `MEDIA_USE`, the `media-setup` project, the cached model; the fixtures' portrait clip.

### Files To Create

- `web/tests-e2e/screens.spec.ts`, `web/src/ui/dev-hooks.ts`.

### Files To Modify

- `web/playwright.config.ts`, `package.json`, `web/src/ui/pages/EditorPage.tsx`, `docs/technicalspec.md` (§5: `dev-hooks.ts`), any component the self-review fixes, `docs/v4/v4changelog.md`.

### Architecture Requirements

- CI never runs `screens` (CI names its projects: `non-media`, `media`; G item 8, quick reference).
- The screenshots stay in `web/test-results/` (git-ignored; SAB rule 9).
- The hooks are absent from `web/dist` (checked now locally, proven in Prompt 124).

### Data Types / Interfaces

```ts
declare global { interface Window { __offcutDev?: { openWordField(i: number): void; failClip(code: ErrorCode): void; fakeExport(f: number): void } } }
```

### Implementation Requirements

#### 1. The eight states (G 10.8)
empty (`/app`); model download (fresh context, model absent, at about 30%); processing (portrait fixture, after step two); ready; editing a word (static field via the hook); style tab (static cards); error (`failClip("E_GPU_LOST")`); exporting (`fakeExport(0.42)`).

#### 2. Self-review record
A list in the entry: each finding and its fix.

### Error Handling

None.

### Concurrency / Async Requirements

The spec runs with one worker.

### Persistence Requirements

None committed.

### API Requirements

None.

### Security Requirements

Screenshots never committed or uploaded; the portrait fixture is synthetic.

### Testing Requirements

#### Unit Tests
None.

#### Integration Tests
`pnpm screens` writes 16 PNGs.

#### Failure-Path Tests
`pnpm --filter web exec playwright test --list` without `--project` lists `screens.spec.ts` only under the `screens` project.

### Validation Commands

```bash
pnpm screens && ls web/test-results/screens | wc -l              # 16
pnpm build && grep -rl "__offcutDev" web/dist && echo LEAK || echo clean   # clean
pnpm e2e 2>&1 | tail -2                                           # 12 passed
pnpm e2e:media 2>&1 | tail -3                                     # green (keyed; screens not included)
```

### Acceptance Criteria

- [ ] `pnpm screens` writes sixteen captures of the eight states at both widths.
- [ ] The hooks exist in dev builds only; a production build is clean.
- [ ] Every G 10.9 finding is fixed and listed; G M-10's commands pass; CI green and not running `screens`.

### Agent Instructions

Before: read D-155, D-191, G 10.8-10.9, M-10. During: capture, look, fix, capture again. After: SAB gate; commit `V4 P110 (S10): screen capture tool, dev hooks, self-review fixes`; push; read CI.

### Expected Result

Sixteen screenshots of a finished-looking shell, ready for the founder.

---

## Prompt 111 (V4 21/38) — Design review 1 (S10A, human gate)

**Effort: Medium.** Preparation and record-keeping around the founder's review; **switch to High** if the founder asks for a structural change (layout, tab set, a new state) rather than tokens and spacing.

### Objective

Prepare the sixteen captures and a one-page note, hold the review with the founder (four questions per screen, the brand colour, the name "Mono"), make every requested change through the tokens and primitives where possible, capture again, get the founder's "go", and record it.

### Project State Before This Prompt

- Prompt 110 committed; `pnpm screens` writes sixteen captures; the self-review is done.

### Why This Comes Here

D-191: no review flow, drag, transcript editor or style card is wired before the founder has approved the shell they will live in (G item 8, §5).

### Scope

#### Implement

- G 11.1: fresh captures from the committed build; copy them to `~/v4-review-1/` (outside the repository) named `01-empty-1280.png` … in the order of §17.2; write `~/v4-review-1/NOTE.md`: per screen, what it is for, what the user does next, what is not wired yet.
- G 11.2 (**Human**): present the images (or `pnpm dev` at both window sizes) and ask, per screen: (1) premium and calm, with the video as the hero? (2) obvious what to do next without reading? (3) every word plain enough for a creator who is not technical? (4) anything off (spacing, alignment, colour, size)? Plus two decisions: keep `--brand` (D-179's violet) or another; keep the name "Mono". Stop and wait for the answers.
- G 11.3: make every requested change, through `tokens.css` when the change should be global and through a primitive when it concerns a control; capture again into `~/v4-review-1/after/`; show only the changed screens; get a "go".
- The changelog entry "Design review 1: go on <date>" with the list of changes asked and made, the brand colour decision, the Mono decision, and the founder's words.

#### Do Not Implement Yet

- Any flow (Prompts 112 on). A requested change that needs a flow (for example "the transcript should look like X when editing") is recorded and applied in the prompt that builds that flow.

### Existing Components To Reuse

- `pnpm screens`, the tokens and primitives.

### Files To Create

- Outside the repository only: `~/v4-review-1/*`.

### Files To Modify

- Whatever the review changes (`tokens.css`, primitives, layout, cards, `messages.ts`), `docs/v4/v4changelog.md`.

### Architecture Requirements

- A change that would be global goes into a token, never repeated in components (G 11.3).
- A renamed style ("Mono") changes copy only; `StyleId::Tech` stays (D-170).

### Data Types / Interfaces

None.

### Implementation Requirements

#### 1. Recording the founder's words
Quote the answers as given; summarise nothing away. Questions the founder skipped are written as "not answered".

#### 2. Machine checks after the changes
G M-10's commands again.

### Error Handling

If the founder does not say "go", the next prompt does not start; the open requests are listed.

### Concurrency / Async Requirements

None.

### Persistence Requirements

Screenshots stay outside the repository.

### API Requirements

None.

### Security Requirements

Screenshots of the fixtures only; never uploaded.

### Testing Requirements

#### Unit Tests
None.

#### Integration Tests
G M-10's commands after the changes.

#### Failure-Path Tests
None.

### Validation Commands

```bash
ls ~/v4-review-1 | wc -l                                          # 17 (16 captures + NOTE.md)
pnpm check && pnpm e2e 2>&1 | tail -2                             # exit 0; 12 passed
pnpm screens && ls web/test-results/screens | wc -l              # 16 (after the changes)
```

### Acceptance Criteria

- [ ] The sixteen captures and the note were prepared outside the repository.
- [ ] **Human:** the founder answered the four questions per screen and the two decisions.
- [ ] Every requested change is in the code or recorded for the flow prompt that will apply it; changed screens recaptured.
- [ ] **Human:** the founder said "go"; the changelog entry records it with the changes and decisions; CI green.

### Agent Instructions

Before: read D-177 to D-181, D-191, §17.1, G Phase 11. During: stop after presenting; resume with the founder's answers. After: SAB gate; commit `V4 P111 (S10A): design review 1 changes` (or, if nothing changed, commit the changelog entry alone with that title); push; read CI.

### Expected Result

The founder has approved the editor's look on real screens before any flow depends on it.

---

## Prompt 112 (V4 22/38) — Seek, prefetch, the resize rule, preview statistics, the redraw while paused (S11 part 1)

**Effort: High.** Decoder restarts at keyframes, one-in-flight seeking where the latest wins, an adaptive downscale measured over 60 frames, and frame lifetimes that must all close (INV-11).

### Objective

Make the preview scrubbable and observable: `previewSeek`, `prefetch`, the keyframe restart on a far seek, the 360 x 640 downscale rule, the statistics written as `data-*` attributes, the scrub bar, the seek events of the preview machine, and the redraw of the current frame after `setScene` while paused.

### Project State Before This Prompt

- Prompt 111 committed with the founder's "go".
- `video-source.ts` has a `prefetch` stub; `frame-decoder.ts` can restart at a keyframe index; `preview-job.ts` holds play, pause and clock.

### Why This Comes Here

Direct manipulation (Prompt 113) and every edit (Prompts 114-117) need a preview that shows the frame at a position while paused (D-173's redraw) and a position to edit at.

### Scope

#### Implement

- G 12.1 rows 1-6:
  1. `render/frame-decoder.ts`, `render/video-source.ts` §15.3: `prefetch(from)` (decode from the keyframe at or before `from` without handing a frame out); a seek outside the queue (before its head, or past its horizon by more than 1 s) flushes and restarts through `restartAt(keyframeIndex)`.
  2. `render/preview-job.ts`, `render/preview-loop.ts` §15.2, D-149: `previewSeek` (stop a running loop, `frameAtBlocking(srcOf(at))`, draw, answer); the 60-frame mean cost and the one-time `session.resize(360, 640)` with the scene set again at the preview profile scaled by 2/3 when the mean exceeds `DOWNSCALE_MEAN_MS = 28`; `downscaled` in the statistics; the page's canvas keeps 540 x 960 CSS pixels.
  3. `crates/offcut-wasm-render/src/session.rs` §11.1: `resize(width, height)` calling `Renderer::resize`; `load-render.ts`'s `RenderSession.resize`.
  4. `usecases/control-preview.ts` §16.2: `seek(at: OutMs)` with one in flight and the latest winning; sound restarts at `at` when it was playing; `lastStats` stored after each play.
  5. `state/preview-store.ts` §16.1: `position: OutMs` (each clock sync and seek), `lastStats`, `seekStart()`, `seekDone(playing)`: the three seek events of V2's table fire now.
  6. `PreviewPlayer.tsx` D-148, D-155: the scrub bar (`Slider`, 0 to the program duration, step one frame, live position at 4 Hz while playing; pointer-down → `seekStart`, input → `seek(at)`, pointer-up → `seekDone`), the spinner after `SEEK_SPINNER_MS = 300`, the `data-frames`, `data-late`, `data-max-drift-ms`, `data-downscaled` attributes on its root.
- G 12.2: in `render.worker.ts`'s `setScene`, when a preview canvas is attached and the loop is not running, draw the frame at the last position (kept by `preview-job.ts`) after the scene is set (D-173).
- Fill the V3 markers this touches (`video-source.ts` `prefetch`, `preview-store.ts` seek events): remove each `V4` marker as its code lands.

#### Do Not Implement Yet

- `hitTest`, gestures, drags (Prompt 113). Edit flows (Prompts 114-117). The dev-only caption mirrors (Prompt 113).

### Existing Components To Reuse

- `frameAtBlocking` (TS §20.1), the audio clock master, `ClockSync` every 250 ms, the generation counter of `frame-decoder.ts`, the preview machine's declared events.

### Files To Create

None.

### Files To Modify

- `web/src/workers/render/{frame-decoder,video-source,preview-job,preview-loop}.ts`, `web/src/workers/render.worker.ts`, `crates/offcut-wasm-render/src/session.rs`, `web/src/wasm/load-render.ts`, `web/src/usecases/control-preview.ts`, `web/src/state/preview-store.ts`, `web/src/ui/components/PreviewPlayer.tsx`, `web/src/ui/styles/preview.module.css` (new module, already in the tree), `docs/v4/v4changelog.md`.

### Architecture Requirements

- Every `VideoFrame` handed out is closed once (INV-11): a flushed queue closes its frames; the shown frame is closed when replaced.
- The preview reads program time from the audio clock, never the wall clock for time on the clip (Appendix A for `preview-job.ts`).
- The downscale is never undone for that clip (D-149).
- The statistics reach the page, never the network (D-155).

### Data Types / Interfaces

```ts
previewSeek(p: { at: OutMs }): Promise<void>;
previewPlay(p: { clock: ClockSync }): Promise<PreviewStats>;   // { frames, late, maxDriftMs, downscaled }
```

```rust
#[wasm_bindgen] impl RenderSession { pub fn resize(&mut self, width: u32, height: u32) -> Result<(), JsValue>; }
```

### Implementation Requirements

#### 1. Seek in flight
`control-preview.seek` holds at most one `previewSeek` in flight and one pending target; when the in-flight one answers, the pending one (the latest) is sent.

#### 2. Look at it (G 12.4 rows 1 and 6)
Scrub a 60 s clip while paused: the frame follows, at most a brief spinner on a far jump; after a 20 s play `data-max-drift-ms` is present and small.

### Error Handling

A decode failure during a seek follows V3's `E_DECODE_VIDEO` path; the seek promise rejects and the spinner hides.

### Concurrency / Async Requirements

`previewSeek` stops a running loop before drawing; `previewPlay` after a seek starts from the seeked position; no two loops run.

### Persistence Requirements

None.

### API Requirements

The binding gains `resize`.

### Security Requirements

None beyond the SAB.

### Testing Requirements

#### Unit Tests
None required (Prompt 121's `run-pipeline.test.ts` and Prompt 122's `preview-sync.spec.ts` cover it).

#### Integration Tests
`pnpm e2e:media` green.

#### Failure-Path Tests
A seek during a seek: only the latest frame is drawn (checked in dev with the Performance panel).

### Validation Commands

```bash
pnpm build:wasm && pnpm check && pnpm test                        # exit 0
git grep -nE "\bV4\b" -- web/src/workers/render/video-source.ts web/src/state/preview-store.ts   # no output
pnpm e2e:media 2>&1 | tail -3                                     # green (keyed)
```

### Acceptance Criteria

- [ ] Scrubbing while paused shows the frame of the position (within 300 ms on a far jump on D1).
- [ ] `prefetch` and the keyframe restart exist; the resize rule runs once when the mean cost exceeds 28 ms; `downscaled` reported.
- [ ] The statistics are on `PreviewPlayer`'s root as `data-*`; the seek events fire.
- [ ] A `setScene` while paused redraws the current frame at once.
- [ ] The two V3 markers are filled; media suite green; CI green.

### Agent Instructions

Before: read D-148, D-149, D-155, D-173, §15.2, §15.3, §16.1-§16.2, G 12.1-12.2. During: worker side first, then use-case, store, component. After: SAB gate plus the media run; commit `V4 P112 (S11): seek, prefetch, resize rule, preview statistics, redraw while paused`; push; read CI.

### Expected Result

A preview that scrubs, adapts to a slow machine, reports its drift, and shows a change while paused.

---

## Prompt 113 (V4 23/38) — Hit testing and direct manipulation on the preview (S11 part 2)

**Effort: High.** Pointer handling over a canvas owned by a worker: hover throttled to one hit test per animation frame, snaps within 16 px, drags that commit once on release, chrome that must never be drawn into a frame, and keyboard equivalents.

### Objective

Make the preview directly editable while paused: the cursor says what a press will do, a click on a caption word opens it in the Captions tab, the caption block drags up and down with snaps at Top, Middle and Bottom, the video drags sideways on landscape and square clips, each drag committing once on release through a minimal `review-update.ts`.

### Project State Before This Prompt

- Prompt 112 committed; the preview seeks and redraws while paused; `Scene::hit` and the binding's `hit` exist (Prompt 104).

### Why This Comes Here

D-183: direct manipulation is the first way to edit; it needs the paused redraw (Prompt 112) and is the first caller of the update rule, written minimally here and completed in Prompt 114 (G (b) 7).

### Scope

#### Implement

G 12.3 rows 1-5 and G 12.4:

1. `workers/protocol.ts`, `render.worker.ts` §14.1, §15.1: `hitTest({ at, x, y }) → Hit`, answered only while the loop is not running (else `{ kind: "video" }` without work). **REOPENED V1 CONTRACT** (addition).
2. `usecases/preview-interact.ts` §16.2: `hoverAt(x, y)` (one per animation frame; only when paused), `clickAt(x, y)` (a word: pause, select the Captions tab, set `ui-store.editingWord`), `dragCaptions(y)`, `dragVideo(x)` (live guide only), the snap rule (`SNAP_DISTANCE_PX = 16` of Top 0.15, Middle 0.55, Bottom 1.0).
3. `usecases/set-caption-position.ts`, `usecases/set-crop-offset.ts` §16.2: commit on release through a **minimal** `usecases/review-update.ts` (mode `scene`: `editStarted`, `setScene`, `editsRepo.put`, `editDone`, `review_action`), completed in Prompt 114; their two `eslint.config.js` edges (`set-caption-position.ts`, `set-crop-offset.ts` → `review-update.ts`; `review-update.ts` → `control-preview.ts`) added now with reasons. **REOPENED V2 CONTRACT (use-case edges)**.
4. `PreviewPlayer.tsx` + `preview.module.css` §17.4, D-183: a transparent pointer layer over the canvas; cursor by hit (`text`, `ns-resize`, `ew-resize`, `default`); the 2 px word outline, the snap guide line and the 9:16 frame outline during a crop drag, all as DOM chrome above the canvas; the caption block reachable by Tab as an accessible handle (arrow keys move it in 0.05 steps, committed on keyup); in dev and test builds only, `data-caption-text`, `data-caption-colors`, `data-caption-position` mirroring the visible chunk (D-155, §20.4), stripped from production.
5. The Free watermark badge (D-172) when no Creator token is cached.

G 12.4's look checks on the portrait and landscape clips.

#### Do Not Implement Yet

- The full update rule (pause and restore, failure and undo, history: Prompt 114). The Captions tab's editor (Prompt 115): the click sets `editingWord` and opens the tab.

### Existing Components To Reuse

- `Scene::hit` through the binding; the snap points of `placement.rs` (0.15, 0.55, 1.0), mirrored **once** in `preview-interact.ts` as `CAPTION_SNAPS` with a comment naming `placement.rs` as their home (the page cannot import a scene constant; "Gaps settled" item 8), guarded by Prompt 123's drag case, which asserts `data-caption-position` equals 0.15 after a snap to Top; the `Slider` primitive for the crop's accessible path (Prompt 116).

### Files To Create

- `web/src/usecases/{preview-interact,set-caption-position,set-crop-offset,review-update}.ts`.

### Files To Modify

- `web/src/workers/{protocol,render.worker}.ts`, `web/src/ui/components/PreviewPlayer.tsx`, `web/src/ui/styles/preview.module.css`, `web/eslint.config.js`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- Chrome is never part of a frame (D-183): the outline, guide and frame outline are DOM above the canvas.
- One drag is one `setScene` and one `review_action` (D-142); nothing commits on `input` or `pointermove` (Appendix A for `preview-interact.ts`).
- Hit tests run in the render worker, never on the main thread (D-193).
- The dev-only mirrors and hooks are absent from production (Prompt 124 proves it).

### Data Types / Interfaces

```ts
hitTest(p: { at: OutMs; x: number; y: number }): Promise<Hit>;   // x, y fractions of the canvas
type Hit = { kind: "word"; word: WordIdx } | { kind: "captions" } | { kind: "video" };
export function setCaptionPosition(p: number): Promise<void>;     // CaptionPlacement value
export function setCropOffset(o: number): Promise<void>;          // landscape and square only
```

### Implementation Requirements

#### 1. Hover budget
At most one `hitTest` per animation frame; a pending hover replaced by the newest; none while playing.

#### 2. Look at it (G 12.4)
Hover a caption word → text cursor and outline; drag captions to the top → snap at Top, the visuals below the captions; drag the landscape video → outline during, framing moves on release; click a caption word → the Captions tab opens and `editingWord` is set.

#### 3. Hit test timing
A console loop of 100 `hitTest` calls while paused (dev build): record the mean and worst against 16 ms (§21.1 "Hit test and hover").

### Error Handling

A failed `setScene` in the minimal update restores the previous `EditState` and shows the existing failure path; Prompt 114 adds "Undo this change".

### Concurrency / Async Requirements

A second drag while the first's commit is in flight waits (the clip machine refuses `edit_start` from `updating`; D-147).

### Persistence Requirements

`editsRepo.put` after each commit (the record already takes the whole `EditState`).

### API Requirements

`review_action { action: "caption_position" }` and `{ action: "crop_offset" }` are sent; the first is accepted by the server only after Prompt 120 widens the enum: until then the analytics client drops the unknown value as V3's allowlist does (recorded).

### Security Requirements

Analytics carry enums only (INV-18).

### Testing Requirements

#### Unit Tests
None required (Prompt 121's `review-update` block, Prompt 123's drag cases).

#### Integration Tests
`pnpm e2e:media` green.

#### Failure-Path Tests
Drag while playing: nothing happens (direct manipulation is paused-only).

### Validation Commands

```bash
pnpm build:wasm && pnpm check && pnpm test                        # exit 0
pnpm build && grep -rlE "data-caption-(text|colors|position)" web/dist && echo LEAK || echo clean   # clean
pnpm e2e:media 2>&1 | tail -3                                     # green (keyed)
```

### Acceptance Criteria

- [ ] Hover shows the right cursor and a word outline while paused; the hit test round trip is recorded against 16 ms.
- [ ] Caption drag snaps within 16 px at Top, Middle, Bottom and moves the visuals to the free side; the video drag moves the crop; each commits once on release.
- [ ] Clicking a caption word opens the Captions tab with `editingWord` set.
- [ ] The badge shows when not on Creator; dev mirrors absent from production; REOPENED CONTRACT lines; G M-12 commands pass; CI green.

### Agent Instructions

Before: read D-142, D-171 to D-174, D-183, §14.1, §16.2, §17.4, G 12.3-12.4, M-12; load a design skill if available. During: worker method, use-cases, then the player. After: SAB gate plus the media run; commit `V4 P113 (S11): hit testing, caption and video drags, direct manipulation`; push; read CI.

### Expected Result

A creator can grab the captions or the video on the preview and move them, and click a word to start fixing it.

---

## Prompt 114 (V4 24/38) — The update rule, undo and redo, word edits, "Change all" (S12 part 1)

**Effort: High.** One rule for every edit (pause and restore, optimistic state, plan or scene mode, failure and undo, history), sentence re-detection through a worker that keeps the transcript, and replace-all as one undo step.

### Objective

Complete `review-update.ts` (Appendix C.2), add the 50-step history and undo/redo, wire the worker's `redetectSentence` and edit-aware `detect`, and write `edit-word.ts` and `replace-all.ts`.

### Project State Before This Prompt

- Prompt 113 committed; a minimal `review-update.ts` serves the two drags.

### Why This Comes Here

Every control of Prompts 115-117 goes through this rule; it must exist, with undo, before the transcript editor calls it.

### Scope

#### Implement

- G 13.1 rows 1-4:
  1. `state/history.ts` §16.1, D-184: `push`, `undo`, `redo`, `canUndo`, `canRedo`, `clear`; `HISTORY_MAX = 50`; a new change after undo drops the redo branch; cleared on a new clip; its own store so the top bar reads it.
  2. `usecases/review-update.ts` Appendix C.2: `update(change, mode, action)`: return unless `ready`; `previous`; `next = change(previous)`; `editStarted(next)`; pause if playing and remember; mode `plan`: `redetectSentence` → `rulesPlan` → `setEvents`, `setPlan`; `setScene({ transcript, plan, edit: next, timeMap, profile: "preview" })`; `ui-store.setSave("saving")`, `editsRepo.put`, `setSave("saved")`; `editDone()`; play again if remembered; `history.push(previous)` only after success; `review_action`. Any failure in the worker or storage steps: `editFailed(previous, failure)`, `client_error`, the scene not rebuilt, `ErrorPanel` with "Undo this change" (wired fully in Prompt 119).
  3. `web/eslint.config.js` §4: the use-case edges `edit-word.ts`, `toggle-event.ts`, `set-style.ts`, `set-highlight-numbers.ts`, `replace-all.ts`, `undo-redo.ts` → `review-update.ts` (the two drag edges exist since Prompt 113), each with its reason as D-58's. **REOPENED V2 CONTRACT.**
  4. `usecases/undo-redo.ts` D-184: mode `plan` when `word_edits` differ, else `scene`; `review_action { action: "undo" | "redo" }`; the top bar's buttons enabled from `history`.
- G 13.2 rows 1-3:
  1. `render.worker.ts` §15.1: `redetectSentence({ sentence, wordEdits })` using the kept transcript, prosody and last events, keeping and returning the new events; `detect` with edits for replace-all. Fill the V3 markers in `render.worker.ts`, `load-render.ts` and `detect_api.rs`'s caller (the binding is frozen; the TypeScript view gains `redetectSentence` where V3 marked it).
  2. `usecases/edit-word.ts` D-146: `editWord(word, text)` → mode `plan` for the word's sentence; an empty text hides the word.
  3. `usecases/replace-all.ts` D-175: words whose **original** text equals the edited word's original (case-folded, whole word) and not yet edited; one change; `detect` over the whole transcript with the new edits; one history step; `review_action { action: "replace_all" }`.

#### Do Not Implement Yet

- `CaptionEditor.tsx` and the toast (Prompt 115). Style, visuals and framing panels (Prompt 116). Shortcut keys (Prompt 117).

### Existing Components To Reuse

- `clip-store`'s `editStarted`, `editDone`, `editFailed` (add them now if Prompt 113's minimal rule did not), the clip machine's `edit_start`/`edit_done` events (V2), `editsRepo.put`, `RenderApi.redetectSentence` (V3), `track()` for analytics.

### Files To Create

- `web/src/state/history.ts`, `web/src/usecases/{undo-redo,edit-word,replace-all}.ts`.

### Files To Modify

- `web/src/usecases/review-update.ts`, `web/src/state/clip-store.ts`, `web/src/workers/{protocol,render.worker}.ts`, `web/src/wasm/load-render.ts`, `web/src/ui/layout/TopBar.tsx` (undo/redo enabled from history, save state from `ui-store`), `web/eslint.config.js`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- Edits are optimistic and local (TS §13.4); nothing awaits the network (Appendix A).
- A change while one is in flight waits (D-147); an `IllegalTransitionError` is a bug.
- Every state change goes through `transition()` (INV-16).
- One home per value: `HISTORY_MAX` in `history.ts`.

### Data Types / Interfaces

Appendix C.2:

```ts
export function update(change: (edit: EditState) => EditState, mode: "plan" | "scene", action: ReviewAction): Promise<void>;
export function editWord(word: WordIdx, text: string): Promise<void>;
export function replaceAll(fromWord: WordIdx, to: string): Promise<number>;   // returns the number changed
export function undo(): Promise<void>; export function redo(): Promise<void>;
```

### Implementation Requirements

#### 1. Undo after a failed edit
`editFailed(previous, failure)` restores `previous` in the store and sets the failure; "Undo this change" (Prompt 119) calls `setScene` with `previous`.

#### 2. Replace-all count
`replaceAll` returns N for the toast (Prompt 115); N excludes the word just edited.

### Error Handling

Worker failures are stored, not thrown; a fault of the use-case itself is thrown (Appendix C.2).

### Concurrency / Async Requirements

A queue of one: `update` awaits an in-flight update before starting.

### Persistence Requirements

`editsRepo.put(clipId, { editState })` per change; `ui-store.save` reflects it.

### API Requirements

`review_action` values `word_edit`, `replace_all`, `undo`, `redo` (the last three accepted by the server from Prompt 120).

### Security Requirements

Analytics carry enums and booleans only; no word text ever (INV-18).

### Testing Requirements

#### Unit Tests
`history.test.ts` is Prompt 121's; a temporary `zz-` check of the history's limit is allowed and deleted.

#### Integration Tests
`pnpm e2e:media` green.

#### Failure-Path Tests
In `pnpm dev`, `__offcutDev`-free: close the session from DevTools during an edit → the previous state is restored.

### Validation Commands

```bash
pnpm check && pnpm test                                           # exit 0
git grep -nE "\bV4\b" -- web/src/workers/render.worker.ts web/src/wasm/load-render.ts   # no output
pnpm e2e:media 2>&1 | tail -3                                     # green (keyed)
```

### Acceptance Criteria

- [ ] `review-update.ts` follows Appendix C.2's eight steps; history is pushed only after success.
- [ ] Undo and redo work across word edits, toggles (when they exist) and drags; the top bar reflects `canUndo`/`canRedo` and the save state.
- [ ] `editWord` re-detects the word's sentence and rebuilds the rules plan; `replaceAll` changes every matching unedited word in one step.
- [ ] The lint edges are added with reasons (REOPENED CONTRACT); V3 markers filled; CI green.

### Agent Instructions

Before: read D-146, D-147, D-175, D-184, §15.1, §16.1-§16.2, Appendix C.2, G 13.1-13.2 rows 1-3. During: the rule first, then history, then the word use-cases. After: SAB gate plus the media run; commit `V4 P114 (S12): update rule, undo and redo, word edits, change all`; push; read CI.

### Expected Result

Every edit flows through one rule and can be undone; words can be fixed one at a time or everywhere at once.

---

## Prompt 115 (V4 25/38) — The transcript editor: click a word to fix it (S12 part 2)

**Effort: High.** A flowing, keyboard-complete transcript editor that follows playback, opens a word in place with its text selected, commits only on Enter or Tab, and must keep edit-to-preview within 150 ms.

### Objective

Build `CaptionEditor.tsx` (the Captions tab), the replace-all toast, and measure edit-to-preview in the Performance panel.

### Project State Before This Prompt

- Prompt 114 committed; `editWord`, `replaceAll`, `undo`, `redo` exist; a click on a caption word sets `editingWord`.

### Why This Comes Here

D-182: the transcript editor is the main way a creator fixes words, and it needs the full update rule.

### Scope

#### Implement

- G 13.2 rows 4-5:
  - `ui/components/CaptionEditor.tsx` D-182, §17.3: the transcript as flowing text in blocks that are the caption chunks, each block with its start time (click → `controlPreview.seek`); while playing, the spoken word highlighted and kept in view; the "Follow playback" `Switch` (on by default; off as soon as the user scrolls); click a word → seek to it, pause, the word becomes an inline field with its text selected; Enter or Tab commits (Tab opens the next word), Escape cancels, an empty commit hides; edited words with a dotted underline; hidden words struck through with a "Show" button; the ⋯ menu (on hover or the context-menu key) with "Hide" and "Restore original"; after a commit, when `replaceAll` would change N > 0 words, a toast "Change all N “a” to “b”?" with one action; `ui-store.editingWord` opens a word from the preview click; the hint "Click any word to fix it."
  - Edit-to-preview timing: in `pnpm dev`'s Performance panel, commit → `setScene` answer → the redraw; target ≤ 150 ms (`EDIT_TO_PREVIEW_MAX_MS`); over → look at cache misses first (D-173). Record five readings.
- The chunk blocks come from the scene's own chunking, never from a second implementation in TypeScript: `Scene::chunk_ranges() -> Vec<(WordRange, OutMs)>` in `offcut-scene/src/lib.rs` (a read of what `build_scene` already computed), returned by `scene_api::set_scene` as `SceneReport.chunks`, with a changelog line ("Gaps settled" item 6).

#### Do Not Implement Yet

- Style, Visuals, Framing tabs (Prompt 116); shortcuts sheet (Prompt 117).

### Existing Components To Reuse

- The primitives (`Switch`, `Tooltip`, `Toast`, `IconButton`, `Kbd`), `editWord`, `replaceAll`, `controlPreview.seek`, `preview-store.position`, `ui-store`.

### Files To Create

- `web/src/ui/components/CaptionEditor.tsx`.

### Files To Modify

- `web/src/ui/layout/SidePanel.tsx` (Captions tab content), `web/src/ui/styles/panel.module.css`, `crates/offcut-scene/src/lib.rs` (`chunk_ranges`), `crates/offcut-wasm-render/src/scene_api.rs` and `web/src/workers/protocol.ts` (`SceneReport.chunks`), `web/src/copy/messages.ts` (only if a key is missing), `docs/v4/v4changelog.md`.

### Architecture Requirements

- No timeline UI (PS forbids it; Appendix A: `CaptionEditor` must not show a timeline).
- A field never commits on blur without Enter or Tab (Appendix A).
- A 90 s transcript (about 300 words) renders without virtualisation and without a main-thread task over 50 ms (D-193).
- The transcript editor is the canvas's text alternative (D-192): the canvas's accessible name points to it.

### Data Types / Interfaces

```ts
type SceneReport = { dropped: Dropped[]; chunks: Array<{ start: WordIdx; end: WordIdx; at: OutMs }> };
```

### Implementation Requirements

#### 1. Follow
Scrolling by the user (wheel, touch, keyboard) sets follow off; the switch turns it back on and scrolls to the active word.

#### 2. Tab order
Inside an open field, Tab commits and opens the next visible word; Shift+Tab the previous; Escape returns focus to the word button.

### Error Handling

A failed commit shows the update rule's failure (the field closes, the previous text stays, the error card offers "Undo this change").

### Concurrency / Async Requirements

A commit while another is in flight waits (the update queue).

### Persistence Requirements

Through `review-update.ts`.

### API Requirements

None.

### Security Requirements

No word text in analytics (INV-18).

### Testing Requirements

#### Unit Tests
None required.

#### Integration Tests
`pnpm e2e:media` green; Prompt 123's cases cover the editor.

#### Failure-Path Tests
Commit an edit with the render worker made to fail through DevTools: the previous text stays and the error card appears.

### Validation Commands

```bash
cargo test -p offcut-scene                                        # chunk_ranges covered by an inline test
pnpm build:wasm && pnpm check && pnpm test                        # exit 0
pnpm e2e:media 2>&1 | tail -3                                     # green (keyed)
```

### Acceptance Criteria

- [ ] The Captions tab shows the transcript in chunk blocks, follows playback, and opens a word in place on click or from the preview.
- [ ] Enter/Tab commit, Escape cancels, empty hides; edited and hidden words are marked; the ⋯ menu works; the replace-all toast appears when N > 0.
- [ ] Edit-to-preview is ≤ 150 ms in five readings on D1 (recorded), or the cause is found and fixed in this prompt.
- [ ] Chunk ranges come from the scene, not a second implementation; CI green.

### Agent Instructions

Before: read D-173, D-175, D-182, D-192, D-193, §17.3-§17.4, G 13.2 rows 4-5; load a design skill if available. During: build, then time. After: SAB gate plus the media run; commit `V4 P115 (S12): transcript editor, click a word to fix it`; push; read CI.

### Expected Result

A creator fixes a word by clicking it, sees the fix on the video almost at once, and can fix it everywhere with one click.

---

## Prompt 116 (V4 26/38) — Style cards, Visuals and Framing tabs (S12 part 3)

**Effort: High.** Live thumbnails rendered by the real renderer into a borrowed canvas with the preview restored after, debounced and cancelled correctly, plus three panels that must stay plain and accessible.

### Objective

Wire the Style tab (three live cards from the clip itself and the Highlight numbers switch), the Visuals tab (suggested visuals with a switch each, the count, the empty state) and the Framing tab (caption position and crop), and the empty `CutsSection`.

### Project State Before This Prompt

- Prompt 115 committed; the update rule and the Captions tab work.

### Why This Comes Here

D-186, D-187: each tab has one job and goes through the update rule; the thumbnails need the paused redraw and the scene's style switch.

### Scope

#### Implement

G 13.3 rows 1-6:

1. `workers/protocol.ts`, `render.worker.ts` §15.1: `styleThumbnails({ at, width, height })`: one `OffscreenCanvas` (216 x 384), the scene rebuilt three times at a thumbnail profile with the current edits and each style, `transferToImageBitmap` after each render, the preview's scene and canvas restored after (as the export switch does). **REOPENED V1 CONTRACT** (addition).
2. `usecases/style-thumbnails.ts`, `set-style.ts`, `set-highlight-numbers.ts` §16.2: `refreshThumbnails()` debounced `THUMB_DEBOUNCE_MS = 300`, only while the Style tab is shown, the latest call wins; `setStyle` and `setHighlightNumbers` through the update rule (mode `scene`).
3. `ui/components/StylePanel.tsx`, `StylePicker.tsx` D-186: three cards (the image in a 9:16 frame, the name, one line; a ring when selected; `aria-pressed`); a `Skeleton` while loading; the Highlight numbers `Switch`.
4. `usecases/toggle-event.ts`, `ui/components/PlanItemList.tsx` D-187: rows in time order with the kind's icon (`Hash`, `List`, `ArrowRight`, `Zap`), its friendly label ("Big number", "List", "Before → after", "Emphasis"), its figure ("90 min → 10 min"), its time (click seeks), a `Switch`; "N of M visuals on"; with none, `noEventsFound` and `visuals.whatWeLookFor` (D-154); `toggleEvent(id, on)` sets `event_overrides[id]`, mode `scene`, `review_action { action: "event_toggle", event_kind, enabled }`.
5. `ui/components/FramingPanel.tsx`, `CropOffsetControl.tsx` D-187: `SegmentedControl` Top · Middle · Bottom (the current value shown even between snaps), the hint "Or drag the captions on the video."; the crop `Slider` (-1.0 to 1.0, step 0.01, committed on release) for landscape and square clips only, with "Or drag the video sideways."
6. `ui/components/CutsSection.tsx`: renders nothing while the plan has no cuts (V6.2).

#### Do Not Implement Yet

- Shortcuts and the toast host's final wiring (Prompt 117). Cancel and error actions (Prompt 119).

### Existing Components To Reuse

- `setCaptionPosition`, `setCropOffset` (Prompt 113); the primitives; the icons; the plan's `ValidItem` list as the scene reports it (the page holds the `EditPlan`; the visible list is the plan's items minus the dropped ones in `SceneReport`).

### Files To Create

- `web/src/usecases/{style-thumbnails,set-style,set-highlight-numbers,toggle-event}.ts`, `web/src/ui/components/{StylePanel,StylePicker,PlanItemList,FramingPanel,CropOffsetControl,CutsSection}.tsx`.

### Files To Modify

- `web/src/workers/{protocol,render.worker}.ts`, `web/src/ui/layout/SidePanel.tsx`, `web/src/ui/styles/panel.module.css`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- What a card shows is what the export will show (D-172): the real renderer, the real scene, the current edits.
- Thumbnails, hit tests and scene builds run in the render worker (D-193).
- The figure on a row comes from the item's `EventParams` display strings, never parsed in TypeScript (INV-22).

### Data Types / Interfaces

```ts
styleThumbnails(p: { at: OutMs; width: number; height: number }): Promise<ImageBitmap[]>;   // transferred
export function toggleEvent(id: EventId, on: boolean): Promise<void>;
export function setStyle(style: StyleId): Promise<void>;
export function setHighlightNumbers(on: boolean): Promise<void>;
```

### Implementation Requirements

#### 1. Restoring the preview after thumbnails
The preview's canvas, scene and last position are restored before the call answers; the next preview frame equals the one before the call.

#### 2. Timing
Ten openings of the Style tab: time to three images; record against 400 ms (§21.1).

### Error Handling

A thumbnail failure leaves the cards' skeletons and does not block the tab; the style buttons still work.

### Concurrency / Async Requirements

Debounce 300 ms; the latest call wins; a call while the loop plays waits until pause (thumbnails are drawn while paused only).

### Persistence Requirements

Through the update rule.

### API Requirements

`review_action` values `style`, `highlight_numbers`, `event_toggle`, `caption_position`, `crop_offset`.

### Security Requirements

Analytics carry enums and booleans only.

### Testing Requirements

#### Unit Tests
None required.

#### Integration Tests
`pnpm e2e:media` green.

#### Failure-Path Tests
Open and close the Style tab quickly five times: no stale images, no leaked `ImageBitmap` (closed when replaced).

### Validation Commands

```bash
pnpm build:wasm && pnpm check && pnpm test                        # exit 0
pnpm e2e:media 2>&1 | tail -3                                     # green (keyed)
```

### Acceptance Criteria

- [ ] Three style cards show the clip's own frame in Clean, Bold, Mono; choosing one applies it; thumbnails timed against 400 ms.
- [ ] Highlight numbers switch works; the Visuals tab lists items with switches, the count, and the empty state.
- [ ] Framing tab: caption position segmented control; crop slider only for landscape and square clips.
- [ ] `CutsSection` renders nothing; REOPENED CONTRACT line for `styleThumbnails`; CI green.

### Agent Instructions

Before: read D-154, D-169, D-171, D-186, D-187, §15.1, §16.2, §17.3, G 13.3; load a design skill if available. During: worker method first, then use-cases, then panels. After: SAB gate plus the media run; commit `V4 P116 (S12): style cards, visuals and framing tabs`; push; read CI.

### Expected Result

The side panel's four tabs work, each with one job and plain words.

---

## Prompt 117 (V4 27/38) — Shortcuts, toasts, and the creator walk-through (S12 part 4)

**Effort: Medium.** One keyboard hook, a sheet and a toast host, then a scripted walk-through; **switch to High** if the walk-through shows edit-to-preview over 150 ms or a main-thread task over 50 ms.

### Objective

Add the keyboard map of D-185 with its sheet, wire the toast host, then do G 13.5's walk-through as a creator would and fix what it finds, closing S12.

### Project State Before This Prompt

- Prompt 116 committed; all four tabs work.

### Why This Comes Here

Shortcuts are the last control of D-185; the walk-through is G M-13's proof that the editor works end to end before robustness work begins.

### Scope

#### Implement

- G 13.4: `ui/hooks/use-shortcuts.ts` (one map: Space play/pause, ←/→ one frame when paused, Shift+←/→ one second, Ctrl/⌘+Z undo, Shift+Ctrl/⌘+Z and Ctrl+Y redo, Escape closes a field or menu, `?` opens the sheet; ignored while focus is in a text field except Escape); `ui/components/ShortcutsSheet.tsx` (the list, each with `Kbd`); `ui/components/ToastHost.tsx` wired to `ui-store` (one stack; replace-all, hide and undo toasts).
- G 13.5: the walk-through, without looking at the code: fix "90" to "9" in the transcript; fix it by clicking the word on the video; a repeated word and "Change all N" (the labeled `w`-file text loaded through the dev hooks if needed, never a lent clip); Ctrl+Z three times and Shift+Ctrl+Z once; open Style (cards within half a second; Bold applies on click); Highlight numbers off; Visuals: switch the list off ("3 of 4 visuals on"); Framing: Top; Space, ←, →. Record each result.
- G M-13: in the Performance panel, no main-thread task over 50 ms during the tasks (D-193); edit-to-preview under 150 ms.

#### Do Not Implement Yet

- Cancel, crash, error actions, export pill (Prompts 118-119).

### Existing Components To Reuse

- `Kbd`, `Toast`, `ui-store`, `controlPreview`, `undo`, `redo`.

### Files To Create

- `web/src/ui/hooks/use-shortcuts.ts`, `web/src/ui/components/{ShortcutsSheet,ToastHost}.tsx`.

### Files To Modify

- `web/src/ui/layout/EditorLayout.tsx`, `docs/v4/v4changelog.md`, and whatever the walk-through fixes.

### Architecture Requirements

- Every shortcut is also a visible control (D-185).
- Shortcuts are disabled during an export (TS §12.4 "Available when the export finishes").

### Data Types / Interfaces

```ts
export function useShortcuts(): void;   // mounted once by EditorLayout
```

### Implementation Requirements

#### 1. Text fields
Space inside a field types a space; Escape inside a field cancels the field.

#### 2. Walk-through record
A table in the entry: task, expected (G 13.5), observed, fixed (yes/no, how).

### Error Handling

None new.

### Concurrency / Async Requirements

Undo/redo shortcuts go through the update queue like the buttons.

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

None beyond the SAB.

### Testing Requirements

#### Unit Tests
None required.

#### Integration Tests
`pnpm e2e:media` green.

#### Failure-Path Tests
Shortcuts during an export do nothing.

### Validation Commands

```bash
pnpm build:wasm && pnpm check && pnpm test                        # exit 0
pnpm e2e:media 2>&1 | tail -3                                     # green (keyed)
```

### Acceptance Criteria

- [ ] The keyboard map of D-185 works; the sheet opens with `?`; toasts stack and announce.
- [ ] Every task of G 13.5 behaves as expected on D1 (table recorded).
- [ ] Edit-to-preview under 150 ms and no main-thread task over 50 ms during the tasks (G M-13); CI green.

### Agent Instructions

Before: read D-185, D-193, §17.4, G 13.4-13.5, M-13. During: build, then walk through as a creator. After: SAB gate plus the media run; commit `V4 P117 (S12): shortcuts, toasts, creator walk-through fixes`; push; read CI.

### Expected Result

The editor is complete for a creator: every change is fast, visible, reversible, and reachable by mouse or keyboard.

---

## Prompt 118 (V4 28/38) — The RPC layer: progress throttle, cancel, crash, the pool (S13 part 1)

**Effort: High.** A throttle that must never drop a feed line, cancellation with a timeout and restart, and crash detection that rejects every pending call: concurrency where a mistake hangs the app.

### Objective

Add the progress throttle to the worker side, `cancel`, `onCrash` and `isDead` to the client side, and `pool.cancel(kind)` with a 2 s timeout and `pool.restartWorker(kind)`.

### Project State Before This Prompt

- Prompt 117 committed; the editor works.
- `rpc.ts` and `rpc-worker.ts` are V3's code, split (Prompt 94).

### Why This Comes Here

Every cancel path (Prompt 119) and every crash recovery depends on this layer; its tests come in Prompt 121.

### Scope

#### Implement

G 14.1 rows 1-3:

1. `workers/rpc-worker.ts` D-151: at most 10 `progress` messages per second per job (`PROGRESS_MIN_INTERVAL_MS = 100`): a progress without a feed line inside 100 ms of the last one sent is held and replaced by the next; the held one is flushed before the job's `res`; a progress with a feed line is sent at once.
2. `workers/rpc.ts` §14.2: `cancel(jobId)` posts `{ type: "cancel", jobId }`; `onCrash(cb)`; `isDead()`; a worker `error` or `messageerror` rejects every pending call with `E_WORKER_CRASH` (the pool row's stage) and marks the client dead.
3. `workers/pool.ts` §14.3: `cancel(kind) → "cancelled" | "restarted" | "idle"` resolving when the pending job answers or after `CANCEL_TIMEOUT_MS = 2000`, then `restartWorker(kind)`; `restartWorker` terminates the worker, rejects its pending calls with `E_WORKER_CRASH`, and the next call creates a fresh one; a dead client is replaced on the next call; a restarted render worker has no session.

#### Do Not Implement Yet

- `cancel-job.ts` and the use-case wiring (Prompt 119). `rpc.test.ts` (Prompt 121).

### Existing Components To Reuse

- `serveWorker`'s job context and `CANCELLED`; `lazyClient`; `WorkerCallError`; the pool rows' stages.

### Files To Create

None.

### Files To Modify

- `web/src/workers/{rpc-worker,rpc,pool}.ts`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- Feed lines are never dropped (TS §14.3, Appendix A for `rpc-worker.ts`).
- Transferred buffers stay transferred (TS §31); the dev assertion on a non-detached buffer stays.
- `CANCEL_TIMEOUT_MS` has one home (`pool.ts`).

### Data Types / Interfaces

```ts
interface Client<Api> { call…; cancel(jobId: number): void; onCrash(cb: (f: AppFailure) => void): void; isDead(): boolean }
export const pool: { media; asr; audio; render; cancel(kind: WorkerKind): Promise<"cancelled" | "restarted" | "idle">; restartWorker(kind: WorkerKind): void };
```

### Implementation Requirements

#### 1. Throttle timing
One timer per job at most; cleared at `res` or `err`.

#### 2. Cancel resolution
`"idle"` when no job is pending; `"cancelled"` when the job answers `CANCELLED` before the timeout; `"restarted"` after the timeout.

### Error Handling

`E_WORKER_CRASH` with the stage; a call after a crash creates a new worker.

### Concurrency / Async Requirements

The pending table is the single source of truth; a crash clears it after rejecting; a late `res` from a terminated worker is impossible (terminated) and a late `res` for an unknown job is ignored.

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

None beyond the SAB.

### Testing Requirements

#### Unit Tests
Prompt 121 (`rpc.test.ts`); a temporary `zz-` Vitest for the throttle is allowed and deleted.

#### Integration Tests
`pnpm e2e:media` green (feed lines all shown).

#### Failure-Path Tests
In `pnpm dev`, terminate the render worker from DevTools while paused: the pending call rejects with `E_WORKER_CRASH`.

### Validation Commands

```bash
pnpm check && pnpm test                                           # exit 0
pnpm e2e:media 2>&1 | tail -3                                     # green (keyed)
```

### Acceptance Criteria

- [ ] Progress is throttled to 10/s per job with every feed line kept and the held progress flushed before `res`.
- [ ] `cancel`, `onCrash`, `isDead` exist; a crash rejects every pending call with `E_WORKER_CRASH`.
- [ ] `pool.cancel` resolves `cancelled`, `restarted` or `idle` as specified; `restartWorker` replaces a worker; CI green.

### Agent Instructions

Before: read D-151 to D-153, §14.2, §14.3, Appendix D, G 14.1. During: worker side, client side, pool. After: SAB gate plus the media run; commit `V4 P118 (S13): progress throttle, cancel, crash handling, pool restart`; push; read CI.

### Expected Result

The worker layer can be cancelled, survives a crash, and never floods or starves the page.

---

## Prompt 119 (V4 29/38) — Cancel everywhere, errors in plain words, the export pill (S13 part 2)

**Effort: High.** Four cancel scopes that must each end in the right state with nothing left behind, fourteen error messages in a creator's words with the right single action each, and an export pill with honest progress.

### Objective

Write `cancel-job.ts` and honour it in import, pipeline, model download and export; give every code a person can meet its copy; make `ErrorPanel` choose its one action by context; add `reloadPreview`; and turn the export into a pill with stage, percent, time left and Cancel.

### Project State Before This Prompt

- Prompt 118 committed; `pool.cancel` and crash handling exist.

### Why This Comes Here

C-12 and C-14 (D-152, D-153, D-189, D-190): recovery is part of the experience; it needs the RPC layer and the update rule.

### Scope

#### Implement

- G 14.2 rows 1-3:
  1. `usecases/cancel-job.ts` D-152: owns the abort controllers; `signalFor(scope)`, `cancelJob(scope)` for `"import" | "pipeline" | "model" | "export"`; import → media; pipeline → whichever worker holds the current job, then `pool.audio.release()` and `pool.render.closeSession()`, clip `processing → accepted` (`cancelledRun()`); model → the model manager's abort (bytes kept for resume); export → render, the partial file deleted, export `rendering|muxing → cancelled → idle`; each sends `job_cancelled { stage }`.
  2. `run-pipeline.ts`, `import-clip.ts`, `start-export.ts`: read `signalFor`; end in the scope's state; fill their V3 markers (cancel, `client_error`); every shown failure sends `client_error { error_code, stage }` once (D-153); `retried()` (`failed → processing`) runs the pipeline from step A.
  3. `ProcessingCard`, `ModelDownloadPanel`, `ExportProgress`: Cancel buttons wired; the cancelled stage card ("Cancelled." with "Process again" and "Start over").
- G 14.3 rows 1-4:
  1. `copy/messages.ts`: the 14 error entries of D-159 (`title`, `body`, `action`) in D-190's tone (e.g. `E_GPU_LOST`: "The preview stopped" / "Your graphics card was reset, which can happen when another app uses it." / "Reload preview"); `scripts/check-copy-codes.mjs`'s `COPY_PENDING` loses those 14.
  2. `ErrorPanel.tsx` D-190: one primary action by context (Try again for a failed clip → `runPipeline`; Reload preview → `reloadPreview`; Undo this change → restore and `setScene`), "Start over" secondary; `client_error` once per shown failure; a collapsed "Details" only in development builds; never a code or a file name.
  3. `usecases/control-preview.ts`: `reloadPreview()` (open the session, set the scene, attach a new canvas; no re-processing).
  4. `ExportProgress.tsx`, `TopBar.tsx` D-189: the Export button becomes a pill: the stage in words ("Rendering video", "Finishing file"), percent, a determinate bar, time left once 3 s of frames give a rate, Cancel; done shows success with "Download again"; review controls disabled during an export with the tooltip "Available when the export finishes".
- Dev-only hooks for Prompt 124 and G 14.5 in `web/src/ui/dev-hooks.ts` and their worker halves: `crash(kind)`, `failNextSetScene()`, `hangWorker(kind)`.

#### Do Not Implement Yet

- The analytics enum (Prompt 120). The restart budget, `.persistent` copy, resume from the last stage (V7).

### Existing Components To Reuse

- `pool.cancel`, `restartWorker`, the export machine's `cancel`/`cancel_done` (V2), the clip machine's `cancel`/`retry` (V2), the model manager's existing abort signal, `opfs` delete of the temporary export.

### Files To Create

- `web/src/usecases/cancel-job.ts`.

### Files To Modify

- `web/src/usecases/{run-pipeline,import-clip,start-export,control-preview}.ts`, `web/src/ui/components/{ProcessingCard,ModelDownloadPanel,ExportProgress,ErrorPanel}.tsx`, `web/src/ui/layout/TopBar.tsx`, `web/src/ui/dev-hooks.ts`, `web/src/workers/render.worker.ts` (dev halves), `web/src/copy/messages.ts`, `scripts/check-copy-codes.mjs`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- `cancel-job.ts` leaves no temporary file (Appendix A); the export's partial file is deleted.
- A use-case may import `cancel-job.ts` (TS §7; no new lint edge).
- `models/*` is untouched: the abort uses the existing signal (§4).
- Error copy is interpolated from `messages.ts` only; `ErrorPanel` shows no code (D-190).

### Data Types / Interfaces

```ts
export type CancelScope = "import" | "pipeline" | "model" | "export";
export function signalFor(scope: CancelScope): AbortSignal;
export function cancelJob(scope: CancelScope): Promise<void>;
export function reloadPreview(): Promise<void>;
```

### Implementation Requirements

#### 1. The four ends
import → `idle`, no OPFS clip directory; pipeline → `accepted` with "Process again"; model → partial bytes kept; export → `idle`, no file under `exports/tmp/`.

#### 2. One `client_error` per shown failure
Guarded by the failure's identity, not by a timer.

### Error Handling

A cancel that times out restarts the worker (Prompt 118) and still ends in the scope's state.

### Concurrency / Async Requirements

`cancelJob` is idempotent; a second call while the first runs returns the same promise.

### Persistence Requirements

The temporary export file deleted on cancel; the model's partial bytes kept.

### API Requirements

`job_cancelled { stage }` and `client_error { error_code, stage }` sent (already allowlisted since V1).

### Security Requirements

No file name, code or path in any user-facing text (D-190, P-11).

### Testing Requirements

#### Unit Tests
None required (Prompt 121's pipeline cancel cases).

#### Integration Tests
`pnpm check` (copy codes: 14 fewer pending); `pnpm e2e:media` green.

#### Failure-Path Tests
Prompt 120's break-it session.

### Validation Commands

```bash
node scripts/check-copy-codes.mjs                                 # exit 0; COPY_PENDING 14 shorter
pnpm check && pnpm test                                           # exit 0
git grep -nE "\bV4\b" -- web/src/usecases                         # no output
pnpm build && grep -rl "__offcutDev" web/dist && echo LEAK || echo clean   # clean
pnpm e2e:media 2>&1 | tail -3                                     # green (keyed)
```

### Acceptance Criteria

- [ ] Import, pipeline, model download and export can each be cancelled and end in their scope's state with nothing left behind; `job_cancelled` sent.
- [ ] The 14 codes have copy in D-190's tone; `ErrorPanel` offers the right single action; `reloadPreview` works.
- [ ] The export pill shows stage, percent, a bar, time left and Cancel; done offers "Download again".
- [ ] The V3 use-case markers are filled; dev hooks absent from production; CI green.

### Agent Instructions

Before: read D-152, D-153, D-159, D-189, D-190, §16.2, §17.2, §17.5, G 14.2-14.3; load a design skill if available. During: cancel, then errors, then the pill. After: SAB gate plus the media run; commit `V4 P119 (S13): cancel everywhere, error cards, export pill`; push; read CI.

### Expected Result

Every long job can be stopped, every failure a person can meet speaks plainly with one way forward, and the export shows honest progress.

---

## Prompt 120 (V4 30/38) — The analytics enum widened, and trying to break it (S13 part 3)

**Effort: Medium.** One shared enum and its server tests, regenerated types, then a scripted break-it session.

### Objective

Add `caption_position`, `highlight_numbers`, `replace_all`, `undo` and `redo` to `ReviewActionKind`, regenerate the types, run the server's tests, then try to break the robustness work as G 14.5 lists, closing S13.

### Project State Before This Prompt

- Prompt 119 committed.

### Why This Comes Here

§13: the server must accept the new review actions before the deploy; G 14.5 is S13's proof.

### Scope

#### Implement

- G 14.4: `crates/offcut-api-types/src/analytics.rs`: the five members; in its inline `#[cfg(test)]` module, one round-trip case per new member (`{"name":"review_action","props":{"action":"<member>"}}` reads and writes back) and the generated-inline assertion for `ReviewActionKind` updated to `"word_edit, event_toggle, style, crop_offset, caption_position, highlight_numbers, replace_all, undo, redo"`; `server/tests/analytics_allowlist.rs` stays frozen (it already proves the route accepts `review_action` through the shared type: "Gaps settled" item 7); `pnpm gen:types && sh scripts/check-gen-clean.sh` (`gen/api.ts` and `ANALYTICS_EVENT_DOCS` regenerated); close the Prompt 92 row for `analytics.rs`. **REOPENED CONTRACT.**
- `set -a; . ./.env; set +a; cargo test -p offcut-api-types -p offcut-api` (Docker running; local Postgres on port 9000).
- G 14.5: each row, with the result recorded: cancel during transcription; cancel the model download at 30% (resume next run); cancel an export at 50% (no file in OPFS `exports/tmp/`); kill the render worker while paused (`__offcutDev.crash("render")`) → "The preview stopped", "Reload preview" restores it with the edits; the same during processing → "Try again"; a failed edit (`__offcutDev.failNextSetScene()`) → "Undo this change" restores.
- G M-14.

#### Do Not Implement Yet

- Any server route or table (none in V4).

### Existing Components To Reuse

- The allowlist test file of `offcut-api` (V1); the generator.

### Files To Create

None.

### Files To Modify

- `crates/offcut-api-types/src/analytics.rs` (members and inline tests), `web/src/gen/api.ts` (generated), `docs/v4/v4changelog.md`.

### Architecture Requirements

- Analytics props are enums and booleans only (INV-18).
- The server image rebuilds with the wider enum and deploys first at the merge (§13, G 19.4).

### Data Types / Interfaces

```rust
pub enum ReviewActionKind { /* V3's members */, CaptionPosition, HighlightNumbers, ReplaceAll, Undo, Redo }   // serde snake_case
```

### Implementation Requirements

#### 1. Where the cases live
`analytics.rs`'s inline tests (they already assert each enum's generated list at its line ~338); `server/tests/analytics_allowlist.rs` is not edited ("Gaps settled" item 7).

#### 2. Break-it record
A table: action, expected (G 14.5), observed.

### Error Handling

None new.

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

`POST /api/v1/analytics` accepts the five new `review_action` values.

### Security Requirements

`.env` holds the local values only; no production query.

### Testing Requirements

#### Unit Tests
Five round-trip cases and the updated enum-list assertion in `analytics.rs`.

#### Integration Tests
Server tests; the full gate.

#### Failure-Path Tests
G 14.5's six rows.

### Validation Commands

```bash
set -a; . ./.env; set +a; cargo test -p offcut-api-types -p offcut-api   # all pass
sh scripts/check-gen-clean.sh                                     # exit 0
pnpm check && pnpm test                                           # exit 0
pnpm e2e:media 2>&1 | tail -3                                     # green (keyed)
```

### Acceptance Criteria

- [ ] `ReviewActionKind` has the five members with a test case each; generated types clean; server tests pass locally and in CI.
- [ ] Every row of G 14.5 behaves as written (table recorded).
- [ ] The frozen check prints nothing beyond the recorded exceptions; G M-14 passes; CI green.

### Agent Instructions

Before: read §13, D-152, D-153, G 14.4-14.5, M-14. During: enum, regenerate, server tests, then the session. After: SAB gate plus the media run; commit `V4 P120 (S13): analytics review actions; robustness checked`; push; read CI (the server tests run there).

### Expected Result

The server accepts V4's review actions and the editor survives cancels, crashes and failed edits in plain words.

---

## Prompt 121 (V4 31/38) — Vitest: RPC, the two machines, the pipeline with fake workers, the history (S14)

**Effort: High.** A scripted fake `Worker` with fake timers for the throttle and the cancel timeout, every legal and illegal transition of two machines, and a pipeline test that drives five cancel points and a held failure.

### Objective

Write the five Vitest files of §20.3 and make them pass.

### Project State Before This Prompt

- Prompt 120 committed; RPC, pool, cancel, the update rule and the history exist; Vitest is 92.

### Why This Comes Here

G Phase 15: the RPC layer, the machines V2 declared but never tested, the pipeline and the history are covered before the end-to-end suites rely on them.

### Scope

#### Implement

G 15.1 rows 1-5:

1. `web/src/workers/rpc.test.ts` §14.2: a scripted fake `Worker` (a class with `postMessage`, `terminate` and handlers the test drives); request and response; failure; the throttle with `vi.useFakeTimers()` (30 progress messages in 1 s arrive as at most 10; every feed line arrives; the held progress arrives before `res`); cancel answered; cancel not answered → the timeout path through `pool` with a fake clock (`"restarted"`); a crash rejects every pending call with `E_WORKER_CRASH`; a call after a crash creates a new worker through the factory; the dev assertion on a non-detached buffer.
2. `web/src/state/machines/clip-machine.test.ts`: every legal transition of TS §12.2's clip table from a fresh store; the four illegal ones of TS §12.3 (`rejected → processing`, `importing → processing`, `processing → importing`, `ready → importing`) throw `IllegalTransitionError`; `updating` refuses `edit_start`.
3. `web/src/state/machines/model-machine.test.ts`: TS §12.2's model table; `verifying → downloading` without `failed` refused.
4. `web/src/usecases/run-pipeline.test.ts` §16.2: `vi.mock("../workers/pool")` with scripted results; the order A, B1 ‖ B2, C, D, E (`rulesPlan`), F (`writeAudio` before `assemble`); `REJECT_NO_SPEECH`; a B2 failure held until B1 settles (D-90); a cancel at each of five stages ends `accepted` with `release` called; a crash ends `failed` with `client_error`; `retry` from `failed` runs again from A; a `review-update` block (mode `plan` calls `redetectSentence` and `rulesPlan`, mode `scene` does not; a failure restores the previous `EditState`; history pushed only on success).
5. `web/src/state/history.test.ts`: push, undo, redo, the 50-step limit, the redo branch dropped by a new change, `clear` on a new clip.

#### Do Not Implement Yet

- End-to-end suites (Prompts 122-124).

### Existing Components To Reuse

- The machine tables (V2), `IllegalTransitionError`, the stores' `reset` helpers if present, Vitest's fake timers.

### Files To Create

- The five test files above.

### Files To Modify

- `docs/v4/v4changelog.md`; source files only to fix a bug a test finds (each named in the entry).

### Architecture Requirements

- Tests use the public API of each module; no test-only export added to a source file except where §20.3 needs a factory injection (`createClient(factory)` already takes one: check before adding).
- No new `*.test.ts` beyond the five (SAB rule 6).

### Data Types / Interfaces

None new.

### Implementation Requirements

#### 1. Determinism
No real timers, no real workers, no network; each test resets the stores.

#### 2. A test that finds a bug
Fix the source in this prompt, name the bug in the entry, keep the test.

### Error Handling

None new.

### Concurrency / Async Requirements

Fake timers advance explicitly; promises flushed with `await vi.runAllTimersAsync()` where needed.

### Persistence Requirements

`editsRepo` mocked in the pipeline test.

### API Requirements

None.

### Security Requirements

None beyond the SAB.

### Testing Requirements

#### Unit Tests
The five files.

#### Integration Tests
The full gate.

#### Failure-Path Tests
Inside the files (crash, cancel, held failure, illegal transitions).

### Validation Commands

```bash
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"  # about 160 passed, 0 failed
pnpm check                                                        # exit 0
```

### Acceptance Criteria

- [ ] The five files exist with every case of §20.3 and G 15.1; all pass.
- [ ] Vitest above 92 by them (the number recorded); any bug found is fixed and named; CI green (G M-15).

### Agent Instructions

Before: read §14.2, §16.1-§16.2, §20.3, Appendix C.2, TS §12.2-§12.3, G Phase 15. During: one file at a time. After: SAB gate; commit `V4 P121 (S14): rpc, machine, pipeline and history tests`; push; read CI.

### Expected Result

The page's concurrency, state machines, pipeline and undo are covered by fast deterministic tests.

---

## Prompt 122 (V4 32/38) — E2E helpers and `preview-sync.spec.ts` (S15 part 1)

**Effort: High.** Timing cases in a real browser on a GPU, read through page attributes and screenshots, that must be stable across runs on D1.

### Objective

Extend the E2E helpers and write `preview-sync.spec.ts`'s four cases: drift over 60 s, first frame, seek, edit-to-preview.

### Project State Before This Prompt

- Prompt 121 committed.

### Why This Comes Here

G Phase 16: the preview's timing is TS §20's acceptance and §21.1's first reading; the helpers serve all three new suites.

### Scope

#### Implement

- G 16.1: `web/tests-e2e/helpers/fixtures.ts` gains `previewStats(page)` (the `data-*` attributes), `seekTo(page, ms)`, `editWord(page, index, text)`, `toggleItem(page, label)`, `setStyle(page, name)`, `dragCaptions(page, to)`, `dragVideo(page, offset)`, `axe(page)` (wraps `@axe-core/playwright`, returns serious and critical violations).
- G 16.2: `web/tests-e2e/preview-sync.spec.ts`: **drift** (the portrait clip plays 60 s; `data-max-drift-ms` ≤ 80 and `late / frames` ≤ 0.05); **first frame** (a non-black canvas within 500 ms of `ready`, read through a screenshot of the canvas); **seek** (paused; seek to 55.0 s, the white-frame marker; mean brightness above 200 within 300 ms); **edit to preview** (20 word commits on the paused clip; the 90th percentile of commit → `setScene` answer plus the redraw's `data-frames` tick ≤ `EDIT_TO_PREVIEW_MAX_MS = 150`). The live-frame counter returns to its idle value after play, seek and pause (INV-11).
- `MEDIA_SUITES` in `playwright.config.ts` gains `preview-sync.spec.ts`.

#### Do Not Implement Yet

- `review-edit.spec.ts` (Prompt 123); `cancel.spec.ts` and `pipeline-preview.spec.ts` changes (Prompt 124).

### Existing Components To Reuse

- `MEDIA_USE`, the cached model (`media-setup`), the portrait fixture, V3's helpers.

### Files To Create

- `web/tests-e2e/preview-sync.spec.ts`.

### Files To Modify

- `web/tests-e2e/helpers/fixtures.ts`, `web/playwright.config.ts`, `docs/v4/v4changelog.md`.

### Architecture Requirements

- The tests read the page, never a worker (D-155).
- A flaky case is fixed, not retried (G 16.7): find the missing wait (`ready`, `document.fonts.ready`, the worker's answer) and record it.

### Data Types / Interfaces

```ts
export async function previewStats(page: Page): Promise<{ frames: number; late: number; maxDriftMs: number; downscaled: boolean }>;
export async function axe(page: Page): Promise<AxeResult[]>;   // serious and critical only
```

### Implementation Requirements

#### 1. Percentile
Sort the 20 durations; take index 17 (the 90th percentile of 20).

#### 2. Records
Each run's numbers into the entry (they seed Prompt 126's readings).

### Error Handling

A case that misses its target fails with the measured number in its message.

### Concurrency / Async Requirements

One media worker (as V3's media project).

### Persistence Requirements

None committed (results stay in `web/test-results/`).

### API Requirements

None.

### Security Requirements

Keyed build only in the run's terminal.

### Testing Requirements

#### Unit Tests
None.

#### Integration Tests
`preview-sync.spec.ts`: 4 passed on D1, twice.

#### Failure-Path Tests
None.

### Validation Commands

```bash
pnpm --filter web exec playwright test --project=media tests-e2e/preview-sync.spec.ts 2>&1 | tail -3   # 4 passed (keyed)
pnpm e2e:media 2>&1 | tail -3                                     # green (keyed)
```

### Acceptance Criteria

- [ ] The eight helpers exist; `preview-sync.spec.ts` has the four cases with TS §20's and D-173's targets.
- [ ] All four pass on D1 twice in a row; the numbers are recorded; CI green.

### Agent Instructions

Before: read D-148, D-155, D-173, §20.4, §21.1, G 16.1-16.2. During: helpers, then the suite. After: SAB gate plus the media run; commit `V4 P122 (S15): E2E helpers, preview-sync suite`; push; read CI.

### Expected Result

The preview's sync, first frame, seek and edit-to-preview are proven in Chrome on D1.

---

## Prompt 123 (V4 33/38) — `review-edit.spec.ts`: the editor in Chrome, axe included (S15 part 2)

**Effort: High.** Sixteen cases that drive every control the founder will use, through real gestures and keyboard, with an accessibility scan; this is the suite most likely to expose app bugs.

### Objective

Write `review-edit.spec.ts` with sixteen self-contained cases (each loading its clip through the cached model) and make them pass on D1.

### Project State Before This Prompt

- Prompt 122 committed; the helpers exist.

### Why This Comes Here

§20.4: the editor's behaviour is proven where a creator meets it; the count of sixteen keeps §20.1's total of 76.

### Scope

#### Implement

`web/tests-e2e/review-edit.spec.ts`, the cases of §20.4 grouped into sixteen ("Gaps settled" item 4):

1. **Edit a number**: on the reference clip, the word recognized as "90" edited to "9" → the from-to's row in `PlanItemList` reads "9 minutes"; `review_action { action: "word_edit" }` sent (fake API).
2. **Hide a word**: an empty commit removes it from the caption (`data-caption-text`); the transcript shows it struck through.
3. **Toggle a visual**: switching the list off removes it (its row's state and the next frame at its time); the toggle survives a word edit elsewhere (D-145).
4. **Style**: Bold → the caption text upper case in `data-caption-text`.
5. **Crop on landscape**: on `speech_60s_landscape.mp4` the control exists; dragging the video to the right edge (`dragVideo`) sends one `review_action { action: "crop_offset" }`.
6. **No crop on portrait**: the crop control does not exist.
7. **Pause and restore**: an edit while playing pauses and the preview plays again after.
8. **Undo after a failed edit**: `__offcutDev.failNextSetScene()`; "Undo this change" restores the previous state.
9. **Click a word in the transcript**: the preview seeks and pauses; the field opens with the text selected; Tab commits and opens the next word; Escape cancels.
10. **Click a caption word on the preview**: the Captions tab opens on that word.
11. **Replace all**: a word repeated three times edited once → the toast offers "Change all 2" → all three change; one undo restores all three.
12. **Undo and redo** with Ctrl+Z and Shift+Ctrl+Z across a word edit, a style change and a toggle.
13. **Drag the captions to the top**: snaps at Top (`data-caption-position` = 0.15), the visuals' zone below them; one `review_action { action: "caption_position" }`.
14. **Style cards and Highlight numbers**: three images within 400 ms; choosing Bold applies it; Highlight numbers off → the number no longer in the accent colour (`data-caption-colors`).
15. **Shortcuts**: Space plays and pauses outside a field and types a space inside one; ← and → step frames when paused.
16. **Accessibility**: `axe(page)` on the ready and editing states → zero serious or critical violations; every control reachable by Tab with a visible focus ring (D-192).

`MEDIA_SUITES` gains `review-edit.spec.ts`. The repeated-word clip for case 11: a fixture text through the dev hook that loads a transcript (never a lent clip), or the founder's clip whose transcript has a repeated word; record which.

#### Do Not Implement Yet

- `cancel.spec.ts`, `pipeline-preview.spec.ts` (Prompt 124).

### Existing Components To Reuse

- The helpers of Prompt 122; the fake API's analytics capture (V2); the dev hooks; the dev-only caption mirrors.

### Files To Create

- `web/tests-e2e/review-edit.spec.ts`.

### Files To Modify

- `web/playwright.config.ts`, `docs/v4/v4changelog.md`; source files only to fix bugs the suite finds (each named).

### Architecture Requirements

- Each case is self-contained (its own clip load).
- No pixel-reading for words: the dev/test mirrors are the source (§24 item 13).

### Data Types / Interfaces

None new.

### Implementation Requirements

#### 1. Analytics assertions
Read the fake API's received events; assert names and enum props only.

#### 2. Stability
Run the suite twice; a flaky case is fixed, not retried.

### Error Handling

None new.

### Concurrency / Async Requirements

Wait for the update rule to finish (`ui-store.save === "saved"` or the clip back in `ready`) before each assertion.

### Persistence Requirements

None committed.

### API Requirements

The fake API receives `review_action` with the new values.

### Security Requirements

No lent clip's words in a fixture or a log (SAB rule 9).

### Testing Requirements

#### Unit Tests
None.

#### Integration Tests
`review-edit.spec.ts`: 16 passed on D1, twice.

#### Failure-Path Tests
Cases 8 and 16.

### Validation Commands

```bash
pnpm --filter web exec playwright test --project=media tests-e2e/review-edit.spec.ts 2>&1 | tail -3   # 16 passed (keyed)
pnpm e2e:media 2>&1 | tail -3                                     # green (keyed)
```

### Acceptance Criteria

- [ ] Sixteen cases exist as listed and pass on D1 twice in a row.
- [ ] axe finds no serious or critical violation on the ready and editing states; every control is reachable by Tab.
- [ ] Bugs found are fixed and named; CI green.

### Agent Instructions

Before: read D-145 to D-147, D-169 to D-187, D-192, §20.4, G 16.3. During: write and run case by case. After: SAB gate plus the media run; commit `V4 P123 (S15): review-edit suite with axe`; push; read CI.

### Expected Result

Every control of the editor is proven in Chrome by the gestures and keys a creator uses, and the editor passes an accessibility scan.

---

## Prompt 124 (V4 34/38) — `cancel.spec.ts`, `pipeline-preview.spec.ts` complete, no dev hook in production, two full runs (S15 part 3)

**Effort: High.** Five cancel cases (two runnable without a GPU in CI), the long-task observer across processing and editing, the D-160 rewrites, the production-bundle check, and a 76-case media run that must pass twice.

### Objective

Write `cancel.spec.ts`, complete `pipeline-preview.spec.ts` (zero events, long tasks, statistics, the D-160 rewrites), finish `playwright.config.ts`, prove that no dev hook or mirror reaches `web/dist`, and run everything on D1 twice.

### Project State Before This Prompt

- Prompt 123 committed.

### Why This Comes Here

G M-16: 76 media cases on D1 twice, axe clean, a clean production bundle.

### Scope

#### Implement

- G 16.4: `web/tests-e2e/cancel.spec.ts`: `@no-gpu` **import cancel** (a large `ok_*` file → `idle`, no OPFS clip directory, one `job_cancelled { stage: "import" }`); `@no-gpu` **model cancel** (model absent; download cancelled → partial bytes kept; the next run resumes); **pipeline cancel during ASR** → `accepted`, "Process again" works; **export cancel** → `idle`, no temporary file in OPFS; **cancel timeout** (`__offcutDev.hangWorker("render")` → restarted after 2 s; the clip `accepted`). The live-frame counter returns to idle after each (INV-11).
- `pipeline-preview.spec.ts`: **zero events** (`speech_20s_noevents.mp4` → the Visuals tab shows `noEventsFound`, no item row; D-154); **long tasks** (`PerformanceObserver("longtask")` from drop to `ready` ≤ 100 ms each, excluding the first `WebAssembly.instantiate` per bundle (D-156); then ≤ 50 ms during a word edit, a style switch and a caption drag (D-193)); **statistics** (after a 5 s play, `data-frames` > 100 and `data-max-drift-ms` present); the D-160 rewrites of "Preview plays" (asserts the statistics too) and "Events" (`setScene` takes a plan; the `pipeline_done` counts unchanged).
- G 16.5: `MEDIA_SUITES` gains `cancel.spec.ts`; the `--list` counts: 76 in the media project plus 2 setup; the `@no-gpu` count V3's plus 2.
- G 16.6: `pnpm build`; grep `web/dist` for `__offcutDev`, `data-caption-text`, `data-caption-colors`, `data-caption-position` → clean; `scripts/check-hosts.mjs` gains those marker strings in its bundle scan so the build fails if one appears (G (b) 9).
- G 16.7: `pnpm e2e` 12; `pnpm e2e:media` 78 passed (76 + 2 setup), 1 skipped (D-78), 0 failed; **twice**.

#### Do Not Implement Yet

- CI changes beyond what the new `@no-gpu` cases need (Prompt 125).

### Existing Components To Reuse

- The helpers; the fixtures (`ok_*` large file, `speech_20s_noevents.mp4`); V3's `@no-gpu` tagging and the Windows job.

### Files To Create

- `web/tests-e2e/cancel.spec.ts`.

### Files To Modify

- `web/tests-e2e/pipeline-preview.spec.ts`, `web/playwright.config.ts`, `scripts/check-hosts.mjs`, `docs/v4/v4changelog.md`; source files only to fix bugs found.

### Architecture Requirements

- The `@no-gpu` cases need no GPU and run on the Windows runner (D-72, §19.4).
- `check-hosts.mjs` keeps its existing checks; it gains the four marker strings.

### Data Types / Interfaces

None new.

### Implementation Requirements

#### 1. Counts
Record the `--list` counts and both full runs' summaries in the entry.

#### 2. Long-task exclusions
Only the first `WebAssembly.instantiate` per bundle on the main thread, if any; nothing else is excluded.

### Error Handling

A long task over the limit is fixed by moving work behind a worker method (D-156's contingency), never by raising the limit.

### Concurrency / Async Requirements

None new.

### Persistence Requirements

OPFS assertions read through the page (as V3's cases do).

### API Requirements

`job_cancelled` assertions through the fake API.

### Security Requirements

The production bundle carries no dev hook, mirror or test key.

### Testing Requirements

#### Unit Tests
None.

#### Integration Tests
The whole media suite, twice.

#### Failure-Path Tests
The five cancel cases.

### Validation Commands

```bash
pnpm --filter web exec playwright test --project=media --list | tail -1          # 76 tests (plus 2 setup)
pnpm --filter web exec playwright test --project=media --list --grep @no-gpu | tail -1   # V3's count + 2
pnpm build && grep -rlE "__offcutDev|data-caption-text|data-caption-colors|data-caption-position" web/dist && echo LEAK || echo clean   # clean
node scripts/check-hosts.mjs                                      # exit 0
pnpm e2e:media 2>&1 | tail -3                                     # 78 passed, 1 skipped (keyed), run 1
pnpm e2e:media 2>&1 | tail -3                                     # same, run 2
```

### Acceptance Criteria

- [ ] `cancel.spec.ts` has five cases (two `@no-gpu`); `pipeline-preview.spec.ts` has the zero-event, long-task and statistics cases and the D-160 rewrites.
- [ ] 76 media cases pass on D1 twice (78 passed with setup, 1 skipped); axe clean.
- [ ] The production bundle is clean and `check-hosts.mjs` fails on any marker; CI green, the Windows job running the `@no-gpu` cases (G M-16).

### Agent Instructions

Before: read D-152, D-154, D-156, D-160, D-193, §19.4, §20.1, §20.4, G 16.4-16.7. During: suite by suite, then the two full runs. After: SAB gate plus the two media runs; commit `V4 P124 (S15): cancel suite, pipeline-preview complete, no dev hook in production`; push; read CI.

### Expected Result

The end-to-end proof of V4 is complete: 76 media cases, twice, on D1.

---

## Prompt 125 (V4 35/38) — CI: goldens on lavapipe, unreviewed snapshots refused, the guards seen to fire (S16)

**Effort: Medium.** Workflow lines specified exactly, log reading with `gh`, and three throwaway commits on `v4-build` reverted after.

### Objective

Complete `ci.yml` and `e2e-media.yml`, read the logs for every V4 line, prove three CI guards fire with scratch commits, revert them, and, if the founder chose so, move the Linux ffmpeg URL.

### Project State Before This Prompt

- Prompt 124 committed; the founder has answered V3 CL item 7 (move the URL now or not).
- `ci.yml` already installs Mesa and sets `INSTA_UPDATE=no` and `CI=true` (Prompt 101).

### Why This Comes Here

G Phase 17: CI must run everything a runner can and refuse an unreviewed snapshot before the merge.

### Scope

#### Implement

- G 17.1 rows 1-3: `ci.yml`: Mesa line and version print, `INSTA_UPDATE: "no"`, `CI: "true"` confirmed in the `cargo test` step's env, the golden artifact upload on failure; `e2e-media.yml`: nothing but what the new `@no-gpu` cases need; V3 CL item 7: if the founder chose to move it, change `FFMPEG_LINUX_URL` and `FFMPEG_LINUX_SHA256` (a version-named URL and its SHA-256, checked with `check-external-facts.mjs`) and close the item.
- G 17.2: push; read the log lines of G 17.2's table with `gh run view --log --job <id> | grep -E "golden_frames|ssim|llvmpipe|mesa|test result|pending"`; record the Mesa version.
- G 17.3: three scratch commits on `v4-build`, each pushed and its CI failure read, then `git revert` and push: a snapshot file changed by hand (`display_list_snapshots` fails with a snapshot assertion); a golden darkened (`golden_frames` fails with its SSIM); `mesa-vulkan-drivers` removed (`golden_frames` fails naming a non-software adapter or none). Green again after the reverts.

#### Do Not Implement Yet

- Measurements (Prompt 126).

### Existing Components To Reuse

- V3's workflows, `check-external-facts.mjs`, `gh`.

### Files To Create

None.

### Files To Modify

- `.github/workflows/ci.yml`, `.github/workflows/e2e-media.yml` (only if needed), `docs/v4/v4changelog.md`.

### Architecture Requirements

- CI never sets `OFFCUT_GOLDEN_UPDATE` and never runs the `screens` project.
- The deploy order is unchanged (server first).

### Data Types / Interfaces

None.

### Implementation Requirements

#### 1. The drills' history
Each scratch commit is titled `V4 P125 (S16, drill): …` and reverted by `git revert --no-edit`; the entry lists the three run URLs and the failure lines.

### Error Handling

A drill that does not fail is a missing guard: fix the workflow or the test until it fails, then revert the drill.

### Concurrency / Async Requirements

One drill at a time; wait for each run to finish.

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

No secret added; the ffmpeg URL carries no credential.

### Testing Requirements

#### Unit Tests
None.

#### Integration Tests
Both CI jobs green on the final commit.

#### Failure-Path Tests
The three drills.

### Validation Commands

```bash
gh pr checks 7                                                    # every check pass (final commit)
gh run view --log --job <ci job id> | grep -E "golden_frames|llvmpipe|mesa-vulkan|catalog_snapshot|display_list_snapshots|analytics" | head -20
```

### Acceptance Criteria

- [ ] The Linux job runs `golden_frames` (17 passed on `llvmpipe`) and the snapshot tests with nothing pending; the Mesa version is recorded.
- [ ] Each of the three guards was seen to fail in CI and was reverted; the final commit is green.
- [ ] V3 CL item 7 moved or explicitly left with the founder's answer recorded (G M-17).

### Agent Instructions

Before: read §19.3-§19.4, G Phase 17, the founder's answer on item 7. During: change, push, read; drill, read, revert. After: SAB gate; commit `V4 P125 (S16): CI runs goldens on lavapipe and refuses unreviewed snapshots`; push; read CI.

### Expected Result

CI guards everything V4 added that a runner can run.

---

## Prompt 126 (V4 36/38) — Measure: every reading of §21.1, the bench, BP §6.2's decisions (S17)

**Effort: Medium.** Runs of existing suites and the bench, numbers into one file; the decisions are pre-agreed.

### Objective

Fill every heading of `docs/v4/experiments.md` with question, method, numbers, date and decision, all labelled `d1`; commit the bench file; write BP §6.2's two decisions; note which (assumption)s were measured.

### Project State Before This Prompt

- Prompt 125 committed; CI green.

### Why This Comes Here

G Phase 18: the measurements are taken on the finished build; S18's copy-back replaces the measured assumptions in TS §39.3.

### Scope

#### Implement

- G 18.1, the table's rows: edit to preview (five runs of `preview-sync.spec.ts`'s case; each 90th percentile and the worst; ≤ 150 ms); style thumbnails (10 openings; ≤ 400 ms); hit test (100 calls while paused; ≤ 16 ms each); app shell with the font (`check-hosts.mjs`; ≤ 400 kB gzip; if over, D-180's contingency: system fonts, recorded); preview drift and late frames (five runs, the worst; ≤ 80 ms, ≤ 5%); first frame and seek (≤ 500 ms, ≤ 300 ms); downscale (`data-downscaled` after a 60 s play: `false` on D1); `build_scene` and `frame_at` (`cargo test -p offcut-scene --release -- --ignored scene_timing --nocapture`: ≤ 1 s, mean ≤ 2 ms; write `scene_timing` now if it does not exist yet, as an `#[ignore]` test in an existing scene test file); golden similarity (Prompt 125's log and D1's run, ≥ 0.98 each); `offcut_render.wasm` size (from Prompt 98's record and now).
- G 18.2: `BENCH_DEVICE=d1 BENCH_CLIP=portrait pnpm bench:device` (10 runs, detached); compare `detect_scene` and `render_encode` with `bench/results/d1-2026-10-19-portrait.json` (`detect_scene` within 10% of V3's plus about 30 ms; `render_encode` ≤ 90 s per 60 s, E-4); commit the new result file.
- G 18.3: the two decisions of §21.2 written with the readings that decided them ("drift within budget: no preview-file contingency", "Vello in a worker reliable: no Canvas2D backend", or the contingency taken).
- G 18.4: list in the entry every (assumption) of Appendix D that a reading measured (for Prompt 127's TS §39.3 copy-back).

#### Do Not Implement Yet

- The copy-back itself (Prompt 127).

### Existing Components To Reuse

- The three suites, `pnpm bench:device`, V3's bench file.

### Files To Create

- `bench/results/d1-<date>-portrait.json`.

### Files To Modify

- `docs/v4/experiments.md`, `docs/v4/v4changelog.md`; `crates/offcut-scene/tests/<existing file>.rs` only if `scene_timing` is added.

### Architecture Requirements

- Every reading labelled `d1` with D-114's caveat (a pass on D1 is not a pass on R1).
- A missed target takes its pre-agreed action from §21.1's "If it misses" column; the target is never moved.

### Data Types / Interfaces

None.

### Implementation Requirements

#### 1. One heading per reading
`Question:`, `Method:`, `Numbers:`, `Date:`, `Decision:` all filled; 14 `Decision:` lines.

### Error Handling

A missed target: record it, take the column's action in this prompt if it is small (for example the thumbnail size), or stop and report if it changes design (a preview-file contingency).

### Concurrency / Async Requirements

Bench and media runs detached (SAB rule 11), never at the same time.

### Persistence Requirements

The bench file committed (D-162).

### API Requirements

None.

### Security Requirements

The bench uses the portrait fixture only.

### Testing Requirements

#### Unit Tests
`scene_timing` (ignored by default).

#### Integration Tests
The suites as measurement runs.

#### Failure-Path Tests
None.

### Validation Commands

```bash
git ls-files bench/results | grep "$(date +%Y-%m)"               # the new bench file
grep -c "^Decision:" docs/v4/experiments.md                       # 14
grep -A1 "^Decision:" docs/v4/experiments.md | grep -c "^Decision: *$"   # 0 (none empty)
```

### Acceptance Criteria

- [ ] Every reading of §21.1 is in `experiments.md` with numbers and a decision, labelled `d1`; every target met or its pre-agreed action recorded.
- [ ] The bench file is committed and compared with V3's; BP §6.2's two decisions written.
- [ ] The measured assumptions are listed for the copy-back; CI green (G M-18).

### Agent Instructions

Before: read D-114, D-162, D-166, D-173, D-180, §21, Appendix D, G Phase 18. During: run, read, write. After: SAB gate; commit `V4 P126 (S17): measurements, the bench file and BP §6.2's decisions`; push; read CI.

### Expected Result

V4's performance is measured on D1 and every pre-agreed decision is written down.

---

## Prompt 127 (V4 37/38) — Spec copy-back, the final local gate, design review 2 prepared (S18)

**Effort: High.** The specs must say what V4 built, signature for signature, across TS, BP and PS; the final gate checks every exit box that code can prove; the visual QA must be done honestly before the founder's review.

### Objective

Copy V4's decisions back into the specs, write the "As built" notes, run the final local gate with every count and grep, and prepare design review 2 with G 19.3's visual QA list done and fixed.

### Project State Before This Prompt

- Prompt 126 committed; `experiments.md` complete.

### Why This Comes Here

S18 (G 19.1-19.3): the specs are the truth V5 starts from (§22.3), and the founder's exit review should see a build that has already passed its own QA.

### Scope

#### Implement

- G 19.1: `docs/technicalspec.md`: §24.2's list into the sections it names (TS §40.2; §40.4 with resolution in `validate`, `marks`, the plan limits; §40.5 with the `dst` rectangle, `mono_font`, the renderer's V4 support rule; §40.9 with the wire format fixed at V4; §10.1 `to_out_ms`; §10.6 `EditState`'s two fields; §14.2 `assemble`, `release`, `hitTest`, `styleThumbnails`, `PreviewStats`, `SceneReport.chunks`; §19.2-§19.4 V4's signatures and D-170's style table, closing V4 CL known issue 2; §13.3 the component table of §17.3; §36 self-hosted fonts allowed; §39.3 the measured values of Prompt 126); `docs/buildplan.md` (V4's 13 days and 19 steps; BP §6.3's exit list as §23 holds it; §24 item 10 to BP §9); `docs/product.md` (D-177's audience in PS §3, §4, §6; PS §12.2's style names Clean, Bold, Mono); `docs/v4/v4implementation.md` "As built" notes for every decision that changed while building (from the changelog), and the boxes of §1A ticked.
- G 19.2: the final local gate, every command of G 19.2 with its expected output (counts: Rust about 560 passed, 0 failed, ignored 3 + `scene_timing`; Vitest about 160; `pnpm e2e` 12; `pnpm e2e:media` 78 passed, 1 skipped; no `fetch(` outside the four files; no untagged TODO; no `\bV4\b` marker left in `web/src` or `crates`; no source file over 400 lines; the frozen check clean); the final counts into the changelog, none below Entry 1's.
- G 19.3: `pnpm build && pnpm screens`; G 19.3's visual QA list on both widths (4 px grid and alignment; hover, focus, pressed, disabled on every control by tabbing and hovering; empty, loading and error states; reduced motion: the pill still moves, the interface does not slide; no clipped text in the three styles and the longest error copy; contrast with axe and the picker on `--text-2`); fix what fails; write the QA record; copy the captures to `~/v4-review-2/` with a note.
- Pick the five goldens for Prompt 128 now (`ls fixtures/golden/*.png | shuf -n 5`) and record the list.

#### Do Not Implement Yet

- `gh pr ready`, the merge, the deploy checks, the founder's review, the tag (Prompt 128, Human).

### Existing Components To Reuse

- The changelog's records; `check-file-tree.mjs`; the visual QA list of D-191.

### Files To Create

- Outside the repository: `~/v4-review-2/*`.

### Files To Modify

- `docs/technicalspec.md`, `docs/buildplan.md`, `docs/product.md`, `docs/v4/v4implementation.md`, `docs/v4/v4changelog.md`, and any interface file the QA fixes.

### Architecture Requirements

- The copy-back writes what the code is, read from the code (signatures copied, not retyped from the plan).
- A QA fix that changes a token or a primitive is recaptured and listed.

### Data Types / Interfaces

None new.

### Implementation Requirements

#### 1. The 400-line loop
G 19.2's loop over every file V4 changed (`git diff --name-only 550ff8f -- crates web/src`), tests excluded; it prints "lengths checked" and no "TOO LONG".

#### 2. Markers
Every V4 marker of Entry 1's list is gone or re-tagged to a later version (`V5:`, `V6.1:`).

### Error Handling

A failing gate item is fixed before the review is prepared.

### Concurrency / Async Requirements

None.

### Persistence Requirements

None.

### API Requirements

None.

### Security Requirements

Screenshots stay outside the repository.

### Testing Requirements

#### Unit Tests
All.

#### Integration Tests
The final local gate.

#### Failure-Path Tests
None new.

### Validation Commands

```bash
node scripts/check-file-tree.mjs                                  # exit 0
git grep -nE "\bV4\b" -- web/src crates | grep -v "\.ttf"          # no output
git grep -n "TODO" -- crates web/src | grep -vE "TODO\(V[0-9]+(\.[0-9]+)?\)|V[0-9]+(\.[0-9]+)?:"   # no output
git grep -nE "\bfetch\(" -- web/src | grep -vE "net/http\.ts|net/asset-fetch\.ts|wasm/load-core\.ts|wasm/load-render\.ts"   # no output
pnpm e2e:media 2>&1 | tail -3                                     # 78 passed, 1 skipped (keyed)
```

### Acceptance Criteria

- [ ] TS, BP and PS say what V4 built (every item of §24.2); V4 CL known issue 2 closed; "As built" notes written.
- [ ] The final local gate passes with counts at or above Entry 1's plus V4's, recorded.
- [ ] The visual QA list is done on both widths with every failure fixed; the captures and note are in `~/v4-review-2/`; five goldens picked.
- [ ] CI green.

### Agent Instructions

Before: read §22.3, §23, §24, §24.2, D-191, G 19.1-19.3. During: copy-back from the code, then the gate, then QA; load a design skill for the QA if available. After: SAB gate; commit `V4 P127 (S18): spec copy-back, final gate, design review 2 prepared`; push; read CI; report that the build is ready for the founder.

### Expected Result

The specs describe V4 as built, the branch passes every local check, and design review 2 is ready.

---

## Prompt 128 (V4 38/38) — Merge, deploy, verify, design review 2, close-out, tag (S19)

**Effort: Medium.** A sequence of checks around the founder's merge, review and tag; the judgement is the founder's.

### Objective

With the founder: mark PR #7 ready and merge it (Human), watch the server-first deploy, verify the deployed editor on D1, hold design review 2 and the reading of five goldens (Human), read the analytics rows (Human), tick §23 and §22.3, write the closing entry, and have the founder push the tag `v4`.

### Project State Before This Prompt

- Prompt 127 committed and green; `pnpm e2e:media` all green on D1 on a keyed build (D-72).

### Why This Comes Here

S19: V4 ends deployed, reviewed and tagged; V5 must not start until this passes (G M-19).

### Scope

#### Implement

- G 19.4: `git push && gh pr checks 7` all pass; report "ready to merge" with the final counts. **Human:** `gh pr ready 7` and the merge. Then `export APP_URL=https://offcut-one.vercel.app`; `MERGE=$(git rev-parse origin/main)`; `gh run watch` (ci, e2e-media, deploy-api, deploy-web all success); `curl -s "$APP_URL/api/v1/healthz"` (200 and `$MERGE`); `node scripts/check-headers.mjs "$APP_URL"` (the 7 headers); `E2E_BASE_URL="$APP_URL" pnpm --filter web exec playwright test --project=non-media --grep @smoke` (1 passed).
- G 19.6: the deployed checks on D1: `crossOriginIsolated === true`; the font at `/assets/inter-latin-<hash>.woff2`, 200, `font/woff2`, `Cache-Control: public, max-age=31536000, immutable`; the sample clip: processing steps, `ready`, Clean with the sliding pill, the clip's visuals; edit, change all, undo (as G 13.5); caption drag to Top and a crop drag on a landscape clip; cancel during transcription → "Process again"; `rej_4k.mp4` → V3's message in the restyled card. **Human:** the Neon SQL of G 19.6; paste the rows; check enum props only.
- G 19.5 (**Human**): on the deployed editor at 1280 x 800 and 1920 x 1080: the founder takes the sample clip from drop to export alone while the agent notes every hesitation; the four questions of G 11.2 on the eight states; the five goldens picked in Prompt 127 ("Does each look as intended?"). Record answers, hesitations and requests: a non-cosmetic request becomes a V5 item; a cosmetic one is fixed on a small branch from `main` with its own pull request before the tag (the founder merges it).
- Rollback rule (G 19.6): if `healthz` does not report `$MERGE` within 15 minutes, or a deployed check fails, prepare `git revert -m 1 $MERGE` on a branch from `main` and a pull request for the founder; confirm the revert is deployed and behaves as V3; then fix on `v4-build` and repeat Prompts 127-128's checks.
- G 19.7: tick §23 and §22.3 in `v4implementation.md` (every `- [ ]` → `- [x]` with its evidence in the changelog); write the closing entry ("V4 closed: <date>, merge <SHA>, final counts, reviews passed"), committed through a small pull request if `main`'s protection requires one (the founder merges). **Human:** `git tag -a v4 -m "V4: the look and the editor" <deployed commit>` and `git push origin v4`. Then `git ls-remote --tags origin v4`.

#### Do Not Implement Yet

- Anything of V5.

### Existing Components To Reuse

- `check-headers.mjs`, the `@smoke` case, the deploy jobs of `ci.yml`, G 19.6's SQL.

### Files To Create

None.

### Files To Modify

- `docs/v4/v4implementation.md` (boxes), `docs/v4/v4changelog.md` (closing entry), and only the files a cosmetic review fix needs (on its own branch from `main`).

### Architecture Requirements

- The server deploys first (the analytics enum is new), then the web build (§13, G 19.4).
- The agent never merges, pushes to `main` or tags (D-115).

### Data Types / Interfaces

None.

### Implementation Requirements

#### 1. Order
Pre-merge green → Human merge → deploy watched → deployed checks → Human SQL → Human review 2 → fixes if any → boxes and closing entry → Human tag.

#### 2. The closing entry
Final counts (Rust, Vitest, Playwright, media), the merge SHA, the deployed commit, the review results with the founder's words, the goldens' answers, the SQL's summary (event names and enum props; no user data).

### Error Handling

The rollback rule above.

### Concurrency / Async Requirements

`gh run watch` on the `main` run; no other push to `v4-build` meanwhile.

### Persistence Requirements

None.

### API Requirements

The deployed API reports the merge commit at `/api/v1/healthz`.

### Security Requirements

The agent never queries the production database; the founder pastes rows. No production value is written to a file; `.env.deploy` stays local.

### Testing Requirements

#### Unit Tests
None new.

#### Integration Tests
`@smoke` on the deployed URL; the deployed checks of G 19.6.

#### Failure-Path Tests
The rollback rule, only if needed.

### Validation Commands

```bash
curl -s "$APP_URL/api/v1/healthz"                                 # 200, the merge commit (or the close-out commit after it)
node scripts/check-headers.mjs "$APP_URL"                         # exit 0
git ls-remote --tags origin v4                                    # one line with refs/tags/v4 (after the founder pushes it)
grep -c "^- \[ \]" docs/v4/v4implementation.md                    # 0
```

### Acceptance Criteria

- [ ] **Human:** PR #7 merged; ci, e2e-media, deploy-api, deploy-web all green on `main`; `healthz` reports the merge.
- [ ] The deployed checks of G 19.6 pass on D1; the font is an immutable hashed asset.
- [ ] **Human:** the Neon rows show `review_action`, `job_cancelled`, `client_error` with enum props only.
- [ ] **Human:** design review 2 passed with the founder's words recorded; five goldens answered.
- [ ] Every box of §1A, §22.3 and §23 ticked; the closing entry committed.
- [ ] **Human:** `v4` pushed on the deployed commit (G M-19).

### Agent Instructions

Before: read D-115, D-191, §22.3, §23, G 19.4-19.7, M-19. During: run what is yours, stop at each Human step and say exactly what is needed. After: no SAB gate on `main`; the closing entry follows the founder's merge path. Reply with the deployed state, every box's evidence, and anything carried to V5.

### Expected Result

V4 is deployed, verified, reviewed by the founder, recorded and tagged; V5 starts on its first morning with every precondition of §22.3 true.

---

# Cross-Prompt Contracts

| Contract | Owner → consumers | Shape | Compatibility / change rule |
| --- | --- | --- | --- |
| Baseline and counts | P91 → every gate | `BASE=550ff8f`; Entry 1: 461/0/3 Rust, 92 Vitest, 12 Playwright, 48 media | Counts only rise; the baseline moves only if the founder merges a hotfix from `main` (G 0.1) |
| TS §5 tree and the tree script's tables | P92 (+P110 `dev-hooks.ts`, P101 removals) → `check-file-tree` | Every V4 path; `ALLOWED_CRATE_DEPS` with `offcut-plan` and the dev-only `offcut-detect` edges; `PURE_CRATES` + `offcut-plan` | A new file or edge needs its line in the same commit |
| The frozen check and its three exceptions | P92, P94 → every gate | G 1.6's path list; `lib.rs` `mod scene_api;`, the two worker import lines, `program_identity.rs` | No other diff under the frozen paths |
| The identity reference | P93 → P98 (comparison form), P100 (proof), P101 (deleted) | Six JSON files, every 7th frame, V3's serialization | Never regenerated after P93; a difference is fixed in the switch |
| Split files | P94 → P98-P99, P112, P118 | `rpc.ts`/`rpc-worker.ts`, `video-source.ts`/`frame-decoder.ts`, `render.worker.ts`/`export-job.ts`/`preview-job.ts`, `session.rs`/`scene_api.rs` | Page side never imports worker side; job files never import the worker file |
| `OutMs`, `OutSpan`, `to_out_ms`, `TimeMap`, `SrcPos` | P95 → P98-P100, P112, V6.2 | §6.1-§6.2; identity only; validating `Deserialize` | Frozen (§22.1); `from_segments` is V6.2's |
| The plan wire format | P95 → P96-P99, V6.1 server, IndexedDB from V6.1 | `PLAN_FORMAT = 1`; every field and arm; adjacent tagging of `ItemParams`; the pinned one-item JSON | Frozen (D-119); additive fields only with serde defaults |
| `EditState` + `caption_position`, `highlight_numbers` | P95 → P104, P113-P116, V3's stored records | Serde defaults 1.0 and true | Frozen; V6.x appends with defaults |
| Catalog, `CATALOG_VERSION`, prompt and schema snapshots | P96, P97 → P106 (`catalog_complete`), V6.1 | Four rows; version-named snapshots | Rows appended; a prompt change bumps the version and adds snapshot files |
| `validate`, `ValidPlan`, `ValidItem.marks`, `DropReason`, `PlanConfig` | P96 → P98, P106, V6.x | Appendix C.1; twelve reasons; private construction | Frozen signature; reasons appended |
| `from_rules` | P96 → P99 (via the binding) | Events → `Detected` items | Values never changed by it |
| `SceneInput` v1.2, `Scene` methods (`frame_at(OutMs)`, `source_at`, `hit`, `chunk_ranges`, `summary`, `fonts`) | P98, P104, P115 → binding, goldens, V5-V6.x | §8.1 + `hit` + `chunk_ranges` | Frozen; V6.1 may append `assets` |
| `DrawCmd` order, `FontKey`, `FontSet` keys 0-4, `CropRect`, `VideoSlot`, `ImageKey` | P98, P105 → renderer, snapshots, V6.x | §8.2, §8.3, D-170 | Variants and keys appended only |
| `Theme`, `Role`, `FontRole`, `StyleSpec` (+ `shadow`), `CaptionMotion`, `ActiveTreatment` (+ `Pill`) | P98, P105 → captions, visuals, V6.5, V6.7 | §8.4, §8.5, D-170 | Fields frozen; members appended |
| `ZoneId`, `zone()`, `placement.rs` snaps and formula | P98, P104 → visuals, `preview-interact.ts` (mirrored snaps), V6.6 | D-138, D-171 | Rows appended; snap values have one Rust home and one named mirror |
| Readability constants and the break search | P103 → captions, V6.5 (per-preset limits) | `readability.rs` [ONLY]; DP between hard breaks | Costs tunable as values with a changelog line; the search fixed |
| The line-break table | P102 (+P103 reading) → P103-P104, every later scene change | 30 rows, Clean and Bold | A row changes only with a changelog line naming it and the reason |
| `Renderer::render(sources, list, fonts)`, V4 support rule, `RenderError::Unsupported` | P98 → binding, V6.5 | §9.1-§9.2 | Frozen signature; the rule widens in V6.5's body |
| `new_headless`, `render_to_image`, `RgbaImage`, golden names, `GOLDEN_SSIM_MIN` | P101, P107 → CI, V6.x goldens | §9.4; 16 names; 0.98 (parity 0.99) | Names frozen; the threshold never lowered |
| `assemble`, `silence_profile`, `release`, `prog48` | P100 → preview, V5 (export reads `prog48`), V6.2, V6.3 | §10, §14.1 | Frozen signature; identity postcondition in V4 |
| Binding exports (`rules_plan`, `set_scene` → `SceneReport { dropped, chunks }`, `source_at`, `hit`, `resize`; `assemble`) | P98, P100, P104, P112, P115 → workers | §11.1-§11.2 | Additive only |
| `RenderWorkerApi`, `AudioWorkerApi` | P99, P100, P113, P116 → use-cases | §14.1 (+ `SceneReport.chunks`) | Frozen at V4's end (§22.1) |
| Preview statistics `data-*`, dev-only `data-caption-*` mirrors | P112, P113 → P122-P124 | `data-frames`, `data-late`, `data-max-drift-ms`, `data-downscaled`; `data-caption-text/colors/position` | Names frozen; mirrors never in production |
| `__offcutDev` hooks | P110, P119 → P110 (screens), P120, P123, P124 | `openWordField`, `failClip`, `fakeExport`, `crash`, `failNextSetScene`, `hangWorker` | Dev builds only; `check-hosts.mjs` fails on them in `dist` from P124 |
| Tokens, interface font, primitives' props, icons | P108 (+P111 changes) → every interface prompt, V6.x | D-179 to D-181 | Token names frozen, values tunable at a review |
| `ui-store`, `history.ts` (`HISTORY_MAX`) | P109, P114 → panels, top bar | §16.1 | Additive |
| `review-update.ts` `update(change, mode, action)` | P113 (minimal), P114 (complete) → every edit use-case, V6.x controls | Appendix C.2 | Frozen rule; new controls are new use-cases |
| Copy keys of §17.5 and the 14 error entries | P109, P119 → every component | `messages.ts`; `COPY_PENDING` 14 shorter | Keys appended; INV-15 |
| `cancel-job.ts` scopes, `pool.cancel`, `CANCEL_TIMEOUT_MS` | P118, P119 → use-cases, `cancel.spec.ts` | D-152 | Frozen |
| `ReviewActionKind` + 5 | P120 → server allowlist, analytics rows | §13 | Members appended only |
| E2E helpers, `screens` project and script | P110, P122 → P123, P124, P127 | G 16.1; `--project=screens` | Additive; `screens` never in CI |

# High-Risk Implementation Areas

| Risk | Why it matters | Prompt responsible | Validation strategy |
| --- | --- | --- | --- |
| The contract switch moves a pixel | Every later "it changed nothing" claim would be false; V6.x would inherit a hidden difference | 93, 98, 99, 100 | Identity reference written by V3's code, compared with `f32` equality after a two-rule normalisation; fixed in the switch, never in the reference |
| A frozen file edited by a split | Breaks the V3 freeze silently | 94 | The frozen check with exactly three recorded one-line exceptions |
| `ItemParams` serde tag clash with `Detected(EventParams)` | A plan that round-trips wrong breaks V6.1's server contract | 95 | Adjacent tagging; pinned exact JSON; round trip of every arm |
| `validate` panics or keeps a bad item | A model plan (V6.1) could crash the render worker or draw a wrong number | 96, 97 | No `unwrap`/indexing; 21 rule cases; INV-22 case; trybuild for INV-21 |
| The prompt accepted unread | V6.1's model reads it; a bad instruction costs plan quality | 97 | Snapshots read before acceptance, quoted in the entry |
| Native `wgpu` backends in the browser bundle | Bundle size and a broken wasm build | 92, 101 | Dev-only features; wasm32 tree grep at every `Cargo.toml` change |
| Goldens differ across WARP and lavapipe | CI fails or the threshold gets lowered | 101, 107, 125 | Per-adapter folders if needed, never a lower threshold |
| Line-break table written to suit the search | The quality claim becomes circular | 102, 103 | Table committed before `readability.rs`; the founder reads ten rows; rows change only with a changelog line |
| A V3 caption expectation changed silently by the search or the redesign | Hidden regressions | 103, 105 | Each changed expectation listed with its decision and approved |
| Pill jumps or shows in a pause | The premium default looks cheap | 105, 107 | Unit tests at start, +45, +90 ms and in a pause; goldens looked at; checked moving |
| Watermark meets a caption at some placement | A Free export with overlapping text | 104, 106 | `layout_safe_area.rs` at three snap points over 93 scenes |
| Font served as HTML in production | The interface falls back or breaks | 108, 128 | Hashed `/assets/` file; deployed Network check |
| Interface built before the review | Rework of wired flows | 108-111 | No flow before P111's "go" |
| Edit to preview over 150 ms | "See it at once" fails | 104, 112, 115, 117, 122, 126 | Shaping cache, paused redraw, timing in dev and in `preview-sync.spec.ts` |
| A drag commits on every move | Hundreds of scene builds and analytics events | 113 | Commit on release only; one `review_action` asserted in P123 |
| Undo restores the wrong step | Users lose work | 114, 121, 123 | History pushed only after success; history tests; three-kind undo case |
| Throttle drops a feed line | The processing feed lies | 118, 121 | Feed lines bypass the throttle; fake-timer test |
| Cancel leaves a temporary file or a held buffer | Storage leak; audio kept (V3 §23 item 25) | 100, 119, 124 | `release()` on every early end; OPFS assertions in `cancel.spec.ts` |
| Dev hooks or mirrors in production | Test surface shipped to users | 110, 113, 119, 124 | Mode guard; `dist` grep; `check-hosts.mjs` markers |
| Analytics enum change misses the server | `review_action` rows rejected after deploy | 120, 125, 128 | Inline round-trip cases; server tests in CI; server deployed first |
| D1 is not R1 (D-114) | Budgets met on D1 may miss on a laptop | 126 | Every reading labelled `d1` with the caveat |
| Pushing scratch branches against the one-branch rule | Breaks the founder's workflow | 91 | Runner probe only with the founder's yes; else deferred to P101 |

# Architectural Invariants

1. **Contracts before looks.** No pixel changes before Prompt 100's proof (D-116); every seam S-1 to S-7 exists in V4, and every later version is additive (§22).
2. **One door to the plan.** Only `validate` builds a `ValidPlan` (INV-21); the scene draws nothing that did not pass it.
3. **Numbers come from words.** Every drawn number is parsed from the effective transcript, in `validate` only (INV-22, D-121); the scene and the page never parse a number.
4. **The catalog and the renderer agree.** Catalog kinds and drawn kinds are the same set (INV-23).
5. **The recording's timeline is not changed in V4.** Every map is the identity; `assemble` returns the cleaned voice; no cut, mix or non-identity map is constructed outside tests (INV-5, INV-10, D-164).
6. **Program time everywhere the user looks; recording time only at the decoder** (D-126).
7. **The video is a draw command; fonts a registry; colours a theme; motion data** (S-4, S-5); colours live in `theme.rs` only.
8. **Deterministic output.** Equal inputs give byte-identical display lists, with the shaping cache cold, warm or off (INV-7).
9. **Preview equals export** by construction and by test (D-172); the watermark is the one announced difference, decided by the profile only (INV-9).
10. **Edits are optimistic, local, reversible and fast** (D-147, D-173, D-184); nothing in the edit path awaits the network.
11. **The main thread does no media work** (INV-17); hit tests, thumbnails and scene builds run in the render worker (D-193).
12. **Every `VideoFrame` is closed** (INV-11), through play, seek, cancel and crash.
13. **All copy from `messages.ts`** (INV-15), including tooltips, labels and toasts; plain words for creators in general (D-177).
14. **Every state change through `transition()`** (INV-16); the machine tables are V2's.
15. **The browser contacts the app origin and the asset host only** (INV-1); V4 adds no request; analytics carry enums and booleans only (INV-18).
16. **One home per value** (Appendix D): plan limits in `config.rs`, break costs in `readability.rs`, placement in `placement.rs`, colours in `theme.rs`, styles in `styles.rs`, design values in `tokens.css`.

# Forbidden Implementation Shortcuts

- No visual value changed in Prompts 92-100; no reference file regenerated after Prompt 93; no normalisation rule added to make the proof pass.
- No line-break row edited to match the output; no cost tuned without the table passing; no V3 expectation changed without being named.
- No snapshot or golden accepted unread; no `INSTA_UPDATE` or `OFFCUT_GOLDEN_UPDATE` in CI; no SSIM threshold lowered.
- No number parsed in `offcut-scene` or in TypeScript; no second implementation of chunking, placement or emphasis in the page.
- No `TimeMap` other than the identity outside tests; no cut, mix, image or mask drawn (D-164).
- No native `wgpu` backend in a normal dependency; no `serde_json` schema code in the render bundle.
- No colour literal outside `theme.rs` (Rust) or `tokens.css` (CSS); no inline style; no component library; no font from a CDN; no file under `web/public/fonts/`.
- No interface flow wired before design review 1's "go".
- No commit of a drag or slider on `input`/`pointermove`; no history push before an update succeeds.
- No dropped feed line; no temporary export file left after a cancel; no audio kept in the worker after `assemble`.
- No dev hook or `data-caption-*` mirror in `web/dist`; no `screens` run in CI.
- No `unwrap`/`expect`/`panic!`/indexing in non-test Rust; no `any`/`!`/`@ts-ignore`/empty `catch`; no disabled lint; no skipped or weakened test.
- No edit to a frozen file beyond the three recorded one-line exceptions; no new lint edge without its reason.
- No committed `testclips/`, generated fixture, corpus clip, result, screenshot, Playwright report, `.env*` or `zz-` file; no test key in a file.
- No push to `main`, merge, tag, `gh pr ready` or production database query by the agent; no branch pushed other than `v4-build` (and `scratch/v4-adapter` only with the founder's yes).

# Requirement → Prompt Traceability

| Requirement (BP §6, TS §28 M1.4-M1.5, TS §40.9, the founder's brief) | Prompt(s) | Implementation | Validation |
| --- | --- | --- | --- |
| A `styles.rs` Clean, Bold, Tech (Mono) | 98, 105 | roles, motion, D-170 | snapshots, goldens, G 9.4 |
| A `list_reveal.rs`, `from_to.rs`, `keyword_pop.rs`, `number_reveal.rs` | 98, 106 | four arms | snapshots, goldens, `catalog_complete.rs` |
| A `framing.rs` complete | 106 | the offset used | `framing.rs` |
| A `watermark.rs`, `summary.rs` | 106 | D-143, D-144 | snapshots, `layout_safe_area.rs` |
| A tests and goldens | 101, 106, 107 | 16 goldens, snapshots | `golden_frames.rs` 17 on two adapters |
| A2 S-1 time map | 95, 98, 99 | `OutMs`, `TimeMap::identity` | `timemap_identity.rs` |
| A2 S-2 edit plan | 95, 96, 97, 99 | `offcut-plan`, rules plan through `setScene` | five plan test files |
| A2 S-3 numbers resolved once | 96 | `validate` V-5 | INV-22 case, word-edit snapshot, review-edit case 1 |
| A2 S-4 video as a command | 98, 101 | `DrawCmd::Video`, `video.rs` | crop and punch-in goldens |
| A2 S-5 fonts, theme, motion | 98, 105 | `FontSet`, `Theme`, `CaptionMotion` | `styles_use_theme.rs`, identity proof |
| A2 S-6 program audio | 100 | `assemble`, `prog48` | `program_identity.rs` |
| A2 S-7 review panel from plan items | 113-117 | update rule, four tabs | `review-edit.spec.ts` |
| B preview loop, video source, control-preview, player | 99, 112, 113 | seek, prefetch, resize, stats | `preview-sync.spec.ts` |
| C review UI and use-cases, `NO_EVENTS_FOUND` | 113-117 | editor, `PlanItemList` | `review-edit.spec.ts`, `pipeline-preview.spec.ts` |
| D rpc, pool, cancel, `ErrorPanel`, machine tests, E2E | 118-124 | throttle, cancel, crash, copy | `rpc.test.ts`, machine tests, `cancel.spec.ts` |
| BP §6.2 decisions | 126 | readings | `experiments.md` |
| Quality A: accurate transcription, fixed fast | 114, 115 | click to fix, change all | review-edit cases 1, 9-11 |
| Quality A: precise word timing | 98, 103 | `OutMs` identity; floor from silence only | identity proof; `line_breaks.rs` |
| Quality A: smart line breaks | 102, 103 | `readability.rs`, DP search | `line_breaks.rs`, the founder's ten rows |
| Quality A: highlights on the right words | 104, 105 | three emphasis layers, the pill | snapshots, goldens, review-edit case 14 |
| Quality A: consistent, draggable placement | 104, 113, 116 | `placement.rs`, drags, Framing tab | `placement.rs`, review-edit case 13 |
| Quality A: export equals preview | 104, 107 | one scene, one renderer | `parity.rs`, parity golden |
| Quality B: great default style | 105 | premium Clean | goldens, design review 2 |
| Quality B: smooth synced preview | 112, 122 | audio master, seek, resize | drift, first frame, seek cases |
| Quality B: click a word, see it at once | 104, 112, 115 | cache, paused redraw | edit-to-preview case ≤ 150 ms |
| Quality B: simple controls | 116 | Style, Visuals, Framing tabs | review-edit cases 3-6, 13-14 |
| Quality B: smart suggestions | 114, 116 | change all, suggested visuals, default style | review-edit cases 3, 11 |
| Quality B: reliable exports and recovery | 114, 119, 120 | undo, cancel, error cards, export pill | `cancel.spec.ts`, G 14.5 |
| Quality B: clean premium interface | 108-111, 127, 128 | tokens, primitives, two reviews | axe, design reviews 1 and 2 |
| Analytics: `review_action`, `job_cancelled`, `client_error` | 113-120, 128 | use-cases, enum | fake API assertions; Neon rows |
| Measurements of §21.1 | 98, 101, 104, 113, 115, 116, 122, 126 | readings | `experiments.md` |
| Spec copy-back (§24.2) | 92 (tree, graph), 127 | TS, BP, PS | `check-file-tree`; G 19.1 |
| Deployed, verified, reviewed, tagged | 128 | merge, deploy, checks | `healthz`, headers, `@smoke`, deployed checks, `ls-remote` |

# Final Coverage Audit

- [x] **Every BP §6 item and every seam S-1 to S-7 has a prompt** (table above), including the non-code items: the identity proof (100), the line-break reading (102-103), two design reviews (111, 128), the goldens' reading (128), the bench (126).
- [x] **Every file of §4 is named in a prompt:** workspace and deny (92); `offcut-types` (95); `analytics.rs` (120); `offcut-plan` and its tests (92, 96, 97); scene sources (98, 103-106, 115) and tests (93, 98, 102-104, 106); render (98, 101, 107); dsp (100); bindings (94, 98, 100, 104, 112, 115); `eslint.config.js` (99, 113, 114); `playwright.config.ts` (110, 122-124); `gen/` (95, 96, 120); `messages.ts` (109, 119); workers and RPC (94, 99, 100, 112, 113, 116, 118); loaders (99, 100, 112, 114); stores (99, 100, 109, 112, 114); use-cases (99, 100, 112-116, 119); interface (108-110, 112, 113, 115-117, 119); E2E (110, 122-124); goldens (101, 107); scripts (92, 119, 124); workflows (101, 125); docs (91, 92, 101, 126, 127, 128).
- [x] **Every decision is implemented:** D-113 (92, 101, 110, 127); D-114 (91, 126); D-115 (every prompt; 128); D-116 (93, 98-100); D-117 (93, 98, 100, 101); D-118 to D-120 (95, 96); D-121 (96); D-122 (92, 96, 98); D-123 (96, 97); D-124 (99); D-125 (95); D-126 (98, 99); D-127 (100); D-128 (100, 119); D-129 (99, 100); D-130 (98); D-131 (98, 106); D-132 (98, 105); D-133 (98); D-134 (91, 92, 101); D-135 (101, 107); D-136, D-137 (98, 105); D-138 (98, 104); D-139 (105); D-140, D-141 (106); D-142 (113, 116); D-143, D-144 (106); D-145 (106, 123); D-146 (99, 114); D-147 (113, 114); D-148, D-149 (112); D-150 (94); D-151 (118); D-152, D-153 (118, 119); D-154 (116, 124); D-155 (112, 113, 124); D-156 (124); D-157 (92, 97, 101, 125); D-158 (106); D-159 (109, 119); D-160 (98, 99, 106, 124); D-161 (99, 100: no `PipelineStage` added); D-162 (126); D-163 (95-97, 100); D-164 (95, 96, 98, 100, 127); D-165 (96, 97); D-166 (126); D-167 (102, 103); D-168 (103); D-169 (95, 104); D-170 (105, 107); D-171 (95, 104, 113, 116); D-172 (104, 107, 109, 113); D-173 (104, 112, 115, 122); D-174 (104, 113); D-175 (114, 115); D-176 (116; no doubtful-word flag anywhere); D-177 (102, 109, 111, 127); D-178 (109); D-179 (108, 111); D-180 (91, 108, 126, 128); D-181 (108); D-182 (115); D-183 (113); D-184 (114); D-185 (117); D-186, D-187 (116); D-188 (109); D-189 (119); D-190 (109, 119); D-191 (110, 111, 127, 128); D-192 (108, 115, 123); D-193 (115, 117, 124).
- [x] **Every reading of §21.1 has a prompt:** edit to preview (115, 122, 126); style thumbnails (116, 126); hit test (113, 126); app shell with the font (109, 126); drift and late frames, first frame, seek (122, 126); downscale (112, 126); `build_scene`/`frame_at` (104, 126); bench (126); golden similarity (101, 107, 125, 126); wasm size (98, 126); BP §6.2's decisions (126).
- [x] **Tests are incremental:** each Rust test file lands with its code (95-98, 100-107); Vitest with the robustness work (121); each E2E suite after the interface it checks (122-124); every prompt runs the same gate.
- [x] **Persistence, configuration, errors, security, performance, observability, deployment** are covered: serde defaults for V3's records (95); constants in their homes (Appendix D; 96, 103-105, 108, 118); the error mapping and copy (98, 100, 119); privacy and secrets (SAB 9-10; 110, 124); budgets (115, 122, 126); analytics (113-120, 128); CI and deploy (101, 125, 128).
- [x] **Documentation:** changelog every prompt; `experiments.md` (91, 101, 107, 126); spec copy-back (127); "As built" notes (127).
- [x] **Prompt dependencies are valid:** every prompt needs only earlier prompts, the V4 documents and the listed Human steps; the allowed parallel track (108-110) depends only on 92.
- [x] **Thirty-eight prompts (91-128); Prompt 128 yields a deployed, verified, reviewed, recorded and tagged V4** with every box of §1A, §22.3 and §23 ticked.

# Gaps settled by these prompts

Found while reading `v4implementation.md`, `v4buildguide.md`, the V4 changelog and `scripts/check-file-tree.mjs` together. Each is applied in the prompt named; none changes a decision of §2.

| # | Gap | Settled as | Prompt |
| --- | --- | --- | --- |
| 1 | G 0.5 runs the runner probe through a `workflow_dispatch` workflow on a scratch branch; GitHub runs `workflow_dispatch` only for workflows on the default branch, and pushing any branch but `v4-build` conflicts with D-115 | An `on: push` workflow limited to `scratch/v4-adapter`, pushed only with the founder's yes; otherwise the runner reading moves to Prompt 101, where `golden_frames` on lavapipe is the probe | 91, 101 |
| 2 | §19.2 says the tree script reads the TS §7 graph from the spec; it holds a hand-written `ALLOWED_CRATE_DEPS` that also counts `[dev-dependencies]` | The table gains `offcut-plan` and the scene and binding edges, plus the dev-only `offcut-detect` edges of `offcut-scene` and `offcut-plan`, each commented | 92 |
| 3 | G 2.1 lists the reference frames as "0, 7, 33, …" while saying "every 7th frame" | Every 7th frame: 0, 7, 14, … | 93 |
| 4 | G 16.3 lists eighteen case headings for `review-edit.spec.ts`; §20.1's total of 76 needs sixteen | Grouped into sixteen (Tab and Escape inside "click a word"; drag video inside "crop on landscape") | 123 |
| 5 | The dev hooks of G 10.8 and §20.4 have no file in §4 | `web/src/ui/dev-hooks.ts`, added to TS §5 with it; worker halves behind the same guard | 110, 119 |
| 6 | The transcript editor's chunk blocks (D-182) need the scene's chunking, which lives in Rust | `Scene::chunk_ranges()` returned as `SceneReport.chunks`, never a second implementation in TypeScript | 115 |
| 7 | G 14.4 puts "one `analytics_allowlist` case each" in the server's test, a frozen path | The five cases go in `analytics.rs`'s inline tests (which already assert each enum's list); `server/tests/analytics_allowlist.rs` stays frozen | 120 |
| 8 | The snap points live in `placement.rs` (Rust) and the page snaps drags | Mirrored once in `preview-interact.ts` as `CAPTION_SNAPS` naming its home, guarded by review-edit case 13 | 113 |
| 9 | G 3.1 changes the import line of `media.worker.ts` and `asr.worker.ts`, which G 1.6's frozen check lists | Three recorded one-line exceptions in the SAB's frozen check | 94 |
| 10 | Entry 0.4 says TS §5 already lists `v4buildguide.md`; it did not (the script does not check `docs/`) | Both `v4buildguide.md` and `coding-promptsv4.md` added to TS §5 in this file's commit (Entry 0.5) | - |
| 11 | G M-1 makes S1 one commit | Two commits: records (91), configuration (92) | 91, 92 |
| 12 | G Phase 5 is one milestone across eleven files in two languages | Rust side (98), TypeScript side (99); the media cases that reach `setScene` may fail between them (SAB rule 4) | 98, 99 |
| 13 | `scene_timing` (§20.2, D-166) has no file | An `#[ignore]` test inside an existing scene test file | 126 |
| 14 | §18 says goldens are written "at S7 and S8" | S7 and S9, as the guide | 101, 107 |
| 15 | D-160 names the V3 expectations V4 rewrites; the break search (D-167) and the Clean redesign (D-170) may change other inline caption expectations | Each listed in the entry with its decision as the reason; a search-caused change is approved before it is made | 103, 105 |
| 16 | The guide's TODO grep (`TODO\(V[0-9]+\)`) does not accept `V6.1:` | The grep accepts `V<major>.<minor>` tags | 127 |
