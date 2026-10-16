# v3changelog.md - record of every change made while building V3

**What this is.** A record of every change made while building V3: what was done, what differs from the plan, the guide or the prompts, how it was checked, and what is still open. `v3implementation.md` says what each file contains; `v3buildguide.md` gives the order of work; `coding-promptsv3.md` holds the prompts (61 to 90).

**How it is kept.** One entry per prompt, or per change made outside a prompt. Entries are in date order, oldest first; new entries go at the end. Every commit that changes the repository carries its entry. The two tables "Open items" and "Known issues" are rewritten when an item opens or closes.

**References.** `§n` is a section of `v3implementation.md`, `G n.m` a step of `v3buildguide.md`, `D-n` a decision of §2, `TS §n` of `technicalspec.md`, `V2 §n` of `v2implementation.md`, `V2 CL item n` an open item or known issue of `v2changelog.md`.

**The baseline.** V3 starts from `origin/main` at `8a271a9967ab1e80826feec1107b307da45b80b5`, the deployed V2 (tagged `v2`). Every frozen-file check compares with that commit. The counts there (Entry 1): 262 Rust tests passed with 2 ignored, 92 Vitest, 12 Playwright (`pnpm e2e`), 30 media cases (`pnpm e2e:media`). No count may fall.

---

## Open items
| # | Item | Owner | Version |
|---|---|---|---|
| 2 | **Write down the words spoken in `testclips/v3testclip.mp4`**, sentence by sentence, into `~/v3testclip-words.txt` (outside the repository, D-99) | Human | Before Prompt 65 |
| 3 | **Decide §23 item 26**: after "start over", the clip's transcript stays in IndexedDB until V7. Accept that, or add a `clear()` to each of the three repos, called at import step 4 | Human | Before Prompt 80 |
| 6 | The `buildplan.md` half of V2 §27's copy-back list (the schedule moves of D-18, D-20, D-21, D-23, D-24, D-33) is not applied. Prompt 61 may change `technicalspec.md` only | Agent | Prompt 89 (copy-back) |
| 7 | **The Linux ffmpeg URL is short-lived.** BtbN deletes its dated autobuild releases after about six weeks (on 2026-10-16 the oldest one listed was of 2026-08-31), so the URL of Entry 1 stops working around the end of November 2026. Prompt 87 writes it into `ci.yml`: check it then, and choose there between a newer build of the same day and a copy kept on the asset host | Agent | Prompt 87 |
| 9 | **`fixtures/gen_fixtures.sh` may be checked out with CRLF on the Windows runner.** The repository has no `.gitattributes`; GitHub's Windows images set `core.autocrlf=true`, and `sh` cannot run a script with CRLF line endings. D1 (`core.autocrlf=input`) and Linux check it out LF. Today no Windows job runs a `.sh` file; Prompt 87 makes `e2e-media.yml` run `pnpm fixtures`. Then either a `git config --global core.autocrlf false` step before the checkout, or a root `.gitattributes` with `*.sh text eol=lf` (a new file: TS §5 entry in the same change) | Agent | Prompt 87 |
| 10 | **`pnpm fixtures` takes 118-166 s on D1**, not the "under a minute" D-94 and §17.1 assume (40 s). The two 1080p 60 s clips take most of it even with 4 x264 threads (Entry of Prompt 63). It runs on every `ci` run and every `e2e-media` run: Prompt 87 reads the Linux runner's time and decides between paying it and caching `fixtures/.gen/` by the `STAMP` key (D-94 says no cache) | Agent | Prompt 87 |

