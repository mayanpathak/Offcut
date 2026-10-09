# buildplan.md - Offcut Build Plan (V1 to V10)

**Status:** v1. Built from `product.md` (PS, Product Spec v1.1) and `technicalspec.md` (TS, Technical Spec v1.1).

**What this document is.** The order in which Offcut gets built, as ten versions. Each version is a tagged, deployed, demoable state of the product. V1 is a working skeleton. V2 to V6 add the product, and V6 is feature-complete. V7 to V10 add no features: they are testing, hardening and the launch gate.

**What this document is not.** It defines no feature, no number and no file that the two specs do not already define. Where this plan and a spec disagree, the spec wins and this plan gets fixed. "PS §n" is a product-spec section, "TS §n" a tech-spec section. Step ids such as `M1.3` and experiment ids such as `E-3` and `TE-5` are the specs' own (TS §28, TS §37).

---

## 0. The ten versions at a glance

| Version | Name | Spec steps | Calendar (build starts Mon 12 Oct 2026) | Gate at the end |
|---|---|---|---|---|
| **V1** | Working skeleton | M0.1 | Week 1: 12-18 Oct | Deployed shell + API + CI green |
| **V2** | Proof of concept: one clip in, one MP4 out | M0.2, M0.3, M0.4 | Weeks 2-3: 19 Oct - 1 Nov | **M0 gate** (Sun 1 Nov): stop or continue |
| **V3** | Real ingest, clean voice, real detector | M1.1, M1.2, M1.3 | Week 4, first half: 2-4 Nov | Four event kinds at 0.90 precision |
| **V4** | The look and the editor | M1.4, M1.5 | Week 4, second half: 5-8 Nov | Preview + review UI working |
| **V5** | Export that an independent tool accepts | M1.6 | Week 5, first half: 9-11 Nov | All 11 verifier checks pass |
| **V6** | Accounts, limits, payments: feature-complete | M1.7, M1.8 | Week 5, second half: 12-15 Nov | **M1 gate** (Sun 15 Nov): all P0 features work |
| **V7** | Survive real life: restore and failure recovery | M2.1 | Week 6: 16-22 Nov | Reload, crash and quota cases recover |
| **V8** | Prove the privacy claim; security hardening | M2.2 | Week 7: 23-29 Nov | Network-privacy test blocks releases |
| **V9** | Real clips and real speed | M2.3, M2.4 | Weeks 8-9: 30 Nov - 13 Dec | Corpus checkpoint (Sun 6 Dec); budgets met |
| **V10** | Launch gate | M2.5 | Week 10: 14-20 Dec | **M2 gate** (Sun 20 Dec); public launch Mon 21 Dec |

Build work ends at V10. M3 (Sun 21 Mar 2027) and M4 (Sat 19 Jun 2027) are decisions, not builds (PS §20.5).

**The tight spot.** V3 to V6 are four versions in two weeks. That is the specs' own schedule (five weeks of implementation, PS §9.9), not a choice of this plan. If it slips, the rule is PS §18: cut the P0 feature with the lowest contribution to H1 before moving a date, and keep the five stabilization weeks (V7-V10) intact. One relief valve exists: the V6 server work (M1.7) depends only on V1, so it can be started early whenever client work is blocked.

---

## 1. Rules that apply to every version

### 1.1 Money and hosting (fixed)

- Recurring cost is 0 USD/month with no paid fallback (PS A-3, PS A-10, TS TDR-17).
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
6. The commit is tagged `v1` ... `v10`.

### 1.3 How each file gets written (TS §34)

- Write a per-file spec from `docs/file-specs/TEMPLATE.md` before the file. Give the implementer only that spec, TS §2, §7, §10, §11, the owning deep-dive section, the signatures of allowed dependencies and the generated types. Not the whole repo.
- Signatures in the TS are contracts. A file implements them exactly.
- No source file over 400 lines, tests excluded (TS §29).
- A file that is not in the TS §5 tree fails `check-file-tree.mjs`. If a version needs a new file, the TS §5 tree is updated in the same change.

### 1.4 Lines that are never crossed

