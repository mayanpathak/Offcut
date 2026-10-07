# v1changelog.md - Offcut V1: record of changes

**What this is.** A record of every change made while building V1: what was done, what differs from the specs, how it was checked, and what is still open. `v1implementation.md` says what each file contains and `v1buildguide.md` says what to do next; this file says what actually happened.

**How it is kept.** One entry per prompt of `coding-prompts.md`, or per change made outside a prompt. Entries are in date order, oldest first; new entries go at the end. Every commit that changes the repository adds to this file in the same commit. The two tables below are rewritten whenever an item opens or closes: a closed item is removed and the entry that closed it says so. Item numbers are not reused.

**References.** `§n` is a section of `v1implementation.md`, `G§n` of `v1buildguide.md`, `TS §n` of `technicalspec.md`.

---

## Open items

| # | Item | Who | Needed by |
|---|---|---|---|
| 3 | Steps due before Prompt 01, not confirmed: the Vercel, Render, uptime-monitor and merchant accounts opened; one database at each Postgres candidate (TE-6); sign-up at both asset hosts (TE-7); merchant application submitted (TE-9). Done since: the dev signing keys and the GitHub repository | Human | TE-6, TE-7, TE-9 as early as possible |
| 4 | Review the reading scripts in `fixtures/speech/README.md`. The agent drafted them; the guide lists them as the founder's preparation (G§0.7) | Human | Before V2 |
| 5 | `sh2clips/documents/` is an untouched duplicate of `docs/`. Delete it, or the two will drift | Human | Any time |
| 7 | Review the 19 event descriptions in `ANALYTICS_EVENT_DOCS` (`crates/offcut-api-types/src/analytics.rs`). The agent wrote them; the settings page shows them to users as written | Human | Before Prompt 22 |

## Known issues for later prompts

| # | Issue | Affects |
|---|---|---|
| 1 | Local Postgres listens on port **9000**. The docs write the dev connection string with 5432. Use `postgres://offcut:offcut@localhost:9000/offcut_dev` in `.env` and in every `psql` command | Prompts 09, 11-13, 23 |
| 4 | CI must install binaryen `version_133` and `wasm-bindgen-cli` 0.2.129, the versions used locally | Prompt 26 |
| 5 | `pnpm` is pinned at 10.15.0, the installed version named in G§1.4. pnpm reports 12.9.1 as available, and §3.1 says to pin the latest stable release. Not changed; decide before CI is written | Prompt 26 |
| 6 | `cargo deny check` passes with warnings: `unused-wrapper` for the media and renderer crates, which do not exist until V2, and `duplicate` for ten crates present in two versions. They are warnings, not errors | Every `cargo deny check` |
| 7 | `ts-rs` runs with `no-serde-warnings`, and Cargo applies that feature to every crate in the workspace. `ts-rs` will therefore stay silent about any serde attribute it cannot read. The JSON-shape tests are the guard for any type added later | Any new shared type |
| 9 | Choices in the two type crates go beyond the letter of §6, §7 and §9. They are listed under "Differs from the specs" in the entries of Prompts 03 to 08. `v1implementation.md` has not been edited to match; reconcile it in the exit audit | Prompt 30 |
| 11 | `crates/offcut-types/tests/ui/bare_ms_rejected.stderr` holds compiler output of Rust 1.99.0. Regenerate and re-read it whenever `rust-toolchain.toml` changes; the command is in `tests/ui.rs` | Any toolchain change |
| 15 | In `gen/api.ts`, an optional event prop is typed `prop?: T \| null` (for example `unsupported_reason`). The client should leave an absent prop out; the server accepts `null` too and stores neither | Prompts 17, 18 |
| 18 | A 500 that comes from a panic in a handler does not carry `Cache-Control: no-store` and `X-Content-Type-Options: nosniff`. §10.11 puts the headers layer (4) inside the panic layer (3), so the response built after a panic never passes it. Every other response has both headers. Swapping the two layers would fix it; that is a change to the order the spec gives, so it is left for a decision | Prompt 30 |
| 19 | The SIGTERM shutdown path of `main.rs` exists only on Unix and has not run: this machine is Windows. Check with `docker stop` that the container ends within the grace period | Prompt 27 |
| 20 | The route table lives in `router.rs` and the modules have no `routes()` function, against §10.9, §10.10 and G§3.5 (see the Prompt 12 entry). Correct those sections | Prompt 30 |
| 21 | `cargo test --workspace` needs `DATABASE_URL` in the environment, pointing at a Postgres whose role has `CREATEDB`: the 23 server integration tests create a database each. Locally: `set -a; . ./.env; set +a`. CI needs a Postgres service container and the variable | Prompt 26 |
| 22 | `deny.toml` cannot tell that a pure crate turned on a random feature of `uuid`: `uuid` is a listed wrapper of `getrandom`, because the server uses its `v4` feature. The check that does tell is `cargo tree -p offcut-types -e normal` (and `-p offcut-api-types`) printing no `rand` or `getrandom`. It is run by hand; put it in a gate script or in CI | Prompts 24, 26 |
| 23 | TypeScript is pinned at 6.0.3, one major version behind the latest (7.0.2): `typescript-eslint` needs the compiler API, which version 7 does not have. Move to 7 when `typescript-eslint` supports it; `tsc` and ESLint must use the same version | Prompt 26, then any time |
| 24 | `pnpm install` prints a peer warning: `eslint-plugin-react` 7.37.5 declares ESLint up to 9, and ESLint is 10.12.0. Its one rule in use, `react/jsx-no-literals`, was proven to fire and to pass under ESLint 10. Do not treat warnings as errors in the install step of CI | Prompt 26 |
| 25 | `web/vercel.json` holds the placeholder hosts `render-host.example.invalid` and `assets.example.invalid`, and `web/.env.local` the second one. Replace all three with the real hosts (G§8.7); search for `example.invalid` | Prompt 28 |
| 27 | §11.12 has `LandingPage` and `UnsupportedPage` show the demo video "from `assetUrl`", which lives in `net/asset-fetch.ts`. TS §7 forbids `ui` to import `net`, and the lint rule refuses it. The URL must reach the page another way, for example through a use-case | Prompts 22, 23 |
| 28 | A use-case may import `workers/pool.ts` but not `workers/protocol.ts`, not even its types (TS §7). A use-case that needs to name `AppFailure` must take it from what `pool.ts` exports | Prompt 21 |
| 29 | `start-app.ts` must call `setIllegalTransitionReporter(...)` in production builds only: `if (!env.dev)`. `transition()` throws while no reporter is set, which is the behaviour development and test builds need (see the Prompt 15 entry) | Prompt 21 |
| 16 | `ERROR_CODES`, `REJECT_REASONS` and `UNSUPPORTED_REASONS` in `gen/domain.ts` are written one code per line, between `export const NAME = [` and `] as const;`. `check-copy-codes.mjs` can read them line by line | Prompt 24 |

---

## 2026-10-07 - Environment setup (outside the repository)

Changes to the machine, made for Prompt 01 (G§0.1-G§0.4). None of this is in git.

| Tool | Before | After | How |
|---|---|---|---|
| Rust stable | 1.97.1 | 1.99.0 | `rustup update stable` |
| Rust 1.99.0 toolchain (pinned) | absent | installed with `wasm32-unknown-unknown`, `rustfmt`, `clippy` | installed by rustup on first use of `rust-toolchain.toml` |
| `wasm-bindgen` CLI | 0.2.108 | 0.2.129 | `cargo install wasm-bindgen-cli --version 0.2.129 --locked --force` |
| `cargo-deny` | absent | 0.20.2 | `cargo install cargo-deny --locked` |
| `sqlx-cli` | absent | 0.9.0 | `cargo install sqlx-cli --no-default-features --features rustls,postgres --locked` |
| `wasm-opt` (binaryen) | absent | version 133 | release archive `binaryen-version_133-x86_64-windows.tar.gz`, SHA-256 checked against the published value |

Unchanged: Git 2.48.1, Node 24.19.0, pnpm 10.15.0, psql 18.4, Docker 28.3.2, OpenSSL 3.2.4, Vercel CLI 43.2.0. Not installed: GitHub CLI `gh` (optional).

**Differs from the guide.**

- `wasm-opt.exe` was copied into `~/.cargo/bin`, which is already on PATH. G§0.3 says to add binaryen's `bin` folder to PATH. The result is the same and PATH is unchanged.

**Local Postgres.**

- The service listens on port 9000, not 5432 (known issue 1).
- The `postgres` superuser password was not known, so the role could not be created at first. The human set the two loopback lines of `pg_hba.conf` to `trust` and restarted the service. The agent then ran:
  - `CREATE ROLE offcut LOGIN PASSWORD 'offcut' CREATEDB;`
  - `CREATE DATABASE offcut_dev OWNER offcut;`
- The `postgres` superuser password was not changed.
- A helper script that would have done the whole sequence was written to the session's temporary folder. It made no changes: the agent's attempt to start it was blocked, and when the human ran it, it stopped because `pg_hba.conf` already said `trust`.
- `pg_hba.conf` has not been changed back yet (open items 1 and 2).

**Checked.** `psql "postgres://offcut:offcut@localhost:9000/offcut_dev" -c "select 1"` returns one row. `uname -s` prints `MINGW64_NT-10.0-26200`. Every tool of the Milestone-0 block prints a version.

---

## 2026-10-07 - Prompt 01: preflight and repository skeleton

Commit `7ff841d`.

**Added.**

| Path | Contents |
|---|---|
| `.gitignore` | The paths of §12.8. `web/src/gen/` and `server/.sqlx/` are not ignored |
| `docs/` | `product.md`, `technicalspec.md`, `buildplan.md`, `v1/v1implementation.md`, `v1/v1buildguide.md`, `v1/Offcut V1 — 30 coding prompts.md` |
| `docs/file-specs/TEMPLATE.md` | The per-file specification template, copied from TS §34.2 |
| `fixtures/speech/README.md` | Four reading scripts (A: 60 s reference, 159 words; B: 20 s with no events; C: 20 s with one long pause; D: 30 s of sparse speech), each with its expected transcript and events |
| `bench/results/.gitkeep` | Empty |

Git: `git init -b main`; `core.autocrlf` set to `input` for this repository.

**Differs from the specs.**

- **Repository root.** The repository is `sh2clips/offcut/`, at the human's instruction. The guide assumes `sh2clips/` itself (G§1.1). Read "repo root" in every document as `offcut/`.
- **Docs copied, not moved.** G§1.1 says `mv documents docs`. The specs were copied to `offcut/docs/` and `sh2clips/documents/` was left as it was (open item 5).
- **`.gitignore`.** §12.8 says "generated fixtures" without naming paths. The patterns `fixtures/ok_*`, `fixtures/rej_*` and `fixtures/bad_*` were added, from the fixture names in TS §26.
- **Reading scripts.** Written by the agent against the detector rules of TS §17.2-§17.5 (open item 4). The expected display strings are left open, because `format_quantity` is written in V2.

**Checked.**

- `git check-ignore`: all eleven sample paths that should be ignored are; `web/src/gen/`, `server/.sqlx/`, committed fixtures and both lock files are not.
- `git status` clean after the commit; `docs/` holds the three specs.
- Word counts of the scripts counted with `wc`.

**"Done when".** All three boxes ticked. The `psql` box was ticked later the same day, with the reservation of open item 2.

---

## 2026-10-07 - Prompt 02: Cargo and pnpm workspaces

Commit `1b8ea2c`.

**Added.**

