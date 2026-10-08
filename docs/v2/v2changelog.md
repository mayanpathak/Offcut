# v2changelog.md - Offcut V2: record of changes

**What this is.** A record of every change made while building V2: what was done, what differs from the plan or the guide, how it was checked, and what is still open. `v2implementation.md` says what each file contains; `v2buildguide.md` gives the order of work; `coding-promptsv2.md` holds the prompts (31 to 60). This file says what happened.

**How it is kept.** One entry per prompt, or per change made outside a prompt. Entries are in date order, oldest first; new entries go at the end. Every commit that changes the repository carries its entry. The two tables below are rewritten when an item opens or closes. What happened before Prompt 31 (the V2 documents, the reference clip, the decision on R2) is in `docs/v1/v1changelog.md`, entries of 2026-10-08.

**References.** `§n` is a section of `v2implementation.md`, `G n.m` a step of `v2buildguide.md`, `D-n` a decision of §2, `TS §n` of `technicalspec.md`, `V1 item n` an open item or known issue of `v1changelog.md`.

**The baseline.** V2 starts from `main` at `322c7d307e86cb21ee0a9af341ce2d1657f66b04`, the deployed V1. Every frozen-file check compares with that commit. The test counts there: 170 Rust tests, 76 Vitest tests, 12 Playwright cases. No count may fall.

---

## Open items

| # | Item | Who | Needed by |
|---|---|---|---|
| 1 | Add the origin `http://localhost:4173` to the CORS policy of the asset bucket (D-41). On 2026-10-08 the bucket answers for the app origin and for `http://localhost:5173` only | Human | Prompt 44 |
| 2 | Decide whether to replace `ENTITLEMENT_SIGNING_KEY` in Render: its private half was shown in a chat on 2026-10-08 (`v1changelog.md`). Prompt 33 wrote the public half of the present key into `web/src/config/entitlement-public-key.ts`. If the key is replaced, that literal and §1A item 12 must be derived again in the same change; a token signed with the new key is otherwise refused by every browser. Nothing is signed before V6 | Human | Before V6; sooner is cheaper |
| 3 | R1, a 2021-class Windows laptop with 8 GB and an integrated GPU: book it for the days of Prompts 44, 51 and 59, and set it up once (G 0.6: Node, pnpm, Chrome, Python 3 with NumPy, `ffmpeg`; a clone; the reference clip copied by hand). Boxes 13 and 14 of §1A stay open until then. Both gates are read on R1 and on no other machine | Human | Prompt 44 |
| 4 | After every prompt marked **Push**: `git push` from `offcut/` (the repository is not the `sh2clips` folder), then read the `ci` run of pull request #2. The next prompt waits for green | Human | Prompts 35, 37, 41, 44, ... |
| 5 | Submit the merchant onboarding (TE-9) if it is not submitted. Approval can take two weeks | Human | V6 |
| 6 | Reset the Neon password (V1 item 36) and replace the demo video (V1 item 33) | Human | Before the page is announced |
| 7 | The file `.env.local` in the repository root holds one line with no name, a test-mode API key. Git ignores the file and no program reads that line. Move it into `.env.deploy` under a name | Human | Any time |
| 8 | Turn on branch protection for `main` on GitHub (Settings, Branches): require a pull request and the `ci` check. `main` is unprotected, and a push to it deploys | Human | Before Prompt 59 |

## Known issues for later prompts

| # | Issue | Affects |
|---|---|---|
| 1 | A `src/lib.rs` of 0 bytes fails `cargo fmt --check`: rustfmt wants one line break. The eight skeletons hold one line break (LF) | Any prompt that creates an empty Rust file |
| 2 | `coding-promptsv2.md` does not say which branch Prompts 59 and 60 work on after the merge. On `main`, the line `BASE=$(git merge-base main HEAD)` of the Standard Agent Block gives `HEAD`, and the frozen-file diff then compares nothing. From the merge on, use the baseline SHA above in place of `$BASE`, and put the commits made after the merge on a branch of their own | Prompts 59, 60 |
| 3 | `cargo deny check` passes with warnings, as in V1 (V1 item 6): `unused-wrapper` for crates that are not dependencies yet, and `duplicate`. They are warnings, not errors | Prompts 34 to 48 |
| 4 | **The one cast each minting file may hold is tied to a name** in `web/eslint.config.js` (`MINTING`). It must be the last `return` of a function declaration with that name: `toBytes` in `persistence/opfs.ts`, `models/download.ts` and `models/model-manager.ts`; `newClipId` in `usecases/import-clip.ts`; `newExportId` in `usecases/start-export.ts`; `toTimeMs` in `usecases/control-preview.ts`. In `state/model-store.ts` it is the top-level line `const ZERO_BYTES = 0 as Bytes;`, not exported. An arrow function, another name or a cast anywhere else in the file fails lint | Prompts 37, 39, 40, 53, 54 |
| 5 | `boundaries/dependencies` reports an import only when the imported file exists. An import of a file that is not written yet is reported by `tsc`, and by another ESLint rule, not as a layer violation. A drill on a layer edge needs its target file | Any lint drill |
| 6 | `web/vite.config.ts` and `web/vitest.config.ts` are type-checked by `web/tests-e2e/tsconfig.json`, with Node's types, and no longer by `web/tsconfig.json` (entry of Prompt 32). Prompt 43 adds a build-start step to `vite.config.ts` that uses Node's file API: it type-checks there without a further change | Prompt 43 |
| 7 | No ESLint rule covers `vite.config.ts`, `vitest.config.ts` or `playwright.config.ts`, as in V1. They are type-checked only | Prompts 43, 56 |
| 8 | **The secret scan of CI (gitleaks 8.18.4) takes a 44-character base64 literal assigned to a name with `KEY` in it for a secret.** Found in Prompt 33 with the same version run locally. Such a line must end with a `gitleaks:allow` comment **in the commit that first adds it**: the scan reads the whole history, so a comment added in a later commit does not clear the finding. The test public key (`fake-api.ts` in Prompt 46, `ci.yml` in Prompt 58) is such a literal. Before a commit that adds one, fetch the release archive of that version, check its SHA-256 against the published checksum file, and run `gitleaks detect --no-git --source <folder> --redact` | Prompts 46, 58 |
| 9 | Chrome under Playwright, headless, gives a module worker WebGPU, both encoders and a sync OPFS handle on the development machine (Prompt 33). The adapter is the integrated Intel GPU, not the GTX 1650. No headed run and no launch argument was needed | Every browser check |
| 10 | **`crates/offcut-mp4/src/boxes.rs` has 392 lines above its test module and `sample_table.rs` 387; the limit is 400** (`cargo fmt` puts most signatures and struct literals on several lines). Prompt 35 writes `demux.rs`, `probe.rs` and `validate.rs` and should add nothing to these two. Code that must go into one of them needs room made first | Prompts 35, 41 |
| 11 | **What Prompt 35 builds on in `offcut-mp4`** (entry of Prompt 34). To reach a box: `children`, `child`, `descend(r, top, path)`; the whole file can be given as a `BoxHeader` whose `body` is 0 and whose `end` is the file length. `read_stsd(r, h, handler)` needs the handler type from `read_hdlr`. `read_stsz(r, h, MAX_RESOLVED_SAMPLES)` returns the count always and the size table only for a track that will be resolved; for a longer track the other per-sample tables (`ctts`, `stsc`, `stco`/`co64`, `stss`) need not be read, and `resolve` does not look at them. `stts` and `elst` are always needed. `RawTables.movie_timescale` is from `mvhd`. `SampleEntry.object_type` is the object type indication of `esds` (0x40 AAC; 0x69, 0x6B MP3). A `tkhd` matrix is the nine stored values: 65536 is 1.0 | Prompt 35 |
| 12 | `proptest` writes a folder `proptest-regressions/` beside the crate when a property fails. It is not in the tree of TS §5: delete it once the failure is fixed, or the file-tree check fails | Prompts 35, 36, 41 |