| Rule | Where it comes from |
|---|---|
| The recording's timeline is never changed: no silence, pause, breath, filler or mistake is removed, shortened or moved | INV-5, INV-10, PS §8 principle 7 |
| No media, transcript, caption text, event parameter or file name leaves the browser | INV-2, PS §15 |
| `fetch` exists in four files only; the browser contacts two hosts only | TS §7, TS §24.1 |
| The main thread never decodes, infers, filters, renders or encodes | INV-17 |
| The ASR session and the render session never coexist | INV-12 |
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

### 4.2 Experiments run in V2

| ID | Question | Pass | If it fails |
|---|---|---|---|
| TE-1 | Does the ASR runtime load only from OPFS, make zero third-party requests and return word timestamps in a worker? | Zero requests outside the two hosts; timestamps for the reference clip; WebGPU and WASM backends both run | Second runtime behind the same `whisper-runtime.ts` interface |
| TE-2 | Is a per-word confidence available? | Yes, at no more than 10% time cost | `Confidence(1.0)` for all words |
| E-3 | ASR time on R1 | Median at most 20 s; p90 total at most 180 s | Over 40 s: ship a smaller model (base-size first, then tiny-size), stronger caption editing |
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
| E-3 ASR on R1 | Median at most 20 s | 20-40 s: ship and record the miss. Over 40 s: smaller model | Still failing after the fallback makes cloud transcription launch-blocking (PS §19), which the 0 USD rule cannot fund: treat as a stop |
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

## 5. V3 - Real ingest, clean voice, real detector (M1.1, M1.2, M1.3; week 4, first half: 2-4 Nov)

**Goal.** Replace the three stand-ins from V2 with the real thing: any file a user drops is either accepted or rejected with a specific message; the voice is cleaned to the loudness target without changing its length; all four event kinds are detected by rules at 0.90 precision or better.

**What you can demo.** Drop a 4K file, a 2-minute file and an HEVC file: three different, specific rejections. Drop a good clip: the feed shows "Found: 3-item list", "Found: $2k to $20k"; the audio is audibly cleaner and exactly as long as before.

**Depends on.** V2 and a passed M0 gate.

### 5.1 Build

**A. Ingest and rejections (M1.1)**

| Create or complete | Purpose |
|---|---|
| `crates/offcut-mp4/src/validate.rs` | `validate_probe`: the 11 rules in order, first failure wins (TS §15.2) |
| `probe.rs` complete | Rotation, VFR flag, track counts, codec strings |
| `fixtures/gen_fixtures.sh`, `fixtures/manifest.json` | Generates every `ok_*` and `rej_*` fixture of TS §26 with the ffmpeg CLI; not committed |
| `messages.ts`: one entry per `REJECT_*` | 14 rejection messages with interpolated limits (TS §15.3) |
| `web/src/ui/components/RejectionPanel.tsx`; `state/machines/clip-machine.ts` | Rejection is a value, not an error (TS §11.3) |
| `web/src/persistence/db.ts`, `schema.ts`, `clips-repo.ts`, `transcript-repo.ts`, `edits-repo.ts` | Stage F of C-4 writes the clip, transcript and edits. The `clips` store has no file-name field |
| `crates/offcut-mp4/tests/demux_fixtures.rs`, `probe_rejections.rs`; `web/tests-e2e/rejections.spec.ts` | Every fixture gives its expected outcome |
| `corpus/manifest.json`, `fetch_corpus.sh`, `run_corpus.ts` | Corpus runner; clips stay outside git |

**B. Audio enhancement and prosody (M1.2)**

| Create | Purpose |
|---|---|
| `crates/offcut-dsp/src/` `config.rs`, `highpass.rs`, `denoise.rs`, `compressor.rs`, `limiter.rs`, `loudness.rs`, `chain.rs`, `prosody.rs` | `run_chain` exactly as the worked exemplar in TS §34.3; `measure_prosody` |
| `crates/offcut-wasm-core/src/audio_api.rs`; `web/src/workers/audio.worker.ts` | `runChain`, `measureProsody` |
| `crates/offcut-dsp/tests/` `loudness_target.rs`, `chain_no_clicks.rs`, `duration_preserved.rs`, `prosody.rs` | Postconditions P1-P8 of TS §34.3 |
| Fixtures `speech_20s_long_pause.mp4`, `speech_30s_sparse.mp4` (you record them) | The pause must survive: 20 s in, 20 s out, never 15 s |

