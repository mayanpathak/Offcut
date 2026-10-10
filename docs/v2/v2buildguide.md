# v2buildguide.md - Offcut V2: step-by-step build guide

**What this is.** `v2implementation.md` says what each V2 file contains. This guide says what to do next, which command to run and how to know it worked. V2 starts from the V1 repository as it stands on `main` (the `v1` tag does not exist yet, see "Read this first" item 1) and ends at the `v2` tag: the reference clip goes in, a 1080x1920 MP4 with Clean captions and one NumberReveal comes out, E-3 and E-4 are measured on R1, and the M0 gate decision is written down.

**Revision of 2026-10-08.** The guide was checked against the repository, the development machine and `docs/v2/v2implementation.md`, and corrected: the Python interpreter, the production database variable, CI from the first day, the unstable-API flag, and several checks that could not fire as written. R2 (the Apple M1) is no longer part of V2, by the founder's decision of that date: every measurement and exit box is read on R1. "Read this first", items 9 to 12, and the tables at the end list each change.

## How to read the references

| Notation | Meaning |
|---|---|
| `§n` | Section n of `docs/v2/v2implementation.md` |
| `TS §n` | Section n of `docs/technicalspec.md` |
| `PS §n` | Section n of `docs/product.md` |
| `BP §n` | Section n of `docs/buildplan.md` |
| `V1 §n`, `V1 D-n` | Section or decision of `docs/v1/v1implementation.md` (D-1 to D-17) |
| `D-n` | Decision D-18 to D-64 in §2 (and D-65, once Step 1.1 has added it; D-66, D-67, D-68 and D-69 were added on 2026-10-09, D-70, D-71 and D-72 on 2026-10-10) |
| `S-n` | Build-order step S1 to S16 in §5 |
| `TE-n`, `E-n` | Technical experiment (TS §37), product experiment (PS §19); V2 procedures are in §24.1 |
| `INV-n` | Invariant (TS §35) |
| `Step N.M` | Step M of Phase N of this guide |

## How the guide is organised

Thirteen phases, 0 to 12. Each phase is a list of numbered steps and ends with a milestone: commands, expected output, a `Pass:` line and a commit. **Do not start a phase while the previous milestone fails.** Phases 5 and 8 are gates: a miss there sends you to §24.3 before anything else is built. All work happens on one branch, `v2-build`; it merges to `main` once, in Phase 12. A draft pull request is opened at Milestone 1, because `ci.yml` runs on pull requests and on `main` only: from then on, push after every milestone commit, and do not start the next phase while the `ci` run of that push is red. A pull request never deploys.

## Phase overview

| Phase | Covers (S-steps and sections) | Plan day | Milestone in one line |
|---|---|---|---|
| 0 Environment and V1 baseline | §1A, §3 | Before day 1 (0 days) | Every tool prints a version; `main` is green; baseline SHA recorded |
| 1 Contracts and repo setup | S1; §2, §4, §15.1, §22 | Day 1, first half (0.5) | 12 workspace members; `pnpm check` green; worker probe passes under the production CSP; draft pull request open and `ci` green |
| 2 MP4 read side | S2; §6.1-§6.7, §6.9 | Day 1, second half (0.5) | 17 inline cases green (the 16 of §6.9 and the short-file case); the real clip probes as 1280x720, 74,705 ms |
| 3 Resampler, core bindings, workers, ingest | S3, S4; §7, §13.1-§13.3, §15.2, §15.6, §15.7, §16.1-§16.3 | Day 2 (1) | Reference clip imported in the browser; `pcm48.length === round(durationMs x 48)` |
| 4 Model delivery and muxer | S5, S7; §6.8, §15.4, §17, §18.1, §23.2, §23.3 | Day 3 (1) | Model reaches `ready` and survives a reload with zero requests; 12 round-trip cases green |
| 5 ASR spike (gate) | S6; §8, §13.3, §16.4, §16.5 | Day 4 (1) | TE-1 passes in a production build; first E-3 reading on R1 written down |
| 6 Scene, detection, entitlement | S8, S9; §9, §10, §11, §23.4 | Day 5 (1) | 29 inline cases green, including the cross-language token vector |
| 7 Renderer and render bindings | S10; §12, §13.4, §13.5, §15.5 | Day 6 (1) | Both WASM bundles built; workspace clippy clean; `check-hosts` green with the render bundle |
| 8 Render and encode spike (gate) | S11; §16.6-§16.9, §23.9 | Day 7 (1) | Spike export passes verifier checks 1-6; TE-3, TE-4 recorded; first E-4 reading on R1 |
| 9 State and use-cases | S12; §15.3, §18, §19 | Day 8, first half (0.5) | `tsc`, ESLint, Vitest clean; 8 blocker codes covered by the copy check |
| 10 UI and copy | S13; §20 | Day 8 second half to day 9 first half (1) | Drop, feed, preview and seeded export work by hand in dev and in a production build |
| 11 E2E, CI, bench | S14; §22.2-§22.5, §23.4-§23.8, §23.10 | Day 9, second half (0.5) | Pull request green including the Windows media job; TE-10 recorded |
| 12 Deploy, measure, gate, tag | S15, S16; §24, §25.3, §26 | Day 10 (1) + gate, Sun 1 Nov 2026 | Deployed and verified; R1 results committed; M0 decision written; tagged `v2` |

Plan days sum to 10 (BP §0: weeks 2-3). §5 allows an earlier start; day numbers then count from the real start and the gate is read no later than Sun 1 Nov 2026.

## Read this first: 12 things that will bite you

1. **There is no `v1` tag.** §1A carries the tag over as a later V1 item, while V2 "starts from `v1`". Fix: Phase 0 verifies `main` instead and records its SHA as the baseline; every frozen-file check diffs against that SHA. Do not create the `v1` tag as a side effect: it needs the rest of V1 §16. Applied in Step 0.2.

2. **Shell and OS.** This guide is written for Windows 11 with Git Bash, as found on the development machine on 2026-10-08. Three traps: (a) `bash` must resolve to Git Bash, not `C:\Windows\System32\bash.exe` (WSL), or every `.sh` script runs in the wrong filesystem; (b) §22.2 calls `python verify/verify_mp4.py`, and on this machine plain `python` (and `python3`) is an MSYS2 Python 3.9 with no pip and no NumPy, because `C:\msys64\mingw64\bin` comes first in PATH. The Python to use is 3.13, so its folder is put first in PATH in every terminal (Quick reference a); that also covers the `verify` script and the verifier call inside the E2E suite; (c) on `windows-latest` the default step shell is PowerShell, so `e2e-media.yml` needs `defaults.run.shell: bash`. Applied in Steps 0.1 and 11.4.

3. **Four version pairs must match.** (a) `wasm-bindgen` CLI and crate (TS §3): the root `Cargo.toml` pins the crate exactly (`=0.2.129`), so a new dependency that needs a newer `js-sys` or `web-sys` fails to resolve until the pin, the CLI and the version in `ci.yml` are raised together. (b) `wgpu` must be the exact version the pinned `vello` depends on, or two `wgpu` trees get linked. (c) Root `@playwright/test` must equal `web`'s (D-63). (d) The `/ort/<version>/` files must come from the `onnxruntime-web` that `@huggingface/transformers` resolves; with pnpm that package is not at `web/node_modules/onnxruntime-web`. Applied in Steps 3.2, 7.1, 1.4, 5.2.

4. **The `--cfg=web_sys_unstable_apis` flag is probably not needed, and if it is, `.cargo/config.toml` alone does not deliver it.** In the `web-sys` that `Cargo.lock` holds (0.3.106, read on 2026-10-08) the types `VideoFrame`, `OffscreenCanvas` and `FileSystemSyncAccessHandle` are not gated; only `VideoFrame.rotation`, `.flip` and `.metadata()` are, and V2 calls none of them. So `Renderer::render(&web_sys::VideoFrame)` (TS §19.2) compiles on every target without the flag. The one place a gate can still sit is `wgpu`'s import of a `VideoFrame` as an external image, which exists on wasm32 only. `scripts/build-wasm.sh` sets `RUSTFLAGS` on its `cargo build` line, and an environment `RUSTFLAGS` replaces `build.rustflags` of `.cargo/config.toml` entirely: a flag for the bundle goes on that line. Applied in Step 7.1.

5. **Five places where the spec's build order references a file that does not exist yet.** (a) The pool table (§15.7) names three worker scripts; Vite fails on a `new URL()` to a missing file, so the table grows one row per phase: `media` (Step 3.5), `asr` (Step 5.3), `render` (Step 8.2). (b) `pool.preload()` calls `preloadRender()`, which exists from Step 7.4. (c) `CoreApi.normalizeTranscript` is added in Step 5.1, not with the rest of `CoreApi`. (d) `render.worker.ts` (S11) needs `preview-loop.ts` (S12) for its three preview handlers; they are added in Step 9.4. (e) Copy and `COPY_PENDING` must change in the same step as the component that shows them (V1 D-13), so `messages.ts` and `check-copy-codes.mjs` are edited in Steps 4.5, 9.2 and 10.1, not once in S13. Likewise the two `landing.spec.ts` cases (D-56) are replaced in Step 10.3, when the drop zone is wired, not in S14.

6. **Files the specs need and do not list.** (a) `docs/v2/v2buildguide.md` (this file), `docs/v2/v2implementation-notes.md` and `docs/v2/coding-promptsv2.md` (the agent prompts written from this guide, Prompts 31 to 60) are missing from D-46 and the TS §5 tree. (b) The download that fills `fixtures/.cache/` (§22.3, §23.4) has no file, and Playwright's `globalSetup` is the wrong place for it: it belongs to the whole config, so it would also run for `non-media`, on the Linux `ci` job and on every local `pnpm e2e`, and fetch the model there. It becomes a setup project with one new file, `web/tests-e2e/media.setup.ts`, that only `media` and `bench` depend on. (c) `routes.tsx` must pass `appPath` to `LandingPage` (D-48) and is not in the §4 "CHANGED" list. (d) `.cargo/config.toml`, only if Step 7.1 finds the `web-sys` type itself gated. Each is added to TS §5 or §4 in the commit that touches it. Applied in Steps 1.1, 11.1, 10.3, 7.1.

7. **The spec's checks for S4, S5 and S13 run in `pnpm dev`, where the CSP, COEP and `worker-src 'self'` do not apply the way they do in production.** A runtime that spawns a `blob:` worker or fetches from a hub works in dev and dies deployed. Fix: a 20-minute worker probe on day 1 and both spikes run against `pnpm build` + `vite preview` (production CSP, V1 D-15). Applied in Steps 1.8, 5.5, 8.3.

8. **The test public key must never reach a deployment.** `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` lets anyone with the test seed mint Creator tokens (D-24). Keep it out of every `.env*` file and out of Vercel; export it per terminal, write it literally only in CI step 8, and prove the deploy guard can fail. Applied in Steps 0.5, 11.4, 12.1.

9. **CI does not see a branch.** `ci.yml` runs on pull requests and on pushes to `main`. A branch that is only pushed, or not pushed at all, is never checked, and the first Linux run would be on day 9 with ten phases of `cargo deny` wrappers, license entries, clippy and the secret scan behind it. Fix: push `v2-build` and open a draft pull request at Milestone 1; push after every milestone. One thing only CI can show: gitleaks 8.18.4 may take a 44-character base64 literal beside the word `KEY` for a secret (not tested: gitleaks is not installed on this machine). If the secret scan fails on the public key of Step 1.6 or on the test public key of Step 11.4, end that line with a `gitleaks:allow` comment; both are public values (TS §24.2, D-24). gitleaks reads the whole history, so removing the line in a later commit does not clear the finding. Applied in Milestone 1 and Steps 1.6, 11.4.

10. **`DATABASE_URL` is the local database.** The `.env` that Quick reference a loads points `DATABASE_URL` at the Postgres on this machine (V1 known issue 1). The production string is `PROD_DATABASE_URL` in the git-ignored `.env.deploy` (V1 changelog, 2026-10-08). A query for the deployed analytics rows, or for the E-1 counts of the M0 gate, run with `$DATABASE_URL` answers from your own test rows. Applied in Steps 12.3 and 12.7.

11. **Machine facts the first draft assumed.** The repository root is `/c/Users/Mayan/Desktop/sh2clips/offcut`, not `sh2clips` (the quick reference of the V1 guide has the same slip). `gh` is not installed: Step 0.1 installs it, and every `gh` line can also be done on the GitHub website. Python 3.13 with NumPy 2.5.3 and ffmpeg 9.0.2 are installed (item 2). The development machine is not R1 (Step 0.6). Applied in Steps 0.1, 0.2, 0.6.

12. **No R2.** The founder decided on 2026-10-08 not to measure on R2 (an Apple M1 with 8 GB): no Mac is available. R1 is the only reference machine of V2, and TE-3, TE-4, E-3 and E-4 are read on it alone. The plan still names R2 in §1A (boxes 13 and 14), §5 (S15), §23.1, §24.1 (E-3, TE-3, TE-4, E-4), §25.3 (one box) and §26 (three boxes). Step 1.1 records the decision there as D-65 and rewords those lines, so that no box is left that cannot be ticked. Cost, to be written into that decision: PS §9.3 lists macOS as supported, PS §20.1 and BP §12.1 ask for a run on R2 at the launch gate, and nothing in V2 will have shown that the pipeline runs on a Mac; the AAC priming count of D-33 is measured for the Windows encoder only. Applied in Steps 0.6, 1.1, 8.4, 12.4. **Since D-69 (2026-10-09) R1 is not measured either:** it is not available. Every reading this guide puts on R1 (Steps 8.4 and 8.5, 12.3 to 12.5, 12.7) is taken on the development machine, D1, by the agent, and is recorded under the name `d1`; the bench file is `bench/results/d1-<date>.json`.

---

## Phase 0 - Environment and V1 baseline (§1A, §3)

Goal: every tool V2 needs prints a version on this machine, `main` passes the full V1 suite, and the 14 preconditions of §1A are ticked.

### Step 0.1 - Audit the tools

Run in Git Bash. The states in the table were read on the development machine on 2026-10-08; run the block anyway.

```bash
which bash                                   # /usr/bin/bash   (NOT /c/Windows/System32/bash.exe)
git --version                                # git version 2.x
rustc --version && rustup target list --installed | grep wasm32-unknown-unknown   # wasm32-unknown-unknown
wasm-bindgen --version                       # wasm-bindgen 0.2.N
wasm-opt --version                           # wasm-opt version N
cargo deny --version                         # cargo-deny N
node --version && pnpm --version             # the versions root package.json pins (engines, packageManager)
export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:$PATH"   # item 2b; needed in every terminal (Quick reference a)
python --version                             # Python 3.13.x   (3.9.x: the PATH line above did not run)
python -c "import numpy; print(numpy.__version__)"   # 2.5.3 or later
ffmpeg -version | head -1 && ffprobe -version | head -1   # two version lines
gh --version                                 # gh version 2.x   (item 11)
```

| Tool | State | Action |
|---|---|---|
| Git Bash as `bash` | Check with `which bash` | If WSL wins: open "Git Bash" directly, or put `C:\Program Files\Git\usr\bin` ahead of `System32` in PATH |
| Rust, wasm32 target, `wasm-bindgen`, `wasm-opt`, `cargo-deny`, Node, pnpm | Installed by V1 | Nothing. Do not upgrade any of them in V2 |
| Python 3 | Installed: 3.13.5. Plain `python` is an MSYS2 3.9 until the PATH line above has run | Nothing to install. Leave MSYS2 in PATH: other tools on this machine may use it |
| NumPy | Installed: 2.5.3, in the Python 3.13 | Nothing (pinned later by `verify/requirements.txt`) |
| `ffmpeg`, `ffprobe` | Installed: 9.0.2 | Nothing. Development tool only (TS §36) |
| GitHub CLI `gh` | Not installed | `winget install GitHub.cli`, reopen the terminal, `gh auth login`. Or do each `gh` line of this guide on the GitHub website |
| Chrome stable | Installed | Open `chrome://gpu`: "WebGPU: Hardware accelerated" |
| Same tools on R1 | Unknown | Step 0.6 |

### Step 0.2 - Verify the V1 baseline

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut    # the repository root (item 11)
git switch main && git pull --ff-only        # Already up to date.
git status --porcelain                       # (no output). If it lists the V2 documents, commit them first, with their changelog entry
git tag -l v1                                # (no output expected, item 1)
git rev-parse HEAD                           # write this SHA down: the V1 baseline
pnpm install --frozen-lockfile               # Done
pnpm check && pnpm test && pnpm build && pnpm e2e   # all green (precondition 10)
```

`pnpm test` runs the server tests, which need the V1 environment loaded (`DATABASE_URL` for a real Postgres). Load it the way the V1 guide's quick reference does.

| `git tag -l v1` prints | Do |
|---|---|
| Nothing | Expected. Continue from `main` |
| `v1` | `git merge-base --is-ancestor v1 main && echo ok` must print `ok`; the baseline is still `main` |

Then check the last `ci.yml` run on `main` is green on GitHub (it includes the header check and the `@smoke` case, precondition 2).

### Step 0.3 - Verify the deployed V1 and the remaining preconditions

| Do | Expect |
|---|---|
| Open the deployed URL in Chrome, DevTools console: `crossOriginIsolated` | `true` (precondition 3) |
| DevTools, Application, IndexedDB, `offcut` | Version 1, eight stores, one of them `entitlement` (precondition 11) |
| Console: `(await fetch("<asset base URL>/<any uploaded path>", { headers: { Range: "bytes=0-1023" } })).status` | `206` (precondition 1) |

```bash
pnpm -C web exec tsc --noEmit                                                   # no output (precondition 4)
grep -c "VIDEO_ENCODE_LADDER\|AAC_ENCODE_CONFIG\|KEYFRAME_INTERVAL_FRAMES\|videoConfigFor" web/src/workers/render/encoders.ts   # >= 4 (precondition 5)
ls bench/results                                                                # .gitkeep (precondition 8)
grep -c "twelve thousand dollars" fixtures/speech/README.md                     # >= 1 (precondition 7)
node -e "console.log(Buffer.from('KnENazU2ypDUgG3mibKKiP0g4L5zgrgiWTCeCLNjeLg=','base64').length)"   # 32 (precondition 12)
```

If the signing key in Render was replaced since 2026-10-08, derive the public half again (§1A item 12) without pasting the seed into a file, a commit or a chat.

### Step 0.4 - Accounts

| Account | Do now | Keep for later |
|---|---|---|
| Asset host (chosen in TE-7) | Add CORS origin `http://localhost:4173` beside the app origin and `http://localhost:5173` (D-41) | The upload credentials `scripts/upload-assets.sh` already uses |
| Model source (§3.3) | Start downloading the small-size English model files (D-66) with word-timestamp support, quantized so the set stays at or under 260,000,000 bytes (TS §16.1). The model card must state the export returns cross-attention outputs; without them TE-1 fails on timestamps | The local folder; hashes are taken in Step 4.2 |
| GitHub | Settings, Billing: note the Actions minute allowance and the Windows multiplier for this repository's visibility | Needed for TE-10 in Step 11.5 |
| **Merchant of record (TE-9)** | **Long lead, carried over from V1: if onboarding is not submitted, submit it today.** Approval can take two weeks and V6 needs it (BP §2) | Nothing in V2 |
| Neon (V1 open item 36) | Reset the database password before the page is announced; put the new string into `DATABASE_URL` in Render and into `PROD_DATABASE_URL` in `.env.deploy`, nowhere else | Step 12.3 reads production through `PROD_DATABASE_URL` (item 10) |