---

## 2026-10-08 - Prompt 31: preflight, branch, V2 documents, D-65, crate skeletons

**Branch.** `v2-build`, created from `main` at the baseline. The pending documents were committed first and alone, as `V2: documents` (`28d010e`): `docs/v2/v2implementation.md`, `v2implementation-notes.md`, `v2buildguide.md`, `coding-promptsv2.md`, `docs/v1/v1changelog.md` and `fixtures/speech/README.md`. `main` is untouched: a push to it deploys.

**Tools (the audit of G 0.1).**

| Tool | Version |
|---|---|
| `bash` | `/usr/bin/bash` (Git Bash) |
| Git | 2.48.1.windows.1 |
| Rust, Cargo | 1.99.0, with the target `wasm32-unknown-unknown` |
| `wasm-bindgen` CLI | 0.2.129 |
| `wasm-opt` | version 133 |
| `cargo-deny` | 0.20.2 |
| Node, pnpm | 24.19.0, 10.15.0 |
| Python, NumPy | 3.13.5, 2.5.3 (with the Python 3.13 folder first in PATH) |
| `ffmpeg`, `ffprobe` | 9.0.2 |
| TypeScript, Vite, Playwright | 6.0.3, 8.3.3, 1.63.0 |
| Docker, `psql` | 28.3.2, 18.4 |
| `gh` | Not installed (open item 4) |

**The baseline gate, on the branch before any V2 change.** `pnpm install --frozen-lockfile`, then `pnpm check`, `pnpm test`, `pnpm build` and `pnpm e2e`: all green. 170 Rust tests, 76 Vitest tests, 12 Playwright cases.

**The preconditions of §1A.**

| # | Checked how | Result |
|---|---|---|
| 1 | A temporary Playwright case on the deployed page: `fetch` of an uploaded file with `Range: bytes=0-1023`. Also `curl` from the terminal | 206, `Content-Range: bytes 0-1023/10376359` |
| 2 | `E2E_BASE_URL=https://offcut-one.vercel.app`, the `@smoke` case | 1 passed |
| 3 | The same temporary case: `crossOriginIsolated` | `true` |
| 4 | `pnpm -C web exec tsc --noEmit` | No output |
| 5 | `grep` for the four names in `encoders.ts`; the imports of `capability.ts` | 4 found; `capability.ts` imports `VIDEO_ENCODE_LADDER` and `AAC_ENCODE_CONFIG` |
| 6 | `pnpm e2e` locally; the last `ci` run on `main` | 12 passed; green on `322c7d3` |
| 7 | "twelve thousand dollars" in `fixtures/speech/README.md` | Found (2 lines). Prompt 43 reads how the recognizer writes it |
| 8 | `ls bench/results` | `.gitkeep` |
| 9 | `pnpm check` | Green |
| 10 | The four-command gate; `ci` on `main` | Green; green |
| 11 | The same temporary case: the IndexedDB database of the deployed page | `offcut`, version 1, eight stores, `entitlement` among them |
| 12 | The base64 value of §1A item 12 | Decodes to 32 bytes. Whether the key in Render was replaced since is open item 2 |
| 13 | - | **Open**, the human's (open item 3) |
| 14 | The audit above | Holds on the development machine. **Open** for R1 (open item 3) |

Also checked: `/api/v1/healthz` on Render and through Vercel both report `322c7d3`; `node scripts/check-headers.mjs https://offcut-one.vercel.app` passes, seven headers value for value. The `curl` with `Origin: http://localhost:4173` gets no `Access-Control-Allow-Origin` (open item 1).

Boxes 1 to 12 of §1A are ticked in the plan. Boxes 13 and 14 are not.

**Added.**

| File | Content |
|---|---|
| `crates/offcut-{mp4,dsp,text,detect,entitlement,scene,render,wasm-render}/Cargo.toml` | The `[package]` header of `offcut-types` and `[lints] workspace = true`. No dependency. `offcut-wasm-render` is a `cdylib`; `offcut-entitlement` declares the feature `sign`, empty until V6 |
| The eight `src/lib.rs` | One line break each (known issue 1) |
| `docs/v2/experiments.md` | The header, the form of an entry, and the list of the experiments V2 runs, none run yet |
| `docs/v2/v2changelog.md` | This file |

**Changed.**

| File | Change |
|---|---|
| `Cargo.toml` | The eight new members; the list is now one path per line. 12 members |
| `Cargo.lock` | Eight package entries, no dependency |
| `docs/technicalspec.md` §5 | The tree gains `crates/offcut-mp4/src/mux_boxes.rs` (D-55), `web/tests-e2e/tsconfig.json` (D-63) and the six files of `docs/v2/` (D-46) |
| `docs/v2/v2implementation.md` | D-65 added to §2: R2 is not measured in V2 and R1 is the only reference machine, with the reason and the cost. §0 says "D-18 to D-65". D-46 names all six files of `docs/v2/`, and §4 says "six". Every line that named R2 is reworded for R1 (list below). Boxes 1 to 12 of §1A ticked |

