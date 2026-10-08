# technicalspec.md - Offcut Technical Specification (v1.1)

**Status:** v1.1. Implements `product.md` "Offcut Product Spec v1.1". Changes since v1 are listed in Section 0.

**Precedence.** The product spec (PS) wins on scope, priority, pricing and success numbers. This document only adds technical precision. It adds no feature, changes no priority and restates no product number differently. Labels P0/P1/P2, M0-M4, E-1 to E-10, H1-H4, A-1 to A-12 and reason codes (a)-(f) keep their PS meaning.

**Founder constraints (fixed inputs to this version).**
- Frontend: React + Vite, deployed on Vercel.
- Backend: deployed on Render.
- Recurring infrastructure cost: 0 USD/month, with no paid fallback (PS A-3, PS A-10). Every component below runs on a free tier or has no fixed fee; when a free tier stops fitting, the component moves to another free tier, never to a paid one. Free-tier terms are external facts: each is verified by a named experiment (§37), never assumed.

**Timeline principles (fixed for the MVP; PS §8 principle 7, PS §9.2).** The user is assumed to have provided the take they want. Offcut enhances the recording; it does not edit the recording's timeline.
- **`TIMELINE_PRESERVATION` (INV-5).** For the MVP, Offcut does not automatically remove, shorten, reorder or compress any portion of the user's recording timeline. Source time equals output time for audio, video, transcript, captions and events, for the whole recording.
- **`AUDIO_DURATION_PRESERVATION` (INV-10).** The audio enhancement pipeline must not intentionally change the duration of the recording. The output audio timeline stays aligned with the source media timeline.
- **Audio enhancement is not audio editing.** Enhancement keeps the timeline and changes signal quality (high-pass, denoise, compression, loudness normalization, limiting). Editing changes the timeline (remove silence, cut or shorten a pause, remove a sentence or a mistake, trim the start or end, collapse gaps). The MVP does the first and none of the second (§18, §36).

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

**v1.1 removes automatic timeline editing from the architecture. It is not disabled; it does not exist.**

- **Removed.** Pause tightening and everything that existed only for it: silence profiling, `plan_pauses`, `apply_time_map`, `PauseConfig` and the pause/splice constants, `TimeMap` and `KeptSegment`, the separate source-time and output-time unit types, `EditState.pause_tightening`, `ChangeSummary.pauses_tightened`, `AudioWorkerApi.finishAudio` and `restore`, the `set-pause-tightening.ts` use-case, `offcut-scene/src/timeline.rs`, the separate `chain48` buffer, the pause feed line and analytics props, and their tests.
- **Simplified.** One timeline type (`TimeMs`). The voice chain's output is the final audio (`out48`). Preview and export read the source frame at the output frame's own timestamp.
- **Added.** INV-5 `TIMELINE_PRESERVATION`, INV-10 `AUDIO_DURATION_PRESERVATION`, `duration_preserved.rs`, `timeline-preserved.spec.ts`, two long-pause fixtures (§26).

---

## 1. Architecture Overview