**C. Normalization and the detector (M1.3)**

| Create or complete | Purpose |
|---|---|
| `crates/offcut-text/src/` `numbers.rs` (complete), `units_lex.rs`, `sentences.rs` (complete), `normalize.rs` (complete) | Spelled cardinals to trillions, decimals, k/m/b, currencies, percent, multipliers, units (TS §17.2) |
| `crates/offcut-detect/src/` `from_to.rs`, `list.rs`, `keyword.rs`, `exclusions.rs`, `resolve.rs`; `config.rs` (complete) | Scoring table and thresholds of TS §17.4; resolution order of TS §17.5 |
| `fixtures/labeled/*.json` (30 or more), `fixtures/labeled/SCHEMA.md` | Hand-labeled transcripts covering all four kinds |
| `crates/offcut-text/tests/` `numbers_table.rs`, `numbers_prop.rs`, `sentences.rs` | Number parse and format |
| `crates/offcut-detect/tests/` `rules_table.rs`, `precision_recall.rs`, `redetect.rs`, `determinism.rs` | Edge cases of TS §17.6; precision at least 0.90 |
| `run-pipeline.ts` complete | The C-4 order: audio extraction, then ASR and the voice chain in parallel, `asr.unload()` before `render.openSession()` |
| `ProcessingFeed.tsx` complete | One feed line per real detection, nothing invented |

### 5.2 Non-code work

- Hand-label at least 30 transcripts. This is slow, cannot be delegated to the detector being tested, and gates the version.
- Record the remaining speech fixtures: `speech_60s_landscape.mp4`, `speech_20s_noevents.mp4`.
- Collect the 20-clip ASR accuracy set (10 US, 10 Indian or other English accents) with reference text, and start collecting the 40+ real-clip corpus (VFR, rotated, noisy, accented). V9 needs the full corpus; asking people for clips takes calendar time, so start now.

### 5.3 Experiments

| ID | Pass | If it fails |
|---|---|---|
| TE-12 (first run) | `offcut-mp4` demuxes at least 90% of the corpus clips collected so far | JS demuxer behind the same `SampleMeta` shape |
| TE-13 | Voice chain at most 4 s for the reference clip on R1 | `ChainConfig.denoise = false`: high-pass, loudness and limiter only |
| E-10 | WER at most 12% on the 20-clip set (the caption-edit half is read in V7-V9 from real use) | Larger model, stronger edit UX or narrower claims |

### 5.4 Exit checklist

- [ ] `demux_fixtures.rs`, `probe_rejections.rs`, `rejections.spec.ts` pass.
- [ ] `loudness_target.rs`, `chain_no_clicks.rs`, `duration_preserved.rs`, `prosody.rs` pass.
- [ ] `numbers_table.rs`, `numbers_prop.rs`, `sentences.rs`, `rules_table.rs`, `redetect.rs`, `determinism.rs` (detect) pass.
- [ ] `precision_recall.rs` at 0.90 or above on at least 30 labeled transcripts. If below: raise the per-kind threshold to the lowest value in the printed sweep that reaches 0.90 (TS §17 contingency).
- [ ] TE-12, TE-13, E-10 outcomes recorded.
- [ ] Tagged `v3`.

**Not in V3.** Anything visual beyond what V2 drew. The three new event kinds are detected but only NumberReveal is drawn until V4.

---

## 6. V4 - The look and the editor (M1.4, M1.5; week 4, second half: 5-8 Nov)

**Goal.** The preview shows the finished look, and the user can fix it: three caption styles, four event visuals, 9:16 framing for landscape clips, a synced preview with scrubbing, and the review controls (edit a word, toggle an event, pick a style, drag the crop).

**What you can demo.** The AHA moment of PS §10 J7: drop a clip, watch your own words animate with a number counting up and a list building. Then fix a mis-heard word and see the number graphic update.

**Depends on.** V3.

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

**B. Preview player (M1.5)**