Verify the CORS change:

```bash
curl -sI -H "Origin: http://localhost:4173" "<asset base URL>/<any uploaded path>" | grep -i access-control-allow-origin   # http://localhost:4173
# no output: the origin is not in the bucket's CORS policy yet (the state on 2026-10-08)
```

### Step 0.5 - Keys

V2 adds no secret (§3.4). It adds one test key pair.

```bash
node -e "console.log(require('crypto').randomBytes(32).toString('hex'))"        # 64 hex chars: the TEST seed
```

Derive its public half (replace `<seed hex>`):

```bash
node -e "const c=require('crypto');const k=c.createPrivateKey({key:Buffer.concat([Buffer.from('302e020100300506032b657004220420','hex'),Buffer.from(process.argv[1],'hex')]),format:'der',type:'pkcs8'});console.log(c.createPublicKey(k).export({type:'spki',format:'der'}).subarray(-32).toString('base64'))" <seed hex>
# 44 base64 chars ending in "=": the TEST public key
```

| Value | Lives in | Never in |
|---|---|---|
| `ENTITLEMENT_SIGNING_KEY` (production seed) | Render environment only (TS §24.2) | Any file, commit, log, chat, shell history |
| Production public key | `web/src/config/entitlement-public-key.ts` (Step 1.6) | - (public value) |
| Test seed | The constant `TEST_ENTITLEMENT_SEED` in `web/tests-e2e/helpers/fake-api.ts` (Step 6.5) | `web/src/**` |
| Test public key | `ci.yml` step 8, written literally (Step 11.4); `export` in a terminal for local E2E builds | **Vercel, any `.env*` file, `~/.bashrc`** |

Keep both test values in a scratch note until Step 6.5.

### Step 0.6 - R1 (preconditions 13 and 14)

R1 is a 2021-class Windows laptop with 8 GB of memory and an integrated GPU, on mains power (PS A-7). The development machine is not R1: it has an i5-12450H, 16 GB and a GTX 1650, so a reading taken on it is labelled `dev` and never stands for E-3 or E-4. R2 is not used (item 12).

Book R1 for plan days 4, 7 and 10. On it, once: install Node, pnpm, Chrome stable, Python 3 with NumPy and `ffmpeg`; clone the repository; copy `testclips/speech_scriptA_landscape_720p.mp4` into it by hand (the folder is git-ignored). It does not need Rust: copy `web/dist` from the development machine when a build is needed.

```bash
# on R1 (Git Bash)
node --version && pnpm --version && python --version && ffprobe -version | head -1   # four version lines
```

If you have no R1-class laptop, arrange access now: both questions of the M0 gate are defined on R1 (§24.3), and without it the gate cannot be read. If `python` on R1 is not a Python 3 with NumPy, fix its PATH the way Quick reference a does here.

### Milestone 0

```bash
which bash                                   # /usr/bin/bash
python -c "import numpy, sys; print(sys.version_info[0], numpy.__version__)"   # 3 <version>
ffprobe -version | head -1                   # ffprobe version ...
git status --porcelain                       # (no output)
pnpm check && pnpm test && pnpm build && pnpm e2e   # all green
git switch -c v2-build                       # Switched to a new branch 'v2-build'
git commit --allow-empty -m "V2 S0: baseline $(git rev-parse --short main) verified"
```

Pass: every command above prints the stated output, the three browser checks of Step 0.3 hold, the CORS header names `http://localhost:4173`, and all 14 boxes of §1A are ticked, boxes 13 and 14 for R1 only (item 12).

Commit: `"V2 S0: baseline <sha> verified"` (empty commit; it marks where V2 starts). Do not begin Phase 1 until this passes.

---

## Phase 1 - Contracts and repo setup (S1)

Goal: the repository holds every V2 crate as an empty member, every lint and tree rule V2 code will be judged by, and proof that a module worker gets WebGPU, WebCodecs and a sync OPFS handle under the production CSP.

### Step 1.1 - Docs and the TS §5 tree

Create `docs/v2/experiments.md` and `docs/v2/v2changelog.md`; place `docs/v2/v2implementation.md` and this file beside them. First changelog entry: the baseline SHA from Step 0.2 and the tool versions from Step 0.1.

Edit the tree in `docs/technicalspec.md` §5, same commit:

| Add to TS §5 | Source |
|---|---|
| `crates/offcut-mp4/src/mux_boxes.rs` | D-55 |
| `web/tests-e2e/tsconfig.json` | D-63 |
| `docs/v2/v2implementation.md`, `experiments.md`, `v2changelog.md` | D-46 |
| `docs/v2/v2buildguide.md`, `docs/v2/v2implementation-notes.md`, `docs/v2/coding-promptsv2.md` | Item 6a |

Record the R2 decision in `docs/v2/v2implementation.md`, same commit (item 12):

| Where in the plan | Change |
|---|---|
| §2 | New decision D-65: R2 is not measured in V2; R1 is the only reference machine. Reason and cost as in item 12. To copy back: BP §1.5, §4.4 and §12.1; TS §28 (M0.2 to M0.4), §30 and §37 (TE-3, TE-4); PS A-7 and §20.1 |
| §0, §2 | "D-18 to D-65" |
| §1A, boxes 13 and 14 | R1 only |
| §5 (S15), §23.1 | "on R1" in place of "on R1 and R2" |
| §24.1: E-3, TE-3, TE-4, E-4 | R1 only. TE-4 passes when a ladder entry is supported on R1 |
| §25.3 | "The R1 baseline file is in `bench/results/`" |
| §26 | The three boxes that name R2 (`pnpm e2e:device`, the verifier, the bench files) name R1 only |

### Step 1.2 - Eight crate skeletons, then the workspace members

Directories first, manifest second: `cargo metadata` fails on a member whose directory is missing. Copy the `[package]` header style and `[lints] workspace = true` from `crates/offcut-types/Cargo.toml`.

```bash
for c in mp4 dsp text detect entitlement scene render wasm-render; do
  mkdir -p crates/offcut-$c/src && : > crates/offcut-$c/src/lib.rs
done
```

| Crate `Cargo.toml` | Besides the header |
|---|---|
| `offcut-wasm-render` | `[lib] crate-type = ["cdylib"]` |
| `offcut-entitlement` | `[features] sign = []` (§10) |
| All eight | `[lints] workspace = true`; no dependency yet except `offcut-types` where §22.1 lists it |

Then add the eight paths to `members` in the root `Cargo.toml` (§22.1).

```bash
cargo metadata --no-deps --format-version 1 | node -e "let s='';process.stdin.on('data',d=>s+=d).on('end',()=>console.log(JSON.parse(s).packages.length))"   # 12
```

### Step 1.3 - Tree, purity and ban rules

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `scripts/check-file-tree.mjs` | §22.4, D-28 | Allowed pair `offcut-scene` to `offcut-text`; `PURE_CRATES` gains six names (`mp4`, `dsp`, `text`, `detect`, `scene`, `entitlement`) |
| 2 | `deny.toml` | §22.1 | The `offcut-text` ban gains the wrapper `offcut-scene`. Nothing else yet |
| 3 | `.gitignore` | §22.3 | `web/public/ort/`, `fixtures/.cache/` |

Prove it by, undoing each edit after the rule fires:

| Temporary edit | Rule that must fire |
|---|---|
| `: > crates/offcut-mp4/src/zz.rs` | `node scripts/check-file-tree.mjs`: file not in TS §5 |
| `offcut-text` as a dependency of `offcut-mp4` | `node scripts/check-file-tree.mjs`: undeclared crate edge |
| `rand = "*"` as a dependency of `offcut-entitlement` | `node scripts/check-file-tree.mjs` (pure-crate `cargo tree` check) or `cargo deny check` |
| `web-sys` as a dependency of `offcut-scene` | `cargo deny check`: banned for a pure crate |

```bash
git check-ignore web/public/ort/x fixtures/.cache/x testclips/x   # three paths echoed
```

### Step 1.4 - Test tooling (D-63)

```bash
PW=$(node -p "require('./web/package.json').devDependencies['@playwright/test']")
pnpm add -D -w --save-exact "@playwright/test@${PW#[\^~]}"        # root dev-dependency, same version as web
node -p "require('./web/package.json').engines?.node ?? require('./package.json').engines?.node"   # the Node major to match
pnpm -C web add -D "@types/node@<that major>"
```

Then: remove `tests-e2e` from `include` in `web/tsconfig.json`; create `web/tests-e2e/tsconfig.json` (§22.3); add the second `tsc --noEmit -p web/tests-e2e/tsconfig.json` to the root `check` script (§22.2).

| Temporary edit | Rule that must fire |
|---|---|
| `console.log(process.cwd())` in `web/src/main.tsx` | `pnpm -C web exec tsc --noEmit`: cannot find name `process` |
| The same line in `web/tests-e2e/helpers/fixtures.ts` | Nothing: `pnpm -C web exec tsc --noEmit -p tests-e2e/tsconfig.json` stays clean |

### Step 1.5 - Lint edges

**REOPENED V1 CONTRACT: `web/eslint.config.js`** (D-27, D-58, D-59). Add exactly 6 allow edges (D-27 a-f), 2 use-case-to-use-case imports (D-58) and 7 per-file cast overrides (D-59). Count them in the diff: 15 entries, no existing rule loosened. Most target files do not exist yet; the positive side is proven when each file first lints clean.

| Temporary edit | Rule that must fire |
|---|---|
| `const x = 1 as Bytes;` in `web/src/ui/pages/LandingPage.tsx` | `no-restricted-syntax` brand-cast rule |
| `import type { AppFailure } from "../../workers/protocol";` in a `ui/` file | `boundaries`: `ui` may not import `workers` |
| `import { paths } from "../persistence/opfs";` in `web/src/state/capability-store.ts` | `boundaries`: `state` imports `gen` only |

### Step 1.6 - Config contracts

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/src/config/env.ts` | §15.1, D-24 | One new field. A set value that does not decode to 32 bytes throws at module load |
| 2 | `web/src/config/entitlement-public-key.ts` | §15.1 | The base64 literal of Step 0.3. Never a seed. If the secret scan of the first CI run names this line, end it with `// gitleaks:allow` (item 9) |

| Do | Expect |
|---|---|
| `pnpm dev`, console: `(await import("/src/config/entitlement-public-key.ts")).ENTITLEMENT_PUBLIC_KEYS.map(k => k.length)` | `[32]` |
| Stop; `VITE_ENTITLEMENT_TEST_PUBLIC_KEY=<test public key> pnpm dev`; same line | `[32, 32]` |
| Stop; `VITE_ENTITLEMENT_TEST_PUBLIC_KEY=abc pnpm dev`; same line | The import rejects |

### Step 1.7 - The reference clip (D-39, D-64)

```bash
C=testclips/speech_scriptA_landscape_720p.mp4
ffprobe -v error -show_entries stream=codec_name,width,height,sample_rate,channels -show_entries format=duration,size -of default=nw=1 "$C"
# codec_name=h264  width=1280  height=720  codec_name=aac  sample_rate=48000  channels=2  duration=74.70...  size<=40000000
ffprobe -v error -select_streams v:0 -show_entries stream=duration -of csv=p=0 "$C"   # 74.70...: write the value in ms into fixtures/speech/README.md
ffmpeg -v info -i "$C" -an -vf "select='eq(n,150)+eq(n,1950)',signalstats,metadata=print:key=lavfi.signalstats.YAVG" -f null - 2>&1 | grep YAVG
# two lines, both YAVG >= 230 (the two white frames)
```

### Step 1.8 - Worker probe under the production CSP (TEMPORARY, remove the same day)

Not in §5; added by this guide (item 7). Create `web/src/workers/zz-probe.worker.ts`:

```ts
(async () => {
  const out: Record<string, unknown> = {};
  try {
    const adapter = await (navigator as any).gpu.requestAdapter();
    const device = await adapter.requestDevice();
    const ctx = new OffscreenCanvas(1080, 1920).getContext("webgpu") as any;
    ctx.configure({ device, format: (navigator as any).gpu.getPreferredCanvasFormat() });
    out.webgpuCanvas = true;
  } catch (e) { out.webgpuError = String(e); }
  out.h264 = (await VideoEncoder.isConfigSupported({ codec: "avc1.640028", width: 1080, height: 1920, bitrate: 8_000_000, framerate: 30 })).supported;
  out.aac = (await AudioEncoder.isConfigSupported({ codec: "mp4a.40.2", sampleRate: 48000, numberOfChannels: 2, bitrate: 160_000 })).supported;
  try {
    const root = await navigator.storage.getDirectory();
    const h = await (await root.getFileHandle("zz-probe", { create: true })).createSyncAccessHandle();
    h.close(); await root.removeEntry("zz-probe"); out.opfsSync = true;
  } catch (e) { out.opfsError = String(e); }
  postMessage(out);
})();
```

Add one line at the end of `web/src/main.tsx`:

```ts
new Worker(new URL("./workers/zz-probe.worker.ts", import.meta.url), { type: "module" }).onmessage = (e) => console.log("probe", e.data);
```

```bash
pnpm build && pnpm -C web exec vite preview   # serves http://localhost:4173 with the production headers (V1 D-15)
```

| Do | Expect |
|---|---|
| Open `http://localhost:4173`, console | `probe {webgpuCanvas: true, h264: true, aac: true, opfsSync: true}` and no line starting "Refused to" |
| Any `...Error` key or a `false` | Stop. `webgpuError`: TE-3 is at risk, read TS §19 contingency now. `h264: false`: the next ladder entries (TE-4). Record it in `docs/v2/experiments.md` |

Remove it:

```bash
git checkout web/src/main.tsx && rm web/src/workers/zz-probe.worker.ts
git status --porcelain | grep -c "zz-probe\|main.tsx"   # 0
```

### Milestone 1

```bash
cargo metadata --no-deps --format-version 1 | node -e "let s='';process.stdin.on('data',d=>s+=d).on('end',()=>console.log(JSON.parse(s).packages.length))"   # 12
pnpm check                                   # green: fmt, clippy, deny, ESLint, two tsc runs, file tree, gen clean, copy codes
pnpm test                                    # green (no new tests yet)
git diff --stat "$(git merge-base main HEAD)" -- server crates/offcut-types crates/offcut-api-types web/src/gen web/vercel.json render.yaml clippy.toml   # (no output)
git grep -n "zz-probe" -- web                # (no output)
```

Pass: 12 members, `pnpm check` and `pnpm test` green, every "prove it by" row of Steps 1.3-1.5 fired and was undone, the probe printed four `true` values under `vite preview`, the clip shows 1280x720, H.264/AAC, 74.7 s and two white frames, and the frozen-file diff is empty.

Commit: `"V2 S1: crate skeletons, tree and lint contracts, test tooling, config keys"`.

Then open the draft pull request (item 9):

```bash
git push -u origin v2-build
gh pr create --draft --base main --title "V2: one clip in, one MP4 out" --body "See docs/v2/v2implementation.md"
gh pr checks --watch                         # the ci job: green
```

From here on, `git push` after every milestone commit and read the `ci` run before the next phase. The first run may stop at the secret scan (item 9). Do not begin Phase 2 until the commit's checks and the `ci` run pass.

---

## Phase 2 - MP4 read side (S2)

Goal: `offcut-mp4` opens, probes and validates an MP4/MOV through `RandomAccess` with all 11 rules of TS §15.2, proven by 17 inline cases and by the real reference clip.

### Step 2.1 - Dependencies

Add `thiserror` and (dev) `proptest` to `crates/offcut-mp4/Cargo.toml` through `[workspace.dependencies]`, in the pinning form the V1 lines already use. If either is new to the workspace, pin the latest stable (`cargo search <name> --limit 1`) and record it in `docs/v2/v2changelog.md` (§3.2).

```bash
cargo check -p offcut-mp4 && cargo deny check   # Finished; then either four "ok", or an error under bans
```

If `cargo deny check` stops at `bans`, it names a third-party crate that depends directly on a banned one: `proptest` uses `rand`, and its own dependencies may use `getrandom`. Add each crate it names as a wrapper of that entry in `deny.toml`, as V1 did for `reqwest` and `uuid` (§22.1), and record it in `docs/v2/v2changelog.md`. A workspace crate never becomes a wrapper here.

```bash
cargo deny check                             # advisories ok, bans ok, licenses ok, sources ok
```

### Step 2.2 - Files, in dependency order

