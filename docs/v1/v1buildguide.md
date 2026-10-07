# v1buildguide.md - Offcut V1: step-by-step build guide

**What this is.** The order of work for building V1 on this machine, from an empty folder to the `v1` tag. `v1implementation.md` says *what* each file contains; this guide says *what to do next, which command to run, and how to know it worked*.

**How to read the references.** `§n` means a section of `v1implementation.md`. `TS §n` means `technicalspec.md`. `D-n` is a decision in §2. `S1`-`S12` are the build-order steps of §5.

**How the guide is organised.** Nine phases. Each phase is a list of steps, and each phase ends with a **milestone**: a short block of commands with the output you must see. Do not start a phase while the previous milestone fails. Commit at every milestone.

| Phase | Covers | Plan day | Milestone in one line |
|---|---|---|---|
| 0 Environment | §3 | before day 1 | Every tool prints a version; local Postgres answers |
| 1 Repository and workspaces | S1 | day 1 | `cargo metadata` and `pnpm install` succeed |
| 2 Shared types and codegen | S2, S3, S4 | day 1 | Both type crates pass; generating twice gives no diff |
| 3 Server | S5, S6 | day 2 | `cargo test -p offcut-api` green; three routes answer `curl` |
| 4 Web foundation and capability check | S7, S8 | day 3 | `tsc`, ESLint and Vitest green |
| 5 WASM path | S9 | day 3-4 | The browser compiles `offcut_core.wasm` under the production CSP |
| 6 UI and use-cases | S10 | day 4 | A real email and two analytics events land in local Postgres |
| 7 Checks, E2E, CI | S11 | day 4-5 | `pnpm check && pnpm test && pnpm build && pnpm e2e`; CI green on a PR |
| 8 Deploy and measure | S12 | day 5 | The §16 exit checklist; tag `v1` |

---

## Read this first: four things that will bite you

1. **Use Git Bash for every command in this guide.** On this machine `bash` on the Windows PATH is WSL (`C:\Windows\system32\bash.exe`), not Git Bash. A `scripts/*.sh` file started from PowerShell would run inside WSL, where Cargo is not installed. In VS Code set the default terminal profile to "Git Bash". In `package.json` scripts call `sh`, never `bash`.
2. **The server needs a library target, and the specs do not list one.** `server/tests/*.rs` must call `router::build`, and an integration test cannot import from a binary-only crate. The TS §5 tree lists `server/src/main.rs` and no `lib.rs`. Recommended fix: add `server/src/lib.rs` (module declarations only), keep `main.rs` as the thin entry point, and add the file to the TS §5 tree and to §4. Step 3.2 assumes this. Decide before Phase 3; `check-file-tree.mjs` will fail on an unlisted file.
3. **`web/vercel.json` is needed in Phase 4, not Phase 8.** §5 lists it under S12, but `vite preview` reads its headers (D-15) and `check-hosts.mjs` compares its CSP to `VITE_ASSET_BASE_URL`. Create it in step 4.2 with placeholders and fill in the real hosts in Phase 8.
4. **The `wasm-bindgen` CLI and crate must be the same version.** The CLI installed here is 0.2.108; the newest crate is 0.2.129. Step 0.3 updates the CLI and step 5.1 pins the crate with `=`.

---

## Phase 0 - Environment

Goal: every tool and account exists before any product code is written.

### Step 0.1 - Check what is already here

Found on this machine on 2026-10-07:

| Tool | State | Action |
|---|---|---|
| Git 2.48.1, Git Bash | installed | none |
| Rust 1.97.1 with `wasm32-unknown-unknown` | installed, stable is now 1.99.0 | update (0.2) |
| Node 24.19.0, pnpm 10.15.0 | installed | none |
| `wasm-bindgen` CLI 0.2.108 | installed, old | reinstall (0.3) |
| `wasm-opt` (binaryen) | missing | install (0.3) |
| `cargo-deny`, `sqlx-cli` | missing | install (0.3) |
| Docker 28.3.2 | installed, engine not running | start Docker Desktop before Phase 8 |
| PostgreSQL 18 service (`postgresql-x64-18`) | running | create a dev role (0.4) |
| Google Chrome, Vercel CLI 43.2.0, OpenSSL | installed | none |
| GitHub CLI `gh` | missing | optional (0.3) |

### Step 0.2 - Update Rust

```sh
rustup update stable
rustc --version                      # note the exact version; you pin it in step 1.3
rustup target add wasm32-unknown-unknown
```

### Step 0.3 - Install the missing tools

```sh
cargo install cargo-deny --locked
cargo install sqlx-cli --no-default-features --features rustls,postgres --locked
cargo install wasm-bindgen-cli --version 0.2.129 --locked --force
winget install GitHub.cli            # optional; makes setting CI secrets one command
```

`wasm-opt`: download the Windows archive from the binaryen releases page on GitHub (`WebAssembly/binaryen`), unpack it, and add its `bin` folder to your user PATH. Note the release number: CI must install the same one (step 7.4). Open a new terminal afterwards.

### Step 0.4 - Local Postgres

The integration tests create a fresh database per test (§13.1), so the role needs `CREATEDB`.

```sh
psql -U postgres -h localhost -c "CREATE ROLE offcut LOGIN PASSWORD 'offcut' CREATEDB;"
psql -U postgres -h localhost -c "CREATE DATABASE offcut_dev OWNER offcut;"
```

Local connection string, used from Phase 3 on: `postgres://offcut:offcut@localhost:5432/offcut_dev`.

### Step 0.5 - Open the accounts

Collect what each account must hand you (§3.2). Nothing here costs money.