| Complete | Purpose |
|---|---|
| `render/preview-loop.ts`, `render/video-source.ts` | Audio clock is master; clock sync every 250 ms; queue of 6 frames; seek to keyframe; resize to 360x640 when mean frame cost exceeds 28 ms (TS §20) |
| `usecases/control-preview.ts`; `PreviewPlayer.tsx` | Attach, play, pause, seek; first play follows a user gesture |

**C. Review UI (M1.5)**

| Create | Purpose |
|---|---|
| `web/src/ui/components/ReviewPanel.tsx`, `CaptionEditor.tsx`, `EventToggleList.tsx`, `StylePicker.tsx`, `CropOffsetControl.tsx` | J8. No sliders except the crop offset. No control that removes or shortens anything |
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

- [ ] `display_list_snapshots.rs`, `layout_safe_area.rs`, `framing.rs`, `determinism.rs` (scene), `golden_frames.rs` pass.
- [ ] `pipeline-preview.spec.ts` (all cases), `preview-sync.spec.ts`, `review-edit.spec.ts`, `cancel.spec.ts` pass.
- [ ] `rpc.test.ts`, `clip-machine.test.ts`, `model-machine.test.ts`, `run-pipeline.test.ts` pass.
- [ ] Preview drift at most 80 ms over 60 s; first preview frame within 500 ms of `ready`.
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
| `offcut-mp4/src/mux.rs` complete | `moov` reserve of 256 KiB, `ctts` for reordered frames, `elst` for AAC priming, exact 30 fps timescale (TS §21.2, §21.3) |
| `offcut-entitlement/src/profile.rs` in use | `None` gives Preview; valid Creator gives 1080x1920 without watermark; anything else gives Free, 720x1280 with watermark |
| `web/src/state/blockers.ts`, `blockers.test.ts` | The eight blockers of TS §12.5, evaluated in one place |
| `state/machines/export-machine.ts`, `export-machine.test.ts` | `idle, gating, rendering, muxing, saving, done` |
| `usecases/start-export.ts` complete, `start-export.test.ts` | C-7 gate, C-9 render, C-10 deliver; download name `offcut-<yyyymmdd-hhmm>.mp4`, never the source name |
| `persistence/render-cache-repo.ts` | Cache key = SHA-256 over source hash prefix, transcript, events, edits, profile kind, renderer version |
| `ui/components/ChangeSummary.tsx`, `PostExportQuestion.tsx`; `usecases/answer-post-export.ts` | J11: only changes that were made; the "Would you post this?" question |
| `verify/verify_mp4.py` checks 7-11, `verify/watermark_mask_720.png` | Loudness, A/V sync and timeline against the source, watermark, bitrate, metadata |
| `web/tests-e2e/export-creator.spec.ts` complete, `timeline-preserved.spec.ts` | All 11 checks; INV-5 and INV-10 on both pause fixtures (600 and 900 frames) |

### 7.2 Things to know about V5

- Sign-in does not exist until V6. In V5, tokens come from `helpers/fake-api.ts` in tests and local development, so the deployed V5 can preview but cannot export for a real visitor. That is expected.
- E-4 is measured again here with the full renderer. The same thresholds and fallbacks as the M0 gate apply.

### 7.3 Exit checklist

- [ ] `export-creator.spec.ts` passes with all 11 verifier checks.
- [ ] `timeline-preserved.spec.ts` passes on `speech_20s_long_pause.mp4` and `speech_30s_sparse.mp4`.
- [ ] `start-export.test.ts`, `export-machine.test.ts`, `blockers.test.ts` pass.
- [ ] Live-frame counter is 0 after an export and after a cancelled export (INV-11).
- [ ] Render + encode on R1 re-measured and committed.
- [ ] Tagged `v5`.

**Not in V5.** The free-plan flow end to end (needs sign-in), checkout.

---

## 8. V6 - Accounts, limits, payments: feature-complete (M1.7, M1.8; week 5, second half: 12-15 Nov)

**Goal.** The last missing features: magic-link sign-in, the signed entitlement token, the free plan's three exports a month with watermark, usage receipts, account deletion and export, and checkout for the Creator plan through the merchant of record. After V6 nothing new is added to the product.

**What you can demo.** The whole journey J1 to J12 with a real email address: drop, preview, click Export, see the limits before signing in, sign in from the emailed link, export a 720p watermarked file, see "2 of 3 left", upgrade in the merchant's test mode, export 1080p without watermark.

