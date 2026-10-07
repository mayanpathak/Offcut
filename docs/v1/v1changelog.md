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
| 2 | **`cargo deny check` fails since Prompt 09** (bans and licenses), as predicted: the server's dependencies bring in the banned crates through third parties. Prompt 13 is where the specs settle `deny.toml`. Needed there: (a) wrappers: `getrandom` is used by `uuid`, `rand`, `ring`, `jobserver`; `rand` by `sqlx-postgres`; `js-sys`, `web-sys` and `wasm-bindgen` by `reqwest`, `wasm-bindgen-futures` and each other; (b) the license allowlist, from the first full run. Until then the pure crates are checked by hand: `cargo tree -p offcut-types -e normal` and `-p offcut-api-types` show no `rand` or `getrandom` | Prompt 13 |
| 4 | CI must install binaryen `version_133` and `wasm-bindgen-cli` 0.2.129, the versions used locally | Prompt 26 |
| 5 | `pnpm` is pinned at 10.15.0, the installed version named in G§1.4. pnpm reports 12.9.1 as available, and §3.1 says to pin the latest stable release. Not changed; decide before CI is written | Prompt 26 |
| 6 | `cargo deny check` prints `unused-wrapper` warnings for the media and renderer crates, which do not exist until V2. They are warnings, not errors | Every `cargo deny check` |
| 7 | `ts-rs` runs with `no-serde-warnings`, and Cargo applies that feature to every crate in the workspace. `ts-rs` will therefore stay silent about any serde attribute it cannot read. The JSON-shape tests are the guard for any type added later | Any new shared type |
| 9 | Choices in the two type crates go beyond the letter of §6, §7 and §9. They are listed under "Differs from the specs" in the entries of Prompts 03 to 08. `v1implementation.md` has not been edited to match; reconcile it in the exit audit | Prompt 30 |
| 11 | `crates/offcut-types/tests/ui/bare_ms_rejected.stderr` holds compiler output of Rust 1.99.0. Regenerate and re-read it whenever `rust-toolchain.toml` changes; the command is in `tests/ui.rs` | Any toolchain change |
| 12 | `web/src/gen/*.ts` were type-checked with TypeScript 7.0.2 from a throwaway install outside the repository. TypeScript becomes a repository dependency in Prompt 14; run `tsc` on the two files again there, with the version that gets pinned | Prompt 14 |
| 13 | The generated files are not formatted for a linter: `ts-rs` writes a type on one line, with a comma after the last field, and some lines are over 500 characters long. ESLint must either skip `web/src/gen/` or have no style rule that rejects this | Prompt 14 |
| 14 | In `gen/api.ts`, `ApiError.retry_after_secs` is `number \| null`, and the server writes `null` when there is no retry time. `http.ts` must treat `null` and a missing field the same way | Prompt 16 |
| 15 | In `gen/api.ts`, an optional event prop is typed `prop?: T \| null` (for example `unsupported_reason`). The client should leave an absent prop out; the server accepts `null` too and stores neither | Prompts 17, 18 |
| 17 | Three things `router.rs` must do for `log.rs` to work as specified. (a) Add the trace layer with `Router::layer`, or the route template is not yet known and every request logs `unmatched`. (b) Set the trace layer's response logging to `INFO`; its default is `DEBUG`, and with `LOG_LEVEL=info` no status or latency would be logged. (c) Decide what to do with an `X-Request-Id` sent by the client: `tower-http` keeps it. `log.rs` already refuses to log one that is not a plain id | Prompt 12 |
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