| Account | Do now | Keep for later |
|---|---|---|
| GitHub | Create an empty repo (public gives the most free Actions minutes) | repo URL |
| Vercel (free) | Sign up; `vercel login` | token, org id, project id (step 8.3) |
| Render (free) | Sign up | service host, deploy hook URL (step 8.6) |
| Neon and Supabase | Create one database at each: this starts TE-6 | both TLS connection strings |
| Cloudflare R2 and Hugging Face Hub | Sign up; note which one demands a payment card: this starts TE-7 | public base URL |
| UptimeRobot | Sign up | used in step 8.9 |
| Merchant of record (Paddle, Lemon Squeezy or Dodo Payments) | **Submit the onboarding application today** (TE-9). Approval can take two weeks and V6 needs it | nothing yet |

### Step 0.6 - Generate signing keys

An Ed25519 seed is 32 random bytes; the variable holds its base64 form.

```sh
openssl rand -base64 32      # dev ENTITLEMENT_SIGNING_KEY
openssl rand -base64 32      # dev ACCESS_TOKEN_SIGNING_KEY
```

These two are for local development. In step 8.6 generate two more for production and paste them only into Render (TS §24.2); never into a file, a commit or a chat.

### Step 0.7 - Non-code preparation

- Make the three demo clips by hand (captions, a number landing, a list, a from-to). Phase 8 uploads them.
- Draft the fixture reading scripts for `fixtures/speech/README.md` (about 150 words, two numbers, one three-item list, one from-to, one emphasized word).

### Milestone 0

```sh
uname -s                                  # MINGW64_NT-...   (proves this is Git Bash, not WSL)
rustc --version && cargo --version
rustup target list --installed            # includes wasm32-unknown-unknown
wasm-bindgen --version                    # 0.2.129
wasm-opt --version
cargo deny --version && sqlx --version
node --version && pnpm --version
psql "postgres://offcut:offcut@localhost:5432/offcut_dev" -c "select 1"     # one row
```

Pass: every line prints a version, and the last prints a row. TE-9 is submitted.

---

## Phase 1 - Repository and workspaces (S1)

Goal: an empty but valid Cargo workspace and pnpm workspace under Git.

### Step 1.1 - Rename the docs folder and start Git

```sh
cd /c/Users/Mayan/Desktop/sh2clips
mv documents docs
git init -b main
git config core.autocrlf input           # keep LF: the shell scripts and generated files need it
```

A `.gitattributes` file would do the same job, but it is not in the TS §5 tree and `check-file-tree.mjs` would reject it. In VS Code, keep the line-ending indicator on `LF`.

### Step 1.2 - `.gitignore`

Write it from §12.8: `target/`, `node_modules/`, `web/dist/`, `web/src/wasm/pkg/`, `.vercel/`, `.env*`, `web/test-results/`, `web/playwright-report/`. Do **not** ignore `web/src/gen/` or `server/.sqlx/`.

### Step 1.3 - Create the four crate folders, then the workspace

Create the crates before the root manifest, so `cargo new` does not trip over a workspace whose members do not exist yet.

```sh
cargo new --lib --vcs none crates/offcut-types
cargo new --lib --vcs none crates/offcut-api-types
cargo new --lib --vcs none crates/offcut-wasm-core
cargo new --bin --vcs none --name offcut-api server
```

Then write the root files:

**`rust-toolchain.toml`**

```toml
[toolchain]
channel = "1.99.0"                # the version step 0.2 printed
targets = ["wasm32-unknown-unknown"]
components = ["rustfmt", "clippy"]
```

**`Cargo.toml`** (§12.1)

```toml
[workspace]
resolver = "2"
members = ["crates/offcut-types", "crates/offcut-api-types", "crates/offcut-wasm-core", "server"]

[workspace.dependencies]
# one line per shared dependency; see the lookup command below

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
indexing_slicing = "deny"
wildcard_enum_match_arm = "deny"

[profile.release]
lto = true
codegen-units = 1
# never panic = "abort": the server's CatchPanicLayer needs unwinding
```

Print the current version of every dependency in a form you can paste under `[workspace.dependencies]`:

```sh
for c in serde serde_json ts-rs uuid trybuild thiserror axum tokio tower tower-http sqlx \
         tracing tracing-subscriber time sha2 base64 ed25519-dalek reqwest wasm-bindgen; do
  cargo search "$c" --limit 1 | head -1
done
```

Write `wasm-bindgen = "=0.2.129"` (exact). Features are set per crate in later steps. Add `[lints] workspace = true` to each of the four crate manifests.

**`clippy.toml`**

```toml
disallowed-methods = [
  { path = "std::time::Instant::now",    reason = "pure crates must not read a clock (TS §7)" },
  { path = "std::time::SystemTime::now", reason = "pure crates must not read a clock (TS §7)" },
]
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true
```

**`deny.toml`**: run `cargo deny init`, then edit it to §12.1: advisories deny, a permissive-license allowlist, and the `[bans]` rules (the wrapper rule for `web-sys`, `js-sys`, `wasm-bindgen`, `wgpu`, `rand`, `getrandom`; the media-crate ban for `offcut-api`). The license list is settled on the first real `cargo deny check` in Phase 3, once dependencies exist.

### Step 1.4 - pnpm workspace

**`pnpm-workspace.yaml`**

```yaml
packages:
  - web
```

**`package.json`** (root; scripts are added as their targets appear)

```json
{
  "name": "offcut",
  "private": true,
  "packageManager": "pnpm@10.15.0",
  "engines": { "node": ">=24" },
  "scripts": {}
}
```

**`web/package.json`** (stub; Phase 4 fills it)

```json
{ "name": "web", "private": true, "version": "0.0.0", "type": "module" }
```

### Step 1.5 - Small fixed files

- `docs/file-specs/TEMPLATE.md`: copy the template of TS §34.2.
- `fixtures/speech/README.md`: the reading scripts from step 0.7.
- `bench/results/.gitkeep`: empty.

### Step 1.6 - First commit and push

```sh
git add -A && git commit -m "V1 S1: repository and workspaces"
git remote add origin https://github.com/<owner>/<repo>.git
git push -u origin main
```

