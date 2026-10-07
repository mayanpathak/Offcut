# Offcut V1 — coding-prompts.md

Source documents: `docs/v1/v1implementation.md` (written **§n**) and `docs/v1/v1buildguide.md` (written **G§n**). `TS §n` = `technicalspec.md`. `D-n` = a decision in §2. The prompts were written from those two V1 documents and then checked against `technicalspec.md` and `buildplan.md` §1-§3; the corrections are listed under "Revision notes" at the end.

**How to use.** Paste the **Standard Agent Block (SAB)** below, then one prompt, into your coding agent. Run prompts in order. Do not start Prompt N until Prompt N-1's "Done when" is fully ticked, and commit after each prompt. Steps marked **Human** are yours, not the agent's (see "Who does what").

**Recorded deviation from TS §34.1 and `buildplan.md` §1.3.** Those sections give each file its own spec and hide the rest of the repo from the implementer. These prompts cover several files each and let the agent read the repo. This is accepted for V1 only, because V1 has almost no behaviour. If you want the per-file protocol back for V2, write the specs from `docs/file-specs/TEMPLATE.md` and stop using this format.

---

# Standard Agent Block (SAB) — prepend to every prompt

**Before modifying anything**

1. Inspect the repo. Read the modules this prompt touches and the spec sections it cites.
2. Search for existing types, helpers and constants before creating any. Reuse them.
3. Verify the previous prompts' outputs actually exist and behave as this prompt assumes. If not: decide whether it is a bug or an intentional boundary, make the smallest compatible fix, and report it.

**During implementation**

1. Keep modules small and cohesive (no source file over 400 lines). Preserve dependency direction (see Invariants).
2. Implement only this prompt's scope. If you discover later-prompt work, leave it and report it.
3. No TODO stubs, no `unwrap`/`expect`/`panic!`/slice indexing in non-test Rust, no `any`/`!`/`@ts-ignore` in TS, no disabled lints, no weakened tests, no new dependencies unless this prompt names them.
4. If existing code contradicts the architecture: fix only what blocks this task; otherwise report it and leave it.
5. Use **Git Bash** for all shell commands; scripts are POSIX `sh`, invoked with `sh`, never `bash`.
6. If the specs do not give you a signature or contradict each other, stop and list what is missing. Do not guess (TS §34.1).
7. Create only the files this prompt names. V1 has exactly six test files (§13); every other test is an inline `#[cfg(test)]` module. A file that is not in the TS §5 tree fails `check-file-tree.mjs`.
8. Do not perform a step marked **Human**. List it in your reply so the human can do it.

**After implementation**

1. Format (`cargo fmt`), lint, build, run this prompt's tests and all earlier tests.
2. Fix regressions you caused. Check every "Done when" box. Leave a box that depends on a **Human** step unticked and say so.
3. Reply with: files created/modified, commands run with results, deviations, **Human** steps still open, and issues found but not fixed.

---

# Who does what

The agent writes files and runs commands in the repo. Everything that needs an account, a dashboard, a secret, a browser's DevTools or a week of waiting is a **Human** step.

| When | Human step | Guide |
| --- | --- | --- |
| Before Prompt 01 | Open the accounts; create one database at each Postgres candidate (starts TE-6); sign up at both asset-host candidates (starts TE-7); submit the merchant onboarding application (TE-9); generate two dev signing keys | G§0.5, G§0.6 |
| Before Prompt 09 | Write the local `.env` with the dev keys | G§3.1 |
| Prompt 20 | Look at DevTools: the `.wasm` response, the console under the production CSP, `crossOriginIsolated` | G Milestone 5 |
| Prompt 23 | Walk the Milestone-6 table in Chrome | G Milestone 6 |
| Prompt 26 | Open the pull request; confirm CI is green; merge | G§7.5 |
| Before Prompt 27 | Choose the database (TE-6) and the asset host (TE-7); set the asset host's CORS; create the Vercel project and note its URL | G§8.1-8.3 |
| Prompt 27 | Start Docker Desktop; make the GHCR package public; create the Render service and enter its variables; upload the three demo clips | G§8.4-8.6 |
| Prompt 28 | Create the Vercel token; set the GitHub secrets and the variable | G§8.7 |
| Prompt 29 | Run the measurements; configure the uptime monitor; save the Vercel plan terms | G§8.9 |
| Prompt 30 | Submit a real email on the deployed page; tick §15.3 and §16 | G§8.10 |

---

# Project Implementation Map

## Architecture summary

V1 is a "working skeleton": every layer of Offcut exists, is connected and deployed, with almost no behaviour. A Vite/React landing page on Vercel (cross-origin isolated, strict CSP) runs a capability check, compiles a tiny Rust WASM module, and sends analytics events and waitlist emails through a Vercel rewrite to an Axum API on Render, backed by Postgres. CI gates everything. V2/V3 must be **additive**: no V1 file is restructured later.

## Major components

| Component | Location |
| --- | --- |
| Shared domain types (units, ids, limits, errors, media, transcript…) | `crates/offcut-types` |
| API DTOs + analytics allowlist (19 events) | `crates/offcut-api-types` |
| Minimal WASM bundle | `crates/offcut-wasm-core` |
| Axum API (3 routes), schema, rate limiter, retention | `server/` |
| TS codegen output | `web/src/gen/{domain,api}.ts` |
| Web shell (layers: config, state, persistence, net, analytics, platform, wasm, workers, usecases, copy, ui) | `web/src/` |
| Scripts, CI, hosting files | `scripts/`, `.github/`, `web/vercel.json`, `render.yaml` |

## Dependency graph

```text
repo + workspaces → offcut-types → offcut-api-types → codegen (gen/*.ts)
                                         │                    │
                                         ▼                    ▼
              server (config→log→error→ip→rate limit→db→routes→tests)   web foundation (lint boundaries, state, persistence, net, analytics)
                                                                          │
                                         wasm-core → loader/pool ← capability check
                                                                          ▼
                                                 copy → usecases → components → pages
                                                                          ▼
                                        check scripts → E2E → CI → Docker/Render → Vercel deploy → experiments → exit
```

## Phases

| Phase | Prompts |
| --- | --- |
| A Repository & workspaces | 01–02 |
| B Shared types & codegen | 03–08 |
| C Server | 09–13 |
| D Web foundation & capability | 14–18 |
| E WASM path | 19–20 |
| F UI & use-cases | 21–23 |
| G Checks, E2E, CI | 24–26 |
| H Deploy, measure, exit | 27–30 |

## Prompt dependency graph

`01 → 02 → 03 → 04 → 05 → 06 → 07 → 08 → 09 → 10 → 11 → 12 → 13 → 14 → 15 → 16 → 17 → 18 → 19 → 20 → 21 → 22 → 23 → 24 → 25 → 26 → 27 → 28 → 29 → 30` Parallelism allowed: Prompts 09–13 (server) need only 07, and 14–18 (web) need only 08; run in order for simplicity. Experiments TE-6, TE-7 and TE-9 are *started* by the human before Prompt 01, TE-6 and TE-7 are *decided* before Prompt 27, and Prompt 29 records every outcome.

