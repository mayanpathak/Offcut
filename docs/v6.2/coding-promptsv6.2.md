# Offcut V6.2 "AI edit plan" — coding-promptsv6.2.md (Prompts 183–212)

Source documents: `docs/v6.2/v6.2implementation.md` (written **§n**; its decisions are **D-n**, D-433 to D-500; its founder questions **F-n**, answered before Setup; its checkpoints **CP n**) and `docs/v6.2/v6.2buildguide.md` (written **G n.m**, **G item n** for an item of its "Read this first"). `TS`, `PS`, `BP` are the v1.5 specs; `V6 CL` is `docs/v6/v6changelog.md`, this release's changelog. These prompts were written after reading the code at `aef4a1a` on 2026-11-11.

**How to use (the founder's decision of 2026-11-11: carefully, by hand).** Paste the SAB, then one prompt, **into a fresh session each time** (`/clear`; D-499). Run **Setup, then 183 to 212, in order, one at a time; P187 is done inside P186 and P198 inside P199** (D-499), so there are 28 runs. Do not start a prompt until the one before has its commit and every acceptance box ticked. A prompt that finds its scope too large stops at a clean point, commits what is proven, and the rest becomes `P<nn>b`: never rush a step to fit a prompt. A checkpoint (CP) waits for the founder.

**The provider is DeepSeek, with thinking always off (D-500, the founder's decision of 2026-11-11).** Wherever a prompt below says Gemini, `gemini.rs`, `fixtures/gemini/` or `gemini_adapter.rs`, read DeepSeek, `deepseek.rs`, `fixtures/deepseek/`, `deepseek_adapter.rs`. DeepSeek's request, prices and terms are in D-500 and D-492's notes.

Commits are titled `V6.2 P<nn>: <title>`. Each prompt's **Effort** line is the reasoning effort to set: **High** for every prompt that touches a protected proof, a stored or sent format, the renderer, privacy, money or a third party's API; **Medium** for records and specified screens.

**Think first (every prompt).** Before the first edit, write a short plan in the scratchpad: the files to touch, the existing functions and tests that must not change (D-439, D-440), the tests to run before and after, and the one thing most likely to break. Run the tests that cover the area **before** changing anything, and note their result: a failure that was already there is not yours to hide, and one that appears after is. Read the plan again before the gate.

---

# Standard Agent Block (SAB) — prepend to every prompt

**Every terminal (Git Bash, never PowerShell or WSL)**

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut && set -a; . ./.env; set +a
export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:/c/Users/Mayan/AppData/Local/Microsoft/WinGet/Links:$PATH"
export BENCH_DEVICE=d1
BASE=b73fea5
git branch --show-current     # v6.2-build, always
```

Server tests need Postgres on port 9000 (a Windows service on D1). `gh auth status` shows a login. `$SCRATCH` is the session's scratchpad directory.

**Before modifying anything**

1. Read the plan sections and guide steps this prompt cites, every file it touches, and the previous prompt's entry in V6 CL. **Read only the cited § and D-numbers (Grep for them); never read `technicalspec.md` or the plan whole** (D-499).
2. Search before creating (`git grep -n "<name>"`). `offcut-types` holds the plan, events, edit state and ids; `offcut-plan` the catalog, `validate`, `prompt`; `offcut-scene` `build_scene`, `frame_at`, `video.rs`, `captions*.rs`, `events/`, `easing.rs`, `motion.rs` (the caption motion, not the camera); `offcut-render` `lib.rs`, `composite.rs`, `vello_backend.rs`; `offcut-wasm-render` `session.rs`, `scene_api.rs`; `offcut-dsp` `chain.rs`, `program.rs`; `server/src/transcribe/` (the pattern for `plan/`), `rate_limit.rs`, `config.rs`, `router.rs`, `account/`; `web/src/usecases/run-pipeline.ts`, `review-update.ts`, `restore-session.ts`; `persistence/{db,schema,opfs}.ts`; `workers/protocol.ts`; `net/api-client.ts`; `config/features.ts`; `tests-e2e/helpers/fake-api.ts`.
3. Verify that the previous prompts' outputs behave as this prompt assumes. If not, make the smallest compatible correction and report it.
4. **Precedence.** A decision D-433 to D-500 is binding, also where it refines TS or PS; D-1 to D-432 stand unless one of these replaces them. Otherwise TS wins over the plan and PS over both. If sources disagree in a way these do not settle, or a signature is missing: **stop and list it for the founder.**
5. **TS §41 binds.** Any protected proof that would change beyond DC-7, DC-8 (V6.2's part) and DC-16, any existing test beyond D-440's nine, any new runtime, browser host, dependency or paid service, any experiment whose row offers a choice, any change to what the product promises: **stop and ask with a recommendation.**

**During implementation**

1. Implement only this prompt's scope. A new file under `crates/`, `server/`, `web/`, `scripts/`, `fixtures/`, `verify/` or `.github/` that TS §5 does not list fails `check-file-tree.mjs`; add its row in the same commit, only for the files §4 names.
2. **Frozen:** G 0.4's check stays empty; G 0.4's additive check shows only the removed lines it names. **No existing golden or snapshot changes** (G item 3). No existing test case changes except D-440's nine, each only in its named prompt.
3. **Flags default off** (G item 1). Nothing sets `OFFCUT_FEATURES_ON` outside `build:e2e:flags`.
4. **Reopened contracts and declared changes are exactly:** DC-7, DC-8 (images in the overlay), DC-16; D-440's test changes; D-446, D-453, D-469, D-476, D-480, D-485 (REOPENED); D-467 (`run-pipeline.ts` step 12); the additive members of `protocol.ts`, `api-client.ts`, the WASM bindings. Say **REOPENED CONTRACT** or **DECLARED CHANGE** in the entry for each.
5. No source file over 700 lines above its test module. Rust non-test code: no `unwrap`, `expect`, `panic!`, slice indexing. TypeScript: no `any`, `!`, `@ts-ignore`, empty `catch`, or inline style. No disabled lint, no weakened or deleted test; the one `#[ignore]` is D-440 (9)'s. Dependencies: none new (§3.1).
6. User-facing text only in `copy/` (new strings in `copy/plan.ts`, D-491). `fetch` only in the allowed files. A temporary file is `zz-…` or `zz_…` and is deleted before the gate.
7. **Secrets.** Never print, log, commit or paste a value from `.env`, `.env.deploy`, a token, a cookie or a link. Never `cat` either file; never source `.env.deploy`. A script reads `PLAN_API_KEY` (and DeepSeek's `PLAN_API_KEY2` until it is renamed, D-465) from its environment by name. `bash scripts/v62-gate.sh` searches for both values before every commit. Never query the production database.
8. **What reaches the provider, DeepSeek (F-9, D-500).** Only in P183, P188, P191 (one call), P206 (the whole clip) and P211; only the 15 labelled transcripts of D-493 (`a01`, `w01`-`w14`; never `a02`) and the reference clip's transcript, as text. Never audio, a frame, or another clip's words. Each call listed in the entry (transcript, model, input/output/thinking tokens, status). Stay under the local cap (`AI_MONTHLY_CAP_USD=5`).
9. Never stage `testclips/**`, `testclips_private/**`, `fixtures/.gen*/**`, `corpus/**`, `web/test-results/**`, `web/dist*/**`, `.env*`. Long runs start detached and are read from a log tail; no foreground command over 10 minutes. **Trim test output** to its result lines (`cargo test -q … 2>&1 | grep -E "^test result|FAILED|panicked"`, Playwright's last lines); never read a whole log into the session. Append to the changelog with the Edit tool, never a heredoc holding backticks.
10. **Branch.** Commit on `v6.2-build` only. Push only where the prompt says. Never push to `main`, force-push, delete a branch or tag, or change Render, Vercel or Google settings. Ready, merge and tag are the founder's.

**When existing code disagrees with the plan.** If it blocks the prompt, fix the minimum and record why, with an "As read" note under the decision in the plan. If not, report it under "Found, not fixed".

**The gate.** G 0.5's tier, the new case alone first; then:

```bash
bash scripts/v62-gate.sh      # prints "gate: ok", or each failed check; from Setup on (D-499)
```

The script runs: the branch is `v6.2-build`; no `zz` file; no Gemini or Groq key pattern in the tree; `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` unset; G 0.4's frozen check and its no-modified-golden check; G 0.4's additive check (printing only removed lines not on its allow list); nothing private staged; `check-file-tree.mjs`; no `ignore = "V6.2` after P206.

Counts never fall below the previous entry's; the floor is D-438's. Add an entry of **about 30 lines** to V6 CL in the same commit: what changed, what differs from the plan, **REOPENED CONTRACT** and **DECLARED CHANGE** lines, every real Gemini call, every new golden or snapshot by name, counts (Rust, Vitest, `pnpm e2e`, media cases run), what is open, "Found, not fixed". Rewrite V6 CL's top tables when an item opens or closes. Commit with the trailer `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Reply with files changed, commands and results, deviations, and what the founder must do next.

---

# Who does what

| When | Founder | Agent |
|---|---|---|
| Before Setup | F-1 to F-9 and the review's points (answered 2026-11-11) | — |
| P184 | **CP 2a:** the 72 emoji and icon names | The list, with one line on each |
| P188 | **CP 1:** read ten annotated plans; choose the model; confirm the prices | TE-15, the plans, the table, a recommendation |
| P195 | **CP 2b:** the look of the 72 images | The contact sheet |
| P206 | **CP 3:** the whole reference clip beside your Submagic benchmark; the copy | The export and the captures |
| P211 | **CP 4:** design review 2 on the real product; a listen | The real runs and captures |
| P212 | Render's variables; ready; merge; tag; later the flags (§14.3) | The runs, the push, the live checks, the closing entry |

---

# Prompts

## Setup prompt — The documents, the key's name, the tooling, the floor

**Effort: Medium.**

**Objective.** `v6.2-build` holds the V6.2 documents; the key is named `PLAN_API_KEY`; the gate is one script; the faster build settings are in; the floor is read with them; a draft pull request lets CI run.

**Read first.** §1A, §2.1, §2.2 (D-495 to D-499), §4's "Setup's tooling", §17.1; G Phase 0; V6 CL Entries 39-40.

**Do.**

1. G 0.2: check `origin/main` is `b73fea5`, the branch is `v6.2-build` at `aef4a1a`, and the working tree holds only `docs/v6.2/`, V6 CL's Entry 40 and TS §5's rows.
2. G 0.3 (F-1 answered yes): rename the key's line in `.env` and `.env.deploy`; add `PLAN_ENABLED`, `PLAN_PROVIDER`, `AI_MONTHLY_CAP_USD` to `.env`. Print counts only.
3. G 0.8: the build settings (`[profile.dev]`, `[profile.test]`, `.cargo/config.toml`); one clean build of `offcut-render`'s tests and the server to prove they link; if `rust-lld` fails, remove `.cargo/config.toml` and say so.
4. G 0.9: `scripts/v62-gate.sh` and its TS §5 row; run it: `gate: ok`.
5. G 0.6: the three counts, with the new settings.
6. G 0.7: `experiments.md` and its TS §5 row.
7. V6 CL Entry 41: F-1 to F-9 and the review's answers as recorded (D-493 to D-499); the build settings; `target/`'s size; the floor as V6.2's baseline; "built by hand, one prompt at a time (D-434)".
8. Commit `V6.2 setup: the V6.2 documents, the gate, the floor`; push `origin v6.2-build`; the draft pull request.

**Acceptance.**

- [ ] `git show --stat HEAD` lists docs, `scripts/v62-gate.sh`, `Cargo.toml` (profiles only) and `.cargo/config.toml` (if kept).
- [ ] `grep -c '^PLAN_API_KEY=' .env .env.deploy` gives 1 and 1; `grep -c '^GEMINI_API_KEY=' .env .env.deploy` gives 0 and 0; no value printed anywhere in the session.
- [ ] Counts recorded: Rust 910 / 0 / 4, Vitest 650, `pnpm e2e` 57, or the entry says why not.
- [ ] Draft pull request open; its body ends with the attribution line.

---

## Prompt 183 — Read before building: Gemini, Vello's images, licences, the drift

**Effort: High.**

**Objective.** Every unknown V6.2's code depends on is read from the real thing, so that no contract is written on a guess: what Gemini accepts and returns, what it costs and what its terms say; how Vello draws an image; what the emoji and icon licences require; how a slow drift looks.

**Read first.** §2.5 (D-459), §2.7, §10.1, §15; G Phase 1; G items 11-15; `server/src/transcribe/groq.rs` (the shape of an adapter); `crates/offcut-plan/tests/snapshots/*@v1.snap`; `crates/offcut-render/src/vello_backend.rs`.

**Do.**

1. G 1.1: Gemini (models, terms, prices, the `@v1` texts on `a01` through both candidates, schema keywords, thinking, recorded shapes). Two to four real calls in all.
2. G 1.2: Vello's image fill on WARP (a `zz_` test, deleted after).
3. G 1.3: the licences; the rasterising tool.
4. G 1.4: TE-31's first look.
5. `experiments.md`: each reading with its date. Under each decision it settles (D-457, D-459, D-463, D-470, D-472, D-476), an "As read" note in the plan.

**Stop and ask** if: Gemini's paid-tier terms allow training on what is sent; neither model returns JSON that parses into `EditPlan`; Vello 0.11 has no image fill usable from `vello_backend.rs`; either licence needs more than a notice.

**Acceptance.**

- [ ] Every "(read in P183)" of the plan answered, with an "As read" note where it changes a line.
- [ ] Every Gemini call listed (transcript `a01`, model, tokens, status); the recorded shapes saved in `$SCRATCH` for P190, redacted.
- [ ] No product file changed (`git diff --stat` shows docs only); no `zz` file left.
- [ ] Commit `V6.2 P183: read before building`.

---

## Prompt 183b — The Gemini reads left by P183 (added in P183)

**Done 2026-11-11 as DeepSeek's reads instead (D-500; V6 CL Entry 43; `experiments.md` TE-15).** The founder chose DeepSeek, so the Gemini steps below were not run and are kept as history.

**Effort: Medium.**

**Objective.** The Gemini readings P183 could not take because the key's project was on Google's free tier: the Pro-class answer, and Flash with less thinking. Due before P188; P184 to P187 do not wait for it.

**Read first.** `experiments.md` TE-15's first read; D-457, D-459, D-463 and their "As read (P183)" notes; the scratchpad probe (`gemini-probe.mjs`, rebuild it from experiments.md if the session's scratchpad is gone).

**Before anything:** the founder confirms that the key's Google project has billing on (AI Studio shows the project's tier as paid, not "Free tier"). **No call is made until then.**

**Do.**

1. `gemini-3.1-pro-preview` with call 1's request: status (a 200 is the proof that billing took effect; a 429 naming `free_tier` metrics stops the prompt), latency, usage, parse, `validate` (a `zz_` test, as in P183, deleted after).
2. `gemini-3.8-flash` with thinking lowered (the `generationConfig` field P183's record shows for thinking: a level or a budget; read Google's docs for the exact name): latency, thinking tokens, parse, `validate`, and whether its items differ from call 1's.
3. Optional, if the founder asks: `gemini-4-argon` the same way.
4. `experiments.md`: the readings; D-459's `PLAN_THINKING` value proposed for CP 1.

**Acceptance.**

- [ ] Billing confirmed by the founder before the first call; every call listed (model, tokens, status, ms); `a01` only.
- [ ] Commit `V6.2 P183b: the Gemini reads left by P183`.

---

## Prompt 184 — The types

**Effort: High.**

**Objective.** The five kinds, their wire and resolved params, the asset ids and `Enhance` exist as types, and **nothing behaves differently anywhere**.

**Read first.** §2.4 (D-446, D-447), §2.9 (D-485), §6.1; `offcut-types/src/{events,plan,edit,summary}.rs`; every `match` on `EventKind` and `EventParams` (`git grep -n "EventKind::\|EventParams::" crates`).

**Do.**

1. `events.rs`, `plan.rs`: the members and arms of §6.1 at the end; `DropReason::{Density, HookMissing}` at the end.
2. `assets.rs`: `IconId` and `EmojiId` with their **final names** (D-495): propose about 48 emoji and 24 icons for talking-head shorts (reactions, money, time, growth, warning, idea, check, cross, arrows, fire, rocket…), each with the upstream file it will come from (Fluent Emoji, Lucide, F-4) so P195 can only fail on looks, never on existence. **CP 2a:** put the list to the founder; wait; apply changes. Names are snake case and never renamed after P186.
3. `enhance.rs`: `VoiceMode`, `Enhance`, `v5()`, `is_v5()`, `defaults(motion_flag)`. `EditState.enhance` with D-485's two serde attributes; `Default` gives `v5()`.
4. Every exhaustive `match` gains the arms as "not drawn" (`Ok(None)`, no command, no count). `pnpm gen:types`.
5. Inline tests: a `v5()` state's JSON equals the literal of `b73fea5`'s test; a record without the field reads `v5()`; `motion: true` round-trips; a V6.2 record with `enhance` read by a struct shaped as `b73fea5`'s `EditState` (a test-local copy) succeeds (§14.4).

**Acceptance.**

- [ ] `cargo test -p offcut-types -p offcut-plan -p offcut-scene -p offcut-render`: every existing case passes unchanged; the new ones pass.
- [ ] G 0.4: no golden or snapshot touched; the additive check shows only the enums' and `EditState`'s lines.
- [ ] `check-gen-clean.sh` clean after `gen:types`.
- [ ] The founder's yes to the 72 names recorded (CP 2a).
- [ ] Commit `V6.2 P184: the types`.

---

## Prompt 185 — Validation pieces

**Effort: High.**

**Objective.** `validate` knows how to check the new kinds and the extras, and DC-7 holds, while the catalog still has V4's four rows.

**Read first.** §2.4 (D-448 to D-455), §6.3; `offcut-plan/src/{validate,validate_params,catalog,config}.rs`; `offcut-plan/tests/validate.rs`.

**Do.**

1. `ParamTy::Index { max }`; the params checks of each new arm in `validate_params.rs`.
2. V-1 as `<=` (D-449). `ValidPlan.extras` and D-454's checks; D-455.
3. V-3 (D-450), D-452, V-9 (D-451) as functions of `limit`'s pass, each tested directly on `ValidItem`s.
4. `validate_v62.rs` (new) for what can be tested without the rows; the rest waits for P186.
5. D-440 (2): `case_20_…` and `the_non_item_fields_are_dropped_and_the_items_kept` and `validate.rs`' inline twin now expect the extras kept and cuts, chapters, music dropped. The entry quotes each changed assertion, before and after.

**Acceptance.**

- [ ] `cargo test -p offcut-plan`: every V4 case unchanged except D-440 (2)'s; `validate_rules_plans.rs` and `plan_from_rules.rs` unchanged and passing.
- [ ] A rules plan of the labelled set validates exactly as before (same kept items, same drops).
- [ ] Commit `V6.2 P185: validation pieces`.

---

## Prompt 186 + 187 — The catalog rows, the prompt and the schema, v2

**Effort: High.**

**Objective.** `CATALOG_VERSION = 2` with the five rows of §6.2, and what the model is told and must answer with, generated from them, read in full and pinned. One prompt, one commit (D-499).

**Read first.** §6.2; D-440 (1), (9); D-456, D-457 and P183's "As read" notes; `offcut-plan/src/{catalog,prompt}.rs`; `offcut-scene/tests/catalog_complete.rs`; TS §47's prompt additions.

**Do.**

1. The five rows, `Producers::Model`, the zones `"video"` and `"hook"`; the scene's `ZoneId` gains the two names (no drawing). The `Enum` params list P184's final names.
2. D-440 (1): the two inline tests keep every V4 assertion and gain the rows.
3. `validate_v62.rs`: each kind end to end through `validate` (valid, and each drop reason it can have).
4. `catalog_complete.rs`: its three cases get `#[ignore = "V6.2: the five kinds are drawn by P206"]` (D-440 (9)); nothing else in that file changes.
5. The prompt's new paragraphs (finished short; zooms; punch and emoji; the extras) and the extras' properties in the schema; if P183 found Gemini refuses part of the canonical schema, `output_schema_gemini()` from the same rows.
6. `cargo insta test --review`: the `@v2` files (and `output_schema_gemini@v2` if made), each read in full, accepted. The `@v1` files stay on disk unchanged.

**Acceptance.**

- [ ] `cargo test -p offcut-plan` green, including `catalog_snapshot.rs` on `@v2`.
- [ ] The rules plan's `catalog` is 2; every scene snapshot and golden unchanged (`cargo test -p offcut-scene -p offcut-render`; the plan's catalog number is not drawn).
- [ ] The entry quotes the prompt's new paragraphs and lists the schema's changes.
- [ ] Commit `V6.2 P186: the catalog rows, the prompt and the schema, v2`.

---

## Prompt 188 — TE-15 on Gemini, and checkpoint 1

**Effort: High.**

**Objective.** Whether Gemini plans well enough, on which model, at what cost and speed, read on 15 labelled transcripts (D-493); the founder chooses.

**Read first.** §15 (TE-15); G 2.5; F-5, F-9; P183's readings.

**Do.**

1. G 2.5's script and the validating example (scratch only). Both candidates, twice each, on `a01` and `w01`-`w14`: 60 calls.
2. The table in `experiments.md`: parse rate (first, after one retry), drops by kind and reason, items per plan, p50/p90 latency, tokens (input, output, thinking), cost per plan at the prices read.
3. **Ten plans as annotated transcripts** (D-496), five per model on the same transcripts: the words in order with each kept item inline where it lands (`…saved [ZOOM snap] $1,450 [🔥 under "money"]…`, the hook on top, dropped items struck through with their reason), in `~/v6.2-review/plans/`.
4. A recommendation (D-498): the Flash-class model unless the annotated plans show the Pro-class clearly better; the thinking setting; the prices. Latency: p90 ≤ 10 s targeted, 20 s the hard line.
5. **CP 1:** put the annotated plans, the table and the recommendation to the founder; wait. Record the answer; `.env` gains `PLAN_MODEL` and the prices (printed as names only).

**Stop and ask** (that is CP 1) in every case; if a pass line fails, offer: lower thinking, a shorter prompt, the other model, or stop.

**Acceptance.**

- [ ] TE-15 recorded with numbers; every call counted; total spend under the local cap.
- [ ] The founder's choice in the entry.
- [ ] Commit `V6.2 P188: TE-15 and the model`.

---

## Prompt 188b — Checkpoint 1's answers: the prompt revision and TE-15 on Flash again (added at CP 1)

**Done 2026-11-12 (V6 CL Entry 49; `experiments.md` TE-15).**

**Effort: Medium.**

**Objective.** Record CP 1's answers, and cut Flash's drops with one prompt revision, read on the same 15 transcripts.

**Read first.** `experiments.md` TE-15's full read and its recommendation; `~/v6.2-review/plans/`; `validate`'s rules for zooms (V-3, the 1.5 s gap), anchors and the event zone.

**Do.**

1. The revision in `prompt.rs` (and `punch_in`'s purpose in `catalog.rs`), every number from the catalog or `validate`: fewer zooms, each on its claim's words; one-word anchors written `{"start": w, "end": w + 1}`; a number's range exactly its words; the shared event zone. Read the `@v2` system prompt snapshot again and accept it.
2. TE-15 on `deepseek-flash` only: the same harness, request and 15 transcripts; at most 20 calls. The table beside P188's, the drops, the annotated plans in `~/v6.2-review/p188b/plans/`.
3. CP 1's answers in D-498, the revision in D-456, open item 37 closed.

**Acceptance.**

- [x] `cargo test -p offcut-plan --all-features` green; the `@v2` system prompt snapshot read and accepted.
- [x] 15 calls, every one listed; the founder's choice in the entry.
- [x] Commit `V6.2 P188b: checkpoint 1, the prompt revision, TE-15 on Flash again`.

---

## Prompt 189 — The tables and the configuration

**Effort: High.**

**Objective.** The database and the server's configuration are ready for `/plan`, and the deployed V6.1 binary would run against the migrated database.

**Read first.** D-461 to D-463, D-465; TS §52; `server/migrations/0001_init.sql`; `server/src/{config,db/mod}.rs`; `server/src/account/{delete,export}.rs`.

**Do.**

1. `0002_ai_usage.sql` as TS §52. Run it locally; run today's server tests against it before any other change (the V6.1 binary's view).
2. `db/ai_usage.rs` (increment on success, read the month), `db/ai_spend.rs` (add and return the month's total).
3. S-10: the two tables in delete and export; D-440 (6).
4. `config.rs`: D-465's variables, their defaults and failures; `render.yaml` with `sync: false`.

**Acceptance.**

- [ ] `cargo test -p offcut-api` green against the migrated database; `account_delete.rs` and `account_export.rs` cover the two tables.
- [ ] `PLAN_ENABLED=true` without a key stops the boot naming the variable (a test).
- [ ] Commit `V6.2 P189: the tables and the configuration`.

---

## Prompt 190 — The provider and the Gemini adapter

**Done 2026-11-12 as the DeepSeek adapter (D-500: read `deepseek.rs`, `fixtures/deepseek/`, `deepseek_adapter.rs` for the Gemini names below), with checkpoint 1's item-level parsing in `plan/parse.rs` (V6 CL Entry 50).**

**Effort: High.**

**Objective.** One file that knows Gemini, proven against a local fake built from real answers.

**Read first.** D-458, D-459; P183's recorded shapes; `server/src/transcribe/{provider,groq}.rs`; `server/tests/groq_adapter.rs`.

**Do.**

1. `plan/provider.rs` (`PlanProvider`, `ProviderRequest`, `ProviderOutput`, `PlanProviderError` without provider text) and an `UnconfiguredPlanProvider`.
2. `plan/gemini.rs`: the request exactly as read; the answer's text and token counts (with thinking); every failure to one error; nothing of the provider's text in a log.
3. `tests/fixtures/gemini/` (redacted) and `gemini_adapter.rs` with a local fake: success, 400, 429, 500, timeout, a blocked answer, no candidate, malformed JSON.

**Acceptance.**

- [ ] `cargo test -p offcut-api --test gemini_adapter` green; no real call.
- [ ] `git grep -nE "AIza[0-9A-Za-z_-]{35}"` empty; the fixtures hold no key or account identifier.
- [ ] Commit `V6.2 P190: the provider and the Gemini adapter`.

---

## Prompt 191 — The route, quotas, the cap; push 1

**Effort: High.**

**Objective.** `POST /api/v1/plan` as §7.2, with every failure a test, and one real local call.

**Read first.** §7; D-460 to D-464; `server/src/{router,rate_limit}.rs`; `server/src/transcribe/route.rs`; `server/tests/{transcribe_route,log_redaction,no_media_routes,rate_limit}.rs`.

**Do.**

1. `offcut-api-types/src/plan.rs` (§7.1); `gen:types`.
2. `plan/{route,quota,spend}.rs`; the router entry; **a bearer required, 401 before the body without one (D-494)**; the rate groups `PlanIp` and `PlanUser`; no anonymous quota; D-440 (7).
3. `plan_route.rs`, `ai_cap.rs`, `ai_quota.rs`; `log_redaction.rs`' plan case; `no_media_routes.rs`' plan route (64 kB, 415s).
4. One real local call with the reference transcript (F-9): status, latency, items, the log line (no content). Listed in the entry.
5. Tier 3 (server and Rust parts). **Push 1** (not waited on).

**Acceptance.**

- [ ] §7.3's cases pass; every existing server test unchanged except D-440 (6), (7).
- [ ] The real call's log line shows counts only.
- [ ] Commit `V6.2 P191: the route, quotas and the cap`; pushed.

---

## Prompt 192 — Flags, the request, the store

**Effort: High.**

**Objective.** The client can ask for a plan and keep it, behind `ai_plan`, without touching the pipeline yet.

**Read first.** D-443 to D-445, D-466, D-469; `config/features.ts`; `vite.config.ts`; `package.json`; `net/api-client.ts`; `persistence/{db,schema}.ts`; `persistence/sources-repo.ts` (the pattern).

**Do.**

1. `features.ts`: `ai_plan`, `motion`, defaults false; `build:e2e:flags`' list.
2. `api-client.ts`' `requestPlan` (additive); `request-plan.ts` and its test (each outcome; no user edit in the body; the timer).
3. `schema.ts`' `plans` (structure version 3), `db.ts`' opener for version 3 (as `openSourcesDb`), `plans-repo.ts` and its test (1 → 3 and 2 → 3 keep every record; the clip path still opens version 1).

**Acceptance.**

- [ ] Vitest green; the new tests pass; `pnpm check` clean.
- [ ] `dist/`'s bundle has `ai_plan:!1, motion:!1` (`pnpm build` and a grep of the bundle).
- [ ] Commit `V6.2 P192: flags, the request, the store`.

---

## Prompt 193 — The pipeline chooses the plan

**Effort: High.**

**Objective.** With `ai_plan` on, a clip uses a model plan when one arrives and validates, and the rules plan otherwise; with it off, the pipeline is exactly today's.

**Read first.** D-467, D-490, §8.2, §8.3; `usecases/run-pipeline.ts` and its test; `usecases/restore-session.ts`; `crates/offcut-wasm-render/src/scene_api.rs` (`set_scene`'s report).

**Do.**

1. **First**, before any change: the call-sequence case in `run-pipeline.test.ts` (flag off: the worker calls and their order). It passes on today's code.
2. The change of §8.2; how drops reach the use-case (§8.1's "read in P193"), as an addition to the binding.
3. Restore from `plansRepo`; `regenerate-plan.ts` and its test; the settings switch (local, `meta`).
4. `plan_result` (D-440 (5)); the privacy page's description of it.

**Acceptance.**

- [ ] The call-sequence case passes before and after; every other existing case of `run-pipeline.test.ts` unchanged.
- [ ] New cases: model plan chosen; each fallback; a fully dropped model plan; restore makes no request.
- [ ] Commit `V6.2 P193: the pipeline chooses the plan`.

---

## Prompt 194 — The list, the text extras, the copy

**Effort: Medium.**

**Objective.** A person sees and controls what the plan did.

**Read first.** §12, D-491; `ui/components/{PlanItemList,StylePanel}.tsx`; `ui/layout/SidePanel.tsx`; `copy/messages.ts`, `copy/long.ts`.

**Do.**

1. `copy/plan.ts` with §12's drafts; `messages.ts` gains the import only.
2. `PlanItemList`: the new kinds' names and switches. `TextExtrasPanel`: hook, caption, hashtags, each with Copy. The settings page's switch and line.
3. `pnpm screens` states; axe; captures to `~/v6.2-review/ui/`.

**Acceptance.**

- [ ] axe 0 on every new state; `review-edit.spec.ts` unchanged and passing on `dist/`.
- [ ] Captures listed in the entry.
- [ ] Commit `V6.2 P194: the list, the text extras, the copy`.

---

## Prompt 195 — The image bundle, and checkpoint 2b

**Effort: High.**

**Objective.** The images for the 72 names approved in P184 (D-495), licensed and embedded.

**Read first.** D-447, D-470, D-471, D-495; P183's licence reading; P184's name list; `offcut-scene/src/fonts.rs` (the embedding pattern).

**Do.**

1. The script in `scripts/` that fetches each name's upstream file at a pinned version and writes a 160 px PNG; `manifest.json` with SHA-256s; `LICENSES/`.
2. A test that `IconId` and `EmojiId` equal the manifest, name for name. **The names do not change here** (D-495); if an upstream file cannot be used, choose another image for the same name.
3. `ImageSet::embedded()`.
4. The render binary's size before and after (release build).
5. **CP 2b:** the contact sheet to the founder; wait; swap images, never names.

**Stop and ask** if the binary grows by more than 600 kB.

**Acceptance.**

- [ ] The founder's yes recorded; the manifest matches the files.
- [ ] Size growth recorded.
- [ ] Commit `V6.2 P195: the image bundle`.

---

## Prompt 196 — The renderer draws images

**Effort: High.**

**Objective.** `DrawCmd::Image` drawn natively, with every existing frame unchanged.

**Read first.** D-472, D-473, §9; P183's Vello reading; `offcut-render/src/{lib,composite,vello_backend}.rs`; `tests/golden_frames.rs`.

**Do.**

1. `Renderer::set_images`; decode once, premultiply; the overlay's `Image` arm; a missing key is `Unsupported`.
2. `composite::video_of` accepts `Image` after the video (D-440 (3)).
3. `image_basic` golden: the synthetic source, three emoji at three sizes, one at opacity 0.5, one across the canvas edge. Written once, read by eye at 200%.

**Acceptance.**

- [ ] `cargo test -p offcut-render`: every old golden passes untouched; `image_basic` passes on WARP.
- [ ] `video_pass.rs` and the compositor's video draw: empty diff.
- [ ] Commit `V6.2 P196: the renderer draws images`.

---

## Prompt 197 — Images in the browser, TE-30; push 2, waited on

**Effort: High.**

**Objective.** The browser draws the same image list as the native renderer, and CI's lavapipe agrees.

**Read first.** D-474, §9.1; `offcut-wasm-render/src/session.rs`; `workers/render.worker.ts`; `config/env.ts` (`DEV_HOOKS`).

**Do.**

1. The session takes the images at creation (additive); the worker passes them.
2. A dev-only hook draws `image_basic`'s list; `image-parity.spec.ts` reads the canvas and compares with the golden by the same SSIM (TE-30).
3. Tier 3. **Push 2 and wait for CI.** If lavapipe fails the golden, stop and ask.

**Acceptance.**

- [ ] TE-30: SSIM ≥ 0.98 on D1, recorded.
- [ ] CI green on the pushed head, with `golden_frames` on lavapipe.
- [ ] Commit `V6.2 P197: images in the browser`; pushed; CI read.

---

## Prompt 198 + 199 — Bezier easing and the camera drift

**Effort: High.**

**Objective.** The curves every V6.2 motion uses, exact and tested beside the old ones; then every shot drifts slowly by rules, and with Motion off every frame is today's. One prompt, one commit (D-499); the easing is written and tested first.

**Read first.** D-475 to D-477, D-497, §10.1; `offcut-scene/src/{easing,anim,motion,video,lib,framing}.rs`; P183's TE-31 look.

**Do.**

0. `easing.rs`: `CubicBezier`, `bezier()`, the three named curves; tests: endpoints exact, monotonic over 1,000 points, known values to 1e-4, Newton and bisection agree. `Easing` and `ease()` unchanged.
1. `camera.rs` (pure) and `tests/camera.rs`.
2. `video::Track` with the camera track; `src` from the scale, centred, clamped; `build_scene` reads `edit.enhance.motion`.
3. Tests: `none()` equals today at every frame of a 60 s clip; scale 1.0 gives today's `src` bit for bit; the clamp.
4. `motion_mid` golden; TE-31 recorded.

**Acceptance.**

- [ ] `Easing` and `ease()` unchanged (additive check).
- [ ] Every existing snapshot and golden unchanged (they use `EditState::default()`, so `v5()`).
- [ ] Commit `V6.2 P199: bezier easing and the camera drift`.

---

## Prompt 200 — PunchIn

**Effort: High.**

**Objective.** The plan's zooms, on the same track as the drift, capped.

**Read first.** D-478, D-477; `offcut-scene/src/events/mod.rs`; P199's track.

**Do.** `events/punch_in.rs` (layout only; it draws nothing itself); the track's keys; the product of scales capped at 1.35; `punch_snap` and `punch_push` goldens; cases for overlap with the drift and the release.

**Acceptance.**

- [ ] Old goldens and snapshots unchanged; `punch_in_110.png` (V4's hand-made list) unchanged.
- [ ] Commit `V6.2 P200: PunchIn`.

---

## Prompt 201 — Glow and the Glow style

**Effort: Medium** (D-499; guarded by the old goldens).

**Objective.** A soft glow behind caption text, in a fourth style, with the three styles unchanged.

**Read first.** D-479, D-480 (F-3); `offcut-scene/src/{styles,captions_draw,theme}.rs`; `offcut-types/src/edit.rs` (`StyleId`); `ui/components/{StylePanel,StylePicker}.tsx`; `usecases/style-thumbnails.ts`.

**Do.**

1. `StyleSpec.glow`; the strokes in `captions_draw.rs` (or `captions_emphasis.rs` if the file would pass 650 lines).
2. `StyleId::Glow` at the end; its `StyleSpec` and theme; the picker's fourth entry and thumbnail; `copy/plan.ts`' name.
3. `glow_style` golden; display-list snapshots for Glow (new files).

**Acceptance.**

- [ ] Clean, Bold and Mono goldens and snapshots unchanged; `styles_use_theme.rs` passes with Glow.
- [ ] Commit `V6.2 P201: glow and the Glow style`.

---

## Prompt 202 — Emphasis

**Effort: Medium** (D-499; guarded by the old goldens).

**Objective.** Colour, underline and highlight on one word, drawn by the captions.

**Read first.** D-481; `captions_draw.rs`' `Emphasis` and the `KeywordPop` path (D-169).

**Do.** `events/emphasis.rs` (layout to the captions' emphasis list); the three treatments; goldens `emphasis_colour`, `emphasis_underline`, `emphasis_highlight` in Bold; cases: a hidden word is not emphasised, a word in two chunks only in its own.

**Acceptance.**

- [ ] Old goldens and snapshots unchanged.
- [ ] Commit `V6.2 P202: emphasis`.

---

## Prompt 203 — Punch words

**Effort: High.**

**Objective.** One word alone, in capitals, at 1.9×, with the readability rules kept, and no other clip's chunking changed.

**Read first.** D-482, D-452; `offcut-scene/src/{captions,readability}.rs`; `tests/line_breaks.rs` (the frozen table).

**Do.** The chunk split around a punch word (or `captions_punch.rs`); the size and capitals; `punch_word` golden; cases: the reading floor, two punch words 6 s apart, one at the clip's last word.

**Acceptance.**

- [ ] `line_breaks.rs` and every chunking snapshot unchanged.
- [ ] Commit `V6.2 P203: punch words`.

---

## Prompt 204 — Emoji and icons

**Effort: High.**

**Objective.** The first real lists with images: an emoji under or beside its word, an icon on a disc.

**Read first.** D-483; P197's proof; `offcut-scene/src/{placement,safe_area,hit}.rs`; `testing::bounds` (D-440 (4)).

**Do.** `events/{emoji,icon_pop}.rs`; the `Pop` entry on `OUT`; `Beside`'s room test and fallback; `bounds` learns `Image`; goldens `emoji_under`, `icon_pop`; `layout_safe_area.rs`' new cases.

**Acceptance.**

- [ ] Old goldens and snapshots unchanged; every image inside the safe area at every placement.
- [ ] Commit `V6.2 P204: emoji and icons`.

---

## Prompt 205 — The hook title

**Effort: High.**

**Objective.** A lockup that fills the width for 2 s, or nothing when it cannot fit.

**Read first.** D-484, D-453; `offcut-scene/src/{layout,fonts}.rs`.

**Do.** `lockup.rs` (pure) and `tests/lockup.rs`; `events/hook_title.rs`; captions not drawn under it; goldens `hook_short`, `hook_long`; an impossible hook drawn as nothing and not counted.

**Acceptance.**

- [ ] Old goldens and snapshots unchanged.
- [ ] Commit `V6.2 P205: the hook title`.

---

## Prompt 206 — Everything together, and checkpoint 3; push 3

**Effort: High.**

**Objective.** The five kinds together are deterministic, at parity, affordable per frame, and approved by the founder's eye **on a whole clip** (D-496).

**Read first.** §10.5, §13.1; D-440 (9), D-496; `offcut-scene/tests/{catalog_complete,determinism,parity,display_list_snapshots}.rs`.

**Do.**

1. Remove D-440 (9)'s ignores; `catalog_complete.rs` green.
2. New cases in `determinism.rs`, `parity.rs`, `display_list_snapshots.rs` with all five kinds on the reference transcript.
3. The frame cost (release build) against V5's readings.
4. Captures (G Phase 6's list) to `~/v6.2-review/look/`; §12's copy beside them.
5. **The whole clip** (D-496): `dist-flags/` built locally, the local API with the real key (F-9: the reference clip's transcript only), signed in by the founder through the local API's magic link (the founder is present for CP 3; if local mail is not set up, ask before anything else); the reference clip processed with its real Gemini plan, Motion on, the Glow style; exported; the MP4 and the plan as an annotated transcript in `~/v6.2-review/cp3/`, beside the founder's Submagic clips in `sh2clips/documents/submagic_clips/` (never copied into the repository).
6. **CP 3:** wait for the founder. Changes become `P206a`… before P207.
7. Tier 3. **Push 3** (not waited on).

**Acceptance.**

- [ ] `git grep -n 'ignore = "V6.2' crates` empty.
- [ ] The frame cost under 1.5× V5's, or the founder decided.
- [ ] The founder's go recorded.
- [ ] Commit `V6.2 P206: everything together`; pushed.

---

## Prompt 207 — The levelled voice in the DSP

**Effort: High.**

**Objective.** `level_only` and `assemble`'s voice choice, with the chain and the identity proofs untouched.

**Read first.** D-487; `offcut-dsp/src/{chain,loudness,limiter,program}.rs`; `tests/program_identity.rs`; `offcut-wasm-core/src/audio_api.rs`.

**Do.** `level.rs`' `level_only`; `level_only.rs` tests; `assemble`'s `voice` with `Clean` as the default; the WASM entries (additive); `program_identity.rs`' new case.

**Acceptance.**

- [ ] `run_chain`'s diff empty; `program_identity.rs`' old cases pass unchanged.
- [ ] Commit `V6.2 P207: the levelled voice`.

---

## Prompt 208 — The Enhance panel

**Effort: High.**

**Objective.** Voice cleanup and Motion as instant, undoable, remembered switches, and the export honours them.

**Read first.** D-485 to D-489, §11; `workers/audio.worker.ts`; `persistence/opfs.ts`; `usecases/{review-update,start-export}.ts`; `state/history.ts`.

**Do.**

1. `lvl48` after `runChain` in the worker; `opfs` path; written beside `out48`.
2. `EnhancePanel.tsx`; `set-enhance.ts` and its test; `meta.enhanceDefault`; settings' reset.
3. The export's voice; `export_done`'s three fields (D-440 (5)).
4. The switch time on D1 (read and recorded).

**Acceptance.**

- [ ] Vitest green; the switch inside 150 ms or recorded and put to the founder.
- [ ] Commit `V6.2 P208: the Enhance panel`.

---

## Prompt 209 — Enhance end to end, DC-16; push 4

**Effort: High.**

**Objective.** The panel proven in the browser, and the identity of V5 proven with the flags on.

**Read first.** DC-16; D-440 (8); `tests-e2e/flags-identity.spec.ts`; the dev hooks.

**Do.** The dev hook that sets `Enhance::v5()`; `flags-identity.spec.ts`' case 1 uses it, case 2 added; `enhance-panel.spec.ts`; `FLAG_SUITES`; Tier 3. **Push 4** (not waited on).

**Acceptance.**

- [ ] Both on `dist-flags/`, twice each.
- [ ] Commit `V6.2 P209: Enhance end to end`; pushed.

---

## Prompt 210 — The AI plan end to end

**Effort: High.**

**Objective.** The plan's behaviour and its network rules proven in the browser with a fake API.

**Read first.** §13.4, D-468; `tests-e2e/helpers/fake-api.ts`; `fixtures/` conventions.

**Do.** `fake-api.ts`' `plan` option (default absent, so every existing suite is unchanged); `fixtures/plan/`; `plan-fixtures.ts`; `ai-plan.spec.ts`; `FLAG_SUITES`; Tier 3; V6.1's flag suites still pass in `dist-flags/` (D-444).

**Acceptance.**

- [ ] `ai-plan.spec.ts` twice on `dist-flags/`; the existing suites on `dist/` unchanged.
- [ ] Commit `V6.2 P210: the AI plan end to end`.

---

## Prompt 211 — The real product, and checkpoint 4

**Effort: High.**

**Objective.** The whole release on the real API, read for speed, cost and look, and approved by the founder.

**Read first.** §15 (TE-15's product read, E-11); F-9; G Phase 8.

**Do.**

1. The local API with the real key; `dist-flags/` served locally; the reference clip five times: plan latency, tokens, cost, items, drops.
2. E-11: `plan_result` and `export_done` in the local analytics sink, fields only.
3. Captures to `~/v6.2-review/product/`.
4. **CP 4:** the founder runs it, listens to Voice cleanup off, and says go; changes become `P211a`….

**Acceptance.**

- [ ] Readings recorded; every call listed.
- [ ] The founder's go recorded.
- [ ] Commit `V6.2 P211: the real product`.

---

## Prompt 212 — Copy-back, the full runs, merge, tag

**Effort: High.**

**Objective.** The release is documented as built, fully tested once, merged dark and tagged.

**Read first.** §14, §19, §20; G Phase 8.

**Do.**

1. Copy-back (§20), each edit marked "(V6.2, as built)".
2. Tier 4 (the full media run detached, about 40 minutes; `FLAG_SUITES` once).
3. **Push 5 and wait for CI.**
4. The founder: Render's variables (`PLAN_ENABLED=false`), ready, merge. The agent: the `main` run, §14.2's checks. The founder tags `v6.2` (or the agent at their word).
5. V6 CL's closing entry; the plan's §19 ticked with evidence.

**Acceptance.**

- [ ] Every §19 box ticked or named as the founder's.
- [ ] Commit `V6.2 P212: copy-back and close`; pushed; merged by the founder; tagged.
