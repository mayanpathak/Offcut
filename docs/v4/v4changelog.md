# v4changelog.md - record of every change made while building V4

**What this is.** A record of every change made while building V4: what was done, what differs from the plan, the guide or the prompts, how it was checked, and what is still open. It is opened early, before V4's implementation plan, build guide and prompts exist, because the v1.2 spec revision of 2026-10-21 changes what V4 builds. When V4's build step S1 runs, it keeps Entry 0 and adds its baseline as Entry 1.

**How it is kept.** One entry per prompt, or per change made outside a prompt. Entries are in date order, oldest first; new entries go at the end. Every commit that changes the repository carries its entry. The two tables "Open items" and "Known issues" are rewritten when an item opens or closes.

**References.** `PS §n` is a section of `product.md`, `TS §n` of `technicalspec.md`, `BP §n` of `buildplan.md`, `V3 CL item n` an open item of `v3changelog.md`.

---

## Open items
| # | Item | Owner | Version |
|---|---|---|---|
| 1 | **Launch date.** V6.1-V6.7 now sit between V6 and V7 (BP §8A). The 21 Dec 2026 launch holds only if V6.x fits in the two weeks V1-V3 gained; the founder re-sets M2 and the launch when V6.7 closes (PS A-2, TS §39.3 item 32) | Founder | V6.7 |
| 2 | **AI provider (TE-15).** Free-tier limits, structured output, latency and data-use terms decide the provider and the plan quotas. If the only fitting free tier trains on inputs, the founder decides between a paid tier, disclosing it, or making the AI plan opt-in (PS §17, TS §37) | Founder + V6.1 | V6.1 |
| 3 | **Starting plan quotas** (anonymous 3/day, Free 30/month, Creator 300/month soft) and the default "AI edit plan: on" are (assumption) values chosen in this revision, not founder numbers; TE-15 replaces them (PS §11, TS §40.7) | Founder | V6.1 |
| 4 | **Brand kit is Creator-only** in exports (Free sees it in preview). Chosen in this revision from the founder's "this is what makes people stay subscribed"; confirm or change (PS §11, TS §40.10) | Founder | V6.7 |
| 5 | V3 CL item 7 (the Linux ffmpeg URL, before about 2026-11-27) carries over unchanged | Founder | V4 |
| 6 | ~~**Open a draft pull request `v4-build` → `main`**~~ Done: PR #7 is open as a draft (closed in Entry 1) | Founder | Any time |

## Known issues
| # | Issue | Where | Version |
|---|---|---|---|
| 1 | ~~`AudioWorkerApi` (TS §14.2) is described as frozen, and v1.2 adds `assemble` to it in V4 (TS §40.6, §40.9 S-6). V4's implementation plan must record the protocol change as a REOPENED contract, as V3 did for `FeedLine`~~ Closed in Entry 2: recorded as REOPENED, D-127; closed by P100 (row 12) | TS §14.2 | V4 |
| 2 | TS §19.2, §20.2 and §21.3 still show the v1.1 signatures (`events` in `SceneInput`, `crop()`, `render(frame, crop, rotation, list)`, `to_time_ms`). Each section now opens with a v1.2 note pointing to §40.5 / §40.9; V4's copy-back rewrites the signatures themselves when the code exists | TS §19-§21 | V4 |
| 3 | **WARP on D1 needs DXC for Vello.** With wgpu's fallback shader compiler (FXC, used when no DXC is found) the Vello render crashes WARP (`STATUS_ACCESS_VIOLATION` in `device.poll`); with `dxcompiler.dll` 1.8.2502.11 of the Windows SDK on `PATH` it renders exactly (Entry 1). `golden_frames.rs` must get DXC on Windows: the SDK's DLL found by path, or `wgpu`'s `static-dxc` dev feature (its licence read by `cargo deny`); recorded as a decision in that prompt | `offcut-render/tests/golden_frames.rs` | P101 |
| 4 | `insta` brings `tempfile` → `getrandom` 0.4, and `tempfile` is not a wrapper of `deny.toml`'s `getrandom` ban: `cargo deny check bans` fails until it is added (dev-only path). Likewise the `offcut-detect` ban lists only `offcut-wasm-render`: the dev edges `offcut-scene → offcut-detect` and `offcut-plan → offcut-detect` need `offcut-scene` and `offcut-plan` as wrappers when they are added (Entry 2). **Scene's `offcut-detect` wrapper added in Entry 3**; `tempfile` waits for the first crate that takes `insta`, `offcut-plan`'s wrapper for its `offcut-detect` edge | `deny.toml` | P97 (plan: `insta`, `offcut-detect`), P98 (scene: `insta`) |
| 5 | **The wasm32 tree grep cannot see a misplaced native feature.** `wgpu` 30 gates `wgpu-core` and every native backend off `wasm32`, so `vulkan` on the normal `wgpu` dependency never reaches the browser bundle and G 1.3's wasm32 grep prints nothing for it; it shows in the host normal tree. The grep's `^windows` also matches `windows-sys`/`windows-link` of the host proc-macro `ts-rs-macros`; read the tree with `-e normal,no-proc-macro` (Entry 1). Prompt 92's drill confirmed it (Entry 2); the CI step of §19.1 must read the host normal tree, `cargo tree -p offcut-render -e normal,no-proc-macro --prefix none \| grep -E '^(ash\|d3d12\|windows\|gpu-alloc[a-z-]*) v'`, besides the wasm32 one | G 1.3, G 1.7, §19.1's CI step | P125 |
| 6 | **REOPENED CONTRACT** `offcut-types/src/units.rs`: `OutMs`, `OutSpan`; `FrameIdx::to_out_ms` replaces `to_time_ms` (D-125) | `offcut-types` | P95 |
| 7 | **REOPENED CONTRACT** `offcut-types/src/edit.rs` `EditState`: `caption_position`, `highlight_numbers` with serde defaults (D-169, D-171) | `offcut-types`, `web/src/gen` | P95 |
| 8 | **REOPENED CONTRACT** `offcut-scene/src/display_list.rs`: `FontId` becomes `FontKey`; `DrawCmd` gains `Video` and `Image` (D-130, D-132) | `offcut-scene` | P98 |
| 9 | **REOPENED CONTRACT** `offcut-scene/src/styles.rs`: style colours become theme roles, `CaptionMotion` (D-136) | `offcut-scene` | P98 |
| 10 | **REOPENED CONTRACT** `Renderer::render(sources, list, fonts)` (D-133) | `offcut-render` | P98 |
| 11 | **REOPENED CONTRACT** `web/src/workers/protocol.ts`: `rulesPlan`, `setScene(plan)` → `SceneReport`, `previewSeek(OutMs)`, `ClockSync(OutMs)`, `assemble`, `release`, `hitTest`, `styleThumbnails`, `PreviewStats` (D-124, D-126, D-127, D-128, D-174, D-186) | `web/src/workers` | P99, P100, P113, P116 |
| 12 | **REOPENED CONTRACT** `AudioWorkerApi`: `assemble`, `release` (D-127, D-128; known issue 1) | `audio.worker.ts`, `protocol.ts` | P100 |
| 13 | **REOPENED CONTRACT** `web/eslint.config.js` `MINTING`: `control-preview.ts` mints `OutMs` (D-126) | `web/eslint.config.js` | P99 |
| 14 | **REOPENED CONTRACT** `web/eslint.config.js` use-case edges: the four edit use-cases → `review-update.ts`, `review-update.ts` → `control-preview.ts` (§4) | `web/eslint.config.js` | P114 |
| 15 | **REOPENED CONTRACT** `offcut-api-types/src/analytics.rs` `ReviewActionKind`: five members (§13); the server rebuilds | `offcut-api-types`, `web/src/gen` | P120 |
| 16 | TS §9 (the review call chain: "ReviewPanel shows …", `<EventToggleList …>`) and the TS §13 component table still name `ReviewPanel` and `EventToggleList`, which TS §5 now replaces with `SidePanel` and `PlanItemList` (Entry 2) | TS §9, §13 | P127 (copy-back) |
| 17 | ~~Record "identity reference frozen at <SHA of Prompt 93's commit>" (the SHA is not known inside its own commit, Entry 3); from then on `git diff --stat <SHA> -- crates/offcut-scene/tests/snapshots/v3_reference` must print nothing until Prompt 101 deletes the files~~ Closed in Entry 4: frozen at `09b1085` | `docs/v4/v4changelog.md` | P94 |

