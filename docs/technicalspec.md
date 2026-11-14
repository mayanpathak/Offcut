# technicalspec.md - Offcut Technical Specification (v1.4)

**Status:** v1.5. Implements `product.md` "Offcut Product Spec v1.5". Changes since v1 are listed in Section 0. **v1.2 added §40 (Edit Plan Architecture). v1.3 adds §41-§53 (the compatibility contract, the version map, long videos, shorts, vision, layers, chat, brand and memory, person mask, AI cost). v1.4 revises them in place, each change marked "v1.4" (two entry pages, 45-minute sources read in place and transcribed in the cloud, part copies, stock B-roll, B-roll placed by hand, auto zoom). v1.5 (the founder's decisions of 2026-11-09, the Submagic benchmark) adds the finished-look work, each change marked "v1.5": motion by rules, caption polish and fonts, following the speaker, an automatic picture grade, synthesized sound effects, voice tuning, the Enhance switches and a quality check (§46.6, §47.1-§47.6, DC-16 to DC-19). Later sections win over any older section they contradict**; older sections describe the clip path with every flag off unless they were rewritten. **§41 is binding on every release: read it first.**

**Precedence.** The product spec (PS) wins on scope, priority, pricing and success numbers. This document only adds technical precision. It adds no feature, changes no priority and restates no product number differently. Labels P0/P1/P2, M0-M4, E-1 to E-10, H1-H4, A-1 to A-12 and reason codes (a)-(f) keep their PS meaning.

**Founder constraints (fixed inputs to this version).**
- Frontend: React + Vite, deployed on Vercel.
- Backend: deployed on Render.
- Recurring infrastructure cost: 0 USD/month, with no paid fallback (PS A-3, PS A-10), **except the AI provider, allowed with a monthly spend cap from v1.3 (§52).** Every component below runs on a free tier or has no fixed fee; when a free tier stops fitting, the component moves to another free tier, never to a paid one. Free-tier terms are external facts: each is verified by a named experiment (§37), never assumed.

**Timeline principles (v1.2; PS §8 principle 7, PS §9.2.1).** The user provides the take; the edit plan proposes cuts the user can see and restore.
- **`TIMELINE_INTEGRITY` (INV-5, rewritten in v1.2).** Every removed span of the recording is a validated `Cut` of the clip's stored plan that the user has not restored; no cut starts or ends inside a word; nothing inside a clip's recording is reordered or retimed. (v1.3: a short's parts are chosen and ordered by a validated `Assembly`, §44, which makes the recording that this rule then protects.) Two timelines exist: recording time (`TimeMs`) and program time (`OutMs`), related only by the clip's `TimeMap` (§40.1). With no cuts and no hold (always so in V4-V6, and whenever the AI plan is off), the map is the identity and source time equals output time, as in v1.1.
- **`AUDIO_DURATION_PRESERVATION` (INV-10).** The audio enhancement chain does not change the duration of the recording. Only `offcut-dsp::program::assemble` (§40.6) produces audio of another duration, and only as the `TimeMap` states.
- **Audio enhancement is not audio editing.** Enhancement keeps the timeline and changes signal quality (high-pass, denoise, compression, loudness normalization, limiting): the chain (§18). Editing changes the timeline: `assemble`, driven by the plan's cuts. They stay separate stages.

**Labels defined by this document.**
- `TE-n`: technical experiment (§37). Distinct from PS experiments `E-n`, which are reused unchanged.
- `M0.n`, `M1.n`, `M2.n`: build steps inside PS milestones M0, M1, M2 (§28).
- `TDR-n`: technical decision record (§38).
- `C-n`: call chain (§9). `INV-n`: invariant (§35).
- "PS §n" is a product-spec section. "§n" is a section of this document.

**Conventions.**
- **Rust:** `snake_case` functions and modules, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants, crates named `offcut-*`.
- **TypeScript:** `camelCase` functions, `PascalCase` types and React components, `kebab-case.ts` files, `PascalCase.tsx` component files.
- **SQL:** `snake_case` tables and columns. **Routes:** `kebab-case` under `/api/v1`. **Analytics events:** `snake_case`.
- **Codes:** errors `E_*`, input rejections `REJECT_*`, blockers `B_*`, unsupported reasons `UNSUPPORTED_*`.
- **Unit rule:** a bare number never crosses a function, message or storage boundary when it denotes time, size, money, a count limit or an id. It is wrapped in a unit type from `crates/offcut-types/src/units.rs` or `ids.rs` (§10). Enforced by `crates/offcut-types/tests/ui/` (trybuild compile-fail test `bare_ms_rejected.rs`) and by clippy `disallowed_types` in `clippy.toml`.
- **Signatures shown are contracts, not illustrations.** A file implements them exactly.
- **(assumption)** marks a value this document chose because the PS is silent. Each names the experiment or milestone that measures it and is listed in §39.

---

## 0. Changes From v1

**v1.5 (founder decisions of 2026-11-09, `docs/v6/v6changelog.md` Entry 38) makes the Submagic benchmark a set of rules, and adds what an engine on the computer can do that a server tool does not.**

- **Added.** Motion by rules on every talking-head shot and bezier easing curves (§47.2, flag `motion`); caption glow, punch words, emoji under the keyword and the hook lockup in V6.2, then four display fonts, six presets and word pages in V6.6b (§47.3, TE-28); following the speaker in two-person shots, with a split when unsure (§46.6, DC-17, TE-27, flag `speaker_framing`); the blur pass and an automatic per-shot grade with camera matching, face light and vignette (§47.4, DC-18, TE-29, flag `picture`); synthesized sound effects placed by rules and voice tuning by measured need (§47.1, DC-19, flag `voice_tune`); the Enhance panel, with a switch per enhancement and `Enhance::v5()` as the identity (§47.5, DC-16, INV-33); the quality check with rules fixes (§47.6); `ZoomIn` and `ZoomOut` join transitions (§48.3).
- **Changed.** V6.6 is built as V6.6a "Picture" and V6.6b "Graphics" (§42); `HookTitle` moves to V6.2; DC-8's widening starts in V6.5; the founding release moves to after V6.8; §46 no longer rejects all speaker detection.
- **Closed.** TE-16 (music is the person's own track; sound effects need no licence) and TE-21 (HDR stays rejected with advice).
- **Unchanged.** The clip path with flags off, the compatibility contract (§41), the release numbers, and the Tooscut rule (§53.4): ideas only, in our own code.

**v1.4 (founder decisions of 2026-11-08, `docs/v6/v6changelog.md` Entries 21 and 22) reshapes the long path before any of it is built.**

- **Changed.** Two entry pages: the person chooses "Make shorts from a long video" (`/app/long`) or "Edit a clip" (`/app`); duration no longer routes (§43.1, DC-1). Long sources up to **45 minutes**, with no file-size limit (§43.2). **The dropped file is read in place, never copied whole;** only the parts the Director picks are copied, as small MP4s (§43.3, §44.8). **A long video is transcribed in the cloud only,** in checkpointed windows of at most 90 s, and pauses with Continue instead of falling back to the device (§43.6). `/shorts` takes up to 160 kB (DC-15). Clip-page file size moves to the `high_res` flag (DC-3). INV-3, INV-4, INV-28, P-1, P-5, P-15, TE-17, TE-18, TE-20, §52's cost model, §53.3's budgets.
- **Added.** `PunchIn`'s `motion` (auto zoom, §47.2) and the quality bar against the founder's Submagic benchmark; Pixabay stock B-roll (§48.6, P-17, INV-32, TE-26, flag `stock_broll`, browser host 4); B-roll placed by hand on a word with a part picker, through `plan_ops` built in V6.8 (§48.5, §49.2); `remux_range` and `SourceMedia` (§44.8).
- **Removed.** The whole-source copy, `MAX_SOURCE_FILE_SIZE`, device transcription of a source and its two-pass seam.
- **Unchanged.** The clip path and the compatibility contract (§41). Tooscut stays a study source only (§53.4).

**v1.3 (founder decisions of 2026-11-02) adds the long path: a long video becomes 5-10 finished shorts.**

- **Added.** §41 the compatibility contract, the rule every release obeys; §42 the version map; §43 long sources (20 minutes, 16 GB, 4K, HEVC; v1.4: 45 minutes, read in place), windowed audio, resumable transcription; §44 the `Assembly`, a short as a virtual clip; §45 the Director (`/shorts`, whole-sentence selection, stitching); §46 local vision and face-tracked framing; §47 the launch catalog; §48 layers, B-roll, asset box, join transitions; §49 chat editing through typed `PlanOp`s; §50 creator memory and the back-catalogue queue; §51 person mask; §52 paid AI with a spend cap; §53 experiments TE-17 to TE-25, INV-25 to INV-31, long-path budgets, and what was learned from studying Tooscut.
- **Changed.** V6.x renumbered (§42). INV-2, INV-5, INV-12, P-1, P-5, P-11, §36, §23, §31, A-3. Every change to a protected behaviour is a numbered row of §41.2.
- **Moved to post-launch.** Chapter markers and the lower third.
- **Unchanged.** The clip path. A clip of at most 90 seconds with flags off is V5, byte for byte (INV-26).

**v1.2 (founder decision of 2026-10-21) makes Offcut an AI-planned editor; §40 is the design.**

- **Added.** `EditPlan` produced by the rule detector (V4) or by a cloud LLM through `POST /api/v1/plan` (V6.2); the capability catalog that generates the prompt and output schema (§40.3); the validator (§40.4); `OutMs` and `TimeMap` (two timelines, identity until V6.3); `offcut-dsp::program::assemble` for cuts, music and ducking (§40.6); `DrawCmd::Video` and `DrawCmd::Image`, `FontKey`/`FontSet`, `Theme`, `StyleSpec.motion`, a zone table (§40.5); the `plans` store; `plan_usage` table; TE-15, TE-16, E-11; INV-21 to INV-24; TDR-18 to TDR-21.
- **Changed.** INV-2, INV-5, INV-10, P-1, P-5, the §36 rows on cuts, time maps, music, brand kits and cloud LLM, TDR-4, TDR-7. V4-V6 build the seams of §40.9; V6.1-V6.11 build the features (§40.10, `buildplan.md` §8A).
- **Reversed from v1.1.** The ban on a source-to-output time map "kept for later" (§36, TDR-7): v1.2 builds it in V4 on purpose, as the identity, so that V6.3 changes data, not types.

**v1.1 removes automatic timeline editing from the architecture. It is not disabled; it does not exist.** (Superseded in part by v1.2 above.)

- **Removed.** Pause tightening and everything that existed only for it: silence profiling, `plan_pauses`, `apply_time_map`, `PauseConfig` and the pause/splice constants, `TimeMap` and `KeptSegment`, the separate source-time and output-time unit types, `EditState.pause_tightening`, `ChangeSummary.pauses_tightened`, `AudioWorkerApi.finishAudio` and `restore`, the `set-pause-tightening.ts` use-case, `offcut-scene/src/timeline.rs`, the separate `chain48` buffer, the pause feed line and analytics props, and their tests.
- **Simplified.** One timeline type (`TimeMs`). The voice chain's output is the final audio (`out48`). Preview and export read the source frame at the output frame's own timestamp.
- **Added.** INV-5 `TIMELINE_PRESERVATION`, INV-10 `AUDIO_DURATION_PRESERVATION`, `duration_preserved.rs`, `timeline-preserved.spec.ts`, two long-pause fixtures (§26).

---

## 1. Architecture Overview

**All media work (decode, transcription, DSP, detection, plan validation, cutting, mixing, rendering, encoding, muxing) runs in the user's browser, in four Web Workers around a Rust/WASM core; the server is one stateless Axum process that knows accounts, plan state, export and plan-request counts and allowlisted anonymous events, never receives media, captions, edits or file names, and receives the transcript text and timings only in the body of `POST /plan`, which it forwards to the AI provider and does not store (v1.2, §40.7).**

```text
USER'S BROWSER (desktop Chrome / Edge, cross-origin isolated)
├── Main thread (React + Vite app shell)
│   ├── ui/            pages and components; renders stores; no media work
│   ├── usecases/      one function per user action; orchestrates workers and API
│   ├── state/         Zustand stores + state machines + blockers
│   ├── persistence/   IndexedDB repos, OPFS paths, receipt outbox
│   ├── models/        model manager (download, resume, verify, cache)
│   ├── net/           the only fetch call sites (API + asset CDN)
│   ├── analytics/     allowlisted event client
│   └── AudioContext   preview audio playback (clock master for preview)
├── media.worker       import to OPFS, MP4/MOV demux + probe (Rust), validation,
│                      AAC decode to PCM (WebCodecs), resample 48 kHz + 16 kHz (Rust)
├── asr.worker         local speech model (ONNX Runtime Web: WebGPU or WASM-SIMD+threads),
│                      word timestamps, transcript normalization (Rust)
├── audio.worker       voice enhancement chain, loudness, prosody features (Rust)
├── render.worker      detector, scene build, Vello/WebGPU render, H.264 decode,
│                      preview loop, export loop (H.264 + AAC encode), MP4 mux (Rust)
├── WASM bundles       offcut_core.wasm   (media, asr, audio workers)
│                      offcut_render.wasm (render worker)
└── Storage            OPFS: source clips, model files, enhanced audio, exports
                       IndexedDB: clip refs, transcripts, events, edits, entitlement, outbox

 ═════════════ network boundary (closed list, §24.1) ═════════════

VERCEL (static hosting, app origin)
├── /                    app shell: index.html, JS, CSS, WASM, ORT runtime, fonts
├── /api/v1/*            rewrite (reverse proxy) to the Render service; same-origin for the browser
└── response headers     COOP, COEP, CSP (vercel.json)

ASSET CDN (object storage, free egress; TE-7)
└── GET /models/*, /media/*   speech model files, demo video, sample clip

RENDER (one free web service, Docker image `offcut-api`)
└── Axum process
    ├── /api/v1/healthz              liveness
    ├── /api/v1/auth/*               magic link, verify, refresh, logout
    ├── /api/v1/me, /entitlement     account and signed entitlement token
    ├── /api/v1/usage/receipts       idempotent export receipts
    ├── /api/v1/billing/*            checkout URL, portal URL, provider webhook
    ├── /api/v1/events               allowlisted analytics ingestion
    ├── /api/v1/notify-me            waitlist / unsupported-platform email capture
    ├── /api/v1/plan                 edit plan: transcript → AI provider → EditPlan (v1.2, V6.2; §40.7)
    └── /api/v1/account/*            delete, export

MANAGED POSTGRES (free tier, external to Render; TE-6)   users, sessions, subscriptions, receipts, events
EMAIL PROVIDER (SMTP relay, Brevo free tier; TE-8)      magic-link delivery, STARTTLS on port 2525 (V6, D-248, D-252)
AI PROVIDER (HTTPS API, free tier; TE-15; v1.2)          edit-plan completions, called by the server only
MERCHANT OF RECORD (Dodo Payments; TE-9)                 hosted checkout, customer portal, webhooks; test mode until the founding release (D-285)
UPTIME MONITOR (free tier)                               GET /api/v1/healthz every 5 min
```

**Responsibility split**
- **TypeScript, main thread:** UI, orchestration, state, persistence metadata, network. Never touches pixels, PCM or model tensors.
- **TypeScript, workers:** browser API glue only (WebCodecs, WebGPU canvas, OPFS sync handles, ONNX Runtime). No product rules.
- **Rust/WASM:** every product rule that affects output: container parsing, input validation, number parsing, detection, DSP, layout, animation, rendering, muxing, entitlement-to-profile mapping.
- **Rust, server:** auth, sessions, entitlement signing, usage counting, webhook processing, analytics ingestion, account deletion/export.
- **Vercel:** serves static files and proxies `/api/v1/*`. Runs no functions, no middleware.
- **Render:** runs the single API container. No disk, no background workers, no cron.
- **Postgres provider:** the only durable server-side state.
- **Asset CDN:** immutable, content-hashed public files. Receives no user data.
- **Merchant of record:** card data, tax, invoices, refunds, subscription lifecycle. Source of truth for subscription state (PS §16).
- **Email provider:** delivers one template (magic link). Brevo's SMTP relay, through `lettre` (V6, D-248).
- **The backend explicitly does not:** transcode, transcribe, render, store or proxy media; store or log transcripts, captions, edits, file names or paths (it forwards the transcript of a plan request to the AI provider and forgets it); validate plans against user edits (the client does, §40.4); hold prices; run scheduled jobs outside its own process; keep in-memory state that must survive a restart (Render restarts free services at any time).

---

## 2. System Boundaries

| Layer | Owns | May call | Must never call |
|---|---|---|---|
| `web/src/ui` | React components, pages, CSS modules | `usecases`, `state` (read + subscribe), `copy`, `config` | `workers`, `net`, `persistence`, `wasm`, `models`, `analytics` directly |
| `web/src/usecases` | One function per user action; sequencing; state transitions | `state`, `workers/pool`, `persistence`, `models`, `net/api-client`, `analytics/client`, `entitlement` | `ui`; `fetch`; WebCodecs/WebGPU APIs; WASM exports |
| `web/src/state` | Stores, state machines, blockers | `gen` types only | anything with side effects (`net`, `workers`, `persistence`) |
| `web/src/persistence` | IndexedDB schema and repos, OPFS path layout, receipt outbox | `idb`, OPFS async API, `gen` | `net`, `ui`, `workers` |
| `web/src/models` | Model manifest, download, resume, verify, cache state | `net/asset-fetch`, `persistence/opfs`, `wasm/load-core` (hashing) | `net/http`, `ui` |
| `web/src/net` | The only `fetch` call sites | `config/env`, `gen/api` | `state`, `ui`, `workers`; any host not in `config/allowlist-hosts.ts` |
| `web/src/analytics` | Event typing, batching, flush | `net/api-client` | reading transcript, events params, file metadata, or any store holding user text |
| `web/src/workers/*.worker.ts` | Browser API glue for one pipeline stage | own WASM bundle, WebCodecs, WebGPU, OPFS sync handles, `workers/protocol` | `fetch` (except loading own WASM/ORT files from the app origin via `wasm/load-*.ts`), `state`, `ui`, IndexedDB, other workers directly |
| `crates/offcut-types` | Unit types, ids, domain types, limits | `serde`, `ts-rs` | any other `offcut-*` crate; `web-sys`; I/O |
| `crates/offcut-{mp4,text,detect,dsp,scene}` | Pure algorithms | `offcut-types`; listed third-party crates (§3) | `web-sys`, `wasm-bindgen`, `wgpu`, network, clocks, randomness, global state |
| `crates/offcut-render` | wgpu device use, Vello backend, compositor | `offcut-types`, `offcut-scene`, `wgpu`, `vello` | detection, DSP, mux, network |
| `crates/offcut-wasm-{core,render}` | `wasm-bindgen` exports, JS-value conversion | the pure crates, `serde-wasm-bindgen`, `web-sys` | product rules (no branching on domain values beyond argument decoding) |
| `crates/offcut-entitlement` | Token format, verification; signing behind feature `sign` | `ed25519-dalek`, `offcut-types` | network, storage |
| `crates/offcut-api-types` | Every client-server DTO and the analytics allowlist | `serde`, `ts-rs`, `offcut-types` | server or client code |
| `server` (`offcut-api`) | Routes, DB access, signing, provider adapters | Postgres, SMTP relay, merchant API, AI provider API (v1.2) | any media handling; storing or logging a transcript; client crates other than `offcut-types`, `offcut-api-types`, `offcut-entitlement` and `offcut-plan` (v1.2: catalog, prompt, schema; §40.7) |
| `crates/offcut-plan` (v1.2, V4) | Plan types' logic: catalog, prompt and schema generation, validation, rules producer, time-map construction | `offcut-types`, `offcut-text` (tokenize, `parse_quantity`, `format_quantity`), `serde`, `thiserror`; `serde_json` behind the default feature `prompt` (v4implementation D-122) | `web-sys`, `wasm-bindgen`, `wgpu`, network, clocks, randomness, `offcut-scene`, `offcut-render` |
| Vercel config | Headers, rewrite | Render origin | functions, middleware, edge config |

Enforcement of every row: §7.

**Edges added in V2** (v2implementation D-27, D-58), each a named entry in `web/eslint.config.js`: `usecases/import-clip.ts` may import `net/asset-fetch` (the sample clip); `workers/**` may import `persistence/opfs` (the path builder); `workers/render.worker.ts` may import `config/entitlement-public-key`; `models/**` may import `state/model-store` and `config/model-manifest.json`; `models/model-manager.ts` may import `persistence/db` (`metaGet`, `metaSet`); `state/**`, `models/**` and `usecases/**` may import types only from `workers/protocol.ts` (`FeedLine`, `AppFailure`). Two use-case pairs may import each other: `import-clip.ts` to `run-pipeline.ts`, and `start-export.ts` to `control-preview.ts` (which also exports `detach()`, `lockForExport()` and `unlockAfterExport()`; `import-clip.ts` also exports `dismissClip()`).

---

## 3. Technology Stack

Version strategy for every row: exact versions pinned in `pnpm-lock.yaml` / `Cargo.lock`; upgrades are single-purpose PRs that pass the full CI pipeline (§33). "Latest stable at M0.1" means the version is chosen and pinned in step M0.1.

**Frontend**

| Technology | Purpose | Version strategy |
|---|---|---|
| React | UI | Latest stable major at M0.1, pinned |
| Vite | Dev server, build, worker and WASM bundling | Latest stable major at M0.1, pinned |
| TypeScript (`strict`, `noUncheckedIndexedAccess`) | All non-Rust client code | Pinned |
| Zustand | Stores (§12) | Pinned |
| React Router (library mode) | Client routes (§13.1) | Pinned |
| CSS Modules + CSS custom properties | Styling; no runtime CSS-in-JS (CSP `style-src 'self'`) | Built into Vite |
| `idb` | Typed IndexedDB access | Pinned |
| ONNX Runtime Web through `@huggingface/transformers` | ASR runtime in `asr.worker` (candidate; TE-1 decides) | Pinned; runtime `.wasm` files self-hosted under `/ort/` |
| Vitest | Unit and type-level tests | Pinned |
| Playwright (`channel: "chrome"`) | E2E, network capture | Pinned |
| ESLint + `eslint-plugin-boundaries` | Import-direction and API-ban rules (§7) | Pinned |
| pnpm | Workspace package manager | Pinned via `packageManager` |

**Rust (client core)**

| Technology | Purpose | Version strategy |
|---|---|---|
| Rust stable, target `wasm32-unknown-unknown`, `+simd128` | Core | `rust-toolchain.toml` |
| `wasm-bindgen`, `wasm-bindgen-cli`, `wasm-opt` | JS bindings, size optimization | CLI version equals crate version; checked by `scripts/build-wasm.sh` |
| `serde`, `serde-wasm-bindgen` | Boundary serialization | Pinned |
| `ts-rs` | Generates `web/src/gen/*.ts` from Rust types | Pinned |
| `wgpu` (WebGPU backend), `vello` | GPU device, vector renderer | Pinned as a matching pair; upgraded together only |
| `parley` | Text shaping, line breaking | Pinned with `vello` |
| `rubato` | Resampling | Pinned |
| `nnnoiseless` | Denoise (RNNoise in pure Rust) in `offcut-dsp/src/denoise.rs`; TE-13 kept it on (v3implementation D-89) | Pinned at 0.5.2 (V3 S1); built for `wasm32-unknown-unknown` without threads |
| `ebur128` (pure-Rust build) | BS.1770 integrated loudness with gating, in `offcut-dsp/src/loudness.rs` (v3implementation D-89) | Pinned at 0.1.10 (V3 S1); built for `wasm32-unknown-unknown` without threads |
| `ed25519-dalek`, `sha2` | Entitlement verification, model hashing | Pinned |
| `thiserror` | Error enums | Pinned |
| `proptest`, `insta`, `trybuild` | Property, snapshot, compile-fail tests | Pinned |

**Backend**

| Technology | Purpose | Version strategy |
|---|---|---|
| `axum`, `tokio`, `tower`, `tower-http` | HTTP service | Pinned |
| `sqlx` (Postgres, rustls, offline query cache) | DB access with compile-time-checked SQL | Pinned; `.sqlx/` committed |
| `reqwest` (rustls, `rustls-no-provider`, `ring` installed once in `main.rs`) | Merchant API calls (V6, D-281) | Pinned |
| `lettre` (`smtp-transport`, `tokio1-rustls`, `ring`; no default features) | Magic-link mail over SMTP with STARTTLS (V6, D-248, D-249) | Pinned |
| `ed25519-dalek` | Entitlement and access-token signing | Pinned |
| `tracing`, `tracing-subscriber` (JSON) | Logs to stdout | Pinned |
| `uuid` (v7), `time` | Ids, timestamps | Pinned |

**Infrastructure (all 0 USD/month by design; each verified)**

| Technology | Purpose | Verified by |
|---|---|---|
| Vercel (static + rewrite) | App origin | TE-5 |
| Render free web service (prebuilt Docker image from GHCR) | API | TE-11 |
| Managed Postgres, free tier. Candidates: Neon, Supabase | Server state | TE-6 decides |
| Object storage with free egress. Candidates: Cloudflare R2, Hugging Face Hub | Model files, demo video, sample clip | TE-7 decides |
| Transactional email, free tier: **Brevo's SMTP relay**, port 2525, as Render's free services block 25, 465 and 587 (V6, D-252) | Magic links; 300 a day | TE-8 (cut down by the founder, `docs/v6/experiments.md`) |
| Merchant of record: **Dodo Payments** (PS A-4; founder, 2026-11-02) | Checkout, tax, subscriptions; per-transaction fee only (PS A-9), no fixed fee | TE-9 (test mode read in V6; live approval belongs to the founding release, BP §8A.8) |
| Free uptime monitor (candidate: UptimeRobot) | PS §20.1 uptime monitor; keeps the API warm | TE-11 |
| GitHub Actions + GHCR | CI, image registry | TE-10 |

**Required platform capabilities** (PS §9.3). Checked once at startup by `web/src/platform/capability.ts::runCapabilityCheck` (§13.2); total budget 3 s (PS §9.3).

| Capability | Used for | Startup check | Failure code |
|---|---|---|---|
| WebCodecs `VideoDecoder` | Source decode | `VideoDecoder.isConfigSupported` for H.264 High, 1920x1080 | `UNSUPPORTED_H264_DECODE` |
| WebCodecs `VideoEncoder` | Export | `VideoEncoder.isConfigSupported` for the ladder in §21.2 at 1080x1920, 30 fps | `UNSUPPORTED_H264_ENCODE` |
| WebCodecs `AudioDecoder` / `AudioEncoder` | AAC decode, AAC encode | `isConfigSupported` for `mp4a.40.2`, 48 kHz | `UNSUPPORTED_AAC_DECODE`, `UNSUPPORTED_AAC_ENCODE` |
| WebGPU | Vello renderer, ASR acceleration | `navigator.gpu.requestAdapter()` returns non-null | `UNSUPPORTED_WEBGPU` |
| WebAssembly SIMD | Rust core, ASR WASM path | `WebAssembly.validate` of a SIMD probe module | `UNSUPPORTED_WASM_SIMD` |
| WASM threads | ASR WASM path | `crossOriginIsolated === true` and `SharedArrayBuffer` defined | `UNSUPPORTED_THREADS` |
| OPFS and IndexedDB | Clips, models, state | `navigator.storage.getDirectory()` resolves and `indexedDB.open` succeeds | `UNSUPPORTED_STORAGE` |
| Device memory | Memory budget (§31) | `navigator.deviceMemory >= 4` where reported; absent value passes | `UNSUPPORTED_LOW_MEMORY` |
| Desktop form factor | PS §9.2 excludes phones/tablets | `navigator.userAgentData?.mobile !== true` (assumption, §39: structured client hint, not UA-string parsing) | `UNSUPPORTED_MOBILE` |
| WebCodecs presence | All of the above | `"VideoEncoder" in globalThis` | `UNSUPPORTED_WEBCODECS` |

**Why (one line each)**
- **Postgres, not SQLite:** Render free services have an ephemeral filesystem, so a file database cannot persist; Render's own free Postgres expires after 30 days (verified 2026-10, re-verified in TE-6), so the database is hosted by a separate free provider.
- **Prebuilt Docker image:** avoids compiling Rust inside Render's free build pipeline (TE-11).
- **Same-origin `/api` rewrite:** `*.vercel.app` and `*.onrender.com` are different sites; a cross-site session cookie is unreliable. The rewrite makes the API same-origin, so the cookie is first-party and CSP `connect-src` stays closed.
- **Two WASM bundles:** workers that do not render must not download or instantiate Vello/wgpu.
- **Rust demux and mux in one crate:** the sample tables, timescales and edit lists are one body of knowledge; a JS demuxer is the stated fallback (TE-12).
- **Ed25519 tokens:** the client verifies with a public key; no shared secret ships in the bundle.

---

## 4. Repository Structure

```text
offcut/
├── crates/            Rust workspace members shared by client and server
├── server/            offcut-api: the Axum service, migrations, Dockerfile
├── web/               React + Vite app, workers, E2E tests, vercel.json
├── verify/            independent MP4 verifier (Python + ffprobe); shares no code with crates/
├── fixtures/          fixture generator and manifest, labeled transcripts, golden frames; generated and derived fixtures in fixtures/.gen/, not committed (v3implementation D-76, D-77)
├── corpus/            manifest + runner for the 40+ real-clip corpus (clips stored outside git)
├── bench/             device benchmark runner and committed results for R1/R2
├── scripts/           build, codegen and CI check scripts
├── docs/              product.md, technicalspec.md, buildplan.md, per-file specs
└── .github/workflows/ CI, API image deploy, media E2E
```

---

## 5. Complete File Tree

`[ONLY]` marks a "the only place" file. Test files sit beside the code they test (`*.test.ts`) or in the crate's `tests/` directory.

```text
offcut/
├── Cargo.toml                         workspace manifest; shared dependency versions
├── Cargo.lock
├── rust-toolchain.toml
├── clippy.toml                        disallowed_types / disallowed_methods (§7)
├── deny.toml                          cargo-deny: license allowlist, banned crates
├── package.json                       root scripts (§33)
├── pnpm-workspace.yaml
├── pnpm-lock.yaml
├── render.yaml                        Render blueprint: one free web service from GHCR image
├── .gitignore
├── .dockerignore                      allow-list for the image build context: Cargo files, crates/, server/
├── .gitleaksignore                    findings of the secret scan that are not secrets, by fingerprint (V4 Prompt 101)
├── .github/workflows/
│   ├── ci.yml                         lint, unit, golden, server tests, non-media E2E
│   ├── e2e-media.yml                  media E2E on windows-latest with Chrome stable (TE-10)
│   └── deploy-api.yml                 build image, push GHCR, trigger Render deploy hook
├── crates/
│   ├── offcut-types/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── units.rs               [ONLY] unit newtypes and conversions
│   │   │   ├── timemap.rs             [ONLY] recording ↔ program time; identity only in V4 (v4implementation D-125)
│   │   │   ├── assembly.rs            [ONLY] (V6.3) Part, Assembly: a short's parts of its source (§44)
│   │   │   ├── source.rs              (V6.1, V6.4) SourceInfo, Route, ShortDraft, ShortsPlan, ShortSpec
│   │   │   ├── frame_path.rs          [ONLY] (V6.5) FramePath
│   │   │   ├── vision.rs              (V6.5) Shot, ShotKind, FacePos, ShotSummary
│   │   │   ├── assets.rs              (V6.2) IconId, EmojiId, the bundle's closed names (v6.2 D-447); (V6.8) AssetId, AssetMeta, AssetRef
│   │   │   ├── plan_ops.rs            [ONLY] (V6.9) PlanOp, ChatReply
│   │   │   ├── creator.rs             (V6.10) CreatorMemory, CreatorMemorySummary
│   │   │   ├── ids.rs                 [ONLY] id newtypes
│   │   │   ├── limits.rs              [ONLY] every PS limit as a typed constant
│   │   │   ├── media.rs               ProbeInfo, ClipInfo, RejectReason, Orientation, Rotation
│   │   │   ├── transcript.rs          Word, Sentence, Transcript, NormalizedSpan, Quantity, Unit
│   │   │   ├── events.rs              EventKind, EventParams, DetectedEvent
│   │   │   ├── plan.rs                EditPlan, the plan's wire format, PLAN_FORMAT = 1 (v4implementation D-119, D-120)
│   │   │   ├── prosody.rs             WordProsody, Prosody
│   │   │   ├── edit.rs                EditState, StyleId, CropOffset
│   │   │   ├── enhance.rs             (V6.2) Enhance, VoiceMode (§47.5, v6.2 D-485)
│   │   │   ├── params.rs              (V6.2) Treatment, EmphasisSize, ZoomScale, ZoomMotion, EmojiPlace: the new kinds' named params (v6.2 §6.1)
│   │   │   ├── profile.rs             Plan, ExportProfile
│   │   │   ├── summary.rs             ChangeSummary
│   │   │   ├── stage.rs               PipelineStage
│   │   │   └── error.rs               ErrorCode, UnsupportedReason, FailureStage (shared code lists)
│   │   └── tests/
│   │       ├── ui.rs                  trybuild driver
│   │       └── ui/bare_ms_rejected.rs
│   ├── offcut-mp4/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── reader.rs              RandomAccess trait (read_at)
│   │   │   ├── boxes.rs               box header parsing and typed boxes
│   │   │   ├── sample_table.rs        stts/ctts/stsc/stsz/stco/co64/stss/elst resolution
│   │   │   ├── edit_list.rs           edit-list shapes E0-E4 (v3implementation D-92)
│   │   │   ├── demux.rs               Demuxer
│   │   │   ├── probe.rs               probe() -> ProbeInfo
│   │   │   ├── sps.rs                 full_range(): the brightness range an H.264 stream states (v2implementation D-70)
│   │   │   ├── hevc.rs                (V6.1) hvcC and the HEVC codec string (v6.1implementation D-396)
│   │   │   ├── colr.rs                (V6.1) the colour box; is_hdr from nclx or nclc (D-397)
│   │   │   ├── route.rs               (V6.1) route_probe(): which path a file takes (§43.1, D-395); re-exported by validate.rs
│   │   │   ├── validate.rs            [ONLY] validate_probe(): PS §9.4 limits -> RejectReason
│   │   │   ├── mux.rs                 Mp4Muxer (faststart)
│   │   │   └── mux_boxes.rs           box writers for the muxer (v2implementation D-55)
│   │   └── tests/
│   │       ├── demux_fixtures.rs
│   │       ├── probe_rejections.rs
│   │       ├── mux_roundtrip.rs
│   │       ├── route_probe.rs         (V6.1) every rule of route_probe from both pages; flags off equals validate_probe
│   │       └── mux_reorder.rs         real B-frames through the muxer, read by ffprobe and ffmpeg (v5implementation D-212)
│   ├── offcut-text/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── tokenize.rs
│   │   │   ├── cardinals.rs           spelled cardinals and "point" decimals (v3implementation D-92)
│   │   │   ├── sentences.rs           segment_sentences()
│   │   │   ├── numbers.rs             [ONLY] spoken/written number parsing and display formatting
│   │   │   ├── units_lex.rs           unit and currency lexicon
│   │   │   ├── rebase.rs              [ONLY] (V6.3) a short's transcript from the master's (§44.4)
│   │   │   └── normalize.rs           normalize_transcript()
│   │   └── tests/
│   │       ├── numbers_table.rs
│   │       ├── numbers_prop.rs
│   │       ├── sentences.rs
│   │       └── sentences_long.rs      (V6.1) 45 minutes of words: none lost, none over 40 words, none across a loop's pause
│   ├── offcut-detect/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                 detect(), redetect_sentence()
│   │   │   ├── config.rs              [ONLY] DetectorConfig thresholds and lexicons
│   │   │   ├── from_to.rs
│   │   │   ├── list.rs
│   │   │   ├── number.rs
│   │   │   ├── keyword.rs
│   │   │   ├── exclusions.rs          years, versions, times, phone-like sequences
│   │   │   ├── resolve.rs             overlap resolution and threshold filter
│   │   │   └── event_id.rs            stable EventId derivation
│   │   └── tests/
│   │       ├── rules_table.rs
│   │       ├── precision_recall.rs    reads fixtures/labeled/*.json
│   │       ├── redetect.rs
│   │       └── determinism.rs
│   ├── offcut-dsp/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── config.rs              [ONLY] DSP constants (§18.3)
│   │   │   ├── resample.rs
│   │   │   ├── highpass.rs
│   │   │   ├── denoise.rs
│   │   │   ├── compressor.rs
│   │   │   ├── limiter.rs
│   │   │   ├── loudness.rs
│   │   │   ├── chain.rs               run_chain() (length-preserving)
│   │   │   ├── level.rs               (V6.2) level_only(): loudness and the limiter only, lvl48 (v6.2implementation D-487)
│   │   │   ├── prosody.rs             measure_prosody()
│   │   │   ├── program.rs             [ONLY place that changes duration] assemble(); identity in V4 (v4implementation D-127)
│   │   │   ├── regions.rs             [ONLY] (V6.3) concat_parts: a short's audio from its parts (§44.5)
│   │   │   └── silence.rs             silence_profile() (first reader V6.3)
│   │   └── tests/
│   │       ├── loudness_target.rs
│   │       ├── chain_no_clicks.rs
│   │       ├── duration_preserved.rs
│   │       ├── stream_resample.rs     (V6.1) StreamResampler in pieces equals resample_mono, sample for sample (v6.1implementation D-401)
│   │       ├── prosody.rs
│   │       ├── level_only.rs          (V6.2) length, loudness at the target, the ceiling, one gain only; run_chain unchanged (D-487)
│   │       └── program_identity.rs    the program audio equals the cleaned voice (S-6); (V6.2) or the levelled one
│   ├── offcut-vision/                 (V6.5) pure: shot boundaries, kinds, face tracking; depends only on offcut-types (§46)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── shots.rs               histogram boundaries; ShotKind rules
│   │   │   ├── tracking.rs            [ONLY constructor of a non-empty FramePath] dead zone, smoothing, resets
│   │   │   └── summary.rs             ShotSummary → ShotLine, within the request's budget
│   │   └── tests/
│   │       ├── shots.rs
│   │       └── tracking.rs
│   ├── offcut-plan/                   (V4) the edit plan: catalog, prompt, validation, rules producer (v4implementation D-122)
│   │   ├── Cargo.toml                 default feature "prompt" (serde_json, the schema generator); the binding turns it off
│   │   ├── src/
│   │   │   ├── lib.rs                 module list; re-exports; PlanError
│   │   │   ├── catalog.rs             [ONLY] CATALOG_VERSION, ItemSpec, ParamSpec, the four kinds (D-123)
│   │   │   ├── prompt.rs              system_prompt, output_schema, user_message (feature "prompt"; first caller V6.2)
│   │   │   ├── config.rs              [ONLY] PlanConfig and its defaults (D-165)
│   │   │   ├── resolve.rs             QuantityRef -> Quantity over the effective tokens (D-121)
│   │   │   ├── validate.rs            [ONLY constructor of ValidPlan] the rules V-1 to V-8 (D-121, D-122)
│   │   │   ├── validate_params.rs     validate.rs's per-kind parameter checks, if it nears 400 lines (v4implementation C.1)
│   │   │   ├── shorts.rs              [ONLY constructor of ShortSpec] (V6.4) rules S-1 to S-10 (§45.5)
│   │   │   ├── shorts_prompt.rs       (V6.4) the Director's prompt and schema (feature "prompt")
│   │   │   ├── shorts_rules.rs        (V6.4) shorts_from_rules: the offline list (§45.6)
│   │   │   ├── plan_ops.rs            [ONLY] (V6.9) effective_plan, validate_ops (§49.2)
│   │   │   ├── chat_prompt.rs         (V6.9) the chat prompt and schema (feature "prompt")
│   │   │   └── from_rules.rs          DetectedEvent[] -> EditPlan, the rules producer
│   │   └── tests/
│   │       ├── catalog_snapshot.rs    the prompt and the schema, version-named snapshots
│   │       ├── validate.rs            one case per rule
│   │       ├── validate_v62.rs        (V6.2) the five kinds, the extras, V-3, V-9, D-452, D-453, DC-7 (v6.2implementation §13.1)
│   │       ├── plan_from_rules.rs
│   │       ├── shorts.rs              one case per rule S-1 to S-10
│   │       ├── shorts_prompt_snapshot.rs
│   │       ├── shorts_rules.rs
│   │       ├── plan_ops.rs            each op: apply, refuse, undo
│   │       ├── chat_prompt_snapshot.rs
│   │       ├── validate_rules_plans.rs  no rules plan of fixtures/labeled/ loses an item (D-165)
│   │       ├── ui.rs                  trybuild driver
│   │       ├── ui/valid_plan_private.rs  ValidPlan has no public constructor (INV-21); its .stderr beside it
│   │       └── snapshots/             insta snapshot files
│   ├── offcut-scene/
│   │   ├── Cargo.toml
│   │   ├── assets/
│   │   │   ├── fonts/Inter-Variable.ttf
│   │   │   ├── fonts/JetBrainsMono-Variable.ttf
│   │   │   ├── fonts/NotoEmoji-Variable.ttf
│   │   │   ├── fonts/LICENSES.md
│   │   │   └── images/                (V6.2) the 72 images, 160 px RGBA PNGs: emoji/ (48, Fluent Emoji 3D, MIT), icons/ (24, Lucide, ISC), manifest.json (id, key, file, SHA-256, pinned source, licence), LICENSES/; made by scripts/gen-image-bundle.mjs (v6.2implementation D-470)
│   │   ├── src/
│   │   │   ├── lib.rs                 build_scene(), Scene::frame_at()
│   │   │   ├── styles.rs              [ONLY] StyleSpec for Clean, Bold, Tech; (V6.2) Glow and StyleSpec.glow (v6.2implementation D-479, D-480)
│   │   │   ├── theme.rs               [ONLY colour values] Theme, Role, FontRole, default_for (v4implementation D-136)
│   │   │   ├── motion.rs              CaptionMotion, MotionUnit, MotionPreset, ActiveTreatment (D-137)
│   │   │   ├── camera.rs              (V6.2) CameraTrack: the slow drift by rules, build(), none(), scale_at() (v6.2implementation D-476)
│   │   │   ├── safe_area.rs           [ONLY] logical canvas and safe-area constants
│   │   │   ├── fonts.rs               font registry (embedded bytes)
│   │   │   ├── images.rs              (V6.2) ImageSet::embedded(): the bundle's PNGs by ImageKey; ImageKey::of_emoji, of_icon (v6.2implementation D-471)
│   │   │   ├── captions.rs            caption chunking and line layout; (V6.2) punch words: chunk_punched, PUNCH_SCALE (v6.2implementation D-482)
│   │   │   ├── readability.rs         [ONLY] break costs, CLAUSE_STARTERS, BINDERS, MIN_CHUNK_MS (D-167, D-168)
│   │   │   ├── placement.rs           [ONLY] caption block and event zone from CaptionPlacement (D-171)
│   │   │   ├── captions_draw.rs       drawing a chunk: motion, the Tech panel and cursor, pops; (V6.2) Glow's glow strokes (D-479); the plan's emphasis: colour, underline, highlight (D-481)
│   │   │   ├── hit.rs                 Scene::hit (D-174)
│   │   │   ├── layout.rs              text shaping through parley; (V6.2) cap_height (v6.2implementation D-483)
│   │   │   ├── lockup.rs              (V6.2) the hook's lockup: 2 or 3 lines of their own sizes filling a width, the accent at 2x; pure (v6.2implementation D-484, §10.4)
│   │   │   ├── easing.rs              Easing, ease(); (V6.2) CubicBezier, bezier(), STANDARD, OUT, IN_OUT (D-475)
│   │   │   ├── anim.rs                keyframe tracks evaluated at TimeMs
│   │   │   ├── events/mod.rs
│   │   │   ├── events/number_reveal.rs
│   │   │   ├── events/list_reveal.rs
│   │   │   ├── events/from_to.rs
│   │   │   ├── events/keyword_pop.rs
│   │   │   ├── events/punch_in.rs     (V6.2) PunchIn: the plan's zoom, Snap or Push, released over 300 ms; drawn by the video track (v6.2implementation D-478)
│   │   │   ├── events/emoji.rs        (V6.2) Emoji: under its word below the lines, over them without room, or beside its line; placed by its chunk, drawn after the captions (v6.2implementation D-483)
│   │   │   ├── events/icon_pop.rs     (V6.2) IconPop: an icon on an accent disc in the event zone, centred on its word, leaving with its chunk (D-483)
│   │   │   ├── events/hook_title.rs   (V6.2) HookTitle: the lockup over the first 2 s in the safe area, entering 0.9 to 1.0; the captions hidden under it (D-484)
│   │   │   ├── events/emphasis.rs     (V6.2) Emphasis: a word, its treatment and program start, handed to the captions, which draw it (v6.2implementation D-481)
│   │   │   ├── watermark.rs
│   │   │   ├── framing.rs             [ONLY] crop rectangle from ClipInfo + CropOffset
│   │   │   ├── video.rs               the per-frame Video command: crop, placement (v4implementation D-130); (V6.2) the camera's zoom, MAX_TOTAL_ZOOM (D-477)
│   │   │   ├── display_list.rs        DisplayList, DrawCmd (backend-neutral); (V6.2) DrawCmd::scale public, for tests that compose a list
│   │   │   └── summary.rs             change_summary(); (V6.2) an Emphasis counts as captions emphasized (D-481)
│   │   └── tests/
│   │       ├── display_list_snapshots.rs
│   │       ├── layout_safe_area.rs
│   │       ├── framing.rs
│   │       ├── determinism.rs         (V6.2) and all five kinds; scene_timing_all_kinds, release only (§10.5)
│   │       ├── styles_use_theme.rs    no colour literal outside theme.rs (D-136)
│   │       ├── timemap_identity.rs    the scene on the identity map (S-1)
│   │       ├── catalog_complete.rs    catalog kinds and drawn kinds are one set (INV-23)
│   │       ├── line_breaks.rs         the line-break table, written before the algorithm (D-167)
│   │       ├── parity.rs              preview equals export (D-172)
│   │       ├── placement.rs           caption placement at the snap points and the extremes (D-171)
│   │       ├── scene_cache.rs         identical output with the cache cold, warm and off (D-173)
│   │       ├── hit.rs                 Scene::hit (D-174)
│   │       ├── camera.rs              (V6.2) the drift's segments; motion off is today's scene, on changes only Video.src (D-476, D-477)
│   │       ├── punch_in.rs            (V6.2) a model's zoom in a scene: held, released, with the drift, switched off, counted (D-478)
│   │       ├── emoji_icon.rs          (V6.2) a model's emoji and icon in a scene: size, under, over, beside and its fallback, the pop and the exit, the disc in the event zone, only images added, hidden or off (D-483)
│   │       ├── lockup.rs              (V6.2) the lockup, pure: short, long, one-word, impossible; as large as fits; against a brute force; widths linear in size (D-484)
│   │       ├── hook_title.rs          (V6.2) the hook in a scene: the video and the hook for 2 s, then the frame without it; no caption hit; cannot fit or off: nothing, not counted (D-484)
│   │       ├── punch_word.rs          (V6.2) a model's punch word in a scene: alone, the reading floor, two 6 s apart, the clip's last word, its sentence only, switched off or hidden (D-482, D-452)
│   │       ├── emphasis.rs            (V6.2) a model's emphasis in a scene: its start to its chunk's end, its own chunk only, a hidden word, switched off, only the captions change, counted (D-481)
│   │       ├── image_bundle.rs        (V6.2) the manifest equals EmojiId and IconId; each file 160 px RGBA with its SHA-256; keys distinct; licences (D-447, D-470)
│   │       ├── common/inputs.rs       each kind's written input and instants: the snapshots' and the goldens' scenes
│   │       ├── common/labeled.rs      the labeled set's files as scene inputs; six of them, for parity.rs, scene_cache.rs, determinism.rs, layout_safe_area.rs
│   │       ├── common/model.rs        (V6.2) a model's plan of given items, validated, for a case (P200)
│   │       ├── common/hook.rs         (V6.2) a model's plan with the hook and extras.hook, validated, for a case (P205)
│   │       ├── common/all_kinds.rs    (V6.2) the reference transcript with all five kinds and Motion on, for determinism.rs, parity.rs and the snapshots (P206)
│   │       ├── common/scenes.rs       written sentences timed like the labeled set, detected and validated; their scenes, watermark on or off
│   │       └── snapshots/             insta snapshot files
│   ├── offcut-render/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                 Renderer
│   │   │   ├── gpu.rs                 device/queue/surface setup
│   │   │   ├── video_pass.rs          external frame import, rotate, crop, scale
│   │   │   ├── vello_backend.rs       DisplayList -> vello::Scene -> overlay texture; (V6.2) the images, decoded once by set_images, drawn in list order (v6.2implementation D-472)
│   │   │   ├── composite.rs           video + overlay -> target
│   │   │   ├── check.rs               (V6.2) TE-30's frame, defined once: the goldens' synthetic source and image_basic's list (v6.2implementation D-474)
│   │   │   └── shaders/video.wgsl
│   │   └── tests/
│   │       ├── golden_frames.rs       native wgpu, software adapter; reads fixtures/golden/
│   │       ├── image_draw.rs          (V6.2) images read as pixels: exact at 1x and at half opacity, no dark fringe, a missing key Unsupported (D-472, §9.2)
│   │       └── common/ssim.rs         the similarity measure of the goldens (v4implementation D-135)
│   ├── offcut-entitlement/
│   │   ├── Cargo.toml                 feature "sign" (server only)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── claims.rs              EntitlementClaims
│   │   │   ├── token.rs               encode/decode compact form
│   │   │   ├── verify.rs              verify_token()
│   │   │   ├── sign.rs                sign_token()  (feature "sign")
│   │   │   └── profile.rs             [ONLY] export_profile(): claims -> ExportProfile
│   │   └── tests/
│   │       ├── token_roundtrip.rs
│   │       └── profile.rs
│   ├── offcut-api-types/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── auth.rs
│   │       ├── account.rs
│   │       ├── billing.rs
│   │       ├── usage.rs
│   │       ├── transcribe.rs          (cloud transcription) TranscribeResponse, CloudWord, HealthResponse
│   │       ├── plan.rs                (V6.2) PlanRequest, PlanWord, PlanResponse (v6.2implementation §7.1)
│   │       ├── analytics.rs           [ONLY] analytics allowlist: event names and prop types
│   │       └── errors.rs              ApiError, ApiErrorCode
│   ├── offcut-wasm-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                 panic hook, init
│   │       ├── media_api.rs           open_demuxer, probe_and_validate, resample; the DemuxerHandle probe and audio-sample methods, validate (v2implementation D-53)
│   │       ├── source_api.rs          (V6.1) open_demuxer_reader (a File read in place), route_probe, the StreamResampler class (v6.1implementation D-399)
│   │       ├── text_api.rs            normalize_transcript
│   │       ├── audio_api.rs           run_chain, measure_prosody
│   │       └── hash_api.rs            Sha256Stream
│   └── offcut-wasm-render/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs                 panic hook, init
│           ├── session.rs             RenderSession (owns Renderer, Scene, Demuxer)
│           ├── scene_api.rs           set_scene (validate + build), source_at, summary; rules_plan (v4implementation D-150)
│           ├── detect_api.rs          detect, redetect_sentence
│           ├── profile_api.rs         export_profile_from_token, preview_profile (v2implementation D-53)
│           ├── check_api.rs           (V6.2) check_source, draw_image_check: TE-30's frame for a development build's check (D-474)
│           └── mux_api.rs             Mp4MuxerHandle
├── server/
│   ├── Cargo.toml
│   ├── Dockerfile
│   ├── .sqlx/                         sqlx offline query cache (generated, committed)
│   ├── migrations/
│   │   ├── 0002_ai_usage.sql          (V6.2) ai_usage, ai_shorts_grants, ai_spend (§52)
│   │   └── 0001_init.sql
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs                     module declarations; lets tests/ build the router
│   │   ├── config.rs                  [ONLY] environment parsing
│   │   ├── state.rs                   AppState
│   │   ├── router.rs                  route table, body limits, layers
│   │   ├── error.rs                   AppError -> ApiError mapping
│   │   ├── headers.rs                 security headers
│   │   ├── client_ip.rs               [ONLY] client IP derivation
│   │   ├── rate_limit.rs              token buckets
│   │   ├── log.rs                     [ONLY] tracing setup and redaction
│   │   ├── auth/
│   │   │   ├── mod.rs                 handlers
│   │   │   ├── magic_link.rs
│   │   │   ├── session.rs             refresh rotation, cookie
│   │   │   ├── access_token.rs
│   │   │   └── extract.rs             AuthUser extractor
│   │   ├── entitlement/
│   │   │   ├── mod.rs                 handler
│   │   │   └── issue.rs               [ONLY] plan + remaining-exports computation and signing
│   │   ├── usage/
│   │   │   └── mod.rs                 receipt handler
│   │   ├── billing/
│   │   │   ├── mod.rs                 handlers
│   │   │   ├── provider.rs            BillingProvider trait, BillingEvent
│   │   │   ├── adapter.rs             the chosen provider's adapter (TE-9)
│   │   │   ├── offers.rs              [ONLY] offer -> provider price id -> plan
│   │   │   └── webhook.rs             verify, dedupe, apply
│   │   ├── analytics/
│   │   │   ├── mod.rs                 ingest handler
│   │   │   └── retention.rs           90-day purge task
│   │   ├── account/
│   │   │   ├── mod.rs                 handlers
│   │   │   ├── delete.rs
│   │   │   ├── export.rs
│   │   │   └── notify.rs
│   │   ├── mail/
│   │   │   ├── mod.rs                 Mailer trait
│   │   │   ├── smtp_mailer.rs         SMTP over STARTTLS through the chosen provider (TE-8)
│   │   │   └── templates.rs           magic-link email
│   │   ├── transcribe/                (cloud transcription) POST /transcribe; stores nothing (P-16)
│   │   │   ├── mod.rs
│   │   │   ├── provider.rs            Transcriber trait, UnconfiguredTranscriber
│   │   │   ├── groq.rs                the only file that names the transcription provider
│   │   │   ├── align.rs               written forms onto the provider's words (cloudimplementation D-360)
│   │   │   ├── ogg.rs                 OpusHead and the last granule of an Ogg file; no decoding
│   │   │   ├── budget.rs              [ONLY] the audio budget per hour and per day
│   │   │   └── route.rs               the handler: auth, limits, type, body, Ogg, budget, provider
│   │   ├── plan/                      (V6.2, V6.4, V6.9) the only code that calls the AI provider
│   │   │   ├── mod.rs
│   │   │   ├── provider.rs            PlanProvider trait, UnconfiguredPlanProvider
│   │   │   ├── deepseek.rs            (V6.2) the only file that names the AI provider (v6.2implementation D-500)
│   │   │   ├── parse.rs               (V6.2) the model's answer to an EditPlan, item by item; the server sets format, catalog, source
│   │   │   ├── route.rs               POST /plan
│   │   │   ├── shorts.rs              POST /shorts
│   │   │   ├── chat.rs                POST /chat-edit
│   │   │   ├── quota.rs               ai_usage, grants, in-memory anonymous bucket
│   │   │   └── spend.rs               [ONLY] AI_MONTHLY_CAP_USD and ai_spend
│   │   └── db/
│   │       ├── mod.rs                 pool
│   │       ├── users.rs
│   │       ├── magic_links.rs
│   │       ├── sessions.rs
│   │       ├── subscriptions.rs
│   │       ├── webhook_events.rs
│   │       ├── usage_receipts.rs
│   │       ├── analytics_events.rs
│   │       ├── ai_usage.rs            (V6.2) ai_usage: a success counted, the month read; the export's AI rows (§52, D-461)
│   │       ├── ai_spend.rs            (V6.2) ai_spend: add a call's cost, read the month's total (§52, D-463)
│   │       └── platform_waitlist.rs
│   └── tests/
│       ├── common/mod.rs              test app, fake Mailer, fake BillingProvider
│       ├── common/plan.rs             (V6.2) the plan tests' person, Creator, request and month's counts
│       ├── auth_flow.rs
│       ├── entitlement_rules.rs
│       ├── usage_receipts.rs
│       ├── webhook_idempotency.rs
│       ├── billing_events.rs
│       ├── billing_routes.rs          checkout and the portal, on the fake merchant
│       ├── dodo_adapter.rs            the adapter, from fixtures/dodo/, with no network
│       ├── fixtures/dodo/             real test-mode deliveries, with people's details and ids replaced
│       ├── analytics_allowlist.rs
│       ├── rate_limit.rs
│       ├── account_delete.rs
│       ├── migration_rollback.rs      (V6.2) a rollback across a migration: V6.1 after §14.4's step, V6.2 back, unknown versions ignored
│       ├── plan_route.rs
│       ├── shorts_route.rs
│       ├── chat_route.rs
│       ├── ai_quota.rs
│       ├── ai_cap.rs
│       ├── account_export.rs
│       ├── log_redaction.rs
│       ├── no_media_routes.rs         INV-4, with DC-14's one exception
│       ├── transcribe_route.rs        (cloud transcription) POST /transcribe on a fake provider
│       ├── transcribe_window_log.rs   (V6.1) the log line says window: true or false, never the index; one test per binary (v6.1implementation D-411)
│       ├── groq_adapter.rs            (cloud transcription) the adapter, from fixtures/groq/, against a local mock
│       ├── fixtures/groq/             real Groq responses for the reference clip's audio, request ids replaced
│       ├── deepseek_adapter.rs        (V6.2) the plan adapter, from fixtures/deepseek/, against a local mock
│       ├── fixtures/deepseek/         real DeepSeek answers on the labelled scripts (P188, P188b), ids redacted
│       ├── fixtures/ogg/              small Ogg Opus files: a tone from ffmpeg and one from the browser's encoder
│       └── notify_me.rs               /healthz and /notify-me against a real Postgres
├── web/
│   ├── package.json
│   ├── index.html
│   ├── vite.config.ts
│   ├── vitest.config.ts
│   ├── playwright.config.ts
│   ├── tsconfig.json
│   ├── eslint.config.js               boundaries + restricted globals (§7)
│   ├── vercel.json                    [ONLY] response headers and /api rewrite
│   ├── LICENSES.md                    licences of shipped assets: Inter (OFL-1.1), Lucide icons (ISC) (v4implementation D-180)
│   ├── public/
│   │   ├── robots.txt
│   │   ├── og.png                     the open-graph image of a shared link, 1200 × 630, from the hero clip's poster (designimplementation D-335)
│   │   └── ort/                       ONNX Runtime .wasm files copied at build by vite.config.ts
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── routes.tsx
│   │   ├── gen/
│   │   │   ├── domain.ts              generated from offcut-types (never edited)
│   │   │   └── api.ts                 generated from offcut-api-types (never edited)
│   │   ├── config/
│   │   │   ├── env.ts                 [ONLY] import.meta.env access
│   │   │   ├── allowlist-hosts.ts     [ONLY] hosts the client may contact
│   │   │   ├── pricing.ts             [ONLY] display prices (PS §11)
│   │   │   ├── model-manifest.json    model files: path, size, sha256
│   │   │   ├── models/                (V6.5, V6.11) one manifest per extra model: face, person; the ASR manifest stays above (DC-4)
│   │   │   ├── features.ts            [ONLY] FeatureFlags from a Vite define (§41.3): five flags, each with a default (V6.2 adds `ai_plan` and `motion`, off: v6.2implementation D-443); `featureFlags(on, off)` for vite.config.ts (v6.1implementation D-393)
│   │   │   ├── features.test.ts       (V6.1) the defaults, OFFCUT_FEATURES_ON and _OFF, an unknown name and a name in both lists
│   │   │   └── entitlement-public-key.ts
│   │   ├── copy/
│   │   │   ├── messages.ts            [ONLY] all user-facing copy keyed by code, with the files it imports (INV-15)
│   │   │   ├── landing.ts             the landing page's and the prices' copy, imported by messages.ts (designimplementation D-321)
│   │   │   ├── transcription.ts       (cloud transcription) the card, the processing line and the two toasts (cloudimplementation D-380)
│   │   │   ├── long.ts                (V6.1) the long page's words, REJECT_HDR and the two E_SOURCE_* (v6.1implementation §10.5, D-426)
│   │   │   ├── plan.ts                (V6.2) the AI plan's, the extras' and Enhance's words, imported by messages.ts (v6.2implementation §12, D-491)
│   │   │   ├── plan.test.ts           (V6.2) which line each fallback and each failed regeneration shows; the settings line names DeepSeek and promises no keeping
│   │   │   └── messages.test.ts
│   │   ├── net/
│   │   │   ├── http.ts                [ONLY] fetch to the app origin (/api/v1)
│   │   │   ├── http.test.ts
│   │   │   ├── asset-fetch.ts         [ONLY] fetch to the asset CDN
│   │   │   ├── api-client.ts          typed route functions
│   │   │   └── api-client.test.ts
│   │   ├── platform/
│   │   │   ├── capability.ts
│   │   │   ├── capability.test.ts
│   │   │   ├── simd-probe.ts
│   │   │   ├── wake-lock.ts           (V6.1) the screen wake lock and the title while a long source works (D-418)
│   │   │   └── broadcast.ts           BroadcastChannel "offcut-auth"
│   │   ├── state/
│   │   │   ├── capability-store.ts
│   │   │   ├── model-store.ts
│   │   │   ├── clip-store.ts
│   │   │   ├── source-store.ts        (V6.1) the long job's state
│   │   │   ├── shorts-store.ts        (V6.4) the list of shorts
│   │   │   ├── chat-store.ts          (V6.9) the conversation of the open short
│   │   │   ├── queue-store.ts         (V6.10) queued sources; holds File objects in memory only
│   │   │   ├── preview-store.ts
│   │   │   ├── export-store.ts
│   │   │   ├── auth-store.ts
│   │   │   ├── cloud-store.ts         (cloud transcription) what /healthz said, and this device's choice
│   │   │   ├── plan-store.ts          (V6.2) the settings switch "AI edit plan" for the page (default on)
│   │   │   ├── entitlement-store.ts
│   │   │   ├── entitlement-store.test.ts
│   │   │   ├── ui-store.ts            selected tab, follow switch, editing word, toasts, save state (v4implementation §17)
│   │   │   ├── history.ts             undo and redo of EditState, 50 steps (D-184)
│   │   │   ├── history.test.ts
│   │   │   ├── blockers.ts            [ONLY] blocker codes and conditions
│   │   │   ├── blockers.test.ts
│   │   │   └── machines/
│   │   │       ├── transition.ts      shared transition helper; illegal-transition handling
│   │   │       ├── model-machine.ts
│   │   │       ├── model-machine.test.ts
│   │   │       ├── clip-machine.ts
│   │   │       ├── clip-machine.test.ts
│   │   │       ├── export-machine.ts
│   │   │       ├── export-machine.test.ts
│   │   │       ├── source-machine.ts  (V6.1) §43.8
│   │   │       ├── source-machine.test.ts
│   │   │       ├── auth-machine.ts
│   │   │       └── auth-machine.test.ts
│   │   ├── usecases/
│   │   │   ├── start-app.ts
│   │   │   ├── import-clip.ts
│   │   │   ├── run-pipeline.ts
│   │   │   ├── run-pipeline.test.ts
│   │   │   ├── transcribe-cloud.ts    (cloud transcription) the only use-case that sends audio (D-372)
│   │   │   ├── transcribe-cloud.test.ts
│   │   │   ├── set-transcription.ts   (cloud transcription) the card's choice, for the page and the next visit
│   │   │   ├── edit-word.ts
│   │   │   ├── toggle-event.ts
│   │   │   ├── set-style.ts
│   │   │   ├── set-crop-offset.ts
│   │   │   ├── set-caption-position.ts  (V4) the caption block's position (v4implementation D-171)
│   │   │   ├── set-highlight-numbers.ts  (V4) numbers in the accent colour, on or off (v4implementation D-169)
│   │   │   ├── set-highlight-numbers.ts  (V4) numbers in the accent colour on or off (D-169)
│   │   │   ├── replace-all.ts         (V4) "Change all N" for a repeated mis-hearing (D-175)
│   │   │   ├── undo-redo.ts           (V4) D-184
│   │   │   ├── review-update.ts       (V4) the shared update rule of an edit: pause, queue, setScene, restore, undo (D-147)
│   │   │   ├── import-source.ts       (V6.1) route in place, gate, Start, the hand-over of a short file (D-421)
│   │   │   ├── import-source.test.ts
│   │   │   ├── run-source-pipeline.ts  (V6.1, V6.4, V6.5) the long job, resumable
│   │   │   ├── continue-source.ts     (V6.1) Continue after a pause, a failure or a reload; the same file dropped again (D-419)
│   │   │   ├── run-source-pipeline.test.ts
│   │   │   ├── transcribe-source.ts   (V6.1) a source's cloud windows to its master transcript; the second use-case that sends audio (v6.1implementation §8.1)
│   │   │   ├── transcribe-source.test.ts
│   │   │   ├── request-plan.ts        (V6.2) POST /plan and its fallbacks; the only use-case that builds a `PlanRequest`: the heard words, never the person's edits; off, signed out or offline send nothing; 20 s in all (v6.2implementation D-466, D-494, D-498)
│   │   │   ├── request-plan.test.ts   (V6.2) each outcome; no request when off, signed out or offline; the body; the 20 s timer; never a throw
│   │   │   ├── choose-plan.ts         (V6.2) which plan a run builds its scene from: the plan kept for the clip (no request), or a model's that keeps an item, else the rules plan; keeps it, tells a fallback once, `plan_result` (v6.2implementation D-467, D-469, D-490)
│   │   │   ├── regenerate-plan.ts     (V6.2) "Regenerate plan": the only request for a clip that has a plan; on success the plan is replaced in the update queue and kept, on failure nothing changes (§8.3)
│   │   │   ├── regenerate-plan.test.ts  (V6.2) success; each failure leaves the clip as it was; a clip that went; not ready or flag off asks nothing
│   │   │   ├── set-ai-plan.ts         (V6.2) the settings switch "AI edit plan", for the page and the next visit
│   │   │   ├── set-ai-plan.test.ts    (V6.2) on by default and when unreadable; kept under `aiPlan`; a failed write
│   │   │   ├── set-enhance.ts         (V6.2) the Enhance switches: a review change each (one undo step), remembered under `enhanceDefault`; a new clip's start; "Back to recommended" (v6.2implementation §11, D-486, D-488)
│   │   │   ├── set-enhance.test.ts    (V6.2) undo and redo swap the voice; remembered; flags off ignore the memory; a failed swap restores
│   │   │   ├── voice-program.ts       (V6.2) the program's audio on a voice: `prog48.f32` for Clean (today's read), `lvl48.f32` assembled for Level (D-487)
│   │   │   ├── prepare-short.ts       (V6.3, V6.4) one ShortSpec → a ready virtual clip (§45.7)
│   │   │   ├── request-shorts.ts      (V6.4) POST /shorts and its fallback
│   │   │   ├── open-short.ts          (V6.4) load a short into the editor
│   │   │   ├── export-shorts.ts       (V6.4) sequential batch export
│   │   │   ├── add-asset.ts           (V6.8)
│   │   │   ├── remove-asset.ts        (V6.8)
│   │   │   ├── send-chat-edit.ts      (V6.9) POST /chat-edit; validates and applies ops
│   │   │   ├── manage-queue.ts        (V6.10)
│   │   │   ├── prepare-matte.ts       (V6.11) §51.1
│   │   │   ├── preview-interact.ts    (V4) hover and click hit tests, drag commits (D-183)
│   │   │   ├── style-thumbnails.ts    (V4) the style cards rendered from the clip (D-186)
│   │   │   ├── control-preview.ts
│   │   │   ├── start-export.ts
│   │   │   ├── start-export.test.ts
│   │   │   ├── cancel-job.ts
│   │   │   ├── dev-faults.ts          (V4) development builds only: crash, hang, fail a call on purpose (v4implementation D-155, G 14.5)
│   │   │   ├── restore-clip.ts
│   │   │   ├── restore-session.ts     (V6) C-1's session step; hands `net` and the outbox what they may not import
│   │   │   ├── restore-session.test.ts
│   │   │   ├── sign-in.test.ts        (V6) request-magic-link, complete-sign-in and sign-out
│   │   │   ├── account.test.ts        (V6) start-checkout, confirm-checkout, open-billing-portal, delete-account and export-account-data
│   │   │   ├── request-magic-link.ts
│   │   │   ├── complete-sign-in.ts
│   │   │   ├── sign-out.ts
│   │   │   ├── start-checkout.ts
│   │   │   ├── confirm-checkout.ts
│   │   │   ├── open-billing-portal.ts
│   │   │   ├── answer-post-export.ts
│   │   │   ├── submit-notify-me.ts
│   │   │   ├── clear-local-data.ts
│   │   │   ├── delete-account.ts
│   │   │   └── export-account-data.ts
│   │   ├── entitlement/
│   │   │   ├── entitlement.ts         effective entitlement = token - outbox
│   │   │   └── entitlement.test.ts
│   │   ├── analytics/
│   │   │   ├── client.ts              track(), batching, flush
│   │   │   ├── client.test.ts
│   │   │   └── events.types.test.ts   type-level: no free-text props
│   │   ├── models/
│   │   │   ├── model-manager.ts
│   │   │   ├── download.ts            ranged, resumable download
│   │   │   └── download.test.ts
│   │   ├── persistence/
│   │   │   ├── db.ts                  open + migrations
│   │   │   ├── schema.ts              [ONLY] IndexedDB store names, keys, version
│   │   │   ├── opfs.ts                [ONLY] OPFS path layout
│   │   │   ├── quota.ts
│   │   │   ├── clips-repo.ts
│   │   │   ├── transcript-repo.ts
│   │   │   ├── edits-repo.ts
│   │   │   ├── plans-repo.ts          (V6.2) the plans store at structure version 3, its only reader and writer and the only caller of `openPlansDb` (v6.2implementation D-469)
│   │   │   ├── plans-repo.test.ts     (V6.2) 1 → 3 and 2 → 3 keep every record; the clip path still opens 1 and the long path 2; records, the newer-schemaVersion rule, the sweep
│   │   │   ├── sources-repo.ts        (V6.1) sources, checkpoints, master transcripts
│   │   │   ├── sources-repo.test.ts   (V6.1) records, compound window keys, the newer-schemaVersion rule, removal and the sweep
│   │   │   ├── shorts-repo.ts         (V6.4)
│   │   │   ├── assets-repo.ts         (V6.8)
│   │   │   ├── creator-memory-repo.ts (V6.10)
│   │   │   ├── render-cache-repo.ts
│   │   │   ├── entitlement-repo.ts
│   │   │   ├── prefs-repo.ts          (cloud transcription) the card's choice in the meta store (D-367)
│   │   │   ├── receipt-outbox.ts
│   │   │   └── receipt-outbox.test.ts
│   │   ├── wasm/
│   │   │   ├── load-core.ts           [ONLY] instantiate offcut_core.wasm
│   │   │   └── load-render.ts         [ONLY] instantiate offcut_render.wasm
│   │   ├── workers/
│   │   │   ├── protocol.ts            [ONLY] message types for all four workers
│   │   │   ├── rpc.ts                 request/response/progress/cancel plumbing (the page side)
│   │   │   ├── rpc-worker.ts          serveWorker, the worker side of rpc.ts (v4implementation D-150, D-151)
│   │   │   ├── rpc.test.ts
│   │   │   ├── pool.ts                worker lifecycle; typed clients
│   │   │   ├── media.worker.ts
│   │   │   ├── media/import.ts
│   │   │   ├── media/audio-decode.ts
│   │   │   ├── media/file-random-access.ts  (V6.1) a dropped File read in place: 1 MiB blocks, 8 kept (v6.1implementation D-400)
│   │   │   ├── media/file-random-access.test.ts  (V6.1) reads across blocks, the cache, NotReadableError
│   │   │   ├── media/audio-region.ts  (V6.1) decodeAudioRegion, writeWholePcm16, the level histogram (§7.2, D-402 to D-404)
│   │   │   ├── media/audio-region.test.ts  (V6.1) placement, resume equals a run from 0, rate change, level, cancel
│   │   │   ├── asr.worker.ts
│   │   │   ├── asr/whisper-runtime.ts
│   │   │   ├── asr/model-cache-adapter.ts   serves model files from OPFS; blocks remote loads
│   │   │   ├── asr/word-timestamps.ts
│   │   │   ├── asr/ogg-opus.ts            (cloud transcription) Opus packets into an Ogg file; the end trimmed to the clip
│   │   │   ├── asr/ogg-opus.test.ts
│   │   │   ├── asr/cloud-audio.ts         (cloud transcription) pcm16 to Ogg Opus with WebCodecs; the silence guard (D-382)
│   │   │   ├── asr/cloud-audio.test.ts
│   │   │   ├── asr/vision-runtime.ts      (V6.5) the face detector session (DC-10)
│   │   │   ├── asr/person-runtime.ts      (V6.11) the person-matte session (DC-10)
│   │   │   ├── audio.worker.ts
│   │   │   ├── render.worker.ts
│   │   │   ├── render/video-source.ts       demux + VideoDecoder + frame queue
│   │   │   ├── render/video-source.test.ts  the decoder while a call holds the session (v5implementation D-240)
│   │   │   ├── render/frame-decoder.ts      the VideoDecoder's management, out of video-source.ts (v4implementation D-150)
│   │   │   ├── render/source-position.ts  (V6.3) recording time → source time, through the Assembly (INV-27)
│   │   │   ├── render/preview-job.ts        play, pause, seek, clock, the resize rule (D-148, D-149, D-150)
│   │   │   ├── render/export-job.ts         the export handlers, out of render.worker.ts (D-150)
│   │   │   ├── render/image-check.ts        (V6.2) development builds only: TE-30's frame drawn and read back (v6.2implementation D-474)
│   │   │   ├── render/preview-loop.ts
│   │   │   ├── render/export-loop.ts
│   │   │   ├── render/export-loop.test.ts   the loop with fake WebCodecs: frames, program time, restarts (v5implementation D-219)
│   │   │   ├── render/encoders.ts           [ONLY] WebCodecs encoder configs and ladder
│   │   │   └── render/opfs-sink.ts
│   │   └── ui/
│   │       ├── pages/
│   │       │   ├── LandingPage.tsx
│   │       │   ├── EditorPage.tsx
│   │       │   ├── LongPage.tsx       (V6.1) /app/long: the drop zone, the gate, the work, the transcript (v6.1implementation §10.1)
│   │       │   ├── UnsupportedPage.tsx
│   │       │   ├── AuthCallbackPage.tsx
│   │       │   ├── AccountPage.tsx
│   │       │   ├── SettingsPage.tsx
│   │       │   ├── ShortsPage.tsx     (V6.4) the list of shorts (§45.8)
│   │       │   └── LegalPage.tsx
│   │       ├── landing/               the landing page's sections (designimplementation D-322); the only files that may import GSAP and Motion (D-319)
│   │       │   ├── TopBar.tsx         section 0: wordmark, anchors, "Start free", the progress bar
│   │       │   ├── account-link.ts    (V6.2, D-501) the top bar's "Sign in", or "Account" once signed in, to /account
│   │       │   ├── account-link.test.ts  (V6.2) the link's words in each account state
│   │       │   ├── Hero.tsx           section 1: the headline, the way in, the capability line, the hero clip
│   │       │   ├── CaptionedHeadline.tsx  the captioned headline and the final call's pill (M1, M10)
│   │       │   ├── Marquee.tsx        section 2
│   │       │   ├── BeforeAfter.tsx    section 3
│   │       │   ├── Features.tsx       section 4: the eight feature cards
│   │       │   ├── HowItWorks.tsx     section 5
│   │       │   ├── TryIt.tsx          section 6: the unchanged DropZone
│   │       │   ├── Privacy.tsx        section 7
│   │       │   ├── Pricing.tsx        section 8: the plans, from PRICES_USD and LIMITS
│   │       │   ├── ComingSoon.tsx     section 9: every post-V6 feature, marked "Coming soon"
│   │       │   ├── Updates.tsx        section 10: the unchanged NotifyMeForm
│   │       │   ├── FinalCall.tsx      section 11
│   │       │   ├── SiteFooter.tsx     section 12: legal links, contact, settings
│   │       │   ├── LoopVideo.tsx      the one player of the landing clips (designimplementation §9.5)
│   │       │   ├── parts.tsx          PhoneFrame, Chip, SectionHead, CtaLink, BentoCard, the Motion scope
│   │       │   ├── gsap.ts            loadGsap() and useGsap(): GSAP by import(), inside matchMedia, reverted on unmount
│   │       │   └── gsap-runtime.ts    GSAP and its scroll plugin, registered: the one module gsap.ts imports with import(), so both share one chunk
│   │       ├── assets/
│   │       │   ├── inter-latin.woff2  the interface font, Latin subset of the scene's Inter, emitted hashed under /assets/ (v4implementation D-180)
│   │       │   ├── jetbrains-mono-latin.woff2  the landing page's accent face: JetBrains Mono 700, Latin subset (designimplementation D-330)
│   │       │   └── landing/           the landing clips' WebP posters, one per slot, emitted hashed (designimplementation §9.4)
│   │       ├── icons.tsx              inline SVG icons (D-181)
│   │       ├── dev-hooks.ts           `__offcutDev`: the page's development hooks, never in a production build (v4implementation D-155, v4changelog Entry 21)
│   │       ├── hooks/
│   │       │   └── use-shortcuts.ts   the keyboard shortcuts (D-185)
│   │       ├── layout/
│   │       │   ├── EditorLayout.tsx   top bar, stage, side panel, live region (D-178)
│   │       │   ├── LongLayout.tsx     (V6.1) the editor's bar and one column; reads no clip store (D-423)
│   │       │   ├── TopBar.tsx         save state, undo, redo, Export or the export pill
│   │       │   ├── Stage.tsx          the state's card or the player
│   │       │   └── SidePanel.tsx      the four tabs (in place of a ReviewPanel)
│   │       ├── primitives/            (D-181)
│   │       │   ├── Button.tsx
│   │       │   ├── IconButton.tsx
│   │       │   ├── Tooltip.tsx
│   │       │   ├── Tabs.tsx
│   │       │   ├── Switch.tsx
│   │       │   ├── SegmentedControl.tsx
│   │       │   ├── Slider.tsx
│   │       │   ├── Toast.tsx
│   │       │   ├── Spinner.tsx
│   │       │   ├── Skeleton.tsx
│   │       │   ├── PanelSection.tsx
│   │       │   └── Kbd.tsx
│   │       ├── components/
│   │       │   ├── CapabilityGate.tsx
│   │       │   ├── DropZone.tsx
│   │       │   ├── TranscriptionChoice.tsx  (cloud transcription) the drop zone's Cloud / On this device card
│   │       │   ├── ModelDownloadPanel.tsx
│   │       │   ├── ProcessingFeed.tsx
│   │       │   ├── ProcessingCard.tsx     the processing state's card (v4implementation D-188)
│   │       │   ├── plan-fallback.ts       (V6.2, D-501) the AI plan's fallback toast; signed out, its "Sign in" opens SignInDialog over the editor
│   │       │   ├── plan-fallback.test.ts  (V6.2) the toast and its action for each reason
│   │       │   ├── PreviewPlayer.tsx
│   │       │   ├── PreviewGestures.tsx    the gestures and their chrome over the preview (v4implementation D-183)
│   │       │   ├── CaptionEditor.tsx
│   │       │   ├── CaptionWordControls.tsx  a transcript word's field and menu (v4implementation D-182)
│   │       │   ├── PlanItemList.tsx       the Visuals tab: one row per plan item; (V6.2) the five kinds' icons and "Regenerate plan"
│   │       │   ├── TextExtrasPanel.tsx    (V6.2) "Text for your post": the hook, caption and hashtags that passed, each with Copy (v6.2implementation D-454)
│   │       │   ├── StylePanel.tsx         the style cards (four since V6.2: Glow) and Highlight numbers
│   │       │   ├── EnhancePanel.tsx       (V6.2) Voice cleanup and Motion under the Style tab's cards, with `motion` on (v6.2implementation §11)
│   │       │   ├── StylePicker.tsx
│   │       │   ├── FramingPanel.tsx       caption position and crop
│   │       │   ├── CropOffsetControl.tsx
│   │       │   ├── CutsSection.tsx        renders nothing in V4 (V6.3)
│   │       │   ├── ShortcutsSheet.tsx     (D-185)
│   │       │   ├── ToastHost.tsx
│   │       │   ├── SourceProgress.tsx  (V6.1) the long job's stages, pause and continue
│   │       │   ├── SourceGate.tsx     (V6.1) signed out, cloud checking or off, disk, then Start (§10.2)
│   │       │   ├── SourceTranscript.tsx  (V6.1) the end screen: the sentences at their times (§10.4)
│   │       │   ├── ShortCard.tsx      (V6.4) poster, title, length, reason, keep or delete
│   │       │   ├── AssetBox.tsx       (V6.8)
│   │       │   ├── ChatPanel.tsx      (V6.9)
│   │       │   ├── QueuePanel.tsx     (V6.10)
│   │       │   ├── ExportButton.tsx
│   │       │   ├── ExportProgress.tsx
│   │       │   ├── SignInDialog.tsx
│   │       │   ├── ChangeSummary.tsx
│   │       │   ├── PostExportQuestion.tsx
│   │       │   ├── UpgradePrompt.tsx
│   │       │   ├── WhatLeavesTable.tsx
│   │       │   ├── RecentClips.tsx
│   │       │   ├── RejectionPanel.tsx
│   │       │   ├── ErrorPanel.tsx
│   │       │   └── NotifyMeForm.tsx
│   │       └── styles/
│   │           ├── tokens.css
│   │           ├── pages.module.css
│   │           ├── components.module.css
│   │           ├── editor.module.css      the editor layout (v4implementation §17.6)
│   │           ├── long.module.css        (V6.1) the long page's column, steps and transcript list
│   │           ├── panel.module.css       the side panel and its tabs
│   │           ├── primitives.module.css  the primitives (D-181)
│   │           ├── controls.module.css    the primitives that hold a value: Tabs, Switch, SegmentedControl, Slider (v4changelog Entry 19)
│   │           ├── preview.module.css     the player, its transport and the direct-manipulation chrome
│   │           ├── landing.module.css     the landing page: root, tokens, top bar, hero, shared parts (designimplementation §7)
│   │           ├── landing-sections.module.css  the landing page's sections after the hero
│   │           └── landing-cards.module.css  the features' bento cards and the "Coming soon" cards (split to stay under 700 lines)
│   └── tests-e2e/
│       ├── tsconfig.json              Node types for the tests and the bench only (v2implementation D-63)
│       ├── helpers/network-capture.ts
│       ├── helpers/fake-api.ts
│       ├── helpers/long-windows.ts    (V6.1) each window's answer from the cloud fixture (D-431); the long page's helpers
│       ├── helpers/fixtures.ts
│       ├── helpers/ssim.ts            (V6.2) the golden frames' SSIM, a copy of offcut-render/tests/common/ssim.rs (D-474)
│       ├── helpers/plan-fixtures.ts   (V6.2) the AI plan's fixtures: the cloud answer with marker words planted, the recorded plan, the regenerated one (D-468)
│       ├── media.setup.ts             fills fixtures/.cache/ for the media suites and the bench (v2buildguide item 6b)
│       ├── landing.spec.ts
│       ├── capability.spec.ts
│       ├── unsupported.spec.ts
│       ├── model-download.spec.ts
│       ├── rejections.spec.ts
│       ├── pipeline-preview.spec.ts
│       ├── preview-sync.spec.ts
│       ├── review-edit.spec.ts
│       ├── signin.spec.ts
│       ├── export-free.spec.ts
│       ├── export-creator.spec.ts
│       ├── timeline-preserved.spec.ts
│       ├── checkout.spec.ts
│       ├── cancel.spec.ts
│       ├── cloud-transcribe.spec.ts   (cloud transcription) the cloud path, fallbacks, the card, identity with the switch on
│       ├── fixtures/cloud/            the TranscribeResponse the real adapter gave for speech_60s_portrait's audio (P163)
│       ├── fixtures/plan/             (V6.2) a model's PlanResponse for that transcript, made by hand from TE-15's DeepSeek Flash answers (P210)
│       ├── screens.spec.ts            D1-only screen capture for the design reviews; never in CI (v4implementation D-191)
│       ├── restore.spec.ts
│       ├── failure-recovery.spec.ts
│       ├── api-cold-start.spec.ts
│       ├── privacy-network.spec.ts
│       ├── flags-identity.spec.ts     (V6.1, as built) one clip on both builds: the same records, render-cache key and export (INV-26)
│       ├── enhance-panel.spec.ts      (V6.2) the Enhance switches on the flags-on build: Motion off shows the crop, Voice off plays lvl48, the edit budget, undo, the memory, Back to recommended
│       ├── ai-plan.spec.ts            (V6.2) the AI plan on the flags-on build through the fake /plan: drawn, switched, copied, run again, regenerated; signed out, the switch off, the fallbacks; the network rules (D-468)
│       ├── route-long.spec.ts         (V6.1, as built) the two ways in, the gate, a 12-minute video through the fake windows, the refusals
│       ├── high-res.spec.ts           (V6.1)
│       ├── image-parity.spec.ts       (V6.2) TE-30: the browser's image_basic against its golden, D1 only (D-474)
│       ├── long-resume.spec.ts        (V6.1) tab killed, tab hidden, source evicted
│       ├── cuts_applied.spec.ts       (V6.3)
│       ├── stitched.spec.ts           (V6.3) out-of-order parts
│       ├── long-to-shorts.spec.ts     (V6.4)
│       ├── vision-pass.spec.ts        (V6.5)
│       ├── music-import.spec.ts       (V6.7)
│       ├── join_transition.spec.ts    (V6.8)
│       ├── asset-box.spec.ts          (V6.8)
│       ├── chat-edit.spec.ts          (V6.9)
│       ├── queue.spec.ts              (V6.10)
│       ├── prepare-matte.spec.ts      (V6.11)
│       ├── settings-what-leaves.spec.ts
│       └── clear-local-data.spec.ts
├── verify/
│   ├── verify_mp4.py
│   ├── watermark_mask_720.png          watermark template drawn independently of the renderer
│   ├── make_watermark_mask.py          the one-time tool that drew it, from the font file (v5implementation D-206)
│   ├── test_verify_mp4.py              proves each check tells a good file from a bad one; runs in CI (v5implementation D-209)
│   ├── requirements.txt
│   └── README.md
├── fixtures/
│   ├── gen_fixtures.sh                generates the synthetic and derived fixtures with the ffmpeg CLI into .gen/ (dev and CI tool)
│   ├── manifest.json                  fixture name -> recipe kind, expected outcome, probe properties, tests (26 fixtures)
│   ├── manifest-long.json             (V6.1) the four long-path fixtures, apart from manifest.json (v6.1implementation D-430)
│   ├── speech/README.md               scripts read aloud, the clips as recognized, how each speech fixture is derived
│   ├── labeled/                       >= 30 labeled transcripts (JSON), one file each, frozen before the rules (v3implementation D-87)
│   ├── labeled/SCHEMA.md
│   └── golden/                        PNG goldens per style x event kind; 16 at 540x960 from a synthetic source (v4implementation D-135); (V6.2) image_basic, the bundle's images (v6.2implementation §9.1); motion_mid, the camera drift (§10.1); punch_snap, punch_push, the plan's zoom (D-478); glow_style, the Glow style (D-479, D-480); emphasis_colour, emphasis_underline, emphasis_highlight, Bold (D-481); punch_word, Clean (D-482); emoji_under, Bold at Middle, and icon_pop, Clean at Bottom (D-483); hook_short, hook_long, Clean (D-484)
├── corpus/
│   ├── manifest.json                  clip id, sha256, tags (vfr, rotated, noisy, accent)
│   ├── fetch_corpus.sh
│   └── run_corpus.ts
├── bench/
│   ├── device-bench.ts
│   └── results/                       committed runs, <device>-<date>-<clip>.json (v3implementation §19.19); .gitkeep
├── scripts/
│   ├── build-wasm.sh
│   ├── gen-image-bundle.mjs           (V6.2) the image bundle: fetches each upstream file at its pinned version, writes the 160 px PNGs, manifest.json and LICENSES/ (v6.2implementation D-470)
│   ├── gen-long-fixture.sh            (V6.1) the 45-minute local fixtures into fixtures/.gen-local/, D1 only (v6.1implementation D-430)
│   ├── gen-types.sh
│   ├── check-gen-clean.sh
│   ├── check-file-tree.mjs            fails if a source file is absent from this section
│   ├── v62-gate.sh                    (V6.2) every prompt's gate in one run, printing failures only (v6.2implementation D-499)
│   ├── check-hosts.mjs                greps the built bundle for URLs outside allowlist-hosts.ts
│   ├── check-copy-codes.mjs           every code in gen/domain.ts has copy in messages.ts
│   ├── check-external-facts.mjs       prints the free-tier facts to re-verify (TE-5..TE-11)
│   ├── check-headers.mjs              compares a deployment's response headers with web/vercel.json
│   ├── wait-for-version.mjs           waits until /healthz of a deployment reports one commit
│   ├── metrics.sql                    queries for the PS §20.5 validation metrics
│   └── upload-assets.sh               uploads model files and media to the asset CDN
└── docs/
    ├── product.md
    ├── technicalspec.md
    ├── buildplan.md                   build order as versions V1-V10 and V6.1-V6.11 over the §28 steps
    ├── v1/v1implementation.md         file-by-file implementation plan for V1 (M0.1)
    ├── v1/v1buildguide.md             step-by-step build order for V1, with a milestone per phase
    ├── v1/coding-prompts.md           the 30 V1 coding prompts, run in order
    ├── v1/v1changelog.md              record of every change made while building V1
    ├── v2/v2implementation.md         file-by-file implementation plan for V2 (M0.2 to M0.4)
    ├── v2/v2implementation-notes.md   analysis and consistency check written around the V2 plan
    ├── v2/v2buildguide.md             step-by-step build order for V2, with a milestone per phase
    ├── v2/coding-promptsv2.md         the 30 V2 coding prompts (31 to 60), run in order
    ├── v2/experiments.md              outcome of every experiment V2 runs (TE-n, E-n) and the M0 gate decision
    ├── v2/v2changelog.md              record of every change made while building V2
    ├── v3/v3implementation.md         file-by-file implementation plan for V3 (M1.1 to M1.3)
    ├── v3/v3buildguide.md             step-by-step build order for V3, with a milestone per phase
    ├── v3/coding-promptsv3.md         the 30 V3 coding prompts (61 to 90), run in order
    ├── v3/v3changelog.md              record of every change made while building V3
    ├── v3/experiments.md              outcome of every experiment V3 runs (TE-n, E-n) and the precision gate
    ├── v3/testvideo_scripts/script1.md   founder's recording script: phone, portrait, all four event kinds
    ├── v3/testvideo_scripts/script2.md   founder's recording script: laptop webcam, the exclusion traps
    ├── v3/testvideo_scripts/script3.md   founder's recording script: noisy room and a long pause (voice chain)
    ├── v4/v4implementation.md         file-by-file implementation plan for V4 (M1.4, M1.5 and the seams S-1 to S-7 of §40.9)
    ├── v4/v4buildguide.md             step-by-step build order for V4, with a milestone per phase and three human gates
    ├── v4/coding-promptsv4.md         the 38 V4 coding prompts (91 to 128), run in order
    ├── v4/v4changelog.md              record of every change made while building V4 (opened by the v1.2 spec revision)
    ├── v4/experiments.md              outcome of every reading V4 takes (v4implementation §21) and BP §6.2's two decisions
    ├── v5/v5implementation.md         plan for V5 (M1.6, the seams S-8 and S-9, the speed rules)
    ├── v5/v5buildguide.md             step-by-step build guide for V5: phases, steps, milestones
    ├── v5/coding-promptsv5.md         the V5 coding prompts (Setup, 129 to 136), run in order
    ├── v5/v5changelog.md              record of every change made while building V5
    ├── v6/v6implementation.md         plan for V6 (M1.7, M1.8, the seam S-10, DC-11, DC-12)
    ├── v6/v6buildguide.md             step-by-step build guide for V6: phases, steps, milestones
    ├── v6/coding-promptsv6.md         the V6 coding prompts (Setup, 137 to 150), run in order
    ├── v6/v6changelog.md              record of every change while building V6 and V6.1-V6.11; Entry 0 is the v1.3 revision
    ├── v6/experiments.md              outcome of TE-8 and TE-9 (v6implementation section 17), from Prompt 149
    ├── cloud_transcription/cloudimplementation.md  plan for the cloud transcription release (D-336 to D-381, DC-14)
    ├── cloud_transcription/cloudbuildguide.md      step-by-step build guide for the cloud transcription release
    ├── cloud_transcription/coding-promptscloud.md  its coding prompts (Setup, 161 to 170), run in order
    ├── cloud_transcription/cloudchangelog.md       record of every change while building it
    ├── cloud_transcription/experiments.md          outcome of TE-C1 to TE-C5 (cloudimplementation section 13)
    ├── v6.1/v6.1implementation.md      plan for V6.1, long source (D-383 to D-432; DC-1 to DC-4); its changelog is v6/v6changelog.md
    ├── v6.1/v6.1buildguide.md          step-by-step build guide for V6.1: phases, commands, tiers, checks
    ├── v6.1/coding-promptsv6.1.md      the V6.1 coding prompts (Setup, 171 to 182), run in order
    ├── v6.1/overnight-run.md           the unattended session's master prompt for V6.1: authorisations, safe paths, merge gates
    ├── v6.1/experiments.md             outcome of TE-17, TE-18, TE-20, TE-21 and the TE-12 re-read (v6.1implementation section 13)
    ├── v6.2/v6.2implementation.md      plan for V6.2, AI edit plan (D-433 to D-499; DC-7, DC-8's V6.2 part, DC-16); its changelog is v6/v6changelog.md
    ├── v6.2/v6.2buildguide.md          step-by-step build guide for V6.2: phases, commands, tiers, checks, the founder's checkpoints
    ├── v6.2/coding-promptsv6.2.md      the V6.2 coding prompts (Setup, 183 to 212), run by hand one at a time
    ├── v6.2/experiments.md             outcome of TE-15, TE-30, TE-31 and E-11 (v6.2implementation section 15)
    ├── v5/experiments.md              outcome of every reading V5 takes (v5implementation section 20), from Prompt 136
    └── file-specs/TEMPLATE.md         per-file spec template (§34)
```

---

## 6. Module Responsibilities

| Module | Responsible for | Not responsible for |
|---|---|---|
| `offcut-types` | Unit types, ids, limits, all cross-boundary domain types | Any algorithm; any I/O |
| `offcut-mp4` | Parse MP4/MOV, expose samples and codec configs, probe, validate PS §9.4 limits, write faststart MP4 | Decoding or encoding samples; user-facing text |
| `offcut-text` | Tokenize, segment sentences, parse and format numbers, produce `NormalizedSpan`s | Deciding what becomes a visual event |
| `offcut-detect` | Turn transcript + prosody into `DetectedEvent`s above threshold; stable ids; per-sentence re-detection | Layout, rendering |
| `offcut-dsp` | Resampling, voice enhancement chain, loudness, prosody | Decode, encode, playback; changing the length or timing of the audio (INV-10) |
| `offcut-scene` | Styles, safe areas, caption chunking, event layout, animation, framing rectangle, watermark, `DisplayList` per frame, change summary | GPU, pixels, video decode |
| `offcut-render` | Draw a source frame plus a `DisplayList` to a target texture | Knowing what the scene means; encode |
| `offcut-entitlement` | Token encode/decode/verify; `export_profile()`; signing (server feature) | Counting exports; network; storage |
| `offcut-api-types` | DTOs and the analytics allowlist | Behaviour |
| `offcut-wasm-core`, `offcut-wasm-render` | JS bindings, argument conversion, panic hook | Rules |
| `server/auth` | Magic link issue/verify, refresh rotation, access tokens | Passwords, OAuth (P1), user profile |
| `server/entitlement` | Compute plan and remaining free exports; sign token | Enforcing limits (client-side conversion mechanism, PS §11) |
| `server/usage` | Idempotent receipt insert; return fresh token | Blocking exports; receiving anything but `export_id` |
| `server/billing` | Checkout and portal URLs; webhook verify, dedupe, apply | Prices, tax, card data, invoices |
| `server/analytics` | Validate against allowlist, insert, purge at 90 days | Joining events to users |
| `server/account` | Delete, export, waitlist capture | Deleting merchant billing records (PS §17) |
| `web/usecases` | Sequencing of one user action end to end | Rendering UI; implementing algorithms |
| `web/state` | Truth for UI state; legal transitions; blockers | Side effects |
| `web/workers` | Browser media APIs per stage; progress; cancellation | Rules; persistence metadata; network |
| `web/models` | Get verified model files into OPFS | Running the model |
| `web/persistence` | Durable local state and its schema | Deciding when to persist |
| `web/net` | HTTP with timeout, retry, cold-start handling | Knowing what the data means |
| `web/analytics` | Typed, batched, allowlisted events | Any property not in the generated type |
| `verify/` | Independent checks of exported MP4s | Sharing code or libraries with `crates/` |

---

## 7. Dependency Graph

**Rust crates (arrows point to what may be imported)**

```text
offcut-wasm-core ──► offcut-mp4, offcut-text, offcut-dsp ──────────────┐
offcut-wasm-render ─► offcut-render ─► offcut-scene ─┐                 │
        │            offcut-detect ─► offcut-text ───┤                 ▼
        ├──────────► offcut-mp4 ─────────────────────┼──────────► offcut-types
        ├──────────► offcut-plan ─► offcut-text ─────┤                 ▲
        └──────────► offcut-entitlement ─────────────┘                 │
offcut-api (server) ─► offcut-api-types, offcut-entitlement[sign] ─────┘
```

`offcut-api` also depends on `offcut-plan` with its `prompt` feature from V6.2, for the catalog version, the prompt and the schema (§40.7; v6.2implementation D-458); `offcut-plan` is a wrapper of the `offcut-text` ban in `deny.toml`, so the server still reaches no media crate on its own.

`offcut-scene` also depends on `offcut-text`, for `format_quantity` only (count-up values, §19.5). `check-file-tree.mjs` allows that pair, and the `offcut-text` ban in `deny.toml` lists `offcut-scene` as a wrapper (v2implementation D-28). It creates no cycle.

`offcut-plan` (V4, v4implementation D-122) depends on `offcut-text` and `offcut-types`. `offcut-scene` depends on `offcut-plan` (it reads a `ValidPlan`), and `offcut-wasm-render` depends on it with `default-features = false` (no schema code in the render bundle). Two edges are dev-only, for the labeled-set tests (v4implementation §3.2): `offcut-scene → offcut-detect` and `offcut-plan → offcut-detect`; three more, for the golden frames' scenes (v4implementation §9.4, V4 Prompt 107): `offcut-render → offcut-text`, `offcut-render → offcut-detect` and `offcut-render → offcut-plan`. `check-file-tree.mjs` counts dev-dependencies, so its table allows all of them; the `offcut-text` and `offcut-detect` bans in `deny.toml` list `offcut-plan` and `offcut-render` as wrappers. None creates a cycle: `offcut-detect` and `offcut-text` depend on no crate above them.

**Web (arrows point to what may be imported)**

```text
ui ─► usecases ─► { state, workers/pool, persistence, models, net/api-client,
                    analytics/client, entitlement }
ui ─► state (read/subscribe), copy, config
models ─► net/asset-fetch, persistence/opfs, wasm/load-core
net/api-client ─► net/http ─► config/env, config/allowlist-hosts
workers/*.worker ─► workers/protocol, workers/rpc, wasm/load-*, gen
everything ─► gen (types only)
```

**Forbidden**

| Rule | Enforced by |
|---|---|
| Pure crates (`offcut-types`, `-mp4`, `-text`, `-detect`, `-dsp`, `-scene`, `-plan`) depend on `web-sys`, `js-sys`, `wasm-bindgen` or `wgpu` | `cargo-deny` `[bans]` per-crate wrappers in `deny.toml`; CI step `cargo deny check` |
| Pure crates read a clock or randomness (`std::time`, `rand`, `getrandom`) | `clippy.toml` `disallowed_methods` / `disallowed_types`; CI `cargo clippy -D warnings` |
| `server` depends on `offcut-mp4`, `-text`, `-detect`, `-dsp`, `-scene`, `-render` | `cargo-deny` ban on those crates for package `offcut-api`; `server/tests/no_media_routes.rs` |
| A crate imports a crate above it in the diagram | Cargo rejects cycles; `scripts/check-file-tree.mjs` checks declared dependencies against this section |
| `ui` imports `workers`, `net`, `persistence`, `wasm`, `models`, `analytics` | `eslint-plugin-boundaries` element rules in `web/eslint.config.js` |
| `state` imports anything except `gen` | same |
| `fetch`, `XMLHttpRequest`, `WebSocket`, `EventSource`, `navigator.sendBeacon`, `RTCPeerConnection` outside `net/http.ts`, `net/asset-fetch.ts`, `wasm/load-core.ts`, `wasm/load-render.ts`, and from V6.8 `net/stock-fetch.ts` (v1.4, §24.1 row 4) | ESLint `no-restricted-globals` / `no-restricted-properties` with per-file overrides; runtime backstop: CSP `connect-src` (§24.3); proof: `privacy-network.spec.ts` |
| `import.meta.env` outside `config/env.ts` | ESLint `no-restricted-syntax` |
| String literals of user-facing sentences outside `copy/messages.ts` | ESLint `react/jsx-no-literals` (allowing punctuation) in `ui/` |
| Editing `web/src/gen/*` by hand | `scripts/check-gen-clean.sh` regenerates and fails on diff |
| `unwrap`, `expect`, `panic!`, indexing by `[]` on slices in non-test Rust | clippy `unwrap_used`, `expect_used`, `panic`, `indexing_slicing` = deny |
| `any`, non-null `!`, `@ts-ignore` in `web/src` | `@typescript-eslint/no-explicit-any`, `no-non-null-assertion`, `ban-ts-comment` |
| A source file not listed in §5 | `scripts/check-file-tree.mjs` |

**Circularity guards**
- `offcut-types` imports no `offcut-*` crate. New shared types go there, never into a sibling.
- `workers/protocol.ts` imports only `gen`; workers and `pool.ts` both import it, never each other.
- `usecases` never import each other except `cancel-job.ts` and `restore-clip.ts`, which other use-cases may call, and the two pairs of §2 "Edges added in V2" (`import-clip.ts` to `run-pipeline.ts`, `start-export.ts` to `control-preview.ts`); checked by a `boundaries` rule.
- Stores never import use-cases; use-cases write stores through exported action functions only.
- **Edges added in V6.** `start-app.ts` to `restore-session.ts`: `connectAccount()` runs first in `start-app.ts` and hands `api-client.ts` its `connectAuth` links and `receipt-outbox.ts` its `connectOutbox` links, so `net` and `persistence` import no store (v6implementation D-306). `blockers.ts` to `entitlement.ts`, as §12.5 asks `effective()` (D-309). `offcut-api` uses `getrandom` directly, as a wrapper in `deny.toml`, for the 32 random bytes of a link and a refresh token (D-257).
- **Edges added in the cloud transcription release.** `run-pipeline.ts` to `transcribe-cloud.ts` (cloudimplementation D-372): the pipeline hands the clip's sound to the one use-case that sends it.
- **Edges added in V6.1 (as built, the V6.1 plan §4.1).** `run-source-pipeline.ts` to `transcribe-source.ts`, the second use-case that sends audio (a long source's windows); `import-source.ts` and `continue-source.ts` to `run-source-pipeline.ts`; `import-clip.ts` to `import-source.ts` (the hand-over of a too-long clip's `File`, D-421); `start-app.ts` to `continue-source.ts` (D-419). The UI may name `transcribe-source.ts`' `windowCount` (the long page's progress).
- **Edges added in V6.2 (V6.2, as built: the V6.2 plan §4.1, D-458, D-487).** `run-pipeline.ts` to `choose-plan.ts`, which asks through `request-plan.ts`, the one use-case that sends transcript text (P-1, P-14); `regenerate-plan.ts` to `request-plan.ts` and to `review-update.ts` ("Regenerate plan" goes through the update rule's queue); `set-enhance.ts` to `review-update.ts` (a switch is a review change, D-488); `run-pipeline.ts` to `set-enhance.ts` (a clip starts on the remembered Enhance); `review-update.ts`, `run-pipeline.ts` and `start-export.ts` to `voice-program.ts` (the program on a voice, made in one place). `offcut-api` depends on `offcut-plan` with its `prompt` feature (as planned above); `offcut-render` gains `png` as a normal dependency, to decode the embedded image bundle (P196).

---

## 8. Runtime Data Flow

```text
File dropped (or sample clip)                    usecases/import-clip.ts::importClip
  → copy to OPFS clips/<clipId>/source           workers/media/import.ts::importToOpfs
  → ProbeInfo                                    offcut_mp4::probe::probe
  → ClipInfo | RejectReason                      offcut_mp4::validate::validate_probe
  → PCM 48 kHz mono + PCM 16 kHz mono            workers/media/audio-decode.ts::decodeAudio
                                                   + offcut_dsp::resample::resample_mono
  ╔═ in parallel ════════════════════════════════════════════════════════════════════
  ║ → raw words with timestamps                  workers/asr/whisper-runtime.ts::transcribe
  ║     → Transcript (sentences, numbers)        offcut_text::normalize::normalize_transcript
  ║ → out48 (enhanced audio, same length)        offcut_dsp::chain::run_chain
  ╚══════════════════════════════════════════════════════════════════════════════════
  → Prosody (per-word energy and pitch z)        offcut_dsp::prosody::measure_prosody
  → DetectedEvent[] (above threshold only)       offcut_detect::detect
  → Scene                                        offcut_scene::build_scene
  → preview frames 540x960 + audio               workers/render/preview-loop.ts::runPreview
  → [review edits] → re-detect / rebuild scene   offcut_detect::redetect_sentence, build_scene
  → export gate (blockers, entitlement)          usecases/start-export.ts::startExport
  → frames at ExportProfile size, 30 fps         workers/render/export-loop.ts::runExport
  → H.264 + AAC chunks                           workers/render/encoders.ts
  → MP4 in OPFS exports/<exportId>.mp4           offcut_mp4::mux::Mp4Muxer
  → browser download + ChangeSummary             usecases/start-export.ts::deliverExport
  → usage receipt (export_id only)               persistence/receipt-outbox.ts::flushOutbox
```

Neither parallel branch changes the timeline. ASR produces timestamps on the recording's timeline; the audio branch produces `out48` with the same number of samples as its input. Every later stage (prosody, detection, scene, preview, export) reads that one timeline (INV-5, INV-10).

| Transition | Where it runs | Memory or cost | Failure |
|---|---|---|---|
| File → OPFS copy | `media.worker` | Streamed in 4 MiB chunks; disk up to 500 MB (PS §9.4) | `E_STORAGE_QUOTA`, `E_STORAGE_IO` |
| OPFS → `ProbeInfo` | `media.worker` (Rust) | Reads `moov` only; under 5 MB RAM (assumption, TE-12) | `REJECT_CONTAINER`, `REJECT_CORRUPT` |
| `ProbeInfo` → `ClipInfo` | `media.worker` (Rust) | Negligible | any `REJECT_*` (§15.3) |
| AAC → PCM | `media.worker` | 48 kHz mono f32: 90 s x 48,000 x 4 B = 17.3 MB; 16 kHz: 5.8 MB (derived) | `E_DECODE_AUDIO` |
| PCM16 → words | `asr.worker` | Model session; budget 700 MB (assumption, TE-14); 20 s on R1 (PS §20.2) | `E_ASR_RUNTIME`, `E_ASR_OOM`, `REJECT_NO_SPEECH` |
| PCM48 → out48 | `audio.worker` | 2 x 17.3 MB; 4 s on R1 (PS §20.2) | `E_DSP` |
| words + out48 → `Prosody` | `audio.worker` | No new PCM buffer; under 1 s (assumption, M1.2) | `E_DSP` |
| transcript → events → `Scene` | `render.worker` (Rust) | Under 10 MB; 2 s (PS §20.2) | `E_INTERNAL` |
| Scene → preview frames | `render.worker` | Frame queue of 6 decoded frames; GPU textures (§31) | `E_DECODE_VIDEO`, `E_GPU_INIT`, `E_GPU_LOST` |
| Scene → encoded chunks → MP4 | `render.worker` | Encoder queue of 4 frames; output streamed to OPFS, at most 91.8 MB (derived, §21.2); 90 s + 2 s on R1 (PS §20.2) | `E_ENCODE_VIDEO`, `E_ENCODE_AUDIO`, `E_MUX`, `E_GPU_LOST`, `E_STORAGE_QUOTA` |
| MP4 → download | main thread | Blob URL to an OPFS `File`; no RAM copy | `E_STORAGE_IO` |
| receipt → server | main thread | One request under 200 B; retried from the outbox | `E_NET_OFFLINE`, `E_NET_TIMEOUT`, `E_API_5XX` (never blocks the download) |

---

## 9. Function Call Chains

Notation: `→` call, `⇒` state transition, `✓`/`✗` success/failure branch, `[evt]` analytics event (§22.6). All use-cases live in `web/src/usecases/`; worker methods are defined in `workers/protocol.ts` (§14.2).

**C-1. App start and capability check (J1, J3)**
```text
main.tsx → startApp()
  → analytics.track("landing_view")                                         [evt landing_view]
  → runCapabilityCheck()                    ⇒ capability: unchecked → checking
      ✓ → capabilityStore.setSupported()    ⇒ checking → supported          [evt capability_check result=pass]
      ✗ → capabilityStore.setUnsupported(reason)
                                            ⇒ checking → unsupported         [evt capability_check result=fail]
          → routes: <CapabilityGate> renders <UnsupportedPage reason>
  → apiClient.wake()                        fire-and-forget GET /api/v1/healthz (starts a cold API early)
  → restoreSession()                        POST /api/v1/auth/refresh
      ✓ ⇒ auth: unknown → signed_in; entitlementStore.setToken(token)
      ✗ 401 ⇒ auth: unknown → anonymous
      ✗ network ⇒ auth: unknown → anonymous_offline; entitlement loaded from entitlement-repo
  → modelManager.inspect()                  ⇒ model: unknown → absent | partial | ready
  → restoreClip()                           (C-13)
```

As built in V6 (v6implementation D-289, D-296, D-306, D-308): `connectAccount()` from `restore-session.ts` runs first, before anything else, as the callback page may verify when it mounts; `restoreSession()` starts at `start-app.ts`'s marker and nothing waits for it (INV-3). `main.tsx` tells `startApp` whether to restore: on `/auth/callback` it does not. Refresh runs after `ready()`, under `navigator.locks`, with a 25 s budget (D-297).

**C-2. Drop a clip and validate (J2, J5)**
```text
<DropZone onDrop> → importClip(file, source)
  → blockers.forImport()                    ✗ B_UNSUPPORTED | B_PIPELINE_BUSY → <ErrorPanel>, stop
  → quota.ensureFree(file.size)             ✗ → evict LRU clips (§23.3); still short → E_STORAGE_QUOTA
  → clipStore.begin(clipId)                 ⇒ clip: idle → importing
  → pool.media.importAndProbe({clipId, file})
      → importToOpfs() → open_demuxer() → probe() → validate_probe()
      ✓ ClipInfo                            ⇒ importing → accepted         [evt clip_accepted]
          → clipsRepo.put(clipRef) → runPipeline(clipId)                    (C-3, C-4)
      ✗ RejectReason                        ⇒ importing → rejected         [evt clip_rejected]
          → opfs.removeClip(clipId) → <RejectionPanel code> (copy from messages.ts)
      ✗ error                               ⇒ importing → failed           [evt client_error]
```

**C-2 as built in V3.** Before `importAndProbe`, until V7's eviction exists, the previous clip's `clips/` directory and the `clips`, `transcripts` and `edits` records are removed (`clear()` on each repo), so the device holds the records of one clip at most (v3implementation §23 item 26). `sourceHashPrefix` is computed on the main thread before the file is posted (`clipsRepo.hashPrefix(file)`, §23.1, v3implementation D-81). `clipsRepo.put` runs while the clip is still `importing`, and `clip_accepted` follows it; a storage failure there fails the clip with no `clip_accepted`. `clip_rejected` carries `reject_reason` alone, sent here for the 13 codes `importAndProbe` returns and from `run-pipeline.ts` for `REJECT_NO_SPEECH`; `client_error` is not sent before V4 (v3implementation D-97).

**C-3. First-run model download (J4)**
```text
runPipeline(clipId) → modelManager.ensureReady(onProgress)
  model ready      → return
  model absent|partial ⇒ model: → downloading          <ModelDownloadPanel>
    → download.fetchRanged(file, resumeFrom)  (asset-fetch.ts, HTTP Range, 8 MiB parts)
        → opfs append → Sha256Stream.update()
    ⇒ downloading → verifying
        ✓ sha256 == manifest ⇒ verifying → ready        [evt model_download outcome=ok]
        ✗ mismatch → delete file ⇒ verifying → failed(E_MODEL_HASH)   [evt model_download outcome=hash_mismatch]
    ✗ network → keep partial ⇒ downloading → partial; auto-retry x3 with backoff, then failed(E_MODEL_DOWNLOAD)
    ✗ quota ⇒ failed(E_MODEL_STORAGE)
  meanwhile: clip stays in state waiting_model; audio decode and voice chain proceed (C-4 step A)
```

**C-4. Processing with live feed (J6)**
```text
runPipeline(clipId)                           ⇒ clip: accepted → processing(stage)
  A  pool.media.extractAudio({clipId})        → {pcm48, pcm16} (transferred)    [evt stage_timing stage=probe_audio]
  B  in parallel:
     B1 pool.asr.transcribe({pcm16})          feed: "Transcribing…"
          → whisper-runtime.transcribe() → normalize_transcript()
          ✓ Transcript (words >= MIN_WORDS)                                     [evt stage_timing stage=asr]
          ✗ words < MIN_WORDS ⇒ rejected(REJECT_NO_SPEECH)                      [evt clip_rejected]
     B2 pool.audio.runChain({pcm48})          feed: "Cleaning voice"            [evt stage_timing stage=audio_chain]
  C  pool.asr.unload()                        frees the model session before rendering (§31)
  D  pool.audio.measureProsody({words, sentences}) → measure_prosody()
          ✓ {out48, prosody}                  out48 is the chain output from B2, transferred; nothing is cut or spliced
  E  pool.render.openSession({clipId, clipInfo})
     pool.render.detect({transcript, prosody}) → detect()                       [evt stage_timing stage=detect_scene]
          feed: one line per event from messages.ts ("Found: 3-item list")
     pool.render.setScene({transcript, events, edit, profile: preview})
  F  transcriptRepo.put, editsRepo.put, opfs.writeAudio(out48)
                                              ⇒ processing → ready              [evt pipeline_done]
  any ✗ ⇒ processing → failed(code)           [evt client_error]  → <ErrorPanel> with retry (C-14)
```

**The C-4 order as built** (v3implementation D-90). After step A, `pool.audio.runChain({ pcm48 })` starts at once, in parallel with the model wait (C-3) and with ASR (B1); its feed line "Cleaning voice" is pushed when it starts. `stage_timing audio_chain` is measured on the main thread around the `runChain` call (v2implementation D-35). Step D (`measureProsody`) waits for both B1 and B2. A failure of B2 while B1 runs is held until B1 has settled and `asr.unload()` has run (INV-12); then the clip fails with the first failure in time order. A `REJECT_NO_SPEECH` from B1 waits for B2 to settle, then the clip is rejected; `measureProsody` is not called and the worker is sent nothing, so its `out48` stays in the worker's memory until its next `runChain` replaces it (v3implementation §23 item 25, open until V4). The feed's first two lines are therefore "Cleaning voice" and "Transcribing…", in either order.

**C-5. Preview and AHA (J7)**
```text
<PreviewPlayer mount> → controlPreview.attach(canvas.transferControlToOffscreen(), out48)
  → pool.render.attachPreview({canvas})       AudioBuffer built from out48 on the main thread
<Play> → controlPreview.play()
  → audioSource.start(when, offset)           ⇒ preview: stopped|paused → playing
  → pool.render.previewPlay({clock})          clock = {audioMs, epochMs} re-sent every 250 ms
      → preview-loop.runPreview(): per rAF: t = clock.now()
          → video-source.frameAt(t) → session.render_frame(frame, t)      same t for audio, video and scene
  first play only                             [evt preview_played]
no item in the valid plan → the Visuals tab (PlanItemList) shows messages.visuals.noEventsFound (PS §10 J7; V4 D-154)
<Pause>/<Seek> → controlPreview.pause() / seek(TimeMs) ⇒ playing → paused / → seeking → paused|playing
```

**C-6. Review edits (J8)**
```text
As built in V4 every change goes through one update rule (usecases/review-update.ts, D-184): the clip ready →
updating, the preview paused, the worker's calls, the scene set, then ready, the change written and pushed on the
undo history (50 steps); a failure leaves the edit before it and offers "Undo this change".
<CaptionEditor> (click a word, Enter or Tab commits) → editWord(wordIdx, text)
  → pool.render.redetectSentence({sentenceIdx, wordEdits}) → rulesPlan → setScene(plan, ...)
      → events for that sentence replaced; toggles kept by EventId               [evt review_action action=word_edit]
  → the "Change all" toast → replaceAll(from, to) → detect({wordEdits}) → ...   [evt review_action action=replace_all]
<PlanItemList> (Visuals tab) switch → toggleEvent(id, on) → setScene           [evt review_action action=event_toggle]
<StylePicker> (Style tab cards) → setStyle(styleId) → setScene                  [evt review_action action=style]
<Highlight numbers> → setHighlightNumbers(on) → setScene                        [evt review_action action=highlight_numbers]
<FramingPanel> / caption drag on the preview → setCaptionPosition(p) → setScene [evt review_action action=caption_position]
<CropOffsetControl> / video drag (wider than 9:16 only) → setCropOffset → setScene [evt review_action action=crop_offset]
Ctrl/⌘+Z, Shift+Ctrl/⌘+Z → undo / redo → setScene with the step's edit          [evt review_action action=undo|redo]
all: preview ⇒ playing → paused during the update, then restored
no edit changes the audio, the clip duration or any timestamp
```

**C-7. Export click and gate (J9)**
```text
<ExportButton> shows messages.exportCounter(remaining) from entitlement.effective()   (before the click)
<ExportButton onClick> → startExport(clipId)
  → blockers.forExport()
      B_PIPELINE_NOT_READY | B_EXPORT_IN_PROGRESS → the button is disabled, its tooltip says why, stop
      B_NOT_SIGNED_IN       → <SignInDialog> with Free/Creator limits copy (C-8), stop
      B_ENTITLEMENT_EXPIRED → apiClient.getEntitlement()  ✓ continue  ✗ <ErrorPanel E_ENTITLEMENT_EXPIRED>
      B_NO_FREE_EXPORTS     → <UpgradePrompt placement=limit_reached>           [evt upgrade_prompt]
  remaining == 1 on Free → confirm dialog messages.LAST_FREE_EXPORT (PS §11) before proceeding
  → C-9
```

As built in V6 (v6implementation D-290, D-294, D-309, D-310, D-314): the three account arms open a dialog from a button that stays enabled; `B_ENTITLEMENT_EXPIRED` that cannot be renewed is `gateFailure`, shown by the export-failed card with no `export_started`; the last Free export's question is a dialog of its own; the token is reloaded from `entitlement-repo` first. While the sign-in or last-export dialog is open, the editor's shortcuts stand aside (D-311).

As built in V5 (v5implementation §13, D-229, D-231): the sign-in and limit arms are V6's. `B_STORAGE_LOW` is evaluated inside C-9, after a cache miss, by `forExport({ freeBytes, neededBytes })`, since `navigator.storage.estimate()` is asynchronous and the button reads `forExport()` while drawing. A blocked export returns to `idle` and says why in the `export-status` line beside the button, with nothing swept: it is not an `ErrorPanel`, as nothing failed.

**C-8. Magic-link sign-in (J9)**
```text
<SignInDialog onSubmit(email)> → requestMagicLink(email)
  → apiClient.requestMagicLink({email})       POST /api/v1/auth/magic-link → 202
                                              ⇒ auth: anonymous → link_sent    [evt signin_step step=link_requested]
  ✗ E_API_RATE_LIMITED | E_AUTH_EMAIL_UNAVAILABLE | E_NET_* → message in dialog; state unchanged
user opens the emailed link (new tab): /auth/callback?token=…
<AuthCallbackPage mount> → completeSignIn(token)
  → apiClient.verifyMagicLink({token})        POST /api/v1/auth/verify
      ✓ sets refresh cookie; returns SessionResponse
          ⇒ auth: → signed_in; entitlementStore.setToken; entitlementRepo.put   [evt signin_step step=completed]
          → broadcast.post("signed_in")
          → if this tab has no clip: show messages.SIGNED_IN_RETURN_TO_TAB and a link to /app
      ✗ E_AUTH_LINK_EXPIRED | E_AUTH_LINK_INVALID → message + "send a new link"
original tab: broadcast.on("signed_in") → restoreSession() ⇒ link_sent → signed_in → dialog closes; user clicks Export again
original tab closed: /app in the new tab → restoreClip() (C-13)
```

As built in V6 (v6implementation D-254, D-259, D-295): the magic-link request and the verify are one attempt each, after `ready()`, since a link is single-use and a retried verify would read a good link as used. A relay failure is 503 `email_unavailable` → `E_AUTH_EMAIL_UNAVAILABLE` in the dialog, not a waking API (D-287). The dialog says to open the link on this computer (D-302).

**C-9. Render and export (J10)**
```text
startExport (after gate; as built in V5, v5implementation §13)
  → exportId = newExportId()                  UUIDv7; ⇒ export: idle → gating
  → values = editValues(clip)                 what export_done reports, read now: the person may keep editing
  → controlPreview.lockForExport()            before the worker is called: it takes one job at a time (D-221)
  → pool.render.exportProfile({entitlementToken})   verify signature → ExportProfile; refused ⇒ gating → rendering → failed (D-230)
  → key = cacheKey(ten inputs, profile.kind)  no key while the clip is not `ready`, or its record cannot be read: no cache
  → renderCacheRepo.lookup(key)               ✓ hit: ⇒ gating → saving → done, the stored file under today's name and its original
                                                 exportId; unlock  [evt export_done from_cache=true]; no export_started, no stage_timing
  → forExport({freeBytes + heldBytes, neededBytes})   B_STORAGE_LOW ⇒ gating → idle, says why, nothing swept
  → exportStore.clear()                       ⇒ gating → rendering             [evt export_started]
  → renderCacheRepo.clear(); opfs.remove(exports)     the row, then the file (§23.3)
  → prog48 = opfs.readProgram(clipId)
  → pool.render.exportClip({exportId, entitlementToken, prog48})
      → export_profile_from_token(token)      verified again: the worker trusts nothing the page says of a profile
      → setScene(profile) → encoders.supportedConfigs(profile)
      → export-loop.runExport(): for each supported entry, attempt: for n in 0..frames: scene.source_at(t) → source frame →
        render_frame → VideoFrame → encode → mux
        E_ENCODE_VIDEO: sink.reset(), next entry from frame 0; E_ENCODE_AUDIO: once more on the same entry
      → AAC encode of prog48 → mux            ⇒ rendering → muxing
      → Mp4Muxer.finalize() → opfs-sink.close()
      ✓ {opfsPath, summary, stageTimings}     ⇒ muxing → saving                [evt stage_timing stage=render_encode, mux]
      ✗ ⇒ failed(code)                        [evt export_failed] → <ErrorPanel> (C-14)
  progress events → <ExportProgress> (stage + elapsed; copy RENDERING_LOCALLY)
```

**C-10. Download, summary, receipt (J11)**
```text
deliverExport(exportId, opfsPath, summary)
  → file = opfs.getFile(opfsPath) → anchor download "offcut-<yyyymmdd-hhmm>.mp4"
  → renderCacheRepo.put(cacheKey, exportId, opfsPath)
  → receiptOutbox.enqueue(exportId)           (skipped when the file came from the render cache)
                                              ⇒ export: saving → done          [evt export_done]  ← activation event (PS §10)
  → <ExportOutcome>: summaryRows(summary) + <PostExportQuestion>   (as built in V5; the question once per exportId, D-234)
  → receiptOutbox.flushOutbox()
      → apiClient.postUsageReceipt({export_id})  ✓ new token → entitlementStore.setToken; outbox.remove
                                                 ✗ kept; retried with backoff and on next app start
<PostExportQuestion onAnswer(a)> → answerPostExport(a)                         [evt post_export_answer]
```

**C-11. Upgrade and checkout (J12)**
```text
after a watermarked export: <UpgradePrompt placement=after_export> (once per export, dismissible)  [evt upgrade_prompt action=shown]
<UpgradePrompt onUpgrade(offer)> → startCheckout(offer)                        [evt checkout_step step=started]
  → apiClient.createCheckout({offer})         POST /api/v1/billing/checkout → {url}
      ✗ E_BILLING_UNAVAILABLE → message, stop
  → window.location.assign(url)               hosted checkout (full navigation; no third-party script)
merchant → browser: /account?checkout=success | cancel
<AccountPage mount> → confirmCheckout(result)                                  [evt checkout_step step=returned_success|returned_cancel]
  success → poll apiClient.getMe() every 2 s, up to 60 s, until plan == "creator"
      ✓ entitlementStore.setToken → messages.UPGRADE_ACTIVE
      ✗ timeout → messages.E_BILLING_PENDING (retry button; webhook may still arrive)
<AccountPage "Manage subscription"> → openBillingPortal() → GET /api/v1/billing/portal → navigate
```

As built in V6 (v6implementation D-312 to D-315): the prompt comes once per watermarked export, until dismissed; "Make another clip" is in `after_export` only. `startCheckout` needs a session (`forCheckout`), sends the analytics queue before leaving (one request, not awaited), and on D-300's 403 opens the portal instead. The account page offers the two plans only on Free with no live subscription, and "Manage subscription" whenever a subscription row exists. The top bar's "Account" opens `/account` in a tab of its own, as leaving `/app` would let go of the clip until V7's `restoreClip`. The wait for Creator reads again after a failed read and stops when the session has ended.

**C-12. Cancel**
```text
<Cancel> (during import, processing, model download or export) → cancelJob(scope)
  → pool.cancel(jobId)                        posts {type:"cancel", jobId} to the owning worker
      worker: sets flag; current step returns Cancelled at its next check (§14.3); releases frames, closes encoders
  → export: partial OPFS output deleted; no outbox entry; no render-cache entry
  ⇒ clip: processing → accepted (re-runnable) | export: rendering|muxing → cancelled → idle    [evt job_cancelled]
  model download: ⇒ downloading → partial (bytes kept for resume)
  worker does not acknowledge within CANCEL_TIMEOUT → pool.restartWorker(kind) (§14.4)
```

**C-13. Restore after refresh or crash**
```text
startApp → restoreClip()
  → clipsRepo.latest()                        none → idle
  → opfs.exists(source) && transcriptRepo.get(clipId) && opfs.existsAudio(clipId)
      ✓ all present → pool.render.openSession → setScene(saved edit, events)
                                              ⇒ clip: idle → ready (no re-transcription)
      partial (source only) → runPipeline(clipId)
      source missing → clipsRepo.delete(clipId) → idle
  an export that was in flight is not resumed: its temp file is deleted (opfs.sweepTemp)
```

**C-14. Failure and recovery**
```text
worker "error"/"messageerror" or WASM panic → rpc rejects all pending calls with E_WORKER_CRASH
  → pool.restartWorker(kind) → state ⇒ failed(E_WORKER_CRASH)                 [evt client_error]
GPU device lost (device.lost promise) → E_GPU_LOST → session closed
  → <ErrorPanel> retry → pool.render.openSession (new device) → setScene → resume at ready
<ErrorPanel onRetry> → runPipeline(clipId) or startExport(clipId) from the last completed stage
second failure with the same code → messages.<code>.persistent (suggests smaller clip / closing tabs / "report this")
<ErrorPanel "Report this"> → analytics.track("client_error", {error_code, stage})   (allowlisted enums only)
```

**C-15. API cold start or offline**
```text
http.request(route, opts)
  attempt 1 timeout = API_TIMEOUT (10 s) ✗ → retry with backoff 1 s, 2 s, 4 s, 8 s while total <= API_COLD_START_BUDGET (70 s)
      UI: after 3 s shows messages.API_WAKING ("Connecting to the account service…")
  ✗ budget exhausted → E_NET_TIMEOUT;  navigator.onLine === false → E_NET_OFFLINE immediately
effects: import, processing, preview, review never call the API (INV-3)
         export with a valid cached token proceeds; receipt waits in the outbox
         sign-in, checkout, account pages show the error with retry
```

**C-16. Unsupported browser (J3)**
```text
<UnsupportedPage reason> renders messages.unsupported[reason], the supported list, the demo video (asset CDN), <NotifyMeForm>
<NotifyMeForm onSubmit(email, wanted)> → submitNotifyMe(email, wanted) → POST /api/v1/notify-me → 204
```

**C-17. Clear local data (PS §12.7)**
```text
<SettingsPage "Clear local data"> → clearLocalData()
  → blockers: B_PIPELINE_BUSY | B_EXPORT_IN_PROGRESS → refuse
  → pool.terminateAll() → opfs.removeAll() (clips, exports, models, temp) → db.clearStores(all except receiptOutbox, entitlement, meta)
  ⇒ clip: → idle; model: → absent                                            [evt local_data_cleared]
```

**C-18. Delete account / export account data (PS §17)**
```text
<AccountPage "Delete account"> → deleteAccount() → POST /api/v1/account/delete
  server: cancel subscription at provider → delete rows in one transaction (§22.7) → clear cookie
  ✓ ⇒ auth: signed_in → anonymous; entitlementRepo.clear
<AccountPage "Export my account data"> → exportAccountData() → GET /api/v1/account/export → JSON download
```

**C-19. Merchant webhook (server)**
```text
POST /api/v1/billing/webhook → billing::webhook::handle
  → adapter.verify_signature(headers, raw_body, now) ✗ 401
  → adapter.parse_event(headers, raw_body) → BillingEvent   unknown type → 200 (ignored)
  → tx: webhook_events.insert(provider_event_id)     ✗ unique violation → 200 (already processed; no state change)
        resolve the owner (D-275); none, or a deleted user → 200, nothing changed (D-272, D-284)
        subscriptions.apply(event)  (§22.5); a whole refund of a live subscription also cancels it at the provider
  → 200
```

---

## 10. Core Domain Types

All types below live in `crates/offcut-types` unless a path is given. They derive `Clone, Debug, PartialEq, Serialize, Deserialize, TS`; unit and id types also derive `Copy, Eq, Ord, Hash` and use `#[serde(transparent)]`. Inner fields of unit types are private; construction is through `new` and conversion functions in `units.rs` only.

### 10.1 Unit and id types

```rust
// units.rs  [ONLY]
pub struct TimeMs(u32);       // a position in milliseconds on the recording's (source) timeline
pub struct OutMs(u32);        // v1.2, V4: a position on the program (export) timeline; related to TimeMs only by TimeMap (§40.1)
pub struct DurMs(u32);        // a duration in milliseconds
pub struct Micros(i64);       // WebCodecs timestamps only
pub struct FrameIdx(u32);     // output frame number at OUTPUT_FPS
pub struct SampleCount(u64);  // PCM samples at a stated rate
pub struct Hz(u32);
pub struct FpsMilli(u32);     // frames per second x 1000 (59_940 = 59.94 fps)
pub struct Px(u32);
pub struct Bytes(u64);
pub struct BitsPerSec(u32);
pub struct Confidence(f32);   // invariant 0.0..=1.0, checked in new()
pub struct Lufs(f32);
pub struct Dbfs(f32);
pub struct UnixSecs(i64);
pub struct ExportCount(u32);
pub struct UsdCents(u32);

pub struct Span { pub start: TimeMs, pub end: TimeMs }    // invariant start <= end
pub struct OutSpan { pub start: OutMs, pub end: OutMs }  // V4: the same on the program timeline; made with OutSpan::new

impl FrameIdx { pub fn to_out_ms(self) -> OutMs; pub fn to_micros(self) -> Micros; }   // n * 1000 / 30, n * 1_000_000 / 30
                                                                                          // (as built in V4, D-125: to_out_ms replaces
                                                                                          // to_time_ms; saturates at u32::MAX ms)
pub struct SourceMs(u32);     // v1.3, V6.3: a position in a long source file; related to TimeMs only by Assembly (§44.1)
impl TimeMs   { pub fn to_micros(self) -> Micros; pub fn from_micros_floor(m: Micros) -> TimeMs; }
// No From<u32>. A position (TimeMs) and a length (DurMs) are distinct types. TimeMs and OutMs never convert
// except through TimeMap (§40.1); with the identity map they are numerically equal.

// ids.rs  [ONLY]
pub struct ClipId(Uuid);      // v4, created on import
pub struct ExportId(Uuid);    // v7, created per export; the usage-receipt idempotency key
pub struct UserId(Uuid);
pub struct AnonId(Uuid);      // per-browser analytics id
pub struct EventId(u64);      // stable hash, §17.6
pub struct WordIdx(u32);
pub struct SentenceIdx(u32);
pub struct WordRange { pub start: WordIdx, pub end: WordIdx }   // half-open
pub struct JobId(u32);
```

TypeScript mirrors these as branded types generated into `web/src/gen/domain.ts` (`type TimeMs = number & { readonly __unit: "TimeMs" }`). `as TimeMs` casts are allowed only in `gen/` and `workers/`, and in one minting line per file listed in the `MINTING` table of `web/eslint.config.js`: `persistence/opfs.ts`, `models/download.ts`, `models/model-manager.ts` and `state/model-store.ts` (`Bytes`), `usecases/import-clip.ts` (`ClipId`), `usecases/start-export.ts` (`ExportId`), `usecases/control-preview.ts` (`TimeMs`) (v2implementation D-59), and `persistence/clips-repo.ts` (`Bytes`, for `ClipRecord.sourceBytes`; v3implementation D-103); ESLint `no-restricted-syntax` rejects them elsewhere.

### 10.2 Limits (copied from the product spec)

```rust
// limits.rs  [ONLY]
pub const MAX_CLIP_DURATION: DurMs        = DurMs::new(90_000);          // PS §9.4
pub const MAX_FILE_SIZE: Bytes            = Bytes::new(500_000_000);     // PS §9.4, MB read as 10^6 B (derived)
pub const MAX_LONG_SIDE: Px               = Px::new(1920);               // PS §9.4
pub const MAX_INPUT_FPS: FpsMilli         = FpsMilli::new(60_000);       // PS §9.4
pub const INPUT_FPS_TOLERANCE: FpsMilli   = FpsMilli::new(500);          // (assumption, M1.1) 60.5 fps accepted
pub const OUTPUT_FPS: u32                 = 30;                          // PS §9.4
pub const CREATOR_WIDTH: Px               = Px::new(1080);               // PS §9.4
pub const CREATOR_HEIGHT: Px              = Px::new(1920);
pub const FREE_WIDTH: Px                  = Px::new(720);                // PS §9.4
pub const FREE_HEIGHT: Px                 = Px::new(1280);
pub const FREE_EXPORTS_PER_MONTH: ExportCount    = ExportCount::new(3);  // PS §11
pub const CREATOR_FAIR_USE_PER_MONTH: ExportCount = ExportCount::new(100); // PS §11 (soft; never blocks)
pub const ENTITLEMENT_OFFLINE_TTL_SECS: i64 = 7 * 24 * 3600;             // PS §11
pub const MIN_DEVICE_MEMORY_GB: u32       = 4;                           // PS §9.3
pub const CAPABILITY_CHECK_BUDGET: DurMs  = DurMs::new(3_000);           // PS §9.3
pub const DETECTOR_PRECISION_TARGET: f32  = 0.90;                        // PS §8, §20.1
pub const PREVIEW_MAX_DRIFT: DurMs        = DurMs::new(80);              // PS §20.1
pub const LOUDNESS_TOLERANCE_LU: f32      = 1.0;                         // PS §20.1
pub const ANALYTICS_RETENTION_DAYS: u32   = 90;                          // PS §16
pub const MIN_WORDS: u32                  = 3;                           // (assumption, E-2) below this → REJECT_NO_SPEECH
pub const MAX_RECENT_CLIPS: u32           = 5;                           // (assumption, M2.1)
// v1.3 additions, with their rules: §43.2
```

### 10.3 Media

```rust
// media.rs
pub enum Rotation { R0, R90, R180, R270 }
pub enum Orientation { Portrait, Landscape }           // from display size after rotation; square = Landscape

pub struct ProbeInfo {
    pub container: ContainerKind,                      // Mp4 | Mov | Other
    pub file_size: Bytes,
    pub duration: DurMs,
    pub video_tracks: u32,
    pub audio_tracks: u32,
    pub video: Option<VideoTrackInfo>,
    pub audio: Option<AudioTrackInfo>,
}
pub struct VideoTrackInfo {
    pub codec: VideoCodec,                             // H264 | Hevc | Av1 | Vp9 | ProRes | Other
    pub codec_string: String,                          // WebCodecs codec string, e.g. "avc1.640028"
    pub coded_width: Px, pub coded_height: Px,
    pub rotation: Rotation,
    pub avg_fps: FpsMilli, pub max_fps: FpsMilli,
    pub is_vfr: bool,
    pub frame_count: u32,
}
pub struct AudioTrackInfo { pub codec: AudioCodec /* Aac | Mp3 | Opus | Pcm | Other */, pub codec_string: String,
                            pub sample_rate: Hz, pub channels: u32 }

pub struct ClipInfo {                                  // a validated clip; constructed only by validate_probe()
    pub duration: DurMs,
    pub display_width: Px, pub display_height: Px,     // after rotation
    pub rotation: Rotation,
    pub orientation: Orientation,
    pub is_vfr: bool,
    pub video_codec_string: String,
    pub audio_codec_string: String,
    pub audio_sample_rate: Hz, pub audio_channels: u32,
}

pub enum RejectReason {                                // serialized as REJECT_* strings; analytics enum
    Container, Corrupt, NoVideo, NoAudio, MultiAudioTrack, VideoCodec, Hevc, AudioCodec,
    Duration, FileSize, Resolution, FrameRate, DecodeUnsupported, NoSpeech,
    Hdr,                                               // v1.3, V6.1 (§43.1); appended, so stored values read unchanged
}
```

### 10.4 Transcript

```rust
// transcript.rs
pub struct Word { pub text: String, pub start_ms: TimeMs, pub end_ms: TimeMs, pub confidence: Confidence }  // PS §12.5
pub struct Sentence { pub words: WordRange, pub start_ms: TimeMs, pub end_ms: TimeMs }                      // PS §12.5 (words[] as a range)
pub enum Unit { None, Usd, Eur, Gbp, Inr, Percent, Milliseconds, Seconds, Minutes, Hours, Days, Weeks, Months, Years,
                Times, Bytes, Kilobytes, Megabytes, Gigabytes, Count { noun: String } }
pub struct Quantity { pub value: f64, pub unit: Unit, pub display: String }   // display: "$10k", "800 ms", "10,000"
pub struct NormalizedSpan { pub words: WordRange, pub quantity: Quantity }    // "ten thousand" → 10,000
pub struct Transcript {
    pub words: Vec<Word>,
    pub sentences: Vec<Sentence>,
    pub numbers: Vec<NormalizedSpan>,
    pub model_version: String,
}
```

### 10.5 Events, prosody

```rust
// events.rs
pub enum EventKind { NumberReveal, ListReveal, FromTo, KeywordPop }
pub struct ListItem { pub ordinal: u8, pub text: String, pub at: TimeMs }
pub enum EventParams {
    NumberReveal { value: Quantity, label: Option<String> },
    ListReveal   { count: u8, header: String, items: Vec<ListItem> },
    FromTo       { from: Quantity, to: Quantity, label: Option<String> },      // PS §12.5
    KeywordPop   { word: WordIdx },
}
pub struct DetectedEvent {                              // PS §12.5
    pub id: EventId,
    pub kind: EventKind,
    pub span: Span,
    pub anchors: WordRange,
    pub params: EventParams,
    pub confidence: Confidence,                         // always >= the kind's threshold; lower ones are never constructed
    pub enabled: bool,
}

// prosody.rs
pub struct WordProsody { pub energy_z: f32, pub pitch_z: f32 }   // z-score against the word's sentence
pub struct Prosody { pub per_word: Vec<WordProsody> }            // len == transcript.words.len()
```

Every `TimeMs` in a `Word`, `Sentence`, `ListItem` or `DetectedEvent` is the time at which it was spoken in the recording. v1.2: the scene, the preview and the export use program time (`OutMs`) obtained through the clip's `TimeMap` (§40.1); in V4-V6 that map is the identity, so the values are used unchanged. The plan types (`EditPlan`, `PlanItem`, `Cut`, `QuantityRef`, `Label`, `Chapter`, `TextExtras`, `MusicCue`) live in `plan.rs` and are specified in §40.2.

### 10.6 Edits, profile, summary, stage, errors

```rust
// edit.rs
pub enum StyleId { Clean, Bold, Tech }                  // PS §12.2; default Clean. V4: `Tech` is shown as "Mono" (D-170);
                                                        // the serialized id stays "tech" so stored edits read unchanged
pub struct CropOffset(f32);                             // -1.0 (left) ..= 1.0 (right); 0.0 centre; checked in new()
pub struct CaptionPlacement(f32);                       // V4 (D-171): the caption block's centre, 0.0 = top of the band
                                                        // captions may use, 1.0 = its bottom (V3's zone, the default);
                                                        // new() refuses a value that is not finite or outside 0.0..=1.0
pub struct EditState {
    pub word_edits: BTreeMap<WordIdx, String>,          // replacement text; "" hides the word; timing never changes
    pub event_overrides: BTreeMap<EventId, bool>,       // enabled flag chosen by the user
    pub style: StyleId,
    pub crop_offset: CropOffset,
    #[serde(default = "CaptionPlacement::bottom")]
    pub caption_position: CaptionPlacement,             // V4 (D-171): where the captions sit; snaps Top 0.15, Middle 0.55, Bottom 1.0
    #[serde(default = "yes")]
    pub highlight_numbers: bool,                        // V4 (D-169): numbers in the captions in the accent colour; default true
}                                                       // user choices only; no field changes audio, duration or timing.
                                                        // A V3 record has neither V4 field and reads with the defaults
// v1.2 additions, each in the release that uses it (§40.10): item_overrides keyed by EventId (V4 reuses
// event_overrides), cut_restores: BTreeSet<CutKey> and filler_cuts: bool (V6.3), music: Option<MusicChoice> (V6.7),
// caption_preset (V6.6), brand_kit: bool (V6.10), plan_ops: Vec<PlanOp> (V6.9, §49.2). Each has #[serde(default)] (§41.3).
// Cuts are restored by the user; from V6.9 a user may also ask for one in chat (§49.2).

// profile.rs
pub enum Plan { Free, Creator }
pub enum ProfileKind { Preview, Free, Creator }
pub struct ExportProfile { pub kind: ProfileKind, pub width: Px, pub height: Px, pub watermark: bool,
                           pub video_bitrate: BitsPerSec, pub audio_bitrate: BitsPerSec }

// summary.rs   ("What we changed", PS §10 J11)
pub struct ChangeSummary { pub voice_cleaned: bool,          // the enhancement chain ran (§18)
                           pub captions_emphasized: u32,     // enabled KeywordPop events; (V6.2) and Emphasis items (v6.2implementation D-481)
                           pub visual_moments: u32 }         // enabled NumberReveal + ListReveal + FromTo
// Lists only transformations that are performed. v1.2 adds pauses_shortened: u32 and time_removed: DurMs (V6.3),
// music: bool (V6.7); with no cuts nothing about cuts is reported. (v1.3: no chapters at launch, §42.)

// stage.rs
pub enum PipelineStage { ProbeAudio, Asr, AudioChain, DetectScene, RenderEncode, Mux }   // matches PS §20.2 rows
// v1.2, V6.2: Plan (the plan request and validation) is added; DetectScene also covers time map and assemble

// crates/offcut-entitlement/src/claims.rs
pub struct EntitlementClaims {
    pub v: u8,                                          // format version = 1
    pub sub: UserId,
    pub plan: Plan,
    pub free_exports_remaining: ExportCount,            // meaningful when plan == Free
    pub period_end: UnixSecs,                           // Free: end of the current UTC month; Creator: current_period_end
    pub iat: UnixSecs,
    pub exp: UnixSecs,                                  // iat + ENTITLEMENT_OFFLINE_TTL_SECS
}
```

`ErrorCode` (in `error.rs`) lists every `E_*` code of §11.2. `ApiError { code: ApiErrorCode, retry_after_secs: Option<u32> }` lives in `offcut-api-types/src/errors.rs`.

### 10.7 API DTOs (`crates/offcut-api-types`)

```rust
pub struct MagicLinkRequest   { pub email: String }
pub struct VerifyRequest      { pub token: String }
pub struct SessionResponse    { pub access_token: String, pub access_expires_at: UnixSecs, pub entitlement_token: String }
pub struct MeResponse         { pub email: String, pub plan: Plan, pub subscription: Option<SubscriptionSummary>,
                                pub entitlement_token: String }
pub struct SubscriptionSummary{ pub status: SubscriptionStatus, pub interval: BillingInterval, pub current_period_end: UnixSecs,
                                pub cancel_at_period_end: bool }
pub enum   SubscriptionStatus { Active, PastDue, Canceled }
pub enum   BillingInterval    { Monthly, Annual }
pub enum   Offer              { CreatorMonthly, CreatorAnnual, CreatorAnnualFounding }   // PS §11; founding = E-5 presale
pub struct CheckoutRequest    { pub offer: Offer }
pub struct UrlResponse        { pub url: String }
pub struct UsageReceiptRequest{ pub export_id: ExportId }
pub struct EntitlementResponse{ pub entitlement_token: String }
pub struct EventsBatch        { pub anon_id: AnonId, pub events: Vec<AnalyticsEvent> }   // analytics.rs, §22.6
pub enum   Wanted             { Launch, Safari, Firefox, Mobile, Linux }
pub struct NotifyMeRequest    { pub email: String, pub wanted: Wanted }
pub struct AccountExport      { pub user: ExportedUser, pub sessions: Vec<ExportedSession>,
                                pub subscription: Option<SubscriptionSummary>, pub usage_receipts: Vec<ExportedReceipt> }
```

All request structs use `#[serde(deny_unknown_fields)]`.

### 10.8 Notes

- **Why these representations.** v1.2: two timeline types, `TimeMs` (recording) and `OutMs` (program), because from V6.3 cuts make them different instants; they are introduced in V4 with an identity map so that cuts change data, not types (§40.1, TDR-7). (v1.1 had one type because nothing was cut.) `DurMs` stays separate so a length is never passed where a position is expected. Integer milliseconds match the PS data model (PS §12.5) and are exact at 30 fps to within 1 ms; frame timestamps are derived from `FrameIdx`, never accumulated. `Sentence.words` is a range because sentences never overlap.
- **Serialization.** Rust ↔ worker JS: `serde-wasm-bindgen` for structs; PCM as `Float32Array`; frames as `web_sys::VideoFrame` handles; file access through the `RandomAccess` callback. Worker ↔ main: structured clone of generated types; PCM buffers are transferred, never copied. Client ↔ server: JSON of `offcut-api-types`. Storage: IndexedDB structured clone of generated types wrapped in `{ schemaVersion, value }` (§23.2). Entitlement token: `base64url(JSON claims) "." base64url(Ed25519 signature)`, both without padding; the signature covers the ASCII bytes of the first segment, so no JSON canonicalization is needed. The claims JSON uses the Rust field names of §10.6, `plan` is `"free"` or `"creator"` and `sub` is a UUID string (v2implementation D-25).
- **Ownership (who may write).** `ClipInfo`: `validate_probe` or `route_probe` (§43.1), and for a short `short_clip_info(&SourceInfo, &Assembly)` in the same file. `SourceInfo`: `route_probe` only. `Assembly`: `Assembly::whole` or `from_parts` only. `ShortSpec`: `validate_shorts` only. `FramePath`: `FramePath::none` or `offcut-vision::tracking` only. `Transcript`: `normalize_transcript` only; user edits never mutate it (they live in `EditState`). `DetectedEvent`: `offcut-detect` only; `enabled` is overlaid from `EditState.event_overrides` in `build_scene`. `EditPlan`: `offcut-plan::from_rules` or the server's `/plan` handler (which sets `source`); `ValidPlan`: `offcut-plan::validate` only. `TimeMap`: `TimeMap::identity` or `offcut-plan::timemap::build` only. `ProgramAudio`: `offcut-dsp::program::assemble` only (v1.2). `EditState`: use-cases through `edits-repo.ts`. `EntitlementClaims`: `server/src/entitlement/issue.rs` only. `ExportProfile`: `offcut-entitlement/src/profile.rs::export_profile` only.

---

## 11. Error Architecture

### 11.1 Layers

```text
LOW-LEVEL      DOMException, WebCodecs error callbacks, GPUDevice.lost, wgpu::Error,
               RandomAccess read failures, ONNX Runtime throws, sqlx::Error, reqwest::Error
   │  wrapped in: each pure crate's error enum in its lib.rs (ContainerError, DspError,
   │              SceneError, RenderError, MuxError, TokenError); server/src/error.rs (AppError)
DOMAIN         typed Rust enums; carry no user-facing text
   │  wrapped in: offcut-wasm-core/src/lib.rs and offcut-wasm-render/src/lib.rs (→ {code, detail});
   │              web/src/workers/rpc.ts::toAppFailure; web/src/net/http.ts::toAppFailure
APPLICATION    AppFailure { code: ErrorCode, stage: FailureStage, retryable: boolean, detail?: string }
   │              (workers/protocol.ts). `detail` is shown only in dev builds and is never sent anywhere.
   │  mapped in: web/src/copy/messages.ts  [ONLY]
USER-FACING    { title, body, action } per code; a `.persistent` variant for a repeated failure
```

`FailureStage = PipelineStage | "import" | "model" | "preview" | "storage" | "api"`.

### 11.2 Codes

| Code | Category | Trigger | Retryable | Recovery | Fatal | Test |
|---|---|---|---|---|---|---|
| `E_MODEL_DOWNLOAD` | Model | CDN fetch fails after 3 attempts | Yes | Resume from stored bytes | No | `model-download.spec.ts` |
| `E_MODEL_HASH` | Model | SHA-256 differs from manifest | Yes | Delete file, download from 0 | No | `download.test.ts` |
| `E_MODEL_STORAGE` | Model | Quota error while writing model | Yes | Evict clips, retry | No | `model-download.spec.ts` |
| `E_STORAGE_QUOTA` | Resource | OPFS quota on import or export | Yes | LRU eviction (§23.3), retry | No | `failure-recovery.spec.ts` |
| `E_STORAGE_IO` | Resource | OPFS/IndexedDB operation rejects | Yes | Retry once | No | `failure-recovery.spec.ts` |
| `E_DECODE_AUDIO` | Pipeline | `AudioDecoder` error callback | Yes | Retry once | For the clip, after retry | `failure-recovery.spec.ts` (fixture `bad_audio_payload.mp4`) |
| `E_DECODE_VIDEO` | Pipeline | `VideoDecoder` error callback | Yes | Reopen decoder at previous keyframe once | For the clip, after retry | `failure-recovery.spec.ts` (fixture `bad_video_payload.mp4`) |
| `E_ASR_RUNTIME` | Pipeline | Runtime throws | Yes | Retry on the WASM backend | No | `run-pipeline.test.ts` |
| `E_ASR_OOM` | Resource | Allocation failure in ASR | Yes | Retry on the WASM backend; persistent copy suggests closing tabs | No | `run-pipeline.test.ts` |
| `E_DSP` | Pipeline | `DspError` (empty or non-finite PCM) | No | None | For the clip | `chain_no_clicks.rs`, `run-pipeline.test.ts` |
| `E_GPU_INIT` | Resource | `requestDevice` rejects | Yes | Retry once | No | `failure-recovery.spec.ts` |
| `E_GPU_LOST` | Resource | `device.lost` resolves | Yes | New session, `setScene`, resume at `ready` | No | `failure-recovery.spec.ts` |
| `E_ENCODE_VIDEO` | Pipeline | `VideoEncoder` error callback or no supported config | Yes | Next config in the ladder (§21.2) | After ladder exhausted | `start-export.test.ts` |
| `E_ENCODE_AUDIO` | Pipeline | `AudioEncoder` error callback | Yes | Retry once | For the export | `start-export.test.ts` |
| `E_MUX` | Pipeline | `MuxError` | No | None (bug) | For the export | `mux_roundtrip.rs` |
| `E_WORKER_CRASH` | Internal | Worker `error` event or WASM panic | Yes | Restart worker; resume from last completed stage | After 2 restarts per clip | `rpc.test.ts`, `failure-recovery.spec.ts` |
| `E_NET_OFFLINE` | Network | `navigator.onLine === false` or fetch `TypeError` | Yes | Retry on `online` event | No | `http.test.ts` |
| `E_NET_TIMEOUT` | Network | Cold-start budget exhausted (§22.9) | Yes | Manual retry | No | `api-cold-start.spec.ts` |
| `E_API_5XX` | Network | 5xx response | Yes | Backoff retry | No | `http.test.ts` |
| `E_API_RATE_LIMITED` | Network | 429 | Yes | Wait `retry_after_secs` | No | `rate_limit.rs`, `http.test.ts` |
| `E_AUTH_LINK_INVALID` | Auth | Unknown or used link token | No | Request a new link | No | `auth_flow.rs`, `signin.spec.ts` |
| `E_AUTH_LINK_EXPIRED` | Auth | Link older than its TTL | No | Request a new link | No | `auth_flow.rs`, `signin.spec.ts` |
| `E_AUTH_SESSION_EXPIRED` | Auth | Refresh returns 401 | No | Sign in again | No | `auth_flow.rs` |
| `E_AUTH_EMAIL_UNAVAILABLE` | Auth | Email provider rejects or quota reached | Yes | Retry later | No | `auth_flow.rs` |
| `E_BILLING_UNAVAILABLE` | Billing | Provider API error creating checkout/portal | Yes | Retry | No | `billing_events.rs`, `checkout.spec.ts` |
| `E_BILLING_PENDING` | Billing | Plan not Creator 60 s after checkout return | Yes | Re-poll | No | `checkout.spec.ts` |
| `E_ENTITLEMENT_INVALID` | Entitlement | Signature or format check fails | No | Fetch a fresh token | No | `token_roundtrip.rs`, `start-export.test.ts` |
| `E_ENTITLEMENT_EXPIRED` | Entitlement | `exp` passed and refresh failed | Yes | Go online, retry | No | `start-export.test.ts` |
| `E_INTERNAL` | Internal | Illegal state transition, invariant breach, second job sent to a busy worker | No | Reload | For the session | `clip-machine.test.ts`, `export-machine.test.ts` |

Input rejections (`REJECT_*`) are listed with triggers and tests in §15.3. Unsupported reasons (`UNSUPPORTED_*`) are listed in §3 and tested in `capability.test.ts` and `unsupported.spec.ts`.

### 11.3 Rules

- **Expected rejection is not an error.** `importAndProbe` returns `{ ok: ClipInfo } | { rejected: RejectReason }`. A rejection is a value, shows instructions (not a retry button), fires `clip_rejected`, and never fires `client_error`.
- **Exhaustive handling.** TypeScript: every `switch` over a generated union ends in an `assertNever` default; `@typescript-eslint/switch-exhaustiveness-check` is an error. Rust: `clippy::wildcard_enum_match_arm` is denied for workspace enums. `scripts/check-copy-codes.mjs` fails CI when any `ErrorCode`, `RejectReason`, blocker or unsupported reason lacks an entry in `messages.ts`.
- **Panic policy (client).** Non-test Rust contains no `unwrap`/`expect`/`panic!` (§7). A panic that still occurs is caught by the hook in `offcut-wasm-*/src/lib.rs`, posted as `E_WORKER_CRASH`, and the worker is terminated and recreated. A WASM instance is never reused after a panic.
- **Panic policy (server).** `tower_http::catch_panic::CatchPanicLayer` returns 500 `internal`; the process continues.
- **No silent catch.** `catch` blocks must rethrow or return an `AppFailure`; ESLint `no-empty` and a custom `no-restricted-syntax` rule forbid `catch {}` and `.catch(() => {})` outside `analytics/client.ts` (analytics failures are dropped by design).
- **Cancellation is not an error.** A cancelled job resolves with `{ cancelled: true }` and fires `job_cancelled`.
- **Silence is not an error.** No code exists for silence, a long pause or a recording that is mostly silent; such audio is enhanced and exported at full length (§18.4). The only speech-related outcome is the `REJECT_NO_SPEECH` rejection when fewer than `MIN_WORDS` words are transcribed in the whole clip (§15.3), which is about having nothing to caption, not about pause length.

---

## 12. State Management

### 12.1 Stores

| Store | Owns | Written by |
|---|---|---|
| `capability-store.ts` | Capability state and report | `start-app.ts` |
| `model-store.ts` | Model state, progress | `models/model-manager.ts` through store actions |
| `clip-store.ts` | Current clip state, `ClipInfo`, transcript, events, `EditState`, feed lines, last failure | `import-clip.ts`, `run-pipeline.ts`, edit use-cases, `restore-clip.ts`, `cancel-job.ts`, `clear-local-data.ts` |
| `preview-store.ts` | Preview state, position | `control-preview.ts` |
| `export-store.ts` | Export state, progress, summary | `start-export.ts`, `cancel-job.ts` |
| `auth-store.ts` | Auth state, email, in-memory access token | `start-app.ts`, sign-in/out use-cases, `delete-account.ts` |
| `entitlement-store.ts` | Latest entitlement token and decoded claims | `start-app.ts`, `complete-sign-in.ts`, `start-export.ts`, `confirm-checkout.ts`, `receipt-outbox.ts` callback |

Stores hold no derived data that another store owns. `entitlement/entitlement.ts::effective()` is the only place that combines the token with the outbox. As built in V6 (v6implementation D-267), it subtracts only the outbox rows recorded as Free exports of the token's own account (`sub`), so a Creator export never spends a Free count and another account's rows never count against this one. `entitlement-store` reads claims without verifying, for display and blockers only, and an undecodable token counts as none (D-292); the worker still verifies (INV-9).

### 12.2 State machines

**Capability:** `unchecked → checking → supported | unsupported(reason)`

| From → To | Trigger |
|---|---|
| unchecked → checking | `startApp` |
| checking → supported | all checks pass |
| checking → unsupported(reason) | first failing check, in the order of §13.2 |

**Model:** `unknown → absent | partial | ready; → downloading → verifying → ready`

| From → To | Trigger |
|---|---|
| unknown → absent / partial / ready | `modelManager.inspect` |
| absent, partial, failed → downloading | `ensureReady` or user retry |
| downloading → verifying | last byte written |
| downloading → partial | network failure, user cancel, tab close |
| downloading → failed(`E_MODEL_DOWNLOAD` / `E_MODEL_STORAGE`) | retries exhausted / quota |
| verifying → ready | hash matches |
| verifying → failed(`E_MODEL_HASH`) | hash differs (file deleted) |
| ready, partial → absent | `clearLocalData`; manifest model id changed |

**Clip:** `idle → importing → accepted → processing → ready ⇄ updating`

| From → To | Trigger |
|---|---|
| idle → importing | `importClip` |
| importing → accepted | `validate_probe` ok |
| importing → rejected(reason) | `validate_probe` rejects |
| importing → failed(code) | storage or worker failure |
| accepted → processing(stage, waitingModel) | `runPipeline` |
| processing → ready | stage F of C-4 completes |
| processing → rejected(`NoSpeech`) | transcript shorter than `MIN_WORDS` |
| processing → failed(code) | any stage failure |
| processing → accepted | `cancelJob` |
| ready → updating → ready | any edit use-case (C-6) |
| failed → processing | user retry |
| rejected, failed, ready, accepted → idle | dismiss, new clip, `clearLocalData` |
| idle → ready | `restoreClip` with complete saved state |

**Preview:** `detached → stopped → playing ⇄ paused; seeking; locked`

| From → To | Trigger |
|---|---|
| detached → stopped | `attachPreview` when clip is `ready` |
| stopped, paused → playing | play |
| playing → paused | pause, edit started, tab hidden |
| playing → stopped | end of clip |
| playing, paused → seeking → previous state | scrub |
| any → locked | export enters `rendering` |
| locked → paused | export leaves `rendering`/`muxing` |
| any → detached | clip leaves `ready`/`updating` |

**Export:** `idle → gating → rendering → muxing → saving → done`

| From → To | Trigger |
|---|---|
| idle → gating | `startExport` |
| gating → idle | any blocker |
| gating → rendering | blockers clear, cache miss |
| gating → saving | render-cache hit |
| rendering → muxing | last frame encoded |
| muxing → saving | `finalize` ok |
| saving → done | download triggered, outbox written |
| rendering, muxing → cancelled → idle | `cancelJob` |
| rendering, muxing, saving → failed(code) | failure |
| failed → gating | user retry |
| done, failed → idle | dismiss, new export, new clip |

As built in V5 (v5implementation D-230, D-231): the table is unchanged. A token refused in `gating` goes `gating → rendering → failed`, announced with `export_started` then `export_failed`, as there is no `gating → failed`. A cancel that lands in `gating` goes `gating → idle`, as there is no `gating → cancelled`. A blocked export's code stays in the store until the next start. `failed → gating` is not fired yet (v4implementation D-153).

**Auth:** `unknown → anonymous | anonymous_offline | signed_in`

| From → To | Trigger |
|---|---|
| unknown → signed_in | refresh ok |
| unknown → anonymous | refresh 401 |
| unknown → anonymous_offline | refresh network failure |
| anonymous → link_sent | magic-link request accepted |
| link_sent → signed_in | verify ok in this tab, or `signed_in` broadcast then refresh ok |
| link_sent → anonymous | dialog closed |
| anonymous_offline → signed_in / anonymous | `online` event then refresh |
| signed_in → anonymous | sign-out, `E_AUTH_SESSION_EXPIRED`, account deleted |
| unknown, anonymous → signed_in (V6, D-295) | verify ok in this tab |
| anonymous → signed_in (V6, D-295) | `signed_in` broadcast, then refresh ok |
| unknown, anonymous_offline → link_sent (V6, D-295) | magic-link request accepted: the dialog can be sent from either |
| link_sent → link_sent (V6, D-295) | a resend |

As built in V6: the callback tab does not restore a session (D-296): `main.tsx` tells `startApp` whether to restore, by the path (D-308), so a refresh's answer cannot overwrite the session that verify has just made. A sign-out clears the entitlement token; a lost session does not, so offline exports work until the token's `exp` (D-291).

Server-side subscription status transitions are in §22.5.

### 12.3 Illegal transitions

Every machine goes through `state/machines/transition.ts::transition(machine, state, event)`. A pair absent from the tables above is illegal. On an illegal attempt: dev and test builds throw `IllegalTransitionError`; production builds leave the state unchanged and fire `client_error {error_code: E_INTERNAL}`. Explicitly illegal, each with a unit test in the machine's test file:

- `unsupported → *` (terminal until reload).
- `rejected → processing` (a rejected clip is never processed).
- `importing → processing` without `accepted`.
- `processing → importing`, `ready → importing` (a new clip must first move the old one to `idle`).
- `verifying → downloading` without `failed`.
- `export: rendering → rendering`, `done → rendering` (a new export starts from `idle` with a new `ExportId`).
- `export: gating → rendering` while clip state is not `ready`.
- `preview: detached → playing`.
- `auth: anonymous → signed_in` without a verify in this tab or a refresh: the trigger names which (V6, D-295).

### 12.4 Cross-machine rules

- Capability `!= supported` blocks import, processing and export. It does not block landing, sign-in, account or settings pages.
- Export in `rendering` or `muxing` locks the preview (`locked`), disables every review control, and blocks import and `clearLocalData`. As built in V5 the lock is taken when `gating` starts, since the render worker takes one job at a time and a playing preview would refuse `exportProfile`; a cache hit or a blocker unlocks at once (v5implementation D-221).
- Clip in `importing` or `processing` blocks a second import; the user cancels first (C-12).
- Model `downloading` blocks only the ASR stage; import, audio extraction and the voice chain proceed.
- The ASR session and the render session are never loaded together: `run-pipeline.ts` awaits `pool.asr.unload()` before `pool.render.openSession()` (§31).
- Clip `updating` pauses the preview; it resumes in its previous state when the scene is set.
- Auth transitions never interrupt processing or an export in flight.
- Tab hidden: preview pauses; export continues (workers are not throttled like timers; verified in TE-3).

### 12.5 Blockers

`state/blockers.ts` is the only place these conditions are evaluated. Functions: `forImport()`, `forExport()`, `forCheckout()`, `forClearLocalData()`; each returns the first blocker in the order listed.

| Code | Checked before | Condition |
|---|---|---|
| `B_UNSUPPORTED` | import | capability state `!= supported` |
| `B_PIPELINE_BUSY` | import, clear local data | clip state in `importing`, `processing`, `updating` |
| `B_EXPORT_IN_PROGRESS` | import, export, clear local data | export state in `gating`, `rendering`, `muxing`, `saving` |
| `B_PIPELINE_NOT_READY` | export | clip state `!= ready` |
| `B_STORAGE_LOW` | export | free quota `<` estimated output bytes x 1.2 (derived: bitrate x `ClipInfo.duration`) after eviction |
| `B_NOT_SIGNED_IN` | export, checkout | auth state not `signed_in` and no unexpired cached token |
| `B_ENTITLEMENT_EXPIRED` | export | token `exp <` now |
| `B_NO_FREE_EXPORTS` | export | `plan == Free` and `effective().freeExportsRemaining == 0` |

`B_MODEL_NOT_READY` is not a blocker: the pipeline waits in `processing(waitingModel = true)`.

As built in V6 (v6implementation D-290, D-309, D-310): `forExport()` without `room` asks, after V5's three, the account: `B_NOT_SIGNED_IN` when not signed in and no unexpired token; `B_ENTITLEMENT_EXPIRED` when signed in with none; `B_NO_FREE_EXPORTS` on Free with `effective()` at 0. `disablesExport` disables the button only for `B_EXPORT_IN_PROGRESS` and `B_PIPELINE_NOT_READY`; the three account blockers let the click through, to open the sign-in dialog, renew the token, or open the last-export dialog. `forExport(room)`, in `gating` after a cache miss, asks only the room: the account was asked at the gate, and an export that has begun is not sent back to it. A token that cannot be renewed at the gate is `export-store`'s `gateFailure`, shown by the export-failed card; no machine state changes, and no `export_started` is sent. `forCheckout()` needs a session: a cached token is not one. `startExport` reloads the token from `entitlement-repo` before asking (D-294), as the repository is the truth across tabs. The counter sits outside the button's accessible name, which tests match as "Export" exactly (D-302).

As built in V5 (v5implementation D-229): `forExport(room?)`. Without `room` it leaves `B_STORAGE_LOW` out, so the button is not disabled by a reading that may be stale; with `{ freeBytes, neededBytes }` it evaluates `freeBytes < neededBytes x 1.2`. `startExport` calls it after a cache miss, the cached file's bytes counted as free. A browser that gives no estimate is not blocked. V6 appends the last three codes.

---

## 13. Client Architecture

**Model: a single-page React app whose main thread only renders stores and dispatches use-cases; rejected: a multi-page app (loses in-memory clip state across the sign-in step) and server-side rendering (no server to run it; no benefit).**

### 13.1 Routes

| Path | Page | Notes |
|---|---|---|
| `/` | `LandingPage` | J1 (design release, designimplementation §6): hero with the captioned headline and the hero clip (asset host), the capability line, the supported-browser line and the "What leaves your device" link; sections of features and "Coming soon"; `DropZone` with the sample-clip button; pricing with a way in ("Start free" to `/app`); the update form. Sections in `ui/landing/`, animated with Motion and GSAP (TDR-26). Runs the capability check for the E-8 beacon |
| `/app` | `EditorPage` inside `CapabilityGate` | J2-J12. Unsupported → `UnsupportedPage` in place |
| `/auth/callback` | `AuthCallbackPage` | Reads `token` from the query, then removes it with `history.replaceState` |
| `/account` | `AccountPage` | Plan, export counter, checkout return, manage subscription, delete, export data |
| `/settings` | `SettingsPage` | PS §12.7: `WhatLeavesTable`, processing mode "Local only", model version and cache size, "Clear local data", "Verify it yourself" |
| `/legal/:doc` | `LegalPage` | `terms`, `privacy`, `refunds`, `faq` (static content in `messages.ts`; PS §20.6) |
| `*` | redirect to `/` | |

`vercel.json` rewrites every non-file, non-`/api` path to `/index.html`.

### 13.2 Startup capability check

```text
runCapabilityCheck(): Promise<CapabilityReport>          // platform/capability.ts; budget CAPABILITY_CHECK_BUDGET
  order (first failure wins; later checks still run for the report, each with a 1 s timeout):
   1 UNSUPPORTED_MOBILE         navigator.userAgentData?.mobile === true
   2 UNSUPPORTED_WEBCODECS      !("VideoEncoder" in globalThis && "AudioEncoder" in globalThis)
   3 UNSUPPORTED_THREADS        !crossOriginIsolated || typeof SharedArrayBuffer === "undefined"
   4 UNSUPPORTED_WASM_SIMD      !WebAssembly.validate(SIMD_PROBE_BYTES)
   5 UNSUPPORTED_STORAGE        getDirectory() or indexedDB.open rejects
   6 UNSUPPORTED_LOW_MEMORY     navigator.deviceMemory !== undefined && navigator.deviceMemory < MIN_DEVICE_MEMORY_GB
   7 UNSUPPORTED_WEBGPU         requestAdapter() === null
   8 UNSUPPORTED_H264_DECODE    VideoDecoder.isConfigSupported(H264_DECODE_PROBE).supported === false
   9 UNSUPPORTED_H264_ENCODE    no config in VIDEO_ENCODE_LADDER (§21.2) is supported at 1080x1920
  10 UNSUPPORTED_AAC_DECODE     AudioDecoder.isConfigSupported(AAC_PROBE) false
  11 UNSUPPORTED_AAC_ENCODE     AudioEncoder.isConfigSupported(AAC_ENCODE_CONFIG) false
  report = { result, unsupportedReason?, gpuVendor, memoryBucket, platform }   // enums only
```

`gpuVendor ∈ {intel, amd, nvidia, apple, qualcomm, other, unknown}` from `adapter.info.vendor`; `memoryBucket ∈ {lt4, gb4, gb8plus, unknown}`; `platform ∈ {windows, macos, linux, chromeos, android, other, unknown}` from `navigator.userAgentData?.platform`. No other device attribute is read (PS §18: no fingerprinting beyond enums).

As built in V6 (V6 CL Entry 12, founder's decision of 2026-11-03): `storageAnswer` asks check 5 once more when its first try has not answered in its time, and either try's answer counts; a no or a rejection is final at once. The other checks keep one wait. With a slow disk the run can take up to 4 s, past PS §9.3's 3 s; a normal run is unchanged. Reason: a capable browser on a busy laptop was told it cannot keep files.

### 13.3 Key components

As built in V4 (the editor of the V4 plan's §17, D-178 to D-192), and V5's two rows. Components that later versions add (`SignInDialog`, `UpgradePrompt`, `RecentClips`) keep their rows of the version that builds them.

| Component | Reads | Calls |
|---|---|---|
| `CapabilityGate` | capability store | none |
| `DropZone` (`landing` and `stage` variants) | clip store, blockers | `importClip` |
| `EditorLayout` → `TopBar`, `Stage`, `SidePanel` | ui store, clip store, export store | none; holds the `aria-live` region, which also says a finished export's outcome, and the `ToastHost` (D-192) |
| `TopBar` | clip store (length, save state), history | `undo`, `redo`; holds `ExportButton` or `ExportProgress` |
| `Stage` | clip store, preview store | holds `DropZone`, `ModelDownloadPanel`, `ProcessingCard`, `PreviewPlayer`, `RejectionPanel` or `ErrorPanel` by state |
| `ModelDownloadPanel` | model store | `cancelJob("model")` |
| `ProcessingCard` → `ProcessingFeed` | clip store (stage, feed lines) | `cancelJob("import" \| "pipeline")`; the cancelled card's `runPipeline` ("Process again") and `dismissClip` |
| `PreviewPlayer` → `PreviewGestures` | preview store, clip store | `controlPreview.*`; `hitTest` on hover; caption and video drags (D-183); the `data-*` statistics (D-155) |
| `SidePanel` (`Tabs`: Captions, Style, Visuals, Framing) | ui store | none |
| `CaptionEditor` → `CaptionWordControls` | clip store (transcript, `SceneReport.chunks`), ui store | `editWord`, `replaceAll`, `controlPreview.seek` |
| `StylePanel` → `StylePicker` | clip store, preview store | `setStyle`, `setHighlightNumbers`; `styleThumbnails` |
| `PlanItemList` → `CutsSection` | clip store (plan, overrides) | `toggleEvent`, `controlPreview.seek`; the empty state `NO_EVENTS_FOUND` (D-154); `CutsSection` renders nothing in V4 |
| `FramingPanel` → `CropOffsetControl` | clip store | `setCaptionPosition`, `setCropOffset` |
| `ExportButton` → `LastExportDialog` (V6) | export store, `entitlement.effective()`, blockers | `startExport`, `confirmLastExport`, `closeGate`; the counter outside the button's accessible name; disabled only by D-290's two blockers |
| `SignInDialog` → `SignInForm`, in `GateDialog` (V6) | auth store | `requestMagicLink`; the Free/Creator limits before the email field; "open the link on this computer" (D-302) |
| `UpgradePrompt` (V6) | export store, entitlement store | `startCheckout`, "Make another clip" (`after_export` only), dismiss; non-modal, beside `ExportOutcome`, takes no focus (D-302, D-314) |
| `ExportProgress` (the pill, D-189) | export store | `cancelJob("export")` |
| `ChangeSummary`'s `ExportOutcome`, in `SidePanel`'s slot above the tabs (v5implementation D-232) | export store | none; closing is local, keyed by `exportId` |
| `PostExportQuestion` (D-234) | export store | `answerPostExport` |
| `ShortcutsSheet` (opened with `?`, D-185) | ui store | none |
| `ToastHost` (a stack of at most three) | ui store | the toast's action: replace-all, hide, undo |
| `WhatLeavesTable` | `gen/api.ts::ANALYTICS_EVENT_DOCS` | none |
| `RejectionPanel`, `ErrorPanel` (by context, D-190) | clip/export store failure | retry, `reloadPreview`, undo; `failureShown` sends `client_error` once |
| `NotifyMeForm` | none | `submitNotifyMe` |
| `AuthCallbackPage` (V6) | auth store | `completeSignIn` once, after `ready()`; never refreshes (D-296) |
| `AccountPage` (V6) | auth store, entitlement store | `startCheckout`, `confirmCheckout`, `openBillingPortal`, `exportAccountData`, `deleteAccount` behind a question, sign-out (D-315) |

The primitives (`ui/primitives/`: `Button`, `IconButton`, `Kbd`, `PanelSection`, `SegmentedControl`, `Skeleton`, `Slider`, `Spinner`, `Switch`, `Tabs`, `Toast`, `Tooltip`) take their values from `ui/styles/tokens.css` only (D-179, D-181); no component library is used.

### 13.4 Rules

- **Copy.** Every PS §10 microcopy string, every rejection, error, blocker and unsupported message is a key in `copy/messages.ts`. Numbers inside copy (90 seconds, 3 exports, prices, model size) are interpolated from `gen/domain.ts` limits, `config/pricing.ts` and `config/model-manifest.json`; none is typed into a sentence. Enforced by `messages.test.ts` (asserts no digit literals in message templates except through placeholders).
- **Limits before the click (PS §10 J9).** `ExportButton` renders the counter and plan limits in its idle state; `SignInDialog` renders the Free/Creator comparison before the email field. `export-free.spec.ts` asserts the counter is visible before the first click.
- **No upgrade prompts during processing or editing (PS §11).** `UpgradePrompt` renders only for placements `after_export` and `limit_reached`; the placement type has no other member.
- **One prompt per export.** Dismissal is stored in `export-store` for that `ExportId`. As built in V6 (D-314): once per watermarked export, until dismissed; at the limit, when Export is pressed, and a press while it is showing adds nothing.
- **Edits are optimistic and local.** No edit ever awaits the network.

### 13.5 Acceptance targets

- Capability check completes within 3 s (PS §9.3); `capability.spec.ts`.
- App shell (HTML + JS + CSS, excluding WASM) at most 500 kB gzip (400 until the design release; the founder's yes of 2026-11-04, designimplementation D-320; assumption, M2.4); `scripts/check-hosts.mjs` also reports bundle sizes.
- No main-thread task longer than 100 ms while a clip is processing (assumption, M2.4); `pipeline-preview.spec.ts` with `PerformanceObserver("longtask")`.

**Contingency.** Bundle over budget: lazy-load `/account`, `/settings`, `/legal`. Long tasks: move the offending work behind a worker method.

**Seams (not built now).** Google sign-in (P1): a second button in `SignInDialog` and one route in `server/auth`. Hero A/B for E-6: `landing_view.hero_variant` already has two enum members; `LandingPage` renders `outcome` only until E-6 starts.

---

## 14. Worker Orchestration and Message Protocol

**Model: the main thread is the hub; each worker exposes typed request/response methods with progress events and cooperative cancellation; workers never message each other. Rejected: a worker-to-worker `MessageChannel` mesh (ordering and lifecycle bugs), SharedArrayBuffer ring buffers for PCM (not needed at 17 MB), a generic RPC library (hides transfer lists).**

### 14.1 Envelope

```ts
// workers/protocol.ts  [ONLY]
export type WorkerKind = "media" | "asr" | "audio" | "render";
export type Req<M extends string, P> = { type: "req"; jobId: JobId; method: M; params: P };
export type Res<R> =
  | { type: "res"; jobId: JobId; ok: true; result: R }
  | { type: "res"; jobId: JobId; ok: false; failure: AppFailure }
  | { type: "res"; jobId: JobId; cancelled: true };
export type Progress = { type: "progress"; jobId: JobId; stage: PipelineStage | "import" | "model";
                         done: number; total: number; feed?: FeedLine };
export type Cancel = { type: "cancel"; jobId: JobId };
export type FeedLine =                                   // J6 feed; rendered through messages.ts
  | { kind: "transcribing" } | { kind: "cleaning_voice" }
  | { kind: "event_found"; eventKind: EventKind; display: string; displayTo?: string };   // display stays on device
export type AppFailure = { code: ErrorCode; stage: FailureStage; retryable: boolean; detail?: string };
```

`displayTo` (v3implementation D-83, a reopened V1 contract): for a FromTo, `display` is `from.display` and `displayTo` is `to.display`; for the other three kinds `displayTo` is absent. `messages.feed.eventFound.from_to` is `(display: string, displayTo: string) => string`, "Found: {display} to {displayTo}" (PS §10 J6's "Found: $2k to $20k"); `ProcessingFeed`'s `switch` passes the second value, so the word "to" stays in `messages.ts` (INV-15).

### 14.2 Methods

As built in V4 and V5 (copied from `web/src/workers/protocol.ts`; v4implementation D-124, D-126 to D-128, D-155, D-174, D-182, D-186; v5implementation D-214, D-220):

```ts
export interface MediaWorkerApi {
  importAndProbe(p: { clipId: ClipId; file: File }): Promise<{ ok: ClipInfo } | { rejected: RejectReason }>;
  extractAudio(p: { clipId: ClipId }): Promise<{ pcm48: Float32Array; pcm16: Float32Array }>;   // both transferred
}
export interface AsrWorkerApi {
  load(p: { modelId: string; backend: "webgpu" | "wasm" }): Promise<{ backend: "webgpu" | "wasm" }>;
  transcribe(p: { pcm16: Float32Array }): Promise<{ ok: Transcript } | { rejected: "NoSpeech" }>;
  unload(): Promise<void>;
}
// The audio worker has no method that detects silence or plans a cut. Only `assemble` applies a time map,
// and in V4 the only map is the identity.
export interface AudioWorkerApi {
  runChain(p: { pcm48: Float32Array }): Promise<{ loudness: Lufs }>;                 // out48 kept in the worker; out48.length === pcm48.length
  measureProsody(p: { words: Word[]; sentences: Sentence[] }):
    Promise<{ out48: Float32Array; prosody: Prosody }>;                              // out48 transferred out, unmodified
  assemble(p: { out48: Float32Array; timeMap: TimeMap; speech: OutSpan[] }):
    Promise<{ prog48: Float32Array; loudness: Lufs }>;                               // V4 (D-127): out48 in, prog48 out; the worker keeps nothing
  release(): void;                                                                   // V4 (D-128): fire-and-forget; drops any audio it holds
}
export interface RenderWorkerApi {
  openSession(p: { clipId: ClipId; clipInfo: ClipInfo }): Promise<void>;
  detect(p: { transcript: Transcript; prosody: Prosody; wordEdits?: Record<number, string> }): Promise<DetectedEvent[]>;
                                                                                     // with wordEdits: the transcript as edited (replace-all, D-175)
  redetectSentence(p: { sentence: SentenceIdx; wordEdits: Record<number, string> }): Promise<DetectedEvent[]>;
  rulesPlan(p: { transcript: Transcript; events: DetectedEvent[] }): Promise<EditPlan>;   // V4 (D-124): stateless
  setScene(p: { transcript: Transcript; plan: EditPlan; edit: EditState; timeMap: TimeMap;
                profile: "preview" | { entitlementToken: string } }): Promise<SceneReport>;
                                                                                     // the plan is validated against the transcript as edited first (§40.4)
  attachPreview(p: { canvas: OffscreenCanvas }): Promise<void>;                      // transferred; answers once the first frame is drawn
  previewPlay(p: { clock: ClockSync }): Promise<PreviewStats>;                       // answers when the loop returns
  previewClock(p: { clock: ClockSync }): void;                                       // fire-and-forget
  previewPause(): Promise<void>;
  previewSeek(p: { at: OutMs }): Promise<void>;
  hitTest(p: { at: OutMs; x: number; y: number }): Promise<Hit>;                     // V4 (D-174); { kind: "video" } at once while playing
  styleThumbnails(p: { at: OutMs; width: number; height: number }): Promise<ImageBitmap[]>;
                                                                                     // V4 (D-186): Clean, Bold, Mono of `at`, paused only; transferred
  captionMirror(p: { at: OutMs }): Promise<CaptionMirror | null>;                    // development and test builds only (D-155)
  exportProfile(p: { entitlementToken: string }): Promise<ExportProfile>;            // V5 (D-220): verified here; keeps nothing
  exportClip(p: { exportId: ExportId; entitlementToken: string; prog48: Float32Array }):
    Promise<{ opfsPath: string; summary: ChangeSummary; stageTimings: StageTiming[] }>;   // V5 (D-214): the program's audio, transferred
  closeSession(): Promise<void>;
}
export type ClockSync = { audioMs: OutMs; epochMs: number };   // program time (D-126); epochMs = performance.timeOrigin + performance.now()
export type SceneReport = { dropped: Dropped[]; chunks: SceneChunk[] };              // what validation left out; the captions' chunks (D-182)
export type SceneChunk = { start: WordIdx; end: WordIdx; at: OutMs };
export type PreviewStats = { frames: number; late: number; maxDriftMs: DurMs; downscaled: boolean };   // D-155, D-149
export type CanvasBox = { x: number; y: number; w: number; h: number };              // fractions of the canvas
export type Hit = { kind: "word"; word: WordIdx; rect: CanvasBox } | { kind: "captions"; rect: CanvasBox } | { kind: "video" };
export type CaptionMirror = { text: string; colors: string[] };                      // the shown caption, each word's fill as #rrggbbaa
export type StageTiming = { stage: PipelineStage; durationMs: DurMs };
```

The development builds' fault messages (`devCrash`, `devHang`, `devFailNext`, `devFailEncoder`, `ui/dev-hooks.ts`) are compiled out of production (`__OFFCUT_DEV_FAULTS__`, §20.4 of the V4 plan).

### 14.3 Lifecycle, cancellation, backpressure

- **Creation.** `pool.ts` creates each worker lazily on first use and loads its WASM bundle once. `asr` keeps its worker but drops the model session on `unload()`. `render` lives from `openSession` to `closeSession`.
- **One job per worker.** A second `req` while one is pending is rejected with `E_INTERNAL` (detail `Busy`). Exception: `previewClock`, `previewPause`, `previewSeek` and `cancel` are accepted during `previewPlay`. As built in V4: the render worker has several callers that do not wait for each other, so the calls that must not fail (the update rule's, a seek, a play, a pause) go through `pool.ts`'s `whenFree`, which asks again every 16 ms for up to 2 s; the calls that may be dropped (a hit test, the style cards, the development mirror) take the refusal (V4 CL known issue 31). A worker that was just restarted may hold its OPFS files a moment: `whenReleased` retries a file operation that finds one locked.
- **Cancellation points.** Import: every 4 MiB chunk. Audio decode: every decoded AAC batch. ASR: every 30 s audio window. DSP: every 1 s block. Render/export: every frame. A step past its check finishes, then the job resolves `{cancelled: true}` and releases every `VideoFrame`, closes encoders and deletes temp files.
- **Cancel timeout.** `CANCEL_TIMEOUT_MS = 2000` (`workers/pool.ts`; kept after V4's measurement on D1: a render worker made to hang is restarted and its run back at `accepted` 2.4 to 2.5 s after Cancel is pressed, `cancel.spec.ts`). No response → `pool.restartWorker(kind)`, which ends the worker, rejects its pending calls with `E_WORKER_CRASH` and makes a fresh one on the next call. `usecases/cancel-job.ts` (V4, D-152) owns one abort signal per scope (`import`, `pipeline`, `model`, `export`) and sends one `job_cancelled { stage }` when the job ended as cancelled.
- **Progress backpressure.** Workers post at most 10 progress messages per second per job; feed lines are never dropped.
- **Transfer rule.** Every `Float32Array` and `OffscreenCanvas` in a message is in the transfer list. `rpc.ts` asserts in dev builds that a sent buffer is detached afterwards.
- **Restart budget.** At most 2 automatic restarts per worker per clip; then the failure is surfaced with the persistent copy.

### 14.4 Edge cases

- Worker script fails to load (offline after shell cache miss): `E_WORKER_CRASH` with stage `import`.
- Message arrives for a finished `jobId`: dropped, counted in a dev-only counter.
- Tab closed mid-export: temp file remains; `opfs.sweepTemp()` deletes it at next start (C-13).
- `postMessage` of a detached buffer: dev assertion; production `E_INTERNAL`.

**Acceptance.** `rpc.test.ts` covers request/response, failure, progress throttling, cancel, cancel timeout, crash and restart with a fake `Worker`. `cancel.spec.ts` covers C-12 in the browser.

**Contingency.** If hub forwarding of PCM shows up in profiles (over 50 ms per hop on R1; assumption, M2.4), switch `pcm48`/`out48` to `SharedArrayBuffer` views; message shapes do not change.

**Seams.** v1.3: face and person models run inside `asr.worker` (DC-10), so the worker count stays four. A local LLM (P2) would be a new `WorkerKind` and interface in `protocol.ts`; nothing else in the hub changes.

---

## 15. Media Ingest: Import, Probe, Validate, Decode

**Model: the file is copied once into OPFS; a Rust MP4/MOV demuxer reads it through a random-access callback; WebCodecs decodes. Rejected: FFmpeg.wasm (PS §0 item 4), `HTMLVideoElement` frame grabbing (no control over VFR timing), reading the whole file into memory (500 MB input on an 8 GB machine).**

### 15.1 Interfaces

```rust
// offcut-mp4/src/reader.rs
pub trait RandomAccess {
    fn len(&self) -> Bytes;
    fn read_at(&mut self, offset: Bytes, buf: &mut [u8]) -> Result<(), IoError>;
}
// offcut-mp4/src/demux.rs
pub struct SampleMeta { pub pts: Micros, pub dts: Micros, pub duration: Micros, pub is_keyframe: bool }
impl<R: RandomAccess> Demuxer<R> {
    pub fn open(reader: R) -> Result<Demuxer<R>, ContainerError>;
    pub fn video_decoder_description(&self) -> Option<&[u8]>;        // avcC payload
    pub fn audio_decoder_description(&self) -> Option<&[u8]>;        // AudioSpecificConfig
    pub fn video_sample_count(&self) -> u32;
    pub fn audio_sample_count(&self) -> u32;
    pub fn read_video_sample(&mut self, index: u32, out: &mut Vec<u8>) -> Result<SampleMeta, ContainerError>;
    pub fn read_audio_sample(&mut self, index: u32, out: &mut Vec<u8>) -> Result<SampleMeta, ContainerError>;
    pub fn keyframe_at_or_before(&self, t: TimeMs) -> u32;            // video sample index
}
// offcut-mp4/src/probe.rs
pub fn probe<R: RandomAccess>(d: &Demuxer<R>, file_size: Bytes) -> ProbeInfo;
// offcut-mp4/src/validate.rs  [ONLY]
pub fn validate_probe(p: &ProbeInfo, decode_supported: bool) -> Result<ClipInfo, RejectReason>;
```

```ts
// workers/media/import.ts
export function importToOpfs(clipId: ClipId, file: File, isCancelled: () => boolean): Promise<FileSystemSyncAccessHandle>;
// workers/media/audio-decode.ts
export function decodeAudio(handle: FileSystemSyncAccessHandle, info: ClipInfo, isCancelled: () => boolean): Promise<Float32Array>; // mono, source rate
```

Timestamps returned by the demuxer are presentation times with edit lists applied and the first presented video frame at 0. Audio is shifted by the same offset; `decodeAudio` pads leading silence or trims so that audio sample 0 is video time 0, and pads or trims the tail to the video duration. This is container-level track alignment (the two tracks rarely start and end on the same instant); it never looks at the audio content and removes nothing from inside the recording. After it, `pcm48.len() == round(ClipInfo.duration x 48)` and that length is carried unchanged to `out48` (INV-10). "The video duration" is `ProbeInfo.duration`: the presented duration of the video track with edit lists applied, rounded to the nearest millisecond; the media worker pads or truncates each resampler output by at most one sample to hold the equality exactly (v2implementation D-34).

### 15.2 Pipeline

```text
importAndProbe(clipId, file):
  if file.size > MAX_FILE_SIZE            → rejected(FileSize)        // before any copy
  handle = importToOpfs(clipId, file)     // 4 MiB chunks, cancellable
  d = Demuxer::open(handle)               // Err(NotIsoBmff) → rejected(Container); other Err → rejected(Corrupt)
  p = probe(d, file.size)
  supported = p.video is H264 && VideoDecoder.isConfigSupported({codec: p.video.codec_string, description, codedWidth, codedHeight})
  return validate_probe(p, supported)

validate_probe(p, decode_supported):      // first failing rule wins
   1 p.container == Other                              → Container
   2 p.video_tracks == 0                               → NoVideo
   3 video.codec == Hevc                               → Hevc
   4 video.codec != H264                               → VideoCodec
   5 p.audio_tracks == 0                               → NoAudio
   6 p.audio_tracks > 1                                → MultiAudioTrack
   7 audio.codec != Aac                                → AudioCodec
   8 p.duration > MAX_CLIP_DURATION                    → Duration
   9 max(display_w, display_h) > MAX_LONG_SIDE         → Resolution
  10 video.avg_fps > MAX_INPUT_FPS + INPUT_FPS_TOLERANCE → FrameRate
  11 !decode_supported                                 → DecodeUnsupported
  else Ok(ClipInfo)
```

### 15.3 Rejections (PS §9.4)

Each message is specific and actionable and lives in `messages.ts`; each fires `clip_rejected {reject_reason}` and nothing else (no file name, no content).

**What a message states** (v3implementation D-80). Each of the 14 messages is `{ title, body, action }` in `messages.rejections`; the body states the limit and the advice, interpolating the limit from the generated `LIMITS` (INV-15), for example "Offcut handles clips up to 90 seconds right now." with "Trim it in your phone's gallery or in your editor, then drop it again." as its action. The measured size or duration is not shown: `{ rejected: RejectReason }` carries no value, and `MediaWorkerApi`, `CoreApi` and the clip store do not change for it. `scripts/check-copy-codes.mjs` checks that every `RejectReason` has its message.

| Code | Trigger | Message must state | Fixture | Test |
|---|---|---|---|---|
| `REJECT_FILE_SIZE` | size > 500 MB | the 500 MB limit, how to shrink | built in the page: a `File` of 501 references to one 1 MiB `Blob` (525,336,576 bytes), dropped in a `DataTransfer` (v3implementation D-79) | `rejections.spec.ts` |
| `REJECT_CONTAINER` | not MP4/MOV | accepted containers | `rej_vp9.webm` | `probe_rejections.rs`, `rejections.spec.ts` |
| `REJECT_CORRUPT` | parse failure | file looks damaged; re-export | `rej_truncated.mp4` | same |
| `REJECT_NO_VIDEO` | no video track | needs a video | `rej_no_video.m4a` | same |
| `REJECT_HEVC` | HEVC video | PS §9.4 HEVC copy ("Most Compatible" / export as H.264) | `rej_hevc.mov` | same |
| `REJECT_VIDEO_CODEC` | other non-H.264 | export as H.264 | `rej_av1.mp4` | same |
| `REJECT_NO_AUDIO` | no audio track | needs speech audio | `rej_no_audio.mp4` | same |
| `REJECT_MULTI_AUDIO_TRACK` | more than 1 audio track | one audio track only | `rej_two_audio.mp4` | same |
| `REJECT_AUDIO_CODEC` | non-AAC audio | export with AAC audio | `rej_mp3_audio.mp4` | same |
| `REJECT_DURATION` | longer than 90 s | the 90 s limit, trim advice (PS §9.4 copy, with the limit in place of the measured duration) | `rej_too_long_91s.mp4` | same |
| `REJECT_RESOLUTION` | long side > 1920 px | PS §9.4 4K copy | `rej_4k.mp4` | same |
| `REJECT_FRAME_RATE` | average above 60 fps | record at 60 fps or lower | `rej_120fps.mp4` | same |
| `REJECT_DECODE_UNSUPPORTED` | browser cannot decode this H.264 config | re-export with standard settings | `rej_h264_444.mp4` (H.264 High 4:4:4, a real file, not a mock: `isConfigSupported` runs inside `media.worker`, where a Playwright init script does not run; v3implementation D-78). Chrome stable decodes it on D1 and on the CI runner, so its case is `test.fixme` there with the reason, and the code path is covered by the inline unit test of `validate_probe` and by `probe_rejections.rs` with `decode_supported = false` (measured in V3, docs/v3/experiments.md) | `probe_rejections.rs`, `rejections.spec.ts` |
| `REJECT_NO_SPEECH` | fewer than `MIN_WORDS` words transcribed | no English speech found | `rej_silent.mp4` | `rejections.spec.ts` (D1 only: it needs a whole ASR pass) |

### 15.4 Edge cases

- More than one video track: the first enabled track is used (assumption, TE-12).
- Rotation metadata (90/180/270): decoded frames are treated as unrotated; `video_pass.rs` applies `ClipInfo.rotation`. Orientation and the resolution rule use display size.
- VFR: `is_vfr = true` when sample durations differ by more than 1 ms; accepted; the frame-rate rule uses the average.
- Fragmented MP4 (`moof`): rejected as `REJECT_CONTAINER` at MVP (assumption, TE-12 measures how many corpus clips need it).
- `moov` at end of file: supported (random access).
- Non-speech, non-English, multi-speaker clips: not detected at import. Non-English speech usually yields `REJECT_NO_SPEECH` or poor captions; multi-speaker clips are processed as one speaker (v1.5: from V6.5, two people on screen at once are framed by who is speaking, §46.6; the transcript is still one stream). The drop zone states both constraints (PS §10 J2). Listed as open in §39.
- Sample clip ("Try with a sample clip"): fetched from the asset CDN by `asset-fetch.ts` and passed to `importClip` with `source = "sample"`.

**Acceptance.** Probe + audio extraction at most 3 s for the reference clip on R1 (PS §20.2). Corpus: at least 90% of 40+ clips export; the rest are rejected with a specific code; none crash (PS §20.1); `corpus/run_corpus.ts`.

**Contingency.** TE-12 fails (demux coverage under 90% of the corpus): replace `Demuxer` in the two workers with a maintained JS demuxer behind the same `SampleMeta` shape; `probe`/`validate_probe` keep consuming a `ProbeInfo`. PS §20.4: corpus pass rate under 90% at week 8 → remove the least valuable accepted input (first candidate: input above 30 fps) by tightening `limits.rs`.

**Seams.** v1.3 builds the long-clip and HEVC seams in §43 (`route_probe`; this section stays exact for the clip path with flags off). AV1, WebM and multi-track audio on the clip path are still not built.

---

## 16. Model Manager and Local ASR

**Model: a quantized English Whisper-family model run by ONNX Runtime Web inside `asr.worker` (WebGPU first, WASM-SIMD + threads fallback); model files are downloaded by the app from the asset CDN into OPFS, hash-verified, and served to the runtime only from OPFS. Rejected: the runtime's default hub/CDN loading (third-party hosts break the closed network list), a CPU-only WASM build (misses PS §20.2 on R1), cloud ASR as the only path (PS §9.8). Since the cloud transcription release this model is the device path and every fallback, and a signed-in person's clips go to the cloud by default (Seams below).**

### 16.1 Model manifest

`web/src/config/model-manifest.json` ships inside the app bundle, so the expected hashes are pinned by the deployed build, not fetched.

```json
{ "modelId": "asr-en-v1", "modelVersion": "<runtime+weights version string>",
  "files": [ { "path": "models/asr-en-v1/<file>", "bytes": 0, "sha256": "<hex>" } ],
  "totalBytes": 0 }
```

Candidates (TE-1, E-3, E-10 decide; values filled at M0.2): small-size English model (default candidate; the founder's decision of 2026-10-09, D-66 of `docs/v2/v2implementation.md`), base-size English model (first PS §20.4 fallback), tiny-size English model (second fallback). Constraint: `totalBytes` at most 260 MB, which is 260,000,000 bytes (PS §10 J4, §20.2). The J4 copy interpolates `totalBytes` rounded to 10 MB. `Transcript.model_version` is the manifest's `modelId`: `AsrWorkerApi.load` carries `modelId` and no version, and workers do not import the manifest (v2implementation D-50).

### 16.2 Interfaces

```ts
// models/model-manager.ts
export function inspect(): Promise<"absent" | "partial" | "ready">;
export function ensureReady(onProgress: (p: { done: Bytes; total: Bytes; etaSecs: number | null }) => void,
                            signal: AbortSignal): Promise<void>;
export function cacheInfo(): Promise<{ modelId: string; modelVersion: string; bytes: Bytes }>;
export function clear(): Promise<void>;
// models/download.ts
export function fetchRanged(file: ManifestFile, o: { resumeFrom: Bytes; signal: AbortSignal;
                            onBytes: (n: Bytes) => void }): Promise<void>;
// workers/asr/whisper-runtime.ts
export type RawWord = { text: string; startMs: TimeMs; endMs: TimeMs; confidence: Confidence };
export function loadModel(modelId: string, backend: "webgpu" | "wasm"): Promise<"webgpu" | "wasm">;
export function transcribe(pcm16: Float32Array, onWindow: (done: number, total: number) => void,
                           isCancelled: () => boolean): Promise<RawWord[]>;
export function unloadModel(): Promise<void>;
```

### 16.3 Download and verify

```text
ensureReady:
  navigator.storage.persist()                         // best effort, once
  for file in manifest.files:
     part = opfs "models/<modelId>/<name>.part"; have = part.size
     while have < file.bytes:
        GET <ASSET_BASE_URL>/<file.path>  Range: bytes=have-(have+MODEL_PART_BYTES-1)   // MODEL_PART_BYTES = 8 MiB
        status 206 → append; have += n
        status 200 → truncate part to 0, write full body (server ignored Range)
        failure → retry up to MODEL_RETRIES = 3 with backoff 1 s, 3 s, 9 s; then state partial + E_MODEL_DOWNLOAD
     state verifying: Sha256Stream over the part file in 4 MiB reads
        match → rename to final name;  mismatch → delete, E_MODEL_HASH
  state ready
```

### 16.4 Transcription

```text
loadModel: configure the runtime so that                  // exact option names confirmed in TE-1
   - remote model loading is disabled
   - the model cache is asr/model-cache-adapter.ts (reads OPFS final files only; a miss throws, never fetches)
   - runtime .wasm files load from "/ort/<runtime version>/" on the app origin (versioned: vercel.json marks /ort/* immutable; v2implementation D-37)
   - backend = "webgpu"; on failure → "wasm" with threads = clamp(hardwareConcurrency - 2, 1, 4)
transcribe(pcm16):
   windows of ASR_WINDOW = 30 s with ASR_OVERLAP = 5 s; word-level timestamps requested
   merge overlaps: keep the word instance farther from a window edge
   post-process (asr/word-timestamps.ts):
      clamp to [0, duration]; enforce start_i >= end_(i-1); min word length ASR_MIN_WORD = 40 ms
      confidence = mean token probability when the runtime exposes it (TE-2), else Confidence(1.0)
   → RawWord[] → wasm normalize_transcript(raw, modelVersion) → Transcript       (§17.2)
   words.len() < MIN_WORDS → rejected(NoSpeech)
```

### 16.5 Lifecycle and ownership

- `asr.worker` owns the session. `unload()` disposes it and awaits GPU buffer release before `render.worker` opens its device (§31).
- Model files are immutable; a new `modelId` in the manifest moves the old directory to deletion on next start.
- A partial download survives reloads and tab close; `inspect()` reports `partial` when any `.part` exists.

### 16.6 Edge cases

- Storage eviction by the browser between sessions: `inspect()` returns `absent`; J4 runs again.
- Quota too small for the model: `E_MODEL_STORAGE` after evicting all clips.
- WebGPU backend produces NaN logits on a driver: detected as empty output with non-silent audio (RMS above -50 dBFS) → automatic retry on `wasm`, backend reported in `stage_timing.asr_backend`.
- Hallucinated text over silence at clip end: not removed automatically at MVP; the user hides such words in the caption editor. E-10 counts occurrences and decides whether a filter is needed (§39).

**Acceptance.** Median transcription at most 20 s for the reference clip on R1 (PS §20.2, E-3; a target that the small-size model of V2 does not meet, accepted up to 145 s for 60 s by D-67 of `docs/v2/v2implementation.md`). WER at most 12% and median caption edits at most 8 per 60 s on the 20-clip set (E-10). First-run download plus initialization at most 90 s at 25 Mbps (PS §20.2). Second session skips the download (PS §20.1); `model-download.spec.ts`. No request to a host outside §24.1; `privacy-network.spec.ts`.

**Contingency (PS §20.4).** ASR median above 145 s for a 60 s clip on R1 (D-67; the line was 40 s) → ship a smaller model (base-size first, then tiny-size) and keep the cloud fallback as a P1 trigger. TE-1 fails (runtime cannot be confined to OPFS, or no word timestamps) → second candidate runtime (Rust inference compiled to WASM/WebGPU) behind the same `whisper-runtime.ts` interface; if that also fails, word timestamps are approximated by forced alignment of segment text to energy onsets and E-7 decides whether events remain viable.

**Cloud transcription is built** (cloud transcription release, cloudimplementation §8). `asr.worker` gains `encodeForCloud` (`pcm16` to Ogg Opus, 32 kbps, through WebCodecs and `workers/asr/ogg-opus.ts`) and `fromCloud` (the provider's words through D-382's silence guard, then the same `postProcess` and `normalizeTranscript` as device words) beside an unchanged `transcribe`. `usecases/transcribe-cloud.ts` sends the file to our API, so no browser host is added. `run-pipeline.ts` chooses cloud once per run: the flag `cloud_transcribe` on, `/healthz` not saying no, the drop zone's card on Cloud, and a person signed in; any failure finishes the clip on the device. **v1.4:** a long source is always transcribed in the cloud, in checkpointed windows, and pauses instead of falling back to the device (§43.6). **Seams (not built now).** Other languages (P2): one manifest per language. Local LLM (P2): a separate manifest entry and worker. v1.3: the manifest becomes one file per model under `config/models/`, the ASR file unchanged (DC-4); the face and person models run in `asr.worker` through their own `*-runtime.ts` (DC-10).

---

## 17. Transcript Normalization and Visual-Event Detector

**Model: deterministic rules over word-timestamped, number-normalized text with a per-kind confidence score; a candidate below its threshold is never constructed as an event (PS §8 principle 2, §12.4). Rejected: local LLM, cloud LLM, user-inserted events (PS §12.4).**

### 17.1 Interfaces

```rust
// offcut-text
pub fn normalize_transcript(raw: Vec<RawWord>, model_version: &str) -> Transcript;
pub fn segment_sentences(words: &[Word]) -> Vec<Sentence>;
pub fn parse_quantity(tokens: &[Token]) -> Option<(usize, Quantity)>;      // tokens consumed, value
pub fn format_quantity(value: f64, unit: &Unit) -> String;                 // numbers.rs is the only formatter
// offcut-detect
pub fn detect(t: &Transcript, p: &Prosody, edits: &BTreeMap<WordIdx, String>, cfg: &DetectorConfig) -> Vec<DetectedEvent>;
pub fn redetect_sentence(t: &Transcript, p: &Prosody, edits: &BTreeMap<WordIdx, String>, s: SentenceIdx,
                         prev: &[DetectedEvent], cfg: &DetectorConfig) -> Vec<DetectedEvent>;
pub fn event_id(kind: EventKind, anchors: WordRange) -> EventId;           // FNV-1a 64 over (kind, start, end)
```

The event-id hash input is 9 bytes: the kind as `u8` (NumberReveal 0, ListReveal 1, FromTo 2, KeywordPop 3), then `anchors.start` and `anchors.end` as little-endian `u32` (v2implementation D-47). Ids are stored from V3 on, so this layout is fixed. `Token` and `TokenKind` are defined in `offcut-text/src/tokenize.rs` (v2implementation §8.2, D-42); the recognizer writes an amount such as "$12,000" as two words, `$12` and `,000`, and `parse_quantity` reads that split form as one quantity (v2implementation §8.4).

### 17.2 Normalization

- **Sentences.** A boundary after terminal punctuation (`.`, `?`, `!`) or a gap of at least `SENTENCE_GAP = 700 ms`; a sentence longer than `SENTENCE_MAX_WORDS = 40` is split at its longest internal gap, the earliest of equal gaps, each half checked again (assumptions, E-10; the tie-break makes one-word sentences when every gap is equal, v3implementation §23 item 28, open).
- **Numbers (`numbers.rs`).** Parses: digit forms with separators and decimals; spelled cardinals from zero to trillions ("ten thousand" → 10,000, PS §9.6; the grammar is in `cardinals.rs`, v3implementation D-92); "point" decimals; magnitude suffixes k, m, b, "grand"; currency symbols and words (USD, EUR, GBP, INR); percent; multipliers ("3x", "ten times"); units from `units_lex.rs` (time, data size, count nouns). Not parsed: fractions in words, ranges, dates, ordinals, Indian digit grouping ("2,00,000": the number is not shown rather than misread; v3implementation §23 item 16, open), terabytes (`Unit` has no variant; item 17, open).
  - **Word lists** (v3implementation D-84). The unit, currency, magnitude, percent and count-noun lists and `COMPARATIVES` are the sorted `const` slices of `offcut-text/src/units_lex.rs` (v3implementation §8.4), all (assumption, M1.3), re-checked by E-7, matched against whole lower-cased tokens, never read from a file at run time. `units_lex` is public only because `offcut-detect` reads `COMPARATIVES`.
  - **A wider `Digits` token.** After at most one leading currency symbol (`$ € £ ₹`), at most one trailing sign or suffix (`% k m b x`) and thousands separators are removed, the rest parses as a decimal: `10k`, `$1.5m`, `3x`, `€500` are one `Digits` token each. A clock time (`10:30`) and a range (`5-10`) stay words.
  - **Count nouns** (v3implementation D-101). A number followed by a word of `COUNT_NOUNS` gets `Unit::Count { noun }`, with the noun as spoken. The list nouns of `offcut-detect/src/config.rs` (reasons, ways, tips, …) are never count nouns, so "three reasons" stays a bare 3 for the list announcer.
  - **Two ambiguous unit words** (v3implementation D-102). `Unit::Times` is given only by an `x` written after digits (`3x`) or by "times" followed by a word of `COMPARATIVES` ("ten times faster"); "three times" alone is no unit. "pound" and "pounds" are not a currency word (weight is as likely); `£` and "gbp" are. (assumption, M1.3), re-checked by E-7.
  - **Split groups and a split decimal point.** The recognizer writes "$12,000" as `$12` `,000` and "4.7 minutes" as `4` `.7` `minutes`. After a plain whole number, `parse_quantity` joins split thousands groups and then at most one word of a point and digits only, straight after the number, as its fraction, which ends the number ("4.7 minutes", "1.5 million views"; v2implementation D-42, the founder's decision of 2026-10-18 in the V3 changelog). A point that ends a word ("3." then "5") stays a full stop; a fraction with a sign or suffix (`.7%`), a word in between, or a core that already has a point or a sign is not joined.
  - **A number never crosses a sentence boundary** (v3implementation D-104). `normalize_transcript` parses each sentence's tokens separately, so a `NormalizedSpan` lies inside one sentence.
- **Display.** `Quantity.display` is produced only by `format_quantity`; captions show it in place of the spoken words of the span.
- **Edits.** Detection and captions read effective text: `edits[idx]` when present, else `Word.text`. The transcript itself is immutable. An edited number is read again (v3implementation D-109): `number::find`, `from_to::find` and `list::find` read quantities from the effective tokens of each sentence with `parse_quantity`, not from `Transcript.numbers`, so a span whose words were edited yields the quantity of the new text and a hidden word inside it yields none.

### 17.3 Detection

```text
detect(t, p, edits, cfg):
  toks  = effective tokens (lower-cased, punctuation split)
  cands = from_to::find(toks) ++ list::find(toks) ++ number::find(toks) ++ keyword::find(toks, p)
                                                      // each reads the quantities of the effective tokens, per sentence
  cands = exclusions::apply(cands, toks, t, edits)    // may zero a score
  cands = cands.filter(c => settled(c.score) >= cfg.threshold[c.kind])
  kept  = resolve::resolve(cands, cfg)                // §17.5
  kept.map(c => DetectedEvent { id: event_id(c.kind, c.anchors), enabled: true, confidence: c.score, .. })

redetect_sentence(.., s, prev, ..):
  s out of range → return prev
  scope = sentence s ∪ every sentence whose start_ms lies in (start_ms(s), start_ms(s) + LIST_WINDOW_MS)
          ∪ the sentences of every prev event that shares a word with the scope, repeated until none crosses its edge
  fixed = prev.filter(e => e.anchors share no word with scope)
  new   = steps 1-5 of detect over the whole transcript, the candidates wholly inside scope
  return fixed ++ resolve(new, fixed)                 // ids stable where kind and anchors are unchanged
```

**As built in V3.**
- **`settled`** (v3implementation §9.2 step 5, V3 changelog Prompt 77): a score within `SCORE_NOISE` (1e-6) of a whole hundredth is taken as that hundredth before the threshold, because sums of the config's two-decimal values carry `f32` noise (0.65 + 0.20 is 0.84999996, under 0.85). A score really between two hundredths (0.795) is not moved, and `score >= threshold` is otherwise unchanged, which errs toward a missed event rather than a wrong one. An event's confidence is the settled score, so INV-6 holds.
- **The re-detection scope** (v3implementation D-86): `LIST_WINDOW` is a time (45 s), not a sentence count, and a list announced in `s` may find its markers up to 45 s later. The scope is always whole sentences, and it closes over the events of `prev` that cross its edge: an event that reached into the scope without lying wholly inside it would otherwise be neither kept nor detected again, and be lost on an edit elsewhere (V3 changelog, Prompt 78). `resolve` runs with the events outside the scope fixed, so the rules of §17.5 hold for the whole clip. Measured: 142-290 µs per call on a 157-word transcript on D1, release (`redetect.rs`).

### 17.4 Scoring (`config.rs`; all values are (assumption), tuned at M1.3 against `fixtures/labeled/` and re-checked in E-7)

Scores are clamped to 1.0 and multiplied by the minimum ASR confidence of the anchor words.

| Kind | Candidate requires | Score | Threshold |
|---|---|---|---|
| FromTo | Two quantities in one sentence with compatible units, different values, at most `FROMTO_MAX_GAP = 8` words apart | 0.55 base; +0.25 "from … to" frame, or +0.15 change-verb frame ("went", "dropped", "grew", "down", "up" … "to"); +0.15 both carry the same explicit unit, or +0.05 one inherits the other's; +0.05 gap of 6 words or fewer | 0.85 |
| ListReveal | Announcer (count 2-7 + list noun from the lexicon) followed by that many ordered markers within `LIST_WINDOW = 45 s`; or at least 3 ordered ordinal-word markers starting at "first" | Announcer form: 0.60 base; +0.30 all markers found in order; +0.10 markers are ordinal words. Marker-only form: 0.65 base; +0.20 when no gap between consecutive markers exceeds `LIST_MARKER_GAP_MS = 15,000` | 0.85 |
| NumberReveal | A `NormalizedSpan` not consumed by FromTo or a list announcer/marker | 0.50 base; +0.30 currency, percent, `Times`, a time or data unit, or a count of at least `COUNT_UNIT_MIN = 100` (v3implementation D-101); +0.20 magnitude at least 1,000 or a suffix, magnitude word or "grand"; +0.10 `energy_z >= 1.0` on any word of the span | 0.85 (measured in V3, docs/v3/experiments.md: raised from 0.80 by the precision gate on 31 labeled files) |
| KeywordPop | Content word of at least 4 letters, not a stop word, outside every other event's anchors | 0.50 base; +0.20 x clamp(`energy_z` - 1, 0, 1); +0.20 x clamp(`pitch_z` - 1, 0, 1); +0.20 preceded by an intensifier or in the emphasis lexicon | 0.80 |

**Thresholds in use** (`offcut-detect/src/config.rs`, V3): FromTo 0.85, ListReveal 0.85, NumberReveal 0.85, KeywordPop 0.80. Every other default is v3implementation §9.1's table. A quantity under 1,000 with its unit ("40 seconds", "$500", "120 clients") scores 0.80 and is shown only when said with emphasis (0.90); "3,000 hours", "$12k", "$2k" score 1.00.

**Lexicons** (v3implementation D-84). `LIST_NOUNS`, `ORDINAL_MARKERS`, `CHANGE_VERBS`, `INTENSIFIERS`, `EMPHASIS_WORDS`, `STOP_WORDS`, `LABEL_STOP_VERBS`, `FILLERS`, `VERSION_CONTEXT`, `VERSION_AFTER` and `CLOCK_WORDS` are the eleven sorted `const` slices of `offcut-detect/src/config.rs`, listed word by word in v3implementation §9.1, all (assumption, M1.3), re-checked by E-7. `CLOCK_WORDS` holds `a.m` and `p.m`, the forms `tokenize` gives "a.m." and "p.m." (its trailing point is split as punctuation). "always" and "never" are not emphasis words.

**FromTo units** (v3implementation D-85). Two quantities are compatible when, after inheritance (a `Unit::None` takes the other's unit), they have the same `Unit` variant, or both are in one dimension group: time (`Milliseconds` to `Years`), data (`Bytes` to `Gigabytes`); `Count { noun }` only with the identical noun; two different currencies never. "+0.15 same explicit unit" applies when both carry the identical `Unit` variant from their own words, `None` and `None` included (so "from 2019 to 2024" scores 1.00 before the year exclusion); "+0.05 inherits" when exactly one is `None`. The change-verb frame looks for a `CHANGE_VERBS` word among the 3 tokens before the first quantity in its sentence, or between the two, with "to" right before the second. For each first quantity only the best second is kept (highest score, then nearest). The side that inherited is formatted again with its unit ("90" becomes "90 minutes", v3implementation D-106). `FromTo.label` is `None` in V3; no label row is drawn for it.

**List markers** (v3implementation D-105). A marker is an ordinal word (`first` … `seventh`, `firstly`, `secondly`, `thirdly`), the phrase "number" + a cardinal 1-7, or a cardinal 1-7 in digits or words followed by `,` `.` or `:`; it counts only as the first effective word of a sentence, after any punctuation and words of `FILLERS` ("So, first, …"). The markers of a list name 1, 2, 3, … one after the other (a stray marker in between ends the run), each in a later sentence and each starting before the announcer's start + `LIST_WINDOW_MS`. **A count followed by a list noun claims the markers after it, whatever its count** (V3 changelog, Prompt 76): only an in-range count with all its markers is a candidate, and the marker-only form skips claimed markers, so "eight reasons" or "five tips" with four markers gives no list rather than a partial one. A list's score, like every kind's, is multiplied by the least confidence of its anchor words.

**Candidate parts** (v3implementation D-106). `Candidate` carries `parts: Vec<WordRange>`, the word range of each quantity or marker it is built from (NumberReveal: its span; FromTo: from and to; ListReveal: the announcer's count and every marker; KeywordPop: none), and `sentence`, the sentence of its first word (for §17.5 step 4). Both are crate-internal. `exclusions::apply` inspects the parts.

**Exclusions (`exclusions.rs`, score set to 0).** A part is checked with the quantity's own unit, before a FromTo's inheritance. X1-X5 apply to NumberReveal and FromTo parts, X6 to NumberReveal parts, X7 to every kind; a ListReveal's announcer count is checked by X7 only, so it is exempt from X5.

| # | Rule (named constants in `config.rs`) |
|---|---|
| X1 | Year: `Unit::None`, a whole number in `YEAR_FIRST..=YEAR_LAST` (1900-2099), one plain `Digits` token without a separator |
| X2 | Version or product: a `VERSION_CONTEXT` word right before the part, a `VERSION_AFTER` word right after it, or `VERSION_DOTS` (2) decimal points in its text ("Python 3.12", "iPhone 15 Pro", "version 2") |
| X3 | Clock time: a `CLOCK_WORDS` word right after the part ("at 5 pm"); "10:30" is never a quantity |
| X4 | Phone-like: one plain `Digits` token of `PHONE_DIGITS_MIN` (7) or more digits, `Unit::None` |
| X5 | Bare small number: `Unit::None` and `abs(value) < BARE_NUMBER_MIN` (10) |
| X6 | List marker: the part's words equal a marker's words |
| X7 | A hidden word: an edit that is empty or only spaces for any word inside the candidate's anchors (§17.6); the only rule that reads `edits` itself |

**Derived fields** (v3implementation D-107).
- `NumberReveal.label`: the effective text of up to `NUMBER_LABEL_MAX_WORDS = 3` Word tokens after the span, in the same sentence, stopping at punctuation or at a word of `LABEL_STOP_VERBS`; joined by one space; `None` if empty ("3,000 hours editing by hand" → `editing by hand`).
- `ListReveal.header`: `"{count} {NOUN}"` with the noun upper-cased ("3 REASONS") in the announcer form; empty in the marker-only form (the header row is not drawn).
- `ListItem.text`: up to `LIST_ITEM_MAX_WORDS = 5` tokens after the marker (Word tokens and numbers in digits), leading fillers removed, stopping at `.` `?` `!` `;` `:`, at a `,` once 2 words are taken, at the next marker or at the sentence end; `ListItem.at` is the start of the marker's first word.

### 17.5 Resolution (`resolve.rs`)

```text
priority: FromTo > ListReveal > NumberReveal > KeywordPop
1 drop a candidate whose anchors intersect a higher-priority kept candidate
2 overlay kinds (FromTo, ListReveal, NumberReveal): display window = [span.start - 150 ms, span.end + hold(kind)]
  windows must not overlap; on overlap keep higher priority, then higher score, then earlier
3 at most OVERLAY_MAX_PER_10S = 3 overlay events in any 10 s window (lowest score dropped)
4 KeywordPop: at most one per sentence (highest score); at least KEYWORD_MIN_GAP = 2500 ms apart
```

### 17.6 Edge cases

- "from 2019 to 2024": both excluded as years → no FromTo.
- "three reasons" with fewer than three markers found: no event (a partial list is worse than none).
- "$2k to $20k" without "from" and without a change verb: 0.55 + 0.15 + 0.05 = 0.75 → not shown.
- A word edit that changes a number re-parses that sentence (v3implementation D-109). The event keeps its id when its kind and anchors are unchanged (v2implementation D-47 hashes the kind and the anchors only, and an edit of one word keeps the anchors), and so keeps its toggle in `EditState.event_overrides`: a number the user switched off stays off when its text is corrected ("$10,000" edited to "$20,000" gives `$20k` with the old id; decided 2026-10-18, V3 changelog open item 14). An edit that changes the kind or the anchors gives a new id.
- A hidden word (`""` edit) inside an event's anchors removes the event.
- Empty transcript or zero events: `detect` returns an empty list; the UI shows `NO_EVENTS_FOUND` (PS §10 J7).

**Acceptance.** Precision at least 0.90 per run over at least 30 labeled transcripts covering all four kinds (PS §20.1; written as text and frozen before the rules, v3implementation D-87; V3 read 0.98 on 31 files, docs/v3/experiments.md); `precision_recall.rs` fails below it and prints per-kind precision, recall and a threshold sweep. A prediction is a true positive when kind matches, anchor ranges overlap by at least 50% intersection-over-union, and for FromTo/NumberReveal the parsed values are equal. Same inputs → identical output; `determinism.rs`. Detection at most 1 s for 150 words on R1 (PS §20.2). `redetect_sentence` at most 50 ms (assumption, M1.5).

**Contingency.** Precision under 0.90: raise the per-kind threshold to the lowest value in the sweep that reaches 0.90; if a kind's recall then falls under 0.30 (assumption, E-7), keep it shipped but report it as the E-7 "tune rules or reconsider the wedge" input.

**Seams (not built now).** P1 event kinds (Warning, Question, Quote, Checklist, Timeline): a new `EventKind` variant, one file in `offcut-detect/src/`, one in `offcut-scene/src/events/`, labeled fixtures. Local LLM (P2): another producer of `Vec<DetectedEvent>` behind `RenderWorkerApi.detect`.

---

## 18. Audio Enhancement

**Model: offline block processing of mono 48 kHz f32 in Rust: high-pass → denoise → compressor → loudness normalization → limiter → `out48`. The chain changes the characteristics of the signal, never its timeline: `out48` has exactly as many samples as the decoded input, and sample `i` of the output is sample `i` of the recording (INV-10). Rejected: a Web Audio / `OfflineAudioContext` graph (browser-dependent output, not testable natively), stereo processing (one speaker, PS §9.4), and any silence detection, pause classification, pause shortening or splicing inside this chain. v1.2: pause decisions come from the edit plan and splicing happens after the chain, in `program::assemble` (§40.6); the chain itself stays length-preserving.**

**Enhancement is not editing.** This crate does the left column only.

| | Audio enhancement (built) | Audio editing (not built, not planned for the MVP) |
|---|---|---|
| Timeline | Same | Different |
| Signal | Different quality | Same or different |
| Examples | High-pass, denoise, compression, loudness normalization, limiting, resampling to 48 kHz, channel normalization to mono | Remove silence, cut or shorten a pause, remove a breath, filler word, sentence or mistake, trim the start or end, collapse gaps |

```text
Original:  0 s ── talking ── 5 s pause ── talking ── 90 s
Enhanced:  0 s ── talking ── 5 s pause ── talking ── 90 s      the pause may sound cleaner; it is still 5 s long
```

### 18.1 Interfaces

```rust
pub enum Flow { Continue, Cancel }
pub fn resample_mono(input: &[f32], from: Hz, to: Hz) -> Vec<f32>;
pub fn run_chain(pcm48: &[f32], cfg: &ChainConfig, on_block: &mut dyn FnMut(SampleCount) -> Flow)
    -> Result<Option<ChainOutput>, DspError>;                              // None = cancelled
pub struct ChainOutput { pub samples: Vec<f32>, pub loudness: Lufs, pub peak: Dbfs }   // samples.len() == pcm48.len(); this is out48
pub fn measure_prosody(out48: &[f32], words: &[Word], sentences: &[Sentence]) -> Prosody;   // reads only
```

`resample_mono` changes the sample rate, not the duration: `output.len() == round(input.len() x to / from)`. No function in this crate takes word timings and returns audio, and none returns audio of a different duration than it was given.

**The binding takes no configuration** (v3implementation D-110). `offcut-wasm-core/src/audio_api.rs::run_chain(pcm48, on_block)` runs `ChainConfig::default()`: `AudioWorkerApi.runChain` (§14.2, frozen) has no field for a `cfg`, so `ChainConfig::default()` is the only configuration the app runs, and `offcut-dsp/src/config.rs` is the one place where TE-13's fallback (`denoise: false`) is written. PCM crosses the WASM boundary by copy (in: `&[f32]`; out: a new `Float32Array`), 2 x 17.3 MB for 90 s; between threads it is transferred, never copied (§31).

**The limiter reports its guard.** `limiter.rs::limit(samples, ceiling, rate) -> usize` limits in place and returns how many samples its final guard clamp had to touch (0 when the gain did its work), so `loudness_target.rs` can assert the clamp count.

### 18.2 Pipeline

```text
run_chain (blocks of 1 s; cancel check per block):
  1 high-pass      2nd-order Butterworth at HPF_HZ
  2 denoise        RNNoise-class, 10 ms frames, wet mix DENOISE_MIX           (skipped when cfg.denoise == false)
  3 compressor     feed-forward, soft knee
  4 loudness       integrated loudness (BS.1770 gating), measured as dual mono: the mono programme as it will sound on
                   two loudspeakers, so -14 means a stereo file (v5implementation D-204) → gain to TARGET_LOUDNESS,
                   gain capped at MAX_GAIN_DB; measured loudness below ABSOLUTE_SILENCE → gain 0 dB
  5 limiter        look-ahead, ceiling LIMIT_CEILING
  6 re-measure     if |loudness - TARGET| > 0.5 LU → one corrective gain pass + limiter
  every stage is length-preserving and delay-compensated: the denoiser's frame delay and the limiter's look-ahead
  are removed before the next stage, and the last partial block is processed, not dropped
  → out48, with out48.len() == pcm48.len()          (worked exemplar: §34.3)

measure_prosody:
  per word: energy = RMS dB over [start, end]; pitch = median F0 by autocorrelation in 60-400 Hz (unvoiced → none)
  z-scores against the words of the same sentence; sentences with fewer than 3 words or zero variance → 0.0
```

**Prosody parameters** (v3implementation D-96, D-108; all (assumption, M1.2)).
- **Energy:** RMS in dB over the word's samples, floor `ENERGY_FLOOR_DB = -100 dB`.
- **Pitch on an 8 kHz copy:** each word's samples are decimated by `PROSODY_DECIMATE = 6` with a 6-tap box filter. The normalized autocorrelation runs over lags `PITCH_MIN_LAG = 20` to `PITCH_MAX_LAG = 134` (400 to 60 Hz) in `PITCH_FRAME = 320`-sample frames (40 ms) with a `PITCH_HOP = 80`-sample hop (10 ms). A frame is voiced when its peak is at least `VOICING_MIN = 0.45` and its level at least `VOICED_FRAME_MIN_DB = -60 dB`. The period is the first autocorrelation peak (shortest lag) that comes within `FIRST_PEAK_SHARE = 0.9` of the best one, refined by parabolic interpolation: a periodic voice correlates almost as well at two and three periods, and the plain maximum halves the pitch at random (V3 changelog, Prompt 68). The word's F0 is the median of its voiced frames; no voiced frame gives no pitch. At 48 kHz the autocorrelation over the reference clip's words would cost about 6 x 10^9 multiply-adds, far over the prosody allowance (under 1 s, §8); at 8 kHz about 2 x 10^8, and F0 lies under 400 Hz, so the decimation keeps what is measured.
- **z-scores:** of energy and of `log2(F0)`, within the sentence over the words that have a value; a word without pitch gets `pitch_z = 0.0`; a sentence of fewer than `PROSODY_MIN_WORDS = 3` such words, or of zero variance, gives 0.0 for all its words.

### 18.3 Constants (`config.rs`; all (assumption), tuned at M1.2 and in E-2)

| Constant | Default | Note |
|---|---|---|
| `TARGET_LOUDNESS` | -14.0 LUFS | The target PS §20.1 delegates to this document |
| `LIMIT_CEILING` | -1.0 dBFS | |
| `MAX_GAIN_DB` | 24 dB | Avoids amplifying a near-silent clip |
| `ABSOLUTE_SILENCE` | -70 LUFS | |
| `HPF_HZ` | 80 Hz | |
| `DENOISE_MIX` | 0.85 | |
| Compressor | threshold -18 dBFS, ratio 3:1, attack 5 ms, release 80 ms, knee 6 dB | |
| Limiter look-ahead | 5 ms | Compensated; adds no delay to `out48` |
| `ALIGN_TOLERANCE` | 1 ms (48 samples) | Largest allowed shift of any signal feature between input and output |
| `DENOISE_DELAY_SAMPLES` | 480 (10 ms, one RNNoise frame) | The denoiser's delay, the same for every impulse, compensated exactly: measured in V3 by `impulses_stay_aligned`, docs/v3/experiments.md (v3implementation D-89) |

No constant in this table is a silence threshold for cutting, a minimum pause, a kept-pause length or a crossfade. `ABSOLUTE_SILENCE` only stops the loudness stage from applying gain to a clip with no measurable programme.

### 18.4 Lifecycle and edge cases

- `audio.worker` keeps `out48` between `runChain` and `measureProsody`, then transfers it to the main thread. The worker holds no audio after the pipeline and has no later stage; no review edit re-runs audio.
- A long pause, many pauses, or a clip that is mostly silence: processed like any other samples. The pause keeps its position and its length; denoise may lower the noise floor inside it.
- Breaths, laughs, filler words and hesitations are part of the take and are left in place.
- The whole clip measures below `ABSOLUTE_SILENCE`: gain 0 dB, the chain succeeds. Silence never produces `E_DSP` (§11.3).
- Non-finite samples in input → `DspError::NonFinite` → `E_DSP`.
- Clipping source audio: limiter output never exceeds the ceiling; no de-clip is attempted.

**Acceptance.** Output integrated loudness within ±1 LU of `TARGET_LOUDNESS` (PS §20.1): `loudness_target.rs` on five synthetic and three speech fixtures, and independently by `verify_mp4.py`. No click at block boundaries: `chain_no_clicks.rs` asserts the sample-to-sample step across every 1 s block boundary is at most 2x the local RMS (assumption, M1.2). Duration and alignment (INV-10), `duration_preserved.rs`: `out48.len() == pcm48.len()` for every fixture and for random lengths including non-multiples of the block and frame sizes (proptest); the regression layout 0-5 s speech, 5-10 s silence, 10-20 s speech comes out as 20 s with speech, silence and speech at the same positions (never 15 s); an impulse train through the chain stays within `ALIGN_TOLERANCE`; a clip that is 95% silence and a clip of pure digital silence both return the input length. Chain at most 4 s for the reference clip on R1 (PS §20.2).

**Contingency (PS §20.4, TE-13).** Denoise exceeds the 4 s budget on R1 → `ChainConfig.denoise = false` (high-pass + loudness + limiter only); decided at M1. Measured in V3 on D1, not R1 (v3implementation D-74): 828 ms per 60 s alone, a median of 1,963 ms per 60 s in the bench beside the recognizer; denoise stays on (docs/v3/experiments.md, TE-13).

**Seams (not built now).** User-supplied music with ducking (P1): a `mix_bgm` step after `run_chain` and a `BgmTrack` record; it does not exist in code. Future versions may introduce explicit user-controlled timeline editing; the MVP deliberately does not implement automatic timeline modification, and no type, argument, message or flag is reserved for it.

---

## 19. Scene Graph, Typography and Renderer

**As built in V4 (§40.5; the V4 implementation plan's D-125 to D-143, D-167 to D-174).** `SceneInput` takes a `ValidPlan`, a `TimeMap`, a `Theme` and a `FontSet` instead of `events`; `frame_at` takes `OutMs`; the video is a `DrawCmd::Video` in the list, and `Scene::crop()` is gone; `FontId` became `FontKey` in a `FontSet`; styles reference theme roles. §19.2 to §19.4 below are V4's, copied from the code; the v1.1 signatures they replace are in the history of this file.

**Model: a deterministic scene is compiled on the CPU into a backend-neutral `DisplayList` for any output time; Vello on WebGPU draws the list to an overlay texture; one wgpu pass composites it over the rotated, cropped source frame. Rejected: DOM/CSS overlays captured to canvas (not deterministic, not exportable frame-accurately), layout in JavaScript (two implementations of one rule), Canvas2D as the primary path (PS §9.8 keeps it as the fallback).**

### 19.1 Coordinate system (`safe_area.rs`, the only place)

- Logical canvas: 1080 x 1920 logical pixels (lp). All layout is in lp. Output scale = `profile.width / 1080` (Creator 1.0, Free 2/3, Preview 1/2).
- Safe area insets (assumption, E-2): top 250, bottom 420, left 60, right 120.
- `EVENT_ZONE`: y 860-1180. Captions (V4, D-171, `placement.rs`): the caption block's centre moves along a track whose bottom is `CAPTION_BOTTOM` (the safe area's bottom less `WATERMARK_BAND` = 56 lp less 140 lp: y 1304) and whose top is y 390; `EditState.caption_position` places it (0.0 top, 1.0 bottom, the default), with snaps Top 0.15, Middle 0.55, Bottom 1.0. The lines are centred in the block at and above the track's middle, and below it sit lower, up to half the room under them at Bottom (DC-13, `BOTTOM_LINE_DROP`). The visuals' zone is moved below the captions when they sit at the top. `WATERMARK_ANCHOR`: bottom-left of the safe area, in its own band under the captions (D-143).

### 19.2 Interfaces

```rust
// offcut-scene
pub struct SceneInput<'a> {
    pub transcript: &'a Transcript, pub plan: &'a ValidPlan, pub edit: &'a EditState, pub clip: &'a ClipInfo,
    pub profile: &'a ExportProfile, pub time_map: &'a TimeMap, pub theme: &'a Theme, pub fonts: &'a FontSet,
}
pub fn build_scene(input: SceneInput<'_>) -> Result<Scene, SceneError>;
impl Scene {
    pub fn duration(&self) -> DurMs;                                  // the time map's program length
    pub fn frame_count(&self) -> u32;                                 // ceil(duration_ms * OUTPUT_FPS / 1000); 2,700 for 90 s
    pub fn frame_at(&self, t: OutMs) -> DisplayList;                  // pure: the video, the chunk shown, the items shown, the watermark
    pub fn source_at(&self, t: OutMs) -> SrcPos;                      // which source frame the main video slot shows
    pub fn hit(&self, t: OutMs, x: f32, y: f32) -> Option<Hit>;       // D-174: a word of the chunk shown, the caption block, or the video
    pub fn word_box(&self, t: OutMs, word: WordIdx) -> Option<Rect>;  // the glyph box hit() finds a word by (D-183)
    pub fn chunk_ranges(&self) -> Vec<(WordRange, OutMs)>;            // the chunks as the transcript editor shows them (D-182)
    pub fn caption_words(&self, t: OutMs) -> Vec<(String, Rgba)>;     // the chunk shown, as drawn: development builds' mirror (D-155)
    pub fn caption_block(&self) -> Rect;                              // the box the caption lines are centred in (D-171)
    pub fn fonts(&self) -> &FontSet;
    pub fn summary(&self) -> ChangeSummary;                           // the items drawn, by kind (D-144)
}
pub struct DisplayList { pub cmds: Vec<DrawCmd> }                     // Serialize: the Canvas2D seam
pub enum DrawCmd {
    FillRect  { rect: Rect, radius: f32, color: Rgba },
    FillPath  { path: Vec<PathEl>, color: Rgba },
    GlyphRun  { font: FontKey, size: f32, glyphs: Vec<PositionedGlyph>, fill: Rgba, stroke: Option<Stroke>,
                transform: Affine, text: String },                   // stroked, then filled (D-68); text for the Canvas2D seam
    PushLayer { opacity: f32, clip: Option<Rect> },
    PopLayer,
    Video     { slot: VideoSlot, src: CropRect, dst: Rect, opacity: f32, mask: Option<Vec<PathEl>> },
                                                                      // src in source display pixels; dst an axis-aligned Rect in output
                                                                      // pixels (D-130); mask is V6.6's
    Image     { image: ImageKey, rect: Rect, opacity: f32 },          // V6.x; V4 draws none
}
// offcut-render
pub struct VideoSources<'a> { pub main: (&'a web_sys::VideoFrame, Rotation) }
impl Renderer {
    pub async fn new(canvas: web_sys::OffscreenCanvas, width: Px, height: Px) -> Result<Renderer, RenderError>;
    pub fn resize(&mut self, width: Px, height: Px) -> Result<(), RenderError>;
    pub fn set_canvas(&mut self, canvas: web_sys::OffscreenCanvas, width: Px, height: Px) -> Result<(), RenderError>;
                                                                      // an export's canvas after the preview's, on the same device
    pub fn render(&mut self, sources: &VideoSources<'_>, list: &DisplayList, fonts: &FontSet) -> Result<(), RenderError>;
                                                                      // the list in its order; V4's support rule (D-133, composite.rs):
                                                                      // exactly one Video, the first command, of the Main slot, opaque,
                                                                      // no mask, src inside the frame and dst inside the canvas, and no
                                                                      // Image; any other list is Unsupported
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new_headless(width: Px, height: Px) -> Result<Renderer, RenderError>;   // a software adapter: WARP or lavapipe (D-134)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_to_image(&mut self, source: &RgbaImage, rotation: Rotation, list: &DisplayList,
                           fonts: &FontSet) -> Result<RgbaImage, RenderError>;          // golden tests
}
// offcut-wasm-render/src/session.rs and scene_api.rs (the render worker's binding)
impl RenderSession {
    pub async fn open(clip: ClipInfo, source: JsRandomAccess) -> Result<RenderSession, JsValue>;
    pub fn set_scene(&mut self, input: JsValue) -> Result<JsValue, JsValue>;            // validates the plan, builds; answers SceneReport
    pub async fn attach_canvas(&mut self, canvas: OffscreenCanvas, width: u32, height: u32) -> Result<(), JsValue>;
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), JsValue>;
    pub fn render_frame(&mut self, frame: &VideoFrame, t_out_ms: u32) -> Result<(), JsValue>;
    pub fn source_at(&self, t_out_ms: u32) -> JsValue;
    pub fn hit(&self, t_out_ms: u32, x: f32, y: f32) -> JsValue;
    pub fn caption_mirror(&self, t_out_ms: u32) -> JsValue;
    pub fn frame_count(&self) -> u32;
    pub fn summary(&self) -> JsValue;
}
pub fn rules_plan(transcript: JsValue, events: JsValue) -> Result<JsValue, JsValue>;     // the detector's events as a plan (D-124)
```

`offcut-scene/src/display_list.rs` defines `Rect`, `Rgba`, `PathEl`, `PositionedGlyph`, `Stroke`, `Affine`, `CropRect`, `VideoSlot`, `ImageKey` and `FontKey`. `FontSet::embedded()` holds five faces, keys 0 to 4: Inter 700, Inter 900, JetBrains Mono 700, Noto Emoji, Inter 800 (D-132: a key is appended, never renumbered). `JsRandomAccess` is private to each binding crate and wraps a `FileSystemSyncAccessHandle`; there is no shared binding crate (v2implementation D-30).

### 19.3 Build and frame evaluation

```text
build_scene:
  every time the scene uses is mapped through time_map.to_out exactly once (identity in V4-V6, §40.1)
  video     = video::track(clip, edit.crop_offset)               // one DrawCmd::Video per frame; framing below
  words     = effective words (edits applied, hidden removed, a number's span replaced by its display, §40.4)
  chunks    = captions::chunk(words, style)                      // §19.4; laid out and shaped once (scene cache)
  items     = for each enabled plan item: visuals::<kind>::layout(item, style, theme) → keyframe tracks
  watermark = profile.watermark ? watermark::layout() : none
frame_at(t):
  cmds = [video at t]
  for the chunk shown at t:   captions::draw(chunk, t)           // the active word's treatment, KeywordPop scale
  for each item shown at t:   visuals::<kind>::draw(track, t)    // motion::eval(track, t) with easing
  watermark → cmds
  scale all cmds but Video.dst by the output scale
framing (video.rs):                                              // target aspect 9:16
  a = display_w / display_h
  a > 9/16  → w = display_h * 9/16; x = (display_w - w)/2 * (1 + offset); y = 0; h = display_h      // landscape, square
  a <= 9/16 → h = display_w * 16/9; y = (display_h - h)/2; x = 0; w = display_w                     // portrait; a 3:4 frame loses its sides
```

### 19.4 Captions and styles (`styles.rs`, `readability.rs`, `placement.rs`; visual values are (assumption), reviewed in E-2)

```text
captions::chunk: hard breaks at a sentence end, a gap of CHUNK_GAP_MS (350 ms) or more, max_words, and the line
   and character limits; between two hard breaks the chunk and line boundaries of least total cost (V4, D-167,
   readability.rs: after , ; : — costs 0, before a clause word 1, a plain break 3, after a binder 12, a one-word
   chunk 8 unless a 250 ms pause stands on both sides, unbalanced lines their difference); a number is one word
a chunk is shown from its first word's start less a lead to its last word's end plus CHUNK_TAIL_MS (120 ms), never
   overlapping the next; at least MIN_CHUNK_MS (700 ms) when the silence after it allows (D-168); no word moves
active word = word with start <= t < end, drawn with the style's treatment
KeywordPop word: scale 1.0 → 1.25 → 1.10 over 180 ms from its start, in the style's pop colour
numbers in the captions take the accent colour while EditState.highlight_numbers is on (D-169)
```

| Style (interface name) | Font | Size (lp) | Case | Line limits | Treatment | Motion |
|---|---|---|---|---|---|---|
| Clean (default) | Inter 800 | 72 | As spoken | 16 chars x 2 lines, 4 words | White, 5 lp dark stroke, drop shadow (0 x 4 lp, black at 45%); the active word on a rounded pill (radius 14 lp, padding 12 x 4 lp) in the accent `#FFD60A`, the word in `#111114`; the pill slides word to word over `PILL_SLIDE_MS` (90 ms) and grows from 0.9 to 1.0 | Ease-out cubic, 120 ms |
| Bold | Inter 900 | 84 | Upper | 12 chars x 2 lines, 3 words | White, 10 lp stroke, the same shadow, active word in the accent, KeywordPop in `#FF5E3A` | Back-out, 140 ms |
| Tech ("Mono") | JetBrains Mono 700 | 56 | As spoken | 22 chars x 2 lines, 5 words | Rounded dark panel (radius 24, padding 32) at 85%, green `#3DDC84` active word, block cursor | Linear, 80 ms |

Same layout rules for all three (PS §12.2), from D-170 (the premium default). The colours are the `Theme`'s roles (`theme.rs`, the only place a colour literal may be); `styles_use_theme.rs` asserts no style names one. A glyph run is stroked first and filled over the stroke, so the outline that shows is the outer half of the stroke width (v2implementation D-68). The scene's fonts are OFL-licensed and embedded in `offcut-scene/assets/fonts/`; `LICENSES.md` lists them. They are the only fonts the scene embeds; the interface's self-hosted Inter (`web/src/ui/assets/inter-latin.woff2`, D-180) is listed in `web/LICENSES.md`.

### 19.5 Event visuals (PS §12.2)

| Kind | Enter | Body | Exit |
|---|---|---|---|
| NumberReveal | At anchor start: count-up from 0 over 500 ms for integers of at least 10, otherwise scale-in 0.8 → 1.0 over 200 ms | Value at 200 lp auto-fitted to 900 lp width; unit inline; label at 48 lp beneath | Hold to `span.end + 1200 ms`; fade 200 ms |
| ListReveal | Header at announcer start (omitted when empty) | Each item appears at its own `at`, numbered, stacked; text auto-shrinks for more than 5 items | Hold to last item end + 1500 ms; fade 200 ms |
| FromTo | "from" value at its anchor; arrow and "to" value at the second anchor | Two bars with heights proportional to value / max(from, to), minimum 8%; "to" value in accent color at 1.15 scale | Hold to `span.end + 1500 ms`; fade 200 ms |
| KeywordPop | In-caption scale (§19.4) | none | none |

Count-up values at frame time `t` are `round(lerp(0, value, ease(t)))`, formatted by `format_quantity`.

### 19.6 Rules

- **Determinism.** No `HashMap` iteration (`BTreeMap` only), no accumulated float time, no clock, no randomness in `offcut-scene` (§7). `frame_at` is pure. `determinism.rs` builds each fixture scene twice and compares all frames' display lists byte-for-byte.
- **Safe area.** No `GlyphRun` or `FillRect` bounding box leaves the safe area; `layout_safe_area.rs` checks every frame of every labeled fixture in all three styles.
- **Missing glyphs.** Fallback chain: style font → Noto Emoji (monochrome) → glyph skipped. No tofu box is drawn.
- **Watermark.** Drawn by the scene when `profile.watermark` is true; there is no other code path that draws or removes it (INV-9). Text, size (36 lp) and opacity (60%) are constants in `watermark.rs` (assumption; PS §19 minor question, tested in alpha).
- **Renderer ownership.** `RenderSession` owns the wgpu device, the Vello renderer, the overlay texture and the scene. `render()` closes nothing; the caller closes each `VideoFrame`.

### 19.7 Edge cases

- A long number that cannot fit 900 lp at the minimum size (96 lp): the event is dropped at `build_scene` and removed from `summary()`.
- A long pause between two words: the caption chunk ends 120 ms after the last word before the pause and the next chunk starts 80 ms before the first word after it; nothing is drawn early to fill the pause and no timestamp is moved.
- Crop offset on a portrait source: ignored (control hidden).
- Rotation 90/270: `video_pass.rs` swaps sampling axes; `crop` is in display coordinates.

**Acceptance.** Display-list snapshots are exact (`display_list_snapshots.rs`, `insta`). Pixel goldens: structural similarity at least 0.98 against `fixtures/golden/` on a software adapter for each style x event kind (12 images) plus watermark on/off (assumption for the threshold, M1.4); `golden_frames.rs`. Scene build at most 1 s (PS §20.2). `frame_at` average at most 2 ms on R1 (assumption, M2.4).

**Contingency (PS §20.4, E-4, TE-3).** If Vello on WebGPU in a worker fails or render + encode exceeds 150 s on R1: implement `Canvas2dBackend` in `render.worker` (TypeScript) that draws `drawImage(videoFrame)` then the serialized `DisplayList` with `fillText`/`fillRect`. `offcut-scene` does not change. Separate goldens are recorded for that backend. Decision at M0 and M1.

**Seams.** v1.2 builds them in V4 (§40.9 S-2 to S-5): plan input, `Video` command with per-frame crop (face tracking and punch-in zoom change only that command), `Image` command, font registry, theme roles, motion data, zone table. Brand kit (V6.10): a `Theme` plus user fonts plus a logo. Other aspect ratios: constants in `safe_area.rs`. The Canvas2D backend exists only as the contingency above; if it is ever built it draws `Video` commands with `drawImage` and a clip path.

---

## 20. Preview Player

**v1.2 (V4, §40.1, §40.6):** the clock is program time (`OutMs`) over `prog48`; the scene is read at that `OutMs` and the source frame at `scene.source_at(t)`. With the identity map (V4-V6) this is exactly the v1.1 behaviour below.

**Model: the export renderer at half resolution, driven by the audio clock; the clock time is the recording time, so the source frame, the scene and the audio are all read at the same `TimeMs` with no mapping step. The preview therefore keeps the original pauses, speaking rhythm and duration. Rejected: `HTMLVideoElement` with an overlay canvas (a second renderer that differs from the export; no drift control), a pre-rendered preview file (kept as the PS §9.8 fallback).**

### 20.1 Interfaces

```ts
// usecases/control-preview.ts (as built in V4)
export function attach(canvas: HTMLCanvasElement, prog48: Float32Array): Promise<void>;   // the program audio (D-127)
export function play(): Promise<void>;          // the page's one AudioContext is made at the first play (V4 CL Entry 37)
export function pause(): Promise<void>;         // ends with a still frame of the paused time
export function seek(at: OutMs): Promise<void>; // one seek in flight, the latest target sent when it answers (D-148)
export function scrub(ms: number): Promise<void>;
export function togglePlay(): Promise<void>; export function stepFrame(direction: -1 | 1): Promise<void>; export function stepSecond(direction: -1 | 1): Promise<void>;
// workers/render/video-source.ts (time arguments are the source's: scene.source_at(t))
export class VideoSource {
  constructor(demuxer: DemuxerHandle, info: ClipInfo);
  frameAt(t: TimeMs): VideoFrame | null;          // latest decoded frame with pts <= t; null while decoding
  frameAtBlocking(t: TimeMs): Promise<VideoFrame>; // waits for decode, never returns a stale frame
  prefetch(from: TimeMs): void;                   // V4: after a pause, and when the scene is set before a canvas
  close(): void;                                  // closes every queued frame and the decoder
}
// workers/render/preview-loop.ts
export function runPreview(session: RenderSession, source: VideoSource,
                           clock: () => OutMs, isStopped: () => boolean): Promise<PreviewStats>;
export type PreviewStats = { frames: number; late: number; maxDriftMs: DurMs; downscaled: boolean };
```

`DemuxerHandle` is a TypeScript interface declared in `video-source.ts`; `RenderSession` satisfies it through four extra exported methods, so the render worker reads samples through the session that owns the demuxer and loads only `offcut_render.wasm` (v2implementation D-30, §13.5).

### 20.2 Algorithm

```text
main thread: AudioBufferSourceNode(out48 at 48 kHz) → destination
   audioMs = (ctx.currentTime - startCtxTime) * 1000 + startOffset
   every CLOCK_SYNC_INTERVAL = 250 ms: previewClock({audioMs, epochMs})
worker clock(): audioMs + (performance.timeOrigin + performance.now() - epochMs)

runPreview, per requestAnimationFrame in the worker:
   t = clock(); if t >= program duration → resolve (ended)
   n = floor(t * OUTPUT_FPS / 1000); if n == lastN → return
   tn = FrameIdx(n).to_out_ms()                       // program time (V4); identity map: equal to source time
   frame = source.frameAt(scene.source_at(tn)) ?? lastFrame   // late frame: reuse and count
   session.render_frame(frame, tn)
   stats.maxDriftMs = max(stats.maxDriftMs, |t - tn|)

VideoSource.frameAt(t):
   keep up to PREVIEW_QUEUE = 6 decoded frames ahead; feed the decoder while decodeQueueSize < 3
   close frames with pts < t - one frame
   t before the queue head, or beyond its horizon by more than 1 s → flush; restart at keyframe_at_or_before(t)
```

### 20.3 Rules and edge cases

- Preview size `PREVIEW_WIDTH x PREVIEW_HEIGHT = 540 x 960`. If the mean frame cost over 60 frames exceeds 28 ms, the session resizes to 360 x 640 (assumption, M1.5).
- The audio clock is the master; video never drives time.
- The preview never queues frames it cannot show: it drops to the newest.
- During `updating` and `locked` states the loop is stopped; the last frame stays on the canvas.
- Autoplay policy: the first play follows a user gesture (the drop or a click), which unlocks the `AudioContext`.
- Seeking into a long GOP: target at most 300 ms to first frame for GOPs of 2 s or less (assumption, M1.5); a spinner shows beyond that.

**Acceptance.** Drift at most 80 ms over 60 s (PS §20.1): `preview-sync.spec.ts` plays the reference fixture for 60 s and asserts `PreviewStats.maxDriftMs <= 80` and `late / frames <= 0.05`. First preview frame at most 500 ms after `ready` (assumption, M1.5).

**Contingency (PS §9.8).** If real-time preview misses the drift or late-frame target on R1: render a 360 x 640 preview file with the export loop into OPFS and play it in a `<video>` element; edits re-render it. Decision at M1.5.

**Seams (not built now).** None beyond the contingency.

---

## 21. Export: Encode and Mux

```text
original video timeline + enhanced audio of the same duration + transcript / events / style
  → render every output frame → H.264 encode + AAC encode → MP4 mux → OPFS
```

**v1.2 (V5, §40.9 S-8):** the loop runs over program time: `N = ceil(time_map.out_duration x 30 / 1000)`, frame `n` renders the scene at `FrameIdx(n).to_out_ms()` with the source frame at `scene.source_at(t)`, and the audio encoded is `prog48`. With the identity map this is exactly the v1.1 model below; from V6.3 the export is as long as the map says (INV-5).

**Model: an offline, frame-accurate loop in `render.worker` over the original timeline: for every output frame, take the source frame at that frame's own timestamp, render it with the scene to the WebGPU `OffscreenCanvas`, wrap it as a `VideoFrame`, encode H.264 with WebCodecs, encode `out48` (same duration as the source audio) to AAC, and write a faststart MP4 with the Rust muxer straight to OPFS. There is no audio trimming stage before AAC, no video trimming stage before H.264 and no time map. Rejected: `MediaRecorder` (real-time, not frame-accurate), FFmpeg.wasm (PS §9.8), server rendering (PS §11).**

### 21.1 Interfaces

```ts
// workers/render/export-loop.ts
export function runExport(a: { session: RenderSession; source: VideoSource; profile: ExportProfile;
                               prog48: Float32Array; sink: OpfsSink; canvas: OffscreenCanvas;
                               onProgress: (done: number, total: number) => void; onEncoded: () => void;
                               isCancelled: () => boolean }): Promise<{ bytes: Bytes } | { cancelled: true }>;
// workers/render/encoders.ts  [ONLY]
export const VIDEO_ENCODE_LADDER: readonly VideoLadderEntry[];
export const AAC_ENCODE_CONFIG: AudioEncoderConfig;
export function supportedConfigs(profile: ExportProfile): AsyncGenerator<VideoEncoderConfig>;   // the supported entries, in order: what an export may try; bitrate from profile.video_bitrate (v2implementation D-26)
// workers/render/opfs-sink.ts
export class OpfsSink { static open(path: string): Promise<OpfsSink>; writeAt(offset: Bytes, data: Uint8Array): void;
                        reset(): void; close(): Promise<void>; abort(): Promise<void>; }   // reset empties the file: a failed attempt's output goes
```

```rust
// offcut-mp4/src/mux.rs
pub trait MuxSink { fn write_at(&mut self, offset: Bytes, data: &[u8]) -> Result<(), IoError>; }
pub struct VideoTrackSpec { pub width: Px, pub height: Px, pub avcc: Vec<u8>, pub frame_count_hint: u32 }
pub struct AudioTrackSpec { pub sample_rate: Hz, pub channels: u32, pub asc: Vec<u8> }
impl<S: MuxSink> Mp4Muxer<S> {
    pub fn new(sink: S, video: VideoTrackSpec, audio: AudioTrackSpec) -> Result<Mp4Muxer<S>, MuxError>;
    pub fn add_video_sample(&mut self, data: &[u8], frame: FrameIdx, is_keyframe: bool) -> Result<(), MuxError>;   // V5 (D-211): samples in decode order; frame is the one shown
    pub fn add_audio_sample(&mut self, data: &[u8], pts: Micros, duration: Micros) -> Result<(), MuxError>;
    pub fn finalize(self) -> Result<Bytes, MuxError>;          // total file size
}
// offcut-entitlement/src/profile.rs  [ONLY]
pub fn export_profile(claims: Option<&EntitlementClaims>, now: UnixSecs) -> ExportProfile;
//   None → Preview;  plan Creator and now <= period_end and now <= exp → Creator (1080x1920, no watermark)
//   otherwise → Free (720x1280, watermark)
```

### 21.2 Configuration (`encoders.ts`; bitrates are (assumption), checked in TE-4)

| Item | Value | Source |
|---|---|---|
| Video ladder | 1 `avc1.640028` prefer-hardware; 2 `avc1.4d0028` prefer-hardware; 3 `avc1.640028` prefer-software; 4 `avc1.42e028` prefer-software | TE-4 confirms on R1/R2 |
| Level check | 1080x1920 = 68 x 120 = 8,160 macroblocks ≤ 8,192 (level 4.0 frame limit); 8,160 x 30 = 244,800 MB/s ≤ 245,760 | derived |
| Common video settings | `framerate: 30`, `bitrateMode: "variable"`, `latencyMode: "quality"`, `avc: {format: "avc"}` | |
| Keyframe interval | 60 frames (2 s) | (assumption, TE-4) |
| Video bitrate | Creator 8,000,000 b/s; Free 4,000,000 b/s; Preview not encoded. The values live in `offcut-entitlement/src/profile.rs`, the only builder of an `ExportProfile`; the Preview profile is 540x960, `watermark: false`, both bitrates 0. `CREATOR_VIDEO_BITRATE` in `encoders.ts` is the capability probe's bitrate only and must equal the `profile.rs` constant; TE-4 changes both or neither (v2implementation D-26) | (assumption, TE-4) |
| Audio | `mp4a.40.2`, 48,000 Hz, 2 channels (mono duplicated), 160,000 b/s | (assumption, TE-4) |
| Largest output | 90 s x (8,000,000 + 160,000) / 8 = 91.8 MB | derived |
| MP4 | brands `isom`/`mp42`; video timescale 30,000 with sample delta 1,000 (exact 30 fps CFR); audio timescale 48,000; chunks of 0.5 s; `moov` before `mdat` | |

### 21.3 Algorithm

```text
exportClip(exportId, token, prog48):
  claims  = verify_token(token, PUBLIC_KEY)                    // Err → E_ENTITLEMENT_INVALID
  profile = export_profile(Some(claims), now)
  session.set_scene(scene input with profile); session.attach_canvas(exportCanvas, profile.width, profile.height)
  sink = OpfsSink.open("exports/tmp/<exportId>.mp4")
  for each vcfg of supportedConfigs(profile):                  // none supported: E_ENCODE_VIDEO
   attempt:                                                    // V5 (D-216): encoders, muxer and what watches them are the attempt's
     venc = VideoEncoder(vcfg); aenc = AudioEncoder(AAC_ENCODE_CONFIG); source.rewind(); mux created on the first video chunk (needs avcC)
     N = session.frame_count()                                 // ceil(out_duration x 30 / 1000): 2,700 for a 90 s clip
     for n in 0..N:
        if isCancelled() → abort (close frames, encoders; sink.abort()) → {cancelled}
        t     = FrameIdx(n).to_out_ms()                        // program time
        pos   = session.source_at(t)                           // null → E_INTERNAL NoScene; V5 (D-215)
        frame = await source.frameAtBlocking(recorded(pos))    // waits for decode; never skips
        session.render_frame(frame, t)
        vf = new VideoFrame(exportCanvas, { timestamp: FrameIdx(n).to_micros(), duration: 33_333 })
        while venc.encodeQueueSize > ENCODE_QUEUE_MAX (4): await dequeue event
        venc.encode(vf, { keyFrame: n % 60 == 0 }); vf.close()
        onProgress(n + 1, N)
     await venc.flush()
     audio: pad prog48 with zeros to N * 1600 samples (48,000 / 30; less than one frame of padding, and no sample
            is ever removed); feed AudioData in 1,024-sample frames; await aenc.flush()
     encoder outputs → mux.add_video_sample / add_audio_sample (in arrival order per track)
     size = mux.finalize()
   E_ENCODE_VIDEO: sink.reset(), onProgress(0, N), next entry from frame 0; none left: the last failure
   E_ENCODE_AUDIO: sink.reset(), onProgress(0, N), once more on this entry; twice: thrown (D-217)
   a sink error, E_GPU_LOST, E_MUX and a cancel are never restarted
  sink.close(); move tmp → "exports/<exportId>.mp4"
  return { opfsPath, summary: session.summary(), stageTimings }

Mp4Muxer layout: ftyp | free(MOOV_RESERVE = 256 KiB) | mdat …   finalize(): write moov into the reserved space,
   pad with free; moov larger than the reserve → MuxError::MoovOverflow (cannot occur at ≤ 90 s; guarded by test)
Reordered frames (V5, D-211): samples are written in decode order, each with the frame it is shown as, at most
   MAX_REORDER = 16 ahead of its turn; finalize refuses a gap. When a frame came out of order, moov writes ctts
   version 0 after stts and a video elst starting at the largest delay. Otherwise neither: the file is byte-identical
AAC priming: the first audio chunk's timestamp/duration and the encoder's reported delay become an edit list (elst)
   so that audio presentation time 0 equals video frame 0 (TE-4 verifies offset ≤ one video frame)
```

### 21.4 Rules and edge cases

- **Frame accounting.** Every output frame `0..N` is rendered and encoded exactly once; the loop never drops or duplicates an output frame, and it runs from the first instant of the recording to the last. Source frames may repeat (VFR, source below 30 fps) or be skipped (source above 30 fps) only because of the frame-rate conversion to 30 fps; none is skipped or repeated because of what the audio contains.
- **Duration.** Output duration equals `time_map.out_duration()` to within one video frame; with the identity map that is the source duration, for any amount of silence in the clip. `timeline-preserved.spec.ts` (identity) and `cuts_applied.spec.ts` (V6.3), verifier checks 5 and 8 against the expected duration the test supplies.
- **Frame ownership.** Each `VideoFrame` (decoded or canvas-made) is closed by the code that obtained it, in a `finally` block. `export-loop.ts` keeps a live-frame counter asserted to be 0 at the end. The counter runs in every build, since E2E runs production builds: a non-zero count at the end of an export or after a preview stop fails that call with `E_INTERNAL` (v2implementation D-45).
- **Capture method.** `new VideoFrame(canvas)` after the render call is method A. TE-3 compares it with method B (`copyTextureToBuffer` readback) and picks one; the choice is local to `export-loop.ts`. TE-3 picked method A on 2026-10-09: a readback was 2.8 times slower, and a frame made from a buffer of RGBA bytes is encoded with full-range brightness that the stream does not declare (`docs/v2/experiments.md`; read on the development machine, v2implementation D-69).
- **Encoder failure mid-export.** The export restarts from frame 0 with the next ladder entry; partial output is deleted. An audio encoder failure gets one more try on the same entry. `useTimeLeft` starts again when the frames fall back (v5implementation D-216 to D-238).
- **GPU device lost.** `E_GPU_LOST`; the user retries; no partial resume.
- **Encoder emits reordered frames.** The muxer writes composition offsets (`ctts`), read back on real B-frames by `mux_reorder.rs` through `ffprobe` and `ffmpeg`. D1's encoder never reorders; phone playback of such a file is V10's checklist.
- **Render cache (S-9, as built in V5, §40.9).** Key = lower-case hex SHA-256, by `crypto.subtle` on the main thread, over a canonical JSON (sorted by key at every depth, `undefined` left out, arrays in their order) of ten inputs: source file hash prefix, `Transcript`, events, `EditState`, plan, `TimeMap`, theme (`"default"` until V6.10), music id (`null` until V6.7), the `ProfileKind` **of the token the worker verified** (never one the page read itself, INV-9) and `RENDERER_VERSION`, which `vite.config.ts` computes from `offcut_render_bg.wasm`, `export-loop.ts` and `encoders.ts` (16 hex; `"test"` where no bundler ran). One entry per clip: a miss deletes the row, then the file, then renders; so a hit is "the same edit as the last export", and undoing back to an earlier one misses until V7. An import clears the cache. A hit re-downloads the stored file under its original `ExportId`, renders nothing and sends no new receipt, `export_started` or `stage_timing`. A row without its file, one of another size or of a newer build is a miss; a failed lookup is a miss; a failed `put` is swallowed.
- **Download name.** `offcut-<yyyymmdd-hhmm>.mp4`; never derived from the source file name.

**Acceptance.** Render + encode at most 90 s and mux at most 2 s for the reference clip on R1 (PS §20.2, E-4). `verify_mp4.py` passes on every export in `export-free.spec.ts`, `export-creator.spec.ts` and the corpus run (§27.2). Creator output is 1080x1920 without watermark; Free output is 720x1280 with watermark (PS §20.1). Manual at M2.5: plays in the default gallery apps on one Android and one iOS phone and uploads to at least two target platforms (PS §20.1).

**Contingency (PS §20.4).** Render + encode above 150 s on R1 → Canvas2D overlay path (§19) and input capped at 60 s and 30 fps by changing `limits.rs`; decided at M0 and M1. TE-4 fails for AAC priming → write an explicit silent pre-roll and matching `elst`.

**Seams (not built now).** 60 fps output, 1:1/16:9 output, 4K: `ExportProfile` fields and `safe_area.rs` constants. Three versions per recording (P2): three `SceneInput`s through the same loop. Pro plan (P1): a `Plan` variant and one arm in `export_profile`.

---

## 22. Backend

**Model: one stateless Axum binary on a Render free web service; Postgres is the only durable state; anything held in memory (rate-limit buckets) may vanish at any restart. Rejected: serverless functions on the static host (a second runtime and its limits), Redis (no need: one instance, loss-tolerant buckets), in-memory sessions (Render restarts free services at will).**

### 22.1 Routes

All routes are under `/api/v1`, JSON only, `Cache-Control: no-store`. Default body limit 16 kB. Rate limits are token buckets (§24.5); "IP" means the address from `client_ip.rs`.

| Route | Auth | Purpose | Body limit | Rate limit | Idempotency |
|---|---|---|---|---|---|
| `GET /healthz` | none | Liveness; touches no DB; says `cloud_transcribe` (cloud transcription release, D-355) | none | 60/min per IP | n/a |
| `POST /auth/magic-link` | none | Create link, send email; 202 for any address (no account enumeration); 503 `email_unavailable` when the mail relay fails, sent inside the request with an 8 s timeout (V6, D-254) | 1 kB | 3 per 15 min per email; 10/h per IP | New request invalidates earlier unused links for that email |
| `POST /auth/verify` | none | Consume link; create user if new; set refresh cookie; return `SessionResponse` | 1 kB | 10/min per IP | Token is single-use |
| `POST /auth/refresh` | refresh cookie + CSRF header | Rotate refresh token; return `SessionResponse` | 0 | 30/min per IP | Previous token accepted for 30 s (multi-tab), then reuse revokes the session |
| `POST /auth/logout` | refresh cookie + CSRF header | Revoke session; clear cookie | 0 | 30/min per IP | Idempotent |
| `GET /me` | bearer | `MeResponse` | none | 60/min per user | n/a |
| `GET /entitlement` | bearer | Fresh `EntitlementResponse` | none | 60/min per user | n/a |
| `POST /usage/receipts` | bearer | Record one export; return fresh token | 1 kB | 30/min per user | `export_id` primary key; repeats return the same shape and change nothing |
| `POST /billing/checkout` | bearer | Hosted-checkout URL for an `Offer`; 403 `forbidden` for an `active` or `past_due` subscription (V6, D-300) | 1 kB | 10/min per user | none |
| `GET /billing/portal` | bearer | Customer-portal URL | none | 10/min per user | n/a |
| `POST /billing/webhook` | provider signature over raw body | Apply subscription events | 64 kB | 120/min per IP | `provider_event_id` primary key; replays return 200 and change nothing |
| `POST /events` | none | Analytics batch, at most 50 events | 16 kB | 60 requests/min per IP | None; duplicates tolerated |
| `POST /notify-me` | none | Waitlist / unsupported-platform email | 1 kB | 5/h per IP | Primary key (email, wanted) |
| `POST /transcribe` (cloud transcription release) | bearer | One clip's compressed sound (`audio/ogg`, Opus, mono, at most 92 s) to its words with times, from the transcription provider; checks before the body is read; never stored or logged (P-16) | **1 MiB** (DC-14) | 20 per 24 h per user; 60/h per IP; a global audio budget of 7,000 s an hour and 27,000 s a day (`transcribe/budget.rs`). **v1.4, windows of a long source** (header `X-Offcut-Window: <index>/<total>`, §43.6): instead of the 20, `LONG_AUDIO_PER_USER_DAY = 3 h` of audio per user per 24 h, and 200/h per IP (assumptions; the founder sets them). A client that marks clips as windows gains at most that budget. The global budget's values come from `TRANSCRIBE_BUDGET_HOUR_S` and `TRANSCRIBE_BUDGET_DAY_S`, defaulting to the values above, so the Dev tier raises them without a code change (§52). **(V6.1, as built: D-411, D-412.)** `0 ≤ index < total ≤ MAX_SOURCE_WINDOWS = 35`; a malformed header is 400 before the body is read. A request with the header is counted by `TranscribeWindowUser` (120 windows per 24 h, at most 3 hours of 90 s windows) and `TranscribeWindowIp` (200/h); one without it by `TranscribeUser` and `TranscribeIp` as before. The IP check moved from the route's layer into the handler, at the same place (before the body is read; an unauthenticated request still spends the address's token), so the header chooses the group. The log line gains `window: true/false`, never the index. `config::transcribe_budget()` reads the two budget variables apart from `Config`; `budget.rs`' constants stay the defaults | None; each success counts against the budget |
| `POST /plan` (v1.2, V6.2) | bearer optional; **(V6.2, as built: D-494) bearer required, a request without one is 401 before its body is read** | Edit plan: forward the transcript to the AI provider, return `PlanResponse` (§40.7); body never stored or logged | 64 kB | 10/min per IP; plan quotas of §40.7 | None; each success counts once against the quota |
| `POST /shorts` (v1.3, V6.4) | bearer | Director: sentences → `ShortsPlan` (§45.3); body never stored or logged | **160 kB** (DC-15, v1.4) | 6/min per user | A repeat for the same prefix in a month uses no new grant |
| `POST /broll/search` (v1.4, V6.8) | bearer | Stock B-roll search: one query → at most 12 Pixabay results (§48.6); the query is never stored or logged; the cache is keyed by its hash | 1 kB | 20/min per user; 90 per 60 s for the whole server | Cached 24 h by query hash |
| `POST /chat-edit` (v1.3, V6.9) | bearer | Chat: message and context → `ChatReply` (§49.3); body never stored or logged | 64 kB | 30/min per user | none |
| `POST /account/delete` | bearer | Delete account (§22.7) | 0 | 3/h per user | Idempotent |
| `GET /account/export` | bearer | `AccountExport` JSON | none | 6/h per user | n/a |

`server/tests/no_media_routes.rs` walks the router and asserts: no route has a body limit above 64 kB; `multipart/*`, `application/octet-stream`, `video/*` and `audio/*` requests receive 415 on every route. **The two exceptions:** `POST /transcribe` takes `audio/ogg` with a 1 MiB limit (DC-14), its other media types still getting 415; and `POST /shorts` takes JSON up to 160 kB (DC-15, v1.4), with every media type still getting 415.

### 22.2 Sessions and CSRF

- **Magic link.** 32 random bytes, base64url, sent once by email as `<APP_ORIGIN>/auth/callback?token=…`. Only the SHA-256 hash is stored. `MAGIC_LINK_TTL = 15 min` (assumption, M1.7). Single use.
- **Access token.** Ed25519-signed `{sub, sid, exp}`; `ACCESS_TOKEN_TTL = 15 min` (assumption, M1.7); kept only in `auth-store` memory; sent as `Authorization: Bearer`. Never written to storage.
- **Refresh token.** 32 random bytes; stored only as a hash (PS §17); cookie `offcut_rt; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=5184000` (60 days; assumption, M1.7); rotated on every use.
- **CSRF.** The two cookie-authenticated routes require header `X-Offcut-Csrf: 1` and an `Origin` header equal to `APP_ORIGIN`. All other authenticated routes use the bearer token and ignore cookies.
- **CORS.** The API emits no CORS headers. Browsers reach it only same-origin through the Vercel rewrite.
- **Email normalization.** Trim + lower-case in `auth/magic_link.rs`. `users.email` holds the normalized address, as `magic_links` keeps only the normalized one (V6, D-260; PS §16).
- **Rotation (V6, D-258, D-297).** A session's previous refresh hash stays valid for 30 s from the first rotation, however often it is used, and every accepted refresh rotates and sends a new cookie; a use in the grace does not move or extend the previous hash. After the grace, a reuse revokes the session. The client refreshes under `navigator.locks` across tabs, after `ready()`, with a 25 s retry budget, so a retry never lands outside the grace. A session ends 60 days after sign-in, however active (no sliding).

### 22.3 Entitlement rules (`entitlement/issue.rs`, the only place)

```text
issue(user_id, now) -> signed token
  sub  = subscriptions.get(user_id)
  plan = Creator  if sub exists and now <= sub.current_period_end
         Free     otherwise
  if plan == Free:
     used      = count(usage_receipts where user_id, plan_at_export = 'free', created_at >= start_of_utc_month(now))
     remaining = max(0, FREE_EXPORTS_PER_MONTH - used)
     period_end = end_of_utc_month(now)
  else:
     remaining = 0;  period_end = sub.current_period_end
  claims = { v: 1, sub: user_id, plan, free_exports_remaining: remaining, period_end, iat: now,
             exp: now + ENTITLEMENT_OFFLINE_TTL_SECS }
  return sign_token(claims, ENTITLEMENT_SIGNING_KEY)
```

"Month" for the free quota is the UTC calendar month (assumption, §39). A canceled or past-due subscription keeps Creator until `current_period_end`; a refund sets `current_period_end = now` (§22.5).

### 22.4 Metering flow (PS §11)

1. Export finishes on the client; `receipt-outbox.ts` stores `export_id` in IndexedDB.
2. Client sends `POST /usage/receipts {export_id}` (retries: 5 s, 30 s, 5 min, then at each app start).
3. Server computes the user's current plan and runs `INSERT … ON CONFLICT (export_id) DO NOTHING` with `plan_at_export`.
4. Server calls `issue()` and returns the fresh token.
5. Client stores the token and removes the outbox entry.
6. Until step 5, `entitlement.effective()` = token remaining minus the outbox's Free rows of this account, floored at 0 (V6, D-267). A receipt without a session waits in the outbox (D-268); one answered 429 keeps its row, as does a 4xx with no readable body (D-307).
7. The server never rejects a receipt for exceeding a limit: the export has already happened. A Creator account above `CREATOR_FAIR_USE_PER_MONTH` produces one `fair_use_exceeded` warning log line per day (hashed user id); no behaviour changes (PS §11 "soft").

Limits are conversion mechanisms, not security controls (PS §11). No server logic trusts the client beyond counting what it reports.

### 22.5 Billing

```rust
// billing/provider.rs (as built in V6: v6implementation D-274, D-275, D-301)
#[async_trait]
pub trait BillingProvider: Send + Sync {
    async fn create_checkout_url(&self, user: UserId, email: &str, offer: Offer, return_url: &str, cancel_url: &str) -> Result<String, BillingError>;
    async fn create_portal_url(&self, customer_id: &str, return_url: &str) -> Result<String, BillingError>;
    async fn cancel_subscription(&self, subscription_id: &str) -> Result<(), BillingError>;   // "already cancelled" is Ok
    fn verify_signature(&self, headers: &HeaderMap, raw_body: &[u8], now: UnixSecs) -> Result<(), BillingError>;
    fn parse_event(&self, headers: &HeaderMap, raw_body: &[u8]) -> Result<Option<BillingEvent>, BillingError>;   // None = irrelevant type
}
pub enum BillingError { Unavailable, BadSignature, BadEvent }
pub enum BillingEventKind { Activated, Renewed, PastDue, CancelScheduled, Canceled, Refunded, Resumed, PlanChanged }
pub enum EventOwner { User(UserId), Customer(String) }
pub struct BillingEvent { pub provider_event_id: String, pub owner: EventOwner, pub customer_id: String,
                          pub subscription_id: Option<String>, pub kind: BillingEventKind, pub interval: Option<BillingInterval>,
                          pub current_period_end: Option<UnixSecs>, pub occurred_at: UnixSecs }
```

- **Provider (TE-9, V6).** Dodo Payments, through `billing/adapter.rs`, against `BILLING_API_BASE_URL` (`https://test.dodopayments.com` or `https://live.dodopayments.com`, D-273). Its webhooks follow Standard Webhooks: `provider_event_id` is the `webhook-id` header, which is why `parse_event` reads the headers (D-274). The signature is checked in `adapter.rs` itself: `whsec_` secret, HMAC-SHA256 over `id.timestamp.body`, any `v1,` entry compared in constant time, a timestamp more than 5 minutes off refused (D-278). A secret that is not a `whsec_` value stops the server at boot. The mapping was read from real test-mode deliveries (V6 CL Entries 13 and 18): `subscription.active` → `Activated`; `renewed` → `Renewed`; `on_hold` and `past_due` → `PastDue`; `cancelled` and `expired` → `Canceled`; `plan_changed` → `PlanChanged`; `updated` → `CancelScheduled` or `Resumed` by its `cancel_at_next_billing_date`; a whole `refund.succeeded` → `Refunded`, a partial one (`is_partial`) → `None` (D-277); every `payment.*` → `None`. The interval comes from `offers.rs` by the event's product id; an unknown product is `billing_event_ignored`.
- **Owner (D-275).** A subscription event names its user through `metadata.user_id`. A refund carries no metadata and no subscription id, only the customer: its owner is `Customer`, resolved through `subscriptions.provider_customer_id`. No resolvable owner, or a user since deleted, is 200 with nothing changed (D-272, D-284).
- **Checkout.** Hosted checkout by full-page navigation. `user_id` travels as the provider's `metadata` and returns in every subscription event (TE-9, read). Return URLs: `<APP_ORIGIN>/account?checkout=success|cancel`. A user whose row is `active` or `past_due` is refused with 403 `forbidden`, and the client opens the portal instead (D-300). Any provider failure is 502 `billing_unavailable`. `GET /billing/portal` without a subscription row is 404.
- **Offers (`billing/offers.rs`, the only place).** `Offer → env price id → (Plan::Creator, BillingInterval)`. `CreatorAnnualFounding` (E-5 presale, PS §19) is accepted only while `FOUNDING_OFFER_ENABLED=true`. Amounts (15 USD/month, 144 USD/year, 99 USD/year founding; PS §11, §19) are configured at the provider and displayed from `web/src/config/pricing.ts`; the server holds no amounts.
- **Event application (one transaction with the dedupe insert, C-19).** Events older than `subscriptions.last_event_at` are ignored. An event whose `subscription_id` differs from the row's is ignored, except `Activated` (D-299).

| From | Event | To | Fields set |
|---|---|---|---|
| none, canceled, past_due (V6, D-276: recovery) | `Activated` | active | all provider ids, interval, `current_period_end`, `cancel_at_period_end = false` |
| active, past_due | `Renewed` | active | `current_period_end` |
| active | `PastDue` | past_due | none: the paid period is not moved, as the provider moves `next_billing_date` before the charge |
| active, past_due | `CancelScheduled` | unchanged | `cancel_at_period_end = true` |
| active, past_due, with `cancel_at_period_end` (V6, D-301) | `Resumed` | unchanged | `cancel_at_period_end = false` |
| active, past_due (V6, D-301) | `PlanChanged` | unchanged | interval, `current_period_end` |
| active, past_due | `Canceled` | canceled | `current_period_end` as reported |
| any | `Refunded` | canceled | `current_period_end = occurred_at`; a live subscription is also cancelled at the provider inside the transaction, as a refund does not stop it there (V6 CL Entry 13) |

Any other (from, event) pair is logged as `billing_event_ignored` and changes nothing; `billing_events.rs` covers every cell.

### 22.6 Analytics allowlist (`offcut-api-types/src/analytics.rs`, the only place)

```rust
#[derive(Serialize, Deserialize, TS)]
#[serde(tag = "name", content = "props", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnalyticsEvent { LandingView { hero_variant: HeroVariant }, CapabilityCheck { /* … */ }, /* one variant per row below */ }
pub const ANALYTICS_EVENT_DOCS: &[EventDoc];     // name + plain-language description; generated into gen/api.ts
```

Every property is an enum, a boolean or a bounded integer. No variant has a `String` field. Integers are capped server-side (`*_ms <= 3_600_000`, counts `<= 10_000`).

| Event | Properties | Measures |
|---|---|---|
| `landing_view` | `hero_variant: outcome \| privacy` | Visitors (PS §13); E-6 |
| `capability_check` | `result: pass \| fail`, `unsupported_reason?`, `gpu_vendor`, `memory_bucket`, `platform` | E-8; PS §20.5 capability pass rate |
| `clip_accepted` | `duration_bucket: lt30 \| lt60 \| lte90`, `orientation`, `source: user \| sample` | Activation denominator |
| `clip_rejected` | `reject_reason` | PS §9.4 `reject_reason`; PS §20.5 reject rate |
| `model_download` | `outcome: ok \| failed \| hash_mismatch`, `duration_ms`, `resumed: bool` | J4 time |
| `stage_timing` | `stage: PipelineStage`, `duration_ms`, `asr_backend?: webgpu \| wasm` | E-3, E-4, PS §20.2 |
| `pipeline_done` | `total_ms`, `n_number`, `n_list`, `n_from_to`, `n_keyword` | Detection yield |
| `preview_played` | none | AHA reach |
| `review_action` | `action: word_edit \| event_toggle \| style \| crop_offset`, `event_kind?`, `enabled?: bool` | Edit behaviour |
| `export_started` | `profile: free \| creator` | Funnel |
| `export_done` | `total_ms`, `profile`, `word_edits`, `events_kept`, `events_disabled`, `style`, `crop_adjusted: bool`, `from_cache: bool`; (V6.2, each optional, sent only with its flag on) `items_kept`, `items_disabled` (`ai_plan`), `enhance_off` (`motion`; a list of `EnhanceRow`, each at most once) | Activation (PS §10); E-7 keep rate; E-10 edit count; P1 promotion triggers (PS §9.5) |
| `export_failed` | `error_code`, `stage` | Stabilization |
| `job_cancelled` | `stage` | Abandonment |
| `post_export_answer` | `answer: yes \| small_edits \| no` | PS §20.5 "would you post this" |
| `upgrade_prompt` | `placement: after_export \| limit_reached`, `action: shown \| clicked \| dismissed` | H2 |
| `signin_step` | `step: link_requested \| completed` | J9 friction |
| `checkout_step` | `step: started \| returned_success \| returned_cancel` | H2 |
| `client_error` | `error_code`, `stage` | Error taxonomy (PS §20.1) |
| `local_data_cleared` | none | Settings use |

**Ingestion rules.** Stored per event: `anon_id`, `name`, `props`, server receive time. Not stored: IP, user agent, user id, client timestamp. Events are never joined to a user at MVP (assumption, §39; PS §16 allows a join only with consent, and no consent UI is specified). `anon_id` is a random UUID kept in IndexedDB `meta`. A batch with any invalid event is rejected whole with 400. Repeat-export and paid metrics (PS §20.5) come from `usage_receipts` and `subscriptions`, not from analytics.

### 22.7 Account deletion and export (PS §17)

1. Authenticate the bearer token.
2. If a subscription row exists with status `active` or `past_due`: `cancel_subscription`. Failure → 502 `billing_unavailable`; nothing is deleted.
3. One transaction: delete the user row (cascades to `sessions`, `subscriptions`, `usage_receipts`), `magic_links` and `platform_waitlist` rows for the normalized email.
4. Clear the refresh cookie; respond 204.
5. Deletion is immediate, which satisfies the 30-day bound (PS §17). Analytics rows hold no account link and expire at 90 days. Billing records held by the merchant are outside this system (PS §17).

`GET /account/export` returns the user row, session metadata (no token hashes), subscription summary and usage receipts.

### 22.8 Configuration (`config.rs`, the only place that reads the environment)

`DATABASE_URL`, `APP_ORIGIN`, `PORT`, `LOG_LEVEL`, `ENTITLEMENT_SIGNING_KEY`, `ACCESS_TOKEN_SIGNING_KEY`, `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `MAIL_FROM` (V6, D-251: `MAIL_API_KEY` is gone), `BILLING_API_BASE_URL` (V6, D-273), `BILLING_API_KEY`, `BILLING_WEBHOOK_SECRET`, `BILLING_PRICE_CREATOR_MONTHLY`, `BILLING_PRICE_CREATOR_ANNUAL`, `BILLING_PRICE_CREATOR_ANNUAL_FOUNDING`, `FOUNDING_OFFER_ENABLED`, `TRANSCRIBE_ENABLED` (the server's kill switch), `TRANSCRIBE_API_KEY`, `TRANSCRIBE_API_BASE_URL` (cloud transcription release, D-356), `TRUSTED_PROXY_HOPS`; `GIT_SHA` is optional. **v1.4:** `TRANSCRIBE_BUDGET_HOUR_S` and `TRANSCRIBE_BUDGET_DAY_S` (V6.1, optional, defaulting to 7,000 and 27,000, §22.1); `PIXABAY_API_KEY` (V6.8, required once `stock_broll` ships; set in Render before the deploy that reads it). A missing or malformed variable aborts startup with the variable's name (never its value). `BILLING_WEBHOOK_SECRET` must be a `whsec_` value. Production runs Dodo's test-mode keys from V6's merge until the founding release after V6.6 (D-285, D-303).

### 22.9 Free-tier behaviour (external facts: verified 2026-10, re-verified by TE-11)

- The service sleeps after 15 minutes without inbound traffic and takes about one minute to wake.
- **Keep-warm.** The uptime monitor that PS §20.1 requires calls `GET /api/v1/healthz` every 5 minutes. One always-on instance uses at most 24 x 31 = 744 h of the 750 free instance hours per month (derived).
- **Client tolerance.** `apiClient.wake()` at app start; per-attempt timeout `API_TIMEOUT = 10 s`; retries while total wait is within `API_COLD_START_BUDGET = 70 s` (C-15). Import, processing, preview and review never call the API.
- **Webhooks.** Delivered to the Render host directly. A request that hits a sleeping service may time out; the merchant's retry delivers it. TE-9 read (V6): a failed delivery is retried after about 6 s with the same `webhook-id`; Dodo documents up to 8 attempts over about 27 hours.
- **Boot.** `sqlx::migrate!` runs at startup; the retention purge runs at startup and every 6 h in-process.
- **Memory.** DB pool `max_connections = 5`; no caches. Budget 512 MB (TE-11).

**Acceptance.** All `server/tests/*` pass against a real Postgres in CI. p95 handler time under 150 ms warm, excluding provider calls (assumption, M2.4). Cold request completes within `API_COLD_START_BUDGET` in `api-cold-start.spec.ts` (simulated delay) and in TE-11 (real).

**Contingency.** TE-11 fails (cold start beyond 70 s, instance hours insufficient, or memory): move the same Docker image to another free container host; nothing else changes because the client only knows `/api/v1` on its own origin.

**Seams (not built now).** Pro plan (P1): a price id and a `Plan` variant. Pause subscription (P1): a `BillingEventKind` variant. Google sign-in (P1): one route. Cloud transcription (P1): new routes that would break `no_media_routes.rs` by design and therefore need this spec revised first.

---


**(V6.1, as built: D-412.)** Two optional variables, read by `config::transcribe_budget()` apart from `Config`: `TRANSCRIBE_BUDGET_HOUR_S` and `TRANSCRIBE_BUDGET_DAY_S`, the global audio budget's seconds per UTC hour and day (default 7,000 and 27,000, `budget.rs`' constants). A value that is not a positive whole number stops the server with its name. `render.yaml` lists both with `sync: false`, unset until Groq's Dev tier (V6 CL open item 19).
## 23. Persistence

### 23.1 Local stores

**OPFS (`persistence/opfs.ts` is the only place that builds these paths)**

| Path | Contents | Written by |
|---|---|---|
| `clips/<clipId>/source` | The imported file, byte-identical | `media.worker` |
| `clips/<clipId>/out48.f32` | Enhanced audio (raw f32, 48 kHz mono), same length as the decoded source audio | main, after `measureProsody` |
| `clips/<clipId>/prog48.f32` | The program's audio, what the export encodes (v5implementation D-213). `out48.f32` stays: V6.3 and V6.7 rebuild this from it | main, after `assemble` |
| `clips/<clipId>/lvl48.f32` | (V6.2, with `motion` on) The levelled voice: loudness and the limiter only, same length as `out48.f32` (v6.2implementation D-487). `prog48.f32` stays the clean program; a clip on `VoiceMode::Level` assembles its program from this | main, after `writeAudio` |
| `exports/tmp/<exportId>.mp4` | Export in progress | `render.worker` |
| `exports/<exportId>.mp4` | Finished export (render cache) | `render.worker` |
| `models/<modelId>/<file>.part`, `models/<modelId>/<file>` | Model download in progress / verified | `models/download.ts` |

**IndexedDB database `offcut`, version 1 (`persistence/schema.ts` is the only place that names stores)**

| Store | Key | Value |
|---|---|---|
| `clips` | `clipId` | `{ clipId, createdAt, lastOpenedAt, clipInfo, sourceBytes, sourceHashPrefix }` (no file name, no path) |
| `transcripts` | `clipId` | `{ transcript, prosody }` |
| `events` | `clipId` | `DetectedEvent[]`. No writer and no repo file in V3: on a restore the events are recomputed by `detect` from the stored transcript and prosody, which gives the same events and ids (INV-7), and toggles live in `EditState.event_overrides`, keyed by `EventId` (v3implementation D-82). V7 gives the store a writer or removes it here and from `schema.ts` through a version-2 migration |
| `edits` | `clipId` | `{ editState }` |
| `renderCache` | cache key (hex) | `{ clipId, exportId, opfsPath, bytes, createdAt, summary }` (a hit skips the worker, so the summary comes from the row, v5implementation D-226). Only `persistence/render-cache-repo.ts` writes it |
| `entitlement` | `"current"` | `{ token, storedAt }` |
| `receiptOutbox` | `exportId` | `{ exportId, enqueuedAt, attempts, plan, sub }` (V6, D-267: the plan it was exported under and the account it belongs to; `flushOutbox` sends only the signed-in account's rows) |
| `meta` | string | `anonId`, `persistRequested` |

Every value is wrapped as `{ schemaVersion: number, value }`.

**`sourceHashPrefix`** (v3implementation D-81) is the lower-case hex SHA-256 of the first `min(4 MiB, size)` bytes of the source file followed by its size as an unsigned 64-bit little-endian integer. It is computed on the main thread by `persistence/clips-repo.ts::hashPrefix(file)` with `crypto.subtle.digest("SHA-256", …)` over `file.slice(0, 4 MiB)`, before `importAndProbe` transfers the file, and stored in the `clips` record. No WASM runs for it; V7's restore compares it to tell a re-dropped file from another.

**Writers in V3.** `persistence/clips-repo.ts`, `transcript-repo.ts` and `edits-repo.ts` write `clips` (C-2, while the clip is `importing`), `transcripts` and `edits` (C-4 step F). `sourceBytes` is `file.size`, minted by `clips-repo.ts`'s one `MINTING` entry `toBytes` (v3implementation D-103). Each repo has `clear()`, called at import beside the OPFS sweep until V7's eviction exists (C-2). The repos throw `opfs.OpfsError` (kind `quota` for a `QuotaExceededError`, `io` for any other IndexedDB failure), which `run-pipeline.ts` maps to `E_STORAGE_QUOTA` / `E_STORAGE_IO`.

### 23.2 Server schema (`server/migrations/0001_init.sql`)

```sql
CREATE TABLE users (
  id               UUID PRIMARY KEY,
  email            TEXT NOT NULL,
  email_normalized TEXT NOT NULL UNIQUE,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE magic_links (
  token_hash       BYTEA PRIMARY KEY,
  email_normalized TEXT NOT NULL,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at       TIMESTAMPTZ NOT NULL,
  consumed_at      TIMESTAMPTZ
);
CREATE INDEX magic_links_email_idx ON magic_links (email_normalized);

CREATE TABLE sessions (
  id                 UUID PRIMARY KEY,
  user_id            UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  refresh_token_hash BYTEA NOT NULL UNIQUE,
  prev_token_hash    BYTEA,
  prev_valid_until   TIMESTAMPTZ,
  created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_used_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at         TIMESTAMPTZ NOT NULL,
  revoked_at         TIMESTAMPTZ
);
CREATE INDEX sessions_user_idx ON sessions (user_id);

CREATE TABLE subscriptions (
  user_id                  UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  provider_customer_id     TEXT NOT NULL,
  provider_subscription_id TEXT NOT NULL UNIQUE,
  plan                     TEXT NOT NULL CHECK (plan IN ('creator')),
  billing_interval         TEXT NOT NULL CHECK (billing_interval IN ('monthly','annual')),
  status                   TEXT NOT NULL CHECK (status IN ('active','past_due','canceled')),
  cancel_at_period_end     BOOLEAN NOT NULL DEFAULT false,
  current_period_end       TIMESTAMPTZ NOT NULL,
  last_event_at            TIMESTAMPTZ NOT NULL,
  updated_at               TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE webhook_events (
  provider_event_id TEXT PRIMARY KEY,
  event_type        TEXT NOT NULL,
  processed_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE usage_receipts (
  export_id      UUID PRIMARY KEY,
  user_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  plan_at_export TEXT NOT NULL CHECK (plan_at_export IN ('free','creator')),
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX usage_receipts_user_time_idx ON usage_receipts (user_id, created_at);

CREATE TABLE analytics_events (
  id      BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  anon_id UUID NOT NULL,
  name    TEXT NOT NULL,
  props   JSONB NOT NULL,
  ts      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX analytics_events_ts_idx ON analytics_events (ts);
CREATE INDEX analytics_events_name_ts_idx ON analytics_events (name, ts);

CREATE TABLE platform_waitlist (
  email_normalized TEXT NOT NULL,
  wanted           TEXT NOT NULL CHECK (wanted IN ('launch','safari','firefox','mobile','linux')),
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (email_normalized, wanted)
);
```

`users`, `sessions`, `subscriptions`, `webhook_events`, `usage_receipts`, `analytics_events` are the PS §16 entities. `magic_links` and `platform_waitlist` are added by this document for magic-link auth (PS §17) and the unsupported-page / E-1 email capture (PS §9.3, §19).

### 23.3 Rules

- **Eviction.** `quota.ts::ensureFree(bytes)` deletes, in order, until enough space: exports other than the newest per clip; whole clips beyond `MAX_RECENT_CLIPS` by `lastOpenedAt`; remaining clips oldest first (never the current clip). Model files are never evicted automatically. Deleting a clip removes its OPFS directory, its exports and its rows in `clips`, `transcripts`, `events`, `edits`, `renderCache`.
- **Write order.** OPFS file first, then the IndexedDB row that references it. A row without its file is deleted at startup (`restoreClip`); a file without a row is deleted by `opfs.sweepTemp()`.
- **Clear local data.** Removes all OPFS content and clears `clips`, `transcripts`, `events`, `edits`, `renderCache`. Keeps `receiptOutbox` (pending counts), `entitlement` and `meta`. Files the user already downloaded cannot be removed (PS §17).
- **Local migrations.** `db.ts` runs numbered upgrade functions in `onupgradeneeded`. A value with a `schemaVersion` newer than the code is treated as absent and deleted. **(V6.1, as built: D-414.)** The database's version (its stores) and the records' `schemaVersion` are two numbers: `DB_STRUCTURE_VERSION = 2` adds stores, `DB_VERSION = 1` stays every record's version. The upgrade loop runs only up to the version being opened, so the clip path's opening at 1 never makes the source stores; only `sources-repo.ts` opens at 2. A build that meets a database newer than it asks for (`VersionError`) opens it at its own version and uses only the stores it knows, so a rollback never locks a person out of their clips.
- **Server migrations.** Forward-only `sqlx` migrations run at boot. Each must be compatible with the previously deployed binary (add first, remove in a later release), because Render swaps instances without coordination.
- **Server retention.** `analytics/retention.rs` deletes `analytics_events` older than 90 days (PS §16), `magic_links` older than 1 day, `webhook_events` older than 90 days (assumption, TE-9: longer than the provider's retry horizon), revoked or expired `sessions`.
- **Backups.** None on a free database tier (assumption, TE-6). Accepted cost: subscription state is rebuilt from the merchant by replaying events; usage receipts and waitlist would be lost. Reversal: none inside the 0 USD constraint (PS A-10); a paid tier with backups needs a founder decision and a spec revision.

**Acceptance.** `restore.spec.ts`: reload mid-review returns to `ready` with edits intact and no re-transcription. `clear-local-data.spec.ts`: OPFS empty and stores empty afterwards. `receipt-outbox.test.ts`: entries survive reload and are removed only on a 2xx; as built in V6 (v6implementation §11.5, D-307), also on a 4xx with a readable body other than 401 and 429, as that receipt can never succeed. A 429, a 401, or a 4xx with no readable body keeps its row. Estimated server data at base-case traffic: 12,000 visitors x 30 events x 200 B = 72 MB before retention (derived from PS §13.2; per-visitor event count is an assumption, TE-6).

**v1.3.** Long-path stores and paths: §43.4. Assets: §48.5. Memory: §50.2. Server tables: §52. **Seams (not built now).** `BrandPreset`, `VersionSet`, `CloudJobConsent` (P1) and `Workspace` (P2) from PS §16 are new stores/tables; none exists.

---

## 24. Security and the Network Boundary

### 24.1 Complete network surface (closed list)

**From the browser**

| # | Destination | What | Constraints |
|---|---|---|---|
| 1 | App origin (`APP_ORIGIN`, Vercel) | `GET` static files; `/api/v1/*` | Only via `net/http.ts` and the two WASM loaders |
| 2 | Asset origin (host of `VITE_ASSET_BASE_URL`) | `GET`/`HEAD` model files, demo video, sample clip | No credentials, no query string, no request body; only via `net/asset-fetch.ts` and the demo `<video>` element |
| 3 | Merchant hosted checkout / portal | Top-level navigation to a URL returned by the API | Never fetched, framed or scripted |
| 4 | `cdn.pixabay.com` (v1.4, V6.8, flag `stock_broll`) | `GET` of one stock video file the person or the plan chose, and of search thumbnails | Only via `net/stock-fetch.ts` (the fifth file allowed to fetch, §7), called by `usecases/resolve-stock.ts` and the search panel; no credentials, no body; only while the stock switch is on (§48.6) |

Nothing else: no font CDN, no analytics vendor, no error tracker, no tag manager, no third-party script (PS §17). `config/allowlist-hosts.ts` lists hosts 1 and 2, and from V6.8 host 4 (v1.4), and is the only such list; `scripts/check-hosts.mjs` fails the build when the bundle contains any other `http(s)://` host literal.

**From the server:** Postgres host; the transcription provider's API (`api.groq.com/openai/v1`, `POST /transcribe` only, cloud transcription release); Pixabay's API (`pixabay.com/api/videos/`, `POST /broll/search` only, v1.4, V6.8); the SMTP relay (`smtp-relay.brevo.com:2525`, V6); the merchant API host (`test.dodopayments.com` until the founding release, then `live.dodopayments.com`); AI provider API host (v1.2, V6.2, TE-15; `POST /plan` only). **Into the Render host directly:** merchant webhooks; uptime monitor; the Vercel proxy.

### 24.2 Secrets

- Server secrets exist only as Render environment variables (§22.8). None is logged; `config.rs` implements `Debug` by hand with redaction.
- The client bundle contains only public values: asset base URL and the entitlement public key (`config/entitlement-public-key.ts`, an array so that a rotation can ship the new key before the server switches).
- CI runs a secret scanner on every push (§33).

### 24.3 Response headers (`web/vercel.json`, the only place)

```text
Cross-Origin-Opener-Policy:   same-origin
Cross-Origin-Embedder-Policy: require-corp
Content-Security-Policy:      default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self';
                              style-src 'self'; font-src 'self'; img-src 'self' data: blob:;
                              media-src 'self' blob: <ASSET_ORIGIN>; connect-src 'self' <ASSET_ORIGIN>;
                              base-uri 'none'; form-action 'none'; frame-ancestors 'none'
                              (v1.4, V6.8: connect-src gains https://cdn.pixabay.com, §24.1 row 4; thumbnails
                              are fetched and shown as blob: URLs, so img-src does not change)
Referrer-Policy:              no-referrer
X-Content-Type-Options:       nosniff
Permissions-Policy:           camera=(), microphone=(), geolocation=()
Strict-Transport-Security:    max-age=63072000; includeSubDomains
Cache-Control:                hashed assets: public, max-age=31536000, immutable;  index.html: no-cache
```

- COOP + COEP make the page cross-origin isolated (WASM threads, PS §9.3). The demo video uses `crossorigin="anonymous"`; the asset host sends CORS headers for `APP_ORIGIN` and exposes `Content-Range`, `Accept-Ranges`, `Content-Length`.
- `Referrer-Policy: no-referrer` keeps the magic-link token out of any outbound referrer; `AuthCallbackPage` also strips it from the URL.
- API responses add `Cache-Control: no-store` and `X-Content-Type-Options: nosniff` (`server/src/headers.rs`).

### 24.4 Log redaction (`server/src/log.rs`, the only place)

Logged per request: request id, method, route template, status, latency, and for authenticated routes the first 8 hex characters of SHA-256(user id). Never logged: email addresses, tokens, cookies, request or response bodies, IP addresses, query strings. `log_redaction.rs` runs the auth, receipt, webhook and analytics flows with marker values and asserts none appears in captured log output.

### 24.5 Rate limiting

In-memory token buckets keyed by (route group, key), at most 50,000 keys with least-recently-used eviction. Loss at restart is accepted. `client_ip.rs` takes the address `TRUSTED_PROXY_HOPS` positions from the right of `X-Forwarded-For`, or the socket peer when the header is absent; TE-5 establishes the hop count behind the Vercel rewrite. Limit exceeded → 429 with `retry_after_secs`. Cloud transcription (cloud transcription release) adds `TranscribeUser` (20 per 24 h) and `TranscribeIp` (60 per hour), and a global audio budget in `transcribe/budget.rs`, the only place it is set: 7,000 s per UTC hour and 27,000 s per UTC day, under the provider's free tier, charged before the provider is called and lost at a restart. **(V6.1, as built.)** Windows of a long source have their own groups, `TranscribeWindowUser` (120 per 24 h) and `TranscribeWindowIp` (200 per hour), chosen in the handler by `X-Offcut-Window` (§22.1); the budget's two values come from `TRANSCRIBE_BUDGET_HOUR_S` and `TRANSCRIBE_BUDGET_DAY_S` when set (§22.8), `budget.rs`' constants otherwise.

### 24.6 Integrity and supply chain

- Model files are SHA-256-verified against the bundled manifest before first use (PS §17).
- `cargo deny check` (licenses, advisories, bans) and `pnpm audit --prod --audit-level high` run in CI.
- The ONNX Runtime `.wasm` files are copied from the pinned npm package at build time to `web/public/ort/<runtime version>/` and served from the app origin; the version in the path keeps the year-long immutable cache of `/ort/*` from serving a stale runtime after an upgrade (v2implementation D-37).

### 24.7 Edge cases

- Direct browser calls to the Render host: no CORS headers, so scripts on other origins cannot read responses; state-changing routes need a bearer token or the `SameSite=Strict` cookie plus the CSRF header.
- Replay of a webhook body with a valid signature: deduplicated by `provider_event_id`.
- Magic link opened on another device: signs in that device; the first device continues as anonymous.
- Client tampering with the entitlement token or watermark: possible and accepted (PS §11); no secret is exposed by it.

**Acceptance.** `privacy-network.spec.ts` (hosts), `log_redaction.rs`, `rate_limit.rs`, `no_media_routes.rs`, `auth_flow.rs` (cookie attributes, CSRF header, rotation, reuse revocation) pass. A CI step fetches the deployed preview and asserts every header in §24.3.

**Seams (not built now).** None.

---

## 25. Privacy and Data Handling

### 25.1 Promises and their three proofs

| # | Promise (PS reference) | Structural | Automatic | In the product |
|---|---|---|---|---|
| P-1 | Video, video frames, music, brand assets, captions as edited and project edits are not uploaded. **A clip's sound leaves only as one compressed Opus file in the body of `POST /transcribe`, only for a signed-in person with Cloud chosen in the drop zone's card, and is forwarded only to the transcription provider; the words come back in its response** (cloud transcription release). **v1.4: a long video's sound leaves the same way, as compressed windows of at most 90 s, one `POST /transcribe` each, always: the long path has no on-device choice, and the long page says so before the work (§43.6).** A stock search query leaves only in `POST /broll/search` (P-17). Transcript text, word timings and prosody scores leave only in the body of `POST /plan`, only while "AI edit plan" is on (PS v1.2 §12.7, §15, §17) | No route accepts media but `POST /transcribe`'s `audio/ogg` (§22.1, DC-14); only `usecases/transcribe-cloud.ts` sends audio, and (V6.1, as built) `usecases/transcribe-source.ts` a long source's windows; DTOs carry no free-text field except `email`, `PlanRequest.words[].text` and the words of `TranscribeResponse`; only `usecases/request-plan.ts` builds a `PlanRequest`; server cannot link media crates (§7); fetch exists in four files (§7); CSP `connect-src` (§24.3) | `privacy-network.spec.ts` (plan on and plan off); `no_media_routes.rs`; `transcribe_route.rs`; `cloud-transcribe.spec.ts` (identity, On device sends nothing); `events.types.test.ts`; `log_redaction.rs` (plan and transcribe flows) | Settings → "What leaves your device" with the sound row and the provider; the drop zone's card; "Verify it yourself" note; J4 and J10 copy |
| P-2 | Only allowlisted hosts are contacted (PS §17) | `allowlist-hosts.ts`; CSP; `check-hosts.mjs` | `privacy-network.spec.ts` assertion 1 | Settings lists the two hosts |
| P-3 | Analytics carry only allowlisted names with numeric/enum properties; never file names, transcript or caption text, event parameters or free text (PS §17) | `AnalyticsEvent` has no `String` field; `deny_unknown_fields`; client `track()` accepts only the generated union | `events.types.test.ts` (type-level); `analytics_allowlist.rs` (server rejects extras and strings); `privacy-network.spec.ts` assertions 3 and 5 | Event list rendered on the settings page |
| P-4 | The "What leaves your device" page matches the allowlist exactly (PS §20.1) | Page renders `ANALYTICS_EVENT_DOCS`, generated from `analytics.rs` | `settings-what-leaves.spec.ts` compares rendered rows with `gen/api.ts` | The page itself |
| P-5 | No request during processing other than allowlisted beacons, the `GET /healthz` warm-up, in cloud mode the one `POST /transcribe` (cloud transcription release), and (plan on) the one `POST /plan` (PS §17, v1.2). v1.4: a long source sends one `POST /transcribe` per window, one `POST /shorts`, and per short one `POST /plan`; with stock on, `POST /broll/search` and `GET`s to `cdn.pixabay.com` for the items being resolved | Use-cases between import and `ready` import no `net` module except `transcribe-cloud.ts` and `request-plan.ts` (one boundaries edge each) | `privacy-network.spec.ts` assertion 4; `cloud-transcribe.spec.ts` (On device: only `/events`) | Processing card: "Everything runs on your computer" on the device, the cloud line in cloud mode |
| P-15 | (v1.3; v1.4) No pixel of a source or an asset, and no audio sample of an asset, leaves the browser; a source's sound leaves only as P-1's windows; vision reaches the AI only as enums and indexes (INV-28); chat text, sentence text and asset labels leave only in the AI request bodies, only with the AI switch on, and the server neither stores nor logs them | Types of `ShotLine`, `AssetLine`, `ChatEditRequest`; one use-case per route | `privacy-network.spec.ts` (long path, chat, assets); `log_redaction.rs` | The settings table lists each |
| P-14 | The server never stores or logs a plan request's transcript, and forwards it only to the configured AI provider (v1.2) | `/plan` handler holds the body in memory only; no DB write of `PlanRequest`; `log.rs` logs no body; one provider adapter selected by config | `log_redaction.rs` (marker words in a plan request); a grep test that no `sqlx` query in `server/src/plan/` writes text | Settings names the provider and its data-use terms (TE-15) |
| P-16 | (cloud transcription release) The server never stores or logs a clip's sound or the words returned for it, and forwards the sound only to the configured transcription provider, whose retention is switched off (Zero Data Retention) | `transcribe/` names no database; `route.rs` logs a kind, a length bucket and the status; `groq.rs` never reads an error body | `log_redaction.rs` (marker word and marker bytes); `transcribe_route.rs` (no `sqlx` under `transcribe/`) | The Privacy Policy's "Cloud captions"; the settings table |
| P-17 | (v1.4, V6.8) A stock search query leaves only in `POST /broll/search`, only while the stock switch is on; the server forwards it only to Pixabay, never stores or logs it, and caches results by its hash. Stock video comes from `cdn.pixabay.com` straight to the browser, and nothing goes back | `server/src/broll/` names no database; the cache key type is a hash; `net/stock-fetch.ts` is the one caller of host 4 | `broll_search_route.rs`; `log_redaction.rs` (marker word in a query); `stock-broll.spec.ts` (switch off: no search, no CDN request); `privacy-network.spec.ts` | Settings lists the switch, Pixabay and what is sent; "From Pixabay" on each stock asset |
| P-6 | The model stays in the browser and is verified (PS §10 J4, §17) | Bundled manifest hashes; OPFS-only cache adapter | `download.test.ts`; `model-download.spec.ts` | Settings shows model version, cache size |
| P-7 | "Clear local data" removes clips, transcripts and render cache (PS §20.1) | §23.3 | `clear-local-data.spec.ts` | Settings button |
| P-8 | Account deletion removes server records within 30 days (PS §17) | §22.7 (immediate) | `account_delete.rs` | Account page |
| P-9 | "Export my account data" returns server-side records only (PS §17) | §22.7 | `account_export.rs` | Account page |
| P-10 | Analytics retained 90 days; not joined to users (PS §16) | No `user_id` column on `analytics_events`; purge task | `analytics_allowlist.rs` (schema assertion + purge) | Stated on the settings page |
| P-11 | File names and paths are never stored or sent (PS §15) | `clips` store has no name field; download name is generated | `privacy-network.spec.ts` assertion 3; `restore.spec.ts` inspects the store | Recent clips show date and duration only |
| P-13 | No metadata of a founder recording reaches anything the app or the tests publish (v3implementation D-100) | `gen_fixtures.sh` writes every derived fixture with `-map_metadata -1 -map_chapters -1 -fflags +bitexact`, so no `creation_time`, `location` or `encoder` tag of the reference clip is copied; a recording that carries a location (`testclips/v3testclip.mp4`) is never committed, uploaded or derived into a fixture; the muxer writes no `udta` and zero timestamps | One D1-only case of `export-creator.spec.ts` exports `v3testclip.mp4` and asserts with `ffprobe` that the output has no tag whose key contains `location` or `ISO6709`, no `creation_time`, `title`, `comment` or `encoder` from the source, and no `udta`, `meta` or XMP `uuid` box carrying `©xyz` or `ISO6709` (an early slice of verifier check 11) | Not user-facing: a rule for the fixtures and the corpus. The same promise covers clips lent to the corpus, whose transcripts are never written anywhere (v3implementation D-112) |
| P-12 | No public compliance claim or absolute "never leaves your device" wording before legal review (PS §17) | All copy in `messages.ts` | `messages.test.ts` fails on the banned phrases list ("never leaves", "GDPR", "DPDP", "CCPA", "SOC 2", "compliant") | Landing uses the PS §10 J1 and J4 wording |

### 25.2 `privacy-network.spec.ts`

```text
capture: every request from the page, all workers and any service worker (CDP Network domain), from first navigation
scenario: land → drop speech fixture → first-run model download → processing → preview 5 s → edit one word
          → toggle one event → sign in (fake API on the app origin) → export → download → open settings
assert:
 1 every request host ∈ allowlist-hosts.ts
 2 asset-origin requests: GET/HEAD only, no body, no query string, no cookie header
 3 no URL, header or body contains: any fixture transcript word of 5+ letters (case-insensitive) — except the
   body of the one POST /api/v1/plan (v1.2), the edited replacement word (in any request, including /plan),
   the fixture file name, or any 32-byte window sampled from the source file, the decoded PCM, the program
   audio or the exported MP4 (20 random windows each)
 4 between clip_accepted and pipeline_done: only POST /api/v1/events, one GET /api/v1/healthz and at most one
   POST /api/v1/plan (plus model GETs on first run); between export_started and export_done: only
   POST /api/v1/events
 4b (v1.2) a second run with "AI edit plan" off: no POST /api/v1/plan and no transcript word in any request
 5 every /api/v1/events body parses as EventsBatch from gen/api.ts with no extra keys
 6 no WebSocket, WebRTC, EventSource or sendBeacon activity
 7 (v1.4, long path) a 12-minute source through /app/long: audio leaves only as audio/ogg bodies of
   POST /api/v1/transcribe carrying the window header; no 32-byte window of the source file or of any
   part copy in any request; no frame or file name anywhere
 8 (v1.4, V6.8) stock on: search words only in POST /api/v1/broll/search bodies, cdn.pixabay.com GETs only
   with no body and no cookie, no asset bytes anywhere; stock off: neither appears
```

The suite blocks every release (PS §18 "privacy-claim regression").

### 25.3 Server-side data inventory

| Data | Table | Retention |
|---|---|---|
| Email | `users`, `magic_links`, `platform_waitlist` | Until account deletion / 1 day / until deletion or request |
| Session records (hashed tokens) | `sessions` | Until expiry or revocation, then purged |
| Plan and subscription status | `subscriptions` | Until account deletion |
| Export receipts (`export_id`, time, plan) | `usage_receipts` | Until account deletion |
| Processed webhook ids | `webhook_events` | 90 days |
| Anonymous allowlisted events | `analytics_events` | 90 days |
| AI request counts per user per month (v1.3) | `ai_usage`, `ai_shorts_grants` | Until account deletion |
| Plan-request bodies (transcript text and timings) (v1.2) | none: memory only, for the duration of the request | Not stored |
| A clip's compressed sound and the words returned for it (cloud transcription release); from v1.4 also each window of a long source | none: memory only, for the duration of `POST /transcribe` | Not stored |
| A stock search query (v1.4, V6.8) | none: memory only, for the duration of `POST /broll/search`; the results (Pixabay ids, sizes, URLs) are cached 24 h under the query's hash, as Pixabay's terms require | Not stored; the hash cannot be read back to the words |

**Built: cloud transcription** (cloud transcription release). A signed-in person's choice in the drop zone's card, Cloud by default, replaces PS §12.6's per-clip prompt; P-1 is revised for it and P-16 added; the provider's Zero Data Retention is on; the founder approved the wording, and a lawyer's review of it is on the launch checklist (PS §17). (The v1.2 plan request sends text only and is specified in §40.7 and P-1, P-14.)

---

## 26. Testing Architecture

| Level | Scope | Tooling |
|---|---|---|
| Unit (Rust) | Every public function of the pure crates and the server modules | `cargo test` |
| Unit (TypeScript) | Stores, machines, blockers, `net/http`, outbox, RPC, download, entitlement, analytics client | Vitest |
| Type-level | Unit newtypes cannot mix; analytics props have no free text | `trybuild` (`offcut-types/tests/ui/`); Vitest `expectTypeOf` (`events.types.test.ts`) |
| Property | Chain output length equals input length for any input, number parse/format, mux → demux round trip | `proptest` (`duration_preserved.rs`, `numbers_prop.rs`, `mux_roundtrip.rs`) |
| Conformance | PS §9.4 rejection table; MP4 output against independent tools | `probe_rejections.rs`; `verify/verify_mp4.py` |
| Golden | Display lists (exact); rendered frames (tolerant) | `insta` (`display_list_snapshots.rs`); `golden_frames.rs` on a software adapter |
| Integration (server) | Every route against a real Postgres with fake `Mailer` and `BillingProvider` | `cargo test -p offcut-api` with a Postgres service container |
| Integration (client) | `run-pipeline.ts`, `start-export.ts` with fake workers and fake API | Vitest |
| Real-platform | The media pipeline in Chrome stable with WebGPU and WebCodecs | Playwright on `windows-latest` (`e2e-media.yml`, TE-10); `pnpm e2e:device` on R1/R2 |
| End-to-end | One suite per journey and per promise (§27) | Playwright with `helpers/fake-api.ts` |
| Benchmark | Stage timings and memory on R1/R2 | `bench/device-bench.ts`; results committed to `bench/results/` |
| Regression | 40+ clip corpus; labeled detector set; benchmark deltas against the last committed result | `corpus/run_corpus.ts`; `precision_recall.rs`; `device-bench.ts --compare` |

**Fixture matrix.** No fixture is committed and none is a new recording (v3implementation D-76, D-77). `fixtures/gen_fixtures.sh` (ffmpeg 9.0 CLI, development and CI tool only; `pnpm fixtures`) writes every fixture into `fixtures/.gen/` (git-ignored):
- **Speech fixtures are derived** from the reference clip (v2implementation D-64; `testclips/` or the asset-host copy in `fixtures/.cache/`, D-39) and keep the names below: the portrait and landscape clips are the cut 0.960-60.960 s of the reference scaled to 1080x1920 and 1920x1080 at 30 fps CFR, with a white frame at 5.000 s and 55.000 s; the no-events clip is sentences 12-14; the long-pause and sparse clips splice reference speech with generated room tone (`anoisesrc` pink noise, seed 7). `fixtures/speech/README.md` says, per clip, that it is derived and from which seconds. A real recording stored later in `testclips/` under the same name is preferred by the generator, and no code changes (v3implementation D-76).
- **Every `ok_*` and `rej_*` fixture is synthetic**, made from ffmpeg's `lavfi` sources (`testsrc2` video, a 440 Hz `sine`, `anullsrc` silence), never from a recording (v3implementation D-77).
- **Deterministic and small** (v3implementation D-111). The input options (`-y -hide_banner -loglevel error -nostdin`) come before the first `-i` and the output options (`-threads 1 -fflags +bitexact -flags:v +bitexact -flags:a +bitexact -map_metadata -1 -map_chapters -1`) after the last, since an output option placed before an input applies to that input; H.264 adds `-bf 0 -g 30`. Each recipe uses the lowest size that shows its property (320x180 where size does not matter). Two derived clips keep their names though their lengths differ: `speech_20s_noevents.mp4` is 14.4 s and `speech_30s_sparse.mp4` 30.0 s. The version check accepts `ffmpeg version 9.0` and `n9.0` (Linux static builds print the `n`).
- **PCM for the five speech fixtures only**: `<file>.pcm48.f32`, mono 48 kHz `f32le`, decoded by ffmpeg, because Rust has no AAC decoder. The synthetic signals of `loudness_target.rs` are made inside the test.
- **`fixtures/.gen/STAMP`** holds the ffmpeg version and the SHA-256 of the script, the manifest and the reference clip. A test that reads a fixture first checks it and fails with "run pnpm fixtures" when it does not match. **Six Rust test files read generated fixtures**: `demux_fixtures.rs`, `probe_rejections.rs`, `loudness_target.rs`, `duration_preserved.rs`, `chain_no_clicks.rs` and `prosody.rs` (the last two through their speech PCM cases).

The generator takes 118-166 s on D1 and 83-94 s on the CI runners, where it runs on every pull request (measured in V3, docs/v3/v3changelog.md, Prompts 63 and 87). `fixtures/manifest.json` (committed) lists every generated and derived fixture (25): name, recipe kind, expected outcome (`accept` with the `ClipInfo` fields a test asserts, or the `RejectReason`), the `ffprobe` properties step 8 checks, and the tests that read it.

| Fixture | Properties | Expected | Used by |
|---|---|---|---|
| `speech/speech_60s_portrait.mp4` | Derived from the reference clip (PS §20.2): its 0.960-60.960 s, cropped and scaled to 1080x1920, 30 fps CFR, H.264/AAC, sentences 1-14, white frames 150 and 1650 (v3implementation D-76) | Accepted; four events (the 60 s cut ends before the $12k sentence) | Most E2E suites, bench, verifier, `demux_fixtures.rs`, `loudness_target.rs`, `chain_no_clicks.rs`, `prosody.rs` |
| `speech/speech_60s_landscape.mp4` | The same cut scaled to 1920x1080 | Accepted; crop control shown | `review-edit.spec.ts`, `framing.rs`, `loudness_target.rs` |
| `speech/speech_20s_noevents.mp4` | The reference's sentences 12-14 (46.600-61.000 s), 14.4 s, no numbers or lists | Accepted; zero events; `NO_EVENTS_FOUND` | `pipeline-preview.spec.ts` |
| `speech/speech_20s_long_pause.mp4` | 0-5 s reference speech, 5-10 s generated room tone, 10-20 s reference speech; one white-flash frame in each speech part | Accepted; export is 20 s (never 15 s); words after the pause keep timestamps of 10 s or later; A/V in sync on both sides of the pause | `timeline-preserved.spec.ts`, `duration_preserved.rs` (its audio), `demux_fixtures.rs`, `loudness_target.rs`, verifier |
| `speech/speech_30s_sparse.mp4` | Five 1.1 s pieces of reference speech in generated room tone; under 20% of the clip is speech; one white-flash frame near each end | Accepted; export is 30 s; every pause keeps its length | `timeline-preserved.spec.ts`, `loudness_target.rs`, verifier |
| `ok_vfr.mp4` | Synthetic, 1280x720, 10 s at 30 fps with every fourth frame dropped and timestamps kept | Accepted; `is_vfr`; `avg_fps` 22,000-23,000 milli; duration 9,967 ms; CFR output | `demux_fixtures.rs`, `rejections.spec.ts` (accept case) |
| `ok_rotated_90.mov` | Landscape-coded with 90° rotation metadata (`-display_rotation 90`) | Accepted as portrait; `R270` by ffmpeg's sign convention, display 720x1280 | `demux_fixtures.rs`, `golden_frames.rs` |
| `ok_60fps.mp4` | 60 fps | Accepted; 30 fps output | `demux_fixtures.rs` |
| `ok_90s_exact.mp4` | Exactly 90,000 ms | Accepted | `probe_rejections.rs` |
| `ok_moov_at_end.mp4` | `moov` after `mdat` | Accepted | `demux_fixtures.rs` |
| `ok_portrait_3x4.mp4` | 1080x1440 | Accepted; cropped to 9:16 | `demux_fixtures.rs`, `framing.rs` |
| `ok_two_video_first_disabled.mp4` | Two video tracks (320x180 disabled, 640x360 default) and one audio track | Accepted; the 640x360 track is the video (§15.4) | `demux_fixtures.rs` |
| `ok_bframes.mp4` | Synthetic, 1280x720, 2 s at 30 fps, encoded with B-frames, so with `ctts` | Accepted; constant frame rate; its samples re-muxed come back at 1/30 s | `mux_reorder.rs`, `demux_fixtures.rs` |
| `rej_too_long_91s.mp4` | 91 s | `REJECT_DURATION` | §15.3 tests |
| `rej_4k.mp4` | 3840x2160 | `REJECT_RESOLUTION` | §15.3 tests |
| `rej_hevc.mov` | HEVC | `REJECT_HEVC` | §15.3 tests |
| `rej_av1.mp4` | AV1 in MP4 | `REJECT_VIDEO_CODEC` | §15.3 tests |
| `rej_vp9.webm` | WebM | `REJECT_CONTAINER` | §15.3 tests |
| `rej_no_audio.mp4` | Video only | `REJECT_NO_AUDIO` | §15.3 tests |
| `rej_two_audio.mp4` | Two audio tracks | `REJECT_MULTI_AUDIO_TRACK` | §15.3 tests |
| `rej_mp3_audio.mp4` | MP3 audio | `REJECT_AUDIO_CODEC` | §15.3 tests |
| `rej_120fps.mp4` | 120 fps | `REJECT_FRAME_RATE` | §15.3 tests |
| `rej_no_video.m4a` | Audio only | `REJECT_NO_VIDEO` | §15.3 tests |
| `rej_truncated.mp4` | Cut inside `moov` | `REJECT_CORRUPT` | §15.3 tests |
| `rej_silent.mp4` | Valid video, 5 s of silent AAC (measured in V3: the recognizer gives fewer than `MIN_WORDS` words, docs/v3/experiments.md) | `REJECT_NO_SPEECH` | `rejections.spec.ts` (D1 only) |
| `rej_h264_444.mp4` | H.264 High 4:4:4 Predictive (`yuv444p`), 1280x720, 30 fps, AAC, 2 s; passes rules 1-10 (v3implementation D-78) | `REJECT_DECODE_UNSUPPORTED` where Chrome's `isConfigSupported` refuses it | `probe_rejections.rs` (with `decode_supported = false`), `rejections.spec.ts` (`test.fixme` where Chrome decodes it, §15.3) |
| oversize (generated in the test) | Sparse file over 500 MB | `REJECT_FILE_SIZE` | `rejections.spec.ts` |
| `bad_audio_payload.mp4` | Valid tables, corrupted AAC frames | `E_DECODE_AUDIO` | `failure-recovery.spec.ts` |
| `bad_video_payload.mp4` | Valid tables, corrupted H.264 after 2 s | `E_DECODE_VIDEO` | `failure-recovery.spec.ts` |
| `labeled/*.json` (30 or more; 31 in V3) | Labeled transcripts with expected events, written as text and frozen before the detector's rules (v3implementation D-87): 29 written transcripts, `a01` (the reference clip's transcript in the written form, no emphasis marks) and `a02` (the second recording as recognized, `source: "asr"`, its `words` and `prosody` read from IndexedDB after S8). Format in `labeled/SCHEMA.md`: optional `tags` (the coverage vocabulary the coverage check counts) and `edits` (word index to replacement, `""` hides); an `asr` file whose `words` are still `null` is skipped ("no data yet"), and a file whose `sentences` and `words` do not match its source (a written file holds `sentences` only, an `asr` file `words` and `prosody` only) is refused. A label changes only with a changelog line naming the file, the label and the reason | Precision at least 0.90 | `precision_recall.rs`, `layout_safe_area.rs` |
| `golden/*.png` (14) | 3 styles x 4 event kinds, watermark on, watermark off | Similarity at least 0.98 | `golden_frames.rs` |
| Corpus (40+, external) | Real clips tagged vfr, rotated, noisy, accent (PS §9.6), read from the local directory `OFFCUT_CORPUS_DIR` and listed in `corpus/manifest.json` by id, SHA-256 and tags only, never uploaded (v3implementation D-88); `founder-01` is the reference clip and `founder-02` the second recording | At least 90% export; rest rejected; none crash. TE-12 is "inconclusive" under 10 clips (2 in V3) | `corpus/run_corpus.ts` (Playwright project `corpus`) |
| ASR accuracy set (20, external) | 10 US, 10 Indian/other English accents with reference text (E-10). Does not exist in V3: E-10 is carried to V9, and V3 reads `--wer` on the founder's clips as a smoke reading only (v3implementation D-88) | WER at most 12% | `corpus/run_corpus.ts --wer` |

**What `run_corpus.ts` writes** (v3implementation D-112). Numbers, ids, codes and tags only: a transcript of a clip from anyone but the founder is never written to the repository, a result file or a log. Its `transcripts` mode writes the founder's clips only, through a fixed map from corpus id to labeled file (`founder-02` to `fixtures/labeled/a02-v3testclip.json`; `founder-01` has no `asr` file, `a01` being written), filling `words` and `prosody` and never `expected`.

---

## 27. E2E Testing and Independent Verification

### 27.1 Suites

| Suite | Asserts |
|---|---|
| `landing.spec.ts` | J1: hero copy, supported-browser line, "What leaves your device" link, drop zone and sample button present before any upload; `landing_view` sent; waitlist form posts to `/notify-me` |
| `capability.spec.ts` | J3: check finishes within 3 s; `capability_check` event carries enums only; supported state reaches the editor |
| `unsupported.spec.ts` | Each `UNSUPPORTED_*` (injected) shows its specific message, the supported list, the demo video and the email form (PS §9.3) |
| `model-download.spec.ts` | J4: progress and copy shown; interrupted download resumes from stored bytes; hash mismatch re-downloads; second session skips download |
| `rejections.spec.ts` | J5: every `REJECT_*` fixture shows its message with interpolated limits; `clip_rejected` carries only the code. 16 cases: one per code (14), "start over" and the `ok_vfr.mp4` accept case (its `clips` record). 15 are tagged `@no-gpu` and run on CI's Windows runner; `REJECT_NO_SPEECH` needs a whole ASR pass and runs on the development machine (v3implementation D-93). The oversize file is built in the page (v3implementation D-79); `REJECT_DECODE_UNSUPPORTED` is `test.fixme` where Chrome decodes its fixture (§15.3) |
| `pipeline-preview.spec.ts` | J6-J7: feed shows only real detections; preview plays before sign-in; zero-event clip shows `NO_EVENTS_FOUND`; no long task over 100 ms |
| `preview-sync.spec.ts` | Drift at most 80 ms over 60 s; late frames at most 5% |
| `review-edit.spec.ts` | J8: word edit re-detects its sentence; toggles persist across re-detection by `EventId`; style switch; crop control only for landscape; no control named or behaving like "tighten pauses" / "remove silence" exists; clip duration and audio are identical before and after every edit |
| `signin.spec.ts` | J9/C-8: limits shown before sign-in; link flow in a second tab signs in the first via broadcast; expired and invalid links show their messages |
| `export-free.spec.ts` | J9-J12: counter visible before click; 720x1280 output with watermark passes the verifier; counter decrements; last-export warning before the third; fourth attempt shows the upgrade prompt and does not render |
| `export-creator.spec.ts` | 1080x1920 output without watermark passes the verifier; no prompt; render-cache hit sends no receipt. As built in V5: all 11 checks on a Creator and a Free file; the summary and the question; the same token hits at once with the same bytes; a forged token is refused before any lookup; no room says why and sweeps nothing; an expired Creator token verifies as Free and hits the Free file (INV-9) |
| `timeline-preserved.spec.ts` | INV-5, INV-10 on `speech_20s_long_pause.mp4` and `speech_30s_sparse.mp4`: transcript word timestamps after a pause are not moved earlier (the recognizer may start a word 40 ms early, and runs a phrase on over room tone; v5implementation §25 row 32); the caption for the first word after the 5 s pause appears at its spoken time in the preview; the preview ends at the source duration; the export has `ceil(duration x 30 / 1000)` frames (600 and 900); the verifier passes with `--source`, including check 8 at the start and at the end of each clip. As built in V5, on the long pause: an edit's values and rows, its hit, a miss after undoing everything, and an encoder that fails once (a restart, on the ladder's next entry) and on every entry |
| `checkout.spec.ts` | J12/C-11: navigation to the provider URL; return polling activates Creator; timeout shows `E_BILLING_PENDING`; portal link opens |
| `cancel.spec.ts` | C-12 during import, processing, model download and export; temp files removed; state returns as specified; no receipt after a cancelled export |
| `restore.spec.ts` | C-13: reload at `ready` restores without re-transcription; reload mid-export discards the temp file; `clips` store holds no file name |
| `failure-recovery.spec.ts` | C-14: worker crash, GPU loss (injected), storage quota, decode errors → specific message, retry succeeds, second failure shows the persistent copy |
| `api-cold-start.spec.ts` | C-15: API delayed 45 s → waking copy after 3 s, request succeeds; API down → export with cached token completes and the receipt stays in the outbox |
| `privacy-network.spec.ts` | §25.2 |
| `settings-what-leaves.spec.ts` | PS §12.7 table present; event list equals the generated allowlist; mode "Local only"; model version and cache size shown |
| `clear-local-data.spec.ts` | OPFS and clip stores empty afterwards; refused while busy |

### 27.2 Independent verifier (`verify/verify_mp4.py`)

Uses only the `ffprobe`/`ffmpeg` command-line tools and NumPy. It imports nothing from `crates/` or `web/`, and no library that those use. Invocation: `verify_mp4.py <file> --profile free|creator --expected-duration-ms N [--source <fixture>]`. `N` is the duration the map says, which the test works out and passes apart from the source: with the identity map it is the source's, and from V6.3 the two differ (v5implementation D-210). A file `verify/test_verify_mp4.py` makes with `ffmpeg` shows each check telling good from bad, in CI, where no real export can be made (D-209). Lines `INFO <n>: …` report a reading and judge nothing; a reason names a tag's key, never its value.

| # | Check |
|---|---|
| 1 | Container is MP4 with exactly one video and one audio stream; `moov` precedes `mdat` (faststart) |
| 2 | Video: H.264, `yuv420p`, square pixels, no rotation tag; size is 1080x1920 (`creator`) or 720x1280 (`free`) |
| 3 | Constant frame rate: every frame-to-frame PTS delta equals 1/30 s; frame count equals `ceil(expected_duration_ms x 30 / 1000)` |
| 4 | Audio: AAC-LC, 48,000 Hz, 2 channels |
| 5 | Durations: video within one frame of expected (the source duration); audio within one AAC frame (21.3 ms) of video |
| 6 | Full decode of both streams reports zero errors |
| 7 | Loudness: integrated loudness (ffmpeg `ebur128`) of the stereo stream as it is, within ±1 LU of -14 LUFS |
| 8 | A/V sync and timeline (with `--source`): audio offset by envelope cross-correlation against the source within 20 ms, measured separately over the first 5 s and the last 5 s (10 ms RMS envelopes, lags within ±1 s, correlation at least 0.5; a window with no sound in the source is reported, a late audio start filled with silence), so a removed or shortened pause anywhere in between fails; video offset by white-flash frame positions within one frame (a flash is at least 225 of mean brightness, and 30 above the 30 frames around it) |
| 9 | Watermark: normalized correlation, over the mean of 15 evenly spaced frames scaled to 720x1280 and high-passed by a 9 x 9 box blur, of the watermark region with `watermark_mask_720.png` at least 0.5 for `free`; at most 0.2 for `creator` (thresholds read in V5, §39.3 item 14) |
| 10 | Video bitrate, by packet sizes over the stream's duration, at most 1.5x the profile target: 12 Mb/s `creator`, 6 Mb/s `free` |
| 11 | Metadata: no title, comment, description, artist, album or location tag; `creation_time` absent or the epoch; no source-derived tags (with `--source`); neither `©xyz` nor `ISO6709` in the file's bytes |

---

## 28. Milestone Plan

**Gating rules**
- M0, M1, M2 are the PS §9.9 milestones with the PS dates and gates. `M0.n`, `M1.n`, `M2.n` are this document's build steps inside them.
- A step is done only when every test named in its Definition of Done passes in CI, or the named measurement is committed under `bench/results/`.
- A failed experiment triggers its "If it fails" action (§37) before any dependent step starts.
- No P1 item starts before M2 passes. M3 and M4 contain no build work; they are the PS §20.5 decisions.
- **Read on D1 until R1 exists** (v2implementation D-69, v3implementation D-74). Every gate and measurement that names R1 is read on D1, the development machine, and recorded under the label `d1` (`BENCH_DEVICE=d1`); thresholds do not change. For a budget defined for a 60 s clip, a clip of another length is read against the budget times its length over 60 s (v2implementation D-64). A pass on D1 does not show a pass on an 8 GB laptop with an integrated GPU: TE-13 and the re-read budgets passed on a machine about twice as fast, and are re-read on testers' devices after the final deploy (v3implementation §23 item 21).

| # | Milestone | Depends on | Definition of Done | Experiments |
|---|---|---|---|---|
| M0.1 | Skeleton (week 1): workspace, CI, codegen, static shell on Vercel with §24.3 headers, API on Render with `/healthz`, analytics ingestion, capability check, landing page with waitlist | none | `ci.yml` green; `landing.spec.ts`, `capability.test.ts`, `analytics_allowlist.rs` pass; deployed preview reports `crossOriginIsolated === true`; `/api/v1/healthz` answers through the rewrite | TE-5, TE-6, TE-7, TE-11, TE-9 (onboarding starts); E-1 starts; E-8 beacon live |
| M0.2 | ASR spike (weeks 1-2): model from OPFS in `asr.worker`, word timestamps | M0.1 | `model-download.spec.ts`, `download.test.ts` pass; reference-clip ASR timings on R1 and R2 committed | E-3, TE-1, TE-2 |
| M0.3 | Render/encode spike (weeks 1-2): Vello in `render.worker`, frames → H.264 + AAC → MP4 | M0.1 | `mux_roundtrip.rs` passes; `verify_mp4.py` checks 1-6 pass on a spike export; render + encode timings on R1 and R2 committed | E-4, TE-3, TE-4 |
| M0.4 | Proof of concept (week 3): one clip end to end with captions, NumberReveal, Clean style | M0.2, M0.3 | `pipeline-preview.spec.ts` and `export-creator.spec.ts` pass on the reference clip locally on R1 and R2; `e2e-media.yml` runs them in CI | TE-10, TE-14. **M0 gate: E-1, E-3, E-4 (PS §9.9, §20.4)** |
| M1.1 | Ingest and rejections | M0.4 | `demux_fixtures.rs`, `probe_rejections.rs`, `rejections.spec.ts` | TE-12 |
| M1.2 | Audio enhancement chain, prosody | M0.4 | `loudness_target.rs`, `chain_no_clicks.rs`, `duration_preserved.rs`, `prosody.rs`; `ui.rs` is satisfied by the existing trybuild test still passing: no compile-fail case is added, because `offcut-types` cannot depend on `offcut-dsp` and the unit types the chain uses (`Hz`, `Lufs`, `Dbfs`, `SampleCount`) are covered by `bare_ms_rejected.rs`'s rule (v3implementation D-95) | TE-13 |
| M1.3 | Normalization and detector, four kinds | M0.4 | `numbers_table.rs`, `numbers_prop.rs`, `sentences.rs`, `rules_table.rs`, `redetect.rs`, `determinism.rs` (detect); `precision_recall.rs` at 0.90 or above | E-10 (carried to V9: the 20-clip accent set does not exist; V3 reads a smoke WER on the founder's clips, v3implementation D-88) |
| M1.4 | Scene: three styles, four event visuals, framing, watermark | M1.3 | `display_list_snapshots.rs`, `layout_safe_area.rs`, `framing.rs`, `determinism.rs` (scene), `golden_frames.rs` | none |
| M1.5 | Preview and review UI | M1.2, M1.4 | `pipeline-preview.spec.ts`, `preview-sync.spec.ts`, `review-edit.spec.ts`, `cancel.spec.ts`, `rpc.test.ts`, `clip-machine.test.ts`, `model-machine.test.ts`, `run-pipeline.test.ts` | none |
| M1.6 | Export, verifier, render cache | M1.4 | `export-creator.spec.ts` with all 11 verifier checks; `timeline-preserved.spec.ts`; `start-export.test.ts`, `export-machine.test.ts`, `blockers.test.ts` | E-4 re-measured |
| M1.7 | Backend: auth, entitlement, receipts, account; client sign-in and gate | M0.1 | `auth_flow.rs`, `entitlement_rules.rs`, `usage_receipts.rs`, `account_delete.rs`, `account_export.rs`, `rate_limit.rs`, `log_redaction.rs`, `no_media_routes.rs`, `token_roundtrip.rs`, `profile.rs`, `auth-machine.test.ts`, `entitlement.test.ts`, `http.test.ts`, `signin.spec.ts`, `export-free.spec.ts` | TE-8 |
| M1.8 | Billing | M1.7 | `webhook_idempotency.rs`, `billing_events.rs`, `checkout.spec.ts`; provider test-mode purchase updates `subscriptions` | TE-9. **M1 gate: all P0 working on dev machines (PS §9.9)** |
| M1.16 | (v1.3, V6.1; v1.4) Long source: the long page, `route_probe` with `Entry`, HEVC and 4K, reading in place, windowed audio, cloud transcription in checkpointed windows, source machine, flags | M1.8 | `route_probe.rs`, `audio_region.rs`, `transcribe-source.test.ts`, `route_long.spec.ts`, `high-res.spec.ts`, `long-resume.spec.ts`, `flags_identity.spec.ts` | TE-17, TE-18, TE-20, TE-21 |
| M1.9 | (V6.2) Edit plan: `/plan`, provider adapter, spend cap, quotas, `plans` store, settings switch, catalog kinds, text extras; (v1.5) hook lockup, glow, punch words, motion by rules, easing, the Enhance panel | M1.8 | `validate.rs`, `catalog_complete.rs`, `catalog_snapshot.rs`, `plan_route.rs`, `ai_cap.rs`, `log_redaction.rs` (plan), `run-pipeline.test.ts` (fallbacks), `privacy-network.spec.ts` (plan on/off); (v1.5) `motion.rs`, `easing.rs`, `lockup.rs`, `enhance.rs`, `flags_identity.spec.ts` (DC-16) | TE-15; E-11 instrumentation |
| M1.10 | (V6.3) Cuts and `Assembly`: `timemap::build`, `assemble` splices, jump-cut zoom, cuts review, regions, rebase; part copies (v1.4) | M1.9 | `timemap_build.rs`, `program_splice.rs`, `cuts_applied.spec.ts`, `assembly.rs`, `assembly_readers.rs`, `rebase.rs`, `regions.rs`, `remux.rs`, `source_media.rs`, `stitched.spec.ts` | TE-22, TE-17 (copies) |
| M1.17 | (V6.4) Director: `/shorts`, `validate_shorts`, rules producer, list page, batch export | M1.16, M1.10 | `shorts.rs`, `shorts_prompt_snapshot.rs`, `shorts_rules.rs`, `shorts_route.rs`, `ai_quota.rs`, `run-source-pipeline.test.ts`, `long-to-shorts.spec.ts` | TE-19; E-13 instrumentation |
| M1.18 | (V6.5) Vision and framing; (v1.5) following the speaker | M1.17 | `shots.rs`, `tracking.rs`, `framing.rs`, `vision-pass.spec.ts`; `speaker.rs`, `speaker-framing.spec.ts` | TE-23, TE-27 |
| M1.23 | (v1.5, V6.6a) Picture: the blur pass, the automatic grade, camera matching, face light | M1.18 | `blur.rs`, `grade.rs`, goldens with a grade, `composite.rs` simple-path identity | TE-29 |
| M1.12 | (V6.6b, parts A and B) Structure visuals | M1.23 | Goldens per kind and style; V-5 cases; V-9 | none |
| M1.13 | (V6.6b, parts C and D) Kinetic presets, (v1.5) display fonts, six presets, word pages, video masks, shape transitions | M1.12 | Goldens; `frame_at` budget at the heaviest preset; `composite.rs` simple-path identity | TE-28 |
| M1.14 | (V6.6b, part E) Progress bar, end card `Hold` (v1.5: the hook title is M1.9's) | M1.13 | Goldens; end-card export duration test | none |
| M1.11 | (V6.7) Music (the person's own track, v1.5), ducking, sound cues; (v1.5) synthesized sounds placed by rules, voice tuning | M1.10 | `mix_ducking.rs`, `music-import.spec.ts`, `sound_cues.rs`, `sfx.rs`, `voice_tune.rs`; verifier check 7 on a music export | none (v1.5: TE-16 closed) |
| M1.19 | (V6.8) Layers, B-roll (own and Pixabay stock, placed by the plan or by hand, v1.4), asset box, join transitions (v1.5: zooms), split layout; (v1.5) the quality check | M1.13, M1.18 | `layers_goldens`, `handles.rs`, `join_transition.spec.ts`, `asset-box.spec.ts`, `plan_ops.rs` (`PlaceAsset`, `RemoveAsset`, `AddItem`), `quality.rs`, `quality_corpus.rs`, `broll_search_route.rs`, `resolve-stock.test.ts`, `stock-broll.spec.ts`; verifier 11/11 | TE-14 re-read, TE-26 |
| M1.20 | (V6.9) Chat editing | M1.19 | `plan_ops.rs`, `chat_prompt_snapshot.rs`, `chat_route.rs`, `chat-edit.spec.ts`, `privacy-network.spec.ts` (chat) | E-14 instrumentation |
| M1.15 | (V6.10) Brand kit | M1.14 | Golden matrix under a test kit; `profile.rs` brand-kit cases (INV-9) | none |
| M1.21 | (V6.10) Creator memory, queue | M1.17 | `creator_memory.rs`, `queue.spec.ts` | none |
| M1.22 | (V6.11) Person mask | M1.19 | `matte_goldens`, `prepare-matte.spec.ts`; verifier 11/11 | TE-24, TE-25. **M1.x gate: every PS §9.5 V6.x row works on dev machines** |
| M2.1 | Persistence, restore, quota, clear data, failure recovery | M1.5 | `restore.spec.ts`, `clear-local-data.spec.ts`, `failure-recovery.spec.ts`, `receipt-outbox.test.ts` | none |
| M2.2 | Privacy proof and settings | M1.7 | `privacy-network.spec.ts`, `settings-what-leaves.spec.ts`, `events.types.test.ts`, `messages.test.ts`, `client.test.ts`, `unsupported.spec.ts`, `capability.spec.ts` | E-8 continues |
| M2.3 | Corpus hardening (checkpoint: end of week 8, Sun 6 Dec 2026) | M1.6, M2.1 | `corpus/run_corpus.ts`: at least 90% export and pass the verifier, rest rejected with a code, zero crashes | TE-12 re-checked; PS §20.4 week-8 rule |
| M2.4 | Performance on R1 and R2 | M2.3 | `bench/device-bench.ts` results within §30 budgets committed; `api-cold-start.spec.ts` | E-3, E-4 final; TE-11, TE-14 re-checked |
| M2.5 | Launch gate (end of week 10) | all above | PS §20.1 checklist and §29 complete; live-mode checkout; manual phone playback and two platform uploads; production header check; `scripts/metrics.sql` returns the PS §20.5 metrics | E-2 (weeks 6-9), E-5 (weeks 8-10), E-7 instrumentation live. **M2 gate** |
| M3, M4 | Decisions (PS §20.5) | M2.5 | Metrics read from `scripts/metrics.sql` | E-6, E-7, E-8, E-9 |

---

## 29. Definition of Done

Technical additions to PS §20.1. All must hold at M2.5.

- [ ] Every test file listed in §5 exists, runs in CI and passes.
- [ ] `scripts/check-file-tree.mjs`, `check-gen-clean.sh`, `check-copy-codes.mjs`, `check-hosts.mjs` pass.
- [ ] `cargo clippy -D warnings`, `cargo deny check`, ESLint and `tsc --noEmit` are clean with zero suppressions outside test code.
- [ ] Every `ErrorCode`, `RejectReason`, blocker and unsupported reason has copy in `messages.ts` and a passing test named in §11.2, §15.3, §12.5 or §3.
- [ ] Every (assumption) in §39 is either measured and replaced by its value, or still listed with its experiment.
- [ ] Every experiment in §37 has a recorded outcome in `docs/` or is explicitly still open in §39.
- [ ] All 11 verifier checks pass on Free and Creator exports of both reference clips on R1 and R2.
- [ ] `privacy-network.spec.ts` passes on the production build served with the production headers.
- [ ] Production responses carry every header in §24.3; `crossOriginIsolated` is true.
- [ ] The hosting-plan terms (TE-5) are recorded in writing before live checkout is enabled; no paid plan is used.
- [ ] Voice chain target is -14 LUFS within ±1 LU (PS §20.1 delegates the target here), unless M1.2 replaced the value.
- [ ] A cold API (service asleep) does not prevent import, processing, preview or review.
- [ ] No source file exceeds 700 lines excluding tests (400 until 2026-10-28) (assumption for reviewability, PS §18 "complexity debt"); enforced by `scripts/check-file-tree.mjs`, which also counts lines.

---

## 30. Performance Requirements

**Reference conditions (PS §20.2).** 60-second clip, 1080x1920 or 1920x1080, 30 fps, H.264, about 150 words; model cached. R1: 2021-class Windows laptop, 8 GB RAM, integrated GPU, Chrome stable, plugged in. R2: Apple M1, 8 GB, Chrome stable. Fixture: `speech_60s_portrait.mp4` and `speech_60s_landscape.mp4`. Measurement: `bench/device-bench.ts`, median and p90 of 10 runs, one result file per device, date and clip (`<device>-<date>-<clip>.json`, six stages from V3: `audio_chain` added).

**Read on D1 until R1 exists** (v2implementation D-69, v3implementation D-74). Every gate and measurement that names R1 is read on D1, the development machine, and recorded under the label `d1` (`BENCH_DEVICE=d1`); thresholds do not change. For a budget defined for a 60 s clip, a clip of another length is read against the budget times its length over 60 s (v2implementation D-64). A pass on D1 does not show a pass on an 8 GB laptop with an integrated GPU: TE-13 and the re-read budgets passed on a machine about twice as fast, and are re-read on testers' devices after the final deploy (v3implementation §23 item 21).

| Metric | Budget on R1 | Source |
|---|---|---|
| Probe + audio extraction | 3 s | PS §20.2 |
| Local transcription | 20 s | PS §20.2 |
| Detection + scene build | 2 s (1 s + 1 s) | PS §20.2 |
| Audio chain | 4 s | PS §20.2 |
| Render + encode | 90 s | PS §20.2 |
| Mux + finalize | 2 s | PS §20.2 |
| Median total | 121 s | PS §20.2 (3 + 20 + 2 + 4 + 90 + 2) |
| p90 total | 180 s | PS §20.2 |
| First-run model download + initialization at 25 Mbps | at most 90 s | PS §20.2 |
| Capability check | 3 s | PS §9.3 |
| Preview drift over 60 s | at most 80 ms | PS §20.1 |
| Preview late frames | at most 5% | (assumption, M1.5) |
| First preview frame after `ready` | at most 500 ms | (assumption, M1.5) |
| Edit to updated preview (`redetect` + `setScene`) | at most 150 ms | (assumption, M1.5) |
| Peak tab memory | at most 1.5 GB | (assumption, TE-14) |
| API p95, warm | at most 150 ms | (assumption, M2.4) |
| API cold request | within 70 s | (assumption, TE-11) |

The PS totals assume sequential stages; ASR and the voice chain run in parallel here, which can only lower the total. The promise is not tightened.

**Decision rule when a budget is missed (PS §20.4).**

| Miss | Action | Decided at |
|---|---|---|
| ASR median above 145 s for a 60 s clip on R1 (D-67; was 40 s) | Smaller model; stronger caption editing; cloud fallback remains a P1 trigger | M0, re-checked at M2 |
| Render + encode above 150 s on R1 | Canvas2D overlay path; cap input at 60 s and 30 fps | M0 and M1 |
| Denoise exceeds the audio budget | High-pass + loudness only | M1 |
| Corpus pass rate under 90% at week 8 | Remove the least valuable accepted input rather than slip launch | Week 8 |
| Between budget and the thresholds above | Ship; record the miss in §39; keep stage telemetry | M2.4 |

---

## 31. Resource Management

Memory and GPU objects are the scarce resources: the minimum supported device reports 4 GB (PS §9.3) and R1 has 8 GB with an integrated GPU sharing system memory.

| Object | Rule | Owner |
|---|---|---|
| `VideoFrame` (decoded or canvas-made) | Closed by the code that obtained it, in `finally`; never stored outside `VideoSource`'s queue; queue at most 6 (preview) or 8 (export) | `video-source.ts`, `export-loop.ts`, `preview-loop.ts` |
| `EncodedVideoChunk` / `EncodedAudioChunk` | Copied into the muxer immediately; not retained | `encoders.ts` |
| `VideoEncoder` queue | Feed only while `encodeQueueSize <= 4` | `export-loop.ts` |
| `VideoDecoder` / `AudioDecoder` | One of each per worker; closed on session close, cancel and failure | `video-source.ts`, `audio-decode.ts` |
| ASR session | Loaded for the ASR stage only; disposed before the render session opens | `asr.worker` |
| wgpu device, Vello renderer, textures | One device per `RenderSession`; overlay and target textures reallocated only on resize | `RenderSession` |
| PCM buffers | Transferred between threads, never copied; `pcm16` dropped after ASR; `pcm48` dropped after `runChain` | producing worker → main → consuming worker |
| Source file | Read through an OPFS sync handle; never loaded whole into memory | `media.worker`, `render.worker` |
| Export output | Streamed to OPFS; never assembled in memory | `opfs-sink.ts` |
| OPFS sync access handles | One per file at a time; closed in `finally`; `media.worker` closes the source before `render.worker` opens it | opening worker |
| Object URLs | Revoked 60 s after the download click | `start-export.ts` |
| Workers | At most four; restart budget §14.3 | `pool.ts` |
| Server DB connections | Pool of 5 | `server/src/db/mod.rs` |

**Estimated peak budget (assumptions measured by TE-14 with `performance.measureUserAgentSpecificMemory` and the browser task manager)**

| Phase | Component | Estimate |
|---|---|---|
| ASR | Model session and runtime (small-size model; the 700 MB estimated for the base-size files, scaled by file size) | 1,200 MB |
| ASR | `pcm16` + `pcm48` + chain working copies (5.8 + 17.3 + 34.6 MB, derived) | 58 MB |
| ASR | App shell, `offcut_core.wasm` heaps in three workers | 200 MB |
| ASR | **Phase total** | **about 1.46 GB** |
| Render | `offcut_render.wasm` heap, fonts, scene | 150 MB |
| Render | Decoded frame queue: 8 x 1920 x 1080 x 4 B (derived) | 66 MB |
| Render | Overlay + target textures at 1080x1920 (2 x 8.3 MB, derived) and Vello atlases | 80 MB |
| Render | Encoder and decoder internals | 150 MB |
| Render | `out48` (17.3 MB, derived), app shell | 118 MB |
| Render | **Phase total** | **about 0.56 GB** |
| Any | **Peak (phases do not overlap, §12.4)** | **under 1.5 GB budget** |

Disk: source up to 500 MB (PS §9.4), model up to 260 MB (PS §10 J4), audio 17.3 MB, export up to 91.8 MB (§21.2) per clip; `quota.ts` enforces eviction (§23.3).

---

## 32. Observability and Diagnostics

- **Client, production.** The only telemetry is the allowlisted events of §22.6, batched by `analytics/client.ts` (flush every 10 s, on `visibilitychange`, and at 20 queued events; failures dropped silently). Pipeline stage timings are `performance.now()` deltas measured on the main thread around each worker call: `probe_audio` = `importAndProbe` + `extractAudio`, `asr` = `load` + `transcribe`; export timings come from the worker's `stageTimings` (v2implementation D-35). No console output in production builds (`no-console` lint; dev logging is behind `import.meta.env.DEV` in `config/env.ts`). No third-party error tracker.
- **Client, "Report this" button (PS §14).** Sends one `client_error {error_code, stage}` event. Nothing else.
- **Client, local diagnostics.** `AppFailure.detail` and a ring buffer of the last 200 state transitions are kept in memory, shown on `/settings` in dev builds only, and never sent.
- **Server.** Structured JSON logs to stdout (§24.4), retained by Render per its free-tier policy (external; TE-11). Uptime: the free monitor on `/api/v1/healthz` (PS §20.1) with email alerts.
- **Product metrics.** `scripts/metrics.sql` computes the PS §20.5 metrics: activation (`clip_accepted` → `export_done` by `anon_id`), "would you post this" (`post_export_answer`), 14-day repeat (`usage_receipts` per user), capability pass and reject rates, active paid (`subscriptions`). It is run by hand against the database; no dashboard service is deployed (PS §20.6 "analytics dashboard" is satisfied by these queries; assumption, §39).
- **What leaves the device.** Exactly §24.1 rows 1-2: static GETs, model GETs, `/api/v1` requests whose bodies are the DTOs of §10.7.
- **What leaves the server.** To the email provider: recipient address and the magic-link message. To the merchant: user id, email, offer. To Postgres: §23.2. Nothing else.

---

## 33. Development Tooling and Hosting

**Workspace.** One Cargo workspace (root `Cargo.toml`: `crates/*`, `server`) and one pnpm workspace (`web`, plus root scripts).

**Scripts (root `package.json`)**

| Script | Does |
|---|---|
| `pnpm dev` | `scripts/build-wasm.sh --dev --watch` + `vite` with COOP/COEP dev headers and a proxy for `/api/v1` to a local `offcut-api` |
| `pnpm build:wasm` | `scripts/build-wasm.sh`: `cargo build --target wasm32-unknown-unknown --release` for both wasm crates, `wasm-bindgen`, `wasm-opt -O3` |
| `pnpm gen:types` | `scripts/gen-types.sh`: `ts-rs` export to `web/src/gen/` |
| `pnpm build` | `gen:types`, `build:wasm`, `vite build`, `scripts/check-hosts.mjs` |
| `pnpm check` | fmt, clippy, deny, ESLint, `tsc`, `check-file-tree`, `check-gen-clean`, `check-copy-codes` |
| `pnpm test` | `cargo test --workspace` + `vitest run` |
| `pnpm e2e` / `pnpm e2e:media` / `pnpm e2e:device` | Playwright: non-media suites / media suites / all suites headed on this machine |
| `pnpm fixtures` | `fixtures/gen_fixtures.sh`: every generated and derived fixture into `fixtures/.gen/`, then `STAMP` (§26) |
| `pnpm corpus` | `corpus/fetch_corpus.sh` (checks the local clips of `OFFCUT_CORPUS_DIR` against the manifest's SHA-256) + `corpus/run_corpus.ts`, run as Playwright project `corpus` (v3implementation D-88). Options go straight after the script name (`pnpm corpus --grep founder-02`): after a `--`, Playwright ignores them |
| `pnpm bench:device` | `bench/device-bench.ts` (Playwright project `bench`), writes `bench/results/<device>-<date>-<clip>.json` with six stages, `audio_chain` included (v3implementation D-91) |
| `pnpm verify <file>` | `verify/verify_mp4.py` |
| `pnpm verify:test` | `verify/test_verify_mp4.py`: each check against a good file and a bad one, with no browser (v5implementation D-209) |

`bench:device` takes `BENCH_RUNS`, default 10 (v5implementation D-200).

**CI pipeline, in order (`ci.yml` unless noted)**

First, on every event, the job `changes`: the secret scan, and whether code changed. A pull request that changes only documents skips `ci` and `e2e-media`, the scan still run; a push to `main` always runs everything (v5implementation D-199). A skipped job blocks nothing: `main` has no branch protection. `node scripts/check-file-tree.mjs` by hand before pushing an edit of §5, which a documents-only pull request does not run. After step 4 the Linux job runs the verifier's own test.
1. Checkout; restore Cargo and pnpm caches.
2. `cargo fmt --check`; `cargo clippy --workspace -D warnings`; `cargo deny check`.
3. `pnpm gen:types`; `scripts/check-gen-clean.sh`.
4. `cargo test --workspace` (Postgres service container for the server; software GPU adapter for `golden_frames.rs`). Before it, the Linux job installs a pinned static ffmpeg 9.0.2 build (archive URL and SHA-256 in `ci.yml`'s `env`, refused on mismatch), restores `fixtures/.cache/` (the reference clip, by the hash in its name) and runs `pnpm fixtures`, so the six fixture-reading test files run, not skip. No fixture is cached between runs (v3implementation D-94).
5. `pnpm build:wasm`.
6. ESLint; `tsc --noEmit`; `check-file-tree.mjs`; `check-copy-codes.mjs`.
7. `vitest run`.
8. `vite build`; `check-hosts.mjs`.
9. Playwright non-media suites on Linux with Chrome: `landing`, `unsupported`, `signin`, `checkout`, `settings-what-leaves`, `api-cold-start`.
10. `e2e-media.yml` on `windows-latest` with Chrome stable, called by `ci.yml` before any deploy, on the `web/dist` it built with the E2E test key: `pnpm fixtures` with the ffmpeg the job installs, then the cases tagged `@no-gpu`. A hosted runner has no graphics processor and a clip took over 20 minutes there (TE-10), so the cases that need a processed clip (pipeline, preview, export with `verify_mp4.py`, and `REJECT_NO_SPEECH`) run as `pnpm e2e:media` on the development machine on a keyed build before every merge into `main` (v2implementation D-72; v3implementation D-93, D-94). The rejection suite's 15 `@no-gpu` cases (the 13 refusals before the pipeline, "start over" and the `ok_vfr.mp4` accept case) run on the runner.
11. On `main`: `deploy-api.yml` builds `server/Dockerfile`, pushes to GHCR, calls the Render deploy hook; the web build output is deployed to Vercel as a prebuilt deployment (no Rust toolchain on the host's builders).
12. After deploy: header check against the deployment URL (§24.3).

**Hosting configuration**

| Target | Configuration |
|---|---|
| Vercel (`web/vercel.json`) | Static output `web/dist`; headers of §24.3; rewrite `/api/v1/:path*` → `https://<render-host>/api/v1/:path*`; fallback rewrite of non-file paths to `/index.html`. No functions, no middleware |
| Render (`render.yaml`) | One web service, runtime `image` from GHCR, plan `free`, `healthCheckPath: /api/v1/healthz`, environment variables of §22.8 set in the dashboard (`sync: false`) |
| Postgres | External free-tier instance; TLS required; connection string in `DATABASE_URL` |
| Asset CDN | `scripts/upload-assets.sh` uploads content-hashed files; CORS for `APP_ORIGIN`; `Cache-Control: public, max-age=31536000, immutable` |
| Email | The five SMTP variables of §22.8 in Render; one verified Gmail sender, no domain (TE-8; D-256's risk accepted by the founder) |
| Uptime monitor | `GET https://<render-host>/api/v1/healthz` every 5 min |

**Deployment consequences of the architecture**
- **Hosting plan.** Vercel's free Hobby plan is restricted to non-commercial use (verified 2026-10). Offcut sells subscriptions, so the Hobby plan does not cover the launched product. The founder has decided to deploy on Vercel's free plan and not to pay for hosting (PS A-10), so this is an accepted, open risk rather than a budget line. TE-5 records the plan terms before live checkout is enabled (E-5 presale, week 8); if the terms block the product, the only remedy is the reversal path of TDR-12 (another free static host), never a paid plan. The build output is host-agnostic: static files plus three settings (headers, `/api/v1` proxy, SPA fallback).
- **Deploy order.** Server first, then web. The server rejects unknown request fields, so a new client must never reach an old server. Additive response fields are safe (the client ignores unknown fields).
- **Version skew in open tabs.** A tab loaded before a deploy may request a hashed worker or WASM file that the new deployment no longer serves. `pool.ts` therefore preloads all four worker scripts and both WASM bundles into the HTTP cache at app start (immutable caching makes later instantiation offline-safe).
- **Stateless API.** Render may restart or sleep the instance at any time; nothing but rate-limit buckets is lost.
- **Test-mode production (V6, D-285, D-303).** From V6's merge until the founding release after V6.8 (v1.5; v1.3: after V6.6), production runs Dodo's test-mode keys, with `BILLING_API_BASE_URL=https://test.dodopayments.com` and a test-mode webhook endpoint: a test card buys Creator and no money moves. The founding release switches four variables and registers a live endpoint, then wipes any test-mode `subscriptions` row.
- **Mail from Render (V6, D-252).** Render's free web services block outbound 25, 465 and 587; the relay is reached on 2525 with STARTTLS.
- **Model updates.** A new `modelId` is a new CDN path; old files stay until no deployed manifest references them.
- **Domain.** The system works on the hosts' default domains. A custom domain is not required by the architecture but is likely required by the email provider for sender verification (TE-8); its registration fee is outside the 0 USD infrastructure target.
- **CI minutes.** `e2e-media.yml` runs on a Windows runner; if free minutes are insufficient for a private repository (TE-10), it runs on `main` only and before release, and `pnpm e2e:device` covers pull requests locally.

---

## 34. AI-Assisted Development Protocol

### 34.1 Context given to the implementer of one file

The implementer (human or model) receives exactly:

1. The per-file specification written from the template below.
2. §2, §7, §10 and §11 of this document in full.
3. The deep-dive section that owns the file (one of §13-§25).
4. The public signatures (not bodies) of every file listed under "Allowed dependencies".
5. `web/src/gen/domain.ts` and `web/src/gen/api.ts` for TypeScript files, or `crates/offcut-types/src/*.rs` for Rust files.
6. The names and one-line purposes of the tests that will judge the file.
7. For a modification: the current content of that one file and of its test file.

The implementer does **not** receive the whole repository, other files' bodies, or the product spec. If the task cannot be completed from this context, the correct output is a list of the missing signatures or contradictions, not a guess.

### 34.2 Per-file specification template (`docs/file-specs/TEMPLATE.md`)

```text
Path:
Purpose:                 one sentence
Responsibility:          what this file owns
Non-responsibilities:    what it must never do (copy the row from §2 / §6)
Allowed dependencies:    exact module or crate paths; anything else is forbidden
Public API:              full signatures, exactly as in this spec
Types:                   new types defined here; types consumed (with their home file)
Per-function contract:   for each public function: input, output, errors, side effects, postcondition
State ownership:         state this file owns; state it reads; state it must not touch
Concurrency:             thread/worker it runs on; cancellation points; reentrancy
Performance:             budget from §30 or the owning section
Security and privacy:    data it may see; data it must not log, store or send
Constraints:             lints that apply; banned APIs; size limit (700 lines)
Definition of done:      named test files that must pass; scripts that must stay green
```

### 34.3 Worked exemplar: `offcut_dsp::chain::run_chain`

```text
Path:        crates/offcut-dsp/src/chain.rs
Signature:   pub fn run_chain(pcm48: &[f32], cfg: &ChainConfig, on_block: &mut dyn FnMut(SampleCount) -> Flow)
                 -> Result<Option<ChainOutput>, DspError>
             pub struct ChainConfig { pub denoise: bool, pub hpf: Hz, pub denoise_mix: f32, pub target: Lufs,
                                      pub ceiling: Dbfs, pub max_gain_db: f32, pub absolute_silence: Lufs }   // defaults: §18.3
             pub struct ChainOutput { pub samples: Vec<f32>, pub loudness: Lufs, pub peak: Dbfs }
             pub enum DspError { Empty, NonFinite }

Input:       pcm48        mono 48 kHz f32, aligned to video time 0 and to the video duration (§15.1)
             on_block     called once per 1 s block with the samples done so far; returning Cancel stops the chain
Output:      Ok(Some(out))  out.samples is out48
             Ok(None)       cancelled
Errors:      Empty (pcm48.is_empty()); NonFinite (any NaN or infinite input sample). Quiet or silent input is not an error.
Side effects: none besides on_block. No clock, no randomness, no logging.

Postconditions (each is a test):
  P1  out.samples.len() == pcm48.len(), for every input length                      (INV-10)
  P2  no feature of the signal moves by more than ALIGN_TOLERANCE between input and output
  P3  a silent span of the input is a silent span of the output at the same position and of the same length
  P4  |out.loudness - target| <= 0.5 LU, unless the gain was capped at max_gain_db or the input measured
      below absolute_silence (then gain is 0 dB)
  P5  out.peak <= ceiling; every output sample is finite
  P6  the sample-to-sample step across every block boundary is at most 2x the local RMS
  P7  on_block returning Cancel ⇒ Ok(None) before the next block starts
  P8  same inputs ⇒ identical output

Algorithm:
  1  if pcm48.is_empty(): Err(Empty);  if any sample is not finite: Err(NonFinite)
  2  work = pcm48.to_vec()
  3  for each 1 s block (the last block may be shorter and is still processed):
       high-pass → denoise (when cfg.denoise) → compressor, each carrying its filter state across blocks
       each stage writes back exactly as many samples as it read; the denoiser's frame delay is removed
       if on_block(done) == Cancel: return Ok(None)
  4  measure integrated loudness of work (BS.1770 gating); apply one gain (capped, or 0 dB when below absolute_silence)
  5  limiter over work with look-ahead; the look-ahead delay is removed, so the length is unchanged
  6  re-measure; if off target by more than 0.5 LU: one corrective gain pass + limiter
  7  debug_assert(work.len() == pcm48.len());  Ok(Some(ChainOutput { samples: work, loudness, peak }))

Not in this file, by design: silence detection, gap or pause analysis, cutting, crossfading, concatenation of
segments, time-stretching. The function takes no word timings.

Callers:     crates/offcut-wasm-core/src/audio_api.rs::run_chain   (only caller; it passes ChainConfig::default(),
             since the binding takes no configuration: v3implementation D-110, §18.1)
Callees:     highpass, denoise, compressor, loudness, limiter modules of this crate
Concurrency: called on audio.worker inside one job; cancellation point per block
Performance: at most 4 s for the 60 s reference clip on R1 (PS §20.2, TE-13)
Privacy:     sees audio samples only; must not log
Constraints: no unwrap/expect/indexing; no allocation per block beyond the working buffer; file under 700 lines
Tests:
  crates/offcut-dsp/tests/duration_preserved.rs
    length_equals_input_fixtures (P1)          prop_length_equals_input (P1: random lengths, proptest)
    speech_silence_speech_stays_20s (P1, P3: 0-5 s speech, 5-10 s silence, 10-20 s speech; output is not 15 s)
    impulses_stay_aligned (P2)                 mostly_silent_clip_keeps_length (P1, P3)
    digital_silence_keeps_length (P1, P4)
  crates/offcut-dsp/tests/loudness_target.rs   (P4, P5)
  crates/offcut-dsp/tests/chain_no_clicks.rs   (P6; also Empty, NonFinite, P7, P8)
Definition of done: the three test files pass; clippy clean; file under 700 lines
```

---

## 35. Architectural Invariants

| # | Invariant | Test |
|---|---|---|
| INV-1 | The browser contacts no host outside `config/allowlist-hosts.ts` | `privacy-network.spec.ts` (1); `check-hosts.mjs`; CSP |
| INV-2 | No request URL, header or body contains media bytes, caption text as edited, event or plan parameters, or a file name; transcript text appears only in the body of `POST /api/v1/plan`, and never while "AI edit plan" is off (v1.2) | `privacy-network.spec.ts` (3, 4b) |
| INV-3 | Import, processing, preview and review complete with the API unreachable. Processing waits on the API only in cloud mode, at most 45 s for a waking API and one 30 s request, and completes on the device when it cannot (cloud transcription release, D-375). **v1.4: the long path is the exception.** It needs a session and cloud transcription, says so before the work, and with the API unreachable mid-job it pauses with Continue, losing nothing (§43.6); a short, once prepared, opens, previews and exports with the API unreachable | `api-cold-start.spec.ts`; `transcribe-cloud.test.ts`; `cloud-transcribe.spec.ts` (fallbacks) |
| INV-4 | The server has no route that accepts more than 64 kB or a media content type, except `POST /transcribe`: `audio/ogg` only, at most 1 MiB, bearer only (DC-14); and `POST /shorts`: JSON only, at most 160 kB, bearer only (DC-15, v1.4) | `no_media_routes.rs` |
| INV-5 | `TIMELINE_INTEGRITY` (v1.2). Every removed span is a validated, unrestored `Cut` of the stored plan; no cut starts or ends inside a word; nothing is reordered or retimed; the export has `ceil(time_map.out_duration x 30 / 1000)` frames. With the identity map (V4-V6, and whenever the plan has no applied cuts) source time equals output time and the export has `ceil(source duration x 30 / 1000)` frames | `timeline-preserved.spec.ts` (identity); `timemap_build.rs`, `cuts_applied.spec.ts` (V6.3); `verify_mp4.py` (3, 5, 8) |
| INV-6 | No `DetectedEvent` exists with confidence below its kind's threshold | `rules_table.rs`; assertion in `precision_recall.rs` |
| INV-7 | Equal scene inputs and renderer version give byte-identical display lists for every frame | `offcut-scene/tests/determinism.rs` |
| INV-8 | Every export has exactly `frame_count` frames at exactly 30 fps and passes the independent verifier | `verify_mp4.py` (3) in both export suites and the corpus run |
| INV-9 | Output size, watermark and (v1.2, V6.10) whether the brand kit is applied to an export are functions of the verified entitlement token only, computed in `export_profile` | `offcut-entitlement/tests/profile.rs`; `export-free.spec.ts`; `export-creator.spec.ts` |
| INV-10 | `AUDIO_DURATION_PRESERVATION`. The audio enhancement chain does not change the duration of the recording: `out48.len() == pcm48.len()`, aligned within `ALIGN_TOLERANCE`; no sample is deleted or shifted because it is quiet. Only `program::assemble` produces another length, exactly `time_map.out_duration` at 48 kHz; with the identity map and no music it returns `out48` sample for sample (v1.2) | `duration_preserved.rs`; `program_identity.rs`; `timeline-preserved.spec.ts`; `verify_mp4.py` (5, 8) |
| INV-11 | Every obtained `VideoFrame` is closed; the live-frame counter is 0 after preview stop and after export | Counter in every build; a non-zero count fails the call with `E_INTERNAL` (v2implementation D-45), exercised by `pipeline-preview.spec.ts` and `export-creator.spec.ts` |
| INV-12 | The ASR session and the render session never coexist | `run-pipeline.test.ts` (ordering of `unload` and `openSession`) |
| INV-13 | A usage receipt is stored at most once per `export_id`; a webhook is applied at most once per `provider_event_id` | `usage_receipts.rs`; `webhook_idempotency.rs` |
| INV-14 | Limits are visible before the action they gate; no upgrade prompt renders during processing, editing or rendering | `export-free.spec.ts` |
| INV-15 | All user-facing text and every number inside it come from `copy/messages.ts` and the files it imports (`copy/landing.ts`, designimplementation D-321), and the generated limits | `messages.test.ts`; `react/jsx-no-literals` |
| INV-16 | Every state change goes through `transition()`; an illegal transition changes nothing | The four machine test files |
| INV-17 | The main thread performs no decode, inference, DSP, render or encode | Boundaries lint; long-task assertion in `pipeline-preview.spec.ts` |
| INV-18 | Analytics events have no free-text property and no user identifier | `events.types.test.ts`; `analytics_allowlist.rs` |
| INV-19 | Server logs contain no email, token, IP address or body | `log_redaction.rs` |
| INV-20 | A model file is used only after its SHA-256 matches the bundled manifest | `download.test.ts`; `model-download.spec.ts` |
| INV-21 | (v1.2) Nothing reads a plan that `validate` has not returned: `build_scene`, `timemap::build` and `assemble` accept only `ValidPlan`, and every rule V-1 to V-8 of §40.4 holds for it | `validate.rs` (one case per rule); `ValidPlan` has no public constructor besides `validate` (trybuild) |
| INV-22 | (v1.2) Every number drawn by a plan item equals `parse_quantity` over the effective transcript words its `QuantityRef` points at; no plan field carries a literal shown number | `validate.rs` V-5 cases; `display_list_snapshots.rs` word-edit case |
| INV-23 | (v1.2) The catalog, the prompt and the output schema come from `offcut-plan::catalog` only; every catalog kind is drawable with a golden frame and every drawable kind is in the catalog; a prompt or schema change bumps `CATALOG_VERSION` | `catalog_complete.rs`; `catalog_snapshot.rs` |
| INV-24 | (v1.2) A failed, refused, slow or switched-off plan request never blocks a clip: the rules plan is used and the clip reaches `ready` | `run-pipeline.test.ts` (each failure kind); `api-cold-start.spec.ts` |

---

## 36. What Not to Build

| Trap | Why |
|---|---|
| FFmpeg compiled to WASM | Rejected by PS §0 item 4 and §9.8: size, licensing review (c)(f). `ffmpeg` appears only as a development CLI in `fixtures/` and `verify/` |
| Local LLM for semantics | PS §9.2 (f); P2: a third producer of `EditPlan` (§40) |
| Cloud transcription for a signed-out person, or of the video | Built for signed-in people only, with the card's On device choice (cloud transcription release; P-1 as revised). Video and frames never leave |
| The browser calling an AI provider; a provider key in the client; streaming model output to the browser | §40.7: the server is the only caller; the browser's allowlist stays two hosts |
| Cuts decided by rules (silence thresholds, pause-length rules) without a plan; cuts inside a word; reordering inside a `TimeMap`; speed changes | PS §9.2, §9.2.1: cuts come only from a validated plan and are shortened pauses, lead/trail silence and (opt-in) fillers and false starts. With the plan off, nothing is cut. v1.3: choosing and ordering whole sentences is the `Assembly`'s job (§44, DC-6); a user-asked cut is a chat op (§49.2) |
| Any `TimeMap` constructor other than `TimeMap::identity` and `offcut-plan::timemap::build`; any time conversion outside `TimeMap` | §40.1 (replaces the v1.1 ban on a mapping layer; the map is built in V4 on purpose, TDR-7) |
| Timeline editor, keyframe UI, sliders | PS §9.2 (d)(f); principle 3 allows toggles, cut and part restores, one crop offset, and from V6.9 chat |
| Style DNA, three versions, remix, multi-brand | PS §9.2 (d)(a)(e). One brand kit per account is built in V6.10 |
| Bundled music without a licence TE-16 has confirmed | PS §9.2 (c). User-supplied music with ducking is built in V6.7 |
| A plug-in system for item kinds; an item kind without drawing code and a golden | §40.3: a kind is a catalog entry plus its layout and draw files (INV-23) |
| User-inserted plan items through any control other than chat | PS §12.4 (e); the user toggles what the plan proposed, and from V6.9 asks in words, through the same validator (§49) |
| Generated B-roll, avatars, scheduling, public API, native app, teams | PS §9.2 (d)(c). The user's own B-roll and Pixabay stock (v1.4) are built in V6.8 (§48) |
| Sending frames to any server, a local caption model, an audio speaker model | §46: the founder's decision of 2026-11-02 (v1.5: two people on screen are followed by matching faces to speech, §46.6) |
| YouTube-link import, a server that fetches or holds video | INV-4 and the platform's terms |
| A virality score | §45: unverifiable; the reason is shown instead |
| Copying or translating Tooscut's code, or adding that folder to this repository | §53.4 |
| Pro/Studio plan code, coupons, credit metering | PS §9.2 (a); flat pricing (PS §11) |
| Safari, Firefox, Linux or mobile workarounds | PS §9.2 (f); unsupported page only |
| Multi-language text shaping, i18n framework | PS §9.2 (f)(e); English only |
| 4K, 60 fps output, other aspect ratios | PS §9.4 (f)(e). 4K **input** is built in V6.1 (§43) |
| Server-side rendering, obfuscation or DRM to enforce the watermark or export limit | PS §11: limits are conversion mechanisms; harder enforcement breaks the cost model |
| `MediaRecorder` export, `HTMLVideoElement` frame capture, DOM/CSS overlays | Rejected in §19-§21: not frame-accurate or not deterministic |
| Canvas2D backend "just in case" | Built only if E-4/TE-3 fail (§19) |
| Rust threads in WASM (`rayon`), nightly toolchain | Parallelism comes from workers; threads are needed only by the ASR runtime |
| A general RPC library, worker-to-worker channels, an event bus | §14: four typed interfaces are enough |
| A custom state framework, Redux middleware, sagas | §12: stores + one `transition()` helper |
| Service worker, offline PWA, background sync | Not required by any journey; adds a network actor that the privacy test must then model |
| Passwords, OAuth, 2FA | PS §17: magic link at MVP; Google sign-in is P1 |
| Redis, queues, cron services, microservices, read replicas | PS §15: one small server; nothing here needs them |
| Functions or middleware on the static host | The static host proxies only; a second runtime doubles the security surface |
| Third-party analytics, error trackers, tag managers, web fonts from a CDN (a font self-hosted in the build, served from the app's own origin as a hashed, immutable asset, is allowed: V4's interface font, Inter, `web/src/ui/assets/inter-latin.woff2`, D-180) | PS §17 and the closed network list |
| Admin dashboard, metrics UI | `scripts/metrics.sql` is enough until M4 |
| A/B testing framework | E-6 needs one enum and one branch |
| GraphQL, OpenAPI generators, tRPC | Fifteen routes; DTOs are generated from Rust by `ts-rs` |
| CSS-in-JS runtime, component library | CSP `style-src 'self'`; about 20 components |
| Storing file names "for the recent clips list" | Breaks P-11; date and duration are shown instead |
| Joining analytics to accounts | PS §16 requires consent; no consent UI exists |
| Resumable exports | An export of at most 90 s restarts faster than a resume protocol can be verified. Long jobs resume between stages and windows (§43.7), never inside an export |

---

## 37. Experiments and Open Technical Decisions

PS experiments (`E-n`) keep their PS pass criteria; the "Method" column states the technical instrument this document provides.

**Read on D1 until R1 exists** (v2implementation D-69, v3implementation D-74). Every gate and measurement that names R1 is read on D1, the development machine, and recorded under the label `d1` (`BENCH_DEVICE=d1`); thresholds do not change. For a budget defined for a 60 s clip, a clip of another length is read against the budget times its length over 60 s (v2implementation D-64). A pass on D1 does not show a pass on an 8 GB laptop with an integrated GPU: TE-13 and the re-read budgets passed on a machine about twice as fast, and are re-read on testers' devices after the final deploy (v3implementation §23 item 21).

| ID | Question | Method | Pass criteria | When | If it fails |
|---|---|---|---|---|---|
| E-1 | Do technical creators want this? (PS §19) | `LandingPage`, `/notify-me` with `wanted = launch`, `landing_view` counts | At least 1,000 visitors and at least 5% join the waitlist; fewer than 1,000 visitors is inconclusive (PS) | M0.1 → M0 gate | Rewrite the pitch once; under 3% again → stop at M0 (PS §20.4) |
| E-2 | Is the output postable? (PS §19) | Alpha build for 15 creators; `post_export_answer`; interviews | At least 8 of 15 would post with at most 2 minutes of changes (PS) | Weeks 6-9 (M2.5) | Identify the gap (captions, events, audio) before launch (PS) |
| E-3 | ASR within budget on R1? (PS §19) | `bench/device-bench.ts`, stage `asr` | Median at most 20 s for the reference clip; p90 total at most 180 s (PS) | M0.2, M2.4 | PS §20.4: smaller model; persistent miss makes cloud fallback P0 per PS §19 |
| E-4 | Render + encode within budget on R1? (PS §19) | `bench/device-bench.ts`, stage `render_encode` | At most 90 s for the reference clip (PS) | M0.3, M1.6, M2.4 | PS §20.4: Canvas2D path; cap input at 60 s / 30 fps |
| E-5 | Will strangers prepay? (PS §19) | `Offer::CreatorAnnualFounding` checkout, live mode | At least 5 paying strangers before launch (PS) | Weeks 8-10 (M2.5) | Investigate price versus value before marketing spend (PS) |
| E-6 | Is privacy a purchase driver? (PS §19) | `landing_view.hero_variant` + `signin_step` rates | Directional difference (PS: qualitative) | M3 | Keep privacy as a supporting claim (PS) |
| E-7 | Are events kept? (PS §19) | `export_done.events_kept / (events_kept + events_disabled)`; `precision_recall.rs` | At least 60% kept; precision at least 90% (PS) | M2.5 → M3 | Tune rules or reconsider the wedge (PS) |
| E-8 | How many visitors can run it? (PS §19) | `capability_check` on the landing page | At least 55% pass (PS) | M0.1 → M3 | 40-55%: Safari/Firefox earlier; under 40%: re-plan (PS) |
| E-9 | Do users return? (PS §19) | `usage_receipts`: second export within 14 days | At least 35% (PS) | M3 | Study why before adding features (PS) |
| E-10 | ASR accuracy on accents and noise? (PS §19) | `corpus/run_corpus.ts --wer` on 20 clips; `export_done.word_edits` | WER at most 12%; median edits at most 8 per 60 s (PS) | V9 (moved from M1.3: no 20-clip accent set exists; V3 read a smoke WER of 0.107 on the founder's two clips, which is not E-10; v3implementation D-88) | Larger model, stronger edit UX, or narrower claims (PS) |
| TE-1 | Can the ASR runtime load only from OPFS, make zero third-party requests, and return word timestamps in a worker? | Spike in `asr.worker` under the production CSP; network capture | Zero requests outside §24.1; word timestamps for the reference clip; both backends run | M0.2 | Second runtime candidate behind `whisper-runtime.ts` (§16) |
| TE-2 | Does the runtime expose per-word confidence? | Inspect outputs on the reference clip | A probability per word is available at no more than 10% time cost | M0.2 | `Confidence(1.0)` for all words; detector scores ignore ASR confidence |
| TE-3 | Does Vello on WebGPU run in a worker on an `OffscreenCanvas`, with `VideoFrame` import, and which capture method is faster? | Spike: render 1,800 frames on R1/R2 with method A (canvas → `VideoFrame`) and B (texture readback); hidden-tab run | A correct frame every time; at least 30 frames/s render-only on R1; works with the tab hidden | M0.3 | Other capture method; then Canvas2D backend (§19) |
| TE-4 | Which H.264/AAC encoder configs work on R1/R2, and does the muxed file play everywhere? | `pickVideoConfig` report; `verify_mp4.py`; manual playback on two phones and two platform uploads | A ladder entry supported on both; verifier 11/11; A/V offset within one frame | M0.3, M2.5 | Next ladder entries; explicit silent pre-roll for AAC priming; bitrate change |
| TE-5 | Is the static host usable as specified: plan terms for a paid product, COOP/COEP headers, `/api/v1` rewrite, proxy timeout against a cold API, forwarded-IP header shape? | Read the current plan terms; deploy the shell; call a deliberately slow endpoint through the rewrite; inspect `X-Forwarded-For` | Terms permit the use (or the founder accepts the terms risk in writing; no paid plan, PS A-10); `crossOriginIsolated` true; a 60 s upstream delay survives the proxy; hop count known | M0.1; decision before week 8 | Reversal path of TDR-12; if only the timeout fails, the client calls `/healthz` until warm before any real request |
| TE-6 | Which free Postgres provider persists beyond 30 days, accepts 5 connections, and fits the data estimate? | Read current terms; run migrations; idle for a week; measure reconnect time | No expiry; reconnect under 2 s; storage quota at least 5x the §23.3 estimate | M0.1 | Other candidate; then any other free Postgres provider (no paid tier, PS A-10) |
| TE-7 | Which asset host gives free egress, HTTP Range, CORS that works under COEP, and files of 150 MB? | Upload a 150 MB file; ranged fetch from the deployed shell | 206 responses; CORS readable; no egress charge in terms | M0.1-M0.2 | Other candidate; then split the model into parts under the host's limit |
| TE-8 | Which email provider delivers magic links within its free quota, and does it require a verified sender domain? | Send 50 links to major mailbox providers; read quota terms | At least 95% inbox delivery within 30 s; quota at least 100 emails/day | M1.7 | Other candidate; prefer one that verifies a single sender address without a custom domain, because a domain registration is a cost outside PS A-10. **Read in V6 (`docs/v6/experiments.md`):** Brevo over SMTP on 2525, one verified Gmail sender, 300 a day. Cut down by the founder on 2026-11-02 to two Gmail mailboxes, every mail in the inbox; Outlook and Yahoo unproven, and Brevo calls a free-mail sender "not recommended". A real send from Render worked |
| TE-9 | Which merchant of record onboards an India-based seller, supports hosted redirect checkout with passthrough data, a customer portal, and retries webhooks? | Start onboarding in week 1 (PS §18); test-mode purchase, renewal, cancel, refund; delay the webhook endpoint | All six `BillingEventKind`s observed; a timed-out webhook is retried; a test purchase turns the next token Creator. **v1.3 (V6): live approval moves to the founding release (BP §8A.8)** | M0.1 (start), M1.8 | Second provider behind `BillingProvider` (PS §18). **Read in V6, test mode (`docs/v6/experiments.md`): Dodo Payments; all six kinds and `PlanChanged` observed from real deliveries; a failed delivery retried after about 6 s with the same `webhook-id`; a test purchase on D1 turned the next token Creator. On 2026-11-04 Dodo approved the identity and bank and held the product review until the live site is public** |
| TE-10 | Can CI run the media suites (WebGPU + H.264 + AAC encode) on a hosted Windows runner within free minutes? | Run `e2e-media.yml` | Suites pass headless; monthly minutes within the free allowance | M0.4 | Run on `main` and pre-release only; `pnpm e2e:device` locally |
| TE-11 | Does the API fit the Render free instance: memory, cold-start time, instance hours with keep-warm, image deploy, log retention? | Deploy; measure RSS, cold start over 10 sleeps, monthly hour use | RSS under 400 MB; cold start under 70 s; one always-on service fits the monthly hours | M0.1, M2.4 | Another free container host for the same image (§22.9) |
| TE-12 | Does `offcut-mp4` demux the real-clip corpus (edit lists, rotation, VFR, phone encoders)? | `corpus/run_corpus.ts` | At least 90% of clips demux and export; "inconclusive" with fewer than 10 corpus clips (V3: 2 clips, inconclusive, v3implementation D-88) | M1.1, re-checked at V9 and M2.3 | JS demuxer behind the same interface (§15) |
| TE-13 | Does the denoiser fit the audio budget in WASM on R1? | `bench/device-bench.ts`, stage `audio_chain` | Chain at most 4 s for the reference clip | M1.2 | `ChainConfig.denoise = false` (PS §20.4) |
| TE-14 | Is peak memory within budget on R1 for a 90 s, 1080p, 60 fps input? | Memory measurement per phase (§31) | Peak under 1.5 GB; no tab crash in 10 runs | M0.4, M2.4 | Smaller queues; preview at 360x640; smaller model |
| TE-15 | (v1.2) Which AI provider gives a free tier that fits the plan quotas, structured JSON output, a latency we can wait for, and data terms we can state? | Read current terms (free-tier request and token limits per minute and per day; whether free-tier data is used for training or retained); run the 31 labeled transcripts through two candidates with the generated prompt and schema; measure parse rate, V-rule drop rate, p50/p90 latency through the Render proxy | Parse rate at least 98% after one retry; median V-rule drops at most 10% of items; p90 latency at most 20 s; daily free capacity at least the launch-month estimate (anonymous + Free + Creator quotas x expected users); terms recorded in `docs/` and summarised on the settings page | V6.2 (before the route ships); re-verified by `check-external-facts.mjs` before launch | Second candidate; lower quotas; if no free tier fits or the terms say free-tier data trains models, the founder decides between a paid tier (A-3), accepting and disclosing the terms, or making the AI plan opt-in |
| TE-16 | (v1.2) Is there a music source whose licence allows bundling tracks inside a paid app, with no attribution the export cannot carry? | Read licences of candidate libraries; check redistribution and "in an app" clauses | A written licence that allows it; otherwise none | V6.7 | Ship user-supplied music only. **Closed (v1.5, founder 2026-11-09): music is the person's own track only, and sound effects are synthesized in code (§47.1), so no licence is read** |
| E-11 | (v1.2, PS §19) Is the AI plan better than the rules? | `plan_result`, `export_done` (`items_kept`, `items_disabled`, `cuts_restored`), `post_export_answer` split by plan source | PS §19 E-11 targets | V6.2 → M3 | PS §19 E-11 action |

**v1.3 experiments:** TE-17 to TE-25 are in §53.1, and from v1.4 TE-26; TE-17 and TE-18 were rewritten in v1.4. E-12, E-13, E-14 are PS §19's.

**Non-blocking decisions**
- Router and store libraries may be swapped; only `routes.tsx` and `state/` depend on them.
- Exact fonts, colors, event timings, watermark size and placement: tuned during E-2.
- Annual discount, Google sign-in, final product name (PS §19 minor): the name is one constant in `messages.ts`.

---

## 38. Technical Decision Records

**TDR-1. Client-only media pipeline with a thin server.** Chosen over server-side processing. Why: PS thesis (near-zero cost per export, privacy claim, flat pricing). Cost: the user's device matrix becomes the support surface; reach is limited to desktop Chromium. Reversal: the consented cloud fallback seam (§16); a full reversal is a different product.

**TDR-2. Rust/WASM core in two bundles, four workers, main-thread hub.** Chosen over a TypeScript core or a single WASM bundle. Why: one implementation of every output-affecting rule, testable natively; workers that do not render never load the renderer. Cost: two build targets; data crosses JS between stages. Reversal: merge the bundles (build config only); the hub can become SharedArrayBuffer-backed without changing message shapes.

**TDR-3. ONNX Runtime Web with a Whisper-family English model, files served from OPFS.** Chosen over hub loading, CPU-only WASM builds and cloud ASR. Why: WebGPU acceleration with a WASM fallback; word timestamps; network confinement. Cost: runtime size; dependence on a third-party runtime's worker and WebGPU behaviour. Reversal: `whisper-runtime.ts` is the only file that knows the runtime (TE-1).

**TDR-4. Rule-based detector with per-kind thresholds.** Chosen over LLM classification for V3-V6. Why: PS §12.4; deterministic, debuggable, measurable precision. Cost: limited recall; lexicon upkeep; cannot judge pauses, implied lists or topic shifts. v1.2: kept as the offline producer of the edit plan (`from_rules`, §40.2); the cloud LLM becomes the default producer (TDR-18).

**TDR-5. Backend-neutral `DisplayList`, Vello on WebGPU as the backend.** Chosen over drawing directly with Vello or with Canvas2D. Why: layout is testable without a GPU (exact snapshots); the PS fallback becomes a second consumer instead of a rewrite. Cost: one intermediate representation; glyph runs carry redundant text for the fallback. Reversal: implement `Canvas2dBackend` (§19).

**TDR-6. WebCodecs plus an in-house Rust MP4 demuxer and muxer.** Chosen over FFmpeg.wasm and over JS container libraries. Why: PS §0 item 4 names a Rust muxer; one crate owns sample tables in both directions; no licensing review. Cost: real-world MP4 variety is a stabilization risk. Reversal: JS demuxer behind `SampleMeta` (TE-12); the muxer stays.

**TDR-7 (v1.2, replaces the v1.1 record). Two timelines and a `TimeMap` from V4, identity until V6.3.** Chosen over v1.1's single timeline. Why: PS v1.2 §9.2.1 makes plan-driven cuts launch scope (V6.3); introducing `OutMs` after the scene, preview, export and audio are written would touch every timestamp in them, while introducing it in V4 costs one type and an identity map whose consumers (scene, preview, export, `assemble`) all exist in V4-V5. Cost: one conversion step everywhere and a type that is the identity for three versions. Reversal: none needed; with the plan off the map is the identity. (v1.1 record: one timeline, no map, because nothing was cut.)

**TDR-18. Cloud LLM edit plan through our server.** Chosen over a local LLM (download size, device variance; PS §9.2 (f)) and over browser-to-provider calls (key exposure, a third browser host). Why: semantic judgments (intentional pause, implied list, topic shift) beyond rules (PS §8 principle 7); only text leaves, media stays local. Cost: a per-plan cost and quota (PS §11), provider dependence (TE-15), latency in the pipeline, nondeterminism. Containment: plan stored per clip; validator (INV-21, INV-22); rules fallback (INV-24). Reversal: rules as default and the AI plan opt-in (E-11 failure), or a local producer of the same `EditPlan` (P2).

**TDR-19. The capability catalog generates the prompt and the schema.** Chosen over a hand-written prompt. Why: the model can only ask for what the renderer draws; adding an effect is one catalog entry plus drawing code; prompt drift is caught by a snapshot and a version bump (INV-23). Cost: the prompt's wording is constrained by what can be generated from data. Reversal: a hand-written preamble file is concatenated before the generated part; the schema stays generated.

**TDR-20. Numbers are pointers, not values.** Chosen over letting the model write the numbers it shows. Why: PS §8 principle 2; a hallucinated or re-rounded number on screen is the worst failure; a `QuantityRef` is checked by parsing the spoken words and follows the user's word corrections. Cost: the model cannot show a computed number (a sum, a percentage it derived). Reversal: none planned.

**TDR-21. Video is a draw command.** Chosen over compositing the overlay over a fixed full-frame video (v1.1 §19). Why: punch-in and jump-cut zooms, video inside shapes, shape transitions and face tracking all become changes to one per-frame command instead of renderer modes. Cost: the compositor splits each frame into Vello runs around the video commands. Reversal: none needed; one full-frame `Video` command is the v1.1 picture.

**TDR-22 (v1.3). A short is a virtual clip behind an `Assembly`.** Chosen over stretching `TimeMap` and over a new timeline through the scene. Why: every proven stage keeps its contract; reordering costs nothing (§44); the founder's rule against needless breakage. Cost: a third clock and a second decode session at joins. Reversal: none needed; `Assembly::whole` is V5.

**TDR-23 (v1.3). Sentence-level selection for the Director.** Chosen over word-level. Why: mid-sentence cuts become impossible by type, and the body stays small (under 64 kB for 20 minutes; v1.4: about 115 kB for 45 minutes, inside DC-15's 160 kB, where a word-level body would pass 400 kB). Cost: depends on sentence segmentation (`sentences_long.rs`); a sentence cannot be split by the model. Reversal: `SentenceRange` becomes a word range with a validator rule; not planned.

**TDR-24 (v1.3). Chat returns typed operations.** Chosen over a whole new plan and over scripts. Why: small requests stay small, undo is the existing history, the validator stays the only gate. Cost: a request with no op fitting it gets "I can't do that yet". Reversal: more ops.

**TDR-26 (design release). GSAP and Motion, on the landing page only.** Chosen over hand-written CSS and requestAnimationFrame code for the landing page's animation (designimplementation D-318, D-319). Why: a page whose job is to sell needs timelines, scroll-scrubbed and pinned sections (GSAP with ScrollTrigger) and React-side entrances (Motion), and both libraries do this well with less code of our own. Cost: 44 kB gzip for GSAP in a chunk loaded with `import()` after the first paint, about 30 kB for Motion's `domAnimation` in the main chunk; GSAP's licence is GreenSock's own "no charge" licence, not an OSI one (read 2026-11-05, designchangelog Entry 4). Kept out of the product: ESLint refuses both outside `src/ui/landing/`, and case L-8 proves the editor's route downloads no GSAP. Reversal: Motion and CSS alone, about a day's work (§17 of the plan).

**TDR-25 (v1.3). Matte as a stored low-resolution video.** Chosen over running the model during render. Why: INV-12, memory, and no new playback code. Cost: a pass per short and H.264 on alpha. Reversal: a lossless frame file (§51.1).

**TDR-8. Preview uses the export renderer and the audio clock.** Chosen over a `<video>` element with overlays. Why: what the user approves is what exports, frame for frame. Cost: a decode loop and clock sync to maintain. Reversal: pre-rendered preview file (§20).

**TDR-9. One stateless Axum service on Render's free tier; Postgres on a separate free provider.** Chosen over SQLite (no persistent disk) and Render's free Postgres (expires). Why: founder constraints; PS §15. Cost: cold starts of about a minute, 0.1 CPU, no backups, two vendors. Reversal: the same image and `DATABASE_URL` run on any other free container host and free Postgres provider; a paid tier is outside PS A-10 and needs a founder decision.

**TDR-10. Same-origin API through the static host's rewrite.** Chosen over cross-origin calls with CORS. Why: the default host names are different sites, so a cross-site refresh cookie would be a third-party cookie; same-origin keeps `SameSite=Strict`, a closed `connect-src` and no CORS surface. Cost: an extra proxy hop and its timeout (TE-5). Reversal: a custom domain with `app.` and `api.` subdomains (same site) and a CORS allowlist of one origin.

**TDR-11. Ed25519-signed entitlement token, receipt after export, client outbox.** Chosen over server-authorized exports. Why: PS §11 (7-day offline cache, limits as conversion mechanisms); an API outage or cold start must not block an export. Cost: a tampered client can exceed limits (accepted by PS). Reversal: require a fresh token per export (one blocker change).

**TDR-12. Static hosting on Vercel.** Chosen because the founder specified it. Why: fits a static Vite build; headers and rewrites are declarative. Risk: the free Hobby plan excludes commercial use (verified 2026-10), which conflicts with selling subscriptions once checkout is live. Founder decision: stay on the free plan and pay nothing (PS A-10); the terms risk is accepted and stays open (§39.3). Reversal: the build is static files plus three settings; move to any static host that permits commercial use on its free tier and supports custom headers and a reverse-proxy rule. A paid Vercel plan is not an option. TE-5 records the terms before week 8.

**TDR-13. Merchant of record with hosted redirect checkout behind `BillingProvider`.** Chosen over a direct card processor and over embedded checkout scripts. Why: PS A-4 (cross-border tax); no third-party script in the app shell (PS §17). Cost: per-transaction fee (PS A-9); a full-page redirect. Reversal: a second adapter (TE-9).

**TDR-14. First-party analytics in Postgres with a typed allowlist generated from Rust.** Chosen over an analytics vendor. Why: PS §17 forbids third-party scripts; one source defines client types, server validation and the settings page. Cost: no dashboards; queries by hand. Reversal: export rows to any tool; the client is unaffected.

**TDR-15. Cross-boundary types defined once in Rust and generated to TypeScript.** Chosen over hand-written mirrors or a schema language. Why: two implementers of neighbouring files get the same shapes; CI fails on drift. Cost: a codegen step. Reversal: commit the generated files and maintain them by hand.

**TDR-16. Model and media on free-egress object storage; manifest bundled with the app.** Chosen over serving models from the app origin. Why: model downloads would consume the static host's transfer allowance; bundled hashes pin integrity to the deployed build. Cost: a second origin in the allowlist; CORS configuration. Reversal: any host that serves Range requests; one constant changes.

**TDR-17. Zero recurring infrastructure cost.** Chosen because the founder required it (PS A-3, PS A-10): nothing is paid for, and the project is deployed on Vercel and Render only, plus free external services. Accepted costs: API cold starts, no database backups, shared 0.1 CPU, free-tier limits that can change without notice, hosting-plan terms (TDR-12), and an email sender without a custom domain (TE-8). Reversal: each component moves independently to another free provider with no code change; an upgrade to a paid tier is possible technically but needs a founder decision. `scripts/check-external-facts.mjs` lists the facts to re-verify before launch.

---

## 39. Consistency Traceability

### 39.1 Requirements

| Product requirement (PS section) | Where implemented and verified |
|---|---|
| Capability detection + unsupported page, P0 (PS §9.3, PS §9.5) | §3, §13.2, C-1, C-16; M0.1, M2.2; `capability.test.ts`, `capability.spec.ts`, `unsupported.spec.ts`; E-8 |
| Supported: desktop Chrome/Edge on Windows/macOS; feature detection, not UA sniffing (PS §9.3) | §3, §13.2; `capability.spec.ts` |
| Capability check under 3 s, logged as allowlisted event (PS §9.3) | `CAPABILITY_CHECK_BUDGET` §10.2; `capability_check` §22.6 |
| Unsupported page: exact missing capability, supported list, demo video, email field, `unsupported_reason` (PS §9.3) | C-16, §13.1; `/notify-me` §22.1; `unsupported.spec.ts` |
| Input probe + actionable rejections, P0 (PS §9.4, PS §9.5) | §15; M1.1; `probe_rejections.rs`, `rejections.spec.ts` |
| Container MP4/MOV, H.264, AAC (PS §9.4) | `validate_probe` rules 1-7, §15.3 |
| Duration at most 90 s (PS §9.4) | `MAX_CLIP_DURATION`; `REJECT_DURATION` |
| File size at most 500 MB (PS §9.4) | `MAX_FILE_SIZE`; `REJECT_FILE_SIZE` |
| Resolution up to 1920 px long side (PS §9.4) | `MAX_LONG_SIDE`; `REJECT_RESOLUTION` |
| Up to 60 fps input, 30 fps output (PS §9.4) | `MAX_INPUT_FPS`, `OUTPUT_FPS`; `REJECT_FRAME_RATE`; verifier check 3 |
| Portrait pass-through; landscape static crop with draggable offset (PS §9.4) | `framing.rs` §19.3; `CropOffsetControl`; `framing.rs` test, `review-edit.spec.ts` |
| One speaker; English (PS §9.4, A-6) | Not detected at import (§15.4); `REJECT_NO_SPEECH`; open item 8 |
| One audio track (PS §9.4) | `REJECT_MULTI_AUDIO_TRACK` |
| Output MP4 H.264/AAC 1080x1920 at 30 fps; Free 720x1280 with watermark (PS §9.4) | §21, `export_profile`; verifier checks 2-4, 9; `export-free.spec.ts`, `export-creator.spec.ts` |
| Rejections specific, never generic; counted as `reject_reason` with no file name or content (PS §9.4) | §15.3, `messages.ts`; `clip_rejected` §22.6; `privacy-network.spec.ts` |
| One-time model download with progress and explanation, P0 (PS §9.5, PS §10 J4) | §16, C-3; M0.2; `model-download.spec.ts` |
| Local transcription with word timestamps, P0 (PS §9.5) | §16.4; M0.2; E-3, E-10, TE-1 |
| Number-word normalization (PS §9.6) | `numbers.rs` §17.2; `numbers_table.rs`, `numbers_prop.rs` |
| Pattern detector: numbers, enumerations, from-to, emphasis, P0 (PS §9.5, PS §12.2) | §17; M1.3; `rules_table.rs`, `precision_recall.rs`; E-7 |
| Precision over recall; at least 90% precision; events below threshold not created (PS §8, PS §20.1) | §17.4, INV-6; `precision_recall.rs` |
| Voice cleanup (denoise, high-pass, compress, limit) + loudness normalization, P0 (PS §9.5) | §18; M1.2; `loudness_target.rs`; verifier check 7 |
| Loudness within ±1 LUFS of the target set here (PS §20.1) | `TARGET_LOUDNESS = -14 LUFS` §18.3 |
| The take is the user's; cuts are plan suggestions the user can see and restore; never inside a word; nothing cut without a plan (PS v1.2 §8 principle 7, §9.2.1, §12.5, §20.1) | Timeline principles (header), §40.1, §40.4 V-7, §40.6, INV-5, INV-10, TDR-7, §36; `duration_preserved.rs`, `program_identity.rs`, `timeline-preserved.spec.ts`, `timemap_build.rs`, `cuts_applied.spec.ts`, verifier checks 5 and 8 |
| AI edit plan: catalog, validation, numbers from the transcript, rules fallback, quotas, server never stores the transcript (PS v1.2 §11, §12.8, §17) | §40; INV-21 to INV-24, P-1, P-14; TE-15, E-11 |
| Music with ducking, cards, furniture, transitions, kinetic presets, video in shapes, brand kit (PS v1.2 §9.5) | §40.2, §40.5, §40.6, §40.10; `buildplan.md` §8A |
| Animated captions, 3 styles, default Clean (PS §9.5, PS §12.2) | §19.4; M1.4; `display_list_snapshots.rs`, `golden_frames.rs` |
| Four visual events: NumberReveal, ListReveal, FromTo, KeywordPop (PS §9.5, PS §12.2) | §17, §19.5; `golden_frames.rs` |
| Preview player with synced audio, scrubbing, low-res mode; drift at most 80 ms over 60 s (PS §9.5, PS §9.6, PS §20.1) | §20; M1.5; `preview-sync.spec.ts` |
| AHA before sign-in (PS §10) | C-5 precedes C-7/C-8; `pipeline-preview.spec.ts` |
| Review UI: edit words, toggle events, pick style, crop offset; no other sliders (PS §9.5, PS §10 J8, PS §12.3) | C-6, §13.3; `review-edit.spec.ts` |
| Caption edits re-run detection for the affected sentence (PS §12.5, PS §20.1) | `redetect_sentence` §17.3; `redetect.rs`, `review-edit.spec.ts` |
| Deterministic rendering from (source, transcript, events, style, renderer version) (PS §12.5) | §19.6, INV-7; `determinism.rs` |
| "What we changed" summary, P0 (PS §9.5, PS §10 J11) | `ChangeSummary` §10.6, C-10; `export-free.spec.ts` |
| Account with magic link, entitlement cache, usage receipt, P0 (PS §9.5, PS §17) | §22.2-§22.4, C-8, C-10; M1.7; `auth_flow.rs`, `usage_receipts.rs`, `signin.spec.ts` |
| Checkout via merchant of record, Creator plan, P0 (PS §9.5, A-4) | §22.5, C-11, C-19; M1.8; `checkout.spec.ts`, `billing_events.rs`; TE-9 |
| Settings page "What leaves your device", P0 (PS §9.5, PS §12.7) | §13.1, §25; `settings-what-leaves.spec.ts` |
| Allowlisted analytics + network-privacy test, P0 (PS §9.5, PS §17) | §22.6, §25.2; M2.2; `privacy-network.spec.ts`, `analytics_allowlist.rs` |
| Persistence: OPFS/IndexedDB, recent clips, render cache, "Delete local data" (PS §9.6, PS §16) | §23; M2.1; `restore.spec.ts`, `clear-local-data.spec.ts` |
| Workers with typed protocol, cancellation, progress (PS §9.6) | §14; `rpc.test.ts`, `cancel.spec.ts` |
| Model manager: versioned, resumable, hash-verified, cached, unload (PS §9.6) | §16.3, §16.5; `download.test.ts` |
| A/V sync handling for VFR sources (PS §9.6) | §15.4, §21.4; `ok_vfr.mp4`; verifier checks 3, 8 |
| Typography: font loading, shaping, line breaking, safe areas, emoji fallback (PS §9.6) | §19.1, §19.4, §19.6; `layout_safe_area.rs` |
| Test harness: golden frames, 40+ clip corpus, perf runs on R1/R2, privacy capture, detector precision (PS §9.6) | §26, §27 |
| J1 Land (PS §10) | C-1, §13.1; `landing.spec.ts` |
| J2 Drop a clip, sample clip, no account needed (PS §10) | C-2, §15.4 |
| J3 Capability check (PS §10) | C-1, C-16 |
| J4 First-run model download, at most 90 s at 25 Mbps (PS §10, PS §20.2, A-8) | C-3, §16; §30 |
| J5 Validate (PS §10) | C-2, §15 |
| J6 Processing with real detections only (PS §10) | C-4, `FeedLine` §14.1; `pipeline-preview.spec.ts` |
| J7 Preview; honest "no events" message (PS §10) | C-5; `NO_EVENTS_FOUND` |
| J8 Review (PS §10) | C-6 |
| J9 Export click: limits before the click, counter on the button, magic-link sign-in (PS §10) | C-7, C-8, §13.4, INV-14 |
| J10 Render with stage progress; total at most 3 minutes on R1 (PS §10, PS §20.2) | C-9, §30 |
| J11 Download + summary; activation event (PS §10) | C-10; `export_done` |
| J12 One dismissible upgrade prompt; "Make another clip" (PS §10, PS §11) | C-11, §13.4 |
| Free: full processing and preview, 3 exports/month, 720p, watermark, all styles and events (PS §8, PS §11) | §22.3, `export_profile`; `entitlement_rules.rs`, `export-free.spec.ts` |
| Creator: 15 USD/month or 144 USD/year; unlimited with soft ceiling 100/month; 1080p; no watermark (PS §11) | `pricing.ts`, `offers.rs`, §22.4 step 7; `export-creator.spec.ts` |
| Pro not sold at MVP (PS §11) | Seam only (§21, §22) |
| Flat pricing, no credits (PS §11) | §36 |
| Limits are conversion mechanisms; signed token cached up to 7 days; idempotent receipt; watermark rendered unless token says Creator (PS §11) | §22.3, §22.4, §21.1, INV-9, INV-13; TDR-11 |
| Upgrade triggers 1-4 (PS §11) | C-7, C-11, §13.4; `export-free.spec.ts` |
| Founding-member presale at 99 USD/year (PS §19 E-5) | `Offer::CreatorAnnualFounding`, `FOUNDING_OFFER_ENABLED` §22.5 |
| Event data model `Word`, `Sentence`, `DetectedEvent` (PS §12.5) | §10.4, §10.5 |
| Cloud/AI fallback: P1, consent rules (PS §12.6) | Seams in §16, §25; not built |
| Settings page content: table, mode, model version, cache size, clear button, verify note (PS §12.7) | §13.1; `settings-what-leaves.spec.ts` |
| Validation metrics: activation, "would you post", 14-day repeat, capability pass, reject rate, active paid (PS §20.5) | §22.6 events, `usage_receipts`, `subscriptions`; `scripts/metrics.sql` §32 |
| P1 promotion triggers measurable (PS §9.5, PS §21) | `export_done` props (`crop_adjusted`), `capability_check`, E-7, E-8 |
| "Report this" sends only the allowlisted diagnostic (PS §14) | C-14, §32 |
| Server stores only email, plan, receipts, webhook ids, sessions, anonymous analytics (PS §15, PS §16) | §23.2, §25.3 |
| Never stored or sent: media, transcripts, captions, edits, file names, paths, user text (PS §15) | P-1, P-11, INV-2 |
| Scale: single small server + CDN; no distributed system (PS §15) | §1, §22, §36 |
| Client storage entities (PS §16) | §23.1 |
| Analytics retention 90 days; no user join without consent (PS §16) | §22.6, §23.3, P-10 |
| Auth: magic link, short-lived tokens (PS §17) | §22.2 |
| Sessions: short-lived access token, rotating refresh token in HttpOnly/Secure/SameSite cookie, hashes only (PS §17) | §22.2, §23.2; `auth_flow.rs` |
| Rate limiting on sign-in, magic-link, receipts, webhooks (PS §17) | §22.1, §24.5; `rate_limit.rs` |
| Strict CSP, no third-party scripts, COOP/COEP, model SHA-256 (PS §17) | §24.3, §24.6, INV-20 |
| Privacy documentation matches the allowlist; reviewed when it changes (PS §17) | P-4; `settings-what-leaves.spec.ts`; `LegalPage` |
| Analytics rules and automated network test (PS §17) | P-3, §25.2 |
| Legal-review gate on compliance or absolute claims (PS §17) | P-12; `messages.test.ts` |
| Delete account within 30 days; clear local data; export account data (PS §17) | §22.7, §23.3; `account_delete.rs`, `account_export.rs`, `clear-local-data.spec.ts` |
| Risk: complexity debt (PS §18) | §2, §6, §7, §34; 700-line limit §29 |
| Risk: stabilization overrun; corpus gate at week 8 (PS §18, PS §20.4) | M2.3, §30 |
| Risk: local performance (PS §18) | §30, E-3, E-4, `stage_timing` |
| Risk: visual false positives; log only enum outcomes (PS §18) | §17, `review_action`, `export_done` |
| Risk: WebGPU/driver variance; GPU vendor enum only (PS §18) | `capability_check.gpu_vendor`, TE-3 |
| Risk: payments onboarding starts week 1 (PS §18) | TE-9 at M0.1 |
| Risk: privacy-claim regression blocks releases (PS §18) | §25.2, `e2e-media.yml` |
| Risk: odd inputs (PS §18) | §15, corpus, TE-12 |
| PS §20.1: fixture clip drop → MP4 on R1 and R2 | M0.4, M2.4; `export-creator.spec.ts` via `pnpm e2e:device` |
| PS §20.1: capability check on Chrome/Edge; unsupported page elsewhere | `capability.spec.ts`, `unsupported.spec.ts`; manual Edge run at M2.5 |
| PS §20.1: every PS §9.4 constraint has a rejection test | §15.3 (speaker and language constraints: open item 8) |
| PS §20.1: resumable, hash-verified model; second session skips | `model-download.spec.ts` |
| PS §20.1: detector precision on at least 30 transcripts | `precision_recall.rs` |
| PS §20.1: export formats; plays on phones; uploads to two platforms | Verifier; manual at M2.5 (TE-4) |
| PS §20.1: Free 3 exports/month enforced by token; counter visible | `entitlement_rules.rs`, `export-free.spec.ts` |
| PS §20.1: checkout in test and live mode; idempotent webhooks | `checkout.spec.ts`, `webhook_idempotency.rs`; live purchase at M2.5 |
| PS §20.1: privacy/network verification | `privacy-network.spec.ts` |
| PS §20.1: corpus at least 90%, none crash | `corpus/run_corpus.ts`, M2.3 |
| PS §20.1: error taxonomy and stage timings via allowlist; uptime monitor | §11.2, §22.6, §22.9, §32 |
| PS §20.2 performance budgets and reference hardware (A-7) | §30; `bench/device-bench.ts` |
| PS §20.3 sequential worst-case arithmetic | §30 note |
| PS §20.4 pre-agreed fallbacks | §30 decision rule; §16, §18, §19, §21 contingencies |
| PS §20.6 launch items: legal pages, merchant live, support/FAQ/status, demo videos, dashboard | `LegalPage` §13.1; TE-9; uptime monitor status page; asset CDN; `scripts/metrics.sql` |
| PS §21 roadmap items 1-13 | Seams in §13-§25; none built (§36) |
| H1 value | E-2, E-7, `post_export_answer` |
| H2 willingness to pay | E-5, §22.5, `upgrade_prompt`, `checkout_step` |
| H3 local viability | E-3, E-4, §30, `stage_timing` |
| H4 privacy as pull | E-6, §25 |
| Milestones M0-M4 with PS dates (PS §9.9, A-2) | §28 |
| Non-goals (PS §9.2) | §36; seams only |
| A-3/A-10 cost | Founder constraint (header), TDR-17 |
| A-11 working name | One constant in `messages.ts` |


### 39.2 Reconciliations with the product spec

The product spec was aligned to this document on the points below; none is an open difference any more.

- **Infrastructure cost.** PS A-3 and A-10 now state 0 USD/month with no paid fallback, matching the founder constraint (TDR-17).
- **Storage capability.** PS §9.3 now requires both OPFS and IndexedDB (large files and metadata).
- **"About three minutes" versus clips up to 90 s.** The PS promise and the PS §20.2 budgets are for the 60-second reference clip. A 90-second clip is expected to scale roughly linearly (about 1.5x) and has no budget; `stage_timing` reports it.
- **Analytics and accounts (PS §16, PS §17).** No analytics row is tied to an account, so account deletion has nothing to remove there.
- **Metrics (PS §20.6).** `scripts/metrics.sql`, not a deployed dashboard.
- **Fair use (PS §11).** Never blocked, logged above 100 exports per month.

### 39.3 Known open items

These are open, not contradictions. Each (assumption) is listed with what will measure it; each experiment outcome is pending until its milestone.

1. Hosting plan for a paid product on the static host (TE-5, TDR-12): the founder stays on Vercel's free plan and pays nothing; the plan's non-commercial terms remain an accepted risk, with another free static host as the only remedy.
2. All free-tier facts for Render, Postgres, asset storage, email, CI minutes, uptime monitor (TE-5 to TE-11); re-listed by `scripts/check-external-facts.mjs` before launch.
3. Email sender verification without a registered domain (TE-8): a domain costs money and is outside PS A-10, so the provider must accept a single verified sender address; deliverability of magic links under that setup is unproven. V6: Brevo accepts one Gmail sender; two Gmail mailboxes received every mail; other providers are unread (`docs/v6/experiments.md`). Read with real sign-ups: `signin_step` `link_requested` without `completed`.
4. Mobile detection via `navigator.userAgentData.mobile` (§3) — E-8 data at M2.2.
5. `MAX_FILE_SIZE` read as 500,000,000 bytes; `INPUT_FPS_TOLERANCE`; `MIN_WORDS`; `MAX_RECENT_CLIPS` (§10.2) — M1.1, E-2, M2.1.
6. First video track used when several exist; fragmented MP4 rejected (§15.4) — TE-12.
7. Per-word ASR confidence (§16.4) — TE-2. Run on 2026-10-09 (`docs/v2/experiments.md`): the runtime of V2 returns no probability, so every word has `Confidence(1.0)` and detector scores do not use ASR confidence. Hallucinated words over silence are not filtered (§16.6); long mid-clip pauses are now always kept, so E-10 also counts hallucinated words and timestamp drift inside and after pauses of 3 s or more — E-10.
8. Non-English speech and multiple speakers are not detected at import; these two PS §9.4 constraints have copy but no rejection test (§15.4) — E-2, E-10.
9. Sentence segmentation constants (§17.2) — E-10.
10. Every detector score, threshold, window and lexicon (§17.4, §17.5); empty header for marker-only lists — M1.3, E-7. Measured in V3 (docs/v3/experiments.md): the precision gate over 31 labeled files raised the NumberReveal threshold to 0.85; the other three thresholds held.
11. Loudness target -14 LUFS and every DSP constant, including `ALIGN_TOLERANCE` (§18.3) — M1.2, TE-13, E-2. Measured in V3 (docs/v3/experiments.md): the denoiser's delay is 480 samples, constant, and compensated exactly; TE-13 passes on D1 (not R1) with denoise on.
12. Safe-area insets, zones, style values, event timings, emoji fallback font (§19) — E-2, M1.4.
13. Watermark text, size, opacity, placement (§19.6; PS §19 minor) — alpha.
14. Golden similarity 0.98; verifier thresholds for watermark and sync (§19, §27.2) — M1.4, M1.6. Measured in V4 (docs/v4/experiments.md, D1 and CI): the 16 goldens read 1.0000 on D1's WARP and 0.9994-1.0000 on the runner's lavapipe, the parity cases at least 0.9994: one folder serves both, at 0.98 (and 0.99 for parity). Measured in V5 (docs/v5/experiments.md, D1 only): watermark correlation 0.74 on a Free file and -0.05 to -0.01 on Creator files, against 0.5 and 0.2, which stand; audio offset at most 1.7 ms against 20 ms, at a correlation of at least 0.95 against 0.5; white frames within one frame, a flash being at least 225 of mean brightness and 30 above the frames around it; -14.3 LUFS on every export against -14 ± 1; video 8.3-8.4 Mb/s Creator and 4.1 Mb/s Free against 12 and 6.
15. Preview size, queue length, late-frame and first-frame targets, seek target (§20) — M1.5. Measured in V4 on D1 (not R1, D-114): first frame 76-94 ms after `ready` (500 ms), seek 78-94 ms (300 ms), drift at most 26.9 ms over 60 s with 0% late frames (80 ms, 5%), never downscaled; the queue of 6 and the 360 x 640 downscale size are kept.
16. Video and audio bitrates, keyframe interval, 2-channel audio, encoder ladder order (§21.2) — TE-4.
17. `CANCEL_TIMEOUT`; `API_TIMEOUT`; `API_COLD_START_BUDGET` (§14.3, §22.9) — M1.5, TE-11. Measured in V4 on D1: a hung render worker's run is back at `accepted` 2.4-2.5 s after Cancel; `CANCEL_TIMEOUT_MS = 2000` kept.
18. Magic-link, access-token and refresh-token lifetimes; all rate-limit values in §22.1 — M1.7. Built in V6 as written: a link 15 minutes, single-use; an access token 15 minutes; a session 60 days from sign-in, however active (founder's confirmation), with a 30 s grace for a rotated refresh token (D-258).
19. Free quota month = UTC calendar month (§22.3) — product confirmation at M1.7.
20. Creator until `current_period_end` for canceled and past-due; refund ends access immediately (§22.3, §22.5) — TE-9. Read in V6 test mode: a cancel keeps the paid date; a declined renewal is `on_hold` and does not move it; a refund does not stop the subscription at Dodo, so the webhook cancels it there.
21. Analytics never joined to accounts at MVP (§22.6) — M2.2.
22. Retention of `webhook_events` (90 days) and `magic_links` (1 day); no database backups; 30 events per visitor (§23.3) — TE-9, TE-6. Dodo's documented retries span about 27 hours, inside 90 days.
23. Memory estimates and the 1.5 GB peak budget (§31) — TE-14.
24. App-shell size 500 kB (400 kB until the design release, designimplementation D-320); long-task limit 100 ms; `frame_at` 2 ms; `redetect` 50 ms; edit-to-preview 150 ms; API p95 150 ms (§13.5, §17, §19, §30) — M1.5, M2.4. Measured in V3: `redetect_sentence` 142-290 µs per call on a 157-word transcript, natively on D1 (§17.3). Measured in V4 on D1: app shell 337.2 kB gzip with the interface font (400); no main-thread task over 100 ms while processing (after Prompt 124's two fixes), none over 50 ms in a word edit, a style switch or a caption drag (D-193); `build_scene` 6.3 ms and `frame_at` 0.002 ms on average natively (1 s, 2 ms); edit-to-preview 90th percentile 25-29 ms (150); hit test median 0.5 ms, worst 7.6 ms (16); style thumbnails 18-81 ms (400). Measured in V5 on D1: app shell 339.6 kB; render and encode 30.9 s per 60 s (90), mux 21 ms (2 s); a render-cache hit 12-43 ms (2 s), its key 0.4-1.6 ms on the main thread. Measured in the design release on D1: app shell 444.3 kB gzip (500), of which the main chunk 205.8 kB and the landing page's GSAP chunk 44.1 kB, which the editor's route never downloads; no main-thread task over 50 ms while scrolling the landing page (designchangelog Entry 8). R1 and M2.4 re-read them.
25. 700-line file limit (§29; 400 until 2026-10-28) — reviewed at each milestone.
26. Word edits replace or hide text only; no insertion or retiming (§10.6) — E-2.
27. PS experiments E-1 to E-10 and technical experiments TE-1 to TE-14: outcomes pending per §28. Read in V3 on D1 (docs/v3/experiments.md): TE-13 (pass); E-3 (transcription median 69.5 s per 60 s, the 20 s target missed, under the 145 s fallback line); E-4 (render and encode 30.8 s per 60 s, pass); TE-12 inconclusive (2 clips, re-checked in V9); E-10 carried to V9.
28. Left open by V3, each with the version that handles it (v3implementation §23): an `Edit` that ignores `media_rate` (a dwell edit taken as a media edit) — TE-12, V9; Indian digit grouping not parsed — E-7; no `Unit` for terabytes — a shared-type revision; the `events` store without a writer (§23.1) — V7; KeywordPop precision measured on synthetic emphasis — E-7 from V10; the derived portrait clip is an enlarged 405-pixel strip, cheaper to decode than a real phone clip — a real portrait recording; the capability check admits a software adapter, on which a clip takes over 20 minutes — V8; the cleaned audio of a clip that ends early stays in `audio.worker` until the next clip (C-4) — V4; the 40-word split's tie-break makes one-word sentences (§17.2) — M1.3.
29. (v1.2) Every value marked (assumption) in §40: `CUT_SNAP`, `MIN_KEEP_GAP`, lead/trail keeps, `MAX_CUT_SHARE`, `CUT_XFADE`, `JUMP_ZOOM`, `END_CARD_DUR`, `MUSIC_BED`, `DUCK_DEPTH` and its attack/release, `PLAN_UPSTREAM_TIMEOUT`, `PLAN_CLIENT_TIMEOUT`, the plan quotas — V6.3, V6.7, TE-15, E-2.
30. (v1.2) The AI provider, its free-tier limits and its data-use terms — TE-15. If its free tier trains on inputs, the founder decides (PS §17 edit-plan row).
31. (v1.2) Music licensing for bundled tracks — TE-16.
32. (v1.3) The launch date: V6.1-V6.11 sit before V7; the founder re-sets M2 when V6.11 closes (PS A-2).
33. (v1.2) Colour emoji: the embedded emoji font is monochrome (§19.6); whether V6.2's emoji items need a colour font (and whether Vello draws its colour tables) is decided in V6.2.
34. (V6.1, measured) The long path's assumptions as read: `CLOUD_LONG_CONCURRENCY = 2` meets Groq's free tier's per-minute limit on every long source (open item 24; Groq's Dev tier, or one in flight, before `long_video` goes on); `CLOUD_OVERLAP_S` 10, not 5 (TE-18's first read); the read cache of 8 blocks of 1 MiB and in-place reading hold 34-37 MB for 45 minutes (TE-17); `LONG_SCRATCH = 2 GB` is kept, not measured (`pcm16.f32` is 173 MB for 45 minutes); the window group of 120 a day and the global budget's 7,000 s an hour and 27,000 s a day are unchanged; `MAX_SOURCE_DURATION` is exact, so a file a fraction of a second over 45:00 is refused (the founder decides on a tolerance).
35. (V6.2, measured) The plan's assumptions as read: `deepseek-flash` at 0.30 / 1.20 USD per 1M tokens, about $0.0035 a plan with a 7,433-token prompt; latency 4 to 5 s against the 10 s target; the median drop rate over TE-15's 10% line on several reads (numbers that do not resolve, an icon on a word inside a list), accepted by the founder for V6.2 and to be read again on real clips in V6.3; Free 40 plans a month and Creator 1,000 / 3,000 kept as assumptions; DeepSeek's terms say nothing of keeping or training, so the privacy page and the settings table name DeepSeek, in China, and claim neither (V6 CL open item 35) before `ai_plan` goes on.

---

## 40. Edit Plan Architecture (v1.2)

**Model: one edit plan per clip is the only description of the edit. Two producers write it: the rule detector (V4, offline, always available) and a cloud LLM reached through our server (V6.2, the default). One validator in Rust checks every plan against the transcript before anything reads it. The scene, the program audio and the time map are pure functions of (transcript, valid plan, edit state, clip, profile, theme, assets). The LLM sees transcript text, word timings and prosody scores; it never sees audio or video, and it never writes a number that is shown on screen. Rejected: letting the LLM emit draw commands or timestamps (unverifiable, breaks determinism), a second rule engine for cuts (cannot tell an intentional pause, PS §9.2), calling the provider from the browser (exposes the key, a third host in the allowlist), a local LLM at launch (PS §9.2 (f); P2 seam: a third producer of the same `EditPlan`).**

Precedence: this section implements PS v1.2 §8 principles 2, 3, 4 and 7, §9.2.1 and §12.8. Where an older section of this document says "one timeline", "no time map", "nothing leaves", "no cuts" or "four event kinds only", this section wins and the older section is read as describing the identity case (no cuts, plan off).

### 40.1 Two timelines

```rust
// offcut-types/src/units.rs  (addition, V4)
pub struct OutMs(u32);        // a position on the program timeline (the export's time)
impl FrameIdx { pub fn to_out_ms(self) -> OutMs; }   // replaces to_time_ms(): frames index the program, not the recording
// TimeMs keeps its meaning: a position in the recording (source) timeline. No From between the two.

// offcut-types/src/timemap.rs  [ONLY]  (V4)
pub enum Segment { Source { src: Span }, Hold { at: TimeMs, dur: DurMs } }    // Hold: freeze a source frame (end card)
pub enum SrcPos  { At(TimeMs), Held(TimeMs) }
pub struct TimeMap { segments: Vec<Segment> }        // private field; built only by offcut-plan::timemap
impl TimeMap {
    pub fn identity(duration: DurMs) -> TimeMap;
    pub fn is_identity(&self) -> bool;
    pub fn out_duration(&self) -> DurMs;
    pub fn to_out(&self, t: TimeMs) -> Option<OutMs>;   // None when t lies inside a removed span
    pub fn to_src(&self, t: OutMs) -> SrcPos;
    pub fn joins(&self) -> Vec<OutMs>;                  // program time of every splice, ascending
}
```

- Transcript, prosody, detector events and plans keep `TimeMs` (what was said, when). `build_scene` maps every time it uses through `to_out` exactly once. The scene, the preview clock, frame indexes, the export loop and the program audio run on `OutMs`. `VideoSource` reads source frames at `to_src(t)`.
- `timemap::build(cuts, transcript, silence, end_card) -> TimeMap` in `offcut-plan` is the only constructor besides `identity`. Splice points are snapped to the quietest 10 ms window of `SilenceProfile` within `CUT_SNAP = 40 ms` of the planned boundary and never move into a word (assumption, V6.3).
- V4-V6: every map is `identity(clip.duration)`; `is_identity()` is true; program time equals recording time; INV-5's v1.1 behaviour is the identity case.

### 40.2 The plan

```rust
// offcut-types/src/plan.rs  (V4: the rule kinds and empty cuts; later releases add variants)
pub struct EditPlan {
    pub format: u16,                    // PLAN_FORMAT = 1
    pub catalog: u32,                   // CATALOG_VERSION the plan was produced against
    pub source: PlanSource,
    pub cuts: Vec<Cut>,                 // empty until V6.3
    pub items: Vec<PlanItem>,
    pub chapters: Vec<Chapter>,         // empty until V6.2
    pub extras: TextExtras,             // empty until V6.2
    pub music: Option<MusicCue>,        // None until V6.7
}
pub enum PlanSource { Rules, Model { provider: ProviderId, model: String } }   // set by the producer, never by the LLM
pub enum Cut {
    Lead  { keep: DurMs },                          // silence before word 0 shortened to `keep`
    Trail { keep: DurMs },                          // silence after the last word shortened to `keep`
    Gap   { after: WordIdx, keep: DurMs },          // pause between `after` and `after + 1` shortened to `keep`
    KeepGap { after: WordIdx },                     // an intentional pause: no Gap cut may touch it
    Words { range: WordRange, why: CutWhy },        // CutWhy: Filler | FalseStart; applied only when EditState.filler_cuts
}
pub struct PlanItem { pub id: EventId, pub kind: ItemKind, pub anchors: WordRange, pub params: ItemParams,
                      pub confidence: Option<Confidence> }   // Some for rule items (INV-6), None for model items
pub struct QuantityRef { pub words: WordRange }      // a number is a pointer to spoken words, resolved by validate (§40.4, V4 D-121)
pub struct Label(String);                            // checked by validate (§40.4 V-4)
pub struct Chapter { pub starts_at: WordIdx, pub title: Label }
pub struct TextExtras { pub hook: Option<Label>, pub post_caption: Option<String>, pub hashtags: Vec<Label> }
pub struct MusicCue { pub fits: bool, pub energy: MusicEnergy }   // MusicEnergy: Calm | Steady | Upbeat
```

`ItemKind` keeps codes 0-3 for the v1.1 kinds so stored `EventId`s and `EditState.event_overrides` survive (§17.1 hash layout unchanged; new kinds take codes 4 and up in catalog order). The launch set:

| Release | Kinds (params in brief) |
|---|---|
| V4 | `NumberReveal { value: QuantityRef, label }`, `ListReveal { header: Label, items: [{ at: WordIdx, label: Label }] }`, `FromTo { from: QuantityRef, to: QuantityRef, label }`, `KeywordPop { word }` |
| V6.2 (v1.3: launch list in §47) | `Emphasis { word, treatment: Colour | Underline | Highlight, size: Normal | Punch }` (v1.5: `size`, §47.3), `IconPop { word, icon: IconId }`, `PunchIn { scale: Mild | Strong, motion: Snap | Push }` (v1.4: the auto zoom of §47.2), `Emoji { word, emoji: EmojiId, place: Beside | Under }` (v1.5: `place`), `HookTitle { accent: WordIdx }` (v1.5: moved from V6.6; its text is `extras.hook`, §47.3) |
| V6.6 | `ListBuilder`, `NumberMoment { value: QuantityRef, form: Count | Bar }`, `QuoteCard { quote: WordRange }`, `DefinitionCard { term: Label, gloss: Label }`, `Versus { left: Label, right: Label, values: Option<(QuantityRef, QuantityRef)> }`, `StepTracker { steps: [{ at, label }] }`, `Timeline { points: [{ at, label }] }`, `Checklist { items: [{ at, label }] }` |
| V6.6 | `ShapeFrame { shape: ShapeId }` (video inside a shape), `Transition { at: WordIdx, kind: TransitionId }` |
| V6.6 | `ProgressBar`, `EndCard { cta: Label }` (v1.5: `HookTitle` moved to V6.2); `LowerThird` and `ChapterMarker` moved post-launch (§42) (adds a `Hold` segment of `END_CARD_DUR = 2.5 s`, assumption) |

The rules producer (`offcut-plan::from_rules(&[DetectedEvent], &Transcript) -> EditPlan`) emits the four V4 kinds, converting each `Quantity` of a `DetectedEvent` into a `QuantityRef` to the `NormalizedSpan` it came from (the n-th span inside the event's anchors). `DetectedEvent`, `offcut-detect` and its tests are unchanged. From V6.6 the rules producer may map its kinds onto `ListBuilder` and `NumberMoment`; codes 0-3 stay reserved.

**(V6.2, as built: D-446, D-447, D-453; the V6.2 plan §6.1.)** `ItemKind` is `EventKind` in the code. It gains `Emphasis`, `IconPop`, `PunchIn`, `Emoji` and `HookTitle` at the end (codes 4 to 8). `ItemParams` gains their five wire arms, and `EventParams` (`offcut-types/src/params.rs`) their five resolved arms, which the detector never makes; `ValidItem.params` stays `EventParams`. **`HookTitle { accent: u8 }`, not `WordIdx`:** the accent is the index of a word of the hook (`extras.hook`), counted from 0, and its anchors are `ClipWide`, exactly `0..word count`. `IconId` (24 names) and `EmojiId` (48 names) are closed enums in `offcut-types/src/assets.rs`, equal to the image bundle's manifest (a test). `DropReason` gains `Density` and `HookMissing` at the end. `CATALOG_VERSION = 2`, with two new zones, `video` and `hook`, and `ParamTy::Index { max }` for the hook's accent (D-448).

### 40.3 The capability catalog (the renderer's menu)

```rust
// crates/offcut-plan/src/catalog.rs  [ONLY]
pub const CATALOG_VERSION: u32;
pub struct ItemSpec { pub kind: ItemKind, pub name: &'static str, pub purpose: &'static str,
                      pub anchor: AnchorRule /* Words { min, max } | Point | ClipWide */,
                      pub params: &'static [ParamSpec], pub max_per_clip: u8, pub min_gap: DurMs, pub zone: ZoneId }
pub struct ParamSpec { pub name: &'static str, pub ty: ParamTy /* Label { max_chars } | QuantityRef | WordIdx | Enum(&[&str]) | Bool */,
                       pub required: bool }
pub fn catalog() -> &'static [ItemSpec];
// crates/offcut-plan/src/prompt.rs
pub fn system_prompt() -> String;                    // generated from catalog(); rules for cuts, numbers, labels
pub fn output_schema() -> serde_json::Value;         // JSON Schema of EditPlan restricted to the catalog
pub fn user_message(req: &PlanRequest) -> String;    // one line per word: idx, text, start, end, energy_z, pitch_z;
                                                     // a PAUSE <ms> line at every gap >= 300 ms; sentence breaks
```

- A kind is in the catalog only in the release that draws it: `catalog_complete.rs` asserts every catalog kind has a layout, a draw function and at least one golden frame, and every drawable kind is in the catalog.
- `catalog_snapshot.rs` snapshots `system_prompt()` and `output_schema()`; a change without a `CATALOG_VERSION` bump fails.
- `IconId`, `EmojiId`, `ShapeId`, `TransitionId` are closed sets defined next to their drawing code; the catalog lists their names, so the model can only pick what renders (no tofu, no missing icon).
- The prompt is built on the **server** from the client's structured `PlanRequest`, so prompt changes deploy with the API.

### 40.4 Validation (`crates/offcut-plan/src/validate.rs` [ONLY])

```rust
pub fn validate(plan: EditPlan, t: &Transcript, edits: &BTreeMap<WordIdx, String>, cfg: &PlanConfig)
    -> Result<(ValidPlan, Vec<Dropped>), PlanRejected>;    // PlanRejected only for V-1; everything else drops items
pub struct Dropped { pub kind: ItemKind, pub reason: DropReason }   // DropReason is an analytics enum, no content
```

| # | Rule | On failure |
|---|---|---|
| V-1 | `format == PLAN_FORMAT` and `catalog == CATALOG_VERSION` for a new response; `catalog <= CATALOG_VERSION` for a stored plan (v1.3, DC-7) | Whole plan rejected; rules fallback |
| V-2 | Kind is in this build's catalog; params match its `ParamSpec`s | Item dropped |
| V-3 | Anchors are inside the transcript, non-empty, and satisfy the kind's `AnchorRule`; `at` points lie inside the anchors | Item dropped |
| V-4 | Every `Label` is within `max_chars`; every digit run in a `Label` or in `post_caption` equals a digit run of the effective transcript words; hashtags match `#[A-Za-z][A-Za-z0-9]{0,29}`, at most 8 | Item (or that extra) dropped |
| V-5 | Every `QuantityRef` parses (`parse_quantity` over the effective words in range) to a `Quantity` | Item dropped |
| V-6 | Per-kind `max_per_clip` and `min_gap`; at most one item per zone at a time (earlier anchor wins, then catalog order) | Later items dropped |
| V-7 | Cuts: `Gap.keep >= MIN_KEEP_GAP (150 ms)` and shorter than the real gap; no `Gap` at a `KeepGap`; `Lead.keep >= 250 ms`, `Trail.keep >= 400 ms`; a `Words` cut overlaps no kept item's anchors; total removed at most `MAX_CUT_SHARE = 40%` of the clip (assumptions, V6.3, E-2) | Cut dropped; over `MAX_CUT_SHARE` all cuts dropped |
| V-8 | Chapters strictly increasing, at most 6, each at least 8 s of program time | Extra chapters dropped |
| V-9 | (v1.3, V6.6) For `PlanSource::Model` only, at most one visual item in any 2.5 s window, excluding caption emphasis (§47). A rules plan is exempt, so `validate_rules_plans.rs` and every golden stay as they are | Later item dropped |

- A rules plan passes the same validator. A word edit re-runs `validate` on the stored plan (not a new request): a corrected number shows the corrected value (V-5 reads effective words), an item whose number no longer parses is dropped, and `redetect_sentence` still serves the rules plan.
- `ValidPlan` is the only plan type `build_scene`, `timemap::build` and `assemble` accept. It has no public constructor (INV-21, `tests/ui.rs`).

As built in V4 (D-121, D-165, the V4 plan's §8.7):

```rust
pub struct ValidPlan { /* private */ }                    // source(), items() in draw order (start, kind, first word), is_empty()
pub struct ValidItem { pub id: EventId, pub kind: EventKind, pub anchors: WordRange, pub span: Span,
                       pub params: EventParams, pub marks: Vec<TimeMs>, pub confidence: Option<Confidence> }
pub struct PlanConfig { pub per_kind: BTreeMap<EventKind, KindLimits>, pub min_keep_gap: DurMs,
                        pub max_cut_share_permille: u32, pub max_chapters: u32 }       // PlanConfig::default() reads the catalog
pub struct KindLimits { pub max_per_clip: u8, pub min_gap: DurMs, pub label_chars: u32, pub header_chars: u32 }
```

- **Numbers are resolved here, not in the scene.** `validate` turns every `QuantityRef` into a `Quantity` with `parse_quantity` over the effective words of its range (V-5) and hands the scene `EventParams` with resolved values; the scene parses nothing (INV-24).
- **`marks`** are the recording times a visual steps at, read from the words: a FromTo's second value, which a `DetectedEvent` has no time for.
- **V4's limits** (`catalog.rs`, (assumption, V6.2, E-11)): NumberReveal 8 per clip, min gap 1,500 ms; ListReveal 3, 10,000 ms; FromTo 4, 3,000 ms; KeywordPop 12, 800 ms; labels 32 characters, list headers 24. Every rules plan of the 31 labeled files validates with nothing dropped (`validate_rules_plans.rs`).
- In V4, V-7 and V-8 drop every cut, chapter and text extra: no producer makes them yet.

**(V6.2, as built: D-449 to D-455, D-503, D-508.)**
- **V-1 (DC-7 as one rule):** `catalog <= CATALOG_VERSION` for every plan, so a stored catalog-1 plan stays valid; only the server requires a fresh model answer to equal `CATALOG_VERSION`.
- **V-3, zooms:** two `PunchIn`s never overlap and at most one starts in a sentence (`TooClose`); a `Snap` spans 1 to 3 words and a `Push` 2 to 40, inside one sentence.
- **V-9, density, model plans only:** at most one visual item starts in any 2.5 s window. The visual kinds are `NumberReveal`, `ListReveal`, `FromTo`, `IconPop`, `Emoji` and `HookTitle` (read first); `Emphasis`, `KeywordPop` and `PunchIn` do not count. A dropped item is `Density`; rules plans are exempt.
- **Punch words:** at most one per 6 s, counted from the previous punch word's end, and 4 a clip.
- **The hook:** a `HookTitle` without `extras.hook` is `HookMissing`; its accent must be a word of the hook. A model plan whose hook passed and that kept no `HookTitle` gains one: the accent is the hook's longest word, the id the largest plus one, added before V-9 (D-503).
- **Text extras, validated and kept** in `ValidPlan.extras`:
  - the hook, a `Label` of at most 48 characters and 2 to 9 words, with V-4's digit rule;
  - `post_caption`, at most 2,200 characters, with no digit rule (the person edits it before posting). The prompt asks for one or two sentences of at most 150 (D-508);
  - `hashtags`, at most 8, each `#` then 1 to 30 letters, digits or `_`.
  
  A bad extra is dropped alone (`LabelTooLong`, `LabelDigits`, `ParamsMismatch`, `OverLimit`).
- **Chapters, cuts and music** stay `NotInThisBuild`.

### 40.5 Scene and renderer contracts

```rust
// offcut-scene (V4 contract)
pub struct SceneInput<'a> { pub transcript: &'a Transcript, pub plan: &'a ValidPlan, pub edit: &'a EditState,
                            pub clip: &'a ClipInfo, pub profile: &'a ExportProfile, pub time_map: &'a TimeMap,
                            pub theme: &'a Theme, pub fonts: &'a FontSet, pub assets: &'a SceneAssets }
impl Scene {
    pub fn duration(&self) -> DurMs;                       // == time_map.out_duration()
    pub fn frame_count(&self) -> u32;                      // ceil(duration_ms * OUTPUT_FPS / 1000)
    pub fn frame_at(&self, t: OutMs) -> DisplayList;       // pure
    pub fn source_at(&self, t: OutMs) -> SrcPos;           // which source frame the Main video slot shows
    pub fn summary(&self) -> ChangeSummary;
}                                                          // crop() is gone: the crop lives in each Video command
pub enum DrawCmd {
    // v1.1 commands unchanged, plus:
    Video { slot: VideoSlot, src: CropRect, dst: Affine, mask: Option<Vec<PathEl>>, opacity: f32 },
    Image { image: ImageKey, rect: Rect, opacity: f32 },
}
pub enum VideoSlot { Main }                                // v1.3: V6.8 adds Asset, V6.11 adds Matte (§48.1)
pub struct FontKey(u16);                                   // replaces the closed FontId enum (v2implementation D-29)
pub struct FontSet { /* embedded fonts + user fonts */ }   // FontSet::embedded(); FontSet::with_user(fonts) in V6.10
pub struct Theme { pub text: Rgba, pub stroke: Rgba, pub accent: Rgba, pub accent_alt: Rgba, pub panel: Rgba,
                   pub backdrop: Rgba, pub display_font: FontKey, pub body_font: FontKey }
// offcut-render (V4 contract)
impl Renderer {
    pub fn render(&mut self, sources: &VideoSources, list: &DisplayList) -> Result<(), RenderError>;
}   // VideoSources: slot → (&VideoFrame, Rotation). Commands draw in list order: Vello runs between Video commands.
```

- **Video is a scene node.** Each frame's list contains the `Video { Main, … }` command; framing, crop offset, punch-in zoom, jump-cut zoom, face tracking (P1) and shapes all change that command, not the renderer.
- **V4 implements:** one `Video` command per frame with any scale/translate `dst`, opacity 1.0, no mask; `Image` is in the type but returns `RenderError::Unsupported` until its first user (V6.2 icons if raster, else V6.10 logo); `mask` returns `Unsupported` until V6.6. Display lists and goldens use the new shape from V4, so V6.x adds drawing code, not contract changes.
- **Styles are data on theme roles.** `StyleSpec` names theme roles, never `Rgba` literals (`styles_use_theme.rs`); `StyleSpec.motion: CaptionMotion { unit: Word | Glyph, enter: MotionPreset, stagger: DurMs, active: ActiveTreatment }` expresses Clean, Bold and Tech in V4, and V6.6's kinetic presets are more rows of data plus any new `MotionPreset` arm.
- **Zones.** `safe_area.rs` holds a named zone table (`Top`, `Event`, `Caption`, `LowerThird`, `Full`) in place of the two constants; items claim zones through the catalog (V-6).

**As built in V4** (§19.2 holds the full signatures):
- `SceneInput` has no `assets` field yet: no V4 kind draws an image, so `SceneAssets` comes with `Image`'s first user (V6.2 icons if raster, else V6.10).
- `Video.dst` is an axis-aligned `Rect` in output pixels, not an `Affine` (D-130): every V4-V6.6 framing is a scale and a translate, and the list's scaling leaves `dst` alone.
- `Theme` also has `on_accent` (the word on Clean's pill), `shadow`, and **`mono_font`** (D-136: the font of code-style captions, Tech/"Mono"'s JetBrains Mono): `Theme { text, stroke, accent, accent_alt, panel, backdrop, on_accent, shadow, display_font, body_font, mono_font }`, made by `Theme::default_for(StyleId)`.
- `Renderer::render(&mut self, sources: &VideoSources<'_>, list: &DisplayList, fonts: &FontSet)` takes the `FontSet` the glyph runs are set in; `VideoSources` is `{ main: (&VideoFrame, Rotation) }`.
- The renderer's V4 support rule (D-133): exactly one `Video`, the first command, of the `Main` slot, opaque, with no mask, `src` inside the frame and `dst` inside the canvas, and no `Image`; any other list is `RenderError::Unsupported`.

### 40.6 Program audio (`crates/offcut-dsp/src/program.rs` [ONLY place that changes duration])

```rust
pub struct SilenceProfile { pub rms_db_10ms: Vec<f32> }   // offcut-types; computed by offcut-dsp::silence_profile(out48)
pub struct MusicInput<'a> { pub pcm48: &'a [f32], pub loudness: Lufs }
pub fn assemble(out48: &[f32], map: &TimeMap, speech: &[Span], music: Option<MusicInput<'_>>, cfg: &ProgramConfig)
    -> ProgramAudio;                                      // speech: word spans in program time
pub struct ProgramAudio { pub samples: Vec<f32>, pub loudness: Lufs }   // samples.len() == out_duration at 48 kHz
```

- **Splices:** an equal-power cross-fade of `CUT_XFADE = 20 ms` centred on every join; a `Hold` segment is silent speech (music continues).
- **Music:** looped with a 2 s cross-fade or trimmed; bed at `MUSIC_BED = -20 dB` relative to speech loudness; ducked by `DUCK_DEPTH = -12 dB` while speech is active, attack 80 ms, release 400 ms; faded out over the last 1.5 s or the end card; then the v1.1 loudness stage and limiter run over the mix (all assumption, V6.7, E-2).
- **Identity case (V4-V6):** `assemble(out48, identity, speech, None, cfg)` returns `out48` sample for sample (`program_identity.rs`). INV-10 now reads: the enhancement chain preserves duration; `assemble` alone produces the program duration.
- Runs in `audio.worker` behind `AudioWorkerApi.assemble` (added in V4). The preview plays and the export encodes `prog48`; OPFS `clips/<clipId>/prog48.f32` beside `out48.f32`.

### 40.7 Server: `POST /api/v1/plan` (V6.2)

| Item | Value |
|---|---|
| Auth | Bearer optional. Anonymous requests are allowed so the AHA stays before sign-in (PS §10) |
| Body | `PlanRequest { catalog: u32, duration: DurMs, words: Vec<PlanWord>, sentences: Vec<WordRange> }`, `PlanWord { text, start: TimeMs, end: TimeMs, energy_z: f32, pitch_z: f32 }`; limit 64 kB (INV-4 holds); `deny_unknown_fields` |
| Response | `PlanResponse { plan: EditPlan, plans_remaining: Option<u32> }`, at most 32 kB |
| Errors | `E_PLAN_QUOTA` (429), `E_PLAN_CATALOG` (409, version mismatch), `E_PLAN_UNAVAILABLE` (502/504), `E_PLAN_INVALID` (502: the model's output did not parse after one retry) |
| Provider | `server/src/plan/provider.rs::PlanProvider` trait, one adapter per candidate (TE-15); `system_prompt()`, `user_message()` and `output_schema()` from `offcut-plan`; structured-output mode where the provider has one; `PLAN_UPSTREAM_TIMEOUT = 25 s`; one retry when the output fails to parse, with the parse error appended |
| Server checks | Parse into `EditPlan`; set `source = Model { provider, model }` itself; check V-1 and sizes. Transcript-level validation (V-2 to V-8) is the client's, because only the client has the user's edits |
| Quotas | In-memory per-IP bucket: 10/min and `ANON_PLANS_PER_DAY = 3` for anonymous requests; `ai_usage` for signed-in users (v1.3, §52): `FREE_PLANS_PER_MONTH = 40`, `CREATOR_PLANS_PER_MONTH = 1000` soft (logged above, blocked at 3×). Counted only on success (assumptions, TE-15). The spend cap of §52 applies |
| Storage and logs | The body is never written to the database or to a log; logged: route, status, latency, provider latency bucket, input and output token counts. `log_redaction.rs` runs a plan request with marker words. Account deletion removes `ai_usage` and `ai_shorts_grants` rows; account export includes them |
| Config | `PLAN_ENABLED`, `PLAN_PROVIDER`, `PLAN_MODEL`, `PLAN_API_KEY` (Render environment only; `PLAN_ENABLED=false` makes the route answer `E_PLAN_UNAVAILABLE`) |
| Network | Server → provider API host (HTTPS) joins §24.1 "From the server". The browser's allowlist does not change: it calls the app origin |

`server` may depend on `offcut-plan` (a pure crate: catalog, prompt, schema, types), added to the §2 row for `server`.

**(V6.2, as built: D-458, D-460, D-461, D-464, D-494, D-498, D-500.)**
- **The provider is DeepSeek, not Gemini** (D-500):
  - `server/src/plan/deepseek.rs` is the one adapter behind `PlanProvider`;
  - the request is `POST https://api.deepseek.com/chat/completions` with `deepseek-flash`, thinking disabled, `response_format: json_object` and temperature 0.2;
  - the canonical `output_schema()` goes in the system message as text (DeepSeek refuses `json_schema`), and `validate` stays the gate;
  - the answer is parsed item by item (`plan/parse.rs`), so an unknown emoji or icon name drops that item, not the plan;
  - the server sets `format`, `catalog` and `source` itself, never from the model.
- **`plan/route.rs`, in order:**
  1. `PlanIp` and the 64 kB body limit;
  2. the bearer, before the body (D-494; there is no anonymous quota), then `PlanUser`;
  3. the JSON body;
  4. the catalog (409 `plan_catalog`);
  5. `PLAN_ENABLED` and the month's spend (502 `plan_unavailable`);
  6. the person's row still there (401);
  7. the quota (429 `plan_quota`);
  8. the transcript checked (400).
- **Deadlines:** the provider gets `PLAN_DEADLINE` = 18 s for the whole request and 15 s a call. One retry follows a parse failure, only with 4 s left. A provider failure is 502 `plan_unavailable` and a timeout 504; neither is retried.
- **Size:** an answer over 32 kB is 502 `plan_invalid`.
- **Logs:** one line per call, with a kind, a seconds bucket, the input, cached-input and output tokens, `retried`, `items` and `items_skipped`. Never a word (D-464).
- **The client:** one attempt, 20 s in all (`PLAN_CLIENT_TIMEOUT_MS`). Every failure is the rules plan with its reason (D-466).

### 40.8 Client flow (C-4 from V6.2)

```text
ASR → audio chain → prosody → [AI plan on?] request-plan.ts: POST /plan (PLAN_CLIENT_TIMEOUT = 30 s)
                                 ok → validate → ValidPlan (source Model)
                                 off / offline / error / quota / timeout / V-1 → from_rules → validate (source Rules)
   → timemap::build → assemble (prog48) → build_scene → preview
```

- `GET /healthz` is sent at `clip_accepted` to wake the API while ASR runs (P-5 revised).
- The plan is stored in the IndexedDB `plans` store (schema version 2; the migration removes the unused `events` store, closing §39.3 item 28's `events` entry). Restore uses the stored plan and never re-requests. "Regenerate plan" is the only re-request; overrides keyed by `EventId` survive where ids match.
- `PipelineStage::Plan` joins `stage_timing`. New analytics (enum and count properties only): `plan_result { source, outcome, latency_bucket, items, dropped_by_reason, cuts }`; `export_done` gains `items_kept`, `items_disabled`, `cuts_kept`, `cuts_restored`, `music_on`, `brand_kit_on`.
- Settings: the "AI edit plan" switch (default on), provider name and data-use summary; with the switch off, no plan request is sent and no transcript word appears in any request (`privacy-network.spec.ts` case "plan off").

**(V6.2, as built: D-466, D-467, D-469, D-490, D-504.)**
- **The call sequence:** with `ai_plan` off, the worker calls are those of `b73fea5`, in order (a `run-pipeline.test.ts` case). With it on:
  - step 11b, `choose-plan.ts`' `startPlan`, asks once the prosody is measured, or reuses the plan kept for the clip;
  - step 12b, `choosePlan`, checks a model plan with the stateless binding `validate_plan` and uses it when it keeps at least one item, else the rules plan with its reason, told once in a toast;
  - `setScene` is then called once.
- **No stage:** `PipelineStage::Plan` is not added; the wait sits inside `detect_scene`.
- **The store:** `plans` at IndexedDB structure version 3, opened only by `plans-repo.ts`, holding `{ plan, source, requestedAt, catalog }`. The unused `events` store is kept, and a build with `ai_plan` off never upgrades.
- **Run again, before V7:** no clip is restored on a reload; a clip's run again (Retry, Process again) uses its kept plan with no request. "Regenerate plan" is the only re-request.
- **Word edits:** a word edit keeps a model plan, which the scene validates against the edited words.
- **What is shown:** `set_scene` also answers `kept`, the ids its validation kept, and with `ai_plan` on the visuals list and `export_done`'s item counts use only those (D-504).
- **Analytics:** `plan_result` is sent once per run that chose a plan.

### 40.9 Seams V4, V5 and V6 must build (so V6.2 starts without rework)

| # | Version | Seam | Replaces | Proof |
|---|---|---|---|---|
| S-1 | V4 | `OutMs`, `TimeMap::identity`, `FrameIdx::to_out_ms`; scene and preview on program time; `VideoSource` reads `to_src` | One-timeline rule of §10.1, TDR-7 | `timemap_identity.rs`; `preview-sync.spec.ts` unchanged |
| S-2 | V4 | `offcut-plan` crate: `EditPlan`, `ValidPlan`, `from_rules`, `validate` (V-1 to V-6 for the four kinds), `catalog` (four kinds), `prompt` + `output_schema` generation | `SceneInput.events` | `plan_from_rules.rs`, `validate.rs`, `catalog_snapshot.rs`, `catalog_complete.rs`; goldens unchanged |
| S-3 | V4 | `QuantityRef` resolved from effective words by `validate` (V4 D-121: the scene draws resolved values and parses nothing) | Literal `Quantity` in scene input | `display_list_snapshots.rs` with a word-edit case |
| S-4 | V4 | `DrawCmd::Video` (crop and zoom per frame), `DrawCmd::Image` (type only), renderer drawing in list order | `Renderer::render(frame, crop, rotation, list)`, `Scene::crop()` | Snapshots include the `Video` command; a punch-in golden (scale 1.1) |
| S-5 | V4 | `FontKey` + `FontSet`; `Theme` roles; `StyleSpec.motion`; zone table | Closed `FontId` enum (v2implementation D-29); `EVENT_ZONE`/`CAPTION_ZONE` constants | `styles_use_theme.rs`; goldens unchanged |
| S-6 | V4 | `offcut-dsp::program::assemble` (identity, no music), `silence_profile`, `AudioWorkerApi.assemble`; preview plays `prog48` | Preview playing `out48` directly | `program_identity.rs` |
| S-7 | V4 | Review UI generic over plan items (`PlanItemList`), a cuts section that renders nothing when the plan has no cuts | `EventToggleList` | `review-edit.spec.ts` |
| S-8 | V5, built | Export loop over `OutMs`: `session.source_at(t)`, then `frameAtBlocking(recorded(pos))`; encodes `prog48`, read from `clips/<id>/prog48.f32` beside `out48.f32`; `frame_count` from `out_duration`; the test passes the expected duration as an argument apart from the source, with no sidecar file (v5implementation D-210), and `verify_mp4.py` checks 5 and 8 compare against it (the verifier still shares no code with the product) | Export reading `TimeMs` and `out48` | `timeline-preserved.spec.ts` (identity case) unchanged in outcome; the seam case of `export-loop.test.ts` (a map that jumps and holds), which no browser can reach while `set_scene` refuses other maps |
| S-9 | V5, built | Render-cache key: canonical JSON of ten inputs (§21.4), with the plan, the time map, a theme string and a music id as they are rather than hashes of each, since the whole is hashed; the profile kind from the token the worker verified (v5implementation D-220 to D-224) | §21.4 cache key | The `render-cache` group of `start-export.test.ts`, a case per input |
| S-10 | V6 | Account delete and export iterate a list of per-user tables, so `ai_usage` and `ai_shorts_grants` are two more entries | Hard-coded table list | `account_delete.rs` |

Nothing else is reserved, except the plan's wire format, fixed at `PLAN_FORMAT = 1` in V4 (D-119, D-163): from V6.2 a stored or sent plan is a contract, and fixing it once costs less than changing it. Each later variant (`ItemKind` codes 4+, `Cut` arms used, `VideoSlot` members, masks) is added by the release that draws or applies it.

### 40.10 The V6.x releases

v1.3 reorders them; §42 maps the old numbers. Each row's detail is in the section named; `buildplan.md` §8A gives the build order, demo and exit checklist.

| Release | Builds | Section | Done when |
|---|---|---|---|
| V6.1 Long source | The long page, `route_probe` with `Entry`, 4K and HEVC input, reading in place, windowed audio, cloud transcription in checkpointed windows, source machine, flags, multi-manifest (v1.4) | §43 | A 45-minute file reaches a resumable master transcript; flags off equals V5 |
| V6.2 Edit plan | `/plan`, provider adapter, spend cap, quotas, `ai_usage`; `request-plan.ts`, `plans` store, settings switch; `Emphasis`, `IconPop`, `PunchIn`, `Emoji`; `DrawCmd::Image`; text extras panel; (v1.5) `HookTitle` with its lockup, caption glow, punch words, emoji under the keyword, motion by rules, easing curves, the Enhance panel with Voice cleanup and Motion | §40.7, §40.8, §47, §52 | AI plans validate and render on the reference clip; fallback works with the API down; `privacy-network.spec.ts` revised; E-11 instrumentation; TE-15 recorded; `Enhance::v5()` equals V5 (DC-16) |
| V6.3 Cuts and Assembly | `Cut` arms, `timemap::build`, `assemble` splices, jump-cut alternation (`JUMP_ZOOM = 1.08`, assumption), cuts list, filler switch; `Assembly`, regions, rebase | §40.1, §40.6, §44 | `cuts_applied.spec.ts`; `stitched.spec.ts`: out-of-order parts export correctly |
| V6.4 Director | `/shorts`, `validate_shorts`, rules producer, shorts list, batch export | §45 | A long fixture gives 5 exported shorts passing 11 checks, with the API up or down |
| V6.5 Vision and framing | Coarse and fine passes, `ShotSummary`, `FramePath`; (v1.5) following the speaker, the split of two faces | §46 | A moving speaker stays framed; (v1.5) in a two-person shot the person speaking is on screen (TE-27); the Director reads shot lines; `FramePath::none` equals V5 |
| V6.6a Picture (v1.5) | The blur pass; automatic exposure, white balance and contrast per shot; camera matching; face light and vignette; the Colour & light switch | §47.4 | Goldens with a grade; `grade: None` lists unchanged; TE-29's panel |
| V6.6b Graphics (v1.5; v1.3's V6.6) | Parts A-E of §47: cards, number moments, kinetic presets, masks and shape transitions, progress bar, end card; (v1.5) four display fonts, six presets, word pages | §47, §47.3, §40.5 | Goldens per kind and style; heaviest preset within budget; simple lists unchanged |
| V6.7 Music and sound | Music import (the person's own track only, v1.5), ducking, mix loudness, music cue, sound cues; (v1.5) synthesized sound effects placed by rules, voice tuning | §40.6, §47.1 | `mix_ducking.rs`, `sfx.rs`, `voice_tune.rs`; verifier check 7 on a music export |
| V6.8 Layers | `VideoSlot`, multi-run compositor, asset box, B-roll (own and Pixabay stock, placed by the plan or by hand on a word, v1.4), join transitions (v1.5: zoom in and out), split and full layouts; (v1.5) the quality check | §48, §47.6 | Verifier 11/11 with B-roll and transitions; simple lists unchanged; the quality check over the corpus |
| V6.9 Chat | `PlanOp`, `effective_plan`, `/chat-edit`, `ChatPanel` | §49 | Each op applies, refuses and undoes; privacy test covers chat; non-part ops within the edit budget |
| V6.10 Brand, memory, queue | `BrandKit`, `CreatorMemory`, queue | §50 | Golden matrix under a kit; memory changes the next prompt; a queue of three finishes around a failure |
| V6.11 Person mask | Matte pass, `Video.matte`, text behind, background | §51 | TE-25's gate, per effect; verifier 11/11 |

### 40.11 Still forbidden

- An item kind in the catalog without drawing code and a golden frame; a number on screen that is not parsed from the transcript; a timestamp written by the model.
- A cut inside a word, a reorder inside a `TimeMap`, a speed change (PS §9.2). (v1.3: choosing and ordering whole sentences is §44's; a user-asked whole-sentence cut is §49's.)
- The browser calling the AI provider, the key in the client bundle, streaming provider output to the browser.
- Storing or logging the transcript, sentence list, chat text or asset labels on the server; sending media, music, logos, fonts, frames or edits anywhere.
- Generated B-roll (PS §21 item 14). The user's own B-roll and, from v1.4, Pixabay stock are §48.

---

## 41. The Compatibility Contract (v1.3)

**Model: everything added by V6.1-V6.11 is added beside what exists. A clip of at most 90 seconds, dropped with every feature flag off, behaves exactly as V5 does: the same rejections, the same frames, the same audio samples. A release that cannot prove this does not ship. Rejected: rewriting the clip path around long videos (it would put five versions of proven work at risk for no user gain), a second pipeline for shorts (two implementations of every output rule), a branch per release that merges at the end (nothing is proven against production until the end).**

This section is the founder's rule of 2026-11-02 written as checks: nothing that works is broken without being certain the new thing works; each release leaves the code ready for the next one; a feature not worth what it would break goes back to the founder.

### 41.1 Protected behaviour

These are the proofs that must pass, unchanged, on every commit of V6, V6.1-V6.11 and V7-V10. A file in this list changes only through a row of §41.2.

| Protected | Proof | Why it is enough |
|---|---|---|
| Frames of every existing style, event and framing | `golden_frames.rs`, `display_list_snapshots.rs`, `layout_safe_area.rs`, `determinism.rs`: the stored `.snap` files and golden PNGs have an empty diff | A pixel or a command that moves fails here first |
| The identity case of the audio | `program_identity.rs`: `assemble` with the identity map and no music returns `out48` sample for sample | Cuts, music and regions all add to `assemble`; this keeps the old path exact |
| The identity case of time | `timemap_identity.rs`, `timeline-preserved.spec.ts`: export duration equals source duration within one frame | |
| The export | `export-creator.spec.ts`, `export-free.spec.ts`: all 11 checks of `verify_mp4.py` | The verifier shares no code with the product |
| Preview | `preview-sync.spec.ts`, `pipeline-preview.spec.ts`: first frame, seek, edit and drift within their V5 readings' budgets | |
| Review | `review-edit.spec.ts` with axe at 0 violations | |
| Rejections | `probe_rejections.rs`, `rejections.spec.ts`, with the rows of §41.2 | |
| Failure handling | `cancel.spec.ts`, `failure-recovery.spec.ts` (V7) | |
| Privacy | `privacy-network.spec.ts` from V8; until then `check-hosts.mjs` and the V5 network reading | |

**Test counts only rise.** The Rust, Vitest, `pnpm e2e` and media counts recorded at each release's close may not fall below the previous release's. The baseline is V5's close: Rust 657 passed, 0 failed, 4 ignored; Vitest 336; `pnpm e2e` 12; verifier self-test 16; media 94 passed, 1 skipped.

**The two-path proof.** From V6.1, CI runs the media suites twice in the release's one full run: all feature flags off, and all flags that have shipped on. Both must pass. A reference clip with flags on and the AI plan off must produce the same frames and samples as with flags off (`flags_identity.spec.ts`).

### 41.2 Declared changes

A protected behaviour changes only here. Each row names what changes, when, what stays and which test proves it. Any other change needs a new row, and a new row needs the founder (§41.4).

| # | Release | What changes | What stays | Proof |
|---|---|---|---|---|
| DC-1 | V6.1, flag `long_video` | **v1.4:** a second page, `/app/long`, takes long videos (§43.1); `/app` gains one link to it, and `REJECT_DURATION`'s panel on `/app` gains one action, "Make shorts from it", with its words unchanged. On `/app/long`, `REJECT_DURATION` fires above 45 minutes, and a file of at most 90 s opens on the clip path. `rej_too_long_91s.mp4` stays in `rejections.spec.ts` (the clip page keeps V5's rule), which gains one assertion for the new action with the flag on; `rej_too_long_46min.mp4` (a sparse low-bitrate file) joins `route_long.spec.ts`. *(v1.3 routed by duration from one page.)* | Flag off: exact V5 behaviour. Flag on: the clip page's rules, rejections and words | `route_probe.rs`; both test runs of §41.1 |
| DC-2 | V6.1, flag `high_res` | Inputs up to 3840 px on the long side and HEVC where `isConfigSupported` says yes are accepted on both paths. `rej_4k.mp4` moves to an acceptance case; `rej_8k.mp4` takes its place. `REJECT_HEVC` fires only when the browser cannot decode it | Output sizes; every H.264 clip decodes as before | `demux_fixtures.rs` (hvcC), `high-res.spec.ts` |
| DC-3 | V6.1, flag `high_res` (v1.4; v1.3: `long_video`) | On the clip page, `MAX_FILE_SIZE` becomes 16 GB, so a 90 s 4K clip fits. The in-page `REJECT_FILE_SIZE` fixture references 15,259 × 1 MiB. The check still runs before any copy. The long page has no size check (§43.2) | Flag off: 500 MB | `rejections.spec.ts` |
| DC-4 | V6.1 | `model-manager.ts` removes a directory only if no manifest in `config/models/` names it, instead of "any other model" | The ASR manifest, hashes, and download behaviour | `download.test.ts`, `model-download.spec.ts` |
| DC-5 | V6.3 | `set_scene` stops refusing a non-identity `TimeMap`, and refuses only a map whose duration disagrees with the program audio | Identity behaviour | `scene_api.rs`, `cuts_applied.spec.ts` |
| DC-6 | V6.3 | §36's "no reorder" and "no removing a whole sentence" apply to `Cut`s only. Selecting and ordering whole sentences belongs to the `Assembly` (§44). `TimeMap` keeps refusing out-of-order segments | `TimeMap`, its deserializer and `Cut` are unchanged | `timemap_identity.rs`, `assembly.rs` |
| DC-7 | V6.2 | Rule V-1 reads `catalog <= CATALOG_VERSION` for a stored plan, so a deploy that bumps the catalog does not throw away users' plans. A new response must still equal `CATALOG_VERSION` | `PLAN_FORMAT = 1` | `validate.rs` |
| DC-8 | V6.5, V6.6, V6.8 (v1.5; v1.3: V6.6, V6.8) | The renderer's support rule (D-133) widens: `Image`, then several `Video { Main }` commands from one decoded frame (V6.5, the split of DC-17), then the blur pass and masks (V6.6), then other slots (V6.8) | **A list that meets the old rule takes the old code path, byte for byte.** Goldens unchanged | `golden_frames.rs`, `composite.rs` tests |
| DC-9 | V6.5 | `Video.src` may change per frame through a `FramePath` | With no `FramePath`, `framing.rs` gives V5's crop | `framing.rs` |
| DC-10 | V6.1, V6.5, V6.11 | `asr.worker` hosts every inference session (TDR-3 widens from one file to `workers/asr/*-runtime.ts`). INV-12 reads "no inference session and the render session coexist" | Worker count stays four; `whisper-runtime.ts`'s interface is unchanged | `run-pipeline.test.ts` |
| DC-11 | V6 | Comment-only cleanup of the old V6.x numbers (§42), including regenerated `web/src/gen/domain.ts` comments | No token changes outside comments | `check-gen-clean.sh`; an empty diff with comments stripped |
| DC-12 | V6 | `export-creator.spec.ts`, the case "no token" only: what the person is told changes from "accounts are not open yet" to the sign-in dialog with the limits. Founder's yes of 2026-11-02, "carefully": its title and one assertion change, and one line closes the dialog | Every other assertion of that case, and every other case of the file, unchanged: no download, no `export_started`, nothing in `exports`, the button still enabled. No exported file changes | The spec's diff read in P147; the file's other cases passing unchanged; `verify_mp4.py`'s 11 checks |
| DC-13 | Design release | Caption lines below the middle of the band are lowered in the caption block instead of centred: by a share of the room under centred lines (to the block's bottom less the style's vertical decoration, `StyleSpec::decoration_height`: Clean 6.5 lp, Bold 9, Mono 32) that rises evenly from 0 at the band's middle to `BOTTOM_LINE_DROP` = 0.5 at Bottom (`placement::line_drop`). At Bottom a one-line caption moves 36-45 lp lower (Clean 45, Bold 40, Mono 36), a two-line one 15-23 lp, and the Middle snap (0.55) about 4 lp. Clips saved at Bottom render their captions lower on reopening. Goldens and display-list snapshots of placements below the middle are regenerated; their diffs are only glyph runs', pills' and panels' `y`. The e2e helper `dragCaptions` takes the block at its centre on `data-hit` "captions" or "word" (it named "block", a kind the page never reports, so it matched only a word). Founder's choice, 2026-11-05: option A's mechanism, but "a little bit low", as the apps draw their own UI at the bottom (designchangelog Entry 16) | The block, event zone, safe area, watermark, hit regions, `CaptionPlacement` values and their storage; every frame at placements 0 to 0.5, bit for bit; the export's 11 checks | `layout_safe_area.rs`, `display_list_snapshots.rs` (40 files, every diff a `y`), `golden_frames.rs` (14 of 16 regenerated), `export-*.spec.ts`, `preview-sync.spec.ts`, `review-edit.spec.ts`, `pipeline-preview.spec.ts` |
| DC-14 | Cloud transcription | `media_guard` admits exactly `POST /api/v1/transcribe` with `audio/ogg`; that route's body limit is 1 MiB. `no_media_routes.rs` names it as the one exception (`AUDIO_BODY`): its other media types get 415, `audio/ogg` passes the guard (401 without a bearer, 413 over 1 MiB with one). `/healthz`'s body gains `cloud_transcribe` (`notify_me.rs`'s exact-body assertion gains the field). Founder's decision of 2026-11-06 | Every other route: 415 for every media type, no body over 64 kB, the same assertions. The device path, its frames, samples, transcripts and exports: with the fake's `/healthz` (no field) every protected suite runs on the device | `no_media_routes.rs`, `router.rs`'s unit tests, `transcribe_route.rs`, `cloud-transcribe.spec.ts` (identity); every §41.1 proof unchanged |
| DC-15 | V6.4 (v1.4) | `POST /shorts` takes a JSON body of up to `SHORTS_BODY_LIMIT = 160 kB`, for a 45-minute source (§45.2). `no_media_routes.rs` names it as the second exception to "no body above 64 kB", for size only: it still answers 415 to every media type. Founder's decision of 2026-11-08 (45-minute sources) | Every other route's 64 kB and 415s; DC-14's exception | `no_media_routes.rs`, `shorts_route.rs` |
| DC-16 | V6.2, flag `motion`; each later field of `Enhance` (v1.5) | **The Enhance settings (§47.5).** A new clip's `EditState.enhance` takes `Enhance::defaults(flags)`, the person's remembered choice if any. With `motion` on, a talking-head frame's `Video` command takes the motion track's scale (§47.2), so its display list differs from V5's in `Video.src` only. `flags_identity.spec.ts` sets `Enhance::v5()` through the dev hook and keeps its assertion (flags on, AI plan off and `v5()` give V5's frames and samples); a second case reads the defaults and asserts that the lists differ only in `Video.src` (V6.6a adds `Video.grade`; V6.7 adds the sound cues in `prog48`). A stored clip without the field reads `Enhance::v5()`, so it reopens as it was saved | Every frame and sample under `Enhance::v5()`; every stored clip; goldens | `flags_identity.spec.ts`, `enhance.rs`, `motion.rs` |
| DC-17 | V6.5, flag `speaker_framing` (v1.5) | `FrameKey` gains `layout: FrameLayout { Single, Split { top_cx, bottom_cx } }` (`#[serde(default)]`: `Single`). A `Split` key puts two `Video { Main }` commands in the list, the two faces stacked, from one decoded frame; `composite.rs` gets its second path here (DC-8) | `FramePath::none()` gives V5's crop; a shot with one face gives §46.4's path, unchanged; `is_simple_list` lists take the old path, byte for byte | `framing.rs`, `speaker.rs`, `composite.rs` (simple path identical) |
| DC-18 | V6.6a, flag `picture` (v1.5) | `DrawCmd::Video` gains `grade: Option<Grade>` (`#[serde(default)]`: `None`). `None` takes the existing video pipeline, byte for byte; `Some` draws through a graded variant of the same pass: one pass, one uniform per frame (§47.4) | Every list with `grade: None`; goldens; `is_simple_list` (a graded `Video` is still simple: no second path) | `golden_frames.rs`, `grade.rs` |
| DC-19 | V6.7, flag `voice_tune` (v1.5) | Inside "Voice cleanup", the chain gains a measured-need step: the denoiser's wet share set from the measured noise floor, and a bounded three-band voice EQ (§47.1). `out48` changes for clips prepared with it | `Enhance::v5()` runs V5's chain exactly; `program_identity.rs` (`assemble` over any `out48`) | `voice_tune.rs`, `program_identity.rs`, `flags_identity.spec.ts` |

**(V6.2, as built: D-473, D-440, D-485.)**
- **DC-8's V6.2 part:** `composite::video_of` accepts `Image` commands after the video; a list without one takes the old path unchanged.
- **DC-16:** `EditState.enhance` is skipped when it equals `Enhance::v5()`, so a V5 edit stores and hashes as at `b73fea5` (`flags-identity.spec.ts` case 1). The page leaves it out too (P209).
- **Existing tests changed** (24 entries, each widened, none deleted): listed in the V6.2 plan's D-440. The release's only ignore (`catalog_complete.rs`) was removed in P206.

### 41.3 How each release is made safe

1. **Flag first.** New behaviour sits behind a flag in `web/src/config/features.ts` (`FeatureFlags`, read from a Vite `define`, never from the network). Flags: `cloud_transcribe` (built in the cloud transcription release, with `features.ts`; `OFFCUT_FEATURES_OFF` names the flags a build turns off), `high_res`, `long_video`, `ai_plan`, `motion` (v1.5, §47.2), `cuts`, `assembly`, `director`, `vision`, `framing`, `speaker_framing` (v1.5, §46.6), `picture` (v1.5, §47.4), `finished_look`, `music`, `voice_tune` (v1.5, §47.1), `layers`, `stock_broll` (v1.4, §48.6), `chat`, `brand_kit`, `creator_memory`, `queue`, `person_mask`. Production turns a flag on only after that release's proofs pass on `main`. **(V6.1, as built: D-393, D-394.)** Each flag has a default, and the V6.x flags default to off: `DEFAULTS = { cloud_transcribe: true, long_video: false, high_res: false }`; a build turns flags on with `OFFCUT_FEATURES_ON` (a comma list) beside `OFFCUT_FEATURES_OFF`, and an unknown name, or a name in both, stops the build. So "every flag is on unless the build names it" (the cloud release) becomes "every flag takes its default unless the build names it". Two e2e builds: `pnpm build:e2e` (production's flags) into `dist/`, and `pnpm build:e2e:flags` (`long_video,high_res` on) into `dist-flags/`, served on a second port to the Playwright project `media-flags`, which runs `FLAG_SUITES`; CI builds and runs both. Production sets `OFFCUT_FEATURES_ON` in Vercel only to turn a flag on, after the release (the V6.1 plan's §12.3).
2. **Baseline read first.** Before a release's first code commit, read the protected proofs and record them in its changelog.
3. **Additive data.** A new field on a stored or sent type has `#[serde(default)]` and a default that means "the V5 behaviour". A new IndexedDB store comes through a numbered migration. A server migration stays compatible with the previously deployed binary (§23.3). The plan's wire format stays `PLAN_FORMAT = 1`; new kinds take new codes (§40.2).
4. **Old callers keep their function.** `transcribe`, `decodeAudio`, `importClip`, `run-pipeline.ts` and `validate_probe` keep their signatures. New paths call new functions, which call shared stages.
5. **Identity constructors.** Every new abstraction has a do-nothing constructor (`Assembly::whole`, `FramePath::none`, `SceneAssets::empty`), and the clip path uses it. A test asserts that it changes nothing.
6. **One release, one branch, one merge.** Build on `v6.n-build`; merge after the gate; tag. A failed gate leaves `main` untouched.
7. **Kill switch.** A flag can be turned off by a redeploy of the static build. A plan, short or asset stored under a flag stays readable with the flag off: the clip opens without the feature, it never fails.

### 41.4 Stop and ask the founder

The implementer stops, writes the question into the changelog's "Open items" and asks, with a recommendation, whenever any of these is true. The founder decides to drop, defer or pay the cost.

| Trigger | Example |
|---|---|
| A protected proof would have to change and §41.2 has no row for it | A caption preset needs a change to an existing golden |
| A feature's cost looks out of proportion to its value | Video inside shapes needs more than one new render pass, or more than 2 days beyond its estimate |
| A release runs over twice its estimate | A release estimated at 10 prompts reaches 20 |
| A new runtime, a model download above 50 MB, a new browser host, or a new paid service | A second inference runtime for the face detector |
| A stored or sent format needs a breaking change | A field that cannot default |
| A promise changes | "Your video is not uploaded", "a number on screen was spoken", flat price |
| An experiment fails and its row offers a choice | TE-20 says long jobs die in a hidden tab |

### 41.5 What each release must leave for the next

| Release | Leaves ready | Used by |
|---|---|---|
| V6 | Accounts, entitlement token, `ai_usage`-ready account lists (S-10) | V6.2 quotas, V6.4 long-video grant, V6.10 brand gating |
| Cloud transcription | `Transcriber`, `POST /transcribe`, `encodeForCloud`, `fromCloud`, `ogg-opus.ts`, `transcribe-cloud.ts`, the availability gate, `features.ts` | V6.1 windows of long sources, at most 90 s each |
| V6.1 | `route_probe` with `Entry`, `SourceInfo`, the long page, reading in place, `pcm16` in `sources/`, cloud windows with checkpoints and pauses, the source machine, multi-manifest model manager | V6.4, V6.5, V6.10 queue, V6.11 |
| V6.2 | `/plan`, provider trait, spend cap, plan store, catalog growth rule, `DrawCmd::Image`; (v1.5) `Enhance` and the Enhance panel, `MotionTrack`, `easing.rs`, the lockup layout of `HookTitle`, caption glow | Every release that adds a kind; V6.4 and V6.9 routes share the provider; every release that adds an Enhance row |
| V6.3 | Non-identity `TimeMap`, `Assembly`, `SourcePos`, region audio, rebased transcripts, join handling; `remux_range` and `SourceMedia` (v1.4) | V6.4 shorts and their part copies, V6.8 transitions at joins |
| V6.4 | Virtual clips, shorts list, batch export, `shorts_from_rules` | V6.5 framing per short, V6.9 chat over a short, V6.10 queue |
| V6.5 | `ShotSummary`, `FramePath`, sampled-frame hand-off; (v1.5) face tracks, the speaker decision, `FrameLayout::Split`, `composite.rs`'s second path for several `Video { Main }` | V6.6a grade and face light, V6.8 split layout, V6.11 mask pass |
| V6.6a (v1.5) | The blur pass, `Grade` and its graded video pass, per-shot colour statistics, camera matching | V6.6b glass cards, V6.8 blurred fill and `Whip`, V6.11 blurred background |
| V6.6b (v1.5; v1.3's V6.6) | Mask path in the renderer, kinds and cards, kinetic motion rows; (v1.5) display fonts, presets, word pages | V6.8 layers, V6.10 brand theme coverage, V6.11 matte |
| V6.7 | Multi-source `assemble`, sound items; (v1.5) `sfx.rs` and the rules cues, `voice_tune` | V6.8 B-roll audio and its cues, V6.9 chat music ops |
| V6.8 | `VideoSlot`, multi-run compositor, asset store, `SceneAssets`; `plan_ops` with `PlaceAsset` and `RemoveAsset`, the stock search route (v1.4); (v1.5) `AddItem` for quality fixes, the quality check | V6.9's other ops and chat, V6.11 background and matte; the founding release |
| V6.9 | `PlanOp` log, `effective_plan`, op validation | V6.10 memory reads ops |
| V6.10 | Theme from kit, `CreatorMemory`, queue | Launch |
| V6.11 | Person matte, background layer | Launch; catalog additions after it |

---

## 42. Version Map (v1.3)

On 2026-11-02 the founder set the order: **V6 first, then V6.1-V6.11, then V7-V10.** Older documents, changelogs and code comments use the v1.2 numbers. This table reads them. `M` ids keep their meaning; they are labels, not an order.

| v1.2 name | Contents | v1.3 name | Step |
|---|---|---|---|
| none | Long source: long files, 4K and HEVC input, windowed audio, resumable transcription | **V6.1** | M1.16 |
| V6.1 | AI edit plan backbone, emphasis, icons, emoji, text extras | **V6.2** | M1.9 |
| V6.2 | Pause and silence cuts; now also the `Assembly` for stitched parts | **V6.3** | M1.10 |
| none | Director: a long video becomes a list of shorts | **V6.4** | M1.17 |
| none | Local vision signals and face-tracked framing | **V6.5** | M1.18 |
| V6.4, V6.5, V6.6 | Finished look: structure visuals, kinetic presets, video inside shapes, shape transitions, hook title, progress bar, end card | **V6.6** | M1.12, M1.13, M1.14 |
| V6.3 | Music and ducking; now also sound effects | **V6.7** | M1.11 |
| none | Layers: B-roll, asset box, transitions at joins, split layout | **V6.8** | M1.19 |
| none | Chat editing | **V6.9** | M1.20 |
| V6.7 | Brand kit; now also creator memory and the back-catalogue queue | **V6.10** | M1.15, M1.21 |
| none | Person mask: text behind the speaker, background change | **V6.11** | M1.22 |

**v1.5 (founder, 2026-11-09): the numbers stay.** Each addition of the Submagic benchmark goes into the release that already holds what it needs, and V6.6 is built as two releases, **V6.6a "Picture"** (blur, automatic colour and light) and **V6.6b "Graphics"** (v1.3's V6.6 parts A-E without the hook title, plus fonts and presets), tagged `v6.6a` and `v6.6b`. Code comments that say "V6.6" mean V6.6b unless they name colour or blur.

**Moved to post-launch (founder, 2026-11-02):** chapter markers and the lower third (PS §21 item 15). `Chapter` stays in the plan's wire format, always empty, so `PLAN_FORMAT` does not change.

**Code comments.** About 57 comments in `crates/`, `web/src` and `verify/` use the v1.2 numbers. V6's first prompt rewrites them (DC-11). Closed changelogs and V1-V5 plans stay as written, as history.

---

## 43. The Long Path: Source, Route, Reading in Place, Transcription (V6.1; revised in v1.4)

**Model (v1.4, the founder's decisions of 2026-11-08): the long path has its own page, and the person chooses it; a file's duration no longer chooses the path. A long video is a source. The dropped `File` is read in place for the session and is never copied whole. Its audio is decoded once, in windows, into OPFS; its transcript is built in the cloud, in checkpointed windows of at most 90 s; after the Director picks the parts (§45), only those ranges are copied into OPFS, as one small MP4 per part (§44.8). Nothing else in the product reads the source directly: shorts read their parts through an `Assembly`. Rejected: copying the whole file into OPFS first (v1.3's design: up to 16 GB of disk and minutes of copying before any work, for a video of which a few minutes are used), storing a `FileSystemFileHandle` to read the file again later (the handle's name is a file name stored in the browser, against P-11; permission prompts on every return), transcribing a long source on the device (about 50-60 minutes for 45 minutes of speech on D1, derived from §16's speed, against a few minutes in the cloud, TE-C4), YouTube-link import (a server that downloads video breaks INV-4 and the platform's terms), cloud transcription of a whole source in one request (the route takes at most 92 s).**

**What reading in place costs.** A `File` lives only as long as the page that received it. A reload, a crash or a discarded tab before the parts are copied keeps the audio, the transcript and every checkpoint, but not the file: a stage that needs the file asks for the same file again, checked by `sourceHashPrefix` and `sourceBytes` (§43.7). Once a short's parts are copied, that short never needs the original file again.

### 43.1 Entry and routing

Two entry points (PS §10, §10.1):

| Page | Route | Takes | With `long_video` off |
|---|---|---|---|
| Edit a clip | `/app`, as today | A clip of at most 90 s, by V5's rules exactly, plus DC-2 under `high_res` | Unchanged, with no link to the long page |
| Make shorts from a long video | `/app/long` (new, flag `long_video`) | A video of more than 90 s, up to 45 minutes. A file of at most 90 s dropped here opens on the clip path, with one line: "This one is short, so it opens as a clip." | The route does not exist and redirects to `/app` |

`/app` gains one link to `/app/long` (flag on), and the landing page's "Start free" offers both pages. `/app`'s first screen, drop zone and copy are otherwise unchanged, so `openEditor` and every protected suite see the page they know (DC-1).

```rust
// offcut-types/src/source.rs
pub struct SourceId(Uuid);                // ids.rs: created on import of a long file
pub struct SourceInfo {                   // constructed only by route_probe
    pub duration: DurMs,
    pub display_width: Px, pub display_height: Px,
    pub rotation: Rotation,
    pub orientation: Orientation,
    pub is_vfr: bool,
    pub is_hdr: bool,                     // true is refused unless TE-21 accepted it
    pub video_codec_string: String,
    pub audio_codec_string: String,
    pub audio_sample_rate: Hz, pub audio_channels: u32,
    pub audio_track: u32,                 // the first enabled track; the long path accepts more than one
}
pub enum Entry { Clip, Long }             // v1.4: the page the file was dropped on
pub enum Route { Clip(ClipInfo), Long(SourceInfo) }
pub struct RouteFlags { pub long_video: bool, pub high_res: bool, pub hdr_ok: bool }   // from features.ts; hdr_ok from TE-21

// offcut-mp4/src/validate.rs  [ONLY]
pub fn validate_probe(p: &ProbeInfo, decode_supported: bool) -> Result<ClipInfo, RejectReason>;   // unchanged
// offcut-mp4/src/route.rs, re-exported by validate.rs (V6.1, as built: D-395 as read)
pub fn route_probe(p: &ProbeInfo, detail: &VideoDetail, decode_supported: bool, entry: Entry, flags: RouteFlags)
    -> Result<Route, RouteReject>;   // RouteReject = V5's RejectReason, or SourceReject::Hdr (REJECT_HDR)
```

**(V6.1, as built.)** V5's tests pin `RejectReason`, `ErrorCode`, `limits.rs` and `VideoTrackInfo`, so the new members live beside them: `REJECT_HDR` is `SourceReject::Hdr`, carried with V5's reasons by `RouteReject` (serialized as the bare code); `E_SOURCE_READ` and `E_SOURCE_AUDIO` are `SourceErrorCode`; V6.1's constants are `source.rs`' `SOURCE_LIMITS`; `is_hdr` and the HEVC codec string come in a `VideoDetail` beside the `ProbeInfo` (`Demuxer::video_entry_boxes()` gives the `hvcC` and `colr`). `SourceId` is TypeScript-only.

`route_probe`, first failing rule wins (as built, `route.rs`):

```text
 0 entry == Long && !flags.long_video                     → entry = Clip               // the page does not exist
 1 entry == Clip && !flags.high_res                       → validate_probe(p).map(Route::Clip)     // exact V5
 2 container not MP4/MOV                                  → Container
 3 no video track                                         → NoVideo
 4 video.codec == Hevc && !flags.high_res                 → Hevc                       // V5's rule while the flag is off
 5 video.codec not in {H264, Hevc}                        → VideoCodec
 6 video.codec == Hevc && !decode_supported               → Hevc                       // DC-2
 7 no audio track                                         → NoAudio
 8 the first audio track's codec != Aac                   → AudioCodec
 9 max(display_w, display_h) > (high_res ? MAX_INPUT_LONG_SIDE : MAX_LONG_SIDE) → Resolution
10 video.avg_fps > MAX_INPUT_FPS + INPUT_FPS_TOLERANCE    → FrameRate
11 detail.is_hdr && !flags.hdr_ok                         → Hdr
12 !decode_supported                                      → DecodeUnsupported
13 p.duration <= MAX_CLIP_DURATION && audio_tracks == 1   → Route::Clip                // from either page
14 p.duration <= MAX_CLIP_DURATION                        → MultiAudioTrack            // the clip path keeps its rule
15 entry == Clip                                          → Duration                   // the clip page keeps V5's limit
16 p.duration > MAX_SOURCE_DURATION                       → Duration                   // the long page's limit
17 else                                                   → Route::Long
```

Because rule 6 comes before rule 11, an HDR HEVC file on a machine without HEVC decoding reads `REJECT_HEVC`, not `REJECT_HDR`. **(V6.1, as built: D-427 as read.)** The clip page refuses HEVC with `REJECT_HEVC` even with `high_res` on: the render worker (frozen in V6.1) configures its decoder with the `avcC` only, so `high_res` on `/app` means 4K H.264 and the 16 GB limit; the long page reads HEVC sources' sound (V6 CL open item 25). `entry == Long` with `flags.long_video == false` cannot be reached (the page does not exist); `route_probe` treats it as `Entry::Clip`, and a test proves it. On the clip page, `REJECT_DURATION`'s copy is unchanged; with `long_video` on, its panel gains one action, "Make shorts from it", which opens `/app/long` with the same `File` still in memory (DC-1).

`RejectReason::Hdr` joins the enum with `REJECT_HDR`, copy ("This video is HDR. Export it as SDR, then drop it again.") and fixture `rej_hdr_hlg.mov`.

### 43.2 Limits

```rust
// limits.rs, additions (PS §9.4)
pub const MAX_SOURCE_DURATION: DurMs   = DurMs::new(2_700_000);         // 45 minutes (founder, 2026-11-08; v1.3: 20)
pub const MAX_INPUT_LONG_SIDE: Px      = Px::new(3840);
pub const MAX_SHORTS_PER_SOURCE: u32   = 10;
pub const MIN_SHORTS_PER_SOURCE: u32   = 3;                             // ask for 5-10; accept 3
pub const MIN_SHORT_DURATION: DurMs    = DurMs::new(15_000);            // (assumption, E-12)
pub const MAX_SHORT_DURATION: DurMs    = DurMs::new(90_000);            // equals MAX_CLIP_DURATION by a test
pub const MAX_PARTS_PER_SHORT: u32     = 4;                             // (assumption, E-12)
pub const FREE_LONG_VIDEOS_PER_MONTH: u32 = 1;                          // PS §11
pub const CREATOR_LONG_SOFT_PER_MONTH: u32 = 200;                       // (assumption, TE-15); logged above, blocked at 3×
pub const MAX_QUEUED_SOURCES: u32      = 50;                            // (assumption, V6.10)
```

**The long page has no file-size limit** (founder, 2026-11-08): nothing is copied whole, so a file's size costs reading time, not disk. Duration and resolution are its limits. v1.3's `MAX_SOURCE_FILE_SIZE` is not built. `MAX_CLIP_DURATION`, `MAX_LONG_SIDE` and `MAX_FILE_SIZE` stay in the file and rule the clip page exactly as in V5 while `high_res` is off (DC-3 for `high_res` on). The generated `LIMITS` the copy reads gains the new constants; `messages.ts` picks by page and flag.

**(V6.1, as built.)** V6.1 builds only the limits it reads, in `offcut-types/src/source.rs` (generated as `SOURCE_LIMITS`, since `limits.rs` is pinned at 21 constants by V5's tests): `MAX_SOURCE_DURATION` (45 minutes), `MAX_INPUT_LONG_SIDE` (3840), `MAX_FILE_SIZE_HIGH_RES` (16 GB, DC-3), `CLOUD_WINDOW_S` (90), `CLOUD_OVERLAP_S` (10, §43.6) and `MAX_SOURCE_WINDOWS` (35). The shorts', queue's and plans' limits come with their releases. `messages.ts` words `REJECT_FILE_SIZE` (16 GB) and `REJECT_RESOLUTION` (3840) by `high_res`, today's words with it off; the long page words `REJECT_DURATION` as "longer than 45 minutes" (`copy/long.ts`).

### 43.3 Reading the file in place

```ts
// workers/media/file-random-access.ts
export function fileRandomAccess(file: File): RandomAccessReader;   // FileReaderSync over file.slice; 1 MiB read cache
// workers/media/import.ts, addition
export function probeFile(file: File): Promise<ProbeResult>;
```

The `File` is posted to `media.worker` (a structured clone of the reference; its bytes are not copied), which reads it through `fileRandomAccess` for the probe, the audio (§43.5), the coarse vision pass (§46) and the part copies (§44.8). `sourceHashPrefix` is computed from `file.slice` exactly as §23.1 defines it for clips. `offcut-mp4` gains `hvcC` parsing behind `video_decoder_description` (`hevc.rs`), the HEVC codec string, and `is_hdr` from the `colr` box and the HEVC VUI. Its `RandomAccess` trait does not change. `moof` stays a rejection unless TE-12 reads at least 10% of the long corpus fragmented; then it joins V6.1's scope through §41.4.

**Disk.** Only `pcm16.f32` (about 173 MB for 45 minutes) and the part copies (§44.8) are written. `import-source.ts` calls `navigator.storage.estimate()` before the audio step and shows `B_STORAGE_LOW` when free quota is under `LONG_SCRATCH = 2 GB` (assumption, TE-17).

**(V6.1, as built.)** `fileRandomAccess(file)` (`workers/media/file-random-access.ts`) reads 1 MiB aligned blocks through `FileReaderSync`, with a cache of 8 blocks (least recently used); a `NotReadableError` (the file moved or was deleted) is `E_SOURCE_READ`. The demuxer reads it through `open_demuxer_reader` (`offcut-wasm-core/src/source_api.rs`, a structural `JsByteReader`), and `Demuxer::open_source` opens tracks of up to 400,000 samples (45 minutes of AAC is about 126,600; `open` keeps V5's 20,000). There is no `probeFile` in `import.ts` (frozen): the media worker's new `routeFile` handler opens the reader and calls `route_probe`. `is_hdr` comes from the `colr` box only (`nclx` or `nclc` with transfer 16 or 18), not from the HEVC VUI (D-397). The gate's disk check needs `round(D × 16) × 4 B + LONG_SCRATCH` (D-420).

### 43.4 Local storage

**OPFS additions** (`persistence/opfs.ts`):

| Path | Contents | Written by |
|---|---|---|
| `sources/<sourceId>/pcm16.f32` | 16 kHz mono, whole duration: about 173 MB for 45 minutes | `media.worker`, in windows |
| `sources/<sourceId>/thumbs/<n>.jpg` | Poster frames, one per short, at most 360 px wide | `render.worker` |
| `clips/<clipId>/parts/<k>.mp4` (V6.3) | One part of a short, remuxed from the source without re-encoding, with its handles (§44.8) | `media.worker` |
| `assets/<assetId>/source` (V6.8) | A user's B-roll or image, or a downloaded stock clip (§48.6) | `media.worker` |
| `clips/<clipId>/matte.mp4` (V6.11) | Person matte of one short | `render.worker` |

A short's `out48.f32` and `prog48.f32` live in `clips/<clipId>/` exactly as a clip's do. A short has no `clips/<clipId>/source`; its parts take its place.

**(V6.1, as built: D-414.)** Today's database is version 1, and `DB_VERSION` (every record's `schemaVersion`) stays 1. V6.1 adds `DB_STRUCTURE_VERSION = 2`: upgrade 2 creates `sources`, `sourceTranscripts` and `masterTranscripts` (`SOURCE_STORES`), and only `sources-repo.ts` opens the database at structure version 2 (`openSourcesDb()`); the clip path keeps opening at 1, so a build with `long_video` off never upgrades anyone's database. A build that meets a newer database (`VersionError`) opens it at its own version and uses the stores it knows (V6.1 plan §12.4). As built, `sources` holds `{ sourceId, createdAt, lastOpenedAt, sourceInfo, sourceBytes, sourceHashPrefix, stage, pcm16Seconds, levelP95, windowsDone, windowsTotal }`, `sourceTranscripts` (key `[sourceId, index]`) one window's raw words relative to the window, its length and the model, and `masterTranscripts` the `Transcript`. V6.1 keeps one source (D-415). The rows below that V6.1 does not build come with their releases, each with its own structure version.

**IndexedDB stores by release** (`persistence/schema.ts`):

| Store | Key | Value |
|---|---|---|
| `sources` | `sourceId` | `{ sourceId, createdAt, lastOpenedAt, sourceInfo, sourceBytes, sourceHashPrefix, stage, windowsDone, windowsTotal }`. No name, no path (P-11) |
| `sourceTranscripts` | `[sourceId, window]` | `{ words }` of one cloud window, the checkpoint |
| `masterTranscripts` | `sourceId` | `{ transcript, shotSummary }` |
| `shorts` | `[sourceId, shortId]` | `{ spec: ShortSpec, clipId, state }` |
| `plans` | `clipId` | `{ plan, opLog }`, V6.2 (§40.8) with the op log of V6.8 and V6.9 (§49) |
| `assets` | `assetId` | V6.8 |
| `creatorMemory` | `"current"` | V6.10 |
| `queue` | `queueIndex` | V6.10 |

The `clips` record gains `sourceRef: Option<SourceId>`, `assembly: Option<Assembly>` and `media: Option<SourceMedia>` (§44.8), each default `None`, which reads as "this clip owns its file", so a V5 record is valid unchanged.

**Eviction (§23.3 addition).** A source and all its shorts count as one entry against `MAX_RECENT_CLIPS`. `ensureFree` deletes a source's `pcm16.f32` first (nothing reads it once the shorts are prepared), then the part copies of shorts that are exported, keeping transcript, shorts and plans. A short whose parts were evicted reopens by dropping the same file again, which copies only that short's parts.

### 43.5 Audio in windows

The clip path's `decodeAudio` allocates the whole duration. The long path does not:

```ts
// workers/media/audio-decode.ts, additions; decodeAudio stays
export function decodeAudioRegion(reader: RandomAccessReader, info: SourceInfo, from: SourceMs, to: SourceMs,
                                  onChunk: (mono: Float32Array, at: SourceMs) => void,
                                  isCancelled: () => boolean): Promise<void>;
export function writeWholePcm16(sourceId: SourceId, reader: RandomAccessReader, info: SourceInfo,
                                onProgress: (done: DurMs) => void,
                                isCancelled: () => boolean): Promise<void>;   // decode → resample → append to pcm16.f32
```

- The reader is the dropped `File` (§43.3) for a source, or an OPFS part file for a short (§44.5).
- `decodeAudioRegion` starts at the audio sample whose time is at most `from - AAC_PREROLL` (`AAC_PREROLL = 100 ms`, assumption), decodes forward, and hands chunks of at most 10 s. Peak audio memory on the long path is under 50 MB (assumption, TE-14). **Learned from Tooscut's `audio-render.worker.ts`:** decode only the regions that are needed, in windows. Our code, our types; §53.4.
- **Sample-rate correction.** If a decoded chunk's rate differs from the container's (HE-AAC commonly reports half), the region continues at the decoded rate and the resampler is rebuilt. The clip path's `decodeFailure("SampleRate")` stays as it is; TE-12 decides whether to bring the correction to the clip path through §41.4.
- `writeWholePcm16` is checkpointed by the bytes already in `pcm16.f32`: after a reload with the same file, it continues from the last whole second.
- `pcm16.f32` is read by windows from OPFS, never held whole in memory.

**(V6.1, as built.)** `decodeAudioRegion` and `writeWholePcm16` are in `workers/media/audio-region.ts` (`audio-decode.ts` is frozen). The resampler is `offcut-dsp`'s `StreamResampler`, whose whole output equals `resample_mono` bit for bit (D-401). `pcm16Seconds` is checkpointed every 10 s of flushed sound; a resume truncates to it and starts its resampler on an input sample on the 16 kHz grid (D-403 as read). A histogram of sample levels gives the source's 95th percentile, `levelP95`, which D-382's silence guard uses for every window (D-404, D-407).

### 43.6 Transcription in the cloud, in checkpointed windows (v1.4)

```ts
// usecases/transcribe-source.ts (new); transcribe-cloud.ts, transcribe and run-pipeline.ts stay
export function transcribeSourceWindows(sourceId: SourceId, duration: DurMs,
                                        onWindow: (done: number, total: number) => void,
                                        isCancelled: () => boolean): Promise<SourceAsrOutcome>;
export type SourceAsrOutcome = { kind: "done" } | { kind: "paused"; reason: CloudPauseReason };
export type CloudPauseReason = "unavailable" | "busy" | "signed_out" | "offline";
```

- **Windows.** `CLOUD_WINDOW_S = 90` and **`CLOUD_OVERLAP_S = 10` (V6.1, as built: TE-18's first read found 3 of 31 seams losing or doubling a word with 5 s)**, so 45 minutes is 34 windows (`max(1, ⌈(D − 10) ÷ 80⌉)`, window `k` covering `[80k, min(80k + 90, D))` s). Each window is read from `pcm16.f32`, encoded by `encodeForCloud` (Opus at 32 kbps, about 0.32 MB) and sent as one `POST /transcribe` marked as a window of a source (§22.1). Its words pass `fromCloud` (D-382's silence guard, then `postProcess`) and are shifted to source time. Merging uses the existing `mergeWindows`.
- At most `CLOUD_LONG_CONCURRENCY = 2` windows are in flight (assumption, TE-18).
- **Checkpoints.** Each window's words are written to `sourceTranscripts` as it returns. A resume sends only the missing windows; this needs `pcm16.f32`, not the file. The merge runs when all exist.
- **No device fallback on the long path** (founder, 2026-11-08). A failed window is retried `CLOUD_WINDOW_RETRIES = 2` times, after 1 s and 4 s (assumptions). Then the job pauses (§43.8) with its reason: the switch off or `/healthz` saying no gives `unavailable`; 503 from the audio budget, or 429, gives `busy`; 401 gives `signed_out`; a network failure gives `offline`. The page says why and offers Continue. Clips keep D-348's fallback to the device, unchanged.
- **Before the work.** The long page needs the flag `cloud_transcribe`, `/healthz` saying yes, and a session. It checks all three at the gate (PS L1) and says which is missing before anything is read.
- `normalize_transcript` runs once on the merged words and gives the **master transcript**: `Transcript` with `SourceMs` numerically in its `TimeMs` fields. It is never given to the scene (§44.4).
- **Sentence quality.** The Director selects by sentence (§45), so `sentences.rs` now carries weight it did not have. V6.1 adds `sentences_long.rs` over the long corpus: no sentence longer than `MAX_SENTENCE_MS = 25 s` (assumption, E-12), split at the longest pause inside otherwise.

**(V6.1, as built.)** `transcribeSourceWindows({ sourceId, durationMs, levelP95, signal, onWindow })` gives `{ done, transcript } | { paused: reason } | { rejected: "NoSpeech" } | { failed: "encoder" } | { cancelled }`. Each window: `readPcm16Window` (main thread), `encodeForCloud` (unchanged), `transcribeAudio(ogg, signal, { index, total })` with `X-Offcut-Window` (a clip's request is byte-identical to before), then `asr.worker.cloudWindowWords` (D-382's guard against the source's `levelP95`; the words stay raw and relative to the window), kept in `sourceTranscripts`. When all are kept, `asr.worker.buildMasterTranscript` runs `mergeWindows`, `postProcess` and `normalizeTranscript`; fewer than `MIN_WORDS` is `REJECT_NO_SPEECH`. The asr worker takes one job at a time, so the two lanes' calls to it wait their turn in `transcribe-source.ts`; the requests stay two in flight. Failures (D-410): a timeout, `offline` or other failure is retried twice (1 s, 4 s), then pauses `offline` (when offline) or `busy`; 429 pauses `busy` at once; 503 asks `/healthz` again, and a body without `cloud_transcribe: true` pauses `unavailable`, otherwise it is retried, then `busy`; 401 after `authed()`'s refresh pauses `signed_out`, and Continue opens the sign-in dialog first; a window step that throws while offline (the asr worker's script cannot be fetched) pauses `offline`. A fifth reason, `wake_lost`, is the job's own (§43.7). `MAX_SENTENCE_MS` is not a rule of the code: `sentences_long.rs` holds it over its corpus, and the rule waits for the release that may reopen `sentences.rs`.

**Time budget.** The cloud read about 13× faster than the device on a minute of speech (TE-C4). For 45 minutes at a concurrency of 2 the estimate is a few minutes, plus the audio step of §43.5; TE-18 reads both. v1.3's two-pass seam (a faster model over the whole file) is not built: it existed only for device transcription of a source.

### 43.7 Staying alive

| Threat | Answer | Test |
|---|---|---|
| The screen sleeps | `navigator.wakeLock.request("screen")` while a source job runs; re-acquired on `visibilitychange`; released at end, on pause and on cancel | `long-resume.spec.ts` |
| The laptop sleeps or the tab is discarded | Checkpoints: at most the windows in flight and one short's preparation are lost | `long-resume.spec.ts`, kills the tab mid-run |
| The tab is hidden | Workers keep running under Chrome's rules; TE-20 reads throttling. The copy advises "Keep this tab open" | TE-20 |
| A worker or GPU device dies | `pool.ts` restart budget (§14.3) per window; `E_ASR_DEVICE_LOST` falls back to `wasm` once (the face model of V6.5) | `failure-recovery.spec.ts` |
| Cloud transcription fails | The job pauses with its reason and Continue (§43.6); no window already done is sent again | `transcribe-source.test.ts`, `long-resume.spec.ts` |
| Reload mid-job | `start-app.ts` finds a `sources` row with `stage` below `shorts_ready` and shows "Continue your video". Transcription, the Director and preparing shorts continue at once; audio extraction, the coarse vision pass and the part copies need the file, so they ask "Drop the same file to continue", checked by `sourceHashPrefix` and `sourceBytes` | `long-resume.spec.ts` |
| A short's parts were evicted | The short's row stays; opening it asks for the same file, and only its parts are copied again | `long-resume.spec.ts` |

Progress also goes into `document.title` ("42% - Offcut"), so a hidden tab shows it. No notification permission is requested.

**(V6.1, as built.)** `platform/wake-lock.ts`: the lock is taken when the job starts, taken again when the tab is shown, and let go on pause, cancel, failure and end; a lock lost while visible that cannot be taken again pauses the job as `wake_lost` (D-418). There is no user Pause button in V6.1 (Cancel, and the automatic pauses). A reload finds the newest `sources` row: below `transcribed` it shows "Continue your video"; transcription goes on at once without the file; the audio step asks for the same file (size and hash prefix), and another is refused with "This is a different file" (D-419). TE-20 (experiments.md) reads a hidden and a frozen tab.

### 43.8 The source machine

`state/machines/source-machine.ts`, under the same `transition()` rule as the other four (INV-16):

```text
idle → probing → gated_signin? → extracting_audio → scanning(V6.5) → transcribing → directing(V6.4)
     → copying_parts(V6.4) → preparing_shorts → shorts_ready
any working state → paused (wake lost, user pause, cloud paused §43.6) → the state it left
extracting_audio | scanning | copying_parts, after a reload → awaiting_file → the state it left, once the same file is dropped
any working state → cancelled | failed
```

The coarse vision pass runs while the file is surely in hand, before transcription. The clip machine is not changed. Opening a short loads it as a clip (§45.7).

**(V6.1, as built: D-417.)** V6.1's machine has the states it builds:

```text
idle → probing → gated → (Start) → extracting_audio → transcribing → transcribed        // V6.1's end; V6.4 leads on to directing
probing → rejected | failed | idle (a short file handed to /app)
extracting_audio | transcribing → paused → the state it left (Continue)
idle → awaiting_file (a reload during the audio step) → extracting_audio, once the same file is dropped
idle → transcribing | transcribed (a reload later on)
any working state → cancelled | failed; failed → awaiting_file | transcribing (Continue)
```

### 43.9 Errors and copy

`E_SOURCE_READ` (the file could not be read in place, for example because it was moved or deleted mid-job), `E_SOURCE_AUDIO`, `E_SOURCE_PART_COPY` (V6.4) and `E_SHORTS_QUOTA` join `ErrorCode` and `messages.ts`, each with a `.persistent` variant (`check-copy-codes.mjs`). The four pause reasons of §43.6 and `awaiting_file` have their own copy keys. Every string comes from `messages.ts` (INV-15).

**(V6.1, as built.)** `E_SOURCE_READ`, `E_SOURCE_AUDIO` and `REJECT_HDR`, the four pause reasons, `wake_lost`, `awaiting_file` and every word of the long page are in `copy/long.ts`, which `messages.ts` holds as `messages.long` (D-426); `check-copy-codes.mjs` reads both files. The two source errors are shown only on the long page's card, so they have no `.persistent` variant. `E_SOURCE_PART_COPY` and `E_SHORTS_QUOTA` come with V6.4.

**Acceptance (V6.1).** `route_probe.rs` (one case per rule from both pages, and rule 1's exact equality with `validate_probe` on every existing fixture); `demux_fixtures.rs` (hvcC); `audio_region.rs`; `transcribe-source.test.ts` (windows, checkpoints, each pause reason, no device fallback); `route-long.spec.ts` (both pages, the hand-over of a short file, the clip page's new action), with `fake-api.ts`'s cloud option of the cloud transcription release and its `windows` option (V6.1); `high-res.spec.ts`; `long-resume.spec.ts` (a kill during transcription resumes without the file; a kill during audio extraction asks for it); `flags-identity.spec.ts`. Peak memory read on a 45-minute 4K file. TE-17, TE-18, TE-20, TE-21 recorded.

---

## 44. The Assembly: One Short Is One Virtual Clip (V6.3)

**Model: a short is a virtual clip. An `Assembly` lists the parts of the source file that make it, in play order. The virtual clip has its own continuous `TimeMs` from 0, its own `pcm48`, its own transcript and its own plan. Everything downstream (detect, plan, cuts, `TimeMap`, `assemble`, scene, preview, export, cache) sees an ordinary clip. Only the two places that read the file ask the `Assembly` where a moment really is. Rejected: stretching `TimeMap` to reorder (its contract is ascending source time, `to_out` is single-valued, and every consumer relies on both), a new timeline type through the scene (a rewrite of proven code), rendering each part and concatenating files (a re-encode per part, no captions across joins).**

This is the design that keeps the founder's rule. Moving a hook to the front, which would have broken `TimeMap`, costs nothing here, because the virtual clip's time still only goes forward.

### 44.1 Three clocks

| Clock | Type | Meaning | Converted by |
|---|---|---|---|
| Source | `SourceMs` (new, `units.rs`) | A position in the long file | `Assembly` only |
| Recording | `TimeMs` (existing) | A position in the clip. For a dropped clip this is the file; for a short, the virtual clip | `Assembly`, `TimeMap` |
| Program | `OutMs` (existing) | A position in the export | `TimeMap` only |

For a dropped clip the `Assembly` is `Assembly::whole(duration)`: `SourceMs` and `TimeMs` are numerically equal, exactly as `TimeMs` and `OutMs` are under the identity map. **There is no `From` between any two.**

### 44.2 Type

```rust
// offcut-types/src/assembly.rs  [ONLY]
pub struct SourceSpan { pub start: SourceMs, pub end: SourceMs }          // non-empty
pub struct Part { pub src: SourceSpan, pub role: PartRole }
pub enum PartRole { Whole, Hook, Setup, Body, Payoff, Cta }               // for the review list and the LLM's reasons
pub struct Assembly { parts: Vec<Part> }                                  // private
pub struct SourcePos { pub at: SourceMs, pub part: u32 }

impl Assembly {
    pub fn whole(duration: DurMs) -> Assembly;
    pub fn from_parts(parts: Vec<Part>, source_duration: DurMs) -> Result<Assembly, AssemblyError>;
    pub fn is_whole(&self) -> bool;
    pub fn duration(&self) -> DurMs;                                      // the sum of the parts
    pub fn to_source(&self, t: TimeMs) -> SourcePos;                      // clamps to the last part's end
    pub fn to_recording(&self, s: SourceMs) -> Option<TimeMs>;            // None outside every part
    pub fn joins(&self) -> Vec<TimeMs>;                                   // the time of each boundary, ascending
    pub fn parts(&self) -> &[Part];
}
```

`from_parts` refuses: no parts; an empty part; a part outside the source; two overlapping parts in any order; more than `MAX_PARTS_PER_SHORT` parts; a sum over `MAX_SHORT_DURATION`. `Serialize` writes the parts; `Deserialize` runs the same checks. **Order is free; overlap is not**, so `to_recording` stays single-valued.

### 44.3 What reads it

Exactly four places, in a test-enforced list (`assembly_readers.rs`, a grep test):

| Reader | Change | Identity |
|---|---|---|
| `workers/render/video-source.ts` (through `source-position.ts`) | `frameAt(t: TimeMs)` asks `to_source(t)`, then `SourceMedia` for the part file and the time inside it (§44.8, v1.4) | `Assembly::whole` returns the same number; `SourceMedia::whole` is the clip's own file |
| `workers/render/export-loop.ts` | `recorded(pos)` then `to_source`; the frame index comes from the part file's own time (§44.8) | Same |
| `usecases/prepare-short.ts` | Decodes the regions from the part files into `pcm48` | Not used for dropped clips |
| `crates/offcut-wasm-render/src/scene_api.rs` | Hands `assembly.joins()` to the scene as `join_points` | Empty for `whole` |

**Decoding across a join.** A jump needs a decoder restart at a keyframe. Export is already blocking and needs nothing new. **Preview must not stall:** `video-source.ts` keeps a second decode session (`#next`, at most two alive) that starts `JOIN_LEAD = 2 s` before a join at `keyframe_at_or_before(next part's start)`, and swaps at the join. At most two decoders exist at once per slot. Measured by `preview-sync.spec.ts`'s new join case: no gap above the late-frame budget across a join on the reference fixture, and TE-22 on 4K with long GOPs. **Learned from Tooscut's `frame-renderer.worker.ts`:** request frames per source in ascending time and keep one iterator per contiguous run (§53.4).

### 44.4 Rebased transcript

```rust
// offcut-text/src/rebase.rs  [ONLY]
pub fn rebase_transcript(master: &Transcript, assembly: &Assembly, ranges: &[SentenceRange]) -> Transcript;
```

For each part in play order, takes its words, keeps their text and confidence, and shifts their times onto the virtual clip's clock. It re-runs `normalize_transcript`'s sentence and number passes on the result, so `numbers` and `sentences` are valid for the clip, and a number spoken across a boundary is not joined (a `join_points` check). The output obeys every invariant a `Transcript` has. `WordIdx` is the clip's own. `model_version` is copied. **So `detect`, `from_rules`, `/plan`, `validate`, `build_scene` and the review editor take it unchanged.**

### 44.5 Audio of a short

```rust
// offcut-dsp/src/regions.rs  [ONLY]
pub fn concat_parts(chunks: &[&[f32]], rate: Hz, xfade: DurMs) -> Vec<f32>;
pub const JOIN_XFADE: DurMs = DurMs::new(30);      // (assumption, V6.3) equal-power
```

`prepare-short.ts`: for each part, `decodeAudioRegion` over the part's file (§44.8) into a 48 kHz buffer; `concat_parts`; pad or trim by at most one sample so `pcm48.len() == round(assembly.duration × 48)`. Then the existing stages run: `runChain` → `out48` (INV-10 holds on the virtual clip) → `measureProsody` → `silence_profile` → plan → `timemap::build` → `assemble` → `prog48`. One loudness target applies to the whole short, so parts recorded at different levels do not jump. **`assemble` is unchanged by this section.**

**Part edges.** A part extends `PART_LEAD = 120 ms` before its first word and `PART_TAIL = 200 ms` after its last word, never past the neighbouring sentence's word (assumptions, E-12). The `Cut` rules then treat lead-in and tail as usual.

### 44.6 Cuts and non-identity maps (the old V6.2 scope, unchanged)

`Cut` arms, `timemap::build` with silence snapping, `assemble` splices with cross-fades, jump-cut alternation, the cuts list with restore and the filler switch all ship in this release as §40.1, §40.6 and §40.10 specify, for dropped clips and for shorts alike. `set_scene` stops refusing a non-identity map (DC-5).

**Video at a join.** A short's `join_points` get the same jump-cut punch-in alternation as a `Cut` by default. V6.8 adds real transitions there.

### 44.7 Render cache and restore

The cache key's canonical JSON gains `assembly` and, for a short, `sourceHashPrefix` of the source instead of the clip's. With `Assembly::whole` the key's field is omitted, so a V5 key stays valid. `restore-clip.ts` loads a short by its `clips` row and opens its part files through `SourceMedia` (§44.8, v1.4; v1.3 opened a whole-source copy, which no longer exists).

### 44.8 Part copies: the media a short keeps (v1.4)

A short's parts are copied from the source into OPFS once the Director has chosen them (§43), so the short never needs the original file again.

```rust
// offcut-mp4/src/remux.rs  [ONLY]
pub fn remux_range(src: &mut dyn RandomAccess, probe: &ProbeInfo, span: SourceSpan, handle: DurMs,
                   out: &mut dyn std::io::Write) -> Result<PartFileInfo, RemuxError>;
pub struct PartFileInfo { pub covers: SourceSpan, pub starts_at: SourceMs, pub bytes: Bytes }

// offcut-types/src/assembly.rs
pub struct PartFile { pub covers: SourceSpan, pub starts_at: SourceMs, pub path: OpfsPath }
pub struct SourceMedia { files: Vec<PartFile> }                        // private
impl SourceMedia {
    pub fn whole(path: OpfsPath, duration: DurMs) -> SourceMedia;    // a dropped clip: its own file
    pub fn file_at(&self, s: SourceMs) -> (&PartFile, DurMs);          // the file, and the time inside it
}
```

- **What is copied.** Video: the encoded samples from the sync sample at or before `span.start − handle` to the first sample at or after `span.end + handle`, as they are, with the source's `avcC` or `hvcC`; nothing is re-encoded. Audio: the AAC samples over the same range plus `AAC_PREROLL`. The output is an ordinary MP4 with `moov` first, which `offcut-mp4`'s demuxer reads as it reads a clip, so `video-source.ts`, the export loop and `decodeAudioRegion` need no new parsing.
- `handle = PART_COPY_HANDLE = 1 s` (assumption, V6.3): room for `PART_LEAD`, `PART_TAIL`, V6.8's transition handles (`MAX_HANDLE = 500 ms`) and a small part edit, without the original file.
- `starts_at` is the source time of the file's first sample, so `file_at` turns a `SourceMs` into a time inside the file. `to_source` stays the only door from recording time to source time (INV-27): `SourceMedia` maps a source time to a file, never a recording time.
- Two parts of one short that overlap once their handles are added share one file. Parts of different shorts are copied separately (S-6 allows 20% overlap), so evicting one short never breaks another.
- **Who calls it.** `run-source-pipeline.ts`, in `copying_parts` (§43.8), for every validated short in list order, before `prepare-short.ts`. In V6.3, `stitched.spec.ts` copies its fixed parts of the 60 s fixture the same way, so the real path is tested before the Director exists.
- `remux_range` writes no `udta`, keeps zero creation times and copies no metadata (P-13).
- **Adding a part later** (V6.9 chat's `AddSentences`) needs media the short does not have: the request asks for the same file again, copies the new part, and continues (§49.2).

**Acceptance (V6.3).** `assembly.rs` (every refusal, round trip, property: `to_recording(to_source(t).at) == t`); `assembly_readers.rs`; `rebase.rs`; `regions.rs` (no click across a join; length exact); `remux.rs` (H.264 and HEVC ranges: starts on a sync sample, every sample of the range present, durations exact, the part file passes the clip probe; no metadata copied); `source_media.rs` (`whole` is the identity; `file_at` at every boundary); `stitched.spec.ts`: a fixed `Assembly` with three parts out of order over the 60 s fixture, copied by `remux_range` and exported, passes checks 1-11 with the expected duration, and every white marker frame comes from the source time it should; `cuts_applied.spec.ts`; all §41.1 proofs unchanged.

---

## 45. The Director: From a Long Video to a List of Shorts (V6.4)

**Model: one LLM request per source reads the master transcript by sentence and returns a `ShortsPlan`: which sentences make each short, in what order, and why. A validator checks it like a plan. Each accepted short becomes a virtual clip, then gets its own ordinary `/plan`. If the service fails, local rules produce a smaller list, so no job dead-ends. Rejected: one giant call that also writes every short's visuals (a word-level body is several times the route's limit, 400 kB or more for 45 minutes, and a failed part loses everything), word-level selection (allows the mid-sentence cuts users complain about in other tools), a virality score (unverifiable; `why` is shown instead).**

### 45.1 Types

```rust
// offcut-types/src/source.rs
pub struct ShortId(u32);                                     // index in the validated list
pub struct SentenceRange { pub start: SentenceIdx, pub end: SentenceIdx }  // half-open
pub enum ShortStyle { Hook, Story, Listicle, Explainer, Quote, Reaction, Trivia }
pub struct ShortDraft { pub title: Label, pub style: ShortStyle, pub parts: Vec<PartDraft>, pub why: Label }
pub struct PartDraft { pub sentences: SentenceRange, pub role: PartRole }
pub struct ShortsPlan { pub format: u16, pub source: PlanSource, pub shorts: Vec<ShortDraft> }   // SHORTS_FORMAT = 1
pub struct ShortSpec { pub id: ShortId, pub title: Label, pub style: ShortStyle, pub why: Label,
                       pub assembly: Assembly, pub ranges: Vec<SentenceRange> }
```

`source` is set by the server or the rules producer, never by the model, as for `EditPlan`.

### 45.2 Request

```rust
// offcut-api-types/src/ai.rs
pub struct ShortsRequest { pub duration: DurMs, pub sentences: Vec<ShortsSentence>, pub shots: Vec<ShotLine>,
                           pub memory: Option<CreatorMemorySummary>, pub want: u8 }
pub struct ShortsSentence { pub text: String, pub start: SourceMs, pub end: SourceMs,
                            pub energy_z: f32, pub pause_before: DurMs }
pub struct ShotLine { pub start: SentenceIdx, pub kind: ShotKind, pub face: FacePos }   // §46; empty until V6.5
```

Size (v1.4, 45 minutes): at 200 words a minute, 9,000 words, about 54 kB of text; about 700 sentences at about 80 B of JSON each beside their text, 56 kB; 225 shot lines at 16 B, 3.6 kB; the total is about 115 kB. **`POST /shorts` therefore takes up to `SHORTS_BODY_LIMIT = 160 kB`** (DC-15, the second exception to INV-4's 64 kB, JSON only); every other JSON route keeps 64 kB. **A body above the limit is trimmed by the client:** shot lines first, then `energy_z`; a test proves a 45-minute worst case fits.

Prosody here is per sentence (`energy_z` averaged), read cheaply from `pcm16`; the word-level pass stays per short.

### 45.3 Route

| Item | Value |
|---|---|
| Route | `POST /api/v1/shorts`, bearer **required**, 160 kB (DC-15; v1.3: 64 kB), 6/min per user |
| Response | `ShortsResponse { plan: ShortsPlan, long_videos_remaining: Option<u32> }`, at most 32 kB |
| Errors | `E_SHORTS_QUOTA` (429), `E_PLAN_UNAVAILABLE` (502/504, or at the spend cap §52), `E_PLAN_INVALID` (502 after one retry) |
| Timeout | `SHORTS_UPSTREAM_TIMEOUT = 60 s` (assumption, TE-15), client 75 s |
| Storage and logs | As §40.7: body never stored or logged; `log_redaction.rs` gains the route |
| Prompt | `offcut-plan::shorts_prompt` (feature `prompt`), snapshot-tested with `SHORTS_PROMPT_VERSION` |
| Retry of a source | A second call for the same `sourceHashPrefix` in a month does not use another grant (the client sends the prefix's hash; the server keeps `(user, month, prefix_hash)`), so a failed run is not charged twice |

**Why sign-in is required.** PS §10 keeps the AHA before sign-in for clips. A long video costs the user several minutes, is transcribed in the cloud (§43.6, which needs a session) and costs money per call, and a limit must be shown before the work (PS §8 principle 5). So the long path gates at the drop (`gated_signin`), not after the transcript. Anonymous users keep the whole clip path.

### 45.4 What the prompt asks for

Generated text, in `shorts_prompt.rs`, snapshot-tested. Its content rules:

1. Choose moments that stand alone for a viewer who never saw the video. A short opens with a hook in its first sentence, delivers one idea, and ends on a payoff. Do not end on a setup.
2. Prefer whole thoughts over a fixed length. Target 30-60 seconds; allow 15-90.
3. A short may join up to 4 runs of sentences from anywhere in the video when that makes one complete point: a claim from minute 3 with its proof from minute 8. Do not join sentences whose meaning changes when moved. Do not create a claim the speaker did not make.
4. A hook may be moved to the front only if it still means the same out of its place.
5. Vary the styles. Do not reuse a sentence across shorts unless asked.
6. Give each short a short title and a one-line reason a person can read.
7. Skip intros, sponsor reads, "like and subscribe", and sections that lean on on-screen visuals the words do not explain (the shot lines show these).

### 45.5 Validation

```rust
// offcut-plan/src/shorts.rs  [ONLY constructor of ShortSpec]
pub fn validate_shorts(plan: ShortsPlan, t: &Transcript, cfg: &ShortsConfig) -> Result<(Vec<ShortSpec>, Vec<DroppedShort>), ShortsRejected>;
```

| # | Rule | On failure |
|---|---|---|
| S-1 | `format == SHORTS_FORMAT` | Whole plan rejected; rules producer |
| S-2 | Every `SentenceRange` is inside the transcript and non-empty | Short dropped |
| S-3 | **Parts are made of whole sentences**, by type, so no cut can land mid-sentence | Structural |
| S-4 | `MIN_SHORT_DURATION <= total <= MAX_SHORT_DURATION`; at most `MAX_PARTS_PER_SHORT` | Short dropped |
| S-5 | Parts of one short do not overlap | Short dropped |
| S-6 | Two shorts share at most 20% of their duration (assumption, E-12) | The later is dropped |
| S-7 | A `Label` passes V-4: length limit, every digit run in it was spoken in that short's sentences | Label replaced by the short's first words; short kept |
| S-8 | A short has at least `MIN_WORDS_PER_SHORT = 25` words (assumption) | Short dropped |
| S-9 | At most `MAX_SHORTS_PER_SOURCE`, in the plan's order | Later dropped |
| S-10 | Fewer than `MIN_SHORTS_PER_SOURCE` survive | The rules producer tops up the list, marked as such |

`ShortSpec` has no public constructor besides this function (INV-25, `tests/ui.rs`).

### 45.6 The rules producer

`offcut-plan::shorts_from_rules(&Transcript, &[DetectedEvent], &[ShotLine]) -> ShortsPlan` gives the offline answer with the rules' honesty: single runs of sentences (no joins, no reorder) of 25-75 s that contain a detected list, a number or a from-to, or the highest mean energy, ranked, at most 5, labelled "Picked without AI". Deterministic and unit-tested. It is used when the request fails, is over the cap, or is refused. **A failed plan service never blocks a long video (INV-24 extended).** A signed-in Free user over the monthly grant is told before the job starts; the rules producer does not replace a spent grant silently.

### 45.7 From specs to ready shorts

```text
usecases/run-source-pipeline.ts
  probe → gate → pcm16 (from the File, in place) → [V6.5 coarse vision, from the File] → cloud windows (§43.6)
  → master transcript → request-shorts.ts → validate_shorts → remux_range for every short's parts (§44.8)
  → for each ShortSpec in order: prepare-short.ts
usecases/prepare-short.ts (one short)
  rebase_transcript → decodeAudioRegion×part files → concat_parts → runChain → measureProsody → silence_profile
  → detect → request-plan.ts (existing, per short, or rules) → validate → timemap::build → assemble → store
  → poster frame → state = ready
```

- Shorts prepare in order, and each appears in the list as it finishes, so the user can start reviewing the first one while the rest prepare.
- `audio.worker` runs one at a time. ASR is unloaded before any render session (INV-12). The list needs posters only; a render session opens when a short is opened.
- Preparing does not render or encode. **Only what the user keeps is encoded.**

### 45.8 Reviewing and exporting

| UI | Behaviour |
|---|---|
| `ui/pages/ShortsPage.tsx` | The list: poster, title, length, parts count, reason. Keep or delete; "Open" |
| Opening a short | Loads it as a clip into the existing editor (`clip-machine`, `EditorLayout`). Everything from V4-V5 works: words, toggles, style, undo, export. A "Back to all shorts" link is the only addition to that page |
| Parts list (in `CutsSection`) | Each part with its text and role; delete a part; restore a deleted one. The user cannot type a new range before V6.9 (chat), and adding one then asks for the original file (§44.8). Deleting a part rebuilds the virtual clip without re-requesting the plan: the plan's items outside it are dropped by V-3 |
| `usecases/export-shorts.ts` | "Export kept shorts": sequential, one `start-export` at a time, each with its own `ExportId`, each counted as one export. Free users see "3 exports left" before the click (INV-14). A failure skips to the next and reports at the end |
| Downloads | One file per short through the existing download path (`offcut-<yyyymmdd-hhmm>-<n>.mp4`). No zip at launch |

### 45.9 Analytics (enum and count properties only)

`long_source_accepted { duration_bucket, resolution_bucket, codec }`, `source_stage_timing { stage, ms_bucket }`, `director_result { source, outcome, shorts, dropped_by_reason, joined_shorts }`, `short_action { action: opened | kept | deleted | part_deleted | exported, style, parts_bucket }`. `export_done` gains `from_short: bool`. The E-12 and E-13 readings need these.

**Acceptance (V6.4).** `shorts.rs` (one case per rule), `shorts_prompt_snapshot.rs`, `shorts_rules.rs`, `shorts_route.rs`, `ai_quota.rs`, `run-source-pipeline.test.ts` (each failure kind ends in a usable list), `long-to-shorts.spec.ts`: a 12-minute fixture with a fake provider and the fake transcription goes to 5 exported shorts, each passing all 11 checks, and the shorts reopen after the file is gone (part copies, §44.8); a 45-minute worst-case request fits `SHORTS_BODY_LIMIT`; `no_media_routes.rs` with DC-15. E-13 instrumentation live.

---

## 46. Vision: Local Signals and Face-Tracked Framing (V6.5)

**Model: small local models give the LLM and the framing code facts as text and numbers. No frame leaves the device. Rejected: sending frames to the provider (the founder's decision of 2026-11-02; it changes the privacy promise, kept as a possible later opt-in only if E-13 shows clip choice needs it), a local caption model (a large download and unmeasured speed for little gain on talking-head footage), an audio speaker model (v1.5: a large download whose voices would still have to be matched to faces; the speaker is found by matching each face's mouth movement to the speech instead, §46.6; v1.3 rejected any speaker detection, "one speaker at launch", and the founder reversed that on 2026-11-09).**

### 46.1 Two passes

| Pass | When | Reads | Gives |
|---|---|---|---|
| **Coarse**, whole source | After the audio step and before transcription, while the dropped file is in hand (v1.4, §43.8); it reads the `File` in place | Keyframes only, at most 1 per 3 s (`COARSE_STEP`, assumption). A long GOP gives fewer | `ShotSummary`: shot boundaries, kind, face presence |
| **Fine**, per short | In `prepare-short.ts` | Decoded frames at `FACE_FPS = 5` (assumption, TE-23) over the short's part files (§44.8) | `FramePath` |

**Frame hand-off.** `media.worker` decodes and scales to at most 320 px on the long side, and transfers `ImageBitmap`s through the main hub to `asr.worker` (no worker-to-worker channel, §36).

**v1.5: the fine pass gives more from the same frames.** No extra decode: from the frames it already samples, the fine pass also gives face tracks with a mouth-activity series (§46.6) and, from V6.6a, per-shot colour statistics (§47.4): a 64-bin luma histogram, the mean chroma of near-neutral pixels and the mean luma of the face box. All are numbers that stay on the device.

### 46.2 Models

| Model | Purpose | Runtime | Size limit |
|---|---|---|---|
| Face detector (candidates decided by TE-23: YuNet or a BlazeFace-class ONNX export) | Face boxes | ONNX Runtime already shipped; `workers/asr/vision-runtime.ts` | At most 5 MB |

Shot boundaries and kinds need no model: a luma histogram difference for boundaries (`offcut-vision::shots`), and rules over face presence and edge density for `ShotKind`. **A licence check is part of TE-23;** the licence is recorded in `docs/` and in `model-manifest`. Downloads are verified as for the ASR model (INV-20) from the same asset host, so the host list does not change.

### 46.3 Types

```rust
// offcut-types/src/vision.rs
pub enum ShotKind { TalkingHead, ScreenWithFace, Screen, Footage }
pub enum FacePos { None, Left, Centre, Right }
pub struct Shot { pub start: SourceMs, pub end: SourceMs, pub kind: ShotKind, pub face: FacePos }
pub struct ShotSummary { pub shots: Vec<Shot> }                 // at most MAX_SHOTS = 400; merged beyond it

// offcut-types/src/frame_path.rs  [ONLY]
pub struct FrameKey { pub at: TimeMs, pub cx: f32 }             // centre of the crop, 0.0..=1.0 of source width
pub struct FramePath { keys: Vec<FrameKey> }                    // private; made by offcut-vision or FramePath::none()
impl FramePath { pub fn none() -> FramePath; pub fn is_none(&self) -> bool; pub fn cx_at(&self, t: TimeMs) -> Option<f32>; }
```

### 46.4 Tracking

`offcut-vision::tracking::build_path(faces: &[FaceSample], shots: &[Shot], joins: &[TimeMs]) -> FramePath`, pure, deterministic and native-tested:

- **Dead zone:** the crop holds still until the face leaves the middle `DEAD_ZONE = 30%` of the crop (assumption, E-2). A locked shot looks shot; a jittery one does not.
- **Smooth, then ease:** one-euro filter on the face centre; moves use `ease_in_out` over at least `MIN_PAN = 400 ms`.
- **Hard reset at a shot change or a join:** the crop jumps with the cut and does not pan across it.
- **Lost face:** hold for 2 s, then ease to the last confident position; a shot with no face holds the shot's centre.
- **Limits:** the crop stays inside the source; `cx` is clamped.

### 46.5 Scene and user control

`SceneInput` gains `frame_path: &FramePath`. `framing.rs`: with `is_none()`, V5's crop, which keeps every golden (DC-9). Otherwise `Video.src` centres at `cx_at(recording(t))` plus the user's `crop_offset` as a shift. Dragging still works, so a wrong track is fixed by hand. A portrait source takes no path. A `screen` or `footage` shot on landscape uses the shot's centre until V6.8's layouts.

**A 4K source helps here.** A vertical crop of 3840 × 2160 is 1215 px wide before scaling to 1080, so face-tracked 4K stays sharp; a 1080p crop is upscaled from 607 px. The review shows a one-time note: "Filmed in 4K? Use the 4K file for a sharper short."

**Acceptance (V6.5).** `shots.rs`, `tracking.rs` (table of paths: still face, walking face, cut, lost face), `vision_summary_prompt.rs` (the lines reach the prompt within budget), `framing.rs` (none equals V5), `vision-pass.spec.ts`; (v1.5) `speaker.rs` and `speaker-framing.spec.ts` (§46.6). Reads: coarse pass at most 0.15× the source's duration (about 7 minutes on a 45-minute file; v1.3 read 3 minutes on 20); fine pass under 20 s per 60 s short on D1 (assumptions, TE-23), with the speaker work inside it. TE-23 and TE-27 recorded.

### 46.6 Following the speaker (v1.5, flag `speaker_framing`; the founder's decision of 2026-11-09)

**Model: in a shot that shows two or more people, the crop goes to whoever is speaking, found by matching each face's mouth movement to the speech; when that is unclear, the frame shows both. Rejected: an audio speaker model (above), panning between speakers (it reads as a tracking error; a switch is a cut, as an editor would make it), a split screen all the time (it halves every face, and Submagic's outputs switch).**

**Where it applies.** Only to shots with at least two face tracks. A multi-camera source that already cuts to each person gives one face per shot, and §46.4 frames it as before. A shot of one person never runs any of this.

| Step | How (all values assumptions, TE-27) |
|---|---|
| Face tracks | Face boxes at `FACE_FPS` are joined into tracks by box overlap (IoU at least 0.3) from sample to sample; at most `MAX_FACE_TRACKS = 3` per shot, the largest kept |
| Mouth activity | Per track and sample: the mean absolute difference of the mouth region (the lower third of the box, aligned on the box) from the previous sample, divided by the same measure over the whole box, so head movement cancels out |
| Speech activity | The transcript's word spans in recording time, and the 10 ms RMS of `out48` already computed for silences |
| Who speaks | Over a sliding `SPEAKER_WINDOW = 1.5 s`, each track's score is the correlation of its mouth activity with the speech activity. The speaker is the best track when it beats the next by `SPEAKER_MARGIN`; otherwise the window is "unsure" |
| When it switches | Only at a sentence start or a pause of at least 300 ms, and only when the new speaker holds for at least `MIN_TURN = 1.5 s`. A switch is a hard cut to the new face, like a shot change: §46.4's reset applies, and the motion track restarts (§47.2) |
| Unsure or both | Two people speaking at once, or unsure for longer than `MIN_TURN`: a `Split` key. The two faces are stacked, each in a 1080 × 960 half (left person on top), with a 4 px divider in the theme's `panel` colour. Captions keep their placement |

```rust
// offcut-vision/src/speaker.rs  [ONLY]
pub struct FaceTrack { pub id: u8, pub samples: Vec<FaceSample>, pub mouth: Vec<f32> }
pub fn speaker_path(tracks: &[FaceTrack], speech: &SpeechActivity, sentences: &[Span], shots: &[Shot],
                    joins: &[TimeMs], choice: SpeakerChoice) -> FramePath;   // pure, deterministic
pub enum SpeakerChoice { Follow, Both, One { left: bool } }                 // EditState.speaker, #[serde(default)]: Follow
```

- **The person stays in control.** The Framing panel offers "Follow the speaker" (default), "Show both" (a split for every two-person shot) and "Show one person" (left or right). Dragging still shifts the crop. Nothing leaves the device; the Director's `ShotLine` gains `people: One | Two | More`, an enum (INV-28).
- **Rendering.** A `Split` key makes two `Video { Main }` commands from the same decoded frame, which is `composite.rs`'s second path (DC-8, DC-17). Every single-face list stays on the simple path.
- **Acceptance.** `speaker.rs` (a table: two faces taking turns; both speaking gives a split; a turn shorter than `MIN_TURN` does not switch; a head turning without speech does not switch; one face changes nothing); `speaker-framing.spec.ts` (a generated two-person fixture switches at the sentence the transcript gives, and "Show both" splits); `composite.rs` (simple path identical). **TE-27:** on 10 real two-person podcast shorts, the person speaking is on screen (alone or in a split) for at least 90% of speech time, and no switch lands inside a sentence. If it fails, ask (§41.4): the fallback is "Show both" as the default for two-person shots.

---

## 47. The Catalog at Launch and the Finished Look (V6.2, V6.6a, V6.6b, V6.7, V6.8, V6.11)

**v1.5: the quality bar is now written down (the founder's decisions of 2026-11-09).** Offcut survives only if its shorts match or beat Submagic's. The founder's benchmark (Submagic-made shorts read frame by frame on 2026-11-09, kept outside git) showed that the gap is not the captions but motion, sound, B-roll and following the speaker: something changes about every 2 s, every shot moves, whooshes and pops land on each visual entry, punch words jump in size, and the crop follows whoever talks. §47.1-§47.6 make each of these a rule rather than a model's choice, so the worst short is good, not only the best one; each has a switch in the Enhance panel (§47.5).

`CATALOG_VERSION` bumps with each release's kinds. Codes stay in catalog order (§40.2). Every kind has a draw function, at least one golden per style, and `catalog_complete.rs` proves it. `EventKind` in `events.rs` is the code's name for the §40.2 `ItemKind`.

| Release | Kinds | Notes |
|---|---|---|
| V4 | `NumberReveal`, `ListReveal`, `FromTo`, `KeywordPop` | Built |
| V6.2 | `Emphasis`, `IconPop`, `PunchIn`, `Emoji`, `HookTitle` (v1.5: moved from part E) | `DrawCmd::Image` first used here, for raster emoji and icons from a closed bundled set (licence recorded). `PunchIn` is the auto zoom (§47.2, v1.4). v1.5: `Emphasis.size: Punch`, `Emoji.place: Under`, the hook lockup and caption glow (§47.3) |
| V6.6a (v1.5) | No new kind | The picture grade (§47.4) is a property of the `Video` command, not a plan item |
| V6.6, part A | `ListBuilder`, `NumberMoment`, `QuoteCard`, `Versus` | The cards shorts use most, first |
| V6.6, part B | `DefinitionCard`, `StepTracker`, `Timeline`, `Checklist` | Kept by the founder |
| V6.6, part C | Kinetic `CaptionMotion` rows, per-glyph animation; (v1.5) display fonts, six presets, word pages (§47.3) | Data in `StyleSpec` plus `MotionPreset` arms. v1.4 candidates from the founder's references (§47.2): a typographic lockup (v1.5: built for the hook in V6.2), a mixed-font pair beside the head, and a glass treatment for part A's cards (v1.5: the blur arrives in V6.6a) |
| V6.6, part D | `ShapeFrame`, `Transition` (shape wipes at topic shifts) | First use of `Video.mask`; kept by the founder. §41.4's cost trigger applies here by name |
| V6.6, part E | `ProgressBar`, `EndCard` (v1.5: `HookTitle` moved to V6.2) | `EndCard` adds a `Hold` |
| V6.7 | `SoundCue { at: WordIdx, sound: SoundId }` | v1.5: `SoundId` names a closed set of sounds synthesized in code (§47.1), so no licence is needed. Most cues are placed by rules, not by the plan |
| V6.8 | `Broll { anchors, source: BrollSource, fit: Cover \| Contain \| Pip, keep_audio: bool }` with `BrollSource = Asset(AssetRef) \| Stock { query: Label }` (v1.4; a `Stock` source is resolved to an asset before validation, §48.6), `JoinTransition { after_part, kind: Dissolve \| Whip \| SlideUp \| ZoomIn \| ZoomOut }` (v1.5: the zooms), `Layout { from_word, kind: FollowFace \| ScreenAndFace \| Full }` | Non-text assets only from the asset box |
| V6.11 | `TextBehind { anchors, text: Label }`, `Background { from_word, background: BackgroundRef }` | Gated by TE-25 |
| Post-launch | `ChapterMarker`, `LowerThird` | PS §21 item 15 |

**Limits per short** (`max_per_clip`, assumptions, E-11): `HookTitle` 1, `EndCard` 1, `Broll` 6, `SoundCue` 8, `TextBehind` 2, `Background` 1, `QuoteCard` 2, `Versus` 2. **Density rule (V-9, new, model plans only):** at most one visual item in any 2.5 s window, excluding emphasis in captions (assumption, E-11), so an AI plan cannot make a short look busy. Rules plans are exempt, which keeps their frames as they are (§41.1).

**Prompt additions (V6.2 onward).** The prompt tells the model what a finished short looks like, in generated text: open with the hook title (V6.2 D-503: always, with the hook; `validate` adds it when a model leaves it out); punch in on the sentence's key claim; never stack two visuals; end on the payoff, not a call to action, unless the speaker asks for one. `catalog_snapshot.rs` covers all of it.

### 47.1 Sound (V6.7)

Music, ducking and bed values are §40.6, unchanged. Sound cues are short sounds mixed in `assemble` after ducking at `SFX_LEVEL = -18 dB` (assumption, E-2). `assemble`'s `MusicInput` becomes part of a `MixInput { music, sounds, broll_audio }`, and `MixInput::none()` is the identity (`program_identity.rs` unchanged).

**Music (v1.5, founder 2026-11-09): the person's own track only.** No bundled library at launch; TE-16 is closed for music (PS §21 item 23 keeps a library as a later candidate). The plan's `MusicCue` still says whether music fits and its energy, and the review suggests adding a track when it does.

**Sound effects are synthesized (v1.5, founder 2026-11-09).** `offcut-dsp/src/sfx.rs` renders each `SoundId` from fixed parameters at 48 kHz: deterministic, no files, no download, no licence. TE-16 is closed for sound too.

| `SoundId` | How it is made (parameters are assumptions, tuned in the quality pass) | Used for |
|---|---|---|
| `Whoosh` | White noise through a band-pass whose centre sweeps 400 Hz → 4 kHz over 280 ms, a raised-cosine envelope, slight stereo widening | B-roll, a card or a lockup coming in; a join transition |
| `Swipe` | As `Whoosh`, 160 ms, sweep down | A card or B-roll leaving |
| `Pop` | A sine burst at 900 Hz falling to 300 Hz in 60 ms, with a 2 ms noise click | Emoji, icon pop, a punch word |
| `Click` | A 3 ms filtered noise transient | A list item, a step, a number counting |
| `Impact` | A sine at 60 Hz falling to 40 Hz over 250 ms plus a short noise burst, gently clipped | A snap zoom on the strongest word |
| `Riser` | Noise and a rising sine, 600 ms, ending on the next visual's entry | Before the hook title |

**Placed by rules.** `sfx::cues_for(entries: &[VisualEntry], cfg) -> Vec<Cue>` reads the scene's visual entries (B-roll, cards, lockups, emoji, icon pops, punch words, snap zooms, join transitions) and gives each its sound from the table. At most one cue per `SFX_MIN_GAP = 0.8 s` (the stronger entry wins) and at most `SFX_MAX = 12` per short (assumptions). The plan's `SoundCue` items are added on top, within the same limits. Rules cues are not plan items: they follow the entries, so switching an item off removes its sound, and the Enhance panel's "Sound effects" switch removes them all (§47.5).

**Voice tuning (v1.5, flag `voice_tune`, DC-19).** Two steps join "Voice cleanup", each set by a measurement so a good microphone gets little or nothing:

- **Denoise by need.** The noise floor is measured in the pauses (10th percentile of the 10 ms RMS); the denoiser's wet share runs from 0.25 at a floor below -65 dBFS to 1.0 above -50 dBFS (assumptions).
- **A three-band voice EQ.** A low shelf at 120 Hz, a peak at 250 Hz and a high shelf at 5 kHz, each set by comparing the voice's long-term spectrum with a reference voice curve and bounded to ±3 dB: muddiness is cut, and presence is lifted only if missing. Learned from Tooscut's `audio-engine` EQ (a low shelf, a peak and a high shelf as biquads); ours is set automatically, written in `offcut-dsp`, and runs offline in the chain before the loudness stage.
- **Rejected:** a noise gate (it chops breaths and room tone, and the cuts already shorten pauses), reverb, a de-esser (no measured need yet; the quality pass may add one).

### 47.2 Auto zoom and the quality bar (v1.4)

The founder's decision of 2026-11-08: shorts should move and look like those made by Submagic-class tools. The zoom is the plan's, through `PunchIn`, with no new render path: a `PunchIn` changes only the `Video` command's crop (TDR-21).

| `motion` | What the viewer sees | Timing (assumptions, E-11) |
|---|---|---|
| `Snap` | A cut-in to the zoom at the anchor word's start, held over the anchors | In within one frame; out over `ZOOM_RELEASE = 300 ms`, ease-out, after the anchors |
| `Push` | A slow push-in across the anchors | `ease_in_out` from 1.0 to the scale over the anchors; out as `Snap` |

- **Scales:** `Mild = 1.12`, `Strong = 1.25` (assumptions, E-11). With a `FramePath` (V6.5) the zoom centres on the face; without one, on the crop's centre. The crop never leaves the source (§46.4's clamp).
- **Rules (V-3):** two `PunchIn`s never overlap; a `PunchIn` never overlaps a cut's jump zoom (§44.6), which wins; at most one `PunchIn` per sentence (assumption, E-11). V-9's density rule does not count `PunchIn`, which adds nothing to the frame.
- **Prompt addition:** zoom on the claim, the number or the punchline, never on filler; `Snap` on one strong word, `Push` across a sentence that builds.

**Motion by rules (v1.5, flag `motion`, founder 2026-11-09).** The plan's zooms are moments; the benchmark's shots are never still. So every talking-head shot also drifts slowly, by rules, whatever the plan says:

```rust
// offcut-scene/src/motion.rs  [ONLY]
pub struct MotionTrack { segs: Vec<MotionSeg> }          // private; made by build() or MotionTrack::none()
pub struct MotionSeg { pub start: OutMs, pub end: OutMs, pub from: f32, pub to: f32 }
pub fn build(changes: &[OutMs], shots: &[ShotSpan], sentences: &[Span]) -> MotionTrack;   // pure
impl MotionTrack { pub fn none() -> MotionTrack; pub fn scale_at(&self, t: OutMs) -> f32; }
```

- **Segments** start at every visual change: a shot change, a join, a cut, a speaker switch (§46.6), a B-roll entry or exit. A segment longer than `MOTION_MAX_SEG = 4 s` is split at the sentence start nearest its middle.
- **Each segment drifts** by `MOTION_DRIFT = 0.05` (from 1.00 to 1.05, or back), alternating in and out from one segment to the next, on the `Standard` easing curve below. A segment shorter than 0.8 s holds still (assumptions, tuned in the quality pass).
- **Where.** On `TalkingHead` and `ScreenWithFace` shots, and on a clip with no shot information. Never on `Screen` shots, where text must stay readable, and never under a full-frame card.
- **With the other zooms.** Scales multiply: motion × the plan's `PunchIn` × the cut's jump zoom (§44.6), capped at `MAX_TOTAL_ZOOM = 1.35`. The centre is the face (`FramePath`), else the crop's centre; §46.4's clamp keeps the crop inside the source.
- **B-roll** (V6.8) takes the same alternating drift, so a still stock shot moves too.
- **Switch.** The Enhance panel's "Motion" (§47.5). Off, or `Enhance::v5()`, gives `MotionTrack::none()` and V5's frames (DC-16).

**Easing curves (v1.5).** `offcut-scene/src/easing.rs`: `CubicBezier { x1, y1, x2, y2 }` evaluated in pure Rust (Newton's method with a bisection fallback), with named curves `Standard` (0.4, 0, 0.2, 1), `Out` (0, 0, 0.2, 1) and `InOut` (0.65, 0, 0.35, 1) (assumptions). Zooms, pans, the motion track and caption entries use them in place of the fixed `ease_in_out`; existing styles keep their curves, so goldens do not move. Learned from Tooscut's keyframe evaluator (bezier interpolation per keyframe); ours is a function of time inside `frame_at`, with no keyframe store.
- **The bar.** Quality is judged in the quality pass after V6.11 (`buildplan.md` §8A.12) against the founder's benchmark: Submagic-made shorts and caption references, kept outside git (the repository is public). The references point at three caption families that V6.6 part C weighs: **typographic lockups** (words of mixed size and weight stacked into one block, one accent colour, the camera pushing through the block as it grows); **mixed-font pairs beside the speaker's head** (a bold sans with an italic serif, which needs an OFL serif italic in the font registry and the face position of V6.5); and **glass cards** with the video blurred behind them (§48.4's reduced-resolution blur, brought forward to V6.6 if kept). Each is a §41.4 cost check by name, as part D is.

**(V6.2, as built: D-476, D-477, D-497.)**
- **The drift is `offcut-scene/src/camera.rs`** (`motion.rs` is V4's caption motion): `CameraTrack`, built from the clip's start and the hook's end. Segments over 4 s are split at the sentence start nearest their middle, those under 0.8 s are held; the drift alternates 1.00 to 1.05 and back on `STANDARD`, held across short segments. TE-31: 0.019 px of the steepest frame-to-frame change, no stepping.
- **One zoom for the frame:** `video::Track`'s scale is the drift times the strongest `PunchIn` at that time, never a product of zooms, capped at `MAX_TOTAL_ZOOM = 1.35`; `Track::fixed` gives V5's `src` bit for bit.
- **Motion off** gives the fixed track, and the plan's zooms stay. `motion` stays off in production until V6.5 brings shots (D-497).

### 47.3 Captions (v1.5: V6.2 and V6.6b)

The benchmark's captions are plainer than expected: one word, or two to four, in a heavy white sans with a soft glow; the keyword coloured, glowing, with an emoji under it; punch words about twice the size in capitals; hard swaps rather than bounces. Bold (Inter 800, yellow keyword) is already near that; these close the rest.

| Part | Release | Contract |
|---|---|---|
| **Glow** | V6.2 | `StyleSpec.glow: Option<Glow { role: ThemeRole, layers: u8, spread: f32, alpha: f32 }>`. Drawn as `layers` (4) strokes behind the glyph run, each wider by `spread` and fainter, in Vello: no blur and no second path. Clean, Bold and Mono keep `glow: None`, so their goldens do not move |
| **Punch words** | V6.2 | `Emphasis { size: Punch }` sets the word alone on its page in capitals at `PUNCH_SCALE = 1.9` of the style's size (assumption). At most one per 6 s (its `min_gap`). The model picks it in the plan; the rules producer does not |
| **Emoji under the keyword** | V6.2 | `Emoji { place: Under }` draws the emoji below the caption block, centred on its word, at 1.6 × the cap height, with a `Pop` entry (scale 0.6 → 1.0 over 120 ms). Large emoji also stand in for the benchmark's object stickers |
| **Hook title** | V6.2 | `HookTitle` draws `extras.hook` over the first `HOOK_DUR = 2.0 s` as a lockup (`offcut-scene/src/lockup.rs`): two or three lines of mixed size that fill the width, the `accent` word at 2× in the accent colour, entering with a scale of 0.9 → 1.0 over 150 ms. Captions are hidden under it. With no hook the plan has none; a model's valid hook is always shown, `validate` adding the item when the model leaves it out (V6.2 D-503) |
| **Word pages** | V6.6b | `StyleSpec.paging: Lines(u8) \| Words { max: u8 }` (`#[serde(default)]`: the style's lines). `Words { max: 1 }` and `Words { max: 3 }` break at punctuation and at pauses of at least 250 ms and keep §17's reading floor; words swap hard, with no transition unless the preset names one |
| **Display fonts** | V6.6b | Four OFL fonts join the registry, subset to Latin (at most 400 kB together, assumption): a heavy geometric sans (Montserrat Black class), a condensed (Anton or Bebas Neue class), a comic display (Bangers class), and an italic serif for mixed pairs. TE-28 picks each and records its licence |
| **Presets** | V6.6b | Six new presets, beside Clean, Bold and Mono: rows of `StyleSpec` data on the four fonts, glow, paging and the kinetic rows of part C. **Named by us,** never after a creator. The benchmark sets which is the default for new clips (the quality pass) |

**(V6.2, as built: D-480 to D-484, D-502, D-503.)**
- **Glow:** `StyleId::Glow` is a fourth style, appended; the default stays Clean. It is Inter 800 white with a glow of 4 layers, 6 lp spread and 0.16 alpha, each word glowing in its own colour, the keyword in the accent.
- **Emphasis is drawn by the captions:**
  - `Colour` fills the word in the accent;
  - `Underline` is a bar 0.10 em under the baseline, 0.08 em thick;
  - `Highlight` is a pill (`OnAccent` word on the accent).
- **Punch words:** a chunk of their own, in capitals at `PUNCH_SCALE = 1.9`.
- **Emoji belong to the captions:** placed by their chunk, drawn after them, `Under` (over the lines at Bottom) or `Beside` when the line leaves room.
- **Icons belong to the event zone:** an accent disc with a `Panel` disc inside and a white icon (D-502). The images are 160 px, embedded like the fonts (`assets/images/`, about 1 MB), drawn in Vello with straight alpha; TE-30's SSIM is 0.9995 on D1 and at least 0.9996 on CI's lavapipe.
- **The hook (`lockup.rs`):** each line takes its own size, from 140 lp down to 48 in steps of 4, the accent word at twice its line's, over `[0, min(2 s, length))`. No caption is drawn or hit under it, and every model plan with a valid hook shows one (D-503).

### 47.4 Picture: the blur, colour and light (v1.5, V6.6a, flag `picture`)

**Model: each shot gets one automatic grade, computed from the frames the fine pass already samples and applied by the video pass. A grade only corrects what was measured, within bounds, so a well-lit, well-balanced shot gets the identity grade. Rejected: looks and LUTs at launch (PS §21 item 16), log colour spaces, a colour UI of wheels, curves or HSL (PS §8 principle 3), HDR tone mapping (the founder's decision of 2026-11-09: HDR stays rejected with advice, `REJECT_HDR`; TE-21 is closed).**

**The blur.** `offcut-render/src/blur.rs`: one pass at reduced resolution (downsample to a quarter, a dual-filter blur of two or three levels, upsample), never a per-pixel 169-sample loop (§53.4). Its users: the blurred fill of the `Full` layout and of contained B-roll (V6.8), the glass treatment of cards (V6.6b), `Whip` (V6.8) and the blurred background of V6.11. A list that uses it takes `composite.rs`'s second path (DC-8).

```rust
// offcut-types/src/grade.rs
pub struct Grade { pub exposure_ev: f32, pub temp: f32, pub tint: f32, pub contrast: f32, pub saturation: f32,
                   pub lift: [f32; 3], pub gamma: [f32; 3], pub gain: [f32; 3],
                   pub face_light: Option<FaceLight>, pub vignette: f32 }
pub struct FaceLight { pub cx: f32, pub cy: f32, pub radius: f32, pub gain_ev: f32 }   // in output space
// offcut-vision/src/grade.rs  [ONLY]
pub fn shot_grades(stats: &[ShotColour], reference: usize, faces: &FramePath) -> Vec<Option<Grade>>;   // pure
```

| Step | Rule (bounds are assumptions, TE-29) |
|---|---|
| Exposure | Brings the face box's mean luma (else the frame's median) toward `FACE_LUMA = 0.48`, within ±0.5 EV |
| White balance | Moves the mean chroma of near-neutral pixels toward neutral, at most `WB_MAX` in temperature and tint; skin is never the reference |
| Contrast and saturation | A gentle S-curve when the histogram is flat (at most +8%); saturation +5%, weighted to low-saturation pixels so skin does not go orange |
| Camera matching | In a short whose shots come from different cameras (cut boundaries plus a distance between the shots' colour statistics), each shot's lift, gamma and gain move its means and spread toward the reference shot, the one with the most speech time; bounded like the rest |
| Face light | A soft radial lift on the face from `FramePath`, at most +0.3 EV, falling off over 1.5 face widths |
| Vignette | At most 12% darker at the corners |
| Do no harm | A step below its threshold is zero; a shot with every step zero gets `None` and stays on the old pipeline. A grade changes only at a shot boundary, with the cut, and never drifts within a shot |

- **Drawing (DC-18).** The video pass gains a graded variant: the same draw, one uniform per frame holding the shot's `Grade`. It is still one pass, so a graded list stays simple. Budget: at most 1 ms more per frame at 1080 × 1920 on D1 (TE-29).
- **Learned from Tooscut's colour grading** (`color_grading_uniforms.rs`: lift, gamma and gain wheels, slope, offset and power): the grading model. Ours fills the values automatically from measurements, has no user controls, and is written in Offcut's types.
- **Switch.** The Enhance panel's "Colour & light" (§47.5). When every shot is `None`, the panel says the picture already looks right.

### 47.5 The Enhance panel: every enhancement measured, and each one switchable (v1.5, from V6.2)

The founder's request of 2026-11-09: a creator with a good microphone or good lighting can switch off what they do not need, while reviewing. Each enhancement first measures the need and does only that much; the switch removes it completely.

```rust
// offcut-types/src/enhance.rs
pub struct Enhance { pub voice: VoiceMode, pub motion: bool, pub picture: bool, pub sfx: bool }   // in EditState, #[serde(default)]: v5()
pub enum VoiceMode { Clean, Level }        // Level: loudness normalisation and the limiter only
impl Enhance {
    pub fn v5() -> Enhance;                // Clean with V5's chain, motion, picture and sfx off: V5's frames and samples
    pub fn defaults(flags: &FeatureFlags) -> Enhance;   // every shipped enhancement on
}
pub struct EnhanceReport { pub noise_floor: DbBucket, pub voice_tuned: bool, pub shots_graded: u16, pub shots: u16 }
```

| Row in the panel | Release | Off means | What it says when there is little to do |
|---|---|---|---|
| Voice cleanup | V6.2 (V6.7 adds the tuning) | `VoiceMode::Level`: the original sound, levelled to -14 LUFS and limited, with no denoise, high-pass, compression or EQ | "Your audio is already clean: light touch" |
| Motion | V6.2 | `MotionTrack::none()`; the plan's own zooms stay, as items | |
| Colour & light | V6.6a | Every `grade` is `None` | "Your picture already looks right" |
| Sound effects | V6.7 | No rules cues; the plan's `SoundCue` items stay, as items | |
| Music | V6.7 | `MusicChoice` none (the existing control, shown in the panel) | |

- **Instant in the preview.** Both voices are made when a clip is prepared: `out48` (Clean) and `lvl48` (Level, the cheap path), side by side in OPFS. Switching voice re-runs `assemble` only; motion, colour and sound go through the existing `review-update` path. Every switch is one undo step within the 150 ms edit budget.
- **Remembered.** The panel's last state is stored in this browser (IndexedDB `meta`, key `enhanceDefault`) and becomes the start of the next clip; settings has "Back to recommended". Nothing is sent.
- **Kept honest.** The export reads `EditState.enhance`, which joins the render-cache key (S-9) as part of `EditState`. `export_done` gains `enhance_off`, a list of the rows switched off (an enum list, INV-18), for E-11.

**(V6.2, as built: D-485 to D-489, D-505.)**
- **The type:** `Enhance { voice: VoiceMode (Clean | Level), motion: bool }`; `picture` and `sfx` come with V6.6a and V6.7. There is no `EnhanceReport` yet (D-489).
- **Voice cleanup off:** `offcut-dsp/src/level.rs`' `level_only`, loudness and the limiter only, writes `lvl48.f32` beside `out48.f32` when the build has `motion`. `run_chain` and `program_identity.rs` are unchanged.
- **A switch is one undo step** through `review-update.ts`, measured at 11 to 20 ms (Motion) and 32 to 139 ms (Voice) against the 150 ms budget (P208 to P211a).
- **Memory:** a new clip starts with the switches as last set (`meta.enhanceDefault`); settings' "Back to recommended" forgets them. The panel sits under the Style tab's cards (the founder's choice).
- **The summary:** "What we changed" says "Voice cleaned" only with Voice cleanup on (D-505).

### 47.6 The quality check (v1.5, V6.8)

**Model: the benchmark becomes a check that every prepared short runs before the person sees it. Rules fix what they can; nothing blocks an export; CI runs the same check over the corpus so a release that makes shorts worse fails.**

`offcut-scene/src/quality.rs`: `check(scene: &Scene, speech: &SpeechActivity) -> QualityReport`, pure.

| # | Rule (values are assumptions, tuned in the quality pass) | Fixed by rules in `prepare-short.ts` |
|---|---|---|
| Q-1 | Something changes at least every `Q_MAX_STILL = 2.5 s`: a shot or join, a cut, an item's entry, B-roll, a speaker switch, a motion segment's start | A `PunchIn { Snap }` on the strongest word of the stretch by prosody, else an `Emoji` from the rules set, within V-9's density |
| Q-2 | No talking-head stretch longer than 3 s without motion | Covered by §47.2 while Motion is on; reported when it is off |
| Q-3 | Every visual entry has a sound while Sound effects is on | `sfx::cues_for` already does; the check proves it |
| Q-4 | In a two-person shot, the person speaking is on screen for at least 90% of speech time | Reported; the person can choose "Show both" |
| Q-5 | No caption, card or item in the platforms' UI zones | Guaranteed by `safe_area.rs`; the check proves it |

- **Fixes are ordinary items.** A rules fix is written as `PlanOp::AddItem` with `origin: QualityFix` in `EditState.plan_ops` (V6.8 builds `AddItem` beside `PlaceAsset` and `RemoveAsset`, §49.2), so it shows in the review, can be switched off and undoes like any item, and passes `validate`.
- **CI.** `quality_corpus.rs` prepares the corpus's reference shorts (rules plans, and fixed model plans stored as fixtures) and fails if any rule's failure count rises above the last release's recorded count.: B-roll, Asset Box, Transitions, Split Layout (V6.8)

**Model: a frame's display list may hold several `Video` commands from several sources, drawn in list order with Vello runs between them. Assets are the person's own files stored locally and labelled by them, or (v1.4) stock clips from Pixabay downloaded into the same local store. The plan places them by pointing at words; the person can place one by hand on a word (v1.4), and from V6.9 chat can. Rejected: a timeline for placing them (PS §8 principle 3), a second compositor (one list, one renderer), other stock providers at launch (read 2026-10-29: Pexels had paused new keys, Coverr's free API is non-commercial, Mixkit has no API), generated B-roll. v1.3 rejected stock too (the founder's decision of 2026-11-02); the founder reversed that on 2026-11-08, and §48.6 answers its three reasons: the licence, a server search route, a new browser host.**

### 48.1 Slots and sources

```rust
// offcut-scene: VideoSlot (§40.5 reserved this)
pub enum VideoSlot { Main, Asset(AssetSlot), Matte }      // Matte: V6.11
pub struct AssetSlot(u8);                                  // at most MAX_ASSET_SLOTS = 1 live per frame
// offcut-render
pub struct VideoSources<'a> { pub main: (&'a VideoFrame, Rotation), pub asset: Option<(&'a VideoFrame, Rotation)>,
                              pub matte: Option<&'a VideoFrame> }
```

- `Scene::source_at(t)` keeps its signature for `Main`. New `Scene::asset_at(t) -> Option<(AssetRef, AssetTime)>` tells the loops what else to fetch.
- **At most three decoders run at once** (`Main`, one asset, one matte), plus a join's lead decoder on `Main`. An asset that starts soon gets its decoder `JOIN_LEAD` ahead. Tested at the budget on D1, then R1 in V9.
- **Learned from Tooscut's `RenderFrameTask.textureRequests`:** each frame states which sources and times it needs; the worker groups them by source. Ours sits behind `Scene::asset_at`, and the frames stay `VideoFrame`s (§53.4).

### 48.2 Compositor

`composite.rs` gets two paths:

```text
is_simple_list(list)      // the D-133 rule: one Video, first, Main, opaque, no mask, no Image
  → the existing path, byte for byte (DC-8)
otherwise
  → walk the list: a run of Vello commands renders into a texture (reused; at most RUNS_PER_FRAME = 4, assumption);
    each Video draws with its dst, opacity and mask; results composite in order
```

Masks (V6.6 part D) and mattes (V6.11) use the same path. `golden_frames.rs` adds cases; existing ones stay untouched. The heaviest list must hold the `frame_at` and render budgets on D1, or the kind is dropped through §41.4.

### 48.3 Transitions at joins

`JoinTransition` overlaps the two sides around `join_points[n]`, half its length on each side, using a `boundary` and an outgoing and incoming role. **Learned from Tooscut's `CrossTransitionRef`.** Because the virtual clip has no frames past a part's end, `prepare-short.ts` extends the `Assembly` with **handles**:

```rust
pub struct Part { pub src: SourceSpan, pub role: PartRole, pub handle: DurMs }   // #[serde(default)]: zero
```

A handle is frames available past `src.end`, at most `MAX_HANDLE = 500 ms`, read only for a transition. The audio keeps `JOIN_XFADE` regardless. Kinds: `Dissolve`, `Whip` (a blur and slide, with §47.4's blur), `SlideUp`, and (v1.5) `ZoomIn` and `ZoomOut` (the outgoing side scales up, or the incoming side scales down to 1.0, over the overlap; learned from Tooscut's `ZoomIn`/`ZoomOut` transition types). Each join transition gets a `Whoosh` while Sound effects is on (§47.1). **Text and captions never slide with the video**; they are laid out in program time, above it. Tooscut's transitions move only media, and for us that is correct.

### 48.4 Layouts

`Layout` (see §47) changes `Video` commands, not modes: `ScreenAndFace` puts two `Video { Main }` commands in the list with different `src` and `dst`, from the same decoded frame; the top shows the screen region, the bottom the face crop. The regions come from `Shot` and `FramePath`. `Full` shows the whole landscape frame fitted over a blurred copy of itself, for footage shots. The blur is §47.4's pass at reduced resolution (v1.5: built in V6.6a), never a per-pixel 169-sample loop. (v1.5) The two-face split of §46.6 already puts two `Video { Main }` commands in one list; `ScreenAndFace` reuses that path.

### 48.5 Asset box

```rust
// offcut-types/src/assets.rs
pub struct AssetId(Uuid);
pub enum AssetKind { Video, Image }
pub struct AssetMeta { pub id: AssetId, pub kind: AssetKind, pub label: Label, pub note: Option<Label>,
                       pub duration: Option<DurMs>, pub width: Px, pub height: Px, pub has_audio: bool, pub added_at: UnixSecs,
                       pub origin: AssetOrigin }                     // v1.4; #[serde(default)]: Own
pub enum AssetOrigin { Own, Pixabay { id: u64 } }
pub struct AssetRef { pub id: AssetId, pub start_in_asset: DurMs }
```

| Rule | Value |
|---|---|
| Accepted | MP4, MOV with H.264 or HEVC; JPEG, PNG, WebP. At most 2 minutes, 2 GB, 3840 px (assumptions, E-2) |
| Storage | `assets/<assetId>/source` in OPFS and the `assets` store. `AssetMeta` has no file name (P-11). "Clear local data" removes them |
| Limits | At most 50 assets, 10 GB, per browser (assumptions) |
| Label | The user types a label ("my office", "product demo") and an optional note, at most `ASSET_NOTE_CHARS = 120`. **Label and note leave the device in plan requests while the AI plan is on, and the settings table says so.** The asset's pixels and audio never do |
| Placement by plan | The plan request lists `AssetLine { id, kind, label, note, duration_bucket }`, and `stock: bool` (§48.6). The model places `Broll` items pointing at words where they fit |
| Placement by the person (v1.4) | **By hand, from V6.8:** a word in the caption editor, then "Add B-roll here", then one of the person's assets or a stock search (§48.6) and, for a video, which part of it to use. Stored as `PlanOp::PlaceAsset` in `EditState.plan_ops` through `effective_plan` (§49.2; V6.8 builds the op log with `PlaceAsset` and `RemoveAsset` only), so it is one undo step and passes the same validator. From V6.9, also by chat ("use my office clip when I say headquarters"). The review still toggles what the plan placed |
| Choosing the part (v1.4) | For a video asset, a strip of its frames with a window as long as the anchored words; dragging the window sets `AssetRef.start_in_asset`. It is the one direct control besides the crop offset (PS §8 principle 3, v1.4), and it is not a timeline |
| Rights (v1.4) | The Terms of Service say the person must hold the rights to clips they add (for example a lecture they did not film). Stock clips carry Pixabay's licence (§48.6) |
| Audio | `keep_audio` is off by default; speech stays on top, ducked like music |
| Failure | A missing asset drops the item (V-2), and the review says so |

### 48.6 Stock B-roll from Pixabay (v1.4, flag `stock_broll`)

**The founder's decision of 2026-11-08:** stock is in, from Pixabay, beside the person's own clips. Pixabay's terms as read on 2026-10-29 (re-read before V6.8 is planned, TE-26): a free API key; 100 requests per 60 s per key; searches cached for 24 h; "show your users where the images and videos are from" wherever results are shown; videos may be embedded directly; no mass downloads; the content licence allows commercial use with no credit required, but no standalone redistribution and no competing stock service. `cdn.pixabay.com` sends `Access-Control-Allow-Origin: *`, so the browser can fetch from it under COEP.

```text
search    client → POST /api/v1/broll/search { query } → our server → Pixabay's video API
          ← { results: [{ id, duration, width, height, thumb_url, video_url }] }      (at most 12)
download  client → GET video_url on cdn.pixabay.com → assets/<assetId>/source, origin Pixabay { id }
```

| Item | Value |
|---|---|
| Route | `POST /api/v1/broll/search`, bearer required, JSON, 1 kB body, 20/min per user (assumption) |
| Query | A `Label` of at most 40 characters (V-4's digit rule applies), typed by the person or written by the plan |
| Server | `server/src/broll/`: a `StockProvider` trait and `PixabayProvider`; key `PIXABAY_API_KEY`; a global token bucket of 90 requests per 60 s, under Pixabay's 100 |
| Cache | 24 h in memory, as Pixabay requires, keyed by the SHA-256 of the normalised query. **The query text is neither stored nor logged** (P-17); a cache entry holds only Pixabay's ids, sizes and URLs |
| Rendition | The smallest rendition at least 1080 px on its short side, else the largest; landscape clips are cropped by `fit: Cover` |
| Download | From the browser straight to `cdn.pixabay.com` (§24.1, browser row 4): video bytes never pass through our server |
| Attribution | "From Pixabay" in the search panel and on each stock asset in the asset box; no credit inside the export (the licence asks for none) |
| Switch | Settings, "Stock B-roll (Pixabay)": on by default while the AI plan is on (assumption; the founder confirms at V6.8). Off: the plan request says `stock: false`, and the search box is hidden |
| Failure | Search down, over the limit or empty: a `Stock` item is dropped (V-2) and the review says so once; the person's own assets are unaffected |

**Placed by the plan.** The model may write `Broll { source: Stock { query } }` on words that name something visual when none of the person's assets fits. `usecases/resolve-stock.ts` runs after the plan returns and before `validate`: for each `Stock` item it searches, takes the first result long enough for the anchors, downloads it into the asset store and rewrites the item to `Asset(AssetRef)`. An item it cannot resolve is dropped. The renderer only ever sees local assets. **Placed by hand.** "Add B-roll here" offers the person's assets and a stock search box filled with the anchored words.

**Acceptance (V6.8).** `composite.rs` (simple path identical), `layers_goldens` (B-roll cover, contain, picture-in-picture, split, full), `handles.rs`, `join_transition.spec.ts`, `asset-box.spec.ts` (including placing by hand on a word, choosing the part, and undo), `plan_ops.rs` (`PlaceAsset` and `RemoveAsset`: apply, refuse, undo equals before), `broll_search_route.rs` (bearer, limits, the cache keyed by hash), `log_redaction.rs` (a marker word in a search query), `resolve-stock.test.ts`, `stock-broll.spec.ts` (a fake provider and a fake CDN; the switch off sends nothing), verifier 11/11 on an export with own and stock B-roll and transitions, memory read at three decoders (TE-14 re-read). TE-26 recorded.

---

## 49. Chat Editing (V6.9)

**Model: chat is a typed client over the plan. The model reads the request and returns a list of closed operations; the client validates and applies them locally; the preview updates at once. No operation contains a timestamp or a shown number. Rejected: the model returning a whole new plan (a small request could silently change everything), free-form scripts (unverifiable), a server-side edit state (P-1).**

### 49.1 Operations

```rust
// offcut-types/src/plan_ops.rs  [ONLY]
pub enum PlanOp {
    SetStyle { style: StyleId },
    SetItemEnabled { id: EventId, enabled: bool },
    RemoveItem { id: EventId },
    AddItem { item: PlanItem },                                // the same catalog rules as any plan item
    SetLabel { id: EventId, field: LabelField, text: Label },
    SetHook { text: Option<Label> },
    RestoreCut { key: CutKey },
    CutWords { range: WordRange },                             // user-asked: must be whole sentences or a filler run
    SetFillerCuts { on: bool },
    RemovePart { part: u32 },
    AddSentences { range: SentenceRange, position: u32 },       // from the master transcript; shorts only
    MovePart { from: u32, to: u32 },
    SetMusic { on: bool, energy: Option<MusicEnergy> },
    PlaceAsset { asset: AssetRef, words: WordRange, fit: BrollFit },   // v1.4: built in V6.8 for placing by hand (§48.5)
    RemoveAsset { id: EventId },
    SetLayout { from_word: WordIdx, kind: LayoutKind },
    SetCaptionPosition { snap: CaptionSnap },
    SetBrandKit { on: bool },
}
pub struct ChatReply { pub ops: Vec<PlanOp>, pub message: Label /* at most 200 chars */, pub needs_clarifying: bool }
```

`Label` here is V-4's type, so its digit rule applies: a digit the speaker did not say is refused.

### 49.2 Applying

```rust
// offcut-plan/src/plan_ops.rs  [ONLY]
pub fn effective_plan(base: &EditPlan, ops: &[PlanOp]) -> EditPlan;     // pure
pub fn validate_ops(ops: &[PlanOp], ctx: &OpContext) -> Vec<OpVerdict>;  // each op: applied or refused with a reason enum
```

- The stored plan never changes. `EditState` gains `plan_ops: Vec<PlanOp>` (`#[serde(default)]`), so the existing history (`history.ts`, 50 steps) gives chat undo at no extra cost: one chat message is one undo step. **v1.4:** V6.8 builds `plan_ops`, `effective_plan` and `validate_ops` with `PlaceAsset` and `RemoveAsset` only, for B-roll placed by hand (§48.5); V6.9 adds the other operations and the chat route. **v1.5:** V6.8 also builds `AddItem`, for the quality check's fixes (§47.6); `AddItem` gains `origin: OpOrigin { Person, Chat, QualityFix }` (`#[serde(default)]`: `Person`), an enum that analytics may count.
- `AddSentences` needs media the short does not hold: it asks for the original file, copies the new part (§44.8), then rebuilds.
- Everything downstream reads `validate(effective_plan(base, ops), ...)`. INV-21 holds: **chat cannot put anything on screen that the validator has not returned.**
- An op that changes parts (`RemovePart`, `AddSentences`, `MovePart`) rebuilds the `Assembly` and re-runs `prepare-short.ts` from the audio step; the UI says "Rebuilding this short" for those only. Every other op reaches the preview through the existing `review-update` path, within the existing 150 ms budget.
- `CutWords` is user-invented cutting, which §36 used to forbid. It is allowed here because the user asked in words, it must be whole sentences or a detected filler, and it appears in the cuts list with a restore (PS §8 principle 7).

### 49.3 Route

| Item | Value |
|---|---|
| Route | `POST /api/v1/chat-edit`, bearer required, 64 kB, 30/min per user |
| Body | `ChatEditRequest { message: String /* at most 500 chars */, recent: Vec<ChatTurn> /* at most 6, each at most 200 chars */, words: Vec<PlanWord>, items: Vec<PlanItemSummary>, parts: Vec<PartSummary>, assets: Vec<AssetLine>, master_outline: Option<Vec<OutlineLine>> }` |
| Response | `ChatReply`, at most 8 kB. The server parses and checks size only; the client validates |
| Errors | `E_CHAT_QUOTA` (429), `E_PLAN_UNAVAILABLE`, `E_PLAN_INVALID` |
| Provider | Same `PlanProvider`, `chat_prompt.rs` generated from the catalog and `PlanOp`; structured output; timeout 20 s |
| Storage and logs | As §40.7. **Chat text is a new class of user text:** not stored on the server, never logged, never in analytics. Local history lives only in the clip's state and is cleared with it |

**`master_outline`** gives the model a compact view of the whole video (one line per sentence, trimmed to fit 64 kB) for "add the part where I talk about pricing".

### 49.4 Interface

`ui/components/ChatPanel.tsx`, a tab beside Captions and Visuals. A text box, the conversation, and for every applied message a one-line summary with an Undo button. Refused ops are explained in plain words ("I can't show a number you didn't say"). Three starter suggestions come from `messages.ts`. **All the review controls stay;** chat is another way to do the same things, and no feature is chat-only except adding sentences to a short (v1.4: B-roll is placed by hand from V6.8, §48.5).

**Failure.** Chat offline or over quota never blocks review: the panel shows why, and every control still works.

**Acceptance (V6.9).** `plan_ops.rs` (each op: apply, refuse, undo equals before; property: `validate` output for any op list contains no unspoken number), `chat_prompt_snapshot.rs`, `chat_route.rs`, `log_redaction.rs` (marker in a chat message), `privacy-network.spec.ts` (chat on and off), `chat-edit.spec.ts` with a fake provider, edit-to-preview budget held for non-part ops.

---

## 50. Brand Kit, Creator Memory, Back-catalogue Queue (V6.10)

### 50.1 Brand kit

As §40.10's old V6.7 row: `BrandKit` (logo via `DrawCmd::Image`, colours into `Theme`, user fonts into `FontSet`), one per account, in this browser; applied to an export only when the profile is Creator (INV-9). Its golden matrix now covers every kind of §47, plus each kinetic preset.

### 50.2 Creator memory

The real switching cost: each edit a creator makes teaches the next plan.

```rust
// offcut-types/src/creator.rs
pub struct CreatorMemory {                     // local only; IndexedDB `creatorMemory`
    pub kind_kept: BTreeMap<EventKind, u32>, pub kind_removed: BTreeMap<EventKind, u32>,
    pub style_counts: BTreeMap<ShortStyle, ShortCounts>,   // kept, deleted, exported
    pub length_buckets: [u32; 5],                          // lengths of exported shorts
    pub parts_buckets: [u32; 4],                           // how often joined shorts are kept
    pub caption_style: Option<StyleId>,
    pub cuts_restored_rate_permille: u32,
}
pub struct CreatorMemorySummary { /* at most 1 kB: enums and small integers only */ }
```

- **Written by** `export-short` and `short_action` use-cases, only from enums and counts already allowed in analytics. No transcript text, no labels, no titles (`creator_memory.rs` proves this by type).
- **Read by** the Director and plan prompts as a preference block: "This creator keeps Story and Listicle, deletes Trivia; usually exports 35-50 s; removes most emoji." The validator's rules do not change.
- **Controls:** settings shows it in plain words, with "Reset". A switch turns it off.
- **Local means not synced.** A second browser starts fresh; sync is PS §21 item 17.
- **Performance data** (views per short from platform APIs) is post-launch, PS §21 item 18. It is the longer-term moat and needs OAuth, which §36 excludes for now.

### 50.3 Back-catalogue queue

```text
ui/components/QueuePanel.tsx          drop up to MAX_QUEUED_SOURCES files; Creator only
usecases/manage-queue.ts              one source at a time, through run-source-pipeline.ts
state/queue-store.ts                  order, state, estimate; `File` objects live for the tab's life
```

- A queued `File` stays an in-memory reference, as every source does (v1.4, §43.3): nothing is copied whole, and each finished source keeps only its shorts' part copies (§44.8).
- **A reload loses queued `File`s** (they cannot be persisted without a handle, P-11): the queue shows the finished ones and asks to drop the rest again.
- Wake lock for the whole queue. A running estimate ("about 2 hours left on this computer"). No promise of background work with the tab closed.
- Every source is transcribed in the cloud (§43.6), so a long queue meets the per-user and global audio budgets of §22.1: when one is reached the queue pauses with the reason, as a single source does.
- Each source uses one Director call, counted toward `CREATOR_LONG_SOFT_PER_MONTH`.

**Acceptance (V6.10).** `profile.rs` brand cases; golden matrix; `creator_memory.rs`; prompt snapshot with a fixed memory; `queue.spec.ts` (three small sources, one failing: the others finish; state correct after a reload).

---

## 51. Person Mask: Text Behind the Speaker, Background Change (V6.11)

**Model: a person-matte model runs once per short in an offline pass, and its result is stored as a low-resolution matte video. The renderer then uses it like any other source. Rejected: running the model during render or preview (INV-12, GPU contention, memory), a matte per frame in raw form (about 200 MB per minute), a green-screen look (the output must pass for shot footage).**

### 51.1 The matte pass

```text
usecases/prepare-matte.ts (on demand: when a short first uses TextBehind or Background)
  asr.worker: load person-runtime.ts (one model, TE-24)
  media.worker: decode the short's frames at MATTE_FPS = 15 (assumption), scaled so the long side is 512 px
  for each frame: model → alpha 0-255; temporal smoothing with a one-pole filter on the alpha (TE-25 tunes it)
  render.worker: encode alpha as luma H.264 to clips/<clipId>/matte.mp4       // the existing encoders
  unload the model before any render session (INV-12)
```

- **Budget:** at most 2× the short's duration on D1 (assumption, TE-24). A demo of MODNet in Transformers.js read about 20 fps on one machine (unverified here); TE-24 measures ours.
- **Why a video:** the matte goes through `VideoSource`, so seek, preview and export need no new code, and the frame index maps as for `Main`. Tooscut has no matte; this is our design. **Rejected:** an H.264 alpha artefact worse than the model's edge would fail TE-25, and the fallback is a losslessly compressed frame file.
- Disk: about 5 MB per minute (assumption). Evicted with the clip.

### 51.2 Drawing

```text
TextBehind list:      Video{Main} → GlyphRun(hook text, large) → Video{Main, matte: Matte}
Background list:      Video{Asset | Image | Blur of Main} → Video{Main, matte: Matte}
```

`Video` gains `matte: Option<VideoSlot>` beside `mask` (`#[serde(default)]`), with feather `MATTE_FEATHER = 1.5 px` at output scale (assumption, TE-25). The text sits over the background and behind the person because the list draws it there. **Tooscut draws all text last, above every layer, so this order is impossible in its design;** Offcut's list-order rule (§40.5) is what makes it cheap.

### 51.3 Backgrounds

From the asset box, or a closed set of bundled gradients. Only when the shot is `TalkingHead` and the face stays in frame. With a moving camera the plan's `Background` is refused (V-2); a static background behind a moving camera looks fake.

### 51.4 The quality gate (founder, 2026-11-02: both in V6.11, gated)

| Check (TE-25) | Pass | If it fails |
|---|---|---|
| Flicker: mean frame-to-frame alpha change in still regions, on 20 corpus shorts | At most `MATTE_FLICKER` (a value set by the labelled clips in TE-25) | Stronger smoothing; then the feature stays off |
| Edge quality: a founder panel marks each of 20 shorts postable or not | At least 16 of 20 for text behind; at least 16 of 20 for background | **That kind's flag stays off at launch, and the founder is asked (§41.4).** One can ship without the other |
| Budget | §51.1 on D1 | Lower resolution or `MATTE_FPS`; then off |

A short whose own matte confidence is low (a mean alpha in the uncertain band above a threshold) is not offered the effect, and review says why.

**Acceptance (V6.11).** `matte_goldens` (a fixed matte frame: text behind, background), `prepare-matte.spec.ts`, verifier 11/11 on an export with each, INV-12 test extended, memory read. TE-24, TE-25 recorded.

---

## 52. AI Provider, Cost and the Spend Cap (V6.2)

**The founder's decision of 2026-11-02 changes PS A-3 for AI only:** a paid AI provider is allowed with a monthly spending cap. Hosting, database, email and storage stay on free tiers.

```sql
-- server/migrations/0002_ai_usage.sql
CREATE TABLE ai_usage (
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  month   DATE NOT NULL,
  kind    TEXT NOT NULL CHECK (kind IN ('plan','shorts','chat')),
  count   INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (user_id, month, kind)
);
CREATE TABLE ai_shorts_grants (
  user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  month       DATE NOT NULL,
  prefix_hash BYTEA NOT NULL,                -- hash of the client's sourceHashPrefix; no content
  PRIMARY KEY (user_id, month, prefix_hash)
);
CREATE TABLE ai_spend (
  month     DATE PRIMARY KEY,
  micro_usd BIGINT NOT NULL DEFAULT 0
);
```

`ai_usage` replaces the v1.2 `plan_usage` table name; it was never created. S-10's list gets `ai_usage` and `ai_shorts_grants`.

| Quota | Anonymous | Free | Creator |
|---|---|---|---|
| Plans (`/plan`) | 3 per day per network | 40 per month | 1,000 per month soft, blocked at 3× |
| Long videos (`/shorts`) | Not available | 1 per month | 200 per month soft, blocked at 3× |
| Chat (`/chat-edit`) | Not available | 20 per month | 1,000 per month soft, blocked at 3× |

All are assumptions, set by TE-15 and §39.3. Counted only on success. `FREE_PLANS_PER_MONTH` rises from 30 to 40 so that one long video's ten per-short plans leave room for ordinary clips.

**The cap.**

| Item | Value |
|---|---|
| `AI_MONTHLY_CAP_USD` | Render environment; the founder sets it; starting value 50 |
| Counting | After each provider call: `input_tokens × price_in + output_tokens × price_out`, prices in config, added to `ai_spend` |
| At 80% | One warning in the server log (a status line, no content) |
| At 100% | Every AI route answers `E_PLAN_UNAVAILABLE` until the next month; clients fall back to rules (INV-24). Chat says so. **The founder can raise the cap in a redeploy** |
| Secret | No client sees a price or the cap |

**Cost model (PS §11; re-read in TE-15; v1.4 for 45 minutes).** A 45-minute Director call is about 28,000 input tokens (9,000 words at about 3 tokens per word, plus overhead) and about 1,500 output tokens; v1.3's 20 minutes was about 12,000. A per-short plan is about 3,000 input and 1,000 output. One 45-minute video with 8 shorts is about 52,000 input and 9,500 output. At the Gemini Flash-Lite rates seen on 2026-11-01 ($0.10 and $0.40 per million, from aggregators, unverified at the source) that is about 0.9 cents; at Gemini Flash ($0.30 and $2.50) about 4 cents.

**Transcription (v1.4).** A long video is transcribed in the cloud (§43.6). On Groq's paid tier, $0.04 per hour of audio (cloudimplementation D-342), 45 minutes costs about 3 cents. The free tier's limits (7,200 audio-seconds a rolling hour, 2,000 requests a day, TE-C2) carry about two 45-minute videos an hour across all users, so the founder moves to Groq's Dev tier before the private alpha (D-342's trigger, brought forward). Its cost is not counted in `ai_spend`; it is bounded by the global audio budget of §22.1, whose values the founder raises by configuration with the Dev tier. **Model choice is TE-15's, by quality on the labelled set rather than by price:** clip choice is the product, so a stronger model is worth 10× this.

**Provider rules.** The `PlanProvider` trait gets `complete(request: ProviderRequest) -> ProviderOutput` with token counts. One adapter per candidate. A provider whose terms train on this data is not used. The settings page names the provider and its data terms (P-1, P-14).

**(V6.2, as built: D-461 to D-465, D-494, D-500, P189b.)**
- **The provider:** DeepSeek, `PLAN_PROVIDER=deepseek`, `PLAN_MODEL` `deepseek-flash`, prices 0.30 / 1.20 USD per 1M tokens (the founder at CP 1; cached input is counted at the full price, so the spend counts high).
- **Quotas, signed in only:** Free 40 plans a month; Creator logs a line above 1,000 and is blocked at 3,000. There is no anonymous quota.
- **The cap:** `AI_MONTHLY_CAP_USD`, with a warn line at 80% and every AI route `plan_unavailable` at 100%.
- **Configuration:** `PLAN_ENABLED` (default false), `PLAN_PROVIDER`, `PLAN_MODEL`, `PLAN_API_KEY`, `PLAN_API_BASE_URL`, `AI_MONTHLY_CAP_USD`, `PLAN_PRICE_IN_USD_PER_MTOK` and `PLAN_PRICE_OUT_USD_PER_MTOK`; on without a key, a cap or a price stops the boot.
- **The migration:** `0002_ai_usage.sql` creates its tables `IF NOT EXISTS`. From V6.2, `db::migrate` ignores a version it does not know, so a later rollback boots; the one from V6.2 to V6.1 needs `DELETE FROM _sqlx_migrations WHERE version = 2` first (§14.4 of the V6.2 plan).
- **Measured on the real product (P211):** 4.0 to 5.3 s a plan, about $0.0035 by the server's count.

---

## 53. New Experiments, Invariants and Reads (v1.3)

### 53.1 Technical experiments

| ID | Question | Method | Pass | When | If it fails |
|---|---|---|---|---|---|
| TE-17 | (v1.4) Does reading in place hold up, and are part copies fast and correct? | On D1, R1, R2: the probe and the audio step over a 45-minute 1080p H.264 file and a 45-minute 4K HEVC file read in place (`fileRandomAccess`); `remux_range` of 10 parts of 60 s from each; `storage.estimate` | Probe at most 2 s; audio step at most 4 min; 10 part copies at most 60 s; every part file passes the clip probe and the verifier's decode checks (assumptions) | V6.1 (reading), V6.3 (copies) | Larger read cache; copy the parts during the Director's call; ask (§41.4) |
| TE-18 | (v1.4) Cloud transcription of a long source: time, limits, quality at the seams | A 45-minute fixture through `transcribe-source.ts` on Groq; then the labelled long set | To the master transcript at most 6 min p90 on D1 after the audio step; no word lost or doubled at window seams on the labelled set; the per-user and global budgets hold (assumptions) | V6.1, V9 | A larger overlap; a lower concurrency; the Dev tier; ask (§41.4) |
| TE-19 | Director quality with sentences only | The labelled long set (E-13's) through two candidates | At least 60% of the model's shorts rated postable by the panel | V6.4 | Tune prompt; stronger model; more shot lines |
| TE-20 | Do long jobs survive a hidden tab, Memory Saver and sleep? | Run a 45-min job hidden; with Memory Saver on; with a sleep mid-job | Progress continues hidden, at most 2× slower; after discard or sleep it resumes with at most the windows in flight lost, and asks for the file only in the stages of §43.7 | V6.1 | Copy tells the user to keep the tab visible; ask (§41.4) |
| TE-21 | HDR (HLG, PQ) from phones and cameras | Decode through `importExternalTexture`; compare with an SDR render on `verify_mp4.py` | No washed or clipped frames on 5 clips | V6.1 | `REJECT_HDR`; with advice. **Closed (v1.5, founder 2026-11-09): HDR stays rejected with advice; no tone mapping is built (PS §21 item 22)** |
| TE-22 | Preview across a join on long-GOP 4K | `preview-sync.spec.ts` join case at 4K | Late frames within budget | V6.3 | Lower the lead's resolution; a longer `JOIN_LEAD` |
| TE-23 | Which face detector: speed, accuracy, licence | Three candidates on 20 corpus shorts | Under 15 ms a frame on D1; at least 95% of faces found on frontal shots; a licence that allows bundling in a paid product | V6.5 | Next candidate; a coarser `FACE_FPS` |
| TE-24 | Which person-matte model: speed, size, licence | MODNet-class, RVM-class and others | At most 2× duration on D1; at most 30 MB; licence recorded and acceptable (a GPL model is not) | V6.11 | Next; then ask |
| TE-25 | Matte quality | §51.4 | §51.4 | V6.11 | §51.4 |
| TE-C1 | Does the browser's Opus encoder take 16 kHz mono? (cloud transcription) | `docs/cloud_transcription/experiments.md` | Supported in Chrome and Edge 156; 60 s in 0.25-0.34 s; `OpusHead` pre-skip 104 | Cloud transcription | 48 kHz input |
| TE-C2 | Groq's answer, its accuracy per bitrate, its timing | the same | 32 kbps WER 6.2% (FLAC 5.5%); word starts 33 ms from measured onsets (the device's 264 ms late); an invented closing "Thank you." led to D-382's silence guard | Cloud transcription | A higher bitrate |
| TE-C3 | The same visual events, cloud and device | the same | Every event on six clips, plus the 3-item list the device misses | Cloud transcription | Stop and ask |
| TE-C4 | Drop to ready, cloud and device | the same | 13-14× faster on a minute of speech, model cached | Cloud transcription | Recorded |
| TE-C5 | Vercel's rewrite carries the body; a live request's time | the same, at the merge | (P170) | Cloud transcription | Lower the limit or the bitrate |
| TE-27 | (v1.5) Following the speaker: is the right person on screen? | 10 real two-person podcast shorts (wide two-shots), with the speaker marked by hand per sentence; `speaker_path` with `Follow` | The person speaking on screen (alone or in a split) for at least 90% of speech time; no switch inside a sentence; the fine pass still under 20 s per 60 s short on D1 (assumptions) | V6.5 | Tune `SPEAKER_MARGIN` and `SPEAKER_WINDOW`; then "Show both" as the default for two-person shots, and ask (§41.4) |
| TE-28 | (v1.5) The four display fonts: licence, coverage, size | Read each candidate's licence (OFL or Apache only); render the caption corpus' words, digits, punctuation and the emoji fallback; measure the Latin subset | Licence recorded; no missing glyph on the corpus; at most 400 kB together | V6.6b | The next candidate in the class |
| TE-29 | (v1.5) Does the automatic grade do no harm? | 20 shots from the corpus and the labelled long set (dark, mixed light, two cameras, already good); the founder compares graded and ungraded side by side, unlabelled | Graded judged better or equal on at least 18 of 20, worse on at most 1; every already-good shot gets `None` or near it; at most 1 ms more per frame on D1 (assumptions) | V6.6a | Narrower bounds; then camera matching only; then ask (§41.4) |
| TE-26 | (v1.4) Stock B-roll from Pixabay: terms, relevance, the browser fetch | Re-read Pixabay's API terms and content licence; 20 plan-written queries from the labelled set; a fetch from `cdn.pixabay.com` under the app's COEP and CSP; rendition sizes | Terms as §48.6 states them; at least 12 of 20 queries give a clip the founder would use; the fetch works with no proxy (assumptions) | V6.8 | Stock by hand only (no plan-written queries); ask (§41.4) if the terms changed |

**(V6.1, read on D1, 2026-11-08/09; `docs/v6.1/experiments.md`.)** **TE-17:** read in place, a 45-minute 4K HEVC file (1.48 GB) and a 45-minute 1080p H.264 file (5.6 GB): probe 0.15 s, the audio step 22 s and 55 s, peak page memory 34 and 37 MB; the clip page's 4K read with `high_res` (an 89 s 4K H.264 clip, processed and exported): 883 MB, so F-3's condition holds. Cold reads and USB drives are R1's and R2's. **TE-18:** 40 minutes through Groq by the product (2,690 s of audio, 10 s overlap, two in flight): p50 0.72 s and p90 1.05 s a window, 33 s of windows after the audio step, one 429 (Groq's free tier, about 20 requests a minute) paused the job `busy` and Continue finished it; at the 29 seams, 5 of the 96 word edits against the first loop fall within 2 s of a seam (2 lost, 3 changed, none doubled), the rate of edits elsewhere (the seams' 4 s are 5% of the time), and no word lands in room tone; the plan's "no word lost at a seam" fails by 2 words, recorded for the founder (V6 CL open item 27). **TE-20:** a reload mid-run loses only the windows in flight; the hidden tab and a frozen page could not be simulated on the automated Chrome, so the real sleep test is the founder's. **TE-21:** read on the generated HLG file only (no phone clip): Chrome maps it into sRGB without washing it out but clips saturated colours to black; `hdr_ok` stays `false`.

TE-15 now also covers the paid provider: price, the `/shorts` and `/chat-edit` latency, data terms. Its "free tier" pass line becomes "free tier for development; the cost model of §52 fits the cap".

E-12 and E-13 are PS §19's. Their instrument is §45.9's analytics plus `scripts/metrics.sql`.

**(V6.2, as built: readings in `docs/v6.2/experiments.md`.)**
- **TE-15** (DeepSeek Flash, thinking off, 15 labelled transcripts):
  - parse 15 of 15 and p90 3.3 s at P188;
  - the drop rate's median 0% after P188c's prompt fixes, then 10 to 12.5% across the later re-reads (variance: numbers that do not resolve, an icon inside a list);
  - on the real product, 4.0 to 5.3 s and 9 to 12 items a plan.
- **TE-30:** the browser's images against their golden, SSIM 0.9995 on D1; decode 19 to 24 ms.
- **TE-31:** the drift is smooth, with no stepping.
- **§10.5:** a frame with every kind costs 0.78 to 1.23 times V4's.
- **E-11:** `plan_result` and `export_done`'s V6.2 fields arrive with no content.

### 53.2 Invariants

| # | Invariant | Test |
|---|---|---|
| INV-25 | Nothing reads a shorts plan that `validate_shorts` has not returned; every part is whole sentences | `shorts.rs`; trybuild on `ShortSpec` |
| INV-26 | With every flag off and with `Assembly::whole`, frames and samples equal V5's | `flags_identity.spec.ts`, §41.1 |
| INV-27 | Source time reaches recording time only through `Assembly`; only the four files of §44.3 call it | `assembly_readers.rs`; types with no `From` |
| INV-28 | No pixel or frame of a source or asset, and no audio of an asset, leaves the browser. A source's sound leaves only as compressed windows in `POST /transcribe` bodies (§43.6, P-1; v1.4). The vision signals sent are enums and sentence indexes | `privacy-network.spec.ts`; `ShotLine` type has no float coordinates or bytes |
| INV-29 | `plan_ops` are applied only through `effective_plan`, and the result only reaches the scene through `validate` | `plan_ops.rs`; INV-21's test |
| INV-30 | AI spend never passes `AI_MONTHLY_CAP_USD` by more than one in-flight call per worker | `ai_cap.rs` |
| INV-31 | `CreatorMemory` holds no text | Type-level test: no `String` reachable from it |
| INV-32 | (v1.4) The renderer reads only local media: a `Stock` B-roll item reaches `validate` only after `resolve-stock.ts` has downloaded it into the asset store; a stock search query is neither stored nor logged on the server | `resolve-stock.test.ts`; `broll_search_route.rs`; `log_redaction.rs` |
| INV-33 | (v1.5) Each Enhance switch, off, removes every effect of its stage, and `Enhance::v5()` gives V5's frames and samples. No measurement behind an enhancement (noise floor, colour statistics, face tracks, mouth activity) leaves the device | `enhance.rs` (each switch off equals the result with that stage absent); `flags_identity.spec.ts` (DC-16); `privacy-network.spec.ts` |

**INV-5 is reread:** every removed span of **a clip's recording** is a validated, unrestored `Cut`; a short's parts are a validated `Assembly`; nothing inside a clip's recording is reordered (`TimeMap` ascending). **INV-12 is reread** per DC-10. **INV-2 and P-1 add:** the chat message, its short history, and asset labels and notes, each only in the AI request bodies, only with the AI switch on.

### 53.3 Performance budgets for the long path (assumptions, TE-17, TE-18; v1.4)

Reference (v1.4): a 45-minute 1080p H.264 source, read in place, transcribed in the cloud, shorts 60 s each, 8 prepared, 5 kept. v1.3's reference was 20 minutes, copied whole and transcribed on the device, with 55 minutes to the list.

| Stage | Budget on R1 | Note |
|---|---|---|
| Probe | 2 s | Reads the `File` in place |
| Audio to `pcm16` | 4 min | No copy of the file first |
| Coarse vision | 7 min | At most 0.15× the duration (§46) |
| Cloud transcription | 6 min | 32 windows, 2 in flight; network-bound (TE-18) |
| Director | 90 s | A body of about 115 kB (§45.2) |
| Part copies | 60 s | 8 shorts' parts (§44.8, TE-17) |
| Prepare 8 shorts (audio chain, plan, fine vision) | 8 min | |
| **To the list of shorts** | **at most 30 min** | |
| Export 5 × 60 s | 5 × 90 s = 7.5 min | |
| Peak tab memory | at most 1.5 GB | Unchanged; no ASR model on the long path, so the render session is the peak |

Clip budgets of §30 do not change. A 4K source is read separately and has no budget in its first read.

### 53.4 What we learned from the study folder (Tooscut, read 2026-11-01)

`old_project_study_editor/` holds parts of Tooscut (`github.com/mohebifar/tooscut`), Elastic License 2.0. **Rule (founder, 2026-11-02): we read it to learn how a problem is solved, then design and write Offcut's version in Offcut's types with Offcut's tests. No copying or line-by-line translation of code, shaders, structs or constants. It is outside this repository and is never added.** Mediabunny, which it uses, is MPL-2.0; it is not a dependency of Offcut, and using it would need the §41.4 question.

**v1.4 (founder, 2026-11-08):** Tooscut is the main study source for Offcut's editing engine. Each V6.x plan names the parts of Tooscut it read, and each idea taken from them is recorded in the release's changelog and in the table below. The rule above does not change: the engine is designed for shorts and written in Offcut's own code. The local copy holds no licence file (the `Cargo.toml` and `package.json` files name none); the founder keeps upstream's licence text beside it (v6changelog open item 22).

| Idea | Where it applies here | What differs |
|---|---|---|
| Decode audio regions in windows | §43.5 | PCM goes to `out48`, not a real-time mixer |
| Per-frame source requests, grouped by source, in time order | §44.3, §48.1 | Our scene answers; frames stay `VideoFrame` |
| Layered, z-ordered frame description | §48.2 | Our list-order `DisplayList`; Vello runs between videos |
| Cross-transition with a boundary and an outgoing and incoming side | §48.3 | At `join_points`; handles from the `Assembly` |
| Generic keyframes with bezier easing | §46.4, zooms | Evaluated in pure Rust inside `frame_at` |
| Sample-rate mismatch correction for HE-AAC | §43.5 | Region decoder |
| 3D LUT colour looks | Post-launch candidate, PS §21 item 16 | |
| (v1.5, read 2026-11-09) Colour grading as lift, gamma and gain wheels, and slope, offset and power | §47.4, the automatic grade | Values computed per shot from measurements; no user controls; one uniform in the video pass |
| (v1.5) Bezier easing per keyframe | §47.2 easing curves, the motion track | A function of time in `frame_at`; no keyframe store |
| (v1.5) `ZoomIn` and `ZoomOut` transitions | §48.3 | At joins only, on media; text stays above |
| (v1.5) A three-band EQ of shelves and a peak, as biquads | §47.1 voice tuning | Set automatically against a reference voice curve, bounded to ±3 dB, offline in the chain |
| Not adopted | Text drawn last above all layers; slides on media only as a general rule; a 13×13 blur sample per pixel; preview and export through different decoders; a uniform buffer per layer per frame. (v1.5) Log colour spaces, tone mapping, HSL and power-window controls, rotate, flip and wipe transitions, reverb, a noise gate, speed changes | Each would cost us something we already have, or the founder's decisions of 2026-11-09 exclude it |

### 53.5 Analytics and the crate graph

**Allowlist additions** (`offcut-api-types/src/analytics.rs`, enum and count properties only, INV-18; each is added by the release named):

| Event | Release | Properties |
|---|---|---|
| `long_source_accepted` | V6.1 plan; **not built in V6.1** (as built: `analytics.rs`' tests pin the event list, and no existing test changed in V6.1; built before `long_video` goes on, V6 CL open item 26) | `duration_bucket`, `resolution_bucket`, `codec` |
| `source_stage_timing` | V6.1 plan; not built in V6.1 (as above) | `stage`, `ms_bucket`, `resumed: bool` |
| `source_paused` | V6.1 plan (v1.4); not built in V6.1 (as above) | `reason` (the four of §43.6, or `awaiting_file`), `stage` |
| `plan_result` | V6.2 | As §40.8 |
| `director_result` | V6.4 | `source`, `outcome`, `shorts`, `dropped_by_reason`, `joined_shorts` |
| `short_action` | V6.4 | `action`, `style`, `parts_bucket` |
| `vision_result` | V6.5 | `shots_bucket`, `face_share_bucket`, `backend` |
| `chat_edit` | V6.9 | `ops`, `refused`, `outcome`, `changed_parts: bool` |
| `queue_result` | V6.10 | `done`, `failed`, `cancelled` |
| `export_done` gains | V6.2-V6.11 | `from_short`, `items_kept`, `items_disabled`, `cuts_kept`, `cuts_restored`, `music_on`, `brand_kit_on`, `broll_items`, `framed: bool`, `chat_ops`; (v1.5) `enhance_off` (a list of `EnhanceRow`), `speaker: SpeakerChoice`, `quality_fixes` (a count) |

No property holds text. The settings page's list is generated, so it stays exact (P-4).

**Crate graph (§7).** `offcut-vision` is new and depends only on `offcut-types`; `offcut-wasm-core` may depend on it (V6.5). `ALLOWED_CRATE_DEPS` in `check-file-tree.mjs` gets its row in that release. `offcut-text` keeps its dependencies; `rebase.rs` reads `Assembly` from `offcut-types`. `offcut-plan` keeps `offcut-text` and `offcut-types`. The server's dependency on `offcut-plan` covers the Director and chat prompts (feature `prompt`).

### 53.6 v1.3 open items

Each (assumption) of §41-§52 is measured by the experiment or release named beside it, and joins §39.3's reading when read. The founder's open decisions:

1. `AI_MONTHLY_CAP_USD` (starting value 50), at V6.2.
2. Which provider and model, at TE-15, by quality first (§52).
3. Whether to run E-12 (optional; recommended).
4. Which V6.8-V6.11 release to reorder or drop after the alpha (§8A.0 of `buildplan.md`).
5. Each person-mask effect after TE-25.
6. ~~A 10-minute cap if TE-18 or TE-20 fails on real laptops.~~ v1.4: long sources are transcribed in the cloud (§43.6); a failed TE-20 is still a stop-and-ask.
7. The launch date, when V6.11 closes.
8. Paid hosting, once there is revenue (`buildplan.md` Appendix D).
9. (v1.4) Groq's Dev tier before the private alpha, and the audio budgets it allows (§22.1, §52).
10. (v1.4) The per-user audio budget for long sources, `LONG_AUDIO_PER_USER_DAY` (§22.1), and the stock switch's default (§48.6).
11. (v1.4) Which of the caption families of §47.2 V6.6 part C keeps.
12. (v1.5) If TE-27 fails: "Show both" as the default for two-person shots, or more work on the speaker decision.
13. (v1.5) If TE-29 fails: a narrower automatic grade, camera matching only, or none.
14. (v1.5) E-16's blind test before the founding release (PS §19): what to fix first if Offcut loses.
15. (v1.5) The agent's design choices of v6changelog Entry 38, for the founder to confirm or change before V6.2's plan.