| Prompt | Major deliverable | Depends on | Produces |
| --- | --- | --- | --- |
| 01 | Preflight, git, folders, fixed files | — | Clean repo, docs in `docs/` |
| 02 | Cargo + pnpm workspaces, lints, bans | 01 | Valid empty workspaces |
| 03 | `units.rs`, `ids.rs`, `limits.rs` | 02 | Branded unit types, 21 limits |
| 04 | `stage`, `media`, `transcript`, `events`, `prosody` | 03 | Domain data types |
| 05 | `edit`, `profile`, `summary`, `error`, `lib`, trybuild | 04 | Complete `offcut-types` |
| 06 | `auth`, `account`, `billing`, `usage`, `errors` DTOs | 05 | API DTOs |
| 07 | `analytics.rs` allowlist | 06 | 19-event allowlist + docs |
| 08 | Codegen scripts + generated TS | 07 | `gen/domain.ts`, `gen/api.ts` |
| 09 | `config`, `log`, `error`, `headers`, `client_ip` | 07 | Server primitives |
| 10 | `rate_limit`, `state` | 09 | Limiter, `AppState` |
| 11 | Migration, `db/*`, `lib.rs`, sqlx cache | 10 | Full schema, repo fns |
| 12 | Routes + router + main (vertical slice) | 11 | Running API, 3 routes |
| 13 | Server integration tests | 12 | Green `cargo test -p offcut-api` |
| 14 | Web tooling, ESLint boundaries, `vercel.json` | 08 | Lint-guarded web shell |
| 15 | `env`, hosts, `protocol`, `transition`, store, persistence | 14 | State/persistence layers |
| 16 | `http.ts` cold-start, api-client, asset-fetch + tests | 15 | Network layer |
| 17 | Analytics client | 16 | `track`/`flush` |
| 18 | Capability check + encoders + tests | 15 | `runCapabilityCheck` |
| 19 | `offcut-wasm-core` + `build-wasm.sh` | 05 | WASM bundle |
| 20 | `load-core`, `pool.preload`, browser proof | 19, 14 | WASM compiles under CSP |
| 21 | Copy, `submit-notify-me`, `start-app` | 17, 18, 20 | Use-cases |
| 22 | Styles + components | 21 | UI parts |
| 23 | Pages, routes, entry, local e2e chain | 22 | Working local app |
| 24 | Four `check-*` scripts | 23 | Gates |
| 25 | Playwright suite | 24 | `landing.spec.ts` |
| 26 | Root scripts + `ci.yml` steps 1–9 | 25 | Green PR CI |
| 27 | Dockerfile, `deploy-api.yml`, `render.yaml`, `upload-assets.sh` | 26 + database, asset host and Vercel project chosen (Human) | API live on Render; demo clips on the asset host |
| 28 | Real hosts, Vercel prebuilt deploy, CI steps 10–14 | 27 | Web live, smoke green |
| 29 | Measurements on the deployment + uptime monitor + records | 28 | `docs/v1/experiments.md` |
| 30 | Exit audit, §15.3, §16, tag `v1` | 29 | Production V1 |

---

# Prompts

## Prompt 01 — Preflight and repository skeleton

**Objective.** Verify the machine, start the repo, create the small fixed files. **State before.** Folder `sh2clips` containing `documents/` with the specs. **Why here.** Everything needs a Git repo and correct line endings. **Implement.**

- Run the G§0.1/Milestone-0 checks: `uname -s` must say `MINGW64_NT`; versions of `rustc`, `wasm-bindgen`, `wasm-opt`, `cargo-deny`, `sqlx`, `node`, `pnpm`, `psql` against local DB `postgres://offcut:offcut@localhost:5432/offcut_dev`. Install what is missing per G§0.2–0.4 (update Rust; `wasm-bindgen-cli` pinned to the version you will use as the crate; binaryen).
- `mv documents docs`; `git init -b main`; `git config core.autocrlf input`.
- `.gitignore` per §12.8 (do **not** ignore `web/src/gen/` or `server/.sqlx/`).
- `docs/file-specs/TEMPLATE.md` (TS §34.2), `fixtures/speech/README.md` (reading scripts: \~150 words, two numbers, one three-item list, one from-to, one emphasized word), `bench/results/.gitkeep`, empty `docs/v1/` kept (specs placed there per §4). **Human, before this prompt.** The first row of "Who does what": accounts opened, TE-6 and TE-7 started, TE-9 submitted, dev keys generated. Report any of these that is still open. **Not yet.** Any Cargo/pnpm files. **Files.** `.gitignore`, `docs/**`, `fixtures/speech/README.md`, `bench/results/.gitkeep`. **Tests/validation.** Milestone-0 command block from the guide. **Done when.**
- [ ] Every tool prints a version; the `psql` check returns one row.
- [ ] `git status` clean after first commit; `docs/` holds the three specs; `.gitignore` ignores the right paths.
- [ ] Rust toolchain version noted in your report (used in Prompt 02).

## Prompt 02 — Cargo and pnpm workspaces

**Objective.** Valid, empty, lint-guarded workspaces (S1). **State before.** Prompt 01. **Implement.**

- Create crate stubs first: `cargo new --lib --vcs none` for `crates/offcut-types`, `crates/offcut-api-types`, `crates/offcut-wasm-core`; `cargo new --bin --vcs none --name offcut-api server`.
- Root `Cargo.toml` per §12.1/G§1.3: resolver 2, four members, `[workspace.dependencies]` (every shared dep once, via `cargo search` for current versions; `wasm-bindgen = "=<CLI version>"`), `[workspace.lints.clippy]` denying `unwrap_used, expect_used, panic, indexing_slicing, wildcard_enum_match_arm`, release profile `lto`, `codegen-units = 1`, **no `panic = "abort"`**. Each crate: `[lints] workspace = true`.
- `rust-toolchain.toml` (pinned channel, wasm32 target, rustfmt+clippy), `clippy.toml` (disallow `Instant::now`/`SystemTime::now`; allow unwrap/expect/panic/indexing in tests), `deny.toml` via `cargo deny init` then §12.1 bans (wrapper rule for `web-sys, js-sys, wasm-bindgen, wgpu, rand, getrandom`; `offcut-api` may not depend on the six media crates). License allowlist is finalised in Prompt 13.
- `pnpm-workspace.yaml` (`web`), root `package.json` (`packageManager`, `engines`, empty scripts), `web/package.json` stub, `pnpm install`. **Not yet.** Any real source code, deps beyond the workspace table. **Done when.**
- [ ] `cargo metadata` ok; `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` pass on stubs.
- [ ] `pnpm install` writes `pnpm-lock.yaml`; tree clean after commit including lock files.
- [ ] `wasm-bindgen` is pinned with `=` and equals the installed CLI.

## Prompt 03 — `offcut-types`: units, ids, limits

**Objective.** The type-safety foundation (§6.1–6.3, TS §10.1–10.2). Frozen after V1. **Implement.**

- Manifest: `serde` (derive), `ts-rs`, `uuid` (feature `serde` **only**), dev `trybuild`.
- `units.rs`: all 16 unit structs of TS §10.1 (`TimeMs, DurMs, Micros, FrameIdx, SampleCount, Hz, FpsMilli, Px, Bytes, BitsPerSec, Confidence, Lufs, Dbfs, UnixSecs, ExportCount, UsdCents`) + `Span`; private inner fields; `const fn new/get`; derive rules (§6 header); `Confidence/Lufs/Dbfs` as `f32` with `Option` constructors; custom `Deserialize` for `Confidence` through `new`; `FrameIdx::to_time_ms` (`n*1000/30`), `to_micros` (`n*1_000_000/30`), `TimeMs::to_micros`, `from_micros_floor` (floor, clamp negatives to 0), `checked_add`, `duration_since`, `Span::new`. No `From<u32>`.
- `ids.rs`: `ClipId, ExportId, UserId, AnonId` (Uuid-backed), `EventId(u64)`, `WordIdx, SentenceIdx, WordRange, JobId`; `new/get` only, no generation.
- `limits.rs`: exactly the 21 constants of TS §10.2, with PS refs in comments, nothing else. **Serialization.** Integer units/ids `#[serde(transparent)]`. **Tests.** The six unit tests of §6.7 in `units.rs`; serde round-trips for a unit and an id. **Done when.**
- [ ] `cargo test -p offcut-types` and clippy pass.
- [ ] `units.rs` has exactly 16 unit structs plus `Span`; `limits.rs` has exactly 21 `pub const`s; grep shows no `From<u32>` for any unit.
- [ ] No `unwrap/expect/indexing` in non-test code.

## Prompt 04 — `offcut-types`: stage, media, transcript, events, prosody

**Objective.** Domain data types copied verbatim from TS §10.3–10.5 with D-2 serialization. **Implement.**

