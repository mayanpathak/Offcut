# v1implementation.md - Offcut V1 "Working skeleton": implementation plan

**Implements:** `buildplan.md` section 3 (V1), which is step M0.1 of `technicalspec.md` (TS) §28. Product references are to `product.md` (PS).

**Reader.** Whoever writes the V1 code, file by file. Everything needed to write a V1 file is in this document or in the TS section it names. Where the specs are silent and V1 needs an answer, this document decides and says so (section 2); those decisions are small and each is marked **(V1 decision)**.

**The one-sentence goal.** At the end of V1, every layer of Offcut exists, is connected and is deployed, with almost no behaviour: a landing page on Vercel that is cross-origin isolated, runs the capability check, and stores a waitlist email and analytics events through the Render API into Postgres, with CI checking all of it.

**The second goal, equally important.** V2 and V3 must be additive. No V1 file is restructured later: later versions fill in function bodies, add files and add rows to tables. Section 15 lists, file by file, what V2 and V3 add, and what V1 does today to make that possible.

---

## 0. Contents

1. Scope: what V1 is and is not
2. Decisions this document makes
3. Tools, accounts, secrets, environment variables
4. V1 file tree
5. Build order (12 steps, each with its own check)
6. `offcut-types`: every shared type
7. `offcut-api-types`: DTOs and the analytics allowlist
8. `offcut-wasm-core`: the smallest WASM bundle
9. Code generation contract
10. Server, file by file
11. Web, file by file
12. Root configuration, scripts, CI, hosting files
13. Tests, case by case
14. Experiments to run in V1
15. Hand-over: what V2 and V3 add to each V1 file
16. Exit checklist

---

## 1. Scope

### 1.1 In V1

| Area | What works at the end of V1 |
|---|---|
| Repository | Git repo; Cargo workspace; pnpm workspace; `docs/` holding the specs |
| Shared types | Every type of TS §10 defined in Rust and generated to TypeScript |
| Server | Axum service on Render with `GET /healthz`, `POST /events`, `POST /notify-me`; the full database schema; rate limiting; redacted logs; 90-day analytics purge |
| Web | Vite + React shell on Vercel with the production headers; routes `/`, `/app`, `/settings`; capability check; analytics client; waitlist form; "What leaves your device" table |
| WASM | `offcut_core.wasm` built in CI, shipped as a static asset, compiled in the browser at app start under the production CSP |
| CI and deploy | Lint, tests, codegen-drift check, file-tree check, host check, one Playwright suite; API image to GHCR and Render; prebuilt web deploy to Vercel; post-deploy header check |
| Experiments | TE-5, TE-6, TE-7, TE-11 measured; TE-9 onboarding started; E-1 landing public; E-8 beacon live |

### 1.2 Not in V1

No file import, no media decoding, no speech model, no workers that do work, no rendering, no export, no sign-in, no billing, no editor. `/app` shows a "not available yet" panel with the waitlist form. The drop zone and sample-clip button are visible but inactive.

### 1.3 The journeys V1 serves

- **J1 Land** (PS §10): hero, demo, supported-browser line, privacy line, link to "What leaves your device".
- **J3 Capability check**: green tick or a specific unsupported page.
- **C-1 App start** (TS §9), steps 1-3 of 6: `landing_view`, capability check, API wake. Session restore, model inspect and clip restore are later versions.
- **C-16 Unsupported browser**: reason, supported list, demo video, email form.

---

## 2. Decisions this document makes

The specs leave these open. Each is the smallest choice that keeps V2 and V3 additive. If one turns out wrong, change it here first.

| # | Decision | Why |
|---|---|---|
| D-1 | `UnsupportedReason` and `FailureStage` are Rust enums in `offcut-types/src/error.rs`, generated to TypeScript. `workers/protocol.ts` re-exports `FailureStage` | The analytics allowlist (Rust) needs both as enum props, and TS §10 names no home for them |
| D-2 | Plain enums serialize as `snake_case` strings (`"portrait"`, `"number_reveal"`, `"creator"`). The three code enums serialize as their code: `"E_*"`, `"REJECT_*"`, `"UNSUPPORTED_*"`. Struct fields keep their Rust names (`start_ms`, `export_id`) | Matches the SQL `CHECK` values, the analytics table and the `plan == "creator"` polling in TS C-11 |
| D-3 | `ApiErrorCode` has the 13 members listed in section 7.6 | TS §10.6 names the type but not its members |
| D-4 | `web/src/gen/domain.ts` and `api.ts` are each one file, written by an `#[ignore]`d test in each crate's `lib.rs`, run by `scripts/gen-types.sh` | TS wants two single files and branded unit types; `ts-rs` alone writes one file per type and unbranded aliases |
| D-5 | The generated files also carry `LIMITS` (from `limits.rs`) and `ANALYTICS_EVENT_DOCS` | INV-15 needs limits in copy; PS §12.7 needs the event list on the settings page |
| D-6 | IndexedDB database `offcut` version 1 creates all eight stores of TS §23.1 in V1, although V1 uses only `meta` | TS fixes the schema at version 1. Creating stores later would force a version 2 migration for no reason |
| D-7 | `server/migrations/0001_init.sql` holds the whole TS §23.2 schema in V1 | Same reason; the spec names one migration |
| D-8 | `web/src/workers/protocol.ts` exists in V1 with every type of TS §14.1-§14.2 and no runtime code. `net/http.ts` imports `AppFailure` from it as a type-only import | `AppFailure` lives there by spec and `http.ts` returns it. Having the whole protocol typed in V1 means V2 starts by implementing, not by designing |
| D-9 | `web/src/workers/pool.ts` exists in V1 with one function, `preload()`, which compiles `offcut_core.wasm` at app start | TS §33 already makes `pool.ts` preload WASM at app start. Doing it in V1 proves the WASM build, the bundling, the MIME type and CSP `wasm-unsafe-eval` in production in week 1 |
| D-10 | `net/http.ts` implements the full cold-start behaviour of TS C-15 in V1, with `http.test.ts` | The Render API sleeps. A waitlist signup that hits a cold API must still succeed, or E-1 undercounts |
| D-11 | `server/src/auth/mod.rs` and `auth/magic_link.rs` exist in V1 holding only `normalize_email()` | TS §22.2 puts email normalization there, and `/notify-me` needs it now. V6 fills in the rest of the file without moving anything |
| D-12 | New test file `server/tests/notify_me.rs` (added to the TS §5 tree) | TS §26 wants every route tested against a real Postgres and lists no file for `/notify-me` or `/healthz` |
| D-13 | `scripts/check-copy-codes.mjs` carries a `COPY_PENDING` list: codes whose copy is due in a later version. It fails if a code has neither copy nor a pending entry, and also if a pending code already has copy | The enums are complete in V1 but most messages belong to flows that do not exist yet. The list can only shrink and must be empty at V7 |
| D-14 | `GET /healthz` returns `{"ok":true,"version":"<git sha>"}` | Lets the web deploy wait until the new server is live (TS §33: server first, then web) |
| D-15 | `vite preview` serves the headers of `web/vercel.json`; `vite dev` serves only COOP and COEP | E2E runs against the production CSP locally and in CI; the CSP would break Vite's dev hot reload |
| D-16 | `EditorPage.tsx` exists in V1 as a "not available yet" panel plus the waitlist form | `/app` needs a page behind `CapabilityGate`; V2 replaces the body |
| D-17 | When `X-Forwarded-For` has fewer entries than `TRUSTED_PROXY_HOPS`, the leftmost entry is used | Direct calls to the Render host (uptime monitor, webhooks) pass one proxy fewer than calls through Vercel. TE-5 confirms the counts |

---

## 3. Tools, accounts, secrets, environment variables

### 3.1 Local tools

Pin the latest stable release of each on the day V1 starts and record it in `rust-toolchain.toml`, `package.json` (`packageManager`, `engines`) and the lock files (TS §3 "latest stable at M0.1, pinned").

| Tool | Used for |
|---|---|
| Rust stable + target `wasm32-unknown-unknown` | All Rust |
| `wasm-bindgen-cli` (same version as the `wasm-bindgen` crate), `wasm-opt` | `scripts/build-wasm.sh` |
| `cargo-deny`, `sqlx-cli` (Postgres, rustls) | Bans and licenses; migrations and the offline query cache |
| Node + pnpm; Playwright with `channel: "chrome"` | Web build and E2E |
| Docker | Building `server/Dockerfile` locally; a local Postgres |
| Git Bash | The `scripts/*.sh` files are POSIX shell; on Windows run them from Git Bash |

### 3.2 Accounts and what each must hand you

| Account | You need from it |
|---|---|
| GitHub | Repo; GHCR package `offcut-api` (public, so Render can pull it without credentials) |
| Vercel (free plan) | A project with root directory `web`, framework "Other"; `VERCEL_TOKEN`, `VERCEL_ORG_ID`, `VERCEL_PROJECT_ID` |
| Render (free web service) | A service created from `render.yaml`; its host name; a deploy hook URL |
| Postgres provider (TE-6) | A TLS connection string |
| Asset storage (TE-7) | A public base URL; CORS configured for the app origin |
| Uptime monitor | One HTTP monitor, every 5 minutes |
| Merchant of record (TE-9) | Nothing yet: start onboarding only |

### 3.3 Environment variables

**Render service** (TS §22.8). `config.rs` aborts on any missing or malformed variable and names it, never its value.

| Variable | V1 value |
|---|---|
| `DATABASE_URL` | From the Postgres provider |
| `APP_ORIGIN` | `https://<project>.vercel.app` |
| `PORT` | Set by Render |
| `LOG_LEVEL` | `info` |
| `ENTITLEMENT_SIGNING_KEY`, `ACCESS_TOKEN_SIGNING_KEY` | Two real Ed25519 keys, generated now, base64 of the 32-byte seed. Validated at boot in V1 so a bad key fails now, not in V6 |
| `MAIL_API_KEY`, `MAIL_FROM`, `BILLING_API_KEY`, `BILLING_WEBHOOK_SECRET`, `BILLING_PRICE_CREATOR_MONTHLY`, `BILLING_PRICE_CREATOR_ANNUAL`, `BILLING_PRICE_CREATOR_ANNUAL_FOUNDING` | The literal placeholder `unset-until-v6`. Nothing reads them before V6 |
| `FOUNDING_OFFER_ENABLED` | `false` |
| `TRUSTED_PROXY_HOPS` | From TE-5 |
| `GIT_SHA` | Baked into the image at build time (D-14) |

**Vercel / Vite build**

| Variable | Value |
|---|---|
| `VITE_ASSET_BASE_URL` | Base URL of the asset host chosen in TE-7 |

**GitHub Actions secrets:** `VERCEL_TOKEN`, `VERCEL_ORG_ID`, `VERCEL_PROJECT_ID`, `RENDER_DEPLOY_HOOK_URL`. `GITHUB_TOKEN` pushes to GHCR.

---

## 4. V1 file tree

Every path below is in the TS §5 tree (plus `server/tests/notify_me.rs`, D-12). Marks: **F** = final in V1, later versions do not touch it except to add entries. **P** = partial in V1; section 15 says what is added later. Files not listed here do not exist yet.