| Path | Contents |
|---|---|
| `Cargo.toml` | Workspace, `resolver = "2"`, four members; `[workspace.dependencies]`; `[workspace.lints.clippy]` denying `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `wildcard_enum_match_arm`; release profile `lto = true`, `codegen-units = 1`; no `panic = "abort"` |
| `Cargo.lock` | Four workspace packages, no external dependency yet |
| `rust-toolchain.toml` | Channel `1.99.0`, target `wasm32-unknown-unknown`, components `rustfmt` and `clippy` |
| `clippy.toml` | Disallows `std::time::Instant::now` and `std::time::SystemTime::now`; allows unwrap, expect, panic and indexing in tests |
| `deny.toml` | Advisories; a starting list of eight permissive licenses; the two groups of bans of §12.1 |
| `crates/offcut-types/`, `crates/offcut-api-types/`, `crates/offcut-wasm-core/` | `Cargo.toml` and a `src/lib.rs` holding one doc comment |
| `server/` | `Cargo.toml` (package `offcut-api`) and `src/main.rs` holding `fn main() {}` |
| `pnpm-workspace.yaml`, `package.json`, `web/package.json`, `pnpm-lock.yaml` | Workspace listing `web`; root package with `packageManager: pnpm@10.15.0`, `engines.node >= 24`, empty scripts; the `web` stub |

**Versions in `[workspace.dependencies]`** (from `cargo search` on 2026-10-07; features are set by the crate that uses each one):

| Crate | Version | Crate | Version |
|---|---|---|---|
| `serde` | 1.0.229 | `sqlx` | 0.9.0 |
| `serde_json` | 1.0.151 | `tracing` | 0.1.44 |
| `ts-rs` | 12.0.1 | `tracing-subscriber` | 0.3.23 |
| `uuid` | 1.27.0 | `time` | 0.3.55 |
| `trybuild` | 1.0.121 | `sha2` | 0.11.0 |
| `thiserror` | 2.0.21 | `base64` | 0.23.1 |
| `axum` | 0.8.9 | `ed25519-dalek` | 3.0.0 |
| `tokio` | 1.53.2 | `reqwest` | 0.13.5 |
| `tower` | 0.5.3 | `wasm-bindgen` | `=0.2.129` (exact) |
| `tower-http` | 0.7.1 | | |

**The bans in `deny.toml`.**

- `web-sys`, `js-sys`, `wasm-bindgen`, `wgpu`, `rand`, `getrandom`: an error unless every crate that depends on one directly is `offcut-wasm-core`, `offcut-wasm-render` or `offcut-render`.
- `offcut-mp4`, `-text`, `-detect`, `-dsp`, `-scene`, `-render`: each lists the crates that may depend on it, taken from the TS §7 graph. `offcut-api` is in none of the lists.

**Differs from the specs.**

- **Stub contents.** `cargo new` writes a sample `add` function and test. Each `lib.rs` was reduced to one doc comment, and `main.rs` to an empty `main`.
- **`publish = false`** was added to all four crate manifests, with `[licenses.private] ignore = true` in `deny.toml`, so `cargo deny` does not report the workspace crates as unlicensed.
- **`deny.toml`** was written by hand from the layout that `cargo deny init` produces, not by running `init` in the repository and editing the result.
- **pnpm** stays at 10.15.0 (known issue 5).

**Checked.**

- `cargo metadata`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` and `cargo deny check` all pass.
- Lint drill: a temporary `unwrap`, slice index and `Instant::now()` in non-test code were each rejected by clippy; the same `unwrap` and index in a test module were accepted. Reverted.
- Ban drill: a temporary `wasm-bindgen` dependency passed in `offcut-wasm-core` and failed `cargo deny check bans` in `offcut-types`. Reverted, `Cargo.lock` restored.
- `pnpm install` wrote `pnpm-lock.yaml`.
- `wasm-bindgen --version` prints 0.2.129, equal to the pin.
- `git status` clean after the commit, lock files included.

**"Done when".** All three boxes ticked.

---

## 2026-10-07 - This changelog

**Added.** `docs/v1/v1changelog.md`, at the human's request: every later change is summarised here in the commit that makes it.

**Changed.** `docs/technicalspec.md` §5 and `docs/v1/v1implementation.md` §4: `docs/v1/v1changelog.md` added to both file trees, so the new file is listed where `check-file-tree.mjs` will look (Prompt 24).

**Checked.** `git status` clean after the commit.

---

## 2026-10-07 - The two V1 working documents listed in the file trees

Closes known issue 3: `v1buildguide.md` and the prompts file were tracked but listed in neither file tree, so `check-file-tree.mjs` (Prompt 24) would have rejected them.

**Renamed.** `docs/v1/Offcut V1 — 30 coding prompts.md` to `docs/v1/coding-prompts.md`. The new name is the one in the file's own title. The old name contains spaces, and the file trees separate a path from its description with spaces, so it could not be listed without ambiguity. The file's contents are unchanged. The copy in `sh2clips/documents/` keeps the old name. The Prompt 01 entry above shows the name as it was then.

**Changed.** `docs/technicalspec.md` §5 and `docs/v1/v1implementation.md` §4: `docs/v1/v1buildguide.md` and `docs/v1/coding-prompts.md` added to both file trees.

**Checked.** Every tracked file under `docs/` is now in the TS §5 tree. `git status` clean after the commit.

---

## 2026-10-07 - Prompt 03: `offcut-types` units, ids, limits

**Added.**

| Path | Contents |
|---|---|
| `crates/offcut-types/src/units.rs` | The 16 unit types of TS §10.1 and `Span`; `const fn new` and `get` on the 13 integer units; `Option` constructors on `Confidence`, `Lufs`, `Dbfs` and `Span`; the six conversion and arithmetic functions of §6.1; 13 unit tests |
| `crates/offcut-types/src/ids.rs` | `ClipId`, `ExportId`, `UserId`, `AnonId` (each wraps a `Uuid`), `EventId`, `WordIdx`, `SentenceIdx`, `WordRange`, `JobId`; `const fn new` and `get` on the eight one-field ids; no function generates an id; 7 unit tests |
| `crates/offcut-types/src/limits.rs` | The 21 constants of TS §10.2 with their comments, and nothing else |

**Changed.**

| Path | Change |
|---|---|
| `crates/offcut-types/Cargo.toml` | Dependencies `serde` (`derive`), `ts-rs` (`no-serde-warnings`), `uuid` (`serde` only); dev-dependencies `trybuild`, `serde_json` |
| `crates/offcut-types/src/lib.rs` | Declares `ids`, `limits`, `units` and re-exports their public items at the crate root |
| `Cargo.lock` | The new dependencies |

**How the types serialize.**

- Integer units and ids: a bare number (`#[serde(transparent)]`). `DurMs::new(90_000)` is `90000`.
- `Uuid`-backed ids: the hyphenated string.
- `EventId`: a decimal string, `"18446744073709551615"`. Reading accepts only the form that writing produces: digits, no sign, no leading zeros.
- `Confidence`, `Lufs`, `Dbfs`: a number. Reading goes through `new`, so an out-of-range or non-finite value is an error.
- `Span`, `WordRange`: an object with the Rust field names, `{"start":10,"end":20}`.

**Differs from the specs.** Known issue 9 tracks the first four.

- **`EventId` is not `#[serde(transparent)]`.** §6 says id types are transparent; §9 says `EventId` is serialized as a decimal string because a JavaScript number cannot hold 64 bits. The two cannot both hold. §9 is the specific rule and is marked as a V1 decision, so it was followed, with a hand-written `Serialize` and `Deserialize`.
- **`Lufs` and `Dbfs` validate when deserializing.** §6.1 asks for this only for `Confidence`. Their `new` rejects non-finite values, and a derived `Deserialize` would have let one in past `new`.
- **`Span` and `WordRange` derive `Copy, Eq, PartialOrd, Ord, Hash`.** §6 gives these to "integer unit types and id types"; TS §10 gives them to "unit and id types". The two structs live in `units.rs` and `ids.rs`, so the TS §10 reading was used.
- **`WordRange` has no `new` or `get`.** §6.2 says "each has `new` and `get`" but gives no signature or invariant for a two-field range. Its fields are public, so it is built with a struct literal. A constructor can be added later without breaking anything.
- **`serde_json` is a dev-dependency.** The manifest in the prompt names only `trybuild`. The round-trip tests this prompt asks for, and the JSON-shape tests of Prompts 04 and 05, need a serde format. It is used in tests only.
- **`ts-rs` feature `no-serde-warnings`.** `ts-rs` 12.0.1 does not know `#[serde(transparent)]` and printed a warning for each of the 20 uses on every build. On a one-field struct it already emits the inner type, so the attribute changes nothing for it (known issue 7).
- **`#[ts(type = "...")]` on some fields.** `"number"` on the four 64-bit units (`ts-rs` would say `bigint`) and `"string"` on the `Uuid` ids and `EventId`. This also avoids the `uuid-impl` feature of `ts-rs`.
- **Overflow.** §6.1 gives `n * 1000 / 30` without saying what happens past `u32::MAX` ms. `FrameIdx::to_time_ms` and `TimeMs::from_micros_floor` saturate at `u32::MAX`; neither can panic. No accepted clip comes near that value.
- **The divisor 30** is read from `limits::OUTPUT_FPS`, not typed a second time.
- **`trybuild`** is in the manifest but unused until Prompt 05, as the prompt orders.

**Checked.**

- `cargo test -p offcut-types`: 20 tests pass. They include the six of §6.7, serde round-trips for units and ids, the `EventId` string form (also as a map key), and the `ts-rs` names and shapes.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` and `cargo deny check` pass.
- `cargo check -p offcut-types --target wasm32-unknown-unknown` passes.
- `cargo tree`: `uuid` has only its `default` and `serde` features; no `rand` or `getrandom` in the dependency graph.
- Counts by `grep`: 16 one-field unit structs plus `Span` in `units.rs`; 21 `pub const` and no other item in `limits.rs`; no `From<` implementation anywhere in the crate.
- No `unwrap`, `expect`, `panic!` or slice indexing outside the test modules.
- Longest file: `units.rs`, 338 lines.

**"Done when".** All three boxes ticked.

---

## 2026-10-07 - Prompt 04: `offcut-types` stage, media, transcript, events, prosody

**Added.**

| Path | Types | Tests |
|---|---|---|
| `crates/offcut-types/src/stage.rs` | `PipelineStage` (six stages, PS §20.2 order) | 3 |
| `crates/offcut-types/src/media.rs` | `Rotation`, `Orientation`, `ContainerKind`, `VideoCodec`, `AudioCodec`, `ProbeInfo`, `VideoTrackInfo`, `AudioTrackInfo`, `ClipInfo`, `RejectReason` (14 variants) | 6 |
| `crates/offcut-types/src/transcript.rs` | `Word`, `Sentence`, `Unit`, `Quantity`, `NormalizedSpan`, `Transcript` | 5 |
| `crates/offcut-types/src/events.rs` | `EventKind`, `ListItem`, `EventParams`, `DetectedEvent` | 5 |
| `crates/offcut-types/src/prosody.rs` | `WordProsody`, `Prosody` | 1 |

The definitions are those of TS §10.3-§10.5, field for field. The five files hold types only: no function and no `impl` outside the test modules.

**Changed.** `crates/offcut-types/src/lib.rs`: declares and re-exports the five new modules (eight so far; Prompt 05 adds the last four).

**How the types serialize (D-2).**

- Plain enums: a `snake_case` string. `PipelineStage::ProbeAudio` is `"probe_audio"`, `EventKind::NumberReveal` is `"number_reveal"`, `Rotation::R90` is `"r90"`.
- `RejectReason`: its code, by an explicit rename on each variant. `RejectReason::NoSpeech` is `"REJECT_NO_SPEECH"`.
- Structs: an object with the Rust field names (`start_ms`, `file_size`, `per_word`). An `Option` that is `None` is written as `null`.
- `EventParams`: an object tagged with `kind`, its other fields beside the tag: `{"kind":"keyword_pop","word":7}`.
- `Unit`: a string for the 19 variants without data; `{"count":{"noun":"events"}}` for `Count`.

**Worth knowing.**

- `VideoCodec::ProRes` serializes as `"pro_res"`. That is what `snake_case` gives; the specs name no string for it.
- A `DetectedEvent` carries `kind` twice: once as its own field and once as the tag inside `params`. That follows from the two spec definitions.
- `ts-rs` copies doc comments on types and fields into the TypeScript it emits. The generator of Prompt 08 will carry them into `gen/domain.ts`.

**Differs from the specs.**

- **Enums without data also derive `Copy, Eq, Hash`.** §6 lists six derives for every type and adds more only for unit and id types. The eight enums without data (`PipelineStage`, `Rotation`, `Orientation`, `ContainerKind`, `VideoCodec`, `AudioCodec`, `RejectReason`, `EventKind`) got the three extra, so they can be passed by value, compared and used as map keys without a later edit to a frozen file. Structs, `Unit` and `EventParams` have exactly the six derives of §6 (known issue 9).

**Checked.**

- `cargo test -p offcut-types`: 40 tests pass (20 new). The new ones cover the three shapes the prompt names and more:
  - `RejectReason`: all 14 JSON strings, in the order of §6.4, through a match with no wildcard, so a fifteenth variant does not compile until the test is updated.
  - `EventParams`: the tagged form of all four variants, compared with literal JSON, and rejection of an unknown or missing `kind`.
  - `PipelineStage`: all six `snake_case` strings, and rejection of other spellings.
  - Round-trips with literal JSON for `ProbeInfo`, `ClipInfo`, `AudioTrackInfo`, `Transcript`, `DetectedEvent` and `Prosody`.
  - The `ts-rs` output of `PipelineStage`, `RejectReason`, `Unit` and `EventParams`, which guards known issue 7.
- The TypeScript declarations of 14 of the new types were printed once and read against the JSON above. They agree.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` and `cargo deny check` pass.
- `cargo check -p offcut-types --target wasm32-unknown-unknown` passes.
- No `unwrap`, `expect`, `panic!` or slice indexing outside the test modules.
- Longest file: `units.rs`, 338 lines; `media.rs` has 326.