- `stage.rs`: `PipelineStage` — `ProbeAudio, Asr, AudioChain, DetectScene, RenderEncode, Mux` (PS §20.2 order), `snake_case`.
- `media.rs`: `Rotation, Orientation, ContainerKind, VideoCodec, AudioCodec, ProbeInfo, VideoTrackInfo, AudioTrackInfo, ClipInfo, RejectReason` — 14 `REJECT_*` variants, each `#[serde(rename = "REJECT_…")]` in the order of §6.4.
- `transcript.rs`: `Word, Sentence, Unit, Quantity, NormalizedSpan, Transcript`; `Unit::Count { noun: String }` is the only data variant.
- `events.rs`: `EventKind, ListItem, EventParams` (`#[serde(tag="kind", rename_all="snake_case")]`), `DetectedEvent`. `prosody.rs`: `WordProsody, Prosody`. **Not yet.** `edit`, `profile`, `summary`, `error` (Prompt 05); no logic beyond constructors. **Tests.** Serialization-shape tests: `RejectReason` JSON strings, `EventParams` tagged form, `PipelineStage` snake_case. **Done when.**
- [ ] `cargo check/test -p offcut-types` green; all derives per §6.
- [ ] Count of `RejectReason` variants = 14; names match §6.4 exactly.

## Prompt 05 — `offcut-types`: edit, profile, summary, error, lib, compile-fail test

**Objective.** Finish the crate (§6.4–6.7). **Implement.**

- `edit.rs`: `StyleId, CropOffset (new → None outside -1.0..=1.0), EditState` with `Default` = no edits, `StyleId::Clean`, offset 0.0. `profile.rs`: `Plan, ProfileKind, ExportProfile`. `summary.rs`: `ChangeSummary`.
- `error.rs` (D-1): `ErrorCode` (29, TS §11.2 order, each `rename = "E_…"`), `UnsupportedReason` (11, check order of TS §13.2, `UNSUPPORTED_…`), `FailureStage` (11, `snake_case`), `From<PipelineStage> for FailureStage` (one arm per stage, no wildcard).
- `lib.rs`: declare 12 modules, re-export public items at root; `#[cfg(test)]` round-trip tests for the three code enums (string form is the contract).
- `tests/ui.rs` + `tests/ui/bare_ms_rejected.rs`: compile-fail for bare `u32`→`DurMs` and `TimeMs`→`DurMs`. Generate `.stderr` once with `TRYBUILD=overwrite cargo test -p offcut-types --test ui`; **open it and confirm** it shows type mismatches, not a missing import. **Not yet.** `EntitlementClaims` (V2), generator test (Prompt 08). **Done when.**
- [ ] `cargo test -p offcut-types` green including `ui`; counts 29 / 11 / 11 verified by a test.
- [ ] `.stderr` committed and note added: regenerate when the toolchain changes.

## Prompt 06 — `offcut-api-types`: DTOs and API errors

**Objective.** Request/response types (§7.1, §7.6, TS §10.7). **Implement.**

- Manifest: `serde`, `ts-rs`, `offcut-types`.
- `auth.rs`, `account.rs` (incl. `Wanted`: `launch, safari, firefox, mobile, linux`; `NotifyMeRequest`; `ExportedUser/Session/Receipt` per §7.1 V1 decision), `billing.rs`, `usage.rs`: copy TS §10.7 verbatim. Every request struct has `#[serde(deny_unknown_fields)]`.
- `errors.rs`: `ApiError { code, retry_after_secs }`, `ApiErrorCode` with the 13 members of D-3, `snake_case`. **Tests.** Round-trip for each DTO; unknown field on `NotifyMeRequest` is rejected; unknown `wanted` rejected. **Done when.**
- [ ] `cargo test -p offcut-api-types` green; 13 `ApiErrorCode` members; every request struct denies unknown fields.

## Prompt 07 — `offcut-api-types`: the analytics allowlist

**Objective.** The privacy-critical event schema (§7.2–7.5). Frozen after V1. **Implement.**

- Constants `MAX_EVENTS_PER_BATCH=50`, `MAX_DURATION_MS=3_600_000`, `MAX_COUNT=10_000`.
- 16 prop enums (`snake_case`) with exactly the values in §7.2.
- `AnalyticsEvent` with `#[serde(tag="name", content="props", rename_all="snake_case", deny_unknown_fields)]` and the 19 variants of §7.3; optional props `Option<T>` with `default` + `skip_serializing_if`; unit variants serialize with **no** `props` key.
- `EventsBatch { anon_id, events }` (`deny_unknown_fields`); `EventDoc`; `ANALYTICS_EVENT_DOCS` (19 entries, enum order, one plain-language sentence each); `AnalyticsEvent::name()` via exhaustive match, no wildcard. **Invariant.** No variant has a `String` field. **Tests (§7.5).** Each of 19 variants serializes to `{"name","props"}` and round-trips; `name()` set equals docs names in order, no duplicates; unknown prop / unknown name / string-for-enum all fail to deserialize. **Done when.**
- [ ] `cargo test -p offcut-api-types` green; grep proves no `String` in `analytics.rs` event fields.
- [ ] 19 variants, 19 docs, 16 prop enums.

## Prompt 08 — Code generation to TypeScript

**Objective.** Deterministic generation of `web/src/gen/*.ts` (§9, D-4, D-5, G§2.5–2.6). **Implement.**

- In each crate's `lib.rs`: `#[ignore]`d test `write_typescript` that builds one string and writes it to the path in `OFFCUT_GEN_OUT`, resolved against the workspace root (`CARGO_MANIFEST_DIR/../..`). `\n` endings, fixed order, no timestamp.
- `domain.ts` content in order: header; branded aliases from a name table (`number & {readonly __unit:"X"}`; Uuid ids as branded `string`; `EventId` as branded decimal `string`); all other types via `ts-rs`; `LIMITS` (`as const`, branded values, 21 props); `ERROR_CODES`, `REJECT_REASONS`, `UNSUPPORTED_REASONS` const arrays.
- `api.ts`: header; `import type` from `./domain`; all section-7 types; `ANALYTICS_EVENT_DOCS`; `MAX_EVENTS_PER_BATCH`.
- `scripts/gen-types.sh`, `scripts/check-gen-clean.sh` (`git diff --exit-code -- web/src/gen`); root script `gen:types`. **Done when.**
- [ ] Run `gen-types.sh` twice → no diff; commit `web/src/gen/`; `check-gen-clean.sh` prints success.
- [ ] Counts in files: `ERROR_CODES` 29, `REJECT_REASONS` 14, `UNSUPPORTED_REASONS` 11, `LIMITS` 21, docs 19.
- [ ] Works from Git Bash with LF endings.

## Prompt 09 — Server primitives: config, log, error, headers, client_ip

**Objective.** The non-routing building blocks of `offcut-api` (§10.1–10.5). **Implement.**