Each depends only on the ones above it. Write the listed §6.9 cases in the same sitting as the file.

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `src/lib.rs` | §6.1 | `IoError` 3 variants, `ContainerError` 6, `MuxError` 4. `Malformed` and `Unsupported` carry a box name, never file bytes |
| 2 | `src/reader.rs` | §6.2 | `MemReader`; a read past `len()` is `OutOfBounds`. `MemSink` is not here (Step 4.6) |
| 3 | `src/boxes.rs` | §6.3, D-60 | 2 cases (header sizes; box past the limit). Bound every table count by `body length / entry size` before allocating |
| 4 | `src/sample_table.rs` | §6.4 | 5 cases (`stts` sum, `ctts` v1, no `stss`, one media edit, two media edits). The 20,000-sample bound leaves the track unresolved; it is not an error |
| 5 | `src/demux.rs` | §6.5 | 1 case (`read_video_sample` on an unresolved track), and 1 case this guide adds: a file shorter than one 8-byte box header opens as `NotIsoBmff`, not `Truncated`. §6.3 and §6.5 leave that open, and Step 3.6 expects `REJECT_CONTAINER` for a 6-byte text file. Audio `pts` may be negative; floor division to `Micros` |
| 6 | `src/probe.rs` | §6.6, D-34 | 4 cases (four rotations; `avc1.640028`; the 30,000-sample track; the `lpcm` track). `probe` never fails and never reads the expanded sample list |
| 7 | `src/validate.rs` | §6.7, D-21 | 4 cases (each of 11 rules alone; two rules; the two boundary values; `R90` swap). First failure wins in TS §15.2 order; never returns `FileSize`, `Corrupt`, `NoSpeech` |

```bash
cargo test -p offcut-mp4 --lib 2>&1 | tail -3   # test result: ok. (the 16 table rows of §6.9 and the short-file case; 0 failed)
```

### Step 2.3 - Guards and the real clip

| Temporary edit | Rule that must fire |
|---|---|
| `let _ = v[0];` in `boxes.rs` non-test code | clippy `indexing_slicing` |
| `.unwrap()` in `demux.rs` non-test code | clippy `unwrap_used` |
| `let _ = std::time::Instant::now();` in `probe.rs` | clippy `disallowed_methods` |

Count check: `validate_probe` has 11 rejecting branches; read them against TS §15.2 line by line.

Hand-built fixtures share their author's misreading of the format, so read one real file. **TEMPORARY, never commit** (the clip is not in git and CI would fail): add to `demux.rs` tests

```rust
#[test] #[ignore]
fn zz_real_clip() {
    let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../testclips/speech_scriptA_landscape_720p.mp4")).unwrap();
    let size = offcut_types::Bytes::new(bytes.len() as u64);
    let d = crate::demux::Demuxer::open(crate::reader::MemReader(bytes)).unwrap();
    let p = crate::probe::probe(&d, size);
    println!("{p:#?}\n{:#?}", crate::validate::validate_probe(&p, true));
}
```

```bash
cargo test -p offcut-mp4 --lib -- --ignored zz_real_clip --nocapture
```

| Field printed | Expect |
|---|---|
| `duration` | The millisecond value of Step 1.7, within 1 |
| `video.codec`, `codec_string` | `H264`, starts with `avc1.4d` (Main) |
| `coded_width`, `coded_height`, `rotation` | 1280, 720, `R0` |
| `is_vfr` | `true` |
| `audio` | `Aac`, `mp4a.40.2`, 48,000 Hz, 2 channels |
| `validate_probe` | `Ok(ClipInfo { .. 1280 x 720, Landscape .. })` |

A different result is a demuxer bug; fix it before going on. Then delete the test: `git grep -n zz_real_clip` prints nothing.

### Milestone 2

```bash
cargo test -p offcut-mp4 --lib               # test result: ok. 0 failed
cargo clippy -p offcut-mp4 --all-targets -- -D warnings   # Finished, no warnings
cargo deny check                             # ok
pnpm check                                   # green
git grep -n "zz_real_clip"                   # (no output)
```

Pass: the 16 rows of §6.9 and the short-file case are covered and green, clippy and deny are clean, the three guard edits fired and were undone, and the real clip printed the six expected values.

Commit: `"V2 S2: offcut-mp4 read side (demux, probe, validate)"`. Do not begin Phase 3 until this passes.

---

## Phase 3 - Resampler, core bindings, worker plumbing, ingest (S3, S4)

Goal: the browser copies a dropped file to OPFS, probes and validates it in Rust inside `media.worker`, and returns 48 kHz and 16 kHz mono PCM of exactly the right length.

### Step 3.1 - `offcut-dsp`

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-dsp/Cargo.toml` | §7 | `offcut-types`, `rubato`, `thiserror`; dev `proptest`. Pin `rubato` and record it |
| 2 | `src/lib.rs` | §7 | `DspError { Empty, NonFinite }`, unused until V3 |
| 3 | `src/resample.rs` | §7 | 4 inline cases. `output.len() == round(input.len() x to / from)` exactly; group delay trimmed |

```bash
cargo test -p offcut-dsp                     # test result: ok. 4 passed (1 of them proptest)
```

### Step 3.2 - Core bindings and the `wasm-bindgen` pair

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-wasm-core/Cargo.toml` | §4, §3.2 | Add `offcut-mp4`, `offcut-dsp`, `sha2`, `serde-wasm-bindgen`, `js-sys`, `web-sys`. Not `offcut-text` yet (Step 5.1) |
| 2 | `src/lib.rs` | §13 | Two module declarations now (`media_api`, `hash_api`); the `{ code, detail }` helper. `detail` is a variant name only |
| 3 | `src/hash_api.rs` | §13.3 | `finalize_hex` is lower-case |
| 4 | `src/media_api.rs` | §13.1, §13.2, D-53 | A rejection is a returned value, never a throw past `load-core.ts`. A short read is `IoError::Read` |

**Pair check (item 3a).** Run after `cargo check` has rewritten `Cargo.lock`:

```bash
cargo check -p offcut-wasm-core --target wasm32-unknown-unknown   # Finished
grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | tail -1           # version = "0.2.N"
wasm-bindgen --version                                            # wasm-bindgen 0.2.N   <- same N
```

| Outcome | Do |
|---|---|
| Same | Continue |
| `cargo check` cannot select a version of `wasm-bindgen`, `js-sys` or `web-sys` | The root `Cargo.toml` pins `wasm-bindgen = "=0.2.129"` exactly, so the lock cannot move by itself: a new dependency is asking for a newer one. Raise the pin, `cargo install wasm-bindgen-cli --version <new version> --locked`, and change `WASM_BINDGEN_VERSION` and `WASM_BINDGEN_SHA256` in `ci.yml` in this commit |

```bash
cargo deny check                             # if it names a third-party parent of js-sys / wasm-bindgen / web-sys: add that crate as a wrapper (§22.1), record it in v2changelog.md
```

### Step 3.3 - `load-core.ts`

Extend `CoreApi` per §13.3 **without** `normalizeTranscript` (item 5c). Correct the V1 comment on `loadCore()` (D-36).

```bash
pnpm build:wasm                              # writes web/src/wasm/pkg/core/
pnpm gen:types && bash scripts/check-gen-clean.sh   # no diff: V2 adds no shared type (§21)
pnpm -C web exec tsc --noEmit                # no output
```

| Do (`pnpm dev`, console) | Expect |
|---|---|
| `const core = await (await import("/src/wasm/load-core.ts")).loadCore(); const h = core.newSha256(); h.update(new TextEncoder().encode("abc")); h.finalizeHex()` | `"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"` |
| `core.resample(new Float32Array(48000), 48000, 16000).length` | `16000` |

### Step 3.4 - `persistence/opfs.ts`

Section §15.2. Count check: the `paths` object has 10 functions. One `as Bytes` minting line (D-59); a second one must fail lint.

| Do (dev console) | Expect |
|---|---|
| `const o = await import("/src/persistence/opfs.ts"); o.paths.modelFile("m", "../x")` | Throws |
| `o.paths.clipSource("abc")` | `"clips/abc/source"` |
| `await o.size("nope")` | `null` |

### Step 3.5 - `rpc.ts` and `pool.ts`

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/src/workers/rpc.ts` | §15.6, D-32 | Cancel resolves `{ cancelled: true }`; failure rejects `WorkerCallError`; the retryable column of TS §11.2 is one table here. No throttle, no cancel timeout (V4) |
| 2 | `web/src/workers/pool.ts` | §15.7 | **One table row now: `media`** (item 5a). `preload()` keeps its V1 signature and does not call `preloadRender()` yet (item 5b). Re-export `WorkerCallError` and `Cancelled` |

`rpc.test.ts` is V4 (§23.1); do not write it now.

### Step 3.6 - The media worker

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/src/workers/media/import.ts` | §16.2 | 4 MiB chunks; cancel check before each; never reads `file.name` |
| 2 | `web/src/workers/media/audio-decode.ts` | §16.3 | Every `AudioData` closed in `finally`; alignment uses timestamps only (INV-5) |
| 3 | `web/src/workers/media.worker.ts` | §16.1, D-34 | Top level is `serveWorker(...)` only. Size rule before any copy. Sync handle closed in `finally` |

S4's check, in `pnpm dev` (no code is added for it, so nothing needs removing):

```js
const { pool } = await import("/src/workers/pool.ts");
const [h] = await showOpenFilePicker(); const file = await h.getFile();   // pick the reference clip
const clipId = crypto.randomUUID();
const r = await pool.media.importAndProbe({ clipId, file });
const a = await pool.media.extractAudio({ clipId });
console.log(r.ok.duration, a.pcm48.length, Math.round(r.ok.duration * 48), a.pcm16.length, Math.round(r.ok.duration * 16));
```

| Do | Expect |
|---|---|
| The script above | `duration` within 1 of the Step 1.7 value; second and third numbers equal (3,585,840 for 74,705 ms); fourth and fifth equal (1,195,280) |
| DevTools, Application, Storage, OPFS: `clips/<clipId>/` | One file, `source`, the size of the clip. No name containing the dropped file's name |
| `truncate -s 500000001 /tmp/zz_big.mp4`, then `importAndProbe` with it | `{ rejected: "REJECT_FILE_SIZE" }`; no new directory under `clips/` |
| `echo hello > /tmp/zz_not.mp4`, then `importAndProbe` | `{ rejected: "REJECT_CONTAINER" }` (the short-file rule of Step 2.2) |

### Milestone 3

```bash
cargo test -p offcut-dsp -p offcut-mp4       # ok, 0 failed
pnpm build:wasm && pnpm check && pnpm build  # green; check-hosts passes
grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | tail -1 && wasm-bindgen --version   # same version twice
git diff --stat "$(git merge-base main HEAD)" -- web/src/gen web/src/workers/protocol.ts   # (no output)
```

| Check | Where | Expect |
|---|---|---|
| SHA-256 of `"abc"` | Dev console | `ba7816bf...f20015ad` |
| Import and extract | Dev console | The five numbers of Step 3.6 |
| Two rejections | Dev console | `REJECT_FILE_SIZE`, `REJECT_CONTAINER` |

Pass: the command block is green, the generated types did not change, and the browser returned a `ClipInfo` for the reference clip with `pcm48.length === Math.round(durationMs * 48)` and `pcm16.length === Math.round(durationMs * 16)`.

Commit: `"V2 S3-S4: resampler, core bindings, RPC and pool, media worker"`. Do not begin Phase 4 until this passes.

---

## Phase 4 - Model delivery and muxer (S5, S7)

Goal: the speech model sits on the asset host with a hashed manifest, downloads resumably into OPFS and verifies, and `offcut-mp4` writes a faststart MP4 that its own demuxer reads back byte for byte.

Start the uploads of Step 4.2 first; write the muxer (Step 4.6) while they run (§5, "what can run in parallel").

### Step 4.1 - `scripts/upload-assets.sh` (D-62)

Edit per §22.4: the folder argument accepts `media`, `models` or `models/<modelId>`; add a content-type row for every extension in the model folder that the table lacks.

```bash
ls <model folder> | sed 's/.*\.//' | sort -u          # the extensions the table must know (for example onnx, json, txt)
bash scripts/upload-assets.sh 2>&1 | head -5          # the usage line: use its argument order below
```

| Temporary call (no upload happens) | Rule that must fire |
|---|---|
| Folder `models/../media` | Rejected by the folder pattern, non-zero exit |
| Folder `models/ASR_EN` | Rejected (upper case, underscore), non-zero exit |
| Folder `models/asr-en-v1/extra` | Rejected (one segment only), non-zero exit |

### Step 4.2 - Hash and upload the model and the sample clip

```bash
cd <model folder>
wc -c * | tail -1                             # total <= 260000000 (TS §16.1, D-66). Larger: pick a smaller quantization before uploading
sha256sum * > /tmp/model.sha256 && cat /tmp/model.sha256   # one 64-hex line per file
cd -
```

Upload each model file to `models/asr-en-v1` and the reference clip to `media`, with the upload credentials loaded in this terminal only. Keep every printed path: it is the stored name with its hash segment.

| Uploaded | Printed path goes to |
|---|---|
| Each model file | `files[].path` in `model-manifest.json` (Step 4.3) |
| `testclips/speech_scriptA_landscape_720p.mp4` | `SAMPLE_CLIP_PATH` in `net/asset-fetch.ts` (Step 4.3) |

TE-7 re-check, on the largest model file:

```bash
curl -s -o /dev/null -w "%{http_code} %{size_download}\n" -H "Origin: http://localhost:4173" -H "Range: bytes=0-8388607" "<asset base URL>/<printed path>"   # 206 8388608
curl -sI -H "Origin: http://localhost:4173" "<asset base URL>/<printed path>" | grep -i "cache-control\|access-control-expose-headers"
# cache-control: public, max-age=31536000, immutable ; expose-headers lists Content-Range, Accept-Ranges, Content-Length
```

A 200, or a per-file size limit on the host, is a TE-7 failure: §24.1 (other candidate, or split files). Record the outcome in `docs/v2/experiments.md`.

### Step 4.3 - Manifest and `fetchAsset`

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `web/src/config/model-manifest.json` | §15.1, TS §16.1 | `modelId` `"asr-en-v1"`; `bytes` from `wc -c`, `sha256` from Step 4.2, never retyped by hand; `totalBytes` is their sum |
| 2 | `web/src/net/asset-fetch.ts` | §15.4 | `SAMPLE_CLIP_PATH`; `fetchAsset` sends `Range` and no other custom header, no retry, no `ErrorCode` |

| Do (`pnpm dev`, console) | Expect |
|---|---|
| `const a = await import("/src/net/asset-fetch.ts"); const r = await a.fetchAsset(a.SAMPLE_CLIP_PATH, { range: { start: 0, endInclusive: 1023 }, signal: new AbortController().signal }); [r.ok, r.status]` | `[true, 206]` |
| Network tab, that request | No cookie, no query string, one `Range` header |
| DevTools offline, same call | `{ ok: false, cause: "offline" }` |

### Step 4.4 - Model machine, store, downloader, manager

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `web/src/state/machines/model-machine.ts` | §18.1 | 14 (state, event) pairs. `verifying` to `downloading` must be absent |
| 2 | `web/src/state/model-store.ts` | §18.1, D-59 | One `as Bytes` line (the zero). Written only by the model manager |
| 3 | `web/src/models/download.ts` | §17.2, §17.3 | Injected deps; the part file survives every failure except a hash mismatch |
| 4 | `web/src/models/download.test.ts` | §23.3 | The 16 cases of §23.3, the manifest assertion among them |
| 5 | `web/src/models/model-manager.ts` | §17.1, D-36 | A final name exists only after the hash matched (INV-20); one `ensureReady` promise at a time |

```bash
pnpm -C web exec vitest run src/models/download.test.ts   # 16 cases passed
pnpm -C web exec eslint src/models src/state              # clean: the D-27 d, e, f edges now prove their positive side
```

| Temporary edit | Rule that must fire |
|---|---|
| `import { http } from "../net/http";` in `model-manager.ts` | `boundaries`: `models` may not import `net/http` |
| Change one hex digit of a `sha256` in `model-manifest.json`, then run the browser check below | Model store ends `failed` with `E_MODEL_HASH`; no final file and no `.part` for that file. Restore the digit |

| Do (`pnpm dev`, console) | Expect |
|---|---|
| `const m = await import("/src/models/model-manager.ts"); await m.inspect(); await m.ensureReady(p => console.log(p.done, p.total), new AbortController().signal)` (adapt to the export form of TS §16.2) | Progress lines; Network shows only 206 range requests to the asset host; resolves |
| OPFS `models/asr-en-v1/` | Plain names (`localName`), no `.part` |
| Reload; `inspect()` again, then read the model store | `ready`; zero model requests in Network |
| During a second cold download (delete `models/` first), close the tab at about 30%; reopen; `ensureReady` | First request's `Range` starts at the `.part` size, not 0 |

### Step 4.5 - `ModelDownloadPanel` and its copy

Same step for component and copy (item 5e): `ui/components/ModelDownloadPanel.tsx` (§20.2), `messages.modelDownload` and `messages.errors.E_MODEL_DOWNLOAD`, `E_MODEL_HASH`, `E_MODEL_STORAGE` (§20.3), their three removals from `COPY_PENDING` in `scripts/check-copy-codes.mjs` (D-57), and the panel's classes in `components.module.css` (§20.4). The panel is mounted in Phase 10; here it must lint and type-check.

```bash
node scripts/check-copy-codes.mjs             # green
pnpm -C web exec tsc --noEmit && pnpm -C web exec eslint src/ui src/copy   # clean
```

| Temporary edit | Rule that must fire |
|---|---|
| Put `E_MODEL_HASH` back into `COPY_PENDING` | `check-copy-codes`: a pending code already has copy (V1 D-13) |
| Type `150 MB` into the `modelDownload.body` string | Nothing fires: V1 has no tool for typed-in digits (`messages.test.ts` is V8, BP §10.1), so the rule of §20.3 holds by reading only. Read the new strings once for a digit, undo the edit, and note the gap in `v2changelog.md` |

### Step 4.6 - Muxer (S7)

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-mp4/src/mux_boxes.rs` | §6.8, D-55 | One writer per box, nothing else. No `udta`, no rotation |
| 2 | `crates/offcut-mp4/src/mux.rs` | §6.8, D-33 | `MemSink` lives here. Each `add_*` writes through the sink at once. `MOOV_RESERVE` 256 KiB |
| 3 | `crates/offcut-mp4/tests/mux_roundtrip.rs` | §23.2 | The 12 cases of §23.2, one of them proptest |

```bash
cargo test -p offcut-mp4                      # lib and mux_roundtrip: ok, 0 failed
```

Second opinion from a tool that shares no code. **TEMPORARY, never commit:** in the first round-trip case, after `finalize`, add `std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/zz_mux.mp4"), &sink.0).unwrap();`