**All media work (decode, transcription, DSP, detection, rendering, encoding, muxing) runs in the user's browser, in four Web Workers around a Rust/WASM core; the server is one stateless Axum process that knows only accounts, plan state, export counts and allowlisted anonymous events, and never receives media, transcript, captions, edits or file names.**

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
    └── /api/v1/account/*            delete, export

MANAGED POSTGRES (free tier, external to Render; TE-6)   users, sessions, subscriptions, receipts, events
EMAIL PROVIDER (HTTPS API, free tier; TE-8)              magic-link delivery
MERCHANT OF RECORD (TE-9)                                hosted checkout, customer portal, webhooks
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
- **Email provider:** delivers one template (magic link).
- **The backend explicitly does not:** transcode, transcribe, render, store or proxy media; store transcripts, captions, edits, file names or paths; hold prices; run scheduled jobs outside its own process; keep in-memory state that must survive a restart (Render restarts free services at any time).

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
| `server` (`offcut-api`) | Routes, DB access, signing, provider adapters | Postgres, email API, merchant API | any media or transcript handling; client crates other than `offcut-types`, `offcut-api-types`, `offcut-entitlement` |
| Vercel config | Headers, rewrite | Render origin | functions, middleware, edge config |

Enforcement of every row: §7.

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
| `nnnoiseless` (candidate) | Denoise; TE-13 decides | Pinned |
| `ebur128` (pure-Rust build) | Loudness measurement | Pinned |
| `ed25519-dalek`, `sha2` | Entitlement verification, model hashing | Pinned |
| `thiserror` | Error enums | Pinned |
| `proptest`, `insta`, `trybuild` | Property, snapshot, compile-fail tests | Pinned |

**Backend**

| Technology | Purpose | Version strategy |
|---|---|---|
| `axum`, `tokio`, `tower`, `tower-http` | HTTP service | Pinned |
| `sqlx` (Postgres, rustls, offline query cache) | DB access with compile-time-checked SQL | Pinned; `.sqlx/` committed |
| `reqwest` (rustls) | Email and merchant API calls | Pinned |
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
| Transactional email API, free tier. Candidates: Resend, Brevo | Magic links | TE-8 decides |
| Merchant of record. Candidates: Paddle, Lemon Squeezy, Dodo Payments (PS A-4) | Checkout, tax, subscriptions; per-transaction fee only (PS A-9), no fixed fee | TE-9 decides |
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
├── fixtures/          fixture generators, small committed speech clips, labeled transcripts, golden frames
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
│   │   │   ├── ids.rs                 [ONLY] id newtypes
│   │   │   ├── limits.rs              [ONLY] every PS limit as a typed constant
│   │   │   ├── media.rs               ProbeInfo, ClipInfo, RejectReason, Orientation, Rotation
│   │   │   ├── transcript.rs          Word, Sentence, Transcript, NormalizedSpan, Quantity, Unit
│   │   │   ├── events.rs              EventKind, EventParams, DetectedEvent
│   │   │   ├── prosody.rs             WordProsody, Prosody
│   │   │   ├── edit.rs                EditState, StyleId, CropOffset
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
│   │   │   ├── demux.rs               Demuxer
│   │   │   ├── probe.rs               probe() -> ProbeInfo
│   │   │   ├── validate.rs            [ONLY] validate_probe(): PS §9.4 limits -> RejectReason
│   │   │   ├── mux.rs                 Mp4Muxer (faststart)
│   │   │   └── mux_boxes.rs           box writers for the muxer (v2implementation D-55)
│   │   └── tests/
│   │       ├── demux_fixtures.rs
│   │       ├── probe_rejections.rs
│   │       └── mux_roundtrip.rs
│   ├── offcut-text/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── tokenize.rs
│   │   │   ├── sentences.rs           segment_sentences()
│   │   │   ├── numbers.rs             [ONLY] spoken/written number parsing and display formatting
│   │   │   ├── units_lex.rs           unit and currency lexicon
│   │   │   └── normalize.rs           normalize_transcript()
│   │   └── tests/
│   │       ├── numbers_table.rs
│   │       ├── numbers_prop.rs
│   │       └── sentences.rs
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
│   │   │   └── prosody.rs             measure_prosody()
│   │   └── tests/
│   │       ├── loudness_target.rs
│   │       ├── chain_no_clicks.rs
│   │       ├── duration_preserved.rs
│   │       └── prosody.rs
│   ├── offcut-scene/
│   │   ├── Cargo.toml
│   │   ├── assets/
│   │   │   ├── fonts/Inter-Variable.ttf
│   │   │   ├── fonts/JetBrainsMono-Variable.ttf
│   │   │   ├── fonts/NotoEmoji-Variable.ttf
│   │   │   └── fonts/LICENSES.md
│   │   ├── src/
│   │   │   ├── lib.rs                 build_scene(), Scene::frame_at()
│   │   │   ├── styles.rs              [ONLY] StyleSpec for Clean, Bold, Tech
│   │   │   ├── safe_area.rs           [ONLY] logical canvas and safe-area constants
│   │   │   ├── fonts.rs               font registry (embedded bytes)
│   │   │   ├── captions.rs            caption chunking and line layout
│   │   │   ├── layout.rs              text shaping through parley
│   │   │   ├── easing.rs
│   │   │   ├── anim.rs                keyframe tracks evaluated at TimeMs
│   │   │   ├── events/mod.rs
│   │   │   ├── events/number_reveal.rs
│   │   │   ├── events/list_reveal.rs
│   │   │   ├── events/from_to.rs
│   │   │   ├── events/keyword_pop.rs
│   │   │   ├── watermark.rs
│   │   │   ├── framing.rs             [ONLY] crop rectangle from ClipInfo + CropOffset
│   │   │   ├── display_list.rs        DisplayList, DrawCmd (backend-neutral)
│   │   │   └── summary.rs             change_summary()
│   │   └── tests/
│   │       ├── display_list_snapshots.rs
│   │       ├── layout_safe_area.rs
│   │       ├── framing.rs
│   │       ├── determinism.rs
│   │       └── snapshots/             insta snapshot files
│   ├── offcut-render/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                 Renderer
│   │   │   ├── gpu.rs                 device/queue/surface setup
│   │   │   ├── video_pass.rs          external frame import, rotate, crop, scale
│   │   │   ├── vello_backend.rs       DisplayList -> vello::Scene -> overlay texture
│   │   │   ├── composite.rs           video + overlay -> target
│   │   │   └── shaders/video.wgsl
│   │   └── tests/
│   │       └── golden_frames.rs       native wgpu, software adapter; reads fixtures/golden/
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
│   │       ├── analytics.rs           [ONLY] analytics allowlist: event names and prop types
│   │       └── errors.rs              ApiError, ApiErrorCode
│   ├── offcut-wasm-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                 panic hook, init
│   │       ├── media_api.rs           open_demuxer, probe_and_validate, resample
│   │       ├── text_api.rs            normalize_transcript
│   │       ├── audio_api.rs           run_chain, measure_prosody
│   │       └── hash_api.rs            Sha256Stream
│   └── offcut-wasm-render/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs                 panic hook, init
│           ├── session.rs             RenderSession (owns Renderer, Scene, Demuxer)
│           ├── detect_api.rs          detect, redetect_sentence
│           ├── profile_api.rs         export_profile_from_token
│           └── mux_api.rs             Mp4MuxerHandle
├── server/
│   ├── Cargo.toml
│   ├── Dockerfile
│   ├── .sqlx/                         sqlx offline query cache (generated, committed)
│   ├── migrations/
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
│   │   │   ├── http_mailer.rs         the chosen provider's adapter (TE-8)
│   │   │   └── templates.rs           magic-link email
│   │   └── db/
│   │       ├── mod.rs                 pool
│   │       ├── users.rs
│   │       ├── magic_links.rs
│   │       ├── sessions.rs
│   │       ├── subscriptions.rs
│   │       ├── webhook_events.rs
│   │       ├── usage_receipts.rs
│   │       ├── analytics_events.rs
│   │       └── platform_waitlist.rs
│   └── tests/
│       ├── common/mod.rs              test app, fake Mailer, fake BillingProvider
│       ├── auth_flow.rs
│       ├── entitlement_rules.rs
│       ├── usage_receipts.rs
│       ├── webhook_idempotency.rs
│       ├── billing_events.rs
│       ├── analytics_allowlist.rs
│       ├── rate_limit.rs
│       ├── account_delete.rs
│       ├── account_export.rs
│       ├── log_redaction.rs
│       ├── no_media_routes.rs
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
│   ├── public/
│   │   ├── robots.txt
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
│   │   │   └── entitlement-public-key.ts
│   │   ├── copy/
│   │   │   ├── messages.ts            [ONLY] all user-facing copy keyed by code
│   │   │   └── messages.test.ts
│   │   ├── net/
│   │   │   ├── http.ts                [ONLY] fetch to the app origin (/api/v1)
│   │   │   ├── http.test.ts
│   │   │   ├── asset-fetch.ts         [ONLY] fetch to the asset CDN
│   │   │   └── api-client.ts          typed route functions
│   │   ├── platform/
│   │   │   ├── capability.ts
│   │   │   ├── capability.test.ts
│   │   │   ├── simd-probe.ts
│   │   │   └── broadcast.ts           BroadcastChannel "offcut-auth"
│   │   ├── state/
│   │   │   ├── capability-store.ts
│   │   │   ├── model-store.ts
│   │   │   ├── clip-store.ts
│   │   │   ├── preview-store.ts
│   │   │   ├── export-store.ts
│   │   │   ├── auth-store.ts
│   │   │   ├── entitlement-store.ts
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
│   │   │       ├── auth-machine.ts
│   │   │       └── auth-machine.test.ts
│   │   ├── usecases/
│   │   │   ├── start-app.ts
│   │   │   ├── import-clip.ts
│   │   │   ├── run-pipeline.ts
│   │   │   ├── run-pipeline.test.ts
│   │   │   ├── edit-word.ts
│   │   │   ├── toggle-event.ts
│   │   │   ├── set-style.ts
│   │   │   ├── set-crop-offset.ts
│   │   │   ├── control-preview.ts
│   │   │   ├── start-export.ts
│   │   │   ├── start-export.test.ts
│   │   │   ├── cancel-job.ts
│   │   │   ├── restore-clip.ts
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
│   │   │   ├── render-cache-repo.ts
│   │   │   ├── entitlement-repo.ts
│   │   │   ├── receipt-outbox.ts
│   │   │   └── receipt-outbox.test.ts
│   │   ├── wasm/
│   │   │   ├── load-core.ts           [ONLY] instantiate offcut_core.wasm
│   │   │   └── load-render.ts         [ONLY] instantiate offcut_render.wasm
│   │   ├── workers/
│   │   │   ├── protocol.ts            [ONLY] message types for all four workers
│   │   │   ├── rpc.ts                 request/response/progress/cancel plumbing
│   │   │   ├── rpc.test.ts
│   │   │   ├── pool.ts                worker lifecycle; typed clients
│   │   │   ├── media.worker.ts
│   │   │   ├── media/import.ts
│   │   │   ├── media/audio-decode.ts
│   │   │   ├── asr.worker.ts
│   │   │   ├── asr/whisper-runtime.ts
│   │   │   ├── asr/model-cache-adapter.ts   serves model files from OPFS; blocks remote loads
│   │   │   ├── asr/word-timestamps.ts
│   │   │   ├── audio.worker.ts
│   │   │   ├── render.worker.ts
│   │   │   ├── render/video-source.ts       demux + VideoDecoder + frame queue
│   │   │   ├── render/preview-loop.ts
│   │   │   ├── render/export-loop.ts
│   │   │   ├── render/encoders.ts           [ONLY] WebCodecs encoder configs and ladder
│   │   │   └── render/opfs-sink.ts
│   │   └── ui/
│   │       ├── pages/
│   │       │   ├── LandingPage.tsx
│   │       │   ├── EditorPage.tsx
│   │       │   ├── UnsupportedPage.tsx
│   │       │   ├── AuthCallbackPage.tsx
│   │       │   ├── AccountPage.tsx
│   │       │   ├── SettingsPage.tsx
│   │       │   └── LegalPage.tsx
│   │       ├── components/
│   │       │   ├── CapabilityGate.tsx
│   │       │   ├── DropZone.tsx
│   │       │   ├── ModelDownloadPanel.tsx
│   │       │   ├── ProcessingFeed.tsx
│   │       │   ├── PreviewPlayer.tsx
│   │       │   ├── ReviewPanel.tsx
│   │       │   ├── CaptionEditor.tsx
│   │       │   ├── EventToggleList.tsx
│   │       │   ├── StylePicker.tsx
│   │       │   ├── CropOffsetControl.tsx
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
│   │           └── components.module.css
│   └── tests-e2e/
│       ├── tsconfig.json              Node types for the tests and the bench only (v2implementation D-63)
│       ├── helpers/network-capture.ts
│       ├── helpers/fake-api.ts
│       ├── helpers/fixtures.ts
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
│       ├── restore.spec.ts
│       ├── failure-recovery.spec.ts
│       ├── api-cold-start.spec.ts
│       ├── privacy-network.spec.ts
│       ├── settings-what-leaves.spec.ts
│       └── clear-local-data.spec.ts
├── verify/
│   ├── verify_mp4.py
│   ├── watermark_mask_720.png          watermark template drawn independently of the renderer
│   ├── requirements.txt
│   └── README.md
├── fixtures/
│   ├── gen_fixtures.sh                generates synthetic fixtures with the ffmpeg CLI (dev tool only)
│   ├── manifest.json                  fixture name -> expected outcome
│   ├── speech/speech_60s_portrait.mp4
│   ├── speech/speech_60s_landscape.mp4
│   ├── speech/speech_20s_noevents.mp4
│   ├── speech/speech_20s_long_pause.mp4
│   ├── speech/speech_30s_sparse.mp4
│   ├── speech/README.md               scripts read aloud, expected transcript and events
│   ├── labeled/                       >= 30 hand-labeled transcripts (JSON), one file each
│   ├── labeled/SCHEMA.md
│   └── golden/                        PNG goldens per style x event kind
├── corpus/
│   ├── manifest.json                  clip id, sha256, tags (vfr, rotated, noisy, accent)
│   ├── fetch_corpus.sh
│   └── run_corpus.ts
├── bench/
│   ├── device-bench.ts
│   └── results/.gitkeep
├── scripts/
│   ├── build-wasm.sh
│   ├── gen-types.sh
│   ├── check-gen-clean.sh
│   ├── check-file-tree.mjs            fails if a source file is absent from this section
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
    ├── buildplan.md                   build order as versions V1-V10 over the §28 steps
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
        └──────────► offcut-entitlement ─────────────┘                 ▲
offcut-api (server) ─► offcut-api-types, offcut-entitlement[sign] ─────┘
```

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
| Pure crates (`offcut-types`, `-mp4`, `-text`, `-detect`, `-dsp`, `-scene`) depend on `web-sys`, `js-sys`, `wasm-bindgen` or `wgpu` | `cargo-deny` `[bans]` per-crate wrappers in `deny.toml`; CI step `cargo deny check` |
| Pure crates read a clock or randomness (`std::time`, `rand`, `getrandom`) | `clippy.toml` `disallowed_methods` / `disallowed_types`; CI `cargo clippy -D warnings` |
| `server` depends on `offcut-mp4`, `-text`, `-detect`, `-dsp`, `-scene`, `-render` | `cargo-deny` ban on those crates for package `offcut-api`; `server/tests/no_media_routes.rs` |
| A crate imports a crate above it in the diagram | Cargo rejects cycles; `scripts/check-file-tree.mjs` checks declared dependencies against this section |
| `ui` imports `workers`, `net`, `persistence`, `wasm`, `models`, `analytics` | `eslint-plugin-boundaries` element rules in `web/eslint.config.js` |
| `state` imports anything except `gen` | same |
| `fetch`, `XMLHttpRequest`, `WebSocket`, `EventSource`, `navigator.sendBeacon`, `RTCPeerConnection` outside `net/http.ts`, `net/asset-fetch.ts`, `wasm/load-core.ts`, `wasm/load-render.ts` | ESLint `no-restricted-globals` / `no-restricted-properties` with per-file overrides; runtime backstop: CSP `connect-src` (§24.3); proof: `privacy-network.spec.ts` |
| `import.meta.env` outside `config/env.ts` | ESLint `no-restricted-syntax` |
| String literals of user-facing sentences outside `copy/messages.ts` | ESLint `react/jsx-no-literals` (allowing punctuation) in `ui/` |
| Editing `web/src/gen/*` by hand | `scripts/check-gen-clean.sh` regenerates and fails on diff |
| `unwrap`, `expect`, `panic!`, indexing by `[]` on slices in non-test Rust | clippy `unwrap_used`, `expect_used`, `panic`, `indexing_slicing` = deny |
| `any`, non-null `!`, `@ts-ignore` in `web/src` | `@typescript-eslint/no-explicit-any`, `no-non-null-assertion`, `ban-ts-comment` |
| A source file not listed in §5 | `scripts/check-file-tree.mjs` |

**Circularity guards**
- `offcut-types` imports no `offcut-*` crate. New shared types go there, never into a sibling.
- `workers/protocol.ts` imports only `gen`; workers and `pool.ts` both import it, never each other.
- `usecases` never import each other except `cancel-job.ts` and `restore-clip.ts`, which other use-cases may call; checked by a `boundaries` rule.
- Stores never import use-cases; use-cases write stores through exported action functions only.

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
no events detected → ReviewPanel shows messages.NO_EVENTS_FOUND (PS §10 J7)
<Pause>/<Seek> → controlPreview.pause() / seek(TimeMs) ⇒ playing → paused / → seeking → paused|playing
```

**C-6. Review edits (J8)**
```text
<CaptionEditor onCommit(wordIdx, text)> → editWord(wordIdx, text)
  → editsRepo.put → pool.render.redetectSentence({sentenceIdx, wordEdits})
      → redetect_sentence() → events for that sentence replaced; toggles kept by EventId
  → clipStore.setEvents() → pool.render.setScene(...)                          [evt review_action action=word_edit]
<EventToggleList onToggle(id, on)> → toggleEvent(id, on)
  → editsRepo.put → pool.render.setScene(...)                                  [evt review_action action=event_toggle]
<StylePicker> → setStyle(styleId) → setScene                                   [evt review_action action=style]
<CropOffsetControl> (landscape only) → setCropOffset(offset) → setScene        [evt review_action action=crop_offset]
all: preview ⇒ playing → paused during the update, then restored
no edit changes the audio, the clip duration or any timestamp
```

**C-7. Export click and gate (J9)**
```text
<ExportButton> shows messages.exportCounter(remaining) from entitlement.effective()   (before the click)
<ExportButton onClick> → startExport(clipId)
  → blockers.forExport()
      B_PIPELINE_NOT_READY | B_EXPORT_IN_PROGRESS | B_STORAGE_LOW → <ErrorPanel>, stop
      B_NOT_SIGNED_IN       → <SignInDialog> with Free/Creator limits copy (C-8), stop
      B_ENTITLEMENT_EXPIRED → apiClient.getEntitlement()  ✓ continue  ✗ <ErrorPanel E_ENTITLEMENT_EXPIRED>
      B_NO_FREE_EXPORTS     → <UpgradePrompt placement=limit_reached>           [evt upgrade_prompt]
  remaining == 1 on Free → confirm dialog messages.LAST_FREE_EXPORT (PS §11) before proceeding
  → C-9
```

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

**C-9. Render and export (J10)**
```text
startExport (after gate)
  → exportId = newExportId()                  UUIDv7
  → renderCacheRepo.lookup(cacheKey)          ✓ hit → skip to C-10 with the cached file and its original exportId
  → exportStore.begin(exportId)               ⇒ export: idle → rendering       [evt export_started]
  → controlPreview.pause()                    preview is locked while exporting (§12.4)
  → pool.render.exportClip({exportId, entitlementToken, out48})
      → export_profile_from_token(token)      verify signature → ExportProfile (size, watermark)
      → setScene(profile) → encoders.configure(profile)
      → export-loop.runExport(): for n in 0..frames: source frame at frame n's own time → render_frame → VideoFrame → encode → mux
      → AAC encode of out48 → mux             ⇒ rendering → muxing
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
  → <ChangeSummary summary> + <PostExportQuestion>
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
  → adapter.verify_signature(headers, raw_body)     ✗ 401
  → adapter.parse_event(raw_body) → BillingEvent     unknown type → 200 (ignored)
  → tx: webhook_events.insert(provider_event_id)     ✗ unique violation → 200 (already processed; no state change)
        subscriptions.apply(event)  (§22.5)
  → 200
```

---

## 10. Core Domain Types

All types below live in `crates/offcut-types` unless a path is given. They derive `Clone, Debug, PartialEq, Serialize, Deserialize, TS`; unit and id types also derive `Copy, Eq, Ord, Hash` and use `#[serde(transparent)]`. Inner fields of unit types are private; construction is through `new` and conversion functions in `units.rs` only.

### 10.1 Unit and id types

```rust
// units.rs  [ONLY]
pub struct TimeMs(u32);       // a position in milliseconds on the recording's timeline (the only timeline, INV-5)
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

impl FrameIdx { pub fn to_time_ms(self) -> TimeMs; pub fn to_micros(self) -> Micros; }   // n * 1000 / 30, n * 1_000_000 / 30
impl TimeMs   { pub fn to_micros(self) -> Micros; pub fn from_micros_floor(m: Micros) -> TimeMs; }
// No From<u32>. A position (TimeMs) and a length (DurMs) are distinct types. There is no second timeline type.

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

TypeScript mirrors these as branded types generated into `web/src/gen/domain.ts` (`type TimeMs = number & { readonly __unit: "TimeMs" }`). `as TimeMs` casts are allowed only in `gen/` and `workers/`; ESLint `no-restricted-syntax` rejects them elsewhere.

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

There is no time-map type. Every `TimeMs` in a `Word`, `Sentence`, `ListItem` or `DetectedEvent` is the time at which it was spoken in the recording, and it is used unchanged by the scene, the preview and the export.

### 10.6 Edits, profile, summary, stage, errors

```rust
// edit.rs
pub enum StyleId { Clean, Bold, Tech }                  // PS §12.2; default Clean
pub struct CropOffset(f32);                             // -1.0 (left) ..= 1.0 (right); 0.0 centre; checked in new()
pub struct EditState {
    pub word_edits: BTreeMap<WordIdx, String>,          // replacement text; "" hides the word; timing never changes
    pub event_overrides: BTreeMap<EventId, bool>,       // enabled flag chosen by the user
    pub style: StyleId,
    pub crop_offset: CropOffset,
}                                                       // user choices only; no field changes audio, duration or timing

// profile.rs
pub enum Plan { Free, Creator }
pub enum ProfileKind { Preview, Free, Creator }
pub struct ExportProfile { pub kind: ProfileKind, pub width: Px, pub height: Px, pub watermark: bool,
                           pub video_bitrate: BitsPerSec, pub audio_bitrate: BitsPerSec }

// summary.rs   ("What we changed", PS §10 J11)
pub struct ChangeSummary { pub voice_cleaned: bool,          // the enhancement chain ran (§18)
                           pub captions_emphasized: u32,     // enabled KeywordPop events
                           pub visual_moments: u32 }         // enabled NumberReveal + ListReveal + FromTo
// Lists only transformations that are performed. Nothing is cut, so nothing about cuts is reported.

// stage.rs
pub enum PipelineStage { ProbeAudio, Asr, AudioChain, DetectScene, RenderEncode, Mux }   // matches PS §20.2 rows

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

- **Why these representations.** One timeline type, `TimeMs`, because source time and output time are the same instant for the whole recording (INV-5); a second type would encode a distinction that does not exist. `DurMs` stays separate so a length is never passed where a position is expected. Integer milliseconds match the PS data model (PS §12.5) and are exact at 30 fps to within 1 ms; frame timestamps are derived from `FrameIdx`, never accumulated. `Sentence.words` is a range because sentences never overlap.
- **Serialization.** Rust ↔ worker JS: `serde-wasm-bindgen` for structs; PCM as `Float32Array`; frames as `web_sys::VideoFrame` handles; file access through the `RandomAccess` callback. Worker ↔ main: structured clone of generated types; PCM buffers are transferred, never copied. Client ↔ server: JSON of `offcut-api-types`. Storage: IndexedDB structured clone of generated types wrapped in `{ schemaVersion, value }` (§23.2). Entitlement token: `base64url(JSON claims) "." base64url(Ed25519 signature)`.
- **Ownership (who may write).** `ClipInfo`: `validate_probe` only. `Transcript`: `normalize_transcript` only; user edits never mutate it (they live in `EditState`). `DetectedEvent`: `offcut-detect` only; `enabled` is overlaid from `EditState.event_overrides` in `build_scene`. `EditState`: use-cases through `edits-repo.ts`. `EntitlementClaims`: `server/src/entitlement/issue.rs` only. `ExportProfile`: `offcut-entitlement/src/profile.rs::export_profile` only.

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

Stores hold no derived data that another store owns. `entitlement/entitlement.ts::effective()` is the only place that combines the token with the outbox length.

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
- `auth: anonymous → signed_in` without `link_sent` or a refresh.

### 12.4 Cross-machine rules

- Capability `!= supported` blocks import, processing and export. It does not block landing, sign-in, account or settings pages.
- Export in `rendering` or `muxing` locks the preview (`locked`), disables every review control, and blocks import and `clearLocalData`.
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

---

## 13. Client Architecture

**Model: a single-page React app whose main thread only renders stores and dispatches use-cases; rejected: a multi-page app (loses in-memory clip state across the sign-in step) and server-side rendering (no server to run it; no benefit).**

### 13.1 Routes

| Path | Page | Notes |
|---|---|---|
| `/` | `LandingPage` | J1: hero, before/after demo video (asset CDN), supported-browser line, "What leaves your device" link, `DropZone`, sample-clip button. Runs the capability check for the E-8 beacon |
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

### 13.3 Key components

| Component | Reads | Calls |
|---|---|---|
| `CapabilityGate` | capability store | none |
| `DropZone` | clip store, blockers | `importClip` |
| `ModelDownloadPanel` | model store | `cancelJob("model")`, retry via `runPipeline` |
| `ProcessingFeed` | clip store feed lines | `cancelJob("pipeline")` |
| `PreviewPlayer` | preview store, clip store | `controlPreview.*` |
| `ReviewPanel` → `CaptionEditor`, `EventToggleList`, `StylePicker`, `CropOffsetControl` | clip store | `editWord`, `toggleEvent`, `setStyle`, `setCropOffset` |
| `ExportButton` | export store, `entitlement.effective()`, blockers | `startExport` |
| `ExportProgress` | export store | `cancelJob("export")` |
| `SignInDialog` | auth store | `requestMagicLink` |
| `ChangeSummary`, `PostExportQuestion` | export store | `answerPostExport` |
| `UpgradePrompt` | entitlement store | `startCheckout` |
| `WhatLeavesTable` | `gen/api.ts::ANALYTICS_EVENT_DOCS` | none |
| `RecentClips` | clips repo via clip store | `restoreClip(clipId)` |
| `RejectionPanel`, `ErrorPanel` | clip/export store failure | retry use-case, `track("client_error")` |
| `NotifyMeForm` | none | `submitNotifyMe` |

### 13.4 Rules

- **Copy.** Every PS §10 microcopy string, every rejection, error, blocker and unsupported message is a key in `copy/messages.ts`. Numbers inside copy (90 seconds, 3 exports, prices, model size) are interpolated from `gen/domain.ts` limits, `config/pricing.ts` and `config/model-manifest.json`; none is typed into a sentence. Enforced by `messages.test.ts` (asserts no digit literals in message templates except through placeholders).
- **Limits before the click (PS §10 J9).** `ExportButton` renders the counter and plan limits in its idle state; `SignInDialog` renders the Free/Creator comparison before the email field. `export-free.spec.ts` asserts the counter is visible before the first click.
- **No upgrade prompts during processing or editing (PS §11).** `UpgradePrompt` renders only for placements `after_export` and `limit_reached`; the placement type has no other member.
- **One prompt per export.** Dismissal is stored in `export-store` for that `ExportId`.
- **Edits are optimistic and local.** No edit ever awaits the network.

### 13.5 Acceptance targets

- Capability check completes within 3 s (PS §9.3); `capability.spec.ts`.
- App shell (HTML + JS + CSS, excluding WASM) at most 400 kB gzip (assumption, M2.4); `scripts/check-hosts.mjs` also reports bundle sizes.
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
  | { kind: "event_found"; eventKind: EventKind; display: string };   // display stays on device
export type AppFailure = { code: ErrorCode; stage: FailureStage; retryable: boolean; detail?: string };
```

### 14.2 Methods

```ts
export interface MediaWorkerApi {
  importAndProbe(p: { clipId: ClipId; file: File }): Promise<{ ok: ClipInfo } | { rejected: RejectReason }>;
  extractAudio(p: { clipId: ClipId }): Promise<{ pcm48: Float32Array; pcm16: Float32Array }>;   // transferred
}
export interface AsrWorkerApi {
  load(p: { modelId: string; backend: "webgpu" | "wasm" }): Promise<{ backend: "webgpu" | "wasm" }>;
  transcribe(p: { pcm16: Float32Array }): Promise<{ ok: Transcript } | { rejected: "NoSpeech" }>;
  unload(): Promise<void>;
}
export interface AudioWorkerApi {
  runChain(p: { pcm48: Float32Array }): Promise<{ loudness: Lufs }>;                 // out48 kept in the worker; out48.length === pcm48.length
  measureProsody(p: { words: Word[]; sentences: Sentence[] }):
    Promise<{ out48: Float32Array; prosody: Prosody }>;                              // out48 transferred out, unmodified
}
// The audio worker has no method that detects silence, plans or applies cuts, or changes the length of the audio.
export interface RenderWorkerApi {
  openSession(p: { clipId: ClipId; clipInfo: ClipInfo }): Promise<void>;
  detect(p: { transcript: Transcript; prosody: Prosody }): Promise<DetectedEvent[]>;
  redetectSentence(p: { sentence: SentenceIdx; wordEdits: Record<number, string> }): Promise<DetectedEvent[]>;
  setScene(p: { transcript: Transcript; events: DetectedEvent[]; edit: EditState;
                profile: "preview" | { entitlementToken: string } }): Promise<void>;
  attachPreview(p: { canvas: OffscreenCanvas }): Promise<void>;                      // transferred
  previewPlay(p: { clock: ClockSync }): Promise<void>;
  previewClock(p: { clock: ClockSync }): void;                                       // fire-and-forget
  previewPause(): Promise<void>;
  previewSeek(p: { at: TimeMs }): Promise<void>;
  exportClip(p: { exportId: ExportId; entitlementToken: string; out48: Float32Array }):
    Promise<{ opfsPath: string; summary: ChangeSummary; stageTimings: StageTiming[] }>;
  closeSession(): Promise<void>;
}
export type ClockSync = { audioMs: TimeMs; epochMs: number };   // epochMs = performance.timeOrigin + performance.now()
export type StageTiming = { stage: PipelineStage; durationMs: DurMs };
```

### 14.3 Lifecycle, cancellation, backpressure

- **Creation.** `pool.ts` creates each worker lazily on first use and loads its WASM bundle once. `asr` keeps its worker but drops the model session on `unload()`. `render` lives from `openSession` to `closeSession`.
- **One job per worker.** A second `req` while one is pending is rejected with `E_INTERNAL`. Exception: `previewClock`, `previewPause`, `previewSeek` and `cancel` are accepted during `previewPlay`.
- **Cancellation points.** Import: every 4 MiB chunk. Audio decode: every decoded AAC batch. ASR: every 30 s audio window. DSP: every 1 s block. Render/export: every frame. A step past its check finishes, then the job resolves `{cancelled: true}` and releases every `VideoFrame`, closes encoders and deletes temp files.
- **Cancel timeout.** `CANCEL_TIMEOUT = 2 s` (assumption, M1.5). No response → `pool.restartWorker(kind)`.
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

**Seams (not built now).** A fifth worker for face tracking (P1) or a local LLM (P2) is a new `WorkerKind` and interface in `protocol.ts`; nothing else in the hub changes.

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

Timestamps returned by the demuxer are presentation times with edit lists applied and the first presented video frame at 0. Audio is shifted by the same offset; `decodeAudio` pads leading silence or trims so that audio sample 0 is video time 0, and pads or trims the tail to the video duration. This is container-level track alignment (the two tracks rarely start and end on the same instant); it never looks at the audio content and removes nothing from inside the recording. After it, `pcm48.len() == round(ClipInfo.duration x 48)` and that length is carried unchanged to `out48` (INV-10).

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

| Code | Trigger | Message must state | Fixture | Test |
|---|---|---|---|---|
| `REJECT_FILE_SIZE` | size > 500 MB | actual size, the 500 MB limit, how to shrink | generated sparse file | `rejections.spec.ts` |
| `REJECT_CONTAINER` | not MP4/MOV | accepted containers | `rej_vp9.webm` | `probe_rejections.rs`, `rejections.spec.ts` |
| `REJECT_CORRUPT` | parse failure | file looks damaged; re-export | `rej_truncated.mp4` | same |
| `REJECT_NO_VIDEO` | no video track | needs a video | `rej_no_video.m4a` | same |
| `REJECT_HEVC` | HEVC video | PS §9.4 HEVC copy ("Most Compatible" / export as H.264) | `rej_hevc.mov` | same |
| `REJECT_VIDEO_CODEC` | other non-H.264 | export as H.264 | `rej_av1.mp4` | same |
| `REJECT_NO_AUDIO` | no audio track | needs speech audio | `rej_no_audio.mp4` | same |
| `REJECT_MULTI_AUDIO_TRACK` | more than 1 audio track | one audio track only | `rej_two_audio.mp4` | same |
| `REJECT_AUDIO_CODEC` | non-AAC audio | export with AAC audio | `rej_mp3_audio.mp4` | same |
| `REJECT_DURATION` | longer than 90 s | actual duration, the 90 s limit, trim advice (PS §9.4 copy) | `rej_too_long_91s.mp4` | same |
| `REJECT_RESOLUTION` | long side > 1920 px | PS §9.4 4K copy | `rej_4k.mp4` | same |
| `REJECT_FRAME_RATE` | average above 60 fps | record at 60 fps or lower | `rej_120fps.mp4` | same |
| `REJECT_DECODE_UNSUPPORTED` | browser cannot decode this H.264 config | re-export with standard settings | mocked `isConfigSupported` | `rejections.spec.ts` |
| `REJECT_NO_SPEECH` | fewer than `MIN_WORDS` words transcribed | no English speech found | `rej_silent.mp4` | `rejections.spec.ts` |

### 15.4 Edge cases

- More than one video track: the first enabled track is used (assumption, TE-12).
- Rotation metadata (90/180/270): decoded frames are treated as unrotated; `video_pass.rs` applies `ClipInfo.rotation`. Orientation and the resolution rule use display size.
- VFR: `is_vfr = true` when sample durations differ by more than 1 ms; accepted; the frame-rate rule uses the average.
- Fragmented MP4 (`moof`): rejected as `REJECT_CONTAINER` at MVP (assumption, TE-12 measures how many corpus clips need it).
- `moov` at end of file: supported (random access).
- Non-speech, non-English, multi-speaker clips: not detected at import. Non-English speech usually yields `REJECT_NO_SPEECH` or poor captions; multi-speaker clips are processed as one speaker. The drop zone states both constraints (PS §10 J2). Listed as open in §39.
- Sample clip ("Try with a sample clip"): fetched from the asset CDN by `asset-fetch.ts` and passed to `importClip` with `source = "sample"`.

**Acceptance.** Probe + audio extraction at most 3 s for the reference clip on R1 (PS §20.2). Corpus: at least 90% of 40+ clips export; the rest are rejected with a specific code; none crash (PS §20.1); `corpus/run_corpus.ts`.

**Contingency.** TE-12 fails (demux coverage under 90% of the corpus): replace `Demuxer` in the two workers with a maintained JS demuxer behind the same `SampleMeta` shape; `probe`/`validate_probe` keep consuming a `ProbeInfo`. PS §20.4: corpus pass rate under 90% at week 8 → remove the least valuable accepted input (first candidate: input above 30 fps) by tightening `limits.rs`.

**Seams (not built now).** HEVC/AV1/WebM input, multi-track audio, clips longer than 90 s: constants and enum arms in `limits.rs`/`validate.rs` only.

---

## 16. Model Manager and Local ASR

**Model: a quantized English Whisper-family model run by ONNX Runtime Web inside `asr.worker` (WebGPU first, WASM-SIMD + threads fallback); model files are downloaded by the app from the asset CDN into OPFS, hash-verified, and served to the runtime only from OPFS. Rejected: the runtime's default hub/CDN loading (third-party hosts break the closed network list), a CPU-only WASM build (misses PS §20.2 on R1), cloud ASR (PS §9.8).**

### 16.1 Model manifest

`web/src/config/model-manifest.json` ships inside the app bundle, so the expected hashes are pinned by the deployed build, not fetched.

```json
{ "modelId": "asr-en-v1", "modelVersion": "<runtime+weights version string>",
  "files": [ { "path": "models/asr-en-v1/<file>", "bytes": 0, "sha256": "<hex>" } ],
  "totalBytes": 0 }
```

Candidates (TE-1, E-3, E-10 decide; values filled at M0.2): base-size English model (default candidate), small-size English model (if E-10 WER exceeds 12%), tiny-size English model (PS §20.4 fallback). Constraint: `totalBytes` at most 150 MB (PS §10 J4, §20.2). The J4 copy interpolates `totalBytes` rounded to 10 MB.

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
   - runtime .wasm files load from "/ort/" on the app origin
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

**Acceptance.** Median transcription at most 20 s for the reference clip on R1 (PS §20.2, E-3). WER at most 12% and median caption edits at most 8 per 60 s on the 20-clip set (E-10). First-run download plus initialization at most 90 s at 25 Mbps (PS §20.2). Second session skips the download (PS §20.1); `model-download.spec.ts`. No request to a host outside §24.1; `privacy-network.spec.ts`.

**Contingency (PS §20.4).** ASR median above 40 s on R1 → ship the tiny-size model and keep the cloud fallback as a P1 trigger. TE-1 fails (runtime cannot be confined to OPFS, or no word timestamps) → second candidate runtime (Rust inference compiled to WASM/WebGPU) behind the same `whisper-runtime.ts` interface; if that also fails, word timestamps are approximated by forced alignment of segment text to energy onsets and E-7 decides whether events remain viable.

**Seams (not built now).** Consented cloud transcription (P1, PS §12.6): an alternative implementation of `AsrWorkerApi.transcribe` selected per clip, a new host in `allowlist-hosts.ts`, a consent dialog and a `CloudJobConsent` record. Other languages (P2): one manifest per language. Local LLM (P2): a separate manifest entry and worker.

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

### 17.2 Normalization

- **Sentences.** A boundary after terminal punctuation (`.`, `?`, `!`) or a gap of at least `SENTENCE_GAP = 700 ms`; a sentence longer than `SENTENCE_MAX_WORDS = 40` is split at its longest internal gap (assumptions, E-10).
- **Numbers (`numbers.rs`).** Parses: digit forms with separators and decimals; spelled cardinals from zero to trillions ("ten thousand" → 10,000, PS §9.6); "point" decimals; magnitude suffixes k, m, b, "grand"; currency symbols and words (USD, EUR, GBP, INR); percent; multipliers ("3x", "ten times"); units from `units_lex.rs` (time, data size, count nouns). Not parsed: fractions in words, ranges, dates.
- **Display.** `Quantity.display` is produced only by `format_quantity`; captions show it in place of the spoken words of the span.
- **Edits.** Detection and captions read effective text: `edits[idx]` when present, else `Word.text`. The transcript itself is immutable.

### 17.3 Detection

```text
detect(t, p, edits, cfg):
  toks  = effective tokens (lower-cased, punctuation split)
  cands = from_to::find(toks) ++ list::find(toks) ++ number::find(toks, t.numbers) ++ keyword::find(toks, p)
  cands = exclusions::apply(cands)                    // may zero a score
  cands = cands.filter(c => c.score >= cfg.threshold[c.kind])
  kept  = resolve::resolve(cands, cfg)                // §17.5
  kept.map(c => DetectedEvent { id: event_id(c.kind, c.anchors), enabled: true, confidence: c.score, .. })

redetect_sentence(.., s, prev, ..):
  scope = sentence s ∪ sentences touched by any prev event intersecting s ∪ next cfg.list_window sentences
  new   = detect restricted to scope
  return prev.filter(e => e.anchors outside scope) ++ new       // ids stable where anchors are unchanged
```

### 17.4 Scoring (`config.rs`; all values are (assumption), tuned at M1.3 against `fixtures/labeled/` and re-checked in E-7)

Scores are clamped to 1.0 and multiplied by the minimum ASR confidence of the anchor words.

| Kind | Candidate requires | Score | Threshold |
|---|---|---|---|
| FromTo | Two quantities in one sentence with compatible units, different values, at most `FROMTO_MAX_GAP = 8` words apart | 0.55 base; +0.25 "from … to" frame, or +0.15 change-verb frame ("went", "dropped", "grew", "down", "up" … "to"); +0.15 both carry the same explicit unit, or +0.05 one inherits the other's; +0.05 gap of 6 words or fewer | 0.85 |
| ListReveal | Announcer (count 2-7 + list noun from the lexicon) followed by that many ordered markers within `LIST_WINDOW = 45 s`; or at least 3 ordered ordinal-word markers starting at "first" | Announcer form: 0.60 base; +0.30 all markers found in order; +0.10 markers are ordinal words. Marker-only form: 0.65 base; +0.20 | 0.85 |
| NumberReveal | A `NormalizedSpan` not consumed by FromTo or a list announcer/marker | 0.50 base; +0.30 currency, percent or explicit unit; +0.20 magnitude at least 1,000 or a suffix; +0.10 `energy_z >= 1.0` | 0.80 |
| KeywordPop | Content word of at least 4 letters, not a stop word, outside every other event's anchors | 0.50 base; +0.20 x clamp(`energy_z` - 1, 0, 1); +0.20 x clamp(`pitch_z` - 1, 0, 1); +0.20 preceded by an intensifier or in the emphasis lexicon | 0.80 |

**Exclusions (`exclusions.rs`, score set to 0):** bare integers 1900-2099 without unit (years); tokens adjacent to a version/product pattern ("Python 3.12", "iPhone 15"); clock times ("at 5 pm", "10:30"); phone-like digit runs; bare integers below 10 without unit; a number that is a list marker.

**Derived fields.**
- `NumberReveal.label`: up to `NUMBER_LABEL_MAX_WORDS = 3` following words, stopping at punctuation or a stop-list verb; `None` if empty.
- `ListReveal.header`: `"{count} {NOUN}"` in the announcer form; empty in the marker-only form (the header row is not drawn).
- `ListItem.text`: words after the marker up to a clause boundary, the next marker or `LIST_ITEM_MAX_WORDS = 5`, leading fillers removed.

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
- A word edit that changes a number re-parses that sentence; the old event disappears and a new id is created; its toggle state is not inherited.
- A hidden word (`""` edit) inside an event's anchors removes the event.
- Empty transcript or zero events: `detect` returns an empty list; the UI shows `NO_EVENTS_FOUND` (PS §10 J7).

**Acceptance.** Precision at least 0.90 per run over at least 30 hand-labeled transcripts covering all four kinds (PS §20.1); `precision_recall.rs` fails below it and prints per-kind precision, recall and a threshold sweep. A prediction is a true positive when kind matches, anchor ranges overlap by at least 50% intersection-over-union, and for FromTo/NumberReveal the parsed values are equal. Same inputs → identical output; `determinism.rs`. Detection at most 1 s for 150 words on R1 (PS §20.2). `redetect_sentence` at most 50 ms (assumption, M1.5).

**Contingency.** Precision under 0.90: raise the per-kind threshold to the lowest value in the sweep that reaches 0.90; if a kind's recall then falls under 0.30 (assumption, E-7), keep it shipped but report it as the E-7 "tune rules or reconsider the wedge" input.

**Seams (not built now).** P1 event kinds (Warning, Question, Quote, Checklist, Timeline): a new `EventKind` variant, one file in `offcut-detect/src/`, one in `offcut-scene/src/events/`, labeled fixtures. Local LLM (P2): another producer of `Vec<DetectedEvent>` behind `RenderWorkerApi.detect`.

---

## 18. Audio Enhancement

**Model: offline block processing of mono 48 kHz f32 in Rust: high-pass → denoise → compressor → loudness normalization → limiter → `out48`. The chain changes the characteristics of the signal, never its timeline: `out48` has exactly as many samples as the decoded input, and sample `i` of the output is sample `i` of the recording (INV-10). Rejected: a Web Audio / `OfflineAudioContext` graph (browser-dependent output, not testable natively), stereo processing (one speaker, PS §9.4), and any silence detection, pause classification, pause shortening or splicing (PS §8 principle 7: the take is the user's).**

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

### 18.2 Pipeline

```text
run_chain (blocks of 1 s; cancel check per block):
  1 high-pass      2nd-order Butterworth at HPF_HZ
  2 denoise        RNNoise-class, 10 ms frames, wet mix DENOISE_MIX           (skipped when cfg.denoise == false)
  3 compressor     feed-forward, soft knee
  4 loudness       integrated loudness (BS.1770 gating) → gain to TARGET_LOUDNESS, gain capped at MAX_GAIN_DB;
                   measured loudness below ABSOLUTE_SILENCE → gain 0 dB
  5 limiter        look-ahead, ceiling LIMIT_CEILING
  6 re-measure     if |loudness - TARGET| > 0.5 LU → one corrective gain pass + limiter
  every stage is length-preserving and delay-compensated: the denoiser's frame delay and the limiter's look-ahead
  are removed before the next stage, and the last partial block is processed, not dropped
  → out48, with out48.len() == pcm48.len()          (worked exemplar: §34.3)

measure_prosody:
  per word: energy = RMS dB over [start, end]; pitch = median F0 by autocorrelation in 60-400 Hz (unvoiced → none)
  z-scores against the words of the same sentence; sentences with fewer than 3 words or zero variance → 0.0
```

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

No constant in this table is a silence threshold for cutting, a minimum pause, a kept-pause length or a crossfade. `ABSOLUTE_SILENCE` only stops the loudness stage from applying gain to a clip with no measurable programme.

### 18.4 Lifecycle and edge cases

- `audio.worker` keeps `out48` between `runChain` and `measureProsody`, then transfers it to the main thread. The worker holds no audio after the pipeline and has no later stage; no review edit re-runs audio.
- A long pause, many pauses, or a clip that is mostly silence: processed like any other samples. The pause keeps its position and its length; denoise may lower the noise floor inside it.
- Breaths, laughs, filler words and hesitations are part of the take and are left in place.
- The whole clip measures below `ABSOLUTE_SILENCE`: gain 0 dB, the chain succeeds. Silence never produces `E_DSP` (§11.3).
- Non-finite samples in input → `DspError::NonFinite` → `E_DSP`.
- Clipping source audio: limiter output never exceeds the ceiling; no de-clip is attempted.

**Acceptance.** Output integrated loudness within ±1 LU of `TARGET_LOUDNESS` (PS §20.1): `loudness_target.rs` on five synthetic and three speech fixtures, and independently by `verify_mp4.py`. No click at block boundaries: `chain_no_clicks.rs` asserts the sample-to-sample step across every 1 s block boundary is at most 2x the local RMS (assumption, M1.2). Duration and alignment (INV-10), `duration_preserved.rs`: `out48.len() == pcm48.len()` for every fixture and for random lengths including non-multiples of the block and frame sizes (proptest); the regression layout 0-5 s speech, 5-10 s silence, 10-20 s speech comes out as 20 s with speech, silence and speech at the same positions (never 15 s); an impulse train through the chain stays within `ALIGN_TOLERANCE`; a clip that is 95% silence and a clip of pure digital silence both return the input length. Chain at most 4 s for the reference clip on R1 (PS §20.2).

**Contingency (PS §20.4, TE-13).** Denoise exceeds the 4 s budget on R1 → `ChainConfig.denoise = false` (high-pass + loudness + limiter only); decided at M1.

**Seams (not built now).** User-supplied music with ducking (P1): a `mix_bgm` step after `run_chain` and a `BgmTrack` record; it does not exist in code. Future versions may introduce explicit user-controlled timeline editing; the MVP deliberately does not implement automatic timeline modification, and no type, argument, message or flag is reserved for it.

---

## 19. Scene Graph, Typography and Renderer

**Model: a deterministic scene is compiled on the CPU into a backend-neutral `DisplayList` for any output time; Vello on WebGPU draws the list to an overlay texture; one wgpu pass composites it over the rotated, cropped source frame. Rejected: DOM/CSS overlays captured to canvas (not deterministic, not exportable frame-accurately), layout in JavaScript (two implementations of one rule), Canvas2D as the primary path (PS §9.8 keeps it as the fallback).**

### 19.1 Coordinate system (`safe_area.rs`, the only place)

- Logical canvas: 1080 x 1920 logical pixels (lp). All layout is in lp. Output scale = `profile.width / 1080` (Creator 1.0, Free 2/3, Preview 1/2).
- Safe area insets (assumption, E-2): top 250, bottom 420, left 60, right 120.
- `EVENT_ZONE`: y 860-1180. `CAPTION_ZONE`: y 1220-1500. `WATERMARK_ANCHOR`: bottom-left of the safe area.

### 19.2 Interfaces

```rust
// offcut-scene
pub struct SceneInput<'a> { pub transcript: &'a Transcript, pub events: &'a [DetectedEvent], pub edit: &'a EditState,
                            pub clip: &'a ClipInfo, pub profile: &'a ExportProfile }
pub fn build_scene(input: SceneInput<'_>) -> Result<Scene, SceneError>;
impl Scene {
    pub fn duration(&self) -> DurMs;                        // == clip.duration, always
    pub fn frame_count(&self) -> u32;                       // ceil(duration_ms * OUTPUT_FPS / 1000); 2,700 for 90 s
    pub fn crop(&self) -> CropRect;                         // constant per scene at MVP
    pub fn frame_at(&self, t: TimeMs) -> DisplayList;        // pure function of (scene, t)
    pub fn summary(&self) -> ChangeSummary;
}
pub struct CropRect { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }   // in source display pixels
pub struct DisplayList { pub cmds: Vec<DrawCmd> }                        // Serialize: the Canvas2D seam
pub enum DrawCmd {
    FillRect   { rect: Rect, radius: f32, color: Rgba },
    FillPath   { path: Vec<PathEl>, color: Rgba },
    GlyphRun   { font: FontId, size: f32, glyphs: Vec<PositionedGlyph>, fill: Rgba, stroke: Option<Stroke>,
                 transform: Affine, text: String },                     // text kept for the Canvas2D seam
    PushLayer  { opacity: f32, clip: Option<Rect> },
    PopLayer,
}
// offcut-render
impl Renderer {
    pub async fn new(canvas: web_sys::OffscreenCanvas, width: Px, height: Px) -> Result<Renderer, RenderError>;
    pub fn resize(&mut self, width: Px, height: Px) -> Result<(), RenderError>;
    pub fn render(&mut self, frame: &web_sys::VideoFrame, crop: CropRect, rotation: Rotation,
                  list: &DisplayList) -> Result<(), RenderError>;       // result is on the canvas
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_to_image(&mut self, source: &RgbaImage, crop: CropRect, rotation: Rotation,
                           list: &DisplayList) -> Result<RgbaImage, RenderError>;   // golden tests
}
// offcut-wasm-render/src/session.rs
impl RenderSession {
    pub async fn open(clip: ClipInfo, source: JsRandomAccess) -> Result<RenderSession, JsValue>;
    pub fn set_scene(&mut self, input: JsValue /* SceneInput by value */) -> Result<(), JsValue>;
    pub fn attach_canvas(&mut self, canvas: web_sys::OffscreenCanvas, width: Px, height: Px) -> js_sys::Promise;
    pub fn render_frame(&mut self, frame: &web_sys::VideoFrame, t: TimeMs) -> Result<(), JsValue>;
    pub fn frame_count(&self) -> u32;
    pub fn summary(&self) -> JsValue;
}
```

### 19.3 Build and frame evaluation

```text
build_scene:
  crop      = framing::crop_rect(clip, edit.crop_offset)
  words     = effective words (edits applied, hidden removed, number spans replaced by Quantity.display)
  times     = word and event timestamps exactly as in the transcript (no conversion; INV-5)
  chunks    = captions::chunk(words, style)                    // §19.4
  tracks    = for each enabled event: events::<kind>::layout(event, style) → keyframe tracks
  watermark = profile.watermark ? watermark::layout() : none