### Milestone 1

```sh
cargo metadata --format-version 1 > /dev/null && echo "cargo ok"
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings
pnpm install
git status --short            # prints nothing
```

Pass: `cargo ok`, no clippy warning on the stubs, `pnpm install` writes `pnpm-lock.yaml`, working tree clean after committing the lock files.

---

## Phase 2 - Shared types and code generation (S2, S3, S4)

Goal: every shared type exists in Rust and is generated to TypeScript, deterministically. These files are **frozen** at the end of V1 (§15.1), so copy from the spec carefully.

### Step 2.1 - `offcut-types` manifest

Dependencies: `serde` (feature `derive`), `ts-rs`, `uuid` (feature `serde` only: no `v4`, no `v7`). Dev-dependency: `trybuild`. Nothing else (§6 crate rules).

### Step 2.2 - Write the twelve modules, in dependency order

| Order | File | Source | Watch for |
|---|---|---|---|
| 1 | `units.rs` | §6.1, TS §10.1 | Private inner fields; `const fn new`/`get`; no `From<u32>`; `Confidence` deserializes through `new` |
| 2 | `ids.rs` | §6.2 | No function generates an id |
| 3 | `limits.rs` | §6.3, TS §10.2 | Exactly 21 constants, nothing else |
| 4 | `stage.rs` | §6.4 | Six variants in PS §20.2 order |
| 5 | `media.rs` | §6.4, TS §10.3 | 14 `RejectReason` variants, each renamed `REJECT_...` |
| 6 | `transcript.rs` | TS §10.4 | `Unit::Count { noun }` is the only variant with data |
| 7 | `events.rs`, `prosody.rs` | TS §10.5 | `EventParams` is tagged `kind`, `snake_case` |
| 8 | `edit.rs`, `profile.rs`, `summary.rs` | TS §10.6 | `CropOffset::new` range; `EditState::default()` |
| 9 | `error.rs` | §6.5 | 29 `E_*`, 11 `UNSUPPORTED_*`, 11 `FailureStage`; `From<PipelineStage>` |
| 10 | `lib.rs` | §6.6 | Declares and re-exports all twelve |

Apply the derive rules of §6 and the serialization rules of D-2 to every type. Run `cargo check -p offcut-types` after each file, not at the end.

### Step 2.3 - Tests for `offcut-types`

1. Unit tests in `units.rs`: the six cases of §6.7.
2. Round-trip tests in `lib.rs` for the three code enums: the string form is the contract.
3. Compile-fail test: write `tests/ui.rs` and `tests/ui/bare_ms_rejected.rs`, then generate the expected error once:

```sh
TRYBUILD=overwrite cargo test -p offcut-types --test ui
```

Open the generated `.stderr` and confirm the errors are type mismatches (`expected DurMs, found u32` and `found TimeMs`), not a missing import. Commit it. It is tied to the pinned compiler: regenerate it whenever `rust-toolchain.toml` changes.

### Step 2.4 - `offcut-api-types`

Dependencies: `serde`, `ts-rs`, `offcut-types`.

| Order | File | Source |
|---|---|---|
| 1 | `auth.rs`, `account.rs`, `billing.rs`, `usage.rs` | §7.1, TS §10.7; every request struct has `deny_unknown_fields` |
| 2 | `errors.rs` | §7.6: 13 `ApiErrorCode` members |
| 3 | `analytics.rs` | §7.2-§7.4: 3 constants, 16 prop enums, 19 events, `EventsBatch`, `ANALYTICS_EVENT_DOCS`, `name()` |
| 4 | tests in `analytics.rs` | §7.5: all three groups |
| 5 | `lib.rs` | §7.7 |

The allowlist rule to check by eye: no event variant has a `String` field.

### Step 2.5 - The two generator tests

Add an `#[ignore]`d test named `write_typescript` to each crate's `lib.rs` (§9, D-4). It builds one string and writes it to the path in `OFFCUT_GEN_OUT`.

Two details that make the output stable on Windows and in CI:

- `cargo test` runs with the crate folder as its working directory. Resolve a relative `OFFCUT_GEN_OUT` against the workspace root (`env!("CARGO_MANIFEST_DIR")` plus `../..`), so the script can pass the paths exactly as §9 writes them.
- Write `\n` line endings, a fixed order and no timestamp.

### Step 2.6 - `scripts/gen-types.sh` and `scripts/check-gen-clean.sh`

```sh
#!/usr/bin/env sh
# scripts/gen-types.sh
set -eu
cd "$(dirname "$0")/.."
OFFCUT_GEN_OUT=web/src/gen/domain.ts cargo test -p offcut-types     write_typescript -- --ignored
OFFCUT_GEN_OUT=web/src/gen/api.ts    cargo test -p offcut-api-types write_typescript -- --ignored
```

```sh
#!/usr/bin/env sh
# scripts/check-gen-clean.sh
set -eu
cd "$(dirname "$0")/.."
sh scripts/gen-types.sh
git diff --exit-code -- web/src/gen
```

`git diff` ignores untracked files, so commit `web/src/gen/` once before relying on the check. Add the root scripts `"gen:types": "sh scripts/gen-types.sh"`.

### Milestone 2

```sh
cargo test -p offcut-types            # unit tests + tests/ui.rs
cargo test -p offcut-api-types        # round-trips, docs-match-variants, rejections
cargo clippy --workspace --all-targets -- -D warnings
sh scripts/gen-types.sh && git add web/src/gen && sh scripts/check-gen-clean.sh && echo "gen clean"
```

Then read the two generated files once and count:

| In | Expect |
|---|---|
| `domain.ts` | `export type TimeMs = number & { readonly __unit: "TimeMs" };`; `ClipId` and `EventId` branded strings; `LIMITS` with 21 properties; `ERROR_CODES` 29, `REJECT_REASONS` 14, `UNSUPPORTED_REASONS` 11 |
| `api.ts` | `import type` lines from `./domain`; `ANALYTICS_EVENT_DOCS` with 19 entries; `MAX_EVENTS_PER_BATCH = 50` |

Pass: all tests green, `gen clean` printed, counts match. Commit: "V1 S2-S4: shared types and codegen".

---

## Phase 3 - Server (S5, S6)

Goal: the Axum service runs locally against Postgres with three live routes and the full schema.

### Step 3.1 - Local environment file

Create `.env` at the repo root (git-ignored). The `sqlx` macros read it at compile time; the server itself reads only real environment variables, so you load the file into the shell.

```sh
DATABASE_URL=postgres://offcut:offcut@localhost:5432/offcut_dev
APP_ORIGIN=http://localhost:5173
PORT=8080
LOG_LEVEL=info
ENTITLEMENT_SIGNING_KEY=<dev key 1 from step 0.6>
ACCESS_TOKEN_SIGNING_KEY=<dev key 2 from step 0.6>
MAIL_API_KEY=unset-until-v6
MAIL_FROM=unset-until-v6
BILLING_API_KEY=unset-until-v6
BILLING_WEBHOOK_SECRET=unset-until-v6
BILLING_PRICE_CREATOR_MONTHLY=unset-until-v6
BILLING_PRICE_CREATOR_ANNUAL=unset-until-v6
BILLING_PRICE_CREATOR_ANNUAL_FOUNDING=unset-until-v6
FOUNDING_OFFER_ENABLED=false
TRUSTED_PROXY_HOPS=0
```

Load it in every new terminal before running the server or its tests:

```sh
set -a; . ./.env; set +a
```

### Step 3.2 - Manifest and crate layout

`server/Cargo.toml`: the dependency list of §10, with the `sqlx` features `postgres`, `runtime-tokio`, `tls-rustls`, `uuid`, `time`, `json`, `migrate`, `macros`, and `uuid` with `v4`, `v7`, `serde`. Dev-dependency: `reqwest`.

Resolve the library-target gap from "Read this first" now: `src/lib.rs` declares the modules (`pub mod config; pub mod router;` ...), and `src/main.rs` calls into `offcut_api::...`. Update the TS §5 tree and §4 in the same commit.

### Step 3.3 - The migration

Copy TS §23.2 exactly into `server/migrations/0001_init.sql` (D-7), then apply and inspect it:

```sh
sqlx migrate run --source server/migrations
psql "$DATABASE_URL" -c '\dt'        # 8 tables + _sqlx_migrations
```

### Step 3.4 - Core files (S5)

Write in this order; each depends only on the ones above it.

| Order | File | Section | Tests to write with it |
|---|---|---|---|
| 1 | `config.rs` | §10.1 | Complete env parses; each missing variable names itself; bad key is `Malformed`; `Debug` prints no secret |
| 2 | `log.rs` | §10.2 | none in V1 |
| 3 | `error.rs` | §10.3 | none (covered by integration tests) |
| 4 | `headers.rs` | §10.4 | none |
| 5 | `client_ip.rs` | §10.5 | The seven cases listed there |
| 6 | `rate_limit.rs` | §10.6 | The six cases listed there, with `tokio::time::pause` |
| 7 | `state.rs`, `db/mod.rs` | §10.7, §10.8 | none |
| 8 | `router.rs` | §10.11 | `/healthz`, `not_found`, all seven layers in order, the twelve V6 routes as comments |
| 9 | `main.rs` | §10.12 | none |

**S5 check:**

```sh
cargo run -p offcut-api                               # terminal 1
curl -i http://localhost:8080/api/v1/healthz          # terminal 2
```

Expect `200`, `{"ok":true,"version":"dev"}`, `cache-control: no-store`, `x-content-type-options: nosniff`, an `x-request-id`, and no `access-control-*` header. Then prove the config guard:

```sh
env -u DATABASE_URL cargo run -p offcut-api           # prints "config error: DATABASE_URL", exit code 1
```

### Step 3.5 - Routes (S6)

| Order | File | Section |
|---|---|---|
| 1 | `auth/mod.rs`, `auth/magic_link.rs` | §10.10: `normalize_email` only, with its six unit tests |
| 2 | `db/platform_waitlist.rs` | §10.8 |
| 3 | `account/notify.rs`, `account/mod.rs` | §10.10 |
| 4 | `db/analytics_events.rs` | §10.8 |
| 5 | `analytics/mod.rs` | §10.9: the five-step `ingest` |
| 6 | `analytics/retention.rs` | §10.9 |
| 7 | wire both `routes()` into `router.rs`; `spawn_purge_task` into `main.rs` | §10.11, §10.12 |

### Step 3.6 - The offline query cache

The spec wants the cache in `server/.sqlx/`. Run the command from `server/` **without** `--workspace` (§10.8 writes `--workspace`, which puts the folder at the repo root instead):

```sh
cd server && cargo sqlx prepare -- --all-targets && cd ..
ls server/.sqlx | head -3            # files exist here, and there is no .sqlx at the repo root
SQLX_OFFLINE=true cargo check -p offcut-api
```

Re-run it whenever a `sqlx::query!` changes. Commit `server/.sqlx/`.

### Step 3.7 - Integration tests

1. `tests/common/mod.rs`: `TestApp::spawn()` (§13.1).
2. `tests/analytics_allowlist.rs`: the 14 cases of §13.2.
3. `tests/notify_me.rs`: the 10 cases of §13.3.

Each file starts with the `#![allow(clippy::unwrap_used, ...)]` line of §12.1.

### Milestone 3

```sh
set -a; . ./.env; set +a
cargo test -p offcut-api
cargo clippy --workspace --all-targets -- -D warnings && cargo deny check
```

With the server running in another terminal:

```sh
curl -i -X POST http://localhost:8080/api/v1/notify-me -H 'content-type: application/json' \
  -d '{"email":"  Someone@Example.COM ","wanted":"launch"}'                              # 204
curl -i -X POST http://localhost:8080/api/v1/events -H 'content-type: application/json' \
  -d '{"anon_id":"11111111-1111-4111-8111-111111111111","events":[{"name":"landing_view","props":{"hero_variant":"outcome"}}]}'   # 204
curl -i -X POST http://localhost:8080/api/v1/events -H 'content-type: application/json' \
  -d '{"anon_id":"11111111-1111-4111-8111-111111111111","events":[{"name":"landing_view","props":{"hero_variant":"hello"}}]}'     # 400
curl -i -X POST http://localhost:8080/api/v1/events -H 'content-type: video/mp4' -d 'x'  # 415
curl -i http://localhost:8080/api/v1/nope                                                # 404 {"code":"not_found"...}

psql "$DATABASE_URL" -c "select email_normalized, wanted from platform_waitlist"         # someone@example.com | launch
psql "$DATABASE_URL" -c "select name, props from analytics_events"                       # one landing_view row
```

Pass: all server tests green, the five status codes match, and the two tables hold exactly those rows. Commit: "V1 S5-S6: server".

---

## Phase 4 - Web foundation and capability check (S7, S8)

Goal: the non-visual web layers exist, type-check, pass every lint boundary, and the capability check is tested.

### Step 4.1 - Dependencies

```sh
cd web
pnpm add react react-dom react-router zustand idb
pnpm add -D vite @vitejs/plugin-react typescript vitest jsdom @playwright/test \
  eslint typescript-eslint eslint-plugin-boundaries eslint-plugin-react \
  @types/react @types/react-dom @webgpu/types @types/dom-webcodecs
cd ..
```

If `tsc` later reports duplicate WebCodecs declarations, your TypeScript already ships them: remove `@types/dom-webcodecs`.

Add scripts to `web/package.json`: `dev` (`vite`), `build:vite` (`vite build`), `preview` (`vite preview`). `build:vite` must exist here as well as at the root, because `vercel.json` runs it with `web` as the root directory.

Create `web/.env.local` (git-ignored) so `env.ts` has its variable:

```sh
VITE_ASSET_BASE_URL=https://assets.example.invalid
```

Use the real asset host if TE-7 has already chosen one.

### Step 4.2 - Configuration files

| File | Section | Note |
|---|---|---|
| `tsconfig.json` | §12.3 | `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes` |
| `vercel.json` | §12.4 | Create now. Put the same host as `VITE_ASSET_BASE_URL` where `<ASSET_ORIGIN>` appears; leave `<render-host>` as a placeholder host until step 8.7 |
| `vite.config.ts` | §12.3 | Dev: COOP + COEP only, proxy `/api/v1` to `http://localhost:8080`. Preview: headers read from `vercel.json` |
| `vitest.config.ts` | §12.3 | `node` by default |
| `eslint.config.js` | §12.3 | Every rule on now, including ones with nothing to check yet |
| `index.html`, `public/robots.txt` | §11.1, §12.3 | No inline script, no inline style |

### Step 4.3 - Foundation files (S7)

| Order | File | Section |
|---|---|---|
| 1 | `src/config/env.ts`, `config/allowlist-hosts.ts` | §11.2 |
| 2 | `src/workers/protocol.ts` | §11.3, TS §14.1-§14.2: types only, no function, no constant |
| 3 | `src/state/machines/transition.ts` | §11.4 |
| 4 | `src/state/capability-store.ts` | §11.4 |
| 5 | `src/persistence/schema.ts`, `persistence/db.ts` | §11.5: all eight stores at version 1 (D-6) |
| 6 | `src/net/http.ts` + `http.test.ts` | §11.6, §13.5: write the 12 test cases alongside the code |
| 7 | `src/net/api-client.ts`, `net/asset-fetch.ts` | §11.6 |
| 8 | `src/analytics/client.ts` | §11.7 |

A temporary `src/main.tsx` that renders an empty `<div>` is enough for now.

### Step 4.4 - Capability check (S8)

| Order | File | Section |
|---|---|---|
| 1 | `src/platform/simd-probe.ts` | §11.8 |
| 2 | `src/workers/render/encoders.ts` | §11.8, TS §21.2: constants and `videoConfigFor` only |
| 3 | `src/platform/capability.ts` | §11.8 |
| 4 | `src/platform/capability.test.ts` | §13.4: 11 groups, all with a fake `PlatformProbe` |

A known-good SIMD probe (checked with `WebAssembly.validate` in Node on this machine):

```ts
export const SIMD_PROBE_BYTES = new Uint8Array([
  0, 97, 115, 109, 1, 0, 0, 0, 1, 5, 1, 96, 0, 1, 123, 3, 2, 1, 0,
  10, 10, 1, 8, 0, 65, 0, 253, 15, 253, 98, 11,
]);
```

### Milestone 4

```sh
pnpm --filter web exec tsc --noEmit
pnpm --filter web exec eslint .
pnpm --filter web exec vitest run          # http.test.ts and capability.test.ts
```

Then prove the boundaries really bite. Each of these must make ESLint **fail**; undo each afterwards:

| Temporary edit | Rule that must fire |
|---|---|
| `fetch("/x")` inside `src/state/capability-store.ts` | banned global outside the four fetch files |
| `import.meta.env.MODE` inside `src/net/http.ts` | `import.meta.env` outside `config/env.ts` |
| `1000 as TimeMs` inside `src/net/api-client.ts` | brand cast outside `gen/` and `workers/` |
| `import "../net/http"` inside a file under `src/state/` | boundaries |

Pass: three commands green, four deliberate violations rejected. Commit: "V1 S7-S8: web foundation and capability check".

---

## Phase 5 - WASM path (S9)