**"Done when".** Both boxes ticked: check and test are green with the derives above, and `RejectReason` has 14 variants whose names match §6.4.

---

## 2026-10-07 - Prompt 05: `offcut-types` edit, profile, summary, error, lib, compile-fail test

This prompt finishes the crate: twelve modules, 57 unit tests and one compile-fail test.

**Added.**

| Path | Contents | Tests |
|---|---|---|
| `crates/offcut-types/src/edit.rs` | `StyleId`, `CropOffset` (`new`, `get`), `EditState` with `Default` | 6 |
| `crates/offcut-types/src/profile.rs` | `Plan`, `ProfileKind`, `ExportProfile` | 2 |
| `crates/offcut-types/src/summary.rs` | `ChangeSummary` | 1 |
| `crates/offcut-types/src/error.rs` | `ErrorCode` (29), `UnsupportedReason` (11), `FailureStage` (11), `From<PipelineStage> for FailureStage` | 5 |
| `crates/offcut-types/tests/ui.rs` | Runs every `tests/ui/*.rs` as a compile-fail case; its header says when and how to regenerate the expected output | 1 |
| `crates/offcut-types/tests/ui/bare_ms_rejected.rs` | Passes a bare `u32`, then a `TimeMs`, where a `DurMs` is expected | |
| `crates/offcut-types/tests/ui/bare_ms_rejected.stderr` | The compiler output the case must produce, generated with `TRYBUILD=overwrite` | |

**Changed.**

| Path | Change |
|---|---|
| `crates/offcut-types/src/lib.rs` | Declares and re-exports all twelve modules. Three tests for the code enums (see "Checked") |
| `crates/offcut-types/src/ids.rs` | The `Deserialize` of `EventId` is written without a `match`. The first version ended in a `_ =>` arm; invariant 8 allows no wildcard. Behaviour is unchanged and its five tests still pass |

**How the types serialize (D-2).**

- `ErrorCode`: its code, by an explicit rename on each variant. `ErrorCode::WorkerCrash` is `"E_WORKER_CRASH"`.
- `UnsupportedReason`: likewise. `UnsupportedReason::WebGpu` is `"UNSUPPORTED_WEBGPU"`.
- `FailureStage`, `StyleId`, `Plan`, `ProfileKind`: a `snake_case` string. `Plan::Creator` is `"creator"`.
- `CropOffset`: a number. Reading goes through `new`.
- `EditState`: `{"word_edits":{"3":"ten thousand"},"event_overrides":{"42":false},"style":"clean","crop_offset":0.0}`. JSON map keys are strings, so a `WordIdx` key is written as `"3"`.

**The compile-fail output was read.** It holds two errors, both `E0308: mismatched types`: ``expected `DurMs`, found `u32` `` on line 9 and ``expected `DurMs`, found `TimeMs` `` on line 10. Neither is a missing import or an unresolved name. The file is tied to Rust 1.99.0 (known issue 11).

**Differs from the specs.**

- **`CropOffset` validates when deserializing.** TS §10.6 says the range is "checked in `new()`". A derived `Deserialize` would bypass `new`, so reading goes through it, as for `Confidence` (known issue 9).
- **`StyleId` implements `Default` (`Clean`).** TS §10.6 says "default Clean" in a comment; §6.4 asks only for `EditState::default()` (known issue 9).
- **`CropOffset` has `get` and derives `Copy, PartialOrd`.** The specs name only `new`. Without `get` the value could not be read outside the crate, since the field is private.
- **Enums without data derive `Copy, Eq, Hash`**, as chosen in Prompt 04: `StyleId`, `Plan`, `ProfileKind`, `ErrorCode`, `UnsupportedReason`, `FailureStage`.
- **Where the code-enum tests live.** §6.6 puts the round-trip tests of the three code enums in `lib.rs`. They are there. The tests that pin the order of each list against the spec are next to the enums, in `error.rs` and `media.rs`.

**Checked.**

- `cargo test -p offcut-types`: 57 unit tests and the `ui` test pass, in normal mode (without `TRYBUILD=overwrite`).
- The counts 29 / 11 / 11 are each verified twice by tests:
  - `error.rs`: a literal list of each enum's strings in spec order (29 `E_*` from TS §11.2, 11 `UNSUPPORTED_*` from TS §13.2, 11 stage names), checked through a match with no wildcard, so a new variant does not compile until the test is updated. The `ts-rs` union of each enum must equal the same list.
  - `lib.rs`: for `ErrorCode`, `RejectReason` and `UnsupportedReason`, the codes are read from the TypeScript union and each must round-trip through serde unchanged. Counts 29, 14 and 11; prefixes `E_`, `REJECT_`, `UNSUPPORTED_`; no duplicates.
- `From<PipelineStage>` has one arm per stage, and each stage maps to the failure stage with the same string.
- `EditState::default()` has no edits, no overrides, `StyleId::Clean` and offset `0.0`.
- `CropOffset::new` accepts -1.0, 0.0 and 1.0 and rejects ±1.001, NaN and both infinities.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` and `cargo deny check` pass.
- `cargo check -p offcut-types --target wasm32-unknown-unknown` passes.
- No `unwrap`, `expect`, `panic!`, slice indexing or wildcard match arm outside the test modules.
- Longest file: `error.rs`, 362 lines, of which 233 are tests.

**Not done, as the prompt orders.** `EntitlementClaims` (V2). The generator test `write_typescript` (Prompt 08).

**"Done when".** Both boxes ticked.

---

## 2026-10-07 - Prompt 06: `offcut-api-types` DTOs and API errors

**Added.**

| Path | Types | Tests |
|---|---|---|
| `crates/offcut-api-types/src/auth.rs` | `MagicLinkRequest`, `VerifyRequest`, `SessionResponse` | 4 |
| `crates/offcut-api-types/src/account.rs` | `MeResponse`, `Wanted`, `NotifyMeRequest`, `AccountExport`, `ExportedUser`, `ExportedSession`, `ExportedReceipt` | 6 |
| `crates/offcut-api-types/src/billing.rs` | `SubscriptionSummary`, `SubscriptionStatus`, `BillingInterval`, `Offer`, `CheckoutRequest`, `UrlResponse` | 4 |
| `crates/offcut-api-types/src/usage.rs` | `UsageReceiptRequest`, `EntitlementResponse` | 2 |
| `crates/offcut-api-types/src/errors.rs` | `ApiError`, `ApiErrorCode` (13 members, D-3) | 4 |

The definitions are those of TS §10.7 and §7.6, field for field; the three `Exported*` structs are the V1 decision of §7.1. The files hold definitions only: no function and no `impl` outside the test modules.

**Changed.**

| Path | Change |
|---|---|
| `crates/offcut-api-types/Cargo.toml` | Dependencies `offcut-types`, `serde` (`derive`), `ts-rs`; dev-dependency `serde_json` |
| `crates/offcut-api-types/src/lib.rs` | Declares and re-exports the five modules. `analytics` follows in Prompt 07 |
| `Cargo.toml` | `offcut-types` and `offcut-api-types` added to `[workspace.dependencies]` as path dependencies, so each crate that uses them writes `workspace = true` |
| `Cargo.lock` | The new dependency edges |

**How the types serialize.**

- Enums: a `snake_case` string. `Wanted::Safari` is `"safari"`, `SubscriptionStatus::PastDue` is `"past_due"`, `Offer::CreatorAnnualFounding` is `"creator_annual_founding"`, `ApiErrorCode::NotFound` is `"not_found"`.
- Structs: an object with the Rust field names. `{"email":"a@example.com","wanted":"safari"}`.
- `ApiError`: `{"code":"rate_limited","retry_after_secs":30}`. Without a retry time the field is written as `null`: `{"code":"not_found","retry_after_secs":null}`. A body that leaves the field out is also read.

**The five request structs deny unknown fields:** `MagicLinkRequest`, `VerifyRequest`, `NotifyMeRequest`, `CheckoutRequest`, `UsageReceiptRequest`. The sixth request type, `EventsBatch`, arrives with `analytics.rs` in Prompt 07. Response structs do not deny unknown fields.

**Differs from the specs.**

- **Enums derive `Copy, Eq, Hash`** as well as the six derives of §6, the same choice as in Prompt 04: `Wanted`, `SubscriptionStatus`, `BillingInterval`, `Offer`, `ApiErrorCode`.
- **`serde_json` is a dev-dependency**, for the round-trip tests, as in `offcut-types`.
- **`retry_after_secs` is written as `null` when absent.** The specs give the field as `Option<u32>` and do not say whether an absent value is left out. The plain derive was kept. Prompt 16 (`http.ts`) must accept both `null` and a missing field.

**Checked.**

- `cargo test -p offcut-api-types`: 20 tests pass.
  - A round-trip against literal JSON for each of the 15 structs, and every value of each of the 5 enums.
  - An unknown field on `NotifyMeRequest` is rejected. An unknown `wanted` is rejected (`"edge"`, `"Launch"`, `"LAUNCH"`, `""`), and so is a missing `email` or `wanted`.
  - An unknown field is rejected on each of the other four request structs.
  - `ApiErrorCode`: its 13 strings in the order of §7.6, through a match with no wildcard, and the `ts-rs` union equal to the same list.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` and `cargo deny check` pass.
- `cargo check -p offcut-api-types --target wasm32-unknown-unknown` passes.

**"Done when".** The box is ticked: tests green, 13 `ApiErrorCode` members, every request struct that exists so far denies unknown fields.

---

## 2026-10-07 - Prompt 07: `offcut-api-types` analytics allowlist

**Added.** `crates/offcut-api-types/src/analytics.rs` (392 lines, 7 tests):

- The constants `MAX_EVENTS_PER_BATCH` (50), `MAX_DURATION_MS` (3,600,000) and `MAX_COUNT` (10,000).
- The 16 prop enums of §7.2, each serialized as `snake_case` strings.
- `AnalyticsEvent` with the 19 variants of §7.3, and `AnalyticsEvent::name()`, an exhaustive match with no wildcard.
- `EventsBatch { anon_id, events }`, which denies unknown fields.
- `EventDoc` and `ANALYTICS_EVENT_DOCS`: 19 entries in enum order, one sentence each.

**Changed.** `crates/offcut-api-types/src/lib.rs`: declares and re-exports `analytics`.

**How an event serializes.**

- With props: `{"name":"landing_view","props":{"hero_variant":"outcome"}}`.
- Without props: `{"name":"preview_played"}`, with no `props` key.
- An optional prop that is absent is left out. The four optional props are `unsupported_reason`, `asr_backend`, `event_kind` and `enabled`.

**What is rejected.** Each of these fails to deserialize, and each is a test case:

- An unknown prop; an unknown event name; an unknown key beside `name` and `props`.
- Free text or a number where an enum is expected.
- Text where a number or a boolean is expected; a negative number; a number above `u32::MAX`.
- A missing prop; missing `props`; a missing `name`.
- Props on an event that has none (`preview_played` with an object or a string as `props`).
- In a batch: an unknown key such as `user_id`; an `anon_id` that is not a UUID; a missing `anon_id`; one bad event among good ones.

One thing is accepted that is not in the samples: `"props": null` on an event without props. It is read as the event and written back without `props`, so nothing extra can be carried.