## Known issues
| # | Issue | Where | Version |
|---|---|---|---|
| 1 | `nnnoiseless` 0.5.2 depends on `easyfft`, which keeps a process-wide cache of FFT plans (`generic_singleton`, `parking_lot`). The purity check of §3.2 passes (none of the six forbidden crates), and it is not code of a pure crate, but it is global state inside `offcut-dsp`'s tree. Its output does not depend on the cache; `determinism`-style checks in Prompt 67 should still run the chain twice in one process | `crates/offcut-dsp` (Prompt 67) | V3 |
| 2 | `scripts/check-file-tree.mjs` checks that every file on disk is listed in TS §5, not that every listed file exists. `edit_list.rs` and `cardinals.rs` are therefore listed from Prompt 61 on, before they exist (G 1.3's first behaviour), as many V4-V10 files already are | `scripts/check-file-tree.mjs` | - |

---

## Entry 1 - 2026-10-16 - S1: baseline
| Item | Value |
|---|---|
| Baseline commit | 8a271a9967ab1e80826feec1107b307da45b80b5 |
| `v2` tag | absent at the start; created on 2026-10-16 as an annotated tag on `8a271a9` at the founder's request, pushed in Prompt 62 |
| Rust tests at baseline | 262 passed, 0 failed, 2 ignored: the 262 does not count the two ignored tests |
| Vitest at baseline | 92 |
| `pnpm e2e` at baseline | 12 |
| `pnpm e2e:media` at baseline (D1, keyed) | 30 of 30 (Playwright prints "32 passed": the 30 cases and the 2 cases of the `media-setup` project), 10.7 min; the speech model ran on WebGPU, 69,686 ms |
| SHA-256 `speech_scriptA_landscape_720p.mp4` | 7da948cecc39438a1b7a9591c84dc1ff938fdcfd4dad8218b43926b13240c124 (the copy in `fixtures/.cache/media/` is identical; its name carries `7da948cecc39438a`) |
| SHA-256 `v3testclip.mp4` | e600b6bf2048130aa6ac8468be98496c5ebeede00d819f3b56206b4fc00610bc |
| `nnnoiseless`, `ebur128` | 0.5.2, 0.1.10 (both `default-features = false`; the latest stable release on 2026-10-16) |
| ffmpeg on D1 | `ffmpeg version 9.0.2-full_build-www.gyan.dev`; replacements: none (all eight encoders of G 0.2 and the six filters present, `libx264` lists `yuv444p`). The CI Windows archive (`9.0.2-essentials_build`, SHA-256 `60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba` as in `e2e-media.yml`) has the same, less `libsvtav1` (open item 8) |
| Linux ffmpeg archive | `https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-10-16-13-04/ffmpeg-n9.0.2-28-g669f5ab083-linux64-gpl-9.0.tar.xz`, SHA-256 `1592a01343378ff03db9022878765919119073d4b84ee22e020cc45f57e29457` (equal to the digest GitHub lists for the asset), binaries in `ffmpeg-n9.0.2-28-g669f5ab083-linux64-gpl-9.0/bin/` |
| `offcut_core_bg`, `offcut_render_bg` size at baseline | 459,281 bytes, 6,033,574 bytes |
| Docs review before S1 | 2026-10-15: `v3implementation.md` and this guide made consistent with each other and with the V2 changelog (six fixture-reading tests, `@no-gpu` count 15, `a01`/`a02` handling, sorted lexicons, `limit` count, §8.7 and §19.11 expectations, metadata checks, §23 items 25-28) |
| Founder's decisions on §23 items 26 and 27 | item 27, 2026-10-16: written emphasis gets `pitch_z = 2.0` (score 0.90); no rounding before the threshold (Prompt 62). Item 26: pending (open item 3) |

### Prompt 61 (V3 01/30): preflight, V3 documents, spec tree, ignore rules

**Done.**

- **G 0.1.** Git Bash (`MINGW64_NT-10.0-26200`); `bash` and `sh` are Git's; `cmd //c where sh` finds `C:\Program Files\Git\usr\bin\sh.exe`, so `pnpm fixtures` will find `sh`.
- **G 0.2, tools.** Rust 1.99.0 (as `rust-toolchain.toml`) with `wasm32-unknown-unknown`; wasm-bindgen 0.2.129 (equal to `Cargo.lock`); wasm-opt 133; cargo-deny 0.20.2; Node 24.19.0; pnpm 10.15.0 (the `packageManager`); Playwright 1.63.0; Python 3.13.5 with NumPy 2.5.3; ffmpeg and ffprobe 9.0.2 (full build); sha256sum 8.32, curl 8.12.1, shuf 8.32, tar 1.35, xz 5.6.4.
- **G 0.3.** `v3-build` cut from `origin/main`; `git merge-base` gives `8a271a9`. `git switch -c` set `origin/main` as the branch's upstream; it was removed (`git branch --unset-upstream`) so that no plain `git push` or `git pull` on `v3-build` names `main`. Prompt 87's `git push -u origin v3-build` sets the right one.
- **G 0.4.** `git grep "V3: \|V7: "` in `web/src/usecases` gives the five markers of §1A item 7. TS names `sps.rs`, `mux_boxes.rs`, `media.setup.ts`, `tests-e2e/tsconfig.json` and the six `docs/v2/` files (§1A item 8). Of V2 §27's copy-back list, D-46, D-55, D-63 (not on the list, applied) and D-68 were in TS; the other 16 were not (below).
- **G 0.5.** `v3testclip.mp4`: 46,360,015 bytes, 78.918167 s, a `location` tag, ignored by `.gitignore:23: testclips/`. The reference clip, in `testclips/` and in `fixtures/.cache/media/`: no format or stream tag matching `location|iso6709`, and a byte search finds neither `ISO6709` nor `©xyz` in either file.
- **G 0.6.** Counts above, with `pnpm install --frozen-lockfile`, `cargo test --workspace`, `vitest run` and `pnpm e2e` on the baseline. `pnpm check` **failed** on the baseline (next point). The media run used a build with the test key, exported in that one script and unset after it.
- **`cargo deny` fails on the baseline: four advisories against `rustls` 0.23.45**, published after V2's last green run: RUSTSEC-2026-0346 (`Debug` of client session values shows TLS secrets), -0347 (buffered data sent to the outer server after an ECH rejection), -0348 (Brotli certificate decompression allocates up to 1 GiB before authentication), -0349 (an unauthenticated ServerHello can downgrade later connections to classical key exchange). The solution each names is `rustls` >= 0.23.46. `rustls` is reached only through `sqlx` (the server's Postgres connection). **Fixed in this prompt with `cargo update -p rustls`: `Cargo.lock` only, 0.23.45 to 0.23.46, one package, no `Cargo.toml` change.** Why here, though the prompt changes no dependency: the SAB gate needs `pnpm check` green and every later prompt's gate too, and the smallest change that does it is the lock line (SAB "fix the minimum and record why"). The server's code is not touched and the frozen check is empty. `main` still has the old version (open item 5).
- **G 0.7.** On `scratch/v3-deps`: `cargo add -p offcut-dsp nnnoiseless ebur128 --no-default-features` gave 0.5.2 and 0.1.10. The purity grep of `cargo tree -p offcut-dsp -e normal` printed nothing; `cargo deny check licenses bans`: `bans ok, licenses ok` (no licence to add); `cargo build -p offcut-dsp --target wasm32-unknown-unknown --release` finished. New crates in the normal tree: `nnnoiseless` brings `easyfft`, `array-init`, `generic_singleton`, `anymap3`, `parking_lot` (with `lock_api`, `parking_lot_core`, `scopeguard`), `realfft`, `rustfft`, `num-complex`, `once_cell`; `ebur128` brings `bitflags` 1.3.2, `dasp_frame`, `dasp_sample`, `smallvec` (known issue 1). Reverted with `git checkout -- .`, the branch deleted; `git status` showed only `?? docs/v3/` afterwards.
- **G 0.8.** No static Linux build names 9.0.2 at johnvansickle.com (its release is 7.0.2). BtbN's `FFmpeg-Builds` publishes a daily static `linux64-gpl` build of the 9.0 release branch at dated URLs: the build of 2026-10-16 (`n9.0.2-28-g669f5ab083`, 9.0.2 plus 28 commits of the release branch) was taken, the latest of the ones that exist, because a dated release is deleted after about six weeks (open item 7). Its version line starts `n9.0.2`, which §17.1's check accepts. The archive (151,156,356 bytes) is in the scratchpad, not in the repository; the binary cannot run on D1, so its encoders are read in the CI log in Step 11.10.
- **The essentials archive of CI** was also downloaded (its SHA-256 equals `FFMPEG_SHA256` of `e2e-media.yml`) and audited on D1: open item 8.
- **G 1.1.** This file, and `docs/v3/experiments.md` with the 14 headings of G 1.1, each with empty `Question:`, `Method:`, `Numbers:`, `Date:`, `Decision:` lines.
- **G 1.2.** `docs/v2/experiments.md` ends with the M0 gate decision, as G 1.2 writes it, dated 2026-10-16.
- **G 1.3.** TS §5 lists `crates/offcut-mp4/src/edit_list.rs`, `crates/offcut-text/src/cardinals.rs` and the five `docs/v3/` files with one-line purposes. `check-file-tree.mjs` passes with the two source files listed and absent (known issue 2), so they stay listed. **V2 §27 copy-back into TS, the 16 unapplied decisions**, each cited as "(v2implementation D-n)": §2 a paragraph "Edges added in V2" (D-27, D-58); §7 the `offcut-scene` to `offcut-text` edge under the crate diagram (D-28) and the two use-case pairs in the circularity guards (D-58); §10.1 the `MINTING` files that may cast to a brand (D-59, V2's seven; D-103 adds an eighth at Prompt 62); §10.8 the signed bytes of the token (D-25); §5 the extra binding surface of `media_api.rs` and `profile_api.rs` (D-53); §15.1 which duration the audio is aligned to (D-34); §16.1 `model_version` is the `modelId` (D-50); §16.4 and §24.6 the versioned `/ort/<runtime version>/` path (D-37); §17.1 the 9-byte event-id layout (D-47) and where `Token` lives, with the split "$12" ",000" form (D-42); §19.2 the display-list types and `JsRandomAccess` (D-29, D-30); §20.1 `DemuxerHandle` (D-30); §21.1 and §21.2 where the bitrates and the Preview profile live (D-26); §21.4 and INV-11 the live-frame counter in every build (D-45); §32 where stage timings are measured (D-35).
- **G 1.4.** `.gitignore` gains `fixtures/.gen/`, `corpus/.clips/`, `corpus/.results/` under one comment line; the `.env*` line is untouched.

**Differs from the prompt, the guide or the plan.**

- `Cargo.lock` changes in Prompt 61 (the `rustls` advisories, above). Not in the prompt's file list.
- The upstream of `v3-build` is unset (G 0.3, above).
- `.gitignore`'s three lines have a comment line above them, as every other block of the file has.
- The `v2` tag was created by the agent at the founder's request (the prompt lists it as a Human step); it is not pushed.
- The first keyed media run of this prompt read the test key with a trailing comment of its `ci.yml` line (`# gitleaks:allow`), 61 characters instead of 44: the build's `env.ts` refuses such a key when the module loads, so the app did not start and the setup case "Chrome on this machine can run Offcut" timed out (30 cases not run). The key was read again as the base64 token alone and the run repeated; the count above is the second run's. No file changed for it.

**Checked.** SAB gate, after every change of this prompt: no `zz` file, no test key in the environment, `pnpm check` exit 0 (`cargo deny` green with `rustls` 0.23.46), `cargo test --workspace` 262 passed, 0 failed, 2 ignored, Vitest 92, `pnpm build` exit 0 (WASM sizes unchanged: 459,281 and 6,033,574 bytes), `pnpm e2e` 12 passed; the frozen check of G 1.7 and the check for tracked private files print nothing. Counts: 262 (2 ignored), 92, 12, media 30 of 30, none below the baseline. The validation commands of the prompt: `git merge-base v3-build origin/main` gives `8a271a9`; `check-file-tree.mjs` exit 0; with `fixtures/.gen/probe`, `corpus/.clips/probe`, `corpus/.results/probe` (and `fixtures/.gen/x.mp4`, G 1.9 row 3) touched, `git status --short --untracked-files=all` shows none of them, nor `testclips/`; `grep -cE "^## " docs/v3/experiments.md` gives 14.

**Pinned versions.** No new dependency is committed. `rustls` 0.23.46 in `Cargo.lock` (a patch update of a locked transitive crate).

**Found, not fixed.** None besides the open items and known issues above.

### Outside a prompt (2026-10-16): effort ratings in `coding-promptsv3.md`

**Done.** At the founder's request, each of the 30 prompts gained one line under its heading, `**Effort: High.**` or `**Effort: Medium.**` with its reason (Prompt 61: `Done`). A paragraph after "Three readings are gates" explains the key: 20 High (63–73, 75–79, 81, 84, 85, 89), 9 Medium (62, 74, 80, 82, 83, 86, 87, 88, 90). Prompts 74 and 88 also say when to switch to High. The ratings are for the human choosing the agent's reasoning effort. No instruction, requirement, file list or acceptance criterion of any prompt changed.

**Checked.** `grep -c '^\*\*Effort: High'` gives 20 and `Medium` gives 9 (plus 61's `Done`). Docs only: no code, test or count changed.

---

## 2026-10-16 - Prompt 62 (V3 02/30): crates, reopened contracts D-83 and D-103, corpus manifest

**Done.**

- **G 1.5, crates.** `grep -rn "unused_crate_dependencies" Cargo.toml crates/*/src/lib.rs clippy.toml` prints nothing, so both crates are added now, not in Prompts 66 and 67. `Cargo.toml` `[workspace.dependencies]`: `nnnoiseless = { version = "0.5.2", default-features = false }`, `ebur128 = { version = "0.1.10", default-features = false }` (the versions of Entry 1). `crates/offcut-dsp/Cargo.toml`: both `{ workspace = true }`, dev `sha2`. `crates/offcut-mp4/Cargo.toml`: dev `sha2`. `crates/offcut-text/Cargo.toml`: dev `proptest` (`std`). `crates/offcut-detect/Cargo.toml`: dev `serde` (`derive`), `serde_json`, `proptest` (`std`). All through the workspace. `Cargo.lock` gains the two crates' trees (Entry 1's G 0.7 list) and changes no version already locked; its 13 removed lines are `"bitflags"` references that now read `"bitflags 2.x"`, because `ebur128` brings `bitflags` 1.3.2 beside it.
- **`deny.toml`: unchanged.** `cargo deny check` gives `advisories ok, bans ok, licenses ok, sources ok` with the two trees: every licence in them is already on the allowlist (§18.1 expected BSD-3-Clause to need adding; it was there since V1).
- **D-83, REOPENED V1 CONTRACT (`web/src/workers/protocol.ts`).** The `event_found` arm of `FeedLine` gains `displayTo?: string`; no other line changes.
- **`web/src/copy/messages.ts`.** `feed.eventFound.from_to` is `(display: string, displayTo: string) => string`: "Found: {display} to {displayTo}", or "Found: {display}" when `displayTo` is empty (type completeness only). Its `satisfies` clause had to change with it: `Record<EventKind, (display: string) => string>` cannot hold a two-argument function, so it is now `Record<Exclude<EventKind, "from_to">, (display: string) => string> & { from_to: (display: string, displayTo: string) => string }`. The three other functions are unchanged.
- **`web/src/ui/components/ProcessingFeed.tsx`** (moved here from S9, G item 4). `eventText` takes a third argument, and the `event_found` line passes `line.displayTo ?? ""`; the `from_to` case calls `found.from_to(display, displayTo)`. The `assertNever` defaults stay. `git grep "eventFound\|from_to("` finds no other caller: `pipeline-preview.spec.ts` uses `number_reveal("")` only.
- **D-103, REOPENED V2 CONTRACT (`web/eslint.config.js`).** One `MINTING` entry, `{ file: "src/persistence/clips-repo.ts", brand: "Bytes", helper: "toBytes" }`, in the form of V2's entries (the cast must be the last `return` of a function declaration `toBytes`). No rule changed. The file it names arrives in Prompt 80.
- **G 1.8.** `corpus/manifest.json`: `founder-01` and `founder-02` with the SHA-256 values of Entry 1 and the tags of §17.6, `expect: "accept"`, `owner: "founder"`. No path, no file name, no person's name. Listed in TS §5 since V1.
- **G 1.9, the three drills.** (1) `crates/offcut-dsp/src/stray.rs` → `check-file-tree: 1 problem ... stray.rs: not in the file tree of TS §5`; (2) `found.from_to(display)` → `tsc`: `ProcessingFeed.tsx(25,20): error TS2554: Expected 2 arguments, but got 1.`; (3) `fixtures/.gen/x.mp4` → `git status --short` shows nothing for it. All three undone.
- **The founder's decision on §23 item 27 (2026-10-16): written emphasis gets `pitch_z = 2.0`.** The founder's words: "a missed animation is fine but a wrong animation makes the edit bad". Of the two options, `pitch_z = 2.0` changes only how a test turns a written `*word*` into prosody (score 0.90, clear of the 0.80 threshold); rounding scores before the threshold would have changed the detector itself and let borderline real candidates (0.795 and up) through, the wrong-event direction. The detector keeps `score >= threshold`. Applied in `docs/v3/v3implementation.md`: D-87, the `sentences` row of §17.5, the "*Workflow* matters." row of §19.11 (now 0.90, from 0.80), and §23 item 27 (now "Decided"). Open item 1 closed; Entry 1's decisions row updated.
- **The `v2` tag was pushed** (`git push origin v2`, at the founder's request; tag object `4b7e7dd`). A tag push runs no workflow: `ci.yml` runs on pull requests and pushes to `main` only. Open item 4 closed.

**Differs from the prompt, the guide or the plan.**

- `deny.toml` is not changed (no licence was missing).
- The `satisfies` type of `messages.feed.eventFound` changes (above); the prompt names only the signature.
- `docs/v3/v3implementation.md` changes (item 27), though not in the prompt's file list: the founder's decision is due before Prompt 64 and Prompts 64, 77 and 79 read the plan.

**Checked.** Validation commands: `cargo build --workspace` finished; `cargo deny check` all four ok; the purity grep is empty; `cargo build -p offcut-dsp --target wasm32-unknown-unknown --release` finished; the manifest prints `1 founder-01,founder-02`; `pnpm check` exit 0. SAB gate: no `zz` file, no test key, `pnpm check` exit 0, `cargo test --workspace` 262 passed, 0 failed, 2 ignored, Vitest 92, `pnpm build` exit 0 (`offcut_core_bg` 459,281 bytes, `offcut_render_bg` 6,033,574: unchanged, since no code uses the new crates yet), `pnpm e2e` 12 passed; the frozen check and the tracked-private check print nothing. Counts equal the baseline.

**Pinned versions.** `nnnoiseless` 0.5.2, `ebur128` 0.1.10 (workspace, `default-features = false`).

**Found, not fixed.** None.

---

## 2026-10-17 - Outside a prompt: the `rustls` fix for `main`

At the founder's request, branch `hotfix/rustls-0.23.46` was cut from `origin/main` (`8a271a9`) and pushed with one commit, `69ce295` "Hotfix: rustls 0.23.46 (RUSTSEC-2026-0346 to 0349)": `Cargo.lock` only, the same line `v3-build` carries since Prompt 61. `cargo deny check advisories` gives `advisories ok` on that tree. The branch's upstream is its own remote branch, never `main`. A push of a branch other than `main` runs no workflow and deploys nothing. Opening the pull request and merging it are the founder's (no `gh` on D1): open item 5. This changelog lives on `v3-build`, so the entry is here; the hotfix commit itself carries no changelog line, because the V2 changelog is closed.

## 2026-10-17 - Outside a prompt: three recording scripts for the founder

At the founder's request: `docs/v3/testvideo_scripts/script1.md` (phone, portrait, all four event kinds), `script2.md` (laptop webcam, mostly the traps X1-X5 and D-102), `script3.md` (phone in a room with a steady fan, a four-second pause and a quiet and a loud passage: the voice chain). Each holds the setup that keeps the clip inside the limits (H.264 not HEVC, 30 fps, at most 1080p, under 90 s, the camera's location tag off: D-100), the words to read with emphasis in capitals, and two tables: the events a person would expect and the traps where a person expects none, each with the rule (D-85, D-101, D-102, D-105, X1-X5). The clips are saved as `testclips/v3script1.mp4` to `v3script3.mp4` (git-ignored). They are optional: D-76 still holds (no recording is needed for V3). Once recorded they can be corpus clips `founder-03` to `founder-05` (D-88) and, being the founder's own words, labeled files (D-112). TS §5 lists the three files.

## 2026-10-17 - Prompt 63 (V3 03/30): fixture generator, manifest and observations

**Done.**

- **`fixtures/manifest.json`** (§17.4 format): 25 entries, 7 `ok_*`, 13 `rej_*`, 5 `speech/*`, in that order; `pcm: true` on the five speech entries only. `expect.accept` uses the field names of `ClipInfo` (`orientation`, `is_vfr`, `display_width`, `display_height`, `rotation`) and §17.4's `duration_ms` and `avg_fps_milli`; `expect.reject` the `RejectReason`. `probe` holds what step 8 checks: `video_codec`, `audio_codec`, `width`, `height`, `frames` (constant-rate files), `duration_ms` (within 50 ms), `video_streams`, `audio_streams`. `rej_truncated.mp4` has no `probe` (ffprobe cannot read it). `tests` names the files of §17.2's "Read by" column and of §19 that read each one. Counts: `25 12 13 5`.
- **`fixtures/gen_fixtures.sh`** (POSIX `sh`, LF): the nine steps of §17.1 with the skeleton of G 2.2. Exit codes 2 (ffmpeg not 9.0.x, `n` prefix accepted), 3 (reference clip missing, unreadable, download failed, or its SHA-256 not starting with the name's 16-hex segment), 4 (a recipe, with its id), 5 (a probe field, with the fixture's name). `SAMPLE_CLIP_PATH` is read with `sed` by the same anchored pattern as `referenceClip()` of `helpers/fixtures.ts`. A download goes to `fixtures/.cache/<path>.part` and is renamed after it completes. Temporary files stay in `fixtures/.gen/.tmp/` and are removed before step 8. Step 3 compares the whole `STAMP` text and checks that every manifest file and every PCM file exists. Step 8 is one `node -e` (G 2.2 allows it) that runs `ffprobe` per file and sets `process.exitCode`. `STAMP` is written to `STAMP.part` and renamed, last.
- **Root `package.json`**: `"fixtures": "sh fixtures/gen_fixtures.sh"`.
- **Recipes**: the 20 rows of §17.2 and the 5 of §17.3. Encoders used: `libx264` (every H.264 file; `rej_h264_444.mp4` with `-profile:v high444 -pix_fmt yuv444p`), `libx265` (`rej_hevc.mov`, no `pools` needed: two runs were identical), `libaom-av1 -cpu-used 8 -row-mt 0` (`rej_av1.mp4`; CI's essentials build has no `libsvtav1`), `libvpx-vp9 -deadline realtime` with `libopus` (`rej_vp9.webm`), `libmp3lame` (`rej_mp3_audio.mp4`), `aac` everywhere else. All are in both the full build of D1 and the essentials build of CI's Windows job (Entry 1), so D1 keeps its full build: open item 8 is closed.
- **G 2.3.** Two runs from clean: all 30 files other than `STAMP` byte-identical ("deterministic"); the third run prints "fixtures are up to date" (3 s). 31 files, 160 MB. The leak loop over every generated file (not only `speech/`) finds no `location`, `ISO6709` or `creation_time` tag. The derived clips: portrait `1080,1920,1800` 60.000 s; landscape `1920,1080,1800` 60.000 s; no events `1280,720,432` 14.400 s; long pause `1280,720,600` 20.000 s; sparse `1280,720,900` 30.000 s: G 2.3's table exactly.
- **G 2.4, observed on D1** (also in `docs/v3/experiments.md`):
  - `ok_vfr.mp4`: 225 frames, video stream 9.966667 s, `avg_frame_rate` 6750/299 = 22.575 fps. Manifest: `duration_ms` 9967 (the value §17.4 printed), `avg_fps_milli` `[22000, 23000]`.
  - `rej_h264_444.mp4`: `High 4:4:4 Predictive`, level 31. **Chrome stable on D1 says `isConfigSupported` is `true` for `avc1.f4001f`, for D-78's fallback `avc1.6e001f` (High 10), and for `avc1.64001f`.** D-78's last branch applies: the `rejections.spec.ts` case is to be `test.fixme` with the reason (Prompt 84), the path is covered by `validate_probe`'s inline test, and the fixture stays for `probe_rejections.rs` (`decode_supported = false`). No High 10 file is made.
  - `rej_silent.mp4` dropped in the V2 app (preview build, model cached): the rejected state appears. The fixture stays 5 s (D-111).
  - Rotation and the disabled first track stay `null` for Prompt 71. Seen with `ffprobe` meanwhile: the display matrix of `ok_rotated_90.mov` reads `rotation=90`; in `ok_two_video_first_disabled.mp4` the 320x180 track has `disposition:default=0` and the 640x360 one `default=1` (whether `tkhd` marks the first disabled is what the demuxer reads in Prompt 71).
  - Both observations were taken with a temporary `web/tests-e2e/zz-observe.spec.ts` and `web/zz-pw.config.ts` (SAB rule 5), deleted afterwards.
- **Failure-path tests.** `probe.width` of `rej_4k.mp4` set to 9999: the changed manifest made the stamp stale, the script rebuilt everything and stopped with exit 5 (`rej_4k.mp4: width is 3840, the manifest says 9999`), and no `STAMP` was written. Undone; the rebuild then passed.

**Differs from the prompt, the guide or the plan.**

- **The two 1080p clips are encoded with 4 x264 threads (`OUTF4`), every other recipe with 1** (D-111 says `-threads 1` everywhere). With one thread the pair took 248 of the first run's 326 s (portrait 146 s, landscape 102 s; decoding and scaling are 5 s of it). x264 is deterministic for a fixed thread count; checked by two clean runs, and by two runs of the portrait recipe alone (equal SHA-256). `superfast` (88 s, still the High profile) was tried and not taken; `ultrafast` would have made these clips Constrained Baseline, which a phone clip is not. The whole run is now 118-166 s, still above the 40 s §17.1 assumes (open item 10).
- **`speech_20s_noevents.mp4` is cut by output frame number**, like the two spliced clips: `fps=30`, then `trim=start_frame=1398:end_frame=1830` (46.600 s, 432 frames), audio `atrim=start=46.600:duration=14.400`. The input seek of G 2.2's pattern (`-ss 46.600 ... -t 14.4`) gave 431 frames: a seek into the VFR source lands just after a frame boundary. The two 60 s clips keep the input seek and give 1800 frames.
- **The spliced clips** (`long_pause`, `sparse`) cut video by output frame number at 30 fps (frames 275-574 and 543-842; 260+210, 872+195, 1406+180, 1663+195, 2074+120), audio by sample-exact seconds, room tone from one `anoisesrc` of 30 s split per piece, everything through `aformat` to 48 kHz stereo `fltp` before `concat`. For `sparse`, §17.3 does not say what video lies under the 0.5 s before the first piece: it is the 0.5 s of `REF` before that piece (frame 260, 8.667 s), so the video under each stretch is `REF` running into the piece placed in it.
- `rej_4k.mp4` uses `-preset ultrafast` instead of `veryfast` (§17.2), so the H.264 flags hold no preset and each recipe adds its own (`FAST` or `ultrafast`), with no duplicate `-preset` option.
- `rej_silent.mp4` uses `-t 5` on the output, because `anullsrc` has no duration of its own. `rej_vp9.webm` sets `-b:a 96k` for Opus.
- `STAMP`'s `ffmpeg` field is the whole version token (`9.0.2-full_build-www.gyan.dev`), not `9.0.2`: a different build of the same version must also force a rebuild.
- D-76's "a real recording in `testclips/` under a TS name is preferred" is not built: the prompt's nine steps do not include it, and a real recording would also fail the white-frame and length checks of step 8. To be added with the manifest change when such a recording exists.
- No `.gitattributes` change: `git ls-files --eol` gives `i/lf w/lf` for the script on D1 (`core.autocrlf=input`). The Windows runner is open item 9.

**Checked.** Validation commands: `pnpm fixtures` exit 0; 31 files; deterministic; up to date on the third run; `25 12 13 5`; `STAMP.generated` 25; `git ls-files --eol fixtures/gen_fixtures.sh` after staging: `i/lf w/lf`. SAB gate: no `zz` file, no test key, `pnpm fixtures` "fixtures are up to date", `pnpm check` exit 0, `cargo test --workspace` 262 passed, 0 failed, 2 ignored, Vitest 92, `pnpm build` exit 0 (WASM sizes unchanged), `pnpm e2e` 12 passed; the frozen check and the tracked-private check print nothing. Counts equal the baseline. `pnpm e2e:media` was not run (the prompt does not ask for it; nothing it covers changed).

**Pinned versions.** No dependency added. ffmpeg 9.0.2 (full build on D1).

**Found, not fixed.** None besides open items 9 and 10.

---

## 2026-10-17 - Outside a prompt: the `rustls` fix is on `main`; `v3-build` has a draft pull request

- The founder opened and merged pull request #4 (`hotfix/rustls-0.23.46`) on 2026-10-16 at 20:52 UTC: `main` is at `02c1578`, and its run passed `ci`, `e2e-media`, `deploy-api` and `deploy-web`. The deployed API runs `rustls` 0.23.46. Open item 5 is closed.
- At the founder's request `v3-build` was pushed (`git push -u origin v3-build`; before it, a scan of the branch found no secret beyond the public test values already on `main`), and the founder opened **draft pull request #5** ("V3 build") into `main`. Its first run, on `f80901f`: `ci` success (8 min 11 s), `e2e-media / media` success, the two deploy jobs skipped; GitHub reports the branch mergeable with no conflict. From now on the agent pushes `v3-build` after each prompt's commit (the founder: CI on every commit) and reads the run before the next prompt. The pull request stays a draft until Prompt 90; merging it deploys.
- **To keep CI green from Prompt 67 on:** the ffmpeg install and `pnpm fixtures` steps of Prompt 87 (D-94) move forward into the first prompt that adds a test reading `fixtures/.gen/` (Prompt 67), with open items 9 and 10.

## 2026-10-17 - Prompt 64 (V3 04/30): labeled set part 1 (schema, w01-w15)

**Done.**

- **The founder's decision on §23 item 27 as applied** (Prompt 62): a written emphasis gets `energy_z = 2.0`, `pitch_z = 2.0`; no rounding in `lib.rs`. `SCHEMA.md` states it in the `sentences` row.
- **`fixtures/labeled/SCHEMA.md`**: the format of §17.5 and every row of its field table (`sentences`, word timings, `words`, `prosody`, `expected`, `kind`, `say`, values, `tags`, `edits`, `notes`), the matching rules (normalisation, true positive), the null-data rule, the unit vocabulary of the prompt (`count:<noun>` with the noun as spoken) and the tags vocabulary with a one-line meaning each. Two conventions the plan leaves open are fixed there: **which words a `say` covers** per kind (NumberReveal the quantity with its unit words; FromTo from "from", or from the first quantity, to the end of the second; ListReveal from the announcer's count word, or the first marker, to the end of the last item; KeywordPop the one word), so that every label puts its anchors the same way; and **edits**: word indices count the words of all sentences together from 0, `say` names the words as written, and the expected values are those of the edited text (D-109).
- **`w01-saas-pricing` to `w15-podcast`**, the rows `w01`-`w15` of §17.5's file plan, 5 to 8 sentences each, `frozen` 2026-10-17, `words` and `prosody` null, `edits` only in `w11` (`{ "21": "$4,200" }`: `$3,200` corrected to `$4,200`; the expected NumberReveal is 4,200 on that word). The sentences were written first, then the labels, by what a person would expect; no detector file was opened.
- What the half covers, by the G 2.6 checker: 15 files, 2 without events (`w12`, `w14`), 0 problems. Kinds: NumberReveal 8, FromTo 7, KeywordPop 5, ListReveal 4. Tags: `from_frame` 3, `change_verb` 4, `announcer` 2, `marker_only` 2, `X2` 3, `X5` 2, `X3` 1, `edit_number` 1. Each row's events, traps and edits are as the plan states.

**Differs from the prompt or the plan.**

- **Change-verb FromTos** are written as two quantities on both sides of a verb with "to" ("was 26 percent, and then it went up to 52 percent", "$3,000 a month, and we cut it to $1,100", "$60 an hour, then I raised it to $95", "9 hours, and the rewrite brought it down to 2 hours"), the natural spoken form of TS §17.4's "went … to" frame without "from".
- `w07`'s "4K camera" carries no tag: the tag vocabulary has none for it. `notes` says why it is no event.
- Numbers that are neither a planned event nor a planned trap were kept out of the sentences ("for years" rather than "for three years"), so that each file holds exactly what its row says.
- A file holds its `edits` after `prosody` and before `expected`; §17.5's example has no `edits`.

**Checked.** Every file parses; the G 2.6 checker prints `15 files, 2 without events, 0 problems`; a misspelled `say` ("painfull" in `w09`) gave `say not found: painfull` and was undone; `git ls-files crates/offcut-detect/src` lists `config.rs event_id.rs lib.rs number.rs` only; `check-file-tree.mjs` passes. SAB gate: no `zz` file, no test key, `pnpm fixtures` "fixtures are up to date", `pnpm check` exit 0, `cargo test --workspace` 262 passed, 0 failed, 2 ignored, Vitest 92, `pnpm build` exit 0 (WASM sizes unchanged); the frozen check and the tracked-private check print nothing. **`pnpm e2e` failed twice and then passed 12 of 12** with no change between the runs: in the two failed runs six cases of `landing.spec.ts` timed out at 30 s with the page fully drawn (`page.goto` waits for the page's `load`, which follows the asset host). The asset host answered in about 1 s when measured afterwards, and the third run passed in 33.9 s. Read as a slow network on D1 at that moment, not a fault: this prompt changes nothing under `web/`. Counts equal the baseline.

**Frozen.** From this commit on, a change to a label of these files needs a line here naming the file, the label and the reason; "the detector disagrees" is not a reason (D-87).

**Found, not fixed.** None.
