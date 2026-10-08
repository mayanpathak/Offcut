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
| 2 | Decide whether to replace `ENTITLEMENT_SIGNING_KEY` in Render: its private half was shown in a chat on 2026-10-08 (`v1changelog.md`). If it is replaced, say so before Prompt 33, which writes the public half into `config/entitlement-public-key.ts` | Human | Prompt 33 |
| 3 | R1, a 2021-class Windows laptop with 8 GB and an integrated GPU: book it for the days of Prompts 44, 51 and 59, and set it up once (G 0.6: Node, pnpm, Chrome, Python 3 with NumPy, `ffmpeg`; a clone; the reference clip copied by hand). Boxes 13 and 14 of §1A stay open until then. Both gates are read on R1 and on no other machine | Human | Prompt 44 |
| 4 | Install `gh` (`winget install GitHub.cli`, then `gh auth login`), or do each `gh` step on the GitHub website | Human | Prompt 33 |
| 5 | Submit the merchant onboarding (TE-9) if it is not submitted. Approval can take two weeks | Human | V6 |
| 6 | Reset the Neon password (V1 item 36) and replace the demo video (V1 item 33) | Human | Before the page is announced |
| 7 | The file `.env.local` in the repository root holds one line with no name, a test-mode API key. Git ignores the file and no program reads that line. Move it into `.env.deploy` under a name | Human | Any time |

## Known issues for later prompts

| # | Issue | Affects |
|---|---|---|
| 1 | A `src/lib.rs` of 0 bytes fails `cargo fmt --check`: rustfmt wants one line break. The eight skeletons hold one line break (LF) | Any prompt that creates an empty Rust file |
| 2 | `coding-promptsv2.md` does not say which branch Prompts 59 and 60 work on after the merge. On `main`, the line `BASE=$(git merge-base main HEAD)` of the Standard Agent Block gives `HEAD`, and the frozen-file diff then compares nothing. From the merge on, use the baseline SHA above in place of `$BASE`, and put the commits made after the merge on a branch of their own | Prompts 59, 60 |
| 3 | `cargo deny check` passes with warnings, as in V1 (V1 item 6): `unused-wrapper` for crates that are not dependencies yet, and `duplicate`. They are warnings, not errors | Prompts 34 to 48 |

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