**The allowlist rule.** No variant has a text field: every prop is an enum, a `bool` or a `u32`. The word `String` does not occur in `analytics.rs` at all (`grep -c String` prints 0), and a test fails if it is ever added above the test module or if the generated TypeScript types any prop as `string`.

**Differs from the specs.**

- **The 16 prop enums are declared through a macro**, one line per enum (`GpuVendor: Intel, Amd, Nvidia, Apple, Qualcomm, Other, Unknown;`), in the form of the table in §7.2. Written out one by one they took 150 lines, and the file with its tests would have passed the 400-line limit. The generated enums are the same. A search for `pub enum GpuVendor` finds nothing; search for `GpuVendor:`.
- **`ANALYTICS_EVENT_DOCS` is built with a private `const fn doc(name, description)`** and marked `#[rustfmt::skip]`, one line per event, for the same reason. 20 lines of the file are longer than 100 characters; all are string tables.
- **The `serde` attribute of `AnalyticsEvent` is written as two attributes** (`tag`/`content`/`rename_all`, then `deny_unknown_fields`). The formatter breaks the single attribute of §7.3 over six lines. The meaning is the same.
- **Prop enums derive `Copy, Eq, Hash`**, as chosen in Prompt 04.
- **The event descriptions were written by the agent** (open item 7). §7.3 gives one example, "How long each processing stage took", which is used unchanged for `stage_timing`.

**Checked.**

- `cargo test -p offcut-api-types`: 27 tests pass (7 new).
  - One sample of each of the 19 variants, as the JSON the client sends: it deserializes, its `name()` equals the docs entry at the same position, it serializes back to the identical string, and its keys are `name` and `props`, or `name` alone.
  - The docs names have no duplicates and follow the enum order; the TypeScript union has 19 members; each description is one sentence.
  - The 16 prop enums have exactly the values of §7.2, row for row.
  - The rejections listed above.
- Counts: 19 variants, 19 `name()` arms, 19 docs, 16 prop enums.
- All six request types now deny unknown fields: the five of Prompt 06 and `EventsBatch`.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` and `cargo deny check` pass.
- `cargo check -p offcut-api-types --target wasm32-unknown-unknown` passes.

**"Done when".** Both boxes ticked.

---

## 2026-10-07 - Prompt 08: code generation to TypeScript

**Added.**

| Path | Contents |
|---|---|
| `scripts/gen-types.sh` | Runs the two ignored `write_typescript` tests with `OFFCUT_GEN_OUT` set. POSIX `sh`; it changes to the repository root first, so it works from any folder |
| `scripts/check-gen-clean.sh` | Runs `gen-types.sh`, then `git diff --exit-code -- web/src/gen`; prints `gen clean: ...` on success |
| `web/src/gen/domain.ts` | Generated, committed: 270 lines |
| `web/src/gen/api.ts` | Generated, committed: 135 lines |

**Changed.**

| Path | Change |
|---|---|
| `crates/offcut-types/src/lib.rs` | A second test module, `typescript`: the generator of `domain.ts`, the ignored test `write_typescript` that writes it, and a normal test of its output. 345 lines |
| `crates/offcut-api-types/src/lib.rs` | The same for `api.ts`. 216 lines |
| `package.json` | Root script `gen:types`: `sh scripts/gen-types.sh` |

**What `domain.ts` holds, in order.**

1. A two-line header: generated, do not edit, how to regenerate.
2. 24 branded aliases: the 16 units and 8 ids. `export type TimeMs = number & { readonly __unit: "TimeMs" };`. The four `Uuid` ids and `EventId` are branded strings.
3. The other 35 types from their `ts-rs` declarations, in module order: `Span`, `WordRange`, then media, transcript, events, prosody, edit, profile, summary, stage, error.
4. `LIMITS`: 21 properties, `as const`. A unit value carries its brand (`MAX_CLIP_DURATION: 90000 as DurMs`); a plain one does not (`OUTPUT_FPS: 30`).
5. `ERROR_CODES` (29), `REJECT_REASONS` (14), `UNSUPPORTED_REASONS` (11): one code per line, `as const`.

**What `api.ts` holds, in order.** The header; one `import type { ... } from "./domain";` with 12 names; the 38 types of section 7 in module order (auth, account, billing, usage, analytics, errors); `ANALYTICS_EVENT_DOCS` with 19 entries, `as const`; `export const MAX_EVENTS_PER_BATCH = 50;`.

**How the generators stay complete.** Nothing in them is a second copy of the Rust source:

- The base of each brand (`number` or `string`) comes from the type's own `ts-rs` output.
- The import list of `api.ts` is computed: every name the API types refer to, minus the names they declare.
- The code arrays are read from the `ts-rs` union of each enum.
- A normal test in each crate, run by every `cargo test`, reads the crate's source files and fails if a `pub struct` or `pub enum` (or a row of the `prop_enums!` table) is missing from the output, or declared twice. The same test fails if a `pub const` of `limits.rs` is missing from `LIMITS`, and checks the counts 59 types, 24 brands, 21 limits, 29 / 14 / 11 codes, 38 API types, 19 docs.

**Closes two known issues.**

- **Known issue 8** (brands): done as described above.
- **Known issue 10** (the two maps of `EditState`): no override is needed. `tsc` reads `{ [key in WordIdx]: string }` over a branded key as an index signature: `{}` is a valid value, and a lookup with a `WordIdx` gives `string | undefined` under `noUncheckedIndexedAccess`.

**Differs from the specs.**

- **Rust doc comments are carried into the TypeScript**, on the brands too. §9 does not mention comments. They come from `ts-rs` and cost nothing to keep.
- **Trailing spaces are removed** from each generated line. `ts-rs` leaves one after some fields of a documented type.
- **Each crate has a second, non-ignored generator test.** §9 names only `write_typescript`. Without it, a broken generator would be noticed only when the script runs.
- **`check-gen-clean.sh` prints a success line.** The guide's version prints nothing; the prompt asks that it "prints success".

**Checked.**

- `sh scripts/gen-types.sh` twice: the SHA-256 of both files is unchanged. Also unchanged when run through `pnpm gen:types` and when started from another folder.
- `sh scripts/check-gen-clean.sh` prints `gen clean`. Drill: a doc comment in `units.rs` was changed, the check exited 1 and showed the differing line; reverted, the check passed again.
- Both files and both scripts have LF line endings and no carriage return. The generated files have no trailing spaces.
- Counts in the files: `ERROR_CODES` 29, `REJECT_REASONS` 14, `UNSUPPORTED_REASONS` 11, `LIMITS` 21, `ANALYTICS_EVENT_DOCS` 19.
- **Type check** (known issue 12): `tsc` 7.0.2 with `strict`, `noUncheckedIndexedAccess` and `exactOptionalPropertyTypes` accepts both files, together with a file that uses them: limits with their brands, the three arrays compared with their unions, the two maps of `EditState`, four sample events, a waitlist request, an `ApiError`.
- **Misuse is rejected.** A second file with seven mistakes gives seven errors, one per line: a bare number as a `DurMs`; a `TimeMs` as a `DurMs`; a `ClipId` as an `ExportId`; an event with a prop that is not on the allowlist; an event with free text where an enum is expected; an unknown event name; an unknown `wanted`.
- `cargo test --workspace`: 58 + 28 tests pass, plus the `ui` test; the two `write_typescript` tests are ignored in a normal run.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo deny check` pass.

**"Done when".** All three boxes ticked.

---

## 2026-10-07 - Local Postgres: password checks back on

Outside the repository. Closes open items 1 and 2.

**Changed (by the human).** In `C:\Program Files\PostgreSQL\18\data\pg_hba.conf`, the two loopback lines were set back from `trust` to `scram-sha-256`, and the `postgresql-x64-18` service was restarted. All six rules of the file now say `scram-sha-256`.

**Checked (by the agent, on port 9000).**

- Login as `postgres` without a password is refused: `fe_sendauth: no password supplied`.
- Login as `offcut` with a wrong password is refused: `password authentication failed`.
- `psql "postgres://offcut:offcut@localhost:9000/offcut_dev" -c "select 1"` returns one row. This is the Milestone-0 check of Prompt 01, now proven with the password check on.
- The `offcut` role has `CREATEDB`, which the integration tests of Prompt 13 need.

Whether the `postgres` superuser password was changed while access was open is the human's own record; nothing in Offcut uses that login.

---

## 2026-10-07 - Local `.env` complete

Outside the repository (`.env` is ignored by git). Closes the signing-key part of open item 3.

**Changed (by the human).** `.env` at the repository root was rewritten with all 15 variables of G§3.1, with LF line endings. The first version held only `DATABASE_URL` and had CRLF endings, which would have put a carriage return at the end of every value loaded with `set -a; . ./.env; set +a`. The two dev signing keys were generated on the machine with `openssl rand -base64 32` and written straight to the file.

**Checked (by the agent, without printing any value).** 15 of 15 variables present; `DATABASE_URL` points at `offcut_dev` on port 9000 and the database answers with it; `APP_ORIGIN` is `http://localhost:5173`; `PORT` 8080; both keys are standard base64 that decodes to 32 bytes, and they differ; the seven mail and billing variables are `unset-until-v6`; the file loads in `sh` with no trailing characters.

---

## 2026-10-07 - Prompt 09: server primitives

**Added.**

| Path | Contents | Tests |
|---|---|---|
| `server/src/lib.rs` | Module declarations only: `client_ip`, `config`, `error`, `headers`, `log` | |
| `server/src/config.rs` | `Config`, `ConfigError { Missing, Malformed }`, `Config::from_env()`; a hand-written `Debug` | 8 |
| `server/src/log.rs` | `init(level)` (JSON to stdout), `request_span(req)`, `user_tag(user_id)` | 4 |
| `server/src/error.rs` | `AppError` (7 variants), its `IntoResponse`, `From<sqlx::Error>`, `json_rejection` | 6 |
| `server/src/headers.rs` | `layer()`: `Cache-Control: no-store` and `X-Content-Type-Options: nosniff` on every response | 1 |
| `server/src/client_ip.rs` | `client_ip(headers, peer, trusted_hops)` | 11 |

**Changed.**

| Path | Change |
|---|---|
| `server/Cargo.toml` | The dependencies of §10: `axum`, `tokio`, `tower`, `tower-http`, `sqlx` (`postgres`, `runtime-tokio`, `tls-rustls`, `uuid`, `time`, `json`, `migrate`, `macros`), `serde`, `serde_json`, `thiserror`, `tracing`, `tracing-subscriber` (`json`), `uuid` (`v4`, `v7`, `serde`), `time`, `sha2`, `base64`, `ed25519-dalek`, `offcut-types`, `offcut-api-types`; dev-dependency `reqwest` |
| `Cargo.lock` | The server's dependency tree |
| `docs/technicalspec.md` §5, `docs/v1/v1implementation.md` §4 | `server/src/lib.rs` added to both file trees, as the prompt requires |

`server/src/main.rs` is still `fn main() {}`; Prompt 12 writes it.

**How each file behaves.**

- **`config.rs`.** The only reader of the environment. All 15 variables are required; an empty value counts as missing; `GIT_SHA` is optional and defaults to `dev`. An error carries the variable's name and prints as `config error: <NAME>`, never the value.
  - A signing key must be standard base64 of exactly 32 bytes.
  - `APP_ORIGIN` must be `https://host` (a port is allowed) or `http://localhost:port`, with nothing after it: no path, no query, no trailing slash.
  - `PORT` is a `u16`, `LOG_LEVEL` a tracing level, `FOUNDING_OFFER_ENABLED` exactly `true` or `false`, `TRUSTED_PROXY_HOPS` a `u8`.
  - `Debug` prints `<redacted>` for six fields: `database_url`, both signing keys, `mail_api_key`, `billing_api_key`, `billing_webhook_secret`.
- **`log.rs`.** The request span carries the request id, the method and the route template. When no route matched it logs `unmatched`; the raw path and the query string are never read. `user_tag` is the first 8 hex characters of the SHA-256 of the user id.
- **`error.rs`.** The mapping of the §10.3 table. `RateLimited` also sets a `Retry-After` header. A database error is logged as one fixed word for its kind (`database`, `pool_timed_out`, `io` and so on), never with its message, which can hold values from the query.
- **`client_ip.rs`.** The algorithm of §10.5 with D-17: no header or zero hops gives the peer; otherwise the entry `trusted_hops` from the right; fewer entries than hops gives the leftmost; an entry that does not parse gives the peer.