```text
<repo root>/
├── Cargo.toml                         P   workspace: 3 crates + server now
├── Cargo.lock
├── rust-toolchain.toml                F
├── clippy.toml                        F
├── deny.toml                          P   bans grow as crates arrive
├── package.json                       P   scripts of TS §33 whose targets exist
├── pnpm-workspace.yaml                F
├── pnpm-lock.yaml
├── render.yaml                        F
├── .gitignore                         F
├── .github/workflows/
│   ├── ci.yml                         P   E2E list grows
│   └── deploy-api.yml                 F
├── crates/
│   ├── offcut-types/
│   │   ├── Cargo.toml                 F
│   │   ├── src/lib.rs                 F   re-exports; generator test (D-4)
│   │   ├── src/units.rs               F
│   │   ├── src/ids.rs                 F
│   │   ├── src/limits.rs              F
│   │   ├── src/media.rs               F
│   │   ├── src/transcript.rs          F
│   │   ├── src/events.rs              F
│   │   ├── src/prosody.rs             F
│   │   ├── src/edit.rs                F
│   │   ├── src/profile.rs             F
│   │   ├── src/summary.rs             F
│   │   ├── src/stage.rs               F
│   │   ├── src/error.rs               F   ErrorCode, UnsupportedReason, FailureStage (D-1)
│   │   ├── tests/ui.rs                F
│   │   └── tests/ui/bare_ms_rejected.rs (+ generated .stderr)   F
│   ├── offcut-api-types/
│   │   ├── Cargo.toml                 F
│   │   ├── src/lib.rs                 F   re-exports; generator test
│   │   ├── src/auth.rs                F
│   │   ├── src/account.rs             F
│   │   ├── src/billing.rs             F
│   │   ├── src/usage.rs               F
│   │   ├── src/analytics.rs           F
│   │   └── src/errors.rs              F
│   └── offcut-wasm-core/
│       ├── Cargo.toml                 P
│       └── src/lib.rs                 P   panic hook, init, core_version()
├── server/
│   ├── Cargo.toml                     P
│   ├── Dockerfile                     F
│   ├── .sqlx/                         generated, committed
│   ├── migrations/0001_init.sql       F   whole schema (D-7)
│   ├── src/main.rs                    P
│   ├── src/config.rs                  F
│   ├── src/state.rs                   P   gains mailer and billing in V6
│   ├── src/router.rs                  P   3 of 15 routes
│   ├── src/error.rs                   P   variants grow
│   ├── src/headers.rs                 F
│   ├── src/client_ip.rs               F
│   ├── src/rate_limit.rs              F   all route groups defined now
│   ├── src/log.rs                     F
│   ├── src/auth/mod.rs                P   module declaration only (D-11)
│   ├── src/auth/magic_link.rs         P   normalize_email() only (D-11)
│   ├── src/analytics/mod.rs           F
│   ├── src/analytics/retention.rs     P   purges analytics now; 3 more tables in V6
│   ├── src/account/mod.rs             P   notify only
│   ├── src/account/notify.rs          F
│   ├── src/db/mod.rs                  F
│   ├── src/db/analytics_events.rs     F
│   ├── src/db/platform_waitlist.rs    P   delete-by-email added in V6
│   ├── tests/common/mod.rs            P   fakes added in V6
│   ├── tests/analytics_allowlist.rs   F
│   └── tests/notify_me.rs             F   (D-12)
├── web/
│   ├── package.json                   P
│   ├── index.html                     F
│   ├── vite.config.ts                 P   ORT copy step added in V2
│   ├── vitest.config.ts               F
│   ├── playwright.config.ts           P   projects grow
│   ├── tsconfig.json                  F
│   ├── eslint.config.js               F   every boundary rule of TS §7 now
│   ├── vercel.json                    F
│   ├── public/robots.txt              F
│   ├── src/main.tsx                   F
│   ├── src/App.tsx                    F
│   ├── src/routes.tsx                 P   3 of 7 routes mounted
│   ├── src/gen/domain.ts              generated, committed
│   ├── src/gen/api.ts                 generated, committed
│   ├── src/config/env.ts              F
│   ├── src/config/allowlist-hosts.ts  F
│   ├── src/copy/messages.ts           P   grows every version
│   ├── src/net/http.ts                P   bearer and refresh added in V6
│   ├── src/net/http.test.ts           P
│   ├── src/net/asset-fetch.ts         P   ranged fetch added in V2
│   ├── src/net/api-client.ts          P   3 of 15 route functions
│   ├── src/platform/capability.ts     F
│   ├── src/platform/capability.test.ts F
│   ├── src/platform/simd-probe.ts     F
│   ├── src/state/capability-store.ts  F
│   ├── src/state/machines/transition.ts F
│   ├── src/usecases/start-app.ts      P   3 of 6 steps
│   ├── src/usecases/submit-notify-me.ts F
│   ├── src/analytics/client.ts        F
│   ├── src/persistence/db.ts          P   meta helpers now; migrations hook ready
│   ├── src/persistence/schema.ts      F   all eight stores (D-6)
│   ├── src/wasm/load-core.ts          F
│   ├── src/workers/protocol.ts        F   types only (D-8)
│   ├── src/workers/pool.ts            P   preload() only (D-9)
│   ├── src/workers/render/encoders.ts P   constants only; pickVideoConfig in V2
│   ├── src/ui/pages/LandingPage.tsx   P   drop zone wired in V2
│   ├── src/ui/pages/EditorPage.tsx    P   placeholder (D-16)
│   ├── src/ui/pages/UnsupportedPage.tsx F
│   ├── src/ui/pages/SettingsPage.tsx  P   table only; completed in V8
│   ├── src/ui/components/CapabilityGate.tsx F
│   ├── src/ui/components/DropZone.tsx P   inactive
│   ├── src/ui/components/NotifyMeForm.tsx F
│   ├── src/ui/components/WhatLeavesTable.tsx F
│   ├── src/ui/styles/tokens.css       P
│   ├── src/ui/styles/pages.module.css P
│   ├── src/ui/styles/components.module.css P
│   ├── tests-e2e/helpers/fake-api.ts  P
│   ├── tests-e2e/helpers/fixtures.ts  P
│   └── tests-e2e/landing.spec.ts      F
├── scripts/
│   ├── build-wasm.sh                  P   one bundle now, two in V2
│   ├── gen-types.sh                   F
│   ├── check-gen-clean.sh             F
│   ├── check-file-tree.mjs            F
│   ├── check-hosts.mjs                F
│   ├── check-copy-codes.mjs           P   COPY_PENDING shrinks (D-13)
│   ├── check-external-facts.mjs       F
│   └── upload-assets.sh               P   demo media now; model files in V2
├── fixtures/speech/README.md          F   the reading scripts (needed by V2 on day one)
├── bench/results/.gitkeep             F
└── docs/
    ├── product.md
    ├── technicalspec.md
    ├── buildplan.md
    ├── v1/v1implementation.md         this file
    ├── v1/v1buildguide.md             F   step-by-step build order
    ├── v1/coding-prompts.md           F   the 30 coding prompts
    ├── v1/v1changelog.md              P   one entry per change made while building V1
    └── file-specs/TEMPLATE.md         F
```

---

## 5. Build order

Twelve steps. Each ends with a check you can run; do not start a step while the previous check fails. Days are a guide for a 50-hour week (PS A-2).

| # | Step | Produces | Check |
|---|---|---|---|
| S1 (day 1) | Repo and workspaces | Rename `documents/` to `docs/`; `git init`; root `Cargo.toml`, `rust-toolchain.toml`, `clippy.toml`, `deny.toml`, `package.json`, `pnpm-workspace.yaml`, `.gitignore`; `docs/file-specs/TEMPLATE.md` | `cargo metadata` and `pnpm install` succeed |
| S2 (day 1) | `offcut-types` | Section 6 | `cargo test -p offcut-types` (includes the trybuild compile-fail test); `cargo clippy -D warnings` |
| S3 (day 1) | `offcut-api-types` | Section 7 | `cargo test -p offcut-api-types` (docs-match-variants test) |
| S4 (day 1) | Code generation | `scripts/gen-types.sh`, `check-gen-clean.sh`; `web/src/gen/*.ts` | Run twice: second run produces no diff |
| S5 (day 2) | Server core | `config`, `log`, `error`, `headers`, `client_ip`, `rate_limit`, `state`, `db/mod`, migration, `main`, `router` with `/healthz` | `cargo run -p offcut-api` against a local Postgres; `curl /api/v1/healthz` returns 200 |
| S6 (day 2) | Server routes | `analytics/*`, `account/*`, `auth/magic_link.rs`, `db/analytics_events.rs`, `db/platform_waitlist.rs`; tests | `cargo test -p offcut-api` green against a real Postgres |
| S7 (day 3) | Web foundation | Vite, TS, ESLint, Vitest configs; `config/`, `gen/`, `workers/protocol.ts`, `state/`, `persistence/`, `net/`, `analytics/` | `pnpm --filter web exec tsc --noEmit`; ESLint clean; `http.test.ts` green |
| S8 (day 3) | Capability check | `platform/*`, `workers/render/encoders.ts`, `capability-store.ts` | `capability.test.ts` green |
| S9 (day 3-4) | WASM path | `offcut-wasm-core`, `scripts/build-wasm.sh`, `wasm/load-core.ts`, `workers/pool.ts` | `pnpm build:wasm`; the dev page compiles the module with no console error |
| S10 (day 4) | UI and use-cases | `messages.ts`, pages, components, styles, `start-app.ts`, `submit-notify-me.ts`, `routes.tsx` | `pnpm dev`: landing renders, check runs, form posts to the local API, rows appear in Postgres |
| S11 (day 4-5) | Checks, E2E, CI | The four `check-*` scripts; `landing.spec.ts`; `ci.yml` | `pnpm check && pnpm test && pnpm build && pnpm e2e` locally; `ci.yml` green on a pull request |
| S12 (day 5) | Deploy and measure | `Dockerfile`, `render.yaml`, `deploy-api.yml`, `vercel.json`, Vercel prebuilt deploy, uptime monitor, `upload-assets.sh` for the demo clips; TE-5, TE-6, TE-7, TE-11 | Section 16 exit checklist |

TE-6 (database) and TE-7 (asset host) can be run on day 1-2 in parallel with coding: S5 needs a database choice and S10 needs the asset host for the demo video.

---

## 6. `offcut-types`: every shared type

**Crate rules** (TS §2, §7): depends only on `serde`, `ts-rs`, `uuid` (features `serde` only: no `v4`, no `v7`, because pure crates must not pull in randomness). No I/O, no clock, no other `offcut-*` crate. Non-test code has no `unwrap`, `expect`, `panic!` or slice indexing.

**Derive rules** (TS §10): every type derives `Clone, Debug, PartialEq, Serialize, Deserialize, TS`. Integer unit types and id types add `Copy, Eq, PartialOrd, Ord, Hash` and `#[serde(transparent)]`. The three `f32` units (`Confidence`, `Lufs`, `Dbfs`) add `Copy, PartialOrd` only. Inner fields of unit types are private.

### 6.1 `units.rs` [ONLY]

```rust
pub struct TimeMs(u32);       // a position on the recording's timeline, in ms (the only timeline, INV-5)
pub struct DurMs(u32);        // a length in ms
pub struct Micros(i64);       // WebCodecs timestamps only
pub struct FrameIdx(u32);     // output frame number at OUTPUT_FPS
pub struct SampleCount(u64);
pub struct Hz(u32);
pub struct FpsMilli(u32);     // frames per second x 1000
pub struct Px(u32);
pub struct Bytes(u64);
pub struct BitsPerSec(u32);
pub struct Confidence(f32);   // 0.0..=1.0
pub struct Lufs(f32);
pub struct Dbfs(f32);
pub struct UnixSecs(i64);
pub struct ExportCount(u32);
pub struct UsdCents(u32);

pub struct Span { pub start: TimeMs, pub end: TimeMs }   // start <= end
```

| Function | Contract |
|---|---|
| `pub const fn new(v) -> Self` and `pub const fn get(self) -> inner` on every integer unit | `const` so `limits.rs` can use them |
| `Confidence::new(v: f32) -> Option<Confidence>` | `None` when not finite or outside `0.0..=1.0` |
| `Lufs::new(v: f32)`, `Dbfs::new(v: f32)` -> `Option<Self>` | `None` when not finite |
| `Span::new(start: TimeMs, end: TimeMs) -> Option<Span>` | `None` when `start > end` |
| `FrameIdx::to_time_ms(self) -> TimeMs` | `n * 1000 / 30`, integer division |
| `FrameIdx::to_micros(self) -> Micros` | `n * 1_000_000 / 30` |
| `TimeMs::to_micros(self) -> Micros` | `ms * 1000` |
| `TimeMs::from_micros_floor(m: Micros) -> TimeMs` | Floor division; negative input clamps to 0 |
| `TimeMs::checked_add(self, d: DurMs) -> Option<TimeMs>`; `TimeMs::duration_since(self, earlier: TimeMs) -> Option<DurMs>` | The only arithmetic between a position and a length |

There is no `From<u32>` for any unit and no second timeline type. Deserializing `Confidence` validates the range (a custom `Deserialize` that goes through `new`).

### 6.2 `ids.rs` [ONLY]

```rust
pub struct ClipId(Uuid);      // v4, created on import (in the browser)
pub struct ExportId(Uuid);    // v7, created per export; the usage-receipt idempotency key
pub struct UserId(Uuid);
pub struct AnonId(Uuid);      // per-browser analytics id
pub struct EventId(u64);      // stable hash, TS §17.6
pub struct WordIdx(u32);
pub struct SentenceIdx(u32);
pub struct WordRange { pub start: WordIdx, pub end: WordIdx }   // half-open
pub struct JobId(u32);
```

Each has `new` and `get`. No function here generates an id: generation needs randomness or a clock, so it happens in TypeScript (`crypto.randomUUID()`) and in the server.

### 6.3 `limits.rs` [ONLY]

Exactly the 21 constants of TS §10.2, copied verbatim, with their PS references in comments. Nothing else lives in this file. The generator (section 9) exports them as `LIMITS`.

### 6.4 `media.rs`, `transcript.rs`, `events.rs`, `prosody.rs`, `edit.rs`, `profile.rs`, `summary.rs`, `stage.rs`

Copy the definitions of TS §10.3-§10.6 verbatim. V1 adds only serialization attributes (D-2) and constructors:

| File | Types | V1 notes |
|---|---|---|
| `media.rs` | `Rotation`, `Orientation`, `ContainerKind`, `VideoCodec`, `AudioCodec`, `ProbeInfo`, `VideoTrackInfo`, `AudioTrackInfo`, `ClipInfo`, `RejectReason` | `RejectReason` has 14 variants, each with `#[serde(rename = "REJECT_...")]`: `REJECT_CONTAINER`, `REJECT_CORRUPT`, `REJECT_NO_VIDEO`, `REJECT_NO_AUDIO`, `REJECT_MULTI_AUDIO_TRACK`, `REJECT_VIDEO_CODEC`, `REJECT_HEVC`, `REJECT_AUDIO_CODEC`, `REJECT_DURATION`, `REJECT_FILE_SIZE`, `REJECT_RESOLUTION`, `REJECT_FRAME_RATE`, `REJECT_DECODE_UNSUPPORTED`, `REJECT_NO_SPEECH` |
| `transcript.rs` | `Word`, `Sentence`, `Unit`, `Quantity`, `NormalizedSpan`, `Transcript` | `Unit::Count { noun: String }` is the only variant with data |
| `events.rs` | `EventKind`, `ListItem`, `EventParams`, `DetectedEvent` | `EventParams` is `#[serde(tag = "kind", rename_all = "snake_case")]` |
| `prosody.rs` | `WordProsody`, `Prosody` | |
| `edit.rs` | `StyleId`, `CropOffset`, `EditState` | `CropOffset::new(v: f32) -> Option<CropOffset>`: `None` outside `-1.0..=1.0`. `EditState::default()`: no edits, no overrides, `StyleId::Clean`, offset `0.0` |
| `profile.rs` | `Plan`, `ProfileKind`, `ExportProfile` | |
| `summary.rs` | `ChangeSummary` | |
| `stage.rs` | `PipelineStage` | Variants in PS §20.2 order: `ProbeAudio, Asr, AudioChain, DetectScene, RenderEncode, Mux` |