Goal: Rust compiles to WASM, Vite ships it as a hashed asset, and the browser compiles it under the production CSP.

### Step 5.1 - The crate

`crates/offcut-wasm-core/Cargo.toml`: `crate-type = ["cdylib"]`; dependencies `wasm-bindgen` (workspace, pinned `=0.2.129`) and `offcut-types`. `src/lib.rs`: `init()` with the panic hook and `core_version()`, exactly §8.

### Step 5.2 - `scripts/build-wasm.sh`

Per §12.6, for each `BUNDLES` entry (V1 has one: `offcut-wasm-core:core`):

```sh
RUSTFLAGS="-C target-feature=+simd128" \
  cargo build --target wasm32-unknown-unknown --release -p offcut-wasm-core
wasm-bindgen --target web --out-name offcut_core --out-dir web/src/wasm/pkg/core \
  target/wasm32-unknown-unknown/release/offcut_wasm_core.wasm
wasm-opt -O3 --enable-simd --enable-bulk-memory --enable-nontrapping-float-to-int \
  web/src/wasm/pkg/core/offcut_core_bg.wasm -o web/src/wasm/pkg/core/offcut_core_bg.wasm
```

Also required by the contract:

- Before building, compare `wasm-bindgen --version` with the `wasm-bindgen` version in `Cargo.lock` and exit non-zero when they differ.
- `--dev` skips `wasm-opt`; `--watch` rebuilds on change.
- If `wasm-opt` stops with a validation error that names a feature, add the matching `--enable-<feature>` flag.

Root scripts: `"build:wasm": "sh scripts/build-wasm.sh"`.

### Step 5.3 - The loader

Write `src/wasm/load-core.ts` and `src/workers/pool.ts` from §11.9. `web/src/wasm/pkg/` is git-ignored, so from now on `tsc` only passes after `pnpm build:wasm` has run. CI already has that order (§12.7 steps 5 and 6).

### Step 5.4 - Try it in the browser

Temporarily call `preload()` from `src/main.tsx`, then:

```sh
pnpm build:wasm
pnpm --filter web exec vite                       # http://localhost:5173
pnpm --filter web exec vite build && pnpm --filter web exec vite preview     # http://localhost:4173
```

### Milestone 5

| Check | Where | Expect |
|---|---|---|
| Build output | `ls web/src/wasm/pkg/core` | `offcut_core.js`, `offcut_core_bg.wasm`, `.d.ts` files |
| Version guard | change the CLI version check to a wrong number once | the script exits non-zero |
| Dev page | DevTools, Network, filter `wasm` | status 200, type `application/wasm`, no console error |
| Preview page (production CSP) | DevTools console on `localhost:4173` | no `Content Security Policy` message; in the console `crossOriginIsolated` is `true` |
| Types | `pnpm --filter web exec tsc --noEmit` | green |

Pass: all five rows. This is the proof that CSP `'wasm-unsafe-eval'` and the MIME type are right before any real WASM work depends on them. Commit: "V1 S9: WASM path".

---

## Phase 6 - UI and use-cases (S10)

Goal: the three routes render and the whole chain works locally, browser to Postgres.

### Step 6.1 - Copy

`src/copy/messages.ts` (§11.11): eight sections. All eleven `UNSUPPORTED_*` reasons need copy now. No typed-in number: `{seconds}` comes from `LIMITS.MAX_CLIP_DURATION`. Avoid the banned phrases listed there.

### Step 6.2 - Use-cases

| File | Section |
|---|---|
| `src/usecases/submit-notify-me.ts` | §11.10: the result mapping |
| `src/usecases/start-app.ts` | §11.10: the seven steps in order, idempotent |

### Step 6.3 - Styles, components, pages

| Order | Files | Section |
|---|---|---|
| 1 | `ui/styles/tokens.css`, `pages.module.css`, `components.module.css` | §11.14: system fonts only |
| 2 | `WhatLeavesTable.tsx`, `NotifyMeForm.tsx`, `DropZone.tsx` | §11.13 |
| 3 | `UnsupportedPage.tsx`, then `CapabilityGate.tsx` | §11.12, §11.13 |
| 4 | `LandingPage.tsx`, `EditorPage.tsx`, `SettingsPage.tsx` | §11.12 |

Two rules that are easy to break: the waitlist form submits through script only (`form-action 'none'` blocks a native post), and `DropZone` never reads the dropped `File`.

### Step 6.4 - Entry files

`src/routes.tsx`, `src/App.tsx`, the final `src/main.tsx` (§11.1). Remove the temporary `preload()` call from step 5.4: `startApp()` now does it.

### Step 6.5 - The `dev` script

`pnpm` runs scripts through `cmd.exe` on Windows, so a bare `&` would not background anything. Either use two terminals, or:

```json
"dev": "sh -c \"sh scripts/build-wasm.sh --dev --watch & exec pnpm --filter web exec vite\""
```

### Milestone 6

Terminal 1: `set -a; . ./.env; set +a; cargo run -p offcut-api`. Terminal 2: `pnpm dev`. Open `http://localhost:5173` in Chrome.

| Do | Expect |
|---|---|
| Load `/` | Hero, supported-browser line, privacy line, "What leaves your device" link, drop zone, sample button, waitlist form |
| Wait about a second | The capability line turns to "supported" (or a specific reason) |
| Console: `crossOriginIsolated` | `true` |
| Submit `you@example.com` | Success message; Network shows `POST /api/v1/notify-me` 204 |
| Submit `abc` | Invalid-email message; no request |
| Wait 10 s, or switch tabs | `POST /api/v1/events` 204 |
| Drop a file on the drop zone | The not-ready message; no request |
| Open `/settings` | 4 fixed rows + 19 event rows |
| Open `/app` | The not-ready panel with the form |
| DevTools device toolbar, pick a phone, reload | The unsupported page with the mobile reason and `mobile` preselected |
| Stop the API, submit an email | The waking message after 3 s; start the API again and the submit succeeds |