**Differs from the specs.**

- **`http` types come from `axum::http`.** §10.2 and §10.5 write `http::Request` and `http::HeaderMap`. The `http` crate is not in the dependency list, and `axum` re-exports the same types.
- **`config.rs` has a private `from_lookup`**, which `from_env` calls with `std::env::var`. The tests pass a map to it. Changing the process environment in a test is `unsafe` in the 2024 edition and races with other tests.
- **`ConfigError` implements `Display` and `Error`** (through `thiserror`), so `main` can print it. Both variants print the same text, `config error: <NAME>`, which is the text §10.12 gives.
- **`headers::layer()` returns a named type, `HeadersLayer`**, not `impl tower::Layer<...>`. `Router::layer` needs to know the service type the layer produces, and an opaque return type hides it.
- **The two headers replace** any value a handler set. §10.4 says "adds".
- **`X-Forwarded-For` sent on several header lines is read as one list.** §10.5 speaks of "the header". HTTP allows the split, and reading only the first line would pick the wrong entry.
- **`user_tag` hashes the id as text**, the lower-case hyphenated UUID. §10.2 says "SHA-256(user id)" without a form. The text form can be checked by hand with `sha256sum`.
- **`log.rs` logs a request id only if it is a plain id** (letters, digits, hyphens, at most 64 characters). A client can send its own `X-Request-Id`, and the rule is that no text from a request reaches the log (known issue 17).
- **`log::init` does nothing on a second call** instead of panicking.
- **`error.rs`, `headers.rs` and `log.rs` have unit tests.** G§3.4 lists none for them; they are small and need no database.

**Checked.**

- `cargo test -p offcut-api`: 30 tests pass.
  - `config`: a complete environment parses, field by field; each of the 15 variables, removed or emptied, yields `Missing(<its name>)`; four kinds of bad key yield `Malformed` (not base64, 31 bytes, 33 bytes, URL-safe alphabet); 13 bad origins are rejected and 3 good ones accepted; `Debug` contains none of seven secret strings and exactly six `<redacted>`.
  - `client_ip`: one, two and three entries with hops 1 and 2; absent header; zero hops; garbage in and outside the chosen position; IPv6; spaces and tabs; more hops than entries; several header lines; a line that is not text.
  - `error`: each variant's status and body; `Retry-After`; the JSON extractor's rejections mapped for seven bad bodies (400), three wrong content types (415) and a 3 MB body (413).
  - `headers`: both headers on a normal response, on a handler that set its own `Cache-Control`, and on a 404; no `access-control-*` header.
  - `log`: `user_tag` of a fixed id equals `29af63ce`, the value `sha256sum` gives for that id; an unmatched request logs neither path nor query.
- `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` pass. `cargo test --workspace` passes: 58 + 28 + 30 tests and the `ui` test. `check-gen-clean.sh` prints `gen clean`.
- Only `config.rs` reads the environment. No file uses a `std::time` clock.
- No `unwrap`, `expect`, `panic!`, slice indexing or wildcard match arm outside the test modules.
- Longest file: `config.rs`, 366 lines.

**Not passing: `cargo deny check`.** It fails on bans and licenses now that the server has real dependencies. This was expected and is Prompt 13's work (known issue 2). The pure crates were checked by hand and still pull in no randomness.

**"Done when".** Both boxes ticked: the unit tests are green and clippy is clean; the `Debug` test proves no secret leaks.

---

## 2026-10-07 - GitHub repository

Closes open item 6 (G§1.6).

**Added.** The remote `origin`: `https://github.com/mayanpathak/Offcut.git`. The repository is public and was empty. `main` was pushed with its 12 commits, up to `dd30e99` (Prompt 09), and tracks `origin/main`.

**Checked before the push, over the whole history.**

- `.env` was never committed, and no tracked file is named like a secret.
- Neither dev signing key from `.env` occurs in any commit. They were compared without being printed.
- No private-key block occurs in any commit.
- The only key-shaped strings are the test constants in `server/src/config.rs`.
- The dev database login `offcut:offcut` occurs in the docs and in this changelog. It is the local development login of G§0.4 and reaches nothing outside the machine.

**Checked after the push.** `git ls-remote origin` and the GitHub API both report `dd30e99` as the head of `main`, equal to the local head.

**Worth knowing.**

- The push made public: the product spec, the technical spec, the build plan, the V1 documents and this changelog.
- The commits carry the author name and email of the local git configuration.
- GitHub's quick-start snippet (`git init`, a new `README.md`, a "first commit") was not used. The repository already existed locally, and `README.md` at the root is not in the TS §5 tree.
- Later commits are pushed when the human asks for it. CI does not exist until Prompt 26, so nothing depends on the remote before then.

---

## 2026-10-07 - Prompt 10: rate limiter and app state

**Added.**

| Path | Contents | Tests |
|---|---|---|
| `server/src/rate_limit.rs` | `RouteGroup` (16 groups), `Limit`, `RouteGroup::limit()` (a `const fn` holding the whole TS §22.1 table), `RateKey { Ip, EmailHash, User }`, `RateLimiter::new(max_keys)` and `check(group, key, now)`, the `by_ip` middleware | 11 |
| `server/src/state.rs` | `AppState { db, config, limiter }`, `Clone` | |

**Changed.**

| Path | Change |
|---|---|
| `server/src/lib.rs` | Declares `rate_limit` and `state` |
| `server/Cargo.toml` | Dev-dependency `tokio` with the `test-util` feature |
| `Cargo.lock` | That feature |

**How the limiter behaves.**

- One token bucket per `(group, key)`. A bucket holds `capacity` tokens and gets one back every `window / capacity`: 1 s for `Events`, 720 s for `NotifyMe`.
- A request takes one token. With none left, `check` returns `Err(seconds until one is back)`, rounded up, never 0.
- A refused request costs nothing, but it counts as use for eviction. A client that keeps sending stays limited instead of being evicted and starting with a full bucket.
- At most `max_keys` buckets. A new key at the limit evicts the least recently used bucket.
- `by_ip` takes the peer address from the connection, derives the client IP with `client_ip`, and returns `AppError::RateLimited` on refusal. Without a peer address it returns 500: the server must be served with `into_make_service_with_connect_info::<SocketAddr>()` (Prompt 12).

**Differs from the specs.**

- **A bucket is stored as the instant at which it is full again**, not as a token count. Each token taken moves that instant `window / capacity` further away. The behaviour is the continuous refill of §10.6; the arithmetic is on whole nanoseconds, so there is no rounding of fractions of a token.
- **Eviction uses a second index**, a map from the last-used sequence number to the bucket. §10.6 asks for a sequence number per bucket; the index finds the oldest one without reading all 50,000.
- **`max_keys` of 0 is read as 1.**
- **`tokio` has the `test-util` feature in dev-dependencies.** `tokio::time::pause` and `advance`, which the prompt names, need it. It is not in a release build.
- **`by_ip` has no unit test.** It needs an `AppState`, and so a database pool. The sixth-request case of §13.3 covers it in Prompt 13.

**Checked.**

- `cargo test -p offcut-api`: 41 tests pass (11 new), all on a paused clock:
  - Every group has the limit of the TS §22.1 table, and every group accepts exactly its capacity in a burst.
  - The 61st `Events` request in one instant is refused with `Err(1)`.
  - Requests spread over 30 s use the tokens that came back meanwhile.
  - The retry time is rounded up: 720, 720, 719 and 1 at four instants.
  - Tokens return after `advance`; a full bucket does not grow past its capacity.
  - Two IPs, two groups and the three kinds of key do not share a bucket.
  - Eviction keeps the newest keys, by last use and not by first use; both indexes stay at `max_keys` entries.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass.
- No `std::time` clock in `server/src`: the limiter reads `tokio::time::Instant` only. `std::time::Duration` is used; it is not a clock.
- Longest file: `rate_limit.rs`, 384 lines, of which 192 are tests.

**"Done when".** The box is ticked.

---

## 2026-10-07 - Prompt 11: database layer

**Added.**

| Path | Contents |
|---|---|
| `server/migrations/0001_init.sql` | The SQL of TS §23.2, copied by line range from `docs/technicalspec.md`: 8 tables, 5 indexes, the `CHECK` constraints (D-7) |
| `server/src/db/mod.rs` | `connect(database_url)` (a pool of at most 5 connections) and `migrate(pool)` (`sqlx::migrate!("./migrations")`) |
| `server/src/db/analytics_events.rs` | `insert_batch(pool, anon_id, rows)` and `purge_older_than(pool, days)` |
| `server/src/db/platform_waitlist.rs` | `upsert(pool, email_normalized, wanted)`; one unit test |
| `server/.sqlx/` | The offline query cache: three files, one per query |

**Changed.** `server/src/lib.rs`: declares `db`.

**The three queries.** Each is a `sqlx::query!`, checked against the schema when the server is compiled.

- `insert_batch`: one `INSERT ... SELECT ... FROM UNNEST($2::text[], $3::jsonb[])`. It is one statement, so a batch is stored whole or not at all. `ts` is not in the column list; the database sets it to `now()`.
- `purge_older_than`: `DELETE ... WHERE ts < now() - make_interval(days => $1)`; returns the number of rows deleted.
- `upsert`: `INSERT ... ON CONFLICT (email_normalized, wanted) DO NOTHING`.

**Differs from the specs.**

- **`cargo sqlx prepare -- --all-targets` was run from `server/`**, without `--workspace`, as the prompt and G§3.6 say. §10.8 writes `--workspace`, which would put the cache at the repository root.
- **`platform_waitlist.rs` has a private `wanted_text` and one unit test.** The prompt asks for no tests here. `Wanted` has no function that gives its text, so the column value is written out in a match with no wildcard; the test proves each value equals the JSON form of the variant, which is also what the column's `CHECK` allows (D-2).

**Checked.**

- `sqlx migrate run --source server/migrations` on `offcut_dev`, twice: the first run applied `1/migrate init`, the second did nothing. `\dt` shows the 8 tables and `_sqlx_migrations`.
- `\d analytics_events`: the columns are `id, anon_id, name, props, ts`, and `ts` defaults to `now()`.
- `SQLX_OFFLINE=true cargo check -p offcut-api --all-targets` passes. So does `cargo clippy` with `.env` moved away and no `DATABASE_URL` in the environment, which proves the build needs only `server/.sqlx/`.
- There is no `.sqlx` at the repository root.
- The three statements were run by hand in a transaction that was rolled back: a batch of two events gave two rows with the props as sent; of one row dated 91 days ago and one dated 89 days ago, the purge deleted only the first; the second identical waitlist insert added nothing. No row was left in `offcut_dev`.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass (42 server tests).

**"Done when".** Both boxes ticked.

---

## 2026-10-07 - Prompt 12: routes, router and `main`

The API now runs: `GET /healthz`, `POST /events` and `POST /notify-me` answer under `/api/v1`. Closes known issue 17.

**Added.**

| Path | Contents | Tests |
|---|---|---|
| `server/src/auth/mod.rs` | Declares `magic_link` (D-11) | |
| `server/src/auth/magic_link.rs` | `normalize_email(raw)` | 6 |
| `server/src/account/mod.rs` | Declares `notify` | |
| `server/src/account/notify.rs` | The `notify_me` handler | |
| `server/src/analytics/mod.rs` | The `ingest` handler, `within_bounds`, `props` | 3 |
| `server/src/analytics/retention.rs` | `spawn_purge_task(pool)`, `purge_once(pool)` | |
| `server/src/router.rs` | `build(state)`, `healthz`, `not_found`, the route table, the layers | 6 |

**Changed.**

| Path | Change |
|---|---|
| `server/src/main.rs` | The eight steps of §10.12 |
| `server/src/lib.rs` | Declares `account`, `analytics`, `auth`, `router` |
| `server/src/error.rs` | `db_error_kind` is now `pub`, so the purge task and `main` log a database failure the same way a handler does |

**How it behaves.**