The lines reworded for R1: §1A boxes 13 and 14 and the row for V1 known issue 30; §3.1 (two rows); §3.4 (`BENCH_DEVICE`); §5 (S15); §22.2 (`e2e:device`); §23.1 (two rows); §24.1 (E-3, TE-3, TE-4, E-4; TE-4 now passes when a ladder entry is supported on R1); §25.3 (one box); §26 (three boxes). The edit was made by a script that refuses a replacement unless its old text occurs exactly once: 35 of 35 applied. The file keeps its CRLF line endings.

**Differs from the prompt or the guide.**

- **More R2 lines than the prompt lists.** Prompt 31 and G 1.1 name §1A, §5, §23.1, §24.1, §25.3 and §26. The plan also named R2 in the carried-over table of §1A, in §3.1, §3.4 and §22.2. Those five lines were reworded too; without them the prompt's own check (`grep -nw "R2"`) could not pass.
- **`lib.rs` is not 0 bytes** (known issue 1).
- **The documents were committed on the branch, not on `main`.** G 0.2 commits them on `main` first. The prompt does it this way, and it keeps `main` at the deployed commit. The empty commit "V2 S0" of the guide's Milestone 0 was not made: the commit `V2: documents` marks where V2 starts.
- **The test key pair of G 0.5 was not generated.** Prompt 46 generates it where it is first written down (the prompts file says so).
- `v2implementation-notes.md` still says "D-18 to D-64"; it describes the plan as it was on 2026-10-08 and was left as written.

**Checked.**

- `cargo metadata --no-deps --format-version 1` lists 12 packages.
- `grep -nw "R2" docs/v2/v2implementation.md` shows two lines: box 13 of §1A ("R2 is not used in V2") and D-65.
- `node scripts/check-file-tree.mjs`: 138 files in the tree of TS §5, 12 crates within the graph of TS §7. The script already held all 12 crates, and `deny.toml` already named them.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green; the frozen-file diff against the baseline is empty. **170 Rust, 76 Vitest, 12 Playwright**, the same as the baseline.
- The temporary case `web/tests-e2e/zz-pre.spec.ts` answered `/api/v1/events` itself, so it added no row to production. It was deleted before the gate.

**Not done.** Nothing was pushed. No dependency, no logic and no rule change: those are Prompt 32 and later.

## 2026-10-08 - Prompt 32: tree, purity and ban rules; test tooling; lint edges

**REOPENED V1 CONTRACT: `web/eslint.config.js`** (D-27, D-58, D-59). Nothing in it was loosened: the diff adds lines and removes none.

**Changed.**

| File | Change |
|---|---|
| `scripts/check-file-tree.mjs` | `ALLOWED_CRATE_DEPS["offcut-scene"]` gains `offcut-text` (D-28). `PURE_CRATES` gains `offcut-mp4`, `offcut-dsp`, `offcut-text`, `offcut-detect`, `offcut-scene` and `offcut-entitlement`: the `cargo tree` check now runs on eight crates |
| `deny.toml` | The `offcut-text` ban gains the wrapper `offcut-scene` (D-28). Nothing else |
| `.gitignore` | `web/public/ort/` and `fixtures/.cache/` |
| `package.json` | Dev-dependency `@playwright/test` at exactly `1.63.0`, the version `pnpm-lock.yaml` resolves for `web` (D-63). `check` runs a second `tsc --noEmit -p tests-e2e/tsconfig.json` after the first |
| `web/package.json` | Dev-dependency `@types/node` `^24.19.1`, the Node major of `engines` (D-63). Resolved: 24.19.1 |
| `web/tsconfig.json` | `include` is `src` alone (see "Differs") |
| `web/eslint.config.js` | 15 entries, listed below |
| `.github/workflows/ci.yml` | Step 6 gains the second `tsc` pass (see "Differs") |
| `docs/v2/v2implementation.md` | D-63, §4, §22.3 and §22.5 say what was built (5 replacements, each applied once) |

**Added.** `web/tests-e2e/tsconfig.json`: extends `../tsconfig.json`, sets `types` to the two of the app plus `node`, and includes `tests-e2e`, `playwright.config.ts`, `vite.config.ts`, `vitest.config.ts` and `../bench`.

**The 15 entries of `web/eslint.config.js`.**

| # | Entry | Kind |
|---|---|---|
| 1 | D-27 a: `usecases/import-clip.ts` may import `net/asset-fetch.ts` | Layer policy |
| 2 | D-27 b: `workers/**` may import `persistence/opfs.ts` | Layer policy |
| 3 | D-27 c: `workers/render.worker.ts` may import `config/entitlement-public-key.ts` | Layer policy |
| 4 | D-27 d: `models/**` may import `state/model-store.ts` and `config/model-manifest.json` | Layer policy |
| 5 | D-27 e: `models/model-manager.ts` may import `persistence/db.ts` | Layer policy |
| 6 | D-27 f: `state/**`, `models/**` and `usecases/**` may import `workers/protocol.ts`, types only | Layer policy |
| 7 | D-58: `usecases/import-clip.ts` may import `usecases/run-pipeline.ts` | Layer policy |
| 8 | D-58: `usecases/start-export.ts` may import `usecases/control-preview.ts` | Layer policy |
| 9 | D-59: `persistence/opfs.ts`, one cast to `Bytes`, in `toBytes()` | Cast override |
| 10 | D-59: `models/download.ts`, one cast to `Bytes`, in `toBytes()` | Cast override |
| 11 | D-59: `models/model-manager.ts`, one cast to `Bytes`, in `toBytes()` | Cast override |
| 12 | D-59: `state/model-store.ts`, one cast to `Bytes`, in `const ZERO_BYTES = 0 as Bytes` | Cast override |
| 13 | D-59: `usecases/import-clip.ts`, one cast to `ClipId`, in `newClipId()` | Cast override |
| 14 | D-59: `usecases/start-export.ts`, one cast to `ExportId`, in `newExportId()` | Cast override |
| 15 | D-59: `usecases/control-preview.ts`, one cast to `TimeMs`, in `toTimeMs()` | Cast override |