`EntitlementClaims` is not here: it belongs to `offcut-entitlement`, which arrives in V2.

### 6.5 `error.rs`

```rust
pub enum ErrorCode { /* 29 variants, each #[serde(rename = "E_...")] */ }
pub enum UnsupportedReason { /* 11 variants, each #[serde(rename = "UNSUPPORTED_...")] */ }
#[serde(rename_all = "snake_case")]
pub enum FailureStage { ProbeAudio, Asr, AudioChain, DetectScene, RenderEncode, Mux,
                        Import, Model, Preview, Storage, Api }
impl From<PipelineStage> for FailureStage { /* one arm per stage */ }
```

`ErrorCode`, in TS §11.2 order: `E_MODEL_DOWNLOAD`, `E_MODEL_HASH`, `E_MODEL_STORAGE`, `E_STORAGE_QUOTA`, `E_STORAGE_IO`, `E_DECODE_AUDIO`, `E_DECODE_VIDEO`, `E_ASR_RUNTIME`, `E_ASR_OOM`, `E_DSP`, `E_GPU_INIT`, `E_GPU_LOST`, `E_ENCODE_VIDEO`, `E_ENCODE_AUDIO`, `E_MUX`, `E_WORKER_CRASH`, `E_NET_OFFLINE`, `E_NET_TIMEOUT`, `E_API_5XX`, `E_API_RATE_LIMITED`, `E_AUTH_LINK_INVALID`, `E_AUTH_LINK_EXPIRED`, `E_AUTH_SESSION_EXPIRED`, `E_AUTH_EMAIL_UNAVAILABLE`, `E_BILLING_UNAVAILABLE`, `E_BILLING_PENDING`, `E_ENTITLEMENT_INVALID`, `E_ENTITLEMENT_EXPIRED`, `E_INTERNAL`.

`UnsupportedReason`, in the check order of TS §13.2: `UNSUPPORTED_MOBILE`, `UNSUPPORTED_WEBCODECS`, `UNSUPPORTED_THREADS`, `UNSUPPORTED_WASM_SIMD`, `UNSUPPORTED_STORAGE`, `UNSUPPORTED_LOW_MEMORY`, `UNSUPPORTED_WEBGPU`, `UNSUPPORTED_H264_DECODE`, `UNSUPPORTED_H264_ENCODE`, `UNSUPPORTED_AAC_DECODE`, `UNSUPPORTED_AAC_ENCODE`.

### 6.6 `lib.rs`

Declares the twelve modules and re-exports their public items at the crate root. Holds two `#[cfg(test)]` items: serde round-trip tests for the three code enums (the string form is the contract), and the generator test of section 9.

### 6.7 Tests

| File | Asserts |
|---|---|
| `tests/ui.rs` | `trybuild::TestCases::new().compile_fail("tests/ui/*.rs")` |
| `tests/ui/bare_ms_rejected.rs` | Passing a bare `u32` where `DurMs` is expected does not compile; passing a `TimeMs` where `DurMs` is expected does not compile. The matching `.stderr` file is generated once with `TRYBUILD=overwrite` and committed |
| Unit tests in `units.rs` | `FrameIdx(30).to_time_ms() == TimeMs(1000)`; `FrameIdx(1).to_micros() == Micros(33_333)`; `Confidence::new(1.1)` is `None`; `Confidence::new(f32::NAN)` is `None`; `Span::new` rejects reversed bounds; `from_micros_floor` floors and clamps |

---

## 7. `offcut-api-types`: DTOs and the analytics allowlist

**Crate rules:** depends on `serde`, `ts-rs`, `offcut-types`. Definitions only. Every request struct has `#[serde(deny_unknown_fields)]`.

### 7.1 `auth.rs`, `account.rs`, `billing.rs`, `usage.rs`

Copy TS §10.7 verbatim. V1 compiles and generates all of them; the server uses only `NotifyMeRequest`, `EventsBatch` and `ApiError` in V1.

| File | Types |
|---|---|
| `auth.rs` | `MagicLinkRequest`, `VerifyRequest`, `SessionResponse` |
| `account.rs` | `MeResponse`, `Wanted` (`launch, safari, firefox, mobile, linux`), `NotifyMeRequest`, `AccountExport`, `ExportedUser`, `ExportedSession`, `ExportedReceipt` |
| `billing.rs` | `SubscriptionSummary`, `SubscriptionStatus` (`active, past_due, canceled`), `BillingInterval` (`monthly, annual`), `Offer` (`creator_monthly, creator_annual, creator_annual_founding`), `CheckoutRequest`, `UrlResponse` |
| `usage.rs` | `UsageReceiptRequest`, `EntitlementResponse` |

The three `Exported*` structs are named but not defined in TS §10.7. **(V1 decision)** `ExportedUser { email: String, created_at: UnixSecs }`, `ExportedSession { created_at: UnixSecs, last_used_at: UnixSecs, expires_at: UnixSecs }` (no token hashes, TS §22.7), `ExportedReceipt { export_id: ExportId, created_at: UnixSecs, plan_at_export: Plan }`.

### 7.2 `analytics.rs` [ONLY]: constants and prop enums

```rust
pub const MAX_EVENTS_PER_BATCH: usize = 50;      // TS §22.1
pub const MAX_DURATION_MS: u32 = 3_600_000;      // TS §22.6
pub const MAX_COUNT: u32 = 10_000;               // TS §22.6
```

All prop enums are `#[serde(rename_all = "snake_case")]`:

| Enum | Values |
|---|---|
| `HeroVariant` | `outcome, privacy` |
| `CheckResult` | `pass, fail` |
| `GpuVendor` | `intel, amd, nvidia, apple, qualcomm, other, unknown` |
| `MemoryBucket` | `lt4, gb4, gb8plus, unknown` |
| `Platform` | `windows, macos, linux, chromeos, android, other, unknown` |
| `DurationBucket` | `lt30, lt60, lte90` |
| `ClipSource` | `user, sample` |
| `DownloadOutcome` | `ok, failed, hash_mismatch` |
| `AsrBackend` | `webgpu, wasm` |
| `ReviewActionKind` | `word_edit, event_toggle, style, crop_offset` |
| `ExportedAs` | `free, creator` |
| `PostExportAnswer` | `yes, small_edits, no` |
| `PromptPlacement` | `after_export, limit_reached` |
| `PromptAction` | `shown, clicked, dismissed` |
| `SigninStep` | `link_requested, completed` |
| `CheckoutStep` | `started, returned_success, returned_cancel` |

### 7.3 `analytics.rs`: the event enum

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "name", content = "props", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnalyticsEvent {
    LandingView      { hero_variant: HeroVariant },
    CapabilityCheck  { result: CheckResult, unsupported_reason: Option<UnsupportedReason>,
                       gpu_vendor: GpuVendor, memory_bucket: MemoryBucket, platform: Platform },
    ClipAccepted     { duration_bucket: DurationBucket, orientation: Orientation, source: ClipSource },
    ClipRejected     { reject_reason: RejectReason },
    ModelDownload    { outcome: DownloadOutcome, duration_ms: u32, resumed: bool },
    StageTiming      { stage: PipelineStage, duration_ms: u32, asr_backend: Option<AsrBackend> },
    PipelineDone     { total_ms: u32, n_number: u32, n_list: u32, n_from_to: u32, n_keyword: u32 },
    PreviewPlayed,
    ReviewAction     { action: ReviewActionKind, event_kind: Option<EventKind>, enabled: Option<bool> },
    ExportStarted    { profile: ExportedAs },
    ExportDone       { total_ms: u32, profile: ExportedAs, word_edits: u32, events_kept: u32,
                       events_disabled: u32, style: StyleId, crop_adjusted: bool, from_cache: bool },
    ExportFailed     { error_code: ErrorCode, stage: FailureStage },
    JobCancelled     { stage: FailureStage },
    PostExportAnswer { answer: PostExportAnswer },
    UpgradePrompt    { placement: PromptPlacement, action: PromptAction },
    SigninStep       { step: SigninStep },
    CheckoutStep     { step: CheckoutStep },
    ClientError      { error_code: ErrorCode, stage: FailureStage },
    LocalDataCleared,
}

pub struct EventsBatch { pub anon_id: AnonId, pub events: Vec<AnalyticsEvent> }   // deny_unknown_fields

pub struct EventDoc { pub name: &'static str, pub description: &'static str }
pub const ANALYTICS_EVENT_DOCS: &[EventDoc] = &[ /* 19 entries, same order as the enum */ ];
```

Rules that make this an allowlist:

- No variant has a `String` field. Every prop is an enum, a `bool` or a `u32`.
- Optional props are `Option<T>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`.
- A unit variant serializes as `{"name":"preview_played"}` with no `props` key.
- `EventDoc.description` is one plain-language sentence per event, written for the settings page ("How long each processing stage took").

### 7.4 `analytics.rs`: one method

```rust
impl AnalyticsEvent {
    pub fn name(&self) -> &'static str;          // "landing_view", ... (exhaustive match, no wildcard)
}
```

The integer caps are enforced by the server (section 10.9), not here.

### 7.5 Unit tests in `analytics.rs`

- One sample of each of the 19 variants serializes to the JSON shape `{"name": ..., "props": ...}` and deserializes back equal.
- The set of `name()` values equals the set of `ANALYTICS_EVENT_DOCS` names, in the same order, with no duplicates.
- Deserializing an event with an unknown prop fails. Deserializing an unknown event name fails. Deserializing a string where an enum is expected fails.

### 7.6 `errors.rs`

```rust
pub struct ApiError { pub code: ApiErrorCode, pub retry_after_secs: Option<u32> }

#[serde(rename_all = "snake_case")]
pub enum ApiErrorCode {
    BadRequest, Unauthorized, Forbidden, NotFound, UnsupportedMediaType, PayloadTooLarge,
    RateLimited, LinkInvalid, LinkExpired, SessionExpired, EmailUnavailable,
    BillingUnavailable, Internal,
}
```

V1 produces `bad_request`, `not_found`, `unsupported_media_type`, `payload_too_large`, `rate_limited`, `internal`. The other seven are defined now so V6 adds handlers, not enum members (D-3).

### 7.7 `lib.rs`

Module declarations, re-exports, and the generator test of section 9.

---

## 8. `offcut-wasm-core`: the smallest WASM bundle

**Purpose in V1:** prove the path Rust to `wasm32` to `wasm-bindgen` to `wasm-opt` to Vite asset to browser compile, under the production CSP. V2 adds `media_api.rs`, `text_api.rs`, `audio_api.rs`, `hash_api.rs` beside it.

`Cargo.toml`: `crate-type = ["cdylib"]`; dependencies `wasm-bindgen`, `offcut-types`. Target features `+simd128` set for this crate's wasm build in `scripts/build-wasm.sh` (TS §3).

```rust
// src/lib.rs
#[wasm_bindgen(start)]
pub fn init();                    // installs the panic hook; safe to run more than once

#[wasm_bindgen]
pub fn core_version() -> String;  // env!("CARGO_PKG_VERSION")
```

**Panic hook contract** (TS §11.3). The hook converts a panic into a JS exception whose message starts with the literal `E_WORKER_CRASH`. It writes nothing to the console in release builds. In V2 the worker catches this, posts an `AppFailure` and is terminated; a WASM instance is never reused after a panic. V1 only installs the hook.

This crate contains no product rule and no branching on domain values (TS §2).

---

## 9. Code generation contract

`scripts/gen-types.sh` runs two ignored tests and nothing else:

```text
OFFCUT_GEN_OUT=web/src/gen/domain.ts  cargo test -p offcut-types      write_typescript -- --ignored
OFFCUT_GEN_OUT=web/src/gen/api.ts     cargo test -p offcut-api-types  write_typescript -- --ignored
```

Each test builds one string and writes it to `OFFCUT_GEN_OUT`. Output is deterministic: fixed order, `\n` line endings, no timestamps.

**`web/src/gen/domain.ts` contains, in this order:**

1. A header comment: generated, never edit, how to regenerate.
2. One branded alias per unit and id type, written by hand in the generator from a table of names:
   `export type TimeMs = number & { readonly __unit: "TimeMs" };`
   The `Uuid`-backed ids are branded strings: `export type ClipId = string & { readonly __unit: "ClipId" };`. `EventId` is a `u64`: it is generated as a branded `string` and serialized as a decimal string, because a JavaScript number cannot hold 64 bits. **(V1 decision)**
3. Every other type of section 6, from its `ts-rs` declaration, in module order.
4. `export const LIMITS = { ... } as const;` with one property per constant of `limits.rs`, each typed with its brand (`MAX_CLIP_DURATION: 90000 as DurMs`).
5. Const arrays of every code enum, for exhaustiveness checks and for `check-copy-codes.mjs`: `ERROR_CODES`, `REJECT_REASONS`, `UNSUPPORTED_REASONS`.

**`web/src/gen/api.ts` contains:** a header; `import type` lines for the domain types it references; every type of section 7; `export const ANALYTICS_EVENT_DOCS = [...] as const;`; `export const MAX_EVENTS_PER_BATCH = 50;`.

**Rules.** `as TimeMs`-style casts are allowed only in `gen/` and `workers/`; ESLint `no-restricted-syntax` rejects them elsewhere (TS §10.1). `scripts/check-gen-clean.sh` runs `gen-types.sh` and fails if `git diff --exit-code web/src/gen` reports a change. The two files are committed.

---

## 10. Server, file by file

**Package:** `offcut-api` in `server/`. **Model** (TS §22): one stateless Axum binary; Postgres is the only durable state; anything in memory may vanish at any restart.

**Dependencies in V1:** `axum`, `tokio`, `tower`, `tower-http`, `sqlx` (Postgres, rustls, `uuid`, `time`, `json`, `migrate`, offline cache), `serde`, `serde_json`, `thiserror`, `tracing`, `tracing-subscriber` (JSON), `uuid` (`v4`, `v7`, `serde`), `time`, `sha2`, `base64`, `ed25519-dalek`, `offcut-types`, `offcut-api-types`. Dev: `reqwest`. The server must never depend on a media crate (TS §7; enforced by `deny.toml`).

**Clock rule.** The server reads wall time through `time::OffsetDateTime::now_utc()` and monotonic time through `tokio::time::Instant`, so tests can pause time and the root `clippy.toml` ban on `std::time` clocks stays valid for the whole workspace.

### 10.1 `src/config.rs` [ONLY reader of the environment]

```rust
pub struct Config {
    pub database_url: String,
    pub app_origin: String,                       // scheme + host, no trailing slash
    pub port: u16,
    pub log_level: tracing::Level,
    pub entitlement_signing_key: ed25519_dalek::SigningKey,
    pub access_token_signing_key: ed25519_dalek::SigningKey,
    pub mail_api_key: String,  pub mail_from: String,
    pub billing_api_key: String,  pub billing_webhook_secret: String,
    pub billing_price_creator_monthly: String,
    pub billing_price_creator_annual: String,
    pub billing_price_creator_annual_founding: String,
    pub founding_offer_enabled: bool,
    pub trusted_proxy_hops: u8,
    pub git_sha: String,                          // D-14; defaults to "dev" when unset
}
pub enum ConfigError { Missing(&'static str), Malformed(&'static str) }   // carries the name, never the value
impl Config { pub fn from_env() -> Result<Config, ConfigError>; }
```

- Every variable of section 3.3 is required except `GIT_SHA`. Empty counts as missing.
- `app_origin` must parse as `https://host` (or `http://localhost:port` in development); otherwise `Malformed("APP_ORIGIN")`.
- `Debug` is implemented by hand and prints `<redacted>` for every key and secret (TS §24.2).
- Unit tests: a complete environment parses; each missing variable yields `Missing(<its name>)`; a bad key yields `Malformed`; the `Debug` output contains no secret value.

### 10.2 `src/log.rs` [ONLY tracing setup and redaction]

```rust
pub fn init(level: tracing::Level);                 // JSON to stdout; call once
pub fn request_span<B>(req: &http::Request<B>) -> tracing::Span;   // for tower_http::trace
pub fn user_tag(user_id: offcut_types::UserId) -> String;          // first 8 hex chars of SHA-256(user id)
```

Logged per request (TS §24.4): request id, method, **route template** (`axum::extract::MatchedPath`, never the raw path), status, latency. Never logged: email addresses, tokens, cookies, bodies, IP addresses, query strings. `user_tag` is unused until V6 and exists now so no handler ever formats a user id itself.

### 10.3 `src/error.rs`

```rust
pub enum AppError {
    BadRequest,
    NotFound,
    UnsupportedMediaType,
    PayloadTooLarge,
    RateLimited { retry_after_secs: u32 },
    Db(sqlx::Error),
    Internal,
}
impl axum::response::IntoResponse for AppError { /* status + Json(ApiError) */ }
impl From<sqlx::Error> for AppError;
pub fn json_rejection(r: axum::extract::rejection::JsonRejection) -> AppError;
```

| `AppError` | HTTP | `ApiErrorCode` |
|---|---|---|
| `BadRequest` | 400 | `bad_request` |
| `NotFound` | 404 | `not_found` |
| `UnsupportedMediaType` | 415 | `unsupported_media_type` |
| `PayloadTooLarge` | 413 | `payload_too_large` |
| `RateLimited` | 429 | `rate_limited`, with `retry_after_secs` and a `Retry-After` header |
| `Db`, `Internal` | 500 | `internal` |

`json_rejection` maps a body-size rejection to `PayloadTooLarge`, a wrong content type to `UnsupportedMediaType`, and every parse or validation failure (unknown field, unknown event name, wrong type) to `BadRequest`. Handlers take `Result<Json<T>, JsonRejection>` so this mapping is the only path. The response body never echoes the request. A `Db` error is logged with its kind, never with query parameters.

### 10.4 `src/headers.rs`

`pub fn layer() -> impl tower::Layer<...>`: adds `Cache-Control: no-store` and `X-Content-Type-Options: nosniff` to every API response (TS §24.3). The API emits no CORS header (TS §22.2).

### 10.5 `src/client_ip.rs` [ONLY client IP derivation]

```rust
pub fn client_ip(headers: &http::HeaderMap, peer: std::net::IpAddr, trusted_hops: u8) -> std::net::IpAddr;
```

- No `X-Forwarded-For` header, or `trusted_hops == 0`: return `peer`.
- Otherwise split the header on commas, trim, parse. Return the entry `trusted_hops` positions from the right (1 = last). Fewer entries than hops: return the leftmost (D-17). Any unparsable entry in the chosen position: return `peer`.
- Unit tests: one, two and three entries with hops 1 and 2; absent header; garbage entry; IPv6 entry; spaces.

### 10.6 `src/rate_limit.rs`

```rust
pub enum RouteGroup { Healthz, MagicLinkEmail, MagicLinkIp, Verify, Refresh, Logout, Me, Entitlement,
                      UsageReceipts, Checkout, Portal, Webhook, Events, NotifyMe, AccountDelete, AccountExport }
pub struct Limit { pub capacity: u32, pub window: std::time::Duration }
impl RouteGroup { pub const fn limit(self) -> Limit; }

pub enum RateKey { Ip(std::net::IpAddr), EmailHash([u8; 32]), User(offcut_types::UserId) }

pub struct RateLimiter { /* Mutex<Inner> */ }
impl RateLimiter {
    pub fn new(max_keys: usize) -> RateLimiter;                         // 50,000 (TS §24.5)
    pub fn check(&self, group: RouteGroup, key: RateKey, now: tokio::time::Instant) -> Result<(), u32>;
                                                                        // Err(retry_after_secs)
}
pub async fn by_ip(group: RouteGroup, State(state): State<AppState>, req: Request, next: Next)
    -> Result<Response, AppError>;                                      // middleware used through from_fn_with_state
```

The whole TS §22.1 table is encoded now, so V6 adds routes without touching this file:

| Group | Limit | Key | Live in V1 |
|---|---|---|---|
| `Healthz` | 60 / min | IP | yes |
| `Events` | 60 / min | IP | yes |
| `NotifyMe` | 5 / h | IP | yes |
| `MagicLinkEmail` | 3 / 15 min | email hash | V6 |
| `MagicLinkIp` | 10 / h | IP | V6 |
| `Verify` | 10 / min | IP | V6 |
| `Refresh`, `Logout` | 30 / min | IP | V6 |
| `Me`, `Entitlement` | 60 / min | user | V6 |
| `UsageReceipts` | 30 / min | user | V6 |
| `Checkout`, `Portal` | 10 / min | user | V6 |
| `Webhook` | 120 / min | IP | V6 |
| `AccountDelete` | 3 / h | user | V6 |
| `AccountExport` | 6 / h | user | V6 |

- **Algorithm.** Token bucket per `(group, key)`: `capacity` tokens, refilled continuously at `capacity / window`. A request takes one token. With none left, `Err(ceil(seconds until one token))`.
- **Memory bound.** At most `max_keys` buckets. Each bucket records a last-used sequence number; when full, the least recently used bucket is evicted. Loss at restart is accepted (TS §24.5).
- **Emails are never stored as keys.** The key is SHA-256 of the normalized address.
- **Unit tests:** capacity is honoured; the 61st request in a minute is refused with a plausible `retry_after`; tokens return after time advances (`tokio::time::pause` and `advance`); two IPs do not share a bucket; two groups do not share a bucket; eviction keeps the newest keys.

### 10.7 `src/state.rs`

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: std::sync::Arc<Config>,
    pub limiter: std::sync::Arc<RateLimiter>,
}
```

V6 adds `mailer: Arc<dyn Mailer>` and `billing: Arc<dyn BillingProvider>`. Nothing else is ever held in memory (TS §1: no state that must survive a restart).

### 10.8 `src/db/mod.rs`, `migrations/0001_init.sql`, `db/analytics_events.rs`, `db/platform_waitlist.rs`

```rust
// db/mod.rs
pub async fn connect(database_url: &str) -> Result<sqlx::PgPool, sqlx::Error>;   // max_connections = 5 (TS §22.9)
pub async fn migrate(pool: &sqlx::PgPool) -> Result<(), sqlx::migrate::MigrateError>;   // sqlx::migrate!("./migrations")

// db/analytics_events.rs
pub async fn insert_batch(pool: &PgPool, anon_id: AnonId, rows: &[(&'static str, serde_json::Value)])
    -> Result<(), sqlx::Error>;                              // one statement, one transaction
pub async fn purge_older_than(pool: &PgPool, days: u32) -> Result<u64, sqlx::Error>;

// db/platform_waitlist.rs
pub async fn upsert(pool: &PgPool, email_normalized: &str, wanted: Wanted) -> Result<(), sqlx::Error>;
                                                             // ON CONFLICT (email_normalized, wanted) DO NOTHING
```

- `0001_init.sql` is the SQL of TS §23.2, copied exactly: `users`, `magic_links`, `sessions`, `subscriptions`, `webhook_events`, `usage_receipts`, `analytics_events`, `platform_waitlist`, with their indexes and `CHECK` constraints (D-7). Migrations are forward-only and run at boot.
- `analytics_events.ts` is the server receive time (`DEFAULT now()`); the client sends no timestamp (TS §22.6).
- Queries use `sqlx::query!` so they are checked at compile time; run `cargo sqlx prepare --workspace` and commit `server/.sqlx/`. CI builds with `SQLX_OFFLINE=true`.

### 10.9 `src/analytics/mod.rs` and `analytics/retention.rs`

```rust
// analytics/mod.rs
pub fn routes() -> axum::Router<AppState>;                   // POST /events
async fn ingest(State(s): State<AppState>, body: Result<Json<EventsBatch>, JsonRejection>)
    -> Result<StatusCode, AppError>;                         // 204
fn within_bounds(e: &AnalyticsEvent) -> bool;

// analytics/retention.rs
pub fn spawn_purge_task(pool: PgPool);                       // once at startup, then every 6 h, in-process
pub async fn purge_once(pool: &PgPool) -> Result<u64, sqlx::Error>;
```

`ingest`, in order:

1. Parse failure of any kind: 400 (through `json_rejection`). The whole batch is rejected; nothing is stored.
2. `events.len()` is 0 or above `MAX_EVENTS_PER_BATCH`: 400.
3. Any event with `within_bounds` false: 400. Bounds: every `*_ms` field at most `MAX_DURATION_MS`; every count field (`n_*`, `word_edits`, `events_kept`, `events_disabled`) at most `MAX_COUNT`.
4. For each event: `name = event.name()`; `props` = the `props` member of its serialized JSON, or `{}` for a unit variant.
5. `insert_batch(pool, anon_id, rows)`; return 204.

Stored per event: `anon_id`, `name`, `props`, server time. Not stored and not logged: IP address, user agent, any user id, any client timestamp. Duplicates are tolerated (no idempotency key).

`purge_once` deletes `analytics_events` older than `ANALYTICS_RETENTION_DAYS`. V6 extends it to `magic_links`, `webhook_events` and dead `sessions` (TS §23.3).

### 10.10 `src/auth/magic_link.rs` (V1 part) and `src/account/notify.rs`

```rust
// auth/magic_link.rs
pub fn normalize_email(raw: &str) -> Option<String>;
// account/notify.rs
async fn notify_me(State(s): State<AppState>, body: Result<Json<NotifyMeRequest>, JsonRejection>)
    -> Result<StatusCode, AppError>;                         // 204
// account/mod.rs
pub fn routes() -> axum::Router<AppState>;                   // POST /notify-me (V6 adds /account/delete, /account/export)
```

- `normalize_email`: trim, lower-case. `Some` only when the result is 3 to 254 bytes, contains exactly one `@`, has a non-empty local part and a domain containing a dot, and holds no whitespace or control character. This is a sanity check, not validation: the proof of an address is a delivered email, which V6 provides.
- `notify_me`: `None` from `normalize_email` gives 400. Otherwise `upsert`, then 204. The response is the same whether or not the row already existed, so the endpoint cannot be used to test whether an address is on the list.
- Unit tests for `normalize_email`: case and whitespace folding; missing `@`; two `@`; no dot in the domain; over-length; embedded newline.

### 10.11 `src/router.rs`

```rust
pub fn build(state: AppState) -> axum::Router;
async fn healthz(State(s): State<AppState>) -> Json<serde_json::Value>;   // {"ok":true,"version":"<git sha>"}; touches no DB
async fn not_found() -> AppError;
```

**Route table in V1.** All paths are under `/api/v1`.

| Route | Auth | Body limit | Rate limit | Handler |
|---|---|---|---|---|
| `GET /healthz` | none | none | `Healthz` by IP | `router::healthz` |
| `POST /events` | none | 16 kB | `Events` by IP | `analytics::ingest` |
| `POST /notify-me` | none | 1 kB | `NotifyMe` by IP | `account::notify::notify_me` |
| anything else | | | | `not_found` (404 `not_found`) |

**Layers, outermost first:**

1. Request id (generated per request, returned as `X-Request-Id`).
2. `tower_http::trace` with `log::request_span`.
3. `tower_http::catch_panic::CatchPanicLayer`: a panic becomes 500 `internal`; the process continues (TS §11.3).
4. `headers::layer()`.
5. Media guard: a request whose `Content-Type` starts with `multipart/`, `video/` or `audio/`, or equals `application/octet-stream`, gets 415 on every route, before any handler runs (INV-4).
6. Default body limit 16 kB; per-route limits from the table.
7. Per-route rate limit.

The route table is written as one block of `.route(...)` calls in TS §22.1 order with the twelve V6 routes present as comments, so V6 uncomments and wires handlers.

### 10.12 `src/main.rs`

```text
main():
  1 config = Config::from_env()        error: print "config error: <NAME>" to stderr, exit 1
  2 log::init(config.log_level)
  3 pool = db::connect(&config.database_url)
  4 db::migrate(&pool)
  5 analytics::retention::spawn_purge_task(pool.clone())
  6 state = AppState { db: pool, config: Arc::new(config), limiter: Arc::new(RateLimiter::new(50_000)) }
  7 serve router::build(state) on 0.0.0.0:PORT with into_make_service_with_connect_info::<SocketAddr>()
  8 graceful shutdown on SIGTERM (Render sends it before a restart)
```

---

## 11. Web, file by file

**Stack:** React, Vite, TypeScript (`strict`, `noUncheckedIndexedAccess`), Zustand, React Router in library mode, CSS Modules, `idb`, Vitest, Playwright, ESLint with `eslint-plugin-boundaries` (TS §3).

**Layer rules that every file below obeys** (TS §2, §7):

- `ui` imports only `usecases`, `state` (read and subscribe), `copy`, `config`.
- `usecases` never call `fetch`, WebCodecs, WebGPU or a WASM export.
- `state` imports only `gen` types and has no side effects.
- `fetch` exists only in `net/http.ts`, `net/asset-fetch.ts`, `wasm/load-core.ts` (and `wasm/load-render.ts` from V2).
- `import.meta.env` exists only in `config/env.ts`.
- No user-facing sentence outside `copy/messages.ts`. No `any`, no non-null `!`, no `@ts-ignore`.
- Every `switch` over a generated union ends in an `assertNever` default.

### 11.1 Entry files

| File | Contents |
|---|---|
| `index.html` | `<div id="root">`, one `<script type="module" src="/src/main.tsx">`, charset and viewport metas, a title. No inline script and no inline style: the CSP forbids both |
| `src/main.tsx` | Imports `ui/styles/tokens.css`; calls `startApp()` once (not awaited); renders `<App />` into `#root` |
| `src/App.tsx` | `<RouterProvider router={router} />` |
| `src/routes.tsx` | `export const ROUTES = { landing: "/", app: "/app", authCallback: "/auth/callback", account: "/account", settings: "/settings", legal: "/legal/:doc" } as const;` and `export const router`. Mounted in V1: `/` (LandingPage), `/app` (EditorPage inside CapabilityGate), `/settings` (SettingsPage), `*` (redirect to `/`). The other three paths fall into the redirect until V6 and V8 |

### 11.2 `src/config/env.ts` [ONLY] and `config/allowlist-hosts.ts` [ONLY]

```ts
// env.ts
export const env: { readonly assetBaseUrl: string; readonly dev: boolean };
// allowlist-hosts.ts
export function allowedHosts(): readonly string[];   // [window.location.host, new URL(env.assetBaseUrl).host]
export function isAllowedUrl(url: string): boolean;
```

`env.ts` throws at module load when `VITE_ASSET_BASE_URL` is missing or is not an `https://` URL without a query string, so a misconfigured build fails at once.

### 11.3 `src/workers/protocol.ts` [ONLY] (types only, D-8)

Copy TS §14.1 and §14.2 verbatim: `WorkerKind`, `Req`, `Res`, `Progress`, `Cancel`, `FeedLine`, `AppFailure`, `MediaWorkerApi`, `AsrWorkerApi`, `AudioWorkerApi`, `RenderWorkerApi`, `ClockSync`, `StageTiming`. It imports only from `gen/`. `FailureStage` is re-exported from `gen/domain.ts` (D-1). The file contains no function and no constant.

### 11.4 `src/state/machines/transition.ts` and `state/capability-store.ts`

```ts
// transition.ts
export type MachineDef<S extends string, E extends string> = Readonly<Record<S, Partial<Record<E, S>>>>;
export class IllegalTransitionError extends Error {}
export function setIllegalTransitionReporter(fn: (machine: string, from: string, event: string) => void): void;
export function transition<S extends string, E extends string>(
  name: string, def: MachineDef<S, E>, state: S, event: E): S;
```

A pair missing from the definition is illegal. Development and test builds throw `IllegalTransitionError`. Production builds return `state` unchanged and call the reporter, which `start-app.ts` wires to `track({ name: "client_error", props: { error_code: "E_INTERNAL", stage: "import" } })` (TS §12.3). `state/` cannot import `analytics/`, which is why the reporter is injected.

```ts
// capability-store.ts
export type CapabilityStatus = "unchecked" | "checking" | "supported" | "unsupported";
export type CapabilityState = { status: CapabilityStatus; reason?: UnsupportedReason; report?: CapabilityReport };
export const useCapabilityStore: UseBoundStore<StoreApi<CapabilityState>>;
export function beginCheck(): void;                                   // unchecked -> checking
export function setSupported(report: CapabilityReport): void;         // checking -> supported
export function setUnsupported(reason: UnsupportedReason, report: CapabilityReport): void;   // checking -> unsupported
```

The capability machine has three legal transitions (TS §12.2); `unsupported` is terminal until reload. `CapabilityReport` is defined here as the generated props type of the `capability_check` event, so the store, the check and the analytics event share one shape.

### 11.5 `src/persistence/schema.ts` [ONLY] and `persistence/db.ts`

```ts
// schema.ts
export const DB_NAME = "offcut";
export const DB_VERSION = 1;
export const STORES = { clips: "clips", transcripts: "transcripts", events: "events", edits: "edits",
                        renderCache: "renderCache", entitlement: "entitlement",
                        receiptOutbox: "receiptOutbox", meta: "meta" } as const;
export const META_KEYS = { anonId: "anonId", persistRequested: "persistRequested" } as const;
export type Versioned<T> = { schemaVersion: number; value: T };
export interface OffcutDb extends DBSchema { /* key and value types of TS §23.1, one entry per store */ }

// db.ts
export function openDb(): Promise<IDBPDatabase<OffcutDb>>;            // memoized
export function metaGet<T>(key: string): Promise<T | undefined>;
export function metaSet<T>(key: string, value: T): Promise<void>;
```

`openDb` runs numbered upgrade functions in `onupgradeneeded`. Upgrade 1 creates all eight stores (D-6). A stored value whose `schemaVersion` is newer than the code is treated as absent and deleted (TS §23.3). `persistence/` never imports `net`, `ui` or `workers`.

### 11.6 `src/net/http.ts` [ONLY fetch to the app origin], `api-client.ts`, `asset-fetch.ts`

```ts
// http.ts
export const API_TIMEOUT_MS = 10_000;            // TS §22.9
export const API_COLD_START_BUDGET_MS = 70_000;
export const API_WAKING_NOTICE_MS = 3_000;
export type HttpRequest = {
  method: "GET" | "POST";
  path: `/${string}`;                            // relative to /api/v1; never an absolute URL
  body?: unknown;
  retry: "cold-start" | "none";
  keepalive?: boolean;
  onWaking?: () => void;                         // called at most once, 3 s after the first attempt started
  signal?: AbortSignal;
};
export type HttpResult<T> = { ok: true; status: number; data: T }
                          | { ok: false; failure: AppFailure; apiError?: ApiError };
export type HttpDeps = { fetch: typeof fetch; now: () => number; sleep: (ms: number) => Promise<void>; onLine: () => boolean };
export function createHttp(deps: HttpDeps): { request<T>(req: HttpRequest): Promise<HttpResult<T>> };
export const http: ReturnType<typeof createHttp>;        // built with the browser's own functions
export function toAppFailure(cause: "offline" | "timeout" | "status", status?: number): AppFailure;
```

Behaviour (TS C-15, TS §11.2):

| Situation | Result |
|---|---|
| `onLine()` is false | `E_NET_OFFLINE` at once; `fetch` is not called |
| `fetch` rejects with a `TypeError` | `E_NET_OFFLINE` |
| No response within `API_TIMEOUT_MS`, or status 502, 503 or 504 | `retry: "cold-start"`: retry after 1 s, 2 s, 4 s, 8 s, then every 8 s, while the total stays within `API_COLD_START_BUDGET_MS`; then `E_NET_TIMEOUT`. `retry: "none"`: `E_NET_TIMEOUT` after the first attempt |
| Status 429 | `E_API_RATE_LIMITED`; `apiError.retry_after_secs` carried through; not retried |
| Other 5xx | Retried like a timeout in cold-start mode; finally `E_API_5XX` |
| Status 4xx | Not retried. `failure.code = "E_INTERNAL"`, `retryable: false`, with the parsed `apiError` so the caller can branch on `apiError.code` |
| Status 2xx | `ok: true`; `data` is the parsed JSON, or `undefined` for 204 |

Every request uses `credentials: "same-origin"`, `cache: "no-store"` and, with a body, `Content-Type: application/json`. `failure.stage` is always `"api"`. `failure.detail` is filled only in development builds and is never sent anywhere.

```ts
// api-client.ts  (V1: 3 of 15 route functions)
export function wake(): void;                                         // GET /healthz, cold-start retry, result ignored
export function postEvents(batch: EventsBatch): Promise<HttpResult<void>>;                 // retry "none", keepalive
export function postNotifyMe(req: NotifyMeRequest, onWaking: () => void): Promise<HttpResult<void>>;   // cold-start retry

// asset-fetch.ts  [ONLY fetch to the asset host]
export function assetUrl(path: `${string}`): string;                  // env.assetBaseUrl + "/" + path; rejects "?" and ".."
```

`asset-fetch.ts` has no `fetch` call in V1: the demo video is loaded by a `<video crossorigin="anonymous">` element from `assetUrl(...)`. V2 adds the ranged `fetchAsset` here.

### 11.7 `src/analytics/client.ts`

```ts
export const FLUSH_INTERVAL_MS = 10_000;
export const FLUSH_AT = 20;
export function initAnalytics(o: { anonId: AnonId }): void;
export function track(event: AnalyticsEvent): void;       // never throws; never awaits
export function flush(): Promise<void>;
```

- `track` accepts only the generated `AnalyticsEvent` union, so a property that is not on the allowlist does not type-check (TS §25.1 P-3).
- Events tracked before `initAnalytics` wait in the queue.
- A flush happens every 10 s, when 20 events are queued, and on `visibilitychange` to hidden (TS §32). Each flush sends at most `MAX_EVENTS_PER_BATCH` events through `postEvents`.
- A failed flush drops its events. This is the one file where a swallowed failure is allowed (TS §11.3).
- This module must not import any store, `persistence/` or anything that could hold user text (TS §2). The anonymous id is handed to it.

### 11.8 `src/platform/simd-probe.ts`, `platform/capability.ts`, `workers/render/encoders.ts`

```ts
// simd-probe.ts
export const SIMD_PROBE_BYTES: Uint8Array;        // the smallest valid module that uses one v128 instruction

// encoders.ts  [ONLY WebCodecs encoder configs]  (V1: constants only)
export type VideoLadderEntry = { codec: string; hardwareAcceleration: "prefer-hardware" | "prefer-software" };
export const VIDEO_ENCODE_LADDER: readonly VideoLadderEntry[];        // TS §21.2, four entries, in order
export const AAC_ENCODE_CONFIG: AudioEncoderConfig;                   // mp4a.40.2, 48,000 Hz, 2 channels, 160,000 b/s
export const KEYFRAME_INTERVAL_FRAMES = 60;
export function videoConfigFor(e: VideoLadderEntry, width: Px, height: Px, bitrate: BitsPerSec): VideoEncoderConfig;
                                                                      // framerate 30, variable bitrate, latencyMode "quality", avc format "avc"

// capability.ts
export const PER_CHECK_TIMEOUT_MS = 1_000;
export const H264_DECODE_PROBE: VideoDecoderConfig;                   // avc1.640028, 1920x1080
export const AAC_DECODE_PROBE: AudioDecoderConfig;                    // mp4a.40.2, 48,000 Hz, 2 channels
export type PlatformProbe = {
  mobileHint(): boolean | undefined;              // navigator.userAgentData?.mobile
  platformHint(): string | undefined;             // navigator.userAgentData?.platform
  hasWebCodecs(): boolean;                        // "VideoEncoder" in globalThis && "AudioEncoder" in globalThis
  isolated(): boolean;                            // crossOriginIsolated && typeof SharedArrayBuffer !== "undefined"
  simdOk(): boolean;                              // WebAssembly.validate(SIMD_PROBE_BYTES)
  storageOk(): Promise<boolean>;                  // getDirectory() resolves and indexedDB.open succeeds
  deviceMemoryGb(): number | undefined;
  gpuVendor(): Promise<string | null>;            // null = no adapter
  h264Decode(): Promise<boolean>;
  h264Encode(): Promise<boolean>;                 // any ladder entry supported at 1080x1920, 30 fps
  aacDecode(): Promise<boolean>;
  aacEncode(): Promise<boolean>;
};
export const browserProbe: PlatformProbe;
export function runCapabilityCheck(probe?: PlatformProbe): Promise<CapabilityReport>;
```

`runCapabilityCheck`:

1. Start all eleven checks at once, each wrapped in a `PER_CHECK_TIMEOUT_MS` timeout. A timeout or a thrown error counts as a failed check. Running them together keeps the whole check near one second, inside `LIMITS.CAPABILITY_CHECK_BUDGET` (3 s).
2. The reported reason is the first failing check in the order of TS §13.2 (the order of the `UnsupportedReason` enum). Later checks still run so the report is complete.
3. `deviceMemoryGb()` undefined passes; below `LIMITS.MIN_DEVICE_MEMORY_GB` fails.
4. Build the report with enums only:

| Field | Mapping |
|---|---|
| `gpu_vendor` | Lower-cased vendor string containing `intel`, `amd` (or `advanced micro`), `nvidia`, `apple`, `qualcomm`: that enum; any other non-empty string: `other`; no adapter or empty: `unknown` |
| `memory_bucket` | undefined: `unknown`; below 4: `lt4`; 4 up to below 8: `gb4`; 8 or more: `gb8plus` |
| `platform` | `"Windows"`: `windows`; `"macOS"`: `macos`; `"Linux"`: `linux`; `"Chrome OS"` or `"ChromeOS"`: `chromeos`; `"Android"`: `android`; any other string: `other`; undefined: `unknown` |

No other device attribute is read (PS §18: no fingerprinting beyond enums). `capability.ts` imports the two encoder constants from `workers/render/encoders.ts`; that file is constants-only in V1 and touches no browser API at import time.

### 11.9 `src/wasm/load-core.ts` [ONLY] and `workers/pool.ts` (V1 part)

```ts
// load-core.ts
export type CoreApi = { coreVersion(): string };          // grows in V2
export function preloadCore(): Promise<void>;             // fetch + WebAssembly.compileStreaming; caches the Module; idempotent
export function loadCore(): Promise<CoreApi>;             // instantiates from the cached Module (compiles first if needed)

// pool.ts
export function preload(): Promise<{ ok: true } | { ok: false; failure: AppFailure }>;
```

- `scripts/build-wasm.sh` writes the `wasm-bindgen --target web` output to `web/src/wasm/pkg/core/` (git-ignored). `load-core.ts` imports the glue module from there and the `.wasm` file as a Vite `?url` asset, so the file is content-hashed and served as `application/wasm` from the app origin.
- `preloadCore` is the fetch site. The compile step is what exercises CSP `'wasm-unsafe-eval'`.
- `pool.preload()` calls `preloadCore()` and converts a failure into `{ code: "E_WORKER_CRASH", stage: "import", retryable: true }` (TS §14.4). In V2 it also preloads the four worker scripts and the second bundle (TS §33).
- V1 never instantiates the module on the main thread: `loadCore` exists for the workers of V2.

### 11.10 `src/usecases/start-app.ts` and `usecases/submit-notify-me.ts`

```ts
export function startApp(): Promise<void>;                // idempotent: a second call returns the first promise
export function submitNotifyMe(email: string, wanted: Wanted, onWaking: () => void): Promise<NotifyMeOutcome>;
export type NotifyMeOutcome = { ok: true }
  | { ok: false; reason: "invalid_email" | "rate_limited" | "offline" | "unavailable"; retryAfterSecs?: number };
```

`startApp`, in the order of TS C-1:

```text
1 setIllegalTransitionReporter(...)                          wiring only
2 anonId = metaGet(META_KEYS.anonId) ?? (crypto.randomUUID() then metaSet)
     storage failure: use an in-memory id for this page load (an unsupported browser must still report why)
3 initAnalytics({ anonId })
4 track landing_view { hero_variant: "outcome" }             E-6 starts later; "privacy" is unused until then
5 beginCheck(); report = runCapabilityCheck()
     pass: setSupported(report)        fail: setUnsupported(report.unsupported_reason, report)
     track capability_check { ...report }
6 apiClient.wake()                                           fire-and-forget: starts a cold API early
7 supported only: pool.preload()                             failure: track client_error { E_WORKER_CRASH, stage "import" }
-- added later, in this position order --
   V6: restoreSession()          between 6 and 7
   V2: modelManager.inspect()    after 7
   V7: restoreClip()             last
```

`submitNotifyMe`: trim the email; if it has no `@`, return `invalid_email` without a request. Otherwise `postNotifyMe`. Map the result: 204 gives `ok`; `apiError.code === "bad_request"` gives `invalid_email`; `E_API_RATE_LIMITED` gives `rate_limited` with the delay; `E_NET_OFFLINE` gives `offline`; anything else gives `unavailable`. No analytics event is sent for the waitlist: none exists on the allowlist, and the email must never reach analytics.

### 11.11 `src/copy/messages.ts` [ONLY]

One exported object, `messages`, with typed sections. Entries are strings or functions of the generated limits; no sentence contains a typed-in number (INV-15).

| Section | Keys in V1 | Source of the wording |
|---|---|---|
| `landing` | `hero`, `subhead`, `supportedBrowsers`, `privacyLine`, `whatLeavesLink`, `demoCaption`, `waitlistHeading` | PS §10 J1: "Turn what you say into a finished short." / "Works in Chrome or Edge on Windows and Mac." / "Your video stays on your computer." |
| `dropZone` | `prompt(limits)`, `sampleButton`, `notReady` | PS §10 J2: "Drop a clip (up to {seconds} seconds, English)." with `{seconds}` from `LIMITS.MAX_CLIP_DURATION`; "Try with a sample clip" |
| `capability` | `checking`, `supported` | PS §10 J3: "Your browser can do this." |
| `unsupported` | `title`, `supportedList`, `demoHeading`, and `reasons: Record<UnsupportedReason, string>` (all 11) | PS §9.3: the exact missing capability, for example "Your browser cannot encode H.264 video here"; "Chrome or Edge on Windows or macOS" |
| `notifyMe` | `label`, `labelUnsupported`, `submit`, `success`, `invalidEmail`, `rateLimited(seconds)`, `offline`, `unavailable`, `wanted: Record<Wanted, string>` | PS §9.3: "Tell me when Safari/Firefox/mobile is supported." |
| `api` | `waking` | TS C-15: "Connecting to the account service…" |
| `settings` | `title`, `whatLeavesHeading`, `rows` (the four rows of PS §12.7), `eventListHeading`, `events: Record<event name, string>` | PS §12.7 |
| `errors` | `E_NET_OFFLINE`, `E_NET_TIMEOUT`, `E_API_5XX`, `E_API_RATE_LIMITED`, `E_INTERNAL`, `E_WORKER_CRASH`, each `{ title, body, action }` | TS §11.1 |

Wording rules that already apply: never the phrases "never leaves", "GDPR", "DPDP", "CCPA", "SOC 2" or "compliant" (TS §25.1 P-12; the test that enforces this arrives in V8, the rule applies now). The privacy claim is the factual one: your video is not uploaded.

### 11.12 Pages

| File | Reads | Renders | Calls |
|---|---|---|---|
| `ui/pages/LandingPage.tsx` | capability store | Hero; the E-1 demo video from `assetUrl`; supported-browser line; privacy line with a link to `/settings`; capability status line; `DropZone`; `NotifyMeForm` with `wanted = ["launch"]` | none directly |
| `ui/pages/EditorPage.tsx` | none | `messages.dropZone.notReady` and `NotifyMeForm` with `wanted = ["launch"]` (D-16) | none |
| `ui/pages/UnsupportedPage.tsx` | prop `reason: UnsupportedReason` | `messages.unsupported.reasons[reason]`; the supported list; the demo video; `NotifyMeForm` with `wanted = ["safari", "firefox", "mobile", "linux"]`, preselected `mobile` when the reason is `UNSUPPORTED_MOBILE` | none |
| `ui/pages/SettingsPage.tsx` | none | `WhatLeavesTable` | none |

### 11.13 Components

| File | Props | Behaviour |
|---|---|---|
| `ui/components/CapabilityGate.tsx` | `children` | `checking` or `unchecked`: `messages.capability.checking`. `supported`: children. `unsupported`: `<UnsupportedPage reason>` in place (TS §13.1) |
| `ui/components/DropZone.tsx` | none | V1: a drop target and a "Try with a sample clip" button, both present and marked `aria-disabled`. A drop or click shows `messages.dropZone.notReady`. The dropped `File` is never read, stored or inspected. V2 wires `importClip` |
| `ui/components/NotifyMeForm.tsx` | `wanted: readonly Wanted[]`, `preselect?: Wanted` | Email input, a select when more than one `wanted` option, submit. Local state: `idle, sending, waking, done, error(reason)`. Calls `submitNotifyMe`. Shows `messages.api.waking` when the use-case reports waking |
| `ui/components/WhatLeavesTable.tsx` | none | The four fixed rows of PS §12.7, then one row per entry of `ANALYTICS_EVENT_DOCS` from `gen/api.ts`, so the page cannot drift from the allowlist (TS §25.1 P-4) |

### 11.14 Styles

`ui/styles/tokens.css` defines CSS custom properties (color, spacing, type scale, radius) on `:root` and a system font stack. No web font and no font CDN (TS §24.1). `pages.module.css` and `components.module.css` hold the class names. There is no runtime CSS-in-JS: the CSP is `style-src 'self'`.

---

## 12. Root configuration, scripts, CI, hosting files

### 12.1 Cargo workspace

| File | Contents |
|---|---|
| `Cargo.toml` | `[workspace]` with `resolver = "2"` and members `crates/offcut-types`, `crates/offcut-api-types`, `crates/offcut-wasm-core`, `server`. `[workspace.dependencies]` pins every shared dependency once. `[workspace.lints.clippy]` denies `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `wildcard_enum_match_arm` (TS §7, §11.3); every crate has `[lints] workspace = true`. `[profile.release]`: `lto = true`, `codegen-units = 1`. **Do not set `panic = "abort"`**: the server's `CatchPanicLayer` needs unwinding |
| `rust-toolchain.toml` | The pinned stable channel; `targets = ["wasm32-unknown-unknown"]`; `components = ["rustfmt", "clippy"]` |
| `clippy.toml` | `disallowed-methods`: `std::time::Instant::now`, `std::time::SystemTime::now` (pure crates must not read a clock, TS §7). `allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-panic-in-tests`, `allow-indexing-slicing-in-tests` all `true`. Integration test files under `tests/` start with `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]` |
| `deny.toml` | `[licenses]`: an allowlist of permissive licenses, fixed on the first `cargo deny check`. `[advisories]`: deny. `[bans]`: `web-sys`, `js-sys`, `wasm-bindgen`, `wgpu`, `rand`, `getrandom` may be direct dependencies only of the binding and renderer crates (cargo-deny `wrappers`), so no pure crate can pull them in; package `offcut-api` may not depend on `offcut-mp4`, `-text`, `-detect`, `-dsp`, `-scene`, `-render`. Those crates do not exist yet; the bans are written now so the first V2 commit is already guarded |

### 12.2 Root `package.json` and `pnpm-workspace.yaml`

`pnpm-workspace.yaml` lists `web`. Root scripts in V1 (TS §33); scripts whose targets do not exist yet (`e2e:media`, `e2e:device`, `fixtures`, `corpus`, `bench:device`, `verify`) are added by the version that creates the target.

| Script | Does |
|---|---|
| `dev` | `scripts/build-wasm.sh --dev --watch` and `vite` in `web` (COOP/COEP headers, `/api/v1` proxy to the local API on port 8080) |
| `build:wasm` | `scripts/build-wasm.sh` |
| `gen:types` | `scripts/gen-types.sh` |
| `build:vite` | `vite build` in `web` (used by `vercel build`, which cannot run Rust) |
| `build` | `gen:types`, `build:wasm`, `build:vite`, `scripts/check-hosts.mjs` |
| `check` | `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`, ESLint, `tsc --noEmit`, `check-file-tree`, `check-gen-clean`, `check-copy-codes` |
| `test` | `cargo test --workspace` and `vitest run` |
| `e2e` | Playwright non-media suites (V1: `landing.spec.ts`) |

### 12.3 Web configuration files

| File | Contents |
|---|---|
| `web/package.json` | Dependencies: `react`, `react-dom`, `react-router`, `zustand`, `idb`. Dev: `vite`, `@vitejs/plugin-react`, `typescript`, `vitest`, `@playwright/test`, `eslint`, `typescript-eslint`, `eslint-plugin-boundaries`, `eslint-plugin-react`. All pinned through `pnpm-lock.yaml` |
| `web/tsconfig.json` | `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `noEmit`, `moduleResolution: "bundler"`, `lib` with `DOM`, `DOM.Iterable`, `WebWorker`, `ESNext`; WebCodecs and WebGPU type packages |
| `web/vite.config.ts` | React plugin. `server.headers`: COOP and COEP only. `server.proxy`: `/api/v1` to `http://localhost:8080`. `preview.headers`: every header of `vercel.json` rule `/(.*)`, read from that file so there is one source (D-15). `build.target: "esnext"`. `build.assetsInlineLimit: 0`, so no font or WASM file is inlined as a `data:` URL that the CSP would block. `worker.format: "es"` |
| `web/vitest.config.ts` | `environment: "node"` by default; `jsdom` for files that touch the DOM; includes `src/**/*.test.ts` |
| `web/playwright.config.ts` | `use.channel: "chrome"`; `use.baseURL = process.env.E2E_BASE_URL ?? "http://localhost:4173"`; `webServer` runs `vite preview` unless `E2E_BASE_URL` is set; one project `non-media` |
| `web/eslint.config.js` | Flat config. `boundaries` element types `ui, usecases, state, persistence, models, net, analytics, workers, wasm, platform, config, copy, gen, entitlement` with the allow matrix of TS §2. `no-restricted-globals` and `no-restricted-properties` for `fetch`, `XMLHttpRequest`, `WebSocket`, `EventSource`, `RTCPeerConnection`, `navigator.sendBeacon`, with overrides for the four fetch files. `no-restricted-syntax` for `import.meta.env` outside `config/env.ts`, for `as <Brand>` casts outside `gen/` and `workers/`, and for empty `catch` outside `analytics/client.ts`. `react/jsx-no-literals` in `ui/`. `@typescript-eslint/no-explicit-any`, `no-non-null-assertion`, `ban-ts-comment`, `switch-exhaustiveness-check`. `no-console`. Type-only imports of `workers/protocol` are allowed from `net/` (D-8) |
| `web/public/robots.txt` | Allow all; no sitemap |

Every rule of TS §7 is switched on in V1, including the ones that have nothing to check yet. A V2 file that breaks a boundary fails lint the day it is written.

### 12.4 `web/vercel.json` [ONLY place for response headers and the rewrite]

```json
{
  "framework": null,
  "installCommand": "pnpm install --frozen-lockfile",
  "buildCommand": "pnpm run build:vite",
  "outputDirectory": "dist",
  "rewrites": [
    { "source": "/api/v1/:path*", "destination": "https://<render-host>/api/v1/:path*" },
    { "source": "/((?!api/|assets/|ort/).*)", "destination": "/index.html" }
  ],
  "headers": [
    { "source": "/(.*)", "headers": [
      { "key": "Cross-Origin-Opener-Policy",   "value": "same-origin" },
      { "key": "Cross-Origin-Embedder-Policy", "value": "require-corp" },
      { "key": "Content-Security-Policy",      "value": "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self'; style-src 'self'; font-src 'self'; img-src 'self' data: blob:; media-src 'self' blob: <ASSET_ORIGIN>; connect-src 'self' <ASSET_ORIGIN>; base-uri 'none'; form-action 'none'; frame-ancestors 'none'" },
      { "key": "Referrer-Policy",              "value": "no-referrer" },
      { "key": "X-Content-Type-Options",       "value": "nosniff" },
      { "key": "Permissions-Policy",           "value": "camera=(), microphone=(), geolocation=()" },
      { "key": "Strict-Transport-Security",    "value": "max-age=63072000; includeSubDomains" }
    ] },
    { "source": "/assets/(.*)", "headers": [ { "key": "Cache-Control", "value": "public, max-age=31536000, immutable" } ] },
    { "source": "/ort/(.*)",    "headers": [ { "key": "Cache-Control", "value": "public, max-age=31536000, immutable" } ] },
    { "source": "/((?!assets/|ort/).*)", "headers": [ { "key": "Cache-Control", "value": "no-cache" } ] }
  ]
}
```

Replace `<render-host>` and `<ASSET_ORIGIN>` with the real values once TE-7 and the Render service exist. The header set is exactly TS §24.3. The `/ort/` rule is unused until V2 and is included now so this file is final. `form-action 'none'` means the waitlist form must submit through script (`submitNotifyMe`), never through a native form post. No functions and no middleware are ever added here (TS §36).

### 12.5 `render.yaml` and `server/Dockerfile`

`render.yaml`: one service, `type: web`, `name: offcut-api`, `runtime: image`, image `ghcr.io/<owner>/offcut-api:latest`, `plan: free`, `healthCheckPath: /api/v1/healthz`, and one `envVars` entry with `sync: false` for each variable of section 3.3. No disk, no worker, no cron (TS §1).

`server/Dockerfile`, two stages:

```text
build stage    pinned rust slim image; copy Cargo.toml, Cargo.lock, rust-toolchain.toml, crates/, server/
               ENV SQLX_OFFLINE=true;  ARG GIT_SHA=dev
               cargo build --release -p offcut-api
runtime stage  pinned debian slim image with ca-certificates; non-root user
               copy the binary;  ENV GIT_SHA from the build arg;  CMD ["offcut-api"]
```

The image is built in CI, never on Render (TS §3: no Rust compile in Render's free build pipeline).

### 12.6 Scripts

| Script | Contract |
|---|---|
| `scripts/build-wasm.sh` | For each entry of a `BUNDLES` list (V1: `offcut-wasm-core:core`): `cargo build --target wasm32-unknown-unknown --release -p <crate>` with `+simd128`; `wasm-bindgen --target web --out-dir web/src/wasm/pkg/<name>`; `wasm-opt -O3`. Fails if the `wasm-bindgen` CLI version differs from the crate version in `Cargo.lock` (TS §3). `--dev` skips `wasm-opt`; `--watch` rebuilds on change. V2 adds one list entry |
| `scripts/gen-types.sh` | Section 9 |
| `scripts/check-gen-clean.sh` | Runs `gen-types.sh`; fails on any diff under `web/src/gen/` |
| `scripts/check-file-tree.mjs` | Parses the fenced tree of TS §5 from `docs/technicalspec.md` into a path set. Fails when a tracked source file (under `crates/`, `server/`, `web/`, `scripts/`, `verify/`, `corpus/`, `bench/`, `.github/`, or a root config file) is absent from the set. Ignores generated and data paths: lock files, `server/.sqlx/**`, `**/*.stderr`, snapshot and fixture contents. Also fails when a non-test source file exceeds 400 lines (TS §29), and when a crate's `Cargo.toml` declares an `offcut-*` dependency that TS §7 does not allow. A listed file that does not exist yet is not an error before V10 |
| `scripts/check-hosts.mjs` | Scans `web/dist` for `http://` and `https://` literals. Fails on any host other than the asset host. A short `NON_NETWORK_LITERALS` list (XML namespace URLs and library error-page URLs that are never fetched) is allowed, each entry with a one-line reason. Also fails when the hosts in the `vercel.json` CSP differ from `VITE_ASSET_BASE_URL`. Prints the gzip size of the app shell |
| `scripts/check-copy-codes.mjs` | Reads `ERROR_CODES`, `REJECT_REASONS`, `UNSUPPORTED_REASONS` from `gen/domain.ts` and the keys of `messages.ts`. D-13 rule. `COPY_PENDING` in V1: every `REJECT_*` (due V3), and every `E_*` except the six listed in section 11.11 (due by V7). All eleven `UNSUPPORTED_*` must have copy now |
| `scripts/check-external-facts.mjs` | Prints the free-tier facts to re-verify before launch, one line each with its experiment id (TE-5 to TE-11) and the date last checked. V1 fills in the dates for TE-5, TE-6, TE-7, TE-11 |
| `scripts/upload-assets.sh` | Uploads files to the asset host under a content-hashed name with `Cache-Control: public, max-age=31536000, immutable`, and prints the public path. V1 uses it for the E-1 demo clips under `media/`; V2 for model files under `models/` |

### 12.7 CI and deploy

**`.github/workflows/ci.yml`**, on every pull request and on `main` (TS §33 steps 1-9):

1. Checkout; restore Cargo and pnpm caches; secret scan.
2. `cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo deny check`.
3. `pnpm gen:types`; `scripts/check-gen-clean.sh`.
4. `cargo test --workspace` with a Postgres service container and `DATABASE_URL` pointing at it.
5. `pnpm build:wasm`.
6. ESLint; `tsc --noEmit`; `check-file-tree.mjs`; `check-copy-codes.mjs`.
7. `vitest run`.
8. `vite build` with `VITE_ASSET_BASE_URL` set; `check-hosts.mjs`.
9. Playwright `non-media` project against `vite preview`: `landing.spec.ts`.

On `main` only, after step 9:

10. Call `deploy-api.yml`.
11. Poll `https://<render-host>/api/v1/healthz` until `version` equals the commit SHA (up to 5 minutes).
12. `vercel build --prod`, then `vercel deploy --prebuilt --prod`.
13. Header check: fetch `/` from the deployment and compare every header of the `/(.*)` rule in `vercel.json`, value for value.
14. Smoke: `E2E_BASE_URL=<deployment url>` Playwright with `--grep @smoke`.

**`.github/workflows/deploy-api.yml`** (callable): build `server/Dockerfile` with `--build-arg GIT_SHA`, push to GHCR as `:<sha>` and `:latest`, call the Render deploy hook.

Server first, then web, always (TS §33): the server rejects unknown request fields, so a new client must never reach an old server.

### 12.8 `.gitignore`

`target/`, `node_modules/`, `web/dist/`, `web/src/wasm/pkg/`, `.vercel/`, `.env*`, `web/test-results/`, `web/playwright-report/`, generated fixtures. `web/src/gen/` and `server/.sqlx/` are **not** ignored.

---

## 13. Tests, case by case

V1 has six test files, one compile-fail case and inline unit tests. Inline unit tests for `units.rs`, `analytics.rs`, `config.rs`, `client_ip.rs`, `rate_limit.rs` and `normalize_email` are listed in their sections above.

### 13.1 `server/tests/common/mod.rs`

```rust
pub struct TestApp { pub base_url: String, pub pool: sqlx::PgPool, pub client: reqwest::Client }
impl TestApp { pub async fn spawn() -> TestApp; }
```

`spawn` creates a fresh database with a random name on the server named by `DATABASE_URL`, runs the migrations, builds a `Config` from fixed test values, starts `router::build` on `127.0.0.1:0`, and returns its address. Each test gets its own database, so tests run in parallel. V6 adds a fake `Mailer` and a fake `BillingProvider` here.

### 13.2 `server/tests/analytics_allowlist.rs`

| Case | Expect |
|---|---|
| A batch holding one valid event of each of the 19 kinds | 204; 19 rows; each row's `name` and `props` match what was sent |
| Unknown event name | 400; no rows |
| A valid event with one extra prop | 400; no rows |
| A string where an enum is expected (`hero_variant: "<free text>"`) | 400; no rows |
| A free-text prop on any event | 400; no rows |
| One invalid event among valid ones | 400; none of the batch is stored |
| `duration_ms` above 3,600,000; a count above 10,000 | 400 |
| 51 events; 0 events | 400 |
| A body over 16 kB | 413 |
| `Content-Type: application/octet-stream`; `multipart/form-data`; `video/mp4` | 415 |
| An extra top-level key next to `anon_id` and `events` | 400 |
| Schema: `analytics_events` columns are exactly `id, anon_id, name, props, ts` | No `user_id`, no IP, no user-agent column (TS §25.1 P-10) |
| Purge: one row dated 91 days ago, one dated 89 days ago; `purge_once` | The old row is gone, the recent one stays |
| A stored row after a request that carried `X-Forwarded-For` and a `User-Agent` | Neither value appears in any column |

### 13.3 `server/tests/notify_me.rs` (D-12)

| Case | Expect |
|---|---|
| `GET /api/v1/healthz` | 200; body has `ok: true` and a `version`; `Cache-Control: no-store`; `X-Content-Type-Options: nosniff`; no `Access-Control-*` header |
| Valid email, `wanted: "launch"` | 204; one row with the normalized address |
| The same request again | 204; still one row |
| Same email, `wanted: "safari"` | 204; two rows |
| `"  Someone@Example.COM "` | Stored as `someone@example.com` |
| No `@`; over-length; embedded newline | 400; no row |
| Unknown `wanted` value; an extra field | 400 |
| A body over 1 kB | 413 |
| Six requests from one IP within the hour | The sixth is 429 with `retry_after_secs` and a `Retry-After` header |
| An unknown path under `/api/v1` | 404 with `code: "not_found"` |

### 13.4 `web/src/platform/capability.test.ts`

All cases use a fake `PlatformProbe`.

| Case | Expect |
|---|---|
| Every check passes | `result: "pass"`; no `unsupported_reason` |
| Each of the 11 checks failing alone | Its own `UNSUPPORTED_*` |
| Two checks failing | The earlier one in TS §13.2 order |
| A check that never resolves | Counted as failed after `PER_CHECK_TIMEOUT_MS`; the whole run finishes within the 3 s budget (fake timers) |
| A check that throws | Counted as failed; no unhandled rejection |
| `deviceMemoryGb()` undefined | Passes; `memory_bucket: "unknown"` |
| `deviceMemoryGb()` 2, 4, 8 | `lt4` and fail; `gb4` and pass; `gb8plus` and pass |
| Vendor strings `"Intel Inc."`, `"apple"`, `"ARM"`, `""`, no adapter | `intel`, `apple`, `other`, `unknown`, `unknown` plus `UNSUPPORTED_WEBGPU` |
| Platform hints `"Windows"`, `"macOS"`, `"Chrome OS"`, `"Fuchsia"`, undefined | `windows`, `macos`, `chromeos`, `other`, `unknown` |
| `mobileHint()` true | `UNSUPPORTED_MOBILE`, even when every other check passes |
| Report shape | Every value is a member of its generated enum; no free string survives |

### 13.5 `web/src/net/http.test.ts`

Uses `createHttp` with a fake `fetch`, a fake clock and a fake `sleep`.

| Case | Expect |
|---|---|
| 200 with JSON; 204 | `ok: true` with data; `ok: true` with `undefined` |
| Offline | `E_NET_OFFLINE`; `fetch` not called |
| `fetch` rejects with `TypeError` | `E_NET_OFFLINE` |
| First attempt times out, second returns 204 (`cold-start`) | `ok: true`; the wait before attempt 2 is 1 s; `onWaking` called exactly once |
| Every attempt times out (`cold-start`) | `E_NET_TIMEOUT`; total elapsed at most 70 s; waits follow 1, 2, 4, 8, 8 … s |
| A timeout with `retry: "none"` | `E_NET_TIMEOUT` after one attempt |
| 503 then 200 | `ok: true` |
| 500 on every attempt | `E_API_5XX` |
| 429 with `retry_after_secs: 30` | `E_API_RATE_LIMITED`; the 30 is carried; one attempt only |
| 400 with `code: "bad_request"` | `ok: false`; `apiError.code === "bad_request"`; one attempt only |
| Request shape | URL is `/api/v1` + path, never absolute; `credentials: "same-origin"`; JSON content type only with a body |
| A response that arrives within 3 s | `onWaking` never called |

### 13.6 `web/tests-e2e/helpers/fake-api.ts`, `helpers/fixtures.ts`

```ts
// fake-api.ts
export type FakeApi = { requests: RecordedRequest[]; eventsOf(name: string): unknown[] };
export function installFakeApi(page: Page, o?: { notifyMe?: { status: number; delayMs?: number };
                                                 healthz?: { delayMs?: number } }): Promise<FakeApi>;
// fixtures.ts
export function flushAnalytics(page: Page): Promise<void>;   // advances the page clock past FLUSH_INTERVAL_MS
```

`installFakeApi` intercepts every `**/api/v1/**` request with `page.route`, records it and answers 204 (or the configured status). No E2E test in V1 touches a real server. V6 extends it with sessions and signed tokens.

### 13.7 `web/tests-e2e/landing.spec.ts`

| Case | Expect |
|---|---|
| J1 content | Hero text, the supported-browser line, the privacy line, the "What leaves your device" link, the drop zone and the sample-clip button are visible before any interaction |
| Settings link | Navigates to `/settings`; the four fixed rows and one row per `ANALYTICS_EVENT_DOCS` entry are shown |
| `landing_view` | After a flush, one `POST /api/v1/events` body parses as an `EventsBatch` and contains `landing_view` with `hero_variant: "outcome"` |
| `capability_check` | The same batch, or the next, contains one `capability_check` whose props are all enum members. The test does not assume `pass`: the CI browser may lack WebGPU |
| Waitlist success | Typing an email and submitting posts `{ email, wanted: "launch" }` to `/api/v1/notify-me`; the success message shows |
| Waitlist invalid | Text without `@` shows the invalid-email message and sends no request |
| Waitlist rate-limited | A 429 from the fake API shows the rate-limit message |
| Waitlist waking | The fake API delays 4 s: the waking message appears, then success |
| Inactive drop zone | Dropping a file shows the not-ready message; no request is made |
| `/app` | Shows either the not-ready panel or the unsupported page, never a blank screen |
| No stray traffic | Every request of the session goes to the page origin or the asset host |
| `@smoke` (the only case run against a deployment) | `/api/v1/events` is blocked so the run adds no analytics rows; `crossOriginIsolated` is true; a `.wasm` response has status 200 and type `application/wasm`; no `securitypolicyviolation` event fired; `GET /api/v1/healthz` through the rewrite returns 200 |

---

## 14. Experiments to run in V1

Write each outcome into `docs/v1/experiments.md` (question, method, numbers, date, decision) and update `scripts/check-external-facts.mjs`.

| ID | Procedure | Pass (TS §37) | If it fails |
|---|---|---|---|
| **TE-6** Postgres (days 1-2, then a week of idling) | Create a database at each candidate. Run `0001_init.sql`. Connect with a pool of 5. Read the plan's current terms for expiry and storage. Leave it idle; reconnect after a week and time it | No expiry; reconnect under 2 s; storage at least 5x the 72 MB estimate of TS §23.3 | The other candidate; then any other free Postgres. No paid tier |
| **TE-7** Asset host (days 1-2) | Upload a 150 MB file. From the deployed shell, request `Range: bytes=0-8388607`. Check the response is 206 with `Content-Range`, readable under COEP (CORS for the app origin, `Accept-Ranges` and `Content-Length` exposed). Read the terms for egress charges and whether a payment card is required | 206; CORS readable; no egress charge | The other candidate; then split files under the host's size limit |
| **TE-5** Vercel (day 5) | Read and save the current free-plan terms. Deploy. In the browser console confirm `crossOriginIsolated === true`. Add a temporary server route that sleeps 60 s, call it through the rewrite, then remove it. Log the `X-Forwarded-For` header the API receives through Vercel and directly, to set `TRUSTED_PROXY_HOPS` | Isolation true; a 60 s upstream delay survives the proxy; hop counts known | If only the timeout fails: the client calls `/healthz` until warm before any real request (already how `wake()` works). If the terms block the product: another free static host (TS TDR-12) |
| **TE-11** Render (day 5, then a week) | Deploy the image. Read resident memory from Render's metrics. Let the service sleep 10 times and time each wake. With the uptime ping running, read the instance hours after a week and extrapolate to a month | Memory under 400 MB; cold start under 70 s; one always-on service fits the monthly free hours | The same image on another free container host |
| **TE-9** Merchant (day 1) | Submit the onboarding application as an India-based seller | Approval is needed by V6 | Apply to the second candidate in parallel if there is no answer in a week |
| **E-1** Demand (from day 5) | The landing page goes public with the three hand-made demo clips and the waitlist | Read at the M0 gate, Sun 1 Nov: at least 1,000 visitors and at least 5% join | PS §20.4 |
| **E-8** Reach (from day 5) | The `capability_check` event is the beacon; nothing else to build | Read at M3: at least 55% pass | PS §19 |

The temporary sleep route for TE-5 is the only code in V1 that is written to be deleted. Remove it in the same day; `notify_me.rs` asserts unknown paths return 404.

---

## 15. Hand-over: what V2 and V3 add to each V1 file

The plan asks for a seamless path onward. This section covers the next two versions: V2 (proof of concept) starts the Monday after V1, and V3 (real ingest, voice cleanup, detector) builds straight on it.

### 15.1 What V1 freezes

These are contracts from the end of V1. Changing one later means touching both sides of a boundary, so decide now.

| Frozen in V1 | Where |
|---|---|
| Every shared type and its serialized form | `offcut-types`, `offcut-api-types`, `web/src/gen/*` |
| The worker message protocol | `workers/protocol.ts` |
| The server database schema | `0001_init.sql` |
| The browser database schema | `persistence/schema.ts`, version 1, eight stores |
| The API prefix, the error envelope, the body-limit and rate-limit tables | `router.rs`, `error.rs`, `rate_limit.rs` |
| The response headers and CSP | `web/vercel.json` |
| The analytics allowlist, all 19 events | `analytics.rs` |
| The layer boundaries and banned APIs | `eslint.config.js`, `deny.toml`, `clippy.toml` |
| The HTTP result and failure shapes | `net/http.ts`, `AppFailure` |
| The state-transition helper | `state/machines/transition.ts` |
| Where copy lives and how numbers get into it | `copy/messages.ts`, `LIMITS` |

### 15.2 File by file

| V1 file | V2 adds | V3 adds |
|---|---|---|
| `Cargo.toml` | Members `offcut-mp4`, `-dsp`, `-text`, `-detect`, `-scene`, `-render`, `-entitlement`, `-wasm-render` | Nothing |
| `offcut-types/*` | Nothing. A new shared type goes here, never into a sibling crate | Nothing |
| `offcut-api-types/*` | Nothing | Nothing |
| `offcut-wasm-core` | `media_api.rs`, `hash_api.rs`, `text_api.rs`; `CoreApi` grows | `audio_api.rs` |
| `scripts/build-wasm.sh` | One `BUNDLES` entry: `offcut-wasm-render:render` | Nothing |
| `server/*` | Nothing. The server is next touched in V6 | Nothing |
| `workers/protocol.ts` | Nothing: `rpc.ts`, `pool.ts` and the workers implement it | Nothing |
| `workers/pool.ts` | Lazy worker creation, typed clients, preload of the four worker scripts and the render bundle | Nothing (restart budget in V4) |
| `workers/render/encoders.ts` | `pickVideoConfig()` | Nothing |
| `wasm/load-core.ts` | Used by the media and ASR workers through `loadCore()`; `load-render.ts` appears beside it | Used by the audio worker |
| `persistence/schema.ts` | Nothing | Nothing |
| `persistence/db.ts` | `opfs.ts` appears beside it | `clips-repo.ts`, `transcript-repo.ts`, `edits-repo.ts` appear beside it; no schema change |
| `net/asset-fetch.ts` | `fetchAsset` with HTTP Range for the model download | Nothing |
| `net/http.ts`, `api-client.ts` | Nothing | Nothing (bearer tokens and 12 more route functions in V6) |
| `analytics/client.ts` | Nothing. Eight more events start being tracked by new use-cases | Nothing. `clip_rejected` starts |
| `platform/capability.ts` | Nothing | Nothing |
| `state/capability-store.ts`, `transition.ts` | `model-store`, `clip-store`, `preview-store`, `export-store` and the model machine appear beside them, all using `transition()` | The clip machine |
| `usecases/start-app.ts` | Step "modelManager.inspect()" at the marked position | Nothing |
| `copy/messages.ts` | Model download, processing feed, preview and export copy | The 14 `REJECT_*` messages; they leave `COPY_PENDING` |
| `ui/components/DropZone.tsx` | Wired to `importClip`; reads the clip store and blockers | Hands rejections to `RejectionPanel` |
| `ui/pages/EditorPage.tsx` | The real editor shell replaces the placeholder body | Nothing |
| `ui/pages/LandingPage.tsx` | The sample-clip button becomes active | Nothing |
| `web/vite.config.ts` | Copies the ONNX Runtime `.wasm` files to `public/ort/` | Nothing |
| `web/vercel.json` | Nothing (`/ort/` and the asset host are already there) | Nothing |
| `ci.yml` | `e2e-media.yml` appears beside it; more suites in step 9 | More Rust tests run automatically |
| `scripts/check-copy-codes.mjs` | Pending list shrinks | Pending list shrinks |
| `scripts/upload-assets.sh` | Uploads the model files | Nothing |
| `tests-e2e/helpers/fake-api.ts` | Returns a Creator token signed with a test key | Nothing |
| `fixtures/speech/README.md` | You record `speech_60s_portrait.mp4` from it | You record the two pause fixtures and the landscape and no-events clips |

### 15.3 What must be true on V2's first morning

- [ ] The asset host is chosen, its CORS serves the app origin, and its origin is already in the CSP `connect-src` and `media-src`. A model download needs no header change.
- [ ] `offcut_core.wasm` compiles in production under the CSP. V2's first worker will not be the first WASM the deployment has ever seen.
- [ ] The deployed page is cross-origin isolated, so WASM threads work.
- [ ] `workers/protocol.ts` type-checks against the generated types. V2's first task is `rpc.ts`, not a design discussion.
- [ ] `VIDEO_ENCODE_LADDER` and `AAC_ENCODE_CONFIG` exist and the capability check already uses them, so E-8 measures exactly the configurations V2 will encode with.
- [ ] E2E runs against the production CSP locally and in CI.
- [ ] The reading scripts for the speech fixtures are written, so the reference clip can be recorded on day one of V2.
- [ ] `bench/results/` exists for the E-3 and E-4 timings.
- [ ] Every lint boundary is on. V2 cannot add a `fetch` or a cross-layer import by accident.

### 15.4 What V3 inherits without further work

- `RejectReason` and its 14 serialized codes, `ProbeInfo`, `ClipInfo`: `validate_probe` only has to return them.
- `LIMITS`: rejection messages interpolate the 90 seconds, 500 MB and 1920 px from it.
- `clip_rejected { reject_reason }` is already on the server allowlist.
- The `clips`, `transcripts`, `events`, `edits` stores already exist in every visitor's browser, so stage F of the pipeline (TS C-4) needs no migration.
- `WordProsody`, `Prosody`, `Quantity`, `NormalizedSpan`, `DetectedEvent`, `EventParams`: the detector and the voice chain only have to produce them.

---

## 16. Exit checklist

V1 is done when every box is ticked (`buildplan.md` section 1.2 and section 3.4).

**Code and checks**

- [ ] `pnpm check` is green: fmt, clippy, deny, ESLint, `tsc`, `check-file-tree`, `check-gen-clean`, `check-copy-codes`.
- [ ] `pnpm test` is green: `offcut-types` (unit tests and `tests/ui.rs`), `offcut-api-types`, `analytics_allowlist.rs`, `notify_me.rs`, server unit tests, `capability.test.ts`, `http.test.ts`.
- [ ] `pnpm build` is green, including `check-hosts.mjs`.
- [ ] `pnpm e2e` is green: `landing.spec.ts`.
- [ ] `ci.yml` is green on `main`.
- [ ] No source file exceeds 400 lines.

**Deployment**

- [ ] The API image is on GHCR and running on Render; `/api/v1/healthz` returns the current commit SHA.
- [ ] The web build is on Vercel; `/api/v1/healthz` answers through the Vercel rewrite.
- [ ] The header check passes against the deployed URL; `crossOriginIsolated === true`.
- [ ] The `@smoke` case passes against the deployed URL: `offcut_core.wasm` compiles with no CSP violation.
- [ ] A real email submitted on the deployed page appears in `platform_waitlist`.
- [ ] `landing_view` and `capability_check` rows appear in `analytics_events` with only enum props.
- [ ] The uptime monitor pings `/api/v1/healthz` every 5 minutes. This must be on before the page is shared: without it the API sleeps and visitors are undercounted.

**Experiments and records**

- [ ] TE-5, TE-6, TE-7, TE-11 outcomes are in `docs/v1/experiments.md`; the Postgres and asset hosts are chosen; `TRUSTED_PROXY_HOPS` is set from TE-5.
- [ ] Vercel's free-plan terms are saved with the date read.
- [ ] TE-9 onboarding is submitted.
- [ ] The E-1 landing page is public with the three demo clips; the first posts and outreach messages are out.
- [ ] Any decision of section 2 that changed during the build is corrected here and in the specs.
- [ ] Section 15.3 is fully ticked.
- [ ] Tagged `v1`.