---

## Entry 0 - 2026-10-21 - Spec revision v1.2: the AI edit plan (outside a prompt)

**Why.** The founder decided on 2026-10-21 that Offcut ships as an AI-planned, Submagic-class editor before launch: an LLM reads the transcript (text and timings only; video and voice stay local) and writes an edit and render plan that drives intelligent pause and silence trimming, background music with ducking, kinetic captions, video inside shapes, shape transitions, furniture (progress bar, chapter markers, hook title card, lower third, end card with CTA), the brand kit, the list builder, cards, number moments, emphasis picks, and text extras (hook title, chapter names, post caption, hashtags). The founder also asked that V4, V5 and V6 be written for this plan so that V6.1 meets no blockers. This entry records the specification change; no code changed.

**What changed.**

| File | Change |
|---|---|
| `docs/product.md` (v1.1 → v1.2) | Positioning rewritten; revision note v1.2; §0 note; A-2 (dates after V6.x); §1 pipeline and wedge; principles 2, 3, 4 and 7 rewritten (numbers are pointers; toggles and cut restores; media never leaves, transcript leaves only for the plan and can be switched off; cuts are plan suggestions the user can restore, never inside a word, nothing cut without a plan); §9.1 path; §9.2 rows (music, cloud LLM, B-roll, brand kit, timeline editing); §9.2.1 replaced by "What Offcut does to your recording"; §9.5 seven P0 rows V6.1-V6.7 and the V6.x scope rule; §9.6-§9.8; journey J6, J8, J11; §11 pricing rows and "the one real per-clip cost" (plan quotas); §12.1, §12.3-§12.7; new §12.8 "The AI edit plan"; §15 diagram; §16 data model (PlanUsage, EditPlan, MusicTrack, BrandKit); §17 analytics rule and edit-plan row; §18 two new risks; §19 E-11; §20.1 checklist (cuts, plan validation, music, brand kit, privacy); §21 items 3, 4, 10, 11 updated, item 14 (B-roll) added |
| `docs/technicalspec.md` (v1.1 → v1.2) | Status and precedence; timeline principles rewritten (`TIMELINE_INTEGRITY`); §0 "Changes in v1.2"; §1 overview, diagram (`/plan`, AI provider), backend limits; §2 rows (`server` may use `offcut-plan`; new `offcut-plan` row); §5 tree (`v4/v4changelog.md`, buildplan line); §10.1 `OutMs`, `to_out_ms`; §10.5, §10.6, §10.8 notes; §18 model note; §19, §20, §21 v1.2 notes, duration rule, render-cache key; §22.1 `POST /plan`; §24.1 AI provider host; §25.1 P-1, P-5 rewritten, P-14 added; §25.2 assertions 3, 4, 4b; §25.3 rows; §28 steps M1.9-M1.15; §35 INV-2, INV-5, INV-9, INV-10 rewritten, INV-21 to INV-24 added; §36 rows replaced; §37 TE-15, TE-16, E-11; §38 TDR-4 amended, TDR-7 replaced, TDR-18 to TDR-21 added; §39.1 rows; §39.3 items 29-33; **new §40 Edit Plan Architecture** (two timelines, plan types, capability catalog, validation rules V-1 to V-8, scene and renderer contracts, program audio, the `/plan` route, client flow, the seams S-1 to S-10 that V4-V6 build, the V6.x releases, what stays forbidden) |
| `docs/buildplan.md` (v1 → v1.2) | Header and v1.2 paragraph; §0 table rows V6.1-V6.7 and "Dates after v1.2"; §1.2 tags; §1.4 lines never crossed rewritten and extended; V4 §6.1 block A2 (seams S-1 to S-7) and exit items; V5 seams S-8, S-9; V6 goal (S-10, freeze moved to `v6.7`); **new §8A V6.1-V6.7** (order, reasons, build, demo, exit per release); V7 and V8 depend on V6.7; V7 E-2 notes feed E-11; V8 demo; V10 checklist (cuts, privacy) |
| `docs/v4/v4changelog.md` | Created with this entry |

**Decisions taken in this revision that the founder has not stated** (each is an open item above): the starting plan quotas, "AI edit plan" on by default, brand kit Creator-only in exports, V6.x release order V6.4 → V6.5 → V6.6 among the visual releases, filler-word cuts off by default, the cut limits (150 ms minimum pause, 40% maximum removed), and that the plan request sends the ASR transcript, not the user's word edits (P-1 keeps edits local; validation applies them).

**What V4 must now build that the v1.1 plan did not** (BP §6.1 A2, TS §40.9): `OutMs` and an identity `TimeMap`; the `offcut-plan` crate with the rules producer, validator, catalog and generated prompt; `QuantityRef`; `DrawCmd::Video` and `DrawCmd::Image` with the renderer drawing in list order; `FontKey`/`FontSet`, `Theme`, `StyleSpec.motion`, a zone table; `program::assemble` (identity) with the preview playing `prog48`; a review UI generic over plan items.

**How it was checked.** `node scripts/check-file-tree.mjs` (the new changelog is listed in TS §5). No code changed, so no test count changed.

**Open.** Items 1-5 above; known issues 1-2.

---

## Entry 0.1 - 2026-10-21 - `v4implementation.md` written (outside a prompt)

**What.** `docs/v4/v4implementation.md`, the file-by-file plan for V4, written against spec v1.2 and V3 as built, in the structure of `v3implementation.md`. It covers:
- BP §6 (M1.4, M1.5) together with the seams S-1 to S-7 of TS §40.9
- 54 decisions, D-113 to D-166
- a 17-step build order over ten working days: contracts first, then the identity proof (D-116, D-117), then the look, the workers, review, robustness, tests, CI, measurement and deploy
- the file-by-file hand-over to V5, V6 and V6.1-V6.7 (section 22.2)

TS §5 lists the new file.

**Decisions that refine the spec** (copied back at V4 S16, section 24):
- D-119: the plan's wire format is fixed at `PLAN_FORMAT = 1` in V4.
- D-120, D-121: rule-produced items carry the detector's own values (`ItemParams::Detected`), and number resolution happens in `validate`, not in `build_scene`.
- D-122: `offcut-plan` depends on `offcut-text`, and the scene depends on `offcut-plan`.
- D-130: `Video.dst` is an axis-aligned rectangle.
- D-136: `Theme` gains `mono_font`.
- Section 8.7: `ValidItem.marks` carries FromTo's second time.

**Decisions that reopen V1-V3 contracts:**
- D-125: `FrameIdx::to_time_ms` becomes `to_out_ms`.
- D-126: `ClockSync`, `previewSeek` and the `MINTING` entry move to `OutMs`.
- D-127, D-128: `AudioWorkerApi` gains `assemble` and `release`.
- D-130: `DrawCmd` gains `Video` and `Image`.
- D-132: `FontId` becomes `FontKey`.
- D-133: the `Renderer::render` signature changes.
- D-136: style colours become theme roles.

**Found while reading the code for the plan.** Four files have no room for V4's additions: `render.worker.ts` (372 lines), `rpc.ts` (377), `video-source.ts` (399) and `session.rs` (381). D-150 splits each before anything is added. V3 §23 items 24 and 25 are closed by D-150 and D-128.

**How it was checked.** `node scripts/check-file-tree.mjs` passes. Decision numbers run D-113 to D-166 with no gap and no duplicate. No code changed.