Entries 1 to 8 are eight objects at the end of `layerPolicies`. Entries 9 to 15 are the seven rows of the table `MINTING`, each of which becomes one configuration object.

**How a cast override works.** It does not switch the brand-cast rule off for its file. It replaces the rule's selectors by three: a cast to any other unit or id type is refused; a cast to the file's one type is refused unless it is the expression of the last `return` of the named helper (in `model-store.ts`: unless it is the top-level `const ZERO_BYTES = 0 as Bytes`); an angle-bracket cast is refused. So a second cast in the file fails, also inside the helper. `newClipId` and `newExportId` are the names §19.2 and §19.5 give; `toBytes`, `toTimeMs` and `ZERO_BYTES` were chosen here (known issue 4).

**Differs from the prompt, the guide or the plan.**

- **`web/tsconfig.json` includes `src` alone; `vite.config.ts` and `vitest.config.ts` moved to `web/tests-e2e/tsconfig.json`.** D-63 and the prompt only take `tests-e2e` out of the first. Done that way, the drill of G 1.4 did not fire: with `@types/node` installed, `console.log(process.cwd())` in `web/src/main.tsx` passed `tsc`. The cause, read with `tsc --explainFiles`: `vite/dist/node/index.d.ts` holds a reference to Node's types, and the two configuration files import Vite, so one program with `src` gave app code every Node global. Before V2 the reference found nothing, because `@types/node` was not installed. With the two files in the second program the drill fires, and the sentence of §22.3, "App code therefore never sees a Node global", is true. The test files under `src` import `vitest` and do not bring Node's types in (measured: `src` alone, tests included, refuses `process`). D-63, §4 and §22.3 of the plan are corrected.
- **`ci.yml` step 6 runs the second `tsc` pass.** Neither the prompt nor §22.5 asks for it. CI calls `tsc` directly, not `pnpm check`, so without the step CI would no longer type-check `tests-e2e`, nor now the three configuration files. One step added; §22.5 says so.
- **The drill "`persistence/opfs` imported from `state/`" (G 1.5)** does not reach the layer rule as the guide writes it: `opfs.ts` does not exist before Prompt 37, and the plugin classifies only an import it can resolve (known issue 5). As written, ESLint still fails, on `@typescript-eslint/no-unsafe-assignment`. With a one-line stub `opfs.ts` in place, `boundaries/dependencies` refuses the import. Both runs are in the table below.
- **`engines` in the root `package.json`** was put back on one line after `pnpm add` had spread it over three.

**The nine drills.** Each edit was undone after the rule fired; a frozen file was restored from a copy.

| # | Temporary edit | What fired |
|---|---|---|
| 1 | `crates/offcut-mp4/src/zz.rs` | `check-file-tree`: "not in the file tree of TS §5" |
| 2 | `offcut-text` as a dependency of `offcut-mp4` | `check-file-tree`: "offcut-mp4 may not depend on offcut-text (TS §7)" |
| 3 | `rand = "*"` in `offcut-entitlement` | `check-file-tree`: "resolves rand" and "resolves getrandom"; `cargo deny check bans`: "crate 'rand = 0.10.3' is explicitly banned" |
| 4 | `web-sys = "*"` in `offcut-scene` | `cargo deny check bans`: "crate 'web-sys = 0.3.106' is explicitly banned"; `check-file-tree` too |
| 5 | `console.log(process.cwd())` in `web/src/main.tsx` | `tsc --noEmit`: TS2591, cannot find name `process` (after the change above) |
| 6 | The same line in `web/tests-e2e/helpers/fixtures.ts` | Nothing, as intended: `tsc --noEmit -p tests-e2e/tsconfig.json` stays clean |
| 7 | `1 as Bytes` in `web/src/ui/pages/LandingPage.tsx` | `no-restricted-syntax`: the brand-cast rule |
| 8 | `import type { AppFailure } from "../../workers/protocol"` in the same file | `boundaries/dependencies`: no policy from `ui` to `workers` |
| 9 | `import { paths } from "../persistence/opfs"` in `web/src/state/capability-store.ts` | With a stub `opfs.ts`: `boundaries/dependencies`: no policy from `state` to `persistence`. Without it: `no-unsafe-assignment` |

**Both sides of the 15 entries, proven now and not when each file is first written.** A script wrote 12 stub files under the real names (the entries are keyed by name), linted them, changed one thing at a time, and deleted every stub.

- Positive: the 12 stubs, which use every new edge and each allowed cast once, lint with 0 problems.
- Negative, each refused by the rule named: a second cast outside the helper; a second `return` with a cast that is not the last statement; a nested cast inside the helper; another brand inside the helper; the helper's name in a file that is not listed; `ZERO_BYTES` with a value other than 0; a second constant; an angle-bracket cast (all `no-restricted-syntax`). Another use-case importing `asset-fetch`; `state` importing `persistence/opfs`; `media.worker.ts` importing the public key; `models` importing another state file; `download.ts` importing `persistence/db`; a value import of `workers/protocol` from `state`; a use-case pair that is not one of the two; an allowed pair the other way round (all `boundaries/dependencies`). 16 of 16.

**Checked.**

- `git check-ignore web/public/ort/x fixtures/.cache/x testclips/x` echoes the three paths.
- `git diff web/eslint.config.js`: 88 lines added, none removed. `web/eslint.config.js` is 358 lines, under the limit of 400.
- `node scripts/check-file-tree.mjs`: 139 files in the tree of TS §5; 12 crates within the graph; the eight pure crates resolve no browser or randomness crate.
- `pnpm --filter web exec tsc --noEmit -p tests-e2e/tsconfig.json` type-checks `tests-e2e/**`, `playwright.config.ts`, `vite.config.ts` and `vitest.config.ts`. `playwright.config.ts` was type-checked by nothing before.
- The gate: no `zz-` file; no test key in the environment; `pnpm check` (with two `tsc` passes), `pnpm test`, `pnpm build`, `pnpm e2e` green; the frozen-file diff against the baseline is empty. **170 Rust, 76 Vitest, 12 Playwright**, the same as the baseline.

**Pinned versions.** `@playwright/test` 1.63.0 (root, exact); `@types/node` 24.19.1 (`web`, `^24.19.1`; 26.6.4 is the latest release, and 24 is the major of `engines`).

**Not checked.** The `ci` run: nothing was pushed. The new step of `ci.yml` runs for the first time on the first push of `v2-build`, after Prompt 33. No YAML linter is installed; the step was read by eye.