- Server manifest per §10 (sqlx features `postgres, runtime-tokio, tls-rustls, uuid, time, json, migrate, macros`; `uuid` with `v4, v7, serde`; dev `reqwest`). **Create `server/src/lib.rs`** (module declarations only) and keep `main.rs` thin — required for integration tests (G "Read this first" #2). **Add `server/src/lib.rs` to the TS §5 tree and to §4 in the same commit.**
- `config.rs`: `Config`, `ConfigError{Missing,Malformed}` (name only, never value), `from_env`; all §3.3 variables required except `GIT_SHA` (default `"dev"`); empty = missing; Ed25519 seed base64 validated; `APP_ORIGIN` `https://host` or `http://localhost:port`; hand-written `Debug` printing `<redacted>` for keys/secrets. The only reader of the environment.
- `log.rs`: JSON tracing, `request_span` (request id, method, **route template** via `MatchedPath`, status, latency; never email/token/cookie/body/IP/query), `user_tag`.
- `error.rs`: `AppError` + `IntoResponse` per the §10.3 table, `json_rejection` mapping (size→413, content-type→415, any parse failure→400); body never echoes request.
- `headers.rs`: layer adding `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`; no CORS.
- `client_ip.rs`: algorithm of §10.5 incl. D-17. **Tests.** config (complete env, each missing var, bad key, Debug has no secret); `client_ip` (1/2/3 entries × hops 1/2, absent header, garbage, IPv6, spaces). **Done when.**
- [ ] `cargo test -p offcut-api` unit tests green; clippy clean.
- [ ] `Config` Debug test proves no secret leaks.

## Prompt 10 — Rate limiter and app state

**Objective.** §10.6–10.7. **Implement.**

- `rate_limit.rs`: `RouteGroup` (all 16 groups), `Limit`, `limit()` const fn encoding the whole TS §22.1 table; `RateKey{Ip,EmailHash,User}`; `RateLimiter::new(max_keys)`/`check(group,key,now)` token bucket, continuous refill, `Err(retry_after_secs)` rounded up, LRU eviction at `max_keys`; `by_ip` middleware. Time via `tokio::time::Instant`.
- `state.rs`: `AppState { db, config: Arc<Config>, limiter: Arc<RateLimiter> }`. **Tests.** Capacity honoured; 61st request in a minute refused with plausible retry; refill after `tokio::time::advance`; IPs and groups don't share buckets; eviction keeps newest. **Done when.**
- [ ] Tests green with `tokio::time::pause`; every group has a limit; no `std::time` clock used.

## Prompt 11 — Database layer: migration, repositories, offline cache

**Objective.** Full schema now (D-7), tiny repo layer (§10.8, G§3.3, G§3.6). **Implement.**

- `migrations/0001_init.sql`: TS §23.2 **exactly** (8 tables, indexes, CHECKs). Apply to local DB, confirm `\dt` shows 8 + `_sqlx_migrations`.
- `db/mod.rs` (`connect` max 5 connections, `migrate` via `sqlx::migrate!`), `db/analytics_events.rs` (`insert_batch` single statement/transaction, `purge_older_than`), `db/platform_waitlist.rs` (`upsert … ON CONFLICT (email_normalized, wanted) DO NOTHING`).
- Use `sqlx::query!`. Run `cd server && cargo sqlx prepare -- --all-targets` (**not** `--workspace`); commit `server/.sqlx/` (nothing at repo root). Verify `SQLX_OFFLINE=true cargo check -p offcut-api`. **Constraint.** `analytics_events` columns are exactly `id, anon_id, name, props, ts`; `ts` defaults to `now()`. **Tests.** None in this prompt: the repository functions are covered by Prompt 13's integration tests. Here, prove that the code compiles and the migration applies. **Done when.**
- [ ] Migration applies cleanly twice (idempotent via sqlx); `.sqlx/` present in `server/`.
- [ ] Offline build passes.

## Prompt 12 — Routes, router and `main` (first vertical slice)

**Objective.** A running API with three live routes (§10.9–10.12). **Implement.**

- `auth/mod.rs`, `auth/magic_link.rs`: `normalize_email` only (D-11): trim+lowercase; `Some` iff 3–254 bytes, exactly one `@`, non-empty local part, domain containing a dot, no whitespace/control chars. Six unit tests.
- `account/notify.rs`, `account/mod.rs`: `POST /notify-me` → 204 whether or not the row existed; bad email → 400.
- `analytics/mod.rs` `ingest` (5 ordered steps, whole-batch rejection, bounds on `*_ms` and counts, store only `anon_id, name, props, server time`); `analytics/retention.rs` (`spawn_purge_task` at start then every 6 h; `purge_once` deletes rows older than `offcut_types::ANALYTICS_RETENTION_DAYS`, which already exists in `limits.rs`: do not define a second constant or type the number 90).
- `router.rs`: `/healthz` returning `{"ok":true,"version":"<sha>"}` touching no DB (D-14); the 7 layers in §10.11 order (request id, trace, `CatchPanicLayer`, headers, media guard → 415 for `multipart/`, `video/`, `audio/`, `application/octet-stream`, default 16 kB body limit with per-route 1 kB for notify-me, per-route rate limit); 404 fallback with `code:"not_found"`; the twelve V6 routes as comments.
- `main.rs`: 8 steps of §10.12; config error prints `config error: <NAME>` to stderr, exit 1; graceful SIGTERM. **Not yet.** Any V6 route, mailer, billing. **Validate.** G§3.4 curl checks and Milestone-3 curls (204/204/400/415/404) and psql row checks. **Done when.**
- [ ] `env -u DATABASE_URL cargo run -p offcut-api` exits 1 naming the variable.
- [ ] All five curl status codes match; rows appear only as expected; no IP/user-agent stored.

## Prompt 13 — Server integration tests and license policy

**Objective.** Real-Postgres tests for every V1 route (§13.1–13.3, D-12). **Implement.**

- `tests/common/mod.rs`: `TestApp::spawn()` — fresh randomly named DB per test (role has `CREATEDB`), run migrations, fixed test `Config`, bind `127.0.0.1:0`.
- `tests/analytics_allowlist.rs`: all 14 cases of §13.2 (including the schema-columns assertion and the purge 91/89-day case and the XFF/User-Agent non-storage case).
- `tests/notify_me.rs`: all 10 cases of §13.3 (healthz headers, idempotent upsert, normalization, 400s, 413, sixth request 429 with `Retry-After`, 404).
- Both files start with the `#![allow(clippy::unwrap_used, …)]` line. Run `cargo deny check` and finalise the `deny.toml` license allowlist. **Done when.**
- [ ] `set -a; . ./.env; set +a; cargo test -p offcut-api` green, tests run in parallel.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` and `cargo deny check` pass.

## Prompt 14 — Web tooling and lint boundaries

**Objective.** A configured, strictly-linted web shell (§12.3–12.4, G§4.1–4.2). **Implement.**

- Dependencies per G§4.1 (+ `@webgpu/types`, `@types/dom-webcodecs`; remove the latter if `tsc` shows duplicates). `web/package.json` scripts `dev`, `build:vite`, `preview`.
- `tsconfig.json` (`strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, bundler resolution, libs incl. `WebWorker`), `vite.config.ts` (React plugin; dev headers COOP+COEP only; proxy `/api/v1`→`:8080`; **preview headers read from `vercel.json`**; `assetsInlineLimit: 0`; `worker.format:"es"`; target esnext), `vitest.config.ts`, `index.html` (no inline script/style), `public/robots.txt`, `.env.local` (git-ignored) with `VITE_ASSET_BASE_URL`.
- `vercel.json` per §12.4 **now** (placeholders for `<render-host>` and `<ASSET_ORIGIN>`).
- `eslint.config.js`: **every** TS §7 rule on from day one — boundary matrix, restricted globals with overrides for the four fetch files, `import.meta.env` and brand-cast and empty-catch restrictions, `react/jsx-no-literals` in `ui/`, no-any/non-null/ts-comment, switch exhaustiveness, `no-console`, type-only import of `workers/protocol` allowed from `net/`.
- Temporary `src/main.tsx` rendering an empty div. **Validate (must FAIL then be reverted).** `fetch` in `state/`; `import.meta.env` in `net/http.ts`; `1000 as TimeMs` in `net/`; `state/` importing `net/`. **Done when.**
- [ ] `tsc --noEmit` and `eslint .` green; all four deliberate violations were rejected.

## Prompt 15 — Web foundation: config, protocol, state, persistence

**Objective.** Pure/low-level layers (§11.2–11.5). **Implement.**

- `config/env.ts` (throws at load if `VITE_ASSET_BASE_URL` missing/non-https/has query), `config/allowlist-hosts.ts`.
- `workers/protocol.ts`: TS §14.1–14.2 types only (D-8); no function, no constant; `FailureStage` re-exported from `gen/domain`.
- `state/machines/transition.ts` (dev/test throws `IllegalTransitionError`; prod returns state and calls injected reporter), `state/capability-store.ts` (3 legal transitions; `CapabilityReport` = generated props type of `capability_check`).
- `persistence/schema.ts` (DB `offcut`, v1, **all eight stores**, `META_KEYS`, `Versioned<T>`, `OffcutDb`), `persistence/db.ts` (memoized `openDb`, `metaGet/metaSet`, numbered upgrades, newer `schemaVersion` treated as absent and deleted). `persistence/` imports no `net/ui/workers`. **Tests.** No test file in this prompt. `transition.test.ts`, a capability-store test and a `db.ts` test are not among V1's six test files and are not in the TS §5 tree. These files are checked by `tsc` and ESLint now and exercised by `landing.spec.ts` in Prompt 25; `transition()` gets its own tests with the machine tests of V2. **Done when.**
- [ ] `tsc` and ESLint green; `state/` imports only `gen` types; no new `*.test.ts` file.