**Open.** Items 6 and 7 above (the founder's, before S1).

---

## Entry 0.2 - 2026-10-22 - One branch for V4: `v4-build` (outside a prompt)

**Decision.** The founder chose one branch for all of V4: "exactly like V3 we will have another branch for V4 and we will commit to that branch only" (option B).

**What was done.**
- The local branch `v4-spec` was renamed `v4-build`.
- `origin/main` (`550ff8f`, "Landing hero") was merged into it (`81474bf`). That brings in `docs/v3/v3changelog.md` and one line of `web/src/copy/messages.ts`; there were no conflicts.
- `v4-build` was pushed to `origin`.
- Spec v1.2 reaches `main` with V4's merge, not before.

**Changed in `v4implementation.md`.**
- 1A items 2 and 3 are ticked and rewritten: spec v1.2 is on `v4-build`, and the code baseline is `550ff8f`.
- D-115 is rewritten: one branch, the agent pushes every commit to `v4-build`, and `main`, the merge and the tag are the founder's.
- Section 3.3 is updated.

**Open items.** Item 6 (merge `v4-spec`) and item 7 (the push rule) are closed by this decision. A new item 6 asks for an optional draft pull request.

**How it was checked.** `node scripts/check-file-tree.mjs`; `git merge-base v4-build origin/main` gives `550ff8f`.


---

## Entry 0.3 - 2026-10-22 - `v4implementation.md` revised for caption quality and the editor's design (outside a prompt)

**Why.** The founder's brief of 2026-10-22 sets V4's bar higher: better captions, a better final video and a better user experience than Submagic, for content creators in general (not only technical creators), with a clean, premium and user-friendly interface.

**What changed in the plan.**
- **Header.** A quality-bar section maps every line of the founder's two lists (A: caption intelligence; B: editor experience) to the decision that answers it.
- **Two new decision groups.**
  - Section 2.9, caption intelligence (D-167 to D-176):
    - phrase-aware line breaks by a least-cost search (no article or preposition stranded, balanced lines)
    - a 700 ms reading floor taken from silence only
    - three emphasis layers (spoken word, stressed word, numbers)
    - a redesigned premium default (Clean: Inter 800, soft shadow, the spoken word on a sliding highlight pill)
    - draggable caption placement with snap points, the visuals following
    - preview-export parity tests
    - edit to preview within 150 ms (shaping cache, redraw while paused)
    - hit testing
    - "Change all N" for repeated mis-hearings
    - honest scope for suggestions
  - Section 2.10, editor experience and design (D-177 to D-193):
    - the audience and plain-word copy
    - the editor layout: top bar, preview, four tabs
    - a dark design system in tokens, Inter as the interface font, primitives and icons
    - the transcript editor; direct manipulation on the preview
    - undo, redo and visible autosave; keyboard shortcuts
    - style cards rendered from the clip itself; the Visuals and Framing tabs
    - processing, export and error experiences
    - two founder design reviews on captured screens; accessibility (axe); interface performance
- **Section 17** is replaced by a full interface and interaction specification: principles, screens and states, components, interaction details, copy, styles.
- **Updated to match:**
  - scope (1.1, 1.2); the file tree (new scene files, the editor's layout, primitives and use-cases, the font, `history.ts`)
  - the build order: 19 steps over 13 days, with S8 caption intelligence, S10 design system and shell, S10A design review 1, S19 design review 2
  - the protocol (`hitTest`, `styleThumbnails`); the use-cases and stores
  - tests (`line_breaks.rs` written before the algorithm, `placement.rs`, `parity.rs`, `scene_cache.rs`, `hit.rs`, `history.test.ts`, the editor cases and axe in `review-edit.spec.ts`)
  - the measurements (edit to preview, thumbnails, hit test, shell budget with the font)
  - the hand-over table, the exit checklist (both design reviews, the line-break reading), the spec issues (16-22) and the constants
- **D-139, D-142, D-146, D-154 and D-159** point to their successors. Decisions now run D-113 to D-193.

**New reopened contracts.**
- `EditState` gains `caption_position` and `highlight_numbers`, with serde defaults so V3's records still read (D-169, D-171).
- `ReviewActionKind` in the analytics allowlist gains five members, so the server is rebuilt (section 13).
- `FontSet` gains key 4 (Inter 800), appended (D-170).

**For the founder.**
- At S8, read ten rows of the line-break table.
- At S10A and S19, review the captured screens.
- Copy-back item 16: the product spec's target customer becomes content creators in general (applied at S18).

**How it was checked.** `node scripts/check-file-tree.mjs` passes. Decision numbers run D-113 to D-193 with no gap. No code changed.

---

## Entry 0.4 - 2026-10-22 - `v4buildguide.md` written (outside a prompt)

**What.** `docs/v4/v4buildguide.md`, the step-by-step guide for V4, in the structure of `v3buildguide.md`:
- what it is, how to read the references, how it is organised, a phase overview, and "Read this first" (ten traps)
- **20 phases (0-19) over 13 days.** Each phase has a goal, numbered steps with Order / File / Section / Watch-for tables, commands with their expected output, a "Prove the guards" step where a guard is new, and a milestone ending in `Pass:`, a commit on `v4-build`, a push and a CI read on PR #7.
- **Three human gates:**
  - Step 8.2: the founder reads ten rows of the line-break table, before the algorithm exists.
  - Phase 11: design review 1, on sixteen captured screens.
  - Step 19.5: design review 2 on the deployed editor, plus five goldens.
- a quick reference, and the assumptions and gaps found

TS §5 already lists the file.

**Found while writing it, and applied to `v4implementation.md` in the same commit.**
- **The interface font.** `vercel.json` rewrites every path outside `/api/`, `/assets/` and `/ort/` to `index.html`, so the font moves from `web/public/fonts/` to `web/src/ui/assets/` (a hashed `/assets/` file; D-180, §3.2, §4, §17.6).
- **The screen capture** gets its own Playwright project, `screens`, and a root script `pnpm screens`. CI never runs it (§4, §19.2).
- **Goldens.** S7 writes only the two video-only goldens; the other fourteen are written at S9, because the look changes Clean (§5).

**Other placements.** The guide's "Assumptions (c)" lists them:
- `v3_reference.rs` is rewritten in Phase 5 and passes in Phase 6.
- A minimal `review-update.ts` is written in Phase 12 and completed in Phase 13.
- The new copy keys are written in Phase 10, so the design review reads real words.

**How it was checked.** `node scripts/check-file-tree.mjs` passes. Every phase maps to an S-step of §5. The counts used (461/0/3, 92, 12, 48 media cases) are V3's close. No code changed.


## Entry 0.5 - 2026-10-22 - `coding-promptsv4.md` written (outside a prompt)

**What.** `docs/v4/coding-promptsv4.md`, the coding prompts for V4, in the structure of `coding-promptsv3.md`:
- a header (sources, how to use, the order rules, the three human gates, effort ratings, the design-skill note)
- the Standard Agent Block (terminal, before / during / after rules, the frozen check, the gate, commit and push on `v4-build`, CI read with `gh pr checks 7`)
- "Who does what", the implementation map (architecture, components, dependency graph, phases, prompt table)
- **38 prompts, 91 to 128**, each with the 20 sections of V3's prompts and an effort line (29 High, 9 Medium)
- cross-prompt contracts, high-risk areas, invariants, forbidden shortcuts, traceability, a coverage audit, and "Gaps settled by these prompts" (16 rows)

**Mapping.** The prompts follow the guide's 20 phases one by one; a phase is split only at a step boundary. Phase 5 is split into a Rust prompt (98) and a TypeScript prompt (99); S1 into records (91) and configuration (92). The three human gates are at Prompts 102→103 (ten line-break rows), 111 (design review 1) and 128 (design review 2 and five goldens).

**Found while writing it** (all settled in the prompts, no decision of §2 changed; the full list is the file's last table):
- `scripts/check-file-tree.mjs` holds a hand-written crate table that counts dev-dependencies, so Prompt 92 adds `offcut-plan` and the dev-only `offcut-detect` edges to it (§19.2 said the script reads the graph from the spec).
- G 3.1's import change in `media.worker.ts` and `asr.worker.ts` hits G 1.6's frozen check: three one-line exceptions are recorded in the SAB.
- G 0.5's runner probe uses `workflow_dispatch`, which only runs from the default branch, and a scratch-branch push conflicts with D-115: the founder decides before Prompt 91, else the reading moves to Prompt 101.
- The server's `analytics_allowlist.rs` is frozen: the five new cases go in `analytics.rs`'s inline tests.
- The dev hooks get a file, `web/src/ui/dev-hooks.ts` (Prompt 110). The transcript editor reads chunk ranges from the scene (`SceneReport.chunks`), not from a second implementation.
- `review-edit.spec.ts`'s cases are grouped into sixteen to keep §20.1's 76.

**Correction to Entry 0.4.** Entry 0.4 says TS §5 already listed `v4buildguide.md`. It did not; `check-file-tree.mjs` passed because it does not check `docs/`. TS §5 now lists both `v4buildguide.md` and `coding-promptsv4.md` (D-113), in this commit.

**How it was checked.** Each prompt has all 20 sections and one effort line. Prompt numbers and their "(V4 nn/38)" labels agree. The decision audit maps D-113 to D-193 to prompts. `node scripts/check-file-tree.mjs` passes. No code changed.

## Entry 0.6 - 2026-10-22 - The founder allows one scratch branch for the adapter probe (outside a prompt)

**Decision.** The founder allowed the agent to push one throwaway branch, `scratch/v4-adapter`, to `origin` ("allowed to push a throaway branch"). It carries only the adapter probe of G 0.5 and an `on: push` workflow limited to that branch, so the probe runs on the Linux runner image before S1 (§1A item 7). The agent deletes the branch locally and on `origin` when the reading is recorded. No other branch besides `v4-build` may be pushed (D-115 otherwise unchanged).

**Applied.** `coding-promptsv4.md`'s "Who does what" row now says "Decided 2026-10-22: yes". Prompt 91's "Project State" points to this entry. Prompt 91 records the runner's adapter name, Mesa version and pixel in Entry 1, instead of deferring the reading to Prompt 101.

---

## Entry 1 - 2026-10-22 - P91 (S1): baseline

**What.** Prompt 91 (G Phase 0 and Step 1.1): the tool audit, the V3 baseline run on `v4-build`, the V3 hand-over markers, the software-adapter probe on D1 and on the Linux runner, the dependency dry run, the interface font file, this entry and `docs/v4/experiments.md`. Only the two docs change in this commit. The probe and the dry run lived on two scratch branches, both deleted locally and on `origin` (no scratch branch exists). The records are also in `~/v4-baseline.txt` and the font in `~/inter-latin.woff2`, both outside the repository.

| Item | Value |
|---|---|
| Code baseline | `550ff8fcadacd4e5fff9b25722831e7c1f985907` (`origin/main`, merged into `v4-build` at `81474bf`); `git merge-base v4-build origin/main` = `550ff8f` |
| `origin/main` (§1A item 1) | `550ff8f`, `9122eaf`, `73580eb` (`git log --oneline -3`; first parent: `550ff8f`, `9122eaf`, `7f091fb`); tag `v3` → `7f091fb` (annotated tag object `b8303a0`) |
| PR #7 | OPEN, draft, `main` ← `v4-build`; last run (39686656756) `ci` pass, `e2e-media` pass |
| Rust tests at baseline | 461 passed, 0 failed, 3 ignored |
| Vitest at baseline | 92 |
| `pnpm e2e` at baseline | 12 |
| `pnpm e2e:media` at baseline (D1, keyed) | 48 cases (49 passed with setup, 1 fixme skip), 11.0 min |
| `offcut_core_bg`, `offcut_render_bg` size | 713,306 bytes, 6,113,666 bytes (gzip 333.19 kB, 2,930.48 kB) |
| App shell (check-hosts) | 307.3 kB gzip (the "before" of D-180) |
| Software adapters | D1: Microsoft Basic Render Driver (WARP), `Cpu`, Dx12, driver 10.0.26100.9711; runner: llvmpipe (LLVM 20.1.2, 256 bits), `Cpu`, Vulkan, mesa-vulkan-drivers 25.2.8-0ubuntu0.24.04.4 |
| Adapter probe pixel | Centre `[230, 57, 70, 255]` = the circle's colour, corner `[0, 0, 0, 0]`, on both (D1 with DXC, see below) |
| Runner adapter probe | Done (run 39692861195, `ubuntu-latest` image 20261018.339.1), not deferred |
| Founder's answer on the scratch push | Yes, 2026-10-22 (Entry 0.6) |
| `insta`, `png`, `@axe-core/playwright` | 1.49.0 (`json`), 0.18.1, 4.14.0 (MPL-2.0; `axe-core` 4.14.0) |
| Interface font | Inter Version 4.001;git-9221beed3 (the scene's `Inter-Variable.ttf`), Latin subset, 65,508 bytes, `~/inter-latin.woff2` |
| V3 markers found | 21 (11 in `crates`, 10 in `web/src`), as §1A item 5 lists |
| The four full files | `render.worker.ts` 372, `rpc.ts` 377, `video-source.ts` 399, `session.rs` 381 |
| Untagged TODO | none |
| `gh` version | 2.102.0 (2026-09-30), logged in as `mayanpathak` |

**Tools (G 0.2), D1.** rustc 1.99.0 (`rust-toolchain.toml`), targets `wasm32-unknown-unknown` and `x86_64-pc-windows-msvc`; wasm-bindgen 0.2.129; wasm-opt 133; cargo-deny 0.20.2; cargo-insta 1.49.0 (installed in this prompt, `cargo install cargo-insta --locked`); Node 24.19.0, pnpm 10.15.0, Playwright 1.63.0; ffmpeg and ffprobe 9.0.2 (full_build); Python 3.13.5 with fonttools 4.67.0 and brotli 1.2.0 (installed in this prompt with `pip`); `gh` 2.102.0; Docker 28.3.2.

**The baseline run (G 0.3).** `pnpm install --frozen-lockfile`, `pnpm check` (exit 0), `pnpm fixtures` (up to date), `cargo test --workspace`, Vitest, `pnpm e2e`, the keyed build and `pnpm e2e:media`, then the unkeyed `pnpm build` (the two wasm sizes and `check-hosts.mjs`'s shell line come from it). The test key was set in the run's own shell only, worked out from `ci.yml` as V2 and V3 did, never printed, and unset before the unkeyed build. The first `pnpm e2e` failed one case ("J1: the landing content", the hero heading): `web/dist` was a build of 2026-10-21, older than `550ff8f`'s hero line, and `pnpm e2e` serves `dist`. After the fresh `pnpm build` it passed 12 of 12. No V3 code changed.

**The V3 markers (G 0.4).** `crates`: `offcut-render/src/gpu.rs:33`, `video_pass.rs:174`; `offcut-scene/src/display_list.rs:57`, `events/mod.rs:12`, `events/mod.rs:26`, `framing.rs:11`, `lib.rs:94`, `styles.rs:31`, `styles.rs:77`, `styles.rs:79`; `offcut-wasm-render/src/detect_api.rs:28`. `web/src`: `state/preview-store.ts:33`, `usecases/import-clip.ts:154`, `:199`, `usecases/run-pipeline.ts:40`, `:131`, `:319`, `usecases/start-export.ts:129`, `wasm/load-render.ts:73`, `workers/render.worker.ts:117`, `workers/render/video-source.ts:148`. Each is gone or re-tagged by Prompt 127.

**The adapter probe (G 0.5).** On `scratch/v4-adapter`: `offcut-render`'s `[dev-dependencies]` `wgpu` with `vulkan`, `dx12`, `wgsl` and `pollster` 1.0.1 (the guide said "0.4"; the latest stable was taken; never kept), `examples/adapter_probe.rs` (an instance with `DX12 | VULKAN`, `force_fallback_adapter: true`, a 64 x 64 `Rgba8Unorm` texture with `COPY_SRC | STORAGE_BINDING | RENDER_ATTACHMENT`, a Vello scene of one filled circle `(230, 57, 70)`, a copy to a buffer with 256-byte rows, the centre pixel), and `.github/workflows/adapter-probe.yml` (`on: push` to that branch only, `ubuntu-latest`, `permissions: contents: read`, no secret). **D1 needs DXC.** wgpu's DX12 shader compiler defaults to `Auto`: a statically linked DXC, else `dxcompiler.dll` on `PATH`, else FXC. D1 has no DXC on `PATH`, so FXC was used, and the Vello render crashed WARP with `STATUS_ACCESS_VIOLATION` inside `device.poll` (a plain clear, copy and read-back on the same FXC path gave the exact pixel, so WARP itself works). With the Windows SDK's DXC (`C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64`, `dxcompiler.dll` 1.8.2502.11, wgpu's minimum) put first on `PATH`, the probe printed the expected pixel. G 0.5's table: "both render", so D-134's fallback is not taken; Prompt 101 must make `golden_frames.rs` use DXC on Windows (known issue 3).

**The dependency dry run (G 0.6).** On `scratch/v4-deps` (local only): `cargo add -p offcut-scene --dev insta --features json` (1.49.0), `cargo add -p offcut-render --dev png` (0.18.1), `pnpm --filter web add -D @axe-core/playwright` (4.14.0, `require` prints "ok"). `cargo deny check licenses bans`: **licenses ok** (also with `wgpu`'s `vulkan` and `dx12` dev features, tried on the same branch); **bans failed** on one entry: `insta` → `tempfile` 3.27.0 → `getrandom` 0.4.3, and `tempfile` is not one of the `getrandom` ban's wrappers. `deny.toml` needs `"tempfile"` in that list (a dev-only path: `insta` is a test dependency) in the prompt that gives a crate `insta` (Prompt 93; known issue 4). The wasm32 normal tree of `offcut-render` has no `ash`, `d3d12`, `windows` or `gpu-alloc-*` crate. The guide's grep (`^(ash|d3d12|windows|gpu-alloc-)`) does print `windows-sys` and `windows-link`, also at the baseline: they come from `ts-rs-macros` → `termcolor` → `winapi-util`, a proc-macro built for the host, not a graphics backend. Read with `-e normal,no-proc-macro` the tree is clean. Also found: `wgpu` 30 gates `wgpu-core`, and with it every native backend, off `wasm32` entirely, so a `vulkan` feature on the normal `wgpu` dependency never reaches the browser bundle and the wasm32 grep cannot see it; it shows in the host normal tree (known issue 5, for Prompt 92's guard drill). The branch was reverted and deleted; `pnpm install --frozen-lockfile` and `git status --short` were clean after.

**The font (G 0.7).** `pyftsubset` (fonttools 4.67.0) on `crates/offcut-scene/assets/fonts/Inter-Variable.ttf` with G 0.7's unicode list, `--layout-features='kern,liga,calt,tnum,ss01'`, `--flavor=woff2`: 65,508 bytes, the `wght` axis (100-900) and the `opsz` axis (14-32) kept, features `calt kern ss01 tnum`. Inter 4.001 has no `liga` feature (its ligatures are in `calt`), so none is lost. Licence OFL-1.1, already in `crates/offcut-scene/assets/fonts/LICENSES.md`. The file stays outside the repository until Prompt 108.

**`docs/v4/experiments.md`.** Twelve headings for §21.1's readings and two for §21.2's decisions, each with empty `Question:`, `Method:`, `Numbers:`, `Date:`, `Decision:` lines.

**Inputs that wait on a person (G 0.8).**

| Input | Asked now | Needed at |
|---|---|---|
| The push rule of D-115 | Confirmed 2026-10-22 ("commit to that branch only") | - |
| Ten rows of the line-break table (D-167) | They come on day 5 | After Prompt 102 (G 8.2) |
| Design review 1: sixteen screens at two widths (D-191) | Half an hour on day 8 | Prompt 111 (G Phase 11) |
| Design review 2 and five goldens (D-191, §23) | Half an hour on day 13 | Prompt 128 (G 19.5) |
| `analytics_events` SQL after the deploy | Nothing now | Prompt 128 (G 19.6) |
| V3 CL item 7, the Linux ffmpeg URL (dies about 2026-11-27; today is before it) | Move it in Prompt 125, or leave it? | Before Prompt 125 (G 17.1) |

V4 adds no secret.

**Differs from the guide.** `pollster` 1.0.1, not "0.4". The probe uses `PollType::Wait` and reads one padded row layout (64 x 4 = 256 bytes, no padding needed). D1's probe needed DXC on `PATH` (above). The first `pnpm e2e` was re-run after the build (above). S1 is two commits ("Gaps settled" item 11): this one (records) and Prompt 92's (configuration).

**How it was checked.** `git merge-base v4-build origin/main` = `550ff8f`; `git branch -a | grep -c scratch/` = 0; `git ls-remote --heads origin | grep -c scratch` = 0; `grep -cE "^## " docs/v4/experiments.md` = 14; `grep -c "^Decision:" docs/v4/experiments.md` = 14; `~/inter-latin.woff2` 65,508 bytes. The SAB gate and its counts follow in the commit's own run; the counts are the ones above.

**Open.** Known issues 3-5 (new). Open item 6 closed (PR #7 exists as a draft); items 1-5 unchanged.

---

## Entry 2 - 2026-10-22 - P92 (S1): offcut-plan crate, dependencies, spec tree and graph, frozen check

**What.** Prompt 92 (G 1.2-1.7, Milestone 1): the repository accepts every V4 file and crate edge; nothing behaves differently.

**Created.** `crates/offcut-plan/Cargo.toml` (`[lints] workspace = true`; `offcut-types`, `offcut-text`, `serde` with `derive`, `thiserror`, optional `serde_json`; `[features] default = ["prompt"]`, `prompt = ["dep:serde_json"]`) and `crates/offcut-plan/src/lib.rs` (the crate doc comment only).

**Changed.**
- `docs/technicalspec.md`:
  - §2: the `offcut-plan` row may use `offcut-types`, `offcut-text`, `serde`, `thiserror`, and `serde_json` behind the default feature `prompt` (D-122's correction).
  - §5: every NEW path of v4implementation §4 that the tree did not list yet, one line each with its decision: `offcut-types/src/{timemap,plan}.rs`; the whole `offcut-plan/` crate, including `src/validate_params.rs` (C.1's allowed split), `tests/ui/valid_plan_private.rs` (its `.stderr` is ignored by the script) and `tests/snapshots/`; the scene's `theme.rs`, `motion.rs`, `readability.rs`, `placement.rs`, `captions_draw.rs`, `hit.rs`, `video.rs` and eight test files, plus the temporary `tests/v3_reference.rs` and the six `tests/snapshots/v3_reference/{busy_1080,busy_540,reference_1080,a01_1080,w01_1080,w07_1080}.json`, each marked "temporary, V4 S2 to S7"; `offcut-render/tests/common/ssim.rs`; `offcut-dsp/src/{program,silence}.rs`, `tests/program_identity.rs`; `offcut-wasm-render/src/scene_api.rs`; `web/LICENSES.md`; `workers/rpc-worker.ts`, `render/{frame-decoder,preview-job,export-job}.ts`; `state/{ui-store,history,history.test}.ts`; seven use-cases (`set-caption-position`, `set-highlight-numbers`, `replace-all`, `undo-redo`, `review-update`, `preview-interact`, `style-thumbnails`); `ui/assets/inter-latin.woff2`, `ui/icons.tsx`, `ui/hooks/use-shortcuts.ts`, `ui/layout/{EditorLayout,TopBar,Stage,SidePanel}.tsx`, the twelve `ui/primitives/*.tsx` by name (no `index.ts`: Prompt 108 names none), the components `ProcessingCard`, `PlanItemList`, `StylePanel`, `FramingPanel`, `CutsSection`, `ShortcutsSheet`, `ToastHost`; the four CSS modules; `tests-e2e/screens.spec.ts`; `fixtures/golden/` (one line, now "16 at 540x960"); `docs/v4/experiments.md`. `EventToggleList.tsx` is renamed `PlanItemList.tsx`; `ReviewPanel.tsx` is gone, `layout/SidePanel.tsx` takes its place. Files the tree already listed (it describes V10) were left as they were, e.g. `rpc.test.ts`, the two machine tests, `run-pipeline.test.ts`, `edit-word.ts`, `cancel-job.ts`, `CaptionEditor.tsx`, `golden_frames.rs`, `preview-sync`/`review-edit`/`cancel.spec.ts`. `dev-hooks.ts` is Prompt 110's ("Gaps settled" item 5).
  - §7: the graph gains a line `offcut-wasm-render → offcut-plan → offcut-text`, and a paragraph lists D-122's edges (scene → plan, plan → text, plan → types, wasm-render → plan without default features) and the two dev-only edges (scene → detect, plan → detect). The "Forbidden" table's pure-crate row names `-plan`.
- `scripts/check-file-tree.mjs`: `ALLOWED_CRATE_DEPS` gains `"offcut-plan": ["offcut-text", "offcut-types", "offcut-detect"]`, adds `offcut-plan` and `offcut-detect` to `offcut-scene` and `offcut-plan` to `offcut-wasm-render`, each `offcut-detect` edge commented "dev-dependency only: the labeled-set tests (v4implementation §3.2)" ("Gaps settled" item 2); `PURE_CRATES` gains `offcut-plan`.
- `Cargo.toml`: member `crates/offcut-plan`; `[workspace.dependencies]` `offcut-plan` (path), `insta = { version = "1.49.0", features = ["json"] }`, `png = "0.18.1"` (Entry 1's versions).
- `crates/offcut-render/Cargo.toml`: `[dev-dependencies]` `wgpu = { workspace = true, features = ["vulkan", "dx12", "wgsl"] }`, `png = { workspace = true }` (D-134).
- `deny.toml`: `offcut-plan` added to the `offcut-text` ban's wrappers, the one entry `cargo deny check` named ("crate 'offcut-text = 0.1.0' is explicitly banned"). No licence entry was needed: `licenses ok` with the native backends.
- `web/package.json`: dev dependency `"@axe-core/playwright": "4.14.0"` (exact; `pnpm-lock.yaml` updated by `pnpm --filter web add -D @axe-core/playwright@4.14.0`).
- `Cargo.lock`, `pnpm-lock.yaml`.
- `docs/v4/v4changelog.md`: this entry; known issue 1 closed ("recorded as REOPENED, D-127; closed by P100"); known issue 2 kept open for Prompt 127; known issues 4 and 5 extended; new rows 6-15, one per reopened contract of SAB rule 3 with the prompt that closes it; row 16 (found).

**`unused_crate_dependencies`.** The workspace denies only the clippy lints of `[workspace.lints.clippy]`, not rustc's `unused_crate_dependencies`, so `offcut-plan`'s normal dependencies are added now as G 1.3 says. The dev-dependencies of `offcut-scene` (`insta`, `offcut-detect`; Prompt 93) and `offcut-plan` (Prompt 97) are left to those prompts (this prompt's "Do Not Implement Yet"); each needs a `deny.toml` wrapper then (known issue 4).

**The frozen check (G 1.6).** It is the SAB gate's `git diff --name-only "$BASE" -- …` command, run with `BASE=550ff8f`. Its output in this prompt's gate: (empty).

**Guard drills (G 1.7)**, each undone after:
1. `touch crates/offcut-plan/src/stray.rs` → `check-file-tree: 1 problem` / `crates/offcut-plan/src/stray.rs: not in the file tree of TS §5. Add it there in the same change, or remove the file.`
2. `web-sys = { workspace = true }` in `offcut-plan`'s dependencies → `offcut-plan: resolves web-sys, which a pure crate must not use (TS §7). Check the features of its dependencies.` (and the same for `wasm-bindgen` and `js-sys`). The script runs `cargo tree --locked`: on a first try, with the manifest edited and the lockfile not updated, it failed for every pure crate with "cannot update the lock file … because --locked was passed" instead. That is a failure too, but not the purity rule, so the drill was repeated after `cargo metadata` had updated the lock. `Cargo.lock` was restored from a copy taken before the drills.
3. `vulkan` added to `offcut-render`'s normal `wgpu` features → G 1.3's wasm32 grep prints only `windows-sys v0.61.2` and `windows-link v0.2.1`, which it also prints without the edit (the host proc-macro `ts-rs-macros`); with `-e normal,no-proc-macro` it prints nothing. **The wasm32 grep cannot fire on this edit**: `wgpu` 30 builds no `wgpu-core`, so no native backend, for `wasm32`. The browser bundle is safe by construction. The edit shows in the host normal tree: `cargo tree -p offcut-render -e normal,no-proc-macro --prefix none | grep -E '^(ash|d3d12|windows|gpu-alloc[a-z-]*) v'` prints `ash v0.38.0+1.3.281`, `gpu-allocator v0.28.0`, `windows v0.62.2`; restored, it prints nothing. Known issue 5 carries this to Prompt 125's CI step.

**How it was checked.**
- `cargo build --workspace`: Finished.
- `cargo deny check`: advisories ok, bans ok, licenses ok, sources ok.
- `cargo tree -p offcut-render -e normal --target wasm32-unknown-unknown --prefix none | grep -iE '^(ash|d3d12|windows|gpu-alloc-)'`: `windows-sys`, `windows-link` only, from the host proc-macro, as at the baseline; with `-e normal,no-proc-macro`, no output.
- `cargo tree -p offcut-plan -e normal --prefix none | sort -u | grep -E '^(web-sys|js-sys|wasm-bindgen|getrandom|rand|wgpu) '`: no output.
- `cargo build -p offcut-plan --target wasm32-unknown-unknown --no-default-features --release`: Finished.
- `node scripts/check-file-tree.mjs`: `ok. 278 files in the tree of TS §5; longest source file web/src/workers/render/video-source.ts (399 lines); 13 crates within the graph of TS §7.`
- `grep -c "v4/" docs/technicalspec.md`: 5.
- The SAB gate: on `v4-build`, no `zz` file, no test key; `pnpm fixtures` up to date; `pnpm check` exit 0; Rust 461 passed, 0 failed, 3 ignored; Vitest 92; `pnpm build` exit 0 (app shell 307.3 kB gzip; `offcut_core_bg` 713,306 and `offcut_render_bg` 6,113,666 bytes, the same files as at the baseline); `pnpm e2e` 12 passed; the frozen check prints nothing; no private path tracked. The counts equal Entry 1.

**Pinned versions.** `insta` 1.49.0 (`json`), `png` 0.18.1 (in `[workspace.dependencies]`; `png` also in `offcut-render`'s dev-dependencies), `@axe-core/playwright` 4.14.0. New lockfile entries come from `wgpu`'s native backends (dev only): `ash` 0.38.0+1.3.281, `gpu-allocator` 0.28.0, `windows` 0.62.2 and its family, `spirv` 0.4.0, among others.

**Differs from the guide or the prompt.** The `vulkan` drill's rule is the host tree, not the wasm32 tree (above). `@axe-core/playwright` is written exactly, not with a caret like its neighbours in `web/package.json`, so the manifest itself says "4.14.0". The §7 graph change is a line plus a paragraph rather than a redrawn diagram.

**Found, not fixed.** Known issue 16: TS §9 and §13 still name `ReviewPanel` and `EventToggleList` (copy-back, Prompt 127). `pnpm install` warns that `eslint-plugin-react` 7.37.5 wants `eslint` up to 9.7 while the workspace has 10.12.0; this was there before V4 and lint passes.

**Open.** Known issues 2-16 as the table says; open items 1-5 unchanged.

**Follow-up (same prompt): CI ran out of disk.** The first CI run of `b238150` (run 39697018116, job `ci`) failed in `cargo test`: `ld terminated with signal 7 [Bus error]` while linking `offcut-dsp`'s `duration_preserved`, a test with no `wgpu` in it. The re-run of the failed job failed the same way, and the runner's own log said `No space left on device`. The cause: the Cargo cache key hashes `Cargo.lock`, which this prompt changed, so the job restored the previous `target/` through the fallback key (1.76 GB compressed) and then built `wgpu`'s native backends (`wgpu-core`, `wgpu-hal`, `ash`, …) for every test binary of `offcut-render` on top of it. Fix: `.github/workflows/ci.yml` gains one step after "Check out", "Free disk space", which prints `df -h /`, removes the runner's preinstalled .NET, Android SDK, GHC and CodeQL (none used by the job) and prints `df -h /` again. No build, test or deploy step changes. `ci.yml` is not on the frozen list; Prompt 125 owns the rest of its V4 changes (§19.3). The local gate above was unaffected (D1 has the space).

---

## Entry 3 - 2026-10-22 - P93 (S2): the identity reference, written by the V3 scene

**What.** Prompt 93 (G Phase 2, D-116, D-117): `crates/offcut-scene/tests/v3_reference.rs` builds six scenes through V3's `build_scene(SceneInput { transcript, events, edit: &EditState::default(), clip, profile })` and keeps the display list of every 7th frame (0, 7, 14, …; "Gaps settled" item 3), serialized with `serde_json::to_string(&scene.frame_at(FrameIdx::new(n).to_time_ms()))`. With `OFFCUT_V3_REFERENCE_WRITE=1` it writes one JSON array per input to `crates/offcut-scene/tests/snapshots/v3_reference/<name>.json` (one frame per line, each in `serde_json`'s compact form); without it, it reads the file and compares value for value, failing with the input, the frame and the first differing JSON path. It writes nothing else. No file under `crates/offcut-scene/src` changed (`git diff --name-only 550ff8f -- crates/offcut-scene/src` prints nothing): the files were written by V3's scene code.

| Input | Transcript | Events | Clip | Width | Frames kept (of) | Events | Bytes |
|---|---|---|---|---|---|---|---|
| `busy_1080` | `busy()` of `lib.rs`'s tests, copied | its three NumberReveals | 1080 x 1920, 10,000 ms | 1080 | 43 (300) | 3 | 42,159 |
| `busy_540` | same | same | same | 540 | 43 (300) | 3 | 42,399 |
| `reference_1080` | the reference clip's 157 words (below) | `detect`, zero prosody, no edits | 1920 x 1080, 74,705 ms | 1080 | 321 (2,242) | 4 | 499,303 |
| `a01_1080` | `a01-scriptA.json`, timed as `precision_recall.rs` does | `detect` with the file's `*word*` marks | 1080 x 1920, 67,080 ms (synthesized) | 1080 | 288 (2,013) | 4 | 461,047 |
| `w01_1080` | `w01-saas-pricing.json`, same | same | 1080 x 1920, 29,200 ms (synthesized) | 1080 | 126 (876) | 3 | 190,956 |
| `w07_1080` | `w07-camera.json`, same | same | 1080 x 1920, 23,335 ms (synthesized) | 1080 | 101 (701) | 2 | 160,057 |

The event counts are the expected ones: the reference clip's four of README "Expected events of V3" without the KeywordPop (it needs measured prosody, and the input has none); a01's four labeled events; w01's from-to, number and keyword pop; w07's number and keyword pop. V3 draws NumberReveal only, so the other kinds' tracks draw nothing (that is V3's behaviour and part of the reference). Content: 1,238 to 1,337 glyph runs in the long inputs; 274 of 288, 294 of 321 frames non-empty.

**The inputs, as built.**
- The helpers `word`, `transcript`, `clip`, `profile` (Creator, no watermark, height `width * 16 / 9`) and `number_event` are copied from `lib.rs`'s `testing` module, and `busy()` from its tests, unchanged.
- **The reference clip's words: differs from the prompt.** The prompt says `fixtures/speech/README.md` holds the recognized words "with word times". It holds the 17 sentences with each one's first start and last end, and the six number words with their spans; no other word time exists in the repository. The test's `REFERENCE_WORDS` table (157 rows) keeps every time the README gives exactly, and the split `$12` (62,660-63,100) / `,000` (63,100-63,540) of `offcut-detect`'s tests. Every other word takes an equal share, rounded down, of the gap between its fixed neighbours; each sentence's word count was checked against its word range. Words are contiguous inside a sentence, the sentence pauses are the README's, the shortest word is 244 ms. The table was generated once by a scratch script and is now the input itself: Prompt 100 builds from the same table, so the proof does not depend on how close these times are to the recording. Confidence 1.0 (TE-2).
- The labeled files' times and emphasis come from a copy of `precision_recall.rs`'s `synthesize`. It also returns where the next word would start (the last word's end plus its 400 ms pause), which is the clip's length; none of the three files has `edits`, and the scene uses `EditState::default()`, so `detect` gets no edits.
- `profile` is the tests' constructor, not `offcut-entitlement`'s `preview_profile`: `offcut-scene` may not depend on `offcut-entitlement` (TS §7), and the `busy` inputs must equal the V3 tests' `build()`.

**Changed.**
- `crates/offcut-scene/Cargo.toml`: `[dev-dependencies]` `offcut-detect` and `serde_json` (workspace).
- `deny.toml`: `offcut-scene` added to the `offcut-detect` ban's wrappers, with the comment "a dev-dependency only", the one entry `cargo deny check` named (known issue 4). `cargo tree -p offcut-scene -e normal` still has no `offcut-detect`.
- `Cargo.lock`; `docs/v4/v4changelog.md` (this entry, known issues 4 and 17).
- The test file opens with the `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]` header every V3 integration test has (`clippy.toml`'s test allowances do not reach helper functions of a `tests/` file).

**How it was checked.**
- `OFFCUT_V3_REFERENCE_WRITE=1 cargo test -p offcut-scene --test v3_reference`: 1 passed; six files written, all non-empty.
- `cargo test -p offcut-scene --test v3_reference`: 1 passed (read and compared), three times.
- Drill: `"size":64.0` of frame 63 of `w07_1080.json` hand-edited to `964.0` → `w07_1080: frame 63 (index 9) differs at .cmds[0].GlyphRun.size (964.0 vs 64.0) (reference vs now)`; restored with `git checkout` from the index; passes again.
- `git diff --name-only 550ff8f -- crates/offcut-scene/src`: no output.
- `cargo deny check`: ok. `node scripts/check-file-tree.mjs`: ok, 285 files, 13 crates. `cargo fmt --check`, `cargo clippy -p offcut-scene --all-targets -- -D warnings`: clean.
- The scene uses no libm function that could round differently on Linux (`powi` only), so CI's Linux run must read the same values; it is the first cross-platform check of the files.
- The SAB gate: on `v4-build`, no `zz` file, no test key; `pnpm fixtures` up to date; `pnpm check` exit 0; **Rust 462 passed, 0 failed, 3 ignored** (Entry 2: 461; +1, `v3_reference`); Vitest 92; `pnpm build` exit 0 (app shell 307.3 kB); `pnpm e2e` 12 passed; the frozen check prints nothing; no private path tracked.

**Frozen.** From this commit until Prompt 101 the six files must not change; a change shows in `git diff --stat <this commit> -- crates/offcut-scene/tests/snapshots/v3_reference`. The commit's SHA is not known while this entry is written: Prompt 94's entry records "identity reference frozen at <SHA>" (known issue 17).

**Open.** Known issue 17 (new). Known issue 4 narrowed to `tempfile` and `offcut-plan`'s wrapper.

---

## Entry 4 - 2026-10-22 - P94 (S3): split rpc, video-source, render.worker and session (no behaviour change)

**Identity reference frozen at `09b10856af1aff15303e763a8c6b88be88f89060`** (Prompt 93's commit; known issue 17 closed). `git diff --stat 09b1085 -- crates/offcut-scene/tests/snapshots/v3_reference` prints nothing, and must until Prompt 101.

**What.** Prompt 94 (G Phase 3, D-150): the four full files give part of their bodies to five new files, by function. Names, bodies and comments move as they are; where a body had to reach state it no longer owns, it does so through one argument (below).

| File | Lines before | Lines after |
|---|---|---|
| `web/src/workers/rpc.ts` | 377 | 231 |
| `web/src/workers/rpc-worker.ts` (new) | - | 154 |
| `web/src/workers/render/video-source.ts` | 399 | 251 |
| `web/src/workers/render/frame-decoder.ts` (new) | - | 236 |
| `web/src/workers/render.worker.ts` | 372 | 180 |
| `web/src/workers/render/export-job.ts` (new) | - | 185 |
| `web/src/workers/render/preview-job.ts` (new) | - | 78 |
| `crates/offcut-wasm-render/src/session.rs` | 381 (350 above its tests) | 323 (292 above its tests) |
| `crates/offcut-wasm-render/src/scene_api.rs` (new) | - | 74 |
| `web/src/workers/render/export-loop.ts` (import line split in two) | 396 | 397 |

**G 3.1, `rpc.ts` → `rpc.ts` + `rpc-worker.ts`.** `rpc-worker.ts` takes `JobContext`, `Handlers`, `WorkerFailure`, `CANCELLED`, `serveWorker` and its private `WorkerScope`, `Job`, `transferablesOf`. `rpc.ts` keeps `WorkerCallError`, `Cancelled`, `CallOptions`, `Client`, `createClient` and imports nothing from `rpc-worker.ts`; `pool.ts` (the page's only importer) is unchanged. **Differs from the prompt: `toAppFailure` stays in `rpc.ts`**, with its tables `RETRYABLE` and `STAGES` and the `CRASH` code. The prompt lists it with the worker side, but `createClient` calls it too (a `postMessage` that throws, a buffer that was copied), so moving it would make `rpc.ts` import `rpc-worker.ts`, against the prompt's own rule that the page imports no worker code. `rpc-worker.ts` imports it from `rpc.ts` (worker → shared, the allowed direction). `render.worker.ts` and `render/export-loop.ts` import `toAppFailure` from `./rpc` and the rest from `./rpc-worker`.

**Frozen files: seven one-line import diffs, not three.** Besides `media.worker.ts` and `asr.worker.ts` ("Gaps settled" item 9), four more frozen files import `WorkerFailure` or `CANCELLED` from `rpc.ts`: `workers/media/import.ts`, `workers/media/audio-decode.ts`, `workers/asr/whisper-runtime.ts` and `workers/render/encoders.ts`. Since those two names must move (D-150) and `rpc.ts` may not re-export them, each of the four gets the same one-line change of its import path, `"../rpc"` → `"../rpc-worker"`, and nothing else; this is item 9's settlement applied to the files it missed. `coding-promptsv4.md`'s SAB comment on the frozen check now names all seven (and `program_identity.rs` from Prompt 100). The frozen check's output below lists exactly these files.

**G 3.2, `video-source.ts` → `video-source.ts` + `frame-decoder.ts`.** `FrameDecoder` owns the `VideoDecoder`: the decoder config built in the constructor (with the colour-space memory `toldColours` of D-70), `jumped(tUs, held)` with `JUMP_AHEAD_US` and `STEP_BACK_US`, `stop()` and `start(tUs)` (the two halves of a restart at a keyframe), `feed()` with `DECODE_QUEUE_MAX`, the drain after the last sample, `close()`, the output and error callbacks, `#toldFullRange` and the generation counter. `decodeFailure` moves with it and is exported. `VideoSource` keeps the queue, the shown frame, `#wantUs`/`#ahead`, `#failed`, `#closed`, the waiters, `frameAt`, `frameAtBlocking`, the `prefetch` stub, `close`, `liveFrames`, `PREVIEW_QUEUE`, `DemuxerHandle` and `asFailure`. It gives the decoder a `FrameSink` of eight callbacks (`opened`, `keep`, `release`, `wake`, `fail`, `failed`, `wantsMore`, `startAgain`), each one statement the old code ran in the same place. The order of every step is kept: a restart still closes the old decoder, drops the queue, marks the shown frame old, then starts the new decoder; `close()` still closes frames before the decoder. The one reordering is synchronous and has no observer: the generation counter now goes up inside `FrameDecoder.close()`, after the queue is dropped, where it went up just before. The public surface (constructor, `frameAt`, `frameAtBlocking`, `prefetch`, `close`, `liveFrames`) is unchanged.

**G 3.3, `render.worker.ts` → the worker + `render/export-job.ts` + `render/preview-job.ts`.** `export-job.ts` takes `Exported`, `ExportRun`, `writeExport`, `afterExport` and `exportClip`; `preview-job.ts` takes `previewTime`, `previewPlay`, `previewClock`, `previewPause` and owns the clock, the stop flag and the loop (their single owner), plus `previewForget()`, the two lines `closeSession` ran on them (`stopPreview = true; clock = undefined`). The worker keeps `open`, `scene`, `previewCanvas`, `staged`, `opened`, `now`, `closeFrames`, `openSession`, `detect`, `setScene`, `attachPreview`, `closeSession`, `notYet` and the handler table, whose `previewPlay` and `exportClip` entries pass the state on. The state reaches the job files as **one argument, `w: WorkerState`**: `opened()`, `open()`, `scene()`, `previewCanvas()`, `closeFrames`, `staged`, and `exportProfile(token)`. `exportProfile` is new: ESLint lets only `render.worker.ts` import `config/entitlement-public-key` (INV-9, D-27 c), so the token is still verified in the worker file, with the same keys and clock, and `writeExport` calls it where it called `api.exportProfileFromToken` before. `Session`, `SceneState` (the type of `scene`, named) and `WorkerState` are declared in `export-job.ts`, which `preview-job.ts` and the worker import as types: the prompt says "from where they are declared", but they were declared in `render.worker.ts`, which the job files must never import, and the prompt creates no state module. `export-job.ts` is the file that does not grow in V4 (§4 marks it F). Neither job file imports the worker file; the handler wrappers keep the parameter types of the functions they call.

**G 3.4, `session.rs` → `session.rs` + `scene_api.rs`.** `scene_api.rs` holds `EditInput`, `SceneInputValue` and a second `#[wasm_bindgen] impl RenderSession` with `set_scene` and `summary`, unchanged. `session.rs` keeps `JsRandomAccess`, the error mapping (`scene_error_name` becomes `pub(crate)` for `set_scene`), the struct (its five fields `pub(crate)`), `open`, `open_session`, `attach_canvas`, `render_frame`, `frame_count`, the four decoder calls and its test. `lib.rs`: the one line `mod scene_api;`. After `pnpm build:wasm` the generated `offcut_render.d.ts`, `offcut_render_bg.wasm.d.ts` and `offcut_render.js` are byte-identical to the build before the split, and the wasm is the same size (6,113,666 bytes).

**How it was checked.**
- `pnpm check` after each of G 3.1-3.4: exit 0 each time.
- The nine lengths above: every file under 400; the three old TypeScript files under 300; `session.rs` 323 (292 counted by the tree script, which stops at `#[cfg(test)]`).
- `git diff --stat 550ff8f -- web/tests-e2e crates/*/tests ':!crates/offcut-scene/tests/v3_reference.rs' ':!crates/offcut-scene/tests/snapshots'`: no output (no test changed).
- The SAB gate and the media run: on `v4-build`, no `zz` file, no test key; `pnpm fixtures` up to date; `pnpm check` exit 0; **Rust 462 passed, 0 failed, 3 ignored; Vitest 92; `pnpm e2e` 12 passed** (Entry 3's counts); `pnpm build` exit 0. **The frozen check lists exactly the seven files above, each `1/1` (or `1/0` for `lib.rs`) in `git diff 550ff8f --numstat`.** No private path tracked.
- **`pnpm e2e:media` on D1, keyed build: 49 passed, 1 skipped (10.9 min)**, the 48 cases of the baseline; the key was unset after (`env | grep -c` 0), and `pnpm e2e` passed 12 again on the unkeyed build.
- The page imports no worker code: in `web/dist`, `index-*.js` holds none of `serveWorker`'s strings (`CancelledUnasked`, `NoHandler`) nor `WorkerFailure`; `render.worker-*.js` holds them.
- Sizes: app shell 307.3 → **307.5 kB** gzip (the render worker's chunk 18.2 → 18.5 kB, the module wrappers of the two job files); `offcut_render_bg` 6,113,666 bytes, unchanged in size, with a new content hash (`BcZvX9-w` → `Bta-xoBy`: the second `impl` block orders the module's functions differently; the bindings are byte-identical); `offcut_core_bg` 713,306 bytes, the same file.

**Open.** Known issue 17 closed. Nothing new.