## 2026-10-08 - Prompt 33: config contracts, reference clip, worker probe

**Added.** `web/src/config/entitlement-public-key.ts`: `ENTITLEMENT_PUBLIC_KEYS`, a `readonly Uint8Array[]`. It holds the production key, a base64 literal (the value of §1A item 12), and the test key only when `env.entitlementTestPublicKey` is set. No file imports it yet; the render worker is its one reader (D-27 c, Prompt 50).

**Changed.**

| File | Change |
|---|---|
| `web/src/config/env.ts` | One new field, `entitlementTestPublicKey: string \| null`: the value of `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` when it is set and not empty, else `null`. A value that is not base64, or does not decode to 32 bytes, throws when the module loads |
| `fixtures/speech/README.md` | The row of the reference clip gives the video-stream duration, 74,705 ms, the codecs, the size and the two white frames |

**The production key.** The literal is the public half of the seed that `.env.deploy` holds as `PROD_ENTITLEMENT_SIGNING_KEY`: derived again in a script that printed only "equal" or "not equal"; equal. Whether the key in Render has been replaced since 2026-10-08 is not known to the agent (open item 2). No seed was printed or written.

**The secret scan.** gitleaks 8.18.4, the version of `ci.yml`, was fetched into a scratch folder (the Windows archive; its SHA-256 equals the published checksum) and run. On the new file it reported one finding, the line of the literal. The line now ends with `// gitleaks:allow`, and the scan reports none. It was added before the line was ever committed, which matters: see known issue 8. The history of the branch, scanned the way CI does it: 48 commits, no finding.

**The reference clip (G 1.7).** `testclips/speech_scriptA_landscape_720p.mp4`:

| Read | Value |
|---|---|
| Video | `h264`, 1280x720 |
| Audio | `aac`, 48,000 Hz, 2 channels |
| Video-stream duration | 74.705033 s: 74,705 ms |
| Size | 35,201,023 bytes (limit 40,000,000) |
| Frames 150 and 1950 | `YAVG` 255 and 255 (at least 230 asked) |

**The worker probe (G 1.8), under the production CSP.** A temporary module worker, `web/src/workers/zz-probe.worker.ts`, started by one line in `main.tsx`; a production build served by `vite preview` with the headers of `web/vercel.json`; the console line read by a temporary Playwright case in Chrome, headless.

| Probe | Result |
|---|---|
| `webgpuCanvas` (adapter, device, a `webgpu` context on a 1080x1920 `OffscreenCanvas`, configured) | `true`; adapter `intel / gen-12lp` |
| `h264` (`avc1.640028`, 1080x1920, 8 Mbit/s, 30 fps) | `true` |
| `aac` (`mp4a.40.2`, 48 kHz, stereo, 160 kbit/s) | `true` |
| `opfsSync` (a sync access handle, opened and closed) | `true` |
| `crossOriginIsolated` on the page | `true` |
| Console lines starting "Refused to"; page errors | None |

No `false` and no error key: the gate of this prompt is passed and nothing goes to `experiments.md`. The worker file, the line in `main.tsx` and the Playwright case were removed the same hour; `git grep -n "zz-probe" -- web` prints nothing.

**Browser check (dev).** `vite` on port 5173, started three times; the page imported `/src/config/entitlement-public-key.ts` and returned the key lengths.

| `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` | Result |
|---|---|
| Not set | `[32]` |
| The base64 of 32 random bytes (made for the run, kept nowhere) | `[32, 32]` |
| `abc` | The import rejects: "VITE_ENTITLEMENT_TEST_PUBLIC_KEY must decode to 32 bytes" |

**Differs from the prompt or the guide.**

- **`// gitleaks:allow` was added before the push, not after a failed `ci` run.** The prompt adds it only "if the secret scan names the public key line". The scan was run locally and does name it.
- The probe worker of the guide was given `try`/`catch` around the two encoder questions and one more value, the adapter's vendor and architecture. It is temporary code either way.
- The probe build was `pnpm build:vite`, not `pnpm build`: no Rust file had changed. The gate ran the whole `pnpm build` afterwards.

**Checked.**

- G M-1: 12 workspace packages; `pnpm check` and `pnpm test` green; `git grep -n "zz-probe" -- web` empty; the frozen-file diff empty.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green. **170 Rust, 76 Vitest, 12 Playwright**, the same as the baseline.

**"Done when".**