## Prompt 16 — Network layer: `http.ts` cold-start, api-client, asset-fetch

**Objective.** The only app-origin `fetch` (§11.6, §13.5, D-10). **Implement.**

- `net/http.ts`: constants (10 s timeout, 70 s cold-start budget, 3 s waking notice); `createHttp(deps)` with injectable `fetch/now/sleep/onLine`; behaviour table of §11.6 (offline, `TypeError`, timeout/502/503/504 with 1,2,4,8,then 8 s retries within budget, 429 not retried carrying `retry_after_secs`, other 5xx retried then `E_API_5XX`, 4xx not retried with parsed `apiError`, 204→`undefined`); `credentials:"same-origin"`, `cache:"no-store"`, JSON header only with body; `stage:"api"`; `detail` dev-only; `toAppFailure`.
- `net/api-client.ts`: `wake()`, `postEvents` (retry none, keepalive), `postNotifyMe` (cold-start).
- `net/asset-fetch.ts`: `assetUrl` only (rejects `?` and `..`); no `fetch` yet. **Tests.** All 12 cases of §13.5 using fake fetch/clock/sleep. **Done when.**
- [ ] `http.test.ts` green; `onWaking` is called exactly once and never for a response within 3 s; URLs are never absolute.

## Prompt 17 — Analytics client

**Objective.** §11.7. **Implement.** `analytics/client.ts`: `FLUSH_INTERVAL_MS=10_000`, `FLUSH_AT=20`, `initAnalytics`, `track(AnalyticsEvent)` (never throws/awaits; events before init wait in queue), `flush` (≤50 events via `postEvents`; failures dropped — the one permitted empty `catch`), flush on `visibilitychange` hidden. Imports no store, no persistence; anon id is passed in. **Tests.** No test file in this prompt. `client.test.ts` and the type-level `events.types.test.ts` are in the TS §5 tree but belong to M2.2 (V8), not to V1's six test files. In V1 the client is checked by `tsc` (`track` accepts only the generated union) and by the `landing_view` and `capability_check` cases of `landing.spec.ts` in Prompt 25. **Validate (must FAIL, then revert).** A `track({ name: "landing_view", props: { hero_variant: "outcome", extra: 1 } })` call placed in any file must make `tsc` fail. **Done when.**

- [ ] `tsc` and ESLint green; the deliberate off-allowlist call was rejected by `tsc`; the ESLint boundary proves `analytics/` imports no `state/` or `persistence/`; no new `*.test.ts` file.

## Prompt 18 — Capability check and encoder constants

**Objective.** J3/E-8 measurement (§11.8, §13.4). **Implement.**

- `platform/simd-probe.ts` using the known-good bytes in G§4.4.
- `workers/render/encoders.ts`: `VIDEO_ENCODE_LADDER` (4 entries, TS §21.2), `AAC_ENCODE_CONFIG`, `KEYFRAME_INTERVAL_FRAMES`, `videoConfigFor`; no browser API touched at import.
- `platform/capability.ts`: `PlatformProbe`, `browserProbe`, `runCapabilityCheck`: all 11 checks started together, each with a 1 s timeout (timeout/throw = fail), reason = first failure in `UnsupportedReason` order, memory/GPU/platform mapped to enums only (no other device attribute read). **Tests.** All 11 groups of §13.4 with a fake probe, fake timers for the 3 s budget. **Done when.**
- [ ] `capability.test.ts` green; every report value is a member of its generated enum.

## Prompt 19 — `offcut-wasm-core` and `build-wasm.sh`

**Objective.** Rust→wasm32→wasm-bindgen→wasm-opt (§8, §12.6, G§5.1–5.2). **Implement.**

- Crate: `cdylib`; deps `wasm-bindgen` (workspace `=` pin), `offcut-types`. `#[wasm_bindgen(start)] init()` installs a panic hook (idempotent) that throws a JS error whose message **starts with `E_WORKER_CRASH`** and logs nothing in release; `core_version()` returns `CARGO_PKG_VERSION`. No product logic.
- `scripts/build-wasm.sh`: `BUNDLES` list (V1: `offcut-wasm-core:core`); build with `RUSTFLAGS=-C target-feature=+simd128`; `wasm-bindgen --target web --out-name offcut_core --out-dir web/src/wasm/pkg/core`; `wasm-opt -O3` with required `--enable-*` flags; **fail if CLI version ≠ `wasm-bindgen` version in `Cargo.lock`**; `--dev` skips wasm-opt; `--watch` rebuilds. Root script `build:wasm`. **Validate.** Temporarily break the version check once; script must exit non-zero. **Done when.**
- [ ] `pnpm build:wasm` produces `offcut_core.js`, `offcut_core_bg.wasm`, `.d.ts`; workspace clippy clean; deny wrapper bans not violated.

## Prompt 20 — WASM loader, `pool.preload`, browser proof

**Objective.** Prove WASM under the production CSP in week 1 (§11.9, D-9, G§5.3–5.4). **Implement.** `wasm/load-core.ts` (`preloadCore` = fetch + `compileStreaming`, cached, idempotent; `loadCore` instantiates from cache; `.wasm` imported as Vite `?url`), `workers/pool.ts` (`preload()` → `{ok:true}` or `{ok:false, failure:{code:"E_WORKER_CRASH", stage:"import", retryable:true}}`). Do not instantiate on the main thread. Temporarily call `preload()` from `main.tsx`. **Validate.** Dev page and `vite build && vite preview` (production CSP): Network shows `.wasm` 200 `application/wasm`; console has no CSP message; `crossOriginIsolated === true`. **Done when.**

- [ ] `tsc` green after `pnpm build:wasm`; all five Milestone-5 rows pass; temporary call is noted for removal in Prompt 21.

## Prompt 21 — Copy, use-cases, app start

**Objective.** All user-facing words and orchestration (§11.10–11.11). **Implement.**

- `copy/messages.ts`: sections `landing, dropZone, capability, unsupported (all 11 reasons), notifyMe, api, settings, errors (6 codes)`; numbers only via `LIMITS`; never the phrases "never leaves", "GDPR", "DPDP", "CCPA", "SOC 2", "compliant".
- `usecases/submit-notify-me.ts`: trim, no `@` → `invalid_email` without a request; mapping of results per §11.10; **no analytics for waitlist**.
- `usecases/start-app.ts`: idempotent; 7 steps in order (reporter wiring; anon id with in-memory fallback; `initAnalytics`; `landing_view` outcome; capability check + `capability_check` event; `wake()`; supported-only `pool.preload()` with `client_error` on failure). Remove the temporary preload from Prompt 20. **Tests.** No test file in this prompt: `submit-notify-me.test.ts` and `start-app.test.ts` are not in the TS §5 tree. The result mapping is covered by the four waitlist cases of `landing.spec.ts` (success, invalid, 429, waking) and the start order by its `landing_view` and `capability_check` cases, all in Prompt 25. `messages.test.ts` arrives in V8; the banned-phrase rule applies now. **Done when.**
- [ ] The existing Vitest suites, `tsc` and ESLint are green; every `UNSUPPORTED_*` has copy; a grep of `messages.ts` finds none of the banned phrases; no new `*.test.ts` file.

## Prompt 22 — Styles and components

**Objective.** §11.13–11.14. **Implement.** `tokens.css`, `pages.module.css`, `components.module.css` (system fonts, no CDN, no CSS-in-JS); `WhatLeavesTable` (4 fixed PS §12.7 rows + one row per `ANALYTICS_EVENT_DOCS`), `NotifyMeForm` (states `idle, sending, waking, done, error`; **script submit only**, `form-action 'none'`), `DropZone` (visible, `aria-disabled`, shows `notReady`, **never reads the dropped `File`**), `UnsupportedPage`, `CapabilityGate`. **Done when.**