```bash
cargo test -p offcut-mp4 --test mux_roundtrip
ffprobe -v error -show_entries stream=codec_type,codec_name,nb_frames,time_base -of csv=p=0 target/zz_mux.mp4
# video,h264,...,90 and audio,aac,...,141 with time bases 1/30000 and 1/48000 (payloads are random: decode errors are expected, structure errors are not)
```

Remove the line; `git grep -n zz_mux` prints nothing.

### Milestone 4

```bash
cargo test -p offcut-mp4 -p offcut-dsp        # ok, 0 failed
pnpm -C web exec vitest run                   # all files passed, download.test.ts with 16 cases
pnpm check && pnpm build                      # green
node -e "const m=require('./web/src/config/model-manifest.json');const s=m.files.reduce((a,f)=>a+f.bytes,0);console.log(m.modelId,s===m.totalBytes,s<=260000000)"   # asr-en-v1 true true
git grep -n "zz_mux"                          # (no output)
```

| Check | Where | Expect |
|---|---|---|
| Cold download | Dev page, Network | Only 206 responses from the asset host; model store `ready` |
| Reload | Dev page | `inspect()` gives `ready`; zero model requests |
| Resume | Dev page | First `Range` after reopening starts at the `.part` size |

Pass: the command block is green, the manifest line prints `asr-en-v1 true true`, the three browser checks hold, and the hash-mismatch edit produced `E_MODEL_HASH` and was undone.

Commit: `"V2 S5+S7: model upload, manifest, downloader and manager; MP4 muxer"`. Do not begin Phase 5 until this passes.

---

## Phase 5 - ASR spike (S6) - gate

Goal: `asr.worker` returns a word-timed `Transcript` of the reference clip in a production build with zero requests outside TS §24.1, and the first E-3 number on R1 is written down.

This phase is a gate. Its last step decides whether Phases 6-12 are worth building (§24.3).