- **`normalize_email`.** Trims and lower-cases. `Some` only for 3 to 254 bytes, exactly one `@` with text before it, a dot somewhere after it, and no whitespace or control character.
- **`notify_me`.** A body that does not parse is 400 (413 or 415 where that applies); an address that does not normalize is 400; otherwise the row is inserted if absent and the answer is 204 either way.
- **`ingest`.** The five steps of §10.9. A batch of 0 or more than 50 events, or any event over a cap, is 400 and nothing is stored. The `props` column gets the `props` member of the event's JSON, or `{}`.
- **Retention.** The purge runs once at start and then every 6 hours. The number of days is `offcut_types::ANALYTICS_RETENTION_DAYS`; the number 90 is not typed in the server.
- **Layers**, outermost first: request id, trace, panic to 500, the two headers, media guard, 16 kB body limit; then per route the body limit of the table (1 kB for `/notify-me`) and the rate limit.
- **Anything else** is 404 `not_found`: an unknown path under `/api/v1`, a path outside it, and a known path with the wrong method.
- **`main`.** A configuration error prints `config error: <NAME>` to stderr and exits with 1. A failure to connect, migrate or listen is logged and exits with 1. SIGTERM or Ctrl-C starts a graceful shutdown.

**Known issue 17, closed.** (a) The trace layer is added with `Router::layer`: the log shows `"route":"/api/v1/notify-me"`, and `unmatched` only for a path that matched nothing. (b) The response line is logged at `INFO`. (c) A request id sent by the client is replaced: the id is generated per request, as §10.11 says, and the client's text never reaches the log.

**Differs from the specs.**

- **The route table is in `router.rs` and the modules have no `routes()` function.** §10.9 and §10.10 give `analytics::routes()` and `account::routes()`; §10.11 asks for the table as one block of `.route(...)` calls in `router.rs` with the twelve V6 routes as comments, and the TS §5 tree describes `router.rs` as "route table, body limits, layers" and the two `mod.rs` files as handlers. Both cannot hold. §10.11 and the TS were followed: a route's body limit and rate limit are per route, the rate limit needs the `AppState`, and a module that returned its routes without them would have to be rebuilt in V6, when `account` has three routes with three different limits. In consequence `analytics::ingest` and `account::notify::notify_me` are `pub`. `v1implementation.md` was not edited (known issue 20).
- **The twelve V6 routes are comment lines** giving route, body limit and rate limit group, not commented-out `.route(...)` calls: the handlers they would name do not exist, and their names are V6's to choose.
- **The request id layer is a small function in `router.rs`**, not `tower-http`'s. That one keeps an id the client sent.
- **A wrong method on a known path is 404**, not axum's default 405 with an empty body. The table of §10.11 says "anything else" is `not_found`, and `ApiErrorCode` has no member for 405.
- **The media guard compares the media type without its parameters and without regard to case**: `Application/Octet-Stream; charset=binary` is refused too.
- **`router.rs` and `analytics/mod.rs` have unit tests.** The six layers are in a function that takes any router, so the tests run them around two plain routes with no database.
- **A failed migration is logged with its text**; a failed connection only with its kind. At boot no request exists, and a migration error names a migration, which is what the reader needs.

**Checked.**

- `cargo test -p offcut-api`: 57 tests pass (15 new).
  - `normalize_email`: the six cases of §10.10, 27 inputs in all, including a 254-byte address accepted and a 255-byte one refused, and bytes counted rather than characters.
  - `within_bounds`: each duration at the cap and one over; each of the seven counts over the cap alone.
  - `router`: a media request is 415 on a route and off one; a panic is 500 `internal` with a request id; unknown path and wrong method are 404 with both headers; the request id is new for each request and replaces the client's; a JSON body of exactly 16,384 bytes is accepted and one byte more is 413.
- `env -u DATABASE_URL cargo run -p offcut-api` prints `config error: DATABASE_URL` and exits with 1.
- With the server running against `offcut_dev` (G§3.4 and Milestone 3):

  | Request | Result |
  |---|---|
  | `GET /api/v1/healthz` | 200, `{"ok":true,"version":"dev"}`, `cache-control: no-store`, `x-content-type-options: nosniff`, an `x-request-id`, no `access-control-*` header |
  | `POST /notify-me` with `"  Someone@Example.COM "` | 204; the row is `someone@example.com`, `launch` |
  | `POST /events` with one valid `landing_view` | 204; one row with the props as sent |
  | `POST /events` with `hero_variant: "hello"` | 400 `bad_request`; no row |
  | `POST /events` with `content-type: video/mp4` | 415 `unsupported_media_type` |
  | `GET /api/v1/nope` | 404 `not_found` |
  | `POST /notify-me` without `@`; with a body over 1 kB | 400; 413 |
  | The sixth and seventh `POST /notify-me` | 429, `retry-after: 705`, `"retry_after_secs":705` |

- The valid `/events` request carried `X-Forwarded-For: 198.51.100.77` and `User-Agent: MarkerAgent/9.9`. Neither is in the stored row.
- The server's log of that session holds none of: the email address in either spelling, its domain, the forwarded address, the user agent, `127.0.0.1`, a prop name, the anonymous id.
- The rows written by these checks were deleted from `offcut_dev` afterwards.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` and `check-gen-clean.sh` pass.
- Only `config.rs` reads the environment. Longest new file: `router.rs`, 281 lines.

**Not checked.** The SIGTERM path. It exists only on Unix, and this machine is Windows, where the server stops on Ctrl-C. Prompt 27 runs the image in Docker; check there that `docker stop` ends the process within the grace period (known issue 19).

**"Done when".** Both boxes ticked.

---

## 2026-10-07 - Prompt 13: server integration tests and license policy

Closes known issue 2: `cargo deny check` passes again. The server (Phase C, Prompts 09 to 13) is complete.

**Added.**

| Path | Contents | Tests |
|---|---|---|
| `server/tests/common/mod.rs` | `TestApp { base_url, pool, client }` and `TestApp::spawn()` | |
| `server/tests/analytics_allowlist.rs` | The 14 cases of §13.2 | 14 |
| `server/tests/notify_me.rs` | The 10 cases of §13.3 | 9 |

**Changed.**

| Path | Change |
|---|---|
| `deny.toml` | The license list and the ban wrappers, as described below |
| `Cargo.toml` | `reqwest` is declared with `default-features = false` |
| `Cargo.lock` | 47 fewer packages (287 to 240): the TLS stack that `reqwest` no longer brings in |

**How a test runs.** `TestApp::spawn()` creates a database `offcut_test_<random>` on the server that `DATABASE_URL` names, applies the migrations, and serves the real router on `127.0.0.1` at a free port, with a rate limiter of its own. Each test has its own database, so the tests run in parallel. When the `TestApp` is dropped, its database is dropped.

**`deny.toml`.**

- **Licenses.** The list is now exactly what the dependency tree uses. Removed: `BSD-2-Clause`, which nothing uses. Added: `CDLA-Permissive-2.0`, a permissive license for data; `webpki-roots` (Mozilla's root certificates, through sqlx's rustls) is under it.
- **Bans.** cargo-deny checks every direct parent of a banned crate, third-party crates included. The third-party parents are now listed as wrappers: `reqwest`, `wasm-bindgen-futures`, `web-sys` and `js-sys` for the browser crates (they use them when built for wasm); `uuid`, `rand` and `ring` for `getrandom`; `sqlx-postgres` for `rand`. A workspace crate that is not a binding or renderer crate still fails the check if it depends on any of the six.

**Differs from the specs.**

- **`TestApp` has a private fourth field and a `Drop`.** §13.1 lists three fields. The fourth is the database name; `Drop` removes the database, on a thread with a runtime of its own, because `Drop` cannot await. Without it every run would leave 23 databases behind.
- **`notify_me.rs` has 9 test functions for the 10 cases.** The first valid request, the same request again and the same email with `safari` are one test, because each needs the rows of the one before. One test was added: each of the five `wanted` values is accepted by the column's `CHECK`.
- **The test configuration has `trusted_proxy_hops: 1`.** The local `.env` has 0. With 1 the server reads `X-Forwarded-For`, so the case "a request that carried `X-Forwarded-For`" proves the header was read and still not stored.
- **`reqwest` without its default features.** The tests call `127.0.0.1` over plain HTTP. The defaults brought in a TLS stack with a C library to compile and more licenses to allow. The JSON bodies are written with `serde_json`, so the `json` feature is not needed either.
- **The tests' queries are not `sqlx::query!`.** They are checked when they run. The offline cache in `server/.sqlx/` holds the server's three queries and nothing from the tests.

**Checked.**

- `set -a; . ./.env; set +a; cargo test -p offcut-api`: 57 unit tests and 23 integration tests pass, in parallel, in about 8 seconds.
- `analytics_allowlist.rs`, beyond the letter of the 14 cases:
  - The 19 events are stored in the order sent, each with its name and its props; the two events without props are stored with `{}`. The order of the samples equals the order of `ANALYTICS_EVENT_DOCS`.
  - A free-text prop is added to each of the 19 events in turn: all 19 are rejected.
  - A batch with a bad event in the middle or at the end stores nothing. A batch that parses but breaks a cap stores nothing either.
  - Exactly at the caps (3,600,000 ms; a count of 10,000; 50 events) is accepted.
  - A rejected body is answered with `{"code":"bad_request","retry_after_secs":null}` and nothing of the request.
- `notify_me.rs`: the sixth request is 429, the `Retry-After` header and `retry_after_secs` are equal and between 1 and 720, and `/healthz` still answers for the same IP.
- After the run, no `offcut_test_*` database is left on the server.
- Without `DATABASE_URL`, each integration test fails at once with `DATABASE_URL is not set. Load it first: set -a; . ./.env; set +a`.
- `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`. It still prints warnings: unused wrappers (known issue 6) and ten crates present in two versions.
- Ban drill: `rand` added to `offcut-types` made `cargo deny check bans` fail with `direct parent 'offcut-types' of banned crate 'rand' was not marked as a wrapper`. Reverted.
- `cargo tree -p offcut-types -e normal` and `-p offcut-api-types` show no `rand`, `getrandom`, `wasm-bindgen`, `js-sys` or `web-sys`.
- `cargo sqlx prepare --check -- --all-targets` in `server/`: the cache is up to date.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` and `check-gen-clean.sh` pass.

**"Done when".** Both boxes ticked.

---

## 2026-10-07 - Prompt 14: web tooling and lint boundaries

Closes known issues 12 and 13. The web shell builds, type-checks and lints; it has no application code yet.

**Added.**

| Path | Contents |
|---|---|
| `web/tsconfig.json` | `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `noEmit`, bundler resolution, libs `DOM`, `DOM.Iterable`, `WebWorker`, `ESNext`; includes `src` and the two config files |
| `web/vite.config.ts` | React plugin; dev server with COOP and COEP only and a proxy from `/api/v1` to `http://localhost:8080`; preview headers read from `vercel.json`; `build.target: "esnext"`, `assetsInlineLimit: 0`, `worker.format: "es"` |
| `web/vitest.config.ts` | `environment: "node"`, `include: src/**/*.test.ts` |
| `web/eslint.config.js` | Every rule of TS §7, described below |
| `web/vercel.json` | §12.4, with two placeholder hosts |
| `web/index.html` | `<div id="root">`, one module script, charset and viewport metas, a title; no inline script or style |
| `web/public/robots.txt` | Allow all; no sitemap |
| `web/src/main.tsx` | Temporary: renders an empty `<div>`. Prompt 23 writes the real one |
| `web/.env.local` | `VITE_ASSET_BASE_URL=https://assets.example.invalid`. Ignored by git |

**Changed.**

| Path | Change |
|---|---|
| `web/package.json` | Scripts `dev`, `build:vite`, `preview`; the dependencies below |
| `pnpm-lock.yaml` | The dependency tree |

**Versions installed** (latest on 2026-10-07, except TypeScript):

| Package | Version | Package | Version |
|---|---|---|---|
| `react`, `react-dom` | 19.3.0 | `vite` | 8.3.3 |
| `react-router` | 8.4.0 | `@vitejs/plugin-react` | 6.1.2 |
| `zustand` | 5.0.15 | `vitest` | 5.0.3 |
| `idb` | 8.0.4 | `jsdom` | 30.1.2 |
| `typescript` | 6.0.3 | `@playwright/test` | 1.63.0 |
| `eslint` | 10.12.0 | `typescript-eslint` | 8.71.1 |
| `eslint-plugin-boundaries` | 7.2.0 | `eslint-plugin-react` | 7.37.5 |
| `@types/react`, `@types/react-dom` | 19.3.0 | `@types/dom-webcodecs` | 0.1.19 |