- [ ] ESLint (incl. `jsx-no-literals`) and `tsc` pass; no inline styles; components read only `usecases/state/copy/config`.

## Prompt 23 — Pages, routes, entry files, local end-to-end chain

**Objective.** Working local app (§11.1, §11.12, G Phase 6). **Implement.** `LandingPage`, `EditorPage` (placeholder, D-16), `SettingsPage`, `routes.tsx` (`ROUTES`, 3 of 7 routes mounted, `*`→`/`), `App.tsx`, final `main.tsx` (`startApp()` once, not awaited). Root `dev` script using `sh -c` (Windows-safe); demo `<video crossorigin="anonymous">` from `assetUrl`. The demo clips are uploaded in Prompt 27, so until then the video element points at a path that does not load; that is expected and is the only allowed console error. **Validate (Human, in Chrome).** Run API + `pnpm dev`; perform the full Milestone-6 table (landing content, capability line, notify success/invalid, events 204, drop-zone message, `/settings` 4+19 rows, `/app`, mobile emulation, API-down waking message) and the two psql queries. **Done when.**

- [ ] Every Milestone-6 row passes; the local `analytics_events` rows show only enum props; no console error other than the missing demo video.

## Prompt 24 — Gate scripts

**Objective.** Mechanical enforcement (§12.6, D-13, G§7.1). **Implement.**

- `check-file-tree.mjs`: parse the fenced TS §5 tree; fail on unlisted tracked source files under the listed roots (ignore lock files, `server/.sqlx/**`, `*.stderr`, fixtures/snapshots); fail on non-test files >400 lines; fail on disallowed `offcut-*` crate deps (TS §7); listed-but-missing is OK before V10.
- `check-copy-codes.mjs` with `COPY_PENDING` (all `REJECT_*`; every `E_*` except the six of §11.11); fails if a code has neither copy nor pending entry, or if a pending code already has copy; all 11 `UNSUPPORTED_*` required.
- `check-hosts.mjs`: scan `web/dist` for `http(s)://` hosts other than the asset host (with a justified `NON_NETWORK_LITERALS` list); fail if the `vercel.json` CSP hosts ≠ `VITE_ASSET_BASE_URL`; print gzip size of the shell.
- `check-external-facts.mjs`: print TE-5…TE-11 with last-checked dates. **Validate (each must fail, then revert).** Add `web/src/stray.ts`; delete one `UNSUPPORTED_*` message; put `https://example.com` in a string and build. **Done when.**
- [ ] Three deliberate failures caught; all four scripts pass on the clean tree (including that `server/src/lib.rs` is in the §5 tree).

## Prompt 25 — Playwright suite

**Objective.** §13.6–13.7, G§7.3. **Implement.** `playwright.config.ts` (channel `chrome`, baseURL `E2E_BASE_URL ?? http://localhost:4173`, `vite preview` webServer unless overridden, project `non-media`); `helpers/fake-api.ts` (`page.route("**/api/v1/**")`, records requests, configurable status/delay); `helpers/fixtures.ts` (`flushAnalytics`); `landing.spec.ts` with the 12 cases (J1 content, settings link, `landing_view` hero `outcome`, `capability_check` all-enum props without assuming `pass`, waitlist success/invalid/429/waking(4 s), inactive drop zone, `/app` never blank, no stray traffic, `@smoke`). **`@smoke` case.** Blocks `/api/v1/events`; asserts `crossOriginIsolated`, a `.wasm` 200 `application/wasm`, no `securitypolicyviolation`, `GET /api/v1/healthz` 200. **Done when.**

- [ ] `pnpm build && pnpm e2e` green locally; no test touches a real server.

## Prompt 26 — Root scripts and CI steps 1–9

**Objective.** One local command per gate and identical PR gates (§12.2, §12.7, G§7.2/7.4). **Implement.** Root `package.json` scripts: `dev, gen:types, build:wasm, build:vite, build, check, test, e2e` per G§7.2 (`sh`, not `bash`). `.github/workflows/ci.yml` steps 1–9: caches + secret scan; fmt/clippy/deny; gen + drift check; `cargo test --workspace` with Postgres service and `SQLX_OFFLINE=true`; `build:wasm` with the same `wasm-bindgen-cli` and binaryen versions as local; ESLint/tsc/file-tree/copy-codes; vitest; `vite build` + `check-hosts` with repo variable `VITE_ASSET_BASE_URL`; Playwright `non-media`. **Not yet.** `main`-only steps 10–14. **Validate.** `set -a; . ./.env; set +a; pnpm check && pnpm test && pnpm build && pnpm e2e`; open a PR. **Done when.**

- [ ] The four-command local run passes in one go; CI is green on the PR; merge.

## Prompt 27 — API container and Render deploy

**Objective.** The API live on Render (§12.5, G§8.1–8.6). **Human, before this prompt.** Three values must exist, because this prompt and Prompt 28 consume them:

- The production `DATABASE_URL`: choose between the two Postgres candidates (TE-6, G§8.1).
- The asset host's public base URL, with CORS set for the app origin: choose between the two candidates (TE-7, G§8.2).
- The Vercel production URL `https://<project>.vercel.app`: run `vercel link` from the repo root, set Root Directory `web` and Framework "Other", do **not** connect the Git repository (G§8.3). Render's `APP_ORIGIN` is this URL.

If any is missing, stop and report it. **Implement (agent).** `server/Dockerfile` (pinned Rust slim build stage with `SQLX_OFFLINE=true`, `ARG GIT_SHA`; pinned Debian slim runtime with ca-certificates, non-root user, `ENV GIT_SHA`), `.github/workflows/deploy-api.yml` (triggers `workflow_call` and `workflow_dispatch`; build with `--build-arg GIT_SHA`; push `:<sha>` and `:latest` to GHCR; call the Render deploy hook), `render.yaml` (one web service, `runtime: image`, `plan: free`, `healthCheckPath: /api/v1/healthz`, `sync:false` for every §3.3 variable; no disk/worker/cron), `scripts/upload-assets.sh` (content-hashed upload with immutable cache header, prints the public path). After the human has uploaded the demo clips, put the printed paths into the `assetUrl(...)` calls of `LandingPage` and `UnsupportedPage`. **Human, in this order.**

1. Start Docker Desktop for the local image check.
2. Run `deploy-api.yml` once by hand. The image push succeeds; the deploy-hook step fails because the service does not exist yet. That is expected on this first run only.
3. Make the GHCR package `offcut-api` public.
4. Create the Render service from `render.yaml` and enter the variables: `DATABASE_URL` and `APP_ORIGIN` from the prerequisites, two **new** signing keys (`openssl rand -base64 32`, pasted into Render and nowhere else), `unset-until-v6` for the seven mail and billing variables, `FOUNDING_OFFER_ENABLED=false`, `LOG_LEVEL=info`, `TRUSTED_PROXY_HOPS=1` for now.
5. Copy the Render host name and the deploy hook URL; set the GitHub secret `RENDER_DEPLOY_HOOK_URL`; run `deploy-api.yml` again and confirm the hook step passes.
6. Upload the three demo clips with `scripts/upload-assets.sh` and hand the printed paths to the agent.

**Validate.** Local `docker build` + `docker run` (G§8.4), then `curl https://<render-host>/api/v1/healthz` returns the commit SHA. **Done when.**

- [ ] Image runs locally and on Render; `/healthz` shows the SHA; no secret in git history.
- [ ] The second `deploy-api.yml` run is fully green; the demo video plays on the local page from the asset host.

## Prompt 28 — Real hosts and web deploy from CI