frame_at(t):
  cmds = []
  for chunk visible at t:   captions::draw(chunk, t)           // active word, KeywordPop scale
  for event visible at t:   events::<kind>::draw(track, t)     // anim::eval(track, t) with easing
  watermark → cmds
  scale all cmds by output scale
framing::crop_rect(clip, offset):                              // target aspect 9:16
  a = display_w / display_h
  a > 9/16  → w = display_h * 9/16; x = (display_w - w)/2 * (1 + offset); y = 0; h = display_h      // landscape, square
  a <= 9/16 → h = display_w * 16/9; y = (display_h - h)/2; x = 0; w = display_w                     // portrait
```

### 19.4 Captions and styles (`styles.rs`; visual values are (assumption), reviewed in E-2)

```text
captions::chunk: start a new chunk when
   sentence boundary | gap to previous word >= CHUNK_GAP (350 ms)
   | characters would exceed style.max_chars_per_line * style.max_lines | words == style.max_words
chunk visible from first word start - 80 ms to last word end + 120 ms, clamped so chunks never overlap
active word = word with start <= t < end; drawn in style.active_color
KeywordPop word: scale 1.0 → 1.25 → 1.10 over 180 ms from its start, style.pop_color
```

| Style | Font | Size (lp) | Case | Line limits | Treatment | Motion |
|---|---|---|---|---|---|---|
| Clean (default) | Inter 700 | 64 | As spoken | 18 chars x 2 lines, 5 words | White fill, 6 lp dark stroke, active word accent | Ease-out cubic, 120 ms |
| Bold | Inter 900 | 84 | Upper | 12 chars x 2 lines, 3 words | White fill, 10 lp stroke, active word accent, word-by-word pop | Back-out, 140 ms |
| Tech | JetBrains Mono 700 | 56 | As spoken | 22 chars x 2 lines, 5 words | Rounded dark panel at 85% opacity, green active word, block cursor | Linear, 80 ms |

Same layout rules for all three (PS §12.2). Fonts are OFL-licensed and embedded in `offcut-scene/assets/fonts/`; `LICENSES.md` lists them; `cargo deny` plus a CI grep assert no other font files exist.

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

**Seams (not built now).** Colors/fonts customization and brand presets (P1-P2): `StyleSpec` is already data. Face-tracking reframe (P1): `Scene::crop()` becomes `crop_at(t)`. Other aspect ratios: constants in `safe_area.rs`. The Canvas2D backend exists only as the contingency above.

---

## 20. Preview Player

**Model: the export renderer at half resolution, driven by the audio clock; the clock time is the recording time, so the source frame, the scene and the audio are all read at the same `TimeMs` with no mapping step. The preview therefore keeps the original pauses, speaking rhythm and duration. Rejected: `HTMLVideoElement` with an overlay canvas (a second renderer that differs from the export; no drift control), a pre-rendered preview file (kept as the PS §9.8 fallback).**

### 20.1 Interfaces

```ts
// usecases/control-preview.ts
export function attach(canvas: HTMLCanvasElement, out48: Float32Array): Promise<void>;
export function play(): Promise<void>;
export function pause(): Promise<void>;
export function seek(at: TimeMs): Promise<void>;
// workers/render/video-source.ts
export class VideoSource {
  constructor(demuxer: DemuxerHandle, info: ClipInfo);
  frameAt(t: TimeMs): VideoFrame | null;          // latest decoded frame with pts <= t; null while decoding
  frameAtBlocking(t: TimeMs): Promise<VideoFrame>; // export only: waits for decode, never returns a stale frame
  prefetch(from: TimeMs): void;
  close(): void;                                  // closes every queued frame and the decoder
}
// workers/render/preview-loop.ts
export function runPreview(session: RenderSession, source: VideoSource,
                           clock: () => TimeMs, isStopped: () => boolean): Promise<PreviewStats>;