### Step 5.1 - `offcut-text` and its binding

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-text/Cargo.toml`, `src/lib.rs` | §8 | Deps `offcut-types`, `serde` only |
| 2 | `src/tokenize.rs` | §8.2, D-42 | 3 cases. A leading `$` and a trailing `%` stay on the `Digits` token |
| 3 | `src/sentences.rs` | §8.3 | 1 case. `SENTENCE_MAX_WORDS` declared, unused |
| 4 | `src/numbers.rs` | §8.4 | 5 cases. An unparseable sequence is `None`, never a guess. Leave the conditional `dollars` form out for now |
| 5 | `src/normalize.rs` | §8.1, D-31 | 2 cases. `RawWord` is camelCase with `deny_unknown_fields`; only constructor of `Transcript` |
| 6 | `crates/offcut-wasm-core/src/text_api.rs`, third `mod` in `lib.rs`, `offcut-text` in `Cargo.toml` | §13.3 | `json_compatible()` serializer |
| 7 | `web/src/wasm/load-core.ts` | §13.3 | Add `normalizeTranscript` to `CoreApi` (deferred from Step 3.3) |

```bash
cargo test -p offcut-text                     # the 11 rows of §8.5 (the 12th is conditional): ok
pnpm build:wasm && pnpm -C web exec tsc --noEmit   # clean
node scripts/check-file-tree.mjs && cargo deny check   # green: edge wasm-core -> text is in TS §7
```

### Step 5.2 - Runtime package and the ORT copy step (D-37)

```bash
pnpm -C web add --save-exact @huggingface/transformers   # record the version in docs/v2/v2changelog.md
pnpm -C web why onnxruntime-web                          # the ONE version transformers resolves: this names /ort/<version>/
```

Edit `web/vite.config.ts` per §22.3. With pnpm the runtime is not at `web/node_modules/onnxruntime-web` (item 3d); resolve it from the package that depends on it:

```ts
import { createRequire } from "node:module";
const fromWeb = createRequire(import.meta.url);
const fromHf = createRequire(fromWeb.resolve("@huggingface/transformers"));
const ortEntry = fromHf.resolve("onnxruntime-web");   // walk up from here to the package.json whose "name" is "onnxruntime-web"; read "version"; copy its dist/*.wasm (and the .mjs loaders it needs)
```

```bash
pnpm build && ls web/public/ort/              # exactly one directory, named as the version above
ls web/public/ort/*/ | grep -c "\.wasm$"      # >= 1
git status --porcelain web/public/ort         # (no output: ignored since Step 1.3)
```

`pnpm build` is expected to fail at `check-hosts.mjs` from here until Step 5.5. That is the D-38 work, not a mistake; do not loosen the script yet.

### Step 5.3 - The ASR worker

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/src/workers/asr/word-timestamps.ts` | §16.5 | Never drops, merges or shifts a word because of a pause (INV-5) |
| 2 | `web/src/workers/asr/model-cache-adapter.ts` | §16.5, D-62 | A miss throws, never fetches; reads final names only |
| 3 | `web/src/workers/asr/whisper-runtime.ts` | §16.5, TS §16.2 | The only importer of the runtime package. Remote loading off, browser cache off, `.wasm` path `/ort/<version>/`. Write each confirmed option name into `v2changelog.md` |
| 4 | `web/src/workers/asr.worker.ts` | §16.4, D-50 | `unload()` resolves only after the session is disposed (INV-12) |
| 5 | `web/src/workers/pool.ts` | §15.7 | Add the `asr` row (item 5a) |

| Temporary edit | Rule that must fire |
|---|---|
| `import { pipeline } from "@huggingface/transformers";` in `asr.worker.ts` | The V1 restricted-import rule for the runtime (TDR-3). If nothing fires, the rule is missing: add it to `eslint.config.js` and note it in `v2changelog.md` |

### Step 5.4 - First transcript, in dev

```js
// pnpm dev console, model already in OPFS from Phase 4
const { pool } = await import("/src/workers/pool.ts");
const [h] = await showOpenFilePicker(); const file = await h.getFile();
const clipId = crypto.randomUUID();
await pool.media.importAndProbe({ clipId, file });
const { pcm16 } = await pool.media.extractAudio({ clipId });
const t0 = performance.now();
const { backend } = await pool.asr.load({ modelId: "asr-en-v1", backend: "webgpu" });
const r = await pool.asr.transcribe({ pcm16 }, { transfer: [pcm16.buffer] });
await pool.asr.unload();
console.log(backend, Math.round(performance.now() - t0), r.ok.words.length, r.ok.numbers, r.ok.words.slice(0, 12));
```

| Read | Expect | If not |
|---|---|---|
| `backend` | `webgpu` | `wasm`: note it; TE-1 needs both to run |
| Word count | About 150 to 190 (Script A read in 74.7 s) | Under 5 gives `{ rejected: "NoSpeech" }`: the runtime returned nothing; fix before continuing |
| Word times | Increasing, first under 3,000 ms, last under the clip duration | Timestamps absent: the export lacks cross-attention outputs (Step 0.4); TE-1 fails |
| `numbers` | One span for the twelve-thousand-dollar amount with `unit: Usd`, display `$12k` | Written as `12,000 dollars` or in words: add the conditional form of §8.4 and its test row now (D-42), record it in `v2changelog.md`, rebuild, rerun |
| `confidence` | Below 1.0 on some words | All 1.0: TE-2 is not wired yet; Step 5.5 |

Paste the full word list into `fixtures/speech/README.md` as the expected transcript, with the expected events (the amounts that must be found). Phase 11 asserts against it.

### Step 5.5 - TE-1 and TE-2 under the production CSP

First make the build pass. **REOPENED V1 CONTRACT: `scripts/check-hosts.mjs`** (D-38): add the optional `chunk` field, then one entry per literal the failing run names, each with its reason and the chunk pattern for the `asr.worker` chunk or `ort/<version>/`.

| Temporary edit | Rule that must fire |
|---|---|
| Paste one of the newly listed host literals as a string constant into `web/src/main.tsx` | `pnpm build`: `check-hosts` fails on that literal in the main chunk |
| Remove the `chunk` field from one new entry's match by pointing it at a non-matching pattern | `pnpm build`: fails on the worker chunk |

Then the spike hook. **TEMPORARY, never commit, remove the same day:** create `web/src/zz-spike.ts`

```ts
import { pool } from "./workers/pool";
import * as modelManager from "./models/model-manager";
(globalThis as Record<string, unknown>).zz = { pool, modelManager };
```

and add `import "./zz-spike";` as the last line of `web/src/main.tsx`.

```bash
pnpm build:wasm && pnpm -C web exec vite build && pnpm -C web exec vite preview   # http://localhost:4173, production headers
```

| Do (Chrome at `http://localhost:4173`, DevTools open, Network "Preserve log") | Expect |
|---|---|
| `await zz.modelManager.inspect(); await zz.modelManager.ensureReady(() => {}, new AbortController().signal)` | Model downloads again (new origin); asset-host GETs only. This uses the CORS origin of Step 0.4 |
| Clear the Network log. Run the Step 5.4 script with `zz.pool` in place of the import | A transcript. Network shows requests to `localhost:4173` only (`/ort/<version>/...` and app chunks). **Zero** rows for any other host |
| Console | No line starting "Refused to" (CSP), no `blob:` worker error |
| Same script with `backend: "wasm"` | A transcript; `backend` is `wasm`; same network result; note the thread count |
| Compare timings with token probabilities on and off (TE-2) | Probabilities present at no more than 10% extra time. Otherwise `confidence` stays 1.0 (§24.1) |
| With probabilities on: the `confidence` of the words of the dollar span (`r.ok.numbers` gives their index range in `r.ok.words`) | Each at least 0.80. The NumberReveal score is 1.0 times the lowest confidence of its anchor words, against a threshold of 0.80 (§9.3). Below it no event is created and V2 shows nothing; D-42 covers how the amount is written, not this. Stop and decide before Phase 6, and record the choice as a V2 decision in §2 and in `experiments.md`: either TE-2 is read as failed for V2 and every word gets `Confidence(1.0)`, or the clip is recorded again. The threshold is not lowered (TS §17.4) |

Any request to another host, or a CSP refusal, is a TE-1 failure: stop, §24.1 (second runtime behind the same `whisper-runtime.ts` interface). Do not add a CSP host.

### Step 5.6 - First E-3 reading on R1

Copy `web/dist` to R1's clone (`web/dist`), then on R1:

```bash
pnpm install --frozen-lockfile && pnpm -C web exec vite preview   # http://localhost:4173
```

**Since D-67 this step is not run:** the founder's reading of 2026-10-09 (about 150,000 ms on an R1-class laptop) stands for E-3 here, and it is in `docs/v2/experiments.md`. The step as written, for a later reading: download the model once with the hook, then run the Step 5.4 script five times (reload between runs; it times `load` + `transcribe`, D-35). Write to `docs/v2/experiments.md`: the five values, the median, the median normalised to 60 s (`x 60000 / 74705`), backend, Chrome version, date.

| Median on R1 (reference clip) | Do |
|---|---|
| At most 25,000 ms | Continue |
| 25,000 to 180,000 ms (D-67; was 50,000) | Continue; record the miss |
| Over 180,000 ms (D-67; was 50,000) | **Stop.** §24.3: smaller model, new manifest (repeat Steps 4.2-4.4), re-measure |

Remove the hook on the development machine:

```bash
git checkout web/src/main.tsx && rm web/src/zz-spike.ts
git status --porcelain | grep -c "zz-spike\|main.tsx"   # 0
```

### Milestone 5

```bash
cargo test -p offcut-text                     # ok, 0 failed
pnpm check                                    # green
pnpm build                                    # green, check-hosts included, runtime in web/dist (D-38)
ls web/dist/ort/                              # one version directory
git grep -n "zz-spike\|globalThis as Record" -- web/src   # (no output)
grep -c "TE-1\|TE-2\|E-3" docs/v2/experiments.md          # >= 3
```

| Check | Where | Expect |
|---|---|---|
| TE-1 | `vite preview`, Network | Zero requests outside the app origin while loading and transcribing, both backends |
| Dollar amount | Transcript | A `Usd` span with display `$12k` (directly or through the D-42 form), its words at a confidence of 0.80 or more, or the decision of Step 5.5 recorded |
| E-3 | `experiments.md` | The founder's reading (D-67), at most 180,000 ms, the column it fell in |

Pass: the command block is green, TE-1 and TE-2 each have a recorded outcome (pass or fallback taken), the transcript carries the dollar amount as a `Usd` quantity, and the R1 reading of D-67 is recorded and not above 180,000 ms.

Commit: `"V2 S6: offcut-text, ASR worker, ORT copy step, chunk-scoped host entries; TE-1, TE-2, first E-3"`. Do not begin Phase 6 until this passes.

---

## Phase 6 - Scene, detection, entitlement (S8, S9)

Goal: three pure crates turn a transcript into placed NumberReveal events and per-frame display lists, and turn a signed token into an `ExportProfile`, with the token format proven against the TypeScript minter.

### Step 6.1 - Fonts and `parley`

Download the three variable fonts named in §4 from their projects' official releases into `crates/offcut-scene/assets/fonts/`, with the exact file names of §4, and write `LICENSES.md` from the licence file shipped with each (expected: SIL OFL 1.1 for all three; confirm in the files).

```bash
git ls-files -o -c --exclude-standard | grep -i "\.\(ttf\|otf\|woff2\?\)$"   # exactly 3 lines, all under crates/offcut-scene/assets/fonts/
cargo search vello --limit 1 && cargo search parley --limit 1               # latest stable of each
cargo info vello@<version> | grep -i "wgpu\|peniko\|skrifa"                  # the versions vello pins: parley must agree on the shared ones
```

Add `parley` to `[workspace.dependencies]` and to `offcut-scene`; write the `vello`, `wgpu` and `parley` versions you will use into `v2changelog.md` now, so Phase 7 pins the same set.

### Step 6.2 - `offcut-scene` (S8)

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `src/safe_area.rs` | §11.2, TS §19.1 | Constants verbatim: insets 250, 420, 60, 120; zones 860-1180 and 1220-1500 |
| 2 | `src/easing.rs`, `src/anim.rs` | §11.2 | Time computed from `t`, never accumulated |
| 3 | `src/display_list.rs` | §11.4, D-29 | `FontId` has 4 members in the order given; the order is frozen |
| 4 | `src/styles.rs` | §11.2 | Three explicit arms; `Bold` and `Tech` return the Clean row |
| 5 | `src/fonts.rs`, `src/layout.rs` | §11.3 | `include_bytes!`; a missing glyph falls to `NotoEmoji`, then is skipped |
| 6 | `src/framing.rs` | §11.6 | 2 cases (1920x1080 gives `x: 656.25, w: 607.5`; portrait is the whole frame). Offset forced to 0.0 on one marked line |
| 7 | `src/captions.rs` | §11.5 | 3 cases (12 words; the gap; one active run). Chunks never overlap |
| 8 | `src/events/number_reveal.rs`, `src/events/mod.rs` | §11.7 | 2 cases (the `$10k` timeline; the unfittable display). All four `EventParams` arms matched explicitly. Count-up runs shaped at layout time, at most 16 |
| 9 | `src/lib.rs` | §11.1 | 4 cases (frame counts 1,800 and 601; safe area for every command; two builds equal; width 540 halves coordinates) |

```bash
cargo test -p offcut-scene                    # the 11 rows of §11.8: ok
cargo clippy -p offcut-scene --all-targets -- -D warnings   # clean
node scripts/check-file-tree.mjs && cargo deny check        # green: scene -> text is the allowed pair of Step 1.3
```

| Temporary edit | Rule that must fire |
|---|---|
| `use std::collections::HashMap; let _m: HashMap<u8, u8> = HashMap::new();` in `captions.rs` | clippy `disallowed_types`. If nothing fires, `clippy.toml` lacks the entry: record the gap in `v2changelog.md` (the file is unchanged in V2, §22.1) and rely on review |
| `wgpu` as a dependency of `offcut-scene` | `cargo deny check` and the pure-crate check of `check-file-tree.mjs` |
| Copy any `.ttf` to `web/public/zz.ttf` | The font `grep` of Step 6.1 prints 4 lines |

### Step 6.3 - `offcut-detect` (S9)

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `src/config.rs` | §9.1 | Four thresholds 0.85, 0.85, 0.80, 0.80; scoring 0.50, 0.30, 0.20, 0.10; lead 150; hold 1,400 |
| 2 | `src/event_id.rs` | §9.2, D-47 | 2 cases. 9 input bytes: kind `u8`, two little-endian `u32` |
| 3 | `src/number.rs` | §9.3 | Label `None`. Clamp to 1.0, then multiply by the minimum anchor confidence |
| 4 | `src/lib.rs` | §9.2, D-49 | 7 cases. Nothing below the threshold is ever constructed (INV-6) |

```bash
cargo test -p offcut-detect                   # the 9 rows of §9.4: ok
```

### Step 6.4 - `offcut-entitlement` (S9)

Add `ed25519-dalek` with `default-features = false` and only the features verification needs, `base64`, `serde_json` (D-54).

```bash
cargo tree -p offcut-entitlement -e normal | grep -ci "rand\|getrandom"   # 0
```

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `src/lib.rs`, `src/claims.rs` | §10.1, TS §10.6 | `TokenError` has 5 variants; `deny_unknown_fields` |
| 2 | `src/token.rs` | §10.2, D-25, D-60 | Length over 2,048 is `Format` before any decoding; unpadded base64url only |
| 3 | `src/verify.rs` | §10.3 | Expiry is not checked here; an invalid key is skipped |
| 4 | `src/profile.rs` | §10.4, D-26 | Only builder of `ExportProfile`. Sizes from `offcut_types::limits` |

Write 8 of the 9 rows of §10.5 now; the cross-language vector needs Step 6.5.

### Step 6.5 - Token helpers and the cross-language vector

Add to `web/tests-e2e/helpers/fake-api.ts` (§23.4): `TEST_ENTITLEMENT_SEED` (the test seed of Step 0.5, as bytes), `testPublicKeyBase64`, `mintEntitlementToken`, `seedEntitlement`.

Mint the vector with fixed claims so the constant never changes. **TEMPORARY:** `web/tests-e2e/zz-mint.spec.ts`

```ts
import { test } from "@playwright/test";
import { mintEntitlementToken, testPublicKeyBase64 } from "./helpers/fake-api";
test("zz mint", () => {
  console.log("TOKEN", mintEntitlementToken({ plan: "creator", iat: 1760000000, exp: 1760604800, periodEnd: 1762592000 }));
  console.log("PUBKEY", testPublicKeyBase64());
});
```

```bash
pnpm -C web exec playwright test zz-mint --project non-media   # prints TOKEN <a>.<b> and PUBKEY <44 chars>; needs an existing web/dist for the preview server
rm web/tests-e2e/zz-mint.spec.ts
```

`mintEntitlementToken` also sets `sub` and `freeExportsRemaining`; if `sub` is random by default, give the helper a fixed value for this call so the claims are fully known. Paste the token, the claims and the 32 public-key bytes into the `offcut-entitlement` test as constants (row 1 of §10.5).

| Check | Expect |
|---|---|
| `PUBKEY` | Equals the test public key noted in Step 0.5. Different: the seed constant was mistyped |
| `cargo test -p offcut-entitlement` | The 9 rows of §10.5: ok |

| Temporary edit | Rule that must fire |
|---|---|
| Change one character in the pasted token's first segment | The vector test fails with `Signature` |
| `git grep -n "<first 16 hex chars of the test seed>" -- web/src crates` | No output: the seed exists only in `web/tests-e2e/helpers/fake-api.ts` |

### Milestone 6

```bash
cargo test -p offcut-scene -p offcut-detect -p offcut-entitlement   # ok: 11 + 9 + 9 rows, 0 failed
cargo clippy -p offcut-scene -p offcut-detect -p offcut-entitlement --all-targets -- -D warnings   # clean
pnpm check                                    # green (tests-e2e tsc included)
git ls-files | grep -ci "\.\(ttf\|otf\|woff2\?\)$"                  # 3
git ls-files web/tests-e2e | grep -c zz-                            # 0
bash scripts/check-gen-clean.sh               # no diff: EntitlementClaims and the display-list types are not generated (§21)
```

Pass: 29 spec rows are covered and green, the cross-language vector verifies and fails when one character changes, exactly three font files are tracked, and the generated types are unchanged.

Commit: `"V2 S8-S9: offcut-scene, offcut-detect, offcut-entitlement, token helpers"`. Do not begin Phase 7 until this passes.

---

## Phase 7 - Renderer and render bindings (S10)

Goal: `offcut_render.wasm` builds beside `offcut_core.wasm`, exports the session, detection, profile and muxer bindings, and the whole workspace is clippy-clean natively.

### Step 7.1 - Pin the GPU stack and settle the `web-sys` flag

```bash
cargo info vello@<version from Step 6.1> | grep -i wgpu      # the wgpu version to pin: exactly this one (item 3b)
```

Add `vello` and `wgpu` (default features off, WebGPU backend only) to `[workspace.dependencies]`; add the `web-sys` features `offcut-render` needs (`VideoFrame`, `OffscreenCanvas` and the GPU types `wgpu` does not already enable).

Then settle the flag (item 4). The call that hands a `VideoFrame` to `wgpu` exists on wasm32 only, so in `video_pass.rs` it sits behind `#[cfg(target_arch = "wasm32")]`: `cargo clippy --workspace --all-targets` and `cargo test --workspace` compile this crate natively.

```bash
V=$(grep -A1 '^name = "web-sys"$' Cargo.lock | tail -1 | tr -dc '0-9.')   # the locked web-sys: 0.3.106 on 2026-10-08
grep -B16 "pub type VideoFrame;" ~/.cargo/registry/src/*/web-sys-$V/src/features/gen_VideoFrame.rs | grep -c web_sys_unstable_apis   # 0: the type is not gated
cargo check -p offcut-render --target wasm32-unknown-unknown   # run it again once video_pass.rs imports the frame (Step 7.2)
```

| Outcome | Do |
|---|---|
| The count is 0, and the wasm32 check passes with `video_pass.rs` written | No flag, no `.cargo/config.toml`. Continue |
| The wasm32 check fails in `video_pass.rs` on a `wgpu` item that mentions `VideoFrame` | The gate is in `wgpu`, and only the bundle needs the flag. In `scripts/build-wasm.sh` change `RUSTFLAGS="-C target-feature=+simd128"` to `RUSTFLAGS="-C target-feature=+simd128 --cfg=web_sys_unstable_apis"`. That line is the only place: an environment `RUSTFLAGS` replaces `build.rustflags` of `.cargo/config.toml` entirely. For the `cargo check --target wasm32-unknown-unknown` lines of this guide, set the same two flags in `RUSTFLAGS` for that command. Record it in `v2changelog.md` |
| The count is 1 or more: another `web-sys` got locked, and it gates the type itself | The signature of `Renderer::render` then needs the flag on every target. Do the row above, and also create `.cargo/config.toml` with `[build]` / `rustflags = ["--cfg=web_sys_unstable_apis"]` for native clippy and tests. No workflow sets `RUSTFLAGS` (read on 2026-10-08); if one does by then, append the flag there too. Add the file to the TS §5 tree and correct §22.4 ("for this bundle only") in this commit; record it in `v2changelog.md` |

```bash
cargo tree -d -p offcut-render | grep -c "^wgpu \|^peniko \|^skrifa "   # 0: no duplicated GPU or font crate
cargo deny check                              # add the wrappers it names (for example vello for wgpu), record each (§22.1)
```

### Step 7.2 - `offcut-render`

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `src/gpu.rs` | §12 | One adapter, one device; device-lost sets a flag the next `render` reports |
| 2 | `src/shaders/video.wgsl`, `src/video_pass.rs` | §12, TS §19.7 | Axes swapped for `R90`/`R270`; `crop` is in display coordinates |
| 3 | `src/vello_backend.rs` | §12 | Glyph stroke, then the fill over it (D-68); fonts by `FontId` from `offcut-scene`'s bytes |
| 4 | `src/composite.rs` | §12 | Premultiplied-alpha blend of the overlay |
| 5 | `src/lib.rs` | §12, D-61 | `pub use offcut_scene as scene;`. Never closes a `VideoFrame`. `RenderError` has 6 variants |

No V2 test runs this crate natively (§12). Verification is compilation on both targets:

```bash
cargo check -p offcut-render --target wasm32-unknown-unknown   # Finished
cargo clippy -p offcut-render --all-targets -- -D warnings     # clean (native)
```

### Step 7.3 - `offcut-wasm-render`

`Cargo.toml`: `offcut-render`, `offcut-detect`, `offcut-mp4`, `offcut-entitlement`, `offcut-types`, the binding crates of D-54; **not** `offcut-scene` (D-61).

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `src/lib.rs` | §13, §13.4 | V1 panic hook; the `{ code, detail }` helper; `detail` is a variant name only |
| 2 | `src/detect_api.rs` | §13.4 | `DetectorConfig::default()`; no `redetect_sentence` |
| 3 | `src/profile_api.rs` | §13.4 | Non-32-byte array elements skipped; every `TokenError` is `E_ENTITLEMENT_INVALID` |
| 4 | `src/mux_api.rs` | §13.4 | `JsMuxSink` calls `sink.writeAt(offset, data)`; a throw is `IoError::Write` |
| 5 | `src/session.rs` | §13.5, D-30 | Names scene items through `offcut_render::scene`. Four extra methods make it a `DemuxerHandle`. Closes no `VideoFrame` |

| Temporary edit | Rule that must fire |
|---|---|
| `offcut-scene` as a dependency of `offcut-wasm-render` | `node scripts/check-file-tree.mjs` (undeclared edge) and `cargo deny check` (`offcut-scene` ban) |
| `offcut-dsp` as a dependency of `offcut-wasm-render` | `node scripts/check-file-tree.mjs`: edge not in TS §7 |
| `log::info!("{token}")` or `format!("{token}")` into an error `detail` in `profile_api.rs` | No tool catches this: read `profile_api.rs` once for any use of the token besides `verify_token`, and write "checked" in `v2changelog.md` |

### Step 7.4 - Bundle, loader, preload

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `scripts/build-wasm.sh` | §22.4 | `BUNDLES` gains `offcut-wasm-render:render` |
| 2 | `web/src/wasm/load-render.ts` | §15.5 | Only instantiation site of `offcut_render.wasm`; main thread calls `preloadRender` only |
| 3 | `web/src/workers/pool.ts` | §15.7 | `preload()` now also calls `preloadRender()` (deferred from Step 3.5) |

```bash
pnpm build:wasm                               # both bundles
ls web/src/wasm/pkg/core/*.wasm web/src/wasm/pkg/render/*.wasm   # two files
grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | tail -1 && wasm-bindgen --version   # same version (item 3a, again: wgpu and vello can move it)
ls -l web/src/wasm/pkg/render/*.wasm | awk '{print $5}'          # write the size into v2changelog.md
pnpm -C web exec tsc --noEmit                 # clean
```

### Step 7.5 - Host literals in the render bundle (D-38)

```bash
pnpm build                                    # expected to fail at check-hosts on literals inside offcut_render_bg-*.wasm
```

Add one `chunk`-scoped entry per literal, each with its reason (font name table, dependency error text).

| Temporary edit | Rule that must fire |
|---|---|
| Point one new entry's `chunk` at `offcut_core_bg-` | `pnpm build`: `check-hosts` fails on the render bundle |

### Milestone 7

```bash
cargo clippy --workspace --all-targets -- -D warnings   # clean
cargo test --workspace                        # ok, 0 failed
pnpm build:wasm                               # writes pkg/core and pkg/render
pnpm check                                    # green
pnpm build                                    # green: check-hosts passes with the render bundle in web/dist
ls web/dist/assets | grep -c "offcut_render_bg-.*\.wasm$"   # 1
```

| Check | Where | Expect |
|---|---|---|
| Preload | `pnpm dev`, Network | `offcut_render_bg` and `offcut_core_bg` both fetched once at app start; `crossOriginIsolated` still `true` |

Pass: workspace clippy and tests are clean with the same flags CI uses, both bundles exist, `wasm-bindgen` CLI and crate versions are equal, and `pnpm build` passes `check-hosts` with the render bundle present.

Commit: `"V2 S10: offcut-render, offcut-wasm-render, render bundle and loader"`. Do not begin Phase 8 until this passes.

---

## Phase 8 - Render and encode spike (S11) - gate

Goal: the reference clip exports to a 1080x1920 MP4 in a production build, an independent verifier accepts it on checks 1-6, and the first E-4 number on R1 is written down.

### Step 8.1 - The verifier first

Write `verify/verify_mp4.py` (§23.9, TS §27.2), `verify/requirements.txt`, `verify/README.md`, and the root `verify` script (§22.2). It must be able to fail before it is trusted to pass:

```bash
python -m pip install -r verify/requirements.txt
python verify/verify_mp4.py testclips/speech_scriptA_landscape_720p.mp4 --profile creator --expected-duration-ms 74705; echo "exit=$?"
# FAIL 2 (1280x720), FAIL 3 (variable frame gaps) and FAIL 5 (the clip's audio track, 74.518 s, is 187 ms shorter than its video track);
# a final line listing checks 7-11 as not implemented; exit=1
ffmpeg -v error -y -i testclips/speech_scriptA_landscape_720p.mp4 -t 2 -c copy -movflags -faststart /tmp/zz_nofast.mp4
python verify/verify_mp4.py /tmp/zz_nofast.mp4 --profile creator --expected-duration-ms 2000 | head -1   # FAIL 1: moov after mdat
git grep -n "import " verify/verify_mp4.py | grep -c "crates\|web"   # 0
```

### Step 8.2 - Encoders, sink, source, loop, worker

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/src/workers/render/encoders.ts` | §16.9, D-26 | Add 3 names only: `pickVideoConfig`, `ENCODE_QUEUE_MAX = 4`, `AAC_PRIMING_SAMPLES = 0`. V1 constants untouched |
| 2 | `web/src/workers/render/opfs-sink.ts` | §16.9, TS §21.1 | One sync handle per file; `abort` removes the file |
| 3 | `web/src/workers/render/video-source.ts` | §16.7, D-30, D-45 | `liveFrames.count` incremented at every obtain, decremented at every `close()` |
| 4 | `web/src/workers/render/export-loop.ts` | §16.8, TS §21.3 | Every frame `0..N` rendered and encoded exactly once; `vf.close()` in `finally`; audio padded, never trimmed |
| 5 | `web/src/workers/render.worker.ts` | §16.6 | Handlers `openSession`, `detect`, `setScene`, `attachPreview`, `exportClip`, `closeSession`. `exportClip` restores the preview (§16.8, step 7) only when a preview canvas was attached: the spike of Step 8.3 attaches none, and §16.8 does not say what happens then. The three preview handlers wait for Step 9.4 (item 5d) and answer `E_INTERNAL` until then, like `redetectSentence` and `previewSeek` (D-32) |
| 6 | `web/src/workers/pool.ts` | §15.7 | Add the `render` row with its one-way and during-preview lists: the table now has 3 rows |

```bash
pnpm -C web exec tsc --noEmit && pnpm -C web exec eslint src/workers   # clean: D-27 b and c prove their positive side
grep -c "8_000_000\|8000000" web/src/workers/render/encoders.ts crates/offcut-entitlement/src/profile.rs   # each file: >= 1, and the two values are equal (D-26)
```

| Temporary edit | Rule that must fire |
|---|---|
| `import { ENTITLEMENT_PUBLIC_KEYS } from "../config/entitlement-public-key";` in `media.worker.ts` | `boundaries`: only `render.worker.ts` may import it (D-27 c) |
| Remove the `vf.close()` in `export-loop.ts` and run Step 8.3 | `exportClip` fails with `E_INTERNAL` (live-frame counter, D-45). Restore it |

### Step 8.3 - Spike export in a production build

Mint a long-lived Creator token with the Step 6.5 temporary spec (`exp` and `periodEnd` in the future; delete the spec afterwards). **TEMPORARY, never commit:** restore the `zz-spike.ts` hook of Step 5.5.

```bash
export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=<test public key>     # this terminal only (item 8)
pnpm build:wasm && pnpm -C web exec vite build && pnpm -C web exec vite preview
```

```js
// http://localhost:4173 console; model already cached on this origin since Step 5.5
const p = zz.pool, clipId = crypto.randomUUID(), exportId = crypto.randomUUID();
const [h] = await showOpenFilePicker(); const file = await h.getFile();
const info = (await p.media.importAndProbe({ clipId, file })).ok;
const { pcm48, pcm16 } = await p.media.extractAudio({ clipId });
await p.asr.load({ modelId: "asr-en-v1", backend: "webgpu" });
const transcript = (await p.asr.transcribe({ pcm16 }, { transfer: [pcm16.buffer] })).ok; await p.asr.unload();
const prosody = { per_word: transcript.words.map(() => ({ energy_z: 0, pitch_z: 0 })) };
await p.render.openSession({ clipId, clipInfo: info });
const events = await p.render.detect({ transcript, prosody });
await p.render.setScene({ transcript, events, edit: /* EditState default, as in gen/domain.ts */ EDIT, profile: "preview" });
const t0 = performance.now();
const r = await p.render.exportClip({ exportId, entitlementToken: "<token>", out48: pcm48 }, { transfer: [pcm48.buffer] });
console.log(events.length, Math.round(performance.now() - t0), r.stageTimings, r.opfsPath);
const f = await (await (await (await navigator.storage.getDirectory()).getDirectoryHandle("exports")).getFileHandle(exportId + ".mp4")).getFile();
Object.assign(document.createElement("a"), { href: URL.createObjectURL(f), download: "zz_spike.mp4" }).click();
```

| Read | Expect |
|---|---|
| `events.length` | At least 1 |
| Console | No "Refused to" line; no `E_*` rejection |
| A wrong token (change one character) | Rejects with `E_ENTITLEMENT_INVALID`; `exports/` gains no file |

```bash
python verify/verify_mp4.py ~/Downloads/zz_spike.mp4 --profile creator --expected-duration-ms <duration from Step 1.7>; echo "exit=$?"
# PASS 1 .. PASS 6, exit=0. Frame count is ceil(74705 x 30 / 1000) = 2242
```

Open the file in a player: captions sit in the lower third, the `$12k` reveal counts up when the amount is spoken, and the white frame near 4.99 s is white in the output at the same second (a sync check against a frame you can see).

| Verifier line | Likely cause |
|---|---|
| `FAIL 3` (frame count or deltas) | A frame skipped or repeated in `runExport`; `t = n x 1000 / 30` not integer |
| `FAIL 5` (audio vs video) | Priming not edited: Step 8.4 sets `AAC_PRIMING_SAMPLES` |
| `FAIL 1` | `moov` written after `mdat`: `finalize` did not patch the reserved region |
| `E_MUX` at run time | Chunks arrived out of frame order (V5 `ctts` work, §16.8): take the next ladder entry in TE-4 |

### Step 8.4 - TE-3 and TE-4

Both are read on R1 alone (item 12). Capture method, hidden tab and priming can be tried on the development machine first; the values that are recorded are R1's, taken in the R1 session of Step 8.5 with the same temporary build, before the hook is removed.

| Experiment | Do | Record in `docs/v2/experiments.md` |
|---|---|---|
| TE-3 capture | Export with method A (`new VideoFrame(canvas)`) and with method B (texture readback); compare ten sampled frames between the two files; inspect one captioned frame by eye | The method chosen; name it in `export-loop.ts` and TS §21.4 |
| TE-3 speed | Time a render-only pass (no encode) over all 2,242 frames | Frames per second; pass is at least 30 on R1 |
| TE-3 hidden | Start an export, switch to another tab until it ends, verify the file | PASS 1-6 with the tab hidden |
| TE-4 ladder | Log the entry `pickVideoConfig` returns for 1080x1920 and for 720x1280 | The entry on R1; any reorder also goes to TS §21.2 |
| TE-4 priming | `ffprobe -v error -select_streams a:0 -show_entries stream=start_time,start_pts -show_entries packet=pts,duration -read_intervals "%+#3" -of default=nw=1 zz_spike.mp4`, and the first audio chunk's timestamp logged in the loop; decode and measure leading silence | The priming sample count. Set `AAC_PRIMING_SAMPLES` to it; re-export; check 5 passes. It is the count of the Windows encoder: note in `experiments.md` that no other platform was measured (item 12) |

If TE-4 changes a bitrate, change `encoders.ts` and `profile.rs` together (D-26) and rerun the `grep` of Step 8.2.

### Step 8.5 - First E-4 reading on R1, then remove the hook

On R1, with the `web/dist` of Step 8.3: take the TE-3 and TE-4 readings of Step 8.4, then run the export three times, read `r.stageTimings` (`render_encode`, `mux`); verify one file on R1 with `verify_mp4.py`. Write the values, the median and the 60 s normalisation to `experiments.md`.

| `render_encode` median on R1 | Do |
|---|---|
| At most 112,000 ms | Continue |
| 112,000 to 187,000 ms | Continue; record the miss |
| Over 187,000 ms | **Stop.** §24.3: Canvas2D overlay path; 60 s and 30 fps cap |

`mux` budget: 2,500 ms (§6.8). Then, on the development machine:

```bash
git checkout web/src/main.tsx && rm -f web/src/zz-spike.ts web/tests-e2e/zz-mint.spec.ts
git status --porcelain | grep -c "zz-"        # 0
unset VITE_ENTITLEMENT_TEST_PUBLIC_KEY
```

### Milestone 8

```bash
pnpm check                                    # green
pnpm build                                    # green (built WITHOUT the test key)
grep -rl "<test public key>" web/dist | wc -l # 0
python verify/verify_mp4.py ~/Downloads/zz_spike.mp4 --profile creator --expected-duration-ms <duration>; echo "exit=$?"   # six PASS lines, exit=0
grep -n "AAC_PRIMING_SAMPLES" web/src/workers/render/encoders.ts   # a measured value, with a TE-4 comment
git grep -n "zz-spike\|zz_spike" -- web       # (no output)
grep -c "TE-3\|TE-4\|E-4" docs/v2/experiments.md                   # >= 3
```

Pass: the verifier failed on the two bad files of Step 8.1 and passes checks 1-6 on a spike export made after `AAC_PRIMING_SAMPLES` was set, TE-3 and TE-4 are recorded with the chosen capture method and ladder entry, the R1 `render_encode` median is recorded and not above 187,000 ms, and no build output contains the test key.

Commit: `"V2 S11: render worker, export loop, encoders, verifier; TE-3, TE-4, first E-4"`. Do not begin Phase 9 until this passes.

---

## Phase 9 - State and use-cases (S12)

Goal: one call to `importClip` takes a file to a `ready` clip store, and one call to `startExport` takes a seeded token to a downloaded MP4, with every state change going through a machine table.

### Step 9.1 - Machines and stores

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/src/state/machines/clip-machine.ts` | §18.2, D-18 | 17 (state, event) pairs. Absent on purpose: `rejected` to `processing`, `importing` to `processing`, `processing` or `ready` to `importing` |
| 2 | `web/src/state/machines/export-machine.ts` | §18.4, D-18 | 16 pairs. Absent: `rendering` to `rendering`, `done` to `rendering` |
| 3 | `web/src/state/clip-store.ts` | §18.2, D-51 | `reset()` drops `out48`; feed lines are appended, never dropped |
| 4 | `web/src/state/preview-store.ts` | §18.3, D-19 | Table inside the store; `detached` to `playing` absent; seek events exist, never fired |
| 5 | `web/src/state/export-store.ts` | §18.4 | `unavailable` flag; `cache_hit`, `cancel`, `cancel_done`, `retry` exist, never fired |