**Objective.** Production web, server-first ordering (§12.4, §12.7 steps 10–14, G§8.3/8.7/8.8). **Human.** Create a Vercel token; read `orgId` and `projectId` from `.vercel/project.json` (written by `vercel link` before Prompt 27); set the GitHub secrets `VERCEL_TOKEN, VERCEL_ORG_ID, VERCEL_PROJECT_ID` (`RENDER_DEPLOY_HOOK_URL` was set in Prompt 27) and the variable `VITE_ASSET_BASE_URL`; put the same asset URL in `web/.env.local`. **Implement (agent).** Fill `<render-host>` and both `<ASSET_ORIGIN>` entries in `vercel.json` with the values from Prompt 27; extend `ci.yml` (main only): call `deploy-api.yml`; poll `/api/v1/healthz` until `version == $GITHUB_SHA` (≤5 min); `vercel pull`, `vercel build --prod`, `vercel deploy --prebuilt --prod`; header check comparing every header of the `/(.*)` rule value-for-value; `E2E_BASE_URL=<url>` Playwright `--grep @smoke`. Vercel project is **not** Git-connected. **Done when.**

- [ ] `pnpm build` still passes `check-hosts` with real values; a merge to `main` runs 1–14 green; `/api/v1/healthz` answers through the Vercel rewrite with the current SHA; the header check and `@smoke` pass.

## Prompt 29 — Measurements, monitor and records

**Objective.** Measure the deployment and write every outcome down (§14, G§8.9). The database and asset host were already chosen before Prompt 27; this prompt records why, and runs the measurements that need the deployed page. **Implement (agent).** Create `docs/v1/experiments.md` (question, method, numbers, date, decision) and fill it from the numbers the human supplies; update the dates in `check-external-facts.mjs`; add, and the same day remove, the temporary TE-5 route and logging. **Human: measure and hand over the numbers.** TE-6 (both Postgres candidates: migration run, pool-of-5, plan terms, reconnect time; idle-week start date; which one was chosen and why), TE-7 (150 MB upload; from the **deployed** page, `Range: bytes=0-8388607` → 206 + `Content-Range`, readable under COEP; egress and payment-card terms; which one was chosen and why), TE-5 (60 s upstream delay survives the Vercel rewrite; `X-Forwarded-For` hop counts through Vercel and directly → set `TRUSTED_PROXY_HOPS` in Render), TE-11 (Render memory, 10 sleep/wake timings, instance-hours extrapolation), TE-9 (date the application was submitted), E-1 (page public with the three demo clips), E-8 (beacon live). Save Vercel's free-plan terms with the date read. Configure an uptime monitor on `GET https://<render-host>/api/v1/healthz` every 5 min. **Constraint.** The TE-5 sleep route and XFF logging are temporary: add, use, **remove the same day**; `notify_me.rs` must still pass (unknown paths 404). **Done when.**

- [ ] Four experiment outcomes + TE-9 recorded; `TRUSTED_PROXY_HOPS` set from data; monitor active; no temporary route in `main`.

## Prompt 30 — Exit audit, hand-over verification, tag `v1`

**Objective.** A verified, tagged, production V1 (§15.3, §16, G Milestone 8). **Implement / verify (with evidence for each).**

- Run `pnpm check && pnpm test && pnpm build && pnpm e2e`; `ci.yml` green on `main`; no file >400 lines.
- Production: `/healthz` SHA == `origin/main`; all 7 headers match `vercel.json`; `crossOriginIsolated`; `@smoke` passes; submit a real email on the deployed page and confirm the row in production `platform_waitlist`; confirm `landing_view` and `capability_check` rows contain only enum props.
- Walk §15.3 (V2 first-morning checklist) and §16 line by line; fix any §2 decision that changed, in this document and the specs (including the `server/src/lib.rs` addition and the `cargo sqlx prepare` invocation).
- `git tag v1 && git push origin v1`. **Not in scope.** Any V2 feature. **Done when.**
- [ ] Every box of §15.3 and §16 is ticked with evidence; tag `v1` pushed; report lists residual risks.

---

# Cross-Prompt Contracts

| Contract | Owner → consumer | Shape | Change rule |
| --- | --- | --- | --- |
| Shared types + serialization | P03–05 → everything | D-2 string forms (`E_*`, `REJECT_*`, `UNSUPPORTED_*`, snake_case enums) | Frozen; change only via Rust + regenerate + both sides |
| Analytics allowlist | P07 → server ingest (P12), `track` (P17), settings table (P22) | 19 events, no free text | Frozen; additions need allowlist + docs + tests together |
| Generated TS | P08 → all web code | `gen/domain.ts`, `gen/api.ts` | Never hand-edit; `check-gen-clean` guards |
| `AppError`/`ApiError` envelope | P09 → `http.ts` (P16) | `{code, retry_after_secs?}` | Frozen |
| Route/body/rate tables | P10, P12 → V6 | 16 groups, 3 live routes | V6 only uncomments |
| DB schema | P11 → V6 | `0001_init.sql` | Forward-only migrations |
| `HttpResult`/`AppFailure` | P16 → usecases | §11.6 | Frozen |
| `workers/protocol.ts` | P15 → V2 workers | types only | Frozen |
| IndexedDB schema v1 (8 stores) | P15 → V2/V3 | `schema.ts` | Frozen |
| CSP/headers | P14/P28 `vercel.json` → preview (D-15), check-hosts, header check | one source of truth | Edit only here |
| `LIMITS` | P03/P08 → copy (P21) | branded consts | Frozen |

# High-Risk Implementation Areas

| Risk | Why | Prompt | Validation |
| --- | --- | --- | --- |
| WASM under CSP / MIME / isolation | A V2 failure here is expensive | 19, 20, 28 | Preview-mode browser proof; `@smoke` |
| Analytics privacy leak | Core product promise | 07, 12, 13 | Allowlist tests, schema-columns test, XFF/UA non-storage test |
| `wasm-bindgen` CLI/crate mismatch | Cryptic runtime breakage | 02, 19, 26 | Version guard script; CI installs same version |
| sqlx offline cache wrong location | CI/Docker builds fail | 11, 27 | `SQLX_OFFLINE=true cargo check` |
| Cold-start retry logic | Waitlist undercount | 16, 21, 29 | `http.test.ts` 12 cases; real sleep/wake timings |
| Client IP / hop count | Rate limiting wrong or bypassable | 09, 29 | Unit tests; TE-5 measurement |
| Rate limiter memory/locking | Free-tier RAM | 10 | Eviction test; TE-11 memory |
| Windows shell (WSL vs Git Bash, CRLF) | Silent script failures | 01, all | `uname -s` check; `sh` invocations; LF config |
| Codegen non-determinism | Drift-check flakiness | 08 | Run twice, no diff |
| Server-before-web deploy order | `deny_unknown_fields` rejects new client | 28 | CI polls SHA before Vercel deploy |
| Panic handling | `panic=abort` kills CatchPanic | 02, 12 | No abort in profile; layer present |
| Secrets exposure | Signing keys | 09, 27 | Redacted `Debug`; keys only in Render |

# Architectural Invariants

1. Layer imports follow the TS §2 matrix: `ui` → `usecases/state/copy/config` only; `usecases` never call fetch/WebCodecs/WebGPU/WASM; `state` imports only `gen` types and has no side effects; `analytics` imports no store/persistence.
2. `fetch` exists only in `net/http.ts`, `net/asset-fetch.ts`, `wasm/load-core.ts`; `import.meta.env` only in `config/env.ts`; the environment is read only in `server/src/config.rs`.
3. Pure crates read no clock and generate no randomness (`uuid` without `v4/v7`; clippy bans `std::time` clocks); the server uses `OffsetDateTime::now_utc()` and `tokio::time::Instant`.
4. The server never depends on media crates; no media bytes ever reach it (media guard → 415).
5. Analytics props are enums, bools or capped integers; no free text; no IP/UA/user id/client timestamp stored.
6. No user-facing sentence outside `copy/messages.ts`; no typed-in numbers that exist in `LIMITS`.
7. Shared types live in `offcut-types`; branded casts only in `gen/` and `workers/`.
8. Every `switch` over a generated union ends in `assertNever`; every Rust `match` over enums has no wildcard.
9. The API emits no CORS headers; CSP is `style-src 'self'` with no inline script or style.
10. V1 files are not restructured later; later versions add bodies, files, rows.