**What `eslint.config.js` enforces.** It lints `src/**/*.ts` and `.tsx`, with type information. `src/gen/`, `src/wasm/pkg/` and `dist/` are not linted.

- **Layers.** Each folder under `src/` is one layer: `ui, usecases, state, persistence, models, net, analytics, workers, wasm, platform, config, copy, gen, entitlement`. An import from one layer to another is refused unless this table allows it. Every layer may import `gen`.

  | Layer | May import |
  |---|---|
  | `ui` | `usecases`, `state`, `copy`, `config` |
  | `usecases` | `state`, `persistence`, `models`, `analytics`, `entitlement`, `platform`; of `workers` only `pool.ts`; of `net` only `api-client.ts`; of the other use-cases only `cancel-job.ts` and `restore-clip.ts` |
  | `models` | of `net` only `asset-fetch.ts`; of `persistence` only `opfs.ts`; of `wasm` only `load-core.ts` |
  | `net` | `config`; the types of `workers/protocol.ts`, never its code (D-8) |
  | `analytics` | of `net` only `api-client.ts` |
  | `workers` | `wasm` |
  | `platform` | of `workers` only `render/encoders.ts` (§11.8) |
  | `copy`, `entitlement` | `config` |
  | `state`, `persistence`, `wasm`, `config`, `gen` | nothing else |

  Files of one layer may import each other, except in `usecases`. A test file may import the file it tests.
- **Network.** `fetch`, `XMLHttpRequest`, `WebSocket`, `EventSource`, `RTCPeerConnection`, the same names on `window`, `self` and `globalThis`, and `navigator.sendBeacon` are refused. `fetch` is allowed in four files: `net/http.ts`, `net/asset-fetch.ts`, `wasm/load-core.ts`, `wasm/load-render.ts`.
- **Syntax.** `import.meta.env` outside `config/env.ts`. A cast to a branded type (`as TimeMs`) outside `workers/`; the 24 brand names are read from `src/gen/domain.ts` when ESLint starts, so the list cannot fall behind. An empty `catch` block or an empty `.catch()` handler outside `analytics/client.ts`; a comment inside does not make it acceptable.
- **Text.** `react/jsx-no-literals` in `src/ui/` and the entry files: no string in JSX children except punctuation. Props are not checked.
- **TypeScript.** No `any`, no non-null `!`, no `@ts-ignore` or `@ts-nocheck` (`@ts-expect-error` with a description is allowed, for type-level tests). A `switch` over a union must name every member. `no-console`.

**Differs from the specs.**

- **TypeScript is 6.0.3, not the latest (7.0.2).** The `typescript` package of version 7 has no compiler API (`createProgram` is absent), and `typescript-eslint` cannot parse a file without it; its supported range ends below 6.1. §3.1 says to pin the latest stable release. Known issue 12 is closed with this version: `tsc` accepts both generated files.
- **`@webgpu/types` was installed and removed.** TypeScript 6.0.3 declares WebGPU itself, and the package's declarations conflict with it (44 errors with library checking on). This is the condition the prompt gives for removing `@types/dom-webcodecs`, applied to the other package. `@types/dom-webcodecs` shows no conflict and stays. A probe file using `navigator.gpu`, `VideoEncoder`, `AudioEncoder` and `VideoDecoder` type-checked without either package.
- **`vercel.json` holds two placeholder hosts**: `render-host.example.invalid` in the rewrite and `https://assets.example.invalid` in the CSP. The prompt says placeholders `<render-host>` and `<ASSET_ORIGIN>`; G§4.2 says to put the host of `VITE_ASSET_BASE_URL` in the CSP, because `check-hosts.mjs` compares the two. The guide was followed. `.invalid` never resolves.
- **The layer table has rows the specs do not give.** TS §2 has no row for `platform`, `wasm`, `config`, `copy`, `gen` or `entitlement`. They were given the smallest set that the files of §11 need.
- **File-level limits.** TS §7 writes `workers/pool`, `net/api-client`, `net/asset-fetch`, `persistence/opfs` and `wasm/load-core`, not the whole layers. The table follows that.
- **`eslint-plugin-boundaries` 7 has a new rule, `boundaries/dependencies`**, with policies and selectors; `element-types`, which older guides show, is deprecated. The new one is used.
- **ESLint's own recommended set is not on.** It lives in `@eslint/js`, which is not in the dependency list of G§4.1. The base is `typescript-eslint`'s `recommendedTypeChecked`, plus the rules above.
- **`tsconfig.json` includes `vite.config.ts` and `vitest.config.ts`**, so they are type-checked, and sets `resolveJsonModule` so the first can import `vercel.json`.
- **The root `package.json` is unchanged.** Its scripts arrive in Prompt 26.

**Checked.**

- `pnpm --filter web exec tsc --noEmit` and `pnpm --filter web exec eslint .` pass. `pnpm install --frozen-lockfile` reports the lock file up to date.
- **The four violations of the prompt, each in a temporary file, each rejected:**

  | Temporary edit | Rule that fired |
  |---|---|
  | `fetch("/x")` in a file under `src/state/` | `no-restricted-globals` |
  | `import.meta.env.MODE` in `src/net/http.ts` | `no-restricted-syntax` |
  | `1000 as TimeMs` in `src/net/api-client.ts` | `no-restricted-syntax` |
  | `import "../net/http"` in a file under `src/state/` | `boundaries/dependencies` |

- **33 more cases, in the same run.** Refused: `ui` importing `net`; a use-case importing another use-case, `net/http.ts`, `workers/protocol.ts` and `workers/render/encoders.ts`; `platform` importing `workers/pool.ts`; a value import of `workers/protocol.ts` from `net`; `XMLHttpRequest` in `net/http.ts` and `WebSocket` in `wasm/load-core.ts`; `window.fetch`; `navigator.sendBeacon`; an empty `catch` holding only a comment; `.catch(() => {})`; `any`; `!`; `@ts-ignore`; `console.log`; a `switch` that misses two members; the text `Hello world` in JSX. Accepted: `fetch` in `net/http.ts` and `wasm/load-core.ts`; `import.meta.env` in `config/env.ts`; a brand cast in `workers/protocol.ts`; a type import of `workers/protocol.ts` from `net`; a use-case importing `cancel-job.ts`, `workers/pool.ts` and `net/api-client.ts`; `platform` importing `workers/render/encoders.ts`; a test importing the file it tests; an empty `catch` in `analytics/client.ts`; `{", "}` and `className="x"` in JSX; `typeof fetch` as a type in a test file.
- Every temporary file was deleted; `src/` holds `gen/` and `main.tsx`.
- `pnpm run build:vite` writes `dist/index.html`, one script of 219.54 kB (68.56 kB gzip) and `robots.txt`. `dist/index.html` has no inline script.
- `vite preview` answers `/` with all seven headers of the `/(.*)` rule of `vercel.json`, value for value. `vite dev` answers with COOP and COEP and no CSP. With no API running, `/api/v1/healthz` through the dev server is 502: the proxy is active.
- `web/dist`, `web/.env.local` and `web/node_modules` are ignored by git.

**Not done, as the prompt orders.** `playwright.config.ts` (Prompt 25). `vitest run` exits with 1 until the first test file exists (Prompt 16).

**"Done when".** The box is ticked.

---

## 2026-10-07 - Prompt 15: web foundation

**Added.**

| Path | Contents |
|---|---|
| `web/src/config/env.ts` | `env { assetBaseUrl, dev }`. The only reader of `import.meta.env` |
| `web/src/config/allowlist-hosts.ts` | `allowedHosts()`, `isAllowedUrl(url)` |
| `web/src/workers/protocol.ts` | The 13 types of TS §14.1 and §14.2, and the re-export of `FailureStage`. No function, no constant |
| `web/src/state/machines/transition.ts` | `MachineDef`, `IllegalTransitionError`, `setIllegalTransitionReporter`, `transition` |
| `web/src/state/capability-store.ts` | `CapabilityReport`, `CapabilityStatus`, `CapabilityState`, `useCapabilityStore`, `beginCheck`, `setSupported`, `setUnsupported` |
| `web/src/persistence/schema.ts` | `DB_NAME`, `DB_VERSION`, `STORES` (all eight, D-6), `META_KEYS`, `Versioned<T>`, `OffcutDb`, and one record type per store |
| `web/src/persistence/db.ts` | `openDb()`, `metaGet(key)`, `metaSet(key, value)` |

No file outside `web/src/` changed.

**How each file behaves.**

- **`env.ts`.** Throws when the module loads if `VITE_ASSET_BASE_URL` is missing, empty, not a URL, not `https://`, or has a query string or a fragment. Trailing slashes are removed, so `assetBaseUrl + "/" + path` has one slash.
- **`allowlist-hosts.ts`.** The two hosts are the page's own and the asset host. `isAllowedUrl` reads a URL relative to the page; a `data:` or `blob:` URL, a protocol-relative URL to another host, and a host that merely starts with the asset host are all refused.
- **`transition.ts`.** A pair missing from a machine's definition is illegal. With no reporter set, an illegal attempt throws `IllegalTransitionError`. Once `setIllegalTransitionReporter` has been called, it calls the reporter and returns the state unchanged.
- **`capability-store.ts`.** Three legal transitions: `unchecked` to `checking`, and `checking` to `supported` or to `unsupported`. The reason and the report are stored only when the transition was legal. `unsupported` accepts nothing more.
- **`db.ts`.** `openDb` opens the database once and returns the same promise afterwards; a failed attempt is forgotten, so a later call tries again. Upgrade 1 creates the eight stores. `metaGet` returns `undefined` for a missing key, and for a value whose `schemaVersion` is above `DB_VERSION`, which it also deletes (TS §23.3).

**Differs from the specs.**

- **`transition()` decides between throwing and reporting by whether a reporter is set**, not by reading the build mode. §11.4 says development and test builds throw and production builds report, but `state/` may import only `gen`, and `import.meta.env` may be read only in `config/env.ts`. So the mode is handed in the same way the reporter is: `start-app.ts` must call `setIllegalTransitionReporter` in production builds only (known issue 29).
- **`schema.ts` exports six record types** (`ClipRecord`, `TranscriptRecord`, `EditsRecord`, `RenderCacheRecord`, `EntitlementRecord`, `ReceiptOutboxRecord`) beside the names §11.5 lists. They are the value types of `OffcutDb`, named so the repositories of V3 need not spell them again.
- **Choices TS §23.1 leaves open.** A time (`createdAt`, `storedAt` and so on) is a plain number of milliseconds since the Unix epoch. Keys are given on every write, not read from the value, because every value is wrapped in `Versioned`. The `schemaVersion` written is `DB_VERSION`.
- **`env.ts` also refuses a fragment** (`#`), not only a query string, and strips trailing slashes.
- **`db.ts` closes its connection when a newer build in another tab asks to upgrade**, and forgets a connection the browser terminated. Neither is in §11.5; without the first, a version 2 in one tab would wait for every other tab to be closed.

**Checked.**

- `pnpm --filter web exec tsc --noEmit` and `pnpm --filter web exec eslint .` pass. The lint run covers the layer rules: `state/` imports only `gen/`, `zustand` and its own `machines/transition.ts`; `persistence/` imports only `gen/` and `idb`; `protocol.ts` imports only `gen/`.
- **A throwaway test file, run once and deleted** (the prompt allows no new test file): 15 cases, all passing.
  - `env`: two good values; seven bad ones each throw an error naming the variable.
  - `allowlist-hosts`: seven URLs, the five refusals listed above among them.
  - Capability store: the two legal paths; all six illegal attempts throw and leave the stored reason untouched; with a reporter set, an illegal attempt is reported as `("capability", "unchecked", "pass")` and the state stays `unchecked`.
  - `db`, against `fake-indexeddb` installed outside the repository: the database has version 1 and exactly the eight stores; `openDb` returns the same connection twice; a value round-trips and is stored as `{ schemaVersion: 1, value }`; a value with `schemaVersion: 2` reads as absent and is gone afterwards.
- There is no `*.test.ts` file under `web/src/`.

**"Done when".** The box is ticked.

---

## 2026-10-07 - Prompt 16: network layer

