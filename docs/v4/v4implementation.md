# v4implementation.md - Offcut V4 "The look and the editor": implementation plan

**Implements:** `buildplan.md` section 6 (V4), which is steps M1.4 and M1.5 of `technicalspec.md` (TS) §28, **together with the edit-plan seams S-1 to S-7 of TS §40.9** (spec v1.2, the founder's decision of 2026-10-21). Product references are to `product.md` (PS, v1.2). "BP §n" is a section of `buildplan.md`; "V2 §n" of `docs/v2/v2implementation.md`; "V3 §n" of `docs/v3/v3implementation.md`; "V3 CL" is `docs/v3/v3changelog.md`; "V4 CL" is `docs/v4/v4changelog.md`. Decisions D-1 to D-112 are V1's, V2's and V3's; this document continues at D-113.

**Reader.** Whoever writes the V4 code, file by file. Everything needed to write a V4 file is in this document or in the TS section it names. V3 as built is the truth this plan builds on: where `v3implementation.md` and its changelog disagree, the changelog's "As built" wins. Where the specs and the earlier plans are silent or contradict each other, this document decides in section 2; each decision is marked **(V4 decision)**. A decision that changes something V1, V2 or V3 froze (V2 §25.1, V3 §21.1) is marked **REOPENED CONTRACT**, with the cost on both sides.

**The one-sentence goal.** At the end of V4, a dropped clip plays back in a synced preview with phrase-aware captions in a premium default style (or Bold, or Mono), all four visuals, 9:16 framing and a working scrub bar, in a clean dark editor where a creator fixes a word by clicking it, drags the captions or the framing on the video, switches visuals off, picks a style by looking at their own clip, undoes anything, and sees every change within 150 ms, with nothing ever waiting on the network; deployed and measured on D1 (BP §6).

**The quality bar (the founder's brief of 2026-10-22).** V4 decides how every short looks and how it feels to make one, so V4 is held to a higher bar than "the features of BP §6 work": **better captions, a better final video and a better experience than Submagic, for content creators in general**, not only technical creators. Two lists set that bar; sections 2.9 and 2.10 answer each line.

| Part A: caption intelligence (the output) | Where |
|---|---|
| Accurate transcription of names, slang and jargon | V2-V3 (the recognizer); V4 makes fixing fast: click a word (D-182), fix it everywhere (D-175) |
| Precise word-level timing | V2-V3; V4 keeps every time (INV-5) and adds a reading-time floor that uses silence only (D-168) |
| Smart line breaks, easy to read on a phone | D-167 |
| Highlights and animations on the right words | D-169, D-170 |
| Consistent placement that does not cover faces or content; draggable | D-171, D-183 |
| The export looks exactly like the preview | D-172 |

| Part B: editor experience (the process) | Where |
|---|---|
| A great default style, no settings needed | D-170, D-176 |
| A fast, smooth, synchronized preview | TS §20, D-148, D-149, D-193 |
| Click a word to correct it, see it at once | D-173, D-182, D-183 |
| Simple controls for style, highlights, framing, effects | D-183, D-186, D-187 |
| Smart suggestions instead of editing every detail | D-175, D-176 |
| Reliable exports, clear progress, easy recovery | D-152, D-184, D-189, D-190 |
| A clean, premium, user-friendly interface | D-177 to D-181, D-191, D-192, section 17 |

**The second goal, which decides how every file is written.** V4 is the last version that may change the shape of the scene, the renderer, the timeline or the audio. From V5 to V6.7 every later version must be additive: it fills function bodies, adds files, adds enum variants, catalog rows and table rows. In particular, **V6.1 (the AI edit plan), V6.2 (cuts) and V6.3 (music) must meet no contract that V4 has to reopen.** So V4 builds, under the features of BP §6, the seven seams of TS §40.9: the scene reads a validated **edit plan** (the rule detector is its first producer), time runs through an identity **time map** from recording time to program time, the **video is a draw command**, fonts come from a **registry** and colours from a **theme**, caption motion is **data**, and the preview plays **assembled program audio**. Section 22 lists, file by file, what V5 to V6.7 add, and shows that each row is additive.

---

## 0. Contents

1. Scope: what V4 is and is not
1A. Preconditions: what V4 starts from
2. Decisions this document makes (D-113 to D-193)
3. Tools, dependencies, environment variables
4. V4 file tree
5. Build order (17 steps, each with its own check)
6. `offcut-types`: two timelines, the plan's wire format
7. `offcut-plan` (new): catalog, prompt, validation, the rules producer
8. `offcut-scene`: the plan-driven scene, caption intelligence, the look
9. `offcut-render`: video as a command, native rendering, golden frames
10. `offcut-dsp`: program audio (identity)
11. WASM binding crates
12. Code generation
13. Server: one enum widened
14. Web infrastructure: protocol, RPC, pool, loaders
15. Workers: `render.worker`, `audio.worker`, the preview loop, the video source
16. State and use-cases: edits, cancel, retry, seek
17. Interface and interaction design
18. Fixtures, snapshots, golden frames
19. Root configuration, scripts, CI, hosting
20. Tests, case by case
21. Measurements and the V4 decisions of BP §6.2
22. Hand-over: V5, V6 and V6.1 to V6.7
23. Exit checklist
24. Open references, spec issues found, decisions to copy back
Appendix A. Per-file specifications in brief
Appendix B. Invariants V4 touches, and where each is enforced
Appendix C. Two per-file specifications written out
Appendix D. Named constants V4 introduces

---

## 1. Scope

### 1.1 In V4

| Area | What works at the end of V4 |
|---|---|
| Edit plan (seam S-2) | `crates/offcut-plan` exists: the plan's wire format (`EditPlan`, `PLAN_FORMAT = 1`), the capability catalog of the four V3 kinds (`CATALOG_VERSION = 1`), the generated LLM system prompt and output schema (snapshot-tested, sent nowhere), the rules producer `from_rules`, and `validate`, the only constructor of a `ValidPlan`. The scene reads a `ValidPlan` and nothing else describes the edit (D-118 to D-124) |
| Timelines (S-1) | `OutMs` (program time) beside `TimeMs` (recording time), and `TimeMap::identity`. The scene, the preview clock, the scrub bar and the frame index run on program time; source frames are read at `scene.source_at(t)`. Every map is the identity in V4, so program time equals recording time and INV-5's v1.1 behaviour holds exactly (D-125, D-126) |
| Captions | **Phrase-aware line breaks** that never strand an article or split a phrase, balanced lines, a 700 ms reading floor taken from silence (D-167, D-168); the **premium default** (Clean: Inter 800, a soft shadow, the spoken word on a sliding highlight pill), Bold, and Mono (V3's Tech), written as theme roles plus `CaptionMotion` data (D-136, D-137, D-170); numbers highlighted, stressed words popping (D-169) |
| Event visuals | All four kinds drawn (TS §19.5): NumberReveal (V2, now on program time), ListReveal (header, items appearing at their own times, numbered, auto-shrink past five), FromTo (two values, arrow, proportional bars, the "to" value emphasized), KeywordPop (D-140, D-141) |
| Caption placement | Drag the captions up or down on the preview (snaps at Top, Middle, Bottom) or pick a position in the Framing tab; the visuals move to the free side; the watermark never meets a caption (D-171, D-183) |
| Preview = export | One scene and one renderer at every size; display lists equal after scaling, and a 1080-vs-540 golden at SSIM ≥ 0.99 (D-172) |
| Framing | Portrait passes through; landscape and square get the 9:16 crop with the user's offset. The crop is part of the per-frame `Video` command (S-4), so a later zoom or reframe changes data only (D-131, D-142) |
| Watermark | Drawn by the scene when `profile.watermark` is set (INV-9), at the bottom-left of the safe area, never over a caption (D-143) |
| Renderer (S-4, S-5) | Draws the display list in order with the video as one `Video` command (any crop of the source, any axis-aligned placement); `Image` and video masks are typed but answer `Unsupported` until their first user. Fonts come from a `FontSet` registry (`FontKey` replaces `FontId`). Natively, `render_to_image` draws to memory on a software adapter for the golden frames (D-130 to D-135) |
| Program audio (S-6) | `offcut-dsp::program::assemble` exists and runs in `audio.worker` (`AudioWorkerApi.assemble`); with the identity map and no music it returns the cleaned voice sample for sample. The preview plays `prog48` (D-127, D-128) |
| Preview | Audio clock master; clock sync every 250 ms; a queue of six decoded frames; scrubbing (`seeking` state, keyframe restart, `previewSeek`); the 360x640 fallback when a frame costs over 28 ms on average; drift at most 80 ms over 60 s (TS §20) |
| Editor (S-7) | A dark, premium editor (D-177 to D-181): top bar with save state, undo, redo and Export; the preview as the hero with its transport; a side panel with **Captions** (the transcript: click a word to fix it, "Change all N", hide, restore, follow playback; D-182, D-175), **Style** (three cards rendered from the clip itself, Highlight numbers; D-186), **Visuals** (suggested visuals, a switch each; D-187) and **Framing** (caption position, crop; D-187). On the preview: click a caption word to edit it, drag the captions, drag the video sideways (D-183). Undo and redo (D-184), keyboard shortcuts (D-185). Every change is optimistic and local and on screen within 150 ms (D-173). A cuts section shows nothing while a plan has no cuts |
| Zero events | `NO_EVENTS_FOUND`: "No numbers or lists found in this clip. Captions and cleanup applied." (PS §10 J7) |
| Processing and export | A four-step checklist in plain words with live found-moment chips (D-188); an export pill with stage, percent, time left and Cancel (D-189); errors as inline cards in plain words with one action (D-190) |
| Design quality | Tokens, primitives, icons, Inter as the interface font, WCAG AA, an axe check, two design reviews by the founder on captured screens (D-179 to D-181, D-191, D-192) |
| Robustness | `rpc.ts` progress throttle (10/s, feed lines never dropped), cancel with `CANCEL_TIMEOUT = 2 s`, worker crash handling; `pool.cancel`, `pool.restartWorker`; `cancel-job.ts` for import, pipeline, model download and export; `ErrorPanel` with the copy of every code a person can meet in V4 and a retry; `client_error` and `job_cancelled` are sent (C-12, C-14) |
| Tests | Display-list snapshots (exact), safe-area check over the 31 labeled transcripts in three styles, framing, scene determinism, 16 golden frames, the plan crate's tests, the identity proofs; `rpc.test.ts`, the machine tests, `run-pipeline.test.ts`; `preview-sync.spec.ts`, `review-edit.spec.ts`, `cancel.spec.ts`, `pipeline-preview.spec.ts` complete |
| Analytics | `review_action`, `job_cancelled`, `client_error` start (BP Appendix C); `review_action`'s `action` gains `caption_position`, `highlight_numbers`, `replace_all`, `undo`, `redo` (section 13). `pipeline_done` unchanged |
| Measurements | Edit-to-preview time (at most 150 ms), preview drift and late frames, first preview frame, seek time, `frame_at` cost, scene build time, golden similarity across adapters; all on D1 (D-114) |

### 1.2 Not in V4

| Left out | Stand-in in V4 | Arrives |
|---|---|---|
| The export as an independent tool judges it: verifier checks 7-11, `timeline-preserved.spec.ts`, the render cache, `ChangeSummary` on screen | The V2 export path, now drawing the V4 look; it reads `out48.f32` (equal to `prog48` in V4) | V5 (S-8, S-9 included) |
| The watermark in a real Free export | Drawn whenever a profile asks for it; no Free export exists until sign-in | V6 |
| Accounts, the export gate's sign-in, checkout | V2's fake-API tokens in tests | V6 |
| A cloud plan, the `/plan` route, the `plans` IndexedDB store, the settings switch | The rules plan only, held in memory and rebuilt from the stored transcript (D-82 holds) | V6.1 |
| Any applied cut, a non-identity `TimeMap`, cross-fades | Every map is `TimeMap::identity`; `validate` drops any cut, chapter, text extra and music cue with `DropReason::NotInThisBuild` (D-121) | V6.2, V6.3, V6.6 |
| Video masks, the `Image` command, a second video slot | Typed; the renderer answers `RenderError::Unsupported` (D-133) | V6.5, V6.7 (images), P2 (B-roll slot) |
| Kinetic presets beyond the three styles, per-glyph motion | `CaptionMotion` holds the three; `MotionUnit::Glyph` exists and is not used | V6.5 |
| Brand kit, user fonts | `Theme::default_for(style)`; `FontSet::embedded()` | V6.7 |
| Restore after reload, recent clips, the restart budget, `.persistent` error copy, retry from the last completed stage | Retry runs the pipeline again from step A; a crashed worker is restarted once per call | V7 |
| The empty-output retry of the recognizer on WASM (TS §16.6, listed for V4 by V2 §25.2) | Not built; `E_ASR_RUNTIME` shows its copy and a retry | V7 (failure recovery), recorded in section 24 |
| The network-privacy test | V3's guards; V4 adds no request | V8 |
| The Canvas2D backend | Built only if section 21's decision asks for it | Contingency |
| Captions that move around a detected face on their own | The drag and the three snap points (D-171) | P1 (face tracking, PS §9.5) |
| "Maybe mis-heard" flags on words | None: the recognizer gives no per-word confidence (TE-2, D-176) | V6.1 (the model may mark doubtful words), V9 (E-10) |
| Custom caption fonts and colours | Three styles and Highlight numbers | V6.5 (presets), V6.7 (brand kit) |
| Undo across a reload | Undo within the session; edits are autosaved | V7 |

Nothing from TS §36 is built. Two rows matter most in V4. The time map exists (TS §40.1, TDR-7 as revised), but its only constructor in V4 is `identity`: there is no silence detection, no gap analysis, no cut, no cross-fade. And no user-inserted item: the user toggles what the plan proposed (PS §12.4).

### 1.3 Journeys and use-case steps served

| Flow | Served in V4 | Missing until |
|---|---|---|
| J6 Processing feed | Full, with a cancel button | - |
| J7 Preview (AHA) | Full: all four visuals, three styles, the honest zero-event message | - |
| J8 Review | Full for PS v1.1's controls: edit and hide words, toggle items, style, crop. The V6.x controls (cuts, music, presets, brand kit, copy post text) are V6.x rows of the same panel | V6.1-V6.7 |
| J10, J11 Export and download | V2's path with the V4 look | V5, V6 |
| C-5 Preview | Full, including seek | - |
| C-6 Review edits | Full | - |
| C-12 Cancel | Full for import, pipeline, model download, export | - |
| C-13 Restore | The records exist; nothing reads them | V7 |
| C-14 Failure and recovery | `ErrorPanel`, retry from step A, `client_error`, one restart of a crashed worker, GPU loss reopening the session | V7 (restart budget, resume) |

---

## 1A. Preconditions: what V4 starts from

Do not start S1 until every box is ticked.

- [x] 1. **V3 is closed.** `origin/main` holds the V3 merge `7f091fb`, the V3 close-out `9122eaf` and the founder's landing commit `550ff8f` ("Landing hero"), deployed by `ci.yml`. Tag `v3` points at `7f091fb`. Check: `git log --oneline -3 origin/main`; `git ls-remote --tags origin v3`.
- [x] 2. **Spec v1.2 is on `v4-build`** (the founder's choice of 2026-10-22: one branch for all of V4). The local branch `v4-spec` was renamed `v4-build`; it holds `ef1c42d` ("Spec v1.2: AI edit plan, V6.1-V6.7, seams for V4-V6") and `16b3a4c` (this plan), and `origin/main` (`550ff8f`) was merged into it (`81474bf`). Spec v1.2 reaches `main` with V4's merge, not before. Check: `git merge-base --is-ancestor ef1c42d v4-build`.
- [x] 3. **The branch.** `v4-build` is on `origin` (pushed 2026-10-22). The V4 baseline for code is `550ff8f`, the `origin/main` commit merged into it: every frozen-file check compares with it (the two spec commits change documents only). Check: `git merge-base v4-build origin/main` prints `550ff8f` until `main` moves; it is written into Entry 1 of `docs/v4/v4changelog.md`.
- [x] 4. **The test counts at the baseline** are taken on D1 and written into Entry 1: at V3's close they were 461 Rust passed, 0 failed, 3 ignored; 92 Vitest; 12 Playwright (`pnpm e2e`); 48 media cases (49 passed with the setup cases, 1 `test.fixme` skip). No count may fall during V4, except the V2 and V3 cases this plan rewrites by name (D-160). Check: `pnpm test`, `pnpm e2e`, `pnpm e2e:media` on a keyed build.
- [x] 5. **The V3 hand-over markers are where V3 §21.3 says.** `git grep -nE "\bV4\b" -- web/src crates` lists the places V4 fills: `render.worker.ts` (word edits), `run-pipeline.ts` and `import-clip.ts` (cancel, `client_error`), `start-export.ts` (cancel), `preview-store.ts` (seek events), `load-render.ts` and `detect_api.rs` (`redetectSentence`), `video-source.ts` (`prefetch`), and the nine `V4: ` markers in `offcut-scene` and `offcut-render`. Check: the command's output, compared with section 4's list.
- [x] 6. **The four full files have their sizes of section 4.** `web/src/workers/render.worker.ts` 372 lines, `web/src/workers/rpc.ts` 377, `web/src/workers/render/video-source.ts` 399, `crates/offcut-wasm-render/src/session.rs` 381 (tests included where a file has them). V4 adds to each, so each is split first (D-150). Check: `wc -l`.
- [x] 7. **A software graphics adapter runs natively on D1 and on the Linux runner.** On D1: WARP through `wgpu`'s DX12 backend. On the runner: Mesa's `lavapipe` through Vulkan (`mesa-vulkan-drivers`). Check: Step 0 of the build guide runs a 20-line probe (`cargo run -p offcut-render --example adapter_probe`, deleted after) on both, which prints the adapter name and `DeviceType::Cpu`, and renders one Vello scene to a buffer. If either fails, D-134's fallback is taken before S1.
- [x] 8. **The two new dependencies pass `cargo deny`** on a scratch branch: `insta` (Apache-2.0), `png` (MIT OR Apache-2.0). Check: `cargo deny check licenses bans` with both added as dev-dependencies.
- [x] 9. **The V3 bench files are present**: `bench/results/d1-2026-10-19-portrait.json` and `-v3testclip.json`. V4's preview and scene measurements compare against them.
- [x] 10. **V3 CL open item 7** (the Linux ffmpeg URL dies around 2026-11-27) is either resolved or still before its date. If the date is near, the founder decides at Step 0 whether S1 moves the URL (V4 CL item 5).

**Carried over from V3.** None blocks V4. V3 §23 items 15-28 stay open with the version each names. Two are V4's: item 24 (`video-source.ts` has no room) is D-150; item 25 (the cleaned audio of a clip that ends early stays in `audio.worker`) is closed by D-128, because the worker no longer keeps audio between calls.

---

## 2. Decisions this document makes

Numbering continues from V3's D-112. Each is the smallest choice that keeps V5 to V6.7 additive. Numbers no experiment has measured are marked **(assumption, ID)**.

### 2.1 Process

| # | Decision (all **(V4 decision)**) | Why |
|---|---|---|
| D-113 | `docs/v4/v4implementation.md`, `docs/v4/v4buildguide.md`, `docs/v4/coding-promptsv4.md` (Prompts 91 onward), `docs/v4/v4changelog.md` (opened 2026-10-21 with Entry 0) and `docs/v4/experiments.md` are in the TS §5 tree. The changelog and this plan are listed already; the other three are added in the commit that creates each. A temporary file (D-117's test and its JSON files) is listed in TS §5 for as long as it exists | Same pattern as D-46, D-73. `check-file-tree.mjs` fails on an unlisted file |
| D-114 | **D-69 and D-74 hold for V4.** Every gate and measurement that names R1 is read on D1 and recorded under the label `d1`; thresholds do not change; a budget for a 60 s clip is read against the budget times the clip's length over 60 s | The founder's standing decision. A pass on D1 does not show a pass on R1 (V3 §23 item 21) |
| D-115 | **All of V4 is built on one branch, `v4-build`**, which starts with the spec v1.2 commits and `origin/main` at `550ff8f` merged in (the founder's decision of 2026-10-22: "exactly like V3 … commit to that branch only"). Commits are titled `V4 P<nn>: <title>`. Each commit goes on `v4-build` only and is pushed to `origin v4-build` by the agent, and CI is read on it (the founder's rule of 2026-10-18 for V3, confirmed for V4 on 2026-10-22). Opening and merging the pull request, pushing to `main` and tagging are the founder's. Before the merge: `pnpm e2e:media` all green on D1 on a keyed build (D-72) | A push to `main` deploys. Same practice as V2 and V3 |
| D-116 | **Contracts first, then the identity proof, then the look.** S1-S6 change types, crates and the renderer without changing a pixel: the V3 tests pass unchanged and the display lists of D-117's reference set are equal to V3's. Only then do S7 and later add Bold, Tech, the three event visuals, the watermark, the crop offset and the review flow | A seam that is proven to change nothing is a seam V6.x can trust. Mixing the contract switch with new visuals would make every difference ambiguous |
| D-117 | **The identity reference.** At S2, before any scene file changes, a temporary test (`offcut-scene/tests/v3_reference.rs`, kept until S6, deleted at S7) writes the display lists of six scenes built by the V3 code to `crates/offcut-scene/tests/snapshots/v3_reference/*.json`: the `busy()` transcript of `lib.rs` at 1080 and 540, the reference clip's transcript and events from `fixtures/speech/README.md` at 1080, and three labeled transcripts (`a01`, `w01`, `w07`) with their detected events. At S6 the same test builds the same inputs through the V4 contract (rules plan, identity map, `Theme::default_for(Clean)`, `FontSet::embedded()`) and asserts equality after one normalisation: the leading `Video` command is removed and every `FontKey(n)` reads as V3's `FontId` of the same order (D-132). The JSON files are deleted with the test | No snapshot or golden exists from V2 or V3, so a recorded reference is the only way to prove the switch is a pure refactor |

### 2.2 The edit plan (TS §40.2-§40.4)

| # | Decision | Why |
|---|---|---|
| D-118 | **A plan item's kind is `EventKind`.** No `ItemKind` type is added. The four V3 kinds keep their codes 0-3 in the event-id hash (D-47, unchanged); each later kind is a new variant appended to `EventKind` by the release that draws it, with the next code (4, 5, …). `EventId` stays the id of a plan item; `EditState.event_overrides` (unchanged field name) holds the user's toggles of plan items | One enum, one id function, one override map: stored toggles survive from V3 to V6.x with no migration. A second kind enum would duplicate the analytics enum, the feed's `eventKind` and the hash |
| D-119 | **The wire format is fixed once, at `PLAN_FORMAT = 1`.** `offcut-types/src/plan.rs` defines `EditPlan { format, catalog, source, cuts, items, chapters, extras, music }` with every field and every nested type of TS §40.2, including all five `Cut` arms, `Chapter`, `TextExtras`, `MusicCue`, `PlanSource` with both arms, `Label` and `QuantityRef`. Serde: snake_case, `deny_unknown_fields`, tagged enums (`"kind"`). **This refines TS §40.9's "nothing else is reserved"**: the types of the wire format exist from V4; what the later releases add is the code that applies each field | The plan is a contract between three parties from V6.1 on (server, client, IndexedDB). Fixing its shape once means V6.1 stores plans V6.2 can read, and the generated TypeScript type does not change shape in every release |
| D-120 | **Item parameters.** `PlanItem { id: EventId, kind: EventKind, anchors: WordRange, params: ItemParams, confidence: Option<Confidence> }`. `ItemParams` is a tagged enum with one arm per catalog kind that **points at words** (`NumberReveal { value: QuantityRef, label: Option<Label> }`, `ListReveal { header: Option<Label>, items: Vec<ListPoint> }` with `ListPoint { at: WordIdx, label: Label }`, `FromTo { from: QuantityRef, to: QuantityRef, label: Option<Label> }`, `KeywordPop { word: WordIdx }`), **plus one arm `Detected(EventParams)`** that only the rules producer writes: the detector's own reading of the effective words, with its unit inheritance (D-85) and labels (D-107) | The detector's FromTo inherits units across its two ends ("90 to 10 minutes" shows "90 minutes"); a word pointer re-parsed alone would lose that, and re-implementing the detector's rules in the validator would make two implementations of one rule. The rules plan is rebuilt from fresh detector output on every word edit (D-146), so its values never go stale. The model never writes `Detected`: V-2 refuses it in a `Model` plan, and the generated schema does not contain it |
| D-121 | **`validate` resolves; the scene draws resolved values.** `offcut-plan::validate(plan, transcript, edits, cfg) -> Result<(ValidPlan, Vec<Dropped>), PlanRejected>` applies V-1 to V-6 of TS §40.4 to items, turns every `QuantityRef::Words` into a `Quantity` with `parse_quantity` over the effective tokens of its range (V-5), copies a `Detected` arm through after checking its anchors (V-3), computes each item's `Span` from the transcript, and returns `ValidItem { id, kind, anchors, span, params: EventParams, marks, confidence }`, where `marks` are the recording times a visual steps at (FromTo's second value, section 8.7). In V4, V-7 and V-8 drop **every** cut, chapter, text extra and music cue with `DropReason::NotInThisBuild` (each later release replaces that drop with its rule). **This refines TS §40.9 S-3** ("resolved at scene build"): resolution happens once, in `validate`, and `offcut-scene` never parses a number | One place parses numbers for the screen (INV-22), and that place re-runs on every edit. The scene keeps its V3 layout code, which takes `EventParams` |
| D-122 | **`ValidPlan` lives in `offcut-plan`**, with private fields and no public constructor besides `validate` (INV-21, checked by a trybuild compile-fail case). `offcut-scene` depends on `offcut-plan`; `offcut-plan` depends on `offcut-types`, `offcut-text` (tokenize, `parse_quantity`, `format_quantity`), `serde`, `thiserror`, and `serde_json` only behind its default feature `prompt` (the schema generator); `offcut-wasm-render` depends on it with `default-features = false`, so the render bundle carries no schema code. **Spec correction** to TS §2 (the `offcut-plan` row named only `offcut-types` and `serde_json`) and TS §7 (two new edges: scene → plan, plan → text) | The validator must parse numbers (V-5) and format inherited units; the scene must name the validated type. A type with a private constructor can only be guarded inside the crate that builds it |
| D-123 | **The V4 catalog** holds the four kinds, each `producers: Both`, in the order NumberReveal, ListReveal, FromTo, KeywordPop. `CATALOG_VERSION = 1`. `prompt::system_prompt()`, `prompt::output_schema()` and `prompt::user_message()` are written and snapshot-tested with `insta` in V4, and called by nothing outside tests until V6.1. `catalog_complete.rs` asserts that every catalog kind has a layout, a draw function and a golden frame, and that every drawable kind is in the catalog (INV-23) | TS §40.3: the prompt must be generated from code before the first request exists, so V6.1 adds a route, not a prompt. Writing it in V4 also proves the catalog can describe the four kinds the renderer draws |
| D-124 | **The rules plan is computed in the render worker and held by the page.** A new stateless method `RenderWorkerApi.rulesPlan({ transcript, events }) -> EditPlan` runs `from_rules` in the render bundle. The page keeps the plan in the clip store and sends it to `setScene`, which validates it in Rust before building the scene. Nothing stores a plan in V4: on a restore (V7) the rules plan is rebuilt from the stored transcript and prosody, as D-82 rebuilds events | From V6.1 the page receives the model's plan from the network and must hand it to the same `setScene`; the rules plan takes the same road from V4, so V6.1 swaps the producer, not the flow |

### 2.3 Timelines and audio (TS §40.1, §40.6)

| # | Decision | Why |
|---|---|---|
| D-125 | **REOPENED V1 CONTRACT (`units.rs`).** `OutMs(u32)` joins the unit types, with `OutSpan { start: OutMs, end: OutMs }`. `FrameIdx::to_time_ms` is renamed `to_out_ms` and returns `OutMs`; its two Rust callers and the TypeScript frame arithmetic follow. `TimeMap` (`offcut-types/src/timemap.rs` [ONLY]) has a private `segments` field; V4 gives it one constructor, `TimeMap::identity(duration)`, and the methods `is_identity`, `out_duration`, `to_out`, `to_src`, `joins` and `out_span(Span) -> Option<OutSpan>`. `TimeMap` serializes as `{ "segments": [...] }` so the page can hand it to two workers | TS §40.1. Cost on the V1 side: one renamed function and its callers; on the TypeScript side a new branded type. A frame index is a program position from now on |
| D-126 | **The scene runs on program time.** `build_scene` maps every transcript and item time through `time_map.out_span` exactly once; captions, chunks, keyframe tracks and visibility are in `OutMs`; `frame_at(t: OutMs)`; `source_at(t: OutMs) -> SrcPos`; `duration() == time_map.out_duration()`. `anim::Track` keys become `OutMs`. **REOPENED V1 CONTRACT (`protocol.ts`)**: `ClockSync.audioMs` and `previewSeek.at` are `OutMs`; `control-preview.ts`'s minting entry (D-59) mints `OutMs` instead of `TimeMs` | Everything the user sees and hears is in program time; only the source decoder reads recording time. With the identity map every number is unchanged, so the V3 tests' expectations hold |
| D-127 | **REOPENED V3 CONTRACT (`AudioWorkerApi`, frozen by V3 §21.1).** One method is added: `assemble(p: { out48: Float32Array; timeMap: TimeMap; speech: OutSpan[] }): Promise<{ prog48: Float32Array; loudness: Lufs }>`. `out48` is transferred in and `prog48` out; the worker keeps nothing after the call. `measureProsody` keeps transferring `out48` out unmodified. C-4 step F becomes: `opfs.writeAudio(out48)` (the copy the export reads in V4), then `assemble` with the identity map, then `prog48` to the clip store for the preview. `speech` is ignored until music (V6.3). `loudness` is measured over `prog48` with the chain's meter | TS §40.6, seam S-6. The page plays program audio from V4, so V6.2 and V6.3 change what `assemble` computes, not who calls it. Cost: one more worker call (about 30 ms on D1 for 60 s: a copy and a loudness reading, assumption S8) and one more contract row; on the V3 side, the worker stops holding audio between calls |
| D-128 | **The worker holds no audio between calls.** `audio.worker`'s `kept` buffer is still set by `runChain` and handed over by `measureProsody`, as V3 built it; `assemble` holds nothing. On `cancel`, on a pipeline failure and on "start over", `run-pipeline.ts` calls `pool.audio.release()`, a new one-way method that drops `kept`. **Closes V3 §23 item 25** | TS §18.4: the worker keeps audio only until `measureProsody`. The cancel path of V4 makes the early-ending clip common, so the leak would now happen often |
| D-129 | **V4 exports still read `out48.f32`.** `start-export.ts` and `export-loop.ts` keep V2's audio path; with the identity map `prog48` equals `out48` sample for sample (`program_identity.rs`). V5 (S-8) writes `prog48.f32` and encodes it. `export-loop.ts` changes in V4 only where the session's `render_frame` now takes `OutMs` (the same number) | BP §6: "Not in V4: export beyond the V2 spike". Keeping the export untouched keeps V5's verifier work separate |

### 2.4 Scene and renderer (TS §19, §40.5)

| # | Decision | Why |
|---|---|---|
| D-130 | **REOPENED V2 CONTRACT (`display_list.rs`, frozen by V2 §25.1).** `DrawCmd` gains `Video { slot: VideoSlot, src: CropRect, dst: Rect, opacity: f32, mask: Option<Vec<PathEl>> }` and `Image { image: ImageKey, rect: Rect, opacity: f32 }`; `VideoSlot` has one member, `Main`. `CropRect` moves from `offcut-scene/src/lib.rs` to `display_list.rs`. **`dst` is an axis-aligned `Rect`, not an `Affine`**: this refines TS §40.5, because every V6.x use (zoom, picture in picture, a shape frame's bounds) is axis-aligned and the mask carries any other shape. The scene emits exactly one `Video { Main, src: crop, dst: full canvas, opacity: 1.0, mask: None }` as the first command of every frame | Seam S-4. A rectangle is what the video pass can place with four vertices; a rotation would need a second sampler path no feature asks for |
| D-131 | **`Scene::crop()` is removed; the crop is per frame.** `framing::crop_rect` is still the only place the crop is worked out (`[ONLY]`), now with the user's offset (V2's stand-in removed); `frame_at` puts it in the `Video` command. A per-frame crop is how V6.1's punch-in and V6.2's jump-cut zoom will be expressed (a smaller `src` around the same centre), and how face tracking (P1) will move it | TS §19 seam "`crop()` becomes `crop_at(t)`" taken now, in the form of the command |
| D-132 | **REOPENED V2 CONTRACT (D-29, `FontId`).** `FontId` is replaced by `FontKey(u16)` and a `FontSet` registry. `FontSet::embedded()` registers the three files at five instances, with keys 0 = Inter 700, 1 = Inter 900, 2 = JetBrains Mono 700, 3 = Noto Emoji (the order of V2's enum) and 4 = Inter 800 (D-170, appended). A `FontSet` holds, per key, the font file, its family name, its weight and its normalized coordinates; `fonts.rs` builds the parley collection from it. `Renderer::render` takes `&FontSet` beside the list (D-133). `FontSet::with_user(files)` is V6.7's | Seam S-5. No snapshot of `FontId` exists, so the cost on the V2 side is code only; the order is kept so the identity reference (D-117) maps one to the other |
| D-133 | **REOPENED V2 CONTRACT (`Renderer::render`).** `render(&mut self, sources: &VideoSources, list: &DisplayList, fonts: &FontSet) -> Result<(), RenderError>`, where `VideoSources` maps `VideoSlot::Main` to the decoded frame and the clip's rotation. The compositor draws commands in list order. **V4 supports:** one `Video` command per list, first in the list; any `src` inside the source's display frame; any `dst` inside the canvas; opacity 1.0; no mask. A list outside that, an `Image` command or a mask returns the new `RenderError::Unsupported`, mapped to `E_INTERNAL` with detail `Unsupported`. Pixels of the canvas outside `dst` are black | The general compositor (several video commands, Vello layers under and over the video, masks) is V6.5's work and changes only the compositor's body |
| D-134 | **Native rendering for tests.** `offcut-render` gains, behind `#[cfg(not(target_arch = "wasm32"))]`, `Renderer::new_headless(width, height)` and `render_to_image(&mut self, source: &RgbaImage, rotation, list, fonts) -> Result<RgbaImage, RenderError>`, reading the frame from memory and the target into memory. The dev-dependency `wgpu` turns on the native backends `vulkan` and `dx12`, and the test asks for a software adapter (`force_fallback_adapter: true`; WARP on Windows, `lavapipe` on Linux). **Fallback if 1A item 7 fails on one machine:** the golden test runs on the machine where it works (the Linux runner first) and is skipped elsewhere with the reason printed; it is never weakened | TS §19 acceptance asks for goldens on a software adapter; V2 §25.2 listed `render_to_image` for V4. A software adapter makes the goldens the same on every machine that has it |
| D-135 | **Golden frames.** `fixtures/golden/*.png` holds 16 images at 540x960 (the preview size, half the cost of 1080x1920): the 12 of TS §26 (3 styles x 4 kinds), watermark on and off (Clean, NumberReveal), one landscape crop at offset 0.5, one punch-in (a hand-written list whose `Video.src` is the centre 1/1.1 of the frame). The source frame is a synthetic pattern drawn by the test (gradient, grid, a disc), never a recording. Similarity is SSIM over 8x8 luma windows, implemented in the test file (no crate), at least 0.98 (assumption, M1.4: V3 §23 item 14). `OFFCUT_GOLDEN_UPDATE=1` writes the images; CI never sets it | TS §26 names 14; the crop and the punch-in prove S-4. A synthetic source keeps recordings out of git (D-76) |
| D-136 | **REOPENED V2 CONTRACT (`styles.rs`): colours are theme roles.** `offcut-scene/src/theme.rs` [ONLY colour values] defines `Theme { text, stroke, accent, accent_alt, panel, backdrop: Rgba, display_font, body_font, mono_font: FontKey }` and `Theme::default_for(StyleId)`. `StyleSpec` names roles (`fill: Role::Text`, `active: Role::Accent`, …) and a `FontRole` (`Display`, `Body`, `Mono`), never an `Rgba` or a `FontKey`. **Refines TS §40.5**: a third font role, `mono_font`, because Tech's captions are monospaced and a brand kit replaces display and body fonts, not the code font. `styles_use_theme.rs` asserts that no `Rgba {` literal appears in `offcut-scene/src` outside `theme.rs` and `display_list.rs` | Seam S-5. A brand kit (V6.7) then becomes one `Theme` value |
| D-137 | **Caption motion is data.** `StyleSpec.motion: CaptionMotion { unit: MotionUnit, enter: MotionPreset, active: ActiveTreatment, easing: Easing, duration_ms: u32 }`. V4's values: Clean `Word`, `Fade`, `Highlight`, ease-out cubic, 120 ms; Bold `Word`, `Pop`, `Highlight`, back-out, 140 ms; Tech `Word`, `None`, `Cursor`, linear, 80 ms (TS §19.4, all (assumption, E-2)). `MotionUnit::Glyph` exists; no V4 style uses it. `MotionPreset` and `ActiveTreatment` get their V6.5 members by appending | Seam S-5. Kinetic presets (V6.5) are rows of this data plus new arms |
| D-138 | **Zones are a table.** `safe_area.rs` replaces `EVENT_ZONE` and `CAPTION_ZONE` with `pub fn zone(id: ZoneId) -> Rect` over `ZoneId { Event, Caption, Full }` (Event y 860-1180, Caption y 1220-1500, Full the safe area). Later releases append rows (`Top`, `LowerThird`, …) | Seam S-5; V6.x items claim zones through the catalog (V-6) |
| D-139 | **Style values** (TS §19.4, all (assumption, E-2); **Clean and Bold are superseded by D-170**; Tech is kept as written here and named "Mono" in the interface): Clean Inter 700 64 lp, as spoken, 18 chars x 2 lines, 5 words, white fill, 6 lp dark stroke, active word in accent `#FFD60A`; Bold Inter 900 84 lp, upper case, 12 x 2, 3 words, 10 lp stroke, the active word pops 1.0 → 1.12 → 1.0 over 140 ms back-out; Tech JetBrains Mono 700 56 lp, as spoken, 22 x 2, 5 words, no stroke, a rounded panel (radius 24 lp, 32 lp padding) at 85% opacity behind the chunk, active word green `#3DDC84`, a block cursor (0.6 em wide, the line's height) after the active word. KeywordPop in every style: 1.0 → 1.25 → 1.10 over 180 ms from the word's start, colour `accent_alt` | TS §19.4 gives the table; the pop amplitude for Bold, the panel's radius and padding, and the cursor size are not given |

### 2.5 Event visuals, watermark, summary

| # | Decision | Why |
|---|---|---|
| D-140 | **ListReveal layout.** In the Event zone: the header (when not empty) at 56 lp in the display font, then up to five items at 48 lp, each `"{ordinal}. {text}"`, stacked with 16 lp between lines, left-aligned at the zone's left plus 40 lp. From six items on, every text shrinks by the same factor so that all lines fit (minimum 32 lp; below that the extra items are not drawn and the event counts as drawn). Each item fades in and rises 24 lp over 200 ms ease-out from its own `at` (in program time); the whole list holds until the last item's end + 1,500 ms and fades over 200 ms (TS §19.5) | TS §19.5 gives the timing, not the sizes |
| D-141 | **FromTo layout.** Two columns in the Event zone. Each shows its value at 96 lp (auto-fit to the column, minimum 56 lp; below it the event is not drawn, as TS §19.7 does for a number) above a bar whose height is `value / max(from, to)` of 180 lp, minimum 8%. The "from" column appears at the first anchor, the arrow (a filled path) and the "to" column at the second anchor's first word; the "to" value in `accent`, scaled 1.15. No label row in V4 (`FromTo.label` is `None`, D-85). A negative or zero value draws a bar of 8%. Hold to `span.end + 1,500 ms`, fade 200 ms | Same; the second anchor's time is read from `ValidItem.anchors` and the transcript, not stored in `EventParams` |
| D-142 | **The crop offset.** Dragging the video on the preview is the main way (D-183); `CropOffsetControl` (landscape and square clips only) is a range input from -1.0 to 1.0, step 0.01; the value is committed on release (`change`), not on every move, so one drag is one `setScene` and one `review_action`. Dragging previews through the next `setScene`, not through a per-move redraw | PS §8 principle 3 allows one crop offset; a scene rebuild per pointer move would cost a scene build per event |
| D-143 | **Watermark.** `watermark.rs`: the text `WATERMARK_TEXT = "Made with Offcut"` (the working name A-11; one constant), Inter 700 at 36 lp, white at 60% opacity with a 3 lp dark stroke, its baseline 6 lp above the bottom of the safe area and its left at the safe area's left (TS §19.1 anchor). `layout_safe_area.rs` asserts that its box overlaps no caption box in any style on any labeled transcript, at the three snap points of D-171. (assumption, alpha: PS §19 minor) | The bottom of the caption zone is the bottom of the safe area: an overlap is possible with Bold's two lines and must be shown not to happen, not assumed |
| D-144 | **`summary.rs`** (`change_summary`): `voice_cleaned: true` (the chain always runs since V3), `captions_emphasized`: the drawn KeywordPop items, `visual_moments`: the drawn items of the other kinds. A disabled or undrawable item is not counted | TS §10.6. V2's stand-in returned `false` and counted NumberReveal only |
| D-145 | **Item visibility in the scene.** An item is drawn when `event_overrides.get(id)` is `true`, or is absent and the item is enabled by default (every V4 item is). Overrides of ids that no longer exist after a word edit are kept in `EditState` and ignored | TS §10.8: toggles survive by `EventId`; an override must not be lost because a sentence was re-detected and the event came back with the same id |

### 2.6 Review, preview, robustness

| # | Decision | Why |
|---|---|---|
| D-146 | **A word edit, end to end.** The interface is the transcript editor of D-182 and the click on the preview of D-183; this row is the flow behind them. `edit-word.ts`: clip `ready → updating`; `edit.word_edits` updated; `editsRepo.put`; `pool.render.redetectSentence({ sentence, wordEdits })` → events; `pool.render.rulesPlan({ transcript, events })` → plan; `pool.render.setScene(...)`; clip `updating → ready`; `review_action { action: "word_edit" }`. The worker keeps the transcript, the prosody and the last events from `detect` to serve `redetectSentence` (the frozen method takes no transcript) | C-6, with the plan step of D-124. One word at a time is the smallest control that fixes a mis-heard word; PS forbids a timeline |
| D-147 | **The review panel's update rule.** Every review use-case: if the preview is `playing`, `controlPreview.pause()` first and `play()` after `setScene` resolves (TS §12.4); a second change while one is in flight waits for it (one queue in `clip-store`'s `updating` state, the machine refuses `edit_start` from `updating`); an `IllegalTransitionError` is a bug. The edit is optimistic: the store holds the new `EditState` before the worker answers, and a worker failure there is shown by `ErrorPanel` with "Undo this change", which restores the previous `EditState` and calls `setScene` again | TS §13.4 "edits are optimistic and local"; C-6 "preview paused during the update, then restored" |
| D-148 | **Seek.** `PreviewPlayer` gains a scrub bar (`input type="range"`, 0 to the clip's program duration, step one frame). Pointer-down → `previewStore.seek()` (`playing|paused → seeking`); each input event → `controlPreview.seek(at)` (throttled to one in flight); pointer-up → `seek_done_playing` or `seek_done_paused` and, when it was playing, the audio restarts at `at`. `RenderWorkerApi.previewSeek({ at })` draws the frame of `at` with `frameAtBlocking` and resolves; `video-source` restarts at the keyframe at or before the source time when `at` is outside its queue (TS §20.2). Target: first frame at most 300 ms after the seek for GOPs of 2 s or less (assumption, M1.5); a spinner after 300 ms. The scrub bar is a player control, not an edit slider (PS principle 3 is about edits) | TS §20, C-5's `<Seek>` |
| D-149 | **The preview's 360x640 fallback.** `runPreview` measures `render_frame`'s cost; when the mean over the last 60 frames exceeds 28 ms, it calls `session.resize(360, 640)` once and the scene is set again at the preview profile scaled by 2/3. The page's canvas element keeps 540x960 CSS pixels. It is recorded in `PreviewStats.downscaled` and never undone for that clip (assumption, M1.5) | TS §20.3 |
| D-150 | **Making room in four full files** (V3 §23 item 24, TS §29's 400 lines). `workers/rpc.ts` splits into `rpc.ts` (the page side: `createClient`, `WorkerCallError`, `Cancelled`, the shared message types) and `rpc-worker.ts` (the worker side: `serveWorker`, `WorkerFailure`, `CANCELLED`, `toAppFailure`, `JobContext`, the throttle). `workers/render/video-source.ts` moves its decoder management (creating, feeding, draining and restarting the `VideoDecoder`, the colour-space memory of D-70) into `render/frame-decoder.ts`; `video-source.ts` keeps the queue, `frameAt`, `frameAtBlocking`, `prefetch` and `close`. `workers/render.worker.ts` moves the export handlers (`writeExport`, `afterExport`, `exportClip`) into `render/export-job.ts` and the preview handlers into `render/preview-job.ts`; the worker file keeps the session state and the handler table. `offcut-wasm-render/src/session.rs` moves the scene-building exports (`set_scene`, `rules_plan`, `source_at`, `summary`) into `scene_api.rs`. Each split is its own step with no behaviour change (S3), checked by the V3 tests | The only way to add V4's code under the limit; a split by function keeps each file's role readable |
| D-151 | **Progress throttle.** `rpc-worker.ts` sends at most 10 `progress` messages per second per job: a progress without a feed line inside 100 ms of the last one sent is held and replaced by the next; a held one is flushed before the job's `res`. A progress carrying a feed line is always sent at once (TS §14.3: feed lines are never dropped) | TS §14.3 |
| D-152 | **Cancel.** `pool.cancel(kind)` posts `{ type: "cancel", jobId }` for that worker's pending job and resolves when the job answers or after `CANCEL_TIMEOUT = 2,000 ms` (assumption, M1.5); on timeout `pool.restartWorker(kind)` terminates the worker, rejects its pending calls with `E_WORKER_CRASH` and makes a fresh one on the next call. `usecases/cancel-job.ts::cancelJob(scope)` with `scope: "import" \| "pipeline" \| "model" \| "export"`: import → media; pipeline → whichever of media, asr, audio, render holds the pipeline's current job, then `pool.audio.release()` and `pool.render.closeSession()`, clip `processing → accepted`; model → `modelManager` aborts its download (the bytes stay for resume); export → render, partial file deleted, export `rendering|muxing → cancelled → idle`. Each sends `job_cancelled { stage }` | C-12, TS §14.3 |
| D-153 | **Crash and retry.** A worker's `error` or `messageerror` event rejects its pending calls with `E_WORKER_CRASH` (stage of the pool row) and marks the client dead; the next call creates a new worker (`restartWorker`). In V4 there is no restart budget (V7): a second crash in the same clip is shown as the first was. `ErrorPanel` shows the code's copy (section 17) and one retry: for a clip in `failed`, `runPipeline` from step A (`failed → processing` by `retry`); for a failure while `ready` (the preview or an edit, `E_GPU_LOST`, `E_GPU_INIT`, `E_WORKER_CRASH` of the render worker), "Reload preview" reopens the session, sets the scene and attaches a new canvas without re-processing. Every shown failure sends `client_error { error_code, stage }` once | C-14; resume from the last completed stage needs the records V7 reads |
| D-154 | **`NO_EVENTS_FOUND`** is shown in the Visuals tab (`PlanItemList`, D-187) when the valid plan has no item, in place of the empty list; the copy is PS §10 J7's sentence. `pipeline-preview.spec.ts` asserts it on `speech_20s_noevents.mp4` | PS §10 J7, V3 §21.2 |
| D-155 | **Preview statistics reach the tests through the page** (and, in development and test builds only, `data-caption-text`, `data-caption-colors` and `data-caption-position` mirror the visible chunk for the editor's tests; stripped from production like the dev hooks of section 20.4). `previewPlay` resolves with `PreviewStats { frames, late, maxDriftMs, downscaled }`; `control-preview.ts` stores the last one in `preview-store` (`lastStats`); `PreviewPlayer` writes it as `data-frames`, `data-late`, `data-max-drift-ms` and `data-downscaled` on its root element. Nothing is sent anywhere | `preview-sync.spec.ts` must read the drift (TS §20 acceptance) without a test hook in a worker |
| D-156 | **The long-task check.** `pipeline-preview.spec.ts` records `PerformanceObserver("longtask")` entries on the page from the drop to `ready`, and asserts that none exceeds 100 ms, excluding the first `WebAssembly.instantiate` of each bundle on the main thread if any (TS §13.5, assumption M2.4). A failure is fixed by moving the work behind a worker method (TS §13.5 contingency), not by raising the limit | V3 §21.2 assigned it to V4 |

### 2.7 Tests, dependencies, copy

| # | Decision | Why |
|---|---|---|
| D-157 | **`insta`** (dev) in `offcut-scene` and `offcut-plan`, with the `json` feature; snapshots under each crate's `tests/snapshots/`, committed; CI runs with `INSTA_UPDATE=no` and `CI=true` so an unreviewed snapshot fails. **`png`** (dev) in `offcut-render` for the goldens | V2 §25.2 named `insta` for V4; `png` is the smallest crate that reads and writes the golden files |
| D-158 | **`layout_safe_area.rs` builds its scenes from the labeled set.** It reads the 31 files of `fixtures/labeled/`, synthesizes word times as `precision_recall.rs` does (V3 §19.12), detects with `offcut-detect` (a dev-dependency of `offcut-scene`), builds the rules plan, validates it and builds the scene in each of the three styles at 1080 wide, then checks every frame (step 33 ms) for the safe area and the watermark rule of D-143 | TS §26 names the labeled files as its input; V3 §21.3 promised them |
| D-159 | **Copy.** The keys of section 17.5, written in the tone of D-177 and D-190. **Error copy** (`title`, `body`, `action`) for every code a person can meet in V4 leaves `COPY_PENDING`: `E_STORAGE_QUOTA`, `E_STORAGE_IO`, `E_DECODE_AUDIO`, `E_DECODE_VIDEO`, `E_ASR_RUNTIME`, `E_ASR_OOM`, `E_DSP`, `E_GPU_INIT`, `E_GPU_LOST`, `E_ENCODE_VIDEO`, `E_ENCODE_AUDIO`, `E_MUX`, `E_WORKER_CRASH`, `E_INTERNAL`. The `.persistent` variants stay V7's; the auth, billing and network codes stay pending until V6 | `ErrorPanel` must say something specific for every failure it can show (PS §9.4 rule, TS §13.4); `check-copy-codes.mjs` enforces it |
| D-160 | **The V2 and V3 cases V4 rewrites**, by name. `pipeline-preview.spec.ts`: "Preview plays" (the canvas now also carries the `data-*` statistics; the assertion of D-155 is added), "Events" (`setScene` takes a plan: the `pipeline_done` counts are unchanged). `export-creator.spec.ts`: none. `offcut-scene`'s inline tests in `lib.rs`, `captions.rs`, `framing.rs` and `number_reveal.rs` change their calls to the V4 contract (plan, map, theme, fonts, `OutMs`); their expected values do not change, except `framing.rs`'s "any offset" case, which now asserts the offset (V2's stand-in is gone). No other V2 or V3 expectation changes | V3's rule: a changed expectation is named, never silent |
| D-161 | **Stage timing.** The plan and the assembly are measured inside `stage_timing detect_scene` (from `openSession` to the first `setScene` resolving, `rulesPlan` and `assemble` included). No `PipelineStage` is added in V4: `Plan` arrives with V6.1 (TS §10.6) | `PipelineStage` is an analytics enum; adding a member is V6.1's, when the stage exists |
| D-162 | **Bench.** `bench/device-bench.ts` is unchanged in its stages and file name (V3 §21.1). S17 runs it once on the portrait clip so that `detect_scene` (now with the plan and the assembly) and `render_encode` (now with the V4 look) are compared with V3's file | The bench shape is frozen; the reading is what V4 owes |

### 2.8 What V4 builds for V6.x without a user (the complete list)

| # | Decision | Why |
|---|---|---|
| D-163 | **The complete list of V4 code that nothing in V4 needs**, each with the release that is its first user: `EditPlan` fields `cuts`, `chapters`, `extras`, `music` and their types (V6.1-V6.3, D-119); `PlanSource::Model` (V6.1); `QuantityRef::Words` and the word-pointer arms of `ItemParams` with their validation (V6.1; V4 tests them with hand-written plans); `prompt.rs` (V6.1); `TimeMap::joins`, `SrcPos::Held` (V6.2, V6.6); `assemble`'s `speech` argument (V6.3); `DrawCmd::Image`, `Video.mask`, `VideoSlot` as an enum (V6.7, V6.5, P2); `MotionUnit::Glyph` (V6.5); `FontRole::Body` (V6.7: no V4 style uses the body font); the cuts section of `ReviewPanel` (V6.2). Nothing else is reserved | TS §40.9 says nothing beyond the seams is reserved. Writing the list down makes the exception explicit and keeps it from growing |
| D-164 | **No V6.x behaviour leaks into V4.** No network call is added. No catalog kind beyond the four. No cut is applied, no music mixed, no image drawn. `validate` in V4 drops every non-item field (D-121), so a hand-made plan with a cut cannot shorten an export | The seams must not change what V4 does; they only remove what V6.x would otherwise have to rewrite |
| D-165 | **The plan crate's `validate` takes `PlanConfig`**, holding the per-kind limits of V-4 and V-6 (`max_per_clip`, `min_gap`, label lengths), with `PlanConfig::default()` read from the catalog. V4's limits for the four kinds: NumberReveal 8 per clip, min gap 1,500 ms; ListReveal 3, 10,000 ms; FromTo 4, 3,000 ms; KeywordPop 12, 800 ms; labels 32 characters, list headers 24 (all (assumption, V6.1, E-11)). **The rules plan passes V-6 untouched in practice**: the detector's `resolve` already removes overlaps (TS §17.5), and `validate_rules_plans.rs` asserts that no rules plan of the labeled set loses an item to V-6 | One validator for both producers (TS §40.4); a limit that cut detector output would change the V3 event counts silently |
| D-166 | **Scene build budget.** `build_scene` at most 1 s and `frame_at` on average at most 2 ms for the 60 s portrait clip (TS §19 acceptance), now including validation and the time map; measured natively in release (`scene_timing`, an `#[ignore]` test) and in the browser through `stage_timing detect_scene`. Over budget: profile before S12; the usual cause is shaping per frame, which the layout-once rule of V2 forbids | TS §19, §30 |

### 2.9 Caption intelligence: the output quality (founder's brief of 2026-10-22, part A)

| # | Decision | Why |
|---|---|---|
| D-167 | **Smart line breaks.** `captions::chunk` keeps its hard breaks (a sentence end; a pause of `CHUNK_GAP_MS` or more; `max_words`; the line and character limits) and, between two hard breaks, chooses the chunk and line boundaries with the least total cost, by dynamic programming over the words (at most 40 words between hard breaks: TS §17.2's split, so the table is small). Costs, all in `readability.rs` [ONLY] (assumption, E-2): a break after `,` `;` `:` `—` costs 0; a break before a clause word (`CLAUSE_STARTERS`: and, but, so, because, which, that, when, if, or, then, while, until, …) costs 1; a plain break costs 3; a break **after a binder** (`BINDERS`: articles, possessives, prepositions, "to", auxiliaries and modals, "not", number words that modify the next word) costs 12, so "the / product" or "to / build" almost never happens; a chunk of one word costs 8 unless a pause of at least 250 ms stands on both sides of it (a deliberate one-word beat stays alone); within a chunk, two lines whose lengths differ by more than a third cost their difference in characters (balanced lines, no orphan word on line two). A number is one caption word already (TS §17.2) and is never split. The result is deterministic (INV-7) | "Smart line breaks that are easy to read on a phone." Character-count chunking breaks phrases in half; a phrase-aware cost is what professional subtitling does, and it needs no model |
| D-168 | **Reading time.** A chunk is shown at least `MIN_CHUNK_MS = 700 ms`: when its words are spoken faster, its show end is extended into the silence after it, up to the next chunk's lead (never overlapping it, TS §19.4) and never past the next word by more than `CHUNK_TAIL_MS`. When there is no silence to extend into, the chunk is shorter than 700 ms and nothing else changes: no word is moved (INV-5) | A caption that flashes is unreadable; the floor uses silence the speaker already left, so timing stays truthful |
| D-169 | **Emphasis in captions, three layers**, drawn in this precedence: (1) the **spoken word**, through the style's active treatment; (2) a **stressed word** (a KeywordPop item: prosody and lexicon, V3) pops in `accent_alt`; (3) a **number** (a caption word that shows a quantity's display) is drawn in `accent` when `highlight_numbers` is on (default on, a switch in the Style tab, D-186). No other word is coloured: emphasis that is everywhere is nowhere | "Highlights and animations that emphasize the right words." The three signals V4 has are real (timing, prosody, parsed numbers); the AI plan of V6.1 adds its own picks as items, through the same layers |
| D-170 | **The default style is the premium one.** `Clean` (the default, `StyleId::Clean`) is redesigned for content creators in general: Inter 800 (a fifth font instance, `FontKey(4)`, appended to `FontSet::embedded()`, D-132's rule), 72 lp, as spoken, 16 characters x 2 lines, 4 words, white with a 5 lp dark stroke and a hard drop shadow (the run drawn again under itself, offset 0 x 4 lp, black at 45%); the **active word sits on a rounded pill** (radius 14 lp, padding 12 x 4 lp) in `accent` (`#FFD60A`) with the word in `#111114`; the pill **slides** from word to word over 90 ms ease-out and grows from 0.9 to 1.0 as it arrives. `Bold`: Inter 900, upper case, 84 lp, 3 words, the active word in `accent`, 10 lp stroke, each word pops 1.0 → 1.12 → 1.0 over 140 ms back-out. `Tech` keeps its ID and its look (JetBrains Mono, dark panel, green active word, block cursor) and is named **"Mono"** in the interface. `ActiveTreatment` gains `Pill`; `StyleSpec` gains `shadow: Option<ShadowSpec { offset: (f32, f32), role: Role, opacity: f32 }>`. Supersedes D-139 for Clean; all values (assumption, E-2) | "A great default style immediately." The pill is the look creators recognise from the best caption tools; a creator who never opens the Style tab must still get it |
| D-171 | **Caption placement the user can drag.** `EditState` gains `caption_position: CaptionPlacement` (a checked `f32` in `0.0..=1.0`, like `CropOffset`; default `1.0`), the vertical place of the caption block's centre between `CAPTION_TOP = SAFE.top + 140` (0.0) and `CAPTION_BOTTOM = SAFE.bottom - 140` (1.0) lp, so the default reproduces V3's zone (centre 1360). Three snap points: Top 0.15, Middle 0.55, Bottom 1.0. **The event zone follows the captions**: when the caption centre is at or below the canvas middle (960 lp), the event zone ends 40 lp above the caption block (the default gives V3's 860-1180); above the middle, it starts 40 lp below it. The watermark's band is below the lowest caption position at every placement (D-143's test runs at the three snap points). Automatic placement around a face is P1 (face tracking): the drag and the snaps are how a creator keeps captions off a face or a product in V4. **REOPENED V1 CONTRACT (`EditState`)**: two fields, `caption_position` and `highlight_numbers` (D-169), both with serde defaults so the `edits` records of V3 still read | "Consistent positioning … without covering faces or important content (user should be able to drag)." One number per clip is the smallest control that does it |
| D-172 | **The preview is the export, by construction and by test.** One scene, one renderer, layout in logical pixels scaled to the output, no font hinting (Vello draws unhinted outlines), so the 540 x 960 preview and the 720 and 1080 exports differ only by scale. `tests/parity.rs`: for six scenes, the display lists at 540, 720 and 1080 are equal after scaling, every coordinate within 0.01 lp; `golden_frames.rs`'s parity case renders the three styles at 1080 and at 540 and compares the 1080 image box-filtered to 540 with the 540 image at SSIM ≥ 0.99. The one deliberate difference, the Free watermark (INV-9), is announced by the page, not hidden: the preview shows a "Free exports include a small watermark" badge next to the player (page chrome, not drawn into frames) when the user is not on Creator. V5 adds an exported frame against a preview frame (section 22.2) | "Final exports that look exactly like the preview." Saying it is not enough; the two tests make it a property of the code |
| D-173 | **Edit to preview within 150 ms** (TS §13.5, assumption M1.5), measured from the commit of a word edit to the first frame drawn with it, on the 60 s portrait clip on D1. Two mechanisms: (1) `layout.rs` keeps, per thread, a cache of shaped runs (2,048 entries, least recently used) keyed by text, `FontKey` and size bits; `scene_cache.rs` asserts output identical with the cache cold, warm and disabled (INV-7 holds); (2) after `setScene` the worker **redraws the frame at the current position** when the preview is paused, so a correction is on screen without pressing play | "Click a word to correct it; see the correction reflected immediately." A full scene build re-shapes every word; with the cache only the edited chunk is shaped |
| D-174 | **Hit testing.** `Scene::hit(t: OutMs, x: f32, y: f32) -> Option<Hit>` in logical pixels, `Hit::Word(WordIdx)` for a caption word drawn at `t` (its glyph box plus 8 lp), `Hit::Captions` for the caption block's box, `Hit::Video` otherwise. Pure, from the chunk layout already built. Exposed as `RenderWorkerApi.hitTest({ at, x, y })` with `x`, `y` as fractions of the canvas | Click a caption word on the preview to edit it, drag the caption block, drag the video: three gestures, one function |
| D-175 | **"Fix it everywhere."** When a word is edited from text `a` to `b`, and other words whose **original** text equals `a` (case-folded, whole word) are not edited yet, a toast offers "Change all N"; accepting adds every one of them to `word_edits` in one change (one undo step, D-184), and the events are recomputed with `detect` over the whole transcript with the new edits (the render bundle's `detect` takes edits since V3) | Names, brands and jargon are what the recognizer gets wrong, and they repeat; fixing "Jhon" once should fix it in all six places |
| D-176 | **Suggestions, honestly scoped.** In V4 the product suggests: the visuals (rules plan items, shown as "Suggested visuals" with a switch each), "Change all N" (D-175), the premium default style, highlighted numbers. It does **not** flag words as "maybe mis-heard": the recognizer returns no per-word confidence (TE-2, `Confidence(1.0)` everywhere), and a flag without a signal would be noise. Recorded for V9 (E-10) and V6.1 (the model may mark doubtful words; a catalog row) | "Smart suggestions instead of making users edit every detail", without inventing a signal V4 does not have |

### 2.10 Editor experience and design (founder's brief of 2026-10-22, part B)

| # | Decision | Why |
|---|---|---|
| D-177 | **Audience: content creators in general**, not only technical creators (the founder's decision of 2026-10-22; copied back to PS §3, §4, §6 at S16, technical creators remaining the first distribution channel). Consequences for V4: no jargon anywhere a person reads. The interface says "Visuals" (not events or plan items), "Captions", "Style", "Framing", "Export"; visual kinds are "Big number", "List", "Before → after", "Emphasis"; styles "Clean", "Bold", "Mono". Never "LUFS", "detector", "scene", "plan", "WebGPU" outside the settings page's technical note | A creator who is not technical must understand every word on screen without help |
| D-178 | **The editor's layout** (desktop, from 1024 px wide; the capability check already refuses phones). **Top bar** (56 px): product mark; clip length; save state ("Saved on this device", D-184); undo and redo; **Export** as the one primary button. **Stage** (centre): the preview in a 9:16 frame with rounded corners at the largest size that fits (at most 960 px tall), on a dark backdrop, with the transport under it: play/pause, current time / length, scrub bar. **Side panel** (right, 360 px; 320 px below 1280 px wide): four tabs, **Captions · Style · Visuals · Framing**, the selected one remembered per session. Processing shows the same frame with the steps inside it (D-188), so the page does not jump when the preview arrives. Nothing scrolls horizontally; the side panel scrolls on its own | Creators judge an editor in seconds; the industry-standard layout (big preview, one panel of controls, one primary action) needs no learning |
| D-179 | **Visual design system** in `ui/styles/tokens.css` (`[ONLY]` design values): the editor is **dark** (`--bg #0E0E11`, `--surface-1 #16161B`, `--surface-2 #1D1D23`, `--border #2A2A32`, `--text #F4F4F6`, `--text-2 #A3A3AD`, `--brand #8B5CF6`, `--brand-hover #7C4DEE`, `--ok #34D399`, `--warn #FBBF24`, `--danger #F87171`); the landing page keeps following the system's light or dark (unchanged). Type scale 12/13/14/16/20/24/32 px, weights 400/500/600/700; spacing on a 4 px grid (4, 8, 12, 16, 24, 32, 48); radii 6 (controls), 10 (cards), 16 (panels, the preview frame); two elevations (a 1 px border and a soft shadow); motion `--fast 120ms`, `--base 200ms`, `ease-out`, all non-essential motion off under `prefers-reduced-motion`; focus ring 2 px `--brand` with 2 px offset on every interactive element; hit targets at least 32 x 32 px; text and controls at WCAG AA contrast or better. All (assumption, design review D-191) | "Clean, premium and user friendly." A system of tokens is what makes 30 components look like one product; a dark editor makes the video the brightest thing on screen |
| D-180 | **The interface font is Inter**, self-hosted as one variable WOFF2 (Latin subset), `web/src/ui/assets/inter-latin.woff2`, referenced from `tokens.css` so that Vite emits it as a hashed file under `/assets/` (immutable caching; `vercel.json` rewrites every other non-API path to `index.html`, so a `/fonts/` path would not serve), `font-display: swap`, loaded by the editor route; `system-ui` is the fallback. CSP `font-src 'self'` allows it; TS §36 forbids fonts from a CDN, not from our origin. Its licence (OFL) joins `LICENSES.md`; the app-shell budget (400 kB gzip, TS §13.5) is measured with it at S17, and if the font breaks the budget the shell returns to system fonts. `tokens.css`'s first comment ("system fonts only") is corrected. The captions' fonts are unchanged (embedded in the scene) | The same typeface in the interface and in the captions makes the product feel designed, and Inter is already licensed and in the repository |
| D-181 | **Interface primitives**, in `ui/primitives/`, CSS Modules on the tokens, no component library (TS §36): `Button` (primary, secondary, ghost, danger; sm, md), `IconButton` (always with a tooltip and an accessible name), `Tabs` (roving tab index, arrow keys), `Switch`, `SegmentedControl`, `Slider` (a native range, styled, value shown), `Tooltip` (after 400 ms, on focus too), `Toast` (bottom centre, one action, 5 s, paused on hover), `Spinner`, `Skeleton`, `PanelSection` (title, optional hint, body), `Kbd`. Each is keyboard-operable and labelled from `messages.ts`. **Icons**: `ui/icons.tsx`, about 20 inline SVG paths taken from Lucide (ISC licence, listed in `LICENSES.md`), 20 px, 1.75 stroke, `currentColor` | A small owned set keeps the bundle small and every control consistent |
| D-182 | **The transcript editor** (the Captions tab; this is `CaptionEditor.tsx`, replacing D-146's word buttons): the transcript as flowing text, in blocks that are the caption chunks (each with its start time, clickable to seek). While playing, the spoken word is highlighted and kept in view (a "Follow" switch, on by default, off as soon as the user scrolls). **Click a word**: the preview seeks to it and pauses, and the word becomes an inline field with its text selected; Enter or Tab commits (Tab moves to the next word), Escape cancels, an empty commit hides the word. Edited words carry a dotted underline; hidden words are struck through and keep a "Show" button; a word's menu (⋯ on hover, or the context-menu key) offers "Hide", "Restore original". After a commit, D-175's toast may offer "Change all N" | "Click a word to correct it", the way creators already fix transcripts in the best tools, with the video following the text |
| D-183 | **Direct manipulation on the preview** (only while paused or stopped; the cursor says what a press will do, from `hitTest`, at most once per animation frame): **click a caption word** → the Captions tab opens with that word in its field; **drag the caption block** up or down → its position, snapping within 16 px of Top, Middle and Bottom, a thin guide line showing the snap, committed on release as one change (D-171); **drag the video sideways** on a landscape or square clip → the crop offset, with the 9:16 frame outline drawn as interface chrome during the drag, committed on release (D-142's slider stays in the Framing tab as the accessible way). Interface chrome drawn over the canvas is never part of a frame | Direct manipulation is faster than any panel, and each gesture maps to one value the scene already reads |
| D-184 | **Undo and redo, and autosave.** `state/history.ts` keeps up to 50 `EditState` snapshots of the current clip: every review change pushes one (a replace-all is one); **Ctrl/⌘+Z** and **Shift+Ctrl/⌘+Z** (and Ctrl+Y), and the two top-bar buttons, apply a snapshot through `review-update.ts` (mode `plan` when the word edits differ, else `scene`). The history is cleared with the clip and is not kept across a reload (V7). Every change is already written by `editsRepo.put`; the top bar shows "Saved on this device" (and "Saving…" while a write is pending) | Fear of breaking something is what stops people editing; undo removes it. Autosave already exists and should be visible |
| D-185 | **Keyboard**: Space plays and pauses (unless focus is in a text field); ← and → step one frame when paused, Shift+← and Shift+→ one second; Ctrl/⌘+Z, Shift+Ctrl/⌘+Z undo and redo; Escape closes a field or a menu; `?` opens a shortcuts sheet. One hook, `ui/hooks/use-shortcuts.ts`, owns the map; every shortcut is also a visible control | Fast for the people who edit every day, invisible to the people who do not |
| D-186 | **The Style tab.** Three **style cards**, each showing the clip's current frame drawn in that style by the real renderer: `RenderWorkerApi.styleThumbnails({ at })` renders three 216 x 384 frames with the current edits and returns three `ImageBitmap`s (transferred); they refresh, debounced 300 ms, when the position, the words or the placement change, only while the tab is shown. Each card: the image, the name, one line ("Clean: soft and premium, the spoken word on a highlight"). Below: the **Highlight numbers** switch (D-169). What the card shows is what the export will show (D-172) | Choosing by looking at your own video is faster and more trustworthy than reading style names |
| D-187 | **The Visuals tab**: the suggested visuals in time order, each a row with its kind's icon, its friendly label, its figure ("90 min → 10 min"), its time (click to seek) and a switch; a header "4 of 5 visuals on". With none: the `NO_EVENTS_FOUND` sentence and one line on what Offcut looks for ("numbers, lists and before-and-after comparisons you say out loud"). The **Framing tab**: caption position (segmented Top · Middle · Bottom, plus "drag the captions on the preview" hint) and, for landscape and square clips, the crop slider with "or drag the video" | Each tab has one job and tells the user what to do |
| D-188 | **Processing screen**: inside the preview frame, a checklist of four steps in plain words (Preparing audio · Cleaning your voice · Writing captions · Finding visual moments), the current one animated, done ones ticked; found moments appear as chips under it as they arrive (the feed of V3); the line "Everything runs on your computer. This usually takes a couple of minutes."; Cancel. The model's first-run download uses the same frame and tone (the size, a progress bar, "only the first time") | Waiting is part of the experience; showing real progress in plain words keeps people from leaving |
| D-189 | **Export progress** (V4's part; V5 adds the summary): the Export button turns into a progress pill in the top bar (percent and the stage in words: "Rendering video", "Finishing file"), a determinate bar, time left once 3 s of frames give a rate, and Cancel; done shows a success state with "Download again". Errors through `ErrorPanel` (D-190) | "Reliable exports, clear progress" |
| D-190 | **Errors and recovery, in words people use**: `ErrorPanel` is an inline card where the failure happened (the stage area, the panel, or the export pill), never a modal: a short title, what happened in one sentence, what to do, one primary action (Try again, Reload preview, Undo this change) and "Start over" as secondary. Codes and details never show to users (development builds may expand a "Details" section). Copy written for the 14 codes of D-159 in this tone | "Easy recovery from errors" |
| D-191 | **Design review gates.** (1) **Mid-version review** (step S10A): with the shell, the primitives and the editor layout showing real states, `tests-e2e/screens.spec.ts` (D1 only, not in CI) captures eight screens (empty, model download, processing, ready, editing a word, style tab, error, exporting) at 1280 x 800 and 1920 x 1080 into `web/test-results/screens/` (git-ignored); the founder reviews them and the requested changes are made before the review flows are wired. (2) **Exit review**: the same capture on the finished build, a visual QA list (4 px grid, alignment, hover, focus and pressed states on every control, empty, loading and error states present, reduced motion, no clipped text in the three styles at both widths) and the five goldens | "Good UI/UX design is a must": a design is accepted by looking at it, on the reference widths, before and after the code that depends on it |
| D-192 | **Accessibility**: every control reachable and operable by keyboard with a visible focus ring; names and hints from `messages.ts`; the preview canvas has an accessible name, and the transcript editor is its text alternative; processing steps and toasts announce through one `aria-live="polite"` region; `@axe-core/playwright` (dev dependency, MPL-2.0) runs in `review-edit.spec.ts` on the ready and editing states with zero serious or critical violations | Premium includes everyone; the check keeps it true as V6.x adds controls |
| D-193 | **Interface performance**: no main-thread task over 50 ms during editing interactions (the long-task check of D-156 is extended to a word edit, a style switch and a caption drag); thumbnails, hit tests and scene builds run in the render worker; the transcript of a 90 s clip (about 300 words) renders without virtualisation; every interaction gives feedback within 100 ms (pressed state, spinner after 300 ms) | A premium feel is mostly speed |

---

## 3. Tools, dependencies, environment variables

Only what is new or changed since V3 §3.

### 3.1 Tools

| Tool | Used for | Where | New or changed |
|---|---|---|---|
| A software graphics adapter: WARP (Windows, through DX12) and Mesa `lavapipe` (Linux, through Vulkan; the apt package `mesa-vulkan-drivers`) | `golden_frames.rs`, `render_to_image` (D-134) | D1; the Linux CI runner | New. The Linux job installs the package before `cargo test` (section 19) |
| `cargo insta` (optional, D1 only) | Reviewing snapshot changes | D1 | New, optional; CI does not need it |
| Chrome stable with WebGPU | Every media case, the three new suites, the bench | D1 | Unchanged |
| `ffmpeg`, `ffprobe` 9.0.x | Fixtures, the verifier | D1, both runners | Unchanged |

### 3.2 New dependencies

Pin each at its latest stable release on the day S1 adds it, and record the version in `v4changelog.md`.

| Package | Crate | Kind | Features | Note |
|---|---|---|---|---|
| `insta` | `offcut-scene`, `offcut-plan` | dev | `json` | Snapshots (D-157). Apache-2.0 |
| `png` | `offcut-render` | dev | default | Golden files (D-135). MIT OR Apache-2.0 |
| `wgpu` (the workspace pin) | `offcut-render` | dev | `vulkan`, `dx12` | Native backends for the tests only (D-134). The normal dependency keeps `webgpu` only, so the browser bundle does not grow |
| `offcut-plan` (path) | `offcut-scene`, `offcut-wasm-render` | normal | - | The new crate (section 7) |
| `offcut-text` (path) | `offcut-plan` | normal | - | `tokenize`, `parse_quantity`, `format_quantity` (D-122) |
| `offcut-detect` (path) | `offcut-scene`, `offcut-plan` | dev | - | `layout_safe_area.rs` (D-158), `validate_rules_plans.rs`, `plan_from_rules.rs` |
| `trybuild` (workspace) | `offcut-plan` | dev | - | `ValidPlan` has no public constructor (INV-21) |
| `@axe-core/playwright` | `web` | dev | - | The accessibility check (D-192). MPL-2.0 |
| Inter variable WOFF2, Latin subset | `web/src/ui/assets/` | asset | - | The interface font (D-180). OFL-1.1, the same Inter release as the scene's TTF |

No runtime web dependency is added.

**Purity check.** `offcut-plan` is a pure crate: `scripts/check-file-tree.mjs`'s `PURE_CRATES` list gains it (V2 §22.4: no `web-sys`, `js-sys`, `wasm-bindgen`, `getrandom`, `rand`, `wgpu`). The native `wgpu` features are dev-only and `cargo tree -p offcut-render -e normal --target wasm32-unknown-unknown` must show neither `vulkan` nor `dx12` crates; S1 records the tree.

### 3.3 Accounts

Nothing new. The founder reviews the captured screens at S10A and at the exit (D-191). The founder opens a draft pull request `v4-build` → `main` (optional, for CI on every push, as in V3); at the exit, reads five golden frames and says whether each looks as intended (section 23), then merges and tags.

### 3.4 Environment variables

| Variable | Where | Value | New or changed |
|---|---|---|---|
| `OFFCUT_GOLDEN_UPDATE` | `golden_frames.rs` | `1` writes the PNGs instead of comparing | New; never set in CI |
| `INSTA_UPDATE` | `cargo test` | `no` in CI (D-157) | New in `ci.yml` |
| `OFFCUT_FIXTURES_DIR`, `OFFCUT_REFERENCE_CLIP`, `BENCH_CLIP`, `BENCH_DEVICE`, `CORPUS_MODE`, `OFFCUT_CORPUS_DIR` | As in V3 | Unchanged | - |

No secret is added.

---

## 4. V4 file tree

Marks: **NEW** = created in V4. **F** = final in V4 (later versions add rows or arms only where section 22 says). **P** = partial; section 22 says what is added later. **CHANGED** = a V1-V3 file V4 edits, with what changes. **SPLIT** = a file that gives part of its body to a new file with no behaviour change (D-150). Files not listed are untouched. Every path is in the TS §5 tree after S1, which adds the new files below to it in the same commit.

```text
<repo root>/
├── Cargo.toml                          CHANGED  member + offcut-plan; [workspace.dependencies] insta, png, offcut-plan
├── package.json                        CHANGED  script "screens" (the design-review capture, D-191)
├── deny.toml                           CHANGED  licence entries insta and png need, if absent
├── .github/workflows/
│   ├── ci.yml                          CHANGED  Linux job: mesa-vulkan-drivers; INSTA_UPDATE=no; golden test runs (section 19)
│   └── e2e-media.yml                   CHANGED  the three new suites join; @no-gpu cases of cancel.spec.ts run
├── crates/
│   ├── offcut-types/
│   │   ├── src/lib.rs                  CHANGED  `mod timemap; mod plan;` and re-exports
│   │   ├── src/units.rs                CHANGED  OutMs, OutSpan; FrameIdx::to_out_ms replaces to_time_ms (D-125)
│   │   ├── src/timemap.rs              NEW P    [ONLY] TimeMap, Segment, SrcPos; identity only (V6.2: from_segments)
│   │   ├── src/plan.rs                 NEW F    the wire format, PLAN_FORMAT = 1 (D-119, D-120)
│   │   ├── src/events.rs               CHANGED  doc comment: EventKind is the plan item kind (D-118); no variant added
│   │   ├── src/edit.rs                 CHANGED  REOPENED: caption_position (CaptionPlacement), highlight_numbers, serde defaults (D-169, D-171)
│   │   └── src/summary.rs              unchanged  (fields unchanged; V6.2 adds two)
│   ├── offcut-api-types/
│   │   └── src/analytics.rs            CHANGED  ReviewActionKind + caption_position, highlight_numbers, replace_all, undo, redo (section 13)
│   ├── offcut-plan/                    NEW
│   │   ├── Cargo.toml                  NEW F
│   │   ├── src/lib.rs                  NEW P    module list; re-exports; PlanError
│   │   ├── src/catalog.rs              NEW P    [ONLY] CATALOG_VERSION, ItemSpec, ParamSpec, the four kinds (V6.x: rows)
│   │   ├── src/prompt.rs               NEW P    system_prompt, output_schema, user_message (V6.1: first caller)
│   │   ├── src/validate.rs             NEW P    [ONLY constructor of ValidPlan] V-1..V-6; V-7, V-8 drop all (V6.2, V6.6: rules)
│   │   ├── src/resolve.rs              NEW F    QuantityRef → Quantity over effective tokens; unit inheritance for FromTo
│   │   ├── src/from_rules.rs           NEW P    DetectedEvent[] → EditPlan (V6.4 may map onto new kinds)
│   │   ├── src/config.rs               NEW P    [ONLY] PlanConfig and its defaults (D-165)
│   │   ├── tests/catalog_snapshot.rs   NEW F
│   │   ├── tests/validate.rs           NEW P    one case per rule (V6.x: cut, chapter, extras cases)
│   │   ├── tests/plan_from_rules.rs    NEW F
│   │   ├── tests/validate_rules_plans.rs  NEW F  no rules plan of the labeled set loses an item (D-165)
│   │   ├── tests/ui.rs, tests/ui/valid_plan_private.rs  NEW F  trybuild: ValidPlan cannot be built outside the crate
│   │   └── tests/snapshots/            NEW      insta files of the prompt and the schema
│   ├── offcut-scene/
│   │   ├── Cargo.toml                  CHANGED  + offcut-plan; dev: insta, offcut-detect
│   │   ├── src/lib.rs                  CHANGED  SceneInput v1.2; Scene on OutMs; source_at; crop() removed; summary moved
│   │   ├── src/display_list.rs         CHANGED  FontKey for FontId; Video, Image; CropRect, VideoSlot, ImageKey (D-130, D-132)
│   │   ├── src/fonts.rs                CHANGED  FontSet registry, embedded() (D-132); P (V6.7: with_user)
│   │   ├── src/layout.rs               CHANGED  shapes through a FontSet; normalized_coords per key
│   │   ├── src/theme.rs                NEW P    [ONLY colour values] Theme, Role, FontRole, default_for (V6.7: brand)
│   │   ├── src/styles.rs               CHANGED  [ONLY] Clean, Bold, Tech as roles + CaptionMotion (D-136, D-137, D-139); P (V6.5 presets)
│   │   ├── src/motion.rs               NEW P    CaptionMotion, MotionUnit, MotionPreset, ActiveTreatment (V6.5: arms)
│   │   ├── src/safe_area.rs            CHANGED  [ONLY] ZoneId, zone(); WATERMARK_ANCHOR (D-138); P (rows)
│   │   ├── src/captions.rs             CHANGED  OutMs; theme roles; words, chunks and lines; the break search of D-167
│   │   ├── src/readability.rs          NEW P    [ONLY] break costs, CLAUSE_STARTERS, BINDERS, MIN_CHUNK_MS (D-167, D-168); V6.x: other languages
│   │   ├── src/placement.rs            NEW F    [ONLY] caption block and event zone from CaptionPlacement (D-171)
│   │   ├── src/hit.rs                  NEW F    Scene::hit (D-174)
│   │   ├── src/captions_draw.rs        NEW P    drawing a chunk: motion, Tech panel and cursor, pops (V6.5: presets)
│   │   ├── src/anim.rs                 CHANGED  keys in OutMs
│   │   ├── src/easing.rs               unchanged
│   │   ├── src/framing.rs              CHANGED  [ONLY] the offset is used (D-131); F
│   │   ├── src/video.rs                NEW P    the per-frame Video command: crop, placement (V6.1 punch-in, V6.2 jump zoom, V6.5 mask)
│   │   ├── src/watermark.rs            NEW F    D-143
│   │   ├── src/summary.rs              NEW P    change_summary (D-144); V6.2 adds cut fields
│   │   ├── src/events/mod.rs           CHANGED  layout(ValidItem), four arms; P (V6.x arms)
│   │   ├── src/events/number_reveal.rs CHANGED  OutSpan; theme roles; F
│   │   ├── src/events/list_reveal.rs   NEW F    D-140
│   │   ├── src/events/from_to.rs       NEW F    D-141
│   │   ├── src/events/keyword_pop.rs   NEW F    the in-caption scale (TS §19.4); the track the captions read
│   │   ├── tests/display_list_snapshots.rs  NEW P  (V6.x: one case per new kind)
│   │   ├── tests/layout_safe_area.rs   NEW P    (V6.x kinds join automatically through the plan)
│   │   ├── tests/framing.rs            NEW F
│   │   ├── tests/determinism.rs        NEW F
│   │   ├── tests/styles_use_theme.rs   NEW F    D-136
│   │   ├── tests/timemap_identity.rs   NEW F    S-1 (lives here: it exercises the scene on the identity map)
│   │   ├── tests/catalog_complete.rs   NEW P    INV-23: catalog kinds and drawn kinds are the same set (V6.x kinds join)
│   │   ├── tests/line_breaks.rs        NEW F    the table of D-167, written before the algorithm
│   │   ├── tests/parity.rs             NEW F    D-172
│   │   ├── tests/placement.rs          NEW F    D-171 at the snap points and the extremes
│   │   ├── tests/scene_cache.rs        NEW F    D-173: identical output, cache cold, warm, off
│   │   ├── tests/hit.rs                NEW F    D-174
│   │   ├── tests/v3_reference.rs       TEMPORARY  S2 to S6 only (D-117), deleted at S7 with its JSON files
│   │   └── tests/snapshots/            NEW      insta files
│   ├── offcut-render/
│   │   ├── Cargo.toml                  CHANGED  dev: png; dev wgpu features vulkan, dx12 (D-134)
│   │   ├── src/lib.rs                  CHANGED  render(sources, list, fonts); Unsupported; new_headless, render_to_image (native)
│   │   ├── src/gpu.rs                  CHANGED  a headless target (texture + readback) beside the canvas
│   │   ├── src/video_pass.rs           CHANGED  src crop and dst rectangle from the Video command; an image source (native)
│   │   ├── src/vello_backend.rs        CHANGED  FontKey through the FontSet; skips Video commands
│   │   ├── src/composite.rs            CHANGED  list order; the V4 support rule (D-133); P (V6.5: layers, masks)
│   │   ├── tests/golden_frames.rs      NEW P    16 images (D-135); V6.x adds one per new kind
│   │   └── tests/common/ssim.rs        NEW F    the similarity measure
│   ├── offcut-dsp/
│   │   ├── src/lib.rs                  CHANGED  `pub mod program; pub mod silence;`
│   │   ├── src/program.rs              NEW P    [ONLY place that changes duration] assemble, identity path (V6.2 splices, V6.3 mix)
│   │   ├── src/silence.rs              NEW P    silence_profile (V6.2 is its first reader)
│   │   └── tests/program_identity.rs   NEW F    S-6
│   ├── offcut-wasm-core/
│   │   └── src/audio_api.rs            CHANGED  + assemble (D-127)
│   └── offcut-wasm-render/
│       ├── Cargo.toml                  CHANGED  + offcut-plan
│       ├── src/session.rs              SPLIT    keeps the session, the source, frames, the decoder calls, resize
│       ├── src/scene_api.rs            NEW P    set_scene (validate + build), source_at, summary; rules_plan (D-150)
│       └── src/detect_api.rs           unchanged (redetect_sentence is called from the worker now)
├── web/
│   ├── eslint.config.js                CHANGED  MINTING: control-preview.ts mints OutMs (D-126); use-case edges: the four edit use-cases → review-update.ts, review-update.ts → control-preview.ts (cancel-job.ts is an allowed target already)
│   ├── playwright.config.ts            CHANGED  MEDIA_SUITES + preview-sync, review-edit, cancel; a `screens` project (D1 only, D-191)
│   ├── src/gen/domain.ts               generated  OutMs, OutSpan, TimeMap, EditPlan and its types (section 12)
│   ├── src/copy/messages.ts            CHANGED  review, preview seek, cancel, noEventsFound, 14 error entries (D-159)
│   ├── src/workers/protocol.ts         CHANGED  REOPENED (D-124, D-126, D-127, D-128, D-174, D-186): rulesPlan, setScene(plan) → SceneReport, previewSeek(OutMs), ClockSync(OutMs), hitTest, styleThumbnails, assemble, release, PreviewStats
│   ├── src/workers/rpc.ts              SPLIT    page side only; + throttle-aware progress; F
│   ├── src/workers/rpc-worker.ts       NEW F    serveWorker side (D-150, D-151)
│   ├── src/workers/rpc.test.ts         NEW F
│   ├── src/workers/pool.ts             CHANGED  cancel, restartWorker, crash marking (D-152, D-153); P (V7: budget)
│   ├── src/workers/render.worker.ts    SPLIT    session state and the handler table; rulesPlan, redetectSentence, previewSeek handlers
│   ├── src/workers/render/export-job.ts   NEW F  the export handlers moved out (D-150)
│   ├── src/workers/render/preview-job.ts  NEW P  play, pause, seek, clock, resize rule (D-148, D-149)
│   ├── src/workers/render/preview-loop.ts CHANGED  OutMs; source_at; cost measure; stats.downscaled
│   ├── src/workers/render/video-source.ts SPLIT  queue and API; prefetch; seek restart (D-148)
│   ├── src/workers/render/frame-decoder.ts NEW F  decoder management moved out (D-150)
│   ├── src/workers/render/export-loop.ts  CHANGED  render_frame takes OutMs (same number, D-129)
│   ├── src/workers/audio.worker.ts     CHANGED  assemble, release (D-127, D-128)
│   ├── src/wasm/load-core.ts           CHANGED  CoreApi gains assemble
│   ├── src/wasm/load-render.ts         CHANGED  RenderApi: rulesPlan; RenderSession: set_scene(plan) → report, source_at, resize
│   ├── src/state/clip-store.ts         CHANGED  plan, prog48 (in place of out48), events setter, edit actions, editing queue
│   ├── src/state/history.ts            NEW F    undo and redo of EditState, 50 steps (D-184)
│   ├── src/state/history.test.ts       NEW F
│   ├── src/state/ui-store.ts           NEW P    selected tab, follow switch, editing word, toasts, save state (V6.x: new tabs)
│   ├── src/state/preview-store.ts      CHANGED  position, lastStats; seek actions fire
│   ├── src/state/machines/clip-machine.test.ts  NEW F
│   ├── src/state/machines/model-machine.test.ts NEW F
│   ├── src/usecases/run-pipeline.ts    CHANGED  rulesPlan, assemble, cancel signal, client_error, retry; release on every early end
│   ├── src/usecases/run-pipeline.test.ts NEW P  fake workers (V6.1: plan fallbacks)
│   ├── src/usecases/import-clip.ts     CHANGED  the cancel signal is honoured
│   ├── src/usecases/control-preview.ts CHANGED  seek; OutMs; prog48; lastStats; reloadPreview (D-153)
│   ├── src/usecases/start-export.ts    CHANGED  cancel (the export machine's cancel arm fires)
│   ├── src/usecases/edit-word.ts       NEW F    D-146
│   ├── src/usecases/toggle-event.ts    NEW F    toggles a plan item by EventId
│   ├── src/usecases/set-style.ts       NEW F
│   ├── src/usecases/set-crop-offset.ts NEW F
│   ├── src/usecases/review-update.ts   NEW F    the shared update rule of D-147 (pause, queue, setScene, restore, undo)
│   ├── src/usecases/cancel-job.ts      NEW F    D-152
│   ├── src/usecases/set-caption-position.ts NEW F D-171
│   ├── src/usecases/set-highlight-numbers.ts NEW F D-169
│   ├── src/usecases/replace-all.ts     NEW F    D-175
│   ├── src/usecases/undo-redo.ts       NEW F    D-184
│   ├── src/usecases/preview-interact.ts NEW F   hover and click hit tests, drag commits (D-183)
│   ├── src/usecases/style-thumbnails.ts NEW F   D-186
│   ├── src/ui/assets/inter-latin.woff2 NEW      the interface font (D-180), emitted hashed under /assets/
│   ├── LICENSES.md                     NEW      Inter (OFL-1.1), Lucide icons (ISC)
│   ├── src/ui/pages/EditorPage.tsx     CHANGED  EditorLayout by clip state (section 17.2); preloads the font
│   ├── src/ui/layout/EditorLayout.tsx  NEW P    top bar, stage, side panel, live region (V6.x: nothing)
│   ├── src/ui/layout/TopBar.tsx        NEW P    save state, undo, redo, Export / export pill (V6: account menu)
│   ├── src/ui/layout/Stage.tsx         NEW F    the state's card or the player
│   ├── src/ui/layout/SidePanel.tsx     NEW P    the four tabs (V6.x: Cuts, Music, Brand tabs or sections)
│   ├── src/ui/components/PreviewPlayer.tsx    CHANGED  9:16 frame, transport, scrub bar, direct manipulation, watermark badge, data-* statistics
│   ├── src/ui/components/PreviewGestures.tsx  NEW P   the gestures of D-183 and their chrome, split from PreviewPlayer (V4 P113; V6.x: shape, lower third, logo drags)
│   ├── src/ui/components/ProcessingCard.tsx   NEW F   D-188
│   ├── src/ui/components/CaptionEditor.tsx    NEW F   the transcript editor (D-182)
│   ├── src/ui/components/CaptionWordControls.tsx NEW F a word's field and menu, split from CaptionEditor (V4 P115)
│   ├── src/ui/components/StylePanel.tsx       NEW P   cards and Highlight numbers (V6.5: presets)
│   ├── src/ui/components/StylePicker.tsx      NEW P   the three cards (V6.5: preset cards)
│   ├── src/ui/components/PlanItemList.tsx     NEW P   the Visuals tab (V6.x kinds: one label and icon each)
│   ├── src/ui/components/FramingPanel.tsx     NEW F   caption position and crop
│   ├── src/ui/components/CropOffsetControl.tsx NEW F
│   ├── src/ui/components/CutsSection.tsx      NEW P   renders nothing in V4 (V6.2)
│   ├── src/ui/components/ErrorPanel.tsx       NEW P   D-190 (V7: persistent copy, "report this")
│   ├── src/ui/components/ShortcutsSheet.tsx   NEW F   D-185
│   ├── src/ui/components/ToastHost.tsx        NEW F
│   ├── src/ui/components/ReviewPanel.tsx      not created: SidePanel takes its place (TS §5 updated at S1)
│   ├── src/ui/components/ProcessingFeed.tsx   CHANGED  the found-moment chips inside ProcessingCard; its cancel moves to the card
│   ├── src/ui/components/ModelDownloadPanel.tsx CHANGED restyled; cancel button
│   ├── src/ui/components/ExportProgress.tsx   CHANGED  the export pill (D-189)
│   ├── src/ui/components/RejectionPanel.tsx   CHANGED  restyled as a stage card; copy unchanged
│   ├── src/ui/components/DropZone.tsx         CHANGED  restyled as the empty stage
│   ├── src/ui/primitives/*.tsx                NEW F   Button, IconButton, Tabs, Switch, SegmentedControl, Slider, Tooltip, Toast, Spinner, Skeleton, PanelSection, Kbd (D-181)
│   ├── src/ui/icons.tsx                NEW P    about 20 inline SVG icons (V6.x: more)
│   ├── src/ui/hooks/use-shortcuts.ts   NEW F    D-185
│   ├── src/ui/styles/tokens.css        CHANGED  [ONLY design values] D-179; @font-face (D-180)
│   ├── src/ui/styles/editor.module.css, panel.module.css, primitives.module.css, preview.module.css  NEW
│   ├── src/ui/styles/components.module.css    CHANGED  the classes the restyled V2/V3 components keep
│   ├── tests-e2e/helpers/fixtures.ts   CHANGED  previewStats(), seekTo(), editWord(), toggleItem(), setStyle(), dragCrop()
│   ├── tests-e2e/pipeline-preview.spec.ts     CHANGED  zero-event case, long-task case, statistics (D-154, D-156, D-160); F
│   ├── tests-e2e/preview-sync.spec.ts  NEW F
│   ├── tests-e2e/review-edit.spec.ts   NEW P    (V6.x: one case per new control); axe check (D-192)
│   ├── tests-e2e/screens.spec.ts       NEW F    D1-only screen capture for the design reviews (D-191)
│   └── tests-e2e/cancel.spec.ts        NEW F
├── fixtures/
│   └── golden/*.png                    NEW      16 images (D-135)
├── scripts/
│   ├── check-file-tree.mjs             CHANGED  PURE_CRATES + offcut-plan; the TS §7 graph has 13 crates
│   └── check-copy-codes.mjs            CHANGED  14 codes leave COPY_PENDING (D-159)
└── docs/
    ├── technicalspec.md                CHANGED  §5 tree, §7 graph, §2 row; the copy-back of section 24
    └── v4/
        ├── v4implementation.md         this file
        ├── v4buildguide.md             NEW      order of work, commands, checks
        ├── coding-promptsv4.md         NEW      Prompts 91 onward
        ├── v4changelog.md              exists   Entry 0 (spec v1.2); Entry 1 at S1
        └── experiments.md              NEW      section 21's records
```

**Untouched in V4:** every file under `server/` (the server only rebuilds with the wider analytics enum, section 13), every file of `crates/offcut-api-types/` but `analytics.rs`, `crates/offcut-entitlement/`, `crates/offcut-mp4/`, `crates/offcut-text/`, `crates/offcut-detect/` (the plan reads them, no file changes); `offcut-dsp`'s chain, its stages and its four test files; `offcut-wasm-core/src/{media_api,hash_api,text_api}.rs`; `offcut-wasm-render/src/{lib,profile_api,mux_api,detect_api}.rs`; `web/vercel.json`; `persistence/*` (the repos are called, not changed; `edits-repo.put` already takes the whole `EditState`); `net/*`; `analytics/client.ts`; `models/*` except through `cancel-job.ts`'s abort, which uses the existing signal; `platform/*`; `state/{capability-store,model-store,export-store,blockers}.ts`; `state/machines/{transition,clip-machine,model-machine,export-machine}.ts` (tables unchanged: V4 fires the events V2 already declared); `workers/{media.worker,asr.worker}.ts` and `workers/{media,asr}/*`; `workers/render/{encoders,opfs-sink}.ts`; `verify/*`; `bench/*`; `corpus/*`; `fixtures/gen_fixtures.sh`, `manifest.json`, `labeled/*`, `speech/*`.

Three untouched files are worth naming. `state/machines/clip-machine.ts` already has `edit_start`, `edit_done`, `cancel` and `retry`; V4 fires them and writes their tests. `persistence/schema.ts` stays at `DB_VERSION = 1`: V4 stores no plan (D-124). `offcut-detect` gains no line: `redetect_sentence` has been ready since V3.

---

## 5. Build order

Nineteen steps and one review. BP §0 gives V4 4 days (5 to 8 Nov 2026); the build is ahead of that calendar (V3 closed on 2026-10-21), and V4 carries the seven seams, the caption intelligence and the editor's design besides, so the plan takes **thirteen working days**. Do not start a step while the previous step's check fails. Order: contracts, the identity proof, splits, caption intelligence, the look, the design system and a review of it, direct manipulation, editing, robustness, tests, CI, measure, a second review, deploy.

| # | Day | Step | Produces | Check |
|---|---|---|---|---|
| S1 | 1 | Setup and contracts on paper | Branch (D-115); `docs/v4/` files (D-113); TS §5 tree with every NEW file of section 4, TS §7's graph with `offcut-plan`, TS §2's row (D-122); `Cargo.toml` member and workspace dependencies; `deny.toml`; Entry 1 of the changelog with the baseline counts and SHA; `cargo tree` records (section 3.2); the frozen-file list | `pnpm check` green; counts equal to 1A item 4 |
| S2 | 1 | The identity reference | `tests/v3_reference.rs` and its six JSON files written by the **V3** scene code (D-117) | A second run compares equal |
| S3 | 2 | The four splits | `rpc-worker.ts`, `frame-decoder.ts`, `render/export-job.ts`, `render/preview-job.ts`, `scene_api.rs` (D-150); no behaviour change | `pnpm check`, `pnpm test`, `pnpm e2e`, `pnpm e2e:media` (48) unchanged; every file under 400 lines |
| S4 | 2-3 | Types and the plan crate | `units.rs`, `timemap.rs`, `plan.rs`, `edit.rs` (two fields, D-171); `offcut-plan` whole with its five test files; `pnpm gen` | `cargo test -p offcut-types -p offcut-plan`; `check-gen-clean`; prompt and schema snapshots committed |
| S5 | 3-4 | Scene and renderer on the new contract, V3's look | `SceneInput` v1.2, `OutMs`, `FontSet`, `Theme::default_for(Clean)` reproducing V3's colours exactly, zones, `Video` command, `video.rs`, `Renderer::render(sources, list, fonts)`; the binding, `protocol.ts`, `load-render.ts` reopened; `rulesPlan` in the pipeline; `timemap_identity.rs` | `cargo test --workspace` with the V3 inline tests adapted (D-160); in `pnpm dev` the reference clip previews as in V3 |
| S6 | 4 | **The identity proof**; program audio | `v3_reference.rs` against the S2 files (D-117); `program.rs`, `silence.rs`, `program_identity.rs`, `assemble` in the binding and the worker, the page plays `prog48` (D-127) | Both identity tests pass; the 48 V3 media cases green. **If the reference differs beyond the normalisation, stop** |
| S7 | 5 | Native rendering, first goldens | Delete `v3_reference.rs`; `new_headless`, `render_to_image`, `ssim.rs`; two video-only goldens (crop, punch-in), which the look of S9 cannot change; the other fourteen at S9; the Linux job's Mesa line added now so the cross-adapter reading can happen (`v4buildguide.md` Step 7.3) | `cargo test -p offcut-render` on D1 (WARP) and in CI (lavapipe); cross-adapter similarity recorded |
| S8 | 5-6 | **Caption intelligence** | `tests/line_breaks.rs`'s table written and committed **first** (30 sentences, the expected chunks and lines decided by reading, the founder reads ten), then `readability.rs` and the break search (D-167), the reading floor (D-168), the emphasis layers (D-169), `placement.rs` (D-171), `hit.rs` (D-174), the shaping cache (D-173), `parity.rs` (D-172) | `line_breaks.rs`, `placement.rs`, `hit.rs`, `scene_cache.rs`, `parity.rs` pass; no expected break changed after the table's commit without a changelog line |
| S9 | 6-7 | The look | `theme.rs`; Clean (the pill, the shadow), Bold, Mono (D-170); `motion.rs`; `captions_draw.rs`; `list_reveal.rs`, `from_to.rs`, `keyword_pop.rs`; framing offset; `watermark.rs`; `summary.rs`; the remaining goldens and the parity golden | All scene and render tests of section 20.2 pass; the 16 goldens committed, the parity case passing |
| S10 | 7-8 | **Design system and editor shell** | `tokens.css` (D-179), the font (D-180), `ui/primitives/`, `icons.tsx` (D-181), `EditorLayout`, `TopBar`, `Stage`, `SidePanel` with real states and static tab content, `ProcessingCard` (D-188), restyled `DropZone`, `RejectionPanel`, `ModelDownloadPanel`; `screens.spec.ts` | `pnpm check`; every state of section 17.2 reachable in `pnpm dev` |
| S10A | 8 | **Design review 1** (D-191) | The eight screens at two widths captured and reviewed by the founder; requested changes made | The founder's go, recorded in the changelog with the list of changes |
| S11 | 8-9 | Preview and direct manipulation | Seek, `prefetch`, resize rule (D-148, D-149); the transport; the statistics (D-155); `hitTest`, hover cursors, click to edit, caption drag with snaps, video drag (D-183); the watermark badge (D-172); the redraw after `setScene` while paused (D-173) | In `pnpm dev`: scrub shows the frame within 300 ms; dragging the captions to Top moves the visuals below them; a click on a caption word opens it in the Captions tab |
| S12 | 9-10 | Editing | `review-update.ts` and the edit use-cases; `CaptionEditor` (D-182), `replace-all.ts` (D-175), `history.ts` and `undo-redo.ts` (D-184), `use-shortcuts.ts` and `ShortcutsSheet` (D-185), `StylePanel` with live cards (D-186), `PlanItemList`, `FramingPanel` (D-187), `CutsSection`, `ToastHost`; the copy of section 17.5 | In `pnpm dev`: editing "90" to "9" shows "9 minutes" within 150 ms; "Change all" fixes every copy; Ctrl+Z restores each step; the cards show the clip in three styles |
| S13 | 10-11 | Robustness and its interface | Throttle, cancel timeout, crash (D-151 to D-153); `cancel-job.ts`; cancel buttons; `ErrorPanel` (D-190); the export pill (D-189); `client_error`, `job_cancelled`; the 14 error entries; the analytics enum (section 13) | `vitest run` with `rpc.test.ts`; copy check green; cancel and a killed worker behave as D-152, D-153 say |
| S14 | 11 | Unit and machine tests | `rpc.test.ts`, `clip-machine.test.ts`, `model-machine.test.ts`, `run-pipeline.test.ts` | `vitest run` all pass |
| S15 | 11-12 | E2E | `preview-sync.spec.ts`, `review-edit.spec.ts` (with axe, D-192), `cancel.spec.ts`; `pipeline-preview.spec.ts` complete | `pnpm e2e:media` on D1, keyed build, all green (section 20.1) |
| S16 | 12 | CI | `ci.yml` (lavapipe, `INSTA_UPDATE=no`), `e2e-media.yml`, `playwright.config.ts`, the check scripts | The pull request's jobs green; `golden_frames` runs in the Linux log |
| S17 | 12 | Measure | Section 21's readings, edit-to-preview first; the bench; the shell budget with the font (D-180); BP §6.2's decisions | `docs/v4/experiments.md` written; the bench file committed |
| S18 | 13 | Spec copy-back | Section 24's corrections in TS, BP, PS (the audience of D-177 included) | `check-file-tree` green |
| S19 | 13 | **Design review 2**, merge, deploy, tag | The exit capture and visual QA list (D-191), the five goldens; `pnpm e2e:media` all green (D-72); the founder merges; the deployed page checked; tag `v4` | Section 23 ticked |

**What can run in parallel.** S4's crate is pure and can start while S3's splits are reviewed. S7's native renderer needs S5's renderer contract only. S10 (the design system) needs no Rust and can run beside S8 and S9. S11 (direct manipulation) and S12 (editing) touch different files once S10A's review is in. S13 is independent of S11 and S12 except for `EditorPage.tsx`. If client work is blocked, V6's server work still depends only on V1 (BP §0).

**What each step must not do.** No step changes a pixel before S7 (D-116). No interface code past the shell is written before S10A's review. No step adds a `TODO` without a version tag. No step touches a file of section 4's untouched list; the frozen-file diff against the baseline lists only section 4's files. No step adds a network request (D-164).

---

## 6. `offcut-types`: two timelines, the plan's wire format

**Purpose** (TS §10). Unchanged: the shared types, generated to TypeScript. V4 adds program time and the plan's wire format, both pure data with constructors; no behaviour beyond the time map's arithmetic.

### 6.1 `units.rs` (D-125)

```rust
pub struct OutMs(u32);                                   // program time; same derives and rules as TimeMs
pub struct OutSpan { pub start: OutMs, pub end: OutMs }  // invariant start <= end; OutSpan::new -> Option
impl FrameIdx { pub fn to_out_ms(self) -> OutMs; pub fn to_micros(self) -> Micros; }   // n * 1000 / 30 (floor)
impl OutMs   { pub fn to_micros(self) -> Micros; pub fn checked_add(self, d: DurMs) -> Option<OutMs>;
               pub fn duration_since(self, earlier: OutMs) -> Option<DurMs>; }
```

`to_time_ms` is removed; its two callers (`offcut-scene`'s tests, `offcut-wasm-render`'s export frame arithmetic if any) move to `to_out_ms`. `bare_ms_rejected.rs` (trybuild) gains no case: `OutMs` is made by the same macro, so the rule covers it. `OutMs` has no `From<TimeMs>` and `TimeMs` none from `OutMs`; a test asserts that `TimeMs::new(5).get() == OutMs::new(5).get()` is the only way to compare them, through numbers (it compiles only that way).

### 6.2 `timemap.rs` [ONLY] (D-125)

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Segment { Source { src: Span }, Hold { at: TimeMs, dur: DurMs } }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SrcPos { At(TimeMs), Held(TimeMs) }            // not serialized: the binding returns it as a plain object

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
pub struct TimeMap { segments: Vec<Segment> }            // private; deserialization is checked (below)

impl TimeMap {
    pub fn identity(duration: DurMs) -> TimeMap;         // one Source segment [0, duration)
    pub fn is_identity(&self) -> bool;                   // exactly one Source segment starting at 0
    pub fn out_duration(&self) -> DurMs;                 // sum of segment lengths
    pub fn to_out(&self, t: TimeMs) -> Option<OutMs>;    // None inside a removed span or past the end
    pub fn to_src(&self, t: OutMs) -> SrcPos;            // clamped to the last instant
    pub fn out_span(&self, s: Span) -> Option<OutSpan>;  // both ends mapped; None if either is removed
    pub fn joins(&self) -> Vec<OutMs>;                   // program time of each segment boundary after the first
}

pub struct SilenceProfile { pub rms_db_10ms: Vec<f32> }  // the time map's input from V6.2 (TS §40.6); made by offcut-dsp
```

- **Deserialization is validated.** A hand-written `Deserialize` accepts a `segments` list only if it is non-empty, every `Source` span is non-empty and later than the previous one, and no `Hold` has a zero duration; otherwise it fails. In V4 a map that is not the identity is refused by the binding too (D-164): `scene_api::set_scene` returns `E_INTERNAL` `"TimeMapNotIdentity"`.
- **Arithmetic.** Integer milliseconds; no floats. `to_src` for the identity map returns `At(TimeMs(t))`. The end instant: `to_out(duration)` is `Some(out_duration)` (an end of a span maps to the end), so a word ending at the clip's end maps.
- **Tests** (inline): identity round trips for 0, 1, 33, 59,999, 60,000 ms; `out_span` of every span of a 60 s identity map equals its numbers; a deserialization case per refusal; `joins` of the identity is empty.

### 6.3 `plan.rs` (D-119, D-120)

```rust
pub const PLAN_FORMAT: u16 = 1;

pub struct EditPlan { pub format: u16, pub catalog: u32, pub source: PlanSource, pub cuts: Vec<Cut>,
                      pub items: Vec<PlanItem>, pub chapters: Vec<Chapter>, pub extras: TextExtras,
                      pub music: Option<MusicCue> }
pub enum PlanSource { Rules, Model { provider: String, model: String } }    // provider: a ProviderId string (V6.1 decides the enum)
pub enum Cut { Lead { keep: DurMs }, Trail { keep: DurMs }, Gap { after: WordIdx, keep: DurMs },
               KeepGap { after: WordIdx }, Words { range: WordRange, why: CutWhy } }
pub enum CutWhy { Filler, FalseStart }
pub struct PlanItem { pub id: EventId, pub kind: EventKind, pub anchors: WordRange, pub params: ItemParams,
                      pub confidence: Option<Confidence> }
pub enum ItemParams {
    NumberReveal { value: QuantityRef, label: Option<Label> },
    ListReveal { header: Option<Label>, items: Vec<ListPoint> },
    FromTo { from: QuantityRef, to: QuantityRef, label: Option<Label> },
    KeywordPop { word: WordIdx },
    Detected(EventParams),                                  // rules producer only (V-2)
}
pub struct ListPoint { pub at: WordIdx, pub label: Label }
pub struct QuantityRef { pub words: WordRange }            // V4 has one form; resolved by validate (D-121)
pub struct Label(String);                                  // transparent; length and digit rules are validate's
pub struct Chapter { pub starts_at: WordIdx, pub title: Label }
pub struct TextExtras { pub hook: Option<Label>, pub post_caption: Option<String>, pub hashtags: Vec<Label> }
pub enum MusicEnergy { Calm, Steady, Upbeat }
pub struct MusicCue { pub fits: bool, pub energy: MusicEnergy }
```

- **Serde.** Every struct `deny_unknown_fields`; every enum `#[serde(tag = "kind", rename_all = "snake_case")]` except `ItemParams`, which is adjacently tagged (`#[serde(tag = "kind", content = "params", rename_all = "snake_case")]`) so that `Detected(EventParams)` (itself tagged `"kind"`) does not collide; its `kind` repeats `PlanItem.kind` and V-2 checks that the two agree. `Label` is `#[serde(transparent)]` (a JSON string; `ts-rs` writes `string`). `QuantityRef` is a one-field struct, so the JSON reads `{ "words": { "start": 3, "end": 5 } }` and a later form can be added beside `words` without breaking format 1. `TextExtras` defaults to empty (`#[serde(default)]`).
- **No method** beyond `Label::as_str` and `EditPlan::empty_rules()` (format 1, catalog `CATALOG_VERSION` passed in, rules, everything empty). All logic is in `offcut-plan`.
- **Tests** (inline): a round trip of a plan holding every arm; refusal of an unknown field at every level; the exact JSON of a one-item rules plan (a contract with V6.1's server, so it is pinned here as V3 pinned `EventParams`); `ItemParams::Detected` round trips without a tag clash.
- **Generated TypeScript.** Every type above goes to `web/src/gen/domain.ts` (section 12).

### 6.4 `edit.rs` (D-169, D-171; REOPENED V1 CONTRACT)

```rust
pub struct CaptionPlacement(f32);            // 0.0 (top of the allowed band) ..= 1.0 (bottom); new() checks; Deserialize checks
pub struct EditState {
    pub word_edits: BTreeMap<WordIdx, String>,
    pub event_overrides: BTreeMap<EventId, bool>,
    pub style: StyleId,
    pub crop_offset: CropOffset,
    #[serde(default = "CaptionPlacement::bottom")] pub caption_position: CaptionPlacement,   // 1.0
    #[serde(default = "yes")] pub highlight_numbers: bool,                                   // true
}
```

The two defaults make the `edits` records V3 wrote read as the V4 default (bottom, numbers highlighted). Tests: an EditState JSON without the two fields reads with them; `CaptionPlacement::new` refuses NaN, infinities and values outside 0..=1.

### 6.5 `events.rs`

No variant is added in V4 (D-118). The doc comment of `EventKind` says it is the kind of a plan item, that codes 0-3 are fixed by the event-id layout (D-47), and that a release that draws a new kind appends a variant with the next code.

---

## 7. `offcut-plan` (new): catalog, prompt, validation, the rules producer

**Purpose** (TS §40.2-§40.4). The edit plan's logic: what the renderer can draw (the catalog), how to ask a model for a plan (the prompt and schema, used from V6.1), how a plan is checked and its numbers resolved (`validate`), and how the rule detector's events become a plan (`from_rules`). It holds no state, reads no file, makes no request.

**Crate rules.** Pure (D-122, section 3.2): depends on `offcut-types`, `offcut-text`, `serde`, `thiserror`; `serde_json` is optional, behind the default feature `prompt`, which gates `prompt.rs` (the render bundle builds without it; the server of V6.1 and the tests build with it). No `web-sys`, `js-sys`, `wasm-bindgen`, `wgpu`, clock, randomness or global state. No `unwrap`, `expect`, `panic!` or slice indexing in non-test code. `BTreeMap` only.

### 7.1 `lib.rs`

```rust
pub mod catalog; pub mod config; pub mod from_rules; pub mod resolve; pub mod validate;
#[cfg(feature = "prompt")] pub mod prompt;
pub use catalog::{CATALOG_VERSION, catalog, ItemSpec, ParamSpec, ParamTy, AnchorRule, Producers};
pub use config::PlanConfig;
pub use from_rules::from_rules;
pub use validate::{validate, ValidPlan, ValidItem, Dropped, DropReason, PlanRejected};
```

### 7.2 `catalog.rs` [ONLY] (D-123)

```rust
pub const CATALOG_VERSION: u32 = 1;
pub enum Producers { Rules, Model, Both }
pub enum AnchorRule { Words { min: u32, max: u32 }, Point, ClipWide }
pub enum ParamTy { Label { max_chars: u32 }, QuantityRef, WordIdx, ListPoints { max: u32, label_chars: u32 }, Enum(&'static [&'static str]), Bool }
pub struct ParamSpec { pub name: &'static str, pub ty: ParamTy, pub required: bool }
pub struct ItemSpec { pub kind: EventKind, pub name: &'static str, pub purpose: &'static str,
                      pub anchor: AnchorRule, pub params: &'static [ParamSpec], pub producers: Producers,
                      pub zone: ZoneName, pub max_per_clip: u8, pub min_gap_ms: u32 }
pub fn catalog() -> &'static [ItemSpec];
pub fn spec(kind: EventKind) -> Option<&'static ItemSpec>;
```

`ZoneName` is a `&'static str` matching `offcut-scene`'s `ZoneId` names (`"event"`, `"caption"`); `catalog_complete.rs` checks the names match, so the plan crate does not depend on the scene.

The four rows (all limits (assumption, V6.1, E-11), D-165):

| Kind | Purpose (prompt text) | Anchor | Params | Zone | Max | Min gap |
|---|---|---|---|---|---|---|
| `number_reveal` | "A number the speaker says that matters: show it large while it is said." | Words 1-6 | `value: QuantityRef` (required), `label: Label{32}` | event | 8 | 1,500 ms |
| `list_reveal` | "A list the speaker counts out: a header, then each item as it is spoken." | Words 2-400 | `header: Label{24}`, `items: ListPoints{max 7, 32}` (required) | event | 3 | 10,000 ms |
| `from_to` | "A change from one number to another: show both with bars." | Words 3-40 | `from: QuantityRef`, `to: QuantityRef` (both required), `label: Label{32}` | event | 4 | 3,000 ms |
| `keyword_pop` | "One word the speaker stresses: pop it in the caption." | Point | `word: WordIdx` (required) | caption | 12 | 800 ms |

### 7.3 `prompt.rs` (D-123; first caller V6.1)

```rust
pub struct PromptInput<'a> { pub duration: DurMs, pub words: &'a [Word],
                             pub prosody: &'a [WordProsody], pub sentences: &'a [Sentence] }   // V6.1 fills it from the PlanRequest DTO
pub fn system_prompt() -> String;              // from catalog(): the task, the item list, the rules below
pub fn output_schema() -> serde_json::Value;   // JSON Schema of EditPlan restricted to the catalog
pub fn user_message(input: &PromptInput<'_>) -> String;
```

- **System prompt** (deterministic text): a fixed preamble (the task: "plan the edit of a short talking-head video from its transcript"), one paragraph per catalog row (name, purpose, anchor rule, params with their limits), and the rules every plan must keep: "Never write a number: point at the words that say it", "Every item points at word indexes of the transcript", "Return JSON matching the schema; no prose". V4's prompt has no cut, chapter, music or extras paragraph; V6.x add them with their catalog rows and bump `CATALOG_VERSION`.
- **Output schema**: generated from the catalog; `ItemParams` lists only the word-pointing arms (never `detected`); `source` is not in the schema (the server sets it, TS §40.7).
- **User message**: one line per word `"{i}\t{text}\t{start}\t{end}\t{energy_z:.1}\t{pitch_z:.1}"`, a line `"PAUSE {ms}"` before a word whose gap from the previous one is at least 300 ms (assumption, V6.1), a blank line at each sentence end, and a first line `"DURATION {ms}"`. Text is the ASR text, not the user's edits (TS §40.7, V4 CL decisions).
- **Tests**: `catalog_snapshot.rs` snapshots all three outputs (the user message on the `a01` labeled transcript with zero prosody) with `insta`, and asserts that a change to either generated output without a change of `CATALOG_VERSION` fails: the snapshot file names carry the version (`system_prompt@v1.snap`), so a new version needs new files.

### 7.4 `config.rs` [ONLY]

`PlanConfig { per_kind: BTreeMap<EventKind, KindLimits>, min_keep_gap: DurMs, max_cut_share_permille: u32, max_chapters: u32 }` with `KindLimits { max_per_clip, min_gap: DurMs, label_chars, header_chars }`. `PlanConfig::default()` reads the catalog's numbers; the cut fields hold TS §40.4's values (150 ms, 400 ‰, 6), unused until V6.2 and V6.6.

### 7.5 `resolve.rs` (D-121)

```rust
pub(crate) fn effective_tokens(t: &Transcript, edits: &BTreeMap<WordIdx, String>) -> Vec<Token>;  // offcut_text::tokenize
pub(crate) fn quantity_at(tokens: &[Token], words: WordRange) -> Option<Quantity>;   // first parse inside the range; must consume all its tokens
pub(crate) fn inherit(from: Quantity, to: Quantity) -> (Quantity, Quantity);         // D-85: a Unit::None side takes the other's unit; display re-formatted
pub(crate) fn label_digits_ok(label: &str, tokens: &[Token]) -> bool;                 // every digit run of the label is a digit run of a token
```

- `quantity_at` takes the tokens whose `word` lies in the range, calls `parse_quantity` from the first, and returns the quantity only if it consumed every token of the range except trailing punctuation: "$12k next" pointed at three words gives nothing (the range must be the number).
- `inherit` uses `format_quantity` for the new display. It is the rule of D-85, written once here for model plans; the detector keeps its own for rules plans (D-120).
- **Tests** (inline): "ten thousand" → 10,000; an edited word "nine" in place of "ninety" → 9 minutes after inheritance from "to ten minutes"; a range with a word hidden by an edit → `None`; a label "3x faster" against a transcript without "3" → false.

### 7.6 `validate.rs` [ONLY constructor of `ValidPlan`] (D-121, D-122)

```rust
pub struct ValidPlan { source: PlanSource, items: Vec<ValidItem> }     // private fields
impl ValidPlan { pub fn source(&self) -> &PlanSource; pub fn items(&self) -> &[ValidItem]; pub fn is_empty(&self) -> bool; }
pub struct ValidItem { pub id: EventId, pub kind: EventKind, pub anchors: WordRange, pub span: Span,
                       pub params: EventParams, pub marks: Vec<TimeMs>, pub confidence: Option<Confidence> }
pub struct Dropped { pub kind: Option<EventKind>, pub reason: DropReason }   // kind None for a cut, chapter, extra or cue
pub enum DropReason { UnknownKind, WrongProducer, ParamsMismatch, AnchorsOutOfRange, AnchorRule,
                      LabelTooLong, LabelDigits, NumberUnresolved, OverLimit, TooClose, ZoneTaken,
                      NotInThisBuild }                               // serde snake_case; an analytics enum from V6.1
pub struct PlanRejected { pub reason: RejectReason }                 // pub enum RejectReason { Format, Catalog }
pub fn validate(plan: EditPlan, t: &Transcript, edits: &BTreeMap<WordIdx, String>, cfg: &PlanConfig)
    -> Result<(ValidPlan, Vec<Dropped>), PlanRejected>;
```

Algorithm, in this order:

1. **V-1.** `format != PLAN_FORMAT` → `Err(Format)`; `catalog != CATALOG_VERSION` → `Err(Catalog)`.
2. **Non-item fields (V4).** Every cut, chapter, `extras.hook`, `extras.post_caption`, hashtag and `music` gives one `Dropped { kind: None, reason: NotInThisBuild }` (D-121, D-164).
3. Tokens once: `effective_tokens(t, edits)`.
4. For each item, in the plan's order:
   - **V-2.** `catalog::spec(kind)` exists, else `UnknownKind`. `Detected` arm with `source != Rules`, or a word-pointing arm with `source == Rules`, → `WrongProducer`. The arm matches the kind and every required param is present, else `ParamsMismatch`.
   - **V-3.** `anchors` non-empty and inside `0..words.len()`; the anchor rule's word count holds; every `at` and `word` lies inside the anchors; a `Detected` arm's own times lie inside the transcript; else `AnchorsOutOfRange` or `AnchorRule`.
   - **V-4.** Each `Label` within its `max_chars` (characters, not bytes) → else `LabelTooLong`; `label_digits_ok` → else `LabelDigits`. A `Detected` arm's texts (labels and list item texts made by the detector) are not checked: the detector wrote them from the transcript.
   - **V-5.** Each `QuantityRef` resolves (`quantity_at`); a FromTo's two are passed through `inherit`; else `NumberUnresolved`. The result is the `EventParams` of the `ValidItem` (`ListReveal`'s items: `ListItem { ordinal: i + 1, text: label, at: word start }`; its `count` = item count; its header = the label or `""`).
   - `span` = from the first anchor word's start to the last anchor word's end (for `Detected`, the event's own span is kept: the detector's span is the same rule).
   - `marks`: for a FromTo, the start time of the first word of its second quantity (section 8.7); empty for the other V4 kinds. A FromTo without a second quantity is dropped with `NumberUnresolved`.
5. **V-6.** Per kind, in anchor order: drop an item past `max_per_clip` (`OverLimit`), an item starting less than `min_gap` after the previous kept item of its kind ends (`TooClose`), and an item whose zone is held by an earlier kept item at any instant of its span plus the kind's hold (`ZoneTaken`; the hold is the visual's own: 1,200 ms for NumberReveal, 1,500 ms for the other event-zone kinds; the caption zone is shared by every KeywordPop and never "taken").
6. Items sorted by `(span.start, kind code, anchors.start)`; the `ValidPlan` is returned with the dropped list.

**Errors.** No panic; an arithmetic overflow on a time is `AnchorsOutOfRange`.

### 7.7 `from_rules.rs`

```rust
pub fn from_rules(events: &[DetectedEvent], t: &Transcript) -> EditPlan;
```

One `PlanItem` per event, in the detector's order: `id`, `kind`, `anchors`, `confidence: Some(event.confidence)`, `params: ItemParams::Detected(event.params.clone())`. `source: Rules`, `catalog: CATALOG_VERSION`, everything else empty. The event's `enabled` flag is not copied (every V4 item is enabled by default, D-145). The transcript argument is unused in V4 and kept for V6.4's mapping onto new kinds (D-163 lists it).

### 7.8 Tests

| File | Cases |
|---|---|
| `tests/validate.rs` | One case per V-rule and `DropReason` (21 cases), each with the smallest hand-written plan that triggers it; a model plan of the four word-pointing kinds on the `a01` transcript that validates whole; the same plan after a word edit that changes a number shows the new value (INV-22); a plan with one of each non-item field drops each with `NotInThisBuild` and keeps the items |
| `tests/plan_from_rules.rs` | The reference clip's five events (from `fixtures/speech/README.md`) become five items with the same ids, kinds, anchors and params; validating the plan returns them with the detector's spans; an empty event list gives a valid empty plan |
| `tests/validate_rules_plans.rs` | For every labeled file: detect, `from_rules`, `validate` → no item dropped (D-165) |
| `tests/catalog_snapshot.rs` | Section 7.3 |
| `tests/ui.rs` | `valid_plan_private.rs` fails to compile: `ValidPlan { source, items }` outside the crate (INV-21) |

`catalog_complete.rs` (INV-23) lives in `offcut-scene/tests/`, because it needs the scene's draw table and the scene depends on this crate (section 20.2).


---

## 8. `offcut-scene`: the plan-driven scene, the look

**Purpose** (TS §19, §40.5). From a transcript, a valid plan, the user's choices, the clip, the profile, the time map, a theme and a font set, to what each output frame draws. Built once; `frame_at` is a pure function of the scene and a program time. It touches no GPU, pixel, decoder or clock, parses no number (D-121) and moves no timestamp except through the time map (D-126).

**Crate rules.** Pure. Depends on `offcut-types`, `offcut-text` (`format_quantity` for count-ups, D-28), `offcut-plan` (D-122), `parley`, `serde`, `thiserror`; dev: `insta`, `offcut-detect`, `serde_json`. `BTreeMap` only; no accumulated float time; no `unwrap` in non-test code.

### 8.1 `lib.rs`: the contract (S-1, S-2, S-4, S-5)

```rust
pub struct SceneInput<'a> {
    pub transcript: &'a Transcript,
    pub plan: &'a ValidPlan,
    pub edit: &'a EditState,
    pub clip: &'a ClipInfo,
    pub profile: &'a ExportProfile,
    pub time_map: &'a TimeMap,
    pub theme: &'a Theme,
    pub fonts: &'a FontSet,
}
pub fn build_scene(input: SceneInput<'_>) -> Result<Scene, SceneError>;
impl Scene {
    pub fn duration(&self) -> DurMs;                 // == time_map.out_duration()
    pub fn frame_count(&self) -> u32;                // ceil(duration_ms * OUTPUT_FPS / 1000)
    pub fn frame_at(&self, t: OutMs) -> DisplayList; // Video first, then captions, items, watermark
    pub fn source_at(&self, t: OutMs) -> SrcPos;     // the Main slot's source position
    pub fn summary(&self) -> ChangeSummary;          // summary.rs (D-144)
    pub fn fonts(&self) -> &FontSet;                 // what the renderer draws glyphs with (D-132)
}
pub enum SceneError { Font, Layout, TimeMap }        // TimeMap: an item's span has no program time
```

`SceneAssets` (icons, logo) is not in V4's `SceneInput`: its first user is V6.1 or V6.7, and adding a field there is additive (D-163 lists the V4 seams; this is not one of them).

`build_scene`, in order:

```text
crop       = framing::crop_rect(clip, edit.crop_offset)                        // D-131
style      = styles::spec(edit.style)                                           // roles + motion
words      = captions::words(transcript, &edit.word_edits)                      // unchanged rule (TS §17.2)
words_out  = each word's [start, end] through time_map.out_span; a word with no program time is not shown
block      = placement::caption_block(edit.caption_position, style)              // D-171; the event zone follows it
chunks     = captions::chunk(words_out, sentences, style, theme, fonts, block)  // OutMs; the break search of D-167, the floor of D-168
for item in plan.items() where visible(item, edit) (D-145):
    span_out = time_map.out_span(item.span)?       → SceneError::TimeMap if None
    track    = events::layout(item, span_out, transcript, time_map, style, theme, fonts)?   // None = not drawn
keyword pops are handed to the captions (events::keyword_pop gives the word and the start)
watermark  = profile.watermark ? watermark::layout(theme, fonts) : None
video      = video::Track::fixed(crop)                                          // one Video command per frame (D-130)
scale      = profile.width / CANVAS_WIDTH
```

`frame_at(t)`: `video.cmd(t, scale)`, then the visible chunk's commands, then every track's, then the watermark, every command except `Video` scaled by `scale` (the `Video.dst` is already in output pixels: the full canvas).

### 8.2 `display_list.rs` (D-130, D-132)

The V2 types stay, with these changes:

```rust
pub struct FontKey(pub u16);                                         // replaces FontId (D-132)
pub struct CropRect { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }   // moved here from lib.rs; source display pixels
pub enum VideoSlot { Main }
pub struct ImageKey(pub u16);                                        // V6.7's logo, V6.1's raster icons if any
pub enum DrawCmd {
    FillRect { rect: Rect, radius: f32, color: Rgba },
    FillPath { path: Vec<PathEl>, color: Rgba },
    GlyphRun { font: FontKey, size: f32, glyphs: Vec<PositionedGlyph>, fill: Rgba, stroke: Option<Stroke>, transform: Affine, text: String },
    PushLayer { opacity: f32, clip: Option<Rect> },
    PopLayer,
    Video { slot: VideoSlot, src: CropRect, dst: Rect, opacity: f32, mask: Option<Vec<PathEl>> },
    Image { image: ImageKey, rect: Rect, opacity: f32 },
}
```

`DrawCmd::scale` leaves `Video` unchanged (its `dst` is computed in output pixels), scales an `Image`'s rect. Serialization is the display-list snapshot format (insta, JSON): the variant order above is part of it from V4 on, and a new variant is appended.

### 8.3 `fonts.rs` and `layout.rs`: the registry (D-132)

```rust
pub struct FontSet { faces: Vec<FontFace> }                          // index = FontKey
pub struct FontFace { pub data: Arc<[u8]>, pub family: String, pub weight: f32, pub coords: Vec<i16> }
impl FontSet {
    pub fn embedded() -> FontSet;                                    // keys 0..=3 as V2's FontId order
    pub fn get(&self, key: FontKey) -> Option<&FontFace>;
    pub const EMOJI: FontKey = FontKey(3);
}
```

- `embedded()` is built once per thread (as V2's `Fonts`), behind the same `thread_local!`; a `FontSet` is cheap to clone (`Arc` data).
- `layout::shape(text, key, size, fonts)` sets text in the key's family and weight with Noto Emoji behind it (TS §19.6 fallback chain unchanged), and returns runs whose `font` is the key that drew them.
- `normalized_coords` moves into `FontFace.coords`, computed when the face is registered; the renderer reads them from there (V2's `layout::normalized_coords` is removed; its test becomes `fonts.rs`'s).
- **V6.7** adds `FontSet::with_user(files: &[UserFont])`, appending faces after key 3; nothing else changes.

### 8.4 `theme.rs` [ONLY colour values] and `styles.rs` [ONLY] (D-136 to D-139)

```rust
pub struct Theme { pub text: Rgba, pub stroke: Rgba, pub accent: Rgba, pub accent_alt: Rgba, pub panel: Rgba,
                   pub backdrop: Rgba, pub display_font: FontKey, pub body_font: FontKey, pub mono_font: FontKey }
pub enum Role { Text, Stroke, Accent, AccentAlt, Panel, Backdrop }
pub enum FontRole { Display, Body, Mono }
impl Theme { pub fn default_for(style: StyleId) -> Theme; pub fn color(&self, r: Role) -> Rgba; pub fn font(&self, r: FontRole) -> FontKey; }
```

| | Clean | Bold | Tech |
|---|---|---|---|
| `text` | `#FFFFFF` | `#FFFFFF` | `#E8F0EA` |
| `stroke` | `#101014` | `#101014` | `#101014` |
| `accent` (active word) | `#FFD60A` | `#FFD60A` | `#3DDC84` |
| `accent_alt` (KeywordPop, "to" value) | `#FFD60A` | `#FF5E3A` | `#3DDC84` |
| `panel` | `#101014` at 85% | `#101014` at 85% | `#0B0F0C` at 85% |
| `display_font` / `body_font` / `mono_font` | Inter 800 / Inter 700 / JetBrains Mono 700 | Inter 900 / Inter 700 / JetBrains Mono 700 | JetBrains Mono 700 / Inter 700 / JetBrains Mono 700 |
| Pill (active word, D-170) | `accent` behind, word in `#111114` | none | none |
| Shadow | 0 x 4 lp, black 45% | 0 x 4 lp, black 45% | none |

All (assumption, E-2; D-170 for Clean and Bold). At S5, before the redesign, `Theme::default_for(Clean)` reproduces V3's values exactly for the identity proof (D-117); S9 sets the values of this table. `StyleSpec` (one row per style):

```rust
pub struct StyleSpec { pub font: FontRole, pub size: f32, pub case: Case, pub max_chars_per_line: u32,
                       pub max_lines: u32, pub max_words: u32, pub fill: Role, pub stroke_width: f32,
                       pub stroke: Role, pub active: Role, pub pop: Role, pub panel: Option<PanelSpec>,
                       pub motion: CaptionMotion }
pub struct PanelSpec { pub role: Role, pub radius: f32, pub padding: f32 }
```

Values: TS §19.4's table with D-139. Clean `font: Display`, Bold `Display` (Inter 900 through the theme), Tech `Mono`.

### 8.5 `motion.rs` (D-137)

```rust
pub enum MotionUnit { Word, Glyph }
pub enum MotionPreset { None, Fade, Pop }            // V6.5 appends (Rise, Slide, Typewriter, Bounce, …)
pub enum ActiveTreatment { Highlight, Cursor }       // V6.5 appends (Box, Underline, …)
pub struct CaptionMotion { pub unit: MotionUnit, pub enter: MotionPreset, pub active: ActiveTreatment,
                           pub easing: Easing, pub duration_ms: u32 }
pub(crate) fn enter_at(m: &CaptionMotion, since_ms: u32) -> (f32 /*opacity*/, f32 /*scale*/);  // pure
```

`Fade`: opacity 0 → 1 over `duration_ms` from the chunk's show start. `Pop`: each word scales 1.0 → 1.12 → 1.0 over `duration_ms` from its own start (Bold). `None`: shown at once. `Highlight`: the active word's fill is `active`. `Cursor`: a `FillRect` 0.6 em wide, the line's ascent plus descent tall, in `active`, after the active word, shown while the word is active (Tech). Per-glyph timing (`MotionUnit::Glyph`) is not implemented in V4: a style with it is refused by `styles::spec`'s test.

### 8.6 `captions.rs`

V2's chunking and line rules unchanged (TS §19.4), with:

- Times in `OutMs` (the chunk's `show` is an `OutSpan`; `CHUNK_GAP_MS` compares program times).
- Shaping through the `FontSet` and the style's `FontRole`.
- Tech's panel: one `FillRect` (radius 24 lp) behind the chunk's lines, the lines' box plus 32 lp padding, drawn before the text; the panel's box is what the safe-area test checks for Tech.
- Bold's word pop and KeywordPop's scale: a glyph run's transform scales about the word's centre on its baseline. KeywordPop: the keyword word of a visible KeywordPop item (handed over by `events::keyword_pop`) uses `pop` and scales 1.0 → 1.25 at 90 ms → 1.10 at 180 ms, held 1.10 until the word ends (TS §19.4).
- The file stays under 400 lines: the drawing of a chunk (text, Tech's panel and cursor, Bold's pop, KeywordPop's scale) moves to `captions_draw.rs`; `captions.rs` keeps the words, the chunking and the lines (D-150's rule).

### 8.7 `events/`

| File | Layout input | Draw |
|---|---|---|
| `mod.rs` | `layout(item: &ValidItem, span: OutSpan, t: &Transcript, map: &TimeMap, style, theme, fonts) -> Result<Option<EventTrack>, SceneError>`; one arm per kind | `draw(track, t: OutMs, out)` |
| `number_reveal.rs` | V2's layout on `OutSpan`, colours from roles; unchanged rules (count-up, fit, label) | Unchanged |
| `list_reveal.rs` | D-140; each item's program time is `map.to_out(item.at)` (an item with no program time is not drawn) | Header and items with their own fade-and-rise |
| `from_to.rs` | D-141; the time of the "to" value: below | Two columns, arrow path, bars |
| `keyword_pop.rs` | The word and its program start; no track of its own in the event zone | Nothing: the captions draw the pop |

**The time of FromTo's "to" value.** A `Detected` FromTo carries no word range per value (`EventParams` is frozen, V3 §21.1). `validate` therefore records it: for a `Detected` FromTo, V-5's pass finds the second quantity inside the anchors by running `parse_quantity` from each effective token in turn and taking the start word of the second quantity found (the detector built the event from exactly two quantities, D-106), and stores its start time in `ValidItem.marks` (`marks: Vec<TimeMs>`, a field of `ValidItem` that holds the times a visual steps at: FromTo's second value, empty for the other V4 kinds). A word-pointing FromTo (V6.1 on) gets the same mark from its `to` range. The scene reads `marks` through the time map and never tokenizes. Tested on every FromTo of the 31 labeled files: each gets one mark, later than its first word.

### 8.8 `framing.rs` [ONLY], `video.rs`, `watermark.rs`, `summary.rs`, `safe_area.rs`

- `framing::crop_rect(clip, offset)`: TS §19.3 exactly, with `offset.get()` used (V2's stand-in removed).
- `video.rs`: `pub(crate) struct Track { crop: CropRect }` and `fn cmd(&self, t: OutMs, canvas: (f32, f32)) -> DrawCmd` returning `Video { Main, src: crop, dst: full canvas, opacity: 1.0, mask: None }`. V6.1 adds zoom keys (punch-in), V6.2 the jump-cut alternation, V6.5 masks and placements: all inside this file.
- `watermark.rs`: D-143.
- `summary.rs`: D-144.
- `safe_area.rs`: D-138. `SAFE_AREA`, `CANVAS_WIDTH`, `CANVAS_HEIGHT`, `WATERMARK_ANCHOR` keep their values.

### 8.9 Caption intelligence (D-167 to D-174)

**`readability.rs` [ONLY]** holds the English lists and costs of D-167 and `MIN_CHUNK_MS` (D-168), each a sorted `const` slice or a named constant, all (assumption, E-2). `CLAUSE_STARTERS` (about 40 words), `BINDERS` (about 70: articles, possessives, prepositions, "to", the auxiliaries and modals, "not", "very", "really", the cardinals that are not the last word of a number). Matching is on the lower-cased effective text of a caption word, punctuation stripped.

**The break search** (`captions::chunk`):

```text
segments = split the caption words at hard breaks (sentence end, gap >= CHUNK_GAP_MS)
for each segment (at most 40 words, TS §17.2):
    best[i] = least cost of chunking words[0..i]
    best[i] = min over j < i where words[j..i] fits the style (max_words, lines x chars):
                 best[j] + cost_of_break_before(i) + cost_of_chunk(j..i)
    cost_of_break_before(i): 0 after , ; : —   | 1 before a CLAUSE_STARTER | 12 after a BINDER | 3 otherwise
    cost_of_chunk(j..i):     8 if one word and not isolated by pauses >= 250 ms both sides | 0 otherwise
                             + the best line split of the chunk: min over k of
                               (binder rule for the line break) + imbalance(len(line1), len(line2))
    imbalance(a, b) = 0 if |a - b| <= max(a, b) / 3, else |a - b| in characters
ties: the earlier break wins (deterministic)
```

A segment of one word is one chunk. The search costs O(n x max_words) per segment: microseconds.

**The reading floor** (D-168) is applied after chunking: a chunk's `show.end` becomes `max(show.end, show.start + MIN_CHUNK_MS)` capped at the next chunk's `show.start` and at its own last word's end plus `CHUNK_TAIL_MS` plus the silence after it. Lead and tail rules of TS §19.4 are otherwise unchanged.

**Emphasis** (D-169): `captions_draw.rs` chooses each word's fill in the order active → KeywordPop → number (when `edit.highlight_numbers`) → `fill`. A caption word is a number when it was made from a `NormalizedSpan` (TS §17.2) or when its effective text parses as a quantity in the same way the captions show it (`captions::words` already knows which words carry a number's display).

**Placement** (`placement.rs` [ONLY], D-171): `caption_block(p: CaptionPlacement, style) -> Rect` (the zone of height 280 lp whose centre is `CAPTION_TOP + p x (CAPTION_BOTTOM - CAPTION_TOP)`); `event_zone(caption_block) -> Rect` (D-171's rule); `zone(ZoneId)` in `safe_area.rs` reads them for the clip's placement. The three snap points are `SNAP_TOP = 0.15`, `SNAP_MIDDLE = 0.55`, `SNAP_BOTTOM = 1.0`.

**The pill** (D-170): drawn by `captions_draw.rs` as a `FillRect` (radius 14 lp) behind the active word, before the word's run; between two words it moves: its rectangle at `t` is the eased interpolation, over `PILL_SLIDE_MS = 90`, from the previous word's box to the active word's box, starting at the active word's start; its scale grows 0.9 → 1.0 over the same time. The pill exists only while a word is active (between words, during a pause, none is drawn). The shadow: the run again, before the stroke, offset by `ShadowSpec.offset`, in black at `opacity`.

**Hit testing** (`hit.rs`, D-174): the chunk visible at `t`, its lines and words as laid out (boxes in logical px, padded 8 lp); the caption block of `placement.rs`; else `Hit::Video`. No shaping happens in `hit`.

**The shaping cache** (D-173): `layout.rs`'s `thread_local!` gains an `LruCache`-like structure written in the file (a `BTreeMap` keyed by `(String, FontKey, u32 size bits)` plus a use counter; 2,048 entries, the least recently used evicted), deterministic because shaping a key always gives the same runs. `scene_cache.rs` builds six scenes three ways (cold, warm, cache disabled through a test-only switch) and compares every frame's display list.

### 8.10 Tests

| File | Cases |
|---|---|
| `tests/display_list_snapshots.rs` | Exact JSON snapshots (insta) at 1080 wide of: each style x each kind at three instants (before, during, after the visual: 36 snapshots of a fixed small transcript per kind); the watermark on and off; a landscape crop at -1, 0, 1; a word edit that changes a number (the snapshot shows the edited value); a disabled item absent |
| `tests/layout_safe_area.rs` | D-158: 31 labeled files x 3 styles, every 33 ms; no `GlyphRun`, `FillRect` or `FillPath` box leaves the safe area; the watermark's box meets no caption box |
| `tests/framing.rs` | `ok_portrait_3x4` (1080x1440 → top and bottom go), 1920x1080 at offsets -1, -0.5, 0, 0.5, 1 (x = 0, 328.125, 656.25, 984.375, 1312.5), 1080x1920 whole frame at any offset, rotated 90 display sizes (720x1280) |
| `tests/determinism.rs` | Each of six scenes built twice: byte-equal `Scene` and display lists for every frame (INV-7); `frame_at` asked out of order equal |
| `tests/styles_use_theme.rs` | D-136's text search over `src/` |
| `tests/line_breaks.rs` | The 30-sentence table of D-167 (written and committed at S8 before the search, the founder reads ten): for each, the expected chunks and lines in Clean and Bold; plus property checks over the 31 labeled transcripts: no line ends with a word of `BINDERS` unless the chunk's words allow no other split, no chunk exceeds its style's limits, every word appears exactly once and in order |
| `tests/placement.rs` | At 0.0, 0.15, 0.55, 1.0: the caption block inside the safe area, the event zone inside it and disjoint from the caption block, the watermark box disjoint from both; at 1.0 the event zone is V3's 860-1180 |
| `tests/parity.rs` | D-172 |
| `tests/scene_cache.rs` | D-173 |
| `tests/hit.rs` | A word's centre hits that word; a point in the block between lines hits `Captions`; a point outside hits `Video`; nothing hits while no chunk is visible |
| `tests/timemap_identity.rs` | For the reference clip's plan: a scene on `TimeMap::identity(d)` has `duration == d`, `frame_count` of TS §19.2, `source_at(t) == At(t)` for every frame time, and each chunk's and item's program span equal to its recording span |
| `tests/catalog_complete.rs` | INV-23 (section 7.8) |
| inline (`lib.rs`, `captions.rs`, `framing.rs`, `number_reveal.rs`) | V2's cases on the V4 contract (D-160) |

---

## 9. `offcut-render`: video as a command, native rendering, golden frames

**Purpose** (TS §19). Draws a display list, with the video frames it names, on a canvas (browser) or into memory (native tests). It knows no scene semantics and closes no `VideoFrame`.

### 9.1 `lib.rs` (D-133, D-134)

```rust
pub enum RenderError { NoAdapter, DeviceRequest, DeviceLost, Surface, FrameImport, Overlay, Unsupported }
pub struct VideoSources<'a> { pub main: (&'a web_sys::VideoFrame, Rotation) }     // wasm32
impl Renderer {
    pub async fn new(canvas: OffscreenCanvas, width: Px, height: Px) -> Result<Renderer, RenderError>;
    pub fn resize(&mut self, width: Px, height: Px) -> Result<(), RenderError>;
    pub fn set_canvas(&mut self, canvas: OffscreenCanvas, width: Px, height: Px) -> Result<(), RenderError>;
    pub fn render(&mut self, sources: &VideoSources<'_>, list: &DisplayList, fonts: &FontSet) -> Result<(), RenderError>;
}
#[cfg(not(target_arch = "wasm32"))]
impl Renderer {
    pub async fn new_headless(width: Px, height: Px) -> Result<Renderer, RenderError>;   // software adapter
    pub fn render_to_image(&mut self, source: &RgbaImage, rotation: Rotation, list: &DisplayList,
                           fonts: &FontSet) -> Result<RgbaImage, RenderError>;
}
pub struct RgbaImage { pub width: u32, pub height: u32, pub data: Vec<u8> }        // straight alpha, row-major
```

`VideoSources` is `cfg`-split: on wasm32 it holds a `VideoFrame`; natively, `render_to_image` builds the source from the image. Both paths share every other line.

### 9.2 `composite.rs` (D-133)

```text
render(sources, list, fonts):
  find the Video commands; V4 rule: exactly one, at index 0, slot Main, opacity 1.0, mask None,
     dst inside the canvas → else Unsupported; any Image → Unsupported
  video_pass.load(source, rotation, src)            // the crop is now per call (it was per scene)
  overlay = commands after the Video command → vello_backend (empty list: no overlay pass)
  one render pass: clear black; video quad at dst; overlay over it (premultiplied)
```

V6.5 replaces the rule with layers (commands before a `Video` go into an under-layer, several `Video` commands, masks through a Vello-rendered alpha texture); the function's signature does not change.

### 9.3 `video_pass.rs`, `gpu.rs`, `vello_backend.rs`

- `video_pass.rs`: `load(..., src: CropRect)` as V2 (rotation in display coordinates, TS §19.7); `draw(pass, dst: Rect, target: (u32, u32))` sets the quad's vertices from `dst` (a uniform of four floats). Natively, `load_image(&RgbaImage, rotation, src)` uploads the image as the source texture.
- `gpu.rs`: a headless target (an `Rgba8Unorm` texture with `COPY_SRC`) and a readback buffer for `render_to_image`; `force_fallback_adapter: true` for `new_headless`.
- `vello_backend.rs`: glyph runs read font data and coordinates from the `FontSet` by key (it caches one `vello::peniko::Font` per key per renderer); `Video` and `Image` commands are skipped (the compositor owns them).

### 9.4 Tests: `golden_frames.rs` (D-134, D-135)

- Source frame: `pattern(w, h)`: a diagonal gradient, an 8x8 grid of 2-pixel lines, a filled disc at a third of the width: drawn by the test into an `RgbaImage` 1920x1080 (landscape cases) or 1080x1920 (portrait cases).
- Scenes: built with `build_scene` from small fixed transcripts and rules plans (the snapshot inputs of section 8.10), the preview profile (540x960); each golden is the frame at a fixed instant inside the visual.
- The 16 images, each named `<style>_<kind>.png`, `clean_number_watermark.png`, `clean_number_nowatermark.png`, `crop_landscape_050.png`, `punch_in_110.png` (a hand-written list), compared with SSIM ≥ 0.98 (`tests/common/ssim.rs`: luma, 8x8 windows, the standard constants).
- A missing golden fails with "run with OFFCUT_GOLDEN_UPDATE=1 and review the image"; an adapter that is not a software one fails with its name (D-134).

---

## 10. `offcut-dsp`: program audio (identity)

**Purpose** (TS §40.6). V4 adds the one place that may produce audio of another length than the recording, and makes it return the recording unchanged.

```rust
// program.rs [ONLY]
pub struct ProgramConfig { /* empty in V4; V6.2 cross-fade, V6.3 ducking */ }
pub struct MusicInput<'a> { pub pcm48: &'a [f32], pub loudness: Lufs }
pub struct ProgramAudio { pub samples: Vec<f32>, pub loudness: Lufs }
pub fn assemble(out48: &[f32], map: &TimeMap, speech: &[OutSpan], music: Option<MusicInput<'_>>,
                cfg: &ProgramConfig) -> Result<ProgramAudio, DspError>;
// silence.rs
pub fn silence_profile(out48: &[f32]) -> SilenceProfile;     // RMS dB per 10 ms window, floor -100 dB
```

- **V4 behaviour.** `map.is_identity()` and `music.is_none()` → `samples = out48.to_vec()`, `loudness` measured with the chain's meter (`loudness.rs`). Anything else → `DspError::Unsupported` (a new arm, mapped to `E_DSP` with detail `Unsupported`), so no V4 path can cut or mix (D-164).
- `SilenceProfile { rms_db_10ms: Vec<f32> }` is defined in `offcut-types/src/timemap.rs` (it is the time map's input in V6.2, TS §40.6). `silence_profile` is written and tested in V4 because it is pure and small; nothing calls it until V6.2 (D-163 lists it).
- **Tests** (`tests/program_identity.rs`): on the derived portrait and long-pause PCM (`fixtures/.gen/`), `assemble` with the identity map returns the input sample for sample and a loudness within 0.01 LU of `run_chain`'s; a non-identity map and a music input each return `Unsupported`; `silence_profile` of 1 s of zeros is 100 windows at -100 dB, of a full-scale 1 kHz sine 100 windows near -3 dB.

---

## 11. WASM binding crates

### 11.1 `offcut-wasm-render`

`session.rs` keeps the session (open, the demuxer calls of D-30, `attach_canvas`, `resize`, `render_frame`, `frame_count`). `scene_api.rs` (new, D-150) holds:

```rust
#[wasm_bindgen] pub fn rules_plan(transcript: JsValue, events: JsValue) -> Result<JsValue, JsValue>;   // EditPlan
#[wasm_bindgen] impl RenderSession {
    pub fn set_scene(&mut self, input: JsValue) -> Result<JsValue, JsValue>;   // → SceneReport { dropped: Dropped[] }
    pub fn source_at(&self, t_out_ms: u32) -> JsValue;                         // { at: number } | { held: number }
    pub fn hit(&self, t_out_ms: u32, x: f32, y: f32) -> JsValue;               // Hit (D-174); x, y fractions of the canvas
    pub fn summary(&self) -> JsValue;
}
```

`set_scene`'s input: `{ transcript, plan: EditPlan, edit: EditInput, profile, timeMap: TimeMap }`. It runs `validate` (refusal → `E_INTERNAL` with detail `PlanFormat` or `PlanCatalog`), refuses a non-identity map in V4 (`TimeMapNotIdentity`, D-164), takes `Theme::default_for(edit.style)` and `FontSet::embedded()`, builds the scene, and answers the dropped list. `render_frame(frame, t_out_ms)` gives the renderer `VideoSources { main: (frame, clip.rotation) }`, the list of `frame_at(OutMs(t))` and the scene's fonts. `resize(width, height)` calls `Renderer::resize` (D-149).

### 11.2 `offcut-wasm-core`

`audio_api.rs` gains `assemble(out48: &[f32], time_map: JsValue, speech: JsValue) -> Result<JsValue, JsValue>` returning `{ prog48: Float32Array, loudness }`; PCM crosses by copy, as D-110.

---

## 12. Code generation

`pnpm gen` (V1's `ts-rs` export) adds to `web/src/gen/domain.ts`: `OutMs` (branded), `OutSpan`, `Segment`, `TimeMap` (with its `segments` field public in TypeScript: the type is data on the page; only Rust validates it), `EditPlan`, `PlanSource`, `Cut`, `CutWhy`, `PlanItem`, `ItemParams`, `ListPoint`, `QuantityRef`, `Label` (`string`), `Chapter`, `TextExtras`, `MusicCue`, `MusicEnergy`, `DropReason`, `Dropped`. `PLAN_FORMAT` and `CATALOG_VERSION` join the generated constants beside `LIMITS`. `check-gen-clean.sh` fails if the committed file differs. `web/eslint.config.js`'s branded-type list is read from the generated file (V2), so `OutMs` is branded with no edit there; the `MINTING` table moves `control-preview.ts`'s entry from `TimeMs` to `OutMs` (D-126).

---

## 13. Server: one enum widened

V4 adds no route, no table, no variable. One shared type changes: `ReviewActionKind` in `offcut-api-types/src/analytics.rs` (the allowlist, TS §22.6) gains `caption_position`, `highlight_numbers`, `replace_all`, `undo` and `redo`, so the server accepts the new `review_action` values; `analytics_allowlist.rs` gains one case per member, and `ANALYTICS_EVENT_DOCS` (the settings page's list) is regenerated. The image is rebuilt and deployed server-first as always. V6's seam S-10 is V6's work (BP §8.1). `git diff --name-only <baseline> -- server` lists only the regenerated analytics test fixtures, if any.

---

## 14. Web infrastructure: protocol, RPC, pool, loaders

### 14.1 `workers/protocol.ts` (REOPENED V1 CONTRACT; D-124, D-126, D-127, D-128)

```ts
export interface AudioWorkerApi {
  runChain(p: { pcm48: Float32Array }): Promise<{ loudness: Lufs }>;
  measureProsody(p: { words: Word[]; sentences: Sentence[] }): Promise<{ out48: Float32Array; prosody: Prosody }>;
  /** out48 transferred in, prog48 out; the worker keeps nothing (D-127). */
  assemble(p: { out48: Float32Array; timeMap: TimeMap; speech: OutSpan[] }): Promise<{ prog48: Float32Array; loudness: Lufs }>;
  /** Fire-and-forget: drops any audio the worker holds (D-128). */
  release(): void;
}
export interface RenderWorkerApi {
  openSession(p: { clipId: ClipId; clipInfo: ClipInfo }): Promise<void>;
  detect(p: { transcript: Transcript; prosody: Prosody }): Promise<DetectedEvent[]>;
  redetectSentence(p: { sentence: SentenceIdx; wordEdits: Record<number, string> }): Promise<DetectedEvent[]>;
  rulesPlan(p: { transcript: Transcript; events: DetectedEvent[] }): Promise<EditPlan>;
  setScene(p: { transcript: Transcript; plan: EditPlan; edit: EditState; timeMap: TimeMap;
                profile: "preview" | { entitlementToken: string } }): Promise<SceneReport>;
  attachPreview(p: { canvas: OffscreenCanvas }): Promise<void>;
  previewPlay(p: { clock: ClockSync }): Promise<PreviewStats>;
  previewClock(p: { clock: ClockSync }): void;
  previewPause(): Promise<void>;
  previewSeek(p: { at: OutMs }): Promise<void>;
  /** x, y as fractions of the canvas (D-174). Only while the preview is not playing. */
  hitTest(p: { at: OutMs; x: number; y: number }): Promise<Hit>;
  /** Three frames of `at` in Clean, Bold, Mono with the current edits; transferred (D-186). */
  styleThumbnails(p: { at: OutMs; width: number; height: number }): Promise<ImageBitmap[]>;
  exportClip(p: { exportId: ExportId; entitlementToken: string; out48: Float32Array }):
    Promise<{ opfsPath: string; summary: ChangeSummary; stageTimings: StageTiming[] }>;
  closeSession(): Promise<void>;
}
export type ClockSync = { audioMs: OutMs; epochMs: number };
export type SceneReport = { dropped: Dropped[] };
export type PreviewStats = { frames: number; late: number; maxDriftMs: DurMs; downscaled: boolean };
export type Hit = { kind: "word"; word: WordIdx } | { kind: "captions" } | { kind: "video" };
```

Costs on the V1 side: `setScene`'s `events` becomes `plan` plus `timeMap`, and it answers a report; `previewPlay` answers statistics; `ClockSync` and `previewSeek` change type, not shape. The `serveWorker` options of the render worker gain nothing (`previewSeek` was already allowed during a preview); the audio worker's `oneWay` gains `release`.

### 14.2 `rpc.ts` and `rpc-worker.ts` (D-150, D-151, D-153)

- Split with no change first (S3), then: the throttle on the worker side (D-151); `createClient` exposes `cancel(jobId)` and `onCrash(cb)`; a worker `error`/`messageerror` rejects every pending call with `E_WORKER_CRASH` and the pool row's stage, and the client is dead (`isDead()`).
- `rpc.test.ts` (Vitest, a fake `Worker` that answers by script): request and response; failure; progress throttled to 10/s with feed lines all kept and the held progress flushed before `res`; cancel answered; cancel not answered → the caller's timeout path (tested in `pool`'s terms through a fake clock); crash rejects all pending; a call after a crash makes a new worker through the factory; a transferred buffer is detached (dev assertion).

### 14.3 `pool.ts` (D-152, D-153)

```ts
export const pool: { media, asr, audio, render: PoolClient<…>;
  cancel(kind: WorkerKind): Promise<"cancelled" | "restarted" | "idle">;   // CANCEL_TIMEOUT 2,000 ms
  restartWorker(kind: WorkerKind): void; }
```

`lazyClient` keeps its worker until `restartWorker` or a crash; the next call creates a fresh one. A restarted render worker has no session: the use-cases that need one (`reloadPreview`, D-153) open it again.

### 14.4 `wasm/load-render.ts`, `wasm/load-core.ts`

`RenderApi` gains `rulesPlan(transcript, events): EditPlan`; `RenderSession`'s TypeScript view: `set_scene(input): SceneReport`, `source_at(tOut: number): { at: TimeMs } | { held: TimeMs }`, `resize(width, height): void`. `CoreApi` gains `assemble(out48, timeMap, speech): { prog48: Float32Array<ArrayBuffer>; loudness: Lufs }`.

---

## 15. Workers: `render.worker`, `audio.worker`, the preview loop, the video source

### 15.1 `render.worker.ts` and its two job files (D-150)

The worker file keeps the session state (`open`, the scene input, the preview canvas, the last detection) and the handler table. New or changed handlers:

| Handler | Behaviour |
|---|---|
| `detect` | As V3, and keeps `{ transcript, prosody, events }` for `redetectSentence` |
| `redetectSentence({ sentence, wordEdits })` | `RenderApi.redetectSentence(transcript, prosody, wordEdits, sentence, lastEvents)`; keeps and returns the new events |
| `rulesPlan({ transcript, events })` | `RenderApi.rulesPlan` (stateless) |
| `setScene({ transcript, plan, edit, timeMap, profile })` | `session.set_scene`; keeps the input for the export's switch and back (as V2 kept it) |
| `previewSeek({ at })` | `preview-job.ts`: stops a running loop, `source.frameAtBlocking(srcOf(at))`, `render_frame`, resolves |
| `previewPlay` | `preview-job.ts`: as V2, with `OutMs`, `source_at` and the statistics returned |
| `setScene` (after the first) | When a preview canvas is attached and not playing, redraws the frame at the last position (D-173) |
| `hitTest({ at, x, y })` | `scene_api::hit` (D-174) |
| `styleThumbnails({ at, width, height })` | One `OffscreenCanvas` of that size; the scene rebuilt three times at a thumbnail profile with the current edits and each style; `transferToImageBitmap` after each render; the preview's scene and canvas restored after (as the export switch does) |

`render/export-job.ts` holds V2's `writeExport`, `afterExport` and `exportClip` unchanged in behaviour; the profile switch and back now pass the plan and the map.

### 15.2 `render/preview-loop.ts` (D-148, D-149)

```text
runPreview(session, source, clock, isStopped):
  per step: t = clock() (OutMs); n = floor(t * 30 / 1000); if n >= total → end
            tn = n * 1000 / 30 (floor)                      // program time of frame n
            src = session.source_at(tn)                     // { at } in V4, always
            frame = source.frameAt(src.at) ?? lastFrame     // late → reuse, count
            cost = time of render_frame(frame, tn)
            mean of the last 60 costs > 28 ms and not downscaled → session.resize(360, 640); downscaled = true
  returns { frames, late, maxDriftMs, downscaled }
```

### 15.3 `render/video-source.ts` and `render/frame-decoder.ts` (D-150, D-148)

The split keeps `frameAt` and `frameAtBlocking`'s contracts (TS §20.1). `prefetch(from)` starts decoding at the keyframe at or before `from` without handing out a frame. A seek outside the queue (before its head, or past its horizon by more than 1 s) flushes and restarts at the keyframe at or before the time (TS §20.2), now reached through `frame-decoder.ts`'s `restartAt(keyframeIndex)`.

### 15.4 `audio.worker.ts` (D-127, D-128)

`assemble` calls `CoreApi.assemble` on the transferred `out48`, transfers `prog48` back and keeps nothing. `release` sets `kept = null`. The worker's comment keeps its V3 sentence ("it decides nothing about the audio") with one addition: "`assemble` applies a time map it is given; in V4 the only map is the identity".

---

## 16. State and use-cases: edits, cancel, retry, seek

### 16.1 Stores

- **`clip-store.ts`**: `plan?: EditPlan`, `timeMap?: TimeMap`, `prog48: Float32Array | null` (replaces `out48`, D-127; the export still reads `out48.f32` from OPFS, D-129), `cancelled: boolean` (the clip is `accepted` after a cancel, D-152). New actions: `setPlan(plan)`, `setEvents(events)`, `editStarted(edit)` (`ready → updating`, the optimistic `EditState`), `editDone()` (`updating → ready`), `editFailed(previous, failure)` (`updating → ready` with the old state and a failure), `cancelledRun()` (`processing → accepted`), `retried()` (`failed → processing`). The machine's table is unchanged (V2 declared every event).
- **`history.ts`** (D-184): `push(edit)`, `undo() -> EditState | undefined`, `redo() -> EditState | undefined`, `canUndo`, `canRedo`, `clear()`; a store of its own so the top bar can read it.
- **`ui-store.ts`**: the selected tab, the follow switch, the word being edited, the toast queue, the save state (`saved`, `saving`); written by use-cases and by the components' local actions (tab selection), never by a worker.
- **`preview-store.ts`**: `position: OutMs` (updated at each clock sync and seek), `lastStats?: PreviewStats`; actions `seekStart()`, `seekDone(playing: boolean)`; the three seek events now fire.

### 16.2 Use-cases

| File | Flow |
|---|---|
| `run-pipeline.ts` | V3's steps with: after detection, `rulesPlan` → `setPlan`; `timeMap = identity(clipInfo.duration)`; `setScene({ …, plan, timeMap })`; step F writes `out48.f32` first, then `assemble` (D-127) and stores `prog48`. The run takes an `AbortSignal` from `cancel-job.ts`; an abort ends it with `cancelledRun()`, `release()` and `closeSession()`. Every failure shown sends `client_error` (D-153); every early end calls `pool.audio.release()` (D-128) |
| `run-pipeline.test.ts` | With fake workers (a scripted `pool`): the happy path's order (A, B1 ‖ B2, C, D, E with `rulesPlan`, F with `writeAudio` before `assemble`); `REJECT_NO_SPEECH`; a B2 failure held until B1 settles (D-90); a cancel at each stage ends in `accepted` with `release` called; a worker crash ends in `failed` with `client_error`; `retry` from `failed` runs again from A |
| `review-update.ts` | `update(change: (edit) => EditState, mode: "plan" \| "scene")`: the D-147 rule. Mode `plan` (a word edit) runs `redetectSentence` and `rulesPlan` before `setScene`; mode `scene` (toggle, style, crop) runs `setScene` only. Writes `editsRepo.put`, sends `review_action` |
| `edit-word.ts` | `editWord(word: WordIdx, text: string)` → `update` mode `plan` with the sentence of the word (D-146) |
| `toggle-event.ts` | `toggleEvent(id: EventId, on: boolean)` → `event_overrides[id] = on`, mode `scene`; `review_action { action: "event_toggle", event_kind, enabled }` |
| `set-style.ts` | `setStyle(style)`, mode `scene`; `review_action { action: "style" }` |
| `set-crop-offset.ts` | `setCropOffset(offset)` (landscape and square only), mode `scene`; `review_action { action: "crop_offset" }` |
| `set-caption-position.ts` | `setCaptionPosition(p)`, mode `scene`; `review_action { action: "caption_position" }` |
| `set-highlight-numbers.ts` | `setHighlightNumbers(on)`, mode `scene`; `review_action { action: "highlight_numbers" }` |
| `replace-all.ts` | `replaceAll(from, to)` (D-175): one change with every matching index; events by `detect` with the new edits; `review_action { action: "replace_all" }` |
| `undo-redo.ts` | `undo()`, `redo()` (D-184) through `review-update.ts`; `review_action { action: "undo" \| "redo" }` |
| `preview-interact.ts` | `hoverAt(x, y)` (throttled to one per animation frame; hit test only when paused), `clickAt(x, y)` (a word: pause, open it in the Captions tab), `dragCaptions(y)` and `dragVideo(x)` (live guide only; the commit on release goes to `setCaptionPosition` or `setCropOffset`) |
| `style-thumbnails.ts` | `refreshThumbnails()` (D-186): debounced 300 ms, only while the Style tab is shown, the latest call wins |
| `cancel-job.ts` | D-152. It owns the abort controllers: `signalFor(scope)` is imported by `run-pipeline.ts`, `import-clip.ts` and `start-export.ts` (a use-case may import `cancel-job.ts`, TS §7), and `cancelJob(scope)` aborts it, calls `pool.cancel` and applies the scope's store change |
| `control-preview.ts` | `attach(canvas, prog48)`; `seek(at: OutMs)` (D-148: one in flight, the latest wins; sound restarts at `at` when it was playing); `lastStats` stored after each play; `reloadPreview()` (D-153: open the session, set the scene, attach a new canvas) |
| `import-clip.ts`, `start-export.ts` | Honour the cancel signal (`cancelJob("import")`, `cancelJob("export")`); the export machine's `cancel` and `cancel_done` fire |

---

## 17. Interface and interaction design

This section is the design specification of the editor (D-177 to D-193). It is written for content creators in general: someone who has never opened a video editor must get a postable short without reading anything, and someone who edits every day must never wait for the interface.

### 17.1 Principles

1. **The video is the hero.** The preview is the largest and brightest thing on the screen; the interface is dark, quiet and neutral around it (D-178, D-179).
2. **Good by default.** The default style, the highlighted numbers, the suggested visuals and the bottom caption position make a postable short with no click (D-170, D-176).
3. **Direct before indirect.** Fix a word where you see it; move captions and the crop by dragging them; every panel control is the accessible second way to do the same thing (D-182, D-183).
4. **Instant and reversible.** Every change shows within 150 ms (D-173), autosaves, and can be undone (D-184).
5. **Plain words.** No jargon, no codes, one primary action per screen (D-177, D-190).
6. **One system.** Every colour, size, space, radius and motion comes from the tokens (D-179); every control from the primitives (D-181).

### 17.2 Screens and states

| State (clip status) | What the screen shows | Primary action |
|---|---|---|
| Empty (`idle`) | The editor shell with the stage holding the drop zone: "Drop a video (up to 90 seconds)", "or try a sample", the supported-browser line; the side panel collapsed | Choose a video |
| Model download (first run) | The stage frame with the download card (size, progress, "only the first time"), Cancel | - |
| Processing (`importing`, `accepted`, `processing`) | The stage frame with the four-step checklist and the found-moment chips (D-188); side panel tabs disabled with a skeleton | Cancel |
| Cancelled (`accepted` after a cancel) | "Cancelled." in the stage, "Process again" and "Start over" | Process again |
| Ready (`ready`) | Preview with transport; side panel on the Captions tab; top bar with Export enabled | Export |
| Updating (`updating`) | As ready; the changed control shows its pressed state; a spinner on the preview only if the update takes over 300 ms | - |
| Rejected (`rejected`) | The V3 `RejectionPanel` content in the stage card, restyled | Choose another video |
| Failed (`failed`) | `ErrorPanel` in the stage card (D-190) | Try again |
| Exporting (export `rendering`, `muxing`, `saving`) | The export pill in the top bar (D-189); review controls disabled with a tooltip "Available when the export finishes" (TS §12.4) | Cancel |
| Preview failure while ready | `ErrorPanel` over the stage, the panel still usable | Reload preview |

### 17.3 Components

| Component | Contents and behaviour | Calls |
|---|---|---|
| `EditorLayout` (new) | Top bar, stage, side panel (D-178); the `aria-live` region (D-192) | - |
| `TopBar` (new) | Mark, clip length, save state, undo, redo, Export / export pill | `undo`, `redo`, `startExport`, `cancelJob("export")` |
| `Stage` (new) | Holds `DropZone`, the processing card, `PreviewPlayer`, `RejectionPanel` or `ErrorPanel` by state | - |
| `PreviewPlayer` | The canvas in its 9:16 frame; pointer handling of D-183 (hover cursor, click, drags, snap guides); the transport (play/pause, time, scrub bar, D-148); the Free watermark badge (D-172); the `data-*` statistics (D-155) | `controlPreview.*`, `hitTest`, `setCaptionPosition`, `setCropOffset` |
| `ProcessingCard` (new) | The steps, the chips, the line, Cancel (D-188) | `cancelJob("pipeline")` |
| `ModelDownloadPanel` | Restyled; Cancel | `cancelJob("model")` |
| `SidePanel` (new) | `Tabs`: Captions, Style, Visuals, Framing | - |
| `CaptionEditor` (Captions tab) | The transcript editor (D-182), follow switch, word menu, replace-all toast | `editWord`, `replaceAll`, `controlPreview.seek` |
| `StylePanel` (new; contains `StylePicker`) | Three style cards with live thumbnails, Highlight numbers switch (D-186) | `setStyle`, `setHighlightNumbers` |
| `PlanItemList` (Visuals tab) | The rows of D-187, or the empty state | `toggleEvent`, `controlPreview.seek` |
| `FramingPanel` (new; contains `CropOffsetControl`) | Caption position segmented control and hint; crop slider and hint (D-187) | `setCaptionPosition`, `setCropOffset` |
| `CutsSection` | Renders nothing in V4 (V6.2) | - |
| `ErrorPanel` | D-190 | retry, `reloadPreview`, undo |
| `ExportProgress` | The pill of D-189 | `cancelJob("export")` |
| `ShortcutsSheet` (new) | The list of D-185, opened with `?` | - |
| `Toast` host (new) | One stack; replace-all, hide and undo toasts | - |

### 17.4 Interaction details

- **Feedback timing.** A press shows its pressed state at once; anything that takes longer than 300 ms shows a spinner in place; nothing shows a spinner for less than 300 ms (no flicker).
- **Motion.** Panels and toasts fade and rise 8 px over `--base`; tabs switch without motion; the pill in the captions is the content's motion, not the interface's. Under `prefers-reduced-motion` only opacity changes.
- **Hover on the canvas** (paused): over a caption word the cursor is a text cursor and the word gets a 2 px outline drawn as chrome; over the caption block (not a word) a vertical-move cursor; over the video of a landscape clip a horizontal-move cursor.
- **Seek.** Clicking a transcript block or a visual's time seeks without playing; the scrub bar plays from the new place if it was playing.
- **Edits while playing.** A change pauses the preview and resumes it after (D-147); the transcript editor pauses on click.
- **Empty and long content.** A 90 s transcript fits the Captions tab with its own scroll; a word longer than the field's width scrolls inside its field; a style card's thumbnail keeps 9:16.
- **Widths.** Reference widths 1280 x 800 and 1920 x 1080 (D-191); below 1024 px the capability check has already refused the device.

### 17.5 Copy (`copy/messages.ts`, D-159, D-177)

Every string below is a key; every number in it is interpolated (INV-15). Tone: second person, short sentences, verbs first on buttons, no exclamation marks, no jargon.

| Area | Keys (examples of the text) |
|---|---|
| Top bar | `editor.saved` "Saved on this device", `editor.saving` "Saving…", `editor.undo` "Undo", `editor.redo` "Redo", `editor.export` "Export" |
| Tabs | `panel.captions` "Captions", `panel.style` "Style", `panel.visuals` "Visuals", `panel.framing` "Framing" |
| Captions tab | `captions.hint` "Click any word to fix it.", `captions.follow` "Follow playback", `captions.hide` "Hide word", `captions.restore` "Restore original", `captions.show` "Show", `captions.replaceAll(n, from, to)` "Change all {n} “{from}” to “{to}”?" |
| Style tab | `style.clean` "Clean", `style.cleanLine` "Soft and premium: the spoken word on a highlight.", `style.bold`, `style.boldLine`, `style.mono` "Mono", `style.monoLine`, `style.highlightNumbers` "Highlight numbers" |
| Visuals tab | `visuals.count(on, total)` "{on} of {total} visuals on", `visuals.kind.{number_reveal, list_reveal, from_to, keyword_pop}` "Big number", "List", "Before → after", "Emphasis", `noEventsFound` (PS §10 J7), `visuals.whatWeLookFor` |
| Framing tab | `framing.captionPosition` "Caption position", `framing.top`, `framing.middle`, `framing.bottom`, `framing.dragCaptions` "Or drag the captions on the video.", `framing.crop` "Framing", `framing.dragVideo` "Or drag the video sideways." |
| Preview | `preview.play`, `preview.pause`, `preview.seek` "Position", `preview.loadingFrame`, `preview.watermarkBadge` "Free exports include a small watermark" |
| Processing | `processing.steps.{audio, voice, captions, visuals}`, `processing.reassure` "Everything runs on your computer. This usually takes a couple of minutes.", `cancel.*` |
| Shortcuts | `shortcuts.title`, one key per shortcut |
| Errors | `errors.<code>.{title, body, action}` for the 14 codes of D-159, e.g. `E_GPU_LOST`: "The preview stopped" / "Your graphics card was reset, which can happen when another app uses it." / "Reload preview" |

### 17.6 Styles and assets

- `ui/styles/tokens.css` [ONLY design values]: D-179's tokens; the editor sets `data-theme="dark"` on its root; the landing page is unchanged.
- `ui/styles/*.module.css`: one module per component group (`editor`, `panel`, `primitives`, `preview`); no inline style (CSP `style-src 'self'`); every colour, size and duration a token.
- `web/src/ui/assets/inter-latin.woff2` (D-180), its `@font-face` in `tokens.css` (Vite hashes it into `/assets/`), loaded by the editor route.
- `ui/icons.tsx` (D-181).

---

## 18. Fixtures, snapshots, golden frames

- **No new media fixture.** V4 reads V3's generated set (`pnpm fixtures`): the portrait and landscape clips for the preview suites and the crop, `speech_20s_noevents.mp4` for the zero-event case, `ok_rotated_90.mov` for one golden's rotation check (its display size only; the golden's pixels come from the synthetic pattern).
- **Snapshots** (`offcut-scene/tests/snapshots/`, `offcut-plan/tests/snapshots/`): JSON, committed, reviewed in the prompt that creates them; a later change to a snapshot is named in the changelog with its reason, as a label change was in V3 (D-87's rule, applied to snapshots).
- **Golden frames** (`fixtures/golden/*.png`, 16, D-135): written once on D1 with `OFFCUT_GOLDEN_UPDATE=1` at S7 and S8, looked at by the founder at the exit (five of them, chosen at random), compared on every CI run.
- **The identity reference** (D-117): written at S2, deleted at S7; never committed past S7.
- **`fixtures/labeled/`**: read by `layout_safe_area.rs`, `validate_rules_plans.rs` and `plan_from_rules.rs`; unchanged.

---

## 19. Root configuration, scripts, CI, hosting

### 19.1 Cargo workspace

`Cargo.toml`: member `crates/offcut-plan`; `[workspace.dependencies]` `offcut-plan` (path), `insta` (with `json`), `png`. `offcut-render`'s `[dev-dependencies]` re-declares `wgpu` with `features = ["vulkan", "dx12", "wgsl"]` (D-134); a CI step asserts the wasm32 normal tree has neither (section 3.2).

### 19.2 Scripts

| Script | Change |
|---|---|
| `scripts/check-file-tree.mjs` | `PURE_CRATES` + `offcut-plan`; the TS §7 graph (read from the spec) has the edges scene → plan, plan → text, plan → types, wasm-render → plan |
| `scripts/check-copy-codes.mjs` | 14 codes leave `COPY_PENDING` (D-159) |
| `package.json` | One new script, `screens` (D-191). `pnpm test` already runs `cargo test --workspace` and Vitest |

### 19.3 `ci.yml`

Linux `ci` job, before step 4 (`cargo test`): `sudo apt-get install -y mesa-vulkan-drivers` (pinned by the runner image; the version is printed and recorded in the changelog at S16), `INSTA_UPDATE=no` and `CI=true` in the step's `env`. `golden_frames.rs` runs in that step; its output line names the adapter (`llvmpipe`/`lavapipe`). Nothing else changes; the deploy order is unchanged.

### 19.4 `e2e-media.yml` and `playwright.config.ts`

`MEDIA_SUITES` gains `preview-sync.spec.ts`, `review-edit.spec.ts`, `cancel.spec.ts`. The Windows media job runs the `@no-gpu` cases only (D-72): `cancel.spec.ts`'s import cancel and model-download cancel carry the tag; every other new case needs a GPU and runs on D1 before the merge.

### 19.5 Hosting

Unchanged. The web build grows by the plan crate in `offcut_render.wasm` (S5 records the size; over 300 kB of growth is investigated before S6).

---

## 20. Tests, case by case

### 20.1 What runs where

| Suite | Where | Count after V4 (at least) |
|---|---|---|
| `cargo test --workspace` | D1, Linux CI (with fixtures and lavapipe) | 461 + V4's (section 20.2): about 560 passed, 3 ignored (+ `scene_timing`, ignored) |
| Vitest | D1, CI | 92 + `rpc.test.ts`, `clip-machine.test.ts`, `model-machine.test.ts`, `run-pipeline.test.ts`, `history.test.ts` (D-184): about 160 |
| `pnpm e2e` | D1, CI | 12 (unchanged) |
| `pnpm e2e:media` | D1 (all), Windows CI (`@no-gpu`) | 48 + `preview-sync` 4 + `review-edit` 16 + `cancel` 5 + `pipeline-preview` 3 new = 76 cases (`screens.spec.ts` is a D1 tool, not counted) |

The exact counts are recorded at the exit; none may fall below the baseline (1A item 4).

### 20.2 Rust

| File | Cases |
|---|---|
| `offcut-types` inline | `OutMs`/`OutSpan` constructors; `TimeMap` (section 6.2); `plan.rs` serde (section 6.3) |
| `offcut-plan/tests/validate.rs` | 21 rule cases + whole model plan + edited number + non-item drops (section 7.8) |
| `offcut-plan/tests/plan_from_rules.rs` | Reference events → items; empty plan |
| `offcut-plan/tests/validate_rules_plans.rs` | 31 files, nothing dropped |
| `offcut-plan/tests/catalog_snapshot.rs` | Three snapshots; version-named |
| `offcut-plan/tests/ui.rs` | `ValidPlan` private |
| `offcut-scene/tests/display_list_snapshots.rs` | Section 8.9 (about 45 snapshots) |
| `offcut-scene/tests/layout_safe_area.rs` | 93 scenes (31 x 3), every 33 ms |
| `offcut-scene/tests/framing.rs` | Section 8.9 |
| `offcut-scene/tests/determinism.rs` | Six scenes x all frames |
| `offcut-scene/tests/styles_use_theme.rs` | One text search |
| `offcut-scene/tests/timemap_identity.rs` | Section 8.9 |
| `offcut-scene/tests/catalog_complete.rs` | INV-23 |
| `offcut-render/tests/golden_frames.rs` | 16 images; SSIM ≥ 0.98; adapter is software |
| `offcut-dsp/tests/program_identity.rs` | Section 10 |
| `offcut-scene/tests/line_breaks.rs`, `placement.rs`, `parity.rs`, `scene_cache.rs`, `hit.rs` | Section 8.10 |
| `scene_timing` (ignored, release) | `build_scene` ≤ 1 s, mean `frame_at` ≤ 2 ms on the portrait clip's plan (D-166) |

### 20.3 TypeScript (Vitest)

| File | Cases |
|---|---|
| `workers/rpc.test.ts` | Section 14.2 |
| `state/machines/clip-machine.test.ts` | Every legal transition of TS §12.2's clip table; the illegal ones of TS §12.3 (`rejected → processing`, `importing → processing`, `processing → importing`, `ready → importing`) throw `IllegalTransitionError` in tests; `updating` refuses `edit_start` (D-147) |
| `state/machines/model-machine.test.ts` | TS §12.2's model table; `verifying → downloading` without `failed` refused |
| `usecases/run-pipeline.test.ts` | Section 16.2 |
| `state/history.test.ts` | Push, undo, redo, the 50-step limit, a new change after undo drops the redo branch, clear on a new clip |

### 20.4 End to end (Playwright, D1 unless tagged)

| Suite | Cases |
|---|---|
| `pipeline-preview.spec.ts` (complete) | V3's cases as rewritten by D-160; **zero events**: `speech_20s_noevents.mp4` reaches `ready`, the review panel shows `noEventsFound`, no item row; **long tasks** (D-156); **statistics**: after a 5 s play, `data-frames` > 100 and `data-max-drift-ms` present |
| `preview-sync.spec.ts` | **Drift**: the portrait clip plays 60 s; `data-max-drift-ms` ≤ 80 and `late / frames` ≤ 0.05 (TS §20 acceptance); **first frame**: the canvas shows a non-black frame within 500 ms of `ready` (pixel read of the page's canvas through a screenshot); **seek**: seeking to 30 s while paused shows the frame of 30 s (the white-frame marker at 55 s is used: seek to 55.0 s, the canvas's mean brightness rises above 200) within 300 ms on D1 |
| `review-edit.spec.ts` | **Edit a number**: on the reference clip, the word that was recognized as "90" (the from-to's first value, "Found: 90 minutes to 10 minutes") edited to "9" → the from-to's row in `PlanItemList` reads "9 minutes", and `review_action { action: "word_edit" }` is sent; **hide a word**: an empty commit removes it from the caption (`CaptionEditor` shows it struck through); **toggle**: switching the list off removes its row's "on" state and the next frame at the list's time has no list (the item row's state and `data-` of the panel); the toggle survives a word edit elsewhere (D-145); **style**: Bold → the caption's text in upper case on the canvas's accessible label mirror (`data-caption-text`, written by `PreviewPlayer` from the scene's visible chunk, test-only attribute in dev and test builds); **crop**: on `speech_60s_landscape.mp4` the control exists and a drag to 1.0 sends one `review_action { action: "crop_offset" }`; on the portrait clip it does not exist; **pause and restore**: an edit while playing pauses and the preview plays again after; **undo**: a forced `setScene` failure (the session closed from the test through a dev-only hook in `render.worker`, compiled out of production) shows "Undo this change", and pressing it restores the previous state |
| `review-edit.spec.ts` (the editor, D-182 to D-187) | **Click a word in the transcript**: the preview seeks to it and pauses, the field opens with the text selected; **Tab** commits and opens the next word; **Escape** cancels; **click a caption word on the preview**: the Captions tab opens on that word; **replace all**: the founder clip's transcript with a word repeated three times edited once → the toast offers "Change all 2" → all three change, one undo restores all three; **undo and redo** with Ctrl+Z and Shift+Ctrl+Z across a word edit, a style change and a toggle; **drag the captions** to the top: snaps at Top, the visuals' zone is below them (`data-caption-position` on the player), one `review_action { action: "caption_position" }`; **drag the video** on the landscape clip; **style cards**: three images appear within 400 ms, choosing Bold applies it; **Highlight numbers** off: the number in the caption is no longer in the accent colour (the display list mirror `data-caption-colors`, dev/test only); **shortcuts**: Space plays and pauses outside a field and types a space inside one; **accessibility**: axe on ready and editing states, zero serious or critical violations; every control reachable by Tab with a visible focus ring (D-192) |
| `preview-sync.spec.ts` (addition) | **Edit to preview** ≤ 150 ms (section 21.1) |
| `cancel.spec.ts` | `@no-gpu` **import cancel**: a large `ok_*` file's import cancelled → `idle`, no OPFS clip directory, one `job_cancelled { stage: "import" }`; `@no-gpu` **model cancel**: with the model absent, the download cancelled → partial bytes kept, the next run resumes; **pipeline cancel during ASR** → `accepted`, "Process again" works; **export cancel** → `idle`, no temporary file in OPFS, no render-cache entry; **cancel timeout**: a worker made unresponsive by a dev-only hook → restarted after 2 s, the clip is `accepted` |

The dev-only hooks named above are behind `import.meta.env.DEV || import.meta.env.MODE === "test"` and stripped from the production bundle; `check-hosts.mjs`'s bundle scan asserts their marker string is absent from `web/dist` (as the test key is, V2).

---

## 21. Measurements and the V4 decisions of BP §6.2

Write each outcome into `docs/v4/experiments.md` (question, method, numbers, date, decision), replace the matching (assumption) in TS §39.3, label every reading `d1` (D-114).

### 21.1 Readings

| Reading | Step | Procedure | Target | If it misses | Recorded in |
|---|---|---|---|---|---|
| **Edit to preview** | S12, S17 | A Playwright case of `preview-sync.spec.ts`: on the paused 60 s portrait clip, commit a word edit and time until the canvas changes (`requestVideoFrameCallback`-free: the worker's `setScene` answer plus the redraw's `data-frames` tick), 20 runs, the 90th percentile | ≤ 150 ms (TS §13.5, D-173) | Profile the build; cache misses first (D-173); a scene build split per chunk is the next step, recorded as a V4 decision | `experiments.md` |
| Style thumbnails | S12, S17 | Time from the Style tab opening to three images, 10 runs | ≤ 400 ms (assumption, D-186) | Lower the thumbnail size; render one card at a time | Same |
| Hit test and hover | S11 | `hitTest` round trip while paused | ≤ 16 ms (one frame) | Hover hit tests every second frame | Same |
| App shell with the font | S17 | `check-hosts.mjs`'s bundle sizes | ≤ 400 kB gzip (TS §13.5) | D-180: system fonts | `v4changelog.md` |
| Preview drift and late frames | S15, S17 | `preview-sync.spec.ts`'s drift case, five runs, the worst | ≤ 80 ms; late ≤ 5% (TS §20, PS §20.1) | BP §6.2 row 1 (section 21.2) | `experiments.md` |
| First preview frame | S15 | `preview-sync.spec.ts` | ≤ 500 ms after `ready` (assumption, M1.5) | Profile `attachPreview`; record | Same |
| Seek to first frame | S15 | `preview-sync.spec.ts`, GOP of the portrait clip (30 frames) | ≤ 300 ms (assumption, M1.5) | Record; the spinner covers it; V9 re-reads | Same |
| Downscale rule | S17 | `data-downscaled` after a 60 s play on D1 | `false` on D1 | Recorded; R1 may differ (D-114) | Same |
| `build_scene`, `frame_at` | S17 | `scene_timing` natively, release | ≤ 1 s; mean ≤ 2 ms | D-166 | Same |
| `detect_scene` and `render_encode` in the browser | S17 | `pnpm bench:device`, `BENCH_CLIP=portrait`, 10 runs | Within 10% of V3's file for `detect_scene` plus the assembly (about 30 ms); `render_encode` ≤ 90 s per 60 s (E-4) | `render_encode` over 150 s: BP §6.2 row 2 | `bench/results/d1-<date>-portrait.json` |
| Golden similarity across adapters | S7, S16 | The D1-written goldens read on lavapipe in CI | ≥ 0.98 each | Per-adapter folders `fixtures/golden/{warp,lavapipe}/` and the reason recorded; the threshold is not lowered | `experiments.md`; TS §39.3 item 14 |
| `offcut_render.wasm` size | S5 | Before and after the plan crate | Recorded | Over 300 kB growth: find the crate responsible (`cargo bloat`-style size listing of the release build) and record it; `serde_json` is already out of the bundle (feature `prompt` off, D-122) | `v4changelog.md` |

### 21.2 BP §6.2's decisions

| If this misses on D1 | Do this | Decide at |
|---|---|---|
| Drift over 80 ms across 60 s, or more than 5% late frames, after the 360x640 rule | The TS §20 contingency: render a 360x640 preview file with the export loop into OPFS and play it in a `<video>` element; edits re-render it. A V4 decision recorded with the readings; it changes `control-preview.ts` and `preview-job.ts` only | S15 |
| Vello in a worker unreliable (a frame fails or differs between runs) | The TS §19 contingency: `Canvas2dBackend` drawing the same `DisplayList` (a `Video` command is `drawImage` of the frame's `src` into `dst`); separate goldens | S15 |

---

## 22. Hand-over: V5, V6 and V6.1 to V6.7

### 22.1 What V4 freezes

Contracts from the end of V4. Changing one later means touching both sides of a boundary; each V6.x release in section 22.2 is checked against this list.

| Frozen in V4 | Where |
|---|---|
| The plan's wire format, `PLAN_FORMAT = 1`: every field, every arm, the serde tagging | `offcut-types/src/plan.rs` (D-119, D-120) |
| `validate`'s signature, `ValidPlan`'s private construction, `ValidItem`'s fields, `DropReason`'s members (V6.x appends) | `offcut-plan/src/validate.rs` |
| The catalog's types (`ItemSpec`, `ParamSpec`, `ParamTy`, `AnchorRule`, `Producers`); rows are appended, `CATALOG_VERSION` bumped with each prompt change | `offcut-plan/src/catalog.rs` |
| `TimeMap`'s methods and serialization; `OutMs`, `OutSpan`, `SrcPos`; `FrameIdx::to_out_ms` | `offcut-types` |
| `SceneInput`'s fields (V6.x may append `assets`), `Scene`'s methods, `frame_at(OutMs)` | `offcut-scene/src/lib.rs` |
| `DrawCmd`'s variants and their order; `FontKey`; `CropRect`, `VideoSlot`, `ImageKey` | `display_list.rs` |
| `FontSet`'s keys 0-3 | `fonts.rs` |
| `Theme`'s fields and roles; `StyleSpec`'s fields; `CaptionMotion`'s fields | `theme.rs`, `styles.rs`, `motion.rs` |
| `Renderer::render(sources, list, fonts)`, `render_to_image`, `RenderError` | `offcut-render` |
| `assemble`'s signature and its identity postcondition | `offcut-dsp/src/program.rs` |
| `AudioWorkerApi` and `RenderWorkerApi` as in section 14.1 | `workers/protocol.ts` |
| `pool.cancel`, `pool.restartWorker`, `CANCEL_TIMEOUT` | `workers/pool.ts` |
| The review use-cases' update rule | `usecases/review-update.ts` |
| The `data-*` statistics names | `PreviewPlayer.tsx` (D-155) |
| The golden image names and the SSIM rule | `golden_frames.rs` |
| `EditState`'s fields, including `caption_position` and `highlight_numbers` (V6.x appends with serde defaults) | `offcut-types/src/edit.rs` |
| The break-cost model (costs may be tuned as values; the search and its hard breaks are fixed) | `readability.rs`, `captions.rs` |
| `Scene::hit` and `Hit` | `hit.rs`, `protocol.ts` |
| The design tokens' names (values may be tuned), the primitives' props, the editor's four tabs | `tokens.css`, `ui/primitives/`, `SidePanel.tsx` |
| The `review_action` action names | `offcut-api-types/src/analytics.rs` |

V2's and V3's freezes stand, except the rows D-125, D-126, D-127, D-130, D-132, D-133 and D-136 reopened, each named in section 2.

### 22.2 File by file: what each later version adds

Every cell is additive: a body, a file, an arm, a row. Where a later release touches a V4 file, the column says what, and nothing in it changes a signature frozen in 22.1.

| V4 file | V5 | V6 | V6.1 edit plan | V6.2 cuts | V6.3 music | V6.4 structure | V6.5 motion | V6.6 furniture | V6.7 brand |
|---|---|---|---|---|---|---|---|---|---|
| `offcut-types/plan.rs` | - | - | - | - | - | - | - | - | - |
| `offcut-types/timemap.rs` | - | - | - | `TimeMap::from_segments` (pub(crate)-style, called by `offcut-plan` only, lint-checked) | - | - | - | `Hold` used | - |
| `offcut-types/events.rs` (`EventKind`) | - | - | `Emphasis`, `IconPop`, `PunchIn`, `Emoji` (codes 4-7) | - | - | `ListBuilder`, `NumberMoment`, `QuoteCard`, `DefinitionCard`, `Versus`, `StepTracker`, `Timeline`, `Checklist` | `ShapeFrame`, `Transition` | `HookTitle`, `LowerThird`, `ChapterMarker`, `ProgressBar`, `EndCard` | - |
| `offcut-types/summary.rs`, `stage.rs` | - | - | `PipelineStage::Plan` | `pauses_shortened`, `time_removed` | `music` | - | - | `chapters` | - |
| `offcut-plan/catalog.rs` | - | - | 4 rows; version 2 | cut paragraph in the prompt; version 3 | music cue; version 4 | 8 rows | 2 rows | 5 rows | - |
| `offcut-plan/validate.rs` | - | - | `ItemParams` arms of the new kinds | V-7 replaces the cut drop | music cue kept | arms | arms | V-8 replaces the chapter drop | - |
| `offcut-plan/from_rules.rs` | - | - | - | - | - | may map onto new kinds | - | - | - |
| `offcut-plan/prompt.rs` | - | - | first caller (server) | - | - | - | - | - | - |
| `offcut-plan/timemap.rs` (new in V6.2) | - | - | - | `build(cuts, transcript, silence, end_card)` | - | - | - | end card | - |
| `offcut-scene/lib.rs` | - | - | `SceneInput.assets` | - | - | - | - | - | - |
| `offcut-scene/events/*` | - | - | 4 files | - | - | 8 files | 2 files | 5 files | - |
| `offcut-scene/video.rs` | - | - | punch-in keys | jump-cut alternation | - | - | masks, placements | - | - |
| `offcut-scene/styles.rs`, `motion.rs`, `captions_draw.rs` | - | - | - | - | - | - | preset rows, motion arms, glyph unit | - | - |
| `offcut-scene/theme.rs`, `fonts.rs` | - | - | - | - | - | - | - | - | `Theme` from a kit; `FontSet::with_user` |
| `offcut-scene/safe_area.rs` | - | - | - | - | - | - | - | `Top`, `LowerThird` rows | - |
| `offcut-scene/summary.rs` | - | - | - | cut counts | music | - | - | chapters | - |
| `offcut-render/composite.rs` | - | - | - | - | - | - | layers, several videos, masks | - | `Image` |
| `offcut-dsp/program.rs` | - | - | - | splices, cross-fades | mix, ducking, mix loudness | - | - | end-card tail | - |
| `offcut-wasm-render/scene_api.rs` | - | - | `set_scene` takes a model plan (same call) | non-identity maps accepted | - | - | - | - | brand kit input |
| `workers/protocol.ts` | - | - | - | - | - | - | - | - | - |
| `workers/render.worker.ts`, `render/*-job.ts` | export reads `prog48` (S-8) | - | - | - | - | - | - | - | - |
| `workers/audio.worker.ts` | - | - | - | - | music PCM input | - | - | - | - |
| `usecases/run-pipeline.ts` | `prog48.f32` written | - | `request-plan.ts` replaces the `rulesPlan` call (fallback to it) | map from the plan | music | - | - | - | - |
| `usecases/review-update.ts` and the four edit files | - | - | - | `restore-cut.ts`, `set-filler-cuts.ts` beside them | `set-music.ts` | - | `set-caption-preset.ts` | - | `set-brand-kit.ts` |
| `usecases/start-export.ts` | `prog48`, cache key (S-9), verifier | gate, sign-in | - | - | - | - | - | - | - |
| `ui/layout/SidePanel.tsx` and its tabs | - | - | a "Post" section (hook, caption, hashtags with copy buttons) | `CutsSection` filled (Captions or a Cuts tab) | a Music section | - | preset cards in Style | furniture rows in Visuals | a Brand section in Style |
| `ui/components/PlanItemList.tsx` | - | - | labels of 4 kinds | - | - | 8 labels | 2 | 5 | - |
| `ui/components/ErrorPanel.tsx` | export copy | auth copy | plan fallback notice | - | - | - | - | - | - |
| `state/clip-store.ts` | - | - | plan source, dropped counts | cut restores | music | - | preset | - | kit |
| `copy/messages.ts` | export | auth, plan | plan, settings switch | cuts | music | kinds | presets | furniture | kit |
| `tests-e2e/review-edit.spec.ts` | - | - | regenerate plan | restore a cut | music on/off | - | presets | - | kit |
| `golden_frames.rs` | - | - | +4 | jump zoom | - | +8 | +2 | +5 | kit matrix |
| `offcut-scene/readability.rs` | - | - | the model's own emphasis picks join the layers of D-169 as items | - | - | - | per-preset break limits | - | - |
| `offcut-scene/placement.rs` | - | - | - | - | - | - | - | zones for hook title and lower third | logo corner |
| `ui/primitives/`, `tokens.css` | - | - | - | - | - | - | - | - | brand colours shown in the Style tab only, never in the interface |
| `state/history.ts` | - | - | regenerate plan is one step | restore a cut is one step | music changes are steps | - | - | - | kit changes are steps |
| `usecases/preview-interact.ts` | - | - | - | - | - | - | drag a shape frame | drag a lower third | drag the logo |

### 22.3 What must be true on V5's first morning

- [x] V4 is tagged `v4`, merged into `main` and deployed; the deployed page previews the reference clip in all three styles.
- [x] `docs/v4/experiments.md` holds section 21's readings and BP §6.2's decisions.
- [x] The 16 goldens and the snapshots are committed and pass in CI on lavapipe.
- [x] `program_identity.rs` passes: V5 can switch the export to `prog48` (S-8) knowing it equals `out48`.
- [x] `git grep -nE "\bV5\b" -- web/src crates` lists the places V5 fills: the export's audio source, the render cache, the verifier's checks 7-11, `ChangeSummary` on screen.
- [x] No `TODO` without a version tag.
- [x] Section 24's copy-back is applied.

### 22.4 What V5 to V6.7 inherit without further work

- A scene that reads a validated plan: V6.1's model plan takes the same road as the rules plan (`setScene`).
- A prompt and a schema generated from the catalog: V6.1 adds a route that sends them.
- Program time everywhere: V6.2's cuts change the map's data and `assemble`'s body; the preview, the scrub bar, the export loop (from V5) and the scene follow.
- Program audio on the page: V6.3's music changes `assemble`'s body.
- A per-frame video command: punch-ins (V6.1), jump-cut zooms (V6.2), shapes and transitions (V6.5) are data in `video.rs` and the compositor's body.
- A font registry and a theme: the brand kit (V6.7) is a `Theme` and three font files.
- A review panel built from plan items and a generic update rule: every V6.x control is one use-case and one section.
- A design system, primitives, an undo history and direct manipulation: a V6.x control is designed once in the tokens and gets undo, autosave and keyboard access for free.

---

## 23. Exit checklist

V4 is done when every box is ticked (BP §1.2, BP §6.3, TS §28 M1.4-M1.5, TS §40.9 S-1 to S-7).

**Code and checks**

- [x] `pnpm check` green: fmt, clippy, deny, ESLint, both `tsc` programs, `check-file-tree`, `check-gen-clean`, `check-copy-codes`.
- [x] `pnpm fixtures` then `pnpm test` green on D1 and in CI.
- [x] BP §6.3: `display_list_snapshots.rs`, `layout_safe_area.rs`, `framing.rs`, `determinism.rs` (scene), `golden_frames.rs` pass.
- [x] BP §6.3: `pipeline-preview.spec.ts` (all cases), `preview-sync.spec.ts`, `review-edit.spec.ts`, `cancel.spec.ts` pass on D1 on a keyed build.
- [x] BP §6.3: `rpc.test.ts`, `clip-machine.test.ts`, `model-machine.test.ts`, `run-pipeline.test.ts` pass.
- [x] BP §6.3 (v1.2): `timemap_identity.rs`, `plan_from_rules.rs`, `validate.rs`, `catalog_snapshot.rs`, `catalog_complete.rs`, `styles_use_theme.rs`, `program_identity.rs` pass; the scene takes no `events` argument; no `TimeMs` reaches `frame_at`; the renderer has no crop parameter.
- [x] The identity proof of D-117 passed at S6 (its record in the changelog), and the temporary test is gone.
- [x] Caption intelligence: `line_breaks.rs` (the table unchanged since its commit, or each change logged), `placement.rs`, `parity.rs`, `scene_cache.rs`, `hit.rs` pass.
- [ ] The editor: every state of section 17.2 is reachable and matches the reviewed screens; the axe check passes; every control works by keyboard.
- [x] No source file over 700 lines, tests excluded; the four splits of D-150 are in place.
- [x] `git grep` finds no `fetch(` outside the four files, no network call added (D-164), and no cut, mix or non-identity map constructed outside tests.
- [x] Test counts at least the baseline's plus V4's (section 20.1); none fell except the cases D-160 names.

**Measurements**

- [x] Edit to preview ≤ 150 ms (90th percentile) on D1; style thumbnails ≤ 400 ms; the app shell within 400 kB gzip with the font.
- [x] Preview drift ≤ 80 ms over 60 s and late frames ≤ 5% on D1; first frame ≤ 500 ms; seek ≤ 300 ms (or recorded with the spinner).
- [x] `build_scene` ≤ 1 s and mean `frame_at` ≤ 2 ms natively.
- [x] The bench file of S15 committed; `render_encode` within E-4's line.
- [x] Golden similarity across adapters recorded.
- [x] BP §6.2's two decisions recorded.

**Deployment**

- [x] `ci.yml` green on `main`; API (unchanged image) then web deployed; header check and `@smoke` pass; `crossOriginIsolated === true`.
- [x] On the deployed page on D1: the reference clip previews in the premium default; Bold and Mono switch from their cards; clicking a word on the preview and editing it changes the number at once; "Change all" and undo work; dragging the captions to the top moves the visuals below them; a list switched off disappears; the landscape clip's crop moves by dragging; scrubbing works; cancel during transcription returns to "Process again".
- [x] `review_action`, `job_cancelled` and `client_error` rows appear in `analytics_events` with enum props only (the founder's SQL).

**Records**

- [x] `docs/v4/experiments.md` written; every reading labelled `d1`.
- [ ] Design review 1 (S10A) and design review 2 (S19) done by the founder on the captured screens at 1280 x 800 and 1920 x 1080, with the visual QA list of D-191 ticked; the founder has looked at five golden frames chosen at random and said whether each looks as intended.
- [x] The founder has read ten rows of `line_breaks.rs`'s table and said whether each break reads naturally.
- [x] Every (assumption) V4 measured is replaced in TS §39.3, or still listed.
- [x] `v4changelog.md` lists every pinned version (`insta`, `png`, the lavapipe package), every snapshot or golden change after its first commit, and the identity proof.
- [x] Section 22.3 fully ticked.
- [x] Tagged `v4`.

---

## 24. Open references, spec issues found, decisions to copy back

| # | Issue | Where | Handled by |
|---|---|---|---|
| 1 | TS §40.9 S-3 places number resolution at scene build; one place should parse numbers for the screen, and the scene should not parse | TS §40.9 | D-121: in `validate`. Copy back |
| 2 | TS §40.9 says nothing beyond the seams is reserved; the plan's wire format is a contract from V6.1 on and is cheaper fixed once | TS §40.9 | D-119, D-163. Copy back: "except the plan's wire format, fixed at `PLAN_FORMAT = 1` in V4" |
| 3 | TS §40.5 gives `Video.dst` as an `Affine` | TS §40.5 | D-130: an axis-aligned `Rect`. Copy back |
| 4 | TS §40.5's `Theme` has no font for code-style captions | TS §40.5 | D-136: `mono_font`. Copy back |
| 5 | TS §2's `offcut-plan` row omits `offcut-text`; TS §7's graph has no `offcut-plan` | TS §2, §7 | D-122. Applied at S1 (the checks need it) |
| 6 | A `Detected` FromTo has no time for its second value; `EventParams` is frozen | TS §19.5 vs §10.5 | Section 8.7: `ValidItem.marks`. Copy back to TS §40.4 |
| 7 | `AudioWorkerApi` is "frozen" (V3 §21.1) and TS §40.6 adds `assemble` | V4 CL known issue 1 | D-127, D-128. Copy back TS §14.2 |
| 8 | TS §19.2, §20.2, §21.3 show v1.1 signatures | V4 CL known issue 2 | S16 rewrites them from the code |
| 9 | `EventToggleList.tsx` in TS §5 and BP §6.1 is `PlanItemList.tsx` | TS §5 | Applied at S1 |
| 10 | The empty-output retry on WASM (TS §16.6) was listed for V4 by V2 §25.2 but not by BP §6 | V2 §25.2, BP §6 | Not in V4; V7 (failure recovery). Copy back to BP §9 |
| 11 | Goldens across two software adapters may differ above SSIM's tolerance | D-135 | Section 21.1: per-adapter folders if so |
| 12 | The watermark's bottom-left anchor sits at the caption zone's bottom edge | TS §19.1 | D-143: a test proves no overlap; alpha (PS §19) may move it |
| 13 | `review-edit.spec.ts` needs the visible caption text; reading pixels for words is not reliable | TS §27 | Section 20.4: a dev/test-only `data-caption-text` attribute; not in production |
| 14 | V3 §23 item 25 (audio kept in the worker after an early end) | V3 §23 | Closed by D-128 |
| 15 | V3 §23 item 24 (`video-source.ts` has no room) | V3 §23 | Closed by D-150 |
| 16 | The audience is content creators in general, not only technical creators | PS §3, §4, §6 (ICP, persona, value proposition) | D-177; the founder's decision of 2026-10-22. Copy back at S18: the primary ICP becomes short-form creators who film talking-head clips; technical creators stay the first channel; the persona gains a non-technical creator |
| 17 | TS §19.4's style table is V1's; V4 redesigns Clean and Bold and renames Tech's label | TS §19.4 | D-170. Copy back |
| 18 | `EditState` (a V1 type, TS §10.6) gains `caption_position` and `highlight_numbers` | TS §10.6 | D-169, D-171. Copy back; V3's stored records read with defaults |
| 19 | TS §5 lists `ReviewPanel.tsx` and `EventToggleList.tsx`; the editor of section 17 has `SidePanel` and its tabs | TS §5, §13.3 | Section 4. TS §5 and §13.3's component table updated at S1 |
| 20 | `tokens.css` says "system fonts only"; TS §36 bans CDN fonts, not self-hosted ones | `tokens.css`, TS §36 | D-180. TS §36's row clarified at S18 |
| 21 | No per-word confidence to flag doubtful words (TE-2) | TS §16.4, §39.3 item 7 | D-176: no flag in V4; a V6.1 catalog row or V9 |
| 22 | Automatic face-aware caption placement | PS §9.5 (face tracking, P1) | D-171: drag and snaps in V4; the face model when face tracking ships |

### 24.1 Cross-check: BP §6 items against sections and tests

| BP §6 item | Section | Test | Exit box |
|---|---|---|---|
| A `styles.rs` Clean, Bold, Tech | 8.4, 8.5 | snapshots, goldens | Code and checks |
| A `list_reveal.rs`, `from_to.rs`, `keyword_pop.rs`, `number_reveal.rs` | 8.7 | snapshots, goldens, `catalog_complete.rs` | Code and checks |
| A `framing.rs` complete | 8.8 | `framing.rs` | Code and checks |
| A `watermark.rs`, `summary.rs` | 8.8 | snapshots, `layout_safe_area.rs` | Code and checks |
| A tests and 14 goldens | 8.9, 9.4 | 16 goldens | Code and checks |
| A2 seams S-1 to S-7 | 6, 7, 8, 9, 10, 14, 16, 17 | section 20.2 | Code and checks (v1.2 box) |
| B preview loop, video source, control-preview, player | 15.2, 15.3, 16.2, 17.1 | `preview-sync.spec.ts` | Code and checks; Measurements |
| C review UI and use-cases, `NO_EVENTS_FOUND` | 16.2, 17 | `review-edit.spec.ts`, `pipeline-preview.spec.ts` | Code and checks |
| D rpc, pool, cancel, ErrorPanel, machine tests, E2E | 14.2, 14.3, 16.2, 17.1, 20 | `rpc.test.ts`, `cancel.spec.ts`, machine tests | Code and checks |
| 6.2 decisions | 21.2 | - | Measurements |

### 24.2 Decisions to copy back (S18)

D-167 to D-171 (captions, placement, `EditState` fields, the style table), D-177 (audience, to PS), D-180 (fonts, TS §36), D-181 to D-187 (TS §13.3's component table), D-119 (wire format fixed), D-121 (resolution in `validate`), D-122 (graph), D-125 (`to_out_ms`), D-127 (`assemble`, `release` in TS §14.2), D-130 (`dst` rectangle), D-133 (renderer contract and its V4 support rule), D-136 (`mono_font`), section 8.7 (`marks`), D-150 (the split files in TS §5), D-152 (`CANCEL_TIMEOUT` measured or kept), D-165 (plan limits in TS §40.4), every (assumption) measured in section 21.

### 24.3 As built (S18 copy-back)

What changed while V4 was built, decision by decision, from `docs/v4/v4changelog.md` (its known issues 23, 24, 27, 28, 32, 34 and 35 are the lists this section applies; each is closed by Entry 38). Where this section and a row above disagree, this section is V4 as built.

| Decision or section | As built | Entry |
|---|---|---|
| D-130 | `Video.dst` is an axis-aligned `Rect` in output pixels; the list's scaling leaves it alone | 8 |
| D-132 | `FontSet::embedded()` has five keys: Inter 700, Inter 900, JetBrains Mono 700, Noto Emoji, Inter 800 (appended, D-170) | 16 |
| D-133 | The support rule as `composite.rs` checks it: exactly one `Video`, the first command, `Main`, opaque, no mask, `src` inside the frame and `dst` inside the canvas, no `Image` | 8 |
| D-140 | The fit applies whenever the lines do not fit the 320 lp zone, width included | 17 |
| D-143 | The watermark's box sits on the safe area's bottom, about 10.2 lp high, in its own `WATERMARK_BAND` (56 lp, the founder's choice) under the captions | 17 |
| D-163 | `FontRole::Body` is the watermark's font | 17 |
| D-171 | `CAPTION_BOTTOM` = the safe area's bottom − `WATERMARK_BAND` 56 − 140 = 1,304 lp; the track runs from 390 to 1,304 (`CAPTION_TRACK` of `preview-interact.ts`), snaps Top 0.15, Middle 0.55, Bottom 1.0 (`CAPTION_SNAPS`) | 17, 24 |
| §8.10 | `ok_portrait_3x4` (a 3:4 frame) loses its sides, as a portrait clip does | 17 |
| D-172 | The Free badge shows unless the cached token's plan is Creator | 24 |
| D-174, §14.1 | `Hit` carries `rect` (the word's glyph box, or the caption block); `RenderWorkerApi` gains `captionMirror` (development and test builds); `SceneReport.chunks` (`SceneChunk`) from `Scene::chunk_ranges` | 24, 26 |
| D-179, D-181, §17.6, §4 | `ui/styles/controls.module.css` (Tabs, Switch, SegmentedControl, Slider); the primary button is filled with `--brand-hover` (AA contrast, open item 8); tabs share the list's width with the icon above the label; a control disabled with a reason is `aria-disabled` and focusable; a toast has no live region of its own (the host puts it in D-192's region); `preview.module.css` from S10 | 19, 20, 22 |
| D-182, §16.1, §4 | `clip-store.chunks`; `CaptionWordControls.tsx` beside `CaptionEditor.tsx` | 26 |
| D-183, §4, §17.3 | `PreviewGestures.tsx` holds the pointer handling of `PreviewPlayer` | 24 |
| D-184, §16.2 | `review-update.ts` keeps a queue of one; `update` takes `ReviewDetail` and an optional `joinsLastStep`: "Change all" right after the edit it was offered for joins that edit's undo step (D-175) | 25, 27, 34 |
| D-185, §16.1, §16.2 | `ui-store.shortcuts`; a queued toast's `key`; `control-preview`'s `togglePlay`, `stepFrame` (a frame's time rounded up), `stepSecond` | 28 |
| D-186, §15.1, §16.2 | `styleThumbnails` draws at the preview profile; the first thumbnail call is not debounced | 27 |
| §16.2 | A pause ends with a still frame of the paused time (`previewSeek` to where the sound stopped); `cropMoves` in `clip-store.ts`; `cachedPlan` in `start-export.ts` | 23, 24 |
| §14.3, §16.2 | `pool.ts`'s `whenFree`: a render call that must not fail waits while the worker refuses it as busy (every 16 ms, up to 2 s) — the update rule, the seek, the play, the pause; `whenReleased`: a worker just ended still holds its OPFS files a moment | 28, 30 |
| D-152, §16.1, §16.2 | `cancel-job.ts` also keeps the shown failures (`failureShown`, one `client_error` per failure); the export's `cancel` and `cancel_done` go through the machine's table in `start-export.ts`; a cancelled import goes `importing → failed → idle` in one moment; `clip-store.cancelled`, `failureDuring`, `cancelledRun`, `retried`, `cancelledImport`, `clearFailure`; `preview-store.canvas`. `CANCEL_TIMEOUT_MS = 2000` kept: a hung render worker's run is back at `accepted` 2.4 to 2.5 s after Cancel | 30, 35 |
| §17.3, D-189, D-190 | The failed export's card is in the side panel with "Try again" and "Close"; the success state sits beside the Export button; the pill's "on your computer" line is the bar's description; §17.2's rejected card keeps "Start over"; `editor.export` is `export.button`; `DropZone` has `landing` and `stage` variants; G 14.5: after a render crash the panel shows `E_WORKER_CRASH`'s words | 20, 30 |
| §19.2, D-155 | `vite.config.ts`'s `define` `__OFFCUT_DEV_FAULTS__`; the dev faults' worker halves in `rpc-worker.ts`; `usecases/dev-faults.ts`; the media suites run on D1 on `pnpm build:e2e` (a `--mode test` production bundle with the test key), the only build with the caption mirrors and `__offcutDev` | 30, 33 |
| §16.2 (control-preview) | The page keeps one `AudioContext`, made at the page's first play (a click), suspended at `detach`; the clip's buffer is a context-free `AudioBuffer`. A context per clip at `ready` was a 250-330 ms main-thread task (D-156); one made when the editor opened slowed every page of CI's runner, which has no sound device | 35, 37 |
| §15.3 (render worker) | `drawStill` prefetches the frame when the scene is set before a canvas exists, so the first frame after `ready` finds it decoded (50-210 ms where 620 ms was seen) | 35 |
| §20.1, §21.1 | The edit-to-preview reading ends at "Saving…" (after `setScene`'s answer, which follows the redraw), not a `data-frames` tick; `golden_frames` has 19 tests, 17 of them on the adapter | 33, 36 |
| §20.4, G 16.1, G 16.4-16.7 | The helpers as written (`toggleItem(page, label, on)`, `dragVideo(page, dx)`, `trackSaves`/`settled(page, since)`, `captionMirror`, `transcriptWords`, `routeAssets`'s `holdAfterBytes`); INV-11 after a cancel is read through the page (no `client_error`; a play and a pause draw); the cancel timeout by its time and the restarted worker's run; the reference clip is wider than 9:16 (the portrait fixture is review-edit case 6's clip); "78 passed, 1 skipped" is 77 passed and the D-78 skip of the 78 listed | 33, 34, 35 |
| §19.2, §19.4 (scripts, CI) | `check-hosts.mjs`'s third check fails a build holding `__offcutDev` or a `data-caption-*` marker; `ci.yml`'s step "No native GPU backend outside the tests" reads the wasm32 and host normal trees of `offcut-render`; the guard drills of G 17.3 ran locally (the founder's choice), the Mesa one in an `ubuntu:24.04` container; the Linux ffmpeg is BtbN's month-end build of 2026-09-30 (BtbN keeps a daily build about two weeks, a month's last build about two years) | 35, 36 |
| §23 | "No source file over 700 lines": the founder raised the limit from 400 on 2026-10-28 (`MAX_LINES` of `check-file-tree.mjs`) | 28 |

---

## Appendix A. Per-file specifications in brief (BP §1.3, TS §34.2)

Each file below gets a per-file spec from `docs/file-specs/TEMPLATE.md` before it is written; the implementer is given that spec, TS §2, §7, §10, §11, §40, this document's section for the file, and the signatures of the files it may import.

| File | Purpose | Inputs | Outputs | Must not |
|---|---|---|---|---|
| `offcut-types/src/timemap.rs` | Recording ↔ program time | durations, spans | `TimeMap`, `SrcPos` | construct a non-identity map outside `from_segments` (V6.2) |
| `offcut-types/src/plan.rs` | The plan's wire format | - | types | hold logic |
| `offcut-plan/src/catalog.rs` | What the renderer can draw | - | `ItemSpec`s | name a kind the scene does not draw |
| `offcut-plan/src/prompt.rs` | The model's instructions | catalog, transcript | strings, schema | include user edits; include a number from the transcript as an instruction |
| `offcut-plan/src/validate.rs` | The only door to `ValidPlan` | plan, transcript, edits | `ValidPlan`, drops | panic; keep an item that fails a rule |
| `offcut-plan/src/resolve.rs` | Numbers from words | tokens, ranges | `Quantity` | accept a range that is not exactly a number |
| `offcut-plan/src/from_rules.rs` | Events → plan | events | `EditPlan` | change a detector value |
| `offcut-scene/src/lib.rs` | Scene from a valid plan | `SceneInput` | `Scene` | parse a number; move a time except through the map |
| `offcut-scene/src/theme.rs` | Colours and font roles | style | `Theme` | be read by name from another file than `styles.rs`, `captions_draw.rs`, `events/*`, `watermark.rs` |
| `offcut-scene/src/video.rs` | The per-frame video command | crop | `DrawCmd::Video` | crop anywhere else |
| `offcut-scene/src/events/list_reveal.rs` | ListReveal visual | item, map, style | track, commands | shape text in `draw` |
| `offcut-scene/src/events/from_to.rs` | FromTo visual | item (with `marks`), map | track, commands | tokenize |
| `offcut-scene/src/watermark.rs` | The watermark | theme, fonts | commands | read anything but the profile flag (INV-9) |
| `offcut-render/src/composite.rs` | List-order compositing | list, sources, fonts | pixels | close a `VideoFrame` |
| `offcut-dsp/src/program.rs` | Program audio | `out48`, map | `ProgramAudio` | change length in V4 |
| `offcut-wasm-render/src/scene_api.rs` | Scene exports | JS values | report, positions | build a scene from an unvalidated plan |
| `web/src/workers/rpc-worker.ts` | Worker side of RPC | messages | answers, throttled progress | drop a feed line |
| `web/src/workers/render/preview-job.ts` | Play, pause, seek, clock, resize | session, source | frames, statistics | read the wall clock for time on the clip |
| `web/src/workers/render/frame-decoder.ts` | The `VideoDecoder` | samples | frames | close a frame it handed out |
| `web/src/usecases/review-update.ts` | The edit update rule | change | stores, scene | await the network |
| `web/src/usecases/cancel-job.ts` | C-12 | scope | stores, pool | leave a temporary file |
| `web/src/ui/components/CaptionEditor.tsx` | The transcript editor (D-182) | clip, preview and ui stores | `editWord`, `replaceAll`, `seek` | show a timeline; commit on blur without Enter or Tab |
| `offcut-scene/src/readability.rs` | Break costs and lists | words | costs | hold anything but English lists and constants |
| `offcut-scene/src/placement.rs` | Caption block and event zone | placement, style | rectangles | read the watermark flag (preview and export must agree) |
| `offcut-scene/src/hit.rs` | What a point on the canvas is | scene, time, point | `Hit` | shape text |
| `web/src/ui/styles/tokens.css` | Every design value | - | custom properties | be bypassed by a literal colour or size in a module |
| `web/src/ui/primitives/*` | The controls | props | elements | hold product logic; render text not from `messages.ts` |
| `web/src/usecases/preview-interact.ts` | Gestures on the preview | pointer positions | hit tests, commits | commit before release |
| `web/src/state/history.ts` | Undo and redo | snapshots | snapshots | hold more than 50 |
| `web/src/ui/components/ErrorPanel.tsx` | Failures | stores | retry calls, `client_error` | show a code or a file name |

---

## Appendix B. Invariants V4 touches, and where each is enforced

| # | Invariant | V4 enforcement |
|---|---|---|
| INV-5 | `TIMELINE_INTEGRITY` (v1.2) | Every map is the identity: `scene_api` refuses another (D-164); `timemap_identity.rs`; V3's duration tests unchanged |
| INV-7 | Equal inputs give byte-identical display lists | `determinism.rs` |
| INV-9 | Size and watermark from the token only | `watermark.rs` reads `profile.watermark` only; snapshots on and off |
| INV-10 | The chain preserves duration; only `assemble` may change it | `program_identity.rs`; V3's `duration_preserved.rs` |
| INV-11 | Every `VideoFrame` closed | The live-frame counter after preview, seek and cancel (`preview-sync.spec.ts`, `cancel.spec.ts`) |
| INV-14 | No upgrade prompt during editing | No such component exists in V4 |
| INV-15 | All copy from `messages.ts` | `messages.test.ts`, `react/jsx-no-literals`, `check-copy-codes.mjs` |
| INV-16 | Every state change through `transition()` | The machine tests; the edit queue of D-147 |
| INV-17 | No media work on the main thread | The long-task check (D-156); boundaries lint |
| INV-21 | Only `validate` builds a `ValidPlan` | `tests/ui.rs` (trybuild) |
| INV-22 | Every drawn number is parsed from the effective transcript | `validate.rs` V-5 cases; the word-edit snapshot; `review-edit.spec.ts` "edit a number" |
| INV-23 | Catalog kinds and drawn kinds are the same set | `catalog_complete.rs`; `catalog_snapshot.rs` |
| INV-15 (editor) | All copy from `messages.ts`, including tooltips, aria labels and toasts | `react/jsx-no-literals`; `check-copy-codes.mjs` |
| INV-24 | A plan failure never blocks a clip | V4: the rules plan is the only producer and `validate` never rejects it (`validate_rules_plans.rs`); V6.1 adds the fallback tests |

---

## Appendix C. Two per-file specifications written out

### C.1 `crates/offcut-plan/src/validate.rs`

- **Purpose.** Turn any `EditPlan` into a `ValidPlan` that the scene can draw without a further check, and say what was dropped.
- **Public API.** `validate`, `ValidPlan` (with `source`, `items`, `is_empty`), `ValidItem`, `Dropped`, `DropReason`, `PlanRejected`, `RejectReason` (section 7.6).
- **Allowed imports.** `offcut-types` (plan, transcript, events, units, ids), `crate::{catalog, config, resolve}`, `std::collections::BTreeMap`.
- **Algorithm.** Section 7.6, steps 1-6, in that order; a rule never mutates the input plan, it builds the output.
- **Errors.** `PlanRejected` for V-1 only; every other failure is a `Dropped`. No panic, no `unwrap`, no indexing.
- **Invariants.** INV-21 (no other constructor), INV-22 (V-5), D-164 (non-item fields dropped in V4).
- **Tests.** `tests/validate.rs` (one case per rule and reason), `tests/validate_rules_plans.rs`, `tests/ui.rs`.
- **Size.** Under 400 lines; the per-kind parameter checks go to a private `validate_params.rs` if needed (listed in the TS §5 tree at S1 as an allowed split).

### C.2 `web/src/usecases/review-update.ts`

- **Purpose.** One rule for every review change: pause, apply optimistically, rebuild the scene (and the plan when words changed), restore the preview, persist, report.
- **Public API.** `update(change: (edit: EditState) => EditState, mode: "plan" | "scene", action: ReviewAction): Promise<void>`.
- **Allowed imports.** `state/clip-store`, `state/preview-store`, `persistence/edits-repo`, `workers/pool`, `analytics/client`, `usecases/control-preview` (one new use-case edge, added to `eslint.config.js` with its reason, as D-58 added two).
- **Algorithm.** 1. If the clip is not `ready`, return (the panel is disabled then). 2. `previous = edit`; `next = change(previous)`; `clipStore.editStarted(next)` (`ready → updating`). 3. If the preview is `playing`, `controlPreview.pause()` and remember to play. 4. Mode `plan`: `events = pool.render.redetectSentence(...)`, `plan = pool.render.rulesPlan(...)`, `clipStore.setEvents`, `clipStore.setPlan`. 5. `report = pool.render.setScene({ transcript, plan, edit: next, timeMap, profile: "preview" })`. 6. `editsRepo.put(clipId, { editState: next })`. 7. `clipStore.editDone()`; play again if remembered. 8. `track(review_action)`. Any failure in 4-6: `clipStore.editFailed(previous, failure)`, `client_error`, and the scene is not rebuilt (the preview keeps the last good frame).
- **Errors.** Worker failures are stored, not thrown; a fault of this file is thrown.
- **Tests.** Through `run-pipeline.test.ts`'s fake pool (a `review-update` block) and `review-edit.spec.ts`.

---

## Appendix D. Named constants V4 introduces

| Constant | Value | File | Status |
|---|---|---|---|
| `PLAN_FORMAT` | 1 | `offcut-types/src/plan.rs` | Fixed (D-119) |
| `CATALOG_VERSION` | 1 | `offcut-plan/src/catalog.rs` | Bumped with each prompt change |
| Per-kind `max_per_clip`, `min_gap`, label limits | Section 7.2 | `offcut-plan/src/config.rs` | (assumption, V6.1, E-11) |
| `PAUSE_MARK_MS` | 300 | `offcut-plan/src/prompt.rs` | (assumption, V6.1) |
| `MIN_KEEP_GAP`, `MAX_CUT_SHARE`, `MAX_CHAPTERS` | 150 ms, 40%, 6 | `offcut-plan/src/config.rs` | Unused until V6.2, V6.6 (TS §40.4) |
| Theme colours | Section 8.4 | `offcut-scene/src/theme.rs` | (assumption, E-2) |
| Motion values | D-137 | `offcut-scene/src/styles.rs` | (assumption, E-2) |
| Bold pop 1.12, KeywordPop 1.25/1.10/180 ms | D-139 | `styles.rs`, `captions_draw.rs` | (assumption, E-2) |
| Tech panel radius 24, padding 32, cursor 0.6 em | D-139 | `styles.rs` | (assumption, E-2) |
| ListReveal sizes, gap, rise, hold | D-140 | `events/list_reveal.rs` | (assumption, E-2) |
| FromTo value 96/56 lp, bar 180 lp, min 8%, "to" scale 1.15 | D-141 | `events/from_to.rs` | (assumption, E-2) |
| `WATERMARK_TEXT`, 36 lp, 60%, stroke 3, baseline offset 6 | D-143 | `watermark.rs` | (assumption, alpha) |
| `GOLDEN_SSIM_MIN` | 0.98 | `golden_frames.rs` | (assumption, M1.4) |
| `CANCEL_TIMEOUT_MS` | 2,000 | `workers/pool.ts` | (assumption, M1.5) |
| `PROGRESS_MIN_INTERVAL_MS` | 100 | `workers/rpc-worker.ts` | TS §14.3 |
| `DOWNSCALE_MEAN_MS`, `DOWNSCALE_WINDOW` | 28, 60 frames | `render/preview-job.ts` | (assumption, M1.5) |
| `SEEK_SPINNER_MS` | 300 | `PreviewPlayer.tsx` | (assumption, M1.5) |
| `LONG_TASK_MAX_MS` | 100 (processing), 50 (editing, D-193) | `pipeline-preview.spec.ts`, `review-edit.spec.ts` | (assumption, M2.4) |
| Break costs 0 / 1 / 3 / 12, one-word chunk 8, imbalance threshold 1/3 | D-167 | `readability.rs` | (assumption, E-2) |
| `MIN_CHUNK_MS` | 700 | `readability.rs` | (assumption, E-2) |
| `ISOLATED_WORD_PAUSE_MS` | 250 | `readability.rs` | (assumption, E-2) |
| `CAPTION_TOP`, `CAPTION_BOTTOM`, snaps 0.15 / 0.55 / 1.0, event-zone gap 40 lp | D-171 | `placement.rs` | (assumption, E-2) |
| Clean: Inter 800, 72 lp, 16 x 2, 4 words, stroke 5, shadow 0 x 4 at 45%, pill radius 14, padding 12 x 4, `PILL_SLIDE_MS` 90 | D-170 | `styles.rs`, `theme.rs`, `captions_draw.rs` | (assumption, E-2) |
| `SHAPE_CACHE_ENTRIES` | 2,048 | `layout.rs` | (assumption, D-173) |
| `EDIT_TO_PREVIEW_MAX_MS` | 150 | `preview-sync.spec.ts` | TS §13.5 (assumption, M1.5) |
| `HISTORY_MAX` | 50 | `state/history.ts` | (assumption) |
| `THUMB_DEBOUNCE_MS`, thumbnail size | 300, 216 x 384 | `style-thumbnails.ts` | (assumption, D-186) |
| `SNAP_DISTANCE_PX` | 16 | `preview-interact.ts` | (assumption, design review) |
| Design tokens | D-179 | `tokens.css` | (assumption, design review D-191) |