# Forbidden Implementation Shortcuts

- No `bash` in scripts or `package.json`; no `&` backgrounding in pnpm scripts.
- No hand-editing `web/src/gen/*` or `.sqlx` files; no `.gitattributes`/other files absent from the TS §5 tree without updating the tree in the same commit.
- No native form post for the waitlist; no reading/inspecting the dropped `File`.
- No logging of emails, tokens, bodies, IPs or query strings; no raw path in logs (route template only).
- No `unwrap/expect/panic/indexing` in non-test Rust; no `any`/`!`/`@ts-ignore`; no `eslint-disable` or `#[allow]` to pass a gate (except the sanctioned test-file allow line and the one permitted `catch` in `analytics/client.ts`).
- No `panic = "abort"`; no Rust compile on Render; no Vercel Git builds.
- No paid services; no web fonts or CDNs; no external hosts beyond the asset host.
- No weakening or deleting a test to go green; no regenerating `.stderr` without reading it.
- No test file beyond V1's six (§13) unless §4, §13 and the TS §5 tree are updated in the same commit.
- No second copy of a value that lives in `limits.rs` (for example the 90-day retention).
- No secrets in files, commits or chat; production signing keys exist only in Render.
- No leaving the TE-5 sleep route or XFF logging in the codebase.

# Requirement → Prompt Traceability

| Requirement | Prompt(s) | Implementation | Validation |
| --- | --- | --- | --- |
| Shared types, unit safety | 03–05 | `offcut-types` | Unit tests, trybuild |
| API DTOs | 06 | `offcut-api-types` | Round-trips, deny-unknown tests |
| Analytics allowlist (19 events) | 07, 12, 13, 17 | Enum schema + ingest + `track` | Allowlist tests, E2E |
| Codegen + drift | 08, 26 | Generator tests, scripts | Run twice; CI drift step |
| Full DB schema (D-7) | 11 | `0001_init.sql` | `\dt`; integration tests |
| `/healthz`, `/events`, `/notify-me` | 12, 13 | Router + handlers | curl; integration tests |
| Rate limiting (all groups) | 10, 12, 13 | Token bucket | Unit + 429 test |
| Redacted logs, config guard | 09 | `log.rs`, `config.rs` | Config Debug test |
| 90-day purge | 12, 13 | `retention.rs` | 91/89-day test |
| Lint boundaries | 14 | `eslint.config.js` | 4 deliberate violations |
| Capability check (11 checks) | 18, 21 | `capability.ts` | §13.4 tests; E-8 beacon |
| Cold-start HTTP (D-10) | 16 | `http.ts` | 12-case test |
| IndexedDB v1 (D-6) | 15 | `schema.ts`/`db.ts` | Typecheck; store creation |
| Worker protocol typed (D-8) | 15 | `protocol.ts` | `tsc` |
| WASM build + compile under CSP (D-9) | 19, 20, 28 | `build-wasm.sh`, loader, pool | Browser proof; `@smoke` |
| Waitlist flow | 12, 21–23, 25 | Server + `submitNotifyMe` + form | Local chain; E2E |
| Landing, settings, unsupported, `/app` | 22, 23, 25 | Pages/components | E2E |
| Copy rules + COPY_PENDING (D-13) | 21, 24 | `messages.ts`, script | Script failure drill |
| File-tree / host / 400-line gates | 24 | Check scripts | Failure drills |
| CI (steps 1–9) | 26 | `ci.yml` | Green PR |
| Docker + Render | 27 | Dockerfile, `render.yaml`, deploy workflow | `/healthz` SHA |
| Vercel prebuilt + headers | 14, 28 | `vercel.json`, CI 10–14 | Header check, `@smoke` |
| TE-5/6/7/9/11, E-1, E-8, uptime | Human before 01 (start), before 27 (choose), 29 (measure and record) | `experiments.md`, monitor | Recorded outcomes |
| Hand-over §15.3, exit §16, tag | 30 | Audit | Checklist with evidence |

# Final Coverage Audit

- [x] Every V1 scope row of §1.1 (repository, shared types, server, web, WASM, CI/deploy, experiments) maps to a prompt.
- [x] Every file marked in the §4 tree appears in a prompt's scope (plus `server/src/lib.rs`, flagged for tree update).
- [x] Every V1 decision D-1…D-17 is implemented: D-1/2/3 (05–07), D-4/5 (08), D-6 (15), D-7 (11), D-8 (15), D-9 (20), D-10 (16), D-11 (12), D-12 (13), D-13 (21, 24), D-14 (12), D-15 (14), D-16 (23), D-17 (09).
- [x] Tests are incremental (inline Rust units in 03–12, integration 13, web unit 16 and 18, E2E 25), not deferred to the end. They are exactly the six test files, one compile-fail case and the inline unit tests of §13; Prompts 15, 17 and 21 add no test file.
- [x] Configuration, error architecture, security headers/CSP, logging redaction and health checks are covered; observability is limited to what V1 specifies (JSON logs, request id, healthz).
- [x] Performance: capability check ≤3 s budget (18); cold-start ≤70 s (16); memory/cold-start measured (29).
- [x] Deployment and documentation covered (27–30); README/API docs beyond the specs are **not** in V1 scope.
- [x] Dependencies are valid: every prompt needs only earlier prompts plus the specs.
- [x] Exactly 30 prompts; Prompt 30 yields a tagged, verified, deployed V1.

**Issues in the source documents to fix when you apply this (already reflected above):** (1) add `server/src/lib.rs` to the TS §5 tree and §4; (2) §10.8 says `cargo sqlx prepare --workspace`, which writes `.sqlx` at the repo root — run it from `server/` instead; (3) `vercel.json` is needed in Prompt 14, not at deploy time; (4) pin `wasm-bindgen` with `=` and match the CLI; (5) `wasm-opt` flags may need extra `--enable-*` features.

# Revision notes

Changes made after checking this file against `technicalspec.md` and `buildplan.md`:

| # | Where | Change | Why |
| --- | --- | --- | --- |
| 1 | Prompt 03 | "17 unit structs" corrected to 16, with the names listed | TS §10.1 defines 16 plus `Span` |
| 2 | Prompts 15, 21 | New test files removed | They are not in the TS §5 tree, so `check-file-tree.mjs` (Prompt 24) would fail; §13 fixes V1 at six test files |
| 3 | Prompt 17 | `client.test.ts` and the type-level test removed; replaced by a `tsc` failure drill | Both files belong to M2.2 (V8) |
| 4 | Prompt 12 | Retention uses `ANALYTICS_RETENTION_DAYS` from `limits.rs` | The constant already exists (TS §10.2); a second one would break the "limits live in one file" rule |
| 5 | Prompts 01, 27, 28, 29 | Accounts and TE-6/TE-7/TE-9 start before Prompt 01; database, asset host and Vercel project are fixed before Prompt 27; Prompt 29 only measures and records | Prompt 27 needs `DATABASE_URL` and `APP_ORIGIN`, and Prompt 28 needs the asset origin, before Prompt 29 ran |
| 6 | Prompt 27 | First-run order for `deploy-api.yml` and the Render service spelled out; demo-clip upload moved here | The deploy hook cannot exist before the service, and the service cannot start before the image exists |
| 7 | SAB, "Who does what" | Human steps separated from agent steps | Dashboards, secrets, DevTools checks and week-long measurements cannot be ticked by an agent |
| 8 | Header | Deviation from TS §34.1 and `buildplan.md` §1.3 recorded | These prompts group files and expose the repo; the specs prescribe one spec per file |
| 9 | Dependency table | Prompt 19 depends on 05, not 02; server prompts need 07, not 08 | `offcut-wasm-core` depends on `offcut-types`; the server does not use the generated TypeScript |
| 10 | Prompts 11, 23 | Test wording and the "Done when" wording made exact | "Repo-level tests or deferred" and "production-DB-style rows" were ambiguous |