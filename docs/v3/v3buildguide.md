# v3buildguide.md - Offcut V3: step-by-step build guide

## What this is

`docs/v3/v3implementation.md` says what each V3 file contains. This guide says what to do next, which command to run, and how to know it worked. V3 starts from `origin/main` at `8a271a9` (V2 merged and deployed; the `v2` tag may still be missing, §1A) on a new branch `v3-build`. It ends with V3 merged into `main`, deployed by `ci.yml`, every box of §22 ticked, and the tag `v3` pushed.

## How to read the references

| Notation | Means |
|---|---|
| `§n` | Section `n` of `docs/v3/v3implementation.md` |
| `TS §n` | Section `n` of `docs/technicalspec.md` |
| `BP §n` | Section `n` of `docs/buildplan.md` |
| `PS §n` | Section `n` of `docs/product.md` |
| `V2 §n`, `V2 CL item n` | `docs/v2/v2implementation.md`; a row of the "Open items" or "Known issues" table of `docs/v2/v2changelog.md` |
| `D-n` | A decision: D-1 to D-72 are V1's and V2's, D-73 to D-112 are V3's (§2) |
| `S-n` | A build-order step of §5 (S1 to S15) |
| `TE-n`, `E-n` | Technical and product experiments (TS §37) |
| `INV-n`, `P-n` | Invariants (TS §35), privacy promises (TS §25.1) |
| `P1`-`P8` | Postconditions of `run_chain` (TS §34.3) |
| `X1`-`X7`, `E0`-`E4`, `H1`-`H8` | Exclusions (§9.7), edit-list shapes (§6.1), probe hardening (§6.3) |
| D1 | The founder's Windows machine; every V3 measurement is read there and labelled `d1` (D-74) |
| "keyed build" | A web build carrying `VITE_ENTITLEMENT_TEST_PUBLIC_KEY`, exactly as V2's media runs used it (D-72) |
| "the frozen check" | The command of Step 1.7 |

## How the guide is organised

Fourteen phases, numbered 0 to 13. Each phase is a list of numbered steps and ends with a milestone: commands with their expected output, a `Pass:` line and a commit message. **Do not start a phase while the previous milestone fails.** Every milestone except Milestone 0 ends in a commit on `v3-build`; Phase 13 ends with the merge into `main` and the `v3` tag. The phase order differs from §5 in two places, for the reasons in item 8 below.

## Phase overview

| Phase | Covers (S-steps and sections) | Plan day | Milestone in one line |
|---|---|---|---|
| 0 Environment and V2 baseline | §1A, §3 | Day 1 (morning) | Every tool prints its version, every encoder exists, the V2 counts are 262/2, 92, 12, 30 |
| 1 Contracts, dependencies, docs | S1; §2, §4, §18.1, §18.3, D-83, D-103 | Day 1 | `pnpm check` green with the new crates and the reopened contracts; test counts unchanged |
| 2 Fixtures and the frozen labeled set | S2; §17.1-§17.5, §17.7 | Day 1-2 | 25 fixtures generated twice byte-identical; 31 labeled files committed before any detector rule |
| 3 Voice chain and prosody, native | S4; §7, §19.4-§19.7 | Day 2 | `cargo test -p offcut-dsp` green; `DENOISE_DELAY_SAMPLES` measured; native TE-13 under 2 s |
| 4 Audio worker in the browser (TE-13) | S6; §10.1, §13.1-§13.4, §14 | Day 3 | The chain runs in `audio.worker` on D1 in at most 4 s per 60 s, length unchanged |
| 5 Demuxer hardening | S3; §6, §19.2, §19.3 | Day 3 | Every `ok_*` fixture accepted, every `rej_*` fixture refused with its code |
| 6 Number grammar and sentences | S5; §8, §19.8-§19.10 | Day 4 | 62 parse rows and 10 display rows of `numbers_table.rs` green |
| 7 Detector rules | S7 part 1; §9.1-§9.8, §19.11 | Day 4-5 | 34 rows of `rules_table.rs` green |
| 8 Re-detection, determinism, precision gate | S7 part 2; §9.2, §10.2, §13.3, §19.12-§19.14, §20.3 | Day 5 | Precision at least 0.90 over at least 30 labeled files |
| 9 Records and the full pipeline | S8; §13.5, §15 | Day 6 | The reference clip reaches `ready` in `pnpm dev` with "Cleaning voice", five "Found:" lines and three IndexedDB records |
| 10 Rejection panel and copy | S9; §16, §18.4 | Day 6 | 14 rejection messages; `COPY_PENDING` holds no `REJECT_*` |
| 11 E2E suites, corpus tooling, CI | S10, S11; §17.6, §18.2-§18.6, §19.15-§19.20 | Day 7 | `pnpm e2e:media` on D1 green (at least 48 cases); both CI jobs green on the pull request |
| 12 Measure | S12; §20 | Day 8 | Bench files committed; TE-13, TE-12, E-3, E-4, E-10 smoke and the precision gate recorded |
| 13 Copy-back, merge, deploy, tag | S13, S14, S15; §21.3, §22, §23 | Day 8 | The deployed page refuses `rej_4k.mp4` with its message; `v3` pushed |

Total: 8 working days, as §5 plans (BP §0 gives V3 three days; §5 overrides it, see "Assumptions").

## Read this first: 8 things that will bite you

1. **Six Rust test files read generated fixtures, and none of them can pass before `pnpm fixtures`.** Besides the two `offcut-mp4` files and `loudness_target.rs`, `duration_preserved.rs`, §19.5 (a speech PCM case and `chain_timing`) and §19.7 (the portrait PCM case) read `fixtures/.gen/speech/*.pcm48.f32` (§6.4 and §19.1 say so since the review of 2026-10-15). Copy the helper of §6.4 (stamp check included) into all six files: `demux_fixtures.rs`, `probe_rejections.rs`, `loudness_target.rs`, `duration_preserved.rs`, `chain_no_clicks.rs`, `tests/prosody.rs`. From Phase 2 on, `cargo test --workspace` fails with "run `pnpm fixtures`" on any machine where the fixtures are missing or stale (after any edit of `fixtures/manifest.json` or `gen_fixtures.sh`). Applied in Steps 3.1 and 5.3; the CI order (fixtures before step 4) in Step 11.9.

2. **The fixture script runs under three shells and two ffmpeg builds.** `pnpm fixtures` runs `sh fixtures/gen_fixtures.sh` through `cmd.exe` on Windows, which finds `sh` only because Git Bash exported its PATH: run pnpm from Git Bash, never from PowerShell or WSL. The script must be stored with LF line endings. `-map_metadata`, `-map_chapters`, `-fflags`, `-flags` and `-threads` are output options and must come after the last `-i`; `-y -hide_banner -loglevel error -nostdin` come before (§17.1). The version check must accept the common Linux static builds, which print `ffmpeg version n9.0.2`. Both are done with two flag variables and a `case` that accepts both prefixes (snippet in Step 2.2). Applied in Steps 0.1, 2.2, 0.8.

3. **The labeled set is frozen in S2, but one of its files cannot have data until Phase 12, and the coverage check reads `tags`.** V2 stores no transcript and measures no prosody, so `a01` and `a02` cannot carry measured `words` and `prosody` in S2 (D-87, §17.5). Hence: (a) `a01-scriptA.json` is written-form from the recognized sentences in `fixtures/speech/README.md`, no emphasis marks, no KeywordPop expected; (b) `a02-v3testclip.json` is committed in S2 with its `expected` labels from the founder's words and `"words": null, "prosody": null`; `precision_recall.rs` skips an `asr` file with null data, prints `skipped (no data yet)`, and counts only read files toward 30; its data is filled in Step 12.3 with a changelog line "data, not labels"; (c) the coverage of traps, list forms and edge cases is counted from the optional `"tags"` array (`deny_unknown_fields` refuses any other extra field). The written files' real totals are NumberReveal 10, ListReveal 7, announcer 4, empty files 8; with `a01` every minimum is met. All of this is in §17.5 since the review of 2026-10-15. **Before writing any file, the founder decides §23 item 27** (a written emphasis scores exactly 0.80, the KeywordPop threshold). Applied in Steps 0.9, 2.5, 2.6, 2.7, 8.3, 12.3.

4. **Three frozen contracts are reopened, and one of them breaks compilation in the step that reopens it.** D-83 changes `messages.feed.eventFound.from_to` to two arguments in S1, but §4 assigns the `ProcessingFeed.tsx` edit that calls it to S9: `tsc` fails from S1 to S9. Fix: make the `ProcessingFeed.tsx` change in Step 1.6. D-91 (three `pipeline-preview.spec.ts` cases, `bench/device-bench.ts` `STAGES`) and D-103 (one `MINTING` entry) are the other two. Every other V1/V2 file listed as untouched in §4 must stay untouched: run the frozen check of Step 1.7 at every milestone.

5. **The two new crates can break purity, licences or the WASM build, and they are committed in S1 before anything uses them.** Dry-run them on a scratch branch in Step 0.7 (`cargo tree` against the forbidden list, `cargo deny`, a `wasm32` build). If the workspace denies `unused_crate_dependencies`, adding them in S1 fails clippy: Step 1.5 checks this and moves the two lines to Phase 3 if needed. After Phase 4, `nnnoiseless` grows `offcut_core_bg-*.wasm` and may carry URL text that `check-hosts.mjs` rejects (§18.7): Step 4.6.

6. **Five fixture facts depend on the machine and must be observed, not guessed.** The rotation value the demuxer reports for `ok_rotated_90.mov` (R90 or R270), the measured duration and average rate of `ok_vfr.mp4`, whether ffmpeg marks the first track of `ok_two_video_first_disabled.mp4` disabled, whether Chrome decodes `rej_h264_444.mp4` (D-78), and whether the recognizer finds fewer than 3 words in `rej_silent.mp4` (D-111). The manifest holds `null` for the first three until observed. The `rej_vp9.webm` recipe needs `libopus` (§1A item 6), and `libx265` may need `pools=1:frame-threads=1` to be deterministic. Applied in Steps 0.2, 2.4, 5.3.

7. **The spec tree must list new files before `check-file-tree.mjs` sees them.** Add `crates/offcut-mp4/src/edit_list.rs`, `crates/offcut-text/src/cardinals.rs` and `docs/v3/{v3implementation.md, v3buildguide.md, coding-promptsv3.md, v3changelog.md, experiments.md}` to TS §5 in Step 1.3 (D-73). §4 marks `crates/offcut-mp4/Cargo.toml` (dev `sha2`), `web/tests-e2e/tsconfig.json` and `scripts/check-external-facts.mjs` as changed; the frozen check of Step 1.7 must not list them. `corpus/manifest.json` goes in Step 1.8 (S1, D-99), the corpus scripts and `device-bench.ts` in Phase 11 (S11).

8. **The phase order is not §5's order.** S6 (the chain in the browser) runs before S3 and S5, because TE-13 is the result most likely to change the design and §5 itself says "S4 goes first because TE-13 can change the chain's design"; S6 depends on S4 only. S7 is split over two adjacent phases (7 and 8) because it is 2.5 days of work. The "as recognized" sections of `fixtures/speech/README.md` are filled in Step 9.6 (S8, as §5 and D-99 say): V2 has no path to read a transcript, detects only NumberReveal and measures no prosody. Commit titles keep their S-numbers (D-75).

---

## Phase 0 - Environment and V2 baseline (§1A, §3)

Goal: every tool V3 needs prints its version on D1, the V2 baseline at `8a271a9` passes with its recorded counts on a new branch `v3-build`, and every lead-time input is requested.

### Step 0.1 - Open the right shell

```bash
echo "$SHELL"; uname -s                 # expect: /usr/bin/bash; MINGW64_NT-10.0-...
which bash sh                           # expect: /usr/bin/bash and /usr/bin/sh (Git's), not /c/Windows/System32/bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut   # the repository is offcut/, not the sh2clips folder above it
cmd //c where sh                        # expect first line: C:\Program Files\Git\usr\bin\sh.exe
```

If `cmd //c where sh` prints nothing, `pnpm fixtures` will fail with "'sh' is not recognized". Fix once, per user: `pnpm config set script-shell "C:\\Program Files\\Git\\bin\\bash.exe"`, then re-run the check with `pnpm exec sh -c 'echo ok'` (expect `ok`).

### Step 0.2 - Audit the tools

| Tool | Check | Need | Action if it fails |
|---|---|---|---|
| Rust | `rustc --version`; `rustup target list --installed` | The version in `rust-toolchain.toml`; `wasm32-unknown-unknown` | `rustup show` installs the pinned toolchain; `rustup target add wasm32-unknown-unknown` |
| wasm-bindgen CLI | `wasm-bindgen --version` | Equal to the crate in `Cargo.lock` (`grep -A1 'name = "wasm-bindgen"' Cargo.lock`) | `cargo install wasm-bindgen-cli --version <that> --locked` |
| wasm-opt | `wasm-opt --version` | As V2 | V2's install |
| cargo-deny | `cargo deny --version` | As V2 | `cargo install cargo-deny --locked` |
| Node, pnpm | `node --version`; `pnpm --version` | `packageManager` of `package.json` | `corepack enable` |
| Playwright | `pnpm --filter web exec playwright --version` | As V2 | `pnpm --filter web exec playwright install chrome` |
| Chrome stable | `chrome://version` in Chrome | WebGPU enabled (V2) | Update Chrome |
| Python, NumPy | `python -c "import numpy; print(numpy.__version__)"` | As V2 | `pip install -r verify/requirements.txt` |
| ffmpeg, ffprobe | `ffmpeg -hide_banner -version \| head -1`; `ffprobe -version \| head -1` | `9.0.2`, the "essentials" archive of `e2e-media.yml` | Install that archive; D1 and the Windows runner must use the same one |
| sha256sum, curl, shuf, tar, xz | `sha256sum --version \| head -1` (and the same for each) | Git Bash's | Git for Windows |
| Docker, psql | not needed | V3 changes no server code; rows are read in the Neon SQL editor | - |

```bash
ffmpeg -hide_banner -version | head -1                                   # expect: ffmpeg version 9.0.2-essentials_build-...
for e in libx264 libx265 libaom-av1 libsvtav1 libvpx-vp9 libmp3lame libopus aac; do
  ffmpeg -hide_banner -encoders 2>/dev/null | grep -qw "$e" && echo "ok      $e" || echo "MISSING $e"
done                                                                     # expect: ok for all but at most one of libaom-av1 / libsvtav1
ffmpeg -hide_banner -h encoder=libx264 2>/dev/null | grep -o yuv444p | head -1   # expect: yuv444p
for f in testsrc2 sine anullsrc anoisesrc drawbox concat; do
  ffmpeg -hide_banner -filters 2>/dev/null | grep -qw "$f" && echo "ok      $f" || echo "MISSING $f"
done                                                                     # expect: ok for all six
```

**`libopus` is needed by `rej_vp9.webm` (§17.2) though §1A item 6 does not list it.** If any encoder is missing from the essentials archive, choose the replacement now (the "full" archive, or `libsvtav1` for AV1), write it into `~/v3-baseline.txt`, and plan the matching change of the Windows runner's install step in Step 11.9.

### Step 0.3 - Cut the branch from the V2 baseline

```bash
git fetch origin --tags --prune
git status --short                              # expect: no output
git rev-parse --short origin/main               # expect: 8a271a9
git tag -l v2                                   # expect: v2, or nothing
git switch -c v3-build origin/main
git merge-base v3-build origin/main | cut -c1-7 # expect: 8a271a9
```