export type PreviewStats = { frames: number; late: number; maxDriftMs: DurMs };
```

### 20.2 Algorithm

```text
main thread: AudioBufferSourceNode(out48 at 48 kHz) → destination
   audioMs = (ctx.currentTime - startCtxTime) * 1000 + startOffset
   every CLOCK_SYNC_INTERVAL = 250 ms: previewClock({audioMs, epochMs})
worker clock(): audioMs + (performance.timeOrigin + performance.now() - epochMs)

runPreview, per requestAnimationFrame in the worker:
   t = clock(); if t >= clip duration → resolve (ended)
   n = floor(t * OUTPUT_FPS / 1000); if n == lastN → return
   tn = FrameIdx(n).to_time_ms()                      // output time == source time (INV-5)
   frame = source.frameAt(tn)  ?? lastFrame           // late frame: reuse and count
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

**Model: an offline, frame-accurate loop in `render.worker` over the original timeline: for every output frame, take the source frame at that frame's own timestamp, render it with the scene to the WebGPU `OffscreenCanvas`, wrap it as a `VideoFrame`, encode H.264 with WebCodecs, encode `out48` (same duration as the source audio) to AAC, and write a faststart MP4 with the Rust muxer straight to OPFS. There is no audio trimming stage before AAC, no video trimming stage before H.264 and no time map. Rejected: `MediaRecorder` (real-time, not frame-accurate), FFmpeg.wasm (PS §9.8), server rendering (PS §11).**

### 21.1 Interfaces

```ts
// workers/render/export-loop.ts
export function runExport(a: { session: RenderSession; source: VideoSource; profile: ExportProfile;
                               out48: Float32Array; sink: OpfsSink; onProgress: (done: number, total: number) => void;
                               isCancelled: () => boolean }): Promise<{ bytes: Bytes } | { cancelled: true }>;
// workers/render/encoders.ts  [ONLY]
export const VIDEO_ENCODE_LADDER: readonly VideoLadderEntry[];
export const AAC_ENCODE_CONFIG: AudioEncoderConfig;
export function pickVideoConfig(profile: ExportProfile): Promise<VideoEncoderConfig>;   // first supported entry, else E_ENCODE_VIDEO
// workers/render/opfs-sink.ts
export class OpfsSink { static open(path: string): Promise<OpfsSink>; writeAt(offset: Bytes, data: Uint8Array): void;
                        close(): Promise<void>; abort(): Promise<void>; }
```