Machine tests are V4 and V5 (§23.1). Count the pairs against §18 by reading, then:

| Do (`pnpm dev`, console) | Expect |
|---|---|
| Call the clip store's `accepted(...)` action while the status is `idle` | `transition()` rejects the pair the way V1's capability store does (INV-16); the status stays `idle` |
| Export store: `start(id)`, `clear()`, `encoded()`, `finalized()`, `saved()` | `gating`, `rendering`, `muxing`, `saving`, `done` |

```bash
pnpm -C web exec tsc --noEmit && pnpm -C web exec eslint src/state   # clean: type-only imports of workers/protocol.ts are allowed (D-27 f)
```

### Step 9.2 - Blockers and the copy check (D-20, D-57)

One step, four edits: `web/src/state/blockers.ts` (§18.5); `scripts/check-copy-codes.mjs` reads `BLOCKER_CODES` (§22.4); `messages.blockers` gains 4 keys (§20.3); `COPY_PENDING` gains the other 4 codes.

```bash
grep -o '"B_[A-Z_]*"' web/src/state/blockers.ts | sort -u | wc -l   # 8
node scripts/check-copy-codes.mjs             # green
```

| Temporary edit | Rule that must fire |
|---|---|
| Delete `messages.blockers.B_PIPELINE_BUSY` | `check-copy-codes`: blocker code without copy or pending entry |
| Add `B_UNSUPPORTED` to `COPY_PENDING` | `check-copy-codes`: pending code already has copy |
| Add a ninth code to `BLOCKER_CODES` | `check-copy-codes`: no copy, no pending entry |

### Step 9.3 - `persistence/entitlement-repo.ts` (D-23)

| Do (`pnpm dev`, console) | Expect |
|---|---|
| `const e = await import("/src/persistence/entitlement-repo.ts"); await e.get()` | `undefined` |
| `await e.put("a.b"); await e.get()` | `{ token: "a.b", storedAt: ... }` |
| DevTools, IndexedDB, `offcut`, `entitlement` | One record, key `current`, `schemaVersion: 1` |
| Delete the record in DevTools | `get()` is `undefined` again |

### Step 9.4 - Preview loop and the three preview handlers

`web/src/workers/render/preview-loop.ts` (§16.7, TS §20.2), then `previewPlay`, `previewClock` and `previewPause` in `render.worker.ts` (§16.6; deferred from Step 8.2). The loop never drives time; the audio clock does. After `previewPlay` returns the live-frame counter must be 0 (D-45).

```bash
grep -c "previewPlay\|previewClock\|previewPause" web/src/workers/render.worker.ts   # >= 3
grep -c "redetectSentence\|previewSeek" web/src/workers/render.worker.ts             # the two that still answer E_INTERNAL
```

### Step 9.5 - Use-cases

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/src/usecases/control-preview.ts` | §19.4, D-58, D-59 | One `AudioContext`; one `as TimeMs` line; `previewClock` goes through `notify` every 250 ms |
| 2 | `web/src/usecases/run-pipeline.ts` | §19.3, D-22, D-35 | 12 steps. `unload()` always runs before the render session opens (INV-12). The two stand-ins carry a `V3:` comment naming §25 |
| 3 | `web/src/usecases/import-clip.ts` | §19.2, D-40, D-48 | 9 steps. Never reads `file.name`. The `clips/` sweep carries a `V7:` comment. One `as ClipId` line |
| 4 | `web/src/usecases/start-export.ts` | §19.5, D-43, D-52 | 11 steps. `planOf` feeds analytics only. The `exports/` sweep carries a `V7:` comment. One `as ExportId` line |
| 5 | `web/src/usecases/start-app.ts` | §19.1 | One line: `void modelManager.inspect()` as step 8 |

The eight analytics events of §1.1 are tracked here, not in components. No `client_error` for a pipeline failure (V4).

```bash
git grep -n "V3: \|V7: " -- web/src/usecases | wc -l          # 4 (two per version)
git grep -n "file\.name\|webkitRelativePath" -- web/src   # (no output): the file name is never read (P-11). An error's name, as in the QuotaExceededError test of import.ts, is not one
pnpm -C web exec eslint src/usecases                           # clean: D-27 a, D-58 and the D-59 lines prove their positive side
```

| Temporary edit | Rule that must fire |
|---|---|
| `import { startExport } from "./start-export";` in `run-pipeline.ts` | `boundaries`: use-case to use-case, not one of the two D-58 pairs |
| A second `as ClipId` in `import-clip.ts` | The brand-cast rule: the override covers one minting line |
| `fetch("/x")` in `start-export.ts` | The V1 `fetch` restriction |

| Do (`pnpm dev`, console; model cached) | Expect |
|---|---|
| `const u = await import("/src/usecases/import-clip.ts"); const [h] = await showOpenFilePicker(); await u.importClip(await h.getFile(), "user")` then read the clip store after about a minute | `status: "ready"`, `events.length >= 1`, `feed[0].kind === "transcribing"`, `out48.length === Math.round(durationMs * 48)` |
| Network during that minute | Only `POST /api/v1/events` |
| With no `entitlement` record (delete it in DevTools), `await (await import("/src/usecases/start-export.ts")).startExport(clipId)` | Export store `unavailable: true`, status `idle`, no file under `exports/` |
| `await u.dismissClip()` | Clip store `idle`; OPFS `clips/` empty |

### Milestone 9

```bash
pnpm -C web exec tsc --noEmit                 # no output
pnpm -C web exec eslint .                     # clean
pnpm -C web exec vitest run                   # all passed
pnpm check                                    # green
grep -c "media.worker\|asr.worker\|render.worker" web/src/workers/pool.ts   # 3 rows, nothing else names a worker (§25.3)
git diff --stat "$(git merge-base main HEAD)" -- web/src/workers/protocol.ts web/src/persistence/schema.ts web/src/persistence/db.ts web/src/net/http.ts web/src/net/api-client.ts web/src/analytics/client.ts web/src/platform web/src/state/capability-store.ts web/src/state/machines/transition.ts   # (no output)
```

Pass: the command block is green, 8 blocker codes are covered by copy or a pending entry, the dev-console import reached `ready` with at least one event and only event POSTs on the network, and the frozen web files are unchanged.

Commit: `"V2 S12: machines, stores, blockers, entitlement repo, preview loop, use-cases"`. Do not begin Phase 10 until this passes.

---

## Phase 10 - UI and copy (S13)

Goal: a person can drop the clip on the deployed-style build, watch the feed, play the preview and, with a seeded token, download the MP4.

### Step 10.1 - Copy and styles

`messages.ts` gains `feed`, `preview`, `export`, `editor` (§20.3; `modelDownload`, `errors` and `blockers` are already there). `dropZone.notReady` is **not** removed yet: it goes in Step 10.3 with the component that stops using it (item 5e). New classes in the two CSS Modules files (§20.4).

```bash
node scripts/check-copy-codes.mjs             # green
git grep -n "never leaves\|GDPR\|DPDP\|CCPA\|SOC 2\|compliant" -- web/src/copy   # (no output) (P-12)
```

### Step 10.2 - Components

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `ui/components/ProcessingFeed.tsx` | §20.2 | `switch` over `kind` and `eventKind`, each ending in `assertNever`; no invented line |
| 2 | `ui/components/PreviewPlayer.tsx` | §20.2, D-51 | Canvas 540x960; `attach` on mount, `detach` on unmount; `pause()` when the tab hides; buttons disabled while `locked` |
| 3 | `ui/components/ExportButton.tsx` | §20.2 | Reads `forExport()`; `unavailable` copy; no counter, no upgrade prompt (INV-14) |
| 4 | `ui/components/ExportProgress.tsx` | §20.2 | Bar width through `style.setProperty` on a custom property; no `style` attribute string |

| Temporary edit | Rule that must fire |
|---|---|
| `<p>Rendering</p>` in `ExportProgress.tsx` | `react/jsx-no-literals` |
| `import { pool } from "../../workers/pool";` in `PreviewPlayer.tsx` | `boundaries`: `ui` may not import `workers` |
| `useClipStore.setState(...)` in a component | The V1 rule that components do not write stores; if none fires, note the gap in `v2changelog.md` |

### Step 10.3 - Drop zone, pages, routes, and the two landing cases

One step, because each edit makes the others true:

| File | Change | Section |
|---|---|---|
| `ui/components/DropZone.tsx` | Wired to `importClip(file, "user")` and `importSampleClip()`; no longer `aria-disabled`; first dropped file only | §20.2 |
| `copy/messages.ts` | `dropZone.notReady` deleted | D-56 |
| `ui/pages/LandingPage.tsx` | `navigate(appPath)` when the clip leaves `idle` | §20.1, D-48 |
| `routes.tsx` | Passes `appPath` to `LandingPage`. **Not in the §4 "CHANGED" list (item 6c): add it there in this commit** | D-48 |
| `ui/pages/EditorPage.tsx` | Body by clip status; placeholder and its `NotifyMeForm` removed | §20.1 |
| `web/tests-e2e/landing.spec.ts` | **REOPENED V1 CONTRACT:** the two cases of §23.5 replace two V1 cases; every other case untouched | D-56 |

```bash
git grep -n "notReady" -- web                 # (no output)
pnpm check && pnpm build && pnpm e2e          # green: landing.spec.ts with its two replaced cases, on a build without the test key
```

### Step 10.4 - Walk through it in dev

| Do (`pnpm dev`, model cached) | Expect |
|---|---|
| Open `/`, drop the reference clip | URL becomes `/app`; feed shows "Transcribing…" first |
| Wait | At least one "Found:" line with `$12k`; no "Cleaning voice" line; then the player |
| Play | Video in a 9:16 centre crop, captions word by word in the lower third, sound in sync; the reveal appears as the amount is spoken |
| Pause, Play | Resumes from the same position |
| Let it reach the end | Replay is offered; status `stopped` |
| Export, with no token | The unavailable message; no download |
| "Start over" after forcing a rejection (drop a `.txt` renamed to `.mp4`) | Stub message, no code shown; back to the drop zone |
| Delete OPFS `models/`, reload, drop | The model panel with the size from the manifest, a moving bar, a time remaining, then the feed |
| Sample button | Requests `SAMPLE_CLIP_PATH` from the asset host; same flow |

### Step 10.5 - Walk through it in a production build, with a seeded token

```bash
export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=<test public key>     # this terminal only
pnpm build:wasm && pnpm -C web exec vite build && pnpm -C web exec vite preview
```

Seed a Creator token (minted as in Step 6.5 with future `exp` and `periodEnd`) in the console at `http://localhost:4173`:

```js
indexedDB.open("offcut").onsuccess = (e) => e.target.result.transaction("entitlement", "readwrite").objectStore("entitlement")
  .put({ schemaVersion: 1, value: { token: "<token>", storedAt: Date.now() } }, "current");
```

If the `put` throws on the key, read `web/src/persistence/schema.ts` for the store's key form and the type of `storedAt`, and match it.

| Do | Expect |
|---|---|
| Drop the clip, wait for the player, press Export | Stage labels rendering, then muxing; "rendering on your computer" line throughout; a bar that advances |
| Download | Named `offcut-<yyyymmdd>-<hhmm>.mp4` |
| Play in the preview again | Works (the preview canvas and profile were restored) |
| Console | No "Refused to" line during the whole flow |

```bash
python verify/verify_mp4.py ~/Downloads/offcut-*.mp4 --profile creator --expected-duration-ms <duration>; echo "exit=$?"   # six PASS, exit=0
unset VITE_ENTITLEMENT_TEST_PUBLIC_KEY && rm -f web/tests-e2e/zz-mint.spec.ts
```

### Milestone 10

```bash
pnpm check && pnpm test                       # green
pnpm build && pnpm e2e                        # green (no test key in this build)
grep -rl "<test public key>" web/dist | wc -l # 0
git grep -n "notReady\|zz-" -- web            # (no output)
```

| Check | Where | Expect |
|---|---|---|
| Drop to preview | Dev and `vite preview` | Feed, at least one "Found:" line, playing preview with sound |
| Seeded export | `vite preview` with the test key | A download that passes verifier checks 1-6 |
| No token | Either | Unavailable message, no download |

Pass: the command block is green, both walk-throughs match every row, and the exported file passes checks 1-6.

Commit: `"V2 S13: editor shell, components, copy, drop zone and landing navigation"`. Do not begin Phase 11 until this passes.

---

## Phase 11 - E2E, CI, bench (S14)

Goal: the three media suites, the verifier and the bench run from one command locally and in CI on Windows, and the cost of the CI job is known.

The plan gives this phase half a day (S14). Thirty E2E cases, two workflows and the bench rarely fit in that. The build started ahead of the calendar (§5): spend that slack here rather than dropping a case.

### Step 11.1 - Helpers, Playwright projects, root scripts

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/tests-e2e/helpers/fixtures.ts` | §23.4, D-39, D-41 | Adds `routeAssets`, `dropClip`, `ensureModelCached`, `opfsList`, `sourceDurationMs`, `referenceClip`, and `fillAssetCache()`, which downloads what `fixtures/.cache/` lacks and checks each hash. `dropClip` names the file `clip.mp4`. `sourceDurationMs` and the verifier call start `ffprobe` and `python` from PATH (item 2b) |
| 2 | `web/tests-e2e/media.setup.ts` | Item 6b | New file, added to TS §5 and §4 in this commit. One Playwright setup test that calls `fillAssetCache()`, with its timeout raised for a download of up to 260 MB. The config has no `globalSetup` |
| 3 | `web/playwright.config.ts` | §22.3 | Projects `media-setup` (`testMatch` of `media.setup.ts`), `media` (3 suites, `channel: "chrome"`, 300 s, `dependencies: ["media-setup"]`) and `bench` (`testDir: "../bench"`, headed, the same dependency). `non-media` gets a `testMatch` of `landing.spec.ts`, which it does not have today, and no dependency, so `pnpm e2e` and the Linux job never download the model |
| 4 | Root `package.json` | §22.2 | Scripts `e2e:media`, `e2e:device`, `bench:device` (`verify` exists since Step 8.1) |
| 5 | `fixtures/speech/README.md` | §23.7, §25.3 | The expected transcript and expected events of Step 5.4, final |

```bash
pnpm -C web exec tsc --noEmit -p tests-e2e/tsconfig.json     # clean
pnpm -C web exec playwright test --list --project media | tail -1   # 0 tests yet, no config error
ls fixtures/.cache 2>/dev/null | wc -l                        # filled by the media-setup project on the first real run
```

### Step 11.2 - The three media suites

| Order | File | Section | Tests to write |
|---|---|---|---|
| 1 | `web/tests-e2e/model-download.spec.ts` | §23.6 | 8 cases |
| 2 | `web/tests-e2e/pipeline-preview.spec.ts` | §23.7 | 11 cases; the long-task case writes its number and asserts nothing |
| 3 | `web/tests-e2e/export-creator.spec.ts` | §23.8 | 11 cases; each export checked by `verify_mp4.py` from inside the suite |

```bash
export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=<test public key>
pnpm build && pnpm e2e:media                  # 30 passed
```

| Temporary edit | Rule that must fire |
|---|---|
| Build without the variable, run `export-creator.spec.ts` | "Creator export" fails with `E_ENTITLEMENT_INVALID`: the suite depends on the key, as intended |
| In `routeAssets`, answer one model request with 200 for a non-asset host | "Requests" case of §23.6 fails |
| Comment out `await pool.asr.unload()` in `run-pipeline.ts` | Nothing in V2 asserts INV-12 directly; note it in `v2changelog.md` as covered by review until V4's `run-pipeline.test.ts`. Restore the line |

### Step 11.3 - `bench/device-bench.ts`

Section §23.10. Sanity run on the development machine (not a recorded device):

```bash
BENCH_DEVICE=dev pnpm bench:device            # 1 warm-up + 10 runs; writes bench/results/dev-<date>.json
node -e "const r=require('./bench/results/'+require('fs').readdirSync('bench/results').find(f=>f.startsWith('dev-')));console.log(r.runs,Object.keys(r.stages).join(),r.stages.asr.ms.length,typeof r.total.p90)"
# 10 probe_audio,asr,detect_scene,render_encode,mux 10 number
grep -c "clip.mp4\|twelve" bench/results/dev-*.json   # 0: numbers and enums only
rm bench/results/dev-*.json
```

### Step 11.4 - CI

| File | Change | Section |
|---|---|---|
| `.github/workflows/e2e-media.yml` | New, reusable, `windows-latest`. **Set `defaults: run: shell: bash`** (item 2c). Steps 10.1-10.5 of §22.5 | D-44 |
| `.github/workflows/ci.yml` | Step 8 builds with `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` written literally and uploads `web/dist` on every event; step 10 calls the media workflow before any deploy; the Vercel step is split into build, **test-key guard**, deploy | §22.5 |
| `scripts/check-external-facts.mjs` | TE-10 line with its date | §22.4 |

| Tool in the workflows | Must equal |
|---|---|
| Node, pnpm | Root `package.json` `engines` / `packageManager` |
| `wasm-bindgen` CLI | `Cargo.lock` (Step 7.4) |
| Playwright | The pinned `@playwright/test`; install with `pnpm -C web exec playwright install chrome` |
| Python | `python` on the runner (`actions/setup-python`, pinned to a commit like the other actions), then `python -m pip install -r verify/requirements.txt` |
| `ffmpeg` | Do not count on the runner image having it. Install a release archive in the workflow the way `ci.yml` installs its other tools, with the SHA-256 written in the file, and print `ffmpeg -version` in the log |

Secrets added: none. Variables added: none (the test public key is a literal in step 8). Steps 11-15 run on `main` only.

In the workflow the guard searches what `vercel build` wrote, the directory `.vercel/output/`, not `web/dist`, which that job never builds. If the secret scan names the test public key literal of step 8, end that line with `# gitleaks:allow` (item 9).