```sh
psql "$DATABASE_URL" -c "select email_normalized, wanted, created_at from platform_waitlist order by created_at desc limit 3"
psql "$DATABASE_URL" -c "select name, props from analytics_events order by ts desc limit 5"
```

Pass: every row above, and the second query shows `landing_view` and `capability_check` rows whose props are all enum words. Commit: "V1 S10: UI and use-cases".

---

## Phase 7 - Checks, E2E, CI (S11)

Goal: one command per gate locally, and the same gates on every pull request.

### Step 7.1 - The check scripts

| Script | Section | Prove it by |
|---|---|---|
| `scripts/check-file-tree.mjs` | §12.6 | Add `web/src/stray.ts`: it must fail. Remove it |
| `scripts/check-copy-codes.mjs` | §12.6, D-13 | Delete one `UNSUPPORTED_*` message: it must fail. Restore it |
| `scripts/check-hosts.mjs` | §12.6 | Add `https://example.com` to a string in `messages.ts`, build: it must fail. Remove it |
| `scripts/check-external-facts.mjs` | §12.6 | Prints TE-5 to TE-11, one line each |

### Step 7.2 - Root scripts

```json
"scripts": {
  "dev": "sh -c \"sh scripts/build-wasm.sh --dev --watch & exec pnpm --filter web exec vite\"",
  "gen:types": "sh scripts/gen-types.sh",
  "build:wasm": "sh scripts/build-wasm.sh",
  "build:vite": "pnpm --filter web run build:vite",
  "build": "pnpm gen:types && pnpm build:wasm && pnpm build:vite && node scripts/check-hosts.mjs",
  "check": "cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo deny check && pnpm --filter web exec eslint . && pnpm --filter web exec tsc --noEmit && node scripts/check-file-tree.mjs && sh scripts/check-gen-clean.sh && node scripts/check-copy-codes.mjs",
  "test": "cargo test --workspace && pnpm --filter web exec vitest run",
  "e2e": "pnpm --filter web exec playwright test --project=non-media"
}
```

### Step 7.3 - Playwright

```sh
pnpm --filter web exec playwright install chrome
```

Write `web/playwright.config.ts` (§12.3), `tests-e2e/helpers/fake-api.ts`, `helpers/fixtures.ts` (§13.6) and `tests-e2e/landing.spec.ts` with the 12 cases of §13.7. The suite runs against `vite preview`, so run `pnpm build` first. No case touches a real server.

### Step 7.4 - `ci.yml`, steps 1 to 9

Follow §12.7 in order. Points that are not obvious from the list:

- A `postgres` service container, and `DATABASE_URL` pointing at it for step 4.
- `SQLX_OFFLINE: "true"` for every Cargo step, so no step needs a database at compile time.
- Install the same tool versions as locally: `cargo-deny`, `wasm-bindgen-cli` 0.2.129, and the binaryen release from step 0.3 (the Ubuntu package is much older).
- `VITE_ASSET_BASE_URL` as a repository variable, used by steps 6 to 9.
- `pnpm --filter web exec playwright install --with-deps chrome` before step 9.
- Leave the `main`-only steps 10 to 14 for step 8.8; they need secrets that do not exist yet.

### Step 7.5 - Open a pull request

```sh
git switch -c v1-ci && git add -A && git commit -m "V1 S11: checks, E2E, CI"
git push -u origin v1-ci      # then open the PR on GitHub
```

### Milestone 7

```sh
set -a; . ./.env; set +a
pnpm check && pnpm test && pnpm build && pnpm e2e
```

Pass: the four commands succeed locally in one run, the three deliberate failures of step 7.1 were each caught, and `ci.yml` is green on the pull request. Merge it.

---

## Phase 8 - Deploy and measure (S12)

Goal: production is live, measured and tagged. The steps are ordered so that every value exists before the step that needs it.

### Step 8.1 - Choose the database (TE-6)

Run the migration on each candidate, record plan terms, pool-of-5 behaviour and reconnect time in `docs/v1/experiments.md` (§14). Pick one. The week-long idle measurement finishes after V1; record the date it started.

### Step 8.2 - Choose the asset host (TE-7)

Upload a 150 MB file, configure CORS for the app origin, and test a ranged request (the full test from the deployed page happens in step 8.9). Write `scripts/upload-assets.sh` (§12.6), upload the three demo clips under `media/`, and put the printed paths where `LandingPage` and `UnsupportedPage` call `assetUrl(...)`.

### Step 8.3 - Create the Vercel project

```sh
vercel link               # from the repo root; create a new project
```

In the project settings: Root Directory `web`, Framework "Other". Do **not** connect the Git repository: Vercel's builders cannot build the WASM, so every deploy comes prebuilt from CI. Read `orgId` and `projectId` from `.vercel/project.json`, create a token in the account settings, and note the production URL `https://<project>.vercel.app`.

### Step 8.4 - The Docker image, locally

Start Docker Desktop. Write `server/Dockerfile` (§12.5), then:

```sh
docker build -f server/Dockerfile --build-arg GIT_SHA=$(git rev-parse HEAD) -t offcut-api:local .
docker run --rm -p 8080:8080 --env-file .env \
  -e DATABASE_URL="<the hosted dev connection string from 8.1>" offcut-api:local
curl -s http://localhost:8080/api/v1/healthz        # "version" is the commit SHA
```

### Step 8.5 - Push the image

Write `.github/workflows/deploy-api.yml` (§12.7) and run it once from `main`. Then open the `offcut-api` package on GitHub and set its visibility to **public**, so Render can pull it without credentials.

### Step 8.6 - Create the Render service

Write `render.yaml` (§12.5) and create the service from it. Enter every variable of §3.3 in the dashboard:

