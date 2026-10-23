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
| 7 | ~~**Read ten rows of the line-break table** (D-167, G 8.2): rows 28, 27, 25, 26, 11, 20, 21, 12, 7, 16 of `crates/offcut-scene/tests/line_breaks.rs`, shown as a phone shows them (Entry 12). For each: "I would break it this way: yes / no, because …". A "no" changes that row now, with the reason, before the break search is written~~ Done 2026-10-23: yes to all ten (Entry 13) | Founder | P103 |

## Known issues
| # | Issue | Where | Version |
|---|---|---|---|
| 1 | ~~`AudioWorkerApi` (TS §14.2) is described as frozen, and v1.2 adds `assemble` to it in V4 (TS §40.6, §40.9 S-6). V4's implementation plan must record the protocol change as a REOPENED contract, as V3 did for `FeedLine`~~ Closed in Entry 2: recorded as REOPENED, D-127; closed by P100 (row 12) | TS §14.2 | V4 |
| 2 | TS §19.2, §20.2 and §21.3 still show the v1.1 signatures (`events` in `SceneInput`, `crop()`, `render(frame, crop, rotation, list)`, `to_time_ms`). Each section now opens with a v1.2 note pointing to §40.5 / §40.9; V4's copy-back rewrites the signatures themselves when the code exists | TS §19-§21 | V4 |
| 3 | ~~**WARP on D1 needs DXC for Vello.** With wgpu's fallback shader compiler (FXC, used when no DXC is found) the Vello render crashes WARP (`STATUS_ACCESS_VIOLATION` in `device.poll`); with `dxcompiler.dll` 1.8.2502.11 of the Windows SDK on `PATH` it renders exactly (Entry 1). `golden_frames.rs` must get DXC on Windows: the SDK's DLL found by path, or `wgpu`'s `static-dxc` dev feature (its licence read by `cargo deny`); recorded as a decision in that prompt~~ Closed in Entry 11: `new_headless` takes the newest Windows SDK's `dxcompiler.dll` by path (or `WGPU_DX12_COMPILER`); an overlay drawn under FXC still crashes WARP, under it renders exactly | `offcut-render/tests/golden_frames.rs` | P101 |
| 4 | ~~`insta` brings `tempfile` → `getrandom` 0.4, and `tempfile` is not a wrapper of `deny.toml`'s `getrandom` ban: `cargo deny check bans` fails until it is added (dev-only path). Likewise the `offcut-detect` ban lists only `offcut-wasm-render`: the dev edges `offcut-scene → offcut-detect` and `offcut-plan → offcut-detect` need `offcut-scene` and `offcut-plan` as wrappers when they are added (Entry 2). **Scene's `offcut-detect` wrapper added in Entry 3**; `tempfile` waits for the first crate that takes `insta`, `offcut-plan`'s wrapper for its `offcut-detect` edge~~ Closed in Entry 7: `tempfile` and `offcut-plan` are wrappers | `deny.toml` | P97 (plan: `insta`, `offcut-detect`), P98 (scene: `insta`) |
| 5 | **The wasm32 tree grep cannot see a misplaced native feature.** `wgpu` 30 gates `wgpu-core` and every native backend off `wasm32`, so `vulkan` on the normal `wgpu` dependency never reaches the browser bundle and G 1.3's wasm32 grep prints nothing for it; it shows in the host normal tree. The grep's `^windows` also matches `windows-sys`/`windows-link` of the host proc-macro `ts-rs-macros`; read the tree with `-e normal,no-proc-macro` (Entry 1). Prompt 92's drill confirmed it (Entry 2); the CI step of §19.1 must read the host normal tree, `cargo tree -p offcut-render -e normal,no-proc-macro --prefix none \| grep -E '^(ash\|d3d12\|windows\|gpu-alloc[a-z-]*) v'`, besides the wasm32 one | G 1.3, G 1.7, §19.1's CI step | P125 |
| 6 | ~~**REOPENED CONTRACT** `offcut-types/src/units.rs`: `OutMs`, `OutSpan`; `FrameIdx::to_out_ms` replaces `to_time_ms` (D-125)~~ Closed in Entry 5 | `offcut-types` | P95 |
| 7 | ~~**REOPENED CONTRACT** `offcut-types/src/edit.rs` `EditState`: `caption_position`, `highlight_numbers` with serde defaults (D-169, D-171)~~ Closed in Entry 5 | `offcut-types`, `web/src/gen` | P95 |
| 8 | ~~**REOPENED CONTRACT** `offcut-scene/src/display_list.rs`: `FontId` becomes `FontKey`; `DrawCmd` gains `Video` and `Image` (D-130, D-132)~~ Closed in Entry 8 | `offcut-scene` | P98 |
| 9 | ~~**REOPENED CONTRACT** `offcut-scene/src/styles.rs`: style colours become theme roles, `CaptionMotion` (D-136)~~ Closed in Entry 8 | `offcut-scene` | P98 |
| 10 | ~~**REOPENED CONTRACT** `Renderer::render(sources, list, fonts)` (D-133)~~ Closed in Entry 8 | `offcut-render` | P98 |
| 11 | **REOPENED CONTRACT** `web/src/workers/protocol.ts`: `rulesPlan`, `setScene(plan)` → `SceneReport`, `previewSeek(OutMs)`, `ClockSync(OutMs)`, `assemble`, `release`, `hitTest`, `styleThumbnails`, `PreviewStats` (D-124, D-126, D-127, D-128, D-174, D-186). P99's part done in Entry 9: `rulesPlan`, `setScene` → `SceneReport`, `ClockSync` and `previewSeek` on `OutMs`, `previewPlay` → `PreviewStats`; P100's in Entry 10: `assemble`, `release`. `hitTest` and `styleThumbnails` remain | `web/src/workers` | P99, P100, P113, P116 |
| 12 | ~~**REOPENED CONTRACT** `AudioWorkerApi`: `assemble`, `release` (D-127, D-128; known issue 1)~~ Closed in Entry 10 | `audio.worker.ts`, `protocol.ts` | P100 |
| 13 | ~~**REOPENED CONTRACT** `web/eslint.config.js` `MINTING`: `control-preview.ts` mints `OutMs` (D-126)~~ Closed in Entry 9 (and `run-pipeline.ts` mints the identity map's `TimeMs`) | `web/eslint.config.js` | P99 |
| 14 | **REOPENED CONTRACT** `web/eslint.config.js` use-case edges: the four edit use-cases → `review-update.ts`, `review-update.ts` → `control-preview.ts` (§4) | `web/eslint.config.js` | P114 |
| 15 | **REOPENED CONTRACT** `offcut-api-types/src/analytics.rs` `ReviewActionKind`: five members (§13); the server rebuilds | `offcut-api-types`, `web/src/gen` | P120 |
| 16 | TS §9 (the review call chain: "ReviewPanel shows …", `<EventToggleList …>`) and the TS §13 component table still name `ReviewPanel` and `EventToggleList`, which TS §5 now replaces with `SidePanel` and `PlanItemList` (Entry 2) | TS §9, §13 | P127 (copy-back) |
| 17 | ~~Record "identity reference frozen at <SHA of Prompt 93's commit>" (the SHA is not known inside its own commit, Entry 3); from then on `git diff --stat <SHA> -- crates/offcut-scene/tests/snapshots/v3_reference` must print nothing until Prompt 101 deletes the files~~ Closed in Entry 4: frozen at `09b1085` | `docs/v4/v4changelog.md` | P94 |
| 18 | ~~§12 lists `DropReason` and `Dropped` among the generated TypeScript types; they live in `offcut-plan`, which the generator in `offcut-types` cannot name, and nothing on the page reads them in V4. Not generated (Entry 6); generate them when a page or analytics consumer exists, or correct §12 in the copy-back~~ Closed in Entry 9: `Dropped` and `DropReason` moved to `offcut-types/src/plan.rs` (re-exported by `offcut-plan`) and generated, the page's `SceneReport` being their first reader | `web/src/gen`, plan §12 | P127 (copy-back) or V6.1 |
| 19 | ~~**D-165 at risk for FromTo and ListReveal.** V-6's `min_gap` (3,000 ms FromTo, 10,000 ms ListReveal) is larger than the spacing the detector guarantees between two overlays (about 1,850 ms), so a rules plan could lose an item to `TooClose`. `validate_rules_plans.rs` runs every labeled file; if one loses an item, the (assumption) limit is revisited, not the detector (Entry 6)~~ Closed in Entry 7: 31 files, 0 items dropped | `offcut-plan/src/catalog.rs` | P97 |
| 20 | ~~**The identity proof cannot pass on `busy_1080` and `busy_540` as recorded.** V3's `busy()` gives its first two NumberReveals spans no word of its transcript has (1,000-1,400 and 3,500-3,800 ms); `validate` takes every item's span from its anchor words (D-121), so through the V4 contract they become 1,100-1,350 ("three") and 3,500-3,750 ("wanted."). In the comparison run (Entry 8) the four detector-built inputs are identical (836 frames) and each `busy` input differs in exactly 4 frames, all inside those two events' windows (count-up start, hold end, fade). **Decided by the founder on 2026-10-22 (Entry 9): re-derive the two `busy` files with V3's scene code (`09b1085`) on these anchored events, the other four files unchanged**; the alternative, accepting the 8 named frames, was declined. Prompt 100 does it, records it, and the proof then reads "6 inputs, identical"~~ Closed in Entry 10: re-derived, "6 inputs, 922 frames, identical" | `offcut-scene/tests/v3_reference.rs`, `tests/snapshots/v3_reference/busy_*.json` | P100 |
| 21 | ~~Record "line-break table frozen at <SHA of Prompt 102's commit>" (the SHA is not known inside its own commit, Entry 12); from then on a row of `line_breaks.rs` changes only with a changelog line naming the row and the reason (§23)~~ Closed in Entry 13: frozen at `01f406f` | `docs/v4/v4changelog.md` | P103 |
| 22 | `crates/offcut-scene/src/captions.rs` is exactly at the 400-line limit above its tests (Entry 14: the number rule of D-169 took the last lines). The next change there moves something out first: the reading floor to `readability.rs` (which holds `MIN_CHUNK_MS`) or the number rule beside the words. Prompt 105 changed one line in place (Entry 16); `captions_draw.rs` is at 396 | `offcut-scene/src/captions.rs` | P106 |

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

---

## Entry 5 - 2026-10-22 - P95 (S4): OutMs, TimeMap, the plan wire format, EditState fields

**What.** Prompt 95 (G 4.1-4.3, §6, §12, D-118 to D-120, D-125, D-169, D-171): the shared types of V4 exist, are tested and are generated to TypeScript; nothing calls them yet.

**REOPENED CONTRACT `offcut-types/src/units.rs` (D-125).** `OutMs(u32)`, made by the same `int_unit!` macro as `TimeMs` (so `bare_ms_rejected.rs` covers it, no new trybuild case), with `to_micros`, `checked_add`, `duration_since`; `OutSpan { start, end }` with `OutSpan::new -> Option`; `FrameIdx::to_time_ms` is renamed `to_out_ms` and returns `OutMs` (same arithmetic, `n * 1000 / 30` floored). No `From` either way; `the_two_timelines_meet_only_as_numbers` compares them through `get()`. The module comment's "there is no second timeline type (INV-5)" now names both timelines. Known issue 6 closed.

**REOPENED CONTRACT `offcut-types/src/edit.rs` `EditState` (D-169, D-171).** `CaptionPlacement(f32)`, checked like `CropOffset` (`new -> Option`, `get`, `bottom()` = 1.0, a `Deserialize` through `new`); `EditState` gains `caption_position` (`#[serde(default = "CaptionPlacement::bottom")]`) and `highlight_numbers` (`#[serde(default = "yes")]`, true); `EditState::default()` sets both. V3's stored JSON `{"word_edits":{},"event_overrides":{},"style":"clean","crop_offset":0}` reads as `EditState::default()`. `DB_VERSION` stays 1. Known issue 7 closed.

**New files.**
- `crates/offcut-types/src/timemap.rs` [ONLY] (§6.2): `Segment { Source { src: Span }, Hold { at, dur } }` (tagged `"kind"`), `SrcPos { At, Held }` (not serialized), `TimeMap { segments }` with a private field, `SilenceProfile { rms_db_10ms }` (not serialized). `identity`, `is_identity`, `out_duration`, `to_out` (the end instant of a source segment maps to the end of its stretch), `to_src` (past the end: the last instant), `out_span`, `joins`. Integer milliseconds only. A hand-written `Deserialize` reads `{ "segments": [...] }` (unknown fields refused) and refuses an empty list ("a time map must have at least one segment"), an empty or reversed `Source` span ("a source segment must not be empty"), a `Source` starting before the previous one ended ("source segments must follow each other in the recording") and a zero `Hold` ("a hold segment must not be empty"). Nothing outside tests constructs or calls a `TimeMap` yet (Prompt 98). `identity(DurMs(0))` would be an empty span its own deserializer refuses; no accepted clip has length 0.
- `crates/offcut-types/src/plan.rs` (§6.3): `PLAN_FORMAT = 1`, `EditPlan`, `PlanSource`, `Cut`, `CutWhy`, `PlanItem`, `ItemParams`, `ListPoint`, `QuantityRef`, `Label`, `Chapter`, `TextExtras`, `MusicEnergy`, `MusicCue`, with `Label::as_str` and `EditPlan::empty_rules(catalog)`, nothing more. `PlanItem.kind` is `EventKind` (D-118).

**Serde, as built.** Every struct `deny_unknown_fields`. `PlanSource`, `Cut` and `Segment` are tagged `"kind"`, snake case. `ItemParams` is adjacently tagged (`"kind"` + `"params"`): `{"kind":"detected","params":{"kind":"keyword_pop","word":31}}`, so `Detected(EventParams)` keeps its own tag with no clash. `Label` is transparent (a JSON string, `string` in TypeScript). `QuantityRef` is `{"words":{"start":3,"end":5}}`. `TextExtras` is `#[serde(default)]` (every field may be left out). **Two choices the plan leaves open, made here:**
1. **`CutWhy` and `MusicEnergy` are plain snake-case strings** (`"false_start"`, `"upbeat"`), not `{"kind":"…"}` objects. §6.3 says "every enum `tag = "kind"`", but on an enum of names only that writes an object per value; V3's name-only enums (`EventKind`, `StyleId`, `Unit`) are strings, and so is the generated TypeScript (`"filler" | "false_start"`). The pinned JSON and `the_names_on_the_wire` fix the choice.
2. **An extra field beside `{"kind":"rules"}` is not refused.** Serde cannot refuse unknown fields on a field-less variant of an internally tagged enum, even with `deny_unknown_fields` (probed: `{"kind":"rules","x":0}` reads as `Rules`; `{"kind":"model",…,"x":0}` is refused). §6.3 writes `Rules` as a unit variant and the producer sets the source, never a model (TS §40.2), so the shape is kept and the refusal test reads the `Model` arm. Every other level refuses: the plan, an item, a param block, `QuantityRef`, `TextExtras`, `MusicCue`, a `Segment`, the time map.

**The pinned JSON** (`the_json_of_a_one_item_rules_plan_is_pinned`), a contract with V6.1's server and stored plans:
`{"format":1,"catalog":1,"source":{"kind":"rules"},"cuts":[],"items":[{"id":"42","kind":"number_reveal","anchors":{"start":132,"end":134},"params":{"kind":"detected","params":{"kind":"number_reveal","value":{"value":12000.0,"unit":"usd","display":"$12k"},"label":"in sales"}},"confidence":1.0}],"chapters":[],"extras":{"hook":null,"post_caption":null,"hashtags":[]},"music":null}`.
`id` is a decimal string, as V3's `EventId` always is (D-31-style: a u64 does not fit a JavaScript number).

**Other changes.**
- `events.rs`: the D-118 doc comment on `EventKind` only (the kind of a plan item; codes 0-3 of `event_id.rs` fixed; a new kind appends a variant).
- `lib.rs`: `mod plan; mod timemap;` and their re-exports. The generator writes `OutMs` (branded, 25 brands), `OutSpan`, the thirteen plan types, `CaptionPlacement`, `Segment`, `TimeMap` and `export const PLAN_FORMAT = 1;` after `LIMITS`. `CATALOG_VERSION` joins when Prompt 96 defines it (it lives in `offcut-plan`).
- `crates/offcut-wasm-render/src/scene_api.rs` (not in the prompt's list; it did not compile otherwise): `EditInput` gains the two fields with the same defaults (`CaptionPlacement::bottom`, and `highlighted()`, which reads `EditState::default().highlight_numbers` so the value keeps one home), passed into the `EditState` it builds. A record V3 stored reaches it without the two fields.
- The two callers of the renamed function: `offcut-scene`'s inline `frames()` test helper (`lib.rs`, a call D-160 allows to change) and `tests/v3_reference.rs` now pass `TimeMs::new(FrameIdx::new(n).to_out_ms().get())` to V3's `frame_at`: the identity map, through numbers. No expected value changed; `v3_reference` still compares equal.
- `web/src/gen/domain.ts` regenerated (+125/-2). `web/src/state/clip-store.ts`: `empty().edit` gains `caption_position: 1, highlight_numbers: true`; it is the only `EditState` literal in `web/` (`git grep -n "crop_offset" -- web/src web/tests-e2e`). On the page V3 writes `edits` records and never reads one back, so no old record reaches TypeScript that expects the fields.

**V1-V3 test expectations changed, each because of a reopened contract above** (SAB rule 4: named with the reason):
- `edit.rs` `default_edit_state_has_no_edits` and `edit_state_round_trips_with_ids_as_map_keys`: their expected JSON of a whole `EditState` gains `"caption_position":1.0,"highlight_numbers":true` (D-171, D-169).
- `units.rs` `frame_30_is_one_second` and `frame_conversions_do_not_overflow`: `to_time_ms`/`TimeMs` → `to_out_ms`/`OutMs`, same numbers (D-125).
- `lib.rs` `typescript_declares_every_type_and_limit_once`: it now reads `timemap.rs` and `plan.rs` too; 79 declared names (was 59), 25 brands (was 24), `PLAN_FORMAT` present, and an explicit `NOT_GENERATED` list (`SrcPos`, `SilenceProfile`, not serialized by §6.2), each asserted absent from the TypeScript.

**New tests** (inline): `units.rs` 3 (`program_time_has_the_arithmetic_of_recording_time`, `out_span_rejects_reversed_bounds`, `the_two_timelines_meet_only_as_numbers`); `timemap.rs` 5 (identity round trips at 0, 1, 33, 59,999, 60,000 ms with `to_src` = `At(t)`, the end instant, `joins` empty; spans of a 60 s identity; a checked map with a removed span and a hold; the identity's JSON; one refusal per rule plus unknown fields); `plan.rs` 5 (every arm round trips; the names on the wire; the pinned JSON; `Detected` without a tag clash; unknown-field refusal at seven levels); `edit.rs` 3 (V3's record reads with the defaults; `CaptionPlacement::new` refuses NaN, ±infinity, -0.01, 1.01; deserialization refuses -0.01, 1.01, 1e39 (infinite as f32), `null`, a string, and a bad `caption_position` inside an `EditState`).

**How it was checked.**
- `cargo test -p offcut-types`: 74 passed, 1 ignored (the generator), trybuild ok.
- `git grep -n "to_time_ms" -- crates web/src`: no output.
- `cargo test -p offcut-scene --test v3_reference`: 1 passed (still V3's scene).
- `pnpm gen:types && sh scripts/check-gen-clean.sh`: clean after the commit of `domain.ts`.
- The SAB gate: on `v4-build`, no `zz` file, no test key; `pnpm fixtures` up to date; `pnpm check` exit 0 (the first run failed only at `check-gen-clean.sh`, which diffs `web/src/gen` against git: it passes once `domain.ts` is staged, as the prompt says); **Rust 478 passed, 0 failed, 3 ignored** (Entry 4: 462; +16, the new tests above); Vitest 92; `pnpm build` exit 0 (app shell 307.5 kB); `pnpm e2e` 12 passed; the frozen check lists the seven recorded one-line exceptions of Entry 4 and nothing else; no private path tracked.

**Open.** Known issues 6 and 7 closed. Nothing new.

---

## Entry 6 - 2026-10-22 - P96 (S4): offcut-plan catalog, config, resolve, validate, from_rules

**What.** Prompt 96 (G 4.4 rows 1-5 and 7, §7.1-§7.7, C.1, D-118 to D-124, D-165): the edit plan's logic in `offcut-plan`, each file tested inline as it landed (`cargo test -p offcut-plan` after each). Nothing outside the crate calls it yet (Prompt 98).

**Files.**

| File | Lines (above its tests) | What |
|---|---|---|
| `src/catalog.rs` [ONLY] | 238 (180) | `CATALOG_VERSION = 1`; `Producers`, `AnchorRule`, `ParamTy`, `ParamSpec`, `ItemSpec` (`zone: &'static str`); the four rows of §7.2's table exactly, in code order, each `Producers::Both`; `catalog()`, `spec(kind)`; `EVENT_ZONE = "event"`, `CAPTION_ZONE = "caption"` |
| `src/config.rs` [ONLY] | 110 (69) | `PlanConfig { per_kind, min_keep_gap: 150 ms, max_cut_share_permille: 400, max_chapters: 6 }`, `KindLimits { max_per_clip, min_gap, label_chars, header_chars }`; `Default` reads the catalog (NumberReveal 8 / 1,500 ms / 32; ListReveal 3 / 10,000 ms / 32 / header 24; FromTo 4 / 3,000 ms / 32; KeywordPop 12 / 800 ms) |
| `src/resolve.rs` | 182 (85) | `effective_tokens` (`offcut_text::tokenize` with the edits), `quantity_at` (every word of the range has a token, the quantity starts at the first and takes every token but trailing punctuation), `inherit` (D-85; display re-written by `format_quantity`), `label_digits_ok` |
| `src/validate.rs` [ONLY constructor] | 239 above its tests | `ValidPlan` (private fields; `source`, `items`, `is_empty`), `ValidItem` (with `marks`), `Dropped`, `DropReason` (twelve, snake_case), `RejectReason`, `PlanRejected` (thiserror); `validate`: V-1, the non-item drops, tokens once, the per-item checks, V-6, the sort |
| `src/validate_params.rs` (C.1's allowed split, used) | 307 | `check_item`: V-2, V-3, V-4, V-5, the span, the marks |
| `src/from_rules.rs` | 89 (28) | one `Detected` item per event, `confidence: Some`, `enabled` not copied; `t` unused (D-163) |
| `src/lib.rs` | 18 | §7.1's modules and re-exports (`RejectReason` too); `prompt` arrives in Prompt 97 |

**How `validate` reads §7.6, where the text leaves a choice:**
- **V-2.** `spec(kind)` and a `PlanConfig` entry, else `UnknownKind`. A `Detected` arm from a `Model` plan, a word-pointing arm from the `Rules`, or a producer the row does not allow → `WrongProducer`. The arm's kind differs from `PlanItem.kind` → `ParamsMismatch`. "Every required param is present": the typed arms make `value`, `from`, `to` and `word` always present, so the check that remains is a list with no point (both arms) or, for a word-pointing list, more points than the row's `ListPoints { max: 7 }` → `ParamsMismatch`. A detector list keeps its own count.
- **V-3.** Anchors empty or past the last word → `AnchorsOutOfRange`; the anchor rule's word count → `AnchorRule`. "Every `at` and `word` inside the anchors" is applied to `ListPoint.at`, `KeywordPop.word` (also a `Detected` one) and **each `QuantityRef` range** (non-empty and inside the anchors), each → `AnchorsOutOfRange`. A `Detected` arm's own times: its list items' `at` lie between the first word's start and the last word's end.
- **V-4.** Word-pointing labels only (the detector's texts are not checked): `label` and each list point's label against `label_chars`, the header against `header_chars`, by `chars().count()` → `LabelTooLong`; then `label_digits_ok` against every effective token → `LabelDigits`.
- **V-5.** `quantity_at` for each `QuantityRef`; FromTo through `inherit`; `ListReveal` becomes `ListItem { ordinal: i + 1, text: label, at: word start }`, `count` = points, header = label or `""`.
- **Marks** (§8.7). A word-pointing FromTo: the start of its `to` range's first word. A `Detected` FromTo: `parse_quantity` run from each effective token inside the anchors in turn, a found quantity's tokens skipped, and the start of the second one taken; none → `NumberUnresolved`.
- **Span** from the first anchor word's start to the last one's end, for every arm (the detector's span is the same rule, `from_to.rs` and `number.rs`).
- **V-6** in anchor order, ties by kind code: `OverLimit` when the kind has `max_per_clip` already; `TooClose` when the start is less than `min_gap` after the previous kept item of the kind ends; `ZoneTaken` when an earlier kept item of the same zone holds it, each item taking its zone over `[start, end + hold)` with the holds 1,200 ms (NumberReveal) and 1,500 ms (the other event-zone kinds), private constants of `validate.rs`; the caption zone is never taken.
- **Order**: by `(span.start, kind code, anchors.start)`. The dropped list holds the non-item drops first, then the item drops in plan order, then V-6's.
- **No panic, no `unwrap`, no indexing** in the crate's non-test code; every time sum saturates; a missing word time is `AnchorsOutOfRange`.

**Changes outside the crate.**
- `crates/offcut-types/src/events.rs`: `EventKind` gains `PartialOrd, Ord` (§7.4's `BTreeMap<EventKind, KindLimits>` needs a key order; V3's enum had none). Declaration order is the code order; the generated TypeScript changes by its doc comment only.
- `crates/offcut-types/src/lib.rs`: the generator writes `export const CATALOG_VERSION = 1;` after `PLAN_FORMAT` (§12). `CATALOG_VERSION` lives in `offcut-plan/src/catalog.rs` and `offcut-types` may not depend on the plan crate (TS §7), so the generator reads the constant's line from that file with `include_str!`; the generator's test asserts the constant is there, and fails if the line moves.
- `crates/offcut-plan/Cargo.toml`: `[dev-dependencies] serde_json` (the inline tests read JSON also without the feature `prompt`). The workspace does not deny unused dependencies (Entry 2), so no normal dependency waited.
- `web/src/gen/domain.ts` regenerated.

**Found, not fixed.**
- §12 lists `DropReason` and `Dropped` among the generated TypeScript types. They live in `offcut-plan`, which the generator in `offcut-types` cannot name, and nothing on the page reads them in V4 (they become an analytics enum in V6.1). Not generated; known issue 18.
- **D-165 at risk for two kinds (Prompt 97 decides).** The detector keeps two overlays apart by its windows only (150 ms lead, holds 1,400/1,700 ms: at least about 1,850 ms between a FromTo's end and the next one's start), while V-6's `min_gap` is 3,000 ms for FromTo and 10,000 ms for ListReveal. A rules plan with two FromTos 1.85-3 s apart would lose one to `TooClose`. `validate_rules_plans.rs` (Prompt 97) runs every labeled file; if one loses an item, the limit is the (assumption) to revisit, not the detector. Known issue 19.

**Tests.** Inline, 20: `catalog.rs` 2 (code order and unique names; §7.2's numbers), `config.rs` 1, `resolve.rs` 5 (§7.5's four: "ten thousand" → 10,000; "nine" for "ninety" → 9 minutes after inheritance from "to ten minutes"; a hidden word → `None`; "3x faster" without a "3" → false; plus inheritance leaving two units or none alone), `validate.rs` 11 (V-1 `Format` and `Catalog`; a model plan of the four kinds valid whole and sorted, with resolved values, list items and the FromTo mark; an edited number drawn as edited and an edited-away one dropped; the eight non-item fields `NotInThisBuild`; V-2 `WrongProducer` both ways and `ParamsMismatch` three ways; V-3 `AnchorsOutOfRange` and `AnchorRule`; V-4 `LabelTooLong` by characters and `LabelDigits`; V-5 `NumberUnresolved`; a `Detected` FromTo's mark and one without a second quantity; V-6 `OverLimit`, `TooClose`, `ZoneTaken` and a KeywordPop beside a visual; snake_case names), `from_rules.rs` 1. Every one of the twelve drop reasons is returned by some path (`UnknownKind` only for a kind missing from the catalog or the config, which no V4 kind is).

**How it was checked.**
- `cargo test -p offcut-plan` (default features and `--no-default-features`): 20 passed.
- `cargo clippy -p offcut-plan --all-targets -- -D warnings`, with and without default features: clean.
- `cargo build -p offcut-plan --target wasm32-unknown-unknown --no-default-features --release`: Finished.
- Line counts above: every file under 400 above its tests.
- `pnpm gen:types && sh scripts/check-gen-clean.sh`: clean with `domain.ts` staged.
- The SAB gate: on `v4-build`, no `zz` file, no test key; `pnpm fixtures` up to date; `pnpm check` exit 0 (`check-file-tree`: 298 files, 13 crates; `check-gen-clean` clean with `domain.ts` staged); **Rust 498 passed, 0 failed, 3 ignored** (Entry 5: 478; +20, the inline tests above); Vitest 92; `pnpm build` exit 0 (app shell 307.5 kB); `pnpm e2e` 12 passed; the frozen check lists the seven recorded one-line exceptions of Entry 4 and nothing else; no private path tracked.

**Open.** Known issues 18 and 19 (new).

---

## Entry 7 - 2026-10-22 - P97 (S4): plan prompt, validation tests, catalog snapshots

**What.** Prompt 97 (G 4.4 rows 6 and 8-11, G 4.5, Milestone 4; §7.3, §7.8, D-123, D-163 to D-165): `prompt.rs`, the crate's five test files, the three snapshots read and committed, the guards proven. `offcut-plan` is complete for V4; nothing outside the crate calls it yet (Prompt 98).

**`src/prompt.rs`** (behind the default feature `prompt`; the render bundle builds without it): `PromptInput { duration, words, prosody, sentences }`, `PAUSE_MARK_MS = 300` (assumption, V6.1), and three functions, all generated from the catalog and sent nowhere until V6.1 (D-164):
- `system_prompt()`: a preamble (the task), the transcript's line format, one paragraph per catalog row (name, purpose, anchor rule, params with their limits, zone, per-clip limit, gap), and seven rules. It carries §7.3's three rules verbatim and no example number of its own; the only numbers in it are the catalog's limits.
- `output_schema()`: JSON Schema (draft 2020-12) of an `EditPlan` restricted to the catalog. `items` is a `oneOf` of the four kinds with their word-pointing params under `"params"` (never `"detected"`), `confidence: null`; there is no `source` (the server sets it, TS §40.7); `format` and `catalog` are constants; `cuts` and `chapters` are empty arrays, `extras` an empty object, `music` null, so a plan written to it reads as an `EditPlan` once the server adds `source` (an inline test proves it).
- `user_message()`: `DURATION {ms}`, then `{i}\t{text}\t{start}\t{end}\t{energy_z:.1}\t{pitch_z:.1}` per word, `PAUSE {ms}` before a word after a silence of at least `PAUSE_MARK_MS`, a blank line after each sentence. The ASR text, not the user's edits.

**Read before accepting (G 4.4).** The system prompt's first paragraph, as committed:

> You plan the edit of a short talking-head video from its transcript. The speaker talks to the camera; you choose the moments to show as visuals over the video, using only the kinds of item listed below, and you point at the words of the transcript where each one belongs. Leave out an item rather than force one: a plan with few good items is better than a busy one.

One catalog paragraph:

> from_to: A change from one number to another: show both with bars. Anchors: from 3 to 40 words in a row. Params: from (required): the range of words that says the number; to (required): the range of words that says the number; label (optional): text of at most 32 characters. Zone: event. At most 4 in a clip, at least 3000 ms apart.

The rules: "Never write a number: point at the words that say it." / "Every item points at word indexes of the transcript." / "Every item has an id, a string of digits no other item of the plan has, and a confidence of null." / the half-open range form and that pointed words lie inside the anchors / "A label repeats or shortens what the speaker says; it holds no digit the speaker did not say." / one item per zone at a time and the distance between items of a kind / "Return JSON matching the schema; no prose." **Changed on reading:** the first draft did not say what to write in `id` and `confidence`, which the schema requires; the "id … confidence of null" rule was added before the snapshot was accepted. The schema was read whole (no `detected`, no `source`, the four kinds only), and the user message on `a01` (157 word lines, 16 `PAUSE 400` lines, a blank line after each of 17 sentences; insta trims the last).

**Snapshots** (`crates/offcut-plan/tests/snapshots/`, ignored by the tree script as `*.snap`): `catalog_snapshot__system_prompt@v1.snap`, `catalog_snapshot__output_schema@v1.snap`, `catalog_snapshot__user_message@v1.snap`, accepted with `cargo insta accept` after the reading above. The names carry `CATALOG_VERSION`.

**Test files.**

| File | Cases |
|---|---|
| `tests/validate.rs` | The prompt's 21 cases, each with the smallest plan, on the `a01` transcript: V-1 format; V-1 catalog; `UnknownKind` (a config without KeywordPop, since every V4 kind is in the catalog); `WrongProducer` (Detected in Model; word arm in Rules); `ParamsMismatch` (arm against kind; a list with no point); `AnchorsOutOfRange` (empty; past the end); `AnchorRule` (seven words for a NumberReveal; a list point outside the anchors); `LabelTooLong` (33 characters; 32 two-byte characters pass); `LabelDigits` ("3x faster"); `NumberUnresolved` ("spent 3000 hours"; a Detected FromTo with one quantity in its anchors); `OverLimit`; `TooClose` (two KeywordPops 80 ms apart); `ZoneTaken` ("90 minutes" inside the FromTo); a KeywordPop never `ZoneTaken`; `NotInThisBuild` for each of the five cut arms, a chapter, the hook, the post caption, a hashtag and music, one at a time; the sort order. Plus: a model plan of the four kinds (five items) on `a01` that validates whole, with resolved values, list points, the FromTo mark and spans from the anchors; INV-22 (word 23 edited "3000" → "4000" shows `4,000 hours`); the six non-item fields dropped beside five kept items. 24 tests |
| `tests/plan_from_rules.rs` | The reference clip's five events become five items with the detector's ids, kinds, anchors and params, and validate with the detector's spans; no event gives a valid empty plan. 2 tests |
| `tests/validate_rules_plans.rs` | All 31 labeled files (written ones timed as `precision_recall.rs`, `a02`'s ASR words and prosody, `w11`'s and `w23`'s edits passed to `detect` and `validate`): **"31 files, 0 items dropped"**; 41 items kept with the detector's spans, ids, params and anchors; each of the 11 FromTos has one mark, after its first word and not after its last. 1 test |
| `tests/catalog_snapshot.rs` | The three snapshots (`#![cfg(feature = "prompt")]`). 3 tests |
| `tests/ui.rs`, `tests/ui/valid_plan_private.rs` (+ `.stderr`) | `ValidPlan { source, items }` outside the crate: `error[E0451]: fields \`source\` and \`items\` of struct \`ValidPlan\` are private`. 1 test |

**Inputs, as built.**
- **The reference clip's five events** come from `fixtures/labeled/a01-scriptA.json`, which README says holds the reference clip's recognized text in written form; its word indexes match README's (23-24, 31, 54-59, 75-94, 132-133). A written file has no prosody, and README says the KeywordPop passes "only with its loudness and pitch", so word 31 "brutal" is given the labeled files' stressed value, 2.0. The detector then finds README's five in order: `3,000 hours` "editing by hand", `brutal`, 90 → 10 minutes, the list "3 REASONS", `$12k` "in sales". (The per-word times of the real recording are not in the repository; Entry 3.)
- The labeled loader and `synthesize` are copied into each of the four files that need them (the tree lists no `tests/common/` for this crate).

**Changed in Prompt 96's code.** The prompt's case list names "`AnchorRule` (`at` outside anchors)"; Prompt 96 had mapped a pointer outside the anchors to `AnchorsOutOfRange` (§7.6 allowed either). Now a `ListPoint.at`, a `KeywordPop` word (also a detector's) or a `QuantityRef` range outside the anchors is `AnchorRule`; anchors outside the transcript and a detector list item's time outside it stay `AnchorsOutOfRange`. One assertion of `validate.rs`'s inline V-3 test followed (`validate_params.rs` 307 → 315 lines).

**Other changes.**
- `Cargo.toml` of `offcut-plan`: dev-dependencies `insta`, `offcut-detect`, `trybuild` (beside Prompt 96's `serde_json`); `offcut-text` is a normal dependency already.
- `deny.toml` (known issue 4, closed): `tempfile` joins the `getrandom` ban's wrappers (`insta` writes pending snapshots through it; dev only), and `offcut-plan` joins the `offcut-detect` ban's wrappers (dev only). `cargo tree -p offcut-plan -e normal` resolves none of `web-sys js-sys wasm-bindgen getrandom rand wgpu offcut-detect insta tempfile`.
- `lib.rs`: `#[cfg(feature = "prompt")] pub mod prompt;`.

**Guards (G 4.5)**, each undone:
1. `ValidPlan`'s fields made `pub` → `tests/ui.rs`: "Expected test case to fail to compile, but it succeeded."
2. One word of the preamble ("edit" → "cut") → `catalog_snapshot.rs` with `INSTA_UPDATE=no`: "snapshot assertion for 'system_prompt@v1' failed"; no `.snap.new` left.
3. A `Detected` item in a `Model` plan → `WrongProducer`: `case_04_wrong_producer_detected_in_a_model_plan` passes.

**Known issue 19 closed.** The sweep drops nothing: on the labeled set no rules plan meets V-6's `min_gap` of 3,000 ms (FromTo) or 10,000 ms (ListReveal). The risk stays in principle for clips outside the set; the limits remain the (assumptions) of §7.2 (E-11).

**How it was checked.**
- `cargo test -p offcut-plan`: 23 inline + 3 + 2 + 24 + 1 + 1 = 54 passed; no pending snapshot. `INSTA_UPDATE=no CI=true cargo test -p offcut-plan` (what CI runs): the same. `--no-default-features`: 20 inline (no `prompt`), the snapshot file compiles empty, the rest pass.
- `cargo clippy -p offcut-plan --all-targets -- -D warnings`, with and without default features: clean. `cargo deny check`: ok.
- `cargo build -p offcut-plan --target wasm32-unknown-unknown --no-default-features --release`: Finished.
- The SAB gate: on `v4-build`, no `zz` file, no test key; `pnpm fixtures` up to date; `pnpm check` exit 0 (`check-file-tree`: 305 files, 13 crates); **Rust 532 passed, 0 failed, 3 ignored** (Entry 6: 498; +34: 3 inline in `prompt.rs`, 24 + 2 + 1 + 3 + 1 in the five files); Vitest 92; `pnpm build` exit 0 (app shell 307.5 kB); `pnpm e2e` 12 passed; the frozen check lists the seven recorded one-line exceptions of Entry 4 and nothing else; no private path tracked.

**Open.** Known issues 4 and 19 closed. Nothing new.

**Follow-up (same prompt).** The first commit of this entry (`dc389db`) struck open item 4 (the founder's brand-kit decision) instead of known issue 4: the script closing rows matched the first table. The follow-up commit restores open item 4 unchanged and closes known issue 4.

---

## Entry 8 - 2026-10-22 - P98 (S5): scene, renderer and binding on the plan contract (V3 look)

**What.** Prompt 98 (G 5.1 rows 1-11, §8.1-§8.8, §9.1-§9.3, §11.1; D-116, D-121, D-126, D-130 to D-138): the Rust side of the V4 contract. The scene reads a `ValidPlan` on program time with a `FontSet` and a `Theme`; the video is a draw command; the renderer draws a list in its order; the binding makes the rules plan and validates it. Every visual value is V3's. The page still speaks V3's protocol until Prompt 99, so the media cases that reach `setScene` fail in between (SAB rule 4); `pnpm e2e:media` was not run.

**REOPENED CONTRACT `offcut-scene/src/display_list.rs` (D-130, D-132; V2 §25.1).** `FontId` is gone: `FontKey(pub u16)`, the index of a face in a `FontSet`, serialized as its number. `CropRect` moved here from `lib.rs` (re-exported from the crate root, so `framing.rs` is unchanged). New: `VideoSlot { Main }`, `ImageKey(pub u16)`, and `DrawCmd::Video { slot, src, dst, opacity, mask }` and `DrawCmd::Image { image, rect, opacity }`, appended after `PopLayer` in §8.2's order. `scale` leaves `Video` alone (its `dst` is in output pixels) and scales an `Image`'s rect. Known issue 8 closed.

**REOPENED CONTRACT `offcut-scene/src/styles.rs` (D-136, D-137; V2 §25.1).** `StyleSpec` names roles and a font role, never an `Rgba` or a key: `font: FontRole`, `fill`, `stroke`, `active`, `pop: Role`, `panel: Option<PanelSpec>`, `motion: CaptionMotion` (V3's `stroke_color`, `active_color`, `pop_color`, `easing` and `motion_ms` fields are gone). All three styles are still V3's Clean row: `Display`, 64 lp, as spoken, 18 x 2, 5 words, `Text` fill, 6 lp `Stroke`, `Accent` active and pop, no panel, and the motion `{ Word, enter: None, Highlight, ease-out cubic, 120 ms }` (V3 showed a chunk at once; `None` keeps that, D-137's Fade is Prompt 105's). Its two V3 markers ("the Bold row", "the Tech row") are kept word for word. Known issue 9 closed.

**REOPENED CONTRACT `Renderer::render` (D-133; V2 §25.1).** `render(&mut self, sources: &VideoSources<'_>, list: &DisplayList, fonts: &FontSet)`, with `VideoSources { main: (&VideoFrame, Rotation) }` and the new `RenderError::Unsupported`. Known issue 10 closed.

**New files.**

| File | Lines (above its tests) | What |
|---|---|---|
| `src/theme.rs` [ONLY colour values] | 149 (123) | `Role`, `FontRole { Display, Body, Mono }`, `Theme` (§8.4's nine fields), `default_for`, `color`, `font`. V3's three colours moved here from `styles.rs` (white, `#101014`, `#FFD60A`); `default_for` returns V3's theme for all three styles: text white, stroke `#101014`, accent and accent_alt `#FFD60A`, panel `#101014` at 85% (217), backdrop opaque black, display and body Inter 700 (key 0), mono JetBrains Mono 700 (key 2). Panel and backdrop are drawn by nothing yet |
| `src/motion.rs` | 106 (68) | `MotionUnit { Word, Glyph }`, `MotionPreset { None, Fade, Pop }`, `ActiveTreatment { Highlight, Cursor }`, `CaptionMotion`, `POP_SCALE = 1.12`, `enter_at` (pure: `None` (1, 1); `Fade` opacity eased over `duration_ms`; `Pop` up over the first half and back over the second) |
| `src/video.rs` | 69 (39) | `Track::fixed(crop)`, `cmd(t, canvas)` → `Video { Main, src: crop, dst: (0, 0, canvas), opacity 1.0, mask None }` |
| `tests/timemap_identity.rs` | 218 | §8.10's S-1 test, below |

**Changed in the scene, by row.**
1. `display_list.rs`: above.
2. `fonts.rs`, `layout.rs`: `FontSet { faces }` (private), `FontFace { data: Arc<[u8]>, family, weight, coords }` (a `Debug` that prints the file's length, not its bytes), `FontSet::embedded()` (keys 0-3 in V2's order, built once per thread behind a `thread_local!` `OnceCell`), `get`, `EMOJI = FontKey(3)`. The shaping state (`Shaper`: parley's contexts and the registered files) is the `thread_local!` of V2, built empty; a set's files are registered on first use, each once, in V2's order (Inter, JetBrains Mono, Noto Emoji), so the collection is built from the set. A face's coordinates are read when it is registered, with V2's probe and family stack. `embedded()` keeps §8.3's infallible signature: if an embedded file cannot be read the set is empty and every use fails as `SceneError::Font` (V2's `Fonts::new` returned that error). `layout::shape(text, key, size, fonts)`; a run says which key drew it by its blob id and file. V2's `fonts::bytes`, `fonts::weight` and `layout::normalized_coords` are gone; the coordinates test moved to `fonts.rs`. The family names read from the files are "Inter Variable", "JetBrains Mono" and "Noto Emoji".
3. `theme.rs`, `styles.rs`, `motion.rs`: above.
4. `safe_area.rs`: `EVENT_ZONE` and `CAPTION_ZONE` are replaced by `ZoneId { Event, Caption, Full }` and `const fn zone(id)` with V3's numbers (Event y 860-1180, Caption y 1220-1500, Full the safe area).
5. `anim.rs`, `captions.rs`, `events/mod.rs`, `events/number_reveal.rs`: `OutMs`/`OutSpan` throughout. `captions::words(transcript, edits, map)` maps each caption word's `[start, end]` through `map.out_span` once (a word with no program time is not shown); `chunk(words, sentences, style, theme, fonts)` and `draw(chunk, t, style, theme, out)` read colours through `theme.color(role)` and the font through `theme.font(style.font)`. `draw` applies `enter_at`: a `Fade` puts the chunk in a layer, a `Pop` scales a word about its centre on the baseline; with V3's `None` neither emits anything, and the transform is V3's expression. `ActiveTreatment::Cursor` draws the fill only, with a new marker for Prompt 105 ("V4: the block after the spoken word"). `events::layout(item: &ValidItem, span: OutSpan, t, map, style, theme, fonts)` takes the `Detected` params; `t` and `map` wait for ListReveal and FromTo (Prompt 106). `number_reveal` takes its easing from `style.motion.easing` and its colours from the theme.
6. `video.rs`: above.
7. `lib.rs`: `SceneInput { transcript, plan: &ValidPlan, edit, clip, profile, time_map, theme, fonts }`; `SceneError::TimeMap` (an item's span with no program time); items shown when `event_overrides` says so or by default (D-145); `duration() == time_map.out_duration()`; `frame_at(OutMs)` puts the `Video` command first; `source_at(OutMs) -> SrcPos`; `fonts()`; `crop()` removed; `summary()` V2's. `scene` keeps clones of the map, theme and font set. The watermark marker stays.
8. Inline tests on the new calls (D-160), every expectation V3's. The `testing` helpers: `number_event(value, word, start, end)` now names the word its anchors point at, `plan(transcript, events)` is `from_rules` then `validate` (asserting nothing is dropped), `build` passes the identity map, Clean's theme and the embedded fonts, `bounds` gives no box for a `Video`. **Call-level changes, each forced by the contract and named here:** `busy()`'s three NumberReveals are put on words 2 ("three"), 10 ("wanted.") and 13 ("$250,000"): V3's helper anchored all three on word 0, and `validate` takes the span from the anchors, so all three would have become 500-750 ms and two would have been dropped as `TooClose`. The width test gains a `(Video, Video)` arm (same `src`, `dst` the whole canvas at each width). `a_number_that_cannot_fit_at_96_lp_is_dropped` asserted an empty frame; every frame now starts with the video, so it asserts the frame is the `Video` command alone. `number_reveal`'s and `captions`'s match arms list `Video` and `Image` (the workspace denies wildcard arms).

**Rows 9, 9b.**
- `tests/timemap_identity.rs`: the reference clip's plan is `a01` with "brutal" stressed, as Prompt 97's `plan_from_rules.rs` builds it (the reference recording's own word times are not in the repository, Entry 3); detect, `from_rules`, `validate` (five items, none dropped), the identity map of the clip's length. It asserts `duration == d`, TS §19.2's frame count, `source_at(t) == At(t)` at every frame, every caption word's start equal to its recording start and the word in the active colour at its first and last millisecond, and each NumberReveal (3,000 hours, $12k) drawn from its recording start to the end of its hold and fade and not one millisecond outside. Only public calls (`frame_at`, `source_at`, `captions::words`).
- `tests/v3_reference.rs`, comparison form (G 6.1), `#[ignore = "identity proof, S6"]`: the six inputs through `from_rules` → `validate` → `build_scene` on `TimeMap::identity(clip.duration)`, `Theme::default_for(Clean)`, `FontSet::embedded()`; every 7th frame serialized, the leading `Video` removed and `FontKey(n)` renamed to V3's font of index `n`, compared with `serde_json` equality (`f32` exact). The writing path is removed: nothing can rewrite the frozen files. It reports every input and every differing frame, then fails if any differs. `busy()`'s events are anchored as in row 8. `git diff --stat 09b1085 -- crates/offcut-scene/tests/snapshots/v3_reference` prints nothing.

**The identity reference while working** (`cargo test -p offcut-scene --test v3_reference -- --ignored --nocapture`, after rows 5, 7 and 10; the same result each time):

| Input | Frames compared | Items | Dropped | Differ |
|---|---|---|---|---|
| `busy_1080` | 43 | 3 | 0 | 4: frames 35, 42, 77, 154 |
| `busy_540` | 43 | 3 | 0 | 4: frames 35, 42, 77, 154 |
| `reference_1080` | 321 | 4 | 0 | 0 |
| `a01_1080` | 288 | 4 | 0 | 0 |
| `w01_1080` | 126 | 3 | 0 | 0 |
| `w07_1080` | 101 | 2 | 0 | 0 |

The four detector-built inputs (836 frames) are identical to V3's: glyphs, positions, colours, sizes, strokes, layers, the crop. The 8 differing frames are all inside the two `busy` events whose spans moved: frames 35 and 42 (1,166 and 1,400 ms), the 40% count-up now starting at 1,100 ms instead of 1,000; frame 77 (2,566 ms), the 40% fading (hold to 2,550) where V3's was opaque (hold to 2,600); frame 154 (5,133 ms), the "3" fading at 0.085 instead of 0.335 (its end 3,750 instead of 3,800). This is not a difference of the switch but of V3's hand-made spans, which no plan item can carry (D-121). Known issue 20 asks Prompt 100 for the decision.

**Row 10, `offcut-render`.**
- `composite.rs`: `video_of(list, canvas)`, V4's support rule (D-133): exactly one `Video`, the first command, slot `Main` (an irrefutable `let`, so a new slot fails to compile here), opacity 1.0, no mask, `src` finite, not empty and not before the frame's corner, `dst` finite, not empty and inside the canvas; any `Image` → `Unsupported`. The commands after the video go to the overlay; a list of the video alone costs no overlay pass (V3: an empty list).
- `video_pass.rs`: `load(device, queue, frame, src, rotation, (dst, target))`: the crop per call, and `rectangle(dst, target)` → `[x/W, y/H, W/w, H/h]` in a third `vec4` of the placement uniform (48 bytes). `shaders/video.wgsl`: the fragment measures the point in the rectangle as `(uv - dst.xy) * dst.zw` and outputs black outside it with `select`, after sampling, so the sample stays in uniform control flow. On the whole canvas `dst.zw` is exactly 1.0 and the sampled coordinate is V3's bit for bit. The entry points and their bindings are unchanged (the shader test passes as it was).
- `vello_backend.rs`: draws `&[DrawCmd]` with `&FontSet`; one cached `peniko::FontData` per `FontKey`, read again if the set's face for that key is another file (`Arc::ptr_eq`); the coordinates come from `FontFace.coords`; `Video` and `Image` are skipped.
- `lib.rs`: above. `gpu.rs` is unchanged.

**Row 11, `offcut-wasm-render`.** `Cargo.toml`: `offcut-plan = { workspace = true, default-features = false }`. The scene takes it the same way, so no crate turns `prompt` on in the browser build (`cargo tree -p offcut-wasm-render --target wasm32-unknown-unknown -e features -i offcut-plan` shows no feature; the bundle's one `serde_json` comes from `offcut-entitlement`, as at the baseline). `scene_api.rs`: `rules_plan(transcript, events) -> EditPlan` (a free function); `set_scene({ transcript, plan, edit, profile, timeMap })` → `SceneReport { dropped }`, through a native-testable `scene_of`: word-edit keys (`SceneInput`), a non-identity map (`TimeMapNotIdentity`, checked first), `validate` with `PlanConfig::default()` and the word edits (`PlanFormat`, `PlanCatalog`), `Theme::default_for(edit.style)`, `FontSet::embedded()`, `build_scene` (its error's name), each `E_INTERNAL`; `source_at(t_out_ms)` → `{ at }` / `{ held }`, `null` before a scene; `summary` unchanged. `session.rs`: `render_frame(frame, t_out_ms)` builds `VideoSources { main: (frame, clip.rotation) }` and draws `frame_at(OutMs)` with `scene.fonts()`; `RenderError::Unsupported` → `E_INTERNAL` `Unsupported`; `SceneError::TimeMap` → `TimeMap`. `lib.rs` (frozen) is unchanged. `hit` and `resize` are Prompts 104 and 112.

**V3 hand-over markers.** Two of Entry 1's eleven in `crates` are resolved here: `display_list.rs:57` (the closed `FontId` enum) and `styles.rs:31` ("A KeywordPop word (V4)"). One is added, `captions.rs` (the Tech cursor, Prompt 105).

**Tests added** (Rust +13 passed, +1 ignored; `v3_reference` no longer runs by default): `offcut-scene` inline 12 → 19 (`fonts.rs` 2, of which the coordinates test moved from `layout.rs`; `layout.rs` 1; `theme.rs` 1 (`default_for` equals V3's constants for each style); `motion.rs` 1; `styles.rs` 1 (no style is per-glyph); `safe_area.rs` 1; `video.rs` 1); `timemap_identity.rs` 1; `offcut-render` inline 2 → 5 (`rectangle`; a scene's list drawn, with a smaller `dst`; `Unsupported` for an image, two videos, a video not first, no video, a mask, half opacity, a `dst` past the canvas, an empty `src`); `offcut-wasm-render` `scene_api.rs` 3 (an empty rules plan on the identity map builds and reports `{ "dropped": [] }`; a non-identity map, format 2, catalog 2 and a bad word key refused with their details; a hook dropped as `not_in_this_build`); `session.rs`'s name test extended by `Unsupported` and `TimeMap`.

**How it was checked.**
- `cargo test -p offcut-scene -p offcut-render -p offcut-plan`: all pass, V3's expected values.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean. `cargo fmt --all --check`: clean.
- `git grep -n "FontId\|fn crop()" -- crates`: no output.
- `cargo check -p offcut-render --target wasm32-unknown-unknown`: Finished.
- `pnpm build:wasm`: `offcut_render_bg` **6,287,309 bytes** (Entry 1: 6,113,666; +173,643, under §21.1's 300 kB: the validator, the rules producer, the plan and time-map types' serde and the font registry); `offcut_core_bg` 713,306, unchanged.
- `node scripts/check-file-tree.mjs`: ok, 309 files, 13 crates; every file under 400 lines above its tests (`captions.rs` 342, `video_pass.rs` 335, `fonts.rs` 255).
- `cargo deny check`: advisories, bans, licenses, sources ok (its "unmatched wrapper" notes were there before this prompt).
- The SAB gate: on `v4-build`, no `zz` file, no test key; `pnpm fixtures` up to date; `pnpm check` exit 0 (`check-file-tree`: 309 files, 13 crates); **Rust 545 passed, 0 failed, 4 ignored** (Entry 7: 532/0/3; +13, and `v3_reference` now ignored until Prompt 100); Vitest 92; `pnpm build` exit 0 (app shell 307.6 kB, Entry 7 307.5: the render worker's bindings; `offcut_render_bg` gzip 2,990.79 kB, Entry 1 2,930.48); `pnpm e2e` 12 passed; no private path tracked. `pnpm e2e:media` not run (SAB rule 4: the page still sends `events` until Prompt 99).
- The frozen check lists the seven recorded one-line exceptions of Entry 4 and nothing else.

**Differs from the plan or the guide.**
- `captions::words` takes the time map and returns program times (§8.1's pseudo-code maps after it); one step, the same rule.
- `dst` reaches the video pass as a uniform the fragment shader reads, not as the quad's vertices (§9.3): the pipeline, its entry points and its bindings stay V2's, and the whole-canvas case is exact.
- `VideoSources` holds a `VideoFrame` on every target for now; its native form arrives with `render_to_image` (Prompt 101).
- `src` is not checked against the frame's size: the renderer knows it only after the import, and the sampler holds the edge.

**Found, not fixed.**
- **An item span past the time map fails the whole scene.** `SceneError::TimeMap` (§8.1) refuses a scene whose item has no program time. On the page, word times are clamped to the decoded audio's length (`word-timestamps.ts`), not to `ClipInfo.duration`, which the identity map will be built from (G 5.2 row 6). If the audio is longer than the container's duration, an item on the last word would fail `setScene`. Prompt 99 should build the map from a duration that holds every word, or confirm the two lengths agree.
- **A scene's profile must match the canvas it is drawn on.** `dst` is the profile's canvas; drawn on a smaller canvas the list is `Unsupported` (V3 drew it at the wrong scale). `export-job.ts` sets the scene and attaches the canvas back to back, with no render between, so nothing hits it today.

**Open.** Known issues 8, 9 and 10 closed. Known issue 20 (new, for Prompt 100). Open items 1-5 unchanged.

---

## Entry 9 - 2026-10-22 - P99 (S5): protocol, worker and pipeline on the plan contract (V3 look)

**What.** Prompt 99 (G 5.2 rows 1-8, G 5.3, M-5; §14.1, §14.4, §15.1, §15.2, §16.1, §16.2; D-124, D-126, D-129): the page speaks the V4 contract. After detection the pipeline asks the render worker for the rules plan, keeps it in the clip store and sends it to `setScene` with the clip's identity time map; the worker's `set_scene` validates it in Rust and answers what it dropped. The clock, the seek and the preview loop run on program time. The app looks exactly as V3. This ends the media failures of SAB rule 4 between Prompts 98 and 99.

**Founder's decision, 2026-10-22 (known issue 20).** "Re-record just those two files with the old engine, with the pop-ups placed on real words": Prompt 100 re-derives `busy_1080.json` and `busy_540.json` with V3's scene code at `09b1085` on the anchored events of Entry 8, the other four reference files unchanged. Recorded in the row; nothing of it is done in this prompt.

**REOPENED CONTRACT `web/src/workers/protocol.ts` (D-124, D-126; V1).** `RenderWorkerApi` gains `rulesPlan({ transcript, events }) → EditPlan` (stateless); `setScene({ transcript, plan, edit, timeMap, profile }) → SceneReport` (was `events`, answered nothing); `previewPlay → PreviewStats`; `previewSeek({ at: OutMs })` (type only, its handler is Prompt 112's); `ClockSync.audioMs: OutMs`. New types: `SceneReport = { dropped: Dropped[] }` and `PreviewStats = { frames, late, maxDriftMs: DurMs, downscaled }` (moved from `preview-loop.ts`, with `downscaled`, always `false` until Prompt 112). `assemble`, `release` (Prompt 100), `hitTest`, `styleThumbnails` (Prompts 113, 116) are not added. Known issue 11 advanced (P99's members done; P100, P113, P116 remain).

**REOPENED CONTRACT `web/eslint.config.js` `MINTING` (D-126; V2).** `control-preview.ts`'s entry mints `OutMs` in `toOutMs` (was `TimeMs` in `toTimeMs`). **One entry more than the plan says:** `run-pipeline.ts` mints `TimeMs` in `recordingTime`. The identity map's `src` is a `Span` of two `TimeMs` (`{ start: 0, end: duration }`, G 5.2 row 6), a use-case may not cast to a brand, and no typed `TimeMs` exists on the page to build it from; this entry is the smallest change that lets `identityTimeMap` exist there without a second rule. The comment's count ("Seven files", already stale at eight) now says nine. Drill: a second `as TimeMs` in `run-pipeline.ts` and an `as TimeMs` in `control-preview.ts` were each refused ("In this file one cast is allowed: to TimeMs, in the last return of recordingTime()" / "… to OutMs, in the last return of toOutMs()"), then undone. Known issue 13 closed.

**`Dropped` and `DropReason` are generated (known issue 18 closed).** `load-render.ts` (the `wasm` layer, which may import `gen/` only) must type `set_scene`'s answer, and `protocol.ts` imports `gen/` only; the prompt names "the generated `Dropped`". Both types moved from `offcut-plan/src/validate.rs` to `offcut-types/src/plan.rs` with `#[derive(TS)]`, unchanged in shape and serde (`kind: Option<EventKind>`, the twelve reasons in snake case); `validate.rs` re-exports them (`pub use offcut_types::{DropReason, Dropped}`), so every Rust path that named them still does (`offcut_plan::Dropped`, `crate::validate::DropReason`). The generator visits them after `MusicCue`; `web/src/gen/domain.ts` gains `Dropped` and `DropReason` (+9 lines). **Test expectation changed** (SAB rule 4, named): `offcut-types`' `typescript_declares_every_type_and_limit_once` counts 81 declared names (was 79), the two new types. `offcut-plan`'s 54 tests pass unchanged.

**Changed, by row.**
1. `protocol.ts`: above.
2. `wasm/load-render.ts`: `SceneInput = { transcript, plan, edit, profile, timeMap }`; `SourcePos = { at: TimeMs } | { held: TimeMs }`; `RenderSession.set_scene → { dropped: Dropped[] }`, `source_at(tOut: OutMs) → SourcePos | null`, `render_frame(frame, tOut: OutMs)` (the module's `number` typed); `RenderApi.rulesPlan(transcript, events)` over the binding's `rules_plan`. `resize` is Prompt 112's.
3. `render.worker.ts`: the `rulesPlan` handler; `setScene` passes the plan and the map, keeps `{ transcript, plan, edit, timeMap }` for the export's switch and back, and answers the report (`Promise.resolve`: a handler answers a promise, and the lint refuses an `async` function without `await`); `detect` keeps `{ transcript, prosody, events }` on the open session (`Session.detection`, D-146), so it goes with `closeSession`; a module variable written and not yet read would fail `no-unused-vars` until Prompt 114's `redetectSentence` reads it.
4. `render/preview-job.ts`, `render/preview-loop.ts`: `previewTime()` and the loop's clock are `OutMs`; frame `n` is program time `floor(n x 1000 / 30)`, `session.source_at(tn)` gives the recording time (a held position shows its frame), `source.frameAt` takes it, `render_frame(frame, tn)`; `previewPlay` answers the loop's statistics after the frame-leak check. A `null` source position (no scene) ends the loop.
5. `render/export-job.ts`, `render/export-loop.ts` (D-129): `SceneState` is `{ transcript, plan, edit, timeMap }`, so the switch to the export profile and back passes the plan and the map; `Session.detection` (row 3). In the loop one number `t` is the frame's time, typed `TimeMs` for the source and `OutMs` for `render_frame`, with the identity map named in the comment (V5 reads it from `source_at`). `export-loop.ts` 397 → 399 lines.
6. `usecases/run-pipeline.ts`: after the feed lines, `rulesPlan` → `clipStore.setPlan(plan)` → `setScene({ transcript, plan, edit, timeMap: identityTimeMap(clipInfo.duration), profile: "preview" })`; `ready` gets the map. `identityTimeMap(duration)` is the one place in `web/src` that writes a time map. The `pipeline_done` counts are V3's (from the events).
7. `usecases/control-preview.ts`, `web/eslint.config.js`: above.
8. `state/clip-store.ts`: `plan?: EditPlan`, `timeMap?: TimeMap` (in memory only, D-124, D-82); `setPlan(plan)` and `setEvents(events)` (no state change); `ready` takes the map. `reset` drops both, as it drops everything.

**The look (G 5.3), on D1** (`d1`), the keyed build, the reference clip, Chrome through a temporary Playwright check (`zz-look.spec.ts` and `zz-pw.config.ts`, deleted after; the 15 canvas pictures stay in `web/test-results/zz-look/` on D1):

| Check | Seen |
|---|---|
| Captions | V3's Clean: Inter 700, white with the dark stroke, the spoken word in `#FFD60A`, two lines in the caption zone. The page offers no style choice before Prompt 116, so Bold and Tech are shown by `theme.rs`'s test (V3's colours and font for all three, Entry 8) |
| NumberReveal | "3,000 hours" with "editing by hand" counts up (1,464 hours at 12.4 s, 3,000 at 12.8 s); "$12k" with "in sales" counts up ($9,408 at 63.3 s). FromTo, ListReveal and KeywordPop are not drawn (only captions at "from 90 minutes to 10 minutes") |
| Console | No console message and no page error on the page during processing and play |
| `setScene`'s answer | `{"dropped":[]}`, read where the render worker's replies reach the page |
| `previewPlay`'s answer | `{ frames: 70, late: 7, maxDriftMs: 27.1, downscaled: false }` for a 3 s play |
| Feed | Checked by the media suite's two feed cases (below) |

**Prompt 98's finding, measured.** On the reference clip the last word ends at 71,520 ms and `clipInfo.duration` is 74,705 ms: every word has a program time on the identity map. The rest of the media suite's clips built their scenes too (below), so none of them hit `SceneError::TimeMap`. The risk stays for a clip whose decoded audio runs past its video track (word times are clamped to the audio's length); it is still recorded under Entry 8's "Found, not fixed".

**How it was checked.**
- `pnpm --filter web exec tsc --noEmit` (app and e2e) and `eslint .`: clean after rows 1-8.
- `git grep -n 'segments: \[{ kind: "source"' -- web/src`: one line, `run-pipeline.ts:51`, the identity helper. `git grep -n "toTimeMs" -- web/src`: no output.
- `cargo test -p offcut-types -p offcut-plan`: 74 passed and 1 ignored (the generator), and 54; clippy clean.
- `pnpm build:wasm`: `offcut_render_bg` 6,287,309 bytes and `offcut_core_bg` 713,306, the same as Entry 8.
- `cargo test -p offcut-scene --test v3_reference -- --ignored`: the same as Entry 8: `reference_1080`, `a01_1080`, `w01_1080`, `w07_1080` identical (836 frames), `busy_1080` and `busy_540` 4 frames each, Prompt 100's list (known issue 20, decided).
- `pnpm e2e:media` on D1, keyed build: **49 passed, 1 skipped (10.8 min)**, the 48 cases of the baseline with their setup (Entry 1: 49 and 1), every expectation V3's, among them "preview before sign-in: … the player plays", "pause and resume", "events: the clip, its four stages, the result, and one preview for two plays" (the `pipeline_done` counts), both feed cases, and "second recording: … its preview plays" (the founder's second clip, its scene built too). The key was set in the build's own shell only and is not in the environment after (`env | grep -c` 0).
- The SAB gate: on `v4-build`, no `zz` file, no test key in the environment or in the gate's `dist`; `pnpm fixtures` up to date; `pnpm check` exit 0 (`check-file-tree`: 309 files, 13 crates, longest `export-loop.ts` 399; `check-gen-clean` clean with `domain.ts` staged); **Rust 545 passed, 0 failed, 4 ignored** (Entry 8's counts: the type move added no test); Vitest 92; `pnpm build` exit 0 (app shell 307.8 kB, Entry 8 307.6: the render worker's new handler and the pipeline's plan step); `pnpm e2e` 12 passed; the frozen check lists the seven recorded one-line exceptions of Entry 4 and nothing else; no private path tracked.

**Differs from the plan or the guide.**
- `MINTING` gains `run-pipeline.ts` (above).
- `Dropped` and `DropReason` live in `offcut-types` (above); §12's list of generated types is now true as written.
- `detect`'s keeping is on the session object, not in a module variable (row 3).
- The export loop does not call `source_at` (D-129); the preview loop does (§15.2).

**Found, not fixed.**
- `maxDriftMs` is typed `DurMs` but carries a fraction (27.11): the clock's time is a float and V3's loop never rounded it. Prompt 124 reads it in `preview-sync.spec.ts`; rounding there or in the loop is that prompt's choice.

**Open.** Known issues 13 and 18 closed; 11 advanced; 20 decided. Open items 1-5 unchanged.

---

## Entry 10 - 2026-10-22 - P100 (S6): identity proof passed; program audio (identity) in the worker and the preview

**What.** Prompt 100 (G Phase 6, M-6; D-116, D-117, D-127 to D-129, §10, §11.2, §14.1, §15.4, §16.2): the identity proof passes, and program audio exists from the DSP crate to the preview. The seams S-1 to S-6 are in place and proven to change nothing: the same display lists, the same audio, the same media results.

**Identity proof passed at S6.** `cargo test -p offcut-scene --test v3_reference -- --nocapture`: **"6 inputs, 922 frames, identical"** (busy_1080 43, busy_540 43, reference_1080 321, a01_1080 288, w01_1080 126, w07_1080 101), value for value after D-117's normalisation, `f32` by equality. The `#[ignore]` of Prompt 98 is removed; the test runs in CI from this commit until Prompt 101 deletes it. **No file of Prompts 98-99 needed a change**: the switch moved nothing.

**The two `busy` files, re-derived (the founder's decision, known issue 20).** In a scratch worktree at `09b1085` (Prompt 93's commit: V3's scene code and Prompt 93's writer, in the scratchpad, outside the repository, removed after), `busy()`'s first two NumberReveals were given the spans of the words Prompt 98 anchors them on (40% on "three", 1,100-1,350 ms; 3 on "wanted.", 3,500-3,750 ms; `$250k` already on "$250,000", 6,550-7,000) and `OFFCUT_V3_REFERENCE_WRITE=1 cargo test -p offcut-scene --test v3_reference` was run there, with its own target directory. No file under `crates/offcut-scene/src` differed from `09b1085`. The writer reproduced `reference_1080`, `a01_1080`, `w01_1080` and `w07_1080` byte for byte (`cmp`), which also shows it is deterministic; `busy_1080.json` and `busy_540.json` changed in exactly the four frame lines Entry 8 named (frames 35, 42, 77, 154; 42,159 → 42,180 and 42,399 → 42,419 bytes) and were copied in. **Frozen-file exception, recorded:** `git diff --stat 09b1085 -- crates/offcut-scene/tests/snapshots/v3_reference` lists these two files only (4 lines each); the prompt's "prints nothing" holds for the other four. The files were written by V3's code, as D-117 requires. In `tests/v3_reference.rs` the two events carry the same spans (the spans are unused by `validate`, which reads the anchors) and the module comment says why the two files are newer.

**Program audio, Rust.**
- `crates/offcut-dsp/src/program.rs` [ONLY place that changes duration]: `ProgramConfig {}` (empty in V4), `MusicInput { pcm48, loudness }`, `ProgramAudio { samples, loudness }`, `assemble(out48, map, speech, music, cfg)`: a map that is not the identity, or any music → `DspError::Unsupported`; empty input → `Empty`; else `out48.to_vec()` and `loudness::integrated` (the chain's meter), falling back as the chain does to `ChainConfig::default().absolute_silence`. The map's length is not compared with the samples': the identity plays the recording as it is (on the reference clip the two agree exactly, below). `speech` and `cfg` are unused until V6.3 and V6.2.
- `src/silence.rs`: `silence_profile(out48) -> SilenceProfile`, the RMS of each 480-sample window in dB, at least `FLOOR_DB = -100.0`; a short last window is measured over what it has. Nothing calls it (D-163).
- `src/lib.rs`: the two modules and their re-exports; `DspError::Unsupported` (new arm); the crate comment now names `assemble` as the one function that may change a length (INV-10), and in V4 does not.
- `tests/program_identity.rs` (the one new file under `offcut-dsp/tests/`, through V3's stamp helper, copied from `loudness_target.rs`): on `speech_60s_portrait` and `speech_20s_long_pause`, after `run_chain`, `assemble` on the identity map returns every sample bit for bit and a loudness within 0.01 LU of the chain's; a map with a second taken out, a map with a hold, and a music input are each `Unsupported`; `silence_profile` of 1 s of zeros is 100 windows at -100 dB and of a full-scale 1 kHz sine 100 windows at -3.01 dB (±0.01). **Dev-dependency added:** `serde_json` (the workspace's version) in `offcut-dsp`, to read the two non-identity maps the test refuses: `TimeMap` has no other constructor (D-164).
- `crates/offcut-wasm-core/src/audio_api.rs`: `assemble(out48, time_map, speech)` → `{ prog48: Float32Array, loudness }` (PCM by copy, D-110; a bad map or span list is `E_INTERNAL` `TimeMap`/`Speech`); `dsp_detail(DspError)`, the one mapping to `E_DSP` details, now used by `run_chain` too (`Empty`, `NonFinite`, `Unsupported`), with an inline test.

**REOPENED CONTRACT `AudioWorkerApi` (D-127, D-128; V3 §21.1).** `assemble({ out48, timeMap, speech }) → { prog48, loudness }` (`out48` transferred in, `prog48` out, nothing kept) and `release()` (one-way). The protocol's comment says what the worker still never does (detect silence, plan a cut) and that only `assemble` applies a map, the identity in V4. Known issue 12 closed (with it known issue 1, struck in Entry 2 as "closed by P100"); known issue 11 advanced.

**Program audio, TypeScript.**
- `wasm/load-core.ts`: `CoreApi.assemble(out48, timeMap, speech)`.
- `workers/audio.worker.ts`: the `assemble` handler (the core's result returned, so `serveWorker` transfers `prog48`) and `release` (`kept = null`), `oneWay: ["release"]`; the file's comment gains D-127's sentence ("`assemble` applies a time map it is given; in V4 the only map is the identity") and keeps "It decides nothing about the audio".
- `usecases/run-pipeline.ts`, step F: `opfs.writeAudio(out48)` first, then `pool.audio.assemble({ out48, timeMap, speech: [] })` with `out48` transferred, then the transcript and edit records, then `clipStore.ready({ …, prog48 })`. `Run.audioStarted` is set when the chain is given the clip; `releaseAudio(run)` posts `pool.audio.notify("release", {})` on the model-failure end, the no-speech end and in the catch (every thrown failure, an `assemble` failure included, which follows V3's `E_DSP` path), each after the chain has settled (D-90), and never for a run that did not reach the chain (a release would otherwise create the audio worker). The two comments touched in Prompts 99-100 were tightened to keep the file at 397 lines (401 otherwise).
- `state/clip-store.ts`: `prog48` replaces `out48`; `usecases/control-preview.ts`: `attach(canvas, prog48)`; `ui/components/PreviewPlayer.tsx`: reads `prog48`. The export still reads `out48.f32` from OPFS (D-129, `start-export.ts` unchanged).
- **Not in the prompt's file list, not changed:** `import-clip.ts`'s "start over" (`dismissClip`). It cannot run while a clip is processing (it returns early), and every early end of a run now releases, so a release there would only create the audio worker after an import rejection. Cancel's release is Prompt 119's.
- **V3 §23 item 25 closed** (D-128): the audio worker no longer keeps a clip's cleaned voice after a run that ended before `measureProsody`.

**G 6.3, on D1** (`d1`; keyed build; the reference clip in Chrome through a temporary `zz-audio.spec.ts` with `zz-pw.config.ts`, deleted after): the `prog48` the audio worker handed the page has **3,585,840 samples = `clipInfo.duration` 74,705 ms × 48**, and its FNV-1a hash (`c6408129`) equals that of the `out48.f32` written to OPFS: the preview plays the cleaned voice bit for bit, as V3 did. Loudness −14.39 LUFS (the chain's target −14, within its 0.5 LU). `setScene` answered `{"dropped":[]}`; a 3 s play answered `{ frames: 42, late: 12, maxDriftMs: 30.4, downscaled: false }`; no console message or page error.

**How it was checked.**
- `cargo test -p offcut-scene --test v3_reference -- --nocapture`: 1 passed, "6 inputs, 922 frames, identical".
- `cargo test -p offcut-dsp --test program_identity`: 3 passed. `cargo test -p offcut-dsp --lib`: 38 (2 new). `cargo test -p offcut-wasm-core`: 7 (1 new).
- `git diff --name-only 550ff8f -- crates/offcut-dsp/tests`: `program_identity.rs` only (a new file).
- `cargo clippy --workspace --all-targets -- -D warnings`: clean; `cargo check -p offcut-wasm-core --target wasm32-unknown-unknown`: Finished.
- `pnpm build:wasm`: `offcut_core_bg` **743,885 bytes** (Entry 1: 713,306; +30,579, the time map's and spans' deserialization and `assemble`), `offcut_render_bg` 6,287,309, unchanged.
- `pnpm e2e:media` on D1, keyed build: **49 passed, 1 skipped (16.1 min)**, the 48 cases of the baseline. Every case ran slower than in Entry 9 (10.8 min), the exports too, which do not call `assemble` (they read `out48.f32`): load on D1, not this change; Prompt 126's bench is the measurement. The key was unset after (`env | grep -c` 0).
- The SAB gate: on `v4-build`, no `zz` file, no test key in the environment or in the gate's `dist`, the scratch worktree removed (`git worktree list`: one); `pnpm fixtures` up to date; `pnpm check` exit 0 (`check-file-tree`: 312 files, 13 crates); **Rust 552 passed, 0 failed, 3 ignored** (Entry 9: 545/0/4; +7: `v3_reference` now runs, `program_identity.rs` 3, `program.rs` and `silence.rs` 1 each, `audio_api.rs` 1; the ignored are V3's three again); Vitest 92; `pnpm build` exit 0 (app shell 308.8 kB, Entry 9 307.8: the workers' glue for `assemble`); `pnpm e2e` 12 passed; the frozen check lists the seven recorded one-line exceptions of Entry 4 and `crates/offcut-dsp/tests/program_identity.rs` (new, allowed); no private path tracked.

**Differs from the plan or the guide.**
- The two `busy` reference files are re-derived (above), by the founder's decision.
- `assemble` does not compare the map's length with the samples' (above).
- **D-161 and step F disagree.** D-161 says `rulesPlan` and `assemble` are measured inside `stage_timing detect_scene` ("from `openSession` to the first `setScene` resolving"); D-127 and §16.2 put `assemble` in step F, after that stage has ended. The stage boundaries of V3 are kept (the bench compares against them), so `assemble` is in no stage's timing. For Prompt 126 (measure) and the copy-back.
- `offcut-dsp` gains the dev-dependency `serde_json` (above).

**Found, not fixed (for the copy-back, Prompt 127).** BP §6.1 A2 still places `timemap.rs` in `offcut-plan` and "`QuantityRef` resolution in `build_scene`"; the time map is in `offcut-types` (D-125) and resolution in `validate` (D-121). §24's item 1 covers TS §40.9 only.

**Open.** Known issues 12 and 20 closed; 11 advanced. Open items 1-5 unchanged.

---

## Entry 11 - 2026-10-22 - P101 (S7): native rendering, SSIM, two video-only goldens; identity reference retired

**What.** Prompt 101 (G Phase 7, M-7; D-134, D-135, §9.1, §9.3, §9.4, §21.1): the identity reference is retired; the renderer draws into memory on a software adapter; the SSIM measure exists; the two video-only goldens are committed and read the same on D1's WARP and the runner's lavapipe. Two commits, as the prompt says: `1623d88` (`wip`: the code, the goldens, Mesa in CI, the reference retired) and this one (the lavapipe reading, `experiments.md`, this entry).

**G 7.1, the identity reference retired.** `git rm` of `crates/offcut-scene/tests/v3_reference.rs` and the six `tests/snapshots/v3_reference/*.json` (its record stays in Entries 3, 8 and 10: "6 inputs, 922 frames, identical" at S6). TS §5 loses its seven "temporary, V4 S2 to S7" lines; `snapshots/` is the last entry of the scene's `tests/`. `offcut-scene/Cargo.toml` keeps `offcut-detect` and `serde_json` as dev-dependencies (`timemap_identity.rs` uses both); their comment no longer names the reference. `git ls-files crates/offcut-scene/tests | grep v3_reference`: no output.

**G 7.2, native rendering (§9.1, §9.3).** Both paths share every line but the source and the target:
- `gpu.rs`: the canvas is one of two `Target`s, `Canvas { surface, config }` and, natively, `Image { texture, readback }` (an `Rgba8Unorm` texture with `RENDER_ATTACHMENT | COPY_SRC`, and a `MAP_READ` buffer made once per size). `frame()` gives the texture a frame is drawn to; `present(frame)` shows a canvas frame; `read(&frame)` copies the image texture to the buffer with each row padded to 256 bytes (`padded_row`: 540 px x 4 = 2,160 → 2,304), maps it, polls with `PollType::wait_indefinitely()` and strips the padding row by row into an `RgbaImage`. `Gpu::new_headless(width, height, dx12)`: DX12 only on Windows, Vulkan only elsewhere, `force_fallback_adapter: true`, the same `device()` (default limits, the lost flag) as the browser path; `NoAdapter` when there is no software adapter, or when the build has no native backend (the browser build, and a native build without the dev-dependency's features). `resize` remakes the image target; `set_canvas` refuses a headless renderer (`Surface`). The V3 marker `gpu.rs:33` ("V4: an image in memory takes the place of the canvas") is resolved.
- `video_pass.rs`: `load` (a `VideoFrame`, copied in by the browser) and `load_image(&RgbaImage, src, rotation, (dst, target))` (`queue.write_texture`, its length checked) share `texture_for` (the texture of a frame's size, kept between frames) and `place` (the placement uniform of Prompt 98). The V3 marker `video_pass.rs:174` is resolved.
- `lib.rs`: `RgbaImage { width, height, data }` (sRGB-encoded, straight alpha, row-major); `Renderer::on(gpu)` builds the passes for both constructors; `render` and `render_to_image` share `video_of` (the lost check and V4's support rule) and `draw` (the overlay of the commands after the video, then one pass); behind `cfg(not(target_arch = "wasm32"))`: `new_headless(width, height)`, `render_to_image(source, rotation, list, fonts)`, and `adapter_info()`, the one call §9.1 does not list, for the test's adapter assertion. `composite.rs`: `present` becomes `draw(gpu, target, video, with_overlay)`, which submits the pass; presenting or reading is the caller's.
- **Known issue 3 closed, the decision recorded.** `new_headless` chooses the DX12 shader compiler (`lib.rs`, `dx12_compiler`): wgpu's own `WGPU_DX12_COMPILER` when it is set, else the newest `%ProgramFiles(x86)%/Windows Kits/10/bin/<version>/x64/dxcompiler.dll` by path (D1: 10.0.26100.0, DXC 1.8.2502.11, wgpu's minimum), else wgpu's `Auto`. Rejected: `wgpu`'s `static-dxc` feature (it downloads a prebuilt DXC while building) and putting the SDK on `PATH` (an `unsafe` `set_var` in Rust 2024). Off Windows there is no `ProgramFiles(x86)`, so nothing changes there. **Drill:** a temporary test (deleted) drew a `Video` and a `FillRect` over it: with the SDK's DXC the pixel inside the rectangle was exactly `[255, 214, 10, 255]`; with `WGPU_DX12_COMPILER=fxc` the process died with `STATUS_ACCESS_VIOLATION` (`0xc0000005`), Entry 1's crash. The two video-only goldens pass under FXC too: they draw no overlay, so Vello never runs. The choice matters from Prompt 107's goldens on.
- `gpu.rs` was 428 lines above its tests with the compiler choice in it; the choice moved to `lib.rs`'s native block (beside `new_headless`), leaving `gpu.rs` under 400. No new source file.

**`tests/common/ssim.rs`** (included by `golden_frames.rs` with `#[path]`; TS §5 lists it, no `mod.rs`): BT.709 luma (`0.2126 R + 0.7152 G + 0.0722 B` of the encoded values), the 8 x 8 windows that tile the image, `C1 = (0.01 x 255)²`, `C2 = (0.03 x 255)²`, the mean over windows. Its two tests: an image against itself is 1.0 and against its inverse 0.5 or less; a one-step change keeps it above 0.999 and an image halved in brightness falls below 0.98.

**`tests/golden_frames.rs`** (§9.4): `pattern(w, h)` (a diagonal gradient, an 8 x 8 grid of lines 2 px wide in `#F0F0F0`, a disc `#E63946` at a third of the width, radius a sixth of the shorter side), `video_only(src)` (one `Video { Main, src, dst: 0, 0, 540, 960, opacity 1, no mask }`), a `block_on` written in the file (`Waker::noop()`; no dependency), the adapter assertion (`DeviceType::Cpu`, else the test fails with the adapter's info), `GOLDEN_SSIM_MIN = 0.98`, `OFFCUT_GOLDEN_UPDATE=1`, the missing-golden message, `target/golden-out/<name>.png` written for a golden that differs, and the line `ssim <name> = … on <adapter> (<backend>)`. The cases: `crop_landscape_050` (1920 x 1080, `src` x 984.375, w 607.5, h 1080: TS §19.3 at offset 0.5, written by hand because `framing.rs` still centres every clip until Prompt 106) and `punch_in_110` (1080 x 1920, `src` the centre 1/1.1: w 981.82, h 1745.45, x 49.09, y 87.27).

**The two PNGs, looked at** (`fixtures/golden/`, 540 x 960 RGBA, 25,824 and 44,850 bytes): `crop_landscape_050` is the strip right of the centre of the landscape pattern (source x 984-1,591): no disc (its centre is at x 640), two vertical grid lines (source x 1,200 and 1,440, at about x 192 and 405), the gradient green to magenta toward the bottom right. `punch_in_110` is the centre of the portrait pattern at 1.1: the disc at about x 171 with a radius of about 109, the eight grid columns, the lines about 1.1 px against 1.0 px without the punch-in (thicker by the 1.1, which at this size is a fine difference).

**Drills.** A golden darkened by half by hand (not committed): `punch_in_110: ssim 0.7726 < 0.98 on Microsoft Basic Render Driver; what was drawn is in …/target/golden-out/punch_in_110.png`; restored byte for byte (`cmp`), passing again. A golden moved away: `crop_landscape_050: no …/fixtures/golden/crop_landscape_050.png: run with OFFCUT_GOLDEN_UPDATE=1 and review the image`; restored. FXC: above.

**G 7.3, CI.** `ci.yml`'s Linux job, before `cargo test`: "Install the software Vulkan driver" (`apt-get update`, `apt-get install -y -q mesa-vulkan-drivers`, `dpkg -s mesa-vulkan-drivers | grep Version`); `cargo test`'s step gains `env: INSTA_UPDATE: "no"`, `CI: "true"`; then "Golden frames, with their readings" (`cargo test -p offcut-render --test golden_frames -- --nocapture`, because `cargo test` keeps a passing test's output, and G 7.3 reads the line); and "Keep the frames that differ from their goldens" (`actions/upload-artifact`, `target/golden-out`, on failure only; the same pinned action as the Playwright report). Phase 17 (Prompt 125) completes the job.

**The reading (G 7.3), run 39825539363 on `1623d88`:** `mesa-vulkan-drivers` 25.2.8-0ubuntu0.24.04.4 (installed by the step, not preinstalled), image `ubuntu-24.04` 20261012.334.1; `ssim punch_in_110 = 0.9999 on llvmpipe (LLVM 20.1.2, 256 bits) (Vulkan)`, `ssim crop_landscape_050 = 1.0000 on llvmpipe (LLVM 20.1.2, 256 bits) (Vulkan)`; `golden_frames` 4 passed in both steps. On D1: both 1.0000 on Microsoft Basic Render Driver (DX12). **Both at least 0.98: one folder of goldens, no per-adapter folders** (G 7.3's table); recorded in `experiments.md`, "Golden similarity across adapters". The adapter names agree with Prompt 91's probe (Entry 1; its image was 20261018.339.1, the Mesa package the same).

**How it was checked.**
- `OFFCUT_GOLDEN_UPDATE=1 cargo test -p offcut-render --test golden_frames -- --nocapture`: 2 written on "Microsoft Basic Render Driver (Dx12)". Then `cargo test -p offcut-render`: inline 6 (5 and `a_row_is_padded_to_256_bytes`), `golden_frames` 4 (2 goldens at 1.0000, 2 SSIM).
- `cargo clippy --workspace --all-targets -- -D warnings` and `cargo clippy -p offcut-render --target wasm32-unknown-unknown -- -D warnings`: clean.
- Prompt 92's two tree checks (`cargo tree -p offcut-render -e normal,no-proc-macro`, wasm32 and host, grep `^(ash|d3d12|windows|gpu-alloc[a-z-]*) v`): no output. No `Cargo.toml` of the renderer changed.
- `git ls-files fixtures/golden`: the two PNGs.
- The SAB gate (on `1623d88`'s tree and this one's): on `v4-build`, no `zz` file (the FXC drill file deleted), no test key; `pnpm fixtures` up to date; `pnpm check` exit 0 (`check-file-tree`: 307 files, 13 crates); **Rust 556 passed, 0 failed, 3 ignored** (Entry 10: 552; −1 `v3_reference`, +1 `a_row_is_padded_to_256_bytes`, +4 `golden_frames`); Vitest 92; `pnpm build` exit 0 (app shell 308.8 kB, unchanged); `pnpm e2e` 12 passed; the frozen check lists the seven recorded one-line exceptions of Entry 4 and `offcut-dsp/tests/program_identity.rs`; no private path tracked. `pnpm e2e:media` not run (the prompt asks for none; nothing on the page changed).
- CI on `1623d88`: `ci` passed (14m23s) with the readings above.

**Differs from the plan or the guide.**
- `Renderer::adapter_info()` is added beside §9.1's calls (native only), for the adapter assertion.
- `new_headless` asks DX12 only on Windows and Vulkan only elsewhere, so that the fallback adapter is the one §9.4 names on each machine.
- The DX12 compiler choice (above) is the decision known issue 3 asked this prompt to record.
- CI runs `golden_frames` a second time with `--nocapture` for its readings (a few seconds).

**Found, not fixed.** TS §39.3 item 14 (the 0.98 assumption across adapters) is now read for two video-only goldens; the copy-back (Prompt 127) writes the reading there, and Prompt 107 reads the fourteen that draw glyphs.

**Open.** Known issue 3 closed. Open items 1-5 unchanged.

**Follow-up (same prompt): the secret scan.** CI on `5cd1a24` failed at "Scan for secrets" (`gitleaks detect`, "leaks found: 1"; run 39828007019). No secret: gitleaks 8.18's default rule `generic-api-key` read `experiments.md` line 78, where the adapter name lavapipe was followed by a colon and a golden's name, as a key named "…api…" (the "api" in the name) with a value of entropy 3.61. The rule, reproduced in a scratch script over the commit's added lines, matches that line only and nothing in `1623d88`, which passed. The line is reworded ("crop_landscape_050 is 1.0000 on d1 WARP and 1.0000 on lavapipe; …", which the rule does not match), and since the scan reads the whole history, `.gitleaksignore` (new, at the root, listed in TS §5) holds that one finding's fingerprint, `5cd1a24…:docs/v4/experiments.md:generic-api-key:78`. Rewriting the pushed commit was not chosen: it needs a force-push of the pull request's branch.

---

## Entry 12 - 2026-10-23 - P102 (S8, table): the line-break table, before the algorithm

**What.** Prompt 102 (G 8.1, G 8.2's preparation; D-167, D-170, D-177, §8.10, G item 6): `crates/offcut-scene/tests/line_breaks.rs`, thirty sentences and how a person would break each into caption chunks and lines, in Clean (16 characters x 2 lines, 4 words, as spoken: D-170's values) and Bold (12 x 2, 3 words, upper case), written and committed **before any break search exists**: `captions::chunk` is V3's, and `git ls-files crates/offcut-scene/src | grep readability` prints nothing. Every expectation was decided by reading the sentence aloud; no code produced or changed one. Then ten rows, drawn at random, are shown to the founder (open item 7). The comparison waits for the break search: Prompt 103 records the founder's answers first, then writes `readability.rs`.

**The file.** `struct Row { id, sentence, pause_before, clean, bold }` and `const ROWS: [Row; 30]` (`#[rustfmt::skip]`, one row per three lines); the timing of D-167's table, 300 ms a word, the words one after another, 400 ms of silence before each word index `pause_before` names (400 ms is past `CHUNK_GAP_MS`, 350, so a pause is a hard break, as it is in V3); `Limits` (Clean: `AsSpoken`, 16, 2, 4; Bold: `Upper`, 12, 2, 3), applied over `styles::spec`, which shapes and places but holds V3's limits until Prompt 105; `transcript(row)` (`normalize_transcript`, so numbers become caption words as the pipeline makes them); `broken(row, limits)` (`captions::words` on the identity map, then `captions::chunk`, as text); `assert_fits` (lines, characters, upper case); and the one test, `table`, `#[ignore = "the break search arrives in P103"]`, which compares all sixty breaks and lists every row that differs.

**What a line holds.** Caption words, shown as the screen shows them (TS §17.2): a number with its unit is one word with its display, so `3 hours` (from "3" and "hours") counts as one word and `$2k`, `$20k` are whole (`format_quantity`'s money rule, read in `offcut-text/src/numbers.rs`); Bold upper-cases the displays (`$2K`). A line of one word may be longer than its style's characters: V2 scales it (rows 21, 22). A chunk that fits on one line is one line. The sentences avoid spelled numbers ("three" becomes "3"), counted nouns after a number ("3 views" would join), and a number at a sentence end (its display drops the full stop), so that every row tests a break and not a number's display.

**Coverage** (G 8.1's cases, each at least twice; a row often shows more than its heading):

| Case | Rows |
|---|---|
| An article never ends a line or a chunk | 1, 2, 3 (also 6, 11, 29) |
| A preposition never ends a line or a chunk | 4, 5, 6 (also 2, 17, 18) |
| A break after a comma | 7, 8 (also 23, 24, 25, 27, 28) |
| A clause starter begins a chunk | 9, 10 (also 23, 25, 26, 28, 29, 30) |
| Balanced lines, not a full line and a short one | 11, 12 (also 9, 15, 16, 25) |
| A one-word beat between pauses stays alone | 13 (a pause before "nothing."), 14 (pauses before and after "wow") |
| Otherwise no word is left alone | 15, 16 (also 12, 18) |
| A number stays whole | 17 (`$2k`, `$20k`), 18 (`3 hours`) |
| A sentence end is a hard break | 19, 20 (two sentences that would fit one chunk) |
| A word longer than a line | 21 (longer than both styles' lines), 22 (longer than Bold's only) |
| Everyday creator speech (D-177) | 23 recipe step, 24 fitness cue, 25 product review, 26 travel tip, 27 reaction |
| Several rules at once | 28, 29 (a pause), 30 |

**Decided while writing, before the commit.** Several first sentences were changed because a style could not break them without a binder at a line's end, which would make the row test nothing: "at the airport" became "at Heathrow" (Bold: "AT THE / AIRPORT." or "AT / THE AIRPORT."), "to a new server" became "to Postgres", "to my channel" became "to YouTube", "for the trip" became "for Japan", "It worked because we cached everything" became "… cached it" (no 2-line Clean split fits 16), "This is the part nobody tells you about" was dropped (Bold must break after "is" or "the"), "Salt the water" and "Book your train tickets" were replaced by rows 23 and 26. G 8.1's own examples were not used as written: they were drawn for 18 characters a line (D-139), and "the new onboarding" (18) does not fit Clean's 16. Row 7 keeps "HONESTLY," alone in Bold: the comma's natural beat, and every other split ends a chunk with "was".

**Checked by hand, and by a scratch script that decides nothing** (`check_table.py`, in the scratchpad, not committed): it reads the rows as written and reports any line over its characters (a one-word line excepted), more than two lines, more caption words in a chunk than the style allows, a lower-case Bold line, a two-line chunk that fits one line, and chunks that do not hold the sentence's caption words in order. On the table: 30 rows, 0 problems. Drill: a copy with a 20-character Clean line, a lower-case Bold line and a 5-word Clean chunk gave exactly those three reports.

**The ten rows for the founder (G 8.2).** `seq 1 30 | shuf -n 10` (GNU coreutils 8.32): **28 27 25 26 11 20 21 12 7 16**, shown in the reply of this prompt as a phone would show them, Clean then Bold, with "Would you break it this way?" for each, and nothing about how a break will be computed. Open item 7. Prompt 103 records the answers before any code; a "no" changes its row then, with the founder's reason.

**Frozen.** From this commit on a row changes only with a changelog line naming the row and the reason (§23). Its SHA is not known inside the commit: Prompt 103 records "line-break table frozen at <SHA>" (known issue 21).

**How it was checked.**
- `cargo test -p offcut-scene --test line_breaks`: 0 passed, 1 ignored ("the break search arrives in P103").
- `grep -c "id:" crates/offcut-scene/tests/line_breaks.rs`: 31, the 30 rows and the field `id: u8` of `struct Row` that the prompt's own interface names (`grep -c "Row { id:"`: 30).
- `git ls-files crates/offcut-scene/src | grep readability`: no output.
- `cargo clippy -p offcut-scene --all-targets -- -D warnings`: clean; `node scripts/check-file-tree.mjs`: ok (TS §5 has listed `line_breaks.rs` since Prompt 92).
- The SAB gate: on `v4-build`, no `zz` file, no test key; `pnpm fixtures` up to date; `pnpm check` exit 0 (the first run failed at `cargo fmt --check` on one `assert!` the helper's renaming had lengthened; `cargo fmt` and the check run again, exit 0; `check-file-tree`: 309 files, 13 crates); **Rust 556 passed, 0 failed, 4 ignored** (Entry 11: 556/0/3; the ignored count rises by 1, `line_breaks::table`, as the prompt says); Vitest 92; `pnpm build` exit 0 (app shell 308.8 kB); `pnpm e2e` 12 passed; the frozen check lists the seven recorded one-line exceptions of Entry 4 and `offcut-dsp/tests/program_identity.rs`; no private path tracked.

**Open.** Open item 7 (new, the founder's). Known issue 21 (new).

---

## Entry 13 - 2026-10-23 - P103 (S8): the founder's reading; readability, the break search, the reading floor

**Line-break table frozen at `01f406f`** (`01f406f86778b2a7b4aabedd2964b7a2f744a1f2`, Prompt 102's commit; known issue 21 closed). From it on a row of `crates/offcut-scene/tests/line_breaks.rs` changes only with a changelog line naming the row and the reason (§23).

**Line-break reading (D-167, G 8.2).** The founder read the ten rows drawn in Entry 12, each shown as a phone shows it (Clean, then Bold), on 2026-10-23. The answer, in the founder's words: "yes for all as far as it matches with the voice perfectly and eventually llm will tell the words that are emphasasized to make them bold and render".

| Row | Sentence | Answer |
|---|---|---|
| 28 | "We spent months building it, and nobody used it." | yes |
| 27 | "Oh wow, I love this!" | yes |
| 25 | "The battery lasts all day, which is great." | yes |
| 26 | "Pack light and arrive early." | yes |
| 11 | "We found my favorite coffee shop." | yes |
| 20 | "We tried it. It failed." | yes |
| 21 | "Internationalization matters." | yes |
| 12 | "The team loved it." | yes |
| 7 | "Honestly, it was worth every minute." | yes |
| 16 | "See you next week." | yes |

No "no", so **no row changes**: the table stays as committed in `01f406f`. The founder's two remarks, and where each is held: "as far as it matches with the voice perfectly" is the timing rule that a caption word is shown, and its active state changes, at its own spoken time, and that no word moves (INV-5; the reading floor of D-168 only extends a chunk into the silence after it, never into the next words); "eventually llm will tell the words that are emphasasized" is V6.1's model plan, whose emphasis picks are drawn through the emphasis layers of D-169 (Prompt 104 builds them for V4's spoken-stress signal).

**What.** Prompt 103 (G 8.2, G 8.3; D-167, D-168, §8.9, §8.10): the founder's reading in its own commit before any code (`7da641e`, `V4 P103 (S8, reading): the founder's ten rows`, the block above), then, in this commit, `readability.rs`, the break search, the reading floor, and `line_breaks.rs` un-ignored with the property checks over the labeled set. All 30 rows (60 breaks) pass as frozen at `01f406f`: no row changed. The costs were tuned against the table and the property checks, and every failure was fixed in the costs or the search (G 8.3's rule). No V3 inline caption expectation changed: `captions.rs`'s V3 tests pass as they were (`diff` of its test module against `7da641e`: only the two new tests added at its end), so there was nothing to list for approval.

**`crates/offcut-scene/src/readability.rs`** (new, [ONLY], `pub mod`):
- **The lists.** These are sorted, lower-case `const` slices, matched by binary search on the caption word's lower-cased text with the punctuation around it stripped.
  - `CLAUSE_STARTERS`: 39 words (after, although, and, as, because, …, which, while, who, why, yet).
  - `BINDERS`: 82 words: articles, possessives, prepositions, "to", the auxiliaries and modals, "not", "very", "really", and the cardinals one to ten. A number is one caption word with its display (TS §17.2), so a cardinal here is a spelled one.
  - `VERB_BINDERS`: 24 words, the auxiliaries, modals and "not", a subset of `BINDERS` (see below).
  - An inline test checks that the lists are sorted, lower case and free of repeats, and that `VERB_BINDERS` lies inside `BINDERS`.
- **D-167's costs.** These are named constants, all (assumption, E-2):
  - `COST_AFTER_PUNCT = 0`, `COST_CLAUSE = 1`, `COST_PLAIN = 3`, `COST_AFTER_BINDER = 12`, `COST_ONE_WORD = 8`, `ISOLATED_WORD_PAUSE_MS = 250`, the one-third imbalance rule, and D-168's `MIN_CHUNK_MS = 700`.
  - Added by this prompt, each explained below: `COST_OPEN_CLAUSE = 8`, `COST_BINDER_AT_SCREEN_END = 16`, `CHUNK_BREAK_WEIGHT = 2`, `COST_NEW_CHUNK = 1` and `MAX_IMBALANCE = 8`.
- **The functions:**
  - `break_cost(prev, next)`: a line break inside a chunk.
  - `chunk_break_cost(prev, next)`: a break that starts a new chunk.
  - `imbalance(a, b)`.
  - `ends_a_phrase(word)`: the word ends on `,` `;` `:` or `—`.
  - `binds_to_next(word)`: the one `pub` question, which the property checks ask.
  - The chunk's own cost, §8.9's `cost_of_chunk`: `Unit`, `Limits`, `width`, `fits` (a one-word line always fits; V2 scales it), `isolated` and `chunk_cost`, which gives one line when it fits, else the best two-line split with ties to the earlier split.
- **Inline tests:** each kind of break and screen break; `imbalance` at the threshold (12 against 8 costs 0, 12 against 7 costs 5) and at the cap; `chunk_cost` on a one-word chunk (alone, after a comma, set apart by silence), on a chunk too long for its style, and on a two-line split.

**The break search** (`captions::chunk`, §8.9):
- **Hard breaks.** Words are cut into runs at hard breaks: a sentence end, a pause of `CHUNK_GAP_MS` (350) or more, and `MAX_SEGMENT_WORDS = 40` words (TS §17.2's split; now `pub`, for the test's own reading of the runs).
- **The search.** `search` lays out each run: `best[i]` over `j` in `i - max_words .. i`, `best[j] + chunk_break_cost(before j) + chunk_cost(j..i)`. The run's first chunk has no break cost.
- **Output.** `chunk` keeps V3's line shaping and placement (`lines`) and V3's show times (`CHUNK_LEAD_MS`, `CHUNK_TAIL_MS`, the hand-over to the next chunk).
- **Determinism (INV-7).** Every cost is an integer, and every tie has one answer.

**The reading floor** (D-168): `captions::hold_for_reading(chunks, program_end)`, which `build_scene` calls after `chunk`.
- A chunk's `show.end` becomes `max(show.end, min(show.start + MIN_CHUNK_MS, cap))`. The cap is the next chunk's `show.start`, or for the last chunk the end of the program (`map.out_duration()`).
- The next chunk starts at most 80 ms before its first word, so the floor reaches no later than the next word's start and the silence after the chunk's last word. No word moves (INV-5), and a chunk is never shortened.
- The floor is a function of its own rather than part of `chunk`, so V3's inline test of `chunk`'s show times stays true unedited.
- Inline tests: a 400 ms "Yes." before a pause is held to 700 ms; a chunk held to the next chunk's start; the last chunk held only to the end of the program; and never shortened.

**`crates/offcut-scene/tests/line_breaks.rs`:**
- `table` lost its `#[ignore]`. Its rows are untouched (`ROWS` is byte-for-byte as in `01f406f`).
- New: the property checks of §8.10 over the 31 transcripts of `fixtures/labeled/`, for each style:
  - **Loading.** Prompt 97's loader pattern: written files timed as `precision_recall.rs` times them (without prosody), ASR files as recorded, each file's word edits applied.
  - **Check 1.** Every word appears exactly once and in order.
  - **Check 2.** No chunk is over its style's words, lines or characters.
  - **Check 3.** No line ends on a binder unless no layout allows it. This is checked more strictly than §8.10's "unless the chunk allows no other split": for every run, the lines the search ends on a binder (the run's own last word not counted) equal the fewest that any layout of the run within the limits can have. A separate DP in the test, over every layout, finds that fewest. Equality implies §8.10's per-chunk rule, since a chunk that could split without a binder would give a layout with one binder end fewer.
- **The reading** (`--nocapture`): `31 files; Clean: 678 chunks, 14 unavoidable binder ends; Bold: 833 chunks, 88 unavoidable binder ends`.
- **Limits.** The test sets D-170's limits (Clean 16 x 2, 4 words, as spoken; Bold 12 x 2, 3 words, upper case) over `styles::spec`, because `styles.rs` still holds V3's limits until Prompt 105 (the prompt's "record which").

**`captions.rs` under 400 lines.** The search took `captions.rs` to 516 lines above its tests. Two moves bring it to 373, with no new file outside TS §5:
- The chunk cost and its helpers (above) moved into `readability.rs`, beside the other costs (299 lines in all, 228 above its tests). The search, `chunk` and the floor stay in `captions.rs`.
- `draw` moved, unchanged, into `captions_draw.rs`, the file TS §5 already lists for drawing a chunk (70 lines). This is the move Prompt 104 names ("the chunk drawing moved out of `captions.rs`"), made one prompt early; Prompt 104 adds the emphasis order to it.
  - `captions.rs` keeps `pub use crate::captions_draw::draw`, so `Scene::frame_at` and V3's inline test of `draw` are unedited.
  - That test reads `DrawCmd`, so `captions.rs` imports it under `#[cfg_attr(not(test), allow(unused_imports))]`. A `#[cfg(test)]` import would have hidden the file from `check-file-tree`, which counts up to the first `#[cfg(test)]` line.
- `lib.rs`: `pub mod readability;`, `mod captions_draw;`, and the floor's call.

**Differs from D-167 and §8.9 as written.** Below, `[ ]` is a screen (a chunk) and `/` a line break inside one. These are costs and rules the table or the property checks needed. The drill below shows that each one is needed.

| Change | Why |
|---|---|
| **(a) Ties between layouts go to the layout whose last chunk starts later**, so the earlier screens are the fuller, as a reader fills them. §8.9 says "ties to the earlier break". A chunk's own line split still ties to the earlier split. | Without it, rows 2, 5 and 6 differ. |
| **(b) A word ending on `,` `;` `:` `—` may be a chunk alone without the one-word cost:** the comma is the beat. | Without it, row 7 differs ("HONESTLY," alone in Bold). |
| **(c) A screen break is weighed twice a line break, plus 1** (`CHUNK_BREAK_WEIGHT`, `COST_NEW_CHUNK`). The reader loses the context when the screen changes, not when a line wraps; of two layouts that read alike, the one with fewer screens wins. D-167 has one cost for both. | Weight 1: rows 9, 11, 25, 26 and 28 differ. No extra 1: rows 7 and 25 differ. |
| **(d) A clause starter left at the end of a screen costs `COST_OPEN_CLAUSE = 8`** ("it worked because / we cached it": the clause waits for the next screen). Inside a chunk "which / it" stays a plain break. The cost is below a binder's, so a screen ends on "then" before it ends on "from". | At 12, a02 got `[and then from] [that feedback]`, a screen ending on "from", where `[and then] [from that / feedback]` avoids it. Without the cost, rows 25 and 26 differ. |
| **(e) A break before an article, possessive or preposition costs `COST_CLAUSE`** (1, where a phrase starts), not plain (3). D-167 gives 1 only before a clause starter. | Without it, row 11 differs (Clean `[We found] [my favorite / coffee shop.]`). |
| **(f) The discount in (e) leaves out `VERB_BINDERS`:** a break before "was" or "is" separates a subject from its verb. | With "every binder", row 7 differs. |
| **(g) The imbalance is capped at `MAX_IMBALANCE = 8`**, below a binder's break less a plain one, so balance never buys a line that ends on a binder. D-167 has no cap. | Uncapped, w01's `[If your / customers]` (12 + 0) beat `[If / your customers]` (1 + 12): property check 3 fails. |
| **(h) A binder left at the end of a screen costs `COST_BINDER_AT_SCREEN_END = 16`**, not 12, so that a screen ends on a binder only where no layout of the run avoids it. | At 12, four Bold runs ended a screen on "A", "THE" or "NOT" where a layout with a one-word chunk avoids it: w16, w20, w23 and w28 (w16: `[IT IS A] [DATE PARSER]`). 16 was the lowest value tried (16, 20 and 24 all pass). |

The other differences from the plan:
- **The floor is a separate function.** The floor is `hold_for_reading`, applied by `build_scene`, not code inside `chunk`. Its cap for the last chunk is the end of the program; the prompt's "the last word's end plus `CHUNK_TAIL_MS` plus the silence after it" is that same time.
- **Two items made `pub` for the test.** `binds_to_next` and `MAX_SEGMENT_WORDS` are `pub`, so the property checks read the same lists and runs.
- **The checks are one test.** The three property checks are three groups of assertions in one test, `every_labeled_transcript_breaks_within_its_limits_and_on_no_avoidable_binder`.
- **The chunk cost lives in `readability.rs`.** The acceptance line says "the search and the floor are in `captions.rs`", which holds. The chunk's cost is in `readability.rs` with the other costs, for the line limit.

**Drill (not committed): each change reverted alone** (`drill103.py` in the scratchpad), with `cargo test -p offcut-scene --test line_breaks`:

| Reverted | Table | Property checks |
|---|---|---|
| (a) | 3 of 60 differ (rows 2, 5, 6) | pass |
| (b) | 1 differs (row 7) | pass |
| (c), weight 1 | 5 differ (rows 9, 11, 25, 26, 28) | pass |
| (c), no extra 1 | 2 differ (rows 7, 25) | pass |
| (d) | 2 differ (rows 25, 26) | pass |
| (e) | 1 differs (row 11) | pass |
| (f) | 1 differs (row 7) | pass |
| (g) | passes | w01: 2 binder ends, 1 must |
| (h) | passes | w16: 2 binder ends, 1 must |

The files were restored, and the table and property checks pass again.

**How it was checked.**
- `cargo test -p offcut-scene --test line_breaks -- --nocapture`: `table` ok (60 of 60), and the property test ok with the reading above.
- `cargo test -p offcut-scene`: 25 inline tests (the crate's 19 of `7da641e`, unchanged, and 6 new: 3 lists-and-costs, the chunk cost, 2 floor), 2 in `line_breaks`, 1 in `timemap_identity`.
- `git log --diff-filter=A --format=%h -- crates/offcut-scene/src/readability.rs`: this commit, after `01f406f` (the table) and `7da641e` (the reading).
- `cargo clippy -p offcut-scene --all-targets -- -D warnings`: clean. `node scripts/check-file-tree.mjs`: ok (311 files; `captions.rs` 373 lines counted).
- **The SAB gate:**
  - On `v4-build`, no `zz` file, no test key; `pnpm fixtures` up to date; `pnpm check` exit 0.
  - **Rust: 564 passed, 0 failed, 3 ignored.** Entry 12 read 556/0/4. The table's `#[ignore]` is gone (ignored back to 3), and 8 tests were added: the table, the property test, 3 `readability` tests, the chunk-cost test and the 2 floor tests.
  - Vitest 92; `pnpm build` exit 0 (app shell 308.8 kB); `pnpm e2e` 12 passed.
  - The frozen check lists the seven recorded one-line exceptions of Entry 4 and `offcut-dsp/tests/program_identity.rs`. No private path is tracked.
  - `pnpm e2e:media` was not run: the prompt asks for none. CI's `e2e-media` job runs it on this commit.

**Found, not fixed.**
- Prompt 104's "the chunk drawing moved out of `captions.rs`" is already done (above); Prompt 104 adds the emphasis to `captions_draw.rs`.
- Prompt 104's binder-cost drill now has two binder costs to drill: `COST_AFTER_BINDER` for lines, and `COST_BINDER_AT_SCREEN_END` for screens.

**Open.** Open items 1-5 unchanged; open item 7 and known issue 21 closed (above). No new item or issue.

---

## Entry 14 - 2026-10-23 - P104 (S8): emphasis, placement, hit test, shaping cache, parity

**What.** Prompt 104 (G 8.4, G 8.5, M-8; D-169 to D-174, §8.6, §8.9, §8.10, §11.1): the caption block and the event zone from `EditState.caption_position`, the emphasis order in `captions_draw.rs`, `Scene::hit`, the per-thread shaping cache, the preview-equals-export parity test, and the binding's `hit`. With Prompt 103 this completes caption intelligence in the scene (S8).

**1. Placement** (`placement.rs`, new, [ONLY], D-171):
- **Constants.** `CAPTION_TOP = SAFE.top + 140` (390) and `CAPTION_BOTTOM = SAFE.bottom - 140` (1,360); `SNAP_TOP = 0.15`, `SNAP_MIDDLE = 0.55`, `SNAP_BOTTOM = 1.0`; `CAPTION_BLOCK_HEIGHT = 280`, `EVENT_ZONE_HEIGHT = 320`, `EVENT_ZONE_GAP = 40`, all (assumption, E-2).
- **`caption_block(p, style)`.** The block is as wide as the safe area and 280 lp tall, centred at `CAPTION_TOP + p x (CAPTION_BOTTOM - CAPTION_TOP)`. Every V4 style has the same block: `style` is the seam for one that will not, and an inline test asserts all three styles agree.
- **`event_zone(block)`.** When the block's centre is at or below 960, the zone ends 40 lp above the block; above 960, it starts 40 lp below it. At 1.0 this gives exactly V3's zones (block 1,220-1,500, event zone 860-1,180); at 0.0, block 250-530 and event zone 570-890.
- **No profile.** Nothing in `placement.rs` reads `profile.watermark`.
- **`safe_area::zone(id, placement)`.** It now takes the clip's placement and reads the same two functions (through `block_at`).
- **`build_scene`.** It computes the block and the event zone once:
  - `captions::chunk` takes the `block` (the plan's signature, §8.1), and `lines` centres in it.
  - `events::layout` takes `(span, zone)`, when and where, as one argument, because it already had clippy's maximum of 7.
  - `number_reveal::layout` takes the zone, and its `Track` keeps the zone's centre (`centre_x`), so `draw` no longer reads a fixed zone.
- **`Scene::caption_block()`** gives the block (for `hit` and the tests).

**2. Emphasis** (`captions_draw.rs`, D-169):
- **The order.** `fill_role` gives each word's role, the first that applies of:
  1. The spoken word, through the style's active treatment.
  2. A KeywordPop word (`pop`).
  3. A number, when `highlight_numbers` is on (`accent`).
  4. `fill`.
- **Cursor styles.** A style whose active word is not a colour (Tech's `Cursor`) leaves the spoken word to the next layers.
- **The emphasis.** `Emphasis { pops: Vec<(WordIdx, OutMs)>, numbers }` is built by `build_scene`: each KeywordPop item gives its word and its program start (the hand-over of §8.1; `events/keyword_pop.rs` is Prompt 106's). A popped word is in `pop` from that start while its chunk is shown.
- **Drawing.** `draw_with(chunk, t, style, theme, emphasis, out)` draws the scene. `draw` (no emphasis besides the spoken word, V3's look) stays for V3's inline test.
- **Which words are numbers.** `CaptionWord` gains `number: bool`, set by `captions::words` (§8.9):
  - A word made from a number span is a number.
  - So is a word whose text, less a closing mark, `parse_quantity` reads whole and `format_quantity` writes back unchanged: "$500" and "3" are numbers; "2024" (shown "2,024") and "five" (shown "5") are not.
  - The scene's crate doc now says it parses no item's number (D-121) and reads a caption word only to know whether it shows one. `Cargo.toml`'s comment on `offcut-text` names the two functions.
- **Tests:**
  - `captions_draw.rs` inline: every combination of the layers, and the cursor case.
  - `lib.rs` inline: in a scene, "$5" is in `accent` with numbers highlighted and in `text` without, and a KeywordPop word turns `pop` from its start and not before.
  - `captions.rs` inline: number spans, an edited "$500", an edited "2024", and "five,".

**3. Hit testing** (`hit.rs`, new, D-174):
- **`Hit`.** `Hit::{Word(WordIdx), Captions, Video}`.
- **`word_boxes(chunk)`.** Each word's glyph box from the layout already built, at its line's scale; nothing is shaped.
- **`Scene::hit(t, x, y)`.** The rules, in order:
  1. A point off the canvas, or not a number, gives `None`.
  2. While no chunk is shown, `Video`.
  3. Otherwise, a word within `HIT_PADDING` (8 lp) of its box. Where two padded boxes meet, the nearer box wins, and of two equally near, the first drawn (the lines of a chunk touch, so padded boxes overlap).
  4. Otherwise, `Captions` inside the block, `Video` outside it.
- **`tests/hit.rs`:**
  - Every word's centre over 85 instants hits that word (its index from `captions::words`).
  - The block's four corners hit `Captions`, and points outside it hit `Video`.
  - A word is hit 7.5 lp to its left and not 8.5 lp.
  - Before and after the captions, the block's middle is `Video`.
  - Points off the canvas, `NaN` and infinity give `None`, and a sweep of 8,591 points at three times never panics.

**4. The shaping cache** (`layout.rs`, D-173):
- **The structure.** A `thread_local!` `Cache` written in the file, following D-173's design:
  - A `BTreeMap` keyed by `(String, FontKey, u32 size bits)` holds the runs and each entry's last use.
  - A second `BTreeMap` from use to key evicts the least recently used past `SHAPE_CACHE_ENTRIES = 2,048`.
- **Faces.** The cache remembers the `FontSet` it was filled with, compared by `FontSet::same_faces`: the same files by pointer, not byte by byte, and the same families, weights and coordinates. For another set it empties itself, so a key never answers for faces it was not shaped with.
- **Errors.** A failed shaping is not kept.
- **The switch.** `layout::set_shape_cache(on)` is `#[doc(hidden)]`; it turns the cache on or off and empties it. The program never calls it.
- **`FontSet::empty()`.** A `const fn`, for the `const` thread-local.
- **Tests:**
  - Inline: the least recently used is dropped, the size is part of the key, another set empties the cache, and nothing is kept while off.
  - `tests/scene_cache.rs`: six scenes, each built cold, warm and with the cache off. The three `Scene`s and every frame's display list are equal.

**Build times** (`cargo test -p offcut-scene --release --test scene_cache -- --nocapture`, D1, d1):

| Scene | Frames | Cold | Warm | Without the cache |
|---|---|---|---|---|
| a01-scriptA | 2,013 | 17.8 ms | 1.4 ms | 8.2 ms |
| a02-v3testclip | 2,386 | 4.2 ms | 2.7 ms | 5.9 ms |
| w01-saas-pricing | 876 | 2.9 ms | 0.7 ms | 2.1 ms |
| w16-open-source | 740 | 3.4 ms | 0.6 ms | 2.6 ms |
| w20-battery-life | 976 | 2.9 ms | 1.1 ms | 2.8 ms |
| w28-mistakes | 906 | 1.7 ms | 0.6 ms | 1.4 ms |
| **Six scenes** | | **32.8 ms** | **7.2 ms** | |

- **The first build.** a01's cold build is the thread's first: it includes building the shaper and registering the fonts.
- **Debug build.** The same run gave 434.0 ms cold and 22.2 ms warm for the six.
- **The edit-to-preview budget.** D-173's 150 ms (TS §13.5) is measured end to end on the 60 s clip in Phase 13. A warm rebuild here is a few milliseconds.

**5. Parity** (`tests/parity.rs`, D-172):
- **The comparison.** For each of the six scenes, every frame at 540 and 720 wide is compared with 1080, command by command, in logical pixels.
- **Lengths.** Glyph positions on the canvas, sizes, stroke widths, rectangles, radii, clip rectangles, path points and the video's `dst` are within 0.01 lp.
- **Equal values.** Fonts, glyph ids, texts, colours, opacities and the video's source rectangle are equal.
- **Result.** It passes. The image comparison of D-172 (`golden_frames.rs`'s parity case) is Prompt 107's.

**The six scenes** (`tests/common/scenes.rs`, new, included by `#[path]` as `offcut-render` includes `common/ssim.rs`; added to TS §5's tree under the scene's `tests/`):
- **Sources.** a01, a02, w01, w16, w20 and w28 of the labeled set, detected and validated as the pipeline does. Written files are timed and stressed as `precision_recall.rs` does them; a02 takes its recorded words and prosody; each file's own edits are applied.
- **Choices across the six:**

| Scene | Style | Placement | Other choice |
|---|---|---|---|
| a01 | Clean | 1.0 | |
| a02 | Bold | 0.15 | |
| w01 | Tech | 0.55 | |
| w16 | Clean | 0.0 | numbers not highlighted |
| w20 | Bold | 1.0 | word 3 edited to "$500" |
| w28 | Clean | 0.55 | its first visual turned off |

- **Output.** The tests print only file names and times; a02 is the founder's clip, and its words are not printed.

**6. The binding** (`scene_api.rs`, §11.1):
- **`RenderSession.hit(t_out_ms, x, y)`.** `x` and `y` are fractions of the canvas, mapped to logical pixels by `CANVAS_WIDTH` and `CANVAS_HEIGHT`.
- **The answer.** `{ kind: "word", word }`, `{ kind: "captions" }` or `{ kind: "video" }`, through a tagged `HitValue`; `null` off the canvas or before a scene is set.
- **Wiring.** Prompt 113 wires it to the worker's `hitTest`.
- **Inline test:** the fractions, off-canvas answers, and the three JSON shapes.

**`tests/placement.rs`** (§8.10):
- **Inside and apart.** At 0.0, 0.15, 0.55 and 1.0, the block and the event zone are inside the safe area and apart, and `safe_area::zone` gives the same two boxes.
- **The watermark's band.** The band is taken from the safe area until `watermark.rs` exists (D-143: 36 lp, baseline 6 lp above the safe area's bottom; band 1,458-1,500). It lies below `CAPTION_BOTTOM`, the lowest a block's centre goes, and is clear of the event zone everywhere and of the block at 0.0, 0.15 and 0.55.
- **Drawn where placed.** Captions and numbers are drawn where the placement puts them: in the a01 scene at each placement, every caption run's baseline (over 1,000) is in the block and every number run's (over 50) is in the event zone.
- **The watermark flag.** It changes no placement: at every placement, the scene with the watermark flag has the same block, and each frame without it is the start of the same frame with it. The watermark, once drawn (Prompt 106), is the last command.
- **At 1.0.** Exactly 860-1,180.

**The V3 expectations at placement 1.0** (the prompt's requirement). Every V3 inline test passes with its expectations unchanged. Three V3 tests changed by their calls only, because the functions they call now take the placement:
- `safe_area`'s zone test: `zone(id, CaptionPlacement::bottom())`.
- `captions.rs`'s `chunks_of` and its zone check: the default block, and the test module's imports of `zone`.
- `number_reveal.rs`'s `track_of` and its room check: the default event zone.

`line_breaks.rs` passes the default block to `chunk` in its two calls, with its `ROWS` byte for byte as in `01f406f`.

**Changed beyond the prompt's files.** The emphasis of D-169 is on by default (`highlight_numbers` defaults to `true`, Prompt 95), and in Clean's V3 colours numbers and pops share the spoken word's `accent`. So `timemap_identity.rs`, which asserts that the one accent run at each word's time is that word, builds its scene with `highlight_numbers: false` and the KeywordPop item turned off (a comment says why). It tests timing, which this does not change. Its other assertions are unchanged.

**Guards (G 8.5),** each a temporary edit, the test run, then the files restored and checked identical by SHA-256 (`drill104.py`, in the scratchpad):

| Temporary edit | Result |
|---|---|
| `COST_AFTER_BINDER` 12 → 0 | `line_breaks.rs` fails: `table` 4 of 60 differ, rows 3 (an article row), 5 and 6 (preposition rows) and 25; the property check, a01: "9 lines end on a binder, 0 must" |
| Both binder costs 0 (with `COST_BINDER_AT_SCREEN_END`, as Entry 13 asked) | 13 of 60 differ (rows 2, 3, 5, 6, 7, 11, 17, 25, 30); a01: 17 against 0 |
| `placement.rs` gains `caption_block_for(p, style, profile)`, which raises the block by 48 lp when `profile.watermark` is set, and `build_scene` calls it | `placement.rs` fails `the_watermark_flag_changes_no_placement` at placement 0.0 |
| The cache key `(text, key, 0)`, without the size | `scene_cache.rs` fails on a01: a text shaped at one size answers for another |

A first form of the watermark drill (a third parameter on `caption_block`) did not compile, because the other callers use the two-argument form. The form above is the mistake as it would be written.

**How it was checked.**
- `cargo test -p offcut-scene`: all pass. Inline 32 (Entry 13: 25; +2 `placement`, +1 `captions_draw`, +1 `hit`, +1 `layout`, +1 `captions`, +1 `lib`), `line_breaks` 2, `placement` 3, `hit` 3, `scene_cache` 1, `parity` 1, `timemap_identity` 1.
- `cargo test -p offcut-scene -p offcut-plan`: all pass (M-8).
- `git log --oneline -3 -- crates/offcut-scene/tests/line_breaks.rs`: `3659c8f` (P103), `01f406f` (the table). `git log --diff-filter=A --format=%h -- crates/offcut-scene/src/readability.rs`: `3659c8f`, newer than the table's `01f406f` (M-8).
- `cargo clippy --workspace --all-targets -- -D warnings` and `cargo clippy -p offcut-wasm-render --target wasm32-unknown-unknown -- -D warnings`: clean.
  - `scene_cache.rs`'s timing helper carries `#[allow(clippy::disallowed_methods)]` for `Instant::now`, the one clock of the scene's tests, as `offcut-detect/tests/redetect.rs` does. The crate itself reads no clock.
- `node scripts/check-file-tree.mjs`: ok, 318 files. `captions.rs` is 400 lines above its tests, exactly the limit (known issue 22).
- **The SAB gate:**
  - On `v4-build`, no `zz` file, no test key; `pnpm build:wasm` exit 0; `pnpm fixtures` up to date; `pnpm check` exit 0.
  - **Rust: 580 passed, 0 failed, 3 ignored.** Entry 13 read 564/0/3; the 16 added are the 7 inline scene tests above, 8 in the four new test files, and 1 in `scene_api.rs`.
  - Vitest 92; `pnpm build` exit 0 (app shell 308.8 kB, unchanged); `pnpm e2e` 12 passed.
  - The frozen check lists the seven recorded one-line exceptions of Entry 4 and `offcut-dsp/tests/program_identity.rs`. No private path is tracked.
  - `pnpm e2e:media` was not run: the prompt asks for none. CI's `e2e-media` job runs it on this commit.

**Differs from the plan or the prompt.**
- **The watermark box at 1.0.** §8.10's "the watermark box disjoint from both" cannot hold for the block at 1.0: the block reaches the safe area's bottom, as V3's caption zone does, and the watermark sits there. The test asserts the band against the event zone everywhere, against the block at the three higher places, and below `CAPTION_BOTTOM`. D-143's test of the caption glyph boxes against the watermark's (`layout_safe_area.rs`, Prompt 106) is what guards 1.0.
- **`events::layout`'s signature.** It takes `(span, zone)` as one argument, rather than §8.1's list plus a zone.
- **`draw` stays beside `draw_with`.** `draw` is kept for V3's inline test of the active colour; `frame_at` calls `draw_with`.
- **Hit points between two words.** Where padded boxes meet, the nearer box wins (D-174 does not say).
- **A KeywordPop has no end of its own.** A pop lasts from its start while its chunk is shown, because §8.7 gives `keyword_pop.rs` "the word and its program start" only. Its scale (1.0 → 1.25 → 1.10) is Prompt 105's.
- **The cache compares faces.** It also compares the `FontSet` it was filled with (by pointer), beside D-173's key, so that another set never gets runs it did not shape.
- **New test file.** `tests/common/scenes.rs` and its TS §5 line.

**Found, not fixed.**
- **The summary.** `Scene::summary` still counts no emphasized caption (`captions_emphasized: 0`) although pops are now drawn; `summary.rs` (D-144) is Prompt 106's.
- **A pop on a number.** It is matched on a caption word's first transcript word only, so a KeywordPop on the second word of a spoken number does not colour it. This is rare: the detector's KeywordPop is a content word outside every other event's anchors (TS §17.5), though a spoken number with no NumberReveal ("three thousand") is not excluded. Prompt 106's `keyword_pop.rs` can carry the range.
- **The manual check.** M-8's "Captions read naturally" (`pnpm dev`, the sample clip: no "the" or "to" at a line end, two-line chunks of similar length) is a look on the page, for the founder; the property check of Entry 13 tests the same rule over the 31 labeled transcripts.

**Open.** Open items 1-5 unchanged. Known issue 22 (new): `captions.rs` is at 400 lines; Prompt 105 must move something out before adding to it.

---

## Entry 15 - 2026-10-23 - CI: the downloaded Ubuntu packages are cached; apt retries three times (outside a prompt)

**Why.** The founder asked why CI had grown slow. Runs took 28 to 30 minutes up to P101's fix (`a6dc4bc`, 27.7 min) and 40 to 49 minutes on P103 (`3659c8f`, 48.6 min) and P104 (`0fc25f9`, 39.8 min). The `ci` job's own steps were unchanged: `cargo test` took 2.4 to 4.6 minutes in every run. Two steps that download Ubuntu packages from the runner's mirror (`azure.archive.ubuntu.com`) made the difference:

| Run | `ci` job | Install the software Vulkan driver | Install Chrome for Playwright |
|---|---|---|---|
| `a6dc4bc` (P101) | 15.2 min | under 20 s | 0.4 min |
| `01f406f` (P102) | 19.3 min | under 20 s | 0.6 min |
| `3659c8f` (P103) | 33.9 min | 9.5 min | 10.1 min |
| `0fc25f9` (P104) | 26.1 min | 6.9 min | 7.2 min |

- **The Vulkan step's log on `0fc25f9`.** `apt-get update` fetched 12.5 MB in 1 s (10.9 MB/s). The one package, `mesa-vulkan-drivers` 25.2.8-0ubuntu0.24.04.4 (17.5 MB), started at 22:51:37 and was given up (`Ign`) at 22:58:10; apt asked again, and the second try took 9 s. The step reported "Fetched 398 kB in 6min 45s (983 B/s)".
- **The Chrome step.** `playwright install --with-deps chrome` installs fonts and libraries with `apt-get` from the same mirror: "Fetched 22.0 MB in 6min 53s (53.3 kB/s)".
- **In the fast runs** the same downloads took seconds: it is the mirror's speed on the day, not a change of ours.
- **The media job.** `e2e-media` (Windows, no apt) waits for the `ci` job's build, so all of the delay adds to the run.

**What changed** (`.github/workflows/ci.yml` only; no build, test or deploy command changes):
- **"Make apt retry"** is a new step after "Free disk space". It writes `Acquire::Retries "3"` to `/etc/apt/apt.conf.d/zz-offcut-retries`, read after the image's own settings, and prints the values apt then uses. Every `apt-get` of the job, Playwright's included, asks again for a dropped file up to three times, where the image allows one. The `deploy-web` job (main only) gets the same step before its Chrome install.
- **"Restore the downloaded Ubuntu packages"** is a new `actions/cache` step (the pinned v6, as the other caches use) of `~/apt-archives`. Its key is `apt-<os>-<hash of ci.yml and pnpm-lock.yaml>`, the two files that decide which packages are installed, and it falls back to the last `apt-<os>-` cache.
  - The Vulkan step first copies the cached `.deb` files into `/var/cache/apt/archives/` (and says how many).
  - apt takes a file from its archive only when it is the version the index asks for, and downloads any other.
- **"Keep the downloaded Ubuntu packages"** comes after the Chrome install, the job's last `apt-get`. It copies `/var/cache/apt/archives/*.deb` to `~/apt-archives`, which the cache saves at the end of a job whose key was not found. An image that keeps no downloaded package leaves the cache empty, not the job failed.

**What it does not do.**
- **No timeout of ours.** The image already sets a 15 s timeout, which ends a silent connection, not a slow one that still sends.
- **Where the gain comes from.** The cache removes the downloads when the versions are unchanged. A new Mesa or font version is downloaded once from the mirror, and the cache keeps the old file until its key changes (an edit of `ci.yml` or of the lockfile).
- **The media job still waits for `ci`.** Running it beside `ci` is a restructure of the jobs, left to Prompt 125, which owns the rest of V4's CI changes (§19.3).
- **The time limit.** The `ci` job's `timeout-minutes: 45` is unchanged. P103's run used 33.9 of it.

**How it was checked.** On CI only: the steps run on the Linux runner, not on D1.

- **`75c02c3`, the first form (run 39872773582, green, 49.7 min).** It had a step "Make apt fail fast and retry", which wrote `Acquire::http::Timeout "20"`, `Acquire::https::Timeout "20"` and `Acquire::Retries "3"` to `80-fail-fast`. `apt-config dump` printed those three and then `Timeout "15"`, `Timeout "15"`, `Retries "1"`: the image's own file is read after `80-…`, so its values won.
  - **Its timings.** The downloads did not stall; they trickled. Mesa came at 44.7 kB/s (17.5 MB in 6 min 31 s; step 6.9 min) and Playwright's packages at 50.8 kB/s (22.0 MB in 7 min 13 s; step 7.7 min). No timeout would have cut them, so the step was reduced to the retries, in a file named to be read last.
  - **The cache.** "Cache not found for input keys: apt-Linux-6e6f…"; "Keep the downloaded Ubuntu packages" printed "12 packages, 38M"; "Cache saved with key: apt-Linux-6e6f…".
  - **Overall.** `ci` 32.4 min, `e2e-media` 17.2 min.
- **This commit.** It changes `ci.yml`, so its key is new, and the cache falls back to the `apt-Linux-` cache saved above. Its run is the first to install from the cache; its reading is recorded in the next entry.

**Open.** Unchanged.

---

## Entry 16 - 2026-10-23 - P105 (S9): Inter 800, the theme table, Clean with the sliding pill, Bold, Mono

**What.** Prompt 105 (G 9.1 rows 1-6; D-136 to D-139, D-170, §8.4, §8.5, §8.9): the premium default. Clean is Inter 800 with a drop shadow, and its spoken word sits on a yellow pill that slides from word to word. Bold pops each word in upper case. Mono (`StyleId::Tech`) has its dark panel, green spoken word and block cursor. Every colour is a theme role and every motion value is data. Until now the three styles drew V3's Clean.

**1. Fonts** (`fonts.rs`, D-170):
- **Key 4.** Inter 800 is appended after Noto Emoji (D-132's rule: V2's keys keep their faces). Its coordinates are computed by `coords_of` like the others' (the same file as keys 0 and 1, one registration).
- **Tests.** The two inline tests now cover five faces: key 4 is "Inter Variable" at 800 and shares the file of keys 0 and 1, key 5 is empty, and 800's coordinates differ from 700's and 900's and lie between them.

**2. The theme** (`theme.rs`, §8.4, D-136):
- **The table.** `Theme::default_for` gives each style its row:

| | Clean | Bold | Tech ("Mono") |
|---|---|---|---|
| `text` | `#FFFFFF` | `#FFFFFF` | `#E8F0EA` |
| `stroke` | `#101014` | `#101014` | `#101014` |
| `accent` | `#FFD60A` | `#FFD60A` | `#3DDC84` |
| `accent_alt` | `#FFD60A` | `#FF5E3A` | `#3DDC84` |
| `panel` | `#101014` at 85% (alpha 217) | `#101014` at 85% | `#0B0F0C` at 85% |
| `on_accent` (new) | `#111114` | `#111114` | `#111114` |
| `shadow` (new) | `#000000` | `#000000` | `#000000` |
| display / body / mono | Inter 800 (key 4) / Inter 700 / JetBrains Mono 700 | Inter 900 / Inter 700 / JetBrains Mono 700 | JetBrains Mono 700 / Inter 700 / JetBrains Mono 700 |

- **V3's colours are gone.** `v3()` (the identity proof's theme, D-117) is removed. The proof was retired in Prompt 101.
- **Two roles beyond §8.4's struct.** The table's pill row ("word in `#111114`") and shadow row ("black") need colours that §8.4's `Theme` has no field for, and the prompt asks for the pill's word "via the theme". So `Role` gains `OnAccent` and `Shadow`, and `Theme` gains `on_accent` and `shadow`. `backdrop` (the canvas behind no video) holds the same black, but a brand kit (V6.7) may change it without changing the shadow.

**3. Motion** (`motion.rs`, D-137, D-170):
- **New types.** `ActiveTreatment::Pill` is appended, and so is `ShadowSpec { offset, role, opacity }`.
- **`enter_at`.** It is unchanged. A new inline test reads it on the styles' own values:
  - Clean's fade (120 ms, ease-out cubic): opacity 0 at 0 ms, 0.875 at 60, 1 at 120.
  - Bold's pop (140 ms, back-out): 1.0 at 0 ms, past 1.12 at 49, exactly 1.12 at 70, 1.0 at 140.
  - Tech: at rest at once.

**4. The styles** (`styles.rs`, D-139, D-170):

| | Clean | Bold | Tech |
|---|---|---|---|
| font, size | `Display` (Inter 800), 72 lp | `Display` (Inter 900), 84 lp | `Mono`, 56 lp |
| case; chars x lines; words | as spoken; 16 x 2; 4 | upper; 12 x 2; 3 | as spoken; 22 x 2; 5 |
| stroke | 5 lp | 10 lp | none |
| shadow | 0 x 4 lp, `Shadow`, 45% | the same | none |
| panel | none | none | radius 24, padding 32, `Panel` |
| motion | `Fade`, `Pill`, ease-out cubic, 120 ms | `Pop`, `Highlight`, back-out, 140 ms | `None`, `Cursor`, linear, 80 ms |
| fill / active / pop | `Text` / `Accent` / `AccentAlt` | the same | the same |

- **The constants.** `PILL_RADIUS = 14`, `PILL_PADDING = (12, 4)` and `CURSOR_WIDTH_EM = 0.6` are in `styles.rs` (Appendix D), and so is the shared `DROP_SHADOW`.
- **KeywordPop's colour.** It moves from Clean's `Accent` to `AccentAlt` in every style (D-139). The value is the same in Clean.
- **`StyleSpec::decoration_width()`.** The width beside a line, both sides together, that what is drawn around its glyphs needs: half the stroke plus the shadow's sideways move, the pill's padding, the panel's padding plus the cursor's cell. Clean gives 24, Bold 10, Tech 131.2.
- **Tests.** Every row's values. The existing `no_style_moves_glyph_by_glyph` is the refusal of `MotionUnit::Glyph`.

**5. Drawing** (`captions_draw.rs`, §8.9). A chunk is drawn in this order, all inside the fade's layer while Clean fades in:
1. Tech's panel.
2. The shadow.
3. Clean's pill.
4. The words.
5. Tech's cursor.

- **The pill** (D-170): a `FillRect` in the style's `active` (`accent`), drawn only while a word is spoken. Between two words, in a pause, before the first word and after the last, there is none.
  - **The slide.** From the spoken word's start, over `PILL_SLIDE_MS = 90`, eased ease-out cubic, its box goes from the previous word's box to the spoken word's. The first word of a chunk grows on its own box. Its scale goes from `PILL_GROW_FROM = 0.9` to 1, and its 14 lp radius scales with it. It is padded 12 x 4 lp.
  - **The word on it.** At rest (from +90 ms) it is in `on_accent`, with no outline and no shadow.
  - **While the pill slides in** (the first 90 ms), the spoken word is drawn twice:
    - as it looks off the pill (outlined, its shadow cast), under the pill;
    - in ink, over the pill, inside a `PushLayer { opacity: 1, clip: pill }`.
    It turns to ink exactly where the pill has reached it (see "The look").
- **The shadow:** every run but the pill's word, drawn again moved by the offset, in `shadow` with the same stroke, all inside one `PushLayer` at the shadow's opacity, before any word is drawn.
- **Bold's pop:** each word scales about its centre on the baseline, from its own start (`enter_at`). The spoken word is in `accent`.
- **Tech:**
  - **The panel.** One `FillRect` (radius 24) behind the lines: the box of the lines and of a cursor's cell on either side of each, plus 32 lp of padding.
  - **The cursor.** A `FillRect` of 0.6 em by the word's height, right after the spoken word, in `active`. The spoken word is in `active` too.
- **KeywordPop's scale:** 1.0 → 1.25 at 90 ms → 1.10 at 180 ms from the item's start, eased ease-out cubic (`KEYWORD_POP_PEAK`, `KEYWORD_POP_HOLD` and `keyword_scale` in `motion.rs`, beside Bold's `POP_SCALE`, through `anim::Track`), held until the word ends, then 1.0. Its colour stays as Prompt 104 made it. On a popped word the KeywordPop's scale replaces the style's pop. The pill grows with the word it sits behind.
- **A pop that would leave the safe area:** a word grows by its pop or its KeywordPop only as far as the safe area allows (`room_to_grow`), counting:
  - its outline, which grows with it;
  - the pill's and panel's padding, the cursor's cell and the shadow's drop, which do not.
  A word that fills its line pops less, rather than leave the area (TS §19.6). See "Checked beyond the prompt" below.
- **Word boxes.** The pill and the cursor are placed by `hit::word_box(line, placed)`, the box hit testing uses (D-174), split out of `word_boxes`.
- **Tests** (inline, 9 new):
  - The emphasis order with the cursor and the pill.
  - The pill at the word start, +45 ms and +90 ms, never jumping (each millisecond moves it at most ease-out cubic's steepest step), on the first word, and absent in the pause and outside the words.
  - The pill's word in ink with no outline.
  - The shadow's layer and offset, in Bold too, none in Tech.
  - The shadow and the pill change no layout box.
  - Bold's pop.
  - Tech's panel and cursor at three times.
  - KeywordPop's scale and the pill growing with it.
  - A full-line word popping only within the safe area.
  - The word turning to ink under the sliding pill, and the ink alone at rest.

**6. `tests/styles_use_theme.rs`** (D-136): reads every `.rs` file under `offcut-scene/src` (19 files) and fails on any line with `Rgba {` outside `src/theme.rs` and `src/display_list.rs`, naming the file and line.

**Changed beyond the prompt's files.**
- **`captions.rs`, one line.** `lines` fits each line in the block less `style.decoration_width()`, where it took off the stroke only. A line that fits is unchanged. A line wider than the block less the pill's padding (Clean), the panel and cursor (Tech) is drawn smaller, so that the pill, the panel and the cursor stay in the block. Without it, the inline safe-area test (`every_command_of_every_frame_is_inside_the_safe_area`, whose 18-letter word fills its line) fails on the pill. The file is still 400 lines above its tests (known issue 22 stays open).
- **`hit.rs`.** `word_box` is split out of `word_boxes`, so the pill and cursor use the same box. No behaviour changes.
- **`lib.rs`** inline tests: see below.
- **`tests/timemap_identity.rs`:** see below.

**Changed inline expectations** (each because of D-170's redesign, as the prompt's requirement 2 asks):
- **`theme.rs`.** `every_style_is_drawn_in_v3s_colours_and_font` is replaced by `each_style_is_drawn_in_the_colours_and_fonts_of_its_row` and `each_style_sets_its_captions_in_its_rows_font` (§8.4's table, D-170).
- **`fonts.rs`.** `the_embedded_set_has_v2s_four_fonts_in_v2s_order` becomes `..._then_inter_800` (five faces). The coordinates test reads five (D-170).
- **`captions.rs`** (V3's test). `inside_a_chunk_exactly_one_glyph_run_uses_the_active_colour` becomes `..._is_the_spoken_word_on_the_pill`.
  - **Now:** the same three words at the same five times give the one run in ink, with no outline, and the other runs outlined at 5 lp.
  - **Was:** the one run in `accent`, every run outlined at 6 lp, exactly three commands.
  - **Allowed now:** layers, rectangles, and the spoken word's second run while the pill slides in.
- **`lib.rs`** (V3's and P104's tests):
  - `a_number_is_in_accent_when_highlighted_and_a_keyword_pop_from_its_start`: its helper skips shadow runs (fill `shadow`), and the spoken "now." is `on_accent` (was `accent`).
  - `at_a_profile_width_of_540_every_coordinate_is_half`: gains a `FillRect` arm (half the rectangle and the radius, the same colour); V3 drew no rectangle. Its `PushLayer` arm compares a clip as half the box, where it compared clips as equal. V3's layers had no clip, and a clip is a length, which `DrawCmd::scale` halves.
- **`captions_draw.rs`** (P104's test). `the_spoken_word_then_a_keyword_pop_then_a_number_then_the_fill`'s cursor case: Tech's spoken word is now in `active` (D-139 and D-170 both say "green active word"), where Prompt 104 had left it to the next layers. Prompt 104 read §8.5's `Cursor` as no colour; the decisions bind over the plan section (SAB precedence). The test's base style sets `Highlight` itself, since Clean is `Pill` now.
- **`timemap_identity.rs`.**
  - It finds the spoken word as the one run in `on_accent`, where it was the one run in `accent`.
  - Its edit is the default again: the two overrides Prompt 104 added (numbers not highlighted, the KeywordPop off) are gone, because no other run is in `on_accent`. It now times the default scene.
  - Its other assertions are unchanged.

**Checked beyond the prompt** (temporary `zz_` tests, deleted before the gate):
- **The safe area, corpus-wide.** All 31 labeled files in the three styles at 0.0, 0.15, 0.55 and 1.0, every frame (313,068 frames). The check covered every glyph run (outline included), shadow, pill, panel and cursor.
  - **The first run found Bold's pop leaving the area.** 8 frames of w10 ("25,000 DOWNLOADS", one caption word wider than Bold's line, shrunk to the block) went out by 59.0 lp at the pop's height, along with their shadow runs. This is new with this prompt (Bold did not pop before), and Prompt 106's `layout_safe_area.rs` would have failed on it. Hence `room_to_grow`.
  - **The second run** found 0.13 lp in 2 frames of w11: the outline grows with the word, which the first form of `room_to_grow` had counted as fixed. It is counted now.
  - **The third run:** nothing outside.
- **The pill at rest, corpus-wide.** In Clean, over the same files and placements, the pill at rest (its box the spoken word's, padded) never overlapped another word's glyph box on its line (0 of 56,012 frames). The 4 lp of vertical padding reaches into the empty top of the next line's box, which is expected.
- **The look:** see "The look" below.

**Guards.**

| Temporary edit | Result |
|---|---|
| `const DRILL: Rgba = Rgba { r: 255, g: 0, b: 0, a: 255 };` appended to `captions_draw.rs` | `styles_use_theme.rs` fails, naming the file, the line and the text; the file was restored and checked identical by SHA-256 |

The prompt's drill text, `Rgba { r: 1.0, … }`, does not compile (the channels are `u8`), so the drill wrote 255.

**Differs from the plan or the prompt.**
- **`Theme` and `Role` gain `on_accent`/`OnAccent` and `shadow`/`Shadow`.** See 2.
- **One shadow layer.** The shadow runs are opaque in one layer at 45%, rather than each run's colour at 45%: drawn separately, a run's fill and stroke would overlap and darken to about 70%, giving a two-tone shadow.
- **The word on the pill has no outline and no shadow.** D-170 does not say. A dark outline on dark text only thickens Inter 800, and a shadow on the pill shows as a smudge.
- **The ink is revealed by the pill while it slides.** D-170 has the word in `#111114` from its start, while the pill takes 90 ms to arrive. Drawn that way, the start of the word was dark text with no outline on the video for 2 or 3 frames (seen in the look frames below). Now it is outlined until the pill reaches it.
- **Tech's panel** is the lines' box with a cursor's cell on either side, plus 32 lp. §8.6 says the lines' box plus 32 lp, but a cursor after a line's last word (0.6 x 56 = 33.6 lp) would then reach past the panel's edge. Taking a cell on both sides keeps the panel centred on the text and its padding even.
- **Tech's spoken word is coloured.** D-139 and D-170 over §8.5, as above.
- **Lines are fitted to the block less the decorations** (`captions.rs`, one line). See above.
- **A pop is limited by the safe area** (`room_to_grow`). TS §19.6 over D-170's 1.12 and D-139's 1.25.
- **Values the decisions do not give:**
  - KeywordPop's easing (ease-out cubic).
  - That a KeywordPop's scale replaces Bold's pop on the same word.
  - That the pill's radius scales with the pill.
  - That the pill slides across lines, diagonally, when the previous word ends a line.
- **The cursor does not blink.** G 9.4 says "the cursor blinks only while a word is active", but no decision gives a rate. It shows while a word is spoken, so it goes and comes between words.
- **Tech's motion values** (linear, 80 ms) move nothing: its `enter` is `None`.
- **Where the new constants live.** Each is in one file:
  - `captions_draw.rs`: `PILL_SLIDE_MS` (named by §8.9) and `PILL_GROW_FROM`.
  - `motion.rs`: the KeywordPop keys, beside Bold's `POP_SCALE`. They were first written in `captions_draw.rs`, which then had 417 lines above its tests, so they moved; it now has 396.
  - `styles.rs`: the pill's radius and padding and the cursor's width (Appendix D).

**Observations** (for design review 1 and E-2):
- **Bold's numbers share the spoken word's colour.** Numbers highlighted (D-169, on by default) are in `accent`, which in Bold is also the spoken word's colour (§8.4). This is the clash Prompt 104 found in V3's Clean, which Clean's pill now avoids.
- **Tech's numbers** are green like its spoken word.

**The look** (`d1`). A temporary `offcut-render/tests/zz_look.rs` drew one sentence ("we spent three months building something nobody wanted.") over the golden pattern on WARP at 1080 x 1920 (PNGs in the session's scratchpad, not committed; the goldens are Prompt 107's):

| Frame | Seen |
|---|---|
| Clean, "spent" at rest | Two lines, "we spent / three months". The pill sits behind "spent" only, in ink, and the other words are white with a dark outline and the hard shadow below |
| Clean, 30 ms into "three" (first form) | The pill was half-way from the end of line 1 to the start of line 2, and "th" was dark on the video with no outline. This led to the reveal above |
| Clean, 30 and 60 ms into "three" (the reveal) | 30 ms: the word white and outlined where the pill has not reached it, ink where it has. 60 ms: the pill almost on "three", the word in ink |
| Clean, the first word at its start | The pill grows on "we" itself |
| Clean, between two words | No pill |
| Bold, 70 ms into "SPENT" | Upper case, "SPENT" in yellow at 1.12. **At the pop's height its outline touches "WE"**: 1.12 about the word's centre closes the 21 lp space. It lasts 2 or 3 frames. D-170's value is kept; it is a question for design review 1 (E-2) |
| Mono, "spent" spoken | The dark panel, even on every side; "spent" green; the cursor right after it, a monospaced cell wide, touching the next word as a terminal's cursor does |

**Out of the prompt: `ctutils` 0.4.3 → 0.4.5** (`Cargo.lock` only).
- **The advisory.** `pnpm check` failed on `cargo deny check advisories`: RUSTSEC-2026-0385 ("Non-constant-time mask generation and `u128` comparisons"), published after Prompt 104's run, against `ctutils` 0.4.3. The fix is "Upgrade to >=0.4.4".
- **Where it is used.** `ctutils` comes in through `digest` 0.11.3: `ed25519-dalek` (the entitlement check), `sha2` and `sqlx-postgres`.
- **The bump.** `cargo update -p ctutils` took it to 0.4.5, its latest release. No manifest and no other package changes.
- **CI.** It runs the same check, so this commit needs it.

**REOPENED CONTRACT** `offcut-scene/src/styles.rs` (D-136, D-170):
- `StyleSpec` gains `shadow`, and the three rows take their V4 values.
- Known issue 9 (closed in Entry 8) covered the move to theme roles. This is D-170's part of the same contract, so it gets no new row.

**How it was checked.**
- **`cargo test -p offcut-scene`:** all pass.
  - Inline 45 (Entry 14: 32): +1 `theme`, +1 `motion`, +2 `styles`, +9 `captions_draw`.
  - Integration: `hit` 3, `line_breaks` 2, `parity` 1, `placement` 3, `scene_cache` 1, `styles_use_theme` 1 (new), `timemap_identity` 1.
- **The prompt's grep.** `git grep -n "Rgba {" -- crates/offcut-scene/src | grep -vE "theme.rs|display_list.rs"` prints nothing.
- **Clippy.** `cargo clippy --workspace --all-targets -- -D warnings` and the wasm32 `offcut-wasm-render` clippy are clean; `cargo fmt --check` passes.
- **`node scripts/check-file-tree.mjs`:** ok, 319 files. `captions.rs` is 400 lines above its tests (unchanged), `captions_draw.rs` 396.
- **The SAB gate:**
  - On `v4-build`, no `zz` file, no test key. `pnpm build:wasm` exit 0 (render wasm 6,330,865 B; 6,287,309 at Prompt 98). `pnpm fixtures` up to date. `pnpm check` exit 0, after the `ctutils` bump.
  - **Rust: 594 passed, 0 failed, 3 ignored.** Entry 14 read 580/0/3; the 14 added are the 13 inline tests above and `styles_use_theme`.
  - Vitest 92. `pnpm build` exit 0 (app shell 308.8 kB, unchanged). `pnpm e2e` 12 passed.
  - The frozen check lists only the seven recorded one-line exceptions of Entry 4 and `offcut-dsp/tests/program_identity.rs`. No private path is tracked.
  - `pnpm e2e:media` was not run: the prompt asks for none. CI's `e2e-media` job runs it on this commit.
- **The look in `pnpm dev`** (the prompt's third validation command) is for the founder: see "Open".

**CI reading promised by Entry 15** (run 39878070122 on `86101dc`, green):
- **Times.** The run took 23.1 min: `ci` 13.0 min (00:25:57-00:38:55), `e2e-media` 10.1 min.
- **The cache.** "Restore the downloaded Ubuntu packages" fell back to the `apt-Linux-6e6f…` cache saved on `75c02c3` ("Cache hit for restore-key"). The Vulkan step reported "12 packages from the cache" and "Need to get 0 B/17.5 MB of archives", and took 10 s (6.9 min on `0fc25f9`).
- **Chrome.** The Playwright install needed "0 B/22.0 MB" and "0 B/145 MB" and took 22 s (7.2 min).
- **Saved.** The cache was saved under the new key `apt-Linux-f342…`.

**Found, not fixed.**
- **A KeywordPop in Tech may reach past the panel.** The panel is built from the layout boxes. A popped word grows 12.5% of its width each side at 1.25, past the panel's 65.6 lp of side room for a word wider than about 525 lp (16 or more monospaced characters at 56 lp). The corpus check above tested the safe area, not the panel. Prompt 106's `layout_safe_area.rs` checks Tech's panel box.
- **`Scene::summary`.** It still counts no emphasized caption (`summary.rs`, Prompt 106).
- **`captions.rs`.** Still exactly 400 lines (known issue 22, now Prompt 106's).

**Open.**
- **The look in motion, for the founder** (G 9.4, the prompt's `pnpm dev` check). Switch `empty().edit.style` in `web/src/state/clip-store.ts` to `"bold"`, then `"tech"`, and revert. Look for: Clean's pill gliding and never jumping, no pill in a pause; Bold popping, upper case, three words a chunk; Mono's panel following the chunk.
- **Bold's touching words** at the pop's height: keep 1.12, or lower it (E-2, design review 1).
- Open items 1-5 unchanged.
- Known issue 22 moves to Prompt 106.