Prove the guard can fire, locally, with the same search the workflow uses:

```bash
VITE_ENTITLEMENT_TEST_PUBLIC_KEY=<test public key> pnpm build >/dev/null && grep -rl "<test public key>" web/dist | wc -l   # >= 1: the guard would fail this build
pnpm build >/dev/null && grep -rl "<test public key>" web/dist | wc -l                                                        # 0
```

Never upload an exported MP4 or a trace as an artifact (§22.5 step 10.5): check the `upload-artifact` paths name the HTML report only.

### Step 11.5 - Pull request and TE-10

```bash
git push && gh pr ready                       # the draft pull request of Milestone 1
gh pr checks --watch                          # every job green, the media job among them
gh run view <run id> --json jobs --jq '.jobs[] | [.name, .startedAt, .completedAt] | @tsv'   # minutes of the Windows job
```

| Outcome | Do |
|---|---|
| WebGPU unavailable on the runner | Add the launch arguments that make it available to the `media` project; record them as the TE-10 arguments in `v2changelog.md` |
| The capability check on the runner reports `UNSUPPORTED_AAC_ENCODE` or `UNSUPPORTED_H264_ENCODE` | A Windows Server image may lack the system encoders Chrome uses (not tested). No launch argument adds them: take the TE-10 fallback below and record the reason |
| Suites green; minutes x expected runs per month fit the allowance | Record in `experiments.md`; keep the job on pull requests |
| Does not fit, or WebGPU cannot be made to work | TE-10 fallback (§22.5): step 10 on `main` and manual trigger only; `pnpm e2e:device` before every merge. Record it |

### Milestone 11

```bash
export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=<test public key>
pnpm check && pnpm test && pnpm build && pnpm e2e && pnpm e2e:media   # all green; e2e:media: 30 passed
unset VITE_ENTITLEMENT_TEST_PUBLIC_KEY
gh pr checks                                  # all pass, or the recorded TE-10 fallback
grep -c "TE-10" docs/v2/experiments.md scripts/check-external-facts.mjs   # each >= 1
git status --porcelain bench/results          # (no output: no dev-*.json left)
```

Pass: the local chain is green with 30 media cases, the pull request is green including the Windows media job (or the TE-10 fallback is recorded and in the workflow), the guard search returned at least 1 with the key and 0 without, and TE-10 has minutes and a date.

Commit: `"V2 S14: media E2E suites, helpers, bench, e2e-media workflow, test-key guard"`. Do not begin Phase 12 until this passes.

---

## Phase 12 - Deploy, measure, gate, tag (S15, S16)

Goal: V2 is live and externally verified, R1 numbers are committed, the M0 decision is written, and the commit is tagged `v2`.

### Step 12.1 - Fill in the real values

Every value must be final before the merge. Nothing here is a secret.

| File or place | Value | Set in | Check now |
|---|---|---|---|
| `web/src/config/model-manifest.json` | Final model paths, sizes, hashes | Step 4.3 (or again after an E-3 fallback) | `download.test.ts` manifest case green |
| `web/src/net/asset-fetch.ts` | `SAMPLE_CLIP_PATH` | Step 4.3 | `curl -sI "<asset base URL>/<path>"` is 200 |
| `web/src/config/entitlement-public-key.ts` | Production public key | Step 1.6 | Decodes to 32 bytes; equals the value derived from Render's seed |
| `web/src/workers/render/encoders.ts` | `AAC_PRIMING_SAMPLES`, ladder order | Step 8.4 | A number with a TE-4 comment |
| `crates/offcut-entitlement/src/profile.rs` | Bitrates equal to `encoders.ts` | Step 8.4 | The `grep` of Step 8.2 |
| `web/src/workers/asr/whisper-runtime.ts` | Backend order, thread rule, option names | Step 5.5 | Listed in `v2changelog.md` |
| `web/src/workers/render/export-loop.ts` | Capture method A or B | Step 8.4 | Named in a comment and in TS §21.4 |
| `.github/workflows/ci.yml` | Test public key literal (step 8); guard (step 14) | Step 11.4 | Step 11.4 guard proof |
| `web/playwright.config.ts` | WebGPU launch arguments | Step 11.5 | Media job green |
| Vercel project environment | **`VITE_ENTITLEMENT_TEST_PUBLIC_KEY` absent**; `VITE_ASSET_BASE_URL` unchanged | - | `vercel env ls production` shows no line with that name |
| Render environment | Unchanged | - | Not opened |

### Step 12.2 - Merge and deploy, server first

```bash
gh pr merge --squash --delete-branch=false    # or the merge style V1 used
git switch main && git pull --ff-only && git rev-parse HEAD   # the commit that must be deployed
gh run watch                                  # ci.yml on main
```

Order inside the run (§22.5): media job, `deploy-api`, wait for `/api/v1/healthz` to report the commit, `vercel build --prod` without the test key, guard, `vercel deploy --prebuilt --prod`, header check, `@smoke`.

| If | Do |
|---|---|
| `wait-for-version` times out | The API did not deploy; the web is not deployed either. Fix on Render, re-run the job. Never deploy the web by hand first |
| The guard fails | A variable leaked into the Vercel build. Remove it from Vercel, re-run. Nothing was deployed |
| Header check or `@smoke` fails after deploy | Roll the web back to the previous production deployment in Vercel (`vercel rollback`), then fix forward. The API of the new commit is compatible: the server did not change in V2 (§14) |

### Step 12.3 - Verify the deployed system

```bash
curl -s https://<api host>/api/v1/healthz     # reports the merge commit
curl -sI https://<app host>/ | grep -i "cross-origin-opener-policy\|cross-origin-embedder-policy\|content-security-policy"   # three headers, as in web/vercel.json
V=$(ls web/public/ort/); F=$(ls web/public/ort/$V | grep "\.wasm$" | head -1)
curl -sI "https://<app host>/ort/$V/$F" | grep -i "content-type\|cache-control"   # application/wasm ; immutable
curl -s "https://<app host>/" | grep -o 'assets/[^"]*\.js' | sort -u | while read f; do curl -s "https://<app host>/$f"; done | grep -c "<test public key>"   # 0
```

| Do (Chrome on R1, deployed URL) | Expect |
|---|---|
| Console: `crossOriginIsolated` | `true` |
| Drop the reference clip | Model downloads from the asset host (206 rows only), feed, at least one "Found:" line, preview plays with sound |
| Export | The unavailable message; no download (§1.2: expected for a real visitor) |
| Reload, sample button | No model request; the sample clip is fetched from the asset host; same flow |
| Console during all of it | No "Refused to" line |

Analytics rows, against the production database, read-only. Its connection string is `PROD_DATABASE_URL` in `.env.deploy`; `$DATABASE_URL` is the local database (item 10). Load that one variable in this terminal only and never echo it. If the Neon password was reset (Step 0.4), `.env.deploy` must hold the new string.

```bash
eval "$(grep '^PROD_DATABASE_URL=' .env.deploy)"              # production, not $DATABASE_URL (item 10); sets the variable, prints nothing
psql "$PROD_DATABASE_URL" -c "select current_database()"      # the Neon database; offcut_dev means the local one: stop
psql "$PROD_DATABASE_URL" -c "\d analytics_events"            # columns id, anon_id, name, props, ts (0001_init.sql)
psql "$PROD_DATABASE_URL" -c "select name, count(*) from analytics_events where ts > now() - interval '1 hour' group by 1 order by 1"
# rows for clip_accepted, model_download, stage_timing, pipeline_done, preview_played
psql "$PROD_DATABASE_URL" -c "select props from analytics_events where name = 'pipeline_done' order by ts desc limit 1"
# integers and enums only: no text of the transcript, no file name
```

### Step 12.4 - Measure on R1

Build once with the test key on the development machine and copy `web/dist` to R1. R2 is not measured (item 12).

```bash
# on R1
export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=<test public key>
E2E_REAL_ASSETS=1 pnpm -C web exec playwright test model-download --project media -g "First run"   # real first-run timing (D-41); note the model_download duration
pnpm e2e:device                               # non-media and media, headed: all passed
BENCH_DEVICE=r1 pnpm bench:device             # bench/results/r1-<date>.json
python verify/verify_mp4.py <an export made on this device> --profile creator --expected-duration-ms <duration>; echo "exit=$?"   # exit=0
```

| Record in `docs/v2/experiments.md` | From | Threshold (reference clip, D-64) |
|---|---|---|
| E-3: `asr` median and p90 | `stages.asr` | Target 25,000 ms on R1; accepted up to 180,000 ms (D-67) |
| E-3 total: `total.p90` + 5,000 ms audio-chain allowance | `total` | At most 379,000 ms (D-67; was 224,000) |
| E-4: `render_encode` median and p90 | `stages.render_encode` | At most 112,000 ms on R1 |
| `probe_audio`, `detect_scene`, `mux` medians | `stages` | 3,700; 1,250; 2,500 ms (recorded, not gated) |
| Each value normalised to 60 s | `x 60000 / 74705` | - |
| Cold capability check time on R1 (V1 known issue 30) | DevTools Performance, first load in a fresh profile | Against the 2 s timeout |

Commit `bench/results/r1-<date>.json`.

### Step 12.5 - TE-14

```bash
ffmpeg -v error -stream_loop 1 -i testclips/speech_scriptA_landscape_720p.mp4 -t 89.9 -vf "scale=1920:1080,fps=60" \
  -c:v libx264 -preset veryfast -pix_fmt yuv420p -c:a aac -ar 48000 -ac 2 testclips/zz_stress_90s_1080p60.mp4
ffprobe -v error -show_entries format=duration,size -show_entries stream=width,height,avg_frame_rate -of default=nw=1 testclips/zz_stress_90s_1080p60.mp4
# 1920x1080, 60/1, duration < 90.0, size < 500000000
```

`-t 89.9`, not 90: a trailing AAC frame past 90,000 ms would be rejected for duration and the test would measure nothing. The file stays in `testclips/` (ignored). On R1, in the seeded production build: import, pipeline, export, 10 times; read memory per phase from the Chrome task manager and `measureUserAgentSpecificMemory`.

| Result | Do |
|---|---|
| Peak under 1.5 GB, no tab crash in 10 runs | Record; replace TS §39.3 item 23 |
| Otherwise | §24.1 fallbacks (smaller queues, 360x640 preview, smaller model); re-run; record |

### Step 12.6 - Records and repository checks

| Update | With | Section |
|---|---|---|
| `docs/v2/experiments.md` | TE-1, TE-2, TE-3, TE-4, TE-7 re-check, TE-10, TE-14, E-3, E-4: question, method, numbers, date, decision | §24 |
| `docs/technicalspec.md` §39.3 | Each measured (assumption) replaced by its value | §24, §26 |
| `docs/technicalspec.md` §2, §7, §9 C-3, §25.1 P-5, §32 and the decisions listed at the end of §27 | The corrections of §27 items 1, 2, 4, 7, 12, 14 | §27 |
| `docs/buildplan.md` | Schedule moves D-18, D-20, D-21, D-23, D-24, D-33 | §27 |
| `docs/v2/v2implementation.md` | Any D-n that changed during the build; in §4, `routes.tsx`, `web/tests-e2e/media.setup.ts` and, if Step 7.1 created it, `.cargo/config.toml`; the decisions this guide adds (D-65 of Step 1.1, the short-file rule of Step 2.2, the confidence decision of Step 5.5 if one was taken) | §26 |
| `docs/buildplan.md`, `docs/technicalspec.md`, `docs/product.md` | D-65 copied back: BP §1.5, §4.4, §12.1; TS §28, §30, §37; PS A-7, §20.1 | Item 12 |
| `docs/v2/v2changelog.md` | Every pinned version; every runtime option name; deny wrappers added; lint gaps noted in Steps 5.3, 6.2, 10.2, 11.2 | §26 |

```bash
git grep -n "fetch(" -- web/src | grep -v "net/http.ts\|net/asset-fetch.ts\|wasm/load-core.ts\|wasm/load-render.ts"   # (no output)
git ls-files | grep -i "\.\(ttf\|otf\|woff2\?\)$" | grep -vc "^crates/offcut-scene/assets/fonts/"                       # 0
git ls-files 'crates/*.rs' 'web/src/*.ts' 'web/src/*.tsx' 'scripts/*' | grep -v "\.test\.\|/tests/" | xargs wc -l | awk '$2!="total" && $1>400'   # (no output)
git grep -n "TODO" -- crates web/src scripts | grep -v "TODO(V[0-9]\+)\|V[0-9]\+:"                                      # (no output)
git diff --stat <baseline SHA of Step 0.2> HEAD -- server crates/offcut-types crates/offcut-api-types web/src/gen web/vercel.json render.yaml clippy.toml   # (no output)
```

Tick §25.3 (13 boxes) and §26 (26 boxes) in `docs/v2/v2implementation.md`, each against the evidence above.

### Step 12.7 - The M0 gate (S16, no later than Sun 1 Nov 2026)

```bash
eval "$(grep '^PROD_DATABASE_URL=' .env.deploy)"              # production, not $DATABASE_URL (item 10); sets the variable, prints nothing
psql "$PROD_DATABASE_URL" -c "\d platform_waitlist"
psql "$PROD_DATABASE_URL" -c "select count(*) from analytics_events where name = 'landing_view'"     # visitors since V1 went public
psql "$PROD_DATABASE_URL" -c "select count(*) from platform_waitlist where wanted = 'launch'"                  # joined
```

Write the decision in `docs/v2/experiments.md` in the form §24.3 fixes: the three numbers (E-1, E-3 on R1, E-4 on R1, measured and normalised), the column each fell in, the action taken, the date. E-1 with fewer than 1,000 visitors or no announcement is "inconclusive": decide on E-3 and E-4 alone.

| Decision | Do |
|---|---|
| Continue | Tag (Milestone 12) |
| Fallback, then continue | Apply it, re-measure the affected reading, write both, then tag |
| Stop | Do not tag `v2`. Record the decision; V3 does not start (BP §5) |

### Milestone 12

```bash
git switch main && git pull --ff-only && git status --porcelain      # (no output)
pnpm check && pnpm test && pnpm build && pnpm e2e                    # green on the tagged commit
gh run list --branch main --workflow ci.yml --limit 1 --json conclusion --jq '.[0].conclusion'   # success
curl -s https://<api host>/api/v1/healthz                            # reports HEAD
ls bench/results | grep -c "^r1-.*\.json$"                            # 1
grep -c "TE-1\b\|TE-2\|TE-3\|TE-4\|TE-10\|TE-14" docs/v2/experiments.md   # >= 6
grep -n "M0 gate" docs/v2/experiments.md                             # the decision line, with a date
grep -c "\- \[ \]" docs/v2/v2implementation.md                       # 0: tick "Tagged v2" in this last commit; the tag below then points at it
git tag -a v2 -m "V2: one clip in, one MP4 out; M0 gate: <decision>" && git push origin v2
git ls-remote --tags origin v2                                       # one line
```

| Check | Where | Expect | Exit box (§26) |
|---|---|---|---|
| Local chain | Development machine | `pnpm check`, `test`, `build`, `e2e` green | Code and checks 1-4, 9, 10 |
| Media suites | R1, `pnpm e2e:device` | All passed | Code and checks 5, 6 |
| CI | GitHub, `main` | `ci.yml` green with the media job, or the recorded fallback | Code and checks 7; Deployment 1 |
| Verifier | An export made on R1 | Checks 1-6, exit 0 | Code and checks 8 |
| Guard | CI log, and the `curl` of Step 12.3 | Passed; 0 hits in the deployed bundle | Deployment 2 |
| Headers, smoke, isolation | Deployed URL | Pass; `crossOriginIsolated === true` | Deployment 3 |
| Drop on R1 | Deployed URL | Model download, feed, preview; export unavailable | Deployment 4 |
| `/ort/<version>/` | `curl -I` | `application/wasm`, immutable | Deployment 5 |
| Sample button | Deployed URL | Works | Deployment 6 |
| Analytics rows | `psql` | Five event names; enum and integer props only | Deployment 7 |
| Bench files | `bench/results/` | `r1-*.json`, 10 runs | Experiments 1 |
| Experiments | `experiments.md` | Six TE outcomes, fallbacks applied | Experiments 2-4 |
| Gate | `experiments.md` | Three readings, columns, action, date | Experiments 5 |
| Records | Specs, changelog, §25.3 | Updated and ticked | Experiments 6-8 |
| Tag | `git ls-remote` | `v2` on the remote | Experiments 9 |