```rust
// offcut-mp4/src/mux.rs
pub trait MuxSink { fn write_at(&mut self, offset: Bytes, data: &[u8]) -> Result<(), IoError>; }
pub struct VideoTrackSpec { pub width: Px, pub height: Px, pub avcc: Vec<u8>, pub frame_count_hint: u32 }
pub struct AudioTrackSpec { pub sample_rate: Hz, pub channels: u32, pub asc: Vec<u8> }
impl<S: MuxSink> Mp4Muxer<S> {
    pub fn new(sink: S, video: VideoTrackSpec, audio: AudioTrackSpec) -> Result<Mp4Muxer<S>, MuxError>;
    pub fn add_video_sample(&mut self, data: &[u8], frame: FrameIdx, is_keyframe: bool) -> Result<(), MuxError>;
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
| Video bitrate | Creator 8,000,000 b/s; Free 4,000,000 b/s; Preview not encoded | (assumption, TE-4) |
| Audio | `mp4a.40.2`, 48,000 Hz, 2 channels (mono duplicated), 160,000 b/s | (assumption, TE-4) |
| Largest output | 90 s x (8,000,000 + 160,000) / 8 = 91.8 MB | derived |
| MP4 | brands `isom`/`mp42`; video timescale 30,000 with sample delta 1,000 (exact 30 fps CFR); audio timescale 48,000; chunks of 0.5 s; `moov` before `mdat` | |

### 21.3 Algorithm

```text
exportClip(exportId, token, out48):
  claims  = verify_token(token, PUBLIC_KEY)                    // Err → E_ENTITLEMENT_INVALID
  profile = export_profile(Some(claims), now)
  session.set_scene(scene input with profile); session.attach_canvas(exportCanvas, profile.width, profile.height)
  vcfg = pickVideoConfig(profile); venc = VideoEncoder(vcfg); aenc = AudioEncoder(AAC_ENCODE_CONFIG)
  sink = OpfsSink.open("exports/tmp/<exportId>.mp4"); mux created on the first video chunk (needs avcC)
  N = session.frame_count()                                    // ceil(clip duration x 30 / 1000): 2,700 for a 90 s clip
  for n in 0..N:
     if isCancelled() → abort (close frames, encoders; sink.abort()) → {cancelled}
     t     = FrameIdx(n).to_time_ms()                          // output time == source time (INV-5)
     frame = await source.frameAtBlocking(t)                   // waits for decode; never skips
     session.render_frame(frame, t)
     vf = new VideoFrame(exportCanvas, { timestamp: FrameIdx(n).to_micros(), duration: 33_333 })
     while venc.encodeQueueSize > ENCODE_QUEUE_MAX (4): await dequeue event
     venc.encode(vf, { keyFrame: n % 60 == 0 }); vf.close()
     onProgress(n + 1, N)
  await venc.flush()
  audio: pad out48 with zeros to N * 1600 samples (48,000 / 30; less than one frame of padding, and no sample
         is ever removed); feed AudioData in 1,024-sample frames; await aenc.flush()
  encoder outputs → mux.add_video_sample / add_audio_sample (in arrival order per track)
  size = mux.finalize(); sink.close(); move tmp → "exports/<exportId>.mp4"
  return { opfsPath, summary: session.summary(), stageTimings }

Mp4Muxer layout: ftyp | free(MOOV_RESERVE = 256 KiB) | mdat …   finalize(): write moov into the reserved space,
   pad with free; moov larger than the reserve → MuxError::MoovOverflow (cannot occur at ≤ 90 s; guarded by test)
AAC priming: the first audio chunk's timestamp/duration and the encoder's reported delay become an edit list (elst)
   so that audio presentation time 0 equals video frame 0 (TE-4 verifies offset ≤ one video frame)