**Depends on.** V5 for the client parts. The server parts depend only on V1 and can start earlier. TE-9 merchant approval, started in week 1.

### 8.1 Build

**A. Server: auth (M1.7)**

| Create | Purpose |
|---|---|
| `server/src/auth/mod.rs`, `magic_link.rs`, `session.rs`, `access_token.rs`, `extract.rs` | Magic link (32 random bytes, hash stored, 15 min, single use); refresh cookie `HttpOnly; Secure; SameSite=Strict`, rotated on every use; Ed25519 access token, 15 min, memory only; CSRF header on the two cookie routes (TS §22.2) |
| `server/src/mail/mod.rs`, `http_mailer.rs`, `templates.rs` | `Mailer` trait; the TE-8 provider's adapter; one template |
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

Replace the placeholder mail and billing variables in Render with real values. Keep `FOUNDING_OFFER_ENABLED=false` until V9. Prices (15 USD/month, 144 USD/year, 99 USD/year founding) are configured at the merchant and shown from `web/src/config/pricing.ts`; the server holds no amounts.

**D. Client**

| Create or complete | Purpose |
|---|---|
| `web/src/state/auth-store.ts`, `entitlement-store.ts`, `machines/auth-machine.ts` | Auth state machine (TS §12.2) |
| `web/src/entitlement/entitlement.ts` | `effective()` = token remaining minus outbox length, floored at 0; the only place the two are combined |
| `web/src/persistence/entitlement-repo.ts`, `receipt-outbox.ts` | Token cache; receipts survive reload and retry at 5 s, 30 s, 5 min, then each app start |
| `web/src/platform/broadcast.ts` | The sign-in tab tells the original tab over `BroadcastChannel` |
| `web/src/config/entitlement-public-key.ts`, `pricing.ts` | Public key as an array (rotation); display prices |
| `net/http.ts` complete | 10 s per attempt, backoff within the 70 s cold-start budget, "Connecting to the account service" after 3 s (C-15) |
| `web/src/usecases/` `request-magic-link.ts`, `complete-sign-in.ts`, `sign-out.ts`, `start-checkout.ts`, `confirm-checkout.ts`, `open-billing-portal.ts`, `delete-account.ts`, `export-account-data.ts`; `start-app.ts` complete (session restore) | C-8, C-11, C-18 |
| `web/src/ui/components/SignInDialog.tsx`, `UpgradePrompt.tsx`; `ui/pages/AuthCallbackPage.tsx`, `AccountPage.tsx` | Limits shown before the email field; the callback strips the token from the URL; the prompt has two placements only, `after_export` and `limit_reached` |
| `ExportButton.tsx` complete | Counter visible before the click; "last free export this month" confirm before the third (PS §11) |

### 8.2 Experiments

| ID | What you do | Pass | If it fails |
|---|---|---|---|
| TE-8 | Send 50 magic links to the major mailbox providers; read the quota terms | At least 95% in the inbox within 30 s; at least 100 emails/day; works from a single verified sender address with no purchased domain | Other candidate. There is no paid remedy: if neither delivers, sign-in is unreliable and that is a launch risk to state plainly |
| TE-9 | Test-mode purchase, renewal, cancel, refund; delay the webhook endpoint | Account approved; all six `BillingEventKind`s observed; a timed-out webhook is retried | Second provider behind `BillingProvider` |

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
- [ ] Tagged `v6`. **Feature freeze starts here.**

**Not in V6, and not ever in the MVP.** Passwords, OAuth, a Pro plan, coupons, credit metering, server-side enforcement of limits (TS §36).

---

## 9. V7 - Survive real life: restore and failure recovery (M2.1; week 6: 16-22 Nov)

**From here on, no features.** V7 to V10 make the V6 product reliable, provably private, fast enough and launchable. A change that adds a capability is out of scope until after the M2 gate (TS §28: no P1 item starts before M2 passes).

**Goal.** The product behaves well when things go wrong: the tab is reloaded, the browser crashes, the disk fills up, a worker dies, the GPU device is lost, a decoder chokes on a bad sample.

