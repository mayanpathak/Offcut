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
| 6 | **Merge `v4-spec` into `main`** (spec v1.2 and `v4implementation.md`; docs only, a push to `main` deploys). V4's 1A item 2: nothing is built before it | Founder | Before V4 S1 |
| 7 | **Confirm the push rule for V4** (D-115: the agent pushes each prompt's commit to `v4-build`, as in V3), or change it | Founder | Before V4 S1 |

## Known issues
| # | Issue | Where | Version |
|---|---|---|---|
| 1 | `AudioWorkerApi` (TS §14.2) is described as frozen, and v1.2 adds `assemble` to it in V4 (TS §40.6, §40.9 S-6). V4's implementation plan must record the protocol change as a REOPENED contract, as V3 did for `FeedLine` | TS §14.2 | V4 |
| 2 | TS §19.2, §20.2 and §21.3 still show the v1.1 signatures (`events` in `SceneInput`, `crop()`, `render(frame, crop, rotation, list)`, `to_time_ms`). Each section now opens with a v1.2 note pointing to §40.5 / §40.9; V4's copy-back rewrites the signatures themselves when the code exists | TS §19-§21 | V4 |

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