| Variable | Production value |
|---|---|
| `DATABASE_URL` | from step 8.1 |
| `APP_ORIGIN` | the Vercel URL from step 8.3, no trailing slash |
| `ENTITLEMENT_SIGNING_KEY`, `ACCESS_TOKEN_SIGNING_KEY` | two **new** keys (`openssl rand -base64 32`), pasted here and nowhere else |
| the seven mail and billing variables | `unset-until-v6` |
| `FOUNDING_OFFER_ENABLED` | `false` |
| `LOG_LEVEL` | `info` |
| `TRUSTED_PROXY_HOPS` | `1` for now; step 8.9 measures the real value |

```sh
curl -s https://<render-host>/api/v1/healthz        # {"ok":true,"version":"<sha>"}; the first call may take a minute
```

Copy the service host name and the deploy hook URL.

### Step 8.7 - Fill in the real hosts

1. `web/vercel.json`: replace `<render-host>` and both `<ASSET_ORIGIN>` entries.
2. GitHub secrets: `VERCEL_TOKEN`, `VERCEL_ORG_ID`, `VERCEL_PROJECT_ID`, `RENDER_DEPLOY_HOOK_URL` (`gh secret set NAME`).
3. GitHub variable `VITE_ASSET_BASE_URL`, and the same value in `web/.env.local`.
4. `pnpm build` locally: `check-hosts.mjs` must still pass with the real values.

### Step 8.8 - The `main`-only CI steps

Add steps 10 to 14 of §12.7 to `ci.yml`: call `deploy-api.yml`; poll `/healthz` until `version` equals the commit SHA; `vercel pull --yes --environment=production`, `vercel build --prod`, `vercel deploy --prebuilt --prod`; the header check; the `@smoke` Playwright case. Server first, then web, always. Merge to `main` and watch the run.

### Step 8.9 - Measure (TE-5, TE-7, TE-11) and switch on the monitor

| Experiment | Do | Record |
|---|---|---|
| TE-5 | In the deployed page's console, check `crossOriginIsolated`. Add the temporary 60 s sleep route and temporary `X-Forwarded-For` logging, call it through the rewrite and directly, **remove both the same day** | Isolation; whether 60 s survives; hop counts. Set `TRUSTED_PROXY_HOPS` in Render |
| TE-7 | From the deployed page, fetch `Range: bytes=0-8388607` from the asset host | 206, `Content-Range`, readable under COEP |
| TE-11 | Read memory in Render's metrics; let the service sleep 10 times and time each wake | Memory, cold-start times |
| Monitor | UptimeRobot: `GET https://<render-host>/api/v1/healthz` every 5 minutes | Start date, for the instance-hours reading a week later |

Write each outcome into `docs/v1/experiments.md`, update the dates in `scripts/check-external-facts.mjs`, and save Vercel's free-plan terms with the date read.

### Step 8.10 - Close out

Tick §15.3 and §16 line by line. Correct any §2 decision that changed during the build, here and in the specs. Then:

```sh
git tag v1 && git push origin v1
```

### Milestone 8

```sh
APP=https://<project>.vercel.app
curl -s $APP/api/v1/healthz                                   # version == git rev-parse origin/main
curl -sI $APP/ | grep -i -E "cross-origin|content-security|strict-transport|referrer|permissions"
E2E_BASE_URL=$APP pnpm --filter web exec playwright test --grep @smoke
```

Then, by hand on the deployed page: submit a real email, wait for a flush, and query production:

```sh
psql "<production DATABASE_URL>" -c "select email_normalized, wanted from platform_waitlist order by created_at desc limit 3"
psql "<production DATABASE_URL>" -c "select name, props from analytics_events order by ts desc limit 5"
```

Pass: `/healthz` answers through the Vercel rewrite with the current SHA; all seven headers match `vercel.json`; `@smoke` passes (isolated, WASM compiled, no CSP violation); your email and the `landing_view` and `capability_check` rows are in the production database; the monitor is pinging; the four experiment outcomes are written down; every box of §16 is ticked; the `v1` tag is pushed.

---

## Quick reference

**Every new terminal**

```sh
cd /c/Users/Mayan/Desktop/sh2clips && set -a; . ./.env; set +a
```

**After changing ...**

| You changed | Run |
|---|---|
| A type in `offcut-types` or `offcut-api-types` | `pnpm gen:types`, commit `web/src/gen/` |
| A `sqlx::query!` | `cd server && cargo sqlx prepare -- --all-targets`, commit `server/.sqlx/` |
| The Rust toolchain version | `TRYBUILD=overwrite cargo test -p offcut-types --test ui`, review, commit |
| `wasm-bindgen` crate version | reinstall `wasm-bindgen-cli` at the same version, locally and in CI |
| `vercel.json` hosts | `pnpm build` (runs `check-hosts.mjs`) |
| Added a source file | add it to the TS §5 tree in the same commit |

**When something fails**

| Symptom | Likely cause |
|---|---|
| `cargo: command not found` inside a script | The script ran in WSL bash. Use a Git Bash terminal |
| `$'\r': command not found` | CRLF line endings in a `.sh` file; step 1.1 |
| `set DATABASE_URL to use query macros` | `.env` missing at the repo root, or `SQLX_OFFLINE` unset with no database |
| `tsc` cannot find `./pkg/core/offcut_core` | `pnpm build:wasm` has not run since the last clean checkout |
| `SharedArrayBuffer is not defined`, `crossOriginIsolated` false | The page was not served by Vite with the COOP/COEP headers, or a cross-origin asset lacks CORS |
| Demo video does not load | The asset host sends no CORS header for the app origin, or `<video>` lacks `crossorigin="anonymous"` |
| Render logs `config error: <NAME>` | That variable is empty or malformed in the Render dashboard |
| Waitlist submit from the deployed page returns 404 | `<render-host>` in `vercel.json` is still a placeholder |