**What you can demo.** Reload the tab mid-review: you are back at the preview with your edits, with no second transcription. Kill a worker from the browser's task manager: a specific message, a retry that works.

**Depends on.** V6.

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
| `web/tests-e2e/restore.spec.ts`, `clear-local-data.spec.ts`, `failure-recovery.spec.ts`; `persistence/receipt-outbox.test.ts` | The V7 proofs |

### 9.2 Non-code work: E-2 starts (weeks 6-9)

- Give the alpha build to 15 technical creators and have each run their own clip (PS §19 E-2).
- Ask each one: would you post this result with at most 2 minutes of changes? Target: at least 8 of 15.
- Write down what each person changed, what broke, and any request for music. These feed the E-10 edit count, the top-3-errors-per-week rule (PS §14) and the music promotion trigger (PS §9.5).
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

**Goal.** "Your video is not uploaded" stops being a statement and becomes a test that blocks every release. Every one of the twelve privacy promises in TS §25.1 has its structural guard, its automatic test and its place in the product.

**What you can demo.** Run `privacy-network.spec.ts`: it drives the whole journey while recording every request from the page and all workers, then proves no request went to an unlisted host and none contained a transcript word, the file name or any byte window of the media.

**Depends on.** V6 (all flows must exist to be captured). Can overlap with V7.

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
- [ ] `no_media_routes.rs` re-run: no route accepts more than 64 kB or a media content type (INV-4).
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
- [ ] Each of P-1 to P-12 (TS §25.1) traced to its guard, its test and its place in the UI.
- [ ] `scripts/metrics.sql` runs against the production database.
- [ ] Tagged `v8`.

---

## 11. V9 - Real clips and real speed (M2.3, M2.4; weeks 8-9: 30 Nov - 13 Dec)

**Goal.** The product works on clips nobody prepared for it, and it meets its time budgets on the reference machines.

**What you can demo.** The corpus report: 40 or more real clips, at least 90% exported and verified, the rest rejected with a specific code, none crashed. The benchmark report: every stage inside its budget on R1.

**Depends on.** V5 and V7. The corpus you started collecting in V3.

### 11.1 Week 8: corpus hardening (M2.3; checkpoint Sun 6 Dec 2026)

| Do | Detail |
|---|---|
| Finish the corpus | 40 or more real clips tagged `vfr`, `rotated`, `noisy`, `accent`; `corpus/manifest.json` records id, SHA-256 and tags; clips stay outside git |
| Run `pnpm corpus` | Each clip goes through import, pipeline and export, then `verify_mp4.py` |
| Fix by frequency | Group failures by error code; fix the most common first; add a small fixture for each bug found so it cannot return |
| Re-check TE-12 | Demux coverage at least 90% |
| Re-run E-10 | WER on the accent set after any model change |

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
| ASR median above 40 s | Smaller model; stronger caption editing |
| Render + encode above 150 s | Canvas2D overlay path; cap input at 60 s and 30 fps |
| Denoise exceeds the audio budget | High-pass, loudness and limiter only |
| App shell over 400 kB | Lazy-load `/account`, `/settings`, `/legal` |
| Long tasks over 100 ms | Move the offending work behind a worker method |
| PCM hand-off over 50 ms per hop | Switch `pcm48`/`out48` to `SharedArrayBuffer` views |
| Between a budget and its threshold | Ship; record the miss in TS §39; keep stage telemetry |

Add `web/tests-e2e/api-cold-start.spec.ts`: an API delayed 45 s shows the waking copy after 3 s and then succeeds; with the API down, an export with a cached token completes and its receipt waits in the outbox (INV-3). Re-check TE-11 (Render memory, cold start, monthly hours) and TE-14 (peak memory).

### 11.3 Non-code work: E-5 presale starts (weeks 8-10)

Before any real money is taken:

- [ ] Vercel's plan terms are recorded in writing (TE-5), and you have accepted the non-commercial-terms risk in writing (TS §29). No paid plan is used.
- [ ] The merchant account is approved for live mode and tax settings are confirmed.
- [ ] Refund policy page is live (14 days).