```

### 21.4 Rules and edge cases

- **Frame accounting.** Every output frame `0..N` is rendered and encoded exactly once; the loop never drops or duplicates an output frame, and it runs from the first instant of the recording to the last. Source frames may repeat (VFR, source below 30 fps) or be skipped (source above 30 fps) only because of the frame-rate conversion to 30 fps; none is skipped or repeated because of what the audio contains.
- **Duration.** Output duration equals source duration to within one video frame, for any amount of silence in the clip. `timeline-preserved.spec.ts` and verifier checks 5 and 8.
- **Frame ownership.** Each `VideoFrame` (decoded or canvas-made) is closed by the code that obtained it, in a `finally` block. `export-loop.ts` keeps a live-frame counter asserted to be 0 at the end (dev builds).
- **Capture method.** `new VideoFrame(canvas)` after the render call is method A. TE-3 compares it with method B (`copyTextureToBuffer` readback) and picks one; the choice is local to `export-loop.ts`.
- **Encoder failure mid-export.** The export restarts from frame 0 with the next ladder entry; partial output is deleted.
- **GPU device lost.** `E_GPU_LOST`; the user retries; no partial resume.
- **Encoder emits reordered frames.** The muxer writes composition offsets (`ctts`) when chunk timestamps are not monotonic.
- **Render cache.** Key = SHA-256 over (source file hash prefix, `Transcript`, events, `EditState`, `ProfileKind`, `RENDERER_VERSION`), computed in `start-export.ts` through `hash_api`. A hit re-downloads the stored file under its original `ExportId` and sends no new receipt.
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
| `GET /healthz` | none | Liveness; touches no DB | none | 60/min per IP | n/a |
| `POST /auth/magic-link` | none | Create link, send email; always 202 (no account enumeration) | 1 kB | 3 per 15 min per email; 10/h per IP | New request invalidates earlier unused links for that email |
| `POST /auth/verify` | none | Consume link; create user if new; set refresh cookie; return `SessionResponse` | 1 kB | 10/min per IP | Token is single-use |
| `POST /auth/refresh` | refresh cookie + CSRF header | Rotate refresh token; return `SessionResponse` | 0 | 30/min per IP | Previous token accepted for 30 s (multi-tab), then reuse revokes the session |
| `POST /auth/logout` | refresh cookie + CSRF header | Revoke session; clear cookie | 0 | 30/min per IP | Idempotent |
| `GET /me` | bearer | `MeResponse` | none | 60/min per user | n/a |
| `GET /entitlement` | bearer | Fresh `EntitlementResponse` | none | 60/min per user | n/a |
| `POST /usage/receipts` | bearer | Record one export; return fresh token | 1 kB | 30/min per user | `export_id` primary key; repeats return the same shape and change nothing |
| `POST /billing/checkout` | bearer | Hosted-checkout URL for an `Offer` | 1 kB | 10/min per user | none |
| `GET /billing/portal` | bearer | Customer-portal URL | none | 10/min per user | n/a |
| `POST /billing/webhook` | provider signature over raw body | Apply subscription events | 64 kB | 120/min per IP | `provider_event_id` primary key; replays return 200 and change nothing |
| `POST /events` | none | Analytics batch, at most 50 events | 16 kB | 60 requests/min per IP | None; duplicates tolerated |
| `POST /notify-me` | none | Waitlist / unsupported-platform email | 1 kB | 5/h per IP | Primary key (email, wanted) |
| `POST /account/delete` | bearer | Delete account (§22.7) | 0 | 3/h per user | Idempotent |
| `GET /account/export` | bearer | `AccountExport` JSON | none | 6/h per user | n/a |

`server/tests/no_media_routes.rs` walks the router and asserts: no route has a body limit above 64 kB; `multipart/*`, `application/octet-stream`, `video/*` and `audio/*` requests receive 415 on every route.

### 22.2 Sessions and CSRF

- **Magic link.** 32 random bytes, base64url, sent once by email as `<APP_ORIGIN>/auth/callback?token=…`. Only the SHA-256 hash is stored. `MAGIC_LINK_TTL = 15 min` (assumption, M1.7). Single use.
- **Access token.** Ed25519-signed `{sub, sid, exp}`; `ACCESS_TOKEN_TTL = 15 min` (assumption, M1.7); kept only in `auth-store` memory; sent as `Authorization: Bearer`. Never written to storage.
- **Refresh token.** 32 random bytes; stored only as a hash (PS §17); cookie `offcut_rt; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=5184000` (60 days; assumption, M1.7); rotated on every use.
- **CSRF.** The two cookie-authenticated routes require header `X-Offcut-Csrf: 1` and an `Origin` header equal to `APP_ORIGIN`. All other authenticated routes use the bearer token and ignore cookies.
- **CORS.** The API emits no CORS headers. Browsers reach it only same-origin through the Vercel rewrite.
- **Email normalization.** Trim + lower-case in `auth/magic_link.rs`; `users.email` keeps the address as given (PS §16).

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
6. Until step 5, `entitlement.effective()` = token remaining minus outbox length, floored at 0.
7. The server never rejects a receipt for exceeding a limit: the export has already happened. A Creator account above `CREATOR_FAIR_USE_PER_MONTH` produces one `fair_use_exceeded` warning log line per day (hashed user id); no behaviour changes (PS §11 "soft").

Limits are conversion mechanisms, not security controls (PS §11). No server logic trusts the client beyond counting what it reports.

### 22.5 Billing

```rust
// billing/provider.rs
#[async_trait]
pub trait BillingProvider: Send + Sync {
    async fn create_checkout_url(&self, user: UserId, email: &str, offer: Offer, return_url: &str) -> Result<String, BillingError>;
    async fn create_portal_url(&self, customer_id: &str) -> Result<String, BillingError>;
    async fn cancel_subscription(&self, subscription_id: &str) -> Result<(), BillingError>;
    fn verify_signature(&self, headers: &HeaderMap, raw_body: &[u8]) -> Result<(), BillingError>;
    fn parse_event(&self, raw_body: &[u8]) -> Result<Option<BillingEvent>, BillingError>;   // None = irrelevant type
}
pub enum BillingEventKind { Activated, Renewed, PastDue, CancelScheduled, Canceled, Refunded }
pub struct BillingEvent { pub provider_event_id: String, pub user_id: UserId, pub customer_id: String,
                          pub subscription_id: String, pub kind: BillingEventKind, pub interval: BillingInterval,
                          pub current_period_end: UnixSecs, pub occurred_at: UnixSecs }
```

- **Checkout.** Hosted checkout by full-page navigation. `user_id` travels as the provider's custom/passthrough data and returns in every event (TE-9). Return URLs: `<APP_ORIGIN>/account?checkout=success|cancel`.
- **Offers (`billing/offers.rs`, the only place).** `Offer → env price id → (Plan::Creator, BillingInterval)`. `CreatorAnnualFounding` (E-5 presale, PS §19) is accepted only while `FOUNDING_OFFER_ENABLED=true`. Amounts (15 USD/month, 144 USD/year, 99 USD/year founding; PS §11, §19) are configured at the provider and displayed from `web/src/config/pricing.ts`; the server holds no amounts.
- **Event application (one transaction with the dedupe insert, C-19).** Events older than `subscriptions.last_event_at` are ignored.

| From | Event | To | Fields set |
|---|---|---|---|
| none, canceled | `Activated` | active | all provider ids, interval, `current_period_end`, `cancel_at_period_end = false` |
| active, past_due | `Renewed` | active | `current_period_end` |
| active | `PastDue` | past_due | none |
| active, past_due | `CancelScheduled` | unchanged | `cancel_at_period_end = true` |
| active, past_due | `Canceled` | canceled | `current_period_end` as reported |
| any | `Refunded` | canceled | `current_period_end = occurred_at` |

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
| `export_done` | `total_ms`, `profile`, `word_edits`, `events_kept`, `events_disabled`, `style`, `crop_adjusted: bool`, `from_cache: bool` | Activation (PS §10); E-7 keep rate; E-10 edit count; P1 promotion triggers (PS §9.5) |
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

`DATABASE_URL`, `APP_ORIGIN`, `PORT`, `LOG_LEVEL`, `ENTITLEMENT_SIGNING_KEY`, `ACCESS_TOKEN_SIGNING_KEY`, `MAIL_API_KEY`, `MAIL_FROM`, `BILLING_API_KEY`, `BILLING_WEBHOOK_SECRET`, `BILLING_PRICE_CREATOR_MONTHLY`, `BILLING_PRICE_CREATOR_ANNUAL`, `BILLING_PRICE_CREATOR_ANNUAL_FOUNDING`, `FOUNDING_OFFER_ENABLED`, `TRUSTED_PROXY_HOPS`. A missing or malformed variable aborts startup with the variable's name (never its value).

### 22.9 Free-tier behaviour (external facts: verified 2026-10, re-verified by TE-11)

- The service sleeps after 15 minutes without inbound traffic and takes about one minute to wake.
- **Keep-warm.** The uptime monitor that PS §20.1 requires calls `GET /api/v1/healthz` every 5 minutes. One always-on instance uses at most 24 x 31 = 744 h of the 750 free instance hours per month (derived).
- **Client tolerance.** `apiClient.wake()` at app start; per-attempt timeout `API_TIMEOUT = 10 s`; retries while total wait is within `API_COLD_START_BUDGET = 70 s` (C-15). Import, processing, preview and review never call the API.
- **Webhooks.** Delivered to the Render host directly. A request that hits a sleeping service may time out; the merchant's retry delivers it (TE-9 confirms the retry schedule).
- **Boot.** `sqlx::migrate!` runs at startup; the retention purge runs at startup and every 6 h in-process.
- **Memory.** DB pool `max_connections = 5`; no caches. Budget 512 MB (TE-11).

**Acceptance.** All `server/tests/*` pass against a real Postgres in CI. p95 handler time under 150 ms warm, excluding provider calls (assumption, M2.4). Cold request completes within `API_COLD_START_BUDGET` in `api-cold-start.spec.ts` (simulated delay) and in TE-11 (real).

**Contingency.** TE-11 fails (cold start beyond 70 s, instance hours insufficient, or memory): move the same Docker image to another free container host; nothing else changes because the client only knows `/api/v1` on its own origin.

**Seams (not built now).** Pro plan (P1): a price id and a `Plan` variant. Pause subscription (P1): a `BillingEventKind` variant. Google sign-in (P1): one route. Cloud transcription (P1): new routes that would break `no_media_routes.rs` by design and therefore need this spec revised first.

---

## 23. Persistence

### 23.1 Local stores

**OPFS (`persistence/opfs.ts` is the only place that builds these paths)**

| Path | Contents | Written by |
|---|---|---|
| `clips/<clipId>/source` | The imported file, byte-identical | `media.worker` |
| `clips/<clipId>/out48.f32` | Enhanced audio (raw f32, 48 kHz mono), same length as the decoded source audio | main, after `measureProsody` |
| `exports/tmp/<exportId>.mp4` | Export in progress | `render.worker` |
| `exports/<exportId>.mp4` | Finished export (render cache) | `render.worker` |
| `models/<modelId>/<file>.part`, `models/<modelId>/<file>` | Model download in progress / verified | `models/download.ts` |

**IndexedDB database `offcut`, version 1 (`persistence/schema.ts` is the only place that names stores)**

| Store | Key | Value |
|---|---|---|
| `clips` | `clipId` | `{ clipId, createdAt, lastOpenedAt, clipInfo, sourceBytes, sourceHashPrefix }` (no file name, no path) |
| `transcripts` | `clipId` | `{ transcript, prosody }` |
| `events` | `clipId` | `DetectedEvent[]` |
| `edits` | `clipId` | `{ editState }` |
| `renderCache` | cache key (hex) | `{ clipId, exportId, opfsPath, bytes, createdAt }` |
| `entitlement` | `"current"` | `{ token, storedAt }` |
| `receiptOutbox` | `exportId` | `{ exportId, enqueuedAt, attempts }` |
| `meta` | string | `anonId`, `persistRequested` |

Every value is wrapped as `{ schemaVersion: number, value }`. `sourceHashPrefix` is SHA-256 over the first 4 MiB of the source plus its length.

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
- **Local migrations.** `db.ts` runs numbered upgrade functions in `onupgradeneeded`. A value with a `schemaVersion` newer than the code is treated as absent and deleted.
- **Server migrations.** Forward-only `sqlx` migrations run at boot. Each must be compatible with the previously deployed binary (add first, remove in a later release), because Render swaps instances without coordination.
- **Server retention.** `analytics/retention.rs` deletes `analytics_events` older than 90 days (PS §16), `magic_links` older than 1 day, `webhook_events` older than 90 days (assumption, TE-9: longer than the provider's retry horizon), revoked or expired `sessions`.
- **Backups.** None on a free database tier (assumption, TE-6). Accepted cost: subscription state is rebuilt from the merchant by replaying events; usage receipts and waitlist would be lost. Reversal: none inside the 0 USD constraint (PS A-10); a paid tier with backups needs a founder decision and a spec revision.

**Acceptance.** `restore.spec.ts`: reload mid-review returns to `ready` with edits intact and no re-transcription. `clear-local-data.spec.ts`: OPFS empty and stores empty afterwards. `receipt-outbox.test.ts`: entries survive reload and are removed only on a 2xx. Estimated server data at base-case traffic: 12,000 visitors x 30 events x 200 B = 72 MB before retention (derived from PS §13.2; per-visitor event count is an assumption, TE-6).

**Seams (not built now).** `BrandPreset`, `BgmTrack`, `VersionSet`, `CloudJobConsent` (P1) and `StyleProfile`, `Workspace` (P2) from PS §16 are new stores/tables; none exists.

---

## 24. Security and the Network Boundary

### 24.1 Complete network surface (closed list)

**From the browser**

| # | Destination | What | Constraints |
|---|---|---|---|
| 1 | App origin (`APP_ORIGIN`, Vercel) | `GET` static files; `/api/v1/*` | Only via `net/http.ts` and the two WASM loaders |
| 2 | Asset origin (host of `VITE_ASSET_BASE_URL`) | `GET`/`HEAD` model files, demo video, sample clip | No credentials, no query string, no request body; only via `net/asset-fetch.ts` and the demo `<video>` element |
| 3 | Merchant hosted checkout / portal | Top-level navigation to a URL returned by the API | Never fetched, framed or scripted |

Nothing else: no font CDN, no analytics vendor, no error tracker, no tag manager, no third-party script (PS §17). `config/allowlist-hosts.ts` lists hosts 1 and 2 and is the only such list; `scripts/check-hosts.mjs` fails the build when the bundle contains any other `http(s)://` host literal.

**From the server:** Postgres host; email API host; merchant API host. **Into the Render host directly:** merchant webhooks; uptime monitor; the Vercel proxy.

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

In-memory token buckets keyed by (route group, key), at most 50,000 keys with least-recently-used eviction. Loss at restart is accepted. `client_ip.rs` takes the address `TRUSTED_PROXY_HOPS` positions from the right of `X-Forwarded-For`, or the socket peer when the header is absent; TE-5 establishes the hop count behind the Vercel rewrite. Limit exceeded → 429 with `retry_after_secs`.

### 24.6 Integrity and supply chain

- Model files are SHA-256-verified against the bundled manifest before first use (PS §17).
- `cargo deny check` (licenses, advisories, bans) and `pnpm audit --prod --audit-level high` run in CI.
- The ONNX Runtime `.wasm` files are copied from the pinned npm package at build time and served from the app origin.

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
| P-1 | Video, audio, transcript, captions and project edits are not uploaded (PS §12.7, §15, §17) | No route accepts media or free text (§22.1); DTOs carry no free-text field except `email`; server cannot link media crates (§7); fetch exists in four files (§7); CSP `connect-src` (§24.3) | `privacy-network.spec.ts`; `no_media_routes.rs`; `events.types.test.ts` | Settings → "What leaves your device"; "Verify it yourself" note; J4 and J10 copy |
| P-2 | Only allowlisted hosts are contacted (PS §17) | `allowlist-hosts.ts`; CSP; `check-hosts.mjs` | `privacy-network.spec.ts` assertion 1 | Settings lists the two hosts |
| P-3 | Analytics carry only allowlisted names with numeric/enum properties; never file names, transcript or caption text, event parameters or free text (PS §17) | `AnalyticsEvent` has no `String` field; `deny_unknown_fields`; client `track()` accepts only the generated union | `events.types.test.ts` (type-level); `analytics_allowlist.rs` (server rejects extras and strings); `privacy-network.spec.ts` assertions 3 and 5 | Event list rendered on the settings page |
| P-4 | The "What leaves your device" page matches the allowlist exactly (PS §20.1) | Page renders `ANALYTICS_EVENT_DOCS`, generated from `analytics.rs` | `settings-what-leaves.spec.ts` compares rendered rows with `gen/api.ts` | The page itself |
| P-5 | No request during processing other than allowlisted beacons (PS §17) | Use-cases between import and `ready` import no `net` module (boundaries rule) | `privacy-network.spec.ts` assertion 4 | Processing feed copy "Everything is rendering on your computer" |
| P-6 | The model stays in the browser and is verified (PS §10 J4, §17) | Bundled manifest hashes; OPFS-only cache adapter | `download.test.ts`; `model-download.spec.ts` | Settings shows model version, cache size |
| P-7 | "Clear local data" removes clips, transcripts and render cache (PS §20.1) | §23.3 | `clear-local-data.spec.ts` | Settings button |
| P-8 | Account deletion removes server records within 30 days (PS §17) | §22.7 (immediate) | `account_delete.rs` | Account page |
| P-9 | "Export my account data" returns server-side records only (PS §17) | §22.7 | `account_export.rs` | Account page |
| P-10 | Analytics retained 90 days; not joined to users (PS §16) | No `user_id` column on `analytics_events`; purge task | `analytics_allowlist.rs` (schema assertion + purge) | Stated on the settings page |
| P-11 | File names and paths are never stored or sent (PS §15) | `clips` store has no name field; download name is generated | `privacy-network.spec.ts` assertion 3; `restore.spec.ts` inspects the store | Recent clips show date and duration only |
| P-12 | No public compliance claim or absolute "never leaves your device" wording before legal review (PS §17) | All copy in `messages.ts` | `messages.test.ts` fails on the banned phrases list ("never leaves", "GDPR", "DPDP", "CCPA", "SOC 2", "compliant") | Landing uses the PS §10 J1 and J4 wording |

### 25.2 `privacy-network.spec.ts`

```text
capture: every request from the page, all workers and any service worker (CDP Network domain), from first navigation
scenario: land → drop speech fixture → first-run model download → processing → preview 5 s → edit one word
          → toggle one event → sign in (fake API on the app origin) → export → download → open settings
assert:
 1 every request host ∈ allowlist-hosts.ts
 2 asset-origin requests: GET/HEAD only, no body, no query string, no cookie header
 3 no URL, header or body contains: any fixture transcript word of 5+ letters (case-insensitive), the edited
   replacement word, the fixture file name, or any 32-byte window sampled from the source file, the decoded
   PCM or the exported MP4 (20 random windows each)
 4 between clip_accepted and pipeline_done, and between export_started and export_done: only POST /api/v1/events
   (plus model GETs on first run)
 5 every /api/v1/events body parses as EventsBatch from gen/api.ts with no extra keys
 6 no WebSocket, WebRTC, EventSource or sendBeacon activity
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

**Seams (not built now).** Consented cloud fallback (P1, PS §12.6): per-clip consent prompt, per-export mode display on the settings page, legal review. It changes P-1 for consenting clips only and requires a revision of this section before any code.

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

**Fixture matrix.** Committed speech clips are recorded by the founder reading scripts in `fixtures/speech/README.md`. Derived and synthetic fixtures are produced by `fixtures/gen_fixtures.sh` (ffmpeg CLI, development tool only) and are not committed. `fixtures/manifest.json` maps each name to its expected outcome.

| Fixture | Properties | Expected | Used by |
|---|---|---|---|
| `speech/speech_60s_portrait.mp4` | Reference clip (PS §20.2): 1080x1920, 30 fps, H.264/AAC, about 150 words, two numbers, one 3-item list, one from-to, one emphasized word, two white-flash frames | Accepted; 4+ events | Most E2E suites, bench, verifier |
| `speech/speech_60s_landscape.mp4` | 1920x1080, same script | Accepted; crop control shown | `review-edit.spec.ts`, `framing.rs` |
| `speech/speech_20s_noevents.mp4` | No numbers or lists | Accepted; zero events; `NO_EVENTS_FOUND` | `pipeline-preview.spec.ts` |
| `speech/speech_20s_long_pause.mp4` | 0-5 s speech, 5-10 s silence (room tone only), 10-20 s speech; one white-flash frame in each speech part | Accepted; export is 20 s (never 15 s); words after the pause keep timestamps of 10 s or later; A/V in sync on both sides of the pause | `timeline-preserved.spec.ts`, `duration_preserved.rs` (its audio), verifier |
| `speech/speech_30s_sparse.mp4` | Five short phrases separated by four pauses of 3-6 s; under 20% of the clip is speech; one white-flash frame near each end | Accepted; export is 30 s; every pause keeps its length | `timeline-preserved.spec.ts`, verifier |
| `ok_vfr.mp4` | Reference re-encoded with variable frame timing | Accepted; CFR output | `demux_fixtures.rs`, corpus smoke |
| `ok_rotated_90.mov` | Landscape-coded with 90° rotation metadata | Accepted as portrait | `demux_fixtures.rs`, `golden_frames.rs` |
| `ok_60fps.mp4` | 60 fps | Accepted; 30 fps output | `demux_fixtures.rs` |
| `ok_90s_exact.mp4` | Exactly 90,000 ms | Accepted | `probe_rejections.rs` |
| `ok_moov_at_end.mp4` | `moov` after `mdat` | Accepted | `demux_fixtures.rs` |
| `ok_portrait_3x4.mp4` | 1080x1440 | Accepted; cropped to 9:16 | `framing.rs` |
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
| `rej_silent.mp4` | Valid video, silent AAC | `REJECT_NO_SPEECH` | `rejections.spec.ts` |
| oversize (generated in the test) | Sparse file over 500 MB | `REJECT_FILE_SIZE` | `rejections.spec.ts` |
| `bad_audio_payload.mp4` | Valid tables, corrupted AAC frames | `E_DECODE_AUDIO` | `failure-recovery.spec.ts` |
| `bad_video_payload.mp4` | Valid tables, corrupted H.264 after 2 s | `E_DECODE_VIDEO` | `failure-recovery.spec.ts` |
| `labeled/*.json` (30 or more) | Hand-labeled transcripts with expected events | Precision at least 0.90 | `precision_recall.rs`, `layout_safe_area.rs` |
| `golden/*.png` (14) | 3 styles x 4 event kinds, watermark on, watermark off | Similarity at least 0.98 | `golden_frames.rs` |
| Corpus (40+, external) | Real clips tagged vfr, rotated, noisy, accent (PS §9.6) | At least 90% export; rest rejected; none crash | `corpus/run_corpus.ts` |
| ASR accuracy set (20, external) | 10 US, 10 Indian/other English accents with reference text (E-10) | WER at most 12% | `corpus/run_corpus.ts --wer` |

---

## 27. E2E Testing and Independent Verification

### 27.1 Suites

| Suite | Asserts |
|---|---|
| `landing.spec.ts` | J1: hero copy, supported-browser line, "What leaves your device" link, drop zone and sample button present before any upload; `landing_view` sent; waitlist form posts to `/notify-me` |
| `capability.spec.ts` | J3: check finishes within 3 s; `capability_check` event carries enums only; supported state reaches the editor |
| `unsupported.spec.ts` | Each `UNSUPPORTED_*` (injected) shows its specific message, the supported list, the demo video and the email form (PS §9.3) |
| `model-download.spec.ts` | J4: progress and copy shown; interrupted download resumes from stored bytes; hash mismatch re-downloads; second session skips download |
| `rejections.spec.ts` | J5: every `REJECT_*` fixture shows its message with interpolated limits; `clip_rejected` carries only the code |
| `pipeline-preview.spec.ts` | J6-J7: feed shows only real detections; preview plays before sign-in; zero-event clip shows `NO_EVENTS_FOUND`; no long task over 100 ms |
| `preview-sync.spec.ts` | Drift at most 80 ms over 60 s; late frames at most 5% |
| `review-edit.spec.ts` | J8: word edit re-detects its sentence; toggles persist across re-detection by `EventId`; style switch; crop control only for landscape; no control named or behaving like "tighten pauses" / "remove silence" exists; clip duration and audio are identical before and after every edit |
| `signin.spec.ts` | J9/C-8: limits shown before sign-in; link flow in a second tab signs in the first via broadcast; expired and invalid links show their messages |
| `export-free.spec.ts` | J9-J12: counter visible before click; 720x1280 output with watermark passes the verifier; counter decrements; last-export warning before the third; fourth attempt shows the upgrade prompt and does not render |
| `export-creator.spec.ts` | 1080x1920 output without watermark passes the verifier; no prompt; render-cache hit sends no receipt |
| `timeline-preserved.spec.ts` | INV-5, INV-10 on `speech_20s_long_pause.mp4` and `speech_30s_sparse.mp4`: transcript word timestamps after a pause are not moved earlier; the caption for the first word after the 5 s pause appears at its spoken time in the preview; the preview ends at the source duration; the export has `ceil(duration x 30 / 1000)` frames (600 and 900); the verifier passes with `--source`, including check 8 at the start and at the end of each clip |
| `checkout.spec.ts` | J12/C-11: navigation to the provider URL; return polling activates Creator; timeout shows `E_BILLING_PENDING`; portal link opens |
| `cancel.spec.ts` | C-12 during import, processing, model download and export; temp files removed; state returns as specified; no receipt after a cancelled export |
| `restore.spec.ts` | C-13: reload at `ready` restores without re-transcription; reload mid-export discards the temp file; `clips` store holds no file name |
| `failure-recovery.spec.ts` | C-14: worker crash, GPU loss (injected), storage quota, decode errors → specific message, retry succeeds, second failure shows the persistent copy |
| `api-cold-start.spec.ts` | C-15: API delayed 45 s → waking copy after 3 s, request succeeds; API down → export with cached token completes and the receipt stays in the outbox |
| `privacy-network.spec.ts` | §25.2 |
| `settings-what-leaves.spec.ts` | PS §12.7 table present; event list equals the generated allowlist; mode "Local only"; model version and cache size shown |
| `clear-local-data.spec.ts` | OPFS and clip stores empty afterwards; refused while busy |

### 27.2 Independent verifier (`verify/verify_mp4.py`)

Uses only the `ffprobe`/`ffmpeg` command-line tools and NumPy. It imports nothing from `crates/` or `web/`, and no library that those use. Invocation: `verify_mp4.py <file> --profile free|creator --expected-duration-ms N [--source <fixture>]`. `N` is always the duration of the source clip: an export is never shorter than its recording.

| # | Check |
|---|---|
| 1 | Container is MP4 with exactly one video and one audio stream; `moov` precedes `mdat` (faststart) |
| 2 | Video: H.264, `yuv420p`, square pixels, no rotation tag; size is 1080x1920 (`creator`) or 720x1280 (`free`) |
| 3 | Constant frame rate: every frame-to-frame PTS delta equals 1/30 s; frame count equals `ceil(expected_duration_ms x 30 / 1000)` |
| 4 | Audio: AAC-LC, 48,000 Hz, 2 channels |
| 5 | Durations: video within one frame of expected (the source duration); audio within one AAC frame (21.3 ms) of video |
| 6 | Full decode of both streams reports zero errors |
| 7 | Loudness: integrated loudness (ffmpeg `ebur128`) within ±1 LU of -14 LUFS |
| 8 | A/V sync and timeline (with `--source`): audio offset by envelope cross-correlation against the source within 20 ms, measured separately over the first 5 s and the last 5 s, so a removed or shortened pause anywhere in between fails; video offset by white-flash frame positions within one frame |
| 9 | Watermark: normalized correlation of the watermark region with `watermark_mask_720.png` at least 0.5 for `free`; at most 0.2 for `creator` (thresholds are (assumption), M1.6) |
| 10 | Video bitrate at most 1.5x the profile target |
| 11 | Metadata: no title, comment, location or source-derived tags; no creation time copied from the source |

---

## 28. Milestone Plan

**Gating rules**
- M0, M1, M2 are the PS §9.9 milestones with the PS dates and gates. `M0.n`, `M1.n`, `M2.n` are this document's build steps inside them.
- A step is done only when every test named in its Definition of Done passes in CI, or the named measurement is committed under `bench/results/`.
- A failed experiment triggers its "If it fails" action (§37) before any dependent step starts.
- No P1 item starts before M2 passes. M3 and M4 contain no build work; they are the PS §20.5 decisions.

| # | Milestone | Depends on | Definition of Done | Experiments |
|---|---|---|---|---|
| M0.1 | Skeleton (week 1): workspace, CI, codegen, static shell on Vercel with §24.3 headers, API on Render with `/healthz`, analytics ingestion, capability check, landing page with waitlist | none | `ci.yml` green; `landing.spec.ts`, `capability.test.ts`, `analytics_allowlist.rs` pass; deployed preview reports `crossOriginIsolated === true`; `/api/v1/healthz` answers through the rewrite | TE-5, TE-6, TE-7, TE-11, TE-9 (onboarding starts); E-1 starts; E-8 beacon live |
| M0.2 | ASR spike (weeks 1-2): model from OPFS in `asr.worker`, word timestamps | M0.1 | `model-download.spec.ts`, `download.test.ts` pass; reference-clip ASR timings on R1 and R2 committed | E-3, TE-1, TE-2 |
| M0.3 | Render/encode spike (weeks 1-2): Vello in `render.worker`, frames → H.264 + AAC → MP4 | M0.1 | `mux_roundtrip.rs` passes; `verify_mp4.py` checks 1-6 pass on a spike export; render + encode timings on R1 and R2 committed | E-4, TE-3, TE-4 |
| M0.4 | Proof of concept (week 3): one clip end to end with captions, NumberReveal, Clean style | M0.2, M0.3 | `pipeline-preview.spec.ts` and `export-creator.spec.ts` pass on the reference clip locally on R1 and R2; `e2e-media.yml` runs them in CI | TE-10, TE-14. **M0 gate: E-1, E-3, E-4 (PS §9.9, §20.4)** |
| M1.1 | Ingest and rejections | M0.4 | `demux_fixtures.rs`, `probe_rejections.rs`, `rejections.spec.ts` | TE-12 |
| M1.2 | Audio enhancement chain, prosody | M0.4 | `loudness_target.rs`, `chain_no_clicks.rs`, `duration_preserved.rs`, `prosody.rs`, `ui.rs` | TE-13 |
| M1.3 | Normalization and detector, four kinds | M0.4 | `numbers_table.rs`, `numbers_prop.rs`, `sentences.rs`, `rules_table.rs`, `redetect.rs`, `determinism.rs` (detect); `precision_recall.rs` at 0.90 or above | E-10 |
| M1.4 | Scene: three styles, four event visuals, framing, watermark | M1.3 | `display_list_snapshots.rs`, `layout_safe_area.rs`, `framing.rs`, `determinism.rs` (scene), `golden_frames.rs` | none |
| M1.5 | Preview and review UI | M1.2, M1.4 | `pipeline-preview.spec.ts`, `preview-sync.spec.ts`, `review-edit.spec.ts`, `cancel.spec.ts`, `rpc.test.ts`, `clip-machine.test.ts`, `model-machine.test.ts`, `run-pipeline.test.ts` | none |
| M1.6 | Export, verifier, render cache | M1.4 | `export-creator.spec.ts` with all 11 verifier checks; `timeline-preserved.spec.ts`; `start-export.test.ts`, `export-machine.test.ts`, `blockers.test.ts` | E-4 re-measured |
| M1.7 | Backend: auth, entitlement, receipts, account; client sign-in and gate | M0.1 | `auth_flow.rs`, `entitlement_rules.rs`, `usage_receipts.rs`, `account_delete.rs`, `account_export.rs`, `rate_limit.rs`, `log_redaction.rs`, `no_media_routes.rs`, `token_roundtrip.rs`, `profile.rs`, `auth-machine.test.ts`, `entitlement.test.ts`, `http.test.ts`, `signin.spec.ts`, `export-free.spec.ts` | TE-8 |
| M1.8 | Billing | M1.7 | `webhook_idempotency.rs`, `billing_events.rs`, `checkout.spec.ts`; provider test-mode purchase updates `subscriptions` | TE-9. **M1 gate: all P0 working on dev machines (PS §9.9)** |
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
- [ ] No source file exceeds 400 lines excluding tests (assumption for reviewability, PS §18 "complexity debt"); enforced by `scripts/check-file-tree.mjs`, which also counts lines.

---

## 30. Performance Requirements

**Reference conditions (PS §20.2).** 60-second clip, 1080x1920 or 1920x1080, 30 fps, H.264, about 150 words; model cached. R1: 2021-class Windows laptop, 8 GB RAM, integrated GPU, Chrome stable, plugged in. R2: Apple M1, 8 GB, Chrome stable. Fixture: `speech_60s_portrait.mp4` and `speech_60s_landscape.mp4`. Measurement: `bench/device-bench.ts`, median and p90 of 10 runs.

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
| ASR median above 40 s on R1 | Smaller model; stronger caption editing; cloud fallback remains a P1 trigger | M0, re-checked at M2 |
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
| ASR | Model session and runtime | 700 MB |
| ASR | `pcm16` + `pcm48` + chain working copies (5.8 + 17.3 + 34.6 MB, derived) | 58 MB |
| ASR | App shell, `offcut_core.wasm` heaps in three workers | 200 MB |
| ASR | **Phase total** | **about 0.96 GB** |
| Render | `offcut_render.wasm` heap, fonts, scene | 150 MB |
| Render | Decoded frame queue: 8 x 1920 x 1080 x 4 B (derived) | 66 MB |
| Render | Overlay + target textures at 1080x1920 (2 x 8.3 MB, derived) and Vello atlases | 80 MB |
| Render | Encoder and decoder internals | 150 MB |
| Render | `out48` (17.3 MB, derived), app shell | 118 MB |
| Render | **Phase total** | **about 0.56 GB** |
| Any | **Peak (phases do not overlap, §12.4)** | **under 1.5 GB budget** |

Disk: source up to 500 MB (PS §9.4), model up to 150 MB (PS §10 J4), audio 17.3 MB, export up to 91.8 MB (§21.2) per clip; `quota.ts` enforces eviction (§23.3).

---

## 32. Observability and Diagnostics

- **Client, production.** The only telemetry is the allowlisted events of §22.6, batched by `analytics/client.ts` (flush every 10 s, on `visibilitychange`, and at 20 queued events; failures dropped silently). Stage timings come from `performance.now()` deltas in workers. No console output in production builds (`no-console` lint; dev logging is behind `import.meta.env.DEV` in `config/env.ts`). No third-party error tracker.
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
| `pnpm fixtures` | `fixtures/gen_fixtures.sh` |
| `pnpm corpus` | `corpus/fetch_corpus.sh` + `corpus/run_corpus.ts` |
| `pnpm bench:device` | `bench/device-bench.ts`, writes `bench/results/<device>-<date>.json` |
| `pnpm verify <file>` | `verify/verify_mp4.py` |

**CI pipeline, in order (`ci.yml` unless noted)**
1. Checkout; restore Cargo and pnpm caches; secret scan.
2. `cargo fmt --check`; `cargo clippy --workspace -D warnings`; `cargo deny check`.
3. `pnpm gen:types`; `scripts/check-gen-clean.sh`.
4. `cargo test --workspace` (Postgres service container for the server; software GPU adapter for `golden_frames.rs`).
5. `pnpm build:wasm`.
6. ESLint; `tsc --noEmit`; `check-file-tree.mjs`; `check-copy-codes.mjs`.
7. `vitest run`.
8. `vite build`; `check-hosts.mjs`.
9. Playwright non-media suites on Linux with Chrome: `landing`, `unsupported`, `signin`, `checkout`, `settings-what-leaves`, `api-cold-start`.
10. `e2e-media.yml` on `windows-latest` with Chrome stable: every other suite in §27.1, then `verify_mp4.py` on each export (TE-10).
11. On `main`: `deploy-api.yml` builds `server/Dockerfile`, pushes to GHCR, calls the Render deploy hook; the web build output is deployed to Vercel as a prebuilt deployment (no Rust toolchain on the host's builders).
12. After deploy: header check against the deployment URL (§24.3).

**Hosting configuration**

| Target | Configuration |
|---|---|
| Vercel (`web/vercel.json`) | Static output `web/dist`; headers of §24.3; rewrite `/api/v1/:path*` → `https://<render-host>/api/v1/:path*`; fallback rewrite of non-file paths to `/index.html`. No functions, no middleware |
| Render (`render.yaml`) | One web service, runtime `image` from GHCR, plan `free`, `healthCheckPath: /api/v1/healthz`, environment variables of §22.8 set in the dashboard (`sync: false`) |
| Postgres | External free-tier instance; TLS required; connection string in `DATABASE_URL` |
| Asset CDN | `scripts/upload-assets.sh` uploads content-hashed files; CORS for `APP_ORIGIN`; `Cache-Control: public, max-age=31536000, immutable` |
| Email | API key in Render; sender identity per TE-8 |
| Uptime monitor | `GET https://<render-host>/api/v1/healthz` every 5 min |

**Deployment consequences of the architecture**
- **Hosting plan.** Vercel's free Hobby plan is restricted to non-commercial use (verified 2026-10). Offcut sells subscriptions, so the Hobby plan does not cover the launched product. The founder has decided to deploy on Vercel's free plan and not to pay for hosting (PS A-10), so this is an accepted, open risk rather than a budget line. TE-5 records the plan terms before live checkout is enabled (E-5 presale, week 8); if the terms block the product, the only remedy is the reversal path of TDR-12 (another free static host), never a paid plan. The build output is host-agnostic: static files plus three settings (headers, `/api/v1` proxy, SPA fallback).
- **Deploy order.** Server first, then web. The server rejects unknown request fields, so a new client must never reach an old server. Additive response fields are safe (the client ignores unknown fields).
- **Version skew in open tabs.** A tab loaded before a deploy may request a hashed worker or WASM file that the new deployment no longer serves. `pool.ts` therefore preloads all four worker scripts and both WASM bundles into the HTTP cache at app start (immutable caching makes later instantiation offline-safe).
- **Stateless API.** Render may restart or sleep the instance at any time; nothing but rate-limit buckets is lost.
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
Constraints:             lints that apply; banned APIs; size limit (400 lines)
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

Callers:     crates/offcut-wasm-core/src/audio_api.rs::run_chain   (only caller)
Callees:     highpass, denoise, compressor, loudness, limiter modules of this crate
Concurrency: called on audio.worker inside one job; cancellation point per block
Performance: at most 4 s for the 60 s reference clip on R1 (PS §20.2, TE-13)
Privacy:     sees audio samples only; must not log
Constraints: no unwrap/expect/indexing; no allocation per block beyond the working buffer; file under 400 lines
Tests:
  crates/offcut-dsp/tests/duration_preserved.rs
    length_equals_input_fixtures (P1)          prop_length_equals_input (P1: random lengths, proptest)
    speech_silence_speech_stays_20s (P1, P3: 0-5 s speech, 5-10 s silence, 10-20 s speech; output is not 15 s)
    impulses_stay_aligned (P2)                 mostly_silent_clip_keeps_length (P1, P3)
    digital_silence_keeps_length (P1, P4)
  crates/offcut-dsp/tests/loudness_target.rs   (P4, P5)
  crates/offcut-dsp/tests/chain_no_clicks.rs   (P6; also Empty, NonFinite, P7, P8)
Definition of done: the three test files pass; clippy clean; file under 400 lines
```

---

## 35. Architectural Invariants

| # | Invariant | Test |
|---|---|---|
| INV-1 | The browser contacts no host outside `config/allowlist-hosts.ts` | `privacy-network.spec.ts` (1); `check-hosts.mjs`; CSP |
| INV-2 | No request URL, header or body contains media bytes, transcript or caption text, event parameters or a file name | `privacy-network.spec.ts` (3) |
| INV-3 | Import, processing, preview and review complete with the API unreachable | `api-cold-start.spec.ts` |
| INV-4 | The server has no route that accepts more than 64 kB or a media content type | `no_media_routes.rs` |
| INV-5 | `TIMELINE_PRESERVATION`. Offcut does not automatically remove, shorten, reorder or compress any portion of the recording's timeline. Source time equals output time: transcript, caption and event timestamps are the spoken times, and the export has `ceil(source duration x 30 / 1000)` frames | `timeline-preserved.spec.ts`; `verify_mp4.py` (3, 5, 8) |
| INV-6 | No `DetectedEvent` exists with confidence below its kind's threshold | `rules_table.rs`; assertion in `precision_recall.rs` |
| INV-7 | Equal scene inputs and renderer version give byte-identical display lists for every frame | `offcut-scene/tests/determinism.rs` |
| INV-8 | Every export has exactly `frame_count` frames at exactly 30 fps and passes the independent verifier | `verify_mp4.py` (3) in both export suites and the corpus run |
| INV-9 | Output size and watermark are functions of the verified entitlement token only, computed in `export_profile` | `offcut-entitlement/tests/profile.rs`; `export-free.spec.ts`; `export-creator.spec.ts` |
| INV-10 | `AUDIO_DURATION_PRESERVATION`. The audio enhancement pipeline does not change the duration of the recording: `out48.len() == pcm48.len()`, aligned within `ALIGN_TOLERANCE`; no sample is deleted or shifted because it is quiet | `duration_preserved.rs`; `timeline-preserved.spec.ts`; `verify_mp4.py` (5, 8) |
| INV-11 | Every obtained `VideoFrame` is closed; the live-frame counter is 0 after preview stop and after export | Dev-build counter asserted in `pipeline-preview.spec.ts` and `export-creator.spec.ts` |
| INV-12 | The ASR session and the render session never coexist | `run-pipeline.test.ts` (ordering of `unload` and `openSession`) |
| INV-13 | A usage receipt is stored at most once per `export_id`; a webhook is applied at most once per `provider_event_id` | `usage_receipts.rs`; `webhook_idempotency.rs` |
| INV-14 | Limits are visible before the action they gate; no upgrade prompt renders during processing, editing or rendering | `export-free.spec.ts` |
| INV-15 | All user-facing text and every number inside it come from `copy/messages.ts` and the generated limits | `messages.test.ts`; `react/jsx-no-literals` |
| INV-16 | Every state change goes through `transition()`; an illegal transition changes nothing | The four machine test files |
| INV-17 | The main thread performs no decode, inference, DSP, render or encode | Boundaries lint; long-task assertion in `pipeline-preview.spec.ts` |
| INV-18 | Analytics events have no free-text property and no user identifier | `events.types.test.ts`; `analytics_allowlist.rs` |
| INV-19 | Server logs contain no email, token, IP address or body | `log_redaction.rs` |
| INV-20 | A model file is used only after its SHA-256 matches the bundled manifest | `download.test.ts`; `model-download.spec.ts` |

---

## 36. What Not to Build

| Trap | Why |
|---|---|
| FFmpeg compiled to WASM | Rejected by PS §0 item 4 and §9.8: size, licensing review (c)(f). `ffmpeg` appears only as a development CLI in `fixtures/` and `verify/` |
| Local LLM for semantics | PS §9.2 (f); P2 seam only |
| Cloud transcription or cloud LLM, even "temporarily" | Breaks P-1 and the closed network list; P1 with consent and a spec revision (PS §12.6) |
| Silence detection for trimming, pause detection or classification (intentional, dramatic, hesitation, dead air), pause removal or shortening, breath or filler-word removal, "bad take" or mistake cutting, any automatic change to the recording's timeline | PS §8 principle 7 and PS §9.2: the user already chose the take; INV-5, INV-10. Not a disabled toggle, config flag or identity time map either: the concept is absent |
| A source-time/output-time mapping layer kept "for later" | One timeline exists (TDR-7). User-controlled timeline editing, if it is ever added, starts with a spec revision, not a dormant abstraction |
| Timeline editor, keyframe UI, sliders | PS §9.2 (d)(f); principle 3 allows toggles and one crop offset |
| Style DNA, brand kits, three versions, remix | PS §9.2 (d)(a)(e) |
| Bundled music; user music with ducking | PS §9.2 (c)(a); the second is P1 |
| Face-tracking reframe | PS §9.2 (f); P1 seam (`crop_at`) |
| More than four event kinds, or a plug-in system for events | PS §9.2 (a); a new kind is two files, no framework needed |
| User-inserted events | Rejected PS §12.4 (e) |
| B-roll, stock footage, avatars, scheduling, public API, native app, teams | PS §9.2 (d)(c) |
| Pro/Studio plan code, coupons, credit metering | PS §9.2 (a); flat pricing (PS §11) |
| Safari, Firefox, Linux or mobile workarounds | PS §9.2 (f); unsupported page only |
| Multi-language text shaping, i18n framework | PS §9.2 (f)(e); English only |
| 4K, 60 fps output, other aspect ratios | PS §9.4 (f)(e) |
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
| Third-party analytics, error trackers, tag managers, web fonts from a CDN | PS §17 and the closed network list |
| Admin dashboard, metrics UI | `scripts/metrics.sql` is enough until M4 |
| A/B testing framework | E-6 needs one enum and one branch |
| GraphQL, OpenAPI generators, tRPC | Fifteen routes; DTOs are generated from Rust by `ts-rs` |
| CSS-in-JS runtime, component library | CSP `style-src 'self'`; about 20 components |
| Storing file names "for the recent clips list" | Breaks P-11; date and duration are shown instead |
| Joining analytics to accounts | PS §16 requires consent; no consent UI exists |
| Resumable exports | An export of at most 90 s restarts faster than a resume protocol can be verified |

---

## 37. Experiments and Open Technical Decisions

PS experiments (`E-n`) keep their PS pass criteria; the "Method" column states the technical instrument this document provides.

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
| E-10 | ASR accuracy on accents and noise? (PS §19) | `corpus/run_corpus.ts --wer` on 20 clips; `export_done.word_edits` | WER at most 12%; median edits at most 8 per 60 s (PS) | M1.3 | Larger model, stronger edit UX, or narrower claims (PS) |
| TE-1 | Can the ASR runtime load only from OPFS, make zero third-party requests, and return word timestamps in a worker? | Spike in `asr.worker` under the production CSP; network capture | Zero requests outside §24.1; word timestamps for the reference clip; both backends run | M0.2 | Second runtime candidate behind `whisper-runtime.ts` (§16) |
| TE-2 | Does the runtime expose per-word confidence? | Inspect outputs on the reference clip | A probability per word is available at no more than 10% time cost | M0.2 | `Confidence(1.0)` for all words; detector scores ignore ASR confidence |
| TE-3 | Does Vello on WebGPU run in a worker on an `OffscreenCanvas`, with `VideoFrame` import, and which capture method is faster? | Spike: render 1,800 frames on R1/R2 with method A (canvas → `VideoFrame`) and B (texture readback); hidden-tab run | A correct frame every time; at least 30 frames/s render-only on R1; works with the tab hidden | M0.3 | Other capture method; then Canvas2D backend (§19) |
| TE-4 | Which H.264/AAC encoder configs work on R1/R2, and does the muxed file play everywhere? | `pickVideoConfig` report; `verify_mp4.py`; manual playback on two phones and two platform uploads | A ladder entry supported on both; verifier 11/11; A/V offset within one frame | M0.3, M2.5 | Next ladder entries; explicit silent pre-roll for AAC priming; bitrate change |
| TE-5 | Is the static host usable as specified: plan terms for a paid product, COOP/COEP headers, `/api/v1` rewrite, proxy timeout against a cold API, forwarded-IP header shape? | Read the current plan terms; deploy the shell; call a deliberately slow endpoint through the rewrite; inspect `X-Forwarded-For` | Terms permit the use (or the founder accepts the terms risk in writing; no paid plan, PS A-10); `crossOriginIsolated` true; a 60 s upstream delay survives the proxy; hop count known | M0.1; decision before week 8 | Reversal path of TDR-12; if only the timeout fails, the client calls `/healthz` until warm before any real request |
| TE-6 | Which free Postgres provider persists beyond 30 days, accepts 5 connections, and fits the data estimate? | Read current terms; run migrations; idle for a week; measure reconnect time | No expiry; reconnect under 2 s; storage quota at least 5x the §23.3 estimate | M0.1 | Other candidate; then any other free Postgres provider (no paid tier, PS A-10) |
| TE-7 | Which asset host gives free egress, HTTP Range, CORS that works under COEP, and files of 150 MB? | Upload a 150 MB file; ranged fetch from the deployed shell | 206 responses; CORS readable; no egress charge in terms | M0.1-M0.2 | Other candidate; then split the model into parts under the host's limit |
| TE-8 | Which email provider delivers magic links within its free quota, and does it require a verified sender domain? | Send 50 links to major mailbox providers; read quota terms | At least 95% inbox delivery within 30 s; quota at least 100 emails/day | M1.7 | Other candidate; prefer one that verifies a single sender address without a custom domain, because a domain registration is a cost outside PS A-10 |
| TE-9 | Which merchant of record onboards an India-based seller, supports hosted redirect checkout with passthrough data, a customer portal, and retries webhooks? | Start onboarding in week 1 (PS §18); test-mode purchase, renewal, cancel, refund; delay the webhook endpoint | Account approved within 2 weeks; all six `BillingEventKind`s observed; a timed-out webhook is retried | M0.1 (start), M1.8 | Second provider behind `BillingProvider` (PS §18) |
| TE-10 | Can CI run the media suites (WebGPU + H.264 + AAC encode) on a hosted Windows runner within free minutes? | Run `e2e-media.yml` | Suites pass headless; monthly minutes within the free allowance | M0.4 | Run on `main` and pre-release only; `pnpm e2e:device` locally |
| TE-11 | Does the API fit the Render free instance: memory, cold-start time, instance hours with keep-warm, image deploy, log retention? | Deploy; measure RSS, cold start over 10 sleeps, monthly hour use | RSS under 400 MB; cold start under 70 s; one always-on service fits the monthly hours | M0.1, M2.4 | Another free container host for the same image (§22.9) |
| TE-12 | Does `offcut-mp4` demux the real-clip corpus (edit lists, rotation, VFR, phone encoders)? | `corpus/run_corpus.ts` | At least 90% of clips demux and export | M1.1, M2.3 | JS demuxer behind the same interface (§15) |
| TE-13 | Does the denoiser fit the audio budget in WASM on R1? | `bench/device-bench.ts`, stage `audio_chain` | Chain at most 4 s for the reference clip | M1.2 | `ChainConfig.denoise = false` (PS §20.4) |
| TE-14 | Is peak memory within budget on R1 for a 90 s, 1080p, 60 fps input? | Memory measurement per phase (§31) | Peak under 1.5 GB; no tab crash in 10 runs | M0.4, M2.4 | Smaller queues; preview at 360x640; smaller model |

**Non-blocking decisions**
- Router and store libraries may be swapped; only `routes.tsx` and `state/` depend on them.
- Exact fonts, colors, event timings, watermark size and placement: tuned during E-2.
- Annual discount, Google sign-in, final product name (PS §19 minor): the name is one constant in `messages.ts`.

---

## 38. Technical Decision Records

**TDR-1. Client-only media pipeline with a thin server.** Chosen over server-side processing. Why: PS thesis (near-zero cost per export, privacy claim, flat pricing). Cost: the user's device matrix becomes the support surface; reach is limited to desktop Chromium. Reversal: the consented cloud fallback seam (§16); a full reversal is a different product.

**TDR-2. Rust/WASM core in two bundles, four workers, main-thread hub.** Chosen over a TypeScript core or a single WASM bundle. Why: one implementation of every output-affecting rule, testable natively; workers that do not render never load the renderer. Cost: two build targets; data crosses JS between stages. Reversal: merge the bundles (build config only); the hub can become SharedArrayBuffer-backed without changing message shapes.

**TDR-3. ONNX Runtime Web with a Whisper-family English model, files served from OPFS.** Chosen over hub loading, CPU-only WASM builds and cloud ASR. Why: WebGPU acceleration with a WASM fallback; word timestamps; network confinement. Cost: runtime size; dependence on a third-party runtime's worker and WebGPU behaviour. Reversal: `whisper-runtime.ts` is the only file that knows the runtime (TE-1).

**TDR-4. Rule-based detector with per-kind thresholds.** Chosen over LLM classification. Why: PS §12.4; deterministic, debuggable, measurable precision. Cost: limited recall; lexicon upkeep. Reversal: another producer of `Vec<DetectedEvent>` behind `RenderWorkerApi.detect`.

**TDR-5. Backend-neutral `DisplayList`, Vello on WebGPU as the backend.** Chosen over drawing directly with Vello or with Canvas2D. Why: layout is testable without a GPU (exact snapshots); the PS fallback becomes a second consumer instead of a rewrite. Cost: one intermediate representation; glyph runs carry redundant text for the fallback. Reversal: implement `Canvas2dBackend` (§19).

**TDR-6. WebCodecs plus an in-house Rust MP4 demuxer and muxer.** Chosen over FFmpeg.wasm and over JS container libraries. Why: PS §0 item 4 names a Rust muxer; one crate owns sample tables in both directions; no licensing review. Cost: real-world MP4 variety is a stabilization risk. Reversal: JS demuxer behind `SampleMeta` (TE-12); the muxer stays.

**TDR-7. One timeline, no `TimeMap`, no automatic timeline editing.** Chosen over pause tightening with a source-to-output time map (v1), and over keeping an identity map as a placeholder. Why: PS §8 principle 7 (the user provides the take; Offcut does not decide which parts to keep); with nothing cut, source time and output time are the same instant, so one type (`TimeMs`) is the simplest correct model and an identity map would have no consumer. Cost: a clip with dead air stays as long as it was recorded; the user re-records or trims elsewhere. Reversal: user-controlled timeline editing would reintroduce a mapping type through a spec revision; nothing is reserved for it now.

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
| The take is the user's: no silence, pause, breath, filler word or mistake is removed; the recording's timeline and duration are unchanged; timestamps stay on the original timeline (PS §8 principle 7, PS §9.2, PS §12.5, PS §20.1) | Timeline principles (header), §18, §20, §21, INV-5, INV-10, TDR-7, §36; `duration_preserved.rs`, `timeline-preserved.spec.ts`, verifier checks 5 and 8 |
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
| Risk: complexity debt (PS §18) | §2, §6, §7, §34; 400-line limit §29 |
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
3. Email sender verification without a registered domain (TE-8): a domain costs money and is outside PS A-10, so the provider must accept a single verified sender address; deliverability of magic links under that setup is unproven.
4. Mobile detection via `navigator.userAgentData.mobile` (§3) — E-8 data at M2.2.
5. `MAX_FILE_SIZE` read as 500,000,000 bytes; `INPUT_FPS_TOLERANCE`; `MIN_WORDS`; `MAX_RECENT_CLIPS` (§10.2) — M1.1, E-2, M2.1.
6. First video track used when several exist; fragmented MP4 rejected (§15.4) — TE-12.
7. Per-word ASR confidence (§16.4) — TE-2. Hallucinated words over silence are not filtered (§16.6); long mid-clip pauses are now always kept, so E-10 also counts hallucinated words and timestamp drift inside and after pauses of 3 s or more — E-10.
8. Non-English speech and multiple speakers are not detected at import; these two PS §9.4 constraints have copy but no rejection test (§15.4) — E-2, E-10.
9. Sentence segmentation constants (§17.2) — E-10.
10. Every detector score, threshold, window and lexicon (§17.4, §17.5); empty header for marker-only lists — M1.3, E-7.
11. Loudness target -14 LUFS and every DSP constant, including `ALIGN_TOLERANCE` and whether the denoiser's delay can be compensated to within it (§18.3) — M1.2, TE-13, E-2.
12. Safe-area insets, zones, style values, event timings, emoji fallback font (§19) — E-2, M1.4.
13. Watermark text, size, opacity, placement (§19.6; PS §19 minor) — alpha.
14. Golden similarity 0.98; verifier thresholds for watermark and sync (§19, §27.2) — M1.4, M1.6.
15. Preview size, queue length, late-frame and first-frame targets, seek target (§20) — M1.5.
16. Video and audio bitrates, keyframe interval, 2-channel audio, encoder ladder order (§21.2) — TE-4.
17. `CANCEL_TIMEOUT`; `API_TIMEOUT`; `API_COLD_START_BUDGET` (§14.3, §22.9) — M1.5, TE-11.
18. Magic-link, access-token and refresh-token lifetimes; all rate-limit values in §22.1 — M1.7.
19. Free quota month = UTC calendar month (§22.3) — product confirmation at M1.7.
20. Creator until `current_period_end` for canceled and past-due; refund ends access immediately (§22.3, §22.5) — TE-9.
21. Analytics never joined to accounts at MVP (§22.6) — M2.2.
22. Retention of `webhook_events` (90 days) and `magic_links` (1 day); no database backups; 30 events per visitor (§23.3) — TE-9, TE-6.
23. Memory estimates and the 1.5 GB peak budget (§31) — TE-14.
24. App-shell size 400 kB; long-task limit 100 ms; `frame_at` 2 ms; `redetect` 50 ms; edit-to-preview 150 ms; API p95 150 ms (§13.5, §17, §19, §30) — M1.5, M2.4.
25. 400-line file limit (§29) — reviewed at each milestone.
26. Word edits replace or hide text only; no insertion or retiming (§10.6) — E-2.
27. PS experiments E-1 to E-10 and technical experiments TE-1 to TE-14: outcomes pending per §28.