- [x] The probe printed `webgpuCanvas`, `h264`, `aac`, `opfsSync` all `true` under `vite preview`, with no line starting "Refused to".
- [x] The clip reads 1280x720, H.264 and AAC, 48 kHz stereo, 74,705 ms, at most 40,000,000 bytes, two white frames.
- [x] `git grep -n "zz-probe" -- web` is empty; G M-1 passes; the draft pull request is open and `ci` is green (pull request #2, run of 2026-10-08 on `3ad4c06`; see the entry "The first push").

## 2026-10-09 - Prompt 34: `offcut-mp4`: errors, reader, boxes, sample tables

**Started before the `ci` run of Prompt 33 was read.** The prompts file asks for every "Done when" box of the previous prompt first. The push is the human's and had not been made; the human asked for Prompts 33 and 34 in one sitting. Open item 4 stands.

**Added.**

| File | Content |
|---|---|
| `crates/offcut-mp4/src/reader.rs` | `RandomAccess` (TS §15.1) and `MemReader`, the file in memory that tests use: a read past `len()` is `IoError::OutOfBounds`. `Cursor`: big-endian fields from bytes already in memory; a read past their end is `Truncated` |
| `crates/offcut-mp4/src/boxes.rs` | `BoxHeader`, `read_header`, `children` (§6.3); `child` and `descend`; the readers `read_ftyp`, `read_mvhd`, `read_mdhd`, `read_tkhd`, `read_hdlr`, `read_stsd` with their structs `Ftyp`, `TimeHeader`, `Tkhd`, `SampleEntry` |
| `crates/offcut-mp4/src/sample_table.rs` | `Sample`, `TrackTable`, `RawTables`, `resolve` (§6.4, steps 1 to 6), `MAX_RESOLVED_SAMPLES` (20,000); the readers `read_stts`, `read_ctts`, `read_stsc`, `read_stsz`, `read_stco`, `read_co64`, `read_stss`, `read_elst` with `StscEntry`, `Stsz`, `Edit` |

**Changed.**

| File | Change |
|---|---|
| `crates/offcut-mp4/src/lib.rs` | The three modules; `IoError` (3 variants), `ContainerError` (6), `MuxError` (4), as §6.1. `Malformed` and `Unsupported` carry a `&'static str`, so they cannot carry bytes of the file |
| `crates/offcut-mp4/Cargo.toml` | `offcut-types`, `thiserror`; dev `proptest` with the feature `std` |
| `Cargo.toml` | `[workspace.dependencies]`: `proptest = { version = "1.11.0", default-features = false }` |
| `deny.toml` | Two wrappers, both third-party crates (G 2.1): `proptest` for `rand`, `rand_core` for `getrandom`. `cargo deny check` named `rand 0.9.5` and `getrandom 0.3.4`; their direct parents were read with `cargo tree -i` |
| `docs/v2/v2implementation.md` | §6.2 and §6.3 say what was built (2 replacements, each applied once) |

**Pinned.** `proptest` 1.11.0, the latest release (`cargo search`), without its default features: no fork, no timeout, no temporary files. `thiserror` was already in the workspace (2.0.21).

**The bounds, and where each is.**

| Bound | Where |
|---|---|
| A box must end at or before its limit, and the limit is never past the end of the file | `read_header` |
| At most 4,096 children in one container: `Malformed("children")` | `children` |
| A path of more than 16 nested boxes: `Malformed("depth")` | `descend` |
| A table cannot have more records than its box has bytes for: `Malformed(<box>)`, checked before the records are read | `load_table` |
| A header box is read up to the bytes its fields need, never whole | `load` |
| An `avcC` or `esds` box over 1 MiB: `Malformed` | `load_descriptor` |
| A track of more than 20,000 samples is left unresolved: no per-sample memory, and no error | `resolve`; `read_stsz` does not load its size table |

**Decided here, where the plan gives a name and no shape, or is silent.** Each is the smallest choice that keeps Prompt 35 able to write `demux.rs` and `probe.rs` as §6.5 and §6.6 describe them.

- **`BoxHeader.body` is an offset,** the first byte after the header, like `start` and `end`. `body_len()` gives the length.
- **`RawTables`** (§6.4 names it and gives no fields): `timescale` (from `mdhd`), `movie_timescale` (from `mvhd`; an edit's length is in it), `file_len`, `stts`, `ctts`, `stsc`, `stsz`, `chunk_offsets` (from `stco` or `co64`), `stss`, `elst`. An absent `ctts` or `elst` is an empty list; an absent `stss` is `None`.
- **`SampleEntry`** (§6.3 lists its content): the four-character code; width and height; channel count and sample rate; `object_type`, the object type indication of `esds`, which §6.6 needs to tell AAC from MP3; `config`, the raw `avcC` payload or the `AudioSpecificConfig`.
- **`read_stsd` takes the track's handler type** (`vide`, `soun`). A sample entry does not say which of the two layouts it has; `hdlr` does. For audio it reads the three layouts QuickTime and ISO files use (versions 0, 1 and 2) and finds `esds` directly in the entry or inside `wave`.
- **`read_stsz` takes the largest size table it may load.** The count is always returned.
- **A `ctts` offset is an `i64`:** version 0 is read unsigned and version 1 signed, into one type.
- **Fewer than 8 bytes left in a container end its list of children** without an error: padding, or the terminator QuickTime writes. §6.3 does not name the case.
- **An edit of length 0** shows the media from its start time to its end. §6.4 does not name the case; read literally it would give a track of length 0.
- **An edit's length is converted to the media timescale to the nearest tick.**
- **Lenient where a table is longer or shorter than the track:** a `ctts` that covers fewer samples leaves the rest at `pts = dts`; a number in `stss` past the last sample is ignored. Chunks that run out before the samples do are `Malformed("stsc")`; a size table whose length is not the sample count is `Malformed("stsz")`.

**Differs from the prompt, the guide or the plan.**

- **The readers of the eight sample-table boxes are in `sample_table.rs`, not in `boxes.rs`** as §6.3 lists them. With all fourteen readers `boxes.rs` had 565 lines after `cargo fmt`, over the limit of 400 (TS §29). The tree of TS §5 already describes `sample_table.rs` by those eight boxes, so no file was added. §6.3 of the plan says so now.
- **`RandomAccess` has one provided method more than TS §15.1, `is_empty()`.** Copied verbatim, the trait fails clippy (`len_without_is_empty`), and no lint is switched off to pass a gate. An implementation does not write the method. To copy back into TS §15.1 (Prompt 60).
- **The depth bound is in `descend`,** not in `children`: the signature §6.3 gives `children` has no depth, and `BoxHeader` has no field for one. Nothing in the crate walks boxes recursively, so `descend` is the only place a file could ask for depth.
- **`proptest` is used already,** in one of the seven cases, so the dependency is not dead until Prompt 41: any bytes, any offset and any limit never make `read_header` panic, a header it returns lies inside its limit, and `children` and four readers run on it without a panic.

**Tests (inline): the seven rows of §6.9 for this prompt.**

| Row of §6.9 | Test |
|---|---|
| Header with 32-bit size, 64-bit size, size 0 | `boxes::a_header_with_a_32_bit_size_a_64_bit_size_and_size_0` (also `children` and `descend` on the three boxes, and the depth bound) |
| Box that ends past the limit | `boxes::a_box_that_ends_past_the_limit_is_truncated` (also a size under the header length, a cut header, a 64-bit size of `u64::MAX`, 4,097 children, and the property above) |
| `stts` counts that do not sum to the `stsz` count | `sample_table::stts_counts_that_do_not_sum_to_the_stsz_count_are_malformed` |
| `ctts` version 1 with a negative offset | `sample_table::ctts_version_1_with_a_negative_offset_puts_pts_before_dts` (the box is built in bytes and read with `read_ctts`) |
| No `stss` | `sample_table::without_stss_every_sample_is_a_keyframe` |
| One media edit with `media_time` = 2 frames | `sample_table::one_media_edit_of_two_frames_presents_the_first_frame_at_zero` (also a leading empty edit) |
| Two media edits | `sample_table::two_media_edits_are_unsupported` |

**The drill.** `let _ = v[0];` in the non-test code of `boxes.rs`: `cargo clippy -p offcut-mp4 --all-targets -- -D warnings` fails with "indexing may panic". Reverted from a copy.

**A real file, read with a temporary test (not asked for until Prompt 35; never committed).** The readers and `resolve` on `testclips/speech_scriptA_landscape_720p.mp4`, beside `ffprobe`:

| Read | This crate | `ffprobe` |
|---|---|---|
| Top-level boxes | `ftyp`, `moov`, `free`, `mdat` | - |
| Video samples | 2,246, resolved | 2,246 frames |
| Video keyframes (`stss`) | 10 | 10 |
| Video, presented | 2,241,151 ticks of 30,000: 74.705033 s | 74.705033 s |
| Video edit list | One media edit, `media_time` 2,393; the first frame's `pts` minus the offset is 0, and it is the smallest | Start time 0 |
| `avcC` | 41 bytes, starting `01 4d 40 1f` (Main) | Main |
| Video entry, `tkhd` | `avc1`, 1280x720; identity matrix; enabled | `avc1` |
| Audio samples | 3,493, resolved, every one a keyframe | 3,493 frames |
| Audio, presented | 3,576,841 ticks of 48,000: 74.517521 s | 74.517521 s |
| Audio entry | `mp4a`, 2 channels, 48,000 Hz, object type 0x40, `AudioSpecificConfig` `11 90` | `mp4a`, LC |
| Last video sample | Ends at byte 35,201,023, the length of the file | - |

Every value agrees. The test and its name are gone: `git grep -n zz_real -- crates` prints nothing.

**Checked.**

- `cargo test -p offcut-mp4 --lib`: 7 passed. `cargo clippy -p offcut-mp4 --all-targets -- -D warnings`: clean.
- `cargo deny check`: advisories, bans, licenses, sources ok. `node scripts/check-file-tree.mjs`: 143 files in the tree, and `offcut-mp4` checked as a pure crate (its normal dependencies resolve no browser or randomness crate; `proptest` is a dev-dependency).
- Lines above the test module: `boxes.rs` 392, `sample_table.rs` 387, `reader.rs` 97, `lib.rs` 54.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green; the frozen-file diff against the baseline is empty. **177 Rust, 76 Vitest, 12 Playwright** (Rust was 170; the seven are new).

**"Done when".**

- [x] `cargo test -p offcut-mp4 --lib` and `cargo clippy -p offcut-mp4 --all-targets -- -D warnings` pass; the seven rows are covered.
- [x] `cargo deny check` and `node scripts/check-file-tree.mjs` pass with `offcut-mp4` now checked as a pure crate.

**Not checked.** `ci`: nothing was pushed (open item 4). `cargo deny` and clippy ran on Windows only.

## 2026-10-09 - The first push: draft pull request #2, `ci` green

**Done by the human.** `git push -u origin v2-build` and a draft pull request, #2, `v2-build` into `main`. The first attempt ran in `sh2clips`, which is not a repository; the repository is `sh2clips/offcut`.

**Read by the agent** (the public API of GitHub; `gh` is not installed).

| Read | Result |
|---|---|
| `origin/v2-build` | `3ad4c06`, the commit of Prompt 34, equal to the local branch |
| Pull request #2 | Open, draft |
| The `ci` run on `3ad4c06` | Success. Job `ci`: 2.7 minutes. Jobs `deploy-api` and `deploy-web`: skipped, as on every pull request |
| Steps that ran on V2 code for the first time | The secret scan (the public key line with its `gitleaks:allow`), `cargo clippy`, `cargo deny` (the two new wrappers), `cargo test` (177), both `tsc` steps (the second is new in Prompt 32), the file tree, the hosts check, Playwright: all success |
| Production | Unchanged: `/api/v1/healthz` reports `322c7d3`; `main` is at `322c7d3` |

**Closes.** The last "Done when" box of Prompt 33, and with it G M-1. The three things the entry of Prompt 32 and 33 left unchecked hold on Linux: the new `tsc` step, `cargo deny` with `proptest`, and the secret scan.

**Changed.** This file only: open item 4 is now the standing push step, open item 8 (branch protection) is new, the box of Prompt 33 is ticked.

## 2026-10-09 - Prompt 35: `offcut-mp4`: demux, probe, validate

**Added.**

| File | Content |
|---|---|
| `crates/offcut-mp4/src/demux.rs` | `SampleMeta` and `Demuxer<R>` with the eight signatures of TS §15.1; `open`, steps 1 to 4 of §6.5; crate-private accessors for the probe (`brand`, `video`, `audio`, `track_counts`). After the first `#[cfg(test)]`: `fixture`, a module that builds small MP4 files in memory for the tests of this crate |
| `crates/offcut-mp4/src/probe.rs` | `probe`, every field rule of §6.6. It reads the sample count, the runs of `stts` and `presented`, never the sample list |
| `crates/offcut-mp4/src/validate.rs` | `validate_probe`: the 11 rules of TS §15.2 in that order, and the one place a `ClipInfo` is built |

**Changed.** `crates/offcut-mp4/src/lib.rs`: the three modules.

**`open`, as built.**

| Step | Behaviour |
|---|---|
| 1 | A file shorter than 8 bytes is `NotIsoBmff` (the rule of G 2.2), and so is one whose first box is not `ftyp`, `moov`, `mdat`, `free`, `wide` or `skip`. Top-level boxes are read until `moov`: it may come after `mdat`. A `moof` before it, or `mvex` inside it, is `Fragmented`. At most 4,096 top-level boxes are read |
| 2 | Every `vide` and `soun` track is counted. The video track is the first enabled one; the audio track is the first one |
| 3 | The chosen tracks are resolved. For a track of more than 20,000 samples only `stsd`, `stts`, the count of `stsz` and `elst` are read |
| 4 | `t0` is the smallest `pts - edit_offset` of the video track, in its own ticks. A time is `(pts - edit_offset) / timescale - t0`, computed in 128-bit integers and rounded down to microseconds. With no video samples to look at, `t0` is 0 |

**Decided here, where §6.5 and §6.6 are silent.**

- **No enabled video track:** the first video track is used. Some files leave the flag unset on their only track; refusing them as "no video" would be wrong, and the count of tracks is the same either way.
- **No `moov` at all** (an interrupted recording): `Malformed("moov")`, which becomes `REJECT_CORRUPT`.
- **A `trak` with no `hdlr`** is skipped and not counted.
- **A sample index with no track of that kind** is `Malformed("index")`.
- **A file with no `ftyp`** is reported as `Mp4`: §6.6 gives `Mov` for the brand `qt  ` and `Mp4` otherwise, and that is what is built. A classic QuickTime file has no `ftyp`; V3 may want it to say `Mov` (TE-12).
- **Sample rate and channel count of AAC** come from the `AudioSpecificConfig`; a channel configuration of 0 there (the layout is described elsewhere) falls back to the sample entry.
- **`is_vfr` counts every sample duration,** the last one too, as §6.6 says. A file whose last frame is shorter by more than 1 ms than the others reads as variable. The flag changes no rule; it is carried to `ClipInfo`.
- **`validate_probe` with `video_tracks > 0` and no `video`** (or the same for audio) answers `NoVideo` (`NoAudio`). `probe` never produces that; a hand-made `ProbeInfo` can.

**Tests (inline): the other nine rows of §6.9 and the short-file case. 17 in the crate.**

| Row of §6.9 | Test |
|---|---|
| `tkhd` matrices for 0, 90, 180, 270 degrees | `probe::the_tkhd_matrices_of_the_four_quarter_turns` |
| `avcC` bytes `64 00 28` | `probe::avcc_bytes_64_00_28_give_the_codec_string_avc1_640028` (also the sizes, both frame rates, the audio fields and the duration of that file) |
| Each of the 11 rules violated alone | `validate::each_of_the_eleven_rules_violated_alone_gives_its_reason` |
| Two rules violated | `validate::of_two_violated_rules_the_earlier_one_wins` (three pairs) |
| Exactly 90,000 ms; 60,500 `FpsMilli` | `validate::exactly_90_000_ms_and_60_500_fps_milli_are_accepted` (also a long side of exactly 1,920) |
| A 1920x1080 probe with `R90` | `validate::a_1920x1080_probe_turned_90_degrees_is_a_1080x1920_portrait_clip` (also unturned, and a square) |
| A video track of 30,000 samples in one `stts` run | `probe::a_video_track_of_30_000_samples_opens_unresolved_and_is_too_long`: 1,000,000 ms, 30,000 `FpsMilli`, `Duration` |
| A valid video track beside an `lpcm` track of 4,320,000 samples | `probe::an_lpcm_audio_track_of_4_320_000_samples_opens_and_is_the_wrong_codec`: `AudioCodec` |
| `read_video_sample(0, ..)` on an unresolved track | `demux::read_video_sample_on_an_unresolved_track_is_malformed_samples` (also a resolved track read: bytes, times, the keyframe to seek to, an index out of range) |
| The short-file case (the guide's) | `demux::a_file_shorter_than_one_box_header_is_not_iso_bmff` (6 bytes, 0 bytes, 20 bytes of text; also `moof`, `mvex`, and no `moov`) |

**The two drills.** Each was undone from a copy.

| Temporary edit | What fired |
|---|---|
| `.unwrap()` in the non-test code of `demux.rs` | clippy: "used `unwrap()` on an `Option` value" |
| `let _ = std::time::Instant::now();` in `probe.rs` | clippy: "use of a disallowed method `std::time::Instant::now`", with the reason of `clippy.toml`, "pure crates must not read a clock (TS §7)" |

**The real clip (the temporary test `zz_real_clip` of G 2.3; never committed).**

| Field | Printed | Expected |
|---|---|---|
| `duration` | 74,705 | 74,705 within 1 |
| `video.codec`, `codec_string` | `H264`, `avc1.4d401f` | `H264`, starts with `avc1.4d` |
| `coded_width`, `coded_height`, `rotation` | 1280, 720, `R0` | The same |
| `frame_count` | 2,246 | 2,246 |
| `is_vfr` | `true` | `true` |
| `avg_fps`, `max_fps` | 30,064 and 32,930 | Not fixed by the guide |
| `audio` | `Aac`, `mp4a.40.2`, 48,000 Hz, 2 channels | The same |
| `validate_probe(&p, true)` | `Ok(ClipInfo)`: 1280x720, `Landscape`, 74,705 ms | `Ok(ClipInfo)` with `Landscape` |

The test also read every video sample and printed more than the guide asks, to check `open` step 4 and the seek against `ffprobe`:

| Read | This crate | `ffprobe` |
|---|---|---|
| Smallest video `pts` | 0 | Start time 0 |
| End of the last video frame | 74,705,033 microseconds | Duration 74.705033 s |
| First video sample | `pts` 0, `dts` -79,767, keyframe, 91,736 bytes | The first packet is a keyframe at 0 |
| End of the last audio frame | 74,517,520 microseconds | Duration 74.517521 s |
| `keyframe_at_or_before` 0 ms, 30 s, 74 s | Samples 0, 896, 2040 | Keyframes are packets 0, 896 (29.807867 s) and 2040 (67.855600 s) |

No expectation was adjusted. `git grep -n zz_real_clip -- crates web scripts` prints nothing.

**Checked.**

- G M-2: `cargo test -p offcut-mp4 --lib`, 17 passed; `cargo clippy -p offcut-mp4 --all-targets -- -D warnings` clean; `cargo deny check` ok; `pnpm check` green; no `zz_real_clip` outside the documents that describe it.
- `validate_probe` has 11 `return Err(RejectReason::...)`, numbered 1 to 11 in the code and read against TS §15.2: `Container`, `NoVideo`, `Hevc`, `VideoCodec`, `NoAudio`, `MultiAudioTrack`, `AudioCodec`, `Duration`, `Resolution`, `FrameRate`, `DecodeUnsupported`. It cannot return `FileSize`, `Corrupt` or `NoSpeech`. Its limits are the four it imports from `offcut_types::limits`.
- Lines above the test module: `demux.rs` 369, `probe.rs` 180, `validate.rs` 85. `boxes.rs` and `sample_table.rs` were not touched.
- The gate: no `zz-` file; no test key in the environment; `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e` green; the frozen-file diff against the baseline is empty. **187 Rust, 76 Vitest, 12 Playwright** (Rust was 177; the ten are new).

**"Done when".**

- [x] `cargo test -p offcut-mp4 --lib`: the 16 rows of §6.9 and the short-file case pass.
- [x] The real clip printed the values above; `git grep -n zz_real_clip` finds it in the two documents that describe the test and nowhere else.
- [x] `validate_probe` has 11 rejecting branches, read against TS §15.2 line by line; G M-2 passes.

**Human.** Push; read `ci` (open item 4).