Pass: every command prints the stated output, every row of the table holds, all 26 boxes of §26 and all 13 of §25.3, as reworded for R1 in Step 1.1, are ticked with evidence, the gate decision says continue (directly or after a fallback), and `v2` exists on the remote at the deployed commit.

Commit: `"V2 S15-S16: bench results, experiments, spec corrections, M0 gate decision"` (the commit the tag points to). V3 does not begin until this passes.

---

## Quick reference

### a. Every new terminal

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut && set -a; . ./.env; set +a          # the local environment: DATABASE_URL is the local database (item 10)
export PATH="/c/Users/Mayan/AppData/Local/Programs/Python/Python313:$PATH"    # python is 3.13 with NumPy, not the MSYS2 3.9 (item 2b)
# only in a terminal that builds for E2E, the spikes or the bench, and never in a profile file:
export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=<test public key>
```

### b. After changing ... / Run ...

| After changing | Run |
|---|---|
| Any Rust under `crates/` that a bundle includes | `pnpm build:wasm` (both bundles), then restart `pnpm dev` |
| `Cargo.toml` or `Cargo.lock` | `cargo deny check`; `node scripts/check-file-tree.mjs`; compare `wasm-bindgen --version` with `Cargo.lock` |
| `vello`, `wgpu` or `parley` version | `cargo tree -d -p offcut-render`; `pnpm build` (new `check-hosts` literals); record in `v2changelog.md` |
| Anything in `offcut-types` or `offcut-api-types` | Not allowed in V2. If it happened: `pnpm gen:types && bash scripts/check-gen-clean.sh`, and flag it as a reopened V1 contract |
| `@huggingface/transformers` version | `pnpm build` (the copy step replaces `web/public/ort/<version>/`); Step 5.5 again (TE-1); update the D-38 entries |
| Model files | Steps 4.2-4.4: hash, upload, manifest; the CI cache key changes with the manifest by itself; clear `fixtures/.cache/` |
| The reference clip | Upload to `media`; `SAMPLE_CLIP_PATH`; `fixtures/speech/README.md`; every duration in this guide |
| `messages.ts`, `BLOCKER_CODES`, any error or reject code's copy | `node scripts/check-copy-codes.mjs` |
| A file added, moved or deleted | Update the TS §5 tree in the same commit; `node scripts/check-file-tree.mjs` |
| `web/eslint.config.js` | Re-run the "prove it by" rows of Step 1.5 |
| `TEST_ENTITLEMENT_SEED` | New public key in `ci.yml` step 8 and the guard; re-mint the vector of Step 6.5 |
| `ENTITLEMENT_SIGNING_KEY` in Render | Derive the public half again; `entitlement-public-key.ts` |
| `AAC_PRIMING_SAMPLES`, ladder, bitrates | Both `encoders.ts` and `profile.rs`; export; `verify_mp4.py`; TS §21.2 |
| `@playwright/test` in `web` | Same version in the root package (D-63); `pnpm -C web exec playwright install chrome` |
| `scripts/check-hosts.mjs` entries | `pnpm build`, then the negative test of Step 5.5 |
| Pinned tool versions | The same version in `ci.yml` and `e2e-media.yml` |

### c. When something fails / Likely cause

| Symptom | Likely cause |
|---|---|
| A `.sh` script cannot find files that exist | `bash` is WSL. `which bash` (item 2) |
| `No module named numpy`, `No module named pip`, or the verifier fails only inside `pnpm e2e:media` | `python` is the MSYS2 3.9: the PATH line of Quick reference a did not run in this terminal (item 2b) |
| The analytics or waitlist queries return no rows, or only rows of your own tests | `$DATABASE_URL` was used, which is the local database. Production is `PROD_DATABASE_URL` (item 10) |
| The `ci` job fails at the secret scan on a public key | gitleaks took the base64 literal for a secret: a `gitleaks:allow` comment on that line (item 9) |
| `build-wasm.sh`: "it looks like the Rust project used to create this wasm file was linked against version X" | `wasm-bindgen` crate moved in `Cargo.lock`; pair rule of Step 3.2 |
| The render bundle fails to build on a `VideoFrame` item of `web_sys` or `wgpu` while `cargo check` passes | `--cfg=web_sys_unstable_apis` is missing on the `RUSTFLAGS` line of `scripts/build-wasm.sh`; that line replaces `.cargo/config.toml` (Step 7.1) |
| Two `wgpu` versions in `cargo tree -d`; type mismatch between `vello` and `wgpu` types | `wgpu` not pinned to the version `vello` depends on (Step 7.1) |
| `pnpm build` fails in `check-hosts.mjs` after a dependency change | New literal in the runtime, `ort/` or the render bundle: add a `chunk`-scoped entry with a reason (D-38). Never widen an entry to all chunks |
| Works in `pnpm dev`, "Refused to ..." in `vite preview` or deployed | Production CSP: a `blob:` worker, a remote model host, or a `.wasm` path outside `/ort/<version>/` |
| Vite: "Could not resolve ./asr.worker.ts" (or `render.worker.ts`) | The pool table names a worker script that does not exist yet (item 5a) |
| ASR returns words without times, or `NoSpeech` on the reference clip | Model export without cross-attention outputs; or the cache adapter returned the wrong file for a name (D-62 `localName`) |
| Model request for a plain name returns 404 from the asset host | The runtime tried to fetch: remote loading is not off, or the cache adapter was not installed before `loadModel` |
| Model download restarts at 0 after a reload | The writable was not closed per part (D-36), or `resumeFrom` was not read from the `.part` size |
| `E_MODEL_HASH` on every attempt | Manifest hash taken from a different file than the one uploaded; rerun `sha256sum` on the uploaded bytes |
| `pcm48.length` off by one | D-34 pad or truncate missing after the resampler |
| Export fails with `E_INTERNAL` right at the end | Live-frame counter non-zero: a `VideoFrame` not closed in a `finally` (D-45) |
| Export fails with `E_MUX` | Encoder output not in frame order (`OutOfOrder`): take the next ladder entry (TE-4); `ctts` is V5 |
| Verifier `FAIL 5` | `AAC_PRIMING_SAMPLES` still 0 or wrong; no `elst` written |
| Verifier `FAIL 3` | A frame skipped or repeated; non-integer frame time; wrong `stts` delta |
| `export-creator.spec.ts`: `E_ENTITLEMENT_INVALID` for a valid token | The build under test lacks `VITE_ENTITLEMENT_TEST_PUBLIC_KEY`, or it does not match `TEST_ENTITLEMENT_SEED` |
| CI deploy job stops at the guard | The test key reached the Vercel build: remove the variable from Vercel |
| `e2e-media.yml`: a script line is "not recognized" | Step ran in PowerShell: `defaults.run.shell: bash` missing |
| Media job: capability unsupported, suites skip or fail at the drop | WebGPU unavailable on the runner: launch arguments of TE-10, or the fallback |
| Real-asset run on port 4173 blocked by CORS | The bucket lacks the origin `http://localhost:4173` (Step 0.4, D-41) |
| `check-copy-codes` fails after adding copy | The code is still in `COPY_PENDING` (V1 D-13), or a blocker code has neither |
| TE-14 clip rejected for duration | Cut at 90.0 s: a trailing audio frame passes 90,000 ms; use `-t 89.9` |

---

## Assumptions I made and gaps found

### (a) Environment values assumed

The first draft was written without the machine and without `v1buildguide.md` and `v1implementation.md`. On 2026-10-08 it was read against the repository and the development machine; the rows below give what was found.

| Value | Found | Where it is checked or matters |
|---|---|---|
| OS and shell | Windows 11; `bash` is Git Bash (`/usr/bin/bash`) | Step 0.1 |
| Repository path | `/c/Users/Mayan/Desktop/sh2clips/offcut` | Step 0.2; Quick reference a |
| Installed tools | V1's toolchain; Python 3.13.5 with NumPy 2.5.3, behind an MSYS2 Python 3.9 in PATH; ffmpeg 9.0.2. Not installed: `gh`, gitleaks | Step 0.1 audit table; item 2b |
| Package installer | `winget`, for `gh` | Step 0.1; any installer works |
| State of V1 | Code complete and deployed on `main`, `ci.yml` green, **not tagged** (§1A, 2026-10-08) | Step 0.2 |
| Accounts | Vercel, Render, Neon, the asset host, GitHub exist from V1; the `vercel` and `psql` CLIs are available, `gh` is not | Steps 0.1, 0.4, 11.5, 12.2, 12.3 |
| Days | 10 working days plus the gate day (BP §0, §5) | Phase overview |
| Reference machines | R1 only (founder's decision, 2026-10-08). The development machine is not R1 | Step 0.6; item 12 |
| Spending | Free tiers only (BP 0 USD rule) | TE-7, TE-10 fallbacks |
| How V1 loads its environment in a terminal | `set -a; . ./.env; set +a`. That file is the local environment; production values are in `.env.deploy` | Quick reference a; item 10 |
| Format exemplar | Structure taken from the prompt's rules, not from `v1buildguide.md` | Whole document |
| `upload-assets.sh` argument order; `analytics_events` column names; the `entitlement` store's key form | `<folder> <file>...`; `name`, `props`, `ts`; key `"current"` with value `{ schemaVersion, value: { token, storedAt } }`, `storedAt` a number | Steps 4.1, 12.3, 10.5 |
| V1 lint rules for the runtime import, store writes from components, `HashMap` and typed-in digits | None of the four exists in `web/eslint.config.js`, `clippy.toml` or the scripts | Steps 5.3, 10.2, 6.2, 4.5 say what to do when nothing fires |

### (b) Spec contradictions and omissions, and how the guide resolves them

| # | Finding | Resolution |
|---|---|---|
| 1 | V2 "starts from the `v1` tag"; §1A says the tag is carried over and does not exist | Baseline is the `main` SHA recorded in Step 0.2; frozen-file checks diff against it |
| 2 | §22.4 scopes any `wgpu` build `cfg` to the render bundle, while `Renderer::render` takes `&web_sys::VideoFrame` on every target. With the locked `web-sys` (0.3.106) that type is not gated, so §22.4 stands; and `build-wasm.sh` sets `RUSTFLAGS` itself, which a `.cargo/config.toml` cannot add to | Step 7.1: test the lock; a flag for `wgpu` goes on the `RUSTFLAGS` line of `build-wasm.sh`; `.cargo/config.toml` only if another `web-sys` gates the type |
| 3 | The pool table (§15.7) names three scripts at S4; two do not exist until S6 and S11 | Rows added in Steps 3.5, 5.3, 8.2 |
| 4 | `pool.preload()` calls `preloadRender()` (S4) before `load-render.ts` exists (S10) | Call added in Step 7.4 |
| 5 | `CoreApi.normalizeTranscript` is typed at S3; `text_api.rs` is S6 | Added in Step 5.1 |
| 6 | `render.worker.ts` (S11) needs `preview-loop.ts` (S12) | Three preview handlers added in Step 9.4 |
| 7 | S13 lists `messages.ts` once; V1 D-13 forbids a pending code that has copy, and S5 already shows the model errors | Copy and `COPY_PENDING` change with their component: Steps 4.5, 9.2, 10.1, 10.3 |
| 8 | S14 lists the `landing.spec.ts` cases; the drop zone that makes the V1 cases false is wired in S13 | Cases replaced in Step 10.3 so `pnpm e2e` never goes red between phases |
| 9 | `globalSetup` (§22.3, §23.4) has no file in the tree, and as a config-wide hook it would download the model for `non-media` too | A setup project, `web/tests-e2e/media.setup.ts`, that `media` and `bench` depend on (Step 11.1) |
| 10 | `routes.tsx` must pass `appPath` (D-48) and is not in the §4 "CHANGED" list | Edited in Step 10.3; §4 updated in that commit |
| 11 | `docs/v2/v2buildguide.md` is not in D-46 or TS §5 | Added in Step 1.1 |
| 12 | The checks of S4, S5 and S13 run only in `pnpm dev`, where the production CSP is not enforced | Worker probe (Step 1.8) and both spikes on `vite preview` |
| 13 | With pnpm, `onnxruntime-web` is a dependency of `@huggingface/transformers` and is not resolvable from `web/` directly; §22.3 says "from the pinned package" | Resolution snippet in Step 5.2 |
| 14 | `e2e-media.yml` runs `.sh`-style commands on `windows-latest`, whose default shell is PowerShell; §22.2 calls `python`, which on the development machine is an MSYS2 3.9 | `defaults.run.shell: bash`; the Python 3.13 folder first in PATH (item 2b) |
| 15 | Hand-built demuxer and muxer tests cannot catch a shared misreading of the format | Real-clip probe (Step 2.3) and `ffprobe` on a muxed file (Step 4.6), both temporary |
| 16 | The specs give no way to drive the workers from a production build before the UI exists (TE-1, S11 spike) | Temporary `zz-spike.ts` hook, removed the same day with a removal check |
| 17 | TE-14 asks for a 90 s clip; an AAC frame past 90,000 ms fails the duration rule | `-t 89.9` in Step 12.5 |
| 18 | The spike of S6 runs on the `vite preview` origin, whose OPFS is empty and whose port the bucket's CORS did not allow | CORS origin added in Step 0.4, before day 1 (D-41 places it at the first real-asset run) |
| 19 | `ci.yml` runs on pull requests and on `main` only; §5 has the pull request at S14 | Draft pull request at Milestone 1 (item 9) |
| 20 | The deployment boxes of §26 and the E-1 reading of §24.1 query the production database; the `DATABASE_URL` a terminal has loaded is the local one | `PROD_DATABASE_URL` from `.env.deploy` (Steps 12.3, 12.7) |
| 21 | §6.3 and §6.5 do not say what a file shorter than one box header is; by §6.1, `Truncated` would make it `REJECT_CORRUPT` | `NotIsoBmff`, with an inline case (Step 2.2) |
| 22 | §20.3 says the no-typed-digit rule "already applies"; no V1 tool checks it before V8 | Step 4.5 reads the strings by eye and notes the gap |
| 23 | §9.3: the one visible event needs a word confidence of 0.80; D-42 covers only how the amount is written | Step 5.5 reads the confidence and names the decision to take |
| 24 | §16.8, step 7, restores the preview canvas after every export; nothing says what happens when none was attached, as in the S11 spike | Restore only when one was attached (Step 8.2) |
| 25 | §1A, §5, §23.1, §24.1, §25.3 and §26 name R2; the founder decided on 2026-10-08 not to use it | D-65, added to the plan in Step 1.1 (item 12) |
| 26 | TE-10's fallback is written for a runner without WebGPU; a Windows Server runner may also lack the AAC or H.264 encoder (not tested) | A second row in Step 11.5, with the same fallback |
| 27 | §22.5 lists "the `ffmpeg` CLI" on the runner and no way to get it | A SHA-256-checked archive, as `ci.yml` installs its other tools (Step 11.4) |
| 28 | TS §3 and the first draft expect `Cargo.lock` to move `wasm-bindgen`; the root `Cargo.toml` pins it exactly | Step 3.2: the failure is a resolution error, and the pin, the CLI and `ci.yml` move together |
| 29 | The first `proptest` dependency can stop `cargo deny check` at `bans` (`rand`); §22.1 gives the rule, S2 does not mention it | Wrapper added in Step 2.1 |
| 30 | §26 expects every box ticked before the tag that ticks the last one | The box is ticked in the commit the tag points to (Milestone 12) |
| 31 | The guard of §22.5, step 14, searches "the build output"; in `deploy-web` that is `.vercel/output/`, not `web/dist` | Step 11.4 |
| 32 | On the reference clip itself the verifier also fails check 5: its audio track is 187 ms shorter than its video track | Expected output of Step 8.1 lists FAIL 2, 3 and 5 |

### (c) S-steps and files placed differently from §5

| Item | §5 says | This guide | Reason |
|---|---|---|---|
| S7 (muxer) | Day 3, parallel with S5-S6 | Phase 4, with S5 | It needs only S2 and fills the upload wait; one milestone |
| `text_api.rs`, `CoreApi.normalizeTranscript` | Typed at S3, built at S6 | Step 5.1 | Cannot compile earlier |
| Pool rows `asr`, `render`; `preloadRender()` | S4 | Steps 5.3, 8.2, 7.4 | The scripts and the loader do not exist at S4 |
| Preview handlers of `render.worker.ts` | S11 | Step 9.4 (S12) | They import `preview-loop.ts` |
| Model copy, blocker copy, `COPY_PENDING` edits | S13 | Steps 4.5 and 9.2 | V1 D-13; the model panel is built in S5 |
| `landing.spec.ts` replaced cases | S14 | Step 10.3 (S13) | Keeps `pnpm e2e` green at Milestone 10 |
| `verify/*` | S11, after the worker files | Step 8.1, first in the phase | The verifier must fail on known-bad files before it judges the spike |
| CORS origin `http://localhost:4173` | D-41, first real-asset run | Step 0.4 | Needed by the Phase 5 spike |
| Analytics `track` calls | S13 | Step 9.5 (S12) | §19 puts them in the use-cases |
| Worker probe (Step 1.8) | Not in §5 | Phase 1 | Proves the riskiest platform assumptions on day 1, under the production CSP |
| Pull request | S14 | Draft at Milestone 1, marked ready in Step 11.5 | `ci.yml` does not run on a branch without one |
| TE-3 and TE-4 readings | S11, on R1 and R2 | The R1 session of Step 8.5 | No R2 (item 12); the temporary build must still be on R1 when they are taken |
| Download of the test assets | `globalSetup` (§22.3) | Setup project `media-setup` (Step 11.1) | Keeps `pnpm e2e` and the Linux job free of a download of up to 260 MB |