# buildplan.md - Offcut Build Plan (V1 to V10, with V6.1 to V6.11)

**Status:** v1.5. Built from `product.md` (PS, Product Spec v1.5) and `technicalspec.md` (TS, Technical Spec v1.5).

**What this document is.** The order in which Offcut gets built, as ten versions plus eleven point releases. Each version is a tagged, deployed, demoable state of the product. V1 is a working skeleton. V2 to V6 add the MVP product and, from v1.2, **the seams the AI edit plan needs** (TS §40.9), so that nothing written in V4-V6 has to be rewritten later. V6.1 to V6.11 add the long path and the AI-planned editor (TS §40.10, §41-§53, §8A below), and V6.11 ends with the feature freeze. V7 to V10 add no features: they are testing, hardening and the launch gate.

**v1.5 in one paragraph (founder decisions of 2026-11-09).** The product survives only if its shorts match or beat Submagic's. The benchmark's look becomes rules inside the releases that already hold what each needs, and the numbers stay: motion, caption polish and the hook title in V6.2; following the speaker in V6.5; V6.6 built as **V6.6a "Picture"** (blur, automatic colour and light) and **V6.6b "Graphics"** (all of v1.3's V6.6 but the hook title, plus fonts and presets); synthesized sound effects and voice tuning in V6.7; zoom transitions and a quality check in V6.8. An Enhance panel lets a person switch any enhancement off. The founding release moves to after V6.8, behind a blind test against Submagic (E-16). HDR stays rejected; music is the person's own track. About 30 more prompts (§8A.0).

**v1.3 in one paragraph (founder decisions of 2026-11-02).** V6 comes first, then V6.1-V6.11, then V7-V10. A long video up to 20 minutes (v1.4: 45) becomes 5-10 finished shorts, built on a "virtual clip" design that leaves everything downstream unchanged. **A clip of at most 90 seconds with new features off stays V5, byte for byte: TS §41, the compatibility contract, binds every release.** The old V6.x are renumbered (table below; TS §42), so older documents and code comments are read through that map. Chapter markers and the lower third moved post-launch.

**v1.2 in one paragraph (founder decision of 2026-10-21).** V4, V5 and V6 are written for the edit-plan design from the start: the scene reads an `EditPlan` (the rule detector is its first producer), time runs through an identity `TimeMap`, the video is a draw command, fonts and colours come from a registry and a theme, and the audio the preview plays is assembled program audio. V6.2 then adds the cloud LLM producer, and each later point release adds drawing code and data, not new contracts.

**What this document is not.** It defines no feature, no number and no file that the two specs do not already define. Where this plan and a spec disagree, the spec wins and this plan gets fixed. "PS §n" is a product-spec section, "TS §n" a tech-spec section. Step ids such as `M1.3` and experiment ids such as `E-3` and `TE-5` are the specs' own (TS §28, TS §37).

---

## 0. The ten versions at a glance

**The design release (2026-11-05).** Between V6 and the cloud-transcription release that comes before V6.1: a new landing page (animated, every working feature named, everything after V6 marked "Coming soon", "Start free" into the app), seven product clips made with Remotion, the other pages on the dark system, and lower Bottom captions (DC-13) on a pull request of their own. Plan `docs/design/designimplementation.md`, record `docs/design/designchangelog.md`, tag `design`.

**The cloud transcription release (2026-11-06).** Between the design release and V6.1, built as a version of its own (plan, guide, prompts P161-P170 and record in `docs/cloud_transcription/`, decisions D-336 to D-382, declared change DC-14, tag `cloud`). A signed-in person's clips are transcribed by Groq's `whisper-large-v3-turbo` by default, chosen in a card in the drop zone ("Cloud · fast" or "On device · private"): the clip's 16 kHz sound is compressed in the browser to Ogg Opus (about 0.32 MB for 90 s), sent once through `POST /api/v1/transcribe` on our API, and the words come back; nothing is stored. Any failure finishes the clip on the device. Signed out, or with the card on device, the product is today's. Two kill switches: `TRANSCRIBE_ENABLED` on Render and the build flag `cloud_transcribe`. Measured on D1: about 13× faster than the device on a minute of speech, the same visual events and one more (TE-C3, TE-C4).

**v1.4 in one paragraph (founder decisions of 2026-11-08; `docs/v6/v6changelog.md` Entries 21 and 22).** The long path is reshaped before it is built; the build order does not change. Two separate pages, "Make shorts from a long video" and "Edit a clip", sharing one editor that applies the AI's plan automatically. Long videos up to 45 minutes, read in place and never copied whole; only the parts chosen for shorts are copied (TS §43, §44.8). A long video is transcribed in the cloud only, and pauses with Continue on failure (TS §43.6). B-roll from the person's own clips and from Pixabay stock, placed by the plan or by hand on a word (TS §48). Auto zoom (TS §47.2). After V6.11, a quality pass against the founder's Submagic benchmark (§8A.12). Tooscut stays the study source for the engine, never copied (TS §53.4).

**Where things stand (2026-11-08).** V1-V6 are closed, tagged and live, and so are two releases between V6 and V6.1: the design release (tag `design`) and cloud transcription (tag `cloud`). **V6.1 is next**, under v1.4. *As first written on 2026-11-02:* V1-V5 are closed, tagged and live. **V6 is next.** Dates below are as first written; V1-V5 ran ahead of them, and everything after V6 moves. Estimated public launch: about February 2027 (§8A.0); the founder sets it when V6.11 closes.

| Version | Name | Spec steps | Calendar (build starts Mon 12 Oct 2026) | Gate at the end |
|---|---|---|---|---|
| **V1** | Working skeleton | M0.1 | Week 1: 12-18 Oct | Deployed shell + API + CI green |
| **V2** | Proof of concept: one clip in, one MP4 out | M0.2, M0.3, M0.4 | Weeks 2-3: 19 Oct - 1 Nov | **M0 gate** (Sun 1 Nov): stop or continue |
| **V3** | Real ingest, clean voice, real detector | M1.1, M1.2, M1.3 | Week 4, first half: 2-4 Nov | Four event kinds at 0.90 precision |
| **V4** | The look and the editor | M1.4, M1.5 | Week 4, second half: 5-8 Nov | Preview + review UI working |
| **V5** | Export that an independent tool accepts | M1.6 | Week 5, first half: 9-11 Nov | All 11 verifier checks pass |
| **V6** | Accounts, limits, payments: the MVP features complete | M1.7, M1.8 | Week 5, second half: 12-15 Nov | **M1 gate** (Sun 15 Nov): all v1.1 P0 features work |
| **V6.1** | Long source (v1.4): its own page, 45 minutes read in place, 4K, HEVC, cloud transcription in resumable windows | M1.16 | After V6; dated when V6 closes | Master transcript survives a killed tab; flags off equals V5. **As built:** P171-P182 on `v6.1-build` (2026-11-08/09), merged dark (`long_video` and `high_res` off in production); tag `v6.1` on the merge commit |
| **V6.2** | AI edit plan (backbone) + emphasis, emoji, text extras; spend cap; (v1.5) hook lockup, caption glow, punch words, motion by rules, the Enhance panel | M1.9 | | AI plans validate and render; fallback works; privacy test revised; `Enhance::v5()` equals V5 |
| **V6.3** | Pause and silence cuts; the `Assembly` (stitched parts) | M1.10 | | Cut and stitched exports pass the verifier at the map's duration |
| **V6.4** | Director: a long video becomes 5-10 shorts; list; batch export | M1.17 | | A long fixture gives 5 exported shorts, API up or down. **Private alpha after this** |
| **V6.5** | Local vision and face-tracked framing; (v1.5) following the speaker in two-person shots | M1.18 | | A moving speaker stays framed; the person speaking is on screen (TE-27); no frame leaves; `FramePath::none` equals V5 |
| **V6.6a** | (v1.5) Picture: blur, automatic colour and light per shot, camera matching, face light | M1.23 | | `grade: None` lists unchanged; TE-29's panel |
| **V6.6b** | Graphics: cards, number moments, kinetic presets, (v1.5) fonts and presets, shapes, progress bar, end card | M1.12, M1.13, M1.14 | | Goldens per kind; frame budget |
| **V6.7** | Music (the person's own track), ducking; (v1.5) synthesized sound effects by rules, voice tuning | M1.11 | | Ducking and mix loudness measured |
| **V6.8** | Layers: B-roll (own and Pixabay stock, placed by the plan or by hand, v1.4), asset box, join transitions, split layout; (v1.5) the quality check | M1.19 | | Verifier 11/11 with B-roll; simple lists unchanged. **v1.5: E-16, then the founding release and first public demos after this** |
| **V6.9** | Chat editing | M1.20 | | Ops apply, refuse and undo; privacy test covers chat |
| **V6.10** | Brand kit, creator memory, back-catalogue queue | M1.15, M1.21 | | Golden matrix; memory changes the next prompt |
| **V6.11** | Person mask: text behind speaker, background change | M1.22 | | TE-25 per effect. **M1.x gate**: every PS §9.5 V6.x row works, or the founder dropped it. **Feature freeze** |
| **Quality pass** | (v1.4) Shorts compared with the founder's Submagic benchmark; presets, timing and catalog tuned (§8A.12) | E-15 | After V6.11; length set by the founder | Every difference fixed, tuned or knowingly accepted |
| **V7** | Survive real life: restore and failure recovery | M2.1 | Week 6: 16-22 Nov | Reload, crash and quota cases recover |
| **V8** | Prove the privacy claim; security hardening | M2.2 | Week 7: 23-29 Nov | Network-privacy test blocks releases |
| **V9** | Real clips and real speed | M2.3, M2.4 | Weeks 8-9: 30 Nov - 13 Dec | Corpus checkpoint (Sun 6 Dec); budgets met |
| **V10** | Launch gate | M2.5 | Week 10: 14-20 Dec | **M2 gate** (Sun 20 Dec); public launch Mon 21 Dec |

Build work ends at V10. M3 (Sun 21 Mar 2027) and M4 (Sat 19 Jun 2027) are decisions, not builds (PS §20.5).

**Dates after v1.3.** V6 and V6.1-V6.11 are dated as each closes, and V7-V10 keep five weeks after V6.11. The calendar dates of V7-V10 above no longer hold. The founder re-sets the launch date when V6.11 closes (PS A-2). If V6.x must be shortened, the founder chooses what to cut, from the tail or from any later release (§8A.0).

**The tight spot.** V3 to V6 are four versions in two weeks. That is the specs' own schedule (five weeks of implementation, PS §9.9), not a choice of this plan. If it slips, the rule is PS §18: cut the P0 feature with the lowest contribution to H1 before moving a date, and keep the five stabilization weeks (V7-V10) intact. One relief valve exists: the V6 server work (M1.7) depends only on V1, so it can be started early whenever client work is blocked.

---

## 1. Rules that apply to every version

### 1.1 Money and hosting (fixed)

- Recurring cost is 0 USD/month with no paid fallback (PS A-3, PS A-10, TS TDR-17), **except the AI provider from V6.2: paid, under a monthly cap the founder sets (`AI_MONTHLY_CAP_USD`, TS §52)**, and (v1.4) Groq's paid tier for transcription, from before the private alpha (about 3 cents per 45-minute video, bounded by the audio budget; cloudimplementation D-342, TS §52). Pixabay's API is free.
- Frontend: static files on Vercel's free plan, which also proxies `/api/v1/*`. Backend: one Docker container on a Render free web service. Nothing else is deployed by you.
- Free external services, each chosen by an experiment in V1: Postgres (TE-6), model/asset storage (TE-7), transactional email (TE-8), merchant of record (TE-9), uptime monitor, GitHub Actions + GHCR (TE-10).
- When a free tier stops fitting, the component moves to another free tier. A paid tier is a founder decision plus a spec revision, never a quiet fix.

### 1.2 What "version done" means

A version is done only when all of these hold:

1. Every test named in its exit checklist passes in CI, or the named measurement is committed under `bench/results/` (TS §28 gating rule).
2. `pnpm check` and `pnpm test` are green: fmt, clippy `-D warnings`, `cargo deny`, ESLint, `tsc --noEmit`, `check-file-tree`, `check-gen-clean`, `check-copy-codes` (TS §33).
3. It is deployed: server first, then web (TS §33 deploy order), and the header check passes against the deployed URL (TS §24.3).
4. Every experiment listed for the version has a recorded outcome in `docs/`, or its "if it fails" action has been taken before the next version starts (TS §28).
5. Every (assumption) the version measured is replaced by its value in TS §39.3, or stays listed there.
6. The commit is tagged `v1` ... `v10`, and `v6.1` ... `v6.11` for the point releases (v1.3).
7. **v1.3: every proof of TS §41.1 still passes, with flags off and with shipped flags on; any change to one is a row of TS §41.2.**

### 1.3 How each file gets written (TS §34)

- Write a per-file spec from `docs/file-specs/TEMPLATE.md` before the file. Give the implementer only that spec, TS §2, §7, §10, §11, the owning deep-dive section, the signatures of allowed dependencies and the generated types. Not the whole repo.
- Signatures in the TS are contracts. A file implements them exactly.
- No source file over 700 lines, tests excluded (TS §29).
- A file that is not in the TS §5 tree fails `check-file-tree.mjs`. If a version needs a new file, the TS §5 tree is updated in the same change.

### 1.4 Lines that are never crossed

| Rule | Where it comes from |
|---|---|
| Nothing is cut without a validated plan; a cut is a shortened pause, lead/trail silence, an opt-in filler/false start or a user-asked whole sentence in chat (V6.9), never inside a word, never a reorder inside a `TimeMap` or a speed change; every cut can be restored. A short's parts are a validated `Assembly` of whole sentences. Until V6.3 nothing is cut at all | INV-5, INV-10, INV-25, PS §8 principle 7, §9.2.1, §12.9 |
| No media, frame, caption text as edited, event or plan parameter or file name leaves the browser; transcript text, sentence text, chat text and asset labels leave only in the bodies of the AI routes while the AI plan is on, and the server never stores or logs them | INV-2, INV-28, P-1, P-14, P-15, PS §15 |
| `fetch` exists in four files only; the browser contacts two hosts only; only the server calls the AI provider | TS §7, TS §24.1, TS §40.7 |
| **A clip with flags off is V5, byte for byte; a protected proof changes only by a row of TS §41.2; a feature not worth what it would break goes back to the founder** | INV-26, TS §41 |
| **Tooscut is read to learn approaches. No code, shader, struct or constant is copied or translated from it, and the folder stays out of this repository** | TS §53.4 |
| Nothing reads a plan that `validate` did not return; no number on screen that was not parsed from the transcript; a plan failure never blocks a clip | INV-21, INV-22, INV-24 |
| Time converts between recording and program time only through `TimeMap`, and between source and recording time only through `Assembly` | TS §40.1, INV-27 |
| The main thread never decodes, infers, filters, renders or encodes | INV-17 |
| No inference session (speech, face, person) and the render session coexist | INV-12, DC-10 |
| All user-facing text and every number in it come from `copy/messages.ts` and generated limits | INV-15 |
| No event exists below its kind's confidence threshold | INV-6 |
| Nothing from TS §36 "What Not to Build" gets built, including "temporary" versions of it | TS §36 |

### 1.5 Reference machines and devices you need access to

| Need | Used for | First needed |
|---|---|---|
| R1: 2021-class Windows laptop, 8 GB RAM, integrated GPU, Chrome stable, plugged in | Every performance budget | V2 |
| R2: Apple M1, 8 GB, Chrome stable | Same budgets, second platform | V2 |
| One Android phone and one iOS phone | Manual playback of exported MP4s | V10 |
| Accounts on two target platforms (for example YouTube and LinkedIn) | Manual upload test | V10 |

If you do not own an R2-class Mac or one of the phones, arrange access before V2 and V10. Without them, launch-checklist items stay unchecked (PS §20.1).

---

## 2. Before V1 (now until Sun 11 Oct 2026)

No product code. Everything here is free.

**Local tools**

- Rust stable with target `wasm32-unknown-unknown`; `wasm-bindgen-cli`; `wasm-opt` (binaryen); `cargo-deny`; `sqlx-cli`.
- Node with pnpm; Playwright with the Chrome channel.
- Docker (to build the API image the way CI does).
- Python with NumPy, plus the `ffmpeg` and `ffprobe` command-line tools. These are development tools for fixtures and the verifier only; FFmpeg never ships in the product (TS §36).

**Accounts to open**

| Account | Purpose | Note |
|---|---|---|
| GitHub | Repo, Actions, GHCR | A public repo has the most generous free Actions minutes; TE-10 measures the Windows-runner cost |
| Vercel (free plan) | Frontend hosting | Its terms restrict the free plan to non-commercial use: accepted open risk (TS TDR-12) |
| Render (free web service) | API hosting | Sleeps after 15 minutes idle, about a minute to wake (TS §22.9) |
| Postgres candidates: Neon, Supabase | Server state | TE-6 picks one |
| Asset storage candidates: Cloudflare R2, Hugging Face Hub | Model files, demo video, sample clip | TE-7 picks one; check whether a candidate demands a payment card on file even for its free allowance, and prefer the one that does not |
| Email candidates: Resend, Brevo | Magic links | TE-8 picks one; it must work without a purchased domain |
| Merchant of record candidates: Paddle, Lemon Squeezy, Dodo Payments | Checkout, tax, subscriptions | Start onboarding in week 1: approval can take two weeks (PS §18) |
| UptimeRobot (or equivalent free monitor) | Uptime, keep-warm ping, status page | One monitor, every 5 minutes |

**Non-code preparation**

- Make three demo clips by hand in any editor, showing the intended output (captions, a number landing, a list, a from-to). These are the E-1 landing-page demos (PS §19). Label them honestly as demos.
- List 50 technical creators for direct outreach (PS §13.7).
- Write the reading scripts for the speech fixtures into `fixtures/speech/README.md` (TS §26): the 60-second reference script needs about 150 words, two numbers, one three-item list, one from-to and one emphasized word.

---

## 3. V1 - Working skeleton (M0.1, week 1: 12-18 Oct)

**Goal.** Every layer exists, is connected and is deployed, with almost no behaviour: a landing page on Vercel that is cross-origin isolated, runs the capability check, and records a waitlist email and two analytics events through the Render API into Postgres. CI builds and checks all of it.

**What you can demo.** Open the Vercel URL in Chrome: the landing page loads, the capability check goes green in under 3 seconds, and submitting an email writes a row. Open it in Firefox: a specific "unsupported" message.

**Depends on.** Section 2.

**Detailed plan.** `docs/v1/v1implementation.md` gives the file tree, types, function signatures, test cases and hand-over to V2 and V3. It pulls a few small files forward into V1 so later versions are purely additive: `workers/protocol.ts` (types only), `workers/pool.ts` (`preload()` only), `persistence/db.ts` and `schema.ts` (all eight stores), the complete cold-start logic of `net/http.ts` with `http.test.ts`, a placeholder `EditorPage.tsx`, `auth/magic_link.rs` holding only `normalize_email()`, and a new server test `notify_me.rs`. Where that document and the tables below differ on which version creates a file, that document is the more precise one.

### 3.1 Build

**A. Repository and workspace**

| Create | Purpose |
|---|---|
| `git init`; move `documents/*.md` to `docs/` (the TS §4 layout) | Repo root |
| `Cargo.toml`, `rust-toolchain.toml`, `clippy.toml`, `deny.toml` | Cargo workspace; banned types and methods; license and crate bans (TS §7) |
| `package.json`, `pnpm-workspace.yaml`, `.gitignore` | Root scripts of TS §33 |
| `docs/file-specs/TEMPLATE.md` | Per-file spec template (TS §34.2) |

**B. Shared types and code generation.** Define every type now, exactly as TS §10 gives it. They are small, and fixing the boundaries first is what lets later versions be built file by file.

| Create | Contents |
|---|---|
| `crates/offcut-types/src/` `units.rs`, `ids.rs`, `limits.rs`, `media.rs`, `transcript.rs`, `events.rs`, `prosody.rs`, `edit.rs`, `profile.rs`, `summary.rs`, `stage.rs`, `error.rs` | TS §10.1-§10.6 |
| `crates/offcut-types/tests/ui.rs`, `tests/ui/bare_ms_rejected.rs` | A bare number cannot stand in for a unit type |
| `crates/offcut-api-types/src/` `auth.rs`, `account.rs`, `billing.rs`, `usage.rs`, `analytics.rs`, `errors.rs` | TS §10.7 DTOs and the analytics allowlist of TS §22.6 (all 19 events) |
| `scripts/gen-types.sh`, `scripts/check-gen-clean.sh` | `ts-rs` export to `web/src/gen/domain.ts` and `api.ts`; CI fails on drift |
| `crates/offcut-wasm-core/src/lib.rs` (panic hook and init only), `scripts/build-wasm.sh`, `web/src/wasm/load-core.ts` | Proves the Rust to WASM to browser path under the production CSP in week 1, not week 2 |

**C. Server (three routes live)**

| Create | Purpose |
|---|---|
| `server/src/main.rs`, `config.rs`, `state.rs`, `router.rs`, `error.rs`, `headers.rs`, `log.rs` | Axum process; the only env reader; redacted JSON logs (TS §22.8, §24.4) |
| `server/src/client_ip.rs`, `rate_limit.rs` | Client IP from `X-Forwarded-For` by `TRUSTED_PROXY_HOPS`; token buckets (TS §24.5) |
| `server/migrations/0001_init.sql` | The whole TS §23.2 schema, all eight tables, in the first migration |
| `server/src/db/mod.rs`, `db/analytics_events.rs`, `db/platform_waitlist.rs` | Pool of 5; the two tables V1 writes |
| `server/src/analytics/mod.rs`, `analytics/retention.rs` | `POST /events` with allowlist validation; 90-day purge at boot and every 6 h |
| `server/src/account/mod.rs`, `account/notify.rs` | `POST /notify-me` |
| `GET /healthz` in `router.rs` | Liveness, touches no DB |
| `server/Dockerfile`, `server/.sqlx/`, `render.yaml` | Prebuilt image; offline query cache; one free web service |
| `server/tests/common/mod.rs`, `server/tests/analytics_allowlist.rs` | Test app; allowlist, schema assertion and purge |

`config.rs` aborts on any missing variable (TS §22.8). Mail and billing are not live until V6, so set those variables to placeholder values in Render for now. Generate the two Ed25519 signing keys now and store them only as Render environment variables (TS §24.2).

**D. Web shell**

| Create | Purpose |
|---|---|
| `web/package.json`, `index.html`, `vite.config.ts`, `vitest.config.ts`, `playwright.config.ts`, `tsconfig.json`, `eslint.config.js` | Vite with COOP/COEP dev headers and an `/api/v1` proxy; boundaries and banned-global lint rules (TS §7) |
| `web/vercel.json` | The TS §24.3 headers, the `/api/v1/:path*` rewrite to the Render host, the SPA fallback |
| `web/src/main.tsx`, `App.tsx`, `routes.tsx` | Routes of TS §13.1 |
| `web/src/config/env.ts`, `config/allowlist-hosts.ts` | The only `import.meta.env` reader; the two allowed hosts |
| `web/src/copy/messages.ts` | All copy, starting with landing, capability and unsupported messages |
| `web/src/net/http.ts`, `net/api-client.ts`, `net/asset-fetch.ts` | The only fetch call sites |
| `web/src/platform/capability.ts`, `capability.test.ts`, `simd-probe.ts` | The 11 checks of TS §13.2, in order |
| `web/src/workers/render/encoders.ts` (constants only) | `VIDEO_ENCODE_LADDER` and `AAC_ENCODE_CONFIG`, which checks 9 and 11 need |
| `web/src/state/capability-store.ts`, `state/machines/transition.ts` | First store; the shared `transition()` helper |
| `web/src/usecases/start-app.ts`, `submit-notify-me.ts` | C-1 (without session and clip restore yet), C-16 |
| `web/src/analytics/client.ts` | Typed `track()`, batching, flush (TS §32) |
| `web/src/ui/pages/LandingPage.tsx`, `UnsupportedPage.tsx`, `SettingsPage.tsx` (table only) | J1; C-16; the "What leaves your device" link needs a target from day one |
| `web/src/ui/components/CapabilityGate.tsx`, `NotifyMeForm.tsx`, `DropZone.tsx`, `WhatLeavesTable.tsx`, `ui/styles/*` | `landing.spec.ts` requires the drop zone and sample button to be present. Plan choice: in V1 they are rendered but lead to a "not available yet" message from `messages.ts`, and the waitlist form is the active call to action |
| `web/tests-e2e/landing.spec.ts`, `helpers/fake-api.ts`, `helpers/fixtures.ts` | J1 assertions |

**E. CI, checks and deploy**

| Create | Purpose |
|---|---|
| `.github/workflows/ci.yml` | Steps 1-9 of TS §33 |
| `.github/workflows/deploy-api.yml` | Build image, push to GHCR, call the Render deploy hook |
| Vercel prebuilt deployment from CI | No Rust toolchain on Vercel's builders (TS §33 step 11) |
| `scripts/check-file-tree.mjs`, `check-hosts.mjs`, `check-copy-codes.mjs`, `check-external-facts.mjs` | Tree, bundle hosts, copy coverage, free-tier fact list |
| Post-deploy header check | Asserts every TS §24.3 header on the deployed URL |

### 3.2 Experiments run in V1

| ID | What you do | Pass |
|---|---|---|
| TE-5 | Read Vercel's current plan terms and record them; deploy the shell; call a deliberately slow endpoint through the rewrite; inspect `X-Forwarded-For` | `crossOriginIsolated` true; a 60 s upstream delay survives the proxy; hop count known and set in `TRUSTED_PROXY_HOPS` |
| TE-6 | Run migrations on each Postgres candidate; leave idle a week; measure reconnect | No expiry; reconnect under 2 s; quota at least 5x the 72 MB estimate |
| TE-7 | Upload a 150 MB file; ranged fetch from the deployed shell | 206 responses; CORS readable under COEP; no egress charge |
| TE-11 | Deploy the API; measure RSS, cold start over 10 sleeps, monthly instance hours with the keep-warm ping | RSS under 400 MB; cold start under 70 s; 744 h fits the 750 free hours |
| TE-9 | Start merchant onboarding | Clock starts; result needed by V6 |
| E-1 | Landing page with the three hand-made demo clips and the waitlist goes public | Runs three weeks; read at the M0 gate |
| E-8 | `capability_check` beacon live on the landing page | Runs until M3 |

### 3.3 Non-code work this week

- First build-in-public posts linking the landing page; first 15 of the 50 outreach messages. E-1 needs at least 1,000 visitors by 1 Nov to be conclusive.
- Point the uptime monitor at `https://<render-host>/api/v1/healthz` every 5 minutes.

### 3.4 Exit checklist

- [ ] `ci.yml` green.
- [ ] `landing.spec.ts`, `capability.test.ts`, `analytics_allowlist.rs`, `offcut-types/tests/ui.rs` pass.
- [ ] The deployed page reports `crossOriginIsolated === true`.
- [ ] `/api/v1/healthz` answers through the Vercel rewrite.
- [ ] A submitted email appears in `platform_waitlist`; `landing_view` and `capability_check` rows appear in `analytics_events`.
- [ ] `offcut_core.wasm` loads in the deployed page under the production CSP.
- [ ] TE-5, TE-6, TE-7, TE-11 outcomes written down; Postgres and asset hosts chosen.
- [ ] Tagged `v1`.

**Not in V1.** Any media handling, any model, sign-in, billing, the editor.

---

## 4. V2 - Proof of concept: one clip in, one MP4 out (M0.2, M0.3, M0.4; weeks 2-3: 19 Oct - 1 Nov)

**Goal.** Answer the two questions that can kill the project before most of it is built: can a browser on R1 transcribe a 60-second clip in about 20 seconds, and can it render and encode it in about 90 seconds? Then join the two answers into one real path: the reference clip goes in, a 1080x1920 MP4 with Clean-style captions and one NumberReveal comes out.

**What you can demo.** Drop the reference clip, watch the model download once, see a preview with captions and an animated number, download an MP4 that plays.

**Depends on.** V1. The reference fixture `fixtures/speech/speech_60s_portrait.mp4`, recorded by you from the script (include the two white-flash frames the verifier uses for sync, TS §26).

### 4.1 Build

**A. Minimal ingest (happy path only; rejections come in V3)**

| Create | Purpose |
|---|---|
| `crates/offcut-mp4/src/` `reader.rs`, `boxes.rs`, `sample_table.rs`, `demux.rs`, `probe.rs` | `RandomAccess`, box parsing, sample tables with edit lists, `Demuxer`, `probe()` (TS §15.1) |
| `crates/offcut-dsp/src/resample.rs` | `resample_mono` to 48 kHz and 16 kHz |
| `crates/offcut-wasm-core/src/media_api.rs`, `hash_api.rs` | `open_demuxer`, `probe_and_validate`, `resample`; `Sha256Stream` |
| `web/src/workers/protocol.ts`, `rpc.ts`, `pool.ts` | The four worker interfaces of TS §14.2; request/response plumbing; lazy worker creation |
| `web/src/workers/media.worker.ts`, `media/import.ts`, `media/audio-decode.ts` | Copy to OPFS in 4 MiB chunks; AAC decode; track alignment to video time 0 |
| `web/src/persistence/opfs.ts` | The only place OPFS paths are built |

**B. ASR spike (M0.2)**

| Create | Purpose |
|---|---|
| `web/src/config/model-manifest.json` | Paths, sizes, SHA-256 of the model files; total at most 260 MB |
| `scripts/upload-assets.sh` | Uploads content-hashed model files to the asset host chosen in TE-7 |
| `web/src/models/model-manager.ts`, `download.ts`, `download.test.ts` | `inspect`, `ensureReady`; ranged, resumable download in 8 MiB parts; hash verify (TS §16.3) |
| `web/src/state/model-store.ts`, `state/machines/model-machine.ts` | Model state machine (TS §12.2) |
| `web/src/workers/asr.worker.ts`, `asr/whisper-runtime.ts`, `asr/model-cache-adapter.ts`, `asr/word-timestamps.ts` | ONNX Runtime Web loaded from `/ort/` on the app origin; model served from OPFS only, a cache miss throws and never fetches; 30 s windows with 5 s overlap (TS §16.4) |
| `web/public/ort/` via `vite.config.ts` | Runtime `.wasm` files copied from the pinned npm package |
| `web/src/ui/components/ModelDownloadPanel.tsx` | J4 copy and progress |
| `web/tests-e2e/model-download.spec.ts` | Progress shown; interrupted download resumes; hash mismatch re-downloads; second session skips |

**C. Render and encode spike (M0.3)**

| Create | Purpose |
|---|---|
| `crates/offcut-mp4/src/mux.rs`, `tests/mux_roundtrip.rs` | Faststart `Mp4Muxer`; mux then demux round trip |
| `crates/offcut-scene/src/` `lib.rs`, `safe_area.rs`, `styles.rs` (Clean only), `fonts.rs`, `layout.rs`, `captions.rs`, `easing.rs`, `anim.rs`, `display_list.rs`, `framing.rs` (portrait pass-through), `events/mod.rs`, `events/number_reveal.rs`; `assets/fonts/*` | `build_scene`, `Scene::frame_at`, the `DisplayList` (TS §19) |
| `crates/offcut-render/src/` `lib.rs`, `gpu.rs`, `video_pass.rs`, `vello_backend.rs`, `composite.rs`, `shaders/video.wgsl` | Source frame plus `DisplayList` to a WebGPU canvas |
| `crates/offcut-wasm-render/src/` `lib.rs`, `session.rs`, `mux_api.rs`, `profile_api.rs`; `web/src/wasm/load-render.ts` | `RenderSession`; second WASM bundle |
| `web/src/workers/render.worker.ts`, `render/video-source.ts`, `render/export-loop.ts`, `render/encoders.ts` (complete), `render/opfs-sink.ts` | Frame-accurate export loop of TS §21.3 |
| `verify/verify_mp4.py` (checks 1-6), `verify/requirements.txt`, `verify/README.md` | Independent check with `ffprobe` and NumPy only |

**D. Proof-of-concept glue (M0.4)**

| Create | Purpose |
|---|---|
| `crates/offcut-text/src/` `lib.rs`, `tokenize.rs`, `sentences.rs`, `numbers.rs` (digits and simple spelled numbers), `normalize.rs`; `offcut-wasm-core/src/text_api.rs` | Enough normalization for one number |
| `crates/offcut-detect/src/` `lib.rs`, `config.rs`, `number.rs`, `event_id.rs`; `offcut-wasm-render/src/detect_api.rs` | NumberReveal only |
| `crates/offcut-entitlement/src/` `lib.rs`, `claims.rs`, `token.rs`, `verify.rs`, `profile.rs` | The export loop sizes its output from a verified token. In V2 the token is a Creator token signed with a test key by `helpers/fake-api.ts` |
| `web/src/usecases/import-clip.ts`, `run-pipeline.ts`, `control-preview.ts`, `start-export.ts` (no gate yet) | C-2, C-4, C-5, C-9 in their simplest form |
| `web/src/state/clip-store.ts`, `preview-store.ts`, `export-store.ts` | Stores for the three flows |
| `web/src/workers/render/preview-loop.ts` | Basic preview at 540x960 |
| `web/src/ui/pages/EditorPage.tsx`, `components/ProcessingFeed.tsx`, `PreviewPlayer.tsx`, `ExportButton.tsx`, `ExportProgress.tsx`; `DropZone` wired | The one path, no review controls |
| `.github/workflows/e2e-media.yml` | Media suites on `windows-latest` with Chrome stable |
| `bench/device-bench.ts`, `bench/results/` | Median and p90 of 10 runs per stage |
| `web/tests-e2e/pipeline-preview.spec.ts`, `export-creator.spec.ts` | Reference-clip cases only for now |

**Moved into V2 as built** (each a **(schedule)** decision of `docs/v2/v2implementation.md`; the implementation document is the more precise one on which version creates a file):
- `state/machines/clip-machine.ts` and `export-machine.ts`, with the complete transition tables of TS §12.2 (planned for V3 and V5). Their test files stay in V4 and V5 (v2implementation D-18).
- `state/blockers.ts` with all eight `BLOCKER_CODES`, `forImport()` complete and `forExport()` holding its two state-only conditions (planned for V5; v2implementation D-20).
- `crates/offcut-mp4/src/validate.rs` with all 11 rules of TS §15.2 (planned for V3). V3 adds `probe_rejections.rs`, the fixtures, the 14 messages, `RejectionPanel` and `clip_rejected` (v2implementation D-21).
- `persistence/entitlement-repo.ts` with `get()` and `put()`: the export reads its token from the IndexedDB `entitlement` store, which tests write (planned for V6; v2implementation D-23).
- `config/entitlement-public-key.ts`: the production key, plus the test key only in a build with `VITE_ENTITLEMENT_TEST_PUBLIC_KEY`; CI refuses a deployment whose bundle holds the test key (planned for V6; v2implementation D-24).
- The muxer's audio edit list for AAC priming (`elst.media_time`), planned for V5; V5 keeps `ctts` and the overflow test (v2implementation D-33).

### 4.2 Experiments run in V2

| ID | Question | Pass | If it fails |
|---|---|---|---|
| TE-1 | Does the ASR runtime load only from OPFS, make zero third-party requests and return word timestamps in a worker? | Zero requests outside the two hosts; timestamps for the reference clip; WebGPU and WASM backends both run | Second runtime behind the same `whisper-runtime.ts` interface |
| TE-2 | Is a per-word confidence available? | Yes, at no more than 10% time cost | `Confidence(1.0)` for all words |
| E-3 | ASR time on R1 | Median at most 20 s; p90 total at most 180 s | Over 145 s for a 60 s clip (D-67 of the V2 plan; was 40 s): ship a smaller model (base-size first, then tiny-size), stronger caption editing |
| TE-3 | Vello on WebGPU in a worker on an `OffscreenCanvas`; which frame-capture method is faster; hidden tab | Correct frame every time; at least 30 frames/s render-only on R1; works hidden | Other capture method, then the Canvas2D backend |
| TE-4 | Which H.264/AAC configs work on R1 and R2 | A ladder entry supported on both; A/V offset within one frame | Next ladder entry; silent pre-roll for AAC priming |
| E-4 | Render + encode time on R1 | At most 90 s | Over 150 s: Canvas2D overlay path; cap input at 60 s and 30 fps |
| TE-10 | Can CI run the media suites on a hosted Windows runner within free minutes? | Suites pass headless; minutes fit | Run on `main` and pre-release only; `pnpm e2e:device` locally |
| TE-14 | Peak memory on R1 for a 90 s, 1080p, 60 fps input | Under 1.5 GB; no tab crash in 10 runs | Smaller queues; preview at 360x640; smaller model |

### 4.3 The M0 gate (Sun 1 Nov 2026)

This is the one point where the right answer may be to stop. Decide from three readings (PS §9.9, PS §20.4):

| Reading | Continue | Apply the fallback, then continue | Stop |
|---|---|---|---|
| E-1 waitlist | At least 1,000 visitors and at least 5% join | Under 1,000 visitors: inconclusive, keep going and keep measuring. 3-5%: rewrite the pitch once and re-run | Under 3% again after the rewrite, with at least 1,000 visitors |
| E-3 ASR on R1 | Median at most 20 s | 20-145 s for a 60 s clip: ship and record the miss (D-67 of the V2 plan; the band ended at 40 s). Over 145 s: smaller model | Still failing after the fallback makes cloud transcription launch-blocking (PS §19), which the 0 USD rule cannot fund: treat as a stop |
| E-4 render + encode on R1 | At most 90 s | 90-150 s: ship and record. Over 150 s: Canvas2D path, 60 s / 30 fps cap | No path under budget |

### 4.4 Exit checklist

- [ ] `model-download.spec.ts`, `download.test.ts`, `mux_roundtrip.rs` pass.
- [ ] `verify_mp4.py` checks 1-6 pass on a spike export.
- [ ] `pipeline-preview.spec.ts` and `export-creator.spec.ts` pass on the reference clip locally on R1 and on R2, and `e2e-media.yml` runs them in CI.
- [ ] ASR and render + encode timings for R1 and R2 committed under `bench/results/`.
- [ ] TE-1, TE-2, TE-3, TE-4, TE-10, TE-14 outcomes recorded; model, capture method and encoder ladder order fixed.
- [ ] M0 gate decision written down with the three readings.
- [ ] Tagged `v2`.

**Not in V2.** Rejections, voice cleanup, three of the four event kinds, two of the three styles, landscape crop, review controls, accounts.

---

## 5. V3 - Real ingest, clean voice, real detector (M1.1, M1.2, M1.3; week 4, first half: 2-4 Nov; built in eight working days)

**Goal.** Replace the three stand-ins from V2 with the real thing: any file a user drops is either accepted or rejected with a specific message; the voice is cleaned to the loudness target without changing its length; all four event kinds are detected by rules at 0.90 precision or better.

**What you can demo.** Drop a 4K file, a 2-minute file and an HEVC file: three different, specific rejections. Drop a good clip: the feed shows "Found: 3-item list", "Found: $2k to $20k"; the audio is audibly cleaner and exactly as long as before.

**Depends on.** V2 and a passed M0 gate.

**Eight working days, not three** (v3implementation §5). V3 is built in eight working days on the branch `v3-build` (S1 to S15): S1 contracts and docs; S2 fixtures and the frozen labeled set; S4 the voice chain natively and S6 in the browser, risk first; S3 the demuxer; S5 the number grammar; S7 the detector rules, re-detection and the precision gate (two and a half days); S8 records and the full pipeline; S9 the rejection panel and copy; S10-S11 the E2E suites, corpus tooling and CI; S12 the measurements; S13-S15 copy-back, merge, deploy and tag. The calendar of section 0 moves by the difference; the rule of section 0's "tight spot" applies.

### 5.1 Build

**A. Ingest and rejections (M1.1)**

| Create or complete | Purpose |
|---|---|
| `crates/offcut-mp4/src/validate.rs` | `validate_probe`: the 11 rules in order, first failure wins (TS §15.2). Created whole in V2 (v2implementation D-21); V3 tests it |
| `probe.rs` complete; `edit_list.rs` | Rotation, VFR flag, track counts, codec strings; edit-list shapes E0-E4 (v3implementation D-92) |
| `fixtures/gen_fixtures.sh`, `fixtures/manifest.json` | Generates every `ok_*` and `rej_*` fixture of TS §26 with the ffmpeg CLI from `lavfi` sources, and derives the five speech fixtures from the reference clip; nothing is committed (v3implementation D-76, D-77) |
| `messages.ts`: one entry per `REJECT_*` | 14 rejection messages with interpolated limits (TS §15.3) |
| `web/src/ui/components/RejectionPanel.tsx` (`state/machines/clip-machine.ts` exists since V2, v2implementation D-18) | Rejection is a value, not an error (TS §11.3) |
| `web/src/persistence/db.ts`, `schema.ts`, `clips-repo.ts`, `transcript-repo.ts`, `edits-repo.ts` | Stage F of C-4 writes the clip, transcript and edits. The `clips` store has no file-name field |
| `crates/offcut-mp4/tests/demux_fixtures.rs`, `probe_rejections.rs`; `web/tests-e2e/rejections.spec.ts` | Every fixture gives its expected outcome |
| `corpus/manifest.json`, `fetch_corpus.sh`, `run_corpus.ts` | Corpus runner; clips stay outside git, in the local directory `OFFCUT_CORPUS_DIR`, never uploaded (v3implementation D-88) |

**B. Audio enhancement and prosody (M1.2)**

| Create | Purpose |
|---|---|
| `crates/offcut-dsp/src/` `config.rs`, `highpass.rs`, `denoise.rs`, `compressor.rs`, `limiter.rs`, `loudness.rs`, `chain.rs`, `prosody.rs` | `run_chain` exactly as the worked exemplar in TS §34.3; `measure_prosody` |
| `crates/offcut-wasm-core/src/audio_api.rs`; `web/src/workers/audio.worker.ts` | `runChain`, `measureProsody` |
| `crates/offcut-dsp/tests/` `loudness_target.rs`, `chain_no_clicks.rs`, `duration_preserved.rs`, `prosody.rs` | Postconditions P1-P8 of TS §34.3 |
| Fixtures `speech_20s_long_pause.mp4`, `speech_30s_sparse.mp4`, derived by `gen_fixtures.sh` from the reference clip with generated room tone; nothing is recorded (v3implementation D-76) | The pause must survive: 20 s in, 20 s out, never 15 s |

**C. Normalization and the detector (M1.3)**

| Create or complete | Purpose |
|---|---|
| `crates/offcut-text/src/` `numbers.rs` (complete), `units_lex.rs`, `sentences.rs` (complete), `normalize.rs` (complete) | Spelled cardinals to trillions, decimals, k/m/b, currencies, percent, multipliers, units (TS §17.2) |
| `crates/offcut-detect/src/` `from_to.rs`, `list.rs`, `keyword.rs`, `exclusions.rs`, `resolve.rs`; `config.rs` (complete) | Scoring table and thresholds of TS §17.4; resolution order of TS §17.5 |
| `fixtures/labeled/*.json` (30 or more), `fixtures/labeled/SCHEMA.md` | Labeled transcripts covering all four kinds, written as text and frozen before the detector's rules (v3implementation D-87) |
| `crates/offcut-text/tests/` `numbers_table.rs`, `numbers_prop.rs`, `sentences.rs` | Number parse and format |
| `crates/offcut-detect/tests/` `rules_table.rs`, `precision_recall.rs`, `redetect.rs`, `determinism.rs` | Edge cases of TS §17.6; precision at least 0.90 |
| `run-pipeline.ts` complete | The C-4 order: audio extraction, then ASR and the voice chain in parallel, `asr.unload()` before `render.openSession()` |
| `ProcessingFeed.tsx` complete | One feed line per real detection, nothing invented |

### 5.2 Non-code work

- **Label at least 30 transcripts, as text, before the detector exists** (v3implementation D-87). The labeled set is written text, not recordings: 29 written transcripts, the reference clip's transcript in the written form, and the second recording as recognized. The labels are frozen in one commit before any detector rule is written; after that a label changes only with a changelog line naming the file, the label and the reason, never to match the detector. The labeling cannot be delegated to the detector being tested; the founder reads five files chosen at random and says whether each label is what a person would expect (an exit box).
- **No speech fixture is recorded** (v3implementation D-76). `speech_60s_landscape.mp4`, `speech_20s_noevents.mp4` and the other speech fixtures are derived from the reference clip by `gen_fixtures.sh` and stay out of git (D-39). A real recording made later is stored in `testclips/` under the same name and is preferred, with no code change.
- Start collecting the 40+ real-clip corpus (VFR, rotated, noisy, accented). V9 needs the full corpus; asking people for clips takes calendar time, so start now. The 20-clip ASR accuracy set moves to V9 with E-10 (below).

### 5.3 Experiments

Read on D1, the development machine, and recorded under the label `d1` until R1 exists; thresholds do not change, and a clip of another length is read against the budget times its length over 60 s (v3implementation D-74, as v2implementation D-69).

| ID | Pass | If it fails |
|---|---|---|
| TE-12 (first run) | `offcut-mp4` demuxes at least 90% of the corpus clips collected so far; "inconclusive" with fewer than 10 clips, re-checked in V9 (v3implementation D-88) | JS demuxer behind the same `SampleMeta` shape |
| TE-13 | Voice chain at most 4 s for the reference clip on R1 | `ChainConfig.denoise = false`: high-pass, loudness and limiter only |
| E-10 | **Carried to V9** (v3implementation D-88): no 20-clip accent set exists. V3 builds `run_corpus.ts --wer` and reads it on the founder's clips as a smoke reading that is not E-10. In V9: WER at most 12% on the 20-clip set (the caption-edit half is read in V7-V9 from real use) | Larger model, stronger edit UX or narrower claims |

### 5.4 Exit checklist

- [ ] `demux_fixtures.rs`, `probe_rejections.rs`, `rejections.spec.ts` pass.
  - **Where the rejection cases run** (v3implementation D-93, D-94). CI's Linux job installs a pinned ffmpeg build and runs `pnpm fixtures` before `cargo test`, so the six fixture-reading Rust test files run on every pull request. The 15 cases of `rejections.spec.ts` tagged `@no-gpu` (the 13 refusals before the pipeline, "start over" and the `ok_vfr.mp4` accept case) run on CI's Windows media runner after `pnpm fixtures`. `REJECT_NO_SPEECH` needs a whole ASR pass and runs on the development machine with the other GPU cases, in `pnpm e2e:media` on a keyed build before the merge.
- [ ] `loudness_target.rs`, `chain_no_clicks.rs`, `duration_preserved.rs`, `prosody.rs` pass.
- [ ] `numbers_table.rs`, `numbers_prop.rs`, `sentences.rs`, `rules_table.rs`, `redetect.rs`, `determinism.rs` (detect) pass.
- [ ] `precision_recall.rs` at 0.90 or above on at least 30 labeled transcripts. If below: raise the per-kind threshold to the lowest value in the printed sweep that reaches 0.90 (TS §17 contingency).
- [ ] TE-12, TE-13 outcomes recorded; E-10's smoke reading recorded and E-10 carried to V9.
- [ ] Tagged `v3`.

**Not in V3.** Anything visual beyond what V2 drew. The three new event kinds are detected but only NumberReveal is drawn until V4.

---

## 6. V4 - The look and the editor (M1.4, M1.5; week 4, second half: 5-8 Nov; planned at thirteen working days in 19 steps, built from 22 Oct)

**Goal.** The preview shows the finished look, and the user can fix it: three caption styles, four event visuals, 9:16 framing for landscape clips, a synced preview with scrubbing, and the review controls (edit a word, toggle an event, pick a style, drag the crop).

**What you can demo.** The AHA moment of PS §10 J7: drop a clip, watch your own words animate with a number counting up and a list building. Then fix a mis-heard word and see the number graphic update.

**Depends on.** V3.

**As built (V4 copy-back).** `docs/v4/v4implementation.md` planned V4 at thirteen working days in 19 steps (S1 to S19, its §2) instead of this section's four, because V4 carries the seven seams of A2, the caption intelligence and the editor besides the look; the build started on 22 Oct 2026, ahead of this calendar (V3 closed on 21 Oct). Beyond the tables below it built: smart line breaks, a reading-time floor and number highlighting in the captions (D-167 to D-169); caption placement by drag with Top, Middle and Bottom snaps (D-171); a premium default style, Clean with a sliding pill on the spoken word, and Tech named "Mono" in the interface (D-170); a self-hosted interface font and a design system of tokens and primitives (D-179 to D-181); an editor with a top bar, a stage and a side panel of four tabs (Captions, Style, Visuals, Framing), direct manipulation on the preview, a transcript editor with "Change all", undo and redo of 50 steps, keyboard shortcuts and toasts (D-178, D-182 to D-187); two design reviews by the founder (D-191). `ReviewPanel.tsx` became `SidePanel` and its tabs (C below). The record is `docs/v4/v4changelog.md`.

### 6.1 Build

**A. Scene: styles, events, framing, watermark (M1.4)**

| Create or complete | Purpose |
|---|---|
| `crates/offcut-scene/src/styles.rs` | Clean, Bold, Tech as data (TS §19.4 table) |
| `events/list_reveal.rs`, `events/from_to.rs`, `events/keyword_pop.rs`; `number_reveal.rs` complete | Enter, body, exit timings of TS §19.5 |
| `framing.rs` complete | Landscape and square crop with offset; rotation handled in display coordinates |
| `watermark.rs`, `summary.rs` | Watermark drawn only when the profile says so (INV-9); `change_summary()` |
| `crates/offcut-scene/tests/` `display_list_snapshots.rs`, `layout_safe_area.rs`, `framing.rs`, `determinism.rs`, `snapshots/` | Exact display-list snapshots; nothing leaves the safe area; byte-identical frames |
| `crates/offcut-render/tests/golden_frames.rs`, `fixtures/golden/*.png` (14) | 3 styles x 4 event kinds, watermark on and off; similarity at least 0.98 on a software adapter |

**A2. Edit-plan seams (v1.2; TS §40.9 S-1 to S-7). Build these with A, not after it: they are the shape A is written in.**

| Create or change | Purpose |
|---|---|
| `offcut-types/src/units.rs` (`OutMs`, `FrameIdx::to_out_ms`), `timemap.rs` (`TimeMap::identity`, `to_out`, `to_src`, `joins`) | Two timelines from the start; identity until V6.3 (S-1) |
| `offcut-types/src/plan.rs`; new crate `crates/offcut-plan/` with `catalog.rs`, `prompt.rs`, `validate.rs`, `from_rules.rs`, `timemap.rs` (identity path only) | The scene reads a `ValidPlan`; the rules are its first producer; the prompt and schema are generated and snapshot-tested even though nothing sends them yet (S-2) |
| `QuantityRef` resolution in `build_scene` | Numbers on screen come from the (edited) transcript words (S-3, INV-22) |
| `DrawCmd::Video` and `DrawCmd::Image` in `display_list.rs`; `offcut-render` draws the list in order with the video as a command; `Scene::crop()` removed | Zoom, shapes and transitions later change one command, not the renderer (S-4, TDR-21) |
| `FontKey` + `FontSet` (replacing `FontId`), `Theme` roles, `StyleSpec.motion`, zone table in `safe_area.rs` | Brand kit and kinetic presets are data later (S-5) |
| `offcut-dsp/src/program.rs` (`assemble`, identity), `silence.rs` (`silence_profile`); `AudioWorkerApi.assemble`; preview plays `prog48` | Program audio exists before cuts and music need it (S-6) |
| `PlanItemList.tsx` instead of `EventToggleList.tsx`; a cuts section that renders nothing when there are no cuts | Review UI generic over plan items (S-7) |
| Tests: `timemap_identity.rs`, `plan_from_rules.rs`, `validate.rs`, `catalog_snapshot.rs`, `catalog_complete.rs`, `styles_use_theme.rs`, `program_identity.rs`; a punch-in golden (scale 1.1) | The identity case is proven equal to v1.1 behaviour; goldens unchanged |
| TS §5 file tree | Gains every new file above in the same change (§1.3) |

**B. Preview player (M1.5)**

| Complete | Purpose |
|---|---|
| `render/preview-loop.ts`, `render/video-source.ts` | Audio clock is master; clock sync every 250 ms; queue of 6 frames; seek to keyframe; resize to 360x640 when mean frame cost exceeds 28 ms (TS §20) |
| `usecases/control-preview.ts`; `PreviewPlayer.tsx` | Attach, play, pause, seek; first play follows a user gesture |

**C. Review UI (M1.5)**

| Create | Purpose |
|---|---|
| `web/src/ui/components/ReviewPanel.tsx`, `CaptionEditor.tsx`, `PlanItemList.tsx` (v1.2, was `EventToggleList.tsx`), `StylePicker.tsx`, `CropOffsetControl.tsx` | J8. No sliders except the crop offset. Until V6.3 no control removes or shortens anything |
| `web/src/usecases/edit-word.ts`, `toggle-event.ts`, `set-style.ts`, `set-crop-offset.ts` | C-6: a word edit re-runs detection for its sentence; toggles survive by `EventId` |
| `messages.ts`: `NO_EVENTS_FOUND` | The honest zero-event message (PS §10 J7) |

**D. Worker robustness (M1.5)**

| Create or complete | Purpose |
|---|---|
| `workers/rpc.ts`, `pool.ts` complete; `rpc.test.ts` | Progress throttling, cooperative cancel, 2 s cancel timeout, crash and restart with a fake `Worker` |
| `web/src/usecases/cancel-job.ts`; `ErrorPanel.tsx` | C-12; specific error copy with retry |
| `state/machines/clip-machine.test.ts`, `model-machine.test.ts`; `usecases/run-pipeline.test.ts` | Legal and illegal transitions; the unload-before-open ordering (INV-12) |
| `web/tests-e2e/preview-sync.spec.ts`, `review-edit.spec.ts`, `cancel.spec.ts`; `pipeline-preview.spec.ts` complete | Drift, edits, cancel, and the remaining preview cases including the zero-event clip |

### 6.2 Decision in V4

| If this misses on R1 | Do this | Source |
|---|---|---|
| Preview drift over 80 ms across 60 s, or more than 5% late frames | Render a 360x640 preview file with the export loop into OPFS and play it in a `<video>` element; edits re-render it | TS §20 contingency |
| Vello in a worker still unreliable after TE-3's fallback | Implement `Canvas2dBackend` drawing the same `DisplayList`; record separate goldens | TS §19 contingency |

### 6.3 Exit checklist

As built, V4's exit list is `docs/v4/v4implementation.md` §23 (code and checks, measurements, deployment, records); these are its headlines:

- [ ] `display_list_snapshots.rs`, `layout_safe_area.rs`, `framing.rs`, `determinism.rs` (scene), `golden_frames.rs` (16 goldens, on WARP and lavapipe) pass.
- [ ] Caption intelligence: `line_breaks.rs`, `placement.rs`, `parity.rs`, `scene_cache.rs`, `hit.rs` pass.
- [ ] The editor: every state reachable and matching the reviewed screens; axe finds no serious or critical violation; every control works by keyboard; no source file over 700 lines (tests excluded).
- [ ] Edit to preview at most 150 ms (90th percentile), style thumbnails at most 400 ms, the app shell within 400 kB gzip with the font; `build_scene` at most 1 s and mean `frame_at` at most 2 ms; seek within 300 ms; the bench file committed and `render_encode` within E-4's line.
- [ ] `pipeline-preview.spec.ts` (all cases), `preview-sync.spec.ts`, `review-edit.spec.ts`, `cancel.spec.ts` pass.
- [ ] `rpc.test.ts`, `clip-machine.test.ts`, `model-machine.test.ts`, `run-pipeline.test.ts` pass.
- [ ] Preview drift at most 80 ms over 60 s; first preview frame within 500 ms of `ready`.
- [ ] v1.2 seams S-1 to S-7: `timemap_identity.rs`, `plan_from_rules.rs`, `validate.rs`, `catalog_snapshot.rs`, `catalog_complete.rs`, `styles_use_theme.rs`, `program_identity.rs` pass; the scene takes no `events` argument; no `TimeMs` reaches `frame_at`; the renderer has no crop parameter.
- [ ] Deployed: `ci.yml` green on `main`, the header check and `@smoke` pass, and the editor works on the deployed page on D1; `review_action`, `job_cancelled` and `client_error` rows hold enum props only.
- [ ] Design reviews 1 and 2 passed by the founder; five goldens looked at.
- [ ] Tagged `v4`.

**Not in V4.** Export beyond the V2 spike, the watermark in a real export, accounts.

---

## 7. V5 - Export that an independent tool accepts (M1.6; week 5, first half: 9-11 Nov)

**Goal.** The exported file is correct by an outside measure, not by the code's own opinion: `verify_mp4.py`, which shares no code with the product, passes all 11 checks on it. Output size and watermark come only from the verified entitlement token. The export is exactly as long as the recording.

**What you can demo.** Export the long-pause fixture: 20 seconds in, 20 seconds out, pause intact, audio and video in sync on both sides. Export the same edit twice: the second is served from the render cache instantly.

**Depends on.** V4.

### 7.1 Build

| Create or complete | Purpose |
|---|---|
| `render/export-loop.ts` complete | Every output frame rendered and encoded exactly once; encoder-queue backpressure; restart from frame 0 on the next ladder entry if the encoder fails; cancel per frame; every `VideoFrame` closed in `finally` (TS §21.4) |
| `offcut-mp4/src/mux.rs` complete | `moov` reserve of 256 KiB, `ctts` for reordered frames, exact 30 fps timescale (TS §21.2, §21.3); the `elst` for AAC priming is written since V2 (v2implementation D-33) |
| `offcut-entitlement/src/profile.rs` in use | `None` gives Preview; valid Creator gives 1080x1920 without watermark; anything else gives Free, 720x1280 with watermark |
| `web/src/state/blockers.ts` (complete), `blockers.test.ts` | The eight blockers of TS §12.5, evaluated in one place; the file exists since V2 (v2implementation D-20) |
| `state/machines/export-machine.ts` (since V2, v2implementation D-18), `export-machine.test.ts` | `idle, gating, rendering, muxing, saving, done` |
| `usecases/start-export.ts` complete, `start-export.test.ts` | C-7 gate, C-9 render, C-10 deliver; download name `offcut-<yyyymmdd-hhmm>.mp4`, never the source name |
| `persistence/render-cache-repo.ts` | Cache key = SHA-256 over source hash prefix, transcript, events, edits, profile kind, renderer version |
| `ui/components/ChangeSummary.tsx`, `PostExportQuestion.tsx`; `usecases/answer-post-export.ts` | J11: only changes that were made; the "Would you post this?" question |
| `verify/verify_mp4.py` checks 7-11, `verify/watermark_mask_720.png` | Loudness, A/V sync and timeline against the source, watermark, bitrate, metadata |
| `web/tests-e2e/export-creator.spec.ts` complete, `timeline-preserved.spec.ts` | All 11 checks; INV-5 and INV-10 on both pause fixtures (600 and 900 frames) |
| v1.2 seams S-8, S-9 (TS §40.9) | The export loop runs over `OutMs`, reads source frames at `scene.source_at(t)`, encodes `prog48`, takes `frame_count` from `out_duration`; the test writes the expected duration beside each export and `verify_mp4.py` checks 5 and 8 compare against it; the render-cache key adds plan, time-map, theme and music hashes |

**As built** (`docs/v5/v5implementation.md`, its section "V5 as built"; `docs/v5/v5changelog.md`). All of the above, on D1 (D-202). The cache key has ten inputs, the profile kind from the token the worker verified (D-220). The test passes the expected duration as an argument, with no file beside the export (D-210). `blockers.ts` evaluates five of the eight codes; V6 appends the other three. Beyond the table: the verifier's self-test in CI (D-209), the stereo loudness fix (D-204), an encoder failure restarting on the next ladder entry (D-216), and `VideoSource.whileSessionHeld` (D-240). R1 stays unread: the render reading is `d1`.

### 7.2 Things to know about V5

- Sign-in does not exist until V6. In V5, tokens come from `helpers/fake-api.ts` in tests and local development, so the deployed V5 can preview but cannot export for a real visitor. That is expected.
- E-4 is measured again here with the full renderer. The same thresholds and fallbacks as the M0 gate apply.

### 7.3 Exit checklist

- [x] `export-creator.spec.ts` passes with all 11 verifier checks.
- [x] `timeline-preserved.spec.ts` passes on `speech_20s_long_pause.mp4` and `speech_30s_sparse.mp4`.
- [x] `start-export.test.ts`, `export-machine.test.ts`, `blockers.test.ts` pass.
- [x] Live-frame counter is 0 after an export and after a cancelled export (INV-11).
- [x] Render + encode on R1 re-measured and committed (read on D1, D-202).
- [x] Tagged `v5`.

**Not in V5.** The free-plan flow end to end (needs sign-in), checkout.

---

## 8. V6 - Accounts, limits, payments: the MVP's features (M1.7, M1.8; week 5, second half: 12-15 Nov)

**Goal.** The last missing MVP features: magic-link sign-in, the signed entitlement token, the free plan's three exports a month with watermark, usage receipts, account deletion and export, and checkout for the Creator plan through the merchant of record. After V6 the only new features are V6.1-V6.11 (§8A). Seam S-10: account deletion and export iterate one list of per-user tables, so V6.2's `ai_usage` and `ai_shorts_grants` are two more entries. **v1.3: V6 comes first, because nothing can be sold without it, and V6.2 and V6.4 need accounts for quotas.**

**First prompt of V6 (DC-11).** Open `docs/v6/v6implementation.md` and the V6 branch. Rewrite the old V6.x numbers in about 57 code comments through the map of TS §42: comment-only, proven by an empty diff with comments stripped, and `check-gen-clean.sh` after regenerating `web/src/gen/domain.ts`. Also address V5's carried items first, as planned: known issue 10 (`aria-details`), known issue 3 (`openEditor`).

**What you can demo.** The whole journey J1 to J12 with a real email address: drop, preview, click Export, see the limits before signing in, sign in from the emailed link, export a 720p watermarked file, see "2 of 3 left", upgrade in the merchant's test mode, export 1080p without watermark.

**Depends on.** V5 for the client parts. The server parts depend only on V1 and can start earlier. TE-9 merchant approval, started in week 1.

### 8.1 Build

**A. Server: auth (M1.7)**

| Create | Purpose |
|---|---|
| `server/src/auth/mod.rs`, `magic_link.rs`, `session.rs`, `access_token.rs`, `extract.rs` | Magic link (32 random bytes, hash stored, 15 min, single use); refresh cookie `HttpOnly; Secure; SameSite=Strict`, rotated on every use; Ed25519 access token, 15 min, memory only; CSRF header on the two cookie routes (TS §22.2) |
| `server/src/mail/mod.rs`, `smtp_mailer.rs`, `templates.rs` | `Mailer` trait; SMTP through Brevo on port 2525 with `lettre` (as built, v6implementation D-248, D-252); one template |
| `server/src/db/users.rs`, `magic_links.rs`, `sessions.rs` | Queries for the three tables |
| Routes `POST /auth/magic-link`, `/auth/verify`, `/auth/refresh`, `/auth/logout`, `GET /me` | Body limits, rate limits and idempotency of TS §22.1 |

**B. Server: entitlement, usage, account (M1.7)**

| Create | Purpose |
|---|---|
| `server/src/entitlement/mod.rs`, `issue.rs`; `offcut-entitlement/src/sign.rs` (feature `sign`) | The only place plan and remaining exports are computed and signed; free quota counts the UTC calendar month; 7-day token (TS §22.3) |
| `server/src/usage/mod.rs`, `db/usage_receipts.rs` | `INSERT ... ON CONFLICT (export_id) DO NOTHING`; never rejects a receipt; returns a fresh token (TS §22.4) |
| `server/src/account/delete.rs`, `export.rs` | Cancel the subscription first, then delete in one transaction; JSON export of server records (TS §22.7) |
| Routes `GET /entitlement`, `POST /usage/receipts`, `POST /account/delete`, `GET /account/export` | |

**C. Server: billing (M1.8)**

| Create | Purpose |
|---|---|
| `server/src/billing/provider.rs`, `adapter.rs`, `offers.rs`, `webhook.rs`, `mod.rs` | `BillingProvider` trait; the TE-9 provider's adapter; offer to price id; verify signature, dedupe by `provider_event_id`, apply in one transaction (C-19) |
| `server/src/db/subscriptions.rs`, `webhook_events.rs` | The transition table of TS §22.5; events older than `last_event_at` ignored |
| Routes `POST /billing/checkout`, `GET /billing/portal`, `POST /billing/webhook` | Hosted checkout by full-page navigation; no third-party script |

Replace the placeholder mail and billing variables in Render with real values. Keep `FOUNDING_OFFER_ENABLED=false` until the founding release after V6.8 (§8A.8; v1.5, v1.3: after V6.6). Prices (15 USD/month, 144 USD/year, 99 USD/year founding) are configured at the merchant and shown from `web/src/config/pricing.ts`; the server holds no amounts.

**D. Client**

| Create or complete | Purpose |
|---|---|
| `web/src/state/auth-store.ts`, `entitlement-store.ts`, `machines/auth-machine.ts` | Auth state machine (TS §12.2) |
| `web/src/entitlement/entitlement.ts` | `effective()` = token remaining minus the outbox's Free rows of the token's own account, floored at 0 (as built, v6implementation D-267); the only place the two are combined |
| `web/src/persistence/entitlement-repo.ts` (since V2, v2implementation D-23), `receipt-outbox.ts` | Token cache; receipts survive reload and retry at 5 s, 30 s, 5 min, then each app start |
| `web/src/platform/broadcast.ts` | The sign-in tab tells the original tab over `BroadcastChannel` |
| `web/src/config/entitlement-public-key.ts` (since V2, v2implementation D-24), `pricing.ts` | Public key as an array (rotation); display prices |
| `net/http.ts` complete | 10 s per attempt, backoff within the 70 s cold-start budget, "Connecting to the account service" after 3 s (C-15) |
| `web/src/usecases/` `request-magic-link.ts`, `complete-sign-in.ts`, `sign-out.ts`, `start-checkout.ts`, `confirm-checkout.ts`, `open-billing-portal.ts`, `delete-account.ts`, `export-account-data.ts`; `start-app.ts` complete (session restore) | C-8, C-11, C-18 |
| `web/src/ui/components/SignInDialog.tsx`, `UpgradePrompt.tsx`; `ui/pages/AuthCallbackPage.tsx`, `AccountPage.tsx` | Limits shown before the email field; the callback strips the token from the URL; the prompt has two placements only, `after_export` and `limit_reached` |
| `ExportButton.tsx` complete | Counter visible before the click; "last free export this month" confirm before the third (PS §11) |

**As built** (`docs/v6/v6implementation.md`, D-241 to D-315; `docs/v6/v6changelog.md`). All of the above, in 14 prompts (P137-P150) on `v6-build`. Mail goes through Brevo's SMTP relay on port 2525, as Render's free services block 25, 465 and 587 (D-248, D-252); TE-8 was cut down by the founder. The merchant is Dodo Payments, run in test mode in production until the founding release (D-273, D-285); its adapter was written from 64 real test-mode deliveries (`server/tests/fixtures/dodo/`), which showed that a refund does not stop a subscription there, so the webhook cancels it. Beyond the tables: two more billing kinds, `Resumed` and `PlanChanged` (D-301); a checkout refused for a live subscription, which goes to the portal (D-300, D-315); `users.email` normalized (D-260); a 30 s grace for a rotated refresh token and a cross-tab lock (D-258, D-297); outbox rows that record their plan and account (D-267); a sign-out that clears the token while a lost session keeps offline exports (D-291); the storage check's second try (V6 CL Entry 12); `LegalPage` and pricing on the landing page, shipped ahead of V6 for the merchant's review. One protected assertion changed, by row DC-12 of TS §41.2: `export-creator.spec.ts`'s "no token" case now expects the sign-in dialog.

### 8.2 Experiments

| ID | What you do | Pass | If it fails |
|---|---|---|---|
| TE-8 | Send 50 magic links to the major mailbox providers; read the quota terms | At least 95% in the inbox within 30 s; at least 100 emails/day; works from a single verified sender address with no purchased domain | Other candidate. There is no paid remedy: if neither delivers, sign-in is unreliable and that is a launch risk to state plainly |
| TE-9 | Test-mode purchase, renewal, cancel, refund; delay the webhook endpoint | All six `BillingEventKind`s observed in test mode; a timed-out webhook is retried. Approval for live mode is the founding release's box (§8A.8; v1.5, v1.3: §8A.6), not V6's | Second provider behind `BillingProvider` |

### 8.3 The M1 gate (Sun 15 Nov 2026): all P0 features work on dev machines

Walk the PS §9.5 P0 rows one by one on your own machine:

- [ ] Capability detection and the unsupported page
- [ ] Input probe and actionable rejections
- [ ] One-time model download with progress and explanation
- [ ] Local transcription with word timestamps
- [ ] Pattern detector: numbers, enumerations, from-to, emphasized keywords
- [ ] Voice cleanup and loudness normalization, duration unchanged
- [ ] Animated captions in Clean, Bold, Tech
- [ ] NumberReveal, ListReveal, FromTo, KeywordPop
- [ ] 9:16 framing: portrait pass-through, landscape crop with offset
- [ ] Preview player with synced audio
- [ ] Review UI: edit words, toggle events, pick style
- [ ] Export: Creator 1080x1920, Free 720x1280 with watermark
- [ ] "What we changed" summary
- [ ] Account with magic link, entitlement cache, usage receipt
- [ ] Checkout through the merchant of record, Creator plan
- [ ] Settings page "What leaves your device"
- [ ] Allowlisted analytics, all 19 events firing (their formal proof, the network-privacy test, is V8)

### 8.4 Exit checklist

- [ ] Server: `auth_flow.rs`, `entitlement_rules.rs`, `usage_receipts.rs`, `account_delete.rs`, `account_export.rs`, `rate_limit.rs`, `log_redaction.rs`, `no_media_routes.rs`, `webhook_idempotency.rs`, `billing_events.rs` pass against a real Postgres in CI.
- [ ] Crate: `token_roundtrip.rs`, `profile.rs` pass.
- [ ] Client: `auth-machine.test.ts`, `entitlement.test.ts`, `http.test.ts` pass.
- [ ] E2E: `signin.spec.ts`, `export-free.spec.ts`, `checkout.spec.ts` pass.
- [ ] A test-mode purchase at the merchant updates the `subscriptions` row and the next token says Creator.
- [ ] TE-8 and TE-9 outcomes recorded.
- [ ] M1 gate list above fully checked.
- [ ] Tagged `v6`. (v1.3: the feature freeze moves to `v6.11`.)

**Not in V6, and not ever in the MVP.** Passwords, OAuth, a Pro plan, coupons, credit metering, server-side enforcement of export limits (TS §36). (The plan-request quota of V6.2 is server-enforced because it is a real cost, TS §40.7.)

---

## 8A. V6.1 to V6.11 - The long path and the AI-planned editor (v1.3)

**Goal.** After V6, turn the product into a finished-shorts maker on two paths: a long video becomes 5-10 finished shorts, and a raw clip becomes one. One AI edit plan per short drives cuts, music, kinetic captions, auto zoom, structure visuals, B-roll, transitions and the brand kit, and the editor applies it automatically (v1.4). Each path has its own page (v1.4). **(v1.5) Every short gets the benchmark's look by rules, not by chance: motion, polished captions, following the speaker, an automatic picture grade, sound effects and a quality check, each with a switch in the Enhance panel.** Each release is tagged (`v6.1` ... `v6.11`), deployed and demoable, under the §1.2 rules.

**Read first.** TS §41, the compatibility contract, binds every release: flags, declared changes, both-path test runs, and when to stop and ask the founder. PS §12.9-§12.15 say what each feature is. TS §43-§53 give the contracts. This section gives the order, the build lists, the demos and the exit boxes.

### 8A.0 Rules for all eleven releases

**Order and why.** V6.1 first because it holds the largest feasibility risk: if long jobs do not survive on real computers, everyone should learn that now. (v1.4: cloud transcription removed the device-speed half of that risk; reading a 45-minute 4K file in place, keeping the tab alive and the cloud windows remain. The founder kept the order on 2026-11-08.) V6.2 builds the one new contract with the provider. V6.3 changes time and audio while the visual layer is small. V6.4 is the headline feature. V6.5 makes a landscape video usable. Then the look (V6.6) and sound (V6.7), layers (V6.8), chat (V6.9), retention (V6.10) and the riskiest effects (V6.11). The order may change only where the "Depends on" lines allow it.

**v1.5 (founder, 2026-11-09): quality work goes where its dependencies already are, and the numbers stay.** Motion needs only V6.2's zoom path; following the speaker needs V6.5's face tracks; colour and face light need V6.5's shots and faces; cards and layers then use V6.6a's blur; sound effects follow the visual entries of V6.2-V6.8. So V6.2, V6.5, V6.7 and V6.8 grow, and V6.6 becomes V6.6a "Picture" then V6.6b "Graphics". The founding release waits for V6.8, when every element of the benchmark exists, and for E-16's blind test. V6.9-V6.11 stay before launch, built while the founding offer sells.

**Every release does these, in this order:**

1. Branch `v6.n-build` from `main`. Read and record the protected proofs (TS §41.1) as the baseline in `docs/v6/v6changelog.md`.
2. Add the release's flag, off. Build behind it.
3. Keep every §41.1 proof passing on every commit. A change to one needs a row of TS §41.2 or a question to the founder.
4. Run targeted tests per prompt, one CI run per batch, and one full media run per release, with flags off and with shipped flags on (TS §41.1).
5. Merge after the exit boxes; tag; deploy dark; read the release on the live site; turn the flag on in production by a separate deploy.
6. Record the release's measured assumptions in TS §39.3.

**Stop and ask (TS §41.4).** The implementer stops and asks, with a recommendation, on: a protected proof with no declared row; a cost out of proportion to the value; a release at twice its estimate; a new runtime, a model above 50 MB, a new browser host or a paid service; a breaking format change; a changed promise; a failed experiment that offers a choice. The founder decides to keep, defer or drop.

**What the founder does while it builds (not code).**

| When | Founder's work | Decides |
|---|---|---|
| Beside V6, any time before V6.4 | **E-12, optional by the founder's decision of 2026-11-02, and recommended:** test OpusClip, Submagic, Vizard and Descript by hand on one video; free shorts for 20 YouTubers from a throwaway script; ask who would pay (the founding plan is sold from the founding release after V6.8, v1.5) | Whether stitched shorts lead the pitch; the order of V6.8-V6.11 |
| V6.4 | Mark postable moments on at least 10 real long videos: the labelled set for TE-19 and E-13 | Whether the Director is good enough to keep going |
| After V6.4 | **Private alpha** with 10-15 YouTubers on their own videos (the long-path half of E-2) | What to fix first; which of V6.6b's parts and V6.7-V6.11 matter |
| V6.5 (v1.5) | At least 10 two-person podcast videos (wide two-shots), with the speaker marked per sentence, for TE-27; kept outside git | Whether following the speaker ships as the default |
| V6.6a (v1.5) | TE-29's panel: 20 shots, graded and ungraded side by side, unlabelled | Whether the automatic grade ships |
| After V6.8 (v1.5; v1.3: after V6.6) | **E-16, the blind test** (PS §19): the same 3 videos through Submagic and Offcut, 20 viewers; then **first public demos,** made with the product; **the founding release (§8A.8): live payments, then the founding-plan offer (E-5)**; waitlist, affiliate talks | Whether the look is good enough to sell |
| V6.11 | The person-mask panel on 20 shorts (TE-25) | Which effects ship |
| Any time before the quality pass (v1.4) | **The benchmark:** a few Submagic-made shorts downloaded from YouTube and the caption references (`sh2clips/documents/`), kept outside git, as the repository is public | What "good enough" looks like |
| After V6.11 | **The quality pass (§8A.12), then feature freeze.** Marketing runs alongside V7-V10 | What to tune; the launch date |

**Estimate (founder's pace: V4, 38 prompts in about 8 days; V5, 8 prompts in about 2 days; V6.1, 12 prompts in under 2 days).** Rough, with no rework and no failed experiment. A release at twice its estimate is a stop-and-ask.

| Release | Prompts (rough) | v1.5 change |
|---|---|---|
| V6 | 14, as built (P137-P150) | |
| V6.1 | 12, as built (P171-P182) | |
| V6.2 | 19 (v1.4: 10) | Motion by rules +2, easing curves +1, the hook lockup +2 (moved from V6.6), glow, punch words and emoji under the keyword +1, the Enhance panel with Voice cleanup +3 |
| V6.3 | 16 | |
| V6.4 | 14 | |
| V6.5 | 14 (v1.4: 8) | Following the speaker, with the split and `composite.rs`'s second path, +6 |
| V6.6a | 9 (new) | The blur +2, automatic grade and camera matching +5, face light and vignette +2 |
| V6.6b | 21 (v1.4: V6.6, 20) | The hook title -2 (to V6.2); fonts, presets and word pages +3 |
| V6.7 | 10 (v1.4: 7) | Synthesized sounds placed by rules +1, voice tuning +2 |
| V6.8 | 21 (v1.4: 19) | Zoom transitions +1, the blur already built -1, the quality check and `AddItem` +2 |
| V6.9 | 10 | |
| V6.10 | 10 | |
| V6.11 | 10 | |
| Quality pass | Set by the founder (§8A.12) | |
| **Total** | **about 180 with V6 and V6.1; 154 still to build, about 6-8 weeks** | **+30 against v1.4.** HDR (dropped by the founder) and the music library would have added about 7 |

**Risk.** Two additions rest on experiments: following the speaker (TE-27) and the automatic grade (TE-29). If either doubles its estimate, that is up to about 11 more prompts, roughly 2 days, and the stop-and-ask rule brings it back to the founder.

V7-V10 keep their five weeks, so public launch is about **February to early March 2027** (ESTIMATE; v1.4 said February). The founder sets the date when V6.11 closes.

---

### 8A.1 V6.1 - Long source (M1.16)

**Goal (v1.4).** A long video up to 45 minutes, 4K or HEVC, dropped on its own page, is read in place and reaches a resumable master transcript, transcribed in the cloud. Nothing else uses it yet. With the flags off, nothing has changed. *(v1.3: 20 minutes and 16 GB, copied whole, transcribed on the device.)*

**Demo (dev flag).** Open "Make shorts from a long video" and drop a 45-minute 4K file. Watch sound and transcription progress; nothing is copied first. Kill the tab at window 16. Reopen, press Continue, and see it finish from window 16 without the file. Kill it again during the sound step: it asks for the same file. Turn the cloud off: the job pauses with its reason and Continue.

**Depends on.** V6 (the gate needs an account); the cloud transcription release (`encodeForCloud`, `POST /transcribe`, `fake-api.ts`'s cloud option).

**Build (TS §43).**

| Create or change | Purpose |
|---|---|
| `web/src/config/features.ts`, Vite `define` | The flags `long_video` and `high_res` (TS §41.3) |
| `offcut-types`: `source.rs` with `SourceId`, `Entry`, limits of TS §43.2, `RejectReason::Hdr` | Types, additive |
| `offcut-mp4`: `route_probe(p, decode_supported, entry, flags)` in `validate.rs`, `hevc.rs` | Routing by page. `validate_probe` is untouched and rule 1 calls it |
| `ui/pages/LongPage.tsx` at `/app/long`; one link on `/app`; "Make shorts from it" on `REJECT_DURATION`'s panel; the landing page's "Start free" offers both | The two pages (TS §43.1, DC-1) |
| `workers/media/file-random-access.ts`; `probeFile` in `import.ts` | Reading the `File` in place |
| `audio-decode.ts`: `decodeAudioRegion` and `writeWholePcm16` over a `RandomAccessReader`, rate correction on the region path | Sound in windows to `pcm16.f32`. `decodeAudio` is untouched |
| `usecases/transcribe-source.ts` | Cloud windows, checkpoints, retries, the four pause reasons (TS §43.6). `transcribe-cloud.ts` and `transcribe` are untouched |
| Server: the window header on `POST /transcribe`, `LONG_AUDIO_PER_USER_DAY`, `TRANSCRIBE_BUDGET_HOUR_S` and `TRANSCRIBE_BUDGET_DAY_S` (TS §22.1) | The long path's audio budget |
| `offcut-text`: `sentences_long.rs` tests, `MAX_SENTENCE_MS` split | Sentences carry weight from V6.4 |
| `persistence/`: IndexedDB v3, `sources-repo.ts`; `opfs.ts` paths; `quota.ts` eviction | Storage |
| `state/source-store.ts`, `machines/source-machine.ts` with `paused` and `awaiting_file` | The job's state |
| `usecases/import-source.ts`, `run-source-pipeline.ts` | The job; wake lock; resume; asking for the same file; title progress |
| `ui/components/SourceProgress.tsx` | Stages, pause, continue, the pause reasons |
| `models/model-manager.ts`, `config/models/` | Several manifests (DC-4) |
| Fixtures: `long_12min.mp4`, `long_45min_4k_hevc.mov` (outside git, generated), `rej_too_long_46min.mp4`, `rej_8k.mp4`, `rej_hdr_hlg.mov` | Test inputs |

**As built (V6.1, `docs/v6.1/v6.1implementation.md` and V6 CL Entries 25-37).** Twelve prompts, 171-182, on `v6.1-build`. `route_probe(p, detail, decode_supported, entry, flags)` lives in `offcut-mp4/src/route.rs`, re-exported by `validate.rs`, with `hevc.rs` and `colr.rs`; the reading is the media worker's `routeFile` handler over `file-random-access.ts` (`import.ts` is frozen, so there is no `probeFile`); the sound is `workers/media/audio-region.ts` (not `audio-decode.ts`, frozen) with `offcut-dsp`'s `StreamResampler`. Storage is IndexedDB structure version 2 (`DB_VERSION` stays 1), one source kept; **`quota.ts` eviction moved to V7** (REOPENED BP §8A.1), as did clearing older clips from the disk check. No model manifest changed (DC-4 is not used). The landing page is unchanged (D-425). The three analytics events wait for V6.4 (V6 CL open item 26). Windows overlap by 10 s (TE-18's first read). The tests as built: `route-long.spec.ts`, `long-resume.spec.ts`, `high-res.spec.ts`, `flags-identity.spec.ts` in the Playwright project `media-flags` over the flags-on build `dist-flags/`, and `audio-region.test.ts` in place of `audio_region.rs`.

**Experiments.** TE-17 (reading in place), TE-18 (cloud windows), TE-20 (hidden tab, sleep), TE-21 (HDR), TE-12 re-read on long files (moof, audio rate).

**Exit.**

- [ ] `route_probe.rs`: every rule from both pages; rule 1 equals `validate_probe` on every existing fixture.
- [ ] `demux_fixtures.rs` (hvcC), `audio_region.rs`, `sentences_long.rs`, `source-machine.test.ts`, `transcribe-source.test.ts`.
- [ ] `route_long.spec.ts`, `high-res.spec.ts`, `long-resume.spec.ts`, `flags_identity.spec.ts`; `transcribe_route.rs` with the window budget.
- [ ] Peak memory under 1.5 GB on the 45-minute file on D1.
- [ ] TE-17, TE-18, TE-20, TE-21 recorded. If TE-18 or TE-20 fails: stop and ask.
- [ ] Every §41.1 proof unchanged except the rows DC-1 to DC-4. Tagged `v6.1`.
- [ ] Groq's Dev tier is on, or the founder has decided to stay on the free tier for now (TS §52).

**Watch.** Fragmented MP4 from OBS, PCM audio from cameras. If the corpus shows these are common, this is a stop-and-ask, not a quiet addition. Reading a file in place on a slow external drive: TE-17 reads it.

---

### 8A.2 V6.2 - AI edit plan (M1.9)

**Goal.** One AI request per clip returns a validated plan; the renderer draws it; a failed request never blocks anything. As the old V6.1, plus the spend cap and `DrawCmd::Image`. **(v1.5)** And the first half of the benchmark's look on every clip: a hook title over the first 2 seconds, glowing captions with punch words and emoji under keywords, and motion on every talking-head shot, with the Enhance panel's Voice cleanup and Motion switches.

**Demo.** The reference clip with the AI plan on opens already edited: emphasis, a snap zoom on one word and a slow push across a sentence (auto zoom, v1.4), an icon and an emoji. Switch any of them off; undo. Turn off the API: the same clip reaches `ready` on the local rules, and the feed says so once. **(v1.5)** The short opens on a hook lockup; one word jumps to twice the size; an emoji sits under a keyword; the shot never holds still. Open Enhance: switch Motion off and the drift stops at once; switch Voice cleanup off and the sound is the original, levelled; the next clip remembers both.

**Depends on.** V6.

**Build.** `server/src/plan/` (`route.rs`, `provider.rs`, `quota.rs`, `spend.rs`), `migrations/0002_ai_usage.sql` (TS §52); `usecases/request-plan.ts`, `persistence/plans-repo.ts`; settings switch and provider text; catalog kinds `Emphasis`, `IconPop`, `PunchIn` (with `motion: Snap | Push`, the auto zoom of TS §47.2, v1.4), `Emoji` with drawing and goldens; `DrawCmd::Image` and a closed icon and emoji bundle; text extras panel with copy buttons; DC-7 on V-1; prompt additions of TS §47. **(v1.5, TS §47.2, §47.3, §47.5, DC-16):** `HookTitle` with `offcut-scene/src/lockup.rs`; `StyleSpec.glow`; `Emphasis.size`, `Emoji.place`; `offcut-scene/src/motion.rs` and `easing.rs`; the flag `motion`; `offcut-types/src/enhance.rs`; `lvl48` beside `out48` in prepare and `assemble`'s voice choice; `ui/components/EnhancePanel.tsx`; `meta.enhanceDefault`; `flags_identity.spec.ts`'s two cases; `export_done.enhance_off`.

**Experiment.** TE-15: provider, price, parse rate, latency, data terms; labelled transcripts.

**Exit.**

- [ ] `validate.rs` (V-1 change), `catalog_complete.rs`, `catalog_snapshot.rs`, `plan_route.rs`, `ai_cap.rs`, `ai_quota.rs`, `log_redaction.rs` (plan), `run-pipeline.test.ts` (every fallback).
- [ ] `privacy-network.spec.ts` revised: plan on and off.
- [ ] E-11 instrumentation live; TE-15 recorded; `AI_MONTHLY_CAP_USD` set by the founder.
- [ ] (v1.5) `motion.rs` (segments, alternation, no motion on `Screen` shots, the zoom cap), `easing.rs`, `lockup.rs`, `enhance.rs` (each switch off equals the stage absent), goldens for glow, punch, emoji under, the hook and a motion frame; `enhance-panel.spec.ts` (each switch flips the preview within the edit budget; the choice is remembered); `flags_identity.spec.ts` under `Enhance::v5()` (DC-16).
- [ ] Goldens for existing styles unchanged. Tagged `v6.2`.

---

### 8A.3 V6.3 - Cuts and the Assembly (M1.10)

**Goal.** Pause and silence cuts, as the old V6.2, and the `Assembly`: a short is a virtual clip made of parts of a source, in any order, while everything downstream stays as it is.

**Demo.** The 20 s long-pause fixture exports shorter, with no click and in sync; restore a cut. A dev-only fixed `Assembly` takes three parts of the 60 s fixture out of order: preview and export are in sync, with captions continuous across joins.

**Depends on.** V6.2 for cuts; V6.1 for sources.

**Build (TS §40.1, §40.6, §44).**

| Part | Create or change |
|---|---|
| Cuts | `Cut` arms in validation; `timemap::build` with silence snapping; `assemble` splices; jump-cut alternation; cuts list with restore; filler switch; `ChangeSummary` cut fields; `set_scene` (DC-5) |
| Assembly | `offcut-types/src/assembly.rs`, `SourceMs`; `offcut-text/src/rebase.rs`; `offcut-dsp/src/regions.rs`; `workers/render/source-position.ts`; second decode session across joins in `video-source.ts`; `join_points` in the scene; cache key field; `clips` row fields |
| Use-case | `usecases/prepare-short.ts`: everything from `ShortSpec` to `ready`, driven by a test fixture until V6.4 |
| Part copies (v1.4) | `offcut-mp4/src/remux.rs` (`remux_range`, H.264 and HEVC); `SourceMedia` in `assembly.rs`; `source-position.ts` and the export loop read part files; `clips/<clipId>/parts/` in `opfs.ts` (TS §44.8) |

**Experiment.** TE-22: preview across a join on long-GOP 4K.

**Exit.**

- [ ] `timemap_build.rs`, `program_splice.rs`, `cuts_applied.spec.ts`.
- [ ] `assembly.rs`, `assembly_readers.rs`, `rebase.rs`, `regions.rs`, `remux.rs`, `source_media.rs`, `stitched.spec.ts` (its parts copied by `remux_range`); `preview-sync.spec.ts` join case; TE-17's part-copy reading.
- [ ] `program_identity.rs` and `timemap_identity.rs` unchanged and passing: `Assembly::whole` is V5 (INV-26).
- [ ] A dropped clip's render-cache key is unchanged. Tagged `v6.3`.

**Watch.** This release touches the most protected code. If `Assembly` needs more readers than the four listed, stop and ask. `SourceMedia::whole` must leave a dropped clip's reading exactly as V5's.

---

### 8A.4 V6.4 - Director (M1.17)

**Goal.** The headline feature. Drop a long video; get a list of 5-10 shorts of whole sentences; open any; export the ones you keep. A failed AI request still gives a usable list.

**Demo.** A 12-minute talk becomes a list with titles and reasons. One short joins a claim from minute 3 with its proof from minute 8. Delete a part; restore it. Close the tab, reopen without the file: every short still opens (part copies). Export three. All pass the verifier.

**Depends on.** V6.1, V6.2, V6.3.

**Build (TS §45).**

| Create or change | Purpose |
|---|---|
| `offcut-plan`: `shorts.rs`, `shorts_prompt.rs`, `shorts_rules.rs` | Validator S-1 to S-10, prompt, offline list |
| `offcut-api-types/src/ai.rs`; `server/src/plan/shorts.rs`; grants in `quota.rs` | `POST /shorts` |
| `usecases/request-shorts.ts`; `run-source-pipeline.ts` complete, with `copying_parts` (v1.4, TS §44.8); `prepare-short.ts` complete; `open-short.ts`; `export-shorts.ts` | The flow |
| Server: `SHORTS_BODY_LIMIT = 160 kB` on `/shorts` (DC-15) | 45-minute requests |
| `state/shorts-store.ts`; `persistence/shorts-repo.ts` | The list |
| `ui/pages/ShortsPage.tsx`, `ui/components/ShortCard.tsx`; a parts list in `CutsSection`; "Back to all shorts" | The UI. The editor itself is unchanged |
| `analytics`: the events of TS §45.9 | E-13 |
| `sign-in gate`: `gated_signin` in the source machine; Free's 1-a-month copy | Limits before the work |

**Non-code.** The labelled set of at least 10 real long videos. **Then the private alpha.**

**Experiment.** TE-19: Director quality through two models on the labelled set.

**Exit.**

- [ ] `shorts.rs`, `shorts_prompt_snapshot.rs`, `shorts_rules.rs`, `shorts_route.rs`, `ai_quota.rs`, `log_redaction.rs` (shorts).
- [ ] `run-source-pipeline.test.ts`: each failure kind ends with a usable list.
- [ ] `long-to-shorts.spec.ts` with a fake provider: 5 exported shorts, 11 checks each. API down: the rules list.
- [ ] A worst-case 45-minute request fits in 160 kB; `no_media_routes.rs` passes with DC-15's exception and nothing else changed.
- [ ] TE-19 at or above its line, or the founder has decided. Tagged `v6.4`.
- [ ] `long_video`, `assembly`, `director` flags on in production only after the alpha reading.

---

### 8A.5 V6.5 - Vision and framing (M1.18)

**Goal.** Landscape video becomes a well-framed vertical short, and the Director knows what is on screen. No frame leaves the computer. **(v1.5)** In a two-person shot, the frame follows whoever is speaking.

**Demo.** A speaker walking across a landscape shot stays framed; the crop holds still while the speaker stands still, and does not pan across a cut. A screen-recording section is skipped by the Director, whose reason mentions it. **(v1.5)** A two-person podcast short cuts to each speaker at the start of their sentence; when both talk, it shows both stacked; "Show both" in the Framing panel splits every two-person shot.

**Depends on.** V6.4.

**Build (TS §46).** `crates/offcut-vision/` (and its row in `ALLOWED_CRATE_DEPS` and TS §7: depends only on `offcut-types`); `offcut-types`: `vision.rs`, `frame_path.rs`; `workers/asr/vision-runtime.ts`; `config/models/face.json`; `media.worker` frame sampling; coarse pass in `run-source-pipeline.ts`; fine pass in `prepare-short.ts`; `ShotLine` in the Director request; `framing.rs` with `FramePath` (DC-9); `offset` as a shift on the path; the 4K note. **(v1.5, TS §46.6, DC-17):** `offcut-vision/src/speaker.rs` (face tracks, mouth activity, the speaker decision), `FrameLayout::Split`, `composite.rs`'s second path for two `Video { Main }` commands (DC-8), `EditState.speaker` and the Framing panel's three choices, `ShotLine.people`, the flag `speaker_framing`.

**Experiment.** TE-23: face detector speed, accuracy and licence. **(v1.5)** TE-27: the right person on screen in 10 real two-person shorts; if it fails, stop and ask.

**Exit.**

- [ ] `shots.rs`, `tracking.rs`, `vision_summary_prompt.rs`.
- [ ] (v1.5) `speaker.rs`, `speaker-framing.spec.ts`, `composite.rs` (simple path identical); TE-27 recorded.
- [ ] `framing.rs`: `FramePath::none` equals V5 on every existing case; goldens unchanged.
- [ ] `vision-pass.spec.ts`; `privacy-network.spec.ts`: no frame in any request (INV-28).
- [ ] Coarse pass at most 0.15× the duration (about 7 min on 45 minutes), reading the file in place; fine pass under 20 s per 60 s short on D1, over part files. TE-23 recorded, with its licence. Tagged `v6.5`.

---

### 8A.6 V6.6 - Finished look, in two releases (v1.5): V6.6a Picture, V6.6b Graphics

**v1.5 (founder, 2026-11-09).** v1.3's V6.6 grew past 30 prompts with the benchmark's additions, so it ships as two releases, each with its own branch, gate and tag. The founding release no longer follows it: it moves to after V6.8 (§8A.8).

#### 8A.6a V6.6a - Picture (M1.23)

**Goal.** Every shot looks graded and every camera matches, without the person touching a control, and a good picture is left alone. The blur that later layouts and cards need is built once, here. What an engine on the computer can do that a server tool like Submagic does not.

**Demo.** A dim, warm webcam shot comes out with the face lit and neutral whites. A two-camera podcast's angles match in colour. A shot that was already good shows no change, and the Enhance panel says "Your picture already looks right". Switch "Colour & light" off: the original returns at once. A `Full` layout test frame shows the landscape shot over a blurred copy of itself.

**Depends on.** V6.5 (shots, faces, colour statistics from the fine pass).

**Build (TS §47.4, DC-8, DC-18).** `offcut-render/src/blur.rs` and its use in `composite.rs`'s second path; `offcut-types/src/grade.rs`; `offcut-vision/src/grade.rs` (`shot_grades`, camera matching, face light, vignette); the fine pass's colour statistics; `Video.grade` and the graded variant of the video pass; `Enhance.picture` and the panel's "Colour & light" row; the flag `picture`.

**Experiment.** TE-29: the founder's side-by-side panel on 20 shots; the frame budget on D1. If it fails: narrower bounds, then camera matching only, then ask.

**Exit.**

- [ ] `grade.rs` (each step's bound; an already-good shot gives `None`; a grade changes only at a shot boundary), `blur.rs`, goldens with a grade and with the blur; `golden_frames.rs` and `composite.rs`: every `grade: None` list and every simple list unchanged.
- [ ] `enhance.rs`: Colour & light off equals no grade. TE-29 recorded. Tagged `v6.6a`.

#### 8A.6b V6.6b - Graphics (M1.12, M1.13, M1.14)

**Goal.** A short looks made by an editor. Built in five parts, each with its own gate, in value order. **The founder kept all of it on 2026-11-02, and again on 2026-11-09** (cards B and part D stay before launch); a part whose cost grows past its worth is a stop-and-ask, by name. (v1.5: the hook title moved to V6.2.)

| Part | Build | Exit, besides goldens per kind and style |
|---|---|---|
| **A** Core visuals | `ListBuilder`, `NumberMoment`, `QuoteCard`, `Versus`; V-9 density rule; (v1.5) the glass treatment over V6.6a's blur | V-5 cases for each number-bearing kind |
| **C** Kinetic captions (v1.5: before B, as the benchmark weighs captions above cards) | `CaptionMotion` rows, per-glyph animation; **(v1.5, TS §47.3)** four OFL display fonts (TE-28), six presets with our own names, word pages of one word and one to three words. v1.4's caption families: the lockup is built in V6.2; the mixed-font pair beside the head is a cost check by name | `frame_at` within budget at the heaviest preset on D1; TE-28 recorded |
| **E** Furniture | `ProgressBar`, `EndCard` with a `Hold` segment | End-card export duration equals the map's |
| **B** More cards | `DefinitionCard`, `StepTracker`, `Timeline`, `Checklist` | Same as A |
| **D** Shapes | `Video.mask` in the renderer's second path; `ShapeFrame`; shape `Transition` at topic shifts | **`composite.rs`: a simple list takes the old path, byte for byte (DC-8)**; render budget held |

**Demo.** Switch between the six new presets live; a one-word-at-a-time preset; "three reasons" builds item by item; "from 800 ms to 120 ms" becomes two bars; a quote on glass; the speaker framed in a circle; a shape wipe between two topics; a progress bar and an end card.

**Depends on.** V6.2 for the catalog; V6.3 for `EndCard`'s map; V6.6a for the blur and the second path. Parts A, C and E may ship before B and D.

**Exit.** The part rows above; `catalog_complete.rs`; existing goldens unchanged. Tagged `v6.6b`.

---

### 8A.7 V6.7 - Music and sound (M1.11)

**Goal.** Music that dips under every sentence, and short sound cues at moments and joins. **(v1.5)** Every visual entry lands with a sound made by Offcut itself, and the voice is tuned only as far as it needs.

**Demo.** A short with music ducking under speech, fading at the end, a whoosh at a join. **(v1.5)** Whooshes as cards come in, pops on emoji, a riser before the hook, never two within 0.8 s. A muddy laptop-mic recording comes out clearer; a studio-mic recording is left nearly as it was. Switch "Sound effects" off: only the plan's own cues remain.

**Depends on.** V6.3. V6.2 for the music cue.

**Build (TS §40.6, §47.1, DC-19).** Music import to OPFS (the person's own track only, v1.5), picker, ducking and mix in `assemble`, `MixInput` with `none()`, plan `MusicCue`; `SoundCue` kind; `MusicChoice` in `EditState` (default none). **(v1.5)** `offcut-dsp/src/sfx.rs` (the six synthesized sounds) and `cues_for` over the scene's visual entries; `offcut-dsp/src/voice_tune.rs` (denoise by need, the three-band EQ) and the flag `voice_tune`; the panel's "Sound effects" and "Music" rows; the noise-floor line of `EnhanceReport`.

**Experiment.** None. (v1.5: TE-16 is closed. The founder chose the person's own music only, and the sound effects are synthesized, so no licence is read.) The sounds' parameters and levels are tuned by ear in the quality pass.

**Exit.** `mix_ducking.rs`, `sound_cues.rs`, `music-import.spec.ts`; verifier check 7 on a music export; `program_identity.rs` unchanged. **(v1.5)** `sfx.rs` (each sound deterministic: the same samples on every run; peak and loudness within bounds), `cues_for` (gap and count limits; an item switched off loses its sound), `voice_tune.rs` (a clean fixture changes by at most 1 dB in any band; a muddy one is cut), `flags_identity.spec.ts` under `Enhance::v5()`. Tagged `v6.7`.

---

### 8A.8 V6.8 - Layers, B-roll, asset box (M1.19)

**Goal.** Several videos in one frame. B-roll from the person's own clips and (v1.4) from Pixabay stock, placed by the plan or by hand on a word; real transitions at joins; a split layout for screen recordings.

**Demo.** Drop an office clip, label it. The plan places it on "our headquarters". On "black hole", the plan places a Pixabay clip, marked "From Pixabay". Click another word, "Add B-roll here", pick a lecture clip of your own, drag the window to the 4 seconds you want; undo; redo. A dissolve at a join. A screen recording shows the screen above and the face below. Turn the stock switch off: nothing is searched. **(v1.5)** A zoom transition at a join, with a whoosh; the B-roll drifts like the speaker's shots; a short with a 4-second still stretch comes back with a snap zoom there, listed as an item that can be switched off.

**Depends on.** V6.6b part D (masks), V6.6a (the blur), V6.5 (regions, and the second render path, v1.5).

**Build (TS §48).** `VideoSlot`, `Scene::asset_at`, `VideoSources`; `composite.rs` runs up to `RUNS_PER_FRAME`; decoder limit of three; `offcut-types/src/assets.rs`, `AssetRef`, `AssetOrigin`; `persistence/assets-repo.ts`; `usecases/add-asset.ts`, `remove-asset.ts`; `ui/components/AssetBox.tsx`; `Broll` (with `BrollSource`), `JoinTransition`, `Layout` kinds; `Part.handle`; `AssetLine` and `stock` in plan requests; `keep_audio` in `MixInput`; "What leaves" rows for labels and stock search. **v1.4:** `plan_ops` with `PlaceAsset` and `RemoveAsset`, `effective_plan`, `validate_ops` (TS §49.2, the rest in V6.9); "Add B-roll here" in the caption editor; the part picker; `server/src/broll/` (`StockProvider`, `PixabayProvider`, the hashed 24 h cache, `POST /broll/search`); `net/stock-fetch.ts`, `usecases/resolve-stock.ts`, the search panel; `cdn.pixabay.com` in `allowlist-hosts.ts`, CSP and `check-hosts.mjs`; the flag `stock_broll` and the settings switch; `PIXABAY_API_KEY` in Render before the deploy that reads it; one Terms of Service line on the rights to added clips. **(v1.5, TS §47.6, §48.3):** `ZoomIn` and `ZoomOut`; motion on B-roll; `offcut-scene/src/quality.rs` and the fixes in `prepare-short.ts` as `PlanOp::AddItem` with `origin: QualityFix`; `quality_corpus.rs` in CI.

**Experiment (v1.4).** TE-26: Pixabay's terms re-read, relevance of plan-written queries, the browser fetch under COEP.

**Exit.**

- [ ] `composite.rs`: the simple path identical; goldens unchanged. New goldens for cover, contain, picture-in-picture, split, full.
- [ ] `handles.rs`, `join_transition.spec.ts`, `asset-box.spec.ts` (placing by hand, the part picker, undo).
- [ ] `plan_ops.rs` (`PlaceAsset`, `RemoveAsset`), `broll_search_route.rs`, `resolve-stock.test.ts`, `stock-broll.spec.ts`, `log_redaction.rs` (a search query).
- [ ] Verifier 11/11 on an export with own and stock B-roll and transitions; TE-14 re-read at three decoders.
- [ ] (v1.5) `quality.rs` (each rule Q-1 to Q-5 on a fixture), `plan_ops.rs` (`AddItem` applies, refuses and undoes), `quality_corpus.rs` recording each rule's failure count as the baseline for later releases.
- [ ] `privacy-network.spec.ts`: labels only in AI bodies; search words only in `POST /broll/search`; `cdn.pixabay.com` only with the switch on; no asset bytes. TE-26 recorded. Tagged `v6.8`.

**E-16, then the founding release, right after `v6.8` (v1.5: the founder's decision of 2026-11-09; v1.3 put it after V6.6; moved from V9, TS §22.5, E-5).** First the blind test (PS §19 E-16): 3 source videos through Submagic's defaults and Offcut's defaults, 20 viewers; its result is recorded in the changelog, and a loss puts its reasons at the top of E-15's list and goes to the founder. Then the founding release: no new version number, a deploy with its own commits on the branch of V6.9. It adds no product feature. Early paying strangers test the price directly. With it come the first public demos made with the product.

- [ ] `/legal/:doc`: Terms of Service, Privacy Policy, Refund and Cancellation Policy (14 days), written as §10.3 says, with public pricing and a contact route. **Built ahead of V6 on 2026-11-02** (`site-pages`), since Dodo's FAQ says its review reads these on the live site. This box: re-read them against what is built by then, and add the seller's name if the founder chooses.
- [ ] Dodo approves live payments, and tax settings are confirmed.
- [ ] Vercel's plan terms are recorded, and the founder accepts the non-commercial-terms risk in writing (TS §29), since real money now passes through the site. V9 listed this; it comes first now.
- [ ] Copy says only what BP §10.3 allows. `privacy-network.spec.ts` is V8's, so before it exists the founder reads one network capture of a clip on the live site, as V5 did, and records it. PS §17 ties the claim to that test.
- [ ] Live products, key and webhook secret in Render with `BILLING_API_BASE_URL=https://live.dodopayments.com`. The live webhook URL is registered. Test-mode `subscriptions` rows are wiped. This makes every offer purchasable, not only the founding one.
- [ ] One real purchase and one refund, read in `subscriptions`.
- [ ] Then `FOUNDING_OFFER_ENABLED=true`, and the offer goes to the waitlist and alpha people. Target: at least 5 paying strangers (PS §19 E-5).

---

### 8A.9 V6.9 - Chat editing (M1.20)

**Goal.** Ask in words. Every message is typed operations through the same validator, one undo step, and the preview updates at once where sentences do not change.

**Demo.** "Make this shorter." "No emoji." "Use my office clip when I say headquarters." "Add the part where I talk about pricing." "Show 50% here" when the speaker never said it: refused, in plain words.

**Depends on.** V6.4, V6.8 (`PlaceAsset`). Ops for later kinds are added with their kinds.

**Build (TS §49).** `offcut-types/src/plan_ops.rs` and `offcut-plan/src/plan_ops.rs` (v1.4: started in V6.8 with `PlaceAsset` and `RemoveAsset`; this release adds the other operations), `chat_prompt.rs`; `EditState.plan_ops` (from V6.8); `AddSentences` asking for the original file to copy the new part (TS §44.8); `server/src/plan/chat.rs`; `usecases/send-chat-edit.ts`, `state/chat-store.ts`; `ui/components/ChatPanel.tsx`; settings row for chat text; `chat_edit` analytics (enums only: `ops`, `refused`, `outcome`).

**Exit.**

- [ ] `plan_ops.rs`: each op applies, refuses, and undoes to equal; the property that no unspoken number reaches the scene.
- [ ] `chat_prompt_snapshot.rs`, `chat_route.rs`, `log_redaction.rs` (a marker in a chat message), `chat-edit.spec.ts`.
- [ ] `privacy-network.spec.ts`: chat on and off. Non-part ops stay within the 150 ms edit budget. Tagged `v6.9`.

---

### 8A.10 V6.10 - Brand kit, creator memory, queue (M1.15, M1.21)

**Goal.** Reasons to stay. A logo, colours and fonts on everything; plans that fit better each week; a whole back catalogue processed without counting minutes. **Feature-complete except the gated effects.**

**Demo.** Set a kit once: every preset, card and item wears it. Delete three Trivia shorts: the next Director list has fewer, and settings says why. Queue three videos and leave.

**Depends on.** V6.6 and V6.4; V6.1 for the queue.

**Build (TS §50).** `BrandKit`, logo through `DrawCmd::Image`, `Theme` and `FontSet` from the kit, Creator-only exports (INV-9); `offcut-types/src/creator.rs`, `persistence/creator-memory-repo.ts`, prompt block, settings panel with Reset and off; `state/queue-store.ts`, `usecases/manage-queue.ts`, `ui/components/QueuePanel.tsx`.

**Exit.** Golden matrix under a test kit, all §47 kinds; `profile.rs` brand cases; `creator_memory.rs` (no text by type, INV-31); prompt snapshot with fixed memory; `queue.spec.ts` (one of three fails; the others finish; correct after a reload). Tagged `v6.10`.

---

### 8A.11 V6.11 - Person mask (M1.22)

**Goal.** Text behind the speaker and background change, each only if it passes the quality gate.

**Demo.** A hook whose big words sit behind the speaker's head. A plain room replaced by a gradient on a steady shot.

**Depends on.** V6.8 (slots, second render path), V6.5 (shot kinds).

**Build (TS §51).** `workers/asr/person-runtime.ts`, `config/models/person.json`; `usecases/prepare-matte.ts`; matte encoded to `clips/<id>/matte.mp4` through existing encoders; `VideoSlot::Matte`, `Video.matte`; `TextBehind`, `Background` kinds; gating by shot kind and matte confidence.

**Experiments.** TE-24 (model speed, size, licence), TE-25 (the gate: founder panel, flicker, budget).

**Exit.**

- [ ] `matte_goldens`, `prepare-matte.spec.ts`, INV-12 extended, memory read, verifier 11/11 on each effect.
- [ ] TE-25 recorded **per effect**. A failed effect stays off, and the founder is asked. Launch does not wait on it.
- [ ] **M1.x gate: every PS §9.5 V6.x row works on dev machines, or the founder has dropped it.** Tagged `v6.11`.

**Tagged `v6.11`. Feature freeze starts here.**

---

### 8A.12 Quality pass (v1.4, the founder's decision of 2026-11-08)

**Goal.** Shorts that stand beside the best of the category. The founder chose to build every feature first, then inspect and improve.

**Input.** The founder's benchmark: Submagic-made shorts downloaded from YouTube, and the caption references in `sh2clips/documents/`, all outside git (the repository is public).

**How.** The founder takes 3-5 source videos (both paths), makes shorts in Offcut, and compares them side by side with the benchmark: captions (motion, timing, fonts, placement), auto zoom, cards, B-roll, transitions, sound, pacing. Each difference becomes a row of a checklist in the changelog, with the agent's estimate.

**v1.5.** E-16's result comes first. The default preset, motion drift, sound levels, grade bounds and the quality check's limits are set here from the benchmark, since most people never open settings. The Creator export bitrate (8 Mb/s, TS §21.2) is re-read against the benchmark under TE-4, as constant motion is harder to encode; a change needs its §41.2 row.

**What changes.** Presets, timings, theme values, prompt text and catalog limits, inside the existing contracts. A new item kind or a new render path is a stop-and-ask (TS §41.4). Every §41.1 proof holds; a golden that moves needs a row of TS §41.2.

**Exit.** E-15 (PS §19): every row fixed, tuned or knowingly accepted by the founder. Its length is the founder's; V7 starts after it.

---

## 9. V7 - Survive real life: restore and failure recovery (M2.1; week 6: 16-22 Nov)

**From here on, no features.** V7 to V10 make the V6.11 product reliable, provably private, fast enough and launchable. A change that adds a capability is out of scope until after the M2 gate (TS §28: no P1 item starts before M2 passes).

**Goal.** The product behaves well when things go wrong: the tab is reloaded, the browser crashes, the disk fills up, a worker dies, the GPU device is lost, a decoder chokes on a bad sample.

**What you can demo.** Reload the tab mid-review: you are back at the preview with your edits, with no second transcription. Kill a worker from the browser's task manager: a specific message, a retry that works.

**Depends on.** V6.11.

**v1.3: long jobs raise the stakes here.** V6.1 built the source job's checkpoints; V7 proves the rest. A reload during `preparing_shorts`, a crash in a batch export, a disk filling mid-copy of the parts, a queue with a bad file, a long job paused by the cloud (v1.4), and restore of a short whose part copies were evicted all join `restore.spec.ts`, `failure-recovery.spec.ts` and `long-resume.spec.ts`.

### 9.1 Build

| Create or complete | Purpose |
|---|---|
| `web/src/persistence/quota.ts` | `ensureFree(bytes)`: evict older exports, then clips beyond the 5 most recent, then oldest clips; never the current clip; never the model (TS §23.3) |
| `web/src/usecases/restore-clip.ts`; `start-app.ts` complete | C-13: all pieces present gives `ready` with no re-transcription; source only re-runs the pipeline; source missing deletes the row; an in-flight export is never resumed |
| `opfs.sweepTemp()`; write order in the repos | OPFS file first, then the IndexedDB row; orphans cleaned at startup |
| `persistence/db.ts` migrations | Numbered upgrade functions; a value with a newer `schemaVersion` is treated as absent |
| `web/src/ui/components/RecentClips.tsx` | Date and duration only; no file names |
| `web/src/usecases/clear-local-data.ts` | C-17: refused while busy; removes all OPFS content and the clip stores; keeps the receipt outbox, the entitlement and `meta` |
| `pool.ts`: restart budget and preload | At most 2 automatic restarts per worker per clip; all four worker scripts and both WASM bundles preloaded at app start so a deploy does not break open tabs (TS §33) |
| GPU-loss, quota and decode-error handling; `ErrorPanel.tsx` complete | C-14: retry from the last completed stage; a second failure with the same code shows the `.persistent` copy; "Report this" sends one `client_error {error_code, stage}` event and nothing else |
| `messages.ts`: every `E_*` code of TS §11.2 with `title`, `body`, `action` and a `.persistent` variant | `check-copy-codes.mjs` enforces coverage |
| Fixtures `bad_audio_payload.mp4`, `bad_video_payload.mp4` | Valid tables, corrupted payloads |
| The empty-output retry on WASM (TS §16.6): a transcription that comes back empty on WebGPU is tried once on WASM before `REJECT_NO_SPEECH` | Listed for V4 by V2 §25.2; moved here with failure recovery (V4 implementation plan §24 item 10) |
| `web/tests-e2e/restore.spec.ts`, `clear-local-data.spec.ts`, `failure-recovery.spec.ts`; `persistence/receipt-outbox.test.ts` | The V7 proofs |

### 9.2 Non-code work: E-2 starts (weeks 6-9)

- Give the alpha build to 15 technical creators and have each run their own clip (PS §19 E-2).
- Ask each one: would you post this result with at most 2 minutes of changes? Target: at least 8 of 15.
- Write down what each person changed, what broke, which plan items they switched off and which cuts they restored. These feed the E-10 edit count, the top-3-errors-per-week rule (PS §14) and E-11 (PS §19, v1.2).
- Tune the items the specs hand to E-2: safe-area insets, style values, event timings, watermark size and placement (TS §37 non-blocking decisions).

### 9.3 Exit checklist

- [ ] `restore.spec.ts`: reload mid-review returns to `ready` with edits intact and no re-transcription; reload mid-export discards the temp file; the `clips` store holds no file name.
- [ ] `clear-local-data.spec.ts`: OPFS and clip stores empty afterwards; refused while busy.
- [ ] `failure-recovery.spec.ts`: worker crash, injected GPU loss, storage quota and both decode errors each show their specific message; retry succeeds; a second failure shows the persistent copy.
- [ ] `receipt-outbox.test.ts`: entries survive reload and are removed only on a 2xx.
- [ ] The first five alpha users have run a clip.
- [ ] Tagged `v7`.

---

## 10. V8 - Prove the privacy claim; security hardening (M2.2; week 7: 23-29 Nov)

**Goal.** "Your video is not uploaded" stops being a statement and becomes a test that blocks every release. Every privacy promise in TS §25.1 (P-1 to P-14 from v1.2) has its structural guard, its automatic test and its place in the product.

**What you can demo.** Run `privacy-network.spec.ts`: it drives the whole journey while recording every request from the page and all workers, then proves no request went to an unlisted host, none contained the file name or any byte window of the media, and transcript words appeared only in the one plan request (none at all with the AI plan off; v1.2).

**Depends on.** V6.11 (all flows must exist to be captured). Can overlap with V7.

**v1.3.** The scenario adds the long path, chat and assets. New assertions: no frame, audio sample or asset byte in any request (INV-28); sentence text only in `/shorts`, chat text only in `/chat-edit`, labels only in AI bodies; with the AI switch off, none of them; P-15 traced like the rest.

### 10.1 Build

| Create or complete | Purpose |
|---|---|
| `web/tests-e2e/helpers/network-capture.ts` | Captures every request from the page, all workers and any service worker through the CDP Network domain, from first navigation |
| `web/tests-e2e/privacy-network.spec.ts` | The six assertions of TS §25.2 over: land, drop, first-run model download, processing, preview, edit a word, toggle an event, sign in, export, download, open settings |
| `web/src/analytics/events.types.test.ts`, `client.test.ts` | Type-level: no analytics property is free text; batching and flush behaviour |
| `web/src/copy/messages.test.ts` | No digit literals in templates except through placeholders; the banned-phrase list fails the build: "never leaves", "GDPR", "DPDP", "CCPA", "SOC 2", "compliant" |
| `web/src/ui/pages/SettingsPage.tsx` complete | PS §12.7: the table, mode "Local only", model version, cache size, "Clear local data", the "Verify it yourself" note |
| `web/tests-e2e/settings-what-leaves.spec.ts` | The rendered event list equals the generated allowlist exactly |
| `web/src/ui/pages/LegalPage.tsx` with `terms`, `privacy`, `refunds`, `faq` | Static content in `messages.ts` (PS §20.6); 14-day refund policy (PS §18) |
| `web/tests-e2e/capability.spec.ts`, `unsupported.spec.ts` | Check within 3 s with enums only; each of the 11 `UNSUPPORTED_*` reasons, injected, shows its specific message, the supported list, the demo video and the email form |
| `scripts/metrics.sql` | Activation, "would you post this", 14-day repeat, capability pass rate, reject rate, active paid (PS §20.5) |

### 10.2 Security sweep

- [ ] Production responses carry every header of TS §24.3; `crossOriginIsolated` is true.
- [ ] `check-hosts.mjs` finds no `http(s)://` host in the bundle outside `allowlist-hosts.ts`.
- [ ] `log_redaction.rs` re-run: no email, token, IP address or body in server logs (INV-19).
- [ ] `no_media_routes.rs` re-run: no route accepts more than 64 kB or a media content type (INV-4), except DC-14's `POST /transcribe` and (v1.4) DC-15's `POST /shorts`.
- [ ] `rate_limit.rs` re-run with the hop count from TE-5.
- [ ] Secret scan, `cargo deny check`, `pnpm audit --prod --audit-level high` clean in CI.
- [ ] Cookie attributes, CSRF header, refresh rotation and reuse revocation asserted in `auth_flow.rs`.
- [ ] The client bundle contains only public values: the asset base URL and the entitlement public key.

### 10.3 The legal-review gate under the 0 USD rule

PS §17 requires a lawyer's review before any public compliance claim or any absolute wording. A lawyer is not free. The way through is to never trigger the gate: make no compliance claim at all, and say only the factual sentence the network test backs, "your video is not uploaded to our servers". `messages.test.ts` enforces this mechanically. The Terms, Privacy Policy and refund policy are written to match the allowlist exactly and reviewed by you; they are not lawyer-reviewed, and that is a known gap to state honestly if anyone asks.

### 10.4 Exit checklist

- [ ] `privacy-network.spec.ts` passes and is wired to block every release.
- [ ] `settings-what-leaves.spec.ts`, `events.types.test.ts`, `messages.test.ts`, `client.test.ts`, `unsupported.spec.ts`, `capability.spec.ts` pass.
- [ ] Security sweep fully checked.
- [ ] Each of P-1 to P-15 (TS §25.1) traced to its guard, its test and its place in the UI.
- [ ] `scripts/metrics.sql` runs against the production database.
- [ ] Tagged `v8`.

---

## 11. V9 - Real clips and real speed (M2.3, M2.4; weeks 8-9: 30 Nov - 13 Dec)

**Goal.** The product works on clips nobody prepared for it, and it meets its time budgets on the reference machines.

**What you can demo.** The corpus report: 40 or more real clips, at least 90% exported and verified, the rest rejected with a specific code, none crashed. The benchmark report: every stage inside its budget on R1.

**Depends on.** V5 and V7. The corpus you started collecting in V3.

**v1.3: a long corpus and long budgets.** Add at least 15 real long videos (4K, HEVC, screen recordings, varied mics) to the corpus, and read TS §53.3's budgets on R1 and R2, with `bench/device-bench.ts --long`. R1 has not been read for any long-path number before this. **If R1 misses the 55-minute line:** the two-pass model (TS §43.6); then the founder decides between a 10-minute cap and stating a longer time.

### 11.1 Week 8: corpus hardening (M2.3; checkpoint Sun 6 Dec 2026)

| Do | Detail |
|---|---|
| Finish the corpus | 40 or more real clips tagged `vfr`, `rotated`, `noisy`, `accent`; `corpus/manifest.json` records id, SHA-256 and tags; clips stay outside git |
| Run `pnpm corpus` | Each clip goes through import, pipeline and export, then `verify_mp4.py` |
| Fix by frequency | Group failures by error code; fix the most common first; add a small fixture for each bug found so it cannot return |
| Re-check TE-12 | Demux coverage at least 90% (V3 read it on 2 clips: inconclusive) |
| Run E-10 | WER on the 20-clip accent set (10 US, 10 Indian or other English accents, with reference text): its first full run, carried from V3 (v3implementation D-88), and again after any model change |

**The week-8 rule (PS §20.4).** If the pass rate is under 90% on Sun 6 Dec, do not slip the launch. Remove the least valuable accepted input by tightening `limits.rs` (first candidate: input above 30 fps), update the rejection copy, and re-run.

### 11.2 Week 9: performance on R1 and R2 (M2.4)

Reference conditions: the 60-second reference clip, model cached, `bench/device-bench.ts`, median and p90 of 10 runs (TS §30).

| Metric | Budget on R1 |
|---|---|
| Probe + audio extraction | 3 s |
| Local transcription | 20 s |
| Detection + scene build | 2 s |
| Audio chain | 4 s |
| Render + encode | 90 s |
| Mux + finalize | 2 s |
| Median total | 121 s |
| p90 total | 180 s |
| First-run model download + initialization at 25 Mbps | at most 90 s |
| Capability check | 3 s |
| Preview drift over 60 s | at most 80 ms |
| Preview late frames | at most 5% |
| First preview frame after `ready` | at most 500 ms |
| Edit to updated preview | at most 150 ms |
| Peak tab memory | at most 1.5 GB |
| App shell (HTML + JS + CSS, no WASM) | at most 400 kB gzip |
| Longest main-thread task while processing | at most 100 ms |
| API p95, warm | at most 150 ms |
| API cold request | within 70 s |

Also measure a 90-second clip. It has no budget; it is expected at about 1.5x, and the number goes into the record so the "about three minutes for a 60-second clip" promise stays honest (PS §20.3).

| If a budget is missed | Action |
|---|---|
| ASR median above 145 s for a 60 s clip (D-67 of the V2 plan; was 40 s) | Smaller model; stronger caption editing |
| Render + encode above 150 s | Canvas2D overlay path; cap input at 60 s and 30 fps |
| Denoise exceeds the audio budget | High-pass, loudness and limiter only |
| App shell over 400 kB | Lazy-load `/account`, `/settings`, `/legal` |
| Long tasks over 100 ms | Move the offending work behind a worker method |
| PCM hand-off over 50 ms per hop | Switch `pcm48`/`out48` to `SharedArrayBuffer` views |
| Between a budget and its threshold | Ship; record the miss in TS §39; keep stage telemetry |

Add `web/tests-e2e/api-cold-start.spec.ts`: an API delayed 45 s shows the waking copy after 3 s and then succeeds; with the API down, an export with a cached token completes and its receipt waits in the outbox (INV-3). Re-check TE-11 (Render memory, cold start, monthly hours) and TE-14 (peak memory).

### 11.3 Non-code work: E-5 presale continues (moved to after V6.8, §8A.8; v1.5, v1.3: after V6.6)

The founding offer has been live since the founding release of §8A.8 (founder's decision of 2026-11-02), with its boxes ticked there. Keep offering it at 99 USD/year to waitlist and alpha people. Target: at least 5 paying strangers, not friends or family, before launch (PS §19 E-5). E-2 finishes at the end of week 9.

### 11.4 Exit checklist

- [ ] `corpus/run_corpus.ts`: at least 90% export and pass the verifier, the rest rejected with a code, zero crashes.
- [ ] `bench/device-bench.ts` results inside the budgets above committed for R1 and R2, or each miss handled by its action and recorded.
- [ ] `api-cold-start.spec.ts` passes.
- [ ] TE-11, TE-12, TE-14, E-3, E-4 final outcomes recorded.
- [ ] E-2 result written down: N of 15 would post.
- [ ] Tagged `v9`.

---

## 12. V10 - Launch gate (M2.5; week 10: 14-20 Dec)

**Goal.** Every box on three checklists is ticked, on the production build with production headers, and the product is opened to the public on Mon 21 Dec 2026.

**Depends on.** Everything above.

### 12.1 Product checklist (PS §20.1)

- [ ] The fixture 60-second English clip goes from drop to downloaded MP4 on R1 and R2 without errors.
- [ ] The capability check passes on Chrome and Edge on Windows and macOS (Edge is a manual run), and a specific unsupported page shows elsewhere.
- [ ] Every PS §9.4 constraint has a test that produces its rejection message, except one speaker and English, which are stated at the drop zone.
- [ ] Model download is resumable and hash-verified; a second session skips it.
- [ ] Detector precision at least 90% on at least 30 hand-labeled transcripts; no event below threshold exists.
- [ ] Caption edits re-run detection for the affected sentence.
- [ ] Voice chain output within ±1 LU of -14 LUFS.
- [ ] With no cuts, the export has the same duration as the source within one video frame; the 0-5 s speech, 5-10 s silence, 10-20 s speech fixture exports as 20 seconds, in sync on both sides of the pause.
- [ ] With cuts (v1.2), the same fixture under a fixed test plan exports at the time map's duration within one video frame, no cut inside a word, no audible click, in sync on both sides of every join (PS §20.1).
- [ ] Preview drift at most 80 ms over 60 s.
- [ ] Creator export is 1080x1920 H.264/AAC; Free export is 720x1280 with watermark; both play in the default gallery app on one Android and one iOS phone and upload to two target platforms (manual).
- [ ] Free account: 3 exports per month enforced by the token; counter visible before each export.
- [ ] Checkout works end to end in test mode and live mode; replayed webhooks change nothing.
- [ ] The network-privacy test passes: no media bytes; transcript text only in the one plan request, and none with the AI plan off; only allowlisted hosts and events (v1.2).
- [ ] The "What leaves your device" page matches the allowlist exactly.
- [ ] "Clear local data" removes clips, transcripts and the render cache.
- [ ] The 40+ clip corpus exports at 90% or better; the rest are rejected specifically; none crash.
- [ ] (v1.3; v1.4) Every long-path box of PS §20.1 is ticked: 45-minute fixture to 5 exported shorts on R1 and R2; no short starts or ends inside a sentence; a killed tab loses at most the windows in flight; a cloud failure pauses with Continue; the AI-down list works; B-roll by hand and from Pixabay exports.
- [ ] (v1.3) With every new feature off, frames and samples equal V5's (INV-26).
- [ ] Error codes and stage timings flow through the allowlist; the uptime monitor is on the API.

### 12.2 Technical checklist (TS §29)

- [ ] Every test file listed in TS §5 exists, runs in CI and passes.
- [ ] `check-file-tree.mjs`, `check-gen-clean.sh`, `check-copy-codes.mjs`, `check-hosts.mjs` pass.
- [ ] `cargo clippy -D warnings`, `cargo deny check`, ESLint and `tsc --noEmit` are clean with zero suppressions outside test code.
- [ ] Every `ErrorCode`, `RejectReason`, blocker and unsupported reason has copy and a passing test.
- [ ] Every (assumption) in TS §39 is measured and replaced, or still listed with its experiment.
- [ ] Every experiment in TS §37 has a recorded outcome or is explicitly still open.
- [ ] All 11 verifier checks pass on Free and Creator exports of both reference clips on R1 and R2.
- [ ] `privacy-network.spec.ts` passes on the production build served with the production headers.
- [ ] Production responses carry every TS §24.3 header; `crossOriginIsolated` is true.
- [ ] The hosting-plan terms are recorded in writing; no paid hosting plan is used. (v1.3) The AI provider's terms and `AI_MONTHLY_CAP_USD` are recorded.
- [ ] A cold API does not prevent import, processing, preview or review.
- [ ] No source file exceeds 700 lines, tests excluded.

### 12.3 Launch items (PS §20.6)

- [ ] Terms of Service, Privacy Policy and refund policy live since the founding release (§8A.8), and re-read against the allowlist (section 10.3 explains the review they did and did not get).
- [ ] Merchant-of-record account live since the founding release (§8A.8), tax settings re-confirmed.
- [ ] Support inbox, FAQ page, and the uptime monitor's public status page.
- [ ] Demo videos made with the product itself replace the hand-made E-1 demos; uploaded to the asset host.
- [ ] `scripts/metrics.sql` returns all PS §20.5 metrics.
- [ ] `scripts/check-external-facts.mjs` run: every free-tier fact from TE-5 to TE-11 re-verified in the launch week, because free tiers change without notice.

### 12.4 Read-outs before the switch

| Experiment | Target | If it missed |
|---|---|---|
| E-2 | At least 8 of 15 would post with at most 2 minutes of changes | Identify the gap (captions, events or audio) and fix that before launch |
| E-5 | At least 5 paying strangers | Investigate price against value before spending effort on marketing; launch can still proceed |
| E-7 | Instrumentation live: `events_kept` and `events_disabled` arriving in `export_done` | Fix before launch; the M3 decision needs it |
| E-13 | At least 60% of proposed shorts kept; parts deleted on at most 25% (PS §19) | Tune the prompt, try a stronger model, offer fewer shorts; the founder decides |
| E-14 | Instrumentation live: results split by first path used | Fix before launch |
| E-10 | WER at most 12%; median edits at most 8 per 60 s | Larger model, stronger edit UX or narrower claims |

### 12.5 Launch-day runbook

1. Freeze `main`. Run the full pipeline: `ci.yml`, `e2e-media.yml`, the corpus, the device benchmark.
2. Deploy the server, wait for `/api/v1/healthz`, then deploy the web build (TS §33 deploy order).
3. Run the header check and `privacy-network.spec.ts` against production.
4. Do one real purchase in live mode and one refund; confirm both in `subscriptions`.
5. Set `FOUNDING_OFFER_ENABLED=false`.
6. Confirm the uptime monitor is green and its status page is public.
7. Tag `v10`.
8. One coordinated launch, not three (PS §13.7): Show HN or Product Hunt, then social posts made with the product. PS §18 allows moving the public push to the first week of January if the holidays would bury it.

**If a checklist item cannot be ticked.** The date does not move by default. Cut the P0 input or behaviour with the lowest contribution to H1 (PS §18), update the copy and the specs, and re-run. An unticked privacy, payment or duration item is the exception: those block the launch.

---

## 13. After V10

- **Time split.** About 60% distribution and user conversations, 40% bug fixes and quality (PS §13.7). **v1.3: PS §13.9 holds the 3,000-paid goal, its arithmetic and the channels; outreach starts at V6.4, not here.** Fix the top 3 errors each week (PS §14).
- **No P1 work until the evidence asks for it.** Each P1 item has a measurable promotion trigger in PS §9.5 and PS §21. None starts on a hunch.
- **Month 1 check.** Activation under 40% of valid uploads: pause marketing and fix the funnel.
- **Decisions, not builds.**

| Date | Continue | Pivot | Kill |
|---|---|---|---|
| M3: Sun 21 Mar 2027 | Activation at least 60%, 14-day repeat at least 35%, at least 5 active paid | Value signals met but paid under 2; or paid at least 5 but repeat under 25% | All of: paid under 2, repeat under 20%, activation under 40% |
| M4: Sat 19 Jun 2027 | At least 16 active paid and repeat at least 35% | 10-15 active paid or repeat 25-35% | Under 10 active paid and repeat under 25% |

---

## Appendix A. Which version first makes each test pass

| Version | Rust tests | TypeScript unit tests | End-to-end suites |
|---|---|---|---|
| V1 | `offcut-types/tests/ui.rs`, `analytics_allowlist.rs` | `capability.test.ts` | `landing.spec.ts` |
| V2 | `mux_roundtrip.rs` | `download.test.ts` | `model-download.spec.ts`; `pipeline-preview.spec.ts` and `export-creator.spec.ts` (reference-clip cases) |
| V3 | `demux_fixtures.rs`, `probe_rejections.rs`, `loudness_target.rs`, `chain_no_clicks.rs`, `duration_preserved.rs`, `prosody.rs`, `numbers_table.rs`, `numbers_prop.rs`, `sentences.rs`, `rules_table.rs`, `precision_recall.rs`, `redetect.rs`, `determinism.rs` (detect) | none | `rejections.spec.ts` |
| V4 | `display_list_snapshots.rs`, `layout_safe_area.rs`, `framing.rs`, `determinism.rs` (scene), `golden_frames.rs` | `rpc.test.ts`, `clip-machine.test.ts`, `model-machine.test.ts`, `run-pipeline.test.ts` | `pipeline-preview.spec.ts` (complete), `preview-sync.spec.ts`, `review-edit.spec.ts`, `cancel.spec.ts` |
| V5 | none new | `start-export.test.ts`, `export-machine.test.ts`, `blockers.test.ts` | `export-creator.spec.ts` (all 11 checks), `timeline-preserved.spec.ts` |
| V6 | `auth_flow.rs`, `entitlement_rules.rs`, `usage_receipts.rs`, `account_delete.rs`, `account_export.rs`, `rate_limit.rs`, `log_redaction.rs`, `no_media_routes.rs`, `webhook_idempotency.rs`, `billing_events.rs`, `token_roundtrip.rs`, `profile.rs` | `auth-machine.test.ts`, `entitlement.test.ts`, `http.test.ts` | `signin.spec.ts`, `export-free.spec.ts`, `checkout.spec.ts` |
| V6.1 | `route_probe.rs`, `audio_region.rs`, `sentences_long.rs`, `transcribe_route.rs` (window budget, v1.4) | `source-machine.test.ts`, `transcribe-source.test.ts` (v1.4) | `route_long.spec.ts`, `high-res.spec.ts`, `long-resume.spec.ts`, `flags_identity.spec.ts` |
| V6.2 | `plan_route.rs`, `ai_cap.rs`, `ai_quota.rs`, `catalog_complete.rs` (kinds), `motion.rs`, `easing.rs`, `lockup.rs`, `enhance.rs` (v1.5) | `run-pipeline.test.ts` (fallbacks) | `privacy-network.spec.ts` (plan on and off, early form), `enhance-panel.spec.ts` (v1.5) |
| V6.3 | `timemap_build.rs`, `program_splice.rs`, `assembly.rs`, `assembly_readers.rs`, `rebase.rs`, `regions.rs`, `remux.rs` and `source_media.rs` (v1.4) | none | `cuts_applied.spec.ts`, `stitched.spec.ts` |
| V6.4 | `shorts.rs`, `shorts_prompt_snapshot.rs`, `shorts_rules.rs`, `shorts_route.rs` | `run-source-pipeline.test.ts` | `long-to-shorts.spec.ts` |
| V6.5 | `shots.rs`, `tracking.rs`, `vision_summary_prompt.rs`, `speaker.rs` (v1.5) | none | `vision-pass.spec.ts`, `speaker-framing.spec.ts` (v1.5) |
| V6.6a | `grade.rs`, `blur.rs`, goldens with a grade | none | none |
| V6.6b | Goldens per kind and preset; `composite.rs` simple-path identity | none | end-card duration |
| V6.7 | `mix_ducking.rs`, `sound_cues.rs`, `sfx.rs`, `voice_tune.rs` (v1.5) | none | `music-import.spec.ts` |
| V6.8 | `layers_goldens`, `handles.rs`, `plan_ops.rs` (v1.4), `broll_search_route.rs` (v1.4) | `resolve-stock.test.ts` (v1.4) | `join_transition.spec.ts`, `asset-box.spec.ts`, `stock-broll.spec.ts` (v1.4) |
| V6.9 | `plan_ops.rs`, `chat_prompt_snapshot.rs`, `chat_route.rs` | none | `chat-edit.spec.ts` |
| V6.10 | `creator_memory.rs`, brand-kit golden matrix | none | `queue.spec.ts` |
| V6.11 | `matte_goldens` | none | `prepare-matte.spec.ts` |
| V7 | none new | `receipt-outbox.test.ts` | `restore.spec.ts`, `clear-local-data.spec.ts`, `failure-recovery.spec.ts` |
| V8 | none new | `events.types.test.ts`, `messages.test.ts`, `client.test.ts` | `privacy-network.spec.ts`, `settings-what-leaves.spec.ts`, `capability.spec.ts`, `unsupported.spec.ts` |
| V9 | none new | none | `api-cold-start.spec.ts`; corpus run; device benchmark |
| V10 | all | all | all, on the production build |

## Appendix B. Experiment calendar

Weeks 1-10 are the calendar as first written; the v1.3 rows are by release.

| Week | Dates | Product experiments (PS) | Technical experiments (TS) |
|---|---|---|---|
| 1 | 12-18 Oct | E-1 starts; E-8 beacon live | TE-5, TE-6, TE-7, TE-11; TE-9 onboarding starts |
| 2-3 | 19 Oct - 1 Nov | E-3, E-4; **M0 gate reads E-1, E-3, E-4** | TE-1, TE-2, TE-3, TE-4, TE-10, TE-14 |
| 4 | 2-8 Nov | E-10 smoke reading only (E-10 carried to V9, v3implementation D-88) | TE-12 first run (inconclusive under 10 clips), TE-13 |
| 5 | 9-15 Nov | E-4 re-measured | TE-8, TE-9 completed |
| 6 | 16-22 Nov | E-2 starts | none |
| 7 | 23-29 Nov | E-2, E-8 continue | none |
| 8 | 30 Nov - 6 Dec | E-2; E-5 starts; E-10 (WER, first full run) | TE-12 re-checked (corpus checkpoint 6 Dec) |
| 9 | 7-13 Dec | E-2 ends; E-5; E-3, E-4 final | TE-11, TE-14 re-checked |
| 10 | 14-20 Dec | E-5 ends; E-7 instrumentation live | all free-tier facts re-verified |
| V6 | as it closes | E-12 (optional), any time before V6.4 | TE-8, TE-9 |
| V6.1 | | | TE-17 (reading in place), TE-18 (cloud windows), TE-20, TE-21; TE-12 re-read |
| V6.2 | | E-11 instrumentation | TE-15 |
| V6.3 | | | TE-22 |
| V6.4 | | E-13 labelled set; private alpha (long-path E-2) | TE-19 |
| V6.5 | | (v1.5) two-person labelled set | TE-23, TE-27 (v1.5) |
| V6.6a | | (v1.5) TE-29's panel | TE-29 |
| V6.6b | | | TE-28 (v1.5) |
| V6.7 | | | none (v1.5: TE-16 closed) |
| V6.8 | | E-16 (v1.5); first public demos; E-5 opens | TE-14 re-read, TE-26 (v1.4) |
| V6.11 | | | TE-24, TE-25 |
| After launch | from launch | E-6, E-7, E-8, E-9, E-13, E-14 read at M3 | none |

## Appendix C. Which version starts sending each analytics event

| Version | Events (the 19 of TS §22.6, plus v1.3's) |
|---|---|
| V1 | `landing_view`, `capability_check` |
| V2 | `clip_accepted`, `model_download`, `stage_timing`, `pipeline_done`, `preview_played`, `export_started`, `export_done`, `export_failed` |
| V3 | `clip_rejected` |
| V4 | `review_action`, `job_cancelled`, `client_error` |
| V5 | `post_export_answer` |
| V6 | `signin_step`, `upgrade_prompt`, `checkout_step` |
| V6.1 | `long_source_accepted`, `source_stage_timing`, `source_paused` (v1.4) |
| V6.2 | `plan_result` |
| V6.4 | `director_result`, `short_action` |
| V6.5 | `vision_result` |
| V6.9 | `chat_edit` |
| V6.10 | `queue_result` |
| V7 | `local_data_cleared` |

## Appendix D. Open risks that the 0 USD rule leaves in place

v1.3 allows a capped paid AI provider. Hosting, database, email and storage remain under the rule, and these risks remain. **A large free-tier API on Render will carry long-path traffic poorly at scale:** moving to paid hosting is the founder's decision once there is revenue.

These are not solved by this plan. Each is watched at the version shown.

| Risk | Why it stays open | Watched at | Only remedy without paying |
|---|---|---|---|
| Vercel's free plan is restricted to non-commercial use, and Offcut sells subscriptions | A paid plan is ruled out | V1 (TE-5), V9 before live checkout | Move the static build to another free host that allows commercial use and supports custom headers and a reverse-proxy rule |
| Magic-link email without a purchased domain may land in spam | A domain costs money | V6 (TE-8) | A provider that verifies a single sender address; if none delivers, sign-in is unreliable |
| No database backups on a free Postgres tier | Backups are a paid feature | V1 (TE-6) | Subscription state can be rebuilt from the merchant by replaying events; usage receipts and the waitlist would be lost |
| If local transcription stays too slow, the specs make cloud transcription launch-blocking | Server compute costs money | V2 (M0 gate) | None: treat a persistent E-3 failure as a stop |
| The policies are not lawyer-reviewed | A lawyer costs money | V8 | Make no compliance or absolute claim, enforced by `messages.test.ts` |
| Render's free service sleeps; free tiers can change terms without notice | Inherent to free tiers | V1, V9, V10 (TE-11, `check-external-facts.mjs`) | Keep-warm ping; move the same Docker image to another free container host |
| CI minutes for the Windows media suite | Free allowance is finite | V2 (TE-10) | Run media suites on `main` and before release only; `pnpm e2e:device` locally |