Then set `FOUNDING_OFFER_ENABLED=true` and offer the founding-member plan at 99 USD/year to waitlist and alpha people. Target: at least 5 paying strangers, not friends or family, before launch (PS §19 E-5). E-2 finishes at the end of week 9.

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
- [ ] The export has the same duration as the source within one video frame; the 0-5 s speech, 5-10 s silence, 10-20 s speech fixture exports as 20 seconds, in sync on both sides of the pause.
- [ ] Preview drift at most 80 ms over 60 s.
- [ ] Creator export is 1080x1920 H.264/AAC; Free export is 720x1280 with watermark; both play in the default gallery app on one Android and one iOS phone and upload to two target platforms (manual).
- [ ] Free account: 3 exports per month enforced by the token; counter visible before each export.
- [ ] Checkout works end to end in test mode and live mode; replayed webhooks change nothing.
- [ ] The network-privacy test passes: no media bytes, no transcript or caption text, only allowlisted hosts and events.
- [ ] The "What leaves your device" page matches the allowlist exactly.
- [ ] "Clear local data" removes clips, transcripts and the render cache.
- [ ] The 40+ clip corpus exports at 90% or better; the rest are rejected specifically; none crash.
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
- [ ] The hosting-plan terms are recorded in writing; no paid plan is used.
- [ ] A cold API does not prevent import, processing, preview or review.
- [ ] No source file exceeds 400 lines, tests excluded.

### 12.3 Launch items (PS §20.6)

- [ ] Terms of Service, Privacy Policy and refund policy live (section 10.3 explains the review they did and did not get).
- [ ] Merchant-of-record account live, tax settings confirmed.
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

- **Time split.** About 60% distribution and user conversations, 40% bug fixes and quality (PS §13.7). Fix the top 3 errors each week (PS §14).
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
| V7 | none new | `receipt-outbox.test.ts` | `restore.spec.ts`, `clear-local-data.spec.ts`, `failure-recovery.spec.ts` |
| V8 | none new | `events.types.test.ts`, `messages.test.ts`, `client.test.ts` | `privacy-network.spec.ts`, `settings-what-leaves.spec.ts`, `capability.spec.ts`, `unsupported.spec.ts` |
| V9 | none new | none | `api-cold-start.spec.ts`; corpus run; device benchmark |
| V10 | all | all | all, on the production build |

## Appendix B. Experiment calendar

| Week | Dates | Product experiments (PS) | Technical experiments (TS) |
|---|---|---|---|
| 1 | 12-18 Oct | E-1 starts; E-8 beacon live | TE-5, TE-6, TE-7, TE-11; TE-9 onboarding starts |
| 2-3 | 19 Oct - 1 Nov | E-3, E-4; **M0 gate reads E-1, E-3, E-4** | TE-1, TE-2, TE-3, TE-4, TE-10, TE-14 |
| 4 | 2-8 Nov | E-10 (WER) | TE-12 first run, TE-13 |
| 5 | 9-15 Nov | E-4 re-measured | TE-8, TE-9 completed |
| 6 | 16-22 Nov | E-2 starts | none |
| 7 | 23-29 Nov | E-2, E-8 continue | none |
| 8 | 30 Nov - 6 Dec | E-2; E-5 starts | TE-12 re-checked (corpus checkpoint 6 Dec) |
| 9 | 7-13 Dec | E-2 ends; E-5; E-3, E-4 final | TE-11, TE-14 re-checked |
| 10 | 14-20 Dec | E-5 ends; E-7 instrumentation live | all free-tier facts re-verified |
| After launch | from 21 Dec | E-6, E-7, E-8, E-9 read at M3 | none |

## Appendix C. Which version starts sending each analytics event

| Version | Events (the 19 of TS §22.6) |
|---|---|
| V1 | `landing_view`, `capability_check` |
| V2 | `clip_accepted`, `model_download`, `stage_timing`, `pipeline_done`, `preview_played`, `export_started`, `export_done`, `export_failed` |
| V3 | `clip_rejected` |
| V4 | `review_action`, `job_cancelled`, `client_error` |
| V5 | `post_export_answer` |
| V6 | `signin_step`, `upgrade_prompt`, `checkout_step` |
| V7 | `local_data_cleared` |

## Appendix D. Open risks that the 0 USD rule leaves in place

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