If `v2` does not exist, tag it when convenient (founder's action, D-75): `git tag -a v2 -m "V2" 8a271a9 && git push origin v2`. Write "present" or "absent" into `~/v3-baseline.txt`.

### Step 0.4 - Check the V2 stand-ins and the V2 copy-back

```bash
git grep -n "V3: \|V7: " -- web/src/usecases | wc -l       # expect: 5 (§1A item 7)
git grep -n "V3: " -- web/src/usecases                     # expect: the two markers in run-pipeline.ts
grep -cE "sps\.rs|mux_boxes\.rs|media\.setup\.ts" docs/technicalspec.md   # expect: 3 or more (§1A item 8)
```

Open V2 §27's "copy back" list and mark each item applied or not in `~/v3-baseline.txt`; the unapplied ones are applied in Step 1.3.

### Step 0.5 - Check the two founder clips and keep them out of git

```bash
ls -l testclips/                                            # expect: speech_scriptA_landscape_720p.mp4, v3testclip.mp4
ffprobe -v error -show_entries format=duration,size -of compact testclips/v3testclip.mp4
# expect: format|duration=78.918...|size=46360015
ffprobe -v error -show_entries format_tags -of compact testclips/v3testclip.mp4 | grep -io location
# expect: location   (the GPS tag that must never leave D1, D-100)
for f in testclips/speech_scriptA_landscape_720p.mp4 fixtures/.cache/media/speech_scriptA_landscape_720p.*.mp4; do
  ffprobe -v error -show_entries format_tags:stream_tags -of compact "$f" | grep -qiE 'location|iso6709' && echo "LOCATION IN $f"
done; echo checked
# expect: checked, no LOCATION line. The reference clip is public on the asset host as the sample clip (D-39);
# a byte search on 2026-10-15 found no location atom in it. If one appears, stop and tell the founder before anything else.
git check-ignore -v testclips/v3testclip.mp4                # expect: .gitignore:<n>:<pattern>  testclips/v3testclip.mp4
sha256sum testclips/speech_scriptA_landscape_720p.mp4 testclips/v3testclip.mp4 >> ~/v3-baseline.txt
```

**If `git check-ignore` prints nothing, do not run `git add -A` or `git add .` until Step 1.4 adds `testclips/` to `.gitignore`.**

### Step 0.6 - Run the V2 baseline and record the counts

Find how V2 builds for the media suites:

```bash
grep -nE '"(build|e2e|e2e:media|e2e:device|bench:device|verify|build:wasm)"' package.json
```

If `e2e:media` does not build with the test key itself, export `VITE_ENTITLEMENT_TEST_PUBLIC_KEY` in the shell (value from where V2's changelog says it lives; it is a public test key and is never committed) and build first.

```bash
pnpm install --frozen-lockfile                              # expect: Lockfile is up to date
pnpm check                                                  # expect: exit 0
cargo test --workspace 2>&1 | awk '/^test result:/ {p+=$4; f+=$6; i+=$8} END {print p" passed, "f" failed, "i" ignored"}'
# expect: 262 passed, 0 failed, 2 ignored, or 260 passed and 2 ignored: V2's changelog writes "262 Rust (2 ignored)"
# without saying whether the 262 counts the two. Record the exact line; every later "262" in this guide means that line
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"      # expect: Tests  92 passed (92)
pnpm e2e 2>&1 | tail -3                                     # expect: 12 passed
pnpm e2e:media 2>&1 | tail -3                               # expect: 30 passed (D1, keyed build)
ls fixtures/.cache/                                         # expect: the reference clip, named with its 16-hex hash segment
pnpm build && ls -l web/dist/assets/ | grep -E 'offcut_(core|render)_bg-.*\.wasm'
# record both sizes in ~/v3-baseline.txt (the "before" of Step 4.6)
```

The export cases of `pnpm e2e:media` run `verify_mp4.py` checks 1-6, which satisfies §1A item 10.

### Step 0.7 - Dry-run the two new crates on a scratch branch (§1A item 9, D-89)

```bash
git switch -c scratch/v3-deps
cargo add -p offcut-dsp nnnoiseless --no-default-features
cargo add -p offcut-dsp ebur128 --no-default-features
grep -E '^(nnnoiseless|ebur128)' crates/offcut-dsp/Cargo.toml          # record the two versions
cargo tree -p offcut-dsp -e normal --prefix none | sort -u \
  | grep -E '^(web-sys|js-sys|wasm-bindgen|getrandom|rand|wgpu) '    # expect: no output
cargo deny check licenses bans 2>&1 | tail -4                          # expect: licenses ok, bans ok, or the licence to add
cargo build -p offcut-dsp --target wasm32-unknown-unknown --release    # expect: Finished
cargo tree -p offcut-dsp -e normal > ~/v3-dsp-tree.txt
git checkout -- . && git switch v3-build && git branch -D scratch/v3-deps
git status --short                                                     # expect: no output
```

| Result | Do |
|---|---|
| A licence outside the allowlist | Note the SPDX id; it is added to `deny.toml` with its crate in a comment in Step 1.5 |
| A forbidden crate behind a default feature | Find the feature in `cargo tree -e features -i <crate>`; turn it off in Step 1.5 |
| A forbidden crate that cannot be turned off, or no `wasm32` build | Stop. `ebur128`: replace the crate. `nnnoiseless`: TE-13's fallback (`denoise: false`) applies from the start; record it |

### Step 0.8 - Pick the Linux ffmpeg archive (lead time, D-94)

The Linux CI job needs a version-pinned static build. Choose a URL whose path names `9.0.2` (or the nearest `9.0.x`), never a "latest" URL.

```bash
export FFMPEG_LINUX_URL='<a URL whose path names the version>'
curl -fL -o /tmp/ffmpeg-linux.tar.xz "$FFMPEG_LINUX_URL"    # expect: exit 0
sha256sum /tmp/ffmpeg-linux.tar.xz | cut -d' ' -f1          # this is FFMPEG_LINUX_SHA256
tar -tJf /tmp/ffmpeg-linux.tar.xz | grep -E '/(ffmpeg|ffprobe)$'   # expect: two paths; note the folder they are in
```

Write the URL, the hash and the binaries' folder into `~/v3-baseline.txt`. The binary cannot run on D1; its version line is read in the CI log in Step 11.10.

### Step 0.9 - Request the inputs that wait on a person

| Input or account | Do now | Keep for later |
|---|---|---|
| The words spoken in `v3testclip.mp4` (D-99) | Write them, sentence by sentence, into `~/v3testclip-words.txt` (outside the repo) | Step 2.7 (labels and README), Step 12.5 (WER reference) |
| Corpus clips lent by other people (D-88) | Ask now; store them on D1 in one folder, named `<id>.<ext>` | `OFFCUT_CORPUS_DIR`, Step 12.4 |
| Founder's five-file reading (D-87) | Nothing | Step 12.6 |
| Founder's confirmation of D-112 for `a01`, `a02` | Nothing | Step 13.6 |
| Founder's decision on §23 item 27 (written emphasis at exactly the KeywordPop threshold) | Ask now: `pitch_z` 2.0 for written emphasis, or round scores before the threshold | Step 2.5, before any labeled file is written |
| Founder's decision on §23 item 26 (IndexedDB records outlive "start over" until V7) | Ask now: accept until V7, or a `clear()` per repo at import step 4 | Step 9.1 |
| GitHub Actions | Nothing (the repository is public; about one Linux minute more per run) | - |
| Asset host | Nothing. **Never upload `v3testclip.mp4` or a derived fixture** | - |
| Neon (Postgres) | Nothing | SQL editor in Step 13.5 |
| Vercel, Render | Nothing; `ci.yml` deploys | Step 13.4 |

V3 adds no secret. `FFMPEG_LINUX_URL` and `FFMPEG_LINUX_SHA256` are public values in `ci.yml`. The test public key lives in your shell only.

### Milestone 0

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut
git branch --show-current                                   # expect: v3-build
git rev-parse --short HEAD                                  # expect: 8a271a9
ffmpeg -hide_banner -version | head -1                      # expect: ffmpeg version 9.0.2-...
cargo deny --version && wasm-bindgen --version && wasm-opt --version   # expect: three version lines
git check-ignore -q testclips/v3testclip.mp4 && echo ignored   # expect: ignored (or the Step 1.4 note is in ~/v3-baseline.txt)
cat ~/v3-baseline.txt                                       # expect: counts 262/2, 92, 12, 30; two SHA-256 lines; two wasm sizes; two crate versions; the Linux ffmpeg URL and hash
```

Pass: every tool prints a version, every encoder of Step 0.2 is present or its replacement is recorded, the four counts equal 262 passed and 2 ignored, 92, 12, 30, and the scratch branch left no change.

Commit: none. Phase 0 changes no tracked file; its records go into the first changelog entry in Step 1.1.

Do not start Phase 1 until this passes.

---

## Phase 1 - Contracts, dependencies and docs (S1)

Goal: `v3-build` holds the V3 docs, the updated spec tree, the two new crates, the reopened contracts D-83 and D-103, the ignore rules and the corpus manifest, with `pnpm check` green and every V2 test count unchanged.

### Step 1.1 - Create the `docs/v3/` files

```bash
mkdir -p docs/v3
cp <where you keep it>/v3implementation.md docs/v3/v3implementation.md
cp <where you keep it>/v3buildguide.md docs/v3/v3buildguide.md
```

`docs/v3/v3changelog.md` (no format is given in the specs; this one mirrors the V2 changelog's two tables):

```markdown
# v3changelog.md - record of every change made while building V3

## Entry 1 - <date> - S1: baseline
| Item | Value |
|---|---|
| Baseline commit | 8a271a9<rest of the SHA> |
| `v2` tag | present / absent |
| Rust tests at baseline | 262 passed, 2 ignored |
| Vitest at baseline | 92 |
| `pnpm e2e` at baseline | 12 |
| `pnpm e2e:media` at baseline (D1, keyed) | 30 |
| SHA-256 `speech_scriptA_landscape_720p.mp4` | <hex> |
| SHA-256 `v3testclip.mp4` | <hex> |
| `nnnoiseless`, `ebur128` | <version>, <version> |
| ffmpeg on D1 | <first line of ffmpeg -version>; replacements: <none or list> |
| Linux ffmpeg archive | <URL>, <SHA-256> |
| `offcut_core_bg`, `offcut_render_bg` size at baseline | <bytes>, <bytes> |
| Docs review before S1 | 2026-10-15: `v3implementation.md` and this guide made consistent with each other and with the V2 changelog (six fixture-reading tests, `@no-gpu` count 15, `a01`/`a02` handling, sorted lexicons, `limit` count, §8.7 and §19.11 expectations, metadata checks, §23 items 25-28) |
| Founder's decisions on §23 items 26 and 27 | <answers> |

## Open items
| # | Item | Owner | Version |
|---|---|---|---|

## Known issues
| # | Issue | Where | Version |
|---|---|---|---|
```

`docs/v3/experiments.md`: one heading per row of §20.1 (`TE-13 native`, `TE-13 browser`, `DENOISE_DELAY_SAMPLES`, `Precision gate`, `TE-12`, `E-3 re-read`, `E-4 re-read`, `E-10 smoke`, `E-10` (carried to V9, D-88), `rej_h264_444 in Chrome`, `rej_silent`, `Derived clips' events`, `Second recording`, `Bundle size`), each with the lines `Question:`, `Method:`, `Numbers:`, `Date:`, `Decision:` left empty.

### Step 1.2 - Close V2's experiments file

Append to `docs/v2/experiments.md` (§1A "Carried over"):

```markdown
## M0 gate decision (recorded <date>, at V3 S1)
Decision: continue (the founder's, 2026-10-15).
E-3: about 150 s for 60 s on the founder's machine (D-67); 62-96 s on D1.
E-4: 29,641 ms on D1.
E-1: inconclusive; the page was not announced.
S15 of V2 was not run; V3's Step 12.1 commits the first `d1` bench file.
```

### Step 1.3 - Update the spec tree (TS §5) before any new file exists

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `docs/technicalspec.md` §5 | D-73, D-92 | Under `crates/offcut-mp4/src/` after `sample_table.rs`: `edit_list.rs  edit-list shapes E0-E4 (v3implementation D-92)`. Under `crates/offcut-text/src/` after `tokenize.rs`: `cardinals.rs  spelled cardinals and "point" decimals (v3implementation D-92)`. Under `docs/`: `v3/v3implementation.md`, `v3/v3buildguide.md`, `v3/coding-promptsv3.md`, `v3/v3changelog.md`, `v3/experiments.md`, each with a one-line purpose in the style of the `v2/` lines |
| 2 | `docs/technicalspec.md` (other sections) | V2 §27 | Every V2 copy-back item Step 0.4 found unapplied |

```bash
node scripts/check-file-tree.mjs                            # expect: exit 0
```

If it fails because `edit_list.rs` or `cardinals.rs` is listed but missing, remove those two lines now and add them in Steps 5.1 and 6.3 in the same commit as the file. Record which behaviour the script has in `v3changelog.md`.

### Step 1.4 - Ignore the generated and private folders (D-98)

```bash
printf '%s\n' 'fixtures/.gen/' 'corpus/.clips/' 'corpus/.results/' >> .gitignore
git check-ignore -q testclips/v3testclip.mp4 || printf '%s\n' 'testclips/' >> .gitignore
mkdir -p fixtures/.gen corpus/.clips corpus/.results
touch fixtures/.gen/probe corpus/.clips/probe corpus/.results/probe
git status --short --untracked-files=all | grep -E 'fixtures/\.gen|corpus/\.|testclips'   # expect: no output
rm fixtures/.gen/probe corpus/.clips/probe corpus/.results/probe
```

### Step 1.5 - Add the dependencies (§3.2, §18.1)

First check whether an unused dependency fails the build:

```bash
grep -rn "unused_crate_dependencies" Cargo.toml crates/*/src/lib.rs clippy.toml   # expect: no output
```

If it prints a line, add the `offcut-dsp` lines below in Step 3.1 instead, and each dev-dependency in the phase whose tests use it.

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `Cargo.toml` `[workspace.dependencies]` | §18.1, D-89 | `nnnoiseless` and `ebur128` with `default-features = false`, at the versions of Step 0.7 |
| 2 | `crates/offcut-dsp/Cargo.toml` | §18.1 | `nnnoiseless = { workspace = true }`, `ebur128 = { workspace = true }`; dev `sha2` |
| 3 | `crates/offcut-mp4/Cargo.toml` | §18.1 (not marked in §4) | dev `sha2 = { workspace = true }` |
| 4 | `crates/offcut-text/Cargo.toml` | §18.1 | dev `proptest = { workspace = true, features = ["std"] }` |
| 5 | `crates/offcut-detect/Cargo.toml` | §18.1 | dev `serde` (`derive`), `serde_json`, `proptest` |
| 6 | `deny.toml` | §18.1 | Only the licences `cargo deny check licenses` names, each with its crate in a comment; no ban loosened |

```bash
cargo build --workspace                                     # expect: Finished
cargo deny check                                            # expect: advisories ok, bans ok, licenses ok, sources ok
cargo tree -p offcut-dsp -e normal --prefix none | sort -u | grep -E '^(web-sys|js-sys|wasm-bindgen|getrandom|rand|wgpu) '   # expect: no output
cargo build -p offcut-dsp --target wasm32-unknown-unknown --release   # expect: Finished
```

### Step 1.6 - Reopen the two V1/V2 contracts that S1 owns (D-83, D-103)

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `web/src/workers/protocol.ts` | §13.1, D-83 | One optional field `displayTo?: string` on the `event_found` arm; no other line changes. **REOPENED V1 CONTRACT** |
| 2 | `web/src/copy/messages.ts` | §16.3 | `feed.eventFound.from_to` becomes `(display: string, displayTo: string) => string`; `displayTo === ""` gives `Found: {display}` |
| 3 | `web/src/ui/components/ProcessingFeed.tsx` | §16.2 (moved here from S9, item 4) | `from_to` lines call `found.from_to(line.display, line.displayTo ?? "")`; the `assertNever` default stays |
| 4 | `web/eslint.config.js` | D-103 | One `MINTING` entry `{ brand: "Bytes", helper: "toBytes" }` scoped to `persistence/clips-repo.ts`, in the form V2's entries use. **REOPENED V2 CONTRACT** |

```bash
pnpm check                                                  # expect: exit 0
```

### Step 1.7 - Define the frozen check

Run this at every milestone from now on. It lists every file §4 says V3 must not touch.

```bash
git diff --name-only 8a271a9 -- \
  server crates/offcut-types crates/offcut-api-types crates/offcut-scene crates/offcut-render crates/offcut-entitlement \
  crates/offcut-mp4/src/reader.rs crates/offcut-mp4/src/boxes.rs crates/offcut-mp4/src/demux.rs \
  crates/offcut-mp4/src/sps.rs crates/offcut-mp4/src/validate.rs crates/offcut-mp4/src/mux.rs crates/offcut-mp4/src/mux_boxes.rs \
  crates/offcut-dsp/src/resample.rs crates/offcut-detect/src/event_id.rs \
  crates/offcut-wasm-core/src/media_api.rs crates/offcut-wasm-core/src/hash_api.rs crates/offcut-wasm-core/src/text_api.rs \
  crates/offcut-wasm-render/src/lib.rs crates/offcut-wasm-render/src/session.rs \
  crates/offcut-wasm-render/src/profile_api.rs crates/offcut-wasm-render/src/mux_api.rs \
  web/vercel.json web/src/gen web/src/net web/src/models web/src/platform web/src/state web/src/analytics/client.ts \
  web/src/persistence/schema.ts web/src/persistence/db.ts web/src/persistence/opfs.ts web/src/persistence/entitlement-repo.ts \
  web/src/usecases/start-app.ts web/src/usecases/control-preview.ts web/src/usecases/start-export.ts \
  web/src/workers/rpc.ts web/src/workers/media.worker.ts web/src/workers/asr.worker.ts web/src/workers/render.worker.ts \
  web/src/workers/media web/src/workers/asr web/src/workers/render \
  verify render.yaml
# expect: no output
```

And the positive view: `git diff --name-only 8a271a9 | sort` must list only files of §4 (which since the review of 2026-10-15 includes `crates/offcut-mp4/Cargo.toml`, `web/tests-e2e/tsconfig.json` and `scripts/check-external-facts.mjs`) and, from Step 12.1, `bench/results/`. It must never list `.env*`, `testclips/`, `fixtures/.gen/`, `fixtures/.cache/`, `corpus/.clips/`, `corpus/.results/`, `web/test-results/` or `web/playwright-report/`.

### Step 1.8 - Create `corpus/manifest.json` (D-99, §17.6)

Two entries, `founder-01` and `founder-02`, exactly as the example of §17.6, with the SHA-256 values from Step 0.5. Tags as §17.6 shows; `expect: "accept"`; `owner: "founder"`.

```bash
node -e 'const m=require("./corpus/manifest.json"); console.log(m.schema, m.clips.map(c=>c.id).join(","))'
# expect: 1 founder-01,founder-02
```

### Step 1.9 - Prove the guards

| Temporary edit | Rule that must fire |
|---|---|
| `touch crates/offcut-dsp/src/stray.rs` | `node scripts/check-file-tree.mjs` fails naming `stray.rs` |
| In `ProcessingFeed.tsx`, call `found.from_to(line.display)` with one argument | `pnpm check` fails in `tsc` |
| `touch fixtures/.gen/x.mp4` | `git status --short` shows nothing for it |

Undo all three; `git status --short` shows only this phase's intended files.

### Milestone 1

```bash
pnpm check                                                  # expect: exit 0
cargo deny check                                            # expect: exit 0
cargo test --workspace 2>&1 | awk '/^test result:/ {p+=$4; f+=$6; i+=$8} END {print p" passed, "f" failed, "i" ignored"}'
# expect: 262 passed, 0 failed, 2 ignored
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"      # expect: Tests  92 passed (92)
pnpm e2e 2>&1 | tail -2                                     # expect: 12 passed
# run the frozen check of Step 1.7                          # expect: no output
git status --short --untracked-files=all | grep -E 'testclips|fixtures/\.gen|corpus/\.'   # expect: no output
```

Pass: all checks exit 0, the counts equal the baseline, the frozen check prints nothing, and no private or generated path is staged.

Commit: `"V3 S1: contracts, dependencies, docs/v3 and corpus manifest"`

Do not start Phase 2 until this passes.

---

## Phase 2 - Fixtures and the frozen labeled set (S2)

Goal: `pnpm fixtures` builds the 25 fixtures of §17.2-§17.3 deterministically on D1, and the labeled set (`w01`-`w29`, `a01`, `a02`) is committed before any detector rule file exists.

### Step 2.1 - Write `fixtures/manifest.json`

One entry per fixture, in this order: the 20 synthetic fixtures of §17.2 (7 `ok_*`, 13 `rej_*`), then the 5 derived fixtures of §17.3. Format: §17.4. Set `"pcm": true` on the 5 speech fixtures only (§17.1 step 7: the synthetic signals of `loudness_target.rs` are made inside the test, §19.4). Fields that must be observed stay `null` until Step 2.4 or 5.3: `ok_rotated_90.mov` `expect.accept.rotation`, `ok_vfr.mp4` `duration_ms` and `avg_fps_milli`.

```bash
node -e '
const m=require("./fixtures/manifest.json"), f=m.fixtures;
const acc=f.filter(x=>x.expect.accept).length, rej=f.filter(x=>x.expect.reject).length;
console.log(f.length, "fixtures;", acc, "accept;", rej, "reject;", f.filter(x=>x.pcm).length, "pcm")'
# expect: 25 fixtures; 12 accept; 13 reject; 5 pcm
```

### Step 2.2 - Write `fixtures/gen_fixtures.sh`

The steps are §17.1's; the recipes are §17.2 and §17.3. The skeleton below fixes what the specs do not: the split of input and output flags, the version check, the hash form, and exact quoting under Git Bash. Recipes not shown follow the same pattern.

```sh
#!/bin/sh
# fixtures/gen_fixtures.sh - generated and derived fixtures (v3implementation §17, D-76, D-77, D-100, D-111)
set -eu
ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT=${OFFCUT_FIXTURES_DIR:-$ROOT/fixtures/.gen}
MANIFEST=$ROOT/fixtures/manifest.json
hash() { sha256sum "$1" | cut -d' ' -f1; }

# Step 1: ffmpeg 9.0 (Windows prints "9.0.2-essentials...", Linux static builds print "n9.0.2...")
ver=$(ffmpeg -hide_banner -version | head -1)
case "$ver" in "ffmpeg version 9.0"*|"ffmpeg version n9.0"*) ;; *) echo "ffmpeg 9.0 is needed, found: $ver" >&2; exit 2 ;; esac

# Input-side flags go before the first -i; output flags after the last -i (§17.1 lists them together)
GLOBAL="-y -hide_banner -loglevel error -nostdin"
OUTF="-threads 1 -fflags +bitexact -flags:v +bitexact -flags:a +bitexact -map_metadata -1 -map_chapters -1"
H264="-c:v libx264 -preset veryfast -bf 0 -g 30 -pix_fmt yuv420p"
AAC="-c:a aac -b:a 128k -ar 48000 -ac 2"
src() { echo "-f lavfi -i testsrc2=size=$1x$2:rate=$3:duration=$4 -f lavfi -i sine=frequency=440:sample_rate=48000:duration=$4"; }

# Step 2: reference clip (adapt the sed pattern to the literal form in asset-fetch.ts; check it with echo)
SAMPLE_CLIP_PATH=$(sed -n 's/.*SAMPLE_CLIP_PATH[^"]*"\([^"]*\)".*/\1/p' "$ROOT/web/src/net/asset-fetch.ts" | head -1)
# ... choose REF per §17.1 step 2; compare the first 16 hex of hash "$REF" with the hash segment of the name; exit 3 on mismatch

# Step 3: STAMP up to date -> "fixtures are up to date", exit 0     Step 4: rm -rf "$OUT"; mkdir -p "$OUT/speech"

# Step 5, example synthetic recipe
ffmpeg $GLOBAL $(src 320 180 30 91) $OUTF $H264 $AAC -movflags +faststart "$OUT/rej_too_long_91s.mp4" \
  || { echo "recipe rej_too_long_91s failed" >&2; exit 4; }

# Step 6, example derived recipe: fps first, so frame numbers count 30 fps output frames
ffmpeg $GLOBAL -ss 0.960 -i "$REF" $OUTF -t 60 \
  -vf "fps=30,crop=ih*9/16:ih,scale=1080:1920:flags=lanczos,drawbox=x=0:y=0:w=iw:h=ih:color=white:t=fill:enable='eq(n\,150)+eq(n\,1650)'" \
  $H264 $AAC -movflags +faststart "$OUT/speech/speech_60s_portrait.mp4" \
  || { echo "recipe speech_60s_portrait failed" >&2; exit 4; }

# Step 7: for each "pcm": true entry: ffmpeg $GLOBAL -i "$f" -map 0:a:0 -ac 1 -ar 48000 -f f32le "$f.pcm48.f32"
# Step 8: ffprobe checks against the manifest (JSON parsed with `node -e`; node is present wherever pnpm runs); exit 5 on mismatch
# Step 9: write STAMP last (§17.4)
```

| Watch for | Why |
|---|---|
| `$GLOBAL`, `$OUTF`, `$H264`, `$AAC`, `$(src ...)` are unquoted on purpose | Word splitting turns them into separate arguments; no value inside them may contain a space |
| `\,` inside the double-quoted `-vf` | `sh` keeps the backslash; ffmpeg needs it to escape the comma in `eq(n\,150)` |
| `rej_truncated.mp4` | Write the faststart file to a temporary name, then `head -c 1500 tmp > "$OUT/rej_truncated.mp4"` |
| `rej_hevc.mov` | If the run-twice check of Step 2.3 differs on this file, add `pools=1:frame-threads=1` to `-x265-params` and record it |
| `rej_av1.mp4` | `libaom-av1 -cpu-used 8 -row-mt 0`, or `libsvtav1 -preset 12` when Step 0.2 found no libaom; record which |
| Room tone (§17.3) | `anoisesrc=color=pink:amplitude=0.001:sample_rate=48000:seed=7`, joined with `concat` in one `filter_complex`; no crossfade |
| The script uses `node -e` in step 8 | A small deviation from §17.1's tool list; node is installed wherever `pnpm fixtures` runs |

```bash
git ls-files --eol fixtures/gen_fixtures.sh 2>/dev/null; file fixtures/gen_fixtures.sh   # expect after the first add: i/lf w/lf; "POSIX shell script ... text" without CRLF
```

### Step 2.3 - Add `pnpm fixtures`, run it twice, check determinism and metadata

Root `package.json`: `"fixtures": "sh fixtures/gen_fixtures.sh"` (§18.2). The `corpus` script is added in Step 11.6.

```bash
time pnpm fixtures                                          # expect: exit 0; about 40 s on D1 (record it)
find fixtures/.gen -type f | wc -l                          # expect: 31 (25 fixtures + 5 PCM + STAMP)
( cd fixtures/.gen && find . -type f ! -name STAMP | sort | xargs sha256sum ) > /tmp/gen1.txt
rm -rf fixtures/.gen && pnpm fixtures
( cd fixtures/.gen && find . -type f ! -name STAMP | sort | xargs sha256sum ) > /tmp/gen2.txt
diff /tmp/gen1.txt /tmp/gen2.txt && echo deterministic      # expect: deterministic
pnpm fixtures                                               # expect: fixtures are up to date
for f in fixtures/.gen/speech/*.mp4; do
  ffprobe -v error -show_entries format_tags:stream_tags -of compact "$f" | grep -qiE 'location|creation_time' && echo "LEAK $f"
done; echo checked                                          # expect: checked (no LEAK line, D-100)
for f in fixtures/.gen/speech/*.mp4; do
  printf '%s ' "$(basename "$f")"
  ffprobe -v error -select_streams v:0 -show_entries stream=width,height,nb_frames -show_entries format=duration -of csv=p=0 "$f" | tr '\n' ' '; echo
done
```

| Fixture | Expect from the last command |
|---|---|
| `speech_60s_portrait.mp4` | `1080,1920,1800` and duration `60.0...` |
| `speech_60s_landscape.mp4` | `1920,1080,1800`, `60.0...` |
| `speech_20s_noevents.mp4` | `1280,720,432`, `14.4...` |
| `speech_20s_long_pause.mp4` | `1280,720,600`, `20.0...` |
| `speech_30s_sparse.mp4` | `1280,720,900`, `30.0...` |

### Step 2.4 - Observe the machine-dependent facts (item 6)

| Fact | Do | Record |
|---|---|---|
| `ok_vfr.mp4` duration and rate | `ffprobe -v error -select_streams v:0 -show_entries stream=avg_frame_rate,nb_frames:format=duration -of csv=p=0 fixtures/.gen/ok_vfr.mp4` | Duration in ms and average fps x 1000 into the manifest; expect roughly 22,500 |
| Chrome decodes `rej_h264_444.mp4`? | `ffprobe -v error -select_streams v:0 -show_entries stream=profile,level -of csv=p=0 fixtures/.gen/rej_h264_444.mp4` (expect `High 4:4:4 Predictive,31`); in `pnpm dev`, DevTools console: `await VideoDecoder.isConfigSupported({ codec: "avc1.f4001f", codedWidth: 1280, codedHeight: 720 })` | `supported: false` keeps the fixture. `true`: switch to `rej_h264_hi10.mp4` (`-profile:v high10 -pix_fmt yuv420p10le`, codec `avc1.6e001f`) and test again; both `true`: D-78's `test.fixme` path |
| `rej_silent.mp4` gives fewer than 3 words | In `pnpm dev` (V2 app, model cached), drop it | The V2 rejected stub appears: keep. Accepted: shorten to 2 s (D-111), regenerate, try again |
| Rotation of `ok_rotated_90.mov`, the disabled first track | Leave `null` | Observed through the demuxer in Step 5.3 |

Regenerate after any manifest change (`pnpm fixtures`; the stamp forces it) and write Entry 2 of `v3changelog.md` with every observation, the encoders used and the generator's time.

### Step 2.5 - Write `fixtures/labeled/SCHEMA.md`

Content: §17.5's format and every row of its field table, including the four rows added in the review of 2026-10-15: `tags` (one entry per trap: `X4` twice for a file with two phone-like traps), `edits`, the null-data rule and `say` matching. Write the founder's answer to §23 item 27 into it (the `energy_z`/`pitch_z` a written emphasis gets) before Step 2.6.

### Step 2.6 - Write the 29 written files `w01` to `w29`

Follow the file plan of §17.5 row by row. Write the sentences first, then the `expected` list from what a person would expect from those sentences, never from what a rule would score. Mark emphasis with `*word*` only where the plan asks for a KeywordPop. Put each file's coverage in `"tags"`.

```bash
node -e '
const fs=require("fs"), d="fixtures/labeled", K=new Set(["number_reveal","list_reveal","from_to","keyword_pop"]);
const norm=s=>s.toLowerCase().replace(/\*/g,"").replace(/[^a-z0-9$%]+/g," ").trim();
let bad=0, n=0, empty=0; const kinds={}, tags={};
for (const f of fs.readdirSync(d).filter(x=>x.endsWith(".json"))) {
  n++; const j=JSON.parse(fs.readFileSync(`${d}/${f}`,"utf8"));
  if (j.id+".json"!==f) { bad++; console.log(f,"id mismatch"); }
  const text=j.sentences ? " "+norm(j.sentences.join(" "))+" " : null;
  if (!j.expected.length) empty++;
  for (const e of j.expected) {
    if (!K.has(e.kind)) { bad++; console.log(f,"kind",e.kind); }
    kinds[e.kind]=(kinds[e.kind]||0)+1;
    if (text && !text.includes(" "+norm(e.say)+" ")) { bad++; console.log(f,"say not found:",e.say); }
  }
  for (const t of j.tags||[]) tags[t]=(tags[t]||0)+1;
}
console.log(n,"files,",empty,"without events,",bad,"problems"); console.log(kinds); console.log(tags)'
```

After `w29`, expect: `29 files, 8 without events, 0 problems`; kinds `number_reveal 10, list_reveal 7, from_to 11, keyword_pop 9` (the file plan's real totals, item 3); tags `X1` to `X5` each 2 or more, `announcer 4`, `marker_only 3`, `from_frame 5`, `change_verb 6`, `no_change_pair 2`, `edit_number 1`, `hidden_word 1`.

### Step 2.7 - Write `a01`, `a02` and the README changes

| Order | File | Section | Watch for |
|---|---|---|---|
| 1 | `fixtures/labeled/a01-scriptA.json` | §17.5, item 3 | `source: "written"`, sentences copied from "The V2 reference clip as recognized" of the README, no `*` marks; expected: NumberReveal x2, FromTo (from-frame), ListReveal (announcer); `notes` says it is the recognized text written out |
| 2 | `fixtures/labeled/a02-v3testclip.json` | D-99, item 3 | `source: "asr"`, `words: null`, `prosody: null`, `sentences: null`; `expected` from `~/v3testclip-words.txt` (what a person expects from what was said); `notes` says "data filled in Step 12.3" |
| 3 | `fixtures/speech/README.md` | §17.7 | Rows of the five derived clips marked "derived in V3 from the reference clip (D-76)" with their cuts; the `v3testclip.mp4` row "not in git (D-99)"; Script A's V3 expected events and expected feed; new sections "The derived clips as recognized" and "The V3 test clip as recognized", the latter holding the founder's words now and "recognized transcript: filled in Step 9.6" |

Run the Step 2.6 command again. Expect `31 files, 8 without events, 0 problems` (a02's `say` values are not checked: no sentences), kinds at least `number_reveal 12, list_reveal 8, from_to 12, keyword_pop 9`, `announcer 5`, which meets every minimum of §17.5.

### Step 2.8 - Freeze the labels

```bash
git ls-files crates/offcut-detect/src | sort              # expect: lib.rs config.rs number.rs event_id.rs (V2) - no from_to/list/keyword/exclusions/resolve
git add fixtures/labeled fixtures/manifest.json fixtures/gen_fixtures.sh fixtures/speech/README.md package.json docs/v3
```

After the milestone commit, write its SHA into the next changelog entry as "labels frozen at <SHA> on <date>". From then on, a change under `fixtures/labeled/` needs a `v3changelog.md` line naming the file, the label and the reason (D-87).

### Milestone 2

```bash
rm -rf fixtures/.gen && pnpm fixtures                       # expect: exit 0
pnpm fixtures                                               # expect: fixtures are up to date
find fixtures/.gen -type f | wc -l                          # expect: 31
diff /tmp/gen1.txt <( cd fixtures/.gen && find . -type f ! -name STAMP | sort | xargs sha256sum ) && echo deterministic   # expect: deterministic (after any recipe change, regenerate /tmp/gen1.txt first)
node -e 'console.log(require("./fixtures/.gen/STAMP").generated)'   # expect: 25
# Step 2.6 command                                          # expect: 31 files, 8 without events, 0 problems
pnpm check                                                  # expect: exit 0
# frozen check (Step 1.7)                                   # expect: no output
git status --short | grep -E 'testclips|\.gen|\.clips|\.results'   # expect: no output
```

| Check | Where | Expect |
|---|---|---|
| A derived clip plays with white frames | Open `fixtures/.gen/speech/speech_60s_portrait.mp4` in any player | Portrait, 60 s, one white frame at 0:05 and 0:55 |
| The long-pause clip | Same, `speech_20s_long_pause.mp4` | Speech 0-5 s, room tone 5-10 s, speech 10-20 s |

Pass: 25 fixtures and 5 PCM files are generated byte-identically twice, none carries a location or creation time, the manifest holds every observed value, and 31 labeled files are committed while `offcut-detect/src` still has only V2's four files.

Commit: `"V3 S2: fixtures, manifest and the frozen labeled set"`

Do not start Phase 3 until this passes.

---

## Phase 3 - Voice chain and prosody, native (S4)

Goal: `offcut-dsp` holds the full chain and prosody, length-preserving and on target in native tests, with `DENOISE_DELAY_SAMPLES` measured and the native TE-13 time recorded.

### Step 3.1 - Constants, module list and the shared fixture helper

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-dsp/src/config.rs` | §7.2, Appendix D | 1 inline test: `ChainConfig::default()` equals TS §18.3's values. `Lufs::new`/`Dbfs::new` return `Option`: build the default without `unwrap` (§7.2's const helper) |
| 2 | `crates/offcut-dsp/src/lib.rs` | §7.1 | Add one `pub mod` line as each file lands; `DspError` is V2's, unchanged |
| 3 | The fixture helper | §6.4, item 1 | 25 lines, copied into each of the four DSP test files of this phase: directory, stamp check against the manifest's SHA-256 (`sha2`), the panic message "the generated fixtures are stale or missing: run `pnpm fixtures`" |

```bash
grep -cE '^pub const [A-Z_0-9]+: ' crates/offcut-dsp/src/config.rs   # expect: 32 (the constants of §7.2: 23 for the chain, 9 for prosody)
cargo test -p offcut-dsp --lib config                        # expect: 1 passed
```

If the count differs, compare with §7.2 line by line; Appendix D lists the same names.

### Step 3.2 - High-pass and compressor

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `src/highpass.rs` | §7.3 | 6 inline cases: block length, cutoff 0 or above Nyquist passes through, DC decays, 1 kHz within 0.05 dB, 40 Hz down 10 dB, impulse response peaks at sample 0. `indexing_slicing` is denied: use `iter_mut` |
| 2 | `src/compressor.rs` | §7.5 | §7.5 gives no case list; write at least: under the knee unchanged, gain never above 1, length unchanged. f64 state, f32 samples |

```bash
cargo test -p offcut-dsp --lib highpass compressor           # expect: 9 or more passed
```

### Step 3.3 - Loudness and limiter

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `src/loudness.rs` | §7.6 | 5 inline cases of §7.6. Feed `ebur128` in `BLOCK_SAMPLES` chunks; channel `Center` |
| 2 | `src/limiter.rs` | §7.7 | 4 situations of §7.7. **`limit` returns the number of samples the guard clamp touched (`usize`)**, as §7.7 says: `loudness_target.rs` asserts "touched 0 samples". `chain.rs` ignores the value |

```bash
cargo test -p offcut-dsp --lib loudness limiter              # expect: 9 or more passed
```

### Step 3.4 - Denoise

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `src/denoise.rs` | §7.4 | 3 inline cases now (total length over many calls plus `finish`; silence stays silent; `mix` 0.0 gives the input delayed and compensated, sample-exact). The impulse case is `impulses_stay_aligned` of Step 3.5. Scale by `DENOISE_SCALE` in and out; discard the VAD value |

```bash
cargo test -p offcut-dsp --lib denoise                       # expect: 3 passed
```

### Step 3.5 - The chain, clicks and duration; measure the denoiser's delay

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `src/chain.rs` | §7.8, TS §34.3 | The inline cases of §7.10 that concern `run_chain` (4). No release-time pad or trim branch: a length mismatch is a bug of `denoise.rs` |
| 2 | `tests/chain_no_clicks.rs` | §19.5 | 7 cases (`chain_timing` `#[ignore]`); the fixture helper |
| 3 | `tests/duration_preserved.rs` | §19.6 | 7 cases with TS §34.3's names; the fixture helper |

```bash
cargo test -p offcut-dsp --test duration_preserved -- --nocapture impulses_stay_aligned
# first run: it may fail and print the measured offset
```

Set `DENOISE_DELAY_SAMPLES` in `config.rs` to the offset the test prints, re-run until it passes, and write the value to `docs/v3/experiments.md` under `DENOISE_DELAY_SAMPLES`. If no constant makes it pass (the offset varies), the denoiser is not delay-stable: set `denoise: false` in `ChainConfig::default()` (TS §18 contingency) and record it.

```bash
cargo test -p offcut-dsp --test chain_no_clicks --test duration_preserved   # expect: 13 passed, 1 ignored
```

### Step 3.6 - Loudness on target

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `tests/loudness_target.rs` | §19.4 | 10 cases; the fixture helper; the guard-clamp count from Step 3.3 is 0 in every case |

```bash
cargo test -p offcut-dsp --test loudness_target              # expect: 10 passed
```

### Step 3.7 - Prosody

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `src/prosody.rs` | §7.9, D-96, D-108 | The prosody inline cases of §7.10 (5). Decimate before the autocorrelation; median of voiced frames, lower middle for an even count |
| 2 | `tests/prosody.rs` | §19.7 | 11 cases; the speech case prints and asserts nothing; the budget case asserts under 2 s in release only (`#[cfg(not(debug_assertions))]`); the fixture helper |

```bash
cargo test -p offcut-dsp --test prosody -- --nocapture      # expect: 11 passed; the printed z-scores of the "brutal" word
```

Copy the printed "brutal" reading into `experiments.md`: it is the measured input of the conditional KeywordPop (§17.7).

### Step 3.8 - TE-13, native

```bash
cargo test -p offcut-dsp --release --test chain_no_clicks -- --ignored --nocapture chain_timing
# run five times; take the median of the printed times for 60 s of audio
```

| Median | Do |
|---|---|
| Under 2 s | Record and continue |
| 2 s or more | **Stop.** Look for per-sample work that belongs per block (allocations, `exp` or `log10` per sample, `ebur128` fed sample by sample). If it stays at 2 s or more, take TE-13's fallback now (section 20.2: `denoise: false`) and record it before Phase 4 |

### Step 3.9 - Prove the guards

| Temporary edit | Rule that must fire |
|---|---|
| `DENOISE_DELAY_SAMPLES = 0` | `impulses_stay_aligned` fails |
| In `chain.rs`, skip the last partial block | `prop_length_equals_input` fails with a shrunk length |
| Change one byte of `fixtures/.gen/STAMP` | Every fixture-reading DSP test panics "run `pnpm fixtures`" |
| `let x = v[0];` in `highpass.rs` | `cargo clippy -p offcut-dsp -- -D warnings` fails on `indexing_slicing` |

Undo all four; run `pnpm fixtures` (the stamp is restored).

### Milestone 3

```bash
pnpm fixtures                                               # expect: fixtures are up to date
cargo test -p offcut-dsp 2>&1 | awk '/^test result:/ {p+=$4; f+=$6; i+=$8} END {print p" passed, "f" failed, "i" ignored"}'
# expect: 0 failed, 1 ignored (chain_timing), passed = the V2 count of the crate + the cases of Steps 3.1-3.7
cargo clippy -p offcut-dsp --all-targets -- -D warnings     # expect: exit 0
cargo build -p offcut-dsp --target wasm32-unknown-unknown --release   # expect: Finished
git grep -niE 'silence|pause|gap' -- crates/offcut-dsp/src  # expect: only comments saying what the crate does not do, ABSOLUTE_SILENCE, and the z-score code of prosody.rs
grep -n "DENOISE_DELAY_SAMPLES" crates/offcut-dsp/src/config.rs   # expect: the measured value, without "(assumption"
pnpm check                                                  # expect: exit 0
# frozen check (Step 1.7)                                   # expect: no output
```

Pass: every `offcut-dsp` test passes, the delay constant holds a measured value, the native TE-13 median is under 2 s (or the fallback is recorded), and the WASM build succeeds.

Commit: `"V3 S4: voice chain and prosody (native)"`

Do not start Phase 4 until this passes.

---

## Phase 4 - Audio worker in the browser: TE-13 (S6)

Goal: the chain runs in the new `audio.worker` on D1 under the production build constraints, returns `out48` of exactly the input's length at -14 LUFS ±1, and its time for 60 s of audio is at most 4 s.

### Step 4.1 - The binding

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-wasm-core/src/audio_api.rs` | §10.1, D-110 | No test file (TS §5 has none): covered by Steps 4.5 and Phase 11. `out48` must be a new `Float32Array` with its own buffer (so the worker can transfer it); a throwing callback becomes `E_INTERNAL` `Callback` |
| 2 | `crates/offcut-wasm-core/src/lib.rs` | §10.1 | `mod audio_api;` and nothing else |

```bash
pnpm build:wasm                                             # expect: exit 0
cargo clippy -p offcut-wasm-core --target wasm32-unknown-unknown -- -D warnings   # expect: exit 0
```

### Step 4.2 - The TypeScript side

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `web/src/wasm/load-core.ts` | §13.2 | `ChainResult`, `runChain`, `measureProsody`; one cast `as ChainResult` of the decoded object, no `as <Brand>` of a primitive |
| 2 | `web/src/workers/audio.worker.ts` | §14.1, Appendix C | `kept` set to `null` first in `runChain`; `measureProsody` with nothing kept throws `WorkerFailure("E_INTERNAL", "NoAudio")`; about 60 lines |
| 3 | `web/src/workers/pool.ts` | §13.4 | The import line with `?worker&url`, one `WORKERS` row (stage `audio_chain`), one `lazyClient`; `preload()` now adds 4 links |

```bash
pnpm check                                                  # expect: exit 0
grep -c "modulepreload" web/src/workers/pool.ts              # inspect: preload covers four worker scripts
```

### Step 4.3 - Measure TE-13 in the browser (no code change)

Start the dev server and open `/app`. In the DevTools console:

```js
const { pool } = await import("/src/workers/pool.ts");
const pick = Object.assign(document.createElement("input"), { type: "file" }); document.body.prepend(pick);
// click the file input that appeared at the top of the page and choose
// fixtures/.gen/speech/speech_60s_portrait.mp4.pcm48.f32, then run:
const pcm48 = new Float32Array(await pick.files[0].arrayBuffer()); const n = pcm48.length;
try { await pool.audio.measureProsody({ words: [], sentences: [] }); } catch (e) { console.log("expected:", e); }
const t0 = performance.now();
const r = await pool.audio.runChain({ pcm48 }, { transfer: [pcm48.buffer] });
const ms = Math.round(performance.now() - t0);
const p = await pool.audio.measureProsody({ words: [], sentences: [] });
console.log({ ms, loudness: r.loudness, n, same: p.out48.length === n, detached: pcm48.byteLength === 0 });
```

| Do | Expect |
|---|---|
| The first `measureProsody` | `expected:` with code `E_INTERNAL`, detail `NoAudio` |
| The `console.log` | `same: true`, `detached: true`, `n: 2880000`, `loudness` between -15 and -13 |
| `ms` | At most 4000 |

If `pool` is not the exported name, open `web/src/workers/pool.ts` and use the exported client object. Run the measurement three times and record the median under `TE-13 browser` in `experiments.md` (D1, `pnpm dev`; Step 12.1 re-reads it in the production build).

| Median for 60 s | Do |
|---|---|
| At most 4,000 ms | Continue |
| Over 4,000 ms | Set `denoise: false` in `ChainConfig::default()` (§20.2), re-run `cargo test -p offcut-dsp` and this step, record both readings |

### Step 4.4 - Check the production build and the host allowlist

```bash
pnpm build                                                  # expect: exit 0, including check-hosts.mjs
ls -l web/dist/assets/ | grep -E 'offcut_core_bg-.*\.wasm'  # record the new size; growth over 2 MB: find which crate (§20.1)
```

If `check-hosts.mjs` reports a URL literal inside `offcut_core_bg-*.wasm`, add it to `NON_NETWORK_LITERALS` with `chunk` scoped to that file and a reason naming the crate (§18.7, D-38). Never allow it globally.

### Step 4.5 - Verify under production headers

```bash
pnpm --filter web exec vite preview --port 4173 &            # serves web/dist; the preview config must send COOP/COEP as V2 does
```

Open `http://localhost:4173/app`, confirm `crossOriginIsolated` is `true` in the console, and repeat the measurement of Step 4.3 once. Expect the same `same: true` and a time within 20% of Step 4.3's. Stop the preview server (`kill %1`).

If `vite preview` does not send the V2 headers, use the way V2 checked the production build (its changelog) instead; do not add headers to any config in V3.

### Milestone 4

```bash
pnpm build:wasm                                             # expect: exit 0
pnpm check                                                  # expect: exit 0
pnpm build                                                  # expect: exit 0 (check-hosts passes)
cargo test --workspace 2>&1 | awk '/^test result:/ {f+=$6} END {print f" failed"}'   # expect: 0 failed
pnpm e2e:media --grep @no-gpu 2>&1 | tail -2                # expect: 8 passed (V2's no-GPU cases still pass with the fourth worker)
# frozen check (Step 1.7)                                   # expect: no output
```

| Check | Where | Expect |
|---|---|---|
| TE-13 browser | `docs/v3/experiments.md` | A median in ms for 60 s, the decision, the date, label `d1` |
| Core bundle size | `v3changelog.md` | Before and after in bytes |

Pass: `out48` has exactly the input's length, loudness is -14 ±1 LUFS, the browser median is at most 4,000 ms per 60 s (or `denoise: false` is set and recorded), and the production build passes `check-hosts.mjs`.

Commit: `"V3 S6: audio worker and the voice chain in the browser"`

Do not start Phase 5 until this passes.

---

## Phase 5 - Demuxer hardening (S3)

Goal: `offcut-mp4` accepts the five edit-list shapes, carries the probe hardening H1-H8, and every generated fixture is accepted or refused with its manifest code in native tests.

### Step 5.1 - `edit_list.rs`

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-mp4/src/edit_list.rs` | §6.1 | 12 inline cases of §6.1; `MAX_EDITS = 16`; checked additions; a tick of mismatch accepted, two refused |
| 2 | `crates/offcut-mp4/src/lib.rs` | §4 | `mod edit_list;` (private) |
| 3 | `crates/offcut-mp4/src/sample_table.rs` | §6.2 | `edits()` keeps its signature and calls `resolve_edits`; `to_media` and `shown` move out; V2's two inline edit-list tests pass unchanged |
| 4 | `docs/technicalspec.md` §5 | Step 1.3 | Only if Step 1.3 had to defer the `edit_list.rs` line |

```bash
cargo test -p offcut-mp4 --lib                              # expect: V2's count + 12, 0 failed
awk '/#\[cfg\(test\)\]/{print NR-1; exit}' crates/offcut-mp4/src/sample_table.rs   # expect: under 400
```

### Step 5.2 - `probe.rs` H1-H8

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-mp4/src/probe.rs` | §6.3 | An inline case for H1 (matrix rounding: off by one unit gives R90/R270; off by more than 1/256 gives R0), H3 (last sample duration of 1 tick ignored), H4 (frequency index 15); H2, H5-H8 are covered by the fixture tests. `validate.rs` is not edited |

```bash
cargo test -p offcut-mp4 --lib probe                        # expect: all passed
git diff --name-only 8a271a9 -- crates/offcut-mp4/src/validate.rs   # expect: no output
```

### Step 5.3 - `demux_fixtures.rs`, and fill the observed values

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-mp4/tests/demux_fixtures.rs` | §19.2 | 12 cases; the fixture helper; read `expect.accept` from `fixtures/manifest.json` with a small `serde_json` dev struct |

Run once with printing before asserting the observed values:

```bash
cargo test -p offcut-mp4 --test demux_fixtures -- --nocapture 2>&1 | grep -E "rotation|track|ok_vfr"
```

| Observation | Do |
|---|---|
| Rotation of `ok_rotated_90.mov` | Write `R90` or `R270` (whichever the demuxer reports; both give display 720x1280) into the manifest |
| The video track of `ok_two_video_first_disabled.mp4` is 640x360 | Keep the entry. Otherwise remove it from the manifest (D-77 as written in §17.2) and keep H2 as an inline test only |

Each manifest edit changes the stamp: run `pnpm fixtures`, then the test again. Record both observations in `v3changelog.md`.

```bash
pnpm fixtures && cargo test -p offcut-mp4 --test demux_fixtures   # expect: 12 passed (11 if the two-video entry was removed)
```

### Step 5.4 - `probe_rejections.rs`

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-mp4/tests/probe_rejections.rs` | §19.3 | 15 cases; the mapping of `ContainerError` to a code is reproduced from `media_api.rs` in one private function; the manifest-coverage case reads every `rej_*` entry except `rej_silent.mp4` (decided by ASR) |

```bash
cargo test -p offcut-mp4 --test probe_rejections            # expect: 15 passed
```

### Step 5.5 - Prove the guards

| Temporary edit | Rule that must fire |
|---|---|
| Remove the `rej_mp3_audio.mp4` row from the test's mapping table | The manifest-coverage case fails naming `rej_mp3_audio.mp4` |
| Set `MAX_EDITS` to 2 | The E3 inline case (three edits) fails |
| Move `fixtures/.gen` aside (`mv fixtures/.gen /tmp/gen.bak`) | `demux_fixtures.rs` panics "run `pnpm fixtures`" |

Undo all three (`mv /tmp/gen.bak fixtures/.gen`).

### Milestone 5

```bash
pnpm fixtures                                               # expect: fixtures are up to date
cargo test -p offcut-mp4 2>&1 | awk '/^test result:/ {p+=$4; f+=$6} END {print p" passed, "f" failed"}'   # expect: 0 failed
cargo clippy -p offcut-mp4 --all-targets -- -D warnings     # expect: exit 0
for f in crates/offcut-mp4/src/edit_list.rs crates/offcut-mp4/src/sample_table.rs crates/offcut-mp4/src/probe.rs; do
  printf '%s ' "$f"; awk '/#\[cfg\(test\)\]/{print NR-1; f=1; exit} END{if(!f) print NR}' "$f"; done   # expect: every number under 400
pnpm check                                                  # expect: exit 0
# frozen check (Step 1.7)                                   # expect: no output
```

Pass: every `ok_*` fixture and every derived speech clip opens and validates, every `rej_*` fixture except `rej_silent.mp4` gives its manifest code, the observed rotation is in the manifest, and no source file reaches 400 lines.

Commit: `"V3 S3: demuxer edit lists, probe hardening, fixture tests"`

Do not start Phase 6 until this passes.

---

## Phase 6 - Number grammar and sentences (S5)

Goal: `offcut-text` parses and formats every row of §19.8, splits long sentences, and keeps every quantity inside one sentence, with V2's tests unchanged except the one §8.6 names.

### Step 6.1 - Word lists

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-text/src/units_lex.rs` | §8.4, D-101, D-102 | Inline: every slice sorted, unique, lower-case (one test per slice). "pound", "pounds", "tb", "terabytes" are absent on purpose |
| 2 | `crates/offcut-text/src/lib.rs` | §4 | `pub mod units_lex;` (and `mod cardinals;` in Step 6.3) |

```bash
grep -cE '^pub const [A-Z_]+' crates/offcut-text/src/units_lex.rs   # expect: 9 (the 8 lists of §8.4 + COUNT_UNIT_MIN)
cargo test -p offcut-text --lib units_lex                    # expect: all passed
```

### Step 6.2 - Wider `Digits`

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-text/src/tokenize.rs` | §8.1 | The table of §8.1 as inline cases (10); `Token` and `TokenKind` do not change; Indian grouping, `10:30`, `5-10` stay `Word` |

### Step 6.3 - Spelled cardinals

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-text/src/cardinals.rs` | §8.2 | The 13 rows of §8.2 as inline cases; magnitudes strictly descending, each at most once; "and" consumed only before 1-99 |
| 2 | `docs/technicalspec.md` §5 | Step 1.3 | Only if Step 1.3 deferred the line |

### Step 6.4 - `parse_quantity`, `format_quantity`

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-text/src/numbers.rs` | §8.3, §8.5 | The inline cases of §8.7 that concern it (`format_quantity` of -5 and NaN). **Rewrite exactly one V2 inline test**: "ten million" now parses as 10,000,000 using 2 tokens (§8.6). `numbers.rs` remains the only formatter |

```bash
git diff 8a271a9 -- crates/offcut-text/src | grep -cE '^-\s*assert'   # expect: 1 (only the "ten million" expectation removed)
```

### Step 6.5 - Sentences and per-sentence numbers

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-text/src/sentences.rs` | §8.6 | Split at the longest internal gap, earliest of equals, recursively |
| 2 | `crates/offcut-text/src/normalize.rs` | §8.6, D-104 | Tokenize and scan per sentence; the "$12 / ,000" sentence-boundary case of §8.7 |

### Step 6.6 - The three test files

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-text/tests/numbers_table.rs` | §19.8 | 62 parse rows and 10 display rows as two data tables, one test each, printing the failing row |
| 2 | `crates/offcut-text/tests/numbers_prop.rs` | §19.9 | 5 properties; the speller is test-only |
| 3 | `crates/offcut-text/tests/sentences.rs` | §19.10 | 8 cases |

```bash
grep -cE '^\s*\("' crates/offcut-text/tests/numbers_table.rs   # expect: 72 (62 + 10 rows, if each row is a tuple on one line)
cargo test -p offcut-text                                   # expect: 0 failed
```

### Step 6.7 - Prove the guards

| Temporary edit | Rule that must fire |
|---|---|
| Swap two adjacent words of `COUNT_NOUNS` | The "sorted" inline test fails |
| Add `"pounds"` to `CURRENCY_WORDS` (sorted) | `numbers_table.rs` row `ten pounds` fails |
| Make `normalize.rs` scan the whole transcript instead of each sentence | The `$12 / ,000` boundary case fails |

Undo all three.

### Milestone 6

```bash
cargo test -p offcut-text 2>&1 | awk '/^test result:/ {p+=$4; f+=$6} END {print p" passed, "f" failed"}'   # expect: 0 failed
cargo clippy -p offcut-text --all-targets -- -D warnings    # expect: exit 0
awk '/#\[cfg\(test\)\]/{print NR-1; f=1; exit} END{if(!f) print NR}' crates/offcut-text/src/numbers.rs   # expect: under 400
pnpm build:wasm                                             # expect: exit 0 (text_api.rs is unchanged but rebuilds)
pnpm check                                                  # expect: exit 0
# frozen check (Step 1.7)                                   # expect: no output
```

Pass: all 72 rows of `numbers_table.rs`, the 5 properties and the 8 sentence cases pass, exactly one V2 assertion changed, and `numbers.rs` stays under 400 lines.

Commit: `"V3 S5: complete number grammar, units and the sentence split"`

Do not start Phase 7 until this passes.

---

## Phase 7 - Detector rules (S7, part 1)

Goal: `offcut-detect` has every finder, the exclusions, resolution and the full `detect`, and the 34 rows of `rules_table.rs` pass, all written after the labels were frozen.

### Step 7.1 - Confirm the labels came first

```bash
L=$(git log --format=%ct -1 -- fixtures/labeled/w01-saas-pricing.json)
git ls-files crates/offcut-detect/src | grep -E 'from_to|list|keyword|exclusions|resolve' ; echo "none yet"   # expect: none yet
git status --short fixtures/labeled                          # expect: no output
echo "labels frozen at $L"
```

### Step 7.2 - Configuration and lexicons

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-detect/src/config.rs` | §9.1, D-84 | V2's 10 fields keep names and values; 29 fields appended; `threshold(kind)`, `hold_ms(kind)`; 11 lexicons. Inline: defaults, the four thresholds, hold 1,400 / 1,700 / 1,700 / 0, each lexicon sorted, unique, lower-case |

```bash
grep -cE '^\s+pub [a-z_]+:' crates/offcut-detect/src/config.rs    # expect: 39
grep -cE '^pub const [A-Z_]+: &\[' crates/offcut-detect/src/config.rs   # expect: 11
cargo test -p offcut-detect --lib config                    # expect: all passed
```

### Step 7.3 - Write the finders, adding their `rules_table.rs` rows with each

Create `crates/offcut-detect/tests/rules_table.rs` now with the timing-synthesis helper (40 lines, the same rule as §17.5) and add rows as each finder lands; the file passes in full at Step 7.5.

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `src/number.rs` | §9.3, D-106, D-107, D-109 | `Candidate.parts`, `Found`, `quantities`, labels. **Rewrite V2's "edited number" test**: an edit to `$20,000` now gives an event of `$20k` with a new id; the hidden half still gives none (§9.9). Rows 1-7 and 32 of §19.11 |
| 2 | `src/list.rs` | §9.5, D-105 | `markers` is `pub(crate)` (read by `exclusions.rs`); items stop at `,` only after 2 words. Rows 13-18 |
| 3 | `src/from_to.rs` | §9.4, D-85 | Best `b` per `a`; inherited unit re-formatted with `format_quantity`. Rows 8-12 |
| 4 | `src/keyword.rs` | §9.6 | Skip words of quantities and markers. Rows 19-21 |
| 5 | `src/exclusions.rs` | §9.7 | X1-X7; the announcer count exempt from X5. Rows 22-26, 33 |

```bash
cargo build -p offcut-detect                                 # expect: Finished, after each file
```

### Step 7.4 - Resolution and `detect`

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `src/resolve.rs` | §9.8, Appendix C | Total order of step 0; windows that touch count as overlapping (V2 as built); fixed events never dropped. Rows 27-31 |
| 2 | `src/lib.rs` | §9.2 | `detect` in the seven steps of §9.2; output sorted by `span.start` then kind order. Row 34 (empty transcript) and the remaining inline cases of §9.9 |

### Step 7.5 - Run the table

```bash
cargo test -p offcut-detect --test rules_table -- --nocapture   # expect: 34 passed (or one test with 34 rows, printing none failed)
cargo test -p offcut-detect --lib                           # expect: 0 failed
```

A failing row is fixed in the rule, never by editing a label or the row's expectation without a changelog line naming the spec section that decides it.

### Step 7.6 - Prove the guards

| Temporary edit | Rule that must fire |
|---|---|
| Comment out X1 in `exclusions.rs` | Row 11 (`From 2019 to 2024`) fails |
| Make `resolve` keep candidates whose windows touch | Row 10 (`$2k to $20k`) fails: both numbers kept |
| Lower `threshold_list` to 0.60 | Row 17 (marker-only with a 20 s gap) fails |

Undo all three.

### Milestone 7

```bash
cargo test -p offcut-detect --lib --test rules_table 2>&1 | awk '/^test result:/ {f+=$6} END {print f" failed"}'   # expect: 0 failed
cargo clippy -p offcut-detect --all-targets -- -D warnings  # expect: exit 0
for f in crates/offcut-detect/src/*.rs; do printf '%s ' "$f"; awk '/#\[cfg\(test\)\]/{print NR-1; f=1; exit} END{if(!f) print NR}' "$f"; done   # expect: every number under 400
git log --format=%ct -1 -- fixtures/labeled                  # expect: a time earlier than any commit touching crates/offcut-detect/src/from_to.rs (none committed yet)
git status --short fixtures/labeled                          # expect: no output
pnpm check                                                  # expect: exit 0
# frozen check (Step 1.7)                                   # expect: no output
```

Pass: all 34 rows of `rules_table.rs` and every inline test of `offcut-detect` pass, no label changed, and no detector file reaches 400 lines.

Commit: `"V3 S7: detector finders, exclusions and resolution (part 1)"`

Do not start Phase 8 until this passes.

---

## Phase 8 - Re-detection, determinism and the precision gate (S7, part 2)

Goal: `redetect_sentence` exists in Rust and in the render bundle, detection is deterministic, and precision over the labeled set is at least 0.90 with the result recorded.

### Step 8.1 - `redetect_sentence`

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-detect/src/lib.rs` | §9.2, D-86 | The five steps; `s` out of range returns `prev` unchanged |
| 2 | `crates/offcut-detect/tests/redetect.rs` | §19.13 | 7 cases on Script A as recognized (the sentences of `a01-scriptA.json` with synthesized timing); the timing case prints only |

```bash
cargo test -p offcut-detect --release --test redetect -- --nocapture   # expect: 7 passed; printed microseconds per call (record it)
```

### Step 8.2 - Determinism

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-detect/tests/determinism.rs` | §19.14 | 3 cases; the proptest vocabulary mixes number words, list nouns, ordinals, fillers, stop words, emphasis words, digits |

```bash
cargo test -p offcut-detect --test determinism              # expect: 3 passed
```

### Step 8.3 - The precision gate

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-detect/tests/precision_recall.rs` | §19.12, Step 2.5 | Reads `fixtures/labeled/*.json` with `deny_unknown_fields` (`tags` and `edits` optional); skips an `asr` file with `words: null` and prints `skipped (no data yet)`; coverage check from `tags`; prints the per-kind table, the sweep and the false positives; three assertions |

```bash
cargo test -p offcut-detect --test precision_recall -- --nocapture 2>&1 | tee /tmp/precision.txt | tail -40
# expect: "30 files read, 1 skipped"; overall precision >= 0.90; test result: ok. 1 passed
```

| Reading | Do (§20.3) |
|---|---|
| Overall precision 0.90 or more | Copy the printed table, sweep and false positives into `experiments.md` under `Precision gate`, with "default thresholds" |
| Under 0.90 | For each kind under 0.90, set its threshold in `config.rs` to the lowest value of the printed sweep at which it reaches 0.90; re-run; record old and new thresholds and every kind's recall. **Never edit a label to pass** |
| A kind's recall under 0.30 after the raise | Keep the kind; record it as the E-7 input "tune rules or reconsider the wedge" |
| A kind under 0.90 at every threshold | Set its threshold to 1.01 (no event can reach it, INV-6); record; V4 draws three kinds (§20.4) |
| A label is wrong (a person would not expect it) | Fix it with a `v3changelog.md` line naming the file, the label and the reason |

Any threshold changed here is also written into TS §17.4 in Step 13.1; note it in `v3changelog.md` now.

### Step 8.4 - The binding and `RenderApi`

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `crates/offcut-wasm-render/src/detect_api.rs` | §10.2 | `redetect_sentence` export; detail values `Transcript`, `Prosody`, `Edits`, `Events`; nothing calls it in V3 |
| 2 | `web/src/wasm/load-render.ts` | §13.3 | `redetectSentence` on `RenderApi` |

```bash
pnpm build:wasm                                             # expect: exit 0
pnpm check                                                  # expect: exit 0
git grep -n "redetectSentence" -- web/src                   # expect: load-render.ts, protocol.ts, and render.worker.ts's V2 line `redetectSentence: notYet,` unchanged (it still answers E_INTERNAL until V4)
```

### Step 8.5 - Prove the guards

| Temporary edit | Rule that must fire |
|---|---|
| `threshold_number = 0.50` | `precision_recall.rs` fails below 0.90 and lists the new false positives |
| Make `resolve` sort by score only (drop the tie-breakers) | `determinism.rs` proptest finds two calls that differ, or a test of event order fails |
| Edit one `expected` value in a labeled file without a changelog line | Not caught by a test: the review rule of D-87. Confirm `git status --short fixtures/labeled` shows the file, then undo |

Undo all three.

### Milestone 8

```bash
pnpm fixtures                                               # expect: fixtures are up to date
cargo test -p offcut-detect 2>&1 | awk '/^test result:/ {p+=$4; f+=$6} END {print p" passed, "f" failed"}'   # expect: 0 failed
cargo test -p offcut-detect --test precision_recall -- --nocapture 2>&1 | grep -iE "overall|files read"   # expect: precision >= 0.90; 30 files read
cargo test --workspace 2>&1 | awk '/^test result:/ {p+=$4; f+=$6; i+=$8} END {print p" passed, "f" failed, "i" ignored"}'   # expect: 0 failed; passed above 262
pnpm build:wasm && pnpm check                               # expect: exit 0
git diff --stat "$(git log --format=%H -1 -- fixtures/labeled)" -- fixtures/labeled   # expect: no output, or only changes with a changelog line
# frozen check (Step 1.7)                                   # expect: no output
```

Pass: precision is at least 0.90 over at least 30 read files, every predicted event is at or above its threshold, re-detection keeps ids of unchanged events, detection is deterministic, and the gate's table is in `experiments.md`.

Commit: `"V3 S7: re-detection, determinism and the precision gate (part 2)"`

Do not start Phase 9 until this passes.

---

## Phase 9 - Records and the full pipeline (S8)

Goal: an accepted clip in `pnpm dev` runs the voice chain in parallel with transcription, is measured, detected with all four kinds, and leaves three IndexedDB records, with `REJECT_NO_SPEECH` reported as a rejection event.

### Step 9.1 - `clips-repo.ts`

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `web/src/persistence/clips-repo.ts` | §13.5, D-81, D-103 | `hashPrefix`, `put`, `get`, `toBytes`; `HASH_PREFIX_BYTES = 4,194,304`; size appended as u64 little-endian (`DataView.setBigUint64(0, BigInt(size), true)`); `QuotaExceededError` to `OpfsError("quota")`, any other to `OpfsError("io")`; a newer `schemaVersion` is deleted. No Vitest file (TS §5 lists none; covered by the accept case of Step 11.2). If the founder chose a `clear()` per repo for §23 item 26, add it to all three repos and call it from `import-clip.ts` step 4 beside the D-40 sweep, and write the decision into §13.5 and §15.1 |

Prove the minting rule:

| Temporary edit | Rule that must fire |
|---|---|
| In `transcript-repo.ts` (once it exists), write `const b = 5 as Bytes;` | `pnpm check` fails on the `MINTING` rule (D-59, D-103) |

### Step 9.2 - The other two repos

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `web/src/persistence/transcript-repo.ts` | §13.5 | `put`, `get`; value `{ transcript, prosody }` |
| 2 | `web/src/persistence/edits-repo.ts` | §13.5 | `put`, `get`; value `{ editState }` |

```bash
pnpm check                                                  # expect: exit 0 (after undoing the minting probe)
```

### Step 9.3 - `import-clip.ts`

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `web/src/usecases/import-clip.ts` | §15.1, D-97 | `hashPrefix` before `importAndProbe` (the file is transferred after); `clip_rejected { reject_reason }` and nothing else; `clipsRepo.put` with `sourceBytes: toBytes(file.size)`; a failed `put` removes the clip folder; the file name is never read |

### Step 9.4 - `run-pipeline.ts`

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `web/src/usecases/run-pipeline.ts` | §15.2, D-90 | Remove the two `V3:` stand-ins and the line `out48 = owned(audio.pcm48)`; `settle` never rejects; `pcm48` transferred to the audio worker; `await chain` before any failure is stored and before `noSpeech()`; the earlier failure in time wins; `unload()` before `openSession` (INV-12); step F writes OPFS, then the two repos; `displayToOf` for `from_to` |

```bash
wc -l web/src/usecases/run-pipeline.ts                      # expect: under 400 (about 352)
git grep -n "V3: " -- web/src crates                        # expect: no output
git grep -n "V4: " -- web/src                               # expect: the places V4 fills (render.worker handlers, cancel and retry)
pnpm check                                                  # expect: exit 0
```

### Step 9.5 - Run the reference clip in `pnpm dev`

Start from an empty IndexedDB without losing the model: in DevTools, Application, IndexedDB, delete the database `offcut` only. "Clear site data" would also delete the model files in OPFS and force a 215 MB download.

| Do | Expect |
|---|---|
| Drop `testclips/speech_scriptA_landscape_720p.mp4` | Feed: "Cleaning voice" and "Transcribing…" as the first two lines in either order |
| Wait for `ready` | Feed then shows, in time order: `Found: 3,000 hours`, optionally `Found: brutal`, `Found: 90 minutes to 10 minutes`, `Found: 3-item list`, `Found: $12k` |
| Play the preview | The voice sounds cleaner than the source; the number animates; nothing else is drawn |
| DevTools, Network, the `/api/v1/events` request bodies | `stage_timing` with `stage` values `probe_audio`, `audio_chain`, `asr`, `detect_scene`; `pipeline_done` with `n_number: 2, n_list: 1, n_from_to: 1, n_keyword: 0` or `1`; no word of the transcript in any body |
| DevTools, Application, IndexedDB `offcut` | One record each in `clips`, `transcripts`, `edits`; `clips` has `sourceBytes` and a 64-hex `sourceHashPrefix` and no file name; `transcripts` has `prosody.per_word` as long as `transcript.words` |
| DevTools console: OPFS size of `out48.f32` (V2's helper, or `(await (await (await (await navigator.storage.getDirectory()).getDirectoryHandle("clips")).getDirectoryHandle("<clipId>")).getFileHandle("out48.f32")).getFile()).size`) | Equal to 4 x `round(74,705 x 48)` = 14,343,360 bytes |

### Step 9.6 - Fill the "as recognized" sections of the README

For each derived clip and for `testclips/v3testclip.mp4`, drop it in `pnpm dev`, wait for `ready`, then copy its transcript out of IndexedDB in the console:

```js
const db = await new Promise((ok, no) => { const r = indexedDB.open("offcut"); r.onsuccess = () => ok(r.result); r.onerror = () => no(r.error); });
const req = (r) => new Promise((ok, no) => { r.onsuccess = () => ok(r.result); r.onerror = () => no(r.error); });
const latest = (await req(db.transaction("clips").objectStore("clips").getAll()))
  .map((x) => x.value).sort((a, b) => b.createdAt - a.createdAt)[0];
const t = (await req(db.transaction("transcripts").objectStore("transcripts").get(latest.clipId))).value.transcript;
copy(t.sentences.map((s, i) => `${i + 1} | ${s.start_ms}-${s.end_ms} | ` +
  t.words.slice(s.words.start, s.words.end).map((w) => w.text).join(" ")).join("\n"));
```

The transcript is read by the newest clip's `clipId`, so earlier clips in the store do not matter and the model stays cached. Use the field and key names of `web/src/gen/domain.ts` and `persistence/schema.ts` if they differ.

| Clip | Write into `fixtures/speech/README.md` | Expect |
|---|---|---|
| `speech_60s_portrait.mp4`, `speech_60s_landscape.mp4` | Sentences, feed lines | Four events: 3,000 hours, (brutal), 90 to 10 minutes, 3-item list |
| `speech_20s_noevents.mp4` | Sentences, feed lines | No "Found:" line. If one appears, record it and change the README expectation with its reason; never change the detector to fit |
| `speech_20s_long_pause.mp4`, `speech_30s_sparse.mp4` | Sentences with times | Words after the pause start at 10,000 ms or later |
| `v3testclip.mp4` | Recognized sentences with times, numbers, events, and where the text differs from the founder's words | Accepted, `ready`; a rejection or failure here blocks the exit (§20.1) |

**The founder's own transcripts may be committed (D-112); never paste a lent clip's transcript anywhere.**

### Milestone 9

```bash
pnpm build:wasm                                             # expect: exit 0
pnpm check                                                  # expect: exit 0
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"      # expect: 92 passed
cargo test --workspace 2>&1 | awk '/^test result:/ {f+=$6} END {print f" failed"}'   # expect: 0 failed
pnpm e2e 2>&1 | tail -2                                     # expect: 12 passed
pnpm e2e:media --grep @no-gpu 2>&1 | tail -2                # expect: 8 passed
wc -l web/src/usecases/run-pipeline.ts web/src/usecases/import-clip.ts   # expect: both under 400
# frozen check (Step 1.7)                                   # expect: no output
```

| Check | Where | Expect |
|---|---|---|
| Drop `fixtures/.gen/rej_silent.mp4` | `pnpm dev` | V2's rejected stub (the panel comes in Phase 10); one `clip_rejected` with `"reject_reason":"REJECT_NO_SPEECH"` in the events request |
| Drop `fixtures/.gen/rej_4k.mp4` | `pnpm dev` | V2's rejected stub; one `clip_rejected` with `"reject_reason":"REJECT_RESOLUTION"`; no `clip_accepted` |
| IndexedDB after the reference clip | DevTools | Three records as in Step 9.5 |
| README | `fixtures/speech/README.md` | The six "as recognized" entries filled |

Pass: the reference clip reaches `ready` with the expected feed and three records, `audio_chain` appears in `stage_timing`, both rejection paths send `clip_rejected` with the code alone, and the README holds every recognized transcript.

Commit: `"V3 S8: records, the voice chain in the pipeline, rejection events"`

Do not start Phase 10 until this passes.

---

## Phase 10 - Rejection panel and copy (S9)

Goal: every refused file shows its own title, body and action from `messages.rejections`, with limits interpolated from `LIMITS`, and the copy check counts all 14 codes as covered.

### Step 10.1 - The 14 messages

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `web/src/copy/messages.ts` | §16.3, D-80 | `rejections` `satisfies Record<RejectReason, ErrorCopy>`; `BYTES_PER_MB = 1_000_000`, `FPS_MILLI_PER_FPS = 1000` beside `MS_PER_SECOND`; no limit typed in a sentence; remove `editor.rejectedStub` |
| 2 | `scripts/check-copy-codes.mjs` | §18.4 | Remove the 14 `REJECT_*` entries and their comment from `COPY_PENDING` |

```bash
grep -oE '\bREJECT_[A-Z_]+:' web/src/copy/messages.ts | sort -u | wc -l   # expect: 14
grep -c "REJECT_" scripts/check-copy-codes.mjs              # expect: 0
grep -niE "never leaves|gdpr|dpdp|ccpa|soc 2|compliant" web/src/copy/messages.ts   # expect: no output (P-12)
node scripts/check-copy-codes.mjs                           # expect: exit 0
```

### Step 10.2 - The panel and the page

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `web/src/ui/styles/components.module.css` | §16.4 | `.rejection`, `.rejectionTitle`, `.rejectionBody`, `.rejectionAction`; `composes:` V2's `.stub`; no `style` attribute |
| 2 | `web/src/ui/components/RejectionPanel.tsx` | §16.2 | `role="alert"`, `data-testid="clip-rejected"`, `data-reject={code}`; no retry button; no code, size or name shown |
| 3 | `web/src/ui/pages/EditorPage.tsx` | §16.1 | The `rejected` arm becomes `<RejectionPanel />` |

```bash
pnpm check                                                  # expect: exit 0
git grep -n "rejectedStub" -- web/src                       # expect: no output
```

### Step 10.3 - Look at every rejection in `pnpm dev`

| Do | Expect |
|---|---|
| Drop `rej_4k.mp4` | Title "This video is larger than Offcut handles"; body contains `1920`; "start over" button; no retry button |
| Drop `rej_hevc.mov` | Body names HEVC; action names "Most Compatible" |
| Drop `rej_too_long_91s.mp4` | Body contains `90` |
| Drop `rej_120fps.mp4` | Body and action contain `60` |
| Click "start over", drop `ok_vfr.mp4` | The drop zone returns; the clip is accepted |
| Inspect the panel element | `data-reject="REJECT_FRAME_RATE"` (for the last refusal); the page text has no file name |

### Step 10.4 - Prove the guards

| Temporary edit | Rule that must fire |
|---|---|
| Delete the `REJECT_AUDIO_CODEC` entry | `tsc` fails (`satisfies Record`) and `check-copy-codes.mjs` names `REJECT_AUDIO_CODEC` |
| Type `90` into the `REJECT_DURATION` body | Review rule (INV-15; `messages.test.ts` arrives in V8): confirm `grep -n "90 seconds" web/src/copy/messages.ts` finds it, then undo |
| Add a literal sentence in `RejectionPanel.tsx` JSX | ESLint `react/jsx-no-literals` fails |

Undo all three.

### Milestone 10

```bash
pnpm check                                                  # expect: exit 0
node scripts/check-copy-codes.mjs                           # expect: exit 0
pnpm build                                                  # expect: exit 0
pnpm e2e 2>&1 | tail -2                                     # expect: 12 passed
pnpm e2e:media --grep @no-gpu 2>&1 | tail -2                # expect: 8 passed
# frozen check (Step 1.7)                                   # expect: no output
```

| Check | Where | Expect |
|---|---|---|
| Four refusals of Step 10.3 | `pnpm dev` | Their exact titles and interpolated limits |
| `COPY_PENDING` | `scripts/check-copy-codes.mjs` | `E_*` codes of V2 §22.4 (including `E_DSP`) and the four blockers; no `REJECT_*` |

Pass: the 14 codes have copy, the copy check passes without pending rejections, each refusal of Step 10.3 shows its own limit, and no forbidden phrase exists.

Commit: `"V3 S9: rejection panel and the 14 rejection messages"`

Do not start Phase 11 until this passes.

---

## Phase 11 - E2E suites, corpus tooling and CI (S10, S11)

Goal: the rejection suite and the rewritten preview cases pass on D1, the corpus runner and bench accept the new clips, and both CI jobs generate the fixtures and pass on the pull request.

### Step 11.1 - Helpers

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `web/tests-e2e/helpers/fixtures.ts` | §19.15 | `fixturesDir`, `genFixture` (stamp check), `dropFile`, `dropOversize` (501 parts of 1,048,576 bytes), `v3TestClip`, `readIndexedDb`, `rejectCodeOf`; `dropClip` calls `dropFile` |

### Step 11.2 - The rejection suite

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `web/tests-e2e/rejections.spec.ts` | §19.16, D-93 | 16 cases: one loop over the manifest's `rej_*` entries **excluding `rej_silent.mp4`** (12 files) plus the oversize case (13 import-time codes, `@no-gpu`); `REJECT_NO_SPEECH` (untagged, D1); "start over" (`@no-gpu`); the `ok_vfr.mp4` accept case (`@no-gpu`, with `sourceHashPrefix` recomputed in Node from the file). 15 cases carry `@no-gpu` (D-93) |
| 2 | `web/playwright.config.ts` | §18.3 | `rejections.spec.ts` joins `MEDIA_SUITES` |

```bash
pnpm e2e:media --list 2>&1 | grep -c "rejections.spec.ts"   # expect: 16
pnpm e2e:media --grep @no-gpu --list 2>&1 | tail -1         # expect: Total: 23 tests (V2's 8 + 15)
pnpm e2e:media --grep @no-gpu 2>&1 | tail -2                # expect: 23 passed
```

### Step 11.3 - The preview and export cases

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `web/tests-e2e/pipeline-preview.spec.ts` | §19.17, D-91 | Rewrite "Feed", "Real detections only", "Events"; extend the OPFS case with the three records; add the D1-only second-recording case (skip reason of D-99). **REOPENED V2 CONTRACT** |
| 2 | `web/tests-e2e/export-creator.spec.ts` | §19.18, D-100 | One D1-only case: no tag key containing `location` or `ISO6709` (any case, format and stream level), no `creation_time` from the source, no `title`, `comment`, `encoder` tag, and no `©xyz` or `ISO6709` bytes in the export; verifier checks 1-6; delete the temp file in a `finally` |

### Step 11.4 - The bench

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `bench/device-bench.ts` | §19.19, D-91 | `STAGES` gains `"audio_chain"` (six); `BENCH_CLIP` of `reference`, `portrait`, `v3testclip`; file name `bench/results/d1-<date>-<clip>.json`. **REOPENED V2 CONTRACT** |

### Step 11.5 - The corpus runner

| Order | File | Section | Tests to write with it / Watch for |
|---|---|---|---|
| 1 | `corpus/fetch_corpus.sh` | §17.6 | No network; founder clips from `testclips/`; `ok`, `missing`, `hash mismatch` per clip; LF endings |
| 2 | `corpus/run_corpus.ts` | §17.6, D-88, D-112 | Modes `demux`, `export`, `wer`, `transcripts`; results hold ids, codes, booleans and numbers only; a failure message names the clip id and outcome code only. `transcripts` mode writes only `founder-02` to `a02-v3testclip.json` by §17.6's fixed map, never `a01` (written form), and refuses `owner: "lent"` |
| 3 | `web/playwright.config.ts` | §18.3 | Project `corpus`: `testDir: "../corpus"`, `testMatch: "run_corpus.ts"`, `dependencies: ["media-setup"]`, timeout x 2, one worker, `{ ...MEDIA_USE, screenshot: "off", video: "off", trace: "off" }` (no page content of a lent clip on disk, D-112) |
| 4 | `web/tests-e2e/tsconfig.json` | §18.3 | `include` gains `../corpus` |
| 5 | `package.json` | §18.2 | `"corpus": "sh corpus/fetch_corpus.sh && pnpm --filter web exec playwright test --project=corpus"` |

```bash
export OFFCUT_CORPUS_DIR="$HOME/offcut-corpus"; mkdir -p "$OFFCUT_CORPUS_DIR"
sh corpus/fetch_corpus.sh                                    # expect: founder-01 ok, founder-02 ok
pnpm --filter web exec playwright test --project=corpus --list 2>&1 | tail -1   # expect: Total: 2 tests (or one per present clip)
pnpm check                                                  # expect: exit 0 (both tsc programs see corpus/)
```

### Step 11.6 - Run the whole media suite on D1

```bash
pnpm build:wasm && pnpm fixtures
pnpm e2e:media 2>&1 | tail -3                               # expect: 48 or more passed, 0 failed, 0 skipped on D1
```

The 48 are V2's 30, the 16 rejection cases and the 2 second-recording cases. A skip means `testclips/v3testclip.mp4` was not found.

### Step 11.7 - CI: the Linux job

Edit `.github/workflows/ci.yml` (§18.5). Values come from Step 0.8.

```yaml
env:
  FFMPEG_LINUX_URL: <URL of Step 0.8>
  FFMPEG_LINUX_SHA256: <hash of Step 0.8>

      # before the step that runs cargo test (TS §33 step 4)
      - name: Install ffmpeg (Linux, pinned)
        run: |
          curl -fsSL -o /tmp/ffmpeg.tar.xz "$FFMPEG_LINUX_URL"
          echo "$FFMPEG_LINUX_SHA256  /tmp/ffmpeg.tar.xz" | sha256sum --check
          mkdir -p "$RUNNER_TEMP/ffmpeg"
          tar -xJf /tmp/ffmpeg.tar.xz -C "$RUNNER_TEMP/ffmpeg" --strip-components=1
          echo "$RUNNER_TEMP/ffmpeg/<folder of Step 0.8 holding the binaries, or nothing>" >> "$GITHUB_PATH"
      - name: ffmpeg version
        run: ffmpeg -hide_banner -version | head -1
      - name: Restore the reference clip
        uses: actions/cache@<the pinned version e2e-media.yml uses>
        with:
          path: fixtures/.cache
          key: <the key e2e-media.yml uses for fixtures/.cache>
      - name: Generate the fixtures
        run: pnpm fixtures
```

`pnpm` must be installed before "Generate the fixtures": move the step after the pnpm setup if `ci.yml` installs pnpm later. `VITE_ASSET_BASE_URL` must be in the job's `env` (the generator downloads the reference clip on a cache miss).

`scripts/check-external-facts.mjs`: one line for the Linux ffmpeg archive, its URL and the date its hash was taken (§18.4).

### Step 11.8 - CI: the Windows media job

Edit `.github/workflows/e2e-media.yml` (§18.6): move "Restore the model files and the reference clip" to directly after "Install ffmpeg", then add `run: pnpm fixtures`. If Step 0.2 chose a different ffmpeg archive, change the install step to it. Update the comment at the head to name the rejection suite.

### Step 11.9 - Push and read the CI logs

```bash
git push -u origin v3-build
gh pr create --base main --head v3-build --title "V3" --body "docs/v3/v3implementation.md"   # or open it on github.com
gh run watch                                                # expect: ci and e2e-media green
gh run view --log --job=<ci job id> | grep -E "ffmpeg version|running [0-9]+ tests" | head
gh run view --log --job=<ci job id> | grep -E "demux_fixtures|probe_rejections|loudness_target|duration_preserved|chain_no_clicks|tests/prosody" | grep -c "Running"
# expect: an "ffmpeg version 9.0..." or "n9.0..." line; 6
```

`gh` is not installed on D1 (Assumptions (a)): push from Git Bash, open the pull request on github.com, read each job's conclusion with `curl -s https://api.github.com/repos/mayanpathak/Offcut/commits/$(git rev-parse HEAD)/check-runs`, and have the founder paste the two log greps above from the job page. The same applies to every `gh` line in Phases 11 and 13.

On the Windows job, confirm `REJECT_DECODE_UNSUPPORTED` passed (D-78 on the runner); record it in `experiments.md` under `rej_h264_444 in Chrome`.

### Step 11.10 - Prove the CI guards

| Temporary edit | Rule that must fire |
|---|---|
| Change one hex digit of `FFMPEG_LINUX_SHA256` in a throwaway commit | The Linux job fails at `sha256sum --check` |
| Remove "Generate the fixtures" in a throwaway commit | The Linux job fails in step 4 with "run `pnpm fixtures`" |

Push each as a separate commit, watch it fail, then `git revert` it and push (or drop it with `git reset --hard HEAD~1 && git push --force-with-lease` before anyone reviews). End state: the branch is green.

### Milestone 11

```bash
pnpm check && pnpm fixtures && pnpm test                     # expect: exit 0
pnpm build                                                  # expect: exit 0
pnpm e2e 2>&1 | tail -2                                     # expect: 12 passed
pnpm e2e:media 2>&1 | tail -2                               # expect: 48 or more passed, 0 failed, 0 skipped (D1, keyed)
gh pr checks                                                # expect: every check pass
# frozen check (Step 1.7)                                   # expect: no output
```

Pass: every media case passes on D1 with none skipped, the rejection suite has 16 cases of which 15 run on the Windows runner, the Linux job runs all six fixture-reading Rust test files, and the pull request is green.

Commit: `"V3 S10-S11: rejection suite, corpus tooling, bench and CI fixtures"`

Do not start Phase 12 until this passes.

---

## Phase 12 - Measure (S12)

Goal: the V3 measurements of §20 are read on D1 in the production build, recorded with their decisions, and the first `d1` bench files are committed.

### Step 12.1 - The bench (TE-13, E-3, E-4)

```bash
export BENCH_DEVICE=d1
BENCH_CLIP=portrait pnpm bench:device                        # expect: bench/results/d1-<date>-portrait.json, 10 runs, six stages
BENCH_CLIP=v3testclip pnpm bench:device                      # expect: bench/results/d1-<date>-v3testclip.json
node -e '
for (const c of ["portrait","v3testclip"]) {
  const f=require("fs").readdirSync("bench/results").filter(x=>x.endsWith(`-${c}.json`)).sort().pop();
  console.log(c, f, JSON.stringify(require(`./bench/results/${f}`).stages ?? "see file"));
}'
```

Normalise every reading of `v3testclip` to 60 s: measured x 60,000 / 78,918 (D-74).

| Stage | Budget per 60 s on R1, read on D1 | Fallback |
|---|---|---|
| `audio_chain` (TE-13) | 4,000 ms | `denoise: false` (§20.2) |
| `asr` (E-3) | 20,000 ms target; 145,000 ms the line (D-67) | Over 145 s per 60 s: base-size model, a V4 decision |
| `render_encode` (E-4) | 90,000 ms | Over 150 s per 60 s: Canvas2D path, a V4 decision |

Write each median and decision into `experiments.md`, labelled `d1`.

### Step 12.2 - The second recording, end to end

| Do | Expect |
|---|---|
| `pnpm e2e:media --grep "second recording\|no metadata"` | 2 passed |
| `experiments.md` under `Second recording` | Accepted, `ready`, verifier checks 1-6 pass, no source metadata in the export |

### Step 12.3 - Fill `a02`'s data and re-run the gate

```bash
CORPUS_MODE=transcripts pnpm corpus -- --grep founder-02     # expect: fixtures/labeled/a02-v3testclip.json gains words and prosody
git diff --stat fixtures/labeled                            # expect: only a02-v3testclip.json changed
git diff fixtures/labeled | grep -E '^[-+].*"expected"'      # expect: no output (labels unchanged)
cargo test -p offcut-detect --test precision_recall -- --nocapture 2>&1 | grep -iE "overall|files read"
# expect: 31 files read, 0 skipped; precision >= 0.90
```

Add a `v3changelog.md` line: "a02-v3testclip.json: data filled from IndexedDB, labels unchanged (D-87, D-112)". If precision falls under 0.90, apply §20.3 (Step 8.3's table).

### Step 12.4 - TE-12, first run

```bash
sh corpus/fetch_corpus.sh                                    # expect: one line per manifest clip
CORPUS_MODE=demux pnpm corpus                                # expect: one test per present clip; 0 crashes
ls corpus/.results/                                          # expect: <date>-demux.json
```

Fewer than 10 clips present: write `TE-12: inconclusive (n clips)`, to be re-checked in V9 (D-88). Ten or more under 90%: record each failing clip's code; the JS-demuxer decision is taken before V4's preview work (§20.4). Copy counts and rates (no lent ids) into `experiments.md`.

### Step 12.5 - E-10 smoke reading

```bash
# local reference texts, never committed:
#   corpus/.clips/founder-01.txt  <- Script A text from fixtures/speech/README.md
#   corpus/.clips/founder-02.txt  <- ~/v3testclip-words.txt
cp ~/v3testclip-words.txt corpus/.clips/founder-02.txt
CORPUS_MODE=wer pnpm corpus                                  # expect: <date>-wer.json with counts only
```

Write both WER values into `experiments.md` under `E-10 smoke`, marked "smoke, not E-10 (carried to V9, D-88)".

### Step 12.6 - The founder reads five labels (D-87)

```bash
ls fixtures/labeled/w*.json | shuf -n 5                      # the five files
```

The founder reads each file's sentences and `expected` list and answers, per file, "a person would expect these labels: yes / no, because ...". Record the five names and answers in `experiments.md`. A "no" is fixed per D-87 with a changelog line, then Step 12.3's gate command is re-run.

### Step 12.7 - Replace the measured assumptions

In `docs/technicalspec.md` §39.3 and the sections they come from, replace each (assumption) V3 measured with its value and the reference "measured in V3, docs/v3/experiments.md": `DENOISE_DELAY_SAMPLES`, TE-13's decision, thresholds changed by the gate, the generator's time, `rej_silent.mp4` length, the decodability of `rej_h264_444.mp4`. Leave unmeasured ones listed.

### Milestone 12

```bash
ls bench/results/ | grep -E '^d1-.*-(portrait|v3testclip)\.json$' | wc -l   # expect: 2
grep -cE "^## " docs/v3/experiments.md                      # expect: 14 (the headings of Step 1.1)
grep -c "Numbers: $" docs/v3/experiments.md                  # expect: 1 (only E-10, carried to V9)
cargo test -p offcut-detect --test precision_recall 2>&1 | grep "test result"   # expect: ok. 1 passed
git status --short corpus fixtures/labeled                   # expect: only a02-v3testclip.json (no .clips, no .results)
pnpm check                                                  # expect: exit 0
# frozen check (Step 1.7)                                   # expect: no output
```

Pass: both bench files are committed, every experiment of §20.1 has numbers and a decision except E-10 (carried to V9), the gate passes on 31 files, the five-file reading is recorded, and no corpus clip, result or lent transcript is staged.

Commit: `"V3 S12: measurements, experiment records and the first d1 bench files"`

Do not start Phase 13 until this passes.

---

## Phase 13 - Copy-back, merge, deploy, tag (S13, S14, S15)

Goal: the specs say what V3 built, V3 is merged into `main` and deployed with the deployed page refusing and processing clips as on D1, and `v3` is tagged.

### Step 13.1 - Copy the decisions back (S13)

| File | What to copy | Source |
|---|---|---|
| `docs/technicalspec.md` | Every D-n of §23.2's first paragraph into its named section; the thresholds in use into §17.4; the corrections of §23 items 1-14 | §23, §23.2 |
| `docs/technicalspec.md` | The corrections that were found while planning and are now in `v3implementation.md`: six fixture-reading tests (item 1); split input and output ffmpeg flags (item 2); `limit` returns the clamp count (Step 3.3); SCHEMA's `tags`, `edits`, null-data rule and the written `a01` (item 3); `transcripts` mode's file map (Step 11.5); 15 `@no-gpu` cases (Step 11.2) | "Assumptions" below |
| `docs/buildplan.md` | D-74, D-76, D-87, D-88, D-93, D-94 into §5; V3's eight days | §23.2 |
| `docs/product.md` | D-80 into §9.4 | §23.2 |
| `docs/v3/v3implementation.md` | Nothing to copy: the review of 2026-10-15 applied this guide's findings there. Check that §23 items 25-28 hold their outcome (decided, or still open with their version) | §23 |

```bash
node scripts/check-file-tree.mjs                            # expect: exit 0
```

### Step 13.2 - The final local gate

```bash
pnpm install --frozen-lockfile && pnpm check                # expect: exit 0
pnpm fixtures && pnpm test                                   # expect: exit 0
cargo test --workspace 2>&1 | awk '/^test result:/ {p+=$4; f+=$6; i+=$8} END {print p" passed, "f" failed, "i" ignored"}'
# expect: 0 failed; passed well above 262; ignored = 2 + the V3 ignored cases (chain_timing)
pnpm --filter web exec vitest run 2>&1 | grep -E "Tests +[0-9]"      # expect: 92 or more passed
pnpm build                                                  # expect: exit 0
pnpm e2e 2>&1 | tail -2                                     # expect: 12 passed
pnpm e2e:media 2>&1 | tail -2                               # expect: 48 or more passed, 0 failed, 0 skipped (D1, keyed)
git grep -nE "\bfetch\(" -- web/src | grep -vE "net/http\.ts|net/asset-fetch\.ts|wasm/load-core\.ts|wasm/load-render\.ts"   # expect: no output
git grep -n "TODO" -- crates web/src | grep -vE "TODO\(V[0-9]+\)|V[0-9]+:"   # expect: no output (every TODO carries a version)
for f in $(git diff --name-only 8a271a9 -- crates web/src | grep -E '\.(rs|ts|tsx)$' | grep -v '/tests'); do
  n=$(awk '/#\[cfg\(test\)\]/{print NR-1; f=1; exit} END{if(!f) print NR}' "$f"); [ "$n" -ge 400 ] && echo "TOO LONG $f $n"
done; echo lengths checked                                  # expect: lengths checked
# frozen check (Step 1.7)                                   # expect: no output
```

Write the final counts into `v3changelog.md`; none may be below the baseline of Entry 1.

### Step 13.3 - Fill in the real values

V3 adds no secret. Check every value that must be real before the merge:

| File or place | Value | Check |
|---|---|---|
| `.github/workflows/ci.yml` `env` | `FFMPEG_LINUX_URL`, `FFMPEG_LINUX_SHA256` | The Linux job's "ffmpeg version" line on the pull request |
| `corpus/manifest.json` | Two SHA-256 values | `sh corpus/fetch_corpus.sh` prints `ok` twice |
| `fixtures/manifest.json` | Observed rotation, `ok_vfr.mp4` duration and rate | `pnpm fixtures` step 8 passes |
| `crates/offcut-dsp/src/config.rs` | `DENOISE_DELAY_SAMPLES`, `ChainConfig::default().denoise` | `impulses_stay_aligned`; `experiments.md` |
| `crates/offcut-detect/src/config.rs` | Thresholds in use | Equal to TS §17.4 |
| Production web build | No test public key | The test-key guard of `ci.yml` passes on `main` |
| Render, Vercel | Unchanged | - |

### Step 13.4 - Merge and deploy (S14)

```bash
git push                                                     # the Phase 12 and 13 commits
gh pr checks                                                # expect: every check pass
```

The founder merges the pull request (D-75). `ci.yml` on `main` deploys the server first, waits for its version, then the web build (TS §33).

```bash
export APP_URL=https://<your Vercel production host>         # from the Vercel dashboard or ci.yml
MERGE=$(git rev-parse origin/main)                           # after git fetch origin
gh run watch                                                # expect: the main run green, including the deploy steps
curl -s "$APP_URL/api/v1/healthz"                            # expect: 200 and the commit $MERGE (the field wait-for-version.mjs reads)
node scripts/check-headers.mjs <the arguments ci.yml passes it>   # expect: exit 0
```

### Step 13.5 - Verify the deployed system on D1

| Check | Where | Expect |
|---|---|---|
| `crossOriginIsolated` | DevTools console on `$APP_URL/app` | `true` |
| Drop `fixtures/.gen/rej_4k.mp4` | Deployed app | "This video is larger than Offcut handles", body containing `1920`, `data-reject="REJECT_RESOLUTION"` |
| Drop `fixtures/.gen/rej_hevc.mov` | Deployed app | The HEVC message with "Most Compatible" |
| "Try with a sample clip" | Deployed app | "Cleaning voice", "Transcribing…", the four always-expected "Found:" lines; the preview plays the cleaned voice |
| `@smoke` case | `pnpm e2e --grep @smoke` against the deployed URL, as V2 ran it | Passed |

In the Neon SQL editor:

```sql
select name, props, ts from analytics_events
where name in ('clip_rejected', 'stage_timing') and ts > now() - interval '1 hour'
order by ts desc limit 20;
-- expect: clip_rejected rows with props {"reject_reason":"REJECT_RESOLUTION"} and {"reject_reason":"REJECT_HEVC"}, no other key;
--         stage_timing rows whose props contain "stage":"audio_chain"
```

**Rollback.** If `healthz` does not report `$MERGE` within 15 minutes, or any check above fails on the deployed system, open a pull request with `git revert -m 1 $MERGE`, have it merged, and confirm `healthz` reports the revert's commit and the deployed app behaves as V2. Then fix on `v3-build` and repeat Steps 13.2-13.5.

### Step 13.6 - Tick the exit checklist and tag (S15)

| §22 group | Where it was proven |
|---|---|
| Code and checks | Step 13.2 |
| Deployment | Steps 13.4, 13.5 |
| Experiments and records | Phase 12; Step 13.1 |
| §21.3 "V4's first morning" | Steps 13.2 (markers, TODO), 12.1 (bench files), 9.6 (README), 8.3 and 13.1 (thresholds) |
| Founder confirms D-112 for `a01` and `a02` | Ask now; record the answer in `v3changelog.md` |

Write the last changelog entry ("V3 closed: <date>, merge <SHA>, final counts"), commit it on `main` through a small pull request if branch protection requires one, then tag:

```bash
git fetch origin && git switch main && git pull --ff-only
git tag -a v3 -m "V3: real ingest, clean voice, real detector" "$(git rev-parse HEAD)"
git push origin v3
git ls-remote --tags origin v3                                # expect: one line with refs/tags/v3
```

### Milestone 13

```bash
curl -s "$APP_URL/api/v1/healthz"                            # expect: 200, commit equal to the merge (or the changelog commit after it)
node scripts/check-headers.mjs <ci.yml arguments>           # expect: exit 0
git describe --tags --exact-match origin/main 2>/dev/null || git tag --points-at v3   # expect: v3 points at the commit deployed
git diff --name-only v2 v3 -- server                          # expect: no output (if v2 exists; else compare with 8a271a9)
grep -c "\[x\]" docs/v3/v3implementation.md                   # if you tick §22 in place: every box of §22 ticked
```

| Check | Where | Expect |
|---|---|---|
| Rejection on the deployed app | `$APP_URL/app` | `rej_4k.mp4` refused with its message |
| Analytics rows | Neon SQL editor | `clip_rejected` with the code alone; `stage_timing` with `audio_chain` |
| Spec copy-back | `docs/technicalspec.md`, `docs/buildplan.md`, `docs/product.md` | Every item of Step 13.1 present |
| Tag | GitHub | `v3` on the deployed commit |

Pass: the deployed app refuses `rej_4k.mp4` and `rej_hevc.mov` with their messages and processes the sample clip with the cleaned voice and the four always-expected events, the analytics rows carry only enums, every box of §22 is ticked, and `v3` is pushed.

Commit: `"V3 S13: spec copy-back and close-out"` (S14 is the merge, S15 the tag).

V4 must not start until this passes.

---

## Quick reference

### Start every terminal

```bash
cd /c/Users/Mayan/Desktop/sh2clips/offcut && export BENCH_DEVICE=d1 OFFCUT_CORPUS_DIR="$HOME/offcut-corpus" && ffmpeg -hide_banner -version | head -1 && git branch --show-current
```

Add `export VITE_ENTITLEMENT_TEST_PUBLIC_KEY=...` only in terminals that build for the media suites; never write it into a tracked file.

### After changing ... run ...

| After changing | Run |
|---|---|
| `fixtures/manifest.json` or `fixtures/gen_fixtures.sh` | `pnpm fixtures` (the stamp forces regeneration), then `cargo test --workspace` |
| A recipe | `rm -rf fixtures/.gen && pnpm fixtures` twice and the determinism diff of Step 2.3 |
| The reference clip | `pnpm fixtures` (its hash is in the stamp) |
| Any Rust crate used in the browser | `pnpm build:wasm`, then the media suites |
| `Cargo.toml` dependencies | `cargo deny check`; the purity `cargo tree` grep of Step 1.5; `cargo build -p offcut-dsp --target wasm32-unknown-unknown --release` |
| A shared type (none planned in V3) | `pnpm gen:types && sh scripts/check-gen-clean.sh` |
| `web/src/copy/messages.ts` | `node scripts/check-copy-codes.mjs`; `pnpm check` |
| A new source file, or the TS §5 tree | `node scripts/check-file-tree.mjs` |
| `fixtures/labeled/*.json` after the freeze | A `v3changelog.md` line; `cargo test -p offcut-detect --test precision_recall -- --nocapture` |
| A detector threshold or lexicon | `rules_table.rs`, `precision_recall.rs`, `determinism.rs`; note it for TS §17.4 |
| A DSP constant or `ChainConfig::default()` | `duration_preserved.rs`, `loudness_target.rs`, `chain_no_clicks.rs`; Step 4.3 |
| `web/playwright.config.ts` | `pnpm e2e:media --list` and compare totals (48 on D1, 23 `@no-gpu`) |
| `.github/workflows/*.yml` | Push and read the log lines of Step 11.9 |
| `bench/device-bench.ts` | One `pnpm bench:device` run; check the six stages in the result |
| Anything at all | The frozen check of Step 1.7 |

### When something fails

| When something fails | Likely cause |
|---|---|
| `'sh' is not recognized` from `pnpm fixtures` | pnpm launched outside Git Bash; Step 0.1 |
| `$'\r': command not found` in a `.sh` file | CRLF line endings; re-save as LF, check `git ls-files --eol` |
| `ffmpeg 9.0 is needed, found: ffmpeg version n9.0...` | The version `case` lacks the `n` prefix; Step 2.2 |
| `Option map_metadata not found` or a muxer ignoring `-map_metadata` | Output flags placed before `-i`; split `GLOBAL` and `OUTF` |
| The determinism diff differs on `rej_hevc.mov` | x265 thread pool; add `pools=1:frame-threads=1` |
| A Rust test panics "run `pnpm fixtures`" | `fixtures/.gen/` missing or the manifest changed since the stamp |
| `impulses_stay_aligned` fails | `DENOISE_DELAY_SAMPLES` not the denoiser's real delay; Step 3.5 |
| `prop_length_equals_input` fails at a length like 47,999 or 481 | The last partial block or the denoiser's pending samples dropped; `Denoise::finish` |
| `loudness_target.rs` reports clamped samples | The limiter's ramp does not reach the required gain at the peak; check the moving minimum covers `LIMITER_LOOKAHEAD + 1` samples |
| Clippy `indexing_slicing` in DSP or detector code | `v[i]` in non-test code; use iterators or `.get()` |
| `tsc` fails in `ProcessingFeed.tsx` after Phase 1 | The `from_to` copy signature changed without the component; Step 1.6 |
| `check-hosts.mjs` fails after Phase 4 | URL text inside `offcut_core_bg-*.wasm` from a new crate; Step 4.4 |
| `measureProsody` fails with `NoAudio` in the pipeline | `runChain` failed or was not awaited, or a rejected clip's buffer was dropped; §15.2 steps 9-11 |
| Two clips' events mixed, or a stale `out48` | `kept` not reset at the start of `runChain`; §14.1 |
| `precision_recall.rs` reads 29 files | `a01` missing or `a02` skipped while another file has invalid JSON; read the printed list |
| `precision_recall.rs` fails `deny_unknown_fields` | A field outside SCHEMA in a labeled file; Step 2.5's list |
| `REJECT_DECODE_UNSUPPORTED` case fails on the runner | Chrome there decodes the fixture; D-78's fallback chain |
| `rejections.spec.ts` NO_SPEECH case gets `clip_accepted` | The recognizer heard words in `rej_silent.mp4`; D-111's 2 s fixture |
| `pipeline-preview.spec.ts` "Events" fails on `n_keyword` | The KeywordPop of "brutal" depends on prosody; the case must accept 0 or 1 |
| The Linux CI job cannot find `ffmpeg` | Wrong folder appended to `GITHUB_PATH`; the archive layout of Step 0.8 |
| `healthz` reports the old commit after the merge | The deploy step failed or Render is still building; read the `main` run, then Step 13.5's rollback rule |

---

## Assumptions I made and gaps found

### (a) Environment values assumed

| Value | Assumed | How to correct |
|---|---|---|
| OS and shell | Windows 11 (D1), Git Bash for every command; WSL bash exists and is avoided | Step 0.1 checks it |
| Repo path | `/c/Users/Mayan/Desktop/sh2clips/offcut` (the repository; the folder above it is not one) | Replace once in the Quick reference line |
| GitHub CLI | `gh` is **not installed** on D1. Every `gh` command below has an equivalent: open the pull request on github.com; read checks with `curl -s https://api.github.com/repos/mayanpathak/Offcut/commits/<sha>/check-runs`; job logs need a login, so the founder pastes the lines asked for | `coding-promptsv3.md` Prompts 87 and 90 use the API |
| Tools | V2's toolchain is installed; ffmpeg 9.0.2 essentials is on PATH | Step 0.2 audits it |
| Keyed build | Built as V2's changelog records; the key lives in the shell | Step 0.6 checks `package.json` |
| Postgres | Neon (V2 CL mentions it); rows read in its SQL editor | Use your provider's console |
| App URL, script arguments | `$APP_URL` from Vercel; `check-headers.mjs` and `wait-for-version.mjs` take the arguments `ci.yml` passes | Read `ci.yml` |
| Days | 8 working days (§5), not BP §0's three | - |
| `pool` export name, IndexedDB field names | As §13.4 and `gen/domain.ts` say | Steps 4.3 and 9.6 say what to do if they differ |

### (b) Spec contradictions and omissions, and how the guide resolves them

**Status on 2026-10-15.** Every finding below except 16 and 17 (which only S1 and Step 0.8 can settle) has been written into `v3implementation.md`, which now agrees with this guide. The table stays as the record of why. The same review added four open items to §23 (25: cleaned audio left in `audio.worker`; 26: IndexedDB records outlive "start over"; 27: written emphasis lands exactly on the KeywordPop threshold; 28: the 40-word split's tie-break) and fixed, in the plan: two unsorted lexicons (`CURRENCY_SYMBOLS`, `ORDINAL_MARKERS`) that their own "sorted" tests would fail; two wrong expectations of §8.7; two of §19.11 ("I have 3 tools." and "*Workflow* matters.", which scores 0.80, not 0.90); D-78's length (2 s); D-86's wording against §9.2; the metadata checks of D-100, §19.2 and §19.18 (Apple's `ISO6709` key); the corpus project's screenshots and traces (D-112).

| # | Finding | Resolution | Step |
|---|---|---|---|
| 1 | §6.4 and §19.1 say four tests read generated fixtures; §19.5 and §19.7 make it six | Helper in six files | 3.1, 5.3 |
| 2 | §17.1 lists input and output ffmpeg flags as one set; its version check rejects `n9.0.2` Linux builds | Two variables; a `case` with both prefixes | 2.2 |
| 3 | §17.1 step 7 makes PCM for "three synthetic fixtures loudness_target.rs reads"; none exists (§19.4 makes its signals in the test) | PCM for the 5 speech fixtures only | 2.1 |
| 4 | §1A item 6 omits `libopus`, used by `rej_vp9.webm` | Audited in Step 0.2 | 0.2 |
| 5 | D-87 freezes `a01`/`a02` in S2, but their measured data cannot exist before S8 | `a01` written-form; `a02` labels frozen with null data, filled in Step 12.3; null-data files skipped | 2.7, 8.3, 12.3 |
| 6 | §17.5's coverage check needs trap, form and edge-case information the schema cannot carry under `deny_unknown_fields` | Optional `tags` in SCHEMA | 2.5 |
| 7 | §17.5's stated totals (NumberReveal 12, ListReveal 8, announcer 5, empty 7) do not match its own file plan (10, 7, 4, 8); with `a01` the minimums are met | Expected tallies from the plan; copied back | 2.6, 13.1 |
| 8 | D-83 changes the copy signature in S1; §4 changes `ProcessingFeed.tsx` in S9: `tsc` fails in between | Component change moved to S1 | 1.6 |
| 9 | §7.7's `limit` returns nothing, but `loudness_target.rs` must assert a clamp count | `limit` returns the count | 3.3 |
| 10 | §5 puts the "as recognized" README sections at S2; V2 cannot show a transcript, three of the kinds or prosody | Moved to Step 9.6 | 9.6 |
| 11 | `corpus/fetch_corpus.sh`, `run_corpus.ts` and the bench change have no S-step | Placed in Phase 11 | 11.4, 11.5 |
| 12 | §17.6 names `a0N-<id>.json`; D-99 names `a02-v3testclip.json` | A fixed map in `transcripts` mode | 11.5 |
| 13 | D-93 says 13 cases are `@no-gpu`; §19.16 tags 15 | 15 | 11.2 |
| 14 | §19.16's manifest loop would include `rej_silent.mp4`, a GPU case | Excluded from the loop | 11.2 |
| 15 | §4 omits `offcut-mp4/Cargo.toml`, `web/tests-e2e/tsconfig.json`, `scripts/check-external-facts.mjs` | Allowed by the frozen check; copied back | 1.7, 13.1 |
| 16 | It is unknown whether `check-file-tree.mjs` fails on a listed but missing file | Checked in Step 1.3, lines deferred if needed | 1.3, 5.1, 6.3 |
| 17 | The Linux ffmpeg 9.0.2 static build's URL is not given | Chosen and pinned in Step 0.8 | 0.8, 11.7 |
| 18 | §5's test-count target "30 + 16 + 2" assumes no V2 media case was removed | 48 or more | 11.6 |

### (c) S-steps and files placed differently from §5

| Item | §5 | This guide | Reason |
|---|---|---|---|
| S6 (audio in the browser) | After S3 and S5 | Phase 4, right after S4 | Risk first; S6 depends on S4 only; §5's own note on S4 |
| S3 (demuxer) | Day 2, before S4 | Phase 5 | Lower risk; no dependency on S4 or S6 |
| S7 | One step | Phases 7 and 8 (adjacent) | 2.5 days of work |
| README "as recognized" sections | S2 | Step 9.6 | Needs V3's pipeline and records |
| `a02` data | S2 | Step 12.3 | Needs S8's records; its labels stay in S2 |
| `ProcessingFeed.tsx` | S9 | Step 1.6 | Compiles only with D-83's signature |
| `corpus/manifest.json` | (no step) | Step 1.8 | D-99 says S1 |
| `corpus/fetch_corpus.sh`, `run_corpus.ts`, `bench/device-bench.ts`, the `corpus` script and project | (no step) | Phase 11 | Playwright and E2E helpers exist there |
| `pnpm fixtures` script | §18.2 (S11 by section) | Step 2.3 | S2 needs it |