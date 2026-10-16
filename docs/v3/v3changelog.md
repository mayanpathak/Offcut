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
| 5 | **`main`, and so the deployed API, still locks `rustls` 0.23.45**, which RUSTSEC-2026-0346 to 0349 name (Entry 1). `v3-build` locks 0.23.46 from Prompt 61 on; `main` gets it with V3's merge (Prompt 90). A fix sooner is a one-line `cargo update -p rustls` on a branch of `main` and a pull request; the founder decides. Until then any push to `main` fails `cargo deny check` in `ci` | Human | Before any push to `main` |
| 6 | The `buildplan.md` half of V2 §27's copy-back list (the schedule moves of D-18, D-20, D-21, D-23, D-24, D-33) is not applied. Prompt 61 may change `technicalspec.md` only | Agent | Prompt 89 (copy-back) |
| 7 | **The Linux ffmpeg URL is short-lived.** BtbN deletes its dated autobuild releases after about six weeks (on 2026-10-16 the oldest one listed was of 2026-08-31), so the URL of Entry 1 stops working around the end of November 2026. Prompt 87 writes it into `ci.yml`: check it then, and choose there between a newer build of the same day and a copy kept on the asset host | Agent | Prompt 87 |
| 8 | **D1's ffmpeg is the "full" build; CI's Windows job installs the "essentials" build** of the same version 9.0.2 (`e2e-media.yml`). Both have every encoder and filter G 0.2 asks for except that essentials has no `libsvtav1`: the AV1 recipe must use `libaom-av1`. G 0.2 asks for one archive on both machines; Prompt 63 decides whether `pnpm fixtures` on D1 runs the essentials binary (checked to run on D1, see Entry 1) so that the observations of G 2.4 are the runner's | Agent | Prompt 63 |

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
