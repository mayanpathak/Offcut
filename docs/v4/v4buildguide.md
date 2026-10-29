# v4buildguide.md - Offcut V4: step-by-step build guide

## What this is

`docs/v4/v4implementation.md` says what each V4 file contains and why. This guide says what to do next, which command to run, and how to know it worked. V4 is built on the branch `v4-build`, which already holds spec v1.2 (`ef1c42d`), the V4 plan (`16b3a4c`, revised in `1b9db25`) and `origin/main` at `550ff8f` merged in (`81474bf`), and has a draft pull request, #7, into `main`. V4 ends with that pull request merged, `ci.yml` deploying it, every box of §23 ticked, the founder's two design reviews done, and the tag `v4` pushed.

V4 is the version that decides how every short looks and how it feels to make one. Two kinds of check run through this guide: **machine checks** (tests, counts, budgets) at every milestone, and **human reviews** at three gates where only a person can judge (Step 8.1's line-break reading, Phase 11's design review, Phase 19's exit review). A human gate is a milestone like the others: the next phase does not start until it passes.

## How to read the references

| Notation | Means |
|---|---|
| `§n` | Section `n` of `docs/v4/v4implementation.md` |
| `TS §n` | Section `n` of `docs/technicalspec.md` (spec v1.2; §40 is the edit-plan architecture) |
| `BP §n`, `PS §n` | `docs/buildplan.md`, `docs/product.md` |
| `V3 §n`, `V3 CL item n` | `docs/v3/v3implementation.md`; a row of the tables of `docs/v3/v3changelog.md` |
| `V4 CL item n` | A row of the "Open items" or "Known issues" table of `docs/v4/v4changelog.md` |
| `D-n` | A decision: D-1 to D-112 are V1-V3's, D-113 to D-193 are V4's (§2) |
| `S-n` (S1 to S19, S10A) | A build-order step of §5 |
| `S-1` to `S-10` (with a hyphen) | A seam of TS §40.9 |
| `V-1` to `V-8` | The validation rules of TS §40.4 and §7.6 |
| `INV-n`, `P-n`, `TE-n`, `E-n` | Invariants (TS §35), privacy promises (TS §25.1), experiments (TS §37) |
| D1 | The founder's Windows machine; every V4 measurement is read there and labelled `d1` (D-114) |
| "keyed build" | A web build carrying `VITE_ENTITLEMENT_TEST_PUBLIC_KEY`, as V2 and V3 used it (D-72) |
| "the frozen check" | The command of Step 1.6 |
| "the identity reference" | The six display-list files of D-117, written in Phase 2 and compared in Phase 6 |
| lp | Logical pixels of the 1080 x 1920 canvas (TS §19.1) |

## How the guide is organised

Twenty phases, numbered 0 to 19. Each phase has a goal, numbered steps, a "Prove the guards" step where a guard is new, and ends with a milestone: commands with their expected output, a `Pass:` line and a commit message. **Do not start a phase while the previous milestone fails.** Every milestone except Milestone 0 ends in a commit on `v4-build`, pushed to `origin v4-build` (D-115), with CI read on PR #7 before the next phase (`gh pr checks 7`). Commit titles carry the S-number (`V4 S5: …`); when the work is done through `coding-promptsv4.md`, the prompt number goes in front as in V3 (`V4 P96 (S5): …`).

## Phase overview

| Phase | Covers (S-steps and sections) | Plan day | Milestone in one line |
|---|---|---|---|
| 0 Environment and V3 baseline | §1A, §3 | Day 1 (morning) | Tools, software adapters and the V3 counts 461/0/3, 92, 12, 48 recorded; dependencies dry-run clean |
| 1 Contracts, dependencies, docs | S1; §2, §3.2, §4, §19.1-§19.2 | Day 1 | `pnpm check` green with `offcut-plan` in the workspace and the tree; counts unchanged |
| 2 The identity reference | S2; D-117 | Day 1 | Six display-list files written by the V3 scene code and committed |
| 3 The four splits | S3; D-150 | Day 2 | Every split file under 400 lines; every V3 test and the 48 media cases unchanged |
| 4 Types and the plan crate | S4; §6, §7, §12 | Day 2-3 | `offcut-plan` green: catalog, prompt, validation, rules producer; generated types clean |
| 5 Scene and renderer on the new contract | S5; §8.1-§8.3, §9.1-§9.3, §11, §14.1, §15.1-§15.2 | Day 3-4 | The V4 contract compiles end to end; the reference clip previews exactly as in V3 |
| 6 The identity proof and program audio | S6; D-117, §10, D-127 | Day 4 | The identity reference matches; `prog48` equals `out48`; the 48 media cases green |
| 7 Native rendering | S7; §9.4, D-134, D-135 | Day 5 | `render_to_image` on WARP and lavapipe; the two video-only goldens; cross-adapter similarity recorded |
| 8 Caption intelligence | S8; §8.9, D-167 to D-174 | Day 5-6 | Line-break table read by the founder and passing; placement, parity, cache and hit tests green |
| 9 The look | S9; §8.4-§8.8, D-136 to D-145, D-170 | Day 6-7 | Clean (pill, shadow), Bold, Mono, four visuals, watermark; 16 goldens committed |
| 10 Design system and editor shell | S10; §17, D-177 to D-181, D-188 | Day 7-8 | Every state of §17.2 renders in the new shell with tokens, font, primitives and icons |
| 11 Design review 1 | S10A; D-191 | Day 8 | The founder approves the eight captured screens at two widths, changes made |
| 12 Preview and direct manipulation | S11; §15.2-§15.3, D-148, D-149, D-155, D-172 to D-174, D-183 | Day 8-9 | Scrub within 300 ms; caption drag snaps and moves the visuals; click a caption word opens it |
| 13 Editing | S12; §16, §17.3, D-146, D-147, D-175, D-182, D-184 to D-187 | Day 9-10 | A word edit shows within 150 ms; Change all; undo and redo; live style cards |
| 14 Robustness and its interface | S13; §14.2-§14.3, D-151 to D-153, D-159, D-189, D-190, §13 | Day 10-11 | Cancel everywhere, crash recovery, error cards, export pill; copy check green |
| 15 Unit and machine tests | S14; §20.3 | Day 11 | `rpc.test.ts`, the machine tests, `run-pipeline.test.ts`, `history.test.ts` green |
| 16 End-to-end suites | S15; §20.4 | Day 11-12 | `pnpm e2e:media` on D1 green, 76 cases, axe clean |
| 17 CI | S16; §19.3-§19.4 | Day 12 | PR #7's jobs green with the golden test running on lavapipe |
| 18 Measure | S17; §21 | Day 12 | Edit-to-preview, drift, seek, scene and bench readings in `experiments.md` |
| 19 Copy-back, design review 2, merge, deploy, tag | S18, S19; §22.3, §23, §24 | Day 13 | The founder's exit review passed; deployed page checked; `v4` pushed |

Total: 13 working days, as §5 plans.

## Read this first: 10 things that will bite you

1. **The identity reference must be written by V3's code, and the test that writes it must be rewritten before it can compare.** Phase 2 runs on V3's scene (events in, `TimeMs`, `FontId`); Phase 6 compares through the V4 contract (plan, `OutMs`, `FontKey`). The same file `crates/offcut-scene/tests/v3_reference.rs` is written twice: in Phase 2 its only job is to write the six JSON files; in Phase 6 it builds the same inputs the V4 way and compares with the files after D-117's normalisation. Commit the JSON files in Phase 2 so that Phase 5's work can never regenerate them. Applied in Steps 2.1-2.3, 6.1. The test and the files are listed in TS §5 while they exist (D-113) and deleted in Step 7.1.

2. **Phase 5 is one contract switch across eleven files in two languages, and nothing compiles in between.** `SceneInput`, `frame_at(OutMs)`, `Renderer::render(sources, list, fonts)`, `scene_api.rs`, `protocol.ts`, `load-render.ts`, `render.worker.ts`, `preview-job.ts`, `export-job.ts`, `export-loop.ts` and `run-pipeline.ts` change together. Work crate by crate with `cargo check -p <crate>` between files (Step 5.1's order), keep `Theme::default_for(Clean)` and the `FontSet` keys exactly V3's, and do not change a single visual value: Phase 6 proves that nothing moved. If you see a pixel change in `pnpm dev` at Milestone 5, stop there.

3. **Goldens made before the look will change in the look.** D-135 lists 16 images; four of them were planned for S7. This guide writes only the two **video-only** goldens in Phase 7 (`crop_landscape_050.png`, `punch_in_110.png`, lists with a `Video` command and nothing else), which no later phase can change, and the other 14 in Phase 9 once Clean, Bold and Mono have their final values. Recorded in "Assumptions (c)".

4. **Native wgpu must never reach the browser bundle.** The `vulkan` and `dx12` features are dev-dependency features of `offcut-render` only (D-134). After every `Cargo.toml` change, run Step 1.3's two tree checks; a native backend in the `wasm32` normal tree is a bug, even if the build passes.

5. **Snapshots and goldens are reviewed, never blessed.** `insta` writes `.snap.new` files when a snapshot changes; review them with `cargo insta review` (or read the `.snap.new` and rename it by hand), and record every changed snapshot after its first commit in `v4changelog.md` with the reason (D-157, §18). CI runs with `INSTA_UPDATE=no` and `CI=true`; a pending snapshot fails the job. The same rule holds for goldens written with `OFFCUT_GOLDEN_UPDATE=1`.

6. **The line-break table is frozen before the algorithm, like V3's labels.** `crates/offcut-scene/tests/line_breaks.rs`'s 30 rows (sentence, expected chunks and lines in Clean and Bold) are written by reading, committed, and ten of them read by the founder **before** `readability.rs` exists (Steps 8.1-8.2). After that an expected break changes only with a changelog line naming the row and the reason, never to match the output (D-167, §23).

7. **Two shared types change, and the second one touches the server.** `EditState` gains `caption_position` and `highlight_numbers` with serde defaults (D-169, D-171); `ReviewActionKind` gains five members in `offcut-api-types/src/analytics.rs` (§13), so the server's tests run against them in CI and `deploy-api` runs on the merge. After either change: `pnpm gen:types && sh scripts/check-gen-clean.sh`, and update `clip-store.ts`'s default `edit` in the same step (`tsc` fails otherwise).

8. **The design review blocks the interface.** Phase 10 builds the shell, the tokens, the font, the primitives and the icons with every state reachable, and Phase 11 is the founder's review of eight captured screens at 1280 x 800 and 1920 x 1080. No review flow, drag, transcript editor or style card is wired before Phase 11 passes (§5). The screens are captured by `tests-e2e/screens.spec.ts`, which needs dev-only hooks to reach the error and exporting states (Step 10.8); those hooks and the `data-caption-*` mirrors must be absent from `web/dist` (Step 16.6).

9. **The interface font must be a hashed asset.** `vercel.json` rewrites every path outside `/api/`, `/assets/` and `/ort/` to `index.html`; a file in `web/public/fonts/` would be served as HTML in production. The font lives in `web/src/ui/assets/` and is referenced from `tokens.css`, so Vite emits it under `/assets/` with a hash (D-180). `vercel.json` stays untouched. Check in Step 10.2 and again on the deployed page in Step 19.6.

10. **Commit and push only on `v4-build`, and read CI after each push.** `git branch --show-current` before every commit; `git push` (upstream is `origin/v4-build`); then `gh pr checks 7 --watch` (about 30 minutes for both jobs, D-115). The draft pull request is how CI runs: `ci.yml` runs on pull requests and on `main` only. Merging, pushing to `main` and tagging stay the founder's.

---

## Phase 0 - Environment and V3 baseline (§1A, §3)

Goal: every tool V4 needs prints its version on D1, a software adapter renders natively on D1 and on the Linux runner image, the V3 baseline passes on `v4-build` with its recorded counts, the new dependencies are dry-run clean, and every input that waits on a person is requested.

### Step 0.1 - Open the right shell and branch

```bash
echo "$SHELL"; uname -s                         # expect: /usr/bin/bash; MINGW64_NT-10.0-...
cd /c/Users/Mayan/Desktop/sh2clips/offcut
git fetch origin --tags --prune
git switch v4-build && git status --short       # expect: on v4-build, no output
git log --oneline -5                            # expect: 1b9db25 (or later) ... 81474bf, 16b3a4c, 550ff8f, ef1c42d among them
git merge-base v4-build origin/main | cut -c1-7 # expect: 550ff8f (the code baseline, §1A item 3)
git rev-parse --abbrev-ref @{upstream}          # expect: origin/v4-build
gh pr view 7 --json state,isDraft,baseRefName,headRefName --jq '.'   # expect: OPEN, true, main, v4-build
```

If `origin/main` has moved past `550ff8f` (a hotfix), merge it now (`git merge origin/main`), run Phase 0 against the merge, and write the new baseline SHA everywhere this guide says `550ff8f`.

### Step 0.2 - Audit the tools

| Tool | Check | Need | Action if it fails |
|---|---|---|---|
| Rust, wasm target | `rustc --version`; `rustup target list --installed` | `rust-toolchain.toml`'s version; `wasm32-unknown-unknown` | `rustup show`; `rustup target add wasm32-unknown-unknown` |
| wasm-bindgen CLI | `wasm-bindgen --version` | `=0.2.129` (the pin in `Cargo.toml`) | `cargo install wasm-bindgen-cli --version 0.2.129 --locked` |
| wasm-opt, cargo-deny | `wasm-opt --version`; `cargo deny --version` | As V3 | V3's installs |
| cargo-insta (optional) | `cargo insta --version` | Any recent | `cargo install cargo-insta --locked` (D1 only; CI does not need it) |
| Node, pnpm, Playwright, Chrome | As V3 Step 0.2 | As V3 | As V3 |
| ffmpeg, ffprobe | `ffmpeg -hide_banner -version \| head -1` | 9.0.x (V3) | As V3 |
| Python 3 with `fonttools` and `brotli` | `python -c "import fontTools, brotli; print(fontTools.version)"` | For the font subset of Step 0.7 only | `pip install fonttools brotli` |
| GitHub CLI | `gh --version`; `gh auth status` | Logged in (installed since V3) | `gh auth login` |

### Step 0.3 - Run the V3 baseline and record the counts (§1A item 4)

```bash
export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=...                 # as V3's media runs used it; this terminal only
pnpm install --frozen-lockfile                              # expect: Lockfile is up to date
pnpm check                                                  # expect: exit 0
pnpm fixtures                                               # expect: fixtures are up to date (or a regeneration, then up to date)
cargo test --workspace 2>&1 | awk '/^test result:/ {p+=$4; f+=$6; i+=$8} END {print p" passed, "f" failed, "i" ignored"}'
# expect: 461 passed, 0 failed, 3 ignored
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"      # expect: Tests  92 passed (92)
pnpm e2e 2>&1 | tail -3                                     # expect: 12 passed
pnpm e2e:media 2>&1 | tail -3                               # expect: 49 passed, 1 skipped (48 cases + 2 setup; the fixme of D-78)
pnpm build && ls -l web/dist/assets/ | grep -E 'offcut_(core|render)_bg-.*\.wasm'
node scripts/check-hosts.mjs 2>&1 | tail -3                 # the bundle sizes it reports: the app-shell "before" of D-180
```

Write all of it into `~/v4-baseline.txt`: the four counts, the two wasm sizes, the shell size.

### Step 0.4 - Check the V3 hand-over markers and the full files (§1A items 5, 6)

```bash
git grep -nE "\bV4\b" -- web/src crates | grep -v "\.ttf" | wc -l      # expect: 21 (11 in crates, 10 in web/src: the places §1A item 5 names)
wc -l web/src/workers/render.worker.ts web/src/workers/rpc.ts web/src/workers/render/video-source.ts crates/offcut-wasm-render/src/session.rs
# expect: 372, 377, 399, 381
git grep -n "TODO" -- crates web/src | grep -vE "TODO\(V[0-9]+\)|V[0-9]+:"   # expect: no output
```

If the marker count differs, list them (`git grep -nE "\bV4\b" -- web/src crates`) into `~/v4-baseline.txt`; every one must be gone or turned into a later version's marker by Milestone 19.

### Step 0.5 - Prove a software adapter on D1 and on the runner image (§1A item 7, D-134)

On a scratch branch, add a 30-line example that asks `wgpu` for a fallback adapter, prints its info, and renders one Vello scene (a filled circle) to a 64 x 64 texture read back to memory:

```bash
git switch -c scratch/v4-adapter
# Cargo.toml of offcut-render, temporarily:
#   [dev-dependencies] wgpu = { workspace = true, features = ["vulkan", "dx12", "wgsl"] }
#   pollster = "0.4"   (a block_on for the example only; not kept)
# crates/offcut-render/examples/adapter_probe.rs: request_adapter(force_fallback_adapter: true), print adapter.get_info(),
#   render a Vello scene to an Rgba8Unorm texture, copy to a buffer, print the centre pixel
cargo run -p offcut-render --example adapter_probe
# expect on D1: name contains "Microsoft Basic Render Driver" (WARP), device_type Cpu, centre pixel = the circle's colour
```

On the runner image, run the same example once through a throwaway workflow on the scratch branch (`workflow_dispatch`, `ubuntu-latest`, `sudo apt-get install -y mesa-vulkan-drivers`, then the `cargo run`); expect `llvmpipe` (lavapipe), `Cpu`, the same pixel. Record both adapter names, the Mesa package version and the pixel in `~/v4-baseline.txt`, then delete the branch locally and on the remote:

```bash
git switch v4-build && git branch -D scratch/v4-adapter && git push origin --delete scratch/v4-adapter 2>/dev/null; git status --short   # expect: no output
```

| Result | Do |
|---|---|
| Both render the pixel | Continue |
| D1 fails, the runner works | D-134's fallback: `golden_frames.rs` runs on the runner and is skipped on D1 with the reason printed; write it into `~/v4-baseline.txt` |
| The runner fails | Stop: try `dx12` WARP on `windows-latest` for the golden job instead; if neither works, the founder decides before Phase 1 (goldens on D1 only, run before every merge like the media suites) |

### Step 0.6 - Dry-run the new dependencies (§1A item 8, §3.2)

```bash
git switch -c scratch/v4-deps
cargo add -p offcut-scene --dev insta --features json
cargo add -p offcut-render --dev png
grep -E '^(insta|png)' crates/offcut-scene/Cargo.toml crates/offcut-render/Cargo.toml   # record the two versions
cargo deny check licenses bans 2>&1 | tail -4                # expect: licenses ok, bans ok (or the licence to add)
cargo tree -p offcut-render -e normal --target wasm32-unknown-unknown --prefix none | grep -iE '^(ash|d3d12|windows|gpu-alloc-|naga .*spv)' # expect: no output
pnpm --filter web add -D @axe-core/playwright && pnpm --filter web exec node -e 'require("@axe-core/playwright"); console.log("ok")'   # expect: ok; record the version
git checkout -- . && git clean -fd -- crates web && git switch v4-build && git branch -D scratch/v4-deps
pnpm install --frozen-lockfile && git status --short         # expect: no output
```

### Step 0.7 - Make the interface font (D-180)

Take `InterVariable.woff2` (or the TTF) from the **same Inter release** as `crates/offcut-scene/assets/fonts/Inter-Variable.ttf` (its version is in the font's name table: `python -c "from fontTools.ttLib import TTFont; print(TTFont('crates/offcut-scene/assets/fonts/Inter-Variable.ttf')['name'].getDebugName(5))"`). Subset it to Latin and keep the weight axis:

```bash
pyftsubset crates/offcut-scene/assets/fonts/Inter-Variable.ttf \
  --unicodes="U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+2000-206F,U+2074,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD" \
  --layout-features='kern,liga,calt,tnum,ss01' --flavor=woff2 --output-file="$HOME/inter-latin.woff2"
ls -l "$HOME/inter-latin.woff2"                               # expect: under 120 kB; record the size
```

Keep the file outside the repo until Step 10.2. Its licence (OFL-1.1) is the scene's font licence already in `crates/offcut-scene/assets/fonts/LICENSES.md`.

### Step 0.8 - Request the inputs that wait on a person

| Input | Do now | Needed at |
|---|---|---|
| The push rule of D-115 | Confirmed by the founder on 2026-10-22 ("commit to that branch only") | - |
| Ten rows of the line-break table (D-167) | Tell the founder it comes on day 5 | Step 8.2 |
| Design review 1: eight screens at two widths (D-191) | Book half an hour on day 8 | Phase 11 |
| Design review 2 and five goldens (D-191, §23) | Book half an hour on day 13 | Step 19.5 |
| `analytics_events` SQL after the deploy | Nothing | Step 19.6 |
| V3 CL item 7 (the Linux ffmpeg URL, about 2026-11-27) | Ask whether to move it in Phase 17 | Step 17.1 |

V4 adds no secret.

### Milestone 0

```bash
git branch --show-current                                   # expect: v4-build
git status --short                                          # expect: no output
cat ~/v4-baseline.txt                                       # expect: 461/0/3, 92, 12, 48 (+2 setup, 1 skip); wasm and shell sizes;
                                                            # 21 markers; four file lengths; both adapter names and the pixel;
                                                            # insta, png, axe versions; the font file's size and Inter version
```

Pass: every tool prints a version, a software adapter renders on D1 and on the runner image (or D-134's fallback is recorded), the counts equal V3's close, the scratch branches left nothing, and the font file exists outside the repo.

Commit: none. Phase 0 changes no tracked file; its records go into Entry 1 of the changelog in Step 1.1.

Do not start Phase 1 until this passes.

---

## Phase 1 - Contracts, dependencies and docs (S1)

Goal: `v4-build` holds the V4 docs, the TS §5 tree with every new file, TS §7's graph and TS §2's row for `offcut-plan`, an empty `offcut-plan` crate in the workspace, the dependencies, and the frozen check, with `pnpm check` green and every count unchanged.

### Step 1.1 - Create the changelog's Entry 1 and the experiments file

`docs/v4/v4changelog.md` exists (Entries 0, 0.1, 0.2, 0.3). Append:

```markdown
## Entry 1 - <date> - S1: baseline
| Item | Value |
|---|---|
| Code baseline | 550ff8f<rest of the SHA> (origin/main merged into v4-build at 81474bf) |
| Rust tests at baseline | 461 passed, 0 failed, 3 ignored |
| Vitest at baseline | 92 |
| `pnpm e2e` at baseline | 12 |
| `pnpm e2e:media` at baseline (D1, keyed) | 48 cases (49 passed with setup, 1 fixme skip) |
| `offcut_core_bg`, `offcut_render_bg` size | <bytes>, <bytes> |
| App shell (check-hosts) | <kB gzip> |
| Software adapters | D1: <name>; runner: <name>, mesa-vulkan-drivers <version> |
| `insta`, `png`, `@axe-core/playwright` | <version>, <version>, <version> |
| Interface font | Inter <version>, Latin subset, <bytes> |
| V3 markers found | 21 |
```

`docs/v4/experiments.md`: one heading per row of §21.1 (`Edit to preview`, `Style thumbnails`, `Hit test and hover`, `App shell with the font`, `Preview drift and late frames`, `First preview frame`, `Seek to first frame`, `Downscale rule`, `build_scene and frame_at`, `detect_scene and render_encode`, `Golden similarity across adapters`, `offcut_render.wasm size`) and the two decisions of §21.2, each with the lines `Question:`, `Method:`, `Numbers:`, `Date:`, `Decision:` left empty.

### Step 1.2 - Update the spec tree, the graph and the crate row (D-113, D-122, §4)

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `docs/technicalspec.md` §5 | §4 | Every **NEW** path of §4, with a one-line purpose in the style of the tree: `crates/offcut-plan/` and its files and tests; the new scene files (`theme.rs`, `motion.rs`, `video.rs`, `watermark.rs`, `summary.rs`, `readability.rs`, `placement.rs`, `hit.rs`, `captions_draw.rs`, `events/list_reveal.rs`, `events/from_to.rs`, `events/keyword_pop.rs`) and tests; `offcut-types/src/timemap.rs`, `plan.rs`; `offcut-render/tests/golden_frames.rs`, `tests/common/ssim.rs`; `offcut-dsp/src/program.rs`, `silence.rs`, `tests/program_identity.rs`; `offcut-wasm-render/src/scene_api.rs`; every new web file (workers, use-cases, state, layout, components, primitives, icons, hooks, styles, the font asset, `web/LICENSES.md`, the three new E2E suites and `screens.spec.ts`); `fixtures/golden/`; `docs/v4/v4buildguide.md`, `coding-promptsv4.md`, `experiments.md`; the temporary `crates/offcut-scene/tests/v3_reference.rs` and `tests/snapshots/v3_reference/` marked "temporary, V4 S2 to S7" |
| 2 | `docs/technicalspec.md` §5 | §4, §24 item 19 | Rename `EventToggleList.tsx` to `PlanItemList.tsx`; replace `ReviewPanel.tsx` with `layout/SidePanel.tsx` |
| 3 | `docs/technicalspec.md` §7 | D-122 | The graph gains `offcut-plan` with edges `offcut-scene → offcut-plan`, `offcut-plan → offcut-text`, `offcut-plan → offcut-types`, `offcut-wasm-render → offcut-plan` |
| 4 | `docs/technicalspec.md` §2 | D-122 | The `offcut-plan` row: may use `offcut-types`, `offcut-text`, `serde`, `thiserror`, and `serde_json` behind the feature `prompt` |

```bash
node scripts/check-file-tree.mjs                            # expect: exit 0 (a listed but missing file passes, V3 CL known issue 2)
```

### Step 1.3 - The workspace and the dependencies (§3.2, §19.1)

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `crates/offcut-plan/Cargo.toml`, `src/lib.rs` | §7.1 | The crate with `[lints] workspace = true`, its dependencies of D-122, `[features] default = ["prompt"]`, `prompt = ["dep:serde_json"]`; `lib.rs` holds only the crate doc comment for now |
| 2 | `Cargo.toml` | §19.1 | Member `crates/offcut-plan`; `[workspace.dependencies]` `offcut-plan = { path = "crates/offcut-plan" }`, `insta = { version = "<0.6's>", features = ["json"] }`, `png = "<0.6's>"` |
| 3 | `crates/offcut-render/Cargo.toml` | D-134 | `[dev-dependencies]` `wgpu = { workspace = true, features = ["vulkan", "dx12", "wgsl"] }`, `png = { workspace = true }` |
| 4 | `deny.toml` | §3.2 | Only the licences `cargo deny` names |
| 5 | `scripts/check-file-tree.mjs` | §19.2 | `PURE_CRATES` gains `offcut-plan` |

If the workspace denies `unused_crate_dependencies`, add each dependency in the phase whose code uses it instead (V3's rule).

```bash
cargo build --workspace                                     # expect: Finished
cargo deny check                                            # expect: advisories ok, bans ok, licenses ok, sources ok
cargo tree -p offcut-render -e normal --target wasm32-unknown-unknown --prefix none | grep -iE '^(ash|d3d12|windows|gpu-alloc-)'   # expect: no output
cargo tree -p offcut-plan -e normal --prefix none | sort -u | grep -E '^(web-sys|js-sys|wasm-bindgen|getrandom|rand|wgpu) '      # expect: no output
cargo build -p offcut-plan --target wasm32-unknown-unknown --no-default-features --release   # expect: Finished
```

### Step 1.4 - The two Playwright and package additions

`web/package.json` dev dependency `@axe-core/playwright` at the version of Step 0.6. Nothing else in `web/` changes in S1.

### Step 1.5 - Record what will be reopened

Add to the changelog's "Known issues" table, one row each, the reopened contracts this version will touch and the phase that touches them: `units.rs` (Phase 4), `edit.rs` (Phase 4), `display_list.rs` / `FontId` (Phase 5), `Renderer::render` (Phase 5), `protocol.ts` (Phases 5, 6, 12), `AudioWorkerApi` (Phase 6), `styles.rs` (Phase 5), `eslint.config.js` MINTING and use-case edges (Phases 5, 13), `analytics.rs` (Phase 14). Each row is closed by the phase that does it.

### Step 1.6 - Define the frozen check

Run at every milestone from now on. It lists every file §4 says V4 must not touch.

```bash
git diff --name-only 550ff8f -- \
  server crates/offcut-entitlement crates/offcut-mp4 crates/offcut-text crates/offcut-detect \
  crates/offcut-api-types/src/lib.rs crates/offcut-api-types/src/errors.rs \
  crates/offcut-dsp/src/chain.rs crates/offcut-dsp/src/config.rs crates/offcut-dsp/src/highpass.rs crates/offcut-dsp/src/denoise.rs \
  crates/offcut-dsp/src/compressor.rs crates/offcut-dsp/src/limiter.rs crates/offcut-dsp/src/loudness.rs crates/offcut-dsp/src/prosody.rs \
  crates/offcut-dsp/src/resample.rs crates/offcut-dsp/tests \
  crates/offcut-wasm-core/src/media_api.rs crates/offcut-wasm-core/src/hash_api.rs crates/offcut-wasm-core/src/text_api.rs \
  crates/offcut-wasm-render/src/lib.rs crates/offcut-wasm-render/src/profile_api.rs crates/offcut-wasm-render/src/mux_api.rs \
  crates/offcut-wasm-render/src/detect_api.rs \
  web/vercel.json web/src/persistence web/src/net web/src/analytics/client.ts web/src/models web/src/platform \
  web/src/state/capability-store.ts web/src/state/model-store.ts web/src/state/export-store.ts web/src/state/blockers.ts \
  web/src/state/machines/transition.ts web/src/state/machines/clip-machine.ts web/src/state/machines/model-machine.ts \
  web/src/state/machines/export-machine.ts \
  web/src/workers/media.worker.ts web/src/workers/asr.worker.ts web/src/workers/media web/src/workers/asr \
  web/src/workers/render/encoders.ts web/src/workers/render/opfs-sink.ts \
  verify bench/device-bench.ts corpus fixtures/gen_fixtures.sh fixtures/manifest.json fixtures/labeled fixtures/speech render.yaml
# expect: no output, except crates/offcut-dsp/tests/program_identity.rs from Phase 6 on (a new file: allowed)
```

And the positive view: `git diff --name-only 550ff8f | sort` lists only files of §4, the docs, and from Phase 18 `bench/results/`. It never lists `.env*`, `testclips/`, `fixtures/.gen/`, `fixtures/.cache/`, `corpus/.clips/`, `corpus/.results/`, `web/test-results/` or `web/playwright-report/`.

### Step 1.7 - Prove the guards

| Temporary edit | Rule that must fire |
|---|---|
| `touch crates/offcut-plan/src/stray.rs` | `node scripts/check-file-tree.mjs` fails naming it |
| Add `web-sys = { workspace = true }` to `offcut-plan`'s dependencies | The purity check of `check-file-tree.mjs` (or Step 1.3's tree grep) fails |
| Move `vulkan` into `offcut-render`'s normal `wgpu` features | Step 1.3's wasm32 tree grep prints a line |

Undo all three; `git status --short` shows only this phase's intended files.

### Milestone 1

```bash
pnpm check                                                  # expect: exit 0
cargo deny check                                            # expect: exit 0
cargo test --workspace 2>&1 | awk '/^test result:/ {p+=$4; f+=$6; i+=$8} END {print p" passed, "f" failed, "i" ignored"}'
# expect: 461 passed, 0 failed, 3 ignored
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"      # expect: Tests  92 passed (92)
pnpm e2e 2>&1 | tail -2                                     # expect: 12 passed
# the frozen check (Step 1.6)                               # expect: no output
git push && gh pr checks 7 --watch                          # expect: ci pass, e2e-media pass
```

Pass: all checks exit 0, the counts equal the baseline, the frozen check prints nothing, CI is green.

Commit: `"V4 S1: offcut-plan crate, dependencies, spec tree and graph, changelog Entry 1"`

Do not start Phase 2 until this passes.

---

## Phase 2 - The identity reference (S2)

Goal: six display-list files written by V3's scene code are committed, so that Phase 6 can prove the contract switch changed nothing (D-116, D-117).

### Step 2.1 - Write the reference test against V3's API

`crates/offcut-scene/tests/v3_reference.rs` (temporary). Six inputs, each built with V3's `build_scene(SceneInput { transcript, events, edit: &EditState::default(), clip, profile })`:

| Name | Transcript | Events | Clip | Profile width |
|---|---|---|---|---|
| `busy_1080` | The `busy()` words of `lib.rs`'s tests (copy them; a test file cannot reach `#[cfg(test)]` helpers) | Its three NumberReveals | 1080 x 1920, 10 s | 1080 |
| `busy_540` | Same | Same | Same | 540 |
| `reference_1080` | The reference clip's words as recognized, with their times, from `fixtures/speech/README.md` "The V2 reference clip as recognized" (copied into the test as a constant) | `offcut_detect::detect` over it with zero prosody (dev-dependency `offcut-detect`) | 1920 x 1080, 74,705 ms | 1080 |
| `a01_1080`, `w01_1080`, `w07_1080` | The labeled file's sentences with times synthesized as `precision_recall.rs` does (V3 §19.12) | `detect` with the file's prosody marks | 1080 x 1920, the synthesized length | 1080 |

For each, the frames `0, 7, 33, …` every 7th frame to the end (a prime step: every chunk and every event phase is sampled), serialized with `serde_json::to_string(&scene.frame_at(FrameIdx::new(n).to_time_ms()))`, written one JSON array per input to `crates/offcut-scene/tests/snapshots/v3_reference/<name>.json`, only when `OFFCUT_V3_REFERENCE_WRITE=1`; otherwise the test reads and compares (so a second run proves the writer is deterministic).

| Watch for | Why |
|---|---|
| Write nothing outside `tests/snapshots/v3_reference/` | The files are listed in TS §5 since Step 1.2 |
| `offcut-detect` and `serde_json` as dev-dependencies of `offcut-scene` | The writer needs them; §3.2 lists `offcut-detect` (dev) anyway |
| No `unwrap` concerns | It is a test file |

### Step 2.2 - Write, then compare

```bash
OFFCUT_V3_REFERENCE_WRITE=1 cargo test -p offcut-scene --test v3_reference   # expect: 1 passed; six files written
ls -l crates/offcut-scene/tests/snapshots/v3_reference/      # expect: six .json files, each non-empty
cargo test -p offcut-scene --test v3_reference              # expect: 1 passed (read and compared)
```

### Step 2.3 - Freeze them

```bash
git add crates/offcut-scene/tests/v3_reference.rs crates/offcut-scene/tests/snapshots/v3_reference crates/offcut-scene/Cargo.toml
```

From this commit until Step 7.1 the six files must not change. A change shows in `git diff --stat <this commit> -- crates/offcut-scene/tests/snapshots/v3_reference` and means the switch moved something.

### Milestone 2

```bash
cargo test -p offcut-scene --test v3_reference              # expect: 1 passed
git diff --name-only 550ff8f -- crates/offcut-scene/src     # expect: no output (no scene source changed yet)
pnpm check                                                  # expect: exit 0
# the frozen check                                          # expect: no output
git push && gh pr checks 7 --watch                          # expect: green
```

Pass: the six reference files exist, compare equal on a second run, and were written while every scene source was V3's.

Commit: `"V4 S2: the identity reference, written by the V3 scene"`. Write the commit's SHA into the next changelog entry as "identity reference frozen at <SHA>".

Do not start Phase 3 until this passes.

---

## Phase 3 - The four splits (S3, D-150)

Goal: the four full files give part of their bodies to new files with no change of behaviour, so V4's additions fit under 400 lines. Every V3 test and the 48 media cases pass unchanged.

### Step 3.1 - `rpc.ts` into `rpc.ts` and `rpc-worker.ts`

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/src/workers/rpc-worker.ts` | §14.2 | Move `serveWorker`, `WorkerFailure`, `CANCELLED`, `toAppFailure`, `JobContext`, `Handlers` and their private helpers, unchanged |
| 2 | `web/src/workers/rpc.ts` | §14.2 | Keeps `createClient`, `Client`, `WorkerCallError`, `Cancelled` and the message types both sides use; re-exports nothing from `rpc-worker.ts` (a page-side import must not pull worker code) |
| 3 | The four `*.worker.ts` files and `render/*.ts` | - | Import `serveWorker`, `WorkerFailure`, `CANCELLED` from `./rpc-worker` (`../rpc-worker` under `render/`) |

```bash
pnpm check                                                   # expect: exit 0
git grep -n "from \"\./rpc\"" -- web/src/workers/*.worker.ts # expect: only type imports, if any
```

### Step 3.2 - `video-source.ts` into `video-source.ts` and `frame-decoder.ts`

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/src/workers/render/frame-decoder.ts` | §15.3 | A class `FrameDecoder` owning the `VideoDecoder`: create, configure (the colour-space memory of D-70 moves with it), feed samples, drain, restart at a keyframe index, close; an output callback the source gives it; the generation counter that closes stale output |
| 2 | `web/src/workers/render/video-source.ts` | §15.3 | Keeps the queue, `frameAt`, `frameAtBlocking`, the shown frame, `liveFrames`, `close`; delegates decoding to a `FrameDecoder` |

The public surface of `VideoSource` (constructor, `frameAt`, `frameAtBlocking`, `prefetch` stub, `close`, `liveFrames`) does not change.

### Step 3.3 - `render.worker.ts` into the worker and two job files

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/src/workers/render/export-job.ts` | §15.1 | `writeExport`, `afterExport`, `exportClip`, `ExportRun`, unchanged; they receive the worker's session state through one argument (`state: () => Session`, `scene: () => SceneState`), never by importing the worker file |
| 2 | `web/src/workers/render/preview-job.ts` | §15.1 | `previewPlay`, `previewClock`, `previewPause`, `previewTime`, the loop's stop flag; the same argument style |
| 3 | `web/src/workers/render.worker.ts` | §15.1 | Keeps `open`, `scene`, `previewCanvas`, `openSession`, `detect`, `setScene`, `attachPreview`, `closeSession`, `closeFrames`, `staged`, and the handler table that now points at the two job files |

### Step 3.4 - `session.rs` into `session.rs` and `scene_api.rs`

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `crates/offcut-wasm-render/src/scene_api.rs` | §11.1 | A second `#[wasm_bindgen] impl RenderSession` block holding `set_scene` and `summary`, and the `EditInput` and `SceneInputValue` structs; `mod scene_api;` in `lib.rs` (the `lib.rs` edit is the one line; `lib.rs` is otherwise frozen) |
| 2 | `crates/offcut-wasm-render/src/session.rs` | §11.1 | Keeps `JsRandomAccess`, the error mapping, `RenderSession`'s struct, `open`, `attach_canvas`, `render_frame`, `frame_count`, the four decoder calls; its fields become `pub(crate)` for `scene_api.rs` |

`wasm-bindgen` accepts several `impl` blocks of one exported struct; the generated JavaScript is the same class.

### Step 3.5 - Check sizes and behaviour

```bash
for f in web/src/workers/rpc.ts web/src/workers/rpc-worker.ts web/src/workers/render/video-source.ts web/src/workers/render/frame-decoder.ts \
         web/src/workers/render.worker.ts web/src/workers/render/export-job.ts web/src/workers/render/preview-job.ts \
         crates/offcut-wasm-render/src/session.rs crates/offcut-wasm-render/src/scene_api.rs; do printf '%5s %s\n' "$(wc -l < "$f")" "$f"; done
# expect: every number under 400, and room left in the four old files (target: each under 300)
pnpm build:wasm && pnpm check && pnpm test                   # expect: exit 0; Rust 461/0/3; Vitest 92
pnpm e2e 2>&1 | tail -2                                      # expect: 12 passed
pnpm e2e:media 2>&1 | tail -3                                # expect: 49 passed, 1 skipped (D1, keyed)
```

### Milestone 3

```bash
pnpm check && pnpm test                                      # expect: exit 0, counts as Milestone 1
git diff --stat 550ff8f -- web/tests-e2e crates/*/tests      # expect: no output (no test changed)
# the frozen check                                           # expect: no output
git push && gh pr checks 7 --watch                           # expect: green
```

Pass: nine files under 400 lines, no test changed, every count and the 48 media cases unchanged.

Commit: `"V4 S3: split rpc, video-source, render.worker and session (no behaviour change)"`

Do not start Phase 4 until this passes.

---

## Phase 4 - Types and the plan crate (S4)

Goal: `OutMs`, `TimeMap`, the plan's wire format, `EditState`'s two fields, and the whole `offcut-plan` crate exist and are tested natively; the generated TypeScript is clean; the prompt and schema snapshots are reviewed and committed. Nothing calls the crate yet.

### Step 4.1 - Units and the time map (§6.1, §6.2, D-125)

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-types/src/units.rs` | §6.1 | `OutMs`, `OutSpan` through the same macro as `TimeMs`; rename `FrameIdx::to_time_ms` to `to_out_ms` returning `OutMs`; fix its two callers (`cargo check --workspace` names them). **REOPENED V1 CONTRACT** |
| 2 | `crates/offcut-types/src/timemap.rs` | §6.2 | `Segment`, `SrcPos`, `TimeMap` (private field, the validating `Deserialize`), `SilenceProfile`; inline tests of §6.2 |
| 3 | `crates/offcut-types/src/lib.rs` | §4 | `mod timemap; pub use timemap::*;` |

```bash
cargo test -p offcut-types                                   # expect: all pass; the V3 tests unchanged except the rename
```

### Step 4.2 - The plan's wire format and `EditState` (§6.3, §6.4, D-119, D-120, D-169, D-171)

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-types/src/plan.rs` | §6.3 | Every type; the serde rules (adjacent tagging of `ItemParams`, transparent `Label`, `deny_unknown_fields`); the inline tests: a round trip with every arm, unknown-field refusal at every level, the exact JSON of a one-item rules plan |
| 2 | `crates/offcut-types/src/edit.rs` | §6.4 | `CaptionPlacement` (checked like `CropOffset`); the two fields with their serde defaults; a test that V3's JSON (`{"word_edits":{},"event_overrides":{},"style":"clean","crop_offset":0}`) reads with `caption_position` 1.0 and `highlight_numbers` true. **REOPENED V1 CONTRACT** |
| 3 | `crates/offcut-types/src/events.rs` | §6.5 | The doc comment of D-118 only |

### Step 4.3 - Generate and fix the TypeScript side

```bash
pnpm gen:types && sh scripts/check-gen-clean.sh              # the generator writes; the check compares
git diff --stat web/src/gen                                  # expect: domain.ts grows by the new types and constants (§12)
```

| Order | File | Watch for |
|---|---|---|
| 1 | `web/src/state/clip-store.ts` | `empty().edit` gains `caption_position: 1` and `highlight_numbers: true` (plain values in the generated type, as `crop_offset` is); `tsc` fails until this is done |
| 2 | Any other place that builds an `EditState` | `git grep -n "crop_offset: 0" -- web/src web/tests-e2e` |

```bash
pnpm check                                                   # expect: exit 0
```

### Step 4.4 - The plan crate (§7)

Write the files in this order; `cargo test -p offcut-plan` after each.

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `src/catalog.rs` | §7.2 | The four rows exactly as §7.2's table; `spec(kind)`; an inline test that the rows are in `EventKind` code order and unique |
| 2 | `src/config.rs` | §7.4, D-165 | `PlanConfig::default()` from the catalog |
| 3 | `src/resolve.rs` | §7.5 | Its four inline tests (§7.5) |
| 4 | `src/validate.rs` | §7.6, Appendix C.1 | Steps 1-6 in order; `ValidItem.marks` (§8.7); every `DropReason`; no `unwrap`, no indexing; if it nears 400 lines, the per-kind parameter checks go to `validate_params.rs` (already in the tree) |
| 5 | `src/from_rules.rs` | §7.7 | - |
| 6 | `src/prompt.rs` | §7.3 | Behind `#[cfg(feature = "prompt")]`; deterministic text; `serde_json::json!` for the schema |
| 7 | `src/lib.rs` | §7.1 | Module list and re-exports |
| 8 | `tests/validate.rs` | §7.8 | 21 rule cases, the whole model plan on `a01`, the edited number (INV-22), the non-item drops |
| 9 | `tests/plan_from_rules.rs`, `tests/validate_rules_plans.rs` | §7.8 | Need `offcut-detect` (dev) and the labeled files (`CARGO_MANIFEST_DIR/../../fixtures/labeled`) |
| 10 | `tests/catalog_snapshot.rs` | §7.3 | Three snapshots named with the catalog version |
| 11 | `tests/ui.rs`, `tests/ui/valid_plan_private.rs` | §7.8 | trybuild: the expected error file is committed beside the case |

```bash
cargo test -p offcut-plan                                    # expect: all pass; new snapshots pending on the first run
cargo insta review                                           # read the system prompt, the schema and the user message; accept
cargo test -p offcut-plan                                    # expect: all pass, no pending snapshot
cargo test -p offcut-plan --test validate_rules_plans -- --nocapture   # expect: "31 files, 0 items dropped"
cargo build -p offcut-plan --target wasm32-unknown-unknown --no-default-features --release   # expect: Finished
```

**Read the prompt before accepting it.** It is the text a model will read from V6.1 on: it must name the four kinds with their purposes, say "never write a number: point at the words that say it", and contain no example number of its own (D-123).

### Step 4.5 - Prove the guards

| Temporary edit | Rule that must fire |
|---|---|
| In `tests/ui/valid_plan_private.rs`, nothing; in `validate.rs`, make `ValidPlan`'s fields `pub` | `tests/ui.rs` fails (the case now compiles) |
| Change one word of `system_prompt()` | `catalog_snapshot.rs` fails |
| Give a `Model` plan a `Detected` item in a test | `validate.rs`'s case drops it with `WrongProducer` |

Undo all three.

### Milestone 4

```bash
cargo test -p offcut-types -p offcut-plan                    # expect: all pass
sh scripts/check-gen-clean.sh                                # expect: exit 0
pnpm check && pnpm test                                      # expect: exit 0; Rust above 461 by the new tests, 0 failed
# the frozen check                                           # expect: no output
git push && gh pr checks 7 --watch                           # expect: green
```

Pass: the plan crate's five test files and the types' inline tests pass, the snapshots are committed after a reading, the generated types are clean, and nothing outside the types and the new crate changed behaviour.

Commit: `"V4 S4: OutMs, TimeMap, the plan wire format, EditState fields, offcut-plan"`

Do not start Phase 5 until this passes.

---

## Phase 5 - Scene and renderer on the new contract, V3's look (S5)

Goal: the scene reads a `ValidPlan` on program time with a `FontSet` and a `Theme`, the video is a draw command, the renderer draws lists in order, and the page sends a rules plan, with every visual value still V3's. The reference clip previews in `pnpm dev` exactly as in V3.

### Step 5.1 - The Rust side, crate by crate

Work in this order; `cargo check -p <crate>` after each row, `cargo test -p <crate>` at the end of each crate.

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `offcut-scene/src/display_list.rs` | §8.2, D-130, D-132 | `FontKey` for `FontId` (**REOPENED V2 CONTRACT**); `CropRect` moved here; `Video`, `Image`, `VideoSlot`, `ImageKey`; `scale` leaves `Video` alone |
| 2 | `offcut-scene/src/fonts.rs`, `layout.rs` | §8.3 | `FontSet::embedded()` with keys 0-3 in V2's order (key 4 arrives in Phase 9); faces keep their coordinates; `shape(text, key, size, fonts)` |
| 3 | `offcut-scene/src/theme.rs`, `styles.rs`, `motion.rs` | §8.4, §8.5 | `Theme::default_for(_)` returns **V3's colours and fonts for all three styles** (V3 drew Bold and Tech as Clean); `StyleSpec` on roles; `CaptionMotion` holding V3's single motion (ease-out cubic, 120 ms, `Highlight`). **REOPENED V2 CONTRACT (`styles.rs`)** |
| 4 | `offcut-scene/src/safe_area.rs` | D-138 | `ZoneId { Event, Caption, Full }` and `zone(id)` with V3's numbers; `placement.rs` is Phase 8's |
| 5 | `offcut-scene/src/anim.rs`, `captions.rs`, `events/mod.rs`, `events/number_reveal.rs` | §8.6, §8.7 | `OutMs` and `OutSpan` throughout; layout from `ValidItem` (the `Detected` params); colours from roles |
| 6 | `offcut-scene/src/video.rs` | §8.8 | `Track::fixed(crop)`, `cmd(t, canvas)` |
| 7 | `offcut-scene/src/lib.rs` | §8.1 | `SceneInput` v1.2, `build_scene` as §8.1 without placement and readability (Phase 8), `frame_at(OutMs)` putting `Video` first, `source_at`, `fonts()`; `crop()` removed; `summary()` kept as V2's until Phase 9 |
| 8 | `offcut-scene` inline tests | D-160 | Move every V2/V3 inline test to the new calls; **no expected value changes**; the "frames" helper uses `to_out_ms` |
| 9 | `offcut-scene/tests/timemap_identity.rs` | §8.10 | The S-1 test |
| 9b | `offcut-scene/tests/v3_reference.rs` | D-117, Step 6.1 | It cannot compile on V3's API any more: rewrite it now into its comparison form (Step 6.1 describes it) and mark the test `#[ignore = "identity proof, S6"]`. Run it with `--ignored` as you work: it is the fastest way to see what the switch moved |
| 10 | `offcut-render/src/lib.rs`, `composite.rs`, `video_pass.rs`, `vello_backend.rs` | §9.1-§9.3 | `render(sources, list, fonts)`; the V4 support rule (`Unsupported` otherwise); `src` per call, `dst` rectangle uniform; glyphs by `FontKey` through the `FontSet`. **REOPENED V2 CONTRACT** |
| 11 | `offcut-wasm-render/src/scene_api.rs`, `session.rs` | §11.1 | `rules_plan`; `set_scene` validating the plan (refusal → `E_INTERNAL` `PlanFormat`/`PlanCatalog`), refusing a non-identity map (`TimeMapNotIdentity`), answering the dropped list; `render_frame(frame, t_out_ms)`; `source_at` |

```bash
cargo test -p offcut-scene -p offcut-render                  # expect: all pass with V3's expected values
cargo clippy --workspace --all-targets -- -D warnings         # expect: clean
pnpm build:wasm                                              # expect: exit 0; record offcut_render_bg's size (§21.1)
```

### Step 5.2 - The TypeScript side

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/src/workers/protocol.ts` | §14.1 | `rulesPlan`; `setScene({ transcript, plan, edit, timeMap, profile }) → SceneReport`; `ClockSync.audioMs: OutMs`; `previewSeek({ at: OutMs })`; `previewPlay → PreviewStats`. The audio worker's methods and `hitTest`, `styleThumbnails` come in Phases 6 and 12. **REOPENED V1 CONTRACT** |
| 2 | `web/src/wasm/load-render.ts` | §14.4 | `RenderApi.rulesPlan`; `RenderSession.set_scene → SceneReport`, `source_at` |
| 3 | `web/src/workers/render.worker.ts` | §15.1 | `rulesPlan` handler; `setScene` passes `plan` and `timeMap`; `detect` keeps transcript, prosody, events (D-146) |
| 4 | `web/src/workers/render/preview-job.ts`, `preview-loop.ts` | §15.2 | `OutMs`; `session.source_at(tn)` → `source.frameAt(at)`; `previewPlay` resolves with the statistics (no resize rule yet) |
| 5 | `web/src/workers/render/export-job.ts`, `export-loop.ts` | D-129 | The plan and the map in the profile switch and back; `render_frame` with the same number typed `OutMs` |
| 6 | `web/src/usecases/run-pipeline.ts` | §16.2 | After `detect`: `rulesPlan` → `clipStore.setPlan`; `timeMap = { segments: [{ kind: "source", src: { start: 0, end: duration } }] }` built through one helper that names it the identity; `setScene` with both |
| 7 | `web/src/usecases/control-preview.ts`, `web/eslint.config.js` | D-126 | `toTimeMs` becomes `toOutMs`; the `MINTING` entry moves from `TimeMs` to `OutMs`. **REOPENED V2 CONTRACT (`eslint.config.js`)** |
| 8 | `web/src/state/clip-store.ts` | §16.1 | `plan?`, `timeMap?`, `setPlan`, `setEvents` |

```bash
pnpm check                                                   # expect: exit 0
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"   # expect: 92 passed
```

### Step 5.3 - Look at it

`pnpm dev` (port 5174 if 5173 is taken, V3), drop the sample clip, wait for `ready`, play.

| Check | Expect |
|---|---|
| Captions | V3's Clean look, in Clean, Bold and Tech alike (as V3) |
| NumberReveal | Counts up as in V3; the other three kinds still not drawn |
| DevTools console | No error; the `setScene` answer has `dropped: []` |
| Feed | V3's lines |

If anything looks different from V3, stop: Phase 6's test will name the difference, but find it now.

### Milestone 5

```bash
pnpm build:wasm && pnpm check && pnpm test                   # expect: exit 0; Rust 0 failed; Vitest 92
cargo test -p offcut-scene --test v3_reference -- --ignored # information only: identical already, or the differences Phase 6 fixes
pnpm e2e 2>&1 | tail -2                                      # expect: 12 passed
# the frozen check                                           # expect: no output
git push && gh pr checks 7 --watch                           # expect: green (the ignored reference test does not run in CI)
```

Pass: the workspace and the web app compile and pass on the V4 contract, every inline expectation is V3's, and the preview looks as V3's.

Commit: `"V4 S5: scene, renderer and protocol on the plan contract (V3 look)"`

Do not start Phase 6 until this passes.

---

## Phase 6 - The identity proof and program audio (S6)

Goal: the V4 path draws exactly V3's display lists for the six reference inputs, the page plays assembled program audio equal to the cleaned voice, and the 48 V3 media cases pass.

### Step 6.1 - Make the identity reference pass (D-117)

Remove the `#[ignore]` of Step 5.1 row 9b. `crates/offcut-scene/tests/v3_reference.rs`, comparison form: the same six inputs, now built as `rules plan → validate → build_scene` on `TimeMap::identity`, `Theme::default_for(Clean)`, `FontSet::embedded()`. For each frame: serialize `frame_at(FrameIdx::new(n).to_out_ms())`, then normalise (remove the first command when it is `Video`; read `FontKey(n)` as V3's `FontId` of index `n`, i.e. rename the `"font"` field's value `{"0":k}`/`k` to V3's variant name) and compare with the JSON file's frame, value for value, with `f32` equality (no tolerance: the layout code is the same).

```bash
cargo test -p offcut-scene --test v3_reference -- --nocapture   # expect: 1 passed; prints "6 inputs, <n> frames, identical"
```

| If it fails | Where to look |
|---|---|
| Glyph positions differ | `layout.rs`: the font face or weight per key differs from V3's `FontId` |
| Colours differ | `theme.rs`: a role maps to another colour than V3's constant |
| Times differ (a chunk shows one frame early or late) | `captions.rs`: an `OutSpan` conversion rounds differently from V3's `TimeMs` arithmetic |
| An event missing | `validate.rs` dropped it (print the dropped list) or `events/mod.rs` reads the wrong arm |

### Step 6.2 - Program audio (§10, D-127, D-128)

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-dsp/src/silence.rs`, `program.rs`, `lib.rs` | §10 | `DspError::Unsupported`; identity returns a copy and the loudness from `loudness.rs`; `assemble` is the only function of the crate that may return another length |
| 2 | `crates/offcut-dsp/tests/program_identity.rs` | §10 | Reads `fixtures/.gen/speech/*.pcm48.f32` (the V3 stamp helper) |
| 3 | `crates/offcut-wasm-core/src/audio_api.rs` | §11.2 | `assemble(out48, time_map, speech)` |
| 4 | `web/src/wasm/load-core.ts` | §14.4 | `CoreApi.assemble` |
| 5 | `web/src/workers/protocol.ts`, `audio.worker.ts` | §14.1, §15.4 | `assemble` (transfer in and out), `release` (one-way; `kept = null`); `serveWorker`'s `oneWay: ["release"]`. **REOPENED V3 CONTRACT** |
| 6 | `web/src/usecases/run-pipeline.ts` | §16.2, D-127, D-128 | Step F: `opfs.writeAudio(out48)` first (it still reads the buffer), then `pool.audio.assemble` with `out48` transferred, then `clipStore.ready({ …, prog48 })`; every early end calls `pool.audio.release()` |
| 7 | `web/src/state/clip-store.ts`, `usecases/control-preview.ts`, `ui/components/PreviewPlayer.tsx` | §16.1 | `out48` → `prog48` in the store and in `attach` |

```bash
cargo test -p offcut-dsp --test program_identity             # expect: all pass
pnpm build:wasm && pnpm check                                # expect: exit 0
```

### Step 6.3 - The browser identity check

```bash
pnpm e2e:media 2>&1 | tail -3                                # expect: 49 passed, 1 skipped (the V3 media cases, unchanged expectations)
```

In `pnpm dev`, after `ready`, DevTools console in the page: `useClipStore.getState().prog48.length` equals the clip's samples at 48 kHz (duration x 48), and the preview sounds as in V3.

### Milestone 6

```bash
cargo test -p offcut-scene --test v3_reference               # expect: 1 passed (no #[ignore] left)
cargo test --workspace 2>&1 | awk '/^test result:/ {p+=$4; f+=$6; i+=$8} END {print p" passed, "f" failed, "i" ignored"}'   # expect: 0 failed; ignored 3
pnpm check && pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"   # expect: exit 0; 92 passed
pnpm e2e:media 2>&1 | tail -3                                # expect: 49 passed, 1 skipped
git diff --stat <Phase 2's commit> -- crates/offcut-scene/tests/snapshots/v3_reference   # expect: no output
# the frozen check                                           # expect: no output except program_identity.rs (new, allowed)
git push && gh pr checks 7 --watch                           # expect: green
```

Pass: the identity reference matches frame for frame, `prog48` equals `out48`, and the 48 media cases pass. Write into the changelog: "identity proof passed at S6: 6 inputs, <n> frames identical".

Commit: `"V4 S6: identity proof passed; program audio (identity) in the worker and the preview"`

Do not start Phase 7 until this passes. **If the reference differs anywhere beyond the normalisation, the switch changed a pixel: fix it in Phase 5's files, never in the reference.**

---

## Phase 7 - Native rendering (S7, D-134, D-135)

Goal: the renderer draws to memory on a software adapter, the SSIM measure exists, two video-only goldens are committed, and the similarity between D1's WARP and the runner's lavapipe is recorded.

### Step 7.1 - Retire the identity reference

```bash
git rm -r crates/offcut-scene/tests/v3_reference.rs crates/offcut-scene/tests/snapshots/v3_reference
```

Remove its lines from TS §5 in the same commit (D-113).

### Step 7.2 - Headless rendering (§9.1, §9.3)

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `offcut-render/src/gpu.rs` | §9.3 | The headless target and readback buffer, row padding to 256 bytes (wgpu's copy alignment) removed when reading back |
| 2 | `offcut-render/src/video_pass.rs` | §9.3 | `load_image(&RgbaImage, rotation, src)` beside `load` |
| 3 | `offcut-render/src/lib.rs` | §9.1 | `new_headless`, `render_to_image`, `RgbaImage`; all behind `#[cfg(not(target_arch = "wasm32"))]` |
| 4 | `offcut-render/tests/common/ssim.rs` | §9.4 | Luma (BT.709), 8 x 8 windows, C1 = (0.01 x 255)², C2 = (0.03 x 255)², the mean over windows |
| 5 | `offcut-render/tests/golden_frames.rs` | §9.4 | The pattern source; two cases now: `crop_landscape_050` (1920 x 1080 source, a list with one `Video { src: the 0.5-offset crop, dst: full }`) and `punch_in_110` (the portrait source, `src` the centre 1/1.1); the adapter-name assertion; `OFFCUT_GOLDEN_UPDATE` |

```bash
OFFCUT_GOLDEN_UPDATE=1 cargo test -p offcut-render --test golden_frames -- --nocapture   # expect: 2 written; prints the adapter name
cargo test -p offcut-render --test golden_frames             # expect: 2 passed
```

Open the two PNGs and look: the crop shows the right-of-centre strip of the pattern; the punch-in shows the centre enlarged with the grid lines thicker.

### Step 7.3 - Read the D1 images on lavapipe

```bash
git add fixtures/golden crates/offcut-render && git commit -m "V4 S7 (wip): golden_frames and two video-only goldens" && git push
gh pr checks 7 --watch                                       # the Linux job runs golden_frames on lavapipe (needs Phase 17's apt line: add it now, see below)
```

Add now (it belongs to Phase 17, but the comparison needs it): in `ci.yml`'s Linux job, before `cargo test`, `sudo apt-get install -y mesa-vulkan-drivers` and `env: INSTA_UPDATE: "no"`. Read the log line the test prints per golden (`ssim crop_landscape_050 = 0.99xx on llvmpipe`).

| Reading | Do |
|---|---|
| Both ≥ 0.98 | Continue; record the two values in `experiments.md` |
| One below 0.98 | Per-adapter folders `fixtures/golden/{warp,lavapipe}/` (§21.1): write the lavapipe images from the CI artifact (the test uploads its output on failure: add `actions/upload-artifact` for `target/golden-out/`), commit both, record why. The threshold is never lowered |

### Milestone 7

```bash
cargo test -p offcut-render                                  # expect: all pass on D1
git ls-files fixtures/golden                                 # expect: the two PNGs (or two per adapter folder)
git ls-files crates/offcut-scene/tests | grep v3_reference   # expect: no output
# the frozen check                                           # expect: no output
gh pr checks 7                                               # expect: green; the Linux log shows golden_frames "2 passed"
```

Pass: headless rendering works on both adapters, two goldens pass on both, the cross-adapter similarity is recorded, and the identity reference is gone.

Commit: `"V4 S7: native rendering, SSIM, two video-only goldens; identity reference retired"`

Do not start Phase 8 until this passes.

---

## Phase 8 - Caption intelligence (S8, D-167 to D-174)

Goal: captions break at phrase boundaries with balanced lines, stay up long enough to read, highlight the right words, can be placed at any height with the visuals following, and render identically at every output size; a correction rebuilds quickly; a point on the canvas can be hit-tested. The line-break table is written and read by the founder before the algorithm exists.

### Step 8.1 - Write the line-break table first (D-167)

`crates/offcut-scene/tests/line_breaks.rs`, the table part only: 30 rows, each a sentence (with word times: 300 ms per word, a 400 ms pause where the row says), and the expected chunks and lines in Clean (16 characters x 2, 4 words, after Phase 9's values) and Bold (12 x 2, 3 words, upper case). Cover, at least twice each:

| Case | Example sentence | Expected (Clean) |
|---|---|---|
| Article never ends a line | "I finally shipped the new onboarding flow today" | `I finally shipped / the new onboarding` · `flow today` (never `…shipped the / new…`) |
| Preposition never ends a line | "we moved everything to a single database" | `we moved everything / to a single database`… within the limits |
| Break after a comma | "honestly, it was worth every minute" | `honestly, / it was worth every` · `minute` is not alone (balanced instead) |
| Clause starter begins a chunk | "it worked because we cached everything" | a break before `because` |
| Balanced two lines | "this is the part nobody tells you about" | two lines of similar length, not 16 + 3 |
| One-word beat kept alone (pauses both sides) | "and then… nothing." | `nothing.` alone, because pauses surround it |
| One word not stranded otherwise | "we launched on Monday" | no chunk of the single word `Monday` |
| Numbers stay whole | "we went from $2k to $20k" | `$2k` and `$20k` never split |
| Sentence end is a hard break | two short sentences | never one chunk across the full stop |
| Long words | "internationalization matters" | fits by the line rule of V2 (scale), still one chunk |

Add five rows of everyday creator speech (a recipe step, a fitness cue, a product review line, a travel tip, a reaction) so the table is not only technical speech (D-177).

```bash
git add crates/offcut-scene/tests/line_breaks.rs && git commit -m "V4 S8 (table): the line-break table, before the algorithm" && git push
```

### Step 8.2 - Human gate: the founder reads ten rows

Choose ten rows at random (`shuf -n 10` over the row numbers), show the founder each sentence and its expected chunks as they would appear on a phone (two lines each), and ask: "Would you break it this way?" Record the answers in the changelog. A "no" changes that row's expectation **now**, with the founder's reason, before Step 8.3. From here on a row changes only with a changelog line (§23).

### Step 8.3 - Readability, the break search and the floor (§8.9, D-167, D-168)

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `offcut-scene/src/readability.rs` | §8.9 | The two lists sorted, a test that they are sorted and lower case; the costs and `MIN_CHUNK_MS`, `ISOLATED_WORD_PAUSE_MS` as named constants |
| 2 | `offcut-scene/src/captions.rs` | §8.9 | The search between hard breaks; ties to the earlier break; the line split inside each chunk; the reading floor after chunking |
| 3 | `tests/line_breaks.rs` | §8.10 | The property checks over the 31 labeled transcripts |

```bash
cargo test -p offcut-scene --test line_breaks -- --nocapture  # expect: 30 rows passed + the property checks; prints the chunks of any failing row
```

A failing row is fixed in the costs or the search, never in the row (unless the founder's reading of Step 8.2 changed it).

### Step 8.4 - Emphasis, placement, hit testing, the cache, parity (§8.9, D-169 to D-174)

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `offcut-scene/src/placement.rs`, `safe_area.rs` | §8.9, D-171 | `caption_block`, `event_zone`; `zone(id)` reads them for the clip's placement; `tests/placement.rs` (at 1.0 the event zone is exactly 860-1180) |
| 2 | `offcut-scene/src/captions_draw.rs` | §8.6, §8.9, D-169 | The chunk drawing moved out of `captions.rs`; the emphasis order active → KeywordPop → number → fill; `highlight_numbers` read from `EditState` |
| 3 | `offcut-scene/src/hit.rs`, `lib.rs` | §8.9, D-174 | `Scene::hit`; `tests/hit.rs` |
| 4 | `offcut-scene/src/layout.rs` | D-173 | The shaping cache and its test-only switch; `tests/scene_cache.rs` |
| 5 | `offcut-scene/tests/parity.rs` | D-172 | Six scenes at 540, 720, 1080; equal after scaling within 0.01 lp |
| 6 | `offcut-wasm-render/src/scene_api.rs` | §11.1 | `hit(t, x, y)` (wired to the worker in Phase 12) |

```bash
cargo test -p offcut-scene                                   # expect: all pass, including the inline tests (the V3 expectations hold at placement 1.0)
cargo test -p offcut-scene --release --test scene_cache -- --nocapture   # prints build times cold and warm: record both
```

### Step 8.5 - Prove the guards

| Temporary edit | Rule that must fire |
|---|---|
| Set the binder cost to 0 | `line_breaks.rs` fails on the article rows |
| Make `placement.rs` read `profile.watermark` | Its D-171 test (identical layout with and without the watermark flag) fails |
| Change the cache key to omit the size | `scene_cache.rs` fails |

Undo all three.

### Milestone 8

```bash
cargo test -p offcut-scene -p offcut-plan                    # expect: all pass
git log --oneline -3 -- crates/offcut-scene/tests/line_breaks.rs   # expect: the table commit precedes readability.rs's first commit
git log --diff-filter=A --format=%h -- crates/offcut-scene/src/readability.rs   # its SHA is newer than the table's
pnpm build:wasm && pnpm check && pnpm test                   # expect: exit 0
# the frozen check                                           # expect: no output
git push && gh pr checks 7 --watch                           # expect: green
```

| Check | Where | Expect |
|---|---|---|
| Captions read naturally | `pnpm dev`, the sample clip | No "the" or "to" at a line end; two-line chunks of similar length |
| The founder's reading | `v4changelog.md` | Ten rows, each "yes" or changed with a reason |

Pass: the table came first and was read, it passes, and every new scene test passes.

Commit: `"V4 S8: caption intelligence: phrase-aware breaks, reading floor, emphasis, placement, hit test, cache, parity"`

Do not start Phase 9 until this passes.

---

## Phase 9 - The look (S9, D-136 to D-145, D-170)

Goal: Clean is the premium default (Inter 800, shadow, the sliding highlight pill), Bold and Mono have their own looks, all four visuals are drawn, the crop offset works, the watermark draws when asked, the summary counts what is drawn, and the remaining 14 goldens are committed.

### Step 9.1 - Fonts, theme and styles (D-170, §8.4, §8.5)

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `offcut-scene/src/fonts.rs` | D-170 | Key 4 = Inter 800, appended; its coordinates computed like the others |
| 2 | `offcut-scene/src/theme.rs` | §8.4 | The table of §8.4 (Clean's pill and shadow rows included) |
| 3 | `offcut-scene/src/motion.rs` | D-137, D-170 | `ActiveTreatment::Pill`; `ShadowSpec`; `enter_at` |
| 4 | `offcut-scene/src/styles.rs` | D-170, D-139 | Clean and Bold per D-170; Tech per D-139 (the interface calls it Mono) |
| 5 | `offcut-scene/src/captions_draw.rs` | §8.9 | The pill (90 ms slide, 0.9 → 1.0), the shadow run, Bold's pop, Tech's panel and cursor, KeywordPop's scale |
| 6 | `offcut-scene/tests/styles_use_theme.rs` | D-136 | The text search |

### Step 9.2 - The visuals, framing, watermark, summary

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `events/list_reveal.rs` | D-140 | Item times through the map; the shrink rule past five items |
| 2 | `events/from_to.rs` | D-141, §8.7 | The second value's time from `ValidItem.marks` |
| 3 | `events/keyword_pop.rs`, `events/mod.rs` | §8.7 | The four arms; nothing returns `Ok(None)` for a kind any more except an undrawable item |
| 4 | `framing.rs` | D-131 | The offset used; `tests/framing.rs` |
| 5 | `watermark.rs` | D-143 | The text constant; the overlap rule at the three snap points in `layout_safe_area.rs` |
| 6 | `summary.rs`, `lib.rs` | D-144 | `voice_cleaned: true`; drawn counts |
| 7 | `tests/display_list_snapshots.rs`, `layout_safe_area.rs`, `determinism.rs`, `catalog_complete.rs` | §8.10 | The snapshot set of §8.10; review every snapshot before accepting |

```bash
cargo test -p offcut-scene                                   # expect: all pass after the snapshots are reviewed
cargo insta review                                           # read each new snapshot: kinds, styles, instants
```

### Step 9.3 - The remaining goldens and the parity case (§9.4, D-172)

Fourteen images: 3 styles x 4 kinds (the frame inside each visual, at a moment when a caption word is active, so the pill shows in Clean), Clean NumberReveal with and without the watermark; and the parity case (three styles at 1080 box-filtered to 540 against 540, SSIM ≥ 0.99).

```bash
OFFCUT_GOLDEN_UPDATE=1 cargo test -p offcut-render --test golden_frames -- --nocapture   # expect: 14 written
cargo test -p offcut-render --test golden_frames             # expect: 17 passed (16 goldens + parity)
```

**Look at all 16 PNGs.** For each, check: the pill sits behind exactly one word and does not touch the stroke of the next; the shadow is below the text, not beside it; no text is clipped by the safe area; Bold's upper case fits; Mono's panel has even padding; the list's items align; the from-to's bars start on one baseline; the watermark is legible and away from the captions. A golden that looks wrong is fixed in code, then rewritten.

### Step 9.4 - Look at it moving

`pnpm build:wasm && pnpm dev`, the sample clip, each style. The style picker arrives in Phase 13: until then change `empty().edit.style` in `clip-store.ts` to `"bold"`, then `"tech"`, reload each time, and revert before the commit (`git diff web/src/state/clip-store.ts` shows nothing).

| Check | Expect |
|---|---|
| Clean | The pill glides from word to word, never jumps; no pill during a pause |
| Bold | Words pop one by one; upper case; three words a chunk |
| Mono | The panel follows the chunk; the cursor blinks only while a word is active |
| Visuals | "3 reasons" builds its list item by item; "90 minutes to 10 minutes" shows two bars, the second in the accent |

### Milestone 9

```bash
cargo test -p offcut-scene -p offcut-render -p offcut-plan   # expect: all pass
git ls-files fixtures/golden | wc -l                         # expect: 16 (or 32 with per-adapter folders)
pnpm build:wasm && pnpm check && pnpm test && pnpm e2e        # expect: exit 0; 12 passed
pnpm e2e:media 2>&1 | tail -3                                # expect: green (the D-160 rewrites only; no expectation on looks)
# the frozen check                                           # expect: no output
git push && gh pr checks 7 --watch                           # expect: green; Linux shows golden_frames 17 passed
```

Pass: three styles and four visuals draw as D-139 to D-143 and D-170 say, 16 goldens and the parity case pass on both adapters, and every snapshot was read before it was accepted.

Commit: `"V4 S9: the look: premium Clean with the highlight pill, Bold, Mono, four visuals, framing, watermark, summary, 16 goldens"`

Do not start Phase 10 until this passes.

---

## Phase 10 - Design system and editor shell (S10, §17, D-177 to D-181, D-188)

Goal: the editor is a dark, premium shell (top bar, stage, side panel) built only from tokens and primitives, in the interface font, with every state of §17.2 reachable and looking finished, so the founder can review it in Phase 11. No editing flow is wired yet: the tabs show their real content where it already exists (the preview, the processing card) and static placeholders elsewhere.

### Step 10.1 - Tokens (D-179)

`web/src/ui/styles/tokens.css` [ONLY design values]. Keep the landing page's existing variables and add the editor's, scoped to `[data-theme="dark"]` on the editor's root, so the landing page is unchanged.

| Group | Tokens | Values (D-179) |
|---|---|---|
| Colour | `--bg`, `--surface-1`, `--surface-2`, `--border`, `--text`, `--text-2`, `--brand`, `--brand-hover`, `--ok`, `--warn`, `--danger` | `#0E0E11`, `#16161B`, `#1D1D23`, `#2A2A32`, `#F4F4F6`, `#A3A3AD`, `#8B5CF6`, `#7C4DEE`, `#34D399`, `#FBBF24`, `#F87171` |
| Type | `--fs-12` … `--fs-32`, `--fw-400` … `--fw-700`, `--lh-tight 1.2`, `--lh-body 1.5` | 12/13/14/16/20/24/32 px |
| Space | `--sp-1` … `--sp-7` | 4, 8, 12, 16, 24, 32, 48 px |
| Radius | `--r-control`, `--r-card`, `--r-panel` | 6, 10, 16 px |
| Elevation | `--shadow-1`, `--shadow-2` | a 1 px border colour plus `0 1px 2px rgb(0 0 0 / .4)`; `0 8px 24px rgb(0 0 0 / .45)` |
| Motion | `--fast`, `--base`, `--ease` | 120 ms, 200 ms, `cubic-bezier(.2,.8,.2,1)`; `@media (prefers-reduced-motion: reduce)` sets both durations to `0.01ms` for transforms |
| Focus | `--focus` | `0 0 0 2px var(--bg), 0 0 0 4px var(--brand)` (the ring with its 2 px offset) |
| Layout | `--topbar-h`, `--panel-w`, `--panel-w-sm` | 56, 360, 320 px |

Correct the file's first comment ("system fonts only") to name the self-hosted interface font (D-180).

```bash
grep -cE '^\s+--[a-z0-9-]+:' web/src/ui/styles/tokens.css    # record the count
git grep -nE '#[0-9a-fA-F]{3,8}\b' -- 'web/src/ui/**/*.css' ':!web/src/ui/styles/tokens.css'   # expect: no output (every colour a token)
```

### Step 10.2 - The interface font (D-180)

```bash
mkdir -p web/src/ui/assets && cp "$HOME/inter-latin.woff2" web/src/ui/assets/inter-latin.woff2
```

In `tokens.css`: `@font-face { font-family: "Inter UI"; src: url("../assets/inter-latin.woff2") format("woff2"); font-weight: 100 900; font-display: swap; }`, and `--font-ui: "Inter UI", system-ui, sans-serif;` used by the editor root. `web/LICENSES.md`: Inter (OFL-1.1, its copyright line from the font) and Lucide (ISC, for Step 10.4).

```bash
pnpm build && ls web/dist/assets | grep -E '^inter-latin-.*\.woff2$'   # expect: one hashed file (served under /assets/, immutable)
git grep -n "public/fonts" -- web                             # expect: no output (Read this first, item 9)
```

### Step 10.3 - Primitives (D-181)

`web/src/ui/primitives/`, one file each, styles in `primitives.module.css`. Every primitive: a visible focus ring from `--focus`, hover and pressed states, a disabled state with a reason available as a tooltip, its text from props (the caller passes `messages.ts` strings: INV-15).

| Order | Primitive | Props | Behaviour to get right |
|---|---|---|---|
| 1 | `Button` | `variant: primary \| secondary \| ghost \| danger`, `size: sm \| md`, `busy` | `busy` shows a spinner after 300 ms and keeps the width |
| 2 | `IconButton` | `icon`, `label`, `size` | `aria-label` = `label`; the tooltip shows `label` and its shortcut (`Kbd`) |
| 3 | `Tooltip` | `content`, `children` | After 400 ms on hover, at once on keyboard focus; `role="tooltip"`, `aria-describedby` |
| 4 | `Tabs` | `tabs`, `selected`, `onSelect` | `role="tablist"`, roving `tabindex`, ← → Home End move, Enter/Space selects |
| 5 | `Switch` | `checked`, `label`, `hint` | `role="switch"`, `aria-checked`, the whole row clickable |
| 6 | `SegmentedControl` | `options`, `value` | Radio-group semantics, arrow keys |
| 7 | `Slider` | `min`, `max`, `step`, `value`, `onCommit` | Native `input type="range"`; `onCommit` on `change` (release), live value shown while dragging |
| 8 | `Toast` and `ToastHost` | `message`, `action?`, `onAction?` | Bottom centre, 5 s, paused on hover and focus, one stack, announced through the live region |
| 9 | `Spinner`, `Skeleton` | `size` | `Skeleton` shimmer off under reduced motion |
| 10 | `PanelSection` | `title`, `hint?`, `children` | Title at `--fs-13` `--fw-600` `--text-2` upper-case tracking; consistent `--sp-4` padding |
| 11 | `Kbd` | `keys` | `⌘` on macOS, `Ctrl` elsewhere (`navigator.userAgentData.platform`) |

### Step 10.4 - Icons (D-181)

`web/src/ui/icons.tsx`: one component per icon, `viewBox="0 0 24 24"`, `stroke="currentColor"`, `stroke-width="1.75"`, `fill="none"`, 20 px default, `aria-hidden="true"` (the button carries the label). The set: `Play`, `Pause`, `Undo`, `Redo`, `Download`, `X`, `Check`, `AlertTriangle`, `Type` (Captions), `Palette` (Style), `Sparkles` (Visuals), `Crop` (Framing), `Eye`, `EyeOff`, `MoreHorizontal`, `Keyboard`, `ArrowUp`, `ArrowDown`, `Hash` (Big number), `List`, `ArrowRight` (Before → after), `Zap` (Emphasis). Paths copied from Lucide at a pinned version (record it in `web/LICENSES.md`).

### Step 10.5 - The layout (D-178, §17.3)

| Order | File | Watch for |
|---|---|---|
| 1 | `ui/layout/EditorLayout.tsx` + `editor.module.css` | CSS grid: `topbar` row of `--topbar-h`; `stage` and `panel` columns (`1fr` and `--panel-w`, `--panel-w-sm` below 1280 px); the whole layout is the viewport height, the panel scrolls on its own; `data-theme="dark"` on the root; the one `aria-live="polite"` region |
| 2 | `ui/layout/TopBar.tsx` | Mark at left; clip length; save state (static "Saved on this device" for now); `IconButton` undo and redo (disabled); `Button` primary "Export" (wired to the existing `ExportButton` logic) |
| 3 | `ui/layout/Stage.tsx` | Centres a 9:16 frame (`aspect-ratio: 9 / 16`, `max-height: min(960px, 100% - space)`, `--r-panel`, `--shadow-2`) on `--bg`; puts in it the card of the state (§17.2) or `PreviewPlayer` |
| 4 | `ui/layout/SidePanel.tsx` + `panel.module.css` | `Tabs`: Captions, Style, Visuals, Framing; disabled with a `Skeleton` body while processing; the selected tab in `ui-store` |
| 5 | `state/ui-store.ts` | Tab, follow switch, editing word, toasts, save state (§16.1) |
| 6 | `ui/pages/EditorPage.tsx` | Renders `EditorLayout` for every clip state; preloads nothing else |

### Step 10.6 - The state cards (§17.2, D-188)

| Order | File | Watch for |
|---|---|---|
| 1 | `ui/components/DropZone.tsx` (restyled) | The empty stage: a dashed `--border` frame, an `ArrowUp` icon, "Drop a video (up to 90 seconds)", a secondary "Try a sample" button, the browser line at `--text-2`; drag-over highlights with `--brand` |
| 2 | `ui/components/ProcessingCard.tsx` + `ProcessingFeed.tsx` | The four steps with `Check` for done, a `Spinner` for the current, muted for the rest; the found-moment chips (pills at `--surface-2`) from the feed; the reassurance line; Cancel (`ghost`, wired in Phase 14) |
| 3 | `ui/components/ModelDownloadPanel.tsx` (restyled) | Size, progress bar on `--brand`, "only the first time", Cancel |
| 4 | `ui/components/RejectionPanel.tsx` (restyled) | Title, body, advice, "Choose another video"; V3's copy unchanged; `data-reject` unchanged (V3 freeze) |
| 5 | `ui/components/ErrorPanel.tsx` | The card of D-190 with its three slots (title, one sentence, action) and "Start over" |
| 6 | `ui/components/PreviewPlayer.tsx` | The canvas fills the 9:16 frame; the transport under the frame (play/pause `IconButton`, `--fs-13` tabular time, a placeholder scrub bar wired in Phase 12); the watermark badge (D-172) as a small pill under the transport when the user is not on Creator |

### Step 10.7 - Copy (section 17.5)

Add every key of §17.5 to `messages.ts` now (tone of D-177), including the ones the next phases use, so Phase 11 reviews real words. Run `node scripts/check-copy-codes.mjs`.

### Step 10.8 - The screen capture tool (D-191)

`web/tests-e2e/screens.spec.ts` (D1 only), run by a Playwright project of its own, `screens` (`testMatch: "screens.spec.ts"`, `dependencies: ["media-setup"]`, `use: MEDIA_USE`, one worker) in `web/playwright.config.ts`, and a root script `"screens": "pnpm --filter web exec playwright test --project=screens"`. CI names its projects (`non-media`, `media`), so it never runs this one: for each of the eight states and both viewports (1280 x 800, 1920 x 1080), reach the state, wait for fonts (`document.fonts.ready`) and for animations to settle, then `page.screenshot({ path: "test-results/screens/<state>-<w>x<h>.png", fullPage: false })`.

| State | How the spec reaches it |
|---|---|
| empty | Open `/app` |
| model download | A fresh browser context with the model absent; screenshot at about 30% |
| processing | Drop the portrait fixture; screenshot after the second step is done |
| ready | Wait for `ready` |
| editing a word | Ready, then open the Captions tab (static in this phase: the first word shown as an open field through the dev hook `__offcutDev.openWordField(0)`) |
| style tab | Ready, select Style (static cards in this phase) |
| error | The dev hook `__offcutDev.failClip("E_GPU_LOST")` |
| exporting | The dev hook `__offcutDev.fakeExport(0.42)` |

`__offcutDev` exists only when `import.meta.env.MODE !== "production"` (D-155's rule) and is removed from production by dead-code elimination; Step 16.6 proves it.

```bash
pnpm screens   # expect: 16 PNGs in web/test-results/screens/
```

### Step 10.9 - Self-review before the founder sees it

Open the 16 screenshots side by side and fix, before Phase 11, anything on this list:

| Look for | Rule |
|---|---|
| Spacing not on the 4 px grid | `--sp-*` only; measure with DevTools |
| Two greys for the same role | One token per role |
| Text under AA contrast | `--text-2` on `--surface-2` is the lowest pair allowed; check with DevTools' contrast picker |
| A control without hover, focus or disabled state | Every primitive has all four |
| The preview not the brightest thing on screen | Nothing in the interface brighter than `--text`; the brand colour only on the one primary action and the selected tab |
| Jargon | D-177's list |
| Layout shift between processing and ready | The 9:16 frame stays where it was |

### Milestone 10

```bash
pnpm check                                                   # expect: exit 0 (jsx-no-literals, copy codes, boundaries)
pnpm build && ls web/dist/assets | grep -c inter-latin       # expect: 1
node scripts/check-hosts.mjs 2>&1 | tail -3                  # record the shell size against Step 0.3's
pnpm e2e 2>&1 | tail -2                                      # expect: 12 passed (the landing page unchanged)
pnpm e2e:media 2>&1 | tail -3                                # expect: green (selectors the V2/V3 suites use are kept: data-testid values unchanged)
ls web/test-results/screens | wc -l                          # expect: 16
# the frozen check                                           # expect: no output
git push && gh pr checks 7 --watch                           # expect: green
```

Pass: every state of §17.2 renders in the shell from tokens and primitives, the font is a hashed asset, the old suites still find their elements, and 16 screenshots exist for the review.

Commit: `"V4 S10: design system, interface font, primitives, icons, editor shell and state cards"`

Do not start Phase 11 until this passes.

---

## Phase 11 - Design review 1 (S10A, D-191)

Goal: the founder has looked at the editor on real screens before any editing flow depends on it, and every change asked for is made.

### Step 11.1 - Prepare the review

```bash
pnpm screens   # fresh captures from the Milestone 10 build
```

Put the 16 images in one folder outside the repo (`~/v4-review-1/`), named `01-empty-1280.png` … in the order of §17.2. Write a one-page note beside them: what each screen is for, what the user does next, what is not wired yet.

### Step 11.2 - The review (human gate)

Show the founder the images (or the running `pnpm dev` at both window sizes). For each screen, ask the same four questions and write down the answers:

1. Does it look premium and calm, with the video as the hero?
2. Is it obvious what to do next, without reading?
3. Is every word plain enough for a creator who is not technical?
4. Anything that looks off (spacing, alignment, colour, size)?

Also ask for one decision: the brand colour (`--brand`, D-179's violet) — keep, or another. And whether Mono should keep its name.

### Step 11.3 - Make the changes and capture again

Every requested change goes through the tokens or the primitives where it can (a change in one component that should be global is a token change). Capture again, show the changed screens, and get a "go".

### Milestone 11

| Check | Where | Expect |
|---|---|---|
| The founder's go | `v4changelog.md` | An entry "Design review 1: go on <date>", the list of changes asked and made, the brand colour decision |
| The changed screens | `~/v4-review-1/after/` | One capture per changed screen |
| Machine checks | Milestone 10's commands | Unchanged results |

Pass: the founder said "go" on the captured screens and every change asked for is in the code.

Commit: `"V4 S10A: design review 1 changes"` (or no commit if nothing changed; the changelog entry is committed either way).

Do not start Phase 12 until this passes.

---

## Phase 12 - Preview and direct manipulation (S11)

Goal: the preview scrubs, prefetches, downscales on a slow machine, reports its statistics, and is directly editable: hover shows what a press will do, a click on a caption word opens it, the captions drag up and down with snaps, the video drags sideways, and a change while paused redraws at once.

### Step 12.1 - Seek, prefetch, the resize rule, statistics (D-148, D-149, D-155)

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `render/frame-decoder.ts`, `video-source.ts` | §15.3 | `prefetch(from)`; the restart at the keyframe at or before a far seek |
| 2 | `render/preview-job.ts`, `preview-loop.ts` | §15.2, D-149 | `previewSeek` (stop the loop, `frameAtBlocking`, draw, answer); the 60-frame mean cost and the one-time `session.resize(360, 640)`; `downscaled` in the statistics |
| 3 | `crates/offcut-wasm-render/src/session.rs` | §11.1 | `resize(width, height)` |
| 4 | `usecases/control-preview.ts` | §16.2 | `seek(at)` with one in flight and the latest winning; the sound restarts at `at` when it was playing; `lastStats` |
| 5 | `state/preview-store.ts` | §16.1 | `position`, `lastStats`, `seekStart`, `seekDone` (the three seek events of V2's table fire now) |
| 6 | `PreviewPlayer.tsx` | D-148, D-155 | The scrub bar (`Slider`, step one frame, live position while playing at 4 Hz), the 300 ms spinner, the `data-*` statistics |

### Step 12.2 - The redraw after a change while paused (D-173)

In `render.worker.ts`'s `setScene`: when a preview canvas is attached and the loop is not running, draw the frame at the last position (`preview-job.ts` keeps it) after the scene is set. Without this, a correction is invisible until play.

### Step 12.3 - Hit testing and gestures (D-174, D-183)

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `workers/protocol.ts`, `render.worker.ts` | §14.1, §15.1 | `hitTest({ at, x, y }) → Hit`, only when the loop is not running (else answers `{ kind: "video" }` without work) |
| 2 | `usecases/preview-interact.ts` | §16.2 | `hoverAt` (one per animation frame), `clickAt`, `dragCaptions`, `dragVideo`, the snap rule (16 px of Top 0.15, Middle 0.55, Bottom 1.0) |
| 3 | `usecases/set-caption-position.ts`, `set-crop-offset.ts` | §16.2 | Commit on release through `review-update.ts` mode `scene` (the update rule is Phase 13's: write the minimal version now, completed in Step 13.1) |
| 4 | `PreviewPlayer.tsx` + `preview.module.css` | §17.4, D-183 | Pointer events on a transparent layer over the canvas: cursor by hit (`text`, `ns-resize`, `ew-resize`, `default`); the 2 px word outline and the snap guide line and the 9:16 frame outline during a crop drag, all as DOM chrome above the canvas, never drawn into a frame; keyboard: arrow keys move the caption block in 0.05 steps when it has focus (an accessible handle) |
| 5 | The Free watermark badge | D-172 | Shown when no Creator token is cached |

### Step 12.4 - Look at it

`pnpm build:wasm && pnpm dev`, the portrait clip and the landscape clip.

| Check | Expect |
|---|---|
| Scrub a 60 s clip while paused | The frame follows the bar; at most a brief spinner on a far jump |
| Hover over a caption word (paused) | Text cursor and a thin outline on that word |
| Drag the captions to the top | They snap at Top; the visuals now appear below the captions |
| Drag the landscape clip's video | The 9:16 outline shows during the drag; on release the framing moves |
| Click a caption word | The Captions tab opens (its editor arrives in Phase 13; for now the tab opens and `ui-store`'s editing word is set) |
| `data-max-drift-ms` after a 20 s play | Present, small |

### Milestone 12

```bash
pnpm build:wasm && pnpm check && pnpm test                   # expect: exit 0
pnpm e2e:media 2>&1 | tail -3                                # expect: green
# the frozen check                                           # expect: no output
git push && gh pr checks 7 --watch                           # expect: green
```

Pass: scrubbing, prefetch, the resize rule, the statistics, the redraw after a paused change, hover, click, both drags and the snaps work on D1.

Commit: `"V4 S11: seek, prefetch, resize rule, statistics, hit testing and direct manipulation"`

Do not start Phase 13 until this passes.

---

## Phase 13 - Editing (S12)

Goal: the side panel's four tabs work. A word is fixed by clicking it, shows on the video within 150 ms, and can be fixed everywhere at once; every change can be undone; the style is chosen from cards showing the clip itself; visuals and framing are one switch or one segment away.

### Step 13.1 - The update rule and history (D-147, D-184)

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `state/history.ts` | §16.1, D-184 | 50 snapshots; a new change after undo drops the redo branch; `clear()` on a new clip |
| 2 | `usecases/review-update.ts` | Appendix C.2 | The full rule: pause and restore, optimistic `editStarted`, mode `plan` (redetect, rules plan) or `scene`, `setScene`, `editsRepo.put` (save state "Saving…" then "Saved on this device"), `editDone`, `history.push`, `review_action`; on failure `editFailed` and an `ErrorPanel` with "Undo this change" |
| 3 | `web/eslint.config.js` | §4 | The use-case edges: `edit-word.ts`, `toggle-event.ts`, `set-style.ts`, `set-crop-offset.ts`, `set-caption-position.ts`, `set-highlight-numbers.ts`, `replace-all.ts`, `undo-redo.ts` may import `review-update.ts`; `review-update.ts` may import `control-preview.ts`; each with its reason, as D-58's edges. **REOPENED V2 CONTRACT** |
| 4 | `usecases/undo-redo.ts` | D-184 | Mode `plan` when `word_edits` differ, else `scene` |

### Step 13.2 - Words: the transcript editor and replace-all (D-146, D-175, D-182)

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `render.worker.ts` | §15.1 | `redetectSentence` handler (uses the kept transcript, prosody, events); `detect` with edits for replace-all |
| 2 | `usecases/edit-word.ts` | D-146 | Mode `plan` for the word's sentence |
| 3 | `usecases/replace-all.ts` | D-175 | Original text equal, case-folded, whole word, not yet edited; one change; `detect` over the whole transcript |
| 4 | `ui/components/CaptionEditor.tsx` | D-182, §17.3 | Blocks by chunk with clickable start times; the active word highlighted and followed while playing (the Follow switch turns off on manual scroll); click → seek, pause, inline field with text selected; Enter or Tab commits (Tab opens the next word), Escape cancels, empty hides; dotted underline for edited, strike-through and "Show" for hidden; the ⋯ menu (Hide, Restore original); the replace-all toast after a commit when it applies |
| 5 | Edit-to-preview timing | D-173 | In `pnpm dev`, the Performance panel: commit → `setScene` answer → the redraw. Target ≤ 150 ms; if over, look at cache misses first |

### Step 13.3 - Style, Visuals, Framing (D-186, D-187)

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `workers/protocol.ts`, `render.worker.ts` | §15.1 | `styleThumbnails({ at, width, height })`: one `OffscreenCanvas`, three renders with the current edits, `transferToImageBitmap` each, the preview's scene and canvas restored |
| 2 | `usecases/style-thumbnails.ts`, `set-style.ts`, `set-highlight-numbers.ts` | §16.2 | Debounce 300 ms, only while the Style tab is shown, the latest call wins |
| 3 | `ui/components/StylePanel.tsx`, `StylePicker.tsx` | D-186 | Three cards (image in a 9:16 frame, name, one line, ring when selected, `aria-pressed`); the Highlight numbers `Switch` |
| 4 | `usecases/toggle-event.ts`, `ui/components/PlanItemList.tsx` | D-187 | Rows with the kind's icon, label, figure, time (click seeks), `Switch`; "N of M visuals on"; the empty state |
| 5 | `ui/components/FramingPanel.tsx`, `CropOffsetControl.tsx` | D-187 | `SegmentedControl` Top · Middle · Bottom (the current value shown even when dragged between snaps); the crop `Slider` for landscape and square clips only; the two hints |
| 6 | `ui/components/CutsSection.tsx` | §17.3 | Renders nothing while the plan has no cuts |

### Step 13.4 - Shortcuts and toasts (D-185)

`ui/hooks/use-shortcuts.ts` (one map: Space, ←/→, Shift+←/→, Ctrl/⌘+Z, Shift+Ctrl/⌘+Z, Ctrl+Y, Escape, `?`; ignored while focus is in a text field except Escape), `ShortcutsSheet.tsx`, `ToastHost.tsx` wired to `ui-store`.

### Step 13.5 - Look at it as a creator would

`pnpm build:wasm && pnpm dev`, the sample clip. Do this end to end without looking at the code:

| Task | Expect |
|---|---|
| Fix "90" to "9" by clicking it in the transcript | The from-to reads "9 minutes" on the paused video almost at once |
| Fix it by clicking the word on the video instead | Same field opens |
| A repeated misheard word (edit the clip's transcript in a test clip, or use the labeled `w`-file text in `pnpm dev`'s debug load if available) | "Change all N" toast; one click fixes all |
| Ctrl+Z three times, Shift+Ctrl+Z once | Each step goes back and forward exactly |
| Open Style | Three cards of your own frame within half a second; Bold applies on click |
| Turn Highlight numbers off | The number in the caption turns white |
| Visuals: switch the list off | It disappears from the video; "3 of 4 visuals on" |
| Framing: Top | Captions at the top, visuals under them |
| Space, ←, → | Play/pause, frame steps |

### Milestone 13

```bash
pnpm build:wasm && pnpm check && pnpm test                   # expect: exit 0
pnpm e2e:media 2>&1 | tail -3                                # expect: green
# the frozen check                                           # expect: no output
git push && gh pr checks 7 --watch                           # expect: green
```

Pass: every task of Step 13.5 works on D1, the edit-to-preview time is under 150 ms in the Performance panel, and no main-thread task over 50 ms appears during the tasks (D-193).

Commit: `"V4 S12: editing: transcript editor, change all, undo/redo, style cards, visuals, framing, shortcuts"`

Do not start Phase 14 until this passes.

---

## Phase 14 - Robustness and its interface (S13)

Goal: every long job can be cancelled, a crashed worker recovers, every failure a person can meet has plain words and one action, the export shows clear progress, and the analytics allowlist accepts the new review actions.

### Step 14.1 - RPC and the pool (D-151 to D-153)

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `workers/rpc-worker.ts` | D-151 | The throttle: 10 progress messages per second per job, feed lines always sent, the held one flushed before `res` |
| 2 | `workers/rpc.ts` | §14.2 | `cancel(jobId)`, `onCrash`, `isDead`; a worker `error`/`messageerror` rejects all pending calls with `E_WORKER_CRASH` |
| 3 | `workers/pool.ts` | §14.3 | `cancel(kind)` with `CANCEL_TIMEOUT_MS = 2000`, `restartWorker(kind)`; a dead client is replaced on the next call |

### Step 14.2 - Cancel everywhere (D-152)

| Order | File | Watch for |
|---|---|---|
| 1 | `usecases/cancel-job.ts` | `signalFor(scope)`, `cancelJob(scope)`; the four scopes; `job_cancelled { stage }` |
| 2 | `run-pipeline.ts`, `import-clip.ts`, `start-export.ts` | Read `signalFor`; end in the scope's state; `release()` and `closeSession()` for the pipeline; the export's temporary file deleted |
| 3 | `ProcessingCard`, `ModelDownloadPanel`, `ExportProgress` | Cancel buttons; the cancelled stage card ("Cancelled." with "Process again" and "Start over") |

### Step 14.3 - Errors and the export pill (D-153, D-159, D-189, D-190)

| Order | File | Watch for |
|---|---|---|
| 1 | `copy/messages.ts` | The 14 error entries of D-159 in the tone of D-190; `scripts/check-copy-codes.mjs`'s `COPY_PENDING` loses those 14 |
| 2 | `ErrorPanel.tsx` | One action by context: Try again (clip failed: `runPipeline`), Reload preview (`reloadPreview`), Undo this change; "Start over"; `client_error` once per shown failure; a collapsed "Details" only in development builds |
| 3 | `usecases/control-preview.ts` | `reloadPreview()`: open the session, set the scene, attach a new canvas |
| 4 | `ExportProgress.tsx`, `TopBar.tsx` | The pill: stage in words, percent, a bar, time left after 3 s, Cancel; done state with "Download again" |

### Step 14.4 - The analytics enum (§13)

| Order | File | Watch for |
|---|---|---|
| 1 | `crates/offcut-api-types/src/analytics.rs` | `ReviewActionKind` + `caption_position`, `highlight_numbers`, `replace_all`, `undo`, `redo`; one `analytics_allowlist` case each in its tests |
| 2 | `pnpm gen:types && sh scripts/check-gen-clean.sh` | `gen/api.ts` and `ANALYTICS_EVENT_DOCS` regenerated |
| 3 | The Step 1.5 row for `analytics.rs` | Closed |

```bash
set -a; . ./.env; set +a; cargo test -p offcut-api-types -p offcut-api   # expect: all pass (server tests need DATABASE_URL, V3 memory)
```

### Step 14.5 - Try to break it

| Do | Expect |
|---|---|
| Cancel during transcription | Stage card "Cancelled."; "Process again" runs it from the start |
| Cancel the first-run model download at 30% | Partial bytes kept; the next run resumes |
| Cancel an export at 50% | Back to the editor; no file under OPFS `exports/tmp/` (DevTools → Application → Storage) |
| DevTools → kill the render worker (Sources → Threads → terminate, or `__offcutDev.crash("render")`) while paused | `ErrorPanel` "The preview stopped", "Reload preview" brings it back with the edits |
| Same during processing | "Try again" runs the pipeline again |
| A failed edit (`__offcutDev.failNextSetScene()`) | "Undo this change" restores the previous state |

### Milestone 14

```bash
pnpm check                                                   # expect: exit 0 (copy codes: the 14 have copy)
pnpm test                                                    # expect: exit 0
pnpm e2e:media 2>&1 | tail -3                                # expect: green
# the frozen check (crates/offcut-api-types/src/analytics.rs is allowed: §4)   # expect: no output otherwise
git push && gh pr checks 7 --watch                           # expect: green, including the server tests with the new enum members
```

Pass: every case of Step 14.5 behaves as written, the copy check is green with 14 fewer pending codes, and the server accepts the new review actions in CI.

Commit: `"V4 S13: cancel, crash recovery, error cards, export pill, analytics review actions"`

Do not start Phase 15 until this passes.

---

## Phase 15 - Unit and machine tests (S14, §20.3)

Goal: the RPC layer, the two machines V2 declared but never tested, the pipeline with fake workers and the undo history are covered by Vitest.

### Step 15.1 - The four test files

| Order | File | Section | Cases (all of §20.3) / Watch for |
|---|---|---|---|
| 1 | `web/src/workers/rpc.test.ts` | §14.2 | A scripted fake `Worker` (a class with `postMessage`, `terminate`, and handlers the test drives); request and response; failure; the throttle with fake timers (`vi.useFakeTimers()`): 30 progress messages in 1 s arrive as at most 10, every feed line arrives, the held progress arrives before `res`; cancel answered; crash rejects all pending; a call after a crash makes a new worker; dev assertion on a non-detached buffer |
| 2 | `web/src/state/machines/clip-machine.test.ts` | §20.3 | Every legal transition of TS §12.2's clip table, each from a fresh store; the four illegal ones of TS §12.3 throw `IllegalTransitionError`; `updating` refuses `edit_start` |
| 3 | `web/src/state/machines/model-machine.test.ts` | §20.3 | TS §12.2's model table; `verifying → downloading` without `failed` refused |
| 4 | `web/src/usecases/run-pipeline.test.ts` | §16.2 | `vi.mock("../workers/pool")` with scripted results; the order A, B1 ‖ B2, C, D, E (`rulesPlan`), F (`writeAudio` before `assemble`); `REJECT_NO_SPEECH`; a held B2 failure (D-90); a cancel at each of five stages ends `accepted` with `release` called; a crash ends `failed` with `client_error`; `retry` runs again from A; a `review-update` block (mode `plan` calls redetect and rules plan, mode `scene` does not; a failure restores the previous `EditState`) |
| 5 | `web/src/state/history.test.ts` | §20.3 | Push, undo, redo, 50-step limit, the redo branch dropped by a new change, `clear` |

### Milestone 15

```bash
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"   # expect: about 160 passed, 0 failed
pnpm check                                                   # expect: exit 0
git push && gh pr checks 7 --watch                           # expect: green
```

Pass: the five files pass and the Vitest count is above 92 by them.

Commit: `"V4 S14: rpc, machine, pipeline and history tests"`

Do not start Phase 16 until this passes.

---

## Phase 16 - End-to-end suites (S15, §20.4)

Goal: the preview's timing, the editor's behaviour, cancel and accessibility are proven in Chrome on D1; the zero-event and long-task cases complete `pipeline-preview.spec.ts`; no dev hook reaches production.

### Step 16.1 - Helpers

`web/tests-e2e/helpers/fixtures.ts` gains `previewStats(page)` (reads the `data-*` attributes), `seekTo(page, ms)`, `editWord(page, index, text)`, `toggleItem(page, label)`, `setStyle(page, name)`, `dragCaptions(page, to)`, `dragVideo(page, offset)`, `axe(page)` (wraps `@axe-core/playwright`, returns serious and critical violations).

### Step 16.2 - `preview-sync.spec.ts` (four cases)

Drift over 60 s (`data-max-drift-ms` ≤ 80, `late / frames` ≤ 0.05); first frame within 500 ms of `ready`; seek to 55.0 s (the white frame of the portrait clip) shows a bright canvas within 300 ms; edit to preview ≤ 150 ms (90th percentile of 20 commits on a paused clip).

### Step 16.3 - `review-edit.spec.ts` (sixteen cases)

The cases of §20.4 in this order, each self-contained (its own clip load through the cached model): edit a number; hide a word; toggle; toggle survives a word edit; style; crop on landscape; no crop control on portrait; pause and restore; undo after a failed edit; click a word in the transcript; Tab and Escape; click a caption word on the preview; replace all; undo and redo across three kinds of change; drag captions to Top; style cards and Highlight numbers; shortcuts; axe and keyboard reachability.

### Step 16.4 - `cancel.spec.ts` (five cases) and `pipeline-preview.spec.ts`

`cancel.spec.ts` as §20.4, the import and model cases tagged `@no-gpu`. In `pipeline-preview.spec.ts`: the zero-event case (`speech_20s_noevents.mp4` → the Visuals tab shows `noEventsFound`), the long-task case (D-156: ≤ 100 ms while processing; D-193: ≤ 50 ms during a word edit, a style switch and a caption drag), the statistics case; and the D-160 rewrites.

### Step 16.5 - `playwright.config.ts`

`MEDIA_SUITES` gains the three new suites. Then:

```bash
pnpm --filter web exec playwright test --project=media --list | tail -1          # expect: 76 tests in the media project (plus 2 setup)
pnpm --filter web exec playwright test --project=media --list --grep @no-gpu | tail -1   # expect: V3's 23 plus the 2 new
```

### Step 16.6 - No dev hook in production

```bash
pnpm build
grep -rlE "__offcutDev|data-caption-text|data-caption-colors|data-caption-position" web/dist && echo "LEAK" || echo clean   # expect: clean
node scripts/check-hosts.mjs                                  # expect: exit 0 (and it now fails on the marker strings: add them to its list)
```

### Step 16.7 - Run everything on D1

```bash
pnpm e2e 2>&1 | tail -2                                      # expect: 12 passed
pnpm e2e:media 2>&1 | tail -3                                # expect: 78 passed (76 + 2 setup), 1 skipped (D-78), 0 failed
```

A flaky case is fixed, not retried: find the race (usually a missing wait for `ready`, for `document.fonts.ready`, or for the worker's answer) and record it in the changelog.

### Milestone 16

```bash
pnpm e2e:media 2>&1 | tail -3                                # expect: 78 passed, 1 skipped (D1, keyed)
# Step 16.6                                                  # expect: clean
# the frozen check                                           # expect: no output
git push && gh pr checks 7 --watch                           # expect: green (Windows runs the @no-gpu cases)
```

Pass: 76 media cases pass on D1 twice in a row, axe finds no serious or critical violation, and the production bundle carries no dev hook.

Commit: `"V4 S15: preview-sync, review-edit, cancel suites; pipeline-preview complete"`

Do not start Phase 17 until this passes.

---

## Phase 17 - CI (S16, §19.3-§19.4)

Goal: CI runs everything V4 added that a runner can run, the golden test on lavapipe included, and fails on any unreviewed snapshot.

### Step 17.1 - The workflows

| Order | File | Watch for |
|---|---|---|
| 1 | `.github/workflows/ci.yml` | The `mesa-vulkan-drivers` line and `INSTA_UPDATE: "no"` (added in Step 7.3); `CI: "true"` in the `cargo test` step's env; the version of the Mesa package printed (`dpkg -s mesa-vulkan-drivers \| grep Version`) |
| 2 | `.github/workflows/e2e-media.yml` | Nothing but what the new `@no-gpu` cases need (they use the existing fixtures) |
| 3 | V3 CL item 7 | If the founder chose to move the Linux ffmpeg URL now (Step 0.8), change `FFMPEG_LINUX_URL` and `FFMPEG_LINUX_SHA256` here and close the item |

### Step 17.2 - Push and read the logs

```bash
git push && gh pr checks 7 --watch
gh run view --log --job <ci job id> | grep -E "golden_frames|ssim|llvmpipe|mesa|test result" | head -40
```

| Log line | Expect |
|---|---|
| `mesa-vulkan-drivers` version | Printed; recorded in the changelog |
| `golden_frames` | `17 passed`, the adapter `llvmpipe` |
| `catalog_snapshot`, `display_list_snapshots` | Passed; no "pending snapshot" |
| `analytics_allowlist` (server tests) | Passed with the five new members |
| `e2e-media` | The `@no-gpu` cases, all passed |

### Step 17.3 - Prove the CI guards

| Temporary edit, on a scratch commit | Rule that must fire in CI |
|---|---|
| Change one snapshot file by hand | `display_list_snapshots` fails ("snapshot assertion") |
| Darken one golden PNG | `golden_frames` fails with its SSIM |
| Remove `mesa-vulkan-drivers` | `golden_frames` fails naming a non-software adapter or none |

Revert the scratch commit (`git revert`), push, and check green again.

### Milestone 17

```bash
gh pr checks 7                                               # expect: every check pass
```

Pass: both jobs green on the latest commit, the golden test ran on lavapipe, and each CI guard was seen to fire once.

Commit: `"V4 S16: CI runs goldens on lavapipe and refuses unreviewed snapshots"`

Do not start Phase 18 until this passes.

---

## Phase 18 - Measure (S17, §21)

Goal: every reading of §21.1 is in `docs/v4/experiments.md` with its decision, the bench file is committed, and BP §6.2's two decisions are written.

### Step 18.1 - Readings

Fill each heading of `experiments.md` (Question, Method, Numbers, Date, Decision), all labelled `d1`:

| Reading | How | Target |
|---|---|---|
| Edit to preview | `preview-sync.spec.ts`'s case, five runs of 20 commits; the 90th percentile of each and the worst | ≤ 150 ms |
| Style thumbnails | `review-edit.spec.ts`'s timing, 10 openings | ≤ 400 ms |
| Hit test | A console loop of 100 `hitTest` calls while paused (dev build) | ≤ 16 ms each |
| App shell with the font | `check-hosts.mjs`'s bundle size | ≤ 400 kB gzip |
| Preview drift and late frames | `preview-sync.spec.ts`, five runs, the worst | ≤ 80 ms; ≤ 5% |
| First frame, seek | Same suite | ≤ 500 ms; ≤ 300 ms |
| Downscale | `data-downscaled` after a 60 s play | `false` on D1 |
| `build_scene`, `frame_at` | `cargo test -p offcut-scene --release -- --ignored scene_timing --nocapture` | ≤ 1 s; mean ≤ 2 ms |
| Golden similarity | The CI log of Step 17.2 and D1's own run | ≥ 0.98 each |

### Step 18.2 - The bench (D-162)

```bash
BENCH_DEVICE=d1 BENCH_CLIP=portrait pnpm bench:device        # 10 runs; writes bench/results/d1-<date>-portrait.json
node -e 'const a=require("./bench/results/d1-2026-10-19-portrait.json"), b=require("./bench/results/<new file>");
for (const s of ["detect_scene","render_encode"]) console.log(s, a.stages[s]?.median_ms, "->", b.stages[s]?.median_ms)'
# expect: detect_scene within 10% of V3's plus about 30 ms; render_encode at most 90 s per 60 s (E-4)
```

### Step 18.3 - BP §6.2's decisions (§21.2)

Write both rows into `experiments.md`: "drift within budget: no preview-file contingency" (or the contingency taken), "Vello in a worker reliable: no Canvas2D backend" (or the contingency taken), each with the readings that decided it.

### Step 18.4 - Replace the measured assumptions

Every (assumption) of Appendix D that a reading measured is replaced in TS §39.3 (S18 copies it there) and noted in the changelog.

### Milestone 18

```bash
git ls-files bench/results | grep "$(date +%Y-%m)"           # expect: the new bench file
grep -c "^Decision:" docs/v4/experiments.md                  # expect: 14 (twelve readings, two decisions), none empty
git push && gh pr checks 7 --watch                           # expect: green
```

Pass: every reading is written with a decision, every target is met or its pre-agreed action is recorded, and the bench file is committed.

Commit: `"V4 S17: measurements, the bench file and BP §6.2's decisions"`

Do not start Phase 19 until this passes.

---

## Phase 19 - Copy-back, design review 2, merge, deploy, tag (S18, S19)

Goal: the specs say what V4 built, the founder has passed the exit design review, V4 is merged into `main` and deployed, the deployed editor works on D1, and `v4` is tagged.

### Step 19.1 - Copy the decisions back (S18, §24.2)

| File | What to copy | Source |
|---|---|---|
| `docs/technicalspec.md` | §24.2's list into the sections it names: TS §40.2, §40.4 (resolution in `validate`, `marks`, the plan limits), §40.5 (`dst` rectangle, `mono_font`, the renderer's V4 support rule), §40.9 (the wire format fixed at V4), §10.1 (`to_out_ms`), §10.6 (`EditState`'s two fields), §14.2 (`assemble`, `release`, `hitTest`, `styleThumbnails`, `PreviewStats`), §19.2-§19.4 (V4's signatures and the style table of D-170), §13.3 (the component table of §17.3), §36 (self-hosted fonts are allowed), §39.3 (measured values) | §24, §24.2 |
| `docs/buildplan.md` | V4's 13 days and 19 steps; BP §6.3's exit list as §23 holds it; §24 item 10 (the empty-output retry moves to V7) into BP §9 | §24 |
| `docs/product.md` | §24 item 16: the audience (PS §3, §4, §6); PS §12.2's style names (Clean, Bold, Mono) | D-177, D-170 |
| `docs/v4/v4implementation.md` | The "As built" notes of every decision that changed while building, as V3 did | The changelog |

```bash
node scripts/check-file-tree.mjs                             # expect: exit 0
```

### Step 19.2 - The final local gate

```bash
pnpm install --frozen-lockfile && pnpm check                 # expect: exit 0
pnpm fixtures && pnpm test                                   # expect: exit 0
cargo test --workspace 2>&1 | awk '/^test result:/ {p+=$4; f+=$6; i+=$8} END {print p" passed, "f" failed, "i" ignored"}'
# expect: 0 failed; passed about 560; ignored 3 + scene_timing
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"      # expect: about 160 passed
pnpm build                                                   # expect: exit 0; Step 16.6 clean
pnpm e2e 2>&1 | tail -2                                      # expect: 12 passed
pnpm e2e:media 2>&1 | tail -3                                # expect: 78 passed, 1 skipped (D1, keyed; D-72)
git grep -nE "\bfetch\(" -- web/src | grep -vE "net/http\.ts|net/asset-fetch\.ts|wasm/load-core\.ts|wasm/load-render\.ts"   # expect: no output (D-164)
git grep -n "TODO" -- crates web/src | grep -vE "TODO\(V[0-9]+\)|V[0-9]+:"   # expect: no output
git grep -nE "\bV4\b" -- web/src crates | grep -v "\.ttf"    # expect: no output (every V4 marker filled or moved to a later version)
for f in $(git diff --name-only 550ff8f -- crates web/src | grep -E '\.(rs|ts|tsx)$' | grep -v '/tests'); do
  [ -f "$f" ] || continue
  n=$(awk '/#\[cfg\(test\)\]/{print NR-1; f=1; exit} END{if(!f) print NR}' "$f"); [ "$n" -ge 400 ] && echo "TOO LONG $f $n"
done; echo lengths checked                                   # expect: lengths checked
# the frozen check                                           # expect: no output
```

Write the final counts into the changelog; none below Entry 1's.

### Step 19.3 - Prepare design review 2 (D-191)

```bash
pnpm build && pnpm screens                                   # the 16 captures on the finished build
```

Go through the visual QA list of D-191 yourself first, on both widths, and fix what fails:

| Check | How |
|---|---|
| 4 px grid, alignment | DevTools box model on each panel and card |
| Hover, focus, pressed, disabled on every control | Tab through the whole editor; hover each control |
| Empty, loading and error states present | The captures of `empty`, `processing`, `error`; the Visuals tab with no item; the Style cards while loading |
| Reduced motion | DevTools → Rendering → `prefers-reduced-motion: reduce`: the pill still moves (content), the interface does not slide |
| No clipped text | Each style at both widths in the captures; the longest copy strings (errors) in their cards |
| Contrast | Step 16.3's axe run, and the contrast picker on `--text-2` |

### Step 19.4 - Merge and deploy (S19)

```bash
git push && gh pr checks 7                                   # expect: every check pass
gh pr ready 7                                                # out of draft, when the founder agrees
```

The founder merges PR #7 (D-115). `ci.yml` on `main` deploys the server first (the analytics enum is new), waits for its version, then the web build.

```bash
export APP_URL=https://offcut-one.vercel.app
git fetch origin && MERGE=$(git rev-parse origin/main)
gh run watch                                                 # the main run: ci, e2e-media, deploy-api, deploy-web all success
curl -s "$APP_URL/api/v1/healthz"                            # expect: 200 and the commit $MERGE
node scripts/check-headers.mjs "$APP_URL"                    # expect: the 7 headers, value for value
E2E_BASE_URL="$APP_URL" pnpm --filter web exec playwright test --project=non-media --grep @smoke   # expect: 1 passed
```

### Step 19.5 - Design review 2 and the goldens (human gate, D-191, §23)

On the **deployed** editor, at 1280 x 800 and 1920 x 1080, with the founder:

1. The founder takes the sample clip from drop to export alone, without being told how, while you watch and note every hesitation.
2. The four questions of Step 11.2 on the eight states.
3. Five of the 16 goldens, chosen with `ls fixtures/golden/*.png | shuf -n 5`: "Does each look as intended?"

Record the answers, the hesitations and any requested change in the changelog. A requested change that is not cosmetic becomes a V5 item; a cosmetic one is fixed on a small branch from `main` before the tag, with its own pull request.

### Step 19.6 - Verify the deployed system on D1

| Check | Where | Expect |
|---|---|---|
| `crossOriginIsolated` | DevTools console on `$APP_URL/app` | `true` |
| The interface font | DevTools → Network, filter `woff2` | `/assets/inter-latin-<hash>.woff2`, 200, `font/woff2`, `Cache-Control: public, max-age=31536000, immutable` (Read this first, item 9) |
| The sample clip | Deployed app | Processing card steps; `ready`; Clean with the sliding pill; the four visuals of the clip |
| Edit, change all, undo | Deployed app | As Step 13.5 |
| Caption drag to Top, crop drag on a landscape clip | Deployed app | As Step 12.4 |
| Cancel during transcription | Deployed app | "Cancelled." with "Process again" |
| Rejections unchanged | Drop `fixtures/.gen/rej_4k.mp4` | V3's message, restyled card |

In the Neon SQL editor:

```sql
select name, props, ts from analytics_events
where name in ('review_action', 'job_cancelled', 'client_error') and ts > now() - interval '1 hour'
order by ts desc limit 30;
-- expect: review_action rows with props like {"action":"word_edit"}, {"action":"replace_all"}, {"action":"caption_position"},
--         {"action":"event_toggle","event_kind":"list_reveal","enabled":false}; job_cancelled with {"stage":"asr"};
--         no text, no number but enums and booleans
```

**Rollback.** If `healthz` does not report `$MERGE` within 15 minutes, or a check above fails on the deployed system, open a pull request with `git revert -m 1 $MERGE`, have it merged, confirm `healthz` reports the revert and the deployed app behaves as V3; then fix on a new branch from `v4-build` and repeat Steps 19.2-19.6.

### Step 19.7 - Tick the exit checklist and tag

| §23 group | Where it was proven |
|---|---|
| Code and checks | Steps 19.2, 16.6 |
| Measurements | Phase 18 |
| Deployment | Steps 19.4, 19.6 |
| Records (design reviews, line-break reading, goldens) | Steps 8.2, 11.2, 19.5 |
| §22.3 "V5's first morning" | Steps 19.2 (markers, TODO), 18.2 (bench), 6.3 (`program_identity`), 19.1 (copy-back) |

Write the closing changelog entry ("V4 closed: <date>, merge <SHA>, final counts, reviews passed"), committed through a small pull request if `main`'s protection requires one, then the founder tags:

```bash
git fetch origin && git switch main && git pull --ff-only
git tag -a v4 -m "V4: the look and the editor" "$(git rev-parse HEAD)"
git push origin v4
git ls-remote --tags origin v4                               # expect: one line with refs/tags/v4
```

### Milestone 19

```bash
curl -s "$APP_URL/api/v1/healthz"                            # expect: 200, the merge commit (or the changelog commit after it)
node scripts/check-headers.mjs "$APP_URL"                    # expect: exit 0
git tag --points-at v4                                       # expect: v4 on the deployed commit
grep -c "^- \[ \]" docs/v4/v4implementation.md               # expect: 0 (every box of §1A, §22.3 and §23 ticked)
```

| Check | Where | Expect |
|---|---|---|
| The editor on the deployed app | `$APP_URL/app` | Premium default, edits within a moment, drags, undo, cancel |
| Analytics rows | Neon | Enums only |
| Design review 2 | Changelog | Passed, with the founder's words |
| Tag | GitHub | `v4` on the deployed commit |

Pass: the deployed editor works end to end on D1, the founder passed the exit review and the goldens, every box of §23 is ticked, and `v4` is pushed.

Commit: `"V4 S18: spec copy-back and close-out"` (S19 is the review, the merge and the tag).

V5 must not start until this passes.

---

## Quick reference

### Start every terminal

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut && export BENCH_DEVICE=d1 && git branch --show-current && git status --short | head -5
```

Add `export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=...` only in terminals that build for the media suites; never write it into a tracked file. Before `cargo test --workspace`, `set -a; . ./.env; set +a` (the server tests need `DATABASE_URL`).

### After changing ... run ...

| After changing | Run |
|---|---|
| A shared type (`offcut-types`, `offcut-api-types`) | `pnpm gen:types && sh scripts/check-gen-clean.sh`; fix `clip-store.ts`'s defaults; `pnpm check` |
| Any Rust crate used in the browser | `pnpm build:wasm`, then `pnpm dev` or the media suites |
| `Cargo.toml` dependencies | `cargo deny check`; Step 1.3's two tree checks |
| A scene file | `cargo test -p offcut-scene`; review snapshot changes with `cargo insta review`; a changed snapshot after its first commit gets a changelog line |
| A visual value (theme, style, a visual's size or timing) | The scene tests, then `OFFCUT_GOLDEN_UPDATE=1` for the goldens it changes, a look at each PNG, a changelog line |
| `readability.rs` or the break search | `cargo test -p offcut-scene --test line_breaks`; never edit a row to match (Read this first, item 6) |
| The catalog or `prompt.rs` | `cargo test -p offcut-plan`; bump `CATALOG_VERSION` if the prompt or schema snapshot changes |
| `tokens.css` or a primitive | `pnpm screens` and look at the 16 captures |
| `web/src/copy/messages.ts` | `node scripts/check-copy-codes.mjs`; `pnpm check` |
| `web/playwright.config.ts` | `--list` counts of Step 16.5 |
| `.github/workflows/*.yml` | Push and read the lines of Step 17.2 |
| A new source file, or the TS §5 tree | `node scripts/check-file-tree.mjs` |
| Anything at all | The frozen check of Step 1.6 |

### When something fails

| When something fails | Likely cause |
|---|---|
| `v3_reference` differs in glyph positions | A `FontKey` maps to another face or weight than V3's `FontId` of the same index |
| `v3_reference` differs by one frame in a chunk's show | An `OutSpan` conversion rounds differently from V3's `TimeMs` arithmetic |
| `golden_frames` panics "not a software adapter" | `force_fallback_adapter` ignored, or Mesa missing on the runner; Step 0.5 |
| `golden_frames` SSIM just under 0.98 on lavapipe only | Rasterisation differences between WARP and lavapipe; per-adapter folders (§21.1), never a lower threshold |
| A wgpu native crate in the wasm build | `vulkan`/`dx12` moved into the normal dependency; Step 1.3 |
| CI fails "pending snapshot" | A `.snap.new` not reviewed and committed; `cargo insta review` |
| `tsc` fails on `EditState` after a type change | `clip-store.ts`'s default `edit` lacks a new field |
| The correction is not visible until play | The redraw after `setScene` while paused (Step 12.2) |
| Edit to preview over 150 ms | Shaping cache misses (a key without the size bits, or a cache rebuilt per call); the full scene rebuilt where a chunk would do |
| Style cards blank | `transferToImageBitmap` before the render finished, or the preview's scene not restored after (Step 13.3) |
| The pill jumps instead of sliding | Its rectangle computed from the active word only; it must interpolate from the previous word's box over 90 ms |
| A chunk ends with "the" | `BINDERS` missing the word, or a list not sorted (the binary search fails silently); the sorted-list test |
| Captions overlap the watermark at Bottom | `placement.rs`'s `CAPTION_BOTTOM` ignores the watermark band, or the band reads the profile (it must not) |
| A drag commits on every move | The commit wired to `input` instead of release (`change`/pointer-up) |
| Undo restores the wrong step | `history.push` called before the update succeeded, or on a failed update |
| The font comes back as HTML in production | It is under `public/`; it must be a hashed `/assets/` file (Read this first, item 9) |
| `screens.spec.ts` runs in CI | A CI command without `--project`; it runs only with `--project=screens` |
| `analytics_allowlist` fails in CI | The enum member added in `analytics.rs` without its test case, or `gen/api.ts` not regenerated |
| `healthz` reports the old commit after the merge | The deploy failed or Render is still building; Step 19.6's rollback rule |

---

## Assumptions I made and gaps found

### (a) Environment values assumed

| Value | Assumed | How to correct |
|---|---|---|
| OS and shell | Windows 11 (D1), Git Bash for every command | Step 0.1 checks it |
| Repo path | `/c/Users/Mayan/Desktop/sh2clips/offcut` | Replace once in the Quick reference line |
| GitHub CLI | Installed and logged in (since V3) | Step 0.2 |
| Software adapters | WARP through DX12 on D1; Mesa lavapipe through Vulkan on `ubuntu-latest` | Step 0.5 proves both, or takes D-134's fallback |
| `fonttools` | Installed with `pip` for the one subset of Step 0.7 | Any machine with Python can make the file once |
| App URL | `https://offcut-one.vercel.app` (V3's deployed URL) | Read it from Vercel if it changed |
| Days | 13 working days (§5), BP §0's four overridden | - |

### (b) Gaps and contradictions found while writing this guide, and how they are resolved

| # | Finding | Resolution | Where |
|---|---|---|---|
| 1 | §4 and D-180 put the font in `web/public/fonts/`; `vercel.json` rewrites every path outside `/api/`, `/assets/` and `/ort/` to `index.html` | Moved to `web/src/ui/assets/` so Vite emits a hashed `/assets/` file; `vercel.json` untouched. Applied to `v4implementation.md` in the same commit as this guide | Read this first item 9, Step 10.2, Step 19.6 |
| 2 | `screens.spec.ts` needs a Playwright project and a script; §4 named neither | A `screens` project and a root `screens` script; CI names its projects, so it never runs there. Applied to §4 and §19.2 | Step 10.8 |
| 3 | The identity reference test cannot compile on V3's API once Phase 5 changes it, but D-117 compares only at S6 | The test is rewritten into its comparison form in Phase 5 and `#[ignore]`d until Phase 6 | Step 5.1 row 9b, Step 6.1 |
| 4 | D-135 makes four goldens at S7, two of which (Clean NumberReveal with and without the watermark) S9's redesign would change | Phase 7 writes only the two video-only goldens; Phase 9 writes the other 14 | Read this first item 3, Phases 7, 9 |
| 5 | The golden comparison on lavapipe needs Phase 17's CI lines, but the cross-adapter reading belongs to Phase 7 | The `mesa-vulkan-drivers` line and `INSTA_UPDATE` are added to `ci.yml` in Step 7.3; Phase 17 completes and checks it | Step 7.3 |
| 6 | The screens of Phase 11 need the error, editing and exporting states before their flows exist | Dev-only hooks `__offcutDev.*`, removed from production and checked in Step 16.6 | Step 10.8, Step 16.6 |
| 7 | `review-update.ts` is Phase 13's, but the drags of Phase 12 commit through it | A minimal version in Phase 12, completed in Step 13.1 | Step 12.3 |
| 8 | §20.1's media count (76) assumed no V3 case is removed or split | 76 cases + 2 setup = 78 passed, with V3's one fixme skip | Steps 16.5, 16.7 |
| 9 | `check-hosts.mjs` checks host names, not dev markers | It gains the marker strings of Step 16.6 | Step 16.6 |
| 10 | The analytics enum change makes the server's tests and deploy part of V4 though §13 once said "no changes" | §13 already revised ("one enum widened"); the server tests run in Phase 14, the deploy order in Step 19.4 | Phase 14 |

### (c) S-steps and files placed differently from §5

| Item | §5 | This guide | Reason |
|---|---|---|---|
| S7's four goldens | S7 | Two in Phase 7, fourteen in Phase 9 | Item (b) 4 |
| `ci.yml`'s Mesa and `INSTA_UPDATE` lines | S16 | Step 7.3 | Item (b) 5 |
| `v3_reference.rs` comparison form | S6 | Written in Phase 5, passing in Phase 6 | Item (b) 3 |
| `review-update.ts` | S12 | Minimal in Phase 12, complete in Phase 13 | Item (b) 7 |
| `messages.ts`'s new keys | S12, S13 | Phase 10 | The design review reads real words |
| `set-caption-position.ts`, `set-crop-offset.ts` | S12 | Phase 12 | The drags need them |
| `styleThumbnails`, `hitTest` in `protocol.ts` | S11-S12 | Phases 12 and 13, each with its first user | A method enters the protocol with its handler |