Closes known issue 14: `http.ts` reads a missing `retry_after_secs` as `null`.

**Added.**

| Path | Contents | Tests |
|---|---|---|
| `web/src/net/http.ts` | The three constants (10 s, 70 s, 3 s), `HttpRequest`, `HttpResult`, `HttpDeps`, `createHttp(deps)`, `http`, `toAppFailure` | |
| `web/src/net/http.test.ts` | The 12 cases of §13.5 and 9 more | 21 |
| `web/src/net/api-client.ts` | `wake()`, `postEvents(batch)`, `postNotifyMe(req, onWaking)` | |
| `web/src/net/asset-fetch.ts` | `assetUrl(path)`; no `fetch` yet | |

**Changed.** `web/vitest.config.ts`: `test.env` sets `VITE_ASSET_BASE_URL`. `config/env.ts` refuses to load without it, and `web/.env.local` is not in the repository, so CI would have had no value.

**How a request behaves.**

| Situation | Result |
|---|---|
| The browser reports it is offline | `E_NET_OFFLINE` at once; `fetch` is not called |
| `fetch` rejects with a `TypeError` | `E_NET_OFFLINE`; not tried again |
| No complete response within 10 s, or status 502, 503, 504 | `retry: "cold-start"`: tried again; finally `E_NET_TIMEOUT`. `retry: "none"`: `E_NET_TIMEOUT` after one attempt |
| Another 5xx | Tried again in cold-start mode; finally `E_API_5XX` |
| 429 | `E_API_RATE_LIMITED` with the `apiError`; not tried again |
| Another 4xx | `E_INTERNAL`, not retryable, with the `apiError`; not tried again |
| 2xx | `ok: true`; `data` is the parsed JSON, or `undefined` when the body is empty |

- **Retries.** The waits are 1, 2, 4, 8 s and then 8 s each time. Another attempt is made only if it would start before 70 s have passed since the first one started; the timeout of an attempt is cut to the time that is left.
- **The waking notice.** `onWaking` is called once, 3 s after the first attempt started, if the request is still going then, and never afterwards.
- **The request.** The URL is `/api/v1` plus the path, never absolute. `credentials: "same-origin"`, `cache: "no-store"`; the JSON content type is sent with a body and only with a body.
- **`failure`.** `stage` is always `"api"`. `detail` is set in development builds only and holds a fixed word or the status number, nothing from the request.
- **`assetUrl`.** The base URL, a slash, the path. A path containing `?` or `..` throws.

**Differs from the specs.**

- **The 10 s timeout covers reading the body**, not only waiting for the headers. A response that started and then stalled would otherwise never end.
- **An `apiError` is accepted only if its `code` is one of the 13 the server defines.** The list in `http.ts` is typed `Record<ApiErrorCode, true>`, so `tsc` fails if the generated union gains or loses a member.
- **A 429 with no readable body takes its wait from the `Retry-After` header**, which the server also sets.
- **A 2xx with an empty body gives `undefined` whatever its status**, not only 204. §11.6 names 204; the V6 route that answers 202 has no body either.
- **A 2xx whose body is not JSON is `E_INTERNAL`.** §11.6 does not say.
- **A request the caller aborts through `signal` ends with `E_INTERNAL`, not retryable, and is not tried again.** §11.6 gives the field and no outcome for it. No V1 caller passes a signal.
- **`assetUrl` drops a leading slash from the path**, so `assetUrl("/media/x")` does not produce a double slash.
- **`http.ts` does not import `config/allowlist-hosts.ts`**, which the TS §7 graph shows. It builds no absolute URL, so there is no host to check.

**Checked.**

- `pnpm --filter web exec vitest run`: 21 tests pass, on a fake clock; none waits in real time. Also with `web/.env.local` moved away.
  - The 12 cases of §13.5.
  - All timing out: attempts start at 0, 11, 23, 37 and 55 s, and the request fails at 65 s, inside the 70 s budget. Every timed-out attempt was aborted.
  - 500 every time: attempts at 0, 1, 3, 7, 15, 23, 31, 39, 47, 55 and 63 s, then `E_API_5XX`.
  - More than §13.5: 502 and 504 as well as 503; a 500 with `retry: "none"`; the `Retry-After` fallback; six bodies that are not an `ApiError`, each leaving `apiError` undefined; a caller's abort; the notice given at exactly 3,000 ms for a slow answer; the table of `toAppFailure`.
- **The tests were shown to fail when the code is wrong.** Four temporary changes to `http.ts`, each reverted: the notice given without checking that the request is still going (1 test failed); a third wait of 3 s (2 failed); a 429 tried again (1 failed); an absolute URL (1 failed).
- `assetUrl`, in a throwaway test that was deleted: two good paths, four refused.
- `tsc --noEmit` and `eslint .` pass. `fetch` appears in `http.ts` only; `asset-fetch.ts` has none.
- Longest file: `http.test.ts`, 361 lines; `http.ts` has 239.

**"Done when".** The box is ticked.

---

## 2026-10-07 - Prompt 17: analytics client

**Added.** `web/src/analytics/client.ts` (64 lines): `FLUSH_INTERVAL_MS` (10,000), `FLUSH_AT` (20), `initAnalytics({ anonId })`, `track(event)`, `flush()`.

**How it behaves.**

- `track` puts an event on a queue. It takes only the generated `AnalyticsEvent` union. It never throws and never waits.
- Before `initAnalytics`, events wait in the queue and `flush` does nothing.
- `initAnalytics` stores the anonymous id and starts the flushes. A second call does nothing.
- A flush happens every 10 s, when 20 events are queued, and when the page becomes hidden.
- One flush sends at most `MAX_EVENTS_PER_BATCH` (50) events, the oldest first, through `postEvents`. The rest wait for the next flush.
- The events of a flush that fails are dropped. `flush` never rejects.
- The file imports `gen/` and `net/api-client.ts`, and nothing else.

**Differs from the specs.**

- **The `catch` in `flush` holds only a comment.** It is the one swallowed failure TS §11.3 allows, and the lint rule exempts this file alone. `postEvents` returns its failures instead of throwing them, so the `catch` is a second guard: it keeps `flush` from rejecting whatever a later change to `postEvents` does.

**Checked.**

- `tsc --noEmit` and `eslint .` pass.
- **The call the prompt names was rejected by `tsc`**, in a temporary file: `track({ name: "landing_view", props: { hero_variant: "outcome", extra: 1 } })` gives `TS2353: 'extra' does not exist in type ...`. Five more calls in the same file were each rejected too: an event name that is not on the allowlist; free text where an enum is expected; a `props` object on an event that has none; text where a number is expected; a missing property.
- **The layer rule was shown to bite**, in a temporary file under `analytics/`: an import from `state/`, from `persistence/`, from `net/http.ts` and from `config/` were each refused by `boundaries/dependencies`.
- **Behaviour, in a throwaway test with a fake `postEvents` and fake timers** (6 cases, all passing): an event tracked before `initAnalytics` is sent on the first 10 s tick with the anonymous id, and not at 9,999 ms; the twentieth event triggers a flush; of 70 queued events the first flush sends 50 and the next 20; a flush happens on `visibilitychange` to hidden and not to visible; a flush that fails or throws resolves and its events are not sent again; a second `initAnalytics` does not change the id.
- All three temporary files were deleted. `http.test.ts` is still the only test file under `web/src/`, and its 21 tests pass.

**"Done when".** The box is ticked.

---

## 2026-10-07 - Prompt 18: capability check and encoder constants

Closes known issue 26. Phase D (web foundation and capability check, Prompts 14 to 18) is complete: Milestone 4 of the guide passes.

**Added.**

| Path | Contents | Tests |
|---|---|---|
| `web/src/platform/simd-probe.ts` | `SIMD_PROBE_BYTES`, the 31 bytes of G§4.4 | |
| `web/src/workers/render/encoders.ts` | `VideoLadderEntry`, `VIDEO_ENCODE_LADDER` (four entries), `AAC_ENCODE_CONFIG`, `KEYFRAME_INTERVAL_FRAMES`, `CREATOR_VIDEO_BITRATE`, `videoConfigFor` | |
| `web/src/platform/capability.ts` | `PER_CHECK_TIMEOUT_MS`, `H264_DECODE_PROBE`, `AAC_DECODE_PROBE`, `PlatformProbe`, `browserProbe`, `runCapabilityCheck` | |
| `web/src/platform/capability.test.ts` | The 11 groups of §13.4 and one more | 54 |

**How the check behaves.**

- All eleven checks start together. Each has a timeout of 1 s; a check that throws, rejects or does not answer in time has failed. The whole check therefore ends after about 1 s at most, inside the 3 s budget.
- The reason reported is the first failed check in the order of the generated `UNSUPPORTED_REASONS` list, which is the order of TS §13.2. The later checks still run, so the report is complete.
- A browser that does not report its memory passes. Below `LIMITS.MIN_DEVICE_MEMORY_GB` fails.
- The report holds four enum values, and `unsupported_reason` only when a check failed; on a pass the key is absent, not `null`. This is the form known issue 15 asks of the client.
- `gpu_vendor`, `memory_bucket` and `platform` are mapped as the table of §11.8 says. No string the browser gives reaches the report.
- `browserProbe` reads `navigator.userAgentData` (`mobile`, `platform`), `navigator.deviceMemory`, the GPU adapter's vendor, and the yes-or-no answers of the other checks. Nothing else.

**Differs from the specs.**

- **`encoders.ts` exports `CREATOR_VIDEO_BITRATE` (8,000,000 b/s)**, which the list of §11.8 does not have. Check 9 asks whether the encoder supports 1080x1920, and a `VideoEncoderConfig` needs a bitrate. The value is in the configuration table of TS §21.2, which belongs to this file; and a `BitsPerSec` can only be made in `gen/` or `workers/`, because the lint rule refuses the cast elsewhere.
- **The storage check opens and deletes a database of its own**, `offcut-capability-probe`. Opening the app's database there, without a version, would create it empty at version 1, and `openDb` would then find no stores and run no upgrade.
- **`capability.ts` declares the two navigator fields itself** (`userAgentData`, `deviceMemory`): TypeScript 6.0.3 has neither. This was known issue 26.
- **`capability.ts` has its own alias for `CapabilityReport`**, the same generated type that `state/capability-store.ts` exports under that name. `platform` may not import `state`.
- **A mobile hint that cannot be read is a failed check**, like any other check that throws. `undefined`, which is what a browser without the API gives, passes.
- **The ladder entries are probed together**, not one after another, so check 9 also stays inside its second.
- **The test file has a twelfth group**: the SIMD bytes are a valid module in Node, and the ladder and the AAC configuration equal TS §21.2. A typing mistake in either would otherwise show only in a browser.

**Checked.**

- `pnpm --filter web exec vitest run`: 75 tests pass (21 in `http.test.ts`, 54 here).
  - Each of the 11 reasons alone; all 55 pairs, each reporting the earlier one; all eleven failing at once.
  - A check that never answers has failed at exactly 1,000 ms and not at 999; six hanging checks end after one second, not six; a browser that answers at once leaves no timer behind.
  - A check that throws at once, one that rejects, every probe function throwing, and one that rejects two seconds after its timeout: no unhandled rejection.
  - Memory 2, 3.9, 4, 7.9, 8 and 64 GB and unreported; nine vendor strings and no adapter; nine platform hints and none; the mobile hint true, false and absent.
  - Report shape: 126 combinations of odd vendor and platform strings, memory values and mobile hints, every value a member of its enum.
- **The tests were shown to fail when the code is wrong.** Four temporary changes to `capability.ts`, each reverted: the last failed check reported (4 tests failed); the timeout removed (2 failed); unreported memory made a failure (1 failed); the raw vendor string put in the report (11 failed).
- **`browserProbe` in real Chrome**, through a temporary page on the dev server, headless and headed, then deleted: every one of the twelve functions answered without throwing; the report was `pass`, `intel`, `gb8plus`, `windows`; the check took 139 ms; the probe database was gone afterwards.
- `tsc --noEmit` and `eslint .` pass. The lint run covers the layers: `platform/` imports `gen/` and `workers/render/encoders.ts` only.
- Milestone 4: `tsc`, ESLint and Vitest are green, and the four deliberate lint violations were rejected in Prompt 14.

**"Done when".** The box is ticked.